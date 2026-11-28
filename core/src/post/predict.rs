// SPDX-License-Identifier: GPL-3.0-or-later
//! Bộ máy **Text Prediction** song ngữ Anh - Việt (P6).
//!
//! Cung cấp hai chức năng dự đoán chính qua cơ chế Inline Tab & Ghost Text:
//! 1. **Prefix Completion**: Hoàn tất từ đơn, từ ghép tiếng Việt (`phát` → `phát triển`)
//!    và từ vựng kỹ thuật/kinh doanh tiếng Anh (`techn` → `technology`).
//! 2. **Next-Word / Phrase Prediction**: Dự đoán cụm từ tiếp theo theo ngữ cảnh
//!    (`kính gửi ` + Tab → `quý khách hàng`, `thank you ` + Tab → `very much`).
//!
//! Bất biến:
//! - **0 I/O**: Dữ liệu nén nhúng tĩnh (`include_str!`).
//! - **Hiệu năng cao**: Tra cứu < 50 µs, 0 allocation khi reject (S1).
//! - **Bảo vệ tiếng Việt**: Tuân thủ luật R2-56 (từ Việt ngắn không bị cướp).
//! - **Chống tràn buffer**: Giới hạn độ dài chèn `<= MAX_TEXT = 64` (R1-2).
//!
//! Vòng 52 (mở rộng corpora ~31k mục): nguồn mới `en_words10k.txt`
//! (google-10000-english — CHỈ prediction, không đụng chính sách restore);
//! tối ưu quét: binary search trên danh sách đã sắp + bucket theo byte cuối
//! của khoá phrase + dedup HashSet — giữ NGUYÊN ngữ nghĩa xếp hạng cũ.
//! Vòng 59: thêm `en_office_phrases.tsv` (≥ 10.000 hàng văn phòng — upward/
//! downward/tech/business/everyday + bilingual Việt-Eng code-switch) —
//! cùng cơ chế bucket byte-cuối, lookup vẫn < 1 ms (test `en_office_phrases_10k`).

use crate::post::restore_en::{
    is_known_en_word, EN_COMMON_DATA, EN_FREQ_DATA, EN_WORDS_10K_DATA, TECH_BUSINESS_TERMS,
};
use crate::post::suggest_vn::VN_SYLLABLES_DATA;

/// E84 — từ điển Việt `data/assist/vi-DauMoi.dic` (nguồn sự thật "từ tiếng Việt
/// thật" trong repo): mục âm tiết là từ EN đã biết mà không có ở đây = nhiễm
/// corpus, loại khỏi prediction (xem `vn_syllables_sorted`).
const VN_DIC_DAUMOI_DATA: &str = include_str!("../../../data/assist/vi-DauMoi.dic");
use crate::transform::charset::strip_diacritics;
use crate::transform::tone::normalize_tone;
use crate::transform::DiacriticStyle;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Vòng 41 (perf audit): dữ liệu tĩnh parse **MỘT LẦN** qua OnceLock.
// Vòng 52: danh sách từ vựng sắp sẵn alphabet để binary search mỗi lần gọi —
// pool ~20k mục quét tuyến tính từng đo ~100+ µs/call trên đường peek.
// ---------------------------------------------------------------------------

/// Âm tiết VN đã sắp alphabet (binary search 3 biến thể dấu).
/// E84: corpus tần suất (OpenSubtitles) nhiễm tiếng Anh — mục là từ EN đã biết
/// VÀ không có trong từ điển Việt `vi-DauMoi.dic` ("guy", "king", "thing",
/// "my", "by", "kong", "on", "po", "them") bị loại tại điểm đọc. Từ Việt ASCII
/// thật ("em", "con", "do", "an", "song", "than"…) đều có trong dic — giữ nguyên.
/// `data/vn_syllables.txt` chưa regen (số corpora README chốt theo tệp) —
/// `cargo xtask gen-vocab` đã có cùng luật để regen ở lần bump số liệu kế.
fn vn_syllables_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let dic_set: std::collections::HashSet<&str> =
            VN_DIC_DAUMOI_DATA.lines().map(str::trim).collect();
        let dic_has = |w: &str| -> bool {
            let old = restyle(w, DiacriticStyle::Old);
            dic_set.contains(w) || dic_set.contains(old.as_str())
        };
        let mut v: Vec<&'static str> = VN_SYLLABLES_DATA
            .lines()
            .map(str::trim)
            .filter(|l| {
                // LOOP-2b: âm tiết có dấu (không ASCII) không thể là từ EN — tập EN toàn ASCII
                // (test `loop2b_tap_en_toan_ascii`) — bỏ `to_lowercase` + 2 lần tra cho ~6k mục.
                !l.is_empty()
                    && !l.starts_with('#')
                    && (!l.is_ascii() || !is_known_en_word(l) || dic_has(l))
            })
            .collect();
        v.sort_unstable();
        v
    })
}

/// R2-168 (ngosen#613 `Sose2026`): âm tiết có thật trong từ điển âm tiết
/// (`data/vn_syllables.txt`, ~6.7k mục kiểu dấu cũ) — so cả 3 biến thể kiểu dấu
/// (nguyên văn / cũ / mới) bằng binary search, không alloc ngoài 2 chuỗi restyle.
pub fn is_known_vn_syllable(word: &str) -> bool {
    let lower = word.to_lowercase();
    if lower.is_empty() {
        return false;
    }
    let sorted = vn_syllables_sorted();
    let hit = |w: &str| sorted.binary_search(&w).is_ok();
    hit(&lower)
        || hit(&restyle(&lower, DiacriticStyle::Old))
        || hit(&restyle(&lower, DiacriticStyle::New))
}

/// RSI-7 — âm tiết Việt đã **bỏ dấu**, sắp alphabet (build 1 lần qua OnceLock).
/// So dạng bỏ dấu thay vì byte gốc vì tiền tố đang gõ thường chưa có dấu:
/// `"gụ"` không bắt đầu bằng byte `"gu"` (ụ là 2 byte UTF-8) nhưng bỏ dấu thì
/// có — cổng ngôn ngữ phải bắt được cả trường hợp này. Xây TỪ danh sách đã lọc
/// E84 (`vn_syllables_sorted`) — một nguồn.
fn vn_syllables_deacc_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static str> = vn_syllables_sorted()
            .iter()
            .map(|l| {
                let de: String = strip_diacritics(&l.trim().chars().collect::<Vec<_>>())
                    .into_iter()
                    .collect();
                let de: &'static str = Box::leak(de.into_boxed_str());
                de
            })
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    })
}

/// RSI-7 — **cổng ngôn ngữ**: còn âm tiết Việt nào có thể lớn lên từ tiền tố
/// đang gõ không (tiền tố là âm tiết/tiền tố Việt hợp lệ)? Nhận tiền tố ở
/// BẤT KỲ dạng (có/không dấu, hoa/thường) — tự lowercase + `strip_diacritics`.
pub fn vn_prefix_possible(typed: &str) -> bool {
    if typed.chars().count() < 2 {
        // predict_impl chỉ chạy với tiền tố ≥ 2 ký tự; cổng cũng giữ chuẩn đó.
        return false;
    }
    let lower = typed.to_lowercase();
    let de: String = strip_diacritics(&lower.chars().collect::<Vec<_>>())
        .into_iter()
        .collect();
    vn_prefix_possible_unaccented(&de)
}

/// Lõi cổng — `typed_unaccented` đã lowercase + strip_diacritics (predict_impl
/// đã tính sẵn `typed_unaccented` → gọi lõi này, 0 alloc thêm trên đường nóng).
fn vn_prefix_possible_unaccented(typed_unaccented: &str) -> bool {
    let sorted = vn_syllables_deacc_sorted();
    let start = sorted.partition_point(|w| *w < typed_unaccented);
    sorted[start..]
        .iter()
        .take_while(|w| w.starts_with(typed_unaccented))
        .next()
        .is_some()
}

/// Từ ghép kèm trọng số: `(word, deaccented, weight)` — deaccent build 1 lần
/// (trước đây tính strip_diacritics mỗi từ mỗi call → alloc × ~1200 mục).
fn vn_compounds() -> &'static [(&'static str, &'static str, u32)] {
    static CACHE: OnceLock<Vec<(&'static str, &'static str, u32)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        VN_COMPOUNDS_DATA
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let mut parts = line.split('\t');
                let word = parts.next().unwrap_or("").trim();
                let weight: u32 = parts
                    .next()
                    .and_then(|w| w.trim().parse().ok())
                    .unwrap_or(50);
                if word.is_empty() {
                    return None;
                }
                let deaccented: String = strip_diacritics(&word.chars().collect::<Vec<_>>())
                    .into_iter()
                    .collect();
                let de: &'static str = Box::leak(deaccented.into_boxed_str());
                Some((word, de, weight))
            })
            .collect()
    })
}

fn en_common_lines() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        EN_COMMON_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}

/// Vòng 53 (perf): từ ghép bucket theo **byte đầu** — giữ cho `restore_en` dùng chung.
/// Nhóm từ ghép tiếng Anh theo byte đầu: (từ, weight).
pub(crate) type EnCompoundRow = (&'static str, u32);

pub(crate) fn en_compounds() -> &'static [EnCompoundRow] {
    static CACHE: OnceLock<Vec<EnCompoundRow>> = OnceLock::new();
    CACHE.get_or_init(|| {
        EN_COMPOUNDS_DATA
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let mut parts = line.split('\t');
                let word = parts.next().unwrap_or("").trim();
                let weight: u32 = parts
                    .next()
                    .and_then(|w| w.trim().parse().ok())
                    .unwrap_or(50);
                if word.is_empty() {
                    return None;
                }
                Some((word, weight))
            })
            .collect()
    })
}

pub(crate) fn en_compounds_buckets(
) -> &'static std::collections::HashMap<u8, &'static [EnCompoundRow]> {
    static CACHE: OnceLock<std::collections::HashMap<u8, &'static [EnCompoundRow]>> =
        OnceLock::new();
    CACHE.get_or_init(|| {
        let mut groups: std::collections::HashMap<u8, Vec<EnCompoundRow>> =
            std::collections::HashMap::new();
        for &(word, weight) in en_compounds() {
            if let Some(&b0) = word.as_bytes().first() {
                groups.entry(b0).or_default().push((word, weight));
            }
        }
        groups
            .into_iter()
            .map(|(k, v)| {
                let slice: &'static [EnCompoundRow] = Box::leak(v.into_boxed_slice());
                (k, slice)
            })
            .collect()
    })
}

fn tech_terms_lower() -> &'static [(&'static str, &'static str)] {
    static CACHE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        TECH_BUSINESS_TERMS
            .iter()
            .map(|t| {
                let lower: String = t.to_lowercase();
                let lo: &'static str = Box::leak(lower.into_boxed_str());
                (*t, lo)
            })
            .collect()
    })
}

/// Danh sách EN đã sắp alphabet — 1 binary search / call.
fn en_freq_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static str> = EN_FREQ_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        v.sort_unstable();
        v
    })
}

/// google-10000-english (vòng 52) — nguồn PREDICTION prefix, đã sắp.
fn en_words10k_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static str> = EN_WORDS_10K_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        v.sort_unstable();
        v
    })
}

/// Hàng TSV ngữ cảnh đã parse: `(key, prediction, weight)`.
type PhraseRow = (&'static str, &'static str, u32);

fn vn_phrase_rows() -> &'static [PhraseRow] {
    static CACHE: OnceLock<Vec<PhraseRow>> = OnceLock::new();
    CACHE.get_or_init(|| parse_phrase_rows(VN_PHRASES_DATA))
}

fn en_phrase_rows() -> &'static [PhraseRow] {
    static CACHE: OnceLock<Vec<PhraseRow>> = OnceLock::new();
    CACHE.get_or_init(|| parse_phrase_rows(EN_PHRASES_DATA))
}

/// Hàng bảng văn phòng (vòng 59) — parse một lần như hai bảng cũ.
fn office_phrase_rows() -> &'static [PhraseRow] {
    static CACHE: OnceLock<Vec<PhraseRow>> = OnceLock::new();
    CACHE.get_or_init(|| parse_phrase_rows(EN_OFFICE_PHRASES_DATA))
}

fn parse_phrase_rows(data: &'static str) -> Vec<PhraseRow> {
    data.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.split('\t');
            let (Some(key), Some(pred)) = (parts.next(), parts.next()) else {
                return None;
            };
            let (key, pred) = (key.trim(), pred.trim());
            if key.is_empty() || pred.is_empty() {
                return None;
            }
            let weight: u32 = parts
                .next()
                .and_then(|w| w.trim().parse().ok())
                .unwrap_or(50);
            Some((key, pred, weight))
        })
        .collect()
}

/// Bảng phrase đang đánh bucket (vòng 59: thêm bảng văn phòng thứ ba).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhraseTable {
    Vn,
    En,
    Office,
}

/// Bucket các hàng phrase theo **byte cuối của khoá** (vòng 52): ngữ cảnh chỉ
/// có thể khớp khoá mà kết thúc bằng đúng byte cuối của nó → mỗi call chỉ quét
/// bucket tương ứng (giảm ~20-30× với ~10k hàng thay vì quét toàn bảng).
/// Vòng 53: nhận bảng tường minh thay so-con-trỏ (an toàn khi một bảng rỗng).
fn phrase_buckets(
    rows: &'static [PhraseRow],
    table: PhraseTable,
) -> &'static std::collections::HashMap<u8, &'static [PhraseRow]> {
    static VN: OnceLock<std::collections::HashMap<u8, &'static [PhraseRow]>> = OnceLock::new();
    static EN: OnceLock<std::collections::HashMap<u8, &'static [PhraseRow]>> = OnceLock::new();
    static OFFICE: OnceLock<std::collections::HashMap<u8, &'static [PhraseRow]>> = OnceLock::new();
    let cache = match table {
        PhraseTable::Vn => &VN,
        PhraseTable::En => &EN,
        PhraseTable::Office => &OFFICE,
    };
    cache.get_or_init(|| {
        let mut groups: std::collections::HashMap<u8, Vec<PhraseRow>> =
            std::collections::HashMap::new();
        for &row in rows {
            let last = *row.0.as_bytes().last().unwrap_or(&b' ');
            groups.entry(last).or_default().push(row);
        }
        groups
            .into_iter()
            .map(|(k, v)| {
                let slice: &'static [PhraseRow] = Box::leak(v.into_boxed_slice());
                (k, slice)
            })
            .collect()
    })
}

/// Đặt lại dấu thanh từng từ theo `style` (`hòa` ↔ `hoà`, `thủy` ↔ `thuỷ`).
///
/// Dữ liệu dự đoán viết theo **kiểu cũ**; người dùng chọn kiểu mới thì ngữ cảnh
/// gõ ra là `hoà` — phải đưa về cùng kiểu mới so khớp được, và ứng viên chèn
/// vào phải theo kiểu người dùng (không trộn hai kiểu trong một văn bản).
pub fn restyle(text: &str, style: DiacriticStyle) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    fn flush(word: &mut Vec<char>, out: &mut String, style: DiacriticStyle) {
        if !word.is_empty() {
            normalize_tone(word, style);
            out.extend(word.iter());
            word.clear();
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut word: Vec<char> = Vec::new();
    for c in text.chars() {
        if c.is_alphabetic() {
            word.push(c);
        } else {
            flush(&mut word, &mut out, style);
            out.push(c);
        }
    }
    flush(&mut word, &mut out, style);
    out
}

/// Từ điển âm tiết tiếng Việt (vòng 52: 3000 tần suất + dict bổ sung).
pub const VN_COMPOUNDS_DATA: &str = include_str!("../../../data/vn_compounds.txt");

/// Từ điển từ ghép & cụm danh từ tiếng Anh thông dụng (~1957 mục, 16 domain).
pub const EN_COMPOUNDS_DATA: &str = include_str!("../../../data/en_compounds.txt");

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Việt (TSV: context \t prediction \t weight).
pub const VN_PHRASES_DATA: &str = include_str!("../../../data/vn_phrases.tsv");

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Anh (TSV: context \t prediction \t weight).
pub const EN_PHRASES_DATA: &str = include_str!("../../../data/en_phrases.tsv");

/// Bảng ngữ cảnh câu/cụm **văn phòng** tiếng Anh (vòng 59 — ≥ 10.000 hàng):
/// upward reporting / downward management / tech / business / everyday glue /
/// bilingual Viet-Eng code-switch (key Việt-Eng trộn → prediction tiếng Anh).
/// Sinh bởi `cargo xtask gen-en-office` (E59) — KHÔNG sửa tay.
pub const EN_OFFICE_PHRASES_DATA: &str = include_str!("../../../data/en_office_phrases.tsv");

/// Ứng viên prefix với thông tin xếp hạng — giữ `&'static str` (vòng 52: không
/// alloc String cho ứng viên bị loại; chỉ `take(limit)` cuối cùng mới cấp phát).
#[derive(Debug, Clone)]
struct PrefixCandidate {
    word: std::borrow::Cow<'static, str>,
    exact_match: bool,
    char_count: usize,
    weight: u32,
    /// Từ ghép có ÂM ĐẦU == tiền tố ("tiến độ" cho "tiến") — mở rộng đúng
    /// nghĩa; compound chỉ prefix-match byte ("người yêu" cho "ngư") thì false.
    ext: bool,
    /// P7 (vòng 54): từ CÁ NHÂN (style_profile) — khoá sort TOP. List rỗng
    /// mặc định → khoá này không đổi kết quả gì (0 hồi quy).
    personal: bool,
}

/// Dự đoán hoàn tất tiền tố (Prefix Completion).
///
/// Trả về danh sách ứng viên (tối đa `limit`), xếp theo độ ưu tiên:
/// khớp trực tiếp (có dấu) > độ dài ký tự ngắn hơn (hoàn tất âm tiết trước) > weight cao hơn.
pub fn predict_prefix_candidates(typed_lower: &str, is_vi: bool, limit: usize) -> Vec<String> {
    // Lời gọi trần (test / bench hiệu năng) = permissive `en_context = true`:
    // hợp đồng 0.2.80 `prefix_en_khong_bi_che_o_che_do_vi` — nguồn EN KHÔNG bị
    // cấm ở mode VI khi không biết ngữ cảnh. Engine (`try_vn_complete`) truyền
    // ngữ cảnh thật → cổng RSI-7 chỉ hoạt động nơi có ngữ cảnh.
    predict_prefix_candidates_with_personal(typed_lower, is_vi, limit, &[], 0, true)
}

/// P7 (vòng 54): như [`predict_prefix_candidates`] + ứng viên CÁ NHÂN
/// (`style_profile.words` — (từ, count)) — `personal: true` xếp TOP.
/// List rỗng → hành vi byte-for-byte cũ (0 hồi quy).
/// `style_touches` = số entry CUỐI danh sách `personal` được engine touch kể từ
/// lần nạp options (F-A1-2 audit vòng 59) — chỉ chúng có recency boost; ngoài
/// phạm vi đó thứ tự là count-desc (sanitize sort sau load) và boost phải = 0.
/// `en_context` (RSI-7): từ ngay trước từ đang gõ là từ tiếng Anh thật — cho
/// phép nguồn EN kể cả khi tiền tố là tiền tố/âm tiết Việt hợp lệ (code-switch
/// "gửi report" vẫn hoàn tất từ EN); `false` + tiền tố Việt hợp lệ → ngừng
/// nguồn EN tĩnh (giảm nhiễu chữ mờ VI: `gu`→`guys`, `ho`→`horse`, `ba`→`bars`).
pub fn predict_prefix_candidates_with_personal(
    typed_lower: &str,
    is_vi: bool,
    limit: usize,
    personal: &[(String, u32)],
    style_touches: usize,
    en_context: bool,
) -> Vec<String> {
    predict_impl(
        typed_lower,
        is_vi,
        limit,
        personal,
        style_touches,
        en_context,
    )
}

/// Ngưỡng số ứng viên tối thiểu trong cùng tiền tố để dựng sẵn bảng top-K.
const HEAVY_PREFIX_THRESHOLD: usize = 24;
/// Số ứng viên dựng sẵn tối đa cho mỗi tiền tố nặng (đủ phủ mọi `limit <= 24`).
const HEAVY_TOP_K: usize = 24;
/// Độ dài tiền tố (tính theo ký tự) được đánh chỉ mục top-K sẵn. LOOP-2b: 2..=3 —
/// độ dài 4–5 chỉ có ~10 nhóm > 24 mục nhưng tốn ~0,7 ms dựng lúc khởi động lạnh;
/// dải `partition_point` của tiền tố ≥ 4 ký tự đủ ngắn để quét trực tiếp (< 1 µs).
const HEAVY_MAX_PREFIX_CHARS: usize = 3;

/// LOOP-2b — băm FNV-1a 64-bit cho các bảng tra tĩnh (khoá là dữ liệu nhúng hoặc
/// tiền tố người dùng đang gõ, không có bề mặt tấn công HashDoS: bảng chỉ đọc,
/// dựng một lần). Nhanh hơn SipHash mặc định ~3× trên khoá ngắn — cắt thời gian
/// dựng lexicon lúc khởi động lạnh và lượt tra `get(v)` mỗi phím.
#[derive(Default, Clone, Copy)]
struct Fnv1a(u64);

impl std::hash::Hasher for Fnv1a {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        let mut h = if self.0 == 0 {
            0xcbf2_9ce4_8422_2325
        } else {
            self.0
        };
        for &b in bytes {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = h;
    }
}

type FnvBuild = std::hash::BuildHasherDefault<Fnv1a>;
type FnvMap<K, V> = std::collections::HashMap<K, V, FnvBuild>;
type FnvSet<K> = std::collections::HashSet<K, FnvBuild>;

const ENTRY_FROM_VN: u8 = 1;
const ENTRY_COMPOUND_SRC: u8 = 2;
/// Mục EN mà từ đã có trong tập Việt (first-writer-wins của pool VN+EN cũ).
const ENTRY_SHADOWED_BY_VN: u8 = 4;

/// 40 byte/mục (2 `&str` + weight + char_count + cờ) — mỗi từ lưu MỘT lần trong
/// lexicon dùng chung (LOOP-2b), chế độ ngôn ngữ chỉ là bộ lọc [`PoolMode::admits`].
#[derive(Debug, Clone, Copy)]
struct StaticEntry {
    word: &'static str,
    key: &'static str,
    weight: u32,
    char_count: u16,
    flags: u8,
}

impl StaticEntry {
    #[inline]
    fn is_vn(&self) -> bool {
        self.flags & ENTRY_FROM_VN != 0
    }
    #[inline]
    fn is_compound_src(&self) -> bool {
        self.flags & ENTRY_COMPOUND_SRC != 0
    }
    #[inline]
    fn shadowed_by_vn(&self) -> bool {
        self.flags & ENTRY_SHADOWED_BY_VN != 0
    }
}

#[derive(Debug, Clone, Copy)]
struct UnaccCompoundEntry {
    word: &'static str,
    deaccented: &'static str,
    char_count: u16,
    weight: u32,
    is_single: bool,
}

type StaticRankCmp = fn(&StaticEntry, &StaticEntry) -> std::cmp::Ordering;

fn rank_single_vi(a: &StaticEntry, b: &StaticEntry) -> std::cmp::Ordering {
    a.char_count
        .cmp(&b.char_count)
        .then_with(|| b.weight.cmp(&a.weight))
        .then_with(|| a.word.cmp(b.word))
}

fn rank_weight_first(a: &StaticEntry, b: &StaticEntry) -> std::cmp::Ordering {
    b.weight
        .cmp(&a.weight)
        .then_with(|| a.char_count.cmp(&b.char_count))
        .then_with(|| a.word.cmp(b.word))
}

/// Ba chế độ nguồn tĩnh (trước LOOP-2b là ba pool nhân bản mục — ~2× bộ nhớ và
/// ~2× thời gian dựng lúc khởi động lạnh). Ngữ nghĩa giữ nguyên từng mục:
/// - `EnOnly` (mode EN): mọi mục EN (khử trùng first-writer-wins giữa các nguồn EN).
/// - `VnOnly` (mode VI + cổng RSI-7 chặn EN): mọi mục Việt.
/// - `VnPlusEn` (mode VI cho phép EN): mục Việt + mục EN có từ CHƯA có trong tập Việt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PoolMode {
    EnOnly,
    VnOnly,
    VnPlusEn,
}

impl PoolMode {
    #[inline]
    fn admits(self, e: &StaticEntry) -> bool {
        match self {
            PoolMode::EnOnly => !e.is_vn(),
            PoolMode::VnOnly => e.is_vn(),
            PoolMode::VnPlusEn => e.is_vn() || !e.shadowed_by_vn(),
        }
    }

    /// Thứ tự xếp từ đơn của chế độ (VI: ngắn trước; EN: weight trước).
    fn single_rank(self) -> StaticRankCmp {
        match self {
            PoolMode::EnOnly => rank_weight_first,
            PoolMode::VnOnly | PoolMode::VnPlusEn => rank_single_vi,
        }
    }
}

/// Lexicon tĩnh dùng chung cho cả ba chế độ — dựng MỘT lần (OnceLock).
struct Lexicon {
    /// Từ đơn, sắp theo `(key, word, flags)` để `partition_point` theo tiền tố.
    singles: Vec<StaticEntry>,
    /// Từ ghép (có dấu cách), cùng cách sắp.
    compounds: Vec<StaticEntry>,
    /// Từ ghép nguồn compound nhóm theo âm/từ đầu — chỉ số vào `compounds`, sắp
    /// `rank_weight_first`; lọc theo chế độ lúc tra (thứ tự con giữ nguyên).
    ext_by_first_word: FnvMap<&'static str, Box<[u32]>>,
}

/// 8 byte đầu của khoá dạng big-endian (đệm 0) — giữ đúng thứ tự từ điển của
/// byte UTF-8 (dữ liệu không chứa NUL), so một phép `u64` thay vì so chuỗi.
fn key_prefix8(key: &str) -> u64 {
    let mut buf = [0u8; 8];
    let n = key.len().min(8);
    buf[..n].copy_from_slice(&key.as_bytes()[..n]);
    u64::from_be_bytes(buf)
}

/// Sắp mục theo `(key, word, flags)` — khoá `u64` tiền tố giải quyết gần hết các
/// phép so (LOOP-2b: sort lexicon là phần lớn chi phí dựng lúc khởi động lạnh).
fn sort_by_key_prefix(entries: Vec<StaticEntry>) -> Vec<StaticEntry> {
    let mut keyed: Vec<(u64, StaticEntry)> = entries
        .into_iter()
        .map(|e| (key_prefix8(e.key), e))
        .collect();
    keyed.sort_by(|(pa, a), (pb, b)| {
        pa.cmp(pb)
            .then_with(|| a.key.cmp(b.key))
            .then_with(|| a.word.cmp(b.word))
            .then_with(|| a.flags.cmp(&b.flags))
    });
    // Vec mới đúng `len` — `collect()` tại chỗ sẽ giữ vùng nhớ 48 B/mục của `keyed`.
    let mut sorted = Vec::with_capacity(keyed.len());
    sorted.extend(keyed.into_iter().map(|(_, e)| e));
    sorted
}

fn lexicon() -> &'static Lexicon {
    static CACHE: OnceLock<Lexicon> = OnceLock::new();
    CACHE.get_or_init(|| {
        let vn_syll = vn_syllables_sorted();
        let vn_comp = vn_compounds();
        let tech = tech_terms_lower();
        let en_common = en_common_lines();
        let en_freq = en_freq_sorted();
        let en_10k = en_words10k_sorted();
        let en_comp = en_compounds();

        let mut singles: Vec<StaticEntry> = Vec::with_capacity(
            vn_syll.len() + tech.len() + en_common.len() + en_freq.len() + en_10k.len(),
        );
        let mut compounds: Vec<StaticEntry> = Vec::with_capacity(vn_comp.len() + en_comp.len());
        let mut push = |word: &'static str, key: &'static str, weight: u32, flags: u8| {
            let entry = StaticEntry {
                word,
                key,
                weight,
                char_count: word.chars().count() as u16,
                flags,
            };
            if word.contains(' ') {
                compounds.push(entry);
            } else {
                singles.push(entry);
            }
        };

        // Nguồn Việt: first-writer-wins trong tập Việt.
        let mut vn_seen: FnvSet<&'static str> =
            FnvSet::with_capacity_and_hasher(vn_syll.len() + vn_comp.len(), FnvBuild::default());
        for &syll in vn_syll {
            if vn_seen.insert(syll) {
                push(syll, syll, 90, ENTRY_FROM_VN);
            }
        }
        for &(word, _, weight) in vn_comp {
            if vn_seen.insert(word) {
                push(word, word, weight, ENTRY_FROM_VN | ENTRY_COMPOUND_SRC);
            }
        }

        // Nguồn EN: first-writer-wins trong tập EN; mục trùng từ Việt bị che ở VnPlusEn.
        let mut en_seen: FnvSet<&'static str> = FnvSet::with_capacity_and_hasher(
            tech.len() + en_common.len() + en_freq.len() + en_10k.len() + en_comp.len(),
            FnvBuild::default(),
        );
        let mut push_en = |word: &'static str, key: &'static str, weight: u32, compound: bool| {
            if !en_seen.insert(word) {
                return;
            }
            let mut flags = if compound { ENTRY_COMPOUND_SRC } else { 0 };
            if vn_seen.contains(word) {
                flags |= ENTRY_SHADOWED_BY_VN;
            }
            push(word, key, weight, flags);
        };
        for &(term, tl) in tech {
            push_en(term, tl, 85, false);
        }
        for &word in en_common {
            push_en(word, word, 70, false);
        }
        for &word in en_freq {
            push_en(word, word, 60, false);
        }
        for &word in en_10k {
            push_en(word, word, 55, false);
        }
        for &(word, weight) in en_comp {
            push_en(word, word, weight, true);
        }

        let singles = sort_by_key_prefix(singles);
        let compounds = sort_by_key_prefix(compounds);

        let mut ext_groups: FnvMap<&'static str, Vec<u32>> = FnvMap::default();
        for (i, e) in compounds.iter().enumerate() {
            if e.is_compound_src() {
                if let Some(first) = e.word.split(' ').next() {
                    ext_groups.entry(first).or_default().push(i as u32);
                }
            }
        }
        let ext_by_first_word = ext_groups
            .into_iter()
            .map(|(k, mut v)| {
                v.sort_unstable_by(|&a, &b| {
                    rank_weight_first(&compounds[a as usize], &compounds[b as usize])
                });
                (k, v.into_boxed_slice())
            })
            .collect();

        Lexicon {
            singles,
            compounds,
            ext_by_first_word,
        }
    })
}

/// Chỉ mục top-K dựng sẵn cho tiền tố nặng (2..=3 ký tự, > 24 mục của chế độ) —
/// lưu CHỈ SỐ `u32` vào bảng lexicon (không sao chép mục), xếp theo thứ tự của chế độ.
type HeavyIndex = FnvMap<&'static str, Box<[u32]>>;

struct ModeIndex {
    singles: HeavyIndex,
    compounds: HeavyIndex,
}

fn char_prefix(s: &str, n_chars: usize) -> Option<&str> {
    if n_chars == 0 {
        return Some("");
    }
    let mut indices = s.char_indices();
    for _ in 0..n_chars {
        indices.next()?;
    }
    let end = match indices.next() {
        Some((idx, _)) => idx,
        None => s.len(),
    };
    Some(&s[..end])
}

fn build_heavy_index(
    table: &'static [StaticEntry],
    mode: PoolMode,
    rank_cmp: StaticRankCmp,
) -> HeavyIndex {
    let admitted: Vec<u32> = (0..table.len() as u32)
        .filter(|&i| mode.admits(&table[i as usize]))
        .collect();
    let cmp = |a: &u32, b: &u32| rank_cmp(&table[*a as usize], &table[*b as usize]);
    let mut heavy = HeavyIndex::default();
    for n_chars in 2..=HEAVY_MAX_PREFIX_CHARS {
        let mut i = 0;
        while i < admitted.len() {
            let Some(prefix) = char_prefix(table[admitted[i] as usize].key, n_chars) else {
                i += 1;
                continue;
            };
            let mut j = i + 1;
            while j < admitted.len() && table[admitted[j] as usize].key.starts_with(prefix) {
                j += 1;
            }
            if j - i > HEAVY_PREFIX_THRESHOLD {
                let mut top: Vec<u32> = admitted[i..j].to_vec();
                top.select_nth_unstable_by(HEAVY_TOP_K, cmp);
                top.truncate(HEAVY_TOP_K);
                top.sort_unstable_by(cmp);
                heavy.insert(prefix, top.into_boxed_slice());
            }
            i = j;
        }
    }
    heavy
}

/// Chỉ mục top-K của một chế độ — dựng LƯỜI khi chế độ được dùng lần đầu
/// (người chỉ gõ EN không trả phí dựng chỉ mục VI và ngược lại).
fn mode_index(mode: PoolMode) -> &'static ModeIndex {
    static EN: OnceLock<ModeIndex> = OnceLock::new();
    static VN: OnceLock<ModeIndex> = OnceLock::new();
    static VN_EN: OnceLock<ModeIndex> = OnceLock::new();
    let cell = match mode {
        PoolMode::EnOnly => &EN,
        PoolMode::VnOnly => &VN,
        PoolMode::VnPlusEn => &VN_EN,
    };
    cell.get_or_init(|| {
        let lex = lexicon();
        ModeIndex {
            singles: build_heavy_index(&lex.singles, mode, mode.single_rank()),
            compounds: build_heavy_index(&lex.compounds, mode, rank_weight_first),
        }
    })
}

/// LOOP-2b — số byte heap mà chỉ mục dự đoán tiền tố giữ (lexicon + chỉ mục top-K
/// của các chế độ ĐÃ dựng). Tất định cho cùng dữ liệu nhúng — dùng làm gate bộ nhớ
/// trong test (DLL TSF nạp vào MỌI tiến trình gõ chữ: mỗi MB nhân theo số app).
#[doc(hidden)]
pub fn prefix_index_heap_bytes() -> usize {
    fn map_bytes(m: &FnvMap<&'static str, Box<[u32]>>) -> usize {
        m.capacity() * (std::mem::size_of::<(&str, Box<[u32]>)>() + 1)
            + m.values().map(|v| v.len() * 4).sum::<usize>()
    }
    let lex = lexicon();
    let mut total = (lex.singles.capacity() + lex.compounds.capacity())
        * std::mem::size_of::<StaticEntry>()
        + map_bytes(&lex.ext_by_first_word);
    for mode in [PoolMode::EnOnly, PoolMode::VnOnly, PoolMode::VnPlusEn] {
        let idx = mode_index(mode);
        total += map_bytes(&idx.singles) + map_bytes(&idx.compounds);
    }
    total
}

/// Từ ghép Việt sắp theo `(deaccented ASC, weight DESC, word ASC)` phục vụ nhánh không dấu.
fn vn_compounds_deacc_sorted() -> &'static [UnaccCompoundEntry] {
    static CACHE: OnceLock<Vec<UnaccCompoundEntry>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
        let mut v: Vec<UnaccCompoundEntry> = vn_compounds()
            .iter()
            .filter_map(|&(word, deaccented, weight)| {
                if !seen.insert(word) {
                    return None;
                }
                Some(UnaccCompoundEntry {
                    word,
                    deaccented,
                    char_count: word.chars().count() as u16,
                    weight,
                    is_single: !word.contains(' '),
                })
            })
            .collect();
        v.sort_unstable_by(|a, b| {
            a.deaccented
                .cmp(b.deaccented)
                .then_with(|| b.weight.cmp(&a.weight))
                .then_with(|| a.word.cmp(b.word))
        });
        v
    })
}

fn compare_prefix_candidates(
    a: &PrefixCandidate,
    b: &PrefixCandidate,
    has_ext: bool,
    is_vi: bool,
) -> std::cmp::Ordering {
    let a_single = !a.word.contains(' ');
    let b_single = !b.word.contains(' ');
    b.exact_match
        .cmp(&a.exact_match)
        .then_with(|| b.personal.cmp(&a.personal))
        .then_with(|| {
            if a.personal && b.personal {
                b.weight
                    .cmp(&a.weight)
                    .then_with(|| a.char_count.cmp(&b.char_count))
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .then_with(|| b.ext.cmp(&a.ext))
        .then_with(|| {
            let a_pri = if has_ext { !a_single } else { a_single };
            let b_pri = if has_ext { !b_single } else { b_single };
            b_pri.cmp(&a_pri)
        })
        .then_with(|| {
            if a_single && b_single {
                if !is_vi {
                    b.weight
                        .cmp(&a.weight)
                        .then_with(|| a.char_count.cmp(&b.char_count))
                } else {
                    a.char_count
                        .cmp(&b.char_count)
                        .then_with(|| b.weight.cmp(&a.weight))
                }
            } else {
                b.weight
                    .cmp(&a.weight)
                    .then_with(|| a.char_count.cmp(&b.char_count))
            }
        })
        .then_with(|| a.word.cmp(&b.word))
}

fn is_ext_of(entry: &StaticEntry, variant: &str) -> bool {
    entry.is_compound_src()
        && entry.word.as_bytes().get(variant.len()) == Some(&b' ')
        && entry.word.starts_with(variant)
}

fn predict_impl(
    typed_lower: &str,
    is_vi: bool,
    limit: usize,
    personal: &[(String, u32)],
    style_touches: usize,
    en_context: bool,
) -> Vec<String> {
    if limit == 0 || typed_lower.len() < 2 {
        return Vec::new();
    }
    let is_ascii = typed_lower.is_ascii();
    if !is_ascii && typed_lower.chars().nth(1).is_none() {
        return Vec::new();
    }

    let typed_unaccented: std::borrow::Cow<'_, str> = if is_ascii {
        std::borrow::Cow::Borrowed(typed_lower)
    } else {
        std::borrow::Cow::Owned(
            strip_diacritics(&typed_lower.chars().collect::<Vec<_>>())
                .into_iter()
                .collect::<String>(),
        )
    };
    let typed_has_diacritics = !is_ascii && typed_lower != typed_unaccented.as_ref();

    let (typed_old, typed_new) = if is_ascii {
        (String::new(), String::new())
    } else {
        (
            restyle(typed_lower, DiacriticStyle::Old),
            restyle(typed_lower, DiacriticStyle::New),
        )
    };
    let mut variants: [&str; 3] = [typed_lower, "", ""];
    let mut n_variants: usize = 1;
    if !is_ascii {
        if typed_old != typed_lower {
            variants[n_variants] = typed_old.as_str();
            n_variants += 1;
        }
        if typed_new != typed_lower && typed_new != typed_old {
            variants[n_variants] = typed_new.as_str();
            n_variants += 1;
        }
    }
    let active_variants = &variants[..n_variants];

    let starts = |w: &str| active_variants.iter().any(|v| w.starts_with(v));
    let same = |w: &str| active_variants.contains(&w);
    let compound_ext = |word: &str| word.split(' ').next().is_some_and(same);

    // 0. Ứng viên CÁ NHÂN (khớp `starts(pword) && !same(pword)`).
    let mut personal_hits: Vec<PrefixCandidate> = Vec::new();
    if !personal.is_empty() {
        let recency_boost = |from_end: usize, touches: usize| -> u32 {
            if from_end >= touches {
                0
            } else if from_end < 12 {
                12
            } else if from_end < 24 {
                6
            } else {
                0
            }
        };
        for (i, (pword, pcount)) in personal.iter().enumerate() {
            if !starts(pword) || same(pword) || pword.chars().count() < 2 {
                continue;
            }
            if personal_hits
                .iter()
                .any(|c| c.word.as_ref() == pword.as_str())
            {
                continue;
            }
            personal_hits.push(PrefixCandidate {
                word: std::borrow::Cow::Owned(pword.clone()),
                exact_match: true,
                char_count: pword.chars().count(),
                weight: 100u32
                    .saturating_add((*pcount).min(30))
                    .saturating_add(recency_boost(personal.len() - 1 - i, style_touches)),
                ext: compound_ext(pword),
                personal: true,
            });
        }
    }

    // 1. Chọn chế độ nguồn tĩnh theo ngôn ngữ + cổng RSI-7 (LOOP-2b: một lexicon
    //    dùng chung, chế độ chỉ là bộ lọc `PoolMode::admits`).
    let en_allowed =
        !is_vi || en_context || !vn_prefix_possible_unaccented(typed_unaccented.as_ref());
    let mode = match (is_vi, en_allowed) {
        (false, _) => PoolMode::EnOnly,
        (true, false) => PoolMode::VnOnly,
        (true, true) => PoolMode::VnPlusEn,
    };
    let lex = lexicon();
    let index = mode_index(mode);

    // Tính `ext_slices` và `has_ext` trước khi xuất personal (personal xếp theo `has_ext`).
    // Nhóm ext hợp lệ khi chế độ có ≥ 1 mục trong nhóm và (biến thể là chính chuỗi gõ
    // hoặc nhóm có mục Việt) — đúng ngữ nghĩa pool riêng trước LOOP-2b.
    let mut ext_slices: [Option<&'static [u32]>; 3] = [None; 3];
    let mut static_has_ext = false;
    for (idx, &v) in active_variants.iter().enumerate() {
        if let Some(slice) = lex.ext_by_first_word.get(v) {
            let mut any_admitted = false;
            let mut any_vn = false;
            for &i in slice.iter() {
                let e = &lex.compounds[i as usize];
                if mode.admits(e) {
                    any_admitted = true;
                    if e.is_vn() {
                        any_vn = true;
                        break;
                    }
                }
            }
            let valid = any_admitted && (v == typed_lower || any_vn);
            if valid {
                ext_slices[idx] = Some(&slice[..]);
                static_has_ext = true;
            }
        }
    }
    let has_ext = personal_hits.iter().any(|c| c.ext) || static_has_ext;

    let mut out: Vec<String> = Vec::with_capacity(limit.min(64));
    if !personal_hits.is_empty() {
        personal_hits.sort_by(|a, b| compare_prefix_candidates(a, b, has_ext, is_vi));
        for c in personal_hits.iter().take(limit) {
            out.push(c.word.to_string());
        }
        if out.len() >= limit {
            return out;
        }
    }

    let is_shadowed_by_personal = |word: &str| {
        !personal_hits.is_empty() && personal_hits.iter().any(|p| p.word.as_ref() == word)
    };
    let entry_excluded = |e: &StaticEntry, v: &str, skip_ext: bool| -> bool {
        !mode.admits(e)
            || (v != typed_lower && !e.is_vn())
            || e.key == typed_lower
            || same(e.word)
            || (skip_ext && is_ext_of(e, v))
            || is_shadowed_by_personal(e.word)
    };

    // Ứng viên bảng cho MỘT biến thể: thử top-K dựng sẵn; đủ `need` (hoặc nhóm
    // chỉ có < K mục) thì xong, ngược lại quét dải `partition_point` của bảng.
    let collect_variant = |table: &'static [StaticEntry],
                           heavy: &'static HeavyIndex,
                           v: &str,
                           skip_ext: bool,
                           need: usize,
                           rank_cmp: StaticRankCmp,
                           sink: &mut Vec<&'static StaticEntry>| {
        if let Some(top) = heavy.get(v) {
            let before = sink.len();
            for &i in top.iter() {
                let e = &table[i as usize];
                if !entry_excluded(e, v, skip_ext) {
                    sink.push(e);
                    if sink.len() - before == need {
                        return;
                    }
                }
            }
            if top.len() < HEAVY_TOP_K {
                return;
            }
            // Dự phòng khi `need + excluded > HEAVY_TOP_K`: bỏ phần đã lấy, quét cả dải.
            sink.truncate(before);
        }
        let start = table.partition_point(|e| e.key < v);
        let before = sink.len();
        sink.extend(
            table[start..]
                .iter()
                .take_while(|e| e.key.starts_with(v))
                .filter(|e| !entry_excluded(e, v, skip_ext)),
        );
        let local = &mut sink[before..];
        if local.len() > need {
            local.select_nth_unstable_by(need, |a, b| rank_cmp(a, b));
            sink.truncate(before + need);
        }
    };

    let take_table = |table: &'static [StaticEntry],
                      heavy: &'static HeavyIndex,
                      skip_ext: bool,
                      rank_cmp: StaticRankCmp,
                      out: &mut Vec<String>| {
        let need = limit - out.len();
        if need == 0 {
            return;
        }
        let mut merged: Vec<&'static StaticEntry> = Vec::with_capacity(need.min(64));
        for &v in active_variants {
            collect_variant(table, heavy, v, skip_ext, need, rank_cmp, &mut merged);
        }
        if merged.len() > need {
            merged.select_nth_unstable_by(need, |a, b| rank_cmp(a, b));
            merged.truncate(need);
        }
        merged.sort_unstable_by(|a, b| rank_cmp(a, b));
        for e in merged {
            out.push(e.word.to_string());
        }
    };

    let take_ext = |out: &mut Vec<String>| {
        let need = limit - out.len();
        if need == 0 || !static_has_ext {
            return;
        }
        let mut merged: Vec<&'static StaticEntry> = Vec::with_capacity(need.min(64));
        for (idx, &v) in active_variants.iter().enumerate() {
            if let Some(slice) = ext_slices[idx] {
                let before = merged.len();
                for &i in slice {
                    let e = &lex.compounds[i as usize];
                    if !entry_excluded(e, v, false) {
                        merged.push(e);
                        if merged.len() - before == need {
                            break;
                        }
                    }
                }
            }
        }
        if merged.len() > need {
            merged.select_nth_unstable_by(need, |a, b| rank_weight_first(a, b));
            merged.truncate(need);
        }
        merged.sort_unstable_by(|a, b| rank_weight_first(a, b));
        for e in merged {
            out.push(e.word.to_string());
        }
    };

    let single_rank_cmp = mode.single_rank();

    if has_ext {
        take_ext(&mut out);
        if out.len() < limit {
            take_table(
                &lex.compounds,
                &index.compounds,
                true,
                rank_weight_first,
                &mut out,
            );
        }
        if out.len() < limit {
            take_table(
                &lex.singles,
                &index.singles,
                false,
                single_rank_cmp,
                &mut out,
            );
        }
    } else {
        take_table(
            &lex.singles,
            &index.singles,
            false,
            single_rank_cmp,
            &mut out,
        );
        if out.len() < limit {
            take_table(
                &lex.compounds,
                &index.compounds,
                false,
                rank_weight_first,
                &mut out,
            );
        }
    }

    // 3. Nhánh không dấu của `vn_compounds` (`exact_match = false`) chỉ khi còn slot.
    if is_vi && !typed_has_diacritics && out.len() < limit && !typed_unaccented.is_empty() {
        let need = limit - out.len();
        let table = vn_compounds_deacc_sorted();
        let start = table.partition_point(|e| e.deaccented < typed_unaccented.as_ref());
        let mut unacc: Vec<&UnaccCompoundEntry> = table[start..]
            .iter()
            .take_while(|e| e.deaccented.starts_with(typed_unaccented.as_ref()))
            .filter(|e| !starts(e.word) && !same(e.word) && !is_shadowed_by_personal(e.word))
            .collect();
        let cmp_unacc = |a: &&UnaccCompoundEntry, b: &&UnaccCompoundEntry| {
            let a_pri = if has_ext { !a.is_single } else { a.is_single };
            let b_pri = if has_ext { !b.is_single } else { b.is_single };
            b_pri
                .cmp(&a_pri)
                .then_with(|| {
                    if a.is_single && b.is_single {
                        a.char_count
                            .cmp(&b.char_count)
                            .then_with(|| b.weight.cmp(&a.weight))
                    } else {
                        b.weight
                            .cmp(&a.weight)
                            .then_with(|| a.char_count.cmp(&b.char_count))
                    }
                })
                .then_with(|| a.word.cmp(b.word))
        };
        if unacc.len() > need {
            unacc.select_nth_unstable_by(need, cmp_unacc);
            unacc.truncate(need);
        }
        unacc.sort_unstable_by(cmp_unacc);
        for e in unacc {
            out.push(e.word.to_string());
        }
    }

    out
}

/// Dự đoán cụm từ tiếp theo theo ngữ cảnh (Next-Word / Phrase Prediction).
///
/// `context_tail`: Đuôi văn bản gần nhất trước con trỏ (đã trim khoảng trắng đuôi).
/// Trả về danh sách ứng viên cụm từ tiếp theo (tối đa `limit`), xếp theo độ dài khóa ngữ cảnh
/// giảm dần (trigram trước bigram) và trọng số `weight`.
pub fn predict_next_phrase_candidates(context_tail: &str, limit: usize) -> Vec<String> {
    predict_next_phrase_matches_with_personal(context_tail, limit, &[])
        .into_iter()
        .map(|m| m.prediction.into_owned())
        .collect()
}

/// Một ứng viên cụm từ tiếp theo kèm **số từ của khoá ngữ cảnh** đã khớp.
///
/// `key_words >= 2` = ngữ cảnh nhiều từ ("phát triển", "kính gửi") — tín hiệu
/// mạnh hơn hoàn tất tiền tố của riêng âm tiết cuối (engine dùng để quyết định
/// thứ tự Tab: `phát triển`+Tab phải ra "phần mềm", không phải "triển khai").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseMatch {
    pub prediction: std::borrow::Cow<'static, str>,
    pub key_words: usize,
}

/// Như [`predict_next_phrase_candidates`] nhưng trả kèm số từ của khoá khớp.
///
/// Ngữ cảnh kết thúc bằng **xuống dòng** (`"Kính gửi\n"`) là dòng mới — không
/// dự đoán nối tiếp dòng trên (trước đây chèn `" quý khách hàng"` đầu dòng mới).
pub fn predict_next_phrase_matches(context_tail: &str, limit: usize) -> Vec<PhraseMatch> {
    predict_next_phrase_matches_with_personal(context_tail, limit, &[])
}

/// Mở rộng [`predict_next_phrase_matches`] với dữ liệu từ vựng cá nhân học được (P7/Dream RSI).
pub fn predict_next_phrase_matches_with_personal(
    context_tail: &str,
    limit: usize,
    personal: &[(String, u32)],
) -> Vec<PhraseMatch> {
    let trimmed_end = context_tail.trim_end_matches([' ', '\t']);
    if trimmed_end.ends_with(['\n', '\r']) {
        return Vec::new();
    }
    let trimmed = trimmed_end.trim_start();
    if trimmed.is_empty() || limit == 0 {
        return Vec::new();
    }
    // Khoá dữ liệu viết kiểu dấu cũ — "cộng hoà" (kiểu mới) vẫn khớp "cộng hòa".
    let context_lower = restyle(&trimmed.to_lowercase(), DiacriticStyle::Old);

    // (độ dài khoá, trọng số, dự đoán, số từ của khoá)
    let mut matches: Vec<(usize, u32, std::borrow::Cow<'static, str>, usize)> = Vec::new();

    // 1. Khớp cụm từ CÁ NHÂN (P7 / Dream RSI evolution):
    // Cụm từ cá nhân (2-4 từ) khớp tiền tố đuôi ngữ cảnh -> dự đoán phần còn lại với boost weight >= 100
    // R2-150 (perf): chạy mỗi phím trên cả hồ sơ — loại nhanh bằng từ cuối của khoá trước khi
    // ghép/chuẩn hoá. Đổi kiểu dấu giữ số chữ và phụ âm đầu nên phép loại không bỏ sót khớp thật.
    let last_run = |s: &str| {
        let run = s.rsplit(|c: char| !c.is_alphabetic()).next().unwrap_or("");
        (run.chars().count(), run.chars().next())
    };
    let (ctx_len, ctx_first) = last_run(&context_lower);
    let may_end_context = |word: &str| {
        let (len, first) = last_run(word);
        let first_differs = match (first, ctx_first) {
            (Some(a), Some(b)) => a.is_ascii() && b.is_ascii() && !a.eq_ignore_ascii_case(&b),
            _ => false,
        };
        len == ctx_len && !first_differs
    };
    for (pentry, pcount) in personal {
        if !pentry.contains(' ') {
            continue;
        }
        let mut words: [&str; 4] = [""; 4];
        let mut nwords = 0usize;
        for w in pentry.split_whitespace() {
            if nwords >= 4 {
                nwords = 5;
                break;
            }
            words[nwords] = w;
            nwords += 1;
        }
        if (2..=4).contains(&nwords) {
            let words = &words[..nwords];
            for k in (1..words.len()).rev() {
                if !may_end_context(words[k - 1]) {
                    continue;
                }
                let key = words[..k].join(" ");
                let key_norm = restyle(&key.to_lowercase(), DiacriticStyle::Old);
                if context_lower.ends_with(&key_norm) {
                    let prefix_len = context_lower.len() - key_norm.len();
                    let matched_boundary = prefix_len == 0
                        || matches!(
                            context_lower.as_bytes()[prefix_len - 1],
                            b' ' | b'.' | b',' | b'!' | b'?' | b'\n' | b'\t' | b'(' | b'"' | b':'
                        );
                    if matched_boundary {
                        let cont = words[k..].join(" ");
                        let weight = 100u32.saturating_add((*pcount).min(30));
                        matches.push((key_norm.len(), weight, std::borrow::Cow::Owned(cont), k));
                        break;
                    }
                }
            }
        }
    }

    // 2. Khớp từ bảng tĩnh (VN / EN / văn phòng EN — vòng 59)
    // Vòng 52 (perf): bucket theo byte cuối — chỉ quét các hàng có khả năng
    // khớp (`ends_with` bắt buộc byte cuối trùng), thay vì quét toàn bảng.
    // Vòng 59: bảng văn phòng EN quét song song hai bảng cũ — context là chuỗi
    // thuần nên bảng nào cũng khớp được ở cả hai mode (bilingual code-switch
    // miễn phí: key trộn Việt-Eng sống trong bảng văn phòng).
    for (table, rows) in [
        (PhraseTable::Vn, vn_phrase_rows()),
        (PhraseTable::En, en_phrase_rows()),
        (PhraseTable::Office, office_phrase_rows()),
    ] {
        let buckets = phrase_buckets(rows, table);
        if let Some(&last) = context_lower.as_bytes().last() {
            if let Some(bucket) = buckets.get(&last) {
                collect_phrase_matches(bucket, &context_lower, &mut matches);
            }
        }
    }

    // Xếp theo: key_len lớn hơn (trigram > bigram) -> weight cao hơn
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    // Vòng 53: guard limit độc hại (usize::MAX sẽ abort khi cấp phát —
    // call site cứng 5 nhưng đề phòng API công khai).
    let mut out: Vec<PhraseMatch> = Vec::with_capacity(limit.min(64));
    if limit <= 32 {
        for (_, _, pred, key_words) in matches {
            if !out.iter().any(|m| m.prediction.as_ref() == pred.as_ref()) {
                out.push(PhraseMatch {
                    prediction: pred,
                    key_words,
                });
                if out.len() >= limit {
                    break;
                }
            }
        }
    } else {
        let mut seen = std::collections::HashSet::new();
        for (_, _, pred, key_words) in matches {
            if seen.insert(pred.to_string()) {
                out.push(PhraseMatch {
                    prediction: pred,
                    key_words,
                });
                if out.len() >= limit {
                    break;
                }
            }
        }
    }
    out
}

/// So khoá khớp đuôi ngữ cảnh **trọn từ** (trước khoá là đầu chuỗi / khoảng
/// trắng / dấu câu). Vòng 41 (perf): nhận hàng ĐÃ parse từ OnceLock —
/// split/parse TSV chạy một lần lúc khởi động, phần per-call chỉ còn
/// `ends_with` + kiểm ranh giới từ. Vòng 52: gọi trên **bucket** byte-cuối
/// (xem [`phrase_buckets`]) nên số hàng quét giảm ~20-30×.
fn collect_phrase_matches(
    rows: &[PhraseRow],
    context_lower: &str,
    out: &mut Vec<(usize, u32, std::borrow::Cow<'static, str>, usize)>,
) {
    for &(key, pred, weight) in rows {
        if !context_lower.ends_with(key) {
            continue;
        }
        // Ranh giới từ ở đầu khoá (byte trước khoá; ký tự nhiều byte có byte
        // cuối là continuation byte nên không bao giờ trùng các byte ASCII này).
        let prefix_len = context_lower.len() - key.len();
        let matched_boundary = prefix_len == 0
            || matches!(
                context_lower.as_bytes()[prefix_len - 1],
                b' ' | b'.' | b',' | b'!' | b'?' | b'\n' | b'\t' | b'(' | b'"' | b':'
            );
        if matched_boundary {
            out.push((
                key.len(),
                weight,
                std::borrow::Cow::Borrowed(pred),
                key.split_whitespace().count(),
            ));
        }
    }
}

/// Chữ hoa/thường của **cụm nối tiếp** (next-phrase) theo từ đứng trước.
///
/// Khác [`adapt_capitalization`] (hoàn tất chính từ đang gõ): cụm nối tiếp là
/// từ **khác** — từ trước viết hoa chữ đầu (đầu câu "Xin", "Best", tiêu đề)
/// KHÔNG được làm cụm sau viết hoa ("Xin Chào", "Best Regards" sai chính tả).
/// Chỉ từ trước viết **HOA TOÀN BỘ** (≥ 2 chữ cái — "CỘNG HÒA", không phải
/// đại từ "I") mới kéo cụm sau thành chữ hoa (quốc hiệu, tiêu đề văn bản).
/// Còn lại giữ nguyên chữ hoa/thường chuẩn trong dữ liệu ("Việt Nam").
pub fn adapt_continuation_case(prev_word: &str, text: &str) -> String {
    if is_all_caps_word(prev_word) {
        text.to_uppercase()
    } else {
        text.to_string()
    }
}

/// Từ viết hoa toàn bộ, có ít nhất 2 chữ cái ("HD", "PHÁT" — không phải "I", "A").
fn is_all_caps_word(word: &str) -> bool {
    let mut letters = 0usize;
    for c in word.chars().filter(|c| c.is_alphabetic()) {
        if !c.is_uppercase() {
            return false;
        }
        letters += 1;
    }
    letters >= 2
}

/// Khôi phục chữ hoa/thường theo mẫu gõ gốc.
///
/// - Nếu mẫu gốc viết hoa chữ cái đầu (`"Phát"`, `"Thank"`) → viết hoa chữ cái đầu kết quả (`"Phát triển"`, `"Thank you"`).
/// - Nếu mẫu gốc viết hoa toàn bộ (`"PHÁT"`, `"THANK"`; ≥ 2 chữ cái) → viết hoa toàn bộ kết quả.
/// - Ngược lại giữ nguyên chữ hoa/thường chuẩn của ứng viên.
pub fn adapt_capitalization(sample: &str, text: &str) -> String {
    if sample.is_empty() || text.is_empty() {
        return text.to_string();
    }
    let first_sample = sample.chars().next().unwrap();

    if is_all_caps_word(sample) {
        text.to_uppercase()
    } else if first_sample.is_uppercase() {
        let mut chars = text.chars();
        if let Some(first) = chars.next() {
            let mut out = first.to_uppercase().collect::<String>();
            out.extend(chars);
            out
        } else {
            text.to_string()
        }
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_prefix_compounds_vietnamese() {
        let candidates = predict_prefix_candidates("phát", true, 3);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0], "phát triển");

        // Gõ không dấu vẫn tìm được từ ghép
        let candidates_unaccented = predict_prefix_candidates("phat", true, 3);
        assert!(!candidates_unaccented.is_empty());
        assert_eq!(candidates_unaccented[0], "phát triển");

        let c_cong = predict_prefix_candidates("công", true, 3);
        assert!(c_cong.contains(&"công nghệ".to_string()));
    }

    #[test]
    fn test_predict_prefix_english() {
        let candidates = predict_prefix_candidates("techn", false, 3);
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.starts_with("techn")));
        // Vòng 52: nguồn 10k — "atmos" hoàn tất từ google-10000
        let c10k = predict_prefix_candidates("atmos", false, 5);
        assert!(c10k.iter().any(|c| c.starts_with("atmos")));
    }

    #[test]
    fn test_predict_next_phrase_vietnamese() {
        let preds = predict_next_phrase_candidates("cộng hòa", 3);
        assert!(!preds.is_empty());
        assert_eq!(preds[0], "xã hội chủ nghĩa Việt Nam");
        // Kiểu dấu mới ("hoà") khớp khoá kiểu cũ ("hòa").
        assert_eq!(
            predict_next_phrase_candidates("Cộng hoà", 1),
            vec!["xã hội chủ nghĩa Việt Nam".to_string()]
        );
        assert_eq!(
            restyle("hòa bình thủy", DiacriticStyle::New),
            "hoà bình thuỷ"
        );
        assert_eq!(restyle("hoà, thuỷ", DiacriticStyle::Old), "hòa, thủy");

        let preds_kinh = predict_next_phrase_candidates("kính gửi", 3);
        assert!(!preds_kinh.is_empty());
        assert_eq!(preds_kinh[0], "quý khách hàng");

        let preds_tt = predict_next_phrase_candidates("trân trọng", 3);
        assert!(!preds_tt.is_empty());
        assert_eq!(preds_tt[0], "cảm ơn");
    }

    /// Vòng 52: thành ngữ — gõ nửa đầu gợi ý nửa sau.
    #[test]
    fn thanh_ngu_du_doan_nua_sau() {
        let preds = predict_next_phrase_candidates("có công mài sắt", 5);
        assert!(
            preds.iter().any(|p| p.contains("nên kim")),
            "có công mài sắt => {preds:?}"
        );
        let preds2 = predict_next_phrase_candidates("ăn quả", 5);
        assert!(preds2.iter().any(|p| p.contains("trồng cây")));
        // Câu thật chia ranh giới: ngữ cảnh dài khớp chính xác phần còn lại.
        let m = predict_next_phrase_matches("anh chị vui lòng kiểm tra lại", 5);
        assert!(
            m.iter().any(|p| p.prediction.contains("thông tin")),
            "splits => {:?}",
            m
        );
    }

    #[test]
    fn test_predict_next_phrase_english() {
        let preds = predict_next_phrase_candidates("thank you", 3);
        assert!(!preds.is_empty());
        assert_eq!(preds[0], "very much");

        let preds_look = predict_next_phrase_candidates("looking forward to", 3);
        assert!(!preds_look.is_empty());
        assert_eq!(preds_look[0], "hearing from you");

        // Idiom EN (vòng 52)
        let preds_idiom = predict_next_phrase_candidates("kill two birds", 5);
        assert!(preds_idiom.iter().any(|p| p.contains("one stone")));
    }

    #[test]
    fn test_adapt_capitalization() {
        assert_eq!(adapt_capitalization("phát", "phát triển"), "phát triển");
        assert_eq!(adapt_capitalization("Phát", "phát triển"), "Phát triển");
        assert_eq!(adapt_capitalization("PHÁT", "phát triển"), "PHÁT TRIỂN");
        assert_eq!(adapt_capitalization("Thank", "very much"), "Very much");
    }

    /// R2-120: cụm nối tiếp không thừa kế "viết hoa chữ đầu" của từ trước
    /// ("Xin" đầu câu → "Xin chào", không "Xin Chào"); chỉ HOA TOÀN BỘ (≥2
    /// chữ cái) mới kéo theo; chữ hoa chuẩn trong dữ liệu ("Việt Nam") giữ.
    #[test]
    fn cum_noi_tiep_giu_chu_hoa_chuan() {
        assert_eq!(adapt_continuation_case("Xin", "chào"), "chào");
        assert_eq!(adapt_continuation_case("Best", "regards"), "regards");
        assert_eq!(adapt_continuation_case("XIN", "chào"), "CHÀO");
        assert_eq!(adapt_continuation_case("I", "hope"), "hope");
        assert_eq!(
            adapt_continuation_case("hòa", "xã hội chủ nghĩa Việt Nam"),
            "xã hội chủ nghĩa Việt Nam"
        );
        assert_eq!(adapt_capitalization("I", "it"), "It");
    }

    /// R2-120: xuống dòng = ngữ cảnh mới — không dự đoán nối dòng trên.
    #[test]
    fn khong_du_doan_qua_dong_moi() {
        assert!(predict_next_phrase_candidates("kính gửi\n", 3).is_empty());
        assert!(predict_next_phrase_candidates("kính gửi\r\n  ", 3).is_empty());
        assert!(!predict_next_phrase_candidates("kính gửi  ", 3).is_empty());
        let m = predict_next_phrase_matches("ông phát triển", 3);
        assert_eq!(m[0].key_words, 2);
        assert_eq!(predict_next_phrase_matches("xin", 1)[0].key_words, 1);
        // Ranh giới trọn từ: "toàn" không khớp khoá "an toàn" trong "bàn toàn".
        assert!(predict_next_phrase_candidates("ban toàn", 3).is_empty());
    }

    /// Vòng 52 (data invariant): mọi hàng TSV đều parse được, prediction ≤ 60
    /// ký tự (R1-2), không dòng rác. Vòng 59: bảng văn phòng ≥ 10.000 + khoá
    /// ≤ 60 (F-A2-2 audit vòng 59 — docstring hứa mà thiếu assert; bảng vn/en
    /// cũ còn hàng legacy khoá dài hơn — F-A2-3, dọn trong vòng data kế tiếp
    /// cùng đồng bộ số README). F-A2-1: không `\r` (script gen phải ghi LF).
    #[test]
    fn du_lieu_phrases_invariant() {
        for (name, data, min_rows) in [
            ("vn", VN_PHRASES_DATA, 2000),
            ("en", EN_PHRASES_DATA, 2000),
            ("office", EN_OFFICE_PHRASES_DATA, 10_000),
        ] {
            assert!(
                !data.contains('\r'),
                "{name}: dữ liệu phải LF, thấy CRLF (F-A2-1)"
            );
            let rows = parse_phrase_rows(data);
            assert!(
                rows.len() >= min_rows,
                "{name}: quá ít hàng: {}",
                rows.len()
            );
            let mut uniq: Vec<&PhraseRow> = rows.iter().collect();
            uniq.sort();
            uniq.dedup();
            assert_eq!(uniq.len(), rows.len(), "{name}: hàng trùng");
            for (key, pred, weight) in rows {
                assert!(!key.is_empty() && !pred.is_empty(), "{name}: hàng rỗng");
                assert!(
                    pred.chars().count() <= 60,
                    "{name}: prediction > 60 ký tự: {pred}"
                );
                if name == "office" {
                    assert!(key.chars().count() <= 60, "office: khoá > 60 ký tự: {key}");
                }
                assert!(weight > 0 && weight <= 100, "{name}: weight {weight}");
                assert!(
                    !key.chars().any(|c| c.is_uppercase()),
                    "{name}: khoá phải lowercase: {key}"
                );
            }
        }
    }

    /// Vòng 59 (office): 5 nhóm nghiệp vụ đều phủ + bilingual code-switch bắn.
    #[test]
    fn bang_van_phong_phu_5_nhom_va_bilingual() {
        // Upward reporting — exec summary / escalation / risk
        assert!(
            predict_next_phrase_candidates("one key risk to highlight is", 3)
                .first()
                .is_some_and(|p| p.contains("the "))
        );
        assert!(!predict_next_phrase_candidates("here is a quick status update on", 3).is_empty());
        assert!(
            !predict_next_phrase_candidates("please find the executive summary below", 3)
                .is_empty()
        );
        assert!(
            !predict_next_phrase_candidates("escalating this blocker for your visibility", 3)
                .is_empty()
        );
        // Downward management — delegation / unblock / recognition
        assert!(!predict_next_phrase_candidates("could you please prioritize", 3).is_empty());
        assert!(
            !predict_next_phrase_candidates("what support do you need to unblock", 3).is_empty()
        );
        assert!(!predict_next_phrase_candidates("great job on delivering", 3).is_empty());
        // Tech — RCA / CI / deploy
        assert!(
            !predict_next_phrase_candidates("the fix has been verified on ci and", 3).is_empty()
        );
        assert!(
            !predict_next_phrase_candidates("deploying the hotfix to production", 3).is_empty()
        );
        // Business ops — action items / stakeholder / follow-up
        assert!(
            !predict_next_phrase_candidates("we need stakeholder sign off before", 3).is_empty()
        );
        assert!(
            !predict_next_phrase_candidates("following up on our action items from", 3).is_empty()
        );
        // Everyday glue
        assert!(!predict_next_phrase_candidates("thanks for flagging this", 3).is_empty());
        assert!(!predict_next_phrase_candidates("please treat this as", 3).is_empty());
        // Bilingual Việt-Eng code-switch — key trộn, prediction tiếng Anh giữa câu Việt.
        let sw = predict_next_phrase_candidates("mình sync lại", 3);
        assert!(
            sw.first().is_some_and(|p| p.contains("status")),
            "bilingual: mình sync lại => {sw:?}"
        );
        assert!(predict_next_phrase_candidates("anh check giúp em", 3)
            .first()
            .is_some_and(|p| p.starts_with("lỗi")));
        assert!(predict_next_phrase_candidates("mình sẽ update", 3)
            .first()
            .is_some_and(|p| p.contains("timeline")));
        assert!(predict_next_phrase_candidates("fix đã được", 3)
            .first()
            .is_some_and(|p| p.contains("verify")));
        // Ranh giới trọn từ: không khớp giữa từ ("lại" ≠ khoá "mình sync lại").
        assert!(predict_next_phrase_candidates("sync", 3).is_empty());
    }

    /// Bản tham chiếu brute-force gốc trước LOOP-2 — CHỈ dùng trong `#[cfg(test)]`
    /// để chứng minh chỉ mục tiền tố top-K cho kết quả giống hệt 100% mọi nhánh.
    fn predict_impl_reference(
        typed_lower: &str,
        is_vi: bool,
        limit: usize,
        personal: &[(String, u32)],
        style_touches: usize,
        en_context: bool,
    ) -> Vec<String> {
        if typed_lower.chars().count() < 2 || limit == 0 {
            return Vec::new();
        }
        let mut ranked: Vec<PrefixCandidate> = Vec::new();
        let mut seen: std::collections::HashSet<std::borrow::Cow<'static, str>> =
            std::collections::HashSet::new();
        let typed_unaccented: String = strip_diacritics(&typed_lower.chars().collect::<Vec<_>>())
            .into_iter()
            .collect();
        let typed_has_diacritics = typed_lower != typed_unaccented;
        let typed_old = restyle(typed_lower, DiacriticStyle::Old);
        let typed_new = restyle(typed_lower, DiacriticStyle::New);
        let starts = |w: &str| {
            w.starts_with(typed_lower)
                || w.starts_with(typed_old.as_str())
                || w.starts_with(typed_new.as_str())
        };
        let same = |w: &str| w == typed_lower || w == typed_old || w == typed_new;
        let compound_ext = |word: &str| word.split(' ').next().is_some_and(same);

        let recency_boost = |from_end: usize, touches: usize| -> u32 {
            if from_end >= touches {
                0
            } else if from_end < 12 {
                12
            } else if from_end < 24 {
                6
            } else {
                0
            }
        };
        for (i, (pword, pcount)) in personal.iter().enumerate() {
            if pword.chars().count() < 2 || same(pword) {
                continue;
            }
            if starts(pword) {
                let word: std::borrow::Cow<str> = std::borrow::Cow::Owned(pword.clone());
                if seen.insert(word.clone()) {
                    ranked.push(PrefixCandidate {
                        word,
                        exact_match: true,
                        char_count: pword.chars().count(),
                        weight: 100u32
                            .saturating_add((*pcount).min(30))
                            .saturating_add(recency_boost(personal.len() - 1 - i, style_touches)),
                        ext: compound_ext(pword),
                        personal: true,
                    });
                }
            }
        }

        if is_vi {
            let push_syll = |syll: &'static str,
                             ranked: &mut Vec<PrefixCandidate>,
                             seen: &mut std::collections::HashSet<
                std::borrow::Cow<'static, str>,
            >| {
                if same(syll) || !seen.insert(std::borrow::Cow::Borrowed(syll)) {
                    return;
                }
                ranked.push(PrefixCandidate {
                    word: std::borrow::Cow::Borrowed(syll),
                    exact_match: true,
                    char_count: syll.chars().count(),
                    weight: 90,
                    ext: false,
                    personal: false,
                });
            };
            for variant in [typed_lower, typed_old.as_str(), typed_new.as_str()] {
                let sorted = vn_syllables_sorted();
                let start = sorted.partition_point(|w| *w < variant);
                for &syll in &sorted[start..] {
                    if !syll.starts_with(variant) {
                        break;
                    }
                    if starts(syll) {
                        push_syll(syll, &mut ranked, &mut seen);
                    }
                }
            }

            for &(word, word_unaccented, weight) in vn_compounds() {
                if same(word) {
                    continue;
                }
                let matches_accented = starts(word);
                let matches_unaccented =
                    if !matches_accented && !typed_has_diacritics && !typed_unaccented.is_empty() {
                        word_unaccented.starts_with(&typed_unaccented)
                    } else {
                        false
                    };
                if (matches_accented || matches_unaccented)
                    && seen.insert(std::borrow::Cow::Borrowed(word))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(word),
                        exact_match: matches_accented,
                        char_count: word.chars().count(),
                        weight,
                        ext: compound_ext(word),
                        personal: false,
                    });
                }
            }
        }

        let en_allowed = !is_vi || en_context || !vn_prefix_possible_unaccented(&typed_unaccented);
        if en_allowed {
            for &(term, tl) in tech_terms_lower() {
                if tl.starts_with(typed_lower)
                    && tl != typed_lower
                    && seen.insert(std::borrow::Cow::Borrowed(term))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(term),
                        exact_match: true,
                        char_count: term.chars().count(),
                        weight: 85,
                        ext: false,
                        personal: false,
                    });
                }
            }
            for &word in en_common_lines() {
                if word.starts_with(typed_lower)
                    && word != typed_lower
                    && seen.insert(std::borrow::Cow::Borrowed(word))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(word),
                        exact_match: true,
                        char_count: word.chars().count(),
                        weight: 70,
                        ext: false,
                        personal: false,
                    });
                }
            }
            for &word in en_freq_sorted() {
                if word.starts_with(typed_lower)
                    && word != typed_lower
                    && seen.insert(std::borrow::Cow::Borrowed(word))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(word),
                        exact_match: true,
                        char_count: word.chars().count(),
                        weight: 60,
                        ext: false,
                        personal: false,
                    });
                }
            }
            for &word in en_words10k_sorted() {
                if word.starts_with(typed_lower)
                    && word != typed_lower
                    && seen.insert(std::borrow::Cow::Borrowed(word))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(word),
                        exact_match: true,
                        char_count: word.chars().count(),
                        weight: 55,
                        ext: false,
                        personal: false,
                    });
                }
            }
            for &(word, weight) in en_compounds() {
                if word != typed_lower
                    && word.starts_with(typed_lower)
                    && seen.insert(std::borrow::Cow::Borrowed(word))
                {
                    ranked.push(PrefixCandidate {
                        word: std::borrow::Cow::Borrowed(word),
                        exact_match: true,
                        char_count: word.chars().count(),
                        weight,
                        ext: compound_ext(word),
                        personal: false,
                    });
                }
            }
        }

        let has_ext = ranked.iter().any(|c| c.ext);
        ranked.sort_by(|a, b| compare_prefix_candidates(a, b, has_ext, is_vi));
        ranked
            .into_iter()
            .take(limit)
            .map(|c| c.word.to_string())
            .collect()
    }

    #[test]
    fn loop2_prefix_index_exact_parity() {
        let prefixes = [
            "a", "t", "đ", "co", "ph", "ng", "th", "tr", "gu", "ho", "ba", "re", "in", "pr", "st",
            "de", "cô", "đư", "qu", "ca", "ti", "nh", "ch", "kh", "gi", "vi", "ma", "en", "ap",
            "un", "con", "the", "phá", "pha", "côn", "ngư", "ngu", "hoà", "hòa", "thu", "api",
            "git", "doc", "mac", "pro", "com", "tra", "trâ", "khô", "đượ", "tiế", "vie", "xay",
            "ban", "phát", "phat", "công", "cong", "thủy", "thuỷ", "hoà ", "techn", "atmos",
            "docker", "machin", "machine", "javasc", "typesc", "người", "triển", "không", "khong",
            "chúng", "trân", "hòa b", "tiến", "tổng", "nhân", "phần", "quản", "khách", "cộng",
            "hệ th", "zzzz", "xqyz", "đđđ",
        ];
        for &p in &prefixes {
            for is_vi in [true, false] {
                for en_ctx in [true, false] {
                    for limit in [0, 1, 2, 3, 5, 10, 30] {
                        let got = predict_prefix_candidates_with_personal(
                            p,
                            is_vi,
                            limit,
                            &[],
                            0,
                            en_ctx,
                        );
                        let exp = predict_impl_reference(p, is_vi, limit, &[], 0, en_ctx);
                        assert_eq!(
                            got, exp,
                            "parity mismatch: prefix={p:?}, is_vi={is_vi}, en_ctx={en_ctx}, limit={limit}"
                        );
                    }
                }
            }
        }

        // Personal overlays: trùng tĩnh, đảo `has_ext`, recency touches, và > HEAVY_TOP_K fallback
        let personal_overlap = vec![
            ("con người".to_string(), 5),
            ("control".to_string(), 20),
            ("phát triển".to_string(), 10),
            ("techn stack".to_string(), 8),
            ("hòa hợp".to_string(), 4),
            ("hoà bình".to_string(), 7),
        ];
        for &p in &["con", "phát", "phat", "techn", "hòa", "hoà", "trân"] {
            for is_vi in [true, false] {
                for en_ctx in [true, false] {
                    for touches in [0, 2, 6] {
                        for limit in [1, 3, 5, 30] {
                            let got = predict_prefix_candidates_with_personal(
                                p,
                                is_vi,
                                limit,
                                &personal_overlap,
                                touches,
                                en_ctx,
                            );
                            let exp = predict_impl_reference(
                                p,
                                is_vi,
                                limit,
                                &personal_overlap,
                                touches,
                                en_ctx,
                            );
                            assert_eq!(
                                got, exp,
                                "personal parity mismatch: prefix={p:?}, is_vi={is_vi}, en_ctx={en_ctx}, touches={touches}, limit={limit}"
                            );
                        }
                    }
                }
            }
        }

        // Hồ sơ lớn (30 mục bắt đầu bằng "con") để ép đường fallback > HEAVY_TOP_K (24)
        let mut heavy_personal: Vec<(String, u32)> = (0..30)
            .map(|i| (format!("concustom{i:02}"), (i % 25 + 1) as u32))
            .collect();
        heavy_personal.push(("con người".to_string(), 15));
        for touches in [0, 12, 24, 31] {
            for limit in [5, 25, 40] {
                let got = predict_prefix_candidates_with_personal(
                    "con",
                    true,
                    limit,
                    &heavy_personal,
                    touches,
                    true,
                );
                let exp =
                    predict_impl_reference("con", true, limit, &heavy_personal, touches, true);
                assert_eq!(
                    got, exp,
                    "heavy personal fallback touches={touches}, limit={limit}"
                );
            }
        }
    }

    #[test]
    fn loop2_next_phrase_personal_stack_words() {
        let personal = vec![
            ("trân trọng kính chào".to_string(), 10),
            ("xin cảm ơn nhiều".to_string(), 5),
            ("một hai ba bốn năm".to_string(), 99), // 5 từ: phải bị bỏ qua
        ];
        let m1 = predict_next_phrase_matches_with_personal("trân trọng", 5, &personal);
        assert_eq!(m1[0].prediction, "kính chào");
        assert_eq!(m1[0].key_words, 2);

        let m2 = predict_next_phrase_matches_with_personal("xin cảm ơn", 5, &personal);
        assert_eq!(m2[0].prediction, "nhiều");
        assert_eq!(m2[0].key_words, 3);

        let m5 = predict_next_phrase_matches_with_personal("một hai ba bốn", 5, &personal);
        assert!(!m5.iter().any(|m| m.prediction == "năm"));
    }

    /// LOOP-2b — parity quét rộng: MỌI tiền tố 2 ký tự có trong lexicon + 1/7 tiền tố
    /// 3 ký tự, đủ 3 chế độ nguồn (EN / VN / VN+EN qua `is_vi` × `en_ctx`), so bản
    /// tham chiếu brute-force trước LOOP-2. Bắt lỗi lọc chế độ (`PoolMode::admits`),
    /// che mục EN trùng Việt (`shadowed_by_vn`) và top-K theo chế độ.
    #[test]
    fn loop2b_lexicon_mode_parity_sweep() {
        let lex = lexicon();
        let mut prefixes: Vec<&'static str> = Vec::new();
        for table in [&lex.singles, &lex.compounds] {
            for n_chars in [2usize, 3] {
                let mut last: Option<&'static str> = None;
                let mut k = 0usize;
                for e in table.iter() {
                    if let Some(p) = char_prefix(e.key, n_chars) {
                        if last != Some(p) {
                            last = Some(p);
                            k += 1;
                            // `k % 7` thay `is_multiple_of` (Rust 1.87): ci-deb/ci-aur dùng rustc 1.85.
                            #[allow(clippy::manual_is_multiple_of)]
                            let sampled = k % 7 == 0;
                            if n_chars == 2 || sampled {
                                prefixes.push(p);
                            }
                        }
                    }
                }
            }
        }
        prefixes.sort_unstable();
        prefixes.dedup();
        assert!(
            prefixes.len() > 300,
            "quét quá ít tiền tố: {}",
            prefixes.len()
        );
        let mut modes_seen = [false; 3];
        for &p in &prefixes {
            for (is_vi, en_ctx) in [(false, false), (true, true), (true, false)] {
                if is_vi && !en_ctx {
                    let de: String = strip_diacritics(&p.chars().collect::<Vec<_>>())
                        .into_iter()
                        .collect();
                    modes_seen[if vn_prefix_possible_unaccented(&de) {
                        1
                    } else {
                        2
                    }] = true;
                } else {
                    modes_seen[if is_vi { 2 } else { 0 }] = true;
                }
                for limit in [1, 25] {
                    let got =
                        predict_prefix_candidates_with_personal(p, is_vi, limit, &[], 0, en_ctx);
                    let exp = predict_impl_reference(p, is_vi, limit, &[], 0, en_ctx);
                    assert_eq!(
                        got, exp,
                        "LOOP-2b parity: prefix={p:?}, is_vi={is_vi}, en_ctx={en_ctx}, limit={limit}"
                    );
                }
            }
        }
        assert_eq!(modes_seen, [true; 3], "phải phủ đủ EnOnly/VnOnly/VnPlusEn");
    }

    /// LOOP-2b — tiền đề của lối tắt ASCII trong bộ lọc E84 (`vn_syllables_sorted`): tập
    /// `is_known_en_word` (en_common + en_freq) chỉ gồm từ ASCII. Ai thêm từ EN có dấu
    /// (`café`…) vào dữ liệu ⇒ đỏ ở đây, phải bỏ lối tắt chứ không lặng lẽ để lọt âm tiết.
    #[test]
    fn loop2b_tap_en_toan_ascii() {
        for (name, data) in [("en_common", EN_COMMON_DATA), ("en_freq3000", EN_FREQ_DATA)] {
            for line in data.lines().map(str::trim) {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                assert!(
                    line.is_ascii(),
                    "{name}: mục EN không ASCII {line:?} — bỏ lối tắt is_ascii() trong vn_syllables_sorted"
                );
            }
        }
        // Lối tắt không đổi kết quả: đối chiếu với bộ lọc đầy đủ.
        let dic_set: std::collections::HashSet<&str> =
            VN_DIC_DAUMOI_DATA.lines().map(str::trim).collect();
        let full: Vec<&str> = {
            let mut v: Vec<&str> = VN_SYLLABLES_DATA
                .lines()
                .map(str::trim)
                .filter(|l| {
                    let old = restyle(l, DiacriticStyle::Old);
                    !l.is_empty()
                        && !l.starts_with('#')
                        && (!is_known_en_word(l)
                            || dic_set.contains(l)
                            || dic_set.contains(old.as_str()))
                })
                .collect();
            v.sort_unstable();
            v
        };
        assert_eq!(vn_syllables_sorted(), &full[..]);
    }

    /// LOOP-2b / E90 — gate bộ nhớ TẤT ĐỊNH của chỉ mục tiền tố (lexicon + top-K cả 3 chế độ).
    /// DLL TSF nạp vào mọi tiến trình gõ chữ ⇒ mỗi MB nhân theo số app. LOOP-2 (3 pool
    /// nhân bản mục 40 B + bản sao top-K) đo ~2,1 MB cấu trúc; LOOP-2b ~0,95 MB. Đột biến
    /// quay lại pool nhân bản / sao chép mục vào top-K ⇒ vượt ngân sách ⇒ đỏ.
    #[test]
    fn loop2b_prefix_index_footprint_budget() {
        const BUDGET_BYTES: usize = 1_150_000;
        assert_eq!(std::mem::size_of::<StaticEntry>(), 40, "StaticEntry phình");
        let bytes = prefix_index_heap_bytes();
        println!("prefix index heap = {bytes} B");
        assert!(
            bytes <= BUDGET_BYTES,
            "chỉ mục tiền tố {bytes} B > ngân sách {BUDGET_BYTES} B (LOOP-2b)"
        );
        let lex = lexicon();
        let shadowed = lex
            .singles
            .iter()
            .chain(lex.compounds.iter())
            .filter(|e| e.shadowed_by_vn())
            .count();
        assert!(shadowed > 0, "phải có mục EN trùng từ Việt bị che ở VN+EN");
    }
}
