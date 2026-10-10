// SPDX-License-Identifier: GPL-3.0-or-later
//! Viết hoa tự động — `config.auto_capitalize` (P0-3 §1.1: "sau `. ! ?` + Enter").
//!
//! Chữ đầu câu = chữ cái đầu từ gõ khi đuôi text ngay trước con trỏ là `. ! ?` **rồi
//! khoảng trắng** (Space/Tab), hoặc là Enter. Engine hỏi `sentence_start` trên đuôi text nó
//! biết (`Engine::recent`) ngay lúc chữ đầu từ được gõ — không giữ cờ riêng, nên Backspace,
//! phím điều hướng hay chord tự đúng theo text còn lại.
//!
//! R2-59: dấu chấm **không** kèm khoảng trắng không kết câu — `google.com`, `file.txt`,
//! `a@b.com`, `3.5 kg`, `v.v.` giữ nguyên chữ thường (bản cũ bật cờ ngay khi gõ `.`).
//! Chữ đầu từ chỉ bị `capitalize` **trước khi** fold (nên `ban` → `Ban` và dấu thanh vẫn
//! đặt đúng).

/// Ký tự kết câu (trước khoảng trắng).
pub fn is_sentence_end(c: char) -> bool {
    matches!(c, '.' | '!' | '?')
}

/// Dấu đóng được phép đứng giữa dấu kết câu và khoảng trắng: `chào.") Bạn`.
fn is_closer(c: char) -> bool {
    matches!(
        c,
        ')' | ']' | '}' | '"' | '\'' | '\u{201D}' | '\u{2019}' | '\u{BB}'
    )
}

/// Đuôi text `tail` (ngay trước con trỏ) có đặt con trỏ ở **đầu câu** không: kết thúc
/// bằng xuống dòng (có thể kèm khoảng trắng thụt đầu dòng), hoặc `. ! ?` (có thể kèm dấu
/// đóng) rồi ít nhất một Space/Tab. Đuôi rỗng (đầu văn bản / sau khi bỏ dấu vết con trỏ)
/// → không.
pub fn sentence_start(tail: &[char]) -> bool {
    let mut i = tail.len();
    let mut saw_space = false;
    while i > 0 {
        match tail[i - 1] {
            '\n' | '\r' => return true,
            ' ' | '\t' | '\u{A0}' => {
                saw_space = true;
                i -= 1;
            }
            _ => break,
        }
    }
    if !saw_space {
        return false;
    }
    while i > 0 && is_closer(tail[i - 1]) {
        i -= 1;
    }
    i > 0 && is_sentence_end(tail[i - 1])
}

/// Viết hoa 1 ký tự (không phải chữ cái → giữ nguyên).
pub fn capitalize(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts(s: &str) -> bool {
        sentence_start(&s.chars().collect::<Vec<_>>())
    }

    #[test]
    fn sentence_end_chars() {
        for c in ['.', '!', '?'] {
            assert!(is_sentence_end(c), "`{c}` phải kết câu");
        }
        for c in [' ', ',', ';', ':', '\t', 'a', '\n'] {
            assert!(!is_sentence_end(c), "`{c}` không kết câu");
        }
    }

    #[test]
    fn sentence_start_needs_space_or_newline() {
        for s in [
            "chào. ",
            "Hả? ",
            "Hay!\t",
            "chào.  ",
            "chào.\") ",
            "chào\n",
            "chào\n  ",
            "a.\r",
        ] {
            assert!(starts(s), "{s:?} phải là đầu câu");
        }
        // R2-59: dấu chấm không kèm khoảng trắng (URL, tên file, số thập phân, `v.v.`).
        for s in [
            "", "chào.", "google.", "file.", "3.", "a@b.", "v.", "v.v", "chào ", "chào, ", "3.5 ",
            " ",
        ] {
            assert!(!starts(s), "{s:?} KHÔNG phải đầu câu");
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
