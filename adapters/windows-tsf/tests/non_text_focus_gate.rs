// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-159 — hợp đồng nguồn: `handle_key` phải chặn mở composition khi focus ở control KHÔNG nhập
//! văn bản (danh sách file Explorer…) TRƯỚC khi xin edit session. Mở composition ở đó làm TSF vẽ
//! khung `CiceroUIWndFrame` chứa chữ vừa gõ ở góc trái trên màn hình (chủ repo, 2026-11-08).
//! Kiểm hành vi thật: `tools/win/e2e-non-text-focus.ps1 -ForceVietnamese` (cần bản đã cài).

const KEY_EVENT: &str = include_str!("../src/key_event.rs");

#[test]
fn handle_key_gates_non_text_focus_before_requesting_an_edit_session() {
    let gate = KEY_EVENT
        .find("is_non_text_window_class(&class)")
        .expect("handle_key phải gọi is_non_text_window_class");
    let session = KEY_EVENT[gate..]
        .find("KeyEditSession::new(")
        .map(|i| gate + i)
        .expect("KeyEditSession::new phải đứng SAU cổng");
    assert!(gate < session);
    // Cổng chỉ áp khi chưa composing (không cắt ngang từ đang gõ) và phím sinh chữ.
    let window: String = KEY_EVENT[..gate]
        .chars()
        .rev()
        .take(400)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert!(
        window.contains("!shared.is_composing()") && window.contains("needs_session_when_idle")
    );
    // Phím phải tới app (không ăn) và engine dọn trạng thái.
    let after: String = KEY_EVENT[gate..].chars().take(600).collect();
    assert!(after.contains("reset_engine()") && after.contains("return false;"));
}

#[test]
fn trace_logs_class_name_only_never_typed_content() {
    // S2/G8: dòng trace chỉ có tên lớp cửa sổ.
    assert!(KEY_EVENT.contains("skip non-text focus class={class}"));
}
