/* gtk_mock.h — Standalone mock of GTK4 APIs for TextVN Settings Panel unit tests
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: PLAN §2.3 M6, P3-5-ui-packaging-release.md
 * Provides headless GTK4 mock structures and inline functions.
 */

#ifndef TEXTVN_GTK_MOCK_H
#define TEXTVN_GTK_MOCK_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void*          gpointer;
typedef int            gboolean;
typedef int            gint;
typedef unsigned int   guint;
typedef unsigned long  gulong;
typedef char           gchar;
typedef void (*GDestroyNotify)(gpointer data);

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif

#define G_CALLBACK(f) ((void (*)(void))(f))
#define G_OBJECT(w) ((gpointer)(w))

typedef enum {
    GTK_ORIENTATION_HORIZONTAL,
    GTK_ORIENTATION_VERTICAL
} GtkOrientation;

typedef enum {
    GTK_ALIGN_FILL,
    GTK_ALIGN_START,
    GTK_ALIGN_END,
    GTK_ALIGN_CENTER,
    GTK_ALIGN_BASELINE_FILL,
    GTK_ALIGN_BASELINE_CENTER
} GtkAlign;

typedef struct GtkWidget GtkWidget;
typedef struct GtkWindow GtkWindow;
typedef struct GtkApplication GtkApplication;
typedef struct GtkButton GtkButton;
typedef struct GtkCheckButton GtkCheckButton;
typedef struct GtkDropDown GtkDropDown;
typedef struct GtkBox GtkBox;
typedef struct GtkGrid GtkGrid;
typedef struct GtkFrame GtkFrame;
typedef struct GtkLabel GtkLabel;
typedef struct GtkAlertDialog GtkAlertDialog;

struct GtkWidget {
    bool visible;
    int width;
    int height;
    char text[256];
    bool active;
    guint selected_index;
    bool hexpand;
    bool vexpand;
    GtkAlign halign;
    GtkAlign valign;
    int margin_start;
    int margin_end;
    int margin_top;
    int margin_bottom;
    void *child;
    void *group;
    void (*clicked_cb)(void *btn, void *user_data);
    void *clicked_data;
};

#define GTK_WINDOW(w)       ((GtkWindow *)(w))
#define GTK_BUTTON(w)       ((GtkButton *)(w))
#define GTK_CHECK_BUTTON(w) ((GtkCheckButton *)(w))
#define GTK_DROP_DOWN(w)    ((GtkDropDown *)(w))
#define GTK_BOX(w)          ((GtkBox *)(w))
#define GTK_GRID(w)         ((GtkGrid *)(w))
#define GTK_FRAME(w)        ((GtkFrame *)(w))
#define GTK_LABEL(w)        ((GtkLabel *)(w))
#define GTK_APPLICATION(w)  ((GtkApplication *)(w))

static inline GtkWidget *gtk_mock_widget_new(void) {
    GtkWidget *w = (GtkWidget *)calloc(1, sizeof(GtkWidget));
    if (w) {
        w->visible = true;
    }
    return w;
}

static inline void gtk_widget_set_visible(GtkWidget *w, gboolean visible) {
    if (w) w->visible = (visible != FALSE);
}

static inline gboolean gtk_widget_get_visible(GtkWidget *w) {
    return w ? (w->visible ? TRUE : FALSE) : FALSE;
}

static inline void gtk_widget_set_margin_start(GtkWidget *w, gint margin) {
    if (w) w->margin_start = margin;
}

static inline void gtk_widget_set_margin_end(GtkWidget *w, gint margin) {
    if (w) w->margin_end = margin;
}

static inline void gtk_widget_set_margin_top(GtkWidget *w, gint margin) {
    if (w) w->margin_top = margin;
}

static inline void gtk_widget_set_margin_bottom(GtkWidget *w, gint margin) {
    if (w) w->margin_bottom = margin;
}

static inline void gtk_widget_set_hexpand(GtkWidget *w, gboolean expand) {
    if (w) w->hexpand = (expand != FALSE);
}

static inline void gtk_widget_set_vexpand(GtkWidget *w, gboolean expand) {
    if (w) w->vexpand = (expand != FALSE);
}

static inline void gtk_widget_set_halign(GtkWidget *w, GtkAlign align) {
    if (w) w->halign = align;
}

static inline void gtk_widget_set_valign(GtkWidget *w, GtkAlign align) {
    if (w) w->valign = align;
}

static inline void gtk_widget_set_size_request(GtkWidget *w, gint width, gint height) {
    if (w) {
        if (width >= 0) w->width = width;
        if (height >= 0) w->height = height;
    }
}

/* Window */
static inline GtkWidget *gtk_application_window_new(GtkApplication *app) {
    (void)app;
    return gtk_mock_widget_new();
}

static inline void gtk_window_set_title(GtkWindow *win, const gchar *title) {
    GtkWidget *w = (GtkWidget *)win;
    if (w && title) {
        strncpy(w->text, title, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
}

static inline void gtk_window_set_default_size(GtkWindow *win, gint width, gint height) {
    GtkWidget *w = (GtkWidget *)win;
    if (w) {
        w->width = width;
        w->height = height;
    }
}

static inline void gtk_window_set_resizable(GtkWindow *win, gboolean resizable) {
    (void)win; (void)resizable;
}

static inline void gtk_window_set_child(GtkWindow *win, GtkWidget *child) {
    GtkWidget *w = (GtkWidget *)win;
    if (w) w->child = child;
}

static inline void gtk_window_close(GtkWindow *win) {
    GtkWidget *w = (GtkWidget *)win;
    if (w) w->visible = false;
}

static inline void gtk_window_present(GtkWindow *win) {
    GtkWidget *w = (GtkWidget *)win;
    if (w) w->visible = true;
}

/* Box */
static inline GtkWidget *gtk_box_new(GtkOrientation orientation, gint spacing) {
    (void)orientation; (void)spacing;
    return gtk_mock_widget_new();
}

static inline void gtk_box_append(GtkBox *box, GtkWidget *child) {
    (void)box; (void)child;
}

/* Grid */
static inline GtkWidget *gtk_grid_new(void) {
    return gtk_mock_widget_new();
}

static inline void gtk_grid_set_row_spacing(GtkGrid *grid, guint spacing) {
    (void)grid; (void)spacing;
}

static inline void gtk_grid_set_column_spacing(GtkGrid *grid, guint spacing) {
    (void)grid; (void)spacing;
}

static inline void gtk_grid_attach(GtkGrid *grid, GtkWidget *child, gint column, gint row, gint width, gint height) {
    (void)grid; (void)child; (void)column; (void)row; (void)width; (void)height;
}

/* Frame */
static inline GtkWidget *gtk_frame_new(const gchar *label) {
    GtkWidget *w = gtk_mock_widget_new();
    if (w && label) {
        strncpy(w->text, label, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
    return w;
}

static inline void gtk_frame_set_child(GtkFrame *frame, GtkWidget *child) {
    GtkWidget *w = (GtkWidget *)frame;
    if (w) w->child = child;
}

/* Label */
static inline GtkWidget *gtk_label_new(const gchar *str) {
    GtkWidget *w = gtk_mock_widget_new();
    if (w && str) {
        strncpy(w->text, str, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
    return w;
}

/* DropDown */
static inline GtkWidget *gtk_drop_down_new_from_strings(const char *const *strings) {
    (void)strings;
    return gtk_mock_widget_new();
}

static inline void gtk_drop_down_set_selected(GtkDropDown *dd, guint pos) {
    GtkWidget *w = (GtkWidget *)dd;
    if (w) w->selected_index = pos;
}

static inline guint gtk_drop_down_get_selected(GtkDropDown *dd) {
    GtkWidget *w = (GtkWidget *)dd;
    return w ? w->selected_index : 0;
}

/* CheckButton */
static inline GtkWidget *gtk_check_button_new_with_label(const gchar *label) {
    GtkWidget *w = gtk_mock_widget_new();
    if (w && label) {
        strncpy(w->text, label, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
    return w;
}

static inline void gtk_check_button_set_active(GtkCheckButton *btn, gboolean is_active) {
    GtkWidget *w = (GtkWidget *)btn;
    if (w) w->active = (is_active != FALSE);
}

static inline gboolean gtk_check_button_get_active(GtkCheckButton *btn) {
    GtkWidget *w = (GtkWidget *)btn;
    return w ? (w->active ? TRUE : FALSE) : FALSE;
}

static inline void gtk_check_button_set_group(GtkCheckButton *btn, GtkCheckButton *group) {
    GtkWidget *w = (GtkWidget *)btn;
    if (w) w->group = group;
}

/* Button */
static inline GtkWidget *gtk_button_new_with_label(const gchar *label) {
    GtkWidget *w = gtk_mock_widget_new();
    if (w && label) {
        strncpy(w->text, label, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
    return w;
}

static inline void gtk_button_set_label(GtkButton *btn, const gchar *label) {
    GtkWidget *w = (GtkWidget *)btn;
    if (w && label) {
        strncpy(w->text, label, sizeof(w->text) - 1);
        w->text[sizeof(w->text) - 1] = '\0';
    }
}

static inline const gchar *gtk_button_get_label(GtkButton *btn) {
    GtkWidget *w = (GtkWidget *)btn;
    return w ? w->text : "";
}

/* Signal */
static inline gulong g_signal_connect(gpointer instance, const gchar *detailed_signal, void (*c_handler)(void), gpointer data) {
    (void)detailed_signal;
    GtkWidget *w = (GtkWidget *)instance;
    if (w) {
        w->clicked_cb = (void (*)(void *, void *))c_handler;
        w->clicked_data = data;
    }
    return 1;
}

static inline void g_object_set_data_full(gpointer obj, const gchar *key, gpointer data, GDestroyNotify destroy) {
    (void)obj; (void)key; (void)data; (void)destroy;
}

/* AlertDialog */
static inline GtkAlertDialog *gtk_alert_dialog_new(const char *format, ...) {
    (void)format;
    return (GtkAlertDialog *)calloc(1, 16);
}

static inline void gtk_alert_dialog_show(GtkAlertDialog *dialog, GtkWindow *parent) {
    (void)dialog; (void)parent;
    free(dialog);
}

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_GTK_MOCK_H */
