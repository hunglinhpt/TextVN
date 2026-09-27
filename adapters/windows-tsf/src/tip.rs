// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfTextInputProcessorEx` and `ITfThreadMgrEventSink` implementation for VietIME TSF (WIN-010/014).

#[cfg(windows)]
use std::cell::{RefCell, UnsafeCell};
#[cfg(windows)]
use std::rc::Rc;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::key_event::KeySink;
#[cfg(windows)]
use crate::ThreadState;

#[cfg(windows)]
struct TipInner {
    tid: u32,
    keymgr: Option<ITfKeystrokeMgr>,
    _sink: Option<ITfKeyEventSink>,
    thread_state: Option<Rc<RefCell<ThreadState>>>,
}

#[cfg(windows)]
#[implement(ITfTextInputProcessorEx, ITfTextInputProcessor, ITfThreadMgrEventSink)]
pub struct Tip {
    _guard: ObjGuard,
    inner: UnsafeCell<TipInner>,
}

#[cfg(windows)]
impl Tip {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
            inner: UnsafeCell::new(TipInner {
                tid: 0,
                keymgr: None,
                _sink: None,
                thread_state: None,
            }),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut TipInner) -> R) -> R {
        // SAFETY: TSF callbacks for a given TIP occur on the same STA thread.
        f(unsafe { &mut *self.inner.get() })
    }
}

#[cfg(windows)]
impl Default for Tip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl ITfTextInputProcessor_Impl for Tip_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        let (tid, keymgr) = self.with(|i| (i.tid, i.keymgr.take()));
        self.with(|i| {
            i._sink.take();
            i.thread_state.take();
        });
        if let Some(km) = keymgr {
            // SAFETY: Unadvising key sink registered during Activate.
            unsafe {
                let _ = km.UnadviseKeyEventSink(tid);
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
impl ITfTextInputProcessorEx_Impl for Tip_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _dwflags: u32) -> Result<()> {
        let mgr = ptim.ok()?.clone();

        let keymgr: ITfKeystrokeMgr = mgr.cast()?;

        // Khởi tạo ThreadState cho tiến trình hiện hành
        let exe_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
            .unwrap_or_else(|| "unknown.exe".into());

        let state = match ThreadState::new(exe_name, vietime_strategy::IME_CAP_PREEDIT) {
            Ok(s) => Rc::new(RefCell::new(s)),
            Err(e) => return Err(Error::from_hresult(HRESULT(e))),
        };

        let sink: ITfKeyEventSink = KeySink::new(tid, state.clone()).into();
        // SAFETY: keymgr is a valid ITfKeystrokeMgr interface.
        unsafe { keymgr.AdviseKeyEventSink(tid, &sink, true) }?;

        self.with(|i| {
            i.tid = tid;
            i.keymgr = Some(keymgr);
            i._sink = Some(sink);
            i.thread_state = Some(state);
        });

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
        pdimnew: Ref<'_, ITfDocumentMgr>,
        _pdimprev: Ref<'_, ITfDocumentMgr>,
    ) -> Result<()> {
        // Bug B2 (commit-before-hide): nếu mất focus (pdimnew == NULL), reset engine buffer
        if pdimnew.is_null() {
            self.with(|i| {
                if let Some(state) = &i.thread_state {
                    let mut s = state.borrow_mut();
                    let _ = s.engine.reject_edit_session();
                }
            });
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
