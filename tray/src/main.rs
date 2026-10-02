// SPDX-License-Identifier: GPL-3.0-or-later
//! Ứng dụng khay hệ thống TextVN Tray (WIN-050 / WIN-051 / WIN-053 — P1-4 §1).
//!
//! Chạy 1 instance duy nhất với Mutex `Local\TextVNTray`.
//! Lắng nghe IPC pipe, điều phối cấu hình & trạng thái, hiển thị tray icon và menu ngữ cảnh.

// TextVN là ứng dụng tray GUI ở mọi profile. Không để `cargo run`/debug build
// tạo console thứ hai khi người dùng chỉ mở Bảng điều khiển.
#![cfg_attr(windows, windows_subsystem = "windows")]
// Tray = Windows-only: trên non-Windows item Win32 không có caller (xem lib.rs).
#![cfg_attr(not(windows), allow(dead_code))]

use std::io::{Read, Write};
use std::sync::atomic::AtomicBool;
#[cfg(windows)] // chỉ xài trong vòng lặp message Windows
use std::sync::atomic::Ordering;
use std::sync::Arc;
#[cfg(windows)]
use std::time::Duration;

// cfg(windows): load_app_icon là Win32 (HICON) — bin vẫn build trên Linux/macOS
// cho gate CI --workspace; thiếu cfg làm ubuntu CI fail E0432 (bắt được ở
// ci-shared 2026-09-30, không bắt được bằng clippy trên host Windows).
#[cfg(windows)]
use textvn_tray::icons::{load_app_icon, IDI_ICON_E, IDI_ICON_V};
use textvn_tray::ipc_server::{IpcServer, PIPE_NAME};
use textvn_tray::menu::TrayMenu;
#[cfg(windows)]
use textvn_tray::menu::ID_EXIT;
use textvn_tray::svc::SvcManager;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(windows)]
use windows::Win32::System::Registry::*;
#[cfg(windows)]
use windows::Win32::System::Threading::*;
#[cfg(windows)]
use windows::Win32::UI::Shell::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

const MUTEX_NAME: &str = r"Local\TextVNTray";
const WINDOW_CLASS_NAME: &str = "TextVNTrayWndClass";
#[cfg(windows)]
const TSF_TIP_REGISTRY_KEY: &str = r"Software\Classes\CLSID\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}";
/// Khóa TIP của CTF (tương đối HKLM\SOFTWARE / HKCU\Software) — khớp `textvn-cli register`.
#[cfg(windows)]
const TSF_CTF_TIP_KEY: &str = r"Software\Microsoft\CTF\TIP\{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}";
#[cfg(windows)]
const TSF_PROFILE_GUID: &str = "{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}";
#[cfg(windows)]
const TSF_LANGS: [u16; 2] = [0x042A, 0x0409];
#[cfg(windows)] // WM_APP chỉ có trong import WindowsAndMessaging (cfg-gated)
const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_ICON_UID: u32 = 100;

static RUNNING: AtomicBool = AtomicBool::new(true);
/// ID message `TaskbarCreated` (RegisterWindowMessageW) — Explorer broadcast khi khởi động lại.
#[cfg(windows)]
static TASKBAR_CREATED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[cfg(windows)]
const TIMER_TRAY_RETRY: usize = 1;

// ---- Ctrl+Shift tap toàn cục (UniKey semantics) --------------------------------
// LL hook CHỈ QUAN SÁT modifier: không ăn phím (mọi event đi tiếp qua
// CallNextHookEx), không inject. Tồn tại để Ctrl+Shift đổi mode ở MỌI app — kể
// cả khi TextVN TIP không phải bộ gõ active (user đang đứng ở bàn phím
// US/Microsoft Việt trong danh sách Win+Space). Trùng lặp với đường in-process
// của TIP được khoá bằng `try_claim_global_toggle()` (E11): nguồn sau trong
// 250ms bị bỏ qua, cả hai nguồn tính cùng giá trị từ cùng state nền.
#[cfg(windows)]
static HOTKEY_CTRL_DOWN: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static HOTKEY_SHIFT_DOWN: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
static HOTKEY_OTHER_KEY_DOWN: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
unsafe extern "system" fn tray_ll_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    // SAFETY: lparam trỏ KBDLLHOOKSTRUCT hợp lệ theo hợp đồng WH_KEYBOARD_LL.
    let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
    let vk = kbd.vkCode;
    let is_up = (kbd.flags.0 & LLKHF_UP.0) != 0;
    let is_down = !is_up;

    let is_ctrl = matches!(vk, 0x11 | 0xA2 | 0xA3);
    let is_shift = matches!(vk, 0x10 | 0xA0 | 0xA1);

    if is_down && is_ctrl {
        HOTKEY_CTRL_DOWN.store(true, Ordering::Release);
    } else if is_down && is_shift {
        HOTKEY_SHIFT_DOWN.store(true, Ordering::Release);
    } else if is_down {
        HOTKEY_OTHER_KEY_DOWN.store(true, Ordering::Release);
    } else if is_up && (is_ctrl || is_shift) {
        if HOTKEY_CTRL_DOWN.load(Ordering::Acquire)
            && HOTKEY_SHIFT_DOWN.load(Ordering::Acquire)
            && !HOTKEY_OTHER_KEY_DOWN.load(Ordering::Acquire)
            && textvn_tray::try_claim_global_toggle()
        {
            let raw_hwnd = textvn_tray::TRAY_HWND.load(Ordering::Acquire);
            if raw_hwnd != 0 {
                let _ = PostMessageW(
                    Some(HWND(raw_hwnd as *mut _)),
                    textvn_tray::WM_TOGGLE_HOTKEY,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
            // Nuốt LẦN NHẢ CÙNG LÚC của cặp modifier còn lại khỏi bộ đếm "phím
            // khác" — không ăn phím, mọi event vẫn đi tiếp cho app.
            HOTKEY_OTHER_KEY_DOWN.store(true, Ordering::Release);
        }

        if is_ctrl {
            HOTKEY_CTRL_DOWN.store(false, Ordering::Release);
        }
        if is_shift {
            HOTKEY_SHIFT_DOWN.store(false, Ordering::Release);
        }
        if !HOTKEY_CTRL_DOWN.load(Ordering::Acquire) && !HOTKEY_SHIFT_DOWN.load(Ordering::Acquire) {
            HOTKEY_OTHER_KEY_DOWN.store(false, Ordering::Release);
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

struct TrayApp {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
    menu: TrayMenu,
    icon_vi: isize,
    icon_en: isize,
}

static APP_INSTANCE: std::sync::OnceLock<TrayApp> = std::sync::OnceLock::new();

/// `NIM_ADD` icon khay theo trạng thái hiện tại. `false` khi shell chưa sẵn sàng.
#[cfg(windows)]
fn add_tray_icon(hwnd: HWND, app: &TrayApp) -> bool {
    let enabled = app.svc.is_global_enabled();
    let raw_icon = if enabled { app.icon_vi } else { app.icon_en };
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: HICON(raw_icon as *mut std::ffi::c_void),
        ..Default::default()
    };
    copy_to_wide_buf(
        &mut nid.szTip,
        if enabled {
            "TextVN - Tiếng Việt [V] (Tím)"
        } else {
            "TextVN - English [E] (Xanh)"
        },
    );
    unsafe { Shell_NotifyIconW(NIM_ADD, &nid) }.as_bool()
}

#[cfg(windows)]
fn update_tray_icon(hwnd: HWND, app: &TrayApp) {
    let enabled = app.svc.is_global_enabled();
    let raw_icon = if enabled { app.icon_vi } else { app.icon_en };
    let h_icon = HICON(raw_icon as *mut std::ffi::c_void);
    let tip = if enabled {
        "TextVN - Tiếng Việt [V] (Tím)"
    } else {
        "TextVN - English [E] (Xanh)"
    };

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        uFlags: NIF_ICON | NIF_TIP,
        hIcon: h_icon,
        ..Default::default()
    };
    copy_to_wide_buf(&mut nid.szTip, tip);
    let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) };
}

#[cfg(windows)]
fn ensure_hook_running() {
    let mutex_name_wide: Vec<u16> = r"Local\TextVNHookMutex"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let existing_mutex =
        unsafe { OpenMutexW(MUTEX_ALL_ACCESS, false, PCWSTR(mutex_name_wide.as_ptr())) };
    if let Ok(h) = existing_mutex {
        let _ = unsafe { CloseHandle(h) };
        return;
    }

    if let Some(path) = textvn_tray::compatibility_hook_path() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new(path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
}

/// TSF là đường gõ mặc định nên phải có TIP profile trước khi tray chạy. Bản
/// portable có thể được mở thẳng hoặc bị chuyển thư mục; khi đó registry thiếu
/// hoặc trỏ tới DLL cũ và Windows không thể nạp TIP → không gõ được tiếng Việt.
/// Mỗi lần khởi động kiểm tra lại và đăng ký per-user bằng CLI cạnh executable
/// (không tạo console, không cần quyền Administrator).
#[cfg(windows)]
fn ensure_tsf_tip_registered() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    let Some(dll) = ["textvn-tsf.dll", "textvn_win_tsf.dll"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.is_file())
    else {
        return;
    };
    if tsf_registration_is_current(&dll) {
        return;
    }
    let cli_path = dir.join("textvn-cli.exe");
    if !cli_path.is_file() {
        return;
    }

    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let status = std::process::Command::new(cli_path)
        .arg("register")
        .arg("--dll")
        .arg(&dll)
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if !status.is_ok_and(|s| s.success()) {
        // Không hiện hộp thoại khi autostart (tránh làm phiền mỗi lần đăng nhập),
        // nhưng lần mở Settings người dùng có nút "Cài & bật TSF" với lỗi rõ ràng.
        eprintln!("TextVN: TSF registration did not complete; open Settings to repair it.");
    }
}

/// COM server trỏ đúng DLL đang có + đầy đủ profile VI/EN. Một CTF root rỗng
/// từng khiến tray bỏ qua sửa chữa dù TextVN chưa thể gõ được.
#[cfg(windows)]
fn tsf_registration_is_current(dll: &std::path::Path) -> bool {
    let inproc = format!(r"{TSF_TIP_REGISTRY_KEY}\InprocServer32");
    let server = read_registry_string(HKEY_CURRENT_USER, &inproc)
        .or_else(|| read_registry_string(HKEY_LOCAL_MACHINE, &inproc));
    let server_ok = server.is_some_and(|p| {
        p.eq_ignore_ascii_case(&dll.to_string_lossy()) && std::path::Path::new(&p).is_file()
    });
    server_ok && TSF_LANGS.iter().all(|lang| tsf_profile_is_current(*lang))
}

#[cfg(windows)]
fn tsf_profile_is_current(lang: u16) -> bool {
    let key = format!(r"{TSF_CTF_TIP_KEY}\LanguageProfile\0x{lang:08x}\{TSF_PROFILE_GUID}");
    let user_ok = registry_key_exists(HKEY_CURRENT_USER, &key)
        && read_registry_dword(HKEY_CURRENT_USER, &key, "Enable").is_some_and(|v| v != 0);
    user_ok || registry_key_exists(HKEY_LOCAL_MACHINE, &key)
}

#[cfg(windows)]
fn registry_key_exists(root: HKEY, path: &str) -> bool {
    let key_wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let mut key = HKEY::default();
    // SAFETY: buffer nul-terminated; key đóng ngay khi mở được.
    unsafe {
        if RegOpenKeyExW(root, PCWSTR(key_wide.as_ptr()), None, KEY_READ, &mut key) == ERROR_SUCCESS
        {
            let _ = RegCloseKey(key);
            true
        } else {
            false
        }
    }
}

#[cfg(windows)]
fn read_registry_string(root: HKEY, path: &str) -> Option<String> {
    let key_wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let mut buf = [0u16; 1024];
    let mut size = std::mem::size_of_val(&buf) as u32;
    // SAFETY: buffer/kích thước khớp nhau; RRF_RT_REG_SZ bảo đảm chuỗi kết thúc nul.
    let status = unsafe {
        RegGetValueW(
            root,
            PCWSTR(key_wide.as_ptr()),
            PCWSTR::null(),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut _),
            Some(&mut size),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}

#[cfg(windows)]
fn read_registry_dword(root: HKEY, path: &str, name: &str) -> Option<u32> {
    let key_wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mut value = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: key/name NUL-terminated; `value` đúng kích thước REG_DWORD.
    let status = unsafe {
        RegGetValueW(
            root,
            PCWSTR(key_wide.as_ptr()),
            PCWSTR(name_wide.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    (status == ERROR_SUCCESS && size == std::mem::size_of::<u32>() as u32).then_some(value)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--autostart" => {
                // Khởi động từ OS Startup, tiếp tục chạy ngầm vào tray
            }
            "--settings" => {
                // Mở Bảng điều khiển cài đặt
            }
            "--status" => {
                check_status();
                return;
            }
            "--stop" => {
                stop_running_instance();
                return;
            }
            "--free-ctrl-shift" => {
                // Dành Ctrl + Shift cho TextVN: gỡ phím tắt đổi bố cục/ngôn ngữ của Windows
                // (installer gọi khi người dùng chọn; `hotkey.rs`).
                std::process::exit(free_ctrl_shift_cli());
            }
            "--help" | "-h" => {
                println!("TextVN - Bo go Tieng Viet chuyen nghiep");
                println!("Usage: TextVN [OPTIONS]");
                println!("Options:");
                println!("  --autostart   Khoi dong ngam tu OS Startup (mini to tray)");
                println!("  --settings    Mo Bang dieu khien cai dat");
                println!("  --status      Kiem tra trang thai IPC server");
                println!("  --stop        Yeu cau dung instance dang chay");
                println!("  --free-ctrl-shift  Danh Ctrl+Shift cho TextVN (go phim tat doi ban phim cua Windows)");
                println!("  --help        Hien thi tro giup");
                return;
            }
            _ => {}
        }
    }

    #[cfg(windows)]
    run_tray_app();

    #[cfg(not(windows))]
    println!("TextVN chi ho tro he dieu hanh Windows.");
}

#[cfg(windows)]
fn free_ctrl_shift_cli() -> i32 {
    match textvn_tray::hotkey::free_ctrl_shift() {
        Ok(()) => {
            println!("Ctrl+Shift: danh cho TextVN");
            0
        }
        Err(e) => {
            eprintln!("Ctrl+Shift: {e}");
            1
        }
    }
}

#[cfg(not(windows))]
fn free_ctrl_shift_cli() -> i32 {
    0
}

#[cfg(windows)]
fn run_tray_app() {
    // 1. Single Instance Check qua Mutex
    let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
    let mutex_handle = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };
    // GetLastError phải chụp NGAY sau CreateMutexW — lời gọi khác (alloc, match)
    // có thể đè last-error làm mất cờ đã-chạy (review R3 minor 4).
    let mutex_error = unsafe { GetLastError() };

    let mutex = match mutex_handle {
        Ok(h) => h,
        Err(_) => {
            eprintln!("TextVN is already running.");
            return;
        }
    };

    let is_autostart = std::env::args().any(|a| a == "--autostart");
    let is_settings = std::env::args().any(|a| a == "--settings");

    if mutex_error == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(mutex) };
        if !is_autostart {
            // Nếu người dùng click chạy app hoặc --settings khi đã chạy ngầm -> mở Bảng điều khiển
            let class_name_wide: Vec<u16> =
                WINDOW_CLASS_NAME.encode_utf16().chain(Some(0)).collect();
            let existing_hwnd =
                unsafe { FindWindowW(PCWSTR(class_name_wide.as_ptr()), PCWSTR::null()) };
            if let Ok(h) = existing_hwnd {
                if !h.is_invalid() {
                    unsafe {
                        let _ = PostMessageW(
                            Some(h),
                            textvn_tray::WM_OPEN_SETTINGS,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }
            }
        }
        return;
    }

    // 2. Khởi tạo Service Manager & IPC Server
    let svc = SvcManager::new(None);
    let ipc = IpcServer::new(svc.clone());
    let menu = TrayMenu::new(svc.clone(), ipc.clone());

    // Khởi động background Named Pipe loop
    ipc.start();

    // Build/release smoke must prove tray IPC lifecycle without rewriting the
    // developer's active TSF registration to a transient target\release DLL.
    // Normal user launches still repair a missing or stale per-user registration.
    if std::env::var_os("TEXTVN_SKIP_TSF_REGISTRATION").is_none() {
        ensure_tsf_tip_registered();
    }

    // Giải phóng phím tắt Ctrl+Shift khỏi Windows Layout Hotkey để TextVN sử dụng
    let _ = textvn_tray::hotkey::free_ctrl_shift();

    // TSF là đường gõ chuẩn mặc định. Toggle Ctrl+Shift xử lý IN-PROCESS trong
    // TIP (ModifierToggle + KeyTraceSink, compose.rs) — tray chỉ nhận kết quả
    // qua IPC để đổi icon. KHÔNG cài WH_KEYBOARD_LL trong tray: một lần bấm
    // từng bị toggle ĐÔI (TSF + hook cùng bắn → "bấm không đổi mode", 2026-10-01)
    // và chính sách AV (docs/specs/antivirus-false-positive.md A2/A3) chỉ cho
    // hook LL trong gói compatibility opt-in.

    // 3. Đăng ký Win32 Window Class & Tạo Hidden Message Window
    let class_name_wide: Vec<u16> = WINDOW_CLASS_NAME.encode_utf16().chain(Some(0)).collect();
    let h_instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: h_instance.into(),
        lpszClassName: PCWSTR(class_name_wide.as_ptr()),
        ..Default::default()
    };

    let _ = unsafe { RegisterClassW(&wc) };

    let hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR(class_name_wide.as_ptr()),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            // Cửa sổ top-level ẨN (không bao giờ ShowWindow), KHÔNG phải message-only:
            // message-only không nhận broadcast `TaskbarCreated` (icon khay mất vĩnh viễn
            // khi Explorer khởi động lại — bug OpenKey #288/#307) và FindWindowW không tìm
            // thấy nó (mở TextVN lần 2 không bật được Bảng điều khiển).
            None,
            None,
            Some(h_instance.into()),
            None,
        )
    } {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to create message-only window: {e}");
            let _ = unsafe { CloseHandle(mutex) };
            return;
        }
    };

    textvn_tray::TRAY_HWND.store(hwnd.0 as isize, Ordering::Release);

    // Nạp icon chế độ: 'V' (Tím) cho tiếng Việt, 'E' (Xanh) cho tiếng Anh
    let icon_vi = load_app_icon(h_instance.into(), IDI_ICON_V, "textvn_v.ico");
    let icon_en = load_app_icon(h_instance.into(), IDI_ICON_E, "textvn_e.ico");

    let _ = APP_INSTANCE.set(TrayApp {
        svc: svc.clone(),
        ipc: ipc.clone(),
        menu,
        icon_vi: icon_vi.0 as isize,
        icon_en: icon_en.0 as isize,
    });

    // 4. Thêm icon vào khay hệ thống. Khi tự khởi động cùng Windows, Explorer có thể
    // chưa sẵn sàng: thử lại theo timer thay vì bỏ cuộc/crash (OpenKey #273/#308).
    TASKBAR_CREATED.store(
        unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) },
        Ordering::Release,
    );
    if let Some(app) = APP_INSTANCE.get() {
        if !add_tray_icon(hwnd, app) {
            let _ = unsafe { SetTimer(Some(hwnd), TIMER_TRAY_RETRY, 2000, None) };
        }
    }

    // App foreground gần nhất cho mục menu "Bật tiếng Việt cho {app}" (review R3
    // minor 6). Giữ guard tới hết hàm: drop → UnhookWinEvent.
    let _foreground_tracker = textvn_tray::foreground::ForegroundTracker::start();

    // Xử lý mở hộp thoại Bảng điều khiển:
    // - Khi có cờ --autostart: Khởi động chế độ chạy ngầm minimized to tray (không bật popup hộp thoại).
    // - Khi khởi động bình thường (không có --autostart): Kiểm tra cấu hình show_dialog_on_startup,
    //   nếu true hoặc có cờ --settings thì hiển thị bảng điều khiển, nếu false thì thu về khay.
    if !is_autostart && (is_settings || svc.config().show_dialog_on_startup) {
        textvn_tray::settings_dialog::show_settings_dialog(svc.clone(), ipc.clone());
    }

    // 5. Message Loop
    // LL hook chỉ quan sát Ctrl+Shift tap (không ăn phím, không inject) — cài
    // trước vòng lặp, gỡ trong dọn dẹp. Thất bại (hiếm) → Ctrl+Shift vẫn hoạt
    // động qua đường TIP in-process khi TextVN là bộ gõ active.
    let ll_hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(tray_ll_keyboard_proc), None, 0) };
    if let Err(e) = &ll_hook {
        eprintln!("TextVN: WH_KEYBOARD_LL hotkey detector unavailable: {e}");
    }

    // GetMessageW trả -1 khi lỗi — `.as_bool()` vẫn true → loop dispatch MSG
    // rác vô hạn (review R3 minor 5). Chuẩn: r <= 0 (−1 lỗi, 0 WM_QUIT) thoát.
    let mut msg = MSG::default();
    loop {
        if !RUNNING.load(Ordering::Acquire) {
            break;
        }
        let r = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if r.0 <= 0 {
            break;
        }
        // Tab/Esc/Enter trong bảng điều khiển và cửa sổ Gõ tắt.
        if textvn_tray::settings_dialog::pre_translate_message(&msg) {
            continue;
        }
        let _ = unsafe { TranslateMessage(&msg) };
        let _ = unsafe { DispatchMessageW(&msg) };
    }

    // 6. Dọn dẹp trước khi thoát
    if let Ok(h) = ll_hook {
        let _ = unsafe { UnhookWindowsHookEx(h) };
    }
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        ..Default::default()
    };
    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
    textvn_tray::TRAY_HWND.store(0, Ordering::Release);
    ipc.stop();
    let _ = unsafe { CloseHandle(mutex) };
}

#[cfg(windows)]
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = (lparam.0 & 0xffff) as u32;
            match event {
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    if let Some(app) = APP_INSTANCE.get() {
                        let current_app = textvn_tray::foreground::last_app();
                        app.menu
                            .show_popup(hwnd, pt.x, pt.y, current_app.as_deref());
                    }
                }
                WM_LBUTTONUP => {
                    // Click chuột trái: Bật/Tắt nhanh tiếng Việt và đổi icon V (Tím) <-> E (Xanh)
                    if let Some(app) = APP_INSTANCE.get() {
                        let (enabled, ver) = app.svc.toggle_global_enabled();
                        app.ipc.broadcast_state_update("*", enabled, ver);
                        update_tray_icon(hwnd, app);
                    }
                }
                WM_LBUTTONDBLCLK => {
                    // Double-click chuột trái: Mở Bảng điều khiển (chuẩn UniKey/EVKey)
                    if let Some(app) = APP_INSTANCE.get() {
                        textvn_tray::settings_dialog::show_settings_dialog(
                            app.svc.clone(),
                            app.ipc.clone(),
                        );
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        textvn_tray::WM_UPDATE_TRAY_STATE => {
            if let Some(app) = APP_INSTANCE.get() {
                update_tray_icon(hwnd, app);
            }
            // Ctrl+Shift / menu khay / IPC đổi trạng thái → bảng điều khiển đang mở cập nhật theo.
            textvn_tray::settings_dialog::refresh_if_open();
            LRESULT(0)
        }
        textvn_tray::WM_TOGGLE_HOTKEY => {
            // Nguồn LL hook — đã claim debounce trong tray_ll_keyboard_proc.
            if let Some(app) = APP_INSTANCE.get() {
                let (enabled, ver) = app.svc.toggle_global_enabled();
                app.ipc.broadcast_state_update("*", enabled, ver);
                update_tray_icon(hwnd, app);
            }
            textvn_tray::settings_dialog::refresh_if_open();
            LRESULT(0)
        }
        textvn_tray::WM_OPEN_SETTINGS => {
            if let Some(app) = APP_INSTANCE.get() {
                textvn_tray::settings_dialog::show_settings_dialog(
                    app.svc.clone(),
                    app.ipc.clone(),
                );
            }
            LRESULT(0)
        }
        textvn_tray::WM_START_COMPATIBILITY_HOOK => {
            ensure_hook_running();
            LRESULT(0)
        }
        textvn_tray::WM_REQUEST_EXIT => {
            if let Some(app) = APP_INSTANCE.get() {
                app.ipc.broadcast_shutdown();
            }
            RUNNING.store(false, Ordering::Release);
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd_id = (wparam.0 & 0xffff) as u32;
            if cmd_id == ID_EXIT {
                if let Some(app) = APP_INSTANCE.get() {
                    app.ipc.broadcast_shutdown();
                }
                RUNNING.store(false, Ordering::Release);
                PostQuitMessage(0);
                return LRESULT(0);
            }
            if let Some(app) = APP_INSTANCE.get() {
                let current_app = textvn_tray::foreground::last_app();
                app.menu.handle_command(cmd_id, current_app.as_deref());
                update_tray_icon(hwnd, app);
            }
            LRESULT(0)
        }
        WM_DESTROY | WM_CLOSE => {
            if let Some(app) = APP_INSTANCE.get() {
                app.ipc.broadcast_shutdown();
            }
            RUNNING.store(false, Ordering::Release);
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_TRAY_RETRY => {
            if let Some(app) = APP_INSTANCE.get() {
                if add_tray_icon(hwnd, app) {
                    let _ = KillTimer(Some(hwnd), TIMER_TRAY_RETRY);
                }
            }
            LRESULT(0)
        }
        m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Acquire) => {
            // Explorer vừa khởi động lại: icon cũ đã mất, thêm lại.
            if let Some(app) = APP_INSTANCE.get() {
                if !add_tray_icon(hwnd, app) {
                    let _ = SetTimer(Some(hwnd), TIMER_TRAY_RETRY, 2000, None);
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
fn copy_to_wide_buf(buf: &mut [u16], s: &str) {
    let wide: Vec<u16> = s.encode_utf16().collect();
    let len = wide.len().min(buf.len() - 1);
    buf[..len].copy_from_slice(&wide[..len]);
    buf[len] = 0;
}

fn check_status() {
    println!("Checking TextVN IPC pipe: {}", PIPE_NAME);
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(PIPE_NAME)
    {
        Ok(mut stream) => {
            let ping = textvn_ipc::Message::Ping;
            if let Ok(frame) = textvn_ipc::encode_frame(&ping) {
                if stream.write_all(&frame).is_ok() && stream.flush().is_ok() {
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).is_ok() {
                        let len = u32::from_le_bytes(len_buf) as usize;
                        if len <= textvn_ipc::MAX_FRAME_BYTES {
                            let mut buf = vec![0u8; 4 + len];
                            buf[..4].copy_from_slice(&len_buf);
                            if stream.read_exact(&mut buf[4..]).is_ok() {
                                if let Ok(textvn_ipc::Message::Pong { uptime_ms }) =
                                    textvn_ipc::decode_exact_frame(&buf)
                                {
                                    println!("TextVN IPC Server: RUNNING");
                                    println!("  Pipe: {}", PIPE_NAME);
                                    println!(
                                        "  Uptime: {} ms ({:.1}s)",
                                        uptime_ms,
                                        uptime_ms as f64 / 1000.0
                                    );
                                    return;
                                }
                            }
                        }
                    }
                }
            }
            println!("TextVN IPC Server: Connected, but response was invalid.");
        }
        Err(e) => {
            println!("TextVN IPC Server: OFFLINE ({e})");
        }
    }
}

fn stop_running_instance() {
    #[cfg(windows)]
    {
        println!("Checking for running TextVN Tray instance...");
        let class_name_wide: Vec<u16> = WINDOW_CLASS_NAME.encode_utf16().chain(Some(0)).collect();
        let hwnd = unsafe { FindWindowW(PCWSTR(class_name_wide.as_ptr()), None) };
        if let Ok(h) = hwnd {
            if !h.0.is_null() {
                println!("Found TextVN window. Requesting a graceful close...");
                // The named-pipe request is the primary control path. It enters
                // the app's own IPC worker, then requests exit on the tray thread
                // (tray broadcast Shutdown cho Hook rồi mới PostQuitMessage).
                if let Ok(mut stream) = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(PIPE_NAME)
                {
                    if let Ok(frame) = textvn_ipc::encode_frame(&textvn_ipc::Message::Shutdown) {
                        let _ = stream.write_all(&frame);
                        let _ = stream.flush();
                    }
                }
                let mut window_pid = 0u32;
                // GetWindowThreadProcessId trả thread ID qua return value; tham số
                // out là PID. Trộn hai giá trị này gửi WM_QUIT vào PID thay vì
                // queue UI, khiến `--stop` luôn phải rơi xuống TerminateProcess.
                let thread_id = unsafe { GetWindowThreadProcessId(h, Some(&mut window_pid)) };
                if thread_id == 0 {
                    println!("TextVN window owner thread could not be resolved.");
                    return;
                }

                let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
                // Đợi tiến trình giải phóng mutex; hết `iterations` mà vẫn giữ → false.
                let wait_released = |iterations: u32| -> bool {
                    for i in 0..iterations {
                        std::thread::sleep(Duration::from_millis(100));
                        let h_mutex =
                            unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };
                        if let Ok(m) = h_mutex {
                            let err = unsafe { GetLastError() };
                            let _ = unsafe { CloseHandle(m) };
                            if err != ERROR_ALREADY_EXISTS {
                                println!(
                                    "TextVN stopped successfully (after {}ms).",
                                    (i + 1) * 100
                                );
                                return true;
                            }
                        }
                    }
                    false
                };

                // Ưu tiên graceful: đợi tray tự xử lý WM_REQUEST_EXIT — broadcast
                // Shutdown cho Hook phải chạy trước khi message loop rời đi. Gửi
                // WM_QUIT ngay sau frame sẽ đua: quit tới trước khi IPC worker kịp
                // đọc frame → không broadcast → Hook mồ côi tới heartbeat timeout.
                if wait_released(20) {
                    return;
                }

                unsafe {
                    // Command-line control path: khi graceful treo, đặt WM_QUIT
                    // trực tiếp vào queue của owner thread để message loop rời đi
                    // và chạy cleanup nội bộ. Message-only windows không được
                    // desktop routing xử lý đáng tin cậy qua WM_CLOSE.
                    if let Err(error) = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0))
                    {
                        println!(
                            "WM_QUIT could not be queued for tray thread {thread_id}: {error}"
                        );
                        return;
                    }
                }
                if wait_released(10) {
                    return;
                }
                // Không TerminateProcess: một exe mở handle PROCESS_TERMINATE tới
                // process khác là mẫu hành vi AV soi (process killer), và dừng
                // cưỡng bức bỏ lỡ cleanup (icon khay, broadcast Shutdown).
                println!(
                    "TextVN (PID {window_pid}) did not stop within 3s; close it from the tray menu."
                );
                return;
            }
        }
        println!("No running TextVN instance detected.");
    }
    #[cfg(not(windows))]
    println!("Stopping tray instance is only supported on Windows.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_valid() {
        assert_eq!(MUTEX_NAME, r"Local\TextVNTray");
        assert_eq!(WINDOW_CLASS_NAME, "TextVNTrayWndClass");
        #[cfg(windows)]
        const {
            assert!(WM_TRAYICON >= WM_APP)
        };
    }
}
