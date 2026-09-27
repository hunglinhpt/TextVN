// SPDX-License-Identifier: GPL-3.0-or-later
//! `tsf-min` — SPIKE WIN-002: TIP (Text Service) tối thiểu bằng Rust (`P1-1 §9`).
//!
//! Mục tiêu checklist `docs/specs/tsf-spike.md`:
//!   #1 `CoCreateInstance(CLSID_TF_ThreadMgr)` trong DLL của ta
//!   #2 QI `ITfKeystrokeMgr` → `AdviseKeyEventSink` → `OnKeyDown` trong Notepad
//!   #3 `GetStart`/`GetSelection` trong edit session
//!   #4 `StartComposition` → `SetText` → `EndComposition` hiện chữ "được" trong Notepad
//!   #5 `OnTestKeyDown` vs `OnKeyDown` (log cả 2 để lập bảng app)
//!
//! An toàn (Handbook S2): log **chỉ** chứa vk/số HRESULT — không ghi text người dùng;
//! chuỗi "được" là hằng số test của dev, không phải input người dùng.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicI32, Ordering};

use windows::core::*;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Com::*;
use windows::Win32::UI::TextServices::*;

/// CLSID TIP — cùng giá trị với `tsf-min-register` (spike; sẽ là GUID khác ở WIN-010).
pub const CLSID_TEXTVN_TIP: GUID = GUID::from_u128(0x6b7e_1f80_4a2d_4e93_9c55_1f0a_7d2e_9c11);
/// GUID language profile (spike, LANGID 0x042A = tiếng Việt).
pub const PROFILE_GUID: GUID = GUID::from_u128(0x9d4c_2a71_83be_4f66_b0aa_55c9_d1e7_ab30);

const HR_S_OK: HRESULT = HRESULT(0);
const HR_S_FALSE: HRESULT = HRESULT(1);
const HR_E_POINTER: HRESULT = HRESULT(0x8000_4003_u32 as i32);
const HR_E_NOTIMPL: HRESULT = HRESULT(0x8000_4001_u32 as i32);
const HR_CLASSNOTAVAILABLE: HRESULT = HRESULT(0x8004_0111_u32 as i32);

/// Phím kích hoạt composition của spike: 'D' → gõ "được" (VK 0x44).
const VK_TRIGGER: usize = 0x44;

static OBJECTS: AtomicI32 = AtomicI32::new(0);

/// Đếm COM object sống → `DllCanUnloadNow`.
struct ObjGuard;

impl ObjGuard {
    fn new() -> Self {
        OBJECTS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for ObjGuard {
    fn drop(&mut self) {
        OBJECTS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Ghi log phụ trợ spike vào `%LOCALAPPDATA%\TextVN\logs\tsf-min.log` (S2: không text người dùng).
fn log(msg: &str) {
    use std::io::Write;
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let mut dir = std::path::PathBuf::from(base);
    dir.push("TextVN");
    dir.push("logs");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let mut path = dir;
    path.push("tsf-min.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(f, "[{ms}] [{pid}] {msg}", pid = std::process::id());
    }
}

// ---------------------------------------------------------------------------
// ITfEditSession — chạy trong write-lock của app (checklist #3 + #4)
// ---------------------------------------------------------------------------

#[implement(ITfEditSession)]
struct EditSession {
    _guard: ObjGuard,
    ctx: ITfContext,
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        // #3a: GetStart
        let start = unsafe { self.ctx.GetStart(ec) }?;
        log("CHK#3 GetStart=OK");

        // #3b: GetSelection (TF_DEFAULT_SELECTION = index hiện tại)
        let mut sel = [TF_SELECTION::default()];
        let mut fetched: u32 = 0;
        match unsafe {
            self.ctx
                .GetSelection(ec, TS_DEFAULT_SELECTION, &mut sel, &mut fetched)
        } {
            Ok(()) => log(&format!("CHK#3 GetSelection=OK fetched={fetched}")),
            Err(e) => log(&format!("CHK#3 GetSelection=FAIL hr={:#010x}", e.code().0)),
        }
        // TF_SELECTION.range là ManuallyDrop → tự giải phóng ref (tránh leak).
        let owned = std::mem::replace(&mut sel[0].range, std::mem::ManuallyDrop::new(None));
        drop(std::mem::ManuallyDrop::into_inner(owned));

        // #4: StartComposition → SetText → EndComposition
        let ctxcomp: ITfContextComposition = self.ctx.cast()?;
        let csink: ITfCompositionSink = CompSink {
            _guard: ObjGuard::new(),
        }
        .into();
        let comp = unsafe { ctxcomp.StartComposition(ec, &start, &csink) }?;
        log("CHK#4 StartComposition=OK");
        // Cùng cảnh báo như register.rs: pad số 0 phía sau slice để TSF wcslen() an toàn.
        let raw: Vec<u16> = "được".encode_utf16().collect();
        let tbuf: Vec<u16> = raw.iter().copied().chain(std::iter::once(0)).collect();
        let crange = unsafe { comp.GetRange() }?;
        unsafe { crange.SetText(ec, 0, &tbuf[..raw.len()]) }?;
        log("CHK#4 SetText=OK");
        unsafe { comp.EndComposition(ec) }?;
        log("CHK#4 EndComposition=OK");
        Ok(())
    }
}

/// Sink nhận thông báo composition kết thúc ngoài dự kiến (B2 — commit-before-hide).
#[implement(ITfCompositionSink)]
struct CompSink {
    _guard: ObjGuard,
}

impl ITfCompositionSink_Impl for CompSink_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Ref<'_, ITfComposition>,
    ) -> Result<()> {
        log("CHK#4 OnCompositionTerminated (xảy ra ngoài phiên của ta)");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ITfKeyEventSink — nhận key từ TSF (checklist #2/#5)
// ---------------------------------------------------------------------------

#[implement(ITfKeyEventSink)]
struct KeySink {
    _guard: ObjGuard,
    tid: u32,
}

impl ITfKeyEventSink_Impl for KeySink_Impl {
    fn OnSetFocus(&self, fforeground: BOOL) -> Result<()> {
        log(&format!("OnSetFocus fg={}", fforeground.0));
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        // #5: ghi nhận app trả gì ở phase TEST — chỉ 'D' là ta "ăn".
        log(&format!("OnTestKeyDown vk={:#x}", wparam.0));
        Ok(BOOL::from(wparam.0 == VK_TRIGGER))
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        log(&format!("OnKeyDown vk={:#x}", wparam.0));
        if wparam.0 != VK_TRIGGER {
            return Ok(BOOL::from(false));
        }
        let ctx = pic.ok()?.clone();
        let ses: ITfEditSession = EditSession {
            _guard: ObjGuard::new(),
            ctx: ctx.clone(),
        }
        .into();
        match unsafe { ctx.RequestEditSession(self.tid, &ses, TF_ES_READWRITE | TF_ES_SYNC) } {
            Ok(hr) if hr.0 >= 0 => log(&format!("CHK#3 RequestEditSession=OK hr={:#010x}", hr.0)),
            Ok(hr) => log(&format!(
                "CHK#3 RequestEditSession=S_OK-but hr={:#010x}",
                hr.0
            )),
            Err(e) => log(&format!(
                "CHK#3 RequestEditSession=FAIL hr={:#010x}",
                e.code().0
            )),
        }
        Ok(BOOL::from(true)) // ăn phím 'D' — text đã được composition thay
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }
}

// ---------------------------------------------------------------------------
// ITfTextInputProcessor — vòng đời (Activate/Deactivate), `P1-1 §3`
// ---------------------------------------------------------------------------

struct TipInner {
    tid: u32,
    keymgr: Option<ITfKeystrokeMgr>,
    _sink: Option<ITfKeyEventSink>,
}

#[implement(ITfTextInputProcessor)]
struct Tip {
    _guard: ObjGuard,
    // STA: mọi callback TSF cùng thread → UnsafeCell đủ (spike; production xem P1-1 §3).
    inner: UnsafeCell<TipInner>,
}

impl Tip {
    fn with<R>(&self, f: impl FnOnce(&mut TipInner) -> R) -> R {
        f(unsafe { &mut *self.inner.get() })
    }
}

impl ITfTextInputProcessor_Impl for Tip_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        log(&format!("Activate tid={tid}"));

        // #1: CoCreateInstance(CLSID_TF_ThreadMgr) từ trong DLL của ta.
        match unsafe {
            CoCreateInstance::<_, ITfThreadMgr>(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER)
        } {
            Ok(_) => log("CHK#1 CoCreateInstance(CLSID_TF_ThreadMgr)=OK"),
            Err(e) => log(&format!(
                "CHK#1 CoCreateInstance=FAIL hr={:#010x}",
                e.code().0
            )),
        }

        // #2: QI ITfKeystrokeMgr → AdviseKeyEventSink.
        let mgr = ptim.ok()?.clone();
        let keymgr: ITfKeystrokeMgr = mgr.cast()?;
        let sink: ITfKeyEventSink = KeySink {
            _guard: ObjGuard::new(),
            tid,
        }
        .into();
        unsafe { keymgr.AdviseKeyEventSink(tid, &sink, true) }?;
        log(&format!("CHK#2 AdviseKeyEventSink(tid={tid})=OK"));

        self.with(|i| {
            i.tid = tid;
            i.keymgr = Some(keymgr);
            i._sink = Some(sink);
        });
        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        let (tid, keymgr) = self.with(|i| (i.tid, i.keymgr.take()));
        let _ = self.with(|i| i._sink.take());
        if let Some(km) = keymgr {
            match unsafe { km.UnadviseKeyEventSink(tid) } {
                Ok(()) => log("Deactivate UnadviseKeyEventSink=OK"),
                Err(e) => log(&format!(
                    "Deactivate UnadviseKeyEventSink=FAIL hr={:#010x}",
                    e.code().0
                )),
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// COM plumbing: ClassFactory + exports
// ---------------------------------------------------------------------------

#[implement(IClassFactory)]
struct ClassFactory {
    _guard: ObjGuard,
}

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut std::ffi::c_void,
    ) -> Result<()> {
        if !punkouter.is_null() {
            return Err(Error::from_hresult(HR_E_NOTIMPL)); // không support aggregation
        }
        if ppvobject.is_null() {
            return Err(Error::from_hresult(HR_E_POINTER));
        }
        unsafe { *ppvobject = std::ptr::null_mut() };
        let tip: ITfTextInputProcessor = Tip {
            _guard: ObjGuard::new(),
            inner: UnsafeCell::new(TipInner {
                tid: 0,
                keymgr: None,
                _sink: None,
            }),
        }
        .into();
        let hr = unsafe { tip.query(riid, ppvobject) };
        if hr < HR_S_OK {
            return Err(Error::from_hresult(hr));
        }
        Ok(())
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

/// `DllGetClassObject` — COM gọi khi loader cần ClassFactory của TIP.
///
/// # Safety
/// `rclsid`/`riid` phải trỏ tới GUID hợp lệ, `ppv` phải là con trỏ ghi được;
/// checked null trước khi dereference (HR_E_POINTER).
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return HR_E_POINTER;
    }
    unsafe { *ppv = std::ptr::null_mut() };
    if unsafe { *rclsid } != CLSID_TEXTVN_TIP {
        return HR_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = ClassFactory {
        _guard: ObjGuard::new(),
    }
    .into();
    let hr = unsafe { factory.query(riid, ppv) };
    log(&format!("DllGetClassObject → {:#010x}", hr.0));
    hr
}

/// `DllCanUnloadNow` — COM hỏi DLL có thể unload không (đếm object sống qua `OBJECTS`).
///
/// # Safety
/// Hàm export thuần đọc atomic toàn cục — không dereference con trỏ ngoài.
#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if OBJECTS.load(Ordering::SeqCst) == 0 {
        HR_S_OK
    } else {
        HR_S_FALSE
    }
}
