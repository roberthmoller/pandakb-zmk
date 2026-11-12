# PandaKB Lily58

ZMK firmware (user-config) plus the macOS companion app that drives the right OLED over Raw HID.

## Layout

| Path | What |
|------|------|
| `boards/`, `config/`, `modules/`, `zephyr/`, `build.yaml` | Firmware — ZMK user-config (CI expects these at repo root) |
| `apps/companion/` | Tauri macOS companion + Keymap Wizard |
| `docs/` | Pointers into shared docs (protocol lives with firmware) |
| `artifacts/` | Local UF2 build output (gitignored) |

## Firmware

OLED + HID builds (companion pair):

- Left: `Lily58_L nice_oled raw_hid_adapter app_layer`
- Right: `Lily58_R game_badge`

Keymaps and west manifest: [`config/`](config/). Custom behaviors / Raw HID: [`modules/app-layer/`](modules/app-layer/).

## Companion

```bash
cd apps/companion
npm install
npm run tauri:dev
```

Details: [`apps/companion/README.md`](apps/companion/README.md)

## Protocol

Host ↔ keyboard Raw HID contract: [`modules/app-layer/PROTOCOL.md`](modules/app-layer/PROTOCOL.md) (also linked from [`docs/`](docs/)).
