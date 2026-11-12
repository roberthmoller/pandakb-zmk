# pandakb-app-layer

Firmware pieces for the PandaKB macOS companion. Built via the repo-root
`zephyr/module.yml` (`ZMK_EXTRA_MODULES` in CI), not as a separate west project.

- Raw HID commands on the central half (`layer_to`, `set_badge` with optional stats)
- Global-locality `&badge` behavior so the peripheral OLED syncs badge + packed stats
- Right OLED: static buddy (idle), game icons + KDA/CS when present
- Shields live in `boards/shields/app_layer` and `boards/shields/game_badge`

See [PROTOCOL.md](PROTOCOL.md).
