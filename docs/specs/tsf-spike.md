# Spike WIN-002 — TSF-in-Rust (`spikes/tsf-min`)

> Task: `WIN-002` (`P1-6-TASKS.md`) · Checklist gốc: `../20-windows/P1-1-tsf.md §9`
> · Spike code: `../../spikes/tsf-min/` · Kết quả 10/10 (mục **Exit** của `P1-1 §9`)
> Ngày chạy: 2026-09-27 · Máy: Windows 11 x64, `windows` crate 0.61.3

**Tóm tắt:** #1–#8 ✅ dùng được → **ADR-005b (C++/WRL fallback) KHÔNG kích hoạt**
(#2/#4 pass như điều kiện kích hoạt). #9 có kết quả rõ → ghi `P1-3`. #10 xem dòng cuối bảng.

Log bằng chứng: `%LOCALAPPDATA%\VietIME\logs\tsf-min.log` (chỉ ghi vk + HRESULT — **không
ghi nội dung phím**, S2 Handbook).

## Bảng kết quả 10 mục

| # | API / câu hỏi | Kết quả | Evidence |
|---|---|---|---|
| 1 | `CoCreateInstance(CLSID_TF_ThreadMgr)` trong DLL của ta | ✅ | `CHK#1 …=OK` |
| 2 | QI `ITfKeystrokeMgr` → `AdviseKeyEventSink` → `OnKeyDown` | ✅ | `CHK#2 AdviseKeyEventSink(tid=…)=OK`; OnKeyDown nhận key trong Notepad/Word/Chrome/VS Code |
| 3 | `CreateDocumentMgr/CreateContext/Push` + `GetStart/GetSelection` | ✅ + 1 finding | `CHK#3 GetStart=OK`, `GetSelection=OK fetched=1` |
| 4 | `StartComposition` → `SetText` → `EndComposition` | ✅ | Notepad → `đượcđượđượca` (spike gõ cứng, không có engine); Chrome omnibox → `được` |
| 5 | `OnTestKeyDown` vs `OnKeyDown` theo app | ✅ bảng ở dưới | log pha TEST/KEYDOWN |
| 6 | `register` scope=user ở máy sạch không admin | ✅ + fallback đã chứng minh | `tsf-register.log`: CLSID HKCU OK; API ghi HKLM cần elevation; đường không-elevation = InstallLayoutOrTip |
| 7 | Win+Space thấy "VietIME"; `ActivateProfile` chuyển được | ✅ (readback registry) | registry `0x0409`+`0x042A` `Enable=1`, readback `VietIME` sạch; verify trực quan Win+Space còn optional |
| 8 | Chrome address bar: autocomplete có đè composition không? (B1) | ✅ — KHÔNG đè | composition sống sót trong omnibox, popup (`list=1`) không đè khi insert; **Enter-commit còn mở** → chốt cho WIN-008 |
| 9 | Password field: TSF có focus? UIA `IsPassword` cross-process? | ✅ có kết quả → P1-3 | xem §#9 bên dưới |
| 10 | SendInput đến app elevated có bị UIPI chặn? | xem §#10 bên dưới | xem §#10 bên dưới |

**Exit:** 10/10 có kết quả → spike WIN-002 đóng.

## Chi tiết từng mục

### #1 — `CoCreateInstance` trong DLL

```rust
// spikes/tsf-min/src/lib.rs — ActivateEx (STA) gọi khi TIP load
match unsafe { CoCreateInstance::<_, ITfThreadMgr>(&CLSID_TF_ThreadMgr, None,
                                                  CLSCTX_INPROC_SERVER) } {
    Ok(_)   => log("CHK#1 CoCreateInstance(CLSID_TF_ThreadMgr)=OK"),
    Err(e)  => log(&format!("CHK#1 CoCreateInstance=FAIL hr={:#010x}", e.code().0)),
}
```

Không cần CoInitialize thêm — `DllGetClassObject`/`ActivateEx` đã chạy trong STA của TSF.

### #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`

```rust
// QI từ thread mgr (đúng đường theo SDK header — spike đã quyết định)
let keymgr: ITfKeystrokeMgr = thread_mgr.cast()?;
let sink: ITfKeyEventSink = KeySink { _guard: ObjGuard::new(), tid }.into();
unsafe { keymgr.AdviseKeyEventSink(tid, &sink, true) }?;
log(&format!("CHK#2 AdviseKeyEventSink(tid={tid})=OK"));
// Nhận OnKeyDown thật trong: Notepad, Word, Chrome, VS Code.
// Unadvise khi OnSetFocus(fg=0) → log "Deactivate UnadviseKeyEventSink=OK".
```

`true` = `fForeground` (chỉ nhận khi window có focus) — đúng hành vi cần giữ khi implement.

### #3 — `GetStart` / `GetSelection` (+ finding)

```rust
let start = unsafe { self.ctx.GetStart(ec) }?;              // CHK#3 GetStart=OK
let mut sel = [TF_SELECTION::default()];
let mut fetched: u32 = 0;
unsafe { self.ctx.GetSelection(ec, TS_DEFAULT_SELECTION, &mut sel, &mut fetched) }?;
// CHK#3 GetSelection=OK fetched=1
// TF_SELECTION.range là ManuallyDrop → phải drop tay, không leak:
let owned = std::mem::replace(&mut sel[0].range, std::mem::ManuallyDrop::new(None));
drop(std::mem::ManuallyDrop::into_inner(owned));
```

> **S3-1 (ghi nhận cho `WIN-010`):** `GetStart(ec)` trả về **đầu document**,
> *không* phải vị trí caret. Anchor của composition/insert **phải** lấy từ
> `GetSelection(TF_DEFAULT_SELECTION)`, không được dùng `GetStart`.

### #4 — `StartComposition` → `SetText` → `EndComposition`

```rust
let ctxcomp: ITfContextComposition = self.ctx.cast()?;
let comp = unsafe { ctxcomp.StartComposition(ec, &start, &csink) }?;   // CHK#4 StartComposition=OK
// Pad số 0 phía sau slice: TSF đọc bằng wcslen() → slice không terminator = đọc rác
let raw: Vec<u16> = "được".encode_utf16().collect();
let tbuf: Vec<u16> = raw.iter().copied().chain(std::iter::once(0)).collect();
unsafe { comp.GetRange()?.SetText(ec, 0, &tbuf[..raw.len()]) }?;       // CHK#4 SetText=OK
unsafe { comp.EndComposition(ec) }?;                                   // CHK#4 EndComposition=OK
```

Kết quả thật: **Notepad** hiện `đượcđượđượca` (mỗi lần gõ 'D' spike thay 4 ký tự — hành vi
cố ý của spike, không có engine); **Chrome omnibox** hiện `được`.

> **S3-2 (đã fix trong spike, bắt buộc áp cho impl):** `AddLanguageProfile`/`SetText`
> đều đọc chuỗi bằng `wcslen()` → truyền slice đúng length nhưng allocation **phải có số 0
> phía sau**; truyền `&[]` (ptr null-ish 0x2) → **AV**. Xem `tsf-registration-spike.md`.

### #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`

| App | `OnTestKeyDown` | `OnKeyDown` | Ý nghĩa |
|---|---|---|---|
| Word | ✅ có (VK_A → `FALSE`, VK_D → `TRUE` → mới gọi `OnKeyDown`) | có | Word tôn trọng phase TEST — `TRUE` ở test = "tôi ăn phím này" |
| Notepad (Win11) | ❌ không gọi | ✅ có | bỏ qua phase TEST |
| Chrome (omnibox) | ❌ không gọi | ✅ có | bỏ qua phase TEST |
| VS Code | ❌ không gọi | ✅ có | bỏ qua phase TEST |
| Win32 `Edit` (legacy) | ✅ có | ✅ có | giống Word |
| WinForms TextBox | ✅ có | ✅ có | giống Word |

**Kết luận cho impl:** không được giả định `OnTestKeyDown` tồn tại — logic ăn phím phải
đúng ở **cả hai** pha, và `OnTestKeyDown` trả `TRUE` thì `OnKeyDown` **phải** xử lý thật
(không được "test true rồi eat rỗng" — app sẽ không gửi lại key).

### #6 — Đăng ký scope=user không admin

| API | Ghi vào | Không-elevated? | Bằng chứng |
|---|---|---|---|
| COM server `HKCU\Software\Classes\CLSID\{…}\InprocServer32` | HKCU | ✅ OK | write/readback OK |
| `ITfInputProcessorProfiles::Register` | **HKLM** (CTF) | ❌ cần elevation | elevate mới OK |
| `AddLanguageProfile` | **HKLM** | ❌ cần elevation | elevate mới OK; readback `VietIME` (utf16 sạch) |
| `EnableLanguageProfileByDefault` | **HKLM** | ❌ → `E_FAIL 0x80004005` | lỗi **vô hại** ở user context |
| `ITfCategoryMgr::RegisterCategory` | **HKLM** | ❌ cần elevation | elevate mới OK |
| **`InstallLayoutOrTip`** (input.dll) | **HKCU user input list** | ✅ **OK, KHÔNG cần elevation** | `InstallLayoutOrTip(0x0409:{CLSID}{ProfileGUID}, DEFPROFILE) → OK` |

**Fallback đã chứng minh (dùng cho `vietime register --scope user`):** chạy elevated **một
lần** cho 4 API ghi HKLM, rồi mọi thứ về sau không-elevated qua `InstallLayoutOrTip`
+ key `HKCU\Software\Microsoft\CTF\...` (Assemblies). Toàn bộ output ghi
`%LOCALAPPDATA%\VietIME\logs\tsf-register.log` qua `say()` — output của tiến trình
elevated bị ẩn, không ghi log sẽ không đọc lại được.

### #7 — Win+Space + `ActivateProfile`

```rust
// Thứ tự BẮT BUỘC (không đảo — cập nhật input list có thể reset enabled flag):
install_layout_or_tip(LANGID_VI);   // 1. input.dll → KHÔNG cần elevation
install_layout_or_tip(LANGID_EN);
prof.EnableLanguageProfileByDefault(&CLSID_VIETIME_TIP, lang, &PROFILE_GUID, true)?; // 2.
prof.ActivateLanguageProfile(&CLSID_VIETIME_TIP, LANGID_EN, &PROFILE_GUID)?;          // 3.
// Activate dưới locale đang chạy (en-US/0x0409) — locale khác = không load (Keyman wiki).
```

Evidence: registry cả `0x0409` và `0x042A` có `Enable=1`; readback mô tả = `VietIME`.
Verify trực quan Win+Space (thấy tên trong flyout) là **optional còn lại** — không chặn exit.

### #8 — Chrome address bar vs autocomplete (bug B1)

- Composition **sống sót** trong omnibox; popup autocomplete (`list=1`) **không** đè lên
  text khi insert bằng `SetText` trong edit session.
- Nhấn **Enter** khi composition đang mở → app nhận commit (không bị autocomplete nuốt).
- **Quyết định:** giữ nguyên chiến lược `SelectionReplace` (không gửi Backspace) +
  chốt `ForwardAsCommit` cho terminal ở WIN-008; preset Chrome không cần đặc cách thêm.

### #9 — Password field → kết quả cho `P1-3`

| Target | TIP activate? | Composition gõ được? | UIA đọc được cross-process? |
|---|---|---|---|
| Chrome `<input type=password>` | ✅ **CÓ** | ✅ CÓ (`OnKeyDown vk=0x44`, Start/Set/End OK; text đối chiếu = `được`) | value bị che (đúng kỳ vọng) |
| Native Win32 `ES_PASSWORD` (GWL_STYLE = 0x20 xác nhận) | ✅ CÓ | ✅ CÓ (OnTestKeyDown + OnKeyDown + composition OK) | ❌ `IsPassword` = **False** dù style có bit `ES_PASSWORD` |
| WinForms `TextBox` (PasswordChar) | ❌ không activate (host PowerShell/WinForms không thấy log) | — | ❌ `IsPassword` = False |
| CredUI "Windows Security" (`CredentialUIBroker.exe`) | ❌ **KHÔNG activate** | — | ❌ **UIA subtree = 1** — không expose descendants |

**Kết quả → `P1-3` (field detect):**
1. **Không dựa** vào `IsPassword` của UIA cho legacy provider (nó sai cả hướng an toàn lẫn
   nguy hiểm) → cần lớp detect theo style/`EM_GETPASSWORDCHAR`/class như `field-detect`.
2. **CredUI là secure field path thật**: TIP không activate + UIA không expose → *mặc định
   an toàn* (fail-open của ta = gõ thẳng). WIN-017 chỉ cần confirm, không cần workaround.
3. Chrome password **không** tự tắt TIP — nếu muốn tắt phải tự detect (P1-3 quyết định).

### #10 — SendInput đến app elevated (UIPI)

**Phương pháp** (không dùng Notepad MSIX — xem common-errors C6): app test tự build
`tfield2.exe` (WinForms, `TextChanged` **ghi ra file** `%TEMP%\tfield_text.log`) → không
cần đọc chéo process. Cùng binary chạy 2 lần: 1 lần `runas` (elevated), 1 lần medium.

| | Medium control | Elevated (cùng app, cùng SendInput) |
|---|---|---|
| `GetForegroundWindow` trước send | ✅ đúng pid target | ✅ đúng pid target (user click tay) |
| `SendInput('d')` return | `2/2` | `2/2` — **cũng báo thành công** |
| App thực nhận phím? | ✅ `text='d' codes=U+0064` (file log) | ❌ **không ghi file** → 0 phím đến nơi |
| `SendMessage(WM_GETTEXT)` cross-process | đọc được text | ❌ `len=0 text=''` (bị chặn) |
| `Focus()` = `AttachThreadInput` + `SetForegroundWindow` từ medium | ✅ `True` | ❌ **`False`** — attach input queue bị UIPI chặn |

**Kết luận:**
1. **SendInput bị UIPI chặn đúng như dự kiến** (`P1-2` / uiAccess plan `P1-4`): với target
   elevated, `SendInput` **vẫn trả 2/2** nhưng phím **không được giao** — không được dùng
   return value làm bằng chứng "đã gõ", phải verify app thật sự nhận (đọc text target).
2. `AttachThreadInput`/`SetForegroundWindow` cross-integrity cũng fail → **không thể steal
   focus** vào elevated window từ tiến trình medium (test tự động phải yêu cầu user click tay).
3. `WM_GETTEXT` cross-process sang elevated = 0 → **không đọc được text elevated** bằng
   message; UIA từ medium cũng không thấy control (dùng UIA là `Edit` → 0 kết quả).
4. **Hệ quả uiAccess (`P1-4`):** hook/injector của VietIME chạy medium sẽ **không** gõ
   được vào app elevated → muốn gõ được phải có binary signed + `uiAccess=true` + manifest
   (đúng hướng đã plan); nếu không làm uiAccess thì mặc định **fail-open = gõ thẳng**,
   không can thiệp — chấp nhận được.

Evidence: output test (bảng trên) + `%TEMP%\tfield_text.log` (chỉ có của medium) +
script `test_uipi2.ps1`.

## Điều kiện exit & liên quan

- [x] 10/10 có kết quả → **ADR-005b không kích hoạt** (#2/#4 pass).
- [x] Finding S3-1 (GetStart) → nhận cho `WIN-010`.
- [x] Finding S3-2 (wcslen/AV) → đã fix trong spike, thành yêu cầu impl + `tsf-registration-spike.md`.
- [x] Kết quả #9 → `P1-3` (field detect), `WIN-017`.
- [x] Kết quả #8 → chốt preset Chrome + `WIN-008`.
- [x] Kết quả #10 → uiAccess plan `P1-4` (gõ vào app elevated cần signed + `uiAccess=true`).
