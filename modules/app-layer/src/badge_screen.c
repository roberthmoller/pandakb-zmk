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

static void draw_media_content(void) {
    lv_draw_label_dsc_t label_dsc;
    init_label_dsc(&label_dsc, LVGL_FOREGROUND, &lv_font_montserrat_12, LV_TEXT_ALIGN_CENTER);

    lv_draw_rect_dsc_t rect_fg;
    lv_draw_rect_dsc_t rect_bg;
    init_rect_dsc(&rect_fg, LVGL_FOREGROUND);
    init_rect_dsc(&rect_bg, LVGL_BACKGROUND);

    const struct pandakb_badge_status *s = &screen_state.status;
    const char *symbol = LV_SYMBOL_STOP;
    const char *label = "Stop";

    switch (s->state) {
    case PANDAKB_MEDIA_PLAYING:
        symbol = LV_SYMBOL_PLAY;
        label = "Play";
        break;
    case PANDAKB_MEDIA_PAUSED:
        symbol = LV_SYMBOL_PAUSE;
        label = "Pause";
        break;
    case PANDAKB_MEDIA_STOPPED:
    default:
        symbol = LV_SYMBOL_STOP;
        label = "Stop";
        break;
    }

    lv_canvas_draw_text(canvas, 0, 40, CANVAS_WIDTH, &label_dsc, symbol);
    lv_canvas_draw_text(canvas, 0, 56, CANVAS_WIDTH, &label_dsc, label);

    /* Volume meter under the playback state. */
    char vol_txt[8];
    snprintk(vol_txt, sizeof(vol_txt), "%u", s->volume);
    lv_canvas_draw_text(canvas, 0, 78, CANVAS_WIDTH, &label_dsc, "Vol");
    lv_canvas_draw_text(canvas, 0, 92, CANVAS_WIDTH, &label_dsc, vol_txt);

    lv_canvas_draw_rect(canvas, 6, 110, 20, 8, &rect_fg);
    lv_canvas_draw_rect(canvas, 7, 111, 18, 6, &rect_bg);
    uint8_t fill_w = (uint8_t)((s->volume * 16) / 100);
    if (fill_w > 0) {
        lv_canvas_draw_rect(canvas, 8, 112, fill_w, 4, &rect_fg);
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
    draw_media_content();
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
