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

#include <raw_hid/events.h>

#include <pandakb/badge.h>
#include <pandakb/protocol.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

static int invoke_media_status(const struct pandakb_badge_status *status) {
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

static void parse_set_media(const uint8_t *data, uint8_t length) {
    if (length < 3) {
        return;
    }

    struct pandakb_badge_status status = {
        .state = data[1],
        .volume = data[2] > 100 ? 100 : data[2],
    };

    LOG_DBG("set_media state=%u vol=%u", status.state, status.volume);
    invoke_media_status(&status);
}

static int on_raw_hid_received(const zmk_event_t *eh) {
    const struct raw_hid_received_event *ev = as_raw_hid_received_event(eh);
    if (ev == NULL || ev->data == NULL || ev->length < 1) {
        return ZMK_EV_EVENT_BUBBLE;
    }

    switch (ev->data[0]) {
    case PANDAKB_HID_CMD_SET_MEDIA:
        parse_set_media(ev->data, ev->length);
        break;
    default:
        /* Ignore legacy layer/badge commands — companion is media-only now. */
        break;
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_raw_hid_commands, on_raw_hid_received);
ZMK_SUBSCRIPTION(pandakb_raw_hid_commands, raw_hid_received_event);
