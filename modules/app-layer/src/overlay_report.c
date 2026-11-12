/*
 * Copyright (c) 2026 PandaKB
 * SPDX-License-Identifier: MIT
 *
 * Reports active layer + pressed key matrix to the companion overlay over Raw HID.
 */

#include <string.h>

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/sys/util.h>

#include <zmk/event_manager.h>
#include <zmk/events/layer_state_changed.h>
#include <zmk/events/position_state_changed.h>
#include <zmk/keymap.h>

#include <pandakb/overlay.h>
#include <pandakb/protocol.h>

LOG_MODULE_DECLARE(zmk, CONFIG_ZMK_LOG_LEVEL);

/* Keymap layer APIs exist on non-split and split centrals only. */
#if !IS_ENABLED(CONFIG_ZMK_SPLIT) || IS_ENABLED(CONFIG_ZMK_SPLIT_ROLE_CENTRAL)
#define PANDAKB_OVERLAY_HAS_KEYMAP 1
#else
#define PANDAKB_OVERLAY_HAS_KEYMAP 0
#endif

#if IS_ENABLED(CONFIG_RAW_HID)
#include <raw_hid/events.h>
#endif

#if PANDAKB_OVERLAY_HAS_KEYMAP && IS_ENABLED(CONFIG_RAW_HID)

#define KEY_MATRIX_BYTES 8 /* 64 positions — Lily58 uses 58 */

static bool previewing;
static bool showing;
static uint8_t shown_layer;
static uint8_t pressed[KEY_MATRIX_BYTES];

static bool name_is(const char *name, const char *want) {
    return name != NULL && want != NULL && strcmp(name, want) == 0;
}

static bool should_show_layer(zmk_keymap_layer_index_t highest) {
    if (highest == 0) {
        return false;
    }
    if (previewing) {
        return true;
    }
    /* Adjust (tri-layer) and sticky game layers surface without double-tap. */
    const char *name = zmk_keymap_layer_name(zmk_keymap_layer_index_to_id(highest));
    return name_is(name, "Adjust") || name_is(name, "WoW") || name_is(name, "LoL");
}

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

static void send_layer_preview(uint8_t show, uint8_t layer) {
    uint8_t data[3] = {PANDAKB_HID_CMD_LAYER_PREVIEW, show, layer};
    send_raw(data, sizeof(data));
}

static void send_key_matrix(void) {
    uint8_t data[1 + KEY_MATRIX_BYTES] = {0};
    data[0] = PANDAKB_HID_CMD_KEY_MATRIX;
    memcpy(&data[1], pressed, KEY_MATRIX_BYTES);
    send_raw(data, sizeof(data));
}

static void set_pressed_bit(uint32_t position, bool down) {
    if (position >= KEY_MATRIX_BYTES * 8) {
        return;
    }
    uint8_t byte = (uint8_t)(position / 8);
    uint8_t bit = (uint8_t)(position % 8);
    if (down) {
        pressed[byte] |= (uint8_t)BIT(bit);
    } else {
        pressed[byte] &= (uint8_t)~BIT(bit);
    }
}

static void sync_overlay(void) {
    zmk_keymap_layer_index_t highest = zmk_keymap_highest_layer_active();
    bool want = should_show_layer(highest);

    if (want) {
        if (!showing || shown_layer != highest) {
            send_layer_preview(PANDAKB_LAYER_PREVIEW_SHOW, (uint8_t)highest);
            showing = true;
            shown_layer = (uint8_t)highest;
            /* Snapshot currently held keys so the overlay lights the layer key(s). */
            send_key_matrix();
        }
    } else if (showing) {
        send_layer_preview(PANDAKB_LAYER_PREVIEW_HIDE, shown_layer);
        showing = false;
        shown_layer = 0;
    }
}

void pandakb_overlay_preview_begin(void) {
    previewing = true;
    sync_overlay();
}

void pandakb_overlay_preview_end(void) {
    previewing = false;
    sync_overlay();
}

static int on_layer_state_changed(const zmk_event_t *eh) {
    ARG_UNUSED(eh);
    sync_overlay();
    return ZMK_EV_EVENT_BUBBLE;
}

static int on_position_state_changed(const zmk_event_t *eh) {
    const struct zmk_position_state_changed *ev = as_zmk_position_state_changed(eh);
    if (ev == NULL) {
        return ZMK_EV_EVENT_BUBBLE;
    }

    set_pressed_bit(ev->position, ev->state);

    if (showing) {
        send_key_matrix();
    }

    return ZMK_EV_EVENT_BUBBLE;
}

ZMK_LISTENER(pandakb_overlay_layer, on_layer_state_changed);
ZMK_SUBSCRIPTION(pandakb_overlay_layer, zmk_layer_state_changed);

ZMK_LISTENER(pandakb_overlay_position, on_position_state_changed);
ZMK_SUBSCRIPTION(pandakb_overlay_position, zmk_position_state_changed);

#else /* !keymap or !RAW_HID */

void pandakb_overlay_preview_begin(void) {}
void pandakb_overlay_preview_end(void) {}

#endif
