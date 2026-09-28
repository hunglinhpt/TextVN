// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfKeyEventSink` của TextVN TSF (WIN-011/014/015).
//!
//! Hai pha `OnTestKeyDown`/`OnKeyDown`: app như Word/Win32 Edit gọi cả hai, còn
//! Notepad (Win11)/Chrome/VS Code chỉ gọi `OnKeyDown` (`docs/specs/tsf-spike.md` #5).
//! Phím được xử lý ĐÚNG MỘT LẦN ở pha tới trước; nếu pha test đã ăn phím thì
//! `OnKeyDown` cùng VK chỉ xác nhận `TRUE` (test TRUE ⇒ app không gửi lại phím,
//! nên không bao giờ "test TRUE rồi keydown bỏ rơi" — finding E6).

#[cfg(windows)]
use std::cell::Cell;
#[cfg(windows)]
use std::rc::Rc;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::{LPARAM, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::compose::{is_modifier_vk, vk as vkc, KeyKind};
#[cfg(windows)]
use crate::edit_session::{EndCompositionSession, KeyEditSession, TsfShared};
#[cfg(windows)]
use textvn_ffi::{ACTION_PASS, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_SUPER};

#[cfg(windows)]
#[implement(ITfKeyEventSink)]
pub struct KeySink {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
}

#[cfg(windows)]
impl KeySink {
    pub fn new(shared: Rc<TsfShared>) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
        }
    }
}

#[cfg(windows)]
impl ITfKeyEventSink_Impl for KeySink_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        if self.shared.pending_eaten_vk.get() == Some(vk) {
            return Ok(true.into());
        }
        self.shared.pending_eaten_vk.set(None);
        let eaten = guarded(|| handle_key(&self.shared, pic, vk, lparam));
        if eaten {
            self.shared.pending_eaten_vk.set(Some(vk));
        }
        Ok(eaten.into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        if self.shared.pending_eaten_vk.take() == Some(vk) {
            return Ok(true.into());
        }
        Ok(guarded(|| handle_key(&self.shared, pic, vk, lparam)).into())
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn OnPreservedKey(&self, pic: Ref<'_, ITfContext>, rguid: *const GUID) -> Result<BOOL> {
        if rguid.is_null() {
            return Ok(false.into());
        }
        // SAFETY: rguid do TSF cấp, đã kiểm non-null.
        if unsafe { *rguid } != crate::guids::GUID_PRESERVED_TOGGLE {
            return Ok(false.into());
        }
        // WIN-015: chỉ xử lý ở đây (không xử lý lại trong OnKeyDown → không toggle 2 lần).
        guarded(|| {
            end_composition(&self.shared, pic.ok().ok());
            self.shared.reset_engine();
            self.shared.ipc.toggle_global();
            true
        });
        Ok(true.into())
    }
}

/// Mọi đường phím: panic không được thoát qua ranh giới COM (abort app).
#[cfg(windows)]
fn guarded(f: impl FnOnce() -> bool) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(false)
}

/// Xử lý 1 phím, trả `eaten`. Mọi nhánh lỗi → `false` (phím tới app nguyên vẹn).
#[cfg(windows)]
fn handle_key(shared: &Rc<TsfShared>, pic: Ref<'_, ITfContext>, vk: u32, lparam: LPARAM) -> bool {
    if is_modifier_vk(vk) {
        return false;
    }
    let ctx = pic.ok().ok();
    let mods = active_modifiers();

    // Chord hệ thống (B6), phím do phần mềm bơm vào (VK_PACKET) hoặc IME khác đã
    // xử lý: commit từ đang gõ rồi để app nhận phím.
    if mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0 || vk == vkc::PACKET || vk == vkc::PROCESSKEY {
        end_composition(shared, ctx);
        shared.reset_engine();
        return false;
    }

    // Reload config xóa từ trong engine → chỉ làm ở ranh giới từ.
    if !shared.is_composing() {
        shared.sync_config();
    }
    if !shared.ipc.is_enabled() {
        end_composition(shared, ctx);
        shared.reset_engine();
        return false;
    }

    let key = KeyKind::classify(vk, translate_key(vk, lparam));
    if !shared.is_composing() && !key.needs_session_when_idle() {
        // Backspace/Esc/điều hướng khi không composing: chỉ cho engine dọn trạng thái.
        if let Ok(mut thread) = shared.thread.try_borrow_mut() {
            let passed = matches!(
                thread.engine.key_event_raw(vk, key.engine_ch(), mods),
                Ok(r) if r.action == ACTION_PASS
            );
            if !passed {
                thread.engine.reset();
            }
        }
        return false;
    }

    let Some(ctx) = ctx else {
        return false;
    };
    let result = Rc::new(Cell::new(None));
    let session: ITfEditSession =
        KeyEditSession::new(shared.clone(), ctx.clone(), vk, mods, key, result.clone()).into();
    // SAFETY: ctx hợp lệ; session đồng bộ để biết `eaten` trước khi trả lời TSF.
    let hr = unsafe { ctx.RequestEditSession(shared.tid, &session, TF_ES_SYNC | TF_ES_READWRITE) };
    match (hr, result.get()) {
        (Ok(code), Some(eaten)) if code.is_ok() => eaten,
        // Không được cấp lock đồng bộ: engine chưa thấy phím → không lệch buffer.
        _ => false,
    }
}

/// Kết thúc composition đang mở (nếu có) — ưu tiên đồng bộ để app nhận phím sau
/// khi text đã commit (B2); không được thì xin session bất đồng bộ.
#[cfg(windows)]
pub fn end_composition(shared: &TsfShared, fallback_ctx: Option<&ITfContext>) {
    let Some(comp) = shared.take_composition() else {
        return;
    };
    // SAFETY: composition thuộc TIP; context lấy từ chính range của nó.
    unsafe {
        let ctx = comp
            .GetRange()
            .and_then(|r| r.GetContext())
            .ok()
            .or_else(|| fallback_ctx.cloned());
        let Some(ctx) = ctx else {
            return;
        };
        let session: ITfEditSession = EndCompositionSession::new(comp).into();
        let sync = ctx.RequestEditSession(shared.tid, &session, TF_ES_SYNC | TF_ES_READWRITE);
        if !matches!(sync, Ok(code) if code.is_ok()) {
            let _ =
                ctx.RequestEditSession(shared.tid, &session, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE);
        }
    }
}

#[cfg(windows)]
fn active_modifiers() -> u32 {
    let down = |vk: VIRTUAL_KEY| {
        // SAFETY: GetKeyState đọc trạng thái phím của message queue thread hiện tại.
        (unsafe { GetKeyState(vk.0 as i32) } as u16 & 0x8000) != 0
    };
    let mut mods = 0u32;
    if down(VK_SHIFT) {
        mods |= MOD_SHIFT;
    }
    if down(VK_CONTROL) {
        mods |= MOD_CTRL;
    }
    if down(VK_MENU) {
        mods |= MOD_ALT;
    }
    if down(VK_LWIN) || down(VK_RWIN) {
        mods |= MOD_SUPER;
    }
    mods
}

/// VK → ký tự theo layout bàn phím hiện hành của thread (hỗ trợ layout khác US).
/// `wFlags = 0x4` (Win10 1607+): KHÔNG đổi trạng thái dead-key của app. Trả 0 khi
/// phím không sinh đúng một ký tự (bản cũ trả `vk` → Delete thành '.', F1 thành 'p').
#[cfg(windows)]
fn translate_key(vk: u32, lparam: LPARAM) -> u32 {
    const TOUNICODE_NO_STATE_CHANGE: u32 = 0x4;
    let raw = lparam.0 as u32;
    let mut scan = (raw >> 16) & 0xFF;
    if raw & (1 << 24) != 0 {
        scan |= 0xE000;
    }
    let mut state = [0u8; 256];
    let mut buf = [0u16; 8];
    // SAFETY: buffer cố định trên stack; layout của chính thread UI.
    let n = unsafe {
        if GetKeyboardState(&mut state).is_err() {
            return 0;
        }
        ToUnicodeEx(
            vk,
            scan,
            &state,
            &mut buf,
            TOUNICODE_NO_STATE_CHANGE,
            Some(GetKeyboardLayout(0)),
        )
    };
    match n {
        1 => u32::from(buf[0]),
        // Ký tự ngoài BMP (cặp surrogate); hai ký tự rời (ligature) → không nhận.
        2 if (0xD800..0xDC00).contains(&buf[0]) => char::decode_utf16(buf[..2].iter().copied())
            .next()
            .and_then(|r| r.ok())
            .map_or(0, u32::from),
        _ => 0,
    }
}
