// SPDX-License-Identifier: GPL-3.0-or-later
//! L1/L2/L11 - lap tu lien tiep ("toi toi", "cua cua", "rat rat").

use super::{is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Từ lặp **có chủ ý** (reduplicative) trong tiếng Việt — "luôn luôn", "mãi mãi",
/// "dần dần"… là cách tăng nghĩa, **không phải lỗi** (Fassist-94).
const INTENTIONAL_REPEATS: &[&str] = &[
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
fn in_quotes(ctx: &Ctx, pos: usize) -> bool {
    ctx.chars[..pos].iter().filter(|c| **c == '"').count() % 2 == 1
}

/// Lặp từ có chủ ý của TIẾNG ANH (vòng 24 — audit EN trap 1-17; đối chiếu
/// corpus `en_basic.ass`: "no no" KHÔNG trong whitelist — PO chốt là lỗi).
const EN_INTENTIONAL_REPEATS: &[&str] = &[
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

/// Bat loi lap tu: hai tu lien tiep giong nhau, cach nhau dung mot khoang trang.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    // Token: [Word][Space][Word] - can nhin 2 token lui.
    for i in 2..ctx.tokens.len() {
        let cur = ctx.tokens[i];
        let gap = ctx.tokens[i - 1];
        let prev = ctx.tokens[i - 2];
        if !is_word(&cur) || !is_word(&prev) {
            continue;
        }
        if gap.kind != TokenKind::Space || gap.len() != 1 {
            continue;
        }
        let cur_text = ctx.token_text(&cur);
        let prev_text = ctx.token_text(&prev);
        // Tu 1 ky tu de tranh bao dong gia ("a a").
        if cur_text.chars().count() < 2 {
            continue;
        }
        if cur_text.to_lowercase() != prev_text.to_lowercase() {
            continue;
        }
        // "luôn luôn", "mãi mãi"… la lap co chu y, khong phai loi (Fassist-94).
        // Vòng 26: chọn whitelist theo **TỪNG TỪ** chứ không theo câu — câu
        // mix ("check the the file nhé") có lặp từ EN giữa câu tiếng Việt:
        // từ ASCII → whitelist EN, từ có dấu Việt → whitelist VI.
        let lower = cur_text.to_lowercase();
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
        // ("Lê Lê ơi") và trích lời nói nguyên văn ("nói \"không không\"")
        // là hợp lệ bất kể bảng chữ.
        let cur_cap = cur_text.chars().next().is_some_and(|c| c.is_uppercase());
        let prev_cap = prev_text.chars().next().is_some_and(|c| c.is_uppercase());
        if cur_cap && prev_cap {
            continue;
        }
        if in_quotes(ctx, cur.start) {
            continue;
        }
        if ctx.is_masked(cur.start, cur.end) {
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
}
