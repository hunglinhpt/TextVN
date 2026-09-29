// SPDX-License-Identifier: GPL-3.0-or-later
//! Marked — quản lý marked text (preedit) của IMK (P2-1 §7).
//!
//! Nguyên tắc (B11/B13):
//! - Marked **không sống lâu**: Space/Enter/động từ → commit ngay.
//! - Giới hạn marked ≤ 8 grapheme; vượt → commit phần đầu (bật config word dài).
//! - Click/focus sang ô khác → **commit-before-hide** (B13): marked hiện tại
//!   trở thành text vĩnh viễn, không mất chữ.
//! - App từ chối marked (terminal…) → self-heal: `ime_reset()` + về Idle,
//!   **không đoán text của app** (P1-1 §4 cùng triết lý).

import CoreBridge
import Foundation

/// Đích áp text — trừu tượng hoá `IMKTextInput` để unit test được toàn bộ
/// vòng đời marked/replace mà không cần app thật.
public protocol TextTarget: AnyObject {
    /// Chèn text tại caret (hoặc thay selection nếu app tự áp dụng — NSTextInput chuẩn).
    func insert(_ text: String, replacementRange: NSRange)
    /// Hiển thị marked text (gạch chân IMK).
    func setMarked(_ text: String, selectionRange: NSRange)
    /// Nhả marked giữ nguyên text hiện có (một số app cần).
    func unmark()
    /// Range marked hiện tại của app; `.notFound` nếu không có.
    func markedRange() -> NSRange
    /// Range selection hiện tại; `.notFound` nếu không truy vấn được.
    func selectionRange() -> NSRange
    /// Xóa `count` ký tự trước caret bằng key binding chuẩn; `false` nếu app không hỗ trợ.
    func deleteBackward(count: Int) -> Bool
}

/// Trạng thái marked của controller — 1 instance theo controller (main thread).
public final class MarkedState {
    /// Giới hạn B11 (P2-1 §7): marked ≤ 8 grapheme.
    public static let maxGraphemes = 8

    public private(set) var text: String = ""

    public var isEmpty: Bool { text.isEmpty }
    public var graphemeCount: Int { text.count }
    /// Số **code point** (scalar) — đơn vị engine dùng cho `delete_count`
    /// (Rust `char` = Unicode scalar). Mọi phép trừ với delete_count dùng cái này.
    public var scalarCount: Int { text.unicodeScalars.count }
    /// Số UTF-16 code unit — đơn vị của NSRange (P2-1 §6.4).
    public var utf16Count: Int { (text as NSString).length }

    public init() {}

    public func clear() {
        text = ""
    }

    /// Ghi text marked (duy nhất qua đây để giữ invariant).
    public func set(_ text: String) {
        self.text = text
    }

    /// `true` nếu text mới vượt giới hạn B11 → caller phải commit-early.
    public static func exceedsLimit(_ candidate: String) -> Bool {
        candidate.count > maxGraphemes
    }
}

extension String {
    public var utf16Count: Int { utf16.count }
}

extension NSRange {
    public static let notFound = NSRange(location: NSNotFound, length: 0)
}
