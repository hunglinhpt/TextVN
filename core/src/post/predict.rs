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

use crate::post::restore_en::{
    EN_COMMON_DATA, EN_FREQ_DATA, EN_WORDS_10K_DATA, TECH_BUSINESS_TERMS,
};
use crate::post::suggest_vn::VN_SYLLABLES_DATA;
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
fn vn_syllables_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
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

/// Vòng 53 (perf): từ ghép bucket theo **byte đầu** — 1.9k mục quét tuyến
/// tính từng ~60µs trên prefix ngắn; bucket chỉ quét nhóm đầu-vòng (giảm ~10×).
/// Kết hợp không-dấu và có-dấu cùng lưu (khớp qua 2 đường).
/// Nhóm từ ghép theo byte đầu: (từ, dạng-không-dấu, weight).
type CompoundRow = (&'static str, &'static str, u32);

fn vn_compounds_buckets() -> &'static std::collections::HashMap<u8, &'static [CompoundRow]> {
    static CACHE: OnceLock<std::collections::HashMap<u8, &'static [CompoundRow]>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut groups: std::collections::HashMap<u8, Vec<CompoundRow>> =
            std::collections::HashMap::new();
        for &(word, de, weight) in vn_compounds() {
            if let Some(&b0) = word.as_bytes().first() {
                groups.entry(b0).or_default().push((word, de, weight));
            }
            if let Some(&b0) = de.as_bytes().first() {
                groups.entry(b0).or_default().push((word, de, weight));
            }
        }
        groups
            .into_iter()
            .map(|(k, v)| {
                let slice: &'static [CompoundRow] = Box::leak(v.into_boxed_slice());
                (k, slice)
            })
            .collect()
    })
}

/// Thuật ngữ tech/business kèm dạng lowercase (build 1 lần — `seen` dedup
/// theo dạng chuẩn giữ case, comparator char_count vẫn đếm từ dạng gốc).
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

/// Bucket các hàng phrase theo **byte cuối của khoá** (vòng 52): ngữ cảnh chỉ
/// có thể khớp khoá mà kết thúc bằng đúng byte cuối của nó → mỗi call chỉ quét
/// bucket tương ứng (giảm ~20-30× với ~10k hàng thay vì quét toàn bảng).
/// Vòng 53: nhận cờ `vn` tường minh thay so-con-trỏ (an toàn khi một bảng rỗng).
fn phrase_buckets(
    rows: &'static [PhraseRow],
    vn: bool,
) -> &'static std::collections::HashMap<u8, &'static [PhraseRow]> {
    static VN: OnceLock<std::collections::HashMap<u8, &'static [PhraseRow]>> = OnceLock::new();
    static EN: OnceLock<std::collections::HashMap<u8, &'static [PhraseRow]>> = OnceLock::new();
    let cache = if vn { &VN } else { &EN };
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

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Việt (TSV: context \t prediction \t weight).
pub const VN_PHRASES_DATA: &str = include_str!("../../../data/vn_phrases.tsv");

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Anh (TSV: context \t prediction \t weight).
pub const EN_PHRASES_DATA: &str = include_str!("../../../data/en_phrases.tsv");

/// Ứng viên prefix với thông tin xếp hạng — giữ `&'static str` (vòng 52: không
/// alloc String cho ứng viên bị loại; chỉ `take(limit)` cuối cùng mới cấp phát).
#[derive(Debug, Clone)]
struct PrefixCandidate {
    word: &'static str,
    exact_match: bool,
    char_count: usize,
    weight: u32,
    /// Từ ghép có ÂM ĐẦU == tiền tố ("tiến độ" cho "tiến") — mở rộng đúng
    /// nghĩa; compound chỉ prefix-match byte ("người yêu" cho "ngư") thì false.
    ext: bool,
}

/// Dự đoán hoàn tất tiền tố (Prefix Completion).
///
/// Trả về danh sách ứng viên (tối đa `limit`), xếp theo độ ưu tiên:
/// khớp trực tiếp (có dấu) > độ dài ký tự ngắn hơn (hoàn tất âm tiết trước) > weight cao hơn.
pub fn predict_prefix_candidates(typed_lower: &str, is_vi: bool, limit: usize) -> Vec<String> {
    if typed_lower.chars().count() < 2 || limit == 0 {
        return Vec::new();
    }

    // Vòng 52: dedup qua HashSet O(1) thay `ranked.iter().any()` O(n) mỗi push
    // (prefix ngắn như "t" từng thu hàng nghìn ứng viên → O(n²) so String).
    let mut ranked: Vec<PrefixCandidate> = Vec::new();
    let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
    let typed_unaccented: String = strip_diacritics(&typed_lower.chars().collect::<Vec<_>>())
        .into_iter()
        .collect();
    let typed_has_diacritics = typed_lower != typed_unaccented;
    // Dữ liệu viết kiểu dấu cũ: "hoà" (kiểu mới) phải khớp "hòa bình".
    // Tiền tố gõ dở đặt dấu khác vị trí với từ đầy đủ ("hòa" kiểu cũ là tiền
    // tố "hoài" chỉ khi đặt kiểu mới "hoà") → so cả hai kiểu.
    let typed_old = restyle(typed_lower, DiacriticStyle::Old);
    let typed_new = restyle(typed_lower, DiacriticStyle::New);
    let starts = |w: &str| {
        w.starts_with(typed_lower)
            || w.starts_with(typed_old.as_str())
            || w.starts_with(typed_new.as_str())
    };
    let same = |w: &str| w == typed_lower || w == typed_old || w == typed_new;
    // Vòng 52 (probe): từ ghép là mở rộng đúng nghĩa khi ÂM ĐẦU == tiền tố
    // ("tiến độ" của "tiến" — người gõ đã gõ xong âm tiết, muốn thành từ
    // ghép); compound chỉ prefix-match byte ("người yêu" của "ngư") thì KHÔNG
    // — "ngư" đang gõ dở âm tiết dài hơn, giữ luật hoàn-tất-âm-tiết-trước.
    let compound_ext = |word: &str| word.split(' ').next().is_some_and(same);

    // 1. Nếu đang ở mode tiếng Việt hoặc chuỗi gõ có dấu tiếng Việt
    if is_vi {
        // 1a. Âm tiết đơn (vn_syllables.txt) — binary search theo từng biến thể
        // (giữ nguyên ngữ nghĩa `starts() && !same()` của vòng trước).
        let push_syll =
            |syll: &'static str,
             ranked: &mut Vec<PrefixCandidate>,
             seen: &mut std::collections::HashSet<&'static str>| {
                if same(syll) || !seen.insert(syll) {
                    return;
                }
                ranked.push(PrefixCandidate {
                    word: syll,
                    exact_match: true,
                    char_count: syll.chars().count(),
                    weight: 90,
                    ext: false,
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

        // 1b. Từ ghép tiếng Việt (vn_compounds.txt) — ~1200 mục, quét tuyến
        // tính nhưng deaccent đã build sẵn trong cache (không alloc per-call).
        // Vòng 53 (perf): quét bucket byte-đầu (có-dấu + không-dấu) thay toàn bảng.
        let buckets = vn_compounds_buckets();
        let mut scan = |key: u8| {
            for &(word, word_unaccented, weight) in buckets.get(&key).copied().into_iter().flatten()
            {
                if same(word) {
                    continue;
                }

                let matches_accented = starts(word);
                // Chỉ khớp không dấu nếu người dùng chưa gõ dấu
                let matches_unaccented =
                    if !matches_accented && !typed_has_diacritics && !typed_unaccented.is_empty() {
                        word_unaccented.starts_with(&typed_unaccented)
                    } else {
                        false
                    };

                if (matches_accented || matches_unaccented) && seen.insert(word) {
                    ranked.push(PrefixCandidate {
                        word,
                        exact_match: matches_accented,
                        char_count: word.chars().count(),
                        weight,
                        ext: compound_ext(word),
                    });
                }
            }
        };
        if let Some(&b0) = typed_lower.as_bytes().first() {
            scan(b0);
        }
        if let Some(&b0) = typed_unaccented.as_bytes().first() {
            scan(b0);
        }
        if let Some(&b0) = typed_old.as_bytes().first() {
            scan(b0);
        }
        if let Some(&b0) = typed_new.as_bytes().first() {
            scan(b0);
        }
    }

    // 2. Tra cứu từ tiếng Anh (khi gõ từ tiếng Anh hoặc thuật ngữ coder/kinh doanh)
    // Ưu tiên TECH_BUSINESS_TERMS — lowercase build 1 lần (OnceLock) thay vì
    // `to_lowercase()` + leak mỗi call (vòng 52: chống rò rỉ bộ nhớ mỗi phím).
    for &(term, tl) in tech_terms_lower() {
        if tl.starts_with(typed_lower) && tl != typed_lower && seen.insert(term) {
            ranked.push(PrefixCandidate {
                word: term,
                exact_match: true,
                char_count: term.chars().count(),
                weight: 85,
                ext: false,
            });
        }
    }

    // en_common (danh sách restore-policy, ~150 mục — tuyến tính)
    for &word in en_common_lines() {
        if word.starts_with(typed_lower) && word != typed_lower && seen.insert(word) {
            ranked.push(PrefixCandidate {
                word,
                exact_match: true,
                char_count: word.chars().count(),
                weight: 70,
                ext: false,
            });
        }
    }

    // en_freq3000 (vòng 52: binary search)
    for word in prefix_scan(en_freq_sorted(), typed_lower) {
        if word != typed_lower && seen.insert(word) {
            ranked.push(PrefixCandidate {
                word,
                exact_match: true,
                char_count: word.chars().count(),
                weight: 60,
                ext: false,
            });
        }
    }

    // en_words10k — google-10000-english (vòng 52): nguồn mở rộng prediction,
    // weight 55 dưới freq3000. KHÔNG đụng chính sách restore/assist.
    for word in prefix_scan(en_words10k_sorted(), typed_lower) {
        if word != typed_lower && seen.insert(word) {
            ranked.push(PrefixCandidate {
                word,
                exact_match: true,
                char_count: word.chars().count(),
                weight: 55,
                ext: false,
            });
        }
    }

    // Xếp theo:
    // Xếp theo (vòng 52):
    // 1. exact_match (khớp trực tiếp có dấu)
    // 2. mở rộng đúng nghĩa — từ ghép có ÂM ĐẦU == tiền tố ("tiến độ" của "tiến")
    // 3. nhóm: có mở rộng đúng nghĩa ("tiến") → từ ghép trước âm tiết khác;
    //    không có ("ngư" — đang gõ dở âm tiết dài hơn) → hoàn tất âm tiết trước
    // 4. từ đơn: ngắn hơn trước; từ ghép: weight cao hơn trước
    let has_ext = ranked.iter().any(|c| c.ext);
    ranked.sort_by(|a, b| {
        let a_single = !a.word.contains(' ');
        let b_single = !b.word.contains(' ');
        b.exact_match
            .cmp(&a.exact_match)
            .then_with(|| b.ext.cmp(&a.ext))
            .then_with(|| {
                let a_pri = if has_ext { !a_single } else { a_single };
                let b_pri = if has_ext { !b_single } else { b_single };
                b_pri.cmp(&a_pri)
            })
            .then_with(|| {
                if a_single && b_single {
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
    });

    ranked
        .into_iter()
        .take(limit)
        .map(|c| c.word.to_string())
        .collect()
}

/// Quét vùng prefix của danh sách đã sắp (vòng 52): binary search điểm bắt đầu
/// rồi đi đến khi từ không còn khớp prefix.
fn prefix_scan<'a>(
    sorted: &'static [&'static str],
    typed: &'a str,
) -> impl Iterator<Item = &'static str> + 'a {
    let start = sorted.partition_point(|w| *w < typed);
    sorted[start..]
        .iter()
        .copied()
        .take_while(move |w| w.starts_with(typed))
}

/// Dự đoán cụm từ tiếp theo theo ngữ cảnh (Next-Word / Phrase Prediction).
///
/// `context_tail`: Đuôi văn bản gần nhất trước con trỏ (đã trim khoảng trắng đuôi).
/// Trả về danh sách ứng viên cụm từ tiếp theo (tối đa `limit`), xếp theo độ dài khóa ngữ cảnh
/// giảm dần (trigram trước bigram) và trọng số `weight`.
pub fn predict_next_phrase_candidates(context_tail: &str, limit: usize) -> Vec<String> {
    predict_next_phrase_matches(context_tail, limit)
        .into_iter()
        .map(|m| m.prediction.to_string())
        .collect()
}

/// Một ứng viên cụm từ tiếp theo kèm **số từ của khoá ngữ cảnh** đã khớp.
///
/// `key_words >= 2` = ngữ cảnh nhiều từ ("phát triển", "kính gửi") — tín hiệu
/// mạnh hơn hoàn tất tiền tố của riêng âm tiết cuối (engine dùng để quyết định
/// thứ tự Tab: `phát triển`+Tab phải ra "phần mềm", không phải "triển khai").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseMatch {
    pub prediction: &'static str,
    pub key_words: usize,
}

/// Như [`predict_next_phrase_candidates`] nhưng trả kèm số từ của khoá khớp.
///
/// Ngữ cảnh kết thúc bằng **xuống dòng** (`"Kính gửi\n"`) là dòng mới — không
/// dự đoán nối tiếp dòng trên (trước đây chèn `" quý khách hàng"` đầu dòng mới).
pub fn predict_next_phrase_matches(context_tail: &str, limit: usize) -> Vec<PhraseMatch> {
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
    let mut matches: Vec<(usize, u32, &'static str, usize)> = Vec::new();
    // Vòng 52 (perf): bucket theo byte cuối — chỉ quét các hàng có khả năng
    // khớp (`ends_with` bắt buộc byte cuối trùng), thay vì toàn bộ ~10k hàng.
    for (i, rows) in [vn_phrase_rows(), en_phrase_rows()].into_iter().enumerate() {
        let buckets = phrase_buckets(rows, i == 0);
        if let Some(&last) = context_lower.as_bytes().last() {
            if let Some(bucket) = buckets.get(&last) {
                collect_phrase_matches(bucket, &context_lower, &mut matches);
            }
        }
    }

    // Xếp theo: key_len lớn hơn (trigram > bigram) -> weight cao hơn
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut seen = std::collections::HashSet::new();
    // Vòng 53: guard limit độc hại (usize::MAX sẽ abort khi cấp phát —
    // call site cứng 5 nhưng đề phòng API công khai).
    let mut out = Vec::with_capacity(limit.min(64));
    for (_, _, pred, key_words) in matches {
        if seen.insert(pred) {
            out.push(PhraseMatch {
                prediction: pred,
                key_words,
            });
            if out.len() >= limit {
                break;
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
    out: &mut Vec<(usize, u32, &'static str, usize)>,
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
            out.push((key.len(), weight, pred, key.split_whitespace().count()));
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
    /// ký tự (R1-2), khoá ≤ 60, không dòng rác.
    #[test]
    fn du_lieu_phrases_invariant() {
        for (name, data) in [("vn", VN_PHRASES_DATA), ("en", EN_PHRASES_DATA)] {
            let rows = parse_phrase_rows(data);
            assert!(rows.len() >= 2000, "{name}: quá ít hàng: {}", rows.len());
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
                assert!(weight > 0 && weight <= 100, "{name}: weight {weight}");
                assert!(
                    !key.chars().any(|c| c.is_uppercase()),
                    "{name}: khoá phải lowercase: {key}"
                );
            }
        }
    }
}
