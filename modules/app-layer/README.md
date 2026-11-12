# PandaKB app-layer module

Typed right-OLED widgets driven by the companion over Raw HID:

- Host sends `SET_WIDGET` (`idle` / `media` / `lol` / `ide`) — see [PROTOCOL.md](PROTOCOL.md)
- Keyboard sends `LAYER_PREVIEW` (`0x20`) on `&mo_prev` double-tap-hold / release for the companion layout overlay
- Central syncs an 8-byte status to the right half via the global `badge` behavior
- Right OLED (`game_badge`): link + battery header, then the active mode body (LoL kill/multikill flash overlay)

Shields: `boards/shields/app_layer` (left Raw HID) and `boards/shields/game_badge` (right screen).
