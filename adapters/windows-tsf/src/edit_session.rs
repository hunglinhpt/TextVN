// SPDX-License-Identifier: GPL-3.0-or-later
//! Edit session của TextVN TSF (WIN-012/013/014/017/018/019).
//!
//! Mỗi phím cần sửa text chạy trọn trong MỘT edit session đồng bộ
//! ([`KeyEditSession`]): đọc selection → tự chữa lệch (self-heal) → security gate
//! từ InputScope/`ES_PASSWORD` → gọi engine → áp [`CompositionPlan`]. Engine chỉ
//! được gọi *bên trong* session: nếu TSF không cấp lock, engine không hề thấy phím
//! và phím đi thẳng tới app (fail-open, không lệch buffer).

#[cfg(windows)]
use std::cell::{Cell, RefCell};
#[cfg(windows)]
use std::rc::{Rc, Weak};

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::E_FAIL;
#[cfg(windows)]
use windows::Win32::Foundation::RECT;
use windows::Win32::System::Variant::{VariantClear, VT_UNKNOWN};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetGUIThreadInfo, GetWindowLongW, ES_PASSWORD, GUITHREADINFO, GWL_STYLE,
};

#[cfg(windows)]
use crate::assist::{
    can_apply_fix, plan_fix, plan_sentence_auto_fix, plan_sentence_user_fix,
    should_auto_fix_sentence, AssistCheck, AssistTrigger, AssistUndoState, FixPlan,
    FixPrecondition, OverlayView, SentenceFixPlan, SENTENCE_MAX_CHARS,
};
#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::compose::{plan_key, text_after, CompositionPlan, EngineStep, KeyKind, ModifierToggle};
#[cfg(windows)]
use crate::ipc_client::IpcClient;
#[cfg(windows)]
use crate::overlay::Anchor;
#[cfg(windows)]
use crate::{classify_tsf_field, ThreadState, TsfFieldSignals};
#[cfg(windows)]
use textvn_ffi::IME_FLAG_ERROR;
#[cfg(windows)]
use textvn_strategy::Strategy;

/// Trạng thái overlay đang hiện của thread + mọi thứ A2 cần để xác minh **vị trí**
/// trước khi sửa (D12): câu trong buffer core lúc hiển thị và caret lúc đó.
///
/// Không giữ nội dung document của app (D8): câu là buffer **của ta**
/// (`Engine::sentence_tail`) — thứ duy nhất ta biết chắc mình đã gõ.
#[cfg(windows)]
pub struct ActiveAssist {
    /// Model hiển thị (câu + lỗi + lỗi đang chọn). Mỗi lỗi mang **kind**
    /// FFI (`IME_ASSIST_KIND_*`) ⇒ biết lỗi nào engine sửa bằng `Fix::Delete`
    /// (xoá span) thay vì thay span — không đoán bằng text (Q-2).
    pub view: OverlayView,
    /// Câu lúc hiện overlay — precondition `buffer_is_ours`.
    pub sentence: String,
    /// Caret (range rỗng) lúc hiện overlay — A2 chỉ so **vị trí**
    /// (`CompareStart`/`CompareEnd`), không đọc chữ.
    pub caret: ITfRange,
    /// Context lúc hiện overlay — click chuột trên panel xin edit session
    /// riêng qua `request_sync_or_async` (vòng 27: panel bấm được).
    pub ctx: ITfContext,
    /// Kết quả FFI nguyên vẹn lúc show — FixAll từ chuột dùng lại
    /// (`ApplySentenceFixSession` chỉ đọc `check.sentence`).
    pub check: AssistCheck,
    /// Vòng 30: popup đến từ **selection-check** (bôi đen) — range chọn
    /// (`None` = popup thường theo luồng gõ). FixAll từ chuột rẽ nhánh
    /// SetText-vùng-chọn thay vì Backspace+Type.
    pub selection: Option<windows::Win32::UI::TextServices::ITfRange>,
    /// Popup được dựng từ `recheck_after_fix` (sau khi sửa lần 1, buffer engine
    /// đã `reset()` nhưng document tại caret vẫn khớp `sentence`).
    pub from_recheck: bool,
    /// Popup do người dùng **chủ động gọi** (Ctrl+Space). Chỉ popup này mới
    /// dùng Tab để chuyển lỗi; popup tự hiện sau dấu câu để Tab tới app
    /// (chuyển ô form/Excel, thụt lề) — R2-122.
    pub on_demand: bool,
}

/// Đánh dấu popup đang hiện là popup on-demand (Ctrl+Space) — R2-122.
#[cfg(windows)]
fn mark_assist_on_demand(shared: &TsfShared) {
    let view = match shared.assist.try_borrow_mut() {
        Ok(mut slot) => slot.as_mut().map(|active| {
            active.on_demand = true;
            // R2-129: chế độ chọn-sửa — vẽ lại dòng nhắc ↑/↓ · Enter sửa.
            active.view.interactive = true;
            active.view.clone()
        }),
        Err(_) => None,
    };
    if let Some(view) = view {
        shared.overlay.set_view(&view);
    }
}

/// Popup đang hiện có phải popup **sống** (R2-129: tự hiện theo câu, không
/// phải Ctrl+Space / bôi đen) không.
#[cfg(windows)]
fn live_popup_visible(shared: &TsfShared) -> bool {
    shared.overlay.is_visible()
        && shared
            .assist
            .try_borrow()
            .ok()
            .and_then(|slot| slot.as_ref().map(|a| !a.on_demand && a.selection.is_none()))
            .unwrap_or(false)
}

/// R2-129 (chủ repo 2026-10-26: "popup cần hiện nhanh nhất hoặc ẩn đi nhanh
/// nhất theo tình trạng câu đúng sai"): kiểm tra lại câu ở mỗi ranh giới từ
/// (và Backspace khi popup sống đang hiện) — câu còn lỗi → hiện/cập nhật popup
/// tại caret; câu đã đúng → ẩn ngay. Không đè popup Ctrl+Space / bôi đen.
#[cfg(windows)]
fn refresh_live_popup(shared: &Rc<TsfShared>, ctx: &ITfContext, ec: u32) {
    if !shared.is_assist_display_enabled() {
        return;
    }
    if shared.overlay.is_visible() && !live_popup_visible(shared) {
        return; // popup chọn-sửa / bôi đen của người dùng: giữ nguyên
    }
    match super::assist::check_sentence(shared) {
        Some(check) => {
            show_assist_overlay(shared, ctx, ec, &check);
        }
        None => {
            if live_popup_visible(shared) {
                hide_assist_overlay(shared);
            }
        }
    }
}

/// State của TIP trên MỘT thread UI. Chia sẻ (Rc) giữa Tip, KeySink, CompSink và
/// edit session; mọi RefCell chỉ mượn trong phạm vi ngắn, không giữ qua lời gọi
/// COM có thể re-entrant. Không `unwrap`/`borrow_mut` có thể panic trong callback.
#[cfg(windows)]
pub struct TsfShared {
    pub tid: u32,
    pub ipc: &'static IpcClient,
    pub thread: RefCell<ThreadState>,
    /// Composition TIP đang sở hữu (None = không composing).
    pub composition: RefCell<Option<ITfComposition>>,
    /// Model text của composition (nguồn sự thật cho `plan_key`).
    pub comp_text: RefCell<Vec<char>>,
    /// VK đã được ăn ở `OnTestKeyDown`, chờ `OnKeyDown` tương ứng.
    pub pending_eaten_vk: Cell<Option<u32>>,
    /// Phím/đích được giữ lại ở `OnTestKeyDown` để xử lý ở `OnKeyDown` (app
    /// IMM32/CUAS, xem `key_event::defer_to_key_down`).
    pub deferred_replay: Cell<Option<crate::replay::DeferredReplay>>,
    /// Phiên bản config đã nạp vào engine của thread này.
    pub config_seen: Cell<u64>,
    /// App/TSF kết thúc composition của ta (OnCompositionTerminated).
    pub terminated: Cell<bool>,
    /// Phím chuyển Ctrl+Shift kiểu UniKey.
    pub modifier_toggle: Cell<ModifierToggle>,
    /// Overlay hiển thị gợi ý trợ lý ngữ pháp (AS-030 · Mode 1).
    pub overlay: crate::overlay::CandidateOverlay,
    /// R2-134: chữ mờ gợi ý sẵn (ghost text) sau con trỏ.
    pub ghost: crate::ghost::GhostText,
    /// Lỗi/câu/caret của overlay đang hiện (AS-031 · A2). `None` = overlay tắt.
    pub assist: RefCell<Option<ActiveAssist>>,
    /// Trạng thái hoàn tác tự sửa cả câu (AS-050 · Mode 2 / A3). `None` = không có undo hoặc đã hết hạn.
    pub assist_undo: RefCell<Option<crate::assist::AssistUndoState>>,
}

#[cfg(windows)]
impl TsfShared {
    pub fn new(tid: u32, ipc: &'static IpcClient, thread: ThreadState) -> Self {
        Self {
            tid,
            ipc,
            thread: RefCell::new(thread),
            composition: RefCell::new(None),
            comp_text: RefCell::new(Vec::new()),
            pending_eaten_vk: Cell::new(None),
            deferred_replay: Cell::new(None),
            config_seen: Cell::new(0),
            terminated: Cell::new(false),
            modifier_toggle: Cell::new(ModifierToggle::default()),
            overlay: crate::overlay::CandidateOverlay::new(),
            ghost: crate::ghost::GhostText::new(),
            assist: RefCell::new(None),
            assist_undo: RefCell::new(None),
        }
    }

    /// Mode 2 (A3 Whole-Sentence Auto-Correction) có đang bật hay không.
    ///
    /// R-5: ngữ nghĩa **duy nhất cho 3 OS** — `assist.mode == "autocorrect"` **và**
    /// `assist.auto_fix == "safe"` (P4-0 §4.3). Chỉ cần `auto_fix` là lệch với
    /// macOS/Linux ⇒ 1 config, 3 hành vi khác nhau.
    pub fn is_assist_auto_fix_enabled(&self) -> bool {
        self.ipc.is_assist_enabled()
            && self
                .thread
                .try_borrow()
                .map(|t| {
                    crate::assist::mode2_auto_fix_enabled(t.assist_mode(), t.assist_auto_fix())
                })
                .unwrap_or(false)
    }

    /// Trợ lý ngữ pháp có đang **hoạt động** không: kill-switch D20 **và**
    /// `assist.mode != "off"` (P4-0 §7 — `mode: off` = tắt hoàn toàn).
    ///
    /// Cùng ngữ nghĩa với `lc_assist_is_active` (Linux) và `isAssistActive`
    /// (macOS) ⇒ 1 config, 3 hành vi giống nhau (R-5).
    pub fn is_assist_enabled(&self) -> bool {
        self.ipc.is_assist_enabled()
            && self
                .thread
                .try_borrow()
                .map(|t| crate::assist::mode1_display_enabled(t.assist_mode()))
                .unwrap_or(false)
    }

    /// Điểm **hiển thị** (Mode 1 — overlay) có bật không (R-6: giống macOS/Linux).
    pub fn is_assist_display_enabled(&self) -> bool {
        self.is_assist_enabled()
    }

    /// Xoá trạng thái hoàn tác tự sửa cả câu (R-3).
    ///
    /// Gọi khi người dùng gõ **phím khác** (không phải Ctrl+Z): giữ state quá
    /// lâu ⇒ Ctrl+Z sau đó xoá nhầm cả chữ vừa gõ và ăn luôn Ctrl+Z của app.
    pub fn clear_assist_undo(&self) {
        if let Ok(mut slot) = self.assist_undo.try_borrow_mut() {
            *slot = None;
        }
    }

    pub fn is_composing(&self) -> bool {
        self.comp_text
            .try_borrow()
            .map(|t| !t.is_empty())
            .unwrap_or(false)
            || self
                .composition
                .try_borrow()
                .map(|c| c.is_some())
                .unwrap_or(false)
    }

    /// Xóa từ đang gõ trong engine (không đụng text của app).
    /// `Some(vi_on)` = phím đi qua engine (VN bật, hoặc VN tắt nhưng còn gõ tắt —
    /// `allow_macro_when_vi_off`); `None` = phím đi thẳng cho app.
    pub fn engine_mode(&self) -> Option<bool> {
        let vi_on = self.ipc.is_enabled();
        let macro_only = !vi_on
            && self
                .thread
                .try_borrow()
                .map(|t| t.macro_when_off())
                .unwrap_or(false);
        (vi_on || macro_only).then_some(vi_on)
    }

    pub fn reset_engine(&self) {
        if let Ok(mut thread) = self.thread.try_borrow_mut() {
            thread.engine.reset();
        }
    }

    /// R2-131: sau khi trợ lý sửa câu ngay trong document — reset engine rồi nạp
    /// câu đã sửa làm ngữ cảnh (phím tắt bấm tiếp dự đoán được; câu cũ không còn).
    pub fn reset_engine_with_tail(&self, tail: &str) {
        if let Ok(mut thread) = self.thread.try_borrow_mut() {
            thread.engine.reset_with_tail(tail);
        }
    }

    /// Bỏ composition khỏi state; trả về object để caller kết thúc nó trong session.
    pub fn take_composition(&self) -> Option<ITfComposition> {
        if let Ok(mut text) = self.comp_text.try_borrow_mut() {
            text.clear();
        }
        self.composition
            .try_borrow_mut()
            .ok()
            .and_then(|mut c| c.take())
    }

    /// Nạp lại config khi tray báo phiên bản mới (WIN-016).
    pub fn sync_config(&self) {
        let latest = self.ipc.config_version();
        if latest != 0 && latest != self.config_seen.get() {
            if let Ok(mut thread) = self.thread.try_borrow_mut() {
                if thread.reload_config_from_file().is_ok() {
                    self.config_seen.set(latest);
                }
            }
        }
    }

    /// Đồng bộ state khi composition bị bên ngoài kết thúc.
    fn absorb_termination(&self, comp: &mut Option<ITfComposition>) {
        if self.terminated.take() {
            *comp = None;
            if let Ok(mut text) = self.comp_text.try_borrow_mut() {
                text.clear();
            }
            self.reset_engine();
        }
    }
}

/// Sink nhận thông báo app kết thúc composition (click chuột, đổi focus, Undo…).
#[cfg(windows)]
#[implement(ITfCompositionSink)]
pub struct CompSink {
    _guard: ObjGuard,
    shared: Weak<TsfShared>,
}

#[cfg(windows)]
impl CompSink {
    pub fn new(shared: Weak<TsfShared>) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
        }
    }
}

#[cfg(windows)]
impl ITfCompositionSink_Impl for CompSink_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Ref<'_, ITfComposition>,
    ) -> Result<()> {
        if let Some(shared) = self.shared.upgrade() {
            // Không đoán text app đã giữ lại: chỉ bỏ ownership + reset engine (§4 P1-1).
            shared.terminated.set(true);
            // Composition đang nằm trong slot (không có session nào giữ nó) → dọn
            // ngay. Nếu một KeyEditSession đang giữ nó, cờ `terminated` được session
            // hấp thụ trước khi trả composition về slot.
            if let Ok(mut comp) = shared.composition.try_borrow_mut() {
                if comp.is_some() {
                    shared.absorb_termination(&mut comp);
                }
            }
        }
        Ok(())
    }
}

/// Session xử lý MỘT phím: kết quả `eaten` ghi vào `result`.
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct KeyEditSession {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
    vk: u32,
    mods: u32,
    key: KeyKind,
    result: Rc<Cell<Option<bool>>>,
}

#[cfg(windows)]
impl KeyEditSession {
    pub fn new(
        shared: Rc<TsfShared>,
        ctx: ITfContext,
        vk: u32,
        mods: u32,
        key: KeyKind,
        result: Rc<Cell<Option<bool>>>,
    ) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
            ctx,
            vk,
            mods,
            key,
            result,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for KeyEditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let eaten = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run(ec)))
            .unwrap_or(false);
        self.result.set(Some(eaten));
        Ok(())
    }
}

#[cfg(windows)]
impl KeyEditSession {
    fn run(&self, ec: u32) -> bool {
        let shared = &*self.shared;
        let mut comp = shared
            .composition
            .try_borrow_mut()
            .ok()
            .and_then(|mut c| c.take());
        shared.absorb_termination(&mut comp);

        let eaten = self.process(ec, &mut comp);

        shared.absorb_termination(&mut comp);
        if let Ok(mut slot) = shared.composition.try_borrow_mut() {
            *slot = comp;
        }
        eaten
    }

    fn process(&self, ec: u32, comp: &mut Option<ITfComposition>) -> bool {
        let shared = &*self.shared;
        let Some(sel) = selection_range(&self.ctx, ec) else {
            // Không có selection = không biết chèn vào đâu → không đoán.
            finish(ec, comp, shared);
            return false;
        };

        // Self-heal: caret đã rời composition (click chuột, app tự sửa text) →
        // commit nguyên văn, bắt đầu từ mới tại vị trí caret hiện tại.
        if let Some(c) = comp.as_ref() {
            if !caret_at_composition_end(ec, c, &sel) {
                finish(ec, comp, shared);
            }
        }

        // Security gate S3, đọc lại ở mỗi phím (context có thể dùng chung nhiều field).
        let signals = read_field_signals(&self.ctx, ec, &sel);
        let mode = shared.engine_mode();
        let allowed = match (mode, shared.thread.try_borrow_mut()) {
            (Some(vi_on), Ok(mut thread)) => {
                thread.apply_tsf_probe(classify_tsf_field(&signals));
                // Gate bảo mật luôn xét như đang bật: ô mật khẩu không bao giờ qua engine,
                // kể cả khi chỉ để bung gõ tắt.
                let open =
                    thread.resolve_strategy_with_state(Some(true), None) != Strategy::Passthrough;
                if open {
                    thread.engine.set_enabled(vi_on);
                }
                open
            }
            _ => false,
        };
        if !allowed {
            finish(ec, comp, shared);
            return false;
        }

        // R2-131 (chủ repo: "Ctrl+Space phải xác định từ lỗi trong câu để sửa hết
        // cùng với text prediction"): phím tắt trợ lý Ctrl+Space (R2-136: chỉ Ctrl+Space) —
        // câu có lỗi người dùng sửa được ⇒ SỬA HẾT trước (kể cả chính tả); câu
        // sạch ⇒ engine dự đoán như Tab (bấm tiếp = duyệt vòng); không có gì ⇒
        // phím về app (IntelliSense, menu hệ thống… không bị chiếm vô ích).
        if self.key == KeyKind::CtrlSpace {
            let has_sel = unsafe { sel.IsEmpty(ec) }
                .map(|empty| !empty.as_bool())
                .unwrap_or(false);
            if !has_sel {
                if let Some(eaten) = hotkey_fix_sentence(&self.shared, &self.ctx, ec, comp) {
                    return eaten;
                }
            }
        }

        let step = match shared.thread.try_borrow_mut() {
            Ok(mut thread) => {
                match thread
                    .engine
                    .key_event_raw(self.vk, self.key.engine_ch(), self.mods)
                {
                    Ok(r) if r.flags & IME_FLAG_ERROR == 0 => EngineStep::from_result(&r),
                    _ => {
                        thread.engine.reset();
                        drop(thread);
                        finish(ec, comp, shared);
                        return false;
                    }
                }
            }
            Err(_) => return false,
        };

        let current = match shared.comp_text.try_borrow() {
            Ok(t) => t.clone(),
            Err(_) => return false,
        };
        let plan = plan_key(&current, self.key, &step);
        match apply_plan(&self.shared, &self.ctx, ec, &sel, comp, &plan) {
            Ok(()) => {
                if let Ok(mut text) = shared.comp_text.try_borrow_mut() {
                    *text = if comp.is_some() {
                        text_after(&current, &plan)
                    } else {
                        Vec::new()
                    };
                }
                // AS-022 (D11): điểm kiểm tra silent — MỌI phím trigger
                // space/,/./!/... SAU khi phím đã commit xong (apply_plan thành
                // công ở trên, composition đã đóng ⇒ không đang compose).
                // Lấy câu từ buffer câu của core rồi `ime_assist_check_v1`
                // (UTF-16, D29); lỗi ⇒ toast tray qua IPC. Fail-open + ngân
                // sách (`assist::CHECK_BUDGET_MS`) — không chặn luồng gõ.
                if comp.is_none() {
                    let trigger = super::assist::assist_trigger(self.key);
                    // R2-129: popup sống — ranh giới từ (space , ; :) và Backspace
                    // khi popup sống đang hiện: hiện/cập nhật/ẩn theo câu đúng sai.
                    if trigger == AssistTrigger::Word
                        || (self.key == KeyKind::Backspace && live_popup_visible(shared))
                    {
                        refresh_live_popup(&self.shared, &self.ctx, ec);
                    }
                    let sentence_check = super::assist::after_trigger(shared, trigger);
                    if trigger == AssistTrigger::Sentence
                        && sentence_check.is_none()
                        && live_popup_visible(shared)
                    {
                        hide_assist_overlay(shared); // câu kết thúc đã đúng → ẩn ngay
                    }
                    if let Some(check) = sentence_check {
                        // AS-030 / AS-050: chỉ điểm hiển thị (kết thúc câu, D11)
                        // mới hiện overlay hoặc tự sửa cả câu; space/,/: chỉ toast.
                        if trigger == AssistTrigger::Sentence {
                            let mut auto_fixed = false;
                            if should_auto_fix_sentence(
                                trigger,
                                shared.ipc.is_assist_enabled(),
                                shared.is_assist_auto_fix_enabled(),
                            ) {
                                auto_fixed =
                                    apply_sentence_auto_fix(&self.shared, &self.ctx, ec, &check);
                            }
                            if !auto_fixed {
                                // R2-126: panel sửa lỗi là kênh duy nhất — không
                                // còn toast "phát hiện lỗi" khi panel không hiện.
                                show_assist_overlay(&self.shared, &self.ctx, ec, &check);
                            }
                        }
                    }
                }
                // R2-134: chữ mờ gợi ý sẵn sau mỗi phím gõ (Tab/phím tắt sẽ chèn đúng nó).
                if matches!(
                    self.key,
                    KeyKind::Char(_) | KeyKind::Backspace | KeyKind::Tab
                ) {
                    update_ghost(&self.shared, &self.ctx, ec);
                }
                if self.key == KeyKind::CtrlSpace && step.action == textvn_ffi::ACTION_PASS {
                    // Ctrl+Space không khớp prediction: kích hoạt Trợ lý ngữ pháp (Text Correct) cho từ và câu
                    let has_sel = unsafe { sel.IsEmpty(ec) }
                        .map(|empty| !empty.as_bool())
                        .unwrap_or(false);
                    if has_sel {
                        let auto_cap = shared
                            .thread
                            .try_borrow()
                            .map(|t| t.auto_capitalize())
                            .unwrap_or(true);
                        let cap = crate::selection_check::MAX_SELECTION_CHARS + 2;
                        let mut buf = vec![0u16; cap];
                        let mut fetched = 0u32;
                        if (unsafe { sel.GetText(ec, 0, &mut buf, &mut fetched) }).is_ok()
                            && fetched > 0
                        {
                            let n = (fetched as usize).min(buf.len());
                            if n <= crate::selection_check::MAX_SELECTION_CHARS {
                                if let Ok(text) = String::from_utf16(&buf[..n]) {
                                    if text.chars().any(|c| c.is_alphabetic()) {
                                        if let Some(check) =
                                            crate::assist::check_text_fragment(&text, auto_cap)
                                        {
                                            let anchor = unsafe {
                                                crate::selection_check::get_selection_anchor(
                                                    &self.ctx, &sel, ec,
                                                )
                                            };
                                            // Chỉ ăn phím khi popup THẬT SỰ hiện
                                            // (display tắt / không dựng được view
                                            // → phím về app như thường).
                                            if show_assist_overlay_for_selection(
                                                &self.shared,
                                                &self.ctx,
                                                ec,
                                                &check,
                                                sel.clone(),
                                                anchor,
                                            ) {
                                                mark_assist_on_demand(shared);
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Không bôi đen: kiểm tra từ / câu ở vị trí con trỏ (on-demand)
                        // Chỉ ăn phím khi panel THẬT SỰ hiện (R2-126: không toast
                        // thay thế, không nuốt Ctrl+Space của app một cách vô ích).
                        if let Some(check) = crate::assist::check_sentence_on_demand(shared) {
                            if show_assist_overlay(&self.shared, &self.ctx, ec, &check) {
                                mark_assist_on_demand(shared);
                                return true;
                            }
                        }
                    }
                    // R2-121: Ctrl+Space tới app (không dự đoán, không lỗi) — như
                    // chord B6: app có thể chèn dấu cách (Win32 Edit sinh WM_CHAR
                    // 0x20 cho Ctrl+Space) hay dời caret mà engine không thấy →
                    // quên đuôi text, nếu không buffer câu/gõ tắt lệch document.
                    shared.reset_engine();
                    return false;
                }
                plan.eaten
            }
            Err(_) => {
                // Áp thất bại giữa chừng: commit phần đang có, reset, phím đi thẳng.
                shared.reset_engine();
                finish(ec, comp, shared);
                false
            }
        }
    }
}

/// R2-134: vẽ (hoặc ẩn) chữ mờ gợi ý theo `peek_completion` của engine. Chỉ vẽ
/// khi đo được caret CHÍNH XÁC (`GetTextExt` — không dùng neo dự phòng của panel)
/// và sau caret không còn chữ trên dòng (không đè lên chữ thật).
#[cfg(windows)]
fn update_ghost(shared: &Rc<TsfShared>, ctx: &ITfContext, ec: u32) {
    let ghost = shared
        .thread
        .try_borrow()
        .ok()
        .and_then(|thread| thread.engine.peek_completion());
    let after = ghost.is_some() && text_after_caret(ctx, ec);
    let anchor = ghost
        .as_ref()
        .filter(|_| !after)
        .and_then(|_| tsf_caret_anchor(ctx, ec));
    if ghost.is_some() && anchor.is_none() {
        // Chỉ lý do, không chữ (S2).
        crate::trace::event(
            shared.tid,
            format_args!(
                "ghost skip: {}",
                if after {
                    "text after caret"
                } else {
                    "no caret rect"
                }
            ),
        );
    }
    match (ghost, anchor) {
        (Some(text), Some(anchor)) => {
            let font_px = estimate_font_px(ctx, ec, anchor.bottom - anchor.top);
            // SAFETY: chỉ đọc HWND foreground để watchdog tắt khi đổi app.
            let owner = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }.0
                as isize;
            shared.ghost.show(&text, anchor, owner, font_px);
        }
        _ => shared.ghost.hide(),
    }
}

/// Cỡ chữ (px, chiều cao em) để chữ mờ trông như chữ của app. Caret rect gồm cả
/// giãn dòng (Notepad Win11: caret ~2× chữ — đo thật), nên lấy thêm ước lượng từ
/// bề rộng của từ vừa gõ (≈ 0,43 em/ký tự — trung bình chữ Việt thường ở font tỉ
/// lệ; font đơn cách rộng hơn chút) và chọn số nhỏ hơn.
#[cfg(windows)]
fn estimate_font_px(ctx: &ITfContext, ec: u32, caret_h: i32) -> i32 {
    let by_caret = (caret_h * 4 / 5).max(10);
    // SAFETY: trong edit session; chỉ đọc chữ/toạ độ tối đa 8 ký tự trước caret.
    let by_word = unsafe {
        (|| -> Option<i32> {
            let view = ctx.GetActiveView().ok()?;
            let sel = selection_range(ctx, ec)?;
            let back = sel.Clone().ok()?;
            back.Collapse(ec, TF_ANCHOR_END).ok()?;
            let mut shifted = 0i32;
            back.ShiftStart(ec, -8, &mut shifted, std::ptr::null())
                .ok()?;
            let mut buf = [0u16; 16];
            let mut fetched = 0u32;
            back.GetText(ec, 0, &mut buf, &mut fetched).ok()?;
            let text = &buf[..(fetched as usize).min(buf.len())];
            // Chỉ phần chữ liền trước caret (từ vừa gõ), không qua khoảng trắng.
            let word = text
                .iter()
                .rev()
                .take_while(|&&u| u > 0x20 && !(0xD800..=0xDFFF).contains(&u))
                .count();
            if word < 2 {
                return None;
            }
            let start = word as i32 - text.len() as i32;
            let mut moved = 0i32;
            back.ShiftStart(ec, -start, &mut moved, std::ptr::null())
                .ok()?;
            let mut rc = windows::Win32::Foundation::RECT::default();
            let mut clipped = windows::core::BOOL::default();
            view.GetTextExt(ec, &back, &mut rc, &mut clipped).ok()?;
            let width = rc.right - rc.left;
            (width > 0).then(|| width * 23 / (10 * word as i32))
        })()
    };
    by_word.map_or(by_caret, |w| w.clamp(8, by_caret))
}

/// Sau caret còn chữ NHÌN THẤY trên cùng dòng (đang sửa giữa câu) không —
/// khoảng trắng cuối dòng không tính (vẽ đè lên khoảng trống không che chữ).
/// VS Code/Monaco: textarea ẩn không phản ánh đúng chữ sau caret (đo thật) ⇒
/// coi như còn chữ ⇒ không vẽ (editor đó vốn có chữ mờ riêng — Copilot).
#[cfg(windows)]
fn text_after_caret(ctx: &ITfContext, ec: u32) -> bool {
    let Some(sel) = selection_range(ctx, ec) else {
        return true; // không biết ⇒ không vẽ
    };
    // SAFETY: trong edit session (ec hợp lệ); chỉ đọc 1 ký tự sau caret.
    unsafe {
        let Ok(next) = sel.Clone() else {
            return true;
        };
        let _ = next.Collapse(ec, TF_ANCHOR_END);
        let mut shifted = 0i32;
        if next
            .ShiftEnd(ec, 32, &mut shifted, std::ptr::null())
            .is_err()
            || shifted <= 0
        {
            return false; // cuối document
        }
        let mut buf = [0u16; 34];
        let mut fetched = 0u32;
        if next.GetText(ec, 0, &mut buf, &mut fetched).is_err() || fetched == 0 {
            return false;
        }
        let after = &buf[..(fetched as usize).min(buf.len())];
        after
            .iter()
            .take_while(|&&u| !matches!(u, 0x0D | 0x0A | 0x0B))
            .any(|&u| !matches!(u, 0x20 | 0x09 | 0xA0))
    }
}

/// R2-131: bước "sửa hết" của phím tắt trợ lý. `None` = câu không có lỗi người
/// dùng sửa được (đi tiếp sang dự đoán); `Some(eaten)` = đã quyết định.
#[cfg(windows)]
fn hotkey_fix_sentence(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    comp: &mut Option<ITfComposition>,
) -> Option<bool> {
    let check = crate::assist::check_sentence_on_demand(shared)?;
    let plan = {
        let thread = shared.thread.try_borrow().ok()?;
        let glyph_len = |text: &str| thread.engine.glyph_len_of(text);
        plan_sentence_user_fix(&check.sentence, &check.all_issue_views(), glyph_len)?
    };
    // Chốt từ đang gõ để chữ thật nằm trong document trước khi lùi — KHÔNG reset
    // engine (`finish`) vì precondition `buffer_is_ours` so buffer câu của engine.
    let was_composing = comp.is_some();
    if let Some(c) = comp.take() {
        // SAFETY: trong DoEditSession với ec hợp lệ; composition của TIP.
        let _ = unsafe { c.EndComposition(ec) };
    }
    if let Ok(mut text) = shared.comp_text.try_borrow_mut() {
        text.clear();
    }
    hide_assist_overlay(shared);
    let session =
        ApplySentenceFixSession::new(shared.clone(), ctx.clone(), check, plan, false, false);
    if was_composing {
        // R2-135: VS Code/Monaco (Electron) dựng lại thay đổi từ trạng thái
        // textarea và vẽ lại textarea BẤT ĐỒNG BỘ sau khi composition kết thúc —
        // sửa câu ngay (cùng session, hay session async kế tiếp) bị gộp thành một
        // thay đổi sai (đo thật: "tôi tôi đi học" → "tôiiot phần đi học"). Hẹn
        // sửa sau `DEFERRED_FIX_MS` như một cú click "Sửa tất cả" (đường đó đúng
        // trên VS Code); gõ phím khác trong lúc chờ ⇒ buffer câu đổi ⇒ session tự
        // bỏ (precondition `buffer_is_ours`).
        let scheduled = schedule_deferred_fix(shared, ctx, session);
        crate::trace::event(
            shared.tid,
            format_args!("assist hotkey fix: deferred scheduled={scheduled}"),
        );
        if !scheduled {
            shared.reset_engine();
        }
        return Some(scheduled);
    }
    let outcome = session.apply_outcome(ec);
    crate::trace::event(shared.tid, format_args!("assist hotkey fix: {outcome:?}"));
    match outcome {
        FixOutcome::Applied => {
            recheck_after_fix(shared, ctx, ec, &session.fixed_sentence, false);
            Some(true)
        }
        FixOutcome::Scheduled => Some(true),
        FixOutcome::Skipped => {
            // Composition đã đóng mà engine còn giữ từ: quên đuôi rồi để phím đi.
            shared.reset_engine();
            Some(false)
        }
    }
}

/// Timer của bước sửa hẹn giờ (R2-135) — overlay WndProc so khớp hằng này.
#[cfg(windows)]
pub(crate) const TIMER_DEFERRED_FIX: usize = 0x5EC2;
/// Đủ để Chromium/Monaco nhận chữ vừa chốt và vẽ lại textarea (đo thật VS Code).
#[cfg(windows)]
const DEFERRED_FIX_MS: u32 = 120;

#[cfg(windows)]
thread_local! {
    /// Bước sửa câu đang chờ timer (thread UI của app).
    static DEFERRED_FIX: RefCell<Option<(std::rc::Weak<TsfShared>, ApplySentenceFixSession)>> =
        const { RefCell::new(None) };
}

/// Hẹn chạy `session` sau `DEFERRED_FIX_MS` trên timer của overlay (ngoài luồng
/// phím, như click chuột — `request_sync_or_async`).
#[cfg(windows)]
fn schedule_deferred_fix(
    shared: &Rc<TsfShared>,
    _ctx: &ITfContext,
    session: ApplySentenceFixSession,
) -> bool {
    let hwnd = shared.overlay.ensure_timer_hwnd();
    if hwnd == 0 {
        return false;
    }
    DEFERRED_FIX.with(|slot| *slot.borrow_mut() = Some((Rc::downgrade(shared), session)));
    // SAFETY: hwnd là overlay của chính thread này.
    let timer = unsafe {
        windows::Win32::UI::WindowsAndMessaging::SetTimer(
            Some(windows::Win32::Foundation::HWND(hwnd as *mut _)),
            TIMER_DEFERRED_FIX,
            DEFERRED_FIX_MS,
            None,
        )
    };
    timer != 0
}

/// Timer fire (gọi từ `overlay_wnd_proc` WM_TIMER) → xin session sửa câu.
#[cfg(windows)]
pub(crate) fn on_deferred_fix_timer(hwnd: windows::Win32::Foundation::HWND) {
    // SAFETY: timer của chính cửa sổ này; một lần fire là đủ.
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), TIMER_DEFERRED_FIX);
    }
    let Some((weak, session)) = DEFERRED_FIX.with(|slot| slot.borrow_mut().take()) else {
        return;
    };
    let Some(shared) = weak.upgrade() else {
        return;
    };
    let ctx = session.ctx.clone();
    let session: ITfEditSession = session.into();
    let requested = request_sync_or_async(&ctx, shared.tid, &session);
    crate::trace::event(
        shared.tid,
        format_args!("assist hotkey fix: timer requested={requested}"),
    );
}

/// Session kết thúc composition (focus đổi, chord, tắt bộ gõ, Deactivate).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct EndCompositionSession {
    _guard: ObjGuard,
    composition: ITfComposition,
}

#[cfg(windows)]
impl EndCompositionSession {
    pub fn new(composition: ITfComposition) -> Self {
        Self {
            _guard: ObjGuard::new(),
            composition,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for EndCompositionSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        // SAFETY: composition thuộc TIP này, ec do TSF cấp cho session hiện hành.
        unsafe { self.composition.EndComposition(ec) }
    }
}

// ---------------------------------------------------------------------------
// L3 (P4-0 §5.1) — "thừa khoảng trắng trước dấu câu": `Fix::Delete` ≠ thay span
// ---------------------------------------------------------------------------

// Engine sửa L3 bằng `Fix::Delete` — **XOÁ** span (khoảng trắng thừa); `suggest`
// của lỗi này chỉ là **đầu câu để HIỂN THỊ** (dấu câu), KHÔNG phải text thay
// thế. Adapter coi `suggest` là text thay thế ⇒ sinh "chào bạn,," (nhân đôi
// dấu câu) — lệch với mô hình Rust (`assist/src/rules/space_before_punct.rs`)
// và với `adapters/linux-common/src/compose.c` (đã sửa) ⇒ lệch parity 3 OS.
//
// Xem `assist/src/rules/space_before_punct.rs`: span = khoảng trắng, fix =
// `Fix::Delete { start, end }`, suggestion = text của dấu câu.
//
// A2 **không** cần nhánh riêng: [`crate::assist::plan_fix`] đã quyết định theo
// **kind** (`IME_ASSIST_KIND_SPACE_BEFORE_PUNCT` ⇒ xoá span) — cùng nguồn với
// A3 ([`crate::assist::plan_sentence_auto_fix`]) ⇒ không lệch parity 3 OS.

// ---------------------------------------------------------------------------
// AS-030 (Mode 1) — overlay cạnh caret
// ---------------------------------------------------------------------------

/// Ẩn overlay + bỏ model (câu người dùng không nằm lại RAM lâu hơn cần — S2).
#[cfg(windows)]
pub fn hide_assist_overlay(shared: &TsfShared) {
    shared.overlay.hide();
    if let Ok(mut slot) = shared.assist.try_borrow_mut() {
        *slot = None;
    }
}

/// `↑`/`↓`: đổi lỗi đang chọn (vòng tròn, chỉ khi overlay đang hiện).
#[cfg(windows)]
pub fn select_assist_issue(shared: &TsfShared, delta: isize) {
    let mut updated = None;
    if let Ok(mut slot) = shared.assist.try_borrow_mut() {
        if let Some(active) = slot.as_mut() {
            if active.view.select(delta) {
                updated = Some(active.view.clone());
            }
        }
    }
    // `set_view` gọi Win32 — chỉ sau khi đã trả borrow (không mượn chéo callback).
    if let Some(view) = updated {
        shared.overlay.set_view(&view);
    }
}

/// Lỗi đang chọn của overlay (0 = không có overlay) — cho `Enter`.
#[cfg(windows)]
pub fn selected_assist_issue(shared: &TsfShared) -> usize {
    shared
        .assist
        .try_borrow()
        .ok()
        .and_then(|slot| slot.as_ref().map(|a| a.view.selected))
        .unwrap_or(0)
}

/// Hiện overlay cho kết quả kiểm tra vừa có (chỉ **điểm hiển thị** — D11/D38).
///
/// Gọi trong `DoEditSession`: chỉ đọc TSF (caret rect + selection) rồi hiện
/// panel Win32; **không** gọi TSF từ window proc/timer (P4-10 · reentrancy).
/// Trả `true` = panel đang hiện và đã giữ state cho A2.
#[cfg(windows)]
pub fn show_assist_overlay(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    check: &AssistCheck,
) -> bool {
    show_assist_overlay_with_recheck(shared, ctx, ec, check, false)
}

#[cfg(windows)]
pub(crate) fn show_assist_overlay_with_recheck(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    check: &AssistCheck,
    from_recheck: bool,
) -> bool {
    // Trace chẩn đoán (`TEXTVN_TSF_TRACE`): chỉ LÝ DO + số đếm, không chữ (S2).
    let skip = |why: &str| {
        crate::trace::event(shared.tid, format_args!("assist overlay skip: {why}"));
        false
    };
    if !shared.is_assist_display_enabled() {
        return skip("display off"); // D20 kill-switch + `assist.mode` (R-6: giống macOS/Linux)
    }
    let Some(view) = check.view() else {
        return skip("no view");
    };
    if !crate::assist::should_show_overlay(AssistTrigger::Sentence, true, view.issues.len()) {
        return skip("no issue");
    }
    let Some(anchor) = caret_anchor(ctx, ec, shared.tid) else {
        return skip("no anchor"); // không neo được (app không cho toạ độ) → im lặng
    };
    let Some(caret) = selection_range(ctx, ec) else {
        return skip("no selection");
    };
    // Chỉ neo khi selection là caret rỗng: bôi đen = người dùng đang chọn text,
    // sửa lúc này là ghi đè lựa chọn của họ (D12 fail-safe).
    if !matches!(unsafe { caret.IsEmpty(ec) }, Ok(empty) if empty.as_bool()) {
        return skip("selection not empty");
    }
    // SAFETY: foreground window chỉ để watchdog biết "app đổi foreground"; 0 =
    // không kiểm tra (không lấy được).
    let owner =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }.0 as isize;
    let model = ActiveAssist {
        view: view.clone(),
        sentence: check.sentence.clone(),
        caret,
        ctx: ctx.clone(),
        check: check.clone(),
        selection: None,
        from_recheck,
        on_demand: false,
    };
    let Ok(mut slot) = shared.assist.try_borrow_mut() else {
        return false;
    };
    *slot = Some(model);
    drop(slot);
    shared.overlay.show(&view, anchor, owner);
    if !shared.overlay.is_visible() {
        // Không tạo được cửa sổ (thiếu quyền/GDI lỗi): bỏ state, không để A2
        // sửa text mà người dùng không thấy gì.
        hide_assist_overlay(shared);
        return skip("window not visible");
    }
    crate::trace::event(
        shared.tid,
        format_args!("assist overlay shown issues={}", view.issues.len()),
    );
    true
}

/// Vòng 30 — hiện popup cho **vùng bôi đen** (selection-check): như
/// `show_assist_overlay` nhưng (a) KHÔNG yêu cầu selection rỗng — selection
/// chính là đối tượng; (b) anchor = rect vùng chọn (GetTextExt); (c) giữ
/// range trong `ActiveAssist.selection` để FixAll SetText đúng vùng.
#[cfg(windows)]
pub fn show_assist_overlay_for_selection(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    check: &AssistCheck,
    range: windows::Win32::UI::TextServices::ITfRange,
    anchor: Option<crate::overlay::Anchor>,
) -> bool {
    if !shared.is_assist_display_enabled() {
        return false;
    }
    let Some(mut view) = check.view() else {
        return false;
    };
    // Popup bôi đen là chế độ chọn-sửa (Enter = sửa tất cả vùng chọn) — R2-129.
    view.interactive = true;
    if !crate::assist::should_show_overlay(
        crate::assist::AssistTrigger::Sentence,
        true,
        view.issues.len(),
    ) {
        return false;
    }
    // SAFETY: foreground chỉ cho watchdog (0 = không kiểm).
    let owner =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }.0 as isize;
    let Some(caret) = selection_range(ctx, ec) else {
        return false;
    };
    let model = ActiveAssist {
        view: view.clone(),
        sentence: check.sentence.clone(),
        caret,
        ctx: ctx.clone(),
        check: check.clone(),
        selection: Some(range),
        from_recheck: false,
        on_demand: false,
    };
    {
        let Ok(mut slot) = shared.assist.try_borrow_mut() else {
            return false;
        };
        *slot = Some(model);
    }
    // Anchor: ưu tiên rect vùng chọn; không có thì fallback caret-anchor thường.
    let anchor = anchor.or_else(|| caret_anchor(ctx, ec, shared.tid));
    let Some(anchor) = anchor else {
        if let Ok(mut s) = shared.assist.try_borrow_mut() {
            *s = None;
        }
        return false;
    };
    shared.overlay.show(&view, anchor, owner);
    if !shared.overlay.is_visible() {
        hide_assist_overlay(shared);
        return false;
    }
    true
}

/// A2: áp gợi ý của lỗi `issue_index`. Trả `true` = đã xử lý phím (overlay đóng);
/// `false` = không sửa được (fail-safe — chỉ gợi ý, D12).
#[cfg(windows)]
pub fn apply_assist_suggestion(
    shared: &Rc<TsfShared>,
    ctx: Option<&ITfContext>,
    issue_index: usize,
) -> bool {
    let taken = shared
        .assist
        .try_borrow_mut()
        .ok()
        .and_then(|mut slot| slot.take());
    hide_assist_overlay(shared);
    let Some(active) = taken else {
        return false;
    };
    // Backspace+Type cần edit session; không có ctx = không sửa (không đoán).
    let Some(ctx) = ctx else {
        return false;
    };
    // Số glyph cần xoá do **core** tính trên câu của ta (D27) — không tự đổi
    // char → glyph ở adapter.
    let plan = {
        let Ok(thread) = shared.thread.try_borrow() else {
            return false;
        };
        let glyph_len = |text: &str| thread.engine.glyph_len_of(text);
        // Ngữ nghĩa `Fix` quyết định theo **kind** FFI (mỗi lỗi trong `view` mang
        // kind): `Fix::Delete` (L3/lặp từ) ⇒ **xoá** span; `Fix::Replace` ⇒
        // **thay** span bằng gợi ý. KHÔNG đoán bằng text (Q-2: so sánh
        // `span.to_lowercase() == suggest.to_lowercase()` làm L6 rơi vào nhánh xoá
        // ⇒ mất 1 ký tự).
        plan_fix(&active.view, issue_index, glyph_len)
    };
    let Some(plan) = plan else {
        return false; // không có gợi ý / gợi ý vô nghĩa → chỉ hiển thị (A1)
    };
    if let Some(range) = active.selection.clone() {
        let head_len = active
            .sentence
            .chars()
            .count()
            .saturating_sub(plan.delete_chars);
        let head: String = active.sentence.chars().take(head_len).collect();
        let fixed = head + &plan.replacement.iter().collect::<String>();
        return crate::selection_check::request_apply_selection_fix(
            shared,
            &active.ctx,
            &range,
            &active.sentence,
            &fixed,
        );
    }
    let session: ITfEditSession =
        ApplyAssistFixSession::new(shared.clone(), ctx.clone(), active, plan).into();
    request_sync_or_async(ctx, shared.tid, &session)
}

/// Xin edit session: ưu tiên đồng bộ (biết ngay kết quả); TSF từ chối thì xin
/// bất đồng bộ. `false` = không session nào được cấp.
#[cfg(windows)]
pub(crate) fn request_sync_or_async(ctx: &ITfContext, tid: u32, session: &ITfEditSession) -> bool {
    // SAFETY: `ctx`/`session` hợp lệ trong callback của TSF.
    let hr = unsafe { ctx.RequestEditSession(tid, session, TF_ES_SYNC | TF_ES_READWRITE) };
    if matches!(hr, Ok(code) if code.is_ok()) {
        return true;
    }
    matches!(
        unsafe { ctx.RequestEditSession(tid, session, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE) },
        Ok(code) if code.is_ok()
    )
}

// ---------------------------------------------------------------------------
// AS-031 (A2) — sửa MỘT từ bằng Backspace+Type (D12/D22/D27)
// ---------------------------------------------------------------------------

/// Session áp dụng sửa lỗi ngữ pháp một từ (AS-031 · A2 / D12).
///
/// **D22:** chỉ Backspace+Type — lùi `delete_glyphs` glyph (D27) rồi gõ lại
/// `suggestion + đuôi` **nguyên văn** từ buffer của core; **không**
/// `SelectionReplace`. Cả xoá lẫn gõ lại nằm trong **một** edit session ⇒
/// Ctrl+Z hoàn tác trọn một lần.
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct ApplyAssistFixSession {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
    /// Model + câu + caret đã chụp lúc hiện overlay (precondition D12).
    active: ActiveAssist,
    plan: FixPlan,
    /// Câu đầy đủ sau fix (head + replacement) — re-check sau session.
    fixed_sentence: String,
}

/// Vòng 28 — re-check sau khi A2/A3 sửa xong: chạy `check_text` trên câu
/// đầy đủ đã sửa (head giữ nguyên + đuôi mới), còn lỗi thì hiện lại panel
/// với tổng lỗi mới; sạch thì panel giữ ẩn. `was_auto_fix` = chỉ lần tự sửa
/// tự động của Mode 2 mới tôn trọng `hide_on_auto_fix`; người dùng chủ động
/// sửa (A2 / nút "Sửa tất cả") vẫn hiện lại nếu còn lỗi.
#[cfg(windows)]
pub(crate) fn recheck_after_fix(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    fixed_sentence: &str,
    was_auto_fix: bool,
) {
    let (auto_capitalize, hide_on_auto_fix, mode2) = {
        let Ok(thread) = shared.thread.try_borrow() else {
            return;
        };
        (
            thread.auto_capitalize(),
            thread.assist_display_hide_on_auto_fix(),
            crate::assist::mode2_auto_fix_enabled(thread.assist_mode(), thread.assist_auto_fix()),
        )
    };
    if was_auto_fix && mode2 && hide_on_auto_fix {
        return; // người dùng chọn ẩn panel khi auto-fix đang chạy
    }
    // Câu đầy đủ sau fix = phần đầu giữ nguyên + đuôi mới (engine đã reset,
    // không đọc lại buffer — ta BIẾT chắc text vừa ghi).
    let Some(new_check) = super::assist::check_text(fixed_sentence, auto_capitalize) else {
        return; // câu sạch → panel giữ ẩn
    };
    let _ = show_assist_overlay_with_recheck(shared, ctx, ec, &new_check, true);
}

#[cfg(windows)]
impl ApplyAssistFixSession {
    pub fn new(
        shared: Rc<TsfShared>,
        ctx: ITfContext,
        active: ActiveAssist,
        plan: FixPlan,
    ) -> Self {
        // Câu sau fix = head (giữ nguyên trước issue.start) + đuôi mới.
        let head_len = active.sentence.chars().count() - plan.delete_chars;
        let head: String = active.sentence.chars().take(head_len).collect();
        let fixed = head + &plan.replacement.iter().collect::<String>();
        Self {
            _guard: ObjGuard::new(),
            shared,
            ctx,
            active,
            plan,
            fixed_sentence: fixed,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for ApplyAssistFixSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.apply(ec)))
            .unwrap_or(false);
        // Dù sửa được hay không, sau khi session này chạm document thì buffer
        // engine không còn chắc khớp: xoá để phím sau bắt đầu sạch (không lệch);
        // sửa xong thì nạp câu đã sửa làm ngữ cảnh dự đoán (R2-131).
        if applied {
            self.shared.reset_engine_with_tail(&self.fixed_sentence);
        } else {
            self.shared.reset_engine();
        }
        self.shared.clear_assist_undo();
        if applied {
            crate::trace::event(self.shared.tid, format_args!("assist A2 applied"));
            // Vòng 28: re-check câu ĐÃ sửa — sạch thì panel tự tắt (đã ẩn
            // trước session), còn lỗi thì panel cập nhật hiển thị phần còn lại.
            recheck_after_fix(&self.shared, &self.ctx, ec, &self.fixed_sentence, false);
        }
        Ok(())
    }
}

#[cfg(windows)]
impl ApplyAssistFixSession {
    /// Trả `true` = document đã nhận text thay thế. Mọi điều kiện sai → `false`
    /// và **không** đụng document.
    fn apply(&self, ec: u32) -> bool {
        // SAFETY: mọi lời gọi TSF dưới đây nằm trong DoEditSession với `ec` hợp lệ.
        unsafe {
            let Some(sel) = selection_range(&self.ctx, ec) else {
                return false;
            };
            // -- Precondition 1: selection là caret rỗng (không bôi đen) --
            let selection_empty = matches!(sel.IsEmpty(ec), Ok(empty) if empty.as_bool());
            // -- Precondition 2: caret vẫn đúng vị trí đã chụp (chỉ so vị trí) --
            let caret_unchanged = self
                .active
                .caret
                .CompareStart(ec, &sel, TF_ANCHOR_START)
                .map(|c| c == 0)
                .unwrap_or(false)
                && self
                    .active
                    .caret
                    .CompareEnd(ec, &sel, TF_ANCHOR_END)
                    .map(|c| c == 0)
                    .unwrap_or(false);
            // -- Precondition 3: buffer câu của core vẫn là câu lúc hiện overlay
            //    (hoặc popup được dựng ngay từ `recheck_after_fix`, khi đó
            //    Precondition 5 `verify_delete_range` bên dưới xác minh nội dung
            //    thật trong document trước caret). --
            let buffer_is_ours = self.active.from_recheck
                || self
                    .shared
                    .thread
                    .try_borrow()
                    .map(|t| t.engine.sentence_tail(SENTENCE_MAX_CHARS) == self.active.sentence)
                    .unwrap_or(false);

            // -- Precondition 4: text store cho lùi ĐÚNG delete_glyphs glyph --
            let Some(want) = backspace_shift(self.plan.delete_glyphs) else {
                return false; // 0 glyph / tràn i32 → không sửa
            };
            let Ok(del) = sel.Clone() else {
                return false;
            };
            let mut shifted = 0i32;
            let backspace_shift_ok = del
                .ShiftStart(ec, want, &mut shifted, std::ptr::null())
                .is_ok()
                && shifted == want;
            // R2-132: app CUAS (Notepad++…) không cho lùi qua chữ đã commit —
            // hẹn sửa bằng Backspace + WM_CHAR, xác minh document feed lúc giao
            // (thay cho `caret_unchanged`/snapshot không đọc được qua TSF).
            if !backspace_shift_ok && selection_empty && buffer_is_ours {
                let keep = self
                    .active
                    .sentence
                    .chars()
                    .count()
                    .saturating_sub(self.plan.delete_chars);
                let tail: String = self.active.sentence.chars().skip(keep).collect();
                let replacement: String = self.plan.replacement.iter().collect();
                if crate::cuas::schedule_tail_fix(
                    self.shared.tid,
                    &self.ctx,
                    &tail,
                    self.plan.delete_glyphs,
                    &replacement,
                ) {
                    return false; // đã hẹn — DoEditSession reset engine như mọi lần
                }
            }

            let pre = FixPrecondition {
                selection_empty,
                caret_unchanged,
                buffer_is_ours,
                backspace_shift_ok,
            };
            if !can_apply_fix(pre) {
                crate::trace::event(self.shared.tid, format_args!("assist A2 skip: pre"));
                return false;
            }

            // Chụp vùng sắp xoá để hoàn tác nếu gõ lại thất bại TRƯỚC khi text
            // mới vào document (R2-103). 1 glyph ≤ 2 UTF-16 unit (emoji) → 2×.
            let mut buf = vec![0u16; self.plan.delete_glyphs as usize * 2 + 2];
            let mut fetched = 0u32;
            let Some(snapshot) = del
                .GetText(ec, 0, &mut buf, &mut fetched)
                .ok()
                .map(|_| buf[..(fetched as usize).min(buf.len())].to_vec())
                .filter(|text| !text.is_empty())
            else {
                // Không đọc được vùng sắp xoá ⇒ **không xác minh được ⇒ không sửa**
                // (D12 fail-safe) — và không có bản sao để hoàn tác.
                crate::trace::event(self.shared.tid, format_args!("assist A2 skip: no snapshot"));
                return false;
            };

            // -- Precondition 5 (R-2): vùng sắp xoá phải **đúng** là đuôi câu mà
            //    engine biết (chúng ta tự gõ). Lệch ⇒ caret không ở cuối câu
            //    (user bấm chuột đặt caret chỗ khác, app tự sửa…) ⇒ KHÔNG xoá.
            let expected = expected_tail(&self.active.sentence, self.plan.delete_chars);
            if !verify_delete_range(&snapshot, &expected, self.plan.delete_glyphs) {
                crate::trace::event(
                    self.shared.tid,
                    format_args!("assist A2 skip: tail mismatch"),
                );
                return false;
            }

            // -- Backspace: xoá đúng vùng từ issue.start tới hết câu --
            let empty = [0u16; 1];
            if del.SetText(ec, 0, &empty[..0]).is_err() {
                return false;
            }

            // -- Type: gõ lại `suggestion + đuôi` qua composition --
            let inserted = (|| -> Result<()> {
                let ctxcomp: ITfContextComposition = self.ctx.cast()?;
                let sink: ITfCompositionSink = CompSink::new(Rc::downgrade(&self.shared)).into();
                let start = sel.Clone()?;
                let comp = ctxcomp.StartComposition(ec, &start, &sink)?;
                let range = comp.GetRange()?;
                let wide = to_wide_nul(&self.plan.replacement);
                range.SetText(ec, 0, &wide[..wide.len() - 1])?;
                // KHÔNG gắn display attribute: text này là text thật của app,
                // không được đổi format (D4) — gạch đỏ chỉ nằm trong overlay.
                set_caret_after(&self.ctx, ec, &range);
                // Text đã vào document: lỗi EndComposition KHÔNG kích hoạt hoàn
                // tác (R2-103) — app kết thúc composition là nguyên nhân thường
                // gặp, hoàn tác lúc này sẽ nhân đôi chữ.
                let _ = comp.EndComposition(ec);
                Ok(())
            })();

            if inserted.is_err() {
                crate::trace::event(self.shared.tid, format_args!("assist A2 restore"));
                let _ = del.SetText(ec, 0, &snapshot);
                set_caret_after(&self.ctx, ec, &del);
                return false;
            }
            true
        }
    }
}

/// Số UTF-16 unit phải lùi (âm) để Backspace đúng `delete_glyphs` glyph; `None`
/// = 0 hoặc vượt `i32` ⇒ **không sửa** (fail-safe).
#[cfg(windows)]
fn backspace_shift(delete_glyphs: u32) -> Option<i32> {
    if delete_glyphs == 0 {
        return None;
    }
    i32::try_from(delete_glyphs).ok().map(|n| -n)
}

/// Đuôi câu (Unicode) mà engine biết: `delete_chars` ký tự cuối của `sentence`.
///
/// `delete_chars` do `plan_fix` tính từ **đuôi** của câu lúc hiện overlay ⇒
/// đuôi `sentence` chính là vùng sắp xoá (D12 · R-2).
#[cfg(windows)]
fn expected_tail(sentence: &str, delete_chars: usize) -> String {
    let chars: Vec<char> = sentence.chars().collect();
    let take = delete_chars.min(chars.len());
    chars[chars.len() - take..].iter().collect()
}

/// **Xác minh vùng sắp xoá** (R-2 · D12 fail-safe).
///
/// `snapshot` = text thật trong document (UTF-16) của vùng sắp lùi `delete_glyphs`
/// code unit; `expected` = đuôi câu mà **engine** biết (Unicode).
///
/// - Không đọc được / lệch ⇒ **không sửa** (fail-safe): đây là chốt chặn
///   "xoá nhầm text lạ" khi engine buffer stale (user bấm chuột đặt caret chỗ
///   khác rồi gõ `.`).
/// - `expected` là **Unicode**, document chứa **glyph** theo bảng mã xuất. Với
///   `unicode_precomposed` (mặc định) 1 char = 1 code unit nên so trực tiếp;
///   bảng mã khác (TCVN3/VNI/Unicode tổ hợp) ⇒ số code unit lệch ⇒ **không
///   xác minh được ⇒ không sửa** (an toàn hơn là sửa nhầm).
#[cfg(windows)]
fn verify_delete_range(snapshot: &[u16], expected: &str, delete_glyphs: u32) -> bool {
    if snapshot.is_empty() || delete_glyphs == 0 {
        return false; // vùng rỗng ⇒ không xác minh được ⇒ không sửa
    }
    let expected_utf16: Vec<u16> = expected.encode_utf16().collect();
    if expected_utf16.len() != delete_glyphs as usize {
        return false; // số glyph ≠ số code unit ⇒ không xác minh được
    }
    snapshot == expected_utf16.as_slice()
}

// ---------------------------------------------------------------------------
// AS-050 (A3) — tự sửa cả câu bằng Backspace+Type (Mode 2 / Precondition D12)
// ---------------------------------------------------------------------------

/// Áp dụng tự sửa cả câu (AS-050 · Mode 2 / A3) bằng Backspace+Type.
/// Trả `true` = đã sửa thành công; `false` = không sửa (fail-safe).
#[cfg(windows)]
pub fn apply_sentence_auto_fix(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    check: &AssistCheck,
) -> bool {
    // R-11: khi core không đếm được glyph (borrow lỗi / engine lỗi) ⇒ **fail-safe**
    // trả 0 ⇒ `plan_sentence_auto_fix` trả `None` ⇒ **không sửa**. Tuyệt đối
    // KHÔNG fallback `chars().count()`: số char ≠ số glyph ở VNI/TCVN3 ⇒ xoá
    // lệch vị trí (mất dữ liệu).
    let glyph_len = |text: &str| {
        shared
            .thread
            .try_borrow()
            .map(|t| t.engine.glyph_len_of(text))
            .unwrap_or(0)
    };
    let issues = check.all_issue_views();
    if issues.is_empty() {
        return false;
    }
    // Ngữ nghĩa `Fix` theo **kind** (L3 `Fix::Delete` ⇒ xoá span, không chèn
    // `suggest` — chèn ⇒ nhân đôi dấu câu "chào bạn,,") đã nằm trong
    // `plan_sentence_auto_fix` (khớp mô hình Rust + `linux-common`).
    let Some(plan) = plan_sentence_auto_fix(&check.sentence, &issues, glyph_len) else {
        return false;
    };
    let session = ApplySentenceFixSession::new(
        shared.clone(),
        ctx.clone(),
        check.clone(),
        plan,
        false,
        true,
    );
    let applied = session.apply(ec);
    if applied {
        // Vòng 28: Mode 2 tự sửa xong — re-check câu đã sửa. Còn lỗi chỉ-gợi ý
        // → panel hiện phần còn lại (trừ khi người dùng chọn
        // `hide_on_auto_fix`); sạch → không panel (như cũ).
        recheck_after_fix(shared, ctx, ec, &session.fixed_sentence, true);
    }
    applied
}

/// Session áp dụng tự sửa cả câu (AS-050 · Mode 2 / A3).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct ApplySentenceFixSession {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
    check: AssistCheck,
    plan: SentenceFixPlan,
    /// Câu đầy đủ sau fix (head + replacement) — re-check sau session.
    fixed_sentence: String,
    from_recheck: bool,
    was_auto_fix: bool,
}

#[cfg(windows)]
impl ApplySentenceFixSession {
    pub fn new(
        shared: Rc<TsfShared>,
        ctx: ITfContext,
        check: AssistCheck,
        plan: SentenceFixPlan,
        from_recheck: bool,
        was_auto_fix: bool,
    ) -> Self {
        // Câu sau fix = head (trước delete_from) + đuôi đã sửa.
        let head: String = check.sentence.chars().take(plan.delete_from).collect();
        let fixed = head + &plan.replacement.iter().collect::<String>();
        Self {
            _guard: ObjGuard::new(),
            shared,
            ctx,
            check,
            plan,
            fixed_sentence: fixed,
            from_recheck,
            was_auto_fix,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for ApplySentenceFixSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.apply_outcome(ec)))
                .unwrap_or(FixOutcome::Skipped);
        if outcome == FixOutcome::Applied {
            recheck_after_fix(
                &self.shared,
                &self.ctx,
                ec,
                &self.fixed_sentence,
                self.was_auto_fix,
            );
        }
        Ok(())
    }
}

/// Kết quả một lần sửa câu (A3) — R2-132: app CUAS sửa qua đường hẹn
/// (`cuas.rs`), kết quả thật quyết định lúc giao.
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixOutcome {
    /// Document đã nhận bản sửa trong session này.
    Applied,
    /// Đã hẹn sửa qua đường CUAS (xác minh document feed lúc giao).
    Scheduled,
    /// Không sửa (precondition sai) — document không bị đụng.
    Skipped,
}

#[cfg(windows)]
impl ApplySentenceFixSession {
    pub fn apply(&self, ec: u32) -> bool {
        self.apply_outcome(ec) == FixOutcome::Applied
    }

    pub(crate) fn apply_outcome(&self, ec: u32) -> FixOutcome {
        unsafe {
            // Trace chẩn đoán: chỉ MÃ lý do + số đếm, không chữ (S2).
            let skip = |why: std::fmt::Arguments| {
                crate::trace::event(self.shared.tid, format_args!("assist A3 skip: {why}"));
                FixOutcome::Skipped
            };
            let Some(sel) = selection_range(&self.ctx, ec) else {
                return skip(format_args!("no selection"));
            };
            // Precondition 1: selection là caret rỗng (không bôi đen)
            let selection_empty = matches!(sel.IsEmpty(ec), Ok(empty) if empty.as_bool());
            if !selection_empty {
                return skip(format_args!("selection not empty"));
            }

            // Precondition 2: buffer câu của core vẫn là câu đang kiểm tra (D8/D12)
            // (hoặc popup đến từ `recheck_after_fix` và Precondition D12
            // `verify_delete_range` bên dưới xác minh nội dung thật trong document).
            let buffer_is_ours = self.from_recheck
                || self
                    .shared
                    .thread
                    .try_borrow()
                    .map(|t| t.engine.sentence_tail(SENTENCE_MAX_CHARS) == self.check.sentence)
                    .unwrap_or(false);
            if !buffer_is_ours {
                return skip(format_args!("buffer changed"));
            }

            // Precondition 3: text store cho lùi đúng delete_glyphs glyph
            let Some(want) = backspace_shift(self.plan.delete_glyphs) else {
                return skip(format_args!("glyphs"));
            };
            let Ok(del) = sel.Clone() else {
                return FixOutcome::Skipped;
            };
            let mut shifted = 0i32;
            let shift_hr = del.ShiftStart(ec, want, &mut shifted, std::ptr::null());
            if shift_hr.is_err() || shifted != want {
                // R2-132: app CUAS (Notepad++…) không cho lùi qua chữ đã commit
                // ⇒ hẹn sửa bằng Backspace + WM_CHAR, xác minh lúc giao.
                let replacement: String = self.plan.replacement.iter().collect();
                if crate::cuas::schedule_tail_fix(
                    self.shared.tid,
                    &self.ctx,
                    &self.plan.original_tail,
                    self.plan.delete_glyphs,
                    &replacement,
                ) {
                    // Nạp sẵn câu đã sửa (lạc quan): việc giao sau tự xác minh
                    // document feed; phím tắt bấm tiếp vẫn dự đoán theo ngữ cảnh.
                    self.shared.reset_engine_with_tail(&self.fixed_sentence);
                    return FixOutcome::Scheduled;
                }
                return skip(format_args!(
                    "shift {shifted}/{want} hr={:?}",
                    shift_hr.err().map(|e| e.code())
                ));
            }

            // Chụp vùng sắp xoá để snapshot/hoàn tác
            let mut buf = vec![0u16; self.plan.delete_glyphs as usize * 2 + 2];
            let mut fetched = 0u32;
            let snapshot = del
                .GetText(ec, 0, &mut buf, &mut fetched)
                .ok()
                .map(|_| buf[..(fetched as usize).min(buf.len())].to_vec())
                .filter(|text| !text.is_empty());

            // Precondition D12 (an toàn văn bản): Vùng sắp xoá trong document phải khớp chính xác
            // với đoạn câu đuôi gốc mà engine đã phân tích (chống trường hợp dời chuột sang vị trí khác).
            // R-2: đọc được snapshot + số glyph khớp số code unit mới được so; lệch
            // ⇒ **không xác minh được ⇒ không sửa** (fail-safe, D12).
            let Some(snapshot) = snapshot else {
                crate::trace::event(self.shared.tid, format_args!("assist A3 skip: no snapshot"));
                return FixOutcome::Skipped;
            };
            if !verify_delete_range(&snapshot, &self.plan.original_tail, self.plan.delete_glyphs) {
                crate::trace::event(
                    self.shared.tid,
                    format_args!("assist A3 skip: tail mismatch"),
                );
                return FixOutcome::Skipped;
            }

            // Backspace: xoá đúng vùng cần thay
            let empty = [0u16; 1];
            if let Err(e) = del.SetText(ec, 0, &empty[..0]) {
                return skip(format_args!("delete hr={:?}", e.code()));
            }

            // Type: gõ lại replacement qua composition
            let inserted = (|| -> Result<()> {
                let ctxcomp: ITfContextComposition = self.ctx.cast()?;
                let sink: ITfCompositionSink = CompSink::new(Rc::downgrade(&self.shared)).into();
                let start = sel.Clone()?;
                let comp = ctxcomp.StartComposition(ec, &start, &sink)?;
                let range = comp.GetRange()?;
                let wide = to_wide_nul(&self.plan.replacement);
                range.SetText(ec, 0, &wide[..wide.len() - 1])?;
                set_caret_after(&self.ctx, ec, &range);
                let _ = comp.EndComposition(ec);
                Ok(())
            })();

            if inserted.is_err() {
                crate::trace::event(self.shared.tid, format_args!("assist A3 restore"));
                let _ = del.SetText(ec, 0, &snapshot);
                set_caret_after(&self.ctx, ec, &del);
                return FixOutcome::Skipped;
            }

            // Lưu trạng thái hoàn tác (Ctrl+Z trong 5 giây)
            let repl_str: String = self.plan.replacement.iter().collect();
            // R-11: không đếm được glyph ⇒ **không lưu** state hoàn tác (số glyph
            // sai ⇒ hoàn tác xoá lệch vị trí). Fail-safe: bỏ undo còn hơn xoá nhầm.
            let repl_glyphs = self
                .shared
                .thread
                .try_borrow()
                .map(|t| t.engine.glyph_len_of(&repl_str) as u32)
                .unwrap_or(0);

            if repl_glyphs > 0 {
                if let Ok(mut undo_slot) = self.shared.assist_undo.try_borrow_mut() {
                    *undo_slot = Some(AssistUndoState::new(
                        self.plan.original_tail.clone(),
                        repl_str.clone(),
                        repl_glyphs,
                        self.plan.fixed_count,
                    ));
                }
            }

            // Buffer engine = câu đã sửa (đúng document trước caret) — R2-131.
            self.shared.reset_engine_with_tail(&self.fixed_sentence);

            // Toast khay DUY NHẤT của assist (vòng 26): Mode 2 vừa TỰ sửa —
            // cooldown 60 s, chỉ mang số lỗi (S2 — không log nội dung câu).
            // Người dùng tự bấm sửa (nút/phím tắt) thì không toast (R2-131).
            if self.was_auto_fix {
                super::assist::notify_auto_fixed(self.plan.fixed_count);
            }
            crate::trace::event(
                self.shared.tid,
                format_args!("assist A3 applied: {} issues", self.plan.fixed_count),
            );

            FixOutcome::Applied
        }
    }
}

// ---------------------------------------------------------------------------
// AS-050 — Hoàn tác tự sửa cả câu qua Ctrl+Z
// ---------------------------------------------------------------------------

/// Áp dụng hoàn tác tự sửa cả câu nếu còn trong 5 giây. Trả `true` nếu đã xử lý.
///
/// R-3 (undo destructive):
/// - **Không `take()` state khi chưa chắc chắn**: nếu TSF không cấp edit session
///   thì state phải còn nguyên (và phím Ctrl+Z phải tới app) — nuốt phím mà
///   không hoàn tác được là mất chức năng Undo của app.
/// - Chỉ xoá state khi hoàn tác **đã áp dụng** thành công.
#[cfg(windows)]
pub fn apply_assist_undo(shared: &Rc<TsfShared>, ctx: &ITfContext) -> bool {
    // Peek (không take) — state chỉ bị xoá khi thực sự áp dụng được.
    let state = shared
        .assist_undo
        .try_borrow()
        .ok()
        .and_then(|slot| slot.clone());
    let Some(state) = state else {
        return false;
    };
    if !state.is_valid() {
        // Hết hạn 5 s ⇒ xoá state, để Ctrl+Z tới app (không nuốt phím).
        shared.clear_assist_undo();
        return false;
    }
    let result = Rc::new(Cell::new(None));
    let session: ITfEditSession =
        ApplyAssistUndoSession::new(shared.clone(), ctx.clone(), state, result.clone()).into();
    // Ưu tiên đồng bộ: biết ngay kết quả áp dụng (không áp được ⇒ trả false ⇒
    // phím tới app, Ctrl+Z của app vẫn chạy).
    // SAFETY: ctx/session hợp lệ trong callback của TSF.
    let hr = unsafe { ctx.RequestEditSession(shared.tid, &session, TF_ES_SYNC | TF_ES_READWRITE) };
    match (hr, result.get()) {
        (Ok(code), Some(applied)) if code.is_ok() => {
            if applied {
                shared.clear_assist_undo();
            }
            return applied;
        }
        _ => {}
    }
    // Không chạy được đồng bộ ⇒ thử bất đồng bộ. Nếu cũng không được ⇒ **không
    // nuốt phím** (trả false) và **giữ state** cho lần sau.
    let granted = matches!(
        unsafe { ctx.RequestEditSession(shared.tid, &session, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE) },
        Ok(code) if code.is_ok()
    );
    if granted {
        // Audit vòng 1 (W3): KHÔNG xoá state ngay khi session được cấp bất đồng bộ
        // — kết quả áp dụng chỉ biết khi `DoEditSession` chạy; session tự xoá
        // state khi hoàn tác thành công. Xoá sớm ⇒ session fail thì vừa nuốt
        // phím Ctrl+Z vừa mất khả năng hoàn tác. State còn sót sẽ tự hết hạn
        // sau 5 s (`AssistUndoState::is_valid`).
        true
    } else {
        false
    }
}

/// Session hoàn tác tự sửa cả câu bằng Backspace+Type (AS-050).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct ApplyAssistUndoSession {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
    undo: AssistUndoState,
    /// Kết quả áp dụng (`Some(true)` = đã hoàn tác) — để caller biết có nên
    /// nuốt phím hay không (R-3).
    result: Rc<Cell<Option<bool>>>,
}

#[cfg(windows)]
impl ApplyAssistUndoSession {
    pub fn new(
        shared: Rc<TsfShared>,
        ctx: ITfContext,
        undo: AssistUndoState,
        result: Rc<Cell<Option<bool>>>,
    ) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
            ctx,
            undo,
            result,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for ApplyAssistUndoSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.apply(ec)))
            .unwrap_or(false);
        self.shared.reset_engine();
        // W3: session tự xoá state khi hoàn tác THÀNH CÔNG — đúng cho cả đường
        // đồng bộ lẫn bất đồng bộ (nhánh bất đồng bộ của caller không chờ biết
        // kết quả ⇒ chỉ session nơi này biết và được phép xoá; dự phòng cho
        // đường đồng bộ, `clear_assist_undo` idempotent).
        if applied {
            self.shared.clear_assist_undo();
        }
        self.result.set(Some(applied));
        if applied {
            crate::trace::event(self.shared.tid, format_args!("assist A3 undo applied"));
        }
        Ok(())
    }
}

#[cfg(windows)]
impl ApplyAssistUndoSession {
    fn apply(&self, ec: u32) -> bool {
        unsafe {
            let Some(sel) = selection_range(&self.ctx, ec) else {
                return false;
            };
            if !matches!(sel.IsEmpty(ec), Ok(empty) if empty.as_bool()) {
                return false;
            }
            let Some(want) = backspace_shift(self.undo.replacement_glyphs) else {
                return false;
            };
            let Ok(del) = sel.Clone() else {
                return false;
            };
            let mut shifted = 0i32;
            if del
                .ShiftStart(ec, want, &mut shifted, std::ptr::null())
                .is_err()
                || shifted != want
            {
                return false;
            }

            // R-3 (fail-safe): vùng sắp xoá phải **đúng** là text ta vừa gõ lại
            // sau khi sửa. Nếu người dùng đã gõ thêm (hoặc app tự sửa) ⇒ vùng
            // này không còn là của ta ⇒ **không xoá** (tránh nuốt chữ người dùng).
            let mut buf = vec![0u16; self.undo.replacement_glyphs as usize * 2 + 2];
            let mut fetched = 0u32;
            let Some(snapshot) = del
                .GetText(ec, 0, &mut buf, &mut fetched)
                .ok()
                .map(|_| buf[..(fetched as usize).min(buf.len())].to_vec())
                .filter(|text| !text.is_empty())
            else {
                return false; // không đọc được ⇒ không xác minh được ⇒ không xoá
            };
            if !verify_delete_range(
                &snapshot,
                &self.undo.replacement,
                self.undo.replacement_glyphs,
            ) {
                crate::trace::event(
                    self.shared.tid,
                    format_args!("assist A3 undo skip: mismatch"),
                );
                return false;
            }

            let empty = [0u16; 1];
            if del.SetText(ec, 0, &empty[..0]).is_err() {
                return false;
            }
            let inserted = (|| -> Result<()> {
                let ctxcomp: ITfContextComposition = self.ctx.cast()?;
                let sink: ITfCompositionSink = CompSink::new(Rc::downgrade(&self.shared)).into();
                let start = sel.Clone()?;
                let comp = ctxcomp.StartComposition(ec, &start, &sink)?;
                let range = comp.GetRange()?;
                let chars: Vec<char> = self.undo.original_tail.chars().collect();
                let wide = to_wide_nul(&chars);
                range.SetText(ec, 0, &wide[..wide.len() - 1])?;
                set_caret_after(&self.ctx, ec, &range);
                let _ = comp.EndComposition(ec);
                Ok(())
            })();
            if inserted.is_err() {
                // Gõ lại thất bại ⇒ hoàn tác phần đã xoá (không mất chữ).
                crate::trace::event(self.shared.tid, format_args!("assist A3 undo restore"));
                let _ = del.SetText(ec, 0, &snapshot);
                set_caret_after(&self.ctx, ec, &del);
                return false;
            }
            inserted.is_ok()
        }
    }
}

/// Session cho `Ctrl+Shift+G`: kiểm tra lại câu rồi hiện overlay (AS-030).
#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct AssistRecallSession {
    _guard: ObjGuard,
    shared: Rc<TsfShared>,
    ctx: ITfContext,
}

#[cfg(windows)]
impl AssistRecallSession {
    pub fn new(shared: Rc<TsfShared>, ctx: ITfContext) -> Self {
        Self {
            _guard: ObjGuard::new(),
            shared,
            ctx,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for AssistRecallSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        let shared = &self.shared;
        let showed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Some(check) = crate::assist::check_sentence(shared) else {
                return false;
            };
            let shown = show_assist_overlay(shared, &self.ctx, ec, &check);
            if shown {
                // R2-131: Ctrl+Space giờ sửa hết ngay — Ctrl+Shift+G là
                // đường chọn từng lỗi bằng phím (↑/↓/Tab · Enter, R2-122).
                mark_assist_on_demand(shared);
            }
            shown
        }))
        .unwrap_or(false);
        if !showed {
            hide_assist_overlay(shared);
        }
        Ok(())
    }
}

/// Commit composition đang mở (nguyên văn) và dọn model + engine. KHÔNG dời
/// caret: khi self-heal, người dùng vừa click sang chỗ khác và từ mới phải bắt
/// đầu đúng tại đó; khi gõ bình thường caret vốn đã ở cuối composition.
#[cfg(windows)]
fn finish(ec: u32, comp: &mut Option<ITfComposition>, shared: &TsfShared) {
    if let Some(c) = comp.take() {
        // SAFETY: đang ở trong DoEditSession với ec hợp lệ.
        let _ = unsafe { c.EndComposition(ec) };
    }
    if let Ok(mut text) = shared.comp_text.try_borrow_mut() {
        text.clear();
    }
    shared.reset_engine();
}

/// Áp kế hoạch vào document. Lỗi bất kỳ → `Err` để caller fail-open.
#[cfg(windows)]
fn apply_plan(
    shared: &Rc<TsfShared>,
    ctx: &ITfContext,
    ec: u32,
    sel: &ITfRange,
    comp: &mut Option<ITfComposition>,
    plan: &CompositionPlan,
) -> Result<()> {
    // SAFETY: mọi lời gọi TSF dưới đây nằm trong DoEditSession với `ec` hợp lệ;
    // buffer UTF-16 luôn có 0 đệm sau (finding S3-2: TSF đọc bằng wcslen()).
    unsafe {
        // Vòng 16 (audit A9): text vùng xoá được chụp TRƯỚC khi xoá — nếu chèn
        // thất bại thì hoàn tác (SetText ngược lại), không mất chữ một nhịp gõ.
        let mut restore: Option<(ITfRange, Vec<u16>)> = None;
        if plan.delete_before > 0 {
            let anchor = match comp.as_ref() {
                Some(c) => c.GetRange()?,
                None => sel.Clone()?,
            };
            let del = anchor.Clone()?;
            del.Collapse(ec, TF_ANCHOR_START)?;
            let want = -i32::from(plan.delete_before);
            let mut shifted = 0i32;
            if let Err(e) = del.ShiftStart(ec, want, &mut shifted, std::ptr::null()) {
                crate::trace::event(
                    shared.tid,
                    format_args!("delete_before shift err {:#x}", e.code().0),
                );
                return Err(e);
            }
            // Text store không cho lùi đủ (app IMM32): không xóa gì cả.
            if shifted != want {
                crate::trace::event(
                    shared.tid,
                    format_args!("delete_before shifted {shifted} want {want}"),
                );
                return Err(Error::from(E_FAIL));
            }
            if let Some(c) = comp.as_ref() {
                // R2-128: TIP đang giữ composition chỉ được sửa text TRONG
                // composition của mình — SetText vùng trước nó bị từ chối
                // `TF_E_RANGE_NOT_COVERED` (0x80040507, bắt bằng trace trên
                // Notepad Win11/Word: `=15000000bc`+Space không bung). Cách
                // chuẩn TSF: NỚI composition ra trước phủ phần cần xoá; SetText
                // composition bên dưới thay cả khối (không xoá rời → không cần
                // hoàn tác A9: lỗi thì text cũ vẫn nguyên trong composition).
                if let Err(e) = c.ShiftStart(ec, &del) {
                    crate::trace::event(
                        shared.tid,
                        format_args!("delete_before extend composition err {:#x}", e.code().0),
                    );
                    return Err(e);
                }
            } else {
                let snap = del.Clone()?;
                let mut buf = vec![0u16; plan.delete_before as usize];
                let mut fetched = 0u32;
                // Vòng 40 (audit Windows T-1): KHÔNG xoá khi không chụp được
                // snapshot hoàn tác — GetText lỗi/thiếu mà xoá rồi thì lỗi
                // insert sau đó không có gì trả lại (mất chữ đã commit).
                // Cùng triết lý "không xác minh được ⇒ không sửa" như
                // verify_delete_range.
                match snap.GetText(ec, 0, &mut buf, &mut fetched) {
                    Ok(()) if fetched as usize == plan.delete_before as usize => {
                        restore = Some((del.Clone()?, buf[..fetched as usize].to_vec()));
                    }
                    Ok(()) => {
                        crate::trace::event(
                            shared.tid,
                            format_args!(
                                "delete_before gettext short {fetched}/{} - bo khong xoa",
                                plan.delete_before
                            ),
                        );
                        return Err(Error::from(E_FAIL));
                    }
                    Err(e) => {
                        crate::trace::event(
                            shared.tid,
                            format_args!(
                                "delete_before gettext err {:#x} - bo khong xoa",
                                e.code().0
                            ),
                        );
                        return Err(e);
                    }
                }
                let empty = [0u16; 1];
                if let Err(e) = del.SetText(ec, 0, &empty[..0]) {
                    crate::trace::event(
                        shared.tid,
                        format_args!("delete_before settext err {:#x}", e.code().0),
                    );
                    return Err(e);
                }
            }
        }

        // Chèn + kết thúc composition đặt trong closure để hoàn tác A9 khi lỗi. Mọi
        // lỗi có thể trả về trong closure đều xảy ra TRƯỚC khi text mới vào document
        // (R2-103) — hoàn tác chỉ trả lại đúng phần vừa xoá, không bao giờ nhân đôi.
        let insert_result: Result<()> = (|| {
            if let Some(text) = &plan.text {
                let wide = to_wide_nul(text);
                let body = &wide[..wide.len() - 1];
                if comp.is_none() && !text.is_empty() {
                    let ctxcomp: ITfContextComposition = ctx.cast()?;
                    let sink: ITfCompositionSink = CompSink::new(Rc::downgrade(shared)).into();
                    let start = sel.Clone()?;
                    *comp = Some(ctxcomp.StartComposition(ec, &start, &sink)?);
                }
                if let Some(c) = comp.as_ref() {
                    let range = c.GetRange()?;
                    range.SetText(ec, 0, body)?;
                    apply_display_attribute(ctx, ec, &range);
                    set_caret_after(ctx, ec, &range);
                }
            }

            // R2-103: tới đây text đã nằm trong document. Lỗi EndComposition KHÔNG
            // được kích hoạt hoàn tác bên dưới — hoàn tác lúc này chèn lại phần đã
            // xoá cạnh từ mới ("vie" + "việ" → lặp chữ), hoặc trả lại chữ mà Backspace
            // vừa xoá có chủ đích. Bỏ qua lỗi như `finish()` (composition đã bị app
            // kết thúc là nguyên nhân thường gặp).
            if plan.end {
                if let Some(c) = comp.take() {
                    if let Ok(range) = c.GetRange() {
                        set_caret_after(ctx, ec, &range);
                    }
                    let _ = c.EndComposition(ec);
                }
            }
            Ok(())
        })();

        if let Err(err) = insert_result {
            if let Some((range, text)) = restore.take() {
                crate::trace::event(
                    shared.tid,
                    format_args!("insert fail - hoan tac text da xoa"),
                );
                let _ = range.SetText(ec, 0, &text);
            }
            return Err(err);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn to_wide_nul(text: &[char]) -> Vec<u16> {
    let mut wide = Vec::with_capacity(text.len() + 1);
    let mut buf = [0u16; 2];
    for ch in text {
        wide.extend_from_slice(ch.encode_utf16(&mut buf));
    }
    wide.push(0);
    wide
}

/// Selection mặc định của context (caret/vùng chọn). S3-1: KHÔNG dùng `GetStart`.
#[cfg(windows)]
pub(crate) fn selection_range(ctx: &ITfContext, ec: u32) -> Option<ITfRange> {
    let mut sel = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    // SAFETY: buffer 1 phần tử + ec hợp lệ trong edit session.
    let ok = unsafe { ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched) }.is_ok();
    // TF_SELECTION.range là ManuallyDrop: luôn lấy ra để Release đúng một lần.
    let range = std::mem::ManuallyDrop::into_inner(std::mem::replace(
        &mut sel[0].range,
        std::mem::ManuallyDrop::new(None),
    ));
    if ok && fetched > 0 {
        range
    } else {
        None
    }
}

/// Lấy tọa độ màn hình của caret hiện tại từ context view (AS-030).
#[cfg(windows)]
pub fn get_caret_rect(ctx: &ITfContext, ec: u32) -> Option<RECT> {
    unsafe {
        let view = ctx.GetActiveView().ok()?;
        let sel = selection_range(ctx, ec)?;
        let mut rc = RECT::default();
        let mut clipped = windows::core::BOOL::default();
        if view.GetTextExt(ec, &sel, &mut rc, &mut clipped).is_ok() {
            Some(rc)
        } else {
            None
        }
    }
}

/// Neo panel cạnh caret — chỉ **vị trí**, không đọc chữ của app (D8). Thứ tự
/// (R2-125 — trước đây Notepad Win11 rơi hết các nhánh nên panel không bao giờ
/// hiện, chỉ còn toast):
/// 1. `ITfContextView::GetTextExt` của caret (rect bề rộng 0 vẫn nhận);
/// 2. `GetTextExt` của **ký tự ngay trước caret** (app không đo range rỗng) —
///    neo cạnh phải ký tự đó;
/// 3. `GetGUIThreadInfo(thread hiện tại).rcCaret` (app IMM32/legacy) đổi
///    **client → màn hình**;
/// 4. góc dưới-trái bên trong control đang focus (gần đúng, vẫn hơn không hiện).
#[cfg(windows)]
pub(crate) fn caret_anchor(ctx: &ITfContext, ec: u32, _client_id: u32) -> Option<Anchor> {
    if let Some(anchor) = tsf_caret_anchor(ctx, ec) {
        return Some(anchor);
    }
    // SAFETY: GetGUIThreadInfo/ClientToScreen/GetWindowRect chỉ đọc trạng thái GUI
    // của thread UI hiện tại (edit session chạy trên thread đó).
    unsafe {
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        // GetGUIThreadInfo nhận THREAD ID Win32 — `TsfShared::tid` là TfClientId
        // của TSF (đối số `RequestEditSession`), truyền vào đây luôn thất bại.
        let thread = windows::Win32::System::Threading::GetCurrentThreadId();
        if GetGUIThreadInfo(thread, &mut info).is_err() {
            return None;
        }
        if !info.hwndCaret.0.is_null() {
            // rcCaret là toạ độ CLIENT của hwndCaret.
            let mut tl = windows::Win32::Foundation::POINT {
                x: info.rcCaret.left,
                y: info.rcCaret.top,
            };
            let mut br = windows::Win32::Foundation::POINT {
                x: info.rcCaret.right,
                y: info.rcCaret.bottom,
            };
            if windows::Win32::Graphics::Gdi::ClientToScreen(info.hwndCaret, &mut tl).as_bool()
                && windows::Win32::Graphics::Gdi::ClientToScreen(info.hwndCaret, &mut br).as_bool()
            {
                if let Some(anchor) = Anchor::from_caret_rect(tl.x, tl.y, br.x, br.y) {
                    return Some(anchor);
                }
            }
        }
        let focus = if !info.hwndFocus.0.is_null() {
            info.hwndFocus
        } else {
            info.hwndActive
        };
        if focus.0.is_null() {
            return None;
        }
        let mut rc = RECT::default();
        if windows::Win32::UI::WindowsAndMessaging::GetWindowRect(focus, &mut rc).is_err() {
            return None;
        }
        Anchor::inside_bottom_left(rc.left, rc.top, rc.right, rc.bottom)
    }
}

/// Bước 1–2 của [`caret_anchor`]: đo caret qua TSF (toạ độ màn hình).
#[cfg(windows)]
fn tsf_caret_anchor(ctx: &ITfContext, ec: u32) -> Option<Anchor> {
    // SAFETY: gọi trong edit session (ec hợp lệ); chỉ đọc toạ độ.
    unsafe {
        let view = ctx.GetActiveView().ok()?;
        let sel = selection_range(ctx, ec)?;
        let mut rc = RECT::default();
        let mut clipped = windows::core::BOOL::default();
        if view.GetTextExt(ec, &sel, &mut rc, &mut clipped).is_ok() {
            if let Some(anchor) = Anchor::from_caret_rect(rc.left, rc.top, rc.right, rc.bottom) {
                return Some(anchor);
            }
        }
        let prev = sel.Clone().ok()?;
        let mut shifted = 0i32;
        if prev
            .ShiftStart(ec, -1, &mut shifted, std::ptr::null())
            .is_ok()
            && shifted == -1
            && view.GetTextExt(ec, &prev, &mut rc, &mut clipped).is_ok()
        {
            return Anchor::from_caret_rect(rc.right, rc.top, rc.right, rc.bottom);
        }
        None
    }
}

/// Đặt caret (selection rỗng) ngay sau `range`.
#[cfg(windows)]
pub(crate) fn set_caret_after(ctx: &ITfContext, ec: u32, range: &ITfRange) {
    // SAFETY: gọi trong edit session; ManuallyDrop được giải phóng thủ công bên dưới.
    unsafe {
        let Ok(caret) = range.Clone() else { return };
        if caret.Collapse(ec, TF_ANCHOR_END).is_err() {
            return;
        }
        let mut sel = [TF_SELECTION {
            range: std::mem::ManuallyDrop::new(Some(caret)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_NONE,
                fInterimChar: false.into(),
            },
        }];
        let _ = ctx.SetSelection(ec, &sel);
        drop(std::mem::ManuallyDrop::into_inner(std::mem::replace(
            &mut sel[0].range,
            std::mem::ManuallyDrop::new(None),
        )));
    }
}

/// Chỉ tiếp tục sửa composition khi selection là caret rỗng đúng tại cuối range
/// của TIP. Click vào giữa từ hoặc chọn một phần composition phải commit trước,
/// nếu không `apply_plan` sẽ ghi đè phần người dùng đang sửa/chọn. Với text store
/// tối giản không trả được vị trí, giữ chính sách cũ: không tự chữa chỉ vì lỗi đọc.
#[cfg(windows)]
fn caret_at_composition_end(ec: u32, comp: &ITfComposition, sel: &ITfRange) -> bool {
    // SAFETY: gọi trong edit session.
    unsafe {
        let Ok(range) = comp.GetRange() else {
            return true;
        };
        // Cả start và end của selection phải trùng comp.end. Điều này đồng thời
        // yêu cầu selection rỗng; bất kỳ kết quả khác 0 nào đều là caret/selection
        // do app hoặc người dùng di chuyển.
        let start_at_end = range
            .CompareEnd(ec, sel, TF_ANCHOR_START)
            .map(|c| c == 0)
            .unwrap_or(true);
        let end_at_end = range
            .CompareEnd(ec, sel, TF_ANCHOR_END)
            .map(|c| c == 0)
            .unwrap_or(true);
        start_at_end && end_at_end
    }
}

/// Tín hiệu field in-proc: read-only, InputScope, style `ES_PASSWORD` của HWND focus.
#[cfg(windows)]
pub(crate) fn read_field_signals(ctx: &ITfContext, ec: u32, sel: &ITfRange) -> TsfFieldSignals {
    let mut signals = TsfFieldSignals::default();
    // SAFETY: các lời gọi đọc trong edit session; VARIANT được VariantClear đúng một lần.
    unsafe {
        if let Ok(status) = ctx.GetStatus() {
            signals.read_only = status.dwDynamicFlags & TF_SD_READONLY != 0;
        }
        if let Ok(prop) = ctx.GetProperty(&GUID_PROP_INPUTSCOPE) {
            if let Ok(mut var) = prop.GetValue(ec, sel) {
                if var.Anonymous.Anonymous.vt == VT_UNKNOWN {
                    let unk: Option<&IUnknown> = var.Anonymous.Anonymous.Anonymous.punkVal.as_ref();
                    if let Some(scope) = unk.and_then(|u| u.cast::<ITfInputScope>().ok()) {
                        signals.input_scopes = input_scopes(&scope);
                    }
                }
                let _ = VariantClear(&mut var);
            }
        }
    }
    signals.password_style = focus_is_password_edit();
    signals
}

#[cfg(windows)]
unsafe fn input_scopes(scope: &ITfInputScope) -> Vec<i32> {
    let mut ptr: *mut InputScope = std::ptr::null_mut();
    let mut count = 0u32;
    if scope.GetInputScopes(&mut ptr, &mut count).is_err() || ptr.is_null() {
        return Vec::new();
    }
    // SAFETY: TSF cấp mảng `count` phần tử bằng CoTaskMemAlloc; copy rồi giải phóng.
    let out = std::slice::from_raw_parts(ptr, count as usize)
        .iter()
        .map(|s| s.0)
        .collect();
    windows::Win32::System::Com::CoTaskMemFree(Some(ptr as *const _));
    out
}

/// Edit/RichEdit (Win32, WinForms, Delphi…) có `ES_PASSWORD`: UIA `IsPassword`
/// không đáng tin cho các control này (`docs/specs/tsf-spike.md` #9), còn style thì
/// đọc được ngay trong process, cùng thread UI.
#[cfg(windows)]
fn focus_is_password_edit() -> bool {
    // SAFETY: GetFocus/GetWindowLongW/GetClassNameW chỉ đọc HWND của thread hiện tại.
    unsafe {
        let hwnd = GetFocus();
        if hwnd.is_invalid() {
            return false;
        }
        if GetWindowLongW(hwnd, GWL_STYLE) & ES_PASSWORD == 0 {
            return false;
        }
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, &mut class).max(0) as usize;
        String::from_utf16_lossy(&class[..len.min(class.len())])
            .to_ascii_lowercase()
            .contains("edit")
    }
}

#[cfg(windows)]
fn apply_display_attribute(ctx: &ITfContext, ec: u32, range: &ITfRange) {
    use crate::guids::DISPATTR_TEXTVN;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::TextServices::{
        CLSID_TF_CategoryMgr, ITfCategoryMgr, GUID_PROP_ATTRIBUTE,
    };

    thread_local! {
        /// Atom của `DISPATTR_TEXTVN` — đăng ký MỘT lần mỗi thread UI (như SampleIME
        /// lưu `_gaDisplayAttributeInput` lúc Activate) thay vì `CoCreateInstance` +
        /// `RegisterGUID` ở mỗi phím. 0 = chưa đăng ký được (thử lại lần sau).
        static ATOM: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }
    let mut atom = ATOM.with(|a| a.get());
    // SAFETY: COM đã khởi tạo; ctx và range hợp lệ trong edit session.
    unsafe {
        if atom == 0 {
            if let Ok(cat) = CoCreateInstance::<_, ITfCategoryMgr>(
                &CLSID_TF_CategoryMgr,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                atom = cat.RegisterGUID(&DISPATTR_TEXTVN).unwrap_or(0);
                ATOM.with(|a| a.set(atom));
            }
        }
        if atom != 0 {
            if let Ok(prop) = ctx.GetProperty(&GUID_PROP_ATTRIBUTE) {
                let var = VARIANT::from(atom as i32);
                let _ = prop.SetValue(ec, range, &var);
            }
        }
    }
}

/// Vòng 27 — dispatch hành động chuột từ panel assist (gọi trên thread UI của
/// app, nơi panel sống): Select → vẽ lại; Apply → A2 sửa một từ; FixAll →
/// A3 tự sửa cả câu (như Mode 2 nhưng **user chủ động** — precondition và
/// Undo 5 giây giống hệt).
#[cfg(windows)]
pub(crate) fn handle_overlay_mouse_action(
    shared: &Rc<TsfShared>,
    action: crate::overlay::OverlayMouseAction,
) {
    crate::trace::event(shared.tid, format_args!("assist mouse {action:?}"));
    match action {
        crate::overlay::OverlayMouseAction::Select(issue_index) => {
            // Chọn lỗi trong model + vẽ lại panel (không đụng document).
            if let Ok(mut slot) = shared.assist.try_borrow_mut() {
                if let Some(active) = slot.as_mut() {
                    if active.view.select_index(issue_index) {
                        let view = active.view.clone();
                        shared.overlay.set_view(&view);
                    }
                }
            }
        }
        crate::overlay::OverlayMouseAction::Apply(issue_index) => {
            // A2: ctx lưu trong ActiveAssist lúc show — click chuột KHÔNG có
            // edit cookie từ TSF nên tự xin session (sync → async).
            let ctx = shared
                .assist
                .try_borrow()
                .ok()
                .and_then(|slot| slot.as_ref().map(|a| a.ctx.clone()));
            let _ = apply_assist_suggestion(shared, ctx.as_ref(), issue_index);
        }
        crate::overlay::OverlayMouseAction::FixAll => {
            apply_sentence_fix_all(shared);
        }
    }
}

/// Vòng 31 (audit v30 F6): Enter trên popup từ BÔI ĐEN = Sửa-tất-cả vùng
/// chọn — cùng đường với nút chuột (precondition session tự kiểm).
#[cfg(windows)]
pub(crate) fn selection_fix_all_from_keyboard(shared: &Rc<TsfShared>) {
    apply_sentence_fix_all(shared);
}

/// "Sửa tất cả" — nút chuột lẫn Enter trên popup (Vòng 31): tính kế hoạch A3
/// từ model đang hiện rồi xin edit session áp dụng qua `request_sync_or_async`
/// (chuột/phím không có edit cookie từ TSF). `ApplySentenceFixSession::apply`
/// tự kiểm precondition (selection rỗng, buffer còn là câu của ta) và lưu
/// Undo 5 giây; popup từ bôi đen đi nhánh `ApplySelectionFixSession`.
#[cfg(windows)]
pub(crate) fn apply_sentence_fix_all(shared: &Rc<TsfShared>) {
    // Vòng 32: tính plan TRƯỚC khi take — popup chỉ chứa lỗi chỉ-gợi ý (không
    // có lỗi objective) thì "Sửa tất cả" không làm được gì ⇒ GIỮ nguyên popup
    // (người dùng còn bấm được từng gợi ý), thay vì đóng mà không có gì xảy
    // ra — trước đây Enter trên popup selection như vậy bị nuốt phím oan.
    let has_plan = shared.assist.try_borrow().ok().and_then(|slot| {
        let active = slot.as_ref()?;
        let thread = shared.thread.try_borrow().ok()?;
        let glyph_len = |text: &str| thread.engine.glyph_len_of(text);
        let all_issues = active.check.all_issue_views();
        plan_sentence_user_fix(&active.sentence, &all_issues, glyph_len)
            .map(|_| ())
            .or_else(|| {
                active
                    .selection
                    .as_ref()
                    .and_then(|_| plan_fix(&active.view, active.view.selected, glyph_len))
                    .map(|_| ())
            })
    });
    if has_plan.is_none() {
        crate::trace::event(shared.tid, format_args!("assist fix-all: no plan"));
        return;
    }
    let Some(active) = shared
        .assist
        .try_borrow_mut()
        .ok()
        .and_then(|mut slot| slot.take())
    else {
        return;
    };
    hide_assist_overlay(shared);
    let all_issues = active.check.all_issue_views();
    // Vòng 30: popup đến từ bôi đen → sửa bằng SetText đúng vùng chọn
    // (precondition: text vùng chọn vẫn bằng câu đã check — session tự kiểm).
    if let Some(range) = active.selection.clone() {
        let fixed = {
            let Ok(thread) = shared.thread.try_borrow() else {
                return;
            };
            let glyph_len = |text: &str| thread.engine.glyph_len_of(text);
            if let Some(plan) = plan_sentence_user_fix(&active.sentence, &all_issues, glyph_len) {
                let head: String = active.sentence.chars().take(plan.delete_from).collect();
                Some(head + &plan.replacement.iter().collect::<String>())
            } else {
                plan_fix(&active.view, active.view.selected, glyph_len).map(|plan| {
                    let head_len = active
                        .sentence
                        .chars()
                        .count()
                        .saturating_sub(plan.delete_chars);
                    let head: String = active.sentence.chars().take(head_len).collect();
                    head + &plan.replacement.iter().collect::<String>()
                })
            }
        };
        let Some(fixed) = fixed else {
            return;
        };
        crate::selection_check::request_apply_selection_fix(
            shared,
            &active.ctx,
            &range,
            &active.sentence,
            &fixed,
        );
        return;
    }
    let plan = {
        let Ok(thread) = shared.thread.try_borrow() else {
            return;
        };
        let glyph_len = |text: &str| thread.engine.glyph_len_of(text);
        // Kế hoạch chỉ áp **lỗi objective** (auto_fixable) — lỗi chỉ-gợi ý giữ
        // lại cho người dùng quyết (A1), đúng ngữ nghĩa Mode 2.
        plan_sentence_user_fix(&active.sentence, &all_issues, glyph_len)
    };
    let Some(plan) = plan else {
        return;
    };
    let session: ITfEditSession = ApplySentenceFixSession::new(
        shared.clone(),
        active.ctx.clone(),
        active.check.clone(),
        plan,
        active.from_recheck,
        false,
    )
    .into();
    let requested = request_sync_or_async(&active.ctx, shared.tid, &session);
    crate::trace::event(
        shared.tid,
        format_args!("assist fix-all: session requested={requested}"),
    );
}

#[cfg(all(test, windows))]
mod tests {
    use crate::input_scope;
    use windows::Win32::UI::TextServices::*;

    #[test]
    fn input_scope_constants_match_sdk() {
        assert_eq!(input_scope::IS_URL, IS_URL.0);
        assert_eq!(input_scope::IS_PASSWORD, IS_PASSWORD.0);
        assert_eq!(input_scope::IS_SEARCH, IS_SEARCH.0);
        assert_eq!(input_scope::IS_NUMERIC_PASSWORD, IS_NUMERIC_PASSWORD.0);
        assert_eq!(input_scope::IS_NUMERIC_PIN, IS_NUMERIC_PIN.0);
        assert_eq!(input_scope::IS_ALPHANUMERIC_PIN, IS_ALPHANUMERIC_PIN.0);
        assert_eq!(
            input_scope::IS_ALPHANUMERIC_PIN_SET,
            IS_ALPHANUMERIC_PIN_SET.0
        );
    }

    /// D27: số Backspace lấy từ glyph của core; 0/tràn `i32` ⇒ không sửa.
    #[test]
    fn backspace_shift_tu_glyph_va_tu_choi_so_vo_ly() {
        assert_eq!(super::backspace_shift(1), Some(-1));
        assert_eq!(super::backspace_shift(5), Some(-5));
        assert_eq!(super::backspace_shift(0), None, "0 glyph = không sửa");
        assert_eq!(
            super::backspace_shift(u32::MAX),
            None,
            "không biểu diễn được trong i32 = không sửa"
        );
    }

    /// R-2 (fail-safe): vùng sắp xoá phải **đúng** bằng đuôi câu engine biết.
    #[test]
    fn verify_delete_range_fail_safe_khi_lenh() {
        let tail = "tôi tôi đi học .";
        let wide: Vec<u16> = tail.encode_utf16().collect();
        // Khớp ⇒ được phép sửa.
        assert!(super::verify_delete_range(&wide, tail, wide.len() as u32));
        // Lệch (caret ở chỗ khác / app tự sửa) ⇒ KHÔNG sửa.
        assert!(!super::verify_delete_range(
            &wide,
            "tôi tôi đi học",
            wide.len() as u32
        ));
        assert!(!super::verify_delete_range(
            &wide[..4],
            tail,
            wide.len() as u32
        ));
        // Số glyph ≠ số code unit ⇒ không xác minh được ⇒ không sửa (emoji/legacy).
        assert!(!super::verify_delete_range(
            &wide,
            tail,
            wide.len() as u32 + 1
        ));
        // Rỗng ⇒ không sửa.
        assert!(!super::verify_delete_range(&[], "", 0));
    }

    /// Đuôi câu engine biết = `delete_chars` ký tự cuối của câu.
    #[test]
    fn expected_tail_lay_duoi_cau() {
        assert_eq!(super::expected_tail("tôi tôi đi học .", 9), " đi học .");
        assert_eq!(super::expected_tail("tôi tôi đi học .", 0), "");
        // `delete_chars` vượt độ dài câu ⇒ lấy hết (không panic).
        assert_eq!(super::expected_tail("abc", 99), "abc");
    }

    /// R-3: state hoàn tác phải bị xoá khi người dùng gõ phím khác.
    #[test]
    fn clear_assist_undo_xoa_state() {
        let client = crate::ipc_client::IpcClient::global("test_undo_clear.exe");
        let thread = crate::ThreadState::new("test_undo_clear.exe", 0).unwrap();
        let shared = super::TsfShared::new(2, client, thread);
        assert!(shared.assist_undo.try_borrow().unwrap().is_none());
        *shared.assist_undo.try_borrow_mut().unwrap() = Some(crate::assist::AssistUndoState::new(
            "tôi tôi".to_string(),
            "tôi".to_string(),
            3,
            1,
        ));
        assert!(shared.assist_undo.try_borrow().unwrap().is_some());
        shared.clear_assist_undo();
        assert!(shared.assist_undo.try_borrow().unwrap().is_none());
    }

    /// R-5: ngữ nghĩa Mode 2 **duy nhất** cho 3 OS.
    #[test]
    fn mode_2_chi_bat_khi_autocorrect_va_safe() {
        use textvn_config::{AssistAutoFix, AssistMode};
        assert!(crate::assist::mode2_auto_fix_enabled(
            AssistMode::Autocorrect,
            AssistAutoFix::Safe
        ));
        // Thiếu 1 trong 2 ⇒ không tự sửa (highlight + safe vẫn chỉ gợi ý).
        assert!(!crate::assist::mode2_auto_fix_enabled(
            AssistMode::Highlight,
            AssistAutoFix::Safe
        ));
        assert!(!crate::assist::mode2_auto_fix_enabled(
            AssistMode::Autocorrect,
            AssistAutoFix::Off
        ));
        assert!(!crate::assist::mode2_auto_fix_enabled(
            AssistMode::Off,
            AssistAutoFix::Off
        ));
        // Mode 1 (điểm hiển thị) bật khi mode != off.
        assert!(crate::assist::mode1_display_enabled(AssistMode::Highlight));
        assert!(crate::assist::mode1_display_enabled(
            AssistMode::Autocorrect
        ));
        assert!(!crate::assist::mode1_display_enabled(AssistMode::Off));
    }

    /// L3 (P4-0 §5.1): engine sửa bằng `Fix::Delete` — **xoá** span (khoảng
    /// trắng thừa); `suggest` chỉ là đầu câu để hiển thị. Adapter KHÔNG được
    /// chèn `suggest` (chèn ⇒ nhân đôi dấu câu "xin chào,, bạn").
    #[test]
    fn l3_xoa_span_khong_nhan_doi_dau_cau() {
        let sentence = "xin chào , bạn";
        // Span THẬT của engine cho L3 = khoảng trắng thừa (8..9); `suggest` =
        // dấu câu (chỉ để hiển thị) — xem `assist/src/rules/space_before_punct.rs`.
        let issue = crate::assist::IssueView {
            start: 8,
            end: 9,
            message: "Thừa khoảng trắng trước dấu câu".to_string(),
            suggestion: Some(",".to_string()),
            auto_fixable: true,
            kind: textvn_ffi::IME_ASSIST_KIND_SPACE_BEFORE_PUNCT,
        };
        let view =
            crate::assist::OverlayView::from_parts(sentence.chars().collect(), vec![issue], 1)
                .expect("view");

        // Mode 2 (A3): theo kind ⇒ kế hoạch XOÁ span (không chèn `suggest`).
        let plan =
            crate::assist::plan_sentence_auto_fix(sentence, &view.issues, |t| t.chars().count())
                .expect("plan");
        assert_eq!(plan.delete_from, 8);
        assert_eq!(plan.original_tail, " , bạn");
        let replacement: String = plan.replacement.iter().collect();
        assert_eq!(replacement, ", bạn");
        let prefix: String = sentence.chars().take(plan.delete_from).collect();
        assert_eq!(
            format!("{prefix}{replacement}"),
            "xin chào, bạn",
            "dấu câu KHÔNG được nhân đôi"
        );

        // Mode 1 (A2): cùng ngữ nghĩa `Fix::Delete` (kind) ⇒ xoá span, KHÔNG
        // chèn `suggest` (không cần nhánh riêng cho L3).
        let a2 =
            crate::assist::plan_fix(&view, 0, |t: &str| t.chars().count()).expect("kế hoạch A2");
        assert_eq!(a2.delete_chars, " , bạn".chars().count());
        assert_eq!(a2.delete_glyphs, 6);
        assert_eq!(a2.replacement.iter().collect::<String>(), ", bạn");
    }

    #[test]
    fn tsf_shared_assist_auto_fix_state_tracking() {
        let client = crate::ipc_client::IpcClient::global("test_autofix.exe");
        let thread = crate::ThreadState::new("test_autofix.exe", 0).unwrap();
        let shared = super::TsfShared::new(1, client, thread);

        // Ban đầu assist tắt
        assert!(!shared.is_assist_auto_fix_enabled());

        // Bật assist trong IPC
        client
            .state
            .assist_enabled
            .store(true, std::sync::atomic::Ordering::Release);
        // Nhưng auto_fix mặc định là Off -> vẫn false
        assert!(!shared.is_assist_auto_fix_enabled());

        // Nạp config với mode = autocorrect và auto_fix = safe (chuẩn Mode 2)
        let cfg_json = br#"{"config_version":1,"assist":{"mode":"autocorrect","auto_fix":"safe"}}"#;
        shared
            .thread
            .borrow_mut()
            .reload_config_bytes(cfg_json)
            .unwrap();
        assert!(shared.is_assist_auto_fix_enabled());

        // Tắt assist từ tray kill-switch -> false
        client
            .state
            .assist_enabled
            .store(false, std::sync::atomic::Ordering::Release);
        assert!(!shared.is_assist_auto_fix_enabled());
    }
}
