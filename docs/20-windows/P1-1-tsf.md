# P1-1 — TSF ADAPTER (Windows) — Solution chi tiết

> WS2 · crate `vietime-win-tsf` → `vietime-tsf.dll` (cdylib, static CRT).
> Chốt theo **ADR-005**: Rust + `windows` crate. Nếu spike `WIN-002` chặn → ADR-005b (C++/WRL glue),
> **giữ nguyên FFI & corpus** (chỉ đổi lớp COM).
> Hợp đồng với core: `../10-shared/P0-2-engine-ffi-contract.md` (đọc lại trước khi code).

## 1. Tổng quan & quyết định

| Quyết định | Nội dung | Vì sao |
|---|---|---|
| In-proc COM TIP | DLL nằm trong process app, engine link static | Gõ không qua IPC → latency <1ms path |
| Rust + `windows` crate | `Win32::UI::TextServices`, `Win32::System::Com` | Một toolchain với core/hook/updater |
| Composition làm mặc định | Dùng `ITfComposition` (strategy `Preedit`) | App chuẩn không thấy backspace |
| Fail-open tuyệt đối | Mọi panic → `catch_unwind` → PASS | S4: DLL chết ≠ app chết |
| 1 instance / thread | `ime_instance_new` trong `ActivateEx` | P0-2 §3 |

**GUID chính thức (freeze — không đổi sau khi release):**

```rust
// adapters/windows-tsf/src/guids.rs — tạo 1 lần, commit, không regenerate
pub const CLSID_VIETIME_TIP: GUID = GUID::from_u128(0x6F2B9C31_8E47_4D2A_9C84_1D5A3E70F9B8);
pub const PROFILE_VIETIME:  GUID = GUID::from_u128(0xC4A91F52_77B3_4E19_8A6D_2F8C0B6E5A13);
pub const DISPATTR_VIETIME: GUID = GUID::from_u128(0x9D2E7A44_1B5C_4F63_A086_33D15E4C7B21);
pub const LANGID_VI: u16 = 0x042A; // vi-VN
```

## 2. Cấu trúc file (bám đúng `P0-1 §1`)

```
adapters/windows-tsf/src/
├── lib.rs          # exports COM: DllGetClassObject/DllCanUnloadNow/DllRegisterServer(=đường dẫn register)
├── class.rs        # ClassFactory (IClassFactory) → tạo Tip
├── tip.rs          # Tip: ITfTextInputProcessorEx + ITfKeyEventSink + ITfThreadMgrEventSink
│                   #      + ITfTextEditSink + ITfFnConfigure (mở Settings từ language bar)
├── thread.rs       # ThreadState: instance engine, doc mgr, cookies, strategy cache — MỘT BẢN CHO MỘT THREAD
├── edit_session.rs # ReplaceEditSession: ITfEditSession (TF_ES_READWRITE) — nơi duy nhất sửa text
├── key_event.rs    # OnTestKeyDown/OnKeyDown → ime_key → quyết định eaten + kế hoạch sửa
├── composition.rs  # start/ensure/end composition, set preedit, commit
├── display_attr.rs # ITfDisplayAttributeInfo (gạch chân) + đăng ký GUID_PROP_ATTRIBUTE
├── register.rs     # logic đăng ký (gọi từ `vietime register`, xem §8)
└── ipc_client.rs   # pipe client: Hello/GetSnapshot/Subscribe/ConfigReload
```

**Ràng buộc code:**
- `#![deny(unsafe_op_in_unsafe_fn)]`; mọi `unsafe` block bắt buộc có comment `// SAFETY: ...` trỏ điều khoản FFI/P0-2.
- Toàn bộ entry COM bọc `catch_unwind` (helper `ffi_guard(|| ...)`) → trả `E_FAIL`/`FALSE` khi lỗi, **không** panic xuyên COM.
- Không `unwrap()`/`expect()` trong đường phím.

## 3. Vòng đời COM (lifecycle)

```
DLL nạp ─► DllGetClassObject(CLSID_VIETIME_TIP) ─► ClassFactory ─► Tip (per thread làm việc)
Tip::ActivateEx(clientId, dwFlags)
  1. CoCreateInstance(CLSID_TF_ThreadMgr) → ITfThreadMgr  (KHÔNG giữ global; theo thread)
  2. QI → ITfKeystrokeMgr → AdviseKeyEventSink(clientId, tip, TRUE)   [verify: WIN-002]
  3. QI → ITfSource → AdviseSink(IID_ITfThreadMgrEventSink, …)         (OnSetFocus)
  4. CreateDocumentMgr + Push (context tạo on-demand, xem §6)
  5. ime_instance_new(config đọc từ %APPDATA%\VietIME\config.json)
  6. ipc_client::connect(\\.\pipe\vietime-ipc-v1) + Subscribe   (thất bại → chạy offline, không block)
Tip::Deactivate()
  1. UnadviseKeyEventSink / UnadviseSink (bắt buộc — rò rỉ cookie = crash lần activate sau)
  2. end composition đang dở (COMMIT text, bug B2)
  3. ime_instance_free + đóng pipe
DllCanUnloadNow: TRUE khi không còn Tip sống
```

**Focus change:** `ITfThreadMgrEventSink::OnSetFocus(pdimNew)`:
- `pdimNew == NULL` → **commit ngay** composition đang dở (B2: commit-before-hide) + `ime_reset()`.
- Document mới → `ime_set_context` với `app_id`/`field_role` lấy từ cache field-detect (P1-3).

## 4. State machine của ThreadState

```
        phím sinh text (engine REPLACE/COMMIT)
 Idle ─────────────────────────────────► Composing(pretedit)
  ▲                                          │
  │ Space/Enter/động từ (WORD_END)          │ phím khác với preedit (→ Backspace/...)
  └──────────────── COMMIT ◄─────────────────┘
  phím hệ thống/Ctrl+combo → không đổi state (PASS)         context lost/focus → COMMIT + Idle
```

- Buffer ngữ nghĩa: engine tự quản (P0-2); ThreadState chỉ track `preedit hiện tại` + `composition alive?`.
- Nếu app tự xoá preedit (rare) → `ITfTextEditSink::OnEndEdit` phát hiện lệch → **self-heal**: `ime_reset()` + `COMMIT` state về Idle (không đoán bừa text của app).

## 5. Key flow

```
ITfKeyEventSink::OnTestKeyDown(ctx, w, l, eaten)  → eaten = sẽ ăn? (gọi ime_key với key_down=1 ở OnKeyDown)
ITfKeyEventSink::OnKeyDown(ctx, w, l, eaten)
  1. System/chord check TRƯỚC: Ctrl/Alt/Win+key, IME hotkey → *eaten=FALSE, PASS (B6)
  2. ctx từ ThreadState: secure/enabled/app_id/field_role/caps (P1-3 cung cấp, cache)
  3. ime_key(inst, {vk: w, ch: ToUnicodeEx(w,l,kbd_state), mods, key_down=1}, &r)
  4. match r.action:
       PASS      → *eaten = FALSE
       REPLACE   → *eaten = TRUE; strategy = resolve(ctx, r):
                     Preedit           → ReplaceEditSession: ensure_composition(r.preedit)
                     SelectionReplace  → ReplaceEditSession: §6.2 (chọn vùng rồi SetText)
                     BackspaceType     → ReplaceEditSession: §6.1
                     ForwardAsCommit   → ReplaceEditSession: §6.3 (gõ thẳng, không composition)
                     Passthrough       → không thể xảy ra ở đây (r đã PASS); defensive PASS
       COMMIT    → *eaten = TRUE; ReplaceEditSession: end composition + SetText(insert)
       RESTORE   → *eaten = TRUE; ReplaceEditSession: §6.1 với delete_count/insert
  5. lParam có bit 30 (previous state) khác → is_repeat; key_up không qua engine (chỉ quan tâm down)
```

> **Quy tắc:** mọi sửa text **chỉ** trong `ITfEditSession::DoEditSession` (TSF không cho sửa ngoài edit session).
> `RequestEditSession(TF_ES_READWRITE | TF_ES_SYNC)`; nếu bị từ chối (EC_READONLY) →
> gọi `ime_reset()` **trước key kế tiếp** rồi fail-open: `*eaten=FALSE`. Engine đã tính
> outcome trước khi TSF từ chối; không reset sẽ làm key sau replace text chưa từng được app nhận.

## 6. `apply_replace` — 3 cách sửa text (mục P0-2 trỏ vào đây)

### 6.1 `BackspaceType` (dùng `ITfRange`)

```text
DoEditSession(ec):
  selection = ctx->GetStart(ec, TF_AE_SELECTION)  // caret
  if delete_count > 0:
      start = clone(selection); start->Collapse(ec, TF_ANCHOR_START)
      start->ShiftStart(ec, selection)            // mở rộng đúng delete_count ký tự (xem §6.4)
  else: start = selection
  start->SetText(ec, NULL, insert_len, insert_utf16)   // thay tại vùng [start, caret)
  selection->Collapse(ec, TF_ANCHOR_END) ; ctx->SetSelection(ec, 1, &sel)
```

### 6.2 `SelectionReplace` (address bar / Excel — bug B1)

```text
  1. Chỉ dùng range adapter đang sở hữu từ composition/insert engine; selection của user khác range đó → PASS + reset,
     không fallback sau khi đã xóa.
  2. Nếu selection rỗng → chỉ mở rộng về TRÁI trong suffix owned, đúng delete_count ký tự.
  3. SetText(ec, NULL, insert_len, text)  → TSF thay đúng range owned.
  4. KHÔNG gửi phím Backspace (app autocomplete không bị kích hoạt lại — đây là lý do tồn tại của strategy).
```

### 6.3 `ForwardAsCommit` (terminal — bug B8)

```text
  Không StartComposition. Chỉ: caret→SetText(insert) → caret sau text.
  Lý do: terminal không render preedit; composition bị loại → không còn "chữ ảo" treo.
```

### 6.4 Quy tắc chung

- `delete_count` luôn ≤ ký tự engine đã gõ trong context này (đội với counter trong ThreadState; nếu app
  tự sửa text (`OnEndEdit` lệch) → reset counter + `ime_reset`, **không** xoá text app.
- Mọi `SetText` truyền UTF-16 (convert từ UTF-32 của `ime_result_v1.insert` — helper `utf32_to_wide`).
- Test: `corpus/win/*` + appcomptest `notepad_replace`, `chrome_address_bar`.

## 7. Hiển thị preedit & display attribute

1. `display_attr.rs` implement `ITfDisplayAttributeInfo` cho `DISPATTR_VIETIME` (gạch chân, màu theo theme).
2. Đăng ký provider: `ITfCategoryMgr::RegisterCategory(CLSID, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER, CLSID)` (§8).
3. Khi preedit đổi: set property `GUID_PROP_ATTRIBUTE` của range composition = `VT_CLSID` chứa `DISPATTR_VIETIME`.
4. Text preedit = `SetText(..., r.preedit)` (P0-2 §4: preedit là chuỗi **hoàn chỉnh**).

## 8. Đăng ký TIP (gọi từ `vietime register`, xem P1-4 §4 installer)

```text
vietime register [--scope user|machine]
 1. [scope] CoCreate ITfInputProcessorProfiles → Register(CLSID_VIETIME_TIP)
 2. AddLanguageProfile(CLSID, LANGID_VI, PROFILE_VIETIME, L"VietIME", icon=<install>\vietime.ico, idx 0)
 3. ITfCategoryMgr::RegisterCategory(CLSID, GUID_TFCAT_TIP_KEYBOARD, CLSID_VIETIME_TIP)
 4. ITfCategoryMgr::RegisterCategory(CLSID, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER, CLSID_VIETIME_TIP)
 5. [scope=user] đăng ký COM server: HKCU\Software\Classes\CLSID\{CLSID}\InprocServer32 = <path> (REG_SZ, ThreadingModel=Apartment)
vietime unregister  → đảo ngược 5 bước; KHÔNG đụng key của app khác
```

- **Spike WIN-003 phải xác nhận**: bước 1–4 chạy được từ user context không cần admin
  (kết quả ghi `docs/specs/tsf-registration-spike.md`; nếu API ghi HKLM → fallback ghi registry TIP per-user
  theo layout `HKCU\Software\Microsoft\CTF\TIP\{CLSID}\...` — chỉ dùng khi spike chứng minh cần).
- Sau đăng ký: `ITfInputProcessorProfileMgr::ActivateProfile(...)` để kích hoạt ngay, **không** buộc đổi
  default của user; hướng dẫn chọn bằng Win+Space.

## 9. Spike checklist (task `WIN-002` — tuần 1, PHẢI làm trước mọi task khác)

Mỗi dòng: API → đã dùng được? (✅/❌) → ghi vào `docs/specs/tsf-spike.md` kèm code snippet 10 dòng.

| # | API / câu hỏi | Kỳ vọng |
|---|---|---|
| 1 | `CoCreateInstance(CLSID_TF_ThreadMgr)` trong DLL của ta | ✅ |
| 2 | **QI `ITfKeystrokeMgr` từ thread mgr** (hoặc đường đúng khác) → `AdviseKeyEventSink` nhận `OnKeyDown` trong Notepad | ✅ *(tên đường lấy đúng từ SDK header — spike quyết định)* |
| 3 | `CreateDocumentMgr/CreateContext/Push` + `GetStart/GetSelection` trong edit session | ✅ |
| 4 | `ITfContextComposition::StartComposition` → `ITfRange::SetText` → `EndComposition` hiện chữ trong Notepad | ✅ |
| 5 | `OnTestKeyDown` vs `OnKeyDown`: app nào ăn `TRUE` ở test (Word/Chrome) | ghi bảng |
| 6 | `vietime register` scope=user có hoạt động ở VM sạch không admin? | ✅ hoặc ghi fallback |
| 7 | Win+Space hiển thị "VietIME"; `ActivateProfile` chuyển được | ✅ |
| 8 | Chrome address bar: composition có bị autocomplete can thiệp không? (chính là B1) | ghi kết quả → quyết định preset |
| 9 | Password field: TSF có được focus không? UIA `IsPassword` đọc được từ process khác? | ghi kết quả → P1-3 |
| 10 | Hook (`P1-2`) với app elevated: SendInput bị UIPI chặn như dự kiến? | ghi → uiAccess plan (P1-4) |

**Exit của spike:** 10/10 có kết quả; nếu #2/#4 fail → kích hoạt ADR-005b (C++/WRL glue) ngay trong tuần 1.

## 10. Playbook triển khai (mapping sang task — chi tiết ở `P1-6-TASKS.md`)

| Bước | Task | Nội dung | Acceptance tóm tắt |
|---|---|---|---|
| 1 | WIN-002/003/004/005 | Spike (TSF, register, UIA, hook) | `docs/specs/*-spike.md` 10/10 kết quả |
| 2 | WIN-010 | DllGetClassObject + ClassFactory + ActivateEx rỗng | Build, đăng ký, Notepad không crash |
| 3 | WIN-011 | Key sink + `ime_key` PASS toàn bộ | Gõ tiếng Anh không đổi hành vi (corpus `combo_pass`) |
| 4 | WIN-012 | ReplaceEditSession §6.1 | `duocj` → `được` trong Notepad |
| 5 | WIN-013 | Composition + preedit + display attr | Gạch chân đúng, commit khi Space/Enter |
| 6 | WIN-014 | Focus/commit-before-hide + `ime_reset` | corpus `bug_B2_chat_enter` pass |
| 7 | WIN-015 | Hotkey preserve + toggle EN/VN (`Ctrl+Shift+Space`) | `combo_pass` + toggle hoạt động mọi app |
| 8 | WIN-016 | ipc_client (Hello/GetSnapshot/Subscribe/ConfigReload) | Đổi config trong tray → gõ đổi ngay |
| 9 | WIN-017 | secure field → `secure=1` (theo P1-3 cung cấp) | corpus `secure_field_passthrough` pass |
| 10 | WIN-018 | RESTORE + auto-restore EN (B5) | corpus `restore_en_*` pass |
| 11 | WIN-019 | Self-heal `OnEndEdit` lệch | Manual: click giữa preedit không crash/đúng |

## 11. Chế độ test & chẩn đoán

- `VIETIME_TSF_DEBUG=1` (env) → log vào `%LOCALAPPDATA%\VietIME\logs\tsf-<pid>.log`
  (mức info: state transition, action, delete/insert **độ dài**, **không nội dung** — S2).
- `vietime doctor` liệt kê: TIP đã đăng ký? profile active? pipe connected? caps?
- Manual smoke script: `tools/win/smoke-tsf.ps1` (mở Notepad, gửi chuỗi qua SendInput, so clipboard —
  chi tiết ở `P1-5 §4.1`).

## 12. Failure modes & xử lý

| Tình huống | Xử lý |
|---|---|
| `ime_key` trả `IME_FLAG_ERROR` | Treat PASS; tăng counter → tray hiện cảnh báo sau 3 lần/phút |
| `RequestEditSession` bị từ chối | PASS (fail-open), log warn |
| Config load lỗi | Theo P0-2 §5 (non-fatal, default) |
| Pipe chết/tray không chạy | Chạy offline với config đã đọc lúc Activate; không block |
| DLL bị unload khi còn Tip sống | `DllCanUnloadNow` trả FALSE khi ref>0; Deactivate giải phóng hết |
| Panic trong bất kỳ đâu | `catch_unwind` → PASS + `CrashReport` IPC (mỗi 60s tối đa 1 msg) |
