// SPDX-License-Identifier: GPL-3.0-or-later
//! L12 - dau phay o cuoi cau ("tinh hinh nay ,").
//!
//! Luat **chi goi y** (khong tu sua): dau phay cuoi cau thuong la loi go
//! (nen dung dau cham) — de nguoi dung quyet (Fassist-90).

use super::{issue_suggestion, Ctx};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Bat dau phay ngay truoc het cau.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // lay token cuoi khac khoang trang
    let Some(last) = ctx.tokens.iter().rposition(|t| t.kind != TokenKind::Space) else {
        return;
    };
    let tok = ctx.tokens[last];
    if tok.kind != TokenKind::Punct {
        return;
    }
    if ctx.chars[tok.start] != ',' {
        return;
    }
    out.push(issue_suggestion(
        IssueKind::CommaAtEnd,
        Span {
            start: tok.start,
            end: tok.end,
        },
        "Dấu phẩy ở cuối câu".to_string(),
        vec!["Đổi thành dấu chấm câu".to_string()],
    ));
}
