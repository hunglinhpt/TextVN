/* settings_window_gtk3.c — Bảng điều khiển TextVN cho Linux (GTK3, RHEL 8.10+)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Parity 1:1 với bản GTK4 (settings_window.c) cho các distro chưa có GTK4
 * như Red Hat Enterprise Linux 8.10 / Rocky Linux 8 / AlmaLinux 8 (GTK 3.22).
 * Chỉ dùng API có từ GTK 3.16 trở lên để biên dịch sạch không cảnh báo trên GCC 8.5.
 */

#include "settings_window.h"

#include <gdk/gdkkeysyms.h>
#include <string.h>

#include "textvn_settings.h"

#ifndef TEXTVN_VERSION
#define TEXTVN_VERSION "dev"
#endif

typedef struct TvWin {
    GtkWidget *window;
    GtkWidget *dd_charset;
    GtkWidget *dd_method;
    GtkWidget *dd_assist_mode;
    GtkWidget *lbl_assist_hint;
    GtkWidget *chk_enabled;
    GtkWidget *rad_new;
    GtkWidget *rad_old;
    GtkWidget *chk_restore;
    GtkWidget *chk_free;
    GtkWidget *chk_capitalize;
    GtkWidget *chk_quick;
    GtkWidget *chk_macro_off;
    GtkWidget *chk_suggest;
    GtkWidget *chk_math;
    GtkWidget *chk_style;
    GtkWidget *btn_style_clear;
    GtkWidget *lbl_style_stats;

    GtkWidget *words_window;
    GtkWidget *words_view;
    GtkWidget *macro_window;
    GtkWidget *macro_view;
    GtkWidget *rad_tab;
    gulong words_destroy_id;
    gulong macro_destroy_id;

    GFileMonitor *monitor;
    tv_paths paths;
    gboolean updating;
} TvWin;

static void show_message(GtkWindow *parent, const char *title, const char *body);

static gboolean on_key_press_close(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)user_data;
    if (event->keyval == GDK_KEY_Escape &&
        (event->state & gtk_accelerator_get_default_mod_mask()) == 0) {
        gtk_window_close(GTK_WINDOW(widget));
        return TRUE;
    }
    return FALSE;
}

static void add_escape_closes(GtkWidget *window) {
    g_signal_connect(window, "key-press-event", G_CALLBACK(on_key_press_close), NULL);
}

/* E69: Tìm binary `textvn` cùng gói (../../bin/textvn từ lib/textvn/, hoặc PATH). */
static char *find_textvn_cli_path(void) {
    char *self = g_file_read_link("/proc/self/exe", NULL);
    if (self) {
        char *dir = g_path_get_dirname(self);
        g_free(self);
        const char *rels[] = {"../../bin/textvn", "../bin/textvn", "textvn"};
        for (size_t i = 0; i < G_N_ELEMENTS(rels); ++i) {
            char *cand = g_build_filename(dir, rels[i], NULL);
            if (!g_file_test(cand, G_FILE_TEST_IS_DIR) &&
                g_file_test(cand, G_FILE_TEST_IS_EXECUTABLE)) {
                g_free(dir);
                return cand;
            }
            g_free(cand);
        }
        g_free(dir);
    }
    char *in_path = g_find_program_in_path("textvn");
    if (in_path) return in_path;
    return g_find_program_in_path("textvn-cli");
}

static gboolean s_diag_export_busy = FALSE;

typedef struct {
    gpointer parent; /* weak-referenced (gpointer tránh strict-aliasing UB với -O3 -Werror) */
    char    *title;
    char    *body;
} DiagResultCtx;

static gboolean finish_diag_export_idle(gpointer data) {
    DiagResultCtx *ctx = (DiagResultCtx *)data;
    s_diag_export_busy = FALSE;
    if (ctx->parent) {
        gpointer win = ctx->parent;
        g_object_remove_weak_pointer(G_OBJECT(win), &ctx->parent);
        show_message(GTK_WINDOW(win), ctx->title, ctx->body);
    }
    g_free(ctx->title);
    g_free(ctx->body);
    g_free(ctx);
    return G_SOURCE_REMOVE;
}

static gpointer diag_export_thread(gpointer data) {
    DiagResultCtx *ctx = (DiagResultCtx *)data;
    char *cli = find_textvn_cli_path();
    if (!cli) {
        ctx->title = g_strdup("Không tìm thấy CLI textvn");
        ctx->body = g_strdup("Không tìm thấy lệnh `textvn` để xuất log chẩn đoán.\n"
                             "Hãy chạy trong terminal: textvn doctor --export ~/textvn-diag.zip");
        g_idle_add(finish_diag_export_idle, ctx);
        return NULL;
    }
    char *argv[] = {cli, "doctor", "--export", "auto", NULL};
    char *out_str = NULL;
    char *err_str = NULL;
    int status = 0;
    GError *gerr = NULL;
    gboolean ok = g_spawn_sync(NULL, argv, NULL, G_SPAWN_DEFAULT, NULL, NULL,
                               &out_str, &err_str, &status, &gerr);
    g_free(cli);
    if (!ok) {
        ctx->title = g_strdup("Không xuất được log chẩn đoán");
        ctx->body = g_strdup_printf("Lỗi thực thi textvn doctor: %s",
                                    gerr ? gerr->message : "không rõ");
        if (gerr) g_error_free(gerr);
    } else {
        const char *saved = out_str ? strstr(out_str, "Đã xuất file chẩn đoán:") : NULL;
        if (saved) {
            const char *eol = strchr(saved, '\n');
            char *line = eol ? g_strndup(saved, (gsize)(eol - saved)) : g_strdup(saved);
            ctx->title = g_strdup("Đã xuất log chẩn đoán TextVN");
            ctx->body = g_strdup_printf(
                "%s\n\nTệp ZIP đã tự động ẩn tên người dùng và KHÔNG chứa nội dung bạn gõ (G8/S2).\n"
                "Hãy đính kèm tệp ZIP này khi bấm [Báo lỗi] tại:\n%s",
                line, tv_issues_url);
            g_free(line);
        } else {
            ctx->title = g_strdup("Không xuất được log chẩn đoán");
            ctx->body = g_strdup_printf("textvn doctor --export trả lỗi:\n%s",
                                        (err_str && *err_str) ? err_str : "(không có chi tiết)");
        }
    }
    g_free(out_str);
    g_free(err_str);
    g_idle_add(finish_diag_export_idle, ctx);
    return NULL;
}

static void on_export_diag(GtkButton *b, gpointer user_data) {
    (void)b;
    if (s_diag_export_busy) return;
    s_diag_export_busy = TRUE;
    DiagResultCtx *ctx = g_new0(DiagResultCtx, 1);
    ctx->parent = user_data;
    if (ctx->parent) {
        g_object_add_weak_pointer(G_OBJECT(ctx->parent), &ctx->parent);
    }
    GThread *th = g_thread_new("textvn-diag", diag_export_thread, ctx);
    g_thread_unref(th);
}

static void on_help_link(GtkButton *b, gpointer user_data) {
    GtkWindow *parent = GTK_WINDOW(user_data);
    const char *url = (const char *)g_object_get_data(G_OBJECT(b), "help-url");
    if (!url) url = tv_discussions_url;
    GError *err = NULL;
    if (!g_app_info_launch_default_for_uri(url, NULL, &err)) {
        show_message(parent, "Không mở được trình duyệt", url);
    }
    if (err) g_error_free(err);
}

static void present_info_dialog(GtkWindow *parent, const char *title, const char *body,
                                gboolean with_help_links) {
    GtkWidget *dlg = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(dlg), title);
    gtk_window_set_transient_for(GTK_WINDOW(dlg), parent);
    gtk_window_set_modal(GTK_WINDOW(dlg), TRUE);
    gtk_window_set_resizable(GTK_WINDOW(dlg), FALSE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(dlg), TRUE);
    add_escape_closes(dlg);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(box, 18);
    gtk_widget_set_margin_end(box, 18);
    gtk_widget_set_margin_top(box, 18);
    gtk_widget_set_margin_bottom(box, 12);
    GtkWidget *label = gtk_label_new(body);
    gtk_label_set_line_wrap(GTK_LABEL(label), TRUE);
    gtk_label_set_max_width_chars(GTK_LABEL(label), 70);
    gtk_label_set_xalign(GTK_LABEL(label), 0.0f);
    gtk_box_pack_start(GTK_BOX(box), label, FALSE, FALSE, 0);

    GtkWidget *ok = gtk_button_new_with_label("Đóng");
    g_signal_connect_swapped(ok, "clicked", G_CALLBACK(gtk_widget_destroy), dlg);
    if (with_help_links) {
        GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
        gtk_widget_set_halign(row, GTK_ALIGN_END);
        GtkWidget *math = gtk_button_new_with_label("Math & Science");
        gtk_widget_set_tooltip_text(math, "Mở tài liệu hướng dẫn Math & Science trên GitHub");
        g_object_set_data_full(G_OBJECT(math), "help-url", g_strdup(tv_math_guide_url), g_free);
        g_signal_connect(math, "clicked", G_CALLBACK(on_help_link), GTK_WINDOW(dlg));
        gtk_box_pack_start(GTK_BOX(row), math, FALSE, FALSE, 0);
        GtkWidget *discuss = gtk_button_new_with_label("Thảo luận");
        gtk_widget_set_tooltip_text(discuss, "Mở trang Thảo luận trên GitHub");
        g_object_set_data_full(G_OBJECT(discuss), "help-url", g_strdup(tv_discussions_url), g_free);
        g_signal_connect(discuss, "clicked", G_CALLBACK(on_help_link), GTK_WINDOW(dlg));
        GtkWidget *report = gtk_button_new_with_label("Báo lỗi");
        gtk_widget_set_tooltip_text(report, "Báo cáo sự cố trên GitHub");
        g_object_set_data_full(G_OBJECT(report), "help-url", g_strdup(tv_issues_url), g_free);
        g_signal_connect(report, "clicked", G_CALLBACK(on_help_link), GTK_WINDOW(dlg));
        GtkWidget *diag = gtk_button_new_with_label("Xuất log chẩn đoán");
        gtk_widget_set_tooltip_text(diag, "Xuất gói ZIP chẩn đoán hệ thống & log (không chứa chữ đã gõ) để gửi báo lỗi");
        g_signal_connect(diag, "clicked", G_CALLBACK(on_export_diag), parent ? (gpointer)parent : (gpointer)dlg);
        gtk_box_pack_start(GTK_BOX(row), discuss, FALSE, FALSE, 0);
        gtk_box_pack_start(GTK_BOX(row), report, FALSE, FALSE, 0);
        gtk_box_pack_start(GTK_BOX(row), diag, FALSE, FALSE, 0);
        gtk_box_pack_start(GTK_BOX(row), ok, FALSE, FALSE, 0);
        gtk_box_pack_start(GTK_BOX(box), row, FALSE, FALSE, 0);
    } else {
        gtk_widget_set_halign(ok, GTK_ALIGN_END);
        gtk_box_pack_start(GTK_BOX(box), ok, FALSE, FALSE, 0);
    }
    gtk_container_add(GTK_CONTAINER(dlg), box);
    gtk_widget_set_can_default(ok, TRUE);
    gtk_window_set_default(GTK_WINDOW(dlg), ok);
    gtk_widget_show_all(dlg);
    gtk_window_present(GTK_WINDOW(dlg));
}

static void show_message(GtkWindow *parent, const char *title, const char *body) {
    present_info_dialog(parent, title, body, FALSE);
}

static void save_failed(TvWin *w) {
    char body[1400];
    g_snprintf(body, sizeof(body),
               "Không ghi được cấu hình vào:\n%s\n\nKiểm tra quyền ghi thư mục ~/.config/TextVN.",
               w->paths.config);
    show_message(GTK_WINDOW(w->window), "TextVN", body);
}

static void set_active(GtkWidget *chk, int value) {
    gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(chk), value ? TRUE : FALSE);
}

static void populate(TvWin *w) {
    tv_snapshot s;
    tv_snapshot_load(&s, &w->paths);
    w->updating = TRUE;
    gtk_combo_box_set_active(GTK_COMBO_BOX(w->dd_charset), s.charset);
    gtk_combo_box_set_active(GTK_COMBO_BOX(w->dd_method), s.method);
    gtk_combo_box_set_active(GTK_COMBO_BOX(w->dd_assist_mode), s.assist_mode);
    if (s.assist_mode >= 0 && s.assist_mode < TV_ASSIST_MODE_COUNT) {
        gtk_label_set_text(GTK_LABEL(w->lbl_assist_hint), tv_assist_mode_hints[s.assist_mode]);
    }
    set_active(w->chk_enabled, s.vi_enabled);
    set_active(s.new_diacritic ? w->rad_new : w->rad_old, 1);
    set_active(w->chk_restore, s.auto_restore_english);
    set_active(w->chk_free, s.free_marking);
    set_active(w->chk_capitalize, s.auto_capitalize);
    set_active(w->chk_quick, s.quick_telex);
    set_active(w->chk_macro_off, s.allow_macro_when_vi_off);
    set_active(w->chk_suggest, s.suggest_enabled);
    set_active(w->chk_math, s.math_enabled);
    set_active(w->chk_style, s.style_profile_enabled);
    {
        char stats[256];
        g_snprintf(stats, sizeof(stats),
                   "Đã học: %u từ · %u cặp sửa  |  Dream RSI: tiết kiệm %u%% số phím (%u ký tự)",
                   s.style_words, s.style_corrections, s.rsi_rate_pct, s.rsi_chars_saved);
        gtk_label_set_text(GTK_LABEL(w->lbl_style_stats), stats);
        gtk_widget_set_sensitive(w->btn_style_clear, s.style_words || s.style_corrections ||
                                                         s.rsi_chars_saved || s.rsi_rate_pct);
    }
    w->updating = FALSE;
}

static void on_dropdown(GtkComboBox *cb, TvWin *w) {
    if (w->updating) return;
    int idx = gtk_combo_box_get_active(cb);
    if (idx < 0) return;
    int rc;
    if (GTK_WIDGET(cb) == w->dd_charset) {
        if (idx >= TV_CHARSET_COUNT) return;
        rc = tv_config_set_str(&w->paths, "output_charset", tv_charset_values[idx]);
    } else if (GTK_WIDGET(cb) == w->dd_assist_mode) {
        if (idx >= TV_ASSIST_MODE_COUNT) return;
        rc = tv_config_set_assist_mode(&w->paths, idx);
        gtk_label_set_text(GTK_LABEL(w->lbl_assist_hint), tv_assist_mode_hints[idx]);
    } else {
        if (idx >= TV_METHOD_COUNT) return;
        rc = tv_config_set_str(&w->paths, "method", tv_method_values[idx]);
    }
    if (rc != 0) save_failed(w);
}

static void on_check(GtkToggleButton *btn, TvWin *w) {
    if (w->updating) return;
    const gboolean on = gtk_toggle_button_get_active(btn);
    GtkWidget *wid = GTK_WIDGET(btn);
    int rc = 0;
    if (wid == w->chk_enabled) {
        rc = tv_state_set_enabled(&w->paths, on);
    } else if (wid == w->rad_new || wid == w->rad_old) {
        if (!on) return;
        rc = tv_config_set_str(&w->paths, "diacritic_style", wid == w->rad_new ? "new" : "old");
    } else {
        const char *key = g_object_get_data(G_OBJECT(btn), "tv-key");
        if (!key) return;
        rc = tv_config_set_bool(&w->paths, key, on);
    }
    if (rc != 0) save_failed(w);
}

static void on_clear_confirmed(GtkButton *b, TvWin *w) {
    GtkWidget *dlg = GTK_WIDGET(g_object_get_data(G_OBJECT(b), "tv-dialog"));
    if (tv_config_clear_learned(&w->paths) != 0) save_failed(w);
    populate(w);
    if (dlg) gtk_widget_destroy(dlg);
}

static void on_style_clear(GtkButton *b, TvWin *w) {
    (void)b;
    GtkWidget *dlg = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(dlg), "Xóa dữ liệu đã học?");
    gtk_window_set_transient_for(GTK_WINDOW(dlg), GTK_WINDOW(w->window));
    gtk_window_set_modal(GTK_WINDOW(dlg), TRUE);
    gtk_window_set_resizable(GTK_WINDOW(dlg), FALSE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(dlg), TRUE);
    add_escape_closes(dlg);
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(box, 18);
    gtk_widget_set_margin_end(box, 18);
    gtk_widget_set_margin_top(box, 18);
    gtk_widget_set_margin_bottom(box, 12);
    GtkWidget *label = gtk_label_new(
        "Xóa toàn bộ từ, cặp sửa và số liệu Dream RSI đã học trên máy này? Việc này không hoàn "
        "tác được. Gõ tắt và từ điển EN của bạn được giữ nguyên.");
    gtk_label_set_line_wrap(GTK_LABEL(label), TRUE);
    gtk_label_set_max_width_chars(GTK_LABEL(label), 70);
    gtk_label_set_xalign(GTK_LABEL(label), 0.0f);
    gtk_box_pack_start(GTK_BOX(box), label, FALSE, FALSE, 0);
    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_widget_set_halign(row, GTK_ALIGN_END);
    GtkWidget *no = gtk_button_new_with_label("Không");
    GtkWidget *yes = gtk_button_new_with_label("Xóa");
    g_object_set_data(G_OBJECT(yes), "tv-dialog", dlg);
    g_signal_connect_swapped(no, "clicked", G_CALLBACK(gtk_widget_destroy), dlg);
    g_signal_connect(yes, "clicked", G_CALLBACK(on_clear_confirmed), w);
    gtk_box_pack_start(GTK_BOX(row), no, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(row), yes, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), row, FALSE, FALSE, 0);
    gtk_container_add(GTK_CONTAINER(dlg), box);
    gtk_widget_set_can_default(no, TRUE);
    gtk_window_set_default(GTK_WINDOW(dlg), no);
    gtk_widget_show_all(dlg);
    gtk_window_present(GTK_WINDOW(dlg));
}

static void on_default(GtkButton *b, TvWin *w) {
    (void)b;
    if (tv_config_reset_defaults(&w->paths) != 0) save_failed(w);
    populate(w);
}

static void on_help(GtkButton *b, TvWin *w) {
    (void)b;
    present_info_dialog(GTK_WINDOW(w->window), "Hướng dẫn TextVN", tv_help_text, TRUE);
}

static void on_about(GtkButton *b, TvWin *w) {
    (void)b;
    char body[1024];
    g_snprintf(body, sizeof(body), "TextVN %s\n%s", TEXTVN_VERSION, tv_about_text);
    show_message(GTK_WINDOW(w->window), "Thông tin TextVN", body);
}

static void on_close(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_window_close(GTK_WINDOW(w->window));
}

static void on_macro_save(GtkButton *b, TvWin *w) {
    (void)b;
    GtkTextBuffer *buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->macro_view));
    GtkTextIter start, end;
    gtk_text_buffer_get_bounds(buf, &start, &end);
    char *text = gtk_text_buffer_get_text(buf, &start, &end, FALSE);
    const gboolean space = !gtk_toggle_button_get_active(GTK_TOGGLE_BUTTON(w->rad_tab));
    uint32_t line = 0;
    const char *msg = NULL;
    int rc = tv_macros_save(&w->paths, text, space, &line, &msg);
    g_free(text);
    if (rc > 0) {
        GtkTextIter ls, le;
        if (line > 0) {
            gtk_text_buffer_get_iter_at_line(buf, &ls, (int)line - 1);
            le = ls;
            if (!gtk_text_iter_ends_line(&le)) gtk_text_iter_forward_to_line_end(&le);
            gtk_text_buffer_select_range(buf, &ls, &le);
            gtk_text_view_scroll_to_iter(GTK_TEXT_VIEW(w->macro_view), &ls, 0.1, FALSE, 0, 0);
        }
        char body[512];
        g_snprintf(body, sizeof(body), "Dòng %u: %s.", line, msg ? msg : "không hợp lệ");
        show_message(GTK_WINDOW(w->macro_window), "Bảng gõ tắt chưa hợp lệ", body);
        gtk_widget_grab_focus(w->macro_view);
        return;
    }
    if (rc < 0) {
        save_failed(w);
        return;
    }
    gtk_widget_destroy(w->macro_window);
}

static void on_macro_cancel(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_widget_destroy(w->macro_window);
}

static void on_macro_destroy(GtkWidget *win, TvWin *w) {
    (void)win;
    w->macro_window = NULL;
    w->macro_view = NULL;
    w->rad_tab = NULL;
    w->macro_destroy_id = 0;
}

static void on_words_save(GtkButton *b, TvWin *w) {
    (void)b;
    GtkTextBuffer *buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->words_view));
    GtkTextIter start, end;
    gtk_text_buffer_get_bounds(buf, &start, &end);
    char *text = gtk_text_buffer_get_text(buf, &start, &end, FALSE);
    int rc = tv_english_words_save(&w->paths, text);
    g_free(text);
    if (rc == 0) {
        gtk_widget_destroy(w->words_window);
    } else {
        show_message(GTK_WINDOW(w->words_window), "Lưu từ điển thất bại",
                     "Không thể lưu từ điển. Kiểm tra quyền thư mục cấu hình rồi thử lại.");
    }
}

static void on_words_cancel(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_widget_destroy(w->words_window);
}

static void on_words_destroy(GtkWidget *win, TvWin *w) {
    (void)win;
    w->words_window = NULL;
    w->words_view = NULL;
    w->words_destroy_id = 0;
}

static void on_words(GtkButton *b, TvWin *w) {
    (void)b;
    if (w->words_window) {
        gtk_window_present(GTK_WINDOW(w->words_window));
        return;
    }
    char *text = tv_english_words_load(&w->paths);

    GtkWidget *win = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    w->words_window = win;
    gtk_window_set_title(GTK_WINDOW(win), "TextVN - Từ điển tiếng Anh");
    gtk_window_set_transient_for(GTK_WINDOW(win), GTK_WINDOW(w->window));
    gtk_window_set_modal(GTK_WINDOW(win), TRUE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(win), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(win), 560, 420);
    w->words_destroy_id = g_signal_connect(win, "destroy", G_CALLBACK(on_words_destroy), w);
    add_escape_closes(win);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    gtk_widget_set_margin_start(box, 12);
    gtk_widget_set_margin_end(box, 12);
    gtk_widget_set_margin_top(box, 12);
    gtk_widget_set_margin_bottom(box, 12);

    GtkWidget *hint = gtk_label_new(
        "Mỗi dòng một từ tiếng Anh bạn muốn TextVN GIỮ NGUYÊN (ví dụ: text, list, cowork).\n"
        "Dòng bắt đầu bằng # là ghi chú. Chỉ chữ cái a–z, tối đa 15 ký tự.");
    gtk_label_set_xalign(GTK_LABEL(hint), 0.0f);
    gtk_box_pack_start(GTK_BOX(box), hint, FALSE, FALSE, 0);

    GtkWidget *scroll = gtk_scrolled_window_new(NULL, NULL);
    gtk_widget_set_vexpand(scroll, TRUE);
    gtk_scrolled_window_set_shadow_type(GTK_SCROLLED_WINDOW(scroll), GTK_SHADOW_IN);
    w->words_view = gtk_text_view_new();
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(w->words_view), TRUE);
    gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->words_view)),
                             text ? text : "", -1);
    ime_settings_string_free(text);
    gtk_container_add(GTK_CONTAINER(scroll), w->words_view);
    gtk_box_pack_start(GTK_BOX(box), scroll, TRUE, TRUE, 0);

    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_pack_start(GTK_BOX(row), spacer, TRUE, TRUE, 0);
    GtkWidget *cancel = gtk_button_new_with_label("Hủy");
    g_signal_connect(cancel, "clicked", G_CALLBACK(on_words_cancel), w);
    gtk_style_context_add_class(gtk_widget_get_style_context(cancel), "flat");
    gtk_box_pack_start(GTK_BOX(row), cancel, FALSE, FALSE, 0);
    GtkWidget *save = gtk_button_new_with_label("Lưu");
    g_signal_connect(save, "clicked", G_CALLBACK(on_words_save), w);
    gtk_style_context_add_class(gtk_widget_get_style_context(save), "suggested-action");
    gtk_box_pack_start(GTK_BOX(row), save, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), row, FALSE, FALSE, 0);

    gtk_container_add(GTK_CONTAINER(win), box);
    gtk_widget_show_all(win);
    gtk_window_present(GTK_WINDOW(win));
}

static void on_macros(GtkButton *b, TvWin *w) {
    (void)b;
    if (w->macro_window) {
        gtk_window_present(GTK_WINDOW(w->macro_window));
        return;
    }
    int space = 0;
    char *text = tv_macros_load(&w->paths, &space);

    GtkWidget *win = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    w->macro_window = win;
    gtk_window_set_title(GTK_WINDOW(win), "TextVN - Gõ tắt");
    gtk_window_set_transient_for(GTK_WINDOW(win), GTK_WINDOW(w->window));
    gtk_window_set_modal(GTK_WINDOW(win), TRUE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(win), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(win), 560, 440);
    w->macro_destroy_id = g_signal_connect(win, "destroy", G_CALLBACK(on_macro_destroy), w);
    add_escape_closes(win);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    gtk_widget_set_margin_start(box, 12);
    gtk_widget_set_margin_end(box, 12);
    gtk_widget_set_margin_top(box, 12);
    gtk_widget_set_margin_bottom(box, 12);

    GtkWidget *hint = gtk_label_new(
        "Mỗi dòng một mục:   gõ tắt = nội dung      (ví dụ:  vn = Việt Nam)\n"
        "Dòng bắt đầu bằng # là ghi chú.  \\n trong nội dung = xuống dòng.  Tối đa 64 ký tự.");
    gtk_label_set_xalign(GTK_LABEL(hint), 0.0f);
    gtk_box_pack_start(GTK_BOX(box), hint, FALSE, FALSE, 0);

    GtkWidget *scroll = gtk_scrolled_window_new(NULL, NULL);
    gtk_widget_set_vexpand(scroll, TRUE);
    gtk_scrolled_window_set_shadow_type(GTK_SCROLLED_WINDOW(scroll), GTK_SHADOW_IN);
    w->macro_view = gtk_text_view_new();
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(w->macro_view), TRUE);
    gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->macro_view)),
                             text ? text : "", -1);
    ime_settings_string_free(text);
    gtk_container_add(GTK_CONTAINER(scroll), w->macro_view);
    gtk_box_pack_start(GTK_BOX(box), scroll, TRUE, TRUE, 0);

    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_box_pack_start(GTK_BOX(row), gtk_label_new("Bung gõ tắt bằng phím:"), FALSE, FALSE, 0);
    w->rad_tab = gtk_radio_button_new_with_label(NULL, "Tab");
    GtkWidget *rad_space =
        gtk_radio_button_new_with_label_from_widget(GTK_RADIO_BUTTON(w->rad_tab), "Space");
    gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(space ? rad_space : w->rad_tab), TRUE);
    gtk_box_pack_start(GTK_BOX(row), w->rad_tab, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(row), rad_space, FALSE, FALSE, 0);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_pack_start(GTK_BOX(row), spacer, TRUE, TRUE, 0);
    GtkWidget *save = gtk_button_new_with_label("Lưu");
    GtkWidget *cancel = gtk_button_new_with_label("Hủy");
    g_signal_connect(save, "clicked", G_CALLBACK(on_macro_save), w);
    g_signal_connect(cancel, "clicked", G_CALLBACK(on_macro_cancel), w);
    gtk_box_pack_start(GTK_BOX(row), save, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(row), cancel, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), row, FALSE, FALSE, 0);

    gtk_container_add(GTK_CONTAINER(win), box);
    gtk_widget_show_all(win);
    gtk_window_present(GTK_WINDOW(win));
    gtk_widget_grab_focus(w->macro_view);
}

static gboolean is_settings_file(GFile *f) {
    if (!f) return FALSE;
    char *name = g_file_get_basename(f);
    gboolean hit = name && (strcmp(name, "config.json") == 0 || strcmp(name, "state.json") == 0);
    g_free(name);
    return hit;
}

static void on_dir_changed(GFileMonitor *m, GFile *file, GFile *other, GFileMonitorEvent ev,
                           TvWin *w) {
    (void)m;
    switch (ev) {
    case G_FILE_MONITOR_EVENT_CHANGES_DONE_HINT:
    case G_FILE_MONITOR_EVENT_CREATED:
    case G_FILE_MONITOR_EVENT_RENAMED:
    case G_FILE_MONITOR_EVENT_MOVED_IN:
        if (is_settings_file(file) || is_settings_file(other)) populate(w);
        break;
    default:
        break;
    }
}

static void watch_config_dir(TvWin *w) {
    char *dir = g_path_get_dirname(w->paths.config);
    g_mkdir_with_parents(dir, 0700);
    GFile *d = g_file_new_for_path(dir);
    w->monitor = g_file_monitor_directory(d, G_FILE_MONITOR_WATCH_MOVES, NULL, NULL);
    if (w->monitor) g_signal_connect(w->monitor, "changed", G_CALLBACK(on_dir_changed), w);
    g_object_unref(d);
    g_free(dir);
}

static void tv_win_free(gpointer data) {
    TvWin *w = data;
    if (w->words_window) {
        if (w->words_destroy_id) {
            g_signal_handler_disconnect(w->words_window, w->words_destroy_id);
            w->words_destroy_id = 0;
        }
        gtk_widget_destroy(w->words_window);
        w->words_window = NULL;
    }
    if (w->macro_window) {
        if (w->macro_destroy_id) {
            g_signal_handler_disconnect(w->macro_window, w->macro_destroy_id);
            w->macro_destroy_id = 0;
        }
        gtk_widget_destroy(w->macro_window);
        w->macro_window = NULL;
    }
    if (w->monitor) {
        g_signal_handlers_disconnect_by_data(w->monitor, w);
        g_file_monitor_cancel(w->monitor);
        g_object_unref(w->monitor);
    }
    g_free(w);
}

static GtkWidget *framed_grid(GtkWidget *parent_box, const char *title) {
    GtkWidget *frame = gtk_frame_new(title);
    GtkWidget *grid = gtk_grid_new();
    gtk_grid_set_row_spacing(GTK_GRID(grid), 8);
    gtk_grid_set_column_spacing(GTK_GRID(grid), 24);
    gtk_grid_set_column_homogeneous(GTK_GRID(grid), TRUE);
    gtk_widget_set_margin_start(grid, 12);
    gtk_widget_set_margin_end(grid, 12);
    gtk_widget_set_margin_top(grid, 8);
    gtk_widget_set_margin_bottom(grid, 10);
    gtk_container_add(GTK_CONTAINER(frame), grid);
    gtk_box_pack_start(GTK_BOX(parent_box), frame, FALSE, FALSE, 0);
    return grid;
}

static GtkWidget *option(TvWin *w, GtkWidget *grid, const char *label, const char *key, int col,
                         int row) {
    GtkWidget *chk = gtk_check_button_new_with_label(label);
    if (key) g_object_set_data(G_OBJECT(chk), "tv-key", (gpointer)key);
    g_signal_connect(chk, "toggled", G_CALLBACK(on_check), w);
    gtk_grid_attach(GTK_GRID(grid), chk, col, row, 1, 1);
    return chk;
}

static GtkWidget *labeled_dropdown(TvWin *w, GtkWidget *grid, const char *label,
                                   const char *const *items, int col, int row, int width) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *l = gtk_label_new(label);
    gtk_label_set_xalign(GTK_LABEL(l), 0.0f);
    gtk_box_pack_start(GTK_BOX(box), l, FALSE, FALSE, 0);
    GtkWidget *cb = gtk_combo_box_text_new();
    for (int i = 0; items[i] != NULL; ++i) {
        gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(cb), items[i]);
    }
    gtk_widget_set_hexpand(cb, TRUE);
    gtk_box_pack_start(GTK_BOX(box), cb, TRUE, TRUE, 0);
    g_signal_connect(cb, "changed", G_CALLBACK(on_dropdown), w);
    gtk_grid_attach(GTK_GRID(grid), box, col, row, width, 1);
    return cb;
}

static GtkWidget *button(GtkWidget *row, const char *icon, const char *label, const char *hint,
                         GCallback cb, TvWin *w) {
    GtkWidget *b = gtk_button_new();
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_pack_start(GTK_BOX(box), gtk_image_new_from_icon_name(icon, GTK_ICON_SIZE_BUTTON),
                       FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(box), gtk_label_new(label), FALSE, FALSE, 0);
    gtk_container_add(GTK_CONTAINER(b), box);
    gtk_widget_set_tooltip_text(b, hint);
    g_signal_connect(b, "clicked", cb, w);
    gtk_box_pack_start(GTK_BOX(row), b, FALSE, FALSE, 0);
    return b;
}

GtkWidget *textvn_settings_window_create(GtkApplication *app, const char *custom_config_path) {
    GtkWindow *existing = gtk_application_get_active_window(app);
    if (existing) return GTK_WIDGET(existing);

    TvWin *w = g_new0(TvWin, 1);
    if (tv_paths_resolve(&w->paths, custom_config_path) != 0) {
        g_free(w);
        return NULL;
    }

    w->window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(w->window), "TextVN - Bảng điều khiển");
    gtk_window_set_icon_name(GTK_WINDOW(w->window), "textvn_v");
    gtk_window_set_resizable(GTK_WINDOW(w->window), FALSE);
    g_object_set_data_full(G_OBJECT(w->window), "tv-win", w, tv_win_free);
    add_escape_closes(w->window);

    GtkWidget *main_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_start(main_box, 12);
    gtk_widget_set_margin_end(main_box, 12);
    gtk_widget_set_margin_top(main_box, 12);
    gtk_widget_set_margin_bottom(main_box, 12);
    gtk_container_add(GTK_CONTAINER(w->window), main_box);

    /* 1. Điều khiển */
    GtkWidget *g1 = framed_grid(main_box, "Điều khiển");
    w->dd_charset = labeled_dropdown(w, g1, "Bảng mã:", tv_charset_labels, 0, 0, 1);
    w->dd_method = labeled_dropdown(w, g1, "Kiểu gõ:", tv_method_labels, 1, 0, 1);
    GtkWidget *hk = gtk_label_new("Phím chuyển:   Ctrl + Shift   (hoặc Ctrl + Shift + Space)");
    gtk_label_set_xalign(GTK_LABEL(hk), 0.0f);
    gtk_grid_attach(GTK_GRID(g1), hk, 0, 1, 2, 1);

    /* 2. Tùy chọn gõ */
    GtkWidget *g2 = framed_grid(main_box, "Tùy chọn gõ");
    w->chk_enabled = option(w, g2, "Bật gõ tiếng Việt", NULL, 0, 0);
    w->rad_new = gtk_radio_button_new_with_label(NULL, "Dấu mới (hoà, thuỷ)");
    g_signal_connect(w->rad_new, "toggled", G_CALLBACK(on_check), w);
    gtk_grid_attach(GTK_GRID(g2), w->rad_new, 1, 0, 1, 1);
    w->chk_restore =
        option(w, g2, "Khôi phục từ tiếng Anh khi gõ sai", "auto_restore_english", 0, 1);
    w->rad_old =
        gtk_radio_button_new_with_label_from_widget(GTK_RADIO_BUTTON(w->rad_new),
                                                    "Dấu cũ (hòa, thủy)");
    g_signal_connect(w->rad_old, "toggled", G_CALLBACK(on_check), w);
    gtk_grid_attach(GTK_GRID(g2), w->rad_old, 1, 1, 1, 1);
    w->chk_free = option(w, g2, "Đặt dấu tự do", "free_marking", 0, 2);
    w->chk_capitalize = option(w, g2, "Tự viết hoa chữ đầu câu", "auto_capitalize", 1, 2);
    w->chk_quick = option(w, g2, "Quick Telex (cc→ch, nn→ng…)", "quick_telex", 0, 3);
    w->chk_macro_off =
        option(w, g2, "Gõ tắt cả khi tắt tiếng Việt", "allow_macro_when_vi_off", 1, 3);
    w->chk_suggest =
        option(w, g2, "Gợi ý hoàn tất từ (Tab / Ctrl+Space)", "suggest.enabled", 0, 4);
    w->chk_math = option(w, g2, "Math & Science", "math.enabled", 1, 4);

    /* 3. Trợ lý ngữ pháp */
    GtkWidget *g3 = framed_grid(main_box, "Trợ lý ngữ pháp");
    w->dd_assist_mode = labeled_dropdown(w, g3, "Chế độ:", tv_assist_mode_labels, 0, 0, 2);
    w->lbl_assist_hint = gtk_label_new("");
    gtk_label_set_xalign(GTK_LABEL(w->lbl_assist_hint), 0.0f);
    gtk_label_set_ellipsize(GTK_LABEL(w->lbl_assist_hint), PANGO_ELLIPSIZE_END);
    gtk_style_context_add_class(gtk_widget_get_style_context(w->lbl_assist_hint), "dim-label");
    gtk_grid_attach(GTK_GRID(g3), w->lbl_assist_hint, 0, 1, 2, 1);
    w->chk_style = option(w, g3, "Tự học từ và cách sửa bạn hay dùng (chỉ trên máy này)",
                          "style_profile.enabled", 0, 2);
    {
        GtkWidget *clear_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
        gtk_widget_set_halign(clear_row, GTK_ALIGN_END);
        w->btn_style_clear = button(clear_row, "edit-clear-symbolic", "Xóa dữ liệu đã học",
                                    "Xóa từ, cặp sửa và số liệu Dream RSI đã học trên máy này",
                                    G_CALLBACK(on_style_clear), w);
        gtk_grid_attach(GTK_GRID(g3), clear_row, 1, 2, 1, 1);
    }
    w->lbl_style_stats = gtk_label_new("");
    gtk_label_set_xalign(GTK_LABEL(w->lbl_style_stats), 0.0f);
    gtk_grid_attach(GTK_GRID(g3), w->lbl_style_stats, 0, 3, 2, 1);

    /* 4. Hàng nút */
    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_widget_set_margin_top(row, 4);
    button(row, "help-browser-symbolic", "Hướng dẫn",
           "Cách gõ Telex/VNI, gõ tắt và các phím tắt", G_CALLBACK(on_help), w);
    button(row, "dialog-information-symbolic", "Thông tin",
           "Phiên bản và thông tin TextVN", G_CALLBACK(on_about), w);
    button(row, "input-keyboard-symbolic", "Gõ tắt...",
           "Thêm/sửa gõ tắt (ví dụ hd = hợp đồng)", G_CALLBACK(on_macros), w);
    button(row, "accessories-dictionary-symbolic", "Từ điển EN...",
           "Từ tiếng Anh luôn giữ nguyên khi gõ (ví dụ text, cow)", G_CALLBACK(on_words), w);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_pack_start(GTK_BOX(row), spacer, TRUE, TRUE, 0);
    button(row, "edit-undo-symbolic", "Mặc định",
           "Đặt mọi tuỳ chọn về mặc định (không xoá gõ tắt)", G_CALLBACK(on_default), w);
    GtkWidget *close = button(row, "window-close-symbolic", "Đóng",
                              "Đóng cửa sổ cài đặt — bộ gõ VẪN CHẠY (IBus/Fcitx5 chạy cùng phiên "
                              "đăng nhập, không có gì để thoát)",
                              G_CALLBACK(on_close), w);
    gtk_box_pack_start(GTK_BOX(main_box), row, FALSE, FALSE, 0);
    gtk_widget_set_can_default(close, TRUE);
    gtk_window_set_default(GTK_WINDOW(w->window), close);

    populate(w);
    watch_config_dir(w);
    gtk_widget_show_all(w->window);

    tv_snapshot s;
    tv_snapshot_load(&s, &w->paths);
    if (s.config_was_corrupt) {
        show_message(GTK_WINDOW(w->window), "TextVN",
                     "File cấu hình bị hỏng nên đang hiển thị giá trị mặc định. Lần lưu đầu "
                     "tiên sẽ giữ bản cũ thành config.json.bak.");
    }
    return w->window;
}
