// SPDX-License-Identifier: GPL-3.0-or-later
//! AppInfo — nguồn version hiển thị **duy nhất** của TextVN.app (R2 finding 2:
//! trước đây "0.1.0" hardcode rải rác ở About/Settings/Health → lệch sau bump).

import Foundation

enum AppInfo {
    /// CFBundleShortVersionString của bundle đang chạy; fallback hằng khi
    /// chạy trong swift test (Bundle.main không có Info.plist app).
    static let displayVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.39"

    /// Popup "Thông tin" — CÙNG nội dung trên Windows/Linux/macOS (test Rust
    /// `about_text_matches_every_platform` đối chiếu từng chữ). Hai popup macOS
    /// (Settings + menu bar) cùng đọc ở đây.
    static let aboutTagline = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)"
    static let aboutDetails =
        "Phát triển bởi: LinhBH.CoM\nGiấy phép: GPL-3.0-or-later\nhttps://github.com/hunglinhpt/TextVN"
    static var aboutTitle: String { "TextVN \(displayVersion)" }
    static var aboutMessage: String { "\(aboutTagline)\n\n\(aboutDetails)" }
}
