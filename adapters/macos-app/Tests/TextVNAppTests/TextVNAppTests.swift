// SPDX-License-Identifier: GPL-3.0-or-later
// TextVNAppTests.swift — Unit tests for macOS Menu Bar & Settings App

import XCTest
@testable import TextVNAppLib

final class TextVNAppTests: XCTestCase {

    // MARK: - 1. Config Defaults & Serialization

    func testDefaultConfig() {
        let config = TextVNConfig.default()
        XCTAssertEqual(config.config_version, 1)
        XCTAssertTrue(config.enabled)
        XCTAssertEqual(config.method, "telex")
        XCTAssertEqual(config.diacritic_style, "new")
        XCTAssertTrue(config.free_marking)
        XCTAssertTrue(config.auto_restore_english)
        XCTAssertTrue(config.auto_capitalize)
        XCTAssertEqual(config.macro_trigger, "tab")
        XCTAssertFalse(config.allow_macro_when_vi_off)
        XCTAssertEqual(config.output_charset, "unicode_precomposed")
        XCTAssertTrue(config.show_dialog_on_startup)
        XCTAssertTrue(config.autostart)
        XCTAssertTrue(config.non_preedit)
        XCTAssertTrue(config.run_in_tray)
        XCTAssertEqual(config.switch_key, "ctrl_shift")
        XCTAssertTrue(config.macros.isEmpty)
    }

    func testConfigSaveAndLoadRoundtrip() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_config_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }

        let fileURL = tempDir.appendingPathComponent("config.json")

        var config = TextVNConfig()
        config.method = "vni"
        config.output_charset = "tcvn3"
        config.diacritic_style = "old"
        config.free_marking = false
        config.autostart = false
        config.show_dialog_on_startup = false
        config.switch_key = "alt_z"
        config.macros = [
            MacroEntry(trigger: "vn", expand: "Việt Nam"),
            MacroEntry(trigger: "tvn", expand: "TextVN")
        ]

        try config.save(to: fileURL)

        let loaded = TextVNConfig.load(from: fileURL)
        XCTAssertEqual(loaded.method, "vni")
        XCTAssertEqual(loaded.output_charset, "tcvn3")
        XCTAssertEqual(loaded.diacritic_style, "old")
        XCTAssertFalse(loaded.free_marking)
        XCTAssertFalse(loaded.autostart)
        XCTAssertFalse(loaded.show_dialog_on_startup)
        XCTAssertEqual(loaded.switch_key, "alt_z")
        XCTAssertEqual(loaded.macros.count, 2)
        XCTAssertEqual(loaded.macros[0].trigger, "vn")
        XCTAssertEqual(loaded.macros[0].expand, "Việt Nam")
    }

    // MARK: - 2. Autostart & LaunchAgent (Rule S5)

    func testAutostartPlistGeneration() {
        let fakePath = "/Applications/TextVN.app/Contents/MacOS/TextVN"
        let plist = AutostartManager.generateLaunchAgentPlist(executablePath: fakePath)

        XCTAssertTrue(plist.contains("<string>vn.textvn.app</string>"))
        XCTAssertTrue(plist.contains("<string>/Applications/TextVN.app/Contents/MacOS/TextVN</string>"))
        XCTAssertTrue(plist.contains("<string>--autostart</string>"))
        XCTAssertTrue(plist.contains("<key>RunAtLoad</key>"))
        XCTAssertTrue(plist.contains("<true/>"))
        XCTAssertTrue(plist.contains("<key>ProcessType</key>"))
        XCTAssertTrue(plist.contains("<string>Interactive</string>"))
    }

    func testLaunchAgentLifecycleInDir() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_launchagent_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }

        // Initially not enabled
        XCTAssertFalse(AutostartManager.isLaunchAgentEnabled(inDir: tempDir))

        // Enable
        let fakeExe = "/Applications/TextVN.app/Contents/MacOS/TextVN"
        try AutostartManager.enableLaunchAgent(inDir: tempDir, executablePath: fakeExe)
        XCTAssertTrue(AutostartManager.isLaunchAgentEnabled(inDir: tempDir))

        // File permissions check (0600)
        let plistPath = tempDir.appendingPathComponent(AutostartManager.plistFileName).path
        let attrs = try FileManager.default.attributesOfItem(atPath: plistPath)
        let posixPerms = (attrs[.posixPermissions] as? NSNumber)?.intValue
        XCTAssertEqual(posixPerms, 0o600)

        // Disable
        try AutostartManager.disableLaunchAgent(inDir: tempDir)
        XCTAssertFalse(AutostartManager.isLaunchAgentEnabled(inDir: tempDir))
    }

    // MARK: - 3. IPC Framing & Codec (schemas/ipc.v1.md)

    func testIpcFrameEncodingAndParsing() {
        let originalMsg: [String: Any] = [
            "type": "ConfigReload",
            "version": 42
        ]

        guard let frame = IpcServer.encodeFrame(originalMsg) else {
            XCTFail("Failed to encode frame")
            return
        }

        // Check 4-byte header length
        XCTAssertGreaterThan(frame.count, 4)
        let length = frame.prefix(4).withUnsafeBytes { $0.loadUnaligned(fromByteOffset: 0, as: UInt32.self) }.littleEndian
        XCTAssertEqual(Int(length), frame.count - 4)

        // Parse single frame
        var buffer = frame
        let parsed = IpcServer.parseFrames(from: &buffer)
        XCTAssertEqual(parsed.count, 1)
        XCTAssertEqual(parsed[0]["type"] as? String, "ConfigReload")
        XCTAssertEqual(parsed[0]["version"] as? Int, 42)
        XCTAssertTrue(buffer.isEmpty)
    }

    func testIpcMultipleStreamingFrames() {
        let msg1: [String: Any] = ["type": "Ping"]
        let msg2: [String: Any] = ["type": "ToggleViEn", "app_id": "com.apple.Safari", "enabled": false]

        guard let frame1 = IpcServer.encodeFrame(msg1),
              let frame2 = IpcServer.encodeFrame(msg2) else {
            XCTFail("Failed to encode frames")
            return
        }

        // Concatenate into single stream
        var streamBuffer = frame1 + frame2
        let parsed = IpcServer.parseFrames(from: &streamBuffer)

        XCTAssertEqual(parsed.count, 2)
        XCTAssertEqual(parsed[0]["type"] as? String, "Ping")
        XCTAssertEqual(parsed[1]["type"] as? String, "ToggleViEn")
        XCTAssertEqual(parsed[1]["app_id"] as? String, "com.apple.Safari")
        XCTAssertEqual(parsed[1]["enabled"] as? Bool, false)
        XCTAssertTrue(streamBuffer.isEmpty)
    }

    func testIpcFrameProtocolViolation() {
        // Construct malformed frame with invalid length (> 65536)
        var badFrame = Data()
        var hugeLen: UInt32 = 100_000
        badFrame.append(Data(bytes: &hugeLen, count: 4))
        badFrame.append(Data([0x7B, 0x7D])) // {}

        var buffer = badFrame
        let parsed = IpcServer.parseFrames(from: &buffer)
        XCTAssertTrue(parsed.isEmpty)
        XCTAssertTrue(buffer.isEmpty) // Buffer cleared on violation
    }

    // MARK: - 4. ConfigStore & Macro State

    func testConfigStoreResetToDefaults() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_store_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }

        let fileURL = tempDir.appendingPathComponent("config.json")
        let store = ConfigStore(url: fileURL)

        store.config.method = "viqr"
        store.config.output_charset = "vni_windows"
        store.persist()

        XCTAssertEqual(store.config.method, "viqr")

        store.resetToDefaults()
        XCTAssertEqual(store.config.method, "telex")
        XCTAssertEqual(store.config.output_charset, "unicode_precomposed")
    }
}
