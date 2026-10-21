/* engine.cpp — Engine Fcitx5 của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Cùng mô hình với IBus (lc_compose.h): cả từ trong preedit, commit ở ranh giới.
 * Mọi đường lỗi fail-open: không bao giờ ném exception ra khỏi callback của Fcitx5.
 */

#include "engine.h"

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
 * engine biết. Không đọc được (client không hỗ trợ surrounding text) hoặc lệch
 * ⇒ trả false ⇒ **không sửa** (đây là chốt chặn "xoá nhầm text lạ" khi engine
 * buffer stale: user bấm chuột đặt caret chỗ khác rồi gõ `.`).
 *
 * `expected` là Unicode (buffer của core); document chứa glyph theo bảng mã
 * xuất — với `unicode_precomposed` (mặc định) 1 char = 1 byte UTF-8-ish nên
 * so được; bảng mã khác ⇒ lệch ⇒ không sửa (an toàn hơn sửa nhầm). */
static bool textBeforeCaretMatches(fcitx::InputContext *ic, const std::string &expected) {
    if (!ic || expected.empty()) return false;
    fcitx::SurroundingText surrounding = ic->surroundingText();
    if (!surrounding.isValid()) return false;
    std::string text = surrounding.text();
    const int cursor = surrounding.cursor();
    if (cursor >= 0 && static_cast<size_t>(cursor) <= text.size()) {
        text = text.substr(0, static_cast<size_t>(cursor));
    }
    if (text.size() < expected.size()) return false;
    return text.compare(text.size() - expected.size(), expected.size(), expected) == 0;
}

std::string utf32ToUtf8(const uint32_t *text, size_t len) {
    std::string out;
    for (size_t i = 0; i < len; ++i) {
        out += fcitx::utf8::UCS4ToUTF8(text[i]);
    }
    return out;
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
                        uint32_t deleteGlyphs, std::string originalTail = std::string())
        : fcitx::CandidateWord(fcitx::Text(comment.empty() ? text : text + " (" + comment + ")")),
          replacement_(std::move(replacement)),
          deleteGlyphs_(deleteGlyphs),
          originalTail_(std::move(originalTail)) {}

    void select(fcitx::InputContext *ic) const override {
        if (!ic) return;
        try {
            if (deleteGlyphs_ > 0) {
                if (!ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) ||
                    !ic->surroundingText().isValid()) {
                    return;
                }
                /* R-2: chỉ xoá khi vùng trước caret **đúng** là đuôi câu engine
                 * biết (caret ở chỗ khác ⇒ không xoá nhầm). */
                if (!originalTail_.empty() && !textBeforeCaretMatches(ic, originalTail_)) {
                    return;
                }
                /* Chỉ xoá **trước** caret: phần sau caret không thuộc phạm vi
                 * sửa (xoá cả 2 phía ⇒ mất dữ liệu người dùng). */
                ic->deleteSurroundingText(-static_cast<int>(deleteGlyphs_), 0);
            }
            if (!replacement_.empty()) {
                ic->commitString(replacement_);
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
};

/* ---- State per IC ---- */

TextVNState::TextVNState(fcitx::InputContext *ic_) : ic(ic_) {
    if (ime_instance_new(nullptr, 0, &inst) != IME_OK) {
        inst = nullptr;
    }
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
    assistEnabled_ = lc_state_read_assist_enabled(nullptr, 0) != 0;
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

void TextVNEngine::finishWord(TextVNState *st) {
    if (!st) return;
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
    if (!viEnabled_ || !lc_assist_is_active(assistEnabled_ ? 1 : 0, st->config.assist_mode)) {
        return planEaten;
    }

    ime_assist_result_v1 res;
    std::memset(&res, 0, sizeof(res));
    res.abi_version = IME_ABI_VERSION;
    if (ime_assist_check_instance(st->inst, &res) != IME_OK || res.count == 0) {
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

    if (isAutoCorrectMode) {
        const bool surrounding =
            st->ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText) &&
            st->ic->surroundingText().isValid();
        if (surrounding) {
            lc_sentence_fix_plan fixPlan;
            if (lc_plan_sentence_auto_fix(st->inst, sentence, &res, &fixPlan) && fixPlan.fixed_count > 0) {
                /* If trigger char was not eaten, commit it first so document tail matches sentence buffer */
                if (!planEaten && triggerChar != 0) {
                    st->ic->commitString(utf32ToUtf8(&triggerChar, 1));
                }
                /* R-2: chỉ xoá khi vùng trước caret **đúng** là đuôi câu engine
                 * biết; không xác minh được ⇒ KHÔNG sửa (fail-safe, D12). */
                if (!textBeforeCaretMatches(st->ic, fixPlan.original_tail)) {
                    return planEaten;
                }
                if (fixPlan.delete_glyphs > 0) {
                    /* Chỉ xoá **trước** caret (sau caret không thuộc phạm vi sửa). */
                    st->ic->deleteSurroundingText(-static_cast<int>(fixPlan.delete_glyphs), 0);
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
                    lc_ipc_client_send_assist_report(ipc_, res.count, fixPlan.fixed_count);
                }
                return true; /* Consumed: replacement already contains trigger/fix */
            }
        }
    }

    /* Mode 1: highlight / suggestions (CandidateList & tooltip).
     * R-6 (D11): chỉ hiện ở **điểm hiển thị** (kết thúc câu) — mỗi dấu cách lại
     * tung panel là UI spam. */
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
                uint32_t delGlyphs = 0;
                std::string rep = issue.suggest;
                std::string origTail;
                if (lc_plan_sentence_auto_fix(st->inst, sentence, &single, &singlePlan)) {
                    delGlyphs = singlePlan.delete_glyphs;
                    rep = singlePlan.replacement;
                    origTail = singlePlan.original_tail;
                }

                candList->append(std::make_unique<AssistCandidateWord>(
                    issue.suggest, issue.message, rep, delGlyphs, origTail));
                added++;
            }

            if (added > 0) {
                auto &panel = st->ic->inputPanel();
                panel.setAuxDown(fcitx::Text(std::string("Trợ lý TextVN: ") + res.issues[0].message));
                panel.setCandidateList(std::move(candList));
                st->ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
                assistUiShowing_ = true;
                if (ipc_) {
                    lc_ipc_client_send_assist_report(ipc_, res.count, 0);
                }
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
        if (st->assistUndoReplacementGlyphs > 0) {
            st->ic->deleteSurroundingText(
                -static_cast<int>(st->assistUndoReplacementGlyphs), 0);
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
            viEnabled_ = appEnabled != 0;
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

    if (!isRelease && assistUiShowing_) {
        if (key.sym() == FcitxKey_Escape) {
            clearAssistUI(st);
            return true;
        }
        // Để Fcitx5 tự xử lý phím số 1..3 chọn ứng viên từ candidateList đang hiện
        if (!states.test(fcitx::KeyState::Ctrl) && !states.test(fcitx::KeyState::Alt) &&
            !states.test(fcitx::KeyState::Super) && key.sym() >= FcitxKey_1 &&
            key.sym() <= FcitxKey_3) {
            return false;
        }
        clearAssistUI(st);
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

    if (st->comp.len == 0) {
        lc_config_sync(st->inst, &st->config, nullptr);
        syncState(st);
        consumeGlobalIpc();
        syncAssistState(st);
        consumeAssistIpc();
    }

    /* B6 chord; S3 ô mật khẩu; VN tắt (trừ khi còn gõ tắt); engine hỏng → phím đi thẳng. */
    const bool secure =
        st->ic->capabilityFlags().testAny(fcitx::CapabilityFlag::PasswordOrSensitive);
    const bool macroOnly = !viEnabled_ && st->config.allow_macro_when_vi_off;
    if ((mods & (IME_MOD_CTRL | IME_MOD_ALT | IME_MOD_SUPER | IME_MOD_META)) || secure ||
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

    uint32_t vk = 0, ch = 0;
    mapFcitxKey(key, &vk, &ch);
    const lc_key k = lc_key_classify(vk, ch);
    ime_key_v1 ik;
    lc_key_to_ime(k, vk, mods & (IME_MOD_SHIFT | IME_MOD_CAPS), &ik);

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
        st->ic->deleteSurroundingText(-static_cast<int>(plan.delete_before), plan.delete_before);
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
        return checkAndApplyAssist(st, k.ch, isSentence, plan.eaten != 0);
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
