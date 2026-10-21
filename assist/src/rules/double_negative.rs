// SPDX-License-Identifier: GPL-3.0-or-later
//! L10 - phu dinh kep ("khong co ... khong").
//!
//! Luat **chi goi y** (khong tu sua) — nhip mau chu "khong co gi do khong"
//! van co the dung nghia ("khong co gi de ma khong lam") => de nguoi dung quyet
//! (Fassist-90).

use super::{issue_suggestion, Ctx};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Số từ tối đa được phép nằm giữa "không có" và "không" thứ hai.
const MAX_GAP_WORDS: usize = 4;

/// Bat "không có ... không" trong cùng câu.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    let words: Vec<usize> = (0..ctx.tokens.len())
        .filter(|&k| ctx.tokens[k].kind == TokenKind::Word)
        .collect();
    for (pos, &w) in words.iter().enumerate() {
        let a = ctx.token_text(&ctx.tokens[w]);
        let b = words
            .get(pos + 1)
            .map(|&n| ctx.token_text(&ctx.tokens[n]))
            .unwrap_or_default();
        if a.to_lowercase() != "không" || b.to_lowercase() != "có" {
            continue;
        }
        // tim "khong" thu hai trong toi da 4 tu ke
        for &n in words.iter().skip(pos + 2).take(MAX_GAP_WORDS + 1) {
            if ctx.token_text(&ctx.tokens[n]).to_lowercase() == "không" {
                out.push(issue_suggestion(
                    IssueKind::DoubleNegative,
                    Span {
                        start: ctx.tokens[w].start,
                        end: ctx.tokens[w].end,
                    },
                    "Phủ định kép: \"không có ... không\"".to_string(),
                    vec!["Cân nhắc viết lại để tránh phủ định kép".to_string()],
                ));
                break;
            }
        }
    }
}
