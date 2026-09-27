// SPDX-License-Identifier: GPL-3.0-or-later
//! Đăng ký và hủy đăng ký TSF TIP (WIN-003 / WIN-010 / P1-1 §8).
//!
//! Quy trình per-user (không cần admin):
//!   1. Ghi `HKCU\Software\Classes\CLSID\{CLSID}\InprocServer32` → đường dẫn DLL.
//!   2. Gọi COM: `ITfCategoryMgr::RegisterCategory`, `ITfInputProcessorProfiles::Register`,
//!      `AddLanguageProfile`, `EnableLanguageProfileByDefault`.
//!   3. Gọi `InstallLayoutOrTip` từ `input.dll` (không cần elevation) để thêm vào danh
//!      sách input method của user (HKCU).
//!
//! Tham khảo: `spikes/tsf-min/src/register.rs` + `docs/specs/tsf-registration-spike.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

// ─── CLSID / Profile strings (text-only; dùng được trên cả non-Windows cho tests) ──────────────

/// CLSID của VietIME TIP — khớp với `adapters/windows-tsf/src/guids.rs`.
pub const CLSID_STR: &str = "{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}";
/// Profile GUID của VietIME TIP.
pub const PROFILE_STR: &str = "{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}";

pub const LANGID_VI: u16 = 0x042A; // vi-VN
pub const LANGID_EN: u16 = 0x0409; // en-US

// ─── Helpers (cross-platform phần text) ──────────────────────────────────────────────────────────

/// `HKCU\Software\Classes\CLSID\{CLSID}` — registry key đăng ký COM per-user.
pub fn clsid_registry_key() -> String {
    format!(r"HKCU\Software\Classes\CLSID\{CLSID_STR}")
}

/// Format spec `InstallLayoutOrTip`: `"0x{lang:04X}:{CLSID}{Profile}"`.
pub fn layout_spec(lang: u16) -> String {
    format!("0x{lang:04X}:{CLSID_STR}{PROFILE_STR}")
}

/// Tìm đường dẫn DLL `vietime-tsf.dll` (hoặc `vietime_win_tsf.dll`).
///
/// Ưu tiên:
/// 1. `custom` nếu được cung cấp và file tồn tại.
/// 2. Cùng thư mục với `vietime.exe` (cạnh CLI binary).
/// 3. Lỗi nếu không tìm thấy.
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

    for name in &["vietime-tsf.dll", "vietime_win_tsf.dll"] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Không tìm thấy vietime-tsf.dll trong {}\n\
         Dùng --dll <path> để chỉ định thủ công hoặc chạy `cargo build` trước.",
        dir.display()
    ))
}

// ─── Ghi log ra console + file ───────────────────────────────────────────────────────────────────

fn say(msg: &str) {
    use std::io::Write;
    println!("{msg}");
    // Ghi log vào %LOCALAPPDATA%\VietIME\logs\register.log (output khi elevated bị ẩn console)
    if let Some(base) = std::env::var_os("LOCALAPPDATA") {
        let mut dir = PathBuf::from(base);
        dir.push("VietIME");
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

// ─── reg.exe wrapper ─────────────────────────────────────────────────────────────────────────────

fn reg_cmd(args: &[&str]) -> bool {
    let ok = Command::new("reg")
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    say(&format!(
        "  reg {} → {}",
        args.join(" "),
        if ok { "OK" } else { "FAIL" }
    ));
    ok
}

// ─── Windows-only COM/TSF impl ────────────────────────────────────────────────────────────────────

#[cfg(windows)]
mod win_impl {
    use super::*;

    use windows::core::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::Win32::UI::TextServices::*;

    // Freeze GUIDs — khớp adapters/windows-tsf/src/guids.rs
    pub const CLSID_TIP: GUID = GUID::from_u128(0x6F2B9C31_8E47_4D2A_9C84_1D5A3E70F9B8);
    pub const PROFILE_GUID: GUID = GUID::from_u128(0xC4A91F52_77B3_4E19_8A6D_2F8C0B6E5A13);

    /// `InstallLayoutOrTip` flag ILOT_DEFPROFILE — đặt profile làm default.
    const ILOT_DEFPROFILE: u32 = 0x0000_0002;

    pub fn com_init() -> bool {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            // S_FALSE (đã init rồi) hoặc RPC_E_CHANGED_MODE → vẫn dùng được
            hr.0 >= 0 || hr.0 == (0x8001_0106u32 as i32)
        }
    }

    /// Gọi `InstallLayoutOrTip` từ `input.dll` (không import lib, dùng LoadLibrary).
    pub fn install_layout_or_tip(lang: u16) {
        unsafe {
            let hmod = match LoadLibraryW(w!("input.dll")) {
                Ok(h) => h,
                Err(e) => {
                    say(&format!(
                        "  InstallLayoutOrTip LoadLibrary FAIL {:#010x}",
                        e.code().0
                    ));
                    return;
                }
            };
            let Some(fp) = GetProcAddress(hmod, s!("InstallLayoutOrTip")) else {
                say("  InstallLayoutOrTip: GetProcAddress = None");
                return;
            };
            type Pfn = unsafe extern "system" fn(*const u16, u32) -> i32;
            let pfn: Pfn = std::mem::transmute(fp);
            let spec = layout_spec(lang);
            let wide: Vec<u16> = spec.encode_utf16().chain(std::iter::once(0)).collect();
            let ret = pfn(wide.as_ptr(), ILOT_DEFPROFILE);
            say(&format!(
                "  InstallLayoutOrTip({spec}, DEFPROFILE) → {}",
                if ret != 0 { "OK" } else { "FAIL" }
            ));
            // Không FreeLibrary — process thoát ngay, vô hại
        }
    }

    pub fn do_register(dll_path: &Path) -> i32 {
        say("=== VietIME register (per-user, HKCU) ===");
        let dll_s = dll_path.to_string_lossy().to_string();
        say(&format!("DLL: {dll_s}"));

        // Bước 1: HKCU COM registry
        let k = clsid_registry_key();
        let mut ok = reg_cmd(&["add", &k, "/ve", "/d", "VietIME TSF", "/f"]);
        ok &= reg_cmd(&[
            "add",
            &format!(r"{k}\InprocServer32"),
            "/ve",
            "/d",
            &dll_s,
            "/f",
        ]);
        ok &= reg_cmd(&[
            "add",
            &format!(r"{k}\InprocServer32"),
            "/v",
            "ThreadingModel",
            "/t",
            "REG_SZ",
            "/d",
            "Apartment",
            "/f",
        ]);
        if !ok {
            say("FAIL: không ghi được registry CLSID");
            return 1;
        }
        say("CHK#6 HKCU CLSID → OK (không cần admin)");

        // Bước 2: COM TSF
        if !com_init() {
            say("!! CoInitializeEx fail");
            return 1;
        }
        // Bọc trong closure trả về Result để dùng được operator ?
        let com_result = (|| -> Result<()> {
            unsafe {
                let cat: ITfCategoryMgr =
                    CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
                let prof: ITfInputProcessorProfiles =
                    CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

                // Dọn profile cũ (tránh hỏng Description)
                for (tag, lang) in [("VI", LANGID_VI), ("EN", LANGID_EN)] {
                    match prof.RemoveLanguageProfile(&CLSID_TIP, lang, &PROFILE_GUID) {
                        Ok(()) => say(&format!("  RemoveLanguageProfile({tag}) → OK")),
                        Err(e) => say(&format!(
                            "  RemoveLanguageProfile({tag}) → {:#010x} (bỏ qua)",
                            e.code().0
                        )),
                    }
                }

                prof.Register(&CLSID_TIP)?;
                say("  Profiles.Register → OK");

                // Null-terminated buffer — tránh wcslen() heap corruption (S3-2)
                let raw: Vec<u16> = "VietIME".encode_utf16().collect();
                let desc_buf: Vec<u16> = raw.iter().copied().chain(std::iter::once(0)).collect();
                let desc = &desc_buf[..raw.len()];
                let icon_pad = [0u16; 4];
                let icon = &icon_pad[..0];

                for (tag, lang) in [("VI", LANGID_VI), ("EN", LANGID_EN)] {
                    match prof.AddLanguageProfile(&CLSID_TIP, lang, &PROFILE_GUID, desc, icon, 0) {
                        Ok(()) => say(&format!("  AddLanguageProfile({tag}) → OK")),
                        Err(e) => say(&format!(
                            "  AddLanguageProfile({tag}) → FAIL {:#010x}",
                            e.code().0
                        )),
                    }
                    match prof.EnableLanguageProfileByDefault(&CLSID_TIP, lang, &PROFILE_GUID, true)
                    {
                        Ok(()) => say(&format!("  EnableLanguageProfileByDefault({tag}) → OK")),
                        Err(e) => say(&format!(
                            "  EnableLanguageProfileByDefault({tag}) → FAIL {:#010x}",
                            e.code().0
                        )),
                    }
                }

                cat.RegisterCategory(&CLSID_TIP, &GUID_TFCAT_TIP_KEYBOARD, &CLSID_TIP)?;
                say("  RegisterCategory(TIP_KEYBOARD) → OK");

                Ok(())
            }
        })();

        if let Err(e) = com_result {
            say(&format!("COM error: {:#010x}", e.code().0));
            return 1;
        }

        // Bước 3: InstallLayoutOrTip (HKCU, không cần admin)
        install_layout_or_tip(LANGID_VI);
        install_layout_or_tip(LANGID_EN);

        say("=== Đăng ký hoàn tất. Dùng Win+Space để chọn VietIME. ===");
        0
    }

    pub fn do_unregister() -> i32 {
        say("=== VietIME unregister ===");

        if com_init() {
            let com_result = (|| -> Result<()> {
                unsafe {
                    let cat: ITfCategoryMgr =
                        CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
                    match cat.UnregisterCategory(&CLSID_TIP, &GUID_TFCAT_TIP_KEYBOARD, &CLSID_TIP) {
                        Ok(()) => say("  UnregisterCategory → OK"),
                        Err(e) => say(&format!("  UnregisterCategory → FAIL {:#010x}", e.code().0)),
                    }

                    let prof: ITfInputProcessorProfiles = CoCreateInstance(
                        &CLSID_TF_InputProcessorProfiles,
                        None,
                        CLSCTX_INPROC_SERVER,
                    )?;
                    for (tag, lang) in [("VI", LANGID_VI), ("EN", LANGID_EN)] {
                        match prof.RemoveLanguageProfile(&CLSID_TIP, lang, &PROFILE_GUID) {
                            Ok(()) => say(&format!("  RemoveLanguageProfile({tag}) → OK")),
                            Err(e) => say(&format!(
                                "  RemoveLanguageProfile({tag}) → {:#010x}",
                                e.code().0
                            )),
                        }
                    }
                    match prof.Unregister(&CLSID_TIP) {
                        Ok(()) => say("  Profiles.Unregister → OK"),
                        Err(e) => say(&format!(
                            "  Profiles.Unregister → FAIL {:#010x}",
                            e.code().0
                        )),
                    }
                    Ok(())
                }
            })();
            if let Err(e) = com_result {
                say(&format!("COM error: {:#010x}", e.code().0));
            }
        }

        // Xóa registry CLSID
        let k = clsid_registry_key();
        reg_cmd(&["delete", &k, "/f"]);

        say("=== Hủy đăng ký hoàn tất. ===");
        0
    }

    pub fn do_status() -> i32 {
        say("=== VietIME status ===");
        let k = clsid_registry_key();
        say(&format!("CLSID key: {k}"));
        let _ = Command::new("reg").args(["query", &k]).status();

        if com_init() {
            unsafe {
                let prof = match CoCreateInstance::<_, ITfInputProcessorProfiles>(
                    &CLSID_TF_InputProcessorProfiles,
                    None,
                    CLSCTX_INPROC_SERVER,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        say(&format!(
                            "CoCreateInstance(profiles) FAIL {:#010x}",
                            e.code().0
                        ));
                        return 1;
                    }
                };
                match prof.IsEnabledLanguageProfile(&CLSID_TIP, LANGID_VI, &PROFILE_GUID) {
                    Ok(b) => say(&format!("IsEnabledLanguageProfile(VI) = {}", b.as_bool())),
                    Err(e) => say(&format!(
                        "IsEnabledLanguageProfile FAIL {:#010x}",
                        e.code().0
                    )),
                }
                let mut lang: u16 = 0;
                let mut pg = GUID::zeroed();
                match prof.GetActiveLanguageProfile(&CLSID_TIP, &mut lang, &mut pg) {
                    Ok(()) => say(&format!("ActiveProfile langid={lang:#06x}")),
                    Err(e) => say(&format!(
                        "GetActiveLanguageProfile FAIL {:#010x}",
                        e.code().0
                    )),
                }
            }
        }
        0
    }
}

// ─── Public API (cross-platform stubs cho non-Windows) ───────────────────────────────────────────

/// Đăng ký TSF TIP. `scope`: `"user"` | `"machine"` (hiện chỉ hỗ trợ per-user).
/// Trả về exit code: 0 thành công, 1 lỗi đăng ký, 2 lỗi tham số.
pub fn register_tip(scope: &str, dll: Option<&Path>) -> i32 {
    if scope == "machine" {
        eprintln!("error: --scope machine chưa hỗ trợ (cần elevation riêng — xem WIN-056)");
        return 2;
    }

    #[cfg(windows)]
    {
        let dll_path = match resolve_dll_path(dll) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        };
        win_impl::do_register(&dll_path)
    }

    #[cfg(not(windows))]
    {
        let _ = dll;
        eprintln!("error: `register` chỉ hỗ trợ trên Windows");
        1
    }
}

/// Hủy đăng ký TSF TIP. Trả về exit code: 0 thành công, 1 lỗi.
pub fn unregister_tip(scope: &str) -> i32 {
    if scope == "machine" {
        eprintln!("error: --scope machine chưa hỗ trợ (cần elevation riêng — xem WIN-056)");
        return 2;
    }

    #[cfg(windows)]
    {
        win_impl::do_unregister()
    }

    #[cfg(not(windows))]
    {
        eprintln!("error: `unregister` chỉ hỗ trợ trên Windows");
        1
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
        let p = std::path::Path::new(r"C:\does\not\exist\vietime-tsf.dll");
        let res = resolve_dll_path(Some(p));
        assert!(res.is_err(), "file không tồn tại phải trả Err");
    }

    #[test]
    fn resolve_dll_path_valid_file_succeeds() {
        // Tạo file tạm để test path resolution
        let tmp = std::env::temp_dir().join("vietime-tsf-test.dll");
        std::fs::write(&tmp, b"").unwrap();
        let res = resolve_dll_path(Some(&tmp));
        std::fs::remove_file(&tmp).ok();
        assert!(res.is_ok(), "file tồn tại phải trả Ok: {res:?}");
    }

    #[test]
    fn register_machine_scope_returns_2() {
        assert_eq!(register_tip("machine", None), 2);
    }

    #[test]
    fn unregister_machine_scope_returns_2() {
        assert_eq!(unregister_tip("machine"), 2);
    }
}
