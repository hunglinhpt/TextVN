// SPDX-License-Identifier: GPL-3.0-or-later
//! L9 - lap tu o ranh gioi cau ("... di hoc. Hoc xong.").
//!
//! Luat **chi goi y** (khong tu sua): bo tu lap thuong lam doi nghia, nen
//! de nguoi dung quyet (Fassist-90).

use super::{is_real_sentence_end, issue_suggestion, Ctx};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Từ **chức năng** (đại từ · giới từ · trạng từ) — lặp những từ này ở ranh giới
/// câu mới là lỗi thật; lặp từ **nội dung** ("Tôi đi học. Học xong về nhà.") là
/// cách viết hợp lệ ⇒ không báo (Fassist-91).
const FUNCTION_WORDS: &[&str] = &[
    "tôi",
    "bạn",
    "anh",
    "chị",
    "em",
    "chúng tôi",
    "chúng ta",
    "các",
    "và",
    "hoặc",
    "nhưng",
    "của",
    "cho",
    "với",
    "cùng",
    "đã",
    "đang",
    "sẽ",
    "không",
    "có",
    "là",
    "rất",
    "nữa",
    "lại",
    "cũng",
    "vẫn",
    "còn",
    "đều",
    "vì",
    "nên",
    "thì",
    "mới",
    "rồi",
    "ra",
    "lên",
    "xuống",
];

/// Bat khi tu cuoi cau truoc bang tu dau cau sau.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for i in 0..ctx.tokens.len() {
        let punct = ctx.tokens[i];
        if punct.kind != TokenKind::Punct || !is_real_sentence_end(ctx, &punct) {
            continue;
        }
        // tu gan nhat truoc dau cham
        let Some(prev) = word_before(ctx, i) else {
            continue;
        };
        // tu gan nhat sau dau cham (bo qua khoang trang)
        let Some(next) = word_after(ctx, i) else {
            continue;
        };
        if prev.len() != next.len() || prev.len() < 2 || ctx.is_masked(prev.start, next.end) {
            continue;
        }
        let prev_text = ctx.token_text(&prev);
        let next_text = ctx.token_text(&next);
        let prev_lower = prev_text.to_lowercase();
        if prev_lower != next_text.to_lowercase() {
            continue;
        }
        // Chi bat voi tu chuc nang (Fassist-91) - lap tu noi dung la cach viet hop le.
        if !FUNCTION_WORDS.contains(&prev_lower.as_str()) {
            continue;
        }
        out.push(issue_suggestion(
            IssueKind::SentenceRepeat,
            Span {
                start: next.start,
                end: next.end,
            },
            format!("Lặp từ qua ranh giới câu: \"{next_text}\""),
            vec![format!("Bỏ từ \"{next_text}\" hoặc thay bằng từ khác")],
        ));
    }
}

/// Từ (token Word) ngay trước vị trí `idx`.
fn word_before(ctx: &Ctx, idx: usize) -> Option<crate::tokenize::Token> {
    (0..idx)
        .rev()
        .find(|&k| ctx.tokens[k].kind == TokenKind::Word)
        .map(|k| ctx.tokens[k])
}

/// Từ (token Word) ngay sau vị trí `idx`.
fn word_after(ctx: &Ctx, idx: usize) -> Option<crate::tokenize::Token> {
    ((idx + 1)..ctx.tokens.len())
        .find(|&k| ctx.tokens[k].kind == TokenKind::Word)
        .map(|k| ctx.tokens[k])
}
