// SPDX-License-Identifier: GPL-3.0-or-later
//! `ITfKeyEventSink` implementation for TextVN TSF (WIN-011).

#[cfg(windows)]
use std::cell::RefCell;
#[cfg(windows)]
use std::rc::Rc;
#[cfg(windows)]
use std::sync::Arc;

#[cfg(windows)]
use windows::core::*;
use windows::Win32::Foundation::{LPARAM, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(windows)]
use windows::Win32::UI::TextServices::*;

#[cfg(windows)]
use crate::class::ObjGuard;
#[cfg(windows)]
use crate::edit_session::{EditAction, ReplaceEditSession};
#[cfg(windows)]
use crate::ipc_client::IpcClient;
#[cfg(windows)]
use crate::{should_bypass_engine, ThreadState};
#[cfg(windows)]
use textvn_ffi::{ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE};
#[cfg(windows)]
use textvn_strategy::Strategy;

#[cfg(windows)]
#[implement(ITfKeyEventSink)]
pub struct KeySink {
    _guard: ObjGuard,
    tid: u32,
    state: Rc<RefCell<ThreadState>>,
    ipc: Arc<IpcClient>,
}

#[cfg(windows)]
impl KeySink {
    pub fn new(tid: u32, state: Rc<RefCell<ThreadState>>, ipc: Arc<IpcClient>) -> Self {
        Self {
            _guard: ObjGuard::new(),
            tid,
            state,
            ipc,
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
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let vk = wparam.0 as u32;
        let mods = get_active_modifiers();
        if should_bypass_engine(vk, mods) {
            return Ok(BOOL::from(false));
        }
        // WIN-015: Nhận diện hotkey toggle Ctrl+Shift+Space
        if vk == VK_SPACE.0 as u32 && (mods & 0x3) == 0x3 {
            return Ok(BOOL::from(true));
        }
        let state = self.state.borrow();
        let strategy = state.resolve_strategy_with_state(self.ipc.app_enabled_override(), None);
        if strategy == Strategy::Passthrough {
            return Ok(BOOL::from(false));
        }
        // Logic ăn phím: nếu là chữ cái gõ được thì báo TRUE để app gửi OnKeyDown
        let is_typing_char = (0x30..=0x5A).contains(&vk) || (0xBA..=0xDF).contains(&vk);
        Ok(BOOL::from(is_typing_char))
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
        let vk = wparam.0 as u32;
        let mods = get_active_modifiers();

        // WIN-015: Hotkey toggle EN/VN
        if vk == VK_SPACE.0 as u32 && (mods & 0x3) == 0x3 {
            let mut state = self.state.borrow_mut();
            let _ = state.toggle_enabled();
            self.ipc.request_toggle_global();
            return Ok(BOOL::from(true));
        }

        // Bước 1: Kiểm tra chord hệ thống (Ctrl/Alt/Win) -> PASS lập tức (B6)
        if should_bypass_engine(vk, mods) {
            return Ok(BOOL::from(false));
        }

        let mut state = self.state.borrow_mut();

        // Kiểm tra tín hiệu reload config từ Tray UI qua IPC (WIN-016)
        if let Some(_ver) = self.ipc.check_config_reload() {
            let _ = state.reload_config_from_file();
        }

        // Kiểm tra strategy: Nếu Passthrough (do secure field theo WIN-017 hoặc disabled) -> PASS ngay
        let strategy = state.resolve_strategy_with_state(self.ipc.app_enabled_override(), None);
        if strategy == Strategy::Passthrough {
            return Ok(BOOL::from(false));
        }

        let ch = vk_to_unicode(vk);

        // Bước 2: Đẩy key vào engine session
        let outcome = state.engine.key_or_bypass(vk, ch, mods);
        let result = match outcome {
            Ok(Some(r)) => r,
            _ => return Ok(BOOL::from(false)),
        };

        // Bước 3: Xử lý hành động từ kết quả engine
        if result.action == ACTION_PASS {
            return Ok(BOOL::from(false));
        }

        let ctx = match pic.ok() {
            Ok(c) => c.clone(),
            Err(_) => return Ok(BOOL::from(false)),
        };

        let insert_slice = &result.insert[..result.insert_len as usize];
        let insert_utf16 = utf32_to_utf16(insert_slice);

        let preedit_slice = &result.preedit[..result.preedit_len as usize];
        let preedit_utf16 = utf32_to_utf16(preedit_slice);

        let edit_action = match result.action {
            ACTION_REPLACE => match strategy {
                Strategy::SelectionReplace => EditAction::SelectionReplace {
                    delete_count: result.delete_count,
                    insert: insert_utf16,
                },
                Strategy::ForwardAsCommit => EditAction::ForwardAsCommit {
                    insert: insert_utf16,
                },
                Strategy::Preedit => EditAction::Preedit {
                    preedit: if preedit_utf16.is_empty() {
                        insert_utf16
                    } else {
                        preedit_utf16
                    },
                },
                _ => EditAction::BackspaceType {
                    delete_count: result.delete_count,
                    insert: insert_utf16,
                },
            },
            ACTION_COMMIT => EditAction::Commit {
                insert: insert_utf16,
            },
            ACTION_RESTORE => EditAction::BackspaceType {
                delete_count: result.delete_count,
                insert: insert_utf16,
            },
            _ => return Ok(BOOL::from(false)),
        };

        let session: ITfEditSession = ReplaceEditSession::new(ctx.clone(), edit_action).into();

        // SAFETY: ctx is valid and RequestEditSession is standard TSF call.
        let hr =
            unsafe { ctx.RequestEditSession(self.tid, &session, TF_ES_READWRITE | TF_ES_SYNC) };

        match hr {
            Ok(code) if code.0 >= 0 => Ok(BOOL::from(true)),
            _ => {
                // RequestEditSession bị từ chối: reset buffer engine (P1-1 §5 note)
                state.engine.reject_edit_session();
                Ok(BOOL::from(false))
            }
        }
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, rguid: *const GUID) -> Result<BOOL> {
        if !rguid.is_null() {
            // SAFETY: rguid is provided by TSF callback, checked non-null above
            let guid = unsafe { *rguid };
            if guid == crate::guids::GUID_PRESERVED_TOGGLE {
                let mut state = self.state.borrow_mut();
                let _ = state.toggle_enabled();
                return Ok(BOOL::from(true));
            }
        }
        Ok(BOOL::from(false))
    }
}

#[cfg(windows)]
fn get_active_modifiers() -> u32 {
    let mut mods = 0u32;
    // SAFETY: GetKeyState reads calling thread's message queue state.
    unsafe {
        if (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 {
            mods |= 0x1;
        }
        if (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 {
            mods |= 0x2;
        }
        if (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 {
            mods |= 0x4;
        }
        if (GetKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0
            || (GetKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0
        {
            mods |= 0x8;
        }
    }
    mods
}

#[cfg(windows)]
fn vk_to_unicode(vk: u32) -> u32 {
    let mut kbd_state = [0u8; 256];
    let mut chars = [0u16; 8];
    // SAFETY: pointers to fixed stack arrays.
    let count = unsafe {
        let _ = GetKeyboardState(&mut kbd_state);
        ToUnicode(vk, 0, Some(&kbd_state), &mut chars, 0)
    };
    if count == 1 {
        chars[0] as u32
    } else {
        vk
    }
}

#[cfg(windows)]
fn utf32_to_utf16(slice: &[u32]) -> Vec<u16> {
    slice
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .flat_map(|ch| {
            let mut buf = [0u16; 2];
            ch.encode_utf16(&mut buf).to_vec()
        })
        .collect()
}
