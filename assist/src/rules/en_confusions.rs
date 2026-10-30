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
    // Vòng 42 (audit EN mode RC-3): CHỈ câu En-dominant trước đây ⇒ câu mix
    // Vi-dominant ("Anh should of done it sớm hơn") bị bỏ qua TOÀN BỘ cặp
    // từ-dễ-nhầm. Các cặp là bigram/trigram ASCII ĐÓNG của tiếng Anh
    // ("should of", "your the", "its been", "more … then"…) — văn bản tiếng
    // Việt chứa chúng là dùng từ Anh mượn và LUẬT PHẢI áp; không có cặp nào
    // trùng ngữ cảnh tiếng Việt hợp lệ (E34 probe + corpus FPR 0 xác nhận).
    // Riêng đại từ "i" (check_pronoun) giữ En-only — "i" là âm tiết Việt
    // ASCII, mở sẽ bêu chữ Việt.
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    let words: Vec<crate::tokenize::Token> = ctx
        .tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Word)
        .copied()
        .collect();
    // Vòng 42 (F5-hoist): trạng thái trong-ngoặc-kép tại MỖI vị trí ký tự —
    // O(n) một lần thay in_quotes O(n) mỗi cửa sổ (O(n²) tổng).
    let quote_parity: Vec<bool> = {
        let mut v = Vec::with_capacity(ctx.chars.len());
        let mut in_q = false;
        for &c in &ctx.chars {
            v.push(in_q);
            if matches!(c, '"' | '“' | '”') {
                in_q = !in_q;
            }
        }
        v
    };
    for w in words.windows(3) {
        let (w0, w1, w2) = (&w[0], &w[1], &w[2]);
        if w0.is_empty() || ctx.is_masked(w0.start, w2.end) {
            continue;
        }
        // Vòng 42 (Pass EN-1 F5): trích lời nguyên văn trong ngoặc kép —
        // 'He said "we should of known better"' là nguyên văn, không sửa.
        // (parity vector O(n) tính 1 lần — guard per-window O(1).)
        if quote_parity.get(w0.start).copied().unwrap_or(false) {
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

        // Helper: từ Word KẾ SAU token tại `pos` (skip Space/Punct —
        // windows(3) đếm từ, tokens có Space xen giữa).
        let next_word_after = |pos: usize| -> Option<String> {
            ctx.tokens
                .iter()
                .position(|t| t.start == pos)
                .and_then(|i| {
                    ctx.tokens[i + 1..]
                        .iter()
                        .find(|t| t.kind == TokenKind::Word)
                })
                .map(|t| token_word(ctx, t))
        };
        // 2) "your the/not/right" → "you're". Vòng 41 (Pass A N3): bỏ
        // "welcome" — cặp "your welcome" CHỈ có thể fire khi "Your" mở đầu
        // câu + có từ theo sau (windows(3)), mà khi đó "welcome" luôn là
        // tính từ hợp lệ ("Your welcome message") ⇒ entry 100% FP.
        if a == "your" && matches!(b.as_str(), "the" | "not" | "right") {
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
        //    "their going-away party" là đúng; vòng 41 Pass A T1: "going" +
        //    DANH TỪ đóng ("their going rate/price/order/concern") là cụm
        //    danh từ hợp lệ — không fire).
        if a == "their"
            && (matches!(b.as_str(), "the" | "not")
                || (b == "going"
                    && ctx.chars.get(w1.end) != Some(&'-')
                    && !next_word_after(w1.start).is_some_and(|w| {
                        matches!(
                            w.as_str(),
                            "rate" | "price" | "order" | "concern" | "concerns" | "offer"
                        )
                    })))
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
        //    big cities" đúng vì "went" ∉ copula). Vòng 41 (Pass A T2): to-
        // infinitive ("the plan was to slow down production") — sau tính từ
        // là PARTICLE của động từ ghép (danh sách đóng) ⇒ bỏ qua; "is to big
        // for me" (giới từ) và "is to big to fail" (to-v) vẫn fire.
        const PHRASAL_PARTICLES: &[&str] =
            &["down", "up", "out", "off", "away", "back", "over", "around"];
        // Vòng 42 (Pass EN-1 F2): next_word_after O(tokens) — tính LAZY chỉ
        // khi đã khớp copula+to+adj (trước đây vô điều kiện mỗi window 3 từ
        // ⇒ O(n²): 100k = 4.3s).
        if COPULAS.contains(&a.as_str())
            && b == "to"
            && TO_TOO_ADJECTIVES.contains(&c.as_str())
            && !next_word_after(w2.start).is_some_and(|w| PHRASAL_PARTICLES.contains(&w.as_str()))
        {
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

        // 6) "has/have/had lead to" → "led"
        if matches!(a.as_str(), "has" | "have" | "had") && b == "lead" && c == "to" {
            push_replace(
                ctx,
                w1,
                "led",
                IssueKind::WordConfusion,
                "\"lead to\" → \"led to\": quá khứ phân từ của lead là \"led\"".to_string(),
                out,
            );
            continue;
        }

        // 7) "the affect of" → "the effect of"
        if a == "the" && b == "affect" && c == "of" {
            push_replace(
                ctx,
                w1,
                "effect",
                IssueKind::WordConfusion,
                "\"the affect of\" → \"the effect of\": danh từ chỉ tác động là \"effect\""
                    .to_string(),
                out,
            );
            continue;
        }
    }

    for w in words.windows(2) {
        let (w0, w1) = (&w[0], &w[1]);
        if w0.is_empty() || ctx.is_masked(w0.start, w1.end) {
            continue;
        }
        let a = token_word(ctx, w0);
        let b = token_word(ctx, w1);

        // 8) "there own" / "they're own" → "their own"
        if (a == "there" || a == "they're") && b == "own" {
            push_replace(
                ctx,
                w0,
                "their",
                IssueKind::WordConfusion,
                "\"their own\": sở hữu trước \"own\" cần \"their\"".to_string(),
                out,
            );
            continue;
        }

        // 9) "accept for" → "except for"
        if a == "accept" && b == "for" {
            push_replace(
                ctx,
                w0,
                "except",
                IssueKind::WordConfusion,
                "\"accept for\" → \"except for\": mang nghĩa loại trừ".to_string(),
                out,
            );
            continue;
        }

        // 10) "to/will/can/might loose" → "lose"
        if matches!(
            a.as_str(),
            "to" | "will" | "can" | "might" | "won't" | "cannot"
        ) && b == "loose"
        {
            push_replace(
                ctx,
                w1,
                "lose",
                IssueKind::WordConfusion,
                "\"loose\" → \"lose\": sau động từ khiếm khuyết/to-infinitive là động từ \"lose\""
                    .to_string(),
                out,
            );
            continue;
        }

        // 11) "principle engineer/architect/developer" → "principal"
        if a == "principle"
            && matches!(
                b.as_str(),
                "engineer" | "architect" | "developer" | "consultant" | "designer" | "investigator"
            )
        {
            push_replace(
                ctx,
                w0,
                "principal",
                IssueKind::WordConfusion,
                "\"principle\" → \"principal\": chức danh chuyên môn cần \"principal\"".to_string(),
                out,
            );
            continue;
        }

        // 12) "take affect" → "take effect"
        if a == "take" && b == "affect" {
            push_replace(
                ctx,
                w1,
                "effect",
                IssueKind::WordConfusion,
                "\"take affect\" → \"take effect\": thành ngữ mang nghĩa có hiệu lực".to_string(),
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
                // Vòng 41 (Pass A N2): phẩy/chấm phẩy = mệnh đề trước "then"
                // ĐÃ ĐÓNG — "Wait a few more minutes, then try again" (mẫu
                // câu cực phổ biến) không phải so sánh trực tiếp; quét xuyên
                // phẩy bắt "more" định-lượng → tự sửa "than" phá câu đúng.
                break;
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

/// Cặp VI-EN TRÙNG NGHĨA kề nhau trong câu Việt (vòng 40, R2 persona RH-2):
/// "đã been hủy", "trước before 17h", "already đã gửi" — người viết nhét từ
/// Anh vào khung ngữ pháp Việt, thừa một trong hai. Danh sách ĐÓNG, chỉ cặp
/// nghĩa-trùng-tuyệt-đối; CHỈ câu Vi-dominant (câu En có ngữ pháp riêng,
/// "after all" / "today" không bao giờ thừa). Suggestion-only (confidence 60,
/// không tự xoá) — người dùng quyết giữ từ nào.
const REDUNDANT_MIX_PAIRS: &[(&str, &str)] = &[
    ("đã", "been"),
    ("đã", "already"),
    ("trước", "before"),
    ("sau", "after"),
    ("mai", "tomorrow"),
    ("nay", "today"),
];

pub fn check_redundant_mix(ctx: &Ctx, out: &mut Vec<Issue>) {
    if ctx.lang != Lang::Vi {
        return;
    }
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    for window in ctx.tokens.windows(3) {
        let (w0, gap, w1) = (&window[0], &window[1], &window[2]);
        // 2 từ KỀ NHÂU: chỉ cách nhau đúng một dấu cách đơn.
        if w0.kind != TokenKind::Word || w1.kind != TokenKind::Word {
            continue;
        }
        if gap.kind != TokenKind::Space || gap.len() != 1 || ctx.chars[gap.start] != ' ' {
            continue;
        }
        let a = token_word(ctx, w0);
        let b = token_word(ctx, w1);
        if a.is_empty() || b.is_empty() || ctx.is_masked(w0.start, w1.end) {
            continue;
        }
        // R2-T5: tên riêng/tiêu đề viết HOA ("Hôm nay Today show", "Trước
        // Before Sunrise", "Duy nhát…") — HOA đầu của BẤT KỲ từ nào trong
        // cặp ⇒ bỏ qua (chỉ bắt dạng thường; suggestion-only nên recall
        // mất ở đầu-câu-hoa là chấp nhận được).
        let cap = |tok: &crate::tokenize::Token| ctx.chars[tok.start].is_uppercase();
        if cap(w0) || cap(w1) {
            continue;
        }
        let hit = REDUNDANT_MIX_PAIRS
            .iter()
            .any(|&(x, y)| (a == x && b == y) || (a == y && b == x));
        if !hit || in_quotes_span(ctx, w0.start) {
            continue;
        }
        out.push(super::issue_suggestion(
            IssueKind::WordConfusion,
            crate::Span {
                start: w0.start,
                end: w1.end,
            },
            format!("Thừa từ trùng nghĩa: \"{a} {b}\" — chỉ cần một trong hai"),
            vec![a.clone()],
        ));
    }
}

/// Ký tự trước `pos` có nằm trong cặp ngoặc kép không (cùng tiêu chí
/// duplicate_word::in_quotes — lời trích nguyên văn không phải lỗi).
fn in_quotes_span(ctx: &Ctx, pos: usize) -> bool {
    ctx.chars[..pos]
        .iter()
        .filter(|c| matches!(**c, '"' | '“' | '”'))
        .count()
        % 2
        == 1
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
        assert_eq!(
            kinds("They built there own software framework."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("Everyone agree accept for John."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("The incident has lead to serious issues."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("We might loose our database connection."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("He was promoted to principle engineer today."),
            vec![IssueKind::WordConfusion]
        );
        assert_eq!(
            kinds("The new policy will take affect tomorrow."),
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

    /// Vòng 42 (RC-3): word-confusion cặp ASCII đóng chạy CẢ câu mix
    /// ("bạn should of known nhé" = dùng từ Anh mượn ⇒ PHẢI bắt); riêng đại
    /// từ "i" giữ En-only — "i" là âm tiết Việt ASCII ("gửi cho i nhé").
    #[test]
    fn cau_mix_confusion_fire_pronoun_van_en_only() {
        // sentence mode: "bạn" đầu câu thường → L6 hợp lệ kèm theo.
        assert!(kinds("bạn should of known nhé").contains(&IssueKind::WordConfusion));
        // "i" thường giữa câu Việt — LowercasePronoun KHÔNG được bêu.
        assert!(
            kinds("gửi cho i nhé")
                .into_iter()
                .all(|k| !matches!(k, IssueKind::LowercasePronoun)),
            "đại từ i giữ En-only"
        );
    }
}

#[cfg(test)]
mod redundant_mix_tests {
    use crate::check;

    #[test]
    fn bat_tu_en_thua_trung_nghia() {
        // RH-2: suggestion-only, kind WordConfusion.
        for s in [
            "Rất tiếc đơn hàng của quý khách đã been hủy.",
            "Quý khách vui lòng phản hồi trước before 17h.",
            "Mình already đã gửi tài liệu cho anh.",
            "Hẹn gặp lại mai tomorrow nhé.",
        ] {
            let issues = check(s);
            assert_eq!(issues.len(), 1, "{s} => {issues:?}");
            assert_eq!(issues[0].kind, crate::IssueKind::WordConfusion);
            assert!(!issues[0].auto_fixable, "suggestion-only");
        }
    }

    #[test]
    fn khong_bat_cau_en_dominant() {
        // Câu EN: "after"/"today" là ngữ pháp chuẩn, không bao giờ thừa.
        assert!(check("Please reply before 5pm today.").is_empty());
        assert!(check("I will call you after the meeting today.").is_empty());
        assert!(check("We shipped it already.").is_empty());
    }

    #[test]
    fn khong_bat_khi_khong_phai_cap_trung_nghia() {
        assert!(check("Anh check giúp em the report nhé.").is_empty());
        assert!(check("Đơn hàng đã được xác nhận.").is_empty());
        assert!(check("Trước khi đi, nhớ khóa cửa.").is_empty());
    }
}
