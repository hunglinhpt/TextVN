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
    static let imkCaps: UInt32 = FFI.capPreedit | FFI.capFieldDetect | FFI.capSelection
    private let caps: UInt32 = TextVNInputController.imkCaps

    /// Trạng thái bật/tắt: toàn cục + override per-app từ Snapshot/StateUpdate
    /// (P0-3 §4). Offline = giữ giá trị cuối. R2-44: MỘT trạng thái cho cả tiến
    /// trình IMK (`ViStateStore.shared`) — mỗi app client có controller riêng, toggle
    /// lúc TextVN.app không chạy phải áp cho mọi app, không chỉ app đang gõ.
    private var viState: ViState {
        get { ViStateStore.shared.state }
        set { ViStateStore.shared.state = newValue }
    }
    /// IPC của controller này đã nhận Snapshot (server đang chạy) — chỉ đổi trên main.
    private var ipcOnline = false
    /// R2-43: strategy qua FFI + preset appdb đi kèm bundle, cache theo app/field.
    private static let strategyResolver = StrategyResolver(
        appdb: StrategyResolver.loadBundledAppdb(),
        caps: TextVNInputController.imkCaps
    )
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
    /// `false` khi khoá thiếu (giữ hành vi gạch chân). R2-49: đọc ngay khi tạo
    /// controller và mỗi lần app được focus, không chỉ khi nhận ConfigReload.
    private var nonPreedit = false

    /// Cấu hình Trợ lý ngữ pháp (AS-060 / P4-6)
    private var assistSettings: AssistSettings = AssistSettings()
    /// Trạng thái hoàn tác 1 bước sau khi auto-fix (Cmd+Z trong 5s)
    private var assistUndo: AssistUndoState?
    /// Candidate panel của IMK cho Mode 1 (Highlight)
    private var candidatePanel: IMKCandidates?
    /// Danh sách gợi ý hiện tại cho candidate panel (kèm kế hoạch A2 — R-7)
    private var currentCandidates: [AssistCandidate] = []

    // R2-106: phím chuyển CỐ ĐỊNH Ctrl+Shift (tap) / Ctrl+Shift+Space như Windows/Linux.
    // Vòng 16 (B3) từng thực thi `switch_key` Alt+Z/Caps Lock riêng trên macOS — lệch
    // parity, và Caps Lock không chặn được trạng thái chữ hoa của hệ thống. Khoá
    // `switch_key` cũ trong config.json (nếu có) bị bỏ qua: Ctrl+Shift luôn hoạt động.

    public override init(server: IMKServer!, delegate: Any!, client: Any!) {
        // Config user trước; hỏng → engine default. Không bao giờ để IMK chết
        // vì config (S4, P0-3 §1.3). ABI lệch → engine nil, mọi phím PASS.
        let cfgData = Self.loadConfig()
        let engine: ImeEngine?
        // R2-44: engine KHÔNG gate theo `config.enabled` — ViState/ctx.enabled là cổng
        // duy nhất, nếu không Ctrl+Shift không bật lại được khi TextVN.app không chạy.
        if let cfg = cfgData, let e = try? ImeEngine(configJSON: Self.engineConfig(cfg)) {
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
        if let e = engine {
            MacSpellProvider.shared.register(with: e)
        }
        self.nonPreedit = Self.configNonPreedit(in: cfgData)
        self.assistSettings = Self.configAssist(in: cfgData)
        if let s = server {
            self.candidatePanel = IMKCandidates(server: s, panelType: kIMKSingleRowSteppingCandidatePanel)
        }
        // Toggle khởi tạo theo `config.enabled` (bản cũ hardcode `true`): khi
        // config tắt VN, hotkey Ctrl+Shift+Space bị lệch một nhịp (bật → vẫn tắt).
        // Chỉ controller ĐẦU TIÊN của tiến trình khởi tạo; sau đó giữ trạng thái
        // chung (có thể đã toggle lúc offline — R2-44).
        ViStateStore.shared.seedIfNeeded(globalEnabled: Self.configEnabled(in: cfgData) ?? true)
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
        // R2-49: config.json có thể đổi lúc TextVN.app không chạy (không có ConfigReload).
        nonPreedit = Self.configNonPreedit(in: Self.loadConfig())
        assistSettings = Self.configAssist(in: Self.loadConfig())
        assistUndo = nil
        candidatePanel?.hide()
        currentCandidates.removeAll()
        // Vòng 16 (audit B2): focus = thời điểm an toàn để nối lại IPC nếu lần
        // trước violation/timeout (connect() idempotent — guard fd/timer).
        ipc.connect()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        // B7: làm ấm/refresh FieldContext ngay khi activate (có AX timeout +
        // gather-guard). Kết quả không dùng trực tiếp -> gán `_` cho rõ ý định.
        _ = gatherContext(from: sender, force: true)
        Diagnostics.log("activateServer pid=\(Self.clientPid(sender))")
    }

    /// Mất focus → commit-before-hide (B13) + reset (P2-1 §3).
    public override func deactivateServer(_ sender: Any!) {
        commitBeforeHide(sender)
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        assistUndo = nil
        candidatePanel?.hide()
        currentCandidates.removeAll()
        Diagnostics.log("deactivateServer — committed-before-hide (B13)")
        super.deactivateServer(sender)
    }

    public override func commitComposition(_ sender: Any!) {
        commitBeforeHide(sender)
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        assistUndo = nil
        candidatePanel?.hide()
        currentCandidates.removeAll()
        Diagnostics.log("commitComposition — committed-before-hide")
        super.commitComposition(sender)
    }

    /// Client đóng app → commit + reset. IMKInputController không có `didClose(_:)`
    /// — hook đúng là `inputControllerWillClose()` (MAC-032).
    public override func inputControllerWillClose() {
        commitBeforeHide(client())
        engine?.reset()
        ctrlDown = false
        shiftDown = false
        otherKeyPressed = false
        assistUndo = nil
        candidatePanel?.hide()
        currentCandidates.removeAll()
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

        // 1a. Hỗ trợ Undo 1 bước khi bấm Cmd+Z ngay sau khi auto-fix (AS-060 / P4-6)
        if Self.isCmdZ(mods: mods, keyCode: event.keyCode) {
            if handleAssistUndo(sender: sender) {
                return true
            }
        } else {
            assistUndo = nil
        }

        // 1b. Chord hệ thống (Cmd bất kỳ khác, Ctrl+phím khác Space toggle) → B6.
        // Cmd+←, Cmd+Z, Cmd+V… đổi con trỏ/text mà engine không thấy: chốt từ đang
        // gõ (marked → text thật) và quên nó, như TSF — giữ lại thì phím dấu kế
        // tiếp xoá `owned` ký tự ở vị trí mới (BackspaceType) hoặc đè marked.
        if mods & FFI.modMeta != 0 {
            finishWordBeforeChord(sender)
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
        // Chord Ctrl/Alt không toggle → luôn PASS (B6), sau khi chốt từ đang gõ.
        if keyEvent.isChordSwift() {
            finishWordBeforeChord(sender)
            return false
        }

        // 4–5. Context + strategy (không I/O đồng bộ trong handle).
        let context = gatherContext(from: sender, force: false)
        // Secure Input mode là trạng thái HỆ THỐNG, đọc trực tiếp mỗi keyDown
        // (Carbon, không AX) — bịt cửa sổ trước khi async gather điền cache (S8).
        secureMode = IsSecureEventInputEnabled()
        let hint = Self.strategyResolver.strategy(
            appID: context.appID, role: context.role,
            enabled: viState.enabled(for: context.appID),
            secure: context.secure || secureMode
        )
        let effectiveHint = (nonPreedit && hint == OutputStrategy.preedit.rawValue)
            ? OutputStrategy.backspaceType.rawValue
            : hint
        let effectiveCaps = nonPreedit ? (caps & ~FFI.capPreedit) : caps
        engine?.setContext(
            enabled: viState.enabled(for: context.appID), secure: context.secure || secureMode,
            fieldRole: context.role, caps: effectiveCaps,
            appId: context.appID,
            elementName: nil,
            hint: effectiveHint
        )

        guard let target = clientTarget(sender) else { return false }

        // Self-heal (P2-1 §4) — PHẢI chạy TRƯỚC engine: app đã tự commit marked
        // (click chuột sang chỗ khác…). Chạy sau thì outcome đã tính với từ cũ
        // (`delete_count = owned`) vẫn được áp → `deleteBackward` xoá chữ ở vị trí
        // con trỏ MỚI (audit 2026-09-30 đã sửa thứ tự này; bị đảo lại sau đó).
        if marked.text.isEmpty == false, target.markedRange().location == NSNotFound {
            Diagnostics.log("self-heal: marked desync — ime_reset")
            marked.clear()
            engine?.reset()
        }

        // 6. Engine quyết định.
        guard let outcome = engine?.key(keyEvent) else { return false }
        if outcome.isError {
            Diagnostics.log("engine error flag — pass-through")
            // Vòng 16 (audit B4): pipeline crash-report từng đứt hoàn toàn
            // (không ai gửi .crashReport → badge lỗi ở app là dead code). Gửi
            // tại đây — nơi duy nhất lộ engine bất thường; offline thì no-op.
            ipc.send(.crashReport(code: 1, count: 1))
            return false
        }

        // Vòng 15 (BUG-05): tôn trọng toggle "Gõ không gạch chân" — Preedit
        // (gạch chân composition) → BackspaceType (delete+type, không gạch chân).
        let effectiveStrategy = Self.outputStrategy(hint: hint, nonPreedit: nonPreedit)

        // B2 — như TSF (`compose.rs` native_boundary_char) và Linux (`lc_plan_key`):
        // Enter/Tab ở ranh giới phải tới app dưới dạng PHÍM thật (chat gửi tin, Tab
        // chuyển ô). Chỉ chốt từ rồi trả phím; chèn "\n"/"\t" thành text là nuốt
        // mất Enter — Messages/Slack xuống dòng thay vì gửi.
        var toApply = outcome
        var forwardKey = false
        if outcome.isWordEnd,
           keyEvent.vk == Self.vkReturn || keyEvent.vk == Self.vkTab,
           let trimmed = Self.withoutNativeBoundary(outcome) {
            toApply = trimmed
            forwardKey = true
        }

        switch toApply.action {
        case .pass:
            // Chủ đích (vòng 32, audit F-F — ghi nhận thiết kế): khi trợ lý bật,
            // phím trigger (space/./,/…/Enter) được IMK CHÈN trực tiếp
            // (`target.insert`) thay vì đi đường key-event native — IMK cần
            // chính điểm kiểm tra này để check câu NGAY TRƯỚC khi dấu câu
            // commit vào document. Hệ quả: app nhạy đường-vào (terminal
            // history, autocomplete) thấy trigger qua insertText thay vì
            // keyDown khi trợ lý bật, và đường native khi tắt. Windows/Linux
            // không có khác biệt này (phím đi native, check silent sau).
            if marked.isEmpty, isAssistActive(for: context.appID) {
                let triggerKind = CoreBridge.FFI.assistTriggerKind(ch: keyEvent.ch)
                if triggerKind != FFI.assistTriggerNone {
                    if let scalar = Unicode.Scalar(keyEvent.ch) {
                        target.insert(String(scalar), replacementRange: NSRange(location: NSNotFound, length: 0))
                    }
                    processAssist(triggerKind: triggerKind, target: target, sender: sender)
                    return true
                }
            }
            return false
        case .replace, .commit, .restore:
            do {
                try ApplyReplace.apply(toApply, strategy: effectiveStrategy,
                                       target: target, marked: marked,
                                       onReset: { [weak self] in self?.engine?.reset() })
                logAction(toApply)
                if marked.isEmpty, isAssistActive(for: context.appID) {
                    let triggerKind = CoreBridge.FFI.assistTriggerKind(ch: keyEvent.ch)
                    if triggerKind != FFI.assistTriggerNone {
                        processAssist(triggerKind: triggerKind, target: target, sender: sender)
                    }
                }
                return !forwardKey
            } catch {
                // 7. Fail-open: forward phím, engine không được treo (S4, P2-1 §12).
                Diagnostics.log("apply failed (\(error)) — fail-open")
                engine?.reset()
                return false
            }
        }
    }

    // ------------------------------------------------------------- commit B13

    /// Chord hệ thống: chốt marked (nếu có) thành text thật rồi quên từ đang gõ.
    private func finishWordBeforeChord(_ sender: Any!) {
        commitBeforeHide(sender)
        engine?.reset()
    }

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

    /// Strategy qua FFI dùng chung (`StrategyResolver` — Swift KHÔNG viết lại thuật
    /// toán, P2-3 §5) → cơ chế áp text. Lỗi FFI (`-1`) → BackspaceType (an toàn
    /// nhất); non-preedit hạ Preedit → BackspaceType. Tách static để test.
    static func outputStrategy(hint: Int64, nonPreedit: Bool) -> OutputStrategy {
        effectiveStrategy(OutputStrategy(raw: hint) ?? .backspaceType, nonPreedit: nonPreedit)
    }

    /// Gather field context: cache-first (MAC-031). Cache miss → role unknown,
    /// AX gather async sẽ `store()` cho lần sau — handle() không bao giờ query AX.
    private func gatherContext(from sender: Any!, force: Bool) -> FieldContext {
        let pid = Self.clientPid(sender)
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
        // B12: IpcClient marshal delegate về main (F16) — ràng buộc bằng
        // precondition để regression (marshal bị gỡ) sập ngay ở debug thay vì
        // data race âm thầm trên state main-thread.
        assert(Thread.isMainThread, "ipcClient must be called on main thread")
        switch message {
        case let .configReload(version):
            Diagnostics.log("ipc ConfigReload v\(version)")
            if let cfg = Self.loadConfig() {
                _ = engine?.reloadConfig(Self.engineConfig(cfg))
                nonPreedit = Self.configNonPreedit(in: cfg)
                assistSettings = Self.configAssist(in: cfg)
                // `config.enabled` là nguồn sự thật của toggle toàn cục (macOS
                // không có state.json) — giữ viState đồng bộ khi đổi từ Settings.
                if let enabled = Self.configEnabled(in: cfg) {
                    viState.apply(stateUpdate: IpcMessage.globalAppID, enabled: enabled)
                }
            }
        case let .stateUpdate(appID, enabled, _):
            // Toàn cục = `IpcMessage.globalAppID` ("*") — MỘT quy ước (R3 F3-1);
            // app khác → override per-app (menu bar "Bật tiếng Việt cho <app>").
            if appID == IpcMessage.assistAppID {
                // Kill-switch Trợ lý ngữ pháp (D20) — KHÔNG phải app override.
                ViStateStore.shared.applyAssistState(enabled)
                Diagnostics.log("ipc assist kill-switch \(enabled ? "on" : "off")")
                return
            }
            viState.apply(stateUpdate: appID, enabled: enabled)
        case let .snapshot(_, state, appdbVersion, _):
            // Server gửi Snapshot sau `GetSnapshot` — áp cả "*" lẫn per-app;
            // trước đây bỏ qua hoàn toàn nên state ban đầu không bao giờ được áp.
            Diagnostics.log("ipc snapshot received (\(state.count) entries, appdb \(appdbVersion))")
            ipcOnline = true
            // `applySnapshot` áp cả khoá kill-switch `state["assist"]` (D20).
            if ViStateStore.shared.applySnapshot(state) {
                // R2-44: toggle lúc offline thắng "*" cũ của server — báo TextVN.app
                // để app ghi `config.enabled` và broadcast cho mọi controller.
                ipc.send(.toggleViEn(appID: IpcMessage.globalAppID, enabled: viState.globalEnabled))
            }
            Diagnostics.log("ipc assist kill-switch \(assistKillSwitchOn ? "on" : "off")")
        default:
            break
        }
    }

    public func ipcClientDidDisconnect(_ client: IpcClient) {
        // Offline: giữ config/state đã đọc — gõ vẫn chạy (P0-3 §4).
        ipcOnline = false
        Diagnostics.log("ipc offline — engine keeps last config")
    }

    // ------------------------------------------------------------- helpers

    private func toggleVietnamese() -> Bool {
        // Offline (TextVN.app không chạy): trạng thái giữ trong ViStateStore và được
        // gửi lại ở Snapshot kế tiếp (R2-44) — send() dưới đây bị bỏ khi offline.
        let newValue = ViStateStore.shared.toggleGlobal(online: ipcOnline)
        Diagnostics.log("toggle vi=\(newValue)")
        // Client gửi ToggleViEn (client→server — review R1 F15); server broadcast
        // StateUpdate lại cho mọi client. Gửi .stateUpdate là sai chiều → disconnect.
        ipc.send(.toggleViEn(appID: IpcMessage.globalAppID, enabled: newValue))
        return true // nuốt Space toggle
    }

    /// VK canonical (Win) của Enter/Tab — `keymap::vk` trong core.
    static let vkReturn: UInt32 = 0x0D
    static let vkTab: UInt32 = 0x09

    /// COMMIT/RESTORE kết thúc bằng ranh giới Enter/Tab (`\n` `\r` `\t`) → bản đã
    /// bỏ ký tự đó (app sẽ nhận phím thật); outcome khác → `nil` (giữ nguyên).
    /// REPLACE có WORD_END là gõ tắt — phím trigger bị nuốt theo P0-3 §1.1.
    static func withoutNativeBoundary(_ outcome: KeyOutcome) -> KeyOutcome? {
        func strip(_ text: String) -> String? {
            guard let last = text.last, last == "\n" || last == "\r" || last == "\t" else {
                return nil
            }
            return String(text.dropLast())
        }
        switch outcome.action {
        case let .commit(insert):
            guard let text = strip(insert) else { return nil }
            return KeyOutcome(action: .commit(insert: text), flags: outcome.flags)
        case let .restore(deleteCount, insert):
            guard let text = strip(insert) else { return nil }
            return KeyOutcome(
                action: .restore(deleteCount: deleteCount, insert: text), flags: outcome.flags
            )
        default:
            return nil
        }
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
    /// B9: ưu tiên danh tính client từ `sender` (bundle id → NSRunningApplication)
    /// vì `NSWorkspace.frontmostApplication` có thể trỏ panel phụ/XPC khi focus
    /// nằm ở cửa sổ không phải frontmost app → preset sai. Không lấy được từ
    /// sender thì rơi về frontmost (hành vi cũ, đúng trong đa số ca).
    static func clientPid(_ sender: Any? = nil) -> pid_t {
        if let pid = pidFromSender(sender) {
            return pid
        }
        return NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
    }

    /// Trích pid từ `sender` (IMKTextInput/NSString bundle id). Trả nil nếu
    /// không xác định được — caller rơi về frontmost.
    private static func pidFromSender(_ sender: Any?) -> pid_t? {
        guard let sender else { return nil }
        var bundleID: String?
        if let client = sender as? IMKTextInput {
            // bundleIdentifier() trả String? — chỉ nhận khi không nil và không rỗng.
            let bid = client.bundleIdentifier()
            if let bid, !bid.isEmpty {
                bundleID = bid
            }
        } else if let s = sender as? String, !s.isEmpty {
            bundleID = s
        }
        guard let bundleID else { return nil }
        let apps = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        return apps.first(where: { $0.isActive })?.processIdentifier ?? apps.first?.processIdentifier
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

    /// Thiếu khoá / config hỏng → `false` (gạch chân) — CÙNG mặc định khi decode
    /// của `ConfigModel` (TextVN.app) và CHANGELOG 0.2.27. Bản cài mới ghi khoá tường minh.
    /// Đọc cấu hình ssist từ config.json.
    static func configAssist(in data: Data?) -> AssistSettings {
        AssistConfigParser.parse(from: data)
    }

    static func configNonPreedit(in data: Data?) -> Bool {
        guard let data,
              let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else { return false }
        return obj["non_preedit"] as? Bool ?? false
    }

    /// R2-44: config cho engine với `enabled = true`. `config.enabled` chỉ là trạng
    /// thái toàn cục BAN ĐẦU (ViState); engine gate `opts.enabled && ctx.enabled`
    /// nên để nguyên `false` thì Ctrl+Shift chỉ đổi ctx và không bật lại được khi
    /// TextVN.app không chạy (không ai gửi ConfigReload). JSON không parse được →
    /// trả nguyên bản (engine tự từ chối như trước).
    static func engineConfig(_ data: Data) -> Data {
        guard var obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else { return data }
        obj["enabled"] = true
        return (try? JSONSerialization.data(withJSONObject: obj)) ?? data
    }

    /// Gather AX bất đồng bộ (không block handle). V1: role từ focused element
    /// nếu query được; MAC-030/031 nâng cấp qua AXObserver + cache chi tiết hơn.
    static func gatherAX(for pid: pid_t, fieldDetect: FieldDetect) {
        // B7: chặn gather chồng cho cùng pid (cache miss liên tiếp) — tránh kết
        // quả cũ ghi đè context mới.
        guard fieldDetect.beginGather(pid: pid) else { return }
        DispatchQueue.global(qos: .utility).async {
            defer { fieldDetect.endGather(pid: pid) }
            var ctx = FieldContext(appID: appID(for: pid), role: FieldRole.unknown, secure: false)
            if pid != 0 {
                let app = AXUIElementCreateApplication(pid)
                // B7: giới hạn messaging timeout — mặc định AX có thể chờ tới ~6s
                // cho app không phản hồi (treo gather). 0.3s đủ cho app khoẻ.
                AXUIElementSetMessagingTimeout(app, 0.3)
                var value: CFTypeRef?
                let rc = AXUIElementCopyAttributeValue(
                    app, kAXFocusedUIElementAttribute as CFString, &value
                )
                // SAFETY: `unsafeDowncast` chỉ chạy sau khi kiểm CFTypeID khớp
                // AXUIElementGetTypeID() — app trả kiểu khác (String/Array…) thì bỏ
                // qua, không trap (fail-open S4).
                if rc == .success, let element = value,
                   CFGetTypeID(element) == AXUIElementGetTypeID() {
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

    // ------------------------------------------------------------- assist (AS-060)

    /// Kill-switch Trợ lý ngữ pháp (D20) — server báo qua `state["assist"]`
    /// (Snapshot) hoặc `StateUpdate { app_id: "assist" }`. Tắt ⇒ adapter
    /// **không** kiểm tra nữa (kể cả khi `assist.mode` khác "off").
    /// Mặc định = BẬT (`ViStateStore.assistEnabled`).
    private var assistKillSwitchOn: Bool { ViStateStore.shared.assistEnabled }

    /// Quy tắc bật assist (thuần, test được không cần IMKServer):
    /// 1. kill-switch D20 phải BẬT;
    /// 2. `assist.mode` phải khác "off";
    /// 3. override per-app nếu có.
    static func assistAllowed(
        killSwitchOn: Bool,
        settings: AssistSettings,
        appID: String
    ) -> Bool {
        guard killSwitchOn else { return false }
        guard settings.isOn else { return false }
        if let appOverride = settings.per_app[appID] {
            return appOverride
        }
        return true
    }

    func isAssistActive(for appID: String) -> Bool {
        Self.assistAllowed(
            killSwitchOn: assistKillSwitchOn,
            settings: assistSettings,
            appID: appID
        )
    }

    @discardableResult
    func processAssist(triggerKind: UInt32, target: TextTarget, sender: Any? = nil) -> Bool {
        // Gate thứ 2 (defense in depth): kill-switch tắt ⇒ không check, không
        // hiện gợi ý, không tự sửa, không báo IPC.
        guard assistKillSwitchOn else { return false }
        guard let engine = engine else { return false }
        // D16 (Fassist-20): crate `assist` không đọc config ⇒ **luôn** trả lỗi L6
        // ("chữ thường đầu câu"). Khi `config.auto_capitalize=true` thì IME/app
        // **đã tự viết hoa** ⇒ báo L6 là trùng chức năng ⇒ **loại** ở đây
        // (sau khi engine đã check, trước khi hiển thị/tự sửa) ⇒ áp dụng cho
        // **cả Mode 1** và **Mode 2** (cùng mảng `issues`).
        let issues = filterAutoCapitalizeConflicts(
            engine.assistCheck(),
            autoCapitalize: assistSettings.auto_capitalize
        )
        guard !issues.isEmpty else {
            candidatePanel?.hide()
            currentCandidates.removeAll()
            return false
        }

        Diagnostics.log("assist: detected \(issues.count) issue(s)")

        // R-5 (P4-0 §4.3): Mode 2 = mode "autocorrect" VÀ auto_fix "safe" VÀ
        // trigger **kết thúc câu** — cùng ngữ nghĩa cho cả 3 OS (space/, chỉ là
        // điểm kiểm tra silent — D11).
        let isSentenceTrigger = triggerKind == FFI.assistTriggerSentence
        // W5: `engine.assistCheck` luôn cắt đuôi 256 char (`ime_assist_check_instance`),
        // nên `sentenceTail` phải dùng đúng `maxChars: 256` để offset `issue.start..end`
        // không lệch khi `assistSettings.max_sentence_chars < 256`.
        let sentence = engine.sentenceTail(maxChars: 256)

        // Vòng 32 (audit F-G): parity Windows D11 — câu < 3 từ không popup/không
        // tự sửa (đoạn ngắn đang gõ dở, thiếu ngữ cảnh; Windows chặn ở
        // `check_text`, hai nền tảng còn lại phải tự chặn như nhau).
        let wordCount = sentence.split(omittingEmptySubsequences: true) { $0.isWhitespace }.count
        if wordCount < 3 {
            return false
        }

        if assistSettings.isMode2AutoFix && isSentenceTrigger {
            // R-11: không đếm được glyph ⇒ 0 ⇒ kế hoạch bị từ chối ⇒ không sửa
            // (tuyệt đối KHÔNG fallback `text.count` — sai ở VNI/TCVN3).
            if let plan = planSentenceAutoFix(sentence: sentence, issues: issues, glyphLen: { [weak engine] text in
                engine?.glyphLen(text) ?? 0
            }) {
                if applyAssistFix(plan: plan, target: target) {
                    let replGlyphs = engine.glyphLen(plan.replacement)
                    if replGlyphs > 0 {
                        assistUndo = AssistUndoState(
                            timestamp: Date(),
                            originalTail: plan.originalTail,
                            replacement: plan.replacement,
                            replacementGlyphs: replGlyphs,
                            fixedCount: plan.fixedCount
                        )
                    }
                    engine.reset()
                    Diagnostics.log("assist auto-fix applied (\(plan.fixedCount) issue(s))")
                    ipc.send(.assistReport(issues: Self.reportIssues(issues), autoFixed: UInt32(plan.fixedCount)))
                    candidatePanel?.hide()
                    currentCandidates.removeAll()
                    return true
                }
                Diagnostics.log("assist auto-fix skipped (precondition / delete failed) — fail-open")
            }
        }

        // Mode 1: Hiển thị gợi ý qua candidate panel của IMK.
        // R-6 (D11): chỉ ở **điểm hiển thị** (kết thúc câu) — mỗi dấu cách lại
        // tung panel là UI spam. Tôn trọng `hide_on_auto_fix` khi Mode 2 bật.
        let hideInMode2 = assistSettings.isMode2AutoFix && assistSettings.display.hide_on_auto_fix
        if assistSettings.isDisplayEnabled && isSentenceTrigger && !hideInMode2 {
            let candidates = Self.buildCandidates(sentence: sentence, issues: issues, engine: engine)
            if !candidates.isEmpty {
                currentCandidates = candidates
                candidatePanel?.update()
                candidatePanel?.show()
                Diagnostics.log("assist highlight shown (\(candidates.count) suggestion(s))")
                ipc.send(.assistReport(issues: Self.reportIssues(issues), autoFixed: 0))
                return true
            }
            candidatePanel?.hide()
            currentCandidates.removeAll()
        }

        return false
    }

    /// Áp dụng kế hoạch sửa (A2/A3) — **có precondition** (D12 · R-2 · R-7).
    ///
    /// Vùng sắp xoá phải **đúng** là đuôi câu mà engine biết; không xác minh
    /// được (app không cho đọc / caret ở chỗ khác) ⇒ **không sửa** (fail-safe).
    /// Chỉ xoá **trước** caret (phần sau caret không thuộc phạm vi sửa).
    @discardableResult
    func applyAssistFix(plan: SentenceFixPlan, target: TextTarget) -> Bool {
        // Q-8 (precondition "selection rỗng"): nếu người dùng đang **bôi đen**
        // text, `insert(..., NSRange(location: NSNotFound, length: 0))` = "chèn tại
        // selection" ⇒ có thể **nuốt mất** lựa chọn của họ. Windows TSF đã có
        // precondition này (`edit_session.rs`: `sel.IsEmpty()`) ⇒ macOS phải có
        // parity. Selection không rỗng ⇒ **không sửa** (fail-safe).
        guard TextVNInputController.selectionIsEmpty(target) else {
            Diagnostics.log("assist: selection not empty — skip")
            return false
        }
        guard plan.deleteGlyphs > 0 else { return false }
        guard let before = target.textBeforeCaret() else {
            Diagnostics.log("assist: precondition unavailable — skip")
            return false
        }
        guard before.hasSuffix(plan.originalTail) else {
            Diagnostics.log("assist: precondition mismatch — skip")
            return false
        }
        // W1-audit (2026-10-23, parity với Windows `verify_delete_range`): số
        // glyph engine (Unicode scalar/bảng mã xuất) phải **trùng** số grapheme
        // của đuôi đã xác minh — `deleteBackward` xoá theo **grapheme**. Lệch
        // (chuỗi kết hợp/NFD, bảng mã xuất ≠ Unicode: 1 chữ Việt = 2 glyph) ⇒
        // xoá thừa/thiếu ký tự ⇒ **không xác minh được ⇒ không sửa** (fail-safe).
        guard plan.originalTail.count == plan.deleteGlyphs,
              plan.originalTail.utf16.count == plan.deleteGlyphs else {
            Diagnostics.log("assist: glyph/grapheme mismatch — skip")
            return false
        }
        guard target.deleteSurroundingText(before: plan.deleteGlyphs, after: 0) else {
            Diagnostics.log("assist: delete failed — fail-open")
            return false
        }
        target.insert(plan.replacement, replacementRange: NSRange(location: NSNotFound, length: 0))
        return true
    }

    /// Selection phải **RỖNG** (caret rỗng) — tương đương `sel.IsEmpty()` của
    /// Windows TSF (`adapters/windows-tsf/src/edit_session.rs`).
    ///
    /// `length == 0` = không có vùng bôi đen. `NSRange(location: NSNotFound,
    /// length: 0)` (client không báo selection) cũng được coi là rỗng — nhưng
    /// đường đó đã bị chặn sớm hơn bởi `textBeforeCaret()` (trả `nil` ⇒ không
    /// sửa), nên không mở lại cửa sổ "xoá nhầm selection".
    static func selectionIsEmpty(_ target: TextTarget) -> Bool {
        target.selectionRange().length == 0
    }

    /// Dựng danh sách gợi ý **có kế hoạch sửa** (A2) — chọn gợi ý ⇒ *thay từ lỗi*
    /// (R-7), không phải chèn thêm ở caret.
    static func buildCandidates(
        sentence: String,
        issues: [AssistIssue],
        engine: ImeEngine
    ) -> [AssistCandidate] {
        var out: [AssistCandidate] = []
        for issue in issues {
            guard out.count < 3 else { break } // tối đa 3 gợi ý (P4-0 §6)
            guard !issue.suggest.isEmpty else { continue }
            guard let plan = planSentenceAutoFix(sentence: sentence, issues: [issue], glyphLen: { text in
                engine.glyphLen(text)
            }) else { continue } // lỗi chỉ gợi ý (A1) → không áp dụng được
            out.append(AssistCandidate(text: issue.suggest, plan: plan))
        }
        return out
    }

    /// Chuyển lỗi engine → wire format `AssistReport` (không có câu người dùng).
    static func reportIssues(_ issues: [AssistIssue]) -> [AssistReportIssue] {
        issues.prefix(3).map { issue in
            AssistReportIssue(
                kind: assistKindName(issue.kind),
                spanStart: issue.start,
                spanEnd: issue.end,
                message: issue.message,
                suggestions: issue.suggest.isEmpty ? [] : [issue.suggest],
                autoFixable: issue.autoFixable,
                confidence: issue.confidence
            )
        }
    }

    @discardableResult
    func handleAssistUndo(sender: Any? = nil, target: TextTarget? = nil) -> Bool {
        // KHÔNG gate bằng kill-switch: undo là HOÀN TÁC lần tự sửa đã áp —
        // chặn nó ⇒ chữ người dùng bị giữ ở dạng đã sửa mà không có đường quay
        // lại. Kill-switch chỉ chặn việc KIỂM TRA mới (D20).
        guard let undo = assistUndo, undo.isValid() else {
            assistUndo = nil
            return false
        }
        let textTarget = target ?? (sender != nil ? clientTarget(sender) : nil)
        guard let validTarget = textTarget else {
            assistUndo = nil
            return false
        }

        // R-3 (fail-safe): vùng sắp xoá phải **đúng** là text ta vừa gõ lại sau
        // khi sửa; người dùng đã gõ thêm ⇒ KHÔNG xoá (tránh nuốt chữ của họ).
        guard let before = validTarget.textBeforeCaret(), before.hasSuffix(undo.replacement) else {
            assistUndo = nil
            return false
        }
        // W1-audit (2026-10-23): cùng guard với `applyAssistFix` — undo xoá theo
        // grapheme (`deleteBackward`), Chromium glyph của engine phải trùng số
        // grapheme của replacement đã xác minh; lệch ⇒ không xoá (fail-safe).
        guard undo.replacement.count == undo.replacementGlyphs,
              undo.replacement.utf16.count == undo.replacementGlyphs else {
            assistUndo = nil
            return false
        }

        if validTarget.deleteSurroundingText(before: undo.replacementGlyphs, after: 0) {
            validTarget.insert(undo.originalTail, replacementRange: NSRange(location: NSNotFound, length: 0))
            engine?.reset()
            assistUndo = nil
            Diagnostics.log("assist undo reverted successfully")
            return true
        }

        assistUndo = nil
        return false
    }

    static func isCmdZ(mods: UInt32, keyCode: UInt16) -> Bool {
        let hasCmd = (mods & FFI.modMeta) != 0
        let noOtherMods = (mods & (FFI.modCtrl | FFI.modAlt | FFI.modShift)) == 0
        return hasCmd && noOtherMods && keyCode == 0x06
    }

    // ------------------------------------------------------------- IMK candidate panel

    public override func candidates(_ sender: Any!) -> [Any]! {
        currentCandidates.map { $0.text } as [Any]
    }

    public override func candidateSelected(_ candidateString: NSAttributedString!) {
        guard let str = candidateString?.string else { return }
        Diagnostics.log("assist: candidate selected (len=\(str.count))")
        // R-7: chọn gợi ý ⇒ **thay từ lỗi** (xoá đúng số glyph rồi gõ lại) —
        // trước đây *chèn* gợi ý ở caret ⇒ sinh "chèn thêm" + trùng lặp.
        if let candidate = currentCandidates.first(where: { $0.text == str }),
           let target = clientTarget(client()) {
            if applyAssistFix(plan: candidate.plan, target: target) {
                // R-4: sau khi sửa câu trong document, buffer câu của engine đã
                // cũ ⇒ phải reset để câu tiếp theo không bị lệch span.
                engine?.reset()
                assistUndo = nil
            } else {
                Diagnostics.log("assist: candidate apply skipped (precondition)")
            }
        }
        candidatePanel?.hide()
        currentCandidates.removeAll()
    }

    public override func candidateSelectionChanged(_ candidateString: NSAttributedString!) {
        // Callback tuỳ chọn khi di chuyển vệt chọn trong candidate panel
    }

    // MARK: - Testing hooks
    var currentAssistSettings: AssistSettings {
        get { assistSettings }
        set { assistSettings = newValue }
    }
    var currentAssistUndo: AssistUndoState? {
        get { assistUndo }
        set { assistUndo = newValue }
    }
    var currentAssistCandidates: [String] {
        currentCandidates.map { $0.text }
    }
}

/// Đếm chord thuần Swift (mirror `KeyEvent::is_chord` — P0-2 §1).
extension KeyEvent {
    func isChordSwift() -> Bool {
        mods & (FFI.modCtrl | FFI.modAlt | FFI.modMeta) != 0
    }
}
