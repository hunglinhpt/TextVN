// SPDX-License-Identifier: GPL-3.0-or-later
//! Quick Telex (kế thừa OpenKey): phụ âm đầu gõ đôi → cụm phụ âm tiếng Việt.
//!
//! `cc→ch` · `gg→gi` · `kk→kh` · `nn→ng` · `qq→qu` · `pp→ph` · `tt→th`.
//! Chỉ ở **đầu từ** (phụ âm đầu); `dd→đ` vẫn là luật Telex gốc. Gõ 3 lần lấy lại chữ
//! đôi nguyên văn (`ccc` → `cc`), cùng tinh thần hoàn tác của Telex (`ddd` → `d`).
//! Hoa/thường giữ theo ký tự người dùng gõ (`Cc` → `Ch`, `CC` → `CH`).

fn expansion(c: char) -> Option<char> {
    match c.to_ascii_lowercase() {
        'c' => Some('h'),
        'g' => Some('i'),
        'k' => Some('h'),
        'n' => Some('g'),
        'q' => Some('u'),
        'p' => Some('h'),
        't' => Some('h'),
        _ => None,
    }
}

/// Áp Quick Telex lên chuỗi hiển thị của từ đang gõ (sau fold).
pub fn apply(display: &mut Vec<char>) {
    let _ = apply_with_undo(display);
}

/// Áp Quick Telex và trả về `true` nếu vừa hoàn tác (`ccc` → `cc`, `ppp` → `pp`, …).
pub fn apply_with_undo(display: &mut Vec<char>) -> bool {
    let (Some(&a), Some(&b)) = (display.first(), display.get(1)) else {
        return false;
    };
    if !a.eq_ignore_ascii_case(&b) {
        return false;
    }
    let Some(second) = expansion(a) else {
        return false;
    };
    if display.get(2).is_some_and(|c| c.eq_ignore_ascii_case(&a)) {
        // Hoàn tác: ba chữ giống nhau → giữ đúng hai chữ người dùng muốn.
        display.remove(2);
        return true;
    }
    display[1] = if b.is_ascii_uppercase() {
        second.to_ascii_uppercase()
    } else {
        second
    };
    false
}

#[cfg(test)]
mod tests {
    use super::apply;

    fn q(s: &str) -> String {
        let mut v: Vec<char> = s.chars().collect();
        apply(&mut v);
        v.into_iter().collect()
    }

    #[test]
    fn maps_every_openkey_pair_at_word_start() {
        assert_eq!(q("cc"), "ch");
        assert_eq!(q("gg"), "gi");
        assert_eq!(q("kk"), "kh");
        assert_eq!(q("nn"), "ng");
        assert_eq!(q("qq"), "qu");
        assert_eq!(q("pp"), "ph");
        assert_eq!(q("tt"), "th");
        assert_eq!(q("nnày"), "ngày");
    }

    #[test]
    fn keeps_case_and_supports_undo() {
        assert_eq!(q("Cc"), "Ch");
        assert_eq!(q("TT"), "TH");
        assert_eq!(q("ccc"), "cc");
    }

    #[test]
    fn leaves_other_words_untouched() {
        assert_eq!(q("c"), "c");
        assert_eq!(q("aa"), "aa");
        assert_eq!(q("dd"), "dd");
        assert_eq!(q("coffee"), "coffee");
        assert_eq!(q("ca"), "ca");
    }
}
