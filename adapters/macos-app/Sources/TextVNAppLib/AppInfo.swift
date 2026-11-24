// SPDX-License-Identifier: GPL-3.0-or-later
//! AppInfo — nguồn version hiển thị **duy nhất** của TextVN.app (R2 finding 2:
//! trước đây "0.1.0" hardcode rải rác ở About/Settings/Health → lệch sau bump).

import Foundation

enum AppInfo {
    /// CFBundleShortVersionString của bundle đang chạy; fallback hằng khi
    /// chạy trong swift test (Bundle.main không có Info.plist app).
    static let displayVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.87"

    /// Popup "Thông tin" — CÙNG nội dung trên Windows/Linux/macOS (test Rust
    /// `about_text_matches_every_platform` đối chiếu từng chữ). Hai popup macOS
    /// (Settings + menu bar) cùng đọc ở đây.
    static let aboutTagline = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)"
    static let aboutDetails =
        "Phát triển bởi: LinhBH.CoM\nGiấy phép: GPL-3.0-or-later\nhttps://github.com/hunglinhpt/TextVN"
    static var aboutTitle: String { "TextVN \(displayVersion)" }
    static var aboutMessage: String { "\(aboutTagline)\n\n\(aboutDetails)" }

    /// Trang Thảo luận / Báo lỗi — CÙNG URL trên Windows/Linux/macOS (nút trong cửa sổ
    /// "Hướng dẫn"; test Rust `help_window_links_match_every_platform` đối chiếu).
    static let discussionsUrl = "https://github.com/hunglinhpt/TextVN/discussions"

    /// Nút **Báo lỗi** mở đúng issue tracker (khác Thảo luận): cùng URL trên cả 3 OS —
    /// khớp `<url type="bugtracker">` của AppStream metainfo.
    static let reportIssueUrl = "https://github.com/hunglinhpt/TextVN/issues"

    /// P11 — nút **Math & Science** trong cửa sổ "Hướng dẫn" mở đúng tài liệu guide
    /// (cùng URL trên cả 3 OS — test Rust `help_window_links_match_every_platform`).
    static let mathGuideUrl =
        "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md"

    static let helpTitle = "Hướng dẫn TextVN"
    static let helpText =
        "Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (không kèm phím khác), hoặc Ctrl + Shift + Space, "
        + "hoặc dùng menu TextVN ở thanh menu.\n\n"
        + "Telex: aa=â  aw=ă  ee=ê  oo=ô  ow=ơ  uw=ư  dd=đ;  s f r x j = sắc huyền hỏi ngã nặng;  z = xoá dấu.\n"
        + "VNI: 1-5 = sắc huyền hỏi ngã nặng;  6 = mũ (â ê ô);  7 = móc (ơ ư);  8 = trăng (ă);  9 = đ;  0 = xoá dấu.\n"
        + "Quick Telex: cc=ch  gg=gi  kk=kh  nn=ng  qq=qu  pp=ph  tt=th.\n\n"
        + "Gõ tắt: bấm [Gõ tắt...], mỗi dòng ghi  gõ tắt = nội dung; khi gõ, nhập chữ tắt rồi nhấn Tab (hoặc Space) để bung ra.\n\n"
        + "Math & Science (mặc định TẮT): bật/tắt ở Tùy chọn gõ trong Cài đặt, hoặc menu bar. Khi TẮT: gõ Việt/Anh thường 100% không đổi. Khi BẬT: phím leader ` gõ ký hiệu/công thức (`a = α, `/` = phân số), lệnh gõ tắt `\\can` `\\ps` `\\tp`; trong `$...$` hoặc `\\text{...}` tiếng Việt tự treo. Hotkeys: Ctrl+Alt+M (đổi chế độ xuất), Ctrl+Shift+M (sticky math), Ctrl+Alt+V (Smart Paste), Ctrl+Alt+Space (palette). Chi tiết: nút [Math & Science].\n\n"
        + "Nếu chưa gõ được tiếng Việt: thêm TextVN trong Cài đặt hệ thống > Bàn phím > Nguồn nhập, rồi đăng xuất và đăng nhập lại."
}
