// SPDX-License-Identifier: GPL-3.0-or-later
//! Auto-restore English — bug **B5** (`PLAN §1.3`: "gõ tiếng Anh ra dấu").
//!
//! Nhánh **cấu trúc** (đã làm): từ đã bị transform mà kết quả **không phải âm tiết Việt**
//! → trả lại đúng chuỗi phím người dùng gõ (kể cả marker) khi gặp ranh giới từ.
//! Ví dụ: `asdf` → fold `àd` (không hợp lệ) → Space → `asdf `.
//!
//! Nhánh **từ điển tiếng Anh** (đã làm, **opt-in**): `config.english_words[]` — danh sách
//! từ EN mà chủ gõ muốn giữ nguyên. `text` → `tết` **hợp lệ về cấu trúc** nên nhánh cấu trúc
//! không bắt được; chỉ khi chủ gõ khai báo `text` là "từ tôi hay gõ bằng tiếng Anh" thì
//! engine mới trả lại `text`.
//!
//! **Vì sao opt-in chứ không bật sẵn:** `test` → `tết` cũng là ca mơ hồ theo chiều ngược lại
//! — `tết` là từ Việt rất hay dùng. Tự bật danh sách sẽ **phá tiếng Việt**. Đây là quyết định
//! của người dùng, không phải suy đoán của engine. Mẫu danh sách: `data/stop_en.txt`
//! (sinh bằng chính engine này làm oracle, xem test `data_stop_en_file_is_self_consistent`).
//!
//! ESC (`restore_last`) đã là đường restore thủ công (P0-3 §1.1 `hotkeys.restore_last`).

use crate::validate::is_valid_word;

/// Từ đã gõ có nằm trong danh sách `english_words` không? (không phân biệt hoa thường)
fn listed(raw: &[char], english_words: &[String]) -> bool {
    if english_words.is_empty() {
        return false;
    }
    let typed: String = raw.iter().collect::<String>().to_lowercase();
    english_words.iter().any(|w| w.to_lowercase() == typed)
}

/// Có nên restore chuỗi gõ thay vì giữ kết quả transform?
///
/// `true` khi: chưa có gì để restore (`raw` rỗng / không biến đổi) → `false`;
/// kết quả fold **không** là âm tiết Việt hợp lệ → `true`;
/// hoặc chuỗi gõ nằm trong `english_words` (nhánh từ điển) → `true`.
pub fn should_restore(raw: &[char], display: &[char], english_words: &[String]) -> bool {
    if raw.is_empty() || raw == display {
        return false;
    }
    !is_valid_word(display) || listed(raw, english_words)
}

/// Nội dung `data/stop_en.txt` — nhúng lúc biên dịch (core **không** I/O: S1).
/// `None` nếu file không có trong bản build (bản dựng không kèm data).
pub const STOP_EN_DATA: &str = include_str!("../../../data/stop_en.txt");

/// Parse `data/stop_en.txt` (mỗi dòng 1 từ; `#` và dòng trống là chú thích).
pub fn parse_word_list(src: &str) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_lowercase())
        .collect()
}

/// Danh sách mẫu đi kèm bản build (rỗng nếu không có file data).
pub fn default_english_words() -> Vec<String> {
    parse_word_list(STOP_EN_DATA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{fold, Method};
    use crate::transform::DiacriticStyle;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    /// Fold kiểu engine (Telex, style mới, free marking) — trùng đường chạy thật.
    fn telex(raw: &str) -> Vec<char> {
        fold(&chars(raw), Method::Telex, DiacriticStyle::New, true)
    }

    #[test]
    fn restores_invalid_fold() {
        // "asdf" → fold "àd" (âm cuối 'd' không hợp lệ) → restore
        assert!(should_restore(&chars("asdf"), &chars("àd"), &[]));
        // "expect" → fold "ẽpect" (vần/âm cuối không hợp lệ) → restore
        assert!(should_restore(&chars("expect"), &chars("ẽpect"), &[]));
    }

    #[test]
    fn keeps_valid_vietnamese() {
        assert!(!should_restore(&chars("duocj"), &chars("được"), &[]));
        assert!(!should_restore(&chars("hoaf"), &chars("hoà"), &[]));
    }

    #[test]
    fn no_restore_when_unchanged() {
        assert!(!should_restore(&chars("hello"), &chars("hello"), &[]));
        assert!(!should_restore(&[], &[], &[]));
    }

    #[test]
    fn known_gap_dictionary_words_stay_folded() {
        // KHÔNG có danh sách: `text` → `tết` hợp lệ về cấu trúc → không restore
        // (đây là hành vi mặc định, và là lý do danh sách phải **opt-in**).
        assert!(!should_restore(&chars("text"), &chars("tết"), &[]));
    }

    #[test]
    fn danh_sach_tieng_anh_dong_gap() {
        let list = vec!["text".to_string()];
        // Có trong danh sách → restore dù fold hợp lệ về cấu trúc
        assert!(should_restore(&chars("text"), &chars("tết"), &list));
        // Không có trong danh sách → vẫn giữ fold
        assert!(!should_restore(&chars("nest"), &chars("nết"), &list));
    }

    #[test]
    fn danh_sach_khong_pha_tieng_viet_that() {
        // `tết` là từ Việt thật và `tet` không phải từ EN ai gõ → không bị danh sách đụng.
        let list = default_english_words();
        assert!(!should_restore(&chars("tet"), &chars("tết"), &list));
        // Ngược lại: chủ gõ khai báo `test` là từ EN → engine trả lại `test`
        assert!(should_restore(&chars("test"), &chars("tết"), &list));
    }

    #[test]
    fn danh_sach_bo_hoa_thuong() {
        let list = vec!["Text".to_string()];
        assert!(
            should_restore(&chars("text"), &chars("tết"), &list),
            "so khớp phải không phân biệt hoa thường"
        );
    }

    #[test]
    fn parse_bo_comment_va_dong_trong() {
        let src = "# chú thích\n\ntext\n  keep  \n#x\n";
        assert_eq!(
            parse_word_list(src),
            vec!["text".to_string(), "keep".to_string()]
        );
    }

    /// Mỗi dòng trong `data/stop_en.txt` phải **thỏa** điều kiện đã ghi trong file:
    /// fold bằng Telex khác gốc và là âm tiết hợp lệ về cấu trúc. Nếu không, mục đó vô
    /// nghĩa (không có gì để restore) và test này đỏ.
    #[test]
    fn data_stop_en_file_is_self_consistent() {
        let words = default_english_words();
        assert!(words.len() >= 20, "danh sách mẫu quá ngắn: {}", words.len());
        for w in &words {
            assert!(
                w.chars().all(|c| c.is_ascii_alphabetic()),
                "`{w}` phải là chữ cái ASCII không khoảng trắng"
            );
            let folded = telex(w);
            let f: String = folded.iter().collect();
            assert_ne!(
                &f, w,
                "`{w}` fold ra chính nó → không có biến đổi để restore"
            );
            assert!(
                is_valid_word(&folded),
                "`{w}` fold thành `{f}` — không phải âm tiết hợp lệ → mục vô nghĩa"
            );
        }
        // không trùng lặp
        let mut uniq = words.clone();
        uniq.sort();
        let before = uniq.len();
        uniq.dedup();
        assert_eq!(before, uniq.len(), "danh sách có từ trùng");
    }

    /// Ca ngược lại: những mục trong danh sách mà fold ra **từ Việt thật** là ca mơ hồ —
    /// phải được nêu rõ trong file data để người dùng biết trước khi bật.
    #[test]
    fn canh_bao_cai_muc_mo_ho() {
        let words = default_english_words();
        // `test` -> `tết` và `list` -> `lít` là từ Việt có nghĩa thật.
        for ambiguous in ["test", "list"] {
            assert!(
                words.iter().any(|w| w == ambiguous),
                "file data phải giữ `{ambiguous}` (minh hoạ ca mơ hồ) + có ghi chú cảnh báo"
            );
        }
    }
}
