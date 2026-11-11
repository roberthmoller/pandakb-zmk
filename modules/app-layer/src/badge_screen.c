/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

#include <lvgl.h>

#include <zmk/display.h>
#include <zmk/event_manager.h>

#include <pandakb/badge.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

static lv_obj_t *badge_label;

struct badge_screen_state {
    uint8_t badge;
};

static void badge_screen_update_cb(struct badge_screen_state state) {
    if (badge_label == NULL) {
        return;
    }

    const char *label = pandakb_badge_label(state.badge);
    if (label == NULL || label[0] == '\0') {
        lv_label_set_text(badge_label, "·");
    } else {
        lv_label_set_text(badge_label, label);
    }
}

static struct badge_screen_state badge_screen_get_state(const zmk_event_t *eh) {
    const struct pandakb_badge_state_changed *ev = as_pandakb_badge_state_changed(eh);
    if (ev != NULL) {
        return (struct badge_screen_state){.badge = ev->badge};
    }
    return (struct badge_screen_state){.badge = pandakb_badge_get()};
}

ZMK_DISPLAY_WIDGET_LISTENER(widget_game_badge, struct badge_screen_state, badge_screen_update_cb,
                            badge_screen_get_state)
ZMK_SUBSCRIPTION(widget_game_badge, pandakb_badge_state_changed);

lv_obj_t *zmk_display_status_screen(void) {
    lv_obj_t *screen = lv_obj_create(NULL);

    badge_label = lv_label_create(screen);
    lv_obj_set_style_text_font(badge_label, lv_theme_get_font_large(screen), LV_PART_MAIN);
    lv_obj_set_style_text_align(badge_label, LV_TEXT_ALIGN_CENTER, LV_PART_MAIN);
    lv_obj_align(badge_label, LV_ALIGN_CENTER, 0, 0);
    lv_label_set_text(badge_label, "·");

    widget_game_badge_init();
    return screen;
}
