// SPDX-License-Identifier: GPL-3.0-or-later
//! **User Style Profile** (vòng 54 — P7): học từ/cặp sửa-lỗi **cá nhân** của
//! user (opt-in, tắt mặc định) cho Text Prediction (boost) và Trợ lý ngữ pháp
//! (correction ưu tiên, suggestion-only).
//!
//! Ràng buộc chủ repo:
//! - **Dữ liệu lưu KHÔNG được phép quá lớn**: words ≤ 200, corrections ≤ 50,
//!   mỗi từ ≤ 32 ký tự, tổng bytes của hai mảng ≤ 16 KB — kiểm + prune lúc
//!   nhận dữ liệu (`sanitize`), không bao giờ fail-open tắt cả tính năng.
//! - **G8**: chỉ từ đơn + số đếm + cặp sai→đúng tối thiểu — không bao giờ
//!   lưu đoạn văn/câu/dấu câu nội dung người dùng.
//! - Kết hợp từ điển cá nhân `english_words` hiện có (đường riêng, không đụng
//!   chính sách restore).

/// Số từ cá nhân tối đa.
pub const MAX_WORDS: usize = 200;
/// Số cặp correction cá nhân tối đa.
pub const MAX_CORRECTIONS: usize = 50;
/// Độ dài từ/cặp tối đa (ký tự).
pub const MAX_ENTRY_CHARS: usize = 32;
/// Tổng bytes JSON của hai mảng tối đa.
pub const MAX_TOTAL_BYTES: usize = 16 * 1024;

/// Một từ cá nhân: từ + số lần user dùng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleWord {
    pub word: String,
    pub count: u32,
}

/// Một cặp sửa-lỗi cá nhân user đã chấp nhận.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleCorrection {
    pub from: String,
    pub to: String,
}

/// Profile cá nhân đã **sanitize** (an toàn để engine dùng trực tiếp).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleProfile {
    pub words: Vec<StyleWord>,
    pub corrections: Vec<StyleCorrection>,
}

/// Từ/cặp có hợp lệ để lưu không? — chặn G8 (đoạn văn), ký tự điều khiển,
/// emoji/HTML, quá dài. Cho phép: chữ cái Unicode + dấu cách + `-` `'` `/`
/// `.` `đ` (đã là alphabetic). Từ ghép 2-3 âm tiết là hợp lệ và hữu ích.
fn valid_entry(s: &str) -> bool {
    let n = s.chars().count();
    if !(2..=MAX_ENTRY_CHARS).contains(&n) {
        return false;
    }
    // Khoảng trắng cho phép TỐI ĐA 2 (từ ghép 3 âm tiết) — chặn đoạn văn G8.
    if s.split_whitespace().count() > 3 {
        return false;
    }
    // Không có khoảng trắng đầu/cuối/kép (đã trim phía caller, giữ cứng).
    if s != s.trim() || s.contains("  ") {
        return false;
    }
    s.chars()
        .all(|c| c.is_alphabetic() || c == ' ' || c == '-' || c == '\'')
}

impl StyleProfile {
    /// Sanitize từ dữ liệu thô (config): lọc từ/cặp không hợp lệ, prune cap,
    /// giữ top theo count (words) / thứ tự (corrections — FIFO mới nhất cuối,
    /// prune bỏ đầu cũ). Trả về `None` khi kết quả trống (caller giữ default).
    pub fn sanitize(words: &[(String, u32)], corrections: &[(String, String)]) -> Option<Self> {
        let mut prof = Self::default();
        // Words: lọc + sort theo count desc (đứng đầu = dùng nhiều), prune cap.
        let mut ws: Vec<(String, u32)> = words
            .iter()
            .filter(|(w, _)| valid_entry(w))
            .map(|(w, n)| (w.trim().to_string(), (*n)))
            .collect();
        ws.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())));
        ws.dedup_by(|a, b| a.0.eq_ignore_ascii_case(&b.0));
        prof.words = ws
            .into_iter()
            .take(MAX_WORDS)
            .map(|(w, n)| StyleWord { word: w, count: n })
            .collect();
        // Corrections: lọc + FIFO giữ MỚI NHẤT (đuôi), trùng `from` giữ bản cuối.
        let mut seen_from: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut cs: Vec<(String, String)> = Vec::new();
        for (from, to) in corrections.iter().rev() {
            let (f, t) = (from.trim(), to.trim());
            if f.eq_ignore_ascii_case(t) || !valid_entry(f) || !valid_entry(t) {
                continue;
            }
            let key = f.to_lowercase();
            if seen_from.insert(key) {
                cs.push((f.to_string(), t.to_string()));
            }
        }
        cs.reverse(); // trở lại thứ tự thời gian (cũ → mới)
        if cs.len() > MAX_CORRECTIONS {
            cs = cs.split_off(cs.len() - MAX_CORRECTIONS);
        }
        prof.corrections = cs
            .into_iter()
            .map(|(f, t)| StyleCorrection { from: f, to: t })
            .collect();

        // Bytes cap — prune words từ cuõi (ít dùng) tới khi ≤ MAX_TOTAL_BYTES.
        while prof.total_bytes() > MAX_TOTAL_BYTES {
            if prof.words.len() > prof.corrections.len() && !prof.words.is_empty() {
                prof.words.pop();
            } else if !prof.corrections.is_empty() {
                prof.corrections.remove(0); // bỏ cặp cũ nhất
            } else {
                break;
            }
        }
        if prof.words.is_empty() && prof.corrections.is_empty() {
            None
        } else {
            Some(prof)
        }
    }

    /// Tổng bytes ước lượng của hai mảng khi serialize (gần đúng — mục đích
    /// cap, không phải dump chính xác).
    pub fn total_bytes(&self) -> usize {
        let words: usize = self.words.iter().map(|w| w.word.len() + 12).sum();
        let corr: usize = self
            .corrections
            .iter()
            .map(|c| c.from.len() + c.to.len() + 20)
            .sum();
        words + corr
    }

    /// Từ cá nhân có khớp prefix `typed` không (kèm weight theo count)?
    /// Trả về danh sách (từ, boost) — boost 90 + min(count, 30) để từ dùng
    /// nhiều nổi trên nguồn chung mà không phá luật ext-rank vòng 52.
    pub fn prefix_matches(&self, typed_lower: &str, limit: usize) -> Vec<(String, u32)> {
        if typed_lower.chars().count() < 2 {
            return Vec::new();
        }
        let mut out: Vec<(String, u32)> = self
            .words
            .iter()
            .filter(|w| {
                w.word.starts_with(typed_lower)
                    && w.word.chars().count() > typed_lower.chars().count()
            })
            .map(|w| (w.word.clone(), 90u32.saturating_add(w.count.min(30))))
            .collect();
        out.truncate(limit);
        out
    }
}

/// Tra cặp correction tuyến tính (≤ 50 mục — P7-DFMEA F11: đủ nhanh, không
/// cần cache per-instance phức tạp).
pub fn correction_for_linear<'a>(
    corrections: &'a [StyleCorrection],
    word_lower: &str,
) -> Option<&'a str> {
    let needle = word_lower.to_lowercase();
    corrections
        .iter()
        .find(|c| c.from.to_lowercase() == needle)
        .map(|c| c.to.as_str())
}

/// Bảng lookup tĩnh cho profile bất biến (dùng khi engine giữ profile tĩnh
/// qua OnceLock — điểm inject assist).
pub struct CorrectionLookup {
    map: std::collections::HashMap<String, String>,
}

impl CorrectionLookup {
    pub fn new(corrections: &[StyleCorrection]) -> Self {
        Self {
            map: corrections
                .iter()
                .map(|c| (c.from.to_lowercase(), c.to.clone()))
                .collect(),
        }
    }

    pub fn get(&self, word_lower: &str) -> Option<&str> {
        self.map.get(word_lower).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(v: &[(&str, u32)]) -> Vec<(String, u32)> {
        v.iter().map(|(a, b)| (a.to_string(), *b)).collect()
    }

    fn c(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn p7_validator_chan_g8_va_rac() {
        // Đoạn văn (G8) — chặn
        assert!(StyleProfile::sanitize(
            &w(&[("họp giao ban đầu tuần và báo cáo kết quả quý", 5)]),
            &[]
        )
        .is_none());
        // Quá dài / quá ngắn / rác
        for bad in [
            "a",
            "",
            "e\u{200D}m.x",
            "tôi<",
            "báo cáo;rm -rf",
            "\u{1F600}vui",
        ] {
            assert!(
                StyleProfile::sanitize(&w(&[(bad, 5)]), &[]).is_none(),
                "{bad}"
            );
        }
        // Hợp lệ: từ đơn + từ ghép ≤3 âm ("an toàn thông tin" 4 từ bị loại).
        let p = StyleProfile::sanitize(
            &w(&[("triển khai", 10), ("khả thi", 8), ("an toàn thông tin", 6)]),
            &[],
        )
        .unwrap();
        assert_eq!(p.words.len(), 2);
    }

    #[test]
    fn p7_cap_va_prune() {
        // > 200 từ — giữ 200 top count
        let many: Vec<(String, u32)> = (0..250)
            .map(|i| {
                (
                    format!(
                        "từ{}{}",
                        (b'a' + (i as u8 / 26 % 26)) as char,
                        (b'a' + i as u8 % 26) as char
                    ),
                    (i % 50) as u32 + 1,
                )
            })
            .collect();
        let p = StyleProfile::sanitize(&many, &[]).unwrap();
        assert_eq!(p.words.len(), MAX_WORDS);
        assert_eq!(p.words[0].count, 50);
        // > 50 corrections — giữ MỚI NHẤT (đuôi). Chữ thuần (validator chặn số).
        let many_c: Vec<(String, String)> = (0..60)
            .map(|i| {
                (
                    format!("sa{}{}", (b'a' + i / 26) as char, (b'a' + i % 26) as char),
                    format!("du{}{}", (b'z' - i / 26) as char, (b'z' - i % 26) as char),
                )
            })
            .collect();
        let p = StyleProfile::sanitize(&[], &many_c).unwrap();
        assert_eq!(p.corrections.len(), MAX_CORRECTIONS);
        assert_eq!(p.corrections[0].from, "saak"); // 60-50=10 đầu bị bỏ
        assert_eq!(p.corrections[49].to, "duxs");
    }

    #[test]
    fn p7_bytes_cap_16kb() {
        // 3-từ 25-30 ký tự (hợp lệ): 200 mục ≈ 8.8KB — bytes-cap là giám sát
        // defense-in-depth (caps hiện tại ≈ 13KB max < 16KB).
        let many: Vec<(String, u32)> = (0..MAX_WORDS)
            .map(|i| {
                (
                    format!(
                        "ba âm {}{}",
                        (b'a' + (i as u8 / 26 % 26)) as char,
                        (b'a' + i as u8 % 26) as char
                    ),
                    1,
                )
            })
            .collect();
        let p = StyleProfile::sanitize(&many, &[]).unwrap();
        assert_eq!(p.words.len(), MAX_WORDS);
        assert!(p.total_bytes() <= MAX_TOTAL_BYTES, "{}", p.total_bytes());
    }

    #[test]
    fn p7_prefix_matches_boost_theo_count() {
        let p = StyleProfile::sanitize(&w(&[("triển khai", 30), ("triển vọng", 5)]), &[]).unwrap();
        let m = p.prefix_matches("triển", 5);
        assert_eq!(m[0].0, "triển khai"); // count cao hơn
        assert!(m[0].1 > m[1].1);
        // Từ == typed (không mở rộng) bị loại.
        assert!(!p
            .prefix_matches("triển khai", 5)
            .iter()
            .any(|(w, _)| w == "triển khai"));
    }

    #[test]
    fn p7_correction_lookup() {
        let p = StyleProfile::sanitize(
            &[],
            &c(&[("sai soat", "ra soát"), ("chuyến bay bay", "chuyến bay")]),
        )
        .unwrap();
        assert_eq!(
            correction_for_linear(&p.corrections, "sai soat"),
            Some("ra soát")
        );
        assert_eq!(
            correction_for_linear(&p.corrections, "SAI SOAT"),
            Some("ra soát")
        );
        assert_eq!(correction_for_linear(&p.corrections, "khác"), None);
        // trùng from — giữ bản cuối
        let p = StyleProfile::sanitize(&[], &c(&[("xx", "yy"), ("xx", "zz")])).unwrap();
        assert_eq!(p.corrections.len(), 1);
        assert_eq!(p.corrections[0].to, "zz");
        let lk = CorrectionLookup::new(&p.corrections);
        assert_eq!(lk.get("xx"), Some("zz"));
    }
}
