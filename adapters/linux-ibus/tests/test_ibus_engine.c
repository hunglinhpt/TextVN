/* test_ibus_engine.c — Unit tests for TextVN IBus Engine implementation
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#define TEXTVN_IBUS_MOCK 1
#include "ibus_mock.h"
#include "textvn_ibus_engine.h"
#include "engine.h"
#include "keymap.h"
#include "apply.h"

#include <stdio.h>
#include <assert.h>
#include <string.h>

static void test_keymap_translation(void) {
    ime_key_v1 k;

    /* Test letter 'a' */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_a, 0, 0, &k) == TRUE);
    assert(k.abi_version == IME_ABI_VERSION);
    assert(k.vk == 'A');
    assert(k.ch == 'a');
    assert(k.mods == 0);
    assert(k.key_down == 1);

    /* Test Shift+'a' -> 'A' */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_A, 0, IBUS_SHIFT_MASK, &k) == TRUE);
    assert(k.vk == 'A');
    assert(k.ch == 'A');
    assert((k.mods & IME_MOD_SHIFT) != 0);

    /* Test Return key */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_Return, 0, 0, &k) == TRUE);
    assert(k.vk == 0x0D);
    assert(k.ch == 0x0D);

    /* Test Backspace key */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_BackSpace, 0, 0, &k) == TRUE);
    assert(k.vk == 0x08);
    assert(k.ch == 0x08);

    /* Test Space key */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_space, 0, 0, &k) == TRUE);
    assert(k.vk == 0x20);
    assert(k.ch == 0x20);

    /* Test Key Release mask -> key_down = 0 */
    assert(ibus_keyval_to_ime_key(IBUS_KEY_a, 0, IBUS_RELEASE_MASK, &k) == TRUE);
    assert(k.key_down == 0);
}

static void test_apply_non_preedit_surrounding(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    self->has_surrounding = TRUE;
    self->non_preedit = TRUE;

    ime_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.delete_count = 1;
    res.insert_len = 1;
    res.insert[0] = 0x00E1; /* 'á' */

    apply_ibus_result(self, &res);

    /* Verify surrounding text deleted before cursor */
    assert(e->mock_deleted_offset == -1);
    assert(e->mock_deleted_count == 1);

    /* Verify committed text is UTF-8 'á' (\xC3\xA1) */
    assert(strcmp(e->mock_committed, "\xC3\xA1") == 0);

    /* Verify preedit is hidden (no underline!) */
    assert(e->mock_preedit_visible == FALSE);

    /* Cleanup via g_object_unref (tests finalize) */
    g_object_unref(e);
}

static void test_apply_fallback_preedit_when_no_surrounding(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    self->has_surrounding = FALSE; /* App does not support surrounding text */
    self->non_preedit = TRUE;      /* User prefers non-preedit */

    ime_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.preedit_len = 2;
    res.preedit[0] = 0x00E1; /* 'á' */
    res.preedit[1] = 'c';
    res.insert_len = 2;
    res.insert[0] = 0x00E1;
    res.insert[1] = 'c';

    apply_ibus_result(self, &res);

    /* When surrounding text is unsupported, must fall back to showing preedit with underline instead of raw commit */
    assert(e->mock_preedit_visible == TRUE);
    assert(strcmp(e->mock_preedit, "\xC3\xA1\x63") == 0);
    assert(e->mock_committed[0] == '\0');

    g_object_unref(e);
}

static void test_apply_preedit_mode(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    self->has_surrounding = TRUE;
    self->non_preedit = FALSE; /* Preedit mode */

    ime_result_v1 res;
    memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    res.action = IME_ACTION_REPLACE;
    res.preedit_len = 4;
    res.preedit[0] = 'v';
    res.preedit[1] = 'i';
    res.preedit[2] = 0x1EC7; /* ệ */
    res.preedit[3] = 't';

    apply_ibus_result(self, &res);

    /* Verify preedit text is "việt" (\x76\x69\xE1\xBB\x87\x74) and visible */
    assert(strcmp(e->mock_preedit, "vi\xE1\xBB\x87t") == 0);
    assert(e->mock_preedit_visible == TRUE);

    g_object_unref(e);
}

static void test_bug_b2_enter_handling(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    textvn_ibus_engine_focus_in(e);

    /* Simulate typing Return key in chat app */
    gboolean handled = textvn_ibus_engine_process_key_event(e, IBUS_KEY_Return, 0, 0);

    /* CRITICAL BUG B2: Return key MUST NOT be swallowed!
     * It must return FALSE so IBus forwards Return to Slack/Discord/Telegram to send the message. */
    assert(handled == FALSE);
    assert(e->mock_preedit_visible == FALSE);

    /* Same for keypad Enter */
    handled = textvn_ibus_engine_process_key_event(e, IBUS_KEY_KP_Enter, 0, 0);
    assert(handled == FALSE);

    g_object_unref(e);
}

static void test_bug_b6_shortcuts(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    textvn_ibus_engine_focus_in(e);

    /* Simulate Ctrl+C (copy shortcut) */
    gboolean handled = textvn_ibus_engine_process_key_event(e, IBUS_KEY_a, 0, IBUS_CONTROL_MASK);
    assert(handled == FALSE);

    /* Simulate Alt+Tab */
    handled = textvn_ibus_engine_process_key_event(e, IBUS_KEY_Tab, 0, IBUS_MOD1_MASK);
    assert(handled == FALSE);

    g_object_unref(e);
}

static void test_hotkey_toggle_vi_en(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    textvn_ibus_engine_focus_in(e);

    assert(self->vi_enabled == TRUE);

    /* Press Ctrl+Shift+Space to toggle (ADR-011) */
    gboolean handled = textvn_ibus_engine_process_key_event(
        e, IBUS_KEY_space, 0, IBUS_CONTROL_MASK | IBUS_SHIFT_MASK);

    /* Must consume the toggle key and toggle state to FALSE */
    assert(handled == TRUE);
    assert(self->vi_enabled == FALSE);

    /* Press again to toggle back to TRUE */
    handled = textvn_ibus_engine_process_key_event(
        e, IBUS_KEY_space, 0, IBUS_CONTROL_MASK | IBUS_SHIFT_MASK);
    assert(handled == TRUE);
    assert(self->vi_enabled == TRUE);

    g_object_unref(e);
}

static void test_secure_field_passthrough(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;
    textvn_ibus_engine_focus_in(e);

    /* Set secure mode (password field) */
    self->secure = 1;

    /* Keys must pass through untouched */
    gboolean handled = textvn_ibus_engine_process_key_event(e, IBUS_KEY_a, 0, 0);
    assert(handled == FALSE);

    g_object_unref(e);
}

static void test_focus_and_lifecycle(void) {
    IBusEngine *e = textvn_ibus_engine_new();
    TextVNIbusEngine *self = (TextVNIbusEngine *)e;

    /* Focus in */
    textvn_ibus_engine_focus_in(e);
    assert(self->inst != NULL);

    /* Set some preedit text */
    e->mock_preedit_visible = TRUE;

    /* Focus out: Must commit-before-hide (B2 / B13) */
    textvn_ibus_engine_focus_out(e);
    assert(e->mock_preedit_visible == FALSE);

    /* Finalize / cleanup via g_object_unref verifies memory free without leak */
    g_object_unref(e);
}

int main(void) {
    test_keymap_translation();
    test_apply_non_preedit_surrounding();
    test_apply_fallback_preedit_when_no_surrounding();
    test_apply_preedit_mode();
    test_bug_b2_enter_handling();
    test_bug_b6_shortcuts();
    test_hotkey_toggle_vi_en();
    test_secure_field_passthrough();
    test_focus_and_lifecycle();

    printf("All TextVN IBus engine unit tests passed successfully!\n");
    return 0;
}
