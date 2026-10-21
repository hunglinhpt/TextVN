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
        if INTENTIONAL_REPEATS.contains(&cur_text.to_lowercase().as_str()) {
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

    #[test]
    fn khong_bat_hai_tu_khac_nhau() {
        assert!(check("của nhà").is_empty());
    }
}
