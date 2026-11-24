/* SPDX-License-Identifier: GPL-3.0-or-later */
/* emoji_picker.c — Bộ chọn Emoji nhẹ cho Linux IBus/Fcitx5 (P12). */

#include "lc_emoji.h"

#include <stdio.h>
#include <string.h>

static const char *const CATEGORY_ICONS[8] = {
    "\xF0\x9F\x95\x92",             /* 🕒 Gần đây */
    "\xF0\x9F\x98\x80",             /* 😀 Cảm xúc */
    "\xF0\x9F\x91\x8B",             /* 👋 Cử chỉ */
    "\xE2\x9D\xA4\xEF\xB8\x8F",     /* ❤️ Biểu tượng */
    "\xF0\x9F\x90\xB6",             /* 🐶 Thiên nhiên */
    "\xF0\x9F\x8D\x9C",             /* 🍜 Đồ ăn */
    "\xF0\x9F\x92\xBB",             /* 💻 Công việc */
    "\xF0\x9F\x8E\x89"              /* 🎉 Hoạt động */
};

static const char *const CATEGORY_NAMES[8] = {
    "G\xE1\xBA\xA7n \xC4\x91\xC3\xA2y",
    "C\xE1\xBA\xA3m x\xC3\xBAc",
    "C\xE1\xBB\xAD ch\xE1\xBB\x89",
    "Bi\xE1\xBB\x83u t\xC6\xB0\xE1\xBB\xA3ng",
    "Thi\xC3\xAAn nhi\xC3\xAAn",
    "\xC4\x90\xE1\xBB\x93 \xC4\x83n",
    "C\xC3\xB4ng vi\xE1\xBB\x87c",
    "Ho\xE1\xBA\xA1t \xC4\x91\xE1\xBB\x99ng"
};

static size_t append_utf8_char(char *buf, size_t len, size_t cap, uint32_t cp) {
    char tmp[4];
    size_t n = 0;
    if (cp < 0x80u) {
        tmp[0] = (char)cp;
        n = 1;
    } else if (cp < 0x800u) {
        tmp[0] = (char)(0xC0u | (cp >> 6));
        tmp[1] = (char)(0x80u | (cp & 0x3Fu));
        n = 2;
    } else if (cp < 0x10000u) {
        tmp[0] = (char)(0xE0u | (cp >> 12));
        tmp[1] = (char)(0x80u | ((cp >> 6) & 0x3Fu));
        tmp[2] = (char)(0x80u | (cp & 0x3Fu));
        n = 3;
    } else if (cp <= 0x10FFFFu) {
        tmp[0] = (char)(0xF0u | (cp >> 18));
        tmp[1] = (char)(0x80u | ((cp >> 12) & 0x3Fu));
        tmp[2] = (char)(0x80u | ((cp >> 6) & 0x3Fu));
        tmp[3] = (char)(0x80u | (cp & 0x3Fu));
        n = 4;
    } else {
        return len;
    }
    if (len + n + 1 >= cap) {
        return len;
    }
    memcpy(buf + len, tmp, n);
    buf[len + n] = '\0';
    return len + n;
}

static void pop_utf8_char(char *buf, size_t *len) {
    if (*len == 0) return;
    size_t i = *len - 1;
    while (i > 0 && ((unsigned char)buf[i] & 0xC0u) == 0x80u) {
        i--;
    }
    buf[i] = '\0';
    *len = i;
}

static void reload_items(lc_emoji_picker *p, const ime_instance *inst) {
    uint8_t raw[4096];
    size_t out_len = 0;
    p->count = 0;
    p->selected = 0;

    int32_t rc = ime_emoji_search(
        inst,
        p->query_len > 0 ? (const uint8_t *)p->query : NULL,
        p->query_len,
        p->category,
        (uint32_t)LC_EMOJI_MAX_ITEMS,
        raw,
        sizeof(raw),
        &out_len
    );
    if (rc != IME_OK || out_len == 0) {
        return;
    }

    size_t pos = 0;
    while (pos < out_len && p->count < LC_EMOJI_MAX_ITEMS) {
        size_t line_start = pos;
        while (pos < out_len && raw[pos] != '\n') pos++;
        size_t line_end = pos;
        if (pos < out_len && raw[pos] == '\n') pos++;

        size_t t1 = line_start;
        while (t1 < line_end && raw[t1] != '\t') t1++;
        if (t1 >= line_end) continue;
        size_t t2 = t1 + 1;
        while (t2 < line_end && raw[t2] != '\t') t2++;
        if (t2 >= line_end) continue;

        size_t g_len = t1 - line_start;
        size_t s_len = t2 - (t1 + 1);
        if (g_len == 0 || g_len >= LC_EMOJI_GLYPH_MAX ||
            s_len == 0 || s_len >= LC_EMOJI_SHORTCODE_MAX) {
            continue;
        }
        lc_emoji_item *it = &p->items[p->count];
        memcpy(it->glyph, raw + line_start, g_len);
        it->glyph[g_len] = '\0';
        memcpy(it->shortcode, raw + t1 + 1, s_len);
        it->shortcode[s_len] = '\0';
        it->category = (t2 + 1 < line_end && raw[t2 + 1] >= '0' && raw[t2 + 1] <= '9')
            ? (uint8_t)(raw[t2 + 1] - '0')
            : 1u;
        p->count++;
    }
}

void lc_emoji_init(lc_emoji_picker *p) {
    if (!p) return;
    memset(p, 0, sizeof(*p));
}

int lc_is_emoji_hotkey(uint32_t keysym, int ctrl, int alt, int shift, int super_mod) {
    if (!ctrl || !alt || shift || super_mod) {
        return 0;
    }
    return (keysym == 'e' || keysym == 'E') ? 1 : 0;
}

void lc_emoji_open(lc_emoji_picker *p, const ime_instance *inst) {
    if (!p) return;
    p->active = 1;
    p->category = 0;
    p->query[0] = '\0';
    p->query_len = 0;
    reload_items(p, inst);
}

void lc_emoji_close(lc_emoji_picker *p) {
    if (!p) return;
    p->active = 0;
    p->query[0] = '\0';
    p->query_len = 0;
    p->count = 0;
    p->selected = 0;
}

int lc_emoji_pick_index(lc_emoji_picker *p, ime_instance *inst,
                        size_t idx, char *out_glyph, size_t out_cap) {
    if (!p || !p->active || idx >= p->count || !out_glyph || out_cap == 0) {
        return 0;
    }
    size_t n = strlen(p->items[idx].glyph);
    if (n + 1 > out_cap) {
        return 0;
    }
    memcpy(out_glyph, p->items[idx].glyph, n + 1);
    if (inst) {
        (void)ime_emoji_record_use(inst, (const uint8_t *)out_glyph, n);
    }
    lc_emoji_close(p);
    return 1;
}

int lc_emoji_handle_key(lc_emoji_picker *p, ime_instance *inst,
                        uint32_t vk, uint32_t ch, int shift,
                        char *out_glyph, size_t out_cap) {
    if (!p || !p->active) {
        return LC_EMOJI_CLOSED;
    }

    /* Esc */
    if (vk == 0x1Bu) {
        lc_emoji_close(p);
        return LC_EMOJI_CLOSED;
    }

    /* Enter */
    if (vk == 0x0Du) {
        if (p->count > 0 && lc_emoji_pick_index(p, inst, p->selected, out_glyph, out_cap)) {
            return LC_EMOJI_COMMIT;
        }
        lc_emoji_close(p);
        return LC_EMOJI_CLOSED;
    }

    /* Tab / Shift+Tab: đổi nhóm 0..7 */
    if (vk == 0x09u) {
        p->category = shift ? (uint8_t)((p->category + 7u) % 8u) : (uint8_t)((p->category + 1u) % 8u);
        p->query[0] = '\0';
        p->query_len = 0;
        reload_items(p, inst);
        return LC_EMOJI_UPDATE;
    }

    /* Backspace */
    if (vk == 0x08u) {
        if (p->query_len > 0) {
            pop_utf8_char(p->query, &p->query_len);
            reload_items(p, inst);
        }
        return LC_EMOJI_UPDATE;
    }

    /* Mũi tên di chuyển chọn */
    if (vk == 0x25u || vk == 0x26u) {
        if (p->selected > 0) p->selected--;
        return LC_EMOJI_UPDATE;
    }
    if (vk == 0x27u || vk == 0x28u) {
        if (p->selected + 1 < p->count) p->selected++;
        return LC_EMOJI_UPDATE;
    }

    /* Số 1..9 chọn nhanh trên trang hiện tại khi chưa nhập chữ tìm kiếm */
    if (p->query_len == 0 && ch >= '1' && ch <= '9') {
        size_t page_start = (p->selected / LC_EMOJI_PAGE_SIZE) * LC_EMOJI_PAGE_SIZE;
        size_t idx = page_start + (size_t)(ch - '1');
        if (idx < p->count && lc_emoji_pick_index(p, inst, idx, out_glyph, out_cap)) {
            return LC_EMOJI_COMMIT;
        }
        return LC_EMOJI_UPDATE;
    }

    /* Ký tự in được: nối vào query tìm kiếm VI/EN không dấu */
    if (ch >= 0x20u && ch != 0x7Fu) {
        size_t next_len = append_utf8_char(p->query, p->query_len, sizeof(p->query), ch);
        if (next_len != p->query_len) {
            p->query_len = next_len;
            reload_items(p, inst);
        }
    }
    return LC_EMOJI_UPDATE;
}

void lc_emoji_format_aux(const lc_emoji_picker *p, char *out, size_t out_cap) {
    if (!out || out_cap == 0) return;
    if (!p || !p->active) {
        out[0] = '\0';
        return;
    }
    uint8_t cat = p->category < 8u ? p->category : 0u;
    if (p->query_len > 0) {
        (void)snprintf(out, out_cap,
                       "\xF0\x9F\x94\x8D %s  (Enter ch\xC3\xA8n \xC2\xB7 Esc \xC4\x91\xC3\xB3ng)",
                       p->query);
    } else {
        (void)snprintf(out, out_cap,
                       "%s %s  |  G\xC3\xB5 t\xC3\xACm \xC2\xB7 Tab nh\xC3\xB3m \xC2\xB7 1-9/Enter ch\xC3\xA8n",
                       CATEGORY_ICONS[cat], CATEGORY_NAMES[cat]);
    }
}
