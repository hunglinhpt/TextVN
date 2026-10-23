// SPDX-License-Identifier: GPL-3.0-or-later
//! L0 - bo qua cau **khong phai tieng Viet** (P4-0 §5.1, D16).
//!
//! V1 khong ho tro tieng Anh (D9) → neu cau thuoc tieng Anh thi cac luat
//! tieng Viet se bao dong gia (vi du: "this is a test." → bao "chu thuong o
//! dau cau"). L0 chay **truoc** cac luat khac va bo qua toan bo cau khi ty le
//! tu co dau thieu.

use crate::tokenize::TokenKind;

/// Tỷ lệ từ "có dấu" tối thiểu để coi câu là tiếng Việt.
const VIETNAMESE_RATIO: f64 = 0.5;

/// Câu này có phải tiếng Việt không (heuristic: tỷ lệ từ có dấu tiếng Việt).
///
/// - Câu rỗng / không có từ nào → `true` (không có gì để bỏ qua).
/// - Câu toàn tiếng Anh → `false` → bỏ qua, không báo lỗi nào.
pub fn is_vietnamese(text: &str) -> bool {
    // Offset cua token la **ky tu Unicode** -> phai tach thanh Vec<char>
    // (slice byte se panic voi tieng Viet co dau, Fassist-39).
    let chars: Vec<char> = text.chars().collect();
    let mut words = 0usize;
    let mut vietnamese = 0usize;
    for token in crate::tokenize::tokenize(text) {
        if token.kind != TokenKind::Word {
            continue;
        }
        words += 1;
        if chars[token.start..token.end]
            .iter()
            .any(|c| is_vietnamese_letter(*c))
        {
            vietnamese += 1;
        }
    }
    if words == 0 {
        return true;
    }
    f64::from(vietnamese as u32) / f64::from(words as u32) >= VIETNAMESE_RATIO
}

/// Ký tự là chữ cái có dấu đặc thù tiếng Việt (đ/ă/â/ê/ô/ơ/ư + dấu thanh).
///
/// Export công khai để luật spelling tái dùng (G15): từ **không chứa** chữ nào
/// trong danh sách này coi là thuật ngữ tiếng Anh / ASCII → bỏ qua.
///
/// So qua **chữ thường** của ký tự ⇒ câu VIẾT HOA TOÀN BỘ ("HÔM NAY TÔI ĐI
/// HỌC") cũng được L0 nhận là tiếng Việt (vòng 23: trước đây list chỉ có hoa
/// cho ă/â/ê/ô/ơ/ư/đ, thiếu toàn bộ nguyên âm mang thanh viết hoa ⇒ câu all-caps
/// có thể rơi dưới ngưỡng 0.5 và bị bỏ qua toàn bộ).
pub fn is_vietnamese_letter(ch: char) -> bool {
    let mut lower = ch.to_lowercase();
    let Some(first) = lower.next() else {
        return false;
    };
    if lower.next().is_some() {
        return false; // ánh xạ nhiều ký tự (İ → i̇) — không phải chữ Việt đơn
    }
    is_vietnamese_letter_lower(first)
}

/// Chữ thường có dấu đặc thù tiếng Việt (bảng gốc của L0).
fn is_vietnamese_letter_lower(ch: char) -> bool {
    matches!(
        ch,
        'đ' | 'Đ'
            | 'ă'
            | 'Ă'
            | 'â'
            | 'Â'
            | 'ê'
            | 'Ê'
            | 'ô'
            | 'Ô'
            | 'ơ'
            | 'Ơ'
            | 'ư'
            | 'Ư'
            | 'à'
            | 'á'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ằ'
            | 'ắ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ầ'
            | 'ấ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'è'
            | 'é'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ề'
            | 'ế'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'ì'
            | 'í'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ò'
            | 'ó'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ồ'
            | 'ố'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ờ'
            | 'ớ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ù'
            | 'ú'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ừ'
            | 'ứ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ỳ'
            | 'ý'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cau_tieng_viet() {
        assert!(is_vietnamese("Tôi đi học ở trường."));
        assert!(is_vietnamese("tôi tôi đi"));
    }

    /// Vòng 23: câu VIẾT HOA TOÀN BỘ cũng là tiếng Việt (ngưỡng 0.5 qua chữ
    /// thường hoá — "NAY" không có dấu nhưng "HÔM/TÔI/ĐI" có).
    #[test]
    fn cau_viet_hoa_toan_bo_van_la_tieng_viet() {
        assert!(is_vietnamese("HÔM NAY TÔI ĐI HỌC"));
        assert!(is_vietnamese("THUỞ XƯA CÓ MỘT CÔ BÉ."));
    }

    #[test]
    fn cau_tieng_anh() {
        assert!(!is_vietnamese("this is a test."));
        assert!(!is_vietnamese("Hello world"));
    }

    #[test]
    fn cau_rong() {
        assert!(is_vietnamese(""));
    }
}
