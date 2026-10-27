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

/* Trợ lý ngữ pháp (P4-0 §7): giá trị của `assist.mode`, cùng thứ tự enum của engine
 * (off / highlight / autocorrect). Nhãn hiển thị thống nhất với Windows. */
const char *const tv_assist_mode_labels[TV_ASSIST_MODE_COUNT + 1] = {
    "Tắt", "Chỉ gợi ý", "Tự sửa lỗi", NULL};
const char *const tv_assist_mode_values[TV_ASSIST_MODE_COUNT] = {
    "off", "highlight", "autocorrect"};
const char *const tv_assist_auto_fix_labels[TV_ASSIST_AUTO_FIX_COUNT + 1] = {
    "Tắt", "An toàn (lỗi khách quan)", NULL};
const char *const tv_assist_auto_fix_values[TV_ASSIST_AUTO_FIX_COUNT] = {"off", "safe"};

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

/* Popup "Thông tin" — CÙNG nội dung trên Windows/Linux/macOS (test Rust
 * `about_text_matches_every_platform`); dòng "TextVN <phiên bản>" ghép ở on_about. */
const char *const tv_about_text =
    "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)\n\n"
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
        /* C9 (audit): khong co '/' (vd --config config.json) truoc day ghi
         * "state.json" theo CWD. Nay dat state.json canh file config: dung "."
         * (thu muc hien hanh) de nhat quan va tranh ghi lech cho. */
        int w;
        if (slash) {
            size_t dir_len = (size_t)(slash - custom_config);
            w = snprintf(p->state, sizeof(p->state), "%.*s/state.json", (int)dir_len,
                         custom_config);
        } else {
            w = snprintf(p->state, sizeof(p->state), "./state.json");
        }
        return (w > 0 && (size_t)w < sizeof(p->state)) ? 0 : -1;
    }
    if (lc_config_resolve_path(p->config, sizeof(p->config)) != 0) return -1;
    return lc_state_resolve_path(p->state, sizeof(p->state));
}

static int get_bool(const ime_settings *s, const char *key, int fallback) {
    int v = ime_settings_get_bool(s, key);
    return v < 0 ? fallback : v;
}

static uint32_t get_u32(const ime_settings *s, const char *key, uint32_t fallback) {
    int64_t v = ime_settings_get_u32(s, key);
    return v < 0 ? fallback : (uint32_t)v;
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
        /* Vong 17 (default cu): thieu khoa = old (dau cu) — khop default config. */
        out->new_diacritic = style && strcmp(style, "new") == 0;
        ime_settings_string_free(style);
        out->auto_restore_english = get_bool(s, "auto_restore_english", 1);
        out->free_marking = get_bool(s, "free_marking", 1);
        out->auto_capitalize = get_bool(s, "auto_capitalize", 1);
        out->quick_telex = get_bool(s, "quick_telex", 0);
        out->allow_macro_when_vi_off = get_bool(s, "allow_macro_when_vi_off", 0);
        /* Đường lồng qua C-API (không parse JSON tay): thiếu key con → mặc định schema. */
        /* Vong 26 (0.2.38): thieu khoa assist.mode = highlight (1) — tro ly
         * bat mac dinh o che do chi goi y; tv_index_of tra 0 (off) khi thieu
         * nen phai doc raw de phan biet. */
        {
            char *raw_mode = ime_settings_get_str(s, "assist.mode");
            if (raw_mode) {
                out->assist_mode = tv_index_of(tv_assist_mode_values,
                                               TV_ASSIST_MODE_COUNT, raw_mode);
                ime_settings_string_free(raw_mode);
            } else {
                out->assist_mode = 1; /* highlight */
            }
        }
        out->assist_auto_fix =
            get_index(s, "assist.auto_fix", tv_assist_auto_fix_values, TV_ASSIST_AUTO_FIX_COUNT);
        out->assist_idle_ms = get_u32(s, "assist.display.idle_ms", TV_ASSIST_DEFAULT_IDLE_MS);
        /* P6: dự đoán văn bản opt-in — thiếu khoá = TẮT (khớp schema). */
        out->suggest_enabled = get_bool(s, "suggest.enabled", 0);
        out->config_was_corrupt = ime_settings_was_corrupt(s);
        ime_settings_free(s);
    } else {
        /* Vòng 32 (audit F-J): chưa có config.json — dialog phải hiện DEFAULT
         * sản phẩm (khớp Config::default của config/src/lib.rs), không phải
         * struct zeroed: trợ lý "Tắt" + 3 tuỳ chọn gõ tắt hết là SAI. */
        out->assist_mode = 1;          /* highlight */
        out->assist_auto_fix = 0;      /* off */
        out->auto_restore_english = 1;
        out->free_marking = 1;
        out->auto_capitalize = 1;
        out->suggest_enabled = 0;      /* dự đoán opt-in — TẮT mặc định */
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

typedef struct kv_u32 {
    const char *key;
    uint32_t value;
} kv_u32;

static int edit_u32(ime_settings *s, const void *arg) {
    const kv_u32 *a = (const kv_u32 *)arg;
    return ime_settings_set_u32(s, a->key, a->value);
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

int tv_config_set_u32(const tv_paths *p, const char *key, uint32_t value) {
    kv_u32 a = {key, value};
    return patch_file(p->config, IME_SETTINGS_CONFIG, edit_u32, &a);
}

/* Trợ lý ngữ pháp: ghi qua đúng đường lồng C-API — `assist.mode` là leaf chuỗi enum,
 * `assist.display.idle_ms` là leaf số; chép sai giá trị trả lỗi, KHÔNG tạo khối lạ. */
int tv_config_set_assist_mode(const tv_paths *p, int mode_index) {
    if (mode_index < 0 || mode_index >= TV_ASSIST_MODE_COUNT) return -1;
    return tv_config_set_str(p, "assist.mode", tv_assist_mode_values[mode_index]);
}

int tv_config_set_assist_auto_fix(const tv_paths *p, int fix_index) {
    if (fix_index < 0 || fix_index >= TV_ASSIST_AUTO_FIX_COUNT) return -1;
    return tv_config_set_str(p, "assist.auto_fix", tv_assist_auto_fix_values[fix_index]);
}

int tv_config_set_assist_idle_ms(const tv_paths *p, uint32_t idle_ms) {
    return tv_config_set_u32(p, "assist.display.idle_ms", idle_ms);
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
