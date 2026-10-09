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
        XCTAssertFalse(config.autostart)
        XCTAssertTrue(config.non_preedit)
        XCTAssertTrue(config.run_in_tray)
        XCTAssertEqual(config.switch_key, "ctrl_shift")
        XCTAssertTrue(config.macros.isEmpty)
    }

    /// Lưu từ bảng cài đặt macOS không được xoá `emoji[]` hay đổi `when: vi_on` của
    /// gõ tắt (Windows/Linux giữ nguyên các khoá này).
    func testSavePreservesEmojiAndMacroWhen() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_keep_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let fileURL = tempDir.appendingPathComponent("config.json")
        let json = """
        {"config_version":1,
         "macros":[{"trigger":"vn","expand":"Việt Nam","when":"vi_on"}],
         "emoji":[{"trigger":":ok","glyph":"👌"}]}
        """
        try json.data(using: .utf8)!.write(to: fileURL)

        var cfg = TextVNConfig.load(from: fileURL)
        XCTAssertEqual(cfg.macros.first?.when, "vi_on")
        XCTAssertEqual(cfg.emoji, [EmojiEntry(trigger: ":ok", glyph: "👌")])
        cfg.quick_telex = true
        try cfg.save(to: fileURL)

        let back = TextVNConfig.load(from: fileURL)
        XCTAssertTrue(back.quick_telex)
        XCTAssertEqual(back.macros, [MacroEntry(trigger: "vn", expand: "Việt Nam", when: "vi_on")])
        XCTAssertEqual(back.emoji, [EmojiEntry(trigger: ":ok", glyph: "👌")])
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

    func testAutostartPlistEscapesExecutablePath() {
        let path = "/Applications/Research & Development/<TextVN>.app/Contents/MacOS/TextVN"
        let plist = AutostartManager.generateLaunchAgentPlist(executablePath: path)
        XCTAssertTrue(plist.contains("Research &amp; Development/&lt;TextVN&gt;.app"))
        XCTAssertFalse(plist.contains("Research & Development"))
    }

    func testStartupDialogIsQuietForSMAppServiceAndLaunchAgent() {
        XCTAssertFalse(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN"], autostartEnabled: true, showDialogOnStartup: true
        ))
        XCTAssertFalse(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN", "--autostart"], autostartEnabled: false, showDialogOnStartup: true
        ))
        XCTAssertTrue(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN", "--settings"], autostartEnabled: true, showDialogOnStartup: true
        ))
        XCTAssertTrue(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN"], autostartEnabled: false, showDialogOnStartup: true
        ))
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

    // MARK: - 5. Review R3 regressions

    private func rawFrame(_ payload: Data) -> Data {
        var len = UInt32(payload.count).littleEndian
        var frame = Data(bytes: &len, count: 4)
        frame.append(payload)
        return frame
    }

    /// F3-3: JSON sai / thiếu `type` là violation → server phải đóng kết nối.
    func testParseFramesCheckedFlagsViolations() {
        guard let ping = IpcServer.encodeFrame(["type": "Ping"]) else {
            XCTFail("encode Ping")
            return
        }
        var badJSON = ping + rawFrame(Data("not json".utf8))
        let r1 = IpcServer.parseFramesChecked(from: &badJSON)
        XCTAssertTrue(r1.violation)
        XCTAssertEqual(r1.messages.count, 1, "frame hợp lệ đứng trước vẫn được giữ")
        XCTAssertTrue(badJSON.isEmpty)

        var noType = rawFrame(Data("{\"app_id\":\"x\"}".utf8))
        XCTAssertTrue(IpcServer.parseFramesChecked(from: &noType).violation)

        var partial = Data(ping.prefix(3))
        let r3 = IpcServer.parseFramesChecked(from: &partial)
        XCTAssertFalse(r3.violation, "frame chưa đủ byte không phải violation")
        XCTAssertTrue(r3.messages.isEmpty)
    }

    /// F3-12: Snapshot đúng bảng v1 đóng — không `uptime_ms`, không key rỗng.
    func testSnapshotMessageMatchesClosedSchema() {
        let msg = IpcServer.snapshotMessage(
            configVersion: 7, state: ["": true, "*": false, "com.apple.Safari": true])
        XCTAssertEqual(Set(msg.keys), ["type", "config_version", "state", "appdb_version", "channel"])
        let state = msg["state"] as? [String: Bool]
        XCTAssertEqual(state, ["*": false, "com.apple.Safari": true])
        XCTAssertEqual(msg["appdb_version"] as? Int, 1, "CR-34: appdb_version là số như textvn-ipc")
    }

    /// F3-1: một quy ước toàn cục "*" (IMK bản cũ gửi "" vẫn được hiểu).
    func testGlobalAppIDNormalization() {
        XCTAssertEqual(IpcServer.globalAppID, "*")
        XCTAssertEqual(IpcServer.normalizedAppID(""), "*")
        XCTAssertEqual(IpcServer.normalizedAppID("*"), "*")
        XCTAssertEqual(IpcServer.normalizedAppID("com.apple.TextEdit"), "com.apple.TextEdit")
    }

    /// F3-14: config hỏng được dời sang `.corrupt-<ts>`, không bị ghi đè mất.
    func testCorruptConfigIsQuarantinedNotOverwritten() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_corrupt_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let fileURL = tempDir.appendingPathComponent("config.json")
        try Data("{ \"macros\": [ broken".utf8).write(to: fileURL)

        let loaded = TextVNConfig.load(from: fileURL)
        XCTAssertEqual(loaded, TextVNConfig.default())
        XCTAssertFalse(FileManager.default.fileExists(atPath: fileURL.path))
        let names = try FileManager.default.contentsOfDirectory(atPath: tempDir.path)
        let backups = names.filter { $0.hasPrefix("config.json.corrupt-") }
        XCTAssertEqual(backups.count, 1)
        let kept = try String(contentsOf: tempDir.appendingPathComponent(backups[0]), encoding: .utf8)
        XCTAssertEqual(kept, "{ \"macros\": [ broken")
    }

    /// F3-14: thiếu khoá (config bản cũ/mới hơn) KHÔNG bị coi là hỏng.
    func testConfigWithMissingKeysUsesDefaults() throws {
        let data = Data("{ \"method\": \"vni\", \"macros\": [ { \"trigger\": \"vn\", \"expand\": \"Việt Nam\" } ] }".utf8)
        let cfg = try JSONDecoder().decode(TextVNConfig.self, from: data)
        XCTAssertEqual(cfg.method, "vni")
        XCTAssertEqual(cfg.macros.count, 1)
        XCTAssertEqual(cfg.output_charset, TextVNConfig.default().output_charset)
        XCTAssertThrowsError(try JSONDecoder().decode(TextVNConfig.self, from: Data("{ \"enabled\": \"yes\" }".utf8)))
    }

    /// F3-5: hot-reload áp bản hợp lệ, GIỮ bản đang dùng khi file sai schema.
    func testReloadFromDiskKeepsCurrentOnInvalidFile() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_reload_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let fileURL = tempDir.appendingPathComponent("config.json")
        let store = ConfigStore(url: fileURL)
        store.config.method = "vni"
        store.persist()
        XCTAssertFalse(store.reloadFromDisk(), "chính app vừa ghi → không đổi")

        var external = store.config
        external.method = "viqr"
        try external.save(to: fileURL)
        XCTAssertTrue(store.reloadFromDisk())
        XCTAssertEqual(store.config.method, "viqr")

        try Data("garbage".utf8).write(to: fileURL)
        XCTAssertFalse(store.reloadFromDisk())
        XCTAssertEqual(store.config.method, "viqr")
        XCTAssertTrue(FileManager.default.fileExists(atPath: fileURL.path), "hot-reload không dời file")
    }

    /// F3-2: cửa sổ host và view dùng chung một nguồn cỡ.
    func testSettingsContentSizeGrowsWhenExpanded() {
        let compact = SettingsView.contentSize(expanded: false)
        let expanded = SettingsView.contentSize(expanded: true)
        XCTAssertEqual(compact.width, expanded.width)
        XCTAssertGreaterThan(expanded.height, compact.height)
    }

    // MARK: - 6. F3-8 (badge/health/menu) + F3-13 (login launch)

    /// F3-13: điều kiện mở Cài đặt lúc launch — login launch (mọi nguồn cấu
    /// hình autostart) phải YÊN LẶNG; `--settings` luôn mở; tắt "hội thoại khởi
    /// động" thì không tự mở.
    func testStartupDialogDecisionMatrix() {
        XCTAssertFalse(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN"], autostartEnabled: true, showDialogOnStartup: true),
            "login launch → không mở Cài đặt")
        XCTAssertTrue(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN", "--settings"], autostartEnabled: true, showDialogOnStartup: false),
            "--settings luôn mở, kể cả khi autostart")
        XCTAssertFalse(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN"], autostartEnabled: false, showDialogOnStartup: false))
        XCTAssertTrue(AppDelegate.shouldShowSettingsOnLaunch(
            arguments: ["TextVN"], autostartEnabled: false, showDialogOnStartup: true))
    }

    /// F3-8: badge dùng SF Symbol template (không bitmap vẽ tay).
    func testBadgeSymbolSelection() {
        XCTAssertEqual(AppDelegate.badgeSymbolName(isVietnamese: true, hasError: false), "keyboard")
        XCTAssertEqual(AppDelegate.badgeSymbolName(isVietnamese: false, hasError: false), "keyboard.badge.ellipsis")
        XCTAssertEqual(AppDelegate.badgeSymbolName(isVietnamese: false, hasError: true), "exclamationmark.triangle")
    }

    /// F3-8: `app_id` menu bar PHẢI khớp `FieldDetect.normalizeAppID` của IMK.
    func testAppIDNormalizationMatchesIMKRule() {
        XCTAssertEqual(
            AppDelegate.normalizeAppID(bundleID: "Com.Apple.Safari", executableName: nil),
            "com.apple.safari")
        XCTAssertEqual(
            AppDelegate.normalizeAppID(bundleID: nil, executableName: "/usr/bin/login"), "login")
        XCTAssertEqual(AppDelegate.normalizeAppID(bundleID: nil, executableName: nil), "unknown")
    }

    /// F3-8: health đọc heartbeat do IMK ghi 5s/lần (P2-4 §6).
    func testHeartbeatSummaryReadsPidAndFlagsStale() throws {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_test_heartbeat_\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }
        let url = tempDir.appendingPathComponent("im-heartbeat.json")
        let now = Date(timeIntervalSince1970: 10_000)
        let fresh: [String: Any] = [
            "pid": 4242,
            "timestamp_ms": (now.timeIntervalSince1970 - 3) * 1000,
        ]
        try JSONSerialization.data(withJSONObject: fresh).write(to: url)
        let freshSummary = AppDelegate.heartbeatSummary(fileURL: url, now: now)
        XCTAssertTrue(freshSummary.contains("pid 4242"))
        XCTAssertFalse(freshSummary.contains("⚠️"))

        let stale: [String: Any] = [
            "pid": 7,
            "timestamp_ms": (now.timeIntervalSince1970 - 60) * 1000,
        ]
        try JSONSerialization.data(withJSONObject: stale).write(to: url)
        XCTAssertTrue(AppDelegate.heartbeatSummary(fileURL: url, now: now).contains("⚠️"))

        XCTAssertTrue(
            AppDelegate.heartbeatSummary(
                fileURL: tempDir.appendingPathComponent("missing.json"), now: now
            ).contains("chưa có"))
    }
}

// MARK: - R2-49 non_preedit mặc định · R2-50 quy tắc gõ tắt · R2-51 gỡ cài đặt

final class RoundTwoMacAppTests: XCTestCase {
    private func tempDir() throws -> URL {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("textvn_r2_\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        addTeardownBlock { try? FileManager.default.removeItem(at: dir) }
        return dir
    }

    /// Thiếu khoá → gạch chân (`false`), CÙNG quy ước với IMK `configNonPreedit`;
    /// config mới (mặc định) vẫn là `true` và được ghi tường minh.
    func testNonPreeditMissingKeyDecodesFalse() throws {
        let missing = try JSONDecoder().decode(TextVNConfig.self, from: Data(#"{"method":"vni"}"#.utf8))
        XCTAssertFalse(missing.non_preedit)
        let explicit = try JSONDecoder().decode(TextVNConfig.self, from: Data(#"{"non_preedit":true}"#.utf8))
        XCTAssertTrue(explicit.non_preedit)
        XCTAssertTrue(TextVNConfig.default().non_preedit)
        let saved = try JSONEncoder().encode(TextVNConfig.default())
        let obj = try XCTUnwrap(try JSONSerialization.jsonObject(with: saved) as? [String: Any])
        XCTAssertEqual(obj["non_preedit"] as? Bool, true, "bản lưu luôn ghi khoá tường minh")
    }

    func testPersistIfMissingWritesDefaultsOnce() throws {
        let url = try tempDir().appendingPathComponent("config.json")
        let store = ConfigStore(url: url)
        XCTAssertTrue(store.persistIfMissing())
        let obj = try XCTUnwrap(
            try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any])
        XCTAssertEqual(obj["non_preedit"] as? Bool, true)
        try Data(#"{"non_preedit":false}"#.utf8).write(to: url)
        XCTAssertFalse(store.persistIfMissing(), "file đã có → không ghi đè")
        XCTAssertEqual(try Data(contentsOf: url), Data(#"{"non_preedit":false}"#.utf8))
    }

    func testMacroRulesMatchWindowsLinux() {
        XCTAssertNil(MacroRules.validate(trigger: "vn", expand: "Việt Nam"))
        XCTAssertEqual(MacroRules.validate(trigger: "", expand: "x"), .emptyTrigger)
        XCTAssertEqual(MacroRules.validate(trigger: "a b", expand: "x"), .triggerHasSpace)
        XCTAssertEqual(MacroRules.validate(trigger: String(repeating: "a", count: 33), expand: "x"), .triggerTooLong)
        XCTAssertNil(MacroRules.validate(trigger: String(repeating: "a", count: 32), expand: "x"))
        XCTAssertEqual(MacroRules.validate(trigger: "dc", expand: ""), .emptyExpansion)
        // 71 ký tự — bản cũ lưu được rồi engine cắt còn 64 khi gõ.
        let address = "Số 12 đường Nguyễn Thị Minh Khai, Phường Đa Kao, Quận 1, TP Hồ Chí Minh"
        XCTAssertEqual(MacroRules.validate(trigger: "dc", expand: address), .expansionTooLong)
        // Đếm theo Unicode scalar như Rust `chars()`: 64 chữ có dấu dựng sẵn vẫn hợp lệ.
        XCTAssertNil(MacroRules.validate(trigger: "x", expand: String(repeating: "ệ", count: 64)))
        XCTAssertEqual(MacroRules.Problem.expansionTooLong.message, "nội dung dài quá 64 ký tự")
    }

    func testMacroUpsertReplacesCaseInsensitivelyAndKeepsWhen() {
        let macros = [
            MacroEntry(trigger: "VN", expand: "Việt Nam", when: "vi_on"),
            MacroEntry(trigger: "cty", expand: "Công ty"),
        ]
        let out = MacroRules.upsert(MacroEntry(trigger: "vn", expand: "Việt Nam!"), into: macros)
        XCTAssertEqual(out.count, 2)
        XCTAssertEqual(out.last, MacroEntry(trigger: "vn", expand: "Việt Nam!", when: "vi_on"))
        XCTAssertFalse(out.contains { $0.trigger == "VN" })
    }

    func testAdminUninstallTargets() {
        let uid: uid_t = 501
        let owners: [String: uid_t] = [
            "/Applications/TextVN.app": 0, // pkg "cài cho mọi người dùng" → root
            "/Library/Input Methods/TextVN-IM.app": 501,
        ]
        let targets = AppDelegate.adminUninstallTargets(
            paths: AppDelegate.systemScopeBundles, uid: uid,
            owner: { owners[$0] },
            parentWritable: { $0.hasPrefix("/Applications") }
        )
        // TextVN.app thuộc root; TextVN-IM.app thuộc user nhưng /Library/Input Methods
        // không ghi được → cả hai cần admin.
        XCTAssertEqual(targets, AppDelegate.systemScopeBundles)
        let userOwned = AppDelegate.adminUninstallTargets(
            paths: ["/Applications/TextVN.app", "/Library/Input Methods/TextVN-IM.app"], uid: uid,
            owner: { $0 == "/Applications/TextVN.app" ? uid : nil },
            parentWritable: { _ in true }
        )
        XCTAssertTrue(userOwned.isEmpty, "bundle của chính user + IM không tồn tại → không xin admin")
    }

    func testAdminUninstallCommandQuoting() {
        XCTAssertNil(AppDelegate.adminUninstallCommand(targets: [], forgetSystemReceipt: false))
        let command = AppDelegate.adminUninstallCommand(
            targets: ["/Library/Input Methods/TextVN-IM.app"], forgetSystemReceipt: true)
        XCTAssertEqual(
            command,
            "/bin/rm -rf '/Library/Input Methods/TextVN-IM.app' && { /usr/sbin/pkgutil --forget vn.textvn.pkg >/dev/null 2>&1 || true; }"
        )
        XCTAssertEqual(AppDelegate.shellQuote("it's"), #"'it'\''s'"#)
        XCTAssertEqual(
            AppDelegate.adminAppleScript(command: #"echo "a\b""#),
            #"do shell script "echo \"a\\b\"" with administrator privileges"#
        )
    }

    func testStageUninstallScriptsCopiesCheckOutOfBundle() throws {
        let resources = try tempDir()
        try Data("#!/bin/bash\n".utf8).write(to: resources.appendingPathComponent("uninstall_macos.sh"))
        try Data("#!/bin/bash\n".utf8).write(to: resources.appendingPathComponent("uninstall-check.sh"))
        let staged = try XCTUnwrap(AppDelegate.stageUninstallScripts(resources: resources.path))
        addTeardownBlock { try? FileManager.default.removeItem(at: staged) }
        XCTAssertTrue(FileManager.default.fileExists(atPath: staged.appendingPathComponent("uninstall_macos.sh").path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: staged.appendingPathComponent("uninstall-check.sh").path))
        XCTAssertNil(AppDelegate.stageUninstallScripts(resources: resources.appendingPathComponent("none").path))
        XCTAssertEqual(AppDelegate.logTail("a\nb\n\nc\n", lines: 2), "b\nc")
    }
}
