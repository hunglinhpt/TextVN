/* compose.c — Mô hình preedit dùng chung (port của windows-tsf/src/compose.rs)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#define _POSIX_C_SOURCE 200809L

#include "lc_compose.h"
#include "linux_common.h"
#include "textvn_settings.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

/* ---- Phân loại phím ---- */

lc_key lc_key_classify(uint32_t vk, uint32_t ch) {
    lc_key k = { LC_KEY_OTHER, 0 };
    switch (vk) {
    case LC_VK_BACK:   k.kind = LC_KEY_BACKSPACE; return k;
    case LC_VK_TAB:    k.kind = LC_KEY_TAB;       return k;
    case LC_VK_RETURN: k.kind = LC_KEY_ENTER;     return k;
    case LC_VK_ESCAPE: k.kind = LC_KEY_ESCAPE;    return k;
    default: break;
    }
    switch (ch) {
    case '\n': case '\r': k.kind = LC_KEY_ENTER;     return k;
    case '\t':            k.kind = LC_KEY_TAB;       return k;
    case 0x08:            k.kind = LC_KEY_BACKSPACE; return k;
    case 0x1B:            k.kind = LC_KEY_ESCAPE;    return k;
    default: break;
    }
    /* Ký tự in được: không phải C0/DEL/C1, không phải surrogate. */
    if (ch >= 0x20 && ch != 0x7F && !(ch >= 0x80 && ch < 0xA0) &&
        !(ch >= 0xD800 && ch <= 0xDFFF) && ch <= 0x10FFFF) {
        k.kind = LC_KEY_CHAR;
        k.ch = ch;
    }
    return k;
}

lc_key lc_key_classify_with_mods(uint32_t vk, uint32_t ch, uint32_t mods) {
    if ((vk == LC_VK_SPACE || ch == ' ') && ((mods & ~IME_MOD_CAPS) == IME_MOD_CTRL)) {
        return (lc_key){ LC_KEY_CTRL_SPACE, 0 };
    }
    return lc_key_classify(vk, ch);
}

uint32_t lc_key_engine_ch(lc_key key) {
    return key.kind == LC_KEY_CHAR ? key.ch : 0;
}

void lc_key_to_ime(lc_key key, uint32_t vk, uint32_t mods, ime_key_v1 *out) {
    memset(out, 0, sizeof(*out));
    out->abi_version = IME_ABI_VERSION;
    switch (key.kind) {
    case LC_KEY_BACKSPACE:  out->vk = LC_VK_BACK;   break;
    case LC_KEY_TAB:        out->vk = LC_VK_TAB;    break;
    case LC_KEY_ENTER:      out->vk = LC_VK_RETURN; break;
    case LC_KEY_ESCAPE:     out->vk = LC_VK_ESCAPE; break;
    case LC_KEY_CTRL_SPACE: out->vk = LC_VK_SPACE;  break;
    case LC_KEY_CHAR:       out->vk = key.ch == ' ' ? LC_VK_SPACE : vk; break;
    case LC_KEY_OTHER:      out->vk = vk; break;
    }
    out->ch = lc_key_engine_ch(key);
    out->mods = mods;
    out->key_down = 1;
}

/* ---- Kế hoạch ---- */

static int boundary_char(lc_key key, uint32_t *out) {
    switch (key.kind) {
    case LC_KEY_CHAR:  *out = key.ch; return 1;
    case LC_KEY_ENTER: *out = '\n';   return 1;
    case LC_KEY_TAB:   *out = '\t';   return 1;
    default: return 0;
    }
}

static void plan_zero(lc_plan *p) {
    memset(p, 0, sizeof(*p));
}

void lc_plan_pass_through(const lc_comp *cur, lc_plan *out) {
    plan_zero(out);
    out->end = cur->len > 0;
    out->eaten = 0;
}

void lc_plan_key(const lc_comp *cur, lc_key key, const ime_result_v1 *r, lc_plan *out) {
    const int composing = cur->len > 0;
    const int word_end = (r->flags & IME_FLAG_WORD_END) != 0;
    plan_zero(out);

    if (r->action == IME_ACTION_PASS) {
        if (key.kind == LC_KEY_CHAR && !word_end) {
            /* Engine nhận ký tự vào từ nhưng chưa biến đổi → đưa vào preedit. */
            if (cur->len + 1 >= LC_COMP_MAX) {
                /* C5 (audit): cur->len == LC_COMP_MAX thi ghi text[cur->len] la
                 * OOB intra-object (vuot mang text[256]) — commit nguyen chuoi
                 * hien tai va KHONG noi phim moi vao: eaten = 0 nen phim di tiep
                 * toi app, ky tu hien SAU phan vua commit (khong mat chu). */
                memcpy(out->text, cur->text, (size_t)cur->len * sizeof(uint32_t));
                out->text_len = cur->len;
                out->has_text = 1;
                out->end = 1;
                out->eaten = (key.kind == LC_KEY_CHAR) ? 0 : 1;
                out->reset_engine = 1;
                return;
            }
            memcpy(out->text, cur->text, cur->len * sizeof(uint32_t));
            out->text[cur->len] = key.ch;
            out->text_len = cur->len + 1;
            out->has_text = 1;
            out->eaten = 1;
            return;
        }
        if (key.kind == LC_KEY_BACKSPACE && composing) {
            /* Engine bỏ 1 ký tự chưa biến đổi → bỏ khỏi preedit. */
            memcpy(out->text, cur->text, (cur->len - 1) * sizeof(uint32_t));
            out->text_len = cur->len - 1;
            out->has_text = 1;
            out->end = out->text_len == 0;
            out->eaten = 1;
            return;
        }
        if (key.kind == LC_KEY_CTRL_SPACE) {
            /* Ctrl+Space không khớp prediction: không chèn space vào preedit, giữ composition */
            out->end = 0;
            out->eaten = 0;
            return;
        }
        lc_plan_pass_through(cur, out);
        return;
    }

    if (r->action == IME_ACTION_REPLACE || r->action == IME_ACTION_RESTORE ||
        r->action == IME_ACTION_COMMIT) {
        size_t keep = cur->len;
        if (r->action != IME_ACTION_COMMIT) {
            size_t del = r->delete_count;
            if (del <= cur->len) {
                keep = cur->len - del;
            } else {
                keep = 0;
                out->delete_before = (uint16_t)(del - cur->len);
            }
        }
        size_t ins = r->insert_len;
        if (ins > IME_MAX_TEXT) ins = IME_MAX_TEXT;
        if (keep + ins > LC_COMP_MAX) {
            lc_plan_pass_through(cur, out);
            out->reset_engine = 1;
            return;
        }
        memcpy(out->text, cur->text, keep * sizeof(uint32_t));
        memcpy(out->text + keep, r->insert, ins * sizeof(uint32_t));
        out->text_len = keep + ins;
        out->has_text = 1;
        out->eaten = 1;

        uint32_t b = 0;
        if (word_end && r->action != IME_ACTION_REPLACE && boundary_char(key, &b) &&
            out->text_len > 0 && out->text[out->text_len - 1] == b) {
            out->text_len -= 1;
            out->eaten = 0;
        }
        out->end = word_end || out->text_len == 0 || !out->eaten;
        return;
    }

    /* Action lạ (ABI mới hơn adapter): commit và để phím đi thẳng. */
    lc_plan_pass_through(cur, out);
}

void lc_comp_apply(lc_comp *cur, const lc_plan *plan) {
    if (plan->end || plan->reset_engine) {
        cur->len = 0;
        return;
    }
    if (plan->has_text) {
        memcpy(cur->text, plan->text, plan->text_len * sizeof(uint32_t));
        cur->len = plan->text_len;
    }
}

const uint32_t *lc_plan_commit_text(const lc_comp *cur, const lc_plan *plan, size_t *out_len) {
    if (plan->has_text) {
        *out_len = plan->text_len;
        return plan->text;
    }
    *out_len = cur->len;
    return cur->text;
}

/* ---- Ctrl+Shift ---- */

void lc_modifier_toggle_down(lc_modifier_toggle *t, lc_modifier which,
                             int ctrl_down, int shift_down, int alt_or_super) {
    if (which == LC_MOD_KEY_CTRL) {
        t->armed = shift_down && !alt_or_super;
    } else if (which == LC_MOD_KEY_SHIFT) {
        t->armed = ctrl_down && !alt_or_super;
    } else {
        t->armed = 0;
    }
}

int lc_modifier_toggle_up(lc_modifier_toggle *t, lc_modifier which) {
    if ((which == LC_MOD_KEY_CTRL || which == LC_MOD_KEY_SHIFT) && t->armed) {
        t->armed = 0;
        return 1;
    }
    return 0;
}

void lc_modifier_toggle_reset(lc_modifier_toggle *t) {
    t->armed = 0;
}

/* ---- config.json ---- */

int lc_textvn_file_path(char *out, size_t max_len, const char *name) {
    if (!out || max_len == 0 || !name) return -1;
    const char *xdg = getenv("XDG_CONFIG_HOME");
    int n;
    if (xdg && xdg[0] == '/') {
        n = snprintf(out, max_len, "%s/TextVN/%s", xdg, name);
    } else {
        const char *home = getenv("HOME");
        if (!home || !home[0]) return -1;
        n = snprintf(out, max_len, "%s/.config/TextVN/%s", home, name);
    }
    return (n > 0 && (size_t)n < max_len) ? 0 : -1;
}

int lc_config_resolve_path(char *out, size_t max_len) {
    return lc_textvn_file_path(out, max_len, "config.json");
}

int lc_state_resolve_path(char *out, size_t max_len) {
    return lc_textvn_file_path(out, max_len, "state.json");
}

/* stat → (mtime_ns, size); 0 nếu là file thường. */
static int file_stamp(const char *path, long long *mtime_ns, long long *size) {
    struct stat sb;
    if (stat(path, &sb) != 0 || !S_ISREG(sb.st_mode)) return -1;
    *mtime_ns = (long long)sb.st_mtim.tv_sec * 1000000000LL + sb.st_mtim.tv_nsec;
    *size = (long long)sb.st_size;
    return 0;
}

int lc_state_read_enabled(const char *path, int default_enabled) {
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return default_enabled;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_STATE);
    if (!s) return default_enabled;
    int v = ime_settings_get_bool(s, "global_enabled");
    ime_settings_free(s);
    return v < 0 ? default_enabled : v;
}

int lc_state_write_enabled(const char *path, int enabled) {
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return -1;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_STATE);
    if (!s) return -1;
    int rc = ime_settings_set_bool(s, "global_enabled", enabled ? 1 : 0);
    if (rc == IME_OK) rc = ime_settings_save(s, path);
    ime_settings_free(s);
    return rc == IME_OK ? 0 : -1;
}

int lc_state_sync(lc_config_state *st, const char *path, int *enabled) {
    if (!st || !enabled) return 0;
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return 0;
        path = buf;
    }
    long long mtime_ns = 0, size = 0;
    if (file_stamp(path, &mtime_ns, &size) != 0) return 0;
    if (st->loaded && st->mtime_ns == mtime_ns && st->size == size) return 0;
    st->mtime_ns = mtime_ns;
    st->size = size;
    st->loaded = 1;
    *enabled = lc_state_read_enabled(path, *enabled);
    return 1;
}

int lc_state_read_assist_enabled(const char *path, int default_enabled) {
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return default_enabled;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_STATE);
    if (!s) return default_enabled;
    int v = ime_settings_get_bool(s, "assist_enabled");
    ime_settings_free(s);
    return v < 0 ? default_enabled : v;
}

int lc_state_write_assist_enabled(const char *path, int enabled) {
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return -1;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_STATE);
    if (!s) return -1;
    int rc = ime_settings_set_bool(s, "assist_enabled", enabled ? 1 : 0);
    if (rc == IME_OK) rc = ime_settings_save(s, path);
    ime_settings_free(s);
    return rc == IME_OK ? 0 : -1;
}

int lc_state_sync_assist(lc_config_state *st, const char *path, int *assist_enabled) {
    if (!st || !assist_enabled) return 0;
    char buf[1024];
    if (!path) {
        if (lc_state_resolve_path(buf, sizeof(buf)) != 0) return 0;
        path = buf;
    }
    long long mtime_ns = 0, size = 0;
    if (file_stamp(path, &mtime_ns, &size) != 0) return 0;
    if (st->loaded && st->mtime_ns == mtime_ns && st->size == size) return 0;
    st->mtime_ns = mtime_ns;
    st->size = size;
    st->loaded = 1;
    *assist_enabled = lc_state_read_assist_enabled(path, *assist_enabled);
    return 1;
}

int lc_assist_is_active(int assist_enabled, int assist_mode) {
    return (assist_enabled != 0) && (assist_mode != LC_ASSIST_MODE_OFF);
}

int lc_assist_should_auto_fix(int assist_mode, int assist_auto_fix, int is_sentence_trigger) {
    /* R-5 · P4-0 §4.3: Mode 2 = mode "autocorrect" AND auto_fix "safe",
     * chỉ khi trigger KẾT THÚC CÂU (không phải space/,). */
    return (assist_mode == LC_ASSIST_MODE_AUTOCORRECT) &&
           (assist_auto_fix == LC_ASSIST_AUTO_FIX_SAFE) &&
           (is_sentence_trigger != 0);
}

int lc_assist_should_display(int assist_mode, int is_sentence_trigger) {
    /* R-6 · D11: chỉ hiện UI ở điểm hiển thị (kết thúc câu) và khi mode != off
     * (ở Mode 2, lỗi mơ hồ vẫn phải hiện candidate — P4-0 §4.3). */
    return (assist_mode != LC_ASSIST_MODE_OFF) && (is_sentence_trigger != 0);
}

int lc_config_read_assist_mode(const char *path, int default_mode) {
    char buf[1024];
    if (!path) {
        if (lc_config_resolve_path(buf, sizeof(buf)) != 0) return default_mode;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_CONFIG);
    if (!s) return default_mode;
    char *mode = ime_settings_get_str(s, "assist.mode");
    int res = default_mode;
    if (mode) {
        if (strcmp(mode, "highlight") == 0) res = LC_ASSIST_MODE_HIGHLIGHT;
        else if (strcmp(mode, "autocorrect") == 0) res = LC_ASSIST_MODE_AUTOCORRECT;
        else if (strcmp(mode, "off") == 0) res = LC_ASSIST_MODE_OFF;
        ime_settings_string_free(mode);
    }
    ime_settings_free(s);
    return res;
}

int lc_config_read_assist_auto_fix(const char *path, int default_auto_fix) {
    char buf[1024];
    if (!path) {
        if (lc_config_resolve_path(buf, sizeof(buf)) != 0) return default_auto_fix;
        path = buf;
    }
    ime_settings *s = ime_settings_load(path, IME_SETTINGS_CONFIG);
    if (!s) return default_auto_fix;
    char *af = ime_settings_get_str(s, "assist.auto_fix");
    int res = default_auto_fix;
    if (af) {
        if (strcmp(af, "safe") == 0) res = LC_ASSIST_AUTO_FIX_SAFE;
        else if (strcmp(af, "off") == 0) res = LC_ASSIST_AUTO_FIX_OFF;
        ime_settings_string_free(af);
    }
    ime_settings_free(s);
    return res;
}

static size_t lc_strlcpy(char *dst, const char *src, size_t siz) {
    if (!dst || siz == 0) return 0;
    if (!src) {
        dst[0] = '\0';
        return 0;
    }
    size_t slen = strlen(src);
    size_t n = (slen < siz - 1) ? slen : siz - 1;
    memcpy(dst, src, n);
    dst[n] = '\0';
    return slen;
}

int lc_plan_sentence_auto_fix(const ime_instance *inst,
                              const char *sentence_utf8,
                              const ime_assist_result_v1 *result,
                              lc_sentence_fix_plan *out_plan) {
    if (!out_plan || !sentence_utf8 || !result) return 0;
    memset(out_plan, 0, sizeof(*out_plan));
    if (result->abi_version != IME_ABI_VERSION || result->count == 0) return 0;

    ime_assist_plan_v1 ffi_plan;
    memset(&ffi_plan, 0, sizeof(ffi_plan));
    ffi_plan.abi_version = IME_ABI_VERSION;

    int32_t rc = ime_assist_plan_sentence_fix(inst, sentence_utf8, result, &ffi_plan);
    if (rc != 1 || ffi_plan.fixed_count == 0) {
        return 0;
    }

    out_plan->fixed_count = ffi_plan.fixed_count;
    out_plan->delete_glyphs = ffi_plan.delete_glyphs;
    lc_strlcpy(out_plan->replacement, ffi_plan.replacement, sizeof(out_plan->replacement));
    lc_strlcpy(out_plan->original_tail, ffi_plan.original_tail, sizeof(out_plan->original_tail));
    return 1;
}

#define LC_CONFIG_MAX_BYTES (256 * 1024)

int lc_config_sync(ime_instance *inst, lc_config_state *st, const char *path) {
    if (!inst || !st) return 0;
    char buf_path[1024];
    if (!path) {
        if (lc_config_resolve_path(buf_path, sizeof(buf_path)) != 0) return 0;
        path = buf_path;
    }
    long long mtime_ns = 0, size = 0;
    if (file_stamp(path, &mtime_ns, &size) != 0) return 0;
    if (st->loaded && st->mtime_ns == mtime_ns && st->size == size) return 0;
    if (size <= 0 || size > LC_CONFIG_MAX_BYTES) return 0;

    FILE *f = fopen(path, "rb");
    if (!f) return 0;
    char *data = (char *)malloc((size_t)size);
    if (!data) {
        fclose(f);
        return 0;
    }
    size_t got = fread(data, 1, (size_t)size, f);
    fclose(f);

    /* Config sai schema → engine giữ config cũ (P0-3 §1.3); vẫn ghi nhận mtime để
     * không đọc lại file hỏng ở mỗi phím. */
    if (ime_reload_config(inst, (const uint8_t *)data, got) == IME_OK) {
        ime_settings *s = ime_settings_load(path, IME_SETTINGS_CONFIG);
        if (s) {
            st->allow_macro_when_vi_off = ime_settings_get_bool(s, "allow_macro_when_vi_off") == 1;
            char *mode = ime_settings_get_str(s, "assist.mode");
            if (mode) {
                if (strcmp(mode, "highlight") == 0) {
                    st->assist_mode = LC_ASSIST_MODE_HIGHLIGHT;
                } else if (strcmp(mode, "autocorrect") == 0) {
                    st->assist_mode = LC_ASSIST_MODE_AUTOCORRECT;
                } else {
                    st->assist_mode = LC_ASSIST_MODE_OFF;
                }
                ime_settings_string_free(mode);
            } else {
                /* Vong 26/28: config có nhưng thiếu assist.mode -> HIGHLIGHT
                 * (mặc định sản phẩm; trước đây là OFF — fresh-install lệch). */
                st->assist_mode = LC_ASSIST_MODE_HIGHLIGHT;
            }
            char *af = ime_settings_get_str(s, "assist.auto_fix");
            if (af) {
                if (strcmp(af, "safe") == 0) {
                    st->assist_auto_fix = LC_ASSIST_AUTO_FIX_SAFE;
                } else {
                    st->assist_auto_fix = LC_ASSIST_AUTO_FIX_OFF;
                }
                ime_settings_string_free(af);
            } else {
                st->assist_auto_fix = LC_ASSIST_AUTO_FIX_OFF;
            }
            /* D16 (Fassist-20): L6 bị lọc ở adapter khi IME đã tự viết hoa.
             * Khoá GỐC của config.v1.json (P0-3 §1.1), mặc định schema = true
             * (`ime_settings_get_bool` đã trả mặc định khi khoá thiếu). */
            int auto_caps = ime_settings_get_bool(s, "auto_capitalize");
            st->auto_capitalize = (auto_caps < 0)
                                      ? LC_ASSIST_DEFAULT_AUTO_CAPITALIZE
                                      : (auto_caps != 0 ? 1 : 0);
            ime_settings_free(s);
        }
    }
    free(data);
    st->mtime_ns = mtime_ns;
    st->size = size;
    st->loaded = 1;
    return 1;
}

/* ---- Bảng cài đặt ---- */

static int is_executable(const char *path) {
    struct stat sb;
    return stat(path, &sb) == 0 && S_ISREG(sb.st_mode) && access(path, X_OK) == 0;
}

static int try_candidate(char *out, size_t max_len, const char *dir, const char *rel) {
    if (!dir || !dir[0]) return -1;
    int n = snprintf(out, max_len, "%s/%s", dir, rel);
    if (n <= 0 || (size_t)n >= max_len) return -1;
    return is_executable(out) ? 0 : -1;
}

int lc_find_settings_binary(char *out, size_t max_len, const char *self_dir) {
    if (!out || max_len == 0) return -1;
    if (try_candidate(out, max_len, self_dir, "textvn-settings") == 0) return 0;
    if (try_candidate(out, max_len, self_dir, "../../bin/textvn-settings") == 0) return 0;

    const char *path = getenv("PATH");
    if (path) {
        char dir[1024];
        const char *p = path;
        while (*p) {
            const char *colon = strchr(p, ':');
            size_t len = colon ? (size_t)(colon - p) : strlen(p);
            /* Chỉ nhận thư mục tuyệt đối: "" hay "." trong PATH là thư mục hiện hành. */
            if (len > 0 && len < sizeof(dir) && p[0] == '/') {
                memcpy(dir, p, len);
                dir[len] = '\0';
                if (try_candidate(out, max_len, dir, "textvn-settings") == 0) return 0;
            }
            if (!colon) break;
            p = colon + 1;
        }
    }
    const char *home = getenv("HOME");
    if (home && home[0] == '/') {
        if (try_candidate(out, max_len, home, ".local/bin/textvn-settings") == 0) return 0;
    }
    return -1;
}
