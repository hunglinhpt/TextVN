// SPDX-License-Identifier: GPL-3.0-or-later
//! SPK-ARM64X probe — trả lời câu hỏi then chốt của ARM-W2/ARM-W3 (P0-5 §5, P0-7 Luồng W):
//!
//! Trên Windows 11 ARM64, MỘT CLSID TIP (một khoá InprocServer32 trong registry 64-bit) phải
//! được nạp bởi cả tiến trình ARM64 native lẫn tiến trình x64 giả lập. Đường đề xuất: DLL
//! **ARM64X pure forwarder** (tiền lệ Weasel `arm64x_wrapper`) chuyển export sang
//! `tsf_min_arm64.dll` / `tsf_min_x64.dll` NẰM CẠNH nó. Probe kiểm:
//!
//! 1. `LoadLibraryExW(<đường dẫn đầy đủ>, LOAD_WITH_ALTERED_SEARCH_PATH)` — cách COM nạp
//!    InprocServer32 — từ thư mục làm việc KHÁC thư mục DLL: forwarder có tìm thấy DLL đích
//!    cạnh nó không (TextVN cài per-user, không vào System32 như Weasel)?
//! 2. `LoadLibraryW` (không cờ) — đối chứng.
//! 3. `DllGetClassObject(CLSID) → IClassFactory::CreateInstance(ITfTextInputProcessor)`.
//! 4. Module đích đúng kiến trúc tiến trình (x64 → `_x64`, ARM64 → `_arm64`).
//!
//! Không ghi registry, không đăng ký TIP — chỉ nạp DLL trong tiến trình của chính probe.
//! Dùng: `arm64x-probe <đường dẫn DLL> [--plain]`; exit 0 = PASS.

use std::ffi::c_void;

use windows::core::{s, Interface, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Com::{CoInitializeEx, IClassFactory, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleW, GetProcAddress, LoadLibraryExW, LoadLibraryW, LOAD_LIBRARY_FLAGS,
    LOAD_WITH_ALTERED_SEARCH_PATH,
};
use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE;
use windows::Win32::System::Threading::{GetCurrentProcess, IsWow64Process2};
use windows::Win32::UI::TextServices::ITfTextInputProcessor;

type DllGetClassObjectFn =
    unsafe extern "system" fn(*const GUID, *const GUID, *mut *mut c_void) -> HRESULT;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn machine_name(m: IMAGE_FILE_MACHINE) -> &'static str {
    match m.0 {
        0 => "native",
        0x014c => "x86",
        0x8664 => "x64",
        0xAA64 => "ARM64",
        _ => "other",
    }
}

fn module_loaded(name: &str) -> bool {
    let w = wide(name);
    unsafe { GetModuleHandleW(PCWSTR(w.as_ptr())) }.is_ok()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("dùng: arm64x-probe <đường dẫn DLL> [--plain]");
        std::process::exit(2);
    };
    let plain = args.iter().any(|a| a == "--plain");
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        "other"
    };

    let (mut proc_m, mut native_m) = (IMAGE_FILE_MACHINE(0), IMAGE_FILE_MACHINE(0));
    let _ = unsafe { IsWow64Process2(GetCurrentProcess(), &mut proc_m, Some(&mut native_m)) };
    let mut line = format!(
        "SPK-ARM64X arch={arch} process={} host={} mode={}",
        machine_name(proc_m),
        machine_name(native_m),
        if plain {
            "LoadLibraryW"
        } else {
            "LoadLibraryExW+ALTERED"
        }
    );

    let w = wide(path);
    let load: windows::core::Result<HMODULE> = unsafe {
        if plain {
            LoadLibraryW(PCWSTR(w.as_ptr()))
        } else {
            LoadLibraryExW(
                PCWSTR(w.as_ptr()),
                None,
                LOAD_LIBRARY_FLAGS(LOAD_WITH_ALTERED_SEARCH_PATH.0),
            )
        }
    };
    let module = match load {
        Ok(m) => m,
        Err(e) => {
            println!("{line} load=FAIL hr={:#010x}", e.code().0);
            std::process::exit(1);
        }
    };
    line.push_str(" load=ok");

    let Some(proc_addr) = (unsafe { GetProcAddress(module, s!("DllGetClassObject")) }) else {
        println!("{line} export=FAIL");
        std::process::exit(1);
    };
    // SAFETY: export do chính spike tsf-min định nghĩa với đúng chữ ký này.
    let get_class: DllGetClassObjectFn = unsafe { std::mem::transmute(proc_addr) };

    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let mut raw: *mut c_void = std::ptr::null_mut();
    let hr = unsafe { get_class(&tsf_min::CLSID_TEXTVN_TIP, &IClassFactory::IID, &mut raw) };
    if hr.is_err() || raw.is_null() {
        println!("{line} factory=FAIL hr={:#010x}", hr.0);
        std::process::exit(1);
    }
    // SAFETY: DllGetClassObject trả con trỏ IClassFactory đã AddRef (S_OK, khác null).
    let factory = unsafe { IClassFactory::from_raw(raw) };
    line.push_str(" factory=ok");
    match unsafe { factory.CreateInstance::<_, ITfTextInputProcessor>(None) } {
        Ok(_tip) => line.push_str(" tip=ok"),
        Err(e) => {
            println!("{line} tip=FAIL hr={:#010x}", e.code().0);
            std::process::exit(1);
        }
    }

    let x64 = module_loaded("tsf_min_x64.dll");
    let a64 = module_loaded("tsf_min_arm64.dll");
    let target = match (x64, a64) {
        (true, false) => "tsf_min_x64.dll",
        (false, true) => "tsf_min_arm64.dll",
        (false, false) => "self",
        (true, true) => "BOTH",
    };
    line.push_str(&format!(" target={target}"));
    // Qua forwarder: đúng một DLL đích, đúng kiến trúc. Nạp thẳng DLL thường ("self") chỉ dùng
    // cho smoke cục bộ trên x64.
    let ok = match (arch, target) {
        ("x86_64", "tsf_min_x64.dll") | ("aarch64", "tsf_min_arm64.dll") => true,
        (_, "self") => std::env::var_os("SPK_ARM64X_ALLOW_SELF").is_some(),
        _ => false,
    };
    println!("{line} => {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
