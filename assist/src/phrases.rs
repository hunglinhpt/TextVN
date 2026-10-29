// SPDX-License-Identifier: GPL-3.0-or-later
//! Cụm cố định hợp lệ (từ láy, thành ngữ, cụm tiếng Anh) — **vùng được bảo vệ**.
//!
//! Vấn đề gốc (user báo 2026-10-29): bộ luật coi mọi cặp từ giống nhau liền
//! kề là lỗi nên bêu và còn tự xoá "song song", "đều đều", "người người",
//! "knock knock"… Từ láy / thành ngữ không phải lỗi, mô tả đúng nghĩa.
//!
//! Cách chữa (G15 — một lớp dùng chung, không rải từng luật): bảng cụm nạp
//! từ `data/assist/{vi,en}_fixed_phrases.txt` (nhúng lúc biên dịch, 0 I/O);
//! [`protected_spans`] tìm mọi cụm trong câu; điểm chặn DUY NHẤT
//! ([`drop_issues_in_protected`], gọi ở `check_with_internal`) bỏ mọi lỗi
//! **từ vựng** (lặp từ, chính tả, nhầm lẫn, phủ định kép…) có span chồng lên
//! vùng đó. Lỗi hình thức (khoảng trắng, dấu câu, viết hoa) vẫn báo bình
//! thường — cụm hợp lệ không miễn trừ việc gõ thừa dấu cách.

use crate::tokenize::{Token, TokenKind};
use crate::{Issue, IssueKind};
use std::collections::HashSet;
use std::sync::OnceLock;

const VI_DATA: &str = include_str!("../../data/assist/vi_fixed_phrases.txt");
const EN_DATA: &str = include_str!("../../data/assist/en_fixed_phrases.txt");

/// Số từ tối đa của một cụm (thành ngữ dài nhất trong bảng ≈ 9 âm tiết).
const MAX_PHRASE_WORDS: usize = 9;

fn load(data: &'static str) -> HashSet<&'static str> {
    data.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

fn vi_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| load(VI_DATA))
}

fn en_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| load(EN_DATA))
}

/// `phrase` (đã lowercase, từ cách nhau một dấu cách) có trong bảng cụm hợp lệ
/// không. Bảng tiếng Việt chỉ áp khi câu **Vi-dominant** hoặc cụm có chữ có dấu
/// — nếu không, \"to to\"/\"he he\" (từ láy Việt ASCII) sẽ che lỗi gõ thật của
/// câu tiếng Anh (\"I went to to the store\").
pub fn is_fixed_phrase_for(phrase: &str, vi_dominant: bool) -> bool {
    en_set().contains(phrase) || (vi_set().contains(phrase) && (vi_dominant || !phrase.is_ascii()))
}

/// Như [`is_fixed_phrase_for`] nhưng coi câu là Vi-dominant (cả hai bảng).
pub fn is_fixed_phrase(phrase: &str) -> bool {
    is_fixed_phrase_for(phrase, true)
}

/// Tìm mọi cụm cố định trong câu: `(start, end)` theo ký tự Unicode, đã gộp
/// vùng chồng nhau. Chỉ nối các từ cách nhau **đúng một dấu cách đơn** (cụm
/// có dấu phẩy/gạch nối xen giữa không khớp — đó là câu khác).
pub fn protected_spans(chars: &[char], tokens: &[Token], vi_dominant: bool) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    // Mỗi "chuỗi từ liền" = các Word token nối nhau bằng đúng 1 dấu cách.
    let mut run: Vec<Token> = Vec::new();
    let flush = |run: &mut Vec<Token>, spans: &mut Vec<(usize, usize)>| {
        if run.len() >= 2 {
            scan_run(chars, run, vi_dominant, spans);
        }
        run.clear();
    };
    let mut i = 0usize;
    while i < tokens.len() {
        let t = tokens[i];
        if t.kind == TokenKind::Word {
            run.push(t);
            let linked = tokens
                .get(i + 1)
                .zip(tokens.get(i + 2))
                .is_some_and(|(g, n)| {
                    g.kind == TokenKind::Space
                        && g.len() == 1
                        && chars[g.start] == ' '
                        && n.kind == TokenKind::Word
                });
            if linked {
                i += 2; // bỏ qua dấu cách, nhảy tới từ kế
                continue;
            }
            flush(&mut run, &mut spans);
        } else {
            flush(&mut run, &mut spans);
        }
        i += 1;
    }
    flush(&mut run, &mut spans);
    merge(spans)
}

fn scan_run(chars: &[char], run: &[Token], vi_dominant: bool, spans: &mut Vec<(usize, usize)>) {
    let lowered: Vec<String> = run
        .iter()
        .map(|t| {
            chars[t.start..t.end]
                .iter()
                .collect::<String>()
                .to_lowercase()
        })
        .collect();
    for start in 0..run.len() {
        let max_len = MAX_PHRASE_WORDS.min(run.len() - start);
        // Cụm dài nhất trước: thành ngữ dài che từ láy nằm trong nó.
        for len in (2..=max_len).rev() {
            let phrase = lowered[start..start + len].join(" ");
            if is_fixed_phrase_for(&phrase, vi_dominant) {
                spans.push((run[start].start, run[start + len - 1].end));
                break;
            }
        }
    }
}

fn merge(mut spans: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    spans.sort_unstable();
    let mut out: Vec<(usize, usize)> = Vec::with_capacity(spans.len());
    for (s, e) in spans {
        match out.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => out.push((s, e)),
        }
    }
    out
}

/// Kind thuộc lớp **từ vựng/ngữ pháp** — được miễn trong vùng bảo vệ. Kind
/// hình thức (dấu cách, dấu câu, viết hoa đầu câu/đại từ) KHÔNG miễn.
fn is_lexical(kind: IssueKind) -> bool {
    !matches!(
        kind,
        IssueKind::SpaceBeforePunct
            | IssueKind::SpaceAfterPunct
            | IssueKind::DoubleSpace
            | IssueKind::LowercaseSentenceStart
            | IssueKind::CommaAtEnd
            | IssueKind::LowercasePronoun
    )
}

/// Bỏ mọi lỗi từ vựng có span chồng lên vùng bảo vệ (điểm chặn duy nhất).
pub fn drop_issues_in_protected(issues: &mut Vec<Issue>, protected: &[(usize, usize)]) {
    if protected.is_empty() {
        return;
    }
    issues.retain(|i| {
        !is_lexical(i.kind)
            || !protected
                .iter()
                .any(|&(s, e)| i.span.start < e && s < i.span.end)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize::tokenize_chars;

    fn spans(text: &str) -> Vec<(usize, usize)> {
        let chars: Vec<char> = text.chars().collect();
        let tokens = tokenize_chars(&chars);
        protected_spans(&chars, &tokens, true)
    }

    #[test]
    fn nhan_ra_tu_lay_giua_cau() {
        // "song song" nằm ở ký tự 21..30.
        assert_eq!(spans("Hai đường thẳng song song với nhau."), vec![(16, 25)]);
        assert_eq!(spans("Tôi đi học đều đều."), vec![(11, 18)]);
    }

    #[test]
    fn khong_nhan_khi_co_dau_cau_xen_giua() {
        assert!(spans("song, song").is_empty());
        assert!(spans("song  song").is_empty(), "hai dấu cách ≠ cụm");
    }

    #[test]
    fn thanh_ngu_dai_che_tu_lay_ben_trong() {
        let s = spans("Họ sống ba chìm bảy nổi.");
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].1 - s[0].0), "ba chìm bảy nổi".chars().count());
    }

    #[test]
    fn cum_tieng_anh_khong_phan_biet_hoa_thuong() {
        assert_eq!(spans("Bye Bye now").len(), 1);
        assert!(
            spans("the the report").is_empty(),
            "lỗi gõ thật không được bảo vệ"
        );
    }

    #[test]
    fn bang_khong_chua_loi_danh_may() {
        // Các cặp này phải LUÔN bị bêu — không được lọt vào bảng cụm hợp lệ.
        for typo in ["tôi tôi", "của của", "và và"] {
            assert!(
                !is_fixed_phrase(typo),
                "{typo} là lỗi gõ, không phải cụm hợp lệ"
            );
        }
        // Câu tiếng Anh: bảng Việt tắt, nên \"to to\"/\"he he\" vẫn bị bêu.
        for typo in ["the the", "is is", "to to", "was was", "he he", "no no"] {
            assert!(
                !is_fixed_phrase_for(typo, false),
                "{typo} là lỗi gõ trong câu tiếng Anh"
            );
        }
        // Từ láy ASCII hợp lệ trong câu tiếng Việt, nhưng KHÔNG che câu tiếng Anh.
        assert!(is_fixed_phrase_for("song song", true));
        assert!(!is_fixed_phrase_for("song song", false));
    }
}
