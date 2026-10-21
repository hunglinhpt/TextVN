/* lc_assist_text.h — So khớp text trước caret cho "Trợ lý ngữ pháp" (D12/R-2)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Hàm THUẦN (không I/O, không phụ thuộc IBus/Fcitx5) nên test được **không cần
 * bộ gõ chạy thật** — đây là phần quyết định "được sửa hay không" của Mode 2
 * (tự sửa cả câu):
 *
 *   1. `lc_text_ends_with` — vùng text TRƯỚC caret trong document có **đúng**
 *      là đuôi câu mà engine biết không? (so byte, cả hai là UTF-8 hợp lệ và
 *      điểm cắt nằm trên ranh giới char ⇒ so byte ⇔ so chuỗi).
 *   2. `lc_assist_expected_document_tail` — đuôi document PHẢI có để được
 *      phép sửa, tính từ đuôi câu engine biết + trạng thái ký tự trigger.
 *
 * Vì sao tách riêng (G15): IBus và Fcitx5 lấy surrounding text theo cách khác
 * nhau (IBus = **bản cache** client gửi lên; Fcitx5 = truy vấn trực tiếp
 * client), nhưng phần **quyết định** phải giống hệt nhau ⇒ đặt ở đây.
 *
 * Không xác minh được ⇒ **không sửa** (fail-safe, D12). Hàm nào trả 0 là
 * "không xác minh được", KHÔNG phải "sửa được".
 */

#ifndef TEXTVN_LC_ASSIST_TEXT_H
#define TEXTVN_LC_ASSIST_TEXT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* R-2 (D12): text trước caret có KẾT THÚC bằng đuôi câu engine biết hay không.
 *
 * - `before` / `before_len`: text trước caret (UTF-8), `before_len` = số byte.
 * - `expected` / `expected_len`: đuôi câu engine biết (UTF-8).
 *
 * Trả 1 nếu khớp, 0 nếu không khớp **hoặc không xác minh được** (thiếu dữ
 * liệu, buffer rỗng, `expected` rỗng). `expected_len == 0` ⇒ 0 (fail-safe:
 * không có gì để đối chiếu ⇒ không được sửa).
 */
int lc_text_ends_with(const char *before, size_t before_len,
                      const char *expected, size_t expected_len);

/* Đuôi document PHẢI có trước caret để adapter được phép tự sửa (D12).
 *
 * - `engine_tail`: đuôi câu engine biết (UTF-8, lấy từ `lc_plan_sentence_auto_fix`
 *   → `original_tail`). Buffer câu của core **luôn kết thúc bằng ký tự trigger**
 *   (core ghi ký tự ranh giới vào buffer câu trước khi adapter áp kế hoạch).
 * - `trigger_char`: ký tự trigger (Unicode) — dùng khi trigger **chưa** nằm
 *   trong document (adapter sẽ tự commit ⇒ phải so với đuôi BỎ ký tự trigger).
 * - `trigger_committed`: 1 nếu ký tự trigger **đã** nằm trong document.
 * - `out` / `out_len`: buffer nhận (NUL-terminated).
 *
 * Trả 1 nếu tính được `out`, 0 nếu **không xác minh được** (fail-safe ⇒
 * caller không sửa). Trường hợp 0: `engine_tail` rỗng, buffer nhỏ hơn cần,
 * ký tự trigger không khớp ký tự cuối của `engine_tail`, hoặc `trigger_char`
 * không phải scalar Unicode hợp lệ.
 */
int lc_assist_expected_document_tail(const char *engine_tail,
                                     uint32_t trigger_char,
                                     int trigger_committed,
                                     char *out, size_t out_len);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_LC_ASSIST_TEXT_H */
