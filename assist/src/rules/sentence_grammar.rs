// SPDX-License-Identifier: GPL-3.0-or-later
//! Kiểm tra ngữ pháp câu tiếng Việt và tiếng Anh (P4-14).
//!
//! 1. Tiếng Việt:
//!    - Cặp liên từ / quan hệ từ hô ứng khập khiễng:
//!      + "Mặc dù... nên..." → "Mặc dù... nhưng..."
//!      + "Vì/Do/Bởi vì... nhưng..." → "Vì... nên..."
//!      + "Không những/Không chỉ... nhưng..." → "Không những... mà còn..."
//!      + "Nếu/Hễ/Giá như... nhưng..." → "Nếu... thì..."
//!      + Thừa liên từ tương phản: "Tuy nhiên nhưng..." → xoá "nhưng"
//!
//! 2. Tiếng Anh:
//!    - Cặp liên từ khập khiễng (Correlative Conjunction Errors):
//!      + "Although / Even though / Though... but..." → xoá "but"
//!      + "Because / Since / As... so..." → xoá "so"
//!    - Hoà hợp chủ ngữ - vị ngữ phổ biến (Subject-Verb Agreement):
//!      + "He / She / It" + bare verb ("go" → "goes", "do" → "does", "have" → "has", "don't" → "doesn't")
//!      + "They / We / You" + singular verb ("is" → "are", "was" → "were", "has" → "have", "does" → "do", "doesn't" → "don't")
//!      + "I" + "is/are" → "am"
//!    - Giới từ thừa & cụm từ kết hợp (Redundant Prepositions & Collocations):
//!      + "discuss about" → "discuss" (xoá "about")
//!      + "return back" / "revert back" / "reply back" → xoá "back"
//!      + "repeat again" → xoá "again"
//!      + "cope up with" → xoá "up"
//!      + "look forward to hear/meet/see" → "... hearing/meeting/seeing"
//!      + "congratulate for" → "congratulate on"
//!      + "pay attention on" → "pay attention to"
//!      + "responsible of" → "responsible for"
//!      + "interested on" → "interested in"
//!      + "depend of" → "depend on"

use super::language::Lang;
use super::{issue, Ctx};
use crate::tokenize::{Token, TokenKind};
use crate::{Fix, Issue, IssueKind, Span};

/// Giữ chữ hoa đầu hoặc hoa toàn bộ của bản gốc khi thay thế.
fn preserve_case(orig: &str, replacement: &str) -> String {
    let chars: Vec<char> = orig.chars().collect();
    if chars.len() > 1 && chars.iter().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if chars.first().is_some_and(|c| c.is_uppercase()) {
        let mut out: Vec<char> = replacement.chars().collect();
        if let Some(first) = out.first_mut() {
            if let Some(up) = first.to_uppercase().next() {
                *first = up;
            }
        }
        return out.into_iter().collect();
    }
    replacement.to_string()
}

/// Đẩy issue thay thế đoạn `[start, end)` bằng `replacement`.
fn push_replace_span(
    start: usize,
    end: usize,
    orig: &str,
    replacement: &str,
    msg: String,
    out: &mut Vec<Issue>,
) {
    if out
        .iter()
        .any(|iss| iss.span.start < end && start < iss.span.end)
    {
        return;
    }
    let fixed = preserve_case(orig, replacement);
    out.push(issue(
        IssueKind::WordConfusion,
        Span { start, end },
        msg,
        vec![fixed.clone()],
        Fix::Replace {
            start,
            end,
            text: fixed,
        },
    ));
}

/// Đẩy issue thay thế từ `tok` bằng `replacement`.
fn push_replace_issue(
    tok: &Token,
    orig: &str,
    replacement: &str,
    msg: String,
    out: &mut Vec<Issue>,
) {
    push_replace_span(tok.start, tok.end, orig, replacement, msg, out);
}

/// Đẩy issue xoá từ `tok` kèm khoảng trắng đứng trước `gap`.
fn push_delete_issue(gap: &Token, tok: &Token, msg: String, out: &mut Vec<Issue>) {
    if out
        .iter()
        .any(|iss| iss.span.start < tok.end && tok.start < iss.span.end)
    {
        return;
    }
    out.push(issue(
        IssueKind::WordConfusion,
        Span {
            start: tok.start,
            end: tok.end,
        },
        msg,
        Vec::new(),
        Fix::Delete {
            start: gap.start,
            end: tok.end,
        },
    ));
}

fn word_lower(ctx: &Ctx, tok: &Token) -> String {
    ctx.token_text(tok).to_lowercase()
}

// ---------------------------------------------------------------------------
// 1. TIẾNG VIỆT — NGỮ PHÁP CÂU & LIÊN TỪ HÔ ỨNG
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViCorrelativeClause {
    Contrast,    // Mặc dù, tuy, dù, cho dù
    Cause,       // Vì, do, bởi vì, tại vì
    Progression, // Không những, không chỉ, chẳng những
    Condition,   // Nếu, hễ, giá như
}

pub fn check_vi(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.lang == Lang::En {
        return;
    }

    // 1.1 Thừa liên từ liền kề: "tuy nhiên nhưng"
    for i in 2..ctx.tokens.len() {
        let (t0, gap, t1) = (ctx.tokens[i - 2], ctx.tokens[i - 1], ctx.tokens[i]);
        if t0.kind == TokenKind::Word
            && gap.kind == TokenKind::Space
            && t1.kind == TokenKind::Word
            && !ctx.is_masked(t0.start, t1.end)
        {
            let w0 = word_lower(ctx, &t0);
            let w1 = word_lower(ctx, &t1);
            if i >= 4 {
                let t_prev = ctx.tokens[i - 4];
                let w_prev = word_lower(ctx, &t_prev);
                if w_prev == "tuy" && w0 == "nhiên" && w1 == "nhưng" {
                    push_delete_issue(
                        &gap,
                        &t1,
                        "Thừa liên từ: \"Tuy nhiên\" và \"nhưng\" đều chỉ sự tương phản, không đi liền nhau".to_string(),
                        out,
                    );
                    continue;
                }
            }
        }
    }

    // 1.2 Cặp liên từ hô ứng hai vế câu
    let mut clause_type: Option<(ViCorrelativeClause, usize)> = None;
    let mut words_with_idx: Vec<(usize, Token, String)> = Vec::new();
    for (i, tok) in ctx.tokens.iter().enumerate() {
        if tok.kind == TokenKind::Word {
            words_with_idx.push((i, *tok, word_lower(ctx, tok)));
        }
    }

    let n = words_with_idx.len();
    for k in 0..n {
        let (tok_idx, tok, ref w) = words_with_idx[k];
        if ctx.is_masked(tok.start, tok.end) {
            continue;
        }

        // Nếu gặp dấu kết câu giữa chừng (. ! ? ;), huỷ vế 1 vì đã sang câu khác
        if let Some((_, marker_idx)) = clause_type {
            let has_sentence_break = ctx.tokens[marker_idx..tok_idx].iter().any(|t| {
                t.kind == TokenKind::Punct && matches!(ctx.chars[t.start], '.' | '!' | '?' | ';')
            });
            if has_sentence_break {
                clause_type = None;
            }
        }

        // Nhận diện vế 1 (ở nửa đầu câu hoặc trước dấu phẩy):
        if clause_type.is_none() && k < n / 2 + 2 {
            let next_w = if k + 1 < n {
                words_with_idx[k + 1].2.as_str()
            } else {
                ""
            };
            if (w == "mặc" && next_w == "dù")
                || (w == "cho" && next_w == "dù")
                || w == "tuy"
                || w == "dù"
            {
                clause_type = Some((ViCorrelativeClause::Contrast, tok_idx));
                continue;
            }
            if (w == "bởi" && next_w == "vì")
                || (w == "tại" && next_w == "vì")
                || w == "vì"
                || w == "do"
            {
                clause_type = Some((ViCorrelativeClause::Cause, tok_idx));
                continue;
            }
            if (w == "không" && next_w == "những")
                || (w == "không" && next_w == "chỉ")
                || (w == "chẳng" && next_w == "những")
            {
                clause_type = Some((ViCorrelativeClause::Progression, tok_idx));
                continue;
            }
            if (w == "giá" && next_w == "như") || w == "nếu" || w == "hễ" {
                clause_type = Some((ViCorrelativeClause::Condition, tok_idx));
                continue;
            }
        }

        // Kiểm tra xung đột ở vế 2 (sau vế 1 ít nhất 2 từ):
        if let Some((clause, marker_idx)) = clause_type {
            // Vòng 39 (R1 văn phòng FP 7-15): comma GIỮA marker và từ xung đột
            // ⇒ mệnh đề vế 1 đã đóng bằng dấu phẩy (trạng ngữ/tác nhân), cặp
            // quan hệ từ không áp — mọi FP đều có phẩy, lỗi thật không.
            let comma_between = ctx.tokens[marker_idx + 1..tok_idx]
                .iter()
                .any(|t| t.kind == TokenKind::Punct && ctx.chars[t.start] == ',');
            if tok_idx > marker_idx + 3 {
                match clause {
                    ViCorrelativeClause::Contrast => {
                        if comma_between {
                            break;
                        }
                        // "Mặc dù... cho nên..." hoặc "Mặc dù... nên..." → sai!
                        let is_cho_nen =
                            w == "cho" && k + 1 < n && words_with_idx[k + 1].2 == "nên";
                        if is_cho_nen {
                            let tok_nen = words_with_idx[k + 1].1;
                            let orig: String = ctx.chars[tok.start..tok_nen.end].iter().collect();
                            push_replace_span(
                                tok.start,
                                tok_nen.end,
                                &orig,
                                "nhưng",
                                "Cặp quan hệ từ khập khiễng: \"Mặc dù/Tuy\" biểu thị tương phản, cần đi với \"nhưng\", không đi với \"cho nên\"".to_string(),
                                out,
                            );
                            break;
                        } else if w == "nên" && (k == 0 || words_with_idx[k - 1].2 != "cho") {
                            let orig = ctx.token_text(&tok);
                            push_replace_issue(
                                &tok,
                                &orig,
                                "nhưng",
                                "Cặp quan hệ từ khập khiễng: \"Mặc dù/Tuy\" biểu thị tương phản, cần đi với \"nhưng\", không đi với \"nên\"".to_string(),
                                out,
                            );
                            break;
                        }
                    }
                    ViCorrelativeClause::Cause => {
                        // Vòng 39 (R1 FP 8-13): CHỈ bêu khi marker ĐỨNG ĐẦU
                        // câu và KHÔNG có phẩy trước "nhưng". FP: "Báo cáo này
                        // do phòng kế toán lập, nhưng..." ("do" = tác nhân bị
                        // động), "Do đó, ..." (anaphora), "Vì sao ..., nhưng..."
                        // (nghi vấn) — đều có phẩy hoặc marker giữa câu.
                        let marker_at_start = marker_idx == 0
                            || ctx.tokens[..marker_idx]
                                .iter()
                                .all(|t| t.kind != TokenKind::Word);
                        let next_w2 = if k + 1 < n {
                            words_with_idx[k + 1].2.as_str()
                        } else {
                            ""
                        };
                        // "Vì sao ... nhưng..." (nghi vấn) và "Do đó ... nhưng..."
                        // (anaphora) — marker không phải nguyên nhân thật.
                        let interrogative_or_anaphora =
                            next_w2 == "sao" || (w == "do" && next_w2 == "đó");
                        if w == "nhưng"
                            && (!marker_at_start || comma_between || interrogative_or_anaphora)
                        {
                            break;
                        }
                        // "Vì... nhưng..." → sai!
                        if w == "nhưng" {
                            let orig = ctx.token_text(&tok);
                            push_replace_issue(
                                &tok,
                                &orig,
                                "nên",
                                "Cặp quan hệ từ khập khiễng: \"Vì/Do\" biểu thị nguyên nhân, cần đi với \"nên\", không đi với \"nhưng\"".to_string(),
                                out,
                            );
                            break;
                        }
                    }
                    ViCorrelativeClause::Progression => {
                        // "Không những... nhưng..." → sai!
                        if w == "nhưng" {
                            let orig = ctx.token_text(&tok);
                            push_replace_issue(
                                &tok,
                                &orig,
                                "mà",
                                "Cặp quan hệ từ tăng tiến: \"Không những/không chỉ\" cần đi với \"mà còn\", không đi với \"nhưng\"".to_string(),
                                out,
                            );
                            break;
                        }
                    }
                    ViCorrelativeClause::Condition => {
                        // Vòng 39 (R1 FP 14-15): chỉ bêu khi mệnh đề "nhưng"
                        // là MỆNH ĐỀ CUỐI (sau nó không còn phẩy + mệnh đề
                        // chính). FP: "Nếu ... nhưng ..., anh vui lòng báo lại."
                        // — "nhưng" nằm trong vế điều kiện.
                        let comma_after = ctx.tokens[tok_idx + 1..]
                            .iter()
                            .any(|t| t.kind == TokenKind::Punct && ctx.chars[t.start] == ',');
                        if w == "nhưng" && comma_after {
                            break;
                        }
                        // "Nếu... nhưng..." → sai!
                        if w == "nhưng" {
                            let orig = ctx.token_text(&tok);
                            push_replace_issue(
                                &tok,
                                &orig,
                                "thì",
                                "Cặp quan hệ từ điều kiện: \"Nếu/Hễ\" cần đi với \"thì\", không đi với \"nhưng\"".to_string(),
                                out,
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. TIẾNG ANH — NGỮ PHÁP CÂU, CHỦ - VỊ & GIỚI TỪ THỪA
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnCorrelativeClause {
    Although,
    Because,
}

/// Vòng 37: gộp chuỗi Word + apostrophe + Word KỀ NHAU thành 1 token logic
/// ("don't", "it's", "they're"...) — tokenizer tách apostrophe thành Punct
/// làm SVA so khớp cặp Word kề nhau luột qua mọi câu có dạng rút gọn
/// ("He don't" — w1 = "don" khác "don't"). Chỉ gộp khi 3 token liền nhau
/// (không có space) và dấu là nháy đơn thường hoặc cong.
fn merge_contractions(ctx: &Ctx) -> Vec<Token> {
    let toks = &ctx.tokens;
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    let mut i = 0usize;
    while i < toks.len() {
        let t = toks[i];
        if t.kind == TokenKind::Word
            && i + 2 < toks.len()
            && t.end == toks[i + 1].start
            && toks[i + 1].kind == TokenKind::Punct
            && toks[i + 1].len() == 1
            && matches!(ctx.chars[toks[i + 1].start], '\u{27}' | '\u{2019}')
            && toks[i + 1].end == toks[i + 2].start
            && toks[i + 2].kind == TokenKind::Word
        {
            out.push(Token {
                kind: TokenKind::Word,
                start: t.start,
                end: toks[i + 2].end,
            });
            i += 3;
            continue;
        }
        out.push(t);
        i += 1;
    }
    out
}

pub fn check_en(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.lang != Lang::En {
        return;
    }
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }

    // Vòng 37: gộp dạng rút gọn ("don't" -> 1 token logic) trước khi so cặp.
    let words: Vec<Token> = merge_contractions(ctx)
        .into_iter()
        .filter(|t| t.kind == TokenKind::Word)
        .collect();

    // 2.1 Cặp liên từ hô ứng câu tiếng Anh (Although... but / Because... so)
    let mut en_clause: Option<EnCorrelativeClause> = None;
    if let Some(first_word) = words.first() {
        let w_first = word_lower(ctx, first_word);
        if w_first == "although"
            || w_first == "though"
            || (w_first == "even" && words.len() > 1 && word_lower(ctx, &words[1]) == "though")
        {
            en_clause = Some(EnCorrelativeClause::Although);
        } else if w_first == "because" || w_first == "since" {
            en_clause = Some(EnCorrelativeClause::Because);
        }
    }

    if let Some(clause) = en_clause {
        for (i, tok) in ctx.tokens.iter().enumerate() {
            if tok.kind != TokenKind::Word || ctx.is_masked(tok.start, tok.end) {
                continue;
            }
            if ctx.tokens[..i].iter().any(|t| {
                t.kind == TokenKind::Punct && matches!(ctx.chars[t.start], '.' | '!' | '?' | ';')
            }) {
                break;
            }
            let w = word_lower(ctx, tok);
            if i > 2 {
                let gap = ctx.tokens[i - 1];
                if clause == EnCorrelativeClause::Although && w == "but" {
                    push_delete_issue(
                        &gap,
                        tok,
                        "Redundant conjunction: do not use \"but\" in a sentence starting with \"Although/Even though\"".to_string(),
                        out,
                    );
                    break;
                } else if clause == EnCorrelativeClause::Because && w == "so" {
                    let next_word_is_subject = ctx.tokens[i + 1..]
                        .iter()
                        .find(|t| t.kind == TokenKind::Word)
                        .is_some_and(|t| {
                            matches!(
                                word_lower(ctx, t).as_str(),
                                "i" | "we" | "they" | "he" | "she" | "you" | "it" | "the"
                            )
                        });
                    if next_word_is_subject {
                        push_delete_issue(
                            &gap,
                            tok,
                            "Redundant conjunction: do not use \"so\" in a sentence starting with \"Because/Since\"".to_string(),
                            out,
                        );
                        break;
                    }
                }
            }
        }
    }

    // 2.2 Hoà hợp chủ ngữ - vị ngữ (Subject-Verb Agreement) & Redundant Prepositions
    const MODALS: &[&str] = &[
        "to", "can", "could", "will", "would", "should", "must", "may", "might", "does", "did",
        "shall", "cannot", "won't",
    ];

    for i in 0..words.len().saturating_sub(1) {
        let (w0, w1) = (words[i], words[i + 1]);
        if ctx.is_masked(w0.start, w1.end) {
            continue;
        }

        // Xác minh w0 và w1 cách nhau đúng 1 dấu cách đơn, không có dấu câu chen giữa
        let Some(gap_idx) = ctx.tokens.iter().position(|t| t.start == w0.end) else {
            continue;
        };
        let gap = ctx.tokens[gap_idx];
        if gap.kind != TokenKind::Space || gap.len() != 1 || gap.end != w1.start {
            continue;
        }
        let a = word_lower(ctx, &w0);
        let b = word_lower(ctx, &w1);
        let orig_b = ctx.token_text(&w1);

        let prev_is_modal = if i > 0 {
            let prev = word_lower(ctx, &words[i - 1]);
            MODALS.contains(&prev.as_str())
        } else {
            false
        };

        if !prev_is_modal {
            // He / She / It + base form
            if matches!(a.as_str(), "he" | "she" | "it") {
                let replacement = match b.as_str() {
                    "go" => Some("goes"),
                    "do" => Some("does"),
                    "have" => Some("has"),
                    "say" => Some("says"),
                    "want" => Some("wants"),
                    "need" => Some("needs"),
                    "like" => Some("likes"),
                    "make" => Some("makes"),
                    "get" => Some("gets"),
                    "think" => Some("thinks"),
                    "know" => Some("knows"),
                    _ => None,
                };
                if let Some(rep) = replacement {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        rep,
                        format!("Subject-verb agreement: \"{a}\" takes \"{rep}\""),
                        out,
                    );
                    continue;
                }
                if b == "don't" || b == "dont" {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        "doesn't",
                        format!("Subject-verb agreement: \"{a}\" takes \"doesn't\""),
                        out,
                    );
                    continue;
                }
            }

            // They / We / You + singular verb
            if matches!(a.as_str(), "they" | "we" | "you") {
                let replacement = match b.as_str() {
                    "is" => Some("are"),
                    "was" => Some("were"),
                    "has" => Some("have"),
                    "does" => Some("do"),
                    _ => None,
                };
                if let Some(rep) = replacement {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        rep,
                        format!("Subject-verb agreement: \"{a}\" takes \"{rep}\""),
                        out,
                    );
                    continue;
                }
                if b == "doesn't" || b == "doesnt" {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        "don't",
                        format!("Subject-verb agreement: \"{a}\" takes \"don't\""),
                        out,
                    );
                    continue;
                }
            }

            // I + is/are
            if a == "i" {
                if b == "is" || b == "are" {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        "am",
                        "Subject-verb agreement: \"I\" takes \"am\"".to_string(),
                        out,
                    );
                    continue;
                }
                if b == "has" {
                    push_replace_issue(
                        &w1,
                        &orig_b,
                        "have",
                        "Subject-verb agreement: \"I\" takes \"have\"".to_string(),
                        out,
                    );
                    continue;
                }
            }
        }

        // 2.3 Giới từ thừa: "discuss about", "return back", "revert back", "reply back", "repeat again"
        if let Some(gap_idx) = ctx.tokens.iter().position(|t| t.start == w0.end) {
            let gap = ctx.tokens[gap_idx];
            if gap.kind == TokenKind::Space {
                if (a == "discuss" || a == "discussed" || a == "discussing") && b == "about" {
                    push_delete_issue(
                        &gap,
                        &w1,
                        "Redundant preposition: \"discuss\" does not take \"about\"".to_string(),
                        out,
                    );
                    continue;
                }
                if (a == "return" || a == "returned") && b == "back" {
                    push_delete_issue(
                        &gap,
                        &w1,
                        "Redundant adverb: \"return\" already means to go or come back".to_string(),
                        out,
                    );
                    continue;
                }
                if (a == "revert" || a == "reverted") && b == "back" {
                    push_delete_issue(
                        &gap,
                        &w1,
                        "Redundant adverb: \"revert\" already means to reply or go back"
                            .to_string(),
                        out,
                    );
                    continue;
                }
                if (a == "reply" || a == "replied") && b == "back" {
                    push_delete_issue(
                        &gap,
                        &w1,
                        "Redundant adverb: \"reply\" already means to answer back".to_string(),
                        out,
                    );
                    continue;
                }
                if (a == "repeat" || a == "repeated") && b == "again" {
                    push_delete_issue(
                        &gap,
                        &w1,
                        "Redundant adverb: \"repeat\" already means to do or say again".to_string(),
                        out,
                    );
                    continue;
                }
            }
        }

        // 2.4 Cặp giới từ sai phổ biến: "responsible of" → "for", "interested on" → "in", "depend of" → "on"
        if a == "responsible" && b == "of" {
            push_replace_issue(
                &w1,
                &orig_b,
                "for",
                "Preposition error: \"responsible\" takes \"for\"".to_string(),
                out,
            );
            continue;
        }
        if a == "interested" && b == "on" {
            push_replace_issue(
                &w1,
                &orig_b,
                "in",
                "Preposition error: \"interested\" takes \"in\"".to_string(),
                out,
            );
            continue;
        }
        if (a == "depend" || a == "depends" || a == "depended") && b == "of" {
            push_replace_issue(
                &w1,
                &orig_b,
                "on",
                "Preposition error: \"depend\" takes \"on\"".to_string(),
                out,
            );
            continue;
        }
        if a == "independent" && b == "on" {
            push_replace_issue(
                &w1,
                &orig_b,
                "of",
                "Preposition error: \"independent\" takes \"of\"".to_string(),
                out,
            );
            continue;
        }
    }

    // 2.5 "look forward to" + V-ing ("hear" → "hearing", "meet" → "meeting", "see" → "seeing")
    for w in words.windows(4) {
        let (w0, w1, w2, w3) = (&w[0], &w[1], &w[2], &w[3]);
        if ctx.is_masked(w0.start, w3.end) {
            continue;
        }
        let a = word_lower(ctx, w0);
        let b = word_lower(ctx, w1);
        let c = word_lower(ctx, w2);
        let d = word_lower(ctx, w3);
        let orig_d = ctx.token_text(w3);

        if (a == "look" || a == "looking") && b == "forward" && c == "to" {
            let linked = (0..3).all(|k| {
                let (curr, next) = (w[k], w[k + 1]);
                ctx.tokens
                    .iter()
                    .position(|t| t.start == curr.end)
                    .is_some_and(|g_idx| {
                        let g = ctx.tokens[g_idx];
                        g.kind == TokenKind::Space && g.len() == 1 && g.end == next.start
                    })
            });
            if !linked {
                continue;
            }
            let rep = match d.as_str() {
                "hear" => Some("hearing"),
                "meet" => Some("meeting"),
                "see" => Some("seeing"),
                "work" => Some("working"),
                "receive" => Some("receiving"),
                _ => None,
            };
            if let Some(r) = rep {
                push_replace_issue(
                    w3,
                    &orig_d,
                    r,
                    format!("Grammar error: \"look forward to\" takes gerund \"{r}\""),
                    out,
                );
            }
        }
    }

    // 2.6 "cope up with" → "cope with" (delete "up")
    for w in words.windows(3) {
        let (w0, w1, w2) = (&w[0], &w[1], &w[2]);
        if ctx.is_masked(w0.start, w2.end) {
            continue;
        }
        let a = word_lower(ctx, w0);
        let b = word_lower(ctx, w1);
        let c = word_lower(ctx, w2);
        if a == "cope" && b == "up" && c == "with" {
            let linked = (0..2).all(|k| {
                let (curr, next) = (w[k], w[k + 1]);
                ctx.tokens
                    .iter()
                    .position(|t| t.start == curr.end)
                    .is_some_and(|g_idx| {
                        let g = ctx.tokens[g_idx];
                        g.kind == TokenKind::Space && g.len() == 1 && g.end == next.start
                    })
            });
            if !linked {
                continue;
            }
            if let Some(gap_idx) = ctx.tokens.iter().position(|t| t.start == w0.end) {
                let gap = ctx.tokens[gap_idx];
                push_delete_issue(
                    &gap,
                    w1,
                    "Redundant preposition: the correct idiom is \"cope with\", not \"cope up with\"".to_string(),
                    out,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn vi_correlative_mac_du_nen() {
        let issues = check("Mặc dù trời mưa nên chúng tôi vẫn đi học.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["nhưng"]);
    }

    #[test]
    fn vi_correlative_vi_nhung() {
        let issues = check("Vì thời tiết xấu nhưng chúng tôi hoãn chuyến đi.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["nên"]);
    }

    #[test]
    fn vi_tuy_nhien_nhung() {
        let issues = check("Kế hoạch đã lập xong. Tuy nhiên nhưng vẫn cần xem lại.");
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].fix, Some(Fix::Delete { .. })));
    }

    #[test]
    fn en_correlative_although_but() {
        let issues = check("Although it was raining, but we went to the office.");
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].fix, Some(Fix::Delete { .. })));
    }

    #[test]
    fn en_subject_verb_agreement() {
        let issues = check("He go to the office every day.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["goes"]);

        let issues = check("They is working on the project.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["are"]);
    }

    #[test]
    fn en_redundant_preposition() {
        let issues = check("We need to discuss about the budget.");
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].fix, Some(Fix::Delete { .. })));
    }

    #[test]
    fn en_look_forward_to_gerund() {
        let issues = check("I look forward to hear from you soon.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["hearing"]);
    }

    #[test]
    fn vi_correlative_mac_du_cho_nen() {
        let issues = check("Mặc dù trời mưa cho nên chúng tôi vẫn đi học.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions, vec!["nhưng"]);
    }

    #[test]
    fn vi_boundary_nen_as_modal_verb_khong_bao_loi() {
        // "nên" ở câu sau mang nghĩa khuyên bảo ("should") -> không báo lỗi
        let issues = check("Mặc dù trời mưa. Bạn nên mang theo ô.");
        assert!(
            issues.is_empty(),
            "Không được báo lỗi khi nên đứng ở câu sau: {issues:?}"
        );
    }

    #[test]
    fn en_so_adverb_and_punct_boundaries_khong_bao_loi() {
        // "so" là phó từ chỉ mức độ trước tính từ ("so happy") -> không báo lỗi
        let issues = check("Because it was cold, he was so happy with the warm tea.");
        assert!(
            issues.is_empty(),
            "so happy không phải liên từ thừa: {issues:?}"
        );

        // Dấu chấm hỏi / dấu phẩy ngăn cách giữa he và go -> không phải lỗi chủ vị
        let issues_q = check("Who is he? Go away.");
        assert!(
            issues_q.is_empty(),
            "dấu câu ngăn cách không báo he goes: {issues_q:?}"
        );
    }

    #[test]
    fn sentence_grammar_auto_fix_plan() {
        use crate::plan_sentence_fix;
        let cases = [
            (
                "Mặc dù trời mưa nên chúng tôi vẫn đi làm.",
                "Mặc dù trời mưa nhưng chúng tôi vẫn đi làm.",
            ),
            (
                "He go to the office every day.",
                "He goes to the office every day.",
            ),
            (
                "They is working on the project.",
                "They are working on the project.",
            ),
            (
                "We need to discuss about the budget.",
                "We need to discuss the budget.",
            ),
        ];

        for (raw, expected) in cases {
            let issues = check(raw);
            assert!(!issues.is_empty(), "Phải phát hiện lỗi trong: {raw}");
            let plan = plan_sentence_fix(raw, &issues).expect("Phải lập kế hoạch sửa câu");
            let prefix: String = raw.chars().take(plan.delete_from).collect();
            let fixed = format!("{prefix}{}", plan.replacement);
            assert_eq!(fixed, expected);
        }
    }
}
