// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfEditSession` implementation for TextVN TSF (WIN-012/013/018).

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;

#[cfg(windows)]
#[implement(ITfCompositionSink)]
pub struct CompSink {
    _guard: ObjGuard,
}

#[cfg(windows)]
impl CompSink {
    pub fn new() -> Self {
        Self {
            _guard: ObjGuard::new(),
        }
    }
}

#[cfg(windows)]
impl Default for CompSink {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl ITfCompositionSink_Impl for CompSink_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Ref<'_, ITfComposition>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Action to perform inside `DoEditSession`.
#[cfg(windows)]
pub enum EditAction {
    /// Sửa text tại caret (xóa delete_count ký tự lùi, chèn insert UTF-16).
    BackspaceType { delete_count: u16, insert: Vec<u16> },
    /// Chèn text composition và giữ composition mở (preedit).
    Preedit { preedit: Vec<u16> },
    /// Đóng composition và commit text cuối cùng.
    Commit { insert: Vec<u16> },
    /// Thay thế vùng chọn do adapter sở hữu (address bar / Excel - Bug B1).
    SelectionReplace { delete_count: u16, insert: Vec<u16> },
    /// Gõ trực tiếp không composition (terminal / console - Bug B8).
    ForwardAsCommit { insert: Vec<u16> },
}

#[cfg(windows)]
#[implement(ITfEditSession)]
pub struct ReplaceEditSession {
    _guard: ObjGuard,
    ctx: ITfContext,
    action: EditAction,
}

#[cfg(windows)]
impl ReplaceEditSession {
    pub fn new(ctx: ITfContext, action: EditAction) -> Self {
        Self {
            _guard: ObjGuard::new(),
            ctx,
            action,
        }
    }
}

#[cfg(windows)]
impl ITfEditSession_Impl for ReplaceEditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        // Finding S3-1: Anchor lấy từ GetSelection, không dùng GetStart.
        let mut sel = [TF_SELECTION::default()];
        let mut fetched: u32 = 0;
        // SAFETY: sel and fetched pointers are valid and caller is in edit session ec.
        unsafe {
            self.ctx
                .GetSelection(ec, TS_DEFAULT_SELECTION, &mut sel, &mut fetched)
        }?;
        if fetched == 0 || sel[0].range.is_none() {
            return Ok(());
        }

        let range = sel[0].range.as_ref().unwrap();

        match &self.action {
            EditAction::BackspaceType {
                delete_count,
                insert,
            } => {
                if *delete_count > 0 {
                    let mut shifted: i32 = 0;
                    // Lùi start sang trái delete_count ký tự UTF-16.
                    // SAFETY: range and ec are valid inside DoEditSession.
                    unsafe {
                        range.ShiftStart(
                            ec,
                            -i32::from(*delete_count),
                            &mut shifted,
                            std::ptr::null(),
                        )
                    }?;
                }

                // Finding S3-2: buffer truyền SetText phải có 0 terminator đệm sau.
                let mut tbuf = insert.clone();
                tbuf.push(0);
                // SAFETY: tbuf is null-terminated and range is valid.
                unsafe { range.SetText(ec, 0, &tbuf[..insert.len()]) }?;

                // Di chuyển selection về cuối văn bản vừa chèn.
                // SAFETY: ec and range are valid.
                unsafe {
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    sel[0].style.ase = TF_AE_END;
                    let _ = self.ctx.SetSelection(ec, &sel);
                }
            }
            EditAction::Preedit { preedit } => {
                let ctxcomp: ITfContextComposition = self.ctx.cast()?;
                let csink: ITfCompositionSink = CompSink::new().into();
                // SAFETY: ctxcomp and csink are valid COM interfaces.
                let comp = unsafe { ctxcomp.StartComposition(ec, range, &csink) }?;
                let mut tbuf = preedit.clone();
                tbuf.push(0);
                // SAFETY: comp.GetRange() returns valid ITfRange.
                unsafe {
                    let comp_range = comp.GetRange()?;
                    comp_range.SetText(ec, 0, &tbuf[..preedit.len()])?;
                }
            }
            EditAction::Commit { insert } => {
                let mut tbuf = insert.clone();
                tbuf.push(0);
                // SAFETY: range is valid in edit session ec.
                unsafe {
                    range.SetText(ec, 0, &tbuf[..insert.len()])?;
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    sel[0].style.ase = TF_AE_END;
                    let _ = self.ctx.SetSelection(ec, &sel);
                }
            }
            EditAction::SelectionReplace {
                delete_count,
                insert,
            } => {
                if *delete_count > 0 {
                    let mut shifted: i32 = 0;
                    // Lùi start sang trái delete_count ký tự UTF-16 mà không gửi phím Backspace
                    // SAFETY: range and ec are valid inside DoEditSession.
                    unsafe {
                        range.ShiftStart(
                            ec,
                            -i32::from(*delete_count),
                            &mut shifted,
                            std::ptr::null(),
                        )
                    }?;
                }
                let mut tbuf = insert.clone();
                tbuf.push(0);
                // SAFETY: tbuf is null-terminated and range is valid.
                unsafe {
                    range.SetText(ec, 0, &tbuf[..insert.len()])?;
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    sel[0].style.ase = TF_AE_END;
                    let _ = self.ctx.SetSelection(ec, &sel);
                }
            }
            EditAction::ForwardAsCommit { insert } => {
                let mut tbuf = insert.clone();
                tbuf.push(0);
                // SAFETY: range is valid in edit session ec.
                unsafe {
                    range.SetText(ec, 0, &tbuf[..insert.len()])?;
                    range.Collapse(ec, TF_ANCHOR_END)?;
                    sel[0].style.ase = TF_AE_END;
                    let _ = self.ctx.SetSelection(ec, &sel);
                }
            }
        }

        // Clean-up ManuallyDrop trong TF_SELECTION
        let owned = std::mem::replace(&mut sel[0].range, std::mem::ManuallyDrop::new(None));
        drop(std::mem::ManuallyDrop::into_inner(owned));

        Ok(())
    }
}
