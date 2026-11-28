// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 pha 3b — ORACLE: bản `ApplyReplace` Swift CŨ (trước khi quyết định chuyển vào
//! `textvn_session::imk` qua `ime_imk_plan`), giữ nguyên văn để `ApplyReplaceParityTests` so
//! chuỗi thao tác trên ma trận đầu vào. Parity tạm (G22): xoá cùng mã cũ khi session là mặc định.

import CoreBridge
import Foundation

@testable import IMKLib

enum LegacyApplyReplace {
    enum ApplyError: Error {
        case cannotDelete
        case rejected
    }

    /// Áp 1 outcome cho target. Trả `true` nếu đã sửa text; ném lỗi khi cơ chế
    /// xóa fail → caller fail-open (forward phím + `ime_reset`, P2-1 §12).
    ///
    /// `onReset` — callback gọi khi adapter tự quyết định reset engine
    /// (commit-early B11). Tách khỏi caller để không kéo engine vào đây.
    static func apply(
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
                if preedit.isEmpty {
                    // Mode E (vi_on = false): dự đoán tiếng Anh hoặc gõ tắt trả về
                    // REPLACE + preedit rỗng → xoá phần chữ đã chèn rồi ghi insert.
                    try backspaceType(deleteCount: deleteCount, insert: insert,
                                      target: target, marked: marked)
                }
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
        // Vòng 16 (audit B1): thu hồi marked TRƯỚC khi deleteBackward — trên
        // NSTextView thật, deleteBackward lúc marked còn hiển thị xoá NGUYÊN
        // marked range (hành vi IME chuẩn) thay vì `real` ký tự thật → mất
        // chữ. Nhất quán với backspaceType (đã làm đúng thứ tự này).
        let markedCP = marked.scalarCount
        if markedCP > 0 {
            target.setMarked("", selectionRange: NSRange(location: 0, length: 0))
            marked.clear()
        }
        let real = max(0, deleteCount - markedCP)
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
        // Vòng 16 (audit B1): thu hồi marked TRƯỚC khi deleteBackward (như
        // preeditReplace) — sau thu hồi, caret nằm tại đầu vùng marked cũ nên
        // chèn tại caret thay vì dùng range stale.
        let markedCP = marked.scalarCount
        let hadMarked = markedCP > 0
        if hadMarked {
            target.setMarked("", selectionRange: NSRange(location: 0, length: 0))
            marked.clear()
        }
        let real = max(0, deleteCount - markedCP)
        if real > 0 {
            guard target.deleteBackward(count: real) else {
                throw ApplyError.cannotDelete
            }
        }
        target.insert(text, replacementRange: .notFound)
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
