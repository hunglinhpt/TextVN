// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 45 (đề xuất persona kế toán/HR/bán hàng) — định dạng vô lý:
//!
//! 1. **Ngày tháng** `dd/dd/yyyy` (bắt buộc có năm 4 chữ số — tránh phân số
//!    "32/13"): ngày > số ngày của tháng (calendar-aware — "31/02" không tồn
//!    tại), tháng > 12, ngày > 31. Suggestion-only — người dùng quyết.
//! 2. **Số tiền** có đ/vnd nhưng nhóm phân cách `.` hỗn loạn
//!    ("1.250.00đ" — nhóm cuối 2 chữ; "1.25.000đ" — nhóm giữa 2 chữ).
//!    "1.500.000đ" chuẩn → sạch.
//!
//! Quét trên `ctx.chars` trực tiếp (số = token Other, `/` = Punct — token
//! stream không giữ nguyên cụm). Guard: trong mask/quote không báo.

use super::{issue_suggestion, Ctx};
use crate::{Issue, IssueKind, Span};

/// Số ngày của tháng (không nhuận — 29/02 vẫn chấp nhận vì năm nhuận cần
/// kiểm năm; 30/02/31/02 luôn sai).
fn days_in_month(month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 29, // 29/02 hợp lệ nếu năm nhuận — không xác nhận được ⇒ im
        _ => 0,
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Bat loi dinh dang vo ly: ngay thang + so tien.
pub fn check(ctx: &Ctx, out: &mut Vec<Issue>) {
    if super::capitalize_sentence::looks_like_code_or_list(ctx) {
        return;
    }
    check_dates(ctx, out);
    check_money(ctx, out);
}

// ------------------------------------------------------------------ dates

fn check_dates(ctx: &Ctx, out: &mut Vec<Issue>) {
    let chars = &ctx.chars;
    let n = chars.len();
    let mut i = 0usize;
    // Pass EN-1 TRUNG-1: pattern ngắn nhất "d/d/dddd" = 8 chars — cũ 9
    // nuốt ngày nằm ĐÚNG CUỐI văn bản ("Hóa đơn ngày 31/4/2026"Miss).
    while i + 8 <= n {
        // Pattern: d{1,2} / d{1,2} / d{4} — quét thủ công, không regex.
        if !is_digit(chars[i]) {
            i += 1;
            continue;
        }
        // Phần ngày: 1-2 chữ số.
        let d_start = i;
        let mut d_end = i;
        while d_end < n && is_digit(chars[d_end]) && d_end - d_start < 2 {
            d_end += 1;
        }
        if d_end >= n || chars[d_end] != '/' {
            i += 1;
            continue;
        }
        // Phần tháng: 1-2 chữ số.
        let m_start = d_end + 1;
        let mut m_end = m_start;
        while m_end < n && is_digit(chars[m_end]) && m_end - m_start < 2 {
            m_end += 1;
        }
        if m_end >= n || chars[m_end] != '/' {
            i += 1;
            continue;
        }
        // Phần năm: ĐÚNG 4 chữ số (bắt buộc — tránh phân số "32/13").
        let y_start = m_end + 1;
        let mut y_end = y_start;
        while y_end < n && is_digit(chars[y_end]) {
            y_end += 1;
        }
        if y_end - y_start != 4 {
            i += 1;
            continue;
        }
        // Ranh giới từ sau năm: chữ số kế tiếp = năm khác dài hơn — skip.
        if y_end < n && is_digit(chars[y_end]) {
            i += 1;
            continue;
        }
        let day: u32 = chars[d_start..d_end]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        let month: u32 = chars[m_start..m_end]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        if ctx.is_masked(d_start, y_end) {
            i = y_end;
            continue;
        }
        let problem = if month == 0 || month > 12 {
            Some(format!("tháng {month} không tồn tại"))
        } else if day == 0 || day > 31 {
            Some(format!("ngày {day} không tồn tại"))
        } else {
            let dim = days_in_month(month);
            if day > dim {
                Some(format!("tháng {month} chỉ có {dim} ngày (đang viết {day})"))
            } else {
                None
            }
        };
        if let Some(why) = problem {
            let text: String = chars[d_start..y_end].iter().collect();
            out.push(issue_suggestion(
                IssueKind::InvalidDate,
                Span {
                    start: d_start,
                    end: y_end,
                },
                format!("Ngày tháng vô lý: \"{text}\" — {why}"),
                vec![],
            ));
        }
        i = y_end;
    }
}

// ------------------------------------------------------------------ money

/// Số tiền: cụm `[0-9](.[0-9]{1,3})+[đd]` hoặc suffix `vnd/VNĐ` — kiểm tra
/// các nhóm phân cách ĐỀU 3 chữ số (nhóm đầu 1-3). Sai ⇒ suggestion.
fn check_money(ctx: &Ctx, out: &mut Vec<Issue>) {
    let chars = &ctx.chars;
    let n = chars.len();
    let mut i = 0usize;
    while i < n {
        if !is_digit(chars[i]) {
            i += 1;
            continue;
        }
        // Quét cụm số + dấu chấm: digit (dot digit{1,3})*
        let start = i;
        let mut groups: Vec<(usize, usize)> = Vec::new(); // (start, end) từng nhóm
        let mut j = i;
        while j < n && is_digit(chars[j]) {
            j += 1;
        }
        groups.push((i, j));
        while j + 1 < n && chars[j] == '.' && is_digit(chars[j + 1]) {
            let g_start = j + 1;
            let mut g_end = g_start;
            while g_end < n && is_digit(chars[g_end]) {
                g_end += 1;
            }
            if g_end - g_start > 3 {
                break; // nhóm > 3 chữ số — có thể là số thập phân dài, bỏ qua
            }
            groups.push((g_start, g_end));
            j = g_end;
        }
        // Suffix tiền tệ VN: đ / Đ / vnd / VND / VNĐ (bắt ngay sau, không
        // space; VND/VNĐ hoa phổ biến trong bảng giá — Pass EN-1 TRUNG-2).
        let suffix_vnd = j < n
            && (chars[j] == 'v' || chars[j] == 'V')
            && j + 2 < n
            && (chars[j + 1] == 'n' || chars[j + 1] == 'N')
            && (chars[j + 2] == 'd' || chars[j + 2] == 'D' || chars[j + 2] == 'Đ');
        let suffix_is_vn = j < n && (chars[j] == 'đ' || chars[j] == 'Đ' || suffix_vnd);
        if groups.len() < 2 || !suffix_is_vn {
            i = j.max(i + 1);
            continue;
        }
        if ctx.is_masked(start, j) {
            i = j;
            continue;
        }
        // Kiểm các nhóm SAU NHÓM ĐẦU phải đúng 3 chữ số.
        let bad = groups
            .iter()
            .skip(1)
            .find(|&&(g_start, g_end)| g_end - g_start != 3);
        if let Some(&(g_start, g_end)) = bad {
            let text: String = chars[start..j].iter().collect();
            let digits = g_end - g_start;
            out.push(issue_suggestion(
                IssueKind::InvalidMoney,
                Span { start, end: j },
                format!(
                    "Số tiền phân cách hỗn loạn: \"{text}\" — nhóm {digits} chữ ở vị trí 3 chữ"
                ),
                vec![],
            ));
        }
        i = j.max(i + 1);
    }
}
