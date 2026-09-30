// SPDX-License-Identifier: GPL-3.0-or-later
// SettingsView.swift — TextVN SwiftUI Settings Panel for macOS (P2-4 §3, PLAN §4.1)

import SwiftUI

public struct SettingsView: View {
    @ObservedObject public var store: ConfigStore
    @State private var isExpanded: Bool = false
    @State private var activeAlert: ActiveAlert?
    @State private var showMacroSheet: Bool = false
    /// Khi lỗi autostart, ta set `store.config.autostart` về giá trị cũ — cờ này
    /// chặn `onChange` chạy lại vòng 2 (F4-03).
    @State private var suppressAutostartChange = false

    /// Một nguồn alert duy nhất — SwiftUI không đảm bảo nhiều `.alert` cùng view.
    private enum ActiveAlert: Identifiable {
        case about
        case autostartFailed(String)

        var id: String {
            switch self {
            case .about: return "about"
            case let .autostartFailed(message): return "autostart:" + message
            }
        }

        var alert: Alert {
            switch self {
            case .about:
                return Alert(
                    title: Text("TextVN \(AppInfo.displayVersion) (macOS)"),
                    message: Text("Bộ gõ tiếng Việt cho macOS.\nPhát triển bởi hunglinhpt.\n\nBản quyền © 2026 hunglinhpt.\nGiấy phép: GNU General Public License v3."),
                    dismissButton: .default(Text("Đồng ý"))
                )
            case let .autostartFailed(message):
                return Alert(
                    title: Text("Không đổi được 'Khởi động cùng OS'"),
                    message: Text(message),
                    dismissButton: .default(Text("Đóng"))
                )
            }
        }
    }

    public var onClose: (() -> Void)?
    /// Báo host đổi cỡ cửa sổ khi Mở rộng/Thu nhỏ: `NSHostingView` KHÔNG tự
    /// resize `NSWindow` — trước đây nửa dưới (nhóm "Hệ thống") bị cắt (review R3 F3-2).
    public var onExpandedChange: ((Bool) -> Void)?

    /// Cỡ nội dung theo trạng thái — nguồn duy nhất cho view và cửa sổ host.
    public static func contentSize(expanded: Bool) -> CGSize {
        CGSize(width: 505, height: expanded ? 490 : 245)
    }

    public init(
        store: ConfigStore = .shared,
        onClose: (() -> Void)? = nil,
        onExpandedChange: ((Bool) -> Void)? = nil
    ) {
        self.store = store
        self.onClose = onClose
        self.onExpandedChange = onExpandedChange
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            // MARK: - 1. Compact Section
            HStack(alignment: .top, spacing: 18) {
                // Left Column: Basic Pickers
                VStack(alignment: .leading, spacing: 12) {
                    // Bảng mã
                    HStack {
                        Text("Bảng mã:")
                            .frame(width: 80, alignment: .leading)
                            .font(.system(size: 13, weight: .medium))
                        Picker("", selection: $store.config.output_charset) {
                            Text("1. Unicode dựng sẵn").tag("unicode_precomposed")
                            Text("2. TCVN3 (ABC)").tag("tcvn3")
                            Text("3. VNI Windows").tag("vni_windows")
                            Text("4. Unicode tổ hợp").tag("unicode_decomposed")
                        }
                        .labelsHidden()
                        .frame(maxWidth: .infinity)
                        .onChange(of: store.config.output_charset) { _ in persistAndNotify() }
                    }

                    // Kiểu gõ
                    HStack {
                        Text("Kiểu gõ:")
                            .frame(width: 80, alignment: .leading)
                            .font(.system(size: 13, weight: .medium))
                        Picker("", selection: $store.config.method) {
                            Text("1. Telex").tag("telex")
                            Text("2. VNI").tag("vni")
                            Text("3. VIQR").tag("viqr")
                            Text("4. Microsoft").tag("simple_telex")
                        }
                        .labelsHidden()
                        .frame(maxWidth: .infinity)
                        .onChange(of: store.config.method) { _ in persistAndNotify() }
                    }

                    // Phím chuyển
                    HStack(alignment: .center) {
                        Text("Phím chuyển:")
                            .frame(width: 80, alignment: .leading)
                            .font(.system(size: 13, weight: .medium))
                        HStack(spacing: 12) {
                            RadioButton(
                                title: "Ctrl + Shift",
                                isSelected: store.config.switch_key == "ctrl_shift"
                            ) {
                                store.config.switch_key = "ctrl_shift"
                                persistAndNotify()
                            }
                            RadioButton(
                                title: "Alt + Z",
                                isSelected: store.config.switch_key == "alt_z"
                            ) {
                                store.config.switch_key = "alt_z"
                                persistAndNotify()
                            }
                            RadioButton(
                                title: "Caps Lock",
                                isSelected: store.config.switch_key == "caps_lock"
                            ) {
                                store.config.switch_key = "caps_lock"
                                persistAndNotify()
                            }
                        }
                    }
                }
                .frame(maxWidth: .infinity)

                // Right Column: Action Buttons
                VStack(spacing: 8) {
                    Button(action: {
                        persistAndNotify()
                        onClose?()
                    }) {
                        Text("Đóng")
                            .frame(width: 100)
                    }
                    .keyboardShortcut(.defaultAction)

                    Button(action: {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            isExpanded.toggle()
                        }
                    }) {
                        Text(isExpanded ? "<< Thu nhỏ" : "Mở rộng >>")
                            .frame(width: 100)
                    }

                    Button(action: {
                        store.resetToDefaults()
                        try? AutostartManager.shared.setAutostart(enabled: store.config.autostart)
                        persistAndNotify()
                    }) {
                        Text("Mặc định")
                            .frame(width: 100)
                    }

                    Button(action: {
                        activeAlert = .about
                    }) {
                        Text("Thông tin")
                            .frame(width: 100)
                    }
                }
            }

            // MARK: - 2. Expanded Section
            if isExpanded {
                Divider()

                VStack(spacing: 12) {
                    // Group: Tùy chọn gõ
                    GroupBox(label: Text("Tùy chọn gõ").font(.system(size: 12, weight: .semibold))) {
                        VStack(alignment: .leading, spacing: 6) {
                            Toggle("Đặt dấu tự do", isOn: $store.config.free_marking)
                                .onChange(of: store.config.free_marking) { _ in persistAndNotify() }
                            Toggle("Tự động khôi phục phím cho từ sai", isOn: $store.config.auto_restore_english)
                                .onChange(of: store.config.auto_restore_english) { _ in persistAndNotify() }
                            HStack {
                                Toggle("Cho phép gõ tắt", isOn: $store.config.allow_macro_when_vi_off)
                                    .onChange(of: store.config.allow_macro_when_vi_off) { _ in persistAndNotify() }
                                Spacer()
                                Button("Bảng gõ tắt...") {
                                    showMacroSheet = true
                                }
                                .font(.system(size: 11))
                            }
                            // Nhãn khớp ui-spec §2 + Linux/Windows (F3-8; thay toggle
                            // "Bỏ dấu kiểu mới" cũ gây nghĩa ngược).
                            HStack(spacing: 12) {
                                RadioButton(
                                    title: "Dấu mới (hoà, thuỷ)",
                                    isSelected: store.config.diacritic_style == "new"
                                ) {
                                    store.config.diacritic_style = "new"
                                    persistAndNotify()
                                }
                                RadioButton(
                                    title: "Dấu cũ (hòa, thủy)",
                                    isSelected: store.config.diacritic_style == "old"
                                ) {
                                    store.config.diacritic_style = "old"
                                    persistAndNotify()
                                }
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.vertical, 4)
                    }

                    // Group: Hệ thống
                    GroupBox(label: Text("Hệ thống").font(.system(size: 12, weight: .semibold))) {
                        VStack(alignment: .leading, spacing: 6) {
                            Toggle("Bật hội thoại này khi khởi động", isOn: $store.config.show_dialog_on_startup)
                                .onChange(of: store.config.show_dialog_on_startup) { _ in persistAndNotify() }
                            Toggle("Khởi động cùng OS", isOn: $store.config.autostart)
                                .onChange(of: store.config.autostart) { newValue in
                                    guard !suppressAutostartChange else { return }
                                    do {
                                        try AutostartManager.shared.setAutostart(enabled: newValue)
                                        persistAndNotify()
                                    } catch {
                                        // Không ghi được SM/LaunchAgent → trả công tắc
                                        // về trạng thái THẬT thay vì "bật ảo"
                                        // (review F4-03).
                                        suppressAutostartChange = true
                                        store.config.autostart = !newValue
                                        suppressAutostartChange = false
                                        store.persist()
                                        activeAlert = .autostartFailed(error.localizedDescription)
                                    }
                                }
                            Toggle("Gõ không gạch chân (Non-preedit)", isOn: $store.config.non_preedit)
                                .onChange(of: store.config.non_preedit) { _ in persistAndNotify() }
                            Toggle("Chạy ngầm trong menu bar", isOn: $store.config.run_in_tray)
                                .onChange(of: store.config.run_in_tray) { _ in persistAndNotify() }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.vertical, 4)
                    }
                }
            }
        }
        .padding(14)
        .frame(
            width: Self.contentSize(expanded: isExpanded).width,
            height: Self.contentSize(expanded: isExpanded).height,
            alignment: .topLeading
        )
        .onChange(of: isExpanded) { expanded in onExpandedChange?(expanded) }
        .alert(item: $activeAlert) { $0.alert }
        .sheet(isPresented: $showMacroSheet) {
            MacroEditorSheet(store: store)
        }
    }

    private func persistAndNotify() {
        store.persist()
        let ver = UInt64(Date().timeIntervalSince1970)
        IpcServer.shared.broadcastConfigReload(version: ver)
    }
}

// MARK: - Subviews & Controls

public struct RadioButton: View {
    public let title: String
    public let isSelected: Bool
    public let action: () -> Void

    public var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: isSelected ? "largecircle.fill.circle" : "circle")
                    .foregroundColor(isSelected ? .accentColor : .secondary)
                    .font(.system(size: 13))
                Text(title)
                    .font(.system(size: 12))
            }
        }
        .buttonStyle(.plain)
    }
}

public struct MacroEditorSheet: View {
    @ObservedObject public var store: ConfigStore
    @Environment(\.dismiss) private var dismiss
    @State private var newTrigger: String = ""
    @State private var newExpand: String = ""

    public var body: some View {
        VStack(spacing: 12) {
            Text("Bảng gõ tắt")
                .font(.headline)

            List {
                ForEach(store.config.macros) { macro in
                    HStack {
                        Text(macro.trigger)
                            .font(.system(.body, design: .monospaced))
                            .bold()
                            .frame(width: 80, alignment: .leading)
                        Text("➔")
                            .foregroundColor(.secondary)
                        Text(macro.expand)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        Button(action: {
                            store.config.macros.removeAll { $0.trigger == macro.trigger }
                            store.persist()
                            MacroEditorSheet.notifyConfigChanged()
                        }) {
                            Image(systemName: "trash")
                                .foregroundColor(.red)
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
            .frame(height: 180)

            HStack(spacing: 8) {
                TextField("Từ gõ tắt", text: $newTrigger)
                    .frame(width: 100)
                TextField("Cụm từ thay thế", text: $newExpand)
                Button("Thêm") {
                    let trig = newTrigger.trimmingCharacters(in: .whitespaces)
                    let exp = newExpand.trimmingCharacters(in: .whitespaces)
                    if !trig.isEmpty && !exp.isEmpty {
                        store.config.macros.removeAll { $0.trigger == trig }
                        store.config.macros.append(MacroEntry(trigger: trig, expand: exp))
                        newTrigger = ""
                        newExpand = ""
                        store.persist()
                    }
                }
                .disabled(newTrigger.trimmingCharacters(in: .whitespaces).isEmpty || newExpand.trimmingCharacters(in: .whitespaces).isEmpty)
            }

            HStack {
                Spacer()
                Button("Đóng") {
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(16)
        .frame(width: 420, height: 320)
    }
}
