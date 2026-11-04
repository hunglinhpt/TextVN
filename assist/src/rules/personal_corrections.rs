// SPDX-License-Identifier: GPL-3.0-or-later
//! P7 (vòng 54) — **correction cá nhân** (`style_profile.corrections`): cặp
//! sai→đúng user ĐÃ CHẤP NHẬN sửa. Tra TRƯỚC bảng tĩnh `collocation_typo` —
//! cặp cá nhân thắng; still suggestion+fix shape giống DiacriticTypo nên
//! apply/undo/precondition hoạt động ngay.
//!
//! FPR bar (P7-DFMEA F8): khớp NGUYÊN VẸN token (không fuzzy); đi qua
//! `drop_issues_in_protected` như mọi luật; toàn bộ cặp có nguồn gốc
//! tường minh (user click Apply) — không suy đoán.

use super::{is_word, Ctx};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Cặp correction cho 1 từ ("xử dụng" → "sử dụng").
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.personal_corrections.is_empty() {
        return;
    }
    for tok in ctx.tokens.iter() {
        if tok.kind != TokenKind::Word || tok.is_empty() {
            continue;
        }
        let text = ctx.token_text(tok);
        let lower = text.to_lowercase();
        let Some(to) = lookup(ctx.personal_corrections, &lower) else {
            continue;
        };
        let start = tok.start;
        let end = tok.end;
        if ctx.is_masked(start, end) || ctx.is_protected(start, end) {
            continue;
        }
        if ctx.quote_parity.get(start).copied().unwrap_or(false) {
            continue;
        }
        // Giữ hoa đầu từ theo mẫu gõ ("Xử dụng" → "Sử dụng").
        let suggestion = preserve_case(&text, to);
        out.push(Issue {
            kind: IssueKind::DiacriticTypo,
            span: Span { start, end },
            message: format!("Cá nhân của bạn: \"{text}\" → \"{to}\""),
            suggestions: vec![suggestion],
            auto_fixable: true,
            confidence: 95,
            fix: Some(crate::Fix::Replace {
                start,
                end,
                text: preserve_case(&text, to),
            }),
        });
    }
    // Cặp 2 từ ("chuyến bay bay" → "chuyến bay" dạng from 2 từ): khớp bigram.
    if ctx.tokens.len() < 3 {
        return;
    }
    for i in 2..ctx.tokens.len() {
        let (t1, gap, t0) = (&ctx.tokens[i], &ctx.tokens[i - 1], &ctx.tokens[i - 2]);
        if !is_word(t1) || !is_word(t0) || t0.is_empty() || t1.is_empty() {
            continue;
        }
        if gap.kind != TokenKind::Space || gap.len() != 1 || ctx.chars[gap.start] != ' ' {
            continue;
        }
        let bigram = format!(
            "{} {}",
            ctx.token_text(t0).to_lowercase(),
            ctx.token_text(t1).to_lowercase()
        );
        let Some(to) = lookup(ctx.personal_corrections, &bigram) else {
            continue;
        };
        let start = t0.start;
        let end = t1.end;
        if ctx.is_masked(start, end) || ctx.is_protected(start, end) {
            continue;
        }
        let first = ctx.token_text(t0);
        out.push(Issue {
            kind: IssueKind::DiacriticTypo,
            span: Span { start, end },
            message: format!("Cá nhân của bạn: \"{bigram}\" → \"{to}\""),
            suggestions: vec![preserve_case(&first, to)],
            auto_fixable: true,
            confidence: 95,
            fix: Some(crate::Fix::Replace {
                start,
                end,
                text: preserve_case(&first, to),
            }),
        });
    }
}

fn lookup<'a>(corrections: &'a [(String, String)], needle: &str) -> Option<&'a str> {
    corrections
        .iter()
        .find(|(f, _)| f.as_str() == needle)
        .map(|(_, t)| t.as_str())
}

/// Giữ hoa chữ cái đầu của mẫu gõ ("Xử" → "Sử"); mẫu HOA toàn phần → HOA.
fn preserve_case(sample: &str, target: &str) -> String {
    let mut chars = sample.chars();
    let Some(first) = chars.next() else {
        return target.to_string();
    };
    if sample
        .chars()
        .skip(1)
        .all(|c| !c.is_alphabetic() || c.is_uppercase())
        && first.is_uppercase()
        && target.chars().filter(|c| c.is_alphabetic()).count() > 1
        && sample
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
    {
        return target.to_uppercase();
    }
    if first.is_uppercase() {
        let mut t = target.chars();
        if let Some(tf) = t.next() {
            let mut out = tf.to_uppercase().collect::<String>();
            out.extend(t);
            return out;
        }
    }
    target.to_string()
}
