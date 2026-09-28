/* e2e_ibus.c — Kiểm thử đầu-cuối với ibus-daemon THẬT
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Chạy qua scripts/e2e-linux.sh (dbus-run-session + ibus-daemon + textvn-ibus-engine).
 * Đóng vai app: tạo IBusInputContext, chọn engine "textvn", gửi phím, nhận
 * commit-text / update-preedit-text; phím engine không nuốt thì "app" tự xử lý.
 */

#include <ibus.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static GString *g_doc;      /* text app đã nhận */
static GString *g_preedit;  /* preedit đang hiển thị */

static void on_commit(IBusInputContext *ic, IBusText *text, gpointer data) {
    (void)ic;
    (void)data;
    g_string_append(g_doc, ibus_text_get_text(text));
}

static void on_preedit(IBusInputContext *ic, IBusText *text, guint cursor, gboolean visible,
                       gpointer data) {
    (void)ic;
    (void)cursor;
    (void)data;
    g_string_assign(g_preedit, visible ? ibus_text_get_text(text) : "");
}

static void on_hide_preedit(IBusInputContext *ic, gpointer data) {
    (void)ic;
    (void)data;
    g_string_truncate(g_preedit, 0);
}

static void pump(void) {
    for (int i = 0; i < 50; ++i) {
        while (g_main_context_iteration(NULL, FALSE)) {
        }
        g_usleep(2000);
    }
}

static void app_handles(guint keyval) {
    /* App tự xử lý phím engine không nuốt. */
    if (keyval == IBUS_KEY_BackSpace) {
        if (g_doc->len) {
            const char *end = g_doc->str + g_doc->len;
            const char *prev = g_utf8_find_prev_char(g_doc->str, end);
            g_string_truncate(g_doc, prev ? (gsize)(prev - g_doc->str) : 0);
        }
    } else if (keyval == IBUS_KEY_Return) {
        g_string_append_c(g_doc, '\n');
    } else {
        gunichar c = ibus_keyval_to_unicode(keyval);
        if (c >= 0x20) g_string_append_unichar(g_doc, c);
    }
}

static void press(IBusInputContext *ic, guint keyval, guint state) {
    gboolean handled = ibus_input_context_process_key_event(ic, keyval, 0, state);
    pump();
    if (!handled) {
        if (g_preedit->len) {
            fprintf(stderr, "B2 vi phạm: app nhận phím khi preedit còn mở (%s)\n", g_preedit->str);
            exit(2);
        }
        app_handles(keyval);
    }
    ibus_input_context_process_key_event(ic, keyval, 0, state | IBUS_RELEASE_MASK);
    pump();
}

static void type(IBusInputContext *ic, const char *s) {
    for (const char *p = s; *p; ++p) {
        guint kv;
        switch (*p) {
        case '\n': kv = IBUS_KEY_Return; break;
        case '\b': kv = IBUS_KEY_BackSpace; break;
        case ' ': kv = IBUS_KEY_space; break;
        default: kv = (guint)(unsigned char)*p; break;
        }
        press(ic, kv, (*p >= 'A' && *p <= 'Z') ? IBUS_SHIFT_MASK : 0);
    }
}

static int check(const char *label, const char *want) {
    GString *all = g_string_new(g_doc->str);
    g_string_append(all, g_preedit->str);
    int ok = strcmp(all->str, want) == 0;
    printf("%s %-28s want=\"%s\" got=\"%s\"\n", ok ? "PASS" : "FAIL", label, want, all->str);
    g_string_free(all, TRUE);
    return ok;
}

static void clear(IBusInputContext *ic) {
    ibus_input_context_reset(ic);
    pump();
    g_string_truncate(g_doc, 0);
    g_string_truncate(g_preedit, 0);
}

int main(void) {
    ibus_init();
    IBusBus *bus = ibus_bus_new();
    if (!ibus_bus_is_connected(bus)) {
        fprintf(stderr, "không kết nối được ibus-daemon\n");
        return 1;
    }
    g_doc = g_string_new("");
    g_preedit = g_string_new("");

    IBusInputContext *ic = ibus_bus_create_input_context(bus, "textvn-e2e");
    ibus_input_context_set_capabilities(ic, IBUS_CAP_PREEDIT_TEXT | IBUS_CAP_FOCUS);
    g_signal_connect(ic, "commit-text", G_CALLBACK(on_commit), NULL);
    g_signal_connect(ic, "update-preedit-text", G_CALLBACK(on_preedit), NULL);
    g_signal_connect(ic, "hide-preedit-text", G_CALLBACK(on_hide_preedit), NULL);
    ibus_input_context_focus_in(ic);

    /* Engine có thể chưa kịp đăng ký: thử lại tối đa ~3s. */
    IBusEngineDesc *desc = NULL;
    for (int i = 0; i < 60 && !desc; ++i) {
        ibus_input_context_set_engine(ic, "textvn");
        pump();
        desc = ibus_input_context_get_engine(ic);
        if (desc && g_strcmp0(ibus_engine_desc_get_name(desc), "textvn") != 0) desc = NULL;
    }
    if (!desc) {
        fprintf(stderr, "engine textvn không khởi động được qua ibus-daemon\n");
        return 1;
    }

    int ok = 1;
    type(ic, "duocj ");
    ok &= check("telex duocj+space", "được ");
    clear(ic);
    type(ic, "Vieetj Nam");
    ok &= check("preedit giữ cả từ", "Việt Nam");
    ok &= strcmp(g_preedit->str, "Nam") == 0;
    clear(ic);
    type(ic, "chaof banj\n");
    ok &= check("Enter commit (B2)", "chào bạn\n");
    clear(ic);
    type(ic, "duocj\b ");
    ok &= check("Backspace trong từ", "đươc ");
    clear(ic);
    type(ic, "hello world ");
    ok &= check("tiếng Anh giữ nguyên", "hello world ");
    clear(ic);

    /* Focus-out giữa từ: phải commit, không mất chữ. */
    type(ic, "tieengs");
    ibus_input_context_focus_out(ic);
    pump();
    ok &= check("focus-out commit", "tiếng");
    ibus_input_context_focus_in(ic);
    pump();
    clear(ic);

    /* Client reset giữa từ (click chuột): commit đúng MỘT lần. */
    type(ic, "tieengs");
    ibus_input_context_reset(ic);
    pump();
    ok &= check("reset commit 1 lần", "tiếng");
    type(ic, " ");
    ok &= check("gõ tiếp sau reset", "tiếng ");
    clear(ic);

    /* Đổi sang bộ gõ khác giữa từ: chữ đang soạn phải còn, đúng một lần. */
    type(ic, "tieengs");
    ibus_input_context_set_engine(ic, "xkb:us::eng");
    pump();
    ok &= check("đổi engine giữa từ", "tiếng");
    ibus_input_context_set_engine(ic, "textvn");
    pump();
    clear(ic);

    /* Ctrl+Shift (nhấn rồi nhả) tắt tiếng Việt, bấm lại để bật. */
    press(ic, IBUS_KEY_Control_L, 0);
    ibus_input_context_process_key_event(ic, IBUS_KEY_Shift_L, 0, IBUS_CONTROL_MASK);
    pump();
    ibus_input_context_process_key_event(ic, IBUS_KEY_Shift_L, 0,
                                         IBUS_CONTROL_MASK | IBUS_SHIFT_MASK | IBUS_RELEASE_MASK);
    pump();
    type(ic, "as ");
    ok &= check("Ctrl+Shift → EN", "as ");
    clear(ic);
    press(ic, IBUS_KEY_Control_L, 0);
    ibus_input_context_process_key_event(ic, IBUS_KEY_Shift_L, 0, IBUS_CONTROL_MASK);
    pump();
    ibus_input_context_process_key_event(ic, IBUS_KEY_Shift_L, 0,
                                         IBUS_CONTROL_MASK | IBUS_SHIFT_MASK | IBUS_RELEASE_MASK);
    pump();
    type(ic, "as ");
    ok &= check("Ctrl+Shift → VN", "á ");
    clear(ic);

    /* Ô mật khẩu: không biến đổi. */
    ibus_input_context_set_content_type(ic, IBUS_INPUT_PURPOSE_PASSWORD, 0);
    pump();
    type(ic, "duocj");
    ok &= check("ô mật khẩu passthrough", "duocj");
    ibus_input_context_set_content_type(ic, IBUS_INPUT_PURPOSE_FREE_FORM, 0);
    pump();

    printf(ok ? "e2e_ibus: OK\n" : "e2e_ibus: FAILED\n");
    return ok ? 0 : 1;
}
