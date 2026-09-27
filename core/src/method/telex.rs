// SPDX-License-Identifier: GPL-3.0-or-later
//! Telex — fold từng phím, dò lại từ đầu mỗi lần (stateless theo `raw`).
//!
//! Quy tắc (kế thừa UniKey/x-unikey golden + Wikipedia Telex + P0-4 §8):
//! - Dấu thanh: `s f r x j` (chỉ lowercase) → áp lên âm chọn theo `diacritic_style`.
//!   Bấm lại đúng dấu → bỏ dấu + gõ literal (`ass` → `as`).
//! - `aw`→ă · `ow`→ơ · `uw`→ư; bấm lại `w` khi đã sừng → về gốc (`uww`→`u`);
//!   `w` không có âm đích → chữ thường, key lặp bị nuốt (`ww` → `w`).
//! - Đôi: `aa`→â · `ee`→ê · `oo`→ô · `dd`→đ; bấm lại → gỡ (`eee`→`e`, `ddd`→`d`).
//! - `d` ngay trước âm viết → `đ` (golden `duocj`→`được`; `y` KHÔNG tính).
//! - Cặp `uo`: gõ `o` ngay sau `u` → `ư`+`ơ` (golden `duocj`, `duong`→`đường`).
//! - `iet`: `e` ngay trước `t` mà trước nữa là `i` → `ê` (ví dụ B1 P0-4: `viet`→`viêt`).
//!
//! Mỗi key xử lý O(len từ) — từ tiếng Việt ngắn, nằm trong ngân sách
//! `ime_key` < 0.5 ms p99 (P0-3 §3.3).

use super::keys_generated::{simple_telex as keys_st, telex as keys};
use super::DiacriticStyle;
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke};
use crate::transform::tone::{apply_key, is_vowel, tone_of};
use crate::transform::vowel_table::{form_like, locate, E, E_CIRC, O_CIRC, O_HOOK, U, U_HOOK, Y};

/// Fold chuỗi phím của một từ → chuỗi hiển thị.
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    fold_with(raw, style, free_marking, true)
}

/// `w_marker = false` → "simple telex": `w` là **chữ thường** (không có ă/ơ/ư qua `w`),
/// nên `ww` → `ww` (không nuốt lặp). Dùng bởi `method/simple_telex.rs` (P0-1 §1).
pub fn fold_with(
    raw: &[char],
    style: DiacriticStyle,
    free_marking: bool,
    w_marker: bool,
) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(raw.len());
    for &c in raw {
        push_key(&mut out, c, style, free_marking, w_marker);
    }
    out
}

fn push_key(out: &mut Vec<char>, c: char, style: DiacriticStyle, free: bool, w_marker: bool) {
    // 1) Key dấu thanh — bảng data (Simple Telex dùng chung bảng, chỉ khác `w`)
    let tone = if w_marker {
        keys::tone_of_key(c)
    } else {
        keys_st::tone_of_key(c)
    };
    if let Some(tone) = tone {
        apply_key(out, tone, c, style, free);
        return;
    }

    // 2) 'w' — horn / undo horn / nuốt lặp (simple telex: không phải marker → đi tiếp)
    if c == 'w' && w_marker {
        match horn(out, &keys::HORN) {
            Horn::Applied | Horn::Undone => return,
            Horn::No => {
                if out.last() == Some(&'w') {
                    return; // ww → w
                }
                out.push(c);
                return;
            }
        }
    }

    // 3) Đôi circumflex: ee→ê (eee→e), aa→â, oo→ô
    if let Some((plain, circ)) = circum_pair(c) {
        if let Some(&last) = out.last() {
            // `uo` được fold tạm thành `ươ` để `duocj` → `được`. Khi người
            // dùng tiếp tục `oo` (chuoi → chuôi), o thứ hai xác nhận vần uô,
            // nên đảo cặp tạm này thay vì để thành `ươo`.
            if c.eq_ignore_ascii_case(&'o')
                && out.len() >= 2
                && matches!(locate(last), Some((O_HOOK, _)))
                && matches!(locate(out[out.len() - 2]), Some((U_HOOK, 0)))
            {
                let idx = out.len() - 1;
                out[idx - 1] = form_like(out[idx - 1], U, 0);
                out[idx] = form_like(last, O_CIRC, tone_of(last).unwrap_or(0));
                return;
            }
            if let Some((le, lt)) = locate(last) {
                if le == circ {
                    let idx = out.len() - 1;
                    out[idx] = form_like(last, plain, lt); // gỡ circumflex, giữ dấu thanh
                    return;
                }
                if le == plain {
                    let idx = out.len() - 1;
                    out[idx] = form_like(last, circ, lt);
                    return;
                }
            }
        }
    }

    // 4) 'd' đôi: dd→đ, ddd→d (gỡ)
    if is_plain_d(c) {
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
        return;
    }

    // 5) Ký tự thường: thêm vào rồi các rule phụ
    out.push(c);
    post_fixes(out);
}

/// Các rule cần biết ký tự đứng trước/kế bên sau khi đã push.
fn post_fixes(out: &mut [char]) {
    let len = out.len();
    if len < 2 {
        return;
    }
    let prev = out[len - 2];
    let last = out[len - 1];

    // d + âm viết (trừ y) → đ — golden `duocj`; 'y' loại để khỏi phá "dying/dynamics"
    if is_plain_d(prev) && is_vowel(last) {
        if let Some((e, _)) = locate(last) {
            if e != Y {
                out[len - 2] = to_stroke(prev);
            }
        }
        return;
    }

    // Cặp uo: o ngay sau u → ư + ơ (golden duocj / duong)
    if (last == 'o' || last == 'O') && (prev == 'u' || prev == 'U') {
        out[len - 2] = form_like(prev, U_HOOK, tone_of(prev).unwrap_or(0));
        out[len - 1] = form_like(last, O_HOOK, tone_of(last).unwrap_or(0));
        return;
    }

    // `iet` → `iêt` (P0-4 B1); với `e` đã mang dấu thanh trước phụ âm cuối
    // `t`, Telex cũng hiểu đó là `ê` (texts: te + x + t + s → tết).
    // Phải giữ tone đang có khi đổi e → ê.
    if (last == 't' || last == 'T') && len >= 3 {
        let e_char = out[len - 2];
        let i_char = out[len - 3];
        if let Some((entry, tone)) = locate(e_char) {
            if entry == E && (i_char == 'i' || i_char == 'I' || tone != 0) {
                out[len - 2] = form_like(e_char, E_CIRC, tone);
            }
        }
    }
}

enum Horn {
    Applied,
    Undone,
    No,
}

/// `w`: aw→ă · ow→ơ · uw→ư; đã sừng → về gốc; không hợp lệ → No.
/// Cặp (âm gốc → âm sừng) lấy từ bảng `telex::HORN` trong `data/tables/telex.toml`.
fn horn(out: &mut [char], table: &[(char, usize, usize)]) -> Horn {
    let Some(idx) = out.iter().rposition(|&c| is_vowel(c)) else {
        return Horn::No;
    };
    let ch = out[idx];
    let Some((e, t)) = locate(ch) else {
        return Horn::No;
    };
    for &(_, from, to) in table {
        if e == to {
            out[idx] = form_like(ch, from, t); // undo horn, giữ dấu thanh
            return Horn::Undone;
        }
        if e == from {
            out[idx] = form_like(ch, to, t);
            return Horn::Applied;
        }
    }
    Horn::No
}

/// Cặp đôi âm → mũ: `aa`→â · `ee`→ê · `oo`→ô (bảng `telex::CIRCUMFLEX`).
fn circum_pair(c: char) -> Option<(usize, usize)> {
    keys::CIRCUMFLEX
        .iter()
        .find(|&&(k, _, _)| k.eq_ignore_ascii_case(&c))
        .map(|&(_, from, to)| (from, to))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str, style: DiacriticStyle) -> String {
        fold(&s.chars().collect::<Vec<_>>(), style, true)
            .into_iter()
            .collect()
    }

    fn n(s: &str) -> String {
        f(s, DiacriticStyle::New)
    }

    #[test]
    fn golden_duocj() {
        // P0-4 §2.3 / PLAN §8
        assert_eq!(n("duocj"), "được");
    }

    #[test]
    fn golden_undo_ass_and_ww() {
        assert_eq!(n("ass"), "as");
        assert_eq!(n("ww"), "w");
    }

    #[test]
    fn golden_hoaf_styles() {
        assert_eq!(f("hoaf", DiacriticStyle::New), "hoà");
        assert_eq!(f("hoaf", DiacriticStyle::Old), "hòa");
    }

    #[test]
    fn english_identity() {
        // Lưu ý: word chứa key dấu (r/f/s/x/j) sau âm KHÔNG phải identity —
        // đó là bản chất Telex (Wikipedia: dính dấu khi gõ xen kẽ tiếng Anh;
        // B5 auto-restore sẽ lo phần này ở slice sau).
        for w in ["hello", "abc", "sun", "cat"] {
            assert_eq!(n(w), w, "{w} phải identity khi không có transform");
        }
    }

    #[test]
    fn stroke_dd_and_d_vowel() {
        assert_eq!(n("dd"), "đ");
        // dùng escape cho âm sừng để không phụ thuộc normalization của literal
        assert_eq!(n("duong"), "\u{111}\u{1B0}\u{1A1}ng");
        assert_eq!(n("ddo"), "đo");
        assert_eq!(n("dy"), "dy"); // y không kích hoạt đ
    }

    #[test]
    fn horn_rules() {
        assert_eq!(n("trawng"), "trăng"); // aw → ă
        assert_eq!(n("mow"), "mơ"); // ow → ơ
        assert_eq!(n("tuw"), "tư"); // uw → ư
        assert_eq!(n("uww"), "u"); // undo horn
    }

    #[test]
    fn circumflex_double() {
        assert_eq!(n("bee"), "bê");
        assert_eq!(n("beee"), "be"); // gỡ ê
        assert_eq!(n("caan"), "cân");
        assert_eq!(n("nhoo"), "nhô");
        assert_eq!(n("ddd"), "d"); // gỡ đ
    }

    #[test]
    fn viet_ie_t_rule_p0_4_b1() {
        assert_eq!(n("viet"), "viêt");
        assert_eq!(n("mien"), "mien"); // không có t → không đổi
        assert_eq!(n("texts"), "tết"); // dấu x có trước t vẫn giữ khi e → ê
    }

    #[test]
    fn tone_rules() {
        // "duocs" → đ + ư + ớ(c) + c — dùng escape âm sừng/toned
        assert_eq!(n("duocs"), "\u{111}\u{1B0}\u{1EDB}c");
        assert_eq!(n("hoaf"), "hoà");
        assert_eq!(n("viets"), "viết"); // tone trên ê sau rule iet (fold từng key một)
        assert_eq!(n("asf"), "à"); // dấu khác thay nhau: sắc → huyền (không literal)
        assert_eq!(n("ass"), "as"); // cùng dấu → undo + literal
    }

    #[test]
    fn uppercase_supported() {
        assert_eq!(n("Duocj"), "Được");
        assert_eq!(n("NGAf"), "NGÀ"); // key f thường vẫn đặt dấu lên 'A' IN HOA
        assert_eq!(n("NGAF"), "NGAF"); // key IN HOA = chữ thường lệ, không phải dấu
    }

    #[test]
    fn strict_mode_requires_adjacent_marker() {
        let strict = fold(
            &"duocj".chars().collect::<Vec<_>>(),
            DiacriticStyle::New,
            false,
        );
        // strict: 'j' không nằm ngay sau âm → literal; đ/cặp uo vẫn áp (không thuộc free_marking)
        assert_eq!(strict.iter().collect::<String>(), "đươcj");
        // free marking (mặc định): dấu tự do
        assert_eq!(f("duocj", DiacriticStyle::New), "được");
    }
}
