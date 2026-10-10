// SPDX-License-Identifier: GPL-3.0-or-later
//! Kênh Microsoft Store (MSIX) — phần THUẦN: đường dẫn, `stage.json`, so version,
//! quyết định guard/autostart, dựng dòng lệnh. Kiểm thử được trên mọi OS; phần gọi
//! Windows API nằm ở `store_win.rs`, luồng bootstrap ở `package_bootstrap.rs`.
//!
//! Vì sao cần cả một kênh riêng (R2-02/R2-19/R2-22): tiến trình full-trust CÓ package
//! identity bị Windows ảo hoá mọi file MỚI tạo dưới `%LOCALAPPDATA%`/`%APPDATA%` và mọi
//! ghi HKCU (hive riêng của package, xoá khi gỡ) — đăng ký TSF/Run key từ đó vô hình
//! với Notepad/Explorer. Luồng đã chốt (chủ repo 2026-10-08):
//!
//! 1. `TextVN.exe` trong package (có identity) KHÔNG đăng ký gì: copy payload ra
//!    `%USERPROFILE%\.textvn\msix-staging\<V>\` (ngoài AppData → không ảo hoá), rồi
//!    chạy chính nó `--msix-relay` với PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY =
//!    BREAKAWAY_ENABLE_PROCESS_TREE và thoát.
//! 2. `--msix-relay` (vẫn trong package, nhưng CON của nó chạy ngoài container): chạy
//!    `<staging>\TextVN.exe --msix-install --pfn <PFN> --ver <V>` rồi thoát.
//! 3. `--msix-install` (không identity): copy vào `%LOCALAPPDATA%\Programs\TextVN-Store\<V>\`
//!    (KHÔNG phải `Programs\TextVN` của bộ cài Inno per-user — R2-08/R2-24), ghi
//!    `TextVN-Store\stage.json`, dừng tray Store cũ, chạy tray mới, dọn bản cũ.
//! 4. Tray chạy dưới `TextVN-Store` + có `stage.json` = kênh Store: mỗi lần khởi động
//!    (kể cả `--autostart`, `--msix-guard`) kiểm package còn không / có bản mới hơn
//!    không (R2-04/R2-05/R2-23); không bao giờ xin UAC (R2-07); hỏi trước khi đổi
//!    Ctrl+Shift của Windows (R2-06).

use std::path::{Path, PathBuf};

/// Thư mục cài kênh Store dưới `%LOCALAPPDATA%\Programs` (khác `TextVN` của Inno).
pub const STORE_DIR_NAME: &str = "TextVN-Store";
/// `TextVN-Store\stage.json` — `{"pfn","ver","dir"}` của bản Store đang dùng.
pub const STAGE_FILE: &str = "stage.json";
/// File đánh dấu một thư mục phiên bản đã copy ĐỦ (staging hoặc TextVN-Store\<V>).
pub const STAGED_MARKER: &str = ".textvn-staged";
/// Gợi ý cách chạy tray sau khi package cài lại bản mới (guard ghi, installer đọc).
pub const RELAUNCH_HINT_FILE: &str = "relaunch-mode";
/// `Application Id` trong AppxManifest — dựng AUMID `<PFN>!TextVN`.
pub const APP_ID: &str = "TextVN";
/// Tên các value HKCU RunOnce hẹn xoá thư mục kênh Store ở lần đăng nhập sau, theo
/// thứ tự lệnh của [`runonce_cleanup_command`] (mục đầu giữ tên cũ để huỷ được lịch
/// của bản trước).
pub const RUNONCE_VALUE_NAMES: [&str; 3] = [
    "TextVN-StoreCleanup",
    "TextVN-StoreCleanup2",
    "TextVN-StoreCleanup3",
];

pub const ARG_RELAY: &str = "--msix-relay";
pub const ARG_INSTALL: &str = "--msix-install";
pub const ARG_GUARD: &str = "--msix-guard";
pub const ARG_AUTOSTART: &str = "--autostart";

/// `%LOCALAPPDATA%\Programs\TextVN-Store`.
pub fn store_root(local_app_data: &Path) -> PathBuf {
    local_app_data.join("Programs").join(STORE_DIR_NAME)
}

/// `%USERPROFILE%\.textvn\msix-staging` — vùng trung chuyển NGOÀI AppData: tiến trình
/// có package identity ghi ở đây là ghi thật (flexible-virtualization: "apart from
/// AppData, the app can write to any location where the user has write access").
pub fn staging_root(user_profile: &Path) -> PathBuf {
    user_profile.join(".textvn").join("msix-staging")
}

/// Thư mục một phiên bản (`<root>\<V>`).
pub fn version_dir(root: &Path, ver: &str) -> PathBuf {
    root.join(ver)
}

/// `A.B.C.D` (4 phần, mỗi phần u16) → mảng so sánh được theo thứ tự từ điển.
pub fn parse_version_quad(s: &str) -> Option<[u16; 4]> {
    let mut out = [0u16; 4];
    let mut parts = s.split('.');
    for slot in &mut out {
        let p = parts.next()?;
        if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = p.parse().ok()?;
    }
    parts.next().is_none().then_some(out)
}

/// Package family name `<Name>_<PublisherId>`: Name 3–50 ký tự `[A-Za-z0-9.-]`,
/// PublisherId 13 ký tự `[a-z0-9]`. Chặn mọi ký tự lạ trước khi đưa vào dòng lệnh
/// hay `shell:AppsFolder\<PFN>!TextVN`.
pub fn is_valid_pfn(pfn: &str) -> bool {
    let Some((name, publisher)) = pfn.rsplit_once('_') else {
        return false;
    };
    (3..=50).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        && publisher.len() == 13
        && publisher
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// Version trong package full name `Name_Version_Arch_ResourceId_PublisherId`.
pub fn version_from_package_full_name(full_name: &str) -> Option<String> {
    let mut parts = full_name.split('_');
    let _name = parts.next()?;
    let ver = parts.next()?;
    parse_version_quad(ver).map(|_| ver.to_string())
}

/// Nội dung `TextVN-Store\stage.json`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StageInfo {
    pub pfn: String,
    pub ver: String,
    pub dir: String,
}

impl StageInfo {
    /// Đọc + kiểm tra hợp lệ (PFN/version đúng dạng, `dir` không rỗng). Sai → `None`
    /// (guard fail-open: không dọn, không chạy lại package dựa trên dữ liệu hỏng).
    pub fn parse(json: &str) -> Option<Self> {
        let info: Self = serde_json::from_str(json).ok()?;
        (is_valid_pfn(&info.pfn)
            && parse_version_quad(&info.ver).is_some()
            && !info.dir.trim().is_empty())
        .then_some(info)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Guard kênh Store quyết định gì ở lần khởi động này.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardAction {
    /// Package còn và không mới hơn (hoặc API lỗi → fail-open).
    Continue,
    /// Package đã bị gỡ: dọn đăng ký/Run/Ctrl+Shift, hẹn xoá thư mục rồi thoát.
    Cleanup,
    /// Package đã có bản mới hơn `stage.json`: chạy app trong package (nó stage lại).
    LaunchPackage,
}

/// `installed` = full name các package cùng family đang cài cho user (`None` = API lỗi).
pub fn guard_decision(staged_ver: &str, installed: Option<&[String]>) -> GuardAction {
    let Some(names) = installed else {
        return GuardAction::Continue;
    };
    if names.is_empty() {
        return GuardAction::Cleanup;
    }
    let newest = names
        .iter()
        .filter_map(|n| version_from_package_full_name(n))
        .filter_map(|v| parse_version_quad(&v))
        .max();
    match (newest, parse_version_quad(staged_ver)) {
        (Some(n), Some(s)) if n > s => GuardAction::LaunchPackage,
        _ => GuardAction::Continue,
    }
}

/// `shell:AppsFolder\<PFN>!TextVN` — kích hoạt app trong package qua Explorer.
pub fn apps_folder_target(pfn: &str) -> String {
    format!(r"shell:AppsFolder\{pfn}!{APP_ID}")
}

/// Trích một đối số theo quy tắc `CommandLineToArgvW`/MSVC (dấu cách, tab, ngoặc kép;
/// gấp đôi `\` đứng trước `"` và ở cuối chuỗi khi bọc ngoặc).
pub fn quote_windows_arg(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '\n', '\u{b}', '"']) {
        return arg.to_string();
    }
    let mut out = String::with_capacity(arg.len() + 2);
    out.push('"');
    let mut backslashes = 0usize;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                out.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                out.push('"');
                backslashes = 0;
            }
            _ => {
                out.extend(std::iter::repeat_n('\\', backslashes));
                out.push(c);
                backslashes = 0;
            }
        }
    }
    out.extend(std::iter::repeat_n('\\', backslashes * 2));
    out.push('"');
    out
}

/// Dòng lệnh đầy đủ cho `CreateProcessW` (exe + đối số đã trích).
pub fn build_command_line(exe: &str, args: &[String]) -> String {
    std::iter::once(quote_windows_arg(exe))
        .chain(args.iter().map(|a| quote_windows_arg(a)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Đối số `--msix-relay <dir> <PFN> <V>` (sau tên lệnh).
pub fn relay_args(src_dir: &Path, pfn: &str, ver: &str) -> Vec<String> {
    vec![
        ARG_RELAY.to_string(),
        src_dir.to_string_lossy().into_owned(),
        pfn.to_string(),
        ver.to_string(),
    ]
}

/// Đọc `<dir> <PFN> <V>` của `--msix-relay` (đúng 3 đối số, PFN/V hợp lệ).
pub fn parse_relay_args(rest: &[String]) -> Option<(PathBuf, String, String)> {
    match rest {
        [dir, pfn, ver]
            if !dir.trim().is_empty() && is_valid_pfn(pfn) && parse_version_quad(ver).is_some() =>
        {
            Some((PathBuf::from(dir), pfn.clone(), ver.clone()))
        }
        _ => None,
    }
}

/// Đối số `--msix-install --pfn <PFN> --ver <V>` (sau tên lệnh).
pub fn install_args(pfn: &str, ver: &str) -> Vec<String> {
    vec![
        ARG_INSTALL.to_string(),
        "--pfn".to_string(),
        pfn.to_string(),
        "--ver".to_string(),
        ver.to_string(),
    ]
}

/// Đọc `--pfn <PFN> --ver <V>` (thứ tự tuỳ ý; thiếu/lạ/sai dạng → `None`).
pub fn parse_install_args(rest: &[String]) -> Option<(String, String)> {
    let (mut pfn, mut ver) = (None, None);
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        let value = it.next()?;
        match flag.as_str() {
            "--pfn" if pfn.is_none() => pfn = Some(value.clone()),
            "--ver" if ver.is_none() => ver = Some(value.clone()),
            _ => return None,
        }
    }
    let (pfn, ver) = (pfn?, ver?);
    (is_valid_pfn(&pfn) && parse_version_quad(&ver).is_some()).then_some((pfn, ver))
}

/// Nguồn của relay/installer phải là một thư mục phiên bản dưới vùng staging hoặc
/// dưới `TextVN-Store` — relay không phải trình chạy exe tuỳ ý.
pub fn source_dir_allowed(dir: &Path, staging_root: &Path, store_root: &Path) -> bool {
    crate::path_is_under_any(
        &dir.to_string_lossy(),
        &[
            staging_root.to_string_lossy().into_owned(),
            store_root.to_string_lossy().into_owned(),
        ],
    )
}

/// Cách chạy tray sau khi cài (R2-05/R2-23: cập nhật lúc đăng nhập không bật Bảng
/// điều khiển; người dùng đã tắt tự khởi động thì chỉ cài, không chạy tray).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// Người dùng mở app (Start): chạy tray bình thường.
    Normal,
    /// Từ Run `--autostart`: chạy tray ngầm.
    Autostart,
    /// Từ Run `--msix-guard` (tự khởi động đang tắt): chỉ cài, không chạy tray.
    GuardOnly,
}

impl LaunchMode {
    /// Đối số cho tray mới; `None` = không chạy tray.
    pub fn tray_args(self) -> Option<Vec<String>> {
        match self {
            Self::Normal => Some(Vec::new()),
            Self::Autostart => Some(vec![ARG_AUTOSTART.to_string()]),
            Self::GuardOnly => None,
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Autostart => ARG_AUTOSTART,
            Self::GuardOnly => ARG_GUARD,
        }
    }

    pub fn from_hint(content: &str) -> Option<Self> {
        match content.trim() {
            "normal" => Some(Self::Normal),
            ARG_AUTOSTART => Some(Self::Autostart),
            ARG_GUARD => Some(Self::GuardOnly),
            _ => None,
        }
    }
}

/// Giá trị HKCU Run `TextVN` của kênh Store: `--autostart` khi bật tự khởi động,
/// `--msix-guard` khi tắt — guard gỡ/cập nhật vẫn chạy mỗi lần đăng nhập.
pub fn run_command(exe: &Path, autostart: bool) -> String {
    format!(
        "\"{}\" {}",
        exe.display(),
        if autostart { ARG_AUTOSTART } else { ARG_GUARD }
    )
}

/// Value Run chỉ chạy guard (tự khởi động đang TẮT theo nghĩa người dùng).
pub fn run_command_is_guard(cmd: &str) -> bool {
    cmd.split_whitespace().any(|a| a == ARG_GUARD)
}

/// Value Run cần ghi cho kênh Store (`None` = giữ nguyên):
/// - chưa có: lần cài đầu → `--autostart` (giống task `autostart` của Inno), sau đó
///   → `--msix-guard` (guard vẫn phải chạy);
/// - đang trỏ vào `TextVN-Store`: giữ chế độ, cập nhật đường dẫn sang `exe` (bản mới);
/// - trỏ chỗ khác (bản cài/portable): không ghi đè mục của kênh khác.
pub fn decide_store_run_value(
    existing: Option<&str>,
    store_root: &Path,
    exe: &Path,
    first_install: bool,
) -> Option<String> {
    match existing.map(str::trim).filter(|c| !c.is_empty()) {
        None => Some(run_command(exe, first_install)),
        Some(cmd) if crate::autostart::command_points_into(cmd, store_root) => {
            let want = run_command(exe, !run_command_is_guard(cmd));
            (want != cmd).then_some(want)
        }
        Some(_) => None,
    }
}

/// Thư mục con của `TextVN-Store`/staging được phép xoá khi dọn bản cũ: đúng dạng
/// version (`A.B.C.D`) hoặc `A.B.C.D.tmp`, khác `keep`. Không bao giờ đụng tên khác.
pub fn prunable_version_dir(name: &str, keep: &str) -> bool {
    if name.eq_ignore_ascii_case(keep) {
        return false;
    }
    parse_version_quad(name).is_some()
        || name
            .strip_suffix(".tmp")
            .is_some_and(|v| parse_version_quad(v).is_some())
}

fn win_norm(p: &str) -> String {
    p.replace('/', "\\").trim_end_matches('\\').to_string()
}

/// Các lệnh HKCU RunOnce xoá ĐÚNG hai thư mục của kênh Store ở lần đăng nhập sau (lúc đó
/// không tiến trình nào còn nạp DLL TextVN): `TextVN-Store` và `.textvn\msix-staging`,
/// rồi `rmdir` (không /s — chỉ xoá khi rỗng) thư mục `.textvn`. `None` khi đường dẫn
/// không đúng dạng mong đợi hoặc chứa ký tự cmd diễn giải (`"`, `%`, xuống dòng).
pub fn runonce_cleanup_command(
    cmd_exe: &str,
    store_root: &Path,
    staging_root: &Path,
) -> Option<Vec<String>> {
    let store = win_norm(&store_root.to_string_lossy());
    let staging = win_norm(&staging_root.to_string_lossy());
    let absolute = |p: &str| {
        let b = p.as_bytes();
        (b.len() > 3 && b[0].is_ascii_alphabetic() && &b[1..3] == b":\\") || p.starts_with(r"\\")
    };
    let safe = |p: &str| absolute(p) && !p.contains(['"', '%', '\r', '\n']);
    let store_tail = format!(r"\programs\{}", STORE_DIR_NAME.to_lowercase());
    if !safe(&store)
        || !safe(&staging)
        || cmd_exe.contains(['"', '%', '\r', '\n'])
        || !store.to_lowercase().ends_with(&store_tail)
        || !staging.to_lowercase().ends_with(r"\.textvn\msix-staging")
    {
        return None;
    }
    let dot_textvn = &staging[..staging.len() - r"\msix-staging".len()];
    // R2-94: mỗi thư mục một mục RunOnce — Windows khuyến nghị lệnh RunOnce ≤ 260 ký
    // tự; ba đường dẫn trong một lệnh vượt ngưỡng khi tên tài khoản dài.
    Some(vec![
        format!("\"{cmd_exe}\" /d /c rmdir /s /q \"{store}\""),
        format!("\"{cmd_exe}\" /d /c rmdir /s /q \"{staging}\""),
        format!("\"{cmd_exe}\" /d /c rmdir \"{dot_textvn}\""),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const PFN: &str = "LinhBH.CoM.TextVN_q1w2e3r4t5y6u";

    #[test]
    fn version_quad_parse_and_order() {
        assert_eq!(parse_version_quad("1.2.27.0"), Some([1, 2, 27, 0]));
        assert_eq!(parse_version_quad("65535.0.0.0"), Some([65535, 0, 0, 0]));
        for bad in [
            "",
            "1.2.27",
            "1.2.27.0.1",
            "1..2.3",
            "1.2.3.x",
            "65536.0.0.0",
            "+1.2.3.4",
        ] {
            assert_eq!(parse_version_quad(bad), None, "{bad}");
        }
        assert!(parse_version_quad("1.2.28.0") > parse_version_quad("1.2.27.0"));
        assert!(parse_version_quad("1.10.0.0") > parse_version_quad("1.9.99.0"));
    }

    #[test]
    fn pfn_validation_rejects_injection() {
        assert!(is_valid_pfn(PFN));
        assert!(is_valid_pfn("12345LinhBH.TextVN_abcdefghjkmn0"));
        for bad in [
            "",
            "TextVN",
            "TextVN_short",
            "Text VN_q1w2e3r4t5y6u",
            "TextVN_Q1W2E3R4T5Y6U",
            r"..\x_q1w2e3r4t5y6u",
            "TextVN_q1w2e3r4t5y6u\" & calc",
            "ab_q1w2e3r4t5y6u",
        ] {
            assert!(!is_valid_pfn(bad), "{bad}");
        }
    }

    #[test]
    fn version_from_full_name() {
        assert_eq!(
            version_from_package_full_name("LinhBH.CoM.TextVN_1.2.27.0_x64__q1w2e3r4t5y6u")
                .as_deref(),
            Some("1.2.27.0")
        );
        assert_eq!(version_from_package_full_name("TextVN"), None);
        assert_eq!(version_from_package_full_name("TextVN_abc_x64__p"), None);
    }

    #[test]
    fn stage_json_roundtrip_and_validation() {
        let info = StageInfo {
            pfn: PFN.to_string(),
            ver: "1.2.27.0".to_string(),
            dir: r"C:\Users\a\AppData\Local\Programs\TextVN-Store\1.2.27.0".to_string(),
        };
        let json = info.to_json();
        assert_eq!(StageInfo::parse(&json), Some(info.clone()));
        let bad_ver = json.replace("1.2.27.0\"", "1.2.27\"");
        assert_eq!(StageInfo::parse(&bad_ver), None);
        assert_eq!(StageInfo::parse(&json.replace(PFN, "x y")), None);
        assert_eq!(StageInfo::parse("{"), None);
        assert_eq!(
            StageInfo::parse(&format!(r#"{{"pfn":"{PFN}","ver":"1.2.27.0","dir":" "}}"#)),
            None
        );
    }

    #[test]
    fn guard_decision_matrix() {
        let full = |v: &str| format!("LinhBH.CoM.TextVN_{v}_x64__q1w2e3r4t5y6u");
        // API lỗi → không làm gì (fail-open, S4).
        assert_eq!(guard_decision("1.2.27.0", None), GuardAction::Continue);
        // Package đã gỡ.
        assert_eq!(guard_decision("1.2.27.0", Some(&[])), GuardAction::Cleanup);
        // Cùng version / cũ hơn → tiếp tục.
        assert_eq!(
            guard_decision("1.2.27.0", Some(&[full("1.2.27.0")])),
            GuardAction::Continue
        );
        assert_eq!(
            guard_decision("1.2.28.0", Some(&[full("1.2.27.0")])),
            GuardAction::Continue
        );
        // Store đã cập nhật → chạy package để stage lại.
        assert_eq!(
            guard_decision("1.2.27.0", Some(&[full("1.2.27.0"), full("1.2.28.0")])),
            GuardAction::LaunchPackage
        );
        // Dữ liệu lạ → không đoán.
        assert_eq!(
            guard_decision("hong", Some(&[full("1.2.28.0")])),
            GuardAction::Continue
        );
        assert_eq!(
            guard_decision("1.2.27.0", Some(&["la".to_string()])),
            GuardAction::Continue
        );
    }

    #[test]
    fn windows_arg_quoting_matches_msvc_rules() {
        assert_eq!(quote_windows_arg("abc"), "abc");
        assert_eq!(quote_windows_arg(""), "\"\"");
        assert_eq!(
            quote_windows_arg(r"C:\Users\John Doe\.textvn"),
            r#""C:\Users\John Doe\.textvn""#
        );
        // `\` cuối phải gấp đôi khi bọc ngoặc, nếu không sẽ nuốt ngoặc đóng.
        assert_eq!(quote_windows_arg(r"C:\a b\"), r#""C:\a b\\""#);
        assert_eq!(quote_windows_arg(r#"a"b"#), r#""a\"b""#);
        assert_eq!(quote_windows_arg(r#"a\"b"#), r#""a\\\"b""#);
        assert_eq!(
            build_command_line(
                r"C:\Program Files\WindowsApps\P\TextVN.exe",
                &relay_args(
                    Path::new(r"C:\Users\J D\.textvn\msix-staging\1.2.27.0"),
                    PFN,
                    "1.2.27.0"
                )
            ),
            format!(
                r#""C:\Program Files\WindowsApps\P\TextVN.exe" --msix-relay "C:\Users\J D\.textvn\msix-staging\1.2.27.0" {PFN} 1.2.27.0"#
            )
        );
    }

    #[test]
    fn relay_and_install_args_roundtrip() {
        let staging = Path::new(r"C:\Users\a\.textvn\msix-staging\1.2.27.0");
        let args = relay_args(staging, PFN, "1.2.27.0");
        assert_eq!(args[0], ARG_RELAY);
        let (dir, pfn, ver) = parse_relay_args(&args[1..]).unwrap();
        assert_eq!(
            (dir.as_path(), pfn.as_str(), ver.as_str()),
            (staging, PFN, "1.2.27.0")
        );
        assert!(parse_relay_args(&args[1..3]).is_none());
        assert!(parse_relay_args(&[String::new(), PFN.into(), "1.2.27.0".into()]).is_none());
        assert!(parse_relay_args(&["d".into(), "x".into(), "1.2.27.0".into()]).is_none());

        let args = install_args(PFN, "1.2.27.0");
        assert_eq!(args[0], ARG_INSTALL);
        assert_eq!(
            parse_install_args(&args[1..]),
            Some((PFN.to_string(), "1.2.27.0".to_string()))
        );
        let swapped: Vec<String> = ["--ver", "1.2.27.0", "--pfn", PFN]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(parse_install_args(&swapped).is_some());
        for bad in [
            vec!["--pfn", PFN],
            vec!["--pfn", PFN, "--ver"],
            vec!["--pfn", PFN, "--ver", "1.2"],
            vec!["--pfn", PFN, "--ver", "1.2.27.0", "--x", "y"],
            vec!["--pfn", PFN, "--pfn", PFN, "--ver", "1.2.27.0"],
        ] {
            let bad: Vec<String> = bad.iter().map(|s| s.to_string()).collect();
            assert!(parse_install_args(&bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn source_dir_must_be_staging_or_store_version_dir() {
        let staging = Path::new(r"C:\Users\a\.textvn\msix-staging");
        let store = Path::new(r"C:\Users\a\AppData\Local\Programs\TextVN-Store");
        assert!(source_dir_allowed(
            Path::new(r"C:\Users\a\.textvn\msix-staging\1.2.27.0"),
            staging,
            store
        ));
        assert!(source_dir_allowed(
            Path::new(r"c:\users\a\appdata\local\programs\textvn-store\1.2.27.0"),
            staging,
            store
        ));
        for bad in [
            r"C:\Users\a\Downloads\TextVN",
            r"C:\Users\a\AppData\Local\Programs\TextVN",
            r"C:\Users\a\.textvn\msix-staging",
            r"C:\Users\a\.textvn\msix-staging-evil\1",
        ] {
            assert!(!source_dir_allowed(Path::new(bad), staging, store), "{bad}");
        }
    }

    #[test]
    fn store_paths_are_channel_specific() {
        let local = Path::new("L");
        assert_eq!(
            store_root(local),
            local.join("Programs").join("TextVN-Store")
        );
        // R2-08/R2-24: không bao giờ trùng thư mục bộ cài Inno per-user.
        assert_ne!(store_root(local), local.join("Programs").join("TextVN"));
        let profile = Path::new("U");
        assert_eq!(
            staging_root(profile),
            profile.join(".textvn").join("msix-staging")
        );
        assert_eq!(
            version_dir(&store_root(local), "1.2.27.0"),
            store_root(local).join("1.2.27.0")
        );
    }

    #[test]
    fn launch_mode_hint_roundtrip() {
        for mode in [
            LaunchMode::Normal,
            LaunchMode::Autostart,
            LaunchMode::GuardOnly,
        ] {
            assert_eq!(LaunchMode::from_hint(mode.hint()), Some(mode));
        }
        assert_eq!(
            LaunchMode::from_hint(" --autostart\r\n"),
            Some(LaunchMode::Autostart)
        );
        assert_eq!(LaunchMode::from_hint("calc.exe"), None);
        assert_eq!(LaunchMode::Normal.tray_args(), Some(vec![]));
        assert_eq!(
            LaunchMode::Autostart.tray_args(),
            Some(vec!["--autostart".to_string()])
        );
        assert_eq!(LaunchMode::GuardOnly.tray_args(), None);
    }

    #[test]
    fn store_run_value_decisions() {
        let root = Path::new(r"C:\U\AppData\Local\Programs\TextVN-Store");
        let exe = Path::new(r"C:\U\AppData\Local\Programs\TextVN-Store\1.2.28.0\TextVN.exe");
        let old_auto =
            r#""C:\U\AppData\Local\Programs\TextVN-Store\1.2.27.0\TextVN.exe" --autostart"#;
        let old_guard =
            r#""C:\U\AppData\Local\Programs\TextVN-Store\1.2.27.0\TextVN.exe" --msix-guard"#;
        let want_auto = format!("\"{}\" --autostart", exe.display());
        let want_guard = format!("\"{}\" --msix-guard", exe.display());

        // Lần cài đầu: bật tự khởi động mặc định.
        assert_eq!(
            decide_store_run_value(None, root, exe, true),
            Some(want_auto.clone())
        );
        // Không phải lần đầu mà mất value: vẫn phải có guard.
        assert_eq!(
            decide_store_run_value(None, root, exe, false),
            Some(want_guard.clone())
        );
        assert_eq!(
            decide_store_run_value(Some("  "), root, exe, false),
            Some(want_guard.clone())
        );
        // Cập nhật: giữ chế độ, đổi đường dẫn sang bản mới.
        assert_eq!(
            decide_store_run_value(Some(old_auto), root, exe, false),
            Some(want_auto.clone())
        );
        assert_eq!(
            decide_store_run_value(Some(old_guard), root, exe, true),
            Some(want_guard.clone())
        );
        // Đã đúng → không ghi lại.
        assert_eq!(
            decide_store_run_value(Some(&want_auto), root, exe, false),
            None
        );
        // Mục của bản cài/portable khác → không ghi đè.
        assert_eq!(
            decide_store_run_value(
                Some(r#""C:\U\AppData\Local\Programs\TextVN\TextVN.exe" --autostart"#),
                root,
                exe,
                true
            ),
            None
        );
        assert!(run_command_is_guard(&want_guard));
        assert!(!run_command_is_guard(&want_auto));
        assert!(!run_command_is_guard(
            r#""C:\a\--msix-guard-x\TextVN.exe" --autostart"#
        ));
    }

    #[test]
    fn prune_only_version_shaped_dirs() {
        assert!(prunable_version_dir("1.2.26.0", "1.2.27.0"));
        assert!(prunable_version_dir("1.2.27.0.tmp", "1.2.27.0"));
        assert!(!prunable_version_dir("1.2.27.0", "1.2.27.0"));
        for keep_out in [
            "stage.json",
            "relaunch-mode",
            "Documents",
            "1.2.x.0",
            "..",
            "",
        ] {
            assert!(!prunable_version_dir(keep_out, "1.2.27.0"), "{keep_out}");
        }
    }

    #[test]
    fn runonce_cleanup_targets_exactly_the_two_store_dirs() {
        let cmd = r"C:\Windows\System32\cmd.exe";
        let store = Path::new(r"C:\Users\a b\AppData\Local\Programs\TextVN-Store");
        let staging = Path::new(r"C:\Users\a b\.textvn\msix-staging");
        assert_eq!(
            runonce_cleanup_command(cmd, store, staging),
            Some(vec![
                r#""C:\Windows\System32\cmd.exe" /d /c rmdir /s /q "C:\Users\a b\AppData\Local\Programs\TextVN-Store""#.to_string(),
                r#""C:\Windows\System32\cmd.exe" /d /c rmdir /s /q "C:\Users\a b\.textvn\msix-staging""#.to_string(),
                r#""C:\Windows\System32\cmd.exe" /d /c rmdir "C:\Users\a b\.textvn""#.to_string(),
            ])
        );
        // R2-94: tên tài khoản dài 40 ký tự — từng lệnh vẫn ≤ 260 ký tự.
        let long_user = "u".repeat(40);
        let long_store = format!(r"C:\Users\{long_user}\AppData\Local\Programs\TextVN-Store");
        let long_staging = format!(r"C:\Users\{long_user}\.textvn\msix-staging");
        let cmds =
            runonce_cleanup_command(cmd, Path::new(&long_store), Path::new(&long_staging)).unwrap();
        assert_eq!(cmds.len(), RUNONCE_VALUE_NAMES.len());
        assert!(cmds.iter().all(|c| c.len() <= 260), "{cmds:?}");
        // Sai thư mục → không bao giờ dựng lệnh xoá đệ quy.
        for (s, g) in [
            (
                r"C:\Users\a\AppData\Local\Programs\TextVN",
                r"C:\Users\a\.textvn\msix-staging",
            ),
            (
                r"C:\Users\a\AppData\Local\Programs",
                r"C:\Users\a\.textvn\msix-staging",
            ),
            (
                r"C:\Users\a\AppData\Local\Programs\TextVN-Store",
                r"C:\Users\a\.textvn",
            ),
            (
                r"C:\Users\a\AppData\Local\Programs\TextVN-Store",
                r"C:\Users\a",
            ),
            (r"Programs\TextVN-Store", r"C:\Users\a\.textvn\msix-staging"),
            (
                r"C:\Users\%x%\AppData\Local\Programs\TextVN-Store",
                r"C:\Users\a\.textvn\msix-staging",
            ),
            (
                r#"C:\Users\a"\AppData\Local\Programs\TextVN-Store"#,
                r"C:\Users\a\.textvn\msix-staging",
            ),
        ] {
            assert_eq!(
                runonce_cleanup_command(cmd, Path::new(s), Path::new(g)),
                None,
                "{s} | {g}"
            );
        }
    }
}
