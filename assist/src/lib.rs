// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-assist` - engine kiem tra cau tieng Viet cho bo go (Phan 4 - P4).
//!
//! Invariants (Handbook S1-S4, P4-0 §3):
//! - **Khong** network / file I/O / spawn thread / log noi dung text.
//! - Offset trong [`Span`] la **ky tu Unicode** (char) - khong phai byte, khong
//!   phai UTF-16; adapter chuyen doi khi can (TSF dung UTF-16 code unit).
//! - Khong global mutable state: moi lan [`check`] tao [`rules::Ctx`] cuc bo.
//!
//! V1 chi chua luat **objective** (P4-0 §5.1) - moi loi deu co [`Fix`] tu
//! dong hoac chi la goi y. Luat mo ho (ngu phap hoc thuat) de cho v2.

pub mod fix;
pub mod mask;
pub mod rules;
pub mod tokenize;

pub use fix::{apply_auto_fixes, Fix};
pub use tokenize::{tokenize, Token, TokenKind};

/// Loai loi - id on dinh dung cho corpus / CLI / JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueKind {
    /// L1/L2/L11 - lap tu lien tiep.
    DuplicateWord,
    /// L3 - thua khoang trang truoc dau cau.
    SpaceBeforePunct,
    /// L4 - thieu khoang trang sau dau cau.
    SpaceAfterPunct,
    /// L5 - khoang trang kep.
    DoubleSpace,
    /// L6 - chu thuong o dau cau.
    LowercaseSentenceStart,
    /// L8 - sai vi tri dau (bang du lieu co chon loc).
    DiacriticTypo,
}

impl IssueKind {
    /// Id chuoi de trong (corpus, CLI `--json`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateWord => "duplicate_word",
            Self::SpaceBeforePunct => "space_before_punct",
            Self::SpaceAfterPunct => "space_after_punct",
            Self::DoubleSpace => "double_space",
            Self::LowercaseSentenceStart => "lowercase_sentence_start",
            Self::DiacriticTypo => "diacritic_typo",
        }
    }

    /// Parse nguoc lai tu [`IssueKind::as_str`].
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "duplicate_word" => Self::DuplicateWord,
            "space_before_punct" => Self::SpaceBeforePunct,
            "space_after_punct" => Self::SpaceAfterPunct,
            "double_space" => Self::DoubleSpace,
            "lowercase_sentence_start" => Self::LowercaseSentenceStart,
            "diacritic_typo" => Self::DiacriticTypo,
            _ => return None,
        })
    }

    /// Loi nay sua duoc tu dong hay khong (chi luat objective moi `true`).
    pub fn auto_fixable(self) -> bool {
        matches!(
            self,
            Self::DuplicateWord
                | Self::SpaceBeforePunct
                | Self::SpaceAfterPunct
                | Self::DoubleSpace
                | Self::LowercaseSentenceStart
                | Self::DiacriticTypo
        )
    }
}

/// Vung loi - do dai theo **ky tu Unicode**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Offset bat dau (ky tu Unicode, inclusive).
    pub start: usize,
    /// Offset ket thuc (exclusive).
    pub end: usize,
}

/// Mot loi tim duoc trong cau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Loai loi.
    pub kind: IssueKind,
    /// Vung loi (de highlight trong overlay).
    pub span: Span,
    /// Thong diep tieng Viet ngan.
    pub message: String,
    /// Goi y sua (chu thay thei doan loi).
    pub suggestions: Vec<String>,
    /// Co sua duoc tu dong hay khong.
    pub auto_fixable: bool,
    /// Cach sua tu dong (`None` = chi goi y, khong tu sua).
    pub fix: Option<Fix>,
}

/// Kiem tra mot **doan** cau (fragment) - giong nhung doan ta vua go trong IME.
///
/// Doan co the nam giua cau nen **khong** coi ky tu dau doan la dau cau
/// (khong bao loi "chu thuong o dau cau" cho doan).
///
/// Khong bao gio tra ve loi cho noi dung da duoc che (URL/email) - xem
/// [`mask::masked_ranges`].
pub fn check(text: &str) -> Vec<Issue> {
    check_with(text, false)
}

/// Kiem tra mot **cau tron ven** - ky tu dau la dau cau that.
///
/// Dung khi nguoi dung muon kiem tra mot cau doc lap (CLI, corpus).
pub fn check_sentence(text: &str) -> Vec<Issue> {
    check_with(text, true)
}

fn check_with(text: &str, start_of_sentence: bool) -> Vec<Issue> {
    let ctx = rules::Ctx::new(text, start_of_sentence);
    let mut issues = Vec::new();
    rules::duplicate_word::check(&ctx, &mut issues);
    rules::space_before_punct::check(&ctx, &mut issues);
    rules::space_after_punct::check(&ctx, &mut issues);
    rules::double_space::check(&ctx, &mut issues);
    rules::capitalize_sentence::check(&ctx, &mut issues);
    rules::diacritic_typo::check(&ctx, &mut issues);
    issues.sort_by_key(|i| (i.span.start, i.span.end, i.kind));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<IssueKind> {
        check(text).into_iter().map(|i| i.kind).collect()
    }

    #[test]
    fn cau_sach_khong_bao_loi() {
        assert!(kinds("Tôi đi học ở trường Đại học Bách Khoa.").is_empty());
    }

    #[test]
    fn lap_tu_bi_bat() {
        assert_eq!(kinds("tôi tôi đi học"), vec![IssueKind::DuplicateWord]);
    }

    #[test]
    fn url_khong_bi_bat() {
        // URL co dau cham va chu thuong - phai duoc che.
        assert!(kinds("xem https://example.com/a.b").is_empty());
    }

    #[test]
    fn cau_rong_khong_loi() {
        assert!(check("").is_empty());
    }
}
