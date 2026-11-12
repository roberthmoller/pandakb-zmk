# PandaKB app-layer module

Typed right-OLED widgets driven by the companion over Raw HID:

- Host sends `SET_WIDGET` (`idle` / `media` / `lol` / `ide`) — see [PROTOCOL.md](PROTOCOL.md)
- Keyboard sends `LAYER_PREVIEW` (`0x20`) for the companion layout overlay (`&mo_prev` preview, Adjust tri-layer, WoW/LoL sticky) and `KEY_MATRIX` (`0x21`) for live key highlighting
- Central syncs an 8-byte status to the right half via the global `badge` behavior
- Right OLED (`game_badge`): link + battery header, then the active mode body (animated buddy on idle; LoL kill flashes stay black-bg with per-event glyphs / streak pips; volume changes briefly take the whole body with a full-height vertical meter)

Shields: `boards/shields/app_layer` (left Raw HID) and `boards/shields/game_badge` (right screen).
