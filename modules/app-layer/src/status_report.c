/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 *
 * Reports left/right battery + active layer to the companion over Raw HID.
 */

#include <string.h>

#include <zephyr/init.h>
#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/sys/util.h>

#include <zmk/battery.h>
#include <zmk/event_manager.h>
#include <zmk/events/battery_state_changed.h>
#include <zmk/events/layer_state_changed.h>
#include <zmk/keymap.h>

#include <pandakb/protocol.h>
#include <pandakb/status.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

#if !IS_ENABLED(CONFIG_ZMK_SPLIT) || IS_ENABLED(CONFIG_ZMK_SPLIT_ROLE_CENTRAL)
#define PANDAKB_STATUS_HAS_KEYMAP 1
#else
#define PANDAKB_STATUS_HAS_KEYMAP 0
#endif

#if PANDAKB_STATUS_HAS_KEYMAP && IS_ENABLED(CONFIG_RAW_HID)

/* zmk_peripheral_battery_state_changed lives in battery_state_changed.h (ZMK v0.3). */

#include <raw_hid/events.h>

#define STATUS_PERIOD_MS 15000

static uint8_t left_soc = PANDAKB_BATTERY_UNKNOWN;
static uint8_t right_soc = PANDAKB_BATTERY_UNKNOWN;
static uint8_t active_layer;
static uint8_t last_sent[4];
static bool have_sent;

static void send_raw(const uint8_t *data, size_t len) {
    uint8_t buf[32] = {0};
    if (len > sizeof(buf)) {
        len = sizeof(buf);
    }
    memcpy(buf, data, len);
    raise_raw_hid_sent_event((struct raw_hid_sent_event){
        .data = buf,
        .length = sizeof(buf),
    });
}

static void send_status_now(bool force) {
    uint8_t data[4] = {
        PANDAKB_HID_CMD_STATUS,
        left_soc,
        right_soc,
        active_layer,
    };
    if (!force && have_sent && memcmp(last_sent, data, sizeof(data)) == 0) {
        return;
    }
    send_raw(data, sizeof(data));
    memcpy(last_sent, data, sizeof(data));
    have_sent = true;
}

void pandakb_status_send(void) { send_status_now(true); }

static void refresh_layer(void) {
    active_layer = (uint8_t)zmk_keymap_highest_layer_active();
}

static void status_loop_handler(struct k_work *work);
static K_WORK_DELAYABLE_DEFINE(status_loop, status_loop_handler);

static void status_loop_handler(struct k_work *work) {
    ARG_UNUSED(work);
    refresh_layer();
    if (left_soc == PANDAKB_BATTERY_UNKNOWN) {
        left_soc = zmk_battery_state_of_charge();
    }
    send_status_now(true);
    k_work_schedule(&status_loop, K_MSEC(STATUS_PERIOD_MS));
}

static int on_battery_changed(const zmk_event_t *eh) {
    const struct zmk_battery_state_changed *ev = as_zmk_battery_state_changed(eh);
    left_soc = (ev != NULL) ? ev->state_of_charge : zmk_battery_state_of_charge();
    send_status_now(false);
    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_status_battery, on_battery_changed);
ZMK_SUBSCRIPTION(pandakb_status_battery, zmk_battery_state_changed);

#if IS_ENABLED(CONFIG_ZMK_SPLIT) && IS_ENABLED(CONFIG_ZMK_SPLIT_ROLE_CENTRAL)
static int on_peripheral_battery_changed(const zmk_event_t *eh) {
    const struct zmk_peripheral_battery_state_changed *ev =
        as_zmk_peripheral_battery_state_changed(eh);
    if (ev == NULL) {
        return ZMK_EV_EVENT_BUBBLE;
    }
    /* Lily58 has a single peripheral (right half). */
    if (ev->source == 0) {
        right_soc = ev->state_of_charge;
        send_status_now(false);
    }
    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_status_peripheral_battery, on_peripheral_battery_changed);
ZMK_SUBSCRIPTION(pandakb_status_peripheral_battery, zmk_peripheral_battery_state_changed);
#endif

static int on_layer_state_changed(const zmk_event_t *eh) {
    ARG_UNUSED(eh);
    refresh_layer();
    send_status_now(false);
    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_status_layer, on_layer_state_changed);
ZMK_SUBSCRIPTION(pandakb_status_layer, zmk_layer_state_changed);

static int status_init(void) {
    left_soc = zmk_battery_state_of_charge();
    refresh_layer();
    send_status_now(true);
    k_work_schedule(&status_loop, K_MSEC(STATUS_PERIOD_MS));
    return 0;
}

SYS_INIT_NAMED(pandakb_status_report, status_init, APPLICATION, 90);

#else /* !keymap or !RAW_HID */

void pandakb_status_send(void) {}

#endif
