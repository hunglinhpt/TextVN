// SPDX-License-Identifier: GPL-3.0-or-later
//! FieldDetect — AX field-role detect cho macOS (P2-3 §2, mirror P1-3 §2).
//!
//! Tách 2 lớp:
//! 1. `FieldRules.map` — hàm **thuần** từ AX snapshot → (role, secure): unit test
//!    được bằng dict giả lập (MAC-030), không đụng AX thật.
//! 2. `FieldDetect` — cache theo pid (TTL 2s, P2-3 §2 budget) + gather async.
//!    **Không** AX query đồng bộ nào trong `handle()` (MAC-031).

import AppKit
import CoreBridge
import Foundation

/// Field role FFI (P0-2 §1 `IME_FIELD_*`) — chuỗi JSON dùng ở appdb.
public enum FieldRole {
    public static let unknown: UInt32 = 0
    public static let body: UInt32 = 1
    public static let editbox: UInt32 = 2
    public static let addressBar: UInt32 = 3
    public static let search: UInt32 = 4
    public static let combo: UInt32 = 5
    public static let candidate: UInt32 = 6
    public static let textarea: UInt32 = 7
    public static let web: UInt32 = 8
    public static let terminal: UInt32 = 9
    public static let secure: UInt32 = 10

    public static func name(_ role: UInt32) -> String {
        switch role {
        case body: return "body"
        case editbox: return "editbox"
        case addressBar: return "address_bar"
        case search: return "search"
        case combo: return "combo"
        case candidate: return "candidate"
        case textarea: return "textarea"
        case web: return "web"
        case terminal: return "terminal"
        case secure: return "secure"
        default: return "unknown"
        }
    }
}

/// Snapshot các AX attribute cần cho R1–R10 (đọc từ AXUIElement thật hoặc mock test).
public struct AXSnapshot {
    public var role: String?
    public var subrole: String?
    public var placeholder: String?
    public var description: String?
    public var title: String?
    public var identifier: String?
    public var secureInputMode: Bool // app đang ở secure input mode (S8)

    public init(
        role: String? = nil, subrole: String? = nil, placeholder: String? = nil,
        description: String? = nil, title: String? = nil, identifier: String? = nil,
        secureInputMode: Bool = false
    ) {
        self.role = role
        self.subrole = subrole
        self.placeholder = placeholder
        self.description = description
        self.title = title
        self.identifier = identifier
        self.secureInputMode = secureInputMode
    }
}

public enum FieldRules {
    /// R1–R10 (P2-3 §2, ưu tiên giảm dần — match đầu là dừng).
    public static func map(_ ax: AXSnapshot) -> (role: UInt32, secure: Bool) {
        // R1 — secure (S3: không override được)
        if ax.secureInputMode || ax.subrole == "AXSecureTextField" {
            return (FieldRole.secure, true)
        }

        // R2 — text field với placeholder/description/title gợi URL/search
        if ax.role == "AXTextField", let hint = firstHint(ax) {
            if hint.contains("address") || hint.contains("url") {
                return (FieldRole.addressBar, false)
            }
            if hint.contains("search") {
                return (FieldRole.search, false)
            }
        }

        // R3 — subrole search field
        if ax.subrole == "AXSearchField" {
            return (FieldRole.search, false)
        }

        // R4 — combo / popup
        if ax.role == "AXComboBox" || ax.role == "AXPopUpButton" {
            return (FieldRole.combo, false)
        }

        // R5 — candidate (Excel/Numbers grid cell; bundle do caller check qua preset)
        if ax.role == "AXTextField", ax.identifier?.lowercased().contains("grid") == true {
            return (FieldRole.candidate, false)
        }

        // R6 — terminal (AXTextArea + bundle check do caller; identifier chứa "term")
        if ax.role == "AXTextArea", isTerminalHint(ax) {
            return (FieldRole.terminal, false)
        }

        // R7 — text area
        if ax.role == "AXTextArea" {
            return (FieldRole.textarea, false)
        }

        // R8 — web area (caller điền khi ancestor là AXWebArea)
        if ax.role == "AXWebArea" {
            return (FieldRole.web, false)
        }

        // R9 — text field thường
        if ax.role == "AXTextField" {
            return (FieldRole.editbox, false)
        }

        // R10 — không query được
        return (FieldRole.unknown, false)
    }

    /// Heuristic R6 phụ: identifier/title của terminal app.
    private static func isTerminalHint(_ ax: AXSnapshot) -> Bool {
        let hints = [ax.identifier, ax.title, ax.description].compactMap { $0?.lowercased() }
        return hints.contains { $0.contains("term") }
    }

    private static func firstHint(_ ax: AXSnapshot) -> String? {
        [ax.placeholder, ax.description, ax.title]
            .compactMap { $0?.lowercased() }
            .first(where: { !$0.isEmpty })
    }
}

/// Context đã detect — đầu vào dựng `ime_context_v1` (P2-3 §5).
public struct FieldContext: Equatable {
    public var appID: String
    public var role: UInt32
    public var secure: Bool

    public init(appID: String, role: UInt32, secure: Bool) {
        self.appID = appID
        self.role = role
        self.secure = secure
    }
}

/// Cache + gather. 1 instance cho cả IMK lẫn tap worker (P2-2 §2).
public final class FieldDetect {
    /// TTL cache (P2-3 §2: 2s/pid).
    public static let ttl: TimeInterval = 2.0

    private struct Entry {
        let context: FieldContext
        let at: Date
    }

    private var cache: [pid_t: Entry] = [:]
    private let lock = NSLock()

    public init() {}

    /// Context đã cache (không query AX — dùng trong `handle()`).
    public func cached(for pid: pid_t) -> FieldContext? {
        lock.lock()
        defer { lock.unlock() }
        guard let entry = cache[pid], Date().timeIntervalSince(entry.at) < Self.ttl else {
            return nil
        }
        return entry.context
    }

    /// Lưu 1 context (gọi từ gather async / AXObserver).
    public func store(_ context: FieldContext, for pid: pid_t, at date: Date = Date()) {
        lock.lock()
        defer { lock.unlock() }
        cache[pid] = Entry(context: context, at: date)
    }

    /// Invalidate khi `didActivateApplication` / focus change (P2-3 §2).
    public func invalidate(pid: pid_t) {
        lock.lock()
        defer { lock.unlock() }
        cache.removeValue(forKey: pid)
    }

    /// Chuẩn hóa app id: bundle id lowercase; nil → executable basename (P2-3 §1).
    public static func normalizeAppID(bundleID: String?, executableName: String?) -> String {
        if let bundleID, !bundleID.isEmpty {
            return bundleID.lowercased()
        }
        if let executableName, !executableName.isEmpty {
            return (executableName as NSString).lastPathComponent.lowercased()
        }
        return "unknown"
    }
}
