// SPDX-License-Identifier: GPL-3.0-or-later
//! AssistTests — kiểm thử toàn diện Trợ lý ngữ pháp macOS (AS-060 / P4-6).
//!
//! Kiểm tra:
//! 1. AssistSettings & AssistDisplay model, parsing từ config.json
//! 2. planSentenceAutoFix cho các luật objective: lặp từ, thừa khoảng trắng trước dấu,
//!    chữ thường đầu câu, khoảng trắng kép, và bỏ qua lỗi spelling không auto-fix được
//! 3. AssistUndoState: thời gian hiệu lực 5s và hết hạn
//! 4. FFI CoreBridge: trigger kind, sentenceTail, glyphLen, assistCheck
//! 5. TextVNInputController:
//!    - Mode 1 (Highlight): hiển thị gợi ý tại kết thúc câu
//!    - Mode 2 (Safe auto-fix): tự động sửa bằng Backspace+Type qua deleteSurroundingText + insertText
//!    - Hỗ trợ Undo 1 bước khi bấm Cmd+Z ngay sau khi auto-fix
//!    - Override per_app

import AppKit
import Carbon.HIToolbox
@testable import CoreBridge
@testable import IMKLib
import XCTest

final class AssistMockTarget: TextTarget {
    var documentText: String = ""
    var inserted: [(text: String, replacement: NSRange)] = []
    var deletedCount: Int = 0
    var allowDelete: Bool = true
    /// Text trước caret mà app trả về (nil = không đọc được → fail-safe).
    var textBeforeCaretValue: String?

    init(initialText: String = "") {
        self.documentText = initialText
    }

    func insert(_ text: String, replacementRange: NSRange) {
        inserted.append((text, replacementRange))
        documentText.append(text)
    }

    func setMarked(_ text: String, selectionRange: NSRange) {}
    func unmark() {}
    func markedRange() -> NSRange { NSRange(location: NSNotFound, length: 0) }
    func selectionRange() -> NSRange { NSRange(location: NSNotFound, length: 0) }

    func deleteBackward(count: Int) -> Bool {
        guard allowDelete else { return false }
        deletedCount += count
        if count <= documentText.count {
            documentText = String(documentText.dropLast(count))
        } else {
            documentText = ""
        }
        return true
    }

    func deleteSurroundingText(before: Int, after: Int) -> Bool {
        deleteBackward(count: before)
    }

    /// Mặc định: text trước caret = toàn bộ document (mô phỏng app đơn giản).
    func textBeforeCaret() -> String? {
        textBeforeCaretValue ?? documentText
    }
}

final class AssistTests: XCTestCase {

    // ------------------------------------------------------------- 1. Config & Model

    func testAssistDisplayDefaultsAndDecoding() throws {
        let display = AssistDisplay()
        XCTAssertTrue(display.show_on_sentence_end)
        XCTAssertEqual(display.idle_ms, 700)
        XCTAssertFalse(display.show_on_comma)

        let json = #"{"show_on_sentence_end": false, "idle_ms": 300, "show_on_comma": true}"#
        let decoded = try JSONDecoder().decode(AssistDisplay.self, from: Data(json.utf8))
        XCTAssertFalse(decoded.show_on_sentence_end)
        XCTAssertEqual(decoded.idle_ms, 300)
        XCTAssertTrue(decoded.show_on_comma)
    }

    func testAssistSettingsDefaultsAndDecoding() throws {
        let settings = AssistSettings()
        XCTAssertEqual(settings.mode, "off")
        XCTAssertEqual(settings.auto_fix, "off")
        XCTAssertEqual(settings.language, "vi")
        XCTAssertFalse(settings.isOn)
        XCTAssertFalse(settings.isHighlight)
        XCTAssertFalse(settings.isAutoFixSafe)

        let highlightJson = #"{"mode": "highlight", "auto_fix": "off"}"#
        let highlightSettings = try JSONDecoder().decode(AssistSettings.self, from: Data(highlightJson.utf8))
        XCTAssertTrue(highlightSettings.isOn)
        XCTAssertTrue(highlightSettings.isHighlight)
        XCTAssertFalse(highlightSettings.isAutoFixSafe)

        let autoFixJson = #"{"mode": "highlight", "auto_fix": "safe"}"#
        let autoFixSettings = try JSONDecoder().decode(AssistSettings.self, from: Data(autoFixJson.utf8))
        XCTAssertTrue(autoFixSettings.isOn)
        XCTAssertTrue(autoFixSettings.isAutoFixSafe)
        XCTAssertFalse(autoFixSettings.isMode2AutoFix)

        let mode2Json = #"{"mode": "autocorrect", "auto_fix": "safe"}"#
        let mode2Settings = try JSONDecoder().decode(AssistSettings.self, from: Data(mode2Json.utf8))
        XCTAssertTrue(mode2Settings.isOn)
        XCTAssertTrue(mode2Settings.isMode2AutoFix)
        XCTAssertTrue(mode2Settings.isDisplayEnabled)
    }

    func testAssistConfigParser() {
        XCTAssertEqual(AssistConfigParser.parse(from: nil), AssistSettings())
        XCTAssertEqual(AssistConfigParser.parse(from: Data("{}".utf8)), AssistSettings())

        let fullJson = """
        {
            "enabled": true,
            "assist": {
                "mode": "highlight",
                "auto_fix": "safe",
                "per_app": {
                    "com.apple.terminal": false
                }
            }
        }
        """
        let parsed = AssistConfigParser.parse(from: Data(fullJson.utf8))
        XCTAssertTrue(parsed.isOn)
        XCTAssertTrue(parsed.isHighlight)
        XCTAssertTrue(parsed.isAutoFixSafe)
        XCTAssertEqual(parsed.per_app["com.apple.terminal"], false)
    }

    // ------------------------------------------------------------- 2. Auto-fix planning

    func testPlanSentenceAutoFixDuplicateWord() {
        let sentence = "chào bạn bạn nhé."
        let issue = AssistIssue(
            kind: FFI.assistKindDuplicateWord,
            start: 9,
            end: 12,
            autoFixable: true,
            confidence: 95,
            message: "Lặp từ",
            suggest: "bạn"
        )

        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }
        XCTAssertEqual(p.fixedCount, 1)
        XCTAssertEqual(p.deleteFrom, 8) // xóa từ space trước từ lặp
        XCTAssertEqual(p.originalTail, " bạn nhé.")
        XCTAssertEqual(p.replacement, " nhé.")
        XCTAssertEqual(p.deleteGlyphs, (" bạn nhé.").count)
    }

    func testPlanSentenceAutoFixSpaceBeforePunct() {
        let sentence = "Chào bạn ."
        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 8,
            end: 10,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng trước dấu câu",
            suggest: "."
        )

        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }
        XCTAssertEqual(p.fixedCount, 1)
        XCTAssertEqual(p.deleteFrom, 8)
        XCTAssertEqual(p.originalTail, " .")
        XCTAssertEqual(p.replacement, ".")
        XCTAssertEqual(p.deleteGlyphs, 2)
    }

    func testPlanSentenceAutoFixLowercaseSentence() {
        let sentence = "hôm nay trời đẹp."
        let issue = AssistIssue(
            kind: FFI.assistKindLowercaseSentence,
            start: 0,
            end: 3,
            autoFixable: true,
            confidence: 90,
            message: "Chữ thường đầu câu",
            suggest: "Hôm"
        )

        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }
        XCTAssertEqual(p.fixedCount, 1)
        XCTAssertEqual(p.deleteFrom, 0)
        XCTAssertEqual(p.originalTail, sentence)
        XCTAssertEqual(p.replacement, "Hôm nay trời đẹp.")
    }

    func testPlanSentenceAutoFixDoubleSpace() {
        let sentence = "Tôi  đi học."
        let issue = AssistIssue(
            kind: FFI.assistKindDoubleSpace,
            start: 3,
            end: 5,
            autoFixable: true,
            confidence: 85,
            message: "Khoảng trắng kép",
            suggest: " "
        )

        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }
        XCTAssertEqual(p.fixedCount, 1)
        XCTAssertEqual(p.deleteFrom, 3)
        XCTAssertEqual(p.originalTail, "  đi học.")
        XCTAssertEqual(p.replacement, " đi học.")
    }

    func testPlanSentenceAutoFixMultipleObjectiveIssues() {
        let sentence = "hôm nay tôi tôi đi  học ."
        let issues = [
            AssistIssue(kind: FFI.assistKindLowercaseSentence, start: 0, end: 3, autoFixable: true, confidence: 90, message: "Hoa", suggest: "Hôm"),
            AssistIssue(kind: FFI.assistKindDuplicateWord, start: 12, end: 15, autoFixable: true, confidence: 95, message: "Lặp", suggest: "tôi"),
            AssistIssue(kind: FFI.assistKindDoubleSpace, start: 18, end: 20, autoFixable: true, confidence: 85, message: "Kép", suggest: " "),
            AssistIssue(kind: FFI.assistKindSpaceBeforePunct, start: 23, end: 25, autoFixable: true, confidence: 90, message: "Dấu", suggest: ".")
        ]

        let plan = planSentenceAutoFix(sentence: sentence, issues: issues, glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }
        XCTAssertEqual(p.fixedCount, 4)
        XCTAssertEqual(p.deleteFrom, 0)
        XCTAssertEqual(p.originalTail, sentence)
        XCTAssertEqual(p.replacement, "Hôm nay tôi đi học.")
    }

    func testPlanSentenceAutoFixSkipsNonAutoFixableSpelling() {
        let sentence = "Hôm nay tôi thấy mọt con mèo."
        let spIssue = AssistIssue(
            kind: FFI.assistKindSpelling,
            start: 17,
            end: 20,
            autoFixable: false, // Lỗi spelling KHÔNG được tự sửa (D12 / A1)
            confidence: 60,
            message: "Không có trong từ điển",
            suggest: "một"
        )

        let plan = planSentenceAutoFix(sentence: sentence, issues: [spIssue], glyphLen: { $0.count })
        XCTAssertNil(plan, "Lỗi autoFixable == false phải bị bỏ qua")
    }

    func testPlanSentenceAutoFixEmptyOrCleanSentence() {
        XCTAssertNil(planSentenceAutoFix(sentence: "", issues: [], glyphLen: { $0.count }))
        XCTAssertNil(planSentenceAutoFix(sentence: "Chào bạn.", issues: [], glyphLen: { $0.count }))
    }

    // ------------------------------------------------------------- 3. Undo State

    func testAssistUndoStateValidityAndExpiration() {
        let now = Date()
        let state = AssistUndoState(
            timestamp: now, originalTail: " .", replacement: ".", replacementGlyphs: 1, fixedCount: 1
        )
        XCTAssertTrue(state.isValid(now: now))
        XCTAssertTrue(state.isValid(now: now.addingTimeInterval(4.9)))
        XCTAssertFalse(state.isValid(now: now.addingTimeInterval(5.1)))
    }

    // ------------------------------------------------------------- 4. FFI Bridge

    func testTriggerKindClassification() {
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x2E), FFI.assistTriggerSentence) // '.'
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x21), FFI.assistTriggerSentence) // '!'
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x3F), FFI.assistTriggerSentence) // '?'
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x20), FFI.assistTriggerWord)     // ' '
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x2C), FFI.assistTriggerWord)     // ','
        XCTAssertEqual(ImeEngine.assistTriggerKind(ch: 0x61), FFI.assistTriggerNone)     // 'a'
    }

    func testEngineSentenceTailAndGlyphLen() throws {
        let engine = try ImeEngine(configJSON: nil)
        engine.setContext(
            enabled: true, secure: false, fieldRole: FieldRole.editbox,
            caps: FFI.capFieldDetect | FFI.capSelection, appId: "com.apple.notes",
            elementName: nil, hint: OutputStrategy.backspaceType.rawValue
        )

        let glyphs = engine.glyphLen("chào bạn")
        XCTAssertEqual(glyphs, 8)

        // Ban đầu câu rỗng
        XCTAssertEqual(engine.sentenceTail(100), "")
    }

    func testEngineAssistCheckIntegration() throws {
        // Kiểm tra hàm standalone assistCheck
        let issues = ImeEngine.assistCheck(text: "hôm nay tôi tôi đi  học .")
        XCTAssertGreaterThanOrEqual(issues.count, 2)
        let hasDuplicate = issues.contains { $0.kind == FFI.assistKindDuplicateWord }
        let hasSpaceBeforePunct = issues.contains { $0.kind == FFI.assistKindSpaceBeforePunct }
        XCTAssertTrue(hasDuplicate || hasSpaceBeforePunct)
    }

    // ------------------------------------------------------------- 5. Controller flow

    func testControllerProcessAssistAutoFixMode() throws {
        let engine = try ImeEngine(configJSON: nil)
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        controller.currentAssistSettings = AssistSettings(
            mode: "highlight",
            auto_fix: "safe",
            language: "vi"
        )

        let target = AssistMockTarget(initialText: "Tôi đi học .")
        // Giả lập engine có lỗi space before punct
        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 10,
            end: 12,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng",
            suggest: "."
        )

        let plan = planSentenceAutoFix(sentence: "Tôi đi học .", issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }

        // R-2/R-7: áp sửa qua precondition (text trước caret phải khớp đuôi câu).
        let applied = controller.applyAssistFix(plan: p, target: target)
        XCTAssertTrue(applied)
        XCTAssertEqual(target.documentText, "Tôi đi học.")

        // Lưu undo
        controller.currentAssistUndo = AssistUndoState(
            timestamp: Date(),
            originalTail: p.originalTail,
            replacement: p.replacement,
            replacementGlyphs: p.replacement.count,
            fixedCount: p.fixedCount
        )

        // Kiểm tra Undo với Cmd+Z
        let reverted = controller.handleAssistUndo(target: target)
        XCTAssertTrue(reverted)
        XCTAssertEqual(target.documentText, "Tôi đi học .")
        XCTAssertNil(controller.currentAssistUndo)
    }

    /// R-2/R-7 (fail-safe): caret ở chỗ khác ⇒ vùng sắp xoá KHÔNG khớp đuôi câu
    /// engine biết ⇒ **không sửa** (không xoá text lạ của người dùng).
    func testAssistFixPreconditionMismatchSkips() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let target = AssistMockTarget(initialText: "Tôi đi học .")
        // App đang chứa text KHÁC (user đã bấm chuột đặt caret chỗ khác).
        target.textBeforeCaretValue = "chỗ khác hoàn toàn"

        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 10,
            end: 12,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng",
            suggest: "."
        )
        let plan = planSentenceAutoFix(sentence: "Tôi đi học .", issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }

        XCTAssertFalse(controller.applyAssistFix(plan: p, target: target))
        XCTAssertEqual(target.documentText, "Tôi đi học .", "document không bị đụng")
        XCTAssertEqual(target.deletedCount, 0, "không được xoá gì")
    }

    /// R-2: không đọc được text trước caret ⇒ không sửa (fail-safe).
    func testAssistFixPreconditionUnavailableSkips() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let target = AssistMockTarget(initialText: "Tôi đi học .")
        target.textBeforeCaretValue = nil

        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 10,
            end: 12,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng",
            suggest: "."
        )
        let plan = planSentenceAutoFix(sentence: "Tôi đi học .", issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }

        XCTAssertFalse(controller.applyAssistFix(plan: p, target: target))
        XCTAssertEqual(target.deletedCount, 0)
    }

    /// R-5: Mode 2 chỉ bật khi mode "autocorrect" **và** auto_fix "safe".
    func testMode2RequiresAutocorrectAndSafe() {
        let highlightSafe = AssistSettings(mode: "highlight", auto_fix: "safe")
        XCTAssertFalse(highlightSafe.isMode2AutoFix, "chỉ auto_fix=safe thì KHÔNG tự sửa")
        let autocorrectOff = AssistSettings(mode: "autocorrect", auto_fix: "off")
        XCTAssertFalse(autocorrectOff.isMode2AutoFix, "chỉ mode=autocorrect thì KHÔNG tự sửa")
        let autocorrectSafe = AssistSettings(mode: "autocorrect", auto_fix: "safe")
        XCTAssertTrue(autocorrectSafe.isMode2AutoFix)
        // R-6: điểm hiển thị bật khi mode != off (kể cả ở Mode 2).
        XCTAssertTrue(autocorrectSafe.isDisplayEnabled)
        XCTAssertFalse(AssistSettings(mode: "off").isDisplayEnabled)
    }

    func testControllerProcessAssistHighlightMode() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        controller.currentAssistSettings = AssistSettings(
            mode: "highlight",
            auto_fix: "off",
            language: "vi"
        )

        XCTAssertTrue(controller.currentAssistSettings.isHighlight)
        XCTAssertFalse(controller.currentAssistSettings.isAutoFixSafe)
    }

    func testControllerAssistPerAppOverride() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        controller.currentAssistSettings = AssistSettings(
            mode: "highlight",
            auto_fix: "safe",
            per_app: [
                "com.apple.terminal": false,
                "com.apple.notes": true
            ]
        )

        XCTAssertFalse(controller.isAssistActive(for: "com.apple.terminal"))
        XCTAssertTrue(controller.isAssistActive(for: "com.apple.notes"))
        XCTAssertTrue(controller.isAssistActive(for: "com.google.chrome")) // mặc định theo mode
    }

    func testCmdZDetection() {
        // Cmd+Z: mods có modMeta, không có Ctrl/Alt/Shift, keyCode = 0x06
        XCTAssertTrue(TextVNInputController.isCmdZ(mods: FFI.modMeta, keyCode: 0x06))

        // Cmd+Shift+Z (Redo): không được nhận là Undo
        XCTAssertFalse(TextVNInputController.isCmdZ(mods: FFI.modMeta | FFI.modShift, keyCode: 0x06))

        // Ctrl+Z: không phải Cmd+Z
        XCTAssertFalse(TextVNInputController.isCmdZ(mods: FFI.modCtrl, keyCode: 0x06))

        // Cmd+C: không phải Z
        XCTAssertFalse(TextVNInputController.isCmdZ(mods: FFI.modMeta, keyCode: 0x08))
    }

    func testControllerAssistUndoExpiresAfterTimeout() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let target = AssistMockTarget(initialText: "Hôm nay tôi đi học.")

        controller.currentAssistUndo = AssistUndoState(
            timestamp: Date().addingTimeInterval(-6.0), // Quá 5 giây
            originalTail: "hôm nay tôi tôi đi  học .",
            replacement: "Hôm nay tôi đi học.",
            replacementGlyphs: 19,
            fixedCount: 3
        )

        let reverted = controller.handleAssistUndo(target: target)
        XCTAssertFalse(reverted, "Undo đã hết hạn 5 giây không được áp dụng")
        XCTAssertEqual(target.documentText, "Hôm nay tôi đi học.", "Document không bị đổi")
        XCTAssertNil(controller.currentAssistUndo)
    }

    /// R-3: hoàn tác chỉ khi vùng sắp xoá **đúng** là text đã gõ lại sau khi
    /// sửa (user gõ thêm ⇒ không xoá nhầm chữ của họ).
    func testAssistUndoRequiresMatchingReplacement() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let target = AssistMockTarget(initialText: "Hôm nay tôi đi học.")
        target.textBeforeCaretValue = "Hôm nay tôi đi học. và thêm nữa"

        controller.currentAssistUndo = AssistUndoState(
            timestamp: Date(),
            originalTail: "hôm nay tôi tôi đi học .",
            replacement: "Hôm nay tôi đi học.",
            replacementGlyphs: 19,
            fixedCount: 1
        )

        let reverted = controller.handleAssistUndo(target: target)
        XCTAssertFalse(reverted, "text trước caret không khớp ⇒ không xoá")
        XCTAssertEqual(target.deletedCount, 0, "không được xoá gì")
    }
}