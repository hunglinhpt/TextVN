// SPDX-License-Identifier: GPL-3.0-or-later
//! Các quy tắc undo của Telex (P0-1 §1 `transform/undo.rs`):
//!
//! - `ass` → `as`: bấm lại **đúng** dấu → bỏ dấu và gõ chính key đó như chữ thường
//!   (áp dụng trong `method/telex.rs::apply_tone_key`).
//! - `ww` → `w`: marker không gắn được (không có âm đích) → nuốt key lặp
//!   (trong `method/telex.rs::push_key`).
//! - `ee` → `e`, `dd` → `d`: key đôi đã tạo ký tự đặc biệt → gỡ về gốc.

use super::vowel_table::{form_like, locate};

/// Gỡ dấu của ký tự âm về không dấu (giữ case). Không phải âm → giữ nguyên.
pub fn unmark(c: char) -> char {
    match locate(c) {
        Some((e, t)) if t > 0 => form_like(c, e, 0),
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmark_keeps_case() {
        assert_eq!(unmark('á'), 'a');
        assert_eq!(unmark('Ấ'), 'Â');
        assert_eq!(unmark('x'), 'x');
    }
}
