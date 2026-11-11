# PandaKB Raw HID protocol

32-byte reports. Usage page `0xFF60`, usage `0x61` (zmk-raw-hid defaults).

## Host → keyboard

| Byte 0 | Byte 1+ | Meaning |
|--------|---------|---------|
| `0x01` | layer | Sticky `layer_to`. Allowed: `0` Base, `4` WoW, `5` LoL |
| `0x02` | badge (+ optional stats) | Set game badge / status |

### `0x02` SET_BADGE

| Offset | Field |
|--------|-------|
| 1 | badge: `0` idle (buddy), `1` WoW, `2` LoL |
| 2 | kills (optional) |
| 3 | deaths (optional) |
| 4 | assists (optional) |
| 5–6 | CS little-endian (optional) |
| 7 | level (optional) |
| 8 | flags: bit0 = stats valid |

If the report is shorter than 9 bytes, stats are cleared and only the badge icon/buddy is shown.

Badge is independent of layer. Changing badge does not change layer and vice versa.

Badge + packed stats use a global-locality behavior so the right/peripheral OLED updates over the split link.

## Right OLED

| Badge | Display |
|-------|---------|
| idle | Animated buddy character |
| WoW | Game icon (+ level if stats valid) |
| LoL | Game icon + `K/D/A` and `CS` / level when Live Client API stats are available |
