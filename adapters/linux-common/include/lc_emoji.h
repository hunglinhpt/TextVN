/* SPDX-License-Identifier: GPL-3.0-or-later */
/* lc_emoji.h — Bộ chọn Emoji nhẹ dùng chung cho IBus và Fcitx5 (P12).
 *
 * Thiết kế siêu nhẹ (zero-dep, hoạt động đồng nhất trên X11 và Wayland):
 * - Dùng trực tiếp C-ABI `ime_emoji_search` + `ime_emoji_record_use`.
 * - Hiển thị qua lookup table / candidate list gốc của IBus và Fcitx5
 *   (tự render Noto Color Emoji / Twemoji, không cướp focus, không lỗi vị trí Wayland).
 * - Hỗ trợ 8 danh mục (Tab / Shift+Tab), lọc song ngữ VI/EN không dấu, phím 1..9 / Enter / Esc.
 * - G8: không ghi log nội dung tìm kiếm hay emoji người dùng chọn.
 */
#ifndef LC_EMOJI_H
#define LC_EMOJI_H

#include <stddef.h>
#include <stdint.h>
#include "textvn_ffi.h"

#ifdef __cplusplus
extern "C" {
#endif

#define LC_EMOJI_MAX_ITEMS      36
#define LC_EMOJI_PAGE_SIZE      9
#define LC_EMOJI_GLYPH_MAX      32
#define LC_EMOJI_SHORTCODE_MAX  32
#define LC_EMOJI_QUERY_MAX      64

/* Kết quả xử lý 1 phím khi Emoji Picker đang mở. */
#define LC_EMOJI_UPDATE  1   /* cập nhật lại candidate list + aux text */
#define LC_EMOJI_COMMIT  2   /* chèn `out_glyph` rồi đóng bảng */
#define LC_EMOJI_CLOSED  3   /* đóng bảng không chèn gì */

typedef struct lc_emoji_item {
    char    glyph[LC_EMOJI_GLYPH_MAX];
    char    shortcode[LC_EMOJI_SHORTCODE_MAX];
    uint8_t category;
} lc_emoji_item;

typedef struct lc_emoji_picker {
    int           active;
    uint8_t       category; /* 0..7 */
    char          query[LC_EMOJI_QUERY_MAX];
    size_t        query_len;
    lc_emoji_item items[LC_EMOJI_MAX_ITEMS];
    size_t        count;
    size_t        selected;
    /* SF-7: bộ chọn dùng chung (`ime_emoji_picker_*`) — chỉ sống khi bảng đang mở. */
    ime_emoji_picker *core;
} lc_emoji_picker;

/* Khởi tạo cấu trúc rỗng. */
void lc_emoji_init(lc_emoji_picker *p);

/* Kiểm tra tổ hợp phím Ctrl+Alt+E (X11/Wayland keysym 'e'/'E', bỏ qua CapsLock). */
int lc_is_emoji_hotkey(uint32_t keysym, int ctrl, int alt, int shift, int super_mod);

/* Mở bảng chọn Emoji ở nhóm 0 (Gần đây MRU) và nạp kết quả đầu tiên. */
void lc_emoji_open(lc_emoji_picker *p, const ime_instance *inst);

/* Đóng bảng chọn và xoá sạch bộ đệm truy vấn (G8). */
void lc_emoji_close(lc_emoji_picker *p);

/* Xử lý 1 phím khi `p->active == 1`:
 * - `vk`: mã VK chuẩn (0x1B Esc, 0x0D Enter, 0x09 Tab, 0x08 Backspace, 0x25..0x28 mũi tên)
 * - `ch`: Unicode code point (0 nếu không sinh ký tự in được)
 * - `shift`: 1 nếu đang giữ Shift
 * Trả `LC_EMOJI_UPDATE`, `LC_EMOJI_COMMIT` (điền `out_glyph`), hoặc `LC_EMOJI_CLOSED`. */
int lc_emoji_handle_key(lc_emoji_picker *p, ime_instance *inst,
                        uint32_t vk, uint32_t ch, int shift,
                        char *out_glyph, size_t out_cap);

/* Chọn trực tiếp mục theo chỉ số toàn cục `idx` (click chuột trên candidate list).
 * Trả 1 nếu hợp lệ (điền `out_glyph`, gọi `ime_emoji_record_use` và đóng bảng). */
int lc_emoji_pick_index(lc_emoji_picker *p, ime_instance *inst,
                        size_t idx, char *out_glyph, size_t out_cap);

/* Dựng dòng tiêu đề auxiliary text (hiện dải nhóm + query hiện tại + hướng dẫn ngắn). */
void lc_emoji_format_aux(const lc_emoji_picker *p, char *out, size_t out_cap);

#ifdef __cplusplus
}
#endif

#endif /* LC_EMOJI_H */
