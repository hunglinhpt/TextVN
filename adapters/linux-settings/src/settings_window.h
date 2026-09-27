/* settings_window.h — UniKey 4.6 RC2 style GTK4 Settings Panel for TextVN
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: PLAN §2.3 M6, P3-5-ui-packaging-release.md
 * Compact: ~505x245px | Expanded: ~505x490px
 */

#ifndef TEXTVN_SETTINGS_WINDOW_H
#define TEXTVN_SETTINGS_WINDOW_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#if defined(TEXTVN_GTK_MOCK)
#include "gtk_mock.h"
#else
#include <gtk/gtk.h>
#endif

#include "linux_common.h"

#ifdef __cplusplus
extern "C" {
#endif

#define TEXTVN_WINDOW_WIDTH_COMPACT   505
#define TEXTVN_WINDOW_HEIGHT_COMPACT  245
#define TEXTVN_WINDOW_WIDTH_EXPANDED  505
#define TEXTVN_WINDOW_HEIGHT_EXPANDED 490

typedef enum {
    TEXTVN_CHARSET_UNICODE = 0,
    TEXTVN_CHARSET_TCVN3,
    TEXTVN_CHARSET_VNI_WINDOWS,
    TEXTVN_CHARSET_UNICODE_DECOMPOSED
} TextVNCharset;

typedef enum {
    TEXTVN_METHOD_TELEX = 0,
    TEXTVN_METHOD_VNI,
    TEXTVN_METHOD_VIQR,
    TEXTVN_METHOD_MICROSOFT
} TextVNMethod;

typedef enum {
    TEXTVN_SWITCH_CTRL_SHIFT = 0,
    TEXTVN_SWITCH_ALT_Z
} TextVNSwitchKey;

/* Model data */
typedef struct {
    uint32_t config_version;
    bool enabled;
    TextVNCharset charset;
    TextVNMethod method;
    TextVNSwitchKey switch_key;

    /* Tùy chọn gõ */
    bool spell_check;          /* Bật kiểm tra chính tả */
    bool auto_restore_english; /* Tự động khôi phục phím với từ sai */
    bool allow_macro;          /* Cho phép gõ tắt */
    bool new_diacritic;        /* Bỏ dấu kiểu mới (òa, ùy) */

    /* Hệ thống */
    bool show_dialog_on_startup; /* Bật hội thoại này khi khởi động */
    bool autostart;              /* Khởi động cùng OS */
    bool non_preedit;            /* Gõ không gạch chân */
    bool run_in_tray;            /* Chạy ngầm trong khay hệ thống */

    /* UI State */
    bool is_expanded;
} TextVNSettings;

/* Window context */
typedef struct {
    GtkWidget *window;
    GtkWidget *compact_box;
    GtkWidget *expanded_box;
    GtkWidget *btn_expand_toggle;

    /* Controls Compact */
    GtkWidget *combo_charset;
    GtkWidget *combo_method;
    GtkWidget *radio_ctrl_shift;
    GtkWidget *radio_alt_z;

    /* Checkboxes Tùy chọn gõ (Expanded) */
    GtkWidget *chk_spell_check;
    GtkWidget *chk_auto_restore;
    GtkWidget *chk_allow_macro;
    GtkWidget *chk_new_diacritic;

    /* Checkboxes Hệ thống (Expanded) */
    GtkWidget *chk_show_dialog;
    GtkWidget *chk_autostart;
    GtkWidget *chk_non_preedit;
    GtkWidget *chk_run_in_tray;

    TextVNSettings settings;
    lc_ipc_client *ipc_client;
    char config_path[512];
} TextVNSettingsWindow;

/* Functions */
void  textvn_settings_set_defaults(TextVNSettings *s);
int   textvn_settings_load_from_json(TextVNSettings *s, const char *json_str);
char* textvn_settings_to_json(const TextVNSettings *s);
int   textvn_settings_load_file(TextVNSettings *s, const char *custom_path);
int   textvn_settings_save_file(const TextVNSettings *s, const char *custom_path);

void  textvn_settings_window_toggle_expanded(TextVNSettingsWindow *win);
void  textvn_settings_window_reset_defaults(TextVNSettingsWindow *win);
void  textvn_settings_window_save_and_sync(TextVNSettingsWindow *win);

TextVNSettingsWindow *textvn_settings_window_new(GtkApplication *app, const char *custom_config_path);
void  textvn_settings_window_destroy(TextVNSettingsWindow *win);
GtkWidget *textvn_settings_window_create(GtkApplication *app, const char *custom_config_path);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_SETTINGS_WINDOW_H */
