/* keymap.c — IBus keyval/state → phím chuẩn hóa của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "keymap.h"

void textvn_ibus_map_key(guint keyval, uint32_t *vk, uint32_t *ch) {
    switch (keyval) {
    case IBUS_KEY_BackSpace:
        *vk = LC_VK_BACK;
        *ch = 0;
        return;
    case IBUS_KEY_Tab:
    case IBUS_KEY_KP_Tab:
    case IBUS_KEY_ISO_Left_Tab:
        *vk = LC_VK_TAB;
        *ch = 0;
        return;
    case IBUS_KEY_Return:
    case IBUS_KEY_KP_Enter:
    case IBUS_KEY_ISO_Enter:
        *vk = LC_VK_RETURN;
        *ch = 0;
        return;
    case IBUS_KEY_Escape:
        *vk = LC_VK_ESCAPE;
        *ch = 0;
        return;
    case IBUS_KEY_KP_Space:
        *vk = LC_VK_SPACE;
        *ch = ' ';
        return;
    default:
        break;
    }
    /* ibus_keyval_to_unicode trả 0 cho phím không sinh ký tự (mũi tên, F-key…). */
    *ch = ibus_keyval_to_unicode(keyval);
    if (keyval >= IBUS_KEY_a && keyval <= IBUS_KEY_z) {
        *vk = keyval - IBUS_KEY_a + 'A';
    } else if (keyval < 0x80) {
        *vk = keyval;
    } else {
        /* Không có VK Windows tương ứng: engine chỉ dùng `ch`. */
        *vk = 0;
    }
}

uint32_t textvn_ibus_mods(guint state) {
    uint32_t mods = 0;
    if (state & IBUS_SHIFT_MASK) mods |= IME_MOD_SHIFT;
    if (state & IBUS_CONTROL_MASK) mods |= IME_MOD_CTRL;
    if (state & IBUS_MOD1_MASK) mods |= IME_MOD_ALT;
    if (state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK | IBUS_HYPER_MASK)) mods |= IME_MOD_SUPER;
    if (state & IBUS_META_MASK) mods |= IME_MOD_META;
    if (state & IBUS_LOCK_MASK) mods |= IME_MOD_CAPS;
    return mods;
}

lc_modifier textvn_ibus_modifier_kind(guint keyval) {
    switch (keyval) {
    case IBUS_KEY_Control_L:
    case IBUS_KEY_Control_R:
        return LC_MOD_KEY_CTRL;
    case IBUS_KEY_Shift_L:
    case IBUS_KEY_Shift_R:
        return LC_MOD_KEY_SHIFT;
    case IBUS_KEY_Alt_L:
    case IBUS_KEY_Alt_R:
    case IBUS_KEY_Meta_L:
    case IBUS_KEY_Meta_R:
    case IBUS_KEY_Super_L:
    case IBUS_KEY_Super_R:
    case IBUS_KEY_Hyper_L:
    case IBUS_KEY_Hyper_R:
    case IBUS_KEY_Caps_Lock:
    case IBUS_KEY_Shift_Lock:
    case IBUS_KEY_Num_Lock:
    case IBUS_KEY_ISO_Level3_Shift:
    case IBUS_KEY_ISO_Level5_Shift:
    case IBUS_KEY_Mode_switch:
        return LC_MOD_KEY_OTHER_MODIFIER;
    default:
        return LC_MOD_NONE;
    }
}
