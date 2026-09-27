/* keymap.h — Map IBus keyval and state to TextVN ime_key_v1
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_IBUS_KEYMAP_H
#define TEXTVN_IBUS_KEYMAP_H

#if defined(TEXTVN_IBUS_MOCK)
#include "ibus_mock.h"
#else
#include <ibus.h>
#endif

#include "textvn_ffi.h"

G_BEGIN_DECLS

/**
 * Maps an incoming IBus key event (keyval, keycode, state) to ime_key_v1.
 * Returns TRUE if key is valid for engine processing, FALSE otherwise.
 */
gboolean ibus_keyval_to_ime_key(guint keyval, guint keycode, guint state, ime_key_v1 *out);

G_END_DECLS

#endif /* TEXTVN_IBUS_KEYMAP_H */
