# PandaKB Companion

macOS menu-bar / tray app that drives the right OLED over Raw HID and hosts the Keymap Wizard UI.

## Run

```bash
cd apps/companion
npm install
npm run tauri:dev
```

The tray title shows the active widget. Left-click opens the **Build & flash** window (learn/trainer UI is hidden for now).

### Build & flash

The main window drives a local west build of the companion OLED pair, then copies each UF2 to `/Volumes/RPI-RP2` after a double-reset.

1. Point **West workspace** at a west tree (default: the existing `pandakb-lily` workspace). First run may need **West update**.
2. **Build left + right** — uses `west` on PATH, otherwise Docker image `zmkfirmware/zmk-build-arm:stable`.
3. Double-reset each half and **Flash left** / **Flash right**.

UF2s are also copied to `artifacts/Lily58_*_oled.uf2`.

### Widgets (priority)

1. **LoL** — Live Client API at `https://127.0.0.1:2999` while in-game ([what we can read](LOL_LIVE_CLIENT.md))
2. **IDE** — `POST http://127.0.0.1:17302/ide` with `{ "errors", "warnings", "build", "tag", "active" }`
3. **Media** — Spotify / Music now-playing + system volume
4. **Idle** — static marker when nothing else is active

Protocol: [`../../modules/app-layer/PROTOCOL.md`](../../modules/app-layer/PROTOCOL.md)

## Firmware

Flash the **companion HID OLED pair** (not the stock OLED pair):

| Half | Artifact | Shields |
|------|----------|---------|
| **Left** (plug USB here) | `Lily58_left_oled.uf2` | `Lily58_L nice_oled raw_hid_adapter app_layer` |
| **Right** | `Lily58_right_oled.uf2` | `Lily58_R game_badge` |

1. Put each nice!nano in bootloader (double-reset) and copy the matching `.uf2`.
2. Prefer a CI / local build that includes `app_layer` + `game_badge` (see `build.yaml` artifacts `Lily58_left_oled` / `Lily58_right_oled`).
3. After flash, leave the **left** half connected over USB so Raw HID reaches the companion.

Stock fallbacks (`Lily58_*_oled_stock.uf2`) are nice_oled only — no widget HID.

### Keymap source of truth

Companion layers / layouts are parsed from the repo file:

`config/Lily58.keymap`

Tauri commands `firmware_keymap_get` / `firmware_keymap_set` read and write that path. On launch (and each overlay show) the app reloads it so the UI matches what you flash. The bundled `src/lib/boards/lily58/keymap.json` is only a fallback for non-Tauri / offline use — regenerate with `npx vite-node scripts/regen-keymap.ts`.

### Layer layout preview

LOWER / RAISE use `&mo_prev` (momentary, like `&mo`). **Double-tap and hold** a layer key to show a floating glass overlay of that layer’s layout; release to dismiss. Firmware sends Raw HID `0x20 LAYER_PREVIEW` (keyboard → host); the Tauri app listens and toggles the always-on-top preview window. Overlay content comes from the local keymap above (Base / Lower / Raise / Adjust / WoW / LoL).

### HID smoke check

```bash
swift scripts/check-hid.swift
```

Expects the left half flashed with the companion OLED build and plugged over USB.
