# PandaKB app-layer module

Raw HID media status for the Lily58 OLED build:

- Host sends `SET_MEDIA` (play/pause/stop + volume 0–100)
- Central applies it and syncs to the right half via a global-locality behavior
- Right OLED: battery + link, then media widgets (no animation)

Shields live in `boards/shields/app_layer` and `boards/shields/game_badge`.
