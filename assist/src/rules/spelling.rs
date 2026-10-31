// SPDX-License-Identifier: GPL-3.0-or-later
//! AS-040 — từ lạ **không có trong từ điển** (chỉ gợi ý).
//!
//! Luật **chỉ gợi ý** (không tự sửa — Fassist-90): từ tiếng Việt có dấu mà từ
//! điển không nhận là có thể sai chính tả → báo với `confidence = 60`, để
//! người dùng quyết định. Khi chưa nạp từ điển ([`NoDictionary`](crate::dict::NoDictionary))
//! luật này im lặng hoàn toàn — `check`/`check_sentence` giữ hành vi cũ.
//!
//! Bỏ qua để không báo động giả:
//! - từ **< 3 ký tự** ("ơi", "eshop" rút gọn…) — trùng tên riêng/rút gọn quá dễ;
//! - từ có **chữ hoa bên trong** ("iPhone") — tên riêng sản phẩm;
//! - từ **không chứa chữ có dấu đặc thù tiếng Việt** — từ tiếng Anh ("pro",
//!   "test"); câu tiếng Anh thuần đã bị chặn ở L0, luật này chặn từ tiếng Anh
//!   lẻ tẻ trong câu tiếng Việt.

use super::{issue_suggestion, Ctx};
use crate::dict::{Budgeted, Dictionary};
use crate::rules::language::is_vietnamese_letter;
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Từ ngắn hơn ngưỡng này bỏ qua (độ dài theo ký tự Unicode).
const MIN_WORD_CHARS: usize = 3;

/// Số gợi ý tối đa lấy từ từ điển cho một từ lạ.
const MAX_SUGGESTIONS: usize = 3;

/// Báo từ tiếng Việt lạ không có trong từ điển (chỉ gợi ý, không tự sửa).
///
/// Ngân sách `suggest` không đếm thủ công tại đây nữa — toàn bộ nằm trong bộ
/// bọc [`Budgeted`](crate::dict::Budgeted) (P4-10 §5.3), gồm:
///
/// - giới hạn số lần gọi;
/// - đo tổng thời gian đã dùng cho `suggest`;
/// - hết ngân sách → gợi ý rỗng (**fail-open**) — từ lạ vẫn được báo bình
///   thường, chỉ thiếu gợi ý kèm theo.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Ngan sach suggest cho mot cau (D40, Fassist-105): provider OS ton
    // 3,1–6,9 ms/tu cho Suggest (P4-10 finding F3) nen chi goi cho it tu la
    // nhat; tu sau do van bao loi nhung khong kem goi y (fail-open).
    let dict = Budgeted::new(ctx.dict, super::DEFAULT_SUGGEST_BUDGET);
    // Vòng 46 (Pass E2 perf): index span O(n log n) một lần — như en_spelling.
    let mut covered: Vec<(usize, usize)> = out.iter().map(|i| (i.span.start, i.span.end)).collect();
    covered.sort_unstable();
    let mut prefix_max_end: Vec<usize> = Vec::with_capacity(covered.len() + 1);
    let mut running = 0usize;
    for &(_, e) in &covered {
        prefix_max_end.push(running.max(e));
        running = running.max(e);
    }
    prefix_max_end.insert(0, 0);
    let covered_by_prior = |start: usize, end: usize| -> bool {
        let idx = covered.partition_point(|&(s, _)| s <= start);
        prefix_max_end[idx] >= end
    };
    for tok in &ctx.tokens {
        if tok.kind != TokenKind::Word || ctx.is_masked(tok.start, tok.end) {
            continue;
        }
        if tok.len() < MIN_WORD_CHARS {
            continue;
        }
        // Không báo trùng trên token đã được luật trước (như `diacritic_typo`) bắt,
        // tránh nhân đôi issue trên cùng span và lãng phí ngân sách suggest OS.
        if covered_by_prior(tok.start, tok.end) {
            continue;
        }
        let word = ctx.token_text(tok);
        // Chữ hoa nằm giữa từ ("iPhone") → tên riêng, bỏ qua. Ký tự đầu được
        // phép hoa: đầu câu, hoặc tên riêng một âm tiết ("Hà", "Bách").
        if word.chars().skip(1).any(|c| c.is_uppercase()) {
            continue;
        }
        // Không có chữ có dấu đặc thù tiếng Việt → từ tiếng Anh, bỏ qua.
        if !word.chars().any(is_vietnamese_letter) {
            continue;
        }
        // Vec<char> offset → so từ điển phải lowercase (trait đổi chuẩn hoá).
        let lower = word.to_lowercase();
        if dict.contains(&lower) {
            continue;
        }
        // Ngan sach suggest nam trong `Budgeted`: het ngan sach -> gợi ý rỗng
        // (fail-open), toan bo loi spelling van duoc bao o duoi.
        let suggestions = dict.suggest(&lower, MAX_SUGGESTIONS);
        out.push(issue_suggestion(
            IssueKind::Spelling,
            Span {
                start: tok.start,
                end: tok.end,
            },
            format!("Có thể sai chính tả: \"{word}\""),
            suggestions,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::dict::Dictionary;
    use crate::rules::Ctx;
    use crate::{Issue, IssueKind};
    use std::collections::HashSet;

    /// Từ điển giả: vài từ tiếng Việt hợp lệ; gợi ý = các từ cùng chữ cái đầu.
    struct FakeDictionary(HashSet<&'static str>);

    impl Dictionary for FakeDictionary {
        fn contains(&self, word: &str) -> bool {
            self.0.contains(word)
        }

        fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
            let first = word.chars().next().unwrap_or('\0');
            let mut out: Vec<String> = self
                .0
                .iter()
                .filter(|w| w.starts_with(first))
                .map(|w| (*w).to_string())
                .collect();
            out.sort();
            out.truncate(limit);
            out
        }
    }

    fn check_text(text: &str, dict: &FakeDictionary) -> Vec<Issue> {
        let ctx = Ctx::new_with_dict(text, false, dict);
        let mut issues = Vec::new();
        check(&ctx, &mut issues);
        issues
    }

    #[test]
    fn tu_la_khong_co_trong_tu_dien_bi_bao() {
        let dict = FakeDictionary(HashSet::from(["tôi", "dùng", "nhé", "xinh", "xiên"]));
        let issues = check_text("tôi dùng xún nhé", &dict);
        assert_eq!(issues.len(), 1, "chỉ từ \"xún\" lạ");
        let issue = &issues[0];
        assert_eq!(issue.kind, IssueKind::Spelling);
        assert!(!issue.auto_fixable, "chỉ gợi ý, không tự sửa");
        assert_eq!(issue.confidence, 60);
        assert_eq!(issue.fix, None);
        assert_eq!(issue.message, "Có thể sai chính tả: \"xún\"".to_string());
        // Span đúng vị trí token "xún" (offset ký tự Unicode).
        assert_eq!(issue.span.start, 9);
        assert_eq!(issue.span.end, 12);
        // Gợi ý lấy từ từ điển (cùng chữ cái đầu "x").
        assert!(!issue.suggestions.is_empty());
        assert!(issue.suggestions.iter().all(|s| s.starts_with("x")));
    }

    #[test]
    fn tu_hop_le_khong_bi_bao() {
        let dict = FakeDictionary(HashSet::from(["tôi", "đi", "học", "ở", "trường"]));
        let issues = check_text("Tôi đi học ở trường.", &dict);
        assert!(issues.is_empty());
    }

    #[test]
    fn tu_tieng_anh_va_ten_rieng_khong_bi_bao() {
        let dict = FakeDictionary(HashSet::from(["tôi", "dùng", "nhé"]));
        // "pro" = từ tiếng Anh không dấu; "iPhone" = chữ hoa bên trong
        // (tên riêng); cả hai phải được bỏ qua.
        let issues = check_text("tôi dùng pro nhé iPhone", &dict);
        assert!(issues.is_empty());
    }

    #[test]
    fn tu_ngan_duoi_3_ky_tu_khong_bi_bao() {
        let dict = FakeDictionary(HashSet::from(["đẹp"]));
        // "ơi" chỉ 2 ký tự — báo là động giả (tên riêng/rút gọn).
        let issues = check_text("ơi đẹp", &dict);
        assert!(issues.is_empty());
    }

    #[test]
    fn no_dictionary_khong_bao_gi() {
        let ctx = Ctx::new("tôi quẫn quá", false);
        let mut issues = Vec::new();
        check(&ctx, &mut issues);
        assert!(issues.is_empty(), "NoDictionary coi mọi từ là đúng");
    }

    #[test]
    fn ngan_sach_suggest_het_thi_tu_la_van_duoc_bao_khong_kem_goi_y() {
        // 3 từ lạ ("xún", "zây", "mền") nhưng ngân sách mặc định
        // DEFAULT_SUGGEST_BUDGET = 2 → chỉ 2 từ đầu có gợi ý (từ điển có từ
        // cùng chữ cái đầu "mều" cho "mền"); từ thứ 3 vẫn bị báo lỗi nhưng
        // suggestions rỗng (fail-open).
        let dict = FakeDictionary(HashSet::from(["tôi", "nhé", "xa", "xện", "zịa", "mều"]));
        let issues = check_text("tôi xún zây mền nhé", &dict);
        assert_eq!(
            issues.len(),
            3,
            "cả 3 từ lạ đều bị báo, kể cả khi hết ngân sách suggest"
        );
        assert_eq!(
            issues[0].suggestions,
            vec!["xa".to_string(), "xện".to_string()],
            "từ đầu tiên còn ngân sách → có gợi ý"
        );
        assert_eq!(issues[1].suggestions, vec!["zịa".to_string()]);
        assert!(
            issues[2].suggestions.is_empty(),
            "hết ngân sách → gợi ý rỗng (fail-open)"
        );
        assert_eq!(issues[2].message, "Có thể sai chính tả: \"mền\"");
        assert_eq!(issues[2].span.start, 12);
        assert_eq!(issues[2].span.end, 15);
    }

    #[test]
    fn tu_ben_trong_url_da_bat_khong_bi_bao() {
        let dict = FakeDictionary(HashSet::from(["tôi", "nhé", "xinh"]));
        let issues = check_text("tôi xem https://viet-nam.vn/abc nhé", &dict);
        assert!(issues.is_empty(), "nội dung URL/email phải được che");
    }
}
