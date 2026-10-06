/* test_ipc_client.c — Unit tests for IPC socket path resolution and frame encoding
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "linux_common.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

#if defined(__linux__) || defined(__unix__)
#include <unistd.h>
#include <sys/socket.h>
#include <sys/un.h>
#endif

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

#if defined(__linux__) || defined(__unix__)
/* Vong 13 (BUG-03): broadcast toan cuc "app_id":"*" tu tray — client cu bo
 * qua im lang. Test nay dung server socket that: ghi frame StateUpdate "*"
 * roi poll o client phai tang seq va tra enabled dung (event-based). */
static void test_global_broadcast_state_update(void) {
    char sock[192];
    snprintf(sock, sizeof(sock), "/tmp/textvn-test-global-%d.sock", (int)getpid());
    unlink(sock);

    int srv = socket(AF_UNIX, SOCK_STREAM, 0);
    assert(srv >= 0);
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    size_t slen = strlen(sock);
    assert(slen < sizeof(addr.sun_path));
    memcpy(addr.sun_path, sock, slen + 1);
    assert(bind(srv, (struct sockaddr *)&addr, sizeof(addr)) == 0);
    assert(listen(srv, 4) == 0);

    lc_ipc_client *client = lc_ipc_client_new("test.app", sock);
    assert(client != NULL);
    assert(lc_ipc_client_is_online(client) == 1);

    /* Chap nhan ket noi + doc frame cua client (Hello/Subscribe/GetSnapshot). */
    int conn = accept(srv, NULL, NULL);
    assert(conn >= 0);
    uint8_t hdr[4];
    char buf[LC_IPC_MAX_FRAME + 1];
    for (int i = 0; i < 3; i++) {
        size_t got = 0;
        while (got < 4) {
            ssize_t n = recv(conn, hdr + got, 4 - got, 0);
            if (n <= 0) { assert(0 && "client frame header"); }
            got += (size_t)n;
        }
        uint32_t len = (uint32_t)hdr[0] | ((uint32_t)hdr[1] << 8) |
                       ((uint32_t)hdr[2] << 16) | ((uint32_t)hdr[3] << 24);
        assert(len <= LC_IPC_MAX_FRAME);
        size_t total = 0;
        while (total < len) {
            ssize_t n = recv(conn, buf + total, len - total, 0);
            if (n <= 0) { assert(0 && "client frame body"); }
            total += (size_t)n;
        }
    }

    /* Truoc broadcast: chua co trang thai toan cuc. */
    int g = 0;
    uint32_t seq = 0;
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 0);

    /* Server broadcast StateUpdate "*" enabled=false (huong ipc.v1.md). */
    const char *msg = "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":false,\"version\":7}";
    uint32_t len = (uint32_t)strlen(msg);
    uint8_t out[4];
    out[0] = (uint8_t)(len & 0xFF);
    out[1] = (uint8_t)((len >> 8) & 0xFF);
    out[2] = (uint8_t)((len >> 16) & 0xFF);
    out[3] = (uint8_t)((len >> 24) & 0xFF);
    assert(send(conn, out, 4, 0) == 4);
    assert((size_t)send(conn, msg, len, 0) == len);

    /* Client poll → tieu thu su kien. */
    assert(lc_ipc_client_poll(client) >= 1);
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 1);
    assert(g == 0);

    /* Broadcast lan 2 (enabled=true) → seq tang, engine tieu thu duoc. */
    msg = "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":true,\"version\":8}";
    len = (uint32_t)strlen(msg);
    assert(send(conn, out, 4, 0) == 4);
    assert((size_t)send(conn, msg, len, 0) == len);
    assert(lc_ipc_client_poll(client) >= 1);
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 2);
    assert(g == 1);

    close(conn);
    close(srv);
    unlink(sock);
    lc_ipc_client_free(client);
}

/* Snapshot ket noi: "*" trong state map phai duoc doc ngay khi connect. */
static void test_global_snapshot_on_connect(void) {
    char sock[192];
    snprintf(sock, sizeof(sock), "/tmp/textvn-test-snap-%d.sock", (int)getpid());
    unlink(sock);

    int srv = socket(AF_UNIX, SOCK_STREAM, 0);
    assert(srv >= 0);
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    size_t slen = strlen(sock);
    memcpy(addr.sun_path, sock, slen + 1);
    assert(bind(srv, (struct sockaddr *)&addr, sizeof(addr)) == 0);
    assert(listen(srv, 4) == 0);

    lc_ipc_client *client = lc_ipc_client_new("test.app", sock);
    assert(client != NULL);

    int conn = accept(srv, NULL, NULL);
    assert(conn >= 0);
    /* Drain 3 frame client (Hello/Subscribe/GetSnapshot) — doc la bo. */
    uint8_t hdr[4];
    char buf[LC_IPC_MAX_FRAME + 1];
    for (int i = 0; i < 3; i++) {
        size_t got = 0;
        while (got < 4) {
            ssize_t n = recv(conn, hdr + got, 4 - got, 0);
            if (n <= 0) { assert(0 && "client frame header"); }
            got += (size_t)n;
        }
        uint32_t len = (uint32_t)hdr[0] | ((uint32_t)hdr[1] << 8) |
                       ((uint32_t)hdr[2] << 16) | ((uint32_t)hdr[3] << 24);
        assert(len <= LC_IPC_MAX_FRAME);
        size_t total = 0;
        while (total < len) {
            ssize_t n = recv(conn, buf + total, len - total, 0);
            if (n <= 0) { assert(0 && "client frame body"); }
            total += (size_t)n;
        }
    }

    /* Snapshot voi state map chua "*" = false (hang dau tien cua map). */
    const char *snap = "{\"type\":\"Snapshot\",\"config_version\":3,\"state\":{\"*\":false,\"vscode.exe\":true},\"appdb_version\":1,\"channel\":\"stable\"}";
    uint32_t len = (uint32_t)strlen(snap);
    uint8_t out[4];
    out[0] = (uint8_t)(len & 0xFF);
    out[1] = (uint8_t)((len >> 8) & 0xFF);
    out[2] = (uint8_t)((len >> 16) & 0xFF);
    out[3] = (uint8_t)((len >> 24) & 0xFF);
    assert(send(conn, out, 4, 0) == 4);
    assert((size_t)send(conn, snap, len, 0) == len);

    assert(lc_ipc_client_poll(client) >= 1);
    int g = 0;
    uint32_t seq = 0;
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 1);
    assert(g == 0);

    close(conn);
    close(srv);
    unlink(sock);
    lc_ipc_client_free(client);
}
#endif

int main(void) {
    test_socket_path_resolution();
    test_offline_fallback();
#if defined(__linux__) || defined(__unix__)
    test_global_broadcast_state_update();
    test_global_snapshot_on_connect();
#endif

    printf("All linux-common IPC client tests passed successfully!\n");
    return 0;
}
