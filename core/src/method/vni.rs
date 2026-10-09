// SPDX-License-Identifier: GPL-3.0-or-later
//! VNI (P0-1 §1 `method/vni.rs`) — kiểu gõ dùng **chữ số**:
//!
//! | Key | Tác dụng |
//! |---|---|
//! | `1` `2` `3` `4` `5` | sắc · huyền · hỏi · ngã · nặng |
//! | `6` | `â` `ê` `ô` (mũ) |
//! | `7` | `ơ` `ư` (sừng) — cụm `uo` → `ươ` |
//! | `8` | `ă` (breve) |
//! | `9` | `đ` |
//! | `0` | **xoá toàn bộ dấu** của từ đang gõ (thanh + dạng âm + `đ`) |
//!
//! - Marker không áp được (không có âm đích) → gõ **literal** (`abc6` → `abc6` khi không
//!   có âm nào nhận mũ). Bấm lại marker đã áp → gỡ dạng + literal (`a66` → `a6`).
//! - Chữ số là **một phần của từ** (không phải ranh giới) → `method::is_word_char`.
//! - Quy ước undo marker chưa có oracle UniKey → ghi `docs/10-shared/P0-REVIEW-LOG.md`.

use super::keys_generated::vni as keys;
use super::DiacriticStyle;
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke};
use crate::transform::tone::apply_key;
use crate::transform::undo::{mark_horn, mark_vowel};
use crate::transform::vowel_table::{base_entry, form_like, locate};

/// Chữ số 0..=9 là marker của VNI (engine coi là ký tự của từ, không phải ranh giới).
/// Nguồn: `data/tables/vni.toml` → `keys_generated::vni::is_marker`.
pub fn is_marker(c: char) -> bool {
    keys::is_marker(c)
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
    // 1..5 — dấu thanh (dùng chung logic Telex: bấm lại đúng dấu → literal)
    if let Some(tone) = keys::tone_of_key(c) {
        apply_key(out, tone, c, style, free);
        return;
    }
    // 6 — mũ · 7 — sừng · 8 — breve: tất cả lấy từ bảng data, không hardcode.
    if keys::CIRCUMFLEX.iter().any(|&(k, _, _)| k == c) {
        mark_vowel(out, c, &keys::CIRCUMFLEX);
    } else if keys::HORN.iter().any(|&(k, _, _)| k == c) {
        mark_horn(out, c, &keys::HORN);
    } else if keys::BREVE.iter().any(|&(k, _, _)| k == c) {
        mark_vowel(out, c, &keys::BREVE);
    } else if keys::REMOVE_MARKS_KEY == Some(c) {
        // 0 chi la phim xoa dau khi thuc su co dau de xoa. Neu khong,
        // giu literal de khong nuot so dien thoai/OTP ("200" -> "200").
        if !unmark_all(out) {
            out.push(c);
        }
    } else if c == keys::STROKE_KEY {
        // 9 — đ
        stroke(out, c);
    } else if is_plain_d(c) {
        // `d` + `d` → đ (VNI dùng `9`, nhưng giữ thói quen gõ đôi cho dễ dùng)
        if let Some(&last) = out.last() {
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

/// `9`: `d` gần nhất → `đ`; đã `đ` → gỡ + literal; không có `d` → literal.
fn stroke(out: &mut Vec<char>, key: char) {
    let Some(idx) = out.iter().rposition(|&ch| is_plain_d(ch) || is_stroke(ch)) else {
        out.push(key);
        return;
    };
    if is_stroke(out[idx]) {
        out[idx] = to_plain(out[idx]);
        out.push(key);
    } else {
        out[idx] = to_stroke(out[idx]);
    }
}

/// `0`: gỡ mọi dấu trong `out` (thanh + mũ/sừng/breve + `đ`) — giữ case.
fn unmark_all(out: &mut [char]) -> bool {
    let mut changed = false;
    for ch in out.iter_mut() {
        if let Some((e, _)) = locate(*ch) {
            let plain = form_like(*ch, base_entry(e), 0);
            changed |= plain != *ch;
            *ch = plain;
        } else if is_stroke(*ch) {
            *ch = to_plain(*ch);
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> String {
        fold(&s.chars().collect::<Vec<_>>(), DiacriticStyle::New, true)
            .into_iter()
            .collect()
    }

    fn o(s: &str) -> String {
        fold(&s.chars().collect::<Vec<_>>(), DiacriticStyle::Old, true)
            .into_iter()
            .collect()
    }

    #[test]
    fn golden_duong() {
        // d9 → đ · uo7 → ươ · 2 → huyền
        assert_eq!(n("d9uo7ng2"), "đường");
        assert_eq!(n("d9u7o7ng2"), "đường");
        assert_eq!(n("d9uong2"), "đuòng"); // không sừng → 2 áp lên 'o'
    }

    #[test]
    fn stroke_and_undo() {
        assert_eq!(n("d9"), "đ");
        assert_eq!(n("d99"), "d9"); // đã đ → gỡ + literal
        assert_eq!(n("a9"), "a9"); // không có d → literal
    }

    #[test]
    fn vowel_marks_and_undo() {
        assert_eq!(n("a6"), "â");
        assert_eq!(n("a66"), "a6");
        assert_eq!(n("e6"), "ê");
        assert_eq!(n("o6"), "ô");
        assert_eq!(n("a8"), "ă");
        assert_eq!(n("u7"), "ư");
        assert_eq!(n("o7"), "ơ");
        assert_eq!(n("i6"), "i6"); // i không nhận mũ → literal
        assert_eq!(n("uo77"), "uo7"); // cặp `ươ` gỡ cả cặp + literal (R2-63)
    }

    #[test]
    fn tone_replacement_and_styles() {
        assert_eq!(n("hoa2"), "hoà"); // style mới: dấu ở âm cuối cụm
        assert_eq!(o("hoa2"), "hòa"); // style cũ: dấu ở âm đầu cụm
        assert_eq!(o("hoa1"), "hóa");
        assert_eq!(n("hoa1"), "hoá");
    }

    #[test]
    fn remove_all_marks() {
        assert_eq!(n("d9uo7ng2"), "đường");
        assert_eq!(n("d9uo7ng20"), "duong"); // 0 xoá thanh + dạng âm + đ
        assert_eq!(n("d9uo7ng200"), "duong0"); // lần 0 kế tiếp là số thường
        assert_eq!(n("200"), "200");
        assert_eq!(n("0912"), "0912");
        assert_eq!(n("ab0"), "ab0");
        assert_eq!(n("0"), "0");
    }

    #[test]
    fn free_marking_off_requires_adjacent() {
        // dấu vẫn áp vì '2' nằm ngay sau âm cuối
        let strict = fold(
            &"hoa2".chars().collect::<Vec<_>>(),
            DiacriticStyle::New,
            false,
        );
        assert_eq!(strict.iter().collect::<String>(), "hoà");
        // '2' không nằm sau âm → literal (free_marking=false)
        let strict = fold(
            &"h2oa".chars().collect::<Vec<_>>(),
            DiacriticStyle::New,
            false,
        );
        assert_eq!(strict.iter().collect::<String>(), "h2oa");
    }

    #[test]
    fn digits_are_markers() {
        assert!(is_marker('7'));
        assert!(!is_marker('w'));
        assert_eq!(n("abc7"), "abc7"); // không có âm đích → literal
    }
}
