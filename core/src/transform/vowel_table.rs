// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng 72 entry âm Việt: 12 âm gốc × 6 dạng (không dấu + 5 dấu thanh).
//!
//! **TẠM viết tay** — theo P0-1 §3 file này eventually là output của
//! `cargo xtask gen-tables` từ `data/tables/*.toml` (không edit tay sau khi xtask có).
//! Thứ tự tone: `[không dấu, sắc, huyền, hỏi, ngã, nặng]` = index 0..=5.

/// Một dòng bảng: âm gốc + 6 dạng.
pub struct VowelEntry {
    pub base: char,
    pub forms: [char; 6],
}

/// Index âm trong bảng (dùng cho map w-horn / circumflex).
pub const A: usize = 0;
pub const A_BREVE: usize = 1; // ă
pub const A_CIRC: usize = 2; // â
pub const E: usize = 3;
pub const E_CIRC: usize = 4; // ê
pub const I: usize = 5;
pub const O: usize = 6;
pub const O_CIRC: usize = 7; // ô
pub const O_HOOK: usize = 8; // ơ
pub const U: usize = 9;
pub const U_HOOK: usize = 10; // ư
pub const Y: usize = 11;

pub const VOWELS: [VowelEntry; 12] = [
    VowelEntry {
        base: 'a',
        forms: ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    },
    VowelEntry {
        base: 'ă',
        forms: ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    },
    VowelEntry {
        base: 'â',
        forms: ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    },
    VowelEntry {
        base: 'e',
        forms: ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    },
    VowelEntry {
        base: 'ê',
        forms: ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    },
    VowelEntry {
        base: 'i',
        forms: ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    },
    VowelEntry {
        base: 'o',
        forms: ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    },
    VowelEntry {
        base: 'ô',
        forms: ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    },
    VowelEntry {
        base: 'ơ',
        forms: ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    },
    VowelEntry {
        base: 'u',
        forms: ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    },
    VowelEntry {
        base: 'ư',
        forms: ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    },
    VowelEntry {
        base: 'y',
        forms: ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
    },
];

/// Tra ký tự bất kỳ → (index âm, index tone 0..=6). Không phải âm → `None`.
/// Chấp nhận cả IN HOA ('Á' → ('a'-entry, tone 1)) để gõ HOA vẫn đặt dấu được.
pub fn locate(c: char) -> Option<(usize, usize)> {
    let lc = if c.is_uppercase() {
        c.to_lowercase().next()?
    } else {
        c
    };
    for (e, entry) in VOWELS.iter().enumerate() {
        if let Some(t) = entry.forms.iter().position(|&f| f == lc) {
            return Some((e, t));
        }
    }
    None
}

/// Dạng lowercase của âm `entry` với tone `tone`.
pub fn form(entry: usize, tone: usize) -> char {
    VOWELS[entry].forms[tone.min(5)]
}

/// Giữ **case của `sample`** khi tạo dạng (mới, tone) — ví dụ 'A' + sắc → 'Á'.
pub fn form_like(sample: char, entry: usize, tone: usize) -> char {
    let f = form(entry, tone);
    if sample.is_uppercase() {
        f.to_uppercase().next().unwrap_or(f)
    } else {
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_72_distinct_forms() {
        let mut all = std::collections::HashSet::new();
        for e in &VOWELS {
            for f in e.forms {
                assert!(all.insert(f), "trùng ký tự trong bảng: {f}");
            }
        }
        assert_eq!(all.len(), 72);
    }

    #[test]
    fn locate_roundtrip() {
        assert_eq!(locate('ự'), Some((U_HOOK, 5)));
        assert_eq!(locate('A'), Some((A, 0))); // IN HOA vẫn tra được
        assert_eq!(locate('Á'), Some((A, 1)));
        assert_eq!(locate('d'), None);
        assert_eq!(locate('ý'), Some((Y, 1)));
    }

    #[test]
    fn case_preserved() {
        assert_eq!(form_like('a', A, 1), 'á'); // sample thường → output thường
        assert_eq!(form_like('A', A, 1), 'Á'); // sample HOA → output HOA
        assert_eq!(form_like('ơ', O_HOOK, 5), 'ợ');
    }
}
