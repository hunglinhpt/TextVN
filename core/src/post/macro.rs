// SPDX-License-Identifier: GPL-3.0-or-later
//! Gõ tắt (`config.macros[]` — P0-3 §1.1) — trigger key `macro_trigger` (`tab`|`space`)
//! thay **text đã gõ** trong document bằng `expand`.
//!
//! - Trigger khớp **không phân biệt hoa/thường** với **đuôi** text engine đã đẩy vào document
//!   (`Engine::recent`) → gõ `CTY` cũng khớp `cty`.
//! - Nhiều trigger cùng khớp → **trigger dài nhất thắng** (không phụ thuộc thứ tự file).
//! - `when`: `always` = chạy cả khi VN tắt · `vi_on` = chỉ khi VN bật (B12/EVKey spec #5).
//! - Trigger key bị **nuốt** (không chèn Tab/Space vào document).
//!
//! Emoji dùng chung cơ chế này — xem `post/emoji.rs`.

/// `config.macros[].when` (P0-3 §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacroWhen {
    #[default]
    Always,
    ViOn,
}

/// Một macro gõ tắt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDef {
    pub trigger: String,
    pub expand: String,
    pub when: MacroWhen,
}

/// Trigger key của engine (`config.macro_trigger`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacroTrigger {
    #[default]
    Tab,
    Space,
}

fn eq_ignore_case(a: char, b: char) -> bool {
    a.to_lowercase().eq(b.to_lowercase())
}

/// Trigger macro khớp đuôi `tail`? (không phân biệt hoa/thường, so theo `char`).
///
/// Macro chữ phải bắt đầu ở word boundary để `xcty<Tab>` không âm thầm thay phần
/// đuôi của một identifier. Emoji vẫn dùng suffix match: ký tự `:` trong trigger
/// đã là delimiter quen thuộc khi người dùng gõ `xin chào:smile`.
fn macro_tail_matches(trigger: &str, tail: &[char]) -> Option<usize> {
    let need: Vec<char> = trigger.chars().collect();
    if need.is_empty() || need.len() > tail.len() {
        return None;
    }
    let start = tail.len() - need.len();
    if start > 0 && (tail[start - 1].is_alphanumeric() || tail[start - 1] == '_') {
        return None;
    }
    let hit = need
        .iter()
        .zip(&tail[start..])
        .all(|(&a, &b)| eq_ignore_case(a, b));
    if hit {
        Some(need.len())
    } else {
        None
    }
}

/// Emoji dùng suffix match; `:` là delimiter nằm trong trigger.
fn emoji_tail_matches(trigger: &str, tail: &[char]) -> Option<usize> {
    let need: Vec<char> = trigger.chars().collect();
    if need.is_empty() || need.len() > tail.len() {
        return None;
    }
    let start = tail.len() - need.len();
    need.iter()
        .zip(&tail[start..])
        .all(|(&a, &b)| eq_ignore_case(a, b))
        .then_some(need.len())
}

/// Tìm macro/emoji khớp đuôi `tail`; trả `(số ký tự trigger, text thay thế)`.
///
/// `vi_on = false` → chỉ macro `when == Always` (emoji luôn chạy — P0-3 §1.1 không có
/// field `when` cho emoji).
pub fn find<'a>(
    macros: &'a [MacroDef],
    emoji: &'a [crate::post::emoji::Emoji],
    tail: &[char],
    vi_on: bool,
) -> Option<(usize, &'a str)> {
    let mut best: Option<(usize, &'a str)> = None;
    let mut consider = |len: usize, text: &'a str| {
        if best.is_none_or(|(blen, _)| len > blen) {
            best = Some((len, text));
        }
    };
    for m in macros {
        if m.when == MacroWhen::ViOn && !vi_on {
            continue;
        }
        if let Some(len) = macro_tail_matches(&m.trigger, tail) {
            consider(len, &m.expand);
        }
    }
    for e in emoji {
        if let Some(len) = emoji_tail_matches(&e.trigger, tail) {
            consider(len, &e.glyph);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::post::emoji::Emoji;

    fn def(trigger: &str, expand: &str, when: MacroWhen) -> MacroDef {
        MacroDef {
            trigger: trigger.into(),
            expand: expand.into(),
            when,
        }
    }

    fn tail(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn matches_suffix_case_insensitive() {
        let macros = vec![def("cty", "Công ty TNHH", MacroWhen::Always)];
        assert_eq!(
            find(&macros, &[], &tail("abc cty"), true),
            Some((3, "Công ty TNHH"))
        );
        assert_eq!(
            find(&macros, &[], &tail("CTY"), true),
            Some((3, "Công ty TNHH"))
        );
        assert_eq!(find(&macros, &[], &tail("ct"), true), None);
        assert_eq!(find(&macros, &[], &tail("xcty"), true), None);
    }

    #[test]
    fn longest_trigger_wins() {
        let macros = vec![
            def("ty", "ty", MacroWhen::Always),
            def("cty", "Công ty TNHH", MacroWhen::Always),
        ];
        assert_eq!(
            find(&macros, &[], &tail("cty"), true),
            Some((3, "Công ty TNHH"))
        );
    }

    #[test]
    fn vi_on_only_when_vietnamese_on() {
        let macros = vec![def("vn", "Việt Nam", MacroWhen::ViOn)];
        assert_eq!(find(&macros, &[], &tail("vn"), true), Some((2, "Việt Nam")));
        assert_eq!(find(&macros, &[], &tail("vn"), false), None);
    }

    #[test]
    fn emoji_always_runs() {
        let emoji = vec![Emoji {
            trigger: ":smile".into(),
            glyph: "😊".into(),
        }];
        assert_eq!(find(&[], &emoji, &tail("a:smile"), false), Some((6, "😊")));
    }
}
