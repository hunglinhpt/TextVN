/* keymap.c — Map IBus keyval and state to TextVN ime_key_v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "keymap.h"
#include <string.h>

#ifndef IBUS_KEY_BackSpace
#define IBUS_KEY_BackSpace 0xff08
#define IBUS_KEY_Tab       0xff09
#define IBUS_KEY_KP_Tab    0xff89
#define IBUS_KEY_Return    0xff0d
#define IBUS_KEY_KP_Enter  0xff8d
#define IBUS_KEY_Escape    0xff1b
#define IBUS_KEY_space     0x0020
#define IBUS_KEY_KP_Space  0xff80
#endif

gboolean ibus_keyval_to_ime_key(guint keyval, guint keycode, guint state, ime_key_v1 *out) {
    if (!out) return FALSE;
    (void)keycode;

    memset(out, 0, sizeof(*out));
    out->abi_version = IME_ABI_VERSION;
    out->key_down = (state & IBUS_RELEASE_MASK) ? 0 : 1;

    /* Map Modifiers */
    if (state & IBUS_SHIFT_MASK) {
        out->mods |= IME_MOD_SHIFT;
    }
    if (state & IBUS_CONTROL_MASK) {
        out->mods |= IME_MOD_CTRL;
    }
    if (state & IBUS_MOD1_MASK) {
        out->mods |= IME_MOD_ALT;
    }
    if (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK)) {
        out->mods |= IME_MOD_SUPER;
    }
    if (state & IBUS_LOCK_MASK) {
        out->mods |= IME_MOD_CAPS;
    }

    /* Map Special Keys */
    if (keyval == IBUS_KEY_BackSpace) {
        out->vk = 0x08;
        out->ch = 0x08;
        return TRUE;
    }
    if (keyval == IBUS_KEY_Tab || keyval == IBUS_KEY_KP_Tab) {
        out->vk = 0x09;
        out->ch = 0x09;
        return TRUE;
    }
    if (keyval == IBUS_KEY_Return || keyval == IBUS_KEY_KP_Enter) {
        out->vk = 0x0D;
        out->ch = 0x0D;
        return TRUE;
    }
    if (keyval == IBUS_KEY_Escape) {
        out->vk = 0x1B;
        out->ch = 0;
        return TRUE;
    }
    if (keyval == IBUS_KEY_space || keyval == IBUS_KEY_KP_Space) {
        out->vk = 0x20;
        out->ch = 0x20;
        return TRUE;
    }

    /* Printable ASCII Letters */
    if (keyval >= 'a' && keyval <= 'z') {
        out->vk = (keyval - 'a' + 'A');
        out->ch = keyval;
        return TRUE;
    }
    if (keyval >= 'A' && keyval <= 'Z') {
        out->vk = keyval;
        out->ch = keyval;
        return TRUE;
    }
    if (keyval >= '0' && keyval <= '9') {
        out->vk = keyval;
        out->ch = keyval;
        return TRUE;
    }

    /* Standard printable punctuation / symbols (0x20 to 0x7E) */
    if (keyval >= 0x20 && keyval <= 0x7E) {
        out->vk = keyval;
        out->ch = keyval;
        return TRUE;
    }

    /* Unicode characters directly encoded in keysym (ISO 10646: 0x01000000 + codepoint or direct) */
    if (keyval >= 0x01000100 && keyval <= 0x0110FFFF) {
        uint32_t cp = keyval - 0x01000000;
        out->vk = cp;
        out->ch = cp;
        return TRUE;
    }
    if (keyval >= 0x00A0 && keyval <= 0x00FF) {
        out->vk = keyval;
        out->ch = keyval;
        return TRUE;
    }

    return FALSE;
}
