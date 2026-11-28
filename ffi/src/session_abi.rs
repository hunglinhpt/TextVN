// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-1b — C ABI `ime_session_*` cho `textvn-session` (P0-6 §4). Thêm vào CÙNG `libtextvn_ffi`
//! (không thư viện thứ hai). Không callback, không cấp phát chéo: adapter cấp
//! [`ime_session_out_v1`] (stack), session ghi vào. Mọi struct có `abi_version`; kích thước/offset
//! kiểm bởi `textvn-cli verify` trên mọi target.
//!
//! Ánh xạ sự kiện → [`SessionEvent`] (trường dùng / bỏ qua):
//!
//! | `kind` | trường |
//! |---|---|
//! | `IME_SEV_KEYDOWN` | `vk`, `ch`, `mods`, `now_ms` |
//! | `IME_SEV_KEYUP` | `vk`, `mods`, `now_ms` |
//! | `IME_SEV_FOCUS_IN` | `text` = app_id, `arg` = field_role, `mods` bit `IME_SFOCUS_SECURE`/`IME_SFOCUS_TERMINAL` |
//! | `IME_SEV_FOCUS_OUT` / `IME_SEV_CARET_MOVED` | — |
//! | `IME_SEV_SURROUNDING` | `text` = chữ trước caret |
//! | `IME_SEV_RESET` | `arg` = `IME_SRESET_*`, `now_ms` |
//! | `IME_SEV_TIMER` | `arg` = timer id, `now_ms` |
//! | `IME_SEV_IPC` | `text` = JSON một message IPC v1 (thân frame `textvn-ipc`, không prefix độ dài) |
//! | `IME_SEV_ASSIST_ACTION` | `arg` = `IME_SASSIST_*`, `ch` = chỉ số gợi ý (Select) |
//!
//! Lệnh ra: `kind` = `IME_SCMD_*`; `arg0`/`arg1` = cursor·chars·timer id·ms·vi_on; chữ (preedit,
//! commit, ghost, JSON IPC, mục trợ lý `text\treplace` cách nhau `\n`) nằm ở `arena[text_off..+len]`.
//! Tràn `IME_SESSION_MAX_CMDS` hoặc `IME_SESSION_ARENA` ⇒ `IME_ERR_BUFFER`, `out.count = 0` và
//! session KHÔNG đổi trạng thái (adapter coi như `ForwardKey`).

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

use textvn_session::{
    AssistAction, Caps, Commands, ResetKind, Session, SessionCommand, SessionEvent, TimerId,
};

use crate::{IME_ERR_ABI, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG, IME_OK};

pub const IME_SESSION_ABI_VERSION: u32 = 1;
pub const IME_SESSION_MAX_CMDS: usize = 16;
pub const IME_SESSION_ARENA: usize = 2048;
/// Kết quả không vừa `ime_session_out_v1` (P0-6 §4) — session giữ nguyên trạng thái.
pub const IME_ERR_BUFFER: i32 = -5;

// Caps (bit) — khớp `textvn_session::Caps`.
pub const IME_SCAP_PREEDIT: u32 = 0x01;
pub const IME_SCAP_SURROUNDING_TEXT: u32 = 0x02;
pub const IME_SCAP_DELETE_SURROUNDING: u32 = 0x04;
pub const IME_SCAP_INLINE_GHOST: u32 = 0x08;
pub const IME_SCAP_KEY_UP_EVENTS: u32 = 0x10;
pub const IME_SCAP_FORWARD_KEY: u32 = 0x20;

// Sự kiện.
pub const IME_SEV_KEYDOWN: u32 = 1;
pub const IME_SEV_KEYUP: u32 = 2;
pub const IME_SEV_FOCUS_IN: u32 = 3;
pub const IME_SEV_FOCUS_OUT: u32 = 4;
pub const IME_SEV_CARET_MOVED: u32 = 5;
pub const IME_SEV_SURROUNDING: u32 = 6;
pub const IME_SEV_RESET: u32 = 7;
pub const IME_SEV_TIMER: u32 = 8;
pub const IME_SEV_IPC: u32 = 9;
pub const IME_SEV_ASSIST_ACTION: u32 = 10;

pub const IME_SFOCUS_SECURE: u32 = 0x1;
pub const IME_SFOCUS_TERMINAL: u32 = 0x2;
pub const IME_SRESET_TOOLKIT: u32 = 0;
pub const IME_SRESET_FOCUS: u32 = 1;
pub const IME_SRESET_APP: u32 = 2;
pub const IME_SASSIST_SELECT: u32 = 0;
pub const IME_SASSIST_DISMISS: u32 = 1;
pub const IME_SASSIST_UNDO: u32 = 2;

// Lệnh.
pub const IME_SCMD_SET_PREEDIT: u32 = 1;
pub const IME_SCMD_COMMIT: u32 = 2;
pub const IME_SCMD_DELETE_BEFORE: u32 = 3;
pub const IME_SCMD_FORWARD_KEY: u32 = 4;
pub const IME_SCMD_EAT_KEY: u32 = 5;
pub const IME_SCMD_GHOST: u32 = 6;
pub const IME_SCMD_GHOST_AFTER_KEYUP: u32 = 7;
pub const IME_SCMD_ASSIST_SHOW: u32 = 8;
pub const IME_SCMD_ASSIST_HIDE: u32 = 9;
pub const IME_SCMD_TIMER_START: u32 = 10;
pub const IME_SCMD_TIMER_CANCEL: u32 = 11;
pub const IME_SCMD_IPC: u32 = 12;
pub const IME_SCMD_MODE_INDICATOR: u32 = 13;
/// SF-3 (R2-171/E78): `arg0` = 1 giữ mốc đầu dòng (`ime_reset_with_tail("\n")`) / 0 xoá sạch.
pub const IME_SCMD_ENGINE_RESET: u32 = 14;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct ime_session_config_v1 {
    pub abi_version: u32,
    pub caps: u32,
    pub config_json: *const u8,
    pub config_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct ime_session_event_v1 {
    pub abi_version: u32,
    pub kind: u32,
    pub vk: u32,
    pub ch: u32,
    pub mods: u32,
    pub now_ms: u64,
    pub text: *const u8,
    pub text_len: usize,
    pub arg: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct ime_session_cmd_v1 {
    pub kind: u32,
    pub arg0: u32,
    pub arg1: u32,
    pub text_off: u32,
    pub text_len: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct ime_session_out_v1 {
    pub abi_version: u32,
    pub count: u32,
    pub cmds: [ime_session_cmd_v1; IME_SESSION_MAX_CMDS],
    pub arena: [u8; IME_SESSION_ARENA],
}

impl ime_session_out_v1 {
    /// Bộ đệm rỗng đúng ABI (test/adapter Rust).
    pub fn new() -> Self {
        Self {
            abi_version: IME_SESSION_ABI_VERSION,
            count: 0,
            cmds: [ime_session_cmd_v1::default(); IME_SESSION_MAX_CMDS],
            arena: [0; IME_SESSION_ARENA],
        }
    }
}

impl Default for ime_session_out_v1 {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle opaque cho C (`typedef struct ime_session ime_session;`).
#[allow(non_camel_case_types)]
pub struct ime_session {
    session: Session,
}

fn caps_from_bits(b: u32) -> Caps {
    Caps {
        preedit: b & IME_SCAP_PREEDIT != 0,
        surrounding_text: b & IME_SCAP_SURROUNDING_TEXT != 0,
        delete_surrounding: b & IME_SCAP_DELETE_SURROUNDING != 0,
        inline_ghost: b & IME_SCAP_INLINE_GHOST != 0,
        key_up_events: b & IME_SCAP_KEY_UP_EVENTS != 0,
        forward_key: b & IME_SCAP_FORWARD_KEY != 0,
    }
}

/// Chuỗi UTF-8 mượn từ con trỏ C (`NULL` + 0 = rỗng). Lỗi ⇒ `None`.
///
/// # Safety
/// `ptr` trỏ tới `len` byte đọc được (hoặc `NULL` với `len == 0`).
unsafe fn borrow_str<'a>(ptr: *const u8, len: usize) -> Option<&'a str> {
    if ptr.is_null() {
        return (len == 0).then_some("");
    }
    std::str::from_utf8(unsafe { slice::from_raw_parts(ptr, len) }).ok()
}

/// `ime_session_event_v1` → [`SessionEvent`] (None = kind/tham số lạ).
fn to_event<'a>(ev: &ime_session_event_v1, text: &'a str) -> Option<SessionEvent<'a>> {
    Some(match ev.kind {
        IME_SEV_KEYDOWN => SessionEvent::KeyDown {
            vk: ev.vk,
            ch: ev.ch,
            mods: ev.mods,
            now_ms: ev.now_ms,
        },
        IME_SEV_KEYUP => SessionEvent::KeyUp {
            vk: ev.vk,
            mods: ev.mods,
            now_ms: ev.now_ms,
        },
        IME_SEV_FOCUS_IN => SessionEvent::FocusIn {
            app_id: text,
            field_role: ev.arg,
            secure: ev.mods & IME_SFOCUS_SECURE != 0,
            terminal: ev.mods & IME_SFOCUS_TERMINAL != 0,
        },
        IME_SEV_FOCUS_OUT => SessionEvent::FocusOut,
        IME_SEV_CARET_MOVED => SessionEvent::CaretMoved,
        IME_SEV_SURROUNDING => SessionEvent::Surrounding { before: text },
        IME_SEV_RESET => SessionEvent::Reset {
            kind: match ev.arg {
                IME_SRESET_TOOLKIT => ResetKind::Toolkit,
                IME_SRESET_FOCUS => ResetKind::Focus,
                IME_SRESET_APP => ResetKind::App,
                _ => return None,
            },
            now_ms: ev.now_ms,
        },
        IME_SEV_TIMER => SessionEvent::Timer {
            id: TimerId(ev.arg),
            now_ms: ev.now_ms,
        },
        IME_SEV_IPC => {
            // Thân frame `textvn-ipc` (JSON) — dựng lại frame có prefix để dùng ĐÚNG codec chung.
            let mut frame = Vec::with_capacity(4 + text.len());
            frame.extend_from_slice(&u32::try_from(text.len()).ok()?.to_le_bytes());
            frame.extend_from_slice(text.as_bytes());
            SessionEvent::Ipc(textvn_ipc::decode_exact_frame(&frame).ok()?)
        }
        IME_SEV_ASSIST_ACTION => SessionEvent::AssistAction {
            action: match ev.arg {
                IME_SASSIST_SELECT => AssistAction::Select {
                    index: u16::try_from(ev.ch).ok()?,
                },
                IME_SASSIST_DISMISS => AssistAction::Dismiss,
                IME_SASSIST_UNDO => AssistAction::Undo,
                _ => return None,
            },
        },
        _ => return None,
    })
}

/// Ghi chuỗi lệnh vào bộ đệm C. Tràn ⇒ `Err(IME_ERR_BUFFER)` và `out.count = 0`.
pub fn encode_commands(cmds: &[SessionCommand], out: &mut ime_session_out_v1) -> Result<(), i32> {
    out.count = 0;
    if cmds.len() > IME_SESSION_MAX_CMDS {
        return Err(IME_ERR_BUFFER);
    }
    let mut used = 0usize;
    let mut put = |out: &mut ime_session_out_v1, s: &str| -> Result<(u32, u32), i32> {
        let b = s.as_bytes();
        if used + b.len() > IME_SESSION_ARENA {
            return Err(IME_ERR_BUFFER);
        }
        out.arena[used..used + b.len()].copy_from_slice(b);
        let off = used as u32;
        used += b.len();
        Ok((off, b.len() as u32))
    };
    for (i, c) in cmds.iter().enumerate() {
        let mut rec = ime_session_cmd_v1::default();
        match c {
            SessionCommand::SetPreedit { text, cursor } => {
                rec.kind = IME_SCMD_SET_PREEDIT;
                rec.arg0 = u32::from(*cursor);
                (rec.text_off, rec.text_len) = put(out, text)?;
            }
            SessionCommand::Commit { text } => {
                rec.kind = IME_SCMD_COMMIT;
                (rec.text_off, rec.text_len) = put(out, text)?;
            }
            SessionCommand::DeleteBefore { chars } => {
                rec.kind = IME_SCMD_DELETE_BEFORE;
                rec.arg0 = u32::from(*chars);
            }
            SessionCommand::ForwardKey => rec.kind = IME_SCMD_FORWARD_KEY,
            SessionCommand::EatKey => rec.kind = IME_SCMD_EAT_KEY,
            SessionCommand::Ghost { text } => {
                rec.kind = IME_SCMD_GHOST;
                (rec.text_off, rec.text_len) = put(out, text)?;
            }
            SessionCommand::GhostAfterKeyUp => rec.kind = IME_SCMD_GHOST_AFTER_KEYUP,
            SessionCommand::AssistShow { items } => {
                rec.kind = IME_SCMD_ASSIST_SHOW;
                rec.arg0 = items.len() as u32;
                let joined: Vec<String> = items
                    .iter()
                    .map(|it| format!("{}\t{}", it.text, it.replace))
                    .collect();
                (rec.text_off, rec.text_len) = put(out, &joined.join("\n"))?;
            }
            SessionCommand::AssistHide => rec.kind = IME_SCMD_ASSIST_HIDE,
            SessionCommand::TimerStart { id, ms } => {
                rec.kind = IME_SCMD_TIMER_START;
                rec.arg0 = id.0;
                rec.arg1 = *ms;
            }
            SessionCommand::TimerCancel { id } => {
                rec.kind = IME_SCMD_TIMER_CANCEL;
                rec.arg0 = id.0;
            }
            SessionCommand::Ipc(msg) => {
                rec.kind = IME_SCMD_IPC;
                let frame = textvn_ipc::encode_frame(msg).map_err(|_| IME_ERR_BUFFER)?;
                let json = std::str::from_utf8(&frame[4..]).map_err(|_| IME_ERR_INTERNAL)?;
                (rec.text_off, rec.text_len) = put(out, json)?;
            }
            SessionCommand::ModeIndicator { vi_on } => {
                rec.kind = IME_SCMD_MODE_INDICATOR;
                rec.arg0 = u32::from(*vi_on);
            }
            SessionCommand::EngineReset { line_start } => {
                rec.kind = IME_SCMD_ENGINE_RESET;
                rec.arg0 = u32::from(*line_start);
            }
        }
        out.cmds[i] = rec;
    }
    out.count = cmds.len() as u32;
    Ok(())
}

/// Tạo session. `cfg->config_json` dành cho SF-2+ (session khung chưa đọc cấu hình).
pub(crate) fn ime_session_new_impl(
    cfg: *const ime_session_config_v1,
    out: *mut *mut ime_session,
) -> i32 {
    if cfg.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    unsafe { *out = std::ptr::null_mut() };
    let cfg = unsafe { &*cfg };
    if cfg.abi_version != IME_SESSION_ABI_VERSION {
        return IME_ERR_ABI;
    }
    if unsafe { borrow_str(cfg.config_json, cfg.config_len) }.is_none() {
        return IME_ERR_INVALID_ARG;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(ime_session {
            session: Session::new(caps_from_bits(cfg.caps)),
        }))
    })) {
        Ok(p) => {
            unsafe { *out = p };
            IME_OK
        }
        Err(_) => IME_ERR_INTERNAL,
    }
}

pub(crate) fn ime_session_free_impl(s: *mut ime_session) {
    if s.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe { drop(Box::from_raw(s)) }));
}

/// Một sự kiện ⇒ chuỗi lệnh trong `out`. Lỗi ⇒ `out->count = 0` (adapter coi như ForwardKey).
pub(crate) fn ime_session_event_impl(
    s: *mut ime_session,
    ev: *const ime_session_event_v1,
    out: *mut ime_session_out_v1,
) -> i32 {
    if s.is_null() || ev.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let (s, ev, out) = unsafe { (&mut *s, &*ev, &mut *out) };
    out.count = 0;
    if ev.abi_version != IME_SESSION_ABI_VERSION || out.abi_version != IME_SESSION_ABI_VERSION {
        return IME_ERR_ABI;
    }
    let Some(text) = (unsafe { borrow_str(ev.text, ev.text_len) }) else {
        return IME_ERR_INVALID_ARG;
    };
    let Some(event) = to_event(ev, text) else {
        return IME_ERR_INVALID_ARG;
    };
    // Ảnh chụp để hoàn nguyên khi tràn bộ đệm hoặc panic (P0-6 §4: session không đổi trạng thái).
    // `Session` còn `Copy` khi là khung rỗng (SF-1a); SF-2+ thêm trạng thái sẽ bỏ `Copy` — ảnh chụp
    // phải là bản sao thật, không phụ thuộc việc kiểu đó có `Copy` hay không.
    #[allow(clippy::clone_on_copy)]
    let before = s.session.clone();
    let res = catch_unwind(AssertUnwindSafe(|| {
        let cmds: Commands = s.session.event(event);
        encode_commands(cmds.as_slice(), out)
    }));
    match res {
        Ok(Ok(())) => IME_OK,
        Ok(Err(code)) => {
            s.session = before;
            out.count = 0;
            code
        }
        Err(_) => {
            s.session = before;
            out.count = 0;
            IME_ERR_INTERNAL
        }
    }
}

/// Nạp lại cấu hình (SF-2+ dùng). Khung SF-1a chỉ kiểm UTF-8 hợp lệ.
pub(crate) fn ime_session_reload_config_impl(
    s: *mut ime_session,
    json: *const u8,
    len: usize,
) -> i32 {
    if s.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    match unsafe { borrow_str(json, len) } {
        Some(_) => IME_OK,
        None => IME_ERR_INVALID_ARG,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ime_session_event, ime_session_free, ime_session_new};
    use textvn_session::AssistItem;

    fn ev(kind: u32) -> ime_session_event_v1 {
        ime_session_event_v1 {
            abi_version: IME_SESSION_ABI_VERSION,
            kind,
            vk: 0x41,
            ch: 'a' as u32,
            mods: 0,
            now_ms: 1,
            text: std::ptr::null(),
            text_len: 0,
            arg: 0,
        }
    }

    fn new_session() -> *mut ime_session {
        let cfg = ime_session_config_v1 {
            abi_version: IME_SESSION_ABI_VERSION,
            caps: IME_SCAP_PREEDIT | IME_SCAP_FORWARD_KEY,
            config_json: std::ptr::null(),
            config_len: 0,
        };
        let mut s = std::ptr::null_mut();
        assert_eq!(ime_session_new(&cfg, &mut s), IME_OK);
        s
    }

    #[test]
    fn sf_1b_keydown_qua_abi_ra_forward_key() {
        let s = new_session();
        let mut out = ime_session_out_v1::new();
        assert_eq!(ime_session_event(s, &ev(IME_SEV_KEYDOWN), &mut out), IME_OK);
        assert_eq!(out.count, 1);
        assert_eq!(out.cmds[0].kind, IME_SCMD_FORWARD_KEY);
        // Sự kiện khác ⇒ rỗng (hợp đồng khung SF-1a).
        assert_eq!(
            ime_session_event(s, &ev(IME_SEV_FOCUS_OUT), &mut out),
            IME_OK
        );
        assert_eq!(out.count, 0);
        ime_session_free(s);
    }

    #[test]
    fn sf_1b_tham_so_sai_va_abi_lech() {
        let s = new_session();
        let mut out = ime_session_out_v1::new();
        let mut bad = ev(IME_SEV_KEYDOWN);
        bad.abi_version = 99;
        assert_eq!(ime_session_event(s, &bad, &mut out), IME_ERR_ABI);
        assert_eq!(
            ime_session_event(s, &ev(999), &mut out),
            IME_ERR_INVALID_ARG
        );
        let mut rst = ev(IME_SEV_RESET);
        rst.arg = 7;
        assert_eq!(ime_session_event(s, &rst, &mut out), IME_ERR_INVALID_ARG);
        let bytes = [0xffu8, 0xfe];
        let mut utf = ev(IME_SEV_SURROUNDING);
        utf.text = bytes.as_ptr();
        utf.text_len = 2;
        assert_eq!(
            ime_session_event(s, &utf, &mut out),
            IME_ERR_INVALID_ARG,
            "UTF-8 sai"
        );
        assert_eq!(
            ime_session_event(std::ptr::null_mut(), &ev(IME_SEV_KEYDOWN), &mut out),
            IME_ERR_INVALID_ARG
        );
        ime_session_free(s);
        ime_session_free(std::ptr::null_mut());
    }

    #[test]
    fn sf_1b_tran_max_cmds_va_arena_tra_err_buffer() {
        let mut out = ime_session_out_v1::new();
        let many = vec![SessionCommand::AssistHide; IME_SESSION_MAX_CMDS + 1];
        assert_eq!(encode_commands(&many, &mut out), Err(IME_ERR_BUFFER));
        assert_eq!(out.count, 0);
        let big = vec![SessionCommand::Commit {
            text: "đ".repeat(IME_SESSION_ARENA),
        }];
        assert_eq!(encode_commands(&big, &mut out), Err(IME_ERR_BUFFER));
        assert_eq!(out.count, 0);
        // Vừa khít arena ⇒ đạt.
        let fit = vec![SessionCommand::Commit {
            text: "a".repeat(IME_SESSION_ARENA),
        }];
        assert_eq!(encode_commands(&fit, &mut out), Ok(()));
        assert_eq!(
            (out.count, out.cmds[0].text_len),
            (1, IME_SESSION_ARENA as u32)
        );
    }

    #[test]
    fn sf_1b_ma_hoa_moi_lenh_va_arena_utf8() {
        let cmds = vec![
            SessionCommand::DeleteBefore { chars: 2 },
            SessionCommand::Commit {
                text: "được".into(),
            },
            SessionCommand::SetPreedit {
                text: String::new(),
                cursor: 0,
            },
            SessionCommand::Ghost {
                text: " thêm".into(),
            },
            SessionCommand::AssistShow {
                items: vec![AssistItem {
                    text: "Sửa".into(),
                    replace: "sửa".into(),
                }],
            },
            SessionCommand::TimerStart {
                id: TimerId(7),
                ms: 1000,
            },
            SessionCommand::ModeIndicator { vi_on: true },
            SessionCommand::EatKey,
        ];
        let mut out = ime_session_out_v1::new();
        assert_eq!(encode_commands(&cmds, &mut out), Ok(()));
        assert_eq!(out.count, 8);
        let text = |c: &ime_session_cmd_v1| {
            std::str::from_utf8(&out.arena[c.text_off as usize..(c.text_off + c.text_len) as usize])
                .unwrap()
                .to_string()
        };
        assert_eq!(
            (out.cmds[0].kind, out.cmds[0].arg0),
            (IME_SCMD_DELETE_BEFORE, 2)
        );
        assert_eq!(text(&out.cmds[1]), "được");
        assert_eq!(
            (out.cmds[2].kind, out.cmds[2].text_len),
            (IME_SCMD_SET_PREEDIT, 0)
        );
        assert_eq!(text(&out.cmds[3]), " thêm");
        assert_eq!(text(&out.cmds[4]), "Sửa\tsửa");
        assert_eq!((out.cmds[5].arg0, out.cmds[5].arg1), (7, 1000));
        assert_eq!(
            (out.cmds[6].kind, out.cmds[6].arg0),
            (IME_SCMD_MODE_INDICATOR, 1)
        );
        assert_eq!(out.cmds[7].kind, IME_SCMD_EAT_KEY);
    }

    #[test]
    fn sf_1b_ma_hoa_5_lenh_con_lai_du_13_loai() {
        // Cùng test trên: đủ 13 IME_SCMD_* (P0-6 §3) — không loại lệnh nào thiếu bộ mã hoá.
        let cmds = vec![
            SessionCommand::ForwardKey,
            SessionCommand::GhostAfterKeyUp,
            SessionCommand::AssistHide,
            SessionCommand::TimerCancel { id: TimerId(9) },
            SessionCommand::Ipc(textvn_ipc::Message::Ping),
        ];
        let mut out = ime_session_out_v1::new();
        assert_eq!(encode_commands(&cmds, &mut out), Ok(()));
        let kinds: Vec<u32> = out.cmds[..out.count as usize]
            .iter()
            .map(|c| c.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                IME_SCMD_FORWARD_KEY,
                IME_SCMD_GHOST_AFTER_KEYUP,
                IME_SCMD_ASSIST_HIDE,
                IME_SCMD_TIMER_CANCEL,
                IME_SCMD_IPC
            ]
        );
        assert_eq!(out.cmds[3].arg0, 9);
        // JSON IPC trong arena = đúng thân frame của codec chung (G22: một bộ mã hoá).
        let c = &out.cmds[4];
        let json = &out.arena[c.text_off as usize..(c.text_off + c.text_len) as usize];
        let frame = textvn_ipc::encode_frame(&textvn_ipc::Message::Ping).unwrap();
        assert_eq!(json, &frame[4..]);
    }

    #[test]
    fn sf_1b_su_kien_ipc_qua_codec_chung() {
        let s = new_session();
        let mut out = ime_session_out_v1::new();
        let msg = textvn_ipc::Message::Ping;
        let frame = textvn_ipc::encode_frame(&msg).unwrap();
        let body = &frame[4..];
        let mut e = ev(IME_SEV_IPC);
        e.text = body.as_ptr();
        e.text_len = body.len();
        assert_eq!(ime_session_event(s, &e, &mut out), IME_OK);
        let junk = b"{not json";
        e.text = junk.as_ptr();
        e.text_len = junk.len();
        assert_eq!(ime_session_event(s, &e, &mut out), IME_ERR_INVALID_ARG);
        ime_session_free(s);
    }
}
