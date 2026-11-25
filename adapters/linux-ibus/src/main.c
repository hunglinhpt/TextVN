/* main.c — Tiến trình engine IBus của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§4
 *
 * ibus-daemon chạy `<exec> --ibus` từ component XML rồi CHỜ tên bus trùng
 * `<name>` của component; tên lệch = daemon timeout, engine không bao giờ dùng được.
 */

#include <fcntl.h>
#include <glib-unix.h>
#include <ibus.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "textvn_ibus_engine.h"

#define TEXTVN_IBUS_COMPONENT "io.github.hunglinhpt.textvn"
#define TEXTVN_IBUS_ENGINE    "textvn"

static void bus_disconnected(IBusBus *bus, gpointer user_data) {
    (void)bus;
    (void)user_data;
    ibus_quit();
}

static gboolean on_term_signal(gpointer user_data) {
    (void)user_data;
    ibus_quit();
    return G_SOURCE_REMOVE;
}

static IBusComponent *make_component(const char *setup_path) {
    const char *setup = (setup_path && setup_path[0]) ? setup_path : "/usr/bin/textvn-settings";
    /* B13/E67: IBus/GNOME tách <setup> bằng g_shell_parse_argv -> trích dẫn khi có khoảng trắng. */
    gchar *quoted_setup = g_shell_quote(setup);
    IBusComponent *component = ibus_component_new(
        TEXTVN_IBUS_COMPONENT, "TextVN Vietnamese Input Method", TEXTVN_VERSION,
        "GPL-3.0-or-later", "LinhBH.CoM", "https://github.com/hunglinhpt/TextVN", "", "textvn");
    IBusEngineDesc *desc = ibus_engine_desc_new_varargs(
        "name", TEXTVN_IBUS_ENGINE,
        "longname", "Vietnamese (TextVN)",
        "description", "Vietnamese Input Method (Telex, VNI, VIQR)",
        "language", "vi",
        "license", "GPL-3.0-or-later",
        "author", "LinhBH.CoM",
        "icon", "textvn_v",
        "layout", "default",
        "symbol", "VN",
        "setup", quoted_setup,
        "rank", (guint)99,
        NULL);
    g_free(quoted_setup);
    ibus_component_add_engine(component, desc);
    return component;
}

int main(int argc, char *argv[]) {
    gboolean launched_by_ibus = FALSE;
    gboolean wait_bus = FALSE;
    const char *setup_path = NULL;
    const char *ready_file = NULL;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--check-abi") == 0) {
            /* E67: kiểm tra loader (glibc/libibus/libglib) mà không cần ibus-daemon. */
            return 0;
        }
        if (strcmp(argv[i], "--version") == 0 || strcmp(argv[i], "-V") == 0) {
            printf("textvn-ibus-engine %s\n", TEXTVN_VERSION);
            return 0;
        }
        if (strcmp(argv[i], "--ibus") == 0 || strcmp(argv[i], "-i") == 0) {
            launched_by_ibus = TRUE;
        } else if (strcmp(argv[i], "--wait-bus") == 0) {
            wait_bus = TRUE;
        } else if (strcmp(argv[i], "--setup") == 0 && i + 1 < argc) {
            setup_path = argv[++i];
        } else if (strcmp(argv[i], "--ready-file") == 0 && i + 1 < argc) {
            ready_file = argv[++i];
        }
    }

    ibus_init();
    lc_log_init("ibus-engine");

    IBusBus *bus = ibus_bus_new();
    if (wait_bus) {
        /* E67: chế độ autostart trên RHEL 8.10 (IBus 1.5.19 bỏ qua IBUS_COMPONENT_PATH
         * khi /var/cache/ibus/bus/registry tồn tại) — chờ tối đa 30s để ibus-daemon khởi chạy. */
        for (int tries = 0; tries < 60 && (!bus || !ibus_bus_is_connected(bus)); ++tries) {
            if (bus) g_object_unref(bus);
            g_usleep(500000);
            bus = ibus_bus_new();
        }
    }
    if (!bus || !ibus_bus_is_connected(bus)) {
        lc_log(LC_LOG_ERROR, "Main", "Cannot connect to ibus-daemon");
        if (bus) g_object_unref(bus);
        lc_log_close();
        return 1;
    }
    if (wait_bus && !launched_by_ibus) {
        /* Nếu ibus-daemon đã có engine textvn (từ XML hệ thống/user HOẶC tiến trình
         * live-register đang chạy trong register_engine_list), thoát êm để không trùng. */
        gboolean exists = FALSE;
        GList *engines = ibus_bus_list_engines(bus);
        for (GList *p = engines; p != NULL; p = p->next) {
            IBusEngineDesc *d = (IBusEngineDesc *)p->data;
            if (d && g_strcmp0(ibus_engine_desc_get_name(d), TEXTVN_IBUS_ENGINE) == 0) {
                exists = TRUE;
                break;
            }
        }
        g_list_free_full(engines, g_object_unref);
        if (!exists) {
            G_GNUC_BEGIN_IGNORE_DEPRECATIONS
            GList *active = ibus_bus_list_active_engines(bus);
            G_GNUC_END_IGNORE_DEPRECATIONS
            for (GList *p = active; p != NULL; p = p->next) {
                IBusEngineDesc *d = (IBusEngineDesc *)p->data;
                if (d && g_strcmp0(ibus_engine_desc_get_name(d), TEXTVN_IBUS_ENGINE) == 0) {
                    exists = TRUE;
                    break;
                }
            }
            g_list_free_full(active, g_object_unref);
        }
        if (exists) {
            g_object_unref(bus);
            lc_log_close();
            return 0;
        }
    }
    g_signal_connect(bus, "disconnected", G_CALLBACK(bus_disconnected), NULL);
    g_unix_signal_add(SIGTERM, on_term_signal, NULL);
    g_unix_signal_add(SIGINT, on_term_signal, NULL);

    GDBusConnection *conn = ibus_bus_get_connection(bus);
    IBusFactory *factory = conn ? ibus_factory_new(conn) : NULL;
    if (!factory) {
        lc_log(LC_LOG_ERROR, "Main", "Cannot create IBusFactory");
        g_object_unref(bus);
        lc_log_close();
        return 1;
    }
    g_object_ref_sink(factory);
    ibus_factory_add_engine(factory, TEXTVN_IBUS_ENGINE, TEXTVN_TYPE_IBUS_ENGINE);

    if (launched_by_ibus) {
        ibus_bus_request_name(bus, TEXTVN_IBUS_COMPONENT, 0);
    } else {
        /* Chạy trực tiếp (debug hoặc chế độ portable live-register trên GNOME Wayland
         * RHEL 8.10 / Ubuntu 22.04 — đăng ký component sống mà KHÔNG restart ibus-daemon). */
        IBusComponent *component = make_component(setup_path);
        g_object_ref_sink(component);
        gboolean ok = ibus_bus_register_component(bus, component);
        g_object_unref(component);
        if (!ok) {
            lc_log(LC_LOG_ERROR, "Main", "ibus_bus_register_component failed");
            g_object_unref(factory);
            g_object_unref(bus);
            lc_log_close();
            return 1;
        }
        if (ready_file && ready_file[0]) {
            int fd = open(ready_file, O_WRONLY | O_CREAT | O_TRUNC | O_NOFOLLOW, 0600);
            if (fd >= 0) {
                if (write(fd, "ready\n", 6) < 0) {
                    lc_log(LC_LOG_WARN, "Main", "Cannot write ready file");
                }
                close(fd);
            }
        }
    }

    lc_log(LC_LOG_INFO, "Main", "textvn-ibus-engine ready");
    ibus_main();

    g_object_unref(factory);
    g_object_unref(bus);
    lc_log_close();
    return 0;
}
