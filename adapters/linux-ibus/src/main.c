/* main.c — Tiến trình engine IBus của TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-1-ibus.md §2-§4
 *
 * ibus-daemon chạy `<exec> --ibus` từ component XML rồi CHỜ tên bus trùng
 * `<name>` của component; tên lệch = daemon timeout, engine không bao giờ dùng được.
 */

#include <ibus.h>
#include <string.h>

#include "textvn_ibus_engine.h"

#define TEXTVN_IBUS_COMPONENT "io.github.hunglinhpt.textvn"
#define TEXTVN_IBUS_ENGINE    "textvn"

static void bus_disconnected(IBusBus *bus, gpointer user_data) {
    (void)bus;
    (void)user_data;
    ibus_quit();
}

static IBusComponent *make_component(void) {
    IBusComponent *component = ibus_component_new(
        TEXTVN_IBUS_COMPONENT, "TextVN Vietnamese Input Method", "0.1.0",
        "GPL-3.0-or-later", "hunglinhpt", "https://github.com/hunglinhpt/TextVN", "", "textvn");
    ibus_component_add_engine(
        component, ibus_engine_desc_new(TEXTVN_IBUS_ENGINE, "Vietnamese (TextVN)",
                                        "Vietnamese Input Method (Telex, VNI, VIQR)", "vi",
                                        "GPL-3.0-or-later", "hunglinhpt", "textvn_v", "default"));
    return component;
}

int main(int argc, char *argv[]) {
    gboolean launched_by_ibus = FALSE;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--ibus") == 0 || strcmp(argv[i], "-i") == 0) launched_by_ibus = TRUE;
    }

    ibus_init();
    lc_log_init("ibus-engine");

    IBusBus *bus = ibus_bus_new();
    if (!bus || !ibus_bus_is_connected(bus)) {
        lc_log(LC_LOG_ERROR, "Main", "Cannot connect to ibus-daemon");
        return 1;
    }
    g_signal_connect(bus, "disconnected", G_CALLBACK(bus_disconnected), NULL);

    IBusFactory *factory = ibus_factory_new(ibus_bus_get_connection(bus));
    g_object_ref_sink(factory);
    ibus_factory_add_engine(factory, TEXTVN_IBUS_ENGINE, TEXTVN_TYPE_IBUS_ENGINE);

    if (launched_by_ibus) {
        ibus_bus_request_name(bus, TEXTVN_IBUS_COMPONENT, 0);
    } else {
        /* Chạy tay (debug): tự đăng ký component với daemon. */
        IBusComponent *component = make_component();
        ibus_bus_register_component(bus, component);
        g_object_unref(component);
    }

    lc_log(LC_LOG_INFO, "Main", "textvn-ibus-engine ready");
    ibus_main();

    g_object_unref(factory);
    g_object_unref(bus);
    lc_log_close();
    return 0;
}
