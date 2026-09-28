/* textvn_ibus_engine.h — TextVN IBus Engine public header
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§6, mô hình preedit: lc_compose.h
 */

#ifndef TEXTVN_IBUS_ENGINE_H
#define TEXTVN_IBUS_ENGINE_H

#include <ibus.h>

#include "textvn_ffi.h"
#include "linux_common.h"
#include "lc_compose.h"

G_BEGIN_DECLS

#define TEXTVN_TYPE_IBUS_ENGINE (textvn_ibus_engine_get_type())

typedef struct _TextVNIbusEngine TextVNIbusEngine;
typedef struct _TextVNIbusEngineClass TextVNIbusEngineClass;

/* ibus-daemon tạo MỘT engine object cho mỗi input context. */
struct _TextVNIbusEngine {
    IBusEngine         parent;
    ime_instance      *inst;
    lc_comp            comp;      /* preedit đang mở (len == 0: không composing) */
    lc_config_state    config;    /* mtime config.json đã nạp */
    lc_modifier_toggle toggle;    /* Ctrl+Shift kiểu UniKey */
    gboolean           secure;    /* input purpose password/PIN */
    int                ctx_enabled; /* ime_context_v1.enabled đã đẩy vào engine */
    IBusPropList      *props;
    IBusProperty      *mode_prop;
    IBusProperty      *setup_prop;
};

struct _TextVNIbusEngineClass {
    IBusEngineClass parent_class;
};

GType textvn_ibus_engine_get_type(void);

G_END_DECLS

#endif /* TEXTVN_IBUS_ENGINE_H */
