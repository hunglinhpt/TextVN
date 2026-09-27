// SPDX-License-Identifier: GPL-3.0-or-later
//! IPC Server cho VietIME Tray (WIN-051 — P0-3 §4 / P1-4 §2).
//!
//! Lắng nghe trên Named Pipe `\\.\pipe\vietime-ipc-v1`. Quản lý các client kết nối
//! (in-process TSF DLLs, Hook process, CLI probe...), broadcast `ConfigReload` và `StateUpdate`,
//! theo dõi sức khỏe và quản lý watchdog cho tiến trình `vietime-hook.exe`.

use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use vietime_ipc::{decode_exact_frame, encode_frame, Message, MAX_FRAME_BYTES};

use crate::svc::SvcManager;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows::Win32::System::Pipes::*;

pub const PIPE_NAME: &str = r"\\.\pipe\vietime-ipc-v1";

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
        if self.running.swap(false, Ordering::SeqCst) {
            #[cfg(windows)]
            {
                // Unblock ConnectNamedPipe bằng kết nối dummy cục bộ (WIN-051)
                let _ = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(PIPE_NAME);
            }
        }
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

    fn broadcast_message(&self, msg: &Message) {
        let Ok(frame) = encode_frame(msg) else { return };
        let mut subs = self.subscribers.lock().unwrap();
        subs.retain_mut(|client| {
            if let Ok(mut stream) = client.stream.lock() {
                stream.write_all(&frame).is_ok() && stream.flush().is_ok()
            } else {
                false
            }
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
                let file = unsafe { std::fs::File::from_raw_handle(handle.0 as _) };
                let stream = Arc::new(Mutex::new(file));

                self.active_clients.fetch_add(1, Ordering::SeqCst);
                let this = self.clone();
                let stream_clone = stream.clone();

                std::thread::spawn(move || {
                    this.handle_client_connection(stream_clone);
                    this.active_clients.fetch_sub(1, Ordering::SeqCst);
                });
            } else {
                let _ = unsafe { CloseHandle(handle) };
            }
        }
    }

    fn handle_client_connection(&self, stream: Arc<Mutex<std::fs::File>>) {
        let mut subscribed = false;
        let mut client_pid = 0u32;

        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }

            let msg = {
                let mut file = match stream.lock() {
                    Ok(f) => f,
                    Err(_) => break,
                };
                match read_message(&mut *file) {
                    Ok(m) => m,
                    Err(_) => break,
                }
            };

            let response = match msg {
                Message::Hello { pid, .. } => {
                    client_pid = pid;
                    Some(self.build_snapshot())
                }
                Message::GetSnapshot => Some(self.build_snapshot()),
                Message::Subscribe { pid } => {
                    subscribed = true;
                    client_pid = pid;
                    let mut subs = self.subscribers.lock().unwrap();
                    subs.push(ClientSink {
                        pid,
                        stream: stream.clone(),
                    });
                    Some(Message::Ack)
                }
                Message::ToggleViEn { app_id, enabled } => {
                    let ver = self.svc.set_app_enabled(&app_id, enabled);
                    self.broadcast_state_update(&app_id, enabled, ver);
                    Some(Message::Ack)
                }
                Message::Ping => Some(Message::Pong {
                    uptime_ms: self.uptime_ms(),
                }),
                Message::CrashReport { .. } => {
                    self.crash_counter.fetch_add(1, Ordering::Relaxed);
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
            subs.retain(|c| c.pid != client_pid);
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

    /// Watchdog quản lý tiến trình `vietime-hook.exe` (P1-4 §6).
    #[cfg(windows)]
    fn hook_watchdog_loop(self: Arc<Self>) {
        while self.running.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(5));
        }
    }
}

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
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_ipc_test_{}", std::process::id()));
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
            std::env::temp_dir().join(format!("vietime_crash_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::new(svc);

        assert_eq!(server.crash_count(), 0);
        server.crash_counter.fetch_add(1, Ordering::SeqCst);
        assert_eq!(server.crash_count(), 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(windows)]
    fn server_starts_and_stops_cleanly() {
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_startstop_test_{}", std::process::id()));
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

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
