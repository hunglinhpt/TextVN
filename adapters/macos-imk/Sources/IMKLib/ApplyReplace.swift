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
//! | `Passthrough`     | không đụng text                                   |
//!
//! Quyết định ForwardAsCommit trên macOS (khác TSF): P0-2 §4 nói "gõ insert
//! ngay, không xóa" — đó là trick range của TSF (`SetText` trên range thu hẹp).
//! IMK `insertText` không có range tương đương đáng tin; delete_count của engine
//! là số ký tự engine **chắc chắn** đã đưa vào document (buffer.rs `owned`/
//! `pending_delete`) nên xóa bằng key binding chuẩn là đúng ngữ nghĩa trên cả
//! Terminal.app lẫn iTerm2. Corpus `mac_bs_type_*`/`bug_B8_*` chốt hành vi này.

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

    /// Áp 1 outcome cho target. Trả `true` nếu đã sửa text; ném lỗi khi cơ chế
    /// xóa fail → caller fail-open (forward phím + `ime_reset`, P2-1 §12).
    public static func apply(
        _ outcome: KeyOutcome, strategy: OutputStrategy,
        target: TextTarget, marked: MarkedState
    ) throws {
        switch outcome.action {
        case .pass:
            return

        case let .replace(deleteCount, insert, preedit):
            switch strategy {
            case .preedit:
                try preeditReplace(preedit: preedit.isEmpty ? insert : preedit,
                                   target: target, marked: marked)
            case .selectionReplace:
                try selectionReplace(deleteCount: deleteCount, insert: insert,
                                     target: target, marked: marked)
            case .backspaceType, .forwardAsCommit:
                try backspaceType(deleteCount: deleteCount, insert: insert,
                                  target: target, marked: marked)
            case .passthrough:
                return
            }

        case let .commit(insert):
            // P0-2 §4: COMMIT = preedit hiện tại thành text vĩnh viễn + chèn `insert`.
            // Trên IMK: insertText với replacementRange = markedRange thay marked
            // bằng text cuối + ký tự ranh giới (bug B2 — Enter không nhân từ).
            let pending = marked.text
            let payload = pending + insert
            target.insert(payload, replacementRange: .notFound)
            marked.clear()

        case let .restore(deleteCount, insert):
            // B5 / ESC: trả lại chuỗi gõ gốc.
            switch strategy {
            case .preedit:
                // marked đang hiển thị → thay trực tiếp bằng chuỗi gốc.
                let pending = marked.text
                if pending.isEmpty {
                    try backspaceType(deleteCount: deleteCount, insert: insert,
                                      target: target, marked: marked)
                } else {
                    target.insert(insert, replacementRange: target.markedRange())
                    marked.clear()
                }
            default:
                try backspaceType(deleteCount: deleteCount, insert: insert,
                                  target: target, marked: marked)
            }
        }
    }

    // ------------------------------------------------------------- §6.1 preedit

    private static func preeditReplace(
        preedit: String, target: TextTarget, marked: MarkedState
    ) throws {
        guard !MarkedState.exceedsLimit(preedit) else {
            // B11: marked quá dài → commit-early phần hiện có, bắt đầu từ mới.
            Diagnostics.log("preedit exceeds limit (\(preedit.count) graphemes) — commit early")
            target.insert(preedit, replacementRange: .notFound)
            marked.clear()
            return
        }
        target.setMarked(preedit, selectionRange: NSRange(location: preedit.count, length: 0))
        marked.text = preedit
    }

    // ------------------------------------------------------- §6.2 selectionReplace

    private static func selectionReplace(
        deleteCount: Int, insert: String, target: TextTarget, marked: MarkedState
    ) throws {
        let sel = target.selectionRange()
        if sel.location != NSNotFound, sel.length > 0 {
            // B1: app giữ selection thật → thay selection, không gửi backspace
            // (autocomplete không bị kích hoạt lại — P2-1 §6.2).
            target.insert(insert, replacementRange: sel)
            marked.clear()
            return
        }
        // Fallback §6.3 (preset đã cảnh báo app không giữ selection).
        try backspaceType(deleteCount: deleteCount, insert: insert,
                          target: target, marked: marked)
    }

    // ------------------------------------------------------- §6.3 backspaceType

    private static func backspaceType(
        deleteCount: Int, insert: String, target: TextTarget, marked: MarkedState
    ) throws {
        let pending = marked.text.count // marked đang hiển thị cũng là text "của" engine
        var toDelete = max(0, deleteCount - pending)

        if marked.text.isEmpty == false {
            // Marked chưa commit: app chưa coi là text — nhả marked rồi xóa phần dư.
            target.unmark()
            marked.clear()
        }

        if toDelete > 0 {
            guard target.deleteBackward(count: toDelete) else {
                // §6.3 fail → lỗi cho caller fail-open (không cố inject mù).
                Diagnostics.log("deleteBackward rejected (\(toDelete)) — fail-open")
                throw ApplyError.cannotDelete
            }
            toDelete = 0
        }
        target.insert(insert, replacementRange: .notFound)
    }
}
