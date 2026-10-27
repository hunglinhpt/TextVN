// SPDX-License-Identifier: GPL-3.0-or-later
//! IMKTextTarget — bridge `IMKTextInput` client → `TextTarget` (Marked.swift).
//!
//! Mọi thao tác text của adapter đi qua đây để unit test mock được (MAC-030).
//! - insert/setMarked/unmark/markedRange/selectedRange = chuẩn IMKTextInput.
//! - deleteBackward: key binding chuẩn `NSResponder.deleteBackward:` qua
//!   `doCommand(by:)` — cơ chế (a) P2-1 §6.3, không cần quyền Accessibility.

import AppKit
import CoreBridge
import Foundation
import InputMethodKit

/// `doCommandBySelector:` thuộc `NSTextInputClient`, KHÔNG có trong protocol
/// `IMKTextInput` (MAC-032: Xcode 26.6 báo "no member 'doCommand'"). Gửi qua
/// protocol @objc riêng = objc_msgSend thuần — chạy cả với client in-process
/// (NSObject) lẫn proxy XPC (NSProxy, nơi `as NSObject` thất bại).
@objc protocol TextCommandTarget {
    @objc(doCommandBySelector:) func doCommand(by selector: Selector)
}

final class IMKTextTarget: TextTarget {
    /// client thật của IMK (`IMKInputController.handle(event:client:)`).
    private let client: IMKTextInput

    init(client: IMKTextInput) {
        self.client = client
    }

    func insert(_ text: String, replacementRange: NSRange) {
        client.insertText(
            text as Any,
            replacementRange: replacementRange
        )
    }

    func setMarked(_ text: String, selectionRange: NSRange) {
        let attrString = NSAttributedString(
            string: text,
            attributes: [
                .underlineStyle: NSUnderlineStyle.single.rawValue /* B10: [] = 0 (none) — mất gạch chân */
            ]
        )
        client.setMarkedText(
            attrString,
            selectionRange: selectionRange,
            replacementRange: .notFound
        )
    }

    /// `IMKTextInput` không có `unmarkText` — commit marked hiện có bằng cách
    /// chèn lại đúng text đó lên range marked (chỉ dùng API IMKTextInput).
    func unmark() {
        let range = client.markedRange()
        guard range.location != NSNotFound, range.length > 0,
              let text = client.attributedSubstring(from: range)?.string else { return }
        client.insertText(text as Any, replacementRange: range)
    }

    func markedRange() -> NSRange {
        client.markedRange()
    }

    func selectionRange() -> NSRange {
        client.selectedRange()
    }

    func deleteBackward(count: Int) -> Bool {
        guard count > 0 else { return true }
        // Cơ chế (a) — P2-1 §6.3: key binding chuẩn qua IMKTextInput.doCommand(by:),
        // không cần quyền Accessibility, tương thích cả in-process NSView lẫn out-of-process XPC session.
        let selector = #selector(NSResponder.deleteBackward(_:))
        // F8: doCommand không có giá trị trả về — client không nhận
        // `doCommandBySelector:` phải báo `false` để ApplyReplace fail-open,
        // không xóa-thiếu âm thầm.
        let object = client as AnyObject
        guard object.responds(to: #selector(TextCommandTarget.doCommand(by:))) else {
            Diagnostics.log("deleteBackward: client không nhận doCommandBySelector:")
            return false
        }
        let target = unsafeBitCast(object, to: TextCommandTarget.self)
        for _ in 0..<count {
            target.doCommand(by: selector)
        }
        return true
    }

    /// Text ngay trước caret — **precondition D12** (R-2/R-7): trước khi xoá để
    /// tự sửa, adapter phải xác minh vùng sắp xoá **đúng** là đuôi câu mà
    /// engine biết; không xác minh được ⇒ **không sửa** (fail-safe).
    ///
    /// Chỉ đọc tối đa 1024 UTF-16 code unit gần nhất trước caret (câu tối đa
    /// 256 ký tự) để tránh cấp phát IPC lớn trên tài liệu dài, và chỉ giữ
    /// trong RAM tạm của tiến trình, không log/không gửi đi (D8/S2).
    func textBeforeCaret() -> String? {
        let selection = client.selectedRange()
        guard selection.location != NSNotFound, selection.location > 0 else { return nil }
        let maxRead = 1024
        if selection.location > maxRead {
            let tailRange = NSRange(location: selection.location - maxRead, length: maxRead)
            if let text = client.attributedSubstring(from: tailRange)?.string {
                return text
            }
        }
        let range = NSRange(location: 0, length: selection.location)
        guard let text = client.attributedSubstring(from: range)?.string else { return nil }
        return text
    }

    /// OP-007 · R2-139: text của vùng chọn hiện tại — dùng bởi chord
    /// `Ctrl+Shift+F9` (chuyển mã vùng bôi đen). Trần 64 000 ký tự khớp
    /// `selection_check::MAX_TRANSCODE_CHARS` của Windows; selection rỗng/không
    /// có ⇒ `nil` — phím được để tới app một cách rõ ràng (fail-open). Text chỉ
    /// giữ trong RAM tạm của tiến trình, không log/không gửi đi (D8/S2).
    func selectedText() -> String? {
        text(in: client.selectedRange())
    }

    /// A4 (vòng 34): text theo range CHO TRƯỚC — caller đọc range 1 lần rồi
    /// lấy text theo đúng range đó, chống TOCTOU giữa 2 lời gọi IMK rời.
    func text(in range: NSRange) -> String? {
        guard range.location != NSNotFound, range.length > 0, range.length <= 64_000 else { return nil }
        guard let text = client.attributedSubstring(from: range)?.string, !text.isEmpty else { return nil }
        return text
    }
}
