/* settings_window.c — UniKey 4.6 RC2 style GTK4 Settings Panel for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: PLAN §2.3 M6, P3-5-ui-packaging-release.md
 * Compact: ~505x245px | Expanded: ~505x490px
 */

#include "settings_window.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void textvn_settings_set_defaults(TextVNSettings *s) {
    if (!s) return;
    memset(s, 0, sizeof(*s));
    s->config_version = 1;
    s->enabled = true;
    s->charset = TEXTVN_CHARSET_UNICODE;
    s->method = TEXTVN_METHOD_TELEX;
    s->switch_key = TEXTVN_SWITCH_CTRL_SHIFT;

    s->spell_check = true;
    s->auto_restore_english = true;
    s->allow_macro = false;
    s->new_diacritic = true;

    s->show_dialog_on_startup = true;
    s->autostart = true;
    s->non_preedit = true;
    s->run_in_tray = true;

    s->is_expanded = false;
}

static bool json_has_key_true(const char *json, const char *key) {
    char needle[64];
    snprintf(needle, sizeof(needle), "\"%s\":", key);
    const char *p = strstr(json, needle);
    if (!p) return false;
    p += strlen(needle);
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') p++;
    return (strncmp(p, "true", 4) == 0);
}

static bool json_get_str_val(const char *json, const char *key, char *out_val, size_t max_len) {
    char needle[64];
    snprintf(needle, sizeof(needle), "\"%s\":", key);
    const char *p = strstr(json, needle);
    if (!p) return false;
    p += strlen(needle);
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') p++;
    if (*p != '"') return false;
    p++;
    size_t i = 0;
    while (*p && *p != '"' && i + 1 < max_len) {
        out_val[i++] = *p++;
    }
    out_val[i] = '\0';
    return true;
}

int textvn_settings_load_from_json(TextVNSettings *s, const char *json_str) {
    if (!s || !json_str) return -1;
    textvn_settings_set_defaults(s);

    char val[64];

    /* Method */
    if (json_get_str_val(json_str, "method", val, sizeof(val))) {
        if (strcmp(val, "vni") == 0) {
            s->method = TEXTVN_METHOD_VNI;
        } else if (strcmp(val, "viqr") == 0) {
            s->method = TEXTVN_METHOD_VIQR;
        } else if (strcmp(val, "simple_telex") == 0) {
            s->method = TEXTVN_METHOD_MICROSOFT;
        } else {
            s->method = TEXTVN_METHOD_TELEX;
        }
    }

    /* Output Charset */
    if (json_get_str_val(json_str, "output_charset", val, sizeof(val))) {
        if (strcmp(val, "tcvn3") == 0) {
            s->charset = TEXTVN_CHARSET_TCVN3;
        } else if (strcmp(val, "vni_windows") == 0) {
            s->charset = TEXTVN_CHARSET_VNI_WINDOWS;
        } else if (strcmp(val, "unicode_decomposed") == 0) {
            s->charset = TEXTVN_CHARSET_UNICODE_DECOMPOSED;
        } else {
            s->charset = TEXTVN_CHARSET_UNICODE;
        }
    }

    /* Diacritic style */
    if (json_get_str_val(json_str, "diacritic_style", val, sizeof(val))) {
        if (strcmp(val, "old") == 0) {
            s->new_diacritic = false;
        } else {
            s->new_diacritic = true;
        }
    }

    /* Booleans */
    s->spell_check = json_has_key_true(json_str, "free_marking");
    s->auto_restore_english = json_has_key_true(json_str, "auto_restore_english");
    s->allow_macro = json_has_key_true(json_str, "allow_macro_when_vi_off");

    if (strstr(json_str, "\"show_dialog_on_startup\":")) {
        s->show_dialog_on_startup = json_has_key_true(json_str, "show_dialog_on_startup");
    }
    if (strstr(json_str, "\"autostart\":")) {
        s->autostart = json_has_key_true(json_str, "autostart");
    }
    if (strstr(json_str, "\"non_preedit\":")) {
        s->non_preedit = json_has_key_true(json_str, "non_preedit");
    }
    if (strstr(json_str, "\"run_in_tray\":")) {
        s->run_in_tray = json_has_key_true(json_str, "run_in_tray");
    }

    return 0;
}

char* textvn_settings_to_json(const TextVNSettings *s) {
    if (!s) return NULL;

    const char *method_str = "telex";
    switch (s->method) {
        case TEXTVN_METHOD_VNI: method_str = "vni"; break;
        case TEXTVN_METHOD_VIQR: method_str = "viqr"; break;
        case TEXTVN_METHOD_MICROSOFT: method_str = "simple_telex"; break;
        default: method_str = "telex"; break;
    }

    const char *charset_str = "unicode_precomposed";
    switch (s->charset) {
        case TEXTVN_CHARSET_TCVN3: charset_str = "tcvn3"; break;
        case TEXTVN_CHARSET_VNI_WINDOWS: charset_str = "vni_windows"; break;
        case TEXTVN_CHARSET_UNICODE_DECOMPOSED: charset_str = "unicode_decomposed"; break;
        default: charset_str = "unicode_precomposed"; break;
    }

    char *buf = (char *)malloc(2048);
    if (!buf) return NULL;

    snprintf(buf, 2048,
        "{\n"
        "  \"config_version\": 1,\n"
        "  \"enabled\": %s,\n"
        "  \"method\": \"%s\",\n"
        "  \"diacritic_style\": \"%s\",\n"
        "  \"free_marking\": %s,\n"
        "  \"auto_restore_english\": %s,\n"
        "  \"auto_capitalize\": true,\n"
        "  \"macro_trigger\": \"tab\",\n"
        "  \"allow_macro_when_vi_off\": %s,\n"
        "  \"output_charset\": \"%s\",\n"
        "  \"show_dialog_on_startup\": %s,\n"
        "  \"autostart\": %s,\n"
        "  \"non_preedit\": %s,\n"
        "  \"run_in_tray\": %s\n"
        "}\n",
        s->enabled ? "true" : "false",
        method_str,
        s->new_diacritic ? "new" : "old",
        s->spell_check ? "true" : "false",
        s->auto_restore_english ? "true" : "false",
        s->allow_macro ? "true" : "false",
        charset_str,
        s->show_dialog_on_startup ? "true" : "false",
        s->autostart ? "true" : "false",
        s->non_preedit ? "true" : "false",
        s->run_in_tray ? "true" : "false"
    );

    return buf;
}

static void resolve_config_path(char *out, size_t max_len, const char *custom) {
    if (custom && custom[0] != '\0') {
        strncpy(out, custom, max_len - 1);
        out[max_len - 1] = '\0';
        return;
    }
    const char *xdg = getenv("XDG_CONFIG_HOME");
    if (xdg && xdg[0] != '\0') {
        snprintf(out, max_len, "%s/TextVN/config.json", xdg);
        return;
    }
    const char *home = getenv("HOME");
    if (home && home[0] != '\0') {
        snprintf(out, max_len, "%s/.config/TextVN/config.json", home);
        return;
    }
    strncpy(out, "/tmp/textvn_config.json", max_len - 1);
}

int textvn_settings_load_file(TextVNSettings *s, const char *custom_path) {
    char path[512];
    resolve_config_path(path, sizeof(path), custom_path);

    FILE *f = fopen(path, "rb");
    if (!f) {
        textvn_settings_set_defaults(s);
        return 0;
    }

    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (len <= 0 || len > 65536) {
        fclose(f);
        textvn_settings_set_defaults(s);
        return -1;
    }

    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        fclose(f);
        return -1;
    }

    size_t read_bytes = fread(buf, 1, (size_t)len, f);
    fclose(f);
    buf[read_bytes] = '\0';

    int rc = textvn_settings_load_from_json(s, buf);
    free(buf);
    return rc;
}

int textvn_settings_save_file(const TextVNSettings *s, const char *custom_path) {
    char path[512];
    resolve_config_path(path, sizeof(path), custom_path);

    char *json = textvn_settings_to_json(s);
    if (!json) return -1;

    FILE *f = fopen(path, "wb");
    if (!f) {
        free(json);
        return -1;
    }

    size_t len = strlen(json);
    size_t written = fwrite(json, 1, len, f);
    fclose(f);
    free(json);

    return (written == len) ? 0 : -1;
}

void textvn_settings_window_toggle_expanded(TextVNSettingsWindow *win) {
    if (!win) return;
    win->settings.is_expanded = !win->settings.is_expanded;

    if (win->expanded_box) {
        gtk_widget_set_visible(win->expanded_box, win->settings.is_expanded);
    }
    if (win->btn_expand_toggle) {
        gtk_button_set_label(GTK_BUTTON(win->btn_expand_toggle),
            win->settings.is_expanded ? "<< Thu nhỏ" : "Mở rộng >>");
    }
    if (win->window) {
        gtk_window_set_default_size(GTK_WINDOW(win->window),
            TEXTVN_WINDOW_WIDTH_COMPACT,
            win->settings.is_expanded ? TEXTVN_WINDOW_HEIGHT_EXPANDED : TEXTVN_WINDOW_HEIGHT_COMPACT);
    }
}

void textvn_settings_window_reset_defaults(TextVNSettingsWindow *win) {
    if (!win) return;
    textvn_settings_set_defaults(&win->settings);

    if (win->combo_charset) gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_charset), win->settings.charset);
    if (win->combo_method) gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_method), win->settings.method);
    if (win->chk_spell_check) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_spell_check), win->settings.spell_check);
    if (win->chk_auto_restore) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_auto_restore), win->settings.auto_restore_english);
    if (win->chk_allow_macro) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_allow_macro), win->settings.allow_macro);
    if (win->chk_new_diacritic) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_new_diacritic), win->settings.new_diacritic);
    if (win->chk_show_dialog) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_show_dialog), win->settings.show_dialog_on_startup);
    if (win->chk_autostart) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_autostart), win->settings.autostart);
    if (win->chk_non_preedit) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_non_preedit), win->settings.non_preedit);
    if (win->chk_run_in_tray) gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_run_in_tray), win->settings.run_in_tray);

    textvn_settings_window_save_and_sync(win);
}

void textvn_settings_window_save_and_sync(TextVNSettingsWindow *win) {
    if (!win) return;

    if (win->combo_charset) win->settings.charset = (TextVNCharset)gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_charset));
    if (win->combo_method) win->settings.method = (TextVNMethod)gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_method));
    if (win->chk_spell_check) win->settings.spell_check = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_spell_check));
    if (win->chk_auto_restore) win->settings.auto_restore_english = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_auto_restore));
    if (win->chk_allow_macro) win->settings.allow_macro = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_allow_macro));
    if (win->chk_new_diacritic) win->settings.new_diacritic = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_new_diacritic));
    if (win->chk_show_dialog) win->settings.show_dialog_on_startup = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_show_dialog));
    if (win->chk_autostart) win->settings.autostart = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_autostart));
    if (win->chk_non_preedit) win->settings.non_preedit = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_non_preedit));
    if (win->chk_run_in_tray) win->settings.run_in_tray = gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_run_in_tray));

    textvn_settings_save_file(&win->settings, win->config_path);

    /* Sync via IPC client */
    if (!win->ipc_client) {
        win->ipc_client = lc_ipc_client_new("textvn-settings", NULL);
    }
    if (win->ipc_client) {
        lc_ipc_client_poll(win->ipc_client);
    }
}

void textvn_settings_window_destroy(TextVNSettingsWindow *win) {
    if (!win) return;
    if (win->ipc_client) {
        lc_ipc_client_free(win->ipc_client);
        win->ipc_client = NULL;
    }
#if defined(TEXTVN_GTK_MOCK)
    free(win->window);
    free(win->compact_box);
    free(win->expanded_box);
    free(win->btn_expand_toggle);
    free(win->combo_charset);
    free(win->combo_method);
    free(win->radio_ctrl_shift);
    free(win->radio_alt_z);
    free(win->chk_spell_check);
    free(win->chk_auto_restore);
    free(win->chk_allow_macro);
    free(win->chk_new_diacritic);
    free(win->chk_show_dialog);
    free(win->chk_autostart);
    free(win->chk_non_preedit);
    free(win->chk_run_in_tray);
#endif
    free(win);
}

static void on_btn_close_clicked(GtkButton *btn, gpointer user_data) {
    (void)btn;
    TextVNSettingsWindow *win = (TextVNSettingsWindow *)user_data;
    if (win && win->window) {
        textvn_settings_window_save_and_sync(win);
        gtk_window_close(GTK_WINDOW(win->window));
    }
}

static void on_btn_expand_clicked(GtkButton *btn, gpointer user_data) {
    (void)btn;
    TextVNSettingsWindow *win = (TextVNSettingsWindow *)user_data;
    if (win) {
        textvn_settings_window_toggle_expanded(win);
    }
}

static void on_btn_default_clicked(GtkButton *btn, gpointer user_data) {
    (void)btn;
    TextVNSettingsWindow *win = (TextVNSettingsWindow *)user_data;
    if (win) {
        textvn_settings_window_reset_defaults(win);
    }
}

static void on_btn_about_clicked(GtkButton *btn, gpointer user_data) {
    (void)btn;
    TextVNSettingsWindow *win = (TextVNSettingsWindow *)user_data;
    if (win && win->window) {
        GtkAlertDialog *dialog = gtk_alert_dialog_new(
            "TextVN 1.0.0 (Linux)\n\n"
            "Bộ gõ tiếng Việt chuyên nghiệp, bảo mật cao.\n"
            "Bản quyền (C) 2026 TextVN Contributors.\n"
            "Giấy phép: GNU General Public License v3."
        );
        gtk_alert_dialog_show(dialog, GTK_WINDOW(win->window));
    }
}

TextVNSettingsWindow *textvn_settings_window_new(GtkApplication *app, const char *custom_config_path) {
    TextVNSettingsWindow *win = (TextVNSettingsWindow *)calloc(1, sizeof(TextVNSettingsWindow));
    if (!win) return NULL;

    resolve_config_path(win->config_path, sizeof(win->config_path), custom_config_path);
    textvn_settings_load_file(&win->settings, win->config_path);

    /* 1. Main Window */
    win->window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(win->window), "TextVN - Bảng điều khiển");
    gtk_window_set_default_size(GTK_WINDOW(win->window), TEXTVN_WINDOW_WIDTH_COMPACT, TEXTVN_WINDOW_HEIGHT_COMPACT);
    gtk_window_set_resizable(GTK_WINDOW(win->window), FALSE);

    GtkWidget *main_vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_start(main_vbox, 12);
    gtk_widget_set_margin_end(main_vbox, 12);
    gtk_widget_set_margin_top(main_vbox, 12);
    gtk_widget_set_margin_bottom(main_vbox, 12);
    gtk_window_set_child(GTK_WINDOW(win->window), main_vbox);

    /* 2. Compact Area (Grid Controls + Buttons) */
    win->compact_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 16);
    gtk_box_append(GTK_BOX(main_vbox), win->compact_box);

    /* Left: Inputs Grid */
    GtkWidget *grid = gtk_grid_new();
    gtk_grid_set_row_spacing(GTK_GRID(grid), 10);
    gtk_grid_set_column_spacing(GTK_GRID(grid), 12);
    gtk_widget_set_hexpand(grid, TRUE);
    gtk_box_append(GTK_BOX(win->compact_box), grid);

    /* Bảng mã */
    GtkWidget *lbl_charset = gtk_label_new("Bảng mã:");
    gtk_widget_set_halign(lbl_charset, GTK_ALIGN_START);
    gtk_grid_attach(GTK_GRID(grid), lbl_charset, 0, 0, 1, 1);

    const char *charsets[] = {"1. Unicode dựng sẵn", "2. TCVN3 (ABC)", "3. VNI Windows", "4. Unicode tổ hợp", NULL};
    win->combo_charset = gtk_drop_down_new_from_strings(charsets);
    gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_charset), win->settings.charset);
    gtk_widget_set_hexpand(win->combo_charset, TRUE);
    gtk_grid_attach(GTK_GRID(grid), win->combo_charset, 1, 0, 1, 1);

    /* Kiểu gõ */
    GtkWidget *lbl_method = gtk_label_new("Kiểu gõ:");
    gtk_widget_set_halign(lbl_method, GTK_ALIGN_START);
    gtk_grid_attach(GTK_GRID(grid), lbl_method, 0, 1, 1, 1);

    const char *methods[] = {"1. Telex", "2. VNI", "3. VIQR", "4. Microsoft", NULL};
    win->combo_method = gtk_drop_down_new_from_strings(methods);
    gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_method), win->settings.method);
    gtk_widget_set_hexpand(win->combo_method, TRUE);
    gtk_grid_attach(GTK_GRID(grid), win->combo_method, 1, 1, 1, 1);

    /* Phím chuyển */
    GtkWidget *lbl_switch = gtk_label_new("Phím chuyển:");
    gtk_widget_set_halign(lbl_switch, GTK_ALIGN_START);
    gtk_grid_attach(GTK_GRID(grid), lbl_switch, 0, 2, 1, 1);

    GtkWidget *switch_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
    win->radio_ctrl_shift = gtk_check_button_new_with_label("Ctrl + Shift");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->radio_ctrl_shift), win->settings.switch_key == TEXTVN_SWITCH_CTRL_SHIFT);
    win->radio_alt_z = gtk_check_button_new_with_label("Alt + Z");
    gtk_check_button_set_group(GTK_CHECK_BUTTON(win->radio_alt_z), GTK_CHECK_BUTTON(win->radio_ctrl_shift));
    gtk_box_append(GTK_BOX(switch_box), win->radio_ctrl_shift);
    gtk_box_append(GTK_BOX(switch_box), win->radio_alt_z);
    gtk_grid_attach(GTK_GRID(grid), switch_box, 1, 2, 1, 1);

    /* Right: Action Buttons Column */
    GtkWidget *btn_vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    gtk_widget_set_valign(btn_vbox, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(win->compact_box), btn_vbox);

    GtkWidget *btn_close = gtk_button_new_with_label("Đóng");
    gtk_widget_set_size_request(btn_close, 110, -1);
    g_signal_connect(btn_close, "clicked", G_CALLBACK(on_btn_close_clicked), win);
    gtk_box_append(GTK_BOX(btn_vbox), btn_close);

    win->btn_expand_toggle = gtk_button_new_with_label("Mở rộng >>");
    gtk_widget_set_size_request(win->btn_expand_toggle, 110, -1);
    g_signal_connect(win->btn_expand_toggle, "clicked", G_CALLBACK(on_btn_expand_clicked), win);
    gtk_box_append(GTK_BOX(btn_vbox), win->btn_expand_toggle);

    GtkWidget *btn_default = gtk_button_new_with_label("Mặc định");
    gtk_widget_set_size_request(btn_default, 110, -1);
    g_signal_connect(btn_default, "clicked", G_CALLBACK(on_btn_default_clicked), win);
    gtk_box_append(GTK_BOX(btn_vbox), btn_default);

    GtkWidget *btn_about = gtk_button_new_with_label("Thông tin");
    gtk_widget_set_size_request(btn_about, 110, -1);
    g_signal_connect(btn_about, "clicked", G_CALLBACK(on_btn_about_clicked), win);
    gtk_box_append(GTK_BOX(btn_vbox), btn_about);

    /* 3. Expanded Area */
    win->expanded_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_visible(win->expanded_box, FALSE);
    gtk_box_append(GTK_BOX(main_vbox), win->expanded_box);

    /* Group: Tùy chọn gõ */
    GtkWidget *frame_opts = gtk_frame_new("Tùy chọn gõ");
    GtkWidget *box_opts = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(box_opts, 8);
    gtk_widget_set_margin_end(box_opts, 8);
    gtk_widget_set_margin_top(box_opts, 8);
    gtk_widget_set_margin_bottom(box_opts, 8);
    gtk_frame_set_child(GTK_FRAME(frame_opts), box_opts);

    win->chk_spell_check = gtk_check_button_new_with_label("Bật kiểm tra chính tả");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_spell_check), win->settings.spell_check);
    gtk_box_append(GTK_BOX(box_opts), win->chk_spell_check);

    win->chk_auto_restore = gtk_check_button_new_with_label("Tự động khôi phục phím với từ sai");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_auto_restore), win->settings.auto_restore_english);
    gtk_box_append(GTK_BOX(box_opts), win->chk_auto_restore);

    win->chk_allow_macro = gtk_check_button_new_with_label("Cho phép gõ tắt");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_allow_macro), win->settings.allow_macro);
    gtk_box_append(GTK_BOX(box_opts), win->chk_allow_macro);

    win->chk_new_diacritic = gtk_check_button_new_with_label("Bỏ dấu kiểu mới (òa, ùy)");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_new_diacritic), win->settings.new_diacritic);
    gtk_box_append(GTK_BOX(box_opts), win->chk_new_diacritic);

    gtk_box_append(GTK_BOX(win->expanded_box), frame_opts);

    /* Group: Hệ thống */
    GtkWidget *frame_sys = gtk_frame_new("Hệ thống");
    GtkWidget *box_sys = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(box_sys, 8);
    gtk_widget_set_margin_end(box_sys, 8);
    gtk_widget_set_margin_top(box_sys, 8);
    gtk_widget_set_margin_bottom(box_sys, 8);
    gtk_frame_set_child(GTK_FRAME(frame_sys), box_sys);

    win->chk_show_dialog = gtk_check_button_new_with_label("Bật hội thoại này khi khởi động");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_show_dialog), win->settings.show_dialog_on_startup);
    gtk_box_append(GTK_BOX(box_sys), win->chk_show_dialog);

    win->chk_autostart = gtk_check_button_new_with_label("Khởi động cùng OS");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_autostart), win->settings.autostart);
    gtk_box_append(GTK_BOX(box_sys), win->chk_autostart);

    win->chk_non_preedit = gtk_check_button_new_with_label("Gõ không gạch chân (Non-preedit)");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_non_preedit), win->settings.non_preedit);
    gtk_box_append(GTK_BOX(box_sys), win->chk_non_preedit);

    win->chk_run_in_tray = gtk_check_button_new_with_label("Chạy ngầm trong khay hệ thống");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_run_in_tray), win->settings.run_in_tray);
    gtk_box_append(GTK_BOX(box_sys), win->chk_run_in_tray);

    gtk_box_append(GTK_BOX(win->expanded_box), frame_sys);

    return win;
}

GtkWidget *textvn_settings_window_create(GtkApplication *app, const char *custom_config_path) {
    TextVNSettingsWindow *win = textvn_settings_window_new(app, custom_config_path);
    return win ? win->window : NULL;
}
