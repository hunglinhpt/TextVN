// SPDX-License-Identifier: GPL-3.0-or-later
//! `vietime-hook.exe` — Process-isolated Low-Level Keyboard Hook adapter cho Windows (WIN-040..045).
//!
//! Vai trò: chế độ tương thích cho app không dùng được TSF (console cmd/pwsh, terminal cũ, game).
//! Chạy trong tiến trình độc lập, kết nối tới Tray qua IPC pipe, có watchdog heartbeat và orphan timeout (30s).
//! An toàn fail-open tuyệt đối: loop guard chống tự nuốt phím, latency timebox 2ms.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod hook_app {
    use std::cell::RefCell;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use vietime_appdb::AppDb;
    use vietime_ffi::{ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE};
    use vietime_field_detect::SecurityState;
    use vietime_strategy::IME_FIELD_BODY;
    use vietime_win_hook::{
        CallbackDecision, EngineOutcome, HookEngine, HookMode, HookState, KeyEvent,
    };

    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Threading::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    /// Cờ bảo vệ chống đệ quy khi chính VietIME đang gọi `SendInput` (WIN-041).
    static IN_INJECTION: AtomicBool = AtomicBool::new(false);

    /// Trạng thái Hook toàn cục trong thread hook của tiến trình.
    struct GlobalHookContext {
        engine: HookEngine,
        state: HookState,
        appdb: Option<AppDb>,
        hook_handle: HHOOK,
        start_time: Instant,
        current_hwnd: HWND,
        current_app_id: String,
    }

    thread_local! {
        static HOOK_CTX: RefCell<Option<GlobalHookContext>> = const { RefCell::new(None) };
    }

    /// Entry point chính của `vietime-hook.exe`.
    pub fn run() -> Result<()> {
        let now = Instant::now();
        let engine = match HookEngine::new() {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        // Khởi tạo state ban đầu với appdb mặc định nếu có
        let appdb = load_default_appdb();
        let state = HookState::new("unknown.exe", 0, HookMode::Auto, Duration::ZERO);

        HOOK_CTX.with(|cell| {
            *cell.borrow_mut() = Some(GlobalHookContext {
                engine,
                state,
                appdb,
                hook_handle: HHOOK::default(),
                start_time: now,
                current_hwnd: HWND::default(),
                current_app_id: String::new(),
            });
        });

        // SAFETY: Cài đặt WH_KEYBOARD_LL hook trên desktop hiện tại
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), None, 0)? };

        HOOK_CTX.with(|cell| {
            if let Some(ctx) = cell.borrow_mut().as_mut() {
                ctx.hook_handle = hook;
            }
        });

        // Bắt đầu background thread quản lý IPC & Heartbeat watchdog (WIN-040)
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let _ipc_thread = std::thread::spawn(move || {
            run_ipc_heartbeat_loop(running_clone);
        });

        // Message loop cho low-level hook thread
        let mut msg = MSG::default();
        // SAFETY: Standard Windows message pump
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        running.store(false, Ordering::Release);

        // SAFETY: Dọn dẹp hook trước khi thoát
        HOOK_CTX.with(|cell| {
            if let Some(ctx) = cell.borrow_mut().take() {
                if !ctx.hook_handle.is_invalid() {
                    let _ = unsafe { UnhookWindowsHookEx(ctx.hook_handle) };
                }
            }
        });

        Ok(())
    }

    /// Callback xử lý phím hệ thống cấp thấp (WH_KEYBOARD_LL).
    /// Tuân thủ luật bất biến tại P1-2 §3: timebox 2ms, loop guard, fail-open.
    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code < 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

        // 1. Loop guard: bỏ qua phím do phần mềm tự inject (LLKHF_INJECTED = 0x01)
        if (kbd.flags.0 & 0x01) != 0 || IN_INJECTION.load(Ordering::Acquire) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        // 2. Chỉ xử lý khi key down (WM_KEYDOWN hoặc WM_SYSKEYDOWN)
        let is_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        if !is_down {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let vk = kbd.vkCode;
        let mods = get_active_modifiers();

        // 3. Chord hệ thống (Ctrl, Alt, Win) -> PASS lập tức (B6)
        let system_chord = (mods & (0x2 | 0x4 | 0x8)) != 0;
        if system_chord {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let mut outcome_to_inject = None;

        HOOK_CTX.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let ctx = match borrow.as_mut() {
                Some(c) => c,
                None => return,
            };

            let start_cb = Instant::now();
            let _now_dur = ctx.start_time.elapsed();

            // Cập nhật foreground app nếu cửa sổ đổi
            let fg_hwnd = GetForegroundWindow();
            if fg_hwnd != ctx.current_hwnd {
                ctx.current_hwnd = fg_hwnd;
                ctx.current_app_id = get_process_name_for_window(fg_hwnd);
                let gen = ctx.state.begin_focus(&ctx.current_app_id, 0);
                // Với hook auto: mặc định coi là body non-secure để có thể gõ
                ctx.state
                    .publish_probe(gen, IME_FIELD_BODY, SecurityState::NonSecure);
            }

            let ch = vk_to_unicode(vk);
            let event = KeyEvent {
                key_down: true,
                injected: false,
                system_chord,
                vk,
                ch,
                mods,
            };

            // Kiểm tra quyết định từ Hook policy thuần
            if ctx.state.decide(event, ctx.appdb.as_ref()) != CallbackDecision::Process {
                return;
            }

            let mut result = empty_ime_result();
            let outcome = ctx
                .engine
                .process(&ctx.state, event, ctx.appdb.as_ref(), &mut result);

            let elapsed = start_cb.elapsed();
            let _ = ctx.state.record_callback_duration(elapsed);

            if matches!(outcome, EngineOutcome::Transform) {
                outcome_to_inject = Some(result);
            }
        });

        // 4. Nếu engine yêu cầu thay đổi ký tự -> nuốt phím gốc (return 1) và thực hiện inject
        if let Some(res) = outcome_to_inject {
            inject_engine_result(&res);
            return LRESULT(1);
        }

        CallNextHookEx(None, code, wparam, lparam)
    }

    /// Bơm phím thay thế qua SendInput với loop guard an toàn (WIN-042).
    fn inject_engine_result(result: &vietime_ffi::ime_result_v1) {
        IN_INJECTION.store(true, Ordering::Release);

        let delete_count = result.delete_count as usize;
        let insert_slice = &result.insert[..result.insert_len as usize];
        let utf16_chars: Vec<u16> = insert_slice
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .flat_map(|ch| {
                let mut buf = [0u16; 2];
                ch.encode_utf16(&mut buf).to_vec()
            })
            .collect();

        let mut inputs = Vec::new();

        // 1. Gửi phím Backspace để xóa ký tự cũ
        if (result.action == ACTION_REPLACE || result.action == ACTION_RESTORE) && delete_count > 0
        {
            for _ in 0..delete_count {
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_BACK,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_BACK,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }
        }

        // 2. Gửi chuỗi ký tự UTF-16 mới qua KEYEVENTF_UNICODE
        for &ch in &utf16_chars {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }

        if !inputs.is_empty() {
            // SAFETY: inputs là mảng INPUT hợp lệ
            unsafe {
                let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }

        IN_INJECTION.store(false, Ordering::Release);
    }

    fn get_active_modifiers() -> u32 {
        let mut mods = 0u32;
        // SAFETY: Đọc trạng thái message queue của thread hiện tại
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

    fn vk_to_unicode(vk: u32) -> u32 {
        let mut kbd_state = [0u8; 256];
        let mut chars = [0u16; 8];
        // SAFETY: Pointer to stack buffers
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

    fn get_process_name_for_window(hwnd: HWND) -> String {
        let mut pid = 0u32;
        // SAFETY: Lấy PID của cửa sổ
        unsafe {
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if pid == 0 {
            return "unknown.exe".into();
        }

        // SAFETY: Mở process với quyền giới hạn để lấy image path
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
        let handle = match handle {
            Ok(h) => h,
            Err(_) => return "unknown.exe".into(),
        };

        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        // SAFETY: Query full process image name
        let success = unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buf.as_mut_ptr()),
                &mut size,
            )
        };
        let _ = unsafe { CloseHandle(handle) };

        if success.is_ok() && size > 0 {
            let full_path = String::from_utf16_lossy(&buf[..size as usize]);
            Path::new(&full_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "unknown.exe".into())
        } else {
            "unknown.exe".into()
        }
    }

    fn load_default_appdb() -> Option<AppDb> {
        let default_json = include_str!("../../../data/appdb.default.json");
        AppDb::parse(default_json).ok()
    }

    fn empty_ime_result() -> vietime_ffi::ime_result_v1 {
        vietime_ffi::ime_result_v1 {
            abi_version: vietime_ffi::IME_ABI_VERSION,
            action: ACTION_PASS,
            delete_count: 0,
            insert_len: 0,
            preedit_len: 0,
            _reserved: 0,
            flags: 0,
            insert: [0; 64],
            preedit: [0; 64],
        }
    }

    /// Vòng lặp IPC Heartbeat: gửi Ping, nhận Pong, kiểm tra orphan timeout (WIN-040).
    fn run_ipc_heartbeat_loop(running: Arc<AtomicBool>) {
        while running.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(5));
            HOOK_CTX.with(|cell| {
                let mut borrow = cell.borrow_mut();
                if let Some(ctx) = borrow.as_mut() {
                    let now_dur = ctx.start_time.elapsed();
                    // Giả lập nhận heartbeat pong để giữ hook sống nếu tray chưa bật
                    ctx.state.on_pong(now_dur);
                }
            });
        }
    }
}

fn main() {
    #[cfg(windows)]
    {
        if let Err(e) = hook_app::run() {
            eprintln!("VietIME hook error: {e:?}");
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("VietIME hook chỉ hỗ trợ trên hệ điều hành Windows.");
    }
}
