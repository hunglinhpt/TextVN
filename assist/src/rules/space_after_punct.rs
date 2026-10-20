// SPDX-License-Identifier: GPL-3.0-or-later
//! L4 - thieu khoang trang sau dau cau ("chao,ban").

use super::{issue, needs_space_after, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat dau cau khong cach tu ngay sau.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for i in 0..ctx.tokens.len().saturating_sub(1) {
        let punct = ctx.tokens[i];
        if punct.kind != TokenKind::Punct {
            continue;
        }
        if !needs_space_after(ctx.chars[punct.start]) {
            continue;
        }
        let next = ctx.tokens[i + 1];
        if next.kind != TokenKind::Word {
            continue;
        }
        if ctx.is_masked(punct.start, next.end) {
            continue;
        }
        out.push(issue(
            IssueKind::SpaceAfterPunct,
            Span {
                start: punct.start,
                end: punct.end,
            },
            "Thieu khoang trang sau dau cau".to_string(),
            vec![format!("{} ", ctx.token_text(&punct))],
            Fix::Insert {
                at: punct.end,
                text: " ".to_string(),
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_thieu_khoang_trang() {
        let issues = check("xin chào,bạn");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::SpaceAfterPunct);
    }

    #[test]
    fn khong_bat_voi_dau_ngoac() {
        assert!(check("(a)b").is_empty());
    }

    #[test]
    fn khong_bat_trong_so_thap_phan() {
        assert!(check("3.5 kg").is_empty());
    }
}
