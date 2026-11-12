# PandaKB Raw HID protocol

32-byte reports. Usage page `0xFF60`, usage `0x61` (zmk-raw-hid defaults).

## Host → keyboard

| Byte 0 | Byte 1+ | Meaning |
|--------|---------|---------|
| `0x03` | state, volume | Set media play/pause + volume for the right OLED |

### `0x03` SET_MEDIA

| Offset | Field |
|--------|-------|
| 1 | state: `0` stopped, `1` playing, `2` paused |
| 2 | volume `0`–`100` |

Legacy `0x01` layer and `0x02` badge commands are ignored.

Media status uses a global-locality behavior so the right/peripheral OLED updates over the split link. Updates should be change-only (companion already dedupes).

## Right OLED

Portrait column (`game_badge` shield):

1. Link / USB
2. Battery
3. Play / Pause / Stop
4. Volume percent + bar
