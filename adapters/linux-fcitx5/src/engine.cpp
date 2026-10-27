/* engine.cpp — Engine Fcitx5 của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Cùng mô hình với IBus (lc_compose.h): cả từ trong preedit, commit ở ranh giới.
 * Mọi đường lỗi fail-open: không bao giờ ném exception ra khỏi callback của Fcitx5.
 */

#include "engine.h"

#include "lc_assist_text.h"
#include "surrounding_match.h"

#include <fcitx-utils/capabilityflags.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/misc.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <fcitx/statusarea.h>
#include <fcitx/text.h>
#include <fcitx/userinterface.h>
#include <fcitx/userinterfacemanager.h>

#include <dlfcn.h>

#include <chrono>
#include <cstring>
#include <string>

namespace textvn {

/* ---- Tiện ích thuần ---- */

/* Số mili giây từ epoch steady (đo hạn 5 s của undo — R-4). */
static uint64_t nowMs() {
    using namespace std::chrono;
    return static_cast<uint64_t>(
        duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count());
}

/* R-2 (fail-safe): so **text trước caret** trong document với đuôi câu mà
 * engine biết — logic + trích dẫn upstream nằm trong `surrounding_match.cpp`
 * (module riêng: chỉ có MỘT chỗ biết quy ước offset của Fcitx5, và test được
 * không cần engine chạy thật).
 *
 * Tóm tắt: `SurroundingText::cursor()` của Fcitx5 là offset theo **KÝ TỰ
 * Unicode** (KHÔNG phải byte) — xem surrounding_match.cpp. Trước đây adapter
 * cắt `std::string` (đơn vị BYTE) bằng cursor ⇒ cắt lệch chỗ với tiếng Việt
 * ⇒ precondition luôn lệch ⇒ Mode 1 / Mode 2 / undo không bao giờ chạy.
 *
 * `expected` là Unicode (buffer của core, `ime_assist_sentence_tail` trả UTF-8);
 * surrounding text của client cũng là UTF-8 ⇒ so khớp được. Với bảng mã legacy
 * (VNI/TCVN3) số **glyph** ≠ số **ký tự Unicode** ⇒ xem `delete_glyphs` ở
 * `checkAndApplyAssist` (rủi ro riêng, không thuộc hàm này). */

std::string utf32ToUtf8(const uint32_t *text, size_t len) {
    std::string out;
    for (size_t i = 0; i < len; ++i) {
        out += fcitx::utf8::UCS4ToUTF8(text[i]);
    }
    return out;
}

/* ---- Xoá surrounding text — đúng quy ước API Fcitx5 ----
 *
 * Header fcitx5 `src/lib/fcitx/inputcontext.h`:
 *
 *     /// Ask client to delete a range of surrounding text.
 *     void deleteSurroundingText (int offset, unsigned int size);
 *     ...
 *     /// Send the delete Surrounding Text request to client.
 *     ///
 *     /// @param offset offset of deletion start, in UCS4 char.
 *     /// @param size length of the deletion in UCS4 char.
 *
 * ⇒ `offset` = vị trí BẮT ĐẦU xoá, lệch so với caret (ÂM = TRƯỚC caret);
 *   `size`   = **SỐ KÝ TỰ CẦN XOÁ** (đơn vị: ký tự Unicode).
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
 * ⇒ `size == 0` ⇒ **XOÁ 0 KÝ TỰ** (KHÔNG phải "xoá tới đầu/cuối") ⇒ đuôi câu
 * cũ vẫn còn trong document ⇒ `commitString(replacement)` ⇒ **NHÂN ĐÔI CÂU**.
 * (Đây chính là lỗi đã lọt qua CI: trước đây adapter gọi `(-n, 0)`.)
 *
 * Vì vậy: MỌI lời gọi xoá trong adapter này PHẢI đi qua hàm này, và tham số
 * thứ 2 LUÔN bằng số ký tự cần xoá (không phải 0). Test hồi quy:
 * tests/test_delete_surrounding.cpp (fake client đếm số ký tự bị xoá). */
void deleteBeforeCaret(fcitx::InputContext *ic, uint32_t n) {
    if (!ic || n == 0) return;
    ic->deleteSurroundingText(-static_cast<int>(n), n);
}

/* OP-007 · R2-139: xoá ĐÚNG vùng selection (chord Ctrl+Shift+F9) — khác
 * `deleteBeforeCaret` ở chỗ selection có thể nằm ở CẢ HAI phía caret:
 * `offset = del_offset ∈ {0, −char_len}` (caret luôn là MỘT đầu selection —
 * invariant được chứng minh trong `lc_selection_slice`, test C
 * `test_transcode_selection_slice`), `nchars = char_len ≥ 1` nên quy ước
 * "size == 0 ⇒ nhân đôi câu" không thể xảy ra. Gate
 * `test_fcitx5_delete_surrounding` allowlist đúng 2 dạng gọi này. */
void deleteSelectionRange(fcitx::InputContext *ic, int offset, unsigned int nchars) {
    if (!ic || nchars == 0) return;
    ic->deleteSurroundingText(offset, nchars);
}

/* Mode 1 (chọn ứng viên trong candidate list): xoá đúng số ký tự cũ rồi gõ
 * bản sửa.
 *
 * Fail-safe (R-2/D12): chỉ xoá khi
 *   1) client hỗ trợ surrounding text và surrounding text hợp lệ, và
 *   2) vùng trước caret **đúng** là đuôi câu engine biết (caret ở chỗ khác
 *      ⇒ không xoá nhầm vùng text khác).
 * Không xác minh được ⇒ trả false ⇒ caller KHÔNG làm gì cả.
 *
 * Chỉ xoá **trước** caret: phần sau caret không thuộc phạm vi sửa (xoá cả 2
 * phía ⇒ mất dữ liệu người dùng) ⇒ `deleteBeforeCaret` (offset âm). */
bool applyAssistCandidate(fcitx::InputContext *ic, uint32_t deleteGlyphs,
                           const std::string &originalTail,
                           const std::string &replacement) {
    if (!ic) return false;
    if (deleteGlyphs > 0) {
        if (!ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) ||
            !ic->surroundingText().isValid()) {
            return false;
        }
        if (!originalTail.empty() && !textBeforeCaretMatches(ic, originalTail)) {
            return false;
        }
        deleteBeforeCaret(ic, deleteGlyphs);
    }
    if (!replacement.empty()) {
        ic->commitString(replacement);
    }
    return true;
}

/* ---- OP-007 (P5 §7.4) · R2-139: Ctrl+Shift+F9 chuyển mã vùng bôi đen ----
 *
 * Selection = đoạn giữa anchor và cursor của surrounding text (CẢ HAI đếm
 * theo KÝ TỰ Unicode — surrounding_match.cpp §"QUY ƯỚC OFFSET"); anchor ==
 * cursor khi không có selection. Trần 64 000 ký tự —
 * `selection_check::MAX_TRANSCODE_CHARS` của Windows.
 *
 * Không selection / trần vượt / UTF-8 lỗi ⇒ KHÔNG làm gì và KHÔNG nuốt phím
 * (fail-open: F9 tới app — khác Windows 1 nhí vì API Fcitx5 đồng bộ, không có
 * RequestEditSession async để nuốt). Đã đọc được selection mà engine không
 * đổi gì ⇒ không ghi document nhưng VẪN nuốt (khớp "đã xử lý đường convert").
 *
 * Recheck chống TOCTOU (đọc lại trước xoá): surrounding là truy vấn DBus
 * TRỰC TIẾP nên 2 lần gọi có thể lệch nhau — lệch ⇒ KHÔNG ghi document
 * (fail-safe R-2), hủy phím (như Windows khi session gặp sự cố). */
static constexpr uint32_t kMaxTranscodeChars = 64000;

static bool performSelectionConvert(TextVNState *st) {
    if (!st || !st->ic || !st->inst) return false;
    if (!st->ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) return false;
    /* S3 (review 1): ô mật khẩu — selection trong ô password chính là mật
     * khẩu; không đọc/không sửa. */
    if (st->ic->capabilityFlags().testAny(fcitx::CapabilityFlag::PasswordOrSensitive)) {
        return false;
    }
    /* Chụp MỘT lần surrounding: text/anchor/cursor phải cùng chụp. */
    const fcitx::SurroundingText surrounding = st->ic->surroundingText();
    if (!surrounding.isValid()) return false;
    const unsigned anchor = surrounding.anchor();
    const unsigned cursor = surrounding.cursor();
    const std::string &text = surrounding.text();
    lc_selection_info info{};
    if (!lc_selection_slice(text.c_str(), cursor, anchor, kMaxTranscodeChars, &info)) {
        return false; /* không selection / trần / UTF-8 lỗi */
    }
    const std::string sel = text.substr(info.start_byte, info.byte_len);

    /* FFI ime_convert_selection: UTF-8 vào/ra; cap 2× đầu vào + 32 (khớp
     * CoreBridge Swift — đầu ra không dài hơn đáng kể); 0 = không đổi / không fit. */
    std::string out(info.byte_len * 2 + 32, '\0');
    const int n = ime_convert_selection(st->inst, sel.data(), sel.size(), out.data(), out.size());
    if (n <= 0 || static_cast<size_t>(n) >= out.size()) {
        return true; /* không đổi gì — nuốt phím nhưng không ghi document */
    }
    out.resize(static_cast<size_t>(n));
    if (out == sel) return true; /* không đổi gì */

    /* Recheck chống TOCTOU: lệch ⇒ KHÔNG ghi document (R-2). */
    const fcitx::SurroundingText recheck = st->ic->surroundingText();
    if (!recheck.isValid() || recheck.text() != text || recheck.anchor() != anchor ||
        recheck.cursor() != cursor) {
        return true;
    }
    deleteSelectionRange(st->ic, static_cast<int>(info.del_offset),
                         static_cast<unsigned int>(info.char_len));
    st->ic->commitString(out);
    /* Document đã đổi: buffer câu engine stale + undo trợ lý không còn đúng
     * (Windows: reset_engine + clear_assist_undo). */
    ime_reset(st->inst);
    st->assistUndoValid = false;
    return true;
}

void mapFcitxKey(const fcitx::Key &key, uint32_t *vk, uint32_t *ch) {
    const fcitx::KeySym sym = key.sym();
    switch (sym) {
    case FcitxKey_BackSpace:
        *vk = LC_VK_BACK;
        *ch = 0;
        return;
    case FcitxKey_Tab:
    case FcitxKey_KP_Tab:
    case FcitxKey_ISO_Left_Tab:
        *vk = LC_VK_TAB;
        *ch = 0;
        return;
    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
    case FcitxKey_ISO_Enter:
        *vk = LC_VK_RETURN;
        *ch = 0;
        return;
    case FcitxKey_Escape:
        *vk = LC_VK_ESCAPE;
        *ch = 0;
        return;
    case FcitxKey_KP_Space:
        *vk = LC_VK_SPACE;
        *ch = ' ';
        return;
    default:
        break;
    }
    /* keySymToUnicode trả 0 cho phím không sinh ký tự (mũi tên, F-key, Delete…). */
    *ch = fcitx::Key::keySymToUnicode(sym);
    if (sym >= FcitxKey_a && sym <= FcitxKey_z) {
        *vk = static_cast<uint32_t>(sym - FcitxKey_a + 'A');
    } else if (static_cast<uint32_t>(sym) < 0x80) {
        *vk = static_cast<uint32_t>(sym);
    } else {
        *vk = 0;
    }
}

uint32_t fcitxMods(fcitx::KeyStates states) {
    uint32_t mods = 0;
    if (states.test(fcitx::KeyState::Shift)) mods |= IME_MOD_SHIFT;
    if (states.test(fcitx::KeyState::Ctrl)) mods |= IME_MOD_CTRL;
    if (states.test(fcitx::KeyState::Alt)) mods |= IME_MOD_ALT;
    if (states.test(fcitx::KeyState::Super) || states.test(fcitx::KeyState::Super2) ||
        states.test(fcitx::KeyState::Hyper) || states.test(fcitx::KeyState::Hyper2)) {
        mods |= IME_MOD_SUPER;
    }
    if (states.test(fcitx::KeyState::Meta)) mods |= IME_MOD_META;
    if (states.test(fcitx::KeyState::CapsLock)) mods |= IME_MOD_CAPS;
    return mods;
}

lc_modifier fcitxModifierKind(fcitx::KeySym sym) {
    switch (sym) {
    case FcitxKey_Control_L:
    case FcitxKey_Control_R:
        return LC_MOD_KEY_CTRL;
    case FcitxKey_Shift_L:
    case FcitxKey_Shift_R:
        return LC_MOD_KEY_SHIFT;
    case FcitxKey_Alt_L:
    case FcitxKey_Alt_R:
    case FcitxKey_Meta_L:
    case FcitxKey_Meta_R:
    case FcitxKey_Super_L:
    case FcitxKey_Super_R:
    case FcitxKey_Hyper_L:
    case FcitxKey_Hyper_R:
    case FcitxKey_Caps_Lock:
    case FcitxKey_Shift_Lock:
    case FcitxKey_Num_Lock:
    case FcitxKey_ISO_Level3_Shift:
    case FcitxKey_ISO_Level5_Shift:
    case FcitxKey_Mode_switch:
        return LC_MOD_KEY_OTHER_MODIFIER;
    default:
        return LC_MOD_NONE;
    }
}

/* ---- Trợ lý ngữ pháp: Candidate word (Mode 1 / AS-060) ---- */
class AssistCandidateWord : public fcitx::CandidateWord {
public:
    AssistCandidateWord(std::string text, std::string comment, std::string replacement,
                        uint32_t deleteGlyphs, std::string originalTail = std::string(),
                        TextVNState *st = nullptr)
        : fcitx::CandidateWord(fcitx::Text(comment.empty() ? text : text + " (" + comment + ")")),
          replacement_(std::move(replacement)),
          deleteGlyphs_(deleteGlyphs),
          originalTail_(std::move(originalTail)),
          st_(st) {}

    void select(fcitx::InputContext *ic) const override {
        if (!ic) return;
        try {
            /* Xoá đúng số ký tự cũ (trước caret) rồi gõ bản sửa. Trả false ⇒
             * không xác minh được ⇒ KHÔNG làm gì cả (fail-safe, R-2/D12). */
            if (!applyAssistCandidate(ic, deleteGlyphs_, originalTail_, replacement_)) {
                return;
            }
            /* R-4: đã thay document ⇒ buffer câu của engine không còn khớp ⇒
             * reset (như Mode 2) để lần trigger sau không so với câu cũ.
             * Chỉ khi KHÔNG còn từ đang gõ (preedit rỗng) — reset giữa lúc
             * đang compose sẽ lệch preedit với engine. */
            if (st_ && st_->inst && st_->comp.len == 0) {
                ime_reset(st_->inst);
            }
            ic->inputPanel().reset();
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        } catch (...) {
        }
    }

private:
    std::string replacement_;
    uint32_t deleteGlyphs_ = 0;
    std::string originalTail_;
    TextVNState *st_ = nullptr;
};

/* ---- State per IC ---- */

TextVNState::TextVNState(fcitx::InputContext *ic_) : ic(ic_) {
    if (ime_instance_new(nullptr, 0, &inst) != IME_OK) {
        inst = nullptr;
    }
    /* Vòng 32 (audit F-A): fresh-install (chưa có config.json) — struct zeroed
     * là OFF, lệch mặc định 3 nền tảng (IBus đặt tường minh trong engine_init).
     * Đặt HIGHLIGHT TRƯỚC sync: file có assist.mode thì sync đè đúng giá trị;
     * thiếu khoá sync cũng trả HIGHLIGHT; không có file thì giữ HIGHLIGHT. */
    config.assist_mode = LC_ASSIST_MODE_HIGHLIGHT;
    lc_config_sync(inst, &config, nullptr);
    if (inst) {
        ime_context_v1 ctx;
        std::memset(&ctx, 0, sizeof(ctx));
        ctx.abi_version = IME_ABI_VERSION;
        ctx.enabled = 1;
        ctx.field_role = IME_FIELD_BODY;
        ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
        ctx.hint = -1;
        ime_set_context(inst, &ctx);
    }
}

TextVNState::~TextVNState() {
    if (inst) ime_instance_free(inst);
}

/* ---- Engine ---- */

/* Thư mục chứa chính libtextvn-fcitx5.so — để tìm textvn-settings của cùng bản cài. */
static std::string selfDir() {
    Dl_info info{};
    if (dladdr(reinterpret_cast<void *>(&selfDir), &info) == 0 || !info.dli_fname) return {};
    std::string path(info.dli_fname);
    const auto slash = path.rfind('/');
    return slash == std::string::npos ? std::string() : path.substr(0, slash);
}

static void launchSettings() {
    char path[1024];
    const std::string dir = selfDir();
    if (lc_find_settings_binary(path, sizeof(path), dir.empty() ? nullptr : dir.c_str()) == 0) {
        fcitx::startProcess({path});
    } else {
        lc_log(LC_LOG_WARN, "Fcitx5", "textvn-settings not found");
    }
}

TextVNEngine::TextVNEngine(fcitx::Instance *instance)
    : instance_(instance),
      factory_([](fcitx::InputContext &ic) { return new TextVNState(&ic); }) {
    lc_log_init("fcitx5-textvn");
    ipc_ = lc_ipc_client_new("fcitx5", nullptr);
    viEnabled_ = lc_state_read_enabled(nullptr, 1) != 0;
    assistEnabled_ = lc_state_read_assist_enabled(nullptr, 1) != 0;
    baseAssistEnabled_ = assistEnabled_;
    instance_->inputContextManager().registerProperty("textvnState", &factory_);

    modeAction_ = std::make_unique<fcitx::SimpleAction>();
    modeAction_->connect<fcitx::SimpleAction::Activated>([this](fcitx::InputContext *ic) {
        try {
            if (auto *st = state(ic)) toggleVietnamese(st);
        } catch (...) {
        }
    });
    instance_->userInterfaceManager().registerAction("textvn-mode", modeAction_.get());

    settingsAction_ = std::make_unique<fcitx::SimpleAction>();
    settingsAction_->setShortText("Cài đặt TextVN…");
    settingsAction_->setIcon("preferences-system");
    settingsAction_->connect<fcitx::SimpleAction::Activated>([](fcitx::InputContext *) {
        try {
            launchSettings();
        } catch (...) {
        }
    });
    instance_->userInterfaceManager().registerAction("textvn-settings", settingsAction_.get());
    updateModeAction(nullptr);
}

TextVNEngine::~TextVNEngine() {
    /* C9 (audit): addon co the bi unload/reloat (fcitx5-remote -r, doi config).
     * Truoc day chi giai phong ipc_/log — con tro action da dang ky van nam
     * trong UserInterfaceManager -> con tro treo khi addon nap lai. Go dang ky
     * TRUOC khi huy unique_ptr. */
    if (instance_) {
        /* unregisterAction() nhan `fcitx::Action*`, KHONG phai ten dang chuoi
         * (xem fcitx::UserInterfaceManager::unregisterAction(Action*)). */
        if (modeAction_) {
            instance_->userInterfaceManager().unregisterAction(modeAction_.get());
        }
        if (settingsAction_) {
            instance_->userInterfaceManager().unregisterAction(settingsAction_.get());
        }
    }
    modeAction_.reset();
    settingsAction_.reset();
    if (ipc_) lc_ipc_client_free(ipc_);
    lc_log_close();
}

TextVNState *TextVNEngine::state(fcitx::InputContext *ic) {
    return ic ? ic->propertyFor(&factory_) : nullptr;
}

void TextVNEngine::showPreedit(TextVNState *st, const uint32_t *text, size_t len) {
    auto &panel = st->ic->inputPanel();
    fcitx::Text preedit;
    if (len > 0) {
        const std::string utf8 = utf32ToUtf8(text, len);
        preedit.append(utf8, fcitx::TextFormatFlag::NoFlag);
        preedit.setCursor(static_cast<int>(utf8.size()));
    }
    /* Client không vẽ được preedit → Fcitx5 hiển thị trong panel của nó. */
    if (st->ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
        panel.setClientPreedit(preedit);
    } else {
        panel.setPreedit(preedit);
    }
    st->ic->updatePreedit();
    st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void TextVNEngine::commitText(TextVNState *st, const uint32_t *text, size_t len) {
    showPreedit(st, nullptr, 0);
    if (len > 0) st->ic->commitString(utf32ToUtf8(text, len));
}

void TextVNEngine::hidePreview(TextVNState *st) {
    if (!previewShowing_) return;
    previewShowing_ = false;
    if (!st || !st->ic || assistUiShowing_) return;
    try {
        st->ic->inputPanel().setAuxDown(fcitx::Text());
        st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    } catch (...) {
    }
}

/* R2-134 (chủ repo: "chữ mờ gợi ý sẵn"): xem trước phần chữ Tab / phím tắt SẼ
 * nối thêm ở dòng phụ của bảng Fcitx5 cạnh con trỏ — không chạm preedit/văn bản. */
void TextVNEngine::updatePreview(TextVNState *st) {
    if (!st || !st->ic || !st->inst || assistUiShowing_ || !viEnabled_) {
        hidePreview(st);
        return;
    }
    char ghost[512];
    char label[540];
    if (ime_peek_completion(st->inst, ghost, sizeof(ghost)) <= 0 ||
        !lc_preview_label(ghost, label, sizeof(label))) {
        hidePreview(st);
        return;
    }
    try {
        st->ic->inputPanel().setAuxDown(fcitx::Text(std::string(label)));
        st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        previewShowing_ = true;
    } catch (...) {
    }
}

/* R2-131 (chủ repo: "Ctrl+Space phải xác định từ lỗi trong câu để sửa hết cùng
 * với text prediction"): câu có lỗi người dùng sửa được ⇒ sửa cả câu (kể cả
 * chính tả), trả true; không thì false (engine dự đoán tiếp như Tab).
 * Precondition D12 kiểm TRƯỚC khi đụng document: từ còn ở preedit chưa nằm
 * trong document ⇒ so surrounding với đuôi câu BỎ phần preedit. */
bool TextVNEngine::hotkeyFixSentence(TextVNState *st) {
    if (!st || !st->ic || !st->inst || !viEnabled_ ||
        !lc_assist_is_active(assistEnabled_ ? 1 : 0, st->config.assist_mode)) {
        return false;
    }
    if (!st->ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) ||
        !st->ic->surroundingText().isValid()) {
        return false;
    }
    ime_assist_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(st->inst, &res) != IME_OK || res.count == 0) return false;
    if (lc_assist_filter_auto_capitalize(&res, lc_assist_auto_capitalize_enabled(&st->config)) == 0) {
        return false;
    }
    char sentence[256 * 4 + 1];
    size_t actualLen = 0;
    if (ime_assist_sentence_tail(st->inst, 256, sentence, sizeof(sentence), &actualLen) != IME_OK ||
        actualLen == 0 || actualLen >= sizeof(sentence)) {
        return false;
    }
    sentence[actualLen] = 0;
    lc_sentence_fix_plan plan;
    if (!lc_plan_sentence_fix_ex(st->inst, sentence, &res, 1, &plan) || plan.delete_glyphs == 0) {
        return false;
    }
    const std::string pending = st->comp.len > 0 ? utf32ToUtf8(st->comp.text, st->comp.len) : std::string();
    char docTail[LC_MAX_SENTENCE_FIX_CHARS];
    if (!lc_strip_suffix(plan.original_tail, pending.c_str(), docTail, sizeof(docTail))) return false;
    if (docTail[0] != 0 && !textBeforeCaretMatches(st->ic, docTail)) return false;

    try {
        if (st->comp.len > 0) {
            commitText(st, st->comp.text, st->comp.len);
            st->comp.len = 0;
        }
        clearAssistUI(st);
        deleteBeforeCaret(st->ic, plan.delete_glyphs);
        st->ic->commitString(plan.replacement);
    } catch (...) {
        ime_reset(st->inst);
        return true; /* document đã bị chạm — không để engine xử lý tiếp phím này */
    }
    size_t replGlyphs = 0;
    if (ime_assist_glyph_len(st->inst, plan.replacement, std::strlen(plan.replacement), &replGlyphs) ==
            IME_OK &&
        replGlyphs > 0) {
        st->assistUndoValid = true;
        st->assistUndoOriginalTail = plan.original_tail;
        st->assistUndoReplacement = plan.replacement;
        st->assistUndoReplacementGlyphs = static_cast<uint32_t>(replGlyphs);
        st->assistUndoTimestampMs = nowMs();
    }
    char fixed[256 * 4 + LC_MAX_SENTENCE_FIX_CHARS];
    if (lc_fixed_sentence(sentence, plan.original_tail, plan.replacement, fixed, sizeof(fixed))) {
        ime_reset_with_tail(st->inst, fixed, std::strlen(fixed));
    } else {
        ime_reset(st->inst);
    }
    return true;
}

void TextVNEngine::finishWord(TextVNState *st) {
    if (!st) return;
    hidePreview(st);
    clearAssistUI(st);
    /* R-4: chốt từ ⇒ trạng thái hoàn tác cũng hết hiệu lực (caret có thể đã
     * đổi ⇒ không xác minh được vùng cần xoá). */
    st->assistUndoValid = false;
    if (st->comp.len > 0) {
        commitText(st, st->comp.text, st->comp.len);
        st->comp.len = 0;
    }
    if (st->inst) ime_reset(st->inst);
}

void TextVNEngine::updateModeAction(fcitx::InputContext *ic) {
    if (!modeAction_) return;
    modeAction_->setShortText(viEnabled_ ? "Tiếng Việt (bấm để tắt)" : "Tiếng Anh (bấm để bật)");
    modeAction_->setLongText("Bật/tắt tiếng Việt: Ctrl+Shift hoặc Ctrl+Shift+Space");
    modeAction_->setIcon(viEnabled_ ? "textvn_v" : "textvn_e");
    if (ic) modeAction_->update(ic);
}

void TextVNEngine::setVietnamese(TextVNState *st, bool on, bool persist) {
    if (on == viEnabled_) return;
    finishWord(st);
    viEnabled_ = on;
    if (persist) {
        /* R2-107: phím chuyển/menu là công tắc TOÀN CỤC → cập nhật cả base (C4). Lần
         * ghi state.json dưới đây bị lc_state_sync "hấp thụ" nên syncState không bao
         * giờ thấy nó; thiếu dòng này, lần activate kế tiếp gán viEnabled_ = base CŨ —
         * đổi V/E rồi chuyển cửa sổ là bị lật lại. */
        baseViEnabled_ = on;
        lc_state_write_enabled(nullptr, on ? 1 : 0);
        int ignored = on ? 1 : 0;
        lc_state_sync(&stateWatch_, nullptr, &ignored); /* hấp thụ lần ghi của chính mình */
    }
    if (ipc_) lc_ipc_client_toggle_vi_en(ipc_, "*", on ? 1 : 0);
    if (st && st->ic) {
        updateModeAction(st->ic);
        st->ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        instance_->showInputMethodInformation(st->ic);
    }
}

/* Vong 13 (BUG-03): tieu thu broadcast toan cuc "*" cua tray — ap dung dung
 * MOT lan moi su kien (so sanh seq, khong sticky de tranh xung dot voi kenh
 * state.json). Poll chi chay tai focus/activate, khong tren duong phim. */
void TextVNEngine::consumeGlobalIpc() {
    if (!ipc_) return;
    lc_ipc_client_poll(ipc_);
    int g = 0;
    uint32_t seq = 0;
    if (lc_ipc_client_get_global_override(ipc_, &g, &seq) && seq != globalSeqSeen_) {
        globalSeqSeen_ = seq;
        baseViEnabled_ = g != 0; /* C4: broadcast "*" = base */
        viEnabled_ = g != 0;
    }
}

void TextVNEngine::toggleVietnamese(TextVNState *st) { setVietnamese(st, !viEnabled_, true); }

/* Bảng cài đặt vừa đổi state.json → theo ngay (một stat mỗi lần gọi). */
void TextVNEngine::syncState(TextVNState *st) {
    int enabled = viEnabled_ ? 1 : 0;
    if (lc_state_sync(&stateWatch_, nullptr, &enabled)) {
        baseViEnabled_ = enabled != 0; /* C4: file = nguon cua base */
        setVietnamese(st, enabled != 0, false);
    }
}

void TextVNEngine::clearAssistUI(TextVNState *st) {
    if (!assistUiShowing_) return;
    assistUiShowing_ = false;
    if (!st || !st->ic) return;
    try {
        st->ic->inputPanel().reset();
        st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    } catch (...) {
    }
}

void TextVNEngine::consumeAssistIpc() {
    if (!ipc_) return;
    int a = 0;
    uint32_t seq = 0;
    if (lc_ipc_client_get_assist_override(ipc_, &a, &seq) && seq != assistSeqSeen_) {
        assistSeqSeen_ = seq;
        baseAssistEnabled_ = (a != 0);
        assistEnabled_ = (a != 0);
    }
}

void TextVNEngine::syncAssistState(TextVNState *) {
    int enabled = assistEnabled_ ? 1 : 0;
    if (lc_state_sync_assist(&assistStateWatch_, nullptr, &enabled)) {
        baseAssistEnabled_ = (enabled != 0);
        assistEnabled_ = (enabled != 0);
    }
}

bool TextVNEngine::checkAndApplyAssist(TextVNState *st, uint32_t triggerChar,
                                      bool isSentenceTrigger, bool planEaten) {
    if (!st || !st->inst || !st->ic) return planEaten;
    const bool dbg = lc_assist_debug_enabled() != 0;
    if (!viEnabled_ || !lc_assist_is_active(assistEnabled_ ? 1 : 0, st->config.assist_mode)) {
        if (dbg) {
            lc_log(LC_LOG_INFO, "Fcitx5", "assist skip: vi=%d state=%d mode=%d",
                              viEnabled_ ? 1 : 0, assistEnabled_ ? 1 : 0,
                              st->config.assist_mode);
        }
        clearAssistUI(st);
        return planEaten;
    }

    ime_assist_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(st->inst, &res) != IME_OK || res.count == 0) {
        /* R2-129: câu đã đúng ⇒ ẩn danh sách sống ngay (parity Windows/IBus). */
        if (dbg) lc_log(LC_LOG_INFO, "Fcitx5", "assist: engine im lặng (count=0)");
        clearAssistUI(st);
        return planEaten;
    }

    /* D16 (Fassist-20) — Q-6: crate `assist` luôn trả L6 ("chữ thường đầu
     * câu"); khi `config.auto_capitalize` bật thì IME đã tự viết hoa ⇒ báo/sửa
     * L6 là trùng chức năng. Lọc **sau** khi engine đã check (giữ nguyên các
     * luật khác) và **trước** cả Mode 2 lẫn Mode 1 ⇒ cả hai đều không thấy L6.
     * Cùng hàm + cùng đường config với IBus (`lc_config_sync` → `st->config`). */
    if (lc_assist_filter_auto_capitalize(&res, lc_assist_auto_capitalize_enabled(&st->config)) == 0) {
        clearAssistUI(st);
        return planEaten;
    }

    /* R-5 (P4-0 §4.3): Mode 2 = mode "autocorrect" VÀ auto_fix "safe" VÀ
     * trigger **kết thúc câu** — cùng ngữ nghĩa cho cả 3 OS. */
    const bool isAutoCorrectMode =
        lc_assist_should_auto_fix(st->config.assist_mode, st->config.assist_auto_fix,
                                  isSentenceTrigger ? 1 : 0) != 0;

    /* Buffer đủ cho 256 ký tự Unicode (cap của core) × 4 byte UTF-8 + NUL.
     * Buffer nhỏ hơn ⇒ `ime_assist_sentence_tail` cắt cụt (có thể cắt giữa
     * ký tự UTF-8) ⇒ câu sai ⇒ span lệch ⇒ kế hoạch xoá nhầm (fail-safe: cần
     * buffer đủ). */
    char sentence[256 * 4 + 1];
    size_t actual_len = 0;
    if (ime_assist_sentence_tail(st->inst, 256, sentence, sizeof(sentence), &actual_len) != IME_OK ||
        actual_len == 0 || actual_len >= sizeof(sentence)) {
        return planEaten;
    }
    sentence[actual_len] = '\0';

    /* Vòng 32 (audit F-G): parity Windows D11 — câu < 3 từ không popup/không
     * tự sửa (đoạn ngắn đang gõ dở, thiếu ngữ cảnh; Windows chặn ở
     * `check_text` còn 2 nền tảng còn lại phải tự chặn như nhau). Đếm từ
     * trên đuôi đã đọc: byte >= 0x80 là lead/continuation UTF-8 = chữ có dấu. */
    {
        int words = 0;
        bool inWord = false;
        for (size_t wi = 0; wi < actual_len; wi++) {
            const unsigned char c = static_cast<unsigned char>(sentence[wi]);
            const bool isWordC = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c >= 0x80;
            if (isWordC) {
                if (!inWord) {
                    words++;
                    inWord = true;
                }
            } else {
                inWord = false;
            }
        }
        if (words < 3) {
            if (dbg) lc_log(LC_LOG_INFO, "Fcitx5", "assist skip: câu < 3 từ (D11)");
            clearAssistUI(st);
            return planEaten;
        }
    }

    if (isAutoCorrectMode) {
        const bool surrounding =
            st->ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
            st->ic->surroundingText().isValid();
        if (surrounding) {
            lc_sentence_fix_plan fixPlan;
            if (lc_plan_sentence_auto_fix(st->inst, sentence, &res, &fixPlan) && fixPlan.fixed_count > 0) {
                /* R-2 (D12) + THỨ TỰ đúng (giống hệt bản IBus đã sửa):
                 *   1) tính đuôi document PHẢI có — nếu ký tự trigger CHƯA nằm
                 *      trong document (adapter sẽ tự commit) thì đuôi document =
                 *      đuôi engine BỎ ký tự trigger (buffer câu của core luôn
                 *      kết thúc bằng ký tự trigger — xem `lc_assist_text.h`);
                 *   2) kiểm precondition **TRƯỚC** khi commit ký tự trigger;
                 *   3) mới commit trigger → xoá đúng số glyph → gõ lại bản sửa.
                 *
                 * Sai thứ tự (commit trigger TRƯỚC rồi mới kiểm) ⇒
                 * (a) điều kiện không khớp ⇒ Mode 2 không bao giờ chạy,
                 * (b) bỏ sửa nhưng đã commit trigger ⇒ app nhận thêm ký tự
                 *     (NHÂN ĐÔI ký tự). */
                char expected[LC_MAX_SENTENCE_FIX_CHARS];
                if (!lc_assist_expected_document_tail(fixPlan.original_tail, triggerChar,
                                                       planEaten ? 1 : 0, expected,
                                                       sizeof(expected))) {
                    return planEaten; /* không xác minh được ⇒ KHÔNG sửa */
                }
                if (!textBeforeCaretMatches(st->ic, expected)) {
                    return planEaten; /* lệch ⇒ KHÔNG sửa (fail-safe, D12) */
                }
                /* Precondition đã khớp: commit ký tự trigger (nếu chưa có trong
                 * document) để document khớp buffer câu của engine. Số glyph
                 * xoá (`delete_glyphs`) tính cho CẢ đuôi engine (đã có trigger)
                 * ⇒ xoá sau khi commit trigger là đúng số ký tự. */
                if (!planEaten && triggerChar != 0) {
                    st->ic->commitString(utf32ToUtf8(&triggerChar, 1));
                }
                /* Chỉ xoá **trước** caret (sau caret không thuộc phạm vi sửa —
                 * xoá cả 2 phía ⇒ mất dữ liệu người dùng). Số **glyph** do
                 * core tính (`ime_assist_glyph_len` trong
                 * `lc_plan_sentence_auto_fix`) — KHÔNG fallback `chars().count()`
                 * (sai ở VNI/TCVN3 ⇒ xoá lệch vị trí).
                 *
                 * QUY ƯỚC API Fcitx5: tham số 2 là **SỐ KÝ TỰ CẦN XOÁ** (không
                 * phải 0!) — xem `deleteBeforeCaret`. Gọi `(-n, 0)` ⇒ xoá 0 ký tự
                 * ⇒ đuôi câu cũ còn nguyên ⇒ commit bản sửa ⇒ **nhân đôi câu**. */
                if (fixPlan.delete_glyphs > 0) {
                    deleteBeforeCaret(st->ic, fixPlan.delete_glyphs);
                }
                st->ic->commitString(fixPlan.replacement);
                /* R-4: sau khi đã thay document, buffer engine phải sạch —
                 * giữ câu cũ ⇒ mỗi trigger lại xoá+commit lại (lặp, nhân bản
                 * history trong app). */
                if (st->inst) ime_reset(st->inst);
                /* R-4: lưu state hoàn tác 1 bước (Ctrl+Z trong 5 s).
                 * Số glyph cần xoá khi hoàn tác là của **replacement** (text đã
                 * gõ lại), không phải của đuôi câu gốc. Không đếm được ⇒ không
                 * lưu state (fail-safe: bỏ undo còn hơn xoá lệch vị trí). */
                size_t replGlyphs = 0;
                if (ime_assist_glyph_len(st->inst, fixPlan.replacement,
                                         strlen(fixPlan.replacement), &replGlyphs) == IME_OK &&
                    replGlyphs > 0) {
                    st->assistUndoValid = true;
                    st->assistUndoOriginalTail = fixPlan.original_tail;
                    st->assistUndoReplacement = fixPlan.replacement;
                    st->assistUndoReplacementGlyphs = static_cast<uint32_t>(replGlyphs);
                    st->assistUndoTimestampMs = nowMs();
                }
                if (ipc_) {
                    lc_ipc_client_send_assist_report(ipc_, &res, fixPlan.fixed_count);
                }
                return true; /* Consumed: replacement already contains trigger/fix */
            }
        }
    }

    /* Mode 1: highlight / suggestions (CandidateList & tooltip).
     * R2-129: danh sách SỐNG — hiện/cập nhật ở mọi điểm kiểm tra (ranh giới từ
     * lẫn kết thúc câu), ẩn khi câu sạch; không chiếm phím gõ (handleKey). */
    if (lc_assist_should_display(st->config.assist_mode, isSentenceTrigger ? 1 : 0)) {
        try {
            auto candList = std::make_unique<fcitx::CommonCandidateList>();
            candList->setLayoutHint(fcitx::CandidateLayoutHint::Vertical);
            size_t added = 0;

            for (uint32_t i = 0; i < res.count && added < 3; ++i) {
                const auto &issue = res.issues[i];
                if (issue.suggest[0] == '\0') continue;

                /* Single issue plan for selection fix */
                ime_assist_result_v1 single;
                std::memset(&single, 0, sizeof(single));
                single.abi_version = IME_ABI_VERSION;
                single.count = 1;
                single.issues[0] = issue;

                lc_sentence_fix_plan singlePlan;
                /* Không lập được kế hoạch sửa ⇒ KHÔNG hiện ứng viên: nếu hiện,
                 * chọn sẽ **chèn** gợi ý vào caret thay vì thay lỗi (sai).
                 * Cùng quy tắc với IBus (`apply_assist_candidate`) và macOS
                 * (`buildCandidates`). */
                /* R2-131: chọn gợi ý = hành động CHỦ ĐỘNG ⇒ gồm cả lỗi chính tả. */
                if (!lc_plan_sentence_fix_ex(st->inst, sentence, &single, 1, &singlePlan)) {
                    continue;
                }
                candList->append(std::make_unique<AssistCandidateWord>(
                    issue.suggest, issue.message, singlePlan.replacement,
                    singlePlan.delete_glyphs, singlePlan.original_tail, st));
                added++;
            }

            if (added > 0) {
                auto &panel = st->ic->inputPanel();
                panel.setAuxDown(fcitx::Text(std::string("Trợ lý TextVN: ") + res.issues[0].message));
                panel.setCandidateList(std::move(candList));
                st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                assistUiShowing_ = true;
                if (ipc_) {
                    lc_ipc_client_send_assist_report(ipc_, &res, 0);
                }
            } else {
                clearAssistUI(st); /* không còn gợi ý áp được ⇒ không để danh sách cũ */
            }
        } catch (...) {
        }
    }

    return planEaten;
}

/* R-4 (AS-050): hoàn tác tự sửa cả câu — 1 bước, hạn 5 s.
 *
 * Fail-safe: trước khi xoá, vùng text trước caret phải **đúng** là text đã
 * gõ lại sau khi sửa (nếu người dùng đã gõ thêm ⇒ không xoá nhầm chữ). */
bool TextVNEngine::applyAssistUndo(TextVNState *st) {
    if (!st || !st->ic || !st->inst || !st->assistUndoValid) return false;
    if (nowMs() - st->assistUndoTimestampMs > 5000) {
        st->assistUndoValid = false; /* hết hạn ⇒ để Ctrl+Z tới app */
        return false;
    }
    if (!textBeforeCaretMatches(st->ic, st->assistUndoReplacement)) {
        return false; /* không xác minh được ⇒ KHÔNG xoá */
    }
    try {
        /* Hoàn tác: xoá đúng số ký tự **đã gõ lại** (trước caret) rồi gõ lại
         * đuôi câu gốc. Cùng quy ước API Fcitx5: tham số 2 = SỐ KÝ TỰ CẦN
         * XOÁ (xem `deleteBeforeCaret`) — KHÔNG phải "số ký tự sau caret" và
         * KHÔNG phải 0 (0 ⇒ xoá 0 ký tự ⇒ hoàn tác không xoá gì ⇒ câu bị nhân
         * đôi khi gõ lại bản sửa). */
        if (st->assistUndoReplacementGlyphs > 0) {
            deleteBeforeCaret(st->ic, st->assistUndoReplacementGlyphs);
        }
        st->ic->commitString(st->assistUndoOriginalTail);
        st->assistUndoValid = false;
        ime_reset(st->inst);
        return true;
    } catch (...) {
        return false;
    }
}

std::string TextVNEngine::subModeLabelImpl(const fcitx::InputMethodEntry &,
                                           fcitx::InputContext &) {
    return viEnabled_ ? "V" : "E";
}
std::string TextVNEngine::subModeIconImpl(const fcitx::InputMethodEntry &,
                                          fcitx::InputContext &) {
    return viEnabled_ ? "textvn_v" : "textvn_e";
}

void TextVNEngine::activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    try {
        auto *ic = event.inputContext();
        auto *st = state(ic);
        if (!st) return;
        lc_config_sync(st->inst, &st->config, nullptr);
        syncState(st);
        consumeGlobalIpc();
        syncAssistState(st);
        consumeAssistIpc();
        int appEnabled = 1;
        if (ipc_ && lc_ipc_client_get_app_override(ipc_, &appEnabled)) {
            /* Vòng 32 (audit F-B, R2-28): TOÀN CỤC AND override — khớp Windows
             * `effective_enabled`: override chỉ TẮT riêng được app, không "hồi
             * sinh" app khi toàn cục đang EN (kill-switch là master). */
            viEnabled_ = baseViEnabled_ && (appEnabled != 0);
        } else {
            /* C4 (audit): override da bi clear - ve ve BASE (state.json/broadcast
             * "*"), khong de ke thua che do cua app truoc. */
            viEnabled_ = baseViEnabled_;
        }
        updateModeAction(ic);
        ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, modeAction_.get());
        ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, settingsAction_.get());
    } catch (...) {
    }
}

/* FOCUS-OUT: client không có CapabilityFlag::ClientUnfocusCommit thì Fcitx5 core đã
 * commit client preedit TRƯỚC khi gọi engine — engine chỉ xóa panel (nếu để nguyên,
 * Reset kế tiếp core commit lần nữa = chữ lặp) và quên từ. Client có cờ đó tự commit.
 * Reset / đổi bộ gõ: core không commit → engine commit (không mất chữ — B2).
 * Cả hai nhánh kiểm chứng bằng tests/e2e_fcitx5.py với fcitx5 thật. */
void TextVNEngine::endWord(TextVNState *st, const fcitx::InputContextEvent &event) {
    if (!st) return;
    clearAssistUI(st);
    /* R-4: đổi focus ⇒ trạng thái hoàn tác hết hiệu lực. */
    st->assistUndoValid = false;
    if (event.type() == fcitx::EventType::InputContextFocusOut) {
        if (st->comp.len > 0) showPreedit(st, nullptr, 0);
        st->comp.len = 0;
        if (st->inst) ime_reset(st->inst);
    } else {
        finishWord(st);
    }
    lc_modifier_toggle_reset(&st->toggle);
}

void TextVNEngine::deactivate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    try {
        endWord(state(event.inputContext()), event);
    } catch (...) {
    }
}

void TextVNEngine::reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    try {
        endWord(state(event.inputContext()), event);
    } catch (...) {
    }
}

bool TextVNEngine::handleKey(TextVNState *st, const fcitx::Key &key, fcitx::KeyStates rawStates,
                             bool isRelease) {
    const fcitx::KeyStates states = key.states();
    const lc_modifier which = fcitxModifierKind(key.sym());

    if (isRelease) {
        if (lc_modifier_toggle_up(&st->toggle, which)) toggleVietnamese(st);
        return false;
    }
    lc_modifier_toggle_down(&st->toggle, which, states.test(fcitx::KeyState::Ctrl),
                            states.test(fcitx::KeyState::Shift),
                            states.test(fcitx::KeyState::Alt) ||
                                states.test(fcitx::KeyState::Super));
    if (which != LC_MOD_NONE) return false;
    /* R2-134: bản xem trước chỉ sống tới phím kế tiếp (vẽ lại sau phím gõ). */
    hidePreview(st);

    /* Phím khi danh sách gợi ý SỐNG đang hiện (R2-129 — parity Windows/IBus):
     * không chiếm phím gõ; Esc đóng; Alt+1..3 áp gợi ý (trước đây phím số trần
     * trả `false` cho app — vừa gõ chữ số vừa KHÔNG chọn được gợi ý, vì Fcitx5
     * không tự chọn ứng viên thay engine); Enter/Tab/mũi tên/chord đóng rồi đi tiếp. */
    if (!isRelease && assistUiShowing_) {
        int live = lc_assist_live_key(static_cast<uint32_t>(key.sym()),
                                      states.test(fcitx::KeyState::Ctrl),
                                      states.test(fcitx::KeyState::Alt),
                                      states.test(fcitx::KeyState::Super));
        if (live == LC_LIVE_CLOSE) {
            clearAssistUI(st);
            return true;
        }
        if (live >= LC_LIVE_APPLY_1) {
            const int idx = live - LC_LIVE_APPLY_1;
            auto list = st->ic->inputPanel().candidateList();
            if (list && idx < list->size()) {
                list->candidate(idx).select(st->ic);
                assistUiShowing_ = false; /* select() đã reset input panel */
                return true;
            }
            live = LC_LIVE_DISMISS;
        }
        if (live == LC_LIVE_DISMISS) clearAssistUI(st);
    }

    uint32_t mods = fcitxMods(states);
    /* Caps Lock bật: engine coi phím dấu Telex viết hoa là phím dấu (VIEETJ → VIỆT). */
    if (rawStates.test(fcitx::KeyState::CapsLock)) mods |= IME_MOD_CAPS;

    /* R-4 (AS-050): Ctrl+Z hoàn tác tự sửa cả câu (1 bước, hạn 5 s).
     * R-3: mọi phím KHÁC ⇒ huỷ state hoàn tác (gõ tiếp không được bị Ctrl+Z
     * sau đó xoá nhầm chữ vừa gõ). */
    const bool isAssistUndoChord =
        states.test(fcitx::KeyState::Ctrl) && !states.test(fcitx::KeyState::Alt) &&
        !states.test(fcitx::KeyState::Super) && !states.test(fcitx::KeyState::Shift) &&
        key.sym() == FcitxKey_z;
    if (isAssistUndoChord) {
        if (applyAssistUndo(st)) {
            return true;
        }
    } else {
        st->assistUndoValid = false;
    }

    /* Ctrl+Shift+Space (ADR-011). */
    if ((mods & (IME_MOD_CTRL | IME_MOD_SHIFT)) == (IME_MOD_CTRL | IME_MOD_SHIFT) &&
        (key.sym() == FcitxKey_space || key.sym() == FcitxKey_KP_Space)) {
        // Ctrl+Shift was already armed before Space arrived. This shortcut
        // toggles immediately, so prevent the two subsequent releases from
        // interpreting the same chord as the Ctrl+Shift toggle again.
        lc_modifier_toggle_reset(&st->toggle);
        toggleVietnamese(st);
        return true;
    }

    /* OP-007 (P5 §7.4) · R2-139: Ctrl+Shift+F9 — chuyển mã vùng bôi đen.
     * Chạy TRƯỚC nhánh chord hệ thống (parity Windows key_event.rs §613-638);
     * chốt preedit trước khi đổi selection (như `end_composition`). Chỉ nuốt
     * khi đã chạy đường convert tới cùng; còn lại phím F9 tới app. */
    if (lc_is_transcode_hotkey(static_cast<uint32_t>(key.sym()),
                               states.test(fcitx::KeyState::Ctrl),
                               states.test(fcitx::KeyState::Alt),
                               states.test(fcitx::KeyState::Shift),
                               states.test(fcitx::KeyState::Super))) {
        if (st->comp.len > 0) finishWord(st);
        if (performSelectionConvert(st)) return true;
        return false;
    }

    if (st->comp.len == 0) {
        lc_config_sync(st->inst, &st->config, nullptr);
        syncState(st);
        consumeGlobalIpc();
        syncAssistState(st);
        consumeAssistIpc();
    }

    /* B6 chord; S3 ô mật khẩu; VN tắt (trừ khi còn gõ tắt); engine hỏng → phím đi thẳng.
     * Ngoại lệ: Ctrl+Space được giữ lại cho Text Prediction & Text Correct. */
    uint32_t vk = 0, ch = 0;
    mapFcitxKey(key, &vk, &ch);
    /* R2-131/R2-136: phím tắt trợ lý = Ctrl+Space (Alt+Space không thuộc TextVN). */
    const bool is_ctrl_space =
        lc_is_assist_hotkey(static_cast<uint32_t>(key.sym()), states.test(fcitx::KeyState::Ctrl),
                            states.test(fcitx::KeyState::Alt), states.test(fcitx::KeyState::Shift),
                            states.test(fcitx::KeyState::Super)) != 0;

    const bool secure =
        st->ic->capabilityFlags().testAny(fcitx::CapabilityFlag::PasswordOrSensitive);
    const bool macroOnly = !viEnabled_ && st->config.allow_macro_when_vi_off;
    if (((mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) && !is_ctrl_space) || secure ||
        !(viEnabled_ || macroOnly) || !st->inst) {
        finishWord(st);
        return false;
    }
    if (st->ctxEnabled != (viEnabled_ ? 1 : 0)) {
        ime_context_v1 ctx;
        std::memset(&ctx, 0, sizeof(ctx));
        ctx.abi_version = IME_ABI_VERSION;
        ctx.enabled = viEnabled_ ? 1 : 0;
        ctx.field_role = IME_FIELD_BODY;
        ctx.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
        ctx.hint = -1;
        ime_set_context(st->inst, &ctx);
        st->ctxEnabled = ctx.enabled;
    }

    /* R2-131: phím tắt — câu có lỗi sửa được ⇒ sửa hết TRƯỚC; không ⇒ dự đoán. */
    if (is_ctrl_space && hotkeyFixSentence(st)) return true;
    if (is_ctrl_space) {
        vk = LC_VK_SPACE;
        ch = ' ';
        mods = (mods & IME_MOD_CAPS) | IME_MOD_CTRL;
    }

    const lc_key k = lc_key_classify_with_mods(vk, ch, mods);
    ime_key_v1 ik;
    uint32_t ime_mods = mods & (IME_MOD_SHIFT | IME_MOD_CAPS);
    if (is_ctrl_space) {
        ime_mods |= IME_MOD_CTRL;
    }
    lc_key_to_ime(k, vk, ime_mods, &ik);

    ime_result_v1 r;
    if (ime_key(st->inst, &ik, &r) != IME_OK || (r.flags & IME_FLAG_ERROR)) {
        finishWord(st);
        return false;
    }

    lc_plan plan;
    lc_plan_key(&st->comp, k, &r, &plan);

    if (plan.delete_before > 0) {
        const bool surrounding =
            st->ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
            st->ic->surroundingText().isValid();
        if (!surrounding) {
            finishWord(st);
            return false;
        }
        /* Xoá `delete_before` ký tự TRƯỚC caret — tham số 2 của API Fcitx5 là
         * SỐ KÝ TỰ CẦN XOÁ (xem `deleteBeforeCaret`). */
        deleteBeforeCaret(st->ic, plan.delete_before);
    }

    if (plan.end) {
        size_t n = 0;
        const uint32_t *t = lc_plan_commit_text(&st->comp, &plan, &n);
        commitText(st, t, n);
    } else if (plan.has_text) {
        showPreedit(st, plan.text, plan.text_len);
    }
    lc_comp_apply(&st->comp, &plan);
    if (plan.reset_engine) ime_reset(st->inst);

    /* Trợ lý ngữ pháp (AS-060 / P4-6): trigger kiểm tra khi commit kết thúc câu hoặc dấu cách */
    const uint32_t triggerKind = (k.kind == LC_KEY_CHAR) ? ime_assist_trigger_kind(k.ch) : IME_ASSIST_TRIGGER_NONE;
    if (triggerKind != IME_ASSIST_TRIGGER_NONE && viEnabled_ && st->comp.len == 0 && st->inst) {
        bool isSentence = (triggerKind == IME_ASSIST_TRIGGER_SENTENCE);
        const bool eaten = checkAndApplyAssist(st, k.ch, isSentence, plan.eaten != 0);
        updatePreview(st);
        return eaten;
    }
    /* R2-129: Backspace khi danh sách sống đang hiện ⇒ kiểm tra lại câu (sửa xong
     * thì ẩn ngay, còn lỗi thì cập nhật). */
    if (k.kind == LC_KEY_BACKSPACE && assistUiShowing_ && viEnabled_ && st->comp.len == 0 &&
        st->inst) {
        const bool eaten = checkAndApplyAssist(st, 0, false, plan.eaten != 0);
        updatePreview(st);
        return eaten;
    }

    if (k.kind == LC_KEY_CHAR || k.kind == LC_KEY_BACKSPACE || k.kind == LC_KEY_TAB) {
        updatePreview(st);
    }
    return plan.eaten != 0;
}

void TextVNEngine::keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &keyEvent) {
    try {
        auto *st = state(keyEvent.inputContext());
        if (st && handleKey(st, keyEvent.key(), keyEvent.rawKey().states(),
                            keyEvent.isRelease())) {
            keyEvent.filterAndAccept();
        }
    } catch (...) {
        /* S4: fail-open — không bao giờ làm sập tiến trình fcitx5. */
    }
}

fcitx::AddonInstance *TextVNEngineFactory::create(fcitx::AddonManager *manager) {
    return new TextVNEngine(manager->instance());
}

} // namespace textvn
