// SPDX-License-Identifier: GPL-3.0-or-later
//! OwnerRule — tránh xử lý đôi IMK ∩ tap (P2-2 §6, mirror P1-2 §6).
//!
//! owner = "imk" (mặc định) → tap KHÔNG xử lý app này (callback §3.4 cho qua).
//! owner = "tap"            → IMK enabled=0 cho context đó (P2-1 §5) — mọi phím PASS.
//!
//! Cả 2 đọc cùng `state.json` + IPC `StateUpdate` (P0-3 §4) — 1 nguồn sự thật.

import Foundation

public enum EngineOwner: String {
    case imk
    case tap

    public static func parse(_ raw: String?) -> EngineOwner {
        raw == "tap" ? .tap : .imk // default imk — fail-safe về đường chính
    }
}

/// Bảng owner hiện tại: app_id → owner. Nạp từ appdb/state (embedder sync).
public final class OwnerRule {
    private var owners: [String: EngineOwner] = [:]
    private let lock = NSLock()

    public init() {}

    /// Sync từ snapshot (appdb preset `engine_owner` + user override đã merge).
    public func update(owners: [String: EngineOwner]) {
        lock.lock()
        defer { lock.unlock() }
        self.owners = owners
    }

    /// App id đã normalize lowercase (P2-3 §1).
    public func owner(of appID: String) -> EngineOwner {
        lock.lock()
        defer { lock.unlock() }
        return owners[appID.lowercased()] ?? .imk
    }

    /// Tap xử lý app này? (dùng trong `tapShouldProcess` — §3.5)
    public func tapOwns(_ appID: String) -> Bool {
        owner(of: appID) == .tap
    }

    /// IMK xử lý app này? (owner=tap → IMK disabled cho context đó — §6)
    public func imkOwns(_ appID: String) -> Bool {
        owner(of: appID) == .imk
    }
}
