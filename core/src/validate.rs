// SPDX-License-Identifier: GPL-3.0-or-later
//! Validate âm tiết — **stage 5** pipeline (PLAN §4.2): "5 quy tắc âm tiết".
//!
//! Dùng cho `post/restore_en.rs` (bug B5: gõ tiếng Anh ra dấu) và cho `doctor`/tooling.
//! 5 quy tắc + 1 luật thanh (đủ để từ tiếng Anh gõ Telex bị coi là **không hợp lệ**):
//!
//! 1. `has_vowel` — phải có ít nhất 1 nguyên âm.
//! 2. `valid_onset` — âm đầu thuộc bảng phụ âm đầu hợp lệ (kể cả rỗng).
//! 3. `valid_nucleus` — vần (cụm nguyên âm, đã bỏ dấu thanh) thuộc bảng vần hợp lệ.
//! 4. `valid_coda` — âm cuối thuộc bảng phụ âm cuối hợp lệ (kể cả rỗng).
//! 5. `tone_marks_ok` — tối đa **1** nguyên âm mang dấu thanh.
//! 6. `stop_coda_tone_ok` — âm cuối tắc `c ch p t` chỉ mang thanh **sắc** hoặc **nặng**
//!    (R2-62: `sort` `part` `hurt` gõ Telex ra `sỏt` `pảt` `hủt` không phải âm tiết Việt).
//! 7. `coda_after_nucleus_ok` — vần kết thúc bằng bán âm (`ai ao au ay ui ưu oi…`) và nguyên
//!    âm đôi mở (`ia ua ưa ya uơ`) không có âm cuối (`using` → `uíng`, `win` → `ưin` không phải
//!    âm tiết Việt; có âm cuối thì viết `iê uô ươ`: `tiến`, `muốn`, `người`).
//!
//! Nguồn: cấu trúc âm tiết Việt (âm đầu + vần + âm cuộc) theo `PLAN §4.2` stage 5;
//! bảng vần viết tay (chưa có `data/tables/` — deviation ghi `docs/00-INDEX §5`).

use crate::transform::tone::{current_tone, is_vowel, tone_of};
use crate::transform::undo::unmark;

/// Phụ âm đầu hợp lệ (P0-1 §1 `validate.rs`).
pub const ONSETS: [&str; 28] = [
    "", "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh",
    "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x",
];

/// Phụ âm cuối hợp lệ (chuẩn: 8 phụ âm cuối, viết dạng đôi `ch/nh/ng`).
pub const CODAS: [&str; 9] = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

/// Bảng vần hợp lệ (đã bỏ dấu thanh; giữ dạng viết `iê/uô/ươ`).
pub const NUCLEI: [&str; 57] = [
    // nguyên âm đơn
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", //
    // 2 nguyên âm
    "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu", "ia", "iê", "iu", "oa", "oă", "oe", "oi", "ôi",
    "ơi", "oo", "ua", "uâ", "uê", "ui", "uô", "uơ", "uy", "ưa", "ưi", "ưu", "ươ", "ya",
    "yê", //
    // 3 nguyên âm
    "iêu", "oai", "oao", "oay", "oeo", "uai", "uây", "uôi", "uya", "uyê", "uyu", "ươi", "ươu",
    "yêu",
];

/// Cấu trúc tách được: `[onset][nucleus][coda]` (chỉ số trong `cs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Structure {
    pub onset_end: usize,
    pub nucleus_end: usize,
}

/// Tách `cs` thành âm đầu / vần / âm cuối. `None` khi không có nguyên âm (rule 1).
///
/// Xử lý riêng `qu` (âm đầu `qu`, `u` **không** thuộc vần: `quy`, `quà`).
pub fn decompose(cs: &[char]) -> Option<Structure> {
    let vstart = cs.iter().position(|&c| is_vowel(c))?;
    let vend = cs.iter().rposition(|&c| is_vowel(c))?;
    // `qu`: âm đầu gồm cả `u` → vần bắt đầu sau `u`. `gi` + nguyên âm khác: `i` thuộc âm
    // đầu `gi` (`giáo`, `giữa`, `giường`); `gì`/`gìn` thì `i` là vần.
    let qu = vstart == 1 && matches!(cs[0], 'q' | 'Q') && matches!(cs[1], 'u' | 'U');
    let gi =
        vstart == 1 && vend > 1 && matches!(cs[0], 'g' | 'G') && matches!(unmark(cs[1]), 'i' | 'I');
    let onset_end = if qu || gi { 2 } else { vstart };
    if onset_end > vend {
        return None; // `qu` mà không còn nguyên âm (`qu`) → không phải âm tiết
    }
    Some(Structure {
        onset_end,
        nucleus_end: vend,
    })
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn lower_str(cs: &[char]) -> String {
    cs.iter().copied().map(lower).collect()
}

/// Rule 1 — có ít nhất 1 nguyên âm.
pub fn has_vowel(cs: &[char]) -> bool {
    cs.iter().any(|&c| is_vowel(c))
}

/// Rule 2 — âm đầu hợp lệ.
pub fn valid_onset(cs: &[char], st: Structure) -> bool {
    ONSETS.contains(&lower_str(&cs[..st.onset_end]).as_str())
}

/// Rule 3 — vần hợp lệ (bỏ dấu thanh trước khi tra bảng).
pub fn valid_nucleus(cs: &[char], st: Structure) -> bool {
    let nucleus: Vec<char> = cs[st.onset_end..=st.nucleus_end]
        .iter()
        .map(|&c| unmark(c))
        .collect();
    NUCLEI.contains(&lower_str(&nucleus).as_str())
}

/// Rule 4 — âm cuối hợp lệ.
pub fn valid_coda(cs: &[char], st: Structure) -> bool {
    CODAS.contains(&lower_str(&cs[st.nucleus_end + 1..]).as_str())
}

/// Rule 5 — tối đa 1 nguyên âm mang dấu thanh.
pub fn tone_marks_ok(cs: &[char]) -> bool {
    cs.iter()
        .filter(|&&c| tone_of(c).is_some_and(|t| t > 0))
        .count()
        <= 1
}

/// Rule 6 — âm tiết khép bằng âm cuối tắc (`c ch p t`) chỉ có thanh sắc hoặc nặng (`tốt`,
/// `học`, `ích`, `đẹp`); không dấu, huyền, hỏi, ngã đều không có (`sỏt`, `pảt`, `tẽt`, `viêt`).
pub fn stop_coda_tone_ok(cs: &[char], st: Structure) -> bool {
    let coda = lower_str(&cs[st.nucleus_end + 1..]);
    !matches!(coda.as_str(), "c" | "ch" | "p" | "t") || matches!(current_tone(cs), 1 | 5)
}

/// Vần (đã bỏ dấu thanh) **không** đi với âm cuối: kết thúc bằng bán âm `i/y/o/u` hoặc là
/// nguyên âm đôi mở `ia ua ưa ya uơ` (có âm cuối thì viết `iê yê uô ươ`).
pub const OPEN_NUCLEI: [&str; 33] = [
    "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu", "ia", "iu", "oi", "ôi", "ơi", "ua", "ui", "ưa",
    "ưi", "ưu", "uơ", "ya", "iêu", "yêu", "oai", "oao", "oay", "oeo", "uai", "uây", "uôi", "uya",
    "uyu", "ươi", "ươu",
];

/// Rule 7 — vần trong [`OPEN_NUCLEI`] không có âm cuối (`uíng`, `ưin`, `ửam` không phải âm tiết).
pub fn coda_after_nucleus_ok(cs: &[char], st: Structure) -> bool {
    if st.nucleus_end + 1 >= cs.len() {
        return true;
    }
    let nucleus: Vec<char> = cs[st.onset_end..=st.nucleus_end]
        .iter()
        .map(|&c| unmark(c))
        .collect();
    !OPEN_NUCLEI.contains(&lower_str(&nucleus).as_str())
}

/// **5 quy tắc âm tiết + luật thanh âm cuối tắc + vần mở** — `true` nếu `cs` là một âm tiết
/// Việt hợp lệ.
pub fn is_valid_word(cs: &[char]) -> bool {
    let Some(st) = decompose(cs) else {
        return false;
    };
    has_vowel(cs)
        && valid_onset(cs, st)
        && valid_nucleus(cs, st)
        && valid_coda(cs, st)
        && tone_marks_ok(cs)
        && stop_coda_tone_ok(cs, st)
        && coda_after_nucleus_ok(cs, st)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> bool {
        is_valid_word(&s.chars().collect::<Vec<_>>())
    }

    #[test]
    fn valid_vietnamese_words() {
        for w in [
            "được",
            "đường",
            "giáo",
            "giữa",
            "giường",
            "giờ",
            "gì",
            "gìn",
            "giếng",
            "quở",
            "thuở",
            "hoà",
            "hòa",
            "nguyễn",
            "quy",
            "quà",
            "qua",
            "y",
            "ăn",
            "tiếng",
            "việt",
            "trường",
            "khuỷu",
            "xoong",
            "thuở",
            "già",
            "gìn",
            "hoa",
            "loan",
            "mía",
        ] {
            assert!(v(w), "`{w}` phải hợp lệ");
        }
    }

    #[test]
    fn invalid_english_and_typos() {
        for w in [
            "àd", "as", "text", "hello", "bs", "qaa", "ngx", "a?", "abc7",
        ] {
            assert!(!v(w), "`{w}` phải KHÔNG hợp lệ");
        }
    }

    /// Vần kết thúc bằng bán âm / nguyên âm đôi mở không có âm cuối.
    #[test]
    fn open_nucleus_takes_no_coda() {
        for w in [
            "tuổi", "người", "rượu", "khuya", "khuỷu", "mưa", "gửi", "lưu", "cái", "này", "giữa",
            "của", "quan", "quang", "xoong", "huynh", "khuếch", "tuần", "giếng", "yên", "hoàng",
        ] {
            assert!(v(w), "`{w}` phải hợp lệ");
        }
        for w in [
            "uíng", "muíc", "duỉng", "ưin", "ứin", "ửam", "tưin", "sưing", "cain", "mưan",
        ] {
            assert!(!v(w), "`{w}` phải KHÔNG hợp lệ");
        }
    }

    /// R2-62: âm cuối tắc `c ch p t` chỉ đi với thanh sắc/nặng.
    #[test]
    fn stop_coda_takes_only_sac_or_nang() {
        for w in [
            "tốt", "học", "ích", "đẹp", "việt", "Việt", "VIỆT", "được", "sách", "mạch", "quốc",
        ] {
            assert!(v(w), "`{w}` phải hợp lệ");
        }
        for w in [
            "sỏt", "pỏt", "pảt", "hủt", "chảt", "mảt", "dỉt", "cẻt", "tẽt", "nẽt", "ảt", "viêt",
            "Viêt", "KIÊT", "bôt", "mêt", "kêp", "hòc", "ãch",
        ] {
            assert!(!v(w), "`{w}` phải KHÔNG hợp lệ");
        }
        // Âm cuối mũi/vần mở không bị ràng buộc.
        for w in ["hỏi", "ngã", "tiên", "nhà", "lòng", "anh"] {
            assert!(v(w), "`{w}` phải hợp lệ");
        }
    }

    #[test]
    fn rule_by_rule() {
        let cs: Vec<char> = "duoc".chars().collect();
        let st = decompose(&cs).expect("có nguyên âm");
        assert_eq!((st.onset_end, st.nucleus_end), (1, 2)); // "d" + "uo" + "c"
        assert!(has_vowel(&cs) && valid_onset(&cs, st) && valid_coda(&cs, st));

        let cs: Vec<char> = "asdf".chars().collect();
        let st = decompose(&cs).expect("có nguyên âm");
        assert!(!valid_coda(&cs, st), "'f' không phải âm cuối hợp lệ");
    }

    #[test]
    fn qu_nucleus_starts_after_u() {
        let cs: Vec<char> = "quy".chars().collect();
        let st = decompose(&cs).unwrap();
        assert_eq!(st.onset_end, 2, "`qu` là âm đầu");
        assert!(valid_nucleus(&cs, st));
    }

    #[test]
    fn no_vowel_is_invalid() {
        assert!(!v("bcd"));
        assert!(!v("b"));
        assert!(!v(""));
    }

    #[test]
    fn at_most_one_toned_vowel() {
        assert!(v("hoà"));
        assert!(!is_valid_word(&"ấề".chars().collect::<Vec<_>>()));
    }
}
