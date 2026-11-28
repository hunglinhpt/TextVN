// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/ui/strings.toml (digest FNV-1a 64 = 0x10f6a7ac1fe454c2) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).
// Đổi chữ: sửa data/ui/strings.toml rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.
/* Chữ UI dùng chung Windows/Linux/macOS (G22). Macro chuỗi để ghép lúc biên dịch
 * (vd `TV_UI_ABOUT_TAGLINE "\n\n" TV_UI_ABOUT_DETAILS_0`) — không thêm tệp .c vào build. */
#ifndef TEXTVN_UI_STRINGS_GENERATED_H
#define TEXTVN_UI_STRINGS_GENERATED_H

#define TV_UI_ABOUT_TAGLINE "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)"

#define TV_UI_ABOUT_DETAILS_COUNT 3
#define TV_UI_ABOUT_DETAILS_0 "Phát triển bởi: (C) 2026 Bui Hung Linh - LinhBH.CoM"
#define TV_UI_ABOUT_DETAILS_1 "Giấy phép: GPL-3.0-or-later"
#define TV_UI_ABOUT_DETAILS_2 "https://github.com/hunglinhpt/TextVN"

#define TV_UI_URL_DISCUSSIONS "https://github.com/hunglinhpt/TextVN/discussions"

#define TV_UI_URL_ISSUES "https://github.com/hunglinhpt/TextVN/issues"

#define TV_UI_URL_MATH_GUIDE "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md"

#define TV_UI_METHOD_LABELS_COUNT 4
#define TV_UI_METHOD_LABELS_0 "Telex"
#define TV_UI_METHOD_LABELS_1 "VNI"
#define TV_UI_METHOD_LABELS_2 "VIQR"
#define TV_UI_METHOD_LABELS_3 "Telex đơn giản"

#define TV_UI_ASSIST_MODE_LABELS_COUNT 3
#define TV_UI_ASSIST_MODE_LABELS_0 "Tắt"
#define TV_UI_ASSIST_MODE_LABELS_1 "Chỉ gợi ý"
#define TV_UI_ASSIST_MODE_LABELS_2 "Tự sửa lỗi an toàn"

#define TV_UI_ASSIST_MODE_HINTS_COUNT 3
#define TV_UI_ASSIST_MODE_HINTS_0 "Không kiểm tra ngữ pháp và chính tả."
#define TV_UI_ASSIST_MODE_HINTS_1 "Dừng gõ 1 giây: hiện lỗi và gợi ý, bạn tự chọn sửa."
#define TV_UI_ASSIST_MODE_HINTS_2 "Tự sửa lỗi chắc chắn khi hết câu; Ctrl+Z hoàn tác."

#endif /* TEXTVN_UI_STRINGS_GENERATED_H */
