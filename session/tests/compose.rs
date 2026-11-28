// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 · khối compose: luật P0-6 §5 hàng *compose* cho cả hai chính sách OS. Bộ test TSF đầy đủ
//! (corpus gõ thật qua engine) ở `adapters/windows-tsf/src/compose.rs`; ở đây là luật thuần + các
//! điểm Linux (`lc_plan_key`) khác TSF.

use textvn_core::{ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE};
use textvn_core::{FLAG_CONSUMED, FLAG_WORD_END};
use textvn_session::compose::{
    plan_key, plan_key_with, text_after, ComposePolicy, CompositionPlan, EngineStep, KeyKind,
};

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

fn step(action: u32, del: u16, insert: &str, flags: u32) -> EngineStep {
    EngineStep {
        action,
        delete_count: del,
        insert: chars(insert),
        flags,
        cursor_left: 0,
    }
}

const BOTH: [ComposePolicy; 2] = [ComposePolicy::TSF, ComposePolicy::LINUX];

#[test]
fn pass_char_enters_composition() {
    for p in BOTH {
        let plan = plan_key_with(p, &chars("vi"), KeyKind::Char('e'), &EngineStep::pass());
        assert_eq!(plan.text, Some(chars("vie")));
        assert!(plan.eaten && !plan.end && !plan.reset_engine);
    }
}

#[test]
fn printable_boundary_policy_differs_tsf_commit_with_word_linux_forward() {
    let we = step(ACTION_PASS, 0, "", FLAG_WORD_END);
    let tsf = plan_key_with(ComposePolicy::TSF, &chars("được"), KeyKind::Char(' '), &we);
    assert_eq!(tsf.text, Some(chars("được ")));
    assert!(
        tsf.end && tsf.eaten,
        "TSF/CUAS: commit cùng từ, ăn phím (tránh \" được\")"
    );

    let linux = plan_key_with(
        ComposePolicy::LINUX,
        &chars("được"),
        KeyKind::Char(' '),
        &we,
    );
    assert_eq!(
        linux,
        CompositionPlan::pass_through(true),
        "Linux: commit từ, forward phím"
    );
}

#[test]
fn restore_with_printable_boundary_linux_forwards_key() {
    // `sort` + Space ⇒ RESTORE "sort " (WORD_END): Linux bỏ ' ' khỏi text và forward phím.
    let r = step(ACTION_RESTORE, 4, "sort ", FLAG_WORD_END);
    let linux = plan_key_with(ComposePolicy::LINUX, &chars("sỏt"), KeyKind::Char(' '), &r);
    assert_eq!(linux.delete_before, 1);
    assert_eq!(linux.text, Some(chars("sort")));
    assert!(linux.end && !linux.eaten);
    // TSF giữ ' ' trong text commit, ăn phím.
    let tsf = plan_key_with(ComposePolicy::TSF, &chars("sỏt"), KeyKind::Char(' '), &r);
    assert_eq!(tsf.text, Some(chars("sort ")));
    assert!(tsf.end && tsf.eaten);
}

#[test]
fn enter_tab_return_real_key_without_newline_b2() {
    for p in BOTH {
        for (key, b) in [(KeyKind::Enter, "\n"), (KeyKind::Tab, "\t")] {
            let r = step(ACTION_COMMIT, 0, b, FLAG_WORD_END);
            let plan = plan_key_with(p, &chars("việt"), key, &r);
            assert_eq!(plan.text, Some(chars("việt")), "{p:?} {key:?}");
            assert!(plan.end && !plan.eaten);
        }
    }
}

#[test]
fn replace_macro_enter_keeps_real_enter_but_swallows_space_trigger() {
    for p in BOTH {
        let enter = plan_key_with(
            p,
            &chars("\\can"),
            KeyKind::Enter,
            &step(ACTION_REPLACE, 4, "√\n", FLAG_WORD_END),
        );
        assert_eq!(enter.text, Some(chars("√")));
        assert!(!enter.eaten);
    }
    // Trigger Space của macro bị nuốt ở TSF (REPLACE không trả ranh giới in được).
    let space = plan_key_with(
        ComposePolicy::TSF,
        &chars("ko"),
        KeyKind::Char(' '),
        &step(ACTION_REPLACE, 2, "không ", FLAG_WORD_END),
    );
    assert_eq!(space.text, Some(chars("không ")));
    assert!(space.eaten && space.end);
}

#[test]
fn pass_consumed_is_eaten_silently_e62() {
    for p in BOTH {
        let plan = plan_key_with(
            p,
            &chars("ab"),
            KeyKind::Char('`'),
            &step(ACTION_PASS, 0, "", FLAG_CONSUMED),
        );
        assert!(plan.eaten && plan.end && plan.text.is_none());
    }
}

#[test]
fn ctrl_space_pass_never_keeps_composition_r2_121() {
    for p in BOTH {
        let plan = plan_key_with(p, &chars("ab"), KeyKind::CtrlSpace, &EngineStep::pass());
        assert!(plan.end && !plan.eaten && plan.text.is_none());
        assert_eq!(plan.reset_engine, p.reset_on_ctrl_space_pass);
        assert!(text_after(&chars("ab"), &plan).is_empty());
    }
    const { assert!(!ComposePolicy::TSF.reset_on_ctrl_space_pass) };
    const { assert!(ComposePolicy::LINUX.reset_on_ctrl_space_pass) };
}

#[test]
fn backspace_shrinks_and_ends_when_empty() {
    for p in BOTH {
        let a = plan_key_with(p, &chars("ab"), KeyKind::Backspace, &EngineStep::pass());
        assert_eq!(a.text, Some(chars("a")));
        assert!(!a.end && a.eaten);
        let b = plan_key_with(p, &chars("a"), KeyKind::Backspace, &EngineStep::pass());
        assert!(b.end && b.eaten);
    }
}

#[test]
fn delete_beyond_composition_becomes_delete_before() {
    for p in BOTH {
        let plan = plan_key_with(
            p,
            &chars("c"),
            KeyKind::Char(' '),
            &step(ACTION_REPLACE, 3, "©", 0),
        );
        assert_eq!(plan.delete_before, 2);
        assert_eq!(plan.text, Some(chars("©")));
    }
}

#[test]
fn linux_overflow_commits_and_resets_c5() {
    let full: Vec<char> = std::iter::repeat_n('a', 255).collect();
    let plan = plan_key_with(
        ComposePolicy::LINUX,
        &full,
        KeyKind::Char('b'),
        &EngineStep::pass(),
    );
    assert_eq!(plan.text, Some(full.clone()));
    assert!(plan.end && !plan.eaten && plan.reset_engine);
    // TSF không giới hạn.
    let tsf = plan_key_with(
        ComposePolicy::TSF,
        &full,
        KeyKind::Char('b'),
        &EngineStep::pass(),
    );
    assert_eq!(tsf.text.as_ref().map(Vec::len), Some(256));

    let big = "x".repeat(300);
    let r = plan_key_with(
        ComposePolicy::LINUX,
        &chars("ab"),
        KeyKind::Char(' '),
        &step(ACTION_REPLACE, 0, &big, FLAG_WORD_END),
    );
    assert_eq!(
        r,
        CompositionPlan {
            reset_engine: true,
            ..CompositionPlan::pass_through(true)
        }
    );
}

#[test]
fn unknown_action_is_fail_open() {
    for p in BOTH {
        let plan = plan_key_with(p, &chars("ab"), KeyKind::Char('c'), &step(99, 3, "x", 0));
        assert_eq!(plan, CompositionPlan::pass_through(true));
    }
}

#[test]
fn plan_key_is_tsf_policy() {
    let we = step(ACTION_PASS, 0, "", FLAG_WORD_END);
    assert_eq!(
        plan_key(&chars("ab"), KeyKind::Char('.'), &we),
        plan_key_with(ComposePolicy::TSF, &chars("ab"), KeyKind::Char('.'), &we)
    );
}

#[test]
fn classify_rules() {
    assert_eq!(KeyKind::classify(0x0D, 0), KeyKind::Enter);
    assert_eq!(
        KeyKind::classify(0x70, 0),
        KeyKind::Other,
        "F1 không phải chữ"
    );
    assert_eq!(
        KeyKind::classify(0xE7, 'a' as u32),
        KeyKind::Other,
        "VK_PACKET"
    );
    assert_eq!(KeyKind::classify(0x41, 'a' as u32), KeyKind::Char('a'));
    assert_eq!(
        KeyKind::classify_with_mods(0x20, ' ' as u32, 0x2 | 0x20),
        KeyKind::CtrlSpace
    );
    assert_eq!(
        KeyKind::classify_with_mods(0x45, 'e' as u32, 0x2 | 0x4),
        KeyKind::AltGr('e')
    );
    let s = EngineStep::from_raw(
        ACTION_REPLACE,
        1,
        &['â' as u32, 'x' as u32],
        1,
        FLAG_WORD_END,
        2,
    );
    assert_eq!(s.insert, vec!['â']);
    assert!(s.word_end() && !s.consumed());
}
