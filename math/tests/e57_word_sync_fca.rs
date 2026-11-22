//! E57 FCA — hổi quy các vết sai của MathEngine facade:
//! a) Backspace/caret không đồng bộ word_buf (lech document xoa lan text nguoi dung)
//! b)-inline eval delete_chars đếm `=` đóng chưa-commit (off-by-one theo document)
//! c) Telex không treo trong inline-eval word (`=sin(0)=` bị fold)

use textvn_math::config::MathOutputMode;
use textvn_math::{ChordState, MathConfig, MathEngine};

fn engine_sticky() -> MathEngine {
    let cfg = MathConfig {
        enabled: true,
        sticky_math_mode: true,
        ..MathConfig::default()
    };
    MathEngine::new(cfg)
}

/// E57a: Backspace phải pop ký tự khỏi word_buf.
#[test]
fn backspace_pops_word_buf() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    for c in "\\mat3x3".chars() {
        e.process_char(c, 0, guard);
    }
    // xóa 4 ký tự: "3x3" → word còn "\mat"
    for _ in 0..4 {
        e.process_backspace();
    }
    e.process_char('3', 0, guard);
    // boundary space không được bung template (word = "\mat3" — KHÔNG phải lệnh
    // template; trước đây word_buf lệch là "\mat3x33" → xóa lấn document).
    let out = e.process_boundary(' ', MathOutputMode::Latex, guard);
    assert!(out.is_none(), " Boundary phải NONE với word chưa khớp");
}

/// E57a: Backspace hủy chord đang armed (zero stuck-state).
#[test]
fn backspace_resets_armed_chord() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    let cfg = e.config.clone();
    let leader = cfg.leader_key;
    e.process_char(leader, 0, guard);
    assert_eq!(e.chord_fsm.state(), ChordState::Armed);
    e.process_backspace();
    assert_ne!(e.chord_fsm.state(), ChordState::Armed);
}

/// E57a: caret_moved xoá sạch word.
#[test]
fn caret_moved_clears_word() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    for c in "\\can".chars() {
        e.process_char(c, 0, guard);
    }
    e.caret_moved();
    assert!(!e.word_is_command());
    assert!(e
        .process_boundary(' ', MathOutputMode::Latex, guard)
        .is_none());
}

/// E57b: inline eval delete_chars KHÔNG tính `=` đóng (keystroke bị consume,
/// chưa vào document) — "=2+3=" phải đếm 4 chứ không phải 5.
#[test]
fn inline_eval_delete_excludes_closing_eq() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    let mut last = None;
    for c in "=2+3=".chars() {
        last = e.process_char(c, 0, guard);
    }
    match last.expect("eval phải bung ở `=` đóng") {
        textvn_math::MathAction::ReplaceBuffer {
            delete_chars, text, ..
        } => {
            assert_eq!(delete_chars, 4, "`=2+3` đã commit = 4 ký tự");
            assert_eq!(text, "5");
        }
        _ => panic!("phải là ReplaceBuffer"),
    }
}

/// E57c: word_is_command đúng cho cả lệnh `\...` lẫn inline-eval `=expr=`.
#[test]
fn word_is_command_includes_inline_eval() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    e.process_char('=', 0, guard);
    assert!(
        e.word_is_command(),
        "word `=` mở eval phải được coi là command"
    );
    e.process_char('2', 0, guard);
    assert!(e.word_is_command());
}

/// E57d: boundary chars (`)`, `,`, `.`...) GIỮA eval span không bị rơi khỏi
/// word_buf — `=sin(0)=` trước đây mất `)` → `eval("sin(0")` = None → không bung.
#[test]
fn eval_span_keeps_boundary_chars() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    let mut fired = None;
    for c in "=sin(0)=".chars() {
        if let Some(a) = e.process_char(c, 0, guard) {
            fired = Some(a);
        }
    }
    match fired.expect("`=sin(0)=` phải bung kết quả") {
        textvn_math::MathAction::ReplaceBuffer {
            delete_chars, text, ..
        } => {
            assert_eq!(delete_chars, 7, "`=sin(0` đã commit = 7 ký tự");
            assert_eq!(text, "0");
        }
        _ => panic!("phải là ReplaceBuffer"),
    }
}

/// E57e: eval KHÔNG hợp lệ (`=max(1,2)=`) — span bị chốt, word KHÔNG kẹt:
/// Telex không bị treo vĩnh viễn, eval kế tiếp (`=2*3=`) vẫn mở word mới.
#[test]
fn failed_eval_does_not_stick() {
    let mut e = engine_sticky();
    let g = textvn_math::AppGuard::default();
    let guard = &g;
    for c in "=max(1,2)=".chars() {
        let _ = e.process_char(c, 0, guard);
    }
    assert!(!e.word_is_command(), "eval fail phải chốt span — word rỗng");
    let mut fired2 = false;
    for c in "=2*3=".chars() {
        if e.process_char(c, 0, guard).is_some() {
            fired2 = true;
        }
    }
    assert!(fired2, "eval kế tiếp vẫn phải bung được");
}
