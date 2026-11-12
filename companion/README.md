# PandaKB Companion (macOS)

Menu bar app that:

1. Watches the frontmost app
2. Drives the **right OLED**: static buddy when idle; game icon (+ live stats) for WoW/LoL
3. Optionally sticky-switches keyboard layers (`Base` / `WoW` / `LoL`)

Badge and layer are independent. Left OLED keeps its normal `nice_oled` layout (including layer name).

## Live stats

While League of Legends is in a match, the companion polls the local Live Client API
(`https://127.0.0.1:2999`) and pushes K/D/A, CS, and level to the keyboard.

WoW currently shows the game icon only (no public live API).

## Run

```bash
cd companion/PandaKBCompanion
swift run
```

Grant **Accessibility** if macOS prompts (needed for reliable focus observation in some setups).

## Firmware

Flash the OLED pair built with:

- Left: `Lily58_L nice_oled raw_hid_adapter app_layer`
- Right: `Lily58_R game_badge`

After enabling Raw HID for the first time, re-pair Bluetooth if the keyboard was already bonded.

Plug in USB (or use BLE HID) so the companion can open the Raw HID interface (`0xFF60` / `0x61`).

## Defaults

| App bundle ID | Badge | Auto layer |
|---------------|-------|------------|
| `com.blizzard.worldofwarcraft` | WoW | WoW |
| `com.blizzard.worldofwarcraft.classic` | WoW | WoW |
| `com.riotgames.LeagueofLegends.GameClient` | LoL | LoL |
| `com.riotgames.LeagueofLegends.LeagueClient` | LoL | off |
| anything else | Buddy | Base |

Protocol details: [../modules/app-layer/PROTOCOL.md](../modules/app-layer/PROTOCOL.md)
