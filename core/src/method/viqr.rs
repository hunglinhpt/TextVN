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
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke}; // is_stroke: nhánh undo dd
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
///
/// Vòng 43 (Pass D1 F1): freeze marker kiểu **E28** — marker đã bị UNDO
/// (bấm lặp đúng dấu/mũ/sừng/gạch) thì các lần gõ **cùng marker** kế tiếp đi
/// thẳng literal, chấm dứt chu kỳ toggle vô tận (`hoa...` từng bật lại dấu
/// nặng, `dddd` từng gạch lại chữ `d` — Telex/VNI đã freeze từ 0.2.42, VIQR
/// bị bỏ sót). State per-word: `fold` xử lý nguyên từ từ đầu nên state là
/// biến cục bộ của 1 lần fold.
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(raw.len());
    let mut undone_tone = [false; 6]; // index 1..=5 (0 bỏ)
    let mut undone_mark = [false; 3]; // ^ ( +
    let mut undone_stroke = false;
    for &c in raw {
        push_key(
            &mut out,
            c,
            style,
            free_marking,
            &mut undone_tone,
            &mut undone_mark,
            &mut undone_stroke,
        );
    }
    out
}

fn marker_mark_index(c: char) -> Option<usize> {
    if keys::CIRCUMFLEX.iter().any(|&(k, _, _)| k == c) {
        Some(0)
    } else if keys::HORN.iter().any(|&(k, _, _)| k == c) {
        Some(1)
    } else if keys::BREVE.iter().any(|&(k, _, _)| k == c) {
        Some(2)
    } else {
        None
    }
}

fn push_key(
    out: &mut Vec<char>,
    c: char,
    style: DiacriticStyle,
    free: bool,
    undone_tone: &mut [bool; 6],
    undone_mark: &mut [bool; 3],
    undone_stroke: &mut bool,
) {
    if let Some(tone) = keys::tone_of_key(c) {
        // Freeze: marker dấu thanh này đã undo → literal.
        if undone_tone[tone] {
            out.push(c);
            return;
        }
        if apply_key(out, tone, c, style, free) {
            undone_tone[tone] = true; // bấm lặp đúng dấu → đã gỡ
        }
        return;
    }
    // ^ mũ · + sừng · ( breve — lấy từ bảng data.
    if let Some(mi) = marker_mark_index(c) {
        // Freeze: marker mũ/sừng/breve này đã undo → literal.
        if undone_mark[mi] {
            out.push(c);
            return;
        }
        let marked = if mi == 1 {
            mark_horn(out, c, &keys::HORN)
        } else if mi == 2 {
            mark_vowel(out, c, &keys::BREVE)
        } else {
            mark_vowel(out, c, &keys::CIRCUMFLEX)
        };
        if marked == Some(crate::transform::undo::Marked::Undone) {
            undone_mark[mi] = true; // đã mũ/sừng → gỡ + literal
        }
    } else if is_plain_d(c) && c.eq_ignore_ascii_case(&keys::STROKE_KEY) {
        // dd → đ, ddd → dd: gỡ + gõ chữ `d` như Telex (R2-63; STROKE_DOUBLE = true cho VIQR).
        // Vòng 43 (Pass D1 F4): `D` hoa cũng là phím gạch (DD→Đ như Telex).
        if *undone_stroke {
            out.push(c);
            return;
        }
        if let Some(&last) = out.last() {
            if is_plain_d(last) {
                let idx = out.len() - 1;
                out[idx] = to_stroke(last);
                return;
            }
            if is_stroke(last) {
                let idx = out.len() - 1;
                out[idx] = to_plain(last);
                out.push(c);
                *undone_stroke = true; // đã gạch → gỡ + literal
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
        assert_eq!(n("ddd"), "dd");
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

#[cfg(test)]
mod freeze_tests {
    use super::*;

    fn n(s: &str) -> String {
        fold(&s.chars().collect::<Vec<_>>(), DiacriticStyle::New, true)
            .into_iter()
            .collect()
    }

    /// Vòng 43 (Pass D1 F1): freeze marker VIQR — hết toggle vô tận (E28).
    #[test]
    fn freeze_marker_lap() {
        // Dấu thanh: undo 1 lần rồi literal.
        assert_eq!(n("hoa."), "hoạ");
        assert_eq!(n("hoa.."), "hoa.");
        assert_eq!(n("hoa..."), "hoa..");
        assert_eq!(n("hoa...."), "hoa...");
        // Gạch dd: undo 1 lần rồi literal.
        assert_eq!(n("dd"), "đ");
        assert_eq!(n("ddd"), "dd");
        assert_eq!(n("dddd"), "ddd");
        // Mũ: undo 1 lần rồi literal.
        assert_eq!(n("a^"), "â");
        assert_eq!(n("a^^"), "a^");
        assert_eq!(n("a^^^"), "a^^");
        // Marker KHÁC nhau không freeze lẫn nhau (free_marking thay dấu).
        assert_eq!(n("hoa.~"), "hoã");
        // "." thứ hai là APPLY lại (thay ngã bằng nặng) — không phải undo.
        assert_eq!(n("hoa.~."), "hoạ");
    }

    /// Vòng 43 (Pass D1 F4): `D` hoa cũng là phím gạch (DD→Đ như Telex).
    #[test]
    fn stroke_hoa() {
        assert_eq!(n("DD"), "Đ");
        assert_eq!(n("Dd"), "Đ");
        // "dD": gạch áp lên chữ ĐẦU — case theo chữ đầu (như Telex/UniKey).
        assert_eq!(n("dD"), "đ");
        assert_eq!(n("DDD"), "DD");
    }
}
