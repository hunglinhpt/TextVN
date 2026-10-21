/* fake_input_context.h — client GIẢ dùng chung cho unit test Fcitx5 adapter
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * `fcitx::InputContext` là class ảo của libFcitx5Core ⇒ để QUAN SÁT được tham
 * số mà engine gửi đi (offset/size của lệnh xoá surrounding text), test cần
 * một client giả ghi lại lời gọi.
 *
 * Vì sao quan sát được: header fcitx5 `src/lib/fcitx/inputcontext.cpp`:
 *     void InputContext::deleteSurroundingText(int offset, unsigned int size) {
 *         deleteSurroundingTextImpl(offset, size);
 *     }
 * ⇒ gọi THẲNG `deleteSurroundingTextImpl` (KHÔNG đẩy event) ⇒ test thấy
 * ngay. Ngược lại `commitString` đẩy event (cần event loop) ⇒ không quan
 * sát được trong unit test này; test tập trung vào lệnh XOÁ + precondition.
 *
 * `SurroundingText` của client giả dùng ĐÚNG quy ước Fcitx5:
 * `setText(text, cursor, anchor)` với cursor/anchor là offset **KÝ TỰ Unicode**
 * (xem `src/lib/fcitx/surroundingtext.cpp` — `setText()` so với
 * `utf8::lengthValidated(text)`).
 */

#ifndef TEXTVN_FCITX5_TEST_FAKE_INPUT_CONTEXT_H
#define TEXTVN_FCITX5_TEST_FAKE_INPUT_CONTEXT_H

#include <string>
#include <vector>

#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>

namespace textvn {
namespace test {

/* Một lời gọi xoá surrounding text mà engine gửi cho client. */
struct DeleteCall {
    int offset;         /* vị trí bắt đầu xoá (âm = TRƯỚC caret) */
    unsigned int size;  /* SỐ KÝ TỰ bị xoá */
};

class FakeInputContext : public fcitx::InputContext {
public:
    explicit FakeInputContext(fcitx::InputContextManager &manager)
        : fcitx::InputContext(manager) {}
    ~FakeInputContext() override { destroy(); }

    const char *frontend() const override { return "textvn-test"; }
    void commitStringImpl(const std::string &) override {}
    void deleteSurroundingTextImpl(int offset, unsigned int size) override {
        deletes.push_back(DeleteCall{offset, size});
    }
    void forwardKeyImpl(const fcitx::ForwardKeyEvent &) override {}
    void updatePreeditImpl() override {}

    std::vector<DeleteCall> deletes;
};

} // namespace test
} // namespace textvn

#endif /* TEXTVN_FCITX5_TEST_FAKE_INPUT_CONTEXT_H */
