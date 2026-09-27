# P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết

> WS3 · crate `textvn-win-hook` → **`textvn-hook.exe` (process riêng)** — theo `P0-1 §1` (F0-001).
> Vai trò: chế độ tương thích cho app **không dùng được TSF** (game, một số app legacy, app elevated).
> Mục tiêu phụ: không bao giờ trở thành bottleneck hay vòng lặp tự gõ.

## 1. Quyết định & phạm vi

| Quyết định | Nội dung |
|---|---|
| Process riêng | Crash/tr deadlock **không** kéo sập tray hay app khác; tray spawn + watchdog restart <500ms (`P1-4 §6`) |
| TSF vẫn primary | Hook là **fallback có điều kiện**, không phải chế độ mặc định cho mọi app |
| `engine_owner` | Mỗi app có chủ sở hữu engine: `"tsf"` (mặc định) hoặc `"hook"` — **tránh xử lý đôi** (§6) |
| `hook.mode` | `off` \| `auto` (mặc định) \| `always`. `auto` = chỉ xử lý khi foreground app thuộc danh sách `engine_owner=hook` |
| Không học phím | Không lưu nội dung; log chỉ có độ dài/action (S2) |

## 2. Kiến trúc tiến trình

```
textvn-tray.exe ──spawn──► textvn-hook.exe
                              ├── main: đọc config, kết nối pipe (IPC server = tray)
                              ├── Hook thread (COMSTA, CoInitialize):
                              │     SetWindowsHookExW(WH_KEYBOARD_LL, cb, self, 0)
                              │     GetMessage loop   ◄── hệ thống gửi WM_KEY* vào đây
                              ├── Focus thread: SetWinEventHook(EVENT_SYSTEM_FOREGROUND,
                              │                 EVENT_OBJECT_FOCUS) → cập nhật hwnd/app_id
                              └── UIA worker (thread riêng): IUIAutomation async → field_role,
                                    IsPassword → ime_set_context  (không bao giờ trong hook cb!)
```

- **`ime_instance` duy nhất**, mọi truy cập từ Hook thread (P0-2 §3); Focus/UIA worker gửi kết quả về
  qua queue lock-free (mpsc) → Hook thread áp `ime_set_context` trước key kế tiếp.
- Callback **không** COM/UIA/SendToFile/Log file — chỉ: check → engine → (nếu thay thế) queue lệnh inject
  → return. Việc inject chạy ngay trong callback (xem §5) nhưng vẫn timeboxed.

## 3. Hook callback — luật bất biến

```text
LRESULT cb(int code, WPARAM wp, LPARAM lp):
  if code < 0           → CallNextHookEx (bắt buộc)
  kbd = (KBDLLHOOKSTRUCT*)lp
  1. LLKHF_INJECTED     → CallNextHookEx   (chống loop — P0-3 §6; engine cũng nhận is_injected=1)
  2. our_inject_flag (thread-local đang trong SendInput) → CallNextHookEx
  3. key_up (lp MSB=0)  → CallNextHookEx   (engine chỉ xử lý key down; key up luôn đi qua)
  4.Foreground app không thuộc chế độ hook (mode=off / owner=tsf / secure) → CallNextHookEx
  5. Chord hệ thống (Ctrl+C/V/X/Z, Alt+Tab, Win+*, Ctrl+Shift+*, IME hotkey) → CallNextHookEx (B6)
  6. Stopwatch bắt đầu:
       ch = ToUnicodeEx(vk, scan, kbd_state, buf, layout=foreground thread!)
       r  = ime_key(inst, {vk, ch, mods, down:1, injected:0}, &res)
       res.flags & ERROR  → CallNextHookEx (fail-open)
       res.action == PASS → CallNextHookEx
       else               → apply §5  (return 1 — NUỐT phím gốc)
  7. elapsed > 2ms?     → log counter, KHÔNG retry; nếu 50 lần liên tiếp → self-disable + báo tray (CrashReport)
```

- **Timebox 2 ms**: engine đã <0.5ms (P0-3 §3.3); nếu UIA chưa kịp (chưa có field_role) → dùng role mặc định
  theo preset (`unknown/body`), **không** chờ.
- Hook phải trả lời trong ~`LowLevelHooksTimeout` (mặc định ~300ms); chậm hơn hệ thống sẽ **gỡ hook âm thầm**
  → watchdog của tray ping định kỳ 5s, mất 2 lần → restart.

## 4. Focus & field detect (không trong callback)

| Sự kiện | Nguồn | Việc làm |
|---|---|---|
| Đổi foreground window | `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` | Update `hwnd→app_id` cache (exe name qua `QueryFullProcessImageName`) |
| Đổi focus element | `EVENT_OBJECT_FOCUS` + fallback poll 500ms | Gửi yêu cầu UIA query cho worker |
| UIA query (worker) | `IUIAutomation::ElementFromHandle(hwnd)` → focused element → `CurrentControlType`, `CurrentClassName`, `CurrentIsPassword`, name | Map → `IME_FIELD_*` (bảng: `P1-3 §2`), `secure = IsPassword` |
| Kết quả | queue → hook thread | `ime_set_context` (cache theo hwnd; invalidate khi hwnd/process đổi) |

- UIA **async + cache**: cùng hwnd giữ role tối đa 2s hoặc tới khi focus event kế tiếp.
- UIA không sẵn (app kỳ lạ) → role `unknown` → theo §P0-3 resolve (BackspaceType/Preedit…) — **không block gõ**.

## 5. Injection modes (nơi P0-2 §4 trỏ tới)

Áp dụng theo `ime_result_v1` + strategy đã resolve. **Mọi inject đặt cờ `our_inject_flag` + dựa vào
`LLKHF_INJECTED` ở phía nhận.**

### 5.1 `BackspaceType`
```text
1) nếu delete_count > 0:  SendInput(n × {VK_BACK, down+up})
   (delay tối thiểu giữa các batch: dùng một mảng SendInput duy nhất, KHÔNG sleep)
2) SendInput(m × {wVK: ch, KEYEVENTF_UNICODE}?) — XEM 5.4
3) preedit hiển thị: hook không có preedit → gõ luôn text cuối (không hiển thị trung gian)
```

### 5.2 `SelectionReplace` (bug B1 — address bar/Excel)
```text
1) nếu delete_count > 0: SendInput(n × {VK_LEFT, KEYEVENTF_UP|KEYEVENTF_EXTENDEDKEY} với Shift held)
   → cụ thể: 1 event Shift down, n × Left (shift held), Shift up
2) chèn text (5.4) — app thay selection, KHÔNG gửi Backspace → autocomplete không kích hoạt lại
3) Nếu app không nhận selection (test fail ở appcomptest) → preset chuyển BackspaceType
```

### 5.3 `ForwardAsCommit`
Gõ text không xóa gì — chỉ 5.4 (terminal/dòng lệnh).

### 5.4 Cách gửi text: `inject_mode` của preset (P0-3 §2.1)

| Mode | Gửi | Dùng khi | Rủi ro |
|---|---|---|---|
| `unicode` | `KEYEVENTF_UNICODE` + `wScan=ch` | Chữ có dấu/UTF-16 surrogate | Một số app legacy không nhận WM_CHAR từ unicode-inject |
| `vk_then_unicode` | Trước: VK hợp lệ cho ASCII (`v`,`s`…) bằng `SendInput` với `KEYEVENTF_UNICODE` **cho phần ASCII**; sau: unicode cho phần không có VK | Chrome address bar, Excel (cần "key event thật" để autocomplete coi là người gõ) | Phức tạp hơn; phải tôn trọng `delete_count` (gửi Backspace VK) |
| `selection` | Shift+Left rồi `unicode` | preset chỉ định riêng | flicker nhẹ |

- **Thứ tự bắt buộc:** modifier state luôn restore (Shift/Ctrl up lại) sau mỗi lần inject — nếu không,
  lần gõ kế tiếp của người dùng bị lệch phím (bug kinh điển).
- **Batch limit:** mỗi `SendInput` ≤ 64 events; text > 64 ký tự → chia batch (hiếm, macro).
- Sau khi inject xong, **cờ loop guard bật trở lại ngay** (P0-3 §6): nếu engine vô tình xử lý chính text mình bơm
  → `is_injected` chặn.

### 5.5 Chọn mode: preset (`inject_mode`) > default `unicode`.

## 6. Tránh xử lý đôi (TSF ∩ Hook) — `engine_owner`

```text
rule: app có engine_owner = "tsf"  → hook: PASS-thru toàn bộ (dù app có dùng TSF hay không)
rule: app có engine_owner = "hook" → TSF dll (nếu app có load): ime_set_context(enabled=0) → mọi phím PASS
```
- Nguồn: appdb preset (`P1-3 §2`) + user override. Game/legacy → `owner=hook`.
- Cả hai adapter đọc **cùng một file state** (IPC broadcast `StateUpdate`, fallback: đọc `state.json` khi khởi động — P0-3 §4).
- Test: `corpus/win/owner_no_double.keys` (mô phỏng 2 adapter nhận cùng key — harness chạy 2 instance theo rule).

## 7. App elevated & uiAccess (UIPI)

| Tình huống | Hành vi |
|---|---|
| App medium-integrity (bình thường) | SendInput hoạt động |
| App elevated (admin) | UIPI chặn SendInput của process medium → **v2.0 dùng uiAccess**; v1.0: tray có tùy chọn "Chạy TextVN với quyền Admin" (như OpenKey cũ) + cảnh báo bảo mật |
| Manifest `uiAccess="true"` | Chỉ có hiệu lực khi: binary **đã ký** + nằm trong Program Files (secure path) → installer ở `P1-4 §4` bật cờ này khi cài system-wide |
| Secure desktop (UAC, Ctrl+Alt+Del) | Không gõ được — đúng kỳ vọng, document rõ |

Task: `WIN-055` (manifest + test trên VM admin).

## 8. Game & anti-cheat

- Game dùng **RawInput/DirectInput** đọc hardware → text bơm vào không hiện (giới hạn chung của mọi bộ gõ).
  Mục tiêu của ta: **không làm hỏng game** (không nuốt phím, không crash, không bị coi là cheat).
- Quy tắc: `hook.mode=auto` + preset `use_hook` cho game → chỉ inject vào cửa sổ chat/in-game text box
  (detect qua class name preset); ngoài ra PASS.
- Không inject vào process có chữ ký anti-cheat đã biết (danh sách `data/games_blocklist.txt` — preset data).

## 9. Chẩn đoán

- `TEXTVN_HOOK_DEBUG=1` → log `%LOCALAPPDATA%\TextVN\logs\hook.log` (action, ms, role — **không text**).
- Counters qua IPC `Ping/Pong` + file `hook-stats.json` (đổi mỗi 5s): callback p50/p99, injected count, self-disable count.
- `textvn doctor` hiển thị: hook running? mode? foreground owner? UIA ok?

## 10. Mapping task (chi tiết `P1-6-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| WIN-005 | Spike hook: LL hook + SendInput + UIA trên GHA runner | `docs/specs/hook-spike.md` (kèm kết quả GHA session — liên kết RW5) |
| WIN-040 | Process skeleton + IPC client + watchdog contract | spawn/kill/restart đúng, `doctor` thấy |
| WIN-041 | Callback theo §3 + loop guard | corpus `loop_guard`, `combo_pass` (bản hook-sim) |
| WIN-042 | Inject engine §5 (3 mode + modifier restore) | corpus `win/hook_*` ≥ 40 case |
| WIN-043 | Focus + UIA worker §4 | Manual + appcomptest: role đúng trên 8 app mẫu |
| WIN-044 | `engine_owner` rule §6 | corpus `owner_no_double` |
| WIN-045 | Game/legacy presets + blocklist | Danh sách 10 game/app đầu vào preset |

## 11. Failure modes

| Tình huống | Xử lý |
|---|---|
| Hệ thống gỡ hook (do chậm) | Watchdog phát hiện (§3.7) → self-disable + restart |
| UIA treo/crash | Worker thread timeout 1s → role mặc định, cache invalid; không ảnh hưởng hook cb |
| SendInput bị chặn (UIPI) | Phát hiện (GetLastError/access denied) → log + hiện trong doctor "cần run-as-admin/uiAccess" |
| Tray chết | Hook nhận pipe EOF → giữ config hiện tại, sau 30s không có Ping → self-exit (không sống mồ côi) |
| Config thay đổi giữa chừng | IPC `ConfigReload` → `ime_reload_config` ở ranh giới key kế tiếp |
