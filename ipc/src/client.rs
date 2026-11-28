// SPDX-License-Identifier: GPL-3.0-or-later
//! Logic client IPC dùng chung 3 hệ điều hành (SF-6 Phần 1).
//!
//! Bao gồm:
//! - Khoá quy ước [`GLOBAL_APP_ID`] (`"*"`) và [`ASSIST_APP_ID`] (`"assist"`).
//! - Bộ lọc sandbox [`allowed_in_sandbox`] (E80/E81/R2-26: cấm rò dữ liệu người dùng).
//! - Bộ tích luỹ frame [`FrameBuffer`] xử lý short-read / multi-frame không cấp phát lố.
//! - Máy trạng thái [`IpcClientState`] áp `Snapshot` / `StateUpdate` / `ConfigReload`.
//! - [`ReconnectBackoff`] luỹ tiến có chặn trần.
//! - Resolver đường dẫn socket [`resolve_linux_socket_path`] (kèm chặn đường dẫn tương đối C9)
//!   và [`resolve_macos_socket_path`].

use std::collections::BTreeMap;
use std::time::Duration;

use crate::{decode_exact_frame, encode_frame, CodecError, Message, MAX_FRAME_BYTES};

/// Khoá trong map trạng thái đại diện công tắc toàn cục.
pub const GLOBAL_APP_ID: &str = "*";

/// Khoá trong map trạng thái đại diện kill-switch Trợ lý ngữ pháp (D20 / AS-023).
pub const ASSIST_APP_ID: &str = "assist";

/// Độ trễ reconnect mặc định (ms).
pub const DEFAULT_BACKOFF_INITIAL_MS: u64 = 100;
/// Trần độ trễ reconnect mặc định (ms).
pub const DEFAULT_BACKOFF_MAX_MS: u64 = 5_000;

/// Chỉ các message điều khiển tối thiểu được phép gửi trên kênh sandbox (E80/E81/R2-26).
/// Tuyệt đối cấm `StyleUpdate`, `AssistReport`, `AssistNotification`, `Shutdown` hoặc hotkey sửa cấu hình.
pub fn allowed_in_sandbox(msg: &Message) -> bool {
    matches!(
        msg,
        Message::Hello { .. }
            | Message::GetSnapshot
            | Message::Subscribe { .. }
            | Message::ToggleViEn { .. }
            | Message::ToggleGlobal
            | Message::ToggleAssist { .. }
            | Message::Ping
            | Message::CrashReport { .. }
    )
}

/// Mã hoá message outgoing; trả `Ok(None)` nếu đang sandbox và message bị chặn bởi E80/R2-26.
pub fn encode_outgoing(msg: &Message, sandboxed: bool) -> Result<Option<Vec<u8>>, CodecError> {
    if sandboxed && !allowed_in_sandbox(msg) {
        return Ok(None);
    }
    encode_frame(msg).map(Some)
}

/// Buffer đọc frame non-blocking cho socket / pipe (xử lý short-read + coalesced frames).
#[derive(Debug, Clone, Default)]
pub struct FrameBuffer {
    buf: Vec<u8>,
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &[u8]) {
        self.buf.extend_from_slice(chunk);
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Trả `Ok(None)` khi chưa đủ bytes cho frame hiện tại; `Err` khi gặp lỗi protocol
    /// (lúc đó buffer bị xoá để caller disconnect / reset an toàn).
    pub fn next_frame(&mut self) -> Result<Option<Message>, CodecError> {
        if self.buf.len() < 4 {
            return Ok(None);
        }
        let length = u32::from_le_bytes(self.buf[..4].try_into().expect("checked >= 4")) as usize;
        if length > MAX_FRAME_BYTES {
            self.buf.clear();
            return Err(CodecError::Oversize);
        }
        let total = 4 + length;
        if self.buf.len() < total {
            return Ok(None);
        }
        let res = decode_exact_frame(&self.buf[..total]);
        if res.is_ok() {
            self.buf.drain(..total);
        } else {
            self.buf.clear();
        }
        res.map(Some)
    }
}

/// Hiệu ứng rút ra sau khi áp một message IPC từ server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientEffect {
    pub config_version_changed: Option<u64>,
    pub effective_enabled_changed: Option<bool>,
    pub assist_changed: Option<bool>,
}

/// Máy trạng thái thuần của IPC client (dùng chung Windows TSF, Linux C ABI, macOS IMK).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcClientState {
    connected: bool,
    sandboxed: bool,
    config_version: u64,
    global_enabled: bool,
    app_override: Option<bool>,
    assist_enabled: bool,
    app_id: String,
    last_states: BTreeMap<String, bool>,
}

impl Default for IpcClientState {
    fn default() -> Self {
        Self::new("")
    }
}

impl IpcClientState {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            connected: false,
            sandboxed: false,
            config_version: 0,
            global_enabled: true,
            app_override: None,
            // Vòng 29: mặc định true trước khi nhận Snapshot để không chặn nhầm trợ lý.
            assist_enabled: true,
            app_id: app_id.into(),
            last_states: BTreeMap::new(),
        }
    }

    pub fn set_connected(&mut self, connected: bool, sandboxed: bool) {
        self.connected = connected;
        self.sandboxed = if connected { sandboxed } else { false };
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn is_sandboxed(&self) -> bool {
        self.sandboxed
    }

    pub fn config_version(&self) -> u64 {
        self.config_version
    }

    pub fn global_enabled(&self) -> bool {
        self.global_enabled
    }

    pub fn app_override(&self) -> Option<bool> {
        self.app_override
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Trạng thái bật/tắt hiệu lực = `global && app_override.unwrap_or(true)`.
    pub fn is_enabled(&self) -> bool {
        self.global_enabled && self.app_override.unwrap_or(true)
    }

    pub fn is_assist_enabled(&self) -> bool {
        self.assist_enabled
    }

    /// Đổi `app_id` hiện hành (cho adapter phục vụ nhiều app như Linux/macOS), tự tính lại override
    /// từ `last_states` đã cache.
    pub fn set_app_id(&mut self, app_id: impl Into<String>) {
        self.app_id = app_id.into();
        self.app_override = lookup_app_override(&self.last_states, &self.app_id);
    }

    /// Cập nhật cục bộ tức thì khi người dùng nhấn hotkey trước khi nhận broadcast từ tray.
    pub fn set_local_global(&mut self, enabled: bool) {
        self.global_enabled = enabled;
    }

    /// Áp toàn bộ map trạng thái (từ `Snapshot` hoặc `state.json`).
    pub fn apply_states(&mut self, states: &BTreeMap<String, bool>) -> ClientEffect {
        let prev_enabled = self.is_enabled();
        let prev_assist = self.assist_enabled;
        self.last_states = states.clone();

        if let Some(&global) = states.get(GLOBAL_APP_ID) {
            self.global_enabled = global;
        }
        if let Some(&assist) = states.get(ASSIST_APP_ID) {
            self.assist_enabled = assist;
        }
        self.app_override = lookup_app_override(&self.last_states, &self.app_id);

        let now_enabled = self.is_enabled();
        let now_assist = self.assist_enabled;
        ClientEffect {
            config_version_changed: None,
            effective_enabled_changed: (now_enabled != prev_enabled).then_some(now_enabled),
            assist_changed: (now_assist != prev_assist).then_some(now_assist),
        }
    }

    /// Áp một message nhận được từ tray server.
    pub fn apply_message(&mut self, msg: &Message) -> ClientEffect {
        match msg {
            Message::Snapshot {
                config_version,
                state,
                ..
            } => {
                let version_changed =
                    (*config_version != self.config_version).then_some(*config_version);
                self.config_version = *config_version;
                let mut effect = self.apply_states(state);
                effect.config_version_changed = version_changed;
                effect
            }
            Message::StateUpdate {
                app_id,
                enabled,
                version,
            } => {
                let prev_enabled = self.is_enabled();
                let prev_assist = self.assist_enabled;
                let version_changed = (*version != self.config_version).then_some(*version);
                self.config_version = *version;
                self.last_states.insert(app_id.clone(), *enabled);

                if app_id == GLOBAL_APP_ID {
                    self.global_enabled = *enabled;
                } else if app_id == ASSIST_APP_ID {
                    self.assist_enabled = *enabled;
                } else if !self.app_id.is_empty() && app_id.eq_ignore_ascii_case(&self.app_id) {
                    self.app_override = Some(*enabled);
                }

                let now_enabled = self.is_enabled();
                let now_assist = self.assist_enabled;
                ClientEffect {
                    config_version_changed: version_changed,
                    effective_enabled_changed: (now_enabled != prev_enabled).then_some(now_enabled),
                    assist_changed: (now_assist != prev_assist).then_some(now_assist),
                }
            }
            Message::ConfigReload { version } => {
                let version_changed = (*version != self.config_version).then_some(*version);
                self.config_version = *version;
                ClientEffect {
                    config_version_changed: version_changed,
                    effective_enabled_changed: None,
                    assist_changed: None,
                }
            }
            _ => ClientEffect::default(),
        }
    }
}

fn lookup_app_override(states: &BTreeMap<String, bool>, app_id: &str) -> Option<bool> {
    if app_id.is_empty() {
        return None;
    }
    if let Some(&v) = states.get(app_id) {
        return Some(v);
    }
    states
        .iter()
        .find(|(k, _)| {
            k.as_str() != GLOBAL_APP_ID
                && k.as_str() != ASSIST_APP_ID
                && k.eq_ignore_ascii_case(app_id)
        })
        .map(|(_, &v)| v)
}

/// Bộ tính độ trễ reconnect luỹ tiến theo bội 2, có chặn trần.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconnectBackoff {
    initial_ms: u64,
    max_ms: u64,
    current_ms: u64,
}

impl Default for ReconnectBackoff {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(DEFAULT_BACKOFF_INITIAL_MS),
            Duration::from_millis(DEFAULT_BACKOFF_MAX_MS),
        )
    }
}

impl ReconnectBackoff {
    pub fn new(initial: Duration, max: Duration) -> Self {
        let initial_ms = u64::try_from(initial.as_millis())
            .unwrap_or(u64::MAX)
            .max(1);
        let max_ms = u64::try_from(max.as_millis())
            .unwrap_or(u64::MAX)
            .max(initial_ms);
        Self {
            initial_ms,
            max_ms,
            current_ms: initial_ms,
        }
    }

    pub fn current_delay(&self) -> Duration {
        Duration::from_millis(self.current_ms)
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay = self.current_ms;
        self.current_ms = self.current_ms.saturating_mul(2).min(self.max_ms);
        Duration::from_millis(delay)
    }

    pub fn reset(&mut self) {
        self.current_ms = self.initial_ms;
    }
}

/// Giải đường dẫn Unix socket IPC trên Linux (khớp 1:1 `lc_ipc_resolve_socket_path`):
/// 1. `custom` khác rỗng
/// 2. `$XDG_RUNTIME_DIR/TextVN/ipc.sock` (chỉ khi là đường dẫn tuyệt đối bắt đầu bằng `/`)
/// 3. `$XDG_CONFIG_HOME/TextVN/ipc.sock` (chỉ khi bắt đầu bằng `/`, chống C9 relative override)
/// 4. `$HOME/.config/TextVN/ipc.sock` (chỉ khi khác rỗng)
/// 5. Fallback `/tmp/textvn-<uid>/ipc.sock`
pub fn resolve_linux_socket_path(
    custom: Option<&str>,
    xdg_runtime_dir: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&str>,
    uid: u32,
) -> String {
    if let Some(c) = custom.filter(|s| !s.is_empty()) {
        return c.to_string();
    }
    if let Some(rt) = xdg_runtime_dir.filter(|s| s.starts_with('/')) {
        return join_unix(rt, "TextVN/ipc.sock");
    }
    if let Some(cfg) = xdg_config_home.filter(|s| s.starts_with('/')) {
        return join_unix(cfg, "TextVN/ipc.sock");
    }
    if let Some(h) = home.filter(|s| !s.is_empty()) {
        return join_unix(h, ".config/TextVN/ipc.sock");
    }
    format!("/tmp/textvn-{uid}/ipc.sock")
}

/// Giải đường dẫn Unix socket IPC trên macOS (`~/Library/Application Support/TextVN/ipc.sock`).
pub fn resolve_macos_socket_path(custom: Option<&str>, home: Option<&str>) -> Option<String> {
    if let Some(c) = custom.filter(|s| !s.is_empty()) {
        return Some(c.to_string());
    }
    home.filter(|s| !s.is_empty())
        .map(|h| join_unix(h, "Library/Application Support/TextVN/ipc.sock"))
}

fn join_unix(base: &str, suffix: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.is_empty() {
        format!("/{suffix}")
    } else {
        format!("{trimmed}/{suffix}")
    }
}

/// Phân tích `state.json` của tray (`global_enabled`, `assist_enabled`, `apps`)
/// thành map trạng thái chuẩn (`"*"`, `"assist"`, `<app lowercase>`).
pub fn parse_state_json(bytes: &[u8]) -> Option<BTreeMap<String, bool>> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let mut states = BTreeMap::new();
    if let Some(global) = value.get("global_enabled").and_then(|v| v.as_bool()) {
        states.insert(GLOBAL_APP_ID.to_string(), global);
    }
    if let Some(assist) = value.get("assist_enabled").and_then(|v| v.as_bool()) {
        states.insert(ASSIST_APP_ID.to_string(), assist);
    }
    if let Some(apps) = value.get("apps").and_then(|v| v.as_object()) {
        for (app, enabled) in apps {
            if let Some(enabled) = enabled.as_bool() {
                states.insert(app.to_ascii_lowercase(), enabled);
            }
        }
    }
    Some(states)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_filter_blocks_user_content_and_admin_messages() {
        assert!(allowed_in_sandbox(&Message::Hello {
            pid: 1,
            abi: 1,
            version: "1.0.0".into(),
        }));
        assert!(allowed_in_sandbox(&Message::GetSnapshot));
        assert!(allowed_in_sandbox(&Message::Subscribe { pid: 1 }));
        assert!(allowed_in_sandbox(&Message::ToggleViEn {
            app_id: "calc.exe".into(),
            enabled: true,
        }));
        assert!(allowed_in_sandbox(&Message::ToggleGlobal));
        assert!(allowed_in_sandbox(&Message::ToggleAssist {
            enabled: false
        }));
        assert!(allowed_in_sandbox(&Message::Ping));
        assert!(allowed_in_sandbox(&Message::CrashReport {
            code: "E1".into(),
            count: 1,
        }));

        assert!(!allowed_in_sandbox(&Message::StyleUpdate {
            json: "{}".into(),
        }));
        assert!(!allowed_in_sandbox(&Message::AssistReport {
            issues: vec![],
            auto_fixed: 0,
        }));
        assert!(!allowed_in_sandbox(&Message::AssistNotification {
            title: "t".into(),
            message: "m".into(),
        }));
        assert!(!allowed_in_sandbox(&Message::Shutdown));
        assert!(!allowed_in_sandbox(&Message::ToggleMath));
        assert!(!allowed_in_sandbox(&Message::CycleMathMode));
        assert!(!allowed_in_sandbox(&Message::OpenEmojiPicker));

        assert_eq!(
            encode_outgoing(&Message::StyleUpdate { json: "{}".into() }, true),
            Ok(None)
        );
        assert!(matches!(
            encode_outgoing(&Message::StyleUpdate { json: "{}".into() }, false),
            Ok(Some(_))
        ));
    }

    #[test]
    fn frame_buffer_handles_short_reads_coalesced_frames_and_oversize() {
        let m1 = Message::Ping;
        let m2 = Message::ToggleGlobal;
        let mut combined = encode_frame(&m1).unwrap();
        let f2 = encode_frame(&m2).unwrap();
        let split = combined.len() + 2;
        combined.extend_from_slice(&f2);

        let mut fb = FrameBuffer::new();
        fb.push(&combined[..3]);
        assert_eq!(fb.next_frame(), Ok(None));

        fb.push(&combined[3..split]);
        assert_eq!(fb.next_frame(), Ok(Some(m1)));
        assert_eq!(fb.next_frame(), Ok(None));

        fb.push(&combined[split..]);
        assert_eq!(fb.next_frame(), Ok(Some(m2)));
        assert!(fb.is_empty());

        let oversize_len = ((MAX_FRAME_BYTES + 1) as u32).to_le_bytes();
        fb.push(&oversize_len);
        assert_eq!(fb.next_frame(), Err(CodecError::Oversize));
        assert!(fb.is_empty());
    }

    #[test]
    fn state_machine_applies_snapshot_updates_and_app_switch() {
        let mut st = IpcClientState::new("chrome.exe");
        assert!(st.is_enabled());
        assert!(st.is_assist_enabled());

        let mut map = BTreeMap::new();
        map.insert("*".into(), true);
        map.insert("assist".into(), false);
        map.insert("chrome.exe".into(), false);
        map.insert("notepad.exe".into(), true);

        let eff = st.apply_message(&Message::Snapshot {
            config_version: 7,
            state: map,
            appdb_version: 1,
            channel: "stable".into(),
        });
        assert_eq!(eff.config_version_changed, Some(7));
        assert_eq!(eff.effective_enabled_changed, Some(false));
        assert_eq!(eff.assist_changed, Some(false));
        assert!(!st.is_enabled());
        assert!(!st.is_assist_enabled());
        assert_eq!(st.config_version(), 7);

        st.set_app_id("Notepad.EXE");
        assert!(st.is_enabled());

        let eff2 = st.apply_message(&Message::StateUpdate {
            app_id: "*".into(),
            enabled: false,
            version: 8,
        });
        assert_eq!(eff2.config_version_changed, Some(8));
        assert_eq!(eff2.effective_enabled_changed, Some(false));
        assert!(!st.is_enabled());

        let eff3 = st.apply_message(&Message::ConfigReload { version: 9 });
        assert_eq!(eff3.config_version_changed, Some(9));
        assert_eq!(st.config_version(), 9);
    }

    #[test]
    fn backoff_doubles_to_cap_and_resets() {
        let mut b = ReconnectBackoff::default();
        assert_eq!(b.next_delay(), Duration::from_millis(100));
        assert_eq!(b.next_delay(), Duration::from_millis(200));
        assert_eq!(b.next_delay(), Duration::from_millis(400));
        assert_eq!(b.next_delay(), Duration::from_millis(800));
        assert_eq!(b.next_delay(), Duration::from_millis(1600));
        assert_eq!(b.next_delay(), Duration::from_millis(3200));
        assert_eq!(b.next_delay(), Duration::from_millis(5000));
        assert_eq!(b.next_delay(), Duration::from_millis(5000));
        b.reset();
        assert_eq!(b.next_delay(), Duration::from_millis(100));
    }

    #[test]
    fn linux_and_macos_socket_path_resolution_matches_c9_guards() {
        assert_eq!(
            resolve_linux_socket_path(
                Some("/custom/sock"),
                Some("/run/user/1000"),
                None,
                None,
                1000
            ),
            "/custom/sock"
        );
        assert_eq!(
            resolve_linux_socket_path(
                None,
                Some("/run/user/1000"),
                Some("/cfg"),
                Some("/home/u"),
                1000
            ),
            "/run/user/1000/TextVN/ipc.sock"
        );
        // C9: relative XDG_RUNTIME_DIR and XDG_CONFIG_HOME are ignored.
        assert_eq!(
            resolve_linux_socket_path(
                None,
                Some("rel/run"),
                Some("rel/cfg"),
                Some("/home/u"),
                1000
            ),
            "/home/u/.config/TextVN/ipc.sock"
        );
        assert_eq!(
            resolve_linux_socket_path(None, None, Some("/xdg/cfg"), Some("/home/u"), 1000),
            "/xdg/cfg/TextVN/ipc.sock"
        );
        assert_eq!(
            resolve_linux_socket_path(None, None, None, None, 1000),
            "/tmp/textvn-1000/ipc.sock"
        );

        assert_eq!(
            resolve_macos_socket_path(None, Some("/Users/dev")),
            Some("/Users/dev/Library/Application Support/TextVN/ipc.sock".into())
        );
        assert_eq!(
            resolve_macos_socket_path(Some("/tmp/dev.sock"), Some("/Users/dev")),
            Some("/tmp/dev.sock".into())
        );
        assert_eq!(resolve_macos_socket_path(None, Some("")), None);
    }

    #[test]
    fn parse_state_json_matches_tray_schema() {
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
}
