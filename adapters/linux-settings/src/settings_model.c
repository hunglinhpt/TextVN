/* settings_model.c — Logic bảng điều khiển TextVN (không phụ thuộc GTK)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#define _POSIX_C_SOURCE 200809L

#include "settings_model.h"

#include <stdio.h>
#include <string.h>

#include "lc_compose.h"
#include "textvn_ffi.h"
#include "textvn_settings.h"

const char *const tv_charset_labels[TV_CHARSET_COUNT + 1] = {
    "Unicode dựng sẵn", "Unicode tổ hợp", "TCVN3 (ABC)", "VNI Windows", NULL};
const char *const tv_charset_values[TV_CHARSET_COUNT] = {
    "unicode_precomposed", "unicode_decomposed", "tcvn3", "vni_windows"};
const char *const tv_method_labels[TV_METHOD_COUNT + 1] = {
    "Telex", "VNI", "VIQR", "Telex đơn giản", NULL};
const char *const tv_method_values[TV_METHOD_COUNT] = {"telex", "vni", "viqr", "simple_telex"};

const char *const tv_help_text =
    "Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (không kèm phím khác), hoặc "
    "Ctrl + Shift + Space, hoặc bấm biểu tượng V/E trên thanh IBus/Fcitx5.\n\n"
    "Telex: aa=â  aw=ă  ee=ê  oo=ô  ow=ơ  uw=ư  dd=đ;  s f r x j = sắc huyền hỏi ngã nặng;  "
    "z = xoá dấu.\n"
    "VNI: 1-5 = sắc huyền hỏi ngã nặng;  6 = mũ (â ê ô);  7 = móc (ơ ư);  8 = trăng (ă);  "
    "9 = đ;  0 = xoá dấu.\n"
    "Quick Telex: cc=ch  gg=gi  kk=kh  nn=ng  qq=qu  pp=ph  tt=th.\n\n"
    "Gõ tắt: bấm [Gõ tắt...], mỗi dòng ghi  gõ tắt = nội dung; khi gõ, nhập chữ tắt rồi "
    "nhấn Tab (hoặc Space) để bung ra.\n\n"
    "Nếu chưa gõ được tiếng Việt: thêm TextVN vào danh sách bộ gõ của IBus (ibus-setup) "
    "hoặc Fcitx5 (fcitx5-configtool), rồi đăng xuất và đăng nhập lại.";

const char *const tv_about_text =
    "TextVN — Bộ gõ tiếng Việt cho Linux (IBus / Fcitx5)\n\n"
    "Phát triển bởi: LinhBH.CoM\n"
    "Giấy phép: GPL-3.0-or-later\n"
    "https://github.com/hunglinhpt/TextVN";

int tv_index_of(const char *const *values, int n, const char *value) {
    if (!value) return 0;
    for (int i = 0; i < n; ++i) {
        if (strcmp(values[i], value) == 0) return i;
    }
    return 0;
}

int tv_paths_resolve(tv_paths *p, const char *custom_config) {
    if (!p) return -1;
    memset(p, 0, sizeof(*p));
    if (custom_config && custom_config[0]) {
        size_t n = strlen(custom_config);
        if (n >= sizeof(p->config)) return -1;
        memcpy(p->config, custom_config, n + 1);
        const char *slash = strrchr(custom_config, '/');
        size_t dir_len = slash ? (size_t)(slash - custom_config) : 0;
        int w = slash ? snprintf(p->state, sizeof(p->state), "%.*s/state.json", (int)dir_len,
                                 custom_config)
                      : snprintf(p->state, sizeof(p->state), "state.json");
        return (w > 0 && (size_t)w < sizeof(p->state)) ? 0 : -1;
    }
    if (lc_config_resolve_path(p->config, sizeof(p->config)) != 0) return -1;
    return lc_state_resolve_path(p->state, sizeof(p->state));
}

static int get_bool(const ime_settings *s, const char *key, int fallback) {
    int v = ime_settings_get_bool(s, key);
    return v < 0 ? fallback : v;
}

static int get_index(const ime_settings *s, const char *key, const char *const *values, int n) {
    char *v = ime_settings_get_str(s, key);
    int idx = tv_index_of(values, n, v);
    ime_settings_string_free(v);
    return idx;
}

void tv_snapshot_load(tv_snapshot *out, const tv_paths *p) {
    memset(out, 0, sizeof(*out));
    ime_settings *s = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    if (s) {
        out->charset = get_index(s, "output_charset", tv_charset_values, TV_CHARSET_COUNT);
        out->method = get_index(s, "method", tv_method_values, TV_METHOD_COUNT);
        char *style = ime_settings_get_str(s, "diacritic_style");
        out->new_diacritic = !style || strcmp(style, "old") != 0;
        ime_settings_string_free(style);
        out->auto_restore_english = get_bool(s, "auto_restore_english", 1);
        out->free_marking = get_bool(s, "free_marking", 1);
        out->auto_capitalize = get_bool(s, "auto_capitalize", 1);
        out->quick_telex = get_bool(s, "quick_telex", 0);
        out->allow_macro_when_vi_off = get_bool(s, "allow_macro_when_vi_off", 0);
        out->config_was_corrupt = ime_settings_was_corrupt(s);
        ime_settings_free(s);
    }
    out->vi_enabled = lc_state_read_enabled(p->state, 1);
}

typedef int (*tv_edit_fn)(ime_settings *s, const void *arg);

/* Đọc bản MỚI NHẤT trên đĩa → sửa → ghi nguyên tử. */
static int patch_file(const char *path, int kind, tv_edit_fn edit, const void *arg) {
    ime_settings *s = ime_settings_load(path, kind);
    if (!s) return -1;
    int rc = edit(s, arg);
    if (rc == IME_OK) rc = ime_settings_save(s, path);
    ime_settings_free(s);
    return rc == IME_OK ? 0 : -1;
}

typedef struct kv {
    const char *key;
    const char *str;
    int value;
} kv;

static int edit_bool(ime_settings *s, const void *arg) {
    const kv *a = (const kv *)arg;
    return ime_settings_set_bool(s, a->key, a->value);
}

static int edit_str(ime_settings *s, const void *arg) {
    const kv *a = (const kv *)arg;
    return ime_settings_set_str(s, a->key, a->str);
}

static int edit_reset(ime_settings *s, const void *arg) {
    (void)arg;
    ime_settings_reset_defaults(s);
    return IME_OK;
}

int tv_config_set_bool(const tv_paths *p, const char *key, int value) {
    kv a = {key, NULL, value ? 1 : 0};
    return patch_file(p->config, IME_SETTINGS_CONFIG, edit_bool, &a);
}

int tv_config_set_str(const tv_paths *p, const char *key, const char *value) {
    kv a = {key, value, 0};
    return patch_file(p->config, IME_SETTINGS_CONFIG, edit_str, &a);
}

int tv_state_set_enabled(const tv_paths *p, int enabled) {
    return lc_state_write_enabled(p->state, enabled ? 1 : 0);
}

int tv_config_reset_defaults(const tv_paths *p) {
    return patch_file(p->config, IME_SETTINGS_CONFIG, edit_reset, NULL);
}

char *tv_macros_load(const tv_paths *p, int *trigger_space) {
    ime_settings *s = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    if (!s) return NULL;
    char *text = ime_settings_macros_text(s);
    if (trigger_space) {
        char *t = ime_settings_get_str(s, "macro_trigger");
        *trigger_space = t && strcmp(t, "space") == 0;
        ime_settings_string_free(t);
    }
    ime_settings_free(s);
    return text;
}

/* Tu dien EN (0.2.13): moi dong mot tu; cung quy tac chuan hoa 3 nen tang. */
char *tv_english_words_load(const tv_paths *p) {
    ime_settings *s = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    if (!s) return NULL;
    char *text = ime_settings_english_words_text(s);
    ime_settings_free(s);
    return text;
}

int tv_english_words_save(const tv_paths *p, const char *text) {
    ime_settings *s = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    if (!s) return -1;
    int rc = ime_settings_set_english_words_text(s, text ? text : "");
    if (rc == IME_OK) rc = ime_settings_save(s, p->config);
    ime_settings_free(s);
    return rc == IME_OK ? 0 : -1;
}

int tv_macros_save(const tv_paths *p, const char *text, int trigger_space, uint32_t *bad_line,
                   const char **message) {
    ime_settings *s = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    if (!s) return -1;
    uint32_t line = 0;
    int rc = ime_settings_set_macros_text(s, text ? text : "", &line);
    if (rc > 0) {
        if (bad_line) *bad_line = line;
        if (message) *message = ime_settings_macro_error_message(rc);
        ime_settings_free(s);
        return rc;
    }
    if (rc == IME_OK) rc = ime_settings_set_str(s, "macro_trigger", trigger_space ? "space" : "tab");
    if (rc == IME_OK) rc = ime_settings_save(s, p->config);
    ime_settings_free(s);
    return rc == IME_OK ? 0 : -1;
}
