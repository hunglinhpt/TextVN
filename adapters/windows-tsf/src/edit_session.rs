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
use windows::Win32::System::Variant::{VariantClear, VT_UNKNOWN};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetWindowLongW, ES_PASSWORD, GWL_STYLE,
};

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::compose::{plan_key, text_after, CompositionPlan, EngineStep, KeyKind, ModifierToggle};
#[cfg(windows)]
use crate::ipc_client::IpcClient;
#[cfg(windows)]
use crate::{classify_tsf_field, ThreadState, TsfFieldSignals};
#[cfg(windows)]
use textvn_ffi::IME_FLAG_ERROR;
#[cfg(windows)]
use textvn_strategy::Strategy;

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
    /// Phiên bản config đã nạp vào engine của thread này.
    pub config_seen: Cell<u64>,
    /// App/TSF kết thúc composition của ta (OnCompositionTerminated).
    pub terminated: Cell<bool>,
    /// Phím chuyển Ctrl+Shift kiểu UniKey.
    pub modifier_toggle: Cell<ModifierToggle>,
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
            config_seen: Cell::new(0),
            terminated: Cell::new(false),
            modifier_toggle: Cell::new(ModifierToggle::default()),
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
    pub fn reset_engine(&self) {
        if let Ok(mut thread) = self.thread.try_borrow_mut() {
            thread.engine.reset();
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
                let _ = thread.reload_config_from_file();
                self.config_seen.set(latest);
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
        let allowed = match shared.thread.try_borrow_mut() {
            Ok(mut thread) => {
                thread.apply_tsf_probe(classify_tsf_field(&signals));
                thread.resolve_strategy_with_state(Some(shared.ipc.is_enabled()), None)
                    != Strategy::Passthrough
            }
            Err(_) => false,
        };
        if !allowed {
            finish(ec, comp, shared);
            return false;
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
        if plan.delete_before > 0 {
            let anchor = match comp.as_ref() {
                Some(c) => c.GetRange()?,
                None => sel.Clone()?,
            };
            let del = anchor.Clone()?;
            del.Collapse(ec, TF_ANCHOR_START)?;
            let want = -i32::from(plan.delete_before);
            let mut shifted = 0i32;
            del.ShiftStart(ec, want, &mut shifted, std::ptr::null())?;
            // Text store không cho lùi đủ (app IMM32): không xóa gì cả.
            if shifted != want {
                return Err(Error::from(E_FAIL));
            }
            let empty = [0u16; 1];
            del.SetText(ec, 0, &empty[..0])?;
        }

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
                set_caret_after(ctx, ec, &range);
            }
        }

        if plan.end {
            if let Some(c) = comp.take() {
                if let Ok(range) = c.GetRange() {
                    set_caret_after(ctx, ec, &range);
                }
                c.EndComposition(ec)?;
            }
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
fn selection_range(ctx: &ITfContext, ec: u32) -> Option<ITfRange> {
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

/// Đặt caret (selection rỗng) ngay sau `range`.
#[cfg(windows)]
fn set_caret_after(ctx: &ITfContext, ec: u32, range: &ITfRange) {
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

/// Caret/selection còn nằm trong composition? Chỉ trả `false` khi CHẮC CHẮN nằm
/// ngoài (click chuột sang chỗ khác, app tự đổi selection). Lỗi API của text store
/// tối giản (CUAS) coi như còn trong: commit nhầm mỗi phím còn tệ hơn.
#[cfg(windows)]
fn caret_at_composition_end(ec: u32, comp: &ITfComposition, sel: &ITfRange) -> bool {
    // SAFETY: gọi trong edit session.
    unsafe {
        let Ok(range) = comp.GetRange() else {
            return true;
        };
        // start(comp) > start(sel): selection bắt đầu trước composition.
        let starts_before = range
            .CompareStart(ec, sel, TF_ANCHOR_START)
            .map(|c| c > 0)
            .unwrap_or(false);
        // end(comp) < end(sel): selection kéo ra sau composition.
        let ends_after = range
            .CompareEnd(ec, sel, TF_ANCHOR_END)
            .map(|c| c < 0)
            .unwrap_or(false);
        !(starts_before || ends_after)
    }
}

/// Tín hiệu field in-proc: read-only, InputScope, style `ES_PASSWORD` của HWND focus.
#[cfg(windows)]
fn read_field_signals(ctx: &ITfContext, ec: u32, sel: &ITfRange) -> TsfFieldSignals {
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
}
