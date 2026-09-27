/* test_field_detect.c — Unit tests for field role detection rules (R1–R10)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "linux_common.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

static void test_password_field(void) {
    lc_field_ctx ctx;

    /* Password by role */
    assert(lc_classify_field("org.gnome.TextEditor", "password_text", 0, "pass", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_SECURE);
    assert(ctx.secure == 1);
    assert(ctx.strategy_hint == IME_STRATEGY_PASSTHROUGH);

    /* Password by state flag */
    assert(lc_classify_field("org.mozilla.firefox", "entry", 0x1, "login", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_SECURE);
    assert(ctx.secure == 1);
    assert(ctx.strategy_hint == IME_STRATEGY_PASSTHROUGH);
}

static void test_terminal_field(void) {
    lc_field_ctx ctx;

    /* Terminal by role */
    assert(lc_classify_field("custom-app", "terminal", 0, "bash", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_TERMINAL);
    assert(ctx.secure == 0);
    assert(ctx.strategy_hint == IME_STRATEGY_FORWARD_AS_COMMIT);

    /* Terminal by app_id */
    assert(lc_classify_field("org.gnome.Terminal", "text", 0, "bash", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_TERMINAL);
    assert(ctx.strategy_hint == IME_STRATEGY_FORWARD_AS_COMMIT);

    assert(lc_classify_field("org.kde.konsole", "text", 0, "zsh", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_TERMINAL);
    assert(ctx.strategy_hint == IME_STRATEGY_FORWARD_AS_COMMIT);
}

static void test_address_bar_field(void) {
    lc_field_ctx ctx;

    /* Address bar by URL in name */
    assert(lc_classify_field("org.mozilla.firefox", "entry", 0, "Search or enter address", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_ADDRESS_BAR);
    assert(ctx.strategy_hint == IME_STRATEGY_SELECTION_REPLACE);

    assert(lc_classify_field("google-chrome", "entry", 0, "Address and search bar", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_ADDRESS_BAR);
    assert(ctx.strategy_hint == IME_STRATEGY_SELECTION_REPLACE);

    /* Vietnamese label */
    assert(lc_classify_field("org.gnome.Nautilus", "entry", 0, "Thanh địa chỉ", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_ADDRESS_BAR);
}

static void test_search_field(void) {
    lc_field_ctx ctx;

    assert(lc_classify_field("org.gnome.Settings", "entry", 0, "Search settings", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_SEARCH);
    assert(ctx.strategy_hint == IME_STRATEGY_SELECTION_REPLACE);

    assert(lc_classify_field("org.gnome.Nautilus", "entry", 0, "Tìm kiếm tệp", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_SEARCH);
}

static void test_chat_field(void) {
    lc_field_ctx ctx;

    /* Slack chat input */
    assert(lc_classify_field("slack.desktop", "editbox", 0, "Send a message to general", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_BODY);
    assert(ctx.strategy_hint == IME_STRATEGY_BACKSPACE_TYPE);

    /* Discord chat input */
    assert(lc_classify_field("discord.desktop", "editbox", 0, "Message #dev", NULL, &ctx) == 0);
    assert(ctx.strategy_hint == IME_STRATEGY_BACKSPACE_TYPE);

    /* Telegram chat */
    assert(lc_classify_field("org.telegram.desktop", "editbox", 0, "Write a message...", NULL, &ctx) == 0);
    assert(ctx.strategy_hint == IME_STRATEGY_BACKSPACE_TYPE);

    /* Zalo chat */
    assert(lc_classify_field("zalo.desktop", "editbox", 0, "Soạn tin nhắn...", NULL, &ctx) == 0);
    assert(ctx.strategy_hint == IME_STRATEGY_BACKSPACE_TYPE);
}

static void test_candidate_field(void) {
    lc_field_ctx ctx;

    assert(lc_classify_field("libreoffice-calc.desktop", "table", 0, "Cell A1", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_CANDIDATE);
    assert(ctx.strategy_hint == IME_STRATEGY_SELECTION_REPLACE);
}

static void test_combo_field(void) {
    lc_field_ctx ctx;

    assert(lc_classify_field("org.gnome.Settings", "combo_box", 0, "Language", NULL, &ctx) == 0);
    assert(ctx.field_role == IME_FIELD_COMBO);
    assert(ctx.strategy_hint == IME_STRATEGY_SELECTION_REPLACE);
}

static void test_utf_conversion(void) {
    /* Test UTF-32 <-> UTF-8 with Vietnamese characters "Tiếng Việt" */
    const uint32_t src[] = { 'T', 'i', 0x1EBF, 'n', 'g', ' ', 'V', 'i', 0x1EC7, 't' };
    size_t src_len = sizeof(src) / sizeof(src[0]);
    char utf8[64];

    size_t written = lc_utf32_to_utf8(src, src_len, utf8, sizeof(utf8));
    assert(written > 0);
    assert(strcmp(utf8, "Tiếng Việt") == 0);

    uint32_t decoded[32];
    size_t dec_count = lc_utf8_to_utf32(utf8, strlen(utf8), decoded, 32);
    assert(dec_count == src_len);
    for (size_t i = 0; i < src_len; ++i) {
        assert(decoded[i] == src[i]);
    }
}

int main(void) {
    test_password_field();
    test_terminal_field();
    test_address_bar_field();
    test_search_field();
    test_chat_field();
    test_candidate_field();
    test_combo_field();
    test_utf_conversion();

    printf("All linux-common field detect and UTF tests passed successfully!\n");
    return 0;
}
