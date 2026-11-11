/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/* Raw HID report: 32 bytes. Host → keyboard. */
#define PANDAKB_HID_CMD_LAYER_TO 0x01
#define PANDAKB_HID_CMD_SET_BADGE 0x02

/* Sticky layer IDs matching config/Lily58.keymap */
#define PANDAKB_LAYER_BASE 0
#define PANDAKB_LAYER_WOW 4
#define PANDAKB_LAYER_LOL 5
