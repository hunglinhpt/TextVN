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

    // 1. Nếu đang ở mode tiếng Việt hoặc chuỗi gõ có dấu tiếng Việt
    if is_vi {
        // 1a. Tra cứu âm tiết đơn tần suất cao (data/vn_freq3000.txt)
        // Ưu tiên hoàn tất âm tiết đang gõ dở
        for line in VN_FREQ_DATA.lines() {
            let syll = line.trim();
            if syll.is_empty() || syll.starts_with('#') {
                continue;
            }
            if syll.starts_with(typed_lower)
                && syll != typed_lower
                && !ranked.iter().any(|c| c.word == syll)
            {
                ranked.push(PrefixCandidate {
                    word: syll.to_string(),
                    exact_match: true,
                    char_count: syll.chars().count(),
                    weight: 90,
                });
            }
        }

        // 1b. Tra cứu từ ghép tiếng Việt (data/vn_compounds.txt)
        for line in VN_COMPOUNDS_DATA.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split('\t');
            let word = parts.next().unwrap_or("").trim();
            let weight: u32 = parts
                .next()
                .and_then(|w| w.trim().parse().ok())
                .unwrap_or(50);

            if word.is_empty() || word == typed_lower {
                continue;
            }

            let matches_accented = word.starts_with(typed_lower);
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
    for line in EN_COMMON_DATA.lines() {
        let word = line.trim();
        if word.is_empty() || word.starts_with('#') {
            continue;
        }
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
    for line in EN_FREQ_DATA.lines() {
        let word = line.trim();
        if word.is_empty() || word.starts_with('#') {
            continue;
        }
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
    let trimmed = context_tail.trim();
    if trimmed.is_empty() || limit == 0 {
        return Vec::new();
    }
    let context_lower = trimmed.to_lowercase();

    // Cấu trúc tạm để chứa (key_len, weight, prediction)
    let mut matches: Vec<(usize, u32, &str)> = Vec::new();

    // 1. Quét ngữ cảnh tiếng Việt
    for line in VN_PHRASES_DATA.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let key = parts[0].trim();
            let pred = parts[1].trim();
            let weight: u32 = parts.get(2).and_then(|w| w.parse().ok()).unwrap_or(50);
            if context_lower.ends_with(key) {
                // Đảm bảo ranh giới từ ở đầu key nếu context dài hơn key
                let matched_boundary = if context_lower.len() == key.len() {
                    true
                } else {
                    let prefix_len = context_lower.len() - key.len();
                    context_lower
                        .as_bytes()
                        .get(prefix_len - 1)
                        .is_none_or(|&b| {
                            b == b' '
                                || b == b'.'
                                || b == b','
                                || b == b'!'
                                || b == b'?'
                                || b == b'\n'
                        })
                };

                if matched_boundary {
                    matches.push((key.len(), weight, pred));
                }
            }
        }
    }

    // 2. Quét ngữ cảnh tiếng Anh
    for line in EN_PHRASES_DATA.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let key = parts[0].trim();
            let pred = parts[1].trim();
            let weight: u32 = parts.get(2).and_then(|w| w.parse().ok()).unwrap_or(50);
            if context_lower.ends_with(key) {
                let matched_boundary = if context_lower.len() == key.len() {
                    true
                } else {
                    let prefix_len = context_lower.len() - key.len();
                    context_lower
                        .as_bytes()
                        .get(prefix_len - 1)
                        .is_none_or(|&b| {
                            b == b' '
                                || b == b'.'
                                || b == b','
                                || b == b'!'
                                || b == b'?'
                                || b == b'\n'
                        })
                };

                if matched_boundary {
                    matches.push((key.len(), weight, pred));
                }
            }
        }
    }

    // Xếp theo: key_len lớn hơn (trigram > bigram) -> weight cao hơn
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(limit);
    for (_, _, pred) in matches {
        if seen.insert(pred) {
            out.push(pred.to_string());
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

/// Khôi phục chữ hoa/thường theo mẫu gõ gốc.
///
/// - Nếu mẫu gốc viết hoa chữ cái đầu (`"Phát"`, `"Thank"`) → viết hoa chữ cái đầu kết quả (`"Phát triển"`, `"Thank you"`).
/// - Nếu mẫu gốc viết hoa toàn bộ (`"PHÁT"`, `"THANK"`) → viết hoa toàn bộ kết quả.
/// - Ngược lại giữ nguyên chữ thường.
pub fn adapt_capitalization(sample: &str, text: &str) -> String {
    if sample.is_empty() || text.is_empty() {
        return text.to_string();
    }
    let first_sample = sample.chars().next().unwrap();
    let is_all_upper = sample
        .chars()
        .all(|c| !c.is_alphabetic() || c.is_uppercase());

    if is_all_upper && sample.chars().any(|c| c.is_alphabetic()) {
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
        assert_eq!(preds[0], "xã hội chủ nghĩa việt nam");

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
}
