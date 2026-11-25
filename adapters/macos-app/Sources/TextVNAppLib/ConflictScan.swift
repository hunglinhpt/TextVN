// SPDX-License-Identifier: GPL-3.0-or-later
// ConflictScan.swift — E70: phát hiện bộ gõ tiếng Việt có sẵn của Apple (Telex / Simple
// Telex / VNI / VIQR) đang BẬT song song với TextVN. Cùng luật với
// `cli/src/doctor.rs::detect_apple_vi_input_sources` (W_CONFLICT_APPLE_VI_IM).
// Chỉ đọc ID nguồn nhập (input source) — không đọc nội dung gõ (G8).

import Carbon.HIToolbox
import Foundation

public enum ConflictScan {
    /// Thuần (test được): ID input source → nhãn chế độ gõ Apple Vietnamese đang bật.
    /// Bundle gốc `com.apple.inputmethod.VietnameseIM` không phải một chế độ gõ → bỏ qua.
    public static func appleVietnameseModes(_ ids: [String]) -> [String] {
        var out: [String] = []
        for raw in ids {
            let lower = raw.lowercased()
            guard lower.hasPrefix("com.apple.inputmethod.vietnamese") else { continue }
            let label: String
            if lower.hasSuffix("simpletelex") {
                label = "Apple Vietnamese Simple Telex"
            } else if lower.hasSuffix("telex") {
                label = "Apple Vietnamese Telex"
            } else if lower.hasSuffix("vni") {
                label = "Apple Vietnamese VNI"
            } else if lower.hasSuffix("viqr") {
                label = "Apple Vietnamese VIQR"
            } else {
                continue
            }
            if !out.contains(label) {
                out.append(label)
            }
        }
        return out
    }

    /// ID mọi input source đang BẬT (Text Input Sources; `includeAllInstalled = false`).
    public static func enabledInputSourceIDs() -> [String] {
        guard
            let sources = TISCreateInputSourceList(nil, false)?.takeRetainedValue()
                as? [TISInputSource]
        else { return [] }
        var ids: [String] = []
        for src in sources {
            guard let ptr = TISGetInputSourceProperty(src, kTISPropertyInputSourceID) else {
                continue
            }
            ids.append(Unmanaged<CFString>.fromOpaque(ptr).takeUnretainedValue() as String)
        }
        return ids
    }
}
