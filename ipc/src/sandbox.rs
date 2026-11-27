// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-26 — đường IPC + cấu hình cho tiến trình **sandbox** Windows (AppContainer / LPAC: ô tìm
//! kiếm Start, Calculator, ứng dụng Store). Thiết kế + số đo thật: `docs/specs/sandbox-appcontainer.md`.
//!
//! Số đo trên tiến trình AppContainer/LPAC thật (spike 2026-11-27, Windows 11 26300):
//! - đọc `%APPDATA%\TextVN\config.json` → lỗi 5; mở pipe khay (DACL chỉ user/SYSTEM/Admins) → lỗi 5;
//! - `%LOCALAPPDATA%` và known-folder LocalAppData bị **chuyển hướng** về `…\Packages\<tên>\AC`,
//!   còn `%APPDATA%`/`%USERPROFILE%` giữ đường thật ⇒ bản sao đặt ở `%APPDATA%\TextVN\sandbox`;
//! - thư mục có ACE `S-1-15-2-1` (AppContainer) + `S-1-15-2-2` (LPAC) đọc được, không cần quyền
//!   trên thư mục cha (bypass traverse);
//! - pipe có ACE đó + nhãn toàn vẹn Low (`S:(ML;;NW;;;LW)`) mở được; `OpenProcess` ngược lên
//!   tray thì **bị từ chối** ⇒ client sandbox không xác thực được tên exe server;
//! - tiến trình AppContainer **không tạo được** pipe tên toàn cục (lỗi 5, chỉ `LOCAL\…` riêng của
//!   nó) ⇒ không ứng dụng sandbox nào giả được pipe sandbox của tray.

#[cfg(windows)]
use std::path::PathBuf;

/// Thư mục bản sao sandbox, nằm TRONG thư mục cấu hình `%APPDATA%\TextVN`.
pub const SANDBOX_DIR_NAME: &str = "sandbox";
/// Hậu tố tên pipe sandbox: `\\.\pipe\textvn-ipc-v1-s<phiên>-sandbox`.
pub const SANDBOX_PIPE_SUFFIX: &str = "-sandbox";
/// SID nhóm `ALL APPLICATION PACKAGES` (mọi AppContainer).
pub const SID_ALL_APP_PACKAGES: &str = "S-1-15-2-1";
/// SID nhóm `ALL RESTRICTED APPLICATION PACKAGES` (LPAC).
pub const SID_ALL_RESTRICTED_APP_PACKAGES: &str = "S-1-15-2-2";

/// Quyền của sandbox trên pipe sandbox: `FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_EA |
/// FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE`. **Không** dùng `GW`: `GENERIC_WRITE` của
/// pipe gồm `FILE_APPEND_DATA` = `FILE_CREATE_PIPE_INSTANCE` ⇒ ứng dụng sandbox tạo được instance
/// server CÙNG TÊN và mạo danh tray với client sandbox khác (audit 0.2.93).
pub const SANDBOX_PIPE_RIGHTS: u32 = 0x0012_008B;
/// Quyền client sandbox xin khi mở pipe (phải là tập con của [`SANDBOX_PIPE_RIGHTS`]):
/// `FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE`.
pub const SANDBOX_CLIENT_ACCESS: u32 = 0x0010_0083;

/// SDDL của pipe khay. `sandbox = false`: pipe đầy đủ, chỉ user hiện tại + SYSTEM + Admins
/// (SEC-05). `sandbox = true`: thêm đọc/ghi DỮ LIỆU cho AppContainer + LPAC
/// ([`SANDBOX_PIPE_RIGHTS`] — không tạo được instance) và hạ nhãn toàn vẹn về Low (không thì
/// tiến trình Low bị chặn ghi lên đối tượng Medium) — server coi MỌI client của pipe này là hạn
/// chế (`sandbox_policy` phía tray).
pub fn pipe_sddl(user_sid: &str, sandbox: bool) -> String {
    let base = format!("D:(A;;GA;;;{user_sid})(A;;GA;;;SY)(A;;GA;;;BA)");
    if sandbox {
        let r = SANDBOX_PIPE_RIGHTS;
        format!(
            "{base}(A;;0x{r:x};;;{SID_ALL_APP_PACKAGES})(A;;0x{r:x};;;{SID_ALL_RESTRICTED_APP_PACKAGES})S:(ML;;NW;;;LW)"
        )
    } else {
        base
    }
}

/// `OpenOptions` cho client mở pipe sandbox: quyền tường minh [`SANDBOX_CLIENT_ACCESS`] (xin
/// `GENERIC_WRITE` sẽ bị từ chối vì DACL không cấp `FILE_APPEND_DATA`) + chống impersonation
/// như [`crate::pipe_client_options`].
pub fn sandbox_pipe_client_options() -> std::fs::OpenOptions {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.access_mode(SANDBOX_CLIENT_ACCESS);
        options.security_qos_flags(1 << 16);
    }
    options
}

/// SDDL thư mục bản sao sandbox: DACL **bảo vệ** (không kế thừa từ `%APPDATA%`), user/SYSTEM/
/// Admins toàn quyền, AppContainer + LPAC chỉ đọc; mọi ACE kế thừa xuống tệp (OICI).
pub fn sandbox_dir_sddl(user_sid: &str) -> String {
    format!(
        "D:P(A;OICI;FA;;;{user_sid})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)\
         (A;OICI;FRFX;;;{SID_ALL_APP_PACKAGES})(A;OICI;FRFX;;;{SID_ALL_RESTRICTED_APP_PACKAGES})"
    )
}

/// SDDL (DACL) có cấp quyền cho CẢ AppContainer lẫn LPAC không? Windows rút gọn
/// `S-1-15-2-1` thành `AC`; LPAC giữ SID dạng số.
pub fn sddl_grants_sandbox(sddl: &str) -> bool {
    (sddl.contains(";;;AC)") || sddl.contains(SID_ALL_APP_PACKAGES))
        && sddl.contains(SID_ALL_RESTRICTED_APP_PACKAGES)
}

/// Đúng tên file `TextVN.exe` (không phân biệt hoa thường), không chỉ phần đuôi (R2-110).
pub fn image_is_textvn_exe(image: &str) -> bool {
    image
        .rsplit(['\\', '/'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("TextVN.exe"))
}

/// Pipe sandbox của phiên hiện tại.
#[cfg(windows)]
pub fn sandbox_pipe_name() -> String {
    format!("{}{SANDBOX_PIPE_SUFFIX}", crate::session_pipe_name())
}

/// Kết quả mở pipe khay.
#[cfg(windows)]
pub enum PipeOpen {
    /// Đã nối và xác thực server. `sandboxed = true`: nối qua pipe sandbox — server chỉ phục vụ
    /// tập lệnh hạn chế, client KHÔNG gửi nội dung người dùng (từ đã học…) qua kết nối này.
    Connected {
        file: std::fs::File,
        sandboxed: bool,
    },
    /// Hết instance (tray bận) — tên pipe để `WaitNamedPipeW` rồi thử lại.
    Busy(String),
    /// Tray không chạy / server không phải TextVN.
    Unavailable,
}

/// Mở pipe khay cho TIP/hook: pipe phiên → pipe tên cũ → (chỉ khi pipe đầy đủ **từ chối quyền**,
/// tức tiến trình sandbox) pipe sandbox. Tiến trình thường không bao giờ rơi xuống pipe sandbox
/// vì bận/không thấy — không thì mất đường lưu từ đã học, toast, Math…
#[cfg(windows)]
pub fn open_tray_pipe() -> PipeOpen {
    open_tray_pipe_from(&crate::candidate_pipe_names(), &sandbox_pipe_name())
}

/// [`open_tray_pipe`] với tên pipe truyền vào (test dùng tên riêng — tray thật đang chạy giữ
/// tên thật).
#[cfg(windows)]
pub fn open_tray_pipe_from(full: &[String], sandbox: &str) -> PipeOpen {
    const ERROR_ACCESS_DENIED: i32 = 5;
    const ERROR_PIPE_BUSY: i32 = 231;
    let mut busy: Option<String> = None;
    let mut denied = false;
    for name in full {
        match crate::pipe_client_options().open(name) {
            Ok(file) if server_is_textvn(&file, false) => {
                return PipeOpen::Connected {
                    file,
                    sandboxed: false,
                }
            }
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                busy.get_or_insert_with(|| name.clone());
            }
            Err(e) if e.raw_os_error() == Some(ERROR_ACCESS_DENIED) => denied = true,
            Err(_) => {}
        }
    }
    if denied && busy.is_none() {
        match sandbox_pipe_client_options().open(sandbox) {
            Ok(file) if server_is_textvn(&file, true) => {
                return PipeOpen::Connected {
                    file,
                    sandboxed: true,
                }
            }
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                return PipeOpen::Busy(sandbox.to_string())
            }
            Err(_) => {}
        }
    }
    match busy {
        Some(name) => PipeOpen::Busy(name),
        None => PipeOpen::Unavailable,
    }
}

/// Audit A7b + R2-110: server đầu ống phải là `TextVN.exe` CÙNG PHIÊN (chống pipe squatting).
///
/// R2-26: client trong sandbox bị từ chối `OpenProcess` lên tray (đo thật) — trên pipe sandbox
/// (`sandbox_pipe = true`), "từ chối quyền" được chấp nhận sau khi phiên khớp: tiến trình
/// AppContainer không tạo được pipe tên toàn cục nên kẻ chiếm tên chỉ có thể là tiến trình
/// thường của CHÍNH user đó (vốn đã sửa được config), và client sandbox không gửi nội dung người
/// dùng qua kết nối này.
#[cfg(windows)]
pub fn server_is_textvn(stream: &std::fs::File, sandbox_pipe: bool) -> bool {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE};
    use windows::Win32::System::Pipes::{GetNamedPipeServerProcessId, GetNamedPipeServerSessionId};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let mut server_pid = 0u32;
    let mut server_session = 0u32;
    // SAFETY: handle hợp lệ (mượn từ `stream`); handle tiến trình mở ra được đóng ngay.
    unsafe {
        let handle = HANDLE(stream.as_raw_handle() as *mut _);
        if GetNamedPipeServerProcessId(handle, &mut server_pid).is_err() {
            return false;
        }
        if GetNamedPipeServerSessionId(handle, &mut server_session).is_err()
            || server_session != crate::current_session_id()
        {
            return false;
        }
        let process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, server_pid) {
            Ok(p) => p,
            Err(e) => return sandbox_pipe && e.code() == ERROR_ACCESS_DENIED.to_hresult(),
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(process);
        ok && image_is_textvn_exe(&String::from_utf16_lossy(
            &buf[..(len as usize).min(buf.len())],
        ))
    }
}

/// SDDL DACL hiện tại của tệp/thư mục (doctor + test kiểm ACE sandbox). `None` = không đọc được.
#[cfg(windows)]
pub fn path_dacl_sddl(path: &std::path::Path) -> Option<String> {
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
    use windows::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW,
        SDDL_REVISION_1, SE_FILE_OBJECT,
    };
    use windows::Win32::Security::{DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};
    let wide: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: SD + chuỗi SDDL do API cấp, giải phóng đúng một lần; chuỗi đường dẫn có NUL.
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        let status = GetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            None,
            None,
            &mut sd,
        );
        if status != ERROR_SUCCESS {
            return None;
        }
        let mut text = PWSTR::null();
        let ok = ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            None,
        )
        .is_ok();
        let out = if ok { text.to_string().ok() } else { None };
        if !text.is_null() {
            let _ = LocalFree(Some(HLOCAL(text.0 as *mut _)));
        }
        let _ = LocalFree(Some(HLOCAL(sd.0)));
        out
    }
}

/// Tiến trình hiện tại chạy trong AppContainer (kể cả LPAC)? Đọc token MỘT lần mỗi tiến trình.
///
/// Audit 0.2.93: TIP chỉ đọc bản sao sandbox trong tiến trình AppContainer THẬT. Trước đó "đọc
/// config thật lỗi ⇒ đọc bản sao" có thể kích hoạt ở ứng dụng thường khi tệp thật bị khoá thoáng
/// qua (tray đang thay tệp, phần mềm diệt virus) ⇒ ứng dụng mất gõ tắt/từ tiếng Anh tới lần nạp
/// lại sau.
#[cfg(windows)]
pub fn current_process_is_app_container() -> bool {
    static CACHED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{GetTokenInformation, TokenIsAppContainer, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        // SAFETY: handle token đóng ngay; buffer 4 byte cho DWORD.
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }
            let mut value = 0u32;
            let mut ret = 0u32;
            let ok = GetTokenInformation(
                token,
                TokenIsAppContainer,
                Some(&mut value as *mut u32 as *mut _),
                4,
                &mut ret,
            )
            .is_ok();
            let _ = CloseHandle(token);
            ok && value != 0
        }
    })
}

/// SID dạng chuỗi của user sở hữu tiến trình hiện tại (`S-1-5-21-…`).
#[cfg(windows)]
pub fn current_user_sid_string() -> Option<String> {
    use windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
    use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    // SAFETY: buffer căn 8 byte (TOKEN_USER chứa con trỏ); mọi handle/chuỗi do API cấp được
    // giải phóng đúng một lần.
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;
        let mut buf = [0u64; 128];
        let mut ret = 0u32;
        let queried = GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr() as *mut _),
            std::mem::size_of_val(&buf) as u32,
            &mut ret,
        );
        let _ = CloseHandle(token);
        queried.ok()?;
        let user = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut sid = windows::core::PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut sid).ok()?;
        let out = sid.to_string().ok();
        let _ = LocalFree(Some(HLOCAL(sid.0 as *mut _)));
        out
    }
}

/// Thư mục hồ sơ (`C:\Users\<tên>`) từ `ProfileList` theo SID của tiến trình — dự phòng khi
/// `%APPDATA%` thiếu. AppContainer đọc được khoá này; LPAC thì không (đo thật) → `None`.
#[cfg(windows)]
fn profile_image_path() -> Option<PathBuf> {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
    };
    let sid = current_user_sid_string()?;
    let key = format!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\{sid}");
    // SAFETY: buffer/độ dài hợp lệ; khoá mở ra được đóng.
    unsafe {
        let mut hk = HKEY::default();
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(key.as_str()),
            Some(0),
            KEY_READ,
            &mut hk,
        )
        .ok()
        .ok()?;
        let mut buf = [0u16; 520];
        let mut len = std::mem::size_of_val(&buf) as u32;
        let status = RegQueryValueExW(
            hk,
            &HSTRING::from("ProfileImagePath"),
            None,
            None,
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut len),
        );
        let _ = RegCloseKey(hk);
        status.ok().ok()?;
        let chars = (len as usize / 2).min(buf.len());
        let text = String::from_utf16_lossy(&buf[..chars]);
        let text = text.trim_end_matches('\0');
        (!text.is_empty() && !text.contains('%')).then(|| PathBuf::from(text))
    }
}

/// Các đường thư mục bản sao sandbox để ĐỌC, theo thứ tự: `%APPDATA%` (giữ đường thật trong
/// AppContainer + LPAC) → `ProfileList` (AppContainer) → `%USERPROFILE%`. Không trùng lặp.
#[cfg(windows)]
pub fn sandbox_dir_candidates() -> Vec<PathBuf> {
    let roaming = |root: PathBuf| root.join("AppData").join("Roaming");
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |base: PathBuf| {
        let dir = base.join("TextVN").join(SANDBOX_DIR_NAME);
        let key = dir.to_string_lossy().to_ascii_lowercase();
        if !out
            .iter()
            .any(|p| p.to_string_lossy().to_ascii_lowercase() == key)
        {
            out.push(dir);
        }
    };
    if let Some(appdata) = std::env::var_os("APPDATA").filter(|v| !v.is_empty()) {
        push(PathBuf::from(appdata));
    }
    if let Some(profile) = profile_image_path() {
        push(roaming(profile));
    }
    if let Some(home) = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty()) {
        push(roaming(PathBuf::from(home)));
    }
    out
}

/// Đọc tệp `file_name` của bản sao sandbox (`config.json` / `state.json`) từ ứng viên đầu tiên
/// đọc được. Tray đang thay tệp (rename) → lỗi chia sẻ 32 thoáng qua: thử lại một lần.
#[cfg(windows)]
pub fn read_sandbox_file(file_name: &str) -> Option<Vec<u8>> {
    read_sandbox_file_in(&sandbox_dir_candidates(), file_name)
}

/// [`read_sandbox_file`] trên danh sách thư mục truyền vào (test).
#[cfg(windows)]
pub fn read_sandbox_file_in(dirs: &[PathBuf], file_name: &str) -> Option<Vec<u8>> {
    const ERROR_SHARING_VIOLATION: i32 = 32;
    for dir in dirs {
        let path = dir.join(file_name);
        for attempt in 0..2 {
            match std::fs::read(&path) {
                Ok(bytes) => return Some(bytes),
                Err(e) if attempt == 0 && e.raw_os_error() == Some(ERROR_SHARING_VIOLATION) => {
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
                Err(_) => break,
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipe_sddl_unchanged_and_has_no_appcontainer_access() {
        let s = pipe_sddl("S-1-5-21-1-2-3-1001", false);
        assert_eq!(s, "D:(A;;GA;;;S-1-5-21-1-2-3-1001)(A;;GA;;;SY)(A;;GA;;;BA)");
        assert!(
            !s.contains("S-1-15-2"),
            "pipe đầy đủ không được mở cho sandbox"
        );
    }

    #[test]
    fn sandbox_pipe_sddl_grants_ac_lpac_data_rights_only_and_low_label() {
        let s = pipe_sddl("S-1-5-21-1-2-3-1001", true);
        assert!(s.starts_with("D:(A;;GA;;;S-1-5-21-1-2-3-1001)(A;;GA;;;SY)(A;;GA;;;BA)"));
        assert!(
            s.contains("(A;;0x12008b;;;S-1-15-2-1)") && s.contains("(A;;0x12008b;;;S-1-15-2-2)")
        );
        assert!(!s.contains("GW;;;S-1-15") && !s.contains("GA;;;S-1-15"));
        assert!(s.ends_with("S:(ML;;NW;;;LW)"));
        // FILE_APPEND_DATA (= FILE_CREATE_PIPE_INSTANCE, 0x4) và WRITE_DAC/WRITE_OWNER không được cấp.
        assert_eq!(
            SANDBOX_PIPE_RIGHTS & 0x4,
            0,
            "sandbox không được tạo instance pipe"
        );
        assert_eq!(SANDBOX_PIPE_RIGHTS & 0x000C_0000, 0);
        // Quyền client xin phải nằm trong quyền DACL cấp.
        assert_eq!(SANDBOX_CLIENT_ACCESS & !SANDBOX_PIPE_RIGHTS, 0);
    }

    #[test]
    fn sandbox_dir_sddl_is_protected_and_read_only_for_sandbox() {
        let s = sandbox_dir_sddl("S-1-5-21-9");
        assert!(
            s.starts_with("D:P"),
            "DACL phải bảo vệ (không kế thừa %APPDATA%)"
        );
        assert!(
            s.contains("(A;OICI;FRFX;;;S-1-15-2-1)") && s.contains("(A;OICI;FRFX;;;S-1-15-2-2)")
        );
        assert!(s.contains("(A;OICI;FA;;;S-1-5-21-9)"));
        assert!(!s.contains("FA;;;S-1-15"), "sandbox chỉ được đọc");
    }

    #[test]
    fn sddl_grants_sandbox_needs_both_ac_and_lpac() {
        assert!(sddl_grants_sandbox(
            "D:P(A;OICI;FRFX;;;AC)(A;OICI;FRFX;;;S-1-15-2-2)"
        ));
        assert!(sddl_grants_sandbox(
            "D:(A;;FR;;;S-1-15-2-1)(A;;FR;;;S-1-15-2-2)"
        ));
        assert!(!sddl_grants_sandbox("D:P(A;OICI;FRFX;;;AC)"), "thiếu LPAC");
        assert!(!sddl_grants_sandbox("D:(A;;FA;;;S-1-5-21-1)"));
    }

    /// R2-110: server phải đúng TÊN FILE `TextVN.exe` — trước chỉ so đuôi chuỗi.
    #[test]
    fn server_image_must_be_textvn_exe_by_file_name() {
        assert!(image_is_textvn_exe(r"C:\Program Files\TextVN\TextVN.exe"));
        assert!(image_is_textvn_exe(
            r"C:\Users\a\AppData\Local\Programs\TextVN\textvn.EXE"
        ));
        assert!(image_is_textvn_exe("TextVN.exe"));
        assert!(!image_is_textvn_exe(r"C:\evil\notTextVN.exe"));
        assert!(!image_is_textvn_exe(r"C:\evil\TextVN.exe.bak"));
        assert!(!image_is_textvn_exe(r"C:\TextVN.exe\other.exe"));
        assert!(!image_is_textvn_exe(""));
    }

    #[cfg(windows)]
    #[test]
    fn sandbox_dir_candidates_start_with_appdata_and_are_unique() {
        let c = sandbox_dir_candidates();
        assert!(!c.is_empty());
        if let Some(appdata) = std::env::var_os("APPDATA") {
            assert_eq!(c[0], PathBuf::from(appdata).join("TextVN").join("sandbox"));
        }
        let mut lower: Vec<String> = c
            .iter()
            .map(|p| p.to_string_lossy().to_ascii_lowercase())
            .collect();
        lower.sort();
        lower.dedup();
        assert_eq!(lower.len(), c.len(), "ứng viên trùng lặp");
    }

    #[cfg(windows)]
    #[test]
    fn read_sandbox_file_uses_first_readable_candidate() {
        let root = std::env::temp_dir().join(format!("textvn-ipc-sbx-{}", std::process::id()));
        let a = root.join("a");
        let b = root.join("b");
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(b.join("state.json"), b"{\"global_enabled\":false}").unwrap();
        let got = read_sandbox_file_in(&[a, b], "state.json").unwrap();
        assert_eq!(got, b"{\"global_enabled\":false}");
        assert!(read_sandbox_file_in(&[root.join("none")], "state.json").is_none());
        let _ = std::fs::remove_dir_all(&root);
    }
}
