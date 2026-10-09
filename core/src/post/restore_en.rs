// SPDX-License-Identifier: GPL-3.0-or-later
//! Auto-restore English — bug **B5** (`PLAN §1.3`: "gõ tiếng Anh ra dấu").
//!
//! Ba nhánh, thứ tự kiểm tra trong `should_restore`:
//!
//! 1. **Cấu trúc**: từ đã bị transform mà kết quả **không phải âm tiết Việt**
//!    → trả lại đúng chuỗi phím người dùng gõ khi gặp ranh giới từ.
//!    Ví dụ: `asdf` → fold `àd` (không hợp lệ) → Space → `asdf `; `download`
//!    → `dơwnload` (không hợp lệ) → Space → `download `.
//!
//! 2. **Từ điển EN thông dụng dựng sẵn** (bật mặc định từ 2026-10-02): chuỗi
//!    gõ nằm trong `data/en_common.txt` — những từ EN mà Telex biến thành âm
//!    tiết Việt **hợp lệ về cấu trúc** (nhánh 1 không bắt được: `text` →
//!    `tẽt`, `see` → `sê`, `is` → `í`) → trả lại chuỗi gõ. Cặp mơ hồ hai chiều
//!    (`cow` là cách gõ Telex của `cơ`, `sex` → `sẽ`, `queen` → `quên`…) được
//!    giải quyết theo hướng **giữ tiếng Việt**: kết quả fold nằm trong
//!    `data/vn_common.txt` thì không restore. Người gõ EN vẫn cứu được bằng
//!    Escape hoặc thêm từ vào `config.english_words`.
//!
//! 3. **`config.english_words[]`** (opt-in): danh sách từ EN của chủ gõ —
//!    luôn restore, kể cả khi fold trùng âm tiết Việt thông dụng (quyết định
//!    tường minh của người dùng thắng mọi phỏng đoán).
//!
//! Opt-out toàn bộ: `config.auto_restore_english = false`.
//!
//! ESC (`restore_last`) là đường restore thủ công (P0-3 §1.1
//! `hotkeys.restore_last`); Tab gợi ý hoàn tất từ EN dùng chung dữ liệu ở
//! `complete_word`.

use crate::transform::stroke::is_stroke;
use crate::transform::tone::is_vowel;
use crate::validate::is_valid_word;

/// `data/en_common.txt` — nhúng lúc biên dịch (core **không** I/O: S1).
pub const EN_COMMON_DATA: &str = include_str!("../../../data/en_common.txt");
/// `data/vn_common.txt` — nhúng lúc biên dịch.
pub const VN_COMMON_DATA: &str = include_str!("../../../data/vn_common.txt");
/// `data/stop_en.txt` — danh sách mẫu cho `config.english_words` (opt-in).
pub const STOP_EN_DATA: &str = include_str!("../../../data/stop_en.txt");

/// Chuỗi gõ có nằm trong danh sách `english_words` của người dùng không?
/// (không phân biệt hoa thường)
fn listed(raw: &[char], english_words: &[String]) -> bool {
    if english_words.is_empty() {
        return false;
    }
    let typed: String = raw.iter().collect::<String>().to_lowercase();
    english_words.iter().any(|w| w.to_lowercase() == typed)
}

/// Data file có chứa dòng bằng đúng `word` không (so lowercase).
///
/// PERF-01 (audit 2026-10-04): lọc theo độ dài TRƯỚC khi `to_lowercase()`
/// từng dòng — mỗi dòng bằng `needle` bắt buộc cùng độ dài, nên bỏ được hầu
/// hết alloc trên đường gõ phím (từ điển 55-82 dòng, gọi ở ranh giới từ/Tab).
fn data_contains(data: &str, word: &str) -> bool {
    let needle = word.to_lowercase();
    data.lines().any(|l| {
        let l = l.trim();
        !l.is_empty()
            && !l.starts_with('#')
            && l.len() == needle.len()
            && l.to_lowercase() == needle
    })
}

/// Từ tiếng Anh thông dụng (nhánh 2)?
pub fn en_common_contains(word: &str) -> bool {
    data_contains(EN_COMMON_DATA, word)
}

/// Âm tiết Việt thông dụng được bảo vệ (đích fold phải tránh)?
pub fn vn_common_contains(syllable: &str) -> bool {
    data_contains(VN_COMMON_DATA, syllable)
}

/// Có nên restore chuỗi gõ thay vì giữ kết quả transform?
///
/// `true` khi: kết quả fold **không** là âm tiết Việt hợp lệ (nhánh 1); hoặc
/// chuỗi gõ là từ EN thông dụng mà fold **không** đụng âm tiết Việt thông
/// dụng (nhánh 2); hoặc nằm trong `english_words` của người dùng (nhánh 3).
/// `false` khi `raw` rỗng / không biến đổi.
pub fn should_restore(raw: &[char], display: &[char], english_words: &[String]) -> bool {
    if raw.is_empty() || raw == display {
        return false;
    }
    if listed(raw, english_words) {
        return true;
    }
    // Token không có nguyên âm mà vẫn bị biến đổi thì biến đổi đó là `đ` (dấu thanh/mũ/
    // sừng đều cần nguyên âm): `50.000đ`, `ĐT`, `ĐHQG`, `đc`, `đ/c` — chữ viết tắt và đơn
    // vị tiền rất thường gặp, tiếng Anh không có từ nào như vậy (R2-58; UniKey `processDd`
    // cũng cố ý cho `dd` trong chuỗi không phải tiếng Việt).
    if !display.iter().any(|&c| is_vowel(c)) && display.iter().any(|&c| is_stroke(c)) {
        return false;
    }
    if !is_valid_word(display) {
        return true;
    }
    let typed: String = raw.iter().collect::<String>().to_lowercase();
    let folded: String = display.iter().collect::<String>().to_lowercase();
    en_common_contains(&typed) && !vn_common_contains(&folded)
}

/// Parse một data word-list (mỗi dòng 1 từ; `#` và dòng trống là chú thích).
pub fn parse_word_list(src: &str) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_lowercase())
        .collect()
}

/// Danh sách mẫu đi kèm bản build cho `config.english_words` (rỗng nếu không có data).
pub fn default_english_words() -> Vec<String> {
    parse_word_list(STOP_EN_DATA)
}

/// **Tab gợi ý hoàn tất từ EN**: `typed` (lowercase) là tiền tố nghiêm ngặt của
/// đúng một từ EN thông dụng (ngắn nhất, rồi theo thứ tự bảng chữ) và fold
/// hiện tại không phải âm tiết Việt thông dụng → trả về từ hoàn chỉnh.
///
/// Chỉ chạy khi từ đã bị transform (word active) — từ chưa biến đổi nào là
/// tiếng Anh thuần, Tab phải đi qua cho app (indent, macro kế…).
pub fn complete_word(
    typed_lower: &str,
    folded_lower: &str,
    english_words: &[String],
) -> Option<String> {
    if typed_lower.len() < 2 || vn_common_contains(folded_lower) {
        return None;
    }
    let mut best: Option<String> = None;
    let consider = |best: &mut Option<String>, cand: &str| {
        if !cand.starts_with(typed_lower) || cand == typed_lower {
            return;
        }
        let better = match best {
            Some(b) => (cand.len(), cand) < (b.len(), b.as_str()),
            None => true,
        };
        if better {
            *best = Some(cand.to_owned());
        }
    };
    for l in EN_COMMON_DATA.lines() {
        let l = l.trim();
        if !l.is_empty() && !l.starts_with('#') {
            consider(&mut best, l);
        }
    }
    for w in english_words {
        // Từ điển cá nhân so khớp không phân biệt hoa thường (như `listed`): người
        // dùng thêm `VnExpress` thì `vn` + Tab vẫn phải gợi ý.
        let w = w.trim().to_lowercase();
        if !w.is_empty() {
            consider(&mut best, &w);
        }
    }
    best.map(|b| b.to_lowercase())
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

    /// R2-58: `đ` trong token không nguyên âm (giá tiền, chữ viết tắt) giữ nguyên.
    #[test]
    fn stroke_without_vowel_is_kept() {
        for (raw, display) in [
            ("dd", "đ"),
            ("000dd", "000đ"),
            ("DDT", "ĐT"),
            ("ddc", "đc"),
            ("DDHQG", "ĐHQG"),
            ("d9", "đ"),
        ] {
            assert!(
                !should_restore(&chars(raw), &chars(display), &[]),
                "`{raw}` → `{display}` phải giữ"
            );
        }
        // Có nguyên âm thì vẫn kiểm cấu trúc như cũ (`add` → `ađ` → restore).
        assert!(should_restore(&chars("add"), &chars("ađ"), &[]));
        // Từ điển cá nhân vẫn thắng.
        assert!(should_restore(
            &chars("ddc"),
            &chars("đc"),
            &["ddc".to_string()]
        ));
    }

    #[test]
    fn no_restore_when_unchanged() {
        assert!(!should_restore(&chars("hello"), &chars("hello"), &[]));
        assert!(!should_restore(&[], &[], &[]));
    }

    #[test]
    fn tu_en_thong_dung_fold_hop_le_duoc_restore() {
        // Từ trong en_common: fold hợp lệ về cấu trúc nhưng KHÔNG phải âm tiết
        // Việt thông dụng → restore (nhánh 2, bật mặc định 2026-10-02).
        for (raw, folded) in [
            ("text", "tẽt"),
            ("see", "sê"),
            ("is", "í"),
            ("yes", "yé"),
            ("saw", "să"),
        ] {
            assert!(
                should_restore(&chars(raw), &chars(folded), &[]),
                "`{raw}` → `{folded}` phải được restore"
            );
        }
    }

    #[test]
    fn cap_mo_ho_giu_tien_g_viet() {
        // Cặp mơ hồ hai chiều: fold là âm tiết Việt thông dụng → GIỮ fold
        // (người gõ tiếng Việt thắng — `cow` là cách gõ Telex của `cơ`).
        // `cow` không nằm trong en_common nên nhánh 2 không bắn; nếu ai thêm
        // vào `english_words` thì nhánh 3 (quyết định tường minh) thắng.
        let user_list = vec!["cow".to_string()];
        assert!(should_restore(&chars("cow"), &chars("cơ"), &user_list));
        // Không khai báo → giữ fold "cơ"
        assert!(!should_restore(&chars("cow"), &chars("cơ"), &[]));
    }

    /// R2-56: âm tiết Việt thật mà từ EN thông dụng fold vào phải được giữ —
    /// `thí`/`hí`/`vơ`/`bõ`/`gá` không có cách gõ Telex nào khác.
    #[test]
    fn am_tiet_viet_that_khong_bi_restore() {
        for raw in [
            "this", "his", "host", "lost", "most", "cost", "best", "vast", "arm", "vow", "row",
            "box", "gas",
        ] {
            let folded = telex(raw);
            let f: String = folded.iter().collect();
            assert!(
                !should_restore(&chars(raw), &folded, &[]),
                "`{raw}` → `{f}` là âm tiết Việt thật — phải giữ"
            );
        }
    }

    /// R2-57: Tab không thay âm tiết Việt thông dụng bằng từ EN (`có`+Tab → `cost`).
    #[test]
    fn tab_khong_cuop_am_tiet_viet_thong_dung() {
        for (typed, folded) in [("cos", "có"), ("bes", "bé"), ("vas", "vá"), ("los", "ló")] {
            assert_eq!(
                complete_word(typed, folded, &[]),
                None,
                "`{typed}` (`{folded}`) + Tab"
            );
        }
        // `có` được bảo vệ cả khi từ điển cá nhân có từ bắt đầu bằng `cos`.
        assert_eq!(complete_word("cos", "có", &["cosplay".to_string()]), None);
    }

    #[test]
    fn danh_sach_nguoi_dung_thang_moi_phong_doan() {
        let list = vec!["Text".to_string()];
        assert!(
            should_restore(&chars("text"), &chars("tẽt"), &list),
            "so khớp phải không phân biệt hoa thường"
        );
        // Từ không ở đâu cả nhưng người dùng khai báo → vẫn restore
        let list2 = vec!["nest".to_string()];
        assert!(should_restore(&chars("nest"), &chars("nết"), &list2));
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

    /// Oracle cho `en_common.txt`: MỌI mục phải **bắn** — fold khác chính nó,
    /// là âm tiết Việt hợp lệ, và KHÔNG nằm trong vn_common (nếu không, mục
    /// vô nghĩa: nhánh cấu trúc đã bắt, hoặc tiếng Việt đã thắng). Báo TẤT CẢ
    /// vi phạm trong một lần chạy để chỉnh data nhanh.
    #[test]
    fn data_en_common_file_fires() {
        let words = parse_word_list(EN_COMMON_DATA);
        // Ngưỡng chỉ chặn lỡ tay xoá trắng file: R2-56 đã bỏ các từ fold ra âm tiết
        // Việt thật (`this`→`thí`, `host`→`hót`…), danh sách còn ~20 mục.
        assert!(
            words.len() >= 15,
            "danh sách EN thông dụng quá ngắn: {}",
            words.len()
        );
        let mut uniq = words.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), words.len(), "en_common có từ trùng");
        let mut bad: Vec<String> = Vec::new();
        for w in &words {
            if !w.chars().all(|c| c.is_ascii_alphabetic()) {
                bad.push(format!("{w}: phải là chữ cái ASCII"));
                continue;
            }
            let folded = telex(w);
            let f: String = folded.iter().collect();
            if &f == w {
                bad.push(format!("{w}: fold ra chính nó — bỏ đi"));
            } else if !is_valid_word(&folded) {
                bad.push(format!(
                    "{w}: fold `{f}` KHÔNG hợp lệ — nhánh cấu trúc đã restore, bỏ đi"
                ));
            } else if vn_common_contains(&f) {
                bad.push(format!(
                    "{w}: fold `{f}` là âm tiết Việt thông dụng — GIỮ tiếng Việt, bỏ khỏi en_common"
                ));
            }
        }
        assert!(
            bad.is_empty(),
            "en_common có {} mục vô nghĩa:
{}",
            bad.len(),
            bad.join(
                "
"
            )
        );
    }

    /// Oracle cho `vn_common.txt`: mỗi mục phải là âm tiết Việt hợp lệ.
    #[test]
    fn data_vn_common_file_is_valid() {
        let syllables = parse_word_list(VN_COMMON_DATA);
        assert!(
            syllables.len() >= 30,
            "danh sách âm tiết Việt quá ngắn: {}",
            syllables.len()
        );
        let mut uniq = syllables.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), syllables.len(), "vn_common có mục trùng");
        let bad: Vec<String> = syllables
            .iter()
            .filter(|s| !is_valid_word(&chars(s)))
            .map(|s| format!("{s}: không phải âm tiết Việt hợp lệ"))
            .collect();
        assert!(
            bad.is_empty(),
            "vn_common có {} mục sai:
{}",
            bad.len(),
            bad.join(
                "
"
            )
        );
    }

    #[test]
    fn tab_goi_y_hoan_tu_tieng_anh() {
        // Tiền tố nghiêm ngặt → từ ngắn nhất khớp ("tes" đang fold thành "té")
        assert_eq!(complete_word("tes", "té", &[]), Some("test".to_string()));
        // Fold là âm tiết Việt thông dụng → không gợi ý (ưu tiên tiếng Việt)
        assert_eq!(complete_word("cow", "cơ", &[]), None);
        // Đã là từ đầy đủ → không gợi ý
        assert_eq!(complete_word("see", "sê", &[]), None);
        // Quá ngắn
        assert_eq!(complete_word("s", "s", &[]), None);
        // Danh sách người dùng tham gia ứng viên
        assert_eq!(
            complete_word("vn", "vn", &["vnexpress".to_string()]),
            Some("vnexpress".to_string())
        );
        // Không có ứng viên → None
        assert_eq!(complete_word("zzz", "zzz", &[]), None);
        // Từ điển cá nhân viết hoa vẫn khớp tiền tố gõ thường
        assert_eq!(
            complete_word("vn", "vn", &["VnExpress".to_string()]),
            Some("vnexpress".to_string())
        );
    }
}
