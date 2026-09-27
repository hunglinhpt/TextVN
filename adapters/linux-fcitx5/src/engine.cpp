/* engine.cpp — Fcitx5 InputMethodEngineV2 implementation for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Implements:
 * - Non-preedit typing (Gõ không gạch chân — fcitx5-lotus style)
 * - Enter key handling for Bug B2 (anti-duplication in chat apps)
 * - Hotkey toggle Ctrl+Shift+Space (ADR-011)
 * - System shortcuts passthrough (Bug B6)
 * - Fail-open (S4) exception boundary
 */

#include "engine.h"
#include "keymap.h"
#include "apply.h"

#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx-utils/capabilityflags.h>
#include <fcitx-utils/keysym.h>

#include <cstring>

namespace textvn {

TextVNEngine::TextVNEngine(fcitx::Instance *instance)
    : fcitx::InputMethodEngineV2(instance), instance_(instance), ipc_client_(nullptr)
{
    lc_log_init("fcitx5-textvn");
    ipc_client_ = lc_ipc_client_new("fcitx5", nullptr);
    lc_log(LC_LOG_INFO, "Engine", "TextVNEngine initialized (Fcitx5 Addon)");
}

TextVNEngine::~TextVNEngine() {
    for (auto &pair : contexts_) {
        if (pair.second.inst) {
            ime_instance_free(pair.second.inst);
            pair.second.inst = nullptr;
        }
    }
    contexts_.clear();

    if (ipc_client_) {
        lc_ipc_client_free(ipc_client_);
        ipc_client_ = nullptr;
    }
    lc_log(LC_LOG_INFO, "Engine", "TextVNEngine destroyed");
    lc_log_close();
}

ContextData *TextVNEngine::getOrCreateContext(fcitx::InputContext *ic) {
    if (!ic) return nullptr;

    auto it = contexts_.find(ic);
    if (it != contexts_.end()) {
        return &it->second;
    }

    ContextData data;
    data.vi_enabled = true;
    data.non_preedit = true; /* Lotus-style non-preedit by default */
    data.field_role = IME_FIELD_UNKNOWN;
    data.strategy_hint = -1;

    /* Initialize core IME instance via C-ABI */
    int32_t rc = ime_instance_new(nullptr, 0, &data.inst);
    if (rc != IME_OK && rc != IME_ERR_CONFIG) {
        lc_log(LC_LOG_ERROR, "Engine", "Failed to create ime_instance: rc=%d", (int)rc);
        data.inst = nullptr;
    }

    auto inserted = contexts_.emplace(ic, data);

    /* Connect destruction callback to free ime_instance and prevent leaks (LNX-020) */
    ic->connect<fcitx::InputContext::Destroyed>([this, ic]() {
        destroyContext(ic);
    });

    return &inserted.first->second;
}

void TextVNEngine::destroyContext(fcitx::InputContext *ic) {
    auto it = contexts_.find(ic);
    if (it != contexts_.end()) {
        if (it->second.inst) {
            ime_instance_free(it->second.inst);
            it->second.inst = nullptr;
        }
        contexts_.erase(it);
    }
}

bool TextVNEngine::isViEnabled(fcitx::InputContext *ic) const {
    auto it = contexts_.find(ic);
    if (it != contexts_.end()) {
        return it->second.vi_enabled;
    }
    return true;
}

bool TextVNEngine::isNonPreedit(fcitx::InputContext *ic) const {
    auto it = contexts_.find(ic);
    if (it != contexts_.end()) {
        return it->second.non_preedit;
    }
    return true;
}

void TextVNEngine::toggleViEn(fcitx::InputContext *ic) {
    auto *data = getOrCreateContext(ic);
    if (!data) return;

    data->vi_enabled = !data->vi_enabled;
    lc_log(LC_LOG_INFO, "Engine", "Toggled VI mode for app '%s': %s",
           ic->program().c_str(), data->vi_enabled ? "ON" : "OFF");

    if (ipc_client_) {
        lc_ipc_client_toggle_vi_en(ipc_client_, ic->program().c_str(), data->vi_enabled ? 1 : 0);
    }

    /* Reset buffer on language toggle */
    if (data->inst) {
        ime_reset(data->inst);
    }
    apply_commit_and_reset(ic);
}

void TextVNEngine::activate(const fcitx::InputMethodEntry &/*entry*/, fcitx::InputContextEvent &event) {
    try {
        auto *ic = event.inputContext();
        if (!ic) return;

        auto *data = getOrCreateContext(ic);
        if (!data || !data->inst) return;

        /* Check for configuration hot-reload from tray */
        uint64_t new_config_ver = 0;
        if (ipc_client_ && lc_ipc_client_check_config_reload(ipc_client_, &new_config_ver)) {
            lc_log(LC_LOG_INFO, "Engine", "Reloading config version %llu", (unsigned long long)new_config_ver);
            ime_reload_config(data->inst, nullptr, 0);
        }

        /* Check app-specific enabled override from tray */
        int app_override = 0;
        if (ipc_client_ && lc_ipc_client_get_app_override(ipc_client_, &app_override)) {
            data->vi_enabled = (app_override != 0);
        }

        /* Probe capability flags */
        bool hasSurrounding = ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText);
        uint32_t caps = IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT | IME_CAP_SELECTION;
        (void)hasSurrounding; /* Surrounding capability utilized directly by apply_result */

        /* Detect text field context via AT-SPI or heuristics */
        lc_field_ctx fctx;
        lc_classify_field(ic->program().c_str(), nullptr, 0, nullptr, nullptr, &fctx);

        data->field_role = fctx.field_role;
        data->strategy_hint = fctx.strategy_hint;

        /* Configure context in engine */
        ime_context_v1 ctx;
        std::memset(&ctx, 0, sizeof(ctx));
        ctx.abi_version = IME_ABI_VERSION;
        ctx.enabled = data->vi_enabled ? 1 : 0;
        ctx.secure = fctx.secure;
        ctx.field_role = fctx.field_role;
        ctx.caps = caps;
        ctx.app_id = ic->program().c_str();
        ctx.hint = fctx.strategy_hint;

        ime_set_context(data->inst, &ctx);
    } catch (...) {
        /* Fail-open S4 */
    }
}

void TextVNEngine::deactivate(const fcitx::InputMethodEntry &/*entry*/, fcitx::InputContextEvent &event) {
    try {
        auto *ic = event.inputContext();
        if (!ic) return;

        /* Bug B13 & Bug B2 protection (Commit-before-hide) */
        apply_commit_and_reset(ic);

        auto it = contexts_.find(ic);
        if (it != contexts_.end() && it->second.inst) {
            ime_reset(it->second.inst);
        }
    } catch (...) {
        /* Fail-open S4 */
    }
}

void TextVNEngine::reset(const fcitx::InputMethodEntry &/*entry*/, fcitx::InputContextEvent &event) {
    try {
        auto *ic = event.inputContext();
        if (!ic) return;

        apply_commit_and_reset(ic);

        auto it = contexts_.find(ic);
        if (it != contexts_.end() && it->second.inst) {
            ime_reset(it->second.inst);
        }
    } catch (...) {
        /* Fail-open S4 */
    }
}

void TextVNEngine::keyEvent(const fcitx::InputMethodEntry &/*entry*/, fcitx::KeyEvent &keyEvent) {
    try {
        auto *ic = keyEvent.inputContext();
        if (!ic) return;

        const auto &key = keyEvent.key();

        /* Ignore key-up (only handle key-down) */
        if (key.isRelease()) {
            return;
        }

        /* Hotkey: Ctrl+Shift+Space to toggle Vietnamese/English (ADR-011) */
        if (key.check(FcitxKey_space) &&
            key.hasModifier(fcitx::KeyModifier::Ctrl) &&
            key.hasModifier(fcitx::KeyModifier::Shift)) {
            toggleViEn(ic);
            keyEvent.filterAndAccept();
            return;
        }

        /* Bug B6: System shortcuts & hotkeys (Ctrl/Alt/Super combinations)
         * Must pass through directly to avoid breaking Ctrl+C, Ctrl+V, Alt+Tab, etc. */
        if (key.hasModifier(fcitx::KeyModifier::Ctrl) ||
            key.hasModifier(fcitx::KeyModifier::Alt)  ||
            key.hasModifier(fcitx::KeyModifier::Super)) {
            auto *data = getOrCreateContext(ic);
            if (data && data->inst) {
                ime_reset(data->inst);
            }
            apply_commit_and_reset(ic);
            return; /* Let application handle hotkey */
        }

        /* Bug B2: Enter key handling in chat applications (Slack, Discord, Telegram, Zalo)
         * If user presses Enter while composing, finish/commit the word immediately
         * and DO NOT consume the Enter key, allowing it to submit the message cleanly. */
        if (key.check(FcitxKey_Return) || key.check(FcitxKey_KP_Enter)) {
            auto *data = getOrCreateContext(ic);
            if (data && data->inst) {
                ime_reset(data->inst);
            }
            apply_commit_and_reset(ic);
            return; /* Do not filterAndAccept: Enter passes through to send message */
        }

        /* Map Fcitx5 key to standardized ime_key_v1 */
        ime_key_v1 k;
        if (!map_fcitx_key_to_ime(key, k)) {
            return;
        }

        /* If Vietnamese mode is disabled, let key pass through */
        auto *data = getOrCreateContext(ic);
        if (!data || !data->vi_enabled || !data->inst) {
            return;
        }

        /* Send key to core Rust engine via C-ABI */
        ime_result_v1 result;
        int32_t rc = ime_key(data->inst, &k, &result);
        if (rc != IME_OK || (result.flags & IME_FLAG_ERROR)) {
            /* Fail-open: pass key on any engine error */
            return;
        }

        /* If engine chose PASS action, forward key to application */
        if (result.action == IME_ACTION_PASS) {
            return;
        }

        /* Apply result to application */
        apply_result(ic, result, data->non_preedit);
        keyEvent.filterAndAccept();

    } catch (...) {
        /* Fail-open S4: Never crash the host Fcitx5 process */
        return;
    }
}

} // namespace textvn
