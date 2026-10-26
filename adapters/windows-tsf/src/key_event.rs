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
use crate::assist::{overlay_action_for, OverlayAction, PopupKind};
#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::compose::{is_modifier_vk, vk as vkc, KeyKind};
#[cfg(windows)]
use crate::edit_session::{
    apply_assist_suggestion, hide_assist_overlay, request_sync_or_async, select_assist_issue,
    selected_assist_issue, AssistRecallSession, EndCompositionSession, KeyEditSession, TsfShared,
};
#[cfg(windows)]
use crate::trace;
#[cfg(windows)]
use textvn_ffi::{ACTION_PASS, MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_SHIFT, MOD_SUPER};

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
        if self.shared.pending_eaten_vk.get() == Some(vk)
            || self
                .shared
                .deferred_replay
                .get()
                .is_some_and(|event| event.matches_vk(vk))
        {
            return Ok(true.into());
        }
        self.shared.pending_eaten_vk.set(None);
        self.shared.deferred_replay.set(None);
        if let Some(event) =
            guarded_option(|| defer_to_key_down(&self.shared, pic.ok().ok(), vk, lparam))
        {
            self.shared.deferred_replay.set(Some(event));
            trace_key(&self.shared, "test", vk, "deferred");
            return Ok(true.into());
        }
        let eaten = guarded(|| handle_key(&self.shared, pic.ok().ok(), vk, lparam));
        if eaten {
            self.shared.pending_eaten_vk.set(Some(vk));
        }
        trace_key(&self.shared, "test", vk, eaten_label(eaten));
        Ok(eaten.into())
    }

    fn OnTestKeyUp(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        if self
            .shared
            .deferred_replay
            .get()
            .is_some_and(|event| event.matches_vk(wparam.0 as u32))
        {
            self.shared.deferred_replay.set(None);
        }
        trace_key(&self.shared, "test-up", wparam.0 as u32, "");
        guarded(|| handle_key_up(&self.shared, pic.ok().ok(), wparam.0 as u32));
        // Vòng 30: bôi đen bằng bàn phím → debounce check ngữ pháp vùng chọn.
        let mods = active_modifiers();
        guarded(|| {
            maybe_schedule_selection_check(&self.shared, pic.ok().ok(), wparam.0 as u32, mods)
        });
        Ok(false.into())
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        if self.shared.pending_eaten_vk.take() == Some(vk) {
            trace_key(&self.shared, "down", vk, "confirm");
            return Ok(true.into());
        }
        let deferred = self
            .shared
            .deferred_replay
            .take()
            .filter(|event| event.matches_vk(vk));
        // Giữ reference trước khi `handle_key` nhận ownership Ref của callback;
        // nhánh deferred cần nó để commit dự phòng nếu edit session không chạy.
        let fallback_ctx = pic.ok().ok();
        let eaten = guarded(|| handle_key(&self.shared, fallback_ctx, vk, lparam));
        // CUAS đã đổi phím thành VK_PROCESSKEY vì pha test nhận nó: từ đã commit, giờ trả
        // phím gốc về cho app, xếp SAU kết quả composition (`replay.rs`).
        if let Some(event) = deferred {
            if eaten {
                // R2-123: TIP đã ĂN phím (Tab hoàn tất/dự đoán, gõ tắt, Ctrl+Space
                // mở trợ lý, ↑/↓ của overlay) — không có "phím gốc" nào phải trả
                // cho app; replay ở đây chèn thêm Tab/Ctrl+Space thừa vào app CUAS.
                trace_key(&self.shared, "down", vk, "consumed");
                return Ok(true.into());
            }
            // Nếu lock/session đã fail sau test TRUE, vẫn đóng composition
            // trước khi replay; CUAS không còn đường nào tự trả phím gốc.
            end_composition(&self.shared, fallback_ctx);
            if guarded(|| crate::replay::schedule(event)) {
                trace_key(&self.shared, "down", vk, "replayed");
            } else {
                trace_key(&self.shared, "down", vk, "target-closed");
            }
            return Ok(true.into());
        }
        trace_key(&self.shared, "down", vk, eaten_label(eaten));
        Ok(eaten.into())
    }

    fn OnKeyUp(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, _l: LPARAM) -> Result<BOOL> {
        if self
            .shared
            .deferred_replay
            .get()
            .is_some_and(|event| event.matches_vk(wparam.0 as u32))
        {
            self.shared.deferred_replay.set(None);
        }
        // Pha nào tới trước xử lý; `ModifierToggle` chỉ trả true một lần.
        trace_key(&self.shared, "up", wparam.0 as u32, "");
        guarded(|| handle_key_up(&self.shared, pic.ok().ok(), wparam.0 as u32));
        // F2 (audit v30): Notepad/Chrome/VS Code bỏ pha TEST ⇒ OnTestKeyUp
        // không fire — hook selection-check ở CẢ hai pha up (PENDING ghi đè
        // + SetTimer re-arm idempotent).
        let mods = active_modifiers();
        guarded(|| {
            maybe_schedule_selection_check(&self.shared, pic.ok().ok(), wparam.0 as u32, mods)
        });
        Ok(false.into())
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn OnPreservedKey(&self, pic: Ref<'_, ITfContext>, rguid: *const GUID) -> Result<BOOL> {
        if rguid.is_null() {
            return Ok(false.into());
        }
        // SAFETY: rguid do TSF cấp, đã kiểm non-null.
        let guid = unsafe { *rguid };
        // R2-133: phím Alt của trợ lý — chỉ ăn khi có việc, không thì phím tới
        // app nguyên vẹn (Alt+Space mở menu hệ thống, Alt+1 của app…).
        let suggestion = match guid {
            g if g == crate::guids::GUID_PRESERVED_ALT1 => Some(0),
            g if g == crate::guids::GUID_PRESERVED_ALT2 => Some(1),
            g if g == crate::guids::GUID_PRESERVED_ALT3 => Some(2),
            _ => None,
        };
        let alt_vk = match suggestion {
            Some(index) => Some(0x31 + index as u32),
            None if guid == crate::guids::GUID_PRESERVED_ASSIST => Some(vkc::SPACE),
            None => None,
        };
        // Phím chính TIP vừa trả lại cho app quay về đây ⇒ để yên (chống vòng lặp).
        if let Some(vk) = alt_vk {
            if take_redelivered(vk) {
                trace::event(self.shared.tid, format_args!("preserved redelivered-again"));
                return Ok(false.into());
            }
        }
        if let Some(index) = suggestion {
            let eaten = guarded(|| {
                if !self.shared.overlay.is_visible() {
                    return false;
                }
                let ctx = pic.ok().ok();
                apply_assist_suggestion(&self.shared, ctx, index);
                true
            });
            if !eaten {
                redeliver_alt_key(0x31 + index as u32);
            }
            trace::event(
                self.shared.tid,
                format_args!("preserved alt+{} {}", index + 1, eaten_label(eaten)),
            );
            return Ok(eaten.into());
        }
        if guid == crate::guids::GUID_PRESERVED_ASSIST {
            let eaten = guarded(|| run_assist_hotkey(&self.shared, pic.ok().ok()));
            if !eaten {
                redeliver_alt_key(vkc::SPACE);
            }
            trace::event(
                self.shared.tid,
                format_args!("preserved alt+space {}", eaten_label(eaten)),
            );
            return Ok(eaten.into());
        }
        if guid != crate::guids::GUID_PRESERVED_TOGGLE {
            return Ok(false.into());
        }
        // WIN-015: chỉ xử lý ở đây (không xử lý lại trong OnKeyDown → không toggle 2 lần).
        // Space của Ctrl+Shift+Space không tới key sink → hủy Ctrl+Shift đang chờ,
        // nếu không nhả Ctrl/Shift sẽ toggle lần nữa.
        let mut pending = self.shared.modifier_toggle.get();
        pending.reset();
        self.shared.modifier_toggle.set(pending);
        trace::event(self.shared.tid, format_args!("preserved toggle"));
        guarded(|| {
            toggle_vietnamese(&self.shared, pic.ok().ok());
            true
        });
        Ok(true.into())
    }
}

#[cfg(windows)]
thread_local! {
    /// R2-133: phím Alt TIP vừa trả lại cho app (vk + lúc trả).
    static REDELIVERED: Cell<Option<(u32, std::time::Instant)>> = const { Cell::new(None) };
}

/// Phím `vk` có phải chính phím TIP vừa trả lại (≤ 500 ms) không — có thì xoá dấu.
#[cfg(windows)]
fn take_redelivered(vk: u32) -> bool {
    REDELIVERED.with(|cell| match cell.get() {
        Some((v, at)) if v == vk && at.elapsed().as_millis() <= 500 => {
            cell.set(None);
            true
        }
        _ => false,
    })
}

/// R2-133: TSF luôn nuốt preserved key — đo thật: `OnPreservedKey` trả FALSE mà
/// Alt+Space vẫn không mở menu hệ thống. Phím Alt TextVN không dùng tới phải tự
/// trả lại:
/// - Alt+Space, cửa sổ gốc có `WS_SYSMENU`: `WM_SYSCOMMAND(SC_KEYMENU, ' ')` —
///   đúng việc `DefWindowProc` làm với Alt+Space (Edge bỏ qua `WM_SYSKEYDOWN` post
///   lại — đo thật); cửa sổ khung tự vẽ KHÔNG có `WS_SYSMENU` (VS Code/Electron
///   bỏ qua cả `SC_KEYMENU` lẫn phím post lại — đo thật) ⇒ tự mở menu hệ thống
///   như Chromium làm (`GetSystemMenu` + `TrackPopupMenu`), ngoài callback TSF
///   (`replay::schedule_system_menu`);
/// - Alt+1..3: post `WM_SYSKEYDOWN`/`WM_SYSKEYUP` (bit 29 = Alt) vào cửa sổ đang
///   focus — app xử lý như phím thật (mnemonic, accelerator Alt+1 của app).
#[cfg(windows)]
fn redeliver_alt_key(vk: u32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetAncestor, PostMessageW, GA_ROOT, SC_KEYMENU, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    // SAFETY: gọi trên thread UI của app (callback preserved key).
    let target = unsafe { GetFocus() };
    if target.is_invalid() {
        return;
    }
    if vk == vkc::SPACE {
        // SAFETY: HWND của thread hiện tại; chỉ đọc style và post tới cửa sổ gốc
        // cùng process.
        let sent = unsafe {
            let root = GetAncestor(target, GA_ROOT);
            let root = if root.is_invalid() { target } else { root };
            let style = windows::Win32::UI::WindowsAndMessaging::GetWindowLongW(
                root,
                windows::Win32::UI::WindowsAndMessaging::GWL_STYLE,
            ) as u32;
            style & windows::Win32::UI::WindowsAndMessaging::WS_SYSMENU.0 != 0
                && PostMessageW(
                    Some(root),
                    WM_SYSCOMMAND,
                    WPARAM(SC_KEYMENU as usize),
                    LPARAM(0x20),
                )
                .is_ok()
        };
        if sent || crate::replay::schedule_system_menu(target) {
            return;
        }
    }
    // SAFETY: MapVirtualKeyW chỉ tra bảng layout.
    let scan = unsafe { MapVirtualKeyW(vk, MAPVK_VK_TO_VSC) } as isize;
    let down = LPARAM(1 | (scan << 16) | (1 << 29));
    let up = LPARAM(1 | (scan << 16) | (1 << 29) | (0b11 << 30));
    REDELIVERED.with(|cell| cell.set(Some((vk, std::time::Instant::now()))));
    // SAFETY: post tới control của app trong cùng process; lỗi bỏ qua.
    unsafe {
        let _ = PostMessageW(Some(target), WM_SYSKEYDOWN, WPARAM(vk as usize), down);
        let _ = PostMessageW(Some(target), WM_SYSKEYUP, WPARAM(vk as usize), up);
    }
}

/// R2-133: Alt+Space = cùng luồng Ctrl+Space (sửa hết lỗi câu → dự đoán → trả
/// phím). Chạy đúng `KeyEditSession` của Ctrl+Space để engine thấy một phím
/// Ctrl+Space (dự đoán/duyệt vòng giống hệt), chỉ khác phím vật lý.
#[cfg(windows)]
fn run_assist_hotkey(shared: &Rc<TsfShared>, ctx: Option<&ITfContext>) -> bool {
    let Some(ctx) = ctx else {
        return false;
    };
    shared.ghost.hide();
    if shared.engine_mode().is_none() {
        return false; // TextVN tắt cho app này: phím thuộc app
    }
    if !shared.is_composing() {
        shared.sync_config();
    }
    let result = Rc::new(Cell::new(None));
    let session: ITfEditSession = KeyEditSession::new(
        shared.clone(),
        ctx.clone(),
        vkc::SPACE,
        MOD_CTRL,
        KeyKind::CtrlSpace,
        result.clone(),
    )
    .into();
    // SAFETY: ctx hợp lệ trong callback preserved key; session đồng bộ.
    let hr = unsafe { ctx.RequestEditSession(shared.tid, &session, TF_ES_SYNC | TF_ES_READWRITE) };
    matches!((hr, result.get()), (Ok(code), Some(true)) if code.is_ok())
}

/// Đảo bật/tắt tiếng Việt: commit từ đang gõ, đổi ngay trong process, báo tray.
#[cfg(windows)]
fn toggle_vietnamese(shared: &TsfShared, ctx: Option<&ITfContext>) {
    end_composition(shared, ctx);
    shared.reset_engine();
    // Bấm đôi trong 250ms = MỘT lần bấm (khớp cửa sổ tray) — lần sau giữ state.
    let on = if crate::ipc_client::try_claim_hotkey_toggle() {
        shared.ipc.toggle_global()
    } else {
        shared.ipc.is_enabled()
    };
    trace::event(
        shared.tid,
        format_args!("toggle -> {}", if on { "VI" } else { "EN" }),
    );
}

#[cfg(windows)]
fn trace_key(shared: &TsfShared, phase: &str, vk: u32, result: &str) {
    if !trace::enabled() {
        return;
    }
    trace::event(
        shared.tid,
        format_args!(
            "{phase} {} {result} composing={}",
            trace::vk_label(vk),
            shared.is_composing()
        ),
    );
}

#[cfg(windows)]
fn eaten_label(eaten: bool) -> &'static str {
    if eaten {
        "eaten"
    } else {
        "pass"
    }
}

/// App IMM32 chạy qua CUAS (Notepad cổ điển, WinForms, Delphi…): khi pha test đóng
/// composition rồi trả "không ăn", kết quả composition tới cửa sổ SAU phím gốc
/// (Enter/Tab ra trước chữ — bắt được bằng test gõ thật). Với phím sẽ đóng composition
/// và đi tới app (Enter, Tab, điều hướng, Ctrl+…), pha test nhận phím; `OnKeyDown` commit
/// từ (lúc CUAS đang dịch phím) rồi trả phím gốc qua `replay.rs`. Alt/Win+… là
/// `WM_SYSKEYDOWN`/phím hệ thống — không giữ lại. App TSF-aware không bị ảnh hưởng.
#[cfg(windows)]
fn defer_to_key_down(
    shared: &TsfShared,
    ctx: Option<&ITfContext>,
    vk: u32,
    lparam: LPARAM,
) -> Option<crate::replay::DeferredReplay> {
    if !shared.is_composing() || is_modifier_vk(vk) {
        return None;
    }
    let ctx = ctx?;
    // SAFETY: ctx hợp lệ trong callback của key sink.
    let transitory = unsafe { ctx.GetStatus() }
        .map(|st| st.dwStaticFlags & TF_SS_TRANSITORY != 0)
        .unwrap_or(false);
    if !transitory {
        return None;
    }
    let mods = active_modifiers();
    if mods & (MOD_ALT | MOD_SUPER) != 0 {
        return None;
    }
    let needs_replay = mods & MOD_CTRL != 0
        || matches!(
            KeyKind::classify(vk, translate_key(vk, lparam)),
            KeyKind::Enter | KeyKind::Tab | KeyKind::Other
        );
    needs_replay
        .then(|| crate::replay::prepare(vk, lparam))
        .flatten()
}

/// Nhả phím: chỉ dùng cho phím chuyển Ctrl+Shift; không bao giờ ăn phím.
#[cfg(windows)]
fn handle_key_up(shared: &TsfShared, ctx: Option<&ITfContext>, vk: u32) -> bool {
    let mut toggle = shared.modifier_toggle.get();
    let fire = toggle.on_key_up(vk);
    shared.modifier_toggle.set(toggle);
    if fire {
        toggle_vietnamese(shared, ctx);
    }
    false
}

/// Vòng 30: key-up của phím selection (Shift/mũi tên/Home/End, không Ctrl)
/// lên lịch kiểm tra ngữ pháp vùng bôi đen — cần `Rc<TsfShared>` (PENDING giữ
/// Weak) nên gọi từ `OnTestKeyUp` có `shared: Rc`.
#[cfg(windows)]
pub(crate) fn maybe_schedule_selection_check(
    shared: &std::rc::Rc<TsfShared>,
    ctx: Option<&ITfContext>,
    vk: u32,
    mods: u32,
) -> bool {
    if let Some(ctx) = ctx {
        crate::selection_check::on_key_up(shared, Some(ctx), vk, mods);
    }
    false
}

/// Phím của overlay đang hiện (P4-0 §4.2): `Some(eaten)` = đã quyết định xong
/// (trả ngay cho TSF); `None` = phím đi tiếp luồng gõ bình thường.
#[cfg(windows)]
fn handle_overlay_key(
    shared: &Rc<TsfShared>,
    ctx: Option<&ITfContext>,
    vk: u32,
    mods: u32,
) -> Option<bool> {
    if is_modifier_vk(vk) {
        return None; // Ctrl/Shift của Ctrl+Shift+G vẫn đi qua đây
    }
    let visible = shared.overlay.is_visible();
    if !visible {
        // Panel có thể đã tự tắt (8 s / click chuột / app đổi foreground) mà
        // watchdog không đụng được state của TIP: dọn phần còn lại để không giữ
        // câu người dùng trong RAM và không dùng caret range cũ (S2/D12).
        let stale = shared
            .assist
            .try_borrow()
            .map(|slot| slot.is_some())
            .unwrap_or(false);
        if stale {
            hide_assist_overlay(shared);
        }
    }
    // Vòng 31/32: popup đến từ BÔI ĐEN (selection) — áp đơn-từ (A2 cần
    // selection rỗng) vô nghĩa trên popup này; Enter/Alt+1..3 đều phải đi
    // Sửa-tất-cả vùng chọn (cùng đường nút chuột), không nuốt phím oan.
    let (selection_popup, on_demand) = shared
        .assist
        .try_borrow()
        .ok()
        .and_then(|slot| slot.as_ref().map(|a| (a.selection.is_some(), a.on_demand)))
        .unwrap_or((false, false));
    let kind = if on_demand {
        PopupKind::OnDemand
    } else if selection_popup {
        PopupKind::Selection
    } else {
        PopupKind::Live
    };
    match overlay_action_for(vk, mods, kind) {
        // Ctrl+Shift+G: kiểm tra lại câu + hiện overlay (không phụ thuộc overlay).
        OverlayAction::Recall => {
            let ctx = ctx?; // không có ctx = không xin được session → để phím đi
            let session: ITfEditSession =
                AssistRecallSession::new(shared.clone(), ctx.clone()).into();
            request_sync_or_async(ctx, shared.tid, &session);
            Some(true)
        }
        _ if !visible => None,
        // ↑/↓: đổi lỗi đang chọn. Ăn phím để mũi tên không dời caret của app
        // trong lúc panel còn neo ở vị trí cũ.
        OverlayAction::Previous => {
            select_assist_issue(shared, -1);
            Some(true)
        }
        OverlayAction::Next => {
            select_assist_issue(shared, 1);
            Some(true)
        }
        // Enter: áp gợi ý của lỗi đang chọn (A2 — Backspace+Type, D22).
        OverlayAction::Apply => {
            if selection_popup {
                super::edit_session::selection_fix_all_from_keyboard(shared);
            } else {
                apply_assist_suggestion(shared, ctx, selected_assist_issue(shared));
            }
            Some(true)
        }
        // Alt+1..3: áp gợi ý của lỗi thứ n (hỗ trợ cả popup thường lẫn popup selection).
        OverlayAction::Suggestion(index) => {
            apply_assist_suggestion(shared, ctx, index);
            Some(true)
        }
        // Esc: đóng, không sửa (không để app nhận Esc).
        OverlayAction::Close => {
            hide_assist_overlay(shared);
            Some(true)
        }
        // Gõ tiếp / chord khác: đóng panel rồi để phím đi bình thường.
        OverlayAction::Dismiss => {
            hide_assist_overlay(shared);
            None
        }
        // R2-129: popup sống — phím gõ đi thẳng, popup giữ nguyên (lần kiểm
        // tra kế tiếp ở ranh giới từ/Backspace cập nhật hoặc ẩn theo câu).
        OverlayAction::Keep => None,
    }
}

/// Nhấn phím: cập nhật máy trạng thái Ctrl+Shift (gọi lại cùng phím không đổi kết quả).
#[cfg(windows)]
fn observe_key_down(shared: &TsfShared, vk: u32, mods: u32) {
    let mut toggle = shared.modifier_toggle.get();
    toggle.on_key_down(
        vk,
        mods & MOD_CTRL != 0,
        mods & MOD_SHIFT != 0,
        mods & (MOD_ALT | MOD_SUPER) != 0,
    );
    shared.modifier_toggle.set(toggle);
}

/// Thấy MỌI phím trước khi TSF xử lý. Ở app TSF-aware (WordPad, Word…) phím chuyển bố cục
/// Ctrl+Shift của Windows nuốt Shift trước key sink (thấy bằng `TEXTVN_TSF_TRACE` trên
/// Windows thật) nên chỉ nơi này nhận ra lần bấm Ctrl+Shift. Chỉ quan sát, không ăn phím;
/// key sink vẫn cập nhật cùng máy trạng thái — ai thấy lần nhả trước thì chuyển, đúng một lần.
#[cfg(windows)]
#[implement(ITfKeyTraceEventSink)]
pub struct KeyTraceSink {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
}

#[cfg(windows)]
impl KeyTraceSink {
    pub fn new(shared: Rc<TsfShared>) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
        }
    }
}

#[cfg(windows)]
impl ITfKeyTraceEventSink_Impl for KeyTraceSink_Impl {
    fn OnKeyTraceDown(&self, wparam: WPARAM, _lparam: LPARAM) -> Result<()> {
        let vk = wparam.0 as u32;
        trace_key(&self.shared, "ktrace", vk, "");
        guarded(|| {
            observe_key_down(&self.shared, vk, active_modifiers());
            false
        });
        Ok(())
    }

    fn OnKeyTraceUp(&self, wparam: WPARAM, _lparam: LPARAM) -> Result<()> {
        let vk = wparam.0 as u32;
        trace_key(&self.shared, "ktrace-up", vk, "");
        guarded(|| handle_key_up(&self.shared, None, vk));
        Ok(())
    }
}

/// Mọi đường phím: panic không được thoát qua ranh giới COM (abort app).
#[cfg(windows)]
fn guarded(f: impl FnOnce() -> bool) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(false)
}

#[cfg(windows)]
fn guarded_option<T>(f: impl FnOnce() -> Option<T>) -> Option<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .ok()
        .flatten()
}

/// R-3 + Q-3: phím này có **huỷ** state hoàn tác tự sửa cả câu không?
///
/// - `Ctrl+Z` (chord hoàn tác) ⇒ **không** huỷ (chính là phím áp hoàn tác).
/// - Phím **modifier** (Ctrl/Shift/Alt/Win/Caps) ⇒ **không** huỷ: modifier
///   **không sinh ký tự**; nếu huỷ ở đây thì lần nhấn Ctrl đã xoá state ⇒
///   `Ctrl+Z` ngay sau đó **không bao giờ** hoàn tác được (Q-3 — lệch parity
///   Linux/macOS vốn `return` **trước** khi clear).
/// - Phím **sinh ký tự** ⇒ **huỷ** (gõ tiếp ⇒ không xoá nhầm chữ vừa gõ).
#[cfg(windows)]
fn clears_assist_undo(vk: u32, mods: u32) -> bool {
    let is_assist_undo_chord = mods == MOD_CTRL && vk == 0x5A;
    !is_assist_undo_chord && !is_modifier_vk(vk)
}

/// Cập nhật state hoàn tác khi 1 phím tới (R-3 · Q-3) — tách riêng để **test
/// được** mà không cần dựng TSF.
#[cfg(windows)]
fn update_assist_undo_state(shared: &TsfShared, vk: u32, mods: u32) {
    if clears_assist_undo(vk, mods) {
        shared.clear_assist_undo();
    }
}

/// Xử lý 1 phím, trả `eaten`. Mọi nhánh lỗi → `false` (phím tới app nguyên vẹn).
#[cfg(windows)]
fn handle_key(shared: &Rc<TsfShared>, ctx: Option<&ITfContext>, vk: u32, lparam: LPARAM) -> bool {
    let mods = active_modifiers();
    observe_key_down(shared, vk, mods);

    // R-3 (undo destructive): trạng thái hoàn tác tự sửa cả câu chỉ sống cho
    // **phím kế tiếp**. Gõ tiếp (bất kỳ phím nào khác Ctrl+Z) ⇒ huỷ state, để
    // Ctrl+Z sau đó KHÔNG xoá nhầm cả chữ vừa gõ và không ăn Ctrl+Z của app.
    // (macOS đã làm đúng như vậy — giữ hành vi giống nhau cho 3 OS.)
    //
    // Q-3: chỉ phím **sinh ký tự** mới huỷ state; phím modifier (Ctrl/Shift/…)
    // bị bỏ qua, nếu không thì nhấn Ctrl đã xoá state ⇒ undo A3 chết.
    let is_assist_undo_chord = mods == MOD_CTRL && vk == 0x5A;
    update_assist_undo_state(shared, vk, mods);
    if !is_modifier_vk(vk) {
        crate::selection_check::cancel_pending();
        // R2-134: chữ mờ chỉ sống tới phím kế tiếp (phím gõ vẽ lại nếu còn gợi ý).
        shared.ghost.hide();
    }

    // AS-030/AS-031 (Mode 1 + A2): phím của overlay (↑/↓, Enter, Alt+1..3, Esc,
    // Ctrl+Shift+G). Bản đồ phím là hàm THUẦN `assist::overlay_action`; mọi lời
    // gọi TSF nằm trong edit session, không trong window proc của panel.
    if let Some(decided) = handle_overlay_key(shared, ctx, vk, mods) {
        return decided;
    }

    if is_modifier_vk(vk) {
        return false;
    }

    // AS-050: Hoàn tác tự sửa cả câu nếu người dùng bấm Ctrl+Z trong vòng 5 giây.
    // R-3: chỉ nuốt phím khi hoàn tác **thực sự áp dụng** được; nếu không (hết
    // hạn / không xin được edit session / vùng cần xoá không còn là của ta) ⇒
    // phím tới app ⇒ Ctrl+Z của app vẫn hoạt động bình thường.
    if is_assist_undo_chord {
        if let Some(ctx) = ctx {
            if crate::edit_session::apply_assist_undo(shared, ctx) {
                return true;
            }
        }
    }

    // OP-007 (P5): Ctrl+Shift+F9 — chuyển mã vùng bôi đen về Unicode dựng sẵn
    // (font cũ .VnTime / VNI-Times / Unicode tổ hợp). Chạy TRƯỚC nhánh chord
    // hệ thống (nhánh đó chỉ kết thúc composition rồi để app nhận phím).
    // Chỉ nuốt phím khi xin được edit session — không thì phím tới app như cũ.
    if crate::assist::is_transcode_chord(vk, mods) {
        if let Some(ctx) = ctx {
            end_composition(shared, Some(ctx));
            if crate::selection_check::request_transcode(shared, ctx) {
                return true;
            }
        }
    }

    // Chord hệ thống (B6), phím do phần mềm bơm vào (VK_PACKET) hoặc IME khác đã
    // xử lý: commit từ đang gõ rồi để app nhận phím.
    // Ngoại lệ: Ctrl+Space được giữ lại cho Text Prediction & Text Correct.
    let is_ctrl_space = vk == 0x20 && (mods & !MOD_CAPS) == MOD_CTRL;
    if (mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0 && !is_ctrl_space)
        || vk == vkc::PACKET
        || vk == vkc::PROCESSKEY
    {
        end_composition(shared, ctx);
        shared.reset_engine();
        return false;
    }

    // Reload config xóa từ trong engine → chỉ làm ở ranh giới từ.
    if !shared.is_composing() {
        shared.sync_config();
    }
    let Some(vi_on) = shared.engine_mode() else {
        end_composition(shared, ctx);
        shared.reset_engine();
        return false;
    };

    let key = KeyKind::classify_with_mods(vk, translate_key(vk, lparam), mods);
    if !shared.is_composing() && !key.needs_session_when_idle() {
        // Backspace/Esc/điều hướng khi không composing: chỉ cho engine dọn trạng thái.
        if let Ok(mut thread) = shared.thread.try_borrow_mut() {
            thread.engine.set_enabled(vi_on);
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
    hide_assist_overlay(shared);
    shared.ghost.hide();
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
    // Caps Lock đang bật (bit toggle): engine coi phím dấu Telex viết hoa là phím dấu.
    // SAFETY: như trên.
    if (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1) != 0 {
        mods |= MOD_CAPS;
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// Q-3 (MAJOR): nhấn **Ctrl** (modifier) KHÔNG được huỷ state hoàn tác.
    ///
    /// Lỗi cũ: `handle_key` clear state **trước** khi kiểm `is_modifier_vk` ⇒
    /// nhấn Ctrl đã xoá state ⇒ `Ctrl+Z` ngay sau đó **không bao giờ** hoàn
    /// tác được (undo A3 chết). Linux/macOS `return` trước khi clear ⇒ lệch
    /// parity.
    #[test]
    fn q3_ctrl_roi_z_trong_5s_thi_undo_van_ap_dung() {
        let client = crate::ipc_client::IpcClient::global("test_assist_undo_chord.exe");
        let thread = crate::ThreadState::new("test_assist_undo_chord.exe", 0).unwrap();
        let shared = TsfShared::new(3, client, thread);
        *shared.assist_undo.try_borrow_mut().unwrap() = Some(crate::assist::AssistUndoState::new(
            "hôm nay tôi tôi đi học .".to_string(),
            "Hôm nay tôi đi học.".to_string(),
            19,
            2,
        ));

        // Nhấn Ctrl (modifier — không sinh ký tự) ⇒ state phải còn.
        update_assist_undo_state(&shared, vkc::CONTROL, MOD_CTRL);
        assert!(
            shared.assist_undo.try_borrow().unwrap().is_some(),
            "phím modifier KHÔNG được huỷ state hoàn tác (Q-3)"
        );

        // Nhấn Z kèm Ctrl (chord hoàn tác) ⇒ state phải còn để undo áp dụng.
        update_assist_undo_state(&shared, 0x5A, MOD_CTRL);
        assert!(
            shared.assist_undo.try_borrow().unwrap().is_some(),
            "chord Ctrl+Z KHÔNG được huỷ state hoàn tác"
        );

        // Gõ tiếp (phím sinh ký tự) ⇒ state bị huỷ (không xoá nhầm chữ vừa gõ).
        update_assist_undo_state(&shared, 0x41, 0);
        assert!(
            shared.assist_undo.try_borrow().unwrap().is_none(),
            "phím sinh ký tự phải huỷ state hoàn tác (R-3)"
        );
    }

    /// Bảng quyết định: chỉ phím **sinh ký tự** mới huỷ state hoàn tác.
    #[test]
    fn q3_bang_quyet_dinh_huy_state_hoan_tac() {
        // Modifier (Ctrl/Shift/Alt/Win/Caps + các phím chức năng tương ứng).
        assert!(!clears_assist_undo(vkc::CONTROL, MOD_CTRL));
        assert!(!clears_assist_undo(vkc::SHIFT, MOD_SHIFT));
        assert!(!clears_assist_undo(vkc::MENU, MOD_ALT));
        assert!(!clears_assist_undo(vkc::LWIN, MOD_SUPER));
        assert!(!clears_assist_undo(vkc::CAPITAL, MOD_CAPS));
        // Chord hoàn tác.
        assert!(!clears_assist_undo(0x5A, MOD_CTRL));
        // Phím sinh ký tự ⇒ huỷ.
        assert!(clears_assist_undo(0x41, 0));
        assert!(clears_assist_undo(0x20, 0));
        assert!(clears_assist_undo(0x5A, 0));
    }
}
