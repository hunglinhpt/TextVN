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

    /// Snapshot từ IPC server; offline = giữ giá trị cuối (P0-3 §4).
    private var viEnabled = true
    private var appdbJSON: Data = Data()
    private var secureMode = false

    public override init(server: IMKServer!, delegate: Any!, client: Any!) {
        // Config user trước; hỏng → engine default. Không bao giờ để IMK chết
        // vì config (S4, P0-3 §1.3). ABI lệch → engine nil, mọi phím PASS.
        let engine: ImeEngine?
        if let cfg = Self.loadConfig(), let e = try? ImeEngine(configJSON: cfg) {
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
        gatherContext(from: sender, force: true)
        Diagnostics.log("activateServer pid=\(Self.clientPid(sender))")
    }

    /// Mất focus → commit-before-hide (B13) + reset (P2-1 §3).
    public override func deactivateServer(_ sender: Any!) {
        commitBeforeHide(sender)
        engine?.reset()
        Diagnostics.log("deactivateServer — committed-before-hide (B13)")
        super.deactivateServer(sender)
    }

    /// Client đóng app → commit + reset. IMKInputController không có `didClose(_:)`
    /// — hook đúng là `inputControllerWillClose()` (MAC-032).
    public override func inputControllerWillClose() {
        commitBeforeHide(client())
        engine?.reset()
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

        // 2. flagsChanged — cập nhật modifier, không nuốt.
        if event.type == .flagsChanged {
            heldMods = KeyTranslator.modifiers(event.modifierFlags)
            return false
        }

        let mods = KeyTranslator.modifiers(event.modifierFlags)
        heldMods = mods

        // 1. Chord hệ thống (Cmd bất kỳ, Ctrl+phím khác Space toggle) → B6.
        if mods & FFI.modMeta != 0 {
            return false
        }

        // Toggle EN/VN: Ctrl+Shift+Space (P2-6 MAC-018; ADR-011 cho CapsLock mode).
        if mods == (FFI.modCtrl | FFI.modShift), event.keyCode == kVK_Space {
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
        let hint = resolveStrategy(context: context)
        engine?.setContext(
            enabled: viEnabled, secure: context.secure || secureMode,
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
                try ApplyReplace.apply(outcome, strategy: strategy,
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
            ctx.enabled = viEnabled ? 1 : 0
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
            }
        case let .stateUpdate(appID, enabled, _):
            // Toàn cục = `IpcMessage.globalAppID` ("*"), cùng quy ước với Windows TSF
            // (`GLOBAL_KEY`) — menu bar toggle phải tới được IMK (review R3 F3-1).
            if appID == IpcMessage.globalAppID {
                viEnabled = enabled
            }
        case .snapshot:
            Diagnostics.log("ipc snapshot received")
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
        viEnabled.toggle()
        Diagnostics.log("toggle vi=\(viEnabled)")
        // Client gửi ToggleViEn (client→server — review R1 F15); server broadcast
        // StateUpdate lại cho mọi client. Gửi .stateUpdate là sai chiều → disconnect.
        ipc.send(.toggleViEn(appID: IpcMessage.globalAppID, enabled: viEnabled))
        return true // nuốt Space toggle
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
                if rc == .success, let element = value {
                    // AXUIElement là CF class — downcast từ CFTypeRef an toàn ở đây
                    // vì attribute đã trả đúng kiểu (rc == .success).
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
            secureInputMode: false
        )
    }
}

/// Đếm chord thuần Swift (mirror `KeyEvent::is_chord` — P0-2 §1).
extension KeyEvent {
    func isChordSwift() -> Bool {
        mods & (FFI.modCtrl | FFI.modAlt | FFI.modMeta) != 0
    }
}
