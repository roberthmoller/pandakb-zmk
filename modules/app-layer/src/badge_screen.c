/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <zephyr/logging/log.h>

#include <lvgl.h>

#include <zmk/display.h>
#include <zmk/event_manager.h>

#include <pandakb/badge.h>

#include "art/art.h"

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

static lv_obj_t *icon_img;
static lv_obj_t *line1_label;
static lv_obj_t *line2_label;
static lv_timer_t *buddy_timer;
static uint8_t buddy_frame;
static uint8_t shown_badge = 0xff;

static const lv_img_dsc_t *buddy_frames[] = {
    &buddy_0,
    &buddy_1,
    &buddy_2,
    &buddy_3,
};

struct badge_screen_state {
    struct pandakb_badge_status status;
};

static void set_buddy_anim(bool enabled) {
    if (buddy_timer == NULL) {
        return;
    }
    if (enabled) {
        lv_timer_resume(buddy_timer);
    } else {
        lv_timer_pause(buddy_timer);
    }
}

static void buddy_timer_cb(lv_timer_t *timer) {
    ARG_UNUSED(timer);
    if (icon_img == NULL || shown_badge != PANDAKB_BADGE_IDLE) {
        return;
    }
    buddy_frame = (buddy_frame + 1) % 4;
    lv_img_set_src(icon_img, buddy_frames[buddy_frame]);
}

static void badge_screen_update_cb(struct badge_screen_state state) {
    if (icon_img == NULL || line1_label == NULL || line2_label == NULL) {
        return;
    }

    const struct pandakb_badge_status *s = &state.status;
    shown_badge = s->badge;

    switch (s->badge) {
    case PANDAKB_BADGE_WOW:
        set_buddy_anim(false);
        lv_obj_align(icon_img, LV_ALIGN_LEFT_MID, 4, 0);
        lv_img_set_src(icon_img, &icon_wow);
        lv_label_set_text(line1_label, "WoW");
        if (s->flags & PANDAKB_STATUS_FLAG_VALID) {
            char buf[24];
            snprintk(buf, sizeof(buf), "Lv %u", s->level);
            lv_label_set_text(line2_label, buf);
        } else {
            lv_label_set_text(line2_label, "");
        }
        break;
    case PANDAKB_BADGE_LOL:
        set_buddy_anim(false);
        lv_obj_align(icon_img, LV_ALIGN_LEFT_MID, 4, 0);
        lv_img_set_src(icon_img, &icon_lol);
        if (s->flags & PANDAKB_STATUS_FLAG_VALID) {
            char kda[24];
            char cs[24];
            snprintk(kda, sizeof(kda), "%u/%u/%u", s->kills, s->deaths, s->assists);
            snprintk(cs, sizeof(cs), "CS %u  Lv%u", s->cs, s->level);
            lv_label_set_text(line1_label, kda);
            lv_label_set_text(line2_label, cs);
        } else {
            lv_label_set_text(line1_label, "LoL");
            lv_label_set_text(line2_label, "");
        }
        break;
    case PANDAKB_BADGE_IDLE:
    default:
        set_buddy_anim(true);
        lv_obj_align(icon_img, LV_ALIGN_CENTER, 0, 0);
        lv_img_set_src(icon_img, buddy_frames[buddy_frame]);
        lv_label_set_text(line1_label, "");
        lv_label_set_text(line2_label, "");
        break;
    }
}

static struct badge_screen_state badge_screen_get_state(const zmk_event_t *eh) {
    const struct pandakb_badge_state_changed *ev = as_pandakb_badge_state_changed(eh);
    if (ev != NULL) {
        return (struct badge_screen_state){.status = ev->status};
    }
    return (struct badge_screen_state){.status = *pandakb_badge_get()};
}

ZMK_DISPLAY_WIDGET_LISTENER(widget_game_badge, struct badge_screen_state, badge_screen_update_cb,
                            badge_screen_get_state)
ZMK_SUBSCRIPTION(widget_game_badge, pandakb_badge_state_changed);

lv_obj_t *zmk_display_status_screen(void) {
    lv_obj_t *screen = lv_obj_create(NULL);

    icon_img = lv_img_create(screen);
    lv_img_set_src(icon_img, buddy_frames[0]);
    lv_obj_align(icon_img, LV_ALIGN_LEFT_MID, 4, 0);

    line1_label = lv_label_create(screen);
    lv_obj_set_style_text_font(line1_label, lv_theme_get_font_large(screen), LV_PART_MAIN);
    lv_obj_align(line1_label, LV_ALIGN_TOP_LEFT, 42, 2);
    lv_label_set_text(line1_label, "");

    line2_label = lv_label_create(screen);
    lv_obj_set_style_text_font(line2_label, lv_theme_get_font_small(screen), LV_PART_MAIN);
    lv_obj_align(line2_label, LV_ALIGN_TOP_LEFT, 42, 18);
    lv_label_set_text(line2_label, "");

    buddy_timer = lv_timer_create(buddy_timer_cb, 450, NULL);
    shown_badge = PANDAKB_BADGE_IDLE;

    widget_game_badge_init();
    return screen;
}
