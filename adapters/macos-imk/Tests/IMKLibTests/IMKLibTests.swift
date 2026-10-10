// SPDX-License-Identifier: GPL-3.0-or-later
//! IMKLibTests — unit test logic adapter macOS (MAC-030, P2-3 §6).
//!
//! Chạy trên macOS runner: `swift test --package-path adapters/macos-imk`
//! (cần `build-rust.sh --lib-only` trước — link libtextvn_ffi.a).

import AppKit
import XCTest
// Hằng `kVK_Space` / `kVK_ANSI_A` nằm ở Carbon.HIToolbox; test dùng chúng để
// kiểm chord toggle mà không cần dựng IMKServer.
import Carbon.HIToolbox

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
        XCTAssertEqual(points.count, 5) // đ,ư,ợ,c + 😀 — UTF-32: 1 unit/scalar
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
        // App thật đang giữ marked range tương ứng (thiếu = kịch bản F9).
        target.setMarked("chào", selectionRange: NSRange(location: 4, length: 0))
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
            try ApplyReplace.apply(
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
        XCTAssertEqual(target.deletedTotal, 9, "không còn marked → 9 ký tự owned là chữ thật")
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

// ---------------------------------------------------------------- IPC codec (R1)

final class IpcCodecTests: XCTestCase {
    func testDecodeWireSamplesIpcV1() {
        // ConfigReload: version LÀ SỐ — decode String làm rớt kết nối (R1 F14).
        let reload = Data("{\"type\":\"ConfigReload\",\"version\":42}".utf8)
        XCTAssertEqual(IpcMessage.decode(reload), .configReload(version: 42))
        // StateUpdate: version số + per-app.
        let update = Data("{\"type\":\"StateUpdate\",\"app_id\":\"com.apple.safari\",\"enabled\":false,\"version\":7}".utf8)
        XCTAssertEqual(
            IpcMessage.decode(update),
            .stateUpdate(appID: "com.apple.safari", enabled: false, version: 7)
        )
        // Hello: version là CHUỖI.
        let hello = Data("{\"type\":\"Hello\",\"pid\":123,\"abi\":1,\"version\":\"0.2.0\"}".utf8)
        XCTAssertEqual(IpcMessage.decode(hello), .hello(pid: 123, abi: 1, version: "0.2.0"))
        // ToggleViEn: chiều client→server.
        let toggle = Data("{\"type\":\"ToggleViEn\",\"app_id\":\"*\",\"enabled\":true}".utf8)
        XCTAssertEqual(IpcMessage.decode(toggle), .toggleViEn(appID: "*", enabled: true))
        // Snapshot: appdb_version SỐ (CR-34); chuỗi của app bản cũ vẫn nhận.
        let snap = Data("{\"type\":\"Snapshot\",\"config_version\":3,\"state\":{\"*\":true},\"appdb_version\":1,\"channel\":\"stable\"}".utf8)
        XCTAssertEqual(
            IpcMessage.decode(snap),
            .snapshot(configVersion: 3, state: ["*": true], appdbVersion: "1", channel: "stable")
        )
        let oldSnap = Data("{\"type\":\"Snapshot\",\"config_version\":3,\"state\":{},\"appdb_version\":\"1.0\",\"channel\":\"stable\"}".utf8)
        XCTAssertEqual(
            IpcMessage.decode(oldSnap),
            .snapshot(configVersion: 3, state: [:], appdbVersion: "1.0", channel: "stable")
        )
    }

    func testDecodeRejectsMalformed() {
        // pid tràn Int32 → từ chối, KHÔNG trap (`Int32.init(Int)` tràn là crash).
        let overflow = Data("{\"type\":\"Hello\",\"pid\":2147483648,\"abi\":1,\"version\":\"x\"}".utf8)
        XCTAssertNil(IpcMessage.decode(overflow))
        // type lạ → violation (danh sách v1 đóng — ipc.v1.md).
        XCTAssertNil(IpcMessage.decode(Data("{\"type\":\"Mystery\"}".utf8)))
        // JSON hỏng → violation.
        XCTAssertNil(IpcMessage.decode(Data("not json".utf8)))
        // thiếu field bắt buộc.
        XCTAssertNil(IpcMessage.decode(Data("{\"type\":\"ToggleViEn\",\"app_id\":\"*\"}".utf8)))
    }

    // ---- nextFrame: u32 LE prefix (R1 F17) ----

    private func push(_ payload: String, into buffer: inout Data) {
        var frame = Data(count: 4)
        frame.withUnsafeMutableBytes { raw in
            raw.storeBytes(of: UInt32(payload.utf8.count).littleEndian, as: UInt32.self)
        }
        frame.append(Data(payload.utf8))
        buffer.append(frame)
    }

    func testNextFrameSplitsCorrectly() {
        var buffer = Data()
        push("abc", into: &buffer)
        push("được", into: &buffer)
        guard case let .frame(first) = IpcClient.nextFrame(from: &buffer) else {
            return XCTFail("cần frame đầu")
        }
        XCTAssertEqual(first, Data("abc".utf8))
        guard case let .frame(second) = IpcClient.nextFrame(from: &buffer) else {
            return XCTFail("cần frame thứ hai")
        }
        XCTAssertEqual(second, Data("được".utf8))
        if case .needMore = IpcClient.nextFrame(from: &buffer) {} else {
            XCTFail("buffer rỗng → needMore")
        }
    }

    func testNextFrameViolations() {
        // length = 0 → violation.
        var zero = Data(count: 8)
        zero.withUnsafeMutableBytes { $0.storeBytes(of: UInt32(0).littleEndian, as: UInt32.self) }
        if case .violation = IpcClient.nextFrame(from: &zero) {} else {
            XCTFail("length 0 → violation")
        }
        // length > max → violation.
        var tooBig = Data(count: 4)
        tooBig.withUnsafeMutableBytes { $0.storeBytes(of: UInt32(70_000).littleEndian, as: UInt32.self) }
        if case .violation = IpcClient.nextFrame(from: &tooBig) {} else {
            XCTFail("length > max → violation")
        }
        // đủ 4 byte length nhưng payload chưa đủ → needMore.
        var partial = Data(count: 4)
        partial.withUnsafeMutableBytes { $0.storeBytes(of: UInt32(10).littleEndian, as: UInt32.self) }
        partial.append(Data("abc".utf8))
        if case .needMore = IpcClient.nextFrame(from: &partial) {} else {
            XCTFail("payload thiếu → needMore")
        }
    }
}

// ---------------------------------------------------------------- ApplyReplace regressions (R1)

final class ApplyReplaceRegressionTests: XCTestCase {
    private func outcome(
        _ action: KeyOutcome.Action, flags: UInt32 = FFI.flagConsumed
    ) -> KeyOutcome {
        KeyOutcome(action: action, flags: flags)
    }

    /// F3 — activate-transition: marked rỗng + delete_count 4 (chữ thật "duoc"
    /// đã vào document qua PASS) → xóa thật 4 ký tự rồi mới setMarked.
    func testPreeditReplaceDeletesPassedPrefix() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 4, insert: "duọc", preedit: "duọc")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 4, "passed prefix phải xóa THẬT (F3)")
        XCTAssertEqual(marked.text, "duọc")
        XCTAssertEqual(target.inserted.count, 0)
    }

    /// F4 — marked active + BackspaceType: thu hồi marked (setMarkedText("")),
    /// KHÔNG unmark-commit → không nhân đôi ("duđu").
    func testBackspaceTypeWithMarkedActiveDoesNotDuplicate() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("du")
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 2, insert: "đu", preedit: "")),
            strategy: .backspaceType, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 0, "marked thu hồi bằng composition")
        XCTAssertEqual(target.inserted.map(\.text), ["đu"], "chỉ 1 lần chèn")
        XCTAssertTrue(marked.isEmpty)
    }

    /// F4 variant — owned > marked: xóa đúng phần chữ thật.
    func testBackspaceTypeDeletesRealPrefixBeyondMarked() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("ưo")
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 5, insert: "được", preedit: "")),
            strategy: .backspaceType, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 3, "5 owned - 2 marked = 3 chữ thật")
        XCTAssertEqual(target.inserted.map(\.text), ["được"])
    }

    /// F5 — REPLACE + preedit rỗng (macro) dưới strategy Preedit → backspaceType,
    /// macro KHÔNG được gạch chân marked.
    func testPreeditStrategyWithEmptyPreeditUsesBackspaceMechanics() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 4, insert: "Cảm ơn", preedit: "")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 4)
        XCTAssertEqual(target.inserted.map(\.text), ["Cảm ơn"])
        XCTAssertTrue(marked.isEmpty)
    }

    /// F6 — commit-early B11 phải gọi onReset, chỉ commit phần đầu đang marked.
    func testMarkedLimitCommitsEarlyAndResets() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("abcdefgh")
        target.setMarked("abcdefgh", selectionRange: NSRange(location: 8, length: 0))
        var resetCalled = false
        let long = "abcdefghi" // 9 graphemes → vượt limit
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 8, insert: long, preedit: long)),
            strategy: .preedit, target: target, marked: marked,
            onReset: { resetCalled = true }
        )
        XCTAssertTrue(resetCalled, "commit-early phải reset engine (F6)")
        // Commit cả phím thứ 9 — không được nuốt mất ký tự (MAC-032).
        XCTAssertEqual(target.inserted.first?.text, "abcdefghi", "commit toàn bộ preedit mới")
        XCTAssertEqual(target.deletedTotal, 0, "marked range được thay, không xóa chữ thật")
        XCTAssertTrue(marked.isEmpty)
    }

    /// COMMIT khi app đã tự commit marked (markedRange notFound) → chỉ chèn
    /// boundary, KHÔNG nhân đôi pending (F9).
    func testCommitWhenAppAlreadyCommittedMarked() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("chào")
        target.markedRangeValue = NSRange(location: NSNotFound, length: 0)
        try ApplyReplace.apply(
            outcome(.commit(insert: "\n")),
            strategy: .preedit, target: target, marked: marked
        )
        XCTAssertEqual(target.inserted.map(\.text), ["\n"], "chỉ boundary — marked đã do app commit")
    }

    /// Đơn vị đếm (F7): marked có emoji — scalar (engine) ≠ utf16 (NSRange) ≠ grapheme.
    func testUnitConsistencyWithEmoji() throws {
        let target = MockTextTarget()
        let marked = MarkedState()
        marked.set("😀x") // 2 scalar / 3 utf16 / 2 grapheme
        try ApplyReplace.apply(
            outcome(.replace(deleteCount: 2, insert: "OK", preedit: "")),
            strategy: .backspaceType, target: target, marked: marked
        )
        XCTAssertEqual(target.deletedTotal, 0, "2 owned = 2 scalar marked → 0 chữ thật")
        XCTAssertEqual(target.inserted.map(\.text), ["OK"])
    }
}

final class MarkedStateUnitTests: XCTestCase {
    /// Ba đơn vị đếm khác nhau (F7): scalar (engine) vs utf16 (NSRange) vs grapheme (B11).
    func testThreeCountingUnits() {
        let marked = MarkedState()
        marked.set("đư😀") // 3 scalar, 4 utf16, 3 grapheme
        XCTAssertEqual(marked.scalarCount, 3)
        XCTAssertEqual(marked.utf16Count, 4)
        XCTAssertEqual(marked.graphemeCount, 3)
        marked.clear()
        XCTAssertTrue(marked.isEmpty)
    }
}

/// Trạng thái EN/VN + hotkey toggle — logic tách khỏi IMKServer để CI macOS
/// chạy được (F3-13 / ADR-011 / per-app state).
final class ViStateAndHotkeyTests: XCTestCase {
    func testToggleChordIgnoresCapsAndFn() {
        let ctrlShift = FFI.modCtrl | FFI.modShift
        XCTAssertTrue(TextVNInputController.isToggleChord(mods: ctrlShift, keyCode: UInt16(kVK_Space)))
        XCTAssertTrue(
            TextVNInputController.isToggleChord(
                mods: ctrlShift | FFI.modCaps | FFI.modFn, keyCode: UInt16(kVK_Space)),
            "Caps Lock/Fn bật vẫn phải toggle được (ADR-011) — so mask, không so =="
        )
        XCTAssertFalse(TextVNInputController.isToggleChord(mods: FFI.modCtrl, keyCode: UInt16(kVK_Space)))
        XCTAssertFalse(
            TextVNInputController.isToggleChord(
                mods: ctrlShift | FFI.modAlt, keyCode: UInt16(kVK_Space)),
            "thêm Alt = chord khác, không toggle"
        )
        XCTAssertFalse(
            TextVNInputController.isToggleChord(mods: ctrlShift, keyCode: UInt16(kVK_ANSI_A)))
    }

    func testCtrlShiftTapDetection() {
        // Cả hai cùng nhấn, sau đó nhả Shift mà không ấn phím khác -> toggle = true
        XCTAssertTrue(
            TextVNInputController.isCtrlShiftTap(
                wasCtrl: true, wasShift: true,
                currentCtrl: true, currentShift: false,
                otherKeyPressed: false
            )
        )
        // Cả hai cùng nhấn, sau đó nhả Ctrl mà không ấn phím khác -> toggle = true
        XCTAssertTrue(
            TextVNInputController.isCtrlShiftTap(
                wasCtrl: true, wasShift: true,
                currentCtrl: false, currentShift: true,
                otherKeyPressed: false
            )
        )
        // Có phím khác chen vào (ví dụ Ctrl+Shift+A) -> toggle = false
        XCTAssertFalse(
            TextVNInputController.isCtrlShiftTap(
                wasCtrl: true, wasShift: true,
                currentCtrl: true, currentShift: false,
                otherKeyPressed: true
            )
        )
        // Chỉ mới nhấn một phím (Shift chưa nhấn) -> toggle = false
        XCTAssertFalse(
            TextVNInputController.isCtrlShiftTap(
                wasCtrl: true, wasShift: false,
                currentCtrl: false, currentShift: false,
                otherKeyPressed: false
            )
        )
    }

    func testViStateSnapshotAndPerAppOverrides() {
        var state = ViState(globalEnabled: true)
        state.apply(snapshot: ["*": false, "com.apple.safari": true])
        XCTAssertFalse(state.globalEnabled)
        XCTAssertTrue(state.enabled(for: "com.apple.safari"), "override per-app thắng mặc định")
        XCTAssertFalse(state.enabled(for: "com.apple.TextEdit"))

        // IMK bản cũ gửi "" — quy về "*" (một quy ước, R3 F3-1).
        state.apply(stateUpdate: "", enabled: true)
        XCTAssertTrue(state.globalEnabled)

        state.apply(stateUpdate: "com.google.chrome", enabled: false)
        XCTAssertFalse(state.enabled(for: "com.google.chrome"))
        XCTAssertTrue(state.enabled(for: nil))
    }

    func testConfigEnabledParsing() {
        XCTAssertEqual(
            TextVNInputController.configEnabled(in: Data("{\"enabled\":false}".utf8)), false)
        XCTAssertNil(TextVNInputController.configEnabled(in: Data("{\"method\":\"vni\"}".utf8)))
        XCTAssertNil(TextVNInputController.configEnabled(in: nil))
        XCTAssertNil(TextVNInputController.configEnabled(in: Data("not json".utf8)))
    }

    /// Vòng 15 (BUG-05): non_preedit đọc từ config + downgrade Preedit→BackspaceType.
    func testConfigNonPreeditParsing() {
        XCTAssertTrue(
            TextVNInputController.configNonPreedit(in: Data("{\"non_preedit\":true}".utf8)))
        XCTAssertFalse(
            TextVNInputController.configNonPreedit(in: Data("{\"non_preedit\":false}".utf8)))
        // Thiếu khoá (config cũ) → false = giữ preedit có gạch chân.
        XCTAssertFalse(TextVNInputController.configNonPreedit(in: Data("{\"enabled\":true}".utf8)))
        XCTAssertFalse(TextVNInputController.configNonPreedit(in: nil))
    }

    func testEffectiveStrategyHonorsNonPreedit() {
        let preedit = OutputStrategy.preedit
        let backspace = OutputStrategy.backspaceType

        // non_preedit ON: Preedit → BackspaceType; strategy khác giữ nguyên.
        XCTAssertEqual(
            TextVNInputController.effectiveStrategy(preedit, nonPreedit: true),
            .backspaceType)
        XCTAssertEqual(
            TextVNInputController.effectiveStrategy(backspace, nonPreedit: true),
            .backspaceType)
        XCTAssertEqual(
            TextVNInputController.effectiveStrategy(.selectionReplace, nonPreedit: true),
            .selectionReplace)

        // non_preedit OFF: giữ nguyên (hành vi gạch chân trước nay).
        XCTAssertEqual(
            TextVNInputController.effectiveStrategy(preedit, nonPreedit: false), .preedit)
    }
}

// ---------------------------------------------------------------- B2 Enter/Tab

/// Enter/Tab ở ranh giới: chốt từ, KHÔNG chèn "\n"/"\t" (app nhận phím thật — B2,
/// như TSF/Linux). Gõ tắt (REPLACE) và ký tự in được giữ nguyên.
final class NativeBoundaryTests: XCTestCase {
    func testCommitAndRestoreDropEnterTab() {
        let wordEnd = FFI.flagConsumed | FFI.flagWordEnd
        let commit = KeyOutcome(action: .commit(insert: "\n"), flags: wordEnd)
        XCTAssertEqual(
            TextVNInputController.withoutNativeBoundary(commit),
            KeyOutcome(action: .commit(insert: ""), flags: wordEnd)
        )
        let restore = KeyOutcome(action: .restore(deleteCount: 3, insert: "text\t"), flags: wordEnd)
        XCTAssertEqual(
            TextVNInputController.withoutNativeBoundary(restore),
            KeyOutcome(action: .restore(deleteCount: 3, insert: "text"), flags: wordEnd)
        )
        let space = KeyOutcome(action: .commit(insert: " "), flags: wordEnd)
        XCTAssertNil(TextVNInputController.withoutNativeBoundary(space))
        let macro = KeyOutcome(
            action: .replace(deleteCount: 3, insert: "Công ty", preedit: ""), flags: wordEnd
        )
        XCTAssertNil(TextVNInputController.withoutNativeBoundary(macro))
    }
}


// ---------------------------------------------------------------- R2-43 strategy (FFI thật)

/// Gọi `ime_strategy_resolve` THẬT (libtextvn_ffi.a) — corpus replay dùng strategy
/// của chính engine nên không bắt được lỗi adapter truyền sai con trỏ appdb (R2-43).
final class StrategyResolverTests: XCTestCase {
    private let caps = TextVNInputController.imkCaps

    private func resolve(
        _ appdb: Data, _ appID: String, _ role: UInt32,
        enabled: Bool = true, secure: Bool = false
    ) -> (rc: Int32, strategy: Int64) {
        StrategyResolver.resolve(
            appdb: appdb, appID: appID, role: role, enabled: enabled, secure: secure, caps: caps
        )
    }

    func testEmptyAppdbPassesNullAndResolvesFieldDefaults() {
        // Data() rỗng có baseAddress KHÁC nil — phải đi đường NULL/0, không phải
        // IME_ERR_CONFIG (bản cũ: mọi phím rc=-2 → luôn BackspaceType).
        let editbox = resolve(Data(), "com.apple.textedit", FieldRole.editbox)
        XCTAssertEqual(editbox.rc, FFI.ok)
        XCTAssertEqual(editbox.strategy, OutputStrategy.preedit.rawValue)
        let address = resolve(Data(), "com.apple.safari", FieldRole.addressBar)
        XCTAssertEqual(address.rc, FFI.ok)
        XCTAssertEqual(address.strategy, OutputStrategy.selectionReplace.rawValue, "B1")
        XCTAssertEqual(
            resolve(Data(), "com.apple.textedit", FieldRole.editbox, secure: true).strategy,
            OutputStrategy.passthrough.rawValue, "S3")
        XCTAssertEqual(
            resolve(Data(), "com.apple.textedit", FieldRole.editbox, enabled: false).strategy,
            OutputStrategy.passthrough.rawValue)
    }

    func testAppdbPresetWinsOverFieldDefault() {
        let appdb = Data(#"""
        {"appdb_version":1,"entries":[{"match":{"bundle":"com.microsoft.excel"},
         "when":{"field_role":["editbox"]},"strategy":"SelectionReplace"}]}
        """#.utf8)
        XCTAssertEqual(
            resolve(appdb, "com.microsoft.excel", FieldRole.editbox).strategy,
            OutputStrategy.selectionReplace.rawValue)
        XCTAssertEqual(
            resolve(appdb, "com.apple.textedit", FieldRole.editbox).strategy,
            OutputStrategy.preedit.rawValue)
    }

    func testCorruptAppdbFallsBackToFieldDefaults() {
        let resolver = StrategyResolver(appdb: Data("not json".utf8), caps: caps)
        XCTAssertEqual(
            resolver.strategy(
                appID: "com.apple.safari", role: FieldRole.addressBar, enabled: true, secure: false),
            OutputStrategy.selectionReplace.rawValue)
        XCTAssertFalse(resolver.hasAppdb, "appdb bị FFI từ chối thì bỏ hẳn (S4)")
    }

    func testCachesPerAppFieldAndState() {
        let resolver = StrategyResolver(appdb: nil, caps: caps)
        _ = resolver.strategy(appID: "com.apple.notes", role: FieldRole.body, enabled: true, secure: false)
        _ = resolver.strategy(appID: "com.apple.notes", role: FieldRole.body, enabled: true, secure: false)
        XCTAssertEqual(resolver.cacheCount, 1, "phím kế tiếp cùng app/field không gọi lại FFI")
        XCTAssertEqual(
            resolver.strategy(appID: "com.apple.notes", role: FieldRole.body, enabled: false, secure: false),
            OutputStrategy.passthrough.rawValue)
        XCTAssertEqual(resolver.cacheCount, 2)
    }

    /// `data/appdb.default.json` (build-macos.sh chép vào bundle IMK) parse được và
    /// preset mac có hiệu lực.
    func testRepoBundledAppdbResolvesMacPresets() throws {
        let repo = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // IMKLibTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // macos-imk
            .deletingLastPathComponent() // adapters
            .deletingLastPathComponent() // gốc repo
        let data = try Data(contentsOf: repo.appendingPathComponent("data/appdb.default.json"))
        let resolver = StrategyResolver(appdb: data, caps: caps)
        XCTAssertEqual(
            resolver.strategy(
                appID: "com.microsoft.excel", role: FieldRole.editbox, enabled: true, secure: false),
            OutputStrategy.selectionReplace.rawValue, "mac.excel.cell (B1)")
        XCTAssertEqual(
            resolver.strategy(
                appID: "com.apple.terminal", role: FieldRole.terminal, enabled: true, secure: false),
            OutputStrategy.forwardAsCommit.rawValue)
        XCTAssertTrue(resolver.hasAppdb, "appdb.default.json phải được FFI chấp nhận")
    }

    /// R2-49 + R2-43: strategy thật → cơ chế áp, theo `non_preedit` đọc từ config.
    func testOutputStrategyHonorsNonPreeditFromConfig() {
        XCTAssertEqual(TextVNInputController.outputStrategy(hint: -1, nonPreedit: false), .backspaceType)
        let hint = StrategyResolver(appdb: nil, caps: caps).strategy(
            appID: "com.apple.textedit", role: FieldRole.body, enabled: true, secure: false)
        XCTAssertEqual(TextVNInputController.outputStrategy(hint: hint, nonPreedit: false), .preedit)
        let on = TextVNInputController.configNonPreedit(in: Data(#"{"non_preedit":true}"#.utf8))
        XCTAssertEqual(TextVNInputController.outputStrategy(hint: hint, nonPreedit: on), .backspaceType)
        // Thiếu khoá → gạch chân (quyết định chủ repo, khớp ConfigModel + CHANGELOG 0.2.27).
        let missing = TextVNInputController.configNonPreedit(in: Data("{}".utf8))
        XCTAssertEqual(TextVNInputController.outputStrategy(hint: hint, nonPreedit: missing), .preedit)
        XCTAssertEqual(
            TextVNInputController.outputStrategy(
                hint: OutputStrategy.selectionReplace.rawValue, nonPreedit: true),
            .selectionReplace)
    }
}

// ---------------------------------------------------------------- R2-44 bật/tắt khi offline

final class EnableGatingTests: XCTestCase {
    func testEngineConfigForcesEnabledAndKeepsOtherKeys() throws {
        let out = TextVNInputController.engineConfig(Data(#"""
        {"enabled":false,"method":"vni","macros":[{"trigger":"vn","expand":"Việt Nam"}]}
        """#.utf8))
        let parsed = try JSONSerialization.jsonObject(with: out)
        let obj = try XCTUnwrap(parsed as? [String: Any])
        XCTAssertEqual(obj["enabled"] as? Bool, true)
        XCTAssertEqual(obj["method"] as? String, "vni")
        XCTAssertEqual((obj["macros"] as? [[String: Any]])?.first?["expand"] as? String, "Việt Nam")
        let junk = Data("not json".utf8)
        XCTAssertEqual(TextVNInputController.engineConfig(junk), junk)
    }

    /// Engine THẬT: config.enabled=false nhưng toggle (ctx.enabled) bật phải gõ được.
    func testToggleOnTypesVietnameseEvenIfConfigDisabled() throws {
        let cfg = Data(#"{"config_version":1,"enabled":false,"method":"telex"}"#.utf8)
        let gated = try ImeEngine(configJSON: cfg)
        XCTAssertEqual(Self.typeWord("vieetj", engine: gated), "vieetj", "đối chứng: engine gate theo config")
        let engine = try ImeEngine(configJSON: TextVNInputController.engineConfig(cfg))
        XCTAssertEqual(Self.typeWord("vieetj", engine: engine), "việt")
    }

    /// Mô phỏng document dưới BackspaceType: PASS = app tự chèn phím.
    static func typeWord(_ word: String, engine: ImeEngine) -> String {
        engine.setContext(
            enabled: true, secure: false, fieldRole: FieldRole.editbox,
            caps: FFI.capFieldDetect | FFI.capSelection, appId: "com.apple.notes",
            elementName: nil, hint: OutputStrategy.backspaceType.rawValue
        )
        var doc: [Unicode.Scalar] = []
        for scalar in word.unicodeScalars {
            let ch = scalar.value
            let vk: UInt32 = (ch >= 0x61 && ch <= 0x7A) ? ch - 0x20 : ch
            switch engine.key(KeyEvent(vk: vk, ch: ch, mods: 0, keyDown: true)).action {
            case .pass:
                doc.append(scalar)
            case let .replace(deleteCount, insert, _):
                doc.removeLast(min(deleteCount, doc.count))
                doc.append(contentsOf: insert.unicodeScalars)
            case let .restore(deleteCount, insert):
                doc.removeLast(min(deleteCount, doc.count))
                doc.append(contentsOf: insert.unicodeScalars)
            case let .commit(insert):
                doc.append(contentsOf: insert.unicodeScalars)
            }
        }
        return String(String.UnicodeScalarView(doc))
    }

    func testViStateStoreSeedsOnceAndSharesToggle() {
        let store = ViStateStore()
        store.seedIfNeeded(globalEnabled: false)
        store.seedIfNeeded(globalEnabled: true) // controller tạo sau đọc config cũ: không đè
        XCTAssertFalse(store.state.globalEnabled)
        XCTAssertTrue(store.toggleGlobal(online: true))
        XCTAssertFalse(store.pendingGlobalSync)
    }

    func testOfflineToggleSurvivesSnapshotAndRequestsResync() {
        let store = ViStateStore()
        store.seedIfNeeded(globalEnabled: false)
        XCTAssertTrue(store.toggleGlobal(online: false))
        XCTAssertTrue(store.pendingGlobalSync)
        // Snapshot đầu tiên sau khi TextVN.app chạy lại: "*" cũ KHÔNG đè toggle
        // offline (caller gửi ToggleViEn), per-app vẫn áp.
        XCTAssertTrue(store.applySnapshot(["*": false, "": false, "com.apple.safari": false]))
        XCTAssertTrue(store.state.globalEnabled)
        XCTAssertFalse(store.state.enabled(for: "com.apple.safari"))
        XCTAssertFalse(store.pendingGlobalSync)
        // Snapshot sau đó áp bình thường.
        XCTAssertFalse(store.applySnapshot(["*": false]))
        XCTAssertFalse(store.state.globalEnabled)
    }
}
