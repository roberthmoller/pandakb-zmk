/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 */

#include <errno.h>
#include <string.h>

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/sys/reboot.h>

#if IS_ENABLED(CONFIG_RETENTION_BOOT_MODE)
#include <zephyr/retention/bootmode.h>
#endif

#if defined(CONFIG_SOC_SERIES_NRF52X)
#include <hal/nrf_power.h>
#endif

#include <zmk/behavior.h>
#include <zmk/event_manager.h>
#include <zmk/keymap.h>

#include <raw_hid/events.h>

#include <pandakb/badge.h>
#include <pandakb/protocol.h>
#include <pandakb/status.h>

/* Keymap layer APIs exist on non-split and split centrals only. */
#if !IS_ENABLED(CONFIG_ZMK_SPLIT) || IS_ENABLED(CONFIG_ZMK_SPLIT_ROLE_CENTRAL)
#define PANDAKB_SET_LAYER_HAS_KEYMAP 1
#else
#define PANDAKB_SET_LAYER_HAS_KEYMAP 0
#endif

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

/* Adafruit nRF52 UF2 bootloader magic (GPREGRET). */
#define PANDAKB_UF2_GPREGRET 0x57

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

static void enter_uf2_bootloader_local(void) {
    LOG_INF("Entering UF2 bootloader");
    /* Let the HID stack finish the write before we reset. */
    k_msleep(40);

#if IS_ENABLED(CONFIG_RETENTION_BOOT_MODE)
    if (bootmode_set(BOOT_MODE_TYPE_BOOTLOADER) == 0) {
        sys_reboot(SYS_REBOOT_WARM);
        return;
    }
    LOG_WRN("bootmode_set failed; falling back to GPREGRET");
#endif

#if defined(CONFIG_SOC_SERIES_NRF52X)
    /* nrfx: (reg, gpregret_id, value) — Adafruit UF2 looks at GPREGRET[0]. */
    nrf_power_gpregret_set(NRF_POWER, 0, PANDAKB_UF2_GPREGRET);
#endif
    sys_reboot(SYS_REBOOT_WARM);
}

static int enter_uf2_bootloader_peripheral(void) {
#if IS_ENABLED(CONFIG_ZMK_SPLIT) && IS_ENABLED(CONFIG_ZMK_SPLIT_ROLE_CENTRAL) &&                    \
    DT_NODE_EXISTS(DT_NODELABEL(bootloader))
    struct zmk_behavior_binding binding = {
        .behavior_dev = DEVICE_DT_NAME(DT_NODELABEL(bootloader)),
        .param1 = 0,
        .param2 = 0,
    };
    struct zmk_behavior_binding_event event = {
        .layer = 0,
        .position = 0,
        .timestamp = k_uptime_get(),
        .source = 0, /* first peripheral */
    };

    LOG_INF("Requesting UF2 bootloader on peripheral");
    return zmk_behavior_invoke_binding(&binding, event, true);
#else
    LOG_WRN("Peripheral bootloader relay unavailable");
    return -ENOTSUP;
#endif
}

static void parse_enter_bootloader(const uint8_t *data, uint8_t length) {
    uint8_t half = PANDAKB_BOOTLOADER_SELF;
    if (length >= 2) {
        half = data[1];
    }

    if (half == PANDAKB_BOOTLOADER_RIGHT) {
        if (enter_uf2_bootloader_peripheral() < 0) {
            LOG_WRN("Right bootloader failed — double-reset that half");
        }
        return;
    }

    /* SELF / LEFT: this central half. */
    enter_uf2_bootloader_local();
}

static void parse_set_layer(const uint8_t *data, uint8_t length) {
    if (length < 2) {
        return;
    }

#if PANDAKB_SET_LAYER_HAS_KEYMAP
    zmk_keymap_layer_index_t index = data[1];
    zmk_keymap_layer_id_t id = zmk_keymap_layer_index_to_id(index);
    if (id == ZMK_KEYMAP_LAYER_ID_INVAL) {
        LOG_WRN("set_layer: invalid index %u", index);
        return;
    }
    LOG_INF("set_layer index=%u id=%u", index, id);
    /* Sticky `&to` semantics (not momentary). */
    (void)zmk_keymap_layer_to(id);
#else
    ARG_UNUSED(data);
    LOG_WRN("set_layer unavailable on this half");
#endif
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
    case PANDAKB_HID_CMD_ENTER_BOOTLOADER:
        parse_enter_bootloader(ev->data, ev->length);
        break;
    case PANDAKB_HID_CMD_SET_LAYER:
        parse_set_layer(ev->data, ev->length);
        break;
    case PANDAKB_HID_CMD_GET_STATUS:
        pandakb_status_send();
        break;
    default:
        break;
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_raw_hid_commands, on_raw_hid_received);
ZMK_SUBSCRIPTION(pandakb_raw_hid_commands, raw_hid_received_event);
