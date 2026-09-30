// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::Command;

fn find_rc_exe() -> Option<PathBuf> {
    if let Ok(output) = Command::new("where.exe").arg("rc.exe").output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = out.lines().next() {
                let p = PathBuf::from(line.trim());
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    let kits_root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if kits_root.exists() {
        if let Ok(entries) = std::fs::read_dir(kits_root) {
            let mut versions: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|x| x.path()))
                .filter(|p| p.is_dir() && p.join("x64").join("rc.exe").exists())
                .collect();
            versions.sort();
            if let Some(latest) = versions.last() {
                return Some(latest.join("x64").join("rc.exe"));
            }
        }
    }

    None
}

fn main() {
    println!("cargo:rerun-if-changed=ffi.rc");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let out_dir = match std::env::var("OUT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => return,
    };

    let Some(rc_exe) = find_rc_exe() else {
        println!("cargo:warning=rc.exe not found. Skipping resource compilation.");
        return;
    };

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let rc_file = manifest_dir.join("ffi.rc");
    let res_file = out_dir.join("ffi.res");

    // PE VersionInfo theo **CARGO_PKG_VERSION** — build.rs patch bản sao .rc
    // trong OUT_DIR (file .rc tĩnh chỉ là mẫu; bump version chỉ sửa Cargo.toml).
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version_commas = format!("{version},0");
    let generated = match std::fs::read_to_string(&rc_file) {
        Ok(content) => content
            .lines()
            .map(|line| {
                let t = line.trim_start();
                if t.starts_with("FILEVERSION ") {
                    return format!("FILEVERSION {version_commas}");
                }
                if t.starts_with("PRODUCTVERSION ") {
                    return format!("PRODUCTVERSION {version_commas}");
                }
                if t.contains("\"FileVersion\"") {
                    return format!("            VALUE \"FileVersion\", \"{version}.0\"");
                }
                if t.contains("\"ProductVersion\"") {
                    return format!("            VALUE \"ProductVersion\", \"{version}.0\"");
                }
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\r\n"),
        Err(e) => {
            println!("cargo:warning=Không đọc được {}: {e}", rc_file.display());
            return;
        }
    };
    let generated_rc = out_dir.join("ffi_versioned.rc");
    if let Err(e) = std::fs::write(&generated_rc, &generated) {
        println!(
            "cargo:warning=Không ghi được {}: {e}",
            generated_rc.display()
        );
        return;
    }

    let status = Command::new(&rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res_file.display()))
        .arg(&generated_rc)
        .current_dir(&manifest_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-arg={}", res_file.display());
        }
        Ok(s) => {
            println!("cargo:warning=rc.exe exited with code: {:?}", s.code());
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute rc.exe: {e}");
        }
    }
}
