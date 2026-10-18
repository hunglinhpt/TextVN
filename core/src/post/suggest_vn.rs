// SPDX-License-Identifier: GPL-3.0-or-later
//! Gợi ý hoàn tất từ tiếng Việt bằng **Tab** (`suggest.enabled` trong
//! `config.v1.json`, P0-3 §1.1 — mặc định TẮT, người dùng bật để gõ nhanh):
//! từ đang gõ (phần hiển thị đã fold) là tiền tố nghiêm ngặt của một âm tiết
//! trong `data/vn_freq3000.txt` (3000 âm tiết tần suất cao nhất — nguồn
//! hermitdave/FrequencyWords, OpenSubtitles 2018 qua OPUS) → thay toàn bộ từ
//! bằng âm tiết hoàn chỉnh và đóng từ, như Tab gợi ý EN (`restore_en.rs`).
//!
//! Tab chỉ hoàn tất khi có ứng viên; không có → Tab đi qua cho app (indent,
//! chuyển ô) như cũ. Chạy sau gõ tắt (macro) và trước Tab gợi ý EN.

/// `data/vn_freq3000.txt` — nhúng lúc biên dịch (core **không** I/O: S1).
pub const VN_FREQ_DATA: &str = include_str!("../../../data/vn_freq3000.txt");

/// Âm tiết hoàn chỉnh ngắn nhất bắt đầu bằng `typed_lower` (so khớp prefix
/// nghiêm ngặt, không phân biệt hoa thường đã xử lý phía caller). `None` khi
/// gõ quá ngắn hoặc không có ứng viên dài hơn.
pub fn complete_vn_word(typed_lower: &str) -> Option<String> {
    if typed_lower.chars().count() < 2 {
        return None;
    }
    let mut best: Option<&str> = None;
    for l in VN_FREQ_DATA.lines() {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        if !l.starts_with(typed_lower) || l == typed_lower {
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
        for l in VN_FREQ_DATA.lines() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            assert!(v(l), "`{l}`: không phải âm tiết Việt hợp lệ");
            n += 1;
        }
        assert!(n >= 2500, "danh sách quá ngắn: {n}");
    }

    #[test]
    fn hoan_tat_prefix_va_bo_qua() {
        assert_eq!(complete_vn_word("ngư"), Some("ngưng".to_string()));
        // Đã là từ đầy đủ → không ứng viên
        assert_eq!(complete_vn_word("người"), None);
        // Quá ngắn
        assert_eq!(complete_vn_word("t"), None);
        // Không có ứng viên
        assert_eq!(complete_vn_word("zz"), None);
    }
}
