// SPDX-License-Identifier: GPL-3.0-or-later
//! AppInfo — nguồn version hiển thị **duy nhất** của TextVN.app (R2 finding 2:
//! trước đây "0.1.0" hardcode rải rác ở About/Settings/Health → lệch sau bump).

import Foundation

enum AppInfo {
    /// CFBundleShortVersionString của bundle đang chạy; fallback hằng khi
    /// chạy trong swift test (Bundle.main không có Info.plist app).
    static let displayVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.96"

    /// Popup "Thông tin" — CÙNG nội dung trên Windows/Linux/macOS (SF-9: sinh từ
    /// `data/ui/strings.toml` → `UiStrings`). Hai popup macOS (Settings + menu bar) cùng đọc ở đây.
    static let aboutTagline = UiStrings.aboutTagline
    static let aboutDetails = UiStrings.aboutDetails.joined(separator: "\n")
    static var aboutTitle: String { "TextVN \(displayVersion)" }
    static var aboutMessage: String { "\(aboutTagline)\n\n\(aboutDetails)" }

    /// Trang Thảo luận / Báo lỗi / Math & Science — CÙNG URL trên Windows/Linux/macOS (sinh từ `data/ui/strings.toml`).
    static let discussionsUrl = UiStrings.urlDiscussions

    /// Nút **Báo lỗi** mở đúng issue tracker (khác Thảo luận): cùng URL trên cả 3 OS —
    /// khớp `<url type="bugtracker">` của AppStream metainfo.
    static let reportIssueUrl = UiStrings.urlIssues

    /// P11 — nút **Math & Science** trong cửa sổ "Hướng dẫn" mở đúng tài liệu guide.
    static let mathGuideUrl = UiStrings.urlMathGuide

    static let helpTitle = "Hướng dẫn TextVN"
    static let helpText =
        "Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (không kèm phím khác), hoặc Ctrl + Shift + Space, "
        + "hoặc dùng menu TextVN ở thanh menu.\n\n"
        + "Telex: aa=â  aw=ă  ee=ê  oo=ô  ow=ơ  uw=ư  dd=đ;  s f r x j = sắc huyền hỏi ngã nặng;  z = xoá dấu.\n"
        + "VNI: 1-5 = sắc huyền hỏi ngã nặng;  6 = mũ (â ê ô);  7 = móc (ơ ư);  8 = trăng (ă);  9 = đ;  0 = xoá dấu.\n"
        + "Quick Telex: cc=ch  gg=gi  kk=kh  nn=ng  qq=qu  pp=ph  tt=th.\n\n"
        + "Gõ tắt: bấm [Gõ tắt...], mỗi dòng ghi  gõ tắt = nội dung; khi gõ, nhập chữ tắt rồi nhấn Tab (hoặc Space) để bung ra.\n\n"
        + "Math & Science (mặc định TẮT): bật/tắt ở Tùy chọn gõ trong Cài đặt, hoặc menu bar. Khi TẮT: gõ Việt/Anh thường 100% không đổi. Khi BẬT: phím leader ` gõ ký hiệu/công thức (`a = α, `/` = phân số), lệnh gõ tắt `\\can` `\\ps` `\\tp`; trong `$...$` hoặc `\\text{...}` tiếng Việt tự treo. Hotkeys: Ctrl+Alt+M (đổi chế độ xuất), Ctrl+Shift+M (sticky math), Ctrl+Alt+V (Smart Paste), Ctrl+Alt+Space (palette). Chi tiết: nút [Math & Science].\n\n"
        + "Nếu chưa gõ được tiếng Việt: thêm TextVN trong Cài đặt hệ thống > Bàn phím > Nguồn nhập, rồi đăng xuất và đăng nhập lại.\n\n"
        + "Gặp lỗi (không gõ được dấu, lặp chữ, xung đột bộ gõ cũ...): bấm nút [Chẩn đoán & xuất log] ngay bên dưới — TextVN sẽ tự kiểm tra máy và lưu sẵn tệp log an toàn (không chứa chữ bạn gõ) để gửi báo lỗi."
}
