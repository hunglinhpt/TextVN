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

pub mod dict;
pub mod fix;
pub mod mask;
pub mod rules;
pub mod tokenize;

pub use fix::{apply_auto_fixes, plan_sentence_fix, Fix, SentenceFixPlan};
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
    /// L9 - lap tu o ranh gioi cau (chi goi y).
    SentenceRepeat,
    /// L10 - phu dinh kep "khong co ... khong" (chi goi y).
    DoubleNegative,
    /// L12 - dau phay o cuoi cau (chi goi y).
    CommaAtEnd,
    /// AS-040 - tu la khong co trong tu dien (chi goi y, confidence 60).
    Spelling,
    /// EN - mạo từ "a/an" không khớp âm đầu ("a apple" → "an apple",
    /// "an university" → "a university"). Chỉ chạy trên câu En-dominant
    /// (vòng 24); danh sách ngoại lệ đóng theo ÂM (không theo chữ).
    ArticleAn,
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
            Self::SentenceRepeat => "sentence_repeat",
            Self::DoubleNegative => "double_negative",
            Self::CommaAtEnd => "comma_at_end",
            Self::Spelling => "spelling",
            Self::ArticleAn => "article_an",
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
            "sentence_repeat" => Self::SentenceRepeat,
            "double_negative" => Self::DoubleNegative,
            "comma_at_end" => Self::CommaAtEnd,
            "spelling" => Self::Spelling,
            "article_an" => Self::ArticleAn,
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
                | Self::ArticleAn
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
    /// Diem tin cay 0-100 (P4-0 D28): luat objective = 100; v2 se co luat mo
    /// ho voi diem thap de khong auto-fix.
    pub confidence: u8,
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
///
/// Ham nay chay voi [`dict::NoDictionary`] — moi tu deu coi la dung, **khong**
/// bao loi spelling (giu hanh vi cu); bao spelling chi khi dung [`check_with`]
/// voi tu dien do caller nap (AS-040).
pub fn check(text: &str) -> Vec<Issue> {
    check_with_internal(text, false, &dict::NoDictionary)
}

/// Kiem tra mot **cau tron ven** - ky tu dau la dau cau that.
///
/// Dung khi nguoi dung muon kiem tra mot cau doc lap (CLI, corpus).
///
/// Giong [`check`]: chay voi [`dict::NoDictionary`] — khong bao spelling.
pub fn check_sentence(text: &str) -> Vec<Issue> {
    check_with_internal(text, true, &dict::NoDictionary)
}

/// Kiem tra mot **doan** cau voi **tu dien** do caller nap (AS-040).
///
/// Giu nguyen gate L0 ([`rules::language::is_vietnamese`]) va chay tat ca luat
/// hien co **cong** luat spelling voi `dict`: tu tieng Viet co dau khong nam
/// trong tu dien bao [`IssueKind::Spelling`] (chi goi y, confidence 60).
///
/// Engine van **0-I/O** (P4-0 §3): noi dung tu dien (FST mmap) do crate khac
/// (vi du `textvn-dict`) nap va truyen vao qua [`dict::Dictionary`].
pub fn check_with(text: &str, dict: &dyn dict::Dictionary) -> Vec<Issue> {
    check_with_internal(text, false, dict)
}

/// Kiem tra mot **cau tron ven** voi **tu dien** do caller nap (AS-040 / AS-042).
///
/// Giong [`check_with`] nhung coi ky tu dau la dau cau that (kiem tra them
/// luat chu thuong o dau cau).
pub fn check_sentence_with(text: &str, dict: &dyn dict::Dictionary) -> Vec<Issue> {
    check_with_internal(text, true, dict)
}

fn check_with_internal(
    text: &str,
    start_of_sentence: bool,
    dict: &dyn dict::Dictionary,
) -> Vec<Issue> {
    // L0 (D16 → vòng 24): phân loại ngôn ngữ. Không nhận diện được → im lặng
    // (fail-open). En-dominant: các luật hiện có **tự chặn** trên nội dung
    // tiếng Việt (diacritic/spelling cần chữ Việt, repeat/negative dùng từ
    // chức năng Việt) nên chạy an toàn; riêng a/an là luật chỉ tiếng Anh.
    let Some(ctx) = rules::Ctx::try_new_with_dict(text, start_of_sentence, dict) else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    rules::duplicate_word::check(&ctx, &mut issues);
    rules::space_before_punct::check(&ctx, &mut issues);
    rules::space_after_punct::check(&ctx, &mut issues);
    rules::double_space::check(&ctx, &mut issues);
    rules::capitalize_sentence::check(&ctx, &mut issues);
    rules::diacritic_typo::check(&ctx, &mut issues);
    rules::sentence_repeat::check(&ctx, &mut issues);
    rules::double_negative::check(&ctx, &mut issues);
    rules::comma_at_end::check(&ctx, &mut issues);
    rules::spelling::check(&ctx, &mut issues);
    // Vòng 26: a/an chạy ở MỌI ngôn ngữ — lỗi mạo từ xuất hiện cả trong câu
    // mix ("please send a email nhé"); guard nội bộ của rule tự chặn từ
    // tiếng Việt có dấu ("a ô" → thoát).
    rules::article_an::check(&ctx, &mut issues);
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

    #[test]
    fn check_sentence_hoa_dau_cau() {
        // Câu trọn vẹn -> ký tự đầu là đầu câu thật (Fassist-51).
        assert_eq!(
            kinds_sentence("Xin chào. bạn khỏe không"),
            vec![IssueKind::LowercaseSentenceStart]
        );
        // Câu bắt đầu bằng chữ thường: ở chế độ "câu trọn vẹn" cả đầu câu
        // đầu tiên cũng được coi là đầu câu thật.
        assert_eq!(kinds_sentence("xin chào. bạn khỏe không").len(), 2);
        // Fragment -> ký tự đầu KHÔNG coi là đầu câu.
        assert!(check("xin chào. bạn khỏe không")
            .iter()
            .all(|i| i.kind != IssueKind::LowercaseSentenceStart || i.span.start > 0));
    }

    fn kinds_sentence(text: &str) -> Vec<IssueKind> {
        check_sentence(text).into_iter().map(|i| i.kind).collect()
    }

    /// Từ điển "rỗng" — không chứa từ nào → mọi từ tiếng Việt đều "lạ".
    struct TuDienRong;

    impl dict::Dictionary for TuDienRong {
        fn contains(&self, _word: &str) -> bool {
            false
        }
        fn suggest(&self, word: &str, _limit: usize) -> Vec<String> {
            vec![format!("?{word}")]
        }
    }

    #[test]
    fn check_khong_bao_spelling_khi_chua_co_tu_dien() {
        // Hành vi cũ: check/check_sentence dùng NoDictionary → từ lạ im lặng
        // (AS-040). Câu này phải sạch như trước khi có luật spelling.
        assert!(kinds("Tôi đi học ở trường Đại học Bách Khoa.").is_empty());
        assert!(kinds_sentence("Tôi đi học ở trường Đại học Bách Khoa.").is_empty());
    }

    #[test]
    fn check_with_bao_tu_la_theo_tu_dien() {
        let issues = check_with("Tôi quẫn quá.", &TuDienRong);
        let spelling: Vec<_> = issues
            .iter()
            .filter(|i| i.kind == IssueKind::Spelling)
            .collect();
        assert!(!spelling.is_empty(), "từ điển rỗng → từ lạ phải được báo");
        for issue in &spelling {
            assert!(!issue.auto_fixable);
            assert_eq!(issue.confidence, 60);
            // Ngan sach suggest (Fassist-105): chi mot phan nho so tu la
            // duoc kem goi y, khong phai moi issue deu co goi y.
            assert!(issue.suggestions.len() <= 1);
        }
        assert!(
            spelling.iter().any(|i| !i.suggestions.is_empty()),
            "phai co it nhat mot tu la kem goi y"
        );
        assert!(
            spelling
                .iter()
                .filter(|i| !i.suggestions.is_empty())
                .count()
                <= rules::DEFAULT_SUGGEST_BUDGET,
            "so lan goi suggest phai nho hon ngan sach"
        );
        assert_eq!(
            IssueKind::from_id("spelling"),
            Some(IssueKind::Spelling),
            "id corpus/CLI phải khớp enum"
        );
    }

    #[test]
    fn check_sentence_with_kiem_tra_ca_dau_cau_va_spelling() {
        let issues = check_sentence_with("tôi quẫn quá.", &TuDienRong);
        let has_lowercase = issues
            .iter()
            .any(|i| i.kind == IssueKind::LowercaseSentenceStart);
        let has_spelling = issues.iter().any(|i| i.kind == IssueKind::Spelling);
        assert!(has_lowercase, "phải bắt lỗi chữ thường ở đầu câu");
        assert!(has_spelling, "phải bắt lỗi chính tả từ điển");
    }

    #[test]
    fn spelling_khong_nhan_doi_tren_tu_da_bat_diacritic_typo() {
        let issues = check_with("Tôi nghiã thế.", &TuDienRong);
        let on_nghia: Vec<_> = issues
            .iter()
            .filter(|i| i.span.start == 4 && i.span.end == 9)
            .collect();
        assert_eq!(
            on_nghia.len(),
            1,
            "từ đã bắt DiacriticTypo không được báo chồng thêm Spelling"
        );
        assert_eq!(on_nghia[0].kind, IssueKind::DiacriticTypo);
    }

    #[test]
    fn abbrev_khong_nuot_tu_2_chu_cai_va_dau_cham_ngoac_kep() {
        // "em." / "ai." là kết thúc câu thật, không phải từ viết tắt.
        let issues = check_sentence("Chào em.Anh đi đây.");
        assert!(issues.iter().any(|i| i.kind == IssueKind::SpaceAfterPunct));
        let issues2 = check_sentence("Anh yêu em. chúc em ngủ ngon.");
        assert!(issues2
            .iter()
            .any(|i| i.kind == IssueKind::LowercaseSentenceStart));
        // Dấu hỏi/chấm than trong ngoặc kép không phải kết thúc câu ngoài.
        assert!(check_sentence("Anh hỏi \"Sao thế?\" rồi đi tiếp.").is_empty());
        // Dấu ba chấm tách khoảng trắng không báo thừa khoảng trắng trước dấu chấm.
        assert!(check_sentence("Tôi nghĩ ... bạn nói đúng.").is_empty());
    }
}
