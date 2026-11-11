# PandaKB Raw HID protocol

32-byte reports. Usage page `0xFF60`, usage `0x61` (zmk-raw-hid defaults).

## Host → keyboard

| Byte 0 | Byte 1 | Meaning |
|--------|--------|---------|
| `0x01` | layer | Sticky `layer_to`. Allowed: `0` Base, `4` WoW, `5` LoL |
| `0x02` | badge | Set game badge. `0` idle, `1` WoW, `2` LoL |

Badge is independent of layer. Changing badge does not change layer and vice versa.

Badge uses a global-locality behavior so the right/peripheral OLED can update over the split link.
