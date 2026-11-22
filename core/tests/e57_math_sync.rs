//! E57 — hồi quy tích hợp core của Math Engine (audit 0.2.79..0.2.84):
//! a) Backspace/arrow đồng bộ word math trong core (word_buf không lệch document)
//! b) inline eval `=expr=`: delete không tính `=` đóng → không xoa lấn ký tự trước
//! c) Telex treo trong inline-eval word: `=sin(0)=` ra đúng "0"

use textvn_core::{keymap, Action, Engine, EngineOptions, KeyEvent};

fn opts_sticky_math() -> EngineOptions {
    let mut o = EngineOptions::default();
    o.math.enabled = true;
    o.math.sticky_math_mode = true;
    o
}

fn backspace() -> KeyEvent {
    KeyEvent::key_down(keymap::vk::BACK)
}

fn left() -> KeyEvent {
    KeyEvent::key_down(keymap::vk::LEFT)
}

fn type_keys(e: &mut Engine, keys: &str, buf: &mut Vec<char>) {
    for c in keys.chars() {
        let outcome = e.key(&KeyEvent::char_down(c));
        match outcome.action {
            Action::Pass => buf.push(c),
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len().saturating_sub(delete_count as usize));
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
}

fn buf_str(e: &mut Engine, keys: &str) -> String {
    let mut buf = Vec::new();
    type_keys(e, keys, &mut buf);
    buf.into_iter().collect()
}

/// E57a: Backspace giữa lệnh math — word math đồng bộ, boundary sau đó không
/// bung lệnh trên word đã bị người dùng xóa bỏ.
#[test]
fn e57a_backspace_syncs_math_word() {
    let mut e = Engine::new(opts_sticky_math());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, "\\mat3x3", &mut buf);
    // Abort = người dùng backspace 4 ký tự rồi space: doc phải còn "\\mat "
    for _ in 0..4 {
        e.key(&backspace());
        buf.pop();
    }
    type_keys(&mut e, " ", &mut buf);
    let s: String = buf.into_iter().collect();
    assert_eq!(
        s, "\\ma ",
        "BS rồi space KHÔNG được bung template (word đã bị xóa)"
    );
}

/// E57a(đối chứng): KHÔNG backspace thì template vẫn bung đúng.
#[test]
fn e57a_template_still_expands_without_backspace() {
    let mut e = Engine::new(opts_sticky_math());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, "\\mat2x2", &mut buf);
    type_keys(&mut e, " ", &mut buf);
    let s: String = buf.into_iter().collect();
    assert!(
        s.contains("a11") && s.contains("a22"),
        "template \\mat2x2 phải bung: {s:?}"
    );
}

/// E57a: phím mũi tên làm caret rời word — math bỏ word, không bung lệnh cũ.
#[test]
fn e57a_arrow_clears_math_word() {
    let mut e = Engine::new(opts_sticky_math());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, "\\can", &mut buf);
    e.key(&left());
    type_keys(&mut e, "x ", &mut buf);
    let s: String = buf.into_iter().collect();
    assert_eq!(
        s, "\\canx ",
        "sau arrow phải coi word đã rời — không bung `\\can`"
    );
}

/// E57b: `a =2+3=` phải ra `a 5` — word eval mở SAU space (`=` đầu word),
/// `=` đóng bị consume, delete đúng 4 ký tự đã commit; sai lệch cũ (delete 5)
/// sẽ nuốt mất dấu space (hoặc chữ trước đó) của người dùng.
#[test]
fn e57b_inline_eval_no_delete_overflow() {
    let mut e = Engine::new(opts_sticky_math());
    assert_eq!(buf_str(&mut e, "a =2+3="), "a 5");
    assert_eq!(buf_str(&mut e, "=2+3="), "5");
}

/// E57c: chữ trong inline-eval không bị Telex fold trong sticky mode.
#[test]
fn e57c_telex_suspended_inside_eval_word() {
    let mut e = Engine::new(opts_sticky_math());
    // `sin(0)` có cặp `i`,`n`,`s` mà nếu Telex chạy thì `s` căng dấu sắc.
    assert_eq!(buf_str(&mut e, "=sin(0)="), "0");
}
