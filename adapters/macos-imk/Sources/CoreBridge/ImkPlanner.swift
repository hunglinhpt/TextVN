// SPDX-License-Identifier: GPL-3.0-or-later
//! ImkPlanner — SF-3 pha 3b: hỏi `textvn-session` (C ABI `ime_imk_plan`) chuỗi thao tác áp một
//! outcome engine lên IMK. Quyết định (thu hồi marked, xoá phần chữ thật, đặt marked, chèn ở đâu,
//! B11 commit-early) là MỘT nguồn trong Rust (G22); `ApplyReplace` chỉ thực thi.

import CTextVNFFI
import Foundation

public enum ImkPlanner {
    /// Một thao tác, theo đúng thứ tự thực thi.
    public enum Op: Equatable {
        /// `setMarkedText("")` + xoá marked của adapter (gỡ glyph, KHÔNG commit).
        case retractMarked
        /// `deleteBackward:` ×n.
        case deleteBackward(Int)
        /// `setMarkedText(text)` + ghi marked của adapter.
        case setMarked(String)
        /// Chèn tại caret.
        case insertAtCaret(String)
        /// Thay marked range của app.
        case insertAtMarkedRange(String)
        /// Thay selection của app (B1).
        case insertAtSelection(String)
        /// Xoá marked của adapter (không đụng app).
        case clearMarkedState
        /// Reset engine (B11 commit-early).
        case resetEngine
    }

    /// Đo đạc phía adapter.
    public struct State {
        public var marked: String
        public var markedRangePresent: Bool
        public var selectionNonEmpty: Bool
        public var preeditOverLimit: Bool

        public init(marked: String, markedRangePresent: Bool, selectionNonEmpty: Bool,
                    preeditOverLimit: Bool) {
            self.marked = marked
            self.markedRangePresent = markedRangePresent
            self.selectionNonEmpty = selectionNonEmpty
            self.preeditOverLimit = preeditOverLimit
        }
    }

    /// Kế hoạch cho `action` theo `strategy` (rawValue của `OutputStrategy`). `nil` = lỗi ABI
    /// (caller fail-open như khi app từ chối xoá).
    public static func plan(_ action: KeyOutcome.Action, strategy: Int64, state: State) -> [Op]? {
        var kind: UInt32 = UInt32(IME_IMK_PASS)
        var deleteCount = 0
        var insert = ""
        var preedit = ""
        switch action {
        case .pass:
            return []
        case let .replace(d, i, p):
            kind = UInt32(IME_IMK_REPLACE); deleteCount = d; insert = i; preedit = p
        case let .commit(i):
            kind = UInt32(IME_IMK_COMMIT); insert = i
        case let .restore(d, i):
            kind = UInt32(IME_IMK_RESTORE); deleteCount = d; insert = i
        }
        guard strategy >= 0, strategy <= 4, deleteCount >= 0 else { return nil }
        var bits: UInt32 = 0
        if state.markedRangePresent { bits |= UInt32(IME_IMK_MARKED_RANGE) }
        if state.selectionNonEmpty { bits |= UInt32(IME_IMK_SELECTION) }
        if state.preeditOverLimit { bits |= UInt32(IME_IMK_PREEDIT_OVER_LIMIT) }

        let ins = Array(insert.utf8)
        let pre = Array(preedit.utf8)
        let mk = Array(state.marked.utf8)
        var out = ime_imk_plan_v1()
        out.abi_version = UInt32(IME_IMK_ABI_VERSION)
        let rc: Int32 = ins.withUnsafeBufferPointer { ip in
            pre.withUnsafeBufferPointer { pp in
                mk.withUnsafeBufferPointer { mp in
                    var input = ime_imk_input_v1()
                    input.abi_version = UInt32(IME_IMK_ABI_VERSION)
                    input.action = kind
                    input.strategy = UInt32(strategy)
                    input.state = bits
                    input.delete_count = UInt32(clamping: deleteCount)
                    input.insert = ip.baseAddress
                    input.insert_len = ip.count
                    input.preedit = pp.baseAddress
                    input.preedit_len = pp.count
                    input.marked = mp.baseAddress
                    input.marked_len = mp.count
                    return ime_imk_plan(&input, &out)
                }
            }
        }
        guard rc == IME_OK else { return nil }

        let count = min(Int(out.count), Int(IME_IMK_MAX_OPS))
        let arena: [UInt8] = withUnsafeBytes(of: &out.arena) { Array($0) }
        let raw: [ime_imk_op_v1] = withUnsafeBytes(of: &out.ops) { b in
            Array(b.bindMemory(to: ime_imk_op_v1.self).prefix(count))
        }
        var ops: [Op] = []
        for op in raw {
            let lo = Int(op.text_off)
            let hi = lo + Int(op.text_len)
            guard hi <= arena.count else { return nil }
            let text = String(decoding: arena[lo..<hi], as: UTF8.self)
            switch op.kind {
            case UInt32(IME_IMK_OP_RETRACT_MARKED): ops.append(.retractMarked)
            case UInt32(IME_IMK_OP_DELETE_BACKWARD): ops.append(.deleteBackward(Int(op.arg)))
            case UInt32(IME_IMK_OP_SET_MARKED): ops.append(.setMarked(text))
            case UInt32(IME_IMK_OP_INSERT):
                switch op.arg {
                case UInt32(IME_IMK_AT_MARKED_RANGE): ops.append(.insertAtMarkedRange(text))
                case UInt32(IME_IMK_AT_SELECTION): ops.append(.insertAtSelection(text))
                default: ops.append(.insertAtCaret(text))
                }
            case UInt32(IME_IMK_OP_CLEAR_MARKED_STATE): ops.append(.clearMarkedState)
            case UInt32(IME_IMK_OP_RESET_ENGINE): ops.append(.resetEngine)
            default: return nil
            }
        }
        return ops
    }
}
