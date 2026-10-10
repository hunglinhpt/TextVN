// SPDX-License-Identifier: GPL-3.0-or-later
//! Đăng ký và hủy đăng ký TSF TIP (WIN-003 / WIN-010 / P1-1 §8).
//!
//! Quy trình (mỗi bước độc lập — một bước lỗi không chặn bước sau):
//!   1. COM server `...\Software\Classes\CLSID\{CLSID}\InprocServer32` (HKCU, hoặc HKLM
//!      với `--scope machine`) + ACL cho AppContainer đọc DLL.
//!   2. Profile + category qua `ITfInputProcessorProfileMgr::RegisterProfile` /
//!      `ITfCategoryMgr::RegisterCategory` (ghi HKLM). Không có quyền admin → fallback
//!      layout CTF per-user dưới `HKCU\Software\Microsoft\CTF\TIP\{CLSID}` (P1-1 §8).
//!   3. `InstallLayoutOrTip` (input.dll, HKCU) — thêm vào danh sách bàn phím của user.
//!   4. `ActivateProfile(..., TF_IPPMF_FORSESSION)` — dùng được ngay.
//!
//! Tham khảo: `spikes/tsf-min/src/register.rs` + `docs/specs/tsf-registration-spike.md`.

use std::path::{Path, PathBuf};

// ─── CLSID / Profile strings (text-only; dùng được trên cả non-Windows cho tests) ──────────────

/// CLSID của TextVN TIP — khớp với `adapters/windows-tsf/src/guids.rs`.
#[cfg_attr(not(windows), allow(dead_code))] // non-Windows: chỉ test dùng (bin không gọi)
pub const CLSID_STR: &str = "{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}";
/// CLSID không ngoặc nhọn — dựng value name cho modern language list.
#[cfg_attr(not(windows), allow(dead_code))]
const CLSID_INNER: &str = "6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8";
/// Profile GUID của TextVN TIP.
#[cfg_attr(not(windows), allow(dead_code))]
pub const PROFILE_STR: &str = "{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}";
/// Profile GUID không ngoặc nhọn — dựng value name cho modern language list.
#[cfg_attr(not(windows), allow(dead_code))]
const PROFILE_INNER: &str = "C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13";

#[cfg_attr(not(windows), allow(dead_code))]
pub const LANGID_VI: u16 = 0x042A; // vi-VN
#[cfg_attr(not(windows), allow(dead_code))]
pub const LANGID_EN: u16 = 0x0409; // en-US

/// Ngôn ngữ đăng ký TextVN. Cả en-US: Windows tiếng Anh (phần lớn máy) dùng được TextVN
/// ngay sau khi cài; chỉ vi-VN thì app mới mở vẫn chạy bàn phím US (test gõ thật trên
/// Windows). Nằm cạnh bàn phím US nghĩa là phím tắt đổi bố cục Ctrl + Shift của Windows
/// nhảy qua lại giữa hai bàn phím — "Dành Ctrl + Shift cho TextVN" (`tray/src/hotkey.rs`).
#[cfg_attr(not(windows), allow(dead_code))]
pub const REGISTER_LANGS: [(&str, u16); 2] = [("VI", LANGID_VI), ("EN", LANGID_EN)];

// ─── Helpers (cross-platform phần text) ──────────────────────────────────────────────────────────

/// `HKCU\Software\Classes\CLSID\{CLSID}` — registry key đăng ký COM per-user.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn clsid_registry_key() -> String {
    format!(r"HKCU\Software\Classes\CLSID\{CLSID_STR}")
}

/// Format spec `InstallLayoutOrTip`: `"0x{lang:04X}:{CLSID}{Profile}"`.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn layout_spec(lang: u16) -> String {
    format!("0x{lang:04X}:{CLSID_STR}{PROFILE_STR}")
}

/// Tìm đường dẫn DLL `textvn-tsf.dll` (hoặc `textvn_win_tsf.dll`).
///
/// Ưu tiên:
/// 1. `custom` nếu được cung cấp và file tồn tại.
/// 2. Cùng thư mục với `textvn.exe` (cạnh CLI binary).
/// 3. Lỗi nếu không tìm thấy.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn resolve_dll_path(custom: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(p) = custom {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
        return Err(format!("DLL không tồn tại: {}", p.display()));
    }

    // Thư mục chứa binary hiện tại
    let mut dir =
        std::env::current_exe().map_err(|e| format!("Không xác định được current_exe: {e}"))?;
    dir.pop();

    for name in &["textvn-tsf.dll", "textvn_win_tsf.dll"] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Không tìm thấy textvn-tsf.dll (hoặc textvn_win_tsf.dll) trong {}\n\
         Dùng --dll <path> để chỉ định thủ công hoặc chạy `cargo build` trước.",
        dir.display()
    ))
}

// ─── Ghi log ra console + file ───────────────────────────────────────────────────────────────────

#[cfg_attr(not(windows), allow(dead_code))]
fn say(msg: &str) {
    println!("{msg}");
    // Ghi log vào %LOCALAPPDATA%\TextVN\logs\register.log (output khi elevated bị ẩn console)
    if let Some(base) = std::env::var_os("LOCALAPPDATA") {
        if let Some(file) = prepare_log_file(Path::new(&base)) {
            append_log_line(&file, &format!("[pid={}] {msg}", std::process::id()));
        }
    }
}

/// Giới hạn `register.log`: vượt ngưỡng thì xoay sang `register.log.1` (R2-21 —
/// trước đây log append vô hạn qua mọi lần chạy).
#[cfg_attr(not(windows), allow(dead_code))]
const LOG_ROTATE_BYTES: u64 = 1024 * 1024;

/// Chuẩn bị `<base>\TextVN\logs\register.log` cho CLI có thể đang chạy ELEVATED
/// (đăng ký phạm vi máy, bộ cài) trong thư mục người dùng ghi được (CWE-59).
///
/// R2-21: kiểm tra reparse point TRƯỚC khi tạo bất cứ thứ gì — bản cũ gọi
/// `create_dir_all` trước nên junction ở `%LOCALAPPDATA%\TextVN` khiến tiến trình
/// elevated tạo `logs` ở vị trí bị điều hướng rồi mới từ chối. Mỗi cấp được tạo
/// bằng `create_dir` (không đi theo link khi đã tồn tại) và kiểm tra lại ngay sau.
/// `None` = bỏ ghi log file (stdout vẫn có).
#[cfg_attr(not(windows), allow(dead_code))]
fn prepare_log_file(base: &Path) -> Option<PathBuf> {
    let app = base.join("TextVN");
    let dir = app.join("logs");
    for level in [&app, &dir] {
        if is_reparse_link(level) {
            return None;
        }
        match std::fs::create_dir(level) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return None,
        }
        if is_reparse_link(level) || !level.is_dir() {
            return None;
        }
    }
    let file = dir.join("register.log");
    if is_reparse_link(&file) {
        return None;
    }
    rotate_log_if_larger(&file, LOG_ROTATE_BYTES);
    Some(file)
}

/// Xoay `file` → `file.1` khi lớn hơn `max_bytes` (best-effort; lỗi thì giữ nguyên).
#[cfg_attr(not(windows), allow(dead_code))]
fn rotate_log_if_larger(file: &Path, max_bytes: u64) {
    let Ok(meta) = std::fs::symlink_metadata(file) else {
        return;
    };
    if !meta.is_file() || meta.len() <= max_bytes {
        return;
    }
    let mut rotated = file.as_os_str().to_owned();
    rotated.push(".1");
    let _ = std::fs::rename(file, PathBuf::from(rotated));
}

/// Append một dòng; trên Windows mở bằng `FILE_FLAG_OPEN_REPARSE_POINT` và từ chối
/// ghi nếu handle là reparse point (link được cài vào giữa kiểm tra và mở).
#[cfg_attr(not(windows), allow(dead_code))]
fn append_log_line(file: &Path, line: &str) {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        opts.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let Ok(mut f) = opts.open(file) else {
        return;
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        match f.metadata() {
            Ok(m) if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0 => {}
            _ => return,
        }
    }
    let _ = writeln!(f, "{line}");
}

/// Symlink hoặc junction (Windows: reparse point "name surrogate") — không đi theo link.
fn is_reparse_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

// ─── Kiểm tra vị trí DLL / quyền sở hữu đăng ký (thuần — test được mọi OS) ─────────────────────────

/// Chuẩn hoá đường dẫn Windows để so tiền tố: bỏ ngoặc kép, `/` → `\`, bỏ tiền tố
/// `\\?\`, bỏ `\` cuối, chữ thường.
#[cfg_attr(not(windows), allow(dead_code))]
fn normalize_win_path(p: &str) -> String {
    let p = p.trim().trim_matches('"').replace('/', "\\");
    let p = p.strip_prefix(r"\\?\").unwrap_or(&p);
    p.trim_end_matches('\\').to_lowercase()
}

/// `path` nằm **dưới** `root` theo ranh giới thư mục (không phải "chứa chuỗi").
#[cfg_attr(not(windows), allow(dead_code))]
pub fn path_is_under(path: &str, root: &str) -> bool {
    let p = normalize_win_path(path);
    let r = normalize_win_path(root);
    !r.is_empty() && p.len() > r.len() && p.starts_with(&r) && p.as_bytes()[r.len()] == b'\\'
}

/// R2-03/R2-15: đăng ký **phạm vi máy** (HKLM — mọi tài khoản, cả tiến trình
/// elevated, nạp DLL này) chỉ được phép khi DLL đã canonicalize nằm dưới Program
/// Files (known folder, chỉ admin ghi được) và KHÔNG dưới `WindowsApps` (đường dẫn
/// package đổi theo version, bị xoá sau mỗi lần Store cập nhật).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn machine_scope_dll_allowed(canonical_dll: &str, program_files_roots: &[String]) -> bool {
    !normalize_win_path(canonical_dll).contains(r"\windowsapps\")
        && program_files_roots
            .iter()
            .any(|root| path_is_under(canonical_dll, root))
}

/// Bỏ tiền tố verbatim `\\?\` của `canonicalize` (ghi registry dạng `C:\...` như
/// mọi đường dẫn InprocServer32 khác). `\\?\UNC\srv\share` → `\\srv\share`.
#[cfg_attr(not(windows), allow(dead_code))]
fn strip_verbatim_prefix(p: &str) -> String {
    if let Some(rest) = p.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else {
        p.strip_prefix(r"\\?\").unwrap_or(p).to_string()
    }
}

/// R2-30: `unregister --if-owned-by <dir>` chỉ gỡ khi đăng ký TIP hiện tại thuộc
/// về `<dir>`: không có đăng ký, hoặc DLL đăng ký không còn tồn tại (mồ côi), hoặc
/// DLL nằm dưới `<dir>`. Đăng ký đang trỏ một bản TextVN KHÁC (bản cài, Store) thì
/// giữ nguyên — gỡ thư mục portable cũ không được tắt bộ gõ của bản đang dùng.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn registration_owned_by_dir(
    server: Option<&str>,
    server_exists: bool,
    dirs: &[String],
) -> bool {
    match server {
        None => true,
        Some(_) if !server_exists => true,
        Some(path) => dirs.iter().any(|d| path_is_under(path, d)),
    }
}

// ─── R2-100: ngôn ngữ do chính TextVN thêm vào danh sách của user ─────────────────

/// Tag trong `HKCU\Control Panel\International\User Profile` ứng với hai LANGID TextVN
/// đăng ký (0x042A, 0x0409).
#[cfg_attr(not(windows), allow(dead_code))]
pub const PROFILE_LANGUAGE_TAGS: [&str; 2] = ["vi", "en-US"];

/// Marker `%APPDATA%\TextVN\languages_added`: mỗi dòng một tag ngôn ngữ mà `register`
/// đã THÊM vào danh sách ngôn ngữ của user (trước đó không có). `InstallLayoutOrTip`
/// tự thêm ngôn ngữ khi gắn bàn phím cho ngôn ngữ chưa có, còn gỡ TIP chỉ bỏ bàn phím —
/// máy chỉ có tiếng Anh cài rồi gỡ TextVN từng còn sót "Tiếng Việt" (Windows tự gắn
/// bàn phím mặc định vào) trong Settings › Language.
#[cfg_attr(not(windows), allow(dead_code))]
pub const LANGUAGES_ADDED_MARKER: &str = "languages_added";

/// Tag trong `tags` mà danh sách ngôn ngữ `before` (value `Languages`) chưa có.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn languages_missing(before: &[String], tags: &[&str]) -> Vec<String> {
    tags.iter()
        .filter(|t| !before.iter().any(|b| b.trim().eq_ignore_ascii_case(t)))
        .map(|t| t.to_string())
        .collect()
}

/// Value trong `User Profile\<tag>` là một bàn phím/TIP (`042A:{…}{…}`, `0409:00000409`),
/// không phải `CachedLanguageName` hay value khác.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn is_input_method_value(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() > 5 && b[4] == b':' && b[..4].iter().all(u8::is_ascii_hexdigit)
}

/// Ngôn ngữ TextVN đã thêm được gỡ hẳn khi, NGAY TRƯỚC lúc gỡ TIP, nó không có bàn phím
/// nào khác ngoài TextVN (`textvn_value`). Bàn phím Windows tự gắn vào trong lúc gỡ
/// (bỏ bàn phím cuối của một ngôn ngữ) không phải lựa chọn của người dùng.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn added_language_removable(values_before_unregister: &[String], textvn_value: &str) -> bool {
    !values_before_unregister
        .iter()
        .any(|v| is_input_method_value(v) && !v.eq_ignore_ascii_case(textvn_value))
}

/// Danh sách `Languages` bỏ `tag` (giữ thứ tự). Không bao giờ trả danh sách rỗng —
/// bỏ ngôn ngữ cuối cùng thì giữ nguyên.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn languages_without(list: &[String], tag: &str) -> Vec<String> {
    let out: Vec<String> = list
        .iter()
        .filter(|l| !l.trim().eq_ignore_ascii_case(tag))
        .cloned()
        .collect();
    if out.is_empty() {
        list.to_vec()
    } else {
        out
    }
}

/// Đọc marker: chỉ nhận tag TextVN quản lý (file người dùng sửa được), bỏ trùng.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn parse_languages_marker(content: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in content.lines().map(str::trim) {
        if let Some(tag) = PROFILE_LANGUAGE_TAGS
            .iter()
            .find(|t| t.eq_ignore_ascii_case(line))
        {
            if !out.iter().any(|o| o == tag) {
                out.push(tag.to_string());
            }
        }
    }
    out
}

// ─── Layout registry TSF (text-only — test được mọi OS) ───────────────────────────────────────────

/// `GUID_TFCAT_TIP_KEYBOARD` — TIP bàn phím.
#[cfg_attr(not(windows), allow(dead_code))]
pub const CAT_TIP_KEYBOARD: &str = "{34745C63-B2F0-4784-8B67-5E12C8701A31}";
/// `GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT` — cho phép TIP nạp trong app immersive/AppContainer
/// (ô tìm kiếm Start, Settings, app Store). Thiếu category này = không gõ được ở đó.
#[cfg_attr(not(windows), allow(dead_code))]
pub const CAT_IMMERSIVE: &str = "{13A016DF-560B-46CD-947A-4C3AF1E0E35D}";
/// `GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT` — hiện trong input indicator của taskbar (Win8+).
#[cfg_attr(not(windows), allow(dead_code))]
pub const CAT_SYSTRAY: &str = "{25504FB4-7BAB-4BC1-9C69-CF81890F0EF5}";
/// `GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER` — cho phép TIP cung cấp thuộc tính hiển thị (tắt gạch chân).
/// Phải khớp `windows::Win32::UI::TextServices::GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER`
/// (đã từng ghi sai `{2464BEB0-…}` — fallback per-user đăng ký category rác,
/// provider không bao giờ được TSF nhận diện).
#[cfg_attr(not(windows), allow(dead_code))]
pub const CAT_DISPLAY_ATTRIBUTE_PROVIDER: &str = "{046B8C80-1647-40F7-9B21-B93B81AABC1B}";

/// Khóa TIP của CTF, tương đối với HKLM\SOFTWARE hoặc HKCU\Software.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn ctf_tip_key() -> String {
    format!(r"Software\Microsoft\CTF\TIP\{CLSID_STR}")
}

/// `...\LanguageProfile\0x0000042a\{PROFILE}` — nơi chứa Description/IconFile/Enable.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn ctf_profile_key(lang: u16) -> String {
    format!(
        r"{}\LanguageProfile\0x{:08x}\{PROFILE_STR}",
        ctf_tip_key(),
        u32::from(lang)
    )
}

/// Cặp khóa category CTF (`Category\Category\{cat}\{clsid}` + `Category\Item\{clsid}\{cat}`).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn ctf_category_keys(cat: &str) -> [String; 2] {
    let tip = ctf_tip_key();
    [
        format!(r"{tip}\Category\Category\{cat}\{CLSID_STR}"),
        format!(r"{tip}\Category\Item\{CLSID_STR}\{cat}"),
    ]
}

// ─── Windows-only COM/TSF impl ────────────────────────────────────────────────────────────────────

#[cfg(windows)]
mod win_impl {
    use super::*;

    use windows::core::*;
    use windows::Win32::Foundation::FreeLibrary;
    use windows::Win32::Foundation::*;
    use windows::Win32::Security::Authorization::*;
    use windows::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, NO_INHERITANCE, PSECURITY_DESCRIPTOR, PSID,
    };
    use windows::Win32::System::Com::*;
    use windows::Win32::System::LibraryLoader::{
        GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
    };
    use windows::Win32::System::Registry::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
    use windows::Win32::UI::TextServices::*;

    // Freeze GUIDs — khớp adapters/windows-tsf/src/guids.rs
    pub const CLSID_TIP: GUID = GUID::from_u128(0x6F2B9C31_8E47_4D2A_9C84_1D5A3E70F9B8);
    pub const PROFILE_GUID: GUID = GUID::from_u128(0xC4A91F52_77B3_4E19_8A6D_2F8C0B6E5A13);

    /// `InstallLayoutOrTip` flag ILOT_UNINSTALL — hủy đăng ký khỏi danh sách bàn phím người dùng.
    const ILOT_UNINSTALL: u32 = 0x0000_0001;
    /// `InstallLayoutOrTip` flag ILOT_DEFPROFILE — đặt profile làm default.
    const ILOT_DEFPROFILE: u32 = 0x0000_0002;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Scope {
        User,
        Machine,
    }

    impl Scope {
        fn root(self) -> HKEY {
            match self {
                Scope::User => HKEY_CURRENT_USER,
                Scope::Machine => HKEY_LOCAL_MACHINE,
            }
        }
        fn label(self) -> &'static str {
            match self {
                Scope::User => "HKCU",
                Scope::Machine => "HKLM",
            }
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    fn root_label(h: HKEY) -> &'static str {
        if h == HKEY_CURRENT_USER {
            "HKCU"
        } else if h == HKEY_LOCAL_MACHINE {
            "HKLM"
        } else {
            "REG"
        }
    }

    /// Tên ký hiệu của mã lỗi Win32 registry thường gặp. Log chỉ in `0x00000005`
    /// thì hộp thoại tray không ghép được gợi ý theo nguyên nhân (nhánh advice
    /// `ACCESS_DENIED` thành dead code) — dòng log phải mang cả tên ký hiệu.
    pub fn lstatus_name(code: u32) -> Option<&'static str> {
        Some(match code {
            2 => "ERROR_FILE_NOT_FOUND",
            3 => "ERROR_PATH_NOT_FOUND",
            5 => "ERROR_ACCESS_DENIED",
            6 => "ERROR_INVALID_HANDLE",
            32 => "ERROR_SHARING_VIOLATION",
            87 => "ERROR_INVALID_PARAMETER",
            183 => "ERROR_ALREADY_EXISTS",
            _ => return None,
        })
    }

    /// `0x00000005 (ERROR_ACCESS_DENIED)` — hex cho máy đọc, tên cho người/người máy đọc.
    fn lstatus_text(code: u32) -> String {
        match lstatus_name(code) {
            Some(name) => format!("{code:#010x} ({name})"),
            None => format!("{code:#010x}"),
        }
    }

    /// `Software\Classes\CLSID\{CLSID}` — tương đối với HKCU hoặc HKLM tùy scope.
    fn clsid_key() -> String {
        clsid_registry_key()
            .trim_start_matches(r"HKCU\")
            .to_string()
    }

    /// Ghi registry qua Win32 API, không spawn `reg.exe` (AV soi child process
    /// sửa registry; lỗi ACL cũng phân biệt được).
    fn set_reg_value(root: HKEY, path: &str, name: Option<&str>, value: RegValue<'_>) -> bool {
        let subkey = wide(path);
        let mut key = HKEY::default();
        // SAFETY: con trỏ tới buffer UTF-16 có nul; key được đóng ngay bên dưới.
        let create = unsafe {
            RegCreateKeyExW(
                root,
                PCWSTR(subkey.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        };
        if create != ERROR_SUCCESS {
            say(&format!(
                "  Registry create {}\\{} → FAIL {}",
                root_label(root),
                path,
                lstatus_text(create.0)
            ));
            return false;
        }
        let status = match value {
            RegValue::None => ERROR_SUCCESS,
            RegValue::Sz(text) => {
                let data = wide(text);
                // SAFETY: đọc đúng kích thước của `data` dưới dạng byte.
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr() as *const u8,
                        std::mem::size_of_val(data.as_slice()),
                    )
                };
                let name_w = name.map(wide);
                let name_ptr = name_w
                    .as_ref()
                    .map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
                // SAFETY: key mở ở trên; buffer hợp lệ.
                unsafe { RegSetValueExW(key, name_ptr, None, REG_SZ, Some(bytes)) }
            }
            RegValue::Dword(v) => {
                let bytes = v.to_le_bytes();
                let name_w = name.map(wide);
                let name_ptr = name_w
                    .as_ref()
                    .map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
                // SAFETY: key mở ở trên; 4 byte DWORD.
                unsafe { RegSetValueExW(key, name_ptr, None, REG_DWORD, Some(&bytes)) }
            }
            RegValue::MultiSz(items) => {
                // REG_MULTI_SZ: mỗi chuỗi kết thúc nul, cả khối kết thúc thêm một nul.
                let mut data: Vec<u16> = Vec::new();
                for item in items {
                    data.extend(item.encode_utf16());
                    data.push(0);
                }
                data.push(0);
                // SAFETY: đọc đúng kích thước của `data` dưới dạng byte.
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr() as *const u8,
                        std::mem::size_of_val(data.as_slice()),
                    )
                };
                let name_w = name.map(wide);
                let name_ptr = name_w
                    .as_ref()
                    .map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
                // SAFETY: key mở ở trên; buffer hợp lệ.
                unsafe { RegSetValueExW(key, name_ptr, None, REG_MULTI_SZ, Some(bytes)) }
            }
        };
        // SAFETY: key hợp lệ.
        let _ = unsafe { RegCloseKey(key) };
        if status != ERROR_SUCCESS {
            say(&format!(
                "  Registry write {}\\{} → FAIL {}",
                root_label(root),
                path,
                lstatus_text(status.0)
            ));
            return false;
        }
        true
    }

    enum RegValue<'a> {
        None,
        Sz(&'a str),
        Dword(u32),
        MultiSz(&'a [String]),
    }

    /// Xoá MỘT value (giữ key) — dùng cho dọn entry danh sách ngôn ngữ hiện đại.
    fn delete_reg_value(root: HKEY, path: &str, name: &str) -> bool {
        let subkey = wide(path);
        let mut key = HKEY::default();
        // SOURCE: RegOpenKeyExW + RegDeleteValueW (Win32 registry API).
        let open = unsafe {
            RegOpenKeyExW(
                root,
                PCWSTR(subkey.as_ptr()),
                Some(0),
                KEY_SET_VALUE,
                &mut key,
            )
        };
        if open != ERROR_SUCCESS {
            return false;
        }
        let name_w = wide(name);
        // SAFETY: key hợp lệ; name nul-terminated.
        let status = unsafe { RegDeleteValueW(key, PCWSTR(name_w.as_ptr())) };
        let _ = unsafe { RegCloseKey(key) };
        status == ERROR_SUCCESS
    }

    pub fn reg_key_exists(root: HKEY, path: &str) -> bool {
        let subkey = wide(path);
        let mut key = HKEY::default();
        // SAFETY: buffer nul-terminated; key đóng ngay nếu mở được.
        let status =
            unsafe { RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), None, KEY_READ, &mut key) };
        if status == ERROR_SUCCESS {
            let _ = unsafe { RegCloseKey(key) };
            true
        } else {
            false
        }
    }

    pub fn reg_read_string(root: HKEY, path: &str) -> Option<String> {
        let subkey = wide(path);
        let mut buf = [0u16; 1024];
        let mut size = std::mem::size_of_val(&buf) as u32;
        // SAFETY: buffer và kích thước khớp nhau; RRF_RT_REG_SZ đảm bảo kết thúc nul.
        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
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

    /// Đọc giá trị REG_SZ có tên `name` trong `path` (`None` = không có).
    pub fn reg_read_named(root: HKEY, path: &str, name: &str) -> Option<String> {
        let subkey = wide(path);
        let value = wide(name);
        let mut buf = [0u16; 256];
        let mut size = std::mem::size_of_val(&buf) as u32;
        // SAFETY: buffer và kích thước khớp nhau; RRF_RT_REG_SZ đảm bảo kết thúc nul.
        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value.as_ptr()),
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
        Some(String::from_utf16_lossy(&buf[..len]).trim().to_string())
    }

    /// Đọc DWORD có tên trong registry. `Enable=1` của fallback HKCU là một
    /// postcondition bắt buộc: chỉ có key CTF gốc không có nghĩa Windows đã có
    /// profile TIP dùng được.
    fn reg_read_dword(root: HKEY, path: &str, name: &str) -> Option<u32> {
        let subkey = wide(path);
        let value = wide(name);
        let mut out = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        // SAFETY: buffer `out` có đúng bốn byte, tên/key UTF-16 kết thúc nul.
        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_DWORD,
                None,
                Some((&mut out as *mut u32).cast()),
                Some(&mut size),
            )
        };
        (status == ERROR_SUCCESS && size == std::mem::size_of::<u32>() as u32).then_some(out)
    }

    /// Đọc REG_MULTI_SZ có tên (`None` = không có / không đọc được).
    fn reg_read_multi_sz(root: HKEY, path: &str, name: &str) -> Option<Vec<String>> {
        let subkey = wide(path);
        let value = wide(name);
        let mut size = 0u32;
        // SAFETY: hỏi kích thước trước (buffer NULL), rồi đọc vào buffer đúng cỡ.
        let probe = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_MULTI_SZ,
                None,
                None,
                Some(&mut size),
            )
        };
        if probe != ERROR_SUCCESS || size == 0 {
            return None;
        }
        let mut buf = vec![0u16; (size as usize).div_ceil(2) + 1];
        let mut size = (buf.len() * 2) as u32;
        // SAFETY: buffer và kích thước khớp nhau.
        let status = unsafe {
            RegGetValueW(
                root,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_MULTI_SZ,
                None,
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut size),
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let used = (size as usize / 2).min(buf.len());
        Some(
            buf[..used]
                .split(|&c| c == 0)
                .filter(|s| !s.is_empty())
                .map(String::from_utf16_lossy)
                .collect(),
        )
    }

    /// Tên mọi value trong `path` (key không có → rỗng).
    fn reg_value_names(root: HKEY, path: &str) -> Vec<String> {
        let subkey = wide(path);
        let mut key = HKEY::default();
        // SAFETY: key được đóng ở cuối hàm.
        let open = unsafe {
            RegOpenKeyExW(
                root,
                PCWSTR(subkey.as_ptr()),
                Some(0),
                KEY_QUERY_VALUE,
                &mut key,
            )
        };
        if open != ERROR_SUCCESS {
            return Vec::new();
        }
        let mut names = Vec::new();
        for index in 0u32.. {
            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;
            // SAFETY: buffer tên + độ dài khớp nhau; không đọc data.
            let status = unsafe {
                RegEnumValueW(
                    key,
                    index,
                    Some(PWSTR(buf.as_mut_ptr())),
                    &mut len,
                    None,
                    None,
                    None,
                    None,
                )
            };
            if status != ERROR_SUCCESS {
                break;
            }
            names.push(String::from_utf16_lossy(&buf[..len as usize]));
        }
        // SAFETY: key hợp lệ.
        let _ = unsafe { RegCloseKey(key) };
        names
    }

    /// Xoá key + trả true khi xoá được hoặc key không tồn tại. Caller dùng giá
    /// trị này để không báo unregister thành công khi còn key sót.
    fn delete_tree(root: HKEY, path: &str) -> bool {
        let subkey = wide(path);
        // SAFETY: buffer nul-terminated.
        let status = unsafe { RegDeleteTreeW(root, PCWSTR(subkey.as_ptr())) };
        let gone = status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND;
        if gone {
            say(&format!(
                "  Registry delete {}\\{} → OK",
                root_label(root),
                path
            ));
        } else {
            say(&format!(
                "  Registry delete {}\\{} → FAIL {}",
                root_label(root),
                path,
                lstatus_text(status.0)
            ));
        }
        gone
    }

    /// `textvn-cli schedule-delete <path...>` — xoá ngay nếu được, trượt thì hẹn
    /// Windows xoá trước phiên khởi động kế tiếp (`MoveFileExW(path, NULL, 4)`).
    /// Dùng cho uninstaller của bộ cài (chạy elevated — PFO cần admin): DLL bị
    /// TSF nạp trong app đang mở không thể `DeleteFile` (B12). Thứ tự tham số
    /// được giữ nguyên trong PFO nên caller truyền file trước, thư mục sau.
    pub fn schedule_delete(paths: &[String]) -> i32 {
        use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};
        let mut all_ok = true;
        for path in paths {
            let p = Path::new(path);
            if !p.exists() {
                say(&format!("  {path} → không tồn tại (bỏ qua)"));
                continue;
            }
            // Thử xoá trực tiếp trước (thư mục phải rỗng — gọi sau các file trong nó).
            let direct = if p.is_dir() {
                std::fs::remove_dir(p).is_ok()
            } else {
                std::fs::remove_file(p).is_ok()
            };
            if direct {
                say(&format!("  {path} → đã xoá"));
                continue;
            }
            let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            // SAFETY: chuỗi nul-terminated; lpNewFileName = NULL = xoá khi reboot.
            let r = unsafe {
                MoveFileExW(
                    windows::core::PCWSTR(wide.as_ptr()),
                    windows::core::PCWSTR::null(),
                    MOVEFILE_DELAY_UNTIL_REBOOT,
                )
            };
            match r {
                Ok(()) => say(&format!("  {path} → sẽ xoá ở lần khởi động kế tiếp")),
                Err(e) => {
                    say(&format!("  {path} → FAIL {:#010x}", e.code().0));
                    all_ok = false;
                }
            }
        }
        if all_ok {
            0
        } else {
            1
        }
    }

    pub fn com_init() -> bool {
        // SAFETY: khởi tạo COM STA cho thread chính của CLI.
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        // S_FALSE (đã init rồi) hoặc RPC_E_CHANGED_MODE → vẫn dùng được
        hr.0 >= 0 || hr.0 == (0x8001_0106u32 as i32)
    }

    /// `InstallLayoutOrTip` (input.dll) — thêm/gỡ TIP khỏi danh sách bàn phím của
    /// user (HKCU, không cần admin). Không có import lib nên resolve động, nhưng
    /// CHỈ từ System32 (LOAD_LIBRARY_SEARCH_SYSTEM32) để không nạp nhầm DLL giả mạo.
    fn call_layout_or_tip(lang: u16, flags: u32, label: &str) -> bool {
        call_layout_or_tip_spec(&layout_spec(lang), flags, label)
    }

    /// Như [`call_layout_or_tip`] với chuỗi profile bất kỳ (`042A:{…}{…}`,
    /// `042A:0000042A`) — gỡ bàn phím Windows tự gắn khi bỏ ngôn ngữ TextVN đã thêm.
    fn call_layout_or_tip_spec(spec: &str, flags: u32, label: &str) -> bool {
        // SAFETY: input.dll là DLL hệ thống; chữ ký hàm theo tài liệu Microsoft
        // `BOOL InstallLayoutOrTip(LPCWSTR psz, DWORD dwFlags)`.
        unsafe {
            let hmod = match LoadLibraryExW(w!("input.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32) {
                Ok(h) => h,
                Err(e) => {
                    say(&format!(
                        "  InstallLayoutOrTip: nạp input.dll FAIL {:#010x}",
                        e.code().0
                    ));
                    return false;
                }
            };
            let Some(fp) = GetProcAddress(hmod, s!("InstallLayoutOrTip")) else {
                say("  InstallLayoutOrTip: không có trong input.dll");
                let _ = FreeLibrary(hmod);
                return false;
            };
            type Pfn = unsafe extern "system" fn(*const u16, u32) -> i32;
            let pfn: Pfn = std::mem::transmute(fp);
            let spec_w = wide(spec);
            let ok = pfn(spec_w.as_ptr(), flags) != 0;
            // Trả refcount input.dll ngay (CLI gọi 2–4 lần/process; không trả
            // thì mỗi lần gọi leak một refcount).
            let _ = FreeLibrary(hmod);
            say(&format!(
                "  InstallLayoutOrTip({spec}, {label}) → {}",
                if ok { "OK" } else { "FAIL" }
            ));
            ok
        }
    }

    /// Icon cho Win+Space / input indicator: TextVN.exe cạnh DLL nếu có.
    fn icon_path(dll_path: &Path) -> String {
        dll_path
            .parent()
            .map(|d| d.join("TextVN.exe"))
            .filter(|p| p.is_file())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Cho phép process AppContainer (app Store, Start search, Settings) đọc+nạp DLL.
    /// Cài per-user nằm trong %LOCALAPPDATA% — mặc định AppContainer không đọc được,
    /// nên TIP không nạp và người dùng không gõ được tiếng Việt ở các ô đó.
    fn grant_appcontainer_read(dll_path: &Path) {
        let path_w = wide(&dll_path.to_string_lossy());
        for sid_str in ["S-1-15-2-1", "S-1-15-2-2"] {
            // SAFETY: mọi con trỏ do API cấp được LocalFree đúng một lần; không
            // giữ tham chiếu sau khi giải phóng.
            unsafe {
                let mut sid = PSID::default();
                if ConvertStringSidToSidW(&HSTRING::from(sid_str), &mut sid).is_err() {
                    continue;
                }
                let mut old_dacl: *mut ACL = std::ptr::null_mut();
                let mut sd = PSECURITY_DESCRIPTOR::default();
                let got = GetNamedSecurityInfoW(
                    PCWSTR(path_w.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(&mut old_dacl),
                    None,
                    &mut sd,
                );
                if got == ERROR_SUCCESS {
                    let access = EXPLICIT_ACCESS_W {
                        grfAccessPermissions: (GENERIC_READ.0 | GENERIC_EXECUTE.0),
                        grfAccessMode: GRANT_ACCESS,
                        grfInheritance: NO_INHERITANCE,
                        Trustee: TRUSTEE_W {
                            TrusteeForm: TRUSTEE_IS_SID,
                            TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                            ptstrName: PWSTR(sid.0 as *mut u16),
                            ..Default::default()
                        },
                    };
                    let mut new_dacl: *mut ACL = std::ptr::null_mut();
                    if SetEntriesInAclW(Some(&[access]), Some(old_dacl), &mut new_dacl)
                        == ERROR_SUCCESS
                    {
                        let set = SetNamedSecurityInfoW(
                            PCWSTR(path_w.as_ptr()),
                            SE_FILE_OBJECT,
                            DACL_SECURITY_INFORMATION,
                            None,
                            None,
                            Some(new_dacl),
                            None,
                        );
                        say(&format!(
                            "  ACL {sid_str} đọc DLL → {}",
                            if set == ERROR_SUCCESS { "OK" } else { "FAIL" }
                        ));
                        let _ = LocalFree(Some(HLOCAL(new_dacl as *mut _)));
                    }
                    let _ = LocalFree(Some(HLOCAL(sd.0)));
                }
                let _ = LocalFree(Some(HLOCAL(sid.0)));
            }
        }
    }

    /// Đăng ký profile + category qua API TSF. Profile ghi HKLM (cần admin);
    /// RegisterCategory TSF tự quyết định nơi lưu và **thường thành công cả khi
    /// không admin** — vì vậy category được thử ĐỘC LẬP với profile: dừng sớm ở
    /// RegisterProfile từng làm portable non-admin mất category
    /// display-attribute provider → gạch chân không bao giờ tắt được.
    /// Trả `(profiles_ok, categories_ok)`.
    fn register_with_tsf_api(desc: &str, icon: &str) -> (bool, bool) {
        let desc_w = wide(desc);
        let icon_w = wide(icon);
        // Slice đúng độ dài nhưng allocation có nul đệm sau (S3-2: TSF đọc wcslen()).
        let desc_s = &desc_w[..desc_w.len() - 1];
        let icon_s = &icon_w[..icon_w.len() - 1];
        // SAFETY: COM đã init; mọi interface do CoCreateInstance trả về.
        let profiles = (|| -> Result<()> {
            unsafe {
                let mgr: ITfInputProcessorProfileMgr =
                    CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
                for (tag, lang) in REGISTER_LANGS {
                    mgr.RegisterProfile(
                        &CLSID_TIP,
                        lang,
                        &PROFILE_GUID,
                        desc_s,
                        icon_s,
                        0,
                        HKL::default(),
                        0,
                        true,
                        0,
                    )?;
                    say(&format!("  RegisterProfile({tag}) → OK"));
                }
                Ok(())
            }
        })();
        let profiles_ok = match profiles {
            Ok(()) => true,
            Err(e) => {
                say(&format!(
                    "  RegisterProfile qua API TSF → {:#010x} (fallback per-user sẽ ghi)",
                    e.code().0
                ));
                false
            }
        };
        let categories = (|| -> Result<()> {
            unsafe {
                let cat: ITfCategoryMgr =
                    CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
                for (name, guid) in [
                    ("TIP_KEYBOARD", GUID_TFCAT_TIP_KEYBOARD),
                    ("IMMERSIVESUPPORT", GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT),
                    ("SYSTRAYSUPPORT", GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT),
                    (
                        "DISPLAYATTRIBUTEPROVIDER",
                        GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                    ),
                ] {
                    cat.RegisterCategory(&CLSID_TIP, &guid, &CLSID_TIP)?;
                    say(&format!("  RegisterCategory({name}) → OK"));
                }
                Ok(())
            }
        })();
        let categories_ok = match categories {
            Ok(()) => true,
            Err(e) => {
                say(&format!(
                    "  RegisterCategory qua API TSF → {:#010x} (fallback ghi HKCU sau ILOT)",
                    e.code().0
                ));
                false
            }
        };
        (profiles_ok, categories_ok)
    }

    /// Fallback per-user (P1-1 §8): ghi đúng layout TIP của CTF dưới HKCU khi API
    /// không ghi được HKLM. Không đụng key của TIP khác. Category KHÔNG ghi ở đây:
    /// `InstallLayoutOrTip` chạy sau sẽ viết lại cây CTF và xoá mất — gọi
    /// [`register_ctf_categories_per_user`] SAU ILOT.
    fn register_ctf_per_user(desc: &str, icon: &str) -> bool {
        let mut ok = set_reg_value(HKEY_CURRENT_USER, &ctf_tip_key(), None, RegValue::None);
        for (_, lang) in REGISTER_LANGS {
            let key = ctf_profile_key(lang);
            ok &= set_reg_value(
                HKEY_CURRENT_USER,
                &key,
                Some("Description"),
                RegValue::Sz(desc),
            );
            ok &= set_reg_value(
                HKEY_CURRENT_USER,
                &key,
                Some("IconFile"),
                RegValue::Sz(icon),
            );
            ok &= set_reg_value(
                HKEY_CURRENT_USER,
                &key,
                Some("IconIndex"),
                RegValue::Dword(0),
            );
            ok &= set_reg_value(HKEY_CURRENT_USER, &key, Some("Enable"), RegValue::Dword(1));
        }
        say(&format!(
            "  CTF TIP per-user (HKCU) → {}",
            if ok { "OK" } else { "FAIL" }
        ));
        ok
    }

    /// Category CTF per-user (keyboard/immersive/systray/display-attr provider).
    /// PHẢI gọi SAU `InstallLayoutOrTip`: ILOT viết lại cây CTF TIP và xoá mất
    /// key category ghi trước đó (repro 2026-10-02 — category display-attribute
    /// biến mất làm portable không tắt được gạch chân).
    fn register_ctf_categories_per_user() -> bool {
        let mut ok = true;
        for cat in [
            CAT_TIP_KEYBOARD,
            CAT_IMMERSIVE,
            CAT_SYSTRAY,
            CAT_DISPLAY_ATTRIBUTE_PROVIDER,
        ] {
            for key in ctf_category_keys(cat) {
                ok &= set_reg_value(HKEY_CURRENT_USER, &key, None, RegValue::None);
            }
        }
        say(&format!(
            "  CTF category per-user (HKCU, sau ILOT) → {}",
            if ok { "OK" } else { "FAIL" }
        ));
        ok
    }

    /// `textvn-cli activate` — kích hoạt profile TextVN (VI) cho phiên hiện
    /// tại. Tray gọi sau mỗi lần Ctrl+Shift toggle để bộ gõ active TRONG app
    /// cũng chuyển sang TextVN (trường hợp người dùng đang đứng ở bàn phím
    /// khác trong Win+Space — MS Việt / US): mode đổi → typing đổi ngay,
    /// không còn "icon E mà vẫn gõ tiếng Việt".
    pub fn activate_tip() -> i32 {
        if !com_init() {
            eprintln!("error: CoInitializeEx fail");
            return 1;
        }
        // Check-then-activate: TextVN ĐANG là bộ gõ được chọn → không cần làm gì
        // (blind-call ActivateProfile trên profile đang chọn trả E_FAIL — bắt
        // thật máy chủ repo 2026-10-03). Phân biệt `active` vs `enabled`: user
        // có thể đang chọn bàn phím khác trong Win+Space — khi đó PHẢI activate.
        if active_profile_is_textvn() {
            say("=== TextVN đã là bộ gõ active cho phiên này. ===");
            return 0;
        }
        let activated = activate_for_session() || active_profile_is_textvn();
        if activated {
            say("=== TextVN profile đã kích hoạt cho phiên này. ===");
            0
        } else {
            eprintln!(
                "error: ActivateProfile thất bại — chọn TextVN bằng Win+Space hoặc cài bản setup (phạm vi máy) nếu Windows 11 từ chối đăng ký per-user."
            );
            1
        }
    }

    /// Kích hoạt profile cho cả session (không chỉ thread của CLI), trả kết quả
    /// để caller không báo cài đặt thành công khi Windows từ chối profile.
    fn activate_for_session() -> bool {
        // SAFETY: COM đã init.
        unsafe {
            let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) else {
                say("  ActivateProfile(VI): không tạo được profile manager");
                return false;
            };
            // Ma trận thử: một số build Windows từ chối tổ hợp này nhưng nhận
            // tổ hợp khác (bắt thật máy chủ repo 2026-10-03: VI+DONTCARE →
            // E_FAIL nhưng EN+DONTCARE → OK tuỳ phiên). Thứ tự ưu tiên: đúng
            // ngôn ngữ trước, rồi mới tới biến thể.
            let combos: [(u16, &str, u32); 4] = [
                (
                    LANGID_VI,
                    "VI+DONTCARECURRENT",
                    TF_IPPMF_FORSESSION | TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
                ),
                (LANGID_VI, "VI", TF_IPPMF_FORSESSION),
                (
                    LANGID_EN,
                    "EN+DONTCARECURRENT",
                    TF_IPPMF_FORSESSION | TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
                ),
                (LANGID_EN, "EN", TF_IPPMF_FORSESSION),
            ];
            for (lang, label, flags) in combos {
                match mgr.ActivateProfile(
                    TF_PROFILETYPE_INPUTPROCESSOR,
                    lang,
                    &CLSID_TIP,
                    &PROFILE_GUID,
                    HKL::default(),
                    flags,
                ) {
                    Ok(()) => {
                        say(&format!("  ActivateProfile({label}, session) → OK"));
                        return true;
                    }
                    Err(e) => {
                        say(&format!(
                            "  ActivateProfile({label}) → {:#010x}",
                            e.code().0
                        ));
                    }
                }
            }
            let Ok(prof) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) else {
                return false;
            };
            for lang in [LANGID_VI, LANGID_EN] {
                match prof.ActivateLanguageProfile(&CLSID_TIP, lang, &PROFILE_GUID) {
                    Ok(()) => {
                        say(&format!("  ActivateLanguageProfile(0x{lang:04X}) → OK"));
                        return true;
                    }
                    Err(e2) => {
                        say(&format!(
                            "  ActivateLanguageProfile(0x{lang:04X}) → {:#010x}",
                            e2.code().0
                        ));
                    }
                }
            }
            false
        }
    }

    /// Ghi profile TextVN vào **danh sách ngôn ngữ hiện đại** của Windows
    /// (`HKCU\Control Panel\International\User Profile\<tag>`) rồi broadcast
    /// `WM_SETTINGCHANGE("International")`.
    ///
    /// Vì sao bắt buộc: `InstallLayoutOrTip` chỉ ghi danh sách nhập CTF; trên
    /// Win10/11 danh sách hiện đại (nguồn Win+Space / Settings) là store khác
    /// — sau chu kỳ unregister→register (migration nâng cấp 0.2.11), TextVN
    /// biến mất khỏi store hiện đại dù CTF keys đủ → `ActivateProfile` trả
    /// E_FAIL và người dùng không thể chọn lại TextVN (báo cáo 0.2.11: "bật
    /// tray lên cũng không gõ được tiếng Việt"). Store hiện đại không có API
    /// WinRT ghi được — ghi registry + broadcast là cách Windows tự dùng.
    fn ensure_modern_language_list() {
        for (tag, langid) in [("vi", LANGID_VI), ("en-US", LANGID_EN)] {
            let key = format!("Control Panel\\International\\User Profile\\{tag}");
            let value = format!("{langid:04X}:{{{}}}{{{}}}", CLSID_INNER, PROFILE_INNER);
            // Tạo key ngôn ngữ nếu thiếu (thêm ngôn ngữ vào danh sách) rồi set tip.
            let _ = set_reg_value(HKEY_CURRENT_USER, &key, None, RegValue::None);
            let _ = set_reg_value(HKEY_CURRENT_USER, &key, Some(&value), RegValue::Dword(1));
        }
        // Broadcast chuẩn WM_SETTINGCHANGE với lParam = "International" (tên khu
        // vực) — shell/Win+Space mới chắc chắn refresh; lParam(0) trước đây dựa
        // vào app tự đoán khu vực.
        broadcast_international();
    }

    /// `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "International")`
    /// — cùng cơ chế đã kiểm chứng thủ công trên máy thật 2026-10-03.
    fn broadcast_international() {
        use windows::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };
        let area: Vec<u16> = "International".encode_utf16().chain(Some(0)).collect();
        // SAFETY: HWND_BROADCAST + chuỗi nul-terminated sống trong suốt lời gọi.
        unsafe {
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(area.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );
        }
    }

    /// Gỡ profile TextVN khỏi **danh sách ngôn ngữ hiện đại** — không làm thì
    /// unregister để lại GHOST trong Win+Space (TextVN vẫn hiện nhưng CLSID đã
    /// bị xoá → chọn vào là chết; bắt trên máy chủ repo 2026-10-03 khi verify
    /// luồng portable: uninstall.ps1 xong TextVN vẫn nằm trong cả `vi` lẫn
    /// `en-US`). Chỉ xoá VALUE của TIP, giữ nguyên ngôn ngữ.
    fn remove_modern_language_list() {
        for (tag, langid) in [("vi", LANGID_VI), ("en-US", LANGID_EN)] {
            let key = format!("Control Panel\\International\\User Profile\\{tag}");
            let value = format!("{langid:04X}:{{{}}}{{{}}}", CLSID_INNER, PROFILE_INNER);
            let _ = delete_reg_value(HKEY_CURRENT_USER, &key, &value);
        }
        broadcast_international();
    }

    const USER_PROFILE_KEY: &str = r"Control Panel\International\User Profile";

    /// Value bàn phím TextVN trong `User Profile\<tag>`.
    fn textvn_profile_value(tag: &str) -> String {
        let langid = if tag.eq_ignore_ascii_case("vi") {
            LANGID_VI
        } else {
            LANGID_EN
        };
        format!("{langid:04X}:{{{}}}{{{}}}", CLSID_INNER, PROFILE_INNER)
    }

    fn languages_added_marker_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA")
            .filter(|v| !v.is_empty())
            .map(|d| PathBuf::from(d).join("TextVN").join(LANGUAGES_ADDED_MARKER))
    }

    fn read_languages_added() -> Vec<String> {
        languages_added_marker_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|c| parse_languages_marker(&c))
            .unwrap_or_default()
    }

    /// Danh sách ngôn ngữ của user (`User Profile\Languages`).
    fn user_languages() -> Option<Vec<String>> {
        reg_read_multi_sz(HKEY_CURRENT_USER, USER_PROFILE_KEY, "Languages")
    }

    /// R2-100: ghi nhận ngôn ngữ mà lần `register` này thêm vào danh sách của user
    /// (`before` = danh sách chụp TRƯỚC `InstallLayoutOrTip`). Gộp với marker cũ.
    fn record_languages_added(before: &[String]) {
        let missing = languages_missing(before, &PROFILE_LANGUAGE_TAGS);
        if missing.is_empty() {
            return;
        }
        let mut added = read_languages_added();
        for tag in missing {
            if !added.contains(&tag) {
                say(&format!(
                    "  Ngôn ngữ {tag} chưa có trong danh sách của user — TextVN thêm (gỡ cài đặt sẽ bỏ lại)"
                ));
                added.push(tag);
            }
        }
        if let Some(path) = languages_added_marker_path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&path, added.join("\n") + "\n");
        }
    }

    /// R2-100: value của từng ngôn ngữ TextVN đã thêm, chụp TRƯỚC khi gỡ TIP.
    fn snapshot_added_languages() -> Vec<(String, Vec<String>)> {
        read_languages_added()
            .into_iter()
            .map(|tag| {
                let names =
                    reg_value_names(HKEY_CURRENT_USER, &format!("{USER_PROFILE_KEY}\\{tag}"));
                (tag, names)
            })
            .collect()
    }

    /// R2-100: bỏ hẳn ngôn ngữ do TextVN thêm khi người dùng không gắn bàn phím nào khác
    /// cho nó — kể cả bàn phím Windows tự gắn lúc TIP (bàn phím cuối) bị gỡ. Ngôn ngữ có
    /// bàn phím khác của người dùng thì giữ. Xong thì bỏ marker.
    fn remove_added_languages(snapshot: &[(String, Vec<String>)]) {
        if snapshot.is_empty() {
            return;
        }
        for (tag, before) in snapshot {
            let key = format!("{USER_PROFILE_KEY}\\{tag}");
            if !added_language_removable(before, &textvn_profile_value(tag)) {
                say(&format!(
                    "  Ngôn ngữ {tag}: người dùng có bàn phím khác → giữ"
                ));
                continue;
            }
            let list = user_languages().unwrap_or_default();
            let listed = list.iter().any(|l| l.trim().eq_ignore_ascii_case(tag));
            let next = languages_without(&list, tag);
            if listed && next.len() == list.len() {
                say(&format!(
                    "  Ngôn ngữ {tag} là ngôn ngữ duy nhất của user → giữ"
                ));
                continue;
            }
            for name in reg_value_names(HKEY_CURRENT_USER, &key) {
                let auto_added = is_input_method_value(&name)
                    && !before.iter().any(|b| b.eq_ignore_ascii_case(&name));
                if auto_added {
                    call_layout_or_tip_spec(&name, ILOT_UNINSTALL, "UNINSTALL (Windows tự gắn)");
                }
            }
            if listed {
                let _ = set_reg_value(
                    HKEY_CURRENT_USER,
                    USER_PROFILE_KEY,
                    Some("Languages"),
                    RegValue::MultiSz(&next),
                );
            }
            let _ = delete_tree(HKEY_CURRENT_USER, &key);
            say(&format!(
                "  Ngôn ngữ {tag} do TextVN thêm → đã bỏ khỏi danh sách ngôn ngữ"
            ));
        }
        if let Some(path) = languages_added_marker_path() {
            let _ = std::fs::remove_file(path);
        }
        broadcast_international();
    }

    /// Vòng 14 (Zalo 32-bit): đường dẫn registry cho **view 32-bit (WOW64)**.
    /// App x86 (Zalo, Office/Notepad++ 32-bit) đọc COM/CTF ở view này —
    /// nền tảng không redirect tự động cho path ta tự ghi. Dùng prefix
    /// `WOW6432Node` tường minh để tái dùng nguyên bộ helper set/delete.
    fn wow_clsid_key() -> String {
        format!("Software\\Classes\\WOW6432Node\\CLSID\\{CLSID_STR}")
    }
    fn wow_ctf_tip_key() -> String {
        format!("Software\\WOW6432Node\\Microsoft\\CTF\\TIP\\{CLSID_STR}")
    }
    fn wow_ctf_profile_key(lang: u16) -> String {
        format!(
            "{}\\LanguageProfile\\0x{:08x}\\{}",
            wow_ctf_tip_key(),
            lang,
            PROFILE_STR
        )
    }
    fn wow_ctf_category_keys(cat: &str) -> [String; 2] {
        let tip = wow_ctf_tip_key();
        [
            format!(r"{tip}\Category\Category\{cat}\{CLSID_STR}"),
            format!(r"{tip}\Category\Item\{CLSID_STR}\{cat}"),
        ]
    }

    /// Vòng 14: mirror đăng ký sang view 32-bit — DLL x86 (`textvn-tsf-x86.dll`
    /// nằm cạnh DLL 64-bit) + CTF TIP/profile/category. Best-effort: gói cũ
    /// không có DLL x86 → bỏ qua không fail.
    fn register_wow64_mirror(dll_path: &Path, scope: Scope) -> i32 {
        let Some(dir) = dll_path.parent() else {
            return 0;
        };
        let x86 = dir.join("textvn-tsf-x86.dll");
        if !x86.is_file() {
            say("  WOW64 mirror: thiếu textvn-tsf-x86.dll — bỏ qua (gói cũ, app x86 không dùng được TIP)");
            return 0;
        }
        let x86_s = x86.to_string_lossy().to_string();
        let root = scope.root();
        let clsid = wow_clsid_key();
        let inproc = format!(r"{clsid}\InprocServer32");
        let icon = icon_path(dll_path);
        let mut ok = set_reg_value(root, &clsid, None, RegValue::Sz("TextVN TSF"));
        ok &= set_reg_value(root, &inproc, None, RegValue::Sz(&x86_s));
        ok &= set_reg_value(
            root,
            &inproc,
            Some("ThreadingModel"),
            RegValue::Sz("Apartment"),
        );
        for (_, lang) in REGISTER_LANGS {
            let key = wow_ctf_profile_key(lang);
            ok &= set_reg_value(root, &key, Some("Description"), RegValue::Sz("TextVN"));
            ok &= set_reg_value(root, &key, Some("IconFile"), RegValue::Sz(&icon));
            ok &= set_reg_value(root, &key, Some("IconIndex"), RegValue::Dword(0));
            ok &= set_reg_value(root, &key, Some("Enable"), RegValue::Dword(1));
        }
        for cat in [
            CAT_TIP_KEYBOARD,
            CAT_IMMERSIVE,
            CAT_SYSTRAY,
            CAT_DISPLAY_ATTRIBUTE_PROVIDER,
        ] {
            for key in wow_ctf_category_keys(cat) {
                ok &= set_reg_value(root, &key, None, RegValue::None);
            }
        }
        say(&format!(
            "  WOW64 mirror (app x86: Zalo/Office 32-bit) → {}",
            if ok { "OK" } else { "FAIL" }
        ));
        if ok {
            0
        } else {
            1
        }
    }

    /// Vòng 14: xoá mirror 32-bitView khi gỡ (cả 2 scope — HKCU do unregister
    /// user, HKLM do unregister machine/elevated).
    fn unregister_wow64_mirror(scope: Scope) {
        let root = scope.root();
        let _ = delete_tree(root, &wow_clsid_key());
        let _ = delete_tree(root, &wow_ctf_tip_key());
    }

    /// R2-30: chỉ xoá mirror WOW64 của `scope` khi DLL x86 đăng ký nằm dưới một
    /// trong `dirs` (hoặc đã mồ côi) — dùng khi đăng ký 64-bit thuộc bản khác.
    pub fn unregister_wow64_mirror_if_owned(scope: Scope, dirs: &[String]) {
        let inproc = format!(r"{}\InprocServer32", wow_clsid_key());
        let server = reg_read_string(scope.root(), &inproc);
        let exists = server
            .as_deref()
            .is_some_and(|p| Path::new(p.trim().trim_matches('"')).is_file());
        if server.is_some() && registration_owned_by_dir(server.as_deref(), exists, dirs) {
            unregister_wow64_mirror(scope);
            say("  WOW64 mirror thuộc thư mục đang gỡ → đã xoá");
        }
    }

    /// R2-30: đăng ký TIP 64-bit của `scope` có thuộc `dirs` không. User: đăng ký hiện
    /// hành (HKCU che HKLM) — profile per-user bật cho cả bản cài máy, chỉ gỡ khi nó
    /// thuộc thư mục đang gỡ. Machine: chỉ HKLM (R2-93) — override HKCU trỏ bản khác
    /// không được làm bộ gỡ bản máy bỏ sót HKLM trỏ vào thư mục sắp xoá.
    pub fn registration_owned_by(scope: Scope, dirs: &[String]) -> bool {
        let inproc = format!(r"{}\InprocServer32", clsid_key());
        let server = match scope {
            Scope::Machine => reg_read_string(HKEY_LOCAL_MACHINE, &inproc),
            Scope::User => reg_read_string(HKEY_CURRENT_USER, &inproc)
                .or_else(|| reg_read_string(HKEY_LOCAL_MACHINE, &inproc)),
        };
        let exists = server
            .as_deref()
            .is_some_and(|p| Path::new(p.trim().trim_matches('"')).is_file());
        registration_owned_by_dir(server.as_deref(), exists, dirs)
    }

    /// Program Files / Program Files (x86) theo **known folder** (registry HKLM do
    /// hệ thống quản lý) — KHÔNG đọc biến môi trường: `%ProgramFiles%` người dùng
    /// tự đặt được qua `HKCU\Environment` (R2-03/R2-15).
    pub fn program_files_roots() -> Vec<String> {
        use windows::Win32::UI::Shell::{
            FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
        };
        let mut roots = Vec::new();
        for id in [FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86] {
            // SAFETY: GUID hằng hợp lệ; chuỗi trả về được giải phóng bằng CoTaskMemFree.
            let Ok(raw) = (unsafe { SHGetKnownFolderPath(&id, KF_FLAG_DEFAULT, None) }) else {
                continue;
            };
            // SAFETY: `raw` là chuỗi NUL-terminated do shell cấp, còn sống tới CoTaskMemFree.
            let text = unsafe { raw.to_string() }.ok();
            // SAFETY: con trỏ do SHGetKnownFolderPath cấp phát bằng CoTaskMemAlloc.
            unsafe { CoTaskMemFree(Some(raw.0 as *const core::ffi::c_void)) };
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                let canonical = std::fs::canonicalize(&text)
                    .map(|c| c.to_string_lossy().into_owned())
                    .unwrap_or(text);
                roots.push(canonical);
            }
        }
        roots
    }

    /// Liệt kê tên subkey của `path` (một mức). Lỗi mở/liệt kê → rỗng.
    fn enum_subkeys(root: HKEY, path: &str) -> Vec<String> {
        let subkey = wide(path);
        let mut key = HKEY::default();
        // SAFETY: key mở chỉ để liệt kê; đóng ngay trước khi trả về.
        if unsafe { RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), Some(0), KEY_READ, &mut key) }
            != ERROR_SUCCESS
        {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut idx = 0u32;
        loop {
            let mut name = [0u16; 256];
            let mut len = name.len() as u32;
            // SAFETY: buffer `name`/`len` khớp; API ghi tối đa len-1 ký tự + nul.
            let status = unsafe {
                RegEnumKeyExW(
                    key,
                    idx,
                    Some(PWSTR(name.as_mut_ptr())),
                    &mut len,
                    None,
                    None,
                    None,
                    None,
                )
            };
            if status != ERROR_SUCCESS {
                break;
            }
            out.push(String::from_utf16_lossy(&name[..len as usize]));
            idx += 1;
        }
        let _ = unsafe { RegCloseKey(key) };
        out
    }

    /// Vòng 13: xoá value `Default` của các assembly CTF còn trỏ vào TIP
    /// TextVN (Windows ghi khi TextVN trở thành bộ gõ mặc định của một ngôn
    /// ngữ — ActivateLanguageProfile). Trả số value đã xoá.
    fn remove_ctf_assembly_defaults(root: HKEY, assemblies_path: &str) -> usize {
        // {34745C63-B2F0-4784-8B67-5E12C8701A31} = assembly "bàn phím mặc định"
        // của Windows; value `Default` dưới nó cho biết TIP nào là mặc định.
        const DEFAULT_ASSEMBLY: &str = "{34745C63-B2F0-4784-8B67-5E12C8701A31}";
        let mut removed = 0usize;
        for lang in enum_subkeys(root, assemblies_path) {
            let asm_key = format!("{assemblies_path}\\{lang}\\{DEFAULT_ASSEMBLY}");
            if let Some(v) = reg_read_named(root, &asm_key, "Default") {
                if v.contains(CLSID_INNER) && delete_reg_value(root, &asm_key, "Default") {
                    removed += 1;
                    say(&format!("  Đã xoá ghost Assemblies Default: {asm_key}"));
                }
            }
        }
        removed
    }

    /// Vòng 13: xoá `InputMethodOverride` khi còn trỏ TIP TextVN (Windows ghi
    /// khi TextVN là IME override mặc định). `false` = không có/không khớp.
    fn remove_input_method_override() -> bool {
        let path = r"Control Panel\International\User Profile";
        match reg_read_named(HKEY_CURRENT_USER, path, "InputMethodOverride") {
            Some(v) if v.contains(CLSID_INNER) => {
                delete_reg_value(HKEY_CURRENT_USER, path, "InputMethodOverride")
            }
            _ => false,
        }
    }

    /// Windows đã nhận profile để dùng cho một ngôn ngữ chưa. Đây là kiểm tra
    /// sau `InstallLayoutOrTip`, không chỉ là kiểm tra key registry có tồn tại.
    fn profile_is_enabled(lang: u16) -> bool {
        if !com_init() {
            return false;
        }
        // SAFETY: COM được khởi tạo ở trên; interface do hệ thống cấp.
        unsafe {
            CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .and_then(|profiles| profiles.IsEnabledLanguageProfile(&CLSID_TIP, lang, &PROFILE_GUID))
            .map(|enabled| enabled.as_bool())
            .unwrap_or(false)
        }
    }

    /// TextVN có đang là profile ĐANG ĐƯỢC CHỌN (active) không?
    /// KHÁC `profile_is_enabled` (chỉ nghĩa "được phép dùng"): người dùng có thể
    /// đang chọn bàn phím khác (MS Việt / US) — khi đó toggle mode TextVN cần
    /// ActivateProfile thật để chuyển bộ gõ active, không phải no-op.
    /// Dùng `GetActiveLanguageProfile` (trả GUID profile đang hoạt động của TIP).
    fn active_profile_is_textvn() -> bool {
        if !com_init() {
            return false;
        }
        // SAFETY: COM được khởi tạo ở trên; interface do hệ thống cấp.
        unsafe {
            CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .and_then(|prof| {
                let mut langid: u16 = 0;
                let mut guid = GUID::from_u128(0);
                prof.GetActiveLanguageProfile(&CLSID_TIP, &mut langid, &mut guid)
                    .map(|_| guid)
            })
            .map(|guid| guid == PROFILE_GUID)
            .unwrap_or(false)
        }
    }

    fn profile_metadata_ok(lang: u16) -> bool {
        let key = ctf_profile_key(lang);
        // Profile do API machine-wide tạo không luôn có `Enable` dưới HKLM;
        // fallback per-user của TextVN thì phải có nó và đặt thành 1.
        let user_ok = reg_key_exists(HKEY_CURRENT_USER, &key)
            && reg_read_dword(HKEY_CURRENT_USER, &key, "Enable").is_some_and(|v| v != 0);
        user_ok || reg_key_exists(HKEY_LOCAL_MACHINE, &key)
    }

    fn registration_metadata_ok() -> bool {
        REGISTER_LANGS
            .iter()
            .all(|(_, lang)| profile_metadata_ok(*lang))
    }

    fn machine_profile_metadata_ok() -> bool {
        REGISTER_LANGS
            .iter()
            .all(|(_, lang)| reg_key_exists(HKEY_LOCAL_MACHINE, &ctf_profile_key(*lang)))
    }

    pub fn do_register(dll_path: &Path, no_taskbar: bool, scope: Scope) -> i32 {
        say(&format!("=== TextVN register ({}) ===", scope.label()));
        let dll_s = dll_path.to_string_lossy().to_string();
        say(&format!("DLL: {dll_s}"));

        // Bước 1: COM server (bắt buộc — thiếu thì TSF không tạo được TIP).
        let root = scope.root();
        let k = clsid_key();
        let inproc = format!(r"{k}\InprocServer32");
        // Setup đã đăng ký COM/profile ở HKLM rồi gọi `register` trong token
        // người dùng gốc để thêm layout. Không ghi lại COM dưới HKCU ở bước
        // này: nếu UAC dùng một admin khác, uninstaller elevated không thể
        // xóa HKCU của người dùng gốc và sẽ để DLL override trỏ file đã xóa.
        // Portable ở đường dẫn khác vẫn được đăng ký per-user như trước.
        let activate_machine_for_user = scope == Scope::User
            && machine_profile_metadata_ok()
            && reg_read_string(HKEY_LOCAL_MACHINE, &inproc)
                .is_some_and(|p| p.eq_ignore_ascii_case(&dll_s) && Path::new(&p).is_file());
        if activate_machine_for_user {
            say("  Dùng COM/profile HKLM; dọn override TextVN HKCU cũ trước khi bật cho user");
            if !(delete_tree(HKEY_CURRENT_USER, &k)
                & delete_tree(HKEY_CURRENT_USER, &ctf_tip_key()))
            {
                say("FAIL: không dọn được override TSF per-user cũ");
                return 1;
            }
        }
        // COM per-user (HKCU\Software\Classes\CLSID) override HKLM cho user
        // hiện tại: đăng ký machine trên máy từng đăng ký per-user (portable
        // cũ, DLL đã xoá) sẽ bị key cũ che — TSF vẫn nạp DLL không tồn tại.
        // Xoá override HKCU trước khi ghi HKLM.
        if scope == Scope::Machine && reg_key_exists(HKEY_CURRENT_USER, &k) {
            say("  Xoá override per-user HKCU\\Software\\Classes\\CLSID trước khi đăng ký machine");
            if !delete_tree(HKEY_CURRENT_USER, &k) {
                // Per-user COM override HKLM: key cũ còn thì TSF vẫn nạp DLL
                // chết dù HKLM ghi đúng — không được coi là đăng ký thành công.
                say("FAIL: không xoá được override per-user — xoá tay HKCU\\Software\\Classes\\CLSID\\{CLSID} rồi chạy lại");
                return 3;
            }
        }
        let ok = activate_machine_for_user
            || (set_reg_value(root, &k, None, RegValue::Sz("TextVN TSF"))
                && set_reg_value(root, &inproc, None, RegValue::Sz(&dll_s))
                && set_reg_value(
                    root,
                    &inproc,
                    Some("ThreadingModel"),
                    RegValue::Sz("Apartment"),
                ));
        if !ok {
            if scope == Scope::Machine {
                say(&format!(
                    "FAIL: RegCreateKeyExW không ghi được {}\\{k} — chạy lại với quyền Administrator",
                    scope.label()
                ));
                return 3;
            }
            say(&format!(
                "FAIL: RegCreateKeyExW không ghi được {} — \
                 nguyên nhân thật ở dòng \"Registry create\" phía trên",
                clsid_registry_key()
            ));
            return 1;
        }
        say(&format!(
            "  COM server {} → OK",
            if activate_machine_for_user {
                "HKLM"
            } else {
                scope.label()
            }
        ));

        if !com_init() {
            say("!! CoInitializeEx fail");
            return 1;
        }

        if !activate_machine_for_user {
            grant_appcontainer_read(dll_path);
        }

        // Bước 2: profile + category. Mỗi bước độc lập — bản trước dừng ở
        // `Register()?` (HKLM) nên cài per-user KHÔNG bao giờ tới InstallLayoutOrTip.
        let icon = icon_path(dll_path);
        let (api_profiles_ok, api_categories_ok) = if activate_machine_for_user {
            (true, true)
        } else {
            register_with_tsf_api("TextVN", &icon)
        };
        // Machine install khong duoc thanh cong nho metadata HKCU fallback
        // tu ban portable cu. Profile va category phai duoc TSF API ghi HKLM.
        if scope == Scope::Machine && (!api_profiles_ok || !machine_profile_metadata_ok()) {
            say("FAIL: đăng ký profile/category TSF phạm vi máy chưa hoàn tất");
            return 3;
        }
        let machine_registered = machine_profile_metadata_ok();
        let (profiles_ok, used_ctf_fallback) = if !api_profiles_ok && !machine_registered {
            if scope == Scope::Machine {
                return 3;
            }
            (register_ctf_per_user("TextVN", &icon), true)
        } else {
            (true, false)
        };
        let categories_need_hkcu_fallback = !api_categories_ok;
        if !profiles_ok || !registration_metadata_ok() {
            say("FAIL: profile TSF chưa được đăng ký đầy đủ");
            return if scope == Scope::Machine { 3 } else { 1 };
        }

        // HKLM thuộc máy; layout list và ActivateProfile thuộc *user session*.
        // Khi UAC yêu cầu thông tin của một admin khác, làm hai bước sau trong
        // token admin sẽ kích hoạt sai tài khoản và có thể fail trước khi setup
        // kịp gọi ExecAsOriginalUser cho người dùng gốc.
        if scope == Scope::Machine {
            // R2-35: mirror WOW64 (COM 32-bit + CTF) cũng phải ghi ở HKLM — trước
            // đây nhánh máy return trước bước này nên app x86 (Zalo, Office 32-bit)
            // của các tài khoản khác không bao giờ thấy TIP. Best-effort như scope
            // user: thiếu DLL x86 (gói cũ) không làm hỏng đăng ký máy.
            if register_wow64_mirror(dll_path, Scope::Machine) != 0 {
                say("  WARN: mirror WOW64 (HKLM) chưa hoàn tất — app x86 có thể chưa thấy TIP");
            }
            say("=== Đăng ký TSF phạm vi máy hoàn tất; kích hoạt user ở bước riêng. ===");
            return 0;
        }

        // Bước 3: danh sách bàn phím của user (HKCU, không cần admin).
        // VI và EN ĐỀU DEFPROFILE — layout EN KHÔNG phải dup: app có ngôn ngữ
        // nhập en-US (Notepad/runner/máy tiếng Anh) chỉ nhận TextVN qua profile
        // EN; bỏ nó = gõ raw trong mọi app EN (repro CI fecf245 2026-10-02:
        // "got dduocj" thay vì "được "). KHÔNG uninstall trước khi thêm: ILOT
        // UNINSTALL deactivate TIP cho phiên và ActivateProfile phục hồi không
        // phải lúc nào cũng OK (0x80004005 đã biết) — cùng lỗi ở 5269ae1.
        // R2-100: danh sách ngôn ngữ TRƯỚC khi InstallLayoutOrTip tự thêm ngôn ngữ thiếu.
        let languages_before = if no_taskbar { None } else { user_languages() };
        let layouts_ok = if no_taskbar {
            call_layout_or_tip(LANGID_VI, ILOT_UNINSTALL, "UNINSTALL")
                && call_layout_or_tip(LANGID_EN, ILOT_UNINSTALL, "UNINSTALL")
        } else {
            let vi_ok = call_layout_or_tip(LANGID_VI, ILOT_DEFPROFILE, "DEFPROFILE");
            let en_ok = call_layout_or_tip(LANGID_EN, ILOT_DEFPROFILE, "DEFPROFILE");
            if vi_ok && !en_ok {
                say("  WARN: không thêm được profile cho ngôn ngữ EN (best-effort) — VI vẫn dùng được");
            }
            vi_ok
        };
        if !layouts_ok {
            say("FAIL: Windows không thêm được TextVN vào danh sách bàn phím");
            return 1;
        }
        // Danh sách ngôn ngữ hiện đại (nguồn Win+Space trên Win10/11) — bắt
        // buộc, xem doc `ensure_modern_language_list`. Idempotent.
        ensure_modern_language_list();
        if let Some(before) = &languages_before {
            record_languages_added(before);
        }

        if used_ctf_fallback {
            // Spike doc (tsf-registration-spike.md): cập nhật input list có thể
            // reset enabled flag → ghi lại Enable=1 SAU InstallLayoutOrTip,
            // trước hậu kiểm `profile_metadata_ok`.
            for (_, lang) in REGISTER_LANGS {
                set_reg_value(
                    HKEY_CURRENT_USER,
                    &ctf_profile_key(lang),
                    Some("Enable"),
                    RegValue::Dword(1),
                );
            }
        }
        if categories_need_hkcu_fallback {
            // API RegisterCategory thất bại (không admin): ghi category per-user
            // SAU ILOT — ghi trước ILOT thì bị ILOT xoá mất (repro 2026-10-02),
            // mất category display-attribute provider = gạch chân không tắt được.
            register_ctf_categories_per_user();
        }

        // Bước 4: kích hoạt ngay cho session. Tiến trình không có ngữ cảnh nhập
        // tương tác (CLI ẩn console do setup im lặng chạy) nhận E_FAIL dù đăng ký
        // đã đủ — CI windows-package: `ActivateProfile(VI) → 0x80004005` chỉ khi
        // chạy từ setup. Bản thường: TextVN đã nằm trong danh sách bàn phím
        // (DEFPROFILE), tray/Win+Space kích hoạt sau → chỉ cảnh báo. Bản tray-only
        // (`--no-taskbar`) không có mục trong danh sách → kích hoạt là bắt buộc.
        if !activate_for_session() {
            if no_taskbar {
                say("FAIL: Windows không kích hoạt được profile TextVN cho phiên hiện tại");
                return 1;
            }
            say("  WARN: chưa kích hoạt được cho phiên này — chọn TextVN bằng Win+Space hoặc mở TextVN");
        }

        // `--no-taskbar` chủ động gỡ layout khỏi danh sách; không đòi hỏi
        // IsEnabledLanguageProfile trong trường hợp dành riêng cho tray.
        // Hậu kiểm bắt buộc VI; EN chỉ là thông tin (máy không có en-US vẫn
        // gõ được tiếng Việt — fail cứng EN là sai, đã từng chặn đăng ký).
        if !no_taskbar {
            if !registration_ok() {
                say("FAIL: hậu kiểm đăng ký TSF không đạt");
                return 1;
            }
            if !profile_is_enabled(LANGID_EN) {
                say("  WARN: profile EN chưa bật (máy có thể không có ngôn ngữ en-US) — VI vẫn dùng được");
            }
        }

        // Vòng 14 (Zalo 32-bit): mirror đăng ký sang view 32-bit (WOW6432Node)
        // — Zalo PC, Office/Notepad++ x86 là tiến trình 32-bit, không thể nạp
        // DLL 64-bit (ERROR_BAD_EXE_FORMAT 193) và COM 32-bit đọc view
        // WOW6432Node (đo thật: view đó trống hoàn toàn). Best-effort: thiếu
        // textvn-tsf-x86.dll (gói cũ) → bỏ qua, không fail đăng ký.
        let mirror = register_wow64_mirror(dll_path, scope);
        if mirror != 0 {
            say("  WARN: mirror WOW64 chưa hoàn tất — app x86 có thể chưa thấy TIP");
        }

        if no_taskbar {
            say("=== Đăng ký hoàn tất (tray-only, profile đã được kích hoạt). ===");
        } else {
            say("=== Đăng ký hoàn tất. Dùng Win+Space để chọn TextVN. ===");
        }
        0
    }

    pub fn do_unregister(scope: Scope) -> i32 {
        say(&format!("=== TextVN unregister ({}) ===", scope.label()));

        // R2-100: chụp bàn phím của các ngôn ngữ TextVN đã thêm TRƯỚC khi gỡ TIP —
        // Windows tự gắn bàn phím mặc định khi bỏ bàn phím cuối của một ngôn ngữ.
        let added_languages = snapshot_added_languages();

        // Bước 1: gỡ khỏi danh sách layout của user — tránh ghost keyboard.
        call_layout_or_tip(LANGID_VI, ILOT_UNINSTALL, "UNINSTALL");
        call_layout_or_tip(LANGID_EN, ILOT_UNINSTALL, "UNINSTALL");

        // Bước 2: profile/category qua API (HKLM; user thường sẽ FAIL — vô hại).
        if com_init() {
            // SAFETY: COM đã init; interface do CoCreateInstance cấp.
            unsafe {
                if let Ok(cat) = CoCreateInstance::<_, ITfCategoryMgr>(
                    &CLSID_TF_CategoryMgr,
                    None,
                    CLSCTX_INPROC_SERVER,
                ) {
                    for guid in [
                        GUID_TFCAT_TIP_KEYBOARD,
                        GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
                        GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
                        GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                    ] {
                        let _ = cat.UnregisterCategory(&CLSID_TIP, &guid, &CLSID_TIP);
                    }
                }
                if let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
                    &CLSID_TF_InputProcessorProfiles,
                    None,
                    CLSCTX_INPROC_SERVER,
                ) {
                    for lang in [LANGID_VI, LANGID_EN] {
                        let _ = mgr.UnregisterProfile(&CLSID_TIP, lang, &PROFILE_GUID, 0);
                    }
                }
                if let Ok(prof) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
                    &CLSID_TF_InputProcessorProfiles,
                    None,
                    CLSCTX_INPROC_SERVER,
                ) {
                    match prof.Unregister(&CLSID_TIP) {
                        Ok(()) => say("  Profiles.Unregister → OK"),
                        Err(e) => say(&format!("  Profiles.Unregister → {:#010x}", e.code().0)),
                    }
                }
            }
        }

        // Bước 3: registry của chính TextVN. Xoá HKLM CTF TIP khi scope machine
        // (API UnregisterProfile fail im lặng nếu không admin — key machine sót
        // là ghost registration mà hậu kiểm vẫn thấy qua fallback HKLM).
        // `&` (không phải `&&`): delete fail vẫn thử xoá nốt các key còn lại —
        // short-circuit từng bỏ qua CTF TIP HKLM trên path lỗi (audit vòng 2).
        // Danh sách ngôn ngữ hiện đại (Win+Space): gỡ entry TIP — thiếu bước này
        // unregister để lại ghost (đã bắt thật, xem doc remove_modern_language_list).
        //
        // Vòng 13 (chủ repo bắt thật 2026-10-05: gỡ 0.2.19 còn sót 27 path
        // chứa GUID TIP): hai nhóm ghost mà các vòng trước bỏ lỡ —
        // (a) `CTF\Assemblies\<lang>\{34745C63…}` value `Default` = TIP
        //     (Windows ghi khi TextVN thành bộ gõ mặc định của ngôn ngữ);
        // (b) `Control Panel\International\User Profile` value
        //     `InputMethodOverride` = `042A:{TIP}{PROFILE}`.
        // Cả hai ở HKCU — gỡ được không cần elevation; để lại = Win+Space/
        // default IME trỏ TIP đã xoá (chọn vào là chết).
        unregister_wow64_mirror(scope);
        let mut ghost_removed =
            remove_ctf_assembly_defaults(HKEY_CURRENT_USER, r"Software\Microsoft\CTF\Assemblies");
        if remove_input_method_override() {
            ghost_removed += 1;
        }
        if ghost_removed > 0 {
            say(&format!(
                "  Đã xoá {ghost_removed} value ghost (Assemblies Default / InputMethodOverride)"
            ));
        }
        remove_modern_language_list();
        remove_added_languages(&added_languages);
        let mut deleted = delete_tree(HKEY_CURRENT_USER, &ctf_tip_key())
            & delete_tree(HKEY_CURRENT_USER, &clsid_key());
        if scope == Scope::Machine {
            deleted &= delete_tree(HKEY_LOCAL_MACHINE, &clsid_key())
                & delete_tree(HKEY_LOCAL_MACHINE, &ctf_tip_key());
            let _ = remove_ctf_assembly_defaults(
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\CTF\Assemblies",
            );
        } else {
            // Máy từng cài phạm vi máy (portable self-heal B7) còn cả cây HKLM:
            // cố dọn best-effort — không admin thì FAIL bình thường và KHÔNG
            // tính vào exit code của unregister user (ghost máy dọn khi gỡ
            // elevated hoặc bằng `unregister --scope machine`).
            let _ = delete_tree(HKEY_LOCAL_MACHINE, &clsid_key());
            let _ = delete_tree(HKEY_LOCAL_MACHINE, &ctf_tip_key());
        }
        if !deleted {
            say("WARN: còn key chưa xoá được (xem dòng Registry delete phía trên) — đăng ký cũ có thể vẫn hoạt động");
            return 1;
        }

        say("=== Hủy đăng ký hoàn tất. ===");
        0
    }

    /// HKCU\Software\Classes override HKLM cho CLSID. Neu override per-user
    /// tro DLL da xoa, KHONG duoc coi HKLM DLL con ton tai la thanh cong:
    /// TSF van se nap override che do va bo go khong hoat dong.
    ///
    /// Với phạm vi user (không có quyền admin), API TSF `profile_is_enabled`
    /// có thể trả về false vì nó chỉ đọc HKLM. Khi đó kiểm tra `Enable == 1`
    /// trong registry HKCU (đã được ghi bởi `register_ctf_per_user` và `InstallLayoutOrTip`).
    pub fn registration_ok() -> bool {
        let inproc = format!(r"{}\InprocServer32", clsid_key());
        let server = reg_read_string(HKEY_CURRENT_USER, &inproc)
            .or_else(|| reg_read_string(HKEY_LOCAL_MACHINE, &inproc));
        let server_ok = server.is_some_and(|p| Path::new(&p).is_file());
        let vi_enabled = profile_is_enabled(LANGID_VI)
            || reg_read_dword(HKEY_CURRENT_USER, &ctf_profile_key(LANGID_VI), "Enable")
                .is_some_and(|v| v != 0);
        server_ok && registration_metadata_ok() && vi_enabled
    }

    pub fn do_status() -> i32 {
        say("=== TextVN status ===");
        let inproc = format!(r"{}\InprocServer32", clsid_key());
        let server = reg_read_string(HKEY_CURRENT_USER, &inproc)
            .or_else(|| reg_read_string(HKEY_LOCAL_MACHINE, &inproc));
        match &server {
            Some(p) => say(&format!(
                "COM server: {p} ({})",
                if Path::new(p).is_file() {
                    "OK"
                } else {
                    "FILE MISSING"
                }
            )),
            None => say("COM server: missing"),
        }
        say(&format!(
            "TIP profile: HKLM={} HKCU={}",
            reg_key_exists(HKEY_LOCAL_MACHINE, &ctf_tip_key()),
            reg_key_exists(HKEY_CURRENT_USER, &ctf_profile_key(LANGID_VI))
        ));

        if com_init() {
            // SAFETY: COM đã init.
            unsafe {
                if let Ok(prof) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
                    &CLSID_TF_InputProcessorProfiles,
                    None,
                    CLSCTX_INPROC_SERVER,
                ) {
                    match prof.IsEnabledLanguageProfile(&CLSID_TIP, LANGID_VI, &PROFILE_GUID) {
                        Ok(b) => say(&format!("IsEnabledLanguageProfile(VI) = {}", b.as_bool())),
                        Err(e) => say(&format!(
                            "IsEnabledLanguageProfile FAIL {:#010x}",
                            e.code().0
                        )),
                    }
                }
            }
        }
        if registration_ok() {
            say("TIP registration: OK");
            0
        } else {
            say("TIP registration: INCOMPLETE");
            1
        }
    }
}

// ─── Public API (cross-platform stubs cho non-Windows) ───────────────────────────────────────────

/// Đăng ký TSF TIP. `scope`: `"user"` (mặc định, không cần admin) | `"machine"` (HKLM, cần admin).
/// Exit code: 0 thành công, 1 lỗi đăng ký, 2 lỗi tham số, 3 cần quyền Administrator.
/// `textvn-cli schedule-delete <path...>` — xem `win_impl::schedule_delete`.
pub fn schedule_delete(paths: &[String]) -> i32 {
    #[cfg(windows)]
    return win_impl::schedule_delete(paths);

    #[cfg(not(windows))]
    {
        let _ = paths;
        eprintln!("error: `schedule-delete` chỉ hỗ trợ trên Windows");
        1
    }
}

/// `textvn-cli activate` — xem `win_impl::activate_tip`.
pub fn activate_tip() -> i32 {
    #[cfg(windows)]
    return win_impl::activate_tip();

    #[cfg(not(windows))]
    {
        eprintln!("error: `activate` chỉ hỗ trợ trên Windows");
        1
    }
}

pub fn register_tip(scope: &str, dll: Option<&Path>, no_taskbar: bool) -> i32 {
    #[cfg(windows)]
    {
        let scope_label = scope;
        let scope = match scope {
            "user" => win_impl::Scope::User,
            "machine" => win_impl::Scope::Machine,
            _ => return 2,
        };
        let dll_path = match resolve_dll_path(dll) {
            Ok(p) => p,
            Err(e) => {
                // Ghi register.log TRƯỚC khi thoát: hộp thoại tray ghép advice
                // từ đuôi log — lỗi resolve trước đây chỉ vào stderr (console
                // ẩn nuốt mất) nên dialog hiển thị đuôi log của run cũ và khớp
                // sai nguyên nhân. Header "===" đặt ranh giới run mới (nhãn
                // HKCU/HKLM khớp header của do_register).
                let label = if scope_label == "machine" {
                    "HKLM"
                } else {
                    "HKCU"
                };
                say(&format!("=== TextVN register ({label}) ==="));
                say(&format!("error: {e}"));
                eprintln!("error: {e}");
                return 1;
            }
        };
        let dll_path = if scope == win_impl::Scope::Machine {
            match machine_scope_dll(&dll_path) {
                Ok(p) => p,
                Err(e) => {
                    say("=== TextVN register (HKLM) ===");
                    say(&format!("FAIL: {e}"));
                    eprintln!("error: {e}");
                    return 3;
                }
            }
        } else {
            dll_path
        };
        win_impl::do_register(&dll_path, no_taskbar, scope)
    }

    #[cfg(not(windows))]
    {
        let _ = (scope, dll, no_taskbar);
        eprintln!("error: `register` chỉ hỗ trợ trên Windows");
        1
    }
}

/// R2-03/R2-15 (chốt chặn có thẩm quyền — CLI là tiến trình duy nhất thật sự
/// elevated): `--scope machine` chỉ đăng ký DLL đã canonicalize nằm dưới Program
/// Files (known folder) và không dưới `WindowsApps`; áp cho cả `textvn-tsf-x86.dll`
/// cạnh nó. Trả về đường dẫn canonical (không tiền tố `\\?\`) để HKLM trỏ đúng file
/// đã kiểm tra — đường dẫn qua junction trong thư mục người dùng có thể bị trỏ lại
/// sau khi đăng ký. Bản portable/cài riêng tài khoản/Store cần phạm vi máy: dùng
/// bộ cài TextVN cho mọi người dùng (`TextVN-setup-*-machine.exe`).
#[cfg(windows)]
fn machine_scope_dll(dll_path: &Path) -> Result<PathBuf, String> {
    let roots = win_impl::program_files_roots();
    if roots.is_empty() {
        return Err("không xác định được thư mục Program Files (known folder)".to_string());
    }
    let canonical = |p: &Path| -> Result<PathBuf, String> {
        let c = std::fs::canonicalize(p)
            .map_err(|e| format!("không chuẩn hoá được {}: {e}", p.display()))?;
        let s = c.to_string_lossy().into_owned();
        if machine_scope_dll_allowed(&s, &roots) {
            Ok(PathBuf::from(strip_verbatim_prefix(&s)))
        } else {
            Err(format!(
                "từ chối đăng ký phạm vi máy: {} không nằm trong Program Files \
                 (thư mục người dùng ghi được hoặc WindowsApps). Dùng bộ cài TextVN \
                 cho mọi người dùng (TextVN-setup-*-machine.exe); bản portable/Store \
                 chỉ đăng ký cho tài khoản hiện tại.",
                strip_verbatim_prefix(&s)
            ))
        }
    };
    let dll = canonical(dll_path)?;
    if let Some(dir) = dll.parent() {
        let x86 = dir.join("textvn-tsf-x86.dll");
        if x86.is_file() {
            canonical(&x86)?;
        }
    }
    Ok(dll)
}

/// Hủy đăng ký TSF TIP. Trả về exit code: 0 thành công, 1 lỗi, 2 lỗi tham số.
///
/// `owned_by` (R2-30, `--if-owned-by <dir>`): chỉ gỡ khi đăng ký hiện hành thuộc
/// `<dir>` (hoặc mồ côi) — xem [`registration_owned_by_dir`].
pub fn unregister_tip(scope: &str, owned_by: Option<&Path>) -> i32 {
    #[cfg(windows)]
    {
        let scope = match scope {
            "user" => win_impl::Scope::User,
            "machine" => win_impl::Scope::Machine,
            _ => return 2,
        };
        if let Some(dir) = owned_by {
            let mut dirs = vec![dir.to_string_lossy().into_owned()];
            if let Ok(c) = std::fs::canonicalize(dir) {
                dirs.push(c.to_string_lossy().into_owned());
            }
            if !win_impl::registration_owned_by(scope, &dirs) {
                say("=== TextVN unregister (--if-owned-by) ===");
                say(&format!(
                    "  Bỏ qua: TIP đang đăng ký cho một bản TextVN khác (không thuộc {})",
                    dir.display()
                ));
                win_impl::unregister_wow64_mirror_if_owned(win_impl::Scope::User, &dirs);
                if scope == win_impl::Scope::Machine {
                    win_impl::unregister_wow64_mirror_if_owned(scope, &dirs);
                }
                return 0;
            }
        }
        win_impl::do_unregister(scope)
    }

    #[cfg(not(windows))]
    {
        let _ = (scope, owned_by);
        eprintln!("error: `unregister` chỉ hỗ trợ trên Windows");
        1
    }
}

/// TIP đã đăng ký đủ để Windows nạp được (dùng cho `doctor`). Non-Windows: `false`.
pub fn tip_registration_ok() -> bool {
    #[cfg(windows)]
    {
        win_impl::registration_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Windows có đang giữ Ctrl + Shift cho việc đổi ngôn ngữ/bố cục bàn phím không
/// (`HKCU\Keyboard Layout\Toggle`; thiếu `Layout Hotkey` = mặc định = Ctrl + Shift).
/// Cùng quy tắc với `tray/src/hotkey.rs` (tray sửa được; doctor chỉ báo). `None` ngoài Windows.
pub fn ctrl_shift_taken_by_windows() -> Option<bool> {
    #[cfg(windows)]
    {
        use windows::Win32::System::Registry::HKEY_CURRENT_USER;
        const PATH: &str = r"Keyboard Layout\Toggle";
        let get = |name| win_impl::reg_read_named(HKEY_CURRENT_USER, PATH, name);
        let language = get("Language Hotkey").or_else(|| get("Hotkey"));
        let layout = get("Layout Hotkey");
        Some(language.as_deref() == Some("2") || layout.as_deref().is_none_or(|v| v == "2"))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Kiểm tra trạng thái đăng ký. Trả về exit code: 0 OK, 1 lỗi.
pub fn status_tip(_scope: &str) -> i32 {
    #[cfg(windows)]
    {
        win_impl::do_status()
    }

    #[cfg(not(windows))]
    {
        eprintln!("error: `register status` chỉ hỗ trợ trên Windows");
        1
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn reparse_link_detection_does_not_follow_links() {
        let dir = std::env::temp_dir().join(format!("textvn-reparse-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.txt");
        std::fs::write(&target, b"x").unwrap();
        let link = dir.join("register.log");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(is_reparse_link(&link));
        assert!(!is_reparse_link(&target));
        assert!(!is_reparse_link(&dir.join("missing")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// R2-21: junction/symlink ở `%LOCALAPPDATA%\TextVN` → KHÔNG tạo `logs` ở vị
    /// trí bị điều hướng (bản cũ create_dir_all trước rồi mới kiểm tra).
    #[cfg(unix)]
    #[test]
    fn log_writer_checks_reparse_before_creating_dirs() {
        let base = std::env::temp_dir().join(format!("textvn-logprep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let elsewhere = base.join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let local = base.join("local");
        std::fs::create_dir_all(&local).unwrap();
        std::os::unix::fs::symlink(&elsewhere, local.join("TextVN")).unwrap();
        assert!(prepare_log_file(&local).is_none());
        assert!(
            !elsewhere.join("logs").exists(),
            "không được tạo logs qua link"
        );

        // Đường bình thường: tạo đủ cấp, trả về file log.
        let clean = base.join("clean");
        std::fs::create_dir_all(&clean).unwrap();
        let file = prepare_log_file(&clean).expect("đường thường phải ghi được log");
        assert_eq!(file, clean.join("TextVN").join("logs").join("register.log"));
        append_log_line(&file, "dòng 1");
        assert!(std::fs::read_to_string(&file).unwrap().contains("dòng 1"));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// R2-21: log vượt ngưỡng xoay sang `.1`; nhỏ hơn thì giữ nguyên.
    #[test]
    fn log_rotates_when_larger_than_limit() {
        let dir = std::env::temp_dir().join(format!("textvn-logrot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("register.log");
        std::fs::write(&file, vec![b'x'; 64]).unwrap();
        rotate_log_if_larger(&file, 100);
        assert!(file.exists(), "chưa vượt ngưỡng: giữ nguyên");
        rotate_log_if_larger(&file, 10);
        assert!(!file.exists());
        assert!(dir.join("register.log.1").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// R2-03/R2-15: phạm vi máy chỉ nhận DLL dưới Program Files (known folder),
    /// không nhận thư mục người dùng, thư mục tên giống, hay WindowsApps.
    #[test]
    fn machine_scope_rejects_user_writable_and_windowsapps_paths() {
        let roots = vec![
            r"C:\Program Files".to_string(),
            r"\\?\C:\Program Files (x86)".to_string(),
        ];
        for ok in [
            r"C:\Program Files\TextVN\textvn-tsf.dll",
            r"\\?\C:\Program Files\TextVN\textvn-tsf.dll",
            r"c:\program files (x86)\TextVN\textvn-tsf-x86.dll",
        ] {
            assert!(machine_scope_dll_allowed(ok, &roots), "{ok}");
        }
        for bad in [
            r"C:\Users\a\AppData\Local\Programs\TextVN\textvn-tsf.dll",
            r"C:\Users\a\Downloads\TextVN\textvn-tsf.dll",
            r"D:\x\Program Files\TextVN\textvn-tsf.dll",
            r"C:\Program FilesEvil\textvn-tsf.dll",
            r"C:\Program Files",
            r"C:\Program Files\WindowsApps\LinhBH.CoM.TextVN_1.2.27.0_x64__abc\textvn-tsf.dll",
            r"\\?\C:\Program Files\WindowsApps\X\textvn-tsf.dll",
        ] {
            assert!(!machine_scope_dll_allowed(bad, &roots), "{bad}");
        }
        assert!(!machine_scope_dll_allowed(
            r"C:\Program Files\TextVN\textvn-tsf.dll",
            &[]
        ));
        assert_eq!(
            strip_verbatim_prefix(r"\\?\C:\Program Files\TextVN\textvn-tsf.dll"),
            r"C:\Program Files\TextVN\textvn-tsf.dll"
        );
        assert_eq!(
            strip_verbatim_prefix(r"\\?\UNC\srv\share\x.dll"),
            r"\\srv\share\x.dll"
        );
        assert_eq!(strip_verbatim_prefix(r"C:\a.dll"), r"C:\a.dll");
    }

    /// R2-100: chỉ ghi nhận ngôn ngữ trước đó chưa có; gỡ cài đặt chỉ bỏ ngôn ngữ không
    /// có bàn phím nào khác của người dùng; không bao giờ bỏ ngôn ngữ cuối cùng.
    #[test]
    fn languages_added_by_textvn_are_tracked_and_removed_only_when_unused() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        assert_eq!(
            languages_missing(&s(&["en-US"]), &PROFILE_LANGUAGE_TAGS),
            s(&["vi"])
        );
        assert!(languages_missing(&s(&["EN-us", " vi "]), &PROFILE_LANGUAGE_TAGS).is_empty());
        assert_eq!(
            languages_missing(&s(&["vi"]), &PROFILE_LANGUAGE_TAGS),
            s(&["en-US"])
        );

        let ms_telex =
            "042A:{C2CB2CF0-AF47-413E-9780-8BC3A3C16068}{5FB02EC5-0A77-4684-B4FA-DEF8A2195628}";
        assert!(is_input_method_value(ms_telex));
        assert!(is_input_method_value("0409:00000409"));
        assert!(!is_input_method_value("CachedLanguageName"));
        assert!(!is_input_method_value("042A"));

        let tv = format!("042A:{{{CLSID_INNER}}}{{{PROFILE_INNER}}}");
        assert!(added_language_removable(
            &s(&["CachedLanguageName", tv.as_str()]),
            &tv
        ));
        assert!(added_language_removable(
            &s(&[tv.to_lowercase().as_str()]),
            &tv
        ));
        assert!(!added_language_removable(
            &s(&["CachedLanguageName", tv.as_str(), ms_telex]),
            &tv
        ));

        assert_eq!(languages_without(&s(&["en-US", "vi"]), "VI"), s(&["en-US"]));
        assert_eq!(
            languages_without(&s(&["vi"]), "vi"),
            s(&["vi"]),
            "không bỏ ngôn ngữ cuối"
        );
        assert_eq!(
            parse_languages_marker("vi\r\nfoo\nVI\nen-us\n"),
            s(&["vi", "en-US"])
        );
    }

    /// R2-30: gỡ bản portable chỉ gỡ đăng ký khi nó thuộc thư mục đó (hoặc mồ côi).
    #[test]
    fn unregister_if_owned_by_only_touches_own_or_orphaned_registration() {
        let dirs = vec![r"D:\Tools\TextVN".to_string()];
        assert!(registration_owned_by_dir(None, false, &dirs));
        assert!(registration_owned_by_dir(
            Some(r"C:\Program Files\TextVN\textvn-tsf.dll"),
            false,
            &dirs
        ));
        assert!(registration_owned_by_dir(
            Some(r"d:\tools\textvn\textvn-tsf.dll"),
            true,
            &dirs
        ));
        assert!(!registration_owned_by_dir(
            Some(r"C:\Program Files\TextVN\textvn-tsf.dll"),
            true,
            &dirs
        ));
        assert!(!registration_owned_by_dir(
            Some(r"D:\Tools\TextVN2\textvn-tsf.dll"),
            true,
            &dirs
        ));
        // Thư mục canonical dạng \\?\ vẫn khớp đường dẫn registry thường.
        assert!(registration_owned_by_dir(
            Some(r"D:\Tools\TextVN\textvn-tsf.dll"),
            true,
            &[r"\\?\D:\Tools\TextVN".to_string()]
        ));
    }

    #[test]
    fn layout_spec_formatting_vi() {
        let spec = layout_spec(LANGID_VI);
        assert!(
            spec.starts_with("0x042A:"),
            "phải bắt đầu bằng 0x042A: — got: {spec}"
        );
        assert!(spec.contains(CLSID_STR), "phải chứa CLSID — got: {spec}");
        assert!(
            spec.contains(PROFILE_STR),
            "phải chứa Profile GUID — got: {spec}"
        );
    }

    #[test]
    fn layout_spec_formatting_en() {
        let spec = layout_spec(LANGID_EN);
        assert!(spec.starts_with("0x0409:"), "got: {spec}");
    }

    #[test]
    fn registry_key_paths() {
        let key = clsid_registry_key();
        assert!(
            key.starts_with(r"HKCU\Software\Classes\CLSID\"),
            "phải bắt đầu bằng HKCU\\...CLSID\\ — got: {key}"
        );
        assert!(key.contains(CLSID_STR), "phải chứa CLSID — got: {key}");
    }

    #[test]
    fn resolve_dll_path_nonexistent_returns_err() {
        let p = std::path::Path::new(r"C:\does\not\exist\textvn-tsf.dll");
        let res = resolve_dll_path(Some(p));
        assert!(res.is_err(), "file không tồn tại phải trả Err");
    }

    #[test]
    fn resolve_dll_path_valid_file_succeeds() {
        // Tạo file tạm để test path resolution
        let tmp = std::env::temp_dir().join("textvn-tsf-test.dll");
        std::fs::write(&tmp, b"").unwrap();
        let res = resolve_dll_path(Some(&tmp));
        std::fs::remove_file(&tmp).ok();
        assert!(res.is_ok(), "file tồn tại phải trả Ok: {res:?}");
    }

    #[test]
    fn ctf_layout_matches_tsf_registry_schema() {
        assert_eq!(
            ctf_tip_key(),
            format!(r"Software\Microsoft\CTF\TIP\{CLSID_STR}")
        );
        let vi = ctf_profile_key(LANGID_VI);
        assert!(
            vi.ends_with(&format!(r"\LanguageProfile\0x0000042a\{PROFILE_STR}")),
            "{vi}"
        );
        let [by_cat, by_item] = ctf_category_keys(CAT_TIP_KEYBOARD);
        assert!(by_cat.contains(&format!(
            r"\Category\Category\{CAT_TIP_KEYBOARD}\{CLSID_STR}"
        )));
        assert!(by_item.contains(&format!(r"\Category\Item\{CLSID_STR}\{CAT_TIP_KEYBOARD}")));
    }

    /// GUID category display-attribute phải khớp `GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER`
    /// của Windows ({046B8C80-…}): ghi sai GUID (2464BEB0, đã từng xảy ra) là
    /// đăng ký category rác — TSF không bao giờ hỏi provider của TIP.
    #[test]
    fn display_attribute_provider_category_matches_tsf_constant() {
        assert_eq!(
            CAT_DISPLAY_ATTRIBUTE_PROVIDER,
            "{046B8C80-1647-40F7-9B21-B93B81AABC1B}"
        );
    }

    /// Dòng log registry phải mang tên ký hiệu của mã lỗi: hộp thoại tray ghép
    /// gợi ý theo nguyên nhân từ đuôi log — log chỉ in hex thì nhánh gợi ý ACL
    /// (`ACCESS_DENIED`) thành dead code, người dùng nhận gợi ý sai chỗ.
    #[cfg(windows)]
    #[test]
    fn lstatus_name_covers_acl_and_common_registry_errors() {
        assert_eq!(win_impl::lstatus_name(5), Some("ERROR_ACCESS_DENIED"));
        assert_eq!(win_impl::lstatus_name(2), Some("ERROR_FILE_NOT_FOUND"));
        assert_eq!(win_impl::lstatus_name(87), Some("ERROR_INVALID_PARAMETER"));
        assert_eq!(win_impl::lstatus_name(0x1234_5678), None);
    }
}
