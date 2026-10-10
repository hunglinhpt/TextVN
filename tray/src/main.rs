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
/// Tiến trình hiện tại có đang chạy elevated (admin) không? Dùng để BỎ QUA
/// prompt UAC (không thể xin elevation khi đã elevated; và trong CI headless
/// hộp thoại modal sẽ treo tray).
#[cfg(windows)]
fn is_process_elevated() -> bool {
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    // SAFETY: token mở/queries hợp lệ; mọi handle được đóng ngay.
    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        )
        .is_ok();
        let _ = windows::Win32::Foundation::CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// Đăng ký máy đã có chưa? (HKLM COM InprocServer32 tồn tại = bản cài/đăng ký máy.)
///
/// Vòng 13 (dump máy thật: gỡ 0.2.19 còn sót cả cây HKLM): đăng ký máy trỏ
/// vào file KHÔNG còn tồn tại là ghost — coi như CHƯA có để luồng repair
/// (per-user register / đề nghị UAC machine) chạy lại thay vì tin đăng ký
/// chết khiến TSF nạp DLL fail.
///
/// R2-42: COM HKLM được CLI elevated ghi ở BƯỚC ĐẦU, trước RegisterProfile —
/// chỉ dựa vào InprocServer32 thì một lần đăng ký máy dở dang (exit 3) bị coi là
/// hoàn tất mãi mãi. Đòi thêm LanguageProfile HKLM cho cả VI và EN (khớp
/// `machine_profile_metadata_ok` của CLI).
#[cfg(windows)]
fn machine_registration_present() -> bool {
    let inproc = format!(r"{TSF_TIP_REGISTRY_KEY}\InprocServer32");
    let com_ok = match read_registry_string(HKEY_LOCAL_MACHINE, &inproc) {
        Some(path) => {
            let trimmed = path.trim().trim_matches('"');
            !trimmed.is_empty() && std::path::Path::new(trimmed).is_file()
        }
        None => false,
    };
    com_ok
        && TSF_LANGS.iter().all(|lang| {
            let key = format!(r"{TSF_CTF_TIP_KEY}\LanguageProfile\0x{lang:08x}\{TSF_PROFILE_GUID}");
            registry_key_exists(HKEY_LOCAL_MACHINE, &key)
        })
}

/// Bản chạy từ thư mục người dùng ghi được (portable ở Downloads/D:\..., bộ cài
/// riêng tài khoản ở `%LOCALAPPDATA%\Programs`): Windows 11 mới từ chối activation
/// per-user (B7) và TextVN KHÔNG nâng quyền đăng ký phạm vi máy cho DLL ở thư mục
/// người dùng ghi được (SEC-02, R2-03/R2-15). Báo MỘT lần để người dùng biết cách
/// gõ được thay vì hỏng im lặng. Không còn khuyên chạy `register --scope machine`
/// bằng quyền admin từ thư mục portable (CLI cũng từ chối, exit 3).
#[cfg(windows)]
fn show_untrusted_path_notice_once() {
    show_activation_refused_notice_once(
        "uac_blocked_notice_done",
        "Windows từ chối kích hoạt bộ gõ TextVN chỉ đăng ký cho tài khoản này, \
và TextVN không đăng ký cho cả máy từ thư mục người dùng ghi được (bản portable \
hoặc bản cài riêng cho tài khoản).\r\n\r\nCách gõ được: cài TextVN cho mọi người \
dùng bằng bộ cài TextVN-setup-*-machine.exe (cài vào Program Files, đăng ký đúng \
chỗ và gõ được ngay), hoặc chọn TextVN bằng Win+Space nếu Windows cho phép.",
    );
}

/// Kênh Store bị Windows từ chối activation per-user (B7): không xin UAC (R2-07),
/// báo MỘT lần cách gõ được.
#[cfg(windows)]
fn show_store_activation_notice_once() {
    show_activation_refused_notice_once(
        "store_activation_notice_done",
        "Windows từ chối kích hoạt bộ gõ TextVN chỉ đăng ký cho tài khoản này. Bản \
TextVN từ Microsoft Store không đăng ký cho cả máy và không xin quyền Administrator.\r\n\r\n\
Cách gõ được: chọn TextVN bằng Win+Space nếu Windows cho phép, hoặc cài TextVN cho \
mọi người dùng bằng bộ cài TextVN-setup-*-machine.exe (cài vào Program Files) từ trang \
phát hành của TextVN.",
    );
}

/// Hộp thoại thông báo MỘT lần (marker `%APPDATA%\TextVN\<marker>`), trên thread
/// riêng, không hiện khi `--autostart`.
#[cfg(windows)]
fn show_activation_refused_notice_once(marker_name: &'static str, text: &'static str) {
    // Không làm phiền lúc đăng nhập: người dùng chủ động mở app sẽ thấy thông
    // báo; --autostart chỉ chạy ngầm (và build smoke dùng --autostart).
    if std::env::args().any(|a| a == "--autostart") {
        return;
    }
    let Some(marker) = std::env::var_os("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("TextVN").join(marker_name))
    else {
        return;
    };
    if marker.exists() {
        return;
    }
    // Hộp thoại trên THREAD RIÊNG: MessageBoxW chặn thread gọi nó — nếu gọi trên
    // main thread thì `TextVN.exe --stop` không thể kết thúc tiến trình (sự cố
    // smoke "did not stop cleanly" cần tránh; đồng thời không chặn IPC).
    std::thread::spawn(move || {
        let msg: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let cap: Vec<u16> = "TextVN".encode_utf16().chain(Some(0)).collect();
        // SAFETY: hwnd None = không owner; chuỗi nul-terminated sống trong lời gọi.
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(msg.as_ptr()),
                PCWSTR(cap.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        let _ = std::fs::create_dir_all(marker.parent().unwrap_or(std::path::Path::new(".")));
        let _ = std::fs::write(&marker, b"1");
    });
}

/// B7 (Win11 24H2+, build 26300): TIP chỉ đăng ký per-user bị Windows TỪ CHỐI
/// `ActivateProfile` → portable không gõ được dù đã đăng ký "OK". Phát hiện
/// bằng cách chạy `textvn-cli activate` (ẩn) — exit ≠ 0 là bị từ chối; khi đó
/// hỏi người dùng có muốn đăng ký PHẠM VI MÁY (UAC một lần) không. Luồng này
/// đã được chứng minh gõ được (giống bộ cài). Không hỏi khi --autostart
/// (tránh làm phiền lúc đăng nhập) và bỏ qua nếu HKLM đã có.
#[cfg(windows)]
fn offer_machine_registration_if_needed(store_channel: bool) {
    // KHÔNG bỏ qua khi --autostart: nếu Windows từ chối per-user và người dùng
    // không bao giờ tự mở app (chỉ autostart cùng Windows), máy sẽ gõ hỏng IM
    // LẶNG. Hỏi một lần (marker) ngay lúc đăng nhập là đánh đổi đúng — vẫn có
    // guard elevated + marker nên không lặp lại.
    // Tiến trình đang elevated (CI runner, terminal admin): không thể/không cần
    // xin UAC lại, và hộp thoại modal sẽ CHẶN tray trong môi trường headless
    // (sự cố CI 37112480137: "TextVN tray is still running before typing test").
    if is_process_elevated() {
        return;
    }
    if textvn_tray::store_win::refuse_if_packaged("đăng ký phạm vi máy") {
        return;
    }
    if machine_registration_present() {
        return;
    }
    // Chỉ hỏi MỘT lần cho cả máy/người dùng — người dùng bấm Cancel thì tôn
    // trọng, không hỏi lại mỗi lần mở app.
    let marker = std::env::var_os("APPDATA").map(|d| {
        std::path::PathBuf::from(d)
            .join("TextVN")
            .join("uac_prompt_done")
    });
    if let Some(m) = &marker {
        if m.exists() {
            return;
        }
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    let cli = dir.join("textvn-cli.exe");
    if !cli.is_file() {
        return;
    }
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    // Cho TSF một nhịp ổn định sau khi register (ILOT vừa ghi) trước khi kết luận.
    std::thread::sleep(std::time::Duration::from_millis(400));
    let status = std::process::Command::new(&cli)
        .arg("activate")
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if status.is_ok_and(|s| s.success()) {
        return;
    }

    // R2-07: kênh Store KHÔNG BAO GIỜ đề nghị UAC (Store không nhận app cần nâng quyền
    // cho bất kỳ chức năng nào) — chỉ chỉ đường tới bộ cài EXE cho mọi người dùng.
    if store_channel {
        show_store_activation_notice_once();
        return;
    }

    // SEC-02 (audit 2026-10-04) + R2-03/R2-15: chỉ nâng quyền từ thư mục chỉ
    // admin ghi được — Program Files theo known folder, trừ WindowsApps. KHÔNG
    // nâng từ %LOCALAPPDATA%\Programs (bộ cài per-user, bản Store staged),
    // Downloads/Desktop/thư mục portable: DLL ở đó user thường thay được, đăng ký
    // HKLM trỏ vào đó là nạp code của user vào tiến trình mọi tài khoản, kể cả
    // elevated (CWE-426/732). CLI tự từ chối lần nữa (exit 3). Bước này chỉ tới
    // được SAU khi activation per-user thất bại thật (B7).
    let trusted_install = textvn_tray::is_trusted_machine_install(
        &exe.to_string_lossy(),
        &textvn_tray::trusted_install_roots(),
    );
    if !trusted_install {
        show_untrusted_path_notice_once();
        return;
    }

    let msg: Vec<u16> = "Windows từ chối kích hoạt bộ gõ TextVN cho tài khoản này \
(giới hạn của Windows 11 với đăng ký chỉ per-user).\r\n\r\nĐăng ký phạm vi máy để gõ \
được ngay? Sẽ hiện hộp thoại quyền Administrator (UAC) MỘT lần.\r\n\r\nBấm OK để \
tiếp tục, Cancel nếu muốn tự chọn TextVN bằng Win+Space."
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let cap: Vec<u16> = "TextVN".encode_utf16().chain(Some(0)).collect();
    // SAFETY: hwnd None = không owner; chuỗi nul-terminated sống trong lời gọi.
    let answer = unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg.as_ptr()),
            PCWSTR(cap.as_ptr()),
            MB_OKCANCEL | MB_ICONWARNING,
        )
    };
    if let Some(m) = &marker {
        let _ = std::fs::create_dir_all(m.parent().unwrap_or(std::path::Path::new(".")));
        let _ = std::fs::write(m, b"1");
    }
    if answer != IDOK {
        return;
    }

    // Chạy CLI ELEVATED: `register --scope machine` (RegisterProfile API +
    // HKLM COM/category + mirror WOW64). R2-42: CHỜ tiến trình elevated kết thúc
    // và đọc exit code — bản cũ poll HKLM InprocServer32 (CLI ghi nó ở bước ĐẦU)
    // nên bước per-user chạy song song với bước máy còn dở, và báo thành công
    // (bật V) kể cả khi CLI elevated trả exit 3.
    let exit_code = run_elevated_and_wait(&cli, "register --scope machine", dir, 120_000);
    if exit_code.is_none() {
        // Người dùng huỷ UAC (ERROR_CANCELLED) — tôn trọng, không hỏi lại.
        return;
    }
    if !elevated_register_succeeded(exit_code) {
        // Thất bại thật (exit ≠ 0 / quá thời gian): bỏ marker để lần mở app sau
        // còn đề nghị lại, và nói rõ chỗ xem nguyên nhân.
        if let Some(m) = &marker {
            let _ = std::fs::remove_file(m);
        }
        let code = exit_code
            .flatten()
            .map_or("hết thời gian chờ".to_string(), |c| {
                format!("exit code {c}")
            });
        let msg: Vec<u16> = format!(
            "Đăng ký TextVN cho cả máy chưa hoàn tất ({code}).\r\n\r\nChi tiết: \
%LOCALAPPDATA%\\TextVN\\logs\\register.log. Bạn có thể cài lại bằng bộ cài \
TextVN-setup-*-machine.exe."
        )
        .encode_utf16()
        .chain(Some(0))
        .collect();
        // SAFETY: hwnd None = không owner; chuỗi nul-terminated sống trong lời gọi.
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(msg.as_ptr()),
                PCWSTR(cap.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
        return;
    }

    // Phạm vi máy đã XONG (exit 0) → chạy register per-user (layout/ILOT/activate
    // theo ngữ cảnh người dùng) để chốt trạng thái.
    let _ = std::process::Command::new(&cli)
        .arg("register")
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    let _ = std::process::Command::new(&cli)
        .arg("activate")
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    // Sau khi máy vừa được đăng ký là lúc người dùng muốn GÕ TIẾNG VIỆT: nếu
    // state đang là EN (có thể do họ bấm Ctrl+Shift thử trước đó) thì bật lại —
    // nếu không, mọi thứ "đăng ký OK" mà gõ vẫn ra raw và người dùng tưởng hỏng
    // (đúng ca 2026-10-03).
    if let Some(app) = APP_INSTANCE.get() {
        let (enabled, ver) = app.svc.set_global_enabled(true);
        app.ipc.broadcast_state_update("*", enabled, ver);
    }
    textvn_tray::notify_tray_state_changed();
}

/// Kết quả lần chạy elevated: `Some(Some(code))` = đã kết thúc với exit code,
/// `Some(None)` = đã chạy nhưng quá `timeout_ms`/không đọc được exit code,
/// `None` = không khởi chạy được (người dùng huỷ UAC).
#[cfg(windows)]
fn run_elevated_and_wait(
    exe: &std::path::Path,
    params: &str,
    dir: &std::path::Path,
    timeout_ms: u32,
) -> Option<Option<u32>> {
    let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
    let file: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let params_w: Vec<u16> = params.encode_utf16().chain(Some(0)).collect();
    let dir_w: Vec<u16> = dir
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params_w.as_ptr()),
        lpDirectory: PCWSTR(dir_w.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    // SAFETY: các chuỗi nul-terminated sống tới hết lời gọi; `sei` hợp lệ.
    if unsafe { ShellExecuteExW(&mut sei) }.is_err() || sei.hProcess.is_invalid() {
        return None;
    }
    // SAFETY: hProcess hợp lệ (SEE_MASK_NOCLOSEPROCESS), đóng ngay sau khi dùng.
    let code = unsafe {
        let waited = WaitForSingleObject(sei.hProcess, timeout_ms) == WAIT_OBJECT_0;
        let mut code = 0u32;
        let got = waited && GetExitCodeProcess(sei.hProcess, &mut code).is_ok();
        let _ = CloseHandle(sei.hProcess);
        got.then_some(code)
    };
    Some(code)
}

/// Chỉ coi đăng ký phạm vi máy thành công khi tiến trình elevated đã KẾT THÚC
/// với exit 0 (R2-42).
fn elevated_register_succeeded(result: Option<Option<u32>>) -> bool {
    matches!(result, Some(Some(0)))
}

#[cfg(windows)]
fn ensure_tsf_tip_registered(first_run: bool) {
    // Chốt chặn cứng: còn package identity thì mọi ghi đăng ký bị ảo hoá (R2-02).
    if textvn_tray::store_win::refuse_if_packaged("đăng ký TSF") {
        return;
    }
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
    // R2-25: lần đầu tray chạy cho TÀI KHOẢN này → luôn `register` một lần: bản cài cho
    // mọi người dùng chỉ ghi HKLM, nên `tsf_registration_is_current` (HKLM đủ) bỏ qua
    // bước per-user và tài khoản khác không bao giờ có TextVN trong danh sách bàn phím.
    // Sau lần đầu: chỉ ghi lại khi đã lệch — giữ bàn phím người dùng tự gỡ/xếp lại.
    if !first_run && tsf_registration_is_current(&dll) {
        return;
    }
    let cli_path = dir.join("textvn-cli.exe");
    if !cli_path.is_file() {
        return;
    }

    // R2-33: KHÔNG unregister khi đổi phiên bản — chuỗi gỡ (ILOT_UNINSTALL) rồi thêm lại
    // làm mất kích hoạt phiên đang mở và xếp lại bàn phím/mặc định của người dùng.
    // `register` ghi đè COM/profile trỏ DLL hiện tại là đủ.
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
    // Kênh Store (MSIX): `--msix-relay`/`--msix-install` và MỌI lần chạy có package
    // identity đi đường bootstrap rồi thoát — tiến trình trong package không bao giờ
    // tới tray/đăng ký (R2-02/R2-37). Bản portable/Inno: `None`, chạy như cũ.
    #[cfg(windows)]
    if let Some(code) = textvn_tray::package_bootstrap::dispatch(&args) {
        std::process::exit(code);
    }
    if args.len() > 1 {
        match args[1].as_str() {
            "--msix-guard" => {
                // Run value kênh Store khi tự khởi động đang TẮT: chỉ chạy guard gỡ/cập
                // nhật lúc đăng nhập rồi thoát (không chạy tray). Ngoài kênh Store: no-op.
                #[cfg(windows)]
                if let Some(ctx) = textvn_tray::store_win::detect_store_context() {
                    let _ = textvn_tray::store_win::run_store_guard(
                        &ctx,
                        textvn_tray::store::LaunchMode::GuardOnly,
                    );
                }
                return;
            }
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
                // R2-30: `--stop --if-image-under <dir>` chỉ dừng tray chạy từ <dir>
                // (gỡ một bản portable/Store không được tắt bản cài đang dùng).
                // R2-99: `--force` (chỉ bộ gỡ cài đặt) — tray của <dir> không thoát thì
                // dừng cưỡng bức để gỡ sạch. Exit 1 = tray vẫn còn chạy.
                let image_under = flag_value(&args[2..], "--if-image-under");
                let force = args[2..].iter().any(|a| a == "--force");
                let stopped =
                    stop_running_instance(image_under.as_deref().map(std::path::Path::new), force);
                std::process::exit(if stopped { 0 } else { 1 });
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
                println!(
                    "                [--if-image-under <dir>] chi dung khi tray chay tu <dir>"
                );
                println!(
                    "                [--force] (bo go cai dat, can --if-image-under) khong thoat thi dung cuong buc"
                );
                println!("  --free-ctrl-shift  Danh Ctrl+Shift cho TextVN (go phim tat doi ban phim cua Windows)");
                println!(
                    "  --msix-guard  (kenh Store) chi kiem tra goi Store da go/cap nhat roi thoat"
                );
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

/// Mặc định "Dành Ctrl + Shift cho TextVN" cho bản không qua bộ cài (không có task
/// `freectrlshift` của Inno): quyết định ĐÚNG MỘT LẦN (marker `%APPDATA%\TextVN\
/// ctrl_shift_default_applied`), sau đó tôn trọng lựa chọn ở Bảng điều khiển. Bản cài
/// bằng Inno (`unins000.exe` cạnh exe) để bộ cài quyết định.
/// - Portable: áp luôn (như trước).
/// - Kênh Store (R2-06, chính sách Store 10.2.8 "phải có đồng ý của người dùng trước khi
///   đổi cài đặt Windows"): HỎI Có/Không ở lần mở app bình thường đầu tiên; lưu cả câu
///   trả lời Không (marker `declined`) để không hỏi lại và gỡ cài đặt không đụng phím tắt.
/// - Nâng cấp từ bản ≤ 0.2.27 (không ghi marker) mà Ctrl + Shift đã "không gán": nhận
///   lại mục bản cũ đã đặt để gỡ cài đặt vẫn trả Ctrl + Shift (R2-92).
#[cfg(windows)]
fn apply_ctrl_shift_default_once(
    store_channel: bool,
    interactive: bool,
    previous_version: Option<&str>,
) {
    if textvn_tray::store_win::refuse_if_packaged("đổi phím tắt Ctrl+Shift") {
        return;
    }
    let installed = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("unins000.exe").is_file()))
        .unwrap_or(false);
    if installed {
        return;
    }
    let Some(marker) = textvn_tray::ctrl_shift_marker_path() else {
        return;
    };
    if marker.exists() {
        return;
    }
    let before = textvn_tray::hotkey::current();
    let legacy =
        textvn_tray::legacy_freed_layout_hotkey(previous_version, before.layout.as_deref());
    let legacy_entry = || ("Layout Hotkey".to_string(), None);
    if store_channel {
        // Bản Store cũ đã lấy Ctrl+Shift (không hỏi) — giữ quyết định đó, không hỏi lại.
        if legacy {
            textvn_tray::record_ctrl_shift_freed(vec![legacy_entry()]);
            return;
        }
        // Không hỏi lúc đăng nhập (--autostart); Windows không giữ Ctrl+Shift thì
        // không có gì để hỏi.
        if !interactive || !before.ctrl_shift_taken() {
            return;
        }
        // Thread riêng: MessageBoxW chặn thread gọi — không chặn tray/`--stop`.
        std::thread::spawn(|| {
            let answer = textvn_tray::store_win::message_box(
                "TextVN dùng Ctrl + Shift để chuyển gõ tiếng Việt / tiếng Anh (V/E).\r\n\r\n\
Windows đang dùng Ctrl + Shift để đổi bàn phím. Dành Ctrl + Shift cho TextVN? Windows \
vẫn đổi bàn phím bằng Win + Space.\r\n\r\nCó thể đổi lại bất cứ lúc nào trong Bảng \
điều khiển › \"Dành Ctrl + Shift cho TextVN\".",
                MB_YESNO | MB_ICONQUESTION,
            );
            if answer == IDYES {
                if let Ok(record) = textvn_tray::hotkey::free_ctrl_shift() {
                    textvn_tray::record_ctrl_shift_freed(record);
                }
            } else {
                textvn_tray::write_ctrl_shift_marker(textvn_tray::CTRL_SHIFT_MARKER_DECLINED);
            }
        });
        return;
    }
    if let Ok(mut record) = textvn_tray::hotkey::free_ctrl_shift() {
        if legacy && !record.iter().any(|(name, _)| name == "Layout Hotkey") {
            record.push(legacy_entry());
        }
        textvn_tray::record_ctrl_shift_freed(record);
    }
}

#[cfg(windows)]
fn free_ctrl_shift_cli() -> i32 {
    if textvn_tray::store_win::refuse_if_packaged("--free-ctrl-shift") {
        return 1;
    }
    let had_marker = textvn_tray::ctrl_shift_marker_path().is_some_and(|m| m.exists());
    let legacy_freed = textvn_tray::hotkey::current().layout.as_deref() == Some("3");
    match textvn_tray::hotkey::free_ctrl_shift() {
        Ok(mut record) => {
            // R2-40: bộ cài Inno ≤ 0.2.27 dành Ctrl+Shift mà không ghi marker — nâng
            // cấp thấy "không gán" sẵn, không có marker → coi như bản cũ đã đặt (gỡ cài
            // đặt trả lại như trước đây).
            if record.is_empty() && !had_marker && legacy_freed {
                record.push(("Layout Hotkey".to_string(), None));
            }
            textvn_tray::record_ctrl_shift_freed(record);
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

/// Chạy `textvn-cli activate` ẩn, fire-and-forget (thread riêng — không chặn
/// UI thread của tray). Thiếu CLI (cài đặt hỏng) → bỏ qua im lặng: toggle
/// mode vẫn hoạt động, chỉ thiếu hành vi chuyển bộ gõ.
#[cfg(windows)]
fn spawn_activate_profile() {
    std::thread::Builder::new()
        .name("textvn-activate".into())
        .spawn(|| {
            let mut cli = match std::env::current_exe() {
                Ok(p) => p,
                Err(_) => return,
            };
            cli.set_file_name("textvn-cli.exe");
            if !cli.is_file() {
                return;
            }
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let _ = std::process::Command::new(&cli)
                .arg("activate")
                .creation_flags(CREATE_NO_WINDOW)
                .status();
        })
        .ok();
}

#[cfg(windows)]
fn run_tray_app() {
    let is_autostart = std::env::args().any(|a| a == "--autostart");
    let is_settings = std::env::args().any(|a| a == "--settings");

    // Kênh Store (exe dưới %LOCALAPPDATA%\Programs\TextVN-Store + stage.json): guard
    // TRƯỚC single-instance mutex — gói đã gỡ → dọn rồi thoát (R2-04); gói có bản mới
    // hơn → chạy app trong gói để stage lại rồi thoát (R2-05/R2-23). API lỗi → chạy tiếp.
    let store_ctx = textvn_tray::store_win::detect_store_context();
    if let Some(ctx) = &store_ctx {
        let mode = if is_autostart {
            textvn_tray::store::LaunchMode::Autostart
        } else {
            textvn_tray::store::LaunchMode::Normal
        };
        if textvn_tray::store_win::run_store_guard(ctx, mode)
            == textvn_tray::store_win::GuardOutcome::Exit
        {
            return;
        }
    }

    // R2-98: mục Run HKLM (bộ cài cho mọi người dùng) chạy tray ở MỌI tài khoản; tài
    // khoản đã bỏ chọn "Khởi động cùng Windows" (marker) thì thoát ngay. Kênh Store tắt
    // tự khởi động bằng `--msix-guard`, không dùng marker.
    if is_autostart
        && store_ctx.is_none()
        && std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
            .is_some_and(|dir| textvn_tray::autostart::autostart_suppressed_for(&dir))
    {
        return;
    }

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

    // Giải phóng phím tắt Ctrl+Shift khỏi Windows Layout Hotkey để TextVN sử dụng —
    // CHỈ một lần cho bản không qua bộ cài (xem hàm). Trước đây gọi vô điều kiện ở mỗi
    // lần khởi động: người dùng bỏ chọn "Dành Ctrl + Shift" trong Bảng điều khiển (hoặc
    // task của bộ cài) thì lần đăng nhập sau Windows lại mất phím tắt của họ.
    // R2-91: TEXTVN_SKIP_TSF_REGISTRATION (runtime smoke của build-release) = không đổi
    // gì của máy dev — kể cả phím tắt Ctrl+Shift và marker của nó.
    if std::env::var_os("TEXTVN_SKIP_TSF_REGISTRATION").is_none() {
        apply_ctrl_shift_default_once(store_ctx.is_some(), !is_autostart, svc.previous_version());
    }

    // TSF là đường gõ chuẩn mặc định. Toggle Ctrl+Shift xử lý IN-PROCESS trong
    // TIP (ModifierToggle + KeyTraceSink, compose.rs) — tray chỉ nhận kết quả
    // qua IPC để đổi icon. WH_KEYBOARD_LL trong tray (0.2.7+) là BỘ DÒ TAP CHỈ
    // QUAN SÁT (không ăn phím, không inject) giúp Ctrl+Shift hoạt động khi
    // TextVN không phải bộ gõ active; chống toggle đôi bằng try_claim_global_
    // toggle() (250ms chéo nguồn). Chính sách AV cập nhật tương ứng (A3).

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

    // Đăng ký TSF + đề nghị đăng ký máy (B7): chạy SAU khi cửa sổ khay + icon
    // đã tồn tại (vòng 13) — subprocess register/activate mất 1-3s, nếu chạy
    // trước cửa sổ thì `TextVN.exe --stop` lúc khởi động không thấy cửa sổ
    // (smoke "did not stop cleanly" trên máy mới dọn HKLM). Build/release
    // smoke vẫn chứng minh IPC lifecycle; TEXTVN_SKIP_TSF_REGISTRATION giữ
    // nguyên ý nghĩa: không đụng đăng ký TSF thật của máy dev.
    // TSF/offer chạy trên BACKGROUND THREAD (vòng 13): subprocess
    // register/activate mất 1-3s — nếu chạy trên main thread thì `--stop`
    // đến trong giai đoạn đó phải chờ đến khi xong mới thoát (CI bắt
    // "tray still running" 3 lần liên tiếp). Main vào message loop NGAY;
    // process exit khi --stop sẽ kết thúc thread (subprocess CLI tự hoàn
    // tất đăng ký; lệch thì self-heal lần chạy sau).
    // Kênh Store: đồng bộ Run value sang thư mục phiên bản này, dọn bản cũ + staging.
    if let Some(ctx) = store_ctx.clone() {
        std::thread::spawn(move || textvn_tray::store_win::after_tray_start(&ctx));
    }
    if std::env::var_os("TEXTVN_SKIP_TSF_REGISTRATION").is_none() {
        let svc_bg = svc.clone();
        let store_channel = store_ctx.is_some();
        std::thread::spawn(move || {
            ensure_tsf_tip_registered(svc_bg.first_run());
            // B7 trên Win11 24H2+: đăng ký per-user OK nhưng Windows TỪ CHỐI
            // ActivateProfile → portable đứng một mình không gõ được. Đề nghị
            // đăng ký phạm vi máy (UAC một lần) — luồng đã được chứng minh gõ
            // được (giống bộ cài). Chỉ hỏi khi người dùng chủ động mở app
            // (không hỏi lúc --autostart). Kênh Store: không bao giờ xin UAC (R2-07).
            offer_machine_registration_if_needed(store_channel);
        });
    }

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
    // Yêu cầu thoát đến TRƯỚC khi cửa sổ tồn tại (--stop ngay lúc khởi động,
    // B17): chuyển thành WM_REQUEST_EXIT để đi ĐÚNG đường thoát graceful
    // (broadcast Shutdown cho Hook rồi PostQuitMessage).
    if textvn_tray::take_tray_exit_pending() {
        unsafe {
            let _ = PostMessageW(
                Some(hwnd),
                textvn_tray::WM_REQUEST_EXIT,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }
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
            // Kích hoạt profile TextVN cho phiên: khi người dùng bấm hotkey
            // TRONG KHI đang đứng ở bàn phím khác (MS Việt / US trong
            // Win+Space), đổi mode TextVN thôi thì không đổi được thứ họ gõ —
            // phải chuyển cả bộ gõ active sang TextVN thì typing mới theo
            // icon (báo cáo 0.2.9 "đã chuyển EN mà vẫn gõ tiếng Việt").
            // R2-34: người dùng bỏ "Dành Ctrl + Shift cho TextVN" (hoặc từ chối ở
            // bản Store) → Windows đang đổi bàn phím bằng CHÍNH tổ hợp này; ép kích
            // hoạt lại TextVN sẽ kéo họ về TextVN mỗi lần muốn rời đi.
            if !textvn_tray::hotkey::current().ctrl_shift_taken() {
                spawn_activate_profile();
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
    match textvn_ipc::pipe_client_options().open(PIPE_NAME) {
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

/// Giá trị đi sau `flag` trong `rest` (`--flag <value>`), nếu có.
fn flag_value(rest: &[String], flag: &str) -> Option<String> {
    rest.iter()
        .position(|a| a == flag)
        .and_then(|i| rest.get(i + 1))
        .filter(|v| !v.is_empty() && !v.starts_with("--"))
        .cloned()
}

/// Đường dẫn exe của tiến trình `pid` (`QueryFullProcessImageNameW`).
#[cfg(windows)]
fn process_image_path(pid: u32) -> Option<String> {
    // SAFETY: handle chỉ quyền truy vấn tối thiểu, đóng ngay; buffer khớp `len`.
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 32_768];
        let mut len = buf.len() as u32;
        let ok =
            QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
                .is_ok();
        let _ = CloseHandle(h);
        ok.then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// `image` nằm dưới `dir` (so cả dạng gốc lẫn canonical của `dir`).
fn image_is_under(image: &str, dir: &std::path::Path) -> bool {
    let mut roots = vec![dir.to_string_lossy().into_owned()];
    if let Ok(c) = std::fs::canonicalize(dir) {
        roots.push(c.to_string_lossy().into_owned());
    }
    textvn_tray::path_is_under_any(image, &roots)
}

/// Dừng tray đang chạy. `true` = không còn tray (đã dừng, không có, hoặc là bản ở nơi
/// khác khi lọc `image_under`); `false` = tray vẫn chạy.
fn stop_running_instance(image_under: Option<&std::path::Path>, force: bool) -> bool {
    #[cfg(windows)]
    {
        // `--force` chỉ hợp lệ kèm `--if-image-under`: không bao giờ dừng cưỡng bức tray
        // của một bản TextVN khác.
        let force = force && image_under.is_some();
        println!("Checking for running TextVN Tray instance...");
        let class_name_wide: Vec<u16> = WINDOW_CLASS_NAME.encode_utf16().chain(Some(0)).collect();
        let hwnd = unsafe { FindWindowW(PCWSTR(class_name_wide.as_ptr()), None) };
        if let Ok(h) = hwnd {
            if !h.0.is_null() {
                let mut window_pid = 0u32;
                // GetWindowThreadProcessId trả thread ID qua return value; tham số
                // out là PID. Trộn hai giá trị này gửi WM_QUIT vào PID thay vì
                // queue UI, khiến `--stop` luôn phải rơi xuống TerminateProcess.
                let thread_id = unsafe { GetWindowThreadProcessId(h, Some(&mut window_pid)) };
                if thread_id == 0 {
                    println!("TextVN window owner thread could not be resolved.");
                    return false;
                }
                // R2-30: chỉ dừng tray của đúng bản đang gỡ/cập nhật. Không đọc
                // được image path → coi là bản khác (không dừng nhầm).
                if let Some(dir) = image_under {
                    let owned = process_image_path(window_pid)
                        .is_some_and(|image| image_is_under(&image, dir));
                    if !owned {
                        println!(
                            "TextVN (PID {window_pid}) is running from another location; not stopping it."
                        );
                        return true;
                    }
                }
                // R2-99: handle CHỈ quyền SYNCHRONIZE để chờ tiến trình THOÁT HẲN — nhả
                // mutex chưa phải đã thoát: bộ gỡ xoá TextVN.exe ngay sau `--stop` thì
                // file còn bị khoá và gỡ không sạch.
                // SAFETY: handle chỉ dùng để chờ, đóng ở mọi nhánh bên dưới.
                let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, window_pid) }.ok();
                let wait_exit = |ms: u32| -> bool {
                    match process {
                        // SAFETY: handle hợp lệ tới khi CloseHandle cuối hàm.
                        Some(p) => (unsafe { WaitForSingleObject(p, ms) }) == WAIT_OBJECT_0,
                        None => true,
                    }
                };
                let close_process = || {
                    if let Some(p) = process {
                        // SAFETY: đóng đúng một lần handle do OpenProcess cấp.
                        let _ = unsafe { CloseHandle(p) };
                    }
                };
                println!("Found TextVN window. Requesting a graceful close...");
                // The named-pipe request is the primary control path. It enters
                // the app's own IPC worker, then requests exit on the tray thread
                // (tray broadcast Shutdown cho Hook rồi mới PostQuitMessage).
                if let Ok(mut stream) = textvn_ipc::pipe_client_options().open(PIPE_NAME) {
                    if let Ok(frame) = textvn_ipc::encode_frame(&textvn_ipc::Message::Shutdown) {
                        let _ = stream.write_all(&frame);
                        let _ = stream.flush();
                    }
                }

                let mutex_name_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
                let started = std::time::Instant::now();
                // Đợi tiến trình giải phóng mutex rồi THOÁT HẲN; hết hạn → false.
                let wait_stopped = |iterations: u32| -> bool {
                    for _ in 0..iterations {
                        std::thread::sleep(Duration::from_millis(100));
                        let h_mutex =
                            unsafe { CreateMutexW(None, true, PCWSTR(mutex_name_wide.as_ptr())) };
                        if let Ok(m) = h_mutex {
                            let err = unsafe { GetLastError() };
                            let _ = unsafe { CloseHandle(m) };
                            if err != ERROR_ALREADY_EXISTS && wait_exit(5000) {
                                println!(
                                    "TextVN stopped successfully (after {}ms).",
                                    started.elapsed().as_millis()
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
                if wait_stopped(20) {
                    close_process();
                    return true;
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
                    }
                }
                if wait_stopped(10) {
                    close_process();
                    return true;
                }
                close_process();
                // Mặc định KHÔNG TerminateProcess: mở PROCESS_TERMINATE tới process khác
                // là mẫu hành vi AV soi (process killer), và dừng cưỡng bức bỏ lỡ cleanup
                // (icon khay, broadcast Shutdown). Ngoại lệ R2-99: bộ gỡ cài đặt (`--force`,
                // luôn kèm `--if-image-under`) — tray của CHÍNH thư mục đang gỡ còn chạy
                // thì file bị khoá, gỡ không sạch.
                if force {
                    // SAFETY: handle đóng ngay sau khi chờ.
                    if let Ok(p) = unsafe {
                        OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, window_pid)
                    } {
                        let gone = unsafe { TerminateProcess(p, 1) }.is_ok()
                            && unsafe { WaitForSingleObject(p, 5000) } == WAIT_OBJECT_0;
                        let _ = unsafe { CloseHandle(p) };
                        if gone {
                            println!(
                                "TextVN (PID {window_pid}) did not exit gracefully; stopped for uninstall."
                            );
                            return true;
                        }
                    }
                }
                println!(
                    "TextVN (PID {window_pid}) did not stop within 3s; close it from the tray menu."
                );
                return false;
            }
        }
        println!("No running TextVN instance detected.");
        true
    }
    #[cfg(not(windows))]
    {
        let _ = (image_under, force);
        println!("Stopping tray instance is only supported on Windows.");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_filter_flag_parsing_and_image_match() {
        let args: Vec<String> = ["--if-image-under", r"D:\T"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            flag_value(&args, "--if-image-under").as_deref(),
            Some(r"D:\T")
        );
        assert_eq!(flag_value(&args[..1], "--if-image-under"), None);
        assert_eq!(
            flag_value(
                &["--if-image-under".into(), "--x".into()],
                "--if-image-under"
            ),
            None
        );
        assert!(image_is_under(
            r"d:\t\TextVN.exe",
            std::path::Path::new(r"D:\T")
        ));
        assert!(!image_is_under(
            r"C:\Program Files\TextVN\TextVN.exe",
            std::path::Path::new(r"D:\T")
        ));
    }

    /// R2-42: chỉ exit 0 của tiến trình elevated ĐÃ KẾT THÚC mới là thành công.
    #[test]
    fn elevated_register_requires_finished_exit_zero() {
        assert!(elevated_register_succeeded(Some(Some(0))));
        assert!(!elevated_register_succeeded(Some(Some(3))));
        assert!(!elevated_register_succeeded(Some(None)));
        assert!(!elevated_register_succeeded(None));
    }

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
