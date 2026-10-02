// SPDX-License-Identifier: GPL-3.0-or-later
// AppDelegate.swift — macOS Menu Bar App & Status Item Controller (P2-4 §1, PLAN §3.6)

import Cocoa
import SwiftUI

public final class AppDelegate: NSObject, NSApplicationDelegate, IpcServerDelegate, NSMenuDelegate {
    public static let shared = AppDelegate()

    private var statusItem: NSStatusItem?
    private var settingsWindow: NSWindow?
    private let configStore = ConfigStore.shared
    private let ipcServer = IpcServer.shared
    private let autostartManager = AutostartManager.shared
    private var configWatcher: ConfigWatcher?
    private var workspaceObserver: NSObjectProtocol?
    /// App foreground gần nhất (không phải TextVN) — dùng cho mục per-app (F3-8).
    private var lastFrontAppID: String?
    private var lastFrontAppName: String = "app hiện tại"
    /// Số crash report IMK gửi qua IPC → badge `error` (P2-4 §1).
    private var crashCount: UInt32 = 0

    private var isVietnameseMode: Bool = true {
        didSet {
            updateStatusIcon()
            updateMenuState()
        }
    }

    public override init() {
        super.init()
    }

    public func applicationDidFinishLaunching(_ notification: Notification) {
        let isSettings = CommandLine.arguments.contains("--settings")
        // F3-13: SMAppService KHÔNG truyền `--autostart` và trạng thái SM có thể
        // chỉ là `.requiresApproval` (bản chưa ký) ngay khi login item vẫn chạy →
        // không thể chỉ dựa `isAutostartEnabled()`. Thêm ý định trong config
        // (`autostart: true`) để login launch LUÔN yên lặng, không mở Cài đặt.
        let isLoginLaunch = CommandLine.arguments.contains("--autostart")
            || autostartManager.isAutostartConfigured()
            || configStore.config.autostart
        NSLog("[TextVN] launch: settings=%d loginLaunch=%d showDialogOnStartup=%d",
              isSettings ? 1 : 0, isLoginLaunch ? 1 : 0,
              configStore.config.show_dialog_on_startup ? 1 : 0)
        // 1. Single Instance Check (P2-4 §1)
        if isAnotherInstanceRunning() {
            // SMAppService does not forward --autostart. A duplicate login launch
            // must not open Settings in the instance already running.
            if isSettings || !isLoginLaunch {
                DistributedNotificationCenter.default().postNotificationName(
                    Notification.Name("vn.textvn.awake"),
                    object: nil,
                    userInfo: nil,
                    deliverImmediately: true
                )
            }
            NSApp.terminate(nil)
            return
        }

        DistributedNotificationCenter.default().addObserver(
            self,
            selector: #selector(handleAwakeNotification),
            name: Notification.Name("vn.textvn.awake"),
            object: nil
        )

        // Nhớ app foreground gần nhất (bỏ chính TextVN) cho mục per-app (F3-8).
        refreshForegroundApp()
        workspaceObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            guard let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            else { return }
            self?.rememberForegroundApp(app)
        }

        // 2. Start IPC Server (P2-4 §2)
        ipcServer.delegate = self
        do {
            try ipcServer.start()
        } catch {
            NSLog("[TextVN] Failed to start IPC server: %@", error.localizedDescription)
        }

        // Hot-reload config.json sửa ngoài UI → broadcast tới IMK (MAC-053; review R3 F3-5).
        let watcher = ConfigWatcher(fileURL: TextVNConfig.defaultConfigURL()) { [weak self] in
            self?.handleConfigFileChanged()
        }
        watcher.start()
        configWatcher = watcher

        // 3. Initialize Status Item & Menu
        setupStatusItem()

        // 4. Initial Launch Behavior
        if Self.shouldShowSettingsOnLaunch(
            arguments: CommandLine.arguments,
            autostartEnabled: isLoginLaunch,
            showDialogOnStartup: configStore.config.show_dialog_on_startup
        ) {
            showSettingsWindow()
        }
    }

    /// SMAppService launches the main app without custom arguments. When login
    /// startup is enabled, keep that path quiet; --settings remains explicit.
    public static func shouldShowSettingsOnLaunch(
        arguments: [String],
        autostartEnabled: Bool,
        showDialogOnStartup: Bool
    ) -> Bool {
        arguments.contains("--settings")
            || (showDialogOnStartup && !autostartEnabled && !arguments.contains("--autostart"))
    }

    public func applicationWillTerminate(_ notification: Notification) {
        if let workspaceObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(workspaceObserver)
        }
        configWatcher?.stop()
        ipcServer.stop()
    }

    /// config.json đổi trên đĩa (đã debounce). File sai schema / chính app vừa ghi
    /// → `reloadFromDisk()` trả false, không broadcast.
    private func handleConfigFileChanged() {
        let wasEnabled = configStore.config.enabled
        guard configStore.reloadFromDisk() else { return }
        let ver = UInt64(Date().timeIntervalSince1970)
        if configStore.config.enabled != wasEnabled {
            isVietnameseMode = configStore.config.enabled // didSet dựng lại menu
            ipcServer.broadcastStateUpdate(appID: IpcServer.globalAppID, enabled: isVietnameseMode, version: ver)
        } else {
            rebuildMenu()
        }
        ipcServer.broadcastConfigReload(version: ver)
    }

    // MARK: - Single Instance Enforcement

    private func isAnotherInstanceRunning() -> Bool {
        let bundleID = Bundle.main.bundleIdentifier ?? "vn.textvn.app"
        let running = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        let currentPID = ProcessInfo.processInfo.processIdentifier
        return running.contains { $0.processIdentifier != currentPID }
    }

    @objc private func handleAwakeNotification() {
        DispatchQueue.main.async { [weak self] in
            self?.showSettingsWindow()
        }
    }

    // MARK: - Status Item & Badge Drawing

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        isVietnameseMode = configStore.config.enabled
        updateStatusIcon()
        let menu = NSMenu(title: "TextVN")
        menu.delegate = self // menuNeedsUpdate → làm mới mục "app đang gõ" (F3-8)
        statusItem?.menu = menu
        rebuildMenu()
    }

    /// Tên SF Symbol template theo trạng thái (P2-4 §1): badge `vn-on`/`vn-off`/`error`.
    /// Dùng symbol template (không vẽ bitmap) → tự hợp theme sáng/tối, không lệch màu.
    static func badgeSymbolName(isVietnamese: Bool, hasError: Bool) -> String {
        if hasError { return "exclamationmark.triangle" }
        return isVietnamese ? "keyboard" : "keyboard.badge.ellipsis"
    }

    private func updateStatusIcon() {
        guard let button = statusItem?.button else { return }
        let symbol = Self.badgeSymbolName(isVietnamese: isVietnameseMode, hasError: crashCount > 0)
        // Tên symbol lạ (macOS cũ hơn) → fallback `keyboard` để icon không biến mất.
        let image = NSImage(systemSymbolName: symbol, accessibilityDescription: nil)
            ?? NSImage(systemSymbolName: "keyboard", accessibilityDescription: nil)
        image?.isTemplate = true
        button.image = image
        button.imagePosition = .imageLeft
        button.title = isVietnameseMode ? " [V]" : " [E]"
        // Template chỉ đổi màu qua contentTintColor (không phải bitmap vẽ tay).
        if crashCount > 0 {
            button.contentTintColor = .systemOrange
        } else {
            button.contentTintColor = isVietnameseMode ? .controlAccentColor : .secondaryLabelColor
        }
        button.toolTip = "TextVN - Bộ gõ tiếng Việt (\(isVietnameseMode ? "Tiếng Việt [V]" : "Tiếng Anh [E]"))"
    }

    // MARK: - Context Menu (P2-4 §1 — menu bar)

    /// Menu mở lại mỗi lần bấm → mục "app đang gõ" luôn đúng app foreground
    /// (F3-8). Không dựng lại `NSMenu` mới (statusItem giữ nguyên instance).
    public func menuNeedsUpdate(_ menu: NSMenu) {
        refreshForegroundApp()
        rebuildMenu()
    }

    private func rebuildMenu() {
        guard let menu = statusItem?.menu else { return }
        menu.removeAllItems()
        populate(menu)
    }

    /// Menu 10 mục (P2-4 §1, mirror P1-4 §1): thêm submenu "Dấu" + mục
    /// "Bật tiếng Việt cho <app>" (per-app — F3-8).
    private func populate(_ menu: NSMenu) {
        // 1. Bật/Tắt tiếng Việt
        let toggleItem = NSMenuItem(
            title: isVietnameseMode ? "✓ Tiếng Việt" : "  Tiếng Anh",
            action: #selector(toggleVietnameseMode),
            keyEquivalent: ""
        )
        toggleItem.target = self
        menu.addItem(toggleItem)
        menu.addItem(NSMenuItem.separator())

        // 2. Chế độ gõ (Submenu)
        let methodItem = NSMenuItem(title: "Chế độ gõ", action: nil, keyEquivalent: "")
        let methodMenu = NSMenu(title: "Chế độ gõ")
        let methods = [
            ("Telex", "telex"),
            ("VNI", "vni"),
            ("VIQR", "viqr"),
            ("Microsoft (Simple)", "simple_telex")
        ]
        for (name, id) in methods {
            let mItem = NSMenuItem(title: name, action: #selector(selectMethod(_:)), keyEquivalent: "")
            mItem.target = self
            mItem.representedObject = id
            mItem.state = (configStore.config.method == id) ? .on : .off
            methodMenu.addItem(mItem)
        }
        methodItem.submenu = methodMenu
        menu.addItem(methodItem)

        // 3. Bảng mã (Submenu)
        let charsetItem = NSMenuItem(title: "Bảng mã", action: nil, keyEquivalent: "")
        let charsetMenu = NSMenu(title: "Bảng mã")
        let charsets = [
            ("1. Unicode dựng sẵn", "unicode_precomposed"),
            ("2. TCVN3 (ABC)", "tcvn3"),
            ("3. VNI Windows", "vni_windows"),
            ("4. Unicode tổ hợp", "unicode_decomposed")
        ]
        for (name, id) in charsets {
            let cItem = NSMenuItem(title: name, action: #selector(selectCharset(_:)), keyEquivalent: "")
            cItem.target = self
            cItem.representedObject = id
            cItem.state = (configStore.config.output_charset == id) ? .on : .off
            charsetMenu.addItem(cItem)
        }
        charsetItem.submenu = charsetMenu
        menu.addItem(charsetItem)

        // 4. Dấu (Submenu) — nhãn khớp ui-spec §2 (Dấu mới/Dấu cũ — F3-8).
        let diacriticItem = NSMenuItem(title: "Dấu", action: nil, keyEquivalent: "")
        let diacriticMenu = NSMenu(title: "Dấu")
        let diacritics = [
            ("Dấu mới (hoà, thuỷ)", "new"),
            ("Dấu cũ (hòa, thủy)", "old")
        ]
        for (name, id) in diacritics {
            let dItem = NSMenuItem(title: name, action: #selector(selectDiacritic(_:)), keyEquivalent: "")
            dItem.target = self
            dItem.representedObject = id
            dItem.state = (configStore.config.diacritic_style == id) ? .on : .off
            diacriticMenu.addItem(dItem)
        }
        diacriticItem.submenu = diacriticMenu
        menu.addItem(diacriticItem)
        menu.addItem(NSMenuItem.separator())

        // 5. Quyền Accessibility
        let axTrusted = AXIsProcessTrusted()
        let axItem = NSMenuItem(
            title: axTrusted ? "✓ Quyền Accessibility (Đã cấp)" : "⚠️ Yêu cầu quyền Accessibility...",
            action: #selector(checkAccessibilityPermission),
            keyEquivalent: ""
        )
        axItem.target = self
        menu.addItem(axItem)

        // 6. App đang gõ + bật/tắt riêng cho app đó (state per-app — F3-8).
        let appItem = NSMenuItem(
            title: "Bật tiếng Việt cho \(lastFrontAppName)",
            action: #selector(toggleFrontApp),
            keyEquivalent: ""
        )
        appItem.target = self
        appItem.representedObject = lastFrontAppID ?? ""
        appItem.state = frontAppEnabled ? .on : .off
        appItem.isEnabled = lastFrontAppID != nil
        menu.addItem(appItem)

        // 7. Cài đặt...
        let settingsItem = NSMenuItem(title: "Cài đặt...", action: #selector(openSettings), keyEquivalent: ",")
        settingsItem.target = self
        menu.addItem(settingsItem)

        // 8. Sức khỏe
        let healthItem = NSMenuItem(title: "Sức khỏe hệ thống...", action: #selector(showHealth), keyEquivalent: "")
        healthItem.target = self
        menu.addItem(healthItem)
        menu.addItem(NSMenuItem.separator())

        // 9. Gỡ cài đặt...
        let uninstallItem = NSMenuItem(title: "Gỡ cài đặt TextVN...", action: #selector(promptUninstall), keyEquivalent: "")
        uninstallItem.target = self
        menu.addItem(uninstallItem)

        // 10. Thông tin
        let aboutItem = NSMenuItem(title: "Thông tin TextVN", action: #selector(showAbout), keyEquivalent: "")
        aboutItem.target = self
        menu.addItem(aboutItem)
        menu.addItem(NSMenuItem.separator())

        // 11. Thoát
        let quitItem = NSMenuItem(title: "Thoát TextVN", action: #selector(quitApp), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)
    }

    /// Trạng thái hiệu dụng của app foreground: override per-app (nếu có) →
    /// mặc định theo toggle toàn cục (mirror IMK: `appStates[app] ?? viEnabled`).
    private var frontAppEnabled: Bool {
        guard let appID = lastFrontAppID else { return isVietnameseMode }
        return ipcServer.appState(for: appID) ?? isVietnameseMode
    }

    /// Nhớ app foreground gần nhất KHÔNG phải TextVN. Lúc người dùng bấm status
    /// item, app này là foreground — cùng vấn đề Windows tray gặp với taskbar
    /// (R3-6), nên phải nhớ từ notification `didActivateApplication`.
    func rememberForegroundApp(_ app: NSRunningApplication) {
        let ownBundleID = Bundle.main.bundleIdentifier ?? "vn.textvn.app"
        if app.bundleIdentifier == ownBundleID { return }
        let id = Self.normalizeAppID(
            bundleID: app.bundleIdentifier,
            executableName: app.executableURL?.lastPathComponent
        )
        guard id != "unknown" else { return }
        lastFrontAppID = id
        lastFrontAppName = app.localizedName ?? id
    }

    /// Cùng quy tắc `FieldDetect.normalizeAppID` (IMK — P2-3 §1): bundle id
    /// lowercase, fallback executable basename. `app_id` 2 phía PHẢI khớp nhau.
    static func normalizeAppID(bundleID: String?, executableName: String?) -> String {
        if let bundleID, !bundleID.isEmpty {
            return bundleID.lowercased()
        }
        if let executableName, !executableName.isEmpty {
            return (executableName as NSString).lastPathComponent.lowercased()
        }
        return "unknown"
    }

    private func refreshForegroundApp() {
        if let app = NSWorkspace.shared.frontmostApplication {
            rememberForegroundApp(app)
        }
    }

    private func updateMenuState() {
        rebuildMenu()
    }

    // MARK: - Actions

    @objc public func toggleVietnameseMode() {
        isVietnameseMode.toggle()
        configStore.config.enabled = isVietnameseMode
        configStore.persist()
        let ver = UInt64(Date().timeIntervalSince1970)
        // "*" = toàn cục, IMK nay nhận đúng quy ước này (review R3 F3-1).
        ipcServer.broadcastStateUpdate(appID: IpcServer.globalAppID, enabled: isVietnameseMode, version: ver)
        ipcServer.broadcastConfigReload(version: ver)
    }

    @objc private func selectMethod(_ sender: NSMenuItem) {
        if let id = sender.representedObject as? String {
            configStore.config.method = id
            configStore.persist()
            ipcServer.broadcastConfigReload(version: UInt64(Date().timeIntervalSince1970))
            updateMenuState()
        }
    }

    @objc private func selectCharset(_ sender: NSMenuItem) {
        if let id = sender.representedObject as? String {
            configStore.config.output_charset = id
            configStore.persist()
            ipcServer.broadcastConfigReload(version: UInt64(Date().timeIntervalSince1970))
            updateMenuState()
        }
    }

    @objc private func selectDiacritic(_ sender: NSMenuItem) {
        if let id = sender.representedObject as? String {
            configStore.config.diacritic_style = id
            configStore.persist()
            ipcServer.broadcastConfigReload(version: UInt64(Date().timeIntervalSince1970))
            updateMenuState()
        }
    }

    /// Bật/tắt gõ tiếng Việt cho RIÊNG app đang gõ (state per-app — F3-8).
    /// Trạng thái giữ trong bộ nhớ IPC server (macOS chưa có `state.json` per-app
    /// — parity-checklist ghi rõ); IMK áp theo `app_id` qua `StateUpdate`.
    @objc private func toggleFrontApp(_ sender: NSMenuItem) {
        guard let appID = sender.representedObject as? String, !appID.isEmpty else { return }
        let newValue = !frontAppEnabled
        ipcServer.broadcastStateUpdate(
            appID: appID, enabled: newValue, version: UInt64(Date().timeIntervalSince1970))
        updateMenuState()
    }

    @objc private func checkAccessibilityPermission() {
        if !AXIsProcessTrusted() {
            let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
            NSWorkspace.shared.open(url)
        } else {
            let alert = NSAlert()
            alert.messageText = "Quyền Accessibility"
            alert.informativeText = "TextVN đã được cấp đủ quyền Accessibility để tự động nhận diện ô nhập liệu an toàn."
            alert.alertStyle = .informational
            alert.runModal()
        }
    }

    @objc public func openSettings() {
        showSettingsWindow()
    }

    public func showSettingsWindow() {
        if settingsWindow == nil {
            let view = SettingsView(
                store: configStore,
                onClose: { [weak self] in self?.settingsWindow?.close() },
                onExpandedChange: { [weak self] expanded in self?.resizeSettingsWindow(expanded: expanded) }
            )
            let hosting = NSHostingView(rootView: view)
            let initial = SettingsView.contentSize(expanded: false)
            let win = NSWindow(
                contentRect: NSRect(origin: .zero, size: initial),
                styleMask: [.titled, .closable, .miniaturizable],
                backing: .buffered,
                defer: false
            )
            win.center()
            win.title = "TextVN - Bảng điều khiển"
            win.contentView = hosting
            win.isReleasedWhenClosed = false
            settingsWindow = win
        }

        settingsWindow?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    /// Đổi cỡ cửa sổ theo Mở rộng/Thu nhỏ, giữ nguyên mép trên (review R3 F3-2).
    private func resizeSettingsWindow(expanded: Bool) {
        guard let win = settingsWindow else { return }
        let content = NSRect(origin: .zero, size: SettingsView.contentSize(expanded: expanded))
        var frame = win.frameRect(forContentRect: content)
        frame.origin.x = win.frame.origin.x
        frame.origin.y = win.frame.maxY - frame.height
        win.setFrame(frame, display: true, animate: true)
    }

    @objc private func showHealth() {
        let alert = NSAlert()
        alert.messageText = "Sức khỏe hệ thống TextVN"
        let imkPids = Self.imkProcessIds(from: NSWorkspace.shared.runningApplications)
        let heartbeatURL = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/TextVN/im-heartbeat.json")
        let status = """
        • Phiên bản: \(AppInfo.displayVersion)
        • IPC Socket: \(IpcServer.defaultSocketURL().path)
        • Trạng thái IPC: \(ipcServer.isRunning ? "Đang chạy (Online)" : "Chưa kích hoạt")
        • Số client kết nối: \(ipcServer.connectedClientsCount)
        • IMK (vn.textvn.im): \(imkPids.isEmpty ? "không thấy tiến trình" : imkPids.map { String($0) }.joined(separator: ", "))
        • Heartbeat IMK: \(Self.heartbeatSummary(fileURL: heartbeatURL))
        • Quyền Accessibility: \(AXIsProcessTrusted() ? "Đã cấp" : "Chưa cấp")
        • Tự khởi động cùng OS: \(autostartManager.isAutostartEnabled() ? "Đã bật" : "Tắt")
        """
        alert.informativeText = status
        alert.alertStyle = .informational
        alert.runModal()
    }

    /// PID các tiến trình TextVN-IM đang chạy (bundle `vn.textvn.im`) — P2-4 §6.
    /// Best-effort: macOS có thể không liệt kê IMK trong `runningApplications`
    /// (tiến trình do `imklaunchagent` spawn) → trả rỗng thay vì crash.
    static func imkProcessIds(from apps: [NSRunningApplication]) -> [pid_t] {
        apps.filter { $0.bundleIdentifier == "vn.textvn.im" }
            .map { $0.processIdentifier }
            .sorted()
    }

    /// Tóm tắt heartbeat file do TextVN-IM ghi 5s/lần (P2-4 §6).
    static func heartbeatSummary(fileURL: URL, now: Date = Date()) -> String {
        guard let data = try? Data(contentsOf: fileURL),
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
              let pid = obj["pid"] as? Int,
              let ts = obj["timestamp_ms"] as? Double else {
            return "chưa có (IMK chưa chạy hoặc bản cũ)"
        }
        let age = max(0, now.timeIntervalSince1970 * 1000 - ts) / 1000
        return String(format: "pid %d, cập nhật %.0fs trước%@", pid, age, age > 15 ? " ⚠️" : "")
    }

    @objc private func promptUninstall() {
        let alert = NSAlert()
        alert.messageText = "Gỡ cài đặt TextVN?"
        alert.informativeText = "Thao tác này sẽ gỡ bỏ TextVN khỏi hệ thống.\n\nTheo quy tắc S9, cấu hình người dùng sẽ được bảo toàn nguyên vẹn trừ khi bạn chỉ định xóa."
        alert.addButton(withTitle: "Hủy")
        alert.addButton(withTitle: "Gỡ cài đặt")
        alert.alertStyle = .warning

        if alert.runModal() == .alertSecondButtonReturn {
            // Login Item qua SMAppService (BTM) không bị script xoá plist gỡ theo —
            // phải unregister từ chính app trước khi xoá bundle (review R3 F3-7a).
            try? autostartManager.setAutostart(enabled: false)
            let scriptPath = Bundle.main.bundlePath + "/Contents/Resources/uninstall_macos.sh"
            if FileManager.default.fileExists(atPath: scriptPath) {
                let process = Process()
                process.executableURL = URL(fileURLWithPath: "/bin/bash")
                process.arguments = [scriptPath]
                try? process.run()
            }
            NSApp.terminate(nil)
        }
    }

    @objc private func showAbout() {
        let alert = NSAlert()
        alert.messageText = "TextVN \(AppInfo.displayVersion)"
        alert.informativeText = "Bộ gõ tiếng Việt cho macOS.\nPhát triển bởi LinhBH.CoM.\nBản quyền © 2026 LinhBH.CoM.\nGiấy phép: GNU General Public License v3."
        alert.alertStyle = .informational
        alert.runModal()
    }

    @objc public func quitApp() {
        ipcServer.stop()
        NSApp.terminate(nil)
    }

    // MARK: - IpcServerDelegate

    public func ipcServer(_ server: IpcServer, didToggleViEn appID: String, enabled: Bool) {
        // IpcServer đã chuẩn hoá "" (IMK bản cũ) về "*" — một quy ước (review R3 F3-1).
        if appID == IpcServer.globalAppID {
            self.isVietnameseMode = enabled
            self.configStore.config.enabled = enabled
            self.configStore.persist()
            // Đồng bộ `opts.enabled` của MỌI IMK (engine đọc config). Thiếu bước
            // này, bật VN bằng Ctrl+Shift+Space sau khi config tắt sẽ không gõ
            // được (IMK chỉ đổi `ctx.enabled`, engine còn gate theo config).
            server.broadcastConfigReload(version: UInt64(Date().timeIntervalSince1970))
        }
    }

    public func ipcServer(_ server: IpcServer, didReceiveCrashReport code: UInt32, count: UInt32) {
        NSLog("[TextVN] Received crash report: code=%u count=%u", code, count)
        crashCount += 1
        updateStatusIcon()
    }
}
