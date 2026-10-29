// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 39 (audit persona văn phòng) — lặp **cụm 2 từ** liền kề (X Y X Y):
//! "Vui lòng vui lòng kiểm tra giúp em", "Đơn hàng đã được đã được gửi đi".
//!
//! Khác [`super::duplicate_word`] (1 từ lặp), dạng này chữ nào cũng khác nhau
//! nên lọt qua mọi luật cũ. Fix = xoá bản sao thứ hai (kể cả dấu cách trước).
//!
//! FPR bar (không bao giờ bêu chữ đúng) — các dạng lặp HỢP LỆ đều được chặn:
//! - mệnh lệnh nhắc lại có trợ từ sau ("ăn cơm ăn cơm đi", "về nhà về nhà nhé")
//!   hoặc kết thúc câu ("Nhanh lên nhanh lên!") — kiểu nói, không phải lỗi;
//! - xã giao lặp idiomatic ("xin chào xin chào", "cảm ơn cảm ơn", "làm ơn
//!   làm ơn", "xin lỗi xin lỗi", "chúc mừng chúc mừng");
//! - đếm số ("một hai một hai"), hát/tượng thanh ("la la la la" — whitelist
//!   từ đơn của `duplicate_word`);
//! - nhấn mạnh kiểu lặp láy đã nằm trong `vi_fixed_phrases.txt` ("lặng lẽ
//!   lặng lẽ", "liên tục liên tục") → bị `drop_issues_in_protected` gỡ;
//! - EN: cụm 2 từ lặp gần như luôn chứa từ chức năng ("have to have to",
//!   "going to going to" — nhấn mạnh hợp lệ) → bỏ qua khi có từ chức năng;
//! - chữ HOA toàn phần ("LẦM LẦM LẦM LẦM"), vùng che (URL/…) và vùng
//!   được-bảo-vệ (từ láy, thành ngữ) không báo.

use super::capitalize_sentence::looks_like_code_or_list;
use super::duplicate_word::{in_quotes, EN_INTENTIONAL_REPEATS, INTENTIONAL_REPEATS};
use super::{is_word, Ctx};
use crate::tokenize::TokenKind;
use crate::{Issue, IssueKind, Span};

/// Cụm 2 từ lặp có chủ ý (xã giao/lời thoại) — KHÔNG phải lỗi.
const PHRASE_WHITELIST: &[&str] = &[
    "xin chào",
    "cảm ơn",
    "làm ơn",
    "xin lỗi",
    "chúc mừng",
    "thank you",
    "thanks",
    "sorry",
    "please",
];

/// Trợ từ/từ hỏi/số từ/từ chỉ hướng — câu lặp có từ này là kiểu nói hợp lệ
/// ("phải không phải không", "một hai một hai", "ăn cơm ăn cơm đi").
const VI_SKIP_WORDS: &[&str] = &[
    // hỏi + lịch sự
    "không", "gì", "sao", "đâu", "nhé", "nhá", "chứ", "nhỉ", "nào", "thôi", "cứ", "đấy", "này",
    "kia", "ấy", "đó", "ư", "ha", "à", // mệnh lệnh/hướng
    "đi", "lên", "xuống", "ra", "vào", "xem", "thử", // tình thái/thời gian
    // ("sẽ" KHÔNG skip — "sẽ xác nhận lại sẽ xác nhận lại" là lỗi thật, RH-1)
    "rồi", "nữa", "vẫn", "cũng", "đều", "còn", "chưa", "thì", "mà", "là", "của", "và", "nhưng",
    "hoặc", "vì", "nếu", "dù", "tuy", "lắm", "quá", "thế", // số từ đếm
    "một", "hai", "ba", "bốn", "sáu", "bảy", "tám", "chín", "mười",
];

/// Từ chức năng tiếng Anh — cụm 2 từ EN lặp ("have to have to", "there is
/// there is"…) gần như luôn là ngữ pháp/nhấn mạnh, không phải lỗi gõ.
const EN_SKIP_WORDS: &[&str] = &[
    "the", "a", "an", "of", "to", "in", "on", "at", "for", "with", "and", "or", "but", "is", "are",
    "was", "were", "be", "been", "am", "do", "does", "did", "have", "has", "had", "will", "would",
    "can", "could", "should", "must", "may", "might", "that", "this", "these", "those", "it",
    "its", "not", "there", "here",
];

/// Trợ từ mệnh lệnh đứng NGAY SAU cụm lặp ("ăn cơm ăn cơm **đi**") = lời nói
/// nhắc lại có chủ ý, không phải lỗi.
const IMPERATIVE_AFTER: &[&str] = &["đi", "nhé", "nhá", "nào", "thôi", "cứ", "đấy", "xem", "thử"];

/// Bat loi lap cum lien ke: 2 tu (X Y X Y), 3 tu (X Y Z X Y Z) hoac 4 tu
/// (X Y Z T X Y Z T).
///
/// Vòng 40 (R2 persona RH-1): lặp CỤM ≥ 3 TỪ ("đã hoàn thành đã hoàn thành",
/// "sẽ xác nhận lại sẽ xác nhận lại") là dạng paste-double phổ biến nhất ở
/// văn phòng — chỉ quét cửa sổ 2 từ là lọt. Các cụm lồng nhau ("thanh toán"
/// ⊂ "đã thanh toán đã thanh toán") được dedup giữ cụm DÀI nhất.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    if looks_like_code_or_list(ctx) {
        return;
    }
    let mut found: Vec<Issue> = Vec::new();
    for window_size in [2usize, 3usize, 4usize] {
        // Cửa sổ gồm 2*ws từ xen 2*ws-1 dấu cách → 4*ws-1 token, kết thúc tại `i`.
        let span_tokens = 4 * window_size - 1;
        if ctx.tokens.len() < span_tokens {
            continue;
        }
        for i in (span_tokens - 1)..ctx.tokens.len() {
            check_window(ctx, i, window_size, &mut found);
        }
    }
    // Cửa sổ 2 từ lồng trong cửa sổ 3 từ ("thanh toán" ⊂ "đã thanh toán đã
    // thanh toán") — giữ bản DÀI (báo 1 lỗi, không double-report).
    let keep: Vec<bool> = found
        .iter()
        .enumerate()
        .map(|(ai, a)| {
            !found
                .iter()
                .enumerate()
                .any(|(bi, b)| bi != ai && a.span.start >= b.span.start && a.span.end <= b.span.end)
        })
        .collect();
    out.extend(
        found
            .into_iter()
            .zip(keep)
            .filter(|(_, k)| *k)
            .map(|(i, _)| i),
    );
}

/// Kiểm tra cụm lặp `window_size` từ KẾT THÚC tại token `i` (bản sao 2).
fn check_window(ctx: &Ctx, i: usize, window_size: usize, out: &mut Vec<Issue>) {
    let span_tokens = 4 * window_size - 1;
    let first = i + 1 - span_tokens;
    let word_idx: Vec<usize> = (0..2 * window_size).map(|k| first + 2 * k).collect();
    let space_idx: Vec<usize> = (0..2 * window_size - 1)
        .map(|k| first + 2 * k + 1)
        .collect();
    let single_space = |k: usize| {
        let t = &ctx.tokens[k];
        t.kind == TokenKind::Space && t.len() == 1 && ctx.chars[t.start] == ' '
    };
    if !word_idx.iter().all(|&k| is_word(&ctx.tokens[k]))
        || !space_idx.iter().copied().all(single_space)
    {
        return;
    }
    let texts: Vec<String> = word_idx
        .iter()
        .map(|&k| ctx.token_text(&ctx.tokens[k]))
        .collect();
    // Mỗi từ ≥ 2 ký tự ("à ơi à ơi" hát — không đụng).
    if texts.iter().any(|t| t.chars().count() < 2) {
        return;
    }
    let half = window_size;
    let lower = |t: &str| t.to_lowercase();
    let ph1 = texts[..half]
        .iter()
        .map(|t| lower(t))
        .collect::<Vec<_>>()
        .join(" ");
    let ph2 = texts[half..]
        .iter()
        .map(|t| lower(t))
        .collect::<Vec<_>>()
        .join(" ");
    if ph1 != ph2 {
        return;
    }
    // HOA toàn phần = nhấn mạnh/đầu thư, không phải lỗi gõ.
    if texts
        .iter()
        .all(|t| t.chars().next().is_some_and(|c| c.is_uppercase()))
    {
        return;
    }
    let words: Vec<&str> = ph1.split(' ').collect();
    // Cụm xã giao idiomatic.
    if PHRASE_WHITELIST.contains(&ph1.as_str()) {
        return;
    }
    // Whitelist từ đơn + từ hỏi/số/tình trạng: lặp chứa từ này là kiểu nói.
    let skipped = words.iter().any(|w| {
        let ascii = w.chars().all(|c| c.is_ascii_alphabetic());
        let base: &[&str] = if ascii { EN_SKIP_WORDS } else { VI_SKIP_WORDS };
        let intentional: &[&str] = if ascii {
            EN_INTENTIONAL_REPEATS
        } else {
            INTENTIONAL_REPEATS
        };
        base.contains(w) || intentional.contains(w)
    });
    if skipped {
        return;
    }
    let start = ctx.tokens[word_idx[half]].start;
    let end = ctx.tokens[word_idx[2 * half - 1]].end;
    if in_quotes(ctx, start) || ctx.is_masked(start, end) {
        return;
    }
    // Kết thúc câu ngay sau bản sao 2 = lặp kiểu nói ("Nhanh lên nhanh lên!")
    // — chỉ bắt khi sau cụm còn từ nữa, HOẶC câu kết bằng DẤU CHẤM (Pass A
    // T6: paste-double cuối câu "đã hoàn thành đã hoàn thành." rất phổ biến;
    // guard chỉ cần miễn cho "!"/"?" — lặp kiểu nói kết bằng chấm hiếm).
    let has_word_after = ctx.tokens[i + 1..].iter().any(is_word);
    if !has_word_after {
        let ends_with_dot = ctx
            .chars
            .iter()
            .rev()
            .find(|c| !c.is_whitespace())
            .is_some_and(|c| *c == '.');
        if !ends_with_dot {
            return;
        }
    }
    // Trợ từ mệnh lệnh ngay sau ("ăn cơm ăn cơm đi nhé") = lời nhắc lại.
    if let Some(next) = ctx.tokens[i + 1..].iter().find(|t| is_word(t)) {
        if IMPERATIVE_AFTER.contains(&ctx.token_text(next).to_lowercase().as_str()) {
            return;
        }
    }
    // Vòng 40 (R2-T1): SUGGESTION-ONLY — lặp cụm nhấn mạnh là ngữ pháp sống
    // ("ngày mai ngày mai", "chờ đợi chờ đợi", "em ơi em ơi") mà whitelist
    // đóng không thể bao quát; auto-xoá cụm 2-4 từ khi đoán sai là xoá CHỮ
    // CỦA NGƯỜI DÙNG. conf=60, không fix — người dùng quyết (xoá bằng popup).
    out.push(super::issue_suggestion(
        IssueKind::DuplicateWord,
        Span { start, end },
        format!("Lặp cụm từ: \"{ph1}\""),
        vec![texts[..half].join(" ")],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_lap_cum_2_tu_suggestion_only() {
        // R2-T1: suggestion-only — KHÔNG tự xoá (lặp nhấn mạnh là ngữ pháp sống).
        let issues = check("Vui lòng vui lòng kiểm tra giúp em.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DuplicateWord);
        assert!(!issues[0].auto_fixable);
        assert!(issues[0].fix.is_none());
        let text = "Vui lòng vui lòng kiểm tra giúp em.";
        assert_eq!(crate::apply_auto_fixes(text, &issues), text);
    }

    #[test]
    fn bat_lap_cum_trung_vet() {
        assert_eq!(check("Đơn hàng đã được đã được gửi đi rồi.").len(), 1);
        assert_eq!(check("anh vui lòng vui lòng gửi lại hóa đơn").len(), 1);
        assert_eq!(check("bạn vui lòng vui lòng check giúp").len(), 1);
    }

    #[test]
    fn khong_bat_lenh_nhac_lai_co_tro_tu() {
        assert!(check("ăn cơm ăn cơm đi nhé").is_empty());
        assert!(check("về nhà về nhà nào").is_empty());
    }

    /// Vòng 40 (RH-1, R2 persona): lặp CỤM 3 TỪ (X Y Z X Y Z).
    #[test]
    fn bat_lap_cum_3_tu() {
        assert_eq!(check("Em đã hoàn thành đã hoàn thành bài tập.").len(), 1);
        assert_eq!(
            check("Quý khách đã thanh toán đã thanh toán thành công.").len(),
            1
        );
        assert_eq!(check("Tôi sẽ xác nhận lại sẽ xác nhận lại sau.").len(), 1);
        assert_eq!(check("Shop đã xác nhận đã xác nhận đơn hàng.").len(), 1);
        // Suggestion-only: auto-fix KHÔNG đụng text.
        let text = "Em đã hoàn thành đã hoàn thành bài tập.";
        assert_eq!(crate::apply_auto_fixes(text, &check(text)), text);
    }

    #[test]
    fn bat_cum_4_tu_sat_bien_anh_rui() {
        // LH: trùng với assist/tests/abnormal.rs — lần chót xác nhận cùng lib.
        assert_eq!(check("anh đã xác nhận lại đã xác nhận lại rồi").len(), 1);
    }

    #[test]
    fn khong_bat_cum_3_tu_hop_le() {
        // Xã giao/idiomatic vẫn được guard chặn ở cửa sổ 3 từ.
        assert!(check("một hai ba một hai ba").is_empty());
        assert!(check("Cô ấy đi nhanh lắm nhanh lắm").is_empty());
    }

    #[test]
    fn khong_bat_xa_giao_idiomatic() {
        assert!(check("Xin chào xin chào cả nhà ơi").is_empty());
        assert!(check("cảm ơn cảm ơn anh nhiều").is_empty());
        assert!(check("làm ơn làm ơn hãy im lặng").is_empty());
    }

    #[test]
    fn khong_bat_ket_thuc_cau() {
        assert!(check("Nhanh lên nhanh lên!").is_empty());
    }

    #[test]
    fn khong_bat_dem_so_va_hat() {
        assert!(check("một hai một hai").is_empty());
        assert!(check("la la la la").is_empty());
    }

    /// Lặp láy có chủ ý nằm trong vi_fixed_phrases → bị gỡ ở điểm chặn
    /// `drop_issues_in_protected`.
    #[test]
    fn khong_bat_lay_co_chu_y() {
        assert!(check("nói chuyện lặng lẽ lặng lẽ với nhau").is_empty());
        assert!(check("công suất chạy liên tục liên tục cả tuần").is_empty());
    }

    #[test]
    fn khong_bat_hoa_toan_phan_va_en() {
        assert!(check("LẦM LẦM LẦM LẦM").is_empty());
        // EN cụm 2 từ lặp chứa từ chức năng = ngữ pháp/nhấn mạnh.
        assert!(check("I have to have to finish this today").is_empty());
    }

    #[test]
    fn khong_bat_khi_khong_phai_lap() {
        assert!(check("vui lòng kiểm tra giúp em").is_empty());
        assert!(check("vui lòng gửi lại vui lòng sớm").is_empty());
    }
}
