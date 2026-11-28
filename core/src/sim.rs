// SPDX-License-Identifier: GPL-3.0-or-later
//! SIM-1 — **một** bộ mô phỏng văn bản dùng chung (G22): áp [`Action`] engine trả về vào buffer
//! như một app thật. Thay 12 bản sao `fn apply` trong `core/tests/*`, test của `lib.rs` và
//! `sim_apply` của `tools/bench`. Thuần, không I/O (luật `core/`).
//!
//! Ngữ nghĩa = HỢP CHẶT NHẤT của các bản sao cũ:
//! - `Pass` + Backspace ⇒ app tự xoá 1 ký tự; Ctrl+Space là hợp âm,
//!   không chèn; ký tự điều khiển bị bỏ trừ `\n`, `\r`, `\t`; ký tự in được nối cuối.
//! - `Replace`/`Restore` ⇒ xoá `delete_count` rồi chèn — xoá vượt phần gõ được (dưới `floor`)
//!   là **lỗi sổ engine** (nhân đôi/mất chữ, lẹm text có sẵn) ⇒ `panic!` có thông điệp, không
//!   che bằng `saturating_sub`.
//! - `Commit` ⇒ chèn.

use crate::keymap::{vk, KeyEvent};
use crate::Action;

/// Áp `action` (engine trả cho phím `key`) vào `buf` — văn bản bắt đầu rỗng.
pub fn apply(buf: &mut Vec<char>, action: &Action, key: &KeyEvent) {
    apply_from(buf, 0, action, key);
}

/// Như [`apply`] nhưng `buf[..floor]` là văn bản CÓ SẴN trước khi gõ: engine không được xoá lấn
/// vào đó (bất biến chống "xoá lẹm" R2-*). Backspace của NGƯỜI DÙNG (qua `Pass`) vẫn xoá được —
/// app thật làm vậy; `floor` chỉ ràng buộc lệnh xoá do ENGINE phát.
pub fn apply_from(buf: &mut Vec<char>, floor: usize, action: &Action, key: &KeyEvent) {
    match action {
        Action::Pass => {
            if key.vk == vk::BACK {
                // Backspace NGƯỜI DÙNG gõ đi qua tới app: app xoá được cả văn bản có sẵn.
                buf.pop();
                return;
            }
            if key.is_ctrl_space() {
                return;
            }
            if let Some(c) = key.printable() {
                if !c.is_control() || matches!(c, '\n' | '\r' | '\t') {
                    buf.push(c);
                }
            }
        }
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            let del = usize::from(*delete_count);
            let typed = buf.len().saturating_sub(floor);
            assert!(
                del <= typed,
                "sim: delete_count {del} vượt phần gõ được {typed} — lệch sổ engine (nhân đôi/mất chữ/xoá lẹm)"
            );
            buf.truncate(buf.len() - del);
            buf.extend_from_slice(insert);
        }
        Action::Commit { insert } => buf.extend_from_slice(insert),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: char) -> KeyEvent {
        KeyEvent::char_down(c)
    }

    #[test]
    fn sim1_pass_backspace_ctrl_space_va_ky_tu_dieu_khien() {
        let mut b: Vec<char> = Vec::new();
        apply(&mut b, &Action::Pass, &key('a'));
        apply(&mut b, &Action::Pass, &key('\n'));
        apply(&mut b, &Action::Pass, &key('\u{1b}'));
        assert_eq!(b, vec!['a', '\n']);
        let back = KeyEvent {
            vk: vk::BACK,
            ch: 8,
            key_down: true,
            ..Default::default()
        };
        apply(&mut b, &Action::Pass, &back);
        apply(&mut b, &Action::Pass, &back);
        apply(&mut b, &Action::Pass, &back);
        assert!(b.is_empty(), "Backspace không xoá dưới 0");
    }

    #[test]
    fn sim1_replace_commit() {
        let mut b: Vec<char> = "tieng".chars().collect();
        apply(
            &mut b,
            &Action::Replace {
                delete_count: 3,
                insert: "ếng".chars().collect(),
            },
            &key('s'),
        );
        assert_eq!(b.iter().collect::<String>(), "tiếng");
        apply(&mut b, &Action::Commit { insert: vec![' '] }, &key(' '));
        assert_eq!(b.iter().collect::<String>(), "tiếng ");
    }

    #[test]
    #[should_panic(expected = "lệch sổ engine")]
    fn sim1_xoa_lem_text_co_san_la_loi() {
        let mut b: Vec<char> = "abc".chars().collect();
        apply_from(
            &mut b,
            2,
            &Action::Replace {
                delete_count: 2,
                insert: vec![],
            },
            &key('x'),
        );
    }
}
