# PandaKB Raw HID protocol

32-byte reports. Usage page `0xFF60`, usage `0x61` (zmk-raw-hid defaults).

## Host → keyboard

| Byte 0 | Byte 1+ | Meaning |
|--------|---------|---------|
| `0x10` | mode + 7-byte payload | Set typed right-OLED widget |
| `0x03` | playback, volume | Legacy media → widget mode `MEDIA` |

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
| 8 | flags: bit0 = stats valid |

#### Mode `IDE` (3)

| Offset | Field |
|--------|-------|
| 2 | error count |
| 3 | warning count |
| 4 | build: `0` idle, `1` building, `2` ok, `3` fail |
| 5–8 | short tag, up to 4 ASCII chars (NUL-padded) |

#### Mode `IDLE` (0)

Payload ignored / zeroed. Right OLED shows a static idle marker (no animation).

### `0x03` SET_MEDIA (legacy)

| Offset | Field |
|--------|-------|
| 1 | playback |
| 2 | volume `0`–`100` |

Mapped to `SET_WIDGET` media with an empty title.

## Split sync

Widget status is packed into the global `badge` behavior (`param1` + `param2` = 8 bytes) so the right/peripheral OLED updates over the split link. Host updates should be change-only.

## Right OLED

Portrait column (`game_badge` shield), top to bottom:

1. Link / USB
2. Battery
3. Mode body (idle / media / lol / ide)
