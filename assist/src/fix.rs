// SPDX-License-Identifier: GPL-3.0-or-later
//! Sua loi tu dong - dung cho Mode 2 ("tu sua ca cau") va de test (P4-0 §4).

use crate::{Issue, Span};

/// Cach sua mot loi. Offset la **ky tu Unicode** (char).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fix {
    /// Xoa doan `[start, end)`.
    Delete {
        /// Offset bat dau (inclusive).
        start: usize,
        /// Offset ket thuc (exclusive).
        end: usize,
    },
    /// Chen `text` tai `at`.
    Insert {
        /// Offset vi tri chen (ky tu Unicode).
        at: usize,
        /// Chuoi chen vao.
        text: String,
    },
    /// Thay doan `[start, end)` bang `text`.
    Replace {
        /// Offset bat dau (inclusive).
        start: usize,
        /// Offset ket thuc (exclusive).
        end: usize,
        /// Chuoi thay the.
        text: String,
    },
}

/// Doi offset ky tu Unicode sang offset byte UTF-8.
#[allow(dead_code)]
fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
}

/// Ap cac loi sua duoc tu dong vao `text`.
///
/// - Chi ap loi `auto_fixable` va co [`Fix`].
/// - Fix chong nhau (overlap) bi bo qua - loi do giu lai de nguoi dung quyet.
/// - Tinh toan hoan toan trong **khong gian ky tu** (char) nen khong bao gio
///   lech offset khi sua nhiieu loi tren cung mot cau.
///
/// Tra ve `text` goc neu khong co fix nao ap duoc.
pub fn apply_auto_fixes(text: &str, issues: &[Issue]) -> String {
    let chars: Vec<char> = text.chars().collect();

    // Chon cac fix khong chong nhau, sap xep theo vi tri.
    let mut candidates: Vec<(usize, usize, &str)> = Vec::new();
    for issue in issues {
        if !issue.auto_fixable {
            continue;
        }
        let Some(fix) = issue.fix.as_ref() else {
            continue;
        };
        match fix {
            Fix::Delete { start, end } => candidates.push((*start, *end, "")),
            Fix::Replace { start, end, text } => candidates.push((*start, *end, text)),
            Fix::Insert { at, text } => candidates.push((*at, *at, text)),
        }
    }
    candidates.sort_by_key(|(start, end, _)| (*start, *end));

    let mut selected: Vec<(usize, usize, &str)> = Vec::new();
    let mut last_end = 0usize;
    for (start, end, replacement) in candidates {
        if start < last_end {
            // Overlap voi fix da chon - bo qua, giu loi cho nguoi dung.
            continue;
        }
        last_end = end;
        selected.push((start, end, replacement));
    }

    if selected.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for (start, end, replacement) in selected {
        if start > cursor {
            out.extend(chars[cursor..start].iter());
        }
        out.push_str(replacement);
        cursor = end.max(cursor);
    }
    out.extend(chars[cursor.min(chars.len())..].iter());
    out
}

/// Do dai span theo ky tu Unicode.
pub fn span_len(span: Span) -> usize {
    span.end - span.start
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{check, IssueKind};

    #[test]
    fn sua_lap_tu() {
        let text = "tôi tôi đi";
        let fixed = apply_auto_fixes(text, &check(text));
        assert_eq!(fixed, "tôi đi");
    }

    #[test]
    fn sua_khoang_trang() {
        let text = "xin chào , bạn";
        let fixed = apply_auto_fixes(text, &check(text));
        assert_eq!(fixed, "xin chào, bạn");
    }

    #[test]
    fn khong_sua_khi_khong_co_loi() {
        let text = "Tôi đi học.";
        assert_eq!(apply_auto_fixes(text, &check(text)), text);
    }

    #[test]
    fn chi_giu_loi_can_sua() {
        let issues = check("tôi tôi");
        assert!(issues.iter().any(|i| i.kind == IssueKind::DuplicateWord));
    }

    #[test]
    fn scenario_1_cau_tieng_viet_hop_le() {
        let text = "Hôm nay trời đẹp.";
        let issues = crate::check_sentence(text);
        assert!(issues.is_empty());
        assert_eq!(apply_auto_fixes(text, &issues), text);
    }

    #[test]
    fn scenario_2_lap_tu_mode2_tu_sua() {
        let text = "tôi tôi đi học.";
        // Fragment check: chi bat duplicate_word
        let issues = crate::check(text);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DuplicateWord);
        assert_eq!(apply_auto_fixes(text, &issues), "tôi đi học.");

        // Sentence check: bat ca lowercase_sentence_start va duplicate_word
        let s_issues = crate::check_sentence(text);
        assert_eq!(s_issues.len(), 2);
        assert_eq!(apply_auto_fixes(text, &s_issues), "Tôi đi học.");
    }

    #[test]
    fn scenario_3_thua_khoang_trang_truoc_dau() {
        let text = "đi học .";
        // Fragment check: chi bat space_before_punct
        let issues = crate::check(text);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::SpaceBeforePunct);
        assert_eq!(apply_auto_fixes(text, &issues), "đi học.");

        // Sentence check: bat ca lowercase_sentence_start va space_before_punct
        let s_issues = crate::check_sentence(text);
        assert_eq!(s_issues.len(), 2);
        assert_eq!(apply_auto_fixes(text, &s_issues), "Đi học.");
    }

    #[test]
    fn scenario_4_chu_thuong_dau_cau() {
        let text = "hôm nay tôi đi học.";
        let issues = crate::check_sentence(text);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::LowercaseSentenceStart);
        assert_eq!(apply_auto_fixes(text, &issues), "Hôm nay tôi đi học.");
    }
}
