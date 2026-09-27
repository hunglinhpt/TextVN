// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/tables/*.toml · sinh bằng `cargo xtask gen-tables` (P0-1 §3).
// Đổi bảng: sửa file `.toml` rồi chạy lại `cargo xtask gen-tables`.
// `cargo xtask check-tables` (CI) sẽ fail nếu file này lệch với nguồn.
//! Nguồn: `data/tables/{telex,simple_telex,vni,viqr}.toml`
//!   (digest FNV-1a 64 = `0xddfb5dda4593baa6`; bảng âm = `0xc1cc04cde5287ab9`).
//!
//! Mỗi kiểu gõ là 1 `mod`. Hành vi **thuật toán** (undo marker, cụm `uo`, `iet`…)
//! vẫn nằm trong `method/telex.rs`, `vni.rs`, `viqr.rs` — bảng ở đây chỉ mô tả phần bảng.

/// Kiểu gõ **Telex** — xem `data/tables/telex.toml`.
pub mod telex {
    use crate::transform::vowel_table_generated::{
        A, A_BREVE, A_CIRC, E, E_CIRC, O, O_CIRC, O_HOOK, U, U_HOOK,
    };

    /// Key dấu thanh: index 0 → tone 1 (sắc) … index 4 → tone 5 (nặng).
    pub const TONE_KEYS: [char; 5] = ['s', 'f', 'r', 'x', 'j'];
    /// Key dấu thanh → index tone (index 0..=4 → tone 1..=5).
    pub fn tone_of_key(c: char) -> Option<usize> {
        TONE_KEYS.iter().position(|&k| k == c).map(|i| i + 1)
    }
    /// (key, âm gốc, âm đích) — mũ.
    pub const CIRCUMFLEX: [(char, usize, usize); 3] = [
        ('a', A, A_CIRC), // a → â
        ('e', E, E_CIRC), // e → ê
        ('o', O, O_CIRC), // o → ô
    ];
    /// (key, âm gốc, âm đích) — sừng qua marker.
    pub const HORN: [(char, usize, usize); 3] = [
        ('w', A, A_BREVE), // a → ă
        ('w', O, O_HOOK),  // o → ơ
        ('w', U, U_HOOK),  // u → ư
    ];
    /// (key, âm gốc, âm đích) — breve qua marker.
    pub const BREVE: [(char, usize, usize); 0] = [];
    /// Key tạo `đ`. `STROKE_DOUBLE = true` → gõ **hai lần** key (Telex/VIQR `dd`).
    pub const STROKE_KEY: char = 'd';
    pub const STROKE_DOUBLE: bool = true;
    /// Kiểu gõ này không có phím xoá toàn bộ dấu.
    pub const REMOVE_MARKS_KEY: Option<char> = None;
    /// `true` = `w` là marker sừng (Telex); `false` = `w` là chữ thường (Simple Telex).
    pub const W_MARKER: bool = true;
    /// Key này có phải **marker** (một phần của từ, không phải ranh giới)?
    pub fn is_marker(c: char) -> bool {
        tone_of_key(c).is_some()
            || CIRCUMFLEX.iter().any(|&(k, _, _)| k == c)
            || HORN.iter().any(|&(k, _, _)| k == c)
            || BREVE.iter().any(|&(k, _, _)| k == c)
            || REMOVE_MARKS_KEY == Some(c)
            // `stroke` 1 phím (VNI `9`) là marker; `dd` thì không (chữ cái vốn
            // đã thuộc từ) — nên chỉ nhận khi STROKE_DOUBLE = false.
            || (!STROKE_DOUBLE && STROKE_KEY == c)
            || (W_MARKER && c == 'w')
    }
}

/// Kiểu gõ **Simple Telex** — xem `data/tables/simple_telex.toml`.
pub mod simple_telex {
    use crate::transform::vowel_table_generated::{A, A_CIRC, E, E_CIRC, O, O_CIRC};

    /// Key dấu thanh: index 0 → tone 1 (sắc) … index 4 → tone 5 (nặng).
    pub const TONE_KEYS: [char; 5] = ['s', 'f', 'r', 'x', 'j'];
    /// Key dấu thanh → index tone (index 0..=4 → tone 1..=5).
    pub fn tone_of_key(c: char) -> Option<usize> {
        TONE_KEYS.iter().position(|&k| k == c).map(|i| i + 1)
    }
    /// (key, âm gốc, âm đích) — mũ.
    pub const CIRCUMFLEX: [(char, usize, usize); 3] = [
        ('a', A, A_CIRC), // a → â
        ('e', E, E_CIRC), // e → ê
        ('o', O, O_CIRC), // o → ô
    ];
    /// (key, âm gốc, âm đích) — sừng qua marker.
    pub const HORN: [(char, usize, usize); 0] = [];
    /// (key, âm gốc, âm đích) — breve qua marker.
    pub const BREVE: [(char, usize, usize); 0] = [];
    /// Key tạo `đ`. `STROKE_DOUBLE = true` → gõ **hai lần** key (Telex/VIQR `dd`).
    pub const STROKE_KEY: char = 'd';
    pub const STROKE_DOUBLE: bool = true;
    /// Kiểu gõ này không có phím xoá toàn bộ dấu.
    pub const REMOVE_MARKS_KEY: Option<char> = None;
    /// `true` = `w` là marker sừng (Telex); `false` = `w` là chữ thường (Simple Telex).
    pub const W_MARKER: bool = false;
    /// Key này có phải **marker** (một phần của từ, không phải ranh giới)?
    pub fn is_marker(c: char) -> bool {
        tone_of_key(c).is_some()
            || CIRCUMFLEX.iter().any(|&(k, _, _)| k == c)
            || HORN.iter().any(|&(k, _, _)| k == c)
            || BREVE.iter().any(|&(k, _, _)| k == c)
            || REMOVE_MARKS_KEY == Some(c)
            // `stroke` 1 phím (VNI `9`) là marker; `dd` thì không (chữ cái vốn
            // đã thuộc từ) — nên chỉ nhận khi STROKE_DOUBLE = false.
            || (!STROKE_DOUBLE && STROKE_KEY == c)
            || (W_MARKER && c == 'w')
    }
}

/// Kiểu gõ **VNI** — xem `data/tables/vni.toml`.
pub mod vni {
    use crate::transform::vowel_table_generated::{
        A, A_BREVE, A_CIRC, E, E_CIRC, O, O_CIRC, O_HOOK, U, U_HOOK,
    };

    /// Key dấu thanh: index 0 → tone 1 (sắc) … index 4 → tone 5 (nặng).
    pub const TONE_KEYS: [char; 5] = ['1', '2', '3', '4', '5'];
    /// Key dấu thanh → index tone (index 0..=4 → tone 1..=5).
    pub fn tone_of_key(c: char) -> Option<usize> {
        TONE_KEYS.iter().position(|&k| k == c).map(|i| i + 1)
    }
    /// (key, âm gốc, âm đích) — mũ.
    pub const CIRCUMFLEX: [(char, usize, usize); 3] = [
        ('6', A, A_CIRC), // a → â
        ('6', E, E_CIRC), // e → ê
        ('6', O, O_CIRC), // o → ô
    ];
    /// (key, âm gốc, âm đích) — sừng qua marker.
    pub const HORN: [(char, usize, usize); 2] = [
        ('7', O, O_HOOK), // o → ơ
        ('7', U, U_HOOK), // u → ư
    ];
    /// (key, âm gốc, âm đích) — breve qua marker.
    pub const BREVE: [(char, usize, usize); 1] = [
        ('8', A, A_BREVE), // a → ă
    ];
    /// Key tạo `đ`. `STROKE_DOUBLE = true` → gõ **hai lần** key (Telex/VIQR `dd`).
    pub const STROKE_KEY: char = '9';
    pub const STROKE_DOUBLE: bool = false;
    /// Key xoá toàn bộ dấu của từ (VNI `0`).
    pub const REMOVE_MARKS_KEY: Option<char> = Some('0');
    /// `true` = `w` là marker sừng (Telex); `false` = `w` là chữ thường (Simple Telex).
    pub const W_MARKER: bool = false;
    /// Key này có phải **marker** (một phần của từ, không phải ranh giới)?
    pub fn is_marker(c: char) -> bool {
        tone_of_key(c).is_some()
            || CIRCUMFLEX.iter().any(|&(k, _, _)| k == c)
            || HORN.iter().any(|&(k, _, _)| k == c)
            || BREVE.iter().any(|&(k, _, _)| k == c)
            || REMOVE_MARKS_KEY == Some(c)
            // `stroke` 1 phím (VNI `9`) là marker; `dd` thì không (chữ cái vốn
            // đã thuộc từ) — nên chỉ nhận khi STROKE_DOUBLE = false.
            || (!STROKE_DOUBLE && STROKE_KEY == c)
            || (W_MARKER && c == 'w')
    }
}

/// Kiểu gõ **VIQR** — xem `data/tables/viqr.toml`.
pub mod viqr {
    use crate::transform::vowel_table_generated::{
        A, A_BREVE, A_CIRC, E, E_CIRC, O, O_CIRC, O_HOOK, U, U_HOOK,
    };

    /// Key dấu thanh: index 0 → tone 1 (sắc) … index 4 → tone 5 (nặng).
    pub const TONE_KEYS: [char; 5] = ['\'', '`', '?', '~', '.'];
    /// Key dấu thanh → index tone (index 0..=4 → tone 1..=5).
    pub fn tone_of_key(c: char) -> Option<usize> {
        TONE_KEYS.iter().position(|&k| k == c).map(|i| i + 1)
    }
    /// (key, âm gốc, âm đích) — mũ.
    pub const CIRCUMFLEX: [(char, usize, usize); 3] = [
        ('^', A, A_CIRC), // a → â
        ('^', E, E_CIRC), // e → ê
        ('^', O, O_CIRC), // o → ô
    ];
    /// (key, âm gốc, âm đích) — sừng qua marker.
    pub const HORN: [(char, usize, usize); 2] = [
        ('+', O, O_HOOK), // o → ơ
        ('+', U, U_HOOK), // u → ư
    ];
    /// (key, âm gốc, âm đích) — breve qua marker.
    pub const BREVE: [(char, usize, usize); 1] = [
        ('(', A, A_BREVE), // a → ă
    ];
    /// Key tạo `đ`. `STROKE_DOUBLE = true` → gõ **hai lần** key (Telex/VIQR `dd`).
    pub const STROKE_KEY: char = 'd';
    pub const STROKE_DOUBLE: bool = true;
    /// Kiểu gõ này không có phím xoá toàn bộ dấu.
    pub const REMOVE_MARKS_KEY: Option<char> = None;
    /// `true` = `w` là marker sừng (Telex); `false` = `w` là chữ thường (Simple Telex).
    pub const W_MARKER: bool = false;
    /// Key này có phải **marker** (một phần của từ, không phải ranh giới)?
    pub fn is_marker(c: char) -> bool {
        tone_of_key(c).is_some()
            || CIRCUMFLEX.iter().any(|&(k, _, _)| k == c)
            || HORN.iter().any(|&(k, _, _)| k == c)
            || BREVE.iter().any(|&(k, _, _)| k == c)
            || REMOVE_MARKS_KEY == Some(c)
            // `stroke` 1 phím (VNI `9`) là marker; `dd` thì không (chữ cái vốn
            // đã thuộc từ) — nên chỉ nhận khi STROKE_DOUBLE = false.
            || (!STROKE_DOUBLE && STROKE_KEY == c)
            || (W_MARKER && c == 'w')
    }
}
