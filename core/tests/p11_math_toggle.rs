//! P11 — Math & Science toggle từ UI: audit hành vi bật/tắt.
//!
//! 1. Mặc định `math.enabled = false`: gõ Việt/Anh/punctuation y hệt engine cũ
//!    (regression guard — không một phím nào bị math nuốt/sửa).
//! 2. Khi bật math nhưng KHÔNG ở trong math span/sticky: mọi phím printable vẫn
//!    pass-through và `keys_typed` chỉ đếm đúng 1 lần/phím (G19 — chống
//!    double-count khi math_key trả None rồi pipeline chính đếm tiếp).
//! 3. Tắt math qua `set_options` (UI/tray toggle OFF) phải dọn sạch FSM chord
//!    (armed leader) để pass-through 100% ngay lập tức.

use textvn_core::{Action, Engine, EngineOptions, KeyEvent, FLAG_CONSUMED};

fn type_keys(e: &mut Engine, keys: &str, buf: &mut Vec<char>) {
    for c in keys.chars() {
        let k = KeyEvent::char_down(c);
        let outcome = e.key(&k);
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
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
}

fn type_and_get_with(e: &mut Engine, keys: &str) -> String {
    let mut buf: Vec<char> = Vec::new();
    type_keys(e, keys, &mut buf);
    buf.into_iter().collect()
}

fn opts_with_math(enabled: bool) -> EngineOptions {
    let mut o = EngineOptions::default();
    o.math.enabled = enabled;
    o
}

/// 1a. Mặc định TẮT: chuỗi hỗn hợp VI/EN/dấu câu phải y hệt hành vi cũ.
#[test]
fn math_default_off_normal_typing_unchanged() {
    let mut e = Engine::new(EngineOptions::default());
    assert!(
        !e.options().math.enabled,
        "default phải là TẮT (audit default OFF)"
    );
    // Telex fold bình thường + mọi ký tự đặc biệt (` \ = ^ _ / < >) đi thẳng
    assert_eq!(type_and_get_with(&mut e, "chaof "), "chào ");
    assert_eq!(type_and_get_with(&mut e, "dd "), "đ ");
    assert_eq!(type_and_get_with(&mut e, "TextVN "), "TextVN ");
    let s = type_and_get_with(&mut e, "hello `backtick` \\slash =^_/<>. ");
    assert_eq!(s, "hello `backtick` \\slash =^_/<>. ");
}

/// 1b. Unikey dual-mode vẫn đúng khi math TẮT: "dd" -> "đ", "aa" -> "â".
#[test]
fn math_default_off_unikey_style_unchanged() {
    let mut e = Engine::new(EngineOptions::default());
    assert_eq!(type_and_get_with(&mut e, "dd "), "đ ");
    // "aa" có trong từ điển EN → được khôi phục nguyên văn (hành vi restore EN)
    assert_eq!(type_and_get_with(&mut e, "aa "), "aa ");
    assert_eq!(type_and_get_with(&mut e, "chaof "), "chào ");
    assert_eq!(type_and_get_with(&mut e, "TextVN "), "TextVN ");
}

/// 2. Math BẬT nhưng không sticky/không trong $...$: pass-through thuần,
///    keys_typed đếm đúng 1 lần mỗi phím (không double-count).
#[test]
fn math_enabled_passthrough_counts_keys_typed_once() {
    let mut e = Engine::new(opts_with_math(true));
    let before = e.dream_rsi_snapshot().keys_typed;
    let s = type_and_get_with(&mut e, "hello world 123 `~^ ");
    // Không ở trong math span, không sticky → toàn bộ pass-through
    assert_eq!(s, "hello world 123 `~^ ");
    let after = e.dream_rsi_snapshot().keys_typed;
    assert_eq!(
        after - before,
        "hello world 123 `~^ ".chars().count() as u64,
        "mỗi phím printable phải đếm đúng 1 lần keys_typed (G19)"
    );
}

/// 3. Bật math + sticky, arm leader (`) rồi TẮT math qua set_options:
///    chord FSM phải reset — phím tiếp theo đi thẳng, không bị nuốt.
#[test]
fn math_toggle_off_resets_armed_chord() {
    let mut o = opts_with_math(true);
    o.math.sticky_math_mode = true;
    let mut e = Engine::new(o);

    // Arm leader: gõ ` (backtick) → chord FSM armed, phím bị nuốt (Consume)
    let out = e.key(&KeyEvent::char_down('`'));
    assert!(
        out.flags & FLAG_CONSUMED != 0,
        "leader phải bị nuốt khi sticky math đang bật"
    );

    // Tắt math qua set_options (đúng đường UI/tray toggle OFF)
    let mut off = opts_with_math(false);
    off.math.sticky_math_mode = true; // sticky giữ nguyên, chỉ tắt master
    e.set_options(off);

    // Phím tiếp theo phải pass-through 100%, không còn armed
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, "a", &mut buf);
    assert_eq!(buf.iter().collect::<String>(), "a");
    let s = type_and_get_with(&mut e, "bc `x");
    assert_eq!(s, "bc `x", "sau khi tắt math, backtick là ký tự thường");
}

/// 4. Bật math trong math span ($...$): Telex bị treo (pass-through),
///    và keys_typed vẫn đếm đúng 1 lần/phím.
#[test]
fn math_span_suspends_telex_and_counts_once() {
    let mut e = Engine::new(opts_with_math(true));
    let before = e.dream_rsi_snapshot().keys_typed;
    // "$aa" → trong math span, 'a','a' pass thẳng (Telex treo), không thành "â"
    let s = type_and_get_with(&mut e, "$aa");
    assert_eq!(s, "$aa");
    let after = e.dream_rsi_snapshot().keys_typed;
    assert_eq!(after - before, 3, "3 phím ($ a a) đếm đúng 1 lần mỗi phím");
}
