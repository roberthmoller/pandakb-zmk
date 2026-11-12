/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#pragma once

/**
 * Companion layer-preview overlay (Raw HID 0x20 / 0x21).
 *
 * Double-tap-hold &mo_prev begins a preview session (Lower/Raise).
 * Adjust / WoW / LoL auto-surface whenever they are the highest active layer.
 */

void pandakb_overlay_preview_begin(void);
void pandakb_overlay_preview_end(void);
