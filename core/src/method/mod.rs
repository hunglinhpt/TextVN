// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng phím → action theo từng kiểu gõ (P0-1 §1 `core/src/method/`).
//!
//! 4 kiểu gõ đã có bảng riêng: `telex`, `simple_telex`, `vni`, `viqr`.
//! Bảng phím là **data**: `data/tables/*.toml` → `cargo xtask gen-tables` → `method/keys_generated.rs`.
//! File từng method giữ phần **logic** (undo marker, cụm `uo`, rule `iet`) + đọc bảng đã sinh.

// == GENERATED từ `data/tables/{telex,simple_telex,vni,viqr}.toml` (`cargo xtask gen-tables`)
// — bảng phím của 4 kiểu gõ. KHÔNG sửa tay; logic (undo, cụm uo, iet…) vẫn ở file từng method.
pub mod keys_generated;

pub mod simple_telex;
pub mod telex;
pub mod viqr;
pub mod vni;

use crate::transform::DiacriticStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    #[default]
    Telex,
    Vni,
    Viqr,
    SimpleTelex,
}

impl Method {
    /// Tên method trong `config.v1.json` (P0-3 §1.1).
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Telex => "telex",
            Method::Vni => "vni",
            Method::Viqr => "viqr",
            Method::SimpleTelex => "simple_telex",
        }
    }
}

/// Ký tự thuộc **từ đang gõ** theo method?
///
/// Chữ cái luôn thuộc từ; VNI coi chữ số là marker (`d9`, `uo7`), VIQR coi ký tự dấu câu
/// là marker (`` ` `` `'` `?` `~` `.` `^` `(` `+`). Ký tự khác = **ranh giới từ**.
pub fn is_word_char(c: char, method: Method) -> bool {
    if c.is_alphabetic() {
        return true;
    }
    match method {
        Method::Vni => vni::is_marker(c),
        Method::Viqr => viqr::is_marker(c),
        Method::Telex | Method::SimpleTelex => false,
    }
}

/// Fold toàn bộ chuỗi phím của một từ → chuỗi hiển thị.
/// `raw` gồm cả marker đã bị nuốt; kết quả khác `raw` nghĩa là có biến đổi.
pub fn fold(raw: &[char], method: Method, style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    let mut out = match method {
        Method::Telex => telex::fold(raw, style, free_marking),
        Method::SimpleTelex => simple_telex::fold(raw, style, free_marking),
        Method::Vni => vni::fold(raw, style, free_marking),
        Method::Viqr => viqr::fold(raw, style, free_marking),
    };
    fix_uo(&mut out);
    // Dấu thanh gõ trước rồi mới gõ tiếp chữ (`hoaf` + `n`, `thuyr` + `eenf`): dời về đúng chỗ.
    crate::transform::tone::normalize_tone(&mut out, style);
    out
}

/// Chuẩn hoá cặp `u`/`ư` + `ơ` sau khi fold (mọi kiểu gõ):
/// - `qư` không tồn tại: `u` của `qu` là phụ âm → `quơ`, `quở` (VNI `quo7`, Telex `quow`).
/// - `ươ` không đứng cuối âm tiết (luôn có âm cuối: ươc, ươi, ương…), còn `uơ` thì có
///   (`thuở`, `huơ`, `khuơ`) → cặp `ươ` ở cuối từ là `uơ`. Từ còn gõ tiếp thì fold lại từ
///   đầu nên `thuow` → `thuơ` rồi `thuowng` → `thương`.
fn fix_uo(out: &mut [char]) {
    use crate::transform::tone::tone_of;
    use crate::transform::vowel_table::{form_like, locate};
    use crate::transform::vowel_table_generated::{O_HOOK, U, U_HOOK};
    let is = |c: char, e: usize| matches!(locate(c), Some((x, _)) if x == e);
    for i in 1..out.len() {
        if matches!(out[i - 1], 'q' | 'Q') && is(out[i], U_HOOK) {
            out[i] = form_like(out[i], U, tone_of(out[i]).unwrap_or(0));
        }
    }
    let len = out.len();
    if len >= 2 && is(out[len - 1], O_HOOK) && is(out[len - 2], U_HOOK) {
        let u = out[len - 2];
        out[len - 2] = form_like(u, U, tone_of(u).unwrap_or(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str, m: Method) -> String {
        fold(&s.chars().collect::<Vec<_>>(), m, DiacriticStyle::New, true)
            .into_iter()
            .collect()
    }

    /// 5 key dấu thanh của từng kiểu gõ (dùng cho test bảng sinh).
    fn tone_of_marker_keys(name: &str) -> &'static [char; 5] {
        use super::keys_generated as g;
        match name {
            "telex" => &g::telex::TONE_KEYS,
            "simple_telex" => &g::simple_telex::TONE_KEYS,
            "vni" => &g::vni::TONE_KEYS,
            "viqr" => &g::viqr::TONE_KEYS,
            other => panic!("kiểu gõ lạ: {other}"),
        }
    }

    #[test]
    fn every_method_folds_duong() {
        assert_eq!(f("dduocj", Method::Telex), "được");
        assert_eq!(f("d9uo7c5", Method::Vni), "được");
        assert_eq!(f("ddu+o+c.", Method::Viqr), "được");
        assert_eq!(f("dduocj", Method::SimpleTelex), "được");
    }

    #[test]
    fn word_char_classification() {
        assert!(is_word_char('a', Method::Telex));
        assert!(is_word_char('7', Method::Vni));
        assert!(!is_word_char('7', Method::Telex));
        assert!(is_word_char('.', Method::Viqr));
        assert!(!is_word_char('.', Method::Telex));
        assert!(!is_word_char(' ', Method::Viqr));
    }

    #[test]
    fn method_names_match_config() {
        assert_eq!(Method::Telex.as_str(), "telex");
        assert_eq!(Method::Vni.as_str(), "vni");
        assert_eq!(Method::Viqr.as_str(), "viqr");
        assert_eq!(Method::SimpleTelex.as_str(), "simple_telex");
    }

    /// Bảng sinh từ `data/tables/*.toml` phải **tự nhất quán** — `xtask check-tables`
    /// bắt được lệch với data, các test dưới bắt được lỗi trong chính data đó.
    #[test]
    fn generated_key_tables_are_self_consistent() {
        use super::keys_generated as g;
        use crate::transform::vowel_table::VOWEL_ENTRY_COUNT;

        // 1) 5 key dấu thanh khác nhau; `tone_of_key` trả đúng 1..=5 theo thứ tự.
        for (name, tone_of) in [
            ("telex", g::telex::tone_of_key as fn(char) -> Option<usize>),
            (
                "simple_telex",
                g::simple_telex::tone_of_key as fn(char) -> Option<usize>,
            ),
            ("vni", g::vni::tone_of_key as fn(char) -> Option<usize>),
            ("viqr", g::viqr::tone_of_key as fn(char) -> Option<usize>),
        ] {
            let mut seen: Vec<char> = Vec::new();
            for (i, &k) in tone_of_marker_keys(name).iter().enumerate() {
                assert!(!seen.contains(&k), "{name}: key dấu thanh lặp '{k}'");
                seen.push(k);
                assert_eq!(tone_of(k), Some(i + 1), "{name}: tone của '{k}'");
            }
            assert_eq!(seen.len(), 5, "{name}: phải có đúng 5 key dấu thanh");
        }

        // 2) mọi cặp (key, gốc, đích) trỏ tới âm thật, gốc khác đích.
        for table in [
            &g::telex::CIRCUMFLEX[..],
            &g::telex::HORN[..],
            &g::vni::CIRCUMFLEX[..],
            &g::vni::HORN[..],
            &g::vni::BREVE[..],
            &g::viqr::CIRCUMFLEX[..],
            &g::viqr::HORN[..],
            &g::viqr::BREVE[..],
        ] {
            for &(k, from, to) in table {
                assert!(
                    from < VOWEL_ENTRY_COUNT && to < VOWEL_ENTRY_COUNT,
                    "key '{k}' trỏ âm ngoài bảng: {from} → {to}"
                );
                assert_ne!(from, to, "cặp ('{k}') gốc == đích là vô nghĩa");
            }
        }
    }

    /// Simple Telex = Telex trừ `w`; mọi thứ khác phải giống hệt.
    #[test]
    fn simple_telex_chia_bang_telex_tru_w() {
        use super::keys_generated as g;
        assert_eq!(g::simple_telex::TONE_KEYS, g::telex::TONE_KEYS);
        assert_eq!(g::simple_telex::CIRCUMFLEX, g::telex::CIRCUMFLEX);
        assert_eq!(g::simple_telex::STROKE_KEY, g::telex::STROKE_KEY);
        assert!(g::telex::is_marker('w'));
        assert!(!g::simple_telex::is_marker('w'));
        // Không assert thẳng `W_MARKER` (clippy `assertions_on_constants`): so sánh
        // 2 hằng sinh từ data — nếu data đổi thành giống nhau thì test này bắt được.
        assert!(
            g::telex::W_MARKER != g::simple_telex::W_MARKER,
            "Telex và Simple Telex phải khác nhau ở `w`"
        );
    }
}
