// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-3 · khối reset (P0-6 §5, R2-171/E78): reset của toolkit ngay sau Enter giữ mốc đầu dòng;
//! mọi reset khác xoá sạch. Trước SF-3 luật này viết lặp ở IBus và Fcitx5.

use textvn_session::reset::RESET_AFTER_ENTER_MS;
use textvn_session::{Caps, ResetKind, Session, SessionCommand, SessionEvent};

const ENTER: u32 = 0x0D;
const KEY_A: u32 = 0x41;
const SHIFT: u32 = 0x10;

fn down(s: &mut Session, vk: u32, now_ms: u64) {
    let _ = s.event(SessionEvent::KeyDown {
        vk,
        ch: 0,
        mods: 0,
        now_ms,
    });
}

fn reset(s: &mut Session, kind: ResetKind, now_ms: u64) -> bool {
    let cmds = s.event(SessionEvent::Reset { kind, now_ms });
    let resets: Vec<bool> = cmds
        .as_slice()
        .iter()
        .filter_map(|c| match c {
            SessionCommand::EngineReset { line_start } => Some(*line_start),
            _ => None,
        })
        .collect();
    assert_eq!(
        resets.len(),
        1,
        "mỗi Reset ra đúng một EngineReset: {cmds:?}"
    );
    resets[0]
}

#[test]
fn r2_171_toolkit_reset_right_after_enter_keeps_line_start() {
    let mut s = Session::new(Caps::none());
    down(&mut s, ENTER, 1_000);
    assert!(reset(
        &mut s,
        ResetKind::Toolkit,
        1_000 + RESET_AFTER_ENTER_MS - 1
    ));
}

#[test]
fn r2_171_late_reset_clears() {
    let mut s = Session::new(Caps::none());
    down(&mut s, ENTER, 1_000);
    assert!(!reset(
        &mut s,
        ResetKind::Toolkit,
        1_000 + RESET_AFTER_ENTER_MS
    ));
}

#[test]
fn r2_171_any_key_in_between_clears_even_modifier() {
    for k in [KEY_A, SHIFT] {
        let mut s = Session::new(Caps::none());
        down(&mut s, ENTER, 1_000);
        down(&mut s, k, 1_010);
        assert!(!reset(&mut s, ResetKind::Toolkit, 1_020), "vk {k:#x}");
    }
}

#[test]
fn r2_171_focus_change_clears() {
    let mut s = Session::new(Caps::none());
    down(&mut s, ENTER, 1_000);
    let _ = s.event(SessionEvent::FocusOut);
    assert!(!reset(&mut s, ResetKind::Toolkit, 1_010));

    let mut s = Session::new(Caps::none());
    down(&mut s, ENTER, 1_000);
    assert!(
        !reset(&mut s, ResetKind::Focus, 1_005),
        "reset do focus không giữ mốc"
    );
    assert!(!reset(&mut s, ResetKind::Toolkit, 1_010), "và đã xoá mốc");
}

#[test]
fn e78_app_and_plain_resets_clear() {
    let mut s = Session::new(Caps::none());
    assert!(!reset(&mut s, ResetKind::Toolkit, 1_000), "không có Enter");
    down(&mut s, ENTER, 2_000);
    assert!(!reset(&mut s, ResetKind::App, 2_010));
}

#[test]
fn r2_171_enter_mark_survives_toolkit_reset_like_legacy() {
    // IBus `engine_reset` / Fcitx5 `reset` không xoá mốc — reset thứ hai trong cửa sổ vẫn giữ.
    let mut s = Session::new(Caps::none());
    down(&mut s, ENTER, 1_000);
    assert!(reset(&mut s, ResetKind::Toolkit, 1_010));
    assert!(reset(&mut s, ResetKind::Toolkit, 1_020));
}
