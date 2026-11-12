/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#define DT_DRV_COMPAT zmk_behavior_mo_preview

#include <zephyr/device.h>
#include <zephyr/kernel.h>
#include <drivers/behavior.h>
#include <zephyr/logging/log.h>

#include <zmk/behavior.h>
#include <zmk/keymap.h>

#include <pandakb/protocol.h>

#if IS_ENABLED(CONFIG_RAW_HID)
#include <raw_hid/events.h>
#endif

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

#if DT_HAS_COMPAT_STATUS_OKAY(DT_DRV_COMPAT)

#ifndef CONFIG_PANDAKB_MO_PREVIEW_TAP_MS
#define CONFIG_PANDAKB_MO_PREVIEW_TAP_MS 300
#endif

struct mo_preview_state {
    int64_t last_release_ms;
    uint32_t last_position;
    bool armed;
    bool previewing;
};

static struct mo_preview_state state;

static void send_layer_preview(uint8_t show, uint8_t layer) {
#if IS_ENABLED(CONFIG_RAW_HID)
    uint8_t data[32] = {0};
    data[0] = PANDAKB_HID_CMD_LAYER_PREVIEW;
    data[1] = show;
    data[2] = layer;

    raise_raw_hid_sent_event((struct raw_hid_sent_event){
        .data = data,
        .length = sizeof(data),
    });
#else
    ARG_UNUSED(show);
    ARG_UNUSED(layer);
#endif
}

static int on_keymap_binding_pressed(struct zmk_behavior_binding *binding,
                                     struct zmk_behavior_binding_event event) {
    const int64_t now = k_uptime_get();
    const bool double_tap = state.armed && state.last_position == event.position &&
                            (now - state.last_release_ms) <= CONFIG_PANDAKB_MO_PREVIEW_TAP_MS;

    state.armed = false;

    LOG_DBG("mo_prev press pos %d layer %d double_tap %d", event.position, binding->param1,
            double_tap);

    if (double_tap) {
        state.previewing = true;
        send_layer_preview(PANDAKB_LAYER_PREVIEW_SHOW, (uint8_t)binding->param1);
    }

    return zmk_keymap_layer_activate(binding->param1, false);
}

static int on_keymap_binding_released(struct zmk_behavior_binding *binding,
                                      struct zmk_behavior_binding_event event) {
    LOG_DBG("mo_prev release pos %d layer %d previewing %d", event.position, binding->param1,
            state.previewing);

    if (state.previewing) {
        send_layer_preview(PANDAKB_LAYER_PREVIEW_HIDE, (uint8_t)binding->param1);
        state.previewing = false;
    }

    state.last_release_ms = k_uptime_get();
    state.last_position = event.position;
    state.armed = true;

    return zmk_keymap_layer_deactivate(binding->param1, false);
}

#if IS_ENABLED(CONFIG_ZMK_BEHAVIOR_METADATA)
static const struct behavior_parameter_value_metadata param_values[] = {
    {
        .display_name = "Layer",
        .type = BEHAVIOR_PARAMETER_VALUE_TYPE_LAYER_ID,
    },
};

static const struct behavior_parameter_metadata_set param_metadata_set[] = {{
    .param1_values = param_values,
    .param1_values_len = ARRAY_SIZE(param_values),
}};

static const struct behavior_parameter_metadata metadata = {
    .sets_len = ARRAY_SIZE(param_metadata_set),
    .sets = param_metadata_set,
};
#endif

static const struct behavior_driver_api behavior_mo_preview_driver_api = {
    .binding_pressed = on_keymap_binding_pressed,
    .binding_released = on_keymap_binding_released,
    .locality = BEHAVIOR_LOCALITY_CENTRAL,
#if IS_ENABLED(CONFIG_ZMK_BEHAVIOR_METADATA)
    .parameter_metadata = &metadata,
#endif
};

BEHAVIOR_DT_INST_DEFINE(0, NULL, NULL, NULL, NULL, POST_KERNEL, CONFIG_KERNEL_INIT_PRIORITY_DEFAULT,
                        &behavior_mo_preview_driver_api);

#endif /* DT_HAS_COMPAT_STATUS_OKAY(DT_DRV_COMPAT) */
