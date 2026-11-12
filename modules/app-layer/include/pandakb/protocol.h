/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/* Raw HID report: 32 bytes. Host → keyboard. */
#define PANDAKB_HID_CMD_SET_MEDIA 0x03

/*
 * SET_MEDIA payload:
 *   [0] 0x03
 *   [1] state: 0 stopped/idle, 1 playing, 2 paused
 *   [2] volume 0–100
 */
