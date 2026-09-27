/* apply.h — Apply IME result to Fcitx5 InputContext
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_FCITX5_APPLY_H
#define TEXTVN_FCITX5_APPLY_H

#include <fcitx/inputcontext.h>
#include "textvn_ffi.h"

namespace textvn {

/**
 * Apply ime_result_v1 to the Fcitx5 InputContext.
 *
 * In Non-preedit mode (Gõ không gạch chân inspired by fcitx5-lotus):
 * - Deletes surrounding text via ic->deleteSurroundingText(-delete_count, delete_count)
 *   or forwards simulated Backspace keys if surrounding text is not supported.
 * - Commits newly inserted characters via ic->commitString(utf8).
 * - Leaves preedit clear so no underline is displayed.
 *
 * In Preedit mode:
 * - Updates client preedit with underline attribute via ic->inputPanel().setClientPreedit().
 *
 * Handles Bug B1 (Autocomplete / SelectionReplace) and Bug B2 (Commit-before-enter).
 */
void apply_result(fcitx::InputContext *ic, const ime_result_v1 &result, bool non_preedit_mode);

/**
 * Commit any active composition and reset preedit panel.
 */
void apply_commit_and_reset(fcitx::InputContext *ic);

} // namespace textvn

#endif // TEXTVN_FCITX5_APPLY_H
