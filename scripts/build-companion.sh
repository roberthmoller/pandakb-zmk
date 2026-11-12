#!/usr/bin/env bash
# Build the PandaKB companion OLED pair with native `west` → artifacts/.
#
# Usage:
#   ./scripts/build-companion.sh
#   ./scripts/build-companion.sh --force
#   WEST_ROOT=~/.zmk ./scripts/build-companion.sh
#
# Default WEST_ROOT is ~/.zmk. Syncs this repo's config/boards/modules into it,
# and west init + west update if the workspace is missing.
#
# Requires: west on PATH, Zephyr SDK (ZEPHYR_SDK_INSTALL_DIR), cmake, ninja.
# No Docker.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WEST_ROOT="${WEST_ROOT:-$HOME/.zmk}"
# Expand leading ~
case "$WEST_ROOT" in
  "~") WEST_ROOT="$HOME" ;;
  "~/"*) WEST_ROOT="$HOME/${WEST_ROOT#~/}" ;;
esac
FORCE=0

for arg in "$@"; do
  case "$arg" in
    --force|-f) FORCE=1 ;;
    --help|-h)
      sed -n '2,14p' "$0"
      exit 0
      ;;
  esac
done

LEFT_DIR=build/lily58_left_oled
RIGHT_DIR=build/lily58_right_oled
# ZMK v0.3 / Zephyr 3.5 board id (matches build.yaml; not nice_nano/nrf52840)
BOARD="${BOARD:-nice_nano_v2}"
LEFT_SHIELD='Lily58_L nice_oled raw_hid_adapter app_layer'
RIGHT_SHIELD='Lily58_R game_badge'

log() { printf '%s\n' "$*"; }
die() { log "error: $*"; exit 1; }

require_toolchain() {
  command -v west >/dev/null 2>&1 || die "west not on PATH.

Install (once):
  pip3 install --user west
  # ensure \"\$(python3 -m site --user-base)/bin\" is on PATH

  # Zephyr SDK (arm-zephyr-eabi):
  #   https://docs.zephyrproject.org/latest/develop/getting_started/index.html
  # then: export ZEPHYR_SDK_INSTALL_DIR=~/zephyr-sdk-<ver>

Also need: cmake, ninja, and usually dtc (brew install cmake ninja dtc)."

  command -v cmake >/dev/null 2>&1 || die "cmake not on PATH (brew install cmake)"
  command -v ninja >/dev/null 2>&1 || die "ninja not on PATH (brew install ninja)"
}

source_fingerprint() {
  find "$ROOT/config" "$ROOT/boards" "$ROOT/modules/app-layer" "$ROOT/build.yaml" \
    -type f ! -name '.DS_Store' -print0 2>/dev/null | xargs -0 stat -f '%m' 2>/dev/null | sort -n | tail -1 || true
}

write_stamps() {
  local now="$1"
  mkdir -p "$ROOT/artifacts" "$WEST_ROOT/build"
  printf '%s\n' "$now" >"$ROOT/artifacts/.pandakb-companion.stamp"
  printf '%s\n' "$now" >"$WEST_ROOT/build/pandakb-companion.stamp"
}

# Flash source of truth is $ROOT/artifacts/*.uf2 — ~/.zmk is only the west cache.
need_rebuild() {
  [[ "$FORCE" == 1 ]] && return 0
  local left="$ROOT/artifacts/Lily58_left_oled.uf2"
  local right="$ROOT/artifacts/Lily58_right_oled.uf2"
  [[ -f "$left" && -f "$right" ]] || return 0
  local stamp=""
  if [[ -f "$ROOT/artifacts/.pandakb-companion.stamp" ]]; then
    stamp="$ROOT/artifacts/.pandakb-companion.stamp"
  elif [[ -f "$WEST_ROOT/build/pandakb-companion.stamp" ]]; then
    stamp="$WEST_ROOT/build/pandakb-companion.stamp"
  else
    return 0
  fi
  local now
  now="$(source_fingerprint)"
  [[ -n "$now" && "$(cat "$stamp")" == "$now" ]] && return 1
  return 0
}

sync_into_west() {
  log "Syncing $ROOT → $WEST_ROOT"
  mkdir -p "$WEST_ROOT/config" "$WEST_ROOT/boards" "$WEST_ROOT/config/boards/shields" "$WEST_ROOT/modules"
  rsync -a --delete --exclude '.DS_Store' "$ROOT/config/" "$WEST_ROOT/config/"
  rsync -a --delete --exclude '.DS_Store' "$ROOT/boards/" "$WEST_ROOT/boards/"
  for shield in Lily58 app_layer game_badge; do
    if [[ -d "$ROOT/boards/shields/$shield" ]]; then
      rm -rf "$WEST_ROOT/config/boards/shields/$shield"
      rsync -a --exclude '.DS_Store' "$ROOT/boards/shields/$shield/" "$WEST_ROOT/config/boards/shields/$shield/"
    fi
  done
  rm -rf "$WEST_ROOT/modules/app-layer"
  rsync -a --exclude '.DS_Store' "$ROOT/modules/app-layer/" "$WEST_ROOT/modules/app-layer/"
  mkdir -p "$WEST_ROOT/modules/app-layer/zephyr"
  cat >"$WEST_ROOT/modules/app-layer/zephyr/module.yml" <<'EOF'
name: pandakb-app-layer
build:
  cmake: .
  kconfig: Kconfig
  settings:
    dts_root: .
EOF
  if [[ -f "$WEST_ROOT/.west/config" ]] && grep -q 'base = /work/zephyr' "$WEST_ROOT/.west/config"; then
    sed -i.bak 's|base = /work/zephyr|base = zephyr|' "$WEST_ROOT/.west/config"
    rm -f "$WEST_ROOT/.west/config.bak"
  fi
}

run_west() {
  log "$ west $*"
  (
    cd "$WEST_ROOT"
    export ZEPHYR_BASE="${ZEPHYR_BASE:-$WEST_ROOT/zephyr}"
    west "$@"
  )
}

# Docker builds bake /work/... into CMakeCache; west -p always then calls
# /work/zephyr/cmake/pristine.cmake and fails. Wipe the dir instead.
wipe_build_dir_if_needed() {
  local dir="$1"
  local abs="$WEST_ROOT/$dir"
  [[ -d "$abs" ]] || return 0
  local cache="$abs/CMakeCache.txt"
  local wipe=0
  if [[ "$FORCE" == 1 ]]; then
    wipe=1
  elif [[ -f "$cache" ]] && grep -qE '(^|[=:])/work/' "$cache" 2>/dev/null; then
    wipe=1
  elif [[ -f "$cache" ]]; then
    local cached
    cached="$(grep -E '^APPLICATION_SOURCE_DIR:' "$cache" | head -1 | cut -d= -f2- || true)"
    if [[ -n "$cached" && "$cached" != "$WEST_ROOT/zmk/app" ]]; then
      wipe=1
    fi
  fi
  if [[ "$wipe" == 1 ]]; then
    log "removing stale build dir $dir (Docker paths or --force)"
    rm -rf "$abs"
  fi
}

build_half() {
  local dir="$1" shield="$2"
  shift 2
  local extras="$WEST_ROOT/zmk/app/module;$WEST_ROOT/zmk/app/keymap-module;$WEST_ROOT/modules/app-layer"
  wipe_build_dir_if_needed "$dir"
  run_west build -s zmk/app -d "$dir" -b "$BOARD" -- \
    "-DSHIELD=${shield}" \
    "-DZMK_CONFIG=${WEST_ROOT}/config" \
    "-DZEPHYR_EXTRA_MODULES=${extras}" \
    "$@"
}

copy_artifacts() {
  local left_src="$WEST_ROOT/$LEFT_DIR/zephyr/zmk.uf2"
  local right_src="$WEST_ROOT/$RIGHT_DIR/zephyr/zmk.uf2"
  [[ -f "$left_src" ]] || die "missing $left_src"
  [[ -f "$right_src" ]] || die "missing $right_src"
  mkdir -p "$ROOT/artifacts/flash-companion-oled"
  cp -f "$left_src" "$ROOT/artifacts/Lily58_left_oled.uf2"
  cp -f "$right_src" "$ROOT/artifacts/Lily58_right_oled.uf2"
  cp -f "$ROOT/artifacts/Lily58_left_oled.uf2" "$ROOT/artifacts/flash-companion-oled/"
  cp -f "$ROOT/artifacts/Lily58_right_oled.uf2" "$ROOT/artifacts/flash-companion-oled/"
  log "Artifacts (flash these):"
  ls -la "$ROOT/artifacts/Lily58_left_oled.uf2" "$ROOT/artifacts/Lily58_right_oled.uf2"
}

# Drop CMake user-package entries that point at other west trees (e.g. old
# Docker/Codex paths). Otherwise find_package(Zephyr) can load the wrong tree
# and fail with "Invalid BOARD".
scrub_cmake_zephyr_registry() {
  local pkg want
  want="$WEST_ROOT/zephyr/share/zephyr-package/cmake"
  for pkg in Zephyr ZephyrUnittest; do
    local dir="$HOME/.cmake/packages/$pkg"
    [[ -d "$dir" ]] || continue
    local f
    for f in "$dir"/*; do
      [[ -f "$f" ]] || continue
      local pointed
      pointed="$(tr -d '\n' <"$f" 2>/dev/null || true)"
      if [[ -n "$pointed" && "$pointed" != "$want" && "$pointed" != "$WEST_ROOT/zephyr/share/zephyrunittest-package/cmake" ]]; then
        # Keep entries under this WEST_ROOT; remove everything else.
        case "$pointed" in
          "$WEST_ROOT"/*) ;;
          *)
            log "removing stale CMake $pkg registry → $pointed"
            rm -f "$f"
            ;;
        esac
      fi
    done
  done
}

ensure_west_workspace() {
  mkdir -p "$WEST_ROOT"
  sync_into_west
  scrub_cmake_zephyr_registry

  if [[ ! -d "$WEST_ROOT/.west" ]]; then
    log "Initializing west workspace at $WEST_ROOT (west init -l config)"
    run_west init -l config
  fi

  if [[ ! -d "$WEST_ROOT/zmk" || ! -d "$WEST_ROOT/zephyr" ]]; then
    log "Fetching west projects into $WEST_ROOT (west update) — first run can take a while"
    run_west update
  fi

  [[ -d "$WEST_ROOT/zmk" ]] || die "Missing zmk/ under $WEST_ROOT after west update"
  [[ -d "$WEST_ROOT/zephyr" ]] || die "Missing zephyr/ under $WEST_ROOT after west update"
}

main() {
  require_toolchain
  log "Flash target: $ROOT/artifacts/Lily58_*_oled.uf2"
  log "WEST_ROOT=$WEST_ROOT (build cache only; sources from $ROOT)"

  if ! need_rebuild; then
    log "artifacts/ UF2s up to date — skipping rebuild (pass --force to rebuild)"
    log "LEFT_UF2=$ROOT/artifacts/Lily58_left_oled.uf2"
    log "RIGHT_UF2=$ROOT/artifacts/Lily58_right_oled.uf2"
    log "OK"
    exit 0
  fi

  ensure_west_workspace

  run_west zephyr-export || true

  log "── left companion OLED ──"
  build_half "$LEFT_DIR" "$LEFT_SHIELD" \
    -DCONFIG_ZMK_STUDIO=n \
    -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_CUSTOM=y \
    -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_BUILT_IN=n

  log "── right companion OLED ──"
  build_half "$RIGHT_DIR" "$RIGHT_SHIELD" \
    -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_CUSTOM=y \
    -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_BUILT_IN=n

  local now
  now="$(source_fingerprint)"
  [[ -n "$now" ]] || now=0
  copy_artifacts
  write_stamps "$now"
  log "LEFT_UF2=$ROOT/artifacts/Lily58_left_oled.uf2"
  log "RIGHT_UF2=$ROOT/artifacts/Lily58_right_oled.uf2"
  log "OK"
}

main
