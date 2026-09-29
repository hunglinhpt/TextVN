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
    public var output_charset: String
    public var show_dialog_on_startup: Bool
    public var autostart: Bool
    public var non_preedit: Bool
    public var run_in_tray: Bool
    public var switch_key: String
    public var macros: [MacroEntry]

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
        output_charset: String = "unicode_precomposed",
        show_dialog_on_startup: Bool = true,
        autostart: Bool = true,
        non_preedit: Bool = true,
        run_in_tray: Bool = true,
        switch_key: String = "ctrl_shift",
        macros: [MacroEntry] = []
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
        self.output_charset = output_charset
        self.show_dialog_on_startup = show_dialog_on_startup
        self.autostart = autostart
        self.non_preedit = non_preedit
        self.run_in_tray = run_in_tray
        self.switch_key = switch_key
        self.macros = macros
    }

    public static func `default`() -> TextVNConfig {
        TextVNConfig()
    }

    public static func defaultConfigURL() -> URL {
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let textvnDir = appSupport.appendingPathComponent("TextVN", isDirectory: true)
        return textvnDir.appendingPathComponent("config.json")
    }

    public static func load(from url: URL? = nil) -> TextVNConfig {
        let fileURL = url ?? defaultConfigURL()
        guard let data = try? Data(contentsOf: fileURL) else {
            return .default()
        }
        do {
            let decoder = JSONDecoder()
            return try decoder.decode(TextVNConfig.self, from: data)
        } catch {
            return .default()
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
