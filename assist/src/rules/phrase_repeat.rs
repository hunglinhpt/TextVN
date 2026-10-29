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
use super::{is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

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
    "rồi", "nữa", "vẫn", "cũng", "đều", "còn", "chưa", "sẽ", "thì", "mà", "là", "của", "và",
    "nhưng", "hoặc", "vì", "nếu", "dù", "tuy", "lắm", "quá", "thế", // số từ đếm
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

/// Bat loi lap cum 2 tu: X Y X Y (4 tu lien tiep, 3 dau cach don).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    if looks_like_code_or_list(ctx) {
        return;
    }
    // Layout token: [p0][sp][p1][sp][q0][sp][q1] — q1 ở vị trí `i`.
    for i in 6..ctx.tokens.len() {
        let (p0, p1, q0, q1) = (
            ctx.tokens[i - 6],
            ctx.tokens[i - 4],
            ctx.tokens[i - 2],
            ctx.tokens[i],
        );
        if ![p0, p1, q0, q1].iter().all(is_word) {
            continue;
        }
        // 3 dấu cách đơn giữa các từ.
        let single_space = |k: usize| {
            let t = &ctx.tokens[k];
            t.kind == TokenKind::Space && t.len() == 1 && ctx.chars[t.start] == ' '
        };
        if !single_space(i - 5) || !single_space(i - 3) || !single_space(i - 1) {
            continue;
        }
        let texts: Vec<String> = [p0, p1, q0, q1].iter().map(|t| ctx.token_text(t)).collect();
        // Mỗi từ ≥ 2 ký tự ("à ơi à ơi" hát — không đụng).
        if texts.iter().any(|t| t.chars().count() < 2) {
            continue;
        }
        let lower = |t: &str| t.to_lowercase();
        let (ph1, ph2) = (
            format!("{} {}", lower(&texts[0]), lower(&texts[1])),
            format!("{} {}", lower(&texts[2]), lower(&texts[3])),
        );
        if ph1 != ph2 {
            continue;
        }
        // HOA toàn phần = nhấn mạnh/đầu thư, không phải lỗi gõ.
        if texts
            .iter()
            .all(|t| t.chars().next().is_some_and(|c| c.is_uppercase()))
        {
            continue;
        }
        let words: Vec<&str> = ph1.split(' ').collect();
        // Cụm xã giao idiomatic.
        if PHRASE_WHITELIST.contains(&ph1.as_str()) {
            continue;
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
            continue;
        }
        let start = q0.start;
        let end = q1.end;
        if in_quotes(ctx, start) || ctx.is_masked(start, end) {
            continue;
        }
        // Kết thúc câu ngay sau bản sao 2 ("Nhanh lên nhanh lên!") = lặp kiểu
        // nói — chỉ bắt khi sau cụm còn ít nhất một từ nữa.
        let has_word_after = ctx.tokens[i + 1..].iter().any(is_word);
        if !has_word_after {
            continue;
        }
        // Trợ từ mệnh lệnh ngay sau ("ăn cơm ăn cơm đi nhé") = lời nhắc lại.
        if let Some(next) = ctx.tokens[i + 1..].iter().find(|t| is_word(t)) {
            if IMPERATIVE_AFTER.contains(&ctx.token_text(next).to_lowercase().as_str()) {
                continue;
            }
        }
        out.push(issue(
            IssueKind::DuplicateWord,
            Span { start, end },
            format!("Lặp cụm từ: \"{ph1}\""),
            vec![texts[0].clone() + " " + &texts[1]],
            Fix::Delete {
                start: ctx.tokens[i - 3].start,
                end,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_lap_cum_2_tu() {
        let issues = check("Vui lòng vui lòng kiểm tra giúp em.");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DuplicateWord);
        assert_eq!(issues[0].suggestions[0], "Vui lòng");
        // Fix xoá bản sao 2 + dấu cách trước.
        let text = "Vui lòng vui lòng kiểm tra giúp em.";
        let fixed = crate::apply_auto_fixes(text, &issues);
        assert_eq!(fixed, "Vui lòng kiểm tra giúp em.");
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
