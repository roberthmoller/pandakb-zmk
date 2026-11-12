/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <errno.h>
#include <string.h>
#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

#include <pandakb/badge.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

ZMK_EVENT_IMPL(pandakb_badge_state_changed);

static struct pandakb_badge_status current = {
    .state = PANDAKB_MEDIA_STOPPED,
    .volume = 0,
};

const struct pandakb_badge_status *pandakb_badge_get(void) { return &current; }

uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status) {
    return ((uint32_t)status->state) | ((uint32_t)status->volume << 8) |
           ((uint32_t)status->flags << 16);
}

uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status) {
    (void)status;
    return 0;
}

void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out) {
    (void)param2;
    out->state = (uint8_t)(param1 & 0xff);
    out->volume = (uint8_t)((param1 >> 8) & 0xff);
    out->flags = (uint8_t)((param1 >> 16) & 0xff);
    out->reserved = 0;
}

int pandakb_badge_set_status(const struct pandakb_badge_status *status) {
    if (status == NULL || status->state > PANDAKB_MEDIA_PAUSED || status->volume > 100) {
        return -EINVAL;
    }

    if (memcmp(&current, status, sizeof(current)) == 0) {
        return 0;
    }

    current = *status;
    LOG_DBG("media -> state=%u vol=%u", current.state, current.volume);
    raise_pandakb_badge_state_changed((struct pandakb_badge_state_changed){.status = current});
    return 0;
}
