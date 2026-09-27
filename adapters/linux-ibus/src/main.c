/* main.c — Entry point for TextVN IBus Engine process
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§4
 */

#if defined(TEXTVN_IBUS_MOCK)
#include "ibus_mock.h"
#else
#include <ibus.h>
#endif

#include "textvn_ibus_engine.h"
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

static IBusBus     *s_bus = NULL;
static IBusFactory *s_factory = NULL;

static void sigterm_handler(int sig) {
    (void)sig;
    if (s_bus) {
        ibus_quit();
    }
}

static void bus_disconnected(IBusBus *bus, gpointer user_data) {
    (void)bus;
    (void)user_data;
    ibus_quit();
}

int main(int argc, char *argv[]) {
    ibus_init();

    lc_log_init("ibus-engine");
    lc_log(LC_LOG_INFO, "Main", "Starting textvn-ibus-engine process");

    signal(SIGINT, sigterm_handler);
    signal(SIGTERM, sigterm_handler);

    s_bus = ibus_bus_new();
    if (!s_bus || !ibus_bus_is_connected(s_bus)) {
        lc_log(LC_LOG_ERROR, "Main", "Failed to connect to ibus-daemon");
        return 1;
    }

    g_signal_connect(s_bus, "disconnected", G_CALLBACK(bus_disconnected), NULL);

    s_factory = ibus_factory_new(ibus_bus_get_connection(s_bus));
    ibus_factory_add_engine(s_factory, "textvn", TEXTVN_TYPE_IBUS_ENGINE);

    /* Request engine name on bus */
    ibus_bus_request_name(s_bus, "org.freedesktop.IBus.TextVN", 0);

    lc_log(LC_LOG_INFO, "Main", "textvn-ibus-engine registered and entering main loop");
    ibus_main();

    if (s_factory) {
        g_object_unref(s_factory);
        s_factory = NULL;
    }
    if (s_bus) {
        g_object_unref(s_bus);
        s_bus = NULL;
    }

    lc_log(LC_LOG_INFO, "Main", "textvn-ibus-engine stopped");
    lc_log_close();
    return 0;
}
