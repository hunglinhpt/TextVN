/* apply.h — Apply IME result to IBus Engine
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_IBUS_APPLY_H
#define TEXTVN_IBUS_APPLY_H

#include "textvn_ibus_engine.h"

G_BEGIN_DECLS

/**
 * Apply ime_result_v1 to the given TextVNIbusEngine.
 * Handles surrounding text deletion, preedit update, and text commitment.
 */
void apply_ibus_result(TextVNIbusEngine *self, const ime_result_v1 *res);

/**
 * Commit any pending composition text and reset preedit panel.
 */
void apply_ibus_commit(TextVNIbusEngine *self);

/**
 * Reset and hide preedit panel.
 */
void apply_ibus_reset_preedit(TextVNIbusEngine *self);

G_END_DECLS

#endif /* TEXTVN_IBUS_APPLY_H */
