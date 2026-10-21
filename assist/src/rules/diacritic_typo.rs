// SPDX-License-Identifier: GPL-3.0-or-later
//! L8 - sai vi tri dau hay gap ("nghia" sai thanh, "cua" sai thanh).
//!
//! Bang du lieu **co chon loc**: chi nhung loi khong phu thuoc kieu dau
//! (kieu moi/kiieu cu). Loi phu thuoc kieu dau ("thuy" vs "thuỷ", "hoa" vs
//! "hoà") de cho v2 khi engine biet kieu dau dang dung.

use super::{is_word, issue, Ctx};
use crate::{Fix, Issue, IssueKind, Span};

/// Bảng lỗi → lời viết đúng (chữ thường).
const TYPOS: &[(&str, &str)] = &[
    ("nghiã", "nghĩa"),
    ("nghỉa", "nghĩa"),
    ("cuả", "của"),
    ("điếm", "điểm"),
    ("tiêng", "tiếng"),
    ("thuở", "thướ"),
    ("fải", "phải"),
    ("nhứng", "nhưng"),
    // LƯU Ý: không thêm "trương" — đó là từ HỢP LỆ ("khẩn trương", "trầu trương");
    // KHONG them cac tu HOP LE: "truong" (khan truong), "yeu" (yeu tuong) - sai dau o day can ngu canh (Fassist-96).
    // sai dấu ở đây cần ngữ cảnh để phân biệt (Fassist-96).
    ("quyến", "quyển"),
    ("nguyến", "nguyễn"),
];

/// Bat từ sai vị trí dấu.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for token in &ctx.tokens {
        if !is_word(token) {
            continue;
        }
        let text = ctx.token_text(token);
        let lower = text.to_lowercase();
        let Some((_, correct)) = TYPOS.iter().find(|(wrong, _)| *wrong == lower) else {
            continue;
        };
        if ctx.is_masked(token.start, token.end) {
            continue;
        }
        // Giu hoa/thuong cua ky tu dau.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

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
}
