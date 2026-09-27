/* test_fcitx5_addon.cpp — Comprehensive tests for Fcitx5 addon logic
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "fcitx_mock.h"
#include "keymap.h"
#include "apply.h"
#include "engine.h"
#include "linux_common.h"

#include <cassert>
#include <iostream>
#include <cstring>

using namespace textvn;

static void test_key_mapping() {
    /* Test normal letter 'a' */
    fcitx::Key key_a(FcitxKey_a);
    ime_key_v1 k;
    assert(map_fcitx_key_to_ime(key_a, k));
    assert(k.abi_version == IME_ABI_VERSION);
    assert(k.vk == 'A');
    assert(k.ch == 'a');
    assert(k.mods == 0);
    assert(k.key_down == 1);

    /* Test Shift+'a' -> 'A' */
    fcitx::Key key_shift_a(FcitxKey_A, static_cast<uint32_t>(fcitx::KeyModifier::Shift));
    assert(map_fcitx_key_to_ime(key_shift_a, k));
    assert(k.vk == 'A');
    assert(k.ch == 'A');
    assert((k.mods & IME_MOD_SHIFT) != 0);

    /* Test Return key */
    fcitx::Key key_enter(FcitxKey_Return);
    assert(map_fcitx_key_to_ime(key_enter, k));
    assert(k.vk == 0x0D);
    assert(k.ch == 0x0D);

    /* Test Backspace key */
    fcitx::Key key_bs(FcitxKey_BackSpace);
    assert(map_fcitx_key_to_ime(key_bs, k));
    assert(k.vk == 0x08);
    assert(k.ch == 0x08);

    /* Test Space key */
    fcitx::Key key_sp(FcitxKey_space);
    assert(map_fcitx_key_to_ime(key_sp, k));
    assert(k.vk == 0x20);
    assert(k.ch == 0x20);
}

static void test_apply_non_preedit_with_surrounding() {
    fcitx::InputContext ic("test-editor");
    ic.capabilityFlags().set(fcitx::CapabilityFlag::SurroundingText);

    ime_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.delete_count = 1;
    res.insert_len = 1;
    res.insert[0] = 0x00E1; /* 'á' */

    /* Apply in non-preedit mode (Gõ không gạch chân — fcitx5-lotus style) */
    apply_result(&ic, res, true);

    /* Surrounding text delete called with offset=-1, size=1 */
    assert(ic.deletedOffsets().size() == 1);
    assert(ic.deletedOffsets()[0] == -1);
    assert(ic.deletedSizes()[0] == 1);

    /* String committed directly */
    assert(ic.lastCommit() == "\xC3\xA1" /* á */);

    /* Preedit UI remains clean (no underline!) */
    assert(ic.inputPanel().preedit().empty());
    assert(!ic.inputPanel().hasUnderline());
}

static void test_apply_non_preedit_fallback_backspace() {
    fcitx::InputContext ic("legacy-x11-app");
    /* No SurroundingText capability */

    ime_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.delete_count = 2;
    res.insert_len = 1;
    res.insert[0] = 0x01A1; /* 'ơ' */

    apply_result(&ic, res, true);

    /* Forwarded 2 BackSpace keys */
    assert(ic.forwardedKeys().size() == 2);
    assert(ic.forwardedKeys()[0] == FcitxKey_BackSpace);
    assert(ic.forwardedKeys()[1] == FcitxKey_BackSpace);

    /* Committed replacement */
    assert(ic.lastCommit() == "\xC6\xA1" /* ơ */);
    assert(ic.inputPanel().preedit().empty());
}

static void test_apply_preedit_with_underline() {
    fcitx::InputContext ic("traditional-editor");

    ime_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.preedit_len = 4;
    res.preedit[0] = 'v';
    res.preedit[1] = 'i';
    res.preedit[2] = 0x1EC7; /* ệ */
    res.preedit[3] = 't';

    /* Apply in classic preedit mode */
    apply_result(&ic, res, false);

    /* Preedit text set with underline attribute */
    assert(ic.inputPanel().preedit() == "vi\xE1\xBB\x87t" /* việt */);
    assert(ic.inputPanel().hasUnderline());
}

static void test_bug_b2_enter_handling() {
    fcitx::Instance inst;
    TextVNEngine engine(&inst);
    fcitx::InputContext ic("slack.desktop");
    fcitx::InputMethodEntry entry;

    /* Simulate Return key */
    fcitx::Key key_enter(FcitxKey_Return);
    fcitx::KeyEvent key_event(&ic, key_enter);

    engine.keyEvent(entry, key_event);

    /* Crucial for Bug B2: The Enter key MUST NOT be filtered/swallowed!
     * It must pass directly to the chat app to submit the message. */
    assert(!key_event.isFiltered());
}

static void test_bug_b6_shortcuts() {
    fcitx::Instance inst;
    TextVNEngine engine(&inst);
    fcitx::InputContext ic("firefox.desktop");
    fcitx::InputMethodEntry entry;

    /* Simulate Ctrl+C (copy shortcut) */
    fcitx::Key key_ctrl_c(FcitxKey_a, static_cast<uint32_t>(fcitx::KeyModifier::Ctrl));
    fcitx::KeyEvent key_event(&ic, key_ctrl_c);

    engine.keyEvent(entry, key_event);

    /* Shortcut must pass through without interference */
    assert(!key_event.isFiltered());
}

static void test_hotkey_toggle_vi_en() {
    fcitx::Instance inst;
    TextVNEngine engine(&inst);
    fcitx::InputContext ic("gedit.desktop");
    fcitx::InputMethodEntry entry;

    assert(engine.isViEnabled(&ic));

    /* Press Ctrl+Shift+Space to toggle (ADR-011) */
    fcitx::Key key_toggle(FcitxKey_space,
                          static_cast<uint32_t>(fcitx::KeyModifier::Ctrl) |
                          static_cast<uint32_t>(fcitx::KeyModifier::Shift));
    fcitx::KeyEvent key_event(&ic, key_toggle);

    engine.keyEvent(entry, key_event);

    /* Must consume the toggle key and flip VI state to false */
    assert(key_event.isFiltered());
    assert(!engine.isViEnabled(&ic));
}

static void test_lifecycle_context_destroy() {
    fcitx::Instance inst;
    TextVNEngine engine(&inst);
    fcitx::InputMethodEntry entry;

    assert(engine.contextCount() == 0);

    {
        fcitx::InputContext temp_ic("ephemeral-app");
        /* Trigger context creation */
        fcitx::InputContextEvent event(&temp_ic);
        engine.activate(entry, event);
        assert(engine.contextCount() == 1);

        /* Type a key */
        fcitx::Key key_a(FcitxKey_a);
        fcitx::KeyEvent key_event(&temp_ic, key_a);
        engine.keyEvent(entry, key_event);
        assert(engine.contextCount() == 1);
        /* temp_ic goes out of scope and is destroyed here */
    }

    /* Verify context was freed and erased from map without memory leaks (LNX-020) */
    assert(engine.contextCount() == 0);
}

int main() {
    test_key_mapping();
    test_apply_non_preedit_with_surrounding();
    test_apply_non_preedit_fallback_backspace();
    test_apply_preedit_with_underline();
    test_bug_b2_enter_handling();
    test_bug_b6_shortcuts();
    test_hotkey_toggle_vi_en();
    test_lifecycle_context_destroy();

    std::cout << "All Fcitx5 addon unit tests passed successfully!" << std::endl;
    return 0;
}
