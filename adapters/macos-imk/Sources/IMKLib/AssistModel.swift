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
    public var hide_on_auto_fix: Bool

    public init(
        show_on_sentence_end: Bool = true,
        idle_ms: Int = 700,
        show_on_comma: Bool = false,
        hide_on_auto_fix: Bool = false
    ) {
        self.show_on_sentence_end = show_on_sentence_end
        self.idle_ms = idle_ms
        self.show_on_comma = show_on_comma
        self.hide_on_auto_fix = hide_on_auto_fix
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let d = AssistDisplay()
        show_on_sentence_end =
            try c.decodeIfPresent(Bool.self, forKey: .show_on_sentence_end) ?? d.show_on_sentence_end
        idle_ms = try c.decodeIfPresent(Int.self, forKey: .idle_ms) ?? d.idle_ms
        show_on_comma =
            try c.decodeIfPresent(Bool.self, forKey: .show_on_comma) ?? d.show_on_comma
        hide_on_auto_fix =
            try c.decodeIfPresent(Bool.self, forKey: .hide_on_auto_fix) ?? d.hide_on_auto_fix
    }
}

/// Cấu hình Trợ lý ngữ pháp (P4-0 §7 · AS-060). Mặc định BẬT ở chế độ
/// Chỉ gợi ý (vòng 26 — khớp serde `Config::default()`); Tự sửa vẫn tắt.
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
        mode: String = "highlight",
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
            // Không có object `assist` ⇒ dùng mặc định (mode="highlight"), giữ cờ gốc.
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
public typealias SentenceFixPlan = CoreBridge.SentenceFixPlan

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
    case FFI.assistKindArticleAn: return "article_an"
    case FFI.assistKindWordConfusion: return "word_confusion"
    case FFI.assistKindLowercasePronoun: return "lowercase_pronoun"
    default: return "unknown"
    }
}

/// Tính kế hoạch tự sửa câu từ danh sách lỗi của engine (AS-050 / Mode 2 / A3)
/// qua C-ABI chung `ime_assist_plan_sentence_fix`.
///
/// Ngữ nghĩa sửa quyết định theo **kind** của lỗi (`IME_ASSIST_KIND_*`) bởi crate `assist`
/// (`assist/src/fix.rs`) thống nhất cho cả 3 nền tảng (R-7 / Q-2):
/// `duplicate_word` / `space_before_punct` ⇒ **xoá** span;
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
    glyphLen: (String) -> Int,
    explicit: Bool = false
) -> SentenceFixPlan? {
    // R2-131: `explicit` = người dùng CHỦ ĐỘNG (chọn gợi ý, phím tắt sửa câu) ⇒
    // gồm cả lỗi chính tả có gợi ý — nguồn chung FFI `ime_assist_plan_sentence_fix_ex`.
    let result = ImeEngine.encodeAssistResult(issues)
    guard let plan = ImeEngine.planSentenceFix(
        instance: nil, sentence: sentence, result: result, explicit: explicit
    ) else {
        return nil
    }
    let glyphs = glyphLen(plan.originalTail)
    guard glyphs > 0 else { return nil }

    return SentenceFixPlan(
        fixedCount: plan.fixedCount,
        deleteFrom: plan.deleteFrom,
        deleteChars: plan.deleteChars,
        deleteGlyphs: glyphs,
        replacement: plan.replacement,
        originalTail: plan.originalTail
    )
}
