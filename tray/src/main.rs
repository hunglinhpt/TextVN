// SPDX-License-Identifier: GPL-3.0-or-later
//! Ứng dụng khay hệ thống VietIME Tray (WIN-050 / WIN-051 / WIN-053 — P1-4 §1).
//!
//! Chạy 1 instance duy nhất với Mutex `Local\VietIMETray`.
//! Lắng nghe IPC pipe, điều phối cấu hình & trạng thái, hiển thị tray icon và menu ngữ cảnh.

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use vietime_tray::ipc_server::{IpcServer, PIPE_NAME};
use vietime_tray::menu::{TrayMenu, ID_EXIT};
use vietime_tray::svc::SvcManager;

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

const MUTEX_NAME: &str = r"Local\VietIMETray";
const WINDOW_CLASS_NAME: &str = "VietIMETrayWndClass";
const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_ICON_UID: u32 = 100;

static RUNNING: AtomicBool = AtomicBool::new(true);

struct TrayApp {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
    menu: TrayMenu,
}

static APP_INSTANCE: std::sync::OnceLock<TrayApp> = std::sync::OnceLock::new();

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--autostart" => {
                // Khởi động từ Windows Startup, tiếp tục chạy bình thường
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
                println!("VietIME Tray - Khay he thong va IPC Server cho VietIME");
                println!("Usage: vietime-tray [OPTIONS]");
                println!("Options:");
                println!("  --autostart   Khoi dong ngam tu Windows Startup");
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
    println!("VietIME Tray chi ho tro he dieu hanh Windows.");
}

#[cfg(windows)]
fn run_tray_app() {
    // 1. Single Instance Check qua Mutex
    let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
    let mutex_handle = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };

    let mutex = match mutex_handle {
        Ok(h) => h,
        Err(_) => {
            eprintln!("VietIME Tray is already running.");
            return;
        }
    };

    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        println!("VietIME Tray already running in background.");
        let _ = unsafe { CloseHandle(mutex) };
        return;
    }

    // 2. Khởi tạo Service Manager & IPC Server
    let svc = SvcManager::new(None);
    let ipc = IpcServer::new(svc.clone());
    let menu = TrayMenu::new(svc.clone(), ipc.clone());

    // Khởi động background Named Pipe loop
    ipc.start();

    let _ = APP_INSTANCE.set(TrayApp {
        svc: svc.clone(),
        ipc: ipc.clone(),
        menu,
    });

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

    // 4. Thêm icon vào khay hệ thống
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_UID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() },
        ..Default::default()
    };

    let tip = if svc.is_global_enabled() {
        "VietIME - Tiếng Việt (Bật)"
    } else {
        "VietIME - Tiếng Việt (Tắt)"
    };
    copy_to_wide_buf(&mut nid.szTip, tip);

    let _ = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };

    // 5. Message Loop
    let mut msg = MSG::default();
    while RUNNING.load(Ordering::Acquire) && unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool()
    {
        let _ = unsafe { TranslateMessage(&msg) };
        let _ = unsafe { DispatchMessageW(&msg) };
    }

    // 6. Dọn dẹp trước khi thoát
    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
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
                    // Click chuột trái: Bật/Tắt nhanh tiếng Việt
                    if let Some(app) = APP_INSTANCE.get() {
                        let (enabled, ver) = app.svc.toggle_global_enabled();
                        app.ipc.broadcast_state_update("*", enabled, ver);

                        // Cập nhật tooltip
                        let mut nid = NOTIFYICONDATAW {
                            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                            hWnd: hwnd,
                            uID: TRAY_ICON_UID,
                            uFlags: NIF_TIP,
                            ..Default::default()
                        };
                        let tip = if enabled {
                            "VietIME - Tiếng Việt (Bật)"
                        } else {
                            "VietIME - Tiếng Việt (Tắt)"
                        };
                        copy_to_wide_buf(&mut nid.szTip, tip);
                        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd_id = (wparam.0 & 0xffff) as u32;
            if cmd_id == ID_EXIT {
                RUNNING.store(false, Ordering::Release);
                PostQuitMessage(0);
                return LRESULT(0);
            }
            if let Some(app) = APP_INSTANCE.get() {
                app.menu.handle_command(cmd_id, None);
            }
            LRESULT(0)
        }
        WM_DESTROY | WM_CLOSE => {
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
    println!("Checking VietIME IPC pipe: {}", PIPE_NAME);
    match std::fs::OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
        Ok(mut stream) => {
            let ping = vietime_ipc::Message::Ping;
            if let Ok(frame) = vietime_ipc::encode_frame(&ping) {
                if stream.write_all(&frame).is_ok() && stream.flush().is_ok() {
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).is_ok() {
                        let len = u32::from_le_bytes(len_buf) as usize;
                        if len <= vietime_ipc::MAX_FRAME_BYTES {
                            let mut buf = vec![0u8; 4 + len];
                            buf[..4].copy_from_slice(&len_buf);
                            if stream.read_exact(&mut buf[4..]).is_ok() {
                                if let Ok(vietime_ipc::Message::Pong { uptime_ms }) =
                                    vietime_ipc::decode_exact_frame(&buf)
                                {
                                    println!("VietIME IPC Server: RUNNING");
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
            println!("VietIME IPC Server: Connected, but response was invalid.");
        }
        Err(e) => {
            println!("VietIME IPC Server: OFFLINE ({e})");
        }
    }
}

fn stop_running_instance() {
    #[cfg(windows)]
    {
        println!("Checking for running VietIME Tray instance...");
        let class_name_wide: Vec<u16> =
            WINDOW_CLASS_NAME.encode_utf16().chain(Some(0)).collect();
        let hwnd = unsafe { FindWindowW(PCWSTR(class_name_wide.as_ptr()), None) };
        if let Ok(h) = hwnd {
            if !h.0.is_null() {
                println!("Found VietIME Tray window. Sending exit command...");
                unsafe {
                    let _ =
                        PostMessageW(Some(h), WM_COMMAND, WPARAM(ID_EXIT as usize), LPARAM(0));
                }

                // Chờ tối đa 3 giây xem tiến trình đã giải phóng mutex chưa
                let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
                for i in 0..30 {
                    std::thread::sleep(Duration::from_millis(100));
                    let h_mutex =
                        unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };
                    if let Ok(m) = h_mutex {
                        let err = unsafe { GetLastError() };
                        let _ = unsafe { CloseHandle(m) };
                        if err != ERROR_ALREADY_EXISTS {
                            println!(
                                "VietIME Tray stopped successfully (after {}ms).",
                                (i + 1) * 100
                            );
                            return;
                        }
                    }
                }
                println!("VietIME Tray signalled, but process did not exit within 3s.");
                return;
            }
        }
        println!("No running VietIME Tray instance detected.");
    }
    #[cfg(not(windows))]
    println!("Stopping tray instance is only supported on Windows.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_valid() {
        assert_eq!(MUTEX_NAME, r"Local\VietIMETray");
        assert_eq!(WINDOW_CLASS_NAME, "VietIMETrayWndClass");
        const { assert!(WM_TRAYICON >= WM_APP) };
    }
}
