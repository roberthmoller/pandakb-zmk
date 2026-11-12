/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <string.h>

#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/logging/log.h>

#include <lvgl.h>

#include <zmk/battery.h>
#include <zmk/display.h>
#include <zmk/event_manager.h>
#include <zmk/events/battery_state_changed.h>
#include <zmk/events/split_peripheral_status_changed.h>
#include <zmk/events/usb_conn_state_changed.h>
#include <zmk/split/bluetooth/peripheral.h>
#include <zmk/usb.h>

#include <pandakb/badge.h>

#include "art/art.h"

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

/* Portrait logical size for a 128x32 OLED mounted at -90°. */
#define CANVAS_WIDTH 32
#define CANVAS_HEIGHT 128

#define LVGL_BACKGROUND lv_color_white()
#define LVGL_FOREGROUND lv_color_black()

static lv_obj_t *canvas;
static lv_color_t cbuf[CANVAS_HEIGHT * CANVAS_HEIGHT];

struct badge_screen_state {
    struct pandakb_badge_status status;
    uint8_t battery;
    bool charging;
    bool connected;
};

static struct badge_screen_state screen_state;

/* Local overlay so a one-shot HID event still gets ~1.5s even after flags clear. */
static uint8_t lol_flash_event;
static uint8_t lol_flash_seen;
static uint8_t lol_flash_phase;
static lv_timer_t *lol_flash_timer;
static lv_timer_t *lol_flash_anim_timer;

/* Volume change briefly takes the whole mode body (macOS-style HUD). */
static bool vol_flash_active;
static bool vol_flash_have_prev;
static uint8_t vol_flash_level;
static uint8_t vol_last_level;
static uint8_t vol_prev_mode;
static lv_timer_t *vol_flash_timer;

#define LOL_FLASH_MS 1600
#define LOL_FLASH_ANIM_MS 110
#define LOL_FLASH_BODY_Y 36
#define LOL_FLASH_BODY_H 92
#define VOL_FLASH_MS 1600
#define VOL_FLASH_BODY_Y 36
#define VOL_FLASH_BODY_H (CANVAS_HEIGHT - VOL_FLASH_BODY_Y)
/* Full-body vertical meter (90° from the media horizontal bar). */
#define VOL_BAR_X 4
#define VOL_BAR_Y VOL_FLASH_BODY_Y
#define VOL_BAR_W (CANVAS_WIDTH - 8)
#define VOL_BAR_H VOL_FLASH_BODY_H
#define BUDDY_MS 450
#define BUDDY_Y (LOL_FLASH_BODY_Y + (LOL_FLASH_BODY_H - 32) / 2)

static lv_timer_t *buddy_timer;
static uint8_t buddy_frame;

static const lv_img_dsc_t *buddy_frames[] = {
    &buddy_0,
    &buddy_1,
    &buddy_2,
    &buddy_3,
};

static void redraw_screen(void);
static void buddy_anim_sync(void);

static void init_label_dsc(lv_draw_label_dsc_t *label_dsc, lv_color_t color, const lv_font_t *font,
                           lv_text_align_t align) {
    lv_draw_label_dsc_init(label_dsc);
    label_dsc->color = color;
    label_dsc->font = font;
    label_dsc->align = align;
}

static void init_rect_dsc(lv_draw_rect_dsc_t *rect_dsc, lv_color_t bg_color) {
    lv_draw_rect_dsc_init(rect_dsc);
    rect_dsc->bg_color = bg_color;
}

static void copy_padded(char *dst, size_t dst_len, const uint8_t *src, size_t src_len) {
    size_t n = src_len < (dst_len - 1) ? src_len : (dst_len - 1);
    memcpy(dst, src, n);
    dst[n] = '\0';
    for (size_t i = 0; i < n; i++) {
        if (dst[i] < 0x20 || dst[i] > 0x7e) {
            dst[i] = '?';
        }
    }
}

/* Match nice_oled / nice_view: draw portrait, then rotate onto the landscape panel. */
static void rotate_canvas(void) {
    static lv_color_t cbuf_tmp[CANVAS_HEIGHT * CANVAS_HEIGHT];
    memcpy(cbuf_tmp, cbuf, sizeof(cbuf_tmp));

    lv_img_dsc_t img;
    img.data = (void *)cbuf_tmp;
    img.header.cf = LV_IMG_CF_TRUE_COLOR;
    img.header.always_zero = 0;
    img.header.reserved = 0;
    img.header.w = CANVAS_HEIGHT;
    img.header.h = CANVAS_HEIGHT;

    lv_canvas_fill_bg(canvas, LVGL_BACKGROUND, LV_OPA_COVER);
    /* +90° compensates for the OLED mounted at -90°. */
    lv_canvas_transform(canvas, &img, 900, LV_IMG_ZOOM_NONE, -1, 0, CANVAS_HEIGHT / 2,
                        CANVAS_HEIGHT / 2, false);
}

static void draw_connection(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    const char *symbol;
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
    if (screen_state.charging) {
        symbol = LV_SYMBOL_USB;
    } else
#endif
    {
        symbol = screen_state.connected ? LV_SYMBOL_WIFI : LV_SYMBOL_CLOSE;
    }

    lv_canvas_draw_text(canvas, 0, 2, CANVAS_WIDTH, &label_dsc, symbol);
}

static void draw_battery(void) {
    lv_draw_rect_dsc_t rect_fg;
    lv_draw_rect_dsc_t rect_bg;
    init_rect_dsc(&rect_fg, LVGL_FOREGROUND);
    init_rect_dsc(&rect_bg, LVGL_BACKGROUND);

    lv_canvas_draw_rect(canvas, 2, 18, 26, 12, &rect_fg);
    lv_canvas_draw_rect(canvas, 3, 19, 24, 10, &rect_bg);
    lv_canvas_draw_rect(canvas, 28, 21, 2, 6, &rect_fg);

    uint8_t fill = (screen_state.battery > 100) ? 100 : screen_state.battery;
    uint8_t fill_w = (uint8_t)((fill * 22) / 100);
    if (fill_w > 0) {
        lv_canvas_draw_rect(canvas, 4, 20, fill_w, 8, &rect_fg);
    }

#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
    if (screen_state.charging) {
        lv_draw_label_dsc_t bolt_dsc;
        init_label_dsc(&bolt_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);
        lv_canvas_draw_text(canvas, 0, 17, CANVAS_WIDTH, &bolt_dsc, LV_SYMBOL_CHARGE);
    }
#endif
}

static void buddy_timer_cb(lv_timer_t *timer) {
    ARG_UNUSED(timer);
    if (screen_state.status.mode != PANDAKB_WIDGET_IDLE || vol_flash_active) {
        return;
    }
    buddy_frame = (uint8_t)((buddy_frame + 1) % 4);
    redraw_screen();
}

static void buddy_anim_sync(void) {
    if (buddy_timer == NULL) {
        return;
    }
    bool run = screen_state.status.mode == PANDAKB_WIDGET_IDLE && !vol_flash_active;
    if (run) {
        lv_timer_resume(buddy_timer);
    } else {
        lv_timer_pause(buddy_timer);
    }
}

static void draw_idle_content(void) {
    lv_draw_img_dsc_t img_dsc;
    lv_draw_img_dsc_init(&img_dsc);
    lv_canvas_draw_img(canvas, 0, BUDDY_Y, buddy_frames[buddy_frame], &img_dsc);
}

static void vol_flash_timer_cb(lv_timer_t *timer) {
    if (vol_flash_timer == timer) {
        vol_flash_timer = NULL;
    }
    lv_timer_del(timer);
    vol_flash_active = false;
    buddy_anim_sync();
    redraw_screen();
}

static void vol_flash_arm(uint8_t volume) {
    vol_flash_level = volume > 100 ? 100 : volume;
    vol_flash_active = true;
    buddy_anim_sync();

    if (vol_flash_timer != NULL) {
        lv_timer_del(vol_flash_timer);
        vol_flash_timer = NULL;
    }
    vol_flash_timer = lv_timer_create(vol_flash_timer_cb, VOL_FLASH_MS, NULL);
}

static void draw_volume_flash(uint8_t volume) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    lv_draw_rect_dsc_t rect_fg;
    lv_draw_rect_dsc_t rect_bg;
    init_rect_dsc(&rect_fg, LVGL_FOREGROUND);
    init_rect_dsc(&rect_bg, LVGL_BACKGROUND);

    uint8_t level = volume > 100 ? 100 : volume;

    /* Full-body vertical meter — outline, then fill upward from the bottom. */
    lv_canvas_draw_rect(canvas, VOL_BAR_X, VOL_BAR_Y, VOL_BAR_W, VOL_BAR_H, &rect_fg);
    lv_canvas_draw_rect(canvas, VOL_BAR_X + 1, VOL_BAR_Y + 1, VOL_BAR_W - 2, VOL_BAR_H - 2, &rect_bg);

    uint8_t inner_h = VOL_BAR_H - 4;
    uint8_t fill_h = (uint8_t)((level * inner_h) / 100);
    if (fill_h > 0) {
        lv_canvas_draw_rect(canvas, VOL_BAR_X + 2, VOL_BAR_Y + 2 + (inner_h - fill_h), VOL_BAR_W - 4,
                            fill_h, &rect_fg);
    }

    char vol_txt[8];
    snprintk(vol_txt, sizeof(vol_txt), "%u", level);
    /* Backdrop so the percent stays readable over fill or empty. */
    lv_canvas_draw_rect(canvas, 2, VOL_BAR_Y + VOL_BAR_H - 18, CANVAS_WIDTH - 4, 14, &rect_bg);
    lv_canvas_draw_text(canvas, 0, VOL_BAR_Y + VOL_BAR_H - 16, CANVAS_WIDTH, &label_dsc, vol_txt);
}

static void draw_media_content(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    lv_draw_rect_dsc_t rect_fg;
    lv_draw_rect_dsc_t rect_bg;
    init_rect_dsc(&rect_fg, LVGL_FOREGROUND);
    init_rect_dsc(&rect_bg, LVGL_BACKGROUND);

    const struct pandakb_widget_media *m = &screen_state.status.u.media;
    const char *symbol = LV_SYMBOL_STOP;

    switch (m->playback) {
    case PANDAKB_MEDIA_PLAYING:
        symbol = LV_SYMBOL_PLAY;
        break;
    case PANDAKB_MEDIA_PAUSED:
        symbol = LV_SYMBOL_PAUSE;
        break;
    default:
        symbol = LV_SYMBOL_STOP;
        break;
    }

    lv_canvas_draw_text(canvas, 0, 40, CANVAS_WIDTH, &label_dsc, symbol);

    char title[PANDAKB_WIDGET_TITLE_LEN + 1];
    copy_padded(title, sizeof(title), m->title, PANDAKB_WIDGET_TITLE_LEN);
    if (title[0] != '\0') {
        lv_canvas_draw_text(canvas, 0, 56, CANVAS_WIDTH, &label_dsc, title);
    }

    char vol_txt[8];
    snprintk(vol_txt, sizeof(vol_txt), "%u", m->volume);
    lv_canvas_draw_text(canvas, 0, 78, CANVAS_WIDTH, &label_dsc, "Vol");
    lv_canvas_draw_text(canvas, 0, 92, CANVAS_WIDTH, &label_dsc, vol_txt);

    lv_canvas_draw_rect(canvas, 6, 110, 20, 8, &rect_fg);
    lv_canvas_draw_rect(canvas, 7, 111, 18, 6, &rect_bg);
    uint8_t fill_w = (uint8_t)((m->volume * 16) / 100);
    if (fill_w > 0) {
        lv_canvas_draw_rect(canvas, 8, 112, fill_w, 4, &rect_fg);
    }
}

static const char *lol_flash_label(uint8_t event) {
    switch (event) {
    case PANDAKB_LOL_EVENT_KILL:
        return "+";
    case PANDAKB_LOL_EVENT_DOUBLE:
        return "x2";
    case PANDAKB_LOL_EVENT_TRIPLE:
        return "x3";
    case PANDAKB_LOL_EVENT_QUADRA:
        return "x4";
    case PANDAKB_LOL_EVENT_PENTA:
        return "x5";
    case PANDAKB_LOL_EVENT_ACE:
        return "ACE";
    case PANDAKB_LOL_EVENT_FIRST_BLOOD:
        return "FB";
    default:
        return NULL;
    }
}

static const char *lol_flash_sublabel(uint8_t event) {
    switch (event) {
    case PANDAKB_LOL_EVENT_KILL:
        return "KILL";
    case PANDAKB_LOL_EVENT_DOUBLE:
        return "DBL";
    case PANDAKB_LOL_EVENT_TRIPLE:
        return "TRI";
    case PANDAKB_LOL_EVENT_QUADRA:
        return "QUAD";
    case PANDAKB_LOL_EVENT_PENTA:
        return "PENTA";
    case PANDAKB_LOL_EVENT_ACE:
        return "TEAM";
    case PANDAKB_LOL_EVENT_FIRST_BLOOD:
        return "1ST";
    default:
        return NULL;
    }
}

static uint8_t lol_flash_streak(uint8_t event) {
    switch (event) {
    case PANDAKB_LOL_EVENT_KILL:
        return 1;
    case PANDAKB_LOL_EVENT_DOUBLE:
        return 2;
    case PANDAKB_LOL_EVENT_TRIPLE:
        return 3;
    case PANDAKB_LOL_EVENT_QUADRA:
        return 4;
    case PANDAKB_LOL_EVENT_PENTA:
        return 5;
    case PANDAKB_LOL_EVENT_ACE:
        return 5;
    case PANDAKB_LOL_EVENT_FIRST_BLOOD:
        return 1;
    default:
        return 0;
    }
}

static void lol_flash_stop_anim(void) {
    if (lol_flash_anim_timer != NULL) {
        lv_timer_del(lol_flash_anim_timer);
        lol_flash_anim_timer = NULL;
    }
    lol_flash_phase = 0;
}

static void lol_flash_timer_cb(lv_timer_t *timer) {
    if (lol_flash_timer == timer) {
        lol_flash_timer = NULL;
    }
    lv_timer_del(timer);
    lol_flash_stop_anim();
    lol_flash_event = 0;
    redraw_screen();
}

static void lol_flash_anim_cb(lv_timer_t *timer) {
    ARG_UNUSED(timer);
    if (lol_flash_event == PANDAKB_LOL_EVENT_NONE) {
        return;
    }
    lol_flash_phase = (uint8_t)(lol_flash_phase + 1);
    redraw_screen();
}

static void lol_flash_arm(uint8_t event) {
    if (event == PANDAKB_LOL_EVENT_NONE) {
        lol_flash_seen = PANDAKB_LOL_EVENT_NONE;
        return;
    }
    if (event == lol_flash_seen) {
        return;
    }

    lol_flash_seen = event;
    lol_flash_event = event;
    lol_flash_phase = 0;

    if (lol_flash_timer != NULL) {
        lv_timer_del(lol_flash_timer);
        lol_flash_timer = NULL;
    }
    lol_flash_stop_anim();
    lol_flash_timer = lv_timer_create(lol_flash_timer_cb, LOL_FLASH_MS, NULL);
    lol_flash_anim_timer = lv_timer_create(lol_flash_anim_cb, LOL_FLASH_ANIM_MS, NULL);
}

/* Thin lit frame — keeps black fill (no inverted white flash). */
static void draw_lol_frame(bool thick) {
    lv_draw_rect_dsc_t fg;
    init_rect_dsc(&fg, LVGL_FOREGROUND);

    const int x = 1;
    const int y = LOL_FLASH_BODY_Y;
    const int w = CANVAS_WIDTH - 2;
    const int h = LOL_FLASH_BODY_H;
    const int t = thick ? 2 : 1;

    lv_canvas_draw_rect(canvas, x, y, w, t, &fg);
    lv_canvas_draw_rect(canvas, x, y + h - t, w, t, &fg);
    lv_canvas_draw_rect(canvas, x, y, t, h, &fg);
    lv_canvas_draw_rect(canvas, x + w - t, y, t, h, &fg);
}

static void draw_lol_corner_brackets(void) {
    lv_draw_rect_dsc_t fg;
    init_rect_dsc(&fg, LVGL_FOREGROUND);

    const int y0 = LOL_FLASH_BODY_Y + 4;
    const int y1 = LOL_FLASH_BODY_Y + LOL_FLASH_BODY_H - 10;
    const int arm = 6;

    /* Top-left / top-right / bottom-left / bottom-right L brackets. */
    lv_canvas_draw_rect(canvas, 3, y0, arm, 1, &fg);
    lv_canvas_draw_rect(canvas, 3, y0, 1, arm, &fg);
    lv_canvas_draw_rect(canvas, CANVAS_WIDTH - 3 - arm, y0, arm, 1, &fg);
    lv_canvas_draw_rect(canvas, CANVAS_WIDTH - 4, y0, 1, arm, &fg);
    lv_canvas_draw_rect(canvas, 3, y1 + arm - 1, arm, 1, &fg);
    lv_canvas_draw_rect(canvas, 3, y1, 1, arm, &fg);
    lv_canvas_draw_rect(canvas, CANVAS_WIDTH - 3 - arm, y1 + arm - 1, arm, 1, &fg);
    lv_canvas_draw_rect(canvas, CANVAS_WIDTH - 4, y1, 1, arm, &fg);
}

/* Streak pips along the left rail; active pip walks with phase. */
static void draw_lol_streak_pips(uint8_t streak) {
    if (streak == 0) {
        return;
    }

    lv_draw_rect_dsc_t on;
    lv_draw_rect_dsc_t clear;
    init_rect_dsc(&on, LVGL_FOREGROUND);
    init_rect_dsc(&clear, LVGL_BACKGROUND);

    const int pip_h = 4;
    const int gap = 3;
    const int total = (int)streak * pip_h + ((int)streak - 1) * gap;
    int y = LOL_FLASH_BODY_Y + (LOL_FLASH_BODY_H - total) / 2;
    uint8_t active = (uint8_t)(lol_flash_phase % streak);

    for (uint8_t i = 0; i < streak; i++) {
        int w = (i == active) ? 5 : 3;
        int x = (i == active) ? 3 : 4;
        lv_canvas_draw_rect(canvas, x, y, w, pip_h, &on);
        /* Future pips stay hollow so the active pip reads as a chase. */
        if (i > active) {
            lv_canvas_draw_rect(canvas, x + 1, y + 1, w - 2, pip_h - 2, &clear);
        }
        y += pip_h + gap;
    }
}

/* Tiny blade / dagger glyph above the label. */
static void draw_lol_blade(void) {
    lv_draw_rect_dsc_t fg;
    init_rect_dsc(&fg, LVGL_FOREGROUND);

    const int cx = CANVAS_WIDTH / 2;
    const int y = LOL_FLASH_BODY_Y + 10;
    /* Tip */
    lv_canvas_draw_rect(canvas, cx - 1, y, 2, 3, &fg);
    /* Blade */
    lv_canvas_draw_rect(canvas, cx - 2, y + 3, 4, 10, &fg);
    /* Guard */
    lv_canvas_draw_rect(canvas, cx - 5, y + 13, 10, 2, &fg);
    /* Hilt */
    lv_canvas_draw_rect(canvas, cx - 1, y + 15, 2, 5, &fg);
}

/* Droplet for first blood. */
static void draw_lol_droplet(void) {
    lv_draw_rect_dsc_t fg;
    init_rect_dsc(&fg, LVGL_FOREGROUND);

    const int cx = CANVAS_WIDTH / 2;
    const int y = LOL_FLASH_BODY_Y + 10;
    lv_canvas_draw_rect(canvas, cx - 1, y, 2, 3, &fg);
    lv_canvas_draw_rect(canvas, cx - 2, y + 3, 4, 3, &fg);
    lv_canvas_draw_rect(canvas, cx - 3, y + 6, 6, 5, &fg);
    lv_canvas_draw_rect(canvas, cx - 2, y + 11, 4, 2, &fg);
}

/* Crown peaks for ace. */
static void draw_lol_crown(void) {
    lv_draw_rect_dsc_t fg;
    init_rect_dsc(&fg, LVGL_FOREGROUND);

    const int y = LOL_FLASH_BODY_Y + 10;
    lv_canvas_draw_rect(canvas, 6, y + 8, 20, 3, &fg);
    lv_canvas_draw_rect(canvas, 6, y + 4, 3, 4, &fg);
    lv_canvas_draw_rect(canvas, 14, y, 4, 8, &fg);
    lv_canvas_draw_rect(canvas, 23, y + 4, 3, 4, &fg);
    /* Pulsing jewel */
    if ((lol_flash_phase & 1u) == 0) {
        lv_canvas_draw_rect(canvas, 15, y + 2, 2, 2, &fg);
    }
}

static void draw_lol_flash(uint8_t event) {
    /* Keep the body black (panel background). Only light ink for motifs. */
    const bool pulse = (lol_flash_phase & 1u) != 0;
    const bool big = event == PANDAKB_LOL_EVENT_PENTA || event == PANDAKB_LOL_EVENT_ACE ||
                     event == PANDAKB_LOL_EVENT_QUADRA;

    draw_lol_frame(big && !pulse);
    if (big) {
        draw_lol_corner_brackets();
    }

    uint8_t streak = lol_flash_streak(event);
    if (streak > 0) {
        draw_lol_streak_pips(streak);
    }

    switch (event) {
    case PANDAKB_LOL_EVENT_FIRST_BLOOD:
        draw_lol_droplet();
        break;
    case PANDAKB_LOL_EVENT_ACE:
        draw_lol_crown();
        break;
    case PANDAKB_LOL_EVENT_KILL:
    case PANDAKB_LOL_EVENT_DOUBLE:
    case PANDAKB_LOL_EVENT_TRIPLE:
        draw_lol_blade();
        break;
    case PANDAKB_LOL_EVENT_QUADRA:
    case PANDAKB_LOL_EVENT_PENTA:
        draw_lol_blade();
        if (!pulse) {
            /* Secondary spark ticks beside the blade tip. */
            lv_draw_rect_dsc_t fg;
            init_rect_dsc(&fg, LVGL_FOREGROUND);
            lv_canvas_draw_rect(canvas, 6, LOL_FLASH_BODY_Y + 12, 2, 2, &fg);
            lv_canvas_draw_rect(canvas, 24, LOL_FLASH_BODY_Y + 14, 2, 2, &fg);
        }
        break;
    default:
        break;
    }

    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    const char *msg = lol_flash_label(event);
    const char *sub = lol_flash_sublabel(event);
    if (msg != NULL) {
        lv_canvas_draw_text(canvas, 0, 72, CANVAS_WIDTH, &label_dsc, msg);
    }
    if (sub != NULL) {
        lv_canvas_draw_text(canvas, 0, 90, CANVAS_WIDTH, &label_dsc, sub);
    }

    /* Scanning underline under the sublabel. */
    lv_draw_rect_dsc_t scan;
    init_rect_dsc(&scan, LVGL_FOREGROUND);
    int scan_x = 4 + (int)(lol_flash_phase % 5) * 5;
    if (scan_x > 22) {
        scan_x = 22;
    }
    lv_canvas_draw_rect(canvas, scan_x, 106, 6, 2, &scan);
}

static void draw_lol_stat_row(lv_draw_label_dsc_t *prefix_dsc, lv_draw_label_dsc_t *value_dsc,
                              int y, const char *prefix, unsigned value) {
    char num[8];
    snprintk(num, sizeof(num), "%u", value);
    /* Letter flush left, count flush right — readable on 32px portrait. */
    lv_canvas_draw_text(canvas, 2, y, CANVAS_WIDTH - 4, prefix_dsc, prefix);
    lv_canvas_draw_text(canvas, 2, y, CANVAS_WIDTH - 4, value_dsc, num);
}

static void draw_lol_content(void) {
    if (lol_flash_event != PANDAKB_LOL_EVENT_NONE) {
        draw_lol_flash(lol_flash_event);
        return;
    }

    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);
    lv_draw_label_dsc_t prefix_dsc;
    init_label_dsc(&prefix_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_LEFT);
    lv_draw_label_dsc_t value_dsc;
    init_label_dsc(&value_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_RIGHT);

    lv_draw_rect_dsc_t rect_fg;
    init_rect_dsc(&rect_fg, LVGL_FOREGROUND);

    const struct pandakb_widget_lol *g = &screen_state.status.u.lol;

    if (g->flags & PANDAKB_STATUS_FLAG_VALID) {
        char line[8];
        /* 32px portrait: prefix left (k/d/a/c), value right — avoids "10/2/0" overflow. */
        draw_lol_stat_row(&prefix_dsc, &value_dsc, 38, "k", g->kills);
        draw_lol_stat_row(&prefix_dsc, &value_dsc, 52, "d", g->deaths);
        draw_lol_stat_row(&prefix_dsc, &value_dsc, 66, "a", g->assists);

        lv_canvas_draw_rect(canvas, 8, 82, 16, 1, &rect_fg);

        draw_lol_stat_row(&prefix_dsc, &value_dsc, 86, "c", pandakb_lol_cs(g));

        snprintk(line, sizeof(line), "L%u", g->level);
        lv_canvas_draw_text(canvas, 0, 108, CANVAS_WIDTH, &label_dsc, line);
    } else {
        lv_canvas_draw_text(canvas, 0, 70, CANVAS_WIDTH, &label_dsc, "...");
    }
}

static void draw_ide_content(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    const struct pandakb_widget_ide *ide = &screen_state.status.u.ide;
    char tag[PANDAKB_WIDGET_TAG_LEN + 1];
    copy_padded(tag, sizeof(tag), ide->tag, PANDAKB_WIDGET_TAG_LEN);

    lv_canvas_draw_text(canvas, 0, 40, CANVAS_WIDTH, &label_dsc, "IDE");
    if (tag[0] != '\0') {
        lv_canvas_draw_text(canvas, 0, 56, CANVAS_WIDTH, &label_dsc, tag);
    }

    const char *build = "idle";
    switch (ide->build) {
    case PANDAKB_IDE_BUILDING:
        build = "bld";
        break;
    case PANDAKB_IDE_OK:
        build = "ok";
        break;
    case PANDAKB_IDE_FAIL:
        build = "fail";
        break;
    default:
        break;
    }
    lv_canvas_draw_text(canvas, 0, 74, CANVAS_WIDTH, &label_dsc, build);

    char errs[12];
    char warns[12];
    snprintk(errs, sizeof(errs), "E%u", ide->errors);
    snprintk(warns, sizeof(warns), "W%u", ide->warnings);
    lv_canvas_draw_text(canvas, 0, 92, CANVAS_WIDTH, &label_dsc, errs);
    lv_canvas_draw_text(canvas, 0, 108, CANVAS_WIDTH, &label_dsc, warns);
}

static void draw_mode_content(void) {
    /* Volume HUD takes the whole body while armed (any mode). */
    if (vol_flash_active) {
        draw_volume_flash(vol_flash_level);
        return;
    }

    switch (screen_state.status.mode) {
    case PANDAKB_WIDGET_MEDIA:
        draw_media_content();
        break;
    case PANDAKB_WIDGET_LOL:
        draw_lol_content();
        break;
    case PANDAKB_WIDGET_IDE:
        draw_ide_content();
        break;
    case PANDAKB_WIDGET_IDLE:
    default:
        draw_idle_content();
        break;
    }
}

static void redraw_screen(void) {
    if (canvas == NULL) {
        return;
    }

    lv_draw_rect_dsc_t rect_bg;
    init_rect_dsc(&rect_bg, LVGL_BACKGROUND);
    lv_canvas_draw_rect(canvas, 0, 0, CANVAS_HEIGHT, CANVAS_HEIGHT, &rect_bg);

    draw_connection();
    draw_battery();
    draw_mode_content();
    rotate_canvas();
}

static void badge_screen_update_cb(struct badge_screen_state state) {
    screen_state.status = state.status;
    if (state.status.mode == PANDAKB_WIDGET_LOL) {
        lol_flash_arm(pandakb_lol_event(&state.status.u.lol));
    } else {
        lol_flash_arm(PANDAKB_LOL_EVENT_NONE);
        lol_flash_event = PANDAKB_LOL_EVENT_NONE;
        if (lol_flash_timer != NULL) {
            lv_timer_del(lol_flash_timer);
            lol_flash_timer = NULL;
        }
        lol_flash_stop_anim();
    }

    /* MEDIA: volume delta, or entering MEDIA (Idle/LoL interrupted for HUD). */
    if (state.status.mode == PANDAKB_WIDGET_MEDIA) {
        uint8_t vol = state.status.u.media.volume > 100 ? 100 : state.status.u.media.volume;
        bool entered_media = vol_prev_mode != PANDAKB_WIDGET_MEDIA;
        if (entered_media || (vol_flash_have_prev && vol != vol_last_level)) {
            vol_flash_arm(vol);
        }
        vol_last_level = vol;
        vol_flash_have_prev = true;
    }
    vol_prev_mode = state.status.mode;
    buddy_anim_sync();

    redraw_screen();
}

static struct badge_screen_state badge_screen_get_state(const zmk_event_t *eh) {
    const struct pandakb_badge_state_changed *ev = as_pandakb_badge_state_changed(eh);
    struct badge_screen_state state = screen_state;
    if (ev != NULL) {
        state.status = ev->status;
    } else {
        state.status = *pandakb_badge_get();
    }
    return state;
}

ZMK_DISPLAY_WIDGET_LISTENER(widget_game_badge, struct badge_screen_state, badge_screen_update_cb,
                            badge_screen_get_state)
ZMK_SUBSCRIPTION(widget_game_badge, pandakb_badge_state_changed);

struct battery_status_state {
    uint8_t level;
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
    bool usb_present;
#endif
};

static void battery_status_update_cb(struct battery_status_state state) {
    screen_state.battery = state.level;
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
    screen_state.charging = state.usb_present;
#endif
    redraw_screen();
}

static struct battery_status_state battery_status_get_state(const zmk_event_t *eh) {
    const struct zmk_battery_state_changed *ev = as_zmk_battery_state_changed(eh);
    return (struct battery_status_state){
        .level = (ev != NULL) ? ev->state_of_charge : zmk_battery_state_of_charge(),
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
        .usb_present = zmk_usb_is_powered(),
#endif
    };
}

ZMK_DISPLAY_WIDGET_LISTENER(widget_badge_battery, struct battery_status_state,
                            battery_status_update_cb, battery_status_get_state)
ZMK_SUBSCRIPTION(widget_badge_battery, zmk_battery_state_changed);
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
ZMK_SUBSCRIPTION(widget_badge_battery, zmk_usb_conn_state_changed);
#endif

struct peripheral_status_state {
    bool connected;
};

static void peripheral_status_update_cb(struct peripheral_status_state state) {
    screen_state.connected = state.connected;
    redraw_screen();
}

static struct peripheral_status_state peripheral_status_get_state(const zmk_event_t *_eh) {
    ARG_UNUSED(_eh);
    return (struct peripheral_status_state){.connected = zmk_split_bt_peripheral_is_connected()};
}

ZMK_DISPLAY_WIDGET_LISTENER(widget_badge_peripheral, struct peripheral_status_state,
                            peripheral_status_update_cb, peripheral_status_get_state)
ZMK_SUBSCRIPTION(widget_badge_peripheral, zmk_split_peripheral_status_changed);

lv_obj_t *zmk_display_status_screen(void) {
    lv_obj_t *screen = lv_obj_create(NULL);

    canvas = lv_canvas_create(screen);
    lv_obj_align(canvas, LV_ALIGN_TOP_LEFT, 0, 0);
    lv_canvas_set_buffer(canvas, cbuf, CANVAS_HEIGHT, CANVAS_HEIGHT, LV_IMG_CF_TRUE_COLOR);

    screen_state.status = *pandakb_badge_get();
    screen_state.battery = zmk_battery_state_of_charge();
#if IS_ENABLED(CONFIG_USB_DEVICE_STACK)
    screen_state.charging = zmk_usb_is_powered();
#endif
    screen_state.connected = zmk_split_bt_peripheral_is_connected();
    vol_prev_mode = screen_state.status.mode;
    if (screen_state.status.mode == PANDAKB_WIDGET_MEDIA) {
        vol_last_level = screen_state.status.u.media.volume > 100
                             ? 100
                             : screen_state.status.u.media.volume;
        vol_flash_have_prev = true;
    }

    buddy_timer = lv_timer_create(buddy_timer_cb, BUDDY_MS, NULL);
    buddy_anim_sync();

    widget_game_badge_init();
    widget_badge_battery_init();
    widget_badge_peripheral_init();

    redraw_screen();
    return screen;
}
