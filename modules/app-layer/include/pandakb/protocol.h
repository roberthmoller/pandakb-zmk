/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/* Raw HID report: 32 bytes. */
#define PANDAKB_HID_CMD_SET_MEDIA 0x03      /* Host → keyboard (legacy) */
#define PANDAKB_HID_CMD_SET_WIDGET 0x10     /* Host → keyboard */
#define PANDAKB_HID_CMD_LAYER_PREVIEW 0x20  /* Keyboard → host */

#define PANDAKB_LAYER_PREVIEW_HIDE 0
#define PANDAKB_LAYER_PREVIEW_SHOW 1

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
 *
 * LAYER_PREVIEW (keyboard → host):
 *   [0] 0x20
 *   [1] 1 show / 0 hide
 *   [2] layer index (e.g. 1=Lower, 2=Raise)
 */
