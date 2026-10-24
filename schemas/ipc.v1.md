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
| Client → server | `Hello { pid, abi, version }`, `GetSnapshot`, `Subscribe { pid }`, `ToggleViEn { app_id, enabled }`, `CrashReport { code, count }`, `Ping`, `ToggleAssist { enabled }`, `AssistReport { issues, auto_fixed }` |
| Server → client | `Snapshot { config_version, state, appdb_version, channel }`, `Ack`, `ConfigReload { version }`, `StateUpdate { app_id, enabled, version }`, `Pong { uptime_ms }`, `AssistNotification { title, message }` |

Kiểu trường (khớp `textvn-ipc`, CR-34): `pid`, `abi`, `config_version`,
`appdb_version`, `count`, `uptime_ms` và `version` của `ConfigReload`/`StateUpdate`
là **số nguyên** JSON; `Hello.version` (phiên bản sản phẩm) là chuỗi; `state` là
object `string → bool`; `enabled` là bool; còn lại là chuỗi.

`CrashReport.code` là mã phân loại, không được chứa text đang gõ. `state` là map
`app_id → enabled`; `StateUpdate` là push cho client đã Subscribe. `app_id = "*"`
là trạng thái **toàn cục** (bật/tắt tiếng Việt) trong `ToggleViEn`, `StateUpdate`
và `state` — một quy ước cho mọi nền tảng; server macOS chuẩn hoá `""` của IMK bản
cũ về `"*"`. Thêm message
mới yêu cầu protocol version/schema review trước.

### Trợ lý ngữ pháp (P4-2 · AS-023/D20)

- `ToggleAssist { enabled }` — kill-switch 1 bước: tray lưu vào `state.json`
  (`assist_enabled`), broadcast `StateUpdate { app_id: "assist" }` và trả
  `Snapshot` (trong `state` có khoá `"assist"`). Tắt ⇒ adapter **không** kiểm tra
  nữa và tray **không** hiện toast (kể cả khi `assist.mode` khác `off`).
- `AssistReport { issues, auto_fixed }` — **kết quả** kiểm tra do adapter gửi lên
  tray. Mỗi phần tử `issues[]`:

  | Trường | Kiểu | Ý nghĩa |
  |---|---|---|
  | `kind` | chuỗi | Mã loại lỗi ổn định (`duplicate_word`, `space_before_punct`, `space_after_punct`, `double_space`, `lowercase_sentence_start`, `diacritic_typo`, `sentence_repeat`, `double_negative`, `comma_at_end`, `spelling`, `article_an`) |
  | `span_start`, `span_end` | số nguyên | Vùng lỗi theo **ký tự Unicode** (inclusive/exclusive); adapter đổi sang UTF-16 ở biên FFI (D27/D29) |
  | `message` | chuỗi | Thông điệp tiếng Việt ngắn do engine sinh |
  | `suggestions` | mảng chuỗi | Gợi ý thay thế cho đúng vùng lỗi (có thể rỗng) |
  | `auto_fixable` | bool | Chỉ luật objective mới `true` (D7) |
  | `confidence` | số nguyên 0–100 | Điểm tin cậy (D28) |

  `auto_fixed` = số lỗi adapter **đã** tự sửa (Mode 2 — toast "Đã tự sửa N lỗi").
  Các trường `suggestions`/`auto_fixable`/`confidence`/`auto_fixed` có
  `#[serde(default)]` nên payload tối giản vẫn decode được.

> **Privacy (D8 · S2): `AssistReport` KHÔNG mang câu người dùng vừa gõ.** Không có
> trường `text`/`sentence`/`content`; tray chỉ nhận loại lỗi, span, thông điệp và
> gợi ý. Lưu ý `message`/`suggestions` có thể chứa **đoạn** của câu (từ bị lỗi và
> từ thay thế) vì engine sinh chúng từ câu — vì vậy: **không log**, không ghi
> đĩa, không gửi ra ngoài, không đưa vào accessibility tree. Thêm trường chứa
> toàn bộ câu là thay đổi protocol phải review lại.

- `AssistNotification { title, message }` — thông báo đơn giản (giữ cho tương
  thích ngược); tray hiện balloon **chỉ khi kill-switch đang bật** (D20). Dùng
  `AssistReport` cho kết quả kiểm tra mới (có lọc theo `assist.mode` + chống spam).

> **Tương thích ngược:** mọi variant mới được thêm **ở cuối** `Message`; JSON của
> message cũ không đổi. Bản tray cũ gặp `AssistReport` sẽ coi là protocol
> violation (đóng kết nối) — đúng quy ước "message lạ thì disconnect" ở trên; do
> đó adapter chỉ được gửi `AssistReport` khi peer đã `Hello` với `abi` hỗ trợ
> (kiểm ở tầng adapter).


## Implementation

[`textvn-ipc`](../ipc/src/lib.rs) owns JSON model và codec. Adapter phải dùng
`encode_frame`/`decode_exact_frame`; không được tự diễn giải JSON hay bỏ qua
frame lỗi.
