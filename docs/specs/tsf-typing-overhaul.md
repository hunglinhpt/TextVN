# TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

- **Ngày:** 2026-09-28 · **Phạm vi:** `adapters/windows-tsf`, `cli/src/register.rs`, `tray/`, `installer/windows/`, `adapters/windows-hook` (sửa nhỏ)
- Cross-ref: `../20-windows/P1-1-tsf.md` (§3–§8), `tsf-spike.md` (#4, #5, #9), `tsf-registration-spike.md`, `antivirus-false-positive.md` §8–§9
- Kiểm chứng tự động: `cargo test -p textvn-win-tsf` (mô hình composition + gate), `textvn-cli replay corpus/shared corpus/win --adapter tsf` (100/100 case), `cargo clippy --target x86_64-pc-windows-msvc --workspace -- -D warnings`
- **Chưa kiểm chứng trên Windows thật** (container build Linux): cần chạy smoke `P1-5 §4.1` + app matrix trước khi gắn nhãn production.

## 1. Nguyên nhân gốc (Findings — G6, giữ ID)

| ID | Mức | Hiện tượng | Nguyên nhân | Fix |
|---|---|---|---|---|
| Ftsf-1 | blocker | TSF không bao giờ biến đổi phím — gõ ra chữ Latin thô ở mọi app | `ThreadState` khởi tạo `SecurityState::Unknown`; chỉ `publish_uia` mở gate nhưng **không có caller** trong DLL → `resolve_strategy` luôn `Passthrough` | Gate đọc tín hiệu TSF in-proc ở mỗi phím trong edit session: `ITfContext::GetStatus` (read-only), `GUID_PROP_INPUTSCOPE` (`IS_PASSWORD`/PIN), style `ES_PASSWORD` của HWND focus → `classify_tsf_field` → `apply_tsf_probe` |
| Ftsf-2 | blocker | Cài per-user: TextVN không có trong Win+Space | `do_register` gọi `Register()?` (ghi HKLM) — không admin thì lỗi và **return trước** `InstallLayoutOrTip`/Activate | Mỗi bước độc lập; `RegisterProfile` (API mới) → fallback layout CTF per-user dưới `HKCU\Software\Microsoft\CTF\TIP\{CLSID}` (P1-1 §8); installer hỏi cấp quyền admin MỘT lần cho `register --scope machine` |
| Ftsf-3 | blocker | Có biến đổi nhưng ra chữ lặp/lộn (`aá`, `đđược`) | Mỗi phím `StartComposition` mới, không bao giờ `EndComposition`; nhánh `Preedit` bỏ qua `delete_count`; `ShiftStart` về trước caret không hoạt động ở app IMM32 (CUAS chỉ expose composition) | **Mô hình composition** (`compose.rs`): cả từ nằm trong MỘT composition do TIP sở hữu, mọi `delete_count` rơi trong vùng đó; commit tại ranh giới từ |
| Ftsf-4 | major | App treo khi đổi bộ gõ/đóng cửa sổ | `Deactivate` → `IpcClient::stop()` **join** thread đang chặn ở `read_exact` trên pipe | Một IPC client/process, không bao giờ join; `DllCanUnloadNow` giữ DLL khi thread đã chạy |
| Ftsf-5 | major | Tắt tiếng Việt từ khay không có tác dụng trong app TSF | Client bỏ qua `StateUpdate{app_id:"*"}`; `Snapshot` không mang trạng thái toàn cục | Client xử lý `"*"`; tray thêm `"*"` vào Snapshot; offline đọc `state.json` |
| Ftsf-6 | major | Ctrl+Shift+Space không đổi chế độ | Hotkey xử lý ở cả `OnKeyDown` lẫn `OnPreservedKey` → đảo 2 lần | Chỉ xử lý ở `OnPreservedKey`; đổi ngay trong process + báo tray |
| Ftsf-7 | major | Delete làm chữ sau viết hoa; F1 chèn `p`; mũi tên chèn `%` | `vk_to_unicode` trả `vk` khi `ToUnicode` ≠ 1 ký tự | `ToUnicodeEx` (scan code + layout thread + `wFlags=0x4` không phá dead-key); không sinh ký tự → 0 (`KeyKind::Other`) |
| Ftsf-8 | major | Word/Win32 Edit: phím bị nuốt hoặc buffer lệch | `OnTestKeyDown` trả TRUE theo dải VK rồi `OnKeyDown` có thể trả FALSE; Space/Backspace không qua engine khi test FALSE | Xử lý ĐÚNG MỘT LẦN ở pha tới trước (mẫu Weasel), `pending_eaten_vk` xác nhận ở pha sau |
| Ftsf-9 | major | Không gõ được ở ô tìm kiếm Start/Settings/app Store | Thiếu category `GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT`; DLL per-user không đọc được từ AppContainer | Đăng ký `IMMERSIVESUPPORT` + `SYSTRAYSUPPORT`; ACL đọc/thực thi cho `ALL APPLICATION PACKAGES` + `ALL RESTRICTED APPLICATION PACKAGES` trên DLL |
| Ftsf-10 | minor | Portable chuyển thư mục → TIP trỏ DLL cũ | Tray chỉ kiểm tra key CLSID tồn tại | Kiểm tra `InprocServer32` = DLL cạnh exe + file tồn tại + profile TIP (HKLM/HKCU) |
| Ftsf-11 | minor | Config (VNI…) bị bỏ qua khi tray tắt | Engine TSF khởi tạo default, chỉ reload khi có IPC | Nạp `config.json` lúc Activate; reload chỉ ở ranh giới từ (reload giữa từ xóa buffer engine) |
| Ftsf-12 | minor | Panic trong callback COM = abort app | Không có `catch_unwind` ở `ITfKeyEventSink`/`DoEditSession` | `guarded()`/`catch_unwind` + `try_borrow*` (không panic vì RefCell) |

## 2. Mô hình composition (quyết định kiến trúc)

```text
phím → [OnTestKeyDown | OnKeyDown] (xử lý 1 lần) → RequestEditSession(TF_ES_SYNC|READWRITE)
  DoEditSession(ec):
    selection → self-heal (caret nằm NGOÀI composition ⇒ commit nguyên văn, reset)
    gate S3 (read-only / InputScope password-PIN / ES_PASSWORD) → Passthrough ⇒ commit, phím đi thẳng
    engine.key(vk, ch)  ← chỉ gọi TRONG session: không cấp lock ⇒ engine không thấy phím
    plan_key(comp_text, key, result) → {delete_before, text, end, eaten}
    áp: (xóa trước composition nếu xác minh được) → Start/SetText composition → caret cuối → EndComposition
```

Bất biến (test trong `compose.rs`): phím không bị ăn ⇒ composition đã đóng trước khi app nhận phím
(Enter gửi tin nhắn không bao giờ bỏ lại chữ treo — B2); RESTORE/COMMIT ở ranh giới bỏ ký tự ranh
giới khỏi `insert` để app nhận **phím thật** (Enter là Enter, không phải `\n` chèn vào text).
Mọi strategy AppDB ngoài `Passthrough` đều dùng composition trong TSF — đây là đường text duy nhất
hợp lệ cho cả app TSF-aware lẫn IMM32/CUAS; `SelectionReplace`/`ForwardAsCommit` còn ý nghĩa cho hook.

## 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9)

| Thay đổi | Lý do |
|---|---|
| DLL TSF không UIA, không truy cập process khác; field detect đọc tín hiệu in-proc của chính app | Hành vi IME chuẩn Microsoft; không "đọc cửa sổ process khác" |
| Không `SendInput`/Backspace giả trong gói mặc định (composition thay thế) | `SendInput` là tín hiệu keyinject (T1) |
| `input.dll` nạp bằng `LoadLibraryExW(LOAD_LIBRARY_SEARCH_SYSTEM32)` | Chống DLL planting; không resolve động ngoài System32 |
| Bỏ `OpenProcess(PROCESS_TERMINATE)` + `TerminateProcess` khỏi `TextVN --stop` | Mẫu "process killer" |
| Bỏ `textvn_ffi.dll` (export `ime_key`…) khỏi gói Windows | Runtime link engine tĩnh; DLL rời chỉ thêm bề mặt quét |
| Pipe `PIPE_REJECT_REMOTE_CLIENTS` | Chỉ client cục bộ |
| Registry ghi bằng Win32 API, không spawn `reg.exe` | Child process sửa registry là tín hiệu persistence |
