// SPDX-License-Identifier: GPL-3.0-or-later
//! AssistModel — mô hình dữ liệu và logic Trợ lý ngữ pháp macOS (P4-0, AS-060).
//!
//! Khớp 1-1 với ConfigModel / schema schemas/config.v1.schema.json và crate config.

import CoreBridge
import Foundation

/// Cấu hình hiển thị gợi ý (D11 · P4-0 §7).
public struct AssistDisplay: Codable, Equatable {
    public var show_on_sentence_end: Bool
    public var idle_ms: Int
    public var show_on_comma: Bool

    public init(
        show_on_sentence_end: Bool = true,
        idle_ms: Int = 700,
        show_on_comma: Bool = false
    ) {
        self.show_on_sentence_end = show_on_sentence_end
        self.idle_ms = idle_ms
        self.show_on_comma = show_on_comma
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let d = AssistDisplay()
        show_on_sentence_end =
            try c.decodeIfPresent(Bool.self, forKey: .show_on_sentence_end) ?? d.show_on_sentence_end
        idle_ms = try c.decodeIfPresent(Int.self, forKey: .idle_ms) ?? d.idle_ms
        show_on_comma =
            try c.decodeIfPresent(Bool.self, forKey: .show_on_comma) ?? d.show_on_comma
    }
}

/// Cấu hình Trợ lý ngữ pháp (P4-0 §7 · AS-060). Mặc định tắt hoàn toàn (mode: "off").
public struct AssistSettings: Codable, Equatable {
    public var mode: String
    public var auto_fix: String
    public var language: String
    public var trigger_keys: [String]
    public var display: AssistDisplay
    public var max_sentence_chars: Int
    public var per_app: [String: Bool]

    public init(
        mode: String = "off",
        auto_fix: String = "off",
        language: String = "vi",
        trigger_keys: [String] = ["space", ",", "."],
        display: AssistDisplay = AssistDisplay(),
        max_sentence_chars: Int = 500,
        per_app: [String: Bool] = [:]
    ) {
        self.mode = mode
        self.auto_fix = auto_fix
        self.language = language
        self.trigger_keys = trigger_keys
        self.display = display
        self.max_sentence_chars = max_sentence_chars
        self.per_app = per_app
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let d = AssistSettings()
        mode = try c.decodeIfPresent(String.self, forKey: .mode) ?? d.mode
        auto_fix = try c.decodeIfPresent(String.self, forKey: .auto_fix) ?? d.auto_fix
        language = try c.decodeIfPresent(String.self, forKey: .language) ?? d.language
        trigger_keys = try c.decodeIfPresent([String].self, forKey: .trigger_keys) ?? d.trigger_keys
        display = try c.decodeIfPresent(AssistDisplay.self, forKey: .display) ?? d.display
        max_sentence_chars =
            try c.decodeIfPresent(Int.self, forKey: .max_sentence_chars) ?? d.max_sentence_chars
        per_app = try c.decodeIfPresent([String: Bool].self, forKey: .per_app) ?? d.per_app
    }

    public var isOn: Bool { mode != "off" }
    public var isHighlight: Bool { mode == "highlight" }
    public var isAutoFixSafe: Bool { auto_fix == "safe" }
}

/// Parser cấu hình assist từ config.json.
public enum AssistConfigParser {
    public static func parse(from data: Data?) -> AssistSettings {
        guard let data,
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
              let assistObj = obj["assist"]
        else {
            return AssistSettings()
        }
        guard let assistData = try? JSONSerialization.data(withJSONObject: assistObj),
              let settings = try? JSONDecoder().decode(AssistSettings.self, from: assistData)
        else {
            return AssistSettings()
        }
        return settings
    }
}

/// Kế hoạch tự sửa cả câu (AS-050 / Mode 2 / A3) — tương đồng SentenceFixPlan của Windows TSF.
public struct SentenceFixPlan: Equatable {
    public let fixedCount: Int
    public let deleteFrom: Int
    public let deleteChars: Int
    public let deleteGlyphs: Int
    public let replacement: String
    public let originalTail: String

    public init(
        fixedCount: Int,
        deleteFrom: Int,
        deleteChars: Int,
        deleteGlyphs: Int,
        replacement: String,
        originalTail: String
    ) {
        self.fixedCount = fixedCount
        self.deleteFrom = deleteFrom
        self.deleteChars = deleteChars
        self.deleteGlyphs = deleteGlyphs
        self.replacement = replacement
        self.originalTail = originalTail
    }
}

/// Trạng thái hoàn tác của lần tự sửa câu A3 gần nhất (giữ trong 5 giây theo spec P4-0 §4.3).
public struct AssistUndoState: Equatable {
    public let timestamp: Date
    public let originalTail: String
    public let replacementGlyphs: Int
    public let fixedCount: Int

    public init(
        timestamp: Date = Date(),
        originalTail: String,
        replacementGlyphs: Int,
        fixedCount: Int
    ) {
        self.timestamp = timestamp
        self.originalTail = originalTail
        self.replacementGlyphs = replacementGlyphs
        self.fixedCount = fixedCount
    }

    public func isValid(now: Date = Date(), timeout: TimeInterval = 5.0) -> Bool {
        now.timeIntervalSince(timestamp) <= timeout
    }
}

/// Tính kế hoạch tự sửa câu từ danh sách lỗi của engine (AS-050 / Mode 2 / A3).
///
/// - Parameters:
///   - sentence: Chuỗi câu hiện tại trong buffer của engine.
///   - issues: Danh sách lỗi do engine trả về (`assistCheck()`).
///   - glyphLen: Closure tính số glyph của chuỗi theo bảng mã xuất (từ `engine.glyphLen`).
/// - Returns: `SentenceFixPlan` nếu có lỗi objective sửa được; `nil` nếu không có sửa đổi hoặc lỗi chồng lấn.
public func planSentenceAutoFix(
    sentence: String,
    issues: [AssistIssue],
    glyphLen: (String) -> Int
) -> SentenceFixPlan? {
    let chars = Array(sentence)
    guard !chars.isEmpty else { return nil }

    // 1. Thu thập các ứng viên sửa lỗi objective (autoFixable == true)
    struct Candidate {
        let start: Int
        let end: Int
        let replacement: String
    }

    var candidates: [Candidate] = []
    for issue in issues {
        guard issue.autoFixable else { continue }
        let start = Int(issue.start)
        let end = Int(issue.end)
        guard start < end, end <= chars.count else { continue }

        let suggestion = issue.suggest
        if suggestion.unicodeScalars.contains(where: { CharacterSet.controlCharacters.contains($0) }) {
            continue
        }

        let spanText = String(chars[start..<end])
        let sameWord = spanText.lowercased() == suggestion.lowercased()
        let prevIsSpace = start > 0 && chars[start - 1] == " "

        if sameWord && prevIsSpace {
            // Lặp từ: xoá từ thứ hai cùng dấu cách liền trước (semantics Fix::Delete)
            candidates.append(Candidate(start: start - 1, end: end, replacement: ""))
        } else if spanText != suggestion && !suggestion.isEmpty {
            candidates.append(Candidate(start: start, end: end, replacement: suggestion))
        }
    }

    guard !candidates.isEmpty else { return nil }

    // 2. Sắp xếp theo vị trí bắt đầu
    candidates.sort { $0.start < $1.start }

    // 3. Lọc bỏ các khoảng chồng lấn (overlap)
    var nonOverlapping: [Candidate] = []
    var lastEnd = 0
    for cand in candidates {
        if cand.start < lastEnd {
            continue // Overlap -> bỏ qua để an toàn
        }
        lastEnd = cand.end
        nonOverlapping.append(cand)
    }

    guard !nonOverlapping.isEmpty else { return nil }

    // 4. delete_from là vị trí bắt đầu của lỗi đầu tiên cần sửa
    let deleteFrom = nonOverlapping[0].start
    let originalTail = String(chars[deleteFrom...])

    // 5. Dựng new_tail từ deleteFrom
    var newTail = ""
    newTail.reserveCapacity(originalTail.count)
    var cursor = deleteFrom
    for cand in nonOverlapping {
        if cand.start > cursor {
            newTail.append(contentsOf: chars[cursor..<cand.start])
        }
        newTail.append(cand.replacement)
        cursor = cand.end
    }
    if cursor < chars.count {
        newTail.append(contentsOf: chars[cursor...])
    }

    if newTail == originalTail {
        return nil // Không có thay đổi thực tế
    }

    let glyphs = glyphLen(originalTail)
    guard glyphs > 0 else { return nil }

    return SentenceFixPlan(
        fixedCount: nonOverlapping.count,
        deleteFrom: deleteFrom,
        deleteChars: originalTail.count,
        deleteGlyphs: glyphs,
        replacement: newTail,
        originalTail: originalTail
    )
}