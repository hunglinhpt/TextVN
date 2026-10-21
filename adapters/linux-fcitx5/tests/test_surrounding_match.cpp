/* test_surrounding_match.cpp — hồi quy việc so khớp text TRƯỚC caret (R-2/D12)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * VÌ SAO CÓ TEST NÀY
 * ------------------
 * `SurroundingText::cursor()` của Fcitx5 là offset theo **KÝ TỰ Unicode**
 * (char), KHÔNG phải byte:
 *   - `src/lib/fcitx/surroundingtext.cpp`: `setText()` so với
 *     `length = utf8::lengthValidated(text)` rồi mới lưu `cursor_`;
 *     `deleteText()` cắt bằng `utf8::nextNChar` (ký tự).
 *   - fcitx5-gtk tự dịch byte→char trước khi gửi DBus:
 *     `cursor_pos = g_utf8_strlen(p, cursor_index)` (GTK trả cursor theo BYTE).
 *
 * Lỗi cũ: `text.substr(0, cursor)` cắt `std::string` (đơn vị BYTE) tại vị trí
 * KÝ TỰ ⇒ với câu tiếng Việt (ký tự 2–3 byte) vùng "trước caret" bị cắt lệch
 * ⇒ precondition LUÔN lệch ⇒ **Mode 1 / Mode 2 / undo trên Fcitx5 không bao giờ
 * chạy** — mà lỗi này lọt qua CI vì test cũ không hề kiểm hiệu ứng so khớp.
 *
 * Test này dùng **client giả** (fake `fcitx::InputContext`) và câu tiếng Việt
 * có ký tự nhiều byte ⇒ bắt được lỗi cũ (xem README của test: chạy lại test
 * này trên `git show HEAD:adapters/linux-fcitx5/src/engine.cpp` ⇒ FAIL).
 */

#include <cassert>
#include <cstdio>
#include <string>

#include "engine.h"
#include "fake_input_context.h"
#include "surrounding_match.h"

using namespace textvn;
using textvn::test::FakeInputContext;

namespace {

/* "tôi sẽ mất mãi mãi" — 18 KÝ TỰ Unicode / 23 byte UTF-8.
 * Viết bằng escape hex để byte luôn giống nhau mọi platform. */
const char kSentence[] = "t\xC3\xB4i s\xE1\xBA\xBD m\xE1\xBA\xA5t m\xC3\xA3i m\xC3\xA3i";
/* "mãi" — 4 ký tự / 5 byte. */
const char kLastWord[] = "m\xC3\xA3i";
/* "tôi sẽ mất" — 10 ký tự / 13 byte. */
const char kFirstPart[] = "t\xC3\xB4i s\xE1\xBA\xBD m\xE1\xBA\xA5t";
/* "tôi sẽ mấtxyz" — hậu tố KHÔNG khớp. */
const char kWrong[] = "t\xC3\xB4i s\xE1\xBA\xBD m\xE1\xBA\xA5txyz";

/* Số KÝ TỰ Unicode của chuỗi (tính tay, không cần thư viện). */
size_t charCount(const char *s) {
    size_t n = 0;
    for (const unsigned char *p = reinterpret_cast<const unsigned char *>(s); *p; ++p) {
        if ((*p & 0xC0) != 0x80) {
            ++n;
        }
    }
    return n;
}

} // namespace

int main() {
    /* ---- 0) Chuẩn bị: số ký tự / số byte của dữ liệu test ---- */
    const size_t sentenceChars = charCount(kSentence);
    const size_t sentenceBytes = std::string(kSentence).size();
    assert(sentenceChars == 18);
    assert(sentenceBytes == 25);
    /* Chứng minh câu test CÓ ký tự nhiều byte (nếu không, test vô nghĩa). */
    assert(sentenceBytes > sentenceChars);

    fcitx::InputContextManager manager;
    FakeInputContext ic(manager);

    /* ---- 1) Caret ở CUỐI câu, expected = TOÀN BỘ câu ⇒ phải khớp ---- */
    ic.surroundingText().setText(kSentence, 18, 18);
    assert(ic.surroundingText().isValid());
    assert(textBeforeCaretMatches(&ic, kSentence));

    /* ---- 2) Caret ở cuối, expected = hậu tố 1 từ ("mãi") ⇒ phải khớp ---- */
    assert(textBeforeCaretMatches(&ic, kLastWord));

    /* ---- 3) Caret ở GIỮA câu, expected = phần trước caret ⇒ phải khớp ---- */
    ic.surroundingText().setText(kSentence, 10, 10);
    assert(textBeforeCaretMatches(&ic, kFirstPart));

    /* ---- 4) expected KHÔNG phải hậu tố ⇒ phải lệch (fail-safe) ---- */
    assert(!textBeforeCaretMatches(&ic, kWrong));

    /* ---- 5) expected là phần SAU caret ⇒ phải lệch (tuyệt đối không so
     *         phần sau caret — sẽ khớp nhầm rồi xoá nhầm vùng khác) ---- */
    assert(!textBeforeCaretMatches(&ic, kLastWord));

    /* ---- 6) Caret nằm NGOÀI text: Fcitx5 tự invalidate surrounding text
     *         (`setText` so `length < cursor`) ⇒ không xác minh được ⇒ lệch.
     *         (Ngoài ra hàm thuần cũng chặn riêng trường hợp này.) ---- */
    ic.surroundingText().setText(kSentence, 25, 25);
    assert(!ic.surroundingText().isValid());
    assert(!textBeforeCaretMatches(&ic, kSentence));

    /* ---- 7) Surrounding text không hợp lệ ⇒ lệch ---- */
    ic.surroundingText().invalidate();
    assert(!ic.surroundingText().isValid());
    assert(!textBeforeCaretMatches(&ic, kSentence));

    /* ---- 8) Hồi quy ASCII: hành vi cũ phải giữ nguyên ---- */
    ic.surroundingText().setText("viet", 4, 4);
    assert(textBeforeCaretMatches(&ic, "viet"));
    assert(!textBeforeCaretMatches(&ic, "vie"));
    assert(!textBeforeCaretMatches(&ic, "xiet"));

    std::printf("test_fcitx5_surrounding_match: OK\n");
    return 0;
}
