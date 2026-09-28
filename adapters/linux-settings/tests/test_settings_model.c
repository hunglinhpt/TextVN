/* test_settings_model.c — Logic bảng điều khiển Linux với thư viện config THẬT
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Kiểm các lỗi thật của bản cũ: lưu một checkbox từng ghi đè cả file (mất gõ tắt,
 * emoji, khoá lạ), ép auto_capitalize=true, nhãn "Microsoft" cho Telex đơn giản,
 * lần lưu đầu thất bại khi ~/.config/TextVN chưa có.
 */

#define _POSIX_C_SOURCE 200809L

#include "settings_model.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "textvn_settings.h"

static char g_dir[] = "/tmp/textvn_settings_XXXXXX";

static void write_file(const char *path, const char *text) {
    FILE *f = fopen(path, "wb");
    assert(f);
    assert(fwrite(text, 1, strlen(text), f) == strlen(text));
    fclose(f);
}

static char *read_file(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    static char buf[8192];
    size_t n = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[n] = '\0';
    return buf;
}

static void test_labels_and_values(void) {
    assert(strcmp(tv_method_labels[3], "Telex đơn giản") == 0);
    assert(strcmp(tv_method_values[3], "simple_telex") == 0);
    assert(tv_method_labels[TV_METHOD_COUNT] == NULL);
    assert(tv_charset_labels[TV_CHARSET_COUNT] == NULL);
    assert(tv_index_of(tv_charset_values, TV_CHARSET_COUNT, "vni_windows") == 3);
    assert(tv_index_of(tv_charset_values, TV_CHARSET_COUNT, "nope") == 0);
    assert(tv_index_of(tv_charset_values, TV_CHARSET_COUNT, NULL) == 0);
}

static void test_paths(void) {
    tv_paths p;
    assert(tv_paths_resolve(&p, "/x/y/config.json") == 0);
    assert(strcmp(p.state, "/x/y/state.json") == 0);
    setenv("XDG_CONFIG_HOME", "/cfg", 1);
    assert(tv_paths_resolve(&p, NULL) == 0);
    assert(strcmp(p.config, "/cfg/TextVN/config.json") == 0);
    assert(strcmp(p.state, "/cfg/TextVN/state.json") == 0);
    unsetenv("XDG_CONFIG_HOME");
}

static void test_defaults_when_missing(tv_paths *p) {
    tv_snapshot s;
    tv_snapshot_load(&s, p);
    assert(s.charset == 0 && s.method == 0);
    assert(s.vi_enabled == 1 && s.new_diacritic == 1);
    assert(s.free_marking && s.auto_restore_english && s.auto_capitalize);
    assert(!s.quick_telex && !s.allow_macro_when_vi_off && !s.config_was_corrupt);
}

static void test_patch_keeps_user_data(tv_paths *p) {
    write_file(p->config,
               "{\"config_version\":1,\"auto_capitalize\":false,"
               "\"hotkeys\":{\"toggle_vi_en\":\"Ctrl+Shift+Space\"},"
               "\"macros\":[{\"trigger\":\"vn\",\"expand\":\"Việt Nam\",\"when\":\"vi_on\"}],"
               "\"emoji\":[{\"trigger\":\":ok\",\"glyph\":\"👌\"}]}");
    assert(tv_config_set_bool(p, "quick_telex", 1) == 0);
    assert(tv_config_set_str(p, "method", tv_method_values[3]) == 0);
    assert(tv_config_set_str(p, "output_charset", "tcvn3") == 0);
    assert(tv_config_set_str(p, "diacritic_style", "old") == 0);
    const char *out = read_file(p->config);
    assert(strstr(out, "\"toggle_vi_en\": \"Ctrl+Shift+Space\"") && "khoá lạ phải còn");
    assert(strstr(out, "\"vi_on\"") && "gõ tắt phải còn nguyên");
    assert(strstr(out, "👌") && "emoji phải còn");
    assert(strstr(out, "\"auto_capitalize\": false") && "không bị ép về true");

    tv_snapshot s;
    tv_snapshot_load(&s, p);
    assert(s.quick_telex && s.method == 3 && s.charset == 2 && !s.new_diacritic);
    assert(!s.auto_capitalize);

    /* Giá trị ngoài schema bị từ chối, file giữ nguyên. */
    assert(tv_config_set_str(p, "method", "dvorak") != 0);
    assert(tv_config_set_bool(p, "no_such_option", 1) != 0);
    tv_snapshot_load(&s, p);
    assert(s.method == 3);
}

static void test_state(tv_paths *p) {
    assert(tv_state_set_enabled(p, 0) == 0);
    tv_snapshot s;
    tv_snapshot_load(&s, p);
    assert(s.vi_enabled == 0);
    assert(tv_state_set_enabled(p, 1) == 0);
    tv_snapshot_load(&s, p);
    assert(s.vi_enabled == 1);
}

static void test_macros(tv_paths *p) {
    int space = -1;
    char *text = tv_macros_load(p, &space);
    assert(text && strcmp(text, "vn = Việt Nam\n") == 0 && space == 0);
    ime_settings_string_free(text);

    uint32_t line = 0;
    const char *msg = NULL;
    int rc = tv_macros_save(p, "vn = Việt Nam\nsai dòng\n", 1, &line, &msg);
    assert(rc > 0 && line == 2 && msg && strstr(msg, "="));
    text = tv_macros_load(p, &space);
    assert(strcmp(text, "vn = Việt Nam\n") == 0 && space == 0 && "lỗi không đổi gì");
    ime_settings_string_free(text);

    assert(tv_macros_save(p, "vn = Việt Nam\ncty = Công ty\\nTNHH\n", 1, &line, &msg) == 0);
    text = tv_macros_load(p, &space);
    assert(strcmp(text, "vn = Việt Nam\ncty = Công ty\\nTNHH\n") == 0 && space == 1);
    ime_settings_string_free(text);
    assert(strstr(read_file(p->config), "\"vi_on\"") && "giữ `when` của trigger cũ");
}

static void test_reset_defaults(tv_paths *p) {
    assert(tv_config_reset_defaults(p) == 0);
    tv_snapshot s;
    tv_snapshot_load(&s, p);
    assert(s.method == 0 && s.charset == 0 && s.new_diacritic && !s.quick_telex);
    assert(s.auto_capitalize);
    int space = 0;
    char *text = tv_macros_load(p, &space);
    assert(strstr(text, "cty = ") && "Mặc định không xoá gõ tắt");
    ime_settings_string_free(text);
    assert(strstr(read_file(p->config), "👌"));
}

static void test_corrupt_file(tv_paths *p) {
    write_file(p->config, "{ hỏng");
    tv_snapshot s;
    tv_snapshot_load(&s, p);
    assert(s.config_was_corrupt && s.method == 0);
    assert(tv_config_set_bool(p, "quick_telex", 1) == 0);
    char bak[1100];
    snprintf(bak, sizeof(bak), "%s.bak", p->config);
    assert(strcmp(read_file(bak), "{ hỏng") == 0 && "bản hỏng được giữ lại");
    tv_snapshot_load(&s, p);
    assert(!s.config_was_corrupt && s.quick_telex);
}

int main(void) {
    assert(mkdtemp(g_dir));
    tv_paths p;
    char cfg[512];
    /* Thư mục con chưa tồn tại: lần lưu đầu phải tự tạo (bug bản cũ). */
    snprintf(cfg, sizeof(cfg), "%s/TextVN/config.json", g_dir);
    assert(tv_paths_resolve(&p, cfg) == 0);

    test_labels_and_values();
    test_paths();
    test_defaults_when_missing(&p);
    test_state(&p);
    test_patch_keeps_user_data(&p);
    test_macros(&p);
    test_reset_defaults(&p);
    test_corrupt_file(&p);

    char cmd[600];
    snprintf(cmd, sizeof(cmd), "rm -rf %s", g_dir);
    assert(system(cmd) == 0);
    printf("test_settings_model: OK\n");
    return 0;
}
