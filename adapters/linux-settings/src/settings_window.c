/* settings_window.c — Bảng điều khiển TextVN cho Linux (GTK4)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Mỗi thao tác lưu NGAY (như bảng điều khiển Windows): IBus/Fcitx5 đọc lại config.json /
 * state.json theo mtime ở phím kế tiếp, không cần khởi động lại gì. Cửa sổ theo dõi thư
 * mục cấu hình nên Ctrl+Shift bấm lúc cửa sổ đang mở cũng hiện đúng trạng thái V/E.
 *
 * Chỉ dùng API có từ GTK 4.0 (không GtkAlertDialog 4.10) để chạy được trên Debian 12,
 * Ubuntu 22.04 trở lên.
 */

#include "settings_window.h"

#include <string.h>

#include "textvn_settings.h"

#ifndef TEXTVN_VERSION
#define TEXTVN_VERSION "dev"
#endif

typedef struct TvWin {
    GtkWidget *window;
    GtkWidget *dd_charset;
    GtkWidget *dd_method;
    GtkWidget *chk_enabled;
    GtkWidget *rad_new;
    GtkWidget *rad_old;
    GtkWidget *chk_restore;
    GtkWidget *chk_free;
    GtkWidget *chk_capitalize;
    GtkWidget *chk_quick;
    GtkWidget *chk_macro_off;

    GtkWidget *words_window;
    GtkWidget *words_view;
    GtkWidget *macro_window;
    GtkWidget *macro_view;
    GtkWidget *rad_tab;

    GFileMonitor *monitor;
    tv_paths paths;
    gboolean updating; /* đang đổ dữ liệu lên widget: không ghi ngược ra file */
} TvWin;

/* ---- Hộp thông báo (GTK 4.0) ---- */

static void show_message(GtkWindow *parent, const char *title, const char *body) {
    GtkWidget *dlg = gtk_window_new();
    gtk_window_set_title(GTK_WINDOW(dlg), title);
    gtk_window_set_transient_for(GTK_WINDOW(dlg), parent);
    gtk_window_set_modal(GTK_WINDOW(dlg), TRUE);
    gtk_window_set_resizable(GTK_WINDOW(dlg), FALSE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(dlg), TRUE);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(box, 18);
    gtk_widget_set_margin_end(box, 18);
    gtk_widget_set_margin_top(box, 18);
    gtk_widget_set_margin_bottom(box, 12);
    GtkWidget *label = gtk_label_new(body);
    gtk_label_set_wrap(GTK_LABEL(label), TRUE);
    gtk_label_set_max_width_chars(GTK_LABEL(label), 70);
    gtk_label_set_xalign(GTK_LABEL(label), 0.0f);
    gtk_box_append(GTK_BOX(box), label);
    GtkWidget *ok = gtk_button_new_with_label("Đóng");
    gtk_widget_set_halign(ok, GTK_ALIGN_END);
    g_signal_connect_swapped(ok, "clicked", G_CALLBACK(gtk_window_destroy), dlg);
    gtk_box_append(GTK_BOX(box), ok);
    gtk_window_set_child(GTK_WINDOW(dlg), box);
    gtk_window_set_default_widget(GTK_WINDOW(dlg), ok);
    gtk_window_present(GTK_WINDOW(dlg));
}

static void save_failed(TvWin *w) {
    char body[1400];
    g_snprintf(body, sizeof(body),
               "Không ghi được cấu hình vào:\n%s\n\nKiểm tra quyền ghi thư mục ~/.config/TextVN.",
               w->paths.config);
    show_message(GTK_WINDOW(w->window), "TextVN", body);
}

/* Esc đóng cửa sổ (như Windows). */
static void add_escape_closes(GtkWidget *window) {
    GtkEventController *sc = gtk_shortcut_controller_new();
    gtk_shortcut_controller_add_shortcut(
        GTK_SHORTCUT_CONTROLLER(sc),
        gtk_shortcut_new(gtk_keyval_trigger_new(GDK_KEY_Escape, 0),
                         gtk_named_action_new("window.close")));
    gtk_widget_add_controller(window, sc);
}

/* ---- Đổ dữ liệu lên widget ---- */

static void set_active(GtkWidget *chk, int value) {
    gtk_check_button_set_active(GTK_CHECK_BUTTON(chk), value ? TRUE : FALSE);
}

static void populate(TvWin *w) {
    tv_snapshot s;
    tv_snapshot_load(&s, &w->paths);
    w->updating = TRUE;
    gtk_drop_down_set_selected(GTK_DROP_DOWN(w->dd_charset), (guint)s.charset);
    gtk_drop_down_set_selected(GTK_DROP_DOWN(w->dd_method), (guint)s.method);
    set_active(w->chk_enabled, s.vi_enabled);
    set_active(s.new_diacritic ? w->rad_new : w->rad_old, 1);
    set_active(w->chk_restore, s.auto_restore_english);
    set_active(w->chk_free, s.free_marking);
    set_active(w->chk_capitalize, s.auto_capitalize);
    set_active(w->chk_quick, s.quick_telex);
    set_active(w->chk_macro_off, s.allow_macro_when_vi_off);
    w->updating = FALSE;
}

/* ---- Thao tác người dùng → lưu ngay ---- */

static void on_dropdown(GtkDropDown *dd, GParamSpec *pspec, TvWin *w) {
    (void)pspec;
    if (w->updating) return;
    guint idx = gtk_drop_down_get_selected(dd);
    int rc;
    if (GTK_WIDGET(dd) == w->dd_charset) {
        if (idx >= TV_CHARSET_COUNT) return;
        rc = tv_config_set_str(&w->paths, "output_charset", tv_charset_values[idx]);
    } else {
        if (idx >= TV_METHOD_COUNT) return;
        rc = tv_config_set_str(&w->paths, "method", tv_method_values[idx]);
    }
    if (rc != 0) save_failed(w);
}

static void on_check(GtkCheckButton *btn, TvWin *w) {
    if (w->updating) return;
    const gboolean on = gtk_check_button_get_active(btn);
    GtkWidget *wid = GTK_WIDGET(btn);
    int rc = 0;
    if (wid == w->chk_enabled) {
        rc = tv_state_set_enabled(&w->paths, on);
    } else if (wid == w->rad_new || wid == w->rad_old) {
        if (!on) return; /* nút radio vừa bị bỏ chọn — nút kia sẽ báo */
        rc = tv_config_set_str(&w->paths, "diacritic_style", wid == w->rad_new ? "new" : "old");
    } else {
        const char *key = g_object_get_data(G_OBJECT(btn), "tv-key");
        if (!key) return;
        rc = tv_config_set_bool(&w->paths, key, on);
    }
    if (rc != 0) save_failed(w);
}

static void on_default(GtkButton *b, TvWin *w) {
    (void)b;
    if (tv_config_reset_defaults(&w->paths) != 0) save_failed(w);
    populate(w);
}

static void on_help(GtkButton *b, TvWin *w) {
    (void)b;
    show_message(GTK_WINDOW(w->window), "Hướng dẫn TextVN", tv_help_text);
}

static void on_about(GtkButton *b, TvWin *w) {
    (void)b;
    char body[1024];
    g_snprintf(body, sizeof(body), "TextVN %s\n\n%s", TEXTVN_VERSION, tv_about_text);
    show_message(GTK_WINDOW(w->window), "Thông tin TextVN", body);
}

static void on_close(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_window_close(GTK_WINDOW(w->window));
}

/* ---- Cửa sổ Gõ tắt ---- */

static void on_macro_save(GtkButton *b, TvWin *w) {
    (void)b;
    GtkTextBuffer *buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->macro_view));
    GtkTextIter start, end;
    gtk_text_buffer_get_bounds(buf, &start, &end);
    char *text = gtk_text_buffer_get_text(buf, &start, &end, FALSE);
    const gboolean space = !gtk_check_button_get_active(GTK_CHECK_BUTTON(w->rad_tab));
    uint32_t line = 0;
    const char *msg = NULL;
    int rc = tv_macros_save(&w->paths, text, space, &line, &msg);
    g_free(text);
    if (rc > 0) {
        /* Bôi đen dòng sai để sửa ngay. */
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
    gtk_window_destroy(GTK_WINDOW(w->macro_window));
}

static void on_macro_cancel(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_window_destroy(GTK_WINDOW(w->macro_window));
}

static void on_macro_destroy(GtkWidget *win, TvWin *w) {
    (void)win;
    w->words_window = NULL;
    w->words_view = NULL;
    w->macro_window = NULL;
    w->macro_view = NULL;
    w->rad_tab = NULL;
}

/* Tu dien EN (0.2.13) — parity Windows "Tu dien EN...": tu trong danh sach
 * duoc engine giu nguyen khi go ke ca khi fold trung am tiet Viet thong dung. */
static void on_words_save(GtkButton *b, TvWin *w) {
    (void)b;
    GtkTextBuffer *buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->words_view));
    GtkTextIter start, end;
    gtk_text_buffer_get_bounds(buf, &start, &end);
    char *text = gtk_text_buffer_get_text(buf, &start, &end, FALSE);
    int rc = tv_english_words_save(&w->paths, text);
    g_free(text);
    if (rc == 0) {
        gtk_window_destroy(GTK_WINDOW(w->words_window));
    } else {
        GtkWidget *dlg = gtk_message_dialog_new(GTK_WINDOW(w->words_window), GTK_DIALOG_MODAL,
                                                GTK_MESSAGE_ERROR, GTK_BUTTONS_OK,
                                                "Khong the luu tu dien. Kiem tra quyen thu muc cau hinh.");
        g_signal_connect_swapped(dlg, "response", G_CALLBACK(gtk_window_destroy), dlg);
        gtk_window_present(GTK_WINDOW(dlg));
    }
}

static void on_words_cancel(GtkButton *b, TvWin *w) {
    (void)b;
    gtk_window_destroy(GTK_WINDOW(w->words_window));
}

static void on_words_destroy(GtkWidget *win, TvWin *w) {
    (void)win;
    w->words_window = NULL;
    w->words_view = NULL;
}

static void on_words(GtkButton *b, TvWin *w) {
    (void)b;
    if (w->words_window) {
        gtk_window_present(GTK_WINDOW(w->words_window));
        return;
    }
    char *text = tv_english_words_load(&w->paths);

    GtkWidget *win = gtk_window_new();
    w->words_window = win;
    gtk_window_set_title(GTK_WINDOW(win), "TextVN - Tu dien tieng Anh");
    gtk_window_set_transient_for(GTK_WINDOW(win), GTK_WINDOW(w->window));
    gtk_window_set_modal(GTK_WINDOW(win), TRUE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(win), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(win), 560, 420);
    g_signal_connect(win, "destroy", G_CALLBACK(on_words_destroy), w);
    add_escape_closes(win);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    gtk_widget_set_margin_start(box, 12);
    gtk_widget_set_margin_end(box, 12);
    gtk_widget_set_margin_top(box, 12);
    gtk_widget_set_margin_bottom(box, 12);

    GtkWidget *hint = gtk_label_new(
        "Moi dong mot tu tieng Anh ban muon TextVN GIU NGUYEN (vi du: text, list, cowork).
"
        "Dong bat dau bang # la ghi chu. Chi chu cai a-z, toi da 15 ky tu.");
    gtk_label_set_xalign(GTK_LABEL(hint), 0.0f);
    gtk_box_append(GTK_BOX(box), hint);

    GtkWidget *scroll = gtk_scrolled_window_new();
    gtk_widget_set_vexpand(scroll, TRUE);
    gtk_scrolled_window_set_has_frame(GTK_SCROLLED_WINDOW(scroll), TRUE);
    w->words_view = gtk_text_view_new();
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(w->words_view), TRUE);
    gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->words_view)),
                             text ? text : "", -1);
    ime_settings_string_free(text);
    gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scroll), w->words_view);
    gtk_box_append(GTK_BOX(box), scroll);

    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_append(GTK_BOX(row), spacer);
    GtkWidget *cancel = button(row, "Huy", G_CALLBACK(on_words_cancel), w);
    gtk_widget_add_css_class(cancel, "flat");
    GtkWidget *save = button(row, "Luu", G_CALLBACK(on_words_save), w);
    gtk_widget_add_css_class(save, "suggested-action");
    gtk_box_append(GTK_BOX(box), row);

    gtk_window_set_child(GTK_WINDOW(win), box);
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

    GtkWidget *win = gtk_window_new();
    w->macro_window = win;
    gtk_window_set_title(GTK_WINDOW(win), "TextVN - Gõ tắt");
    gtk_window_set_transient_for(GTK_WINDOW(win), GTK_WINDOW(w->window));
    gtk_window_set_modal(GTK_WINDOW(win), TRUE);
    gtk_window_set_destroy_with_parent(GTK_WINDOW(win), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(win), 560, 440);
    g_signal_connect(win, "destroy", G_CALLBACK(on_macro_destroy), w);
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
    gtk_box_append(GTK_BOX(box), hint);

    GtkWidget *scroll = gtk_scrolled_window_new();
    gtk_widget_set_vexpand(scroll, TRUE);
    gtk_scrolled_window_set_has_frame(GTK_SCROLLED_WINDOW(scroll), TRUE);
    w->macro_view = gtk_text_view_new();
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(w->macro_view), TRUE);
    gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(w->macro_view)),
                             text ? text : "", -1);
    ime_settings_string_free(text);
    gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scroll), w->macro_view);
    gtk_box_append(GTK_BOX(box), scroll);

    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_box_append(GTK_BOX(row), gtk_label_new("Bung gõ tắt bằng phím:"));
    w->rad_tab = gtk_check_button_new_with_label("Tab");
    GtkWidget *rad_space = gtk_check_button_new_with_label("Space");
    gtk_check_button_set_group(GTK_CHECK_BUTTON(rad_space), GTK_CHECK_BUTTON(w->rad_tab));
    gtk_check_button_set_active(GTK_CHECK_BUTTON(space ? rad_space : w->rad_tab), TRUE);
    gtk_box_append(GTK_BOX(row), w->rad_tab);
    gtk_box_append(GTK_BOX(row), rad_space);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_append(GTK_BOX(row), spacer);
    GtkWidget *save = gtk_button_new_with_label("Lưu");
    GtkWidget *cancel = gtk_button_new_with_label("Hủy");
    g_signal_connect(save, "clicked", G_CALLBACK(on_macro_save), w);
    g_signal_connect(cancel, "clicked", G_CALLBACK(on_macro_cancel), w);
    gtk_box_append(GTK_BOX(row), save);
    gtk_box_append(GTK_BOX(row), cancel);
    gtk_box_append(GTK_BOX(box), row);

    gtk_window_set_child(GTK_WINDOW(win), box);
    gtk_window_present(GTK_WINDOW(win));
    gtk_widget_grab_focus(w->macro_view);
}

/* ---- Theo dõi config.json / state.json ---- */

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
    if (w->monitor) {
        g_file_monitor_cancel(w->monitor);
        g_object_unref(w->monitor);
    }
    g_free(w);
}

/* ---- Dựng cửa sổ ---- */

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
    gtk_frame_set_child(GTK_FRAME(frame), grid);
    gtk_box_append(GTK_BOX(parent_box), frame);
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
                                   const char *const *items, int col) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    GtkWidget *l = gtk_label_new(label);
    gtk_label_set_xalign(GTK_LABEL(l), 0.0f);
    gtk_box_append(GTK_BOX(box), l);
    GtkWidget *dd = gtk_drop_down_new_from_strings(items);
    gtk_widget_set_hexpand(dd, TRUE);
    gtk_box_append(GTK_BOX(box), dd);
    g_signal_connect(dd, "notify::selected", G_CALLBACK(on_dropdown), w);
    gtk_grid_attach(GTK_GRID(grid), box, col, 0, 1, 1);
    return dd;
}

static GtkWidget *button(GtkWidget *row, const char *label, GCallback cb, TvWin *w) {
    GtkWidget *b = gtk_button_new_with_label(label);
    g_signal_connect(b, "clicked", cb, w);
    gtk_box_append(GTK_BOX(row), b);
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
    gtk_window_set_child(GTK_WINDOW(w->window), main_box);

    /* 1. Điều khiển */
    GtkWidget *g1 = framed_grid(main_box, "Điều khiển");
    w->dd_charset = labeled_dropdown(w, g1, "Bảng mã:", tv_charset_labels, 0);
    w->dd_method = labeled_dropdown(w, g1, "Kiểu gõ:", tv_method_labels, 1);
    GtkWidget *hk = gtk_label_new("Phím chuyển:   Ctrl + Shift   (hoặc Ctrl + Shift + Space)");
    gtk_label_set_xalign(GTK_LABEL(hk), 0.0f);
    gtk_grid_attach(GTK_GRID(g1), hk, 0, 1, 2, 1);

    /* 2. Tùy chọn gõ — cùng vị trí với Windows */
    GtkWidget *g2 = framed_grid(main_box, "Tùy chọn gõ");
    w->chk_enabled = option(w, g2, "Bật gõ tiếng Việt", NULL, 0, 0);
    w->rad_new = option(w, g2, "Dấu mới (hoà, thuỷ)", NULL, 1, 0);
    w->chk_restore = option(w, g2, "Khôi phục từ tiếng Anh khi gõ sai", "auto_restore_english", 0, 1);
    w->rad_old = option(w, g2, "Dấu cũ (hòa, thủy)", NULL, 1, 1);
    gtk_check_button_set_group(GTK_CHECK_BUTTON(w->rad_old), GTK_CHECK_BUTTON(w->rad_new));
    w->chk_free = option(w, g2, "Đặt dấu tự do", "free_marking", 0, 2);
    w->chk_capitalize = option(w, g2, "Tự viết hoa chữ đầu câu", "auto_capitalize", 1, 2);
    w->chk_quick = option(w, g2, "Quick Telex (cc→ch, nn→ng…)", "quick_telex", 0, 3);
    w->chk_macro_off =
        option(w, g2, "Gõ tắt cả khi tắt tiếng Việt", "allow_macro_when_vi_off", 1, 3);

    /* 3. Hàng nút: thông tin/công cụ (trái) · thao tác (phải) */
    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_widget_set_margin_top(row, 4);
    button(row, "Hướng dẫn", G_CALLBACK(on_help), w);
    button(row, "Thông tin", G_CALLBACK(on_about), w);
    button(row, "Gõ tắt...", G_CALLBACK(on_macros), w);
    button(row, "Từ điển EN...", G_CALLBACK(on_words), w);
    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_append(GTK_BOX(row), spacer);
    button(row, "Mặc định", G_CALLBACK(on_default), w);
    GtkWidget *close = button(row, "Đóng", G_CALLBACK(on_close), w);
    gtk_window_set_default_widget(GTK_WINDOW(w->window), close);
    gtk_box_append(GTK_BOX(main_box), row);

    populate(w);
    watch_config_dir(w);

    tv_snapshot s;
    tv_snapshot_load(&s, &w->paths);
    if (s.config_was_corrupt) {
        show_message(GTK_WINDOW(w->window), "TextVN",
                     "File cấu hình bị hỏng nên đang hiển thị giá trị mặc định. Lần lưu đầu "
                     "tiên sẽ giữ bản cũ thành config.json.bak.");
    }
    return w->window;
}
