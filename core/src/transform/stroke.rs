// SPDX-License-Identifier: GPL-3.0-or-later
//! Ký tự đ/Đ (P0-1 §1 `transform/stroke.rs`). Quy tắc gõ: `dd` → đ, `d` + âm viết → đ.

pub fn is_plain_d(c: char) -> bool {
    c == 'd' || c == 'D'
}

pub fn is_stroke(c: char) -> bool {
    c == 'đ' || c == 'Đ'
}

/// d → đ (giữ case).
pub fn to_stroke(c: char) -> char {
    match c {
        'd' => 'đ',
        'D' => 'Đ',
        other => other,
    }
}

/// đ → d (undo).
pub fn to_plain(c: char) -> char {
    match c {
        'đ' => 'd',
        'Đ' => 'D',
        other => other,
    }
}
