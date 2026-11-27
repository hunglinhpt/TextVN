// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-173 — **gợi ý nối tiếp gợi ý** (chủ repo 2026-11-27: "dùng text prediction xong thì
//! gợi ý sau đó cũng bị tắt luôn, cần gợi ý tiếp khi user Space hoặc Tab xong").
//!
//! Nguyên nhân: nhận gợi ý bằng Tab đưa engine vào vòng duyệt ứng viên — Tab kế tiếp từng luôn
//! ĐỔI ứng viên nên `peek_completion` (chữ mờ = "Tab sẽ chèn gì") trả `None`: chữ mờ tắt hẳn
//! ngay sau mỗi lần nhận. Giờ (chủ repo chọn phương án "Tab nối tiếp"):
//! - sau khi nhận, còn cụm nối tiếp khoá ≥ 2 từ → chữ mờ hiện cụm đó, Tab nhận nó;
//! - không còn cụm (hoặc khoá chỉ 1 từ) → Tab duyệt vòng như cũ; Ctrl+Space luôn duyệt vòng;
//! - bất biến: Tab chèn ĐÚNG chữ mờ đang thấy, và chỉ nối thêm (không xoá chữ người dùng);
//! - Dream RSI (G19): mỗi cụm nối tiếp là một lần nhận; Backspace ngay sau gỡ đúng cụm mới.

use textvn_core::keymap::{self, vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

fn engine(vi: bool) -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: vi,
        suggest_vn_complete: true,
        suggest_space_complete: false,
        auto_restore_english: true,
        ..Default::default()
    })
}

fn key(c: char) -> KeyEvent {
    match c {
        ' ' => KeyEvent {
            vk: vk::SPACE,
            ch: ' ' as u32,
            key_down: true,
            ..Default::default()
        },
        '\t' => KeyEvent {
            vk: vk::TAB,
            ch: '\t' as u32,
            key_down: true,
            ..Default::default()
        },
        '\u{8}' => KeyEvent {
            vk: vk::BACK,
            key_down: true,
            ..Default::default()
        },
        _ => KeyEvent::char_down(c),
    }
}

fn ctrl_space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        mods: keymap::MOD_CTRL,
        key_down: true,
        ..Default::default()
    }
}

fn press(e: &mut Engine, doc: &mut Vec<char>, k: &KeyEvent) {
    match e.key(k).action {
        Action::Pass => match k.vk {
            vk::BACK => {
                doc.pop();
            }
            _ if k.is_ctrl_space() => {}
            _ => doc.extend(k.printable()),
        },
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            for _ in 0..delete_count {
                doc.pop();
            }
            doc.extend(insert);
        }
        Action::Commit { insert } => doc.extend(insert),
    }
}

fn type_str(e: &mut Engine, doc: &mut Vec<char>, s: &str) {
    for c in s.chars() {
        press(e, doc, &key(c));
    }
}

fn text(doc: &[char]) -> String {
    doc.iter().collect()
}

#[test]
fn tab_accept_then_ghost_shows_continuation_and_tab_inserts_it() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "phats");
    assert_eq!(e.peek_completion().as_deref(), Some(" triển"));
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "phát triển");
    // Trước R2-173: None (chữ mờ tắt ngay sau khi nhận).
    assert_eq!(e.peek_completion().as_deref(), Some(" phần mềm"));
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "phát triển phần mềm");
}

#[test]
fn chain_works_for_formal_letter_phrases() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "Kinhs guwir");
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "Kính gửi quý khách hàng");
    assert_eq!(e.peek_completion().as_deref(), Some(" tiềm năng"));
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "Kính gửi quý khách hàng tiềm năng");
}

/// Dữ liệu R2-173 (`data/vn_phrases.tsv`): lời chào/chúc thường gặp từng không có cụm nối
/// tiếp khoá 2 từ — "xin"+Tab ra "xin chào" rồi chuỗi gợi ý dừng hẳn.
#[test]
fn greeting_and_office_contexts_chain() {
    for (input, first, ghost, chained) in [
        ("xin", "xin chào", " các bạn", "xin chào các bạn"),
        ("phats sinh", "phát sinh", " chi phí", "phát sinh chi phí"),
    ] {
        let mut e = engine(true);
        let mut doc = Vec::new();
        type_str(&mut e, &mut doc, input);
        if text(&doc) != first {
            press(&mut e, &mut doc, &key('\t'));
        }
        assert_eq!(text(&doc), first, "{input:?}");
        assert_eq!(e.peek_completion().as_deref(), Some(ghost), "{input:?}");
        press(&mut e, &mut doc, &key('\t'));
        assert_eq!(text(&doc), chained, "{input:?}");
    }
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "chucs banj ");
    assert_eq!(e.peek_completion().as_deref(), Some("một ngày tốt lành"));
}

/// Space sau khi nhận vẫn gợi ý tiếp (chữ mờ không kèm dấu cách thừa).
#[test]
fn space_after_accept_keeps_suggesting() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "Kinhs guwir");
    press(&mut e, &mut doc, &key('\t'));
    press(&mut e, &mut doc, &key(' '));
    assert_eq!(text(&doc), "Kính gửi quý khách hàng ");
    assert_eq!(e.peek_completion().as_deref(), Some("tiềm năng"));
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "Kính gửi quý khách hàng tiềm năng");

    let mut en = engine(false);
    let mut doc = Vec::new();
    type_str(&mut en, &mut doc, "thanks ");
    assert_eq!(en.peek_completion().as_deref(), Some("a lot"));
    press(&mut en, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "thanks a lot");
}

/// Khoá 1 từ quá yếu để tự nối: "customer"+Tab (R2-149 khôi phục từ) rồi Tab vẫn ra
/// "customer service" và Tab thứ ba vẫn duyệt vòng sang "customer support".
#[test]
fn one_word_context_keeps_cycling() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "customer");
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "customer");
    assert_eq!(e.peek_completion().as_deref(), Some(" service"));
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "customer service");
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "customer support");
}

/// Ctrl+Space luôn đổi ứng viên, kể cả khi đang có cụm nối tiếp.
#[test]
fn ctrl_space_always_cycles_after_accept() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "phats");
    press(&mut e, &mut doc, &key('\t'));
    assert_eq!(text(&doc), "phát triển");
    press(&mut e, &mut doc, &ctrl_space());
    assert_eq!(text(&doc), "phát sinh");
}

/// Bất biến trên cả Mode VI lẫn Mode EN: sau MỌI phím, nếu chữ mờ hiện `g` thì Tab biến
/// document thành đúng `doc + g` — không xoá, không thay chữ người dùng.
#[test]
fn ghost_is_exactly_what_tab_inserts_across_chains() {
    let vi_inputs = [
        "phats",
        "phats trieenr",
        "hopwj ddoongf",
        "Kinhs guwir",
        "coongj hoaf",
        "xin",
        "caamf",
        "ngaan hangf",
        "coong ty",
        "Thank you",
        "customer",
        "project",
    ];
    let en_inputs = [
        "thank", "how", "machine", "customer", "project", "pull", "source", "thanks ",
    ];
    for (vi, inputs) in [(true, &vi_inputs[..]), (false, &en_inputs[..])] {
        for input in inputs {
            let mut e = engine(vi);
            let mut doc = Vec::new();
            type_str(&mut e, &mut doc, input);
            for step in 0..6 {
                let before = text(&doc);
                let ghost = e.peek_completion();
                press(&mut e, &mut doc, &key('\t'));
                if let Some(g) = ghost {
                    assert_eq!(
                        text(&doc),
                        format!("{before}{g}"),
                        "vi={vi} {input:?} bước {step}: Tab phải chèn đúng chữ mờ"
                    );
                }
            }
        }
    }
}

/// G19 — mỗi cụm nối tiếp là MỘT lần nhận + đúng số ký tự cụm; Backspace ngay sau cụm nối
/// tiếp chỉ gỡ phần của cụm đó (cụm trước đã chốt khi người dùng bấm Tab nối tiếp).
#[test]
fn dream_rsi_counts_each_chained_accept_once() {
    let mut e = engine(true);
    let mut doc = Vec::new();
    type_str(&mut e, &mut doc, "phats");
    let r0 = e.dream_rsi_snapshot();
    press(&mut e, &mut doc, &key('\t')); // " triển"
    let r1 = e.dream_rsi_snapshot();
    assert_eq!(r1.accepts, r0.accepts + 1);
    assert_eq!(
        r1.chars_saved,
        r0.chars_saved + " triển".chars().count() as u64
    );
    press(&mut e, &mut doc, &key('\t')); // " phần mềm"
    let r2 = e.dream_rsi_snapshot();
    assert_eq!(text(&doc), "phát triển phần mềm");
    assert_eq!(r2.accepts, r1.accepts + 1);
    assert_eq!(
        r2.chars_saved,
        r1.chars_saved + " phần mềm".chars().count() as u64
    );
    // Tab là phím gõ thật (cùng định nghĩa `keys_typed`): mỗi Tab +1.
    assert_eq!(r2.keys_typed, r0.keys_typed + 2);
    press(&mut e, &mut doc, &key('\u{8}'));
    let r3 = e.dream_rsi_snapshot();
    assert_eq!(
        (r3.accepts, r3.chars_saved),
        (r1.accepts, r1.chars_saved),
        "Backspace từ chối đúng cụm vừa nối, giữ cụm trước"
    );
}
