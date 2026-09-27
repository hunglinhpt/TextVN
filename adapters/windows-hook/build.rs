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
    println!("cargo:rerun-if-changed=hook.rc");
    println!("cargo:rerun-if-changed=hook.manifest");

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
    let rc_file = manifest_dir.join("hook.rc");
    let res_file = out_dir.join("hook.res");

    let status = Command::new(&rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res_file.display()))
        .arg(&rc_file)
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
