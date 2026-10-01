// SPDX-License-Identifier: GPL-3.0-or-later
//! `IClassFactory` implementation for TextVN TSF (WIN-010).

#[cfg(windows)]
use std::sync::atomic::{AtomicI32, Ordering};

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::System::Com::*;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::tip::Tip;

#[cfg(windows)]
pub static OBJECT_COUNT: AtomicI32 = AtomicI32::new(0);

#[cfg(windows)]
pub struct ObjGuard;

#[cfg(windows)]
impl ObjGuard {
    pub fn new() -> Self {
        OBJECT_COUNT.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

#[cfg(windows)]
impl Default for ObjGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl Drop for ObjGuard {
    fn drop(&mut self) {
        OBJECT_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(windows)]
const HR_E_POINTER: HRESULT = HRESULT(0x8000_4003_u32 as i32);
#[cfg(windows)]
/// Aggregation không hỗ trợ — COM host kiểm tra mã chuẩn này (0x80040110).
const HR_CLASS_E_NOAGGREGATION: HRESULT = HRESULT(0x8004_0110_u32 as i32);
#[cfg(windows)]
const HR_S_OK: HRESULT = HRESULT(0);

#[cfg(windows)]
#[implement(IClassFactory)]
pub struct ClassFactory {
    _guard: ObjGuard,
}

#[cfg(windows)]
impl ClassFactory {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
        }
    }
}

#[cfg(windows)]
impl Default for ClassFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut std::ffi::c_void,
    ) -> Result<()> {
        if !punkouter.is_null() {
            return Err(Error::from_hresult(HR_CLASS_E_NOAGGREGATION)); // Aggregation not supported
        }
        if ppvobject.is_null() || riid.is_null() {
            return Err(Error::from_hresult(HR_E_POINTER));
        }
        // SAFETY: ppvobject is verified non-null above.
        unsafe { *ppvobject = std::ptr::null_mut() };

        let req_iid = unsafe { *riid };
        if req_iid == ITfDisplayAttributeProvider::IID {
            let prov: ITfDisplayAttributeProvider =
                crate::display_attr::TextVNDisplayAttributeProvider::new().into();
            let hr = unsafe { prov.query(riid, ppvobject) };
            if hr < HR_S_OK {
                return Err(Error::from_hresult(hr));
            }
            return Ok(());
        }

        let tip: ITfTextInputProcessor = Tip::new().into();
        // SAFETY: riid and ppvobject are verified non-null above.
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
