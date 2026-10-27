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
        // OP-005 (P5): khoảng trắng thừa giữa CHỮ SỐ và '%' ("100 %" →
        // "100%") — báo cáo tài chính rất hay gõ thừa; corpus kiểm chứng
        // không câu sạch nào dùng "số space %" (FPR = 0). Bypass gate
        // `needs_space_after` vì '%' không nằm trong bộ dấu câu của nó.
        // KHÔNG đòi punct.len()==1: tokenizer gộp punct cùng loại ("100 %.")
        // thành run "%." — chỉ cần run BẮT ĐẦU bằng '%'.
        let is_percent_after_digit = ctx.chars[punct.start] == '%'
            && i >= 2
            && ctx.tokens[i - 1].kind == TokenKind::Space
            && ctx.tokens[i - 1].len() == 1
            && ctx.chars[ctx.tokens[i - 1].start] == ' '
            && ctx.tokens[i - 2].kind == TokenKind::Other
            && ctx.chars[ctx.tokens[i - 2].start..ctx.tokens[i - 2].end]
                .iter()
                .all(|c| c.is_ascii_digit())
            // Vòng 34 (R1-F5): "phép 10 % 3" — modulo toán học có space + CHỮ
            // SỐ sau '%' ⇒ bỏ qua (chỉ flag khi sau '%' là chữ/cụm từ).
            && !ctx.chars.get(punct.end).is_some_and(|c| c.is_ascii_digit())
            && !(punct.end + 1 < ctx.chars.len()
                && ctx.chars[punct.end] == ' '
                && ctx.chars[punct.end + 1].is_ascii_digit());
        if !is_percent_after_digit && !needs_space_after(ctx.chars[punct.start]) {
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
            let after_dot = punct.len() > 1 && ctx.chars.get(punct.start + 1) == Some(&'.');
            if before_space || after_dot {
                continue;
            }
        }
        // Emoticon ASCII (":)", ":(", ";)", ":D", ":v", ":P", ":3"): không báo thừa space trước `:`.
        if matches!(ctx.chars[punct.start], ':' | ';') {
            if punct.len() > 1 {
                continue;
            }
            if let Some(next_tok) = ctx.tokens.get(i + 1) {
                if next_tok.start == punct.end
                    && next_tok.len() == 1
                    && matches!(
                        ctx.chars[next_tok.start],
                        'v' | 'V' | 'd' | 'D' | 'p' | 'P' | '3' | 'o' | 'O'
                    )
                {
                    continue;
                }
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
