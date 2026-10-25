// SPDX-License-Identifier: GPL-3.0-or-later
//! L5 - khoang trang kep ("a  b").

use super::{issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat run khoang trang >= 2 ky tu (bo qua xuong dong / tab).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Dòng code / danh sách (vòng 32 — cùng tiêu chí với capitalize, vòng 24):
    // khoảng trắng kép trong code là căn lề CỐ Ý ("x = 5;  y = 6") — fail-open.
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    for token in &ctx.tokens {
        if token.kind != TokenKind::Space {
            continue;
        }
        let run = &ctx.chars[token.start..token.end];
        if run.iter().any(|c| matches!(c, '\n' | '\r' | '\t')) {
            continue;
        }
        if run.len() < 2 {
            continue;
        }
        // Chuẩn comment cùng dòng (PEP 8: 2 space trước `#`; YAML/shell/C):
        // "i = i + 1  # tăng i" là văn bản ĐÚNG — không bêu.
        if ctx.chars.get(token.end) == Some(&'#') {
            continue;
        }
        if ctx.is_masked(token.start, token.end) {
            continue;
        }
        // Giu lai 1 khoang trang, xoa phan con lai.
        out.push(issue(
            IssueKind::DoubleSpace,
            Span {
                start: token.start + 1,
                end: token.end,
            },
            "Khoảng trắng kép".to_string(),
            vec![" ".to_string()],
            Fix::Delete {
                start: token.start + 1,
                end: token.end,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_khoang_trang_kep() {
        let issues = check("xin  chào");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DoubleSpace);
    }

    #[test]
    fn khong_bat_xuong_dong() {
        assert!(check("a\n\nb").is_empty());
    }

    /// Vòng 32 — khoảng trắng kép trước `#` là chuẩn comment cùng dòng
    /// (PEP 8/YAML/shell) — KHÔNG bêu văn bản đúng.
    #[test]
    fn khong_bat_khoang_truoc_comment() {
        assert!(check("i = i + 1  # tăng i").is_empty());
        assert!(check("gõ  #TextVN nhé").is_empty());
        // Khoảng kép thường vẫn phải bắt.
        assert_eq!(check("xin  chào").len(), 1);
    }

    /// Vòng 32 — dòng code căn lề cố ý ("x = 5;  y = 6") không bêu
    /// (cùng tiêu chí code với capitalize, vòng 24).
    #[test]
    fn khong_bat_dong_code() {
        assert!(check("x = 5;  y = 6").is_empty());
    }
}
