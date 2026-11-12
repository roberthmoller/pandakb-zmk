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

static void draw_idle_content(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);
    lv_canvas_draw_text(canvas, 0, 52, CANVAS_WIDTH, &label_dsc, "PK");
    lv_canvas_draw_text(canvas, 0, 72, CANVAS_WIDTH, &label_dsc, "idle");
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

static void draw_lol_content(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    const struct pandakb_widget_lol *g = &screen_state.status.u.lol;
    lv_canvas_draw_text(canvas, 0, 40, CANVAS_WIDTH, &label_dsc, "LoL");

    if (g->flags & PANDAKB_STATUS_FLAG_VALID) {
        char kda[16];
        char cs[16];
        char lvl[16];
        snprintk(kda, sizeof(kda), "%u/%u/%u", g->kills, g->deaths, g->assists);
        snprintk(cs, sizeof(cs), "CS%u", pandakb_lol_cs(g));
        snprintk(lvl, sizeof(lvl), "Lv%u", g->level);
        lv_canvas_draw_text(canvas, 0, 58, CANVAS_WIDTH, &label_dsc, kda);
        lv_canvas_draw_text(canvas, 0, 78, CANVAS_WIDTH, &label_dsc, cs);
        lv_canvas_draw_text(canvas, 0, 98, CANVAS_WIDTH, &label_dsc, lvl);
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

    widget_game_badge_init();
    widget_badge_battery_init();
    widget_badge_peripheral_init();

    redraw_screen();
    return screen;
}
