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
        let last_ch = ctx.chars[punct.end - 1];
        if !needs_space_after(last_ch) {
            continue;
        }
        // Vòng 24 (audit EN trap 31-35 — bug SỐNG phía VI từ trước): dấu `.`
        // của từ viết tắt/tên miền ("e.g.", "a.m.", "U.S.", "example.com") —
        // chèn space sẽ phá chữ đúng ("a.m." → "a. m.").
        // Vòng 46 (audit perf D-dot): fast-path O(1) — token trước punct là
        // `tokens[i - 1]`, thay scan ngược O(n) ⇒ O(n²) trên chuỗi dot dày
        // ("1.1.1..." 160k chars 4.2s → tuyến tính).
        if last_ch == '.'
            && super::is_abbreviation_dot_at(
                ctx,
                if i > 0 {
                    Some(&ctx.tokens[i - 1])
                } else {
                    None
                },
                &punct,
            )
        {
            continue;
        }
        let next = ctx.tokens[i + 1];
        if next.kind != TokenKind::Word {
            continue;
        }
        // Emoticon chữ (":v", ":D", ":P") khi đứng sau khoảng trắng ("xin chào :v").
        if last_ch == ':'
            && punct.len() == 1
            && i > 0
            && ctx.tokens[i - 1].kind == TokenKind::Space
            && next.len() == 1
            && matches!(
                ctx.chars[next.start],
                'v' | 'V' | 'd' | 'D' | 'p' | 'P' | 'o' | 'O'
            )
        {
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
            "Thiếu khoảng trắng sau dấu câu".to_string(),
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
