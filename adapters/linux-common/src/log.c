/* log.c — Standardized logging for Linux adapters (S2 Compliant)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * RULE S2 / SECURITY.md:
 * NEVER log user keystrokes, character content, or passwords.
 * Only log lifecycle, operational status, error codes, and sanitized info.
 */

/* C9 (audit): localtime_r/mkdir can khai bao khi bat feature-test macro POSIX.
 * Khai bao truoc moi #include de glibc lo dien khai bao. */
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif

#include "linux_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <time.h>

#if defined(__linux__) || defined(__unix__)
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#endif

static FILE *s_log_file = NULL;
static char  s_app_tag[64] = "TextVN";

static void ensure_parent_dirs(const char *file_path) {
#if defined(__linux__) || defined(__unix__)
    char path[512];
    /* C9 (audit): strncpy am tham cat cut ma khong bao — neu duong dan dai hon
     * buffer thi mkdir se tao thu muc SAI (bi cat). Tu choi som khi tran. */
    size_t n = strlen(file_path);
    if (n >= sizeof(path)) return;
    memcpy(path, file_path, n + 1);

    char *slash = strrchr(path, '/');
    if (!slash) return;
    *slash = '\0';

    for (char *p = path + 1; *p; ++p) {
        if (*p == '/') {
            *p = '\0';
            mkdir(path, 0700);
            *p = '/';
        }
    }
    mkdir(path, 0700);
#else
    (void)file_path;
#endif
}

void lc_log_init(const char *app_tag) {
    if (app_tag && app_tag[0] != '\0') {
        /* C9 (audit): cat theo sizeof-1 roi tu ket thuc — tranh am tham cat ma
         * khong co NUL. */
        strncpy(s_app_tag, app_tag, sizeof(s_app_tag) - 1);
        s_app_tag[sizeof(s_app_tag) - 1] = '\0';
    }

    if (s_log_file) return;

    char log_path[512];
    const char *xdg_state = getenv("XDG_STATE_HOME");
    /* C9 (audit): XDG_STATE_HOME phai la duong dan TUYET DOI (spec). Gia tri
     * tuong doi se lam log ghi theo CWD -> doi hoi bat dau bang '/'. */
    if (xdg_state && xdg_state[0] == '/') {
        int w = snprintf(log_path, sizeof(log_path), "%s/TextVN/log/textvn.log", xdg_state);
        if (w < 0 || (size_t)w >= sizeof(log_path)) return;
    } else {
        const char *home = getenv("HOME");
        if (home && home[0] == '/') {
            int w = snprintf(log_path, sizeof(log_path), "%s/%s", home, LC_LOG_DEFAULT_REL);
            if (w < 0 || (size_t)w >= sizeof(log_path)) return;
        } else {
            /* Không có HOME: KHÔNG ghi vào /tmp dùng chung (symlink attack) — bỏ log. */
            return;
        }
    }

    ensure_parent_dirs(log_path);
    s_log_file = fopen(log_path, "a");
}

void lc_log(int level, const char *tag, const char *fmt, ...) {
    static const char *level_names[] = {"DEBUG", "INFO", "WARN", "ERROR"};
    const char *lvl_str = (level >= 0 && level <= 3) ? level_names[level] : "INFO";

    /* Format timestamp */
    char time_str[32] = {0};
    time_t now = time(NULL);
    /* C9 (audit): localtime() tra ve con tro vao buffer tinh (khong reentrant).
     * Adapter co the goi lc_log tu nhieu thread -> dung localtime_r neu co. */
#if defined(__linux__) || defined(__unix__)
    struct tm tm_buf;
    if (localtime_r(&now, &tm_buf)) {
        strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", &tm_buf);
    }
#else
    struct tm *tm_info = localtime(&now);
    if (tm_info) {
        strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", tm_info);
    }
#endif

    va_list args;
    va_start(args, fmt);

    char msg[LC_MAX_LOG_MSG];
    vsnprintf(msg, sizeof(msg), fmt, args);
    va_end(args);

    /* Output to log file if open */
    if (s_log_file) {
        fprintf(s_log_file, "[%s] [%s] [%s:%s] %s\n",
                time_str, lvl_str, s_app_tag, tag ? tag : "core", msg);
        fflush(s_log_file);
    }

    /* Output errors to stderr */
    if (level == LC_LOG_ERROR) {
        fprintf(stderr, "[%s] [%s:%s] %s\n", lvl_str, s_app_tag, tag ? tag : "core", msg);
    }
}

void lc_log_close(void) {
    if (s_log_file) {
        fclose(s_log_file);
        s_log_file = NULL;
    }
}
