/* surrounding_match.h — So khớp "text trước caret" cho Trợ lý ngữ pháp (R-2/D12)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Vì sao tách thành module riêng:
 *   - chỉ có MỘT chỗ biết quy ước offset của Fcitx5 (`cursor()` = KÝ TỰ Unicode)
 *     ⇒ 1 nguồn sự thật, dễ kiểm chứng;
 *   - KHÔNG phụ thuộc FFI Rust / engine ⇒ test được mà không cần bộ gõ chạy
 *     thật (tests/test_surrounding_match.cpp).
 *
 * Hợp đồng (chi tiết + trích dẫn upstream trong surrounding_match.cpp):
 *   - `text`        : surrounding text của client (UTF-8).
 *   - `cursorChars` : offset của caret theo **KÝ TỰ Unicode** (quy ước Fcitx5).
 *   - `expected`    : đuôi câu engine biết (UTF-8).
 *   ⇒ trả true khi vùng TRƯỚC caret **kết thúc** bằng đúng `expected`.
 *   Không xác minh được ⇒ false ⇒ caller KHÔNG sửa (fail-safe, D12).
 */

#ifndef TEXTVN_FCITX5_SURROUNDING_MATCH_H
#define TEXTVN_FCITX5_SURROUNDING_MATCH_H

#include <string>

#include <fcitx/inputcontext.h>

namespace textvn {

/* Hàm thuần — không cần InputContext (test được độc lập). */
bool surroundingTextBeforeCaretMatches(const std::string &text, unsigned int cursorChars,
                                        const std::string &expected);

/* Bọc InputContext: đọc surrounding text + caret của client rồi gọi hàm thuần. */
bool textBeforeCaretMatches(fcitx::InputContext *ic, const std::string &expected);

} // namespace textvn

#endif /* TEXTVN_FCITX5_SURROUNDING_MATCH_H */
