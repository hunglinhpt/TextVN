// SPDX-License-Identifier: GPL-3.0-or-later
//! Cac luat kiem tra cau - moi luat 1 file (nguyen tac tu Harper `linting/`).
//!
//! V1 chi co luat **objective** (P4-0 §5.1): loi nao nguoi dung deu dong y.
//! Luat mo ho (ngu phap hoc thuat) de cho v2 - khi do phai co `auto_fixable =
//! false` va chi hien goi y.

pub mod capitalize_sentence;
pub mod diacritic_typo;
pub mod double_space;
pub mod duplicate_word;
pub mod language;
pub mod space_after_punct;
pub mod space_before_punct;

use crate::tokenize::{tokenize, Token, TokenKind};
use crate::{Issue, IssueKind, Span};

/// Ngữ cảnh dùng chung cho các luat: token hoá + vùng bị che.
pub struct Ctx {
    /// Token cua cau.
    pub tokens: Vec<Token>,
    /// Vung bi che (URL/email) - khong bao loi trong vung nay.
    pub masked: Vec<(usize, usize)>,
    /// Cau tach thanh ky tu (index = char offset).
    pub chars: Vec<char>,
    /// Ky tu dau co phai la dau cau that khong (fragment = false).
    pub start_of_sentence: bool,
}

impl Ctx {
    /// Tao ngữ cảnh cho mot cau.
    ///
    /// `start_of_sentence` = `true` khi text la **cau tron ven** (ky tu dau la
    /// dau cau that); `false` khi la **doan** (co the nam giua cau).
    pub fn new(text: &str, start_of_sentence: bool) -> Self {
        Self {
            tokens: tokenize(text),
            masked: crate::mask::masked_ranges(text),
            chars: text.chars().collect(),
            start_of_sentence,
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
