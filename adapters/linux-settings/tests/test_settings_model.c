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
#include "textvn_ffi.h" /* IME_ERR_* — mã lỗi của ime_settings_* */

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
    /* Tro ly ngu phap: gia tri khop enum engine (off / highlight / autocorrect). */
    assert(strcmp(tv_assist_mode_values[0], "off") == 0);
    assert(strcmp(tv_assist_mode_values[1], "highlight") == 0);
    assert(strcmp(tv_assist_mode_values[2], "autocorrect") == 0);
    assert(tv_assist_mode_labels[TV_ASSIST_MODE_COUNT] == NULL);
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
    /* Vong 17 (default cu): thieu khoa diacritic_style = "old" -> new_diacritic == 0. */
    assert(s.vi_enabled == 1 && s.new_diacritic == 0);
    assert(s.free_marking && s.auto_restore_english && s.auto_capitalize);
    assert(!s.quick_telex && !s.allow_macro_when_vi_off && !s.config_was_corrupt);
    /* Tro ly ngu phap: file chua co khoi assist -> mac dinh schema
     * (Vong 26: HIGHLIGHT = 1, 700 ms; tu sua van tat). */
    assert(s.assist_mode == 1 && s.assist_idle_ms == TV_ASSIST_DEFAULT_IDLE_MS);
    /* P6: dự đoán văn bản opt-in — mặc định TẮT (khớp schema Windows/macOS). */
    assert(!s.suggest_enabled);
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

/* Tro ly ngu phap (P4-0): set/get `assist.mode` + `assist.display.idle_ms` phai
 * di qua dung duong long C-API (ime_settings_* tu parse SettingsDoc) — CAM parse
 * JSON tay trong C. */
static void test_assist(tv_paths *p) {
    write_file(p->config,
               "{\"config_version\":1,\"method\":\"vni\",\"macros\":"
               "[{\"trigger\":\"a\",\"expand\":\"b\",\"when\":\"vi_on\"}]}");
    tv_snapshot s;
    tv_snapshot_load(&s, p);
    /* Vong 26: file cũ thiếu assist -> assist_mode mặc định HIGHLIGHT (1). */
    assert(s.assist_mode == 1 && s.assist_idle_ms == 700 && "file cũ thiếu assist → mặc định");

    /* Ghi qua model — mỗi thao tác là một patch khoá riêng. */
    assert(tv_config_set_assist_mode(p, 1) == 0);
    assert(tv_config_set_assist_idle_ms(p, 250) == 0);
    tv_snapshot_load(&s, p);
    assert(s.assist_mode == 1 && s.assist_idle_ms == 250);

    /* assist.auto_fix: mac dinh off; ghi/doc qua model; chi so ngoai khoang bi tu choi. */
    assert(s.assist_auto_fix == 0);
    assert(tv_config_set_assist_auto_fix(p, 1) == 0);
    tv_snapshot_load(&s, p);
    assert(s.assist_auto_fix == 1 && s.assist_mode == 1);
    assert(tv_config_set_assist_auto_fix(p, TV_ASSIST_AUTO_FIX_COUNT) != 0);
    assert(tv_config_set_assist_auto_fix(p, -1) != 0);
    assert(strcmp(tv_assist_auto_fix_values[0], "off") == 0);
    assert(strcmp(tv_assist_auto_fix_values[1], "safe") == 0);
    assert(tv_assist_auto_fix_labels[TV_ASSIST_AUTO_FIX_COUNT] == NULL);
    assert(tv_config_set_assist_auto_fix(p, 0) == 0);

    /* Đọc riêng từng key con bằng C API thật. */
    ime_settings *doc = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    assert(doc);
    char *mode = ime_settings_get_str(doc, "assist.mode");
    assert(mode && strcmp(mode, "highlight") == 0);
    ime_settings_string_free(mode);
    assert(ime_settings_get_u32(doc, "assist.display.idle_ms") == 250);
    /* Leaf lồng lấy mặc định từ schema khi file không có. */
    assert(ime_settings_get_bool(doc, "assist.display.show_on_sentence_end") == 1);
    /* Kieu sai / khoa la bi tu choi; khong tao rac trong khoi `assist`. */
    assert(ime_settings_get_bool(doc, "assist.mode") == -1);
    assert(ime_settings_get_str(doc, "assist") == NULL);
    assert(ime_settings_set_bool(doc, "assist.bogus", 1) == IME_ERR_INVALID_ARG);
    assert(ime_settings_set_u32(doc, "assist.auto_fix", 1) == IME_ERR_INVALID_ARG);
    assert(ime_settings_set_str(doc, "assist.mode", "dvorak") == IME_ERR_CONFIG);
    ime_settings_free(doc);

    /* File vẫn còn: key phẳng cũ, macros, key lạ trong khối assist. */
    assert(tv_config_set_str(p, "assist.mode", "off") == 0);
    const char *out = read_file(p->config);
    assert(strstr(out, "\"vi_on\"") && "ho g tat phai con");
    assert(strstr(out, "\"vni\"") && "method phai con");
    assert(strstr(out, "\"idle_ms\": 250") && "idle_ms khong bi reset");
    tv_snapshot_load(&s, p);
    /* Đã set "off" tường minh → đọc ra 0 (không phải default highlight). */
    assert(s.assist_mode == 0 && s.assist_idle_ms == 250);

    /* Giá trị out-of-range của mode index bị từ chối, file giữ nguyên. */
    assert(tv_config_set_assist_mode(p, TV_ASSIST_MODE_COUNT) != 0);
    assert(tv_config_set_assist_mode(p, -1) != 0);
    tv_snapshot_load(&s, p);
    assert(s.assist_mode == 0);
}

/* JSON có khối assist THIẾU key con + key lạ trong khối: lấy mặc định, không mất key lạ. */
static void test_assist_partial_block(tv_paths *p) {
    write_file(p->config,
               "{\"config_version\":1,\"assist\":{\"per_app\":{\"code.exe\":false},"
               "\"custom\":9}}");
    assert(tv_config_set_assist_idle_ms(p, 90) == 0);
    ime_settings *doc = ime_settings_load(p->config, IME_SETTINGS_CONFIG);
    assert(doc);
    /* Mặc định đủ: mode và display thiếu đều lay từ schema. */
    char *mode = ime_settings_get_str(doc, "assist.mode");
    /* Vong 26: thieu mode -> mac dinh highlight. */
    assert(mode && strcmp(mode, "highlight") == 0);
    ime_settings_string_free(mode);
    assert(ime_settings_get_bool(doc, "assist.display.show_on_sentence_end") == 1);
    assert(ime_settings_get_u32(doc, "assist.display.idle_ms") == 90);
    ime_settings_free(doc);
    const char *out = read_file(p->config);
    assert(strstr(out, "\"code.exe\": false") && "per_app phai con nguyen");
    assert(strstr(out, "\"custom\": 9") && "key la trong assist phai con");
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
    /* Mặc định 0.2.33 = dấu cũ (new_diacritic == 0). */
    assert(s.method == 0 && s.charset == 0 && !s.new_diacritic && !s.quick_telex);
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
    test_assist(&p);
    test_assist_partial_block(&p);

    char cmd[600];
    snprintf(cmd, sizeof(cmd), "rm -rf %s", g_dir);
    assert(system(cmd) == 0);
    printf("test_settings_model: OK\n");
    return 0;
}
