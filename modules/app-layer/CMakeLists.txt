# Layer-preview momentary behavior is always available so the shared keymap can
# reference &mo_prev even on builds without the app_layer shield / Raw HID.
zephyr_library()
zephyr_include_directories(include)
zephyr_include_directories(${APPLICATION_SOURCE_DIR}/include)
zephyr_include_directories(src)
zephyr_library_sources(src/behavior_mo_preview.c)

if(CONFIG_PANDAKB_APP_LAYER)
  zephyr_library_sources(src/badge_state.c)
  zephyr_library_sources(src/behavior_game_badge.c)
  zephyr_library_sources_ifdef(CONFIG_PANDAKB_APP_LAYER_RAW_HID src/raw_hid_commands.c)
  if(CONFIG_PANDAKB_GAME_BADGE_SCREEN)
    zephyr_library_sources(src/badge_screen.c)
  endif()
endif()
