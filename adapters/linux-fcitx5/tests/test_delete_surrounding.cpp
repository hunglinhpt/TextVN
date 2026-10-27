/* test_delete_surrounding.cpp — hồi quy quy ước xoá surrounding text của Fcitx5
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * VÌ SAO CÓ TEST NÀY
 * ------------------
 * API thật của fcitx5 (header `src/lib/fcitx/inputcontext.h`):
 *
 *     void deleteSurroundingText (int offset, unsigned int size);
 *     /// @param offset offset of deletion start, in UCS4 char.
 *     /// @param size   length of the deletion in UCS4 char.
 *
 * ⇒ tham số 2 là **SỐ KÝ TỰ CẦN XOÁ**, KHÔNG phải "số ký tự sau caret".
 *   `size == 0` ⇒ **xoá 0 ký tự** ⇒ đuôi câu cũ không bị xoá ⇒ lệnh commit bản
 *   sửa chèn thêm ⇒ **NHÂN ĐÔI CÂU**.
 *
 * Lỗi này đã lọt qua CI vì test cũ KHÔNG kiểm được hiệu ứng xoá: mọi lời gọi
 * đều đi qua `fcitx::InputContext` thật ⇒ không quan sát được tham số gửi đi.
 * Test này dùng một **client giả** (fake `fcitx::InputContext`) để ĐẾM đúng
 * số ký tự bị xoá và offset của lệnh xoá.
 *
 * Chuỗi hiện thực (fcitx5 → fcitx5-gtk → GTK3) — cả 3 tầng cùng quy ước:
 *   fcitx5 (dbusfrontend): signal `DeleteSurroundingText` (i offset, u size).
 *   fcitx5-gtk: `delete-surrounding-text` (offset_from_cursor, nchars).
 *   GTK3 `GtkIMContext::delete-surrounding`:
 *     "@offset:  the character offset from the cursor position of the text
 *               to be deleted. A negative value indicates a position before
 *               the cursor.
 *      @n_chars: the number of characters to be deleted"
 *
 * Phần so khớp "text trước caret" (precondition R-2/D12) có test riêng:
 * tests/test_surrounding_match.cpp.
 */

#include <cassert>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>

#include "engine.h"
#include "fake_input_context.h"
#include "surrounding_match.h"

#ifndef TEXTVN_FCITX5_SRC_DIR
#error "TEXTVN_FCITX5_SRC_DIR phải được define (CMake: target_compile_definitions)"
#endif

using namespace textvn;
using textvn::test::FakeInputContext;

namespace {

/* "tôi sẽ mất mãi mãi" — 18 KÝ TỰ Unicode / 23 byte UTF-8.
 * Viết bằng escape hex để byte luôn giống nhau mọi platform. */
const char kSentence[] = "t\xC3\xB4i s\xE1\xBA\xBD m\xE1\xBA\xA5t m\xC3\xA3i m\xC3\xA3i";
/* "mãi" — 4 ký tự / 5 byte. */
const char kLastWord[] = "m\xC3\xA3i";

/* --- Guard chống tái diễn: trong adapter KHÔNG được có lời gọi
 *     deleteSurroundingText thô nào ngoài helper `deleteBeforeCaret`.
 *     Nếu ai đó thêm lại `ic->deleteSurroundingText(-n, 0)` ⇒ test này đỏ.
 *
 *     Kiểm tra bằng RUNTIME (không dùng assert) để không bao giờ "pass rỗng"
 *     khi NDEBUG bật. --- */
bool scanSourceForRawDeleteCalls(const std::string &path, size_t *totalCalls) {
    std::ifstream in(path);
    if (!in.good()) {
        std::fprintf(stderr, "FAIL: khong doc duoc %s\n", path.c_str());
        return false;
    }
    std::stringstream buf;
    buf << in.rdbuf();
    const std::string src = buf.str();

    const std::string needle = "deleteSurroundingText(";
    size_t pos = 0;
    while ((pos = src.find(needle, pos)) != std::string::npos) {
        ++(*totalCalls);
        const size_t lineStart = src.rfind('\n', pos) + 1;
        size_t lineEnd = src.find('\n', pos);
        if (lineEnd == std::string::npos) lineEnd = src.size();
        const std::string line = src.substr(lineStart, lineEnd - lineStart);
        /* Đúng quy ước: (a) offset âm (trước caret) + size = số ký tự cần xoá
         * (deleteBeforeCaret); (b) selection replace — offset ∈ {0, −n} với
         * size = n ≥ 1 (deleteSelectionRange, OP-007/R2-139 — invariant trong
         * lc_selection_slice: caret luôn là MỘT đầu selection). */
        const bool before_caret =
            line.find("deleteSurroundingText(-static_cast<int>(n), n)") !=
            std::string::npos;
        const bool selection_range =
            line.find("deleteSurroundingText(offset, nchars)") != std::string::npos;
        if (!before_caret && !selection_range) {
            std::fprintf(stderr, "FAIL: loi goi sai quy uoc trong %s:\n  %s\n",
                         path.c_str(), line.c_str());
            return false;
        }
        pos += needle.size();
    }
    return true;
}

} // namespace

int main() {
    fcitx::InputContextManager manager;
    FakeInputContext ic(manager);

    /* ---- 1) Quy ước API: xoá n ký tự TRƯỚC caret ⇒ (offset = -n, size = n) ---- */
    /* ---- 2b) OP-007/R2-139: xoá selection qua deleteSelectionRange ----
     * Selection NGƯỢC hướng (caret ở đầu selection): offset = 0 — xoá SAU
     * caret; selection THUẬN (caret ở cuối): offset = -n — xoá TRƯỚC caret. */
    deleteSelectionRange(&ic, 0, 4);
    assert(ic.deletes.back().offset == 0);
    assert(ic.deletes.back().size == 4u);
    deleteSelectionRange(&ic, -4, 4);
    assert(ic.deletes.back().offset == -4);
    assert(ic.deletes.back().size == 4u);
    deleteSelectionRange(&ic, 0, 0); /* size = 0 bị helper chặn — không gọi */
    assert(ic.deletes.size() == 2 + 1); /* 7 + 4 + 4 trước đó, size=0 không cộng */

    deleteBeforeCaret(&ic, 7);
    assert(ic.deletes.size() == 1);
    assert(ic.deletes[0].offset == -7);   /* âm ⇒ vùng xoá nằm TRƯỚC caret */
    assert(ic.deletes[0].size == 7u);      /* size = SỐ KÝ TỰ CẦN XOÁ */
    assert(ic.deletes[0].size != 0u);      /* 0 ⇒ xoá 0 ký tự ⇒ nhân đôi câu */

    /* ---- 2) n = 0 ⇒ KHÔNG gọi API (tránh size = 0) ---- */
    deleteBeforeCaret(&ic, 0);
    assert(ic.deletes.size() == 1);

    /* ---- 3) ic = nullptr ⇒ không crash ---- */
    deleteBeforeCaret(nullptr, 5);
    assert(ic.deletes.size() == 1);

    /* ---- 4) Mode 1 (chọn ứng viên): xoá đúng số ký tự cũ rồi gõ bản sửa ---- */
    ic.setCapabilityFlags(fcitx::CapabilityFlag::SurroundingText);
    assert(ic.capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText));
    ic.surroundingText().setText("viet", 4, 4);
    assert(ic.surroundingText().isValid());

    assert(applyAssistCandidate(&ic, 3, "viet", "vi\xE1\xED\xBB\xA1t"));
    assert(ic.deletes.size() == 2);
    assert(ic.deletes[1].offset == -3);
    assert(ic.deletes[1].size == 3u);

    /* ---- 5) Fail-safe: caret ở chỗ khác ⇒ KHÔNG xoá gì ---- */
    ic.surroundingText().setText("khac", 4, 4);
    assert(!applyAssistCandidate(&ic, 3, "viet", "vi\xE1\xED\xBB\xA1t"));
    assert(ic.deletes.size() == 2);

    /* ---- 6) Client không hỗ trợ surrounding text ⇒ KHÔNG xoá gì ---- */
    ic.setCapabilityFlags(fcitx::CapabilityFlag::NoFlag);
    assert(!applyAssistCandidate(&ic, 3, "viet", "vi\xE1\xED\xBB\xA1t"));
    assert(ic.deletes.size() == 2);

    /* ---- 7) Mode 1 với câu TIẾNG VIỆT (ký tự nhiều byte): precondition phải
     *         khớp (cursor là offset KÝ TỰ) ⇒ vẫn xoá đúng số ký tự ---- */
    ic.setCapabilityFlags(fcitx::CapabilityFlag::SurroundingText);
    ic.surroundingText().setText(kSentence, 18, 18);   /* caret cuối câu */
    assert(ic.surroundingText().isValid());
    assert(applyAssistCandidate(&ic, 3, kLastWord, "m\xC3\xA1y"));
    assert(ic.deletes.size() == 3);
    assert(ic.deletes[2].offset == -3);
    assert(ic.deletes[2].size == 3u);

    /* ---- 8) Hàm thuần: caret ngoài text ⇒ không xác minh được ---- */
    assert(!surroundingTextBeforeCaretMatches(kSentence, 25, kSentence));
    assert(!surroundingTextBeforeCaretMatches(kSentence, 18, ""));
    /* Text không phải UTF-8 hợp lệ ⇒ không xác minh được ⇒ KHÔNG sửa. */
    assert(!surroundingTextBeforeCaretMatches(std::string("a\xC3(b"), 2, "b"));
    /* Hợp lệ: khớp hậu tố theo KÝ TỰ. */
    assert(surroundingTextBeforeCaretMatches(kSentence, 18, kLastWord));
    assert(surroundingTextBeforeCaretMatches(kSentence, 18, kSentence));

    /* ---- 9) Guard chống tái diễn trên mã nguồn adapter ---- */
    const std::string srcDir(TEXTVN_FCITX5_SRC_DIR);
    const char *files[] = {"/src/engine.cpp", "/src/engine.h", "/src/addon.cpp"};
    size_t totalCalls = 0;
    for (const char *rel : files) {
        if (!scanSourceForRawDeleteCalls(srcDir + rel, &totalCalls)) {
            return 1;
        }
    }
    if (totalCalls != 1) {
        std::fprintf(stderr,
                     "FAIL: adapter phai co DUNG 1 loi goi deleteSurroundingText "
                     "(trong deleteBeforeCaret), hien co %zu\n",
                     totalCalls);
        return 1;
    }
    std::printf("  guard: %zu loi goi deleteSurroundingText, dung quy uoc\n",
                totalCalls);

    std::printf("test_fcitx5_delete_surrounding: OK\n");
    return 0;
}
