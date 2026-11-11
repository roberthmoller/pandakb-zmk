# PandaKB Companion (macOS)

Menu bar app that:

1. Watches the frontmost app
2. Sets a **game badge** on the right OLED (`idle` / `WoW` / `LoL`)
3. Optionally sticky-switches keyboard layers (`Base` / `WoW` / `LoL`)

Badge and layer are independent. Left OLED keeps its normal `nice_oled` layout (including layer name).

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
| anything else | Idle | Base |

Protocol details: [../modules/app-layer/PROTOCOL.md](../modules/app-layer/PROTOCOL.md)
