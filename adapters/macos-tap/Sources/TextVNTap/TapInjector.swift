// SPDX-License-Identifier: GPL-3.0-or-later
//! TapInjector — inject text/xóa qua CGEvent (P2-2 §5, mirror P1-2 §5).
//!
//! Mọi event post đều gắn `CGEventSourceSetUserData`-style marker
//! (`.eventSourceUserData = TapMarker.userData`) + engine `is_injected=1`
//! khi handler xử lý lại — chống loop 2 lớp (P0-3 §6).
//!
//! Batch ≤ 64 event/lần post; modifier luôn clear sau mỗi event (P1-2 §5.4:
//! quên restore modifier = "bug kinh điển" — phím kế của user bị lệch).

import Carbon.HIToolbox
import CoreGraphics
import Foundation

public enum TapInjector {
    /// Số event tối đa mỗi lần post (P2-2 §5.4).
    public static let maxBatch = 64
    private static let kVKBackspace: UInt16 = 51 // kVK_Delete

    /// Inject transform: `deleteCount` × Backspace (cơ chế a/b đã fail ở đây —
    /// tap không có client nên phải post key thật) + unicode string chèn.
    /// Trả `false` nếu không post được event nào (caller forward key gốc —
    /// "không mất text", mirror hook `inject_engine_result`).
    @discardableResult
    public static func inject(deleteCount: Int, insert: String) -> Bool {
        guard let source = makeMarkedSource() else { return false }
        var posted = 0

        // 1) Backspace ×deleteCount (kVK 51) — modifier sạch.
        for _ in 0..<min(deleteCount, maxBatch) {
            guard postKey(kVKBackspace, source: source) else { return posted > 0 }
            posted += 1
        }

        // 2) Chèn text — unicode string event (chữ có dấu/surrogate luôn đúng).
        let scalars = Array(insert.unicodeScalars)
        var units: [UInt16] = []
        for scalar in scalars {
            units.append(contentsOf: String(scalar).utf16)
        }
        var offset = 0
        while offset < units.count, posted < maxBatch {
            let chunk = Array(units[offset..<min(offset + 20, units.count)])
            let event = CGEvent(
                keyboardEventSource: source,
                virtualKey: 0,
                keyDown: true
            )
            event?.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: chunk)
            event?.post(tap: .cghidEventTap)
            guard event != nil else { break }
            posted += 1
            offset += chunk.count
        }
        return posted > 0
    }

    /// `SelectionReplace` mở selection bằng Shift+Left ×n rồi chèn (P2-2 §5.2) —
    /// KHÔNG backspace (app không được gợi ý lại — bug B1).
    @discardableResult
    public static func selectAndReplace(leftCount: Int, insert: String) -> Bool {
        guard let source = makeMarkedSource() else { return false }
        var posted = 0
        for _ in 0..<min(leftCount, maxBatch / 2) {
            guard postKey(kVKLeftArrow, source: source, shift: true) else { return posted > 0 }
            posted += 1
        }
        let event = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true)
        let units = Array(insert.utf16)
        event?.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
        event?.post(tap: .cghidEventTap)
        return event != nil
    }

    private static let kVKLeftArrow: UInt16 = 123

    // ---------------------------------------------------------------- internals

    /// Source đánh dấu mọi event post ra — callback §3.1 thấy marker là cho qua.
    static func makeMarkedSource() -> CGEventSource? {
        let source = CGEventSource(stateID: .combinedSessionState)
        source?.userData = TapMarker.userData
        return source
    }

    private static func postKey(_ keyCode: UInt16, source: CGEventSource, shift: Bool = false) -> Bool {
        let down = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: true)
        let up = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: false)
        if shift {
            down?.flags = .maskShift
            up?.flags = .maskShift
        }
        down?.post(tap: .cghidEventTap)
        up?.post(tap: .cghidEventTap)
        return down != nil && up != nil
    }
}
