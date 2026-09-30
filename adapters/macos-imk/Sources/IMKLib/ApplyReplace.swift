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

    /// Áp 1 outcome cho target. Trả `true` nếu đã sửa text; ném lỗi khi cơ chế
    /// xóa fail → caller fail-open (forward phím + `ime_reset`, P2-1 §12).
    ///
    /// `onReset` — callback gọi khi adapter tự quyết định reset engine
    /// (commit-early B11). Tách khỏi caller để không kéo engine vào đây.
    public static func apply(
        _ outcome: KeyOutcome, strategy: OutputStrategy,
        target: TextTarget, marked: MarkedState,
        onReset: () -> Void = {}
    ) throws {
        switch outcome.action {
        case .pass:
            return

        case let .replace(deleteCount, insert, preedit):
            switch strategy {
            case .preedit:
                if preedit.isEmpty {
                    // F5: REPLACE + preedit rỗng = thay đổi text THẬT (macro,
                    // auto-capitalize) — KHÔNG phải composition.
                    try backspaceType(deleteCount: deleteCount, insert: insert,
                                      target: target, marked: marked)
                } else {
                    try preeditReplace(deleteCount: deleteCount, preedit: preedit,
                                       target: target, marked: marked, onReset: onReset)
                }
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
            try commit(insert: insert, target: target, marked: marked)

        case let .restore(deleteCount, insert):
            // B5 / ESC: trả lại chuỗi gõ gốc. Unified: thu hồi marked (nếu có)
            // rồi xóa phần chữ thật + chèn raw — đúng cho mọi strategy.
            try backspaceType(deleteCount: deleteCount, insert: insert,
                              target: target, marked: marked)
        }
    }

    // ------------------------------------------------------------- §6.1 preedit

    private static func preeditReplace(
        deleteCount: Int, preedit: String, target: TextTarget,
        marked: MarkedState, onReset: () -> Void
    ) throws {
        guard !MarkedState.exceedsLimit(preedit) else {
            // B11 (P2-1 §7): marked vượt 8 grapheme → commit TOÀN BỘ preedit mới
            // (gồm phím vừa gõ) thành text thật + reset engine — từ mới bắt đầu
            // sạch; không reset thì owned lệch mãi (review R1 F6). Chỉ commit phần
            // marked cũ rồi nuốt phím = MẤT ký tự thứ 9 (MAC-032, lần đầu test
            // chạy thật trên CI).
            Diagnostics.log("preedit exceeds limit (\(preedit.count) graphemes) — commit early")
            try commitEarly(deleteCount: deleteCount, text: preedit,
                            target: target, marked: marked)
            onReset()
            return
        }

        // F3: delete_count gồm cả passed prefix (chữ THẬT đã nằm trong document
        // từ các phím PASS trước khi từ activate) — phải xóa thật, không thể
        // "thay bằng marked".
        let real = max(0, deleteCount - marked.scalarCount)
        if real > 0 {
            guard target.deleteBackward(count: real) else {
                throw ApplyError.cannotDelete
            }
        }
        target.setMarked(
            preedit,
            selectionRange: NSRange(location: preedit.utf16Count, length: 0)
        )
        marked.set(preedit)
    }

    /// Commit marked đang hiển thị thành text thật (dùng cho commit-early B11).
    /// Chuyển `text` (preedit mới) thành text thật. Còn marked range → thay range
    /// đó, chỉ xóa phần chữ thật ngoài marked. Không còn marked range (app đã tự
    /// commit, F9) → mọi ký tự engine sở hữu đều là chữ thật: xóa đủ rồi chèn.
    private static func commitEarly(
        deleteCount: Int, text: String, target: TextTarget, marked: MarkedState
    ) throws {
        let range = target.markedRange()
        let hasMarked = range.location != NSNotFound
        let real = hasMarked ? max(0, deleteCount - marked.scalarCount) : deleteCount
        if real > 0 {
            guard target.deleteBackward(count: real) else {
                throw ApplyError.cannotDelete
            }
        }
        target.insert(text, replacementRange: hasMarked ? range : .notFound)
        marked.clear()
    }

    // ------------------------------------------------------- §6.2 selectionReplace

    private static func selectionReplace(
        deleteCount: Int, insert: String, target: TextTarget, marked: MarkedState
    ) throws {
        let sel = target.selectionRange()
        if sel.location != NSNotFound, sel.length > 0, marked.isEmpty {
            // B1: app giữ selection thật → thay selection, không gửi backspace
            // (autocomplete không bị kích hoạt lại — P2-1 §6.2).
            target.insert(insert, replacementRange: sel)
            return
        }
        // Fallback §6.3 (app không giữ selection / marked còn treo).
        try backspaceType(deleteCount: deleteCount, insert: insert,
                          target: target, marked: marked)
    }

    // ------------------------------------------------------- §6.3 backspaceType

    /// Cơ chế xóa + chèn dùng chung (BackspaceType / ForwardAsCommit / macro /
    /// RESTORE). Xem công thức cốt lõi ở header file.
    private static func backspaceType(
        deleteCount: Int, insert: String, target: TextTarget, marked: MarkedState
    ) throws {
        let markedCP = marked.scalarCount
        if markedCP > 0 {
            // Thu hồi composition: setMarkedText("") gỡ glyph marked mà KHÔNG
            // commit (unmarkText() sẽ commit — review R1 F4).
            target.setMarked("", selectionRange: NSRange(location: 0, length: 0))
            marked.clear()
        }
        // Phần chữ thật cần xóa = delete_count - phần marked vừa thu hồi.
        let toDelete = max(0, deleteCount - markedCP)
        if toDelete > 0 {
            guard target.deleteBackward(count: toDelete) else {
                Diagnostics.log("deleteBackward rejected (\(toDelete)) — fail-open")
                throw ApplyError.cannotDelete
            }
        }
        target.insert(insert, replacementRange: .notFound)
    }

    // ------------------------------------------------------------- COMMIT (B2)

    /// COMMIT: marked hiện tại trở thành text vĩnh viễn + chèn ký tự ranh giới
    /// (P0-2 §4, bug B2 — Enter không nhân từ).
    private static func commit(
        insert: String, target: TextTarget, marked: MarkedState
    ) throws {
        let pending = marked.text
        if pending.isEmpty {
            target.insert(insert, replacementRange: .notFound)
        } else {
            let range = target.markedRange()
            if range.location != NSNotFound {
                // Thay nguyên marked range bằng text cuối + ký tự ranh giới.
                target.insert(pending + insert, replacementRange: range)
            } else {
                // App đã tự commit marked (NSTextView hay làm khi mất focus) —
                // chỉ chèn ký tự ranh giới, chèn thêm pending là nhân đôi (F9).
                target.insert(insert, replacementRange: .notFound)
            }
        }
        marked.clear()
    }
}
