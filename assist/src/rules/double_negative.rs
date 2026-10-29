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
        let Some(&next_w) = words.get(pos + 1) else {
            break;
        };
        if ctx.tokens[w].len() != 5 || ctx.tokens[next_w].len() != 2 {
            continue;
        }
        let a = ctx.token_text(&ctx.tokens[w]);
        let b = ctx.token_text(&ctx.tokens[next_w]);
        if a.to_lowercase() != "không" || b.to_lowercase() != "có" {
            continue;
        }
        // tim "khong" thu hai trong toi da 4 tu ke
        for &n in words.iter().skip(pos + 2).take(MAX_GAP_WORDS + 1) {
            if ctx.tokens[n].len() == 5
                && !ctx.is_masked(ctx.tokens[w].start, ctx.tokens[n].end)
                && ctx.token_text(&ctx.tokens[n]).to_lowercase() == "không"
            {
                // Vòng 39 (R1 FP 19-22) + vòng 40 (R2-T3): "không" thứ hai là
                // TỪ CUỐI câu hỏi ⇒ trợ từ hỏi tự nhiên ("Không có ai không?",
                // "Không có size 42 không ạ?") — KHÔNG phải phủ định kép.
                // R2-T3: chat/IM hỏi bằng dấu CHẤM + particle ("Không có size
                // 42 không ạ.") rất phổ biến — particle NGAY SAU "không" thứ
                // hai ("ạ", "nhé", "chứ", "nhỉ", "ha", "à", "nhá") miễn bất kể
                // dấu kết câu. Không particle thì cần "?" ("Tôi không có gì
                // để nói không." — corpus anchor — vẫn lỗi; "Không có ai
                // không biết quy định này." không cuối câu — vẫn lỗi).
                const PARTICLES: &[&str] = &["ạ", "nhé", "chứ", "nhỉ", "ha", "à", "nhá"];
                let is_question = ctx
                    .chars
                    .iter()
                    .rev()
                    .find(|c| !c.is_whitespace())
                    .is_some_and(|c| *c == '?');
                // Chỉ xét các từ SAU "không" thứ hai (từ trước n là phần "không có…").
                let after_n: Vec<&usize> = words
                    .iter()
                    .rev()
                    .take_while(|&&k| ctx.tokens[k].start > ctx.tokens[n].end)
                    .collect();
                let particle_after = !after_n.is_empty()
                    && after_n.iter().all(|&&k| {
                        let w = ctx.token_text(&ctx.tokens[k]).to_lowercase();
                        PARTICLES.contains(&w.as_str())
                    });
                if particle_after
                    || (is_question
                        && words.iter().rev().all(|&k| {
                            ctx.tokens[k].start < ctx.tokens[n].end
                                || PARTICLES.contains(
                                    &ctx.token_text(&ctx.tokens[k]).to_lowercase().as_str(),
                                )
                        }))
                {
                    continue;
                }
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
