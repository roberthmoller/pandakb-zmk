# PandaKB Companion (macOS)

Menu bar app that:

1. Pushes **media play/pause + system volume** to the right OLED over Raw HID
2. Hosts **ZmkWiz** (Keymap Wizard) — Lily58 learn / trainer / test / firmware UI

No game layers, LoL stats, or focus switching — those paths overloaded the split link.

## Run

Needs **Node.js ≥ 22.17** (wizard UI; `.nvmrc` → 22.22.0) and Swift 5.9+.

```bash
# once — install ZmkWiz deps (also auto-runs on first Open Keymap Wizard)
cd companion/zmkwiz
npm run install:deps

cd ../PandaKBCompanion
swift run
```

### Keymap Wizard

From the menu bar `KB` menu:

| Item | Action |
|------|--------|
| **Open Keymap Wizard** | Starts local Vite on `127.0.0.1:17301` and opens a WKWebView window |
| **Open Wizard in Browser** | Same server, opens your default browser (better for Web Serial / Firmware Live) |

Override the wizard root with `PANDAKB_ZMKWIZ=/path/to/zmkwiz` if needed.

Visual `.keymap` editing (Keymap Editor–style) is next; for now load maps in Learn/Trainer and use Firmware Live / Clipboard workflows.

## Firmware

Flash the OLED pair built with:

- Left: `Lily58_L nice_oled raw_hid_adapter app_layer`
- Right: `Lily58_R game_badge`

After enabling Raw HID for the first time, re-pair Bluetooth if the keyboard was already bonded.

Plug in USB (or use BLE HID) so the companion can open the Raw HID interface (`0xFF60` / `0x61`).

## Behavior

- Polls now-playing via MediaRemote and output volume about every 2s
- Sends HID only when play state or volume **changes**
- Right OLED shows play/pause/stop, volume, battery, and link

Protocol details: [../modules/app-layer/PROTOCOL.md](../modules/app-layer/PROTOCOL.md)

## Layout

```
companion/
  PandaKBCompanion/   # Swift menu bar + HID + wizard window
  zmkwiz/             # ZmkWiz (SvelteKit) hosted by the companion
  scripts/
```
