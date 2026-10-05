// SPDX-License-Identifier: GPL-3.0-or-later
//! AppInfo — nguồn version hiển thị **duy nhất** của TextVN.app (R2 finding 2:
//! trước đây "0.1.0" hardcode rải rác ở About/Settings/Health → lệch sau bump).

import Foundation

enum AppInfo {
    /// CFBundleShortVersionString của bundle đang chạy; fallback hằng khi
    /// chạy trong swift test (Bundle.main không có Info.plist app).
    static let displayVersion: String =
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        ?? "0.2.25"
}
