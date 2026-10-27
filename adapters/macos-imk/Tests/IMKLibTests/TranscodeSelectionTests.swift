// SPDX-License-Identifier: GPL-3.0-or-later
//! TranscodeSelectionTests — OP-007 · R2-139 trên macOS: chord `Ctrl+Shift+F9`
//! chuyển mã vùng bôi đen qua `ime_convert_selection` (FFI thật).
//!
//! Kiểm tra:
//! 1. `isTranscodeChord`: đúng Ctrl+Shift (bỏ bit Caps/Fn — ADR-011), không
//!    Alt/Cmd; Ctrl+F9 / Shift+F9 (Recall) và Ctrl+Shift+G không dính.
//! 2. `ImeEngine.convertSelection` — wrapper FFI `ime_convert_selection`:
//!    chữ ASCII gõ nhầm chế độ E → gõ lại theo Telex; chữ Việt chuẩn, số thuần,
//!    rỗng → `nil` (không đổi gì ⇒ adapter KHÔNG ghi document).

import AppKit
import Carbon.HIToolbox
@testable import CoreBridge
@testable import IMKLib
import XCTest

final class TranscodeSelectionTests: XCTestCase {

    // ------------------------------------------------------------- 1. Hotkey predicate

    /// Parity với `win-tsf assist::is_transcode_chord` (test `transcode_chord_chi_ctrl_shift_f9`):
    /// chỉ Ctrl+Shift+F9 đúng tổ hợp; CapsLock/Fn bỏ qua mask (ADR-011); Alt/Cmd → sai.
    func testTranscodeChordChiCtrlShiftF9() {
        let ctrlShift = FFI.modCtrl | FFI.modShift
        XCTAssertTrue(
            TextVNInputController.isTranscodeChord(mods: ctrlShift, keyCode: UInt16(kVK_F9)))
        XCTAssertTrue(
            TextVNInputController.isTranscodeChord(
                mods: ctrlShift | FFI.modCaps, keyCode: UInt16(kVK_F9)),
            "CapsLock bật không được làm chết hotkey (ADR-011)")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(mods: FFI.modCtrl, keyCode: UInt16(kVK_F9)),
            "Ctrl+F9 là Recall — không dính")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(mods: FFI.modShift, keyCode: UInt16(kVK_F9)),
            "Shift+F9 là Recall — không dính")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(
                mods: ctrlShift | FFI.modAlt, keyCode: UInt16(kVK_F9)),
            "có Alt → sai")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(
                mods: ctrlShift | FFI.modMeta, keyCode: UInt16(kVK_F9)),
            "có Cmd → sai")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(
                mods: ctrlShift, keyCode: UInt16(kVK_G)),
            "Ctrl+Shift+G là Recall — sai")
        XCTAssertFalse(
            TextVNInputController.isTranscodeChord(
                mods: ctrlShift, keyCode: UInt16(kVK_Space)),
            "Ctrl+Shift+Space là toggle EN/VN — sai")
        // Đối chứng: F9 không đụng hotkey toggle Ctrl+Shift+Space.
        XCTAssertFalse(TextVNInputController.isToggleChord(mods: ctrlShift, keyCode: UInt16(kVK_F9)))
    }

    // ------------------------------------------------------------- 2. Engine (FFI thật)

    /// Parity với `core/tests/selection_convert.rs` (case Telex → chữ Việt):
    /// "Vieetj Nam." gõ nhầm mode E → "Việt Nam." (gõ lại); chữ Việt chuẩn /
    /// số thuần / rỗng → nil (không đổi gì).
    func testConvertSelectionRetypeVaKhongDoi() throws {
        let engine = try ImeEngine(
            configJSON: Data(#"{"config_version":1,"enabled":true,"method":"telex"}"#.utf8))
        XCTAssertEqual(
            engine.convertSelection("Vieetj Nam."), "Việt Nam.", "ASCII gõ nhầm chế độ E → gõ lại")
        XCTAssertNil(
            engine.convertSelection("Đường đi ở đâu"), "chữ Việt chuẩn không đổi")
        XCTAssertNil(engine.convertSelection("123 456"), "số thuần không đổi")
        XCTAssertNil(engine.convertSelection(""), "rỗng không đổi")
    }
}
