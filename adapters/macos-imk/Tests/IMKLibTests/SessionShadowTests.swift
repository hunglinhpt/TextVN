// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-2: shadow `MacModifierTap` (logic Ctrl+Shift cũ của IMK) ↔ `textvn-session`
//! qua C ABI `ime_session_*`. Lái cả hai giống hệt `TextVNInputController.handle`.

import XCTest

@testable import CoreBridge
@testable import IMKLib

final class SessionShadowTests: XCTestCase {
    private struct Pair {
        var tap = MacModifierTap()
        let shadow: SessionShadow
        var ms: UInt64 = 1_000
        var stepMs: UInt64 = 300 // > debounce 250 ms
        var legacyToggles = 0

        mutating func flags(_ ctrl: Bool, _ shift: Bool, _ alt: Bool = false) {
            let out = tap.flagsChanged(isCtrl: ctrl, isShift: shift, hasAltOrCmd: alt,
                                       now: TimeInterval(ms) / 1000)
            if out == .toggle { legacyToggles += 1 }
            shadow.flagsChanged(ctrl: ctrl, shift: shift, altOrCmd: alt,
                                legacyToggled: out == .toggle, nowMs: ms)
            ms += stepMs
        }

        /// keyDown phím thường; `chord` = Ctrl+Shift+Space (mã cũ bật/tắt ở keyDown).
        mutating func key(space: Bool = false, chord: Bool = false) {
            tap.otherKey()
            if chord { legacyToggles += 1 }
            shadow.keyDown(isSpace: space, legacyToggled: chord, nowMs: ms)
            ms += stepMs
        }

        mutating func reset() {
            tap.reset()
            shadow.reset(nowMs: ms)
            ms += stepMs
        }
    }

    func testShadowMatchesLegacyToggle() throws {
        let shadow = try XCTUnwrap(SessionShadow(), "ime_session_new phải tạo được session")
        var p = Pair(shadow: shadow)

        // 1. Ctrl → Shift, nhả Shift trước.
        p.flags(true, false); p.flags(true, true); p.flags(true, false); p.flags(false, false)
        // 2. Shift → Ctrl, nhả Ctrl trước.
        p.flags(false, true); p.flags(true, true); p.flags(false, true); p.flags(false, false)
        // 3. Phím thứ ba huỷ.
        p.flags(true, false); p.flags(true, true); p.key(); p.flags(false, true); p.flags(false, false)
        // 4. Option/Cmd chen giữa huỷ.
        p.flags(true, false); p.flags(true, true); p.flags(true, true, true); p.flags(true, true)
        p.flags(false, true); p.flags(false, false)
        // 5. Ctrl+Shift+Space bật/tắt ở keyDown; nhả sau đó không bật lại.
        p.flags(true, false); p.flags(true, true); p.key(space: true, chord: true)
        p.flags(true, false); p.flags(false, false)
        // 6. Mất focus giữa chừng.
        p.flags(true, false); p.flags(true, true); p.reset(); p.flags(false, true); p.flags(false, false)
        // 7. Ctrl tap riêng.
        p.flags(true, false); p.flags(false, false)
        // 8. Debounce 250 ms: cả hai cùng chặn lần thứ hai.
        p.stepMs = 20
        p.flags(true, false); p.flags(true, true); p.flags(true, false); p.flags(false, false)
        p.flags(true, false); p.flags(true, true); p.flags(true, false); p.flags(false, false)

        XCTAssertEqual(p.legacyToggles, 4)
        XCTAssertEqual(shadow.toggles, 4)
        XCTAssertEqual(shadow.mismatches, 0)
        XCTAssertTrue(shadow.isActive)
    }

    func testShadowCountsMismatch() throws {
        // Shadow phải BẮT được lệch: mã cũ "báo" bật/tắt ở phím thường mà session không.
        let shadow = try XCTUnwrap(SessionShadow())
        shadow.keyDown(isSpace: false, legacyToggled: true, nowMs: 1_000)
        XCTAssertEqual(shadow.mismatches, 1)
    }

    func testShadowOffByDefault() {
        if ProcessInfo.processInfo.environment["TEXTVN_SESSION_SHADOW"] != "1" {
            XCTAssertNil(SessionShadow.fromEnvironment())
        }
    }
}
