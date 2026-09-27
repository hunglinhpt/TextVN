/* keymap.cpp — Map Fcitx5 Key to TextVN ime_key_v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "keymap.h"
#include <fcitx-utils/keysym.h>
#include <cstring>

namespace textvn {

bool map_fcitx_key_to_ime(const fcitx::Key &key, ime_key_v1 &out) {
    std::memset(&out, 0, sizeof(out));
    out.abi_version = IME_ABI_VERSION;
    out.key_down = key.isRelease() ? 0 : 1;

    /* Map Modifiers */
    if (key.hasModifier(fcitx::KeyModifier::Shift)) {
        out.mods |= IME_MOD_SHIFT;
    }
    if (key.hasModifier(fcitx::KeyModifier::Ctrl)) {
        out.mods |= IME_MOD_CTRL;
    }
    if (key.hasModifier(fcitx::KeyModifier::Alt)) {
        out.mods |= IME_MOD_ALT;
    }
    if (key.hasModifier(fcitx::KeyModifier::Super)) {
        out.mods |= IME_MOD_SUPER;
    }
    if (key.states().test(fcitx::KeyState::CapsLock)) {
        out.mods |= IME_MOD_CAPS;
    }

    auto sym = key.sym();

    /* Map special keys */
    if (sym == FcitxKey_BackSpace) {
        out.vk = 0x08;
        out.ch = 0x08;
        return true;
    }
    if (sym == FcitxKey_Tab || sym == FcitxKey_KP_Tab) {
        out.vk = 0x09;
        out.ch = 0x09;
        return true;
    }
    if (sym == FcitxKey_Return || sym == FcitxKey_KP_Enter) {
        out.vk = 0x0D;
        out.ch = 0x0D;
        return true;
    }
    if (sym == FcitxKey_Escape) {
        out.vk = 0x1B;
        out.ch = 0;
        return true;
    }
    if (sym == FcitxKey_space || sym == FcitxKey_KP_Space) {
        out.vk = 0x20;
        out.ch = 0x20;
        return true;
    }

    /* Convert keysym to Unicode character */
    uint32_t unicode = fcitx::Key::keySymToUnicode(sym);
    out.ch = unicode;

    /* Normalize virtual key */
    if (sym >= FcitxKey_a && sym <= FcitxKey_z) {
        out.vk = (sym - FcitxKey_a) + 'A';
    } else if (sym >= FcitxKey_A && sym <= FcitxKey_Z) {
        out.vk = sym;
    } else if (sym >= FcitxKey_0 && sym <= FcitxKey_9) {
        out.vk = sym;
    } else {
        out.vk = static_cast<uint32_t>(sym);
    }

    /* If no unicode was generated and not a special key, ignore */
    if (out.ch == 0 && out.vk == 0) {
        return false;
    }

    return true;
}

} // namespace textvn
