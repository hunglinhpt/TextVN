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
use std::time::{SystemTime, UNIX_EPOCH};

/// Một phát hiện chẩn đoán có cấu trúc (E69) — gắn mã lỗi/cảnh báo cụ thể,
/// nguyên nhân và hướng dẫn xử lý để người dùng tự khắc phục hoặc gửi báo lỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticItem {
    pub code: &'static str,
    pub severity: &'static str, // "error" | "warning"
    pub category: &'static str, // "dep" | "conflict" | "abi" | "config" | "runtime" | "env"
    pub message: String,
    pub suggestion: String,
}

impl DiagnosticItem {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code,
            "severity": self.severity,
            "category": self.category,
            "message": redact_sensitive_paths(&self.message),
            "suggestion": redact_sensitive_paths(&self.suggestion),
        })
    }
}

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
    /// R2-26: bản sao cấu hình cho tiến trình sandbox (ô tìm kiếm Start, ứng dụng Store):
    /// `ok` / `missing` / `no_acl` / `stale`. `None` ngoài Windows.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub sandbox_mirror: Option<&'static str>,
    pub diagnostics: Vec<DiagnosticItem>,
    pub problems: Vec<String>,
}

/// R2-26: tình trạng bản sao sandbox `<thư mục config>\sandbox\` so với `config.json` thật.
/// `stale` = khác bản lọc của config hiện tại (người dùng sửa tay khi tray tắt) — so theo giá trị
/// JSON, không theo định dạng. Hàm thuần trên nội dung (test trên Windows — `textvn-ipc` chỉ là
/// phụ thuộc Windows của CLI, R2-162).
#[cfg(windows)]
pub fn sandbox_mirror_verdict(
    real_config: Option<&str>,
    mirror_config: Option<&str>,
    mirror_state_present: bool,
    dir_sddl: Option<&str>,
) -> &'static str {
    let Some(mirror) = mirror_config else {
        return "missing";
    };
    if !mirror_state_present {
        return "missing";
    }
    if !dir_sddl.is_some_and(textvn_ipc::sandbox::sddl_grants_sandbox) {
        return "no_acl";
    }
    if let Some(expected) = real_config.and_then(textvn_config::sandbox::sandbox_config_json) {
        let a = serde_json::from_str::<serde_json::Value>(&expected).ok();
        let b = serde_json::from_str::<serde_json::Value>(mirror).ok();
        if a.is_none() || a != b {
            return "stale";
        }
    }
    "ok"
}

#[cfg(windows)]
fn sandbox_mirror_status(config_path: Option<&Path>) -> &'static str {
    let Some(dir) = config_path
        .and_then(Path::parent)
        .map(|d| d.join(textvn_ipc::sandbox::SANDBOX_DIR_NAME))
    else {
        return "missing";
    };
    let real = config_path.and_then(|p| std::fs::read_to_string(p).ok());
    let mirror = std::fs::read_to_string(dir.join("config.json")).ok();
    let sddl = textvn_ipc::sandbox::path_dacl_sddl(&dir);
    sandbox_mirror_verdict(
        real.as_deref(),
        mirror.as_deref(),
        dir.join("state.json").is_file(),
        sddl.as_deref(),
    )
}

/// Tạo đường dẫn xuất file chẩn đoán mặc định (`<Desktop hoặc HOME>/textvn-diag-<ts>.zip`).
pub fn default_export_path() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = format!("textvn-diag-{ts}.zip");
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    if let Some(h) = home {
        let desktop = h.join("Desktop");
        if desktop.is_dir() {
            return desktop.join(filename);
        }
        if h.is_dir() {
            return h.join(filename);
        }
    }
    PathBuf::from(filename)
}

#[allow(dead_code)]
pub fn run_doctor(json_out: bool, export_path: Option<&Path>) -> i32 {
    run_doctor_ext(json_out, false, export_path)
}

pub fn run_doctor_ext(json_out: bool, report_out: bool, export_path: Option<&Path>) -> i32 {
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

    if report_out {
        print!("{}", report.to_markdown_report());
    } else if json_out {
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
    let mut diagnostics = Vec::new();

    if !sizes_ok {
        let msg = "ABI struct sizes lệch (P0-2 §6)".to_string();
        problems.push(msg.clone());
        diagnostics.push(DiagnosticItem {
            code: "E_ABI_MISMATCH",
            severity: "error",
            category: "abi",
            message: msg,
            suggestion: "Cài lại đúng bản phát hành TextVN đồng bộ CLI và adapter.".into(),
        });
    }

    let cfg_path = config_path();
    let mut config_status = "chưa có (chạy default)".to_string();
    if let Some(p) = &cfg_path {
        if p.exists() {
            config_status = match std::fs::read_to_string(p) {
                Ok(text) => match textvn_config::parse_config(&text) {
                    Ok(_) => "hợp lệ".into(),
                    Err(e) => {
                        let msg = "config.json không hợp lệ (P0-3 §1.3)".to_string();
                        problems.push(msg.clone());
                        diagnostics.push(DiagnosticItem {
                            code: "E_CONFIG_INVALID",
                            severity: "error",
                            category: "config",
                            message: format!("{msg}: {e}"),
                            suggestion:
                                "Sao lưu và xóa config.json để TextVN tự tạo lại cấu hình chuẩn."
                                    .into(),
                        });
                        format!("KHÔNG hợp lệ ({e}) → engine chạy default")
                    }
                },
                Err(e) => {
                    let msg = "config.json không đọc được".to_string();
                    problems.push(msg.clone());
                    diagnostics.push(DiagnosticItem {
                        code: "E_CONFIG_UNREADABLE",
                        severity: "error",
                        category: "config",
                        message: format!("{msg}: {e}"),
                        suggestion: "Kiểm tra quyền đọc thư mục cấu hình TextVN.".into(),
                    });
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
    // Windows: hotkey đổi bố cục; Linux: Fcitx5 5.0.x gắn Ctrl+Shift vào EnumerateForwardKeys (R2-175).
    let ctrl_shift_taken =
        crate::register::ctrl_shift_taken_by_windows().or_else(linux_fcitx5_ctrl_shift_taken);
    if cfg!(windows) && !tip_registered {
        let msg =
            "TSF TIP chưa đăng ký hoặc trỏ tới DLL không còn tồn tại — chạy `textvn-cli register`"
                .to_string();
        problems.push(msg.clone());
        diagnostics.push(DiagnosticItem {
            code: "E_TSF_UNREGISTERED",
            severity: "error",
            category: "runtime",
            message: msg,
            suggestion:
                "Mở Bảng điều khiển → Hướng dẫn → [Cài & bật TSF], hoặc cài bản TextVN-setup-*-windows-x64-machine.exe."
                    .into(),
        });
    }
    #[cfg(windows)]
    let sandbox_mirror = Some(sandbox_mirror_status(cfg_path.as_deref()));
    #[cfg(not(windows))]
    let sandbox_mirror: Option<&'static str> = None;
    if let Some(state) = sandbox_mirror.filter(|s| *s != "ok") {
        diagnostics.push(DiagnosticItem {
            code: "W_SANDBOX_MIRROR",
            severity: "warning",
            category: "runtime",
            message: format!(
                "Bản sao cấu hình cho ô tìm kiếm Start / ứng dụng Store (AppContainer) đang `{state}` — các ô đó gõ Telex mặc định, không theo cấu hình và nút V/E."
            ),
            suggestion: "Thoát rồi mở lại TextVN ở khay hệ thống (tray ghi lại `%APPDATA%\\TextVN\\sandbox` khi khởi động)."
                .into(),
        });
    }
    if ctrl_shift_taken == Some(true) {
        // Cùng MỘT mã trên mọi OS (G22) — chỉ khác ai giữ phím và cách trả lại.
        let (message, suggestion) = if cfg!(windows) {
            (
                "Windows đang giữ phím Ctrl+Shift để đổi bố cục bàn phím.",
                "Bật 'Dành Ctrl + Shift cho TextVN' trong Bảng điều khiển hoặc chạy `TextVN.exe --free-ctrl-shift`.",
            )
        } else {
            (
                "Fcitx5 đang giữ phím Ctrl+Shift để đổi bộ gõ (Enumerate Input Method Forward/Backward — mặc định của Fcitx5 5.0.x trên Ubuntu 22.04, Mint 21, Debian 12): bấm Ctrl+Shift chỉ đổi sang keyboard-us, công tắc V/E của TextVN không chạy.",
                "Mở `fcitx5-configtool` → Tuỳ chọn chung (Global Options) → xoá phím ở 'Enumerate Input Method Forward' và 'Backward' (hoặc thêm `[Hotkey/EnumerateForwardKeys]` và `[Hotkey/EnumerateBackwardKeys]` để trống vào `~/.config/fcitx5/config` rồi `fcitx5 -r`); hoặc dùng Ctrl+Shift+Space để bật/tắt tiếng Việt.",
            )
        };
        diagnostics.push(DiagnosticItem {
            code: "W_CTRL_SHIFT_TAKEN",
            severity: "warning",
            category: "conflict",
            message: message.into(),
            suggestion: suggestion.into(),
        });
    }

    if let Some(tail) = textvn_config::crash::default_log_dir()
        .and_then(|d| textvn_config::crash::read_panic_log_tail(&d, 64 * 1024))
    {
        let n = count_panic_records(&tail);
        if n > 0 {
            diagnostics.push(DiagnosticItem {
                code: "W_RECENT_PANIC",
                severity: "warning",
                category: "runtime",
                message: format!("Có {n} bản ghi panic trong panic.log (ERR_PANIC)."),
                suggestion: "Xuất gói chẩn đoán (`textvn-cli --export-log`) và đính kèm khi báo lỗi tại https://github.com/hunglinhpt/TextVN/issues — panic.log không chứa nội dung gõ (G8).".into(),
            });
        }
    }

    #[cfg(target_os = "windows")]
    collect_windows_diagnostics(&mut diagnostics);

    #[cfg(target_os = "linux")]
    collect_linux_diagnostics(&mut diagnostics, &mut problems);

    #[cfg(target_os = "macos")]
    collect_macos_diagnostics(&mut diagnostics);

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
        sandbox_mirror,
        diagnostics,
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
            let win = collect_windows_env_summary();
            println!("  OS          : {}", win.os_version);
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
            if let Some(state) = self.sandbox_mirror {
                println!(
                    "  Sandbox     : {state} (bản sao cho Start search / ứng dụng Store — R2-26)"
                );
            }
            if let Some(taken) = self.ctrl_shift_taken {
                println!(
                    "  Ctrl+Shift  : {}",
                    if taken && cfg!(windows) {
                        "Windows đang dùng để đổi bàn phím — bật \"Dành Ctrl + Shift cho TextVN\" \
                         trong Bảng điều khiển (hoặc `TextVN.exe --free-ctrl-shift`)"
                    } else if taken {
                        "Fcitx5 đang dùng để đổi bộ gõ (R2-175) — xoá phím \"Enumerate Input Method \
                         Forward/Backward\" trong fcitx5-configtool, hoặc dùng Ctrl+Shift+Space"
                    } else {
                        "dành cho TextVN"
                    }
                );
            }
            if !win.conflicts.is_empty() {
                println!("  conflicts   : {}", win.conflicts.join(", "));
            }
        }
        #[cfg(target_os = "macos")]
        {
            let mac = collect_macos_env_summary();
            println!("  OS          : {}", mac.os_version);
            if !mac.conflicts.is_empty() {
                println!("  conflicts   : {}", mac.conflicts.join(", "));
            }
        }
        #[cfg(target_os = "linux")]
        {
            print_linux_env_summary();
        }
        if !self.diagnostics.is_empty() {
            println!("  chẩn đoán   : {} mục", self.diagnostics.len());
            for d in &self.diagnostics {
                let tag = if d.severity == "error" {
                    "LỖI"
                } else {
                    "CẢNH BÁO"
                };
                println!("    [{tag}] [{}] {}", d.code, d.message);
                println!("      → Gợi ý: {}", d.suggestion);
            }
        }
    }

    pub fn to_json(&self) -> String {
        let diag_json: Vec<serde_json::Value> = self
            .diagnostics
            .iter()
            .map(DiagnosticItem::to_json_value)
            .collect();
        #[cfg(target_os = "linux")]
        {
            let env = collect_linux_env_summary();
            let obj = serde_json::json!({
                "abi": self.abi_version,
                "sizes_ok": self.sizes_ok,
                "key": self.key_size,
                "result": self.result_size,
                "context": self.context_size,
                "config_status": redact_sensitive_paths(&self.config_status),
                "os": redact_sensitive_paths(&env.os_pretty),
                "kernel": redact_sensitive_paths(&env.kernel),
                "glibc": redact_sensitive_paths(&env.glibc),
                "pkg_mgr": env.pkg_mgr,
                "session": redact_sensitive_paths(&env.session),
                "desktop": redact_sensitive_paths(&env.desktop),
                "ibus_adapter": env.ibus_found,
                "fcitx5_adapter": env.fcitx5_found,
                "settings_backend": redact_sensitive_paths(&env.settings_backend),
                "lsm": redact_sensitive_paths(&env.lsm),
                "conflicts": env.conflicts,
                "diagnostics": diag_json,
                "problems": self.problems.len(),
            });
            serde_json::to_string(&obj).unwrap_or_else(|_| "{}".into())
        }
        #[cfg(target_os = "windows")]
        {
            let win = collect_windows_env_summary();
            let obj = serde_json::json!({
                "abi": self.abi_version,
                "sizes_ok": self.sizes_ok,
                "key": self.key_size,
                "result": self.result_size,
                "context": self.context_size,
                "config_status": redact_sensitive_paths(&self.config_status),
                "os": redact_sensitive_paths(&win.os_version),
                "pipe": self.pipe_listening,
                "tray": self.tray_running,
                "hook": self.hook_running,
                "tip": self.tip_registered,
                "sandbox_mirror": self.sandbox_mirror,
                "conflicts": win.conflicts,
                "diagnostics": diag_json,
                "problems": self.problems.len(),
            });
            serde_json::to_string(&obj).unwrap_or_else(|_| "{}".into())
        }
        #[cfg(target_os = "macos")]
        {
            let mac = collect_macos_env_summary();
            let obj = serde_json::json!({
                "abi": self.abi_version,
                "sizes_ok": self.sizes_ok,
                "key": self.key_size,
                "result": self.result_size,
                "context": self.context_size,
                "config_status": redact_sensitive_paths(&self.config_status),
                "os": redact_sensitive_paths(&mac.os_version),
                "conflicts": mac.conflicts,
                "diagnostics": diag_json,
                "problems": self.problems.len(),
            });
            serde_json::to_string(&obj).unwrap_or_else(|_| "{}".into())
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            let obj = serde_json::json!({
                "abi": self.abi_version,
                "sizes_ok": self.sizes_ok,
                "key": self.key_size,
                "result": self.result_size,
                "context": self.context_size,
                "config_status": redact_sensitive_paths(&self.config_status),
                "diagnostics": diag_json,
                "problems": self.problems.len(),
            });
            serde_json::to_string(&obj).unwrap_or_else(|_| "{}".into())
        }
    }

    /// Báo cáo Markdown sẵn sàng dán vào GitHub Issues (đã ẩn đường dẫn & không chứa nội dung gõ — G8/S2).
    pub fn to_markdown_report(&self) -> String {
        let mut md = String::new();
        md.push_str("# Báo cáo chẩn đoán TextVN (Diagnostic Report)\n\n");
        md.push_str(&format!(
            "- **Phiên bản**: `TextVN {}` (ABI v{}, `{}`, `{}`)\n",
            env!("CARGO_PKG_VERSION"),
            self.abi_version,
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
        md.push_str(&format!(
            "- **ABI structs**: `key={}`, `result={}`, `context={}` ({})\n",
            self.key_size,
            self.result_size,
            self.context_size,
            if self.sizes_ok { "OK" } else { "LỆCH" }
        ));
        md.push_str(&format!(
            "- **Cấu hình**: {}\n",
            redact_sensitive_paths(&self.config_status)
        ));

        #[cfg(target_os = "windows")]
        {
            let win = collect_windows_env_summary();
            md.push_str(&format!(
                "- **Hệ điều hành**: {}\n",
                redact_sensitive_paths(&win.os_version)
            ));
            md.push_str(&format!(
                "- **Trạng thái Windows**: TIP={}, Tray={}, Pipe={}, Hook={}, Ctrl+Shift taken={:?}, Sandbox={}\n",
                self.tip_registered,
                self.tray_running,
                self.pipe_listening,
                self.hook_running,
                self.ctrl_shift_taken,
                self.sandbox_mirror.unwrap_or("-")
            ));
            if let Some(detail) = crate::register::tip_registration_summary() {
                md.push_str(&format!(
                    "- **TIP detail**: `{}`\n",
                    redact_sensitive_paths(&detail)
                ));
            }
            if !win.conflicts.is_empty() {
                md.push_str(&format!(
                    "- **Xung đột bộ gõ phát hiện**: {}\n",
                    win.conflicts.join(", ")
                ));
            }
        }
        #[cfg(target_os = "linux")]
        {
            let env = collect_linux_env_summary();
            md.push_str(&format!(
                "- **Hệ điều hành**: {} (kernel `{}`, glibc `{}`)\n",
                redact_sensitive_paths(&env.os_pretty),
                redact_sensitive_paths(&env.kernel),
                redact_sensitive_paths(&env.glibc)
            ));
            md.push_str(&format!(
                "- **Phiên & Bảo mật**: session=`{}`, desktop=`{}`, LSM=`{}`, pkg_mgr=`{}`\n",
                redact_sensitive_paths(&env.session),
                redact_sensitive_paths(&env.desktop),
                redact_sensitive_paths(&env.lsm),
                env.pkg_mgr
            ));
            md.push_str(&format!(
                "- **Daemon & Adapter**: ibus(installed={}, running={}, adapter={}, abi_ok={}) · fcitx5(installed={}, running={}, adapter={}, abi_ok={}) · settings=`{}`\n",
                env.ibus_installed,
                env.ibus_running,
                env.ibus_found,
                env.ibus_abi_ok,
                env.fcitx5_installed,
                env.fcitx5_running,
                env.fcitx5_found,
                env.fcitx5_abi_ok,
                redact_sensitive_paths(&env.settings_backend)
            ));
            md.push_str(&format!(
                "- **Biến môi trường IM**: `{}`\n",
                redact_sensitive_paths(&env.im_env)
            ));
            md.push_str(&format!(
                "- **Tích hợp**: environment.d={}, xprofile={}, el8_cache={}, el8_autostart={}\n",
                env.env_d_present, env.xprofile_present, env.el8_system_cache, env.el8_autostart
            ));
            if !env.conflicts.is_empty() {
                md.push_str(&format!(
                    "- **Xung đột bộ gõ phát hiện**: {}\n",
                    env.conflicts.join(", ")
                ));
            }
        }
        #[cfg(target_os = "macos")]
        {
            let mac = collect_macos_env_summary();
            md.push_str(&format!(
                "- **Hệ điều hành**: {}\n",
                redact_sensitive_paths(&mac.os_version)
            ));
            if !mac.conflicts.is_empty() {
                md.push_str(&format!(
                    "- **Xung đột bộ gõ phát hiện**: {}\n",
                    mac.conflicts.join(", ")
                ));
            }
        }

        md.push_str("\n## Các vấn đề / cảnh báo phát hiện\n\n");
        if self.diagnostics.is_empty() {
            md.push_str("- Không phát hiện lỗi hoặc xung đột môi trường tự động.\n");
        } else {
            for d in &self.diagnostics {
                md.push_str(&format!(
                    "- **[{} / {}]** `{}`: {}\n  - *Gợi ý xử lý*: {}\n",
                    d.severity.to_uppercase(),
                    d.category,
                    d.code,
                    redact_sensitive_paths(&d.message),
                    redact_sensitive_paths(&d.suggestion)
                ));
            }
        }

        md.push_str("\n## Thông tin người dùng bổ sung khi báo lỗi\n\n");
        md.push_str("- **Ứng dụng đang gõ**: *(vd. Chrome, VS Code, Word, Terminal...)*\n");
        md.push_str("- **Chuỗi phím đã gõ**: *(vd. `dduocj`, `Sirr`...)*\n");
        md.push_str("- **Kết quả thực tế**: \n");
        md.push_str("- **Kết quả mong đợi**: \n");
        md.push_str("\n> *Quyền riêng tư (G8/S2): Gói chẩn đoán này đã tự động ẩn tên người dùng và KHÔNG chứa nội dung văn bản đã gõ hay từ vựng cá nhân.*\n");
        md
    }
}

/// Xuất ZIP lưu file chẩn đoán theo chuẩn PKZIP Stored (Zero Dependency).
pub fn export_diagnostics_zip(report: &DoctorReport, out_path: &Path) -> std::io::Result<usize> {
    let mut zip_entries: Vec<(String, Vec<u8>)> = Vec::new();

    // 0. report_summary.md — bản tóm tắt sẵn sàng dán lên GitHub Issues (E69)
    zip_entries.push((
        "report_summary.md".to_string(),
        report.to_markdown_report().into_bytes(),
    ));

    // 1. version.json
    let mut version_info = serde_json::to_string_pretty(&serde_json::json!({
        "app_version": env!("CARGO_PKG_VERSION"),
        "abi_version": report.abi_version,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        "git_sha": BUILD_GIT_SHA,
        "target": BUILD_TARGET,
        "distribution": current_distribution(),
    }))
    .unwrap_or_else(|_| "{}".into());
    version_info.push('\n');
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

    // 3. system_info.json (chứa cả diagnostics[] có mã lỗi/gợi ý cụ thể — E69)
    let diag_vals: Vec<serde_json::Value> = report
        .diagnostics
        .iter()
        .map(DiagnosticItem::to_json_value)
        .collect();
    let redacted_problems: Vec<String> = report
        .problems
        .iter()
        .map(|p| redact_sensitive_paths(p))
        .collect();
    let sys_obj = serde_json::json!({
        "sizes_ok": report.sizes_ok,
        "pipe_listening": report.pipe_listening,
        "tray_running": report.tray_running,
        "hook_running": report.hook_running,
        "tip_registered": report.tip_registered,
        "sandbox_mirror": report.sandbox_mirror,
        "problems": redacted_problems,
        "diagnostics": diag_vals,
    });
    let mut sys_info = serde_json::to_string_pretty(&sys_obj).unwrap_or_else(|_| "{}".into());
    sys_info.push('\n');
    zip_entries.push(("system_info.json".to_string(), sys_info.into_bytes()));

    #[cfg(target_os = "linux")]
    {
        let env = collect_linux_env_summary();
        let linux_obj = serde_json::json!({
            "os": redact_sensitive_paths(&env.os_pretty),
            "kernel": redact_sensitive_paths(&env.kernel),
            "glibc": redact_sensitive_paths(&env.glibc),
            "pkg_mgr": env.pkg_mgr,
            "session": redact_sensitive_paths(&env.session),
            "desktop": redact_sensitive_paths(&env.desktop),
            "lsm": redact_sensitive_paths(&env.lsm),
            "im_env": redact_sensitive_paths(&env.im_env),
            "ibus_installed": env.ibus_installed,
            "ibus_found": env.ibus_found,
            "ibus_abi_ok": env.ibus_abi_ok,
            "ibus_abi_err": redact_sensitive_paths(&env.ibus_abi_err),
            "ibus_running": env.ibus_running,
            "fcitx5_installed": env.fcitx5_installed,
            "fcitx5_found": env.fcitx5_found,
            "fcitx5_abi_ok": env.fcitx5_abi_ok,
            "fcitx5_abi_err": redact_sensitive_paths(&env.fcitx5_abi_err),
            "fcitx5_running": env.fcitx5_running,
            "settings_backend": redact_sensitive_paths(&env.settings_backend),
            "env_d_present": env.env_d_present,
            "xprofile_present": env.xprofile_present,
            "el8_system_cache": env.el8_system_cache,
            "el8_autostart": env.el8_autostart,
            "conflicts": env.conflicts,
        });
        let mut linux_json =
            serde_json::to_string_pretty(&linux_obj).unwrap_or_else(|_| "{}".into());
        linux_json.push('\n');
        zip_entries.push(("linux_env.json".to_string(), linux_json.into_bytes()));

        let install_diag = collect_linux_install_diag_tail(200);
        zip_entries.push((
            "install_diag_tail.log".to_string(),
            install_diag.into_bytes(),
        ));
    }

    #[cfg(target_os = "windows")]
    {
        let win = collect_windows_env_summary();
        let tip_detail = crate::register::tip_registration_summary()
            .map(|s| redact_sensitive_paths(&s))
            .unwrap_or_default();
        let win_obj = serde_json::json!({
            "os": redact_sensitive_paths(&win.os_version),
            "ctrl_shift_taken": report.ctrl_shift_taken,
            "tip_detail": tip_detail,
            "conflicts": win.conflicts,
        });
        let mut win_json = serde_json::to_string_pretty(&win_obj).unwrap_or_else(|_| "{}".into());
        win_json.push('\n');
        zip_entries.push(("windows_env.json".to_string(), win_json.into_bytes()));

        let reg_tail = collect_windows_register_log_tail(200);
        zip_entries.push(("register_tail.log".to_string(), reg_tail.into_bytes()));
    }

    #[cfg(target_os = "macos")]
    {
        let mac = collect_macos_env_summary();
        let mac_obj = serde_json::json!({
            "os": redact_sensitive_paths(&mac.os_version),
            "conflicts": mac.conflicts,
        });
        let mut mac_json = serde_json::to_string_pretty(&mac_obj).unwrap_or_else(|_| "{}".into());
        mac_json.push('\n');
        zip_entries.push(("macos_env.json".to_string(), mac_json.into_bytes()));
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

    // 4b. panic_tail.log (E70) — chỉ vị trí + ký hiệu backtrace, không payload động (G8).
    let panic_tail = textvn_config::crash::default_log_dir()
        .and_then(|d| textvn_config::crash::read_panic_log_tail(&d, 32 * 1024))
        .map(|t| redact_sensitive_paths(&t))
        .unwrap_or_else(|| "# Không có panic.log (chưa ghi nhận panic nào).\n".into());
    zip_entries.push(("panic_tail.log".to_string(), panic_tail.into_bytes()));

    // 5. tsf_tail.log (200 dòng cuối từ log runtime — đã ẩn tên người dùng S2)
    let log_tail = redact_sensitive_paths(&collect_log_tail(200));
    zip_entries.push(("tsf_tail.log".to_string(), log_tail.into_bytes()));

    // Ghi file ZIP
    let zip_bytes = build_pkzip(&zip_entries)?;
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
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

/// Thay chuỗi không phân biệt hoa/thường ASCII (dùng cho đường dẫn Windows/macOS).
fn replace_ascii_case_insensitive(haystack: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() || haystack.len() < needle.len() {
        return haystack.to_string();
    }
    let h_bytes = haystack.as_bytes();
    let n_bytes = needle.as_bytes();
    let mut out = String::with_capacity(haystack.len());
    let mut i = 0;
    while i < h_bytes.len() {
        if i + n_bytes.len() <= h_bytes.len()
            && h_bytes[i..i + n_bytes.len()].eq_ignore_ascii_case(n_bytes)
            && haystack.is_char_boundary(i)
            && haystack.is_char_boundary(i + n_bytes.len())
        {
            out.push_str(replacement);
            i += n_bytes.len();
        } else {
            let ch = haystack[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Redact đường dẫn người dùng (`C:\Users\<username>`, `/home/<user>`, `USERPROFILE`/`HOME`) theo Rule S2.
pub fn redact_sensitive_paths(input: &str) -> String {
    let mut out = input.to_string();
    let redact_token = |text: &str, token: &str| -> String {
        if cfg!(any(target_os = "windows", target_os = "macos")) {
            replace_ascii_case_insensitive(text, token, "<REDACTED_USER>")
        } else {
            text.replace(token, "<REDACTED_USER>")
        }
    };
    // 1. Ẩn tên thư mục người dùng từ USERPROFILE / HOME (kể cả khi tên thư mục khác USERNAME
    //    hoặc USER/LOGNAME chưa đặt) nhưng vẫn giữ cấu trúc `C:\Users\<REDACTED_USER>` / `/home/<REDACTED_USER>`.
    for dir_key in ["USERPROFILE", "HOME"] {
        if let Some(dir_val) = std::env::var_os(dir_key) {
            if let Some(dir_str) = dir_val.to_str() {
                if let Some(leaf) = Path::new(dir_str).file_name().and_then(|s| s.to_str()) {
                    if leaf.len() >= 2 && leaf != "root" {
                        out = redact_token(&out, leaf);
                    }
                }
            }
        }
    }
    // 2. Ẩn USERNAME / USER / LOGNAME (không phân biệt hoa/thường trên Windows & macOS).
    for env_key in ["USERNAME", "USER", "LOGNAME"] {
        if let Some(username) = std::env::var_os(env_key) {
            if let Some(name_str) = username.to_str() {
                if name_str.len() >= 2 {
                    out = redact_token(&out, name_str);
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
        let log_dir = PathBuf::from(home).join("Library/Logs/TextVN");
        let primary = log_dir.join("textvn.log");
        if primary.exists() {
            log_path = Some(primary);
        } else if let Ok(entries) = std::fs::read_dir(&log_dir) {
            // Diagnostics.swift của macOS IMK ghi ra im-<pid>.log — chọn tệp mới nhất.
            let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name();
                let Some(s) = name.to_str() else { continue };
                if s.starts_with("im-") && s.ends_with(".log") && p.is_file() {
                    let mtime = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(UNIX_EPOCH);
                    if newest.as_ref().is_none_or(|(t, _)| mtime > *t) {
                        newest = Some((mtime, p));
                    }
                }
            }
            log_path = newest.map(|(_, p)| p);
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

/// Kiểm tra lệch biến môi trường IM Linux so với daemon đang chạy (thuần — dùng chung cho unit test).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn detect_linux_im_env_mismatch(
    ibus_running: bool,
    fcitx5_running: bool,
    gtk_im: &str,
    qt_im: &str,
    xmod: &str,
) -> Option<String> {
    if ibus_running
        && !fcitx5_running
        && (gtk_im.starts_with("fcitx") || qt_im.starts_with("fcitx") || xmod.contains("@im=fcitx"))
    {
        return Some(format!(
            "ibus-daemon đang chạy nhưng biến môi trường trỏ tới Fcitx (GTK={gtk_im}, QT={qt_im}, XMOD={xmod})"
        ));
    }
    if fcitx5_running
        && !ibus_running
        && (gtk_im == "ibus" || qt_im == "ibus" || xmod.contains("@im=ibus"))
    {
        return Some(format!(
            "fcitx5 đang chạy nhưng biến môi trường trỏ tới IBus (GTK={gtk_im}, QT={qt_im}, XMOD={xmod})"
        ));
    }
    None
}

/// `fcitx5 --version` (vd "fcitx5 5.0.14", "5.1.7") → (major, minor, patch).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn parse_fcitx5_version(out: &str) -> Option<(u32, u32, u32)> {
    out.split_whitespace().find_map(|tok| {
        let mut it = tok.split('.').map(|p| p.parse::<u32>().ok());
        let (a, b) = (it.next()??, it.next()??);
        let c = it.next().flatten().unwrap_or(0);
        Some((a, b, c))
    })
}

/// Một phím Fcitx5 (`Control+Shift_L`, `Shift+Control_R`) chỉ gồm Ctrl + Shift — đúng công tắc V/E
/// của TextVN.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn fcitx5_key_is_ctrl_shift(key: &str) -> bool {
    let toks: Vec<&str> = key.split('+').map(str::trim).collect();
    let ctrl = |t: &&str| matches!(*t, "Control" | "Ctrl" | "Control_L" | "Control_R");
    let shift = |t: &&str| matches!(*t, "Shift" | "Shift_L" | "Shift_R");
    !toks.is_empty()
        && toks.iter().all(|t| ctrl(t) || shift(t))
        && toks.iter().any(ctrl)
        && toks.iter().any(shift)
}

/// R2-175 — Fcitx5 có giữ Ctrl+Shift (đổi input method / bật-tắt) TRƯỚC khi phím tới TextVN không?
/// Fcitx5 5.0.x (Ubuntu 22.04, Mint 21, Debian 12) mặc định gắn Ctrl+Shift vào
/// `EnumerateForwardKeys`/`EnumerateBackwardKeys` ⇒ Ctrl+Shift đổi `textvn` ⇄ `keyboard-us`, công tắc V/E
/// của TextVN không chạy, `state.json` không đổi (bằng chứng CI ARM-L1). Fcitx5 ≥ 5.1 mặc định trống.
/// `config_text` = nội dung `~/.config/fcitx5/config` (None = chưa có tệp ⇒ mặc định của phiên bản).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn fcitx5_ctrl_shift_taken(version_output: Option<&str>, config_text: Option<&str>) -> bool {
    const ENUM: [&str; 2] = ["EnumerateForwardKeys", "EnumerateBackwardKeys"];
    const OTHER: [&str; 4] = [
        "TriggerKeys",
        "AltTriggerKeys",
        "EnumerateGroupForwardKeys",
        "EnumerateGroupBackwardKeys",
    ];
    // Đọc các mục [Hotkey/<Tên>] → danh sách phím (mục có mặt mà rỗng = người dùng đã xoá phím).
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    for line in config_text.unwrap_or("").lines() {
        let l = line.trim();
        if let Some(name) = l.strip_prefix("[Hotkey/").and_then(|r| r.strip_suffix(']')) {
            sections.push((name.to_string(), Vec::new()));
        } else if l.starts_with('[') {
            sections.push((String::new(), Vec::new()));
        } else if let (Some((_, keys)), Some((_, v))) = (sections.last_mut(), l.split_once('=')) {
            keys.push(v.trim().to_string());
        }
    }
    let explicit = |name: &str| sections.iter().find(|(n, _)| n == name).map(|(_, k)| k);
    let old_defaults =
        parse_fcitx5_version(version_output.unwrap_or("")).is_some_and(|(a, b, _)| (a, b) < (5, 1));
    ENUM.iter().any(|n| match explicit(n) {
        Some(keys) => keys.iter().any(|k| fcitx5_key_is_ctrl_shift(k)),
        None => old_defaults,
    }) || OTHER
        .iter()
        .any(|n| explicit(n).is_some_and(|keys| keys.iter().any(|k| fcitx5_key_is_ctrl_shift(k))))
}

/// R2-175 — đọc trạng thái Fcitx5 của máy (Linux): None khi không có Fcitx5.
#[cfg(target_os = "linux")]
fn linux_fcitx5_ctrl_shift_taken() -> Option<bool> {
    if !command_on_path("fcitx5") {
        return None;
    }
    let version = std::process::Command::new("fcitx5")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    let cfg_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    let config = std::fs::read_to_string(cfg_dir.join("fcitx5/config")).ok();
    Some(fcitx5_ctrl_shift_taken(
        version.as_deref(),
        config.as_deref(),
    ))
}

#[cfg(not(target_os = "linux"))]
fn linux_fcitx5_ctrl_shift_taken() -> Option<bool> {
    None
}

/// Phân tích nội dung `~/.config/fcitx5/profile` tìm các bộ gõ tiếng Việt cũ xung đột TextVN.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn detect_fcitx5_profile_conflicts(profile_text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in profile_text.lines() {
        if let Some(name) = line.trim().strip_prefix("Name=") {
            let n = name.trim();
            if n == "unikey" || n == "bamboo" || n.starts_with("m17n_vi_") {
                let tag = format!("fcitx5:{n}");
                if !found.contains(&tag) {
                    found.push(tag);
                }
            }
        }
    }
    found
}

/// Nhận diện tên bộ gõ tiếng Việt Windows có khả năng xung đột từ chuỗi lệnh/tên giá trị Run registry.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn detect_windows_conflict_name(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    if lower.contains("unikey") {
        Some("UniKey")
    } else if lower.contains("evkey") {
        Some("EVKey")
    } else if lower.contains("openkey") {
        Some("OpenKey")
    } else if lower.contains("gotiengviet") {
        Some("GoTiengViet")
    } else if lower.contains("vietkey") {
        Some("VietKey")
    } else {
        None
    }
}

/// CLSID TIP TextVN (trùng `register::CLSID_STR`, viết hoa).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const TEXTVN_TIP_CLSID: &str = "{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}";
/// CLSID bộ gõ tiếng Việt có sẵn của Microsoft (Windows 10 2004+/11: "Vietnamese Telex" /
/// "Vietnamese Number Key-Based") — kiểm chứng trên máy dev 2026-11-25 qua
/// `HKLM\SOFTWARE\Microsoft\CTF\TIP\{C2CB2CF0-…}\LanguageProfile\0x0000042a`.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub const MS_VI_IME_CLSID: &str = "{C2CB2CF0-AF47-413E-9780-8BC3A3C16068}";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const MS_VI_TELEX_PROFILE: &str = "{5FB02EC5-0A77-4684-B4FA-DEF8A2195628}";
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const MS_VI_VNI_PROFILE: &str = "{591AE943-56BE-48F6-8966-06B43915CC5A}";

/// E70: từ danh sách tên giá trị trong `HKCU\Control Panel\International\User Profile\<lang>`
/// (dạng `042A:{CLSID}{PROFILE}` = TIP; `042A:0000042A` = bố cục bàn phím thường), trả các
/// bộ gõ TSF tiếng Việt (LANGID 0x042A) KHÁC TextVN đang được bật trong danh sách ngôn ngữ.
/// Bố cục bàn phím thường (không có `{`) không phải bộ gõ → bỏ qua.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn detect_other_vi_tips(value_names: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in value_names {
        let upper = name.trim().to_ascii_uppercase();
        let Some(rest) = upper.strip_prefix("042A:") else {
            continue;
        };
        // `get` (không cắt `[..38]`): tên giá trị có ký tự non-ASCII thì byte 38 có thể
        // nằm giữa một ký tự UTF-8 → cắt trực tiếp sẽ panic (audit E70 vòng 1).
        if !rest.starts_with('{') {
            continue;
        }
        let Some(clsid) = rest.get(..38) else {
            continue;
        };
        if clsid == TEXTVN_TIP_CLSID {
            continue;
        }
        let label = if clsid == MS_VI_IME_CLSID {
            let profile = rest.get(38..76).unwrap_or("");
            if profile == MS_VI_TELEX_PROFILE {
                "Microsoft Vietnamese Telex".to_string()
            } else if profile == MS_VI_VNI_PROFILE {
                "Microsoft Vietnamese Number Key-Based (VNI)".to_string()
            } else {
                "Microsoft Vietnamese IME".to_string()
            }
        } else {
            format!("TSF tiếng Việt khác {clsid}")
        };
        if !out.contains(&label) {
            out.push(label);
        }
    }
    out
}

/// E70: từ output `defaults read com.apple.HIToolbox AppleEnabledInputSources` (plist ASCII),
/// trả các bộ gõ tiếng Việt có sẵn của Apple đang bật (Telex / VNI / Simple Telex / VIQR).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn detect_apple_vi_input_sources(defaults_output: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in
        defaults_output.split(|c: char| c == '"' || c.is_whitespace() || c == ';' || c == '=')
    {
        let lower = raw.to_ascii_lowercase();
        if !lower.starts_with("com.apple.inputmethod.vietnamese") {
            continue;
        }
        let label = if lower.ends_with("simpletelex") {
            "Apple Vietnamese Simple Telex"
        } else if lower.ends_with("telex") {
            "Apple Vietnamese Telex"
        } else if lower.ends_with("vni") {
            "Apple Vietnamese VNI"
        } else if lower.ends_with("viqr") {
            "Apple Vietnamese VIQR"
        } else {
            continue; // bundle id gốc `…VietnameseIM` không phải một chế độ gõ cụ thể
        };
        if !out.iter().any(|x| x == label) {
            out.push(label.to_string());
        }
    }
    out
}

/// E70: loại bản phân phối của binary đang chạy (để RCA khác biệt installer/portable/store).
/// Thuần (nhận `exe_dir` + hàm kiểm tồn tại) để unit test được trên mọi OS.
pub fn classify_distribution(
    os: &str,
    exe_dir: &str,
    file_exists: &dyn Fn(&str) -> bool,
) -> &'static str {
    let norm = exe_dir.replace('\\', "/");
    let lower = norm.to_ascii_lowercase();
    match os {
        "windows" => {
            if lower.contains("/textvn-store/") || lower.contains("/windowsapps/") {
                "store"
            } else if file_exists(&format!("{norm}/unins000.exe")) {
                if lower.contains("/program files") {
                    "installer-machine"
                } else {
                    "installer-user"
                }
            } else {
                "portable"
            }
        }
        "macos" => {
            if lower.contains(".app/contents/") {
                "app-bundle"
            } else {
                "cli"
            }
        }
        _ => {
            if lower.starts_with("/usr/") || lower.starts_with("/opt/") {
                "system-package"
            } else if lower.contains("/.local/") {
                "user-install"
            } else {
                "portable"
            }
        }
    }
}

/// Mã commit lúc build (`build.rs` → `TEXTVN_GIT_SHA`).
pub const BUILD_GIT_SHA: &str = env!("TEXTVN_GIT_SHA");
/// Target triple lúc build (`build.rs` → `TEXTVN_BUILD_TARGET`).
pub const BUILD_TARGET: &str = env!("TEXTVN_BUILD_TARGET");

/// Loại bản phân phối của tiến trình hiện tại.
pub fn current_distribution() -> &'static str {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().into_owned()))
        .unwrap_or_default();
    classify_distribution(std::env::consts::OS, &dir, &|p| Path::new(p).is_file())
}

/// Số bản ghi panic trong `panic.log` (E70).
/// R2-166: `gnome-shell --version` ("GNOME Shell 50.1") có thuộc dải mutter 50.0–50.3 bỏ phím
/// bộ gõ gửi lại (mutter#4853, sửa ở 50.4/51.0) không. Thuần, test được trên mọi OS.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn mutter_drops_forwarded_keys(gnome_shell_version: &str) -> bool {
    let Some(ver) = gnome_shell_version
        .split_whitespace()
        .find(|w| w.starts_with(|c: char| c.is_ascii_digit()))
    else {
        return false;
    };
    let mut parts = ver.split('.');
    let major = parts.next().and_then(|p| p.parse::<u32>().ok());
    let minor = parts
        .next()
        .map(|p| p.trim_start_matches(|c: char| !c.is_ascii_digit()))
        .and_then(|p| {
            p.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        });
    matches!((major, minor), (Some(50), Some(0..=3)))
}

pub fn count_panic_records(text: &str) -> usize {
    text.matches("=== TextVN panic").count()
}

/// Chuẩn hoá hiển thị phiên bản Windows (Build >= 22000 → Windows 11).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn format_windows_version(
    product_name: &str,
    display_ver: &str,
    build: &str,
    ubr: Option<u32>,
    edition: &str,
) -> String {
    let build_num: u32 = build.trim().parse().unwrap_or(0);
    let normalized_prod = if build_num >= 22000 && product_name.starts_with("Windows 10") {
        product_name.replacen("Windows 10", "Windows 11", 1)
    } else if product_name.is_empty() {
        "Windows".to_string()
    } else {
        product_name.to_string()
    };
    let mut out = normalized_prod;
    if !display_ver.is_empty() {
        out.push(' ');
        out.push_str(display_ver);
    }
    if !edition.is_empty()
        && !out
            .to_ascii_lowercase()
            .contains(&edition.to_ascii_lowercase())
    {
        out.push_str(&format!(" [{edition}]"));
    }
    if !build.is_empty() {
        match ubr {
            Some(u) => out.push_str(&format!(" (Build {build}.{u})")),
            None => out.push_str(&format!(" (Build {build})")),
        }
    }
    out
}

/// Đọc giá trị chuỗi `<key>NAME</key><string>VALUE</string>` trong plist (macOS SystemVersion.plist).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn parse_plist_string_value(plist_xml: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let after = plist_xml.split_once(&needle)?.1;
    let after_str = after.split_once("<string>")?.1;
    let val = after_str.split_once("</string>")?.0.trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

#[cfg(target_os = "windows")]
struct WindowsEnvSummary {
    os_version: String,
    conflicts: Vec<String>,
}

#[cfg(target_os = "windows")]
fn read_hklm_string(subkey: &str, value_name: &str) -> Option<String> {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
        REG_VALUE_TYPE,
    };
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(subkey),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }
        let mut typ = REG_VALUE_TYPE(0);
        let mut len: u32 = 0;
        let q1 = RegQueryValueExW(
            hkey,
            &HSTRING::from(value_name),
            None,
            Some(&mut typ),
            None,
            Some(&mut len),
        );
        if q1.is_err() || typ != REG_SZ || !(2..=1024).contains(&len) {
            let _ = RegCloseKey(hkey);
            return None;
        }
        let mut buf = vec![0u16; (len as usize).div_ceil(2)];
        let mut byte_len = (buf.len() * 2) as u32;
        let q2 = RegQueryValueExW(
            hkey,
            &HSTRING::from(value_name),
            None,
            Some(&mut typ),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut byte_len),
        );
        let _ = RegCloseKey(hkey);
        if q2.is_err() {
            return None;
        }
        let used = ((byte_len as usize) / 2).min(buf.len());
        let end = buf[..used].iter().position(|&c| c == 0).unwrap_or(used);
        Some(String::from_utf16_lossy(&buf[..end]).trim().to_string())
    }
}

#[cfg(target_os = "windows")]
fn read_hklm_dword(subkey: &str, value_name: &str) -> Option<u32> {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
        REG_DWORD, REG_VALUE_TYPE,
    };
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            &HSTRING::from(subkey),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }
        let mut typ = REG_VALUE_TYPE(0);
        let mut val: u32 = 0;
        let mut len: u32 = std::mem::size_of::<u32>() as u32;
        let ok = RegQueryValueExW(
            hkey,
            &HSTRING::from(value_name),
            None,
            Some(&mut typ),
            Some(&mut val as *mut u32 as *mut u8),
            Some(&mut len),
        )
        .is_ok();
        let _ = RegCloseKey(hkey);
        if ok && typ == REG_DWORD {
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_run_key_for_conflicts(
    root: windows::Win32::System::Registry::HKEY,
    subkey: &str,
    out: &mut Vec<String>,
) {
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegEnumValueW, RegOpenKeyExW, HKEY, KEY_READ, REG_EXPAND_SZ, REG_SZ,
    };
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(root, &HSTRING::from(subkey), Some(0), KEY_READ, &mut hkey).is_err() {
            return;
        }
        let mut idx: u32 = 0;
        loop {
            let mut name_buf = [0u16; 256];
            let mut name_len = name_buf.len() as u32;
            let mut data_buf = [0u8; 1024];
            let mut data_len = data_buf.len() as u32;
            let mut typ: u32 = 0;
            let st = RegEnumValueW(
                hkey,
                idx,
                Some(windows::core::PWSTR(name_buf.as_mut_ptr())),
                &mut name_len,
                None,
                Some(&mut typ),
                Some(data_buf.as_mut_ptr()),
                Some(&mut data_len),
            );
            if st == ERROR_NO_MORE_ITEMS {
                break;
            }
            idx += 1;
            if st == ERROR_MORE_DATA {
                continue;
            }
            if st.is_err() {
                break;
            }
            let nlen = (name_len as usize).min(name_buf.len());
            let name = String::from_utf16_lossy(&name_buf[..nlen]);
            if let Some(hit) = detect_windows_conflict_name(&name) {
                let tag = format!("{hit} (startup)");
                if !out.contains(&tag) {
                    out.push(tag);
                }
                continue;
            }
            if typ == REG_SZ.0 || typ == REG_EXPAND_SZ.0 {
                let dlen = (data_len as usize).min(data_buf.len());
                let u16s: Vec<u16> = (0..dlen / 2)
                    .map(|k| u16::from_le_bytes([data_buf[k * 2], data_buf[k * 2 + 1]]))
                    .take_while(|&v| v != 0)
                    .collect();
                let val = String::from_utf16_lossy(&u16s);
                if let Some(hit) = detect_windows_conflict_name(&val) {
                    let tag = format!("{hit} (startup)");
                    if !out.contains(&tag) {
                        out.push(tag);
                    }
                }
            }
        }
        let _ = RegCloseKey(hkey);
    }
}

#[cfg(target_os = "windows")]
fn collect_windows_env_summary() -> WindowsEnvSummary {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    const CV: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let prod = read_hklm_string(CV, "ProductName").unwrap_or_default();
    let disp = read_hklm_string(CV, "DisplayVersion")
        .or_else(|| read_hklm_string(CV, "ReleaseId"))
        .unwrap_or_default();
    let build = read_hklm_string(CV, "CurrentBuildNumber")
        .or_else(|| read_hklm_string(CV, "CurrentBuild"))
        .unwrap_or_default();
    let ubr = read_hklm_dword(CV, "UBR");
    let edition = read_hklm_string(CV, "EditionID").unwrap_or_default();
    let os_version = format_windows_version(&prod, &disp, &build, ubr, &edition);

    let mut conflicts = Vec::new();
    // 1. Kiểm tra mutex chuẩn của các bộ gõ tiếng Việt đang chạy (không duyệt process list — an toàn AV).
    for (label, mutexes) in [
        (
            "UniKey (đang chạy)",
            &[r"Local\UniKeyMutex", "UniKeyMutex"][..],
        ),
        (
            "EVKey (đang chạy)",
            &[r"Local\EVKeyMutex", "EVKeyMutex"][..],
        ),
        (
            "OpenKey (đang chạy)",
            &[r"Local\OpenKeyMutex", "OpenKeyMutex"][..],
        ),
        (
            "GoTiengViet (đang chạy)",
            &[r"Local\GoTiengVietMutex", "GoTiengVietMutex"][..],
        ),
    ] {
        if mutexes.iter().any(|m| check_instance_mutex(m)) && !conflicts.iter().any(|c| c == label)
        {
            conflicts.push(label.to_string());
        }
    }
    // 2. Kiểm tra khoá tự khởi động Run (HKCU/HKLM).
    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    scan_run_key_for_conflicts(HKEY_CURRENT_USER, RUN_KEY, &mut conflicts);
    scan_run_key_for_conflicts(HKEY_LOCAL_MACHINE, RUN_KEY, &mut conflicts);

    WindowsEnvSummary {
        os_version,
        conflicts,
    }
}

/// E70: tên giá trị của mọi ngôn ngữ trong `HKCU\Control Panel\International\User Profile`.
#[cfg(target_os = "windows")]
fn enum_user_profile_value_names() -> Vec<String> {
    use windows::core::{HSTRING, PWSTR};
    use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegEnumKeyExW, RegEnumValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    };
    const ROOT: &str = r"Control Panel\International\User Profile";
    let mut names = Vec::new();
    unsafe {
        let mut root = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &HSTRING::from(ROOT),
            Some(0),
            KEY_READ,
            &mut root,
        )
        .is_err()
        {
            return names;
        }
        let mut ki: u32 = 0;
        while ki < 64 {
            let mut kbuf = [0u16; 128];
            let mut klen = kbuf.len() as u32;
            let st = RegEnumKeyExW(
                root,
                ki,
                Some(PWSTR(kbuf.as_mut_ptr())),
                &mut klen,
                None,
                None,
                None,
                None,
            );
            if st == ERROR_NO_MORE_ITEMS || st.is_err() {
                break;
            }
            ki += 1;
            let lang = String::from_utf16_lossy(&kbuf[..(klen as usize).min(kbuf.len())]);
            let mut sub = HKEY::default();
            if RegOpenKeyExW(
                root,
                &HSTRING::from(lang.as_str()),
                Some(0),
                KEY_READ,
                &mut sub,
            )
            .is_err()
            {
                continue;
            }
            let mut vi: u32 = 0;
            while vi < 256 {
                let mut nbuf = [0u16; 160];
                let mut nlen = nbuf.len() as u32;
                let st = RegEnumValueW(
                    sub,
                    vi,
                    Some(PWSTR(nbuf.as_mut_ptr())),
                    &mut nlen,
                    None,
                    None,
                    None,
                    None,
                );
                if st == ERROR_NO_MORE_ITEMS {
                    break;
                }
                vi += 1;
                if st.is_err() {
                    continue;
                }
                names.push(String::from_utf16_lossy(
                    &nbuf[..(nlen as usize).min(nbuf.len())],
                ));
            }
            let _ = RegCloseKey(sub);
        }
        let _ = RegCloseKey(root);
    }
    names
}

#[cfg(target_os = "windows")]
fn collect_windows_diagnostics(diagnostics: &mut Vec<DiagnosticItem>) {
    let vi_tips = detect_other_vi_tips(&enum_user_profile_value_names());
    if !vi_tips.is_empty() {
        diagnostics.push(DiagnosticItem {
            code: "W_CONFLICT_VI_TIP",
            severity: "warning",
            category: "conflict",
            message: format!(
                "Bộ gõ tiếng Việt khác đang được bật trong danh sách ngôn ngữ Windows: {}",
                vi_tips.join(", ")
            ),
            suggestion:
                "Win+Space / Alt+Shift có thể chuyển sang bộ gõ này thay vì TextVN (gõ sai kiểu, phím V/E không phản hồi). Gỡ nó tại Cài đặt › Thời gian & ngôn ngữ › Ngôn ngữ & vùng › Tiếng Việt › Tùy chọn ngôn ngữ › Bàn phím › Xóa."
                    .into(),
        });
    }
    let win = collect_windows_env_summary();
    if !win.conflicts.is_empty() {
        diagnostics.push(DiagnosticItem {
            code: "W_CONFLICT_VI_IME",
            severity: "warning",
            category: "conflict",
            message: format!(
                "Phát hiện bộ gõ tiếng Việt khác trên Windows: {}",
                win.conflicts.join(", ")
            ),
            suggestion:
                "Hãy thoát bộ gõ cũ ở khay hệ thống (và tắt tự khởi động cùng Windows) trước khi dùng TextVN để tránh ăn phím, lặp chữ hoặc tranh phím Ctrl+Shift."
                    .into(),
        });
    }
}

#[cfg(target_os = "windows")]
fn collect_windows_register_log_tail(max_lines: usize) -> String {
    let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") else {
        return "# Không xác định được LOCALAPPDATA.\n".into();
    };
    let path = PathBuf::from(local_appdata)
        .join("TextVN")
        .join("logs")
        .join("register.log");
    let Ok(content) = std::fs::read_to_string(path) else {
        return "# Không tìm thấy file register.log.\n".into();
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    redact_sensitive_paths(&(lines[start..].join("\n") + "\n"))
}

#[cfg(target_os = "macos")]
struct MacOsEnvSummary {
    os_version: String,
    conflicts: Vec<String>,
}

#[cfg(target_os = "macos")]
fn collect_macos_env_summary() -> MacOsEnvSummary {
    let os_version = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist")
        .ok()
        .map(|xml| {
            let name =
                parse_plist_string_value(&xml, "ProductName").unwrap_or_else(|| "macOS".into());
            let ver = parse_plist_string_value(&xml, "ProductVersion").unwrap_or_default();
            let build = parse_plist_string_value(&xml, "ProductBuildVersion").unwrap_or_default();
            if ver.is_empty() {
                name
            } else if build.is_empty() {
                format!("{name} {ver}")
            } else {
                format!("{name} {ver} ({build})")
            }
        })
        .unwrap_or_else(|| "macOS".into());

    let mut conflicts = Vec::new();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for (label, app_name) in [
        ("OpenKey", "OpenKey.app"),
        ("EVKey", "EVKey.app"),
        ("GoTiengViet", "GoTiengViet.app"),
    ] {
        let sys_app = Path::new("/Applications").join(app_name);
        let user_app = home.as_ref().map(|h| h.join("Applications").join(app_name));
        if sys_app.exists() || user_app.is_some_and(|p| p.exists()) {
            conflicts.push(format!("{label} (đã cài)"));
        }
    }
    MacOsEnvSummary {
        os_version,
        conflicts,
    }
}

#[cfg(target_os = "macos")]
fn collect_macos_diagnostics(diagnostics: &mut Vec<DiagnosticItem>) {
    let apple_vi = std::process::Command::new("/usr/bin/defaults")
        .args(["read", "com.apple.HIToolbox", "AppleEnabledInputSources"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| detect_apple_vi_input_sources(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    if !apple_vi.is_empty() {
        diagnostics.push(DiagnosticItem {
            code: "W_CONFLICT_APPLE_VI_IM",
            severity: "warning",
            category: "conflict",
            message: format!(
                "Bộ gõ tiếng Việt có sẵn của macOS đang bật song song: {}",
                apple_vi.join(", ")
            ),
            suggestion:
                "Ctrl+Space / Fn có thể chuyển sang bộ gõ Apple thay vì TextVN. Gỡ nó tại System Settings › Keyboard › Text Input › Edit… › chọn bộ gõ Vietnamese của Apple › nút −."
                    .into(),
        });
    }
    let mac = collect_macos_env_summary();
    if !mac.conflicts.is_empty() {
        diagnostics.push(DiagnosticItem {
            code: "W_CONFLICT_VI_IME",
            severity: "warning",
            category: "conflict",
            message: format!(
                "Phát hiện bộ gõ tiếng Việt khác trên macOS: {}",
                mac.conflicts.join(", ")
            ),
            suggestion:
                "Hãy thoát OpenKey/EVKey/GoTiengViet khỏi thanh menu bar (và tắt Accessibility / Input Monitoring cho bộ gõ cũ) khi dùng TextVN để tránh lặp dấu hoặc tranh phím Ctrl+Shift."
                    .into(),
        });
    }
}

#[cfg(target_os = "linux")]
struct LinuxEnvSummary {
    os_pretty: String,
    kernel: String,
    glibc: String,
    pkg_mgr: String,
    session: String,
    desktop: String,
    lsm: String,
    ibus_comp: String,
    fcitx_dirs: String,
    im_env: String,
    gtk_im: String,
    qt_im: String,
    xmod: String,
    ibus_installed: bool,
    ibus_found: bool,
    ibus_abi_ok: bool,
    ibus_abi_err: String,
    fcitx5_installed: bool,
    fcitx5_found: bool,
    fcitx5_abi_ok: bool,
    fcitx5_abi_err: String,
    settings_backend: String,
    ibus_running: bool,
    fcitx5_running: bool,
    env_d_present: bool,
    xprofile_present: bool,
    el8_system_cache: bool,
    el8_autostart: bool,
    conflicts: Vec<String>,
}

#[cfg(target_os = "linux")]
fn command_on_path(cmd: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(cmd);
        if p.is_file() {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn detect_linux_pkg_mgr() -> String {
    for (cmd, label) in [
        ("apt-get", "apt"),
        ("dnf", "dnf"),
        ("yum", "yum"),
        ("zypper", "zypper"),
        ("pacman", "pacman"),
    ] {
        if command_on_path(cmd) {
            return label.into();
        }
    }
    "none".into()
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
fn current_euid() -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").ok().map(|m| m.uid())
}

/// Kiểm tra tiến trình thuộc cùng UID hiện tại và hỗ trợ tên dài >15 ký tự
/// (vượt giới hạn `TASK_COMM_LEN - 1` của `/proc/<pid>/comm` cho `ibus-engine-unikey`/`bamboo`…).
#[cfg(target_os = "linux")]
fn linux_proc_running(target_comm: &str) -> bool {
    use std::os::unix::fs::MetadataExt;
    let my_uid = current_euid();
    let cur_dbus = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    let want15 = if target_comm.len() > 15 {
        &target_comm[..15]
    } else {
        target_comm
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        if !s.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let p = entry.path();
        if let Some(uid) = my_uid {
            let Ok(meta) = std::fs::metadata(&p) else {
                continue;
            };
            if meta.uid() != uid {
                continue;
            }
        }
        let Ok(comm_raw) = std::fs::read_to_string(p.join("comm")) else {
            continue;
        };
        let comm = comm_raw.trim();
        let matched = if target_comm.len() <= 15 {
            comm == target_comm
        } else if comm == want15 || comm.starts_with("python") || comm == "sh" || comm == "bash" {
            std::fs::read(p.join("cmdline")).ok().is_some_and(|bytes| {
                bytes
                    .split(|&b| b == 0)
                    .take(2)
                    .filter(|a| !a.is_empty())
                    .any(|arg| {
                        let s = String::from_utf8_lossy(arg);
                        Path::new(s.as_ref()).file_name().and_then(|f| f.to_str())
                            == Some(target_comm)
                    })
            })
        } else {
            false
        };
        if !matched {
            continue;
        }
        if let Some(ref want_dbus) = cur_dbus {
            if let Ok(env_bytes) = std::fs::read(p.join("environ")) {
                let proc_dbus = env_bytes.split(|&b| b == 0).find_map(|kv| {
                    kv.strip_prefix(b"DBUS_SESSION_BUS_ADDRESS=")
                        .map(|v| String::from_utf8_lossy(v).into_owned())
                });
                if let Some(pd) = proc_dbus {
                    if !pd.is_empty() && pd != *want_dbus {
                        continue;
                    }
                }
            }
        }
        return true;
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
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let env_or = |k: &str| std::env::var(k).unwrap_or_else(|_| "<chưa đặt>".into());
    let session = env_or("XDG_SESSION_TYPE");
    let desktop = env_or("XDG_CURRENT_DESKTOP");
    let ibus_comp = env_or("IBUS_COMPONENT_PATH");
    let fcitx_dirs = env_or("FCITX_ADDON_DIRS");
    let gtk_im = std::env::var("GTK_IM_MODULE").unwrap_or_default();
    let qt_im = std::env::var("QT_IM_MODULE").unwrap_or_default();
    let xmod = std::env::var("XMODIFIERS").unwrap_or_default();
    let im_env = format!(
        "GTK={} · QT={} · XMOD={}",
        if gtk_im.is_empty() {
            "<chưa đặt>"
        } else {
            &gtk_im
        },
        if qt_im.is_empty() {
            "<chưa đặt>"
        } else {
            &qt_im
        },
        if xmod.is_empty() {
            "<chưa đặt>"
        } else {
            &xmod
        }
    );

    let mut ibus_found = false;
    let mut ibus_abi_ok = false;
    let mut ibus_abi_err = String::new();
    let mut fcitx5_found = false;
    let mut fcitx5_abi_ok = false;
    let mut fcitx5_abi_err = String::new();
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
            if let Some(ibus_bin) = ibus_candidates.iter().find(|p| p.exists()) {
                ibus_found = true;
                match std::process::Command::new(ibus_bin)
                    .arg("--check-abi")
                    .output()
                {
                    Ok(out) if out.status.success() => ibus_abi_ok = true,
                    Ok(out) => {
                        ibus_abi_err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    }
                    Err(e) => ibus_abi_err = e.to_string(),
                }
            }

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
            if let Some(fc_so) = fcitx_candidates.iter().find(|p| p.exists()) {
                fcitx5_found = true;
                match crate::probe_dlopen(&fc_so.to_string_lossy()) {
                    Ok(()) => fcitx5_abi_ok = true,
                    Err(e) => fcitx5_abi_err = e,
                }
            }

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
    let xprofile_present = std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".xprofile"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .is_some_and(|s| s.contains("# BEGIN TextVN"));

    let ibus_running = linux_proc_running("ibus-daemon");
    let fcitx5_running = linux_proc_running("fcitx5");
    let ibus_installed = ibus_running || command_on_path("ibus-daemon");
    let fcitx5_installed = fcitx5_running || command_on_path("fcitx5");

    let mut conflicts = Vec::new();
    if ibus_running && fcitx5_running {
        conflicts.push("dual-daemon:ibus-daemon+fcitx5".into());
    }
    for proc in [
        "fcitx",
        "scim",
        "scim-launcher",
        "uim-xim",
        "xvnkb",
        "xvnhook",
        "ibus-engine-unikey",
        "ibus-engine-bamboo",
        "ibus-engine-bogo",
        "ibus-engine-teni",
    ] {
        if linux_proc_running(proc) {
            conflicts.push(format!("running:{proc}"));
        }
    }
    if let Some(prof_path) = xdg_cfg.as_ref().map(|c| c.join("fcitx5/profile")) {
        if let Ok(txt) = std::fs::read_to_string(prof_path) {
            for hit in detect_fcitx5_profile_conflicts(&txt) {
                if !conflicts.contains(&hit) {
                    conflicts.push(hit);
                }
            }
        }
    }

    LinuxEnvSummary {
        os_pretty,
        kernel,
        glibc: detect_linux_glibc(),
        pkg_mgr: detect_linux_pkg_mgr(),
        session,
        desktop,
        lsm: detect_linux_lsm(),
        ibus_comp,
        fcitx_dirs,
        im_env,
        gtk_im,
        qt_im,
        xmod,
        ibus_installed,
        ibus_found,
        ibus_abi_ok,
        ibus_abi_err,
        fcitx5_installed,
        fcitx5_found,
        fcitx5_abi_ok,
        fcitx5_abi_err,
        settings_backend,
        ibus_running,
        fcitx5_running,
        env_d_present,
        xprofile_present,
        el8_system_cache: Path::new("/var/cache/ibus/bus/registry").exists(),
        el8_autostart,
        conflicts,
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_diagnostics(diagnostics: &mut Vec<DiagnosticItem>, problems: &mut Vec<String>) {
    let s = collect_linux_env_summary();

    // 1. Kiểm tra thiếu cả 2 framework bộ gõ hệ thống hoặc cả 2 adapter cùng lỗi ABI.
    if !s.ibus_installed && !s.fcitx5_installed {
        diagnostics.push(DiagnosticItem {
            code: "W_DEP_NO_IM_FRAMEWORK",
            severity: "warning",
            category: "dep",
            message: "Chưa thấy ibus-daemon lẫn fcitx5 trên PATH của hệ thống.".into(),
            suggestion:
                "Chạy lại `./install.sh --install-deps` để tự động cài IBus/Fcitx5 & GTK phù hợp cho distro của bạn."
                    .into(),
        });
    }
    if (s.ibus_found || s.fcitx5_found) && !s.ibus_abi_ok && !s.fcitx5_abi_ok {
        let mut detail = String::from("Không có adapter IBus hay Fcitx5 nào nạp được");
        if !s.ibus_abi_err.is_empty() {
            detail.push_str(&format!(" (IBus: {})", s.ibus_abi_err));
        }
        if !s.fcitx5_abi_err.is_empty() {
            detail.push_str(&format!(" (Fcitx5: {})", s.fcitx5_abi_err));
        }
        problems.push(detail.clone());
        diagnostics.push(DiagnosticItem {
            code: "E_DEP_MISSING",
            severity: "error",
            category: "dep",
            message: detail,
            suggestion:
                "Chạy `./install.sh --install-deps` để cài thư viện runtime (libibus / fcitx5 / gtk3|gtk4), hoặc dựng từ nguồn bằng `./scripts/install_linux.sh`."
                    .into(),
        });
    }
    if s.settings_backend.starts_with("lỗi") {
        diagnostics.push(DiagnosticItem {
            code: "W_GTK_MISSING",
            severity: "warning",
            category: "dep",
            message: format!("Bảng cài đặt GUI không mở được: {}", s.settings_backend),
            suggestion:
                "Cài thêm GTK3/GTK4 (`gtk3` trên RHEL/SLES, `libgtk-4-1` trên Ubuntu/Debian, `gtk4` trên Fedora/Arch) hoặc chạy `./install.sh --install-deps`."
                    .into(),
        });
    }

    // 2. Xung đột hai daemon IME hoặc bộ gõ cũ.
    if s.ibus_running && s.fcitx5_running {
        diagnostics.push(DiagnosticItem {
            code: "CONFLICT_DUAL_IM_DAEMON",
            severity: "warning",
            category: "conflict",
            message: "Cả ibus-daemon và fcitx5 đang chạy đồng thời trên cùng phiên.".into(),
            suggestion:
                "Thoát một trong hai: chạy `fcitx5-remote -e` (nếu dùng IBus) hoặc `ibus exit` (nếu dùng Fcitx5)."
                    .into(),
        });
    }
    for c in &s.conflicts {
        if c == "dual-daemon:ibus-daemon+fcitx5" {
            continue;
        }
        if let Some(proc) = c.strip_prefix("running:") {
            let suggestion = if proc.starts_with("ibus-engine-") {
                "Nhấn Super+Space để chuyển sang 'Vietnamese (TextVN)' và gỡ Unikey/Bamboo khỏi Cài đặt → Bàn phím → Nguồn nhập."
            } else if proc == "fcitx" {
                "Thoát fcitx v4 cũ (`fcitx-remote -e`) và đổi mặc định sang IBus hoặc Fcitx5 bằng `im-config`."
            } else {
                "Thoát bộ gõ cũ đang chạy trước khi gõ bằng TextVN để tránh ăn phím hoặc lặp chữ."
            };
            diagnostics.push(DiagnosticItem {
                code: "W_CONFLICT_VI_IME",
                severity: "warning",
                category: "conflict",
                message: format!("Bộ gõ khác đang chạy song song với TextVN: `{proc}`."),
                suggestion: suggestion.into(),
            });
        } else if let Some(im) = c.strip_prefix("fcitx5:") {
            diagnostics.push(DiagnosticItem {
                code: "W_CONFLICT_FCITX5_VI_IME",
                severity: "warning",
                category: "conflict",
                message: format!("Profile Fcitx5 còn bộ gõ tiếng Việt cũ: `{im}`."),
                suggestion: "Mở `fcitx5-configtool` và gỡ bộ gõ cũ hoặc chỉ giữ lại TextVN để tránh xung đột phím tắt.".into(),
            });
        }
    }

    // 2b. R2-166 (ngosen#1017, mutter#4853, LP#2169784): GNOME Shell 50.0–50.3 bỏ MỌI phím bộ
    //     gõ gửi lại cho app trên Wayland (Ubuntu 26.04 LTS đi kèm 50.1) ⇒ mẫu Math không dời
    //     được con trỏ. Bản sửa có ở 50.4 / 51.0.
    if s.session.eq_ignore_ascii_case("wayland") && s.desktop.to_ascii_lowercase().contains("gnome")
    {
        if let Ok(out) = std::process::Command::new("gnome-shell")
            .arg("--version")
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            if mutter_drops_forwarded_keys(&text) {
                diagnostics.push(DiagnosticItem {
                    code: "W_MUTTER_FORWARD_KEY_BUG",
                    severity: "warning",
                    category: "env",
                    message: format!(
                        "{} (Wayland) có lỗi bỏ phím bộ gõ gửi lại cho ứng dụng — mẫu Math không dời được con trỏ.",
                        text.trim()
                    ),
                    suggestion:
                        "Cập nhật GNOME/mutter lên 50.4 trở lên (`sudo apt upgrade`; Ubuntu 26.04 chờ SRU LP#2169784) hoặc dùng phiên X11 khi cần mẫu Math."
                            .into(),
                });
            }
        }
    }

    // 3. Lệch biến môi trường GTK_IM_MODULE / QT_IM_MODULE / XMODIFIERS.
    if let Some(mismatch) = detect_linux_im_env_mismatch(
        s.ibus_running,
        s.fcitx5_running,
        &s.gtk_im,
        &s.qt_im,
        &s.xmod,
    ) {
        diagnostics.push(DiagnosticItem {
            code: "W_IM_ENV_MISMATCH",
            severity: "warning",
            category: "env",
            message: mismatch,
            suggestion:
                "Chạy `im-config` (hoặc sửa ~/.xprofile / ~/.config/environment.d) cho khớp đúng framework đang chạy rồi đăng xuất & đăng nhập lại."
                    .into(),
        });
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_install_diag_tail(max_lines: usize) -> String {
    let mut path: Option<PathBuf> = None;
    if let Some(xdg) = std::env::var_os("XDG_STATE_HOME") {
        let base = PathBuf::from(xdg);
        if base.is_absolute() {
            let p = base.join("TextVN/log/install-diag.log");
            if p.exists() {
                path = Some(p);
            }
        }
    }
    if path.is_none() {
        if let Some(home) = std::env::var_os("HOME") {
            let p = PathBuf::from(home).join(".local/state/TextVN/log/install-diag.log");
            if p.exists() {
                path = Some(p);
            }
        }
    }
    let Some(p) = path else {
        return "# Chưa có nhật ký cài đặt install-diag.log.\n".into();
    };
    let Ok(content) = std::fs::read_to_string(p) else {
        return "# Không đọc được install-diag.log.\n".into();
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    redact_sensitive_paths(&(lines[start..].join("\n") + "\n"))
}

#[cfg(target_os = "linux")]
fn print_linux_env_summary() {
    let s = collect_linux_env_summary();
    println!(
        "  OS          : {} (kernel {} · glibc {} · pkg={})",
        s.os_pretty, s.kernel, s.glibc, s.pkg_mgr
    );
    println!("  session     : {} ({}) · {}", s.session, s.desktop, s.lsm);
    println!(
        "  daemons     : ibus-daemon={} (installed={}) · fcitx5={} (installed={})",
        if s.ibus_running { "chạy" } else { "tắt" },
        if s.ibus_installed { "có" } else { "không" },
        if s.fcitx5_running { "chạy" } else { "tắt" },
        if s.fcitx5_installed { "có" } else { "không" }
    );
    println!("  IBUS_COMP   : {}", s.ibus_comp);
    println!("  FCITX_DIRS  : {}", s.fcitx_dirs);
    println!("  IM env      : {}", s.im_env);
    println!(
        "  adapters    : ibus={} (abi={}) · fcitx5={} (abi={}) · settings={}",
        if s.ibus_found { "OK" } else { "thiếu" },
        if s.ibus_abi_ok { "OK" } else { "lỗi/chưa" },
        if s.fcitx5_found { "OK" } else { "thiếu" },
        if s.fcitx5_abi_ok { "OK" } else { "lỗi/chưa" },
        s.settings_backend
    );
    println!(
        "  integration : environment.d={} · xprofile={} · ibus-sys-cache={} · el8-autostart={}",
        if s.env_d_present { "OK" } else { "không" },
        if s.xprofile_present { "OK" } else { "không" },
        if s.el8_system_cache { "có" } else { "không" },
        if s.el8_autostart { "có" } else { "không" }
    );
    if !s.conflicts.is_empty() {
        println!("  conflicts   : {}", s.conflicts.join(", "));
    }
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

    /// R2-26: tình trạng bản sao sandbox — thiếu / thiếu ACL / lệch config thật / ổn.
    #[cfg(windows)]
    #[test]
    fn r2_26_sandbox_mirror_verdict() {
        let real = r#"{"config_version":1,"method":"vni","macros":[{"trigger":"a","expand":"b"}]}"#;
        let good = textvn_config::sandbox::sandbox_config_json(real).unwrap();
        let acl = "D:P(A;OICI;FA;;;S-1-5-21-1)(A;OICI;FRFX;;;AC)(A;OICI;FRFX;;;S-1-15-2-2)";
        assert_eq!(
            sandbox_mirror_verdict(Some(real), Some(&good), true, Some(acl)),
            "ok"
        );
        // Định dạng khác nhưng cùng giá trị JSON vẫn "ok".
        let compact =
            serde_json::to_string(&serde_json::from_str::<serde_json::Value>(&good).unwrap())
                .unwrap();
        assert_eq!(
            sandbox_mirror_verdict(Some(real), Some(&compact), true, Some(acl)),
            "ok"
        );
        assert_eq!(
            sandbox_mirror_verdict(Some(real), None, true, Some(acl)),
            "missing"
        );
        assert_eq!(
            sandbox_mirror_verdict(Some(real), Some(&good), false, Some(acl)),
            "missing"
        );
        assert_eq!(
            sandbox_mirror_verdict(
                Some(real),
                Some(&good),
                true,
                Some("D:(A;;FA;;;S-1-5-21-1)")
            ),
            "no_acl"
        );
        assert_eq!(
            sandbox_mirror_verdict(Some(real), Some(&good), true, None),
            "no_acl"
        );
        let telex =
            textvn_config::sandbox::sandbox_config_json(r#"{"config_version":1,"method":"telex"}"#)
                .unwrap();
        assert_eq!(
            sandbox_mirror_verdict(Some(real), Some(&telex), true, Some(acl)),
            "stale"
        );
    }

    /// E76: 2 test xuất ZIP chẩn đoán từng treo im lặng cả binary (máy dev > 1 giờ, runner
    /// Windows của release-candidate v0.2.92 tới timeout 90 phút) khi cùng ghi ZIP vào `%TEMP%`
    /// song song — nghi bộ quét AV giữ tệp nén. Khoá này cho 2 test ghi ZIP lần lượt.
    static ZIP_EXPORT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// E76: watchdog — test treo quá `secs` giây thì in mã lỗi rồi `abort` cả binary (exit ≠ 0,
    /// CI đỏ ngay) thay vì đứng im tới timeout của job. Drop guard = test xong, watchdog thôi.
    struct HangGuard(Option<std::sync::mpsc::Sender<()>>);
    impl Drop for HangGuard {
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }
    fn hang_guard(name: &'static str, secs: u64) -> HangGuard {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) =
                rx.recv_timeout(std::time::Duration::from_secs(secs))
            {
                eprintln!(
                    "E76 watchdog: test `{name}` treo > {secs} s (ghi ZIP vào %TEMP%) — abort"
                );
                std::process::abort();
            }
        });
        HangGuard(Some(tx))
    }

    /// Thư mục tạm riêng theo tên test + pid (không trùng tệp giữa các test / lần chạy).
    fn export_test_path(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("textvn-test-{}-{test}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir.join("diag.zip")
    }

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

    /// R2-166: chỉ mutter 50.0–50.3 bỏ phím bộ gõ gửi lại (ngosen#1017, mutter#4853).
    #[test]
    fn r2_166_mutter_forward_key_bug_range() {
        for bad in ["GNOME Shell 50.0", "GNOME Shell 50.1\n", "GNOME Shell 50.3"] {
            assert!(mutter_drops_forwarded_keys(bad), "{bad}");
        }
        for good in [
            "GNOME Shell 50.4",
            "GNOME Shell 51.1",
            "GNOME Shell 49.5",
            "GNOME Shell 46.0",
            "GNOME Shell 50.rc",
            "",
            "gnome-shell: not found",
        ] {
            assert!(!mutter_drops_forwarded_keys(good), "{good}");
        }
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
                // Không phân biệt hoa/thường trên Windows (E69 audit #9).
                let upper = user_str.to_ascii_uppercase();
                let mixed = format!(r"c:\users\{upper}\appdata\roaming\textvn\config.json");
                let redacted_mixed = redact_sensitive_paths(&mixed);
                assert!(!redacted_mixed
                    .to_ascii_lowercase()
                    .contains(&user_str.to_ascii_lowercase()));
                assert!(redacted_mixed.contains("<REDACTED_USER>"));
            }
        }
    }

    #[test]
    fn export_diagnostics_zip_creates_zip_file_without_user_text() {
        let _guard = hang_guard(
            "export_diagnostics_zip_creates_zip_file_without_user_text",
            120,
        );
        let _lock = ZIP_EXPORT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let temp_zip = export_test_path("zip_no_user_text");
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

        assert!(
            zip_content_lossy.contains("report_summary.md"),
            "Diagnostics ZIP thiếu report_summary.md"
        );
        let _ = std::fs::remove_file(&temp_zip);
        if let Some(dir) = temp_zip.parent() {
            let _ = std::fs::remove_dir(dir);
        }
    }

    #[test]
    fn e69_linux_im_env_mismatch_detection() {
        assert!(
            detect_linux_im_env_mismatch(true, false, "fcitx", "", "").is_some(),
            "IBus chạy nhưng GTK_IM_MODULE=fcitx phải báo lệch"
        );
        assert!(
            detect_linux_im_env_mismatch(false, true, "ibus", "", "").is_some(),
            "Fcitx5 chạy nhưng GTK_IM_MODULE=ibus phải báo lệch"
        );
        assert!(
            detect_linux_im_env_mismatch(true, false, "ibus", "ibus", "@im=ibus").is_none(),
            "IBus khớp biến môi trường không được báo nhầm"
        );
        assert!(
            detect_linux_im_env_mismatch(false, true, "fcitx", "fcitx", "@im=fcitx").is_none(),
            "Fcitx5 khớp biến môi trường không được báo nhầm"
        );
    }

    /// R2-175: Fcitx5 5.0.x mặc định giữ Ctrl+Shift (EnumerateForwardKeys); ≥ 5.1 mặc định trống; mục
    /// đã sửa trong config thắng mặc định; Ctrl+Shift ở mục khác (TriggerKeys) cũng tính.
    #[test]
    fn r2_175_fcitx5_giu_ctrl_shift_theo_phien_ban_va_config() {
        assert_eq!(
            parse_fcitx5_version(
                "fcitx5 5.0.14
"
            ),
            Some((5, 0, 14))
        );
        assert_eq!(parse_fcitx5_version("5.1.7"), Some((5, 1, 7)));
        assert_eq!(parse_fcitx5_version("không có"), None);
        // Mặc định theo phiên bản (chưa có ~/.config/fcitx5/config).
        assert!(fcitx5_ctrl_shift_taken(Some("fcitx5 5.0.14"), None));
        assert!(!fcitx5_ctrl_shift_taken(Some("fcitx5 5.1.7"), None));
        assert!(
            !fcitx5_ctrl_shift_taken(None, None),
            "không biết phiên bản ⇒ không báo bừa"
        );
        // Người dùng đã xoá phím (mục có mặt, rỗng) ⇒ không giữ, kể cả 5.0.x.
        let cleared = "[Hotkey/EnumerateForwardKeys]

[Hotkey/EnumerateBackwardKeys]
";
        assert!(!fcitx5_ctrl_shift_taken(Some("5.0.14"), Some(cleared)));
        // Mục chỉ xoá một nửa: Backward vẫn theo mặc định 5.0.x.
        assert!(fcitx5_ctrl_shift_taken(
            Some("5.0.14"),
            Some(
                "[Hotkey/EnumerateForwardKeys]
"
            )
        ));
        // Ghi tường minh Ctrl+Shift ⇒ giữ, kể cả 5.1.
        let explicit = "[Hotkey/EnumerateForwardKeys]
0=Control+Shift_L
";
        assert!(fcitx5_ctrl_shift_taken(Some("5.1.7"), Some(explicit)));
        let trigger = "[Hotkey/TriggerKeys]
0=Shift+Control_R
";
        assert!(fcitx5_ctrl_shift_taken(Some("5.1.7"), Some(trigger)));
        // Phím khác (Ctrl+Space, Super+Space) không phải công tắc của TextVN.
        let other = "[Hotkey/TriggerKeys]
0=Control+space
[Hotkey/EnumerateForwardKeys]
0=Super+space
";
        assert!(!fcitx5_ctrl_shift_taken(Some("5.1.7"), Some(other)));
    }

    #[test]
    fn e69_fcitx5_profile_conflicts_detection() {
        let profile = "[Groups/0/Items/0]\nName=keyboard-us\n[Groups/0/Items/1]\nName=unikey\n[Groups/0/Items/2]\nName=bamboo\n[Groups/0/Items/3]\nName=textvn\n";
        let hits = detect_fcitx5_profile_conflicts(profile);
        assert_eq!(hits, vec!["fcitx5:unikey", "fcitx5:bamboo"]);
        assert!(detect_fcitx5_profile_conflicts("Name=textvn\nName=keyboard-us\n").is_empty());
    }

    #[test]
    fn e69_windows_conflict_name_and_version_format() {
        assert_eq!(
            detect_windows_conflict_name(r"C:\Program Files\UniKey\UniKeyNT.exe"),
            Some("UniKey")
        );
        assert_eq!(
            detect_windows_conflict_name(r"C:\Tools\EVKey64.exe"),
            Some("EVKey")
        );
        assert_eq!(
            detect_windows_conflict_name(r"C:\Apps\OpenKey64.exe"),
            Some("OpenKey")
        );
        assert_eq!(
            detect_windows_conflict_name(r"C:\Apps\GoTiengViet.exe"),
            Some("GoTiengViet")
        );
        assert_eq!(
            detect_windows_conflict_name(r"C:\Program Files\TextVN\TextVN.exe"),
            None
        );

        assert_eq!(
            format_windows_version(
                "Windows 10 Enterprise",
                "23H2",
                "22631",
                Some(4169),
                "Enterprise"
            ),
            "Windows 11 Enterprise 23H2 (Build 22631.4169)"
        );
        assert_eq!(
            format_windows_version(
                "Windows 10 Pro",
                "22H2",
                "19045",
                Some(5011),
                "Professional"
            ),
            "Windows 10 Pro 22H2 [Professional] (Build 19045.5011)"
        );
    }

    #[test]
    fn e69_macos_plist_parser() {
        let xml = r#"<plist><dict>
            <key>ProductName</key><string>macOS</string>
            <key>ProductVersion</key><string>15.1</string>
            <key>ProductBuildVersion</key><string>24B83</string>
        </dict></plist>"#;
        assert_eq!(
            parse_plist_string_value(xml, "ProductName").as_deref(),
            Some("macOS")
        );
        assert_eq!(
            parse_plist_string_value(xml, "ProductVersion").as_deref(),
            Some("15.1")
        );
        assert_eq!(
            parse_plist_string_value(xml, "ProductBuildVersion").as_deref(),
            Some("24B83")
        );
    }

    #[test]
    fn e70_detect_other_vi_tips_from_user_profile_values() {
        // Giá trị thật đọc trên máy dev Windows 11 (2026-11-25): TextVN + Microsoft Telex.
        let names: Vec<String> = [
            "0409:00000409",
            "0409:{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}",
            "042A:{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}",
            "042A:{C2CB2CF0-AF47-413E-9780-8BC3A3C16068}{5FB02EC5-0A77-4684-B4FA-DEF8A2195628}",
            "CachedLanguageName",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            detect_other_vi_tips(&names),
            vec!["Microsoft Vietnamese Telex".to_string()]
        );
        // VNI + bố cục 042A:0000042A (không phải bộ gõ) + chữ thường + TIP lạ.
        let names: Vec<String> = [
            "042A:0000042A",
            "042a:{c2cb2cf0-af47-413e-9780-8bc3a3c16068}{591ae943-56be-48f6-8966-06b43915cc5a}",
            "042A:{11111111-2222-3333-4444-555555555555}{66666666-7777-8888-9999-000000000000}",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let got = detect_other_vi_tips(&names);
        assert_eq!(got.len(), 2, "{got:?}");
        assert_eq!(got[0], "Microsoft Vietnamese Number Key-Based (VNI)");
        assert!(got[1].contains("{11111111-2222-3333-4444-555555555555}"));
        // Tên giá trị lạ có ký tự non-ASCII quanh byte 38 → bỏ qua, không panic.
        let odd: Vec<String> = vec![
            "042A:{ĐĐĐĐĐĐĐĐĐĐĐĐĐĐĐĐĐĐĐ}".into(),
            "042A:{".into(),
            "042A:".into(),
        ];
        assert!(detect_other_vi_tips(&odd).iter().all(|s| s.contains("TSF")));
        // Chỉ TextVN → không cảnh báo.
        let only: Vec<String> = vec![
            "042A:{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}{C4A91F52-77B3-4E19-8A6D-2F8C0B6E5A13}"
                .into(),
        ];
        assert!(detect_other_vi_tips(&only).is_empty());
    }

    #[test]
    fn e70_detect_apple_vi_input_sources() {
        let out = r#"(
        {
        InputSourceKind = "Keyboard Layout";
        "KeyboardLayout ID" = 0;
        "KeyboardLayout Name" = "U.S.";
    },
        {
        "Bundle ID" = "com.apple.inputmethod.VietnameseIM";
        InputSourceKind = "Keyboard Input Method";
    },
        {
        "Bundle ID" = "com.apple.inputmethod.VietnameseIM";
        "Input Mode" = "com.apple.inputmethod.VietnameseSimpleTelex";
        InputSourceKind = "Input Mode";
    },
        {
        "Bundle ID" = "com.apple.inputmethod.VietnameseIM";
        "Input Mode" = "com.apple.inputmethod.VietnameseVNI";
        InputSourceKind = "Input Mode";
    },
        {
        "Bundle ID" = "com.hunglinhpt.inputmethod.TextVN";
        InputSourceKind = "Keyboard Input Method";
    }
)"#;
        assert_eq!(
            detect_apple_vi_input_sources(out),
            vec![
                "Apple Vietnamese Simple Telex".to_string(),
                "Apple Vietnamese VNI".to_string()
            ]
        );
        assert!(detect_apple_vi_input_sources(
            "( { \"Bundle ID\" = \"com.apple.keylayout.US\"; } )"
        )
        .is_empty());
    }

    #[test]
    fn e70_classify_distribution_all_os() {
        let none = |_: &str| false;
        let unins = |p: &str| p.ends_with("/unins000.exe");
        assert_eq!(
            classify_distribution("windows", r"D:\Tools\Gõ Việt\TextVN", &none),
            "portable"
        );
        assert_eq!(
            classify_distribution("windows", r"C:\Program Files\TextVN", &unins),
            "installer-machine"
        );
        assert_eq!(
            classify_distribution(
                "windows",
                r"C:\Users\a\AppData\Local\Programs\TextVN",
                &unins
            ),
            "installer-user"
        );
        assert_eq!(
            classify_distribution(
                "windows",
                r"C:\Users\a\AppData\Local\Programs\TextVN-Store\0.2.90",
                &unins
            ),
            "store"
        );
        assert_eq!(
            classify_distribution("linux", "/usr/bin", &none),
            "system-package"
        );
        assert_eq!(
            classify_distribution("linux", "/home/a/.local/bin", &none),
            "user-install"
        );
        assert_eq!(
            classify_distribution("linux", "/tmp/TextVN-0.2.90-linux-x86_64/bin", &none),
            "portable"
        );
        assert_eq!(
            classify_distribution("macos", "/Applications/TextVN.app/Contents/MacOS", &none),
            "app-bundle"
        );
        assert!(!BUILD_TARGET.is_empty());
        assert!(!BUILD_GIT_SHA.is_empty());
    }

    #[test]
    fn e70_export_contains_panic_tail_and_build_identity() {
        let _guard = hang_guard("e70_export_contains_panic_tail_and_build_identity", 120);
        let _lock = ZIP_EXPORT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let report = collect_report();
        let out = export_test_path("e70_panic_tail");
        export_diagnostics_zip(&report, &out).expect("export");
        let bytes = std::fs::read(&out).unwrap();
        let _ = std::fs::remove_file(&out);
        if let Some(dir) = out.parent() {
            let _ = std::fs::remove_dir(dir);
        }
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("panic_tail.log"));
        assert!(text.contains("\"git_sha\""));
        assert!(text.contains("\"distribution\""));
        assert!(text.contains("\"target\""));
        assert_eq!(
            count_panic_records("=== TextVN panic a\n=== TextVN panic b\n"),
            2
        );
    }

    #[test]
    fn e69_markdown_report_renders_diagnostics_safely() {
        let mut report = collect_report();
        report.diagnostics.push(DiagnosticItem {
            code: "W_CONFLICT_VI_IME",
            severity: "warning",
            category: "conflict",
            message: "Phát hiện bộ gõ cũ UniKey đang chạy".into(),
            suggestion: "Thoát UniKey ở khay hệ thống trước khi dùng TextVN.".into(),
        });
        let md = report.to_markdown_report();
        assert!(md.contains("# Báo cáo chẩn đoán TextVN"));
        assert!(md.contains("W_CONFLICT_VI_IME"));
        assert!(md.contains("Thoát UniKey"));
        assert!(md.contains("G8/S2"));
    }
}
