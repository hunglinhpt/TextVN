/* test_keymap.cpp — keysym Fcitx5 → phím chuẩn hóa (chạy với libFcitx5Utils thật)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include <cassert>
#include <cstdio>

#include "engine.h"

using namespace textvn;

static lc_key classify(fcitx::KeySym sym) {
    uint32_t vk = 0, ch = 0;
    mapFcitxKey(fcitx::Key(sym), &vk, &ch);
    return lc_key_classify(vk, ch);
}

int main() {
    lc_key a = classify(FcitxKey_a);
    assert(a.kind == LC_KEY_CHAR && a.ch == 'a');
    assert(classify(FcitxKey_space).kind == LC_KEY_CHAR);
    assert(classify(FcitxKey_Return).kind == LC_KEY_ENTER);
    assert(classify(FcitxKey_KP_Enter).kind == LC_KEY_ENTER);
    assert(classify(FcitxKey_BackSpace).kind == LC_KEY_BACKSPACE);
    assert(classify(FcitxKey_Escape).kind == LC_KEY_ESCAPE);
    assert(classify(FcitxKey_ISO_Left_Tab).kind == LC_KEY_TAB);
    assert(classify(FcitxKey_Left).kind == LC_KEY_OTHER);
    assert(classify(FcitxKey_Delete).kind == LC_KEY_OTHER);
    assert(classify(FcitxKey_F1).kind == LC_KEY_OTHER);
    lc_key kp1 = classify(FcitxKey_KP_1);
    assert(kp1.kind == LC_KEY_CHAR && kp1.ch == '1');

    assert(fcitxModifierKind(FcitxKey_Control_L) == LC_MOD_KEY_CTRL);
    assert(fcitxModifierKind(FcitxKey_Shift_R) == LC_MOD_KEY_SHIFT);
    assert(fcitxModifierKind(FcitxKey_ISO_Level3_Shift) == LC_MOD_KEY_OTHER_MODIFIER);
    assert(fcitxModifierKind(FcitxKey_a) == LC_MOD_NONE);

    assert(fcitxMods(fcitx::KeyState::NumLock) == 0);
    assert(fcitxMods(fcitx::KeyState::Ctrl) & IME_MOD_CTRL);

    const uint32_t duoc[] = {0x111, 0x1B0, 0x1EE3, 'c'};
    assert(utf32ToUtf8(duoc, 4) == "\xc4\x91\xc6\xb0\xe1\xbb\xa3" "c");
    std::printf("test_fcitx5_keymap: OK\n");
    return 0;
}
