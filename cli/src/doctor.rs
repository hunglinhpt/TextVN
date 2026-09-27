// SPDX-License-Identifier: GPL-3.0-or-later
//! Module chẩn đoán môi trường và xuất báo cáo `vietime doctor [--json] [--export <path>]` (WIN-058 / P1-4 §9).
//!
//! Chẩn đoán:
//! 1. FFI ABI struct sizes (P0-2 §6)
//! 2. Cấu hình người dùng `%APPDATA%\VietIME\config.json` (redacted đường dẫn khi export — Rule S2)
//! 3. Trạng thái IPC Named Pipe `\\.\pipe\vietime-ipc-v1`
//! 4. Trạng thái tiến trình `vietime-hook.exe` và `vietime-tray.exe`
//! 5. Trạng thái đăng ký TIP (Windows Text Services Framework)
//!
//! Xuất file ZIP chẩn đoán:
//! Chứa `version.json`, `config.redacted.json`, `system_info.json`, `hook_stats.json`, `tsf_tail.log`.
//! Tuân thủ nghiêm ngặt Rule S2: Tuyệt đối không chứa text người dùng đã gõ.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct DoctorReport {
    pub abi_version: u32,
    pub sizes_ok: bool,
    pub key_size: usize,
    pub result_size: usize,
    pub context_size: usize,
    pub config_path: Option<PathBuf>,
    pub config_status: String,
    pub data_dir_exists: bool,
    pub pipe_listening: bool,
    pub hook_running: bool,
    pub tray_running: bool,
    pub tip_registered: bool,
    pub problems: Vec<String>,
}

pub fn run_doctor(json_out: bool, export_path: Option<&Path>) -> i32 {
    let report = collect_report();

    if let Some(dest) = export_path {
        match export_diagnostics_zip(&report, dest) {
            Ok(bytes_written) => {
                println!(
                    "Đã xuất file chẩn đoán: {} ({} bytes)",
                    dest.display(),
                    bytes_written
                );
            }
            Err(e) => {
                eprintln!("error: không thể xuất file zip chẩn đoán: {e}");
                return 2;
            }
        }
    }

    if json_out {
        println!("{}", report.to_json());
    } else {
        report.print_text();
    }

    if report.problems.is_empty() {
        0
    } else {
        for p in &report.problems {
            eprintln!("doctor: {p}");
        }
        1
    }
}

pub fn collect_report() -> DoctorReport {
    let (key_size, result_size, context_size, sizes_ok) = abi_sizes();
    let mut problems = Vec::new();

    if !sizes_ok {
        problems.push("ABI struct sizes lệch (P0-2 §6)".into());
    }

    let cfg_path = config_path();
    let mut config_status = "chưa có (chạy default)".to_string();
    if let Some(p) = &cfg_path {
        if p.exists() {
            config_status = match std::fs::read_to_string(p) {
                Ok(text) => match vietime_config::parse_config(&text) {
                    Ok(_) => "hợp lệ".into(),
                    Err(e) => {
                        problems.push("config.json không hợp lệ (P0-3 §1.3)".into());
                        format!("KHÔNG hợp lệ ({e}) → engine chạy default")
                    }
                },
                Err(e) => {
                    problems.push("config.json không đọc được".into());
                    format!("không đọc được ({e})")
                }
            };
        }
    }

    let data_dir_exists = PathBuf::from("data").is_dir();
    let pipe_listening = check_pipe_listening();
    let hook_running = check_process_running("vietime-hook.exe");
    let tray_running = check_process_running("vietime-tray.exe");
    let tip_registered = check_tip_registered();

    DoctorReport {
        abi_version: vietime_ffi::IME_ABI_VERSION,
        sizes_ok,
        key_size,
        result_size,
        context_size,
        config_path: cfg_path,
        config_status,
        data_dir_exists,
        pipe_listening,
        hook_running,
        tray_running,
        tip_registered,
        problems,
    }
}

impl DoctorReport {
    pub fn print_text(&self) {
        println!("VietIME doctor (Windows)");
        println!(
            "  ABI         : key={} result={} context={} (kỳ vọng 20/532) — {}",
            self.key_size,
            self.result_size,
            self.context_size,
            if self.sizes_ok { "OK" } else { "LỆCH" }
        );
        println!(
            "  config path : {}",
            self.config_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(không xác định)".into())
        );
        println!("  config      : {}", self.config_status);
        println!(
            "  data/       : {}",
            if self.data_dir_exists {
                "tồn tại"
            } else {
                "chưa có"
            }
        );
        println!(
            "  IPC pipe    : {}",
            if self.pipe_listening {
                "đang lắng nghe (\\.\\pipe\\vietime-ipc-v1)"
            } else {
                "chưa kết nối (tray chưa chạy)"
            }
        );
        println!(
            "  Tray proc   : {}",
            if self.tray_running {
                "đang chạy"
            } else {
                "không hoạt động"
            }
        );
        println!(
            "  Hook proc   : {}",
            if self.hook_running {
                "đang chạy"
            } else {
                "không hoạt động"
            }
        );
        println!(
            "  TIP registry: {}",
            if self.tip_registered {
                "đã đăng ký"
            } else {
                "chưa đăng ký"
            }
        );
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"abi\":{},\"sizes_ok\":{},\"key\":{},\"result\":{},\"context\":{},\"config_status\":\"{}\",\"pipe\":{},\"tray\":{},\"hook\":{},\"tip\":{},\"problems\":{}}}",
            self.abi_version,
            self.sizes_ok,
            self.key_size,
            self.result_size,
            self.context_size,
            self.config_status,
            self.pipe_listening,
            self.tray_running,
            self.hook_running,
            self.tip_registered,
            self.problems.len()
        )
    }
}

/// Xuất ZIP lưu file chẩn đoán theo chuẩn PKZIP Stored (Zero Dependency).
pub fn export_diagnostics_zip(report: &DoctorReport, out_path: &Path) -> std::io::Result<usize> {
    let mut zip_entries: Vec<(String, Vec<u8>)> = Vec::new();

    // 1. version.json
    let version_info = format!(
        "{{\n  \"app_version\": \"{}\",\n  \"abi_version\": {},\n  \"os\": \"{}\",\n  \"arch\": \"{}\",\n  \"build\": \"{}\"\n}}\n",
        env!("CARGO_PKG_VERSION"),
        report.abi_version,
        std::env::consts::OS,
        std::env::consts::ARCH,
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );
    zip_entries.push(("version.json".to_string(), version_info.into_bytes()));

    // 2. config.redacted.json
    let mut config_text = String::from("{}");
    if let Some(p) = &report.config_path {
        if let Ok(raw) = std::fs::read_to_string(p) {
            config_text = redact_sensitive_paths(&raw);
        }
    }
    zip_entries.push(("config.redacted.json".to_string(), config_text.into_bytes()));

    // 3. system_info.json
    let sys_info = format!(
        "{{\n  \"sizes_ok\": {},\n  \"pipe_listening\": {},\n  \"tray_running\": {},\n  \"hook_running\": {},\n  \"tip_registered\": {},\n  \"problems\": {:?}\n}}\n",
        report.sizes_ok,
        report.pipe_listening,
        report.tray_running,
        report.hook_running,
        report.tip_registered,
        report.problems
    );
    zip_entries.push(("system_info.json".to_string(), sys_info.into_bytes()));

    // 4. hook_stats.json (nếu có)
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let stats_path = PathBuf::from(local_appdata)
            .join("VietIME")
            .join("hook-stats.json");
        if let Ok(content) = std::fs::read(stats_path) {
            zip_entries.push(("hook_stats.json".to_string(), content));
        }
    }

    // 5. tsf_tail.log (200 dòng cuối từ log)
    let log_tail = collect_log_tail(200);
    zip_entries.push(("tsf_tail.log".to_string(), log_tail.into_bytes()));

    // Ghi file ZIP
    let zip_bytes = build_pkzip(&zip_entries)?;
    let mut file = File::create(out_path)?;
    file.write_all(&zip_bytes)?;
    Ok(zip_bytes.len())
}

/// Redact đường dẫn người dùng (`C:\Users\<username>` -> `C:\Users\<REDACTED>`) theo Rule S2.
pub fn redact_sensitive_paths(input: &str) -> String {
    let mut out = input.to_string();
    if let Some(username) = std::env::var_os("USERNAME") {
        if let Some(name_str) = username.to_str() {
            if !name_str.is_empty() {
                out = out.replace(name_str, "<REDACTED_USER>");
            }
        }
    }
    out
}

/// Lấy 200 dòng cuối từ log TSF / VietIME (không log text gõ - S2).
fn collect_log_tail(max_lines: usize) -> String {
    let mut log_path = None;
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let p = PathBuf::from(local_appdata)
            .join("VietIME")
            .join("logs")
            .join("tsf-min.log");
        if p.exists() {
            log_path = Some(p);
        }
    }

    let Some(path) = log_path else {
        return "# Không tìm thấy file log TSF.\n".to_string();
    };

    let Ok(content) = std::fs::read_to_string(path) else {
        return "# Không đọc được file log.\n".to_string();
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n") + "\n"
}

/// Đường dẫn `config.json` per-OS (P0-3 §1).
pub fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("VietIME").join("config.json"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/VietIME/config.json"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/VietIME/config.json"))
    }
}

pub fn abi_sizes() -> (usize, usize, usize, bool) {
    let key = std::mem::size_of::<vietime_ffi::ime_key_v1>();
    let result = std::mem::size_of::<vietime_ffi::ime_result_v1>();
    let context = std::mem::size_of::<vietime_ffi::ime_context_v1>();
    let ok = key == 20 && result == 532;
    (key, result, context, ok)
}

fn check_pipe_listening() -> bool {
    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(r"\\.\pipe\vietime-ipc-v1")
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn check_process_running(exe_name: &str) -> bool {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("tasklist");
        cmd.args(["/FI", &format!("IMAGENAME eq {exe_name}"), "/NH"]);
        if let Ok(output) = cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_lowercase().contains(&exe_name.to_lowercase())
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        let _ = exe_name;
        false
    }
}

fn check_tip_registered() -> bool {
    #[cfg(windows)]
    {
        // Kiểm tra CLSID VietIME trong HKCU\Software\Classes\CLSID
        let clsid = "{3E076C56-D752-44BB-9321-AEF4955AA3A6}";
        let path = format!(r"Software\Classes\CLSID\{clsid}");
        let mut cmd = std::process::Command::new("reg");
        cmd.args(["query", &format!(r"HKCU\{path}")]);
        if let Ok(output) = cmd.output() {
            output.status.success()
        } else {
            false
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

// =========================================================================
// Zero-Dependency ZIP Writer (Chuẩn PKZIP Stored Mode 0)
// =========================================================================

pub fn build_pkzip(entries: &[(String, Vec<u8>)]) -> std::io::Result<Vec<u8>> {
    let mut zip_data = Vec::new();
    let mut cd_entries = Vec::new();

    for (name, data) in entries {
        let name_bytes = name.as_bytes();
        let crc = crc32(data);
        let size = data.len() as u32;
        let local_header_offset = zip_data.len() as u32;

        // --- Local File Header ---
        zip_data.extend_from_slice(&0x04034b50u32.to_le_bytes()); // signature
        zip_data.extend_from_slice(&20u16.to_le_bytes()); // version needed
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // flags
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // compression (stored = 0)
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // mod time
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // mod date
        zip_data.extend_from_slice(&crc.to_le_bytes()); // crc32
        zip_data.extend_from_slice(&size.to_le_bytes()); // compressed size
        zip_data.extend_from_slice(&size.to_le_bytes()); // uncompressed size
        zip_data.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // name len
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // extra len
        zip_data.extend_from_slice(name_bytes); // filename
        zip_data.extend_from_slice(data); // file payload

        // Lưu thông tin cho Central Directory
        cd_entries.push((name_bytes.to_vec(), crc, size, local_header_offset));
    }

    let cd_start_offset = zip_data.len() as u32;

    // --- Central Directory Header ---
    for (name_bytes, crc, size, offset) in &cd_entries {
        zip_data.extend_from_slice(&0x02014b50u32.to_le_bytes()); // signature
        zip_data.extend_from_slice(&20u16.to_le_bytes()); // version made by
        zip_data.extend_from_slice(&20u16.to_le_bytes()); // version needed
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // flags
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // compression = 0
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // mod time
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // mod date
        zip_data.extend_from_slice(&crc.to_le_bytes()); // crc32
        zip_data.extend_from_slice(&size.to_le_bytes()); // compressed size
        zip_data.extend_from_slice(&size.to_le_bytes()); // uncompressed size
        zip_data.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // name len
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // extra len
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // comment len
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // disk start
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        zip_data.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        zip_data.extend_from_slice(&offset.to_le_bytes()); // local header offset
        zip_data.extend_from_slice(name_bytes); // filename
    }

    let cd_size = (zip_data.len() as u32) - cd_start_offset;
    let entry_count = cd_entries.len() as u16;

    // --- End of Central Directory Record (EOCD) ---
    zip_data.extend_from_slice(&0x06054b50u32.to_le_bytes()); // signature
    zip_data.extend_from_slice(&0u16.to_le_bytes()); // disk number
    zip_data.extend_from_slice(&0u16.to_le_bytes()); // start disk
    zip_data.extend_from_slice(&entry_count.to_le_bytes()); // entries this disk
    zip_data.extend_from_slice(&entry_count.to_le_bytes()); // total entries
    zip_data.extend_from_slice(&cd_size.to_le_bytes()); // cd size
    zip_data.extend_from_slice(&cd_start_offset.to_le_bytes()); // cd offset
    zip_data.extend_from_slice(&0u16.to_le_bytes()); // comment len

    Ok(zip_data)
}

/// Tính CRC32 chuẩn ISO 3309 / IEEE 802.3
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if (crc & 1) != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_standard_vector() {
        // Test vector chuẩn: CRC32 của "123456789" luôn là 0xCBF43926
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn pkzip_builder_creates_valid_structure() {
        let files = vec![
            ("hello.txt".to_string(), b"Hello VietIME".to_vec()),
            ("sub/data.json".to_string(), b"{\"status\":\"ok\"}".to_vec()),
        ];

        let zip = build_pkzip(&files).unwrap();
        assert!(!zip.is_empty());

        // Kiểm tra magic header PK\x03\x04
        assert_eq!(&zip[0..4], b"PK\x03\x04");

        // Tìm magic EOCD PK\x05\x06
        let eocd_pos = zip.windows(4).position(|w| w == b"PK\x05\x06");
        assert!(eocd_pos.is_some(), "EOCD signature not found in ZIP");
    }

    #[test]
    fn redact_paths_replaces_username() {
        if let Some(user) = std::env::var_os("USERNAME") {
            if let Some(user_str) = user.to_str() {
                let test_input =
                    format!(r"C:\Users\{user_str}\AppData\Roaming\VietIME\config.json");
                let redacted = redact_sensitive_paths(&test_input);
                assert!(!redacted.contains(user_str));
                assert!(redacted.contains("<REDACTED_USER>"));
            }
        }
    }

    #[test]
    fn export_diagnostics_zip_creates_zip_file_without_user_text() {
        let temp_zip =
            std::env::temp_dir().join(format!("vietime_diag_{}.zip", std::process::id()));
        let report = collect_report();

        let bytes_written = export_diagnostics_zip(&report, &temp_zip).unwrap();
        assert!(bytes_written > 0);
        assert!(temp_zip.exists());

        // Kiểm tra Rule S2: Quét nội dung zip không chứa username thô nếu có
        let zip_bytes = std::fs::read(&temp_zip).unwrap();
        let zip_content_lossy = String::from_utf8_lossy(&zip_bytes);

        if let Some(user) = std::env::var_os("USERNAME") {
            if let Some(user_str) = user.to_str() {
                if user_str.len() > 2 {
                    // Không được chứa đường dẫn thô C:\Users\<username>
                    let raw_user_path = format!(r"Users\{user_str}");
                    assert!(
                        !zip_content_lossy.contains(&raw_user_path),
                        "Diagnostics ZIP contains unredacted user path: {raw_user_path}"
                    );
                }
            }
        }

        let _ = std::fs::remove_file(&temp_zip);
    }
}
