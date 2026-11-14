// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-159 — hợp đồng nguồn: `handle_key` phải chặn mở composition khi focus ở control KHÔNG nhập
//! văn bản (danh sách file Explorer…) TRƯỚC khi xin edit session. Mở composition ở đó làm TSF vẽ
//! khung `CiceroUIWndFrame` chứa chữ vừa gõ ở góc trái trên màn hình (chủ repo, 2026-11-08).
//! Kiểm hành vi thật: `tools/win/e2e-non-text-focus.ps1 -ForceVietnamese` (cần bản đã cài).

const KEY_EVENT: &str = include_str!("../src/key_event.rs");
const COMPOSE: &str = include_str!("../src/compose.rs");

#[test]
fn handle_key_gates_non_text_focus_before_requesting_an_edit_session() {
    let gate = KEY_EVENT
        .find("should_yield_when_idle")
        .expect("handle_key phải gọi should_yield_when_idle");
    let session = KEY_EVENT[gate..]
        .find("KeyEditSession::new(")
        .map(|i| gate + i)
        .expect("KeyEditSession::new phải đứng SAU cổng");
    assert!(gate < session);
    // Hợp đồng cấu tạo `should_yield_when_idle` phải phủ đủ lớp không nhập văn bản + terminal/X11 + root.
    assert!(COMPOSE.contains("is_non_text_window_class"));
    assert!(COMPOSE.contains("is_non_text_top_level_class"));
    assert!(COMPOSE.contains("is_hook_owned_window_class"));
    // Cổng chỉ áp khi chưa composing (không cắt ngang từ đang gõ) và phím sinh chữ.
    let window: String = KEY_EVENT[..gate]
        .chars()
        .rev()
        .take(1500)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert!(
        window.contains("!shared.is_composing()") && window.contains("needs_session_when_idle")
    );
    // Phím phải tới app (không ăn) và engine dọn trạng thái.
    let after: String = KEY_EVENT[gate..].chars().take(1500).collect();
    assert!(after.contains("reset_engine()") && after.contains("return false;"));
}

#[test]
fn trace_logs_class_name_only_never_typed_content() {
    // S2/G8: dòng trace chỉ có tên lớp cửa sổ và cờ trạng thái.
    assert!(KEY_EVENT.contains("skip non-text focus class="));
}

/// R2-161 / E49: terminal do hook sở hữu (exe + lớp cửa sổ X11/TTY) và context disabled/empty
/// đều được nhường trước khi xin edit session.
#[test]
fn hook_owned_and_taskbar_root_are_gated_before_the_edit_session() {
    let hook = KEY_EVENT
        .find("shared.hook_owned.get()")
        .expect("handle_key phải nhường terminal do hook sở hữu");
    let ctx = KEY_EVENT
        .find("context_blocks_typing")
        .expect("handle_key phải kiểm context rỗng/disabled/chỉ đọc (E49)");
    let yield_fn = KEY_EVENT
        .find("should_yield_when_idle")
        .expect("handle_key phải gọi should_yield_when_idle");
    let session = KEY_EVENT
        .find("KeyEditSession::new(")
        .expect("KeyEditSession::new");
    assert!(hook < session && ctx < session && yield_fn < session);
}
