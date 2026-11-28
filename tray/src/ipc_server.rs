// SPDX-License-Identifier: GPL-3.0-or-later
//! IPC Server cho TextVN Tray (WIN-051 — P0-3 §4 / P1-4 §2).
//!
//! Lắng nghe trên Named Pipe theo phiên `\\.\pipe\textvn-ipc-v1-s<phiên>` và tên cũ
//! `\\.\pipe\textvn-ipc-v1` (R2-109 — DLL TSF bản cũ còn nạp). Quản lý các client kết nối
//! (in-process TSF DLLs, Hook process, CLI probe...), broadcast `ConfigReload` và `StateUpdate`,
//! theo dõi sức khỏe và quản lý watchdog cho tiến trình `textvn-hook.exe`.

use std::collections::VecDeque;
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
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
#[cfg(windows)]
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows::Win32::System::Pipes::*;

pub const PIPE_NAME: &str = textvn_ipc::PIPE_NAME;

/// SEC-05 (audit 2026-10-04): SECURITY_ATTRIBUTES cho named pipe với DACL chỉ
/// cấp quyền cho user hiện tại — pipe không DACL là mở cho MỌI user cùng máy
/// (kể cả phiên RDP khác) đọc ConfigReload/StateUpdate và gửi Shutdown giả.
/// R2-26: `sandbox = true` = pipe sandbox — thêm đọc/ghi cho AppContainer + LPAC và nhãn toàn
/// vẹn Low (`textvn_ipc::sandbox::pipe_sddl`); mọi client của pipe này bị hạn chế
/// (`sandbox_policy`). Pipe đầy đủ giữ nguyên DACL cũ — sandbox KHÔNG mở được nó.
/// Trả None khi API lỗi (rơi về DACL mặc định — pipe vẫn chạy, chỉ kém chặt).
#[cfg(windows)]
fn pipe_security(sandbox: bool) -> Option<(SECURITY_ATTRIBUTES, PSECURITY_DESCRIPTOR)> {
    let sid = textvn_ipc::sandbox::current_user_sid_string()?;
    let sddl = HSTRING::from(textvn_ipc::sandbox::pipe_sddl(&sid, sandbox));
    // SAFETY: SD trả cho caller LocalFree SAU khi CreateNamedPipeW đã sao chép descriptor
    // vào kernel.
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None)
            .ok()?;
        Some((
            SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: sd.0,
                bInheritHandle: FALSE,
            },
            sd,
        ))
    }
}

/// R2-26 — tập lệnh sandbox dùng chung 1 nguồn từ `textvn_ipc::client` (khớp với client).
#[cfg(any(windows, test))]
pub(crate) use textvn_ipc::client::{sandbox_policy, SandboxPolicy};

/// R2-26 — chống làm nghẽn tray từ pipe sandbox (mọi ứng dụng Store của người dùng nối được):
/// tối đa ngần này kết nối sandbox cùng lúc (mỗi kết nối một thread) …
pub(crate) const SANDBOX_MAX_CONNECTIONS: u32 = 32;
/// … và mỗi kết nối tối đa ngần này frame trong một cửa sổ 1 giây (TIP thật gửi vài frame lúc
/// nối + mỗi lần Ctrl+Shift); vượt ⇒ đóng kết nối (spam `ToggleViEn` từng nhân broadcast).
pub(crate) const SANDBOX_MAX_FRAMES_PER_SEC: u32 = 40;

/// Bộ đếm frame theo cửa sổ 1 giây cho một kết nối sandbox. Thuần (đồng hồ truyền vào).
#[derive(Debug)]
pub(crate) struct FrameBudget {
    window_start: Instant,
    used: u32,
}

impl FrameBudget {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            window_start: now,
            used: 0,
        }
    }

    /// `false` = vượt hạn mức trong cửa sổ hiện tại.
    pub(crate) fn take(&mut self, now: Instant) -> bool {
        if now.saturating_duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.used = 0;
        }
        self.used += 1;
        self.used <= SANDBOX_MAX_FRAMES_PER_SEC
    }
}

/// Client thấy được những khoá trạng thái nào.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Visibility {
    /// Client pipe đầy đủ — mọi khoá.
    All,
    /// Client pipe sandbox — chỉ `*`, `assist` và override của CHÍNH app nó (không lộ danh
    /// sách app người dùng đã tắt cho ứng dụng Store khác).
    Sandbox { own_app: Option<String> },
}

impl Visibility {
    pub(crate) fn sees(&self, key: &str) -> bool {
        match self {
            Visibility::All => true,
            Visibility::Sandbox { own_app } => {
                key == textvn_ipc::client::GLOBAL_APP_ID
                    || key == textvn_ipc::client::ASSIST_APP_ID
                    || own_app
                        .as_deref()
                        .is_some_and(|own| own.eq_ignore_ascii_case(key))
            }
        }
    }
}

/// app_id (tên exe viết thường — cùng quy ước TIP `normalize_app_id`) của tiến trình client.
#[cfg(windows)]
fn client_app_id(pid: u32) -> Option<String> {
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: handle tiến trình mở ra được đóng ngay; buffer/độ dài hợp lệ.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(process);
        if !ok {
            return None;
        }
        let image = String::from_utf16_lossy(&buf[..(len as usize).min(buf.len())]);
        image
            .rsplit(['\\', '/'])
            .next()
            .filter(|n| !n.is_empty())
            .map(str::to_ascii_lowercase)
    }
}

/// Ghi frame tới toàn bộ subscriber đang Subscribe. Xóa subscriber ghi lỗi
/// (client chết / pipe vỡ) khỏi danh sách.
///
/// **Bẫy deadlock đã repro (2026-09):** một `ReadFile` sync đang pending trên
/// kernel file object của pipe sẽ serialize chặn mọi `WriteFile` trên handle
/// dup (`try_clone`) cùng object — kể cả khi buffer rỗng và client đang chờ
/// đọc. Nên handler KHÔNG BAO GIỜ được block trong `read_message` khi còn
/// subscriber cần broadcast: đọc phải được gate bằng `PeekNamedPipe`
/// (xem `try_read_pipe_frame`).
fn write_frame_to_subscribers(
    subscribers: Arc<Mutex<Vec<ClientSink>>>,
    frame: Vec<u8>,
    state_key: Option<String>,
) {
    // Clone danh sách sink rồi NHẢ mutex TRƯỚC khi ghi: write vào pipe sync có
    // thể block vô hạn nếu một subscriber bị suspend/ngừng đọc — giữ lock qua
    // write sẽ kẹt toàn bộ broadcast + Subscribe (đường điều khiển trung tâm).
    //
    // Mỗi subscriber có TỐI ĐA một writer và hàng đợi có giới hạn. Một client
    // ngừng đọc không thể tạo vô hạn thread hay giữ mutex toàn cục.
    const WRITE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
    const MAX_PENDING: usize = 32;
    type SinkSnapshot = (
        Arc<Mutex<std::fs::File>>,
        Arc<AtomicBool>,
        Arc<Mutex<VecDeque<Vec<u8>>>>,
    );
    let sinks: Vec<SinkSnapshot> = subscribers
        .lock()
        .map(|subs| {
            subs.iter()
                // R2-26: StateUpdate của app khác không tới client sandbox.
                .filter(|c| state_key.as_deref().is_none_or(|k| c.visibility.sees(k)))
                .map(|c| {
                    (
                        Arc::clone(&c.stream),
                        Arc::clone(&c.write_busy),
                        Arc::clone(&c.pending),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let mut dead: Vec<Arc<Mutex<std::fs::File>>> = Vec::new();
    for (stream, busy, pending) in &sinks {
        let start_writer = if let Ok(mut queue) = pending.lock() {
            if queue.len() >= MAX_PENDING {
                dead.push(Arc::clone(stream));
                continue;
            }
            queue.push_back(frame.clone());
            // Dưới cùng mutex queue: writer không thể thấy queue rỗng và tắt
            // busy giữa lúc broadcast đang enqueue.
            !busy.swap(true, Ordering::AcqRel)
        } else {
            dead.push(Arc::clone(stream));
            continue;
        };
        if !start_writer {
            continue;
        }
        let writer_stream = Arc::clone(stream);
        let writer_busy = Arc::clone(busy);
        let writer_pending = Arc::clone(pending);
        let writer_subscribers = Arc::clone(&subscribers);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut first = true;
            loop {
                let next = match writer_pending.lock() {
                    Ok(mut queue) => match queue.pop_front() {
                        Some(frame) => frame,
                        None => {
                            writer_busy.store(false, Ordering::Release);
                            break;
                        }
                    },
                    Err(_) => break,
                };
                let ok = match writer_stream.lock() {
                    Ok(mut f) => f.write_all(&next).is_ok() && f.flush().is_ok(),
                    Err(_) => false,
                };
                if first {
                    let _ = tx.send(ok);
                    first = false;
                }
                if !ok {
                    if let Ok(mut subs) = writer_subscribers.lock() {
                        subs.retain(|c| !Arc::ptr_eq(&c.stream, &writer_stream));
                    }
                    break;
                }
            }
        });
        match rx.recv_timeout(WRITE_TIMEOUT) {
            Ok(true) => {}
            _ => dead.push(Arc::clone(stream)),
        }
    }
    if !dead.is_empty() {
        if let Ok(mut subs) = subscribers.lock() {
            subs.retain(|c| !dead.iter().any(|d| Arc::ptr_eq(d, &c.stream)));
        }
    }
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

    // Payload chưa đủ: poll tiếp mà KHÔNG giữ read pending. Client gửi header
    // rồi ngừng cũng phải bị đóng, không giữ một handler thread vô hạn.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if !running.load(Ordering::Acquire) {
            return PipeFrame::Eof;
        }
        if Instant::now() >= deadline {
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
    stream: Arc<Mutex<std::fs::File>>,
    write_busy: Arc<AtomicBool>,
    pending: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// R2-26: client sandbox chỉ nhận StateUpdate của khoá nó được thấy.
    visibility: Visibility,
}

pub struct IpcServer {
    svc: Arc<SvcManager>,
    start_time: Instant,
    running: Arc<AtomicBool>,
    subscribers: Arc<Mutex<Vec<ClientSink>>>,
    crash_counter: AtomicU32,
    active_clients: AtomicU32,
    hook_pid: AtomicU32,
    /// R2-26: số lệnh client sandbox gửi bị từ chối (chẩn đoán + test).
    sandbox_denied: AtomicU32,
    /// R2-26: số kết nối pipe sandbox đang mở (trần `SANDBOX_MAX_CONNECTIONS`).
    sandbox_active: AtomicU32,
    /// Mỗi tên một vòng accept riêng; subscriber + broadcast dùng chung. `true` = pipe sandbox.
    #[cfg_attr(not(windows), allow(dead_code))]
    pipes: Vec<(String, bool)>,
}

impl IpcServer {
    pub fn new(svc: Arc<SvcManager>) -> Arc<Self> {
        // Vòng 16 (audit A7): pipe theo phiên — hết xung đột đa phiên/RDP.
        // R2-109: phục vụ THÊM tên cũ `PIPE_NAME`. Nâng cấp từ bản <= 0.2.30 không
        // khởi động lại app: app đang mở vẫn nạp DLL TSF cũ (file đã đổi tên
        // .old-*, vẫn map) — DLL đó chỉ biết tên cũ, nên bật/tắt V/E từ khay không
        // tới các app ấy cho tới khi chúng mở lại. Client mới luôn thử pipe phiên
        // trước (`textvn_ipc::candidate_pipe_names`); DACL chỉ cho đúng user nên
        // phiên khác không dùng được instance tên cũ của phiên này.
        // R2-26: + pipe sandbox cho ô tìm kiếm Start / ứng dụng Store (AppContainer).
        #[cfg(windows)]
        let pipes = [
            (textvn_ipc::session_pipe_name(), false),
            (String::from(PIPE_NAME), false),
            (textvn_ipc::sandbox::sandbox_pipe_name(), true),
        ];
        #[cfg(not(windows))]
        let pipes = [(String::from(PIPE_NAME), false)];
        Self::with_pipes(svc, &pipes)
    }

    /// Test dùng tên pipe riêng: `PIPE_NAME` là namespace toàn máy, tray thật
    /// đang chạy giữ nó thì client của test nối vào tray thật (và ToggleViEn
    /// trong test đảo V/E của người dùng).
    #[cfg(all(test, windows))]
    fn with_pipe_name(svc: Arc<SvcManager>, pipe_name: &str) -> Arc<Self> {
        Self::with_pipes(svc, &[(pipe_name.to_string(), false)])
    }

    pub(crate) fn with_pipes(svc: Arc<SvcManager>, pipes: &[(String, bool)]) -> Arc<Self> {
        Arc::new(Self {
            svc,
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            crash_counter: AtomicU32::new(0),
            active_clients: AtomicU32::new(0),
            hook_pid: AtomicU32::new(0),
            sandbox_denied: AtomicU32::new(0),
            sandbox_active: AtomicU32::new(0),
            pipes: pipes.to_vec(),
        })
    }

    /// R2-26: số lệnh client sandbox gửi bị từ chối từ lúc tray chạy.
    pub fn sandbox_denied(&self) -> u32 {
        self.sandbox_denied.load(Ordering::Acquire)
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
        self.broadcast_scoped(&msg, Some(app_id.to_string()));
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
            write_frame_to_subscribers(subscribers, frame, None);
        });
        let deadline = Instant::now() + Duration::from_millis(200);
        while !worker.is_finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        // Worker kẹt (client chết chiếm pipe buffer) thì detach — đã chờ đủ.
    }

    fn broadcast_message(&self, msg: &Message) {
        self.broadcast_scoped(msg, None);
    }

    /// `state_key = Some(k)`: chỉ client thấy khoá `k` nhận (R2-26).
    fn broadcast_scoped(&self, msg: &Message, state_key: Option<String>) {
        let Ok(frame) = encode_frame(msg) else { return };
        let subscribers = self.subscribers.clone();

        // Named-pipe writes are synchronous and a subscriber can stop consuming
        // at any point.  Never perform one on the tray window thread: doing so
        // would turn a stale Hook/TSF client into a frozen Exit/Settings UI.
        std::thread::spawn(move || {
            write_frame_to_subscribers(subscribers, frame, state_key);
        });
    }

    /// Bắt đầu pipe server loop trên một background thread.
    #[cfg(windows)]
    pub fn start(self: &Arc<Self>) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        for (pipe_name, sandbox) in &self.pipes {
            let this = self.clone();
            let pipe_name = pipe_name.clone();
            let sandbox = *sandbox;
            std::thread::spawn(move || {
                this.server_accept_loop(&pipe_name, sandbox);
            });
        }

        // Khởi động watchdog giám sát hook process (WIN-040, WIN-051 §6)
        let this_watchdog = self.clone();
        std::thread::spawn(move || {
            this_watchdog.hook_watchdog_loop();
        });
    }

    #[cfg(windows)]
    fn server_accept_loop(self: Arc<Self>, pipe_name: &str, sandbox: bool) {
        let pipe_name_wide: Vec<u16> = pipe_name.encode_utf16().chain(Some(0)).collect();

        while self.running.load(Ordering::Acquire) {
            // SEC-05 (audit 2026-10-04): pipe KHÔNG có DACL = mọi user đăng nhập
            // (kể cả phiên RDP khác) đọc/ghi được → nghe lén ConfigReload hoặc
            // gửi Shutdown/ToggleViEn giả. Tạo SECURITY_ATTRIBUTES chỉ cho user
            // hiện tại (+SYSTEM/Administrators như quy ước Windows).
            // SAFETY: descriptor được kernel sao chép khi tạo pipe; LocalFree
            // ngay sau lời gọi (không giữ con trỏ).
            let security = pipe_security(sandbox);
            // SAFETY: Tạo named pipe instance
            let handle = unsafe {
                CreateNamedPipeW(
                    PCWSTR(pipe_name_wide.as_ptr()),
                    PIPE_ACCESS_DUPLEX,
                    // Chỉ phục vụ client cục bộ (TSF/Hook/CLI cùng máy).
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    PIPE_UNLIMITED_INSTANCES,
                    MAX_FRAME_BYTES as u32,
                    MAX_FRAME_BYTES as u32,
                    0,
                    security
                        .as_ref()
                        .map(|(sa, _)| sa as *const SECURITY_ATTRIBUTES),
                )
            };
            if let Some((_, sd)) = security {
                // SAFETY: SD do ConvertStringSecurityDescriptorToSecurityDescriptorW
                // cấp, giải phóng đúng một lần bằng LocalFree.
                let _ = unsafe { LocalFree(Some(HLOCAL(sd.0))) };
            }

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

                // R2-26: trần kết nối sandbox — quá trần thì đóng ngay (không thread mới).
                if sandbox
                    && self.sandbox_active.fetch_add(1, Ordering::AcqRel) >= SANDBOX_MAX_CONNECTIONS
                {
                    self.sandbox_active.fetch_sub(1, Ordering::AcqRel);
                    self.sandbox_denied.fetch_add(1, Ordering::AcqRel);
                    drop(stream);
                    drop(reader);
                    continue;
                }
                self.active_clients.fetch_add(1, Ordering::SeqCst);
                let this = self.clone();
                let stream_clone = stream.clone();

                std::thread::spawn(move || {
                    this.handle_client_connection(reader, stream_clone, peer_pid, sandbox);
                    this.active_clients.fetch_sub(1, Ordering::SeqCst);
                    if sandbox {
                        this.sandbox_active.fetch_sub(1, Ordering::AcqRel);
                    }
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
        sandbox: bool,
    ) {
        let mut subscribed = false;
        // R2-26: client pipe sandbox chỉ thấy `*`, `assist` và app của chính nó.
        let visibility = if sandbox {
            Visibility::Sandbox {
                own_app: client_app_id(peer_pid),
            }
        } else {
            Visibility::All
        };
        let mut budget = FrameBudget::new(Instant::now());
        // Mỗi process có TIP giữ MỘT kết nối suốt đời; poll cố định 10 ms nhân với
        // vài chục app là hàng nghìn lần đánh thức CPU mỗi giây khi không ai gõ.
        // Client gần như chỉ gửi lúc kết nối/toggle → giãn dần tới 100 ms khi rảnh.
        const POLL_MIN: Duration = Duration::from_millis(10);
        const POLL_MAX: Duration = Duration::from_millis(100);
        let mut poll = POLL_MIN;

        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }

            // Đọc poll-gated: không bao giờ block vô hạn với read-pending trên
            // pipe (read-pending serialize chặn broadcast write → hook mồ côi).
            let mut msg = match try_read_pipe_frame(&reader, &self.running) {
                PipeFrame::Empty => {
                    std::thread::sleep(poll);
                    poll = (poll * 2).min(POLL_MAX);
                    continue;
                }
                PipeFrame::Eof => break,
                PipeFrame::Frame(m) => {
                    poll = POLL_MIN;
                    m
                }
            };

            // R2-26: pipe sandbox — bất kỳ ứng dụng Store nào của người dùng cũng nối được, nên
            // chỉ phục vụ tập lệnh tối thiểu (`sandbox_policy`). Lệnh bị từ chối vẫn nhận Ack
            // (client không treo chờ) và không làm gì.
            if sandbox {
                if !budget.take(Instant::now()) {
                    // Spam: đóng kết nối (client thật nối lại sau 500 ms–10 s).
                    self.sandbox_denied.fetch_add(1, Ordering::AcqRel);
                    break;
                }
                let deny = match sandbox_policy(&msg) {
                    SandboxPolicy::Allow => false,
                    SandboxPolicy::RsiOnly => match &msg {
                        Message::StyleUpdate { json } => {
                            match textvn_config::sandbox::style_drain_rsi_only(json) {
                                Some(rsi_only) => {
                                    msg = Message::StyleUpdate { json: rsi_only };
                                    false
                                }
                                None => true,
                            }
                        }
                        _ => true,
                    },
                    SandboxPolicy::Deny => true,
                };
                if deny {
                    self.sandbox_denied.fetch_add(1, Ordering::AcqRel);
                    let ok = stream
                        .lock()
                        .ok()
                        .zip(encode_frame(&Message::Ack).ok())
                        .is_some_and(|(mut file, frame)| {
                            file.write_all(&frame).is_ok() && file.flush().is_ok()
                        });
                    if !ok {
                        break;
                    }
                    continue;
                }
            }

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
                    Some(self.build_snapshot_for(&visibility))
                }
                Message::GetSnapshot => Some(self.build_snapshot_for(&visibility)),
                Message::Subscribe { pid } => {
                    subscribed = true;
                    debug_assert_eq!(pid, peer_pid);
                    // R2-114: một luồng panic khi giữ lock không được chặn mọi client
                    // Subscribe về sau (poison) — dùng tiếp dữ liệu bên trong.
                    let mut subs = self.subscribers.lock().unwrap_or_else(|e| e.into_inner());
                    if !subs.iter().any(|c| Arc::ptr_eq(&c.stream, &stream)) {
                        subs.push(ClientSink {
                            stream: stream.clone(),
                            write_busy: Arc::new(AtomicBool::new(false)),
                            pending: Arc::new(Mutex::new(VecDeque::new())),
                            visibility: visibility.clone(),
                        });
                    }
                    Some(Message::Ack)
                }
                Message::ToggleViEn { app_id, enabled } => {
                    // Phát state thật sau khi lưu: persist lỗi → state cũ giữ nguyên
                    // (review R3 minor 8), TSF áp StateUpdate không so version.
                    let (actual, ver) = if app_id == "*" {
                        // Debounce chéo nguồn (E11): TIP in-process và LL hook cùng
                        // bắn một lần bấm — nguồn sau trong 250ms bị bỏ qua (chỉ
                        // phản hồi state hiện tại, không toggle lần hai).
                        if crate::try_claim_global_toggle() {
                            let r = self.svc.set_global_enabled(enabled);
                            crate::notify_tray_state_changed();
                            r
                        } else {
                            (self.svc.is_global_enabled(), self.svc.state_version())
                        }
                    } else {
                        let v = self.svc.set_app_enabled(&app_id, enabled);
                        (self.svc.is_app_enabled(&app_id), v)
                    };
                    self.broadcast_state_update(&app_id, actual, ver);
                    // Trả SNAPSHOT (state chuẩn) thay vì Ack: TIP gửi giá trị
                    // tuyệt đối tính từ state CỤC BỘ của nó — nếu lệch với tray
                    // (bấm đôi trong 250ms, tray restart…) thì response này
                    // kéo TIP về đúng state thật ngay (báo cáo 0.2.9 "icon E
                    // mà vẫn gõ tiếng Việt").
                    Some(self.build_snapshot_for(&visibility))
                }
                Message::ToggleGlobal => {
                    if crate::try_claim_global_toggle() {
                        let (enabled, ver) = self.svc.toggle_global_enabled();
                        self.broadcast_state_update("*", enabled, ver);
                        crate::notify_tray_state_changed();
                    }
                    // Trùng lần bấm — nguồn khác đã toggle trong 250ms: trả
                    // state chuẩn (snapshot) để TIP tự kéo về đúng.
                    Some(self.build_snapshot_for(&visibility))
                }
                Message::ToggleAssist { enabled } => {
                    let (actual, ver) = self.svc.set_assist_enabled(enabled);
                    self.broadcast_state_update("assist", actual, ver);
                    Some(self.build_snapshot_for(&visibility))
                }
                Message::AssistNotification { title, message } => {
                    // Kill-switch chặn cả kênh thông báo assist cũ (D20).
                    crate::assist::notify_simple(&self.svc, &title, &message);
                    Some(Message::Ack)
                }
                Message::AssistReport { issues, auto_fixed } => {
                    // Kill-switch + `assist.mode` + chống spam nằm trong
                    // `crate::assist::notify_report` (logic thuần, đã test).
                    crate::assist::notify_report(&self.svc, &issues, auto_fixed);
                    Some(Message::Ack)
                }
                Message::StyleUpdate { json } => {
                    // P7 (vòng 57): merge dữ liệu học vào config + persist +
                    // bump version và broadcast ConfigReload tới mọi adapter. Drain chỉ có số đo
                    // Dream RSI được gom/ghi âm thầm — KHÔNG reload (reload reset từ đang gõ).
                    if let Ok(Some(ver)) = crate::style_merge::merge_from_json(&self.svc, &json) {
                        self.broadcast_config_reload(ver);
                    }
                    Some(Message::Ack)
                }
                Message::ToggleMath => {
                    if let Ok((enabled, ver)) = self.svc.toggle_math_enabled() {
                        self.broadcast_config_reload(ver);
                        crate::notify_tray_state_changed();
                        let msg = if enabled {
                            "Math & Science: Đã bật (gõ `a, \\can, =2+3=)"
                        } else {
                            "Math & Science: Đã tắt"
                        };
                        crate::assist::notify_simple(&self.svc, "TextVN Math", msg);
                    }
                    Some(Message::Ack)
                }
                Message::CycleMathMode => {
                    if let Ok((mode, ver)) = self.svc.cycle_math_output_mode() {
                        self.broadcast_config_reload(ver);
                        crate::notify_tray_state_changed();
                        let label = match mode {
                            textvn_config::MathOutputMode::AutoByApp => "Tự động theo ứng dụng",
                            textvn_config::MathOutputMode::Normal => "Unicode trực tiếp (α, √)",
                            textvn_config::MathOutputMode::Latex => "LaTeX (\\alpha, \\frac{}{})",
                            textvn_config::MathOutputMode::UnicodeMath => {
                                "Word Equation (UnicodeMath)"
                            }
                            textvn_config::MathOutputMode::Typst => "Typst",
                        };
                        crate::assist::notify_simple(
                            &self.svc,
                            "TextVN Math",
                            &format!("Chế độ công thức: {label}"),
                        );
                    }
                    Some(Message::Ack)
                }
                Message::OpenEmojiPicker => {
                    crate::notify_open_emoji_picker();
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
            let mut subs = self.subscribers.lock().unwrap_or_else(|e| e.into_inner());
            // Nhiều pipe từ CÙNG PID (TSF trong một app) có thể Subscribe.
            // Chỉ gỡ connection đã đóng, không gỡ nhầm pipe còn sống.
            subs.retain(|c| !Arc::ptr_eq(&c.stream, &stream));
        }
    }

    #[cfg(test)]
    fn build_snapshot(&self) -> Message {
        self.build_snapshot_for(&Visibility::All)
    }

    /// Snapshot theo quyền thấy của client (R2-26: client sandbox không thấy override app khác).
    fn build_snapshot_for(&self, visibility: &Visibility) -> Message {
        // `"*"` = công tắc toàn cục (cùng khóa với StateUpdate). Thiếu khóa này,
        // client kết nối sau khi người dùng tắt tiếng Việt vẫn gõ tiếng Việt.
        let mut state = self.svc.app_states();
        state.retain(|app, _| visibility.sees(app));
        state.insert("*".into(), self.svc.is_global_enabled());
        state.insert("assist".into(), self.svc.is_assist_enabled());
        Message::Snapshot {
            config_version: self.svc.config_version(),
            state,
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

    /// R2-26: pipe sandbox chỉ cho đọc trạng thái + Ctrl+Shift toàn cục; tự học chỉ còn bộ đếm.
    #[test]
    fn sandbox_policy_allows_only_minimal_commands() {
        use SandboxPolicy::*;
        let allow = [
            Message::Hello {
                pid: 1,
                abi: 1,
                version: "x".into(),
            },
            Message::Subscribe { pid: 1 },
            Message::GetSnapshot,
            Message::Ping,
            Message::Pong { uptime_ms: 0 },
            Message::ToggleGlobal,
            Message::ToggleViEn {
                app_id: "*".into(),
                enabled: false,
            },
        ];
        for m in &allow {
            assert_eq!(sandbox_policy(m), Allow, "{m:?}");
        }
        assert_eq!(
            sandbox_policy(&Message::StyleUpdate { json: "{}".into() }),
            RsiOnly
        );
        let deny = [
            Message::Shutdown,
            Message::ToggleViEn {
                app_id: "chrome.exe".into(),
                enabled: false,
            },
            Message::ToggleAssist { enabled: false },
            Message::ToggleMath,
            Message::CycleMathMode,
            Message::OpenEmojiPicker,
            Message::AssistNotification {
                title: "t".into(),
                message: "m".into(),
            },
            Message::ConfigReload { version: 1 },
            Message::Ack,
        ];
        for m in &deny {
            assert_eq!(sandbox_policy(m), Deny, "{m:?}");
        }
    }

    /// R2-26: hạn mức frame mỗi giây của kết nối sandbox — reset theo cửa sổ.
    #[test]
    fn sandbox_frame_budget_resets_every_second() {
        let t0 = Instant::now();
        let mut b = FrameBudget::new(t0);
        for _ in 0..SANDBOX_MAX_FRAMES_PER_SEC {
            assert!(b.take(t0));
        }
        assert!(
            !b.take(t0 + Duration::from_millis(500)),
            "vượt hạn mức trong 1 s"
        );
        assert!(b.take(t0 + Duration::from_millis(1000)), "cửa sổ mới");
    }

    /// R2-26: client sandbox chỉ thấy `*`, `assist` và app của chính nó (không phân biệt hoa).
    #[test]
    fn sandbox_visibility_hides_other_apps() {
        let v = Visibility::Sandbox {
            own_app: Some("searchhost.exe".into()),
        };
        assert!(v.sees("*") && v.sees("assist") && v.sees("SearchHost.exe"));
        assert!(!v.sees("chrome.exe") && !v.sees("zalo.exe"));
        let none = Visibility::Sandbox { own_app: None };
        assert!(none.sees("*") && !none.sees("searchhost.exe"));
        assert!(Visibility::All.sees("chrome.exe"));

        let temp_dir =
            std::env::temp_dir().join(format!("textvn_ipc_vis_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        svc.set_app_enabled("chrome.exe", false);
        svc.set_app_enabled("searchhost.exe", false);
        let server = IpcServer::new(svc);
        let Message::Snapshot { state, .. } = server.build_snapshot_for(&v) else {
            panic!("snapshot");
        };
        assert_eq!(state.get("searchhost.exe"), Some(&false));
        assert!(!state.contains_key("chrome.exe"));
        assert!(state.contains_key("*") && state.contains_key("assist"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

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
                assert_eq!(state.get("*"), Some(&svc.is_global_enabled()));
                assert_eq!(state.get("assist"), Some(&svc.is_assist_enabled()));
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

    /// Tên pipe riêng cho từng test — không bao giờ nối vào tray thật đang
    /// chạy trên máy dev (giữ `PIPE_NAME`), các test cũng chạy song song được.
    #[cfg(windows)]
    fn test_pipe_name(tag: &str) -> String {
        format!(r"\\.\pipe\textvn-ipc-test-{}-{tag}", std::process::id())
    }

    #[test]
    #[cfg(windows)]
    fn server_starts_and_stops_cleanly() {
        let pipe = test_pipe_name("startstop");
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_startstop_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::with_pipe_name(svc, &pipe);

        assert!(!server.is_running());
        server.start();
        assert!(server.is_running());

        // Cho pipe loop vao ConnectNamedPipe
        std::thread::sleep(Duration::from_millis(50));

        // Stop phai unblock va chuyen is_running ve false sach se
        server.stop();
        assert!(!server.is_running());
        // Unblock pending ConnectNamedPipe so the worker thread observes running=false and terminates
        let _ = textvn_ipc::pipe_client_options().open(&pipe);
        std::thread::sleep(Duration::from_millis(50));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// Broadcast phải thực sự tới được subscriber qua named pipe.
    /// Regression: WIP shutdown từng gửi Ack nhưng frame broadcast không tới
    /// client khiến hook mồ côi sau khi tray thoát.
    #[test]
    #[cfg(windows)]
    fn broadcast_reaches_subscriber() {
        use std::io::Write as _;

        let pipe = test_pipe_name("bcast");
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_bcast_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::with_pipe_name(svc, &pipe);
        server.start();
        std::thread::sleep(Duration::from_millis(100));

        let (tx, rx) = std::sync::mpsc::channel();
        let client_pipe = pipe.clone();
        let client = std::thread::spawn(move || {
            let mut file = None;
            for _ in 0..20 {
                if let Ok(f) = textvn_ipc::pipe_client_options().open(&client_pipe) {
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
        let _ = textvn_ipc::pipe_client_options().open(&pipe);
        std::thread::sleep(Duration::from_millis(50));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// R2-109: một server phục vụ NHIỀU tên — pipe phiên + tên cũ cho DLL TSF bản cũ
    /// còn nạp trong app đang mở sau khi nâng cấp. Client nối qua tên nào cũng phải
    /// Subscribe được và nhận CÙNG broadcast.
    #[test]
    #[cfg(windows)]
    fn serves_every_pipe_name() {
        use std::io::Write as _;

        let names = [test_pipe_name("multi-a"), test_pipe_name("multi-b")];
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_multi_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let pipes: Vec<(String, bool)> = names.iter().map(|n| (n.clone(), false)).collect();
        let server = IpcServer::with_pipes(svc, &pipes);
        server.start();
        std::thread::sleep(Duration::from_millis(100));

        let (tx, rx) = std::sync::mpsc::channel();
        let clients: Vec<_> = names
            .iter()
            .cloned()
            .map(|name| {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut file = None;
                    for _ in 0..20 {
                        if let Ok(f) = textvn_ipc::pipe_client_options().open(&name) {
                            file = Some(f);
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    let mut file = file.expect("client pipe open");
                    let sub = encode_frame(&Message::Subscribe {
                        pid: std::process::id(),
                    })
                    .unwrap();
                    file.write_all(&sub).unwrap();
                    file.flush().unwrap();
                    tx.send(read_message(&mut file).expect("ack frame"))
                        .unwrap();
                    tx.send(read_message(&mut file).expect("broadcast frame"))
                        .unwrap();
                })
            })
            .collect();

        for _ in &names {
            let ack = rx
                .recv_timeout(Duration::from_secs(3))
                .expect("Ack within 3s");
            assert_eq!(ack, Message::Ack);
        }
        server.broadcast_config_reload(7);
        for _ in &names {
            let update = rx
                .recv_timeout(Duration::from_secs(3))
                .expect("broadcast within 3s on every pipe name");
            assert_eq!(update, Message::ConfigReload { version: 7 });
        }

        for client in clients {
            client.join().unwrap();
        }
        server.stop();
        for name in &names {
            let _ = textvn_ipc::pipe_client_options().open(name);
        }
        std::thread::sleep(Duration::from_millis(50));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// ToggleViEn("*") phải trả SNAPSHOT (state chuẩn) chứ không chỉ Ack —
    /// TIP nhận snapshot để tự kéo về đúng state khi bị debounce bỏ qua.
    #[cfg(windows)]
    #[test]
    fn toggle_global_responds_with_authoritative_snapshot() {
        use std::io::Write as _;
        // E82/G23: không chồng cửa sổ chống dội toàn cục với test khác (sandbox_e2e).
        let _toggle_lock = crate::GLOBAL_TOGGLE_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let pipe = test_pipe_name("tgl");
        let temp_dir = std::env::temp_dir().join(format!("textvn_tgl_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let server = IpcServer::with_pipe_name(svc, &pipe);
        server.start();
        std::thread::sleep(Duration::from_millis(100));

        let client_pipe = pipe.clone();
        let client = std::thread::spawn(move || {
            let mut file = None;
            for _ in 0..20 {
                if let Ok(f) = textvn_ipc::pipe_client_options().open(&client_pipe) {
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
            let _ = read_message(&mut file).expect("ack");
            let toggle = encode_frame(&Message::ToggleViEn {
                app_id: "*".into(),
                enabled: false,
            })
            .unwrap();
            file.write_all(&toggle).unwrap();
            file.flush().unwrap();
            read_message(&mut file).expect("snapshot response")
        });

        let resp = client.join().expect("client thread");
        match resp {
            Message::Snapshot { state, .. } => {
                assert!(
                    state.contains_key("*"),
                    "snapshot phải chứa global state '*'"
                );
            }
            other => panic!("expected Snapshot, got {other:?}"),
        }
        server.stop();
        let _ = textvn_ipc::pipe_client_options().open(&pipe);
        std::thread::sleep(Duration::from_millis(50));
    }
}
