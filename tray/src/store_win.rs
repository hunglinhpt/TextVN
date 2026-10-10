// SPDX-License-Identifier: GPL-3.0-or-later
//! Kênh Microsoft Store — phần gọi Windows API: package identity, package family,
//! tạo tiến trình thoát khỏi container MSIX, guard gỡ/cập nhật, dọn dẹp, RunOnce.
//! Logic thuần (đường dẫn, `stage.json`, so version, quyết định) ở [`crate::store`];
//! luồng packaged entry → relay → install ở [`crate::package_bootstrap`].

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, WIN32_ERROR,
};
use windows::Win32::Storage::Packaging::Appx::{
    GetCurrentPackageFamilyName, GetCurrentPackageFullName, GetPackagesByPackageFamily,
};
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY, STARTUPINFOEXW,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, SW_SHOWNORMAL,
};

use crate::store::{self, GuardAction, LaunchMode, StageInfo};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// `CREATE_BREAKAWAY_FROM_JOB` — tray Store không được nằm trong job của container
/// package (giữ container cũ sống → bản cập nhật không khởi chạy được, AppModel-Runtime
/// event 215). Job không cho breakaway → `ERROR_ACCESS_DENIED` → thử lại không cờ.
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
/// `PROCESS_CREATION_DESKTOP_APP_BREAKAWAY_ENABLE_PROCESS_TREE` (winbase.h): CON của
/// tiến trình được tạo sẽ chạy NGOÀI môi trường desktop app của package.
const DESKTOP_APP_BREAKAWAY_ENABLE_PROCESS_TREE: u32 = 0x1;
const ERROR_ACCESS_DENIED_CODE: i32 = 5;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

// ─── Package identity ───────────────────────────────────────────────────────────

/// Tiến trình hiện tại có package identity (chạy trong container MSIX) không.
/// `ERROR_INSUFFICIENT_BUFFER` = có; `APPMODEL_ERROR_NO_PACKAGE` (15700) = không.
pub fn has_package_identity() -> bool {
    let mut len = 0u32;
    // SAFETY: truy vấn độ dài, buffer NULL được phép.
    let rc = unsafe { GetCurrentPackageFullName(&mut len, None) };
    if rc == APPMODEL_ERROR_NO_PACKAGE {
        return false;
    }
    rc == ERROR_INSUFFICIENT_BUFFER || rc == ERROR_SUCCESS
}

fn read_package_string(f: impl Fn(&mut u32, Option<PWSTR>) -> WIN32_ERROR) -> Option<String> {
    let mut len = 0u32;
    if f(&mut len, None) != ERROR_INSUFFICIENT_BUFFER || len == 0 {
        return None;
    }
    let mut buf = vec![0u16; len as usize];
    if f(&mut len, Some(PWSTR(buf.as_mut_ptr()))) != ERROR_SUCCESS {
        return None;
    }
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..end]))
}

/// Package full name của tiến trình hiện tại (`Name_Version_Arch_ResId_PublisherId`).
pub fn current_package_full_name() -> Option<String> {
    // SAFETY: `len` khớp kích thước buffer do chính API báo.
    read_package_string(|len, buf| unsafe { GetCurrentPackageFullName(len, buf) })
}

/// Package family name của tiến trình hiện tại (`Name_PublisherId`).
pub fn current_package_family_name() -> Option<String> {
    // SAFETY: `len` khớp kích thước buffer do chính API báo.
    read_package_string(|len, buf| unsafe { GetCurrentPackageFamilyName(len, buf) })
}

/// Full name các package thuộc `pfn` đang cài cho người dùng hiện tại.
/// `Some(vec![])` = package đã bị gỡ; `None` = API lỗi (caller KHÔNG được dọn).
pub fn package_family_full_names(pfn: &str) -> Option<Vec<String>> {
    let fam = wide(pfn);
    let (mut count, mut buflen) = (0u32, 0u32);
    // SAFETY: truy vấn số lượng/độ dài với buffer NULL.
    let rc = unsafe {
        GetPackagesByPackageFamily(PCWSTR(fam.as_ptr()), &mut count, None, &mut buflen, None)
    };
    if rc == ERROR_SUCCESS && count == 0 {
        return Some(Vec::new());
    }
    if rc != ERROR_INSUFFICIENT_BUFFER || count == 0 || buflen == 0 {
        return None;
    }
    let mut names = vec![PWSTR::null(); count as usize];
    let mut buf = vec![0u16; buflen as usize];
    // SAFETY: `names` có `count` phần tử, `buf` có `buflen` WCHAR như API vừa báo;
    // các PWSTR trả về trỏ vào `buf` (còn sống tới hết hàm).
    let rc = unsafe {
        GetPackagesByPackageFamily(
            PCWSTR(fam.as_ptr()),
            &mut count,
            Some(names.as_mut_ptr()),
            &mut buflen,
            Some(PWSTR(buf.as_mut_ptr())),
        )
    };
    if rc != ERROR_SUCCESS {
        return None;
    }
    names.truncate(count as usize);
    Some(
        names
            .iter()
            .filter(|p| !p.is_null())
            // SAFETY: chuỗi NUL-terminated nằm trong `buf`.
            .filter_map(|p| unsafe { p.to_string() }.ok())
            .collect(),
    )
}

/// Chốt chặn cứng (thiết kế kênh Store, mục 5): tiến trình còn package identity thì
/// MỌI ghi đăng ký TSF / Run / phím tắt / HKLM đều bị Windows ảo hoá vào hive riêng của
/// package (vô hình với app khác, mất khi gỡ) → từ chối, ghi log, caller trả về.
pub fn refuse_if_packaged(action: &str) -> bool {
    if has_package_identity() {
        store_log(&format!(
            "từ chối {action}: tiến trình có package identity (MSIX) — ghi sẽ bị ảo hoá"
        ));
        return true;
    }
    false
}

// ─── Log (không bao giờ chứa văn bản người dùng gõ — S2) ─────────────────────────

fn log_path() -> Option<PathBuf> {
    if has_package_identity() {
        // Trong package, file mới dưới AppData bị ảo hoá → ghi vào vùng staging (thật).
        let profile = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty())?;
        Some(store::staging_root(Path::new(&profile)).join("bootstrap.log"))
    } else {
        let local = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty())?;
        Some(
            PathBuf::from(local)
                .join("TextVN")
                .join("logs")
                .join("store.log"),
        )
    }
}

/// Ghi một dòng vào `%LOCALAPPDATA%\TextVN\logs\store.log` (hoặc
/// `%USERPROFILE%\.textvn\msix-staging\bootstrap.log` khi còn package identity).
pub fn store_log(msg: &str) {
    use std::io::Write;
    let Some(path) = log_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        let mut rotated = path.clone().into_os_string();
        rotated.push(".1");
        let _ = std::fs::rename(&path, PathBuf::from(rotated));
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "[{ts} pid={}] {msg}", std::process::id());
    }
}

// ─── UI ─────────────────────────────────────────────────────────────────────────

/// Hộp thoại đồng bộ (caller tự lo thread nếu đang ở UI thread của tray).
pub fn message_box(text: &str, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let text = wide(text);
    let caption = wide("TextVN");
    // SAFETY: chuỗi NUL-terminated sống suốt lời gọi đồng bộ.
    unsafe { MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(caption.as_ptr()), style) }
}

/// Báo lỗi kênh Store (ghi log + hộp thoại).
pub fn report_error(detail: &str) {
    store_log(&format!("LỖI: {detail}"));
    let _ = message_box(
        &format!(
            "TextVN (Microsoft Store) chưa khởi chạy được.\r\n\r\n{detail}\r\n\r\nHãy mở lại \
TextVN từ Start. Nếu vẫn lỗi, dùng bộ cài TextVN-setup-*.exe từ trang phát hành của TextVN."
        ),
        MB_OK | MB_ICONERROR,
    );
}

/// Mở Cài đặt › Ứng dụng để người dùng gỡ gói Store (R2-04).
pub fn open_apps_settings() {
    let verb = wide("open");
    let target = wide("ms-settings:appsfeatures");
    // SAFETY: chuỗi NUL-terminated sống suốt lời gọi.
    unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

// ─── Tạo tiến trình ─────────────────────────────────────────────────────────────

/// `CreateProcessW` + `STARTUPINFOEXW` với PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY =
/// BREAKAWAY_ENABLE_PROCESS_TREE: tiến trình được tạo VẪN trong package, nhưng mọi
/// tiến trình CON của nó chạy ngoài container (UpdateProcThreadAttribute, Remarks) —
/// vì vậy cần hai bước (relay rồi installer).
pub fn spawn_with_desktop_app_breakaway(exe: &Path, args: &[String]) -> Result<(), String> {
    let app = wide(&exe.to_string_lossy());
    let mut cmd = wide(&store::build_command_line(&exe.to_string_lossy(), args));
    let mut size = 0usize;
    // SAFETY: truy vấn kích thước (trả lỗi ERROR_INSUFFICIENT_BUFFER là bình thường).
    let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut size) };
    if size == 0 {
        return Err("InitializeProcThreadAttributeList: kích thước 0".to_string());
    }
    // Buffer căn theo usize (attribute list chứa con trỏ).
    let mut storage = vec![0usize; size.div_ceil(std::mem::size_of::<usize>())];
    let list = LPPROC_THREAD_ATTRIBUTE_LIST(storage.as_mut_ptr().cast());
    let policy: u32 = DESKTOP_APP_BREAKAWAY_ENABLE_PROCESS_TREE;
    // SAFETY: `storage` đủ `size` byte và sống tới DeleteProcThreadAttributeList;
    // `policy` sống tới sau CreateProcessW.
    unsafe {
        InitializeProcThreadAttributeList(Some(list), 1, None, &mut size)
            .map_err(|e| format!("InitializeProcThreadAttributeList: {e}"))?;
        if let Err(e) = UpdateProcThreadAttribute(
            list,
            0,
            PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY as usize,
            Some((&policy as *const u32).cast()),
            std::mem::size_of::<u32>(),
            None,
            None,
        ) {
            DeleteProcThreadAttributeList(list);
            return Err(format!("UpdateProcThreadAttribute: {e}"));
        }
    }
    let mut si = STARTUPINFOEXW::default();
    si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    si.lpAttributeList = list;
    let mut pi = PROCESS_INFORMATION::default();
    // SAFETY: mọi buffer NUL-terminated và sống suốt lời gọi; `si` trỏ attribute list hợp lệ.
    let result = unsafe {
        CreateProcessW(
            PCWSTR(app.as_ptr()),
            Some(PWSTR(cmd.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            PCWSTR::null(),
            &si.StartupInfo,
            &mut pi,
        )
    };
    // SAFETY: list đã khởi tạo ở trên.
    unsafe { DeleteProcThreadAttributeList(list) };
    result.map_err(|e| format!("CreateProcessW: {e}"))?;
    // SAFETY: handle do CreateProcessW trả về; đóng ngay (không chờ).
    unsafe {
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
    }
    Ok(())
}

/// Chạy `exe args` tách khỏi job hiện tại (CREATE_BREAKAWAY_FROM_JOB), thử lại không
/// cờ khi job không cho phép (ERROR_ACCESS_DENIED).
pub fn spawn_detached(exe: &Path, args: &[String]) -> std::io::Result<()> {
    let spawn = |flags: u32| {
        std::process::Command::new(exe)
            .args(args)
            .creation_flags(flags)
            .spawn()
            .map(|_| ())
    };
    match spawn(CREATE_BREAKAWAY_FROM_JOB) {
        Err(e) if e.raw_os_error() == Some(ERROR_ACCESS_DENIED_CODE) => spawn(0),
        other => other,
    }
}

/// Chạy tiến trình ẩn và chờ tối đa `timeout`; quá hạn thì bỏ chờ (không giết).
fn run_hidden_and_wait(exe: &Path, args: &[&std::ffi::OsStr], timeout: Duration) -> bool {
    let Ok(mut child) = std::process::Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    else {
        return false;
    };
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(100))
            }
            _ => return false,
        }
    }
}

/// Dừng tray kênh Store đang chạy (chỉ tiến trình có image dưới `TextVN-Store`, không
/// đụng bản cài/portable — R2-30), chờ tối đa 10 giây.
pub fn stop_store_tray(root: &Path) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let ok = run_hidden_and_wait(
        &exe,
        &[
            "--stop".as_ref(),
            "--if-image-under".as_ref(),
            root.as_os_str(),
        ],
        Duration::from_secs(10),
    );
    store_log(&format!(
        "dừng tray Store cũ → {}",
        if ok { "OK" } else { "?" }
    ));
}

/// Chạy app trong package qua Explorer (`shell:AppsFolder\<PFN>!TextVN`).
pub fn launch_package_app(pfn: &str) -> std::io::Result<()> {
    let explorer = std::env::var_os("SystemRoot")
        .filter(|v| !v.is_empty())
        .map(|r| PathBuf::from(r).join("explorer.exe"))
        .unwrap_or_else(|| PathBuf::from("explorer.exe"));
    std::process::Command::new(explorer)
        .arg(store::apps_folder_target(pfn))
        .spawn()
        .map(|_| ())
}

// ─── Ngữ cảnh kênh Store ────────────────────────────────────────────────────────

/// Tray đang chạy thuộc kênh Store.
#[derive(Debug, Clone)]
pub struct StoreContext {
    /// `%LOCALAPPDATA%\Programs\TextVN-Store`.
    pub root: PathBuf,
    pub exe: PathBuf,
    pub exe_dir: PathBuf,
    /// `None` khi `stage.json` hỏng — guard bỏ qua (fail-open), các quy tắc Store khác vẫn áp.
    pub stage: Option<StageInfo>,
}

pub fn store_root_from_env() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty())?;
    Some(store::store_root(Path::new(&local)))
}

pub fn staging_root_from_env() -> Option<PathBuf> {
    let profile = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty())?;
    Some(store::staging_root(Path::new(&profile)))
}

/// `dir` nằm dưới `root` (so cả dạng gốc lẫn canonical — tên 8.3, `\\?\`).
pub fn dir_is_under(dir: &Path, root: &Path) -> bool {
    let mut roots = vec![root.to_string_lossy().into_owned()];
    if let Ok(c) = std::fs::canonicalize(root) {
        roots.push(c.to_string_lossy().into_owned());
    }
    let mut dirs = vec![dir.to_string_lossy().into_owned()];
    if let Ok(c) = std::fs::canonicalize(dir) {
        dirs.push(c.to_string_lossy().into_owned());
    }
    dirs.iter().any(|d| crate::path_is_under_any(d, &roots))
}

/// Hai đường dẫn cùng một thư mục (so canonical, fallback so chuỗi không phân biệt hoa thường).
pub fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a
            .to_string_lossy()
            .trim_end_matches('\\')
            .eq_ignore_ascii_case(b.to_string_lossy().trim_end_matches('\\')),
    }
}

pub fn read_stage(root: &Path) -> Option<StageInfo> {
    std::fs::read_to_string(root.join(store::STAGE_FILE))
        .ok()
        .and_then(|s| StageInfo::parse(&s))
}

/// Ghi `stage.json` nguyên tử (file tạm + rename).
pub fn write_stage(root: &Path, info: &StageInfo) -> std::io::Result<()> {
    std::fs::create_dir_all(root)?;
    let tmp = root.join(format!("{}.tmp", store::STAGE_FILE));
    std::fs::write(&tmp, info.to_json())?;
    std::fs::rename(&tmp, root.join(store::STAGE_FILE))
}

/// Exe hiện tại chạy dưới `%LOCALAPPDATA%\Programs\TextVN-Store` và có `stage.json`
/// → kênh Store.
pub fn detect_store_context() -> Option<StoreContext> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?.to_path_buf();
    let root = store_root_from_env()?;
    if !root.join(store::STAGE_FILE).is_file() || !dir_is_under(&exe_dir, &root) {
        return None;
    }
    let stage = read_stage(&root);
    Some(StoreContext {
        root,
        exe,
        exe_dir,
        stage,
    })
}

// ─── Run value / RunOnce ────────────────────────────────────────────────────────

/// Đồng bộ HKCU Run `TextVN` của kênh Store (xem [`store::decide_store_run_value`]).
pub fn sync_store_run_value(root: &Path, exe: &Path, first_install: bool) {
    if refuse_if_packaged("ghi HKCU Run") {
        return;
    }
    let existing = crate::autostart::autostart_command();
    if let Some(cmd) = store::decide_store_run_value(existing.as_deref(), root, exe, first_install)
    {
        let result = crate::autostart::write_run_value(&cmd);
        store_log(&format!(
            "Run TextVN ({}) → {}",
            if store::run_command_is_guard(&cmd) {
                "guard"
            } else {
                "autostart"
            },
            if result.is_ok() { "OK" } else { "FAIL" }
        ));
    }
}

fn cmd_exe_path() -> String {
    std::env::var_os("SystemRoot")
        .filter(|v| !v.is_empty())
        .map(|r| {
            PathBuf::from(r)
                .join("System32")
                .join("cmd.exe")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "cmd.exe".to_string())
}

/// Hẹn xoá `TextVN-Store` + `.textvn\msix-staging` ở lần đăng nhập sau (HKCU RunOnce)
/// — lúc đó không tiến trình nào còn nạp DLL/exe từ đó.
pub fn schedule_store_dir_removal(root: &Path) {
    let Some(staging) = staging_root_from_env() else {
        return;
    };
    match store::runonce_cleanup_command(&cmd_exe_path(), root, &staging) {
        Some(cmd) => {
            let r = crate::autostart::set_runonce_value(store::RUNONCE_VALUE_NAME, &cmd);
            store_log(&format!(
                "hẹn xoá thư mục kênh Store (RunOnce) → {}",
                if r.is_ok() { "OK" } else { "FAIL" }
            ));
        }
        None => store_log("không hẹn xoá thư mục: đường dẫn không đúng dạng mong đợi"),
    }
}

/// Bỏ lịch xoá còn treo (người dùng cài lại gói trước lần đăng nhập kế).
pub fn cancel_store_dir_removal() {
    let _ = crate::autostart::delete_runonce_value(store::RUNONCE_VALUE_NAME);
}

// ─── Dọn bản cũ ─────────────────────────────────────────────────────────────────

/// Xoá (best-effort) thư mục phiên bản cũ dưới `root` — chỉ tên dạng version
/// (`store::prunable_version_dir`); DLL cũ còn nạp trong app khác thì để lần sau.
pub fn prune_old_versions(root: &Path, keep: &str) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().is_ok_and(|t| t.is_dir()) && store::prunable_version_dir(&name, keep) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Xoá vùng staging (`%USERPROFILE%\.textvn\msix-staging`) rồi `.textvn` nếu rỗng.
pub fn remove_staging_root(retries: u32) {
    let Some(staging) = staging_root_from_env() else {
        return;
    };
    for attempt in 0..=retries {
        if !staging.exists() || std::fs::remove_dir_all(&staging).is_ok() {
            break;
        }
        if attempt < retries {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    if let Some(dot) = staging.parent() {
        // `remove_dir` chỉ xoá thư mục RỖNG.
        let _ = std::fs::remove_dir(dot);
    }
}

/// Việc nền sau khi tray kênh Store đã giữ single-instance mutex: đồng bộ Run value
/// sang thư mục phiên bản hiện tại, dọn phiên bản cũ và vùng staging.
pub fn after_tray_start(ctx: &StoreContext) {
    sync_store_run_value(&ctx.root, &ctx.exe, false);
    if let Some(keep) = ctx.exe_dir.file_name() {
        prune_old_versions(&ctx.root, &keep.to_string_lossy());
    }
    remove_staging_root(5);
}

// ─── Guard gỡ/cập nhật ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Chạy tiếp tray.
    Continue,
    /// Thoát ngay (đã dọn, đã chuyển sang package, hoặc chỉ chạy guard).
    Exit,
}

fn write_relaunch_hint(root: &Path, mode: LaunchMode) {
    let _ = std::fs::write(root.join(store::RELAUNCH_HINT_FILE), mode.hint());
}

/// Đọc + xoá gợi ý cách chạy tray (chỉ tin gợi ý mới hơn 10 phút).
pub fn take_relaunch_hint(root: &Path) -> Option<LaunchMode> {
    let path = root.join(store::RELAUNCH_HINT_FILE);
    let fresh = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < Duration::from_secs(600));
    let content = std::fs::read_to_string(&path).ok();
    let _ = std::fs::remove_file(&path);
    if fresh {
        content.as_deref().and_then(LaunchMode::from_hint)
    } else {
        None
    }
}

/// Guard mỗi lần khởi động kênh Store (thường, `--autostart`, `--msix-guard`):
/// - package đã gỡ → dọn (R2-04) rồi thoát;
/// - package có bản mới hơn `stage.json` → chạy app trong package (stage lại) rồi
///   thoát (R2-05/R2-23);
/// - API lỗi → không làm gì (fail-open, S4).
pub fn run_store_guard(ctx: &StoreContext, mode: LaunchMode) -> GuardOutcome {
    let after = if mode == LaunchMode::GuardOnly {
        GuardOutcome::Exit
    } else {
        GuardOutcome::Continue
    };
    let Some(stage) = &ctx.stage else {
        return after;
    };
    let mut action =
        store::guard_decision(&stage.ver, package_family_full_names(&stage.pfn).as_deref());
    if action == GuardAction::Cleanup {
        // Hỏi lại sau một nhịp: đầu phiên đăng nhập AppX có thể chưa kịp đăng ký gói —
        // dọn nhầm là tắt bộ gõ của người dùng.
        std::thread::sleep(Duration::from_secs(3));
        action =
            store::guard_decision(&stage.ver, package_family_full_names(&stage.pfn).as_deref());
    }
    match action {
        GuardAction::Continue => after,
        GuardAction::Cleanup => {
            store_log("guard: gói Store đã bị gỡ → dọn đăng ký TextVN");
            store_cleanup(ctx, true);
            GuardOutcome::Exit
        }
        GuardAction::LaunchPackage => {
            write_relaunch_hint(&ctx.root, mode);
            match launch_package_app(&stage.pfn) {
                Ok(()) => {
                    store_log(&format!(
                        "guard: gói có bản mới hơn {} → chạy app trong gói để stage lại",
                        stage.ver
                    ));
                    GuardOutcome::Exit
                }
                Err(e) => {
                    store_log(&format!("guard: không chạy được app trong gói: {e}"));
                    let _ = std::fs::remove_file(ctx.root.join(store::RELAUNCH_HINT_FILE));
                    after
                }
            }
        }
    }
}

/// Dọn mọi thứ kênh Store đã ghi NGOÀI package (MSIX full-trust không có hook gỡ):
/// TIP HKCU (+ mirror WOW64) nếu thuộc `TextVN-Store`, Run value trỏ vào đó, Ctrl+Shift
/// nếu chính TextVN đã đổi, rồi hẹn xoá thư mục ở lần đăng nhập sau. Không đụng HKLM
/// (kênh Store không bao giờ đăng ký phạm vi máy — R2-07). Cấu hình `%APPDATA%\TextVN`
/// được giữ (như bản portable).
pub fn store_cleanup(ctx: &StoreContext, stop_running_tray: bool) {
    if refuse_if_packaged("dọn kênh Store") {
        return;
    }
    if stop_running_tray {
        stop_store_tray(&ctx.root);
    }
    let cli = ctx.exe_dir.join("textvn-cli.exe");
    if cli.is_file() {
        let ok = run_hidden_and_wait(
            &cli,
            &[
                "unregister".as_ref(),
                "--scope".as_ref(),
                "user".as_ref(),
                "--if-owned-by".as_ref(),
                ctx.root.as_os_str(),
            ],
            Duration::from_secs(60),
        );
        store_log(&format!(
            "gỡ đăng ký TSF (HKCU) → {}",
            if ok { "OK" } else { "FAIL" }
        ));
    }
    let _ = crate::autostart::disable_autostart_for_dir(&ctx.root);
    crate::restore_ctrl_shift_if_we_freed();
    schedule_store_dir_removal(&ctx.root);
    remove_staging_root(0);
}
