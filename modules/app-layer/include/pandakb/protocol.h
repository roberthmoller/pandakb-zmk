/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/* Raw HID report: 32 bytes. Host → keyboard. */
#define PANDAKB_HID_CMD_SET_MEDIA 0x03
#define PANDAKB_HID_CMD_SET_WIDGET 0x10

/*
 * SET_MEDIA (legacy, maps to widget mode MEDIA):
 *   [0] 0x03
 *   [1] playback: 0 stopped, 1 playing, 2 paused
 *   [2] volume 0–100
 *
 * SET_WIDGET:
 *   [0] 0x10
 *   [1] mode: 0 idle, 1 media, 2 lol, 3 ide
 *   [2..8] mode payload (7 bytes) — see PROTOCOL.md
 */
