/* test_ipc_client.c — Unit tests for IPC socket path resolution and frame encoding
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "linux_common.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

static void test_socket_path_resolution(void) {
    char path[256];

    /* Explicit custom path */
    assert(lc_ipc_resolve_socket_path(path, sizeof(path), "/tmp/custom.sock") == 0);
    assert(strcmp(path, "/tmp/custom.sock") == 0);

    /* Default resolution when custom is NULL */
    assert(lc_ipc_resolve_socket_path(path, sizeof(path), NULL) == 0);
    assert(strlen(path) > 0);
    assert(strstr(path, "ipc.sock") != NULL);
}

static void test_offline_fallback(void) {
    /* Connect to a nonexistent socket path -> must fail-open gracefully */
    lc_ipc_client *client = lc_ipc_client_new("test.app", "/tmp/nonexistent_testvn_socket_12345.sock");
    assert(client != NULL);

    /* Must be offline */
    assert(lc_ipc_client_is_online(client) == 0);

    /* Operations must not crash */
    uint64_t new_ver = 0;
    assert(lc_ipc_client_check_config_reload(client, &new_ver) == 0);

    int override_val = 0;
    assert(lc_ipc_client_get_app_override(client, &override_val) == 0);

    /* Non-blocking poll returns 0 */
    assert(lc_ipc_client_poll(client) == 0);

    lc_ipc_client_free(client);
}

int main(void) {
    test_socket_path_resolution();
    test_offline_fallback();

    printf("All linux-common IPC client tests passed successfully!\n");
    return 0;
}
