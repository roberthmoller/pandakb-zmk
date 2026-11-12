/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

#include <stdint.h>

/** Unknown / not yet reported state of charge. */
#define PANDAKB_BATTERY_UNKNOWN 0xFF

/**
 * Push a STATUS report (0x22) to the companion: left/right SoC + active layer.
 * No-op when Raw HID / keymap APIs are unavailable on this half.
 */
void pandakb_status_send(void);