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
///
/// Đường nóng của engine (gọi cho mọi ký tự của từ ở mỗi phím): phụ âm ASCII loại ngay,
/// nguyên âm ASCII tra trực tiếp, ký tự có dấu tra bảng chỉ mục trực tiếp theo code point
/// (mọi dạng có dấu nằm trong U+00C0..U+1EFF) dựng một lần — bản cũ quét tuyến tính
/// 72 dạng + `to_lowercase` cho từng ký tự.
pub fn locate(c: char) -> Option<(usize, usize)> {
    if c.is_ascii() {
        return match c.to_ascii_lowercase() {
            'a' => Some((A, 0)),
            'e' => Some((E, 0)),
            'i' => Some((I, 0)),
            'o' => Some((O, 0)),
            'u' => Some((U, 0)),
            'y' => Some((Y, 0)),
            _ => None,
        };
    }
    let idx = (c as usize).checked_sub(MARKED_FIRST)?;
    let packed = *marked_forms().get(idx)?;
    (packed != 0).then(|| {
        let v = usize::from(packed - 1);
        (v / TONES_PER_VOWEL, v % TONES_PER_VOWEL)
    })
}

/// Code point đầu của vùng chứa mọi dạng có dấu (`À`).
const MARKED_FIRST: usize = 0xC0;

/// `table[c - MARKED_FIRST]` = `entry * 6 + tone + 1` cho mọi dạng không phải ASCII
/// (thường và HOA), `0` = không phải âm.
fn marked_forms() -> &'static [u16] {
    static TABLE: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut forms = Vec::with_capacity(VOWEL_ENTRY_COUNT * TONES_PER_VOWEL * 2);
        for (e, entry) in VOWELS.iter().enumerate() {
            for (t, &f) in entry.forms.iter().enumerate() {
                if f.is_ascii() {
                    continue;
                }
                forms.push((f, e, t));
                for up in f.to_uppercase() {
                    if up != f {
                        forms.push((up, e, t));
                    }
                }
            }
        }
        let last = forms
            .iter()
            .map(|&(ch, _, _)| ch as usize)
            .max()
            .unwrap_or(0);
        let mut table = vec![0u16; last + 1 - MARKED_FIRST];
        for (ch, e, t) in forms {
            let slot = &mut table[ch as usize - MARKED_FIRST];
            // Trùng ký tự: giữ dạng gặp trước (thứ tự bảng — như cách quét tuyến tính).
            if *slot == 0 {
                *slot = (e * TONES_PER_VOWEL + t + 1) as u16;
            }
        }
        table
    })
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

    /// Tra nhanh phải cho đúng kết quả như cách quét tuyến tính cũ, với mọi ký tự BMP.
    #[test]
    fn fast_locate_matches_linear_scan() {
        // (Bỏ qua ký tự hạ thành NHIỀU ký tự như `İ` U+0130 → "i̇": bản cũ lấy ký tự đầu nên
        // nhận nhầm là `i`; bản mới trả None — đúng hơn.)
        let slow = |c: char| -> Option<(usize, usize)> {
            if c.to_lowercase().count() != 1 {
                return None;
            }
            let lc = if c.is_uppercase() {
                c.to_lowercase().next()?
            } else {
                c
            };
            VOWELS
                .iter()
                .enumerate()
                .find_map(|(e, entry)| entry.forms.iter().position(|&f| f == lc).map(|t| (e, t)))
        };
        for cp in 0u32..0x2000 {
            if let Some(c) = char::from_u32(cp) {
                assert_eq!(locate(c), slow(c), "U+{cp:04X}");
            }
        }
        for cp in 0x1E00u32..0x1F00 {
            let c = char::from_u32(cp).unwrap();
            assert_eq!(locate(c), slow(c), "U+{cp:04X}");
        }
    }

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
