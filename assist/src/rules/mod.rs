// SPDX-License-Identifier: GPL-3.0-or-later
//! Cac luat kiem tra cau - moi luat 1 file (nguyen tac tu Harper `linting/`).
//!
//! V1 chi co luat **objective** (P4-0 §5.1): loi nao nguoi dung deu dong y.
//! Luat mo ho (ngu phap hoc thuat) de cho v2 - khi do phai co `auto_fixable =
//! false` va chi hien goi y.

pub mod article_an;
pub mod capitalize_sentence;
pub mod collocation_typo;
pub mod comma_at_end;
pub mod diacritic_typo;
pub mod double_negative;
pub mod double_space;
pub mod duplicate_word;
pub mod en_confusions;
pub mod en_spelling;
pub mod language;
pub mod phrase_repeat;
pub mod sentence_grammar;
pub mod sentence_repeat;
pub mod space_after_punct;
pub mod space_before_punct;
pub mod spelling;

use crate::dict::{Dictionary, NoDictionary};
use crate::tokenize::{Token, TokenKind};
use crate::{Issue, IssueKind, Span};

/// Từ điển mặc định của [`Ctx::new`]: mọi từ coi là **đúng** — `check`/
/// `check_sentence` giữ hành vi cũ (không bao giờ báo lỗi spelling) khi caller
/// chưa nạp từ điển (AS-040).
static NO_DICTIONARY: NoDictionary = NoDictionary;

/// Ngữ cảnh dùng chung cho các luật: token hoá + vùng bị che + từ điển.
pub struct Ctx<'a> {
    /// Token cua cau.
    pub tokens: Vec<Token>,
    /// Vung bi che (URL/email) - khong bao loi trong vung nay.
    pub masked: Vec<(usize, usize)>,
    /// Cau tach thanh ky tu (index = char offset).
    pub chars: Vec<char>,
    /// Ky tu dau co phai la dau cau that khong (fragment = false).
    pub start_of_sentence: bool,
    /// Từ điển cho luật spelling ([`NoDictionary`] = mọi từ coi là đúng).
    pub dict: &'a dyn Dictionary,
    /// Ngôn ngữ chi phối của câu (vòng 24 — trợ lý tiếng Anh/mix văn phòng).
    pub lang: language::Lang,
    /// Cụm cố định hợp lệ (từ láy, thành ngữ…) — lỗi từ vựng chồng lên vùng
    /// này bị bỏ ở điểm chặn duy nhất (`phrases::drop_issues_in_protected`).
    pub protected: Vec<(usize, usize)>,
    /// Vòng 42 (F5-hoist + Pass EN-2 R1): trạng thái **trong ngoặc kép** tại
    /// mỗi vị trí ký tự — O(n) một lần, mọi luật dùng chung (per-rule
    /// in_quotes là O(n²)). R1: parity MỞ nhưng KHÔNG còn nháy nào phía sau
    /// ⇒ coi như ngoài quote (chat thiếu dấu đóng / dấu inch `24"` lật parity
    /// suốt phần sau — không được nuốt lỗi phía sau).
    pub quote_parity: Vec<bool>,
}

/// Tính quote-parity có xử lý R1 (parity mở mà không còn nháy phía sau = ngoài).
fn compute_quote_parity(chars: &[char]) -> Vec<bool> {
    let mut v = Vec::with_capacity(chars.len());
    let mut in_q = false;
    let mut last_quote: Option<usize> = None;
    for (i, &c) in chars.iter().enumerate() {
        v.push(in_q);
        if matches!(c, '"' | '“' | '”') {
            in_q = !in_q;
            last_quote = Some(i);
        }
    }
    // Vị trí nào parity mở mà NHÁY CUỐI nằm trước nó ⇒ nháy mở lẻ — ngoài quote.
    if let Some(lq) = last_quote {
        for b in v.iter_mut().skip(lq + 1) {
            *b = false;
        }
    }
    v
}

/// Số lần gọi `suggest` tối đa cho một câu (Fassist-105, P4-0 D40).
///
/// Ngân sách này được áp dụng qua bộ bọc
/// [`Budgeted`](crate::dict::Budgeted) (P4-10 §5.3) — luật spelling **không**
/// đếm thủ công nữa; bộ bọc giới hạn số lần gọi + đo tổng thời gian, hết
/// ngân sách thì trả gợi ý rỗng (fail-open).
pub const DEFAULT_SUGGEST_BUDGET: usize = 2;

impl Ctx<'_> {
    /// Tao ngữ cảnh cho mot cau (không nạp từ điển → không báo spelling).
    ///
    /// `start_of_sentence` = `true` khi text la **cau tron ven** (ky tu dau la
    /// dau cau that); `false` khi la **doan** (co the nam giua cau).
    pub fn new(text: &str, start_of_sentence: bool) -> Ctx<'static> {
        Self::new_with_dict(text, start_of_sentence, &NO_DICTIONARY)
    }

    /// Tạo ngữ cảnh với **từ điển** do caller nạp (cho [`check_with`](crate::check_with)).
    pub fn new_with_dict<'a>(
        text: &str,
        start_of_sentence: bool,
        dict: &'a dyn Dictionary,
    ) -> Ctx<'a> {
        let chars: Vec<char> = text.chars().collect();
        let tokens = crate::tokenize::tokenize_chars(&chars);
        let lang = language::classify_tokens(&chars, &tokens).unwrap_or(language::Lang::Vi);
        let masked = crate::mask::masked_ranges_chars(&chars);
        let protected =
            crate::phrases::protected_spans(&chars, &tokens, lang == language::Lang::Vi);
        let quote_parity = compute_quote_parity(&chars);
        Ctx {
            tokens,
            masked,
            chars,
            start_of_sentence,
            dict,
            lang,
            protected,
            quote_parity,
        }
    }

    /// Tạo ngữ cảnh chỉ khi L0 nhận diện được ngôn ngữ (1 lần tách chars + 1 lần tokenize).
    pub(crate) fn try_new_with_dict<'a>(
        text: &str,
        start_of_sentence: bool,
        dict: &'a dyn Dictionary,
    ) -> Option<Ctx<'a>> {
        let chars: Vec<char> = text.chars().collect();
        let tokens = crate::tokenize::tokenize_chars(&chars);
        let lang = language::classify_tokens(&chars, &tokens)?;
        let masked = crate::mask::masked_ranges_chars(&chars);
        let protected =
            crate::phrases::protected_spans(&chars, &tokens, lang == language::Lang::Vi);
        let quote_parity = compute_quote_parity(&chars);
        Some(Ctx {
            tokens,
            masked,
            chars,
            start_of_sentence,
            dict,
            lang,
            protected,
            quote_parity,
        })
    }

    /// Vùng `[start, end)` có chồng lên cụm cố định hợp lệ không.
    pub fn is_protected(&self, start: usize, end: usize) -> bool {
        self.protected.iter().any(|&(s, e)| start < e && s < end)
    }

    /// Chuoi cua mot token.
    pub fn token_text(&self, token: &Token) -> String {
        self.chars[token.start..token.end].iter().collect()
    }

    /// Slice ky tu cua mot token (0-alloc).
    pub fn token_chars(&self, token: &Token) -> &[char] {
        &self.chars[token.start..token.end]
    }

    /// Vung `[start, end)` co bi che hay khong.
    pub fn is_masked(&self, start: usize, end: usize) -> bool {
        self.masked.iter().any(|(s, e)| start < *e && *s < end)
    }
}

/// Dau cau "manh" - sau do bat dau mot cau moi.
///
/// Khong tinh `…` (ellipsis): trong tieng Viet `…` thuong la dau lot tri
/// ("dân, dạy, dưới…") chu khong phai het cau (Fassist-87).
pub(crate) fn is_sentence_end(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?')
}

/// Dau `.` / `!` / `?` co phai that su ket thuc cau khong?
///
/// - Nam **trong ngoac** ("(Từ điển EN...) đặt ở...") hoac **trong dau nhay**
///   ("Anh hỏi \"Sao thế?\" rồi đi") khong phai het cau -> bo qua (Fassist-86).
pub(crate) fn is_real_sentence_end(ctx: &Ctx, punct: &crate::tokenize::Token) -> bool {
    let slice = &ctx.chars[punct.start..punct.end];
    let first = slice[0];
    let last = *slice.last().unwrap_or(&first);
    // Cho phép token Punct kết thúc bằng `.`/`!`/`?` ngay sau ngoặc đóng / nháy đóng
    // (ví dụ `")."` trong `"Tôi đi học (ở Hà Nội). bạn khỏe không"`).
    let has_end_punct = is_sentence_end(first)
        || (is_sentence_end(last)
            && slice[..slice.len() - 1]
                .iter()
                .all(|&c| matches!(c, ')' | ']' | '"' | '”' | '\'')));
    if !has_end_punct {
        return false;
    }
    if first == '.' || last == '.' {
        // Ba cham lien (hoac nhieu hon) = ellipsis trong tieng Viet, khong phai het
        // cau ("dieu tot... ban nghi sao?") -> bo qua (Fassist-95).
        let dot_pos = slice
            .iter()
            .position(|&c| c == '.')
            .map(|p| punct.start + p)
            .unwrap_or(punct.start);
        let mut dots = 0usize;
        let mut i = dot_pos;
        while i < ctx.chars.len() && ctx.chars[i] == '.' {
            dots += 1;
            i += 1;
        }
        if dots >= 3 {
            return false;
        }
        // Vòng 24 (audit EN trap 27-33): dấu `.` sau **từ viết tắt** không phải kết
        // thúc câu — "8 a.m. nhé" / "U.S. Army" / "i.e. apple": nếu coi là hết câu,
        // capitalize sẽ bêu chữ thường sau viết tắt ("a.m. nhé" → "Nhé").
        if first == '.' && is_abbreviation_dot(ctx, punct) {
            return false;
        }
    }
    // Tính cân bằng ngoặc/nháy đến hết phần đóng ngoặc trước dấu kết câu.
    let check_until = if !is_sentence_end(first) && is_sentence_end(last) {
        punct.end - 1
    } else {
        punct.start
    };
    let mut depth = 0i32;
    let mut in_quote = false;
    // Vòng 32 (audit A5): nháy CONG “ ” cùng họ với " — câu trích
    // ("Anh nói “tôi đi. bạn ở lại”.") không kết thúc ở dấu chấm TRONG nháy.
    for &c in &ctx.chars[..check_until] {
        match c {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            '"' | '“' | '”' => in_quote = !in_quote,
            _ => {}
        }
    }
    depth == 0 && !in_quote
}

/// Danh sách viết tắt kết thúc bằng `.` (đóng — chỉ các dạng phổ biến nhất;
/// không bao gồm từ thường 2 chữ cái như "em", "ai", "to", "it", "me", "ok").
const ABBREV_WORDS: &[&str] = &[
    "etc", "cf", "vs", "eg", "ie", "mr", "mrs", "ms", "dr", "jr", "sr", "st", "al", "co", "inc",
    "ltd", "prof", "fig", "vol", "no", "op", "tp", "ts", "gs", "ks", "bs", "th", "cn",
];

/// `.` tại vị trí `punct` có phải dấu chấm của **từ viết tắt** không (vòng 24)?
///
/// Đúng khi từ ngay sát trước dấu `.` là:
/// - 1 chữ cái ASCII ("a.m.", "p.m.", "U.S.", "U.K.", "e.g.", "i.e.", "J. Smith", "v.v."), HOẶC
/// - trong danh sách đóng [`ABBREV_WORDS`] ("etc.", "cf.", "vs.", "Mr.", "Dr.", "et al."),
/// - "3.a" (số trước + một chữ cái sau — mục con).
pub(crate) fn is_abbreviation_dot(ctx: &Ctx, punct: &crate::tokenize::Token) -> bool {
    // "3.a" (mục con): số ngay trước + MỘT chữ cái ASCII ngay sau — không
    // áp cho chữ thường sau chấm ("rồi.Tạm" là LỖI thiếu space thật; tên
    // miền "example.com" đã được che bởi `mask` nên không tới đây).
    let digit_before = punct.start > 0 && ctx.chars[punct.start - 1].is_ascii_digit();
    let single_letter_after = ctx
        .chars
        .get(punct.end)
        .is_some_and(|c| c.is_ascii_alphabetic())
        && ctx
            .chars
            .get(punct.end + 1)
            .is_none_or(|c| !c.is_alphanumeric());
    if digit_before && single_letter_after {
        return true;
    }
    // Từ phải nằm NGAY SÁT trước dấu `.` (`prev.end == punct.start`).
    let prev = ctx
        .tokens
        .iter()
        .rev()
        .find(|t| t.end <= punct.start && t.kind != crate::tokenize::TokenKind::Space);
    let Some(prev) = prev else {
        return false;
    };
    if prev.end != punct.start || prev.kind != crate::tokenize::TokenKind::Word {
        return false;
    }
    let slice = &ctx.chars[prev.start..prev.end];
    // Vòng 32 (audit A6): viết tắt Việt viết HOA ("ĐH.", "ĐC.", "ĐT.", "QG.")
    // — từ toàn-hoa 2-3 ký tự, bất kể script, đứng trước `.` là viết tắt;
    // chữ thường có dấu ("nhé.") vẫn kết-thúc-câu như cũ.
    if !slice.iter().all(|c| c.is_ascii_alphabetic()) {
        let all_upper = (2..=3).contains(&slice.len()) && slice.iter().all(|c| c.is_uppercase());
        return all_upper;
    }
    if slice.len() == 1 {
        return true;
    }
    let lower: String = slice.iter().map(|c| c.to_ascii_lowercase()).collect();
    ABBREV_WORDS.contains(&lower.as_str())
}

/// Tao issue **chi goi y** (khong tu sua) — cho luat mo ho (Fassist-90).
///
/// Luật objective thi dung `issue()` (co `Fix`); luật mo ho chi bao loi de
/// nguoi dung quyet dinh => `auto_fixable = false`, `fix = None`.
pub(crate) fn issue_suggestion(
    kind: IssueKind,
    span: Span,
    message: String,
    suggestions: Vec<String>,
) -> Issue {
    Issue {
        kind,
        span,
        message,
        suggestions,
        auto_fixable: false,
        confidence: 60,
        fix: None,
    }
}

/// Dau can co khoang trang phia sau (tru `)` `]` `}`).
pub(crate) fn needs_space_after(ch: char) -> bool {
    matches!(ch, ',' | '.' | ';' | ':' | '!' | '?')
}

/// Tao issue mau - dung chung de mọi luat nhat quan.
pub(crate) fn issue(
    kind: IssueKind,
    span: Span,
    message: String,
    suggestions: Vec<String>,
    fix: crate::Fix,
) -> Issue {
    Issue {
        kind,
        span,
        message,
        suggestions,
        auto_fixable: true,
        confidence: 100,
        fix: Some(fix),
    }
}

/// Vòng 42 (Pass EN-2 N1/N2): từ KỀ (trước/sau, bỏ Space/Punct) là **tín
/// hiệu tiếng Việt** — âm tiết Việt-only (`is_vietnamese_ascii_word`, loại
/// giao điểm do/to/an/he/i) hoặc chữ Việt có dấu. Cặp EN đụng âm tiết Việt
/// ("it do lỗi", "cho i") phải skip khi hàng xóm Việt.
pub(crate) fn vn_neighbor_hint(ctx: &Ctx, start: usize) -> bool {
    let vn_word = |t: &Token| -> bool {
        let w = ctx.token_text(t).to_lowercase();
        // Vòng 44 (P-2): âm tiết Việt **loại EN function words** ("that"=thắt,
        // "me"=mẹ, "he"=hề — nếu không, "My brother told me that i could"
        // bị hint nuốt rule). Giao điểm EN freq GIỮ ("di"=đi — FPR-2 gốc).
        (language::is_vn_ascii_syllable(&w) && !language::is_en_function_word(&w))
            || w.chars().any(language::is_vietnamese_letter)
    };
    let idx = match ctx.tokens.iter().position(|t| t.start == start) {
        Some(i) => i,
        None => return false,
    };
    let prev = ctx.tokens[..idx]
        .iter()
        .rev()
        .find(|t| t.kind == TokenKind::Word)
        .is_some_and(vn_word);
    let next = ctx.tokens[idx + 1..]
        .iter()
        .find(|t| t.kind == TokenKind::Word)
        .is_some_and(vn_word);
    prev || next
}

/// Token lai chu / so de rule khong phai tu tach lai.
pub(crate) fn is_word(token: &Token) -> bool {
    token.kind == TokenKind::Word
}
