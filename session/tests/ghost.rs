// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-4 · khối ghost (P0-6 §5): thời điểm vẽ (R2-134/R2-173/E79), cổng, nhãn, và bất biến
//! "chữ mờ = đúng phần Tab chèn" chạy trên engine thật.

use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::{sim, Engine, EngineOptions};
use textvn_session::compose::KeyKind;
use textvn_session::ghost::{gate, ghost_text, ghost_timing, preview_label, GhostTiming};

/// R2-173: chỉ phím sinh/xoá ký tự mà bộ gõ KHÔNG ăn mới hoãn chữ mờ tới key-up (app chưa chèn
/// ký tự lúc bộ gõ chạy). Tab/Ctrl+Space bị ăn (đã chèn gợi ý) → vẽ ngay. (Chuyển từ TSF.)
#[test]
fn r2_173_ghost_waits_for_key_up_only_for_passed_char_keys() {
    assert_eq!(
        ghost_timing(KeyKind::Char(' '), false),
        GhostTiming::AfterKeyUp
    );
    assert_eq!(
        ghost_timing(KeyKind::Backspace, false),
        GhostTiming::AfterKeyUp
    );
    assert_eq!(ghost_timing(KeyKind::Char(' '), true), GhostTiming::Now);
    assert_eq!(ghost_timing(KeyKind::Char('a'), true), GhostTiming::Now);
    assert_eq!(ghost_timing(KeyKind::Tab, true), GhostTiming::Now);
    assert_eq!(ghost_timing(KeyKind::Tab, false), GhostTiming::Now);
    assert_eq!(ghost_timing(KeyKind::CtrlSpace, false), GhostTiming::Now);
}

/// R2-134: phím không gõ chữ (Enter, Esc, điều hướng) không vẽ lại.
#[test]
fn r2_134_other_keys_keep_ghost() {
    for k in [
        KeyKind::Enter,
        KeyKind::Escape,
        KeyKind::Other,
        KeyKind::AltGr('e'),
    ] {
        for eaten in [true, false] {
            assert_eq!(ghost_timing(k, eaten), GhostTiming::Keep, "{k:?}");
        }
    }
}

#[test]
fn ghost_is_peek_verbatim_and_every_gate_hides_it() {
    assert_eq!(
        ghost_text(Some(" phần mềm"), 0).as_deref(),
        Some(" phần mềm")
    );
    assert_eq!(ghost_text(Some("op"), 0).as_deref(), Some("op"));
    assert_eq!(ghost_text(None, 0), None);
    assert_eq!(ghost_text(Some("   "), 0), None);
    for g in [
        gate::SECURE,
        gate::TERMINAL,
        gate::ASSIST_VISIBLE,
        gate::SUGGEST_OFF,
        gate::TEXT_AFTER_CARET,
    ] {
        assert_eq!(ghost_text(Some("op"), g), None, "gate {g:#x}");
    }
}

#[test]
fn preview_label_trims_and_prefixes() {
    assert_eq!(preview_label(" phần mềm").as_deref(), Some("⇥ phần mềm"));
    assert_eq!(preview_label("op ").as_deref(), Some("⇥ op"));
    assert_eq!(preview_label("  "), None);
}

fn type_and_check(engine: &mut Engine, buf: &mut Vec<char>, text: &str, checked: &mut usize) {
    for c in text.chars() {
        let key = if c == ' ' {
            KeyEvent {
                vk: vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..KeyEvent::default()
            }
        } else {
            KeyEvent::char_down(c)
        };
        let out = engine.key(&key);
        sim::apply(buf, &out.action, &key);

        // Bất biến R2-173: nếu có chữ mờ, nhấn Tab (trên bản sao) nối ĐÚNG chữ đó vào cuối văn bản.
        if let Some(ghost) = ghost_text(engine.peek_completion().as_deref(), 0) {
            let mut probe = engine.clone();
            let mut after = buf.clone();
            let tab = KeyEvent {
                vk: vk::TAB,
                ch: '\t' as u32,
                key_down: true,
                ..KeyEvent::default()
            };
            let r = probe.key(&tab);
            sim::apply(&mut after, &r.action, &tab);
            let before: String = buf.iter().collect();
            let got: String = after.iter().collect();
            assert_eq!(
                got,
                format!("{before}{ghost}"),
                "chữ mờ {ghost:?} ≠ phần Tab chèn"
            );
            *checked += 1;
        }
    }
}

#[test]
fn r2_173_ghost_equals_what_tab_inserts_on_real_engine() {
    let mut engine = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        ..EngineOptions::default()
    });
    let mut buf = Vec::new();
    let mut checked = 0;
    for phrase in [
        "phaanf mee",
        "cooj hoaf xax hooij chur nghiax vieetj nam ",
        "ddoocj laapj tuwj do hanhj phus",
        "xin chaof cacs banj ",
        "thoong tin ",
    ] {
        type_and_check(&mut engine, &mut buf, phrase, &mut checked);
    }
    assert!(
        checked > 0,
        "corpus phải sinh ít nhất một chữ mờ để kiểm bất biến"
    );
}
