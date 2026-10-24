// SPDX-License-Identifier: GPL-3.0-or-later
//! L0 — phân loại ngôn ngữ câu (P4-0 §5.1, D16; v1.2 thêm tiếng Anh).
//!
//! Câu được phân loại theo tỷ lệ từ có dấu tiếng Việt:
//! - **Vi-dominant** (≥ 50 % từ có dấu Việt): bộ luật tiếng Việt (như cũ).
//! - **En-dominant** (< 50 % và có từ chức năng tiếng Anh): bộ luật tiếng Anh
//!   (v1: các luật trung lập + a/an). Văn phòng thường gõ xen cụm tiếng Anh
//!   trong câu tiếng Việt ("vui lòng check lại file report nhé") — các luật
//!   **trung lập** (khoảng trắng, dấu câu, lặp từ, viết hoa đầu câu) chạy ở
//!   CẢ HAI chế độ nên câu mix luôn được kiểm tra.
//! - Không có từ nào → coi là Vi (không có gì để bỏ qua — giữ hành vi cũ).

use crate::tokenize::TokenKind;

/// Tỷ lệ từ "có dấu" tối thiểu để coi câu là tiếng Việt.
const VIETNAMESE_RATIO: f64 = 0.5;

/// Ngôn ngữ chi phối của câu — quyết định bộ luật chạy (xem [`classify`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Vi,
    En,
}

/// Số từ chức năng tiếng Anh tối thiểu để nhận câu En-dominant (chống câu
/// Việt chứa từ mượn ASCII như "file", "report" bị nhận nhầm sang En).
const MIN_EN_FUNCTION_WORDS: usize = 2;

/// Từ chức năng tiếng Anh phổ biến nhất (nhúng trong code — crate 0-I/O, S1;
/// `data/assist/en_function_words.txt` là bản tham chiếu dài hơn của PO).
const EN_FUNCTION_WORDS: &[&str] = &[
    "the", "is", "are", "was", "were", "be", "been", "to", "of", "and", "a", "an", "in", "that",
    "it", "you", "for", "on", "with", "as", "at", "by", "this", "from", "or", "not", "he", "she",
    "they", "we", "i", "his", "her", "its", "have", "has", "had", "do", "does", "did", "will",
    "would", "can", "could", "please", "check", "send", "thanks", "team", "meeting",
    // Từ-điểm văn phòng (không phải function word ngữ pháp nhưng là tín hiệu
    // EN mạnh trong câu mix của văn phòng Việt — vòng 24).
    "report", "file", "asap", "email", "update", "deadline", "review", "feedback",
];

/// Câu (đã tách chars) có **từ-điểm tiếng Anh** không — dùng để mở khoá luật
/// a/an trong câu Vi-dominant (vòng 28: "gửi a email nhé" có "email";
/// "an khang thịnh vượng" không có → luật a/an im lặng).
pub fn has_en_indicator(chars: &[char]) -> bool {
    for token in crate::tokenize::tokenize(&chars.iter().collect::<String>()) {
        if token.kind != TokenKind::Word {
            continue;
        }
        let word: String = chars[token.start..token.end]
            .iter()
            .collect::<String>()
            .to_lowercase();
        // "a"/"an" là TỪ TIẾNG VIỆT thật ("an khang") — chính nó không phải
        // tín hiệu EN (vòng 28: nếu tính thì mọi câu chứa "an" mở khoá a/an).
        if word == "a" || word == "an" {
            continue;
        }
        if EN_FUNCTION_WORDS.contains(&word.as_str()) {
            return true;
        }
    }
    false
}

/// Phân loại câu: Vi-dominant / En-dominant / không nhận diện (xem header).
///
/// - `Some(Vi)`: ≥ 50 % từ có dấu Việt (hành vi cũ) hoặc câu không có từ nào.
/// - `Some(En)`: < 50 % từ có dấu Việt **và** đủ dấu hiệu tiếng Anh (≥ 2 từ
///   chức năng EN) — câu mix như "Anh gửi report cho team ASAP".
/// - `None`: không nhận diện được (tên riêng, số, ký hiệu) — **im lặng hoàn
///   toàn**, đúng hành vi cũ của L0 (fail-open: thà bỏ qua còn hơn báo sai).
pub fn classify(text: &str) -> Option<Lang> {
    if is_vietnamese(text) {
        return Some(Lang::Vi);
    }
    let mut en_words = 0usize;
    let mut any_word = false;
    for token in crate::tokenize::tokenize(text) {
        if token.kind != TokenKind::Word {
            continue;
        }
        any_word = true;
        let word: String = text
            .chars()
            .skip(token.start)
            .take(token.end - token.start)
            .collect::<String>()
            .to_lowercase();
        if EN_FUNCTION_WORDS.contains(&word.as_str()) {
            en_words += 1;
        }
    }
    if !any_word {
        return Some(Lang::Vi); // câu rỗng/chỉ ký hiệu — giữ hành vi cũ
    }
    // Câu thuần ASCII không dấu (≥2 từ, không chữ Việt có dấu) — coi là
    // tiếng Anh: các luật tự chặn bảo vệ (diacritic/spelling cần chữ Việt).
    let chars: Vec<char> = text.chars().collect();
    let mut words = 0usize;
    let mut ascii_words = 0usize;
    for token in crate::tokenize::tokenize(text) {
        if token.kind != TokenKind::Word {
            continue;
        }
        words += 1;
        let all_ascii = chars[token.start..token.end]
            .iter()
            .all(|c| c.is_ascii_alphabetic());
        if all_ascii {
            ascii_words += 1;
        }
    }
    if words >= 1 && words == ascii_words {
        return Some(Lang::En);
    }
    if en_words >= MIN_EN_FUNCTION_WORDS {
        Some(Lang::En)
    } else {
        None
    }
}

/// Câu này có phải tiếng Việt (chi phối) không — kể cả câu **mix văn phòng**
/// ("vui lòng check lại file report nhé": chỉ ~30 % từ có dấu nhưng câu là
/// tiếng Việt có cụm tiếng Anh mượn, vòng 24).
///
/// - Câu rỗng / không có từ nào → `true` (không có gì để bỏ qua).
/// - ≥ 50 % từ có dấu Việt → `true`.
/// - Mix: ≥ 2 từ có dấu Việt và tỷ lệ ≥ 25 % → `true` (câu neo tiếng Việt,
///   cụm EN mượn ở giữa — luật VI tự chặn trên từ không dấu).
/// - Còn lại → `false`.
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
    let ratio = f64::from(vietnamese as u32) / f64::from(words as u32);
    ratio >= VIETNAMESE_RATIO || (vietnamese >= 2 && ratio >= MIXED_VI_RATIO)
}

/// Tỷ lệ từ có dấu tối thiểu cho câu **mix** (văn phòng gõ xen cụm EN):
/// thấp hơn ngưỡng chính nhưng đòi hỏi ≥ 2 từ có dấu (2 từ khó trùng ngẫu nhiên).
const MIXED_VI_RATIO: f64 = 0.25;

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

    /// Vòng 24 (trợ lý EN): phân loại câu theo ngôn ngữ chi phối.
    #[test]
    fn phan_loai_ngon_ngu() {
        use super::Lang;
        // Vi-dominant (kể cả câu mix có từ EN mượn — văn phòng).
        assert_eq!(classify("Tôi đi học ở trường."), Some(Lang::Vi));
        assert_eq!(
            classify("vui lòng check lại file report nhé"),
            Some(Lang::Vi)
        );
        assert_eq!(classify("Anh gửi report cho team nhé"), Some(Lang::Vi));
        // En-dominant.
        assert_eq!(
            classify("please check the report and send it to me"),
            Some(Lang::En)
        );
        assert_eq!(
            classify("the meeting is at 8 a.m. in the office"),
            Some(Lang::En)
        );
        // Câu mix chỉ 1 từ có dấu (gửi) nhưng 2 từ-điểm EN (report, team)
        // → En: luật trung lập vẫn phủ toàn câu.
        assert_eq!(classify("Anh gửi report cho team ASAP"), Some(Lang::En));
        // Câu mix neo-tiếng-Việt (≥2 từ có dấu, tỷ lệ ≥25 %) → Vi.
        assert_eq!(classify("gửi report cho tôi nhé"), Some(Lang::Vi));
        // Không nhận diện (tên riêng/số, không từ chức năng EN) — im lặng.
        // Tên riêng thuần Việt (2/3 từ có dấu) → Vi theo luật ≥50 % — hành vi
        // cũ; các luật tự im lặng trên tên riêng (không bêu gì).
        assert_eq!(classify("Nguyễn Văn A"), Some(Lang::Vi));
        // Cụm thuần ASCII ≥2 từ → En (luật tự chặn; không soi gì trên tên riêng).
        assert_eq!(classify("John Smith Jr"), Some(Lang::En));
        // 1 từ thuần ASCII → En (luật trung lập an toàn: "Stop !").
        assert_eq!(classify("Stop !"), Some(Lang::En));
        assert_eq!(classify("123 456 789"), Some(Lang::Vi)); // không từ nào
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
