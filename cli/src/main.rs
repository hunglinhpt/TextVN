// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn` — CLI (P0-1 §2).
//!
//! Slice 1: `replay` (golden corpus, P0-4). Slice này thêm: `sizes` (verify ABI — P0-2 §6),
//! `config init|validate|default` (P0-1 §2, P0-3 §1), `doctor`. `register` (TIP) thuộc
//! WIN-003/WIN-010+ (cần Windows API) và `doctor --export` thuộc WIN-058 — chưa có.

mod doctor;
mod register;
mod replay;
mod verify;

use std::path::PathBuf;
use std::process::exit;

const USAGE: &str = r#"textvn-cli — TextVN CLI

Usage:
  textvn-cli replay <dir-or-file...> [--adapter headless|win|tsf|mac|linux] [--json] [--filter <substring>]
  textvn-cli verify [--header <path>] [--json]  # header C khớp code Rust? (P0-2 §6) + sizeof/offset
  textvn-cli sizes [--json]                  # verify struct ABI (P0-2 §6): ime_key_v1=20 · ime_result_v1=532
  textvn-cli config default                  # in config mặc định ra stdout
  textvn-cli config init                     # ghi config mặc định vào đường dẫn per-OS (không ghi đè)
  textvn-cli config validate <file.json>     # validate config.v1 (P0-3 §1)
  textvn-cli doctor [--json] [--export <path.zip>] # chẩn đoán môi trường / xuất báo cáo (WIN-058)
  textvn-cli register [--scope user|machine] [--dll <path>] [--no-taskbar] # đăng ký Text Services Framework TIP (WIN-003)
  textvn-cli unregister [--scope user|machine] [--if-owned-by <dir>] # hủy đăng ký TSF TIP (--if-owned-by: chỉ khi TIP thuộc <dir>)
  textvn-cli activate                                      # kích hoạt profile TextVN cho phiên hiện tại (tray gọi sau Ctrl+Shift)
  textvn-cli schedule-delete <path...>                     # xoá ngay hoặc hẹn xoá trước lần khởi động kế tiếp (uninstaller dùng cho DLL bị khoá)
  textvn-cli --help

Exit codes:
  replay     : 0 pass hết · 1 có case fail · 2 lỗi dùng/parse (P0-4 §4)
  verify     : 0 header khớp · 1 lệch · 2 lỗi dùng/đọc file
  sizes      : 0 khớp · 1 lệch (đổi struct = bump IME_ABI_VERSION — P0-2 §6)
  config     : 0 hợp lệ · 1 không hợp lệ/tồn tại · 2 lỗi dùng/đọc-ghi file
  doctor     : 0 mọi check pass · 1 có check fail · 2 lỗi xuất file
  register   : 0 thành công · 1 lỗi đăng ký/không thấy file · 2 lỗi dùng · 3 cần quyền Administrator / DLL ngoài Program Files (--scope machine)
  unregister : 0 thành công (kể cả bỏ qua vì TIP thuộc bản khác với --if-owned-by) · 1 lỗi hủy · 2 lỗi dùng
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("replay") => exit(cmd_replay(&args[2..])),
        Some("verify") => exit(cmd_verify(&args[2..])),
        Some("sizes") => exit(cmd_sizes(&args[2..])),
        Some("config") => exit(cmd_config(&args[2..])),
        Some("doctor") => exit(cmd_doctor(&args[2..])),
        Some("register") => exit(cmd_register(&args[2..])),
        Some("unregister") => exit(cmd_unregister(&args[2..])),
        Some("activate") => exit(register::activate_tip()),
        Some("schedule-delete") => exit(register::schedule_delete(
            &args[2..].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )),
        Some("--help") | Some("-h") => print!("{USAGE}"),
        Some(other) => {
            eprintln!("error: lệnh lạ `{other}`");
            eprint!("{USAGE}");
            exit(2);
        }
        None => {
            eprint!("{USAGE}");
            exit(2);
        }
    }
}

/// `replay` — golden corpus (P0-4). Exit: 0 pass hết · 1 có fail · 2 lỗi dùng/parse.
fn cmd_replay(rest: &[String]) -> i32 {
    let mut paths: Vec<String> = Vec::new();
    let mut adapter = "headless".to_string();
    let mut json = false;
    let mut filter: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--adapter" => {
                i += 1;
                adapter = rest.get(i).cloned().unwrap_or_default();
            }
            "--json" => json = true,
            "--filter" => {
                i += 1;
                filter = rest.get(i).cloned();
                if filter.is_none() {
                    eprintln!("error: --filter cần giá trị");
                    return 2;
                }
            }
            other if other.starts_with("--") => {
                eprintln!("error: flag lạ `{other}`");
                return 2;
            }
            other => paths.push(other.to_string()),
        }
        i += 1;
    }
    if paths.is_empty() {
        eprintln!("error: replay cần ít nhất 1 dir/file corpus");
        eprint!("{USAGE}");
        return 2;
    }
    // Giá trị lạ → exit 2 (P0-4 §4)
    if !matches!(
        adapter.as_str(),
        "headless" | "win" | "tsf" | "mac" | "linux"
    ) {
        eprintln!("error: --adapter phải là headless|win|tsf|mac|linux (nhận `{adapter}`)");
        return 2;
    }
    replay::run(&paths, &adapter, json, filter.as_deref())
}

/// `verify` — header C (`ffi/include/textvn_ffi.h`) khớp code Rust? (P0-2 §6).
/// Header đang giữ tay (chưa có `xtask cbindgen`) nên đây là gate thay thế:
/// thêm/bớt trường hoặc đổi hằng ở một bên mà quên bên kia → exit 1.
fn cmd_verify(rest: &[String]) -> i32 {
    let mut header = verify::default_header();
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--header" => {
                i += 1;
                match rest.get(i) {
                    Some(p) => header = p.clone(),
                    None => {
                        eprintln!("error: --header cần đường dẫn");
                        return 2;
                    }
                }
            }
            "--json" => json = true,
            other => {
                eprintln!("error: `verify` không nhận `{other}` (chỉ --header/--json)");
                return 2;
            }
        }
        i += 1;
    }
    match verify::run(&header, json) {
        Ok(text) => {
            println!("{text}");
            0
        }
        Err(list) => {
            if json {
                println!(
                    "{{\"ok\":false,\"mismatches\":[{}]}}",
                    list.iter()
                        .map(|l| format!("\"{}\"", l.replace('"', "'")))
                        .collect::<Vec<_>>()
                        .join(",")
                );
            } else {
                eprintln!("header C LỆCH với code Rust:");
                for l in &list {
                    eprintln!("{l}");
                }
            }
            1
        }
    }
}

/// Kích thước struct ABI (P0-2 §6) → (key, result, context, ok). `context` chứa con trỏ
/// nên chỉ theo platform — không nằm trong điều kiện `ok`.
fn abi_sizes() -> (usize, usize, usize, bool) {
    use std::mem::size_of;
    let key = size_of::<textvn_ffi::ime_key_v1>();
    let result = size_of::<textvn_ffi::ime_result_v1>();
    let context = size_of::<textvn_ffi::ime_context_v1>();
    (key, result, context, key == 20 && result == 532)
}

/// `sizes` — verify struct ABI (P0-2 §6): `ime_key_v1` = 20 · `ime_result_v1` = 532.
/// Lệch → exit 1 (struct đã đổi mà chưa bump `IME_ABI_VERSION`).
fn cmd_sizes(rest: &[String]) -> i32 {
    use std::mem::offset_of;

    let json_out = match rest {
        [] => false,
        [only] if only == "--json" => true,
        _ => {
            eprintln!("error: `sizes` chỉ nhận [--json]");
            return 2;
        }
    };

    let (key, result, context, ok) = abi_sizes();

    if json_out {
        println!(
            "{{\"abi\":{},\"key\":{key},\"result\":{result},\"context\":{context},\"ok\":{ok}}}",
            textvn_ffi::IME_ABI_VERSION
        );
        return i32::from(!ok);
    }

    println!("ABI v{} (IME_ABI_VERSION)", textvn_ffi::IME_ABI_VERSION);
    println!(
        "  ime_key_v1      {key:>4} bytes  {}",
        if key == 20 {
            "OK"
        } else {
            "LỆCH (kỳ vọng 20)"
        }
    );
    println!(
        "  ime_result_v1   {result:>4} bytes  {}",
        if result == 532 {
            "OK"
        } else {
            "LỆCH (kỳ vọng 532)"
        }
    );
    println!("  ime_context_v1  {context:>4} bytes  (theo platform — P0-2 §1 không assert)");
    println!(
        "  offsets: result.insert={} result.preedit={} key.ch={} key.mods={}",
        offset_of!(textvn_ffi::ime_result_v1, insert),
        offset_of!(textvn_ffi::ime_result_v1, preedit),
        offset_of!(textvn_ffi::ime_key_v1, ch),
        offset_of!(textvn_ffi::ime_key_v1, mods),
    );
    if !ok {
        eprintln!("error: struct ABI lệch — sửa struct = bump IME_ABI_VERSION (P0-2 §6)");
    }
    i32::from(!ok)
}

/// `config init` · `config validate <file>` · `config default` (P0-1 §2, P0-3 §1).
/// Không in nội dung config người dùng (S2) — chỉ in kết luận + trường không nhạy cảm.
fn cmd_config(rest: &[String]) -> i32 {
    match rest.first().map(String::as_str) {
        Some("default") => {
            println!("{}", textvn_config::default_json());
            0
        }
        Some("init") => {
            // Ghi config mặc định vào đường dẫn per-OS — không ghi đè file đang tồn tại
            // (tránh mất tuỳ chọn của user; xoá tay hoặc dùng `validate` để kiểm tra).
            let Some(path) = config_path() else {
                eprintln!("error: không xác định được thư mục cấu hình trên OS này");
                return 2;
            };
            if path.exists() {
                eprintln!(
                    "config đã tồn tại: {} — không ghi đè (kiểm tra bằng `textvn config validate {}`)",
                    path.display(),
                    path.display()
                );
                return 1;
            }
            let Some(parent) = path.parent() else {
                eprintln!("error: đường dẫn config không hợp lệ");
                return 2;
            };
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: không tạo được {}: {e}", parent.display());
                return 2;
            }
            if let Err(e) = std::fs::write(&path, textvn_config::default_json() + "\n") {
                eprintln!("error: không ghi được {}: {e}", path.display());
                return 2;
            }
            println!("Đã tạo config mặc định: {}", path.display());
            0
        }
        Some("validate") => {
            let Some(path) = rest.get(1) else {
                eprintln!("error: `config validate` cần đường dẫn file");
                return 2;
            };
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error: không đọc được `{path}`: {e}");
                    return 2;
                }
            };
            match textvn_config::parse_config(&text) {
                Ok(cfg) => {
                    println!(
                        "OK  {path}: config_version={} method={} diacritic_style={:?} macros={} emoji={}",
                        cfg.config_version,
                        cfg.method.as_str(),
                        cfg.diacritic_style,
                        cfg.macros.len(),
                        cfg.emoji.len()
                    );
                    0
                }
                Err(e) => {
                    eprintln!("LỖI {path}: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!("error: `config` cần `validate <file>` hoặc `default`");
            2
        }
    }
}

/// Đường dẫn `config.json` per-OS (P0-3 §1 — một nơi duy nhất cho mỗi OS).
fn config_path() -> Option<PathBuf> {
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
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/TextVN/config.json"))
    }
}

/// `doctor` — chẩn đoán môi trường và xuất báo cáo chẩn đoán (WIN-058 / P1-4 §9).
fn cmd_doctor(rest: &[String]) -> i32 {
    let mut json_out = false;
    let mut export_path: Option<PathBuf> = None;
    let mut i = 0;

    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json_out = true,
            "--export" => {
                if let Some(next) = rest.get(i + 1) {
                    if !next.starts_with("--") {
                        export_path = Some(PathBuf::from(next));
                        i += 1;
                    } else {
                        export_path = Some(PathBuf::from("textvn-diagnostics.zip"));
                    }
                } else {
                    export_path = Some(PathBuf::from("textvn-diagnostics.zip"));
                }
            }
            other => {
                eprintln!("error: `doctor` không nhận tham số `{other}`");
                eprintln!("Usage: textvn doctor [--json] [--export <path.zip>]");
                return 2;
            }
        }
        i += 1;
    }

    doctor::run_doctor(json_out, export_path.as_deref())
}

/// `register` — đăng ký Text Services Framework TIP vào hệ thống (WIN-003 / P1-1 §8).
fn cmd_register(rest: &[String]) -> i32 {
    let mut scope = "user".to_string();
    let mut dll_path: Option<PathBuf> = None;
    let mut no_taskbar = false;
    let mut i = 0;

    if rest.first().map(String::as_str) == Some("status") {
        return register::status_tip(&scope);
    }

    while i < rest.len() {
        match rest[i].as_str() {
            "--scope" => {
                i += 1;
                match rest.get(i) {
                    Some(s) if s == "user" || s == "machine" => scope = s.clone(),
                    Some(other) => {
                        eprintln!("error: --scope phải là `user` hoặc `machine` (nhận `{other}`)");
                        return 2;
                    }
                    None => {
                        eprintln!("error: --scope cần giá trị `user` hoặc `machine`");
                        return 2;
                    }
                }
            }
            "--dll" => {
                i += 1;
                match rest.get(i) {
                    Some(p) => dll_path = Some(PathBuf::from(p)),
                    None => {
                        eprintln!("error: --dll cần đường dẫn file");
                        return 2;
                    }
                }
            }
            "--no-taskbar" => {
                no_taskbar = true;
            }
            "status" => {
                return register::status_tip(&scope);
            }
            other => {
                eprintln!("error: `register` không nhận tham số `{other}`");
                eprintln!(
                    "Usage: textvn register [--scope user|machine] [--dll <path>] [--no-taskbar]"
                );
                return 2;
            }
        }
        i += 1;
    }

    register::register_tip(&scope, dll_path.as_deref(), no_taskbar)
}

/// `unregister` — hủy đăng ký Text Services Framework TIP khỏi hệ thống (WIN-010 / P1-1 §8).
fn cmd_unregister(rest: &[String]) -> i32 {
    let mut scope = "user".to_string();
    let mut owned_by: Option<PathBuf> = None;
    let mut i = 0;

    while i < rest.len() {
        match rest[i].as_str() {
            "--scope" => {
                i += 1;
                match rest.get(i) {
                    Some(s) if s == "user" || s == "machine" => scope = s.clone(),
                    Some(other) => {
                        eprintln!("error: --scope phải là `user` hoặc `machine` (nhận `{other}`)");
                        return 2;
                    }
                    None => {
                        eprintln!("error: --scope cần giá trị `user` hoặc `machine`");
                        return 2;
                    }
                }
            }
            // R2-30: gỡ một bản (portable/Store) không được tắt TIP của bản khác.
            "--if-owned-by" => {
                i += 1;
                match rest.get(i) {
                    Some(d) if !d.is_empty() => owned_by = Some(PathBuf::from(d)),
                    _ => {
                        eprintln!("error: --if-owned-by cần đường dẫn thư mục");
                        return 2;
                    }
                }
            }
            other => {
                eprintln!("error: `unregister` không nhận tham số `{other}`");
                eprintln!("Usage: textvn unregister [--scope user|machine] [--if-owned-by <dir>]");
                return 2;
            }
        }
        i += 1;
    }

    register::unregister_tip(&scope, owned_by.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_register_invalid_scope_returns_2() {
        let args = vec!["--scope".to_string(), "invalid_scope".to_string()];
        assert_eq!(cmd_register(&args), 2);
    }

    #[test]
    fn cmd_register_unknown_flag_returns_2() {
        let args = vec!["--unknown".to_string()];
        assert_eq!(cmd_register(&args), 2);
    }

    #[test]
    fn cmd_unregister_if_owned_by_requires_dir() {
        let args = vec!["--if-owned-by".to_string()];
        assert_eq!(cmd_unregister(&args), 2);
    }

    #[test]
    fn cmd_unregister_invalid_scope_returns_2() {
        let args = vec!["--scope".to_string(), "invalid_scope".to_string()];
        assert_eq!(cmd_unregister(&args), 2);
    }
}
