/* surrounding_match.cpp — So khớp "text trước caret" (R-2 / D12)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * R-2 (fail-safe): vùng text TRƯỚC caret trong document có **đúng** là đuôi câu
 * mà engine biết không? Không đọc được (client không hỗ trợ surrounding text)
 * hoặc lệch ⇒ trả false ⇒ **không sửa** (chốt chặn "xoá nhầm text lạ" khi
 * buffer engine stale: user bấm chuột đặt caret chỗ khác rồi gõ `.`).
 *
 * Fcitx5 khác IBus: `surroundingText()` là **truy vấn trực tiếp** client (DBus)
 * ⇒ luôn phản ánh document hiện tại (IBus chỉ có **cache** client đẩy lên
 * TRƯỚC khi phím) ⇒ không cần ghép "text vừa commit" như bản IBus.
 *
 * ---------------------------------------------------------------------------
 * QUY ƯỚC OFFSET CỦA FCITX5 (đây là chỗ đã sai trước đây)
 * ---------------------------------------------------------------------------
 * `SurroundingText::cursor()` là offset theo **KÝ TỰ Unicode** (char), KHÔNG
 * phải byte. Bằng chứng từ header + source upstream:
 *
 *   - `src/lib/fcitx/surroundingtext.h`:
 *         void setText(const std::string &text, unsigned int cursor,
 *                      unsigned int anchor);
 *         unsigned int cursor() const;
 *         /// @param cursor offset of cursor in character.
 *   - `src/lib/fcitx/surroundingtext.cpp` — `setText()` kiểm tra
 *         `length = utf8::lengthValidated(text)` rồi mới lưu `cursor_`
 *         ⇒ `cursor` là offset Kð TỰ (không thể là byte: byte offset có thể >
 *         số ký tự); `deleteText()` cắt bằng `utf8::nextNChar` (ký tự).
 *   - fcitx5-gtk tự dịch byte→char TRƯỚC khi gửi lên DBus:
 *         `cursor_pos = g_utf8_strlen(p, cursor_index);`
 *     (GTK trả cursor theo BYTE ⇒ fcitx5-gtk phải đổi sang ký tự).
 *
 * ⇒ Cắt `std::string` (đơn vị BYTE) bằng `cursor` ⇒ cắt LỆCH chỗ với tiếng
 *   Việt (ký tự 2–3 byte) ⇒ precondition luôn lệch ⇒ **Mode 1 / Mode 2 / undo
 *   trên Fcitx5 không bao giờ chạy**. Lỗi này lọt qua CI vì test cũ không hề
 *   kiểm hiệu ứng so khớp — xem tests/test_surrounding_match.cpp.
 *
 * ---------------------------------------------------------------------------
 * SO KHỚP THEO KÝ TỰ UNICODE
 * ---------------------------------------------------------------------------
 * `expected` là Unicode (buffer của core — `ime_assist_sentence_tail` trả UTF-8);
 * surrounding text của client cũng là UTF-8. Ta so theo **KÝ TỰ**: điểm cắt
 * (byte offset) được dịch từ offset ký tự, nên lát "trước caret" luôn nằm trên
 * ranh giới ký tự; hai lát UTF-8 hợp lệ có độ dài byte bằng nhau và cùng số ký
 * tự thì so byte ⇔ so ký tự (mã hoá byte khác nhau của cùng một ký tự — ví dụ
 * NFC vs NFD — là *ký tự khác*, vẫn phải lệch).
 *
 * Với bảng mã legacy (VNI/TCVN3) số **glyph** của core ≠ số **ký tự Unicode**
 * trong document ⇒ đó là rủi ro riêng ở `delete_glyphs` (xem
 * `checkAndApplyAssist`), không thuộc hàm này.
 */

#include "surrounding_match.h"

#include "lc_assist_text.h"

#include <fcitx-utils/utf8.h>

#include <cstddef>

namespace textvn {

bool surroundingTextBeforeCaretMatches(const std::string &text, unsigned int cursorChars,
                                        const std::string &expected) {
    if (expected.empty()) return false; /* không có gì để đối chiếu ⇒ fail-safe */
    /* Text của client phải là UTF-8 hợp lệ (Fcitx5 tự invalidate nếu không)
     * — nếu không hợp lệ ⇒ không xác minh được ⇒ fail-safe. */
    const size_t textChars = fcitx::utf8::lengthValidated(text);
    if (textChars == fcitx::utf8::INVALID_LENGTH) return false;
    const size_t expectedChars = fcitx::utf8::lengthValidated(expected);
    if (expectedChars == fcitx::utf8::INVALID_LENGTH || expectedChars == 0) return false;
    /* Caret nằm ngoài text ⇒ dữ liệu không nhất quán ⇒ không xác minh được.
     * TUYỆT ĐỐI không so cả phần SAU caret — sẽ khớp nhầm rồi xoá nhầm vùng
     * khác. */
    if (cursorChars > textChars) return false;
    /* Vùng trước caret có ít ký tự hơn `expected` ⇒ không thể là hậu tố. */
    if (cursorChars < expectedChars) return false;
    /* Dịch offset KÝ TỰ → offset BYTE.
     * Header fcitx5 `src/lib/fcitx-utils/utf8.h`:
     *     /// get the byte length of next N utf-8 character.
     *     template <typename Iter>
     *     inline ssize_t ncharByteLength(Iter iter, size_t n);
     * Hàm này dừng ở byte NUL ⇒ không đọc quá cuối chuỗi. */
    const ssize_t cursorBytes = fcitx::utf8::ncharByteLength(text.begin(), cursorChars);
    if (cursorBytes < 0 || static_cast<size_t>(cursorBytes) > text.size()) return false;
    const std::string before = text.substr(0, static_cast<size_t>(cursorBytes));
    /* So hậu tố bằng hàm DÙNG CHUNG của linux-common (G15). Hàm đó so byte,
     * nhưng lát "trước caret" đã CĂN theo ranh giới ký tự và cả hai chuỗi là
     * UTF-8 hợp lệ ⇒ tương đương so theo KÝ TỰ Unicode (xem comment trong
     * `lc_assist_text.h`). */
    return lc_text_ends_with(before.data(), before.size(), expected.data(),
                              expected.size()) != 0;
}

bool textBeforeCaretMatches(fcitx::InputContext *ic, const std::string &expected) {
    if (!ic || expected.empty()) return false;
    fcitx::SurroundingText surrounding = ic->surroundingText();
    if (!surrounding.isValid()) return false;
    return surroundingTextBeforeCaretMatches(surrounding.text(), surrounding.cursor(),
                                             expected);
}

} // namespace textvn
