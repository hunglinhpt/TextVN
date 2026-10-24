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
        // Vòng 24 (audit EN trap 36): dãy dấu chấm cách (APA ". . .") và "..."
        // rời — space trước MỘT chấm thuộc dãy rung, không phải lỗi. Dấu chấm
        // cuối câu thật có ký tự khác dấu chấm ở cả hai phía của space.
        if ctx.chars[punct.start] == '.' {
            let before_space = prev.start > 0 && ctx.chars[prev.start - 1] == '.';
            let after_dot = ctx.chars.get(punct.end) == Some(&'.');
            if before_space || after_dot {
                continue;
            }
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
            "Thừa khoảng trắng trước dấu câu".to_string(),
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
