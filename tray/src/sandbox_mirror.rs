// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-26 — tray ghi **bản sao cho tiến trình sandbox** (AppContainer/LPAC: ô tìm kiếm Start,
//! Calculator, ứng dụng Store) vào `%APPDATA%\TextVN\sandbox\`.
//!
//! Tiến trình sandbox nạp được DLL TSF nhưng bị từ chối `config.json`/`state.json` (đo thật:
//! lỗi 5) nên trước bản này luôn gõ Telex mặc định, bỏ qua cấu hình và nút V/E. Thư mục bản sao
//! có DACL **bảo vệ**: user/SYSTEM/Admins toàn quyền, `ALL APPLICATION PACKAGES` +
//! `ALL RESTRICTED APPLICATION PACKAGES` chỉ đọc (`textvn_ipc::sandbox::sandbox_dir_sddl`).
//! Vì MỌI ứng dụng Store của người dùng đọc được thư mục này, `config.json` ở đây đã lọc sạch
//! nội dung người dùng (`textvn_config::sandbox::sandbox_config_json`: không gõ tắt, emoji
//! riêng, từ tiếng Anh tự thêm, từ đã học). `state.json` chỉ có công tắc V/E, trợ lý và
//! override theo tên app — chép nguyên.
//!
//! Fail-open: ghi lỗi chỉ làm tiến trình sandbox dùng mặc định như trước, không bao giờ chặn
//! việc lưu cấu hình thật.

use std::path::{Path, PathBuf};

/// Thư mục bản sao trong thư mục cấu hình (`%APPDATA%\TextVN\sandbox`).
pub fn mirror_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(textvn_ipc::sandbox::SANDBOX_DIR_NAME)
}

/// Ghi bản sao. `config_raw` = nội dung `config.json` vừa lưu (lọc trước khi ghi; không phải
/// JSON object → bỏ qua phần config); `state_json` = `state.json` vừa lưu.
pub fn write_mirror(
    config_dir: &Path,
    config_raw: Option<&str>,
    state_json: Option<&str>,
) -> std::io::Result<()> {
    let dir = mirror_dir(config_dir);
    ensure_dir(&dir)?;
    if let Some(raw) = config_raw {
        if let Some(clean) = textvn_config::sandbox::sandbox_config_json(raw) {
            write_if_changed(&dir.join("config.json"), clean.as_bytes())?;
        }
    }
    if let Some(state) = state_json {
        write_if_changed(&dir.join("state.json"), state.as_bytes())?;
    }
    Ok(())
}

/// Bỏ qua khi nội dung không đổi: tray gộp dữ liệu tự học ≤ 1 lần/phút (bản sao không chứa
/// phần đó) — không ghi lại tệp giống hệt mỗi lần.
fn write_if_changed(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|old| old == content) {
        return Ok(());
    }
    textvn_config::doc::atomic_write(path, content)
}

/// Tạo thư mục và (Windows) đặt DACL bảo vệ cho sandbox đọc. Đặt lại mỗi lần tray khởi động
/// lần đầu ghi (hồ sơ roaming chép sang máy khác có thể mất ACL) và mỗi khi thư mục vừa được
/// tạo lại; giữa chừng chỉ ghi tệp (tệp mới kế thừa ACE `OICI`).
fn ensure_dir(dir: &Path) -> std::io::Result<()> {
    let existed = dir.is_dir();
    std::fs::create_dir_all(dir)?;
    #[cfg(windows)]
    {
        use std::sync::Mutex;
        static APPLIED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
        let mut applied = APPLIED.lock().unwrap_or_else(|e| e.into_inner());
        let known = applied.iter().any(|p| p == dir);
        if !existed || !known {
            win::apply_sandbox_dacl(dir)?;
            if !known {
                applied.push(dir.to_path_buf());
            }
        }
    }
    #[cfg(not(windows))]
    let _ = existed;
    Ok(())
}

#[cfg(windows)]
pub mod win {
    use std::path::Path;

    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
        SDDL_REVISION_1, SE_FILE_OBJECT,
    };
    use windows::Win32::Security::{
        GetSecurityDescriptorDacl, ACL, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    };

    fn io_err(code: u32) -> std::io::Error {
        std::io::Error::from_raw_os_error(code as i32)
    }

    /// Đặt DACL bảo vệ `sandbox_dir_sddl(user)` lên thư mục (ACE kế thừa xuống tệp có sẵn).
    pub fn apply_sandbox_dacl(dir: &Path) -> std::io::Result<()> {
        let user = textvn_ipc::sandbox::current_user_sid_string()
            .ok_or_else(|| std::io::Error::other("không đọc được SID người dùng"))?;
        let sddl = HSTRING::from(textvn_ipc::sandbox::sandbox_dir_sddl(&user));
        let path: Vec<u16> = dir
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: SD do API cấp, giải phóng đúng một lần; DACL trỏ vào SD còn sống tới hết lời
        // gọi SetNamedSecurityInfoW; chuỗi đường dẫn kết thúc NUL.
        unsafe {
            let mut sd = PSECURITY_DESCRIPTOR::default();
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                &sddl,
                SDDL_REVISION_1,
                &mut sd,
                None,
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?;
            let mut present = windows::core::BOOL(0);
            let mut defaulted = windows::core::BOOL(0);
            let mut dacl: *mut ACL = std::ptr::null_mut();
            let got = GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted);
            let result = if got.is_ok() && present.as_bool() {
                let status = SetNamedSecurityInfoW(
                    PCWSTR(path.as_ptr()),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    None,
                    None,
                    Some(dacl),
                    None,
                );
                if status == ERROR_SUCCESS {
                    Ok(())
                } else {
                    Err(io_err(status.0))
                }
            } else {
                Err(std::io::Error::other("SDDL không có DACL"))
            };
            let _ = LocalFree(Some(HLOCAL(sd.0)));
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("textvn-sbx-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn mirror_writes_clean_config_and_state() {
        let dir = temp_dir("write");
        let cfg = r#"{"config_version":1,"method":"vni","macros":[{"trigger":"dc","expand":"12 Lê Lợi"}],
            "english_words":["Khanhvy"],"style_profile":{"enabled":true,"words":{"bímật":2}}}"#;
        let state = r#"{"global_enabled":false,"apps":{"zalo.exe":true},"assist_enabled":true}"#;
        write_mirror(&dir, Some(cfg), Some(state)).unwrap();
        let out = std::fs::read_to_string(mirror_dir(&dir).join("config.json")).unwrap();
        assert!(out.contains("\"vni\""));
        for leaked in ["Lê Lợi", "Khanhvy", "bímật"] {
            assert!(!out.contains(leaked), "lọt `{leaked}`");
        }
        assert_eq!(
            std::fs::read_to_string(mirror_dir(&dir).join("state.json")).unwrap(),
            state
        );
        // config hỏng: không ghi đè bản sao config cũ bằng rác, state vẫn ghi.
        write_mirror(&dir, Some("{ hỏng"), Some("{\"global_enabled\":true}")).unwrap();
        assert!(
            std::fs::read_to_string(mirror_dir(&dir).join("config.json"))
                .unwrap()
                .contains("\"vni\"")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn mirror_dir_dacl_is_protected_and_grants_sandbox_read_only() {
        let dir = temp_dir("acl");
        write_mirror(&dir, Some("{}"), Some("{}")).unwrap();
        let sddl = textvn_ipc::sandbox::path_dacl_sddl(&mirror_dir(&dir)).expect("đọc DACL");
        assert!(sddl.starts_with("D:P"), "DACL phải bảo vệ: {sddl}");
        assert!(
            textvn_ipc::sandbox::sddl_grants_sandbox(&sddl),
            "thiếu ACE AppContainer/LPAC: {sddl}"
        );
        let file = textvn_ipc::sandbox::path_dacl_sddl(&mirror_dir(&dir).join("config.json"))
            .expect("DACL tệp");
        assert!(
            textvn_ipc::sandbox::sddl_grants_sandbox(&file),
            "tệp phải kế thừa ACE sandbox: {file}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
