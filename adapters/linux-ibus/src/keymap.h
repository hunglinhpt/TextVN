/* keymap.h — IBus keyval/state → phím chuẩn hóa của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_IBUS_KEYMAP_H
#define TEXTVN_IBUS_KEYMAP_H

#include <ibus.h>

#include "lc_compose.h"

G_BEGIN_DECLS

/* VK canonical + ký tự Unicode của keysym (0 nếu không sinh ký tự). */
void textvn_ibus_map_key(guint keyval, uint32_t *vk, uint32_t *ch);

/* `IME_MOD_*` từ state mask. NumLock (MOD2) và AltGr (MOD5) không phải chord. */
uint32_t textvn_ibus_mods(guint state);

/* Ctrl/Shift/modifier khác/phím thường — cho phím chuyển Ctrl+Shift. */
lc_modifier textvn_ibus_modifier_kind(guint keyval);

G_END_DECLS

#endif /* TEXTVN_IBUS_KEYMAP_H */
