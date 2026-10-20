// SPDX-License-Identifier: GPL-3.0-or-later
//! L6 - chu thuong o dau cau ("chao ban" sau dau cham).

use super::{is_sentence_end, is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat chu cai dau cau viet thuong.
///
/// - Khong bat sau dau `)` `]` `}` hay dau cham giua tu.
/// - Bo qua tu co chu hoa ben trong (`iPhone`) hoac toan chu hoa (`NASA`).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for idx in 0..ctx.tokens.len() {
        let token = ctx.tokens[idx];
        if !is_word(&token) {
            continue;
        }
        if !is_sentence_start(ctx, idx) {
            continue;
        }
        let text = ctx.token_text(&token);
        let Some(first) = text.chars().next() else {
            continue;
        };
        if !first.is_lowercase() {
            continue;
        }
        // `iPhone` / `eBay` - co hoa sau ky tu dau -> khong phai dau cau_that.
        if text.chars().skip(1).any(|c| c.is_uppercase()) {
            continue;
        }
        // `NASA` - toan chu hoa -> khong phai dau cau_that.
        if text.chars().all(|c| !c.is_lowercase()) {
            continue;
        }
        if ctx.is_masked(token.start, token.end) {
            continue;
        }
        let mut upper = first.to_uppercase().collect::<String>();
        upper.push_str(&text[first.len_utf8()..]);
        out.push(issue(
            IssueKind::LowercaseSentenceStart,
            Span {
                start: token.start,
                end: token.start + first.len_utf8(),
            },
            "Chu thuong o dau cau".to_string(),
            vec![upper.clone()],
            Fix::Replace {
                start: token.start,
                end: token.start + first.len_utf8(),
                text: upper,
            },
        ));
    }
}

/// Token co phai la dau cau khong (token 0, hoac sau dau ket thuc cau + khoang trang).
fn is_sentence_start(ctx: &Ctx, idx: usize) -> bool {
    if idx == 0 {
        // Ky tu dau doan chi la dau cau khi doan la cau tron ven.
        return ctx.start_of_sentence;
    }
    // Bo qua 1 token khoang trang ngay truoc.
    let mut j = idx;
    if ctx.tokens[j - 1].kind == TokenKind::Space {
        j -= 1;
    } else {
        return false;
    }
    if j == 0 {
        return false;
    }
    let prev = ctx.tokens[j - 1];
    prev.kind == TokenKind::Punct && is_sentence_end(ctx.chars[prev.start])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_dau_cau_thuong() {
        let issues = check("xin chào. bạn khỏe không");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::LowercaseSentenceStart);
    }

    #[test]
    fn khong_bat_dau_cau_da_hoa() {
        assert!(check("Xin chào. Bạn khỏe không").is_empty());
    }

    #[test]
    fn khong_bat_ten_rieng() {
        assert!(check("iPhone của tôi").is_empty());
    }
}
