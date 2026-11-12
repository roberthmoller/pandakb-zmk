/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/* Raw HID report: 32 bytes. */
#define PANDAKB_HID_CMD_SET_MEDIA 0x03        /* Host → keyboard (legacy) */
#define PANDAKB_HID_CMD_SET_WIDGET 0x10       /* Host → keyboard */
#define PANDAKB_HID_CMD_LAYER_PREVIEW 0x20    /* Keyboard → host */
#define PANDAKB_HID_CMD_KEY_MATRIX 0x21       /* Keyboard → host */
#define PANDAKB_HID_CMD_STATUS 0x22           /* Keyboard → host */
#define PANDAKB_HID_CMD_ENTER_BOOTLOADER 0x30 /* Host → keyboard */
#define PANDAKB_HID_CMD_SET_LAYER 0x31        /* Host → keyboard */
#define PANDAKB_HID_CMD_GET_STATUS 0x32       /* Host → keyboard */

#define PANDAKB_LAYER_PREVIEW_HIDE 0
#define PANDAKB_LAYER_PREVIEW_SHOW 1

/* SET_LAYER / LAYER_PREVIEW layer indices (Lily58 companion keymap). */
#define PANDAKB_LAYER_BASE 0
#define PANDAKB_LAYER_LOWER 1
#define PANDAKB_LAYER_RAISE 2
#define PANDAKB_LAYER_ADJUST 3
#define PANDAKB_LAYER_WOW 4
#define PANDAKB_LAYER_LOL 5

#define PANDAKB_KEY_MATRIX_BYTES 8 /* 64 key positions */

/* ENTER_BOOTLOADER half selectors (byte 1). */
#define PANDAKB_BOOTLOADER_SELF 0
#define PANDAKB_BOOTLOADER_LEFT 1
#define PANDAKB_BOOTLOADER_RIGHT 2

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
 *   [2] layer index (0=Base, 1=Lower, 2=Raise, 3=Adjust, 4=WoW, 5=LoL)
 *
 * KEY_MATRIX (keyboard → host, while overlay is showing):
 *   [0] 0x21
 *   [1..8] bit mask of pressed positions (bit0 of byte0 = position 0)
 *
 * STATUS (keyboard → host):
 *   [0] 0x22
 *   [1] left SoC 0–100, or 0xFF unknown
 *   [2] right SoC 0–100, or 0xFF unknown
 *   [3] active layer index (0=Base … 5=LoL)
 *
 * ENTER_BOOTLOADER (host → keyboard):
 *   [0] 0x30
 *   [1] half: 0 self, 1 left/central, 2 right/peripheral
 *   Soft-enters Adafruit UF2 bootloader (GPREGRET / retained boot mode).
 *   Right is relayed over the split link when possible; otherwise double-reset.
 *
 * SET_LAYER (host → keyboard):
 *   [0] 0x31
 *   [1] layer index (0=Base … 4=WoW, 5=LoL) — sticky `&to` semantics
 *
 * GET_STATUS (host → keyboard):
 *   [0] 0x32
 *   Asks the keyboard to push a STATUS (0x22) report immediately.
 */
