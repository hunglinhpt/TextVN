// SPDX-License-Identifier: GPL-3.0-or-later
//! EN — mạo từ "a"/"an" không khớp **âm đầu** của từ kế tiếp (vòng 24).
//!
//! Chỉ chạy trên câu En-dominant (`lib.rs` gating) — "a ông ơi" tiếng Việt
//! không bao giờ bị soi. Quyết định theo **ÂM** (sound), không theo chữ —
//! nên mọi ngoại lệ là **danh sách đóng**; dạng lạ → im lặng (fail-open):
//!
//! - "a university / useful / user / one-way / euro" — chữ nguyên âm nhưng
//!   âm đầu là /j/ hoặc /w/ → "a".
//! - "an hour / honest / heir / honor" — chữ phụ âm nhưng h câm → "an".
//! - Cả hai dạng đều đúng ("an historic", "a FAQ"/"an FAQ") → KHÔNG bao giờ
//!   soi (danh sách both-ok + acronym mơ hồ).
//! - Số: đọc thành tiếng ("an 8-bit", "a 5-year") — bảng số đóng.
//!
//! `auto_fixable = true` (confidence 100): đích luôn đúng một chiều — "an
//! apple" không bao giờ sai khi nguồn là "a apple", đảo chiều tương tự.

use super::{is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// "a" trước từ bắt đầu bằng **chữ nguyên âm** nhưng **âm phụ âm** /j/,/w/.
const A_DESPITE_VOWEL_LETTER: &[&str] = &[
    "university",
    "universities",
    "useful",
    "user",
    "users",
    "uniform",
    "unique",
    "usual",
    "usually",
    "ubiquitous",
    "euro",
    "european",
    "europe",
    "one",
    "ones",
    "one-way",
    "once",
    "ewe",
    "uterus",
    "ukrainian",
    "you",
    "your",
    "youth",
    "unesco",
    "unicef",
];

/// "an" trước từ bắt đầu bằng **chữ phụ âm** nhưng **âm nguyên âm** (h câm).
const AN_DESPITE_CONSONANT_LETTER: &[&str] =
    &["hour", "hours", "honest", "heir", "honor", "honorary"];

/// Từ/acronym mà CẢ HAI dạng đều nghe tự nhiên — không bao giờ soi.
const BOTH_OK: &[&str] = &[
    "historic",
    "historical",
    "herb",
    "hotel",
    "faq",
    "sql",
    "url",
    "led",
    "ui",
    "ascii",
];

/// Acronym phát âm **như một từ** ("NATO", "NASA") — không đọc từng chữ,
/// áp luật chữ đầu bình thường ("a NATO", "a NASA").
const WORD_PRONOUNCED_ACRONYMS: &[&str] = &[
    "nato", "nasa", "laser", "radar", "scuba", "asap", "tosla", "opec", "mercosur",
];

/// Tên chữ cái phát âm bắt đầu bằng nguyên âm (đọc từng chữ: "an MBA",
/// "an FBI", "an SOS", "an X-ray", "an LCD"). Chỉ áp cho từ ALL-CAPS 2-5 ký tự.
const VOWEL_SOUND_LETTERS: &[char] = &['a', 'e', 'f', 'h', 'i', 'l', 'm', 'n', 'o', 'r', 's', 'x'];

/// Số đọc lên có âm nguyên âm đầu ("eight" → "an 8-bit"). Đóng theo bộ đọc
/// hàng chục/trăm; số khác → "a" (one, two, three, four, five, six, seven,
/// nine, ten… đều âm phụ âm).
fn number_starts_with_vowel_sound(digits: &str) -> Option<bool> {
    let d: Vec<char> = digits.chars().filter(|c| c.is_ascii_digit()).collect();
    if d.is_empty() {
        return None;
    }
    // Mọi số bắt đầu bằng 8 đọc "eight/eighty/eight-hundred" = âm nguyên âm;
    // 11 ("eleven") và 18 ("eighteen") nữa. Số còn lại đọc âm phụ âm
    // (one/two/five/ten…). "1" = "one" (/w/) → "a".
    Some(d[0] == '8' || digits == "11" || digits == "18")
}

/// Mạo từ đúng cho từ kế tiếp `next` (nguyên bản — cần phân biệt hoa/thường
/// cho acronym). `None` = không chắc → bỏ qua.
fn expected_article(next: &str) -> Option<&'static str> {
    if next.is_empty() || !next.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None; // từ có dấu/ký tự lạ → thoát
    }
    let next_lower = next.to_lowercase();
    if BOTH_OK.contains(&next_lower.as_str()) {
        return None; // cả hai đúng — không can thiệp
    }
    // Acronym ALL-CAPS đọc TỪNG CHỮ ("an MBA") — trừ dạng đọc-như-từ.
    // Vòng 28: áp cho MỌI độ dài ("a UNESCO" — 6 chữ vẫn đọc /juː/ → "a";
    // trước đây chỉ kiểm ≤5 chữ → "an UNESCO" sai bị bêu).
    let letters: Vec<char> = next.chars().collect();
    if next.chars().all(|c| c.is_ascii_uppercase())
        && !letters.is_empty()
        && next.chars().all(|c| c.is_ascii_alphabetic())
        && !WORD_PRONOUNCED_ACRONYMS.contains(&next_lower.as_str())
    {
        let first = letters[0].to_ascii_lowercase();
        return Some(if VOWEL_SOUND_LETTERS.contains(&first) {
            "an"
        } else {
            "a"
        });
    }
    // Số: đọc lên tiếng.
    if next.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return number_starts_with_vowel_sound(next).map(|v| if v { "an" } else { "a" });
    }
    if A_DESPITE_VOWEL_LETTER.contains(&next_lower.as_str()) {
        return Some("a");
    }
    if AN_DESPITE_CONSONANT_LETTER.contains(&next_lower.as_str()) {
        return Some("an");
    }
    let first = next_lower.chars().next()?;
    Some(if "aeiou".contains(first) { "an" } else { "a" })
}

/// Bắt mạo từ "a"/"an" lệch với âm đầu của từ kế tiếp.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Vòng 28 (audit HIGH): "an" là từ TIẾNG VIỆT THẬT ("an khang", "an toan")
    // — trong câu Vi-dominant, chỉ báo lỗi a/an khi câu có ít nhất một
    // **từ-điểm tiếng Anh** ("gửi a email nhé" có "email"; "an khang thịnh
    // vượng" không có → im lặng, không bao giờ sửa chữ Việt).
    if ctx.lang == crate::rules::language::Lang::Vi
        && !crate::rules::language::has_en_indicator(ctx.chars.as_slice())
    {
        return;
    }
    let mut i = 0usize;
    while i + 1 < ctx.tokens.len() {
        let article = ctx.tokens[i];
        if !is_word(&article) {
            i += 1;
            continue;
        }
        let text = ctx.token_text(&article);
        let lower = text.to_lowercase();
        if lower != "a" && lower != "an" {
            i += 1;
            continue;
        }
        if ctx.is_masked(article.start, article.end) {
            i += 1;
            continue;
        }
        // Từ kế tiếp (bỏ space).
        let mut j = i + 1;
        if ctx.tokens[j].kind == TokenKind::Space {
            if j + 1 >= ctx.tokens.len() {
                break;
            }
            j += 1;
        }
        let next = ctx.tokens[j];
        let next_text = ctx.token_text(&next);
        // Token số ("a 5-year") là TokenKind::Other và bị tách TỪNG chữ số —
        // gom cả dãy để đọc đúng ("11th" → "11" → "eleven" → "an").
        if next.kind == TokenKind::Other && next_text.chars().all(|c| c.is_ascii_digit()) {
            let mut digits = next_text.clone();
            let mut k = j + 1;
            while k < ctx.tokens.len() {
                let t = ctx.token_text(&ctx.tokens[k]);
                if ctx.tokens[k].kind == TokenKind::Other && t.chars().all(|c| c.is_ascii_digit()) {
                    digits.push_str(&t);
                    k += 1;
                } else {
                    break;
                }
            }
            if let Some(expected) =
                number_starts_with_vowel_sound(&digits).map(|v| if v { "an" } else { "a" })
            {
                if lower != expected {
                    push_issue(&article, &text, expected, out);
                }
            }
            i = k - 1; // nhảy qua dãy số đã xử lý
            continue;
        }
        if !is_word(&next) {
            i += 1;
            continue; // "a." / "a," — mạo từ đứng một mình, thoát
        }
        if next_text.chars().any(|c| !c.is_ascii_alphanumeric()) {
            i += 1;
            continue; // từ có dấu Việt/ký tự lạ trong câu mix → thoát
        }
        if ctx.is_masked(next.start, next.end) {
            i += 1;
            continue;
        }
        let Some(expected) = expected_article(&next_text) else {
            i += 1;
            continue;
        };
        if lower == expected {
            i += 1;
            continue;
        }
        push_issue(&article, &text, expected, out);
        i += 1;
    }
}

/// Đẩy issue a/an (giữ hoa đầu — hiếm: "A apple" đầu câu).
fn push_issue(article: &crate::tokenize::Token, text: &str, expected: &str, out: &mut Vec<Issue>) {
    let mut fixed = expected.to_string();
    if text.chars().next().is_some_and(|c| c.is_uppercase()) {
        if let Some(up) = fixed.chars().next().and_then(|c| c.to_uppercase().next()) {
            fixed.replace_range(0..up.len_utf8(), &up.to_string());
        }
    }
    out.push(issue(
        IssueKind::ArticleAn,
        Span {
            start: article.start,
            end: article.end,
        },
        format!("Mạo từ sai: \"{text}\" → \"{fixed}\""),
        vec![fixed.clone()],
        Fix::Replace {
            start: article.start,
            end: article.end,
            text: fixed,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::rules::Ctx;
    use crate::{Issue, IssueKind};

    fn en_issues(text: &str) -> Vec<Issue> {
        // Ctx tự classify — câu En-dominant.
        let ctx = Ctx::new(text, false);
        let mut out = Vec::new();
        check(&ctx, &mut out);
        out
    }

    #[test]
    fn bat_a_an_sai() {
        assert_eq!(en_issues("please check a apple").len(), 1);
        assert_eq!(en_issues("this is an university").len(), 1);
        assert_eq!(en_issues("he bought an book").len(), 1);
        let issues = en_issues("please send a email");
        assert_eq!(issues[0].kind, IssueKind::ArticleAn);
        assert_eq!(issues[0].suggestions, vec!["an".to_string()]);
    }

    #[test]
    fn khong_bat_ngoai_le_am() {
        // /j/, /w/ — chữ nguyên âm nhưng "a".
        for s in [
            "she goes to a university in europe",
            "it is a useful tool",
            "a one-way street",
            "a euro coin",
            "a user account",
            "a unique case",
        ] {
            assert!(en_issues(s).is_empty(), "\"{s}\" không được soi");
        }
        // h câm — "an".
        for s in [
            "he waited an hour",
            "an honest man",
            "an heir",
            "it is an honor",
        ] {
            assert!(en_issues(s).is_empty(), "\"{s}\" không được soi");
        }
    }

    #[test]
    fn khong_bat_dang_ca_hai_dung() {
        for s in [
            "it was an historic day",
            "a historic day",
            "see a FAQ page",
            "see an FAQ page",
            "run a SQL query",
            "an SQL query",
            "open a URL",
            "an LED light",
        ] {
            assert!(en_issues(s).is_empty(), "\"{s}\" cả hai dạng đều đúng");
        }
    }

    #[test]
    fn acronym_theo_ten_chu() {
        assert!(en_issues("she has an MBA").is_empty());
        assert_eq!(en_issues("she has a MBA").len(), 1);
        assert!(en_issues("a NATO meeting").is_empty());
        assert!(en_issues("an X-ray image").is_empty());
    }

    #[test]
    fn so_doc_len_tieng() {
        assert!(en_issues("an 8-bit value").is_empty());
        assert_eq!(en_issues("a 8-bit value").len(), 1);
        assert!(en_issues("a 5-year plan").is_empty());
        assert!(en_issues("an 11th-hour fix").is_empty());
    }

    /// Vòng 28 (audit HIGH): "an" là từ tiếng Việt thật — câu Vi không có
    /// từ-điểm EN khác thì KHÔNG bao giờ soi a/an (trước đây "an khang" bị
    /// tự sửa thành "a khang"!).
    #[test]
    fn an_tieng_viet_that_khong_bi_soi() {
        for s in [
            "Chúc bạn an khang thịnh vượng nhé.",
            "Làm việc an toan là quan trọng.",
            "Gia đình sống trong an bình.",
        ] {
            assert!(en_issues(s).is_empty(), "\"{s}\" không được soi a/an");
        }
        // Câu mix có từ-điểm EN ("email") → vẫn bắt được lỗi thật.
        assert_eq!(en_issues("Anh gửi a email cho tôi nhé.").len(), 1);
    }

    /// Vòng 28 (audit MEDIUM): UNESCO/unicef đọc /juː/ → "a UNESCO" đúng,
    /// "an UNESCO" sai (trước đây nằm nhầm danh sách đọc-từng-chữ).
    #[test]
    fn unesco_doc_nhu_tu() {
        assert!(en_issues("a UNESCO site is here").is_empty());
        assert_eq!(en_issues("this is an UNESCO site").len(), 1);
    }

    #[test]
    fn tu_co_dau_viet_khong_soi() {
        // "a ô" — thoát vì từ kế không ASCII.
        assert!(en_issues("a ô trong phòng").is_empty());
    }
}
