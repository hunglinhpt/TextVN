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

static const char *g_custom_config = NULL;

static void on_app_activate(GtkApplication *app, gpointer user_data) {
    (void)user_data;
    GtkWidget *win = textvn_settings_window_create(app, g_custom_config);
    if (win) gtk_window_present(GTK_WINDOW(win));
}

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        if ((strcmp(argv[i], "--config") == 0 || strcmp(argv[i], "-c") == 0) && i + 1 < argc) {
            g_custom_config = argv[++i];
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            g_print("Cách dùng: textvn-settings [--config <config.json>]\n");
            return 0;
        }
    }

    /* Cờ 0 (không dùng G_APPLICATION_DEFAULT_FLAGS: chỉ có từ GLib 2.74). */
    GtkApplication *app = gtk_application_new("io.github.hunglinhpt.textvn.settings",
                                              (GApplicationFlags)0);
    g_signal_connect(app, "activate", G_CALLBACK(on_app_activate), NULL);
    /* Tham số đã tự xử lý ở trên — GApplication sẽ báo lỗi tuỳ chọn lạ (--config). */
    char *app_argv[] = {argv[0], NULL};
    int status = g_application_run(G_APPLICATION(app), 1, app_argv);
    g_object_unref(app);
    return status;
}
