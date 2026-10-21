// SPDX-License-Identifier: GPL-3.0-or-later
//! Cac luat kiem tra cau - moi luat 1 file (nguyen tac tu Harper `linting/`).
//!
//! V1 chi co luat **objective** (P4-0 §5.1): loi nao nguoi dung deu dong y.
//! Luat mo ho (ngu phap hoc thuat) de cho v2 - khi do phai co `auto_fixable =
//! false` va chi hien goi y.

pub mod capitalize_sentence;
pub mod comma_at_end;
pub mod diacritic_typo;
pub mod double_negative;
pub mod double_space;
pub mod duplicate_word;
pub mod language;
pub mod sentence_repeat;
pub mod space_after_punct;
pub mod space_before_punct;
pub mod spelling;

use crate::dict::{Dictionary, NoDictionary};
use crate::tokenize::{tokenize, Token, TokenKind};
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
        Ctx {
            tokens: tokenize(text),
            masked: crate::mask::masked_ranges(text),
            chars: text.chars().collect(),
            start_of_sentence,
            dict: &NO_DICTIONARY,
        }
    }

    /// Tạo ngữ cảnh với **từ điển** do caller nạp (cho [`check_with`](crate::check_with)).
    pub fn new_with_dict<'a>(
        text: &str,
        start_of_sentence: bool,
        dict: &'a dyn Dictionary,
    ) -> Ctx<'a> {
        Ctx {
            tokens: tokenize(text),
            masked: crate::mask::masked_ranges(text),
            chars: text.chars().collect(),
            start_of_sentence,
            dict,
        }
    }

    /// Chuoi cua mot token.
    pub fn token_text(&self, token: &Token) -> String {
        self.chars[token.start..token.end].iter().collect()
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

/// Dau `.` co phai that su ket thuc cau khong?
///
/// - `.` nam **trong ngoac** ("(Từ điển EN...) đặt ở...") hoac **trong dau nhay**
///   khong phai het cau -> bo qua (Fassist-86).
pub(crate) fn is_real_sentence_end(ctx: &Ctx, punct: &crate::tokenize::Token) -> bool {
    if !is_sentence_end(ctx.chars[punct.start]) {
        return false;
    }
    if ctx.chars[punct.start] != '.' {
        return true;
    }
    // Ba cham lien (hoac nhieu hon) = ellipsis trong tieng Viet, khong phai het
    // cau ("dieu tot... ban nghi sao?") -> bo qua (Fassist-95).
    let mut dots = 0usize;
    let mut i = punct.start;
    while i < ctx.chars.len() && ctx.chars[i] == '.' {
        dots += 1;
        i += 1;
    }
    if dots >= 3 {
        return false;
    }
    let mut depth = 0i32;
    let mut in_quote = false;
    for &c in &ctx.chars[..punct.start] {
        match c {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            '"' => in_quote = !in_quote,
            _ => {}
        }
    }
    depth == 0 && !in_quote
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

/// Token lai chu / so de rule khong phai tu tach lai.
pub(crate) fn is_word(token: &Token) -> bool {
    token.kind == TokenKind::Word
}
