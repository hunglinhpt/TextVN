// SPDX-License-Identifier: GPL-3.0-or-later
//! IMKLibTests — unit test logic adapter macOS (MAC-030, P2-3 §6).
//!
//! Chạy trên macOS runner: `swift test --package-path adapters/macos-imk`
//! (cần `build-rust.sh --lib-only` trước — link libtextvn_ffi.a).

import AppKit
import XCTest

@testable import CoreBridge
@testable import IMKLib

// ---------------------------------------------------------------- mock client

final class MockTextTarget: TextTarget {
    var inserted: [(text: String, replacement: NSRange)] = []
    var markedText: String = ""
    var markedSet: Int = 0
    var unmarked: Int = 0
    var selection = NSRange(location: 0, length: 0)
    var markedRangeValue = NSRange(location: NSNotFound, length: 0)
    var allowDeleteBackward = true
    private(set) var deletedTotal = 0

    func insert(_ text: String, replacementRange: NSRange) {
        inserted.append((text, replacementRange))
        // mô phỏng NSTextInput: insertText thay marked range nếu có.
        if markedRangeValue.location != NSNotFound {
            markedRangeValue = NSRange(location: NSNotFound, length: 0)
        }
    }

    func setMarked(_ text: String, selectionRange: NSRange) {
        markedText = text
        markedSet += 1
        markedRangeValue = text.isEmpty
            ? NSRange(location: NSNotFound, length: 0)
            : NSRange(location: 0, length: (text as NSString).length)
    }

    func unmark() {
        unmarked += 1
        markedRangeValue = NSRange(location: NSNotFound, length: 0)
    }

    func markedRange() -> NSRange { markedRangeValue }
    func selectionRange() -> NSRange { selection }

    func deleteBackward(count: Int) -> Bool {
        guard allowDeleteBackward else { return false }
        deletedTotal += count
        return true
    }
}

// ---------------------------------------------------------------- FieldRules (MAC-030)

final class FieldRulesTests: XCTestCase {
    func testR1_secureSubrole() {
        let (role, secure) = FieldRules.map(AXSnapshot(role: "AXTextField", subrole: "AXSecureTextField"))
        XCTAssertEqual(role, FieldRole.secure)
        XCTAssertTrue(secure)
    }

    func testR1_secureInputMode() {
        let (role, secure) = FieldRules.map(AXSnapshot(role: "AXTextArea", secureInputMode: true))
        XCTAssertEqual(role, FieldRole.secure)
        XCTAssertTrue(secure)
    }

    func testR2_urlPlaceholder() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextField", placeholder: "Address or URL"))
        XCTAssertEqual(role, FieldRole.addressBar)
    }

    func testR2_searchTitle() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextField", title: "Search"))
        XCTAssertEqual(role, FieldRole.search)
    }

    func testR3_searchSubrole() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextField", subrole: "AXSearchField"))
        XCTAssertEqual(role, FieldRole.search)
    }

    func testR4_combo() {
        let (combo, _) = FieldRules.map(AXSnapshot(role: "AXComboBox"))
        XCTAssertEqual(combo, FieldRole.combo)
        let (popup, _) = FieldRules.map(AXSnapshot(role: "AXPopUpButton"))
        XCTAssertEqual(popup, FieldRole.combo)
    }

    func testR5_gridCandidate() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextField", identifier: "grid-cell-1"))
        XCTAssertEqual(role, FieldRole.candidate)
    }

    func testR6_terminalIdentifier() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextArea", identifier: "terminal-view"))
        XCTAssertEqual(role, FieldRole.terminal)
    }

    func testR7_textArea() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextArea"))
        XCTAssertEqual(role, FieldRole.textarea)
    }

    func testR8_webArea() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXWebArea"))
        XCTAssertEqual(role, FieldRole.web)
    }

    func testR9_textField() {
        let (role, _) = FieldRules.map(AXSnapshot(role: "AXTextField"))
        XCTAssertEqual(role, FieldRole.editbox)
    }

    func testR10_unknown() {
        let (role, secure) = FieldRules.map(AXSnapshot(role: nil))
        XCTAssertEqual(role, FieldRole.unknown)
        XCTAssertFalse(secure)
    }

    /// Đủ ≥30 ca mock AX (MAC-030): ma trận role × subrole × hint.
    func testMatrix_atLeastThirtyCases() {
        let roles = ["AXTextField", "AXTextArea", "AXWebArea", "AXComboBox", "AXButton", nil]
        let subroles = ["AXSecureTextField", "AXSearchField", nil]
        let hints = ["Address and search", "search the web", "url bar", "Tìm kiếm", "Nhập tại đây", nil]
        var cases = 0
        for r in roles {
            for s in subroles {
                for h in hints {
                    let (role, _) = FieldRules.map(
                        AXSnapshot(role: r, subrole: s, placeholder: h)
                    )
                    // Không crash + role luôn trong miền hợp lệ.
                    XCTAssertTrue(role <= FieldRole.secure)
                    cases += 1
                }
            }
        }
        XCTAssertGreaterThanOrEqual(cases, 30)
    }

    func testNormalizeAppID() {
        XCTAssertEqual(
            FieldDetect.normalizeAppID(bundleID: "Com.Google.Chrome", executableName: nil),
            "com.google.chrome"
        )
        XCTAssertEqual(
            FieldDetect.normalizeAppID(bundleID: nil, executableName: "/usr/bin/login"),
            "login"
        )
        XCTAssertEqual(FieldDetect.normalizeAppID(bundleID: nil, executableName: nil), "unknown")
    }

    func testFieldCacheTTL() {
        let detect = FieldDetect()
        let ctx = FieldContext(appID: "com.apple.safari", role: FieldRole.addressBar, secure: false)
        detect.store(ctx, for: 42)
        XCTAssertEqual(detect.cached(for: 42), ctx)
        // Quá TTL → nil.
        detect.store(ctx, for: 43, at: Date(timeIntervalSinceNow: -FieldDetect.ttl - 1))
        XCTAssertNil(detect.cached(for: 43))
        detect.invalidate(pid: 42)
        XCTAssertNil(detect.cached(for: 42))
    }
}

// ---------------------------------------------------------------- KeyMap parity

final class KeyMapMacTests: XCTestCase {
    /// Bảng sinh từ keymap_mac.toml phải khớp một số mốc Action nước đôi (MAC-013).
    func testCanonicalKeyAnchors() {
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x00), 0x41) // A
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x24), 0x0D) // Return
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x31), 0x20) // Space
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x33), 0x08) // Delete (backspace)
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x75), 0x2E) // ForwardDelete
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x7B), 0x25) // Left
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x38), 0x10) // Shift
        XCTAssertEqual(KeyMapMacGenerated.canonicalVK(0x39), 0x14) // CapsLock
        XCTAssertNil(KeyMapMacGenerated.canonicalVK(0x3F)) // Function — không map
        XCTAssertEqual(KeyMapMacGenerated.keyCount, 92)
    }

    /// UTF-16 surrogate — P2-1 §6.4: đếm length cho NSRange bằng utf16, không grapheme.
    func testUtf16CountForNSRange() {
        // "😀" = 1 grapheme, 2 UTF-16 units.
        let emoji = "😀"
        XCTAssertEqual(emoji.count, 1)
        XCTAssertEqual((emoji as NSString).length, 2)
        // Preedit chứa emoji: selectionRange phải dùng utf16 count.
        let preedit = "đư😀"
        XCTAssertEqual((preedit as NSString).length, 4)
        XCTAssertEqual(preedit.count, 3)
    }

    /// UTF-32 decode qua bridge: đúng scalar, đúng surrogate pair.
    func testUtf32Roundtrip() {
        let text = "được😀"
        let points = ImeEngine.utf32(text)
        XCTAssertEqual(points.count, 6) // đ,ư,ơ,c + 2 units của 😀
        let decoded = ImeEngine.string(fromUTF32: points, len: points.count)
        XCTAssertEqual(decoded, text)
    }
}

// ---------------------------------------------------------------- ApplyReplace

final class ApplyReplaceTests: XCTestCase {
    private func outcome(
        _ action: KeyOutcome.Action, flags: UInt32 = FFI.flagConsumed
    ) -> KeyOutcome {
        KeyOutcome(action: action, flags: flags)
    }

    /// Preedit: REPLACE → setMarkedText với preedit đầy đủ (P2-1 §6.1).
    func testPreeditReplaceSetsMarked() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 1, insert: "được", preedit: "được")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(marked.text, "được")
        XCTAssertEqual(target.markedSet, 1)
        XCTAssertEqual(target.inserted.count, 0)
    }

    /// COMMIT: preedit hiện tại + ký tự ranh giới thành text (B2 — P2-1 §7).
    func testPreeditCommitInsertsBoundary() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("chào")
        try ApplyReplace.apply(
            outcome(.commit(insert: "\n")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.inserted.first?.text, "chào\n")
        XCTAssertTrue(marked.isEmpty)
    }

    /// RESTORE (ESC): marked → thay bằng raw (P0-2 §2).
    func testPreeditRestore() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("đang")
        try ApplyReplace.apply(
            outcome(.restore(deleteCount: 4, insert: "dang")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.inserted.first?.text, "dang")
        XCTAssertTrue(marked.isEmpty)
    }

    /// SelectionReplace với selection thật: thay selection, 0 backspace (B1).
    func testSelectionReplaceUsesSelection() throws {
        let target = MockTextTarget()
        target.selection = NSRange(location: 3, length: 3) // "viet" chọn hết
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 3, insert: "viêt", preedit: "")),
            strategy: .selectionReplace, target: target, marked: marked
        )
        XCTAssertEqual(target.inserted.first?.replacement, NSRange(location: 3, length: 3))
        XCTAssertEqual(target.deletedTotal, 0) // KHÔNG backspace
    }

    /// SelectionReplace không có selection → fallback BackspaceType (P2-1 §6.2).
    func testSelectionReplaceFallsBackToBackspace() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 3, insert: "viêt", preedit: "")),
            strategy: .selectionReplace, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 3)
        XCTAssertEqual(target.inserted.first?.text, "viêt")
    }

    /// BackspaceType: delete_count × deleteBackward + chèn (MAC-015).
    func testBackspaceType() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 2, insert: "đưo", preedit: "")),
            strategy: .backspaceType, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 2)
        XCTAssertEqual(target.inserted.first?.text, "đưo")
    }

    /// deleteBackward bị từ chối → throw → caller fail-open (P2-1 §12).
    func testBackspaceTypeFailOpen() {
        let target = MockTextTarget()
        target.allowDeleteBackward = false
        let marked = MarkedState()
        XCTAssertThrowsError(
            ApplyReplace.apply(
                outcome(.replace(deleteCount: 2, insert: "được", preedit: "")),
                strategy: .backspaceType, target: target, marked: marked
            )
        )
        XCTAssertEqual(target.inserted.count, 0, "không được chèn khi xóa fail")
    }

    /// B11: preedit vượt 8 grapheme → commit-early.
    func testMarkedLimitCommitsEarly() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        let long = "abcdefghi" // 9 graphemes
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 9, insert: long, preedit: long)),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.inserted.first?.text, long, "vượt limit → commit-early")
        XCTAssertEqual(target.markedSet, 0, "không setMarked khi vượt limit")
    }

    /// ForwardAsCommit = BackspaceType mechanics (quyết định ApplyReplace §6.3).
    func testForwardAsCommitAppliesDeleteAndInsert() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 4, insert: "được", preedit: "")),
            strategy: .forwardAsCommit, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 4)
        XCTAssertEqual(target.inserted.first?.text, "được")
    }
}

// ---------------------------------------------------------------- Marked state

final class MarkedStateTests: XCTestCase {
    func testGraphemeLimit() {
        XCTAssertTrue(MarkedState.exceedsLimit("123456789")) // 9
        XCTAssertFalse(MarkedState.exceedsLimit("12345678")) // 8
        // Grapheme giữ nguyên nghĩa: "được" = 4.
        XCTAssertFalse(MarkedState.exceedsLimit("được"))
    }
}
