// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 (P0-6 §5 hàng *compose*) — C ABI `ime_compose_plan`: planner preedit/commit dùng chung
//! (`textvn_session::compose`) cho adapter C/C++/Swift. Thay bản C `lc_plan_key` (G22 — một nguồn).
//!
//! Hàm thuần: không trạng thái, không cấp phát chéo; adapter cấp [`ime_compose_plan_v1`] (stack).
//! Kết quả không vừa `IME_COMPOSE_MAX` ⇒ `IME_ERR_BUFFER` (chỉ xảy ra với chính sách không giới hạn
//! độ dài; `IME_COMPOSE_POLICY_LINUX` luôn ≤ 256).

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

use textvn_session::compose::{plan_key_with, ComposePolicy, EngineStep, KeyKind, RawEngineResult};

use crate::session_abi::IME_ERR_BUFFER;
use crate::{ime_result_v1, IME_ERR_ABI, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG, IME_OK};

pub const IME_COMPOSE_ABI_VERSION: u32 = 1;
/// Độ dài tối đa text kế hoạch (= `LC_COMP_MAX`).
pub const IME_COMPOSE_MAX: usize = 256;

// Chính sách theo năng lực OS (`ComposePolicy`).
pub const IME_COMPOSE_POLICY_TSF: u32 = 0;
pub const IME_COMPOSE_POLICY_LINUX: u32 = 1;

// Loại phím (`KeyKind`) — trị số khớp `lc_key_kind` của Linux.
pub const IME_CKEY_CHAR: u32 = 0;
pub const IME_CKEY_BACKSPACE: u32 = 1;
pub const IME_CKEY_ESCAPE: u32 = 2;
pub const IME_CKEY_ENTER: u32 = 3;
pub const IME_CKEY_TAB: u32 = 4;
pub const IME_CKEY_CTRL_SPACE: u32 = 5;
pub const IME_CKEY_OTHER: u32 = 6;
pub const IME_CKEY_ALTGR: u32 = 7;

/// Kế hoạch cho một phím (P0-6 §5). Trường cờ là `0`/`1`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_compose_plan_v1 {
    /// Adapter đặt = `IME_COMPOSE_ABI_VERSION` trước khi gọi.
    pub abi_version: u32,
    /// Số ký tự đã commit trước preedit phải xoá.
    pub delete_before: u32,
    /// P11 math: số bước mũi tên trái sau khi commit.
    pub cursor_left: u32,
    /// 1 = cập nhật preedit/commit thành `text`.
    pub has_text: u32,
    /// 1 = commit rồi đóng preedit.
    pub end: u32,
    /// 1 = nuốt phím; 0 = app nhận phím gốc.
    pub eaten: u32,
    /// 1 = commit + `ime_reset` (tràn; Ctrl+Space tới app).
    pub reset_engine: u32,
    /// Số ký tự hợp lệ trong `text`.
    pub text_len: u32,
    /// UTF-32.
    pub text: [u32; IME_COMPOSE_MAX],
}

/// `EngineStep::from_result(&ime_result_v1)` cho mọi adapter Rust (TSF, cli replay).
impl RawEngineResult for ime_result_v1 {
    fn action(&self) -> u32 {
        self.action
    }
    fn delete_count(&self) -> u16 {
        self.delete_count
    }
    fn insert(&self) -> (&[u32], u16) {
        (&self.insert, self.insert_len)
    }
    fn flags(&self) -> u32 {
        self.flags
    }
    fn cursor_left(&self) -> u16 {
        self.cursor_left
    }
}

fn key_kind(kind: u32, ch: u32) -> Option<KeyKind> {
    let c = || char::from_u32(ch);
    Some(match kind {
        IME_CKEY_CHAR => KeyKind::Char(c()?),
        IME_CKEY_BACKSPACE => KeyKind::Backspace,
        IME_CKEY_ESCAPE => KeyKind::Escape,
        IME_CKEY_ENTER => KeyKind::Enter,
        IME_CKEY_TAB => KeyKind::Tab,
        IME_CKEY_CTRL_SPACE => KeyKind::CtrlSpace,
        IME_CKEY_OTHER => KeyKind::Other,
        IME_CKEY_ALTGR => KeyKind::AltGr(c()?),
        _ => return None,
    })
}

fn policy(p: u32) -> Option<ComposePolicy> {
    match p {
        IME_COMPOSE_POLICY_TSF => Some(ComposePolicy::TSF),
        IME_COMPOSE_POLICY_LINUX => Some(ComposePolicy::LINUX),
        _ => None,
    }
}

/// Thân `ime_compose_plan` (khai báo `#[no_mangle]` ở `lib.rs`).
pub(crate) fn ime_compose_plan_impl(
    policy_id: u32,
    cur: *const u32,
    cur_len: usize,
    kind: u32,
    ch: u32,
    r: *const ime_result_v1,
    out: *mut ime_compose_plan_v1,
) -> i32 {
    if r.is_null() || out.is_null() || (cur.is_null() && cur_len > 0) {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    if out.abi_version != IME_COMPOSE_ABI_VERSION {
        return IME_ERR_ABI;
    }
    let (Some(policy), Some(key)) = (policy(policy_id), key_kind(kind, ch)) else {
        return IME_ERR_INVALID_ARG;
    };
    let cur_u32: &[u32] = if cur_len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(cur, cur_len) }
    };
    let Some(current) = cur_u32
        .iter()
        .map(|&c| char::from_u32(c))
        .collect::<Option<Vec<char>>>()
    else {
        return IME_ERR_INVALID_ARG;
    };
    let r = unsafe { &*r };
    match catch_unwind(AssertUnwindSafe(|| {
        let step = EngineStep::from_result(r);
        plan_key_with(policy, &current, key, &step)
    })) {
        Ok(plan) => {
            let text = plan.text.as_deref().unwrap_or(&[]);
            if text.len() > IME_COMPOSE_MAX {
                return IME_ERR_BUFFER;
            }
            out.delete_before = u32::from(plan.delete_before);
            out.cursor_left = u32::from(plan.cursor_left);
            out.has_text = u32::from(plan.text.is_some());
            out.end = u32::from(plan.end);
            out.eaten = u32::from(plan.eaten);
            out.reset_engine = u32::from(plan.reset_engine);
            out.text_len = text.len() as u32;
            for (dst, &c) in out.text.iter_mut().zip(text) {
                *dst = c as u32;
            }
            IME_OK
        }
        Err(_) => IME_ERR_INTERNAL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IME_ABI_VERSION, IME_FLAG_WORD_END, IME_MAX_TEXT};

    fn result(action: u32, del: u16, insert: &str, flags: u32) -> ime_result_v1 {
        let mut r: ime_result_v1 = unsafe { std::mem::zeroed() };
        r.abi_version = IME_ABI_VERSION;
        r.action = action;
        r.delete_count = del;
        r.flags = flags;
        let v: Vec<u32> = insert.chars().map(|c| c as u32).collect();
        r.insert[..v.len()].copy_from_slice(&v);
        r.insert_len = v.len() as u16;
        r
    }

    fn plan_buf() -> ime_compose_plan_v1 {
        let mut p: ime_compose_plan_v1 = unsafe { std::mem::zeroed() };
        p.abi_version = IME_COMPOSE_ABI_VERSION;
        p
    }

    fn call(
        policy: u32,
        cur: &str,
        kind: u32,
        ch: char,
        r: &ime_result_v1,
    ) -> (i32, ime_compose_plan_v1) {
        let cur: Vec<u32> = cur.chars().map(|c| c as u32).collect();
        let mut p = plan_buf();
        let rc = ime_compose_plan_impl(policy, cur.as_ptr(), cur.len(), kind, ch as u32, r, &mut p);
        (rc, p)
    }

    #[test]
    fn sf3_abi_linux_forwards_printable_boundary_tsf_commits_with_word() {
        let r = result(0, 0, "", IME_FLAG_WORD_END);
        let (rc, l) = call(IME_COMPOSE_POLICY_LINUX, "được", IME_CKEY_CHAR, ' ', &r);
        assert_eq!(rc, IME_OK);
        assert_eq!((l.end, l.eaten, l.has_text), (1, 0, 0));
        let (rc, t) = call(IME_COMPOSE_POLICY_TSF, "được", IME_CKEY_CHAR, ' ', &r);
        assert_eq!(rc, IME_OK);
        assert_eq!((t.end, t.eaten, t.has_text, t.text_len), (1, 1, 1, 5));
        assert_eq!(t.text[4], ' ' as u32);
    }

    #[test]
    fn sf3_abi_rejects_bad_args_and_abi() {
        let r = result(0, 0, "", 0);
        let mut p = plan_buf();
        assert_eq!(
            ime_compose_plan_impl(
                0,
                std::ptr::null(),
                0,
                IME_CKEY_CHAR,
                'a' as u32,
                &r,
                std::ptr::null_mut()
            ),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_compose_plan_impl(
                0,
                std::ptr::null(),
                3,
                IME_CKEY_CHAR,
                'a' as u32,
                &r,
                &mut p
            ),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            call(7, "", IME_CKEY_CHAR, 'a', &r).0,
            IME_ERR_INVALID_ARG,
            "policy lạ"
        );
        assert_eq!(
            call(0, "", 42, 'a', &r).0,
            IME_ERR_INVALID_ARG,
            "loại phím lạ"
        );
        p.abi_version = 9;
        assert_eq!(
            ime_compose_plan_impl(
                0,
                std::ptr::null(),
                0,
                IME_CKEY_CHAR,
                'a' as u32,
                &r,
                &mut p
            ),
            IME_ERR_ABI
        );
        let bad_ch = [0xD800u32];
        let mut p = plan_buf();
        assert_eq!(
            ime_compose_plan_impl(0, bad_ch.as_ptr(), 1, IME_CKEY_OTHER, 0, &r, &mut p),
            IME_ERR_INVALID_ARG,
            "surrogate trong preedit"
        );
    }

    #[test]
    fn sf3_abi_tsf_policy_overflow_is_buffer_error() {
        // TSF không giới hạn độ dài ⇒ text > IME_COMPOSE_MAX trả IME_ERR_BUFFER (Linux luôn ≤ 256).
        let cur = "a".repeat(IME_COMPOSE_MAX);
        let r = result(0, 0, "", 0);
        assert_eq!(
            call(IME_COMPOSE_POLICY_TSF, &cur, IME_CKEY_CHAR, 'b', &r).0,
            IME_ERR_BUFFER
        );
        let (rc, l) = call(
            IME_COMPOSE_POLICY_LINUX,
            &cur[..255],
            IME_CKEY_CHAR,
            'b',
            &r,
        );
        assert_eq!(rc, IME_OK);
        assert_eq!((l.end, l.eaten, l.reset_engine, l.text_len), (1, 0, 1, 255));
        const { assert!(IME_MAX_TEXT < IME_COMPOSE_MAX) };
    }

    #[test]
    fn sf3_ckey_values_match_lc_key_kind() {
        // `lc_key_kind` (adapters/linux-common/include/lc_compose.h): CHAR=0 … OTHER=6.
        assert_eq!(
            [
                IME_CKEY_CHAR,
                IME_CKEY_BACKSPACE,
                IME_CKEY_ESCAPE,
                IME_CKEY_ENTER,
                IME_CKEY_TAB,
                IME_CKEY_CTRL_SPACE,
                IME_CKEY_OTHER
            ],
            [0, 1, 2, 3, 4, 5, 6]
        );
    }
}
