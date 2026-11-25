// SPDX-License-Identifier: GPL-3.0-or-later
//! E70 — Ghi báo cáo panic an toàn quyền riêng tư (G8/S2) cho các tiến trình RIÊNG của
//! TextVN (tray `TextVN.exe`, `textvn-cli`, `textvn-hook.exe`).
//!
//! Vì sao: trước E70 không tiến trình nào cài `std::panic::set_hook` — tray panic thì
//! chết im lặng, gói `doctor --export` không có dấu vết gì để RCA.
//!
//! Luật riêng tư (không thương lượng):
//! - **Không** ghi payload `String` động của panic (`format!` có thể chứa chữ người dùng
//!   đã gõ / nội dung clipboard) — chỉ ghi độ dài. Payload `&'static str` là hằng chuỗi
//!   biên dịch sẵn trong mã nguồn (vd. thông điệp `expect("…")`) nên được giữ.
//! - Backtrace chỉ giữ dòng tên hàm (ký hiệu mã), bỏ dòng `at <path>` của máy build.
//! - Không ghi biến môi trường, đường dẫn người dùng, tham số dòng lệnh.
//!
//! Không cài vào DLL TSF (nạp trong tiến trình của ứng dụng khác) — FFI đã `catch_unwind`.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Once;

/// Tên file panic trong thư mục log.
pub const PANIC_LOG_FILE: &str = "panic.log";
/// Giới hạn kích thước file panic.log (xoay vòng: vượt thì giữ nửa sau).
pub const PANIC_LOG_MAX_BYTES: u64 = 64 * 1024;
/// Số khung backtrace tối đa ghi lại.
pub const MAX_FRAMES: usize = 48;

/// Thông tin một panic sau khi đã lọc riêng tư — dùng chung cho hook và unit test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanicRecord {
    pub unix_ts: u64,
    pub component: String,
    pub version: String,
    pub target: String,
    pub thread: String,
    pub location: String,
    /// Chỉ có khi payload là `&'static str` (hằng chuỗi trong mã nguồn).
    pub static_message: Option<String>,
    /// Độ dài payload động (String) — nội dung KHÔNG được ghi.
    pub dynamic_payload_len: Option<usize>,
    pub frames: Vec<String>,
}

impl PanicRecord {
    /// Định dạng một khối văn bản kết thúc bằng dòng trống.
    pub fn render(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "=== TextVN panic ts={} component={} version={} target={} ===",
            self.unix_ts, self.component, self.version, self.target
        );
        let _ = writeln!(s, "code: ERR_PANIC");
        let _ = writeln!(s, "thread: {}", self.thread);
        let _ = writeln!(s, "location: {}", self.location);
        match (&self.static_message, self.dynamic_payload_len) {
            (Some(m), _) => {
                let _ = writeln!(s, "message(static): {m}");
            }
            (None, Some(n)) => {
                let _ = writeln!(s, "message: <omitted dynamic payload, {n} bytes — G8>");
            }
            (None, None) => {
                let _ = writeln!(s, "message: <non-string payload>");
            }
        }
        if !self.frames.is_empty() {
            let _ = writeln!(s, "backtrace (symbols only):");
            for f in &self.frames {
                let _ = writeln!(s, "  {f}");
            }
        }
        s.push('\n');
        s
    }
}

/// Lọc backtrace `std::backtrace::Backtrace` dạng text: giữ dòng `N: symbol`, bỏ dòng
/// `at <file>:<line>` (đường dẫn máy build) và cắt tối đa [`MAX_FRAMES`].
pub fn sanitize_backtrace(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("at "))
        .filter(|l| {
            l.split_once(':')
                .is_some_and(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .take(MAX_FRAMES)
        .map(|l| l.chars().take(240).collect())
        .collect()
}

/// Thư mục log mặc định per-OS (trùng nơi `textvn doctor --export` đọc):
/// Windows `%LOCALAPPDATA%\TextVN\logs` · macOS `~/Library/Logs/TextVN` ·
/// Linux `$XDG_STATE_HOME/TextVN/log` hoặc `~/.local/state/TextVN/log`.
pub fn default_log_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .filter(|v| !v.is_empty())
            .map(|p| PathBuf::from(p).join("TextVN").join("logs"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .map(|h| PathBuf::from(h).join("Library/Logs/TextVN"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_STATE_HOME") {
            let p = PathBuf::from(xdg);
            if p.is_absolute() {
                return Some(p.join("TextVN/log"));
            }
        }
        std::env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .map(|h| PathBuf::from(h).join(".local/state/TextVN/log"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Nối một bản ghi vào `<dir>/panic.log` (tạo thư mục nếu thiếu; không theo symlink;
/// vượt [`PANIC_LOG_MAX_BYTES`] thì giữ nửa sau). Lỗi I/O bị nuốt — hook panic
/// không được tự panic.
pub fn append_panic_record(dir: &Path, record: &PanicRecord) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(PANIC_LOG_FILE);
    if let Ok(meta) = std::fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink() {
            return Err(std::io::Error::other("panic.log là symlink — từ chối ghi"));
        }
        if meta.len() > PANIC_LOG_MAX_BYTES {
            if let Ok(old) = std::fs::read(&path) {
                let keep_from = old.len() / 2;
                // Cắt ở đầu một khối "=== TextVN panic" để file luôn đọc được.
                let tail = &old[keep_from..];
                let start = find_subslice(tail, b"=== TextVN panic").unwrap_or(0);
                std::fs::write(&path, &tail[start..])?;
            }
        }
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    f.write_all(record.render().as_bytes())?;
    Ok(path)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Dựng [`PanicRecord`] từ `PanicHookInfo` — tách ra để test được phần lọc payload.
pub fn record_from_parts(
    component: &str,
    version: &str,
    payload: &(dyn std::any::Any + Send),
    location: Option<&std::panic::Location<'_>>,
    backtrace_text: &str,
) -> PanicRecord {
    let static_message = payload
        .downcast_ref::<&'static str>()
        .map(|s| s.chars().take(300).collect::<String>());
    let dynamic_payload_len = if static_message.is_none() {
        payload.downcast_ref::<String>().map(String::len)
    } else {
        None
    };
    let unix_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let thread = std::thread::current()
        .name()
        .unwrap_or("<unnamed>")
        .chars()
        .take(64)
        .collect();
    PanicRecord {
        unix_ts,
        component: component.to_string(),
        version: version.to_string(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        thread,
        location: location
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".into()),
        static_message,
        dynamic_payload_len,
        frames: sanitize_backtrace(backtrace_text),
    }
}

static INSTALL: Once = Once::new();

/// Cài panic hook ghi `panic.log` rồi gọi hook mặc định (giữ hành vi stderr cũ).
/// Gọi nhiều lần an toàn (chỉ lần đầu có hiệu lực). `component` = tên tiến trình
/// ngắn (vd. `"tray"`, `"cli"`, `"hook"`), `version` = `env!("CARGO_PKG_VERSION")`
/// của binary gọi.
pub fn install_panic_hook(component: &'static str, version: &'static str) {
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let bt = std::backtrace::Backtrace::force_capture().to_string();
            let rec = record_from_parts(component, version, info.payload(), info.location(), &bt);
            if let Some(dir) = default_log_dir() {
                let _ = append_panic_record(&dir, &rec);
            }
            previous(info);
        }));
    });
}

/// Đọc tối đa `max_bytes` cuối của `panic.log` (cho gói chẩn đoán). `None` nếu chưa có.
pub fn read_panic_log_tail(dir: &Path, max_bytes: usize) -> Option<String> {
    let path = dir.join(PANIC_LOG_FILE);
    let meta = std::fs::symlink_metadata(&path).ok()?;
    if meta.file_type().is_symlink() {
        return None;
    }
    let data = std::fs::read(&path).ok()?;
    let start = data.len().saturating_sub(max_bytes);
    Some(String::from_utf8_lossy(&data[start..]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "mật khẩu ngân hàng 123456 xin chào";

    #[test]
    fn e70_dynamic_payload_is_never_recorded() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(format!("bad word {SECRET}"));
        let rec = record_from_parts("tray", "9.9.9", payload.as_ref(), None, "");
        let out = rec.render();
        assert!(
            !out.contains(SECRET),
            "payload động lọt vào panic.log: {out}"
        );
        assert!(!out.contains("bad word"));
        assert_eq!(rec.static_message, None);
        assert_eq!(
            rec.dynamic_payload_len,
            Some(format!("bad word {SECRET}").len())
        );
        assert!(out.contains("ERR_PANIC"));
        assert!(out.contains("omitted dynamic payload"));
    }

    #[test]
    fn e70_static_payload_is_kept() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("config lock poisoned");
        let rec = record_from_parts("cli", "9.9.9", payload.as_ref(), None, "");
        assert_eq!(rec.static_message.as_deref(), Some("config lock poisoned"));
        assert!(rec
            .render()
            .contains("message(static): config lock poisoned"));
    }

    #[test]
    fn e70_backtrace_keeps_symbols_drops_paths() {
        let raw = "   0: std::backtrace::Backtrace::force_capture\n             at /rustc/abc/library/std/src/backtrace.rs:312:9\n   1: textvn_tray::svc::SvcManager::save\n             at C:\\Users\\runneradmin\\work\\tray\\src\\svc.rs:10:5\nnote: some text\n";
        let frames = sanitize_backtrace(raw);
        assert_eq!(
            frames,
            vec![
                "0: std::backtrace::Backtrace::force_capture".to_string(),
                "1: textvn_tray::svc::SvcManager::save".to_string(),
            ]
        );
        let many: String = (0..200).map(|i| format!("{i}: f{i}\n")).collect();
        assert_eq!(sanitize_backtrace(&many).len(), MAX_FRAMES);
    }

    #[test]
    fn e70_append_rotates_and_stays_readable() {
        let dir = std::env::temp_dir().join(format!(
            "textvn-crash-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let payload: Box<dyn std::any::Any + Send> = Box::new("x");
        let mut rec = record_from_parts("cli", "9.9.9", payload.as_ref(), None, "");
        rec.frames = (0..MAX_FRAMES)
            .map(|i| format!("{i}: {}", "f".repeat(200)))
            .collect();
        for _ in 0..20 {
            append_panic_record(&dir, &rec).expect("append");
        }
        let len = std::fs::metadata(dir.join(PANIC_LOG_FILE)).unwrap().len();
        assert!(
            len <= PANIC_LOG_MAX_BYTES + rec.render().len() as u64,
            "len={len}"
        );
        let tail = read_panic_log_tail(&dir, 32 * 1024).unwrap();
        assert!(tail.contains("ERR_PANIC"));
        let text = std::fs::read_to_string(dir.join(PANIC_LOG_FILE)).unwrap();
        assert!(
            text.starts_with("=== TextVN panic"),
            "rotation cắt giữa khối"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
