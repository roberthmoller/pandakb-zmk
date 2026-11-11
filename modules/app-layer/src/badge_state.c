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
    .badge = PANDAKB_BADGE_IDLE,
};

const struct pandakb_badge_status *pandakb_badge_get(void) { return &current; }

uint8_t pandakb_badge_id(void) { return current.badge; }

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

uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status) {
    return ((uint32_t)status->badge) | ((uint32_t)status->kills << 8) |
           ((uint32_t)status->deaths << 16) | ((uint32_t)status->assists << 24);
}

uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status) {
    return ((uint32_t)status->cs) | ((uint32_t)status->level << 16) |
           ((uint32_t)status->flags << 24);
}

void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out) {
    out->badge = (uint8_t)(param1 & 0xff);
    out->kills = (uint8_t)((param1 >> 8) & 0xff);
    out->deaths = (uint8_t)((param1 >> 16) & 0xff);
    out->assists = (uint8_t)((param1 >> 24) & 0xff);
    out->cs = (uint16_t)(param2 & 0xffff);
    out->level = (uint8_t)((param2 >> 16) & 0xff);
    out->flags = (uint8_t)((param2 >> 24) & 0xff);
}

int pandakb_badge_set_status(const struct pandakb_badge_status *status) {
    if (status == NULL || status->badge > PANDAKB_BADGE_LOL) {
        return -EINVAL;
    }

    if (memcmp(&current, status, sizeof(current)) == 0) {
        return 0;
    }

    current = *status;
    LOG_DBG("badge -> %u flags=0x%02x kda=%u/%u/%u cs=%u lv=%u", current.badge, current.flags,
            current.kills, current.deaths, current.assists, current.cs, current.level);
    raise_pandakb_badge_state_changed((struct pandakb_badge_state_changed){.status = current});
    return 0;
}

int pandakb_badge_set(uint8_t badge) {
    struct pandakb_badge_status status = {
        .badge = badge,
        .flags = 0,
    };
    return pandakb_badge_set_status(&status);
}
