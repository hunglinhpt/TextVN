/* addon.h — Fcitx5 AddonInstance registration for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_FCITX5_ADDON_H
#define TEXTVN_FCITX5_ADDON_H

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <memory>

#include "engine.h"

namespace textvn {

class TextVNAddon : public fcitx::AddonInstance {
public:
    TextVNAddon(fcitx::AddonManager *manager);
    ~TextVNAddon() override;

private:
    std::unique_ptr<TextVNEngine> engine_;
};

class TextVNAddonFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new TextVNAddon(manager);
    }
};

} // namespace textvn

#endif // TEXTVN_FCITX5_ADDON_H
