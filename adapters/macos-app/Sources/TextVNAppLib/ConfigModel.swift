// SPDX-License-Identifier: GPL-3.0-or-later
// ConfigModel.swift — TextVN configuration model for macOS (P0-3 §1, schemas/config.v1)

import Foundation

public struct MacroEntry: Codable, Identifiable, Equatable {
    public var id: String { trigger }
    public var trigger: String
    public var expand: String
    /// `always` | `vi_on` (P0-3 §1.1) — `nil` = không ghi (engine hiểu `always`).
    /// Thiếu trường này thì mỗi lần lưu từ bảng cài đặt macOS biến gõ tắt
    /// `vi_on` (chỉ chạy khi bật tiếng Việt) thành `always`.
    public var when: String?

    public init(trigger: String, expand: String, when: String? = nil) {
        self.trigger = trigger
        self.expand = expand
        self.when = when
    }
}

/// R2-50: cùng quy tắc với `config/src/macro_text.rs` (Windows/Linux kiểm khi bấm
/// Lưu) — bảng gõ tắt macOS từng chỉ trim rồi thêm, nên nội dung > 64 ký tự bị
/// engine cắt im lặng khi gõ (`MAX_TEXT`), trigger có khoảng trắng không bao giờ
/// khớp, `VN`/`vn` cùng tồn tại mà chỉ một cái chạy. Độ dài đếm theo Unicode scalar
/// (= `char` của Rust).
public enum MacroRules {
    public static let triggerMax = 32
    public static let expandMax = 64

    public enum Problem: Equatable {
        case emptyTrigger
        case triggerHasSpace
        case triggerTooLong
        case emptyExpansion
        case expansionTooLong

        /// Cùng câu chữ `MacroLineErrorKind::message_vi` (Windows/Linux).
        public var message: String {
            switch self {
            case .emptyTrigger: return "chưa có chữ gõ tắt"
            case .triggerHasSpace: return "chữ gõ tắt không được chứa khoảng trắng"
            case .triggerTooLong: return "chữ gõ tắt dài quá \(MacroRules.triggerMax) ký tự"
            case .emptyExpansion: return "chưa có nội dung"
            case .expansionTooLong: return "nội dung dài quá \(MacroRules.expandMax) ký tự"
            }
        }
    }

    /// Kiểm một mục (đã trim). `nil` = hợp lệ.
    public static func validate(trigger: String, expand: String) -> Problem? {
        if trigger.isEmpty { return .emptyTrigger }
        if trigger.unicodeScalars.contains(where: { $0.properties.isWhitespace }) {
            return .triggerHasSpace
        }
        if trigger.unicodeScalars.count > triggerMax { return .triggerTooLong }
        if expand.isEmpty { return .emptyExpansion }
        if expand.unicodeScalars.count > expandMax { return .expansionTooLong }
        return nil
    }

    /// Thêm/thay mục: trigger trùng KHÔNG phân biệt hoa/thường (engine khớp như vậy)
    /// thay chỗ mục cũ và giữ `when` của nó (như `macro_text::parse`).
    public static func upsert(_ entry: MacroEntry, into macros: [MacroEntry]) -> [MacroEntry] {
        let key = entry.trigger.lowercased()
        var out = macros
        var added = entry
        if let old = out.first(where: { $0.trigger.lowercased() == key }), added.when == nil {
            added.when = old.when
        }
        out.removeAll { $0.trigger.lowercased() == key }
        out.append(added)
        return out
    }
}

/// `config.emoji[]` (P0-3 §1.1) — giữ nguyên khi lưu (UI macOS chưa sửa emoji).
public struct EmojiEntry: Codable, Equatable {
    public var trigger: String
    public var glyph: String

    public init(trigger: String, glyph: String) {
        self.trigger = trigger
        self.glyph = glyph
    }
}

/// `assist.display` — điểm HIỂN THỊ tách khỏi điểm KIỂM TRA (P4-0 §7, D11).
/// Khớp 1-1 với crate `config` (`config::AssistDisplay`) và
/// `schemas/config.v1.schema.json`: cùng tên trường, cùng mặc định — JSON là
/// hợp đồng chung 3 nền tảng, không đổi một bên mà không đối chiếu bên kia.
public struct AssistDisplay: Codable, Equatable {
    /// Hiện gợi ý khi gặp dấu kết thúc câu (`.` `!` `?`) — mặc định `true`.
    public var show_on_sentence_end: Bool
    /// Ngừng gõ bao lâu (ms) thì hiện gợi ý — mặc định `700`.
    public var idle_ms: Int
    /// Hiện gợi ý ngay khi gõ dấu phẩy — mặc định `false` (tránh spam UI).
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

    /// Decode khoan dung: thiếu trường lấy mặc định (cùng quy ước `TextVNConfig` —
    /// config bản cũ/mới hơn ghi thiếu vẫn đọc được); sai KIỂU vẫn throw.
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

/// `assist` — Trợ lý ngữ pháp (P4-0 §7). Mặc định TẮT HOÀN TOÀN (D20 kill-switch):
/// thiếu khối này (config bản cũ) hay thiếu trường = tắt, không tự sửa, không hiện gì.
/// Khớp 1-1 với crate `config` (`config::AssistSettings`) và schema — THIẾU struct
/// này từng là mẫu mất dữ liệu: model chưa biết khoá thì mỗi lần lưu từ bảng cài
/// đặt macOS âm thầm xoá khoá đó của người dùng (đã gặp với `english_words`,
/// `emoji` — xem chú thích 0.2.13 các trường bên dưới).
public struct AssistSettings: Codable, Equatable {
    /// `off` | `highlight` | `autocorrect` — mức can thiệp, mặc định `off` (D20).
    public var mode: String
    /// `off` | `safe` — chỉ luật objective mới được tự sửa (D7), mặc định `off`.
    public var auto_fix: String
    /// `vi` | `en` | `auto` — v1 chỉ hỗ trợ `vi` (D9), mặc định `vi`.
    public var language: String
    /// Phím kiểm tra im lặng, mặc định `["space", ",", "."]` (P4-0 §7).
    public var trigger_keys: [String]
    /// Điểm hiển thị (D11).
    public var display: AssistDisplay
    /// Độ dài tối đa của một câu được kiểm tra (ký tự Unicode), mặc định `500`.
    public var max_sentence_chars: Int
    /// Bật/tắt riêng theo app (`app_id` lowercase → bool). Thiếu = dùng chung.
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

    /// Decode khoan dung: thiếu trường lấy mặc định (cùng quy ước `TextVNConfig`).
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

    /// Danh sách phím kiểm tra hợp lệ — CÙNG danh sách với `config::ASSIST_TRIGGER_KEYS`
    /// và enum của schema (P4-0 §4.1).
    public static let validTriggerKeys: [String] = ["space", ",", ".", "!", "?", ";", ":"]

    /// Cùng nghĩa với `AssistSettings::is_on()` (Rust): `off` = kill-switch tắt hẳn.
    public var isOn: Bool { mode != "off" }
}

public struct TextVNConfig: Codable, Equatable {
    public var config_version: Int
    public var enabled: Bool
    public var method: String
    public var diacritic_style: String
    public var free_marking: Bool
    public var auto_restore_english: Bool
    public var auto_capitalize: Bool
    public var macro_trigger: String
    public var allow_macro_when_vi_off: Bool
    /// Quick Telex (cc→ch, nn→ng…) — cùng khoá `config.quick_telex` như Windows/Linux.
    public var quick_telex: Bool
    public var output_charset: String
    public var show_dialog_on_startup: Bool
    public var autostart: Bool
    /// Gõ không gạch chân. Bản cài mới/lưu từ Cài đặt luôn ghi khoá này (mặc định
    /// `true`); config THIẾU khoá (bản cũ, sửa tay) → `false` = gạch chân — cùng
    /// quy ước với IMK (`configNonPreedit`) và CHANGELOG 0.2.27 (R2-49).
    public var non_preedit: Bool
    public var run_in_tray: Bool
    /// R2-106: KHÔNG còn hiển thị/thực thi — phím chuyển cố định Ctrl+Shift như
    /// Windows/Linux. Giữ trường để config.json cũ đọc/ghi lại không mất khoá.
    public var switch_key: String
    public var macros: [MacroEntry]
    /// Từ điển EN của người dùng (một từ/dòng trong UI) — engine restore các từ
    /// này bất kể fold có trùng âm tiết Việt thông dụng (xem
    /// docs/specs/language-detection.md). THIẾU trường này từng khiến bản macOS
    /// XOÁ `english_words` mỗi lần lưu config (bắt khi đồng bộ 0.2.13).
    public var english_words: [String]
    /// Gõ tắt emoji — thiếu trường này thì mỗi lần lưu config XOÁ `emoji[]` của
    /// người dùng (tray Windows/Linux giữ nguyên khoá qua SettingsDoc).
    public var emoji: [EmojiEntry]
    /// Trợ lý ngữ pháp (P4-0 §7) — mặc định tắt hoàn toàn (D20). Model PHẢI biết
    /// khoá này (giống `english_words`/`emoji`) nếu không lần lưu nào từ app cũng
    /// xoá cài đặt assist của người dùng.
    public var assist: AssistSettings

    public init(
        config_version: Int = 1,
        enabled: Bool = true,
        method: String = "telex",
        diacritic_style: String = "old",
        free_marking: Bool = true,
        auto_restore_english: Bool = true,
        auto_capitalize: Bool = true,
        macro_trigger: String = "tab",
        allow_macro_when_vi_off: Bool = false,
        quick_telex: Bool = false,
        output_charset: String = "unicode_precomposed",
        show_dialog_on_startup: Bool = true,
        autostart: Bool = false,
        non_preedit: Bool = true,
        run_in_tray: Bool = true,
        switch_key: String = "ctrl_shift",
        macros: [MacroEntry] = [],
        english_words: [String] = [],
        emoji: [EmojiEntry] = [],
        assist: AssistSettings = AssistSettings()
    ) {
        self.config_version = config_version
        self.enabled = enabled
        self.method = method
        self.diacritic_style = diacritic_style
        self.free_marking = free_marking
        self.auto_restore_english = auto_restore_english
        self.auto_capitalize = auto_capitalize
        self.macro_trigger = macro_trigger
        self.allow_macro_when_vi_off = allow_macro_when_vi_off
        self.quick_telex = quick_telex
        self.output_charset = output_charset
        self.show_dialog_on_startup = show_dialog_on_startup
        self.autostart = autostart
        self.non_preedit = non_preedit
        self.run_in_tray = run_in_tray
        self.switch_key = switch_key
        self.macros = macros
        self.english_words = english_words
        self.emoji = emoji
        self.assist = assist
    }

    /// Decode khoan dung: khoá THIẾU lấy mặc định (config do bản cũ/mới hơn ghi
    /// không bị coi là hỏng); khoá sai KIỂU vẫn throw → `.corrupt` (review R3 F3-14).
    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let d = TextVNConfig()
        config_version = try c.decodeIfPresent(Int.self, forKey: .config_version) ?? d.config_version
        enabled = try c.decodeIfPresent(Bool.self, forKey: .enabled) ?? d.enabled
        method = try c.decodeIfPresent(String.self, forKey: .method) ?? d.method
        diacritic_style = try c.decodeIfPresent(String.self, forKey: .diacritic_style) ?? d.diacritic_style
        free_marking = try c.decodeIfPresent(Bool.self, forKey: .free_marking) ?? d.free_marking
        auto_restore_english = try c.decodeIfPresent(Bool.self, forKey: .auto_restore_english) ?? d.auto_restore_english
        auto_capitalize = try c.decodeIfPresent(Bool.self, forKey: .auto_capitalize) ?? d.auto_capitalize
        macro_trigger = try c.decodeIfPresent(String.self, forKey: .macro_trigger) ?? d.macro_trigger
        allow_macro_when_vi_off = try c.decodeIfPresent(Bool.self, forKey: .allow_macro_when_vi_off) ?? d.allow_macro_when_vi_off
        quick_telex = try c.decodeIfPresent(Bool.self, forKey: .quick_telex) ?? d.quick_telex
        output_charset = try c.decodeIfPresent(String.self, forKey: .output_charset) ?? d.output_charset
        show_dialog_on_startup = try c.decodeIfPresent(Bool.self, forKey: .show_dialog_on_startup) ?? d.show_dialog_on_startup
        autostart = try c.decodeIfPresent(Bool.self, forKey: .autostart) ?? d.autostart
        // KHÔNG lấy d.non_preedit (true): thiếu khoá = gạch chân, như IMK (R2-49).
        non_preedit = try c.decodeIfPresent(Bool.self, forKey: .non_preedit) ?? false
        run_in_tray = try c.decodeIfPresent(Bool.self, forKey: .run_in_tray) ?? d.run_in_tray
        switch_key = try c.decodeIfPresent(String.self, forKey: .switch_key) ?? d.switch_key
        macros = try c.decodeIfPresent([MacroEntry].self, forKey: .macros) ?? d.macros
        english_words = try c.decodeIfPresent([String].self, forKey: .english_words) ?? d.english_words
        emoji = try c.decodeIfPresent([EmojiEntry].self, forKey: .emoji) ?? d.emoji
        assist = try c.decodeIfPresent(AssistSettings.self, forKey: .assist) ?? d.assist
    }

    public static func `default`() -> TextVNConfig {
        TextVNConfig()
    }

    public enum LoadError: Error {
        /// File chưa tồn tại / không đọc được.
        case missing
        /// File có nhưng không phải config hợp lệ.
        case corrupt
    }

    public static func defaultConfigURL() -> URL {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let textvnDir = appSupport.appendingPathComponent("TextVN", isDirectory: true)
        return textvnDir.appendingPathComponent("config.json")
    }

    /// Đọc thuần — không đụng file. Dùng cho hot-reload (MAC-053): lỗi → caller giữ
    /// bản đang dùng.
    public static func read(from url: URL? = nil) -> Result<TextVNConfig, LoadError> {
        let fileURL = url ?? defaultConfigURL()
        guard let data = try? Data(contentsOf: fileURL) else { return .failure(.missing) }
        guard let cfg = try? JSONDecoder().decode(TextVNConfig.self, from: data) else {
            return .failure(.corrupt)
        }
        return .success(cfg)
    }

    /// Nạp lúc khởi động. File hỏng → dời sang `config.json.corrupt-<ts>` rồi dùng
    /// mặc định: trước đây bản lỗi nằm yên tới lần persist kế và bị ghi đè → mất
    /// macro của người dùng (review R3 F3-14). Giống `.bak` bên tray Windows.
    public static func load(from url: URL? = nil) -> TextVNConfig {
        let fileURL = url ?? defaultConfigURL()
        switch read(from: fileURL) {
        case let .success(cfg):
            return cfg
        case .failure(.missing):
            return .default()
        case .failure(.corrupt):
            let backup = quarantineCorrupt(fileURL)
            NSLog("[TextVN] config.json hỏng — đã giữ bản lỗi ở %@, dùng mặc định",
                  backup?.path ?? "(không dời được)")
            return .default()
        }
    }

    /// Dời file hỏng sang tên `<file>.corrupt-<unix-ts>`; trả đường dẫn mới.
    @discardableResult
    static func quarantineCorrupt(_ fileURL: URL) -> URL? {
        let ts = Int(Date().timeIntervalSince1970)
        let dst = fileURL.deletingLastPathComponent()
            .appendingPathComponent("\(fileURL.lastPathComponent).corrupt-\(ts)")
        do {
            try FileManager.default.moveItem(at: fileURL, to: dst)
            return dst
        } catch {
            return nil
        }
    }

    public func save(to url: URL? = nil) throws {
        let fileURL = url ?? Self.defaultConfigURL()
        let dir = fileURL.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
            .posixPermissions: 0o700
        ])
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(self)
        try data.write(to: fileURL, options: .atomic)
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: fileURL.path)
    }
}

public final class ConfigStore: ObservableObject {
    public static let shared = ConfigStore()

    @Published public var config: TextVNConfig {
        didSet {
            onConfigChanged?(config)
        }
    }

    public var onConfigChanged: ((TextVNConfig) -> Void)?
    private let customURL: URL?

    public init(url: URL? = nil) {
        self.customURL = url
        self.config = TextVNConfig.load(from: url)
    }

    public func reload() {
        self.config = TextVNConfig.load(from: customURL)
    }

    /// Hot-reload (MAC-053, P2-4 §2): file hợp lệ và KHÁC bản đang dùng → áp dụng,
    /// trả `true`. File thiếu/sai schema → GIỮ bản đang dùng, không dời file (người
    /// dùng có thể đang sửa dở). Chính `persist()` ghi ra → bằng nhau → `false`.
    @discardableResult
    public func reloadFromDisk() -> Bool {
        guard case let .success(cfg) = TextVNConfig.read(from: customURL), cfg != config else {
            return false
        }
        config = cfg
        return true
    }

    public func persist() {
        do {
            try config.save(to: customURL)
        } catch {
            NSLog("[TextVN] Failed to save config: %@", error.localizedDescription)
        }
    }

    /// Lần chạy đầu (bản cài .pkg không tạo config.json): ghi mặc định ra đĩa để
    /// IMK đọc đúng giá trị Cài đặt đang hiển thị (vd `non_preedit: true`) thay vì
    /// mặc định "thiếu khoá" của nó (R2-49). File đã có → không đụng.
    @discardableResult
    public func persistIfMissing() -> Bool {
        let fileURL = customURL ?? TextVNConfig.defaultConfigURL()
        guard !FileManager.default.fileExists(atPath: fileURL.path) else { return false }
        persist()
        return FileManager.default.fileExists(atPath: fileURL.path)
    }

    public func resetToDefaults() {
        // Vòng 16 (audit B5): nút "Mặc định" từng xoá CẢ macros/english_words/
        // emoji và tắt autostart một cách âm thầm (vi phạm S9 — dữ liệu người
        // dùng phải được bảo toàn). Chỉ reset các tuỳ chọn gõ.
        let keepMacros = config.macros
        let keepWords = config.english_words
        let keepEmoji = config.emoji
        let keepAutostart = config.autostart
        self.config = .default()
        self.config.macros = keepMacros
        self.config.english_words = keepWords
        self.config.emoji = keepEmoji
        self.config.autostart = keepAutostart
        persist()
    }
}
