/*
 * e2e_fcitx5.c — Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend
 * (org.fcitx.Fcitx.InputMethod1) — bản C thay e2e_fcitx5.py (E59: policy
 * cấm Python trong repo; precedent e2e_ibus.c dùng lib IBus C cho phía app).
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Chạy qua scripts/e2e-linux.sh: fcitx5 nạp libtextvn-fcitx5.so từ thư mục
 * build. Đóng vai app: tạo input context, gửi phím, nhận CommitString /
 * UpdateFormattedPreedit; phím engine không nuốt thì "app" tự xử lý.
 *
 * Client libdbus thuần (không GLib main loop): pump() tự đọc I/O + dispatch
 * đưa signal về filter đã đăng ký.
 */

#include <dbus/dbus.h>
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

/* ---- Hằng số phím/trạng thái — dịch 1:1 từ e2e_fcitx5.py ---- */
#define KEY_ENTER 0xFF0D
#define KEY_BACKSPACE 0xFF08
#define KEY_TAB 0xFF09
#define KEY_SPACE 0x20
#define CTRL_L 0xFFE3
#define SHIFT_L 0xFFE1
#define STATE_SHIFT (1u << 0)
#define STATE_CAPS (1u << 1)
#define STATE_CTRL (1u << 2)
#define CAP_PREEDIT (1u << 1)
#define CAP_FORMATTED (1u << 4)
#define CAP_PASSWORD (1u << 3)
#define CAP_UNFOCUS_COMMIT (1u << 5)

#define FCITX_NAME "org.fcitx.Fcitx5"
#define IM_IFACE "org.fcitx.Fcitx.InputMethod1"
#define IC_IFACE "org.fcitx.Fcitx.InputContext1"
#define CTRL_IFACE "org.fcitx.Fcitx.Controller1"
#define IM_PATH "/org/freedesktop/portal/inputmethod"
#define CTRL_PATH "/controller"
#define CALL_TIMEOUT_MS 5000

/* ---- Bộ đệm chuỗi nhỏ (thay GString — không kéo GLib vào client) ---- */
typedef struct {
    char *data;
    size_t len;
    size_t cap;
} SB;

static void sb_reserve(SB *sb, size_t need) {
    if (sb->cap == 0) {
        sb->cap = 256;
        sb->data = malloc(sb->cap);
        if (!sb->data) {
            fprintf(stderr, "e2e_fcitx5: OOM\n");
            exit(1);
        }
        sb->data[0] = '\0';
        sb->len = 0;
    }
    if (sb->len + need + 1 > sb->cap) {
        while (sb->len + need + 1 > sb->cap)
            sb->cap *= 2;
        sb->data = realloc(sb->data, sb->cap);
        if (!sb->data) {
            fprintf(stderr, "e2e_fcitx5: OOM\n");
            exit(1);
        }
    }
}

static void sb_append(SB *sb, const char *s) {
    size_t n = strlen(s);
    sb_reserve(sb, n + 1);
    memcpy(sb->data + sb->len, s, n + 1);
    sb->len += n;
}

static void sb_clear(SB *sb) {
    sb_reserve(sb, 1);
    sb->len = 0;
    sb->data[0] = '\0';
}

static void sb_backspace(SB *sb) {
    /* Xoá 1 ký tự UTF-8 cuối (lùi qua các byte tiếp tín 0b10xxxxxx). */
    if (sb->len == 0)
        return;
    size_t i = sb->len - 1;
    while (i > 0 && (sb->data[i] & 0xC0) == 0x80)
        i--;
    sb->len = i;
    sb->data[i] = '\0';
}

/* ---- Trạng thái toàn cục của "app" ---- */
static DBusConnection *conn = NULL;
static SB doc = {0};
static SB preedit = {0};
static char ic_path[512] = "";
static int ok = 1;

static double now_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + ts.tv_nsec / 1e9;
}

static void die(const char *where, const char *msg) {
    fprintf(stderr, "e2e_fcitx5 FATAL: %s: %s\n", where, msg);
    exit(1);
}

static void die_dbus(const char *where, DBusError *err) {
    fprintf(stderr, "e2e_fcitx5 FATAL: %s: %s (%s)\n", where, err->message,
            err->name ? err->name : "-");
    exit(1);
}

/* R2-165: mỗi lần ProcessKeyEvent tối đa 1 commit (text-input-v3 chỉ giữ lệnh cuối mỗi
 * `done`); R2-166: phím engine gửi lại (ForwardKey) phải cân nhấn/nhả. */
static unsigned commits_this_key = 0, max_commits_per_key = 0;
static unsigned fwd_press = 0, fwd_release = 0;
/* R2-167: preedit đang hiện ĐÚNG lúc CommitString tới (commit trước, xoá preedit sau). */
static char preedit_at_commit[256] = "";

/* ---- Signal filter: CommitString + UpdateFormattedPreedit ---- */
static DBusHandlerResult on_signal(DBusConnection *c, DBusMessage *m,
                                   void *user_data) {
    (void)c;
    (void)user_data;
    if (dbus_message_get_type(m) != DBUS_MESSAGE_TYPE_SIGNAL)
        return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
    const char *iface = dbus_message_get_interface(m);
    if (!iface || strcmp(iface, IC_IFACE) != 0)
        return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
    const char *path = dbus_message_get_path(m);
    if (!path || strcmp(path, ic_path) != 0)
        return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
    const char *member = dbus_message_get_member(m);
    if (!member)
        return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;

    DBusMessageIter iter;
    if (!dbus_message_iter_init(m, &iter))
        return DBUS_HANDLER_RESULT_HANDLED;

    if (strcmp(member, "CommitString") == 0) {
        if (dbus_message_iter_get_arg_type(&iter) == DBUS_TYPE_STRING) {
            const char *text = NULL;
            dbus_message_iter_get_basic(&iter, &text);
            snprintf(preedit_at_commit, sizeof preedit_at_commit, "%s",
                     preedit.data ? preedit.data : "");
            if (text)
                sb_append(&doc, text);
            commits_this_key++;
        } else {
            char *sig = dbus_message_iter_get_signature(&iter);
            fprintf(stderr, "CommitString signature lạ: %s\n", sig ? sig : "?");
            dbus_free(sig);
        }
        return DBUS_HANDLER_RESULT_HANDLED;
    }
    if (strcmp(member, "ForwardKey") == 0) {
        /* (u keysym, u state, b isRelease) */
        dbus_uint32_t sym = 0, st = 0;
        dbus_bool_t rel = FALSE;
        DBusError e;
        dbus_error_init(&e);
        if (dbus_message_get_args(m, &e, DBUS_TYPE_UINT32, &sym, DBUS_TYPE_UINT32, &st,
                                  DBUS_TYPE_BOOLEAN, &rel, DBUS_TYPE_INVALID)) {
            if (rel)
                fwd_release++;
            else
                fwd_press++;
        } else {
            dbus_error_free(&e);
        }
        return DBUS_HANDLER_RESULT_HANDLED;
    }
    if (strcmp(member, "UpdateFormattedPreedit") == 0) {
        /* a(si)… — nối chuỗi của từng đoạn (int32 sau mỗi string bị bỏ qua). */
        sb_clear(&preedit);
        if (dbus_message_iter_get_arg_type(&iter) == DBUS_TYPE_ARRAY) {
            DBusMessageIter arr;
            dbus_message_iter_recurse(&iter, &arr);
            while (dbus_message_iter_get_arg_type(&arr) == DBUS_TYPE_STRUCT) {
                DBusMessageIter stru;
                dbus_message_iter_recurse(&arr, &stru);
                if (dbus_message_iter_get_arg_type(&stru) == DBUS_TYPE_STRING) {
                    const char *text = NULL;
                    dbus_message_iter_get_basic(&stru, &text);
                    if (text)
                        sb_append(&preedit, text);
                }
                dbus_message_iter_next(&arr);
            }
        }
        return DBUS_HANDLER_RESULT_HANDLED;
    }
    return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

/* pump(sec) — như bản python: xử lý DBus trong `sec` giây. */
static void pump(double sec) {
    double end = now_sec() + sec;
    while (now_sec() < end) {
        while (dbus_connection_dispatch(conn) == DBUS_DISPATCH_DATA_REMAINS) {
        }
        dbus_connection_read_write(conn, 5); /* chờ I/O tối đa 5ms */
        usleep(5000);
    }
    while (dbus_connection_dispatch(conn) == DBUS_DISPATCH_DATA_REMAINS) {
    }
}

/* ---- Lời gọi method đồng bộ — trả về reply (đã mở) hoặc die(). ---- */
static DBusMessage *method_call(const char *path, const char *iface,
                                const char *method, dbus_int32_t first_arg_type,
                                ...) {
    DBusMessage *msg =
        dbus_message_new_method_call(FCITX_NAME, path, iface, method);
    if (!msg)
        die(method, "không tạo được method call (OOM?)");
    va_list ap;
    va_start(ap, first_arg_type);
    if (!dbus_message_append_args_valist(msg, first_arg_type, ap))
        die(method, "không ráp được args");
    va_end(ap);
    DBusError err;
    dbus_error_init(&err);
    DBusMessage *reply =
        dbus_connection_send_with_reply_and_block(conn, msg, CALL_TIMEOUT_MS, &err);
    dbus_message_unref(msg);
    if (dbus_error_is_set(&err))
        die_dbus(method, &err);
    if (!reply)
        die(method, "không có reply");
    return reply;
}

/* Không đối số, không cần đọc reply. */
static void method_void(const char *path, const char *iface, const char *method) {
    DBusMessage *reply = method_call(path, iface, method, DBUS_TYPE_INVALID);
    if (reply)
        dbus_message_unref(reply);
}

static void ic_set_capability(dbus_uint64_t caps) {
    DBusMessage *reply = method_call(ic_path, IC_IFACE, "SetCapability",
                                     DBUS_TYPE_UINT64, &caps, DBUS_TYPE_INVALID);
    if (reply)
        dbus_message_unref(reply);
}

static dbus_bool_t ic_process_key(dbus_uint32_t keysym, dbus_uint32_t state,
                                   dbus_bool_t release) {
    dbus_uint32_t keycode = 0;
    dbus_uint32_t time = 0;
    DBusMessage *reply =
        method_call(ic_path, IC_IFACE, "ProcessKeyEvent", DBUS_TYPE_UINT32,
                    &keysym, DBUS_TYPE_UINT32, &keycode, DBUS_TYPE_UINT32,
                    &state, DBUS_TYPE_BOOLEAN, &release, DBUS_TYPE_UINT32,
                    &time, DBUS_TYPE_INVALID);
    dbus_bool_t handled = FALSE;
    DBusError err;
    dbus_error_init(&err);
    if (!dbus_message_get_args(reply, &err, DBUS_TYPE_BOOLEAN, &handled,
                               DBUS_TYPE_INVALID))
        die_dbus("ProcessKeyEvent", &err);
    dbus_message_unref(reply);
    return handled;
}

static void ctrl_set_current_im(const char *name) {
    DBusMessage *reply = method_call(CTRL_PATH, CTRL_IFACE, "SetCurrentIM",
                                     DBUS_TYPE_STRING, &name, DBUS_TYPE_INVALID);
    if (reply)
        dbus_message_unref(reply);
}

/* Gọi method CurrentInputMethod() trên org.fcitx.Fcitx.Controller1 (1:1 bản Python:
 * `controller.CurrentInputMethod()`). Đây là DBus method trả (s), KHÔNG phải Properties.Get. */
static char *ctrl_current_im(void) {
    DBusMessage *reply =
        method_call(CTRL_PATH, CTRL_IFACE, "CurrentInputMethod", DBUS_TYPE_INVALID);
    static char cur[128];
    cur[0] = '\0';
    const char *s = NULL;
    DBusError err;
    dbus_error_init(&err);
    if (!dbus_message_get_args(reply, &err, DBUS_TYPE_STRING, &s,
                               DBUS_TYPE_INVALID))
        die_dbus("CurrentInputMethod", &err);
    if (s)
        snprintf(cur, sizeof cur, "%s", s);
    dbus_message_unref(reply);
    return cur;
}

/* ---- Nhấn phím như "app" nhìn thấy (echo khi engine không nuốt) ---- */
static void app_handles(const char *ch_utf8) {
    if (strcmp(ch_utf8, "\b") == 0) {
        sb_backspace(&doc);
    } else {
        sb_append(&doc, ch_utf8);
    }
}

static void pump(double sec);

static dbus_bool_t send_key(dbus_uint32_t keysym, dbus_uint32_t state,
                            dbus_bool_t release) {
    /* Đếm commit của RIÊNG phím này: signal tới trước reply nên pump ngắn là đủ. */
    pump(0.0);
    commits_this_key = 0;
    dbus_bool_t handled = ic_process_key(keysym, state, release);
    pump(0.02);
    if (commits_this_key > max_commits_per_key)
        max_commits_per_key = commits_this_key;
    return handled;
}

static void check(const char *label, const char *want) {
    SB got = {0};
    sb_reserve(&got, doc.len + preedit.len + 1);
    memcpy(got.data, doc.data, doc.len);
    memcpy(got.data + doc.len, preedit.data, preedit.len);
    got.data[doc.len + preedit.len] = '\0';
    int good = strcmp(got.data, want) == 0;
    if (!good)
        ok = 0;
    printf("%s %-40s want=\"%s\" got=\"%s\"\n", good ? "PASS" : "FAIL", label,
           want, got.data);
    free(got.data);
}

static void clear(void) {
    method_void(ic_path, IC_IFACE, "Reset");
    pump(0.1);
    sb_clear(&doc);
    sb_clear(&preedit);
}

static void ctrl_tap(void) {
    send_key(CTRL_L, 0, FALSE);
    pump(0.05);
    send_key(CTRL_L, 0, TRUE);
    pump(0.05);
}

static void ctrl_shift_tap(void) {
    send_key(CTRL_L, 0, FALSE);
    pump(0.05);
    send_key(SHIFT_L, STATE_CTRL, FALSE);
    pump(0.05);
    send_key(SHIFT_L, STATE_CTRL | STATE_SHIFT, TRUE);
    pump(0.05);
    send_key(CTRL_L, STATE_CTRL, TRUE);
    pump(0.05);
}

/* Keysym của 1 ký tự (KEY map cho điều khiển, ngược lại = codepoint UCS). */
static void keysym_of(const char *utf8p, dbus_uint32_t *out_keysym,
                      const char **next) {
    unsigned char c0 = (unsigned char)utf8p[0];
    if (c0 == '\n' || c0 == '\r') {
        *out_keysym = KEY_ENTER;
        *next = utf8p + 1;
    } else if (c0 == '\b' || c0 == 0x7F) {
        *out_keysym = KEY_BACKSPACE;
        *next = utf8p + 1;
    } else if (c0 == '\t') {
        *out_keysym = KEY_TAB;
        *next = utf8p + 1;
    } else if (c0 == ' ') {
        *out_keysym = KEY_SPACE;
        *next = utf8p + 1;
    } else if (c0 < 0x80) {
        *out_keysym = c0;
        *next = utf8p + 1;
    } else {
        /* UTF-8 → codepoint (keysym của ký tự ngoài ASCII = UCS-4). */
        dbus_uint32_t cp = 0;
        int n = 0;
        if ((c0 & 0xE0) == 0xC0) { cp = c0 & 0x1F; n = 1; }
        else if ((c0 & 0xF0) == 0xE0) { cp = c0 & 0x0F; n = 2; }
        else if ((c0 & 0xF8) == 0xF0) { cp = c0 & 0x07; n = 3; }
        else { cp = '?'; n = 0; }
        int consumed = 1;
        for (int i = 1; i <= n; i++) {
            unsigned char ci = (unsigned char)utf8p[i];
            if ((ci & 0xC0) == 0x80) {
                cp = (cp << 6) | (ci & 0x3F);
                consumed++;
            } else {
                break;
            }
        }
        *out_keysym = cp;
        *next = utf8p + consumed;
    }
}

static void press_char(const char *ch_utf8, dbus_uint32_t state) {
    dbus_uint32_t keysym;
    const char *next;
    keysym_of(ch_utf8, &keysym, &next);
    dbus_bool_t handled = send_key(keysym, state, FALSE);
    pump(0.05);
    if (!handled) {
        if (preedit.len > 0) {
            printf("FAIL B2 vi phạm: app nhận phím khi preedit còn mở: \"%s\"\n",
                   preedit.data);
            ok = 0;
        }
        app_handles(ch_utf8);
    }
    send_key(keysym, state, TRUE);
    pump(0.05);
}

static void press_char_range(const char *s, const char *end) {
    while (s < end) {
        /* 1 ký tự UTF-8:lùi qua các byte 0b10xxxxxx. */
        size_t n = 1;
        if (((unsigned char)s[0] & 0xE0) == 0xC0) n = 2;
        else if (((unsigned char)s[0] & 0xF0) == 0xE0) n = 3;
        else if (((unsigned char)s[0] & 0xF8) == 0xF0) n = 4;
        /* Chữ hoa ASCII: kèm Shift giống bản python (ch.isupper()). */
        unsigned char c0 = (unsigned char)s[0];
        dbus_uint32_t state = (c0 >= 'A' && c0 <= 'Z') ? STATE_SHIFT : 0;
        char ch[5];
        if (n > (size_t)(end - s))
            n = (size_t)(end - s);
        memcpy(ch, s, n);
        ch[n] = '\0';
        press_char(ch, state);
        s += n;
    }
}

static void type_string(const char *s) {
    press_char_range(s, s + strlen(s));
}

/* ---- Tệp cấu hình/state — như bảng cài đặt: tmp → rename ---- */
static char config_dir[512];

static void mkdir_p(const char *dir) {
    char tmp[512];
    snprintf(tmp, sizeof tmp, "%s", dir);
    size_t len = strlen(tmp);
    while (len > 1 && tmp[len - 1] == '/')
        tmp[--len] = '\0';
    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(tmp, 0755) != 0 && errno != EEXIST)
                die("mkdir_p", strerror(errno));
            *p = '/';
        }
    }
    if (mkdir(tmp, 0755) != 0 && errno != EEXIST)
        die("mkdir_p", strerror(errno));
}

static void build_config_dir(void) {
    const char *xdg = getenv("XDG_CONFIG_HOME");
    if (xdg && xdg[0])
        snprintf(config_dir, sizeof config_dir, "%s/TextVN", xdg);
    else {
        const char *home = getenv("HOME");
        if (!home)
            die("config", "không có XDG_CONFIG_HOME/HOME");
        snprintf(config_dir, sizeof config_dir, "%s/.config/TextVN", home);
    }
    mkdir_p(config_dir);
}

static void write_textvn_file(const char *name, const char *json) {
    mkdir_p(config_dir);
    /* Chờ cố định 20ms giống Python time.sleep(0.02) để mtime coarse-tick Linux chắc chắn đổi. */
    usleep(20000);
    char tmp[600], dst[600];
    snprintf(tmp, sizeof tmp, "%s/.%s.e2e.tmp", config_dir, name);
    snprintf(dst, sizeof dst, "%s/%s", config_dir, name);
    FILE *f = fopen(tmp, "w");
    if (!f)
        die(name, "không mở được tệp tmp");
    fputs(json, f);
    fclose(f);
    if (rename(tmp, dst) != 0)
        die(name, strerror(errno));
}

/* state.json global_enabled == enabled? (thiếu file → None → fail). */
static void state_says(int enabled) {
    char path[600];
    snprintf(path, sizeof path, "%s/state.json", config_dir);
    FILE *f = fopen(path, "r");
    int found = -1;
    if (f) {
        static char buf[4096];
        size_t n = fread(buf, 1, sizeof buf - 1, f);
        buf[n] = '\0';
        fclose(f);
        const char *key = strstr(buf, "\"global_enabled\"");
        if (key) {
            const char *scan = key + strlen("\"global_enabled\"");
            const char *end = scan + 40;
            while (scan < end && *scan) {
                if (strncmp(scan, "true", 4) == 0) { found = 1; break; }
                if (strncmp(scan, "false", 5) == 0) { found = 0; break; }
                scan++;
            }
        }
    }
    int good = found == enabled;
    if (!good)
        ok = 0;
    printf("%s state.json global_enabled=%s\n", good ? "PASS" : "FAIL",
           enabled ? "true" : "false");
}

int main(void) {
    DBusError err;
    dbus_error_init(&err);
    conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
    if (dbus_error_is_set(&err))
        die_dbus("dbus_bus_get", &err);
    if (!conn)
        die("dbus_bus_get", "NULL connection");
    dbus_connection_add_filter(conn, on_signal, NULL, NULL);

    /* fcitx5 phải đã lên DBus (e2e-linux.sh tiến trình cha lo). */
    int up = 0;
    for (int i = 0; i < 100 && !up; i++) {
        up = dbus_bus_name_has_owner(conn, FCITX_NAME, NULL);
        if (!up)
            usleep(100000);
    }
    if (!up)
        die("wait_service", "fcitx5 không lên DBus");

    /* CreateInputContext: (a(ss)) → (o, s). PORT 1:1 python:
     * im.CreateInputContext([("program", "textvn-e2e"), ("display", "x11:")])
     * — danh sách TUPLE (struct), KHÔNG phải dict a{ss}. */
    DBusMessage *msg =
        dbus_message_new_method_call(FCITX_NAME, IM_PATH, IM_IFACE,
                                     "CreateInputContext");
    if (!msg)
        die("CreateInputContext", "OOM dbus_message_new_method_call");
    {
        DBusMessageIter iter, arr;
        dbus_message_iter_init_append(msg, &iter);
        if (!dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "(ss)", &arr))
            die("CreateInputContext", "open array");
        const char *pairs[][2] = {{"program", "textvn-e2e"}, {"display", "x11:"}};
        for (size_t i = 0; i < sizeof pairs / sizeof pairs[0]; i++) {
            DBusMessageIter stru;
            if (!dbus_message_iter_open_container(&arr, DBUS_TYPE_STRUCT,
                                                  NULL, &stru))
                die("CreateInputContext", "open struct (ss)");
            dbus_message_iter_append_basic(&stru, DBUS_TYPE_STRING, &pairs[i][0]);
            dbus_message_iter_append_basic(&stru, DBUS_TYPE_STRING, &pairs[i][1]);
            dbus_message_iter_close_container(&arr, &stru);
        }
        if (!dbus_message_iter_close_container(&iter, &arr))
            die("CreateInputContext", "close array");
    }
    DBusMessage *reply =
        dbus_connection_send_with_reply_and_block(conn, msg, CALL_TIMEOUT_MS, &err);
    dbus_message_unref(msg);
    if (dbus_error_is_set(&err))
        die_dbus("CreateInputContext", &err);
    if (!reply)
        die("CreateInputContext", "reply NULL");
    {
        const char *path = NULL;
        if (!dbus_message_get_args(reply, &err, DBUS_TYPE_OBJECT_PATH, &path,
                                   DBUS_TYPE_INVALID))
            die_dbus("CreateInputContext", &err);
        snprintf(ic_path, sizeof ic_path, "%s", path);
    }
    dbus_message_unref(reply);
    fprintf(stderr, "e2e_fcitx5: input context %s\n", ic_path);

    sb_reserve(&doc, 64);
    sb_reserve(&preedit, 64);

    ic_set_capability(CAP_PREEDIT | CAP_FORMATTED);
    method_void(ic_path, IC_IFACE, "FocusIn");
    pump(0.2);
    ctrl_set_current_im("textvn");
    pump(0.3);
    const char *cur = ctrl_current_im();
    if (strcmp(cur, "textvn") != 0) {
        fprintf(stderr, "e2e_fcitx5: không kích hoạt được IM textvn: %s\n", cur);
        return 1;
    }

    /* ---- Telex cơ bản + preedit + Enter + Backspace ---- */
    type_string("dduocj ");
    check("telex dduocj+space", "\xc4\x91\xc6\xb0\xe1\xbb\xa3" "c "); /* "được " */
    {
        /* R2-167 (ibus-bamboo#574): lúc commit tới app, preedit cũ vẫn còn. */
        int ordered = strcmp(preedit_at_commit, "\xc4\x91\xc6\xb0\xe1\xbb\xa3" "c") == 0;
        if (!ordered)
            ok = 0;
        printf("%s commit arrives before preedit is cleared (preedit at commit=\"%s\")\n",
               ordered ? "PASS" : "FAIL", preedit_at_commit);
    }
    clear();
    type_string("Vieetj Nam");
    check("preedit gi\xe1\xbb\xaf c\xe1\xba\xa3 t\xe1\xbb\xab",
          "Vi\xe1\xbb\x87t Nam"); /* "Việt Nam" */
    clear();
    type_string("chaof banj\n");
    check("Enter commit (B2)", "ch\xc3\xa0o b\xe1\xba\xa1n\n"); /* "chào bạn\n" */
    clear();
    type_string("dduocj\b ");
    check("Backspace trong t\xe1\xbb\xab",
          "\xc4\x91\xc6\xb0\xe1\xbb\xa3 "); /* "đượ " — R2-55 */
    clear();
    type_string("hello world ");
    check("ti\xe1\xba\xbfng Anh gi\xe1\xbb\xaf nguy\xc3\xaan", "hello world ");
    clear();

    /* ---- Focus-out commit ---- */
    type_string("tieengs");
    method_void(ic_path, IC_IFACE, "FocusOut");
    pump(0.2);
    check("focus-out commit", "ti\xe1\xba\xbfng"); /* "tiếng" */
    method_void(ic_path, IC_IFACE, "FocusIn");
    pump(0.2);
    clear();

    /* Client tự commit khi mất focus (ClientUnfocusCommit): không lặp. */
    ic_set_capability(CAP_PREEDIT | CAP_FORMATTED | CAP_UNFOCUS_COMMIT);
    pump(0.1);
    type_string("tieengs");
    sb_append(&doc, preedit.data); /* client commit preedit của nó */
    sb_clear(&preedit);
    method_void(ic_path, IC_IFACE, "FocusOut");
    pump(0.2);
    check("focus-out (client commit)", "ti\xe1\xba\xbfng"); /* "tiếng" */
    method_void(ic_path, IC_IFACE, "FocusIn");
    pump(0.2);
    clear();
    ic_set_capability(CAP_PREEDIT | CAP_FORMATTED);
    pump(0.1);

    /* ---- Reset chốt đúng 1 lần ---- */
    type_string("tieengs");
    method_void(ic_path, IC_IFACE, "Reset");
    pump(0.2);
    check("reset commit 1 l\xe1\xba\xa7n", "ti\xe1\xba\xbfng"); /* "tiếng" */
    sb_clear(&doc);
    sb_clear(&preedit);

    /* ---- Ctrl+Shift đổi VN/EN + state.json (R2-107: focus không lật lại) ---- */
    build_config_dir();
    ctrl_shift_tap();
    type_string("as ");
    check("Ctrl+Shift \xe2\x86\x92 EN", "as ");
    state_says(0);
    clear();
    method_void(ic_path, IC_IFACE, "FocusOut");
    pump(0.2);
    method_void(ic_path, IC_IFACE, "FocusIn");
    pump(0.2);
    type_string("as ");
    check("EN gi\xe1\xbb\xaf sau focus-out/in", "as ");
    clear();
    ctrl_shift_tap();
    type_string("as ");
    check("Ctrl+Shift \xe2\x86\x92 VN", "\xc3\xa1 ");
    state_says(1);
    clear();
    method_void(ic_path, IC_IFACE, "FocusOut");
    pump(0.2);
    method_void(ic_path, IC_IFACE, "FocusIn");
    pump(0.2);
    type_string("as ");
    check("VN gi\xe1\xbb\xaf sau focus-out/in", "\xc3\xa1 ");
    clear();

    /* ---- Standalone Ctrl tap: chốt từ hợp lệ hoặc khôi phục ---- */
    type_string("tunx");
    ctrl_tap();
    type_string("s ");
    check("Ctrl tap ch\xe1\xbb\x91t \xc3\xa2m ti\xe1\xba\xbft",
          "t\xc5\xa9ns "); /* "tũns " */
    clear();
    type_string("sort");
    ctrl_tap();
    type_string(" ");
    check("Ctrl tap kh\xc3\xb4i ph\xe1\xbb\xa5" "c (sort)",
          "sort "); /* R2-55 nhánh khôi phục */
    clear();

    /* ---- Bảng cài đặt đổi state.json khi IME đang chạy ---- */
    write_textvn_file("state.json", "{\"global_enabled\": false}");
    type_string("as ");
    check("state.json \xe2\x86\x92 EN", "as ");
    write_textvn_file("state.json", "{\"global_enabled\": true}");
    type_string("as ");
    check("state.json \xe2\x86\x92 VN", "as \xc3\xa1 ");
    clear();

    /* ---- Gõ tắt ---- */
    write_textvn_file(
        "config.json",
        "{\"config_version\": 1, \"auto_capitalize\": false, \"macros\":"
        " [{\"trigger\": \"vn\", \"expand\": \"Vi\xe1\xbb\x87t Nam\"}]}");
    type_string("xin vn\t");
    check("g\xc3\xb5 t\xe1\xba\xadt khi b\xe1\xba\xadt VN",
          "xin Vi\xe1\xbb\x87t Nam");
    clear();
    write_textvn_file("state.json", "{\"global_enabled\": false}");
    type_string("xin vn\t");
    check("t\xe1\xba\xadt VN: kh\xc3\xb4ng g\xc3\xb5 t\xe1\xba\xadt", "xin vn\t");
    clear();
    write_textvn_file(
        "config.json",
        "{\"config_version\": 1, \"auto_capitalize\": false,"
        " \"allow_macro_when_vi_off\": true,"
        " \"macros\": [{\"trigger\": \"vn\", \"expand\":"
        " \"Vi\xe1\xbb\x87t Nam\"}]}");
    type_string("xin vieetj vn\t");
    check("t\xe1\xba\xadt VN + g\xc3\xb5 t\xe1\xba\xadt",
          "xin vieetj Vi\xe1\xbb\x87t Nam");
    clear();
    type_string("hello world\n");
    check("t\xe1\xba\xadt VN: ch\xe1\xbb\xaf Anh nguy\xc3\xaan v\xe1\xba\xb9n",
          "hello world\n");
    clear();
    write_textvn_file("state.json", "{\"global_enabled\": true}");
    write_textvn_file("config.json", "{\"config_version\": 1}");
    type_string("as ");
    check("tr\xe1\xbb\x9f l\xe1\xba\xa1i VN", "\xc3\xa1 ");
    clear();

    /* ---- Caps Lock & Shift ---- */
    {
        const char *caps_seq = "VIEETJ ";
        for (const char *p = caps_seq; *p; p++) {
            char ch[2] = {*p, 0};
            dbus_uint32_t keysym = (unsigned char)*p;
            dbus_bool_t handled = send_key(keysym, STATE_CAPS, FALSE);
            pump(0.05);
            if (!handled)
                app_handles(ch);
            send_key(keysym, STATE_CAPS, TRUE);
            pump(0.05);
        }
        check("Caps Lock VIEETJ", "VI\xe1\xbb\x86T "); /* "VIỆT " */
        clear();
        type_string("USA ");
        check("Shift USA gi\xe1\xbb\xaf nguy\xc3\xaan", "USA ");
        clear();
    }

    /* ---- Mật khẩu: passthrough ---- */
    ic_set_capability(CAP_PREEDIT | CAP_FORMATTED | CAP_PASSWORD);
    pump(0.1);
    type_string("duocj");
    check("\xc3\xb4 m\xe1\xba\xadt kh\xe1\xba\xa9u passthrough", "duocj");

    /* ---- Text Prediction (Vòng 47) — suggest.enabled + Tab ---- */
    ic_set_capability(CAP_PREEDIT | CAP_FORMATTED);
    pump(0.1);
    write_textvn_file("config.json",
                      "{\"config_version\": 1, \"auto_capitalize\": false,"
                      " \"suggest\": {\"enabled\": true}}");
    pump(0.3); /* ConfigReload qua file monitor */
    clear();
    type_string("phats");
    send_key(KEY_TAB, 0, FALSE);
    pump(0.1);
    send_key(KEY_TAB, 0, TRUE);
    pump(0.1);
    check("suggest Tab ph\xc3\xa1t\xe2\x86\x92ph\xc3\xa1t tri\xe1\xbb\x83n",
          "ph\xc3\xa1t tri\xe1\xbb\x83n"); /* "phát triển" */
    clear();
    type_string("zzzq");
    send_key(KEY_TAB, 0, FALSE);
    pump(0.1);
    send_key(KEY_TAB, 0, TRUE);
    pump(0.1);
    check("Tab kh\xc3\xb4ng \xe1\xbb\xa9ng vi\xc3\xaan \xc4\x91i qua", "zzzq");
    clear();
    write_textvn_file("config.json", "{\"config_version\": 1}");
    pump(0.3);

    /* ---- R2-166: Math template (Latex) dời con trỏ bằng phím Left gửi lại ---- */
    write_textvn_file("config.json",
                      "{\"config_version\": 1, \"auto_capitalize\": false,"
                      " \"math\": {\"enabled\": true, \"output_mode\": \"Latex\"}}");
    pump(0.3);
    clear();
    type_string("`/");
    {
        int balanced = fwd_press == fwd_release;
        if (!balanced)
            ok = 0;
        printf("%s math cursor_left: ForwardKey press=%u release=%u doc=\"%s%s\"\n",
               balanced ? "PASS" : "FAIL", fwd_press, fwd_release, doc.data ? doc.data : "",
               preedit.data ? preedit.data : "");
    }
    clear();
    write_textvn_file("config.json", "{\"config_version\": 1}");
    pump(0.3);

    /* ---- R2-165: bất biến cả phiên — mỗi phím tối đa 1 commit ---- */
    {
        int atomic = max_commits_per_key <= 1;
        if (!atomic)
            ok = 0;
        printf("%s per-key commits are atomic (max=%u)\n", atomic ? "PASS" : "FAIL",
               max_commits_per_key);
    }

    printf(ok ? "e2e_fcitx5: OK\n" : "e2e_fcitx5: FAILED\n");
    free(doc.data);
    free(preedit.data);
    return ok ? 0 : 1;
}
