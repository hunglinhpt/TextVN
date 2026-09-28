/* compose.c — Mô hình preedit dùng chung (port của windows-tsf/src/compose.rs)
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#define _POSIX_C_SOURCE 200809L

#include "lc_compose.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

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

uint32_t lc_key_engine_ch(lc_key key) {
    return key.kind == LC_KEY_CHAR ? key.ch : 0;
}

void lc_key_to_ime(lc_key key, uint32_t vk, uint32_t mods, ime_key_v1 *out) {
    memset(out, 0, sizeof(*out));
    out->abi_version = IME_ABI_VERSION;
    switch (key.kind) {
    case LC_KEY_BACKSPACE: out->vk = LC_VK_BACK;   break;
    case LC_KEY_TAB:       out->vk = LC_VK_TAB;    break;
    case LC_KEY_ENTER:     out->vk = LC_VK_RETURN; break;
    case LC_KEY_ESCAPE:    out->vk = LC_VK_ESCAPE; break;
    case LC_KEY_CHAR:      out->vk = key.ch == ' ' ? LC_VK_SPACE : vk; break;
    case LC_KEY_OTHER:     out->vk = vk; break;
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
                memcpy(out->text, cur->text, cur->len * sizeof(uint32_t));
                out->text[cur->len] = key.ch;
                out->text_len = cur->len + 1;
                out->has_text = 1;
                out->end = 1;
                out->eaten = 1;
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

int lc_config_resolve_path(char *out, size_t max_len) {
    if (!out || max_len == 0) return -1;
    const char *xdg = getenv("XDG_CONFIG_HOME");
    int n;
    if (xdg && xdg[0] == '/') {
        n = snprintf(out, max_len, "%s/TextVN/config.json", xdg);
    } else {
        const char *home = getenv("HOME");
        if (!home || !home[0]) return -1;
        n = snprintf(out, max_len, "%s/.config/TextVN/config.json", home);
    }
    return (n > 0 && (size_t)n < max_len) ? 0 : -1;
}

#define LC_CONFIG_MAX_BYTES (256 * 1024)

int lc_config_sync(ime_instance *inst, lc_config_state *st, const char *path) {
    if (!inst || !st) return 0;
    char buf_path[1024];
    if (!path) {
        if (lc_config_resolve_path(buf_path, sizeof(buf_path)) != 0) return 0;
        path = buf_path;
    }
    struct stat sb;
    if (stat(path, &sb) != 0 || !S_ISREG(sb.st_mode)) return 0;
    long long mtime_ns = (long long)sb.st_mtim.tv_sec * 1000000000LL + sb.st_mtim.tv_nsec;
    if (st->loaded && st->mtime_ns == mtime_ns && st->size == (long long)sb.st_size) return 0;
    if (sb.st_size <= 0 || sb.st_size > LC_CONFIG_MAX_BYTES) return 0;

    FILE *f = fopen(path, "rb");
    if (!f) return 0;
    char *data = (char *)malloc((size_t)sb.st_size);
    if (!data) {
        fclose(f);
        return 0;
    }
    size_t got = fread(data, 1, (size_t)sb.st_size, f);
    fclose(f);

    /* Config sai schema → engine giữ config cũ (P0-3 §1.3); vẫn ghi nhận mtime để
     * không đọc lại file hỏng ở mỗi phím. */
    (void)ime_reload_config(inst, (const uint8_t *)data, got);
    free(data);
    st->mtime_ns = mtime_ns;
    st->size = (long long)sb.st_size;
    st->loaded = 1;
    return 1;
}
