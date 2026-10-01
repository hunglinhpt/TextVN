// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfDisplayAttributeProvider` + `ITfDisplayAttributeInfo` (P1-1 §7).
//! Cung cấp thuộc tính hiển thị `TF_LS_NONE` để ứng dụng TSF (Notepad, Word, Chrome...)
//! không vẽ gạch chân dưới chữ đang gõ (sửa triệt để "bug gõ chữ bị gạch chân").

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::{E_INVALIDARG, E_POINTER, S_FALSE};
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::guids::DISPATTR_TEXTVN;

#[cfg(windows)]
#[implement(ITfDisplayAttributeProvider)]
pub struct TextVNDisplayAttributeProvider {
    _guard: ObjGuard,
}

#[cfg(windows)]
impl TextVNDisplayAttributeProvider {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
        }
    }
}

#[cfg(windows)]
impl Default for TextVNDisplayAttributeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl ITfDisplayAttributeProvider_Impl for TextVNDisplayAttributeProvider_Impl {
    fn EnumDisplayAttributeInfo(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        let info: ITfDisplayAttributeInfo = TextVNDisplayAttributeInfo::new().into();
        Ok(TextVNEnumDisplayAttributeInfo::new(vec![info]).into())
    }

    fn GetDisplayAttributeInfo(&self, guid: *const GUID) -> Result<ITfDisplayAttributeInfo> {
        if guid.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        let g = unsafe { *guid };
        if g == DISPATTR_TEXTVN {
            Ok(TextVNDisplayAttributeInfo::new().into())
        } else {
            Err(Error::from_hresult(E_INVALIDARG))
        }
    }
}

#[cfg(windows)]
#[implement(ITfDisplayAttributeInfo)]
pub struct TextVNDisplayAttributeInfo {
    _guard: ObjGuard,
}

#[cfg(windows)]
impl TextVNDisplayAttributeInfo {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
        }
    }
}

#[cfg(windows)]
impl Default for TextVNDisplayAttributeInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl ITfDisplayAttributeInfo_Impl for TextVNDisplayAttributeInfo_Impl {
    fn GetGUID(&self) -> Result<GUID> {
        Ok(DISPATTR_TEXTVN)
    }

    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::from("TextVN Clean Composition (No Underline)"))
    }

    fn GetAttributeInfo(&self, pda: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        if pda.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        unsafe {
            *pda = TF_DISPLAYATTRIBUTE {
                crText: TF_DA_COLOR {
                    r#type: TF_CT_NONE,
                    Anonymous: Default::default(),
                },
                crBk: TF_DA_COLOR {
                    r#type: TF_CT_NONE,
                    Anonymous: Default::default(),
                },
                lsStyle: TF_LS_NONE, // Không vẽ đường gạch chân
                fBoldLine: BOOL(0),
                crLine: TF_DA_COLOR {
                    r#type: TF_CT_NONE,
                    Anonymous: Default::default(),
                },
                bAttr: TF_ATTR_INPUT,
            };
        }
        Ok(())
    }

    fn SetAttributeInfo(&self, _pda: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(windows)]
#[implement(IEnumTfDisplayAttributeInfo)]
pub struct TextVNEnumDisplayAttributeInfo {
    _guard: ObjGuard,
    items: Vec<ITfDisplayAttributeInfo>,
    index: std::cell::Cell<usize>,
}

#[cfg(windows)]
impl TextVNEnumDisplayAttributeInfo {
    pub fn new(items: Vec<ITfDisplayAttributeInfo>) -> Self {
        Self {
            _guard: ObjGuard::new(),
            items,
            index: std::cell::Cell::new(0),
        }
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl IEnumTfDisplayAttributeInfo_Impl for TextVNEnumDisplayAttributeInfo_Impl {
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        let copy = TextVNEnumDisplayAttributeInfo::new(self.items.clone());
        copy.index.set(self.index.get());
        Ok(copy.into())
    }

    fn Next(
        &self,
        ulcount: u32,
        rginfo: *mut Option<ITfDisplayAttributeInfo>,
        pcfetched: *mut u32,
    ) -> Result<()> {
        if rginfo.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }
        let cur = self.index.get();
        let avail = self.items.len().saturating_sub(cur);
        let count = (ulcount as usize).min(avail);
        for i in 0..count {
            unsafe {
                *rginfo.add(i) = Some(self.items[cur + i].clone());
            }
        }
        self.index.set(cur + count);
        if !pcfetched.is_null() {
            unsafe { *pcfetched = count as u32 };
        }
        if count == ulcount as usize {
            Ok(())
        } else {
            Err(Error::from_hresult(S_FALSE))
        }
    }

    fn Reset(&self) -> Result<()> {
        self.index.set(0);
        Ok(())
    }

    fn Skip(&self, ulcount: u32) -> Result<()> {
        let cur = self.index.get();
        let next = (cur + ulcount as usize).min(self.items.len());
        self.index.set(next);
        Ok(())
    }
}
