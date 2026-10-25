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

// ===== OP-007 — Bộ chuyển mã văn bản bôi đen (P5, §7.2) ======================
// Phần bổ sung dưới đây (`Charset`, `TextCase`, `decode_tcvn3`, `decode_vni_windows`,
// `strip_diacritics`, `transcode`) chỉ tái dùng `table()` + `encode(...)` hiện có;
// API vốn có (`OutputCharset`, `table`, `encode`) không đổi. Toàn bộ là hàm thuần
// logic: 0 I/O, không unwrap/expect/panic, deterministic — ký tự lạ luôn đi qua
// nguyên, bộ nhớ chỉ tỷ lệ với độ dài đầu vào (tối đa 2 lần cho VNI/Unicode tổ hợp).

/// Các bảng mã mà bộ chuyển mã văn bản bôi đen (`transcode`) hỗ trợ.
///
/// Bốn giá trị đầu trùng tên `OutputCharset`: hướng xuất vẫn đi qua `encode`
/// gốc. `Viqr`/`TelexRaw` chỉ dùng cho luồng bôi đen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    #[default]
    UnicodePrecomposed,
    UnicodeDecomposed,
    Tcvn3,
    VniWindows,
    /// VIQR thô (`to^i la` ai`). **Giới hạn v1:** chưa dựng bảng VIQR →
    /// Unicode nên `transcode` giữ nguyên chuỗi (pass-through); dựng đủ bảng
    /// VIQR hai chiều là việc của v2.
    Viqr,
    /// Chuỗi phím Telex thô (`Tooi ddanwg hojc`). **Giới hạn v1:** vẫn
    /// pass-through như VIQR, vì fold chuẩn chuỗi phím Telex thô về chữ Việt
    /// cần ngữ cảnh buffer gõ phím của engine; decode đủ bảng đặt ở v2.
    TelexRaw,
}

/// Chế độ biến đổi kiểu chữ khi chuyển mã (P5 §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextCase {
    #[default]
    /// Giữ nguyên kiểu chữ như văn bản gốc.
    AsIs,
    /// CHỮ HOA TOÀN BỘ.
    Upper,
    /// chữ thường toàn bộ.
    Lower,
    /// Viết Hoa Chữ Đầu Mỗi Từ (ranh giới từ theo dấu cách/dấu câu cơ bản).
    Title,
    /// KHÔNG DẤU: gỡ dấu tiếng Việt, `đ/Đ` → `d/D`.
    NoAccent,
}

/// Cặp đối 1-1 chữ có dấu → cơ sở: 67 chữ thường + 67 chữ hoa, cùng thứ tự
/// (a ×17, e ×11, i ×5, o ×17, u ×11, y ×5, `đ`). Độ dài tự kiểm bằng test
/// `op007_strip_table_matches_charset_table`; tra cứu dùng `zip` nên kể cả
/// khi dữ liệu lệch cũng chỉ trả lại ký tự gốc, không bao giờ chỉ số ngoài
/// mảng (total function, không panic runtime).
const VIET_CHARS: &str = "àáảãạâầấẩẫậăằắẳẵặèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđÀÁẢÃẠÂẦẤẨẪẬĂẰẮẲẴẶÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴĐ";
const BASE_CHARS: &str = "aaaaaaaaaaaaaaaaaeeeeeeeeeeeiiiiiooooooooooooooooouuuuuuuuuuuyyyyydAAAAAAAAAAAAAAAAAEEEEEEEEEEEIIIIIOOOOOOOOOOOOOOOOOUUUUUUUUUUUYYYYYD";

/// Giải mã chuỗi TCVN3 (font .VnTime) về Unicode dựng sẵn.
///
/// TCVN3 là bảng mã 8-bit: chữ có dấu hoa/thường dùng CÙNG byte (`Á` và `á`
/// đều là 0xB8 — xem `data/tables/charsets.tsv`), nên hoa/thường của từng
/// chữ có dấu không khôi phục nổi từ bảng mã; chỉ chữ Latin không dấu mang
/// byte riêng cho 2 kiểu chữ (0x41/0x61). Bảng đã được xếp theo mã unicode
/// nên dòng hoa đứng trước dòng thường: `rfind` chọn dòng THƯỜNG, ưu tiên
/// văn bản văn phòng viết thường ("Cộng hòa" thay vì "CỘng hOà"). Văn bản
/// TCVN3 viết HOA gặp ngược hướng; có thể lấy lại bằng `TextCase::Upper`
/// qua `transcode`.
pub fn decode_tcvn3(input: &[char]) -> Vec<char> {
    let tbl = table();
    let mut out = Vec::with_capacity(input.len());
    for &c in input {
        if let Some(e) = tbl.iter().rfind(|e| e.tcvn3 == c) {
            out.push(e.unicode);
        } else {
            out.push(c);
        }
    }
    out
}

/// Giải mã chuỗi VNI-Windows (font VNI-Times) về Unicode dựng sẵn.
///
/// Quét greedy longest-match (R1-5): tại mỗi vị trí thử cặp 2 ký tự trước
/// (base + mark Latin-1), chỉ khi cặp không trùng bảng mới thử ký tự đơn
/// (mark trống `'\0'`: đ = 0xF1, Đ = 0xD1, ơ = 0xF4, ư = 0xF6). Greedy bắt
/// buộc: cặp [ö,ø] phải ghép trọn thành "ừ" — nếu khớp ký tự đơn trước sẽ
/// rách thành ư + ø. Ký tự lạ (ASCII, chữ số, emoji) giữ nguyên.
pub fn decode_vni_windows(input: &[char]) -> Vec<char> {
    let tbl = table();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        // 1. Cặp 2 ký tự trước (vd. ộ = [ô Latin-1, dấu nặng]).
        if i + 1 < input.len() {
            let pair = [input[i], input[i + 1]];
            if let Some(e) = tbl.iter().find(|e| e.vni == pair) {
                out.push(e.unicode);
                i += 2;
                continue;
            }
        }
        // 2. Ký tự đơn của bảng.
        let single = [input[i], '\0'];
        if let Some(e) = tbl.iter().find(|e| e.vni == single) {
            out.push(e.unicode);
        } else {
            out.push(input[i]);
        }
        i += 1;
    }
    out
}

/// Gỡ toàn bộ dấu tiếng Việt (thanh điệu, mũ, sừng, đ/Đ). Ký tự ngoài 134
/// chữ trên giữ nguyên; tra cứu qua `zip` 2 chuỗi đối trên là total function.
pub fn strip_diacritics(input: &[char]) -> Vec<char> {
    input
        .iter()
        .map(|&c| {
            VIET_CHARS
                .chars()
                .zip(BASE_CHARS.chars())
                .find(|(v, _)| *v == c)
                .map(|(_, b)| b)
                .unwrap_or(c)
        })
        .collect()
}

/// Chuyển văn bản từ bảng mã nguồn `from` sang bảng mã đích `to` kèm biến
/// đổi kiểu chữ `case`, pivot qua Unicode dựng sẵn: decode `from` → biến đổi
/// chữ → `encode` định mức bảng đích. Total function và deterministic (ký tự
/// lạ đi qua nguyên). VIQR/TelexRaw (cả 2 chiều) v1 pass-through — xem doc
/// của `Charset`.
pub fn transcode(input: &[char], from: Charset, to: Charset, case: TextCase) -> Vec<char> {
    // Bước 1: về Unicode dựng sẵn (pivot).
    let unicode: Vec<char> = match from {
        Charset::UnicodePrecomposed => input.to_vec(),
        Charset::UnicodeDecomposed => {
            let tbl = table();
            let mut res = Vec::with_capacity(input.len());
            let mut i = 0;
            while i < input.len() {
                if i + 1 < input.len() {
                    let pair = [input[i], input[i + 1]];
                    if let Some(e) = tbl.iter().find(|e| e.decomposed == pair) {
                        res.push(e.unicode);
                        i += 2;
                        continue;
                    }
                }
                res.push(input[i]);
                i += 1;
            }
            res
        }
        Charset::Tcvn3 => decode_tcvn3(input),
        Charset::VniWindows => decode_vni_windows(input),
        // Viqr/TelexRaw: v1 chưa dựng bảng decode (xem doc enum) — giữ nguyên.
        Charset::Viqr | Charset::TelexRaw => input.to_vec(),
    };

    // Bước 2: biến đổi kiểu chữ.
    let cased: Vec<char> = match case {
        TextCase::AsIs => unicode,
        TextCase::NoAccent => strip_diacritics(&unicode),
        TextCase::Upper => unicode.into_iter().flat_map(|c| c.to_uppercase()).collect(),
        TextCase::Lower => unicode.into_iter().flat_map(|c| c.to_lowercase()).collect(),
        TextCase::Title => {
            let mut out = Vec::with_capacity(unicode.len());
            let mut new_word = true;
            for c in unicode {
                if c.is_alphabetic() {
                    if new_word {
                        out.extend(c.to_uppercase());
                        new_word = false;
                    } else {
                        out.extend(c.to_lowercase());
                    }
                } else {
                    // Ranh giới từ = ký tự KHÔNG phải chữ/số, trừ nháy đơn
                    // (tên "l'hôpital" không viết hoa chữ sau nháy). Nhờ vậy
                    // "/", ":", "«", "—"… đều mở từ mới (audit vòng implement:
                    // bản đầu chỉ liệt kê 8 dấu nên "A/B" ra "A/b").
                    new_word = !c.is_alphanumeric() && c != '\'';
                    out.push(c);
                }
            }
            out
        }
    };

    // Bước 3: xuất sang bảng mã đích bằng `encode` gốc (giữ nguyên API vốn có).
    match to {
        Charset::UnicodePrecomposed => cased,
        Charset::UnicodeDecomposed => encode(&cased, OutputCharset::UnicodeDecomposed),
        Charset::Tcvn3 => encode(&cased, OutputCharset::Tcvn3),
        Charset::VniWindows => encode(&cased, OutputCharset::VniWindows),
        // Viqr/TelexRaw đích: v1 chưa dựng bảng encode (xem doc enum) — trả
        // về Unicode dựng sẵn của phần nền đã biến đổi.
        Charset::Viqr | Charset::TelexRaw => cased,
    }
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

    // ===== OP-007 — Bộ chuyển mã văn bản bôi đen (P5, §7.2 / §4.2) =====

    #[test]
    fn op007_transcode_legacy_charsets() {
        let text = "Cộng hòa Xã hội Chủ nghĩa Việt Nam";
        let chars: Vec<char> = text.chars().collect();

        // 1. TCVN3: decode trực tiếp (§4.2) + transcode 2 chiều qua `encode` gốc.
        let tcvn3_bytes = encode(&chars, OutputCharset::Tcvn3);
        assert_eq!(decode_tcvn3(&tcvn3_bytes), chars);
        assert_eq!(
            transcode(
                &tcvn3_bytes,
                Charset::Tcvn3,
                Charset::UnicodePrecomposed,
                TextCase::AsIs
            ),
            chars
        );
        assert_eq!(
            transcode(
                &chars,
                Charset::UnicodePrecomposed,
                Charset::Tcvn3,
                TextCase::AsIs
            ),
            tcvn3_bytes
        );

        // 2. VNI-Windows.
        let vni_bytes = encode(&chars, OutputCharset::VniWindows);
        assert_eq!(decode_vni_windows(&vni_bytes), chars);
        assert_eq!(
            transcode(
                &vni_bytes,
                Charset::VniWindows,
                Charset::UnicodePrecomposed,
                TextCase::AsIs
            ),
            chars
        );
        assert_eq!(
            transcode(
                &chars,
                Charset::UnicodePrecomposed,
                Charset::VniWindows,
                TextCase::AsIs
            ),
            vni_bytes
        );

        // 3. Unicode tổ hợp ↔ dựng sẵn (pair decode).
        let decomp = encode(&chars, OutputCharset::UnicodeDecomposed);
        assert_eq!(
            transcode(
                &decomp,
                Charset::UnicodeDecomposed,
                Charset::UnicodePrecomposed,
                TextCase::AsIs
            ),
            chars
        );
        assert_eq!(
            transcode(
                &chars,
                Charset::UnicodePrecomposed,
                Charset::UnicodeDecomposed,
                TextCase::AsIs
            ),
            decomp
        );

        // 4. Gỡ dấu (§4.2): gọi trực tiếp và qua TextCase::NoAccent.
        let src: Vec<char> = "Đại học Bách Khoa Hà Nội".chars().collect();
        let expect = "Dai hoc Bach Khoa Ha Noi";
        assert_eq!(
            strip_diacritics(&src).into_iter().collect::<String>(),
            expect
        );
        assert_eq!(
            transcode(
                &src,
                Charset::UnicodePrecomposed,
                Charset::UnicodePrecomposed,
                TextCase::NoAccent
            )
            .into_iter()
            .collect::<String>(),
            expect
        );
    }

    #[test]
    fn op007_transcode_unknown_chars_pass_through() {
        let chars: Vec<char> = "Hello 123 😀 — (wiki)".chars().collect();
        for cs in [
            Charset::Tcvn3,
            Charset::VniWindows,
            Charset::UnicodeDecomposed,
        ] {
            assert_eq!(
                transcode(&chars, cs, Charset::UnicodePrecomposed, TextCase::AsIs),
                chars,
                "from {cs:?}"
            );
            assert_eq!(
                transcode(&chars, Charset::UnicodePrecomposed, cs, TextCase::AsIs),
                chars,
                "to {cs:?}"
            );
        }
        // VIQR/TelexRaw v1 pass-through: chuỗi lạ đi qua bất biến.
        assert_eq!(
            transcode(&chars, Charset::Viqr, Charset::TelexRaw, TextCase::AsIs),
            chars
        );
    }

    #[test]
    fn op007_text_case_title_upper_lower() {
        let conv = |src: &str, case: TextCase| {
            transcode(
                &src.chars().collect::<Vec<_>>(),
                Charset::UnicodePrecomposed,
                Charset::UnicodePrecomposed,
                case,
            )
            .into_iter()
            .collect::<String>()
        };
        assert_eq!(conv("đơn vị mua bán", TextCase::Title), "Đơn Vị Mua Bán");
        assert_eq!(conv("HỘI đồng", TextCase::Title), "Hội Đồng");
        // Ranh giới từ mở rộng theo dấu ngoặc.
        assert_eq!(
            conv("đơn vị (mua bán)", TextCase::Title),
            "Đơn Vị (Mua Bán)"
        );
        // Mọi dấu phân cách (không chỉ 8 dấu cũ) mở từ mới: "A/B", "A:B".
        assert_eq!(conv("ban a/b", TextCase::Title), "Ban A/B");
        assert_eq!(conv("mục a:b", TextCase::Title), "Mục A:B");
        // Nháy đơn KHÔNG mở từ (tên kiểu "l'hôpital").
        assert_eq!(conv("l'hôpital", TextCase::Title), "L'hôpital");
        assert_eq!(conv("hội đồng", TextCase::Upper), "HỘI ĐỒNG");
        assert_eq!(conv("HỘI ĐỒNG", TextCase::Lower), "hội đồng");
        assert_eq!(conv("ĐẠI HỌC", TextCase::NoAccent), "DAI HOC"); // gỡ dấu, giữ kiểu chữ
    }

    #[test]
    fn op007_vni_longest_match_pair_beats_single() {
        let conv = |chars: &[char]| {
            transcode(
                chars,
                Charset::VniWindows,
                Charset::UnicodePrecomposed,
                TextCase::AsIs,
            )
            .into_iter()
            .collect::<String>()
        };

        // "ư" đơn là byte 0xF6, nhưng cặp [0xF6,0xF9] của bảng là "ứ":
        // greedy phải trọn CẢ CẶP (khớp đơn trước sẽ rách thành ư + ù).
        let in_0: Vec<char> = ['\u{f6}', '\u{f9}'].into();
        assert_eq!(
            decode_vni_windows(&in_0).into_iter().collect::<String>(),
            "ứ"
        );
        assert_eq!(conv(&in_0), "ứ");

        // Hoa cũng vậy: Ứ = [Ö, Ù] (0xD6, 0xD9).
        let in_up: Vec<char> = ['\u{d6}', '\u{d9}'].into();
        assert_eq!(
            decode_vni_windows(&in_up).into_iter().collect::<String>(),
            "Ứ"
        );

        // "đừ" = đ (0xF1, ký tự đơn) + ừ (cặp Ö+ø) — phải tách đúng từng khớp.
        let in_du: Vec<char> = ['\u{f1}', '\u{f6}', '\u{f8}'].into();
        assert_eq!(
            decode_vni_windows(&in_du).into_iter().collect::<String>(),
            "đừ"
        );

        // Ngược lại: "ơ" (0xF4) đứng trước 'a' — cặp [F4,a] KHÔNG có trong
        // bảng nên về khớp ký tự đơn: "ơ" + "a".
        let in_oa: Vec<char> = ['\u{f4}', 'a'].into();
        assert_eq!(
            decode_vni_windows(&in_oa).into_iter().collect::<String>(),
            "ơa"
        );
    }

    #[test]
    fn op007_strip_table_matches_charset_table() {
        let viet: Vec<char> = VIET_CHARS.chars().collect();
        let base: Vec<char> = BASE_CHARS.chars().collect();
        // Độ dài 2 chuỗi cùng = 134 và ánh xạ 1-1 đúng tại từng vị trí.
        assert_eq!(viet.len(), base.len());
        assert_eq!(
            strip_diacritics(&viet).into_iter().collect::<String>(),
            BASE_CHARS
        );
        // 134 chữ này là chính các dòng chữ Việt của bảng mã (186 dòng trừ
        // 52 chữ Latin chưa dấu a-z/A-Z): mỗi ký tự phải có trong bảng mã.
        assert!(table()
            .iter()
            .all(|e| viet.contains(&e.unicode) || e.unicode.is_ascii_alphabetic()));
        for v in &viet {
            assert!(
                table().binary_search_by_key(v, |e| e.unicode).is_ok(),
                "thiếu {v:?} trong bảng mã"
            );
        }
    }
}
