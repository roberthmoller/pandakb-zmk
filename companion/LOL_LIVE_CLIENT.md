# League of Legends — Live Client data

While you are in a LoL game on this Mac, Riot exposes a local HTTPS API:

```text
https://127.0.0.1:2999/liveclientdata/...
```

Self-signed cert (the companion accepts it). Only reachable from localhost, and only while the game client is running.

Primary dump used by PandaKB:

```text
GET /liveclientdata/allgamedata
```

Sample payload: [liveclientdata_sample.json](https://static.developer.riotgames.com/docs/lol/liveclientdata_sample.json)  
Docs: [Riot LoL Live Client Data](https://developer.riotgames.com/docs/lol)

## What PandaKB shows today

The companion polls `allgamedata`, finds the local player, and pushes widget mode `LOL` over Raw HID. The right OLED shows:

| Shown | Source | Layout (32px wide) |
|-------|--------|--------------------|
| K / D / A | `allPlayers[].scores.kills/deaths/assists` | Three stacked numbers (no `10/2/0` single line) |
| CS | `allPlayers[].scores.creepScore` | Number only under a hairline (no `CS` prefix) |
| Level | `activePlayer.level` (fallback: player `level`) | `L12` |

The OLED title `"LoL"` is not drawn (stats only). The HID payload for mode `LOL` is 7 bytes — see [PROTOCOL.md](../modules/app-layer/PROTOCOL.md).

## What the API can provide (unused today)

### Active player (`activePlayer` / `/activeplayer`)

- Summoner / Riot ID name
- Level, current gold
- Full runes
- Ability ranks (Q/W/E/R + passive)
- `championStats`: HP / max HP, resource (mana etc.), AD, AP, armor, MR, attack speed, move speed, lethality / pen, crit, lifesteal, tenacity, …

### Other players (`allPlayers` / `/playerlist`)

- Champion name, skin, team (ORDER / CHAOS), position
- Level, scores (K/D/A, CS, ward score)
- Items, summoner spells, runes (also via dedicated endpoints)
- Bot flag, respawn / dead state where present

### Match (`gameData` / `/gamestats`)

- Game mode, map, game time

### Events (`events` / `/eventdata`)

Chronological feed (also embedded in `allgamedata.events.Events`). Kill-related names:

| EventName | Notes |
|-----------|--------|
| `ChampionKill` | `KillerName`, `VictimName`, `Assisters[]` |
| `Multikill` | `KillerName`, `KillStreak` (2=double … 5=penta) |
| `FirstBlood` | `Recipient` (same time as the kill) |
| `Ace` | `Acer`, `AcingTeam` |

Also: dragon / baron / herald, turret / inhib, game start / end. Companion does not consume these yet — a streak flash would need spare flag bits in the 7-byte LoL payload plus a short firmware overlay timer.

### Focused endpoints (same data, smaller responses)

| Endpoint | Purpose |
|----------|---------|
| `/activeplayer` | Local player blob |
| `/activeplayerabilities` | QWER ranks |
| `/activeplayerrunes` | Runes |
| `/playerlist` | Everyone |
| `/playerscores?summonerName=` | KDA / CS / wards |
| `/playeritems?summonerName=` | Item list |
| `/playersummonerspells?summonerName=` | D / F |
| `/eventdata` | Event stream |
| `/gamestats` | Mode / map / time |

## OLED constraints

Right OLED is a narrow portrait column (~32×128 logical). Mode `LOL` only carries **7 payload bytes** over the split `badge` sync. Adding gold, HP, or events means:

1. Replacing an existing field (e.g. drop level for gold), or
2. Rotating pages on a timer, or
3. Expanding the protocol / packing scheme

## Firmware needed for the widget

Flash the companion HID OLED pair (left Raw HID + right `game_badge`):

| Half | Artifact | Shields |
|------|----------|---------|
| Left (USB) | `Lily58_left_oled.uf2` | `Lily58_L nice_oled raw_hid_adapter app_layer` |
| Right | `Lily58_right_oled.uf2` | `Lily58_R game_badge` |

Then run the Tauri companion (`companion/zmkwiz` → `npm run tauri:dev`) and join a game. Tray title switches to LoL stats when the Live Client responds.
