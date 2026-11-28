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

    /// SF-2 · SF-G1/G2: corpus toggle dùng chung 3 OS (`corpus/toggle/toggle.cases`). IMK chỉ có
    /// `flagsChanged` ⇒ +X/-X đổi bộ cờ (phím giữ lặp không phát sự kiện), K/P là keyDown.
    /// Mã cũ macOS phải ra đúng số lần ghi (hoặc cột `mac=`), session đúng vết vàng.
    func testToggleCorpus() throws {
        let url = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // IMKLibTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // macos-imk
            .deletingLastPathComponent() // adapters
            .deletingLastPathComponent() // gốc repo
            .appendingPathComponent("corpus/toggle/toggle.cases")
        let text = try String(contentsOf: url, encoding: .utf8)
        var ncases = 0
        var failures: [String] = []
        for raw in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let line = raw.trimmingCharacters(in: .whitespaces)
            if line.isEmpty || line.hasPrefix("#") { continue }
            let cols = line.split(separator: "|").map { $0.trimmingCharacters(in: .whitespaces) }
            XCTAssertGreaterThanOrEqual(cols.count, 3, line)
            let name = cols[0]
            let expectSession = try XCTUnwrap(UInt32(cols[2]), line)
            var expectLegacy = expectSession
            for col in cols.dropFirst(3) where col.hasPrefix("mac=") {
                expectLegacy = try XCTUnwrap(UInt32(col.dropFirst(4)), line)
            }

            let shadow = try XCTUnwrap(SessionShadow())
            var p = Pair(shadow: shadow)
            var ctrl = false, shift = false, alt = false
            for ev in cols[1].split(separator: " ") {
                if ev.hasPrefix("w") {
                    p.stepMs = try XCTUnwrap(UInt64(ev.dropFirst()), line)
                    continue
                }
                switch ev {
                case "+C", "-C":
                    let v = ev == "+C"
                    if v != ctrl { ctrl = v; p.flags(ctrl, shift, alt) } else { p.ms += p.stepMs }
                case "+S", "-S":
                    let v = ev == "+S"
                    if v != shift { shift = v; p.flags(ctrl, shift, alt) } else { p.ms += p.stepMs }
                case "+A", "-A":
                    let v = ev == "+A"
                    if v != alt { alt = v; p.flags(ctrl, shift, alt) } else { p.ms += p.stepMs }
                case "K":
                    p.key()
                case "P":
                    // isToggleChord: đúng Ctrl+Shift (Option/Cmd ⇒ không phải chord).
                    p.key(space: true, chord: ctrl && shift && !alt)
                case "R":
                    p.reset()
                default:
                    XCTFail("sự kiện lạ \(ev) ở ca \(name)")
                }
            }
            if UInt32(p.legacyToggles) != expectLegacy || shadow.sessionToggles != expectSession {
                failures.append("\(name): macOS cũ \(p.legacyToggles) (mong \(expectLegacy)), "
                    + "session \(shadow.sessionToggles) (mong \(expectSession))")
            }
            ncases += 1
        }
        XCTAssertGreaterThanOrEqual(ncases, 16)
        XCTAssertEqual(failures, [])
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
