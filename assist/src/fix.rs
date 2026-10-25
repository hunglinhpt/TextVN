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

/// Kế hoạch tự sửa cả câu (AS-050 · D12 · D22 · D27).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceFixPlan {
    /// Số lượng lỗi objective được sửa.
    pub fixed_count: usize,
    /// Vị trí ký tự char đầu tiên bị thay đổi trong câu (0-based).
    pub delete_from: usize,
    /// Số ký tự Unicode trong câu bị xoá (từ delete_from tới hết câu).
    pub delete_chars: usize,
    /// Chuỗi UTF-8 thay thế từ delete_from tới hết câu.
    pub replacement: String,
    /// Đuôi câu gốc bị xoá từ delete_from tới hết câu.
    pub original_tail: String,
}

/// Doi offset ky tu Unicode sang offset byte UTF-8.
#[allow(dead_code)]
fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
}

/// Chọn các fix không chồng nhau, nằm trong biên câu, và thực sự làm đổi chuỗi.
fn select_non_overlapping_fixes<'a>(
    chars: &[char],
    issues: &'a [Issue],
) -> Vec<(usize, usize, &'a str)> {
    let mut candidates: Vec<(usize, usize, &'a str)> = Vec::new();
    for issue in issues {
        if !issue.auto_fixable {
            continue;
        }
        let Some(fix) = issue.fix.as_ref() else {
            continue;
        };
        match fix {
            Fix::Delete { start, end } => candidates.push((*start, *end, "")),
            Fix::Replace { start, end, text } => candidates.push((*start, *end, text.as_str())),
            Fix::Insert { at, text } => candidates.push((*at, *at, text.as_str())),
        }
    }
    candidates.retain(|&(start, end, rep)| {
        start <= end && end <= chars.len() && chars[start..end].iter().copied().ne(rep.chars())
    });
    candidates.sort_by_key(|(start, end, _)| (*start, *end));

    let mut selected: Vec<(usize, usize, &'a str)> = Vec::new();
    let mut last_end = 0usize;
    for (start, end, replacement) in candidates {
        if start < last_end {
            // Overlap voi fix da chon - bo qua, giu loi cho nguoi dung.
            continue;
        }
        last_end = end;
        selected.push((start, end, replacement));
    }
    selected
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
    let selected = select_non_overlapping_fixes(&chars, issues);
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

/// Tính kế hoạch sửa cả câu cho các lỗi objective (Mode 2 / AS-050).
///
/// - Dựa trên logic của [`apply_auto_fixes`]: chỉ áp dụng lỗi `auto_fixable` có [`Fix`].
/// - Chọn các fix không overlap, sắp xếp theo start tăng dần.
/// - `delete_from` là vị trí `start` của fix đầu tiên (`selected[0].0`).
/// - `original_tail` = chuỗi ký tự từ `delete_from` tới hết câu (`chars[delete_from..]`).
/// - `replacement` = chuỗi đã sửa từ `delete_from` tới hết câu.
/// - `delete_chars` = số ký tự Unicode của `original_tail`.
/// - Trả về `None` nếu `replacement == original_tail` hoặc `selected.is_empty()`.
pub fn plan_sentence_fix(text: &str, issues: &[Issue]) -> Option<SentenceFixPlan> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let selected = select_non_overlapping_fixes(&chars, issues);
    if selected.is_empty() {
        return None;
    }

    let delete_from = selected[0].0;
    let original_tail: String = chars[delete_from..].iter().collect();

    let mut replacement = String::with_capacity(original_tail.len());
    let mut cursor = delete_from;
    for (start, end, rep) in &selected {
        if *start > cursor {
            replacement.extend(chars[cursor..*start].iter());
        }
        replacement.push_str(rep);
        cursor = (*end).max(cursor);
    }
    if cursor < chars.len() {
        replacement.extend(chars[cursor..].iter());
    }

    if replacement == original_tail {
        return None;
    }

    let delete_chars = original_tail.chars().count();
    Some(SentenceFixPlan {
        fixed_count: selected.len(),
        delete_from,
        delete_chars,
        replacement,
        original_tail,
    })
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

    #[test]
    fn test_plan_sentence_fix_khong_loi() {
        let text = "Tôi đi học.";
        let issues = crate::check_sentence(text);
        assert_eq!(plan_sentence_fix(text, &issues), None);
    }

    #[test]
    fn test_plan_sentence_fix_lap_tu_fragment() {
        let text = "Tôi tôi đi học.";
        let issues = crate::check(text);
        let plan = plan_sentence_fix(text, &issues).expect("co plan");
        assert_eq!(plan.fixed_count, 1);
        assert_eq!(plan.delete_from, 3);
        assert_eq!(plan.original_tail, " tôi đi học.");
        assert_eq!(plan.replacement, " đi học.");
        assert_eq!(plan.delete_chars, " tôi đi học.".chars().count());

        let head: String = text.chars().take(plan.delete_from).collect();
        assert_eq!(format!("{head}{}", plan.replacement), "Tôi đi học.");
        assert_eq!(
            format!("{head}{}", plan.replacement),
            apply_auto_fixes(text, &issues)
        );
    }

    #[test]
    fn test_plan_sentence_fix_nhieu_loi_sentence() {
        let text = "tôi tôi đi học .";
        let issues = crate::check_sentence(text);
        // Co 3 loi: lowercase start, duplicate word, space before punct
        assert_eq!(issues.len(), 3);
        let plan = plan_sentence_fix(text, &issues).expect("co plan");
        assert_eq!(plan.fixed_count, 3);
        assert_eq!(plan.delete_from, 0);
        assert_eq!(plan.original_tail, text);
        assert_eq!(plan.replacement, "Tôi đi học.");
        assert_eq!(plan.delete_chars, text.chars().count());
        assert_eq!(plan.replacement, apply_auto_fixes(text, &issues));
    }

    #[test]
    fn test_plan_sentence_fix_overlap_bo_qua() {
        let text = "xin chào bạn";
        // Tao 2 issue co fix chong nhau
        let issues = vec![
            Issue {
                kind: IssueKind::DuplicateWord,
                span: Span { start: 0, end: 3 },
                message: "test 1".to_string(),
                suggestions: vec![],
                auto_fixable: true,
                confidence: 100,
                fix: Some(Fix::Replace {
                    start: 0,
                    end: 3,
                    text: "Xin".to_string(),
                }),
            },
            Issue {
                kind: IssueKind::DuplicateWord,
                span: Span { start: 2, end: 5 },
                message: "test 2".to_string(),
                suggestions: vec![],
                auto_fixable: true,
                confidence: 100,
                fix: Some(Fix::Replace {
                    start: 2,
                    end: 5,
                    text: "XYZ".to_string(),
                }),
            },
        ];
        let plan = plan_sentence_fix(text, &issues).expect("co plan");
        assert_eq!(plan.fixed_count, 1);
        assert_eq!(plan.delete_from, 0);
        assert_eq!(plan.replacement, "Xin chào bạn");
    }

    #[test]
    fn test_plan_sentence_fix_bo_qua_non_auto_fixable() {
        let text = "quẫn bách";
        let issues = vec![Issue {
            kind: IssueKind::Spelling,
            span: Span { start: 0, end: 4 },
            message: "từ lạ".to_string(),
            suggestions: vec!["quân".to_string()],
            auto_fixable: false,
            confidence: 60,
            fix: Some(Fix::Replace {
                start: 0,
                end: 4,
                text: "quân".to_string(),
            }),
        }];
        assert_eq!(plan_sentence_fix(text, &issues), None);
    }
}
