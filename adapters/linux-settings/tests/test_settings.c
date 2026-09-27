/* test_settings.c — Standalone test suite for TextVN Linux Settings Panel
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * NGUỒN SỰ THẬT: PLAN §2.3 M6, P3-5-ui-packaging-release.md
 * Verifies UniKey 4.6 RC2 parity, JSON persistence, and GTK4 widget synchronization.
 */

#define TEXTVN_GTK_MOCK 1
#include "gtk_mock.h"
#include "settings_window.h"

#include <stdio.h>
#include <assert.h>
#include <string.h>

#if defined(_WIN32)
#include <io.h>
#define unlink _unlink
#else
#include <unistd.h>
#endif

static void test_default_settings(void) {
    TextVNSettings s;
    textvn_settings_set_defaults(&s);

    assert(s.config_version == 1);
    assert(s.enabled == true);
    assert(s.charset == TEXTVN_CHARSET_UNICODE);
    assert(s.method == TEXTVN_METHOD_TELEX);
    assert(s.switch_key == TEXTVN_SWITCH_CTRL_SHIFT);

    /* Tùy chọn gõ */
    assert(s.spell_check == true);
    assert(s.auto_restore_english == true);
    assert(s.allow_macro == false);
    assert(s.new_diacritic == true);

    /* Hệ thống */
    assert(s.show_dialog_on_startup == true);
    assert(s.autostart == true);
    assert(s.non_preedit == true);
    assert(s.run_in_tray == true);

    /* UI State */
    assert(s.is_expanded == false);
}

static void test_json_parsing(void) {
    const char *json_sample =
        "{\n"
        "  \"config_version\": 1,\n"
        "  \"enabled\": false,\n"
        "  \"method\": \"vni\",\n"
        "  \"diacritic_style\": \"old\",\n"
        "  \"free_marking\": false,\n"
        "  \"auto_restore_english\": false,\n"
        "  \"allow_macro_when_vi_off\": true,\n"
        "  \"output_charset\": \"tcvn3\",\n"
        "  \"show_dialog_on_startup\": false,\n"
        "  \"autostart\": false,\n"
        "  \"non_preedit\": false,\n"
        "  \"run_in_tray\": false\n"
        "}\n";

    TextVNSettings s;
    int rc = textvn_settings_load_from_json(&s, json_sample);
    assert(rc == 0);

    assert(s.method == TEXTVN_METHOD_VNI);
    assert(s.charset == TEXTVN_CHARSET_TCVN3);
    assert(s.new_diacritic == false);
    assert(s.spell_check == false);
    assert(s.auto_restore_english == false);
    assert(s.allow_macro == true);
    assert(s.show_dialog_on_startup == false);
    assert(s.autostart == false);
    assert(s.non_preedit == false);
    assert(s.run_in_tray == false);

    /* Test other methods and charsets */
    const char *json_viqr = "{\"method\":\"viqr\",\"output_charset\":\"vni_windows\"}";
    rc = textvn_settings_load_from_json(&s, json_viqr);
    assert(rc == 0);
    assert(s.method == TEXTVN_METHOD_VIQR);
    assert(s.charset == TEXTVN_CHARSET_VNI_WINDOWS);

    const char *json_ms = "{\"method\":\"simple_telex\",\"output_charset\":\"unicode_decomposed\"}";
    rc = textvn_settings_load_from_json(&s, json_ms);
    assert(rc == 0);
    assert(s.method == TEXTVN_METHOD_MICROSOFT);
    assert(s.charset == TEXTVN_CHARSET_UNICODE_DECOMPOSED);
}

static void test_json_serialization(void) {
    TextVNSettings s;
    textvn_settings_set_defaults(&s);
    s.method = TEXTVN_METHOD_VNI;
    s.charset = TEXTVN_CHARSET_TCVN3;
    s.new_diacritic = false;
    s.spell_check = false;
    s.show_dialog_on_startup = false;
    s.autostart = false;
    s.non_preedit = false;

    char *json = textvn_settings_to_json(&s);
    assert(json != NULL);

    assert(strstr(json, "\"method\": \"vni\"") != NULL);
    assert(strstr(json, "\"output_charset\": \"tcvn3\"") != NULL);
    assert(strstr(json, "\"diacritic_style\": \"old\"") != NULL);
    assert(strstr(json, "\"free_marking\": false") != NULL);
    assert(strstr(json, "\"show_dialog_on_startup\": false") != NULL);
    assert(strstr(json, "\"autostart\": false") != NULL);
    assert(strstr(json, "\"non_preedit\": false") != NULL);

    /* Parse back to verify round-trip */
    TextVNSettings parsed;
    int rc = textvn_settings_load_from_json(&parsed, json);
    assert(rc == 0);
    assert(parsed.method == TEXTVN_METHOD_VNI);
    assert(parsed.charset == TEXTVN_CHARSET_TCVN3);
    assert(parsed.new_diacritic == false);
    assert(parsed.spell_check == false);
    assert(parsed.show_dialog_on_startup == false);
    assert(parsed.autostart == false);
    assert(parsed.non_preedit == false);

    free(json);
}

static void test_file_persistence_roundtrip(void) {
    const char *test_path = "test_textvn_settings_tmp.json";
    unlink(test_path);

    TextVNSettings s1;
    textvn_settings_set_defaults(&s1);
    s1.method = TEXTVN_METHOD_VIQR;
    s1.charset = TEXTVN_CHARSET_VNI_WINDOWS;
    s1.auto_restore_english = false;
    s1.autostart = false;

    int rc = textvn_settings_save_file(&s1, test_path);
    assert(rc == 0);

    TextVNSettings s2;
    rc = textvn_settings_load_file(&s2, test_path);
    assert(rc == 0);

    assert(s2.method == TEXTVN_METHOD_VIQR);
    assert(s2.charset == TEXTVN_CHARSET_VNI_WINDOWS);
    assert(s2.auto_restore_english == false);
    assert(s2.autostart == false);
    assert(s2.spell_check == true);
    assert(s2.non_preedit == true);

    unlink(test_path);
}

static void test_window_creation_and_initial_state(void) {
    const char *test_path = "test_textvn_win_tmp.json";
    unlink(test_path);

    TextVNSettingsWindow *win = textvn_settings_window_new(NULL, test_path);
    assert(win != NULL);
    assert(win->window != NULL);

    /* Compact size ~505x245px */
    assert(win->window->width == TEXTVN_WINDOW_WIDTH_COMPACT);
    assert(win->window->height == TEXTVN_WINDOW_HEIGHT_COMPACT);
    assert(win->settings.is_expanded == false);

    /* Expanded box initially hidden */
    assert(win->expanded_box != NULL);
    assert(gtk_widget_get_visible(win->expanded_box) == FALSE);

    /* Check initial controls state */
    assert(win->combo_charset != NULL);
    assert(gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_charset)) == TEXTVN_CHARSET_UNICODE);
    assert(win->combo_method != NULL);
    assert(gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_method)) == TEXTVN_METHOD_TELEX);

    assert(win->chk_spell_check != NULL);
    assert(gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_spell_check)) == TRUE);
    assert(win->chk_show_dialog != NULL);
    assert(gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_show_dialog)) == TRUE);
    assert(win->chk_autostart != NULL);
    assert(gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_autostart)) == TRUE);
    assert(win->chk_non_preedit != NULL);
    assert(gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_non_preedit)) == TRUE);

    textvn_settings_window_destroy(win);
    unlink(test_path);
}

static void test_toggle_compact_expanded(void) {
    const char *test_path = "test_textvn_win_expand.json";
    unlink(test_path);

    TextVNSettingsWindow *win = textvn_settings_window_new(NULL, test_path);
    assert(win != NULL);

    /* 1. Initially Compact (505x245) */
    assert(win->settings.is_expanded == false);
    assert(win->window->height == TEXTVN_WINDOW_HEIGHT_COMPACT);
    assert(gtk_widget_get_visible(win->expanded_box) == FALSE);
    assert(strcmp(gtk_button_get_label(GTK_BUTTON(win->btn_expand_toggle)), "Mở rộng >>") == 0);

    /* 2. Toggle to Expanded (505x490) */
    textvn_settings_window_toggle_expanded(win);
    assert(win->settings.is_expanded == true);
    assert(win->window->height == TEXTVN_WINDOW_HEIGHT_EXPANDED);
    assert(gtk_widget_get_visible(win->expanded_box) == TRUE);
    assert(strcmp(gtk_button_get_label(GTK_BUTTON(win->btn_expand_toggle)), "<< Thu nhỏ") == 0);

    /* 3. Toggle back to Compact (505x245) */
    textvn_settings_window_toggle_expanded(win);
    assert(win->settings.is_expanded == false);
    assert(win->window->height == TEXTVN_WINDOW_HEIGHT_COMPACT);
    assert(gtk_widget_get_visible(win->expanded_box) == FALSE);
    assert(strcmp(gtk_button_get_label(GTK_BUTTON(win->btn_expand_toggle)), "Mở rộng >>") == 0);

    textvn_settings_window_destroy(win);
    unlink(test_path);
}

static void test_reset_to_defaults(void) {
    const char *test_path = "test_textvn_win_reset.json";
    unlink(test_path);

    TextVNSettingsWindow *win = textvn_settings_window_new(NULL, test_path);
    assert(win != NULL);

    /* Mutate widget values away from default */
    gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_method), TEXTVN_METHOD_VNI);
    gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_charset), TEXTVN_CHARSET_TCVN3);
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_spell_check), FALSE);
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_autostart), FALSE);
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_non_preedit), FALSE);
    textvn_settings_window_save_and_sync(win);

    assert(win->settings.method == TEXTVN_METHOD_VNI);
    assert(win->settings.autostart == false);

    /* Reset to defaults */
    textvn_settings_window_reset_defaults(win);

    /* Verify restored defaults */
    assert(win->settings.method == TEXTVN_METHOD_TELEX);
    assert(win->settings.charset == TEXTVN_CHARSET_UNICODE);
    assert(win->settings.spell_check == true);
    assert(win->settings.autostart == true);
    assert(win->settings.non_preedit == true);

    /* Verify widgets updated */
    assert(gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_method)) == TEXTVN_METHOD_TELEX);
    assert(gtk_drop_down_get_selected(GTK_DROP_DOWN(win->combo_charset)) == TEXTVN_CHARSET_UNICODE);
    assert(gtk_check_button_get_active(GTK_CHECK_BUTTON(win->chk_autostart)) == TRUE);

    textvn_settings_window_destroy(win);
    unlink(test_path);
}

static void test_save_and_ipc_sync(void) {
    const char *test_path = "test_textvn_win_sync.json";
    unlink(test_path);

    TextVNSettingsWindow *win = textvn_settings_window_new(NULL, test_path);
    assert(win != NULL);

    /* User selects Microsoft (simple_telex) and turns off autostart */
    gtk_drop_down_set_selected(GTK_DROP_DOWN(win->combo_method), TEXTVN_METHOD_MICROSOFT);
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_autostart), FALSE);
    gtk_check_button_set_active(GTK_CHECK_BUTTON(win->chk_show_dialog), FALSE);

    textvn_settings_window_save_and_sync(win);

    /* Verify persisted to disk */
    TextVNSettings disk_settings;
    int rc = textvn_settings_load_file(&disk_settings, test_path);
    assert(rc == 0);
    assert(disk_settings.method == TEXTVN_METHOD_MICROSOFT);
    assert(disk_settings.autostart == false);
    assert(disk_settings.show_dialog_on_startup == false);

    textvn_settings_window_destroy(win);
    unlink(test_path);
}

int main(void) {
    test_default_settings();
    test_json_parsing();
    test_json_serialization();
    test_file_persistence_roundtrip();
    test_window_creation_and_initial_state();
    test_toggle_compact_expanded();
    test_reset_to_defaults();
    test_save_and_ipc_sync();

    printf("All TextVN Linux Settings Panel unit tests passed successfully!\n");
    return 0;
}
