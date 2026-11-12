# PandaKB Raw HID protocol

32-byte reports. Usage page `0xFF60`, usage `0x61` (zmk-raw-hid defaults).

## Host → keyboard

| Byte 0 | Byte 1+ | Meaning |
|--------|---------|---------|
| `0x10` | mode + 7-byte payload | Set typed right-OLED widget |
| `0x03` | playback, volume | Legacy media → widget mode `MEDIA` |
| `0x30` | half | Soft-enter UF2 bootloader (`0`/`1` left, `2` right) |
| `0x31` | layer index | Sticky layer switch (`&to` semantics: Base/Lower/Raise/Adjust/WoW/LoL) |

## Keyboard → host

| Byte 0 | Byte 1+ | Meaning |
|--------|---------|---------|
| `0x20` | show/hide + layer | Layer layout preview for the companion overlay |
| `0x21` | 8-byte bit mask | Pressed key positions while the overlay is showing |
| `0x22` | L/R SoC + layer | Battery + active layer for the menu bar |

### `0x20` LAYER_PREVIEW

Sent when the companion overlay should show or hide a layer layout:

- **Lower / Raise** — double-tap-hold `&mo_prev` (preview session)
- **Adjust** — auto when the tri-layer is highest (LOWER+RAISE held)
- **WoW / LoL** — auto while that sticky `&to` layer is active

While a preview session is open, the reported layer follows `zmk_keymap_highest_layer_active()` (so holding both layer keys upgrades Lower/Raise → Adjust).

| Offset | Field |
|--------|-------|
| 1 | `1` show, `0` hide |
| 2 | Layer index (`0` Base, `1` Lower, `2` Raise, `3` Adjust, `4` WoW, `5` LoL) |

### `0x21` KEY_MATRIX

Sent whenever the overlay is visible and the pressed-key set changes (including a snapshot on show).

| Offset | Field |
|--------|-------|
| 1–8 | Bit mask of pressed matrix positions (`bit0` of byte 0 = position 0) |

### `0x22` STATUS

Pushed on battery/layer change, every ~15s, and when the host sends `0x32 GET_STATUS`.

| Offset | Field |
|--------|-------|
| 1 | Left (central) state of charge `0`–`100`, or `0xFF` unknown |
| 2 | Right (peripheral) state of charge `0`–`100`, or `0xFF` unknown |
| 3 | Active layer index (`0` Base … `5` LoL) |

### `0x10` SET_WIDGET

| Offset | Field |
|--------|-------|
| 1 | mode: `0` idle, `1` media, `2` lol, `3` ide |
| 2–8 | mode payload (7 bytes) |

#### Mode `MEDIA` (1)

| Offset | Field |
|--------|-------|
| 2 | playback: `0` stopped, `1` playing, `2` paused |
| 3 | volume `0`–`100` |
| 4–8 | title, up to 5 ASCII chars (NUL-padded) |

#### Mode `LOL` (2)

| Offset | Field |
|--------|-------|
| 2 | kills |
| 3 | deaths |
| 4 | assists |
| 5–6 | CS little-endian |
| 7 | level |
| 8 | flags: bit0 = stats valid; bits 1–3 = one-shot event (see below) |

Event codes (bits 1–3), cleared by the companion on the next poll after a flash:

| Code | Meaning |
|------|---------|
| 0 | none |
| 1 | champion kill |
| 2 | double kill |
| 3 | triple kill |
| 4 | quadra kill |
| 5 | pentakill |
| 6 | ace |
| 7 | first blood |

#### Mode `IDE` (3)

| Offset | Field |
|--------|-------|
| 2 | error count |
| 3 | warning count |
| 4 | build: `0` idle, `1` building, `2` ok, `3` fail |
| 5–8 | short tag, up to 4 ASCII chars (NUL-padded) |

#### Mode `IDLE` (0)

Payload ignored / zeroed. Right OLED shows the animated buddy under the link/battery header.

### `0x03` SET_MEDIA (legacy)

| Offset | Field |
|--------|-------|
| 1 | playback |
| 2 | volume `0`–`100` |

Mapped to `SET_WIDGET` media with an empty title.

### `0x31` SET_LAYER

Sticky layer switch on the central half (`zmk_keymap_layer_to`), same as `&to`.

| Offset | Field |
|--------|-------|
| 1 | Layer index: `0` Base, `1` Lower, `2` Raise, `3` Adjust, `4` WoW, `5` LoL |

While WoW/LoL is active, the keyboard still emits `0x20 LAYER_PREVIEW` for the companion overlay.

### `0x32` GET_STATUS

Host asks the keyboard to push a `0x22 STATUS` report immediately (byte 0 only).

### `0x30` ENTER_BOOTLOADER

Asks the keyboard to reboot into the Adafruit UF2 bootloader so the companion can copy a `.uf2`.

| Offset | Field |
|--------|-------|
| 1 | `0`/`1` this half (left/central), `2` right/peripheral via split |

Right-half entry needs the peripheral powered (USB is fine) and a working split link. If relay fails, double-reset that half.

## Split sync

Widget status is packed into the global `badge` behavior (`param1` + `param2` = 8 bytes) so the right/peripheral OLED updates over the split link. Host updates should be change-only.

## Right OLED

Portrait column (`game_badge` shield), top to bottom:

1. Link / USB
2. Battery
3. Mode body (idle / media / lol / ide)

LoL event flashes keep a black body (no inverted white fill). Lit ink draws a frame, walking streak pips, a small glyph (blade / droplet / crown), and the short label (`+` / `x2`… / `ACE` / `FB`) for ~1.6s.

Volume changes (MEDIA volume delta, or entering MEDIA for a volume HUD) briefly replace the mode body with a full-height vertical meter (fill upward) and percentage for ~1.6s. The companion forces MEDIA for ~1.8s on system volume changes so the HUD still appears over Idle / LoL / IDE.
