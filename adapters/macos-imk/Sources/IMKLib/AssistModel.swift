// SPDX-License-Identifier: GPL-3.0-or-later
//! AssistModel — mô hình dữ liệu và logic Trợ lý ngữ pháp macOS (P4-0, AS-060).
//!
//! Khớp 1-1 với ConfigModel / schema schemas/config.v1.schema.json và crate config.
//!
//! Ngoại lệ D16: `AssistSettings.auto_capitalize` là khoá **gốc** của
//! `config.v1.json` (`config.auto_capitalize`, P0-3 §1.1) — KHÔNG nằm trong
//! object `assist`; `AssistConfigParser` đọc riêng rồi gộp vào.

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
    /// `config.auto_capitalize` (khoá **gốc** của `config.v1.json`).
    ///
    /// D16 (Fassist-20): crate `assist` **không** đọc config ⇒ **luôn** trả lỗi
    /// L6 ("chữ thường đầu câu"). Khi cờ này **bật** thì IME/app **đã tự viết
    /// hoa** ⇒ báo L6 là **trùng chức năng** ⇒ lớp adapter phải **loại** L6.
    ///
    /// LƯU Ý: khoá này **không** nằm trong object `assist` — nó là khoá gốc
    /// (`config.auto_capitalize`, P0-3 §1.1). `AssistConfigParser` đọc riêng
    /// khoá gốc rồi gộp vào đây.
    public var auto_capitalize: Bool

    /// Mặc định của `config.auto_capitalize` — phải khớp crate `config`
    /// (`Config::default().auto_capitalize`).
    public static let defaultAutoCapitalize = true

    public init(
        mode: String = "off",
        auto_fix: String = "off",
        language: String = "vi",
        trigger_keys: [String] = ["space", ",", "."],
        display: AssistDisplay = AssistDisplay(),
        max_sentence_chars: Int = 500,
        per_app: [String: Bool] = [:],
        auto_capitalize: Bool = AssistSettings.defaultAutoCapitalize
    ) {
        self.mode = mode
        self.auto_fix = auto_fix
        self.language = language
        self.trigger_keys = trigger_keys
        self.display = display
        self.max_sentence_chars = max_sentence_chars
        self.per_app = per_app
        self.auto_capitalize = auto_capitalize
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
        auto_capitalize =
            try c.decodeIfPresent(Bool.self, forKey: .auto_capitalize) ?? d.auto_capitalize
    }

    public var isOn: Bool { mode != "off" }
    public var isHighlight: Bool { mode == "highlight" }
    public var isAutoFixSafe: Bool { auto_fix == "safe" }

    /// R-5 (P4-0 §4.3): **Mode 2** (tự sửa cả câu) = `mode == "autocorrect"` **và**
    /// `auto_fix == "safe"` — cùng ngữ nghĩa cho cả 3 OS (Windows/Linux dùng cùng
    /// quy tắc). Chỉ cần `auto_fix == "safe"` là lệch ⇒ 1 config, 3 hành vi khác nhau.
    public var isMode2AutoFix: Bool { mode == "autocorrect" && auto_fix == "safe" }

    /// R-6 (D11): **điểm hiển thị** (Mode 1) bật khi `mode != "off"` (ở Mode 2,
    /// lỗi *mơ hồ* vẫn phải hiện candidate — chỉ lỗi objective mới tự sửa).
    public var isDisplayEnabled: Bool { mode != "off" }
}

/// Parser cấu hình assist từ config.json.
///
/// `config.auto_capitalize` là khoá **gốc** của `config.v1.json` (P0-3 §1.1) —
/// **không** nằm trong object `assist`. Parser đọc riêng khoá gốc rồi gộp vào
/// `AssistSettings` (thiếu khoá ⇒ mặc định `true`, khớp crate `config`).
public enum AssistConfigParser {
    public static func parse(from data: Data?) -> AssistSettings {
        guard let data,
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else {
            return AssistSettings()
        }
        // Khoá gốc `auto_capitalize` — đọc trước để dùng cho cả 2 nhánh.
        let autoCapitalize =
            (obj["auto_capitalize"] as? Bool) ?? AssistSettings.defaultAutoCapitalize
        var settings = AssistSettings()
        settings.auto_capitalize = autoCapitalize
        guard let assistObj = obj["assist"] as? [String: Any] else {
            // Không có object `assist` ⇒ assist tắt, nhưng cờ gốc vẫn phải đúng.
            return settings
        }
        var assist = assistObj
        assist["auto_capitalize"] = autoCapitalize
        guard let assistData = try? JSONSerialization.data(withJSONObject: assist),
              let decoded = try? JSONDecoder().decode(AssistSettings.self, from: assistData)
        else {
            return settings
        }
        // `auto_capitalize` đã được chèn vào object `assist` ở trên ⇒ decoder tự
        // đọc (CodingKeys.auto_capitalize) — KHÔNG gán lại ở đây (`decoded` là let).
        return decoded
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
    /// Text **đã gõ lại** sau khi sửa — dùng để XÁC MINH vùng sắp xoá trước khi
    /// hoàn tác (R-3 · fail-safe): nếu người dùng đã gõ thêm ⇒ vùng đó không
    /// còn là của ta ⇒ **không xoá**.
    public let replacement: String
    public let replacementGlyphs: Int
    public let fixedCount: Int

    public init(
        timestamp: Date = Date(),
        originalTail: String,
        replacement: String,
        replacementGlyphs: Int,
        fixedCount: Int
    ) {
        self.timestamp = timestamp
        self.originalTail = originalTail
        self.replacement = replacement
        self.replacementGlyphs = replacementGlyphs
        self.fixedCount = fixedCount
    }

    public func isValid(now: Date = Date(), timeout: TimeInterval = 5.0) -> Bool {
        now.timeIntervalSince(timestamp) <= timeout
    }
}

/// Gợi ý trong candidate panel (Mode 1) — kèm **kế hoạch A2** để khi người dùng
/// chọn thì **thay từ lỗi** (xoá đúng số glyph rồi gõ lại) thay vì *chèn thêm*
/// ở caret (R-7 · A2 · D12).
public struct AssistCandidate: Equatable {
    /// Chuỗi hiển thị trong panel (gợi ý của engine).
    public let text: String
    /// Kế hoạch sửa (xoá `deleteGlyphs` glyph trước caret + gõ lại `replacement`).
    public let plan: SentenceFixPlan

    public init(text: String, plan: SentenceFixPlan) {
        self.text = text
        self.plan = plan
    }
}

/// **D16** (Fassist-20) — loại lỗi L6 ("chữ thường đầu câu") khi
/// `config.auto_capitalize=true`.
///
/// Crate `assist` **không** đọc config (0-dep, không I/O) ⇒ nó **luôn** trả lỗi
/// L6. Khi `auto_capitalize=true` thì IME/app **đã tự viết hoa** ⇒ báo L6 là
/// **trùng chức năng** ⇒ **lớp adapter** (chỗ đọc được config) phải loại.
///
/// Lọc **sau** khi engine đã trả kết quả (không lọc trước khi check) ⇒ các
/// luật khác giữ nguyên hành vi. Áp dụng cho **cả Mode 1** (gợi ý) và
/// **Mode 2** (tự sửa) — cùng nguồn `issues` ở `processAssist`.
public func filterAutoCapitalizeConflicts(
    _ issues: [AssistIssue],
    autoCapitalize: Bool
) -> [AssistIssue] {
    guard autoCapitalize else { return issues }
    return issues.filter { $0.kind != FFI.assistKindLowercaseSentence }
}

/// Tên loại lỗi ổn định (wire format `AssistReport.issues[].kind` — khớp
/// `IssueKind::as_str()` của crate `assist`, xem `schemas/ipc.v1.md`).
public func assistKindName(_ kind: UInt32) -> String {
    switch kind {
    case FFI.assistKindDuplicateWord: return "duplicate_word"
    case FFI.assistKindSpaceBeforePunct: return "space_before_punct"
    case FFI.assistKindSpaceAfterPunct: return "space_after_punct"
    case FFI.assistKindDoubleSpace: return "double_space"
    case FFI.assistKindLowercaseSentence: return "lowercase_sentence_start"
    case FFI.assistKindDiacriticTypo: return "diacritic_typo"
    case FFI.assistKindSentenceRepeat: return "sentence_repeat"
    case FFI.assistKindDoubleNegative: return "double_negative"
    case FFI.assistKindCommaAtEnd: return "comma_at_end"
    case FFI.assistKindSpelling: return "spelling"
    default: return "unknown"
    }
}

/// Tính kế hoạch tự sửa câu từ danh sách lỗi của engine (AS-050 / Mode 2 / A3).
///
/// Ngữ nghĩa sửa quyết định theo **kind** của lỗi (`IME_ASSIST_KIND_*`) — khớp
/// crate `assist` (`assist/src/fix.rs`), `adapters/linux-common/src/compose.c` và
/// adapter Windows (Q-2): `duplicate_word` / `space_before_punct` ⇒ **xoá** span;
/// còn lại ⇒ **thay** span bằng `suggest`. KHÔNG đoán bằng text.
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
        let prevIsSpace = start > 0 && chars[start - 1] == " "

        // Ngữ nghĩa `Fix` theo **kind** (khớp crate `assist` (`assist/src/fix.rs`)
        // và `adapters/linux-common/src/compose.c`):
        // - `duplicate_word` → **xoá** lần lặp + 1 space liền trước (`Fix::Delete`);
        // - `space_before_punct` → **xoá** span (khoảng trắng thừa), `suggest`
        //   chỉ là đầu câu để **hiển thị** (`Fix::Delete`);
        // - còn lại → **thay** span bằng `suggest` (`Fix::Replace`).
        //
        // KHÔNG đoán bằng text (`spanText.lowercased() == suggestion.lowercased()`):
        // L6 ("chữ thường đầu câu") có span = 1 chữ thường, gợi ý = 1 chữ hoa ⇒
        // `"b" == "B"` ⇒ rơi vào nhánh **xoá** ⇒ **mất 1 ký tự** (Q-2 —
        // "Xin chào. bạn khỏe không" → "Xin chào. ạn khỏe không").
        if issue.kind == FFI.assistKindDuplicateWord && prevIsSpace {
            // Lặp từ: xoá từ thứ hai cùng dấu cách liền trước (semantics Fix::Delete)
            candidates.append(Candidate(start: start - 1, end: end, replacement: ""))
        } else if issue.kind == FFI.assistKindSpaceBeforePunct {
            // L3 "thừa khoảng trắng trước dấu câu" (P4-0 §5.1): span = KHOẢNG
            // TRẮNG thừa, còn `suggest` chỉ là **đầu câu để HIỂN THỊ** (dấu
            // câu). Mô hình sửa của engine là `Fix::Delete` ⇒ phải **XOÁ
            // span**, KHÔNG thay span bằng `suggest` — thay span sẽ nhân đôi
            // dấu câu ("chào bạn,,"). Cùng ngữ nghĩa với crate `assist`
            // (`assist/src/rules/space_before_punct.rs`) và với
            // `adapters/linux-common/src/compose.c` (đã sửa).
            candidates.append(Candidate(start: start, end: end, replacement: ""))
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