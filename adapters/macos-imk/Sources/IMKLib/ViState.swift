// SPDX-License-Identifier: GPL-3.0-or-later
// ViState.swift — trạng thái bật/tắt tiếng Việt phía IMK (P0-3 §4, ipc.v1.md).
//
// Một quy ước `app_id` toàn cục `"*"` (`IpcMessage.globalAppID`); bản IMK cũ gửi
// `""` — chuẩn hoá về `"*"`. Thứ tự áp dụng: override per-app → mặc định toàn cục.
// Tách khỏi `TextVNInputController` để test được không cần IMKServer (CI arm64/x86_64).

import Foundation

public struct ViState: Equatable {
    /// Mặc định toàn cục (`app_id == "*"`).
    public private(set) var globalEnabled: Bool
    /// Override theo `app_id` (bundle id lowercase / tên exe — P2-3 §1).
    public private(set) var appOverrides: [String: Bool]

    public init(globalEnabled: Bool = true, appOverrides: [String: Bool] = [:]) {
        self.globalEnabled = globalEnabled
        self.appOverrides = appOverrides
    }

    /// `""` (IMK bản cũ) quy về `"*"` — cùng quy ước với server (review R3 F3-1).
    public static func normalizedAppID(_ appID: String) -> String {
        appID.isEmpty ? IpcMessage.globalAppID : appID
    }

    /// Áp `Snapshot.state` (gồm `"*"` + per-app) sau khi kết nối.
    public mutating func apply(snapshot state: [String: Bool]) {
        for (appID, enabled) in state {
            apply(stateUpdate: appID, enabled: enabled)
        }
    }

    public mutating func apply(stateUpdate appID: String, enabled: Bool) {
        let key = Self.normalizedAppID(appID)
        if key == IpcMessage.globalAppID {
            globalEnabled = enabled
        } else {
            appOverrides[key] = enabled
        }
    }

    /// Trạng thái hiệu dụng cho app đang gõ.
    public func enabled(for appID: String?) -> Bool {
        guard let appID, !appID.isEmpty else { return globalEnabled }
        return appOverrides[appID] ?? globalEnabled
    }
}

/// R2-44: trạng thái bật/tắt DÙNG CHUNG cho cả tiến trình IMK. Mỗi app client có
/// một controller (và một kết nối IPC) riêng; khi TextVN.app không chạy, toggle
/// Ctrl+Shift ở app này phải có hiệu lực ở mọi app — và controller tạo sau không
/// được đọc lại `config.enabled` cũ. Chỉ dùng trên main thread (như controller).
public final class ViStateStore {
    public static let shared = ViStateStore()

    public var state: ViState
    /// Toggle toàn cục xảy ra lúc IPC offline — chưa tới TextVN.app (nguồn ghi
    /// `config.enabled`). Gửi lại ở Snapshot kế tiếp thay vì để "*" cũ đè mất.
    public private(set) var pendingGlobalSync = false
    private var seeded = false

    public init(state: ViState = ViState()) {
        self.state = state
    }

    /// Controller đầu tiên của tiến trình khởi tạo từ `config.enabled`; các lần sau
    /// giữ nguyên trạng thái đang có.
    public func seedIfNeeded(globalEnabled: Bool) {
        guard !seeded else { return }
        seeded = true
        state = ViState(globalEnabled: globalEnabled, appOverrides: state.appOverrides)
    }

    /// Đảo trạng thái toàn cục; `online == false` → đánh dấu cần đồng bộ với server.
    @discardableResult
    public func toggleGlobal(online: Bool) -> Bool {
        seeded = true
        let newValue = !state.globalEnabled
        state.apply(stateUpdate: IpcMessage.globalAppID, enabled: newValue)
        if !online {
            pendingGlobalSync = true
        }
        return newValue
    }

    /// Áp Snapshot của server. Có toggle offline đang chờ → GIỮ giá trị toàn cục
    /// hiện tại (bỏ "*" của server, vẫn áp per-app) và trả `true`: caller gửi
    /// `ToggleViEn("*", state.globalEnabled)` để server ghi config + broadcast.
    public func applySnapshot(_ snapshot: [String: Bool]) -> Bool {
        seeded = true
        guard pendingGlobalSync else {
            state.apply(snapshot: snapshot)
            return false
        }
        pendingGlobalSync = false
        let perApp = snapshot.filter {
            ViState.normalizedAppID($0.key) != IpcMessage.globalAppID
        }
        state.apply(snapshot: perApp)
        return true
    }
}
