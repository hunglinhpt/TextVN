// SPDX-License-Identifier: GPL-3.0-or-later
//! Vị trí đặt dấu thanh trong âm tiết — quy tắc chính tả như UniKey/OpenKey/Bamboo.
//!
//! 1. Bán âm của phụ âm đầu không nhận dấu: `u` trong `qu` (`quý`, `quở`), `i` trong `gi`
//!    khi sau nó còn nguyên âm (`giáo`, `giữa`, `giúp`; nhưng `gì`, `gìn`).
//! 2. Nguyên âm mang dấu phụ (ă â ê ô ơ ư) nhận dấu; có hai thì lấy cái sau (`ươ` → `ơ`):
//!    `cửa`, `tuổi`, `người`, `tuần`, `huế`, `quyển`, `hươu`, `thuở`.
//! 3. Có phụ âm cuối → nguyên âm cuối của cụm (`hoàng`, `toán`, `huỳnh`).
//! 4. Vần mở 3 âm → âm giữa (`ngoài`, `khuya`, `ngoèo`, `khuỷu`).
//! 5. Vần mở 2 âm: `oa`/`oe`/`uy` theo kiểu dấu — MỚI `hoà khoẻ thuỷ`, CŨ `hòa khỏe thủy`;
//!    còn lại dấu ở âm đầu (`của`, `mía`, `chào`, `cái`, `chịu`).
//!
//! Bản cũ đặt dấu theo "âm cuối cụm" cho mọi vần nên gõ sai `của`→`cuả`, `nghĩa`→`nghiã`,
//! không gõ được `giáo`/`giữa`/`quý`, và kiểu CŨ còn đặt `được` thành `đựơc`.

use super::tone::is_vowel;
use super::vowel_table::{base_entry, locate};
use super::vowel_table_generated::{
    A, A_BREVE, A_CIRC, E, E_CIRC, I, O, O_CIRC, O_HOOK, U, U_HOOK, Y,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiacriticStyle {
    #[default]
    New,
    Old,
}

fn entry(c: char) -> Option<usize> {
    locate(c).map(|(e, _)| e)
}

fn has_diacritic(c: char) -> bool {
    matches!(
        entry(c),
        Some(A_BREVE | A_CIRC | E_CIRC | O_CIRC | O_HOOK | U_HOOK)
    )
}

/// Tìm index ký tự cần đặt dấu trong `out` (từ đang gõ). Không có âm → `None`.
pub fn pick_tone_target(out: &[char], style: DiacriticStyle) -> Option<usize> {
    let last_vowel = out.iter().rposition(|&c| is_vowel(c))?;
    let mut start = last_vowel;
    while start > 0 && is_vowel(out[start - 1]) {
        start -= 1;
    }
    // (1) `qu` / `gi` + nguyên âm: bán âm thuộc phụ âm đầu.
    if start < last_vowel && start > 0 {
        let prev = out[start - 1].to_ascii_lowercase();
        let first = entry(out[start]);
        if (prev == 'q' && first == Some(U)) || (prev == 'g' && first == Some(I)) {
            start += 1;
        }
    }
    if start == last_vowel {
        return Some(start);
    }
    let run = &out[start..=last_vowel];
    // (2) nguyên âm có dấu phụ — cái sau cùng.
    if let Some(i) = run.iter().rposition(|&c| has_diacritic(c)) {
        return Some(start + i);
    }
    // (3) có phụ âm cuối.
    if last_vowel + 1 < out.len() {
        return Some(last_vowel);
    }
    // (4) vần mở 3 âm.
    if run.len() >= 3 {
        return Some(start + 1);
    }
    // (5) vần mở 2 âm.
    let pair = (entry(run[0]).map(base_entry), entry(run[1]).map(base_entry));
    if matches!(pair, (Some(O), Some(A | E)) | (Some(U), Some(Y))) {
        return Some(match style {
            DiacriticStyle::New => last_vowel,
            DiacriticStyle::Old => start,
        });
    }
    Some(start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::tone::tone_of;

    fn target(word: &str, style: DiacriticStyle) -> char {
        let out: Vec<char> = word.chars().collect();
        out[pick_tone_target(&out, style).expect("có âm")]
    }

    #[test]
    fn style_only_changes_open_oa_oe_uy() {
        for (w, new, old) in [
            ("hoa", 'a', 'o'),
            ("khoe", 'e', 'o'),
            ("thuy", 'y', 'u'),
            ("hoan", 'a', 'a'),
            ("đươc", 'ơ', 'ơ'),
            ("khuyên", 'ê', 'ê'),
            ("chao", 'a', 'a'),
        ] {
            assert_eq!(target(w, DiacriticStyle::New), new, "{w} (mới)");
            assert_eq!(target(w, DiacriticStyle::Old), old, "{w} (cũ)");
        }
    }

    #[test]
    fn spelling_rules() {
        for (w, t) in [
            ("cua", 'u'),
            ("cưa", 'ư'),
            ("nghia", 'i'),
            ("kia", 'i'),
            ("quy", 'y'),
            ("quơ", 'ơ'),
            ("quyên", 'ê'),
            ("giao", 'a'),
            ("giưa", 'ư'),
            ("giup", 'u'),
            ("gi", 'i'),
            ("gin", 'i'),
            ("giêng", 'ê'),
            ("ngoai", 'a'),
            ("khuya", 'y'),
            ("ngoeo", 'e'),
            ("khuyu", 'y'),
            ("ngươi", 'ơ'),
            ("muôi", 'ô'),
            ("hươu", 'ơ'),
            ("thuơ", 'ơ'),
            ("tuân", 'â'),
            ("huê", 'ê'),
            ("chiu", 'i'),
            ("gưi", 'ư'),
            ("huynh", 'y'),
        ] {
            assert_eq!(target(w, DiacriticStyle::New), t, "{w}");
        }
        assert_eq!(tone_of('a'), Some(0));
    }

    #[test]
    fn no_vowel_none() {
        let out: Vec<char> = "bcd".chars().collect();
        assert_eq!(pick_tone_target(&out, DiacriticStyle::New), None);
    }
}
