/* test_settings_window.c — Unit tests for GTK4 Settings Panel data model & serialization
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "settings_window.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

static void test_settings_defaults(void) {
    TextVNSettings s;
    textvn_settings_set_defaults(&s);

    assert(s.config_version == 1);
    assert(s.enabled == true);
    assert(s.charset == TEXTVN_CHARSET_UNICODE);
    assert(s.method == TEXTVN_METHOD_TELEX);
    assert(s.switch_key == TEXTVN_SWITCH_CTRL_SHIFT);

    assert(s.spell_check == true);
    assert(s.auto_restore_english == true);
    assert(s.allow_macro == false);
    assert(s.new_diacritic == true);

    assert(s.show_dialog_on_startup == true);
    assert(s.autostart == true);
    assert(s.non_preedit == true);
    assert(s.run_in_tray == true);

    assert(s.is_expanded == false);
}

static void test_settings_json_roundtrip(void) {
    TextVNSettings s;
    textvn_settings_set_defaults(&s);

    s.method = TEXTVN_METHOD_VNI;
    s.charset = TEXTVN_CHARSET_TCVN3;
    s.new_diacritic = false;
    s.show_dialog_on_startup = false;
    s.autostart = false;
    s.non_preedit = false;

    char *json = textvn_settings_to_json(&s);
    assert(json != NULL);
    assert(strstr(json, "\"method\": \"vni\"") != NULL);
    assert(strstr(json, "\"output_charset\": \"tcvn3\"") != NULL);
    assert(strstr(json, "\"diacritic_style\": \"old\"") != NULL);
    assert(strstr(json, "\"show_dialog_on_startup\": false") != NULL);
    assert(strstr(json, "\"autostart\": false") != NULL);
    assert(strstr(json, "\"non_preedit\": false") != NULL);

    TextVNSettings s2;
    assert(textvn_settings_load_from_json(&s2, json) == 0);
    assert(s2.method == TEXTVN_METHOD_VNI);
    assert(s2.charset == TEXTVN_CHARSET_TCVN3);
    assert(s2.new_diacritic == false);
    assert(s2.show_dialog_on_startup == false);
    assert(s2.autostart == false);
    assert(s2.non_preedit == false);

    free(json);
}

static void test_settings_unicode_decomposed(void) {
    TextVNSettings s;
    textvn_settings_set_defaults(&s);

    s.charset = TEXTVN_CHARSET_UNICODE_DECOMPOSED;
    char *json = textvn_settings_to_json(&s);
    assert(json != NULL);
    assert(strstr(json, "\"output_charset\": \"unicode_decomposed\"") != NULL);

    TextVNSettings s2;
    assert(textvn_settings_load_from_json(&s2, json) == 0);
    assert(s2.charset == TEXTVN_CHARSET_UNICODE_DECOMPOSED);

    free(json);
}

static void test_settings_file_save_and_load(void) {
    char temp_path[512];
#ifdef _WIN32
    char temp_dir[MAX_PATH];
    GetTempPathA(sizeof(temp_dir), temp_dir);
    snprintf(temp_path, sizeof(temp_path), "%stextvn_settings_test_%u.json", temp_dir, (unsigned)GetCurrentProcessId());
#else
    snprintf(temp_path, sizeof(temp_path), "/tmp/textvn_settings_test_%u.json", (unsigned)getpid());
#endif

    TextVNSettings s_save;
    textvn_settings_set_defaults(&s_save);
    s_save.method = TEXTVN_METHOD_VIQR;
    s_save.charset = TEXTVN_CHARSET_VNI_WINDOWS;
    s_save.spell_check = false;

    assert(textvn_settings_save_file(&s_save, temp_path) == 0);

    TextVNSettings s_load;
    assert(textvn_settings_load_file(&s_load, temp_path) == 0);

    assert(s_load.method == TEXTVN_METHOD_VIQR);
    assert(s_load.charset == TEXTVN_CHARSET_VNI_WINDOWS);
    assert(s_load.spell_check == false);

    remove(temp_path);
}

static void test_settings_window_toggle_expanded(void) {
    TextVNSettingsWindow win;
    memset(&win, 0, sizeof(win));
    textvn_settings_set_defaults(&win.settings);

    assert(win.settings.is_expanded == false);
    textvn_settings_window_toggle_expanded(&win);
    assert(win.settings.is_expanded == true);
    textvn_settings_window_toggle_expanded(&win);
    assert(win.settings.is_expanded == false);
}

static void test_settings_window_reset_defaults(void) {
    TextVNSettingsWindow win;
    memset(&win, 0, sizeof(win));
    textvn_settings_set_defaults(&win.settings);

    win.settings.method = TEXTVN_METHOD_VNI;
    win.settings.charset = TEXTVN_CHARSET_TCVN3;
    win.settings.show_dialog_on_startup = false;

    textvn_settings_window_reset_defaults(&win);

    assert(win.settings.method == TEXTVN_METHOD_TELEX);
    assert(win.settings.charset == TEXTVN_CHARSET_UNICODE);
    assert(win.settings.show_dialog_on_startup == true);
}

int main(void) {
    test_settings_defaults();
    test_settings_json_roundtrip();
    test_settings_unicode_decomposed();
    test_settings_file_save_and_load();
    test_settings_window_toggle_expanded();
    test_settings_window_reset_defaults();

    printf("All TextVN Linux settings unit tests passed successfully!\n");
    return 0;
}
