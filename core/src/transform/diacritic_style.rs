// SPDX-License-Identifier: GPL-3.0-or-later
//! Vị trí đặt dấu trong cụm âm — bug B12: `hoà` (style mới) vs `hòa` (style cũ).
//!
//! Quy tắc (P0-3 §1.1 `diacritic_style`): sau các ngoại lệ hạt nhân tiếng
//! Việt, `New` (mặc định) đặt ở âm cuối (`hoa`+f → `hoà`), còn `Old` đặt ở âm
//! đầu khi cụm có từ hai âm (`hoa`+f → `hòa`).

use super::tone::is_vowel;
use super::undo::unmark;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiacriticStyle {
    #[default]
    New,
    Old,
}

/// Tìm index ký tự cần đặt dấu trong `out`.
///
/// `New`/`Old` chỉ khác nhau ở những vần còn lại như `oa/oe`. Các vần
/// lướt có đuôi `i/u/y` (như `ươi`, `uôi`) không đặt dấu lên bán âm cuối; các
/// vần `ao/au/ay/eo/eu/iu/oi/ui` đặt vào nguyên âm đầu. Không có âm → `None`.
pub fn pick_tone_target(out: &[char], style: DiacriticStyle) -> Option<usize> {
    let last_vowel = out.iter().rposition(|&c| is_vowel(c))?;
    let mut start = last_vowel;
    while start > 0 && is_vowel(out[start - 1]) {
        start -= 1;
    }
    let run_len = last_vowel - start + 1;

    if run_len == 1 {
        return Some(last_vowel);
    }

    let run: Vec<char> = out[start..=last_vowel].iter().map(|&c| unmark(c)).collect();
    let last = *run.last()?;
    // i/u/y cuối vần là bán âm: người, muối, chuỗi.
    if matches!(last, 'i' | 'I' | 'u' | 'U' | 'y' | 'Y') {
        return Some(last_vowel - 1);
    }

    // Các vần đôi có nguyên âm đầu là hạt nhân: chào, đau, ngày, nghèo,
    // kêu, chịu, hỏi, túi. Không áp dụng cho oa/oe/ua để vẫn tôn trọng
    // `diacritic_style` (hoà/hòa).
    if run_len == 2
        && matches!(
            (run[0].to_ascii_lowercase(), last.to_ascii_lowercase()),
            ('a', 'o' | 'u' | 'y') | ('e', 'o' | 'u') | ('i', 'u') | ('o', 'i') | ('u', 'i' | 'y')
        )
    {
        return Some(start);
    }

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
    fn lexical_nuclei_do_not_tone_semivowels() {
        assert_eq!(
            pick_tone_target(&chars("chao"), DiacriticStyle::New),
            Some(2)
        );
        assert_eq!(
            pick_tone_target(&chars("ngươi"), DiacriticStyle::New),
            Some(3)
        );
        assert_eq!(
            pick_tone_target(&chars("muôi"), DiacriticStyle::New),
            Some(2)
        );
    }

    #[test]
    fn no_vowel_none() {
        assert_eq!(pick_tone_target(&chars("bcd"), DiacriticStyle::New), None);
    }
}
