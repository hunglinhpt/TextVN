// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfTextInputProcessorEx` + `ITfThreadMgrEventSink` của TextVN TSF (WIN-010/014/015).

#[cfg(windows)]
use std::cell::RefCell;
#[cfg(windows)]
use std::rc::Rc;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::edit_session::TsfShared;
#[cfg(windows)]
use crate::guids::GUID_PRESERVED_TOGGLE;
#[cfg(windows)]
use crate::ipc_client::IpcClient;
#[cfg(windows)]
use crate::key_event::{end_composition, KeySink, KeyTraceSink};
#[cfg(windows)]
use crate::ThreadState;

/// Hotkey bật/tắt tiếng Việt trong TSF (khớp `config.hotkeys.toggle_vi_en`).
#[cfg(windows)]
const TOGGLE_KEY: TF_PRESERVEDKEY = TF_PRESERVEDKEY {
    uVKey: VK_SPACE.0 as u32,
    uModifiers: TF_MOD_CONTROL | TF_MOD_SHIFT,
};

#[cfg(windows)]
#[derive(Default)]
struct TipInner {
    tid: u32,
    keymgr: Option<ITfKeystrokeMgr>,
    shared: Option<Rc<TsfShared>>,
    source: Option<ITfSource>,
    sink_cookie: u32,
    trace_cookie: u32,
}

#[cfg(windows)]
#[implement(ITfTextInputProcessorEx, ITfTextInputProcessor, ITfThreadMgrEventSink)]
pub struct Tip {
    _guard: ObjGuard,
    inner: RefCell<TipInner>,
}

#[cfg(windows)]
impl Tip {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
            inner: RefCell::new(TipInner::default()),
        }
    }

    fn shared(&self) -> Option<Rc<TsfShared>> {
        self.inner.try_borrow().ok().and_then(|i| i.shared.clone())
    }
}

#[cfg(windows)]
impl Default for Tip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
fn current_exe_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_else(|| "unknown.exe".into())
}

#[cfg(windows)]
impl ITfTextInputProcessor_Impl for Tip_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        let Ok(mut inner) = self.inner.try_borrow_mut() else {
            return Ok(());
        };
        let inner = std::mem::take(&mut *inner);
        if let Some(shared) = &inner.shared {
            // B2: không bỏ lại chữ đang soạn khi người dùng đổi bộ gõ.
            end_composition(shared, None);
            shared.reset_engine();
        }
        // SAFETY: gỡ đúng các sink/hotkey đã đăng ký trong ActivateEx.
        unsafe {
            if let Some(src) = &inner.source {
                if inner.sink_cookie != 0 {
                    let _ = src.UnadviseSink(inner.sink_cookie);
                }
                if inner.trace_cookie != 0 {
                    let _ = src.UnadviseSink(inner.trace_cookie);
                }
            }
            if let Some(km) = &inner.keymgr {
                let _ = km.UnpreserveKey(&GUID_PRESERVED_TOGGLE, &TOGGLE_KEY);
                let _ = km.UnadviseKeyEventSink(inner.tid);
            }
        }
        // Phím CUAS còn chờ trả cho app được giao luôn; huỷ cửa sổ message-only của thread.
        crate::replay::shutdown();
        // IPC client là của process và KHÔNG bị join ở đây (join từng treo app).
        Ok(())
    }
}

#[cfg(windows)]
impl ITfTextInputProcessorEx_Impl for Tip_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _dwflags: u32) -> Result<()> {
        let mgr = ptim.ok()?.clone();
        let keymgr: ITfKeystrokeMgr = mgr.cast()?;

        let exe_name = current_exe_name();
        let mut thread = ThreadState::new(exe_name.clone(), textvn_strategy::IME_CAP_PREEDIT)
            .map_err(|e| Error::from_hresult(HRESULT(e)))?;
        // Config của người dùng có hiệu lực ngay cả khi tray chưa chạy.
        let _ = thread.reload_config_from_file();

        let ipc = IpcClient::global(&exe_name);
        let shared = Rc::new(TsfShared::new(tid, ipc, thread));
        shared.config_seen.set(ipc.config_version());

        let sink: ITfKeyEventSink = KeySink::new(shared.clone()).into();
        // SAFETY: keymgr là ITfKeystrokeMgr hợp lệ của thread hiện tại.
        unsafe { keymgr.AdviseKeyEventSink(tid, &sink, true) }?;

        // WIN-015: Ctrl+Shift+Space. Mô tả null-terminated (S3-2: TSF đọc bằng wcslen).
        let desc: Vec<u16> = "TextVN Toggle".encode_utf16().chain(Some(0)).collect();
        // SAFETY: keymgr hợp lệ; desc sống tới hết lời gọi.
        unsafe {
            let _ = keymgr.PreserveKey(
                tid,
                &GUID_PRESERVED_TOGGLE,
                &TOGGLE_KEY,
                &desc[..desc.len() - 1],
            );
        }

        // WIN-014: OnSetFocus → commit trước khi đổi document (B2).
        // Ctrl+Shift ở app TSF-aware chỉ thấy được qua key trace (xem `KeyTraceSink`).
        let (source, sink_cookie, trace_cookie) = match mgr.cast::<ITfSource>() {
            Ok(src) => {
                let event_sink: ITfThreadMgrEventSink = self.to_interface();
                let trace_sink: ITfKeyTraceEventSink = KeyTraceSink::new(shared.clone()).into();
                // SAFETY: src hợp lệ; cookie được Unadvise trong Deactivate.
                let cookie = unsafe { src.AdviseSink(&ITfThreadMgrEventSink::IID, &event_sink) }
                    .unwrap_or(0);
                let trace =
                    unsafe { src.AdviseSink(&ITfKeyTraceEventSink::IID, &trace_sink) }.unwrap_or(0);
                (Some(src), cookie, trace)
            }
            Err(_) => (None, 0, 0),
        };

        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            *inner = TipInner {
                tid,
                keymgr: Some(keymgr),
                shared: Some(shared),
                source,
                sink_cookie,
                trace_cookie,
            };
        }
        Ok(())
    }
}

#[cfg(windows)]
impl ITfThreadMgrEventSink_Impl for Tip_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnSetFocus(
        &self,
        _pdimnew: Ref<'_, ITfDocumentMgr>,
        _pdimprev: Ref<'_, ITfDocumentMgr>,
    ) -> Result<()> {
        // Mọi lần đổi document (kể cả mất focus): commit từ đang soạn vào đúng
        // document cũ, rồi bắt đầu sạch ở document mới — không mang buffer sang.
        if let Some(shared) = self.shared() {
            end_composition(&shared, None);
            shared.reset_engine();
            shared.pending_eaten_vk.set(None);
            shared.modifier_toggle.set(Default::default());
        }
        Ok(())
    }

    fn OnPushContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }

    fn OnPopContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }
}
