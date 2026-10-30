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
    ("chuẩn", "đoán", 0, "chẩn"),
    ("cọ", "sát", 1, "xát"),
    ("chính", "chu", 0, "chỉn"),
    ("trao", "dồi", 0, "trau"),
    ("bạc", "mạng", 0, "bạt"),
    ("đầy", "dẫy", 1, "rẫy"),
    ("dấu", "diếm", 0, "giấu"),
    ("giấu", "diếm", 1, "giếm"),
    ("dấu", "giếm", 0, "giấu"),
    ("sơ", "xuất", 1, "suất"),
    ("xuất", "ăn", 0, "suất"),
    ("suất", "sắc", 0, "xuất"),
    // Phân biệt giành / dành theo cặp từ ngữ nghĩa:
    ("dành", "độc", 0, "giành"),
    ("dành", "quyền", 0, "giành"),
    ("dành", "giật", 0, "giành"),
    ("giành", "riêng", 0, "dành"),
    ("giành", "tặng", 0, "dành"),
    ("giành", "dụm", 0, "dành"),
    ("giành", "trọn", 0, "dành"),
    // Vòng 37 (user probe): các cặp sai âm tiết phổ biến trong văn liệu —
    // bigram sai không thể là từ/cụm hợp lệ (đối chiếu dict + nghĩa).
    ("sẳn", "sàng", 0, "sẵn"),
    ("chần", "chờ", 1, "chừ"),
    ("cần", "mẩn", 1, "mẫn"),
    ("cố", "găng", 1, "gắng"),
    ("bàng", "hoang", 1, "hoàng"),
    ("giành", "giựt", 1, "giật"),
    ("hoành", "thành", 0, "hoàn"),
    // Vòng 40 (R2 persona + mở rộng văn phòng): cặp sai âm tiết — bigram sai
    // phải thỏa điều kiện "cặp sai KHÔNG THỂ hợp lệ nào khác" (R2-F1: 4 cặp
    // 'thăm gia'/'hộp hành'/'báo cá'/'duy nhát' bị đối kháng bóc — 'thăm gia
    // đình', 'một hộp hành', 'báo cá nhân', 'Duy nhát một nhát' đều hợp lệ ⇒
    // ĐÃ BỎ; thêm cặp mới phải probe cả nghĩa sống trước khi đưa bảng đóng).
    ("cảm", "kết", 0, "cam"),
    ("xắp", "xếp", 0, "sắp"),
    ("chẩn", "bị", 0, "chuẩn"),
    ("quãng", "cáo", 0, "quảng"),
    ("miến", "phí", 0, "miễn"),
    ("hớp", "đồng", 0, "hợp"),
    ("khai", "tác", 1, "thác"),
    ("khảo", "sác", 1, "sát"),
    ("chỉnh", "sủa", 1, "sửa"),
    ("đanh", "giá", 0, "đánh"),
    ("hoành", "trang", 1, "tráng"),
    ("trãi", "nghiệm", 0, "trải"),
    ("hoàn", "tòan", 1, "toàn"),
    // RH-3 (R2 persona giáo viên): âm tiết Việt KHÔNG DẤU mơ hồ riêng lẻ
    // ("hoc" là âm tiết hợp lệ) nhưng bigram gần như một nghĩa — sửa từ 1,
    // từ 2 giữ nguyên phải là từ đúng sẵn ("sinh").
    ("hoc", "sinh", 0, "học"),
    // Vòng 41 (batch 2 — mỗi cặp đã probe nghĩa sống cả 2 chiều: "y" không
    // đứng trước "kiến/tưởng" với nghĩa khác; "binh" hợp lệ lẻ nhưng "binh
    // thường" không; "tuyệt đôi/vơi", "cổ vấn", "lao dộng", "nghỉ ngợi",
    // "lắm việc" BỊ LOẠI vì nghĩa sống "lắm việc thật").
    ("y", "kiến", 0, "ý"),
    ("y", "tưởng", 0, "ý"),
    ("binh", "thường", 0, "bình"),
    ("tuyệt", "đôi", 1, "đối"),
    ("tuyệt", "vơi", 1, "vời"),
    ("cổ", "vấn", 0, "cố"),
    ("hướng", "dẩn", 1, "dẫn"),
    ("lợi", "nhuần", 1, "nhuận"),
    ("bồi", "dưởng", 1, "dưỡng"),
    ("phầm", "mềm", 0, "phần"),
    ("lao", "dộng", 1, "động"),
    ("nghỉ", "ngợi", 1, "ngơi"),
    // Vòng 41 batch 3 — E34 đã probe: "đường dốc" hợp lệ nhưng "đôn dốc"
    // không; "chốt đơn" hợp lệ nhưng "hạn chốt" không; "chú ý" hợp lệ nhưng
    // "chú đáo" không; "trong nhà" hợp lệ nhưng "cẩn trong" không;
    // "trung thục" (Thục phi) không có nghĩa; "sọan thảo" không phải từ.
    ("sọan", "thảo", 0, "soạn"),
    ("đôn", "dốc", 1, "đốc"),
    ("trung", "thục", 1, "thực"),
    ("chú", "đáo", 0, "chu"),
    // ("cẩn","trong") BỊ E34 vạch ngay khi probe: "cất cẩn trong két" hợp lệ
    // — cặp bỏ khỏi bảng.
    ("hạn", "chốt", 1, "chót"),
];

/// Bảng trigram hành chính cố định: `(từ_1, từ_2, từ_3, vị_trí_sửa: 0|1|2, từ_đúng)`.
const OFFICE_TRIGRAMS: &[(&str, &str, &str, usize, &str)] = &[
    ("giành", "thời", "gian", 0, "dành"),
    ("dành", "chiến", "thắng", 0, "giành"),
    ("vô", "hình", "chung", 2, "trung"),
    ("dành", "thắng", "lợi", 0, "giành"),
    ("dành", "độc", "lập", 0, "giành"),
    ("dành", "quyền", "lực", 0, "giành"),
    ("giành", "tình", "cảm", 0, "dành"),
    ("nghe", "phong", "phanh", 2, "thanh"),
];

/// Vòng 37: cụm SAI CẢ 2 âm tiết — thay TRỌN cụm (1 token/cụm không đủ).
/// ("trực trặc" → "trúc trắc"; "trúc trặc" sai âm 2 handled riêng ở bigram).
const OFFICE_PHRASES: &[(&str, &str)] = &[("trực trặc", "trúc trắc")];
// Vòng 41 (Pass A T3): trigram ("độ","bàng","quang")→"quan" ĐÃ BỎ — "độ
// bàng quang" là thuật ngữ y khoa HỢP LỆ ("Siêu âm đo độ bàng quang"). Lỗi
// thật "thái độ bàng quan" cần anchor 4-từ mà matcher PHRASES chỉ khớp
// bigram — miss có chủ ý theo bar FPR (medical-term FP tệ hơn).

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
        // Gợi ý đã đi riêng trong `suggestions` (panel hiện "… -> sung") —
        // không lặp lại trong thông điệp (R2-127: dòng lỗi bị cắt "-> su…").
        format!("Sai chính tả trong cụm từ: \"{bad_orig}\""),
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
                    let (bad_tok, bad_orig) = match target_idx {
                        0 => (t0, &w0_orig),
                        1 => (t1, &w1_orig),
                        _ => (t2, &w2_orig),
                    };
                    push_collocation_issue(ctx, bad_tok.start, bad_tok.end, bad_orig, right, out);
                    i += 4;
                    continue;
                }
            }
        }

        // 1b. Thử khớp PHRASE 2-âm-sai (vòng 37): thay trọn span 2 từ.
        {
            let phrase = format!("{w0} {w1}");
            if let Some(&(_, right)) = OFFICE_PHRASES.iter().find(|&&(wrong, _)| wrong == phrase) {
                let span_start = t0.start;
                let span_end = t1.end;
                if !out
                    .iter()
                    .any(|iss| iss.span.start < span_end && span_start < iss.span.end)
                {
                    let fixed = preserve_case(&phrase, right);
                    out.push(issue(
                        IssueKind::DiacriticTypo,
                        Span {
                            start: span_start,
                            end: span_end,
                        },
                        format!("Sai chính tả trong cụm từ: \"{phrase}\" → \"{fixed}\""),
                        vec![fixed.clone()],
                        Fix::Replace {
                            start: span_start,
                            end: span_end,
                            text: fixed,
                        },
                    ));
                }
                i += 2;
                continue;
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
