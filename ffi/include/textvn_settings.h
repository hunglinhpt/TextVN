/* textvn_settings.h — C API cho bảng cài đặt (config.json / state.json)
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Nằm trong cùng thư viện textvn_ffi nhưng KHÔNG thuộc ABI engine P0-2 (textvn_ffi.h):
 *  - có I/O file (đọc/ghi nguyên tử, tạo ~/.config/TextVN 0700, file 0600);
 *  - chuỗi trả về được cấp phát → giải phóng bằng ime_settings_string_free();
 *  - chỉ gọi từ tiến trình UI hoặc khi đổi trạng thái (không gọi mỗi phím).
 *
 * Logic là textvn_config::SettingsDoc — dùng chung với tray Windows: vá từng khoá trên
 * JSON đã đọc (giữ macros/emoji/english_words và khoá lạ), mọi giá trị được kiểm theo
 * schema trước khi nhận. Bản chép tay; test `header_matches_exports` trong
 * ffi/src/settings.rs đối chiếu danh sách hàm.
 */
#ifndef TEXTVN_SETTINGS_H
#define TEXTVN_SETTINGS_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define IME_SETTINGS_CONFIG 0   /* config.json: khoá thiếu → mặc định schema */
#define IME_SETTINGS_STATE  1   /* state.json: {"global_enabled": bool, "apps": {...}} */

typedef struct ime_settings ime_settings;

/* Đọc file (đường dẫn UTF-8). File chưa có/hỏng → tài liệu rỗng; NULL chỉ khi tham số sai. */
ime_settings *ime_settings_load(const char *path, int32_t kind);
void          ime_settings_free(ime_settings *s);
/* 1 nếu file gốc tồn tại nhưng hỏng — lần lưu đầu sẽ đổi tên nó thành <file>.bak. */
int32_t       ime_settings_was_corrupt(const ime_settings *s);

/* 1/0 = giá trị; -1 = khoá không có / sai kiểu. */
int32_t       ime_settings_get_bool(const ime_settings *s, const char *key);
/* IME_OK; IME_ERR_INVALID_ARG = khoá không thuộc schema; IME_ERR_CONFIG = giá trị sai. */
int32_t       ime_settings_set_bool(ime_settings *s, const char *key, int32_t value);
/* Chuỗi mới cấp phát hoặc NULL. */
char         *ime_settings_get_str(const ime_settings *s, const char *key);
int32_t       ime_settings_set_str(ime_settings *s, const char *key, const char *value);

/* Bảng gõ tắt dạng text: mỗi dòng "gõ tắt = nội dung"; "\n" = xuống dòng, "\\" = "\". */
char         *ime_settings_macros_text(const ime_settings *s);
/* IME_OK, hoặc mã lỗi > 0 kèm *bad_line (đếm từ 1); lỗi → bảng cũ giữ nguyên. */
int32_t       ime_settings_set_macros_text(ime_settings *s, const char *text,
                                           uint32_t *bad_line);
/* Thông báo tiếng Việt cho mã lỗi ở trên (không cần free). */
const char   *ime_settings_macro_error_message(int32_t code);

/* Nút "Mặc định": mọi tuỳ chọn về mặc định, giữ gõ tắt/emoji/từ tiếng Anh. */
void          ime_settings_reset_defaults(ime_settings *s);
/* Ghi nguyên tử. IME_OK / IME_ERR_INTERNAL (I/O) / IME_ERR_CONFIG. */
int32_t       ime_settings_save(ime_settings *s, const char *path);
void          ime_settings_string_free(char *p);

#ifdef __cplusplus
}
#endif
#endif /* TEXTVN_SETTINGS_H */
