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
/// Profile GUID của TextVN TIP.
#[cfg_attr(not(windows), allow(dead_code))]
pub const PROFILE_STR: &str = "{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}";

#[cfg_attr(not(windows), allow(dead_code))]
pub const LANGID_VI: u16 = 0x042A; // vi-VN
#[cfg_attr(not(windows), allow(dead_code))]
pub const LANGID_EN: u16 = 0x0409; // en-US — chỉ còn để gỡ bản đăng ký cũ.

/// Ngôn ngữ đăng ký TextVN. Chỉ vi-VN: nằm thêm trong en-US (cạnh bàn phím US) thì phím
/// tắt đổi bố cục mặc định của Windows (Ctrl + Shift) nhảy qua lại giữa US và TextVN —
/// đúng phím chuyển V/E của TextVN (xem `tray/src/hotkey.rs`).
#[cfg_attr(not(windows), allow(dead_code))]
pub const REGISTER_LANGS: [(&str, u16); 1] = [("VI", LANGID_VI)];

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

    for name in &["textvn-tsf.dll", "textvn-tsf.dll", "textvn_win_tsf.dll"] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Không tìm thấy textvn-tsf.dll (hoặc textvn-tsf.dll) trong {}\n\
         Dùng --dll <path> để chỉ định thủ công hoặc chạy `cargo build` trước.",
        dir.display()
    ))
}

// ─── Ghi log ra console + file ───────────────────────────────────────────────────────────────────

#[cfg_attr(not(windows), allow(dead_code))]
fn say(msg: &str) {
    use std::io::Write;
    println!("{msg}");
    // Ghi log vào %LOCALAPPDATA%\TextVN\logs\register.log (output khi elevated bị ẩn console)
    if let Some(base) = std::env::var_os("LOCALAPPDATA") {
        let mut dir = PathBuf::from(base);
        dir.push("TextVN");
        dir.push("logs");
        let _ = std::fs::create_dir_all(&dir);
        dir.push("register.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir)
        {
            let _ = writeln!(f, "[pid={}] {msg}", std::process::id());
        }
    }
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
                "  Registry create {path} → FAIL {:#010x}",
                create.0
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
        };
        // SAFETY: key hợp lệ.
        let _ = unsafe { RegCloseKey(key) };
        if status != ERROR_SUCCESS {
            say(&format!(
                "  Registry write {path} → FAIL {:#010x}",
                status.0
            ));
            return false;
        }
        true
    }

    enum RegValue<'a> {
        None,
        Sz(&'a str),
        Dword(u32),
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

    fn delete_tree(root: HKEY, path: &str) {
        let subkey = wide(path);
        // SAFETY: buffer nul-terminated.
        let status = unsafe { RegDeleteTreeW(root, PCWSTR(subkey.as_ptr())) };
        if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
            say(&format!("  Registry delete {path} → OK"));
        } else {
            say(&format!(
                "  Registry delete {path} → FAIL {:#010x}",
                status.0
            ));
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
                return false;
            };
            type Pfn = unsafe extern "system" fn(*const u16, u32) -> i32;
            let pfn: Pfn = std::mem::transmute(fp);
            let spec = layout_spec(lang);
            let spec_w = wide(&spec);
            let ok = pfn(spec_w.as_ptr(), flags) != 0;
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

    /// Đăng ký profile + category qua API TSF (ghi HKLM → cần quyền admin).
    fn register_with_tsf_api(desc: &str, icon: &str) -> bool {
        let desc_w = wide(desc);
        let icon_w = wide(icon);
        // Slice đúng độ dài nhưng allocation có nul đệm sau (S3-2: TSF đọc wcslen()).
        let desc_s = &desc_w[..desc_w.len() - 1];
        let icon_s = &icon_w[..icon_w.len() - 1];
        // SAFETY: COM đã init; mọi interface do CoCreateInstance trả về.
        let result = (|| -> Result<()> {
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
                let cat: ITfCategoryMgr =
                    CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
                for (name, guid) in [
                    ("TIP_KEYBOARD", GUID_TFCAT_TIP_KEYBOARD),
                    ("IMMERSIVESUPPORT", GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT),
                    ("SYSTRAYSUPPORT", GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT),
                ] {
                    cat.RegisterCategory(&CLSID_TIP, &guid, &CLSID_TIP)?;
                    say(&format!("  RegisterCategory({name}) → OK"));
                }
                Ok(())
            }
        })();
        match result {
            Ok(()) => true,
            Err(e) => {
                say(&format!(
                    "  Đăng ký qua API TSF → {:#010x} (cần quyền admin cho HKLM)",
                    e.code().0
                ));
                false
            }
        }
    }

    /// Fallback per-user (P1-1 §8): ghi đúng layout TIP của CTF dưới HKCU khi API
    /// không ghi được HKLM. Không đụng key của TIP khác.
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
        for cat in [CAT_TIP_KEYBOARD, CAT_IMMERSIVE, CAT_SYSTRAY] {
            for key in ctf_category_keys(cat) {
                ok &= set_reg_value(HKEY_CURRENT_USER, &key, None, RegValue::None);
            }
        }
        say(&format!(
            "  CTF TIP per-user (HKCU) → {}",
            if ok { "OK" } else { "FAIL" }
        ));
        ok
    }

    /// Kích hoạt profile cho cả session (không chỉ thread của CLI).
    fn activate_for_session() {
        // SAFETY: COM đã init.
        unsafe {
            let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) else {
                return;
            };
            let r = mgr.ActivateProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                LANGID_VI,
                &CLSID_TIP,
                &PROFILE_GUID,
                HKL::default(),
                TF_IPPMF_FORSESSION | TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE,
            );
            match r {
                Ok(()) => say("  ActivateProfile(VI, session) → OK"),
                Err(e) => say(&format!("  ActivateProfile(VI) → {:#010x}", e.code().0)),
            }
        }
    }

    pub fn do_register(dll_path: &Path, no_taskbar: bool, scope: Scope) -> i32 {
        say(&format!("=== TextVN register ({}) ===", scope.label()));
        let dll_s = dll_path.to_string_lossy().to_string();
        say(&format!("DLL: {dll_s}"));

        // Bước 1: COM server (bắt buộc — thiếu thì TSF không tạo được TIP).
        let root = scope.root();
        let k = clsid_key();
        let inproc = format!(r"{k}\InprocServer32");
        let ok = set_reg_value(root, &k, None, RegValue::Sz("TextVN TSF"))
            && set_reg_value(root, &inproc, None, RegValue::Sz(&dll_s))
            && set_reg_value(
                root,
                &inproc,
                Some("ThreadingModel"),
                RegValue::Sz("Apartment"),
            );
        if !ok {
            if scope == Scope::Machine {
                say("FAIL: không ghi được HKLM — chạy lại với quyền Administrator");
                return 3;
            }
            say("FAIL: không ghi được registry CLSID");
            return 1;
        }
        say(&format!("  COM server {} → OK", scope.label()));

        if !com_init() {
            say("!! CoInitializeEx fail");
            return 1;
        }

        grant_appcontainer_read(dll_path);

        // Bước 2: profile + category. Mỗi bước độc lập — bản trước dừng ở
        // `Register()?` (HKLM) nên cài per-user KHÔNG bao giờ tới InstallLayoutOrTip.
        let icon = icon_path(dll_path);
        let api_ok = register_with_tsf_api("TextVN", &icon);
        let machine_registered = reg_key_exists(HKEY_LOCAL_MACHINE, &ctf_tip_key());
        if !api_ok && !machine_registered {
            if scope == Scope::Machine {
                return 3;
            }
            register_ctf_per_user("TextVN", &icon);
        }

        // Bước 3: danh sách bàn phím của user (HKCU, không cần admin).
        if no_taskbar {
            call_layout_or_tip(LANGID_VI, ILOT_UNINSTALL, "UNINSTALL");
            call_layout_or_tip(LANGID_EN, ILOT_UNINSTALL, "UNINSTALL");
        } else {
            call_layout_or_tip(LANGID_VI, ILOT_DEFPROFILE, "DEFPROFILE");
            // Bản trước còn thêm TextVN vào en-US: gỡ để Ctrl + Shift không đổi sang US.
            call_layout_or_tip(LANGID_EN, ILOT_UNINSTALL, "UNINSTALL");
        }

        // Bước 4: kích hoạt ngay cho session (kể cả bản tray-only).
        activate_for_session();

        if no_taskbar {
            say("=== Đăng ký hoàn tất (tray-only, profile đã được kích hoạt). ===");
        } else {
            say("=== Đăng ký hoàn tất. Dùng Win+Space để chọn TextVN. ===");
        }
        0
    }

    pub fn do_unregister(scope: Scope) -> i32 {
        say(&format!("=== TextVN unregister ({}) ===", scope.label()));

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

        // Bước 3: registry của chính TextVN.
        delete_tree(HKEY_CURRENT_USER, &ctf_tip_key());
        delete_tree(HKEY_CURRENT_USER, &clsid_key());
        if scope == Scope::Machine {
            delete_tree(HKEY_LOCAL_MACHINE, &clsid_key());
        }

        say("=== Hủy đăng ký hoàn tất. ===");
        0
    }

    /// COM server trỏ tới DLL còn tồn tại + profile TIP có ở HKLM hoặc fallback HKCU.
    pub fn registration_ok() -> bool {
        let inproc = format!(r"{}\InprocServer32", clsid_key());
        let server_ok = reg_read_string(HKEY_CURRENT_USER, &inproc)
            .or_else(|| reg_read_string(HKEY_LOCAL_MACHINE, &inproc))
            .is_some_and(|p| Path::new(&p).is_file());
        server_ok
            && (reg_key_exists(HKEY_LOCAL_MACHINE, &ctf_tip_key())
                || reg_key_exists(HKEY_CURRENT_USER, &ctf_tip_key()))
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
        0
    }
}

// ─── Public API (cross-platform stubs cho non-Windows) ───────────────────────────────────────────

/// Đăng ký TSF TIP. `scope`: `"user"` (mặc định, không cần admin) | `"machine"` (HKLM, cần admin).
/// Exit code: 0 thành công, 1 lỗi đăng ký, 2 lỗi tham số, 3 cần quyền Administrator.
pub fn register_tip(scope: &str, dll: Option<&Path>, no_taskbar: bool) -> i32 {
    #[cfg(windows)]
    {
        let scope = match scope {
            "user" => win_impl::Scope::User,
            "machine" => win_impl::Scope::Machine,
            _ => return 2,
        };
        let dll_path = match resolve_dll_path(dll) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
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

/// Hủy đăng ký TSF TIP. Trả về exit code: 0 thành công, 1 lỗi, 2 lỗi tham số.
pub fn unregister_tip(scope: &str) -> i32 {
    #[cfg(windows)]
    {
        match scope {
            "user" => win_impl::do_unregister(win_impl::Scope::User),
            "machine" => win_impl::do_unregister(win_impl::Scope::Machine),
            _ => 2,
        }
    }

    #[cfg(not(windows))]
    {
        let _ = scope;
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
}
