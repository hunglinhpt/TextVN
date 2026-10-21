// SPDX-License-Identifier: GPL-3.0-or-later
//! AssistCapitalizeTests — D16 (L6 vs `config.auto_capitalize`) + Q-8 (selection rỗng).
//!
//! Tách file riêng (không sửa `AssistTests.swift`) để không đụng file test đang
//! được agent khác chỉnh.
//!
//! 1. D16: `auto_capitalize` đọc từ **khoá gốc** `config.v1.json`; lớp adapter
//!    **loại** lỗi L6 ("chữ thường đầu câu") khi cờ này bật (IME/app đã tự viết
//!    hoa ⇒ báo L6 là trùng chức năng — Fassist-20).
//! 2. D16 hồi quy: câu "Xin chào. bạn khỏe không" (có sẵn trong
//!    `corpus/assist/vi_basic.ass` → `lowercase_sentence_start@10-11`).
//! 3. Q-8: trước khi sửa, selection phải **RỖNG** — nếu không, thao tác xoá +
//!    `insert(..., NSRange(location: NSNotFound, length: 0))` có thể **nuốt mất**
//!    lựa chọn của người dùng (Windows TSF đã có precondition này).

import AppKit
import CoreBridge
import Foundation
@testable import IMKLib
import XCTest

/// Mock target cho test D16/Q-8 — độc lập `AssistMockTarget` (file khác) để
/// không phụ thuộc file test của agent khác.
final class CapitalizeMockTarget: TextTarget {
    var documentText: String = ""
    var inserted: [(text: String, replacementRange: NSRange)] = []
    var deletedCount: Int = 0
    /// Selection hiện tại — `length > 0` = người dùng đang **bôi đen** text.
    var selection: NSRange

    init(
        initialText: String = "",
        selection: NSRange = NSRange(location: NSNotFound, length: 0)
    ) {
        self.documentText = initialText
        self.selection = selection
    }

    func insert(_ text: String, replacementRange: NSRange) {
        inserted.append((text, replacementRange))
        documentText.append(text)
    }

    func setMarked(_ text: String, selectionRange: NSRange) {}
    func unmark() {}
    func markedRange() -> NSRange { NSRange(location: NSNotFound, length: 0) }
    func selectionRange() -> NSRange { selection }

    func deleteBackward(count: Int) -> Bool {
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

    /// Mô phỏng app đơn giản: text trước caret = toàn bộ document.
    func textBeforeCaret() -> String? { documentText }
}

final class AssistCapitalizeTests: XCTestCase {

    /// Câu có sẵn trong `corpus/assist/vi_basic.ass`:
    /// "Xin chào. bạn khỏe không" → `lowercase_sentence_start@10-11`.
    private let corpusL6 = "Xin chào. bạn khỏe không"

    // ------------------------------------------------------------- 1. Config

    /// D16: `auto_capitalize` là khoá **gốc** của `config.v1.json` (không nằm
    /// trong object `assist`) — mặc định `true` (khớp crate `config`).
    func testAutoCapitalizeDefaultAndParsing() {
        XCTAssertTrue(
            AssistSettings().auto_capitalize,
            "mặc định phải = true (khớp Config::default của crate config)"
        )
        XCTAssertTrue(AssistConfigParser.parse(from: nil).auto_capitalize)
        XCTAssertTrue(AssistConfigParser.parse(from: Data("{}".utf8)).auto_capitalize)

        let off = Data(#"{"auto_capitalize": false}"#.utf8)
        XCTAssertFalse(
            AssistConfigParser.parse(from: off).auto_capitalize,
            "khoá gốc auto_capitalize=false phải được đọc"
        )

        let full = Data(
            #"{"auto_capitalize": false, "assist": {"mode": "highlight", "auto_fix": "safe"}}"#.utf8
        )
        let parsed = AssistConfigParser.parse(from: full)
        XCTAssertFalse(parsed.auto_capitalize)
        XCTAssertTrue(parsed.isHighlight, "object assist vẫn phải parse bình thường")
        XCTAssertTrue(parsed.isAutoFixSafe)
    }

    // ------------------------------------------------------------- 2. D16 filter

    /// D16 hồi quy: `auto_capitalize=true` ⇒ **không** còn lỗi L6.
    func testAutoCapitalizeOnRemovesL6() {
        let issues = ImeEngine.assistCheck(text: corpusL6)
        XCTAssertTrue(
            issues.contains { $0.kind == FFI.assistKindLowercaseSentence },
            "engine phải bắt được L6 (corpus vi_basic.ass)"
        )

        let filtered = filterAutoCapitalizeConflicts(issues, autoCapitalize: true)
        XCTAssertFalse(
            filtered.contains { $0.kind == FFI.assistKindLowercaseSentence },
            "auto_capitalize=true ⇒ L6 phải bị loại (trùng chức năng)"
        )
    }

    /// D16 hồi quy: `auto_capitalize=false` ⇒ lỗi L6 **vẫn còn**.
    func testAutoCapitalizeOffKeepsL6() {
        let issues = ImeEngine.assistCheck(text: corpusL6)
        XCTAssertTrue(issues.contains { $0.kind == FFI.assistKindLowercaseSentence })

        let kept = filterAutoCapitalizeConflicts(issues, autoCapitalize: false)
        XCTAssertTrue(
            kept.contains { $0.kind == FFI.assistKindLowercaseSentence },
            "auto_capitalize=false ⇒ L6 phải còn (không ai viết hoa giúp)"
        )
        let l6 = kept.first { $0.kind == FFI.assistKindLowercaseSentence }
        XCTAssertEqual(l6?.start, 10)
        XCTAssertEqual(l6?.end, 11)
    }

    /// D16: cờ `auto_capitalize` **chỉ** loại L6 — luật khác giữ nguyên (lọc
    /// **sau** khi engine đã check ⇒ không đổi hành vi luật khác).
    func testFilterKeepsOtherRules() {
        let issues = ImeEngine.assistCheck(text: "Xin chào. tôi tôi đi học")
        XCTAssertTrue(issues.contains { $0.kind == FFI.assistKindLowercaseSentence })
        XCTAssertTrue(issues.contains { $0.kind == FFI.assistKindDuplicateWord })

        let filtered = filterAutoCapitalizeConflicts(issues, autoCapitalize: true)
        XCTAssertFalse(filtered.contains { $0.kind == FFI.assistKindLowercaseSentence })
        XCTAssertTrue(
            filtered.contains { $0.kind == FFI.assistKindDuplicateWord },
            "lặp từ phải còn — chỉ L6 bị loại"
        )
    }

    // ------------------------------------------------------------- 3. Q-8 selection

    /// Q-8: người dùng đang **bôi đen** text ⇒ không được xoá/ghi đè selection.
    func testSelectionNotEmptySkipsFix() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let sentence = "Tôi đi học ."
        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 10,
            end: 11,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng",
            suggest: "."
        )
        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }

        let target = CapitalizeMockTarget(
            initialText: sentence,
            selection: NSRange(location: 4, length: 5)
        )
        XCTAssertFalse(
            controller.applyAssistFix(plan: p, target: target),
            "selection không rỗng ⇒ không được sửa"
        )
        XCTAssertEqual(target.deletedCount, 0, "không được xoá gì")
        XCTAssertTrue(target.inserted.isEmpty, "không được chèn gì")
        XCTAssertEqual(target.documentText, sentence, "document không bị đụng")
    }

    /// Q-8: selection rỗng (caret) ⇒ sửa vẫn chạy bình thường (không phá vỡ).
    func testEmptySelectionAppliesFix() {
        let controller = TextVNInputController(server: nil, delegate: nil, client: nil)
        let sentence = "Tôi đi học ."
        let issue = AssistIssue(
            kind: FFI.assistKindSpaceBeforePunct,
            start: 10,
            end: 11,
            autoFixable: true,
            confidence: 90,
            message: "Thừa khoảng trắng",
            suggest: "."
        )
        let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { $0.count })
        XCTAssertNotNil(plan)
        guard let p = plan else { return }

        let target = CapitalizeMockTarget(
            initialText: sentence,
            selection: NSRange(location: 11, length: 0)
        )
        XCTAssertTrue(controller.applyAssistFix(plan: p, target: target))
        XCTAssertEqual(target.documentText, "Tôi đi học.")
    }
}
