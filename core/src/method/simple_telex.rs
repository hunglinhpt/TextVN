// SPDX-License-Identifier: GPL-3.0-or-later
//! Simple Telex (P0-1 §1 `method/simple_telex.rs`) — Telex mà `w` **chỉ là dấu sừng**
//! (UniKey `SimpleTelexMethodMapping` `{'W', vneHookAll}`, bamboo `w: UOA_ƯƠĂ`):
//! `aw ow uw` → `ă ơ ư` như Telex (R2-61 — bản cũ coi `w` là chữ nên không gõ được
//! `trắng`, `mơ`, `mưa`). Không có âm nhận sừng thì `w` là chữ thường: `ww` → `ww` (Telex
//! nuốt lặp `ww` → `w`).
//!
//! Giữ nguyên phần còn lại của Telex: `s f r x j` (dấu thanh), đôi `aa ee oo dd`,
//! cụm `uo`, rule `iet`. Tách riêng file để bật/tắt bằng `config.method`
//! mà **không** đổi hành vi Telex đã pass corpus.

use super::telex;
use super::DiacriticStyle;

/// Fold chuỗi phím của một từ → chuỗi hiển thị (`w` chỉ là dấu sừng).
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
    fn w_is_horn_marker_only() {
        // R2-61: như UniKey Simple Telex (`vneHookAll`).
        assert_eq!(n("tuw"), "tư");
        assert_eq!(n("mow"), "mơ");
        assert_eq!(n("trawngs"), "trắng");
        assert_eq!(n("muwa"), "mưa");
        assert_eq!(n("cuwar"), "cửa");
        assert_eq!(n("dduongw"), "đương");
        // Không có âm nhận sừng → chữ `w`, không nuốt lặp (Telex: `ww` → `w`).
        assert_eq!(n("ww"), "ww");
        assert_eq!(n("wow"), "wơ");
    }

    #[test]
    fn rest_of_telex_unchanged() {
        assert_eq!(n("dduocj"), "được");
        assert_eq!(n("caan"), "cân");
        assert_eq!(n("viet"), "viêt");
        assert_eq!(n("hoaf"), "hoà");
    }
}
