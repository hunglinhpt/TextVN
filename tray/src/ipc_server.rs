// SPDX-License-Identifier: GPL-3.0-or-later
//! IPC Server cho TextVN Tray (WIN-051 — P0-3 §4 / P1-4 §2).
//!
//! Lắng nghe trên Named Pipe `\\.\pipe\textvn-ipc-v1`. Quản lý các client kết nối
//! (in-process TSF DLLs, Hook process, CLI probe...), broadcast `ConfigReload` và `StateUpdate`,
//! theo dõi sức khỏe và quản lý watchdog cho tiến trình `textvn-hook.exe`.

#[cfg(any(windows, test))]
use std::io::Read;
use std::io::Write;
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

#[cfg(any(windows, test))]
use textvn_ipc::{decode_exact_frame, MAX_FRAME_BYTES};
use textvn_ipc::{encode_frame, Message};

use crate::svc::SvcManager;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows::Win32::System::Pipes::*;

pub const PIPE_NAME: &str = r"\\.\pipe\textvn-ipc-v1";

/// Ghi frame tới toàn bộ subscriber đang Subscribe. Xóa subscriber ghi lỗi
/// (client chết / pipe vỡ) khỏi danh sách.
///
/// **Bẫy deadlock đã repro (2026-09):** một `ReadFile` sync đang pending trên
/// kernel file object của pipe sẽ serialize chặn mọi `WriteFile` trên handle
/// dup (`try_clone`) cùng object — kể cả khi buffer rỗng và client đang chờ
/// đọc. Nên handler KHÔNG BAO GIỜ được block trong `read_message` khi còn
/// subscriber cần broadcast: đọc phải được gate bằng `PeekNamedPipe`
/// (xem `try_read_pipe_frame`).
fn write_frame_to_subscribers(subscribers: Arc<Mutex<Vec<ClientSink>>>, frame: Vec<u8>) {
    let Ok(mut subs) = subscribers.lock() else {
        return;
    };
    subs.retain_mut(|client| {
        let Ok(mut stream) = client.stream.lock() else {
            return false;
        };
        stream.write_all(&frame).is_ok() && stream.flush().is_ok()
    });
}

/// Trạng thái đọc một khung từ pipe **không bao giờ block vô hạn**.
#[cfg(windows)]
enum PipeFrame {
    /// Chưa đủ byte — ngủ poll rồi thử lại.
    Empty,
    Frame(Message),
    /// Pipe vỡ / dữ liệu hỏng framing.
    Eof,
}

/// Số byte đang buffer trong pipe, hoặc `None` nếu pipe đã vỡ.
#[cfg(windows)]
fn peek_available(reader: &std::fs::File) -> Option<u32> {
    let mut total = 0u32;
    let ok = unsafe {
        PeekNamedPipe(
            HANDLE(reader.as_raw_handle()),
            None,
            0,
            None,
            Some(&mut total),
            None,
        )
    };
    ok.ok().map(|_| total)
}

/// Đọc một khung IPC theo kiểu poll: chỉ gọi `ReadFile` khi `PeekNamedPipe`
/// xác nhận đủ byte, nên handler không bao giờ giữ read-pending trên pipe —
/// nhờ đó thread broadcast ghi được song song (xem `write_frame_to_subscribers`).
#[cfg(windows)]
fn try_read_pipe_frame(reader: &std::fs::File, running: &AtomicBool) -> PipeFrame {
    let Some(avail) = peek_available(reader) else {
        return PipeFrame::Eof;
    };
    if avail < 4 {
        return PipeFrame::Empty;
    }

    // 4 byte length đã có sẵn → read_exact trả về ngay, không pending.
    let mut conn = reader;
    let mut len_buf = [0u8; 4];
    if conn.read_exact(&mut len_buf).is_err() {
        return PipeFrame::Eof;
    }
    let length = u32::from_le_bytes(len_buf) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return PipeFrame::Eof;
    }

    // Payload chưa đủ: poll tiếp mà KHÔNG giữ read pending.
    loop {
        if !running.load(Ordering::Acquire) {
            return PipeFrame::Eof;
        }
        let Some(avail) = peek_available(reader) else {
            return PipeFrame::Eof;
        };
        if avail as usize >= length {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    let mut frame = vec![0u8; 4 + length];
    frame[..4].copy_from_slice(&len_buf);
    if conn.read_exact(&mut frame[4..]).is_err() {
        return PipeFrame::Eof;
    }
    match decode_exact_frame(&frame) {
        Ok(m) => PipeFrame::Frame(m),
        Err(_) => PipeFrame::Eof,
    }
}

/// Kênh gửi message broadcast tới một client đã Subscribe.
struct ClientSink {
    pid: u32,
    stream: Arc<Mutex<std::fs::File>>,
}

pub struct IpcServer {
    svc: Arc<SvcManager>,
    start_time: Instant,
    running: Arc<AtomicBool>,
    subscribers: Arc<Mutex<Vec<ClientSink>>>,
    crash_counter: AtomicU32,
    active_clients: AtomicU32,
    hook_pid: AtomicU32,
}

impl IpcServer {
    pub fn new(svc: Arc<SvcManager>) -> Arc<Self> {
        Arc::new(Self {
            svc,
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            crash_counter: AtomicU32::new(0),
            active_clients: AtomicU32::new(0),
            hook_pid: AtomicU32::new(0),
        })
    }

    pub fn uptime_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn active_clients(&self) -> u32 {
        self.active_clients.load(Ordering::Acquire)
    }

    pub fn crash_count(&self) -> u32 {
        self.crash_counter.load(Ordering::Acquire)
    }

    pub fn hook_pid(&self) -> u32 {
        self.hook_pid.load(Ordering::Acquire)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn stop(&self) {
        // Do not open a synchronous "dummy" pipe connection here. `CreateFile`
        // can wait indefinitely when no instance is currently available, which
        // used to deadlock tray shutdown before it could release the singleton
        // mutex. The server thread observes this flag; process teardown closes
        // its pending pipe handle without blocking the UI thread.
        self.running.store(false, Ordering::SeqCst);
    }

    /// Broadcast thông báo cấu hình thay đổi tới toàn bộ client đang Subscribe (P0-3 §4).
    pub fn broadcast_config_reload(&self, version: u64) {
        let msg = Message::ConfigReload { version };
        self.broadcast_message(&msg);
    }

    /// Broadcast cập nhật trạng thái bật/tắt tới toàn bộ client.
    pub fn broadcast_state_update(&self, app_id: &str, enabled: bool, version: u64) {
        let msg = Message::StateUpdate {
            app_id: app_id.to_string(),
            enabled,
            version,
        };
        self.broadcast_message(&msg);
    }

    /// Broadcast thông báo tắt ứng dụng tới toàn bộ client (Hook, TSF).
    ///
    /// Khác với các broadcast thường, lần này đợi worker ghi xong frame (tối đa
    /// 200ms) trước khi trả về: ngay sau đó process sẽ thoát, mà thread vừa
    /// spawn nếu bị kill trước khi kịp flush thì Hook không bao giờ nhận được
    /// Shutdown và thành mồ côi tới tận heartbeat timeout.
    pub fn broadcast_shutdown(&self) {
        let msg = Message::Shutdown;
        let Ok(frame) = encode_frame(&msg) else {
            return;
        };
        let subscribers = self.subscribers.clone();
        let worker = std::thread::spawn(move || {
            write_frame_to_subscribers(subscribers, frame);
        });
        let deadline = Instant::now() + Duration::from_millis(200);
        while !worker.is_finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        // Worker kẹt (client chết chiếm pipe buffer) thì detach — đã chờ đủ.
    }

    fn broadcast_message(&self, msg: &Message) {
        let Ok(frame) = encode_frame(msg) else { return };
        let subscribers = self.subscribers.clone();

        // Named-pipe writes are synchronous and a subscriber can stop consuming
        // at any point.  Never perform one on the tray window thread: doing so
        // would turn a stale Hook/TSF client into a frozen Exit/Settings UI.
        std::thread::spawn(move || {
            write_frame_to_subscribers(subscribers, frame);
        });
    }

    /// Bắt đầu pipe server loop trên một background thread.
    #[cfg(windows)]
    pub fn start(self: &Arc<Self>) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let this = self.clone();
        std::thread::spawn(move || {
            this.server_accept_loop();
        });

        // Khởi động watchdog giám sát hook process (WIN-040, WIN-051 §6)
        let this_watchdog = self.clone();
        std::thread::spawn(move || {
            this_watchdog.hook_watchdog_loop();
        });
    }

    #[cfg(windows)]
    fn server_accept_loop(self: Arc<Self>) {
        let pipe_name_wide: Vec<u16> = PIPE_NAME.encode_utf16().chain(Some(0)).collect();

        while self.running.load(Ordering::Acquire) {
            // SAFETY: Tạo named pipe instance
            let handle = unsafe {
                CreateNamedPipeW(
                    PCWSTR(pipe_name_wide.as_ptr()),
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    PIPE_UNLIMITED_INSTANCES,
                    MAX_FRAME_BYTES as u32,
                    MAX_FRAME_BYTES as u32,
                    0,
                    None,
                )
            };

            if handle.is_invalid() {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            // SAFETY: Đợi client kết nối
            let connected = unsafe { ConnectNamedPipe(handle, None) };
            if !self.running.load(Ordering::Acquire) {
                let _ = unsafe { CloseHandle(handle) };
                break;
            }
            if connected.is_ok() || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED {
                let Some(peer_pid) = connected_client_pid(handle) else {
                    let _ = unsafe { CloseHandle(handle) };
                    continue;
                };
                let reader = unsafe { std::fs::File::from_raw_handle(handle.0 as _) };
                // Một client đọc thường chờ vô hạn trong `read_exact`. Writer phải
                // là handle riêng để broadcast Shutdown/StateUpdate không bị giữ
                // mutex bởi reader và làm treo UI thread lúc thoát ứng dụng.
                let writer = match reader.try_clone() {
                    Ok(file) => file,
                    Err(_) => continue,
                };
                let stream = Arc::new(Mutex::new(writer));

                self.active_clients.fetch_add(1, Ordering::SeqCst);
                let this = self.clone();
                let stream_clone = stream.clone();

                std::thread::spawn(move || {
                    this.handle_client_connection(reader, stream_clone, peer_pid);
                    this.active_clients.fetch_sub(1, Ordering::SeqCst);
                });
            } else {
                let _ = unsafe { CloseHandle(handle) };
            }
        }
    }

    #[cfg(windows)]
    fn handle_client_connection(
        &self,
        reader: std::fs::File,
        stream: Arc<Mutex<std::fs::File>>,
        peer_pid: u32,
    ) {
        let mut subscribed = false;

        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }

            // Đọc poll-gated: không bao giờ block vô hạn với read-pending trên
            // pipe (read-pending serialize chặn broadcast write → hook mồ côi).
            let msg = match try_read_pipe_frame(&reader, &self.running) {
                PipeFrame::Empty => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                PipeFrame::Eof => break,
                PipeFrame::Frame(m) => m,
            };

            // PID trong wire protocol chỉ để chẩn đoán. Không bao giờ tin giá trị
            // do client tự khai báo: lấy PID từ named-pipe kernel handle để tránh
            // client giả mạo subscriber khác rồi gỡ nhầm subscription của họ.
            match &msg {
                Message::Hello { pid, .. } | Message::Subscribe { pid } if *pid != peer_pid => {
                    break;
                }
                _ => {}
            }

            let response = match msg {
                Message::Hello { pid, .. } => {
                    debug_assert_eq!(pid, peer_pid);
                    Some(self.build_snapshot())
                }
                Message::GetSnapshot => Some(self.build_snapshot()),
                Message::Subscribe { pid } => {
                    subscribed = true;
                    debug_assert_eq!(pid, peer_pid);
                    let mut subs = self.subscribers.lock().unwrap();
                    subs.push(ClientSink {
                        pid: peer_pid,
                        stream: stream.clone(),
                    });
                    Some(Message::Ack)
                }
                Message::ToggleViEn { app_id, enabled } => {
                    let ver = if app_id == "*" {
                        let (_, v) = self.svc.set_global_enabled(enabled);
                        crate::notify_tray_state_changed();
                        v
                    } else {
                        self.svc.set_app_enabled(&app_id, enabled)
                    };
                    self.broadcast_state_update(&app_id, enabled, ver);
                    Some(Message::Ack)
                }
                Message::ToggleGlobal => {
                    let (enabled, ver) = self.svc.toggle_global_enabled();
                    self.broadcast_state_update("*", enabled, ver);
                    crate::notify_tray_state_changed();
                    Some(Message::Ack)
                }
                Message::Ping => Some(Message::Pong {
                    uptime_ms: self.uptime_ms(),
                }),
                Message::CrashReport { .. } => {
                    self.crash_counter.fetch_add(1, Ordering::Relaxed);
                    Some(Message::Ack)
                }
                Message::Shutdown => {
                    crate::notify_tray_exit_requested();
                    Some(Message::Ack)
                }
                _ => None,
            };

            if let Some(resp) = response {
                if let Ok(mut file) = stream.lock() {
                    if let Ok(frame) = encode_frame(&resp) {
                        if file.write_all(&frame).is_err() || file.flush().is_err() {
                            break;
                        }
                    }
                }
            }
        }

        if subscribed {
            let mut subs = self.subscribers.lock().unwrap();
            subs.retain(|c| c.pid != peer_pid);
        }
    }

    fn build_snapshot(&self) -> Message {
        Message::Snapshot {
            config_version: self.svc.config_version(),
            state: self.svc.app_states(),
            appdb_version: 1,
            channel: "stable".into(),
        }
    }

    /// Watchdog quản lý tiến trình `textvn-hook.exe` (P1-4 §6).
    #[cfg(windows)]
    fn hook_watchdog_loop(self: Arc<Self>) {
        while self.running.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(5));
        }
    }
}

/// PID được kernel liên kết với đầu client của named pipe. Đây là bằng chứng
/// cục bộ đáng tin cậy hơn các trường `pid` trong IPC message.
#[cfg(windows)]
fn connected_client_pid(pipe: HANDLE) -> Option<u32> {
    let mut pid = 0u32;
    // SAFETY: `pipe` là server handle còn mở sau ConnectNamedPipe và `pid` là
    // vùng nhớ ghi hợp lệ.
    unsafe { GetNamedPipeClientProcessId(pipe, &mut pid) }
        .ok()
        .and((pid != 0).then_some(pid))
}

#[cfg(test)]
fn read_message<R: Read>(reader: &mut R) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let length = u32::from_le_bytes(len_buf) as usize;

    if length > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Frame size exceeds maximum allowed bytes",
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
    use super::*;

    #[test]
    fn server_builds_valid_snapshot() {
        let temp_dir = std::env::temp_dir().join(format!("textvn_ipc_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::new(svc.clone());

        svc.set_app_enabled("chrome.exe", false);

        let snap = server.build_snapshot();
        match snap {
            Message::Snapshot {
                config_version,
                state,
                appdb_version,
                channel,
            } => {
                assert_eq!(config_version, svc.config_version());
                assert_eq!(state.get("chrome.exe"), Some(&false));
                assert_eq!(appdb_version, 1);
                assert_eq!(channel, "stable");
            }
            _ => panic!("Expected Snapshot message"),
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn frame_encoding_and_read_message_roundtrip() {
        let msg = Message::Ping;
        let frame = encode_frame(&msg).unwrap();
        let mut cursor = std::io::Cursor::new(frame);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, Message::Ping);

        let reload_msg = Message::ConfigReload { version: 42 };
        let frame2 = encode_frame(&reload_msg).unwrap();
        let mut cursor2 = std::io::Cursor::new(frame2);
        let decoded2 = read_message(&mut cursor2).unwrap();
        assert_eq!(decoded2, Message::ConfigReload { version: 42 });
    }

    #[test]
    fn crash_counter_increments_atomically() {
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_crash_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::new(svc);

        assert_eq!(server.crash_count(), 0);
        server.crash_counter.fetch_add(1, Ordering::SeqCst);
        assert_eq!(server.crash_count(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[cfg(windows)]
    static TEST_PIPE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    #[cfg(windows)]
    fn server_starts_and_stops_cleanly() {
        let _guard = TEST_PIPE_LOCK.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_startstop_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::new(svc);

        assert!(!server.is_running());
        server.start();
        assert!(server.is_running());

        // Cho pipe loop vao ConnectNamedPipe
        std::thread::sleep(Duration::from_millis(50));

        // Stop phai unblock va chuyen is_running ve false sach se
        server.stop();
        assert!(!server.is_running());
        // Unblock pending ConnectNamedPipe so the worker thread observes running=false and terminates
        let _ = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(PIPE_NAME);
        std::thread::sleep(Duration::from_millis(50));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Broadcast phải thực sự tới được subscriber qua named pipe.
    /// Regression: WIP shutdown từng gửi Ack nhưng frame broadcast không tới
    /// client khiến hook mồ côi sau khi tray thoát.
    #[test]
    #[cfg(windows)]
    fn broadcast_reaches_subscriber() {
        let _guard = TEST_PIPE_LOCK.lock().unwrap();
        use std::io::Write as _;

        let temp_dir =
            std::env::temp_dir().join(format!("textvn_bcast_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::new(svc);
        server.start();
        std::thread::sleep(Duration::from_millis(100));

        let (tx, rx) = std::sync::mpsc::channel();
        let client = std::thread::spawn(move || {
            let mut file = None;
            for _ in 0..20 {
                if let Ok(f) = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(PIPE_NAME)
                {
                    file = Some(f);
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            let mut file = file.expect("client pipe open");
            let pid = std::process::id();
            let sub = encode_frame(&Message::Subscribe { pid }).unwrap();
            file.write_all(&sub).unwrap();
            file.flush().unwrap();

            let ack = read_message(&mut file).expect("ack frame");
            tx.send(ack).unwrap();
            let update = read_message(&mut file).expect("broadcast frame");
            tx.send(update).unwrap();
        });

        let ack = rx
            .recv_timeout(Duration::from_secs(3))
            .expect("Ack within 3s");
        assert_eq!(ack, Message::Ack);

        server.broadcast_state_update("*", false, 2);
        let update = rx
            .recv_timeout(Duration::from_secs(3))
            .expect("StateUpdate broadcast within 3s");
        match update {
            Message::StateUpdate {
                app_id,
                enabled,
                version,
            } => {
                assert_eq!(app_id, "*");
                assert!(!enabled);
                assert_eq!(version, 2);
            }
            other => panic!("expected StateUpdate, got {other:?}"),
        }

        client.join().unwrap();
        server.stop();
        let _ = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(PIPE_NAME);
        std::thread::sleep(Duration::from_millis(50));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
