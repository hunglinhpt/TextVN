// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 pha 3b — C ABI `ime_imk_plan`: kế hoạch áp outcome engine cho macOS IMK
//! (`textvn_session::imk`, thay quyết định trong `ApplyReplace.swift` — G22).
//!
//! Hàm thuần, không trạng thái. Adapter cấp [`ime_imk_plan_v1`] (stack); chữ của thao tác nằm ở
//! `arena[text_off .. text_off + text_len]` (UTF-8, không NUL). Tràn `IME_IMK_MAX_OPS`/
//! `IME_IMK_ARENA` ⇒ `IME_ERR_BUFFER`, `count = 0` (adapter fail-open như khi app từ chối xoá).

use std::panic::{catch_unwind, AssertUnwindSafe};

use textvn_session::imk::{plan_imk, ImkOp, ImkOutcome, ImkState, ImkStrategy, InsertAt};

use crate::session_abi::IME_ERR_BUFFER;
use crate::{IME_ERR_ABI, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG, IME_OK};

pub const IME_IMK_ABI_VERSION: u32 = 1;
pub const IME_IMK_MAX_OPS: usize = 8;
pub const IME_IMK_ARENA: usize = 1024;

// Outcome (= `IME_ACTION_*`).
pub const IME_IMK_PASS: u32 = 0;
pub const IME_IMK_REPLACE: u32 = 1;
pub const IME_IMK_COMMIT: u32 = 2;
pub const IME_IMK_RESTORE: u32 = 3;

// Chiến lược (= `OutputStrategy` rawValue của macOS).
pub const IME_IMK_STRATEGY_PREEDIT: u32 = 0;
pub const IME_IMK_STRATEGY_BACKSPACE_TYPE: u32 = 1;
pub const IME_IMK_STRATEGY_SELECTION_REPLACE: u32 = 2;
pub const IME_IMK_STRATEGY_FORWARD_AS_COMMIT: u32 = 3;
pub const IME_IMK_STRATEGY_PASSTHROUGH: u32 = 4;

// Đo đạc của adapter (bit `state`).
pub const IME_IMK_MARKED_RANGE: u32 = 0x1;
pub const IME_IMK_SELECTION: u32 = 0x2;
pub const IME_IMK_PREEDIT_OVER_LIMIT: u32 = 0x4;

// Thao tác.
pub const IME_IMK_OP_RETRACT_MARKED: u32 = 1;
pub const IME_IMK_OP_DELETE_BACKWARD: u32 = 2;
pub const IME_IMK_OP_SET_MARKED: u32 = 3;
pub const IME_IMK_OP_INSERT: u32 = 4;
pub const IME_IMK_OP_CLEAR_MARKED_STATE: u32 = 5;
pub const IME_IMK_OP_RESET_ENGINE: u32 = 6;

// Vị trí chèn (`arg` của `IME_IMK_OP_INSERT`).
pub const IME_IMK_AT_CARET: u32 = 0;
pub const IME_IMK_AT_MARKED_RANGE: u32 = 1;
pub const IME_IMK_AT_SELECTION: u32 = 2;

/// Đầu vào (chuỗi UTF-8, có thể NULL khi độ dài 0).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_imk_input_v1 {
    pub abi_version: u32,
    pub action: u32,
    pub strategy: u32,
    pub state: u32,
    pub delete_count: u32,
    pub insert: *const u8,
    pub insert_len: usize,
    pub preedit: *const u8,
    pub preedit_len: usize,
    pub marked: *const u8,
    pub marked_len: usize,
}

/// Một thao tác.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ime_imk_op_v1 {
    pub kind: u32,
    /// `DELETE_BACKWARD`: số ký tự · `INSERT`: `IME_IMK_AT_*`.
    pub arg: u32,
    pub text_off: u32,
    pub text_len: u32,
}

/// Kế hoạch (adapter đặt `abi_version` trước khi gọi).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_imk_plan_v1 {
    pub abi_version: u32,
    pub count: u32,
    pub ops: [ime_imk_op_v1; IME_IMK_MAX_OPS],
    pub arena: [u8; IME_IMK_ARENA],
}

unsafe fn borrow_utf8<'a>(p: *const u8, len: usize) -> Option<&'a str> {
    if len == 0 {
        return Some("");
    }
    if p.is_null() {
        return None;
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(p, len) }).ok()
}

fn strategy(s: u32) -> Option<ImkStrategy> {
    Some(match s {
        IME_IMK_STRATEGY_PREEDIT => ImkStrategy::Preedit,
        IME_IMK_STRATEGY_BACKSPACE_TYPE => ImkStrategy::BackspaceType,
        IME_IMK_STRATEGY_SELECTION_REPLACE => ImkStrategy::SelectionReplace,
        IME_IMK_STRATEGY_FORWARD_AS_COMMIT => ImkStrategy::ForwardAsCommit,
        IME_IMK_STRATEGY_PASSTHROUGH => ImkStrategy::Passthrough,
        _ => return None,
    })
}

/// Thân `ime_imk_plan` (khai báo `#[no_mangle]` ở `lib.rs`).
pub(crate) fn ime_imk_plan_impl(input: *const ime_imk_input_v1, out: *mut ime_imk_plan_v1) -> i32 {
    if input.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let (input, out) = unsafe { (&*input, &mut *out) };
    if input.abi_version != IME_IMK_ABI_VERSION || out.abi_version != IME_IMK_ABI_VERSION {
        return IME_ERR_ABI;
    }
    out.count = 0;
    let (Some(insert), Some(preedit), Some(marked)) = (unsafe {
        (
            borrow_utf8(input.insert, input.insert_len),
            borrow_utf8(input.preedit, input.preedit_len),
            borrow_utf8(input.marked, input.marked_len),
        )
    }) else {
        return IME_ERR_INVALID_ARG;
    };
    let Some(strategy) = strategy(input.strategy) else {
        return IME_ERR_INVALID_ARG;
    };
    let del = input.delete_count as usize;
    let outcome = match input.action {
        IME_IMK_PASS => ImkOutcome::Pass,
        IME_IMK_REPLACE => ImkOutcome::Replace {
            delete_count: del,
            insert: insert.into(),
            preedit: preedit.into(),
        },
        IME_IMK_COMMIT => ImkOutcome::Commit {
            insert: insert.into(),
        },
        IME_IMK_RESTORE => ImkOutcome::Restore {
            delete_count: del,
            insert: insert.into(),
        },
        _ => return IME_ERR_INVALID_ARG,
    };
    let st = ImkState {
        marked: marked.into(),
        marked_range_present: input.state & IME_IMK_MARKED_RANGE != 0,
        selection_nonempty: input.state & IME_IMK_SELECTION != 0,
        preedit_exceeds_limit: input.state & IME_IMK_PREEDIT_OVER_LIMIT != 0,
    };
    let Ok(ops) = catch_unwind(AssertUnwindSafe(|| plan_imk(&outcome, strategy, &st))) else {
        return IME_ERR_INTERNAL;
    };
    if ops.len() > IME_IMK_MAX_OPS {
        return IME_ERR_BUFFER;
    }
    let mut arena = 0usize;
    let mut written = [ime_imk_op_v1::default(); IME_IMK_MAX_OPS];
    for (slot, op) in written.iter_mut().zip(&ops) {
        let (kind, arg, text) = match op {
            ImkOp::RetractMarked => (IME_IMK_OP_RETRACT_MARKED, 0, ""),
            ImkOp::DeleteBackward(n) => (IME_IMK_OP_DELETE_BACKWARD, *n as u32, ""),
            ImkOp::SetMarked(t) => (IME_IMK_OP_SET_MARKED, 0, t.as_str()),
            ImkOp::Insert { text, at } => (
                IME_IMK_OP_INSERT,
                match at {
                    InsertAt::Caret => IME_IMK_AT_CARET,
                    InsertAt::MarkedRange => IME_IMK_AT_MARKED_RANGE,
                    InsertAt::Selection => IME_IMK_AT_SELECTION,
                },
                text.as_str(),
            ),
            ImkOp::ClearMarkedState => (IME_IMK_OP_CLEAR_MARKED_STATE, 0, ""),
            ImkOp::ResetEngine => (IME_IMK_OP_RESET_ENGINE, 0, ""),
        };
        let bytes = text.as_bytes();
        if arena + bytes.len() > IME_IMK_ARENA {
            return IME_ERR_BUFFER;
        }
        out.arena[arena..arena + bytes.len()].copy_from_slice(bytes);
        *slot = ime_imk_op_v1 {
            kind,
            arg,
            text_off: arena as u32,
            text_len: bytes.len() as u32,
        };
        arena += bytes.len();
    }
    out.ops = written;
    out.count = ops.len() as u32;
    IME_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        action: u32,
        strategy: u32,
        state: u32,
        del: u32,
        ins: &str,
        pre: &str,
        marked: &str,
    ) -> ime_imk_input_v1 {
        ime_imk_input_v1 {
            abi_version: IME_IMK_ABI_VERSION,
            action,
            strategy,
            state,
            delete_count: del,
            insert: ins.as_ptr(),
            insert_len: ins.len(),
            preedit: pre.as_ptr(),
            preedit_len: pre.len(),
            marked: marked.as_ptr(),
            marked_len: marked.len(),
        }
    }

    fn out_buf() -> Box<ime_imk_plan_v1> {
        let mut o: Box<ime_imk_plan_v1> = Box::new(unsafe { std::mem::zeroed() });
        o.abi_version = IME_IMK_ABI_VERSION;
        o
    }

    #[test]
    fn sf3b_abi_preedit_replace_ops_and_arena() {
        let inp = input(
            IME_IMK_REPLACE,
            IME_IMK_STRATEGY_PREEDIT,
            IME_IMK_MARKED_RANGE,
            3,
            "tiê",
            "tiê",
            "e",
        );
        let mut o = out_buf();
        assert_eq!(ime_imk_plan_impl(&inp, &mut *o), IME_OK);
        assert_eq!(o.count, 3);
        assert_eq!(o.ops[0].kind, IME_IMK_OP_RETRACT_MARKED);
        assert_eq!(
            (o.ops[1].kind, o.ops[1].arg),
            (IME_IMK_OP_DELETE_BACKWARD, 2)
        );
        assert_eq!(o.ops[2].kind, IME_IMK_OP_SET_MARKED);
        let t = &o.arena[o.ops[2].text_off as usize..][..o.ops[2].text_len as usize];
        assert_eq!(std::str::from_utf8(t).unwrap(), "tiê");
    }

    #[test]
    fn sf3b_abi_rejects_bad_input() {
        let mut o = out_buf();
        assert_eq!(
            ime_imk_plan_impl(std::ptr::null(), &mut *o),
            IME_ERR_INVALID_ARG
        );
        let bad = input(9, 0, 0, 0, "", "", "");
        assert_eq!(ime_imk_plan_impl(&bad, &mut *o), IME_ERR_INVALID_ARG);
        let bad = input(IME_IMK_PASS, 9, 0, 0, "", "", "");
        assert_eq!(ime_imk_plan_impl(&bad, &mut *o), IME_ERR_INVALID_ARG);
        let utf = [0xFFu8];
        let mut bad = input(IME_IMK_COMMIT, 0, 0, 0, "", "", "");
        bad.insert = utf.as_ptr();
        bad.insert_len = 1;
        assert_eq!(ime_imk_plan_impl(&bad, &mut *o), IME_ERR_INVALID_ARG);
        let mut v = input(IME_IMK_PASS, 0, 0, 0, "", "", "");
        v.abi_version = 7;
        assert_eq!(ime_imk_plan_impl(&v, &mut *o), IME_ERR_ABI);
    }

    #[test]
    fn sf3b_abi_arena_overflow_is_buffer_error() {
        let big = "x".repeat(IME_IMK_ARENA + 1);
        let inp = input(IME_IMK_COMMIT, IME_IMK_STRATEGY_PREEDIT, 0, 0, &big, "", "");
        let mut o = out_buf();
        assert_eq!(ime_imk_plan_impl(&inp, &mut *o), IME_ERR_BUFFER);
        assert_eq!(o.count, 0);
    }
}
