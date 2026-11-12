# PandaKB Companion

macOS menu-bar / tray app that drives the right OLED over Raw HID and hosts the Keymap Wizard UI.

## App (Tauri)

```bash
cd companion/zmkwiz
npm install
npm run tauri:dev
```

The tray title shows the active widget. Left-click opens the Keymap Wizard window.

### Widgets (priority)

1. **LoL** — Live Client API at `https://127.0.0.1:2999` while in-game
2. **IDE** — `POST http://127.0.0.1:17302/ide` with `{ "errors", "warnings", "build", "tag", "active" }`
3. **Media** — Spotify / Music now-playing + system volume
4. **Idle** — static marker when nothing else is active

Protocol: [../modules/app-layer/PROTOCOL.md](../modules/app-layer/PROTOCOL.md)

## Firmware

Flash the companion OLED pair:

- Left: `Lily58_L nice_oled raw_hid_adapter app_layer` → `Lily58_left_oled`
- Right: `Lily58_R game_badge` → `Lily58_right_oled`

Plug the **left** half over USB for Raw HID. Stock `*_oled_stock` artifacts remain if you need OLED without HID.

## Legacy Swift app

[`PandaKBCompanion/`](PandaKBCompanion/) is kept as a reference for MediaRemote / HID matching. Prefer the Tauri app above.
