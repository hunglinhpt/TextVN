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
        client.setMarkedText(
            text as Any,
            selectionRange: selectionRange,
            replacementRange: .notFound
        )
    }

    func unmark() {
        client.unmarkText()
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
        for _ in 0..<count {
            client.doCommand(by: selector)
        }
        return true
    }
}
