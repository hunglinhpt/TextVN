// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-hook.exe` — Process-isolated Low-Level Keyboard Hook adapter cho Windows (WIN-040..045).
//!
//! Vai trò: chế độ tương thích cho app không dùng được TSF (console cmd/pwsh, terminal cũ, game).
//! Chạy trong tiến trình độc lập, kết nối tới Tray qua IPC pipe, có watchdog heartbeat và orphan timeout (30s).
//! An toàn fail-open tuyệt đối: loop guard chống tự nuốt phím, latency timebox 2ms.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod hook_app {
    use std::cell::RefCell;
    use std::io::{Read, Write};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use textvn_appdb::AppDb;
    use textvn_ffi::{ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, IME_FIELD_BODY};
    use textvn_field_detect::SecurityState;
    use textvn_ipc::{decode_exact_frame, encode_frame, Message, MAX_FRAME_BYTES};
    use textvn_win_hook::{
        CallbackDecision, EngineOutcome, HookEngine, HookMode, HookState, KeyEvent,
    };

    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::System::Threading::*;
    use windows::Win32::UI::Accessibility::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    /// Cờ bảo vệ chống đệ quy khi chính TextVN đang gọi `SendInput` (WIN-041).
    static IN_INJECTION: AtomicBool = AtomicBool::new(false);

    /// Trạng thái phím Ctrl + Shift cho chuyển đổi chế độ gõ
    static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
    static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
    static OTHER_KEY_DOWN: AtomicBool = AtomicBool::new(false);
    static LAST_TOGGLE_MS: AtomicU64 = AtomicU64::new(0);

    static GLOBAL_ENABLED: AtomicBool = AtomicBool::new(true);
    static RELOAD_CONFIG_PENDING: AtomicBool = AtomicBool::new(false);
    static HOOK_MAIN_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    #[derive(Clone, Copy)]
    struct FocusProbe {
        generation: u64,
        field_role: u32,
        security: SecurityState,
    }

    fn load_user_config() -> Option<String> {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let p = std::path::PathBuf::from(&appdata)
                .join("TextVN")
                .join("config.json");
            if p.exists() {
                return std::fs::read_to_string(p).ok();
            }
        }
        None
    }

    fn trigger_global_toggle() {
        std::thread::spawn(|| {
            use std::io::Write;
            use textvn_ipc::{encode_frame, Message};
            if let Ok(mut stream) =
                textvn_ipc::pipe_client_options().open(r"\\.\pipe\textvn-ipc-v1")
            {
                let msg = Message::ToggleGlobal;
                if let Ok(frame) = encode_frame(&msg) {
                    let _ = stream.write_all(&frame);
                    let _ = stream.flush();
                }
            }
        });
    }

    /// Trạng thái Hook toàn cục trong thread hook của tiến trình.
    struct GlobalHookContext {
        engine: HookEngine,
        state: HookState,
        appdb: Option<AppDb>,
        hook_handle: HHOOK,
        start_time: Instant,
        current_hwnd: HWND,
        current_app_id: String,
        focus_probe_tx: Sender<FocusProbe>,
        focus_probe_rx: Receiver<FocusProbe>,
    }

    thread_local! {
        static HOOK_CTX: RefCell<Option<GlobalHookContext>> = const { RefCell::new(None) };
    }

    /// Entry point chính của `textvn-hook.exe`.
    pub fn run() -> Result<()> {
        let mutex_name_wide: Vec<u16> = r"Local\TextVNHookMutex"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mutex_handle = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };
        let mutex = match mutex_handle {
            Ok(h) => h,
            Err(_) => return Ok(()),
        };
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            let _ = unsafe { CloseHandle(mutex) };
            return Ok(());
        }

        HOOK_MAIN_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Release);

        let now = Instant::now();
        let engine = if let Some(ref json) = load_user_config() {
            HookEngine::new_with_config(json).unwrap_or_else(|_| HookEngine::new().unwrap())
        } else {
            HookEngine::new().unwrap()
        };

        // Hook is the usable fallback when the TSF TIP is not selected. The
        // pending security gate still blocks every key until native UIA has
        // classified the focused element; `Always` never bypasses that gate.
        let appdb = load_default_appdb();
        let state = HookState::new("unknown.exe", 0, HookMode::Always, Duration::ZERO);
        let (focus_probe_tx, focus_probe_rx) = mpsc::channel();

        HOOK_CTX.with(|cell| {
            *cell.borrow_mut() = Some(GlobalHookContext {
                engine,
                state,
                appdb,
                hook_handle: HHOOK::default(),
                start_time: now,
                current_hwnd: HWND::default(),
                current_app_id: String::new(),
                focus_probe_tx,
                focus_probe_rx,
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

        let _ = unsafe { CloseHandle(mutex) };

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

        // Tự động nạp lại config nếu có thông báo từ IPC hoặc file thay đổi
        if RELOAD_CONFIG_PENDING.swap(false, Ordering::AcqRel) {
            if let Some(cfg) = load_user_config() {
                HOOK_CTX.with(|cell| {
                    if let Some(ctx) = cell.borrow_mut().as_mut() {
                        let _ = ctx.engine.reload_config(&cfg);
                    }
                });
            }
        }

        let vk = kbd.vkCode;
        let is_ctrl =
            vk == VK_CONTROL.0 as u32 || vk == VK_LCONTROL.0 as u32 || vk == VK_RCONTROL.0 as u32;
        let is_shift =
            vk == VK_SHIFT.0 as u32 || vk == VK_LSHIFT.0 as u32 || vk == VK_RSHIFT.0 as u32;

        let is_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        let is_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;

        // Xử lý Hotkey chuyển đổi chế độ gõ Ctrl + Shift (chuẩn UniKey / EVKey)
        if is_down {
            if is_ctrl {
                CTRL_DOWN.store(true, Ordering::Release);
            } else if is_shift {
                SHIFT_DOWN.store(true, Ordering::Release);
            } else {
                OTHER_KEY_DOWN.store(true, Ordering::Release);
            }
        } else if is_up && (is_ctrl || is_shift) {
            let ctrl = CTRL_DOWN.load(Ordering::Acquire);
            let shift = SHIFT_DOWN.load(Ordering::Acquire);
            let other = OTHER_KEY_DOWN.load(Ordering::Acquire);

            if ctrl && shift && !other {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let last_ms = LAST_TOGGLE_MS.load(Ordering::Acquire);
                if now_ms.saturating_sub(last_ms) >= 250 {
                    LAST_TOGGLE_MS.store(now_ms, Ordering::Release);
                    trigger_global_toggle();
                }
                OTHER_KEY_DOWN.store(true, Ordering::Release);
            }

            if is_ctrl {
                CTRL_DOWN.store(false, Ordering::Release);
            }
            if is_shift {
                SHIFT_DOWN.store(false, Ordering::Release);
            }
            if !CTRL_DOWN.load(Ordering::Acquire) && !SHIFT_DOWN.load(Ordering::Acquire) {
                OTHER_KEY_DOWN.store(false, Ordering::Release);
            }
        }

        // Nếu bộ gõ đang ở chế độ Tiếng Anh (off), không can thiệp, chuyển ngay cho OS
        if !GLOBAL_ENABLED.load(Ordering::Acquire) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        // 2. Chỉ xử lý khi key down (WM_KEYDOWN hoặc WM_SYSKEYDOWN)
        if !is_down {
            return CallNextHookEx(None, code, wparam, lparam);
        }
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

            // Apply only UIA results that match the active focus generation.
            // A delayed worker result therefore cannot unlock a newer field.
            while let Ok(probe) = ctx.focus_probe_rx.try_recv() {
                let _ = ctx
                    .state
                    .publish_probe(probe.generation, probe.field_role, probe.security);
            }

            // Cập nhật foreground app nếu cửa sổ đổi
            let fg_hwnd = GetForegroundWindow();
            if fg_hwnd != ctx.current_hwnd {
                ctx.current_hwnd = fg_hwnd;
                ctx.current_app_id = get_process_name_for_window(fg_hwnd);
                let gen = ctx.state.begin_focus(&ctx.current_app_id, 0);
                let tx = ctx.focus_probe_tx.clone();
                std::thread::spawn(move || {
                    let (field_role, security) = probe_focused_element();
                    let _ = tx.send(FocusProbe {
                        generation: gen,
                        field_role,
                        security,
                    });
                });
            }

            let ch = vk_to_unicode(vk, kbd.scanCode);
            // Caps Lock: suy từ ký tự layout trả về (hoa mà không giữ Shift, hoặc ngược lại)
            // — trạng thái toggle đọc từ thread hook không đáng tin. Engine cần biết để
            // `VIEETJ` (Caps Lock) → `VIỆT` mà `USA` (Shift) vẫn là `USA`.
            let mods = mods | caps_lock_bit(ch, mods);
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
            if ctx.state.record_callback_duration(elapsed) == CallbackDecision::SelfDisabled {
                // Đã tốn quá budget liên tiếp: không inject outcome hiện hành;
                // caller forward key gốc và để watchdog/tray xử lý tiếp.
                outcome_to_inject = None;
                return;
            }

            if matches!(outcome, EngineOutcome::Transform) {
                outcome_to_inject = Some(result);
            }
        });

        // 4. Nếu engine yêu cầu thay đổi ký tự -> nuốt phím gốc (return 1) và thực hiện inject
        if let Some(res) = outcome_to_inject {
            // Chỉ nuốt key gốc sau khi SendInput xác nhận đã gửi đủ event.
            // Nếu inject bị UIPI/partial failure, forward key gốc thay vì mất text.
            if inject_engine_result(&res) {
                return LRESULT(1);
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }

    /// Bơm phím thay thế qua SendInput với loop guard an toàn (WIN-042).
    fn inject_engine_result(result: &textvn_ffi::ime_result_v1) -> bool {
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

        let sent_all = if !inputs.is_empty() {
            // SAFETY: inputs là mảng INPUT hợp lệ
            unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) as usize == inputs.len()
            }
        } else {
            false
        };

        IN_INJECTION.store(false, Ordering::Release);
        sent_all
    }

    /// `IME_MOD_CAPS` nếu chữ cái gõ ra có hoa/thường ngược với trạng thái Shift.
    fn caps_lock_bit(ch: u32, mods: u32) -> u32 {
        const MOD_SHIFT: u32 = 0x1;
        const MOD_CAPS: u32 = 0x20;
        match char::from_u32(ch) {
            Some(c) if c.is_alphabetic() && c.is_uppercase() != (mods & MOD_SHIFT != 0) => MOD_CAPS,
            _ => 0,
        }
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

    /// VK → ký tự theo layout của cửa sổ foreground. `wFlags = 0x4`: không đổi
    /// trạng thái dead-key. Phím không sinh ký tự trả 0 — bản cũ trả `vk`, biến
    /// Delete thành '.', F1 thành 'p' và làm engine nuốt/biến đổi nhầm.
    fn vk_to_unicode(vk: u32, scan: u32) -> u32 {
        const TOUNICODE_NO_STATE_CHANGE: u32 = 0x4;
        let mut kbd_state = [0u8; 256];
        let mut chars = [0u16; 8];
        // SAFETY: buffer cố định trên stack; HKL của thread sở hữu cửa sổ foreground.
        let count = unsafe {
            let _ = GetKeyboardState(&mut kbd_state);
            let fg_thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
            ToUnicodeEx(
                vk,
                scan,
                &kbd_state,
                &mut chars,
                TOUNICODE_NO_STATE_CHANGE,
                Some(GetKeyboardLayout(fg_thread)),
            )
        };
        if count == 1 && chars[0] >= 0x20 && chars[0] != 0x7F {
            u32::from(chars[0])
        } else {
            0
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

    /// Query UIA off the keyboard-hook thread. On any COM/UIA failure this
    /// returns Unknown, which keeps the hook fail-open instead of risking a
    /// password field. UIA's IsPassword property is the security authority.
    fn probe_focused_element() -> (u32, SecurityState) {
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        if !initialized {
            return (IME_FIELD_BODY, SecurityState::Unknown);
        }
        let result = (|| {
            let automation: IUIAutomation =
                unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()? };
            let element = unsafe { automation.GetFocusedElement().ok()? };
            let is_password = unsafe { element.CurrentIsPassword().ok()? }.as_bool();
            Some(if is_password {
                (IME_FIELD_BODY, SecurityState::Secure)
            } else {
                (IME_FIELD_BODY, SecurityState::NonSecure)
            })
        })();
        unsafe { CoUninitialize() };
        result.unwrap_or((IME_FIELD_BODY, SecurityState::Unknown))
    }

    fn empty_ime_result() -> textvn_ffi::ime_result_v1 {
        textvn_ffi::ime_result_v1 {
            abi_version: textvn_ffi::IME_ABI_VERSION,
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

    /// Vòng lặp IPC Heartbeat: kết nối pipe tới Tray, nhận Snapshot, StateUpdate, ConfigReload, Shutdown (WIN-040).
    fn run_ipc_heartbeat_loop(running: Arc<AtomicBool>) {
        let pid = std::process::id();
        let mut last_tray_contact = Instant::now();
        while running.load(Ordering::Acquire) {
            let stream_opt = textvn_ipc::pipe_client_options()
                .open(r"\\.\pipe\textvn-ipc-v1")
                .ok();

            match stream_opt {
                Some(mut stream) => {
                    let hello = Message::Hello {
                        pid,
                        abi: textvn_ffi::IME_ABI_VERSION,
                        version: env!("CARGO_PKG_VERSION").into(),
                    };
                    if send_message(&mut stream, &hello).is_err() {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    last_tray_contact = Instant::now();

                    let sub = Message::Subscribe { pid };
                    if send_message(&mut stream, &sub).is_err() {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }

                    while running.load(Ordering::Acquire) {
                        match read_next_message(&mut stream) {
                            Ok(msg) => match msg {
                                Message::ConfigReload { .. } => {
                                    RELOAD_CONFIG_PENDING.store(true, Ordering::Release);
                                }
                                Message::Snapshot { ref state, .. } => {
                                    last_tray_contact = Instant::now();
                                    if let Some(&enabled) = state.get("*") {
                                        GLOBAL_ENABLED.store(enabled, Ordering::Release);
                                    }
                                    RELOAD_CONFIG_PENDING.store(true, Ordering::Release);
                                }
                                Message::StateUpdate {
                                    ref app_id,
                                    enabled,
                                    ..
                                } => {
                                    last_tray_contact = Instant::now();
                                    if app_id == "*" {
                                        GLOBAL_ENABLED.store(enabled, Ordering::Release);
                                    }
                                }
                                Message::Shutdown => {
                                    running.store(false, Ordering::Release);
                                    let tid = HOOK_MAIN_THREAD_ID.load(Ordering::Acquire);
                                    if tid != 0 {
                                        unsafe {
                                            let _ = PostThreadMessageW(
                                                tid,
                                                WM_QUIT,
                                                WPARAM(0),
                                                LPARAM(0),
                                            );
                                        }
                                    }
                                    return;
                                }
                                Message::Ping => {
                                    let pong = Message::Pong { uptime_ms: 0 };
                                    let _ = send_message(&mut stream, &pong);
                                }
                                _ => {}
                            },
                            Err(_) => {
                                // Mất kết nối hoặc tray đóng pipe
                                break;
                            }
                        }
                    }
                }
                None => {
                    if last_tray_contact.elapsed() >= Duration::from_secs(30) {
                        // Không giữ low-level hook mồ côi sau khi tray đã dừng.
                        running.store(false, Ordering::Release);
                        let tid = HOOK_MAIN_THREAD_ID.load(Ordering::Acquire);
                        if tid != 0 {
                            unsafe {
                                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
                            }
                        }
                        return;
                    }
                    // Pipe chưa sẵn sàng, ngủ một lát rồi thử lại
                    for _ in 0..10 {
                        if !running.load(Ordering::Acquire) {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }
    }

    fn send_message<W: Write>(writer: &mut W, msg: &Message) -> std::io::Result<()> {
        let frame = encode_frame(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}")))?;
        writer.write_all(&frame)?;
        writer.flush()?;
        Ok(())
    }

    fn read_next_message<R: Read>(reader: &mut R) -> std::io::Result<Message> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let length = u32::from_le_bytes(len_buf) as usize;
        if length > MAX_FRAME_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Frame oversize",
            ));
        }

        let mut frame = vec![0u8; 4 + length];
        frame[..4].copy_from_slice(&len_buf);
        reader.read_exact(&mut frame[4..])?;

        decode_exact_frame(&frame)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}")))
    }
}

fn main() {
    #[cfg(windows)]
    {
        if let Err(e) = hook_app::run() {
            eprintln!("TextVN hook error: {e:?}");
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("TextVN hook chỉ hỗ trợ trên hệ điều hành Windows.");
    }
}
