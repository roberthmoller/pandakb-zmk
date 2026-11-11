/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>

#include <zmk/behavior.h>
#include <zmk/event_manager.h>
#include <zmk/keymap.h>

#include <raw_hid/events.h>

#include <pandakb/badge.h>
#include <pandakb/protocol.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

#if DT_NODE_EXISTS(DT_NODELABEL(badge))

static int invoke_badge(uint8_t badge) {
    struct zmk_behavior_binding binding = {
        .behavior_dev = DEVICE_DT_NAME(DT_NODELABEL(badge)),
        .param1 = badge,
        .param2 = 0,
    };
    struct zmk_behavior_binding_event event = {
        .layer = 0,
        .position = 0,
        .timestamp = k_uptime_get(),
#if IS_ENABLED(CONFIG_ZMK_SPLIT)
        .source = 0,
#endif
    };

    return zmk_behavior_invoke_binding(&binding, event, true);
}

#else

static int invoke_badge(uint8_t badge) { return pandakb_badge_set(badge); }

#endif

static bool layer_allowed(uint8_t layer) {
    return layer == PANDAKB_LAYER_BASE || layer == PANDAKB_LAYER_WOW || layer == PANDAKB_LAYER_LOL;
}

static int on_raw_hid_received(const zmk_event_t *eh) {
    const struct raw_hid_received_event *ev = as_raw_hid_received_event(eh);
    if (ev == NULL || ev->data == NULL || ev->length < 2) {
        return ZMK_EV_EVENT_BUBBLE;
    }

    const uint8_t cmd = ev->data[0];
    const uint8_t arg = ev->data[1];

    switch (cmd) {
    case PANDAKB_HID_CMD_LAYER_TO:
        if (!layer_allowed(arg)) {
            LOG_WRN("reject layer_to %u (sticky Base/WoW/LoL only)", arg);
            break;
        }
        LOG_DBG("layer_to %u", arg);
        zmk_keymap_layer_to(arg);
        break;
    case PANDAKB_HID_CMD_SET_BADGE:
        LOG_DBG("set_badge %u", arg);
        invoke_badge(arg);
        break;
    default:
        break;
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_raw_hid_commands, on_raw_hid_received);
ZMK_SUBSCRIPTION(pandakb_raw_hid_commands, raw_hid_received_event);
