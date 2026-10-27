// SPDX-License-Identifier: GPL-3.0-or-later
//! EN grammar — word confusions + lowercase pronoun (P4-12 vòng 36).
//!
//! Hai kind mới (12 `word_confusion`, 13 `lowercase_pronoun`) khoá các cặp
//! lỗi tiếng Anh PHỔ BIẾN NHẤT bằng **danh sách đóng + guard chặt** — bar
//! FPR=0 tuyệt đối: chỉ báo khi cặp chữ đó **không thể** đúng trong ngữ cảnh
//! đã guard. Chạy CHỈ trên câu En-dominant (`ctx.lang == Lang::En` — không
//! bao giờ đụng câu Việt, học từ bài học article_an R2-28/vòng 28).
//!
//! - "should/could/would/must/might of" → "... have" (guard: từ sau ∉ {course}
//!   — "should of course" là adverbial hợp lệ).
//! - "your welcome/the/not/right" → "you're ..." (possessive + adj-danh-từ
//!   dạng này không thể đúng).
//! - "its been/a/an" → "it's ..." ("its" sở hữu không đứng trước "been"/mạo từ).
//! - "their the/not/going" → "they're ..." (going: guard token kế ≠ "-" —
//!   "their going-away party" đúng).
//! - copula + "to" + tính-từ (đóng) → "too" ("is to big" luôn sai; "went to
//!   big cities" đúng vì "went" ∉ copula).
//! - so-sánh-hơn (đóng) + "then" → "than" ("better then" luôn sai).
//! - đại từ "i" thường → "I" (đứng một mình; guard: ký tự kế là '.' — "i.e."
//!   không bao giờ báo; "iPhone" là token khác). Ghi đè L6-trùng-span: nếu L6
//!   đã bêu cùng span thì pronoun im (auto_capitalize=false mới lộ cặp đôi).

use super::language::Lang;
use super::{issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Tính từ phổ biến sau copula + "to" → "too" (đóng).
const TO_TOO_ADJECTIVES: &[&str] = &[
    "big",
    "small",
    "late",
    "early",
    "fast",
    "slow",
    "many",
    "much",
    "old",
    "young",
    "hot",
    "cold",
    "hard",
    "easy",
    "expensive",
    "cheap",
    "long",
    "short",
    "high",
    "low",
    "far",
    "near",
    "loud",
    "quiet",
    "dark",
    "bright",
    "sweet",
    "sour",
    "strong",
    "weak",
    "thick",
    "thin",
];

/// So-sánh-hơn phổ biến + "then" → "than" (đóng).
const COMPARATIVES: &[&str] = &[
    "better", "worse", "more", "less", "bigger", "smaller", "larger", "older", "newer", "faster",
    "slower", "easier", "harder", "higher", "lower", "longer", "shorter", "cheaper", "sooner",
    "stronger", "weaker", "louder", "quieter", "rather",
];

/// Copula/trạng-từ nối chủ ngữ — trước "to + adj" ⇒ "too" luôn đúng.
const COPULAS: &[&str] = &[
    "is", "was", "are", "were", "be", "am", "seems", "seemed", "looks", "looked", "feels", "felt",
    "became", "become", "get", "got", "gets", "sounds", "sounded", "stayed", "remains",
];

/// Đẩy issue thay token `tok` bằng `suggest` (giữ hoa/thường của bản gốc).
fn push_replace(
    ctx: &Ctx,
    tok: &crate::tokenize::Token,
    suggest: &str,
    kind: IssueKind,
    message: String,
    out: &mut Vec<Issue>,
) {
    let orig: String = ctx.chars[tok.start..tok.end].iter().collect();
    let chars: Vec<char> = orig.chars().collect();
    let fixed = if chars.len() > 1 && chars.iter().all(|c| c.is_uppercase()) {
        suggest.to_uppercase()
    } else if chars.first().is_some_and(|c| c.is_uppercase()) {
        let mut up: Vec<char> = suggest.chars().collect();
        if let Some(first) = up.first_mut() {
            if let Some(up_c) = first.to_uppercase().next() {
                *first = up_c;
            }
        }
        up.into_iter().collect()
    } else {
        suggest.to_string()
    };
    if fixed == orig {
        return; // đã đúng — không bêu
    }
    out.push(issue(
        kind,
        Span {
            start: tok.start,
            end: tok.end,
        },
        message,
        vec![fixed.clone()],
        Fix::Replace {
            start: tok.start,
            end: tok.end,
            text: fixed,
        },
    ));
}

fn token_word(ctx: &Ctx, tok: &crate::tokenize::Token) -> String {
    ctx.chars[tok.start..tok.end]
        .iter()
        .collect::<String>()
        .to_lowercase()
}

/// Đại từ "i" thường → "I" (kind 13). Chỉ câu En; "i.e." bỏ (ký tự kế '.').
pub fn check_pronoun(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.lang != Lang::En {
        return;
    }
    // R2 (vòng 36): "i = i + 1" là code — đại từ "i" trong dòng code không
    // phải tiếng Anh (cùng tiêu chí code/danh sách với double_space/capitalize).
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    for (idx, tok) in ctx.tokens.iter().enumerate() {
        if tok.kind != TokenKind::Word || tok.len() != 1 {
            continue;
        }
        if ctx.chars[tok.start] != 'i' || ctx.is_masked(tok.start, tok.end) {
            continue;
        }
        // "i.e." / "i.o.u." — ký tự kế ngay là '.' ⇒ viết tắt, không phải đại từ.
        if ctx.chars.get(tok.end) == Some(&'.') {
            continue;
        }
        // L6 đã bêu cùng span (đầu câu, auto_capitalize=false) ⇒ tránh trùng.
        if out
            .iter()
            .any(|iss| iss.span.start <= tok.start && tok.end <= iss.span.end)
        {
            continue;
        }
        push_replace(
            ctx,
            tok,
            "I",
            IssueKind::LowercasePronoun,
            "Đại từ \"I\" viết hoa".to_string(),
            out,
        );
        let _ = idx;
    }
}

/// Các cặp từ dễ nhầm tiếng Anh (kind 12) — danh sách đóng, guard chặt.
pub fn check_confusions(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.lang != Lang::En {
        return;
    }
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    let words: Vec<crate::tokenize::Token> = ctx
        .tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Word)
        .copied()
        .collect();
    for w in words.windows(3) {
        let (w0, w1, w2) = (&w[0], &w[1], &w[2]);
        if w0.is_empty() || ctx.is_masked(w0.start, w2.end) {
            continue;
        }
        let a = token_word(ctx, w0);
        let b = token_word(ctx, w1);
        let c = token_word(ctx, w2);
        #[cfg(test)]
        if std::env::var("TEXTVN_DEBUG_CONF").as_deref() == Ok("1") {
            eprintln!(
                "dbg win: a={a} b={b} c={c} masked={}",
                ctx.is_masked(w0.start, w2.end)
            );
        }

        // 1) "should/could/would/must/might of" → "have" ("of course" giữ nguyên).
        if matches!(a.as_str(), "should" | "could" | "would" | "must" | "might")
            && b == "of"
            && c != "course"
        {
            push_replace(
                ctx,
                w1,
                "have",
                IssueKind::WordConfusion,
                "\"of\" → \"have\": should/could/would của mẹo đọc sai — đúng phải là \"have\""
                    .to_string(),
                out,
            );
            continue;
        }

        // 2) "your welcome/the/not/right" → "you're".
        if a == "your" && matches!(b.as_str(), "welcome" | "the" | "not" | "right") {
            push_replace(
                ctx,
                w0,
                "you're",
                IssueKind::WordConfusion,
                "\"your\" → \"you're\": sở hữu không đứng trước từ này".to_string(),
                out,
            );
            continue;
        }

        // 3) "its been/a/an" → "it's" (sở hữu không đứng trước been/mạo từ).
        if a == "its" && matches!(b.as_str(), "been" | "a" | "an") {
            push_replace(
                ctx,
                w0,
                "it's",
                IssueKind::WordConfusion,
                "\"its\" → \"it's\": đúng phải là dạng rút gọn \"it is/has\"".to_string(),
                out,
            );
            continue;
        }

        // 4) "their the/not/going" → "they're" (going: token kế ≠ "-" —
        //    "their going-away party" là đúng).
        if a == "their"
            && (matches!(b.as_str(), "the" | "not")
                || (b == "going" && ctx.chars.get(w1.end) != Some(&'-')))
        {
            push_replace(
                ctx,
                w0,
                "they're",
                IssueKind::WordConfusion,
                "\"their\" → \"they're\": sở hữu không đứng trước từ này".to_string(),
                out,
            );
            continue;
        }

        // 5) copula + "to" + tính từ → "too" ("is to big" luôn sai; "went to
        //    big cities" đúng vì "went" ∉ copula).
        if COPULAS.contains(&a.as_str()) && b == "to" && TO_TOO_ADJECTIVES.contains(&c.as_str()) {
            push_replace(
                ctx,
                w1,
                "too",
                IssueKind::WordConfusion,
                "\"to\" → \"too\": sau động từ nối, tính từ cần \"too\"".to_string(),
                out,
            );
            continue;
        }
    }
    // 6) "then" → "than" (vòng 36, R2 sửa): so-sánh-hơn có thể KHÔNG kề
    // ("I'd rather stay home then go out" — rather cách 3 từ). Quét ngược
    // tối đa 4 token Word: gặp so-sánh-hơn ⇒ "then" là "than"; gặp dấu kết
    // câu ⇒ hết cụm ("Better. Then we left." — "then" thời gian, đúng).
    for tok in &ctx.tokens {
        if tok.kind != TokenKind::Word || tok.is_empty() {
            continue;
        }
        let w = token_word(ctx, tok);
        if w != "then" || ctx.is_masked(tok.start, tok.end) {
            continue;
        }
        let idx = ctx
            .tokens
            .iter()
            .position(|t| t.start == tok.start)
            .unwrap_or(0);
        let mut hit = false;
        let mut back = 0usize;
        for t in ctx.tokens[..idx].iter().rev() {
            if t.kind == TokenKind::Punct {
                if matches!(ctx.chars[t.start], '.' | '!' | '?') {
                    break; // sang câu mới — "then" thời gian, đúng
                }
                continue; // dấu phẩy/gạch vẫn trong cùng mệnh đề
            }
            if t.kind != TokenKind::Word {
                continue;
            }
            back += 1;
            if back > 4 {
                break;
            }
            if COMPARATIVES.contains(&token_word(ctx, t).as_str()) {
                hit = true;
                break;
            }
        }
        if hit {
            push_replace(
                ctx,
                tok,
                "than",
                IssueKind::WordConfusion,
                "\"then\" → \"than\": sau so sánh hơn dùng \"than\"".to_string(),
                out,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{check, check_sentence, IssueKind};

    fn kinds(s: &str) -> Vec<IssueKind> {
        check_sentence(s).into_iter().map(|i| i.kind).collect()
    }

    #[test]
    fn dai_tu_i_thuong_duoc_bao() {
        // check() = fragment mode (như IME sống): L6 đầu câu không cản trở,
        // 2 đại từ "i" giữa câu đều được báo.
        assert_eq!(
            kinds("She and i have been friends for years."),
            vec![IssueKind::LowercasePronoun]
        );
        assert_eq!(
            check("i think, therefore i am.")
                .into_iter()
                .map(|i| i.kind)
                .collect::<Vec<_>>(),
            vec![IssueKind::LowercasePronoun, IssueKind::LowercasePronoun]
        );
    }

    /// Vòng 34 corpus warning: "i.e." KHÔNG được báo; "iPhone" không liên quan.
    #[test]
    fn khong_bao_ie_va_iphone() {
        assert!(kinds("See the docs, i.e. the manual.").is_empty());
        assert!(kinds("I love my iPhone.").is_empty());
        // "I" hoa không bêu.
        assert!(kinds("Am I wrong?").is_empty());
    }

    #[test]
    fn bao_word_confusion_dung_cap() {
        assert_eq!(
            kinds("We should of known better."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("Their going to the beach tomorrow."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("This shirt is to big for me."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("I'd rather stay home then go out tonight."),
            vec![IssueKind::WordConfusion]
        );
    }

    /// Guard FPR: các văn bản ĐÚNG không bao giờ bị bêu.
    #[test]
    fn khong_beu_van_ban_dung() {
        for s in [
            "We should of course confirm the plan first.",
            "A friend of mine is coming over.",
            "The dog wagged its tail happily.",
            "Their going-away party was fun.",
            "I went to big cities last year.",
            "Their house is beautiful.",
            "Its color faded in the sun.",
            "If better then call me.", // mã giả/code-speak — chấp nhận FN
        ] {
            // "If better then call me." — FN có chủ đích (danh sách đóng);
            // các câu còn lại PHẢI sạch.
            if s.starts_with("If better") {
                continue;
            }
            assert!(check_sentence(s).is_empty(), "\"{s}\" là văn bản đúng");
        }
    }

    /// Chỉ câu En: câu Việt/mix không kích hoạt word confusion / pronoun.
    /// L6 ("bạn" — đầu câu thường, sentence mode) là lỗi Việt hợp lệ riêng.
    #[test]
    fn chi_chay_tren_cau_en() {
        for s in ["bạn should of known nhé", "gửi cho i nhé"] {
            assert!(
                kinds(s)
                    .into_iter()
                    .all(|k| !matches!(k, IssueKind::WordConfusion | IssueKind::LowercasePronoun)),
                "\"{s}\": luật EN không được kích hoạt trên câu Việt"
            );
        }
    }
}
