// SPDX-License-Identifier: GPL-3.0-or-later
//! Tra cứu âm Việt: bảng 72 ký tự + hàm tìm vị trí (dùng chung mọi method).
//!
//! **Bảng dữ liệu** (72 ký tự, hằng index âm) là GENERATED từ `data/tables/vowels.toml`
//! qua `cargo xtask gen-tables` — nằm ở `transform/vowel_table_generated.rs`.
//! File này chỉ chứa **logic** (tìm vị trí, gỡ dạng âm) — sửa bảng thì sửa `.toml`.
//!
//! Thứ tự tone trong mỗi dạng: `[không dấu, sắc, huyền, hỏi, ngã, nặng]` = index 0..=5.

/// Một dòng bảng: âm gốc + 6 dạng (sinh ra từ `.toml`).
pub struct VowelEntry {
    pub base: char,
    pub forms: [char; 6],
}

// Bảng + hằng index âm đã sinh từ data (không viết lại ở đây).
pub use super::vowel_table_generated::{
    A, A_BREVE, A_CIRC, E, E_CIRC, I, O, O_CIRC, O_HOOK, TONES_PER_VOWEL, U, U_HOOK, VOWELS,
    VOWEL_ENTRY_COUNT, Y,
};

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

/// Entry "gốc" (không mũ/sừng/breve) của một entry: â→a · ă→a · ê→e · ô→o · ơ→o · ư→u.
/// Dùng khi gỡ dạng âm (undo marker `w`/`6`/`7`/`8`/`^`/`+`/`(` và VNI `0`).
pub fn base_entry(e: usize) -> usize {
    match e {
        A_BREVE | A_CIRC => A,
        E_CIRC => E,
        O_CIRC | O_HOOK => O,
        U_HOOK => U,
        other => other,
    }
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

    #[test]
    fn base_entry_maps_modified_forms() {
        assert_eq!(base_entry(A_CIRC), A);
        assert_eq!(base_entry(A_BREVE), A);
        assert_eq!(base_entry(E_CIRC), E);
        assert_eq!(base_entry(O_HOOK), O);
        assert_eq!(base_entry(O_CIRC), O);
        assert_eq!(base_entry(U_HOOK), U);
        assert_eq!(base_entry(I), I);
    }
}
