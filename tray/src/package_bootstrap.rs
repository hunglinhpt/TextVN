// SPDX-License-Identifier: GPL-3.0-or-later
//! Bootstrap kênh Microsoft Store (MSIX) — ba bước, xem tổng quan ở [`crate::store`].
//!
//! Bản cài qua Store đặt exe trong `C:\Program Files\WindowsApps\<Package>_<ver>_...\`:
//! đường dẫn đổi (và bị xoá) sau mỗi lần Store cập nhật, và mọi ghi HKCU / file mới dưới
//! AppData của tiến trình có package identity bị Windows ẢO HOÁ (hive/thư mục riêng của
//! package, vô hình với Notepad/Explorer, mất khi gỡ — R2-02/R2-19/R2-22). Bản trước
//! copy ra `%LOCALAPPDATA%\Programs\TextVN` ngay trong package (bị ảo hoá, trùng thư mục
//! bộ cài Inno per-user — R2-08/R2-24) rồi spawn thường (con vẫn có thể ở trong
//! container), và khi copy lỗi thì chạy tray luôn từ WindowsApps (R2-37).
//!
//! Luồng mới:
//! 1. [`run_packaged_entry`] (có identity): KHÔNG đăng ký gì. Copy payload ra
//!    `%USERPROFILE%\.textvn\msix-staging\<V>\` (ngoài AppData — ghi thật), spawn chính
//!    nó `--msix-relay` với DESKTOP_APP_POLICY breakaway rồi thoát. Copy lỗi → hộp thoại,
//!    KHÔNG BAO GIỜ chạy tray từ WindowsApps.
//! 2. [`run_relay`] (vẫn trong package; con của nó ra ngoài): chạy
//!    `<staging>\TextVN.exe --msix-install --pfn <PFN> --ver <V>` rồi thoát.
//! 3. [`run_install`] (phải KHÔNG có identity): copy vào
//!    `%LOCALAPPDATA%\Programs\TextVN-Store\<V>\` (thư mục tạm rồi rename — bản cũ
//!    nguyên vẹn nếu lỗi), ghi `stage.json`, đặt Run value, dừng tray Store cũ, chạy tray
//!    mới, dọn bản cũ.

use std::path::Path;

#[cfg(windows)]
use crate::store::{self, LaunchMode, StageInfo};
#[cfg(windows)]
use crate::store_win;

/// Copy file ghi-đè; nếu đích đang bị khoá (exe đang chạy / TSF đang nạp DLL) thì
/// **đổi tên** file cũ thành `.old-<ts>` (Windows CHO PHÉP rename file đang map — B8)
/// rồi copy lại. Copy lần hai lỗi → đổi tên bản cũ VỀ chỗ cũ (R2-05: không để thư mục
/// thiếu `TextVN.exe`/`textvn-tsf.dll`).
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
            if let Err(e) = std::fs::copy(src, dst) {
                let _ = std::fs::remove_file(dst);
                let _ = std::fs::rename(&backup, dst);
                return Err(e);
            }
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

/// Copy toàn bộ payload cần cho tray kênh Store từ `pkg_dir` sang `target`. DLL ưu tiên
/// tên đã đổi (`textvn-tsf.dll`), fallback tên build (`textvn_win_tsf.dll`).
/// NGHIÊM NGẶT (R2-37): lỗi ở bất kỳ file nào → trả lỗi; caller copy vào thư mục tạm
/// rồi mới rename ([`stage_version_dir`]) nên bản cũ không bao giờ bị trộn version.
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
    // `textvn-tsf-x86.dll` (vòng 14): `build-msix.ps1` đóng gói nó cho app 32-bit
    // (Zalo PC, Office/Notepad++ x86) — thiếu ở bản staged thì `register` bỏ qua mirror
    // WOW64 và app x86 không bao giờ thấy TIP trên bản Store.
    for name in ["TextVN.exe", "textvn-cli.exe", "textvn-tsf-x86.dll"] {
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
    // LICENSE đi cùng bản cài (GPL-3.0 §4–6: bản phân phối phải kèm văn bản giấy phép).
    for doc in [
        "PRIVACY_POLICY.txt",
        "HUONG_DAN_SU_DUNG.txt",
        "README.md",
        "LICENSE",
        "CHANGELOG.md",
    ] {
        let src = pkg_dir.join(doc);
        if src.is_file() {
            copy_overwrite(&src, &target.join(doc))?;
        }
    }
    Ok(())
}

/// Thư mục phiên bản đã copy ĐỦ: có marker và `TextVN.exe`.
pub fn version_dir_complete(dir: &Path) -> bool {
    dir.join(crate::store::STAGED_MARKER).is_file() && dir.join("TextVN.exe").is_file()
}

/// Dựng `dest` (thư mục một phiên bản) từ `src` theo kiểu nguyên tử (R2-37): đã đủ thì
/// giữ nguyên (bỏ qua copy — R2-05/R2-23); nếu không thì copy vào `<dest>.tmp`, ghi
/// marker, xoá `dest` dở dang cũ rồi rename. Lỗi ở bất kỳ bước nào → `dest` cũ (nếu
/// đã hoàn chỉnh) không bị đụng.
pub fn stage_version_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    if version_dir_complete(dest) {
        return Ok(());
    }
    let name = dest
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no dir name"))?;
    let mut tmp_name = name.to_os_string();
    tmp_name.push(".tmp");
    let tmp = dest.with_file_name(tmp_name);
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp)?;
    }
    stage_package_payload(src, &tmp)?;
    if !tmp.join("TextVN.exe").is_file() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "payload thiếu TextVN.exe",
        ));
    }
    std::fs::write(tmp.join(crate::store::STAGED_MARKER), b"1")?;
    if dest.exists() {
        std::fs::remove_dir_all(dest)?;
    }
    std::fs::rename(&tmp, dest)
}

// ─── Điều phối (Windows) ────────────────────────────────────────────────────────

/// Gọi ĐẦU TIÊN trong `main()`: `--msix-relay` / `--msix-install`, hoặc mọi lần chạy có
/// package identity → chạy bước tương ứng và trả exit code (caller thoát ngay, không
/// bao giờ tới tray). `None` = lần chạy thường (portable/Inno/Store đã stage).
#[cfg(windows)]
pub fn dispatch(args: &[String]) -> Option<i32> {
    match args.get(1).map(String::as_str) {
        Some(store::ARG_RELAY) => Some(run_relay(&args[2..])),
        Some(store::ARG_INSTALL) => Some(run_install(&args[2..])),
        _ if store_win::has_package_identity() => Some(run_packaged_entry()),
        _ => None,
    }
}

/// Đã cài đúng bản `ver` (stage.json khớp + thư mục đủ) → dùng luôn, khỏi copy.
/// Tiến trình trong package ĐỌC được file thật đã tồn tại dưới AppData (chỉ file MỚI
/// tạo mới bị ảo hoá).
#[cfg(windows)]
fn installed_dir_for(ver: &str) -> Option<std::path::PathBuf> {
    let root = store_win::store_root_from_env()?;
    let stage = store_win::read_stage(&root)?;
    let expected = store::version_dir(&root, ver);
    (stage.ver == ver
        && Path::new(&stage.dir)
            .to_string_lossy()
            .eq_ignore_ascii_case(&expected.to_string_lossy())
        && version_dir_complete(&expected))
    .then_some(expected)
}

/// Bước 1 — `TextVN.exe` chạy từ package (có identity).
#[cfg(windows)]
pub fn run_packaged_entry() -> i32 {
    let fail = |detail: String| -> i32 {
        store_win::report_error(&detail);
        1
    };
    let (Some(full), Some(pfn)) = (
        store_win::current_package_full_name(),
        store_win::current_package_family_name(),
    ) else {
        return fail("Không đọc được thông tin gói (package identity).".to_string());
    };
    let Some(ver) = store::version_from_package_full_name(&full) else {
        return fail(format!("Tên gói không đúng dạng: {full}"));
    };
    if !store::is_valid_pfn(&pfn) {
        return fail(format!("Package family name không đúng dạng: {pfn}"));
    }
    let Ok(exe) = std::env::current_exe() else {
        return fail("Không xác định được vị trí TextVN.exe.".to_string());
    };
    let Some(pkg_dir) = exe.parent() else {
        return fail("Không xác định được thư mục gói.".to_string());
    };
    store_win::store_log(&format!("packaged entry: {full}"));
    let src = match installed_dir_for(&ver) {
        Some(dir) => dir,
        None => {
            let Some(staging_root) = store_win::staging_root_from_env() else {
                return fail("Thiếu biến môi trường USERPROFILE.".to_string());
            };
            let dest = store::version_dir(&staging_root, &ver);
            if let Err(e) = stage_version_dir(pkg_dir, &dest) {
                return fail(format!(
                    "Không chép được tệp TextVN ra {}: {e}",
                    dest.display()
                ));
            }
            dest
        }
    };
    if let Err(e) =
        store_win::spawn_with_desktop_app_breakaway(&exe, &store::relay_args(&src, &pfn, &ver))
    {
        return fail(format!("Không khởi chạy được bước cài đặt: {e}"));
    }
    store_win::store_log(&format!("packaged entry: relay → {}", src.display()));
    0
}

/// Bước 2 — `--msix-relay <dir> <PFN> <V>` (vẫn trong package; con của nó ra ngoài).
#[cfg(windows)]
pub fn run_relay(rest: &[String]) -> i32 {
    let Some((src, pfn, ver)) = store::parse_relay_args(rest) else {
        store_win::store_log("relay: đối số sai");
        return 2;
    };
    let (Some(staging_root), Some(root)) = (
        store_win::staging_root_from_env(),
        store_win::store_root_from_env(),
    ) else {
        return 2;
    };
    if !store::source_dir_allowed(&src, &staging_root, &root) {
        store_win::store_log("relay: thư mục nguồn không thuộc staging/TextVN-Store");
        return 2;
    }
    let exe = src.join("TextVN.exe");
    if !exe.is_file() {
        store_win::report_error(&format!("Thiếu {}", exe.display()));
        return 1;
    }
    match store_win::spawn_detached(&exe, &store::install_args(&pfn, &ver)) {
        Ok(()) => 0,
        Err(e) => {
            store_win::report_error(&format!("Không khởi chạy được bước cài đặt: {e}"));
            1
        }
    }
}

/// Bước 3 — `--msix-install --pfn <PFN> --ver <V>` (phải KHÔNG có package identity).
#[cfg(windows)]
pub fn run_install(rest: &[String]) -> i32 {
    if store_win::has_package_identity() {
        store_win::report_error(
            "Bước cài đặt vẫn chạy trong môi trường của gói MSIX: mọi đăng ký bộ gõ sẽ bị \
Windows ảo hoá và không có tác dụng, nên TextVN dừng lại.",
        );
        return 1;
    }
    let Some((pfn, ver)) = store::parse_install_args(rest) else {
        store_win::store_log("install: đối số sai");
        return 2;
    };
    let (Some(staging_root), Some(root)) = (
        store_win::staging_root_from_env(),
        store_win::store_root_from_env(),
    ) else {
        return 2;
    };
    let Some(src) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    else {
        return 2;
    };
    if !store::source_dir_allowed(&src, &staging_root, &root) {
        store_win::store_log("install: exe không chạy từ staging/TextVN-Store");
        return 2;
    }
    let target = store::version_dir(&root, &ver);
    // R2-08/R2-24: không bao giờ cài đè lên thư mục của bộ cài Inno.
    if target.join("unins000.exe").exists() {
        store_win::report_error(&format!(
            "{} chứa trình gỡ của bộ cài TextVN — không cài đè.",
            target.display()
        ));
        return 1;
    }
    let old = store_win::read_stage(&root);
    let first_install = old.is_none();
    if !store_win::same_dir(&src, &target) {
        if let Err(e) = stage_version_dir(&src, &target) {
            store_win::report_error(&format!(
                "Không chép được TextVN vào {}: {e}",
                target.display()
            ));
            return 1;
        }
    }
    // Cài lại trước lần đăng nhập kế sau một lần dọn: bỏ lịch xoá còn treo.
    store_win::cancel_store_dir_removal();
    let info = StageInfo {
        pfn,
        ver: ver.clone(),
        dir: target.to_string_lossy().into_owned(),
    };
    if let Err(e) = store_win::write_stage(&root, &info) {
        store_win::report_error(&format!("Không ghi được stage.json: {e}"));
        return 1;
    }
    let new_exe = target.join("TextVN.exe");
    store_win::sync_store_run_value(&root, &new_exe, first_install);
    let version_changed = old.as_ref().is_none_or(|o| o.ver != ver);
    if version_changed {
        // Tray Store cũ (thư mục phiên bản khác) phải nhường chỗ cho bản mới (R2-05).
        store_win::stop_store_tray(&root);
    }
    let mode = store_win::take_relaunch_hint(&root).unwrap_or(LaunchMode::Normal);
    if let Some(args) = mode.tray_args() {
        if let Err(e) = store_win::spawn_detached(&new_exe, &args) {
            store_win::report_error(&format!("Không khởi chạy được TextVN: {e}"));
            return 1;
        }
    }
    store_win::store_log(&format!(
        "install: {ver} → {} (lần đầu: {first_install}, chạy tray: {mode:?})",
        target.display()
    ));
    store_win::prune_old_versions(&root, &ver);
    // Vùng staging: xoá mọi thư mục phiên bản (exe đang chạy của chính bước này còn
    // bị khoá — tray mới dọn nốt).
    store_win::prune_old_versions(&staging_root, "");
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("textvn-bootstrap-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn stage_payload_copy_du_file_va_dir() {
        let pkg = temp_dir("pkg");
        let target = temp_dir("target");
        std::fs::write(pkg.join("TextVN.exe"), b"exe").unwrap();
        std::fs::write(pkg.join("textvn-cli.exe"), b"cli").unwrap();
        std::fs::write(pkg.join("textvn-tsf.dll"), b"dll").unwrap();
        std::fs::write(pkg.join("textvn-tsf-x86.dll"), b"dll32").unwrap();
        std::fs::write(pkg.join("LICENSE"), b"gpl").unwrap();
        std::fs::create_dir_all(pkg.join("resources")).unwrap();
        std::fs::write(pkg.join("resources").join("textvn_v.ico"), b"ico").unwrap();
        std::fs::create_dir_all(pkg.join("data")).unwrap();
        std::fs::write(pkg.join("data").join("appdb.default.json"), b"{}").unwrap();
        std::fs::write(pkg.join("PRIVACY_POLICY.txt"), b"privacy").unwrap();
        // File của package (manifest, Assets) KHÔNG đi theo.
        std::fs::write(pkg.join("AppxManifest.xml"), b"<x/>").unwrap();

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
        assert_eq!(
            std::fs::read(target.join("textvn-tsf-x86.dll")).unwrap(),
            b"dll32",
            "DLL x86 (Zalo/Office 32-bit) phải được stage"
        );
        assert_eq!(std::fs::read(target.join("LICENSE")).unwrap(), b"gpl");
        assert!(!target.join("AppxManifest.xml").exists());
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

    /// R2-37/R2-05: thư mục phiên bản dựng nguyên tử — đủ rồi thì không copy lại; dở
    /// dang thì dựng lại sạch; nguồn thiếu exe thì lỗi và không để lại gì.
    #[test]
    fn stage_version_dir_is_atomic_and_idempotent() {
        let base = temp_dir("verdir");
        let pkg = base.join("pkg");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(pkg.join("TextVN.exe"), b"v1").unwrap();
        std::fs::write(pkg.join("textvn-tsf.dll"), b"v1").unwrap();
        let dest = base.join("1.2.27.0");

        stage_version_dir(&pkg, &dest).unwrap();
        assert!(version_dir_complete(&dest));
        assert!(!base.join("1.2.27.0.tmp").exists());

        // Đã đủ → bỏ qua copy (file trong gói đổi cũng không ghi đè bản đang chạy).
        std::fs::write(pkg.join("TextVN.exe"), b"v2").unwrap();
        stage_version_dir(&pkg, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("TextVN.exe")).unwrap(), b"v1");

        // Dở dang (thiếu marker, có rác) → dựng lại sạch.
        std::fs::remove_file(dest.join(crate::store::STAGED_MARKER)).unwrap();
        std::fs::write(dest.join("rac.txt"), b"x").unwrap();
        stage_version_dir(&pkg, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("TextVN.exe")).unwrap(), b"v2");
        assert!(!dest.join("rac.txt").exists());

        // Nguồn thiếu TextVN.exe → lỗi, không để lại thư mục tạm/đích.
        let empty = base.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        let dest2 = base.join("1.2.28.0");
        assert!(stage_version_dir(&empty, &dest2).is_err());
        assert!(!dest2.exists());
        assert!(!base.join("1.2.28.0.tmp").exists());

        let _ = std::fs::remove_dir_all(&base);
    }
}
