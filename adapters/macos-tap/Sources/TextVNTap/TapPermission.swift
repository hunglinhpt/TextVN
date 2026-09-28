// SPDX-License-Identifier: GPL-3.0-or-later
//! TapPermission — quyền Accessibility cho tap opt-in (P2-2 §4).
//!
//! - Tap là opt-in: Settings bật cho app cụ thể → mới xin quyền (P2-2 §7).
//! - Từ chối quyền = tắt tính năng, IMK vẫn chạy (P2-2 §1).
//! - Poll `AXIsProcessTrusted` 5s (không private notification — P2-2 §4).

import ApplicationServices
import Foundation

public final class TapPermission {
    public static let pollInterval: TimeInterval = 5.0

    private var pollTimer: Timer?
    private let lock = NSLock()
    private var lastGranted: Bool?

    /// Trạng thái quyền hiện tại (không prompt).
    public static func isGranted() -> Bool {
        AXIsProcessTrusted()
    }

    /// Xin quyền + prompt System Settings (chỉ gọi từ Settings user-driven — §7).
    public static func promptIfNeeded() -> Bool {
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
    }

    /// Bắt đầu poll — `onChange` chạy khi trạng thái thay đổi (grant/revoke).
    public func startPolling(onChange: @escaping (Bool) -> Void) {
        lock.lock()
        defer { lock.unlock() }
        guard pollTimer == nil else { return }
        let timer = Timer(timeInterval: Self.pollInterval, repeats: true) { [weak self] _ in
            guard let self else { return }
            let granted = Self.isGranted()
            let changed = self.lock.withLock { () -> Bool in
                let wasGranted = self.lastGranted
                self.lastGranted = granted
                return wasGranted != granted
            }
            if changed {
                DiagnosticsLog.log("tap permission changed: granted=\(granted)")
                onChange(granted)
            }
        }
        RunLoop.main.add(timer, forMode: .common)
        pollTimer = timer
    }

    public func stopPolling() {
        lock.lock()
        defer { lock.unlock() }
        pollTimer?.invalidate()
        pollTimer = nil
    }
}

private extension NSLock {
    func withLock<T>(_ body: () -> T) -> T {
        lock()
        defer { unlock() }
        return body()
    }
}
