// SPDX-License-Identifier: GPL-3.0-or-later
//! L3 - thua khoang trang truoc dau cau ("xin chao ,").

use super::{issue, needs_space_after, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat khoang trang thua ngay truoc dau cau.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for i in 1..ctx.tokens.len() {
        let punct = ctx.tokens[i];
        if punct.kind != TokenKind::Punct {
            continue;
        }
        if !needs_space_after(ctx.chars[punct.start]) {
            continue;
        }
        let prev = ctx.tokens[i - 1];
        if prev.kind != TokenKind::Space {
            continue;
        }
        // Run khoang trang nhieu ky tu -> luat `double_space` xu ly.
        if prev.len() != 1 {
            continue;
        }
        if ctx.is_masked(prev.start, prev.end) {
            continue;
        }
        out.push(issue(
            IssueKind::SpaceBeforePunct,
            Span {
                start: prev.start,
                end: prev.end,
            },
            "Thua khoang trang truoc dau cau".to_string(),
            vec![ctx.token_text(&punct)],
            Fix::Delete {
                start: prev.start,
                end: prev.end,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_khoang_trang_thua() {
        let issues = check("xin chào , bạn");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::SpaceBeforePunct);
    }

    #[test]
    fn khong_bat_khi_dung_dau_cau() {
        assert!(check("xin chào, bạn").is_empty());
    }
}
