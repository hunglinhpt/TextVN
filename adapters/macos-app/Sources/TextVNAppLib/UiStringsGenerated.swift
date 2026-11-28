// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/ui/strings.toml (digest FNV-1a 64 = 0x10f6a7ac1fe454c2) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).
// Đổi chữ: sửa data/ui/strings.toml rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.

/// Chữ UI dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.
enum UiStrings {
    static let aboutTagline = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)"

    static let aboutDetails: [String] = [
        "Phát triển bởi: (C) 2026 Bui Hung Linh - LinhBH.CoM",
        "Giấy phép: GPL-3.0-or-later",
        "https://github.com/hunglinhpt/TextVN",
    ]

    static let urlDiscussions = "https://github.com/hunglinhpt/TextVN/discussions"

    static let urlIssues = "https://github.com/hunglinhpt/TextVN/issues"

    static let urlMathGuide = "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md"

    static let methodLabels: [String] = [
        "Telex",
        "VNI",
        "VIQR",
        "Telex đơn giản",
    ]

    static let assistModeLabels: [String] = [
        "Tắt",
        "Chỉ gợi ý",
        "Tự sửa lỗi an toàn",
    ]

    static let assistModeHints: [String] = [
        "Không kiểm tra ngữ pháp và chính tả.",
        "Dừng gõ 1 giây: hiện lỗi và gợi ý, bạn tự chọn sửa.",
        "Tự sửa lỗi chắc chắn khi hết câu; Ctrl+Z hoàn tác.",
    ]
}
