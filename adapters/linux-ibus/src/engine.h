/* engine.h — Internal declarations for TextVN IBus Engine
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_IBUS_ENGINE_INTERNAL_H
#define TEXTVN_IBUS_ENGINE_INTERNAL_H

#include "textvn_ibus_engine.h"

G_BEGIN_DECLS

void     textvn_ibus_engine_focus_in(IBusEngine *engine);
void     textvn_ibus_engine_focus_out(IBusEngine *engine);
void     textvn_ibus_engine_enable(IBusEngine *engine);
void     textvn_ibus_engine_disable(IBusEngine *engine);
void     textvn_ibus_engine_reset(IBusEngine *engine);
void     textvn_ibus_engine_set_surrounding_text(IBusEngine *engine,
                                                 IBusText   *text,
                                                 guint       cursor_pos,
                                                 guint       anchor_pos);
gboolean textvn_ibus_engine_process_key_event(IBusEngine *engine,
                                              guint       keyval,
                                              guint       keycode,
                                              guint       state);

G_END_DECLS

#endif /* TEXTVN_IBUS_ENGINE_INTERNAL_H */
