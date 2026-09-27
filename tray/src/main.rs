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
use windows::Win32::System::Threading::*;
#[cfg(windows)]
use windows::Win32::UI::Shell::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

const MUTEX_NAME: &str = r"Local\TextVNTray";
const WINDOW_CLASS_NAME: &str = "TextVNTrayWndClass";
#[cfg(windows)] // WM_APP chỉ có trong import WindowsAndMessaging (cfg-gated)
const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_ICON_UID: u32 = 100;
const IDI_ICON_V: usize = 1;
const IDI_ICON_E: usize = 2;

static RUNNING: AtomicBool = AtomicBool::new(true);

struct TrayApp {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
    menu: TrayMenu,
    icon_vi: isize,
    icon_en: isize,
}

static APP_INSTANCE: std::sync::OnceLock<TrayApp> = std::sync::OnceLock::new();

#[cfg(windows)]
fn load_app_icon(h_instance: HINSTANCE, res_id: usize, file_name: &str) -> HICON {
    unsafe {
        // 1. Thử nạp từ Win32 PE Resource (đã nhúng qua tray.rc)
        let icon_res = LoadImageW(
            Some(h_instance),
            PCWSTR(res_id as *const u16),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        );
        if let Ok(handle) = icon_res {
            let hicon = HICON(handle.0);
            if !hicon.is_invalid() {
                return hicon;
            }
        }

        // 2. Fallback: nạp từ file resources/<file_name> cạnh exe hoặc thư mục dự án
        let mut candidates = Vec::new();
        if let Ok(mut exe) = std::env::current_exe() {
            exe.pop();
            candidates.push(exe.join("resources").join(file_name));
            candidates.push(exe.join(file_name));
        }
        candidates.push(std::path::PathBuf::from("tray/resources").join(file_name));
        candidates.push(std::path::PathBuf::from("resources").join(file_name));

        for path in candidates {
            if path.exists() {
                let path_w: Vec<u16> = path
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(Some(0))
                    .collect();
                let icon_file = LoadImageW(
                    None,
                    PCWSTR(path_w.as_ptr()),
                    IMAGE_ICON,
                    0,
                    0,
                    LR_LOADFROMFILE | LR_DEFAULTSIZE,
                );
                if let Ok(handle) = icon_file {
                    let hicon = HICON(handle.0);
                    if !hicon.is_invalid() {
                        return hicon;
                    }
                }
            }
        }

        // 3. Fallback cuối cùng: default application icon
        LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
    }
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

    let mut candidates = Vec::new();
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        candidates.push(exe.join("textvn-hook.exe"));
        candidates.push(exe.join("textvn-hook.exe"));
    }
    candidates.push(std::path::PathBuf::from("textvn-hook.exe"));
    candidates.push(std::path::PathBuf::from("target/release/textvn-hook.exe"));
    candidates.push(std::path::PathBuf::from(
        "target/x86_64-pc-windows-msvc/release/textvn-hook.exe",
    ));

    for path in candidates {
        if path.exists() {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new(path)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
            break;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--autostart" => {
                // Khởi động từ Windows Startup, tiếp tục chạy ngầm vào tray
            }
            "--status" => {
                check_status();
                return;
            }
            "--stop" => {
                stop_running_instance();
                return;
            }
            "--help" | "-h" => {
                println!("TextVN - Bo go Tieng Viet chuyen nghiep");
                println!("Usage: TextVN [OPTIONS]");
                println!("Options:");
                println!("  --autostart   Khoi dong ngam tu Windows Startup (mini to tray)");
                println!("  --status      Kiem tra trang thai IPC server");
                println!("  --stop        Yeu cau dung instance dang chay");
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
fn run_tray_app() {
    // 1. Single Instance Check qua Mutex
    let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
    let mutex_handle = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };

    let mutex = match mutex_handle {
        Ok(h) => h,
        Err(_) => {
            eprintln!("TextVN is already running.");
            return;
        }
    };

    let is_autostart = std::env::args().any(|a| a == "--autostart");

    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(mutex) };
        if !is_autostart {
            // Nếu người dùng click chạy app khi đã chạy ngầm -> mở Bảng điều khiển
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

    // Tự động khởi chạy background hook engine nếu chưa có
    ensure_hook_running();

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
            Some(HWND_MESSAGE),
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

    // 4. Thêm icon vào khay hệ thống (mặc định tiếng Việt [V] Tím)
    let initial_icon = if svc.is_global_enabled() {
        icon_vi
    } else {
        icon_en
    };

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: initial_icon,
        ..Default::default()
    };

    let tip = if svc.is_global_enabled() {
        "TextVN - Tiếng Việt [V] (Tím)"
    } else {
        "TextVN - English [E] (Xanh)"
    };
    copy_to_wide_buf(&mut nid.szTip, tip);

    let _ = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };

    // Nếu người dùng khởi chạy thủ công (không phải từ --autostart),
    // hiển thị ngay Bảng điều khiển (Control Panel) trên màn hình theo đúng chuẩn UniKey / GoTiengViet
    if !is_autostart {
        textvn_tray::settings_dialog::show_settings_dialog(svc.clone(), ipc.clone());
    }

    // 5. Message Loop
    let mut msg = MSG::default();
    while RUNNING.load(Ordering::Acquire) && unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool()
    {
        let _ = unsafe { TranslateMessage(&msg) };
        let _ = unsafe { DispatchMessageW(&msg) };
    }

    // 6. Dọn dẹp trước khi thoát
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
                        app.menu.show_popup(hwnd, pt.x, pt.y, None);
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
                app.menu.handle_command(cmd_id, None);
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
                // `--stop` là lệnh quản trị tường minh. Nếu UI thread bị treo,
                // không để tray/hook bị orphan vô hạn: terminate đúng PID sở hữu
                // cửa sổ TextVN đã định danh ở trên.
                if window_pid != 0 {
                    if let Ok(process) =
                        unsafe { OpenProcess(PROCESS_TERMINATE, false, window_pid) }
                    {
                        let terminated = unsafe { TerminateProcess(process, 0) }.is_ok();
                        let _ = unsafe { CloseHandle(process) };
                        if terminated {
                            println!(
                                "TextVN did not close gracefully; terminated PID {window_pid}."
                            );
                            return;
                        }
                    }
                }
                println!("TextVN could not be stopped within 3s.");
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
