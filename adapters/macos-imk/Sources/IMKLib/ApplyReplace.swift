// SPDX-License-Identifier: GPL-3.0-or-later
//! ApplyReplace — **nơi duy nhất** sửa text trên macOS (P2-1 §6, P0-2 §4).
//!
//! Mapping strategy → cơ chế (P0-2 §4 + P2-1 §6):
//! | Strategy          | Cơ chế mac                                        |
//! |-------------------|---------------------------------------------------|
//! | `Preedit`         | marked text (`setMarkedText`) — §6.1              |
//! | `SelectionReplace`| thay selection, KHÔNG backspace (B1) — §6.2       |
//! | `BackspaceType`   | `deleteBackward:` ×n + chèn — §6.3 (a)            |
//! | `ForwardAsCommit` | giống BackspaceType — xem ghi chú §6.3-below      |
//! | `Passthrough`     | không đụng text; REPLACE + preedit rỗng (Mode EN) như `BackspaceType` |
//!
//! ## Quy ước đơn vị (review R1 F7 — P2-1 §6.4)
//! - `delete_count` của engine đếm **code point** (Rust `char` = scalar).
//! - So sánh với `delete_count` → luôn dùng `unicodeScalars.count`.
//! - Lập `NSRange` → luôn dùng `utf16.count`.
//! - Giới hạn B11 đếm **grapheme** (`String.count`) — theo đúng spec.
//!
//! ## Công thức cốt lõi (review R1 F3/F4)
//! Engine: `owned = passed(chữ thật đã vào document) + marked(đang hiển thị)`.
//! `delete_count = owned` ⇒ phần "chữ thật" cần xóa thật = `delete_count - markedCP`,
//! phần marked xử lý bằng cách **thu hồi composition** (`setMarkedText("")` —
//! AppKit gỡ glyph mà KHÔNG commit; `unmarkText()` thì NGƯỢC LẠI: chấp nhận
//! marked thành text thật — dùng nhầm là nhân đôi chữ).
//!
//! SF-3 pha 3b (G22): QUYẾT ĐỊNH chuỗi thao tác là MỘT nguồn trong `textvn_session::imk`
//! (C ABI `ime_imk_plan`, qua `ImkPlanner`); file này chỉ đo trạng thái IMK (marked range,
//! selection, B11 đếm grapheme) và THỰC THI thao tác. Bản Swift cũ ở test target làm oracle.
//!
//! Quyết định ForwardAsCommit trên macOS (khác TSF): P0-2 §4 nói "gõ ngay
//! không xóa" — đó là trick range của TSF (`SetText` trên range thu hẹp).
//! IMK `insertText` không có range tương đương đáng tin; delete_count của engine
//! là số ký tự engine **chắc chắn** sở hữu nên xóa bằng key binding chuẩn là
//! đúng ngữ nghĩa trên cả Terminal.app lẫn iTerm2. Corpus `mac_bs_type_*`/`bug_B8_*` chốt hành vi.

import CoreBridge
import Foundation

public enum OutputStrategy: Int64 {
    case preedit = 0
    case backspaceType = 1
    case selectionReplace = 2
    case forwardAsCommit = 3
    case passthrough = 4

    public init?(raw: Int64) {
        self.init(rawValue: raw)
    }
}

public enum ApplyReplace {
    public enum ApplyError: Error {
        case cannotDelete
        case rejected
    }

    /// Áp 1 outcome cho target. Ném lỗi khi cơ chế xóa fail (hoặc ABI lỗi) → caller fail-open
    /// (forward phím + `ime_reset`, P2-1 §12).
    ///
    /// `onReset` — callback gọi khi kế hoạch yêu cầu reset engine (commit-early B11). Tách khỏi
    /// caller để không kéo engine vào đây.
    public static func apply(
        _ outcome: KeyOutcome, strategy: OutputStrategy,
        target: TextTarget, marked: MarkedState,
        onReset: () -> Void = {}
    ) throws {
        if case .pass = outcome.action { return }
        // Đo đạc IMK (chỉ hỏi app khi luật cần — mỗi lần hỏi là một lời gọi tới client).
        var overLimit = false
        var selection = false
        var markedRangePresent = false
        switch outcome.action {
        case let .replace(_, _, preedit):
            if strategy == .preedit, !preedit.isEmpty {
                overLimit = MarkedState.exceedsLimit(preedit)
            }
            if strategy == .selectionReplace {
                let sel = target.selectionRange()
                selection = sel.location != NSNotFound && sel.length > 0
            }
        case .commit:
            if !marked.isEmpty {
                markedRangePresent = target.markedRange().location != NSNotFound
            }
        case .pass, .restore:
            break
        }
        let state = ImkPlanner.State(
            marked: marked.text, markedRangePresent: markedRangePresent,
            selectionNonEmpty: selection, preeditOverLimit: overLimit)
        guard let ops = ImkPlanner.plan(outcome.action, strategy: strategy.rawValue, state: state)
        else {
            Diagnostics.log("ime_imk_plan failed — fail-open")
            throw ApplyError.rejected
        }
        try execute(ops, target: target, marked: marked, onReset: onReset)
    }

    /// Thực thi thao tác theo thứ tự (xem công thức cốt lõi ở header file).
    static func execute(
        _ ops: [ImkPlanner.Op], target: TextTarget, marked: MarkedState,
        onReset: () -> Void
    ) throws {
        for op in ops {
            switch op {
            case .retractMarked:
                // setMarkedText("") gỡ glyph marked mà KHÔNG commit (unmarkText() sẽ commit —
                // review R1 F4).
                target.setMarked("", selectionRange: NSRange(location: 0, length: 0))
                marked.clear()
            case let .deleteBackward(n):
                guard target.deleteBackward(count: n) else {
                    Diagnostics.log("deleteBackward rejected (\(n)) — fail-open")
                    throw ApplyError.cannotDelete
                }
            case let .setMarked(text):
                target.setMarked(text, selectionRange: NSRange(location: text.utf16Count, length: 0))
                marked.set(text)
            case let .insertAtCaret(text):
                target.insert(text, replacementRange: .notFound)
            case let .insertAtMarkedRange(text):
                target.insert(text, replacementRange: target.markedRange())
            case let .insertAtSelection(text):
                target.insert(text, replacementRange: target.selectionRange())
            case .clearMarkedState:
                marked.clear()
            case .resetEngine:
                // B11 (P2-1 §7): preedit vượt 8 grapheme đã được commit thành text thật.
                Diagnostics.log("preedit exceeds limit — committed early, reset engine")
                onReset()
            }
        }
    }
}
