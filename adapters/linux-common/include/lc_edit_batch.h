/* lc_edit_batch.h — gom chỉnh sửa document của MỘT phím thành tối đa 1 lệnh xoá + 1 commit
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * R2-165 (đối chiếu ngosen/ngosen#1019, #207, #1016): giao thức text-input-v3 của Wayland chỉ
 * giữ TRẠNG THÁI CUỐI giữa hai lần `done` — mỗi `done` áp đúng MỘT `delete_surrounding_text`
 * và MỘT `commit_string`. mutter (GNOME) gom mọi lệnh bộ gõ gửi trong một vòng lặp vào một
 * `done` (`defer_done`), IBus đường input-method-v1 cũng chỉ giữ lệnh xoá cuối (ibus#2979).
 * Một phím mà gửi `commit(từ) + commit(dấu cách) + delete(N) + commit(bản sửa)` (trợ lý tự sửa
 * câu, Ctrl+Space sửa câu) thì trên GNOME Wayland commit trước bị đè và lệnh xoá ăn lẹm vào
 * chữ phía trước ⇒ MẤT CHỮ.
 *
 * Module này giữ "document ảo" = document thật (sau `delete_before` ký tự bị xoá trước caret)
 * + `text` (UTF-8 chờ commit). Lệnh xoá trong cùng phím ăn vào `text` trước rồi mới tới
 * document thật ⇒ cuối phím luôn gửi đúng MỘT xoá + MỘT commit, tương đương từng bước.
 *
 * Thuần C, không I/O, dùng chung IBus + Fcitx5 (G15). Đơn vị xoá = KÝ TỰ Unicode (quy ước
 * `delete_surrounding_text` của IBus và `deleteSurroundingText` của Fcitx5).
 */
#ifndef TEXTVN_LC_EDIT_BATCH_H
#define TEXTVN_LC_EDIT_BATCH_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LC_EDIT_BATCH_MAX 8192

typedef struct lc_edit_batch {
    int depth;              /* > 0: đang gom (lồng nhau được — chỉ tầng ngoài cùng gửi) */
    uint32_t delete_before; /* số ký tự xoá TRƯỚC caret trên document thật */
    size_t len;             /* số byte UTF-8 trong text */
    char text[LC_EDIT_BATCH_MAX];
} lc_edit_batch;

/* Bắt đầu gom (tầng ngoài cùng thì xoá sạch nội dung cũ). */
void lc_edit_batch_begin(lc_edit_batch *b);

/* Đang gom? */
int lc_edit_batch_active(const lc_edit_batch *b);

/* Xoá `n` ký tự trước caret của document ảo. Chỉ gọi khi đang gom. */
void lc_edit_batch_delete(lc_edit_batch *b, uint32_t n);

/* Thêm `len` byte UTF-8 vào cuối phần chờ commit. Trả 1 = đã gom; 0 = không gom (không đang
 * gom hoặc hết chỗ) ⇒ caller tự gửi (đã flush trước nếu cần giữ thứ tự). */
int lc_edit_batch_commit(lc_edit_batch *b, const char *utf8, size_t len);

/* Lấy phần đã gom ra để gửi NGAY (ví dụ trước khi forward phím, hoặc khi hết chỗ) rồi làm
 * rỗng. Trả 1 nếu có việc phải gửi. `*text` trỏ vào bộ đệm của `b` — gửi trước khi gom tiếp. */
int lc_edit_batch_take(lc_edit_batch *b, uint32_t *delete_before, const char **text, size_t *len);

/* Kết thúc một tầng gom. Ở tầng ngoài cùng: như `lc_edit_batch_take` (trả 1 nếu phải gửi).
 * Tầng trong: luôn trả 0 (tầng ngoài sẽ gửi). */
int lc_edit_batch_end(lc_edit_batch *b, uint32_t *delete_before, const char **text, size_t *len);

/* Số ký tự Unicode của một chuỗi UTF-8 (đếm byte dẫn — chuỗi hỏng vẫn không đọc vượt). */
size_t lc_utf8_char_count(const char *s, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_LC_EDIT_BATCH_H */
