// SPDX-License-Identifier: GPL-3.0-or-later
//! L1/L2/L11 - lap tu lien tiep ("toi toi", "cua cua", "rat rat").

use super::{is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Từ lặp **có chủ ý** (reduplicative) trong tiếng Việt — "luôn luôn", "mãi mãi",
/// "dần dần"… là cách tăng nghĩa, **không phải lỗi** (Fassist-94).
pub(crate) const INTENTIONAL_REPEATS: &[&str] = &[
    // tăng nghĩa: "luôn luôn", "mãi mãi", "dần dần", "rất rất"…
    "luôn", "mãi", "dần", "chính", "thật", "từ", "từng", "ngay", "vừa", "rất",
    // từ ghép đôi: "lâu lâu", "vội vội", "vàng vàng"… (Fassist-94)
    "lâu", "vội", "vàng", "chầm", "lẩn", "lầm", "cốp", "lốp", "ríu", "húp", "quẩy", "thong",
    "phèng", "sột", "tí",
    // Vòng 23 (audit "như Grammarly", đo FPR trên tập sạch Tatoeba): các từ
    // lặp hợp lệ còn sót — từ tượng thanh ("chộp chộp, bum bum"), cách nhấn
    // thế thao ("ở đâu đâu") và số từ + danh từ cùng hình ("năm năm trước" —
    // "năm" số từ, "năm" danh từ, cách nói chuẩn mực).
    "chộp", "bum", "đâu", "năm",
];

/// Từ có nằm trong cặp ngoặc kép `"` nào không (đếm số `"` trước vị trí).
pub(crate) fn in_quotes(ctx: &Ctx, pos: usize) -> bool {
    // Vòng 32 (audit A5): nháy CONG “ ” cùng họ với " — lặp từ trong
    // trích lời ("cậu nói “không không” mà") là nguyên văn, không phải lỗi.
    ctx.chars[..pos]
        .iter()
        .filter(|c| matches!(**c, '"' | '“' | '”'))
        .count()
        % 2
        == 1
}

/// Lặp từ có chủ ý của TIẾNG ANH (vòng 24 — audit EN trap 1-17; đối chiếu
/// corpus `en_basic.ass`: "no no" KHÔNG trong whitelist — PO chốt là lỗi).
pub(crate) const EN_INTENTIONAL_REPEATS: &[&str] = &[
    // ngữ pháp: "that that" (mệnh đề quan hệ), "had had" (quá khứ hoàn thành)
    "that", "had", // tăng nghĩa/tình thái
    "very", "long", "so", "well", "there", "now", "bye", "night", "win", "great",
    // từ tượng thanh/lời nhạc
    "ha", "blah", "yada", "na", "la", "hey", "yeah", "dum", "boom", "chop",
    // năm đọc theo cặp ("twenty twenty")
    "twenty", "nineteen", "seventy", "eighty", "ninety",
    // vòng 26: gộp từ whitelist VI 0.2.36 thêm (câu mix "Chộp chộp, bum bum"
    // — "bum" ASCII rơi vào whitelist EN nên phải có ở ĐÂY; "chộp" có dấu
    // vẫn ở whitelist VI nhưng gộp cho nhất quán).
    "bum", "chộp", "đâu", "năm",
];

/// Từ CHỨC NĂNG lặp ≥ 3 lần ("the the the file", "đã đã đã gửi") gần như
/// LUÔN là lỗi gõ — guard run≥3 chỉ miễn cho từ nội dung/tượng thanh
/// (vòng 40, R2 persona RH-7). Đại từ lặp viết hoa đầu câu ("Cô Cô sẽ chữa")
/// cũng là lỗi thật (RH-6) — guard vocative chỉ giữ cho tên riêng.
const REPEAT3_FUNCTION_WORDS: &[&str] = &[
    // EN
    "the", "a", "an", "to", "of", "and", "but", "or", "that", "this", "it", "is", "are", "was",
    "were", "do", "does", "did", "have", "has", "had", "will", "would", "can", "could", "should",
    "must", // VI
    "đã", "chưa", "được", "có", "bị", "thì", "mà", "là", "của",
];

/// Đại từ/xưng hô lặp viết hoa đầu câu ("Cô Cô sẽ chữa bài", "Em Em xin lỗi")
/// là lỗi gõ thật — KHÔNG thuộc guard vocative tên riêng ("Lê Lê ơi").
/// "anh/chị/bạn" bỏ ra (vừa là đại từ vừa là tên người — giữ guard cũ).
const PRONOUN_CAP_FIRE: &[&str] = &["em", "cô", "thầy", "tôi", "mình", "chúng", "please", "i"];

/// Số lần liên tiếp từ này lặp ở vị trí `i` (≥ 2 khi `tokens[i]` lặp
/// `tokens[i-2]`): "no no no", "la la la" = nhấn mạnh/lời hát, không phải lỗi.
fn run_length(ctx: &Ctx, i: usize, lower: &str) -> usize {
    let mut n = 2usize; // đã biết tokens[i] == tokens[i-2]
    let mut k = i + 2;
    while k < ctx.tokens.len() {
        let (gap, word) = (ctx.tokens[k - 1], ctx.tokens[k]);
        if !is_word(&word)
            || gap.kind != TokenKind::Space
            || gap.len() != 1
            || ctx.token_text(&word).to_lowercase() != lower
        {
            break;
        }
        n += 1;
        k += 2;
    }
    n
}

/// `tokens[i-4]` cũng là cùng từ (cách nhau 1 dấu cách) ⇒ `tokens[i]` là phần
/// thứ ≥ 3 của chuỗi lặp ("no no no" — lần lặp thứ hai và thứ ba).
fn repeats_before(ctx: &Ctx, i: usize, lower: &str) -> bool {
    if i < 4 {
        return false;
    }
    let (gap, word) = (ctx.tokens[i - 3], ctx.tokens[i - 4]);
    is_word(&word)
        && gap.kind == TokenKind::Space
        && gap.len() == 1
        && ctx.token_text(&word).to_lowercase() == lower
}

/// Bat loi lap tu: hai tu lien tiep giong nhau, cach nhau dung mot khoang trang.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Token: [Word][Space][Word] - can nhin 2 token lui.
    for i in 2..ctx.tokens.len() {
        let cur = ctx.tokens[i];
        let gap = ctx.tokens[i - 1];
        let prev = ctx.tokens[i - 2];
        if !is_word(&cur) || !is_word(&prev) || cur.len() != prev.len() || cur.len() < 2 {
            continue;
        }
        if gap.kind != TokenKind::Space || gap.len() != 1 || ctx.chars[gap.start] != ' ' {
            continue;
        }
        let cur_text = ctx.token_text(&cur);
        let prev_text = ctx.token_text(&prev);
        let lower = cur_text.to_lowercase();
        if lower != prev_text.to_lowercase() {
            continue;
        }
        // "luôn luôn", "mãi mãi"… la lap co chu y, khong phai loi (Fassist-94).
        // Vòng 26: chọn whitelist theo **TỪNG TỪ** chứ không theo câu — câu
        // mix ("check the the file nhé") có lặp từ EN giữa câu tiếng Việt:
        // từ ASCII → whitelist EN, từ có dấu Việt → whitelist VI.
        let is_ascii_word = lower.chars().all(|c| c.is_ascii_alphabetic());
        let whitelist: &[&str] = if is_ascii_word {
            EN_INTENTIONAL_REPEATS
        } else {
            INTENTIONAL_REPEATS
        };
        if whitelist.contains(&lower.as_str()) {
            continue;
        }
        // Vòng 24 (audit EN trap 18-19) — vòng 26 áp cho TỪ ASCII ở MỌI câu
        // (không chỉ câu En-dominant): lặp TÊN RIÊNG hô gọi ("Emma Emma
        // wait!") — CẢ HAI viết hoa; lỗi thật ở đầu câu chỉ có từ ĐẦU viết
        // hoa ("They they said") vẫn phải bắt. Từ trong ngoặc kép bỏ qua.
        // Vòng 26: guard này áp cho MỌI ngôn ngữ — tên riêng có dấu Việt
        // ("Lê Lê ơi") và trích lời nói nguyên văn ("nói "không không"")
        // là hợp lệ bất kể bảng chữ.
        let cur_cap = cur_text.chars().next().is_some_and(|c| c.is_uppercase());
        let prev_cap = prev_text.chars().next().is_some_and(|c| c.is_uppercase());
        if cur_cap && prev_cap && !PRONOUN_CAP_FIRE.contains(&lower.as_str()) {
            continue;
        }
        if in_quotes(ctx, cur.start) {
            continue;
        }
        if ctx.is_masked(prev.start, cur.end) {
            continue;
        }
        // Lặp ≥ 3 lần liền ("no no no", "la la la", "hu hu hu") = nhấn mạnh /
        // tượng thanh có chủ ý — chỉ với từ NỘI DUNG. Từ chức năng lặp ≥ 3
        // ("the the the file", "đã đã đã gửi") là lỗi gõ thật (vòng 40 RH-7),
        // báo đúng MỘT lần ở bản sao đầu; các bản sao sau (repeats_before)
        // là phần còn lại của cùng một chuỗi lỗi.
        let long_run = run_length(ctx, i, &lower) >= 3 || repeats_before(ctx, i, &lower);
        if long_run
            && (!REPEAT3_FUNCTION_WORDS.contains(&lower.as_str()) || repeats_before(ctx, i, &lower))
        {
            continue;
        }
        out.push(issue(
            IssueKind::DuplicateWord,
            Span {
                start: cur.start,
                end: cur.end,
            },
            format!("Lặp từ: \"{cur_text}\""),
            vec![prev_text],
            Fix::Delete {
                start: gap.start,
                end: cur.end,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_lap_tu_thuong() {
        let issues = check("tôi tôi đi học");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::DuplicateWord);
    }

    #[test]
    fn bat_lap_gioi_tu() {
        assert_eq!(check("đã đã xong").len(), 1);
    }

    #[test]
    fn khong_bat_khi_khong_lap() {
        assert!(check("tôi đi học").is_empty());
    }

    /// Vòng 26 — câu mix văn phòng: whitelist theo TỪNG TỪ.
    #[test]
    fn cau_mix_whitelist_theo_tu() {
        // Lặp EN hợp lệ giữa câu Việt ("that that" mệnh đề quan hệ) → không báo.
        assert!(check("bạn check that that case nhé").is_empty());
        assert!(check("gửi link bum bum cho tôi").is_empty());
        // Lặp EN là LỖI trong câu Việt → vẫn bắt.
        let issues = check("anh check the the file nhé");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, crate::IssueKind::DuplicateWord);
        // Lặp VI hợp lệ vẫn qua whitelist VI.
        assert!(check("chờ lâu lâu cũng được").is_empty());
    }

    /// Vòng 26 — guard vocative/quote cho từ ASCII ở mọi ngôn ngữ.
    #[test]
    fn guard_ten_rieng_va_ngoac_kep_moi_ngon_ngu() {
        // Tên riêng hô gọi (cả hai viết hoa) — không báo.
        assert!(check("Linh Linh ơi, xem lại giúp").is_empty());
        // Lời thoại trong ngoặc kép lặp — không báo.
        assert!(check("cậu nói \"không không\" mà").is_empty());
    }

    #[test]
    fn khong_bat_hai_tu_khac_nhau() {
        assert!(check("của nhà").is_empty());
    }

    /// Vòng 40 (RH-6): đại từ lặp viết hoa đầu câu là lỗi thật, không thuộc
    /// guard vocative tên riêng.
    #[test]
    fn bat_dai_tu_lap_hoa_dau_cau() {
        assert_eq!(check("Cô Cô sẽ chữa bài sau.").len(), 1);
        assert_eq!(check("Em Em xin lỗi ạ.").len(), 1);
        // Tên riêng vẫn được guard vocative giữ.
        assert!(check("Linh Linh ơi, xem lại giúp").is_empty());
    }

    /// Vòng 40 (RH-7): từ chức năng lặp ≥ 3 lần = lỗi gõ, báo ĐÚNG 1 lần.
    #[test]
    fn bat_tu_chuc_nang_lap_3_lan() {
        assert_eq!(check("the the the file").len(), 1);
        assert_eq!(check("đã đã đã gửi").len(), 1);
        // Từ nội dung/tượng thanh lặp ≥ 3 vẫn là nhấn mạnh hợp lệ.
        assert!(check("no no no").is_empty());
        assert!(check("la la la").is_empty());
        assert!(check("very very very good").is_empty());
    }
}
