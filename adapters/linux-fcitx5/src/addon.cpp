/* addon.cpp — Fcitx5 AddonInstance registration for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "addon.h"

namespace textvn {

TextVNAddon::TextVNAddon(fcitx::AddonManager *manager) {
    engine_ = std::make_unique<TextVNEngine>(manager->instance());
    manager->registerInputMethod(engine_.get(), "textvn", "TextVN", "vi_VN", "VN", "textvn");
}

TextVNAddon::~TextVNAddon() = default;

} // namespace textvn

FCITX_ADDON_FACTORY(textvn::TextVNAddonFactory)
