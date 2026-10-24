// SPDX-License-Identifier: GPL-3.0-or-later
//! L6 - chu thuong o dau cau ("chao ban" sau dau cham).

use super::{is_word, issue, Ctx};
use crate::tokenize::TokenKind;
use crate::{Fix, Issue, IssueKind, Span};

/// Bat chu cai dau cau viet thuong.
///
/// - Khong bat sau dau `)` `]` `}` hay dau cham giua tu.
/// - Bo qua tu co chu hoa ben trong (`iPhone`) hoac toan chu hoa (`NASA`).
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    for idx in 0..ctx.tokens.len() {
        let token = ctx.tokens[idx];
        if !is_word(&token) {
            continue;
        }
        if !is_sentence_start(ctx, idx) {
            continue;
        }
        let text = ctx.token_text(&token);
        let Some(first) = text.chars().next() else {
            continue;
        };
        if !first.is_lowercase() {
            continue;
        }
        // `iPhone` / `eBay` - co hoa sau ky tu dau -> khong phai dau cau_that.
        if text.chars().skip(1).any(|c| c.is_uppercase()) {
            continue;
        }
        // `NASA` - toan chu hoa -> khong phai dau cau_that.
        if text.chars().all(|c| !c.is_lowercase()) {
            continue;
        }
        if ctx.is_masked(token.start, token.end) {
            continue;
        }
        // Vòng 24 (audit EN trap 25-28): bảo vệ văn bản KHÔNG phải văn xuôi —
        // viết hoa hoá sẽ phá code/danh sách. TextVN chạy trong editor/terminal.
        if looks_like_code_or_list(ctx) {
            continue;
        }
        let first_upper = first.to_uppercase().to_string();
        out.push(issue(
            IssueKind::LowercaseSentenceStart,
            Span {
                start: token.start,
                end: token.start + 1,
            },
            "Chữ thường ở đầu câu".to_string(),
            vec![first_upper.clone()],
            Fix::Replace {
                start: token.start,
                end: token.start + 1,
                text: first_upper,
            },
        ));
    }
}

/// Câu có tín hiệu **code/danh sách** không (vòng 24 — fail-open: bỏ qua viết
/// hoa hoá chứ không sửa code)?
///
/// - Code: chứa `=` `;` `{` `}` (hiếm trong văn xuôi, phổ biến trong code —
///   "i = i + 1", "x = 5", "int main() {") hoặc `):`/"()" ("for i in range(10):").
/// - Danh sách: token đầu là 1-4 chữ thường + `.` ("i. introduction",
///   "a. list item") — marker đề mục, không phải câu.
fn looks_like_code_or_list(ctx: &Ctx) -> bool {
    let text: String = ctx.chars.iter().collect();
    if text.contains('=') || text.contains(';') || text.contains('{') || text.contains('}') {
        return true;
    }
    if text.contains("():") || text.contains("):") || text.contains("()") {
        return true;
    }
    // Marker đề mục: token 0 = từ ASCII 1-4 ký tự thường + dấu `.` ngay sau.
    if let Some(first) = ctx.tokens.first() {
        if first.kind == TokenKind::Word {
            let word: String = ctx.chars[first.start..first.end].iter().collect();
            let len = word.chars().count();
            if (1..=4).contains(&len)
                && word.chars().all(|c| c.is_ascii_lowercase())
                && ctx.chars.get(first.end) == Some(&'.')
            {
                return true;
            }
        }
    }
    false
}

/// Token co phai la dau cau khong (token 0, hoac sau dau ket thuc cau + khoang trang).
fn is_sentence_start(ctx: &Ctx, idx: usize) -> bool {
    if idx == 0 {
        // Ky tu dau doan chi la dau cau khi doan la cau tron ven.
        return ctx.start_of_sentence;
    }
    // Bo qua 1 token khoang trang ngay truoc.
    let mut j = idx;
    if ctx.tokens[j - 1].kind == TokenKind::Space {
        j -= 1;
    } else {
        return false;
    }
    if j == 0 {
        return false;
    }
    let prev = ctx.tokens[j - 1];
    prev.kind == TokenKind::Punct && super::is_real_sentence_end(ctx, &prev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;

    #[test]
    fn bat_dau_cau_thuong() {
        let issues = check("xin chào. bạn khỏe không");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, IssueKind::LowercaseSentenceStart);
    }

    #[test]
    fn khong_bat_dau_cau_da_hoa() {
        assert!(check("Xin chào. Bạn khỏe không").is_empty());
    }

    #[test]
    fn khong_bat_ten_rieng() {
        assert!(check("iPhone của tôi").is_empty());
    }
}
