/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>
#include <zmk/event_manager.h>

enum pandakb_badge {
    PANDAKB_BADGE_IDLE = 0,
    PANDAKB_BADGE_WOW = 1,
    PANDAKB_BADGE_LOL = 2,
};

struct pandakb_badge_state_changed {
    uint8_t badge;
};

ZMK_EVENT_DECLARE(pandakb_badge_state_changed);

uint8_t pandakb_badge_get(void);
int pandakb_badge_set(uint8_t badge);
const char *pandakb_badge_label(uint8_t badge);
