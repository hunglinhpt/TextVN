// SPDX-License-Identifier: GPL-3.0-or-later
//! L5 - khoang trang kep ("a  b").

use super::{issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat run khoang trang >= 2 ky tu (bo qua xuong dong / tab).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
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
}
