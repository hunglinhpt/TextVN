// SPDX-License-Identifier: GPL-3.0-or-later
//! Gợi ý hoàn tất từ tiếng Việt bằng **Tab** (`suggest.enabled` trong
//! `config.v1.json`, P0-3 §1.1 — mặc định TẮT, người dùng bật để gõ nhanh):
//! từ đang gõ (phần hiển thị đã fold) là tiền tố nghiêm ngặt của một âm tiết
//! trong `data/vn_syllables.txt` (vòng 52: 3000 âm tiết tần suất cao nhất từ
//! hermitdave/FrequencyWords + toàn bộ âm tiết còn thiếu từ
//! `data/assist/vi-DauMoi.dic` — tổng ~6700) → thay toàn bộ từ bằng âm tiết
//! hoàn chỉnh và đóng từ, như Tab gợi ý EN (`restore_en.rs`).
//!
//! Tab chỉ hoàn tất khi có ứng viên; không có → Tab đi qua cho app (indent,
//! chuyển ô) như cũ. Chạy sau gõ tắt (macro) và trước Tab gợi ý EN.

/// `data/vn_syllables.txt` — nhúng lúc biên dịch (core **không** I/O: S1).
pub const VN_SYLLABLES_DATA: &str = include_str!("../../../data/vn_syllables.txt");

/// Danh sách âm tiết đã **sắp alphabet** (build 1 lần — binary search).
fn vn_syllables_sorted() -> &'static [&'static str] {
    static CACHE: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static str> = VN_SYLLABLES_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        v.sort_unstable();
        v
    })
}

/// Âm tiết hoàn chỉnh ngắn nhất bắt đầu bằng `typed_lower` (so khớp prefix
/// nghiêm ngặt, không phân biệt hoa thường đã xử lý phía caller). `None` khi
/// gõ quá ngắn hoặc không có ứng viên dài hơn.
///
/// Vòng 52: quét tuyến tính ~6.700 dòng mỗi Tab/peek (peek chạy **mỗi phím**
/// khi ghost bật) → binary search tìm vùng prefix rồi chỉ scan vùng đó.
/// Ngữ nghĩa giữ nguyên: ngắn nhất, rồi theo thứ tự bảng chữ.
pub fn complete_vn_word(typed_lower: &str) -> Option<String> {
    if typed_lower.chars().count() < 2 {
        return None;
    }
    let sorted = vn_syllables_sorted();
    // Vùng ứng viên bắt đầu tại lower_bound(typed); kết thúc khi từ không còn
    // prefix typed (so sánh byte — dữ liệu đã là UTF-8 chuẩn NFC).
    let start = sorted.partition_point(|w| *w < typed_lower);
    let mut best: Option<&str> = None;
    for &l in &sorted[start..] {
        if !l.starts_with(typed_lower) {
            break;
        }
        if l == typed_lower {
            continue;
        }
        let better = match best {
            Some(b) => (l.chars().count(), l) < (b.chars().count(), b),
            None => true,
        };
        if better {
            best = Some(l);
        }
    }
    best.map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::is_valid_word;

    fn v(s: &str) -> bool {
        is_valid_word(&s.chars().collect::<Vec<_>>())
    }

    #[test]
    fn du_lieu_toan_bo_am_tiet_hop_le() {
        let mut n = 0;
        for l in VN_SYLLABLES_DATA.lines() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            assert!(v(l), "`{l}`: không phải âm tiết Việt hợp lệ");
            n += 1;
        }
        // Vòng 52: 3000 tần suất + ~3700 từ dic → tổng ~6700.
        assert!(n >= 6500, "danh sách quá ngắn: {n}");
    }

    #[test]
    fn du_lieu_khong_trung_lap() {
        let mut lines: Vec<&str> = VN_SYLLABLES_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        lines.sort_unstable();
        let before = lines.len();
        lines.dedup();
        assert_eq!(before, lines.len(), "vn_syllables có dòng trùng");
    }

    #[test]
    fn hoan_tat_prefix_va_bo_qua() {
        // Vòng 52: pool rộng hơn (dic) — "ngư" nay hoàn tất về âm tiết ngắn
        // nhất theo (độ dài, bảng chữ): "ngưa" (4) < "ngưu" (4) < "ngưng" (5).
        assert_eq!(complete_vn_word("ngư"), Some("ngưa".to_string()));
        // Đã là từ đầy đủ → không ứng viên
        assert_eq!(complete_vn_word("người"), None);
        // Quá ngắn
        assert_eq!(complete_vn_word("t"), None);
        // Không có ứng viên
        assert_eq!(complete_vn_word("zz"), None);
        // Âm tiết mới từ dic (không có trong 3000 cũ) vẫn hoàn tất được.
        assert!(complete_vn_word("hoan").is_some() || complete_vn_word("hoà").is_some());
    }
}
