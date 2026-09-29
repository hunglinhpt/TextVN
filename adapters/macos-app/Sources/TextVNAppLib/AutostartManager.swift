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
        """
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>Label</key>
            <string>\(label)</string>
            <key>ProgramArguments</key>
            <array>
                <string>\(executablePath)</string>
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

    public func setAutostart(enabled: Bool, executablePath: String? = nil) throws {
        let exe = executablePath ?? Bundle.main.executablePath ?? "/Applications/TextVN.app/Contents/MacOS/TextVN"

        if #available(macOS 13.0, *) {
            do {
                if enabled {
                    if SMAppService.mainApp.status != .enabled {
                        try SMAppService.mainApp.register()
                    }
                } else {
                    if SMAppService.mainApp.status == .enabled {
                        try SMAppService.mainApp.unregister()
                    }
                }
            } catch {
                NSLog("[TextVN] SMAppService failed (%@), using LaunchAgent fallback", error.localizedDescription)
                if enabled {
                    try Self.enableLaunchAgent(inDir: Self.defaultLaunchAgentsDirectory(), executablePath: exe)
                } else {
                    try Self.disableLaunchAgent(inDir: Self.defaultLaunchAgentsDirectory())
                }
            }
        } else {
            if enabled {
                try Self.enableLaunchAgent(inDir: Self.defaultLaunchAgentsDirectory(), executablePath: exe)
            } else {
                try Self.disableLaunchAgent(inDir: Self.defaultLaunchAgentsDirectory())
            }
        }
    }
}
