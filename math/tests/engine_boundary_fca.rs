//! P11 FCA regression: `MathEngine::process_boundary` / `word_is_command` từng là
//! stub (trả None/false) → `\can` + space, `\mat3x3`, `=2+3=` không bao giờ bung.
//! Test này khóa hành vi đúng sau FCA.

use textvn_math::{AppGuard, MathAction, MathConfig, MathEngine, MathOutputMode};

fn engine_sticky() -> MathEngine {
    let cfg = MathConfig {
        enabled: true,
        sticky_math_mode: true,
        ..MathConfig::default()
    };
    MathEngine::new(cfg)
}

fn type_chars(e: &mut MathEngine, s: &str) -> Option<MathAction> {
    let guard = AppGuard::default();
    let mut last = None;
    for c in s.chars() {
        last = e.process_char(c, 0, &guard);
        if last.is_some() {
            return last;
        }
    }
    last
}

#[test]
fn boundary_expands_vn_alias_can() {
    let mut e = engine_sticky();
    assert!(type_chars(&mut e, "\\can").is_none());
    assert!(e.word_is_command(), "word `\\can` phải là command");
    let guard = AppGuard::default();
    let action = e
        .process_boundary(' ', MathOutputMode::Normal, &guard)
        .expect("boundary phải mở rộng alias \\can");
    match action {
        MathAction::ReplaceBuffer {
            delete_chars,
            text,
            cursor_left_steps,
        } => {
            assert_eq!(delete_chars, 4, "xoá đúng 4 ký tự \\can");
            assert_eq!(cursor_left_steps, 0);
            // Phải khớp lexicon (không hardcode để không lệch khi sửa bảng).
            let entry = textvn_math::lexicon::lookup_token("can").unwrap();
            assert_eq!(text, entry.unicode);
        }
        other => panic!("phải là ReplaceBuffer, nhận được {other:?}"),
    }
    assert!(!e.word_is_command(), "sau boundary word phải clear");
}

#[test]
fn boundary_expands_dynamic_template_mat2x2() {
    let mut e = engine_sticky();
    assert!(type_chars(&mut e, "\\mat2x2").is_none());
    let guard = AppGuard::default();
    let action = e
        .process_boundary(' ', MathOutputMode::Latex, &guard)
        .expect("\\mat2x2 phải bung template");
    match action {
        MathAction::ReplaceBuffer {
            delete_chars, text, ..
        } => {
            assert_eq!(delete_chars, 7);
            assert!(
                text.contains("\\begin{matrix}") || text.contains("matrix"),
                "{text}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn inline_eval_compact_syntax() {
    let mut e = engine_sticky();
    // `=2+3=` — ký tự `=` cuối khép kín phải trả kết quả ngay tại process_char
    let action = type_chars(&mut e, "=2+3=").expect("=2+3= phải eval được");
    match action {
        MathAction::ReplaceBuffer {
            delete_chars, text, ..
        } => {
            assert_eq!(delete_chars, 5);
            assert_eq!(text, "5");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn inline_eval_spaced_syntax() {
    let mut e = engine_sticky();
    let action = type_chars(&mut e, "= 2+3 =").expect("= 2+3 = phải eval được");
    match action {
        MathAction::ReplaceBuffer { text, .. } => assert_eq!(text, "5"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn eval_div_zero_stays_literal() {
    let mut e = engine_sticky();
    // 1/0 → None → không action nào, word được giữ nguyên để boundary dọn
    assert!(type_chars(&mut e, "=1/0=").is_none());
    let guard = AppGuard::default();
    assert!(e
        .process_boundary(' ', MathOutputMode::Normal, &guard)
        .is_none());
}

#[test]
fn unknown_command_passes_through_at_boundary() {
    let mut e = engine_sticky();
    assert!(type_chars(&mut e, "\\khongco").is_none());
    let guard = AppGuard::default();
    assert!(e
        .process_boundary(' ', MathOutputMode::Normal, &guard)
        .is_none());
}

#[test]
fn reset_all_clears_word_buf() {
    let mut e = engine_sticky();
    assert!(type_chars(&mut e, "\\ca").is_none());
    assert!(e.word_is_command());
    e.reset_all();
    assert!(!e.word_is_command());
}
