// SPDX-License-Identifier: GPL-3.0-or-later
//! Build script: nhúng Windows resource (icon, manifest) vào vietime-tray.exe.
//!
//! Chỉ chạy khi build trên Windows với feature `embed-resources` bật.
//! Build thông thường (dev, CI cross) không cần feature này.

fn main() {
    // Chỉ cần embed resource trên Windows
    #[cfg(all(windows, feature = "embed-resources"))]
    embed_win_resources();

    // Re-run nếu icon hoặc manifest thay đổi
    println!("cargo:rerun-if-changed=resources/vietime.ico");
    println!("cargo:rerun-if-changed=resources/vietime.manifest");
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(all(windows, feature = "embed-resources"))]
fn embed_win_resources() {
    use std::path::PathBuf;

    let manifest_path = PathBuf::from("resources/vietime.manifest");
    let ico_path = PathBuf::from("resources/vietime.ico");

    let mut res = winresource::WindowsResource::new();

    if ico_path.exists() {
        res.set_icon(ico_path.to_str().unwrap());
    }

    if manifest_path.exists() {
        res.set_manifest_file(manifest_path.to_str().unwrap());
    }

    res.set("ProductName", "VietIME");
    res.set("FileDescription", "VietIME Tray Application");
    res.set("LegalCopyright", "© 2024 VietIME Contributors. GPL-3.0-or-later.");
    res.set("CompanyName", "VietIME Project");
    res.set("InternalName", "vietime-tray");
    res.set("OriginalFilename", "vietime-tray.exe");

    if let Err(e) = res.compile() {
        eprintln!("cargo:warning=winresource compile failed: {e}");
    }
}
