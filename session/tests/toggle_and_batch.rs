// SPDX-License-Identifier: GPL-3.0-or-later
//! Kiểm thử toàn bộ luật P0-6 §5 cho khối `toggle` và `batch` trong `textvn-session` (SF-2).

use textvn_session::batch::utf8_char_count;
use textvn_session::toggle::{
    CTRL_TAP_MAX_MS, MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_SHIFT, MOD_SUPER, TOGGLE_DEBOUNCE_MS,
    VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_SHIFT, VK_SPACE,
};
use textvn_session::{
    Caps, Commands, EditBatch, ResetKind, Session, SessionCommand, SessionEvent, ToggleOutcome,
    ToggleState, EDIT_BATCH_MAX_BYTES,
};

fn kd(vk: u32, mods: u32, now_ms: u64) -> SessionEvent<'static> {
    SessionEvent::KeyDown {
        vk,
        ch: if vk == VK_SPACE { ' ' as u32 } else { 0 },
        mods,
        now_ms,
    }
}

fn ku(vk: u32, mods: u32, now_ms: u64) -> SessionEvent<'static> {
    SessionEvent::KeyUp { vk, mods, now_ms }
}

#[test]
fn sf2_toggle_ctrl_shift_ca_hai_thu_tu_va_khong_toggle_lap() {
    let mut s = Session::new(Caps::none());
    assert!(s.vi_on());

    // Ctrl ↓, Shift ↓, Shift ↑ → toggle 1 lần; nhả Ctrl kế tiếp không toggle lại.
    assert_eq!(
        s.event(kd(VK_LCONTROL, 0, 100)).as_slice(),
        [SessionCommand::ForwardKey]
    );
    assert_eq!(
        s.event(kd(VK_LSHIFT, MOD_CTRL, 110)).as_slice(),
        [SessionCommand::ForwardKey]
    );
    assert_eq!(
        s.event(ku(VK_LSHIFT, MOD_CTRL, 120)).as_slice(),
        [SessionCommand::ModeIndicator { vi_on: false }]
    );
    assert!(!s.vi_on());
    assert!(s.event(ku(VK_LCONTROL, 0, 125)).is_empty());
    assert!(!s.vi_on());

    // Shift ↓ (sau >250 ms), Ctrl ↓, repeat Ctrl ↓, Ctrl ↑ → toggle lại về true.
    assert_eq!(
        s.event(kd(VK_SHIFT, 0, 500)).as_slice(),
        [SessionCommand::ForwardKey]
    );
    assert_eq!(
        s.event(kd(VK_CONTROL, MOD_SHIFT, 510)).as_slice(),
        [SessionCommand::ForwardKey]
    );
    assert_eq!(
        s.event(kd(VK_CONTROL, MOD_SHIFT, 520)).as_slice(),
        [SessionCommand::ForwardKey]
    );
    assert_eq!(
        s.event(ku(VK_CONTROL, MOD_SHIFT, 530)).as_slice(),
        [SessionCommand::ModeIndicator { vi_on: true }]
    );
    assert!(s.vi_on());
}

#[test]
fn sf2_toggle_huy_khi_co_phim_khac_alt_super_hoac_focus_reset() {
    let mut s = Session::new(Caps::none());

    // Ctrl+Shift+Z (phím thứ 3) → huỷ, không toggle.
    s.event(kd(VK_CONTROL, 0, 100));
    s.event(kd(VK_SHIFT, MOD_CTRL, 110));
    s.event(kd(0x5A, MOD_CTRL | MOD_SHIFT, 120));
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL, 130)).is_empty());
    assert!(s.vi_on());

    // Có Alt / Win xen vào → huỷ.
    s.event(kd(VK_SHIFT, MOD_CTRL | MOD_ALT, 200));
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL | MOD_ALT, 210)).is_empty());
    s.event(kd(VK_SHIFT, MOD_CTRL | MOD_SUPER, 250));
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL | MOD_SUPER, 260)).is_empty());
    assert!(s.vi_on());

    // FocusOut hoặc Reset giữa chừng → huỷ.
    s.event(kd(VK_CONTROL, 0, 400));
    s.event(kd(VK_SHIFT, MOD_CTRL, 410));
    assert!(s.event(SessionEvent::FocusOut).is_empty());
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL, 420)).is_empty());

    s.event(kd(VK_CONTROL, 0, 600));
    s.event(kd(VK_SHIFT, MOD_CTRL, 610));
    // Reset chỉ ra lệnh reset engine (SF-3), KHÔNG bật/tắt Việt–Anh.
    assert!(!s
        .event(SessionEvent::Reset {
            kind: ResetKind::Toolkit,
            now_ms: 615,
        })
        .as_slice()
        .iter()
        .any(|c| matches!(c, SessionCommand::ModeIndicator { .. })));
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL, 620)).is_empty());
    assert!(s.vi_on());
}

#[test]
fn sf2_toggle_ctrl_shift_space_va_capslock_adr011_va_debounce_250ms() {
    let mut s = Session::new(Caps::none());

    // Ctrl+Shift+Space với CapsLock bật → vẫn toggle (ADR-011) và ăn phím.
    let out = s.event(kd(VK_SPACE, MOD_CTRL | MOD_SHIFT | MOD_CAPS, 100));
    assert_eq!(
        out.as_slice(),
        [
            SessionCommand::ModeIndicator { vi_on: false },
            SessionCommand::EatKey
        ]
    );
    assert!(!s.vi_on());

    // Nhấn lại trong cửa sổ < 250 ms → debounce nuốt phím nhưng KHÔNG lật vi_on.
    let debounced = s.event(kd(
        VK_SPACE,
        MOD_CTRL | MOD_SHIFT,
        100 + TOGGLE_DEBOUNCE_MS - 1,
    ));
    assert_eq!(debounced.as_slice(), [SessionCommand::EatKey]);
    assert!(!s.vi_on());

    // Đúng >= 250 ms sau lần toggle thật cuối cùng → cho phép lật lại.
    let allowed = s.event(kd(VK_SPACE, MOD_CTRL | MOD_SHIFT, 100 + TOGGLE_DEBOUNCE_MS));
    assert_eq!(
        allowed.as_slice(),
        [
            SessionCommand::ModeIndicator { vi_on: true },
            SessionCommand::EatKey
        ]
    );
    assert!(s.vi_on());

    // Ctrl+Shift+Space đặt `armed = false` nên nhả Shift ngay sau đó không bật lại.
    assert!(s.event(ku(VK_SHIFT, MOD_CTRL, 410)).is_empty());
}

#[test]
fn sf2_standalone_ctrl_tap_gioi_han_400ms_r2_143() {
    let mut s = Session::new(Caps::none());

    // Tap nhanh <= 400 ms → CtrlTap, không toggle VI/EN.
    s.event(kd(VK_LCONTROL, 0, 1000));
    assert!(s
        .event(ku(VK_LCONTROL, 0, 1000 + CTRL_TAP_MAX_MS))
        .is_empty());
    assert_eq!(s.last_toggle_outcome(), ToggleOutcome::CtrlTap);
    assert!(s.vi_on());

    // Giữ > 400 ms (Ctrl+scroll / Ctrl+click) → KHÔNG fire CtrlTap.
    s.event(kd(VK_RCONTROL, 0, 2000));
    assert!(s
        .toggle_state()
        .ctrl_held_too_long(2000 + CTRL_TAP_MAX_MS + 1));
    s.event(ku(VK_RCONTROL, 0, 2000 + CTRL_TAP_MAX_MS + 1));
    assert_eq!(s.last_toggle_outcome(), ToggleOutcome::None);

    // Kèm Alt (VK_LMENU) hoặc Win (VK_LWIN) hoặc Shift (VK_RSHIFT) → huỷ CtrlTap.
    s.event(kd(VK_CONTROL, 0, 3000));
    s.event(kd(VK_LMENU, MOD_CTRL | MOD_ALT, 3010));
    s.event(ku(VK_CONTROL, 0, 3020));
    assert_eq!(s.last_toggle_outcome(), ToggleOutcome::None);

    s.event(kd(VK_CONTROL, 0, 4000));
    s.event(kd(VK_LWIN, MOD_CTRL | MOD_SUPER, 4010));
    s.event(ku(VK_CONTROL, 0, 4020));
    assert_eq!(s.last_toggle_outcome(), ToggleOutcome::None);

    // Phím chữ xen giữa (Ctrl+C) → huỷ CtrlTap.
    s.event(kd(VK_CONTROL, 0, 5000));
    s.event(kd(0x43, MOD_CTRL, 5010));
    s.event(ku(VK_CONTROL, 0, 5020));
    assert_eq!(s.last_toggle_outcome(), ToggleOutcome::None);
}

#[test]
fn sf2_toggle_ipc_dong_bo_trang_thai_global() {
    let mut s = Session::new(Caps::none());
    assert!(s.vi_on());
    s.event(SessionEvent::Ipc(textvn_ipc::Message::ToggleViEn {
        app_id: "*".into(),
        enabled: false,
    }));
    assert!(!s.vi_on());
    s.event(SessionEvent::Ipc(textvn_ipc::Message::StateUpdate {
        app_id: "notepad.exe".into(),
        enabled: true,
        version: 1,
    }));
    assert!(
        !s.vi_on(),
        "per-app StateUpdate khong duoc ghi de co global '*'"
    );
    s.event(SessionEvent::Ipc(textvn_ipc::Message::StateUpdate {
        app_id: "*".into(),
        enabled: true,
        version: 2,
    }));
    assert!(s.vi_on());
}

#[test]
fn sf2_toggle_state_debounce_giu_nguyen_qua_reset() {
    let mut t = ToggleState::new();
    t.on_key_down(VK_CONTROL, 0, 0);
    t.on_key_down(VK_SHIFT, MOD_CTRL, 10);
    assert_eq!(t.on_key_up(VK_SHIFT, 20), ToggleOutcome::ToggleViEn);
    t.reset();
    // Sau reset trong 250 ms vẫn không cho toggle lần 2 (chống 2 pha OnKeyUp/FocusOut).
    assert_eq!(
        t.on_key_down(VK_SPACE, MOD_CTRL | MOD_SHIFT, 100),
        ToggleOutcome::None
    );
}

#[test]
fn sf2_batch_gop_lenh_an_text_utf8_va_tuan_thu_r2_165_167() {
    let mut b = EditBatch::new();
    assert!(!b.is_active());
    assert!(!b.commit("a"), "ngoai batch khong duoc nhan commit");

    b.begin();
    b.begin(); // lồng nhau
    assert!(b.commit("tiế"));
    assert!(b.commit("ng"));
    assert_eq!(utf8_char_count(b.pending_text()), 5);

    // Đóng lớp trong cùng -> chưa xả.
    assert!(b.end().is_none());
    assert!(b.is_active());

    // Xoá 3 ký tự: ăn 3 ký tự cuối trong pending_text ("ế", "n", "g")? -> còn "ti", delete_before=0.
    // Xoá 4 ký tự: hết pending ("ti" + thêm 2 ký tự thật trước caret).
    b.delete(3);
    assert_eq!(b.pending_text(), "ti");
    assert_eq!(b.pending_delete_before(), 0);
    b.delete(4);
    assert_eq!(b.pending_text(), "");
    assert_eq!(b.pending_delete_before(), 2);

    assert!(b.commit("Việt"));
    let flushed = b.end().expect("dong lop ngoai phai tra batch");
    assert_eq!(flushed, (2, "Việt".to_string()));
    assert!(!b.is_active());

    // flush_into(&mut Commands) đẩy <= 1 DeleteBefore rồi <= 1 Commit, giữ bất biến R2-165/R2-167.
    let mut cmds = Commands::new();
    b.begin();
    b.delete(3);
    assert!(b.commit("chào"));
    b.flush_into(&mut cmds);
    cmds.push(SessionCommand::SetPreedit {
        text: String::new(),
        cursor: 0,
    });
    cmds.push(SessionCommand::ForwardKey);
    assert_eq!(
        cmds.as_slice(),
        [
            SessionCommand::DeleteBefore { chars: 3 },
            SessionCommand::Commit {
                text: "chào".into()
            },
            SessionCommand::SetPreedit {
                text: String::new(),
                cursor: 0
            },
            SessionCommand::ForwardKey
        ]
    );
}

#[test]
fn sf2_batch_chan_tran_bo_dem_1024_byte() {
    let mut b = EditBatch::new();
    b.begin();
    let full = "a".repeat(EDIT_BATCH_MAX_BYTES);
    assert!(b.commit(&full));
    assert!(
        !b.commit("b"),
        "vuot EDIT_BATCH_MAX_BYTES phai tu choi an toan"
    );
    assert_eq!(b.end().map(|(_, t)| t.len()), Some(EDIT_BATCH_MAX_BYTES));
}
