// SPDX-License-Identifier: GPL-3.0-or-later
//! GhostPolicy — SF-4 (P0-6 §5 ghost): cổng + nhãn xem trước chữ mờ do `textvn-session` quyết
//! (C ABI `ime_ghost_label`, G22). IMK không vẽ chữ mờ tại caret ⇒ hiện nhãn "⇥ …" trong bảng
//! ứng viên; adapter chỉ đo trạng thái (trợ lý đang hiện, ô secure…).

import CTextVNFFI
import Foundation

public enum GhostPolicy {
    public static let secure = UInt32(IME_GHOST_SECURE)
    public static let terminal = UInt32(IME_GHOST_TERMINAL)
    public static let assistVisible = UInt32(IME_GHOST_ASSIST_VISIBLE)
    public static let suggestOff = UInt32(IME_GHOST_SUGGEST_OFF)
    public static let textAfterCaret = UInt32(IME_GHOST_TEXT_AFTER_CARET)

    /// Nhãn "⇥ …" cho chữ mờ `ghost` với cổng `gates`; `nil` = ẩn (cổng chặn / chữ mờ rỗng / lỗi).
    public static func label(_ ghost: String, gates: UInt32) -> String? {
        let input = Array(ghost.utf8)
        var out = [UInt8](repeating: 0, count: input.count + 16)
        let n: Int32 = input.withUnsafeBufferPointer { ip in
            out.withUnsafeMutableBufferPointer { op in
                ime_ghost_label(ip.baseAddress, ip.count, gates, op.baseAddress, op.count)
            }
        }
        guard n > 0 else { return nil }
        return String(decoding: out[0..<Int(n)], as: UTF8.self)
    }
}
