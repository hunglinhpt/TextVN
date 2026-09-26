// SPDX-License-Identifier: GPL-3.0-or-later
//! Dấu thanh: key Telex → tone index, áp/dỡ dấu trên một ký tự (P0-1 §1 `transform/tone.rs`).

use super::vowel_table::{form_like, locate};

/// Key Telex lowercase → index tone (1..=5). `None` nếu không phải key dấu.
pub fn key_to_tone(c: char) -> Option<usize> {
    match c {
        's' => Some(1), // sắc
        'f' => Some(2), // huyền
        'r' => Some(3), // hỏi
        'x' => Some(4), // ngã
        'j' => Some(5), // nặng
        _ => None,
    }
}

/// Char biểu diễn dấu khi phải gõ literal (undo) — đúng key đã gõ.
pub fn tone_to_key(tone: usize) -> char {
    match tone {
        1 => 's',
        2 => 'f',
        3 => 'r',
        4 => 'x',
        5 => 'j',
        _ => ' ',
    }
}

/// Áp dấu `tone` lên ký tự âm (thay dấu hiện có nếu có — `free_marking`).
pub fn apply_tone(c: char, tone: usize) -> char {
    match locate(c) {
        Some((e, _)) => form_like(c, e, tone),
        None => c,
    }
}

/// Gỡ dấu của ký tự âm → về dạng không dấu (giữ case). Không phải âm → giữ nguyên.
pub fn strip_tone(c: char) -> char {
    match locate(c) {
        Some((e, t)) if t > 0 => form_like(c, e, 0),
        _ => c,
    }
}

/// Tone hiện tại của ký tự âm: 0 = không dấu, 1..=5 = có dấu. Không phải âm → `None`.
pub fn tone_of(c: char) -> Option<usize> {
    locate(c).map(|(_, t)| t)
}

/// Ký tự có phải âm (mọi dạng) không.
pub fn is_vowel(c: char) -> bool {
    locate(c).is_some()
}
