// SPDX-License-Identifier: GPL-3.0-or-later
//! Kiểm tra ngữ pháp **vùng bôi đen** (vòng 30 — yêu cầu chủ repo 2026-10-24):
//! người dùng bôi đen một câu đang soạn thảo → trợ lý check + hiện popup
//! (tổng lỗi + nút "Sửa tất cả") neo cạnh vùng chọn; bấm nút thay toàn bộ
//! vùng bằng câu đã sửa.
//!
//! Phân biệt "bôi đen để đọc/copy" vs "đang soạn thảo":
//! - Chỉ kích hoạt khi selection được tạo/mở rộng bằng **bàn phím**
//!   (Shift+←/→/↑/↓/Home/End — key-up không giữ Ctrl/Win) — người copy
//!   bôi bằng chuột hoặc bấm Ctrl+A/C/X: chuột không qua key sink,
//!   Ctrl+* bị loại theo modifier.
//! - Debounce 500 ms sau phím cuối — gõ dở Shift+→ liên tục không pop.
//! - Chỉ check khi trợ lý Mode 1 bật (highlight/autocorrect), vùng
//!   3..256 ký tự và có chữ cái (không bắt bôi đen số/ảnh).
//!
//! Sửa vùng chọn dùng `SetText` trên đúng range selection (hành vi gõ
//! thường — người dùng đang chọn để thay) — KHÔNG phải engine
//! `SelectionReplace` (D22); undo thuộc app (Ctrl+Z của Notepad/Word).

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use windows::core::{implement, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::TextServices::{
    ITfContext, ITfEditSession, ITfEditSession_Impl, ITfRange, TF_ES_ASYNCDONTCARE, TF_ES_READ,
    TF_ES_READWRITE,
};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::edit_session::{selection_range, TsfShared};
use crate::overlay::Anchor;

/// Timer ID của panel (không đụng TIMER_WATCHDOG của overlay).
/// Debounce sau phím selection cuối.
const DEBOUNCE_MS: u32 = 500;
/// Giới hạn vùng check (ký tự UTF-16). Bằng `SENTENCE_MAX_CHARS` của engine
/// (256 — audit v30 F1: check_text cắt đuôi 256 nên vùng dài hơn luôn làm
/// nút "Sửa tất cả" chết vì precondition đọc-đầu ≠ expect-đuôi).
const MIN_SELECTION_CHARS: usize = 3;
pub const MAX_SELECTION_CHARS: usize = 256;
/// Timer ID — overlay WndProc so khớp hằng này (audit F7: không hardcode 2 chỗ).
pub(crate) const TIMER_SELECTION_CHECK: usize = 0x5EC1;

// (Weak<TsfShared>, ctx) chờ debounce — thread UI của app (TSF apartment).
thread_local! {
    static PENDING: RefCell<Option<(Weak<TsfShared>, ITfContext)>> = const { RefCell::new(None) };
}

// Overlay hwnd (để SetTimer) — đăng ký từ `overlay::get_or_create_hwnd`.
thread_local! {
    static TIMER_HWND: RefCell<isize> = const { RefCell::new(0) };
}

/// Ghi nhận hwnd dùng cho timer debounce (gọi khi overlay tạo hwnd).
#[cfg(windows)]
pub fn register_timer_hwnd(hwnd: isize) {
    TIMER_HWND.with(|c| *c.borrow_mut() = hwnd);
}

/// Huỷ timer selection-check đang chờ và xoá PENDING (khi gõ phím mới, đổi
/// focus hoặc huỷ TIP/overlay).
#[cfg(windows)]
pub fn cancel_pending() {
    let hwnd = TIMER_HWND.with(|c| *c.borrow());
    if hwnd != 0 {
        unsafe {
            let _ = KillTimer(
                Some(windows::Win32::Foundation::HWND(
                    hwnd as *mut std::ffi::c_void,
                )),
                TIMER_SELECTION_CHECK,
            );
        }
    }
    PENDING.with(|p| *p.borrow_mut() = None);
}

/// Key-up có khả năng kết thúc một lựa chọn bàn phím không? (không Ctrl/Win —
/// Ctrl+A/C/X/V là chọn-all/copy/cut/paste: bỏ qua để không nhầm "để copy").
#[cfg(windows)]
fn is_selection_key_up(vk: u32, mods: u32) -> bool {
    const MOD_CTRL: u32 = 0x2; // keymap::MOD_CTRL
    const MOD_WIN: u32 = 0x8; // bàn Win
    if mods & (MOD_CTRL | MOD_WIN) != 0 {
        return false;
    }
    const MOD_ALT: u32 = 0x4; // keymap::MOD_ALT (Alt+← = Back của browser)
    if mods & MOD_ALT != 0 {
        return false;
    }
    matches!(
        vk,
        0x10 |          // Shift (nhả sau Shift+←)
        0x25
            ..=0x28 |   // ← ↑ ↓ →
        0x21 | 0x22 |   // PgUp/PgDn
        0x23 | 0x24 // End/Home
    ) || (vk >= 'A' as u32 && vk <= 'Z' as u32 && mods & 0x1 != 0) // Shift+letter (bôi đầu từ)
}

/// Nhận key-up từ `OnTestKeyUp`/`OnKeyDown`-up path: lên lịch debounce check.
#[cfg(windows)]
pub fn on_key_up(shared: &Rc<TsfShared>, ctx: Option<&ITfContext>, vk: u32, mods: u32) {
    let Some(ctx) = ctx else { return };
    if !is_selection_key_up(vk, mods) {
        return;
    }
    // Chỉ khi trợ lý đang bật (Mode 1 hiển thị) — không đụng gì khi tắt.
    if !shared.is_assist_display_enabled() {
        return;
    }
    let mut hwnd = TIMER_HWND.with(|c| *c.borrow());
    // F4 (audit v30): hwnd có thể chưa được tạo (lần chọn đầu tiên trong phiên)
    // hoặc đã bị DestroyWindow khi TIP deactivate — đảm bảo tạo/làm mới hwnd.
    let alive = hwnd != 0
        && unsafe {
            windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(
                windows::Win32::Foundation::HWND(hwnd as *mut std::ffi::c_void),
            ))
            .as_bool()
        };
    if !alive {
        hwnd = shared.overlay.ensure_timer_hwnd();
        if hwnd == 0 {
            TIMER_HWND.with(|c| *c.borrow_mut() = 0);
            PENDING.with(|p| *p.borrow_mut() = None);
            return;
        }
    }
    PENDING.with(|p| {
        *p.borrow_mut() = Some((Rc::downgrade(shared), ctx.clone()));
    });
    // SAFETY: hwnd thuộc thread này (overlay của cùng TsfShared).
    unsafe {
        let _ = SetTimer(
            Some(windows::Win32::Foundation::HWND(
                hwnd as *mut std::ffi::c_void,
            )),
            TIMER_SELECTION_CHECK,
            DEBOUNCE_MS,
            None,
        );
    }
}

/// Timer fire → chạy check (gọi từ `overlay_wnd_proc` WM_TIMER).
#[cfg(windows)]
pub fn on_timer() {
    let hwnd = TIMER_HWND.with(|c| *c.borrow());
    if hwnd != 0 {
        // SAFETY: một lần fire là đủ — hủy timer.
        unsafe {
            let _ = KillTimer(
                Some(windows::Win32::Foundation::HWND(
                    hwnd as *mut std::ffi::c_void,
                )),
                TIMER_SELECTION_CHECK,
            );
        }
    }
    let Some((weak, ctx)) = PENDING.with(|p| p.borrow_mut().take()) else {
        return;
    };
    let Some(shared) = weak.upgrade() else {
        return;
    };
    let session: ITfEditSession = SelectionCheckSession::new(shared.clone(), ctx.clone()).into();
    // SAFETY: ctx hợp lệ trong callback TSF của thread này.
    let hr =
        unsafe { ctx.RequestEditSession(shared.tid, &session, TF_ES_ASYNCDONTCARE | TF_ES_READ) };
    let _ = hr;
}

/// Session đọc selection + check + hiện popup (vòng 30).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct SelectionCheckSession {
    _guard: crate::class::ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
}

#[cfg(windows)]
impl SelectionCheckSession {
    pub fn new(shared: Rc<TsfShared>, ctx: ITfContext) -> Self {
        Self {
            _guard: crate::class::ObjGuard::new(),
            shared,
            ctx,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for SelectionCheckSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run(ec)));
        Ok(())
    }
}

#[cfg(windows)]
impl SelectionCheckSession {
    fn run(&self, ec: u32) -> bool {
        if !self.shared.is_assist_display_enabled() {
            return false;
        }
        let (auto_cap, app_disabled) = self
            .shared
            .thread
            .try_borrow()
            .map(|t| {
                (
                    t.auto_capitalize(),
                    t.assist_app_disabled(&t.field_context().app_id),
                )
            })
            .unwrap_or((true, false));
        if app_disabled {
            return false;
        }
        unsafe {
            let Some(sel) = selection_range(&self.ctx, ec) else {
                return false;
            };
            // Vùng chọn rỗng (caret) = không bôi đen → thôi.
            let Ok(empty) = sel.IsEmpty(ec) else {
                return false;
            };
            if empty.as_bool() {
                return false;
            }
            // Security gate S3 + read-only: tuyệt đối không đọc/sửa ô mật khẩu,
            // PIN hoặc trường chỉ-đọc.
            let signals = crate::edit_session::read_field_signals(&self.ctx, ec, &sel);
            if signals.read_only
                || crate::classify_tsf_field(&signals).security
                    != textvn_field_detect::SecurityState::NonSecure
            {
                return false;
            }
            // Đọc text vùng chọn với buffer chừa 2 slot dự phòng: nếu
            // `fetched` chạm hết buffer ⇒ vùng DÀI HƠN MAX — check nửa câu
            // là sai ngữ nghĩa → bỏ (guard cắt cụt, audit v30).
            let cap = MAX_SELECTION_CHARS + 2;
            let mut buf = vec![0u16; cap];
            let mut fetched = 0u32;
            if sel.GetText(ec, 0, &mut buf, &mut fetched).is_err() || fetched == 0 {
                return false;
            }
            let n = (fetched as usize).min(buf.len());
            if n > MAX_SELECTION_CHARS {
                return false; // vùng vượt MAX — không popup nửa câu
            }
            let units: Vec<u16> = buf[..n].to_vec();
            if units.len() < MIN_SELECTION_CHARS {
                return false; // quá ngắn
            }
            let Some(text) = String::from_utf16(&units).ok() else {
                return false;
            };
            if !text.chars().any(|c| c.is_alphabetic()) {
                return false; // toàn số/ký hiệu — không phải câu
            }
            // Check với ngữ nghĩa FRAGMENT (vòng 32, audit A7): vùng chọn là
            // đoạn giữa câu — token 0 không phải đầu câu ⇒ L6 tại span 0 là FP.
            let Some(check) = crate::assist::check_text_fragment(&text, auto_cap) else {
                // Vòng 32: vùng chọn sạch mà vẫn còn popup cũ (neo theo caret/
                // câu đã qua) = popup treo sai ngữ cảnh — ẩn cho đồng bộ.
                crate::edit_session::hide_assist_overlay(&self.shared);
                return false; // câu sạch — không popup (D11)
            };
            // Neo popup cạnh VÙNG CHỌN (GetTextExt trên range chọn).
            let anchor = self.selection_anchor(&sel, ec);
            crate::edit_session::show_assist_overlay_for_selection(
                &self.shared,
                &self.ctx,
                ec,
                &check,
                sel.clone(),
                anchor,
            );
            true
        }
    }

    /// Neo cạnh vùng chọn: `GetTextExt` trên range (nhiều app trả về rect
    /// bao vùng chọn — lấy góc dưới phải). Fallback: anchor caret thường.
    unsafe fn selection_anchor(&self, sel: &ITfRange, ec: u32) -> Option<Anchor> {
        get_selection_anchor(&self.ctx, sel, ec)
    }
}

/// Lấy anchor cạnh vùng chọn bôi đen.
pub(crate) unsafe fn get_selection_anchor(
    ctx: &ITfContext,
    sel: &ITfRange,
    ec: u32,
) -> Option<Anchor> {
    if let Ok(view) = ctx.GetActiveView() {
        let mut rc = RECT::default();
        let mut clipped = windows::core::BOOL::default();
        if view.GetTextExt(ec, sel, &mut rc, &mut clipped).is_ok()
            && rc.right > rc.left
            && rc.bottom > rc.top
        {
            return Some(Anchor::new(rc.left, rc.top, rc.right, rc.bottom));
        }
    }
    None
}

/// Session ÁP "Sửa tất cả" lên vùng chọn (vòng 30): SetText câu đã sửa vào
/// đúng range selection (precondition: text vùng chọn VẪN bằng câu đã check).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct ApplySelectionFixSession {
    _guard: crate::class::ObjGuard,
    shared: std::rc::Rc<TsfShared>,
    ctx: ITfContext,
    range: ITfRange,
    expect: String,
    fixed: String,
}

#[cfg(windows)]
impl ApplySelectionFixSession {
    pub fn new(
        shared: std::rc::Rc<TsfShared>,
        ctx: ITfContext,
        range: ITfRange,
        expect: String,
        fixed: String,
    ) -> Self {
        Self {
            _guard: crate::class::ObjGuard::new(),
            shared,
            ctx,
            range,
            expect,
            fixed,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for ApplySelectionFixSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run(ec)));
        Ok(())
    }
}

#[cfg(windows)]
impl ApplySelectionFixSession {
    fn run(&self, _ec: u32) -> bool {
        unsafe {
            // Precondition: selection vẫn còn và text vẫn là câu đã check
            // (người dùng chưa gõ xen/đổi vùng chọn giữa lúc popup hiện và
            // lúc bấm nút — fail-safe không ghi đè gì khác).
            let Ok(empty) = self.range.IsEmpty(_ec) else {
                return false;
            };
            if empty.as_bool() {
                return false;
            }
            // Guard cắt cụt: vùng dài hơn MAX ⇒ đã đổi so với câu đã check.
            let cap = MAX_SELECTION_CHARS + 2;
            let mut buf = vec![0u16; cap];
            let mut fetched = 0u32;
            if self.range.GetText(_ec, 0, &mut buf, &mut fetched).is_err() {
                return false;
            }
            let n = (fetched as usize).min(buf.len());
            if n > MAX_SELECTION_CHARS {
                return false;
            }
            let cur = String::from_utf16(&buf[..n]).unwrap_or_default();
            if cur != self.expect {
                return false;
            }
            // SetText thay vùng chọn — undo thuộc app (Notepad/Word Ctrl+Z).
            let wide: Vec<u16> = self.fixed.encode_utf16().collect();
            let ok = self.range.SetText(_ec, 0, &wide).is_ok();
            if ok {
                // F3 (audit v30): document đã đổi — buffer câu engine stale
                // nếu vùng chọn đè lên phần engine tự gõ; reset như A2/A3.
                self.shared.reset_engine();
                self.shared.clear_assist_undo();
                let auto_cap = self
                    .shared
                    .thread
                    .try_borrow()
                    .map(|t| t.auto_capitalize())
                    .unwrap_or(true);
                // Sau fix re-check câu đã sửa: còn lỗi thì cập nhật lại popup
                // ngay trên vùng chọn để người dùng bấm sửa tiếp được; sạch thì
                // thu gọn selection về cuối câu.
                if let Some(new_check) = crate::assist::check_text_fragment(&self.fixed, auto_cap) {
                    let anchor = if let Ok(view) = self.ctx.GetActiveView() {
                        let mut rc = RECT::default();
                        let mut clipped = windows::core::BOOL::default();
                        if view
                            .GetTextExt(_ec, &self.range, &mut rc, &mut clipped)
                            .is_ok()
                            && rc.right > rc.left
                            && rc.bottom > rc.top
                        {
                            Some(Anchor::new(rc.left, rc.top, rc.right, rc.bottom))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    crate::edit_session::show_assist_overlay_for_selection(
                        &self.shared,
                        &self.ctx,
                        _ec,
                        &new_check,
                        self.range.clone(),
                        anchor,
                    );
                } else {
                    crate::edit_session::set_caret_after(&self.ctx, _ec, &self.range);
                }
            }
            ok
        }
    }
}

/// Yêu cầu áp sửa vùng chọn (gọi từ FixAll khi popup đến từ selection-check).
#[cfg(windows)]
pub fn request_apply_selection_fix(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    range: &ITfRange,
    expect: &str,
    fixed: &str,
) -> bool {
    let session: ITfEditSession = ApplySelectionFixSession::new(
        shared.clone(),
        ctx.clone(),
        range.clone(),
        expect.to_string(),
        fixed.to_string(),
    )
    .into();
    // SAFETY: ctx hợp lệ trên thread UI.
    let hr = unsafe {
        ctx.RequestEditSession(shared.tid, &session, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE)
    };
    matches!(hr, Ok(code) if code.is_ok())
}

/// Trần ký tự cho bộ chuyển mã vùng chọn (OP-007): rộng hơn hạn mức 256 của
/// trợ lý ngữ pháp vì chuyển mã là thao tác văn bản thuần (một đoạn/bảng dài
/// vẫn hợp lệ). 64 000 code unit ≈ 64 KB bộ nhớ tạm — tuyến tính, không lag.
#[cfg(windows)]
pub(crate) const MAX_TRANSCODE_CHARS: usize = 64_000;

/// Session chuyển mã vùng bôi đen sang Unicode dựng sẵn (OP-007 · P5 §7.4).
///
/// Đọc vùng chọn **trong edit lock**, tự nhận diện bảng mã (`core::detect` —
/// fail-safe: mơ hồ ⇒ `UnicodePrecomposed` ⇒ không ghi gì), chuyển mã, rồi
/// ghi lại bằng `SetText` trên đúng range. Undo thuộc app (Ctrl+Z).
///
/// Precondition: đọc-lại vùng chọn phải bằng nội dung đã đọc lúc bắt đầu —
/// người dùng đổi vùng chọn giữa chừng thì **không ghi** (cùng nguyên tắc
/// với `ApplySelectionFixSession`).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct TranscodeSession {
    _guard: crate::class::ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
}

#[cfg(windows)]
impl TranscodeSession {
    pub fn new(shared: Rc<TsfShared>, ctx: ITfContext) -> Self {
        Self {
            _guard: crate::class::ObjGuard::new(),
            shared,
            ctx,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for TranscodeSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run(ec)));
        Ok(())
    }
}

#[cfg(windows)]
impl TranscodeSession {
    fn run(&self, ec: u32) -> bool {
        unsafe {
            let Some(sel) = selection_range(&self.ctx, ec) else {
                crate::trace::event(
                    self.shared.tid,
                    format_args!("transcode skip: no selection"),
                );
                return false;
            };
            match sel.IsEmpty(ec) {
                Ok(empty) if !empty.as_bool() => {}
                _ => {
                    crate::trace::event(
                        self.shared.tid,
                        format_args!("transcode skip: selection empty"),
                    );
                    return false;
                }
            }
            // S3 + read-only (review 1, parity SelectionCheckSession): tuyệt
            // đối không đọc/sửa ô mật khẩu/PIN — selection trong ô password
            // chính là mật khẩu.
            let signals = crate::edit_session::read_field_signals(&self.ctx, ec, &sel);
            if signals.read_only
                || crate::classify_tsf_field(&signals).security
                    != textvn_field_detect::SecurityState::NonSecure
            {
                return false;
            }
            // Đọc vùng chọn: chừa 2 slot để phát hiện vùng DÀI HƠN trần.
            let cap = MAX_TRANSCODE_CHARS + 2;
            let mut buf = vec![0u16; cap];
            let mut fetched = 0u32;
            if sel.GetText(ec, 0, &mut buf, &mut fetched).is_err() || fetched == 0 {
                return false;
            }
            let n = (fetched as usize).min(buf.len());
            if n > MAX_TRANSCODE_CHARS {
                crate::trace::event(
                    self.shared.tid,
                    format_args!("transcode skip: selection > {MAX_TRANSCODE_CHARS}"),
                );
                return false;
            }
            let Some(text) = String::from_utf16(&buf[..n]).ok() else {
                return false;
            };
            // R2-139: chuyển đổi THÔNG MINH theo kiểu gõ + bảng mã đang dùng —
            // chữ lạ do đọc nhầm UTF-8, bảng mã cũ → bảng mã đang dùng, từ ASCII gõ
            // nhầm chế độ E → gõ lại (logic duy nhất trong core, dùng chung 3 OS).
            let converted = self
                .shared
                .thread
                .try_borrow()
                .ok()
                .and_then(|thread| thread.engine.convert_selection(&text));
            let Some(converted) = converted.filter(|c| *c != text) else {
                return false; // không đổi gì ⇒ không ghi document
            };
            // Đọc-lại NGAY TRONG lock trước khi ghi — chống TOCTOU.
            let mut recheck = vec![0u16; cap];
            let mut re_fetched = 0u32;
            if sel.GetText(ec, 0, &mut recheck, &mut re_fetched).is_err()
                || re_fetched as usize != n
                || String::from_utf16(&recheck[..(re_fetched as usize).min(cap)])
                    .map(|s| s != text)
                    .unwrap_or(true)
            {
                return false;
            }
            let wide: Vec<u16> = converted.encode_utf16().collect();
            if sel.SetText(ec, 0, &wide).is_err() {
                return false;
            }
            // Document đã đổi: buffer câu engine stale + undo A3 không còn đúng.
            self.shared.reset_engine();
            self.shared.clear_assist_undo();
            crate::trace::event(
                self.shared.tid,
                format_args!("transcode applied: {n} units"),
            );
            true
        }
    }
}

/// OP-007 — yêu cầu chuyển mã vùng bôi đen (gọi từ chord Ctrl+Shift+F9).
/// `true` = đã xin được session (phím nên bị nuốt).
#[cfg(windows)]
pub fn request_transcode(shared: &Rc<TsfShared>, ctx: &ITfContext) -> bool {
    let session: ITfEditSession = TranscodeSession::new(shared.clone(), ctx.clone()).into();
    let hr = unsafe {
        ctx.RequestEditSession(shared.tid, &session, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE)
    };
    matches!(hr, Ok(code) if code.is_ok())
}

#[cfg(all(test, windows))]
mod tests {
    use super::is_selection_key_up;

    /// Vòng 30 — phân biệt bôi-đen-để-sửa vs đọc/copy: chỉ key-up
    /// selection KHÔNG Ctrl/Win (Ctrl+A/C/X = copy ops → bỏ).
    #[test]
    fn chi_selection_key_khong_ctrl() {
        const SHIFT: u32 = 0x1;
        const CTRL: u32 = 0x2;
        const WIN: u32 = 0x8;
        // Shift + mũi tên/Home/End → là.
        assert!(is_selection_key_up(0x25, SHIFT)); // ←
        assert!(is_selection_key_up(0x27, SHIFT)); // →
        assert!(is_selection_key_up(0x23, SHIFT)); // End
        assert!(is_selection_key_up(0x24, SHIFT)); // Home
        assert!(is_selection_key_up(0x10, 0)); // Shift tự nhả
                                               // Ctrl giữ (Ctrl+C/Ctrl+X/Ctrl+A) → KHÔNG (tránh nhầm copy).
        assert!(!is_selection_key_up(0x43, CTRL)); // C
        assert!(!is_selection_key_up(0x58, CTRL)); // X
        assert!(!is_selection_key_up(0x25, CTRL)); // Ctrl+← (word-nav)
                                                   // Win giữ → không.
        assert!(!is_selection_key_up(0x25, WIN));
        // Phím thường không Shift (gõ chữ) → không kích hoạt.
        assert!(!is_selection_key_up(0x41, 0)); // 'a'
                                                // F-key bị lọc (audit v30 F5: bôi đen bằng chuột rồi bấm F5
                                                // không được coi là "đang soạn thảo").
        assert!(!is_selection_key_up(0x70, 0));
        // Alt+← (Back browser) → KHÔNG.
        assert!(!is_selection_key_up(0x25, 0x4));
    }
}
