/* test_keymap.c — keysym IBus → phím chuẩn hóa (chạy với libibus thật)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include <assert.h>
#include <stdio.h>

#include "keymap.h"

static lc_key classify(guint keyval) {
    uint32_t vk = 0, ch = 0;
    textvn_ibus_map_key(keyval, &vk, &ch);
    return lc_key_classify(vk, ch);
}

int main(void) {
    lc_key a = classify(IBUS_KEY_a);
    assert(a.kind == LC_KEY_CHAR && a.ch == 'a');
    lc_key A = classify(IBUS_KEY_A);
    assert(A.kind == LC_KEY_CHAR && A.ch == 'A');
    assert(classify(IBUS_KEY_space).kind == LC_KEY_CHAR);
    assert(classify(IBUS_KEY_Return).kind == LC_KEY_ENTER);
    assert(classify(IBUS_KEY_KP_Enter).kind == LC_KEY_ENTER);
    assert(classify(IBUS_KEY_BackSpace).kind == LC_KEY_BACKSPACE);
    assert(classify(IBUS_KEY_Escape).kind == LC_KEY_ESCAPE);
    assert(classify(IBUS_KEY_ISO_Left_Tab).kind == LC_KEY_TAB);
    /* Phím không sinh ký tự không bao giờ thành chữ. */
    assert(classify(IBUS_KEY_Left).kind == LC_KEY_OTHER);
    assert(classify(IBUS_KEY_Delete).kind == LC_KEY_OTHER);
    assert(classify(IBUS_KEY_F1).kind == LC_KEY_OTHER);
    assert(classify(IBUS_KEY_Home).kind == LC_KEY_OTHER);
    lc_key kp1 = classify(IBUS_KEY_KP_1);
    assert(kp1.kind == LC_KEY_CHAR && kp1.ch == '1');

    assert(textvn_ibus_modifier_kind(IBUS_KEY_Control_R) == LC_MOD_KEY_CTRL);
    assert(textvn_ibus_modifier_kind(IBUS_KEY_Shift_L) == LC_MOD_KEY_SHIFT);
    assert(textvn_ibus_modifier_kind(IBUS_KEY_ISO_Level3_Shift) == LC_MOD_KEY_OTHER_MODIFIER);
    assert(textvn_ibus_modifier_kind(IBUS_KEY_a) == LC_MOD_NONE);

    assert(textvn_ibus_mods(IBUS_MOD2_MASK) == 0 && "NumLock không phải chord");
    assert(textvn_ibus_mods(IBUS_CONTROL_MASK) & IME_MOD_CTRL);
    printf("test_keymap: OK\n");
    return 0;
}
