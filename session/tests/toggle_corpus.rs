// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-2 · SF-G1: `corpus/toggle/toggle.cases` chạy qua `textvn-session` phải ra đúng số lần
//! bật/tắt đã ghi (vết vàng). Harness TSF/Linux/macOS đọc cùng tệp (SF-G2).

use textvn_session::toggle_corpus::{events, parse, run_session, CorpusEvent, CorpusStep};

const CORPUS: &str = include_str!("../../corpus/toggle/toggle.cases");

#[test]
fn sf2_toggle_corpus_golden_session() {
    let cases = parse(CORPUS).expect("toggle.cases hợp lệ");
    assert!(
        cases.len() >= 16,
        "corpus toggle quá ít ca: {}",
        cases.len()
    );
    let mut failures = Vec::new();
    for case in &cases {
        let got = run_session(case);
        if got != case.session_toggles {
            failures.push(format!(
                "{}: session {got} lần, vết vàng {}",
                case.name, case.session_toggles
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn sf2_toggle_corpus_covers_every_rule() {
    // Mỗi luật P0-6 §5 hàng toggle có ít nhất một ca: cả 2 thứ tự, phím thứ ba, Alt,
    // chord Space, reset, Ctrl tap, debounce.
    let cases = parse(CORPUS).unwrap();
    let has = |pred: &dyn Fn(&CorpusStep) -> bool| cases.iter().any(|c| c.steps.iter().any(pred));
    assert!(has(&|s| matches!(s, CorpusStep::Other)));
    assert!(has(&|s| matches!(s, CorpusStep::Space)));
    assert!(has(&|s| matches!(s, CorpusStep::Reset)));
    assert!(has(&|s| matches!(s, CorpusStep::Gap(n) if *n < 250)));
    for name in [
        "ctrl_shift_release_shift",
        "shift_ctrl_release_ctrl",
        "chord_space",
        "ctrl_tap",
        "double_toggle_fast",
    ] {
        assert!(cases.iter().any(|c| c.name == name), "thiếu ca {name}");
    }
}

#[test]
fn sf2_toggle_corpus_events_track_held_modifiers() {
    let cases = parse("x | +C +S -S -C | 1\n").unwrap();
    let ev = events(&cases[0]);
    assert_eq!(
        ev,
        vec![
            CorpusEvent::KeyDown {
                vk: 0x11,
                mods: 0,
                now_ms: 1_000
            },
            CorpusEvent::KeyDown {
                vk: 0x10,
                mods: 0x2,
                now_ms: 1_300
            },
            CorpusEvent::KeyUp {
                vk: 0x10,
                mods: 0x2,
                now_ms: 1_600
            },
            CorpusEvent::KeyUp {
                vk: 0x11,
                mods: 0,
                now_ms: 1_900
            },
        ]
    );
    assert_eq!(cases[0].legacy_toggles("linux"), 1);
}

#[test]
fn sf2_toggle_corpus_rejects_bad_lines() {
    assert!(parse("x | +C +Q | 1\n").is_err());
    assert!(parse("x | +C | many\n").is_err());
    assert!(parse("x | +C | 1 | win=2\n").is_err());
    assert!(parse("# chỉ chú thích\n").is_err());
}

#[test]
fn sf2_toggle_corpus_detects_wrong_golden() {
    // Đột biến vết vàng: đổi số lần của một ca ⇒ harness phải thấy lệch.
    let mutated = CORPUS.replacen(
        "| +C +S -S -C             | 1",
        "| +C +S -S -C             | 0",
        1,
    );
    assert_ne!(mutated, CORPUS, "mẫu đột biến phải khớp một dòng");
    let cases = parse(&mutated).unwrap();
    assert!(cases.iter().any(|c| run_session(c) != c.session_toggles));
}
