/* linux_common.h — Common utilities for Linux IME adapters (IBus, Fcitx5, X11)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: docs/40-linux/P3-0-MASTER-PLAN.md, P3-2-fcitx5.md, P3-4-strategy-appdb.md
 * Linkable from C (IBus), C++ (Fcitx5), and Rust (X11 via bindgen).
 */

#ifndef TEXTVN_LINUX_COMMON_H
#define TEXTVN_LINUX_COMMON_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include "textvn_ffi.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Constants ---- */
#define LC_MAX_APP_ID       128
#define LC_MAX_ELEMENT_NAME 128
#define LC_MAX_LOG_MSG      512
#define LC_IPC_MAX_FRAME    65536
#define LC_SOCK_DEFAULT_REL ".config/TextVN/ipc.sock"
#define LC_LOG_DEFAULT_REL  ".local/state/TextVN/log/textvn.log"

/* Log levels (Rule S2: DO NOT log typed characters or keys) */
#define LC_LOG_DEBUG 0
#define LC_LOG_INFO  1
#define LC_LOG_WARN  2
#define LC_LOG_ERROR 3

/* Linux input frameworks */
#define LC_FRAMEWORK_UNKNOWN 0
#define LC_FRAMEWORK_IBUS    1
#define LC_FRAMEWORK_FCITX5  2
#define LC_FRAMEWORK_X11     3

/* ---- Field Detection C-ABI ---- */
typedef struct lc_field_ctx {
    char     app_id[LC_MAX_APP_ID];
    char     element_name[LC_MAX_ELEMENT_NAME];
    uint32_t field_role;       /* IME_FIELD_* from textvn_ffi.h */
    uint32_t secure;           /* 1 => password/secure (engine PASS), 0 otherwise */
    int64_t  strategy_hint;    /* IME_STRATEGY_* or -1 (auto resolve) */
} lc_field_ctx;

/**
 * Pure rule classifier for text field context.
 * Useful for unit testing, mock harnesses, and AT-SPI evaluation.
 *
 * Rules (P3-4 §2):
 * - R1: PASSWORD_TEXT / state password -> IME_FIELD_SECURE, secure=1, IME_STRATEGY_PASSTHROUGH
 * - R2: ENTRY + address/url in name/desc -> IME_FIELD_ADDRESS_BAR, IME_STRATEGY_SELECTION_REPLACE
 * - R3: ENTRY + search in name/desc -> IME_FIELD_SEARCH, IME_STRATEGY_SELECTION_REPLACE
 * - R4: COMBO_BOX -> IME_FIELD_COMBO, IME_STRATEGY_SELECTION_REPLACE
 * - R5: TABLE cell (Calc/Excel) -> IME_FIELD_CANDIDATE, IME_STRATEGY_SELECTION_REPLACE
 * - R6: TERMINAL -> IME_FIELD_TERMINAL, IME_STRATEGY_FORWARD_AS_COMMIT
 * - R7: Chat app input (Slack, Discord, Telegram, Zalo) -> IME_FIELD_BODY, IME_STRATEGY_BACKSPACE_TYPE (NonPreedit)
 * - R8: TEXT/SECTION editable -> IME_FIELD_TEXTAREA, IME_STRATEGY_PREEDIT
 * - R9: DOCUMENT_WEB -> IME_FIELD_WEB, IME_STRATEGY_PREEDIT
 * - R10: Generic ENTRY -> IME_FIELD_EDITBOX, IME_STRATEGY_PREEDIT
 * - Default: IME_FIELD_UNKNOWN, strategy_hint = -1
 */
int lc_classify_field(const char *app_id,
                      const char *role_name,
                      uint64_t state_flags,
                      const char *name,
                      const char *description,
                      lc_field_ctx *out);

/**
 * Detect active window and focused field role via AT-SPI.
 * Uses internal 2-second TTL cache to satisfy < 2ms latency budget (P3-4 §2, §3).
 * Falls back safely to preset by app_id if AT-SPI is unavailable or permission denied.
 */
int lc_field_detect(lc_field_ctx *out);

/**
 * Invalidate field detection cache upon window activation or focus switch.
 */
void lc_field_detect_invalidate_cache(void);

/* ---- IPC Client (Unix Domain Socket) ---- */
typedef struct lc_ipc_snapshot {
    uint64_t config_version;
    uint32_t appdb_version;
    char     channel[32];
    int      has_override;
    int      app_enabled;      /* Valid when has_override == 1 */
} lc_ipc_snapshot;

typedef struct lc_ipc_client lc_ipc_client;

/**
 * Create a new IPC client connecting to TextVN Tray.
 * Fallback offline when tray is not yet running (non-blocking).
 * Pass custom_sock_path = NULL to use default ~/.config/TextVN/ipc.sock.
 */
lc_ipc_client *lc_ipc_client_new(const char *app_id, const char *custom_sock_path);

/**
 * Destroy IPC client and close socket connection.
 */
void lc_ipc_client_free(lc_ipc_client *client);

/**
 * Non-blocking poll for incoming IPC messages (ConfigReload, StateUpdate, Snapshot).
 * Returns number of messages processed, or -1 on error.
 */
int lc_ipc_client_poll(lc_ipc_client *client);

/**
 * Check if IPC client is connected to tray.
 * Returns 1 if online, 0 if offline/fallback.
 */
int lc_ipc_client_is_online(const lc_ipc_client *client);

/**
 * Check if a new config version was broadcast from tray.
 * Returns 1 if new config version received (stored in *out_version), 0 otherwise.
 */
int lc_ipc_client_check_config_reload(lc_ipc_client *client, uint64_t *out_version);

/**
 * Check if tray has an enable/disable override for the given app.
 * Returns 1 if override present (stored in *out_enabled), 0 if no override.
 */
int lc_ipc_client_get_app_override(const lc_ipc_client *client, int *out_enabled);

/**
 * Vong 13 (BUG-03): doc trang thai toan cuc "app_id":"*" do tray broadcast
 * (StateUpdate va Snapshot). Returns 1 khi da nhan it nhat MOT broadcast
 * toan cuc; *out_seq tang moi broadcast — engine so sanh voi seq tieu thu
 * lan truoc de ap dung dung MOT lan su kien (khong sticky, tranh xung dot
 * voi kenh state.json).
 */
int lc_ipc_client_get_global_override(const lc_ipc_client *client, int *out_enabled,
                                      uint32_t *out_seq);

/**
 * Send ToggleViEn request to tray.
 */
int lc_ipc_client_toggle_vi_en(lc_ipc_client *client, const char *app_id, int enabled);

/**
 * Resolve Unix domain socket path: expands ~ to $HOME or $XDG_CONFIG_HOME.
 */
int lc_ipc_resolve_socket_path(char *out_path, size_t max_len, const char *custom_path);

/* Low-level framing helpers (u32 little-endian length + JSON payload <= 64KB) */
int lc_ipc_send_frame(int fd, const char *json_utf8, size_t len);
int lc_ipc_recv_frame(int fd, char *out_buf, size_t buf_size, size_t *out_len);

/* ---- Logging (Standardized, Rule S2 Compliant) ---- */
/**
 * Initialize logger.
 * Creates ~/.local/state/TextVN/log/ if needed.
 */
void lc_log_init(const char *app_tag);

/**
 * Log a message.
 * MANDATORY: DO NOT pass user keystrokes, passwords, or typed text (Rule S2 / SECURITY.md).
 */
void lc_log(int level, const char *tag, const char *fmt, ...);

/**
 * Close log file handle.
 */
void lc_log_close(void);

/* ---- Environment & Framework Detection ---- */
typedef struct lc_env_info {
    char gtk_im_module[64];
    char qt_im_module[64];
    char xmodifiers[64];
    char xdg_current_desktop[64];
    int  detected_framework; /* LC_FRAMEWORK_* */
    int  is_consistent;       /* 1 if all env vars align with detected framework */
} lc_env_info;

/**
 * Inspect environment variables (GTK_IM_MODULE, QT_IM_MODULE, XMODIFIERS)
 * and generate diagnostic report for `textvn doctor` (P3-4 §7).
 */
int lc_env_check(lc_env_info *out_info, char *out_report, size_t report_len);

/**
 * Auto-detect active framework (IBus vs Fcitx5 vs X11) based on desktop and env.
 */
int lc_detect_active_framework(void);

/* ---- UTF-32 / UTF-8 Conversion Helpers ---- */
/**
 * Convert UTF-32 buffer (from ime_result_v1) to UTF-8 null-terminated string.
 * Returns bytes written (excluding null terminator).
 */
size_t lc_utf32_to_utf8(const uint32_t *src, size_t src_len, char *dst, size_t dst_len);

/**
 * Convert UTF-8 string to UTF-32 buffer (for ime_key_v1 or suggest).
 * Returns code points written.
 */
size_t lc_utf8_to_utf32(const char *src, size_t src_len, uint32_t *dst, size_t max_dst);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_LINUX_COMMON_H */
