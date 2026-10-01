// SPDX-License-Identifier: GPL-3.0-or-later
//! EventTapController — thread tap + callback (P2-2 §2/§3, mirror P1-2 §3).
//!
//! Luật bất biến của callback (P2-2 §3 — đọc từ trên xuống, match là dừng):
//! 0. `.tapDisabledByTimeout` / `.tapDisabledByUserInput` → tapEnable lại (§9).
//! 1. Marker injected → cho qua (chống loop §6).
//! 2. type ≠ keyDown → cho qua (keyUp luôn qua — §3).
//! 3. Chord Cmd hệ thống → cho qua (B6).
//! 4. Foreground app không owner=tap / secure → cho qua (§6).
//! 5. Stopwatch ≤ 2ms: handler quyết định; Transform → inject (§5) + NUỐT key.
//! 6. elapsed > 2ms → counter; >50 lần liên tiếp → self-disable (§9).
//!
//! **Không** AX query / file I/O / log disk trong callback (P2-3 §2).

import AppKit
import Carbon.HIToolbox
import CoreGraphics
import Foundation

/// Marker userData của event tự post — `TextVNInputController.InjectedMarker.userData`
/// và tap marker PHẢI cùng giá trị (đối chiếu test tap/loop-guard trên máy thật).
public enum TapMarker {
    public static let userData: Int64 = 0x5458_564E // "TXVN"
}

/// Quyết định của engine cho 1 phím tap (engine adapter tự map — P2-2 §5).
public enum TapDecision: Equatable {
    case pass
    /// xóa `deleteCount` ký tự trước caret rồi chèn `insert` (BackspaceType/
    /// SelectionReplace/ForwardAsCommit đều về đây với delete_count engine tính).
    case transform(deleteCount: Int, insert: String)
}

/// Engine handler — implement ở embedder (IMK) với ime_instance B riêng (P0-2 §3).
public protocol TapKeyHandler: AnyObject {
    /// Translate + ime_key. **Phải** <2ms (timebox §3.6).
    func tapHandle(vk: UInt32, ch: UInt32, mods: UInt32, source: CGEventSource?) -> TapDecision
    /// Foreground app có owner=tap + không secure? (đọc cache — không AX đồng bộ).
    func tapShouldProcess(appID: String) -> Bool
}

public final class EventTapController {
    /// Self-disable (P2-2 §9): >2ms liên tiếp 50 lần.
    public static let slowThresholdMs = 2.0
    public static let slowLimitStreak = 50

    // fileprivate: callback C `TapCallback.onEvent` cùng file cần đọc (MAC-032).
    fileprivate let handler: TapKeyHandler
    fileprivate var tap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var runLoop: CFRunLoop?
    private var slowStreak = 0
    private var selfDisabled = false
    private let lock = NSLock()
    /// Cache app foreground (bundle id lowercase) — callback KHÔNG được gọi
    /// `NSWorkspace.frontmostApplication` (XPC, có thể vượt 2ms — P2-2 §3.5);
    /// cập nhật bằng notification `didActivateApplication`.
    private var frontAppID: String = "unknown"
    private var appObserver: NSObjectProtocol?
    /// `stop()` đã được gọi trước khi thread kịp gán `runLoop` → thread thoát
    /// thay vì `CFRunLoopRun()` vô hạn với source đã invalidate.
    private var stopRequested = false
    /// Tap type đã create thành công (doctor hiển thị — P2-2 §4).
    public private(set) var tapTypeName: String?
    /// Số lần bị system disable rồi re-enable (doctor — P2-2 §9).
    public private(set) var reEnableCount = 0

    public init(handler: TapKeyHandler) {
        self.handler = handler
        // Seed cache từ trạng thái hiện tại (loại chính TextVN/tap).
        let ownBundleID = Bundle.main.bundleIdentifier ?? "vn.textvn.tap"
        if let app = NSWorkspace.shared.frontmostApplication,
           app.bundleIdentifier != ownBundleID {
            frontAppID = (app.bundleIdentifier ?? "unknown").lowercased()
        }
        appObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: nil
        ) { [weak self] note in
            guard let self,
                  let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            else { return }
            let id = (app.bundleIdentifier ?? "unknown").lowercased()
            self.lock.lock()
            self.frontAppID = id
            self.lock.unlock()
        }
    }

    deinit {
        if let appObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(appObserver)
        }
        // passUnretained(self) làm refcon: tap phải chết trước owner, nếu không
        // callback keyDown kế tiếp deref con trỏ đã free (crash trên tap thread).
        stop()
    }

    /// App foreground đã cache — callback event tap đọc (không XPC).
    var currentFrontAppID: String {
        lock.lock()
        defer { lock.unlock() }
        return frontAppID
    }

    public var isActive: Bool {
        lock.lock(); defer { lock.unlock() }
        return tap != nil && !selfDisabled
    }

    var isSelfDisabled: Bool {
        lock.lock(); defer { lock.unlock() }
        return selfDisabled
    }

    // ------------------------------------------------------------ lifecycle

    /// Tạo tap theo thứ tự feature-detect 3 loại (P2-2 §4 bước 0):
    /// HID → Session → Annotated. Loại nào create được dùng loại đó.
    @discardableResult
    public func start() -> Bool {
        lock.lock()
        let alreadyRunning = tap != nil
        lock.unlock()
        if alreadyRunning { return true } // start() 2 lần không rò tap/thread cũ
        let attempts: [(CGEventTapLocation, String)] = [
            (.cghidEventTap, "HID"),
            (.cgSessionEventTap, "Session"),
            (.cgAnnotatedSessionEventTap, "Annotated"),
        ]
        for (location, name) in attempts {
            if start(location: location) {
                tapTypeName = name
                return true
            }
        }
        return false
    }

    private func start(location: CGEventTapLocation) -> Bool {
        let mask: CGEventMask = (1 << CGEventType.keyDown.rawValue)
        // Option .defaultTap để NUỐT key khi transform — đây là lý do tồn tại
        // của module opt-in này (P2-2 §1); không tap nào khác được chạy (Farch §8).
        guard let port = CGEvent.tapCreate(
            tap: location,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: mask,
            callback: { proxy, type, event, refcon in
                TapCallback.onEvent(proxy: proxy, type: type, event: event, refcon: refcon)
            },
            userInfo: Unmanaged.passUnretained(self).toOpaque()
        ) else {
            return false
        }
        let source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, port, 0)
        lock.lock()
        tap = port
        runLoopSource = source
        selfDisabled = false
        slowStreak = 0
        stopRequested = false
        lock.unlock()
        let thread = Thread { [weak self] in
            let rl = CFRunLoopGetCurrent()
            // Chi giu owner trong scope ngan. Neu owner bi huy, deinit se
            // invalidate tap; thread khong giu self song vo han.
            let shouldRun: Bool = {
                guard let owner = self else { return false }
                owner.lock.lock()
                defer { owner.lock.unlock() }
                if owner.stopRequested { return false }
                owner.runLoop = rl
                return true
            }()
            if !shouldRun { return }
            CFRunLoopAddSource(rl, source, .commonModes)
            while true {
                let stopped: Bool = {
                    guard let owner = self else { return true }
                    owner.lock.lock()
                    defer { owner.lock.unlock() }
                    return owner.stopRequested
                }()
                if stopped { break }
                // CFRunLoopStop goi ngay truoc RunInMode co the bi mat tin
                // hieu. Timeout 100ms bao dam thoat duoc ca race nay.
                CFRunLoopRunInMode(CFRunLoopMode.defaultMode, 0.1, true)
            }
            CFRunLoopRemoveSource(rl, source, .commonModes)
        }
        thread.name = "vn.textvn.tap"
        thread.qualityOfService = .userInteractive
        thread.start()
        CGEvent.tapEnable(tap: port, enable: true)
        return true
    }

    public func stop() {
        lock.lock()
        defer { lock.unlock() }
        stopRequested = true
        if let port = tap {
            CGEvent.tapEnable(tap: port, enable: false)
            CFMachPortInvalidate(port)
        }
        if let rl = runLoop {
            CFRunLoopStop(rl)
        }
        runLoop = nil
        runLoopSource = nil
        tap = nil
        selfDisabled = true
    }

    func recordSlow(_ elapsedMs: Double) {
        lock.lock()
        defer { lock.unlock() }
        if elapsedMs > Self.slowThresholdMs {
            slowStreak += 1
            if slowStreak >= Self.slowLimitStreak {
                selfDisabled = true
                DiagnosticsLog.log("tap self-disable: slow streak \(slowStreak)")
            }
        } else {
            slowStreak = 0
        }
    }

    fileprivate func noteReEnable() {
        lock.lock()
        reEnableCount += 1
        lock.unlock()
    }
}

// ---------------------------------------------------------------- callback

enum TapCallback {
    static func onEvent(
        proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?
    ) -> Unmanaged<CGEvent>? {
        // Trả lại CHÍNH event nhận vào → passUnretained (callback không sở hữu nó);
        // passRetained = rò 1 CGEvent mỗi phím (audit R8 ghi đã sửa nhưng code chưa đổi).
        guard let refcon else { return Unmanaged.passUnretained(event) }
        let controller = Unmanaged<EventTapController>.fromOpaque(refcon).takeUnretainedValue()

        // 0. Tap bị system disable (callback chậm/user input) → enable lại (§9).
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if let port = controller.tap {
                CGEvent.tapEnable(tap: port, enable: true)
            }
            controller.noteReEnable()
            DiagnosticsLog.log("tap re-enabled after \(type)")
            return Unmanaged.passUnretained(event)
        }

        // 1. Marker → cho qua (chống loop, P2-2 §6).
        if event.getIntegerValueField(.eventSourceUserData) == TapMarker.userData {
            return Unmanaged.passUnretained(event)
        }

        // 2. Chỉ keyDown (keyUp luôn qua — §3).
        guard type == .keyDown else {
            return Unmanaged.passUnretained(event)
        }

        let start = DispatchTime.now()
        defer {
            let ms = Double(DispatchTime.now().uptimeNanoseconds - start.uptimeNanoseconds) / 1_000_000
            controller.recordSlow(ms)
        }

        // 3. Chord Cmd hệ thống → B6 (P2-2 §3.4).
        let flags = event.flags
        if flags.contains(.maskCommand) {
            return Unmanaged.passUnretained(event)
        }

        // 4. Foreground owner=tap + không secure (§3.5 — cache, không AX đồng bộ).
        guard !controller.isSelfDisabled else {
            return Unmanaged.passUnretained(event)
        }
        // App foreground đọc từ CACHE (notification cập nhật) — gọi
        // `NSWorkspace.frontmostApplication` ở đây là XPC, có thể vượt timebox
        // 2ms trong callback (P2-2 §3.5).
        let appID = controller.currentFrontAppID
        guard controller.handler.tapShouldProcess(appID: appID) else {
            return Unmanaged.passUnretained(event)
        }

        // 5. Handler (engine instance B) — timebox do handler tự đảm bảo.
        let mods = CGFlagMapper.toImeMods(flags)
        let decision = controller.handler.tapHandle(
            vk: UInt32(truncatingIfNeeded: event.getIntegerValueField(.keyboardEventKeycode)),
            ch: TapTranslator.shared.character(
                for: UInt16(truncatingIfNeeded: event.getIntegerValueField(.keyboardEventKeycode)),
                shift: flags.contains(.maskShift),
                option: flags.contains(.maskAlternate),
                control: flags.contains(.maskControl)
            ),
            mods: mods,
            source: nil
        )

        switch decision {
        case .pass:
            return Unmanaged.passUnretained(event)
        case let .transform(deleteCount, insert):
            // §5 inject: xóa + chèn kèm marker, rồi NUỐT key gốc.
            TapInjector.inject(deleteCount: deleteCount, insert: insert)
            return nil
        }
    }
}

/// CGEventFlags → IME_MOD_* (P0-2 §1) — map đúng bit, không lấy raw byte.
enum CGFlagMapper {
    static func toImeMods(_ flags: CGEventFlags) -> UInt32 {
        var mods: UInt32 = 0
        if flags.contains(.maskShift) { mods |= 0x1 }
        if flags.contains(.maskControl) { mods |= 0x2 }
        if flags.contains(.maskAlternate) { mods |= 0x4 }
        if flags.contains(.maskCommand) { mods |= 0x10 } // Cmd = IME_MOD_META
        if flags.contains(.maskAlphaShift) { mods |= 0x20 }
        if flags.contains(.maskSecondaryFn) { mods |= 0x40 }
        return mods
    }
}

// --------------------------------------------------------------- translate

/// UCKeyTranslate cho tap (không có NSEvent — dịch trực tiếp từ CGEvent).
final class TapTranslator {
    static let shared = TapTranslator()
    private var deadKeyState: UInt32 = 0

    func character(for keyCode: UInt16, shift: Bool, option: Bool, control: Bool) -> UInt32 {
        guard let layoutData = LayoutCache.current else { return 0 }
        var chars = [UniChar](repeating: 0, count: 4)
        var len = 0
        var modifiers: UInt32 = 0
        if shift { modifiers |= (UInt32(shiftKey) >> 8) }
        if option { modifiers |= (UInt32(optionKey) >> 8) }
        if control { modifiers |= (UInt32(controlKey) >> 8) }
        let status: OSStatus = layoutData.withUnsafeBytes { raw in
            guard let base = raw.baseAddress else { return OSStatus(paramErr) }
            let layoutPtr = base.bindMemory(to: UCKeyboardLayout.self, capacity: 1)
            return withUnsafeMutablePointer(to: &deadKeyState) { dead in
                UCKeyTranslate(
                    layoutPtr, keyCode, UInt16(kUCKeyActionDown), modifiers,
                    UInt32(LMGetKbdType()), UInt32(kUCKeyTranslateNoDeadKeysMask),
                    dead, 4, &len, &chars
                )
            }
        }
        guard status == noErr, len > 0 else { return 0 }
        let string = String(utf16CodeUnits: chars, count: len)
        return string.unicodeScalars.first?.value ?? 0
    }
}

enum LayoutCache {
    static var current: Data? {
        let source = TISCopyCurrentKeyboardLayoutInputSource().takeRetainedValue()
        guard let ptr = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
            return nil
        }
        return Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue() as Data
    }
}

// --------------------------------------------------------------- log shim

/// Log nội bộ tap — chỉ trạng thái + độ dài, không text (S2).
/// Embedder (IMK) cài sink để đổ vào Diagnostics chung.
public enum DiagnosticsLog {
    private static let lock = NSLock()
    private static var sink: ((String) -> Void)?

    public static func install(_ handler: @escaping (String) -> Void) {
        lock.lock(); defer { lock.unlock() }
        sink = handler
    }

    public static func log(_ message: String) {
        lock.lock(); defer { lock.unlock() }
        sink?(message)
        NSLog("textvn-tap: %@", message)
    }
}
