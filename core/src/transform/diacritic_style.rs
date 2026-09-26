// SPDX-License-Identifier: GPL-3.0-or-later
//! Vị trí đặt dấu trong cụm âm — bug B12: `hoà` (style mới) vs `hòa` (style cũ).
//!
//! Quy tắc (P0-3 §1.1 `diacritic_style`):
//! - `New` (mặc định): dấu ở **âm cuối** cụm âm liên tục (`hoa`+f → `hoà`).
//! - `Old`: dấu ở **âm đầu** cụm khi cụm ≥ 2 (`hoa`+f → `hòa`).

use super::tone::is_vowel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiacriticStyle {
    #[default]
    New,
    Old,
}

/// Tìm index ký tự cần đặt dấu trong `out`: âm cuối cùng, rồi giãn theo cụm âm.
/// Không có âm → `None`.
pub fn pick_tone_target(out: &[char], style: DiacriticStyle) -> Option<usize> {
    let last_vowel = out.iter().rposition(|&c| is_vowel(c))?;
    let mut start = last_vowel;
    while start > 0 && is_vowel(out[start - 1]) {
        start -= 1;
    }
    let run_len = last_vowel - start + 1;
    match style {
        DiacriticStyle::New => Some(last_vowel),
        DiacriticStyle::Old if run_len >= 2 => Some(start),
        DiacriticStyle::Old => Some(last_vowel),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn new_style_tone_on_last_of_run() {
        let out = chars("hoa");
        assert_eq!(pick_tone_target(&out, DiacriticStyle::New), Some(2)); // 'a'
    }

    #[test]
    fn old_style_tone_on_first_of_run() {
        let out = chars("hoa");
        assert_eq!(pick_tone_target(&out, DiacriticStyle::Old), Some(1)); // 'o'
    }

    #[test]
    fn run_with_leading_consonant() {
        let out = chars("đươc");
        // cụm "ươ" (index 1..2), âm cuối = ơ (index 2)
        assert_eq!(pick_tone_target(&out, DiacriticStyle::New), Some(2));
        assert_eq!(pick_tone_target(&out, DiacriticStyle::Old), Some(1));
    }

    #[test]
    fn no_vowel_none() {
        assert_eq!(pick_tone_target(&chars("bcd"), DiacriticStyle::New), None);
    }
}
