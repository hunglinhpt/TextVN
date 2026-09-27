// SPDX-License-Identifier: GPL-3.0-or-later
//! Dấu thanh: key Telex → tone index, áp/dỡ dấu trên một ký tự (P0-1 §1 `transform/tone.rs`).

use super::diacritic_style::{pick_tone_target, DiacriticStyle};
use super::undo::unmark;
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

/// Áp **một key dấu thanh** vào `out` — dùng chung cho Telex (`s f r x j`),
/// VNI (`1`..`5`) và VIQR (`' \` ? ~ .`).
///
/// - `free_marking = false` (Telex chặt): key phải nằm **ngay sau** âm, nếu không → literal.
/// - Bấm lại **đúng** dấu → gỡ dấu + gõ literal `key` (`ass` → `as`).
/// - Chưa có âm nào → `key` là chữ thường (`sun`).
/// - `key` = chính ký tự người dùng gõ (để gõ literal khi undo/literal).
pub fn apply_key(out: &mut Vec<char>, tone: usize, key: char, style: DiacriticStyle, free: bool) {
    if !free && !out.last().map(|&l| is_vowel(l)).unwrap_or(false) {
        out.push(key);
        return;
    }
    let Some(tidx) = pick_tone_target(out, style) else {
        out.push(key); // chưa có âm nào → đây là chữ thường
        return;
    };
    let cur = tone_of(out[tidx]).unwrap_or(0);
    if cur == tone {
        // Bấm lại đúng dấu → bỏ dấu + gõ literal (ass → as)
        out[tidx] = unmark(out[tidx]);
        out.push(key);
    } else {
        // Áp / thay dấu (free_marking)
        out[tidx] = apply_tone(out[tidx], tone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn apply_key_tone_and_undo() {
        let mut out = chars("duoc");
        apply_key(&mut out, 1, 's', DiacriticStyle::New, true);
        assert_eq!(out.iter().collect::<String>(), "duóc");

        let mut out = chars("duoc");
        apply_key(&mut out, 2, '2', DiacriticStyle::New, true);
        assert_eq!(out.iter().collect::<String>(), "duòc"); // VNI: key là số

        let mut out = chars("á");
        apply_key(&mut out, 1, 's', DiacriticStyle::New, true);
        assert_eq!(out.iter().collect::<String>(), "as"); // bấm lại đúng dấu
    }

    #[test]
    fn apply_key_literal_when_no_vowel() {
        let mut out = chars("sun");
        apply_key(&mut out, 1, 's', DiacriticStyle::New, false); // strict: 's' không sau âm
        assert_eq!(out.iter().collect::<String>(), "suns");
    }
}
