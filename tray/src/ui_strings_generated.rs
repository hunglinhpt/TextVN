// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/ui/strings.toml (digest FNV-1a 64 = 0x10f6a7ac1fe454c2) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).
// Đổi chữ: sửa data/ui/strings.toml rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.
//! Chữ UI dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.
#![allow(dead_code)]

pub const ABOUT_TAGLINE: &str = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)";

pub const ABOUT_DETAILS: [&str; 3] = [
    "Phát triển bởi: (C) 2026 Bui Hung Linh - LinhBH.CoM",
    "Giấy phép: GPL-3.0-or-later",
    "https://github.com/hunglinhpt/TextVN",
];

pub const URL_DISCUSSIONS: &str = "https://github.com/hunglinhpt/TextVN/discussions";

pub const URL_ISSUES: &str = "https://github.com/hunglinhpt/TextVN/issues";

pub const URL_MATH_GUIDE: &str =
    "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md";

pub const METHOD_LABELS: [&str; 4] = ["Telex", "VNI", "VIQR", "Telex đơn giản"];

pub const ASSIST_MODE_LABELS: [&str; 3] = ["Tắt", "Chỉ gợi ý", "Tự sửa lỗi an toàn"];

pub const ASSIST_MODE_HINTS: [&str; 3] = [
    "Không kiểm tra ngữ pháp và chính tả.",
    "Dừng gõ 1 giây: hiện lỗi và gợi ý, bạn tự chọn sửa.",
    "Tự sửa lỗi chắc chắn khi hết câu; Ctrl+Z hoàn tác.",
];
