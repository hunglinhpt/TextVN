// SPDX-License-Identifier: GPL-3.0-or-later
// ConfigModel.swift — TextVN configuration model for macOS (P0-3 §1, schemas/config.v1)

import Foundation

public struct MacroEntry: Codable, Identifiable, Equatable {
    public var id: String { trigger }
    public var trigger: String
    public var expand: String

    public init(trigger: String, expand: String) {
        self.trigger = trigger
        self.expand = expand
    }
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
    public var non_preedit: Bool
    public var run_in_tray: Bool
    public var switch_key: String
    public var macros: [MacroEntry]
    /// Từ điển EN của người dùng (một từ/dòng trong UI) — engine restore các từ
    /// này bất kể fold có trùng âm tiết Việt thông dụng (xem
    /// docs/specs/language-detection.md). THIẾU trường này từng khiến bản macOS
    /// XOÁ `english_words` mỗi lần lưu config (bắt khi đồng bộ 0.2.13).
    public var english_words: [String]

    public init(
        config_version: Int = 1,
        enabled: Bool = true,
        method: String = "telex",
        diacritic_style: String = "new",
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
        english_words: [String] = []
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
        non_preedit = try c.decodeIfPresent(Bool.self, forKey: .non_preedit) ?? d.non_preedit
        run_in_tray = try c.decodeIfPresent(Bool.self, forKey: .run_in_tray) ?? d.run_in_tray
        switch_key = try c.decodeIfPresent(String.self, forKey: .switch_key) ?? d.switch_key
        macros = try c.decodeIfPresent([MacroEntry].self, forKey: .macros) ?? d.macros
        english_words = try c.decodeIfPresent([String].self, forKey: .english_words) ?? d.english_words
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

    public func resetToDefaults() {
        self.config = .default()
        persist()
    }
}
