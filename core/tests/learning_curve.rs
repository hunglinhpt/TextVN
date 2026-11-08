// SPDX-License-Identifier: GPL-3.0-or-later
//! 0.2.74 — đường cong tự học (P7) + Dream RSI (P10) từ gõ thật tới số đo bền vững:
//! gõ từ riêng lần đầu → học trong phiên → lần sau Tab hoàn tất (accepts/chars_saved tăng) →
//! `rsi_delta` của mỗi lượt drain cộng lại đúng bằng snapshot (host cộng dồn, không đếm đúp).

use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

fn press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    match &out.action {
        Action::Pass => {
            if let Some(c) = k.printable() {
                if !c.is_control() || c == ' ' || c == '\t' {
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

fn space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        ch: ' ' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn tab() -> KeyEvent {
    KeyEvent {
        vk: vk::TAB,
        ch: '\t' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn type_text(e: &mut Engine, buf: &mut Vec<char>, s: &str) {
    for c in s.chars() {
        if c == ' ' {
            press(e, buf, &space());
        } else {
            press(e, buf, &KeyEvent::char_down(c));
        }
    }
}

fn engine(learning: bool) -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        style_learning_enabled: learning,
        ..Default::default()
    })
}

fn delta(json: &str) -> (u64, u64, u64) {
    let v: serde_json::Value = serde_json::from_str(json).expect("drain JSON hợp lệ");
    let d = &v["rsi_delta"];
    (
        d["keys_typed"].as_u64().unwrap_or(0),
        d["accepts"].as_u64().unwrap_or(0),
        d["chars_saved"].as_u64().unwrap_or(0),
    )
}

/// Từ riêng gõ đủ lần đầu → những lần sau chỉ cần 3 chữ + Tab, và savings thật sự tăng.
#[test]
fn tu_rieng_duoc_hoc_roi_tab_hoan_tat_va_tiet_kiem_tang() {
    let mut e = engine(true);
    let mut buf = Vec::new();
    // Lần 1: chưa biết `blorptastic` → phải gõ đủ; chưa có accept nào.
    type_text(&mut e, &mut buf, "blorptastic ");
    assert_eq!(e.dream_rsi_snapshot().accepts, 0);
    let rate_first = e.dream_rsi_snapshot().savings_rate_pct();
    // Lần 2..4: gõ 3 chữ rồi Tab.
    for _ in 0..3 {
        type_text(&mut e, &mut buf, "blo");
        press(&mut e, &mut buf, &tab());
        press(&mut e, &mut buf, &space());
    }
    let s = e.dream_rsi_snapshot();
    assert!(s.accepts >= 3, "đã học → Tab hoàn tất: {s:?}");
    assert!(s.chars_saved >= 3 * 8, "mỗi lần tiết kiệm 8 ký tự: {s:?}");
    assert!(
        s.savings_rate_pct() > rate_first,
        "savings-rate phải tăng khi đã học"
    );
    let text: String = buf.iter().collect();
    assert_eq!(text.matches("blorptastic").count(), 4, "{text}");
}

/// Tắt học (người dùng bấm tắt) → không có gì được ghi để lưu (G8): drain không chứa từ nào.
#[test]
fn tat_tu_hoc_thi_khong_ghi_tu_nao() {
    let mut e = engine(false);
    let mut buf = Vec::new();
    type_text(&mut e, &mut buf, "blorptastic ");
    type_text(&mut e, &mut buf, "blo");
    press(&mut e, &mut buf, &tab());
    press(&mut e, &mut buf, &space());
    let json = e.style_drain_json();
    assert!(
        !json.contains("blorptastic"),
        "tắt tự học: không ghi từ nào: {json}"
    );
    let v: serde_json::Value = serde_json::from_str(&json).expect("JSON hợp lệ");
    assert_eq!(v["words"].as_array().map_or(0, Vec::len), 0, "{json}");
    assert_eq!(v["corrections"].as_array().map_or(0, Vec::len), 0, "{json}");
}

/// `rsi_delta` của các lượt drain cộng lại đúng bằng snapshot — kể cả khi gõ xen giữa hai lượt.
#[test]
fn rsi_delta_cac_luot_drain_cong_lai_bang_snapshot() {
    let mut e = engine(true);
    let mut buf = Vec::new();
    let mut sum = (0u64, 0u64, 0u64);
    type_text(&mut e, &mut buf, "blorptastic ");
    for round in 0..4 {
        type_text(&mut e, &mut buf, "blo");
        press(&mut e, &mut buf, &tab());
        press(&mut e, &mut buf, &space());
        let json = e.style_drain_json();
        let d = delta(&json);
        sum = (sum.0 + d.0, sum.1 + d.1, sum.2 + d.2);
        if round == 1 {
            // Drain ngay lần nữa: không còn gì mới → "{}" hoặc delta 0, không đếm đúp.
            let again = e.style_drain_json();
            assert_eq!(delta(&again), (0, 0, 0), "không đếm đúp: {again}");
        }
    }
    let tail = delta(&e.style_drain_json());
    sum = (sum.0 + tail.0, sum.1 + tail.1, sum.2 + tail.2);
    let s = e.dream_rsi_snapshot();
    assert_eq!(sum, (s.keys_typed, s.accepts, s.chars_saved));
}

/// peek (2-pha) không tiêu thụ delta; chỉ `style_clear_pending` mới chốt.
#[test]
fn peek_khong_tieu_thu_delta() {
    let mut e = engine(true);
    let mut buf = Vec::new();
    type_text(&mut e, &mut buf, "blorptastic blo");
    let a = e.style_peek_json();
    let b = e.style_peek_json();
    assert_eq!(delta(&a), delta(&b));
    assert!(delta(&a).0 > 0);
    e.style_clear_pending();
    assert_eq!(delta(&e.style_peek_json()), (0, 0, 0));
}

/// Rác bàn phím / gõ nhầm không được học (nếu không gợi ý càng dùng càng tệ); từ thật và viết tắt
/// kỹ thuật vẫn được học.
#[test]
fn khong_hoc_rac_ban_phim_nhung_hoc_tu_that_va_viet_tat() {
    let mut e = engine(true);
    let mut buf = Vec::new();
    for junk in ["aaaa", "wwwww", "sdfghjkl", "zzzzzz", "qwrtpsdf"] {
        type_text(&mut e, &mut buf, &format!("{junk} "));
    }
    for good in ["strengths", "npm", "ssh", "blorptastic"] {
        type_text(&mut e, &mut buf, &format!("{good} "));
    }
    let json = e.style_drain_json();
    for junk in ["aaaa", "wwwww", "sdfghjkl", "zzzzzz", "qwrtpsdf"] {
        assert!(
            !json.contains(&format!("\"word\":\"{junk}\"")),
            "{junk} bị học nhầm: {json}"
        );
    }
    for good in ["strengths", "npm", "ssh", "blorptastic"] {
        assert!(
            json.contains(&format!("\"word\":\"{good}\"")),
            "{good} phải được học: {json}"
        );
    }
}
