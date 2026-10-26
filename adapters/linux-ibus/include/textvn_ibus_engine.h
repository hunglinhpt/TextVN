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

/* Candidate suggestion for Mode 1 */
typedef struct _TextVNAssistCandidate {
    uint32_t delete_glyphs;
    char     replacement[LC_MAX_SENTENCE_FIX_CHARS];
    char     original_tail[LC_MAX_SENTENCE_FIX_CHARS];
} TextVNAssistCandidate;

/* ibus-daemon tạo MỘT engine object cho mỗi input context. */
struct _TextVNIbusEngine {
    IBusEngine            parent;
    ime_instance         *inst;
    lc_comp               comp;        /* preedit đang mở (len == 0: không composing) */
    lc_config_state       config;      /* mtime config.json đã nạp */
    lc_modifier_toggle    toggle;      /* Ctrl+Shift kiểu UniKey */
    gboolean              secure;      /* input purpose password/PIN */
    int                   ctx_enabled; /* ime_context_v1.enabled đã đẩy vào engine */
    IBusPropList         *props;
    IBusProperty         *mode_prop;
    IBusProperty         *setup_prop;

    /* Trợ lý ngữ pháp (AS-060 / P4-6) */
    IBusLookupTable      *lookup_table;
    gboolean              assist_ui_showing;
    /* R2-134: dòng phụ đang hiện bản xem trước dự đoán ("⇥ phần mềm"). */
    gboolean              preview_showing;
    guint                 assist_candidate_count;
    TextVNAssistCandidate assist_candidates[3];

    /* Single-step undo (AS-050 / R-4) */
    gboolean              assist_undo_valid;
    guint64               assist_undo_timestamp_ms;
    uint32_t              assist_undo_replacement_glyphs;
    char                  assist_undo_original_tail[LC_MAX_SENTENCE_FIX_CHARS];
    char                  assist_undo_replacement[LC_MAX_SENTENCE_FIX_CHARS];
};

struct _TextVNIbusEngineClass {
    IBusEngineClass parent_class;
};

GType textvn_ibus_engine_get_type(void);

G_END_DECLS

#endif /* TEXTVN_IBUS_ENGINE_H */
