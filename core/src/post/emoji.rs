// SPDX-License-Identifier: GPL-3.0-or-later
//! Emoji gõ tắt (`config.emoji[]` — P0-3 §1.1: `emoji[] = [{trigger, glyph}]`).
//!
//! Cùng cơ chế với `post/macro.rs` (khớp đuôi text đã vào document theo trigger key), chỉ
//! khác nguồn dữ liệu và **luôn chạy** (bảng P0-3 §1.1 không có field `when` cho emoji).
//! Emoji là text nhiều `char`/surrogate pair → engine chỉ trả `Vec<char>`, adapter mã hoá.

/// Một emoji gõ tắt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emoji {
    pub trigger: String,
    pub glyph: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::post::r#macro::{find, MacroWhen};

    #[test]
    fn emoji_glyph_is_multi_char() {
        let emoji = vec![Emoji {
            trigger: ":smile".into(),
            glyph: "😊".into(),
        }];
        let tail: Vec<char> = "hi:smile".chars().collect();
        let (len, text) = find(&[], &emoji, &tail, true).expect("phải khớp");
        assert_eq!(len, 6);
        assert_eq!(text.chars().count(), 1); // 😊 = 1 char (surrogate pair ở UTF-16)
    }

    #[test]
    fn macro_and_emoji_coexist() {
        let macros = vec![crate::post::r#macro::MacroDef {
            trigger: "cty".into(),
            expand: "Công ty".into(),
            when: MacroWhen::Always,
        }];
        let emoji = vec![Emoji {
            trigger: ":ok".into(),
            glyph: "👌".into(),
        }];
        let tail: Vec<char> = "cty".chars().collect();
        assert_eq!(find(&macros, &emoji, &tail, true), Some((3, "Công ty")));
        let tail: Vec<char> = ":ok".chars().collect();
        assert_eq!(find(&macros, &emoji, &tail, true), Some((3, "👌")));
    }
}
