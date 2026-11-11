/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <errno.h>
#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

#include <pandakb/badge.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

ZMK_EVENT_IMPL(pandakb_badge_state_changed);

static uint8_t current_badge = PANDAKB_BADGE_IDLE;

uint8_t pandakb_badge_get(void) { return current_badge; }

const char *pandakb_badge_label(uint8_t badge) {
    switch (badge) {
    case PANDAKB_BADGE_WOW:
        return "WoW";
    case PANDAKB_BADGE_LOL:
        return "LoL";
    case PANDAKB_BADGE_IDLE:
    default:
        return "";
    }
}

int pandakb_badge_set(uint8_t badge) {
    if (badge > PANDAKB_BADGE_LOL) {
        return -EINVAL;
    }

    if (current_badge == badge) {
        return 0;
    }

    current_badge = badge;
    LOG_DBG("game badge -> %u (%s)", badge, pandakb_badge_label(badge));
    raise_pandakb_badge_state_changed((struct pandakb_badge_state_changed){.badge = badge});
    return 0;
}
