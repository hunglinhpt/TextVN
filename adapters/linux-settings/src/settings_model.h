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

/* Nhãn hiển thị (khớp Windows) và giá trị trong config.json, cùng chỉ số. */
extern const char *const tv_charset_labels[TV_CHARSET_COUNT + 1];
extern const char *const tv_charset_values[TV_CHARSET_COUNT];
extern const char *const tv_method_labels[TV_METHOD_COUNT + 1];
extern const char *const tv_method_values[TV_METHOD_COUNT];

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
} tv_snapshot;

void tv_snapshot_load(tv_snapshot *s, const tv_paths *p);

/* Vá một khoá rồi ghi nguyên tử. 0 = OK. */
int tv_config_set_bool(const tv_paths *p, const char *key, int value);
int tv_config_set_str(const tv_paths *p, const char *key, const char *value);
int tv_state_set_enabled(const tv_paths *p, int enabled);
/* Nút Mặc định: mọi tuỳ chọn về mặc định, giữ gõ tắt/emoji/từ tiếng Anh. */
int tv_config_reset_defaults(const tv_paths *p);

/* Bảng gõ tắt dạng text (free bằng ime_settings_string_free). *trigger_space = 1 nếu
 * bung bằng Space, 0 nếu Tab. */
char *tv_macros_load(const tv_paths *p, int *trigger_space);
/* 0 = đã lưu. Lỗi nội dung: trả >0, *bad_line = dòng (đếm từ 1), *message = thông báo
 * tiếng Việt tĩnh. Lỗi ghi file: trả <0. */
int tv_macros_save(const tv_paths *p, const char *text, int trigger_space, uint32_t *bad_line,
                   const char **message);

#ifdef __cplusplus
}
#endif

#endif /* TEXTVN_SETTINGS_MODEL_H */
