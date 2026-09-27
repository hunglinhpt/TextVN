/* textvn_ibus_engine.h — TextVN IBus Engine public header
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§6
 */

#ifndef TEXTVN_IBUS_ENGINE_H
#define TEXTVN_IBUS_ENGINE_H

#if defined(TEXTVN_IBUS_MOCK)
#include "ibus_mock.h"
#else
#include <ibus.h>
#endif

#include "textvn_ffi.h"
#include "linux_common.h"

G_BEGIN_DECLS

#define TEXTVN_TYPE_IBUS_ENGINE (textvn_ibus_engine_get_type())

typedef struct _TextVNIbusEngine TextVNIbusEngine;
typedef struct _TextVNIbusEngineClass TextVNIbusEngineClass;

struct _TextVNIbusEngine {
    IBusEngine    parent;
    ime_instance *inst;
    lc_ipc_client *ipc_client;
    gboolean      vi_enabled;
    gboolean      non_preedit;
    gboolean      has_surrounding;
    uint32_t      field_role;
    uint32_t      secure;
    int64_t       strategy_hint;
    guint         cursor_pos;
};

struct _TextVNIbusEngineClass {
    IBusEngineClass parent_class;
};

GType textvn_ibus_engine_get_type(void);

/**
 * Instantiate a new TextVN IBusEngine object.
 */
IBusEngine *textvn_ibus_engine_new(void);

G_END_DECLS

#endif /* TEXTVN_IBUS_ENGINE_H */
