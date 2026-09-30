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
