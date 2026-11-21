//! E50 Audit: Edge cases for Unikey restore conflict

use textvn_core::{Action, Engine, EngineOptions, KeyEvent};

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

fn type_and_get(keys: &str) -> String {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, keys, &mut buf);
    buf.into_iter().collect()
}

/// Edge case 1: Restore xong, gõ tiếp một từ EN khác + space
/// → Từ mới vẫn phải restore bình thường (flag đã reset)
#[test]
fn e50_restore_then_another_en_word() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // "text " → restore
    type_keys(&mut e, "text ", &mut buf);
    assert_eq!(buf.iter().collect::<String>(), "text ");

    // "file " → vẫn phải restore bình thường (flag đã reset khi bắt đầu word mới)
    buf.clear();
    type_keys(&mut e, "file ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "file ",
        "Sau restore 'text', từ 'file' vẫn phải restore bình thường"
    );
}

/// Edge case 2: Restore xong, gõ tiếp ngay không space
/// → Word mới bắt đầu, flag reset
#[test]
fn e50_restore_then_immediate_typing() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // "text " → restore
    type_keys(&mut e, "text ", &mut buf);

    // Gõ "abc" ngay (không space) → word mới
    type_keys(&mut e, "abc ", &mut buf);
    let result: String = buf.iter().collect();
    // "abc" không có phím dấu Telex, giữ nguyên
    assert!(
        result.contains("abc"),
        "Gõ 'abc' phải có 'abc' trong kết quả: {}",
        result
    );
}

/// Edge case 3: Nhiều từ EN liên tiếp
#[test]
fn e50_multiple_en_words_in_sequence() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    type_keys(&mut e, "text file data system ", &mut buf);
    let result: String = buf.iter().collect();
    assert_eq!(
        result, "text file data system ",
        "Các từ EN liên tiếp phải restore đúng: {}",
        result
    );
}

/// Edge case 4: Từ Việt sau restore
#[test]
fn e50_vietnamese_after_restore() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // "text " → restore
    type_keys(&mut e, "text ", &mut buf);

    // Gõ từ Việt "chaof" → chào (Telex: f = huyền)
    buf.clear();
    type_keys(&mut e, "chaof ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "chào ",
        "Từ Việt sau restore phải fold đúng: {}",
        buf.iter().collect::<String>()
    );
}

/// Edge case 5: Unikey style - gõ 'dd' để được 'đ'
#[test]
fn e50_unikey_dd_after_restore() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // "text " → restore
    type_keys(&mut e, "text ", &mut buf);

    // Gõ "dd" → phải ra "đ" (Unikey style)
    buf.clear();
    type_keys(&mut e, "dd ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "đ ",
        "Gõ 'dd' phải ra 'đ': {}",
        buf.iter().collect::<String>()
    );
}

/// Edge case 6: Không có restore khi không cần
#[test]
fn e50_no_restore_when_not_needed() {
    // Gõ từ không bị transform
    let result = type_and_get("hello ");
    assert_eq!(result, "hello ", "hello không bị transform, giữ nguyên");
}
