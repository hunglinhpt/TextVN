/* keymap.h — Map Fcitx5 Key to TextVN ime_key_v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_FCITX5_KEYMAP_H
#define TEXTVN_FCITX5_KEYMAP_H

#include <fcitx-utils/key.h>
#include "textvn_ffi.h"

namespace textvn {

/**
 * Maps an incoming fcitx::Key event into a standardized ime_key_v1 struct
 * complying with the FFI contract P0-2 §1.
 *
 * Returns true if the key should be forwarded to the engine, false if ignored.
 */
bool map_fcitx_key_to_ime(const fcitx::Key &key, ime_key_v1 &out);

} // namespace textvn

#endif // TEXTVN_FCITX5_KEYMAP_H
