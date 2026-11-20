//! E50: Deep audit & fix bug "đang định gõ âm tiết thành đất, nó ra đata, space cái thì ra ddat"
//! "Task space ra Tassk"
//!
//! Requirement:
//! - Nếu đã restore theo rule mới → skip rule Unikey
//! - Nếu đã tiếp tục gõ thêm ký tự theo style Unikey → check lại từ đúng, ko restore thêm
//!
//! Issue: Sau auto-restore, nếu người dùng gõ tiếp, không nên restore thêm
//! (skip rule Unikey) để phục vụ thói quen người dùng Unikey lâu năm.

use textvn_core::{Action, Engine, EngineOptions, KeyEvent};

/// Helper: gõ một chuỗi qua engine
fn type_keys(e: &mut Engine, keys: &str, buf: &mut Vec<char>) {
    for c in keys.chars() {
        let k = KeyEvent::char_down(c);
        let outcome = e.key(&k);
        match outcome.action {
            Action::Pass => buf.push(c),
            Action::Replace {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Restore {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
}

/// Helper: gõ và nhận kết quả
fn type_and_get(keys: &str) -> String {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, keys, &mut buf);
    buf.into_iter().collect()
}

#[test]
fn e50_unikey_task_space_no_restore() {
    // User báo: "Task space ra Tassk"
    // Gõ "Task " phải ra "Task " (không restore "Tassk")
    let result = type_and_get("Task ");
    assert_eq!(
        result, "Task ",
        "Task + space phải ra 'Task ', nhưng ra '{}'",
        result
    );
}

#[test]
fn e50_dat_space_no_duplicate() {
    // User báo: "nó ra đata, space cái thì ra ddat"
    // Gõ "dat " phải ra "dat " (không restore)
    let result = type_and_get("dat ");
    assert_eq!(
        result, "dat ",
        "dat + space phải ra 'dat ', nhưng ra '{}'",
        result
    );
}

#[test]
fn e50_daat_space_no_duplicate() {
    // Gõ "daat " phải ra "daat " (không có biến đổi Telex, không restore)
    // Theo Telex: 'daat' không có phím dấu nào, giữ nguyên
    let result = type_and_get("daat ");
    assert_eq!(
        result, "daat ",
        "daat + space phải ra 'daat ', nhưng ra '{}'",
        result
    );
}

/// Test E50 core: sau auto-restore, gõ tiếp sẽ skip restore
#[test]
fn e50_after_restore_skip_further_restore() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // Gõ "text " → auto-restore (text → tẽt, restore → text)
    type_keys(&mut e, "text ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "text ",
        "text + space phải restore ra 'text '"
    );

    // Tiếp tục gõ "Task " → KHÔNG restore (skip rule Unikey)
    buf.clear();
    type_keys(&mut e, "Task ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "Task ",
        "Sau restore, Task + space phải ra 'Task ' (skip rule Unikey)"
    );
}
