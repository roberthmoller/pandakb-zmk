/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <zmk/event_manager.h>

/* Kept under badge.h / nodelabel "badge" so split sync DT stays stable. */

enum pandakb_media_state {
    PANDAKB_MEDIA_STOPPED = 0,
    PANDAKB_MEDIA_PLAYING = 1,
    PANDAKB_MEDIA_PAUSED = 2,
};

struct pandakb_badge_status {
    uint8_t state;  /* pandakb_media_state */
    uint8_t volume; /* 0–100 */
    uint8_t flags;
    uint8_t reserved;
};

struct pandakb_badge_state_changed {
    struct pandakb_badge_status status;
};

ZMK_EVENT_DECLARE(pandakb_badge_state_changed);

const struct pandakb_badge_status *pandakb_badge_get(void);
int pandakb_badge_set_status(const struct pandakb_badge_status *status);

/* Pack/unpack for global-locality behavior params (split sync). */
uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status);
uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status);
void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out);
