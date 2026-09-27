/* ibus_mock.h — Standalone mock of GLib and IBus C APIs for testing
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef TEXTVN_IBUS_MOCK_H
#define TEXTVN_IBUS_MOCK_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

#define G_BEGIN_DECLS
#define G_END_DECLS

typedef void*          gpointer;
typedef int            gboolean;
typedef int            gint;
typedef unsigned int   guint;
typedef unsigned long  gulong;
typedef char           gchar;
typedef size_t         GType;

#define TRUE  1
#define FALSE 0

/* Key Modifier Masks */
#define IBUS_SHIFT_MASK   (1 << 0)
#define IBUS_LOCK_MASK    (1 << 1)
#define IBUS_CONTROL_MASK (1 << 2)
#define IBUS_MOD1_MASK    (1 << 3)
#define IBUS_MOD4_MASK    (1 << 6)
#define IBUS_SUPER_MASK   (1 << 26)
#define IBUS_RELEASE_MASK (1u << 30)

/* Attribute Types */
#define IBUS_ATTR_TYPE_UNDERLINE   1
#define IBUS_ATTR_UNDERLINE_SINGLE 1

/* Keysyms */
#define IBUS_KEY_BackSpace 0xff08
#define IBUS_KEY_Tab       0xff09
#define IBUS_KEY_KP_Tab    0xff89
#define IBUS_KEY_Return    0xff0d
#define IBUS_KEY_KP_Enter  0xff8d
#define IBUS_KEY_Escape    0xff1b
#define IBUS_KEY_space     0x0020
#define IBUS_KEY_KP_Space  0xff80
#define IBUS_KEY_a 0x0061
#define IBUS_KEY_z 0x007a
#define IBUS_KEY_A 0x0041
#define IBUS_KEY_Z 0x005a
#define IBUS_KEY_0 0x0030
#define IBUS_KEY_9 0x0039

/* GObject Mock Structure */
typedef struct _GObject GObject;
typedef struct _GObjectClass GObjectClass;

struct _GObject {
    GType g_type;
};

struct _GObjectClass {
    void (*finalize)(GObject *object);
};

#define G_OBJECT_CLASS(klass) ((GObjectClass *)(klass))

/* IBusText Mock */
typedef struct _IBusText {
    char text[256];
    guint attr_type;
    guint attr_val;
} IBusText;

static inline IBusText *ibus_text_new_from_string(const gchar *str) {
    IBusText *t = (IBusText *)calloc(1, sizeof(IBusText));
    if (t && str) {
        strncpy(t->text, str, sizeof(t->text) - 1);
    }
    return t;
}

static inline void ibus_text_append_attribute(IBusText *t, guint type, guint val, guint start, guint end) {
    (void)start; (void)end;
    if (t) {
        t->attr_type = type;
        t->attr_val = val;
    }
}

/* IBusEngine Mock Structure */
typedef struct _IBusEngine IBusEngine;
typedef struct _IBusEngineClass IBusEngineClass;

struct _IBusEngine {
    GObject  parent;
    char     mock_committed[256];
    char     mock_preedit[256];
    gboolean mock_preedit_visible;
    gint     mock_deleted_offset;
    guint    mock_deleted_count;
    gboolean mock_surrounding_supported;
    char     mock_surrounding[512];
    guint    mock_cursor;
    guint    mock_anchor;
};

struct _IBusEngineClass {
    GObjectClass parent_class;
    gboolean (*process_key_event)(IBusEngine *engine, guint keyval, guint keycode, guint state);
    void (*focus_in)(IBusEngine *engine);
    void (*focus_out)(IBusEngine *engine);
    void (*enable)(IBusEngine *engine);
    void (*disable)(IBusEngine *engine);
    void (*reset)(IBusEngine *engine);
    void (*set_surrounding_text)(IBusEngine *engine, IBusText *text, guint cursor_pos, guint anchor_pos);
};

#define IBUS_ENGINE_CLASS(klass) ((IBusEngineClass *)(klass))
#define IBUS_TYPE_ENGINE 1

/* Engine Methods */
static inline void ibus_engine_commit_text(IBusEngine *engine, IBusText *text) {
    if (engine && text) {
        strncpy(engine->mock_committed, text->text, sizeof(engine->mock_committed) - 1);
        free(text);
    }
}

static inline void ibus_engine_update_preedit_text(IBusEngine *engine, IBusText *text, guint cursor_pos, gboolean visible) {
    (void)cursor_pos;
    if (engine && text) {
        strncpy(engine->mock_preedit, text->text, sizeof(engine->mock_preedit) - 1);
        engine->mock_preedit_visible = visible;
        free(text);
    }
}

static inline void ibus_engine_show_preedit_text(IBusEngine *engine) {
    if (engine) engine->mock_preedit_visible = TRUE;
}

static inline void ibus_engine_hide_preedit_text(IBusEngine *engine) {
    if (engine) {
        engine->mock_preedit_visible = FALSE;
        engine->mock_preedit[0] = '\0';
    }
}

static inline void ibus_engine_delete_surrounding_text(IBusEngine *engine, gint offset, guint n_chars) {
    if (engine) {
        engine->mock_deleted_offset = offset;
        engine->mock_deleted_count = n_chars;
    }
}

static inline void ibus_engine_get_surrounding_text(IBusEngine *engine, IBusText **text, guint *cursor_pos, guint *anchor_pos) {
    if (engine && engine->mock_surrounding_supported) {
        if (text) *text = ibus_text_new_from_string(engine->mock_surrounding);
        if (cursor_pos) *cursor_pos = engine->mock_cursor;
        if (anchor_pos) *anchor_pos = engine->mock_anchor;
    } else {
        if (text) *text = NULL;
        if (cursor_pos) *cursor_pos = 0;
        if (anchor_pos) *anchor_pos = 0;
    }
}

/* Bus & Factory Stubs for main.c */
typedef struct _IBusBus IBusBus;
typedef struct _IBusFactory IBusFactory;
typedef void (*GCallback)(void);

static inline void ibus_init(void) {}
static inline void ibus_main(void) {}
static inline void ibus_quit(void) {}
static inline IBusBus *ibus_bus_new(void) { return (IBusBus *)calloc(1, 8); }
static inline gboolean ibus_bus_is_connected(IBusBus *bus) { (void)bus; return TRUE; }
static inline gpointer ibus_bus_get_connection(IBusBus *bus) { (void)bus; return (gpointer)1; }
static inline void ibus_bus_request_name(IBusBus *b, const gchar *n, guint f) { (void)b; (void)n; (void)f; }
static inline IBusFactory *ibus_factory_new(gpointer c) { (void)c; return (IBusFactory *)calloc(1, 8); }
static inline void ibus_factory_add_engine(IBusFactory *f, const gchar *n, GType t) { (void)f; (void)n; (void)t; }
static void (*s_mock_finalize)(GObject *) = NULL;

static inline void g_object_unref(gpointer p) {
    if (!p) return;
    if (s_mock_finalize) {
        s_mock_finalize((GObject *)p);
    }
    free(p);
}

#define G_CALLBACK(f) ((GCallback)(f))

/* G_DEFINE_TYPE Mock Macro */
#define G_DEFINE_TYPE(TN, t_n, T_P) \
    static void t_n##_init(TN *self); \
    static void t_n##_class_init(TN##Class *klass); \
    static gpointer t_n##_parent_class = NULL; \
    static TN##Class s_##t_n##_class; \
    GType t_n##_get_type(void) { \
        static GType type = 1001; \
        static int initialized = 0; \
        if (!initialized) { \
            memset(&s_##t_n##_class, 0, sizeof(s_##t_n##_class)); \
            t_n##_class_init(&s_##t_n##_class); \
            s_mock_finalize = ((GObjectClass *)&s_##t_n##_class)->finalize; \
            initialized = 1; \
        } \
        return type; \
    } \
    gpointer g_object_new(GType type, const gchar *prop, ...) { \
        (void)prop; \
        if (type == t_n##_get_type()) { \
            TN *obj = (TN *)calloc(1, sizeof(TN)); \
            t_n##_init(obj); \
            return obj; \
        } \
        return NULL; \
    }

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_IBUS_MOCK_H */
