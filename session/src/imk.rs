// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 pha 3b — kế hoạch áp outcome engine cho macOS IMK (từ `ApplyReplace.swift`, P2-1 §6).
//!
//! IMK không có "composition do TIP sở hữu" như TSF/Linux: engine trả `preedit` riêng, adapter thu
//! hồi marked (`setMarkedText("")` — gỡ glyph KHÔNG commit) rồi xoá phần chữ THẬT bằng
//! `deleteBackward:`. Hàm ở đây chỉ QUYẾT ĐỊNH chuỗi thao tác; adapter Swift thực thi trên
//! `IMKTextInput` và báo lỗi khi app từ chối xoá (fail-open ở caller).
//!
//! Công thức cốt lõi (review R1 F3/F4): `delete_count` = phần engine sở hữu = chữ thật đã vào
//! document + marked đang hiển thị ⇒ chữ thật cần xoá = `delete_count - marked_scalars`.
//! Đơn vị: `delete_count` và `marked_scalars` đếm code point (Rust `char`).
//!
//! Đo đạc phụ thuộc OS (do adapter cung cấp, không đoán): marked range còn trong app không, app giữ
//! selection không rỗng không, preedit mới vượt giới hạn B11 (8 grapheme — Swift `String.count`).

/// Chiến lược xuất (khớp `OutputStrategy` của macOS / `textvn_strategy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImkStrategy {
    /// Marked text (`setMarkedText`).
    Preedit,
    /// `deleteBackward:` ×n + chèn.
    BackspaceType,
    /// Thay selection thật (B1), không backspace.
    SelectionReplace,
    /// Như BackspaceType trên macOS (P2-1 §6.3).
    ForwardAsCommit,
    /// Không đụng text (trừ REPLACE + preedit rỗng: Mode EN/gõ tắt).
    Passthrough,
}

/// Outcome engine đã giải mã.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImkOutcome {
    /// PASS.
    Pass,
    /// REPLACE.
    Replace {
        /// Số code point engine sở hữu cần xoá.
        delete_count: usize,
        /// Text chèn khi áp như chữ thật.
        insert: String,
        /// Preedit mới (rỗng = thay đổi text THẬT: macro, auto-capitalize — F5).
        preedit: String,
    },
    /// COMMIT (ký tự ranh giới, B2).
    Commit {
        /// Ký tự ranh giới.
        insert: String,
    },
    /// RESTORE (B5/Esc).
    Restore {
        /// Số code point cần xoá.
        delete_count: usize,
        /// Chuỗi gõ gốc.
        insert: String,
    },
}

/// Trạng thái/đo đạc phía adapter lúc áp.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImkState {
    /// Marked adapter đang theo dõi (`MarkedState.text`).
    pub marked: String,
    /// App còn marked range (`markedRange().location != NSNotFound`).
    pub marked_range_present: bool,
    /// App giữ selection không rỗng.
    pub selection_nonempty: bool,
    /// Preedit mới vượt giới hạn B11 (adapter đếm grapheme).
    pub preedit_exceeds_limit: bool,
}

/// Nơi chèn text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertAt {
    /// Tại caret (`replacementRange = NSNotFound`).
    Caret,
    /// Thay nguyên marked range của app.
    MarkedRange,
    /// Thay selection hiện tại (B1).
    Selection,
}

/// Một thao tác adapter thực thi theo thứ tự.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImkOp {
    /// `setMarkedText("")` + xoá marked của adapter (gỡ glyph, KHÔNG commit).
    RetractMarked,
    /// `deleteBackward:` ×n; app từ chối ⇒ adapter dừng và fail-open.
    DeleteBackward(usize),
    /// `setMarkedText(text)` (caret cuối) + ghi marked của adapter.
    SetMarked(String),
    /// Chèn text.
    Insert {
        /// Text.
        text: String,
        /// Vị trí.
        at: InsertAt,
    },
    /// Xoá marked của adapter (không đụng app).
    ClearMarkedState,
    /// Adapter reset engine (B11 commit-early).
    ResetEngine,
}

/// Kế hoạch áp `outcome` theo `strategy` (thứ tự quyết định = `ApplyReplace.apply`).
#[must_use]
pub fn plan_imk(outcome: &ImkOutcome, strategy: ImkStrategy, st: &ImkState) -> Vec<ImkOp> {
    let mut ops = Vec::new();
    match outcome {
        ImkOutcome::Pass => {}
        ImkOutcome::Replace {
            delete_count,
            insert,
            preedit,
        } => match strategy {
            ImkStrategy::Preedit if preedit.is_empty() => {
                backspace_type(&mut ops, *delete_count, insert, st);
            }
            ImkStrategy::Preedit => preedit_replace(&mut ops, *delete_count, preedit, st),
            ImkStrategy::SelectionReplace => {
                if st.selection_nonempty && st.marked.is_empty() {
                    ops.push(ImkOp::Insert {
                        text: insert.clone(),
                        at: InsertAt::Selection,
                    });
                } else {
                    backspace_type(&mut ops, *delete_count, insert, st);
                }
            }
            ImkStrategy::BackspaceType | ImkStrategy::ForwardAsCommit => {
                backspace_type(&mut ops, *delete_count, insert, st);
            }
            ImkStrategy::Passthrough => {
                if preedit.is_empty() {
                    backspace_type(&mut ops, *delete_count, insert, st);
                }
            }
        },
        ImkOutcome::Commit { insert } => {
            let at = if st.marked.is_empty() || !st.marked_range_present {
                // Không marked, hoặc app đã tự commit marked (F9) — chèn thêm pending là nhân đôi.
                ImkOp::Insert {
                    text: insert.clone(),
                    at: InsertAt::Caret,
                }
            } else {
                ImkOp::Insert {
                    text: format!("{}{insert}", st.marked),
                    at: InsertAt::MarkedRange,
                }
            };
            ops.push(at);
            ops.push(ImkOp::ClearMarkedState);
        }
        ImkOutcome::Restore {
            delete_count,
            insert,
        } => backspace_type(&mut ops, *delete_count, insert, st),
    }
    ops
}

/// Thu hồi marked (nếu có) rồi xoá phần chữ thật; trả số marked đã thu hồi.
fn retract_and_delete(ops: &mut Vec<ImkOp>, delete_count: usize, st: &ImkState) {
    let marked_cp = st.marked.chars().count();
    if marked_cp > 0 {
        ops.push(ImkOp::RetractMarked);
    }
    let real = delete_count.saturating_sub(marked_cp);
    if real > 0 {
        ops.push(ImkOp::DeleteBackward(real));
    }
}

fn backspace_type(ops: &mut Vec<ImkOp>, delete_count: usize, insert: &str, st: &ImkState) {
    retract_and_delete(ops, delete_count, st);
    ops.push(ImkOp::Insert {
        text: insert.to_string(),
        at: InsertAt::Caret,
    });
}

fn preedit_replace(ops: &mut Vec<ImkOp>, delete_count: usize, preedit: &str, st: &ImkState) {
    retract_and_delete(ops, delete_count, st);
    if st.preedit_exceeds_limit {
        // B11 (P2-1 §7): commit TOÀN BỘ preedit mới (gồm phím vừa gõ) thành text thật rồi reset
        // engine — chỉ commit marked cũ rồi nuốt phím là mất ký tự thứ 9 (MAC-032).
        ops.push(ImkOp::Insert {
            text: preedit.to_string(),
            at: InsertAt::Caret,
        });
        ops.push(ImkOp::ResetEngine);
    } else {
        ops.push(ImkOp::SetMarked(preedit.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(marked: &str) -> ImkState {
        ImkState {
            marked: marked.into(),
            marked_range_present: !marked.is_empty(),
            ..ImkState::default()
        }
    }

    fn replace(del: usize, insert: &str, preedit: &str) -> ImkOutcome {
        ImkOutcome::Replace {
            delete_count: del,
            insert: insert.into(),
            preedit: preedit.into(),
        }
    }

    #[test]
    fn preedit_retracts_marked_then_deletes_only_real_text_f3() {
        // `passed` "ti" đã vào document + marked "e" ⇒ owned 3; xoá thật 2.
        let ops = plan_imk(&replace(3, "tiê", "tiê"), ImkStrategy::Preedit, &st("e"));
        assert_eq!(
            ops,
            vec![
                ImkOp::RetractMarked,
                ImkOp::DeleteBackward(2),
                ImkOp::SetMarked("tiê".into())
            ]
        );
    }

    #[test]
    fn preedit_empty_replace_is_real_text_change_f5() {
        let ops = plan_imk(&replace(2, "Ko", ""), ImkStrategy::Preedit, &st(""));
        assert_eq!(
            ops,
            vec![
                ImkOp::DeleteBackward(2),
                ImkOp::Insert {
                    text: "Ko".into(),
                    at: InsertAt::Caret
                }
            ]
        );
    }

    #[test]
    fn b11_commit_early_then_reset() {
        let mut s = st("nghiêngg");
        s.preedit_exceeds_limit = true;
        let ops = plan_imk(&replace(8, "", "nghiênggx"), ImkStrategy::Preedit, &s);
        assert_eq!(
            ops,
            vec![
                ImkOp::RetractMarked,
                ImkOp::Insert {
                    text: "nghiênggx".into(),
                    at: InsertAt::Caret
                },
                ImkOp::ResetEngine
            ]
        );
    }

    #[test]
    fn commit_replaces_marked_range_or_only_boundary_when_app_committed_f9() {
        let ops = plan_imk(
            &ImkOutcome::Commit { insert: " ".into() },
            ImkStrategy::Preedit,
            &st("việt"),
        );
        assert_eq!(
            ops,
            vec![
                ImkOp::Insert {
                    text: "việt ".into(),
                    at: InsertAt::MarkedRange
                },
                ImkOp::ClearMarkedState
            ]
        );
        let mut gone = st("việt");
        gone.marked_range_present = false;
        let ops = plan_imk(
            &ImkOutcome::Commit { insert: " ".into() },
            ImkStrategy::Preedit,
            &gone,
        );
        assert_eq!(
            ops[0],
            ImkOp::Insert {
                text: " ".into(),
                at: InsertAt::Caret
            }
        );
    }

    #[test]
    fn selection_replace_only_with_real_selection_and_no_marked_b1() {
        let mut s = st("");
        s.selection_nonempty = true;
        let ops = plan_imk(&replace(3, "abc", ""), ImkStrategy::SelectionReplace, &s);
        assert_eq!(
            ops,
            vec![ImkOp::Insert {
                text: "abc".into(),
                at: InsertAt::Selection
            }]
        );
        let ops = plan_imk(
            &replace(3, "abc", ""),
            ImkStrategy::SelectionReplace,
            &st("x"),
        );
        assert_eq!(ops[0], ImkOp::RetractMarked);
    }

    #[test]
    fn passthrough_only_applies_mode_en_replace_and_pass_is_noop() {
        assert!(plan_imk(&replace(1, "a", "b"), ImkStrategy::Passthrough, &st("")).is_empty());
        assert_eq!(
            plan_imk(&replace(1, "the", ""), ImkStrategy::Passthrough, &st("")).len(),
            2
        );
        assert!(plan_imk(&ImkOutcome::Pass, ImkStrategy::Preedit, &st("x")).is_empty());
    }

    #[test]
    fn restore_uses_backspace_type_for_every_strategy() {
        for s in [
            ImkStrategy::Preedit,
            ImkStrategy::BackspaceType,
            ImkStrategy::SelectionReplace,
            ImkStrategy::ForwardAsCommit,
            ImkStrategy::Passthrough,
        ] {
            let ops = plan_imk(
                &ImkOutcome::Restore {
                    delete_count: 3,
                    insert: "sort".into(),
                },
                s,
                &st("sỏ"),
            );
            assert_eq!(
                ops,
                vec![
                    ImkOp::RetractMarked,
                    ImkOp::DeleteBackward(1),
                    ImkOp::Insert {
                        text: "sort".into(),
                        at: InsertAt::Caret
                    }
                ]
            );
        }
    }
}
