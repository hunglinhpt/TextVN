// SPDX-License-Identifier: GPL-3.0-or-later
//! TextVNInputController — IMKInputController subclass (P2-1 §3–§7).
//!
//! Key flow (P2-1 §5) — mọi bước đều fail-open (lỗi → return false = PASS):
//! 0. Marker injected → return false (chống loop — P2-2 §6).
//! 1. Cmd/Option+chord hệ thống → return false (B6).
//! 2. flagsChanged → ghi modifier state (toggle Ctrl+Shift+Space) → false.
//! 3. `KeyTranslator.translate` (UCKeyTranslate theo layout user).
//! 4. Context từ FieldDetect cache (không AX query đồng bộ — MAC-031).
//! 5. Strategy qua `ime_strategy_resolve` (không viết lại thuật toán — P2-3 §5).
//! 6. `ime_key` → match action → `ApplyReplace` (nơi duy nhất sửa text).
//! 7. Mọi lỗi Swift → return false (S4 fail-open).
//!
//! 1 engine instance / process, main thread duy nhất (P0-2 §3).

import AppKit
import Carbon.HIToolbox
import CoreBridge
import CTextVNFFI // ime_context_v1 / ime_strategy_resolve dùng trực tiếp (MAC-032)
import Foundation
import InputMethodKit

/// Marker gắn vào CGEvent tự bơm — `handle()` loại phím có marker này (P2-2 §6).
public enum InjectedMarker {
    /// Giá trị bất kỳ nhưng cố định cho cả IMK + tap module.
    public static let userData: Int64 = 0x5458_564E // "TXVN"
}

/// `@objc(TextVNInputController)` (review R1 F10): IMKServer instantiate
/// controller qua `NSClassFromString` với tên trong Info.plist
/// `InputMethodServerControllerClass` — Swift class không có tên ObjC tường
/// minh resolve thành tên mangled `IMKLib.TextVNInputController` → IM
/// **không bao giờ** được tạo. Pitfall kinh điển IMK+Swift.
@objc(TextVNInputController)
public final class TextVNInputController: IMKInputController, IpcClientDelegate {
    // ------------------------------------------------------------- state

    /// Engine optional: chỉ `nil` khi ABI lệch lib (P0-2 §5 — doctor cảnh báo);
    /// mọi phím PASS khi nil — IMK không bao giờ crash vì engine hỏng (S4).
    private let engine: ImeEngine?
    private let translator = KeyTranslator()
    private let marked = MarkedState()
    private let fieldDetect = FieldDetect()
    private lazy var ipc = IpcClient()

    /// Caps IMK v1 (P2-1 §1): PREEDIT | FIELD_DETECT | SELECTION.
    /// `IME_CAP_INJECT_VK` **không** tự nhận — chỉ khi spike MAC-004 chứng minh
    /// cơ chế xóa (a)/(b)/(c); ở đây cơ chế (a) deleteBackward luôn có sẵn nên
    /// BackspaceType vẫn chạy được mà không cần inject VK.
    private let caps: UInt32 = FFI.capPreedit | FFI.capFieldDetect | FFI.capSelection

    /// Modifier state hiện tại (cho toggle hotkey — P2-1 §5 bước 2).
    private var heldMods: UInt32 = 0

    /// Trạng thái bật/tắt: toàn cục + override per-app từ Snapshot/StateUpdate
    /// (P0-3 §4). Offline = giữ giá trị cuối.
    private var viState = ViState()
    private var appdbJSON: Data = Data()
    /// Trạng thái theo dõi tổ hợp Ctrl+Shift tap (như Windows & Linux)
    private var ctrlDown = false
    private var shiftDown = false
    private var otherKeyPressed = false
    private var lastToggleTime: TimeInterval = 0

    /// Secure Input mode — đọc mỗi keyDown (Carbon, rẻ) để bịt khoảng trễ của
    /// cache FieldDetect trước khi async AX gather xong (S8).
    private var secureMode = false

    /// Vòng 15 (BUG-05): `non_preedit` từ config.json — bật thì strategy
    /// Preedit bị thay bằng BackspaceType (gõ không gạch chân). Mặc định
    /// `false` khi khoá thiếu (giữ hành vi gạch chân).
    private var nonPreedit = false

    public override init(server: IMKServer!, delegate: Any!, client: Any!) {
        // Config user trước; hỏng → engine default. Không bao giờ để IMK chết
        // vì config (S4, P0-3 §1.3). ABI lệch → engine nil, mọi phím PASS.
        let cfgData = Self.loadConfig()
        let engine: ImeEngine?
        if let cfg = cfgData, let e = try? ImeEngine(configJSON: cfg) {
            engine = e
        } else {
            engine = try? ImeEngine(configJSON: nil)
            if engine == nil {
                Diagnostics.log("FATAL: engine ABI mismatch — all keys will PASS")
            } else {
                Diagnostics.log("config rejected — engine running defaults")
            }
        }
        self.engine = engine
        // Toggle khởi tạo theo `config.enabled` (bản cũ hardcode `true`): khi
        // config tắt VN, hotkey Ctrl+Shift+Space bị lệch một nhịp (bật → vẫn tắt).
        self.viState = ViState(globalEnabled: Self.configEnabled(in: cfgData) ?? true)
        super.init(server: server, delegate: delegate, client: client)
        Diagnostics.log("controller init — caps=\(caps)")
        ipc.delegate = self
        ipc.connect()
    }

    deinit {
        ipc.disconnect()
    }

    // ------------------------------------------------------------- lifecycle

    /// Client focus → (P2-1 §3): ensure instance, ipc snapshot, detect field.
    public override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        marked.clear()
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        gatherContext(from: sender, force: true)
        Diagnostics.log("activateServer pid=\(Self.clientPid(sender))")
    }

    /// Mất focus → commit-before-hide (B13) + reset (P2-1 §3).
    public override func deactivateServer(_ sender: Any!) {
        commitBeforeHide(sender)
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        Diagnostics.log("deactivateServer — committed-before-hide (B13)")
        super.deactivateServer(sender)
    }

    /// Client đóng app → commit + reset. IMKInputController không có `didClose(_:)`
    /// — hook đúng là `inputControllerWillClose()` (MAC-032).
    public override func inputControllerWillClose() {
        commitBeforeHide(client())
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        super.inputControllerWillClose()
    }

    public override func menu() -> NSMenu! {
        // Menu do TextVN.app quản — IMK không thêm mục riêng (P2-4 §1).
        return nil
    }

    // ------------------------------------------------------------- key flow

    public override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask([.keyDown, .flagsChanged]).rawValue)
    }

    public override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event else { return false }
        guard event.type == .keyDown || event.type == .flagsChanged else { return false }

        // 0. Loop guard: phím tự inject (P2-2 §6).
        if let cg = event.cgEvent,
           cg.getIntegerValueField(.eventSourceUserData) == InjectedMarker.userData {
            return false
        }

        // 2. flagsChanged — cập nhật modifier và phát hiện tổ hợp Ctrl+Shift tap (chuẩn macOS/Windows).
        if event.type == .flagsChanged {
            let flags = event.modifierFlags
            let isCtrl = flags.contains(.control)
            let isShift = flags.contains(.shift)
            let hasAltOrCmd = flags.contains(.option) || flags.contains(.command)
            if hasAltOrCmd {
                otherKeyPressed = true
            }

            heldMods = KeyTranslator.modifiers(flags)

            if Self.isCtrlShiftTap(
                wasCtrl: ctrlDown,
                wasShift: shiftDown,
                currentCtrl: isCtrl,
                currentShift: isShift,
                otherKeyPressed: otherKeyPressed
            ) {
                let now = ProcessInfo.processInfo.systemUptime
                if now - lastToggleTime >= 0.25 {
                    lastToggleTime = now
                    _ = toggleVietnamese()
                }
                otherKeyPressed = true
            }

            ctrlDown = isCtrl
            shiftDown = isShift

            if !isCtrl && !isShift {
                otherKeyPressed = false
            }

            return false
        }

        otherKeyPressed = true

        let mods = KeyTranslator.modifiers(event.modifierFlags)
        heldMods = mods

        // 1. Chord hệ thống (Cmd bất kỳ, Ctrl+phím khác Space toggle) → B6.
        if mods & FFI.modMeta != 0 {
            return false
        }

        // Toggle EN/VN: Ctrl+Shift+Space (P2-6 MAC-018; ADR-011 cho CapsLock mode).
        // So MASK (bỏ bit Caps/Fn) — Caps Lock bật vẫn toggle được; so bằng `==`
        // làm hotkey chết khi CapsLock on. Linux so mask y hệt (`engine.cpp` §3).
        if Self.isToggleChord(mods: mods, keyCode: event.keyCode) {
            otherKeyPressed = true
            return toggleVietnamese()
        }

        // Modifier đơn: IMK phát flagsChanged (đã xử lý); keyDown thường không tới đây.
        guard event.type == .keyDown, let keyEvent = translator.translate(event) else {
            return false
        }
        // Key-up: engine không transform (P0-2 §3) — không cần gọi.
        if !keyEvent.keyDown {
            return false
        }
        // Chord Ctrl/Alt không toggle → luôn PASS (B6).
        if keyEvent.isChordSwift() {
            return false
        }

        // 4–5. Context + strategy (không I/O đồng bộ trong handle).
        let context = gatherContext(from: sender, force: false)
        // Secure Input mode là trạng thái HỆ THỐNG, đọc trực tiếp mỗi keyDown
        // (Carbon, không AX) — bịt cửa sổ trước khi async gather điền cache (S8).
        secureMode = IsSecureEventInputEnabled()
        let hint = resolveStrategy(context: context)
        engine?.setContext(
            enabled: viState.enabled(for: context.appID), secure: context.secure || secureMode,
            fieldRole: context.role, caps: caps,
            appId: context.appID,
            elementName: nil,
            hint: hint
        )

        // 6. Engine quyết định.
        guard let outcome = engine?.key(keyEvent) else { return false }
        if outcome.isError {
            Diagnostics.log("engine error flag — pass-through")
            return false
        }

        let strategy = OutputStrategy(raw: hint) ?? .backspaceType
        // Vòng 15 (BUG-05): tôn trọng toggle "Gõ không gạch chân" — Preedit
        // (gạch chân composition) → BackspaceType (delete+type, không gạch chân).
        let effectiveStrategy = Self.effectiveStrategy(strategy, nonPreedit: nonPreedit)
        guard let target = clientTarget(sender) else { return false }

        // Self-heal (P2-1 §4): engine tin đang marked nhưng app đã clear.
        if marked.text.isEmpty == false, target.markedRange().location == NSNotFound {
            Diagnostics.log("self-heal: marked desync — ime_reset")
            marked.clear()
            engine?.reset()
        }

        switch outcome.action {
        case .pass:
            return false
        case .replace, .commit, .restore:
            do {
                try ApplyReplace.apply(outcome, strategy: effectiveStrategy,
                                       target: target, marked: marked,
                                       onReset: { [weak self] in self?.engine?.reset() })
                logAction(outcome)
                return true
            } catch {
                // 7. Fail-open: forward phím, engine không được treo (S4, P2-1 §12).
                Diagnostics.log("apply failed (\(error)) — fail-open")
                engine?.reset()
                return false
            }
        }
    }

    // ------------------------------------------------------------- commit B13

    private func commitBeforeHide(_ sender: Any!) {
        guard !marked.text.isEmpty, let target = clientTarget(sender) else {
            marked.clear()
            return
        }
        // B13: chữ đang marked trở thành text vĩnh viễn — không mất chữ.
        target.insert(marked.text, replacementRange: .notFound)
        marked.clear()
        engine?.reset()
    }

    // ------------------------------------------------------------- strategy

    /// Resolve strategy qua FFI dùng chung — Swift KHÔNG viết lại thuật toán (P2-3 §5).
    private func resolveStrategy(context: FieldContext) -> Int64 {
        var outStrategy: Int64 = Int64(IME_STRATEGY_PASSTHROUGH)
        // Closure nhiều lệnh phải `return` tường minh — thiếu thì rc là `()` (MAC-032).
        let rc: Int32 = appdbJSON.withUnsafeBytes { raw -> Int32 in
            var ctx = ime_context_v1()
            ctx.abi_version = FFI.abiVersion
            ctx.enabled = viState.enabled(for: context.appID) ? 1 : 0
            ctx.secure = (context.secure || secureMode) ? 1 : 0
            ctx.field_role = context.role
            ctx.caps = caps
            ctx.hint = -1
            let appIdC = Array(context.appID.utf8CString)
            return appIdC.withUnsafeBufferPointer { buf -> Int32 in
                ctx.app_id = buf.baseAddress
                return ime_strategy_resolve(
                    &ctx,
                    raw.baseAddress.map { $0.assumingMemoryBound(to: UInt8.self) },
                    raw.count, &outStrategy
                )
            }
        }
        guard rc == IME_OK else {
            Diagnostics.log("strategy_resolve rc=\(rc) — dùng hint=-1")
            return -1
        }
        return outStrategy
    }

    /// Gather field context: cache-first (MAC-031). Cache miss → role unknown,
    /// AX gather async sẽ `store()` cho lần sau — handle() không bao giờ query AX.
    private func gatherContext(from sender: Any!, force: Bool) -> FieldContext {
        let pid = Self.clientPid()
        if !force, let cached = fieldDetect.cached(for: pid) {
            return cached
        }
        let fallback = FieldContext(
            appID: Self.appID(for: pid), role: FieldRole.unknown, secure: false
        )
        fieldDetect.store(fallback, for: pid)
        // Gather async: AX query trên queue riêng, kết quả qua store() (P2-3 §2).
        Self.gatherAX(for: pid, fieldDetect: fieldDetect)
        return fallback
    }

    // ------------------------------------------------------------- IPC delegate

    public func ipcClient(_ client: IpcClient, didReceive message: IpcMessage) {
        switch message {
        case let .configReload(version):
            Diagnostics.log("ipc ConfigReload v\(version)")
            if let cfg = Self.loadConfig() {
                engine?.reloadConfig(cfg)
                nonPreedit = Self.configNonPreedit(in: cfg)
                // `config.enabled` là nguồn sự thật của toggle toàn cục (macOS
                // không có state.json) — giữ viState đồng bộ khi đổi từ Settings.
                if let enabled = Self.configEnabled(in: cfg) {
                    viState.apply(stateUpdate: IpcMessage.globalAppID, enabled: enabled)
                }
            }
        case let .stateUpdate(appID, enabled, _):
            // Toàn cục = `IpcMessage.globalAppID` ("*") — MỘT quy ước (R3 F3-1);
            // app khác → override per-app (menu bar "Bật tiếng Việt cho <app>").
            viState.apply(stateUpdate: appID, enabled: enabled)
        case let .snapshot(_, state, appdbVersion, _):
            // Server gửi Snapshot sau `GetSnapshot` — áp cả "*" lẫn per-app;
            // trước đây bỏ qua hoàn toàn nên state ban đầu không bao giờ được áp.
            Diagnostics.log("ipc snapshot received (\(state.count) entries, appdb \(appdbVersion))")
            viState.apply(snapshot: state)
        default:
            break
        }
    }

    public func ipcClientDidDisconnect(_ client: IpcClient) {
        // Offline: giữ config/state đã đọc — gõ vẫn chạy (P0-3 §4).
        Diagnostics.log("ipc offline — engine keeps last config")
    }

    // ------------------------------------------------------------- helpers

    private func toggleVietnamese() -> Bool {
        let newValue = !viState.globalEnabled
        viState.apply(stateUpdate: IpcMessage.globalAppID, enabled: newValue)
        Diagnostics.log("toggle vi=\(newValue)")
        // Client gửi ToggleViEn (client→server — review R1 F15); server broadcast
        // StateUpdate lại cho mọi client. Gửi .stateUpdate là sai chiều → disconnect.
        ipc.send(.toggleViEn(appID: IpcMessage.globalAppID, enabled: newValue))
        return true // nuốt Space toggle
    }

    /// Ctrl+Shift+Space — so MASK, bỏ bit Caps/Fn (ADR-011: CapsLock là kiểu gõ
    /// hoa, không được làm chết hotkey). Tách static để test không cần IMKServer.
    static func isToggleChord(mods: UInt32, keyCode: UInt16) -> Bool {
        let ignore = FFI.modCaps | FFI.modFn
        return keyCode == kVK_Space
            && (mods & ~ignore) == (FFI.modCtrl | FFI.modShift)
    }

    /// Kiểm tra modifier tap: Ctrl+Shift được bấm cùng lúc rồi nhả ra mà không kèm phím khác (như Windows/Linux).
    static func isCtrlShiftTap(
        wasCtrl: Bool,
        wasShift: Bool,
        currentCtrl: Bool,
        currentShift: Bool,
        otherKeyPressed: Bool
    ) -> Bool {
        wasCtrl && wasShift && (!currentCtrl || !currentShift) && !otherKeyPressed
    }

    private func logAction(_ outcome: KeyOutcome) {
        // S2: log chỉ action + độ dài — không text.
        let len: Int
        switch outcome.action {
        case .pass: len = 0
        case let .replace(_, insert, _): len = insert.count
        case let .commit(insert): len = insert.count
        case let .restore(_, insert): len = insert.count
        }
        Diagnostics.log("action applied (len=\(len))")
    }

    private func clientTarget(_ sender: Any!) -> TextTarget? {
        // Client của IMK conform `IMKTextInput` (documented Apple contract).
        guard let sender, let client = sender as? IMKTextInput else {
            return nil
        }
        return IMKTextTarget(client: client)
    }

    /// PID của app đang focus (frontmost) — app_id cho preset (P2-3 §1).
    /// IMK không expose pid client trực tiếp; NSWorkspace frontmost là nguồn
    /// đáng tin khi `activateServer`/`handle` (client == app đang gõ).
    static func clientPid(_ sender: Any? = nil) -> pid_t {
        NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
    }

    static func appID(for pid: pid_t) -> String {
        guard pid != 0,
              let app = NSRunningApplication(processIdentifier: pid) else {
            return "unknown"
        }
        return FieldDetect.normalizeAppID(
            bundleID: app.bundleIdentifier,
            executableName: app.executableURL?.lastPathComponent
        )
    }

    static func loadConfig() -> Data? {
        let url = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/TextVN/config.json")
        return try? Data(contentsOf: url)
    }

    /// Đọc `enabled` từ config.json (thiếu khoá → nil = giữ mặc định hiện tại).
    static func configEnabled(in data: Data?) -> Bool? {
        guard let data,
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else { return nil }
        return obj["enabled"] as? Bool
    }

    /// Vòng 15 (BUG-05): tôn trọng toggle "Gõ không gạch chân" — Preedit
    /// (gạch chân composition) → BackspaceType (delete+type, không gạch chân).
    /// Tách static để swift test gọi trực tiếp (không cần IMKServer).
    static func effectiveStrategy(
        _ strategy: OutputStrategy, nonPreedit: Bool
    ) -> OutputStrategy {
        (nonPreedit && strategy == .preedit) ? .backspaceType : strategy
    }

    static func configNonPreedit(in data: Data?) -> Bool {
        guard let data,
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else { return false }
        return obj["non_preedit"] as? Bool ?? false
    }

    /// Gather AX bất đồng bộ (không block handle). V1: role từ focused element
    /// nếu query được; MAC-030/031 nâng cấp qua AXObserver + cache chi tiết hơn.
    static func gatherAX(for pid: pid_t, fieldDetect: FieldDetect) {
        DispatchQueue.global(qos: .utility).async {
            var ctx = FieldContext(appID: appID(for: pid), role: FieldRole.unknown, secure: false)
            if pid != 0 {
                let app = AXUIElementCreateApplication(pid)
                var value: CFTypeRef?
                let rc = AXUIElementCopyAttributeValue(
                    app, kAXFocusedUIElementAttribute as CFString, &value
                )
                if rc == .success, let element = value,
                   CFGetTypeID(element) == AXUIElementGetTypeID() {
                    // Kiểm CFTypeID TRƯỚC khi downcast: app trả kiểu khác làm
                    // `unsafeDowncast` trap → giết IMK; fail-open (S4) nên bỏ qua,
                    // giữ context mặc định.
                    let ax = unsafeDowncast(element as AnyObject, to: AXUIElement.self)
                    let snapshot = Self.axSnapshot(of: ax)
                    let (role, secure) = FieldRules.map(snapshot)
                    ctx = FieldContext(appID: ctx.appID, role: role, secure: secure)
                }
            }
            fieldDetect.store(ctx, for: pid)
        }
    }

    static func axSnapshot(of element: AXUIElement) -> AXSnapshot {
        func attr(_ name: String) -> String? {
            var value: CFTypeRef?
            let rc = AXUIElementCopyAttributeValue(element, name as CFString, &value)
            guard rc == .success else { return nil }
            return value as? String
        }
        return AXSnapshot(
            role: attr(kAXRoleAttribute),
            subrole: attr(kAXSubroleAttribute),
            placeholder: attr(kAXPlaceholderValueAttribute),
            description: attr(kAXDescriptionAttribute),
            title: attr(kAXTitleAttribute),
            identifier: attr(kAXIdentifierAttribute),
            // Secure Input mode hệ thống (Terminal sudo/ssh hỏi mật khẩu...):
            // trước đây hardcode `false` nên FieldRules không bao giờ thấy secure
            // khi app không lộ subrole AXSecureTextField (S8).
            secureInputMode: IsSecureEventInputEnabled()
        )
    }
}

/// Đếm chord thuần Swift (mirror `KeyEvent::is_chord` — P0-2 §1).
extension KeyEvent {
    func isChordSwift() -> Bool {
        mods & (FFI.modCtrl | FFI.modAlt | FFI.modMeta) != 0
    }
}
