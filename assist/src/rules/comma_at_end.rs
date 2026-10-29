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
    if ctx.chars[tok.start] != ',' || ctx.is_masked(tok.start, tok.end) {
        return;
    }
    // Vòng 34 (audit R1-F2): ký kết email chuẩn tiếng Việt ("Trân trọng,",
    // "Cảm ơn anh,", "Thân,"…) KHÔNG phải lỗi — câu ≤ 3 từ và từ trước dấu
    // phẩy nằm trong danh sách đóng ⇒ im lặng (corpus không có case này —
    // điểm mù phát hiện khi probe).
    {
        let words: Vec<&crate::tokenize::Token> = ctx
            .tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .collect();
        // BUG-1 (R2): words CÓ THỂ RỖNG ("10," — chữ số là token Other, không
        // phải Word) ⇒ words[0] index out of bounds → panic exit 101 (corpus
        // score_real có dạng này chết cả bộ đo). Chỉ chạy khi 1..=3 từ.
        // Vòng 39 (R1 văn phòng): trần 3→6 từ — "Trân trọng cảm ơn,",
        // "Rất mong sớm nhận được phản hồi của Quý công ty," đều ≥4 từ.
        if (1..=6).contains(&words.len()) {
            // Vòng 34 (R1-F2, siết lại sau probe): KÝ KẾT EMAIL = TỪ ĐẦU câu
            // thuộc danh sách đóng ("Cảm ơn anh,", "Trân trọng,", "Thân,").
            // "Tôi gặp anh," (lỗi thật — dấu phẩy cuối câu treo) vẫn được bêu
            // vì từ đầu "tôi" ∉ danh sách.
            const SIGN_OFF_OPENERS: &[&str] = &[
                // Gồm dạng TỪ ĐẦU của sign-off 2 từ ("trân trọng" → "trân",
                // "cảm ơn" → "cảm", "xin chào" → "xin" — BUG-2 R2: entry 2-từ
                // dead vì so khớp chỉ đọc token Word ĐẦU). Câu ≤ 3 từ nên
                // không có FP thực.
                "trân trọng",
                "trân",
                "thân",
                "thân ái",
                "cảm ơn",
                "cảm",
                "thanks",
                "regards",
                "best",
                "cheers",
                "hello",
                "hi",
                "xin chào",
                "xin",
                "chào",
                // Vòng 39 (R1 văn phòng): opener email/lời chào thật —
                // "Kính gửi Quý công ty,", "Gửi anh,", "Thưa anh,", "Dear Sir,"...
                "kính gửi",
                "kính",
                "gửi",
                "thưa",
                "kính thưa",
                "chúc",
                "mong",
                "rất mong",
                "dear",
            ];
            let first = ctx.chars[words[0].start..words[0].end]
                .iter()
                .collect::<String>()
                .to_lowercase();
            if SIGN_OFF_OPENERS.contains(&first.as_str()) {
                return;
            }
        }
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
