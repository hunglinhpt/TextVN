// SPDX-License-Identifier: GPL-3.0-or-later
//! AS-060 — **EN spelling** qua OS spell-check API (vòng 37): chỉ gợi ý.
//!
//! Luật tiếng Anh chạy CHỈ khi câu En-dominant (`ctx.lang == Lang::En`) và
//! CHỈ khi có provider từ điển OS (qua [`Budgeted`] — [`NoDictionary`] im lặng
//! hoàn toàn). Mục tiêu: bắt lỗi gõ sai thật ("recieve", "seperate",
//! "adress") với **FPR bar chặt** nhờ 4 tầng gate:
//!
//! 1. **Allowlist nội bộ**: từ nằm trong `en_freq3000` (3000 từ tần suất +
//!    domain văn phòng/tech/business vòng 37) → đúng, khỏi tốn ngân sách OS.
//! 2. **Cổng hình thái**: chỉ token ASCII thuần, độ dài ≥ 4; bỏ ALL-CAPS
//!    (acronym "API", "HTTP"), chữ hoa giữa từ ("iPhone" — tên riêng), và
//!    âm tiết Việt không dấu (`is_vietnamese_ascii_word` — "phong", "bao"
//!    trong câu mix không bị báo).
//! 3. **Cổng khoảng cách**: CHỈ báo khi OS gợi ý ≥ 1 từ cách ≥ 1 và ≤ 2 bước
//!    Damerau-Levenshtein (sửa + xoá + chèn + hoán 2 chữ cái). Từ lạ không
//!    có gợi ý gần ("asdfgh") → im lặng.
//! 4. **Ngân sách**: [`Budgeted`] `DEFAULT_SUGGEST_BUDGET` lần suggest/câu —
//!    hết ngân sách → không có gợi ý gần → luật im lặng (fail-open).
//!
//! Kind dùng lại [`IssueKind::Spelling`] (không thêm kind → không phải sync
//! 6 điểm FFI), chỉ-gợi-y (`confidence = 60`, `auto_fixable = false`) — người
//! dùng bấm ứng viên Mode 1 để sửa (cùng đường với luật spelling tiếng Việt).

use super::language::{en_freq_words, is_vietnamese_ascii_word, Lang};
use super::{issue_suggestion, Ctx};
use crate::dict::{Budgeted, Dictionary};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Từ ngắn hơn ngưỡng này bỏ qua (từ 3 ký tự FPR quá cao: tên riêng, viết tắt).
const MIN_WORD_CHARS: usize = 4;

/// Số gợi ý OS lấy cho một từ.
const MAX_SUGGESTIONS: usize = 3;

/// Khoảng cách Damerau-Levenshtein tối đa coi là "gần" (gõ sai nhẹ).
const MAX_DISTANCE: usize = 2;

/// Khoảng cách Damerau-Levenshtein (thuật toán OSA — sửa chèn/xoá/thay + hoán
/// vị 2 chữ cái liền: "recieve"→"receive" = 1). Trả `MAX_DISTANCE + 1` khi
/// chênh lệch độ dài đã vượt ngưỡng (bỏ qua tính ma trận).
fn damerau_distance(a: &[char], b: &[char]) -> usize {
    let (n, m) = (a.len(), b.len());
    if n.abs_diff(m) > MAX_DISTANCE {
        return MAX_DISTANCE + 1;
    }
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, val) in d[0].iter_mut().enumerate() {
        *val = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = best;
        }
    }
    d[n][m]
}

/// Báo từ EN gõ sai chính tả (chỉ gợi ý — cần OS provider + gợi ý gần).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Chỉ câu En-dominant — bài học article_an R2-28: không bao giờ đụng câu
    // Việt (luật spelling tiếng Việt đã lo phần đó).
    if ctx.lang != Lang::En {
        return;
    }
    let dict = Budgeted::new(ctx.dict, super::DEFAULT_SUGGEST_BUDGET);
    // Vòng 46 (Pass E2 perf): index "span đã bêu" O(n log n) MỘT lần —
    // out.iter().any() mỗi Word token = O(n²) khi luật trước để lại nhiều
    // issue ("then then..." — dup/phrase để lại 32k span).
    // Snapshot cố định ĐỦ vì span của chính rule này là từng-từ-rời nhau
    // (không bao giờ tự che token kế — words rời nhau).
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
        // các span bắt đầu <= start — max end trong số đó
        prefix_max_end[idx] >= end
    };
    for tok in &ctx.tokens {
        if tok.kind != TokenKind::Word || ctx.is_masked(tok.start, tok.end) {
            continue;
        }
        if tok.len() < MIN_WORD_CHARS {
            continue;
        }
        // Không chồng issue lên span luật trước đã bêu (dup/diacritic/confusion…).
        if covered_by_prior(tok.start, tok.end) {
            continue;
        }
        let slice = ctx.token_chars(tok);
        // Chỉ từ ASCII thuần: chữ Việt có dấu / số / gạch nối → luật khác lo.
        if !slice.iter().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        // ALL-CAPS = acronym ("API", "HTTP"); chữ hoa giữa từ = tên riêng
        // ("iPhone", "LinkedIn") — cả hai không bao giờ báo.
        let upper = slice.iter().filter(|c| c.is_ascii_uppercase()).count();
        if upper == slice.len() || slice[1..].iter().any(|c| c.is_uppercase()) {
            continue;
        }
        let lower: String = slice.iter().map(|c| c.to_ascii_lowercase()).collect();
        // Âm tiết Việt không dấu trong câu mix ("phong", "bao") → không phải
        // từ EN cần soát chính tả.
        if is_vietnamese_ascii_word(&lower) {
            continue;
        }
        // Allowlist nội bộ: từ đã biết (tần suất + domain) → đúng, không tốn
        // ngân sách gọi OS.
        if en_freq_words().contains(lower.as_str()) {
            continue;
        }
        // Provider vắng mặt (NoDictionary) → contains luôn true → im lặng.
        if dict.contains(&lower) {
            continue;
        }
        // Cổng FPR quyết định: CHỈ báo khi OS gợi ý ≥ 1 từ cách ≤ 2 bước.
        let chars: Vec<char> = lower.chars().collect();
        let suggestions: Vec<String> = dict
            .suggest(&lower, MAX_SUGGESTIONS)
            .into_iter()
            .filter(|s| {
                s.to_lowercase() != lower
                    && damerau_distance(&chars, &s.chars().collect::<Vec<char>>()) <= MAX_DISTANCE
            })
            .collect();
        if suggestions.is_empty() {
            continue; // không có gợi ý gần → fail-open, không báo động giả
        }
        let word: String = slice.iter().collect();
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
    use super::{check, damerau_distance, Lang, MAX_DISTANCE};
    use crate::dict::Dictionary;
    use crate::rules::Ctx;
    use crate::{Issue, IssueKind};
    use std::collections::HashSet;

    /// Từ điển EN giả cho test: `contains` theo tập, `suggest` = danh sách
    /// gợi ý mô phỏng OS (trả về cho mọi từ lạ, không phân biệt tiền tố —
    /// gate khoảng cách của luật sẽ tự lọc).
    struct FakeEnDictionary {
        words: HashSet<&'static str>,
        hints: Vec<&'static str>,
    }

    impl Dictionary for FakeEnDictionary {
        fn contains(&self, word: &str) -> bool {
            self.words.contains(word)
        }

        fn suggest(&self, _word: &str, limit: usize) -> Vec<String> {
            self.hints
                .iter()
                .take(limit)
                .map(|s| (*s).to_string())
                .collect()
        }
    }

    fn base_dict() -> FakeEnDictionary {
        FakeEnDictionary {
            words: HashSet::from([
                "the", "report", "i", "will", "send", "you", "receive", "our", "team", "for",
                "meeting", "asap", "please", "check", "file", "and", "of", "a", "this", "is",
                "with", "on", "in", "to", "deadline", "deploy",
            ]),
            hints: Vec::new(),
        }
    }

    fn check_text(text: &str, dict: &FakeEnDictionary) -> Vec<Issue> {
        let ctx = Ctx::new_with_dict(text, false, dict);
        let mut issues = Vec::new();
        check(&ctx, &mut issues);
        issues
    }

    /// Câu En-dominant chuẩn cho test (≥ 2 từ chức năng EN).
    const EN_SENTENCE: &str = "I will send you the recieve report of our team for the meeting";

    #[test]
    fn typo_gan_dict_bi_bao_voi_goi_y() {
        let mut dict = base_dict();
        dict.hints = vec!["receive"];
        let issues = check_text(EN_SENTENCE, &dict);
        assert_eq!(issues.len(), 1, "chỉ \"recieve\" bị báo");
        let issue = &issues[0];
        assert_eq!(issue.kind, IssueKind::Spelling);
        assert!(!issue.auto_fixable, "chỉ gợi ý — sửa qua ứng viên Mode 1");
        assert_eq!(issue.confidence, 60);
        assert_eq!(issue.fix, None);
        assert_eq!(issue.suggestions, vec!["receive".to_string()]);
        // Span đúng token "recieve" (offset ký tự).
        let start = EN_SENTENCE.find("recieve").unwrap();
        assert_eq!(issue.span.start, start);
        assert_eq!(issue.span.end, start + "recieve".len());
    }

    #[test]
    fn goi_y_xa_khong_bao() {
        let mut dict = base_dict();
        // OS gợi ý nhưng quá xa khoảng cách 2 → không báo (FPR gate).
        dict.hints = vec!["authentication"];
        let issues = check_text(EN_SENTENCE, &dict);
        assert!(issues.is_empty(), "gợi ý xa không được báo");
    }

    #[test]
    fn khong_co_goi_y_thi_im_lang() {
        let mut dict = base_dict();
        dict.hints = vec![]; // OS khônguggest gì (từ lạ hoàn toàn)
        let issues = check_text(EN_SENTENCE, &dict);
        assert!(issues.is_empty(), "fail-open: không gợi ý gần → im lặng");
    }

    #[test]
    fn tu_trong_allowlist_khong_bao() {
        // "deadline" nằm trong allowlist en_freq3000 (dù dict giả cố tình bỏ)
        // → không bao giờ được báo.
        let mut dict = base_dict();
        dict.words.remove("deadline");
        dict.hints = vec!["deadline"];
        let text = "please check the deadline of our team";
        let issues = check_text(text, &dict);
        assert!(issues.is_empty(), "\"deadline\" ∈ allowlist → im lặng");
    }

    #[test]
    fn all_caps_va_chu_hoa_giua_khong_bao() {
        let mut dict = base_dict();
        dict.hints = vec!["receive"];
        // "RECEIVE" = ALL-CAPS (acronym nghi ngờ), "reCEIVE" = chữ hoa giữa
        // (tên riêng) — cả hai bỏ qua dù dict không có.
        let issues = check_text(
            "I will send you the RECEIVE report and the reCEIVE file of our team",
            &dict,
        );
        assert!(issues.is_empty());
    }

    #[test]
    fn am_tiet_viet_khong_dau_khong_bao() {
        let mut dict = base_dict();
        dict.hints = vec!["phone"];
        // "phong" là âm tiết Việt không dấu — kể cả dict không có và gợi ý gần
        // cũng không báo (câu mix En-dominant).
        let issues = check_text("please check the phong of our team", &dict);
        assert!(issues.is_empty());
    }

    #[test]
    fn tu_ngan_duoi_4_ky_tu_khong_bao() {
        let mut dict = base_dict();
        dict.hints = vec!["the"];
        let issues = check_text("please check the tmv file of our team", &dict);
        assert!(issues.is_empty(), "\"tmv\" 3 ký tự → bỏ qua");
    }

    #[test]
    fn cau_tieng_viet_khong_chay_luat() {
        let dict = base_dict();
        // Câu Việt-dominant → lang = Vi → luật EN im lặng hoàn toàn.
        let ctx = Ctx::new_with_dict(
            "Tôi sẽ nhận được báo cáo recieve từ nhóm chúng ta",
            false,
            &dict,
        );
        assert_eq!(ctx.lang, Lang::Vi);
        let mut issues = Vec::new();
        check(&ctx, &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn no_dictionary_im_lang_hoan_toan() {
        let ctx = Ctx::new("I will send you the recieve report", false);
        let mut issues = Vec::new();
        check(&ctx, &mut issues);
        assert!(issues.is_empty(), "NoDictionary → không bao giờ báo");
    }

    #[test]
    fn het_ngan_sach_im_lang_khong_bao_dong_gia() {
        // 3 từ sai nhưng ngân sách suggest = 2: từ thứ 3 hết ngân sách →
        // không có gợi ý gần → im lặng (fail-open, khác luật VI vẫn báo).
        let mut dict = base_dict();
        dict.words.remove("adress");
        dict.words.remove("seperate");
        dict.hints = vec!["receive", "address", "separate"];
        let text = "please check the recieve file and adress and seperate of our team";
        let issues = check_text(text, &dict);
        // "recieve" (gọi 1) + "adress" (gọi 2) dùng hết ngân sách; "seperate"
        // hết ngân sách → không có gợi ý → im lặng.
        assert_eq!(
            issues.len(),
            2,
            "ngân sách 2 lần suggest → chỉ 2 issue có gợi ý"
        );
        assert_eq!(issues[0].suggestions, vec!["receive".to_string()]);
        assert_eq!(issues[1].suggestions, vec!["address".to_string()]);
    }

    #[test]
    fn damerau_tinh_dung() {
        let c = |s: &str| s.chars().collect::<Vec<_>>();
        assert_eq!(damerau_distance(&c("recieve"), &c("receive")), 1); // hoán vị
        assert_eq!(damerau_distance(&c("adress"), &c("address")), 1); // thiếu chữ
        assert_eq!(damerau_distance(&c("seperate"), &c("separate")), 1); // hoán vị
        assert_eq!(damerau_distance(&c("teh"), &c("the")), 1); // hoán vị
        assert_eq!(damerau_distance(&c("kitten"), &c("sitting")), 3); // chuẩn OSA
        assert_eq!(
            damerau_distance(&c("abc"), &c("abcdefgh")),
            MAX_DISTANCE + 1
        ); // lệch dài → cắt sớm
        assert_eq!(damerau_distance(&c("deploy"), &c("deploy")), 0);
    }
}
