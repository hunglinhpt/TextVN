/* main.c — Tiến trình engine IBus của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§4
 *
 * ibus-daemon chạy `<exec> --ibus` từ component XML rồi CHỜ tên bus trùng
 * `<name>` của component; tên lệch = daemon timeout, engine không bao giờ dùng được.
 */

#include <glib-unix.h>
#include <ibus.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>

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
    const char *setup_path = NULL;
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
        } else if (strcmp(argv[i], "--setup") == 0 && i + 1 < argc) {
            setup_path = argv[++i];
        }
    }

    ibus_init();
    lc_log_init("ibus-engine");

    IBusBus *bus = ibus_bus_new();
    if (!bus || !ibus_bus_is_connected(bus)) {
        lc_log(LC_LOG_ERROR, "Main", "Cannot connect to ibus-daemon");
        if (bus) g_object_unref(bus);
        lc_log_close();
        return 1;
    }
    g_signal_connect(bus, "disconnected", G_CALLBACK(bus_disconnected), NULL);
    g_unix_signal_add(SIGTERM, on_term_signal, NULL);
    g_unix_signal_add(SIGINT, on_term_signal, NULL);

    IBusFactory *factory = ibus_factory_new(ibus_bus_get_connection(bus));
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
    }

    lc_log(LC_LOG_INFO, "Main", "textvn-ibus-engine ready");
    ibus_main();

    g_object_unref(factory);
    g_object_unref(bus);
    lc_log_close();
    return 0;
}
