/* main.c — Entry point for TextVN Linux Settings Panel
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: PLAN §2.3 M6, P3-5-ui-packaging-release.md
 * UniKey 4.6 RC2 parity Control Panel.
 */

#include "settings_window.h"
#include <gtk/gtk.h>
#include <string.h>

static char *g_custom_config = NULL;

static void on_app_activate(GtkApplication *app, gpointer user_data) {
    (void)user_data;
    GtkWidget *win = textvn_settings_window_create(app, g_custom_config);
    if (win) {
        gtk_window_present(GTK_WINDOW(win));
    }
}

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        if ((strcmp(argv[i], "--config") == 0 || strcmp(argv[i], "-c") == 0) && i + 1 < argc) {
            g_custom_config = argv[++i];
        }
    }

    GtkApplication *app = gtk_application_new("dev.textvn.settings", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);

    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    return status;
}
