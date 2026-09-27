/* apply.c — Apply IME result to IBus Engine
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "apply.h"
#include <string.h>

void apply_ibus_reset_preedit(TextVNIbusEngine *self) {
    if (!self) return;
    ibus_engine_hide_preedit_text((IBusEngine *)self);
}

void apply_ibus_commit(TextVNIbusEngine *self) {
    if (!self) return;
    apply_ibus_reset_preedit(self);
}

void apply_ibus_result(TextVNIbusEngine *self, const ime_result_v1 *res) {
    if (!self || !res) return;

    char utf8_buf[256];

    switch (res->action) {
    case IME_ACTION_PASS:
        /* No action; let engine forward key */
        break;

    case IME_ACTION_REPLACE:
        if (self->non_preedit && self->has_surrounding) {
            /* Non-preedit mode (Gõ không gạch chân — ibus-bamboo / fcitx5-lotus style) */
            /* Step 1: Delete surrounding characters if requested */
            if (res->delete_count > 0) {
                ibus_engine_delete_surrounding_text((IBusEngine *)self,
                                                    -((gint)res->delete_count),
                                                    res->delete_count);
            }

            /* Step 2: Commit inserted text */
            if (res->insert_len > 0) {
                lc_utf32_to_utf8(res->insert, res->insert_len, utf8_buf, sizeof(utf8_buf));
                IBusText *text = ibus_text_new_from_string(utf8_buf);
                ibus_engine_commit_text((IBusEngine *)self, text);
            }

            /* Hide preedit UI so no underline is displayed */
            ibus_engine_hide_preedit_text((IBusEngine *)self);
        } else {
            /* Classic Preedit mode with underline */
            if (res->preedit_len > 0) {
                lc_utf32_to_utf8(res->preedit, res->preedit_len, utf8_buf, sizeof(utf8_buf));
                IBusText *text = ibus_text_new_from_string(utf8_buf);
                ibus_text_append_attribute(text, IBUS_ATTR_TYPE_UNDERLINE, IBUS_ATTR_UNDERLINE_SINGLE, 0, (guint)strlen(utf8_buf));
                ibus_engine_update_preedit_text((IBusEngine *)self, text, (guint)strlen(utf8_buf), TRUE);
                ibus_engine_show_preedit_text((IBusEngine *)self);
            } else {
                ibus_engine_hide_preedit_text((IBusEngine *)self);
            }
        }
        break;

    case IME_ACTION_COMMIT:
        /* Word boundary or commit: hide preedit and commit string */
        ibus_engine_hide_preedit_text((IBusEngine *)self);
        if (res->insert_len > 0) {
            lc_utf32_to_utf8(res->insert, res->insert_len, utf8_buf, sizeof(utf8_buf));
            IBusText *text = ibus_text_new_from_string(utf8_buf);
            ibus_engine_commit_text((IBusEngine *)self, text);
        }
        break;

    case IME_ACTION_RESTORE:
        /* Auto-restore English sequence */
        if (res->delete_count > 0 && self->has_surrounding) {
            ibus_engine_delete_surrounding_text((IBusEngine *)self,
                                                -((gint)res->delete_count),
                                                res->delete_count);
        }
        if (res->insert_len > 0) {
            lc_utf32_to_utf8(res->insert, res->insert_len, utf8_buf, sizeof(utf8_buf));
            IBusText *text = ibus_text_new_from_string(utf8_buf);
            ibus_engine_commit_text((IBusEngine *)self, text);
        }
        ibus_engine_hide_preedit_text((IBusEngine *)self);
        break;

    default:
        break;
    }
}
