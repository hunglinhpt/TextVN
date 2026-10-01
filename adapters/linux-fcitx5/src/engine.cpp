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
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <fcitx/statusarea.h>
#include <fcitx/text.h>
#include <fcitx/userinterface.h>
#include <fcitx/userinterfacemanager.h>

#include <dlfcn.h>

#include <cstring>
#include <string>

namespace textvn {

/* ---- Tiện ích thuần ---- */

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

void TextVNEngine::toggleVietnamese(TextVNState *st) { setVietnamese(st, !viEnabled_, true); }

/* Bảng cài đặt vừa đổi state.json → theo ngay (một stat mỗi lần gọi). */
void TextVNEngine::syncState(TextVNState *st) {
    int enabled = viEnabled_ ? 1 : 0;
    if (lc_state_sync(&stateWatch_, nullptr, &enabled)) setVietnamese(st, enabled != 0, false);
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
        int appEnabled = 1;
        if (ipc_ && lc_ipc_client_get_app_override(ipc_, &appEnabled)) {
            viEnabled_ = appEnabled != 0;
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

    uint32_t mods = fcitxMods(states);
    /* Caps Lock bật: engine coi phím dấu Telex viết hoa là phím dấu (VIEETJ → VIỆT). */
    if (rawStates.test(fcitx::KeyState::CapsLock)) mods |= IME_MOD_CAPS;

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
