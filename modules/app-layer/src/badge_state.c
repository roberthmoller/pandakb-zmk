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

BUILD_ASSERT(sizeof(struct pandakb_badge_status) == 8, "widget status must pack into 8 split bytes");

ZMK_EVENT_IMPL(pandakb_badge_state_changed);

static struct pandakb_badge_status current = {
    .mode = PANDAKB_WIDGET_IDLE,
};

const struct pandakb_badge_status *pandakb_badge_get(void) { return &current; }

uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status) {
    return ((uint32_t)status->mode) | ((uint32_t)status->u.raw[0] << 8) |
           ((uint32_t)status->u.raw[1] << 16) | ((uint32_t)status->u.raw[2] << 24);
}

uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status) {
    return ((uint32_t)status->u.raw[3]) | ((uint32_t)status->u.raw[4] << 8) |
           ((uint32_t)status->u.raw[5] << 16) | ((uint32_t)status->u.raw[6] << 24);
}

void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out) {
    memset(out, 0, sizeof(*out));
    out->mode = (uint8_t)(param1 & 0xff);
    out->u.raw[0] = (uint8_t)((param1 >> 8) & 0xff);
    out->u.raw[1] = (uint8_t)((param1 >> 16) & 0xff);
    out->u.raw[2] = (uint8_t)((param1 >> 24) & 0xff);
    out->u.raw[3] = (uint8_t)(param2 & 0xff);
    out->u.raw[4] = (uint8_t)((param2 >> 8) & 0xff);
    out->u.raw[5] = (uint8_t)((param2 >> 16) & 0xff);
    out->u.raw[6] = (uint8_t)((param2 >> 24) & 0xff);
}

static bool status_valid(const struct pandakb_badge_status *status) {
    if (status == NULL || status->mode > PANDAKB_WIDGET_IDE) {
        return false;
    }

    switch (status->mode) {
    case PANDAKB_WIDGET_MEDIA:
        if (status->u.media.playback > PANDAKB_MEDIA_PAUSED || status->u.media.volume > 100) {
            return false;
        }
        break;
    case PANDAKB_WIDGET_IDE:
        if (status->u.ide.build > PANDAKB_IDE_FAIL) {
            return false;
        }
        break;
    default:
        break;
    }

    return true;
}

int pandakb_badge_set_status(const struct pandakb_badge_status *status) {
    if (!status_valid(status)) {
        return -EINVAL;
    }

    if (memcmp(&current, status, sizeof(current)) == 0) {
        return 0;
    }

    current = *status;
    LOG_DBG("widget -> mode=%u", current.mode);
    raise_pandakb_badge_state_changed((struct pandakb_badge_state_changed){.status = current});
    return 0;
}
