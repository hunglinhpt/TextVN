// SPDX-License-Identifier: GPL-3.0-or-later
//! Che noi dung khong nen kiem tra (URL / email) - ham y Harper `mask/`.
//!
//! V1 chi che 3 dang: URL co `://`, email co `@`, va tien to `www.`.
//! Noi dung che khong bao gio bao loi (tranh bao dong gia tren link/email).

/// Cac dai ky tu (char offset) bi che trong mot chuoi.
///
/// Ket qua sap xep theo `start` va khong trung nhau.
pub fn masked_ranges(text: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    // "://" - URL (http://, https://, ftp://...).
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == ':' && chars[i + 1] == '/' {
            let start = expand_non_space(&chars, i, true);
            let end = expand_non_space(&chars, i + 2, false);
            ranges.push((start, end));
        }
    }

    // "@" - email.
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '@' {
            let start = expand_word(&chars, i, true);
            let end = expand_word(&chars, i + 1, false);
            if start < end {
                ranges.push((start, end));
            }
        }
    }

    // "www." - URL khong co scheme.
    let lower: String = chars.iter().collect::<String>().to_lowercase();
    let lower_chars: Vec<char> = lower.chars().collect();
    let mut from = 0usize;
    while let Some(pos) = lower_chars[from..]
        .windows(4)
        .position(|w| w == ['w', 'w', 'w', '.'])
    {
        let start = from + pos;
        let end = expand_non_space(&chars, start, false);
        ranges.push((start, end));
        from = start + 4;
    }

    ranges.sort_unstable();
    ranges.dedup();
    ranges
}

/// Mo rong range theo ky tu khong phai khoang trang.
fn expand_non_space(chars: &[char], from: usize, backwards: bool) -> usize {
    if backwards {
        let mut i = from;
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    } else {
        let mut i = from;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        i
    }
}

/// Mo rong range theo ky tu "chu" (chu so, chu cai, `.`, `-`, `_`, `+`).
fn expand_word(chars: &[char], from: usize, backwards: bool) -> usize {
    if backwards {
        let mut i = from;
        while i > 0 && is_word_char(chars[i - 1]) {
            i -= 1;
        }
        i
    } else {
        let mut i = from;
        while i < chars.len() && is_word_char(chars[i]) {
            i += 1;
        }
        i
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '.' | '-' | '_' | '+')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn che_url() {
        let r = masked_ranges("xem https://example.com/a.b nhe");
        assert_eq!(r.len(), 1);
        let (s, e) = r[0];
        assert_eq!(
            &"xem https://example.com/a.b nhe"[s..e],
            "https://example.com/a.b"
        );
    }

    #[test]
    fn che_email() {
        let r = masked_ranges("ghi cho abc@x.vn nha");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], (8, 16));
    }

    #[test]
    fn cau_thuong_khong_che() {
        assert!(masked_ranges("Tôi đi học ở trường.").is_empty());
    }
}
