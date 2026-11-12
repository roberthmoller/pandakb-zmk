# PandaKB Companion (macOS)

Menu bar app for PandaKB. **Raw HID → keyboard is currently disabled** (`CompanionModel.hidEnabled = false`) while the firmware is stabilized.

The app still shows local media play/pause + volume in the menu bar. Keymap Wizard remains available.

## Run

```bash
cd companion/PandaKBCompanion
swift run
```

## Firmware (no HID)

Flash the OLED pair built with stock `nice_oled` only:

- Left: `Lily58_L nice_oled`
- Right: `Lily58_R nice_oled`

To re-enable keyboard HID later: set `CompanionModel.hidEnabled = true` and restore `raw_hid_adapter app_layer` / `game_badge` in `build.yaml`.

Protocol (dormant): [../modules/app-layer/PROTOCOL.md](../modules/app-layer/PROTOCOL.md)
