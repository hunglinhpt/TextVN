// SPDX-License-Identifier: GPL-3.0-or-later
//! Simple Telex (P0-1 §1 `method/simple_telex.rs`) — Telex **không có marker `w`**:
//! `w` là chữ thường (`tuw` → `tuw`, không phải `tư`), nên `ww` → `ww` (không nuốt lặp).
//!
//! Giữ nguyên phần còn lại của Telex: `s f r x j` (dấu thanh), đôi `aa ee oo dd`,
//! `d`+âm → `đ`, cụm `uo`, rule `iet`. Tách riêng file để bật/tắt bằng `config.method`
//! mà **không** đổi hành vi Telex đã pass corpus.

use super::telex;
use super::DiacriticStyle;

/// Fold chuỗi phím của một từ → chuỗi hiển thị (không dùng `w` làm marker).
pub fn fold(raw: &[char], style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    telex::fold_with(raw, style, free_marking, false)
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
    fn w_is_literal() {
        assert_eq!(n("tuw"), "tuw"); // Telex: "tư"
        assert_eq!(n("mow"), "mow"); // Telex: "mơ"
        assert_eq!(n("trawng"), "trawng"); // Telex: "trăng"
        assert_eq!(n("ww"), "ww"); // Telex: "w" (nuốt lặp)
    }

    #[test]
    fn rest_of_telex_unchanged() {
        assert_eq!(n("dduocj"), "được");
        assert_eq!(n("caan"), "cân");
        assert_eq!(n("viet"), "viêt");
        assert_eq!(n("hoaf"), "hoà");
    }
}
