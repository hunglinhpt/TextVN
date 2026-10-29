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

use crate::post::restore_en::{EN_COMMON_DATA, EN_FREQ_DATA, TECH_BUSINESS_TERMS};
use crate::post::suggest_vn::VN_FREQ_DATA;
use crate::transform::charset::strip_diacritics;
use crate::transform::tone::normalize_tone;
use crate::transform::DiacriticStyle;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Vòng 41 (perf audit): dữ liệu tĩnh parse **MỘT LẦN** qua OnceLock.
//
// `predict_prefix_candidates` và `predict_next_phrase_matches` chạy **mỗi
// phím** khi dự đoán bật; quét ~6.000 dòng `lines()` + split/parse TSV mỗi
// gọi đo được ~0,7 ms (release, `core/tests/perf_probe.rs`). Cache tĩnh cắt
// phần parse; so khớp tiền tố / đuôi ngữ cảnh vẫn per-call theo input.
// ---------------------------------------------------------------------------

fn vn_freq_lines() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        VN_FREQ_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}

/// Từ ghép kèm trọng số: `(word, weight)`.
fn vn_compounds() -> &'static [(&'static str, u32)] {
    static CACHE: OnceLock<Vec<(&'static str, u32)>> = OnceLock::new();
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
                Some((word, weight))
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

fn en_freq_lines() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        EN_FREQ_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
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

/// Danh sách từ ghép tiếng Việt thông dụng (văn phòng, công nghệ, hành chính).
pub const VN_COMPOUNDS_DATA: &str = include_str!("../../../data/vn_compounds.txt");

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Việt (TSV: context \t prediction \t weight).
pub const VN_PHRASES_DATA: &str = include_str!("../../../data/vn_phrases.tsv");

/// Bảng ngữ cảnh cụm từ tiếp theo tiếng Anh (TSV: context \t prediction \t weight).
pub const EN_PHRASES_DATA: &str = include_str!("../../../data/en_phrases.tsv");

/// Dự đoán hoàn tất tiền tố (Prefix Completion).
///
/// Trả về danh sách ứng viên (tối đa `limit`), xếp theo độ ưu tiên:
/// từ ghép tiếng Việt khớp tiền tố → âm tiết tần suất cao → thuật ngữ EN / freq EN.
/// Cấu trúc ứng viên prefix với thông tin xếp hạng
#[derive(Debug, Clone)]
struct PrefixCandidate {
    word: String,
    exact_match: bool,
    char_count: usize,
    weight: u32,
}

/// Dự đoán hoàn tất tiền tố (Prefix Completion).
///
/// Trả về danh sách ứng viên (tối đa `limit`), xếp theo độ ưu tiên:
/// khớp trực tiếp (có dấu) > độ dài ký tự ngắn hơn (hoàn tất âm tiết trước) > weight cao hơn.
pub fn predict_prefix_candidates(typed_lower: &str, is_vi: bool, limit: usize) -> Vec<String> {
    if typed_lower.chars().count() < 2 || limit == 0 {
        return Vec::new();
    }

    let mut ranked: Vec<PrefixCandidate> = Vec::new();
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

    // 1. Nếu đang ở mode tiếng Việt hoặc chuỗi gõ có dấu tiếng Việt
    if is_vi {
        // 1a. Tra cứu âm tiết đơn tần suất cao (data/vn_freq3000.txt)
        // Ưu tiên hoàn tất âm tiết đang gõ dở
        for &syll in vn_freq_lines() {
            if starts(syll) && !same(syll) && !ranked.iter().any(|c| c.word == syll) {
                ranked.push(PrefixCandidate {
                    word: syll.to_string(),
                    exact_match: true,
                    char_count: syll.chars().count(),
                    weight: 90,
                });
            }
        }

        // 1b. Tra cứu từ ghép tiếng Việt (data/vn_compounds.txt)
        for &(word, weight) in vn_compounds() {
            if same(word) {
                continue;
            }

            let matches_accented = starts(word);
            // Chỉ khớp không dấu nếu người dùng chưa gõ dấu
            let matches_unaccented =
                if !matches_accented && !typed_has_diacritics && !typed_unaccented.is_empty() {
                    let word_unaccented: String =
                        strip_diacritics(&word.chars().collect::<Vec<_>>())
                            .into_iter()
                            .collect();
                    word_unaccented.starts_with(&typed_unaccented)
                } else {
                    false
                };

            if (matches_accented || matches_unaccented) && !ranked.iter().any(|c| c.word == word) {
                ranked.push(PrefixCandidate {
                    word: word.to_string(),
                    exact_match: matches_accented,
                    char_count: word.chars().count(),
                    weight,
                });
            }
        }
    }

    // 2. Tra cứu từ tiếng Anh (khi gõ từ tiếng Anh hoặc thuật ngữ coder/kinh doanh)
    // Ưu tiên TECH_BUSINESS_TERMS
    for term in TECH_BUSINESS_TERMS {
        let tl = term.to_lowercase();
        if tl.starts_with(typed_lower) && tl != typed_lower {
            let formatted = term.to_string();
            if !ranked.iter().any(|c| c.word == formatted) {
                ranked.push(PrefixCandidate {
                    word: formatted,
                    exact_match: true,
                    char_count: term.chars().count(),
                    weight: 85,
                });
            }
        }
    }

    // en_common
    for &word in en_common_lines() {
        if word.starts_with(typed_lower)
            && word != typed_lower
            && !ranked.iter().any(|c| c.word == word)
        {
            ranked.push(PrefixCandidate {
                word: word.to_string(),
                exact_match: true,
                char_count: word.chars().count(),
                weight: 70,
            });
        }
    }

    // en_freq3000
    for &word in en_freq_lines() {
        if word.starts_with(typed_lower)
            && word != typed_lower
            && !ranked.iter().any(|c| c.word == word)
        {
            ranked.push(PrefixCandidate {
                word: word.to_string(),
                exact_match: true,
                char_count: word.chars().count(),
                weight: 60,
            });
        }
    }

    // Xếp theo:
    // 1. exact_match (khớp trực tiếp có dấu)
    // 2. Từ đơn (âm tiết) ưu tiên trước từ ghép nếu đang gõ âm tiết
    // 3. Nếu là từ đơn: độ dài ngắn hơn trước (hoàn tất âm tiết); nếu là từ ghép: weight cao hơn trước
    ranked.sort_by(|a, b| {
        let a_syll = !a.word.contains(' ');
        let b_syll = !b.word.contains(' ');
        b.exact_match
            .cmp(&a.exact_match)
            .then_with(|| b_syll.cmp(&a_syll))
            .then_with(|| {
                if a_syll && b_syll {
                    a.char_count
                        .cmp(&b.char_count)
                        .then_with(|| b.weight.cmp(&a.weight))
                } else {
                    b.weight
                        .cmp(&a.weight)
                        .then_with(|| a.char_count.cmp(&b.char_count))
                }
            })
            .then_with(|| a.word.cmp(&b.word))
    });

    ranked.into_iter().take(limit).map(|c| c.word).collect()
}

/// Dự đoán cụm từ tiếp theo theo ngữ cảnh (Next-Word / Phrase Prediction).
///
/// `context_tail`: Đuôi văn bản gần nhất (Unicode) trước con trỏ (đã trim khoảng trắng đuôi).
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
    for rows in [vn_phrase_rows(), en_phrase_rows()] {
        collect_phrase_matches(rows, &context_lower, &mut matches);
    }

    // Xếp theo: key_len lớn hơn (trigram > bigram) -> weight cao hơn
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(limit);
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
/// `ends_with` + kiểm ranh giới từ.
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

    #[test]
    fn test_predict_next_phrase_english() {
        let preds = predict_next_phrase_candidates("thank you", 3);
        assert!(!preds.is_empty());
        assert_eq!(preds[0], "very much");

        let preds_look = predict_next_phrase_candidates("looking forward to", 3);
        assert!(!preds_look.is_empty());
        assert_eq!(preds_look[0], "hearing from you");
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
}
