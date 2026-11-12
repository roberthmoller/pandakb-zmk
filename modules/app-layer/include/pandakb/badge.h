/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <zmk/event_manager.h>

/* Kept under badge.h / nodelabel "badge" so split sync DT stays stable. */

enum pandakb_widget_mode {
    PANDAKB_WIDGET_IDLE = 0,
    PANDAKB_WIDGET_MEDIA = 1,
    PANDAKB_WIDGET_LOL = 2,
    PANDAKB_WIDGET_IDE = 3,
};

enum pandakb_media_playback {
    PANDAKB_MEDIA_STOPPED = 0,
    PANDAKB_MEDIA_PLAYING = 1,
    PANDAKB_MEDIA_PAUSED = 2,
};

enum pandakb_ide_build {
    PANDAKB_IDE_IDLE = 0,
    PANDAKB_IDE_BUILDING = 1,
    PANDAKB_IDE_OK = 2,
    PANDAKB_IDE_FAIL = 3,
};

#define PANDAKB_STATUS_FLAG_VALID 0x01
/* LoL flags bits 1–3: one-shot event flash (companion clears next poll). */
#define PANDAKB_LOL_EVENT_SHIFT 1
#define PANDAKB_LOL_EVENT_MASK 0x07
#define PANDAKB_LOL_EVENT_NONE 0
#define PANDAKB_LOL_EVENT_KILL 1
#define PANDAKB_LOL_EVENT_DOUBLE 2
#define PANDAKB_LOL_EVENT_TRIPLE 3
#define PANDAKB_LOL_EVENT_QUADRA 4
#define PANDAKB_LOL_EVENT_PENTA 5
#define PANDAKB_LOL_EVENT_ACE 6
#define PANDAKB_LOL_EVENT_FIRST_BLOOD 7
#define PANDAKB_WIDGET_TITLE_LEN 5
#define PANDAKB_WIDGET_TAG_LEN 4

/* All mode payloads are exactly 7 bytes (no padding) for 8-byte split sync. */
struct pandakb_widget_media {
    uint8_t playback;
    uint8_t volume;
    uint8_t title[PANDAKB_WIDGET_TITLE_LEN];
} __packed;

struct pandakb_widget_lol {
    uint8_t kills;
    uint8_t deaths;
    uint8_t assists;
    uint8_t cs_lo;
    uint8_t cs_hi;
    uint8_t level;
    uint8_t flags;
} __packed;

struct pandakb_widget_ide {
    uint8_t errors;
    uint8_t warnings;
    uint8_t build;
    uint8_t tag[PANDAKB_WIDGET_TAG_LEN];
} __packed;

struct pandakb_badge_status {
    uint8_t mode;
    union {
        struct pandakb_widget_media media;
        struct pandakb_widget_lol lol;
        struct pandakb_widget_ide ide;
        uint8_t raw[7];
    } u;
} __packed;

struct pandakb_badge_state_changed {
    struct pandakb_badge_status status;
};

ZMK_EVENT_DECLARE(pandakb_badge_state_changed);

const struct pandakb_badge_status *pandakb_badge_get(void);
int pandakb_badge_set_status(const struct pandakb_badge_status *status);

static inline uint16_t pandakb_lol_cs(const struct pandakb_widget_lol *lol) {
    return (uint16_t)lol->cs_lo | ((uint16_t)lol->cs_hi << 8);
}

static inline void pandakb_lol_set_cs(struct pandakb_widget_lol *lol, uint16_t cs) {
    lol->cs_lo = (uint8_t)(cs & 0xff);
    lol->cs_hi = (uint8_t)((cs >> 8) & 0xff);
}

static inline uint8_t pandakb_lol_event(const struct pandakb_widget_lol *lol) {
    return (uint8_t)((lol->flags >> PANDAKB_LOL_EVENT_SHIFT) & PANDAKB_LOL_EVENT_MASK);
}

static inline void pandakb_lol_set_event(struct pandakb_widget_lol *lol, uint8_t event) {
    lol->flags = (uint8_t)((lol->flags & ~(PANDAKB_LOL_EVENT_MASK << PANDAKB_LOL_EVENT_SHIFT)) |
                           ((event & PANDAKB_LOL_EVENT_MASK) << PANDAKB_LOL_EVENT_SHIFT));
}

/* Pack/unpack for global-locality behavior params (split sync, 8 bytes). */
uint32_t pandakb_badge_pack_param1(const struct pandakb_badge_status *status);
uint32_t pandakb_badge_pack_param2(const struct pandakb_badge_status *status);
void pandakb_badge_unpack_params(uint32_t param1, uint32_t param2,
                                 struct pandakb_badge_status *out);
