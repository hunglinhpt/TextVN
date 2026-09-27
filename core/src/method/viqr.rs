// SPDX-License-Identifier: GPL-3.0-or-later
//! VIQR (P0-1 §1 `method/viqr.rs`) — kiểu gõ dùng **ký tự dấu câu**:
//!
//! | Key | Tác dụng |
//! |---|---|
//! | `'` | sắc |
//! | `` ` `` | huyền |
//! | `?` | hỏi |
//! | `~` | ngã |
//! | `.` | nặng |
//! | `^` | `â` `ê` `ô` (mũ) |
//! | `(` | `ă` (breve) |
//! | `+` | `ơ` `ư` (sừng) — cụm `uo` → `ươ` |
//! | `dd` | `đ` |
//!
//! - Marker là **một phần của từ** (`method::is_word_char`) → gõ `.`/`?` trong câu vẫn đi qua
//!   engine: marker áp lên âm cuối, gõ lặp → gỡ + literal (`hoa..` → `hoa.`).
//! - Quy ước undo marker chưa có oracle UniKey → ghi `docs/10-shared/P0-REVIEW-LOG.md`.

use super::keys_generated::viqr as keys;
use super::DiacriticStyle;
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke};
use crate::transform::tone::apply_key;
use crate::transform::undo::{mark_horn, mark_vowel};

/// Marker VIQR (engine coi là ký tự của từ, không phải ranh giới).
/// Nguồn: `data/tables/viqr.toml` → `keys_generated::viqr::is_marker`.
pub fn is_marker(c: char) -> bool {
    keys::is_marker(c)
}

/// Key VIQR → index tone (1..=5). `None` nếu không phải dấu thanh.
pub fn marker_to_tone(c: char) -> Option<usize> {
    keys::tone_of_key(c)
}

/// Fold chuỗi phím của một từ → chuỗi hiển thị.
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(raw.len());
    for &c in raw {
        push_key(&mut out, c, style, free_marking);
    }
    out
}

fn push_key(out: &mut Vec<char>, c: char, style: DiacriticStyle, free: bool) {
    if let Some(tone) = keys::tone_of_key(c) {
        apply_key(out, tone, c, style, free);
        return;
    }
    // ^ mũ · + sừng · ( breve — lấy từ bảng data.
    if keys::CIRCUMFLEX.iter().any(|&(k, _, _)| k == c) {
        mark_vowel(out, c, &keys::CIRCUMFLEX);
    } else if keys::HORN.iter().any(|&(k, _, _)| k == c) {
        mark_horn(out, c, &keys::HORN);
    } else if keys::BREVE.iter().any(|&(k, _, _)| k == c) {
        mark_vowel(out, c, &keys::BREVE);
    } else if is_plain_d(c) && c == keys::STROKE_KEY {
        // dd → đ, ddd → d (STROKE_DOUBLE = true cho VIQR)
        if let Some(&last) = out.last() {
            if is_plain_d(last) {
                let idx = out.len() - 1;
                out[idx] = to_stroke(last);
                return;
            }
            if is_stroke(last) {
                let idx = out.len() - 1;
                out[idx] = to_plain(last);
                return;
            }
        }
        out.push(c);
    } else {
        out.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> String {
        fold(&s.chars().collect::<Vec<_>>(), DiacriticStyle::New, true)
            .into_iter()
            .collect()
    }

    #[test]
    fn golden_duong() {
        assert_eq!(n("ddu+o+ng`"), "đường");
        assert_eq!(n("dduong`"), "đuòng"); // không sừng → huyền lên 'o'
    }

    #[test]
    fn tones_and_undo() {
        assert_eq!(n("hoa'"), "hoá");
        assert_eq!(n("hoa`"), "hoà");
        assert_eq!(n("hoa?"), "hoả");
        assert_eq!(n("hoa~"), "hoã");
        assert_eq!(n("hoa."), "hoạ");
        assert_eq!(n("hoa.."), "hoa."); // bấm lại đúng dấu → literal
    }

    #[test]
    fn vowel_marks() {
        assert_eq!(n("a^"), "â");
        assert_eq!(n("e^"), "ê");
        assert_eq!(n("o^"), "ô");
        assert_eq!(n("a("), "ă");
        assert_eq!(n("u+"), "ư");
        assert_eq!(n("o+"), "ơ");
        assert_eq!(n("a^^"), "a^"); // đã mũ → gỡ + literal
    }

    #[test]
    fn stroke_dd() {
        assert_eq!(n("dd"), "đ");
        assert_eq!(n("ddd"), "d");
    }

    #[test]
    fn markers_are_word_chars() {
        assert!(is_marker('.'));
        assert!(is_marker('+'));
        assert!(!is_marker('w'));
        assert_eq!(marker_to_tone('\''), Some(1));
        assert_eq!(marker_to_tone('x'), None);
    }
}
