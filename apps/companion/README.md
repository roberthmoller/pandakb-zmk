# PandaKB Companion

macOS menu-bar / tray app that drives the right OLED over Raw HID and hosts the Keymap Wizard UI.

## Run

```bash
cd apps/companion
npm install
npm run tauri:dev
```

The tray title shows the active widget. Left-click opens the **Flash** window (learn/trainer UI is hidden for now).

### Flash

One button: **Flash keyboard**. Builds via native [`scripts/build-companion.sh`](../../scripts/build-companion.sh) (`west` on PATH — no Docker), soft-enters bootloader over Raw HID, installs both halves. A short wizard appears only if something needs a cable or a double-reset.

CLI:

```bash
./scripts/build-companion.sh          # build → artifacts/Lily58_*_oled.uf2
./scripts/build-companion.sh --force  # wipe/rebuild
WEST_ROOT=~/.zmk ./scripts/build-companion.sh
```

Flash always installs from this repo’s `artifacts/Lily58_*_oled.uf2`. `~/.zmk` is only the west build cache (created with `west init -l config` + `west update` on first rebuild). Needs `west`, Zephyr SDK (`ZEPHYR_SDK_INSTALL_DIR`), cmake, ninja.

- Prefer both halves on USB; one cable still works via the wizard.
- Soft-enter needs firmware with Raw HID `0x30` (first flash / recovery: double-reset).
- Artifact paths, west cache, force rebuild, and build log live under **Advanced**.

### Widgets (priority)

1. **LoL** — Live Client API at `https://127.0.0.1:2999` while in-game ([what we can read](LOL_LIVE_CLIENT.md))
2. **IDE** — `POST http://127.0.0.1:17302/ide` with `{ "errors", "warnings", "build", "tag", "active" }`
3. **Media** — Spotify / Music now-playing + system volume
4. **Idle** — static marker when nothing else is active

System volume changes briefly override that priority (~1.8s of MEDIA) so the right OLED can show a full-body volume HUD. Widget poll interval is 500ms.

Protocol: [`../../modules/app-layer/PROTOCOL.md`](../../modules/app-layer/PROTOCOL.md)

## Firmware

Flash the **companion HID OLED pair** (not the stock OLED pair):

| Half | Artifact | Shields |
|------|----------|---------|
| **Left** (plug USB here) | `Lily58_left_oled.uf2` | `Lily58_L nice_oled raw_hid_adapter app_layer` |
| **Right** | `Lily58_right_oled.uf2` | `Lily58_R game_badge` |

1. Use **Flash keyboard** in the companion (local west/Docker build), or double-reset and copy the matching `.uf2` manually.
2. Build includes `app_layer` + `game_badge` (see `build.yaml` artifacts `Lily58_left_oled` / `Lily58_right_oled`).
3. After flash, leave the **left** half connected over USB so Raw HID reaches the companion.

Stock fallbacks (`Lily58_*_oled_stock.uf2`) are nice_oled only — no widget HID.

### Keymap source of truth

Companion layers / layouts are parsed from the repo file:

`config/Lily58.keymap`

Tauri commands `firmware_keymap_get` / `firmware_keymap_set` read and write that path. On launch (and each overlay show) the app reloads it so the UI matches what you flash. The bundled `src/lib/boards/lily58/keymap.json` is only a fallback for non-Tauri / offline use — regenerate with `npx vite-node scripts/regen-keymap.ts`.

### Layer layout preview

LOWER / RAISE use `&mo_prev` (momentary, like `&mo`). **Double-tap and hold** a layer key to show a floating glass overlay of that layer’s layout; release to dismiss. **Adjust** appears automatically when LOWER+RAISE are held together; **WoW / LoL** appear while those sticky `&to` layers are active. Firmware sends Raw HID `0x20 LAYER_PREVIEW` plus `0x21 KEY_MATRIX` (pressed positions) so the overlay highlights the key you’re holding. Overlay content comes from the local keymap above (Base / Lower / Raise / Adjust / WoW / LoL).

### HID smoke check

```bash
swift scripts/check-hid.swift
```

Expects the left half flashed with the companion OLED build and plugged over USB.

### MCP (agents)

While the tray app is running it serves a Streamable-HTTP MCP endpoint on localhost:

`http://127.0.0.1:17303/mcp`

Override the port with `PANDAKB_MCP_PORT`. Health JSON is at `/`.

**Tools:** `companion_status`, `companion_push_idle`, `companion_enter_bootloader`, `companion_set_layer`, `firmware_status`, `firmware_west_update`, `firmware_build`, `firmware_flash`, `firmware_wait_bootloader`, `firmware_keymap_get`, `firmware_keymap_set`, `flash_keyboard`.

`companion_set_layer` needs left-half firmware with Raw HID `0x31` (sticky `&to`: `wow`, `lol`, `base`, …).

Wired for BB agents on this machine:

| Client | Config |
|--------|--------|
| Cursor / `acp-cursor` (project) | [`.cursor/mcp.json`](../../.cursor/mcp.json) |
| Cursor (user) | `~/.cursor/mcp.json` → `pandakb` |
| Codex | `~/.codex/config.toml` → `[mcp_servers.pandakb]` |

URL transport — companion must already be open. Reload the BB thread (or start a new one) if the `pandakb` tools do not appear yet.

Typical agent loop: `firmware_keymap_get` → edit → `firmware_keymap_set` → `flash_keyboard`. If a half needs a cable or double-reset, `flash_keyboard` returns a wizard payload; finish that half with `firmware_flash` after the bootloader mounts.
