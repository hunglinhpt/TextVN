// SPDX-License-Identifier: GPL-3.0-or-later
//! Audit đối kháng V57-B/C — các điểm chưa phủ trong style_learning_100:
//! 1. Prediction-accept học KHÔNG học từ "ghost chưa accept" (peek không học)
//! 2. Drain sau Tab học candidate (không duplicate count từ word-end)
//! 3. Correction drain shape khớp merge_deltas phía config (serde parse)
//! 4. Cycle-accept: Tab ×2 học candidate thứ 2, không học candidate 1 hai lần
//! 5. Drain 2 lần liên tiếp: lần 2 chỉ chứa pending mới
use textvn_core::keymap::{self, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

fn apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            if let Some(c) = k.printable() {
                if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                    buf.push(c);
                }
            }
        }
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            for _ in 0..*delete_count {
                buf.pop();
            }
            buf.extend(insert.iter().cloned());
        }
        Action::Commit { insert } => buf.extend(insert.iter().cloned()),
    }
}

fn press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    apply(buf, &out.action, k);
}

fn letter(c: char) -> KeyEvent {
    KeyEvent::char_down(c)
}
fn space() -> KeyEvent {
    KeyEvent {
        vk: keymap::vk::SPACE,
        ch: ' ' as u32,
        key_down: true,
        ..Default::default()
    }
}
fn tab() -> KeyEvent {
    KeyEvent {
        vk: keymap::vk::TAB,
        ch: '\t' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn engine() -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        style_learning_enabled: true,
        ..Default::default()
    })
}

/// Audit 1: peek (ghost preview) KHÔNG được học — preview không phải chấp nhận.
#[test]
fn adv1_peek_khong_hoc() {
    let mut e = engine();
    let mut buf = Vec::new();
    press(&mut e, &mut buf, &letter('r'));
    press(&mut e, &mut buf, &letter('e'));
    press(&mut e, &mut buf, &letter('l'));
    press(&mut e, &mut buf, &letter('e'));
    let _ = e.peek_completion(); // preview — KHÔNG accept
    let json = e.style_drain_json();
    assert!(
        !json.contains("release"),
        "peek KHÔNG được học (chỉ Tab mới học): {json}"
    );
}

/// Audit 2: Tab accept học candidate đúng 1 lần với count 1.
#[test]
fn adv2_tab_accept_count_1() {
    let mut e = engine();
    let mut buf = Vec::new();
    press(&mut e, &mut buf, &letter('r'));
    press(&mut e, &mut buf, &letter('e'));
    press(&mut e, &mut buf, &letter('l'));
    press(&mut e, &mut buf, &letter('e'));
    press(&mut e, &mut buf, &tab());
    let json = e.style_drain_json();
    assert!(
        json.contains("\"count\":1"),
        "accept 1 lần = count 1: {json}"
    );
}

/// Audit 3: drain shape khớp JSON output chuẩn (parse thuần string — core 0-dep).
#[test]
fn adv3_drain_shape_json_chuan() {
    let mut e = engine();
    e.style_note_word("hợp đồng");
    e.style_note_correction("xử dụng", "sử dụng");
    let json = e.style_drain_json();
    // Shape đúng hợp đồng tray (style_merge.rs): 2 object với đủ key.
    assert!(json.contains("\"word\":\"hợp đồng\""), "{json}");
    assert!(json.contains("\"count\":1"), "{json}");
    assert!(json.contains("\"from\":\"xử dụng\""), "{json}");
    assert!(json.contains("\"to\":\"sử dụng\""), "{json}");
    assert!(
        json.starts_with("{\"words\":[") && json.contains("\"corrections\":["),
        "{json}"
    );
}

/// Audit 4: cycle Tab ×2 — học candidate duyệt tới, không duplicate candidate cũ.
#[test]
fn adv4_cycle_hoc_dung_candidate() {
    let mut e = engine();
    let mut buf = Vec::new();
    for c in "releas".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    press(&mut e, &mut buf, &tab()); // release
    press(&mut e, &mut buf, &tab()); // cycle — có candidate thứ 2 không?
    let json = e.style_drain_json();
    // drain có "release" (candidate 1); candidate 2 (nếu có) cũng học — KHÔNG
    // có count >= 2 cho cùng 1 word ở lần cycle 1
    let rel_count = json.matches("\"word\":\"release\"").count();
    assert!(rel_count <= 1, "release không học trùng: {json}");
}

/// Audit 5: drain 2 lần liên tiếp — lần 2 chỉ pending mới (drain xóa pending cũ).
#[test]
fn adv5_drain_2_lan_chi_pending_moi() {
    let mut e = engine();
    let mut buf = Vec::new();
    press(&mut e, &mut buf, &letter('a'));
    press(&mut e, &mut buf, &letter('b'));
    press(&mut e, &mut buf, &space());
    let j1 = e.style_drain_json();
    assert!(
        j1.contains("\"abc\""),
        "lần 1 học kết quả space-complete: {j1}"
    );
    press(&mut e, &mut buf, &letter('c'));
    press(&mut e, &mut buf, &letter('d'));
    press(&mut e, &mut buf, &space());
    let j2 = e.style_drain_json();
    assert!(j2.contains("\"cd\""), "lần 2 pending mới: {j2}");
    assert!(
        !j2.contains("\"ab\""),
        "lần 2 KHÔNG chứa pending cũ (đã drain): {j2}"
    );
}
