// SPDX-License-Identifier: GPL-3.0-or-later
//! `tsf-min-register` — SPIKE WIN-002: đăng ký/hủy TIP per-user (checklist #6/#7).
//!
//! - Viết CLSID vào **HKCU**\Software\Classes\CLSID (không cần admin → checklist #6).
//! - Gọi `ITfCategoryMgr::RegisterCategory` (GUID_TFCAT_TIP_KEYBOARD) +
//!   `ITfInputProcessorProfiles::{Register, AddLanguageProfile, ActivateLanguageProfile}`
//!   (LANGID 0x042A = vi-VN) → checklist #7 (Win+Space thấy "TextVN").
//!
//! Dùng: `tsf-min-register install|uninstall|status`

use std::process::Command;

use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::UI::TextServices::*;

use tsf_min::{CLSID_TEXTVN_TIP, PROFILE_GUID};

const LANGID_VI: u16 = 0x042A;
const LANGID_EN: u16 = 0x0409;

fn guid_str(g: &GUID) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        g.data1,
        g.data2,
        g.data3,
        g.data4[0],
        g.data4[1],
        g.data4[2],
        g.data4[3],
        g.data4[4],
        g.data4[5],
        g.data4[6],
        g.data4[7]
    )
}

fn clsid_key() -> String {
    format!(
        "HKCU\\Software\\Classes\\CLSID\\{}",
        guid_str(&CLSID_TEXTVN_TIP)
    )
}

fn dll_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop();
    p.push("tsf_min.dll");
    p
}

/// Ghi log vào file (output elevated bị ẩn console) — %LOCALAPPDATA%\TextVN\logs\tsf-register.log
fn say(msg: &str) {
    use std::io::Write;
    println!("{msg}");
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let mut dir = std::path::PathBuf::from(base);
    dir.push("TextVN");
    dir.push("logs");
    let _ = std::fs::create_dir_all(&dir);
    dir.push("tsf-register.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir)
    {
        let _ = writeln!(f, "[{}] {msg}", std::process::id());
    }
}

/// `reg.exe` → in kết quả, trả về ok?
fn reg(args: &[&str]) -> bool {
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

fn com_init() -> bool {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        // S_FALSE = đã init rồi; RPC_E_CHANGED_MODE = lệch model → vẫn dùng được COM.
        hr.0 >= 0 || hr.0 == (0x8001_0106u32 as i32)
    }
}

fn install() -> Result<()> {
    println!("== tsf-min register (per-user, HKCU) ==");
    let dll = dll_path();
    println!("DLL: {}", dll.display());
    if !dll.exists() {
        println!("!! không thấy DLL — chạy `cargo build` trước (cùng target/debug)");
        return Ok(());
    }
    let dll_s = dll.to_string_lossy().to_string();

    // 1) CLSID (HKCU — checklist #6: VM không admin)
    let k = clsid_key();
    let mut ok = reg(&["add", &k, "/ve", "/d", "TextVN TSF spike", "/f"]);
    ok &= reg(&[
        "add",
        &format!("{k}\\InprocServer32"),
        "/ve",
        "/d",
        &dll_s,
        "/f",
    ]);
    ok &= reg(&[
        "add",
        &format!("{k}\\InprocServer32"),
        "/v",
        "ThreadingModel",
        "/t",
        "REG_SZ",
        "/d",
        "Apartment",
        "/f",
    ]);
    println!(
        "CHK#6 HKCU CLSID → {}",
        if ok {
            "OK (không cần admin)"
        } else {
            "FAIL"
        }
    );
    if !ok {
        return Ok(());
    }

    // 2) COM: category + language profile
    if !com_init() {
        println!("!! CoInitializeEx fail");
        return Ok(());
    }
    unsafe {
        let cat: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        let prof: ITfInputProcessorProfiles =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

        // Description TRƯỚC khi đụng — nếu đã hỏng thì có writer khác/đợt trước.
        readback(&prof, LANGID_VI, "VI trước khi sửa");

        // Dọn đăng ký cũ (hồi phục Description bị hỏng) rồi đăng ký lại sạch.
        for (tag, lang) in [("VI", LANGID_VI), ("EN", LANGID_EN)] {
            match prof.RemoveLanguageProfile(&CLSID_TEXTVN_TIP, lang, &PROFILE_GUID) {
                Ok(()) => say(&format!("  RemoveLanguageProfile({tag}) → OK")),
                Err(e) => say(&format!(
                    "  RemoveLanguageProfile({tag}) → FAIL {:#010x}",
                    e.code().0
                )),
            }
        }

        match prof.Register(&CLSID_TEXTVN_TIP) {
            Ok(()) => say("  Profiles.Register → OK"),
            Err(e) => say(&format!("  Profiles.Register → FAIL {:#010x}", e.code().0)),
        }

        // CẢNH BÁO spike: TSF có bằng chứng wcslen() tham số (desc không null-terminate
        // → ghi rác heap vào Description; icon &[] = ptr 0x2 → AV, process chết giữa chừng).
        // Mẹo: slice đúng length nhưng allocation có sẵn số 0 phía sau → hài hòa cả
        // "dùng cch" lẫn "wcslen" (2 semantics).
        let raw: Vec<u16> = "TextVN".encode_utf16().collect();
        let desc_buf: Vec<u16> = raw.iter().copied().chain(std::iter::once(0)).collect();
        let desc = &desc_buf[..raw.len()];
        let icon_pad = [0u16; 4];
        let icon = &icon_pad[..0];
        match prof.AddLanguageProfile(&CLSID_TEXTVN_TIP, LANGID_VI, &PROFILE_GUID, desc, icon, 0) {
            Ok(()) => say("  AddLanguageProfile(VI) → OK"),
            Err(e) => say(&format!(
                "  AddLanguageProfile(VI) → FAIL {:#010x}",
                e.code().0
            )),
        }
        readback(&prof, LANGID_VI, "VI sau add");

        // Đăng ký cả dưới ngôn ngữ HIỆN TẠI của session (en-US 0x0409) —
        // Keyman wiki: TIP đăng ký ở locale khác ngôn ngữ user → không load được
        // (InstallLayoutOrTip trả success nhưng không Loaded).
        match prof.AddLanguageProfile(&CLSID_TEXTVN_TIP, LANGID_EN, &PROFILE_GUID, desc, icon, 0) {
            Ok(()) => say("  AddLanguageProfile(EN) → OK"),
            Err(e) => say(&format!(
                "  AddLanguageProfile(EN) → FAIL {:#010x}",
                e.code().0
            )),
        }
        readback(&prof, LANGID_EN, "EN sau add");

        match prof.EnableLanguageProfileByDefault(
            &CLSID_TEXTVN_TIP,
            LANGID_VI,
            &PROFILE_GUID,
            true,
        ) {
            Ok(()) => say("  EnableLanguageProfileByDefault(VI) → OK"),
            Err(e) => say(&format!(
                "  EnableLanguageProfileByDefault(VI) → FAIL {:#010x}",
                e.code().0
            )),
        }
        readback(&prof, LANGID_VI, "VI sau enable");

        match prof.EnableLanguageProfileByDefault(
            &CLSID_TEXTVN_TIP,
            LANGID_EN,
            &PROFILE_GUID,
            true,
        ) {
            Ok(()) => say("  EnableLanguageProfileByDefault(EN) → OK"),
            Err(e) => say(&format!(
                "  EnableLanguageProfileByDefault(EN) → FAIL {:#010x}",
                e.code().0
            )),
        }
        readback(&prof, LANGID_EN, "EN sau enable");

        match cat.RegisterCategory(
            &CLSID_TEXTVN_TIP,
            &GUID_TFCAT_TIP_KEYBOARD,
            &CLSID_TEXTVN_TIP,
        ) {
            Ok(()) => say("  RegisterCategory(TIP_KEYBOARD) → OK"),
            Err(e) => say(&format!("  RegisterCategory → FAIL {:#010x}", e.code().0)),
        }
    }
    say("Xong (install). Dùng `activate` rồi mở Notepad gõ 'd'.");
    Ok(())
}

/// Đọc lại Description ngay sau mỗi API call — xác định call nào làm hỏng chuỗi.
fn readback(prof: &ITfInputProcessorProfiles, lang: u16, tag: &str) {
    match unsafe { prof.GetLanguageProfileDescription(&CLSID_TEXTVN_TIP, lang, &PROFILE_GUID) } {
        Ok(b) => {
            let s = b.to_string();
            let hex: String = s.encode_utf16().map(|c| format!("{c:04x} ")).collect();
            say(&format!("  readback({tag}) = {s:?} | utf16={hex}"));
        }
        Err(e) => say(&format!("  readback({tag}) FAIL {:#010x}", e.code().0)),
    }
}

fn uninstall() -> Result<()> {
    println!("== tsf-min unregister ==");
    if com_init() {
        unsafe {
            let cat: ITfCategoryMgr =
                CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
            match cat.UnregisterCategory(
                &CLSID_TEXTVN_TIP,
                &GUID_TFCAT_TIP_KEYBOARD,
                &CLSID_TEXTVN_TIP,
            ) {
                Ok(()) => println!("  UnregisterCategory → OK"),
                Err(e) => println!("  UnregisterCategory → FAIL {:#010x}", e.code().0),
            }
            let prof: ITfInputProcessorProfiles =
                CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
            match prof.RemoveLanguageProfile(&CLSID_TEXTVN_TIP, LANGID_VI, &PROFILE_GUID) {
                Ok(()) => println!("  RemoveLanguageProfile → OK"),
                Err(e) => println!("  RemoveLanguageProfile → FAIL {:#010x}", e.code().0),
            }
            match prof.Unregister(&CLSID_TEXTVN_TIP) {
                Ok(()) => println!("  Profiles.Unregister → OK"),
                Err(e) => println!("  Profiles.Unregister → FAIL {:#010x}", e.code().0),
            }
        }
    }
    let k = clsid_key();
    reg(&["delete", &k, "/f"]);
    println!("Xong.");
    Ok(())
}

fn status() {
    println!("== status ==");
    let k = clsid_key();
    println!("CLSID key: {k}");
    let _ = reg(&["query", &k]);
    if com_init() {
        unsafe {
            let prof = match CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                Ok(p) => p,
                Err(e) => {
                    println!("CoCreateInstance(profiles) FAIL {:#010x}", e.code().0);
                    return;
                }
            };
            match prof.IsEnabledLanguageProfile(&CLSID_TEXTVN_TIP, LANGID_VI, &PROFILE_GUID) {
                Ok(b) => println!("IsEnabledLanguageProfile = {}", b.0),
                Err(e) => println!("IsEnabledLanguageProfile FAIL {:#010x}", e.code().0),
            }
            let mut lang: u16 = 0;
            let mut pg = GUID::zeroed();
            match prof.GetActiveLanguageProfile(&CLSID_TEXTVN_TIP, &mut lang, &mut pg) {
                Ok(()) => println!("ActiveProfile langid={lang:#06x} guid={pg:?}"),
                Err(e) => println!("GetActiveLanguageProfile FAIL {:#010x}", e.code().0),
            }
        }
    }
}

/// ILOT_DEFPROFILE — đặt layout/tip làm default item của ngôn ngữ (MS: win32/tsf/installlayoutortip).
const ILOT_DEFPROFILE: u32 = 0x0000_0002;

/// `InstallLayoutOrTip` từ input.dll — thêm TIP vào danh sách input method của user
/// (không cần elevation; MS docs format: `0x0409:{CLSID}{ProfileGUID}`).
/// Không có import lib → LoadLibrary + GetProcAddress (theo đúng MS sample).
fn install_layout_or_tip(lang: u16) {
    unsafe {
        let hmod = match LoadLibraryW(w!("input.dll")) {
            Ok(h) => h,
            Err(e) => {
                println!("  InstallLayoutOrTip LoadLibrary FAIL {:#010x}", e.code().0);
                return;
            }
        };
        let Some(fp) = GetProcAddress(hmod, s!("InstallLayoutOrTip")) else {
            println!("  InstallLayoutOrTip: GetProcAddress = None");
            return;
        };
        type Pfn = unsafe extern "system" fn(*const u16, u32) -> i32;
        let pfn: Pfn = std::mem::transmute(fp);
        let spec = format!(
            "0x{lang:04X}:{}{}",
            guid_str(&CLSID_TEXTVN_TIP),
            guid_str(&PROFILE_GUID)
        );
        let wide: Vec<u16> = spec.encode_utf16().chain(std::iter::once(0)).collect();
        let ret = pfn(wide.as_ptr(), ILOT_DEFPROFILE);
        // BOOL: khác 0 = TRUE = thành công.
        println!(
            "  InstallLayoutOrTip({spec}, DEFPROFILE) → {}",
            if ret != 0 { "OK" } else { "FAIL" }
        );
        // Không FreeLibrary (bindings 0.61 không export) — CLI thoát ngay, vô hại.
    }
}

fn activate() -> Result<()> {
    println!("== activate (session, không cần admin) ==");
    if !com_init() {
        println!("!! CoInitializeEx fail");
        return Ok(());
    }
    let prof: ITfInputProcessorProfiles =
        unsafe { CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)? };
    match unsafe { prof.GetCurrentLanguage() } {
        Ok(l) => println!("  current language trước: {l:#06x}"),
        Err(e) => println!("  GetCurrentLanguage FAIL {:#010x}", e.code().0),
    }
    // GIỮ nguyên ngôn ngữ session (en-US) — ChangeCurrentLanguage(0x42A) trước đó không
    // đổi được input experience thật (WinForms vẫn thấy en-US), không cần thiết.

    // #7: InstallLayoutOrTip (input.dll, KHÔNG cần elevation — ghi HKCU user list) rồi
    // MỚI EnableLanguageProfile (keytao IMPL: thứ tự không đảo — input list update có
    // thể reset enabled flag). Format MS docs: "0x0409:{CLSID}{ProfileGUID}".
    install_layout_or_tip(LANGID_VI);
    install_layout_or_tip(LANGID_EN);
    for (tag, lang) in [("VI", LANGID_VI), ("EN", LANGID_EN)] {
        match unsafe {
            prof.EnableLanguageProfileByDefault(&CLSID_TEXTVN_TIP, lang, &PROFILE_GUID, true)
        } {
            Ok(()) => println!("  EnableLanguageProfileByDefault({tag}) → OK"),
            Err(e) => println!(
                "  EnableLanguageProfileByDefault({tag}) → FAIL {:#010x}",
                e.code().0
            ),
        }
    }

    // Session đang en-US → activate dưới 0x0409 (locale khác = không load, xem Keyman wiki).
    match unsafe { prof.ActivateLanguageProfile(&CLSID_TEXTVN_TIP, LANGID_EN, &PROFILE_GUID) } {
        Ok(()) => println!("  ActivateLanguageProfile(en-US) → OK"),
        Err(e) => println!(
            "  ActivateLanguageProfile(en-US) → FAIL {:#010x}",
            e.code().0
        ),
    }
    let mut lang: u16 = 0;
    let mut pg = GUID::zeroed();
    match unsafe { prof.GetActiveLanguageProfile(&CLSID_TEXTVN_TIP, &mut lang, &mut pg) } {
        Ok(()) => println!("  ActiveProfile langid={lang:#06x}"),
        Err(e) => println!("  GetActiveLanguageProfile FAIL {:#010x}", e.code().0),
    }
    let d =
        unsafe { prof.GetLanguageProfileDescription(&CLSID_TEXTVN_TIP, LANGID_EN, &PROFILE_GUID) };
    match d {
        Ok(b) => println!("  description = {b:?}"),
        Err(e) => println!("  GetLanguageProfileDescription FAIL {:#010x}", e.code().0),
    }
    println!("  HKCU Assemblies:");
    let _ = Command::new("reg")
        .args(["query", r"HKCU\SOFTWARE\Microsoft\CTF\Assemblies", "/s"])
        .status();
    Ok(())
}

fn main() -> Result<()> {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "status".into());
    match cmd.as_str() {
        "install" => install(),
        "uninstall" => uninstall(),
        "activate" => activate(),
        "status" => {
            status();
            Ok(())
        }
        _ => {
            println!("dùng: tsf-min-register install|uninstall|activate|status");
            Ok(())
        }
    }
}
