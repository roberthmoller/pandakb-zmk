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

static int invoke_badge_status(const struct pandakb_badge_status *status) {
#if DT_NODE_EXISTS(DT_NODELABEL(badge))
    struct zmk_behavior_binding binding = {
        .behavior_dev = DEVICE_DT_NAME(DT_NODELABEL(badge)),
        .param1 = pandakb_badge_pack_param1(status),
        .param2 = pandakb_badge_pack_param2(status),
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
#else
    return pandakb_badge_set_status(status);
#endif
}

static bool layer_allowed(uint8_t layer) {
    return layer == PANDAKB_LAYER_BASE || layer == PANDAKB_LAYER_WOW || layer == PANDAKB_LAYER_LOL;
}

static void parse_set_badge(const uint8_t *data, uint8_t length) {
    struct pandakb_badge_status status = {
        .badge = data[1],
    };

    /* Optional stats: [2]=k [3]=d [4]=a [5]=cs_lo [6]=cs_hi [7]=level [8]=flags */
    if (length >= 9) {
        status.kills = data[2];
        status.deaths = data[3];
        status.assists = data[4];
        status.cs = (uint16_t)data[5] | ((uint16_t)data[6] << 8);
        status.level = data[7];
        status.flags = data[8];
    }

    LOG_DBG("set_badge %u flags=0x%02x", status.badge, status.flags);
    invoke_badge_status(&status);
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
        parse_set_badge(ev->data, ev->length);
        break;
    default:
        break;
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_raw_hid_commands, on_raw_hid_received);
ZMK_SUBSCRIPTION(pandakb_raw_hid_commands, raw_hid_received_event);
