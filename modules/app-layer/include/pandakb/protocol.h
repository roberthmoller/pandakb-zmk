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

/*
 * SET_BADGE payload:
 *   [0] 0x02
 *   [1] badge (0 idle, 1 WoW, 2 LoL)
 * Optional stats (if report length >= 9):
 *   [2] kills
 *   [3] deaths
 *   [4] assists
 *   [5] cs low
 *   [6] cs high
 *   [7] level
 *   [8] flags (bit0 = stats valid)
 */
