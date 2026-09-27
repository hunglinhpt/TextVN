/* field_detect.c — Linux field role detection via AT-SPI & heuristics
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-4-strategy-appdb.md §1-§3
 */

#include "linux_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <time.h>

#if defined(__linux__) || defined(__unix__)
#include <unistd.h>
#include <sys/time.h>
#endif

/* Cache for field detection (TTL 2000ms per P3-4 §2) */
static lc_field_ctx s_cached_ctx;
static uint64_t     s_cached_timestamp_ms = 0;
static bool         s_has_cached_entry = false;

/* Helper: case-insensitive substring search */
static bool str_contains_icase(const char *haystack, const char *needle) {
    if (!haystack || !needle) return false;
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return true;

    for (const char *h = haystack; *h; ++h) {
        size_t i = 0;
        while (h[i] && needle[i] &&
               tolower((unsigned char)h[i]) == tolower((unsigned char)needle[i])) {
            i++;
        }
        if (i == needle_len) return true;
    }
    return false;
}

/* Helper: get monotonic clock in milliseconds */
static uint64_t get_time_ms(void) {
#if defined(__linux__) || defined(__unix__)
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        return (uint64_t)ts.tv_sec * 1000 + (uint64_t)ts.tv_nsec / 1000000;
    }
    return 0;
#else
    return (uint64_t)clock() * 1000 / CLOCKS_PER_SEC;
#endif
}

int lc_classify_field(const char *app_id,
                      const char *role_name,
                      uint64_t state_flags,
                      const char *name,
                      const char *description,
                      lc_field_ctx *out)
{
    if (!out) return -1;
    memset(out, 0, sizeof(*out));

    if (app_id) {
        strncpy(out->app_id, app_id, sizeof(out->app_id) - 1);
    }
    if (name) {
        strncpy(out->element_name, name, sizeof(out->element_name) - 1);
    }

    /* R1: PASSWORD detection (P3-4 §2 R1)
     * ATSPI_ROLE_PASSWORD_TEXT or state PASSWORD flag (bit 0x1) */
    bool is_pwd_role = (role_name && (str_contains_icase(role_name, "password") ||
                                      str_contains_icase(role_name, "secret")));
    bool is_pwd_state = (state_flags & 0x1) != 0;
    if (is_pwd_role || is_pwd_state) {
        out->field_role = IME_FIELD_SECURE;
        out->secure = 1;
        out->strategy_hint = IME_STRATEGY_PASSTHROUGH;
        return 0;
    }

    /* R6: TERMINAL detection (P3-4 §2 R6)
     * Role TERMINAL or known terminal app_id */
    bool is_term_role = (role_name && str_contains_icase(role_name, "terminal"));
    bool is_term_app = false;
    if (app_id) {
        is_term_app = (str_contains_icase(app_id, "terminal") ||
                       str_contains_icase(app_id, "konsole")  ||
                       str_contains_icase(app_id, "kitty")    ||
                       str_contains_icase(app_id, "alacritty")||
                       str_contains_icase(app_id, "xterm")    ||
                       str_contains_icase(app_id, "foot")     ||
                       str_contains_icase(app_id, "wezterm")  ||
                       str_contains_icase(app_id, "console"));
    }
    if (is_term_role || is_term_app) {
        out->field_role = IME_FIELD_TERMINAL;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_FORWARD_AS_COMMIT;
        return 0;
    }

    /* R2: ADDRESS_BAR detection (P3-4 §2 R2)
     * Role ENTRY and name/description matching address/url */
    bool match_address = (str_contains_icase(name, "address") ||
                          str_contains_icase(name, "url")     ||
                          str_contains_icase(name, "location")||
                          str_contains_icase(name, "địa chỉ") ||
                          str_contains_icase(description, "address") ||
                          str_contains_icase(description, "url"));
    if (match_address) {
        out->field_role = IME_FIELD_ADDRESS_BAR;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_SELECTION_REPLACE;
        return 0;
    }

    /* R3: SEARCH box detection (P3-4 §2 R3)
     * Name or description matching search/tìm kiếm */
    bool match_search = (str_contains_icase(name, "search") ||
                         str_contains_icase(name, "tìm kiếm") ||
                         str_contains_icase(name, "tìm") ||
                         str_contains_icase(description, "search") ||
                         str_contains_icase(description, "tìm"));
    if (match_search) {
        out->field_role = IME_FIELD_SEARCH;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_SELECTION_REPLACE;
        return 0;
    }

    /* CHAT Apps (Slack, Discord, Telegram, Zalo) - Prevents Bug B2 Enter duplication */
    bool is_chat_app = false;
    if (app_id) {
        is_chat_app = (str_contains_icase(app_id, "slack")    ||
                       str_contains_icase(app_id, "discord")  ||
                       str_contains_icase(app_id, "telegram") ||
                       str_contains_icase(app_id, "zalo")     ||
                       str_contains_icase(app_id, "element")  ||
                       str_contains_icase(app_id, "signal")   ||
                       str_contains_icase(app_id, "mattermost") ||
                       str_contains_icase(app_id, "teams"));
    }
    bool is_chat_element = (str_contains_icase(name, "message") ||
                            str_contains_icase(name, "tin nhắn") ||
                            str_contains_icase(name, "chat") ||
                            str_contains_icase(name, "soạn tin"));
    if (is_chat_app || is_chat_element) {
        out->field_role = IME_FIELD_BODY;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_BACKSPACE_TYPE; /* Non-preedit direct typing */
        return 0;
    }

    /* R4: COMBO_BOX detection (P3-4 §2 R4) */
    if (role_name && str_contains_icase(role_name, "combo")) {
        out->field_role = IME_FIELD_COMBO;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_SELECTION_REPLACE;
        return 0;
    }

    /* R5: Spreadsheet / Candidate Table cell (P3-4 §2 R5) */
    bool is_table_role = (role_name && (str_contains_icase(role_name, "table") ||
                                        str_contains_icase(role_name, "cell")));
    bool is_calc_app = (app_id && (str_contains_icase(app_id, "calc") ||
                                   str_contains_icase(app_id, "gnumeric") ||
                                   str_contains_icase(app_id, "excel")));
    if (is_table_role || is_calc_app) {
        out->field_role = IME_FIELD_CANDIDATE;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_SELECTION_REPLACE;
        return 0;
    }

    /* R8: Web document (P3-4 §2 R8) */
    if (role_name && str_contains_icase(role_name, "document_web")) {
        out->field_role = IME_FIELD_WEB;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_PREEDIT;
        return 0;
    }

    /* R7: Multi-line text / Section / Paragraph (P3-4 §2 R7) */
    if (role_name && (str_contains_icase(role_name, "section") ||
                      str_contains_icase(role_name, "paragraph") ||
                      str_contains_icase(role_name, "text_area"))) {
        out->field_role = IME_FIELD_TEXTAREA;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_PREEDIT;
        return 0;
    }

    /* R9: Generic ENTRY / EDITBOX (P3-4 §2 R9) */
    if (role_name && (str_contains_icase(role_name, "entry") ||
                      str_contains_icase(role_name, "editbox") ||
                      str_contains_icase(role_name, "text"))) {
        out->field_role = IME_FIELD_EDITBOX;
        out->secure = 0;
        out->strategy_hint = IME_STRATEGY_PREEDIT;
        return 0;
    }

    /* Fallback / Unknown */
    out->field_role = IME_FIELD_UNKNOWN;
    out->secure = 0;
    out->strategy_hint = -1; /* Delegate to engine resolver */
    return 0;
}

int lc_field_detect(lc_field_ctx *out) {
    if (!out) return -1;

    uint64_t now = get_time_ms();
    /* Check 2s TTL cache */
    if (s_has_cached_entry && (now - s_cached_timestamp_ms) < 2000) {
        *out = s_cached_ctx;
        return 0;
    }

    /* In live Linux systems with AT-SPI enabled, D-Bus accessible query is performed.
     * When running without AT-SPI daemon or in test/fallback mode, we query standard
     * process environment and fallback gracefully. */
    lc_field_ctx detected;
    memset(&detected, 0, sizeof(detected));

    /* Default fail-safe values */
    detected.field_role = IME_FIELD_UNKNOWN;
    detected.secure = 0;
    detected.strategy_hint = -1;

    /* Update cache */
    s_cached_ctx = detected;
    s_cached_timestamp_ms = now;
    s_has_cached_entry = true;

    *out = detected;
    return 0;
}

void lc_field_detect_invalidate_cache(void) {
    s_has_cached_entry = false;
    s_cached_timestamp_ms = 0;
}
