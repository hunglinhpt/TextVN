# P0-3 — Config schema · Preset (appdb) · Strategy model · IPC

> Phần 0 · Nguồn sự thật cho `schemas/config.v1.schema.json`, `schemas/appdb.v1.schema.json`,
> `schemas/ipc.v1.md`. Mọi nền tảng dùng **cùng một định dạng file** (kế thừa GoTiengViet parity).

## 1. Config người dùng — đường dẫn per-OS (1 nơi duy nhất cho mỗi OS)

| OS | `config.json` / `appdb.json` / `state.json` | Log |
|---|---|---|
| Windows | `%APPDATA%\VietIME\` | `%LOCALAPPDATA%\VietIME\logs\` |
| macOS | `~/Library/Application Support/VietIME/` | `~/Library/Logs/VietIME/` |
| Linux | `~/.config/VietIME/` (config/state/appdb — chốt `P3-0 §2`) | `~/.local/state/VietIME/log/` |

*(Nhãn tiêu đề bảng trên tham chiếu Windows cho ngắn; nội dung bảng là nguồn sự thật cho mọi OS.)*

### 1.1 Bảng trường (bắt buộc đánh số version để migrate)

| Trường | Type | Default | Ghi chú |
|---|---|---|---|
| `config_version` | int | `1` | migrate tự động, backup `.bak`. **Bắt buộc phải có** — thiếu thì `config: invalid schema` (không đoán version) |
| `enabled` | bool | `true` | master switch |
| `method` | `"telex"\|"vni"\|"viqr"\|"simple_telex"` | `"telex"` | |
| `diacritic_style` | `"new"\|"old"` | `"new"` | `hoà` vs `hòa` (B12) |
| `free_marking` | bool | `true` | gõ dấu tự do kiểu UniKey |
| `auto_restore_english` | bool | `true` | fix B5 |
| `auto_capitalize` | bool | `true` | sau `. ! ?` + Enter |
| `macro_trigger` | `"tab"\|"space"` | `"tab"` | gõ tắt/emoji expand |
| `allow_macro_when_vi_off` | bool | `false` | kế thừa EVKey spec #5 |
| `output_charset` | `"unicode_precomposed"\|"unicode_decomposed"\|"tcvn3"\|"vni_windows"` | `"unicode_precomposed"` | kế thừa WinVNKey breadth |
| `hotkeys` | object | xem dưới | |
| `macros[]` | `[{trigger, expand, when}]` | `[]` | `when: "always"\|"vi_on"` |
| `emoji[]` | `[{trigger, glyph}]` | `[]` | |
| `english_words[]` | `["text", ...]` (chữ cái ASCII) | `[]` | nhánh từ điển của B5 — **mặc định rỗng, opt-in** (xem `data/stop_en.txt`; `test`→`tết` là ca mơ hồ nên không bật sẵn) |
| `ignore_apps[]` | `[{match:"exe"\|"bundle"\|"class", value, mode:"disable_vi"\|"force_vi"\|"passthrough"}]` | `[]` | EVKey spec #1 / P2 persona |
| `app_overrides` | `{app_id: {method?, diacritic_style?, ...}}` | `{}` | per-app ghi đè |
| `secure_fields` | `"always_pass"` | `"always_pass"` | **không cho đổi** (giữ S3) |
| `diagnostics` | `{log_level:"error"\|"warn"\|"info"\|"debug", export_repro:false}` | `warn` | `export_repro` chỉ bật khi user bấm "Báo lỗi" — ghi **chuỗi phím đã ẩn danh hoá** |
| `updates` | `{channel:"stable"\|"beta", auto:bool}` | stable, `true` | |
| `suggest` | `{enabled:false, provider:"local_ngram", ollama:null}` | tắt hoàn toàn | opt-in, tắt mặc định |

`hotkeys` mặc định:
```json
"hotkeys": {
  "toggle_vi_en": "Ctrl+Shift+Space",
  "toggle_method": "Ctrl+Shift+K",
  "restore_last": "Escape",
  "open_settings": "Ctrl+Shift+O"
}
```
> Hotkey **không được trùng phím tắt hệ thống** (check trong `vietime doctor`, bug B6):
> list blacklist `Ctrl+C/V/X/Z`, `Alt+Tab`, `Win+L`, `Cmd+Q`… validator từ chối với lỗi rõ ràng.

### 1.2 Ví dụ đầy đủ

```json
{
  "config_version": 1,
  "enabled": true,
  "method": "telex",
  "diacritic_style": "new",
  "free_marking": true,
  "auto_restore_english": true,
  "auto_capitalize": true,
  "macro_trigger": "tab",
  "allow_macro_when_vi_off": false,
  "output_charset": "unicode_precomposed",
  "hotkeys": { "toggle_vi_en": "Ctrl+Shift+Space", "toggle_method": "Ctrl+Shift+K",
               "restore_last": "Escape", "open_settings": "Ctrl+Shift+O" },
  "macros": [ { "trigger": "cty", "expand": "Công ty TNHH", "when": "always" } ],
  "emoji":  [ { "trigger": ":smile", "glyph": "😊" } ],
  "ignore_apps": [ { "match": "exe", "value": "code.exe", "mode": "disable_vi" } ],
  "app_overrides": { "chrome.exe": { "method": "telex" } },
  "secure_fields": "always_pass",
  "diagnostics": { "log_level": "warn", "export_repro": false },
  "updates": { "channel": "stable", "auto": true },
  "suggest": { "enabled": false, "provider": "local_ngram", "ollama": null }
}
```

**Validation:** `vietime-config` validate bằng JSON Schema **trước** đưa vào engine.
Sai schema → giữ file cũ, tạo `config.invalid.json` để debug, engine chạy default, tray hiện banner.
Hot-reload: tray watch file (debounce 300ms) → gửi `ConfigReload` qua IPC (§5) → adapter gọi `ime_reload_config`.

## 2. Preset theo ứng dụng — `appdb` (`schemas/appdb.v1.schema.json`)

### 2.1 Cấu trúc

```json
{
  "appdb_version": 1,
  "min_engine_version": "1.0.0",
  "updated_at": "2026-09-27T00:00:00Z",
  "entries": [
    {
      "id": "chrome.address-bar",
      "match": { "any": [
          { "exe": "chrome.exe" }, { "exe": "msedge.exe" },
          { "bundle": "com.google.Chrome" }, { "class": "Chrome_WidgetWin_1" } ] },
      "when": { "field_role": ["address_bar", "search", "combo"] },
      "strategy": "SelectionReplace",
      "inject_mode": "vk_then_unicode",
      "notes": "B1: autocomplete dính chữ ở thanh địa chỉ",
      "enabled_default": true,
      "min_os": null
    },
    {
      "id": "windowsterminal.body",
      "match": { "any": [ { "exe": "windowsterminal.exe" }, { "exe": "wezterm-gui.exe" },
                          { "exe": "code.exe", "and_subprocess": false } ] },
      "when": { "field_role": ["terminal", "body"] },
      "strategy": "ForwardAsCommit",
      "inject_mode": "unicode",
      "notes": "B8: terminal không có preedit",
      "enabled_default": true
    }
  ]
}
```

- `engine_owner`: `"tsf"|"hook"|"imk"|"tap"|"ibus"|"fcitx5"|"x11"` (optional; mặc định theo OS: Win=`tsf`,
  macOS=`imk`, Linux=`ibus`) — adapter nào được xử lý app này; mục đích là chống xử lý đôi
  (luật: `P1-2 §6` Windows, `P2-2 §6` macOS, `P3-3 §6.1` Linux).
- `strategy`: `"Preedit" | "BackspaceType" | "SelectionReplace" | "ForwardAsCommit" | "Passthrough"` (P0-3 §3).
- **Mapping `field_role` (JSON) ↔ `IME_FIELD_*` (FFI) — bắt buộc 1-1:**

  | JSON (`field_role`) | FFI | JSON | FFI |
  |---|---|---|---|
  | `unknown` | `IME_FIELD_UNKNOWN (0)` | `combo` | `IME_FIELD_COMBO (5)` |
  | `body` | `IME_FIELD_BODY (1)` | `textarea` | `IME_FIELD_TEXTAREA (7)` |
  | `editbox` | `IME_FIELD_EDITBOX (2)` | `web` | `IME_FIELD_WEB (8)` |
  | `address_bar` | `IME_FIELD_ADDRESS_BAR (3)` | `terminal` | `IME_FIELD_TERMINAL (9)` |
  | `search` | `IME_FIELD_SEARCH (4)` | `secure` | `IME_FIELD_SECURE (10)` |
  | `candidate` | `IME_FIELD_CANDIDATE (6)` | | |

- `inject_mode` (adapter inject: hook/tap/x11): `"unicode"` (gửi chuỗi Unicode — KEYEVENTF_UNICODE/CGEvent/…) | `"vk_then_unicode"` (gửi VK/keycode thật cho ASCII → app nhìn thấy key event thật, quan trọng với autocomplete) | `"selection"` (Shift+Left rồi chèn) | `"keycode_ascii"` (chỉ keycode theo layout hiện tại — X11, dùng khi spike chốt, xem `P3-3 §5.4`).
- **Matching:** adapter chuyển OS → `app_id` chuẩn (Win: `exe` thường chữ thường; macOS: bundle id; Linux: WM_CLASS) → tìm entry khớp **theo thứ tự file, entry đầu khớp thắng**; `when.field_role` lọc thêm.
- **User override** (`%APPDATA%\VietIME\appdb.json`) merge: field nào user đặt → **giành quyền tuyệt đối**.

### 2.2 Ký số & cập nhật

- Bản release kèm `data/appdb.default.json` + `data/appdb.default.json.sig` (Ed25519, key `data/preset.pub`).
- Update preset: tray tải qua updater service (`P1-4 §5`) → verify chữ ký + `appdb_version` không giảm → ghi đè → hot-reload. **Sai chữ ký → bỏ, log warning, không crash.**
- Preset mới có `min_engine_version` > phiên bản engine đang chạy → **bỏ qua entry đó** (không reject cả file).

## 3. Strategy model — cách chọn cách xuất chữ

### 3.1 Quy tắc phân quyền (đọc từ trên xuống, match là dừng)

```
1. ctx.secure == 1                     → Passthrough            (S3)
2. !ctx.enabled (VN tắt / ignore list) → Passthrough            (macro vẫn chạy nếu allow_macro_when_vi_off)
3. User preset override (user appdb.json, §2.1) → strategy user đã khai (thắng mọi preset hệ thống)
4. preset hệ thống match (app_id + field_role)  → strategy của preset
5. field_role-derived default (chỉ khi adapter có cap tương ứng, `ctx.caps`):
      address_bar|search|combo → SelectionReplace   (cần IME_CAP_SELECTION | IME_CAP_FIELD_DETECT)
      terminal                 → ForwardAsCommit
      candidate                → SelectionReplace   (Excel cell/IDE complete — B1/B3)
      secure                   → Passthrough
      khác (body/editbox/web/textarea/unknown) → Preedit nếu IME_CAP_PREEDIT, ngược lại BackspaceType
6. fallback cuối                       → BackspaceType
```

**Adapter báo cáo capability** (qua `ime_context_v1.caps` — xem P0-2 §1): có preedit không, có UIA/AX không.
Không có capability → không được chọn `Preedit`/`SelectionReplace` (an toàn: `BackspaceType`).

**Quy tắc downgrade bắt buộc** (khi preset chọn strategy mà adapter không có cap):
`Preedit → BackspaceType` · `SelectionReplace → BackspaceType` · `ForwardAsCommit/Passthrough → giữ nguyên`.
Adapter **không được** chạy strategy thiếu cap (sẽ gõ sai/thoát chữ ảo) và không được fail — cứ downgrade im lặng,
ghi `debug` log.

### 3.2 Bảng lỗi đã biết → strategy kỳ vọng (dùng làm test oracle)

| Tình huống | Strategy | Test |
|---|---|---|
| Chrome/Edge/Safari address bar | SelectionReplace | `corpus/win/chrome_address_bar_*` |
| Excel cell autocomplete | SelectionReplace | `corpus/win/excel_cell_*` |
| VS Code editor | BackspaceType + ignore default (P2) | `corpus/win/vscode_body_*` |
| Windows Terminal/kitty/iTerm | ForwardAsCommit | `corpus/win/wt_*` |
| Slack/Zalo/Telegram nhập & Enter | Preedit/BackspaceType + **COMMIT** khi Enter (B2) | `corpus/win/chat_enter_*` |
| Google Docs (web canvas) | BackspaceType, debounce ≤ 5ms | `corpus/win/gdocs_*` |
| Ô mật khẩu | Passthrough | manual + UIA test |

### 3.3 Chi phí (budget mọi adapter phải tuân)

| Thành phần | Budget |
|---|---|
| resolve strategy (sau khi có `FieldContext`) | < 0.05 ms (hashmap + role match, cache theo window) |
| `ime_key` toàn bộ | < 0.5 ms p99 |
| end-to-end (key → glyph) | < 10 ms p99 |
| BackspaceType 1 ký tự (gồm 2×SendInput) | < 4 ms |
| SelectionReplace n ký tự | < 8 ms (n ≤ 16) |

## 4. State đồng bộ giữa các process

Trạng thái người dùng đổi chỗ nào? → **tray là source of truth** (file `state.json` chỉ chứa
`{app_id: {enabled, method}}`, không chứa text).

| Sự kiện | Cách lan truyền |
|---|---|
| User đổi config trong Settings | tray ghi `config.json` → broadcast `ConfigReload{version}` → adapter `ime_reload_config` |
| User toggle VN/ANH cho app hiện tại | tray ghi `state.json` → broadcast `StateUpdate{app_id, enabled, version}` |
| Adapter mới kết nối (mở app mới) | gọi `GetSnapshot` → nhận config_version + state + presets_version |
| Tray không chạy | adapter đọc file trực tiếp khi khởi tạo; **không block** |

## 5. IPC — `\\.\pipe\vietime-ipc-v1` (`schemas/ipc.v1.md`)

- **Transport per-OS — cùng 1 schema (`schemas/ipc.v1.md`):**

  | OS | Endpoint | Bảo vệ |
  |---|---|---|
  | Windows | named pipe `\\.\pipe\vietime-ipc-v1` (message mode, `CreateNamedPipe`) | DACL = chỉ current user SID |
  | macOS | unix socket `~/Library/Application Support/VietIME/ipc.sock` | dir 0700, sock 0600 + check uid |
  | Linux | unix socket `~/.config/VietIME/ipc.sock` (chốt `P3-0 §2`) | 0600 + `SO_PEERCRED` |

  **Không TCP/HTTP ở mọi OS.**
- **Codec:** 1 message = 1 frame JSON (u32 length prefix + UTF-8), validate schema trước xử lý.
- **Server:** `vietime-tray.exe`. **Client:** mỗi instance `vietime-tsf.dll` (theo process), `vietime-hook`.

| Message (client→server) | Payload | Server trả |
|---|---|---|
| `Hello` | `{pid, abi, version}` | `Snapshot{config_version, state, appdb_version, channel}` |
| `GetSnapshot` | `{}` | `Snapshot{...}` |
| `Subscribe` | `{pid}` | ack (server đẩy push khi state đổi) |
| `ToggleViEn` | `{app_id, enabled}` | ack + broadcast |
| `CrashReport` | `{code, count}` (không text!) | ack → tray hiện "engine gặp lỗi, đã fail-open" |
| `Ping` | `{}` | `Pong{uptime_ms}` |

**Bảo mật:** client xác thực bằng `GetNamedPipeClientProcessId` → check process path có nằm trong
install dir + chữ ký hợp lệ không (nếu không → disconnect + log). Timeout 2s → adapter **vẫn chạy được offline**
(chỉ mất tính năng sync, không mất gõ).

## 6. Quy tắc chống loop (quan trọng với hook)

1. Adapter set `ime_key.is_injected = 1` cho mọi phím do chính nó bơm (hook đọc cờ `LLKHF_INJECTED`).
2. Engine trả `PASS` ngay nếu `is_injected==1` (**bắt buộc** — test trong `corpus/shared/loop_guard`).
3. Nếu adapter không đọc được cờ injected (trường hợp hiếm) → guard theo watchdog: > 200 key/giây bất thường → tự disable hook + báo tray.

## 7. Test tối thiểu cho phần này (Definition of Done của P0-3)

- [ ] `config.v1.schema.json` validate được mọi ví dụ trong §1 + file config của người dùng thật (fuzz 60s không panic).
- [ ] `appdb` merge: user override thắng preset; entry `min_engine_version` quá cao bị bỏ qua.
- [ ] resolve strategy đúng bảng §3.2 (unit test `strategy::resolve` với mọi `field_role` × capability).
- [ ] IPC: message lạ/răng cưa → disconnect, không panic; DACL chỉ user (test tích hợp Windows).
- [ ] Loop guard: corpus `loop_guard.keys` pass.
