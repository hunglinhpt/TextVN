// SPDX-License-Identifier: GPL-3.0-or-later
//! L8 - sai vi tri dau hay gap ("nghia" sai thanh, "cua" sai thanh).
//!
//! Hai nguon bat loi:
//!
//! 1. **Bang co dinh [`TYPOS`] (khong can tu dien)** - danh sach doi co ban
//!    vi tu sai khong phai la tu (khong the xac minh bang tu dien). Bang nay
//!    chi dung khi caller khong dua tu dien vao (`NoDictionary`).
//! 2. **Sinh bien the dau + xac minh bang tu dien** (v2 - 2026-10-22, D19):
//!    voi mot tu **chua co trong tu dien** ma **co dau**, sinh cac bien the
//!    lech **DUNG MOT thao tac dau** (xem [`variants`]) va chi bao khi
//!    **chi DUNG MOT** bien the la tu hop le. Dieu kien nay la chot chong FP:
//!    - `trong`/`trong`, `tim`/`tim`, `chau`/`cho`, `mac`/`mat`, `ao`/`ao`
//!      deu la **tu hop le ca hai** -> khong bao (khong doan toan).
//!    - Tu dung (ten rieng `Tom`, `Linh`...) thuong **khong co dau** -> bo qua
//!      vi [`variants`] chi sinh tu co dau (xem [`has_diacritic`]).
//!    - `NoDictionary::contains()` **luon `true`** -> khong co tu dien that thi
//!      luat **khong phan doan gi** (fail-open), khong cham `contains` qua lai.
//!
//! Bien the lech mot thao tac (objective, khong heuristics mo):
//! - **E1**: thay **mot** nguyen am bang mot dang cung goc (a/ă/â · e/ê · i ·
//!   o/ô/ơ · u/ư · y) - bao gom ca doi thanh do tren cung am ("đõ" -> "đỗ").
//! - **E2**: `d` <-> `đ`.
//! - **E3**: **chuyen mot thanh do** (sac/huyen/hoi/nga/nang) tu mot nguyen am
//!   sang nguyen am khac ("vạy" -> "vậy").
//!
//! Bo sung "tu co goc khac" (luan -> loan, rua, kho...) khong lam o day: no
//! yeu cau doi nguyen am goc (nguy co FP tren tu dung) -> de lai v2 khi co
//! bo phan doan phuong/ngu canh (Fassist-96/80-83).
//!
//! Bang du lieu **co chon loc**: chi nhung loi khong phu thuoc kieu dau
//! (kieu moi/kiieu cu). Loi phu thuoc kieu dau ("thuy" vs "thuỷ", "hoa" vs
//! "hoà") de cho v2 khi engine biet kieu dau dang dung.

use super::{is_word, issue, Ctx};
use crate::tokenize::Token;
use crate::{Fix, Issue, IssueKind, Span};

/// Bảng lỗi → lời viết đúng (chữ thường) — **không cần từ điển**.
///
/// Chính sách Fassist-96 (giữ nguyên): chỉ thêm cặp mà **từ "sai" KHÔNG phải
/// từ hợp lệ** của tiếng Việt — thêm cặp mà từ sai còn dùng được ("tròng"/
/// "trong", "mặc"/"mặt", "ảo"/"áo", "tim"/"tìm", "chấu"/"chỗ", "hường"/"hưởng")
/// là **dự đoán mò**: sẽ bêu đúng câu đúng (FPR) khi ngữ cảnh là hợp lệ.
///
/// Nguồn các mục mới (2026-10-22, P4-12/D19): 21 ca `diacritic_typo` **bị bỏ
/// sót** trên corpus thật `corpus/assist/vi_real_errors.ass` (đã 2 người đánh
/// giá) — trong đó 16 cặp không-được-phép-bổ-sung ở trên là **từ sai không
/// phải từ** ⇒ an toàn.
const TYPOS: &[(&str, &str)] = &[
    ("nghiã", "nghĩa"),
    ("nghỉa", "nghĩa"),
    ("cuả", "của"),
    ("tiêng", "tiếng"),
    ("fải", "phải"),
    ("nhứng", "nhưng"),
    // LƯU Ý: không thêm "trương" — đó là từ HỢP LỆ ("khẩn trương", "trầu trương");
    // KHONG them cac tu HOP LE: "truong" (khan truong), "yeu" (yeu tuong) - sai dau o day can ngu canh (Fassist-96).
    // sai dấu ở đây cần ngữ cảnh để phân biệt (Fassist-96).
    ("nguyến", "nguyễn"),
    // ---- P4-12/D19 (2026-10-22): ca corpus thật bị bỏ sót, từ sai không phải từ ----
    ("đõ", "đỗ"),     // "đõ xe" → "đỗ xe"
    ("đươc", "được"), // "đươc trưng bày" → "được trưng bày"
    ("ngườì", "người"),
    ("dạp", "đạp"), // "xe dạp" → "xe đạp"
    ("xãy", "xảy"), // "xãy ra" → "xảy ra"
    ("cùa", "của"), // "cùa tôi" → "của tôi"
    ("môt", "một"),
    ("vâỵ", "vậy"),   // "Vâỵ là quá đủ" → "Vậy là…"
    ("mõi", "mỏi"),   // "mệt mõi" → "mệt mỏi"
    ("luạn", "loạn"), // "hỗn luạn" → "hỗn loạn"
    ("rỡa", "rửa"),   // "rỡa sạch" → "rửa sạch"
    ("mội", "một"),   // "mội người" → "một người"
    ("khônf", "không"), // "khônf thể" → "không thể"
                      // VÒNG 23 (2026-10-23, audit "như Grammarly"): XOÁ 5 cặp mà từ "sai" là
                      // từ HỢP LỆ — vi phạm chính sách Fassist-96 của chính file này, bêu + TỰ
                      // SỬA SAI chữ đúng: "quyến rũ"→"quyển", "thuở xưa"→"thướ" ("thướ" còn
                      // không có trong từ điển), "đả thông"→"đã", "áy náy"→"ấy", "ngôi điếm"→
                      // "điểm". Lỗi ngữ cảnh thuộc v2 (như "tròng/trong" đã ghi ở header).
];

/// 12 nhóm nguyên âm tiếng Việt × 6 cột (thanh điệu): không dấu · sắc · huyền ·
/// hỏi · ngã · nặng.
const VOWELS: &[[char; 6]; 12] = &[
    ['a', 'à', 'á', 'ả', 'ã', 'ạ'],
    ['ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ề', 'ế', 'ể', 'ễ', 'ệ'],
    ['i', 'ì', 'í', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ò', 'ó', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ù', 'ú', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự'],
    ['y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ'],
];

/// Nhóm "gốc" của nguyên âm (E1 chỉ đổi trong cùng gốc).
/// `a/ă/â` · `e/ê` · `i` · `o/ô/ơ` · `u/ư` · `y`.
fn base_group(row: usize) -> usize {
    match row {
        0..=2 => 0,
        3 | 4 => 1,
        5 => 2,
        6..=8 => 3,
        9 | 10 => 4,
        _ => 5,
    }
}

/// Vị trí `(nhóm, cột)` của một ký tự nguyên âm có dấu, nếu có.
fn vowel_pos(c: char) -> Option<(usize, usize)> {
    VOWELS
        .iter()
        .enumerate()
        .find_map(|(r, row)| row.iter().position(|&x| x == c).map(|col| (r, col)))
}

/// Từ **có ít nhất một dấu** tiếng Việt (thanh điệu hoặc mũ/trăng/móc/đ).
///
/// Bộ lọc chống FP cho tên riêng không dấu (`Tom`, `Linh`, `Lan`, …): sai dấu
/// thật sự thì nguồn **phải** đã có dấu — không có dấu thì không "sai dấu".
fn has_diacritic(s: &str) -> bool {
    s.chars().any(|ch| {
        for c in ch.to_lowercase() {
            if c == 'đ' {
                return true;
            }
            if let Some((row, col)) = vowel_pos(c) {
                if col > 0 || matches!(row, 1 | 2 | 4 | 7 | 8 | 10) {
                    return true;
                }
            }
        }
        false
    })
}

/// Sinh các biến thể lech **duy nhat mot thao tac dau** (E1/E2/E3 — xem header).
///
/// Kết quả **không** loại trùng (caller loại); có thể có > 1 chuỗi giống nhau
/// từ 2 nhánh khác nhau — caller vẫn chỉ cần **kiểm tra sự tồn tại duy nhất
/// của TỪ hợp lệ**, nên việc trùng chuỗi không ảnh hưởng kết quả.
fn variants(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut out: Vec<String> = Vec::with_capacity(chars.len() * 24);

    for (i, &ch) in chars.iter().enumerate() {
        // E1 — đổi MỘT nguyên âm sang mọi dạng cùng gốc (kể cả đổi cả thanh).
        if let Some((r, c)) = vowel_pos(ch) {
            let g = base_group(r);
            for (r2, row) in VOWELS.iter().enumerate() {
                if base_group(r2) != g {
                    continue;
                }
                for (c2, &v) in row.iter().enumerate() {
                    if r2 == r && c2 == c {
                        continue;
                    }
                    let mut v2 = chars.clone();
                    v2[i] = v;
                    out.push(v2.into_iter().collect());
                }
            }
        }
        // E2 — `d` <-> `đ`.
        if ch == 'd' || ch == 'đ' {
            let mut v2 = chars.clone();
            v2[i] = if ch == 'd' { 'đ' } else { 'd' };
            out.push(v2.into_iter().collect());
        }
    }

    // E3 — chuyển MỘT thanh điệu từ nguyên âm này sang nguyên âm khác.
    let vowel_idx: Vec<usize> = (0..chars.len())
        .filter(|&i| vowel_pos(chars[i]).is_some())
        .collect();
    for &a in &vowel_idx {
        let (_, ca) = vowel_pos(chars[a]).expect("đã lọc là nguyên âm");
        if ca == 0 {
            continue; // nguồn không có thanh để chuyển
        }
        for &b in &vowel_idx {
            if a == b {
                continue;
            }
            let (rb, cb) = vowel_pos(chars[b]).expect("đã lọc là nguyên âm");
            if cb != 0 {
                continue; // đích đã có thanh
            }
            let mut v2 = chars.clone();
            let (ra, _) = vowel_pos(v2[a]).expect("đã lọc là nguyên âm");
            v2[a] = VOWELS[ra][0]; // bỏ thanh ở nguồn
            v2[b] = VOWELS[rb][ca]; // đặt đúng thanh đó vào đích
            out.push(v2.into_iter().collect());
        }
    }

    out
}

/// Số từ **không xác định** được phép sinh biến thể tối đa trong MỘT câu.
///
/// Với từ điển OS (provider `Check` ~0,057 ms — P4-10 F3) mỗi biến thể ~1 nhát
/// `contains`: chặn ở đây để câu bất thường (nhiều tên riêng lạ) không biến
/// thành hàng chục nhát tra cứu. Hết ngân sách ⇒ **bỏ qua** các từ còn lại
/// (fail-open: không báo còn hơn báo sai/chậm).
const MAX_UNKNOWN_TOKENS: usize = 8;

/// Bat tu sai vi tri dau.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    let mut budget = MAX_UNKNOWN_TOKENS;
    for token in &ctx.tokens {
        if !is_word(token) {
            continue;
        }
        if ctx.is_masked(token.start, token.end) {
            continue;
        }
        let text = ctx.token_text(token);
        let lower = text.to_lowercase();

        // (1) Bảng cố định — không cần từ điển (vẫn chạy khi caller chưa nạp).
        if let Some((_, correct)) = TYPOS.iter().find(|(wrong, _)| *wrong == lower) {
            push_issue(token, &text, correct, out);
            continue;
        }

        // (2) Sinh biến thể + xác minh qua từ điển.
        // - Không có dấu ⇒ không phải lỗi "sai dấu" (bảo vệ tên riêng).
        // - Từ đã có trong từ điển ⇒ không đụng (không đoán mò — Fassist-96).
        if !has_diacritic(&lower) || ctx.dict.contains(&lower) {
            continue;
        }
        if budget == 0 {
            continue; // hết ngân sách từ lạ trong câu — fail-open
        }
        budget -= 1;
        let mut hit: Option<String> = None;
        let mut ambiguous = false;
        for cand in variants(&lower) {
            if cand == lower || !ctx.dict.contains(&cand) {
                continue;
            }
            if hit.as_deref() == Some(cand.as_str()) {
                continue; // cùng biến thể sinh ra từ 2 nhánh — không tính là mơ hồ
            }
            if hit.is_some() {
                ambiguous = true; // ≥ 2 từ hợp lệ ⇒ không đủ bằng chứng
                break;
            }
            hit = Some(cand);
        }
        // Chỉ báo khi **đúng một** biến thể là từ hợp lệ (chốt chống FP).
        let Some(correct) = hit.filter(|_| !ambiguous) else {
            continue;
        };
        push_issue(token, &text, &correct, out);
    }
}

/// Đẩy issue, **giữ hoa/thường của ký tự đầu** và dùng span = token.
fn push_issue(token: &Token, text: &str, correct: &str, out: &mut Vec<Issue>) {
    let mut fixed = correct.to_string();
    if let Some(first) = text.chars().next() {
        if first.is_uppercase() {
            let mut chars = fixed.chars();
            let head = chars.next().unwrap_or(' ');
            fixed = head.to_uppercase().collect::<String>() + chars.as_str();
        }
    }
    out.push(issue(
        IssueKind::DiacriticTypo,
        Span {
            start: token.start,
            end: token.end,
        },
        format!("Sai vị trí dấu: \"{text}\" → \"{fixed}\""),
        vec![fixed.clone()],
        Fix::Replace {
            start: token.start,
            end: token.end,
            text: fixed,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;
    use crate::check_sentence;

    #[test]
    fn bat_sai_dau() {
        let issues = check("tôi nghiã kho");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DiacriticTypo);
    }

    #[test]
    fn khong_bat_tu_dung() {
        assert!(check("tôi nghĩa kho").is_empty());
    }

    #[test]
    fn giu_hoa_thuong() {
        let issues = check("Nghiã thế");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions[0], "Nghĩa");
    }

    #[test]
    fn variants_lech_dung_mot_thao_tac() {
        // E1 — cùng gốc, đổi cả thanh: "đõ" -> "đỗ" (o -> ô, ngã -> hỏi).
        assert!(variants("đõ").contains(&"đỗ".to_string()));
        // E1 — bỏ thanh: "ngườì" -> "người".
        assert!(variants("ngườì").contains(&"người".to_string()));
        // E1 — thêm nặng: "môt" -> "một".
        assert!(variants("môt").contains(&"một".to_string()));
        // E2 — d <-> đ: "dạp" -> "đạp".
        assert!(variants("dạp").contains(&"đạp".to_string()));
        // E3 — chuyển nặng từ y sang â: "vạy" -> "vậy".
        assert!(variants("vạy").contains(&"vậy".to_string()));
        // Không sinh biến thể lệch gốc (E1) — "luan" không thành "loan".
        assert!(!variants("luạn").contains(&"loạn".to_string()));
    }

    #[test]
    fn khong_bao_ten_khong_dau() {
        // Tên riêng không dấu — không có "dấu" để sai ⇒ không sinh biến thể.
        assert!(!has_diacritic("Tom"));
        assert!(!has_diacritic("Linh"));
        assert!(has_diacritic("đõ"));
        assert!(has_diacritic("một"));
    }

    #[test]
    fn khong_doan_khi_hai_bien_the_deu_hop_le() {
        // Từ điển giả: cả "khỗ" và "khồ" đều hợp lệ ⇒ "khõ" mơ hồ ⇒ KHÔNG báo.
        // (Chọn "khõ" — không nằm trong bảng cố định TYPOS — để đo **đường
        // từ điển**, không phải đường bảng.)
        struct Fake(&'static [&'static str]);
        impl crate::dict::Dictionary for Fake {
            fn contains(&self, w: &str) -> bool {
                self.0.contains(&w)
            }
            fn suggest(&self, _w: &str, _limit: usize) -> Vec<String> {
                Vec::new()
            }
        }
        let both = Fake(&["khỗ", "khồ"]);
        let issues: Vec<Issue> = crate::check_with("khõ", &both)
            .into_iter()
            .filter(|i| i.kind == IssueKind::DiacriticTypo)
            .collect();
        assert!(issues.is_empty(), "mơ hồ ⇒ không đủ bằng chứng ⇒ bỏ qua");

        // Từ điển giả: chỉ "khỗ" hợp lệ ⇒ báo đúng 1 lỗi với gợi ý "khỗ".
        let one = Fake(&["khỗ"]);
        let issues: Vec<Issue> = crate::check_with("tôi khõ", &one)
            .into_iter()
            .filter(|i| i.kind == IssueKind::DiacriticTypo)
            .collect();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].suggestions[0], "khỗ");
    }

    #[test]
    fn bat_ca_cau_corpus_that_moi_nhat() {
        // 13 cặp còn lại trong bảng TYPOS phải bắt đúng (hồi quy P4-12/D19;
        // vòng 23 bỏ 5 cặp vi phạm Fassist-96 — "đả/áy/điếm/thuở/quyến" là từ
        // hợp lệ, xem comment cuối bảng).
        for (wrong, right) in [
            ("đõ", "đỗ"),
            ("đươc", "được"),
            ("ngườì", "người"),
            ("dạp", "đạp"),
            ("xãy", "xảy"),
            ("cùa", "của"),
            ("môt", "một"),
            ("vâỵ", "vậy"),
            ("mõi", "mỏi"),
            ("luạn", "loạn"),
            ("rỡa", "rửa"),
            ("mội", "một"),
            ("khônf", "không"),
        ] {
            let issues: Vec<Issue> = check(&format!("tài {wrong}"))
                .into_iter()
                .filter(|i| i.kind == IssueKind::DiacriticTypo)
                .collect();
            assert_eq!(issues.len(), 1, "phải bắt được từ sai '{wrong}'");
            assert_eq!(issues[0].suggestions[0], right, "gợi ý của '{wrong}'");
        }
    }

    /// Vòng 23 (audit "như Grammarly"): các từ HỢP LỆ từng nằm nhầm trong bảng
    /// TYPOS giờ phải **im lặng** — không bêu, không tự sửa chữ đúng.
    #[test]
    fn khong_beu_tu_hop_le_tung_o_bang_typos() {
        for sentence in [
            "Cô ấy quyến rũ đến lạ thường.",
            "Thuở xưa có một cô bé.",
            "Họ đả thông mọi bế tắc.",
            "Tôi thấy áy náy lắm.",
            "Ngôi điếm cũ bên đường.",
        ] {
            let issues: Vec<Issue> = check_sentence(sentence)
                .into_iter()
                .filter(|i| i.kind == IssueKind::DiacriticTypo)
                .collect();
            assert!(
                issues.is_empty(),
                "\"{sentence}\" là câu ĐÚNG — không được bêu diacritic_typo (Fassist-96)"
            );
        }
    }
}
