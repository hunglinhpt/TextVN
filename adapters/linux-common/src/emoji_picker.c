/* SPDX-License-Identifier: GPL-3.0-or-later */
/* emoji_picker.c — Bộ chọn Emoji cho Linux IBus/Fcitx5 (P12).
 *
 * SF-7 (P0-6 §5, G22): máy trạng thái (ô tìm, nhóm, chọn, trang, chọn nhanh 1..9, MRU + Dream RSI)
 * là bản DÙNG CHUNG `textvn_session::emoji::EmojiPicker` qua C ABI `ime_emoji_picker_*`
 * (bố cục LIST: 1 cột, trang 9, 36 mục). File này chỉ còn: ánh xạ phím Linux ⇒ IME_EPK_KEY_*,
 * chép ảnh chụp (`ime_emoji_view_v1`) vào `lc_emoji_picker` cho IBus/Fcitx5 vẽ lookup table, và
 * dòng aux. G8: không log ô tìm hay emoji được chọn. */

#include "lc_emoji.h"

#include <stdio.h>
#include <string.h>

static const char *const CATEGORY_ICONS[8] = {
    "🕒", /* Gần đây */
    "😀", /* Cảm xúc */
    "👋", /* Cử chỉ */
    "❤️", /* Biểu tượng */
    "🐶", /* Thiên nhiên */
    "🍜", /* Đồ ăn */
    "💻", /* Công việc */
    "🎉"  /* Hoạt động */
};

static const char *const CATEGORY_NAMES[8] = {
    "Gần đây",
    "Cảm xúc",
    "Cử chỉ",
    "Biểu tượng",
    "Thiên nhiên",
    "Đồ ăn",
    "Công việc",
    "Hoạt động"
};

/* Chép ảnh chụp của bộ chọn dùng chung vào struct mà IBus/Fcitx5 đọc. Chỉ trang ĐANG HIỆN được
 * điền (`items[first_visible + k]`) — engine chỉ vẽ trang `selected / LC_EMOJI_PAGE_SIZE`, trùng
 * `first_visible` vì bố cục LIST phân trang. */
static void sync_view(lc_emoji_picker *p) {
    ime_emoji_view_v1 v;
    memset(&v, 0, sizeof(v));
    v.abi_version = IME_EMOJI_PICKER_ABI_VERSION;
    if (!p->core || ime_emoji_picker_view(p->core, &v) != IME_OK || !v.open) {
        p->active = 0;
        p->count = 0;
        p->selected = 0;
        p->query[0] = '\0';
        p->query_len = 0;
        return;
    }
    p->active = 1;
    p->category = (uint8_t)(v.category < 8u ? v.category : 0u);
    p->selected = v.selected;
    p->count = v.total < LC_EMOJI_MAX_ITEMS ? v.total : LC_EMOJI_MAX_ITEMS;
    size_t q = v.query_len < LC_EMOJI_QUERY_MAX - 1 ? v.query_len : LC_EMOJI_QUERY_MAX - 1;
    memcpy(p->query, v.query, q);
    p->query[q] = '\0';
    p->query_len = q;
    for (uint32_t k = 0; k < v.count; k++) {
        size_t idx = (size_t)v.first_visible + k;
        if (idx >= LC_EMOJI_MAX_ITEMS) break;
        lc_emoji_item *it = &p->items[idx];
        snprintf(it->glyph, sizeof(it->glyph), "%s", (const char *)v.items[k].glyph);
        snprintf(it->shortcode, sizeof(it->shortcode), "%s", (const char *)v.items[k].shortcode);
        it->category = p->category;
    }
}

/* Giải phóng bộ chọn dùng chung — gọi ở MỌI đường đóng (Esc, chèn, close) nên không rò theo IC. */
static void drop_core(lc_emoji_picker *p) {
    if (p->core) {
        ime_emoji_picker_free(p->core);
        p->core = NULL;
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
    drop_core(p);
    if (!inst || ime_emoji_picker_new(IME_EPK_LAYOUT_LIST, &p->core) != IME_OK) {
        p->core = NULL;
        sync_view(p);
        return;
    }
    (void)ime_emoji_picker_open(p->core, inst);
    sync_view(p);
}

void lc_emoji_close(lc_emoji_picker *p) {
    if (!p) return;
    drop_core(p);
    sync_view(p);
}

int lc_emoji_pick_index(lc_emoji_picker *p, ime_instance *inst,
                        size_t idx, char *out_glyph, size_t out_cap) {
    if (!p || !p->active || !p->core || !inst || !out_glyph || out_cap == 0) {
        return 0;
    }
    size_t n = 0;
    int32_t rc = ime_emoji_picker_pick(p->core, inst, (uint32_t)idx,
                                       (uint8_t *)out_glyph, out_cap, &n);
    if (rc != IME_EPK_COMMIT) {
        return 0;
    }
    lc_emoji_close(p);
    return 1;
}

/* Phím Linux (VK canonical + code point) ⇒ phím chuẩn của bộ chọn dùng chung. 0 = bỏ qua. */
static uint32_t map_key(uint32_t vk, uint32_t ch, int shift) {
    switch (vk) {
    case 0x1Bu: return IME_EPK_KEY_ESCAPE;
    case 0x0Du: return IME_EPK_KEY_ENTER;
    case 0x09u: return shift ? IME_EPK_KEY_SHIFT_TAB : IME_EPK_KEY_TAB;
    case 0x08u: return IME_EPK_KEY_BACKSPACE;
    case 0x25u: return IME_EPK_KEY_LEFT;
    case 0x26u: return IME_EPK_KEY_UP;
    case 0x27u: return IME_EPK_KEY_RIGHT;
    case 0x28u: return IME_EPK_KEY_DOWN;
    default: break;
    }
    if (ch >= 0x20u && ch != 0x7Fu) {
        return IME_EPK_KEY_CHAR;
    }
    return 0;
}

int lc_emoji_handle_key(lc_emoji_picker *p, ime_instance *inst,
                        uint32_t vk, uint32_t ch, int shift,
                        char *out_glyph, size_t out_cap) {
    if (!p || !p->active || !p->core) {
        return LC_EMOJI_CLOSED;
    }
    uint32_t key = map_key(vk, ch, shift);
    if (key == 0) {
        return LC_EMOJI_UPDATE;
    }
    size_t n = 0;
    int32_t rc = ime_emoji_picker_key(p->core, inst, key, ch, (uint8_t *)out_glyph, out_cap, &n);
    switch (rc) {
    case IME_EPK_COMMIT:
        lc_emoji_close(p);
        return LC_EMOJI_COMMIT;
    case IME_EPK_REDRAW:
        sync_view(p);
        return LC_EMOJI_UPDATE;
    case IME_EPK_CLOSED:
    case IME_EPK_NOT_OPEN:
        lc_emoji_close(p);
        return LC_EMOJI_CLOSED;
    default:
        /* Lỗi (bộ đệm glyph thiếu chỗ, tham số sai): không chèn gì, giữ trạng thái hiện có. */
        sync_view(p);
        return p->active ? LC_EMOJI_UPDATE : LC_EMOJI_CLOSED;
    }
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
                       "🔍 %s  (Enter chèn · Esc đóng)",
                       p->query);
    } else {
        (void)snprintf(out, out_cap,
                       "%s %s  |  Gõ tìm · Tab nhóm · 1-9/Enter chèn",
                       CATEGORY_ICONS[cat], CATEGORY_NAMES[cat]);
    }
}
