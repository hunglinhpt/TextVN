/* engine.h — Engine Fcitx5 của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Addon loại InputMethod: chính object này là InputMethodEngine (không có API
 * "registerInputMethod" trong Fcitx5 — entry lấy từ inputmethod/textvn.conf).
 * Mô hình preedit dùng chung: lc_compose.h.
 */

#ifndef TEXTVN_FCITX5_ENGINE_H
#define TEXTVN_FCITX5_ENGINE_H

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>

#include <cstdint>
#include <string>

#include "lc_compose.h"
#include "linux_common.h"
#include "textvn_ffi.h"

namespace textvn {

class TextVNEngine;

/* State per InputContext: engine riêng + preedit + config đã nạp. */
class TextVNState : public fcitx::InputContextProperty {
public:
    explicit TextVNState(fcitx::InputContext *ic);
    ~TextVNState() override;
    TextVNState(const TextVNState &) = delete;
    TextVNState &operator=(const TextVNState &) = delete;

    fcitx::InputContext *ic;
    ime_instance *inst = nullptr;
    lc_comp comp{};
    lc_config_state config{};
    lc_modifier_toggle toggle{};
};

class TextVNEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit TextVNEngine(fcitx::Instance *instance);
    ~TextVNEngine() override;

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void deactivate(const fcitx::InputMethodEntry &entry,
                    fcitx::InputContextEvent &event) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    std::string subModeLabelImpl(const fcitx::InputMethodEntry &entry,
                                 fcitx::InputContext &ic) override;

private:
    TextVNState *state(fcitx::InputContext *ic);
    void finishWord(TextVNState *st);
    void endWord(TextVNState *st, const fcitx::InputContextEvent &event);
    void commitText(TextVNState *st, const uint32_t *text, size_t len);
    void showPreedit(TextVNState *st, const uint32_t *text, size_t len);
    void toggleVietnamese(TextVNState *st);
    bool handleKey(TextVNState *st, const fcitx::Key &key, bool isRelease);

    fcitx::Instance *instance_;
    lc_ipc_client *ipc_ = nullptr;
    bool viEnabled_ = true;
    fcitx::FactoryFor<TextVNState> factory_;
};

class TextVNEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override;
};

/* Tiện ích thuần (test được không cần Fcitx5 chạy). */
void mapFcitxKey(const fcitx::Key &key, uint32_t *vk, uint32_t *ch);
uint32_t fcitxMods(fcitx::KeyStates states);
lc_modifier fcitxModifierKind(fcitx::KeySym sym);
std::string utf32ToUtf8(const uint32_t *text, size_t len);

} // namespace textvn

#endif // TEXTVN_FCITX5_ENGINE_H
