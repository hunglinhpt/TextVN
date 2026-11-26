/* main.c — Bảng điều khiển TextVN cho Linux
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Một phiên duy nhất: mở lần hai (menu IBus/Fcitx5, launcher) chỉ đưa cửa sổ đang có
 * lên trước.
 *
 *   textvn-settings [--config <đường dẫn config.json>]
 */

#include <gtk/gtk.h>
#include <string.h>

#include "settings_window.h"

#ifndef TEXTVN_VERSION
#define TEXTVN_VERSION "dev"
#endif

static const char *g_custom_config = NULL;
static gboolean g_run_diagnose = FALSE;

static void on_action_diagnose(GSimpleAction *action, GVariant *param, gpointer user_data) {
    (void)action;
    (void)param;
    GtkApplication *app = GTK_APPLICATION(user_data);
    GtkWidget *win = textvn_settings_window_create(app, g_custom_config);
    if (win) {
        gtk_window_present(GTK_WINDOW(win));
        textvn_settings_trigger_diag(win);
    }
}

static void on_app_activate(GtkApplication *app, gpointer user_data) {
    (void)user_data;
    GtkWidget *win = textvn_settings_window_create(app, g_custom_config);
    if (win) {
        gtk_window_present(GTK_WINDOW(win));
        if (g_run_diagnose) {
            g_run_diagnose = FALSE;
            textvn_settings_trigger_diag(win);
        }
    }
}

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--check-abi") == 0) {
            /* E67: xác minh loader (glibc + GTK3/GTK4) mà không cần X11/Wayland display. */
            return 0;
        }
        if (strcmp(argv[i], "--version") == 0 || strcmp(argv[i], "-V") == 0) {
            g_print("textvn-settings %s (GTK%d)\n", TEXTVN_VERSION, GTK_MAJOR_VERSION);
            return 0;
        }
        if (strcmp(argv[i], "--diagnose") == 0 || strcmp(argv[i], "--export-log") == 0) {
            g_run_diagnose = TRUE;
        } else if ((strcmp(argv[i], "--config") == 0 || strcmp(argv[i], "-c") == 0) &&
                   i + 1 < argc) {
            g_custom_config = argv[++i];
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            g_print("Cách dùng: textvn-settings [--config <config.json>] [--diagnose]\n");
            return 0;
        }
    }

    /* Cờ 0 (không dùng G_APPLICATION_DEFAULT_FLAGS: chỉ có từ GLib 2.74). */
    GtkApplication *app = gtk_application_new("io.github.hunglinhpt.textvn.settings",
                                              (GApplicationFlags)0);
    GSimpleAction *diag_act = g_simple_action_new("diagnose", NULL);
    g_signal_connect(diag_act, "activate", G_CALLBACK(on_action_diagnose), app);
    g_action_map_add_action(G_ACTION_MAP(app), G_ACTION(diag_act));
    g_object_unref(diag_act);
    g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);
    if (g_run_diagnose && g_application_register(G_APPLICATION(app), NULL, NULL) &&
        g_application_get_is_remote(G_APPLICATION(app))) {
        g_action_group_activate_action(G_ACTION_GROUP(app), "diagnose", NULL);
        g_object_unref(app);
        return 0;
    }
    /* Tham số đã tự xử lý ở trên — GApplication sẽ báo lỗi tuỳ chọn lạ (--config/--diagnose). */
    char *app_argv[] = {argv[0], NULL};
    int status = g_application_run(G_APPLICATION(app), 1, app_argv);
    g_object_unref(app);
    return status;
}
