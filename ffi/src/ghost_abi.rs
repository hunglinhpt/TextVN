// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-4 (P0-6 §5 hàng *ghost*) — C ABI `ime_ghost_label`: cổng + nhãn xem trước chữ mờ dùng chung
//! (`textvn_session::ghost`) cho adapter không vẽ được chữ mờ tại caret (IBus aux, Fcitx5 aux-down,
//! IMK bảng ứng viên). Thay `lc_preview_label` (C) và `previewLabel` (Swift) — G22.

use std::panic::{catch_unwind, AssertUnwindSafe};

use textvn_session::ghost::{ghost_text, preview_label};

use crate::session_abi::IME_ERR_BUFFER;
use crate::{IME_ERR_INTERNAL, IME_ERR_INVALID_ARG};

// Bit cổng — khớp `textvn_session::ghost::gate`.
pub const IME_GHOST_SECURE: u32 = 0x01;
pub const IME_GHOST_TERMINAL: u32 = 0x02;
pub const IME_GHOST_ASSIST_VISIBLE: u32 = 0x04;
pub const IME_GHOST_SUGGEST_OFF: u32 = 0x08;
pub const IME_GHOST_TEXT_AFTER_CARET: u32 = 0x10;

/// Thân `ime_ghost_label`: ghi nhãn UTF-8 (KHÔNG NUL) vào `out`, trả số byte; 0 = ẩn (cổng chặn
/// hoặc chữ mờ rỗng); `IME_ERR_BUFFER` nếu `cap` không đủ; `IME_ERR_INVALID_ARG` nếu con trỏ/UTF-8
/// sai.
pub(crate) fn ime_ghost_label_impl(
    ghost: *const u8,
    len: usize,
    gates: u32,
    out: *mut u8,
    cap: usize,
) -> i32 {
    if (ghost.is_null() && len > 0) || (out.is_null() && cap > 0) {
        return IME_ERR_INVALID_ARG;
    }
    let bytes: &[u8] = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ghost, len) }
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return IME_ERR_INVALID_ARG;
    };
    let Ok(label) = catch_unwind(AssertUnwindSafe(|| {
        ghost_text(Some(text), gates).and_then(|g| preview_label(&g))
    })) else {
        return IME_ERR_INTERNAL;
    };
    let Some(label) = label else { return 0 };
    let b = label.as_bytes();
    if b.len() > cap || b.len() > i32::MAX as usize {
        return IME_ERR_BUFFER;
    }
    unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), out, b.len()) };
    b.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(ghost: &str, gates: u32, cap: usize) -> (i32, String) {
        let mut out = vec![0u8; cap];
        let n = ime_ghost_label_impl(ghost.as_ptr(), ghost.len(), gates, out.as_mut_ptr(), cap);
        let s = if n > 0 {
            String::from_utf8(out[..n as usize].to_vec()).unwrap()
        } else {
            String::new()
        };
        (n, s)
    }

    #[test]
    fn sf4_abi_label_gates_and_errors() {
        assert_eq!(call(" phần mềm", 0, 64).1, "⇥ phần mềm");
        assert_eq!(call("   ", 0, 64).0, 0);
        assert_eq!(call("op", IME_GHOST_ASSIST_VISIBLE, 64).0, 0);
        assert_eq!(call("op", IME_GHOST_SECURE | IME_GHOST_TERMINAL, 64).0, 0);
        assert_eq!(call("phần mềm", 0, 4).0, IME_ERR_BUFFER);
        let bad = [0xFFu8];
        let mut out = [0u8; 8];
        assert_eq!(
            ime_ghost_label_impl(bad.as_ptr(), 1, 0, out.as_mut_ptr(), 8),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_ghost_label_impl(std::ptr::null(), 3, 0, out.as_mut_ptr(), 8),
            IME_ERR_INVALID_ARG
        );
    }

    #[test]
    fn sf4_gate_bits_match_session() {
        use textvn_session::ghost::gate;
        assert_eq!(
            [
                IME_GHOST_SECURE,
                IME_GHOST_TERMINAL,
                IME_GHOST_ASSIST_VISIBLE,
                IME_GHOST_SUGGEST_OFF,
                IME_GHOST_TEXT_AFTER_CARET
            ],
            [
                gate::SECURE,
                gate::TERMINAL,
                gate::ASSIST_VISIBLE,
                gate::SUGGEST_OFF,
                gate::TEXT_AFTER_CARET
            ]
        );
    }
}
