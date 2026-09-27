// SPDX-License-Identifier: GPL-3.0-or-later
//! Non-blocking, offline-tolerant IPC client cho VietIME TSF (WIN-016).
//!
//! Kết nối tới `\\.\pipe\vietime-ipc-v1`. Nếu Tray chưa bật hoặc pipe không mở,
//! client fail-open ngay lập tức (offline-tolerant), không bao giờ block STA thread
//! của client app. Khi nhận `ConfigReload`, thông báo để engine reload config.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use vietime_ipc::{decode_exact_frame, encode_frame, Message, MAX_FRAME_BYTES};

pub const PIPE_NAME: &str = r"\\.\pipe\vietime-ipc-v1";

/// Trạng thái IPC được chia sẻ giữa background thread và STA thread của TSF.
#[derive(Debug)]
pub struct IpcState {
    pub connected: AtomicBool,
    pub latest_config_version: AtomicU64,
    pub last_handled_config_version: AtomicU64,
    pub app_enabled: AtomicBool,
    pub has_app_override: AtomicBool,
}

impl Default for IpcState {
    fn default() -> Self {
        Self {
            connected: AtomicBool::new(false),
            latest_config_version: AtomicU64::new(0),
            last_handled_config_version: AtomicU64::new(0),
            app_enabled: AtomicBool::new(true),
            has_app_override: AtomicBool::new(false),
        }
    }
}

/// Handle quản lý IPC client gắn với vòng đời của Tip.
pub struct IpcClient {
    state: Arc<IpcState>,
    stop_signal: Arc<AtomicBool>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl IpcClient {
    /// Bắt đầu IPC client nền. Nếu Tray không chạy, tiến trình nền tự động
    /// ngủ và thử lại theo chu kỳ dài (3s), tuyệt đối không block luồng gọi.
    pub fn start(app_id: String) -> Self {
        let state = Arc::new(IpcState::default());
        let stop_signal = Arc::new(AtomicBool::new(false));

        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop_signal);

        let worker = std::thread::Builder::new()
            .name("vietime-tsf-ipc".into())
            .spawn(move || {
                run_client_loop(app_id, worker_state, worker_stop);
            })
            .ok();

        Self {
            state,
            stop_signal,
            worker: Mutex::new(worker),
        }
    }

    /// Trả về true nếu pipe đang kết nối tới Tray UI.
    pub fn is_connected(&self) -> bool {
        self.state.connected.load(Ordering::Acquire)
    }

    /// Kiểm tra xem có cấu hình mới cần nạp vào engine không.
    /// Nếu có, trả về version mới và đánh dấu đã tiêu thụ.
    pub fn check_config_reload(&self) -> Option<u64> {
        let latest = self.state.latest_config_version.load(Ordering::Acquire);
        let handled = self
            .state
            .last_handled_config_version
            .load(Ordering::Acquire);
        if latest > handled {
            self.state
                .last_handled_config_version
                .store(latest, Ordering::Release);
            Some(latest)
        } else {
            None
        }
    }

    /// Trạng thái bật/tắt do Tray UI chỉ định cho app hiện tại (nếu có override).
    pub fn app_enabled_override(&self) -> Option<bool> {
        if self.state.has_app_override.load(Ordering::Acquire) {
            Some(self.state.app_enabled.load(Ordering::Acquire))
        } else {
            None
        }
    }

    /// Dừng client và ngắt kết nối an toàn.
    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::Release);
        if let Ok(mut lock) = self.worker.lock() {
            if let Some(handle) = lock.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_client_loop(app_id: String, state: Arc<IpcState>, stop: Arc<AtomicBool>) {
    let pid = std::process::id();

    while !stop.load(Ordering::Acquire) {
        // Cố gắng mở named pipe (read + write)
        match OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
            Ok(mut stream) => {
                state.connected.store(true, Ordering::Release);

                // Gửi Hello
                let hello = Message::Hello {
                    pid,
                    abi: vietime_ffi::IME_ABI_VERSION,
                    version: env!("CARGO_PKG_VERSION").into(),
                };
                if send_message(&mut stream, &hello).is_err() {
                    state.connected.store(false, Ordering::Release);
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }

                // Gửi Subscribe
                let sub = Message::Subscribe { pid };
                if send_message(&mut stream, &sub).is_err() {
                    state.connected.store(false, Ordering::Release);
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }

                // Vòng lặp nhận thông điệp từ Tray
                while !stop.load(Ordering::Acquire) {
                    match read_next_message(&mut stream) {
                        Ok(msg) => match msg {
                            Message::ConfigReload { version } => {
                                state
                                    .latest_config_version
                                    .store(version, Ordering::Release);
                            }
                            Message::Snapshot {
                                config_version,
                                state: app_states,
                                ..
                            } => {
                                state
                                    .latest_config_version
                                    .store(config_version, Ordering::Release);
                                if let Some(&enabled) = app_states.get(&app_id) {
                                    state.app_enabled.store(enabled, Ordering::Release);
                                    state.has_app_override.store(true, Ordering::Release);
                                }
                            }
                            Message::StateUpdate {
                                app_id: ref update_app,
                                enabled,
                                ..
                            } => {
                                if update_app == &app_id {
                                    state.app_enabled.store(enabled, Ordering::Release);
                                    state.has_app_override.store(true, Ordering::Release);
                                }
                            }
                            Message::Ping => {
                                let pong = Message::Pong { uptime_ms: 0 };
                                let _ = send_message(&mut stream, &pong);
                            }
                            _ => {}
                        },
                        Err(_) => {
                            // Mất kết nối hoặc Tray thoát
                            break;
                        }
                    }
                }

                state.connected.store(false, Ordering::Release);
            }
            Err(_) => {
                // Pipe chưa sẵn sàng: ngủ 2 giây rồi thử lại
                state.connected.store(false, Ordering::Release);
                for _ in 0..20 {
                    if stop.load(Ordering::Acquire) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
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
    use super::*;

    #[test]
    fn ipc_client_offline_tolerant_starts_and_stops_cleanly() {
        let client = IpcClient::start("notepad.exe".into());
        // Không có tray server: connected = false, nhưng không block, không crash
        assert!(!client.is_connected());
        assert_eq!(client.check_config_reload(), None);
        assert_eq!(client.app_enabled_override(), None);
        client.stop();
    }

    #[test]
    fn ipc_state_reload_detection() {
        let state = IpcState::default();
        assert_eq!(state.latest_config_version.load(Ordering::Acquire), 0);
        state.latest_config_version.store(42, Ordering::Release);

        let client = IpcClient {
            state: Arc::new(state),
            stop_signal: Arc::new(AtomicBool::new(false)),
            worker: Mutex::new(None),
        };

        assert_eq!(client.check_config_reload(), Some(42));
        // Lần thứ hai không báo lại phiên bản cũ
        assert_eq!(client.check_config_reload(), None);
    }
}
