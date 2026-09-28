// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng mã xuất (`config.output_charset`) — kế thừa UniKey/OpenKey.
//!
//! Engine xử lý nội bộ bằng Unicode dựng sẵn; chỉ **chữ đi ra document** được chuyển mã.
//! Vì một chữ Việt có thể thành 2 ký tự (VNI Windows, Unicode tổ hợp), mọi
//! `delete_count` của engine được tính trên chuỗi ĐÃ chuyển mã (xem `Engine::emit`),
//! nên adapter không cần biết bảng mã.
//!
//! Dữ liệu: `data/tables/charsets.tsv` (lấy nguyên từ UniKey `unikey/data.cpp`).
//! TCVN3/VNI Windows là bảng mã 8-bit cho font cũ (.VnTime, VNI-Times): mọi byte chữ Việt
//! ≥ 0xA0 nên được xuất thành ký tự Latin-1 cùng mã — đúng cách UniKey gõ vào app Unicode.

use std::sync::OnceLock;

/// `config.output_charset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputCharset {
    #[default]
    UnicodePrecomposed,
    UnicodeDecomposed,
    Tcvn3,
    VniWindows,
}

struct Entry {
    unicode: char,
    tcvn3: char,
    vni: [char; 2],
    decomposed: [char; 2],
}

const TABLE_TSV: &str = include_str!("../../../data/tables/charsets.tsv");

fn parse_pair(field: &str) -> Option<[char; 2]> {
    let mut it = field.split('+');
    let base = char::from_u32(u32::from_str_radix(it.next()?, 16).ok()?)?;
    let mark = match it.next() {
        Some(m) => char::from_u32(u32::from_str_radix(m, 16).ok()?)?,
        None => '\0',
    };
    Some([base, mark])
}

fn table() -> &'static [Entry] {
    static TABLE: OnceLock<Vec<Entry>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut entries: Vec<Entry> = TABLE_TSV
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let mut f = l.split('\t');
                let unicode = char::from_u32(u32::from_str_radix(f.next()?, 16).ok()?)?;
                let tcvn3 = char::from_u32(u32::from_str_radix(f.next()?, 16).ok()?)?;
                let vni = parse_pair(f.next()?)?;
                let decomposed = parse_pair(f.next()?)?;
                Some(Entry {
                    unicode,
                    tcvn3,
                    vni,
                    decomposed,
                })
            })
            .collect();
        entries.sort_by_key(|e| e.unicode);
        entries
    })
}

fn push_pair(out: &mut Vec<char>, pair: [char; 2]) {
    out.push(pair[0]);
    if pair[1] != '\0' {
        out.push(pair[1]);
    }
}

/// Chuyển chuỗi Unicode dựng sẵn sang bảng mã xuất. Ký tự không phải chữ Việt giữ nguyên.
pub fn encode(text: &[char], charset: OutputCharset) -> Vec<char> {
    if charset == OutputCharset::UnicodePrecomposed {
        return text.to_vec();
    }
    let table = table();
    let mut out = Vec::with_capacity(text.len() * 2);
    for &c in text {
        match table.binary_search_by_key(&c, |e| e.unicode) {
            Ok(i) => {
                let e = &table[i];
                match charset {
                    OutputCharset::UnicodePrecomposed => out.push(c),
                    OutputCharset::UnicodeDecomposed => push_pair(&mut out, e.decomposed),
                    OutputCharset::Tcvn3 => out.push(e.tcvn3),
                    OutputCharset::VniWindows => push_pair(&mut out, e.vni),
                }
            }
            Err(_) => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(s: &str, cs: OutputCharset) -> String {
        encode(&s.chars().collect::<Vec<_>>(), cs)
            .into_iter()
            .collect()
    }

    #[test]
    fn table_covers_all_134_toned_letters_plus_bases() {
        // 186 = 26×2 chữ Latin + 67×2 chữ Việt (a/ă/â/e/ê/i/o/ô/ơ/u/ư/y × 6 thanh − dạng
        // không dấu đã tính, + đ/Đ). Thiếu một dòng = một chữ gõ ra sai bảng mã.
        assert_eq!(table().len(), 186);
        for w in table().windows(2) {
            assert!(w[0].unicode < w[1].unicode, "trùng mã {:?}", w[0].unicode);
        }
    }

    #[test]
    fn unicode_decomposed_matches_unikey_composite() {
        assert_eq!(enc("á", OutputCharset::UnicodeDecomposed), "a\u{301}");
        assert_eq!(enc("Ấ", OutputCharset::UnicodeDecomposed), "Â\u{301}");
        assert_eq!(enc("được", OutputCharset::UnicodeDecomposed), "đươ\u{323}c");
        assert_eq!(enc("đ", OutputCharset::UnicodeDecomposed), "đ");
    }

    #[test]
    fn legacy_charsets_follow_unikey_bytes() {
        assert_eq!(enc("á", OutputCharset::Tcvn3), "\u{b8}");
        assert_eq!(enc("đ", OutputCharset::Tcvn3), "\u{ae}");
        assert_eq!(enc("á", OutputCharset::VniWindows), "a\u{f9}");
        assert_eq!(enc("đ", OutputCharset::VniWindows), "\u{f1}");
        assert_eq!(enc("ừ", OutputCharset::VniWindows), "\u{f6}\u{f8}");
    }

    #[test]
    fn non_vietnamese_text_is_untouched() {
        for cs in [
            OutputCharset::UnicodeDecomposed,
            OutputCharset::Tcvn3,
            OutputCharset::VniWindows,
        ] {
            assert_eq!(enc("hello 123 😀", cs), "hello 123 😀");
        }
        assert_eq!(enc("được", OutputCharset::UnicodePrecomposed), "được");
    }
}
