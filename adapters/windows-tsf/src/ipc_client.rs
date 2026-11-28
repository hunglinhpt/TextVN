// SPDX-License-Identifier: GPL-3.0-or-later
//! IPC client của TSF (WIN-016) — **một client cho mỗi process**, không block app.
//!
//! Kết nối pipe theo phiên `\\.\pipe\textvn-ipc-v1-s<phiên>` (tên cũ `textvn-ipc-v1`
//! dự phòng — `textvn_ipc::candidate_pipe_names`); tray chưa chạy → dùng trạng thái đọc từ
//! `%APPDATA%\TextVN\state.json` và thử lại định kỳ (offline-tolerant).
//!
//! R2-26: tiến trình sandbox (AppContainer/LPAC — ô tìm kiếm Start, ứng dụng Store) bị từ chối
//! cả `state.json` lẫn pipe khay ⇒ đọc bản sao `%APPDATA%\TextVN\sandbox\state.json` và nối
//! pipe sandbox (`textvn_ipc::sandbox`). Trên kết nối sandbox, client KHÔNG gửi nội dung người
//! dùng: drain tự học chỉ còn bộ đếm Dream RSI, không toast/lệnh (tray cũng từ chối).
//!
//! Bất biến quan trọng:
//! - Thread nền **không bao giờ bị join** từ thread UI của app. Bản trước join
//!   trong `Deactivate` trong khi thread đang chặn ở `read_exact` → app treo khi
//!   đổi bộ gõ/đóng cửa sổ. Thread sống theo process; `DllCanUnloadNow` giữ DLL
//!   trong bộ nhớ khi thread đã chạy ([`worker_started`]).
//! - Trạng thái bật/tắt = `global && app_override.unwrap_or(true)`: tắt toàn cục
//!   từ tray (`app_id = "*"`) có hiệu lực ở mọi app; per-app chỉ tắt riêng app.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use textvn_ipc::client::{
    allowed_in_sandbox, parse_state_json, ReconnectBackoff, ASSIST_APP_ID, GLOBAL_APP_ID,
};
use textvn_ipc::sandbox::{open_tray_pipe, PipeOpen};
use textvn_ipc::{decode_exact_frame, encode_frame, Message, MAX_FRAME_BYTES};
use windows::Win32::System::Pipes::WaitNamedPipeW;

/// Vòng 16 (audit A7): pipe phiên (server cùng phiên) trước, legacy sau.
pub fn pipe_name_candidates() -> [String; 2] {
    textvn_ipc::candidate_pipe_names()
}

/// Nối tray (pipe phiên → tên cũ → pipe sandbox khi bị từ chối quyền), đã xác thực server.
/// Trả `(pipe, sandboxed)`; `None` = tray không chạy / bận / server giả.
fn connect_tray() -> Option<(std::fs::File, bool)> {
    match open_tray_pipe() {
        PipeOpen::Connected { file, sandboxed } => Some((file, sandboxed)),
        PipeOpen::Busy(_) | PipeOpen::Unavailable => None,
    }
}

/// Khóa trong map trạng thái đại diện cho công tắc toàn cục (khớp tray).
pub const GLOBAL_KEY: &str = GLOBAL_APP_ID;

static WORKER_STARTED: AtomicBool = AtomicBool::new(false);
static CLIENT: OnceLock<IpcClient> = OnceLock::new();

/// `true` khi thread IPC nền của process đã được tạo (DLL không được unload).
pub fn worker_started() -> bool {
    WORKER_STARTED.load(Ordering::Acquire)
}

/// Trạng thái chia sẻ giữa thread IPC và các thread UI dùng TSF.
#[derive(Debug)]
pub struct IpcState {
    pub connected: AtomicBool,
    /// Tăng mỗi khi tray báo config đổi; thread TSF so với bản đã áp của nó.
    pub config_version: AtomicU64,
    pub global_enabled: AtomicBool,
    pub app_enabled: AtomicBool,
    pub has_app_override: AtomicBool,
    pub assist_enabled: AtomicBool,
}

impl Default for IpcState {
    fn default() -> Self {
        Self {
            connected: AtomicBool::new(false),
            config_version: AtomicU64::new(0),
            global_enabled: AtomicBool::new(true),
            app_enabled: AtomicBool::new(true),
            has_app_override: AtomicBool::new(false),
            // Vòng 29 (root cause "trợ lý không hoạt động 0.2.40"): tray
            // state.json mặc định assist_enabled=true từ 0.2.38, nhưng DLL nạp
            // vào app khi CHƯA nhận Snapshot thì gate này false ⇒ tắt toàn bộ
            // trợ lý (không popup/gạch chân/sửa) dù config nói highlight.
            assist_enabled: AtomicBool::new(true),
        }
    }
}

impl IpcState {
    /// Áp map trạng thái (Snapshot từ tray hoặc `state.json`).
    fn apply_states(&self, app_id: &str, states: &BTreeMap<String, bool>) {
        if let Some(&global) = states.get(GLOBAL_KEY) {
            self.global_enabled.store(global, Ordering::Release);
        }
        if let Some(&assist) = states.get(ASSIST_APP_ID) {
            self.assist_enabled.store(assist, Ordering::Release);
        }
        match states.get(app_id) {
            Some(&enabled) => {
                self.app_enabled.store(enabled, Ordering::Release);
                self.has_app_override.store(true, Ordering::Release);
            }
            None => self.has_app_override.store(false, Ordering::Release),
        }
    }

    fn apply_update(&self, app_id: &str, update_app: &str, enabled: bool) {
        if update_app == GLOBAL_KEY {
            self.global_enabled.store(enabled, Ordering::Release);
        } else if update_app == ASSIST_APP_ID {
            self.assist_enabled.store(enabled, Ordering::Release);
        } else if update_app.eq_ignore_ascii_case(app_id) {
            self.app_enabled.store(enabled, Ordering::Release);
            self.has_app_override.store(true, Ordering::Release);
        }
    }
}

/// Handle tới trạng thái IPC của process.
pub struct IpcClient {
    pub(crate) state: Arc<IpcState>,
    /// app_id chuẩn hoá của process (điền lúc init; dùng cho snapshot response —
    /// chỉ đường toggle thread phía Windows đọc).
    #[cfg_attr(not(windows), allow(dead_code))]
    app_id: String,
}

impl IpcClient {
    /// Client dùng chung của process; lần gọi đầu nạp `state.json` và khởi
    /// động thread nền. Không bao giờ block thread gọi.
    pub fn global(app_id: &str) -> &'static IpcClient {
        CLIENT.get_or_init(|| {
            let state = Arc::new(IpcState::default());
            let app_id = textvn_field_detect::normalize_app_id(app_id);
            if let Some(states) = read_state_file() {
                state.apply_states(&app_id, &states);
            }
            {
                let worker_state = Arc::clone(&state);
                let worker_app_id = app_id.clone();
                let spawned = std::thread::Builder::new()
                    .name("textvn-tsf-ipc".into())
                    .spawn(move || run_client_loop(worker_app_id, worker_state));
                if spawned.is_ok() {
                    WORKER_STARTED.store(true, Ordering::Release);
                }
            }
            #[cfg(not(windows))]
            {
                WORKER_STARTED.store(true, Ordering::Release);
            }
            IpcClient {
                state,
                app_id: app_id.clone(),
            }
        })
    }

    /// Client không có thread nền — cho unit test.
    #[cfg(test)]
    fn detached() -> IpcClient {
        IpcClient {
            state: Arc::new(IpcState::default()),
            app_id: String::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.state.connected.load(Ordering::Acquire)
    }

    /// Phiên bản config mới nhất tray đã báo (0 = chưa có tín hiệu).
    pub fn config_version(&self) -> u64 {
        self.state.config_version.load(Ordering::Acquire)
    }

    /// Bộ gõ có đang bật cho app hiện tại không.
    pub fn is_enabled(&self) -> bool {
        let global = self.state.global_enabled.load(Ordering::Acquire);
        let app = if self.state.has_app_override.load(Ordering::Acquire) {
            self.state.app_enabled.load(Ordering::Acquire)
        } else {
            true
        };
        global && app
    }

    /// Trợ lý ngữ pháp có đang bật hay không (D20 kill-switch 1 bước · AS-023).
    pub fn is_assist_enabled(&self) -> bool {
        self.state.assist_enabled.load(Ordering::Acquire)
    }

    /// Hotkey trong TSF: đổi ngay trong process (phản hồi tức thì, kể cả khi
    /// tray không chạy) rồi báo tray; tray broadcast lại giá trị chuẩn cho mọi app.
    pub fn toggle_global(&self) -> bool {
        let next = !self.state.global_enabled.load(Ordering::Acquire);
        self.state.global_enabled.store(next, Ordering::Release);
        // Gửi giá trị TUYỆT ĐỐI (không phải "đảo"): nếu tray và process này lệch nhau
        // (tray vừa khởi động lại, broadcast chưa tới) thì "đảo" ở tray cho kết quả
        // ngược với cái người dùng vừa thấy.
        {
            let state = std::sync::Arc::clone(&self.state);
            let app_id = self.app_id.clone();
            std::thread::spawn(move || {
                // ERROR_PIPE_BUSY (hết instance khi tray bận): WaitNamedPipe rồi thử
                // lại — bỏ qua nghĩa là tin toggle không tới, tray lệch state với
                // process này tới snapshot kế.
                //
                // Vòng 16 (audit A7): thử pipe phiên → legacy; xác thực server là
                // TextVN.exe (chống squatting).
                // R2-26: `open_tray_pipe` gồm cả pipe sandbox — Ctrl+Shift trong ô tìm kiếm
                // Start/ứng dụng Store giờ cũng tới tray và mọi app khác.
                for _ in 0..3 {
                    let mut stream = match open_tray_pipe() {
                        PipeOpen::Connected { file, .. } => file,
                        PipeOpen::Busy(name) => {
                            // BUSY → chờ instance rảnh (200 ms) rồi thử lại.
                            let name_w: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
                            // SAFETY: chuỗi UTF-16 kết thúc NUL sống tới hết lời gọi.
                            unsafe {
                                let _ = WaitNamedPipeW(windows::core::PCWSTR(name_w.as_ptr()), 200);
                            }
                            continue;
                        }
                        // Tray không chạy → bỏ (toggle hiếm, không chặn phím).
                        PipeOpen::Unavailable => return,
                    };
                    let _ = send_message(
                        &mut stream,
                        &Message::ToggleViEn {
                            app_id: GLOBAL_KEY.to_string(),
                            enabled: next,
                        },
                    );
                    // Tray trả SNAPSHOT state chuẩn (kể cả khi bị debounce bỏ qua):
                    // giá trị cục bộ `next` có thể sai so với truth — nhận snapshot
                    // để không diverge (báo cáo 0.2.9 "icon E mà vẫn gõ tiếng Việt").
                    if let Ok(Message::Snapshot {
                        config_version,
                        state: states,
                        ..
                    }) = read_next_message(&mut stream)
                    {
                        state
                            .config_version
                            .store(config_version, Ordering::Release);
                        state.apply_states(&app_id, &states);
                    }
                    return;
                }
            });
        }
        next
    }
}

/// Toast Trợ lý ngữ pháp lên tray — fire-and-forget trên thread nền (AS-022).
///
/// Gửi đúng API có sẵn: `Message::AssistNotification { title, message }`
/// (`ipc/src/lib.rs`, tray xử lý tại `tray/src/ipc_server.rs`) — **không** sửa
/// crate `ipc/`. Call quy ước xác thực server (`server_is_textvn`) như toggle.
///
/// Bất biến:
/// - Đường gõ **không bao giờ đợi pipe**: thread nền, 1 lần thử, fail-open —
///   tray không chạy / pipe bận → bỏ im lặng.
/// - Caller chịu trách nhiệm về nội dung `title`/`message`: **không chứa câu
///   người dùng** (S2) — `assist::notification_payload` chỉ khắc đã vậy.
pub fn notify_assist_async(title: String, message: String) {
    let spawn = std::thread::Builder::new()
        .name("textvn-assist-notify".into())
        .spawn(move || {
            // Tray không chạy → bỏ im lặng (fail-open). Sandbox (R2-26): tray không nhận
            // toast từ tiến trình sandbox — không gửi.
            let Some((mut stream, sandboxed)) = connect_tray() else {
                return;
            };
            let msg = Message::AssistNotification { title, message };
            if sandboxed && !allowed_in_sandbox(&msg) {
                return;
            }
            // Gửi xong lắng 1 Ack (tray trả `Message::Ack`) rồi đóng — đứt
            // nghĩa tray vừa thoát, bỏ im lặng.
            let _ = send_message(&mut stream, &msg);
            let _ = read_next_message(&mut stream);
        });
    // Spawn hỏng (hiếm, system thread pool sống): im lặng — toast là tối ưu, không bắt buộc.
    drop(spawn);
}

/// P7 (vòng 57): gửi JSON drain dữ liệu HỌC lên tray (fire-and-forget, pattern
/// `notify_assist_async`). Tray merge vào config + persist + bump version →
/// ConfigReload tự phát tới mọi adapter. JSON rỗng "{}" → không gửi.
/// G8: JSON chỉ chứa từ + cặp + count (engine drain đã đảm bảo).
pub fn send_style_update_async(json: String) {
    if json == "{}" || json.is_empty() {
        return;
    }
    let spawn = std::thread::Builder::new()
        .name("textvn-style-drain".into())
        .spawn(move || {
            let Some((mut stream, sandboxed)) = connect_tray() else {
                return; // tray không chạy → giữ pending (fail-open)
            };
            // R2-26 · G8: từ/cặp sửa là chữ người dùng gõ — trên kết nối sandbox chỉ gửi bộ
            // đếm Dream RSI (G19), tray cũng chỉ nhận phần đó.
            let json = if sandboxed {
                match textvn_config::sandbox::style_drain_rsi_only(&json) {
                    Some(rsi_only) => rsi_only,
                    None => return,
                }
            } else {
                json
            };
            let _ = send_message(&mut stream, &Message::StyleUpdate { json });
            let _ = read_next_message(&mut stream);
        });
    drop(spawn);
}

/// Gửi lệnh điều khiển một chiều (ToggleMath / CycleMathMode / OpenEmojiPicker) lên tray
/// trên thread nền (fire-and-forget, xác thực `server_is_textvn`, fail-open khi tray tắt).
pub fn send_command_async(msg: Message) {
    let spawn = std::thread::Builder::new()
        .name("textvn-tsf-cmd".into())
        .spawn(move || {
            // Sandbox (R2-26): tray không nhận lệnh Math/Emoji từ tiến trình sandbox.
            let Some((mut stream, sandboxed)) = connect_tray() else {
                return;
            };
            if sandboxed && !allowed_in_sandbox(&msg) {
                return;
            }
            let _ = send_message(&mut stream, &msg);
            let _ = read_next_message(&mut stream);
        });
    drop(spawn);
}

/// `%APPDATA%\TextVN\state.json` → map trạng thái (`"*"` = toàn cục).
/// Debounce phía TIP cho phím chuyển Ctrl+Shift — khớp cửa sổ 250ms phía tray
/// (`textvn_tray::try_claim_global_toggle`): một lần bấm được MỌI bên nhìn thấy
/// đúng MỘT lần (máy trạng thái modifier của CUAS có thể bắn key-up trùng).
pub static LAST_HOTKEY_TOGGLE_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Thử chiếm lượt toggle phía process này: `true` khi cách lần trước ≥ 250ms.
pub fn try_claim_hotkey_toggle() -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    crate::compose::claim_hotkey_toggle_at(&LAST_HOTKEY_TOGGLE_MS, now)
}

fn read_state_file() -> Option<BTreeMap<String, bool>> {
    // R2-26: tiến trình AppContainer bị từ chối `state.json` thật (lỗi 5) → bản sao sandbox;
    // chỉ khi token đúng là AppContainer (audit 0.2.93 — xem `reload_config_from_file`).
    let bytes = if textvn_ipc::sandbox::current_process_is_app_container() {
        textvn_ipc::sandbox::read_sandbox_file("state.json")?
    } else {
        let path = std::path::PathBuf::from(std::env::var_os("APPDATA")?)
            .join("TextVN")
            .join("state.json");
        std::fs::read(path).ok()?
    };
    parse_state_json(&bytes)
}

fn run_client_loop(app_id: String, state: Arc<IpcState>) {
    let pid = std::process::id();
    let mut backoff =
        ReconnectBackoff::new(Duration::from_millis(2_000), Duration::from_millis(10_000));

    loop {
        // Vòng 16 (audit A7): candidates + verify server; R2-26: + pipe sandbox.
        let Some((mut stream, _sandboxed)) = connect_tray() else {
            state.connected.store(false, Ordering::Release);
            // Tray tắt lâu: giãn nhịp thử lại để không đánh thức CPU vô ích.
            std::thread::sleep(backoff.next_delay());
            continue;
        };
        backoff.reset();

        let hello = Message::Hello {
            pid,
            abi: textvn_ffi::IME_ABI_VERSION,
            version: env!("CARGO_PKG_VERSION").into(),
        };
        if send_message(&mut stream, &hello).is_err()
            || send_message(&mut stream, &Message::Subscribe { pid }).is_err()
        {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }
        state.connected.store(true, Ordering::Release);

        // Thread daemon: chặn ở read là chủ đích; pipe vỡ (tray thoát) → thử lại.
        while let Ok(msg) = read_next_message(&mut stream) {
            match msg {
                Message::ConfigReload { version } => {
                    state.config_version.store(version, Ordering::Release);
                }
                Message::Snapshot {
                    config_version,
                    state: app_states,
                    ..
                } => {
                    state
                        .config_version
                        .store(config_version, Ordering::Release);
                    state.apply_states(&app_id, &app_states);
                }
                Message::StateUpdate {
                    app_id: update_app,
                    enabled,
                    ..
                } => state.apply_update(&app_id, &update_app, enabled),
                Message::Ping => {
                    let _ = send_message(&mut stream, &Message::Pong { uptime_ms: 0 });
                }
                _ => {}
            }
        }
        state.connected.store(false, Ordering::Release);
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn send_message<W: Write>(writer: &mut W, msg: &Message) -> std::io::Result<()> {
    let frame = encode_frame(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}")))?;
    writer.write_all(&frame)?;
    writer.flush()?;
    Ok(())
}

fn read_next_message<R: Read>(reader: &mut R) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let length = u32::from_le_bytes(len_buf) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Frame oversize",
        ));
    }

    let mut frame = vec![0u8; 4 + length];
    frame[..4].copy_from_slice(&len_buf);
    reader.read_exact(&mut frame[4..])?;

    decode_exact_frame(&frame)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    /// Claim lần đầu OK; trong 250ms claim lại phải chặn (khớp cửa sổ tray).
    /// Cell cục bộ + đồng hồ tiêm vào — không nhiễu với test song song.
    #[test]
    fn hotkey_toggle_claim_debounces() {
        let cell = std::sync::atomic::AtomicU64::new(0);
        let claim = |now: u64| {
            use std::sync::atomic::Ordering;
            let last = cell.load(Ordering::Acquire);
            if now.saturating_sub(last) < 250 {
                return false;
            }
            cell.compare_exchange(last, now, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        };
        assert!(claim(1_000));
        assert!(!claim(1_100));
        assert!(claim(1_300));
    }

    use super::*;

    fn map(pairs: &[(&str, bool)]) -> BTreeMap<String, bool> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn global_switch_from_tray_disables_every_app() {
        let client = IpcClient::detached();
        assert!(client.is_enabled());
        client.state.apply_update("notepad.exe", GLOBAL_KEY, false);
        assert!(!client.is_enabled(), "tắt toàn cục phải có hiệu lực ở TSF");
        client.state.apply_update("notepad.exe", GLOBAL_KEY, true);
        assert!(client.is_enabled());
    }

    #[test]
    fn per_app_override_only_disables_its_app() {
        let client = IpcClient::detached();
        client
            .state
            .apply_update("notepad.exe", "chrome.exe", false);
        assert!(client.is_enabled(), "override của app khác không ảnh hưởng");
        client
            .state
            .apply_update("notepad.exe", "NOTEPAD.EXE", false);
        assert!(!client.is_enabled());
    }

    #[test]
    fn snapshot_restores_global_and_clears_stale_override() {
        let client = IpcClient::detached();
        client
            .state
            .apply_states("code.exe", &map(&[("*", true), ("code.exe", false)]));
        assert!(!client.is_enabled());
        client.state.apply_states("code.exe", &map(&[("*", true)]));
        assert!(client.is_enabled(), "snapshot mới không còn override");
        client.state.apply_states("code.exe", &map(&[("*", false)]));
        assert!(!client.is_enabled());
    }

    #[test]
    fn local_toggle_flips_immediately_without_tray() {
        let client = IpcClient::detached();
        assert!(!client.toggle_global());
        assert!(!client.is_enabled());
        assert!(client.toggle_global());
        assert!(client.is_enabled());
    }

    #[test]
    fn state_file_parsing_matches_tray_schema() {
        let parsed = parse_state_json(
            br#"{"global_enabled":false,"assist_enabled":true,"apps":{"Zalo.exe":true,"bad":"x"}}"#,
        )
        .unwrap();
        assert_eq!(parsed.get("*"), Some(&false));
        assert_eq!(parsed.get("assist"), Some(&true));
        assert_eq!(parsed.get("zalo.exe"), Some(&true));
        assert!(!parsed.contains_key("bad"));
        assert!(parse_state_json(b"not json").is_none());
    }

    #[test]
    fn assist_state_update_and_snapshot() {
        let client = IpcClient::detached();
        // Vòng 29: mặc định BẬT (khớp tray state.json + config default) —
        // trước đây false khiến trợ lý chết hoàn toàn khi tray chưa snapshot.
        assert!(client.is_assist_enabled());
        client
            .state
            .apply_states("notepad.exe", &map(&[("assist", false)]));
        assert!(!client.is_assist_enabled());
        client.state.apply_update("notepad.exe", "assist", true);
        assert!(client.is_assist_enabled());
    }
}
