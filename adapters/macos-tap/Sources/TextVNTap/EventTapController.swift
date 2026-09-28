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

    private let handler: TapKeyHandler
    private var tap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var runLoop: CFRunLoop?
    private var slowStreak = 0
    private var selfDisabled = false
    private let lock = NSLock()
    /// Tap type đã create thành công (doctor hiển thị — P2-2 §4).
    public private(set) var tapTypeName: String?
    /// Số lần bị system disable rồi re-enable (doctor — P2-2 §9).
    public private(set) var reEnableCount = 0

    public init(handler: TapKeyHandler) {
        self.handler = handler
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
        let attempts: [(CGEventTapLocation, String)] = [
            (.cgHIDEventTap, "HID"),
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
        lock.unlock()
        let thread = Thread { [weak self] in
            guard let self else { return }
            let rl = CFRunLoopGetCurrent()
            self.lock.lock()
            self.runLoop = rl
            self.lock.unlock()
            CFRunLoopAddSource(rl, source, .commonModes)
            CFRunLoopRun()
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
        guard let refcon else { return Unmanaged.passRetained(event) }
        let controller = Unmanaged<EventTapController>.fromOpaque(refcon).takeUnretainedValue()

        // 0. Tap bị system disable (callback chậm/user input) → enable lại (§9).
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            CGEvent.tapEnable(tap: proxy, enable: true)
            controller.noteReEnable()
            DiagnosticsLog.log("tap re-enabled after \(type)")
            return Unmanaged.passRetained(event)
        }

        // 1. Marker → cho qua (chống loop, P2-2 §6).
        if event.getIntegerValueField(.eventSourceUserData) == TapMarker.userData {
            return Unmanaged.passRetained(event)
        }

        // 2. Chỉ keyDown (keyUp luôn qua — §3).
        guard type == .keyDown else {
            return Unmanaged.passRetained(event)
        }

        let start = DispatchTime.now()
        defer {
            let ms = Double(DispatchTime.now().uptimeNanoseconds - start.uptimeNanoseconds) / 1_000_000
            controller.recordSlow(ms)
        }

        // 3. Chord Cmd hệ thống → B6 (P2-2 §3.4).
        let flags = event.flags
        if flags.contains(.maskCommand) {
            return Unmanaged.passRetained(event)
        }

        // 4. Foreground owner=tap + không secure (§3.5 — cache, không AX đồng bộ).
        guard !controller.isSelfDisabled else {
            return Unmanaged.passRetained(event)
        }
        let frontApp = NSWorkspace.shared.frontmostApplication
        let appID = (frontApp?.bundleIdentifier ?? "unknown").lowercased()
        guard controller.handler.tapShouldProcess(appID: appID) else {
            return Unmanaged.passRetained(event)
        }

        // 5. Handler (engine instance B) — timebox do handler tự đảm bảo.
        let mods = CGFlagMapper.toImeMods(flags)
        let decision = controller.handler.tapHandle(
            vk: UInt32(bitPattern: event.getIntegerValueField(.keyboardEventKeycode)),
            ch: TapTranslator.shared.character(
                for: UInt16(bitPattern: event.getIntegerValueField(.keyboardEventKeycode)),
                shift: flags.contains(.maskShift),
                option: flags.contains(.maskAlternate),
                control: flags.contains(.maskControl)
            ),
            mods: mods,
            source: nil
        )

        switch decision {
        case .pass:
            return Unmanaged.passRetained(event)
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
        if shift { modifiers |= UInt32(alphaShift) }
        if option { modifiers |= UInt32(optionKey) }
        if control { modifiers |= UInt32(controlKey) }
        let status = withUnsafeMutablePointer(to: &deadKeyState) { dead in
            UCKeyTranslate(
                layoutData, keyCode, UInt16(kUCKeyActionDown), modifiers,
                UInt32(LMGetKbdType()), UInt32(kUCKeyTranslateNoDeadKeysMask),
                dead, 4, &len, &chars
            )
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
