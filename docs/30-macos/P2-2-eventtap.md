# P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết

> WS2 · `adapters/macos-tap/` (Swift package — `PLAN §3.2` layout). **ADR-006: opt-in, không bật mặc định.**
> Chỉ dùng cho app **không đi qua IMK** (một số game/legacy). IMK vẫn primary (`P2-1`).
> Đối chiếu format với `../20-windows/P1-2-hook.md` (cùng số mục: §3 callback, §5 injection, §6 loop).

## 1. Quyết định & phạm vi

| Quyết định | Nội dung |
|---|---|
| Opt-in per-app | Chỉ chạy tap cho app có `engine_owner: "tap"` trong preset/user override (P2-3 §3) |
| Quyền riêng | `AXIsProcessTrustedWithOptions(prompt:)` + hướng dẫn System Settings → Privacy & Security → Accessibility; từ chối = tắt tính năng, IMK vẫn chạy |
| Process | Tap chạy **trong `VietIME-IM.app` nhưng thread riêng** với **instance engine riêng** (P0-2 §3: 1 instance = 1 thread) |
| Không xung với IMK | App `owner=tap` → IMK `handle()` trả PASS (`enabled=0` cho context đó) — mirror rule `P1-2 §6` |

## 2. Kiến trúc

```
VietIME-IM.app
├── main thread       : IMKServer + ime_instance A  (P2-1)
└── Tap thread        : CFRunLoop + CGEvent.tapCreate + ime_instance B
      ├── pre-build tap list theo 3 bước (§4.0)
      ├── callback: check → ime_key(B) → §5 inject (nếu cần) → return event/nil
      └── AX cache worker (dùng chung FieldDetect với IMK qua queue — P2-3)
```

- Thread tap có QoS `.userInteractive` nhưng vẫn timebox 2ms (§3).
- Tap chết → tự tạo lại theo §4.0 (không restart process).

## 3. Callback — luật bất biến (mirror `P1-2 §3`)

```text
func tapCallback(proxy, type, event, refcon) -> Unmanaged<CGEvent>?:
  0. type == .tapDisabledByTimeout / .tapDisabledByUserInput → tapEnable lại (§9) → return event
  1. CGEventSourceGetUserData(event) == kVietIMEInjectedMarker → return event   (chống loop §6)
  2. type != .keyDown/.keyUp                                 → return event
  3. keyUp → return event (engine chỉ xử lý down; luôn cho qua up)
  4. Chord Cmd+…, hệ thống (Cmd+Tab, Cmd+Space, brightness…)  → return event (B6)
  5. Foreground app không thuộc owner=tap / secure           → return event
  6. Stopwatch:
       ch = layout translate (CGEventSourceKeyboardType + UCKeyTranslate)
       r  = ime_key(B, {vk, ch, mods, down:1, injected:0}, &res)
       res.flags & ERROR  → return event (fail-open)
       PASS               → return event
       else               → apply §5; return nil (NUỐT phím gốc)
  7. elapsed > 2ms → log counter; >50 lần liên tiếp → self-disable module + báo status item
```

- **Không** AX query/file I/O/log disk trong callback (worker ở thread khác, kết quả qua queue — P2-3 §2).

## 4. Chọn loại tap & permission (feature-detect 3 bước — `PLAN §5.1`)

```text
0. khởi động (chỉ khi có ≥1 app owner=tap trong state):
   thử lần lượt tapCreate(tap:):
     a) kCGHIDEventTap        → (ưu tiên: thấy cả key trước khi app xử lý)
     b) kCGSessionEventTap
     c) kCGAnnotatedSessionEventTap
   loại nào create thành công → dùng, ghi loại vào doctor (không hardcode 1 loại — đúng gonhanh)
1. nếu cả 3 fail (thiếu quyền) → AXIsProcessTrustedWithOptions(prompt: true)
2. vẫn fail → tắt module, status item hiện "Cần cấp quyền Accessibility", giữ IMK nguyên vẹn
```

- **Không dùng private API** (PLAN §5.1 — sẽ bị TCC/Review đứt).
- Quyền thay đổi khi đang chạy → observer `AXIsProcessTrusted` poll 5s (không register private notification).

## 5. Injection modes (đối chiếu `P1-2 §5`)

Mọi event post kèm `CGEventSourceSetUserData(evt, kVietIMEInjectedMarker)` + đặt `is_injected=1` cho engine.

### 5.1 `BackspaceType`
```text
1) nếu delete_count > 0: n × CGEvent(keyboard: keycode 51 kVK_BackSpace, .maskNonCoalesced) → post kCGHIDEventTap
2) chèn text: (a) hoặc (c) bên dưới
```
### 5.2 `SelectionReplace` (bug B1 trên app không IMK)
```text
1) Shift down → n × Left(123) → Shift up   (mở rộng selection sang trái)
2) chèn text (b)/(a) — KHÔNG gửi Backspace
```
### 5.3 `ForwardAsCommit` — chỉ chèn text, không xóa (terminal qua tap — hiếm).

### 5.4 Cách chèn text (`inject_mode` — cùng enum với `P0-3 §2.1`)
| Mode | Gửi | Dùng khi |
|---|---|---|
| `unicode` | `CGEventKeyboardSetUnicodeString(evt, text)` + post | mặc định (chữ có dấu/surrogate) |
| `vk_then_unicode` | keycode thật cho ASCII (a–z, 0–9) + unicode cho phần có dấu | app cần "key event thật" (autocomplete) |
| `selection` | Shift+Left trước khi chèn (§5.2) | preset chỉ định |

- **Batch ≤ 64 event/lần post**; modifier luôn restore (flag .maskShift/.maskCommand… clear sau mỗi lần).
- Sau inject: cờ loop guard bật lại ngay (mirror `P0-3 §6`).

## 6. Tránh xử lý đôi (mirror `P1-2 §6`) — `engine_owner`

```text
owner = "imk" (mặc định) → tap: không cài callback cho app này (foreground check ở §3.5)
owner = "tap"            → IMK handle(): ctx.enabled = 0 → mọi phím PASS (P2-1 §5 bước 6)
```
- Đồng bộ qua IPC `StateUpdate` (P0-3 §4) — cả 2 đọc cùng `state.json` khi offline.
- Test: `corpus/mac/owner_no_double.keys`.

## 7. Bảo mật & minh bạch (RM8)

- Tap **không** bật ở lần cài; chỉ sau khi user chọn app trong Settings → "Gõ tiếng Việt cho app này (cần quyền)".
- Settings hiện rõ: quyền đang có/không, list app đang dùng tap, nút "Tắt toàn bộ".
- Không network, không ghi text (S2), log chỉ action/độ dài.
- Document: `docs/security/tap-permission.md` mô tả chính xác tap đọc được gì (key event trong session) — audit sẵn sàng.

## 8. Test

| Hạng mục | Test |
|---|---|
| Chống loop | `corpus/mac/loop_guard_tap.keys` + manual: gõ 1000 key không nhân đôi |
| Callback timebox | benchmark: p99 < 2ms (counter trong doctor) |
| Chọn tap type | Unit (giả lập `tapCreate` fail a→b→c) |
| owner=tap vs imk | `corpus/mac/owner_no_double.keys` |
| Permission mất giữa chừng | Manual: tắt quyền khi đang chạy → tap dừng, IMK vẫn gõ |

## 9. Failure modes

| Tình huống | Xử lý |
|---|---|
| `.tapDisabledByTimeout` (callback chậm) | `tapEnable` lại ngay; nếu lặp 3 lần/phút → self-disable + báo (mirror `P1-2 §11`) |
| System thay quyền (TCC revoke) | Poll 5s phát hiện → stop tap, giữ IMK |
| App anti-cheat/coi tap là lạ | Không inject vào blocklist (`data/games_blocklist.txt` — dùng chung với Win) |
| Tap thread panic | `catch_unwind` →disable module; IMK unaffected (khác process? không — cùng process: panic bị chặn, thread tap dừng, main thread sống) |

## 10. Task (chi tiết `P2-6-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| MAC-009 | Spike tap: 3 loại tapCreate + permission flow + marker loop | `docs/specs/macos-tap-spike.md` |
| MAC-040 | Tap thread skeleton + foreground/owner check | Doctor thấy tap thread; owner=imk → 0 inject |
| MAC-041 | Callback theo §3 + self-disable | Manual 1000 key; counter p99 < 2ms |
| MAC-042 | Inject §5 (3 mode + marker + restore) | corpus `mac/tap_*` ≥ 20 case |
| MAC-043 | Permission UX §4 (prompt, revocation poll) | Manual 2 tình huống grant/revoke |
| MAC-044 | owner rule §6 + IPC sync | corpus `owner_no_double` pass |
