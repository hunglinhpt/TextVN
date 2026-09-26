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

use super::DiacriticStyle;
use crate::transform::pick_tone_target;
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke};
use crate::transform::tone::{apply_tone, is_vowel, key_to_tone, tone_of};
use crate::transform::undo::unmark;
use crate::transform::vowel_table::{
    form_like, locate, A, A_BREVE, A_CIRC, E, E_CIRC, O, O_CIRC, O_HOOK, U, U_HOOK, Y,
};

/// Fold chuỗi phím của một từ → chuỗi hiển thị.
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(raw.len());
    for &c in raw {
        push_key(&mut out, c, style, free_marking);
    }
    out
}

fn push_key(out: &mut Vec<char>, c: char, style: DiacriticStyle, free: bool) {
    // 1) Key dấu thanh
    if let Some(tone) = key_to_tone(c) {
        apply_tone_key(out, tone, c, style, free);
        return;
    }

    // 2) 'w' — horn / undo horn / nuốt lặp
    if c == 'w' {
        match horn(out) {
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

    // iet: e trước t (và trước e là i) → ê (P0-4 ví dụ B1: viet → viêt)
    if (last == 't' || last == 'T') && len >= 3 {
        let e_char = out[len - 2];
        let i_char = out[len - 3];
        if (i_char == 'i' || i_char == 'I') && (e_char == 'e' || e_char == 'E') {
            out[len - 2] = if e_char == 'E' { 'Ê' } else { 'ê' };
        }
    }
}

enum Horn {
    Applied,
    Undone,
    No,
}

/// `w`: aw→ă · ow→ơ · uw→ư; đã sừng → về gốc; không hợp lệ → No.
fn horn(out: &mut [char]) -> Horn {
    let Some(idx) = out.iter().rposition(|&c| is_vowel(c)) else {
        return Horn::No;
    };
    let ch = out[idx];
    let Some((e, t)) = locate(ch) else {
        return Horn::No;
    };
    let new_e = match e {
        A => A_BREVE,
        O => O_HOOK,
        U => U_HOOK,
        A_BREVE | O_HOOK | U_HOOK => {
            let base = match e {
                A_BREVE => A,
                O_HOOK => O,
                _ => U,
            };
            out[idx] = form_like(ch, base, t); // undo horn, giữ dấu thanh
            return Horn::Undone;
        }
        _ => return Horn::No,
    };
    out[idx] = form_like(ch, new_e, t);
    Horn::Applied
}

fn circum_pair(c: char) -> Option<(usize, usize)> {
    match c {
        'a' | 'A' => Some((A, A_CIRC)),
        'e' | 'E' => Some((E, E_CIRC)),
        'o' | 'O' => Some((O, O_CIRC)),
        _ => None,
    }
}

/// Áp một key dấu thanh vào `out`.
fn apply_tone_key(out: &mut Vec<char>, tone: usize, key: char, style: DiacriticStyle, free: bool) {
    // free_marking=false (Telex chặt): key phải nằm ngay sau âm
    if !free && !out.last().map(|&l| is_vowel(l)).unwrap_or(false) {
        out.push(key);
        return;
    }
    let Some(tidx) = pick_tone_target(out, style) else {
        out.push(key); // chưa có âm nào → đây là chữ thường ("sun")
        return;
    };
    let cur = tone_of(out[tidx]).unwrap_or(0);
    if cur == tone {
        // Bấm lại đúng dấu → bỏ dấu + gõ literal (ass → as)
        out[tidx] = unmark(out[tidx]);
        out.push(key);
    } else {
        // Áp / thay dấu (free_marking)
        out[tidx] = apply_tone(out[tidx], tone);
    }
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
