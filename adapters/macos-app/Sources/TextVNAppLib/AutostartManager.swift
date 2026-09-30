// SPDX-License-Identifier: GPL-3.0-or-later
// AutostartManager.swift — Cross-platform Autostart for macOS (Rule S5, P2-4 §1)

import Foundation
import ServiceManagement

public final class AutostartManager {
    public static let shared = AutostartManager()
    public static let label = "vn.textvn.app"
    public static let plistFileName = "vn.textvn.app.plist"

    public init() {}

    public static func defaultLaunchAgentsDirectory() -> URL {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return home.appendingPathComponent("Library/LaunchAgents", isDirectory: true)
    }

    public static func generateLaunchAgentPlist(executablePath: String) -> String {
        let escapedPath = executablePath
            .replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
            .replacingOccurrences(of: "\"", with: "&quot;")
            .replacingOccurrences(of: "'", with: "&apos;")
        return """
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>Label</key>
            <string>\(label)</string>
            <key>ProgramArguments</key>
            <array>
                <string>\(escapedPath)</string>
                <string>--autostart</string>
            </array>
            <key>RunAtLoad</key>
            <true/>
            <key>ProcessType</key>
            <string>Interactive</string>
        </dict>
        </plist>
        """
    }

    // MARK: - LaunchAgent Helpers (Dir-agnostic for tests & fallback)

    public static func isLaunchAgentEnabled(inDir dir: URL) -> Bool {
        let plistURL = dir.appendingPathComponent(plistFileName)
        return FileManager.default.fileExists(atPath: plistURL.path)
    }

    public static func enableLaunchAgent(inDir dir: URL, executablePath: String) throws {
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true, attributes: [
            .posixPermissions: 0o700
        ])
        let plistURL = dir.appendingPathComponent(plistFileName)
        let content = generateLaunchAgentPlist(executablePath: executablePath)
        try content.write(to: plistURL, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: plistURL.path)
    }

    public static func disableLaunchAgent(inDir dir: URL) throws {
        let plistURL = dir.appendingPathComponent(plistFileName)
        if FileManager.default.fileExists(atPath: plistURL.path) {
            try FileManager.default.removeItem(at: plistURL)
        }
    }

    // MARK: - Public Autostart API

    public func isAutostartEnabled() -> Bool {
        if #available(macOS 13.0, *) {
            let status = SMAppService.mainApp.status
            if status == .enabled {
                return true
            }
        }
        return Self.isLaunchAgentEnabled(inDir: Self.defaultLaunchAgentsDirectory())
    }

    /// "Autostart đã được cấu hình" — rộng hơn `isAutostartEnabled()`: tính cả
    /// `.requiresApproval` (login item đã đăng ký nhưng người dùng chưa bấm Allow
    /// trong System Settings; bản chưa ký hay gặp). App dùng cờ này để giữ YÊN
    /// LẶNG khi macOS tự khởi động lúc login, tránh mở cửa sổ Cài đặt sai
    /// (F3-13). Không dùng cho UI "Đã bật/Tắt" (ở đó `isAutostartEnabled()`).
    public func isAutostartConfigured() -> Bool {
        if #available(macOS 13.0, *) {
            let status = SMAppService.mainApp.status
            if status == .enabled || status == .requiresApproval {
                return true
            }
        }
        return Self.isLaunchAgentEnabled(inDir: Self.defaultLaunchAgentsDirectory())
    }

    public func setAutostart(enabled: Bool, executablePath: String? = nil) throws {
        let exe = executablePath ?? Bundle.main.executablePath ?? "/Applications/TextVN.app/Contents/MacOS/TextVN"
        let agents = Self.defaultLaunchAgentsDirectory()

        if enabled {
            var smEnabled = false
            if #available(macOS 13.0, *) {
                do {
                    if SMAppService.mainApp.status != .enabled {
                        try SMAppService.mainApp.register()
                    }
                    smEnabled = SMAppService.mainApp.status == .enabled
                } catch {
                    NSLog("[TextVN] SMAppService.register failed (%@) — dùng LaunchAgent fallback",
                          error.localizedDescription)
                }
            }
            if smEnabled {
                // SM đã nhận → plist fallback cũ là nguồn thứ 2, gỡ để khỏi chạy đôi.
                try? Self.disableLaunchAgent(inDir: agents)
            } else {
                // SM chưa dùng được (macOS 11–12, chưa ký, hoặc đang chờ duyệt) →
                // plist fallback là nguồn DUY NHẤT. Trước đây nhánh này bị bỏ quên
                // nên "bật autostart" xong restart máy không có gì chạy (F3-13).
                try Self.enableLaunchAgent(inDir: agents, executablePath: exe)
            }
            return
        }

        // Tắt phải gỡ CẢ 2 nguồn: plist fallback (đăng ký lúc register() từng
        // throw) vẫn tồn tại khi SM báo `.notRegistered` (review R3 F3-6); trạng
        // thái `.requiresApproval` cũng phải unregister được.
        if #available(macOS 13.0, *) {
            if SMAppService.mainApp.status != .notRegistered {
                do {
                    try SMAppService.mainApp.unregister()
                } catch {
                    NSLog("[TextVN] SMAppService.unregister failed (%@) — vẫn gỡ plist",
                          error.localizedDescription)
                }
            }
        }
        try Self.disableLaunchAgent(inDir: agents)
    }
}
