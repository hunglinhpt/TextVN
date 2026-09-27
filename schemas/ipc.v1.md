# TextVN IPC v1

SPDX-License-Identifier: GPL-3.0-or-later

## Wire framing

Mỗi message là một frame độc lập: `u32` little-endian length, tiếp theo đúng
`length` byte JSON UTF-8. `length` tối đa 65,536 byte. Short read được buffer;
length quá giới hạn, UTF-8/JSON sai, message type không biết hoặc byte thừa đều
là protocol violation: đóng kết nối, không retry frame đó.

Transport phải giữ local-only: Windows named pipe với DACL current-user; macOS
unix socket với `getpeereid`; Linux unix socket với `SO_PEERCRED`. Không dùng
TCP/HTTP. Authentication peer và timeout 2s là trách nhiệm transport, không
nằm trong `textvn-ipc`.

## Messages

Mọi JSON có discriminator `type`. Danh sách v1 đóng:

| Hướng | JSON payload |
|---|---|
| Client → server | `Hello { pid, abi, version }`, `GetSnapshot`, `Subscribe { pid }`, `ToggleViEn { app_id, enabled }`, `CrashReport { code, count }`, `Ping` |
| Server → client | `Snapshot { config_version, state, appdb_version, channel }`, `Ack`, `ConfigReload { version }`, `StateUpdate { app_id, enabled, version }`, `Pong { uptime_ms }` |

`CrashReport.code` là mã phân loại, không được chứa text đang gõ. `state` là map
`app_id → enabled`; `StateUpdate` là push cho client đã Subscribe. Thêm message
mới yêu cầu protocol version/schema review trước.

## Implementation

[`textvn-ipc`](../ipc/src/lib.rs) owns JSON model và codec. Adapter phải dùng
`encode_frame`/`decode_exact_frame`; không được tự diễn giải JSON hay bỏ qua
frame lỗi.
