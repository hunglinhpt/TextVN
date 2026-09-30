// SPDX-License-Identifier: GPL-3.0-or-later
// AppDelegate.swift — macOS Menu Bar App & Status Item Controller (P2-4 §1, PLAN §3.6)

import Cocoa
import SwiftUI

public final class AppDelegate: NSObject, NSApplicationDelegate, IpcServerDelegate {
    public static let shared = AppDelegate()

    private var statusItem: NSStatusItem?
    private var settingsWindow: NSWindow?
    private let configStore = ConfigStore.shared
    private let ipcServer = IpcServer.shared
    private let autostartManager = AutostartManager.shared

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
        // 1. Single Instance Check (P2-4 §1)
        if isAnotherInstanceRunning() {
            DistributedNotificationCenter.default().postNotificationName(
                Notification.Name("vn.textvn.awake"),
                object: nil,
                userInfo: nil,
                deliverImmediately: true
            )
            NSApp.terminate(nil)
            return
        }

        DistributedNotificationCenter.default().addObserver(
            self,
            selector: #selector(handleAwakeNotification),
            name: Notification.Name("vn.textvn.awake"),
            object: nil
        )

        // 2. Start IPC Server (P2-4 §2)
        ipcServer.delegate = self
        do {
            try ipcServer.start()
        } catch {
            NSLog("[TextVN] Failed to start IPC server: %@", error.localizedDescription)
        }

        // 3. Initialize Status Item & Menu
        setupStatusItem()

        // 4. Initial Launch Behavior
        let isAutostart = CommandLine.arguments.contains("--autostart")
        let isSettings = CommandLine.arguments.contains("--settings")

        if isSettings || (!isAutostart && configStore.config.show_dialog_on_startup) {
            showSettingsWindow()
        }
    }

    public func applicationWillTerminate(_ notification: Notification) {
        ipcServer.stop()
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
        buildMenu()
    }

    private func updateStatusIcon() {
        guard let button = statusItem?.button else { return }

        let size = NSSize(width: 18, height: 18)
        let image = NSImage(size: size, flipped: false) { rect in
            let letter = self.isVietnameseMode ? "V" : "E"
            let badgeColor: NSColor = self.isVietnameseMode
                ? NSColor(calibratedRed: 0.65, green: 0.12, blue: 0.65, alpha: 1.0) // Crimson / Purple
                : NSColor(calibratedRed: 0.10, green: 0.45, blue: 0.85, alpha: 1.0) // Blue

            // Background pill
            let bgPath = NSBezierPath(roundedRect: rect.insetBy(dx: 1, dy: 1), xRadius: 4, yRadius: 4)
            badgeColor.setFill()
            bgPath.fill()

            // Centered Letter
            let font = NSFont.systemFont(ofSize: 11, weight: .bold)
            let attrs: [NSAttributedString.Key: Any] = [
                .font: font,
                .foregroundColor: NSColor.white
            ]
            let str = NSAttributedString(string: letter, attributes: attrs)
            let strSize = str.size()
            let strRect = NSRect(
                x: (rect.width - strSize.width) / 2.0,
                y: (rect.height - strSize.height) / 2.0,
                width: strSize.width,
                height: strSize.height
            )
            str.draw(in: strRect)

            return true
        }

        image.isTemplate = false
        button.image = image
        button.toolTip = "TextVN - Bộ gõ tiếng Việt (\(isVietnameseMode ? "Tiếng Việt" : "Tiếng Anh"))"
    }

    // MARK: - Context Menu (9 Standard Items)

    private func buildMenu() {
        let menu = NSMenu(title: "TextVN")

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
        menu.addItem(NSMenuItem.separator())

        // 4. Quyền Accessibility
        let axTrusted = AXIsProcessTrusted()
        let axItem = NSMenuItem(
            title: axTrusted ? "✓ Quyền Accessibility (Đã cấp)" : "⚠️ Yêu cầu quyền Accessibility...",
            action: #selector(checkAccessibilityPermission),
            keyEquivalent: ""
        )
        axItem.target = self
        menu.addItem(axItem)

        // 5. Cài đặt...
        let settingsItem = NSMenuItem(title: "Cài đặt...", action: #selector(openSettings), keyEquivalent: ",")
        settingsItem.target = self
        menu.addItem(settingsItem)

        // 6. Sức khỏe
        let healthItem = NSMenuItem(title: "Sức khỏe hệ thống...", action: #selector(showHealth), keyEquivalent: "")
        healthItem.target = self
        menu.addItem(healthItem)
        menu.addItem(NSMenuItem.separator())

        // 7. Gỡ cài đặt...
        let uninstallItem = NSMenuItem(title: "Gỡ cài đặt TextVN...", action: #selector(promptUninstall), keyEquivalent: "")
        uninstallItem.target = self
        menu.addItem(uninstallItem)

        // 8. Thông tin
        let aboutItem = NSMenuItem(title: "Thông tin TextVN", action: #selector(showAbout), keyEquivalent: "")
        aboutItem.target = self
        menu.addItem(aboutItem)
        menu.addItem(NSMenuItem.separator())

        // 9. Thoát
        let quitItem = NSMenuItem(title: "Thoát TextVN", action: #selector(quitApp), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)

        statusItem?.menu = menu
    }

    private func updateMenuState() {
        buildMenu()
    }

    // MARK: - Actions

    @objc public func toggleVietnameseMode() {
        isVietnameseMode.toggle()
        configStore.config.enabled = isVietnameseMode
        configStore.persist()
        let ver = UInt64(Date().timeIntervalSince1970)
        ipcServer.broadcastStateUpdate(appID: "*", enabled: isVietnameseMode, version: ver)
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
            let view = SettingsView(store: configStore) { [weak self] in
                self?.settingsWindow?.close()
            }
            let hosting = NSHostingView(rootView: view)
            let win = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 505, height: 245),
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

    @objc private func showHealth() {
        let alert = NSAlert()
        alert.messageText = "Sức khỏe hệ thống TextVN"
        let status = """
        • Phiên bản: \(AppInfo.displayVersion)
        • IPC Socket: \(IpcServer.defaultSocketURL().path)
        • Trạng thái IPC: \(ipcServer.isRunning ? "Đang chạy (Online)" : "Chưa kích hoạt")
        • Số client kết nối: \(ipcServer.connectedClientsCount)
        • Quyền Accessibility: \(AXIsProcessTrusted() ? "Đã cấp" : "Chưa cấp")
        • Tự khởi động cùng OS: \(autostartManager.isAutostartEnabled() ? "Đã bật" : "Tắt")
        """
        alert.informativeText = status
        alert.alertStyle = .informational
        alert.runModal()
    }

    @objc private func promptUninstall() {
        let alert = NSAlert()
        alert.messageText = "Gỡ cài đặt TextVN?"
        alert.informativeText = "Thao tác này sẽ gỡ bỏ TextVN khỏi hệ thống.\n\nTheo quy tắc S9, cấu hình người dùng sẽ được bảo toàn nguyên vẹn trừ khi bạn chỉ định xóa."
        alert.addButton(withTitle: "Hủy")
        alert.addButton(withTitle: "Gỡ cài đặt")
        alert.alertStyle = .warning

        if alert.runModal() == .alertSecondButtonReturn {
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
        alert.informativeText = "Bộ gõ tiếng Việt cho macOS.\nPhát triển bởi hunglinhpt.\nBản quyền © 2026 hunglinhpt.\nGiấy phép: GNU General Public License v3."
        alert.alertStyle = .informational
        alert.runModal()
    }

    @objc public func quitApp() {
        ipcServer.stop()
        NSApp.terminate(nil)
    }

    // MARK: - IpcServerDelegate

    public func ipcServer(_ server: IpcServer, didToggleViEn appID: String, enabled: Bool) {
        if appID == "*" || appID.isEmpty {
            self.isVietnameseMode = enabled
            self.configStore.config.enabled = enabled
            self.configStore.persist()
        }
    }

    public func ipcServer(_ server: IpcServer, didReceiveCrashReport code: UInt32, count: UInt32) {
        NSLog("[TextVN] Received crash report: code=%u count=%u", code, count)
    }
}
