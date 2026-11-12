/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <string.h>

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

static int invoke_widget_status(const struct pandakb_badge_status *status) {
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

    struct pandakb_badge_status status;
    memset(&status, 0, sizeof(status));
    status.mode = PANDAKB_WIDGET_MEDIA;
    status.u.media.playback = data[1];
    status.u.media.volume = data[2] > 100 ? 100 : data[2];

    LOG_DBG("set_media playback=%u vol=%u", status.u.media.playback, status.u.media.volume);
    invoke_widget_status(&status);
}

static void parse_set_widget(const uint8_t *data, uint8_t length) {
    if (length < 2) {
        return;
    }

    struct pandakb_badge_status status;
    memset(&status, 0, sizeof(status));
    status.mode = data[1];

    uint8_t copy = (length > 9) ? 7 : (uint8_t)((length > 2) ? (length - 2) : 0);
    if (copy > 7) {
        copy = 7;
    }
    if (copy > 0) {
        memcpy(status.u.raw, &data[2], copy);
    }

    if (status.mode == PANDAKB_WIDGET_MEDIA && status.u.media.volume > 100) {
        status.u.media.volume = 100;
    }

    LOG_DBG("set_widget mode=%u", status.mode);
    invoke_widget_status(&status);
}

static int on_raw_hid_received(const zmk_event_t *eh) {
    const struct raw_hid_received_event *ev = as_raw_hid_received_event(eh);
    if (ev == NULL || ev->data == NULL || ev->length < 1) {
        return ZMK_EV_EVENT_BUBBLE;
    }

    switch (ev->data[0]) {
    case PANDAKB_HID_CMD_SET_WIDGET:
        parse_set_widget(ev->data, ev->length);
        break;
    case PANDAKB_HID_CMD_SET_MEDIA:
        parse_set_media(ev->data, ev->length);
        break;
    default:
        break;
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_raw_hid_commands, on_raw_hid_received);
ZMK_SUBSCRIPTION(pandakb_raw_hid_commands, raw_hid_received_event);
