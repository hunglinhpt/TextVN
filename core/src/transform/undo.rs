// SPDX-License-Identifier: GPL-3.0-or-later
//! Các quy tắc undo của Telex (P0-1 §1 `transform/undo.rs`):
//!
//! - `ass` → `as`: bấm lại **đúng** dấu → bỏ dấu và gõ chính key đó như chữ thường
//!   (áp dụng trong `method/telex.rs::apply_tone_key`).
//! - `ww` → `w`: marker không gắn được (không có âm đích) → nuốt key lặp
//!   (trong `method/telex.rs::push_key`).
//! - `ee` → `e`, `dd` → `d`: key đôi đã tạo ký tự đặc biệt → gỡ về gốc.
//!
//! `mark_vowel` / `mark_horn`: quy ước undo marker dạng âm cho VNI (`6`/`7`/`8`) và
//! VIQR (`^`/`+`/`(`): marker đã áp lên **đúng** âm cuối → gỡ dạng + gõ literal marker
//! (giống `ass` → `as`); marker không có âm đích → literal. Quy ước này chưa có oracle
//! UniKey cho nhánh undo (ghi trong `docs/10-shared/P0-REVIEW-LOG.md`).

use super::tone::is_vowel;
use super::vowel_table::{form_like, locate, O, O_HOOK, U, U_HOOK};

/// Gỡ dấu của ký tự âm về không dấu (giữ case). Không phải âm → giữ nguyên.
pub fn unmark(c: char) -> char {
    match locate(c) {
        Some((e, t)) if t > 0 => form_like(c, e, 0),
        _ => c,
    }
}

/// Áp marker đổi dạng âm lên âm cuối `out`.
///
/// `table` = bảng **sinh từ data** `(key, âm gốc, âm đích)` — truyền thẳng
/// `method::keys_generated::<method>::CIRCUMFLEX|HORN|BREVE` (P0-1 §3: bảng là data).
/// - âm cuối khớp `gốc` → đổi sang `đích` (giữ dấu thanh)
/// - âm cuối đã ở `đích` → gỡ về `gốc` + gõ literal `key`
/// - không khớp / không có âm → gõ literal `key`
pub fn mark_vowel(out: &mut Vec<char>, key: char, table: &[(char, usize, usize)]) {
    let Some(idx) = out.iter().rposition(|&c| is_vowel(c)) else {
        out.push(key);
        return;
    };
    let ch = out[idx];
    let Some((e, t)) = locate(ch) else {
        out.push(key);
        return;
    };
    for &(k, from, to) in table {
        if k != key {
            continue;
        }
        if e == to {
            out[idx] = form_like(ch, from, t);
            out.push(key);
            return;
        }
        if e == from {
            out[idx] = form_like(ch, to, t);
            return;
        }
    }
    out.push(key);
}

/// Marker sừng `ơ`/`ư` (VNI `7`, VIQR `+`, Telex `w`): áp lên âm cuối; cụm `uo`/`ưo` → `ươ`;
/// đã sừng → gỡ + literal; không áp được → literal.
///
/// `table` = bảng sinh từ data `(key, âm gốc, âm đích)` (xem `mark_vowel`).
/// Quy tắc cụm `uo` là **thuật toán** nên vẫn nằm ở đây, không mô tả được bằng bảng.
pub fn mark_horn(out: &mut Vec<char>, key: char, table: &[(char, usize, usize)]) {
    let Some(idx) = out.iter().rposition(|&c| is_vowel(c)) else {
        out.push(key);
        return;
    };
    let ch = out[idx];
    let Some((e, t)) = locate(ch) else {
        out.push(key);
        return;
    };
    // Cụm `uo` (kể cả `ưo` đã sừng ở u): marker áp cả cụm → ư + ơ
    if e == O {
        if let Some(prev) = idx.checked_sub(1).map(|p| out[p]) {
            if let Some((pe, pt)) = locate(prev) {
                if pe == U || pe == U_HOOK {
                    out[idx - 1] = form_like(prev, U_HOOK, pt);
                    out[idx] = form_like(ch, O_HOOK, t);
                    return;
                }
            }
        }
        out[idx] = form_like(ch, O_HOOK, t);
        return;
    }
    if e == O_HOOK {
        out[idx] = form_like(ch, O, t);
        out.push(key);
        return;
    }
    // u → ư và gỡ ư → u lấy từ bảng (Telex `w` / VNI `7` / VIQR `+` đều dùng chung)
    for &(_, from, to) in table {
        if e == to {
            out[idx] = form_like(ch, from, t);
            out.push(key);
            return;
        }
        if e == from {
            out[idx] = form_like(ch, to, t);
            return;
        }
    }
    out.push(key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::vowel_table::{A, A_CIRC, E, E_CIRC, O, O_HOOK, U, U_HOOK};

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn unmark_keeps_case() {
        assert_eq!(unmark('á'), 'a');
        assert_eq!(unmark('Ấ'), 'Â');
        assert_eq!(unmark('x'), 'x');
    }

    #[test]
    fn mark_vowel_applies_and_undoes() {
        let mut out = chars("a");
        mark_vowel(&mut out, '6', &[('6', A, A_CIRC), ('6', E, E_CIRC)]);
        assert_eq!(out.iter().collect::<String>(), "â");

        let mut out = chars("â");
        mark_vowel(&mut out, '6', &[('6', A, A_CIRC), ('6', E, E_CIRC)]);
        assert_eq!(out.iter().collect::<String>(), "a6"); // đã mũ → gỡ về gốc + literal

        let mut out = chars("ấ");
        mark_vowel(&mut out, '6', &[('6', A, A_CIRC)]);
        assert_eq!(out.iter().collect::<String>(), "á6"); // gỡ mũ nhưng GIỮ dấu thanh

        let mut out = chars("i");
        mark_vowel(&mut out, '6', &[('6', A, A_CIRC), ('6', E, E_CIRC)]);
        assert_eq!(out.iter().collect::<String>(), "i6"); // không có âm đích

        let mut out = chars("ba");
        mark_vowel(&mut out, '6', &[('6', A, A_CIRC)]);
        assert_eq!(out.iter().collect::<String>(), "bâ");
    }

    #[test]
    fn mark_horn_pair_and_single() {
        let mut out = chars("uo");
        mark_horn(&mut out, '7', &[('7', O, O_HOOK), ('7', U, U_HOOK)]);
        assert_eq!(out.iter().collect::<String>(), "ươ");

        let mut out = chars("mo");
        mark_horn(&mut out, '7', &[('7', O, O_HOOK), ('7', U, U_HOOK)]);
        assert_eq!(out.iter().collect::<String>(), "mơ");

        let mut out = chars("tu");
        mark_horn(&mut out, '7', &[('7', O, O_HOOK), ('7', U, U_HOOK)]);
        assert_eq!(out.iter().collect::<String>(), "tư");

        let mut out = chars("tư");
        mark_horn(&mut out, '7', &[('7', O, O_HOOK), ('7', U, U_HOOK)]);
        assert_eq!(out.iter().collect::<String>(), "tu7"); // đã sừng → gỡ + literal

        let mut out = chars("ba");
        mark_horn(&mut out, '7', &[('7', O, O_HOOK), ('7', U, U_HOOK)]);
        assert_eq!(out.iter().collect::<String>(), "ba7"); // không áp được
    }
}
