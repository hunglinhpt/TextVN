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
    masked_ranges_chars(&chars)
}

/// Tinh cac dai ky tu bi che tren slice `&[char]` da tach san (0-alloc trung gian).
pub fn masked_ranges_chars(chars: &[char]) -> Vec<(usize, usize)> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    // "://" - URL (http://, https://, ftp://...).
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == ':' && chars[i + 1] == '/' {
            let start = expand_non_space(chars, i, true);
            let end = expand_non_space(chars, i + 2, false);
            ranges.push((start, end));
        }
    }

    // "@" - email.
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '@' {
            let start = expand_word(chars, i, true);
            let end = expand_word(chars, i + 1, false);
            if start < end {
                ranges.push((start, end));
            }
        }
    }

    // "www." - URL khong co scheme (so khop ASCII khong cap phat String).
    let mut from = 0usize;
    while let Some(pos) = chars[from..].windows(4).position(|w| {
        w[0].eq_ignore_ascii_case(&'w')
            && w[1].eq_ignore_ascii_case(&'w')
            && w[2].eq_ignore_ascii_case(&'w')
            && w[3] == '.'
    }) {
        let start = from + pos;
        let end = expand_non_space(chars, start, false);
        ranges.push((start, end));
        from = start + 4;
    }

    // Vòng 24 (audit EN trap 34): tên miền/phần mở rộng trần — "example.com",
    // "main.rs", "Node.js" — mask.rs cũ chỉ che `://`/`@`/`www.` nên
    // space_after_punct chèn space giữa "example" và "com". Che mọi
    // `từ.tld` với TLD/phần mở rộng trong danh sách đóng.
    let tlds = [
        "com", "net", "org", "io", "vn", "edu", "gov", "co", "uk", "dev", "app", "info", "biz",
        "me", "py", "rs", "ts", "js", "md", "sh", "to", "ai", "exe", "dll",
        // Vòng 32 (audit A2): phần mở rộng file văn phòng / dev phổ biến —
        // thiếu thì space_after_punct chèn space giữa TÊN FILE ("report.pdf"
        // → "report. pdf") và Sửa-tất-cả phá luôn tên file.
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "csv", "json", "yaml", "yml",
        "xml", "html", "css", "sql", "zip", "rar", "png", "jpg", "jpeg", "gif", "webp", "tiff",
        "wav", "ini", "log", "bak", "tmp", "cfg", "conf", "env", "bat", "ps1", "rb", "php", "java",
        "cpp", "cs", "kt", "go",
        // Vòng 34 (audit R1-F1a): tiếp tục bổ sung — thiếu thì space_after_punct
        // chèn space giữa TÊN FILE ("config.toml" → "config. toml") với
        // confidence 100 + auto_fixable ⇒ Sửa-tất-cả phá luôn tên file.
        "swift", "ipynb", "proto", "gradle", "so", "lock", "o", "h", "c", "pas", "vue", "zig",
        "f90", "pl", "lua", "rmd", "sbt", "sln", "csproj", "tcl", "asm", "r", "jl", "d", "nim",
        "ex", "exs", "erl", "hs", "ml", "clj", "groovy", "dart", "cmake", "toml", "rst", "tex",
        "bib", "coffee", "pug", "haml", "slim", "svelte", "astro", "prisma", "graphqlx",
    ];
    for i in 0..chars.len() {
        if chars[i] != '.' || i == 0 || i + 1 >= chars.len() {
            continue;
        }
        // Vòng 34 (R1-F1b): chữ liền trước dấu chấm chấp nhận CẢ Unicode
        // (tên file tiếng Việt "đề.doc" — 'ề' không phải ASCII-alnum nhưng
        // là alnum); ext trong danh sách đóng đã chặn phía sau.
        let before_ok = chars[i - 1].is_alphanumeric();
        if !before_ok {
            continue;
        }
        let after_slice: &[char] = &chars[i + 1..];
        let after_len = after_slice
            .iter()
            .take_while(|c| c.is_alphanumeric())
            .count();
        // Vòng 34 (audit R1-F1a): cap 4 → 6 — "config.toml" (4 ✓ nhưng
        // "main.swift"/"index.ipynb" = 5, "schema.gradle" = 6 từng bị sót).
        if after_len == 0 || after_len > 6 {
            continue;
        }
        let after_chars = &after_slice[..after_len];
        if !after_chars.iter().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        // Không che ".Ai", ".To", ".Me", ".Co" (Title Case sau dấu chấm = thiếu space
        // trước đầu câu mới như "đi học.Ai", "went home.To", không phải tên miền).
        let is_title_case = after_chars[0].is_ascii_uppercase()
            && after_chars[1..].iter().any(|c| c.is_ascii_lowercase());
        if is_title_case {
            continue;
        }
        let start = expand_word(chars, i, true);
        // Vòng 34 (audit R1-F1b): bỏ ràng buộc base thuần ASCII — tên file
        // tiếng Việt có dấu ("Báo cáo.xlsx", "in file đề.doc") từng bị sót
        // mask ⇒ space_after_punct phá tên file. Base qua `expand_word` đã
        // chỉ gồm word-char (alnum Unicode/.-_) liền trước dấu chấm.
        if start >= i {
            continue;
        }
        let after_lower: String = after_chars.iter().map(|c| c.to_ascii_lowercase()).collect();
        if tlds.contains(&after_lower.as_str()) {
            let end = i + 1 + after_len;
            // Bỏ qua nếu đã nằm trong vùng che sẵn (URL `://`, email `@`).
            let already = ranges.iter().any(|&(rs, re)| start >= rs && end <= re);
            if !already {
                ranges.push((start, end));
            }
        }
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
