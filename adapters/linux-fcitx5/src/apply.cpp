/* apply.cpp — Apply IME result to Fcitx5 InputContext
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "apply.h"
#include "linux_common.h"

#include <fcitx/inputpanel.h>
#include <fcitx/text.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/capabilityflags.h>

namespace textvn {

void apply_commit_and_reset(fcitx::InputContext *ic) {
    if (!ic) return;
    ic->inputPanel().reset();
    ic->updatePreedit();
}

void apply_result(fcitx::InputContext *ic, const ime_result_v1 &result, bool non_preedit_mode) {
    if (!ic) return;

    char utf8_buf[256];

    switch (result.action) {
    case IME_ACTION_PASS:
        /* Do nothing; engine passes key */
        break;

    case IME_ACTION_REPLACE:
        if (non_preedit_mode) {
            /* Non-preedit mode (Gõ không gạch chân — fcitx5-lotus style) */
            /* Step 1: Delete surrounding characters */
            if (result.delete_count > 0) {
                if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
                    /* Supported: delete surrounding text directly */
                    ic->deleteSurroundingText(-static_cast<int>(result.delete_count), result.delete_count);
                } else {
                    /* Fallback: forward simulated Backspace keys */
                    for (uint16_t i = 0; i < result.delete_count; ++i) {
                        ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                    }
                }
            }

            /* Step 2: Commit inserted text */
            if (result.insert_len > 0) {
                lc_utf32_to_utf8(result.insert, result.insert_len, utf8_buf, sizeof(utf8_buf));
                ic->commitString(utf8_buf);
            }

            /* Clear preedit UI so no underline shows */
            ic->inputPanel().reset();
            ic->updatePreedit();
        } else {
            /* Classic Preedit mode with underline */
            if (result.preedit_len > 0) {
                lc_utf32_to_utf8(result.preedit, result.preedit_len, utf8_buf, sizeof(utf8_buf));
                fcitx::Text text;
                text.append(utf8_buf, fcitx::TextFormatFlag::Underline);
                ic->inputPanel().setClientPreedit(text);
                ic->updatePreedit();
            } else {
                ic->inputPanel().reset();
                ic->updatePreedit();
            }
        }
        break;

    case IME_ACTION_COMMIT:
        /* Commit text and clear preedit */
        ic->inputPanel().reset();
        ic->updatePreedit();

        if (result.insert_len > 0) {
            lc_utf32_to_utf8(result.insert, result.insert_len, utf8_buf, sizeof(utf8_buf));
            ic->commitString(utf8_buf);
        }
        break;

    case IME_ACTION_RESTORE:
        /* Auto-restore English sequence */
        if (result.delete_count > 0) {
            if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
                ic->deleteSurroundingText(-static_cast<int>(result.delete_count), result.delete_count);
            } else {
                for (uint16_t i = 0; i < result.delete_count; ++i) {
                    ic->forwardKey(fcitx::Key(FcitxKey_BackSpace));
                }
            }
        }
        if (result.insert_len > 0) {
            lc_utf32_to_utf8(result.insert, result.insert_len, utf8_buf, sizeof(utf8_buf));
            ic->commitString(utf8_buf);
        }
        ic->inputPanel().reset();
        ic->updatePreedit();
        break;

    default:
        break;
    }
}

} // namespace textvn
