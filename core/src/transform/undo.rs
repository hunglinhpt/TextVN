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
//! VIQR (`^`/`+`/`(`): marker đã áp lên **đúng** âm → gỡ dạng + gõ literal marker
//! (giống `ass` → `as`); marker không có âm đích → literal. Quy ước này chưa có oracle
//! UniKey cho nhánh undo (ghi trong `docs/10-shared/P0-REVIEW-LOG.md`).
//!
//! Âm nhận marker chọn trên **cả cụm nguyên âm cuối** ([`mark_cluster`], R2-60) như UniKey
//! `processRoof`/`processHook` (bảng chuỗi nguyên âm `withRoof`/`withHook`): gõ dấu sau cả
//! cụm vẫn đúng — `toi6` → `tôi`, `nguoi7` → `ngươi`, `ruou7` → `rươu`, `luu7` → `lưu`.

use super::diacritic_style::vowel_span;
use super::vowel_table::{form_like, locate, O, O_HOOK, U, U_HOOK};
use crate::validate::NUCLEI;

/// Gỡ dấu của ký tự âm về không dấu (giữ case). Không phải âm → giữ nguyên.
pub fn unmark(c: char) -> char {
    match locate(c) {
        Some((e, t)) if t > 0 => form_like(c, e, 0),
        _ => c,
    }
}

/// Kết quả áp một marker dạng âm lên một vị trí.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marked {
    /// Đã thêm dạng âm (mũ/sừng/breve).
    Applied,
    /// Âm đã có dạng đó → gỡ về gốc.
    Undone,
}

/// Cụm `run` (đã bỏ dấu thanh) có phải một vần trong bảng vần hợp lệ không.
fn is_nucleus(run: &[char]) -> bool {
    let bare = || {
        run.iter().map(|&c| {
            let b = unmark(c);
            b.to_lowercase().next().unwrap_or(b)
        })
    };
    NUCLEI.iter().any(|n| n.chars().eq(bare()))
}

/// Áp marker dạng âm lên **cụm nguyên âm cuối** của `out` (R2-60).
///
/// `apply(w, i)` thử marker trên âm `w[i]` (đổi `w` tại chỗ): `Some(Applied)` thêm dạng,
/// `Some(Undone)` gỡ dạng đã có, `None` âm này không nhận marker (không đổi `w`).
/// Thử từ âm cuối sang trái, lấy vị trí đầu tiên gỡ được dạng đã có hoặc áp ra **vần hợp lệ**
/// (UniKey chỉ áp khi chuỗi nguyên âm mới có trong bảng). Không vị trí nào hợp lệ → như bản
/// cũ: chỉ thử âm cuối (`oa6` → `oâ`). `None` = không áp được (gõ literal).
pub fn mark_cluster(
    out: &mut [char],
    apply: impl Fn(&mut [char], usize) -> Option<Marked>,
) -> Option<Marked> {
    let (start, end) = vowel_span(out)?;
    if start < end {
        let mut trial = out.to_vec();
        for idx in (start..=end).rev() {
            trial.copy_from_slice(out);
            match apply(&mut trial, idx) {
                Some(Marked::Applied) if !is_nucleus(&trial[start..=end]) => {}
                Some(m) => {
                    out.copy_from_slice(&trial);
                    return Some(m);
                }
                None => {}
            }
        }
    }
    apply(out, end)
}

/// Áp marker đổi dạng âm lên cụm nguyên âm cuối `out` ([`mark_cluster`]).
///
/// `table` = bảng **sinh từ data** `(key, âm gốc, âm đích)` — truyền thẳng
/// `method::keys_generated::<method>::CIRCUMFLEX|HORN|BREVE` (P0-1 §3: bảng là data).
/// - âm khớp `gốc` → đổi sang `đích` (giữ dấu thanh)
/// - âm đã ở `đích` → gỡ về `gốc` + gõ literal `key`
/// - không khớp / không có âm → gõ literal `key`
pub fn mark_vowel(out: &mut Vec<char>, key: char, table: &[(char, usize, usize)]) {
    let marked = mark_cluster(out, |w, idx| {
        let ch = w[idx];
        let (e, t) = locate(ch)?;
        for &(k, from, to) in table {
            if k != key {
                continue;
            }
            if e == to {
                w[idx] = form_like(ch, from, t);
                return Some(Marked::Undone);
            }
            if e == from {
                w[idx] = form_like(ch, to, t);
                return Some(Marked::Applied);
            }
        }
        None
    });
    if marked != Some(Marked::Applied) {
        out.push(key);
    }
}

/// Marker sừng `ơ`/`ư` (VNI `7`, VIQR `+`): áp lên cụm nguyên âm cuối ([`mark_cluster`]);
/// cụm `uo`/`ưo` → `ươ`; đã sừng → gỡ + literal; không áp được → literal.
///
/// `table` = bảng sinh từ data `(key, âm gốc, âm đích)` (xem `mark_vowel`).
/// Quy tắc cụm `uo` là **thuật toán** nên vẫn nằm ở đây, không mô tả được bằng bảng.
pub fn mark_horn(out: &mut Vec<char>, key: char, table: &[(char, usize, usize)]) {
    let marked = mark_cluster(out, |w, idx| {
        let ch = w[idx];
        let (e, t) = locate(ch)?;
        // Cụm `uo` (kể cả `ưo` đã sừng ở u): marker áp cả cụm → ư + ơ
        if e == O {
            if let Some(prev) = idx.checked_sub(1).map(|p| w[p]) {
                if let Some((pe, pt)) = locate(prev) {
                    if pe == U || pe == U_HOOK {
                        w[idx - 1] = form_like(prev, U_HOOK, pt);
                        w[idx] = form_like(ch, O_HOOK, t);
                        return Some(Marked::Applied);
                    }
                }
            }
            w[idx] = form_like(ch, O_HOOK, t);
            return Some(Marked::Applied);
        }
        if e == O_HOOK {
            w[idx] = form_like(ch, O, t);
            return Some(Marked::Undone);
        }
        // u → ư và gỡ ư → u lấy từ bảng (VNI `7` / VIQR `+` dùng chung)
        for &(_, from, to) in table {
            if e == to {
                w[idx] = form_like(ch, from, t);
                return Some(Marked::Undone);
            }
            if e == from {
                w[idx] = form_like(ch, to, t);
                return Some(Marked::Applied);
            }
        }
        None
    });
    if marked != Some(Marked::Applied) {
        out.push(key);
    }
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
