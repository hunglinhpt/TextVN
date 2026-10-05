// SPDX-License-Identifier: GPL-3.0-or-later
//! Bootstrap cho bản MSIX (WindowsApps) — vòng 12 audit MSIX.
//!
//! Bản cài qua Microsoft Store đặt exe trong `C:\Program Files\WindowsApps\
//! <Package>_<version>_<arch>__<hash>\` — đường dẫn chứa **version** và bị
//! XOÁ sau mỗi lần Store update. Nếu TIP DLL được đăng ký trỏ vào đó:
//! (a) đăng ký COM/CTF treo lơ lửng ngay sau lần update đầu (thư mục cũ bị
//! xoá) → mất gõ cho tới khi mở lại app; (b) ghi registry của tiến trình
//! đóng gói có thể bị **registry virtualization** điều hướng vào hive riêng
//! của package → đăng ký vô hình với Notepad/explorer (tiến trình thường).
//!
//! Giải pháp "kênh phân phối" (pattern đã định hướng từ 0.2.17): exe trong
//! package chỉ làm MỘT việc — stage toàn bộ payload ra thư mục thường
//! `%LOCALAPPDATA%\Programs\TextVN` rồi spawn bản staged và thoát. Bản staged
//! chạy **non-packaged** nên dùng lại NGUYÊN VẸN mọi cơ chế đã chứng minh của
//! bản portable: tự đăng ký TSF, đề nghị UAC phạm vi máy (B7), Run key, IPC.
//!
//! Không đóng gói → không ảnh hưởng bản cài/portable: `is_packaged_exe` chỉ
//! khớp đường dẫn chứa `\WindowsApps\`.

use std::path::{Path, PathBuf};

/// exe có đang chạy từ thư mục package của Store/MSIX không?
pub fn is_packaged_exe(exe: &Path) -> bool {
    exe.to_string_lossy()
        .to_lowercase()
        .contains("\\windowsapps\\")
}

/// Thư mục stage đích (ổn định, không đổi theo version của package).
pub fn staged_install_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    if local.is_empty() {
        return None;
    }
    Some(PathBuf::from(local).join("Programs").join("TextVN"))
}

/// Copy file ghi-đè; nếu đích đang bị khoá (exe đang chạy / TSF đang nạp DLL)
/// thì **đổi tên** file cũ thành `.old-<ts>` (Windows CHO PHÉP rename file
/// đang map — B8) rồi copy lại.
fn copy_overwrite(src: &Path, dst: &Path) -> std::io::Result<()> {
    match std::fs::copy(src, dst) {
        Ok(_) => Ok(()),
        Err(_locked) if dst.exists() => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let name = dst
                .file_name()
                .map(|n| format!("{}.old-{}", n.to_string_lossy(), ts))
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name")
                })?;
            let backup = dst.with_file_name(name);
            std::fs::rename(dst, &backup)?;
            std::fs::copy(src, dst)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Dọn các file `.old-*` của lần stage trước (best-effort).
fn cleanup_old_backups(target: &Path) {
    if let Ok(entries) = std::fs::read_dir(target) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains(".old-") && entry.path().is_file() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            copy_overwrite(&entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Stage toàn bộ payload cần thiết cho tray staged từ thư mục package sang
/// `target`. DLL ưu tiên tên đã đổi (`textvn-tsf.dll`), fallback tên build
/// (`textvn_win_tsf.dll`). Lenient: file nào copy được thì staged, lỗi từng
/// file không chặn (bản staged cũ vẫn dùng được).
pub fn stage_package_payload(pkg_dir: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    cleanup_old_backups(target);

    // DLL: tên release trước, tên build workspace sau (cả hai cùng payload).
    let dll_src = ["textvn-tsf.dll", "textvn_win_tsf.dll"]
        .iter()
        .map(|n| pkg_dir.join(n))
        .find(|p| p.is_file());
    if let Some(src) = dll_src {
        copy_overwrite(&src, &target.join("textvn-tsf.dll"))?;
    }
    for name in ["TextVN.exe", "textvn-cli.exe"] {
        let src = pkg_dir.join(name);
        if src.is_file() {
            copy_overwrite(&src, &target.join(name))?;
        }
    }
    for dir in ["resources", "data"] {
        let src = pkg_dir.join(dir);
        if src.is_dir() {
            copy_dir_recursive(&src, &target.join(dir))?;
        }
    }
    for doc in ["PRIVACY_POLICY.txt", "HUONG_DAN_SU_DUNG.txt", "README.md"] {
        let src = pkg_dir.join(doc);
        if src.is_file() {
            copy_overwrite(&src, &target.join(doc))?;
        }
    }
    Ok(())
}

/// Toàn bộ luồng bootstrap: nếu exe đang chạy từ WindowsApps → stage payload
/// → spawn bản staged với cùng đối số → trả `true` (caller phải THOÁT ngay,
/// trước khi tạo single-instance mutex). Trả `false` khi: không phải package,
/// thiếu LOCALAPPDATA, stage lỗi toàn phần, hoặc spawn lỗi (khi đó chạy tiếp
/// từ package — tốt hơn không gõ gì).
pub fn bootstrap_relaunch_from_package() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if !is_packaged_exe(&exe) {
        return false;
    }
    let Some(pkg_dir) = exe.parent() else {
        return false;
    };
    let Some(target) = staged_install_dir() else {
        return false;
    };
    if let Err(e) = stage_package_payload(pkg_dir, &target) {
        eprintln!("TextVN MSIX bootstrap: stage FAIL: {e}");
        return false;
    }
    let staged_exe = target.join("TextVN.exe");
    if !staged_exe.is_file() {
        eprintln!("TextVN MSIX bootstrap: staged TextVN.exe thieu");
        return false;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match std::process::Command::new(&staged_exe).args(&args).spawn() {
        Ok(_child) => {
            eprintln!("TextVN MSIX bootstrap: da chay tu {}", target.display());
            true
        }
        Err(e) => {
            eprintln!("TextVN MSIX bootstrap: spawn staged FAIL: {e}");
            false
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("textvn-bootstrap-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn is_packaged_exe_nhan_dien_windowsapps_va_loai_truong_hop_thuong() {
        assert!(is_packaged_exe(Path::new(
            r"C:\Program Files\WindowsApps\TextVN_0.2.25.0_x64__abc\TextVN.exe"
        )));
        assert!(is_packaged_exe(Path::new(
            r"c:\program files\windowsapps\textvn_0.2.25.0_x64__abc\TextVN.exe"
        )));
        assert!(!is_packaged_exe(Path::new(
            r"C:\Program Files\TextVN\TextVN.exe"
        )));
        assert!(!is_packaged_exe(Path::new(
            r"C:\Users\u\AppData\Local\Programs\TextVN\TextVN.exe"
        )));
        // không nhầm "windowsapps" ở giữa tên thư mục khác
        assert!(!is_packaged_exe(Path::new(
            r"D:\my-windowsapps-backup\TextVN.exe"
        )));
    }

    #[test]
    fn stage_payload_copy_du_file_va_dir() {
        let pkg = temp_dir("pkg");
        let target = temp_dir("target");
        std::fs::write(pkg.join("TextVN.exe"), b"exe").unwrap();
        std::fs::write(pkg.join("textvn-cli.exe"), b"cli").unwrap();
        std::fs::write(pkg.join("textvn-tsf.dll"), b"dll").unwrap();
        std::fs::create_dir_all(pkg.join("resources")).unwrap();
        std::fs::write(pkg.join("resources").join("textvn_v.ico"), b"ico").unwrap();
        std::fs::create_dir_all(pkg.join("data")).unwrap();
        std::fs::write(pkg.join("data").join("appdb.default.json"), b"{}").unwrap();
        std::fs::write(pkg.join("PRIVACY_POLICY.txt"), b"privacy").unwrap();

        stage_package_payload(&pkg, &target).unwrap();

        assert_eq!(std::fs::read(target.join("TextVN.exe")).unwrap(), b"exe");
        assert_eq!(
            std::fs::read(target.join("textvn-cli.exe")).unwrap(),
            b"cli"
        );
        assert_eq!(
            std::fs::read(target.join("textvn-tsf.dll")).unwrap(),
            b"dll"
        );
        assert_eq!(
            std::fs::read(target.join("resources").join("textvn_v.ico")).unwrap(),
            b"ico"
        );
        assert_eq!(
            std::fs::read(target.join("data").join("appdb.default.json")).unwrap(),
            b"{}"
        );
        assert_eq!(
            std::fs::read(target.join("PRIVACY_POLICY.txt")).unwrap(),
            b"privacy"
        );
        let _ = std::fs::remove_dir_all(&pkg);
        let _ = std::fs::remove_dir_all(&target);
    }

    #[test]
    fn stage_payload_fallback_ten_build_va_ghi_de_khi_stage_lai() {
        let pkg = temp_dir("pkg2");
        let target = temp_dir("target2");
        // Chỉ có tên build workspace → phải được đổi thành textvn-tsf.dll
        std::fs::write(pkg.join("textvn_win_tsf.dll"), b"v1").unwrap();
        std::fs::write(pkg.join("TextVN.exe"), b"v1").unwrap();
        stage_package_payload(&pkg, &target).unwrap();
        assert_eq!(std::fs::read(target.join("textvn-tsf.dll")).unwrap(), b"v1");

        // Stage lại với nội dung mới → ghi đè (không lỗi, không .old thừa khi
        // file không bị khoá)
        std::fs::write(pkg.join("textvn_win_tsf.dll"), b"v2").unwrap();
        std::fs::write(pkg.join("TextVN.exe"), b"v2").unwrap();
        stage_package_payload(&pkg, &target).unwrap();
        assert_eq!(std::fs::read(target.join("textvn-tsf.dll")).unwrap(), b"v2");
        assert_eq!(std::fs::read(target.join("TextVN.exe")).unwrap(), b"v2");

        let _ = std::fs::remove_dir_all(&pkg);
        let _ = std::fs::remove_dir_all(&target);
    }

    #[test]
    fn bootstrap_khong_relaunch_khi_khong_phai_package() {
        // current_exe của test binary chạy từ target\... — không phải WindowsApps
        assert!(!bootstrap_relaunch_from_package());
    }
}
