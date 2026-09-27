// SPDX-License-Identifier: GPL-3.0-or-later
//! Viết hoa tự động — `config.auto_capitalize` (P0-3 §1.1: "sau `. ! ?` + Enter").
//!
//! Engine gọi `is_sentence_end` cho **mọi** ký tự đi vào document; ký tự kết câu bật cờ
//! "chờ viết hoa", chữ cái kế tiếp bị `capitalize` **trước khi** fold (nên `ban` → `Ban`
//! và dấu thanh vẫn đặt đúng).

/// Ký tự kết câu → chữ cái đầu của từ kế tiếp viết hoa.
pub fn is_sentence_end(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '\n')
}

/// Viết hoa 1 ký tự (không phải chữ cái → giữ nguyên).
pub fn capitalize(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_end_chars() {
        for c in ['.', '!', '?', '\n'] {
            assert!(is_sentence_end(c), "`{c}` phải kết câu");
        }
        for c in [' ', ',', ';', ':', '\t', 'a'] {
            assert!(!is_sentence_end(c), "`{c}` không kết câu");
        }
    }

    #[test]
    fn capitalize_letters() {
        assert_eq!(capitalize('b'), 'B');
        assert_eq!(capitalize('B'), 'B');
        assert_eq!(capitalize('1'), '1');
        assert_eq!(capitalize(' '), ' ');
    }
}
