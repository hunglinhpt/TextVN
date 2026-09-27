/* engine.h — Fcitx5 InputMethodEngineV2 implementation for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_FCITX5_ENGINE_H
#define TEXTVN_FCITX5_ENGINE_H

#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <unordered_map>
#include <memory>
#include <string>

#include "linux_common.h"
#include "textvn_ffi.h"

namespace textvn {

struct ContextData {
    ime_instance *inst = nullptr;
    bool          vi_enabled = true;
    bool          non_preedit = true; /* Default: Gõ không gạch chân (fcitx5-lotus style) */
    uint32_t      field_role = IME_FIELD_UNKNOWN;
    int64_t       strategy_hint = -1;
};

class TextVNEngine : public fcitx::InputMethodEngineV2 {
public:
    TextVNEngine(fcitx::Instance *instance);
    ~TextVNEngine() override;

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;

    bool isViEnabled(fcitx::InputContext *ic) const;
    void toggleViEn(fcitx::InputContext *ic);
    bool isNonPreedit(fcitx::InputContext *ic) const;
    size_t contextCount() const { return contexts_.size(); }

private:
    ContextData *getOrCreateContext(fcitx::InputContext *ic);
    void destroyContext(fcitx::InputContext *ic);

    fcitx::Instance *instance_;
    lc_ipc_client   *ipc_client_;
    std::unordered_map<fcitx::InputContext*, ContextData> contexts_;
    std::string current_config_json_;
};

} // namespace textvn

#endif // TEXTVN_FCITX5_ENGINE_H
