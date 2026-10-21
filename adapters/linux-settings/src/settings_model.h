/* settings_model.h — Logic bảng điều khiển TextVN (không phụ thuộc GTK, test được)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * UI thống nhất với bảng điều khiển Windows (tray/src/settings_dialog.rs) — cùng tuỳ chọn,
 * cùng nhãn, cùng thứ tự; đặc tả: docs/release/ui-spec.md.
 *
 * Mọi thay đổi là VÁ MỘT KHOÁ trên file mới nhất (ime_settings_* — textvn_settings.h):
 * không bao giờ ghi đè macros/emoji/english_words hay khoá người dùng tự thêm, và không
 * đè trạng thái V/E mà IME vừa ghi bằng bản cũ đang hiển thị.
 */

#ifndef TEXTVN_SETTINGS_MODEL_H
#define TEXTVN_SETTINGS_MODEL_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TV_CHARSET_COUNT 4
#define TV_METHOD_COUNT  4
/* `assist.mode` (P4-0 §7) — cùng nhãn/giá trị với bảng Windows khi phần UI Trợ lý
 * được làm cho ba nền tảng; API model bên dưới đã sẵn sàng (nhãn chỉ dùng hiển thị). */
#define TV_ASSIST_MODE_COUNT 3
#define TV_ASSIST_AUTO_FIX_COUNT 2
/* schema `assist.display.idle_ms` (P4-0 §7) — dùng khi file không có leaf này. */
#define TV_ASSIST_DEFAULT_IDLE_MS 700

/* Nhãn hiển thị (khớp Windows) và giá trị trong config.json, cùng chỉ số. */
extern const char *const tv_charset_labels[TV_CHARSET_COUNT + 1];
extern const char *const tv_charset_values[TV_CHARSET_COUNT];
extern const char *const tv_method_labels[TV_METHOD_COUNT + 1];
extern const char *const tv_method_values[TV_METHOD_COUNT];
extern const char *const tv_assist_mode_labels[TV_ASSIST_MODE_COUNT + 1];
extern const char *const tv_assist_mode_values[TV_ASSIST_MODE_COUNT];
/* ssist.auto_fix (off | safe) — cùng nhãn với bảng Windows/macOS. */
extern const char *const tv_assist_auto_fix_labels[TV_ASSIST_AUTO_FIX_COUNT + 1];
extern const char *const tv_assist_auto_fix_values[TV_ASSIST_AUTO_FIX_COUNT];

extern const char *const tv_help_text;
extern const char *const tv_about_text;

/* Chỉ số của `value` trong `values`; không có → 0 (mặc định). */
int tv_index_of(const char *const *values, int n, const char *value);

typedef struct tv_paths {
    char config[1024];
    char state[1024];
} tv_paths;

/* ~/.config/TextVN/{config,state}.json; `custom_config` (tuỳ chọn --config) thì state.json
 * nằm cùng thư mục với nó. 0 = OK. */
int tv_paths_resolve(tv_paths *p, const char *custom_config);

/* Ảnh chụp giá trị để đổ lên widget. */
typedef struct tv_snapshot {
    int charset;           /* chỉ số tv_charset_values */
    int method;            /* chỉ số tv_method_values */
    int vi_enabled;        /* state.json global_enabled */
    int new_diacritic;     /* diacritic_style == "new" */
    int auto_restore_english;
    int free_marking;
    int auto_capitalize;
    int quick_telex;
    int allow_macro_when_vi_off;
    int config_was_corrupt; /* file hỏng — lần lưu đầu sẽ sao lưu .bak */
    /* Trợ lý ngữ pháp (P4-0 §7) — qua đường lồng C-API "assist.mode" /
     * "assist.display.idle_ms"; mặc định TẮT, idle 700 ms (schema không đổi). */
    int assist_mode;        /* chỉ số tv_assist_mode_values */
    int assist_auto_fix;    /* chỉ số tv_assist_auto_fix_values */
    uint32_t assist_idle_ms;
} tv_snapshot;

void tv_snapshot_load(tv_snapshot *s, const tv_paths *p);

/* Vá một khoá rồi ghi nguyên tử. 0 = OK. */
int tv_config_set_bool(const tv_paths *p, const char *key, int value);
int tv_config_set_str(const tv_paths *p, const char *key, const char *value);
int tv_config_set_u32(const tv_paths *p, const char *key, uint32_t value);
int tv_state_set_enabled(const tv_paths *p, int enabled);
/* Trợ lý ngữ pháp qua đúng đường lồng C-API: `assist.mode` (chỉ số
 * tv_assist_mode_values) và `assist.display.idle_ms`. IME_SETTINGS_* tự parse —
 * KHÔNG parse JSON tay ở đây (kiến trúc). 0 = OK, <0 = lỗi. */
int tv_config_set_assist_mode(const tv_paths *p, int mode_index);
int tv_config_set_assist_idle_ms(const tv_paths *p, uint32_t idle_ms);
/* ssist.auto_fix (chỉ số tv_assist_auto_fix_values). 0 = OK, <0 = lỗi. */
int tv_config_set_assist_auto_fix(const tv_paths *p, int fix_index);
/* Nút Mặc định: mọi tuỳ chọn về mặc định, giữ gõ tắt/emoji/từ tiếng Anh. */
int tv_config_reset_defaults(const tv_paths *p);

/* Bảng gõ tắt dạng text (free bằng ime_settings_string_free). *trigger_space = 1 nếu
 * bung bằng Space, 0 nếu Tab. */
/* Tu dien EN: doc/ghi danh sach tu (moi dong mot tu). NULL/-1 khi loi. */
char *tv_english_words_load(const tv_paths *p);
int tv_english_words_save(const tv_paths *p, const char *text);

char *tv_macros_load(const tv_paths *p, int *trigger_space);
/* 0 = đã lưu. Lỗi nội dung: trả >0, *bad_line = dòng (đếm từ 1), *message = thông báo
 * tiếng Việt tĩnh. Lỗi ghi file: trả <0. */
int tv_macros_save(const tv_paths *p, const char *text, int trigger_space, uint32_t *bad_line,
                   const char **message);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_SETTINGS_MODEL_H */
