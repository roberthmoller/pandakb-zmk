/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <zmk/event_manager.h>

enum pandakb_badge {
    PANDAKB_BADGE_IDLE = 0,
    PANDAKB_BADGE_WOW = 1,
    PANDAKB_BADGE_LOL = 2,
};

#define PANDAKB_STATUS_FLAG_VALID 0x01

struct pandakb_badge_status {
    uint8_t badge;
    uint8_t kills;
    uint8_t deaths;
    uint8_t assists;
    uint16_t cs;
    uint8_t level;
    uint8_t flags;
};

struct pandakb_badge_state_changed {
    struct pandakb_badge_status status;
};

ZMK_EVENT_DECLARE(pandakb_badge_state_changed);

const struct pandakb_badge_status *pandakb_badge_get(void);
uint8_t pandakb_badge_id(void);
int pandakb_badge_set(uint8_t badge);
int pandakb_badge_set_status(const struct pandakb_badge_status *status);
const char *pandakb_badge_label(uint8_t badge);

/* Pack/unpack for global-locality behavior params (split sync). */
uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status);
uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status);
void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out);
