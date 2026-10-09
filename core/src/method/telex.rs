// SPDX-License-Identifier: GPL-3.0-or-later
//! Telex — fold từng phím, dò lại từ đầu mỗi lần (stateless theo `raw`).
//!
//! Quy tắc (kế thừa UniKey/x-unikey golden + Wikipedia Telex + P0-4 §8):
//! - Dấu thanh: `s f r x j` (chỉ lowercase) → áp lên âm chọn theo `diacritic_style`.
//!   Bấm lại đúng dấu → bỏ dấu + gõ literal (`ass` → `as`).
//! - `aw`→ă · `ow`→ơ · `uw`→ư; bấm lại `w` khi đã sừng → gỡ sừng + gõ chữ `w` (`uww`→`uw`,
//!   cặp `ươ` gỡ cả cặp: `uoww` → `uow`). `w` đứng riêng khi từ chưa có nguyên âm → `ư` như
//!   UniKey (`nhw` → `như`, R2-66); bấm `w` lần nữa → chữ `w` (`ww` → `w`, `www` → `ww`).
//! - Đôi: `aa`→â · `ee`→ê · `oo`→ô · `dd`→đ; bấm lại → gỡ + gõ chữ đó (`eee`→`ee`, `ddd`→`dd`,
//!   `xooong`→`xoong`) — như dấu thanh `ass` → `as` và UniKey `processRoof`/`processHook`/
//!   `processDd` (gỡ rồi `processAppend`, R2-63; bản cũ nuốt phím nên không gõ được `xoong`).
//! - `đ` CHỈ qua phím `d` (như UniKey/OpenKey/Bamboo): `d` + nguyên âm giữ nguyên `d` —
//!   bản cũ tự đổi thành `đ` nên không gõ được `dân`, `dạy`, `dưới`, `dược`…. `d` gõ sau
//!   trong từ đã biến đổi/có âm cuối đổi `d` đầu từ (`duocjwd` → `được`, R2-68, `late_stroke`).
//! - Cặp `uo`: gõ `o` ngay sau `u` → `ư`+`ơ` (`dduocj`→`được`, `dduongf`→`đường`), trừ
//!   `qu` (phụ âm `qu` + `ơ`: `quowr`→`quở`); `w` đầu tiên sau cặp tự tạo là xác nhận
//!   (`nguowif`→`người`). `ươ` đứng cuối từ là vần `uơ` (`thuowr`→`thuở`, `huow`→`huơ`).
//! - Không có luật tự thêm mũ: `viet` giữ `viet` (UniKey/OpenKey không có; R2-64 — luật `iet`
//!   cũ đọc nhầm ví dụ B1 của P0-4 nên `Viet`/`KIET`/`quiet` ra `Viêt`/`KIÊT`/`quiêt`).
//!
//! Mỗi key xử lý O(len từ) — từ tiếng Việt ngắn, nằm trong ngân sách
//! `ime_key` < 0.5 ms p99 (P0-3 §3.3).

use super::keys_generated::{simple_telex as keys_st, telex as keys};
use super::DiacriticStyle;
use crate::transform::stroke::{is_plain_d, is_stroke, to_plain, to_stroke};
use crate::transform::tone::{apply_key, is_vowel, remove_tone, tone_of};
use crate::transform::undo::{mark_cluster, Marked};
use crate::transform::vowel_table::{form_like, locate, O_CIRC, O_HOOK, U, U_HOOK};
use crate::validate::CODAS;

/// Fold chuỗi phím của một từ → chuỗi hiển thị.
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    fold_with(raw, style, free_marking, true)
}

/// `w_marker = false` → "simple telex": `w` **chỉ** là dấu sừng (`aw ow uw` → `ă ơ ư`, như
/// UniKey `vneHookAll`); không có âm nhận sừng thì là chữ `w` (`ww` → `ww`, không nuốt lặp).
/// Dùng bởi `method/simple_telex.rs` (P0-1 §1).
pub fn fold_with(
    raw: &[char],
    style: DiacriticStyle,
    free_marking: bool,
    w_marker: bool,
) -> Vec<char> {
    fold_with_caps(raw, style, free_marking, w_marker, false)
}

/// `caps_lock` = từ gõ khi Caps Lock bật: phím dấu viết hoa `S F R X J W Z` vẫn là phím
/// dấu (UniKey). Tắt Caps Lock thì chữ hoa (gõ bằng Shift) là chữ thường lệ (`USA`, `NGAF`).
pub fn fold_with_caps(
    raw: &[char],
    style: DiacriticStyle,
    free_marking: bool,
    w_marker: bool,
    caps_lock: bool,
) -> Vec<char> {
    let mut out: Vec<char> = Vec::with_capacity(raw.len());
    let mut ws = WState::default();
    for &c in raw {
        push_key(
            &mut out,
            c,
            style,
            free_marking,
            w_marker,
            caps_lock,
            &mut ws,
        );
    }
    out
}

/// Trạng thái phím `w` trong một từ (fold lại từ đầu mỗi phím nên chỉ sống trong `fold`).
#[derive(Default)]
struct WState {
    /// Đã gặp `w` trong từ (`w` đầu tiên xác nhận cặp `ươ` tự tạo).
    seen: bool,
    /// Phím vừa rồi là `w` đứng riêng đã thành `ư` (R2-66).
    mapped: bool,
    /// Đã gỡ `ư` → chữ `w` (`ww`) trong từ: các `w` sau là chữ thường (`www` → `ww`).
    literal: bool,
}

/// Từ đang có cặp `ươ` (do rule `uo` tự tạo khi chưa có `w` nào).
fn has_uo_pair(out: &[char]) -> bool {
    out.windows(2).any(|p| {
        matches!(locate(p[0]), Some((U_HOOK, _))) && matches!(locate(p[1]), Some((O_HOOK, _)))
    })
}

fn push_key(
    out: &mut Vec<char>,
    c: char,
    style: DiacriticStyle,
    free: bool,
    w_marker: bool,
    caps_lock: bool,
    ws: &mut WState,
) {
    let w_mapped = std::mem::take(&mut ws.mapped);
    // Phím dấu so theo chữ thường khi Caps Lock bật; ký tự gõ literal vẫn là `c` gốc.
    let key = if caps_lock { c.to_ascii_lowercase() } else { c };
    // 1) Key dấu thanh — bảng data (Simple Telex dùng chung bảng, chỉ khác `w`)
    let tone = if w_marker {
        keys::tone_of_key(key)
    } else {
        keys_st::tone_of_key(key)
    };
    if let Some(tone) = tone {
        apply_key(out, tone, c, style, free);
        return;
    }

    // 1b) `z` — gỡ dấu thanh (UniKey vneTone0: `toansz` → `toan`, mũ/sừng giữ nguyên).
    //     Từ chưa có dấu thanh → `z` là chữ thường (`pizza`, `zoo`).
    let tone_remove = if w_marker {
        keys::TONE_REMOVE_KEY
    } else {
        keys_st::TONE_REMOVE_KEY
    };
    if tone_remove == Some(key) && remove_tone(out) {
        return;
    }

    // 2) 'w' — horn / undo horn / `w` đứng riêng. Bảng sừng của từng kiểu gõ (Simple Telex
    //    cũng có `aw ow uw` — R2-61); chỉ Telex có `w` đứng riêng → `ư` (R2-66).
    let horn_table: &[(char, usize, usize)] = if w_marker {
        &keys::HORN
    } else {
        &keys_st::HORN
    };
    if horn_table.iter().any(|&(k, _, _)| k == key) {
        // `ww`: `w` vừa thành `ư` → gỡ thành chữ `w` (UniKey `ww` → `w`); các `w` sau trong từ
        // là chữ thường.
        if w_mapped {
            if let Some(last) = out.last_mut() {
                *last = c;
            }
            ws.literal = true;
            return;
        }
        // `uo` đã được rule tự đổi thành `ươ`: `w` đầu tiên của từ là XÁC NHẬN, không
        // phải bấm lại để gỡ — thói quen UniKey `nguowif` → `người`, `dduowcj` → `được`
        // (bản cũ ra `ngưòi`/`đưọc`). `w` kế tiếp mới gỡ như bình thường.
        let first_w = !ws.seen;
        ws.seen = true;
        if first_w && has_uo_pair(out) {
            return;
        }
        match horn(out, horn_table) {
            Horn::Applied => return,
            Horn::Undone => {
                out.push(c); // gỡ sừng + gõ chữ `w` (`uww` → `uw`, R2-63)
                return;
            }
            Horn::No => {
                // Telex: `w` đứng riêng (từ chưa có nguyên âm) → `ư` như UniKey `vne_telex_w`
                // (`nhw` → `như`, `tw` → `tư`, `wf` → `ừ`; R2-66 — bản cũ để chữ `w` mà vẫn nuốt
                // `w` thứ hai). Không có `qư` nên sau `q` vẫn là chữ `w` (`qwert`). Simple
                // Telex: chữ `w`.
                let after_q = out.last().is_some_and(|l| l.eq_ignore_ascii_case(&'q'));
                if w_marker && !ws.literal && !after_q && !out.iter().any(|&ch| is_vowel(ch)) {
                    out.push(if c.is_uppercase() { 'Ư' } else { 'ư' });
                    ws.mapped = true;
                    return;
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
                    out.push(c); // + gõ chữ đó (`xooong` → `xoong`, R2-63)
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

    // 4) 'd' đôi: dd→đ, ddd→dd (gỡ + gõ chữ `d`, R2-63)
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
                out.push(c);
                return;
            }
        }
        if free && late_stroke(out, c) {
            return;
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

    // Cặp uo: o ngay sau u → ư + ơ (`dduocj` → `được`) — trừ `qu` (u thuộc phụ âm đầu).
    let after_q = len >= 3 && matches!(out[len - 3], 'q' | 'Q');
    if (last == 'o' || last == 'O') && (prev == 'u' || prev == 'U') && !after_q {
        out[len - 2] = form_like(prev, U_HOOK, tone_of(prev).unwrap_or(0));
        out[len - 1] = form_like(last, O_HOOK, tone_of(last).unwrap_or(0));
    }
}

/// `d` gõ **sau** trong từ (đặt dấu tự do) → đổi `d` đầu từ thành `đ` như UniKey `processDd`
/// (vị trí phụ âm đầu, không chỉ ký tự liền trước): `duocjwd` → `được` (PLAN §8), `dieend` →
/// `điên`, `dongd` → `đong`; đầu từ đã là `đ` → gỡ + gõ chữ `d` (R2-63). R2-68.
///
/// Chỉ khi âm đầu đúng một `d` + nguyên âm, và từ đã có biến đổi tiếng Việt (dấu thanh/dấu
/// phụ) hoặc đã có phụ âm cuối hợp lệ — từ tiếng Anh chưa biến đổi như `did`, `dad`, `dead`,
/// `died` đi thẳng như cũ (đổi thành `đi`/`đa` thì là âm tiết Việt, auto-restore không cứu
/// được).
fn late_stroke(out: &mut Vec<char>, key: char) -> bool {
    let (Some(&first), Some(&second)) = (out.first(), out.get(1)) else {
        return false;
    };
    if !(is_plain_d(first) || is_stroke(first)) || !is_vowel(second) {
        return false;
    }
    let transformed = out.iter().any(|ch| !ch.is_ascii());
    let has_coda = out.iter().rposition(|&ch| is_vowel(ch)).is_some_and(|v| {
        v + 1 < out.len() && {
            let coda: String = out[v + 1..]
                .iter()
                .map(|ch| ch.to_ascii_lowercase())
                .collect();
            CODAS.contains(&coda.as_str())
        }
    });
    if !(transformed || has_coda) {
        return false;
    }
    if is_plain_d(first) {
        out[0] = to_stroke(first);
    } else {
        out[0] = to_plain(first);
        out.push(key);
    }
    true
}

enum Horn {
    Applied,
    Undone,
    No,
}

/// `w`: aw→ă · ow→ơ · uw→ư; đã sừng → về gốc; không hợp lệ → No.
/// Cặp (âm gốc → âm sừng) lấy từ bảng `telex::HORN` trong `data/tables/telex.toml`.
///
/// Âm nhận sừng chọn trên cả cụm nguyên âm cuối ([`mark_cluster`], R2-65) như UniKey
/// `processHook`: `w` gõ sau cả cụm vẫn đúng — `muaw` → `mưa`, `voiw` → `vơi`, `luuw` →
/// `lưu`, còn `quaw` → `quă`, `oaw` → `oă`.
fn horn(out: &mut [char], table: &[(char, usize, usize)]) -> Horn {
    let marked = mark_cluster(out, |w, idx| {
        let ch = w[idx];
        let (e, t) = locate(ch)?;
        for &(_, from, to) in table {
            if e == to {
                // Gỡ sừng, giữ dấu thanh; `ươ` gỡ cả cặp (UniKey: chuỗi `ươ` bỏ sừng là `uo`).
                w[idx] = form_like(ch, from, t);
                if e == O_HOOK && idx > 0 && matches!(locate(w[idx - 1]), Some((U_HOOK, _))) {
                    w[idx - 1] = form_like(w[idx - 1], U, tone_of(w[idx - 1]).unwrap_or(0));
                }
                return Some(Marked::Undone);
            }
            if e == from {
                w[idx] = form_like(ch, to, t);
                return Some(Marked::Applied);
            }
        }
        None
    });
    match marked {
        Some(Marked::Applied) => Horn::Applied,
        Some(Marked::Undone) => Horn::Undone,
        None => Horn::No,
    }
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
    fn d_only_becomes_stroke_when_doubled() {
        // Bản cũ: `d` + nguyên âm tự thành `đ` → không gõ được các từ này.
        for (keys, want) in [
            ("daan", "dân"),
            ("dajy", "dạy"),
            ("daif", "dài"),
            ("deex", "dễ"),
            ("duf", "dù"),
            ("duwowis", "dưới"),
            ("duocj", "dược"),
            ("dichj", "dịch"),
            ("ddi", "đi"),
            ("ddaan", "đân"),
            ("dddi", "ddi"),
        ] {
            assert_eq!(n(keys), want, "{keys}");
        }
    }

    /// R2-68: `d` gõ sau trong từ làm `đ` đầu từ (UniKey `processDd` khi đặt dấu tự do).
    #[test]
    fn late_d_strokes_the_onset() {
        for (keys, want) in [
            ("duocjwd", "được"),
            ("dieend", "điên"),
            ("dieendf", "điền"),
            ("dongd", "đong"),
            ("Dongd", "Đong"),
            // Đầu từ đã là `đ` → gỡ + gõ chữ `d` (như `ddd` → `dd`).
            ("DDuocjd", "Dượcd"),
        ] {
            assert_eq!(n(keys), want, "{keys}");
        }
        // Từ tiếng Anh chưa biến đổi, không phụ âm cuối: giữ nguyên.
        for w in ["did", "dad", "dead", "died", "dud", "dined"] {
            assert_eq!(n(w), w, "{w}");
        }
        // Đặt dấu chặt (free_marking = false): `d` phải liền sau `d`.
        let strict: String = fold(
            &"duocjwd".chars().collect::<Vec<_>>(),
            DiacriticStyle::New,
            false,
        )
        .into_iter()
        .collect();
        assert!(strict.ends_with('d'), "{strict}");
    }

    #[test]
    fn qu_and_open_uo() {
        // Đi qua `method::fold` (chuẩn hoá `qư`/`ươ` cuối từ dùng chung mọi kiểu gõ).
        let n = |s: &str| -> String {
            crate::method::fold(
                &s.chars().collect::<Vec<_>>(),
                crate::method::Method::Telex,
                DiacriticStyle::New,
                true,
            )
            .into_iter()
            .collect()
        };
        assert_eq!(n("quowr"), "quở");
        assert_eq!(n("quowf"), "quờ");
        assert_eq!(n("quoocs"), "quốc");
        assert_eq!(n("thuowr"), "thuở");
        assert_eq!(n("huow"), "huơ");
        assert_eq!(n("thuowng"), "thương");
        assert_eq!(n("muoons"), "muốn");
        assert_eq!(n("chuoongf"), "chuồng");
    }

    #[test]
    fn uow_like_unikey() {
        assert_eq!(n("dduowcj"), "được");
        assert_eq!(n("nguoiwf"), "người", "w sau cả cụm uoi");
        assert_eq!(n("nguowif"), "người");
        assert_eq!(n("truowngf"), "trường");
        assert_eq!(n("dduocwj"), "được", "w sau phụ âm cuối vẫn xác nhận ươ");
        assert_eq!(n("huowng"), "hương");
        // Cách gõ đầy đủ vẫn đúng như trước.
        assert_eq!(n("nguwowif"), "người");
        assert_eq!(n("dduwowcj"), "được");
        assert_eq!(n("dduocj"), "được");
        // w thứ hai mới gỡ sừng — cả cặp `ươ`, rồi gõ chữ `w` (R2-63).
        assert_eq!(n("uoww"), "uow");
    }

    #[test]
    fn z_removes_tone_only() {
        assert_eq!(n("toansz"), "toan");
        assert_eq!(n("vieetjz"), "viêt", "mũ giữ nguyên, chỉ gỡ dấu thanh");
        assert_eq!(n("dduowcjz"), "đươc");
        assert_eq!(n("asz"), "a");
        assert_eq!(n("aszz"), "az", "đã hết dấu → z là chữ");
        assert_eq!(n("pizza"), "pizza");
        assert_eq!(n("zoo"), "zô");
        assert_eq!(n("Z"), "Z", "Z hoa không phải phím dấu (như s f r x j)");
        let simple = |s: &str| -> String {
            fold_with(
                &s.chars().collect::<Vec<_>>(),
                DiacriticStyle::New,
                true,
                false,
            )
            .into_iter()
            .collect()
        };
        assert_eq!(simple("toansz"), "toan");
    }

    #[test]
    fn golden_duocj() {
        // P0-4 §2.3 / PLAN §8 — `đ` chỉ qua `dd` (UniKey); `duocj` là từ `dược`.
        assert_eq!(n("dduocj"), "được");
        assert_eq!(n("duocj"), "dược");
    }

    #[test]
    fn golden_undo_ass_and_ww() {
        assert_eq!(n("ass"), "as");
        assert_eq!(n("ww"), "w");
    }

    /// R2-66: Telex `w` đứng riêng (từ chưa có nguyên âm) → `ư` như UniKey `vne_telex_w`;
    /// bấm `w` lần nữa → chữ `w`, các `w` sau là chữ thường (`www` → `ww`).
    #[test]
    fn standalone_w_becomes_u_horn() {
        for (keys, want) in [
            ("w", "ư"),
            ("nhw", "như"),
            ("tw", "tư"),
            ("twf", "từ"),
            ("wf", "ừ"),
            ("nhwngx", "những"),
            ("ddwngf", "đừng"),
            ("chwa", "chưa"),
            ("wowcs", "ước"),
            ("ww", "w"),
            ("nhww", "nhw"),
            ("www", "ww"),
            ("qwe", "qwe"),
        ] {
            assert_eq!(n(keys), want, "{keys}");
        }
        // Đã có nguyên âm không nhận sừng → chữ `w` (không nuốt `w` lặp nữa).
        assert_eq!(n("view"), "view");
        assert_eq!(n("eww"), "eww");
        // `W` gõ bằng Shift (không Caps Lock) vẫn là chữ thường lệ như `S F R X J`.
        assert_eq!(n("We"), "We");
    }

    /// R2-63: bấm phím dấu lần ba → gỡ dấu **và** gõ chữ đó (như `ass` → `as`; UniKey
    /// processRoof/processHook/processDd gỡ rồi processAppend) — gõ được `xoong`, `goòng`.
    #[test]
    fn third_press_removes_mark_and_types_key() {
        for (keys, want) in [
            ("aaa", "aa"),
            ("eee", "ee"),
            ("ooo", "oo"),
            ("ddd", "dd"),
            ("aww", "aw"),
            ("uww", "uw"),
            ("xooong", "xoong"),
            ("booong", "boong"),
            ("gooongf", "goòng"),
            ("mooocs", "moóc"),
            ("DDD", "DD"),
        ] {
            assert_eq!(n(keys), want, "{keys}");
        }
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
        assert_eq!(n("dduong"), "\u{111}\u{1B0}\u{1A1}ng");
        assert_eq!(n("duong"), "d\u{1B0}\u{1A1}ng", "d đơn giữ nguyên (dương)");
        assert_eq!(n("ddo"), "đo");
        assert_eq!(n("dy"), "dy");
    }

    #[test]
    fn horn_rules() {
        assert_eq!(n("trawng"), "trăng"); // aw → ă
        assert_eq!(n("mow"), "mơ"); // ow → ơ
        assert_eq!(n("tuw"), "tư"); // uw → ư
        assert_eq!(n("uww"), "uw"); // gỡ sừng + gõ chữ w (R2-63)
    }

    /// R2-65: `w` gõ sau cả cụm nguyên âm nhắm đúng âm (UniKey `processHook` trên chuỗi âm).
    #[test]
    fn horn_after_whole_cluster() {
        for (keys, want) in [
            ("muaw", "mưa"),
            ("chuaw", "chưa"),
            ("cuawr", "cửa"),
            ("guiwr", "gửi"),
            ("luuw", "lưu"),
            ("voiws", "với"),
            ("giuawx", "giữa"),
            ("quaw", "quă"),
            ("hoawcs", "hoắc"),
            ("xoawns", "xoắn"),
            ("nguoiwf", "người"),
        ] {
            assert_eq!(n(keys), want, "{keys}");
        }
        // Bấm lại `w` trên cụm → gỡ sừng + gõ chữ `w` (R2-63).
        assert_eq!(n("muaww"), "muaw");
    }

    #[test]
    fn circumflex_double() {
        assert_eq!(n("bee"), "bê");
        assert_eq!(n("beee"), "bee"); // gỡ ê + gõ chữ e (R2-63)
        assert_eq!(n("caan"), "cân");
        assert_eq!(n("nhoo"), "nhô");
        assert_eq!(n("ddd"), "dd"); // gỡ đ + gõ chữ d (R2-63)
    }

    /// R2-64: không tự thêm mũ cho `iet` (UniKey/OpenKey không có luật này).
    #[test]
    fn no_implicit_circumflex_for_iet() {
        for w in ["viet", "Viet", "KIET", "diet", "quiet", "Kiet"] {
            assert_eq!(n(w), w, "{w} không được tự thêm mũ");
        }
        assert_eq!(n("vieetj"), "việt");
        assert_eq!(n("vieets"), "viết");
        assert_eq!(n("teexts"), "tết");
        assert_eq!(n("ghest"), "ghét");
    }

    #[test]
    fn tone_rules() {
        // "dduocs" → đ + ư + ớ + c — dùng escape âm sừng/toned
        assert_eq!(n("dduocs"), "\u{111}\u{1B0}\u{1EDB}c");
        assert_eq!(n("hoaf"), "hoà");
        assert_eq!(n("vieets"), "viết"); // tone trên ê (dấu phụ đứng trước)
        assert_eq!(n("asf"), "à"); // dấu khác thay nhau: sắc → huyền (không literal)
        assert_eq!(n("ass"), "as"); // cùng dấu → undo + literal
    }

    #[test]
    fn uppercase_supported() {
        assert_eq!(n("DDuocj"), "Được");
        assert_eq!(n("Dduocj"), "Được");
        assert_eq!(n("NGAf"), "NGÀ"); // key f thường vẫn đặt dấu lên 'A' IN HOA
        assert_eq!(n("NGAF"), "NGAF"); // key IN HOA = chữ thường lệ, không phải dấu
    }

    #[test]
    fn strict_mode_requires_adjacent_marker() {
        let strict = fold(
            &"dduocj".chars().collect::<Vec<_>>(),
            DiacriticStyle::New,
            false,
        );
        // strict: 'j' không nằm ngay sau âm → literal; đ/cặp uo vẫn áp (không thuộc free_marking)
        assert_eq!(strict.iter().collect::<String>(), "đươcj");
        // free marking (mặc định): dấu tự do
        assert_eq!(f("dduocj", DiacriticStyle::New), "được");
    }
}
