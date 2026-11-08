// SPDX-License-Identifier: GPL-3.0-or-later
//! P4-13 — từ điển **dấu thanh theo âm tiết**, dựng sẵn trong sản phẩm (0 phụ thuộc).
//!
//! Nguồn dữ liệu: `data/vn_syllables.txt` (dữ liệu riêng của repo — KHÔNG phải hunspell, ADR-014).
//!
//! # Luật quyết định (bảo thủ có chủ đích)
//!
//! Mỗi âm tiết được tách thành **khoá không dấu thanh** (giữ `ă â ê ô ơ ư`) + **thanh** (0..=5).
//! Từ điển nhóm theo khoá: `hoa → {ngang, huyền, sắc, hỏi, ngã, nặng}` → mỗi nhóm nhớ thanh nào là hợp lệ.
//!
//! - Khoá **không có** trong nhóm nào (từ mượn `ôtô`, tên riêng `Huệ` lạ, chữ Anh có dấu `café`) ⇒
//!   `contains = true`: luật chỉ bắt **lỗi dấu thanh**, không bắt "từ lạ".
//! - Khoá có nhóm nhưng **thanh viết ra không có trong dữ liệu** ⇒ `contains = false` (`chềt`, `bẵt`, `cũa`…).
//!   Dữ liệu = `vn_syllables.txt` (sinh tự động) + `vn_syllables_extra.txt` (bổ sung tay các âm tiết hiếm có
//!   thật mà bản sinh thiếu, vd. `ngoạc` trong "nguệch ngoạc"). Đã thử "ngữ âm cho phép mọi thanh ở âm tiết
//!   mở" để khỏi phải bổ sung — recall tổng hợp tụt 99 % → 20 % (ẩn gần hết lỗi `cũa`/`khuỷ`…) nên bỏ.
//! - Vị trí đặt dấu (`hòa` / `hoà`) **không ảnh hưởng** vì chỉ so (khoá, thanh) — hai kiểu đều qua.
//!
//! Vì vậy FPR thấp (mọi từ lạ im lặng) còn recall trên lỗi dấu thanh vẫn cao (thí nghiệm 893 lỗi tổng
//! hợp: 1,5 % → ~77 %, xem `docs/50-assist/P4-12-RECALL-REAL.md` §12–13).
//!
//! # Gợi ý và độ tin cậy
//!
//! Gợi ý = các thanh hợp lệ khác của cùng khoá; cặp hay nhầm (hỏi↔ngã, sắc↔nặng) lên đầu, sau đó theo
//! tần suất (thứ tự dòng trong dữ liệu: 3000 dòng đầu là tần suất cao nhất). Chỉ có **một** ứng viên ⇒
//! `confidence 80` (an toàn để "Sửa tất cả"); nhiều ứng viên ⇒ `55` (người dùng chọn, hành động sửa hàng
//! loạt bỏ qua — xem `ffi::plan_issues_from_raw_ex`).

use crate::dict::Dictionary;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Dữ liệu âm tiết dựng sẵn vào binary (sinh tự động bởi `cargo xtask gen-vocab`).
const DATA: &str = include_str!("../../data/vn_syllables.txt");
/// Âm tiết hiếm có thật mà bản sinh thiếu — sửa tay được (khác `DATA`).
const EXTRA: &str = include_str!("../../data/vn_syllables_extra.txt");

/// `confidence` khi chỉ có đúng một thanh thay thế hợp lệ.
pub const CONFIDENCE_UNAMBIGUOUS: u8 = 80;
/// `confidence` khi có nhiều thanh thay thế hợp lệ — không tự sửa hàng loạt.
pub const CONFIDENCE_AMBIGUOUS: u8 = 55;

/// 12 nguyên âm cơ sở × 6 thanh: ngang, huyền, sắc, hỏi, ngã, nặng.
const TONE_ROWS: [[char; 6]; 12] = [
    ['a', 'à', 'á', 'ả', 'ã', 'ạ'],
    ['ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ề', 'ế', 'ể', 'ễ', 'ệ'],
    ['i', 'ì', 'í', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ò', 'ó', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ù', 'ú', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự'],
    ['y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ'],
];

/// (chữ cơ sở, thanh) của một ký tự; `None` nếu không phải nguyên âm tiếng Việt.
fn decompose(c: char) -> Option<(char, u8)> {
    for row in &TONE_ROWS {
        if let Some(t) = row.iter().position(|&x| x == c) {
            return Some((row[0], t as u8));
        }
    }
    None
}

/// Kết quả tách một chữ: khoá không dấu thanh, thanh duy nhất (nếu có) và số dấu thanh gặp.
struct Split {
    key: String,
    tone: u8,
    marks: usize,
}

fn split(word: &str) -> Split {
    let mut key = String::with_capacity(word.len());
    let mut tone = 0u8;
    let mut marks = 0usize;
    for c in word.chars() {
        match decompose(c) {
            Some((base, t)) => {
                key.push(base);
                if t != 0 {
                    tone = t;
                    marks += 1;
                }
            }
            None => key.push(c),
        }
    }
    Split { key, tone, marks }
}

/// Cặp thanh hay bị nhầm khi gõ/đọc (ưu tiên lên đầu gợi ý).
fn confusable(a: u8, b: u8) -> bool {
    matches!((a, b), (3, 4) | (4, 3) | (2, 5) | (5, 2) | (1, 5) | (5, 1))
}

#[derive(Default)]
struct Family {
    /// Dạng chuẩn của từng thanh (kiểu đặt dấu của dữ liệu).
    forms: [Option<Box<str>>; 6],
    /// Thứ hạng tần suất (số dòng trong dữ liệu — nhỏ = phổ biến).
    rank: [u32; 6],
}

/// Từ điển âm tiết–thanh. Dùng [`SyllableTones::builtin`] cho bản dựng sẵn.
pub struct SyllableTones {
    families: HashMap<String, Family>,
}

impl SyllableTones {
    /// Dựng từ văn bản "một âm tiết mỗi dòng" (`#` = chú thích). Dòng không hợp lệ bị bỏ qua.
    pub fn from_text(text: &str) -> Self {
        let mut families: HashMap<String, Family> = HashMap::new();
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with('#')
                || line.chars().any(|c| c.is_whitespace() || c.is_control())
            {
                continue;
            }
            let lower = line.to_lowercase();
            let s = split(&lower);
            if s.marks > 1 || s.key.chars().count() > 8 {
                continue;
            }
            let fam = families.entry(s.key).or_default();
            let t = s.tone as usize;
            if fam.forms[t].is_none() {
                fam.forms[t] = Some(lower.into_boxed_str());
                fam.rank[t] = idx as u32;
            }
        }
        Self { families }
    }

    /// Bản dựng sẵn từ `data/vn_syllables.txt` — nạp một lần, dùng chung mọi luồng.
    pub fn builtin() -> &'static SyllableTones {
        static CELL: OnceLock<SyllableTones> = OnceLock::new();
        CELL.get_or_init(|| {
            let mut all = String::with_capacity(DATA.len() + EXTRA.len() + 1);
            all.push_str(DATA);
            all.push('\n');
            all.push_str(EXTRA);
            SyllableTones::from_text(&all)
        })
    }

    /// Số nhóm (khoá không dấu thanh) — dùng cho test/đo đạc.
    pub fn family_count(&self) -> usize {
        self.families.len()
    }

    /// Các thanh thay thế hợp lệ của `word` (đã lowercase), theo thứ tự gợi ý.
    fn alternatives(&self, word: &str) -> Vec<&str> {
        let s = split(word);
        let Some(fam) = self.families.get(&s.key) else {
            return Vec::new();
        };
        let mut cands: Vec<(u8, bool, u32, &str)> = Vec::new();
        for t in 0..6u8 {
            if s.marks <= 1 && t == s.tone {
                continue;
            }
            if let Some(form) = fam.forms[t as usize].as_deref() {
                cands.push((t, confusable(s.tone, t), fam.rank[t as usize], form));
            }
        }
        // Cặp hay nhầm trước, sau đó phổ biến trước.
        cands.sort_by_key(|&(_, conf, rank, _)| (!conf, rank));
        cands.into_iter().map(|c| c.3).collect()
    }
}

impl Dictionary for SyllableTones {
    fn contains(&self, word: &str) -> bool {
        let s = split(word);
        match self.families.get(&s.key) {
            // Khoá lạ → không phải lỗi dấu thanh (từ mượn, tên riêng): im lặng.
            None => true,
            Some(fam) => s.marks <= 1 && fam.forms[s.tone as usize].is_some(),
        }
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.alternatives(word)
            .into_iter()
            .take(limit)
            .map(str::to_string)
            .collect()
    }

    fn cheap_suggest(&self) -> bool {
        true
    }

    fn confidence(&self, word: &str, suggestions: &[String]) -> u8 {
        if suggestions.is_empty() {
            return 60;
        }
        if self.alternatives(word).len() == 1 {
            CONFIDENCE_UNAMBIGUOUS
        } else {
            CONFIDENCE_AMBIGUOUS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict() -> &'static SyllableTones {
        SyllableTones::builtin()
    }

    #[test]
    fn du_lieu_dung_san_du_lon() {
        assert!(dict().family_count() > 2000, "{}", dict().family_count());
    }

    #[test]
    fn am_tiet_dung_di_qua() {
        for w in [
            "được", "không", "người", "quân", "quẫn", "toàn", "hoà", "hòa", "hóa", "hoá", "tuỳ",
            "thuý",
        ] {
            assert!(dict().contains(w), "{w} phải hợp lệ");
        }
    }

    #[test]
    fn loi_dau_thanh_bi_bat() {
        // Khoá có thật nhưng thanh viết ra không phải âm tiết.
        assert!(!dict().contains("bẵt"));
        assert!(!dict().contains("tầt"));
        // Giới hạn đã biết: sai CHỮ (ọ thay ợ) làm khoá lạ → im lặng, để luật TYPOS/`diacritic_typo` lo.
        assert!(dict().contains("đưọc"));
    }

    #[test]
    fn thanh_khong_hop_le_cua_khoa_co_that() {
        // `bắt` hợp lệ, `bắc`… nhưng `băt` + ngã (`bẵt`) không phải âm tiết.
        assert!(dict().contains("bắt"));
        assert!(!dict().contains("bẵt"));
        // Âm tiết khép p/t/c/ch chỉ sắc/nặng: `tất`, `tật` hợp lệ; `tầt` thì không.
        assert!(dict().contains("tất") && dict().contains("tật"));
        assert!(!dict().contains("tầt"));
    }

    /// Âm tiết hiếm có thật mà bản sinh thiếu → nằm trong `vn_syllables_extra.txt`, không bị báo nhầm.
    #[test]
    fn am_tiet_hiem_trong_file_bo_sung_khong_bi_bao() {
        for w in ["ngoạc", "nhại", "ngoẹo", "nhễu", "nguệch"] {
            assert!(dict().contains(w), "{w} phải hợp lệ");
        }
        // Mọi dòng của file bổ sung đều được chấp nhận và có khoá hợp lệ.
        for line in EXTRA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
        {
            assert!(dict().contains(line), "{line}");
        }
        assert!(!dict().contains("chềt") && !dict().contains("hõc") && !dict().contains("tầp"));
    }

    #[test]
    fn tu_muon_va_khoa_la_im_lang() {
        for w in ["ôtô", "môtô", "café", "rơmoóc", "xyzzy", "123", "ǆ"] {
            assert!(dict().contains(w), "{w} phải im lặng");
        }
    }

    #[test]
    fn hai_dau_thanh_la_loi() {
        assert!(!dict().contains("hóá"), "hai dấu thanh trên một âm tiết");
        assert!(!dict().contains("hòá"));
    }

    #[test]
    fn goi_y_dung_thanh_va_thu_tu() {
        let s = dict().suggest("hõc", 3);
        assert!(!s.is_empty());
        assert!(s.contains(&"học".to_string()), "{s:?}");
        // ngã↔nặng không nằm trong cặp hay nhầm, nhưng `học` phải có mặt; `hóc/hộc`…
        let s2 = dict().suggest("bẵt", 3);
        assert!(s2.iter().all(|x| dict().contains(x)), "{s2:?}");
    }

    #[test]
    fn goi_y_luon_hop_le_va_khac_tu_goc() {
        for w in ["đưọc", "khôg", "bẵt", "tầt"] {
            for sug in dict().suggest(w, 5) {
                assert!(dict().contains(&sug), "{w} → {sug}");
                assert_ne!(sug, w);
            }
        }
    }

    #[test]
    fn mot_ung_vien_80_nhieu_ung_vien_55() {
        let d = dict();
        // `chêt` chỉ có thanh sắc → `chềt` có đúng một cách sửa.
        let one = d.suggest("chềt", 3);
        assert_eq!(one, vec!["chết".to_string()]);
        assert_eq!(d.confidence("chềt", &one), CONFIDENCE_UNAMBIGUOUS);
        // `hoc` có sắc + nặng → `hõc` mơ hồ.
        let two = d.suggest("hõc", 3);
        assert_eq!(two.len(), 2, "{two:?}");
        assert_eq!(d.confidence("hõc", &two), CONFIDENCE_AMBIGUOUS);
    }

    /// Bất biến dữ liệu: mọi âm tiết trong `vn_syllables.txt` đều được chấp nhận (không tự báo mình).
    #[test]
    fn moi_am_tiet_trong_du_lieu_deu_hop_le() {
        let d = dict();
        let mut n = 0;
        for line in DATA.lines() {
            let w = line.trim();
            if w.is_empty() || w.starts_with('#') || w.chars().any(char::is_whitespace) {
                continue;
            }
            let lower = w.to_lowercase();
            assert!(d.contains(&lower), "{lower}");
            n += 1;
        }
        assert!(n > 6000, "{n}");
    }

    #[test]
    fn khong_goi_y_thi_do_tin_cay_mac_dinh() {
        assert_eq!(dict().confidence("zzz", &[]), 60);
    }

    #[test]
    fn dong_bat_thuong_khong_panic() {
        let d = SyllableTones::from_text("#c\n\n\tx\nđược\nđược\nxx yy\nhóá\n\u{0}\nđuợc\n");
        assert!(d.family_count() >= 1);
        assert!(d.contains("được"));
        assert!(d.suggest("", 3).is_empty());
        assert!(d.suggest("\u{0}", 3).is_empty());
    }

    #[test]
    fn hoa_thuong_khong_phan_biet_khi_goi_qua_spelling() {
        // `contains` nhận chữ thường (spelling.rs lowercase trước); chữ hoa không phải dữ liệu.
        assert!(dict().contains("quân"));
    }

    #[test]
    fn thoi_gian_nap_va_tra_cuu() {
        let t = std::time::Instant::now();
        let d = SyllableTones::builtin();
        let load = t.elapsed();
        let t2 = std::time::Instant::now();
        for _ in 0..10_000 {
            let _ = d.contains("được");
            let _ = d.contains("đưọc");
        }
        let per = t2.elapsed() / 20_000;
        // Rộng rãi cho CI chậm: nạp < 500 ms (thực tế vài ms), tra < 50 µs.
        assert!(load.as_millis() < 500, "{load:?}");
        assert!(per.as_micros() < 50, "{per:?}");
    }
}
