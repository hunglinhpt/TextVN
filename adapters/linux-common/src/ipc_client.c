/* ipc_client.c — Unix domain socket IPC client for TextVN Linux adapters
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: schemas/ipc.v1.md & docs/10-shared/P0-3-config-preset-strategy.md §5
 */

/* struct ucred (SO_PEERCRED) chỉ khai báo khi có _GNU_SOURCE. */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "linux_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>

#if defined(__linux__) || defined(__unix__)
#include <unistd.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/select.h>
#if defined(__linux__)
#include <sys/stat.h>
#endif
#endif

/* Fallback clock helper */
static uint64_t get_time_ms(void) {
#if defined(__linux__) || defined(__unix__)
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        return (uint64_t)ts.tv_sec * 1000 + (uint64_t)ts.tv_nsec / 1000000;
    }
    return 0;
#else
    return (uint64_t)clock() * 1000 / CLOCKS_PER_SEC;
#endif
}

struct lc_ipc_client {
    int      fd;
    char     app_id[LC_MAX_APP_ID];
    char     sock_path[256];
    int      is_online;
    uint64_t latest_config_version;
    uint64_t last_handled_config_version;
    uint32_t appdb_version;
    int      has_app_override;
    int      app_enabled_override;
    uint64_t last_connect_attempt_ms;
};

int lc_ipc_resolve_socket_path(char *out_path, size_t max_len, const char *custom_path) {
    if (!out_path || max_len == 0) return -1;

    if (custom_path && custom_path[0] != '\0') {
        strncpy(out_path, custom_path, max_len - 1);
        out_path[max_len - 1] = '\0';
        return 0;
    }

    /* SEC-04/L-02: uu tien $XDG_RUNTIME_DIR (systemd: thu muc rieng cua user,
     * da 0700, tmpfs) — an toan hon $XDG_CONFIG_HOME va khong bao gio dung
     * /tmp chung. */
    const char *xdg_runtime = getenv("XDG_RUNTIME_DIR");
    if (xdg_runtime && xdg_runtime[0] == '/') {
        snprintf(out_path, max_len, "%s/TextVN/ipc.sock", xdg_runtime);
        return 0;
    }

    const char *xdg_config = getenv("XDG_CONFIG_HOME");
    if (xdg_config && xdg_config[0] != '\0') {
        snprintf(out_path, max_len, "%s/TextVN/ipc.sock", xdg_config);
        return 0;
    }

    const char *home = getenv("HOME");
    if (home && home[0] != '\0') {
        snprintf(out_path, max_len, "%s/%s", home, LC_SOCK_DEFAULT_REL);
        return 0;
    }

    /* Fallback cuoi: /tmp — nhung trong thu muc RIENG theo uid (server tao
     * mode 0700) thay vi /tmp/textvn-ipc.sock dung chung cho moi user. */
#if defined(__linux__) || defined(__unix__)
    snprintf(out_path, max_len, "/tmp/textvn-%u/ipc.sock", (unsigned)getuid());
#else
    strncpy(out_path, "/tmp/textvn-ipc.sock", max_len - 1);
    out_path[max_len - 1] = '\0';
#endif
    return 0;
}

int lc_ipc_send_frame(int fd, const char *json_utf8, size_t len) {
    if (fd < 0 || !json_utf8 || len > LC_IPC_MAX_FRAME) {
        return -1;
    }

#if defined(__linux__) || defined(__unix__)
    /* Wire framing: 4-byte little-endian length + JSON bytes */
    uint8_t header[4];
    header[0] = (uint8_t)(len & 0xFF);
    header[1] = (uint8_t)((len >> 8) & 0xFF);
    header[2] = (uint8_t)((len >> 16) & 0xFF);
    header[3] = (uint8_t)((len >> 24) & 0xFF);

    ssize_t sent = write(fd, header, 4);
    if (sent != 4) return -1;

    size_t total_sent = 0;
    while (total_sent < len) {
        ssize_t n = write(fd, json_utf8 + total_sent, len - total_sent);
        if (n <= 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        total_sent += (size_t)n;
    }
    return 0;
#else
    (void)fd; (void)json_utf8; (void)len;
    return -1;
#endif
}

int lc_ipc_recv_frame(int fd, char *out_buf, size_t buf_size, size_t *out_len) {
    if (fd < 0 || !out_buf || buf_size == 0) return -1;

#if defined(__linux__) || defined(__unix__)
    uint8_t header[4];
    size_t hdr_read = 0;
    while (hdr_read < 4) {
        ssize_t n = read(fd, header + hdr_read, 4 - hdr_read);
        if (n <= 0) {
            if (n < 0 && errno == EINTR) continue;
            return -1;
        }
        hdr_read += (size_t)n;
    }

    uint32_t length = (uint32_t)header[0] |
                      ((uint32_t)header[1] << 8) |
                      ((uint32_t)header[2] << 16) |
                      ((uint32_t)header[3] << 24);

    if (length > LC_IPC_MAX_FRAME || length >= buf_size) {
        return -1; /* Oversize or buffer too small */
    }

    size_t payload_read = 0;
    while (payload_read < length) {
        ssize_t n = read(fd, out_buf + payload_read, length - payload_read);
        if (n <= 0) {
            if (n < 0 && errno == EINTR) continue;
            return -1;
        }
        payload_read += (size_t)n;
    }

    out_buf[length] = '\0';
    if (out_len) *out_len = length;
    return 0;
#else
    (void)fd; (void)out_buf; (void)buf_size; (void)out_len;
    return -1;
#endif
}

/* Minimal parser for IPC JSON v1 messages */
static void handle_ipc_message(lc_ipc_client *client, const char *json) {
    if (!client || !json) return;

    /* Detect ConfigReload */
    const char *p_reload = strstr(json, "\"ConfigReload\"");
    if (p_reload) {
        const char *p_ver = strstr(p_reload, "\"version\":");
        if (p_ver) {
            uint64_t ver = (uint64_t)strtoull(p_ver + 10, NULL, 10);
            if (ver > client->latest_config_version) {
                client->latest_config_version = ver;
            }
        }
        return;
    }

    /* Detect StateUpdate */
    const char *p_state = strstr(json, "\"StateUpdate\"");
    if (p_state) {
        /* Khớp ĐÚNG key JSON ("app_id":"<id>") — strstr trần app_id khớp nhầm
         * "code.exe" trong "vscode.exe" rồi đọc enabled của app khác. */
        char key[192];
        int kw = snprintf(key, sizeof key, "\"app_id\":\"%s\"", client->app_id);
        if (client->app_id[0] != '\0' && kw > 0 && (size_t)kw < sizeof key &&
            strstr(p_state, key)) {
            const char *p_en = strstr(p_state, "\"enabled\":");
            if (p_en) {
                p_en += 10;
                while (*p_en == ' ' || *p_en == '\t') p_en++;
                client->has_app_override = 1;
                client->app_enabled_override = (strncmp(p_en, "true", 4) == 0);
            }
        }
        return;
    }

    /* Detect Snapshot */
    const char *p_snap = strstr(json, "\"Snapshot\"");
    if (p_snap) {
        const char *p_cfg = strstr(p_snap, "\"config_version\":");
        if (p_cfg) {
            client->latest_config_version = (uint64_t)strtoull(p_cfg + 17, NULL, 10);
        }
        const char *p_appdb = strstr(p_snap, "\"appdb_version\":");
        if (p_appdb) {
            client->appdb_version = (uint32_t)strtoul(p_appdb + 16, NULL, 10);
        }
        /* Check state overrides for client->app_id — khớp đúng key
         * "<app_id>": thay vì strstr trần; snapshot thiếu app → clear override
         * cũ (khớp hành vi client Rust apply_states). */
        if (client->app_id[0] != '\0') {
            char key[192];
            int kw = snprintf(key, sizeof key, "\"%s\":", client->app_id);
            const char *p_app = (kw > 0 && (size_t)kw < sizeof key)
                                    ? strstr(p_snap, key)
                                    : NULL;
            if (p_app) {
                const char *p_colon = strchr(p_app, ':');
                if (p_colon) {
                    p_colon++;
                    while (*p_colon == ' ' || *p_colon == '\t') p_colon++;
                    client->has_app_override = 1;
                    client->app_enabled_override = (strncmp(p_colon, "true", 4) == 0);
                }
            } else {
                client->has_app_override = 0;
            }
        }
        return;
    }
}

static int try_connect(lc_ipc_client *client) {
    if (!client) return -1;

#if defined(__linux__) || defined(__unix__)
    client->last_connect_attempt_ms = get_time_ms();

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    /* Đường dẫn dài hơn sun_path: không cắt âm thầm (sẽ nối tới socket khác). */
    size_t path_len = strlen(client->sock_path);
    if (path_len >= sizeof(addr.sun_path)) return -1;
    memcpy(addr.sun_path, client->sock_path, path_len + 1);

#if defined(__linux__)
    /* SEC-04/L-02: duong dan socket co the nam trong thu muc ghi chung (/tmp).
     * Neu doi tuong DA ton tai ma khong phai socket cua chinh user nay thi tu
     * choi — chan pre-create/symlink hijack truoc khi connect. */
    struct stat sb;
    if (lstat(client->sock_path, &sb) == 0) {
        if (!S_ISSOCK(sb.st_mode) || sb.st_uid != getuid()) {
            lc_log(LC_LOG_WARN, "IPC", "Socket path exists but is not a socket owned by this user; refusing");
            return -1;
        }
    }
#endif

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return -1;

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        client->is_online = 0;
        client->fd = -1;
        return -1;
    }

#if defined(__linux__) && defined(SO_PEERCRED)
    /* SO_PEERCRED check: verify peer UID matches current UID (P0-3 §5) */
    struct ucred cred;
    socklen_t cred_len = sizeof(cred);
    if (getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &cred, &cred_len) == 0) {
        if (cred.uid != getuid()) {
            lc_log(LC_LOG_WARN, "IPC", "Peer UID mismatch (%d != %d); rejecting", (int)cred.uid, (int)getuid());
            close(fd);
            client->is_online = 0;
            client->fd = -1;
            return -1;
        }
    }
#endif

    /* Set non-blocking */
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags >= 0) {
        fcntl(fd, F_SETFL, flags | O_NONBLOCK);
    }

    client->fd = fd;
    client->is_online = 1;

    /* Handshake: send Hello, Subscribe, and GetSnapshot */
    char hello[256];
    uint32_t pid = (uint32_t)getpid();
    snprintf(hello, sizeof(hello),
             "{\"type\":\"Hello\",\"pid\":%u,\"abi\":1,\"version\":\"" TEXTVN_VERSION "\"}", pid);
    lc_ipc_send_frame(fd, hello, strlen(hello));

    char sub[128];
    snprintf(sub, sizeof(sub), "{\"type\":\"Subscribe\",\"pid\":%u}", pid);
    lc_ipc_send_frame(fd, sub, strlen(sub));

    const char *snap_req = "{\"type\":\"GetSnapshot\"}";
    lc_ipc_send_frame(fd, snap_req, strlen(snap_req));

    return 0;
#else
    return -1;
#endif
}

lc_ipc_client *lc_ipc_client_new(const char *app_id, const char *custom_sock_path) {
    lc_ipc_client *c = (lc_ipc_client *)calloc(1, sizeof(lc_ipc_client));
    if (!c) return NULL;

    c->fd = -1;
    c->is_online = 0;
    c->latest_config_version = 0;
    c->last_handled_config_version = 0;
    c->has_app_override = 0;
    c->app_enabled_override = 1;

    if (app_id) {
        strncpy(c->app_id, app_id, sizeof(c->app_id) - 1);
    }

    lc_ipc_resolve_socket_path(c->sock_path, sizeof(c->sock_path), custom_sock_path);

    /* Attempt initial non-blocking connection */
    try_connect(c);

    return c;
}

void lc_ipc_client_free(lc_ipc_client *client) {
    if (!client) return;
    if (client->fd >= 0) {
#if defined(__linux__) || defined(__unix__)
        close(client->fd);
#endif
        client->fd = -1;
    }
    free(client);
}

int lc_ipc_client_is_online(const lc_ipc_client *client) {
    return client ? client->is_online : 0;
}

int lc_ipc_client_poll(lc_ipc_client *client) {
    if (!client) return -1;

#if defined(__linux__) || defined(__unix__)
    /* Cooldown reconnection if offline (every 3000ms) */
    if (!client->is_online || client->fd < 0) {
        uint64_t now = get_time_ms();
        if (now - client->last_connect_attempt_ms >= 3000) {
            try_connect(client);
        }
        if (!client->is_online) return 0;
    }

    int msgs_processed = 0;
    static char buf[LC_IPC_MAX_FRAME + 1];

    while (1) {
        fd_set read_fds;
        FD_ZERO(&read_fds);
        FD_SET(client->fd, &read_fds);

        struct timeval tv = {0, 0}; /* non-blocking poll */
        int sel = select(client->fd + 1, &read_fds, NULL, NULL, &tv);
        if (sel <= 0) break;

        size_t len = 0;
        if (lc_ipc_recv_frame(client->fd, buf, sizeof(buf), &len) != 0) {
            /* Disconnected */
            close(client->fd);
            client->fd = -1;
            client->is_online = 0;
            break;
        }

        handle_ipc_message(client, buf);
        msgs_processed++;
    }
    return msgs_processed;
#else
    return 0;
#endif
}

int lc_ipc_client_check_config_reload(lc_ipc_client *client, uint64_t *out_version) {
    if (!client) return 0;
    lc_ipc_client_poll(client);

    if (client->latest_config_version > client->last_handled_config_version) {
        client->last_handled_config_version = client->latest_config_version;
        if (out_version) *out_version = client->latest_config_version;
        return 1;
    }
    return 0;
}

int lc_ipc_client_get_app_override(const lc_ipc_client *client, int *out_enabled) {
    if (!client) return 0;
    if (client->has_app_override) {
        if (out_enabled) *out_enabled = client->app_enabled_override;
        return 1;
    }
    return 0;
}

int lc_ipc_client_toggle_vi_en(lc_ipc_client *client, const char *app_id, int enabled) {
    if (!client || client->fd < 0 || !client->is_online) return -1;

    char msg[256];
    snprintf(msg, sizeof(msg),
             "{\"type\":\"ToggleViEn\",\"app_id\":\"%s\",\"enabled\":%s}",
             app_id ? app_id : client->app_id,
             enabled ? "true" : "false");

    return lc_ipc_send_frame(client->fd, msg, strlen(msg));
}
