// SPDX-License-Identifier: GPL-3.0-or-later
//! Module chẩn đoán môi trường và xuất báo cáo `textvn doctor [--json] [--export <path>]` (WIN-058 / P1-4 §9).
//!
//! Chẩn đoán:
//! 1. FFI ABI struct sizes (P0-2 §6)
//! 2. Cấu hình người dùng `%APPDATA%\TextVN\config.json` (redacted đường dẫn khi export — Rule S2)
//! 3. Trạng thái IPC Named Pipe — pipe theo phiên `\\.\pipe\textvn-ipc-v1-s<phiên>`,
//!    tên cũ `\\.\pipe\textvn-ipc-v1` dự phòng (`textvn_ipc::candidate_pipe_names`)
//! 4. Trạng thái tiến trình `textvn-hook.exe` và `textvn-tray.exe`
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
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub data_dir_exists: bool,
    pub pipe_listening: bool,
    pub hook_running: bool,
    pub tray_running: bool,
    pub tip_registered: bool,
    /// Windows giữ Ctrl + Shift để đổi bàn phím (tranh phím chuyển V/E). `None` ngoài Windows.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub ctrl_shift_taken: Option<bool>,
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
                Ok(text) => match textvn_config::parse_config(&text) {
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

    // data/ nằm cạnh exe (bản cài/portable) hoặc cạnh cwd (cargo run từ gốc
    // repo) — trước đây chỉ nhìn cwd nên chạy doctor từ thư mục khác luôn
    // báo thiếu dù bản cài có đủ (audit 2026-10-01 m5).
    let data_dir_exists = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("data")))
        .into_iter()
        .chain(std::iter::once(PathBuf::from("data")))
        .any(|p| p.is_dir());
    let pipe_listening = check_pipe_listening();
    let hook_running = check_instance_mutex(r"Local\TextVNHookMutex");
    let tray_running = check_instance_mutex(r"Local\TextVNTray");
    let tip_registered = crate::register::tip_registration_ok();
    let ctrl_shift_taken = crate::register::ctrl_shift_taken_by_windows();
    if cfg!(windows) && !tip_registered {
        problems.push(
            "TSF TIP chưa đăng ký hoặc trỏ tới DLL không còn tồn tại — chạy `textvn-cli register`"
                .into(),
        );
    }

    DoctorReport {
        abi_version: textvn_ffi::IME_ABI_VERSION,
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
        ctrl_shift_taken,
        problems,
    }
}

impl DoctorReport {
    pub fn print_text(&self) {
        let os_label = if cfg!(target_os = "windows") {
            "Windows"
        } else if cfg!(target_os = "macos") {
            "macOS"
        } else {
            "Linux"
        };
        println!("TextVN doctor ({os_label})");
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
        #[cfg(target_os = "windows")]
        {
            println!(
                "  data/       : {}",
                if self.data_dir_exists {
                    "tồn tại"
                } else {
                    "chưa có"
                }
            );
            // Vòng 16 (A7): tên pipe theo phiên — không in cứng tên cũ `textvn-ipc-v1`.
            println!(
                "  IPC pipe    : {}",
                if self.pipe_listening {
                    "đang lắng nghe"
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
            if let Some(detail) = crate::register::tip_registration_summary() {
                println!("  TIP detail  : {detail}");
            }
            if let Some(taken) = self.ctrl_shift_taken {
                println!(
                    "  Ctrl+Shift  : {}",
                    if taken {
                        "Windows đang dùng để đổi bàn phím — bật \"Dành Ctrl + Shift cho TextVN\" \
                         trong Bảng điều khiển (hoặc `TextVN.exe --free-ctrl-shift`)"
                    } else {
                        "dành cho TextVN"
                    }
                );
            }
        }
        #[cfg(target_os = "linux")]
        {
            print_linux_env_summary();
        }
    }

    pub fn to_json(&self) -> String {
        let escaped_status =
            serde_json::to_string(&self.config_status).unwrap_or_else(|_| "\"\"".into());
        #[cfg(target_os = "linux")]
        {
            let env = collect_linux_env_summary();
            let jstr = |s: &str| {
                serde_json::to_string(&redact_sensitive_paths(s)).unwrap_or_else(|_| "\"\"".into())
            };
            format!(
                "{{\"abi\":{},\"sizes_ok\":{},\"key\":{},\"result\":{},\"context\":{},\"config_status\":{},\"os\":{},\"glibc\":{},\"session\":{},\"desktop\":{},\"ibus_adapter\":{},\"fcitx5_adapter\":{},\"settings_backend\":{},\"lsm\":{},\"problems\":{}}}",
                self.abi_version,
                self.sizes_ok,
                self.key_size,
                self.result_size,
                self.context_size,
                escaped_status,
                jstr(&env.os_pretty),
                jstr(&env.glibc),
                jstr(&env.session),
                jstr(&env.desktop),
                env.ibus_found,
                env.fcitx5_found,
                jstr(&env.settings_backend),
                jstr(&env.lsm),
                self.problems.len()
            )
        }
        #[cfg(not(target_os = "linux"))]
        format!(
            "{{\"abi\":{},\"sizes_ok\":{},\"key\":{},\"result\":{},\"context\":{},\"config_status\":{},\"pipe\":{},\"tray\":{},\"hook\":{},\"tip\":{},\"problems\":{}}}",
            self.abi_version,
            self.sizes_ok,
            self.key_size,
            self.result_size,
            self.context_size,
            escaped_status,
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

    // 2. config.redacted.json — gõ tắt/emoji/từ điển là NỘI DUNG người dùng tự soạn
    // (địa chỉ, số điện thoại, email…) mà file zip này được hướng dẫn đính kèm vào
    // issue công khai: chỉ giữ số mục (S2).
    let mut config_text = String::from("{}");
    if let Some(p) = &report.config_path {
        if let Ok(raw) = std::fs::read_to_string(p) {
            config_text = redact_sensitive_paths(&redact_user_content(&raw));
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

    #[cfg(target_os = "linux")]
    {
        let env = collect_linux_env_summary();
        let jstr = |s: &str| {
            serde_json::to_string(&redact_sensitive_paths(s)).unwrap_or_else(|_| "\"\"".into())
        };
        let linux_json = format!(
            "{{\n  \"os\": {},\n  \"glibc\": {},\n  \"session\": {},\n  \"desktop\": {},\n  \"lsm\": {},\n  \"ibus_found\": {},\n  \"fcitx5_found\": {},\n  \"settings_backend\": {},\n  \"ibus_running\": {},\n  \"fcitx5_running\": {},\n  \"env_d_present\": {},\n  \"el8_system_cache\": {},\n  \"el8_autostart\": {}\n}}\n",
            jstr(&env.os_pretty),
            jstr(&env.glibc),
            jstr(&env.session),
            jstr(&env.desktop),
            jstr(&env.lsm),
            env.ibus_found,
            env.fcitx5_found,
            jstr(&env.settings_backend),
            env.ibus_running,
            env.fcitx5_running,
            env.env_d_present,
            env.el8_system_cache,
            env.el8_autostart,
        );
        zip_entries.push(("linux_env.json".to_string(), linux_json.into_bytes()));
    }

    // 4. hook_stats.json (nếu có)
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let root = PathBuf::from(local_appdata);
        let primary = root.join("TextVN").join("hook-stats.json");
        let legacy = root.join("TextVN").join("hook-stats.json");
        let stats_path = if primary.exists() { primary } else { legacy };
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

/// Khoá config chứa nội dung người dùng tự soạn — chỉ xuất số mục.
const USER_CONTENT_KEYS: [&str; 3] = ["macros", "emoji", "english_words"];

/// Config cho gói chẩn đoán: thay `macros`/`emoji`/`english_words` bằng
/// `"<REDACTED: n mục>"`; file không parse được → chỉ ghi kích thước (không lộ nội dung).
pub fn redact_user_content(raw: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return format!("{{\"_unparsable_config_bytes\": {}}}\n", raw.len());
    };
    if let Some(map) = value.as_object_mut() {
        for key in USER_CONTENT_KEYS {
            if let Some(v) = map.get_mut(key) {
                let n = v.as_array().map_or(0, Vec::len);
                *v = serde_json::Value::String(format!("<REDACTED: {n} mục>"));
            }
        }
        // G8/S2: style_profile chứa từ và cặp sửa đã học của người dùng -> ẩn nội dung, giữ số đếm.
        if let Some(sp) = map.get_mut("style_profile").and_then(|v| v.as_object_mut()) {
            for sub in ["words", "corrections"] {
                if let Some(v) = sp.get_mut(sub) {
                    let n = v
                        .as_object()
                        .map_or_else(|| v.as_array().map_or(0, Vec::len), serde_json::Map::len);
                    *v = serde_json::Value::String(format!("<REDACTED: {n} mục>"));
                }
            }
        }
    }
    let mut out = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into());
    out.push('\n');
    out
}

/// Redact đường dẫn người dùng (`C:\Users\<username>`, `/home/<user>`) theo Rule S2.
pub fn redact_sensitive_paths(input: &str) -> String {
    let mut out = input.to_string();
    for env_key in ["USERNAME", "USER", "LOGNAME"] {
        if let Some(username) = std::env::var_os(env_key) {
            if let Some(name_str) = username.to_str() {
                if name_str.len() >= 2 {
                    out = out.replace(name_str, "<REDACTED_USER>");
                }
            }
        }
    }
    out
}

/// Lấy 200 dòng cuối từ log TSF / TextVN (không log text gõ - S2).
fn collect_log_tail(max_lines: usize) -> String {
    let mut log_path: Option<PathBuf> = None;
    #[cfg(target_os = "windows")]
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let root = PathBuf::from(local_appdata);
        let primary = root.join("TextVN").join("logs").join("tsf-min.log");
        if primary.exists() {
            log_path = Some(primary);
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        let p = PathBuf::from(home).join("Library/Logs/TextVN/textvn.log");
        if p.exists() {
            log_path = Some(p);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(xdg) = std::env::var_os("XDG_STATE_HOME") {
            let base = PathBuf::from(xdg);
            if base.is_absolute() {
                let p = base.join("TextVN/log/textvn.log");
                if p.exists() {
                    log_path = Some(p);
                }
            }
        }
        if log_path.is_none() {
            if let Some(home) = std::env::var_os("HOME") {
                let p = PathBuf::from(home).join(".local/state/TextVN/log/textvn.log");
                if p.exists() {
                    log_path = Some(p);
                }
            }
        }
    }

    let Some(path) = log_path else {
        #[cfg(target_os = "windows")]
        return "# Không tìm thấy file log TSF.\n".to_string();
        #[cfg(not(target_os = "windows"))]
        return "# Không tìm thấy file log TextVN.\n".to_string();
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
        std::env::var_os("APPDATA").map(|p| {
            let primary = PathBuf::from(&p).join("TextVN").join("config.json");
            let legacy = PathBuf::from(&p).join("TextVN").join("config.json");
            if !primary.exists() && legacy.exists() {
                legacy
            } else {
                primary
            }
        })
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/TextVN/config.json"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            let p = PathBuf::from(xdg);
            if p.is_absolute() {
                return Some(p.join("TextVN/config.json"));
            }
        }
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/TextVN/config.json"))
    }
}

#[cfg(target_os = "linux")]
struct LinuxEnvSummary {
    os_pretty: String,
    glibc: String,
    session: String,
    desktop: String,
    lsm: String,
    ibus_comp: String,
    fcitx_dirs: String,
    im_env: String,
    ibus_found: bool,
    fcitx5_found: bool,
    settings_backend: String,
    ibus_running: bool,
    fcitx5_running: bool,
    env_d_present: bool,
    el8_system_cache: bool,
    el8_autostart: bool,
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn detect_linux_glibc() -> String {
    extern "C" {
        fn gnu_get_libc_version() -> *const std::os::raw::c_char;
    }
    unsafe {
        let p = gnu_get_libc_version();
        if p.is_null() {
            "unknown".into()
        } else {
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}

#[cfg(all(target_os = "linux", not(target_env = "gnu")))]
fn detect_linux_glibc() -> String {
    "non-glibc".into()
}

#[cfg(target_os = "linux")]
fn linux_proc_running(target_comm: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        if !s.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if let Ok(comm) = std::fs::read_to_string(entry.path().join("comm")) {
            if comm.trim() == target_comm {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn detect_linux_lsm() -> String {
    if let Ok(v) = std::fs::read_to_string("/sys/fs/selinux/enforce") {
        return match v.trim() {
            "1" => "SELinux=Enforcing".into(),
            "0" => "SELinux=Permissive".into(),
            _ => "SELinux".into(),
        };
    }
    if let Ok(v) = std::fs::read_to_string("/sys/module/apparmor/parameters/enabled") {
        if v.trim() == "Y" {
            return "AppArmor=Enabled".into();
        }
    }
    "none".into()
}

#[cfg(target_os = "linux")]
fn collect_linux_env_summary() -> LinuxEnvSummary {
    let os_pretty = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("PRETTY_NAME="))
                .map(|v| v.trim_matches(&['"', '\''][..]).to_string())
        })
        .unwrap_or_else(|| "Linux".into());
    let env_or = |k: &str| std::env::var(k).unwrap_or_else(|_| "<chưa đặt>".into());
    let session = env_or("XDG_SESSION_TYPE");
    let desktop = env_or("XDG_CURRENT_DESKTOP");
    let ibus_comp = env_or("IBUS_COMPONENT_PATH");
    let fcitx_dirs = env_or("FCITX_ADDON_DIRS");
    let im_env = format!(
        "GTK={} · QT={} · XMOD={}",
        env_or("GTK_IM_MODULE"),
        env_or("QT_IM_MODULE"),
        env_or("XMODIFIERS")
    );

    let mut ibus_found = false;
    let mut fcitx5_found = false;
    let mut settings_backend = "thiếu".to_string();
    let arch_multiarch = format!("{}-linux-gnu", std::env::consts::ARCH);

    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let ibus_candidates = [
                bin_dir.join("../lib/textvn/textvn-ibus-engine"),
                bin_dir.join("../lib64/textvn/textvn-ibus-engine"),
                bin_dir.join(format!("../lib/{arch_multiarch}/textvn/textvn-ibus-engine")),
                bin_dir.join("../libexec/ibus-engine-textvn"),
            ];
            ibus_found = ibus_candidates.iter().any(|p| p.exists());

            let fcitx_candidates = [
                bin_dir.join("../lib/textvn/fcitx5/libtextvn-fcitx5.so"),
                bin_dir.join("../lib64/fcitx5/libtextvn-fcitx5.so"),
                bin_dir.join("../lib/fcitx5/libtextvn-fcitx5.so"),
                bin_dir.join(format!(
                    "../lib/{arch_multiarch}/fcitx5/libtextvn-fcitx5.so"
                )),
                PathBuf::from(format!(
                    "/usr/lib/{arch_multiarch}/fcitx5/libtextvn-fcitx5.so"
                )),
                PathBuf::from("/usr/lib64/fcitx5/libtextvn-fcitx5.so"),
                PathBuf::from("/usr/lib/fcitx5/libtextvn-fcitx5.so"),
            ];
            fcitx5_found = fcitx_candidates.iter().any(|p| p.exists());

            let settings_bin = bin_dir.join("textvn-settings");
            if settings_bin.exists() {
                if let Ok(out) = std::process::Command::new(&settings_bin)
                    .arg("--version")
                    .output()
                {
                    if out.status.success() {
                        let txt = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        settings_backend = if txt.is_empty() { "OK".into() } else { txt };
                    } else {
                        settings_backend = "lỗi ABI (thiếu GTK3/GTK4)".into();
                    }
                } else {
                    settings_backend = "lỗi thực thi".into();
                }
            }
        }
    }

    let xdg_cfg = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
    let env_d_present = xdg_cfg
        .as_ref()
        .is_some_and(|c| c.join("environment.d/60-textvn.conf").exists());
    let el8_autostart = xdg_cfg
        .as_ref()
        .is_some_and(|c| c.join("autostart/textvn-ibus-autostart.desktop").exists());

    LinuxEnvSummary {
        os_pretty,
        glibc: detect_linux_glibc(),
        session,
        desktop,
        lsm: detect_linux_lsm(),
        ibus_comp,
        fcitx_dirs,
        im_env,
        ibus_found,
        fcitx5_found,
        settings_backend,
        ibus_running: linux_proc_running("ibus-daemon"),
        fcitx5_running: linux_proc_running("fcitx5"),
        env_d_present,
        el8_system_cache: Path::new("/var/cache/ibus/bus/registry").exists(),
        el8_autostart,
    }
}

#[cfg(target_os = "linux")]
fn print_linux_env_summary() {
    let s = collect_linux_env_summary();
    println!("  OS          : {} (glibc {})", s.os_pretty, s.glibc);
    println!("  session     : {} ({}) · {}", s.session, s.desktop, s.lsm);
    println!(
        "  daemons     : ibus-daemon={} · fcitx5={}",
        if s.ibus_running { "chạy" } else { "tắt" },
        if s.fcitx5_running { "chạy" } else { "tắt" }
    );
    println!("  IBUS_COMP   : {}", s.ibus_comp);
    println!("  FCITX_DIRS  : {}", s.fcitx_dirs);
    println!("  IM env      : {}", s.im_env);
    println!(
        "  adapters    : ibus={} · fcitx5={} · settings={}",
        if s.ibus_found { "OK" } else { "thiếu" },
        if s.fcitx5_found { "OK" } else { "thiếu" },
        s.settings_backend
    );
    println!(
        "  integration : environment.d={} · ibus-sys-cache={} · el8-autostart={}",
        if s.env_d_present { "OK" } else { "không" },
        if s.el8_system_cache { "có" } else { "không" },
        if s.el8_autostart { "có" } else { "không" }
    );
}

pub fn abi_sizes() -> (usize, usize, usize, bool) {
    let key = std::mem::size_of::<textvn_ffi::ime_key_v1>();
    let result = std::mem::size_of::<textvn_ffi::ime_result_v1>();
    let context = std::mem::size_of::<textvn_ffi::ime_context_v1>();
    let ok = key == 20 && result == 532;
    (key, result, context, ok)
}

fn check_pipe_listening() -> bool {
    #[cfg(windows)]
    {
        for name in textvn_ipc::candidate_pipe_names() {
            if textvn_ipc::pipe_client_options().open(&name).is_ok() {
                return true;
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Tray/hook giữ mutex đơn-instance của chính TextVN trong suốt vòng đời. Mở
/// mutex đó thay vì spawn `tasklist` — không liệt kê process của người dùng
/// (hành vi AV soi) và không phụ thuộc tên file exe.
fn check_instance_mutex(name: &str) -> bool {
    #[cfg(windows)]
    {
        use windows::core::HSTRING;
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
        // SAFETY: tên là HSTRING hợp lệ; handle mở được thì đóng ngay.
        unsafe {
            match OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, &HSTRING::from(name)) {
                Ok(handle) => {
                    let _ = CloseHandle(handle);
                    true
                }
                Err(_) => false,
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = name;
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
    fn export_config_never_contains_user_authored_content() {
        let raw = r#"{"config_version":1,"method":"vni",
            "macros":[{"trigger":"dc","expand":"12 Lê Lợi, 0912345678"}],
            "emoji":[{"trigger":":x","glyph":"❌"}],
            "english_words":["secretword"],
            "style_profile":{"enabled":true,"words":{"matkhau":5},"corrections":{"sai->dung":2}}}"#;
        let out = redact_user_content(raw);
        for leaked in [
            "Lê Lợi",
            "0912345678",
            "secretword",
            "❌",
            "\"dc\"",
            "matkhau",
            "sai->dung",
        ] {
            assert!(!out.contains(leaked), "lộ `{leaked}` trong: {out}");
        }
        assert!(
            out.contains("\"method\": \"vni\""),
            "tuỳ chọn vẫn phải giữ để chẩn đoán"
        );
        assert!(out.contains("<REDACTED: 1 mục>"));
        let broken = redact_user_content("{ hỏng 0912345678");
        assert!(!broken.contains("0912345678"));
    }

    #[test]
    fn crc32_standard_vector() {
        // Test vector chuẩn: CRC32 của "123456789" luôn là 0xCBF43926
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn pkzip_builder_creates_valid_structure() {
        let files = vec![
            ("hello.txt".to_string(), b"Hello TextVN".to_vec()),
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
                let test_input = format!(r"C:\Users\{user_str}\AppData\Roaming\TextVN\config.json");
                let redacted = redact_sensitive_paths(&test_input);
                assert!(!redacted.contains(user_str));
                assert!(redacted.contains("<REDACTED_USER>"));
            }
        }
    }

    #[test]
    fn export_diagnostics_zip_creates_zip_file_without_user_text() {
        let temp_zip = std::env::temp_dir().join(format!("textvn_diag_{}.zip", std::process::id()));
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
