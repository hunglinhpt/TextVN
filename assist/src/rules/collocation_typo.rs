// SPDX-License-Identifier: GPL-3.0-or-later
//! OP-004 (P5) — phát hiện và tự sửa các **cụm từ hành chính – văn phòng sai
//! chính tả** khi đi liền nhau ("bổ xung" → "bổ sung", "xử dụng" → "sử
//! dụng"…), nơi từng âm tiết rời rạc đều có trong từ điển nên spellcheck đơn
//! từ không bắt được.
//!
//! Danh sách ĐÓNG đã quét kiểm chứng trên 100% `corpus/assist/*.ass` (vòng 2
//! của kế hoạch P5): 0 trùng trên câu sạch (FPR = 0) — riêng "xoay sở" bị
//! loại vì `vi_real_clean.ass:147` dùng nó như văn phạm đúng.
//! Phát `IssueKind::DiacriticTypo` + `Fix::Replace` (auto_fixable) — mọi nền
//! tảng + Mode 1/2/A2/Selection đã hỗ trợ sẵn kind này (R1-3).

use super::{issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bảng bigram hành chính cố định: `(từ_1, từ_2, vị_trí_sửa: 0=từ_1 | 1=từ_2, từ_đúng)`.
const OFFICE_BIGRAMS: &[(&str, &str, usize, &str)] = &[
    ("bổ", "xung", 1, "sung"),
    ("xử", "dụng", 0, "sử"),
    ("kiểm", "xoát", 1, "soát"),
    ("chính", "sác", 1, "xác"),
    ("sát", "nhập", 0, "sáp"),
    ("thăm", "quan", 0, "tham"),
    ("tựu", "chung", 1, "trung"),
    ("xán", "lạng", 1, "lạn"),
];

/// Bảng trigram hành chính cố định: `(từ_1, từ_2, từ_3, vị_trí_sửa: 0|2, từ_đúng)`.
const OFFICE_TRIGRAMS: &[(&str, &str, &str, usize, &str)] = &[
    ("giành", "thời", "gian", 0, "dành"),
    ("dành", "chiến", "thắng", 0, "giành"),
    ("vô", "hình", "chung", 2, "trung"),
];

/// Giữ hoa/thường của bản gốc khi đưa từ đúng ("Bổ xung" → "Bổ sung",
/// "XỬ DỤNG" → "SỬ DỤNG").
fn preserve_case(orig: &str, correct: &str) -> String {
    let chars: Vec<char> = orig.chars().collect();
    if chars.len() > 1 && chars.iter().all(|c| c.is_uppercase()) {
        return correct.to_uppercase();
    }
    if chars.first().is_some_and(|c| c.is_uppercase()) {
        let mut out: Vec<char> = correct.chars().collect();
        if let Some(first) = out.first_mut() {
            if let Some(up) = first.to_uppercase().next() {
                *first = up;
            }
        }
        return out.into_iter().collect();
    }
    correct.to_string()
}

/// Push issue "cụm từ sai" cho token `bad` — bỏ qua nếu span đã có issue
/// khác phủ (spelling/diacritic bắt trước đó).
fn push_collocation_issue(
    ctx: &Ctx,
    bad_start: usize,
    bad_end: usize,
    bad_orig: &str,
    correct: &str,
    out: &mut Vec<Issue>,
) {
    if out
        .iter()
        .any(|iss| iss.span.start < bad_end && bad_start < iss.span.end)
    {
        return;
    }
    let _ = ctx;
    let fixed = preserve_case(bad_orig, correct);
    out.push(issue(
        IssueKind::DiacriticTypo,
        Span {
            start: bad_start,
            end: bad_end,
        },
        format!("Sai chính tả trong cụm từ: \"{bad_orig}\" → \"{fixed}\""),
        vec![fixed.clone()],
        Fix::Replace {
            start: bad_start,
            end: bad_end,
            text: fixed,
        },
    ));
}

pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    let mut i = 0usize;
    while i < ctx.tokens.len() {
        let t0 = ctx.tokens[i];
        if t0.kind != TokenKind::Word {
            i += 1;
            continue;
        }
        // Cần đúng 1 dấu cách đơn giữa t0 và t1.
        if i + 2 >= ctx.tokens.len() {
            break;
        }
        let gap1 = ctx.tokens[i + 1];
        let t1 = ctx.tokens[i + 2];
        if gap1.kind != TokenKind::Space
            || gap1.len() != 1
            || ctx.chars[gap1.start] != ' '
            || t1.kind != TokenKind::Word
            || ctx.is_masked(t0.start, t1.end)
        {
            i += 1;
            continue;
        }

        let w0_orig = ctx.token_text(&t0);
        let w1_orig = ctx.token_text(&t1);
        let w0 = w0_orig.to_lowercase();
        let w1 = w1_orig.to_lowercase();

        // 1. Thử khớp trigram trước (từ thứ 3 cách đúng 1 dấu cách đơn):
        if i + 4 < ctx.tokens.len() {
            let gap2 = ctx.tokens[i + 3];
            let t2 = ctx.tokens[i + 4];
            if gap2.kind == TokenKind::Space
                && gap2.len() == 1
                && ctx.chars[gap2.start] == ' '
                && t2.kind == TokenKind::Word
                && !ctx.is_masked(t0.start, t2.end)
            {
                let w2_orig = ctx.token_text(&t2);
                let w2 = w2_orig.to_lowercase();
                if let Some(&(_, _, _, target_idx, right)) = OFFICE_TRIGRAMS
                    .iter()
                    .find(|&&(a, b, c, _, _)| a == w0 && b == w1 && c == w2)
                {
                    let (bad_tok, bad_orig) = if target_idx == 0 {
                        (t0, &w0_orig)
                    } else {
                        (t2, &w2_orig)
                    };
                    push_collocation_issue(ctx, bad_tok.start, bad_tok.end, bad_orig, right, out);
                    i += 4;
                    continue;
                }
            }
        }

        // 2. Thử khớp bigram.
        if let Some(&(a, b, target_idx, right)) = OFFICE_BIGRAMS
            .iter()
            .find(|&&(a, b, _, _)| a == w0 && b == w1)
        {
            // Guard "kiểm sát nhập …": "sát" thuộc "kiểm sát" (văn phòng
            // kiểm sát), không phải lỗi "sáp nhập" — bỏ khi từ đứng trước
            // là "kiểm" (vòng implement tự rà, ngoài 2 vòng review kế hoạch).
            if (a, b, target_idx) == ("sát", "nhập", 0) && i >= 2 {
                let gap0 = ctx.tokens[i - 1];
                let prev_w = ctx.tokens[i - 2];
                if gap0.kind == TokenKind::Space
                    && prev_w.kind == TokenKind::Word
                    && ctx.token_text(&prev_w).to_lowercase() == "kiểm"
                {
                    i += 1;
                    continue;
                }
            }
            let (bad_tok, bad_orig) = if target_idx == 0 {
                (t0, &w0_orig)
            } else {
                (t1, &w1_orig)
            };
            push_collocation_issue(ctx, bad_tok.start, bad_tok.end, bad_orig, right, out);
            i += 2;
            continue;
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::check_sentence;

    /// Vòng 32 chốt lại bar: câu ĐÚNG không bao giờ bị bêu — corpus kiểm
    /// chứng "xoay sở" là văn phạm đúng nên KHÔNG nằm trong bảng.
    #[test]
    fn khong_beu_cum_tu_hop_le() {
        for s in [
            "Tôi đang xoay sở để dành tiền với đồng lương ít ỏi.",
            "Chúng tôi bổ sung tài liệu và kiểm soát tiến độ.",
            "Đơn vị sử dụng ngân sách rất chính xác sau khi sáp nhập.",
            // "sát" thuộc "kiểm sát" — không phải lỗi "sáp nhập".
            "Văn phòng kiểm sát nhập dữ liệu vào cuối ngày.",
        ] {
            assert!(check_sentence(s).is_empty(), "\"{s}\" là văn bản đúng");
        }
    }

    #[test]
    fn bat_cum_sai_va_giu_hoa_thuong() {
        let issues = check_sentence("Chúng tôi xin bổ xung tài liệu.");
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].suggestions.first().map(String::as_str),
            Some("sung")
        );

        // Viết hoa đầu câu → giữ hoa.
        let issues2 = check_sentence("Xử dụng ngân sách cần minh bạch.");
        assert_eq!(issues2.len(), 1);
        assert_eq!(
            issues2[0].suggestions.first().map(String::as_str),
            Some("Sử")
        );
    }
}
