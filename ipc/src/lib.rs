// SPDX-License-Identifier: GPL-3.0-or-later
//! Codec IPC v1 thuần: frame `u32` little-endian + JSON UTF-8.
//!
//! Crate này không mở named-pipe/socket và không xác thực peer: đó là trách
//! nhiệm adapter theo OS. Mọi frame vượt giới hạn, răng cưa, UTF-8/JSON sai hay
//! message lạ đều là lỗi protocol để transport disconnect thay vì cố đoán.
//! Ngoại lệ duy nhất là [`pipe_client_options`]: cấu hình mở đầu client của pipe
//! Windows ở MỘT chỗ để không call site nào quên cờ chống impersonation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Giới hạn chống cấp phát theo length prefix từ peer không tin cậy.
pub const MAX_FRAME_BYTES: usize = 64 * 1024;

/// Tên pipe IPC v1 trên Windows (`docs/00-INDEX.md` §4).
pub const PIPE_NAME: &str = r"\\.\pipe\textvn-ipc-v1";

/// `OpenOptions` cho đầu **client** của pipe IPC: đọc + ghi, và trên Windows đặt
/// `SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION`.
///
/// Mặc định `CreateFileW` lên named pipe cấp cho server quyền **impersonate** client.
/// TIP nằm trong mọi process (kể cả app chạy quyền admin) và thử kết nối lại liên tục
/// khi tray chưa chạy — process nào chiếm tên pipe trước sẽ nhận kết nối và có thể
/// `ImpersonateNamedPipeClient`. Mức Identification vẫn cho server đọc danh tính
/// (`GetNamedPipeClientProcessId` không cần gì thêm) nhưng không hành động thay client.
pub fn pipe_client_options() -> std::fs::OpenOptions {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        /// `SecurityIdentification << 16` (WinBase.h); std tự OR `SECURITY_SQOS_PRESENT`.
        const SECURITY_IDENTIFICATION: u32 = 1 << 16;
        options.security_qos_flags(SECURITY_IDENTIFICATION);
    }
    options
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    Truncated,
    Oversize,
    InvalidUtf8,
    InvalidMessage,
    TrailingBytes,
}

/// Một message `ipc.v1`. Field `type` là discriminator ổn định trên wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Hello {
        pid: u32,
        abi: u32,
        version: String,
    },
    GetSnapshot,
    Subscribe {
        pid: u32,
    },
    ToggleViEn {
        app_id: String,
        enabled: bool,
    },
    ToggleGlobal,
    Shutdown,
    CrashReport {
        code: String,
        count: u32,
    },
    Ping,
    Snapshot {
        config_version: u64,
        state: BTreeMap<String, bool>,
        appdb_version: u32,
        channel: String,
    },
    Ack,
    ConfigReload {
        version: u64,
    },
    StateUpdate {
        app_id: String,
        enabled: bool,
        version: u64,
    },
    Pong {
        uptime_ms: u64,
    },
}

/// Serialize một message thành frame hoàn chỉnh, gồm prefix length little-endian.
pub fn encode_frame(message: &Message) -> Result<Vec<u8>, CodecError> {
    let json = serde_json::to_vec(message).map_err(|_| CodecError::InvalidMessage)?;
    if json.len() > MAX_FRAME_BYTES || json.len() > u32::MAX as usize {
        return Err(CodecError::Oversize);
    }
    let mut frame = Vec::with_capacity(4 + json.len());
    frame.extend_from_slice(&(json.len() as u32).to_le_bytes());
    frame.extend_from_slice(&json);
    Ok(frame)
}

/// Decode đúng một frame. Caller phải xử lý short read bằng cách buffer đến đủ
/// `4 + length`; trailing byte là protocol violation, không phải frame kế tiếp.
pub fn decode_exact_frame(frame: &[u8]) -> Result<Message, CodecError> {
    if frame.len() < 4 {
        return Err(CodecError::Truncated);
    }
    let length = u32::from_le_bytes(frame[..4].try_into().expect("đã check len")) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(CodecError::Oversize);
    }
    let actual = &frame[4..];
    if actual.len() < length {
        return Err(CodecError::Truncated);
    }
    if actual.len() > length {
        return Err(CodecError::TrailingBytes);
    }
    let text = std::str::from_utf8(actual).map_err(|_| CodecError::InvalidUtf8)?;
    serde_json::from_str(text).map_err(|_| CodecError::InvalidMessage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_wire_is_json_utf8() {
        let message = Message::ToggleViEn {
            app_id: "chrome.exe".into(),
            enabled: false,
        };
        let frame = encode_frame(&message).unwrap();
        assert_eq!(
            u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize,
            frame.len() - 4
        );
        assert_eq!(
            std::str::from_utf8(&frame[4..]).unwrap(),
            r#"{"type":"ToggleViEn","app_id":"chrome.exe","enabled":false}"#
        );
        assert_eq!(decode_exact_frame(&frame), Ok(message));
    }

    #[test]
    fn toggle_global_round_trip() {
        let message = Message::ToggleGlobal;
        let frame = encode_frame(&message).unwrap();
        assert_eq!(
            std::str::from_utf8(&frame[4..]).unwrap(),
            r#"{"type":"ToggleGlobal"}"#
        );
        assert_eq!(decode_exact_frame(&frame), Ok(message));
    }

    #[test]
    fn shutdown_round_trip() {
        let message = Message::Shutdown;
        let frame = encode_frame(&message).unwrap();
        assert_eq!(
            std::str::from_utf8(&frame[4..]).unwrap(),
            r#"{"type":"Shutdown"}"#
        );
        assert_eq!(decode_exact_frame(&frame), Ok(message));
    }

    #[test]
    fn malformed_unknown_or_oversized_frame_is_rejected() {
        assert_eq!(decode_exact_frame(&[1, 2, 3]), Err(CodecError::Truncated));
        assert_eq!(
            decode_exact_frame(&[3, 0, 0, 0, b'{']),
            Err(CodecError::Truncated)
        );
        let unknown = br#"{"type":"SetMode"}"#;
        let mut frame = (unknown.len() as u32).to_le_bytes().to_vec();
        frame.extend_from_slice(unknown);
        assert_eq!(decode_exact_frame(&frame), Err(CodecError::InvalidMessage));
        assert_eq!(
            decode_exact_frame(&[1, 0, 0, 0, b' ', b' ']),
            Err(CodecError::TrailingBytes)
        );
        assert_eq!(
            decode_exact_frame(&((MAX_FRAME_BYTES as u32 + 1).to_le_bytes())),
            Err(CodecError::Oversize)
        );
    }
}
