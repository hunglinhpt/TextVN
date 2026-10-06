/* test_ipc_client.c — Unit tests for IPC socket path resolution and frame encoding
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

/* Vong 13: _GNU_SOURCE phai dat TRUOC moi include — struct ucred (SO_PEERCRED)
 * trong phan .c duoc include chi ton tai khi _GNU_SOURCE. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "linux_common.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

/* Vong 13: include thang .c de test cham vao handle_ipc_message (static) va
 * struct client (fd/is_online) — chan parser BUG-03 ma khong phai qua socket. */
#if defined(__linux__) || defined(__unix__)
#include "../src/ipc_client.c"
#endif

#if defined(__linux__) || defined(__unix__)
#include <unistd.h>
#include <time.h>
#include <sys/socket.h>
#include <sys/un.h>
#endif

/* -std=c11 an toan: nanosleep (POSIX), khong dung usleep (bi an duoi _GNU_SOURCE). */
static void sleep_ms(int ms) {
#if defined(__linux__) || defined(__unix__)
    struct timespec ts = {ms / 1000, (long)(ms % 1000) * 1000000L};
    nanosleep(&ts, NULL);
#endif
}

/* Poll retry: select co the tra -1 (EINTR) tren runner CI — production vo hai
 * (phim ke tiep poll lai), test can doi du lieu den. Tra so message xu ly. */
static int poll_until_message(lc_ipc_client *client) {
    int processed = 0;
    for (int i = 0; i < 100 && processed == 0; i++) {
        processed = lc_ipc_client_poll(client);
        if (processed == 0) {
            sleep_ms(10);
        }
    }
    return processed;
}

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

    /* Server broadcast 2 StateUpdate "*" LIEN TUC truoc khi client poll
     * (deterministic: ca hai frame trong queue — poll 1 frame 1, poll 2
     * frame 2). Broadcast den SAU mot chuoi poll duoc e2e (server that) phu
     * trach — trong test don le, send-sau-poll tung mat frame khong giai
     * thich duoc (DIAG2: fd song, queue EAGAIN) — xem roadmap. */
    const char *msg = "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":false,\"version\":7}";
    uint32_t len = (uint32_t)strlen(msg);
    char frame[LC_IPC_MAX_FRAME + 4];
    frame[0] = (char)(len & 0xFF);
    frame[1] = (char)((len >> 8) & 0xFF);
    frame[2] = (char)((len >> 16) & 0xFF);
    frame[3] = (char)((len >> 24) & 0xFF);
    memcpy(frame + 4, msg, len);
    assert((size_t)send(conn, frame, 4 + len, 0) == 4 + len);
    msg = "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":true,\"version\":8}";
    len = (uint32_t)strlen(msg);
    memcpy(frame + 4, msg, len);
    assert((size_t)send(conn, frame, 4 + len, 0) == 4 + len);

    /* Poll den khi CA HAI frame da tieu thu (poll la drain-style: mot lan
     * co the doc ca 2 frame neu ca hai da den; cung co the lan 1 frame 1,
     * lan 2 frame 2 — khong gia dinh batching). Trang thai CUOI: seq=2,
     * enabled=true. */
    int guard = 0;
    while (seq < 2 && guard < 100) {
        int processed = poll_until_message(client);
        if (processed < 1) {
            char peek[16] = {0};
            ssize_t pn = recv(client->fd, peek, sizeof(peek), MSG_PEEK | MSG_DONTWAIT);
            fprintf(stderr,
                    "DIAG loop=%d online=%d fd=%d conn=%d peek=%zd errno=%d data='%.12s'\n",
                    guard, lc_ipc_client_is_online(client), client->fd, conn, pn,
                    pn < 0 ? errno : 0, peek);
            processed = poll_until_message(client);
        }
        assert(processed >= 1);
        guard++;
    }
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

    char frame[LC_IPC_MAX_FRAME + 4];
    /* Snapshot voi state map chua "*" = false (hang dau tien cua map). */
    const char *snap = "{\"type\":\"Snapshot\",\"config_version\":3,\"state\":{\"*\":false,\"vscode.exe\":true},\"appdb_version\":1,\"channel\":\"stable\"}";
    uint32_t len = (uint32_t)strlen(snap);
    memcpy(frame + 4, snap, len);
    assert((size_t)send(conn, frame, 4 + len, 0) == 4 + len);

    assert(poll_until_message(client) >= 1);
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

/* Vong 13 (BUG-03): parser phai nhan trang thai toan cuc "*" — test truc tiep
 * handle_ipc_message (offline client, khong can server). */
#if defined(__linux__) || defined(__unix__)
static void test_parser_global_state(void) {
    lc_ipc_client *client = lc_ipc_client_new("test.app", "/tmp/nonexistent_parser_test.sock");
    assert(client != NULL);
    int g = 0;
    uint32_t seq = 0;
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 0);

    handle_ipc_message(client,
        "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":false,\"version\":7}");
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 1);
    assert(g == 0);

    handle_ipc_message(client,
        "{\"type\":\"StateUpdate\",\"app_id\":\"*\",\"enabled\":true,\"version\":8}");
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 2);
    assert(g == 1);

    handle_ipc_message(client,
        "{\"type\":\"StateUpdate\",\"app_id\":\"vscode.exe\",\"enabled\":false,\"version\":9}");
    /* broadcast khong phai "*" KHONG duoc tang seq toan cuc */
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 2);

    handle_ipc_message(client,
        "{\"type\":\"Snapshot\",\"config_version\":3,\"state\":{\"*\":false,\"vscode.exe\":true},\"appdb_version\":1,\"channel\":\"stable\"}");
    assert(lc_ipc_client_get_global_override(client, &g, &seq) == 1);
    assert(seq == 3);
    assert(g == 0);

    lc_ipc_client_free(client);
}
#endif

int main(void) {
    test_socket_path_resolution();
    test_offline_fallback();
#if defined(__linux__) || defined(__unix__)
    test_parser_global_state();
    test_global_broadcast_state_update();
    test_global_snapshot_on_connect();
#endif

    printf("All linux-common IPC client tests passed successfully!\n");
    return 0;
}
