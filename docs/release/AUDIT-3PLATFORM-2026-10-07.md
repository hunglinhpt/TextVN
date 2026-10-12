# Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`)

> Ngày: 2026-10-07. Phương pháp: 3 agent audit độc lập đọc **toàn bộ** file
> theo platform (Windows / macOS / Linux+shared) + auditor chính đối chứng
> tay từng finding trọng yếu (đánh dấu 🔎). Phát hiện từ finding trước
> (R2-91…R2-100, B13–B20) đã được đối chiếu để tránh lặp.
> Trạng thái repo khi audit: working tree sạch, CI xanh.

## Tổng quan

| Platform | Đọc | CAO | TRUNG | THẤP |
|---|---|---|---|---|
| Windows | tray/src (1602 dòng), cli/register.rs (2539), windows-tsf (9 file), .iss, build-release, tests | 2 | 6 | 8 |
| macOS | IMKLib (9 file), TextVNAppLib (7 file), FFI header, packaging + scripts | 2 | 5 | 12 |
| Linux + shared | linux-common (9 file), ibus + fcitx5 engines, linux-settings, core, ipc, appdb, strategy, config, ffi, packaging | 0 | 4 | 9 |

---

## A. WINDOWS

### A1. [CAO] ILOT spec cho bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100) 🔎
`cli/src/register.rs:1351-1356` — khi gỡ, value name của
`HKCU\Control Panel\International\User Profile\<tag>` (dạng `042A:{…}{…}`,
**không có `0x`**) được truyền nguyên văn vào `call_layout_or_tip_spec`, trong
khi mọi lời gọi khác dùng `layout_spec()` = `0x{lang:04X}:…` (register.rs:54).
Nếu parser của input.dll yêu cầu `0x`, ILOT trả FALSE âm thầm → bàn phím
Windows tự gắn không được gỡ — đúng mục tiêu R2-100 nhưng chỉ đúng một nửa
(xoá tag khỏi `Languages` nhưng còn bàn phím tự gắn); test CI không bắt được.
**Fix:** chuẩn hoá `if !name.starts_with("0x") { name = format!("0x{name}") }`;
verify 1 lần qua `register.log` (dòng `InstallLayoutOrTip(… UNINSTALL (Windows
tự gắn)) → OK/FAIL`).

### A2. [CAO] Mirror WOW64 CTF cho **HKCU** là đường registry chết 🔎
`cli/src/register.rs:1384-1385` — `wow_ctf_tip_key()` =
`Software\WOW6432Node\Microsoft\CTF\TIP\…` dùng cho **cả hai** root. Nhưng
WOW64 redirection chỉ tách view cho `HKLM\Software` và
`HKCU\Software\Classes` (COM — phần mirror này ĐÚNG); phần còn lại của
`HKCU\Software` là **shared** → `HKCU\Software\WOW6432Node\Microsoft\CTF\TIP`
không bao giờ được app 32-bit đọc (chức năng không gãy vì path shared đã phục
vụ, nhưng là payload chết + sẽ đánh lừa người sửa sau). `unregister_wow64_mirror`
xoá đối xứng nên ghost gate không bắt. **Fix:** mirror CTF chỉ khi
`Scope::Machine`; `Scope::User` chỉ giữ mirror COM. Ghi comment giải thích.

### A3. [TRUNG] Cài/nâng cấp khi tray đang chạy không có bước dừng tray
`installer/windows/TextVN-setup.iss:240-255` — `PrepareToInstall` chỉ chờ
uninstaller Inno khác + rename DLL; `TextVN.exe`/`textvn-cli.exe` đang chạy
(autostart — ca phổ biến nhất của nâng cấp) không được dừng. RestartManager
của Inno chỉ chạy khi elevated; tray **không xử lý `WM_QUERYENDSESSION`**
(main.rs:1185-1319 rơi xuống DefWindowProc) → RM fail. Test CI luôn
Stop-Process trước khi cài nên ca này chưa từng được test (bất đối xứng với
R2-99 đã xử lý chiều gỡ). **Fix:** `PrepareToInstall` chạy
`{app}\TextVN.exe --stop` (ewWaitUntilTerminated) trước rename; thêm handler
`WM_QUERYENDSESSION`/`WM_ENDSESSION` đi đường graceful; thêm test tray-đang-
chạy rồi cài `/VERYSILENT`.

### A4. [TRUNG] Vòng dò `wait_stopped` tự nhận sở hữu mutex + ceiling 100s
`tray/src/main.rs:1468-1486` — `CreateMutexW(None, TRUE, …)` (bInitialOwner)
khiến `--stop` TRỞ THÀNH CHỦ mutex trong khoảng tray cũ đang dọn → một
TextVN.exe người dùng mở đúng khoảnh khắc tưởng instance khác đang chạy và
thoát (main.rs:968-990). Tổng ceiling thực tế ~100s (mỗi vòng có thể cộng
`wait_exit(5000)`) trong khi thông báo ghi "did not stop within 3s".
**Fix:** `OpenMutexW(SYNCHRONIZE, FALSE, …)`; deadline tổng; sửa thông báo.

### A5. [TRUNG] `ShellExecuteExW` (UAC machine) trên thread chưa CoInitialize
`tray/src/main.rs:544-556` — `run_elevated_and_wait` chạy trong background
thread TSF/offer không hề `CoInitializeEx`; MSDN: ShellExecuteEx có thể fail.
Fail → marker bị xoá (main.rs:473-475) → hỏi lại lần sau (prompt lặp).
**Fix:** `CoInitializeEx(APARTMENTTHREADED)` đầu thread/`run_elevated_and_wait`.

### A6. [TRUNG] Uninstall "over-the-shoulder" (admin khác user gốc) để lại ghost
`installer/windows/TextVN-setup.iss:170-181` — `[UninstallRun]` chạy trong
token elevated: `do_unregister` đọc marker + xoá HKCU **của admin**, không
phải user gốc → user gốc còn COM override trỏ DLL đã xoá (TSF nạp fail mỗi
focus). Chiều CÀI đã có ghi nhận (iss:269-272) nhưng chiều GỠ không có
mitigation. **Fix:** `[Code]` `CurUninstallStepChanged(usUninstall)` +
`ExecAsOriginalUser` cho entry unregister user-scope; giữ machine-scope bằng
`Exec` elevated; ghi docs giới hạn (HKCU user không đăng nhập không gỡ được).
**(RÚT LẠI 2026-10-11 — B21, `docs/specs/win-test-common-errors.md`):** block
`ExecAsOriginalUser` tại `usUninstall` chạy tiến trình `textvn-cli.exe` ngay
trước bước Inno xoá file → giữ `{app}` → Inno không xoá được `TextVN.exe`
(đỏ cả per-user lẫn machine). `[UninstallRun]` đã chạy `unregister
--if-owned-by` ngay từ bước đầu quá trình gỡ nên block này THỪA → đã rút.
**Giới hạn A6 vẫn còn, xử lý sau bằng cơ chế khác** (không dùng
`ExecAsOriginalUser` trong `usUninstall`): admin gỡ hộ user GỐC không đăng
nhập vẫn có thể còn HKCU override. Ghi nhận là hạng mục mở, KHÔNG chặn 0.2.31.

### A7. [TRUNG] Pipe toàn cục đa phiên + không chống squatting
`tray/src/ipc_server.rs:45,424-440` — tên pipe toàn máy: 2 phiên (fast user
switching/RDP) cùng tạo server; client phiên B có thể trúng instance phiên A
→ DACL chặn → rớt về state.json (mất broadcast). Ngược lại khi tray chưa
chạy, process cùng uid squat pipe và phục vụ Snapshot giả — client
(`adapters/windows-tsf/src/ipc_client.rs:100-120,260-316`) không xác thực
server. **Fix:** tên pipe kèm SessionId; client kiểm
`GetNamedPipeServerProcessId` → image `TextVN.exe` cùng user.

### A8. [TRUNG] `activate_machine_for_user` bỏ sót mirror WOW64 HKCU cũ
`cli/src/register.rs:1677-1685` — xoá override HKCU thường nhưng không gọi
`unregister_wow64_mirror(Scope::User)`; gói không có x86 DLL → mirror cũ còn
nguyên (đường chết nhưng rác + gây nhiễu chẩn đoán). **Fix:** gọi thêm.

### A9. [TRUNG] `apply_plan` (edit_session.rs:394-436): delete xong, insert fail → mất chữ đã xoá một nhịp gõ
Xác suất thấp (SetText hiếm fail sau ShiftStart đúng) nhưng là đường
data-loss. **Fix:** giữ snapshot text vùng xoá, khôi phục khi insert fail;
hoặc đánh dấu + reset.

### A10. [TRUNG] COM callback `ActivateEx`/`Deactivate`/`OnSetFocus` không bọc `catch_unwind` (tip.rs:90-124,174-183) — panic thoát qua ABI = abort host (Notepad/Chrome). Nhánh `try_borrow_mut` fail tại ActivateEx: Advise đã thực hiện nhưng cookie không lưu → Deactivate không Unadvise (leak tới hết thread). **Fix:** catch_unwind như `DoEditSession`; borrow-fail trả `Err(E_FAIL)`.

### A11. [TRUNG] `svc.rs:135-235` — `RwLock.write().unwrap()`: poison → panic dây chuyền trong `wnd_proc`. **Fix:** `unwrap_or_else(|e| e.into_inner())`.

### A12. [TRUNG] Uninstaller per-user non-elevated: `schedule-delete` (PFO) luôn ACCESS_DENIED → `.old-*` sót vĩnh viễn (register.rs:793-834). **Fix:** RunOnce như uninstall.ps1 portable đã làm.

### A13. [THẤP] Đa user với bản máy: user B còn per-user override trỏ DLL đã xoá; CLI cũng bị xoá → không tự lành (main.rs:601-603 return sớm). Fix dài hạn: marker machine-level yêu cầu tray user B tự unregister khi thấy path chết.

### A14. [THẤP] `reg_value_names` buffer 512 u16 — value dài hơn 511 ký tự
(`User Profile\<tag>` người dùng sửa được) → dừng sớm âm thầm, ghost bàn phím
(register.rs:1523-1558). Fix: xử lý `ERROR_MORE_DATA`.

### A15. [THẤP] `show_activation_refused_notice_once` (main.rs:337-351) ghi marker
SAU khi người dùng bấm OK — process bị kill giữa chừng → hỏi lại. Nhất
quán hoá theo mẫu `uac_prompt_done` (ghi trước).

### A16. [THẤP] `compose`/`replay` — (đã soát, chỉ ghi nhận) replay post đúng
thread; `to_wide_nul` có nul đệm; không finding mới.

### Đã làm ĐÚNG (đối chiếu, không đụng): machine-scope DLL allowlist
(`Program FilesEvil` case phủ), canonicalize trước ghi HKLM,
grant_appcontainer_read + LocalFree, R2-99 phân tầng graceful→WM_QUIT→force
không deadlock, R2-100 logic thuần có unit test + bằng chứng CI thật
(en-US → en-US,vi → en-US), DACL pipe (BA cho uninstaller elevated),
PeekNamedPipe-gated read, catch_unwind phủ đường phím + DoEditSession,
TF_SELECTION ManuallyDrop cả 2 chiều, PE machine gate I386, smoke retry 15s.

---

## B. macOS

### B1. [CAO] `preeditReplace`/`commitEarly` xoá thật TRƯỚC khi thu hồi marked 🔎
`ApplyReplace.swift:102-133,139-152` — `deleteBackward` gọi khi composition
(marked) còn hiển thị: NSTextView chuẩn xoá **nguyên marked range** thay vì
`real` ký tự thật → chữ thật không xoá, marked mất, `commitEarly` còn dùng
`range` stale. `backspaceType` (line 175-194) làm ĐÚNG thứ tự ngược (retract
trước) — không nhất quán. Mock test chỉ đếm `deletedTotal` nên không bắt.
**Fix:** retract marked trước khi `deleteBackward` trong cả 2 hàm (giống
`backspaceType`); thêm MockTextTarget case mô phỏng "deleteBackward xoá
marked" để khoá hành vi.

### B2. [CAO] IpcClient có 3 đường OFFLINE VĨNH VIỄN (không bao giờ retry)
1. **Path socket quá dài**: `IpcClient.swift:209-212` — guard
   `sun_path` (104B) → `closeSocket(); return` KHÔNG `scheduleRetry()` (mọi
   nhánh lỗi khác đều retry). Home dài → chết im lặng mãi.
2. **Protocol violation không retry**: `IpcClient.swift:320-325,338-343` —
   `tearDown(); notifyDisconnect(); return`. Decode rất strict (ví dụ
   `Snapshot.appdb_version` bắt buộc trong khi `config_version` có default
   `?? 0`) → lệch phiên bản nhỏ = IMK zombie: vẫn gõ bằng config cũ nhưng
   mọi toggle/Settings từ app vô hiệu, không chỉ báo.
3. **`activateServer` không gọi `ipc.connect()`** — chỉ `init` mới connect.
**Fix:** (a) path-dài log FATAL + vẫn retry; (b) violation vẫn
`scheduleRetry()`; (c) `activateServer` gọi `connect()` (idempotent);
(d) nới `appdb_version`/`channel` optional default.

### B3. [TRUNG] `switch_key` ("Alt + Z" / "Caps Lock") là UI chết
`SettingsView.swift:106-133` + `ConfigModel` — chỉ app ghi config; **không
file nào trong IMK đọc `switch_key`** (chỉ `isToggleChord` Ctrl+Shift+Space
cứng). Người dùng đổi phím → không có gì xảy ra. Windows/Linux có parity.
**Fix:** ẩn 2 radio trên macOS ghi "sắp có", hoặc implement (alt_z vào
`isToggleChord` theo config; caps_lock qua `flagsChanged` khi nhả).

### B4. [TRUNG] Crash-report pipeline đứt hoàn toàn
IMK không bao giờ `send(.crashReport(...))` (grep toàn IMKLib: 0 lời gọi) →
`crashCount` ở AppDelegate (badge `exclamationmark.triangle`, tint cam) là
dead code. `NSSetUncaughtExceptionHandler` chỉ bắt NSException, không bắt
fatalError/signal (ghi rõ giới hạn). **Fix:** gửi `.crashReport` từ
`ImeEngine.key` khi `rc != OK`; AppDelegate `crashCount = max(crashCount, count)`,
reset khi nhận `Hello` mới.

### B5. [TRUNG] Nút "Mặc định" xoá macros/english_words/emoji + tắt autostart âm thầm
`ConfigModel.swift:314-317` — `resetToDefaults()` thay toàn bộ config
(S9 nói cấu hình được bảo toàn; Windows/Linux chỉ reset tuỳ chọn gõ).
**Fix:** giữ macros/english_words/emoji/autostart, reset từng trường gõ;
hoặc confirm cảnh báo.

### B6. [TRUNG] Double-click TextVN.app khi autostart bật = im lặng
`AppDelegate.swift:47-60` — `isLoginLaunch` luôn true khi SMAppService
enabled (kể cả double-click chủ ý) → instance 2 tự terminate mà không post
`vn.textvn.awake`. **Fix:** luôn post `awake` trừ khi có arg `--autostart`
thật.

### B7. [TRUNG] gatherAX không dedupe in-flight + không AX messaging timeout
`TextVNInputController.swift:356-368,554-576` — cache TTL 2s hết → phím kế
spawn gather mới; AX query có thể 6s timeout mặc định; nhiều gather chồng
cho cùng pid, last-store-wins có thể ghi context CŨ đè MỚI. **Fix:**
in-flight set theo pid trong FieldDetect; `AXUIElementSetMessagingTimeout(app, 0.3)`.

### B8. [TRUNG] `IpcServer.appStates` tăng vô hạn qua ToggleViEn app_id tuỳ ý
`IpcServer.swift:374` — flood cùng-uid → RAM phình + Snapshot vượt 64KB →
`encodeFrame` nil → client thật không bao giờ nhận Snapshot. **Fix:** cap
256 mục, `app_id.count <= 256`, whitelist ký tự bundle-id.

### B9. [THẤP] `clientPid()` bỏ qua `sender`, dùng `NSWorkspace.frontmostApplication` (TextVNInputController.swift:493-495) — panel phụ/XPC focus → appID sai → preset sai. Cải thiện qua sender metadata.

### B10. [THẤP] `IMKTextTarget.swift:40-42` — `NSUnderlineStyle([]).rawValue` = 0
(none) thay vì `.single` — attribute "none" tường minh có thể tắt underline ở
client tôn trọng attribute.

### B11. [THẤP] Dead code/state chết: `heldMods` (chỉ ghi), `noteFlagsChanged()`
(không caller), `Marked.unmark()` (không caller), comment self-heal stale
(TextVNInputController.swift:273-277 — code ĐÚNG thứ tự, comment nói ngược),
`crashCount += 1` bỏ qua `count`.

### B12. [THẤP] `IpcClient.deinit` gọi `tearDown()` ngoài queue (data race nguyên tắc); thiếu `dispatchPrecondition(.onQueue(.main))` cho delegate của controller.

### B13. [THẤP] EnglishWordsSheet filter âm thầm ("Café"→"caf") + không cap 64;
MacroEditorSheet xoá macro case-sensitive trong khi upsert case-insensitive;
FieldDetect cache entry hết hạn không remove (unbounded); Diagnostics log
không rotate (opt-in).

### B14. [THẤP] Test gap: không có test apply `replace` với `strategy: .backspaceType`
(tổ hợp `effectiveStrategy` sinh ra khi non_preedit=true) — khác đường engine
downgrade trong corpus.

### Đã làm ĐÚNG: fail-open xuyên suốt (engine nil → PASS, key lỗi → reset+PASS);
FFI contract (zero-init, không alloc chéo, readUTF32 chặn 64, R2-43 NULL
appdb); đơn vị đếm scalar/UTF-16/grapheme có test; IPC 2 đầu SO_NOSIGPIPE +
frame cap + violation close + lstat probe + getpeereid; delegate hop main
queue; ViStateStore seed-once + pendingGlobalSync; B2/B11/B13 nhất quán 3
platform; self-heal TRƯỚC engine.key (code đúng, chỉ comment stale); persist
atomic + quarantine corrupt; ConfigWatcher debounce + re-arm; pkg per-user
không sudo, uninstall root chỉ path cố định, notarize staple .app TRƯỚC zip;
corpus mac 114 case + keymap digest CI.

---

## C. LINUX + SHARED

### C1. [TRUNG — latent] `lc_ipc_recv_frame` mất byte khi EAGAIN giữa frame
`ipc_client.c:174-219` — đọc stream không stash: header đọc k byte (0<k<4)
hoặc body đọc một phần rồi EAGAIN → `return 1` **vứt mọi byte đã tiêu thụ** →
lần poll sau đọc phần còn lại làm header → desync vĩnh viễn. **Hiện là
dead-path** (production đi qua `lc_ipc_client_poll` với rbuf per-client có
stash — đã sửa đúng ở 0.2.28 CR-25, có test `test_partial_frame_is_buffered`);
nhưng hàm là API public trong `linux_common.h:155` — mồi cho đúng lỗi CR-25.
**Fix:** xoá khỏi header/API, hoặc đổi chữ ký nhận `lc_ipc_client*` để stash
vào rbuf.

### C2. [TRUNG] Parser IPC dispatch bằng `strstr` toàn JSON → nhầm nhánh khi
state key trùng tên message. `ipc_client.c:222-314` — Snapshot có key state
`"ConfigReload"` (app tên vậy) → nhánh ConfigReload khớp trước → **bỏ trọn
Snapshot** (mất BUG-03 fix); tương tự key `"StateUpdate"`. Điều kiện hẹp
(key phải trùng chính xác) nên thực tế khó. **Fix:** khoá loại message bằng
prefix `{"type":"X"` ngay đầu frame, strstr chỉ dùng cho field sau đó.

### C3. [TRUNG — kiến trúc] IPC Linux là dead path: không tồn tại server
Unix socket nào (grep toàn repo: server duy nhất là Named Pipe Windows).
Client retry connect mỗi 3s vĩnh viễn; mọi sửa 0.2.26 (SEC-04) + vòng 13
(BUG-03 seq) chưa bao giờ chạy production Linux. Fcitx5 gọi
`consumeGlobalIpc()` trên đường phím (engine.cpp:374-378) → syscall thừa khi
offline; IBus chỉ poll init/focus_in — **lệch Fcitx5** (toggle tray sẽ không
tới IBus cho tới đổi focus). **Fix kiến trúc:** xây Linux tray/server
(schema ipc.v1.md đã có) hoặc lazy-init + thống nhất điểm poll (khuyến nghị
focus như fcitx5).

### C4. [TRUNG — latent] Override per-app "dính" sang app kế tiếp (ibus
engine.c:234-246, fcitx5 engine.cpp:294-311): snapshot có override cho app A
→ `s_vi_enabled = FALSE`; sang app B không override → `has_app_override` bị
clear → **không ai restore** base value; `sync_state` no-op (file không đổi)
→ app B kế thừa EN của A. **Fix:** giữ `s_base_vi` riêng; focus:
`enabled = has_override ? override : base`.

### C5. [THẤP] compose.c:92-94 — `cur->len == LC_COMP_MAX` → ghi
`out->text[256]` (OOB intra-object, ghi đè `text_len` rồi gán lại — vô hại
thực dụng nhưng UB; USan bắt). Fix: guard case biên trước nhánh nối.

### C6. [THẤP] utf.c:49-106 — `lc_utf8_to_utf32` không chặn overlong
(0xC0 0x80 → U+0000), surrogate (0xED 0xA0 0x80), > U+10FFFF. Không có
caller production (latent); chiều ngược đã chặn surrogate đúng.

### C7. [THẤP] GTK settings (settings_window.c:483,214-250) — `TvWin` free
qua data-full của cửa sổ chính; cửa sổ con giữ con trỏ `w` và ghi khi destroy
→ UAF tiềm năng theo thứ tự destroy. Fix: disconnect signal trong
`tv_win_free`.

### C8. [THẤP] `tv_activate_fcitx5` (textvn-common.sh:355-363) thêm TextVN
vào `[Groups/0]` cứng, không đọc `GroupOrder`/`DefaultIM` — profile nhiều
group → user không thấy bộ gõ.

### C9. [THẤP] Phụ: settings_model.c:58-63 (`--config` không `/` → ghi CWD);
log.c strncpy không phát hiện truncate + localtime non-reentrant;
field_detect.c là stub luôn UNKNOWN (dead — wire AT-SPI hoặc xoá);
ipc_client.c:88-92 nhánh XDG_CONFIG_HOME thiếu check `[0]=='/'`;
fcitx5 destructor không removeAction (addon reload); stage-linux.sh guard
rm -rf còn lỗ hổng.

### Đã làm ĐÚNG: `lc_ipc_client_poll` + rbuf per-client stash (CR-25 fix +
test); SEC-04 (XDG_RUNTIME_DIR + lstat + SO_PEERCRED — TOCTOU được che thực
chất); FFI fail-open (catch_unwind mọi entry, PASS+ERROR trước đường lỗi, từ
chối chứ không cắt kết quả >64, ABI/null check); core engine (Backspace kiểu
UniKey R2-55, restore-EN 3 nhánh R2-58, fix_uo + dời dấu, MAX_WORD_KEYS=63);
config/appdb untrusted (VersionProbe, deny_unknown_fields, matcher chặn
wildcard, first-match-wins, fail-safe db mới hơn); ipc crate decode phân
biệt Truncated/Oversize/Trailing/Invalid; packaging Linux (manifest nguyên
tử, allowlist chống xoá file lạ R2, shell/xml quote R2-46, FCITX_ADDON_DIRS
R2-45, --system R2-54, e2e IBus/Fcitx5 thật trong HOME tạm); GObject
ref_sink/g_clear_object đúng; fcitx5 catch(...) mọi callback + CapsLock từ
rawKey.

---

## D. ƯU TIÊN XỬ LÝ ĐỀ XUẤT (top 10, ghép 3 platform)

> **TRẠNG THÁI 2026-10-11 (v0.2.31):** TẤT CẢ finding của báo cáo **ĐÃ SỬA**.
> Vòng 16 (v0.2.30) sửa mục 1 (B1), 2 (B2), 3 (A1), 4 (A3), 5 (A2), 6 (B3),
> 7 (B4), 8 (B5), 9 (C2), 10 (C4). Đợt roadmap (v0.2.31) đóng nốt:
> **Windows** A4 (wait_stopped OpenMutex + deadline), A5 (CoInitializeEx), A6
> (uninstall over-the-shoulder ExecAsOriginalUser), A7 (pipe theo phiên + verify
> server image), A8 (dọn WOW64 mirror khi activate machine), A9 (insert-fail
> restore), A10 (catch_unwind callback COM + borrow-fail E_FAIL), A11 (RwLock
> poison), A12 (schedule-delete RunOnce), A13 (dọn ghost per-user mọi hive),
> A14 (ERROR_MORE_DATA retry), A15 (marker ghi trước hộp thoại); **macOS**
> B6 (awake khi double-click), B7 (AX dedupe + timeout), B8 (IPC cap app state),
> B9 (clientPid theo sender), B10 (underline single), B11 (xoá state chết),
> B12 (main-queue precondition), B13 (log xoay 1 MiB), B14 (test replace+backspaceType);
> **Linux** C1 (xoá `lc_ipc_recv_frame`), C3 (thống nhất poll IBus↔Fcitx5),
> C7 (hết UAF cửa sổ cài đặt), C8 (Fcitx5 đọc GroupOrder/DefaultIM),
> C9 (hardening --config/XDG/log/localtime_r/fcitx5 unregisterAction/stage guard/field_detect stub).
> Mỗi mục đã rà ≥2 lần theo 3 view (audit/reviewer/user).
>
> **RÀ LẠI 2026-10-11 (sau 0.2.31, sửa ở 0.2.32):** 9 bản sửa của báo cáo này còn lỗi
> hoặc chưa đạt mục tiêu — chi tiết + bằng chứng ở **mục F**: A3 (R2-102, R2-105),
> A7 (R2-101, R2-109, R2-110), A9 (R2-103), A11 (R2-114), A12 (R2-115), A13 (R2-104), B3 (R2-106),
> C4 (R2-107), C8 (R2-108).

| # | Finding | SEV | Effort |
|---|---|---|---|
| 1 | B1 — macOS preeditReplace/commitEarly xoá trước retract → mất chữ | CAO | nhỏ + test |
| 2 | B2 — macOS IPC 3 đường offline vĩnh viễn | CAO | nhỏ |
| 3 | A1 — ILOT spec thiếu `0x` (R2-100 nửa vời) | CAO | 1 dòng + verify log |
| 4 | A3 — cài/nâng cấp khi tray chạy (thiếu stop + WM_QUERYENDSESSION) | TRUNG | vừa + test |
| 5 | A2 — WOW64 CTF HKCU đường chết (nhận thức + thu hẹp mirror) | TRUNG | nhỏ |
| 6 | B3 — switch_key UI chết trên macOS | TRUNG | vừa (implement hoặc ẩn) |
| 7 | B4 — crash-report pipeline đứt | TRUNG | nhỏ |
| 8 | B5 — "Mặc định" xoá dữ liệu user | TRUNG | nhỏ |
| 9 | C2 — strstr dispatch nhầm nhánh (khoá theo `"type":"`) | TRUNG | nhỏ |
| 10 | C4 — override per-app không restore (Linux) | TRUNG | nhỏ |

## E. Ghi chú quy trình

- Finding W-A2 đã 🔎 đối chứng tay (đúng); W-A1 🔎 (đúng — bất đồng bộ với
  `layout_spec`); B1 🔎 (đúng — `backspaceType` trong cùng file làm đúng thứ
  tự ngược lại, chứng tỏ biết vấn đề). Các finding còn lại trích dẫn
  file:line đầy đủ để tra cứu.
- Audit cũ (R2-91…R2-100, B13–B20) đã đối chiếu: không lặp; các mục
  "đã xử lý sẵn" (macOS BUG-03/M-01/M-03, W-B17) được xác nhận bằng code
  hiện tại chứ không theo tài liệu cũ.
- Không sửa trong đợt này — danh sách trên là input cho vòng fix kế (bắt đầu
  từ top 10; mỗi fix đi kèm test chống tái phát theo quy tắc B-lớp).

## F. Rà lại các bản sửa sau 0.2.31 (2026-10-11) — sửa ở 0.2.32

Rà từng dòng diff `v0.2.29..main` (35 commit vòng 16 + roadmap) và kiểm bằng probe
thật. Finding đánh mã `R2-xx` (sổ `code-review-2026-10-09-round2.md` §3.8).

| Mục audit | Còn lỗi gì | Bằng chứng | Sửa (0.2.32) |
|---|---|---|---|
| A7 | `TextVN.exe --status` **panic** (exit 101) khi tray không chạy — `expect("đã kiểm Some ở trên")` mà không kiểm (commit `0fc9a1d`) | chạy binary: `panicked at tray\src\main.rs:1374` | R2-101: báo `OFFLINE (<lỗi>)`, in đúng pipe đã nối; test-portable kiểm |
| A3 | Tray **thoát ngay ở `WM_QUERYENDSESSION`** — đăng xuất/khởi động lại bị huỷ (Notepad chưa lưu → Cancel) thì tray đã mất | cửa sổ tray là top-level nên nhận mọi lần hỏi | R2-102: QUERYENDSESSION chỉ trả TRUE; thoát ở `WM_ENDSESSION(TRUE)`; kiểm thật bằng SendMessage trên máy dev + test-portable |
| A3 | `PrepareToInstall` gọi `--stop` **không** `--if-image-under "{app}" --force` — nâng cấp dừng nhầm tray của bản cài khác (trái R2-93); tray không tự thoát thì file vẫn khoá | đọc `.iss` | R2-105: thêm cờ; test-installer: tray thư mục khác SỐNG sau nâng cấp + nâng cấp đè tray `{app}` đang chạy |
| A9 | Hoàn tác chạy với MỌI lỗi trong closure — cả `EndComposition` lỗi SAU khi chữ mới đã chèn → chèn lại phần đã xoá cạnh từ mới (lặp chữ), hoặc trả lại chữ Backspace vừa xoá | đọc `apply_plan` | R2-103: lỗi `EndComposition` không còn kích hoạt hoàn tác (như `finish()`) |
| A13 | Dọn hive mọi user **chưa từng xoá được gì**: subkey dựng `\<sid>\…` → Registry API trả `ERROR_BAD_PATHNAME` (161); nhánh `RegLoadKey` thiếu SeBackup/SeRestore, mount lồng, COM per-user ở UsrClass.dat không nạp | probe ctypes: `RegOpenKeyExW(HKU, "\<sid>")` = 161 | R2-104: đường dẫn tương đối HKU (hive đang nạp — cả user gốc khi admin gỡ hộ); BỎ nhánh `RegLoadKey` (unload hỏng = user đó vào profile tạm); test gọi API thật |
| A7 | Pipe đổi tên ở 0.2.31 nhưng app đang mở vẫn nạp DLL TSF bản cũ (chỉ biết tên cũ) → bật/tắt từ khay không tới các app đó tới khi mở lại; xác thực server chỉ so đuôi `textvn.exe` | đọc code; kiểm thật | R2-109: tray phục vụ THÊM tên cũ; R2-110: server phải CÙNG PHIÊN + đúng tên file `TextVN.exe` |
| B3 | macOS có 3 lựa chọn phím chuyển riêng (Alt + Z / Caps Lock) — trái ui-spec (phím chuyển CỐ ĐỊNH) và Windows/Linux; Caps Lock trên IMK không chặn được trạng thái chữ hoa. Nhãn bảng mã/kiểu gõ macOS lệch từ đầu (đánh số, TCVN3 đứng thứ 2, "4. Microsoft") | đọc `SettingsView.swift` | R2-106: macOS về đúng ui-spec; IMK luôn Ctrl+Shift; test parity so cả macOS + thứ tự |
| C4 | **Hồi quy**: phím chuyển/menu không cập nhật base → mỗi lần focus-in/activate gán chế độ = base cũ: đổi V/E rồi chuyển cửa sổ là bị lật lại (IBus + Fcitx5, từ 0.2.30) | đọc `set_vietnamese` (lần ghi của chính nó bị "hấp thụ") | R2-107: cập nhật base khi đổi; e2e "EN/VN giữ sau focus-out/in" |
| A11 | Sót 3 chỗ `.read()/.lock().unwrap()` (`SvcManager::app_override` trong wnd_proc, 2 chỗ `subscribers` ở luồng client IPC) | quét mã `tray/src` | R2-114: poison-safe + gate tự động `no_poison_panicking_lock_unwrap_in_tray` |
| A12 | Fallback RunOnce không kiểm giới hạn 260 ký tự (luật R2-94) mà vẫn báo "đã hẹn xoá" | đọc `schedule_delete` | R2-115: `runonce_cleanup_command` > 260 → báo không hẹn được; test đơn vị |
| C8 | `tv_activate_fcitx5` đoán nhóm bằng `DefaultIM` (bộ gõ mặc định TRONG nhóm) → vẫn luôn nhóm đầu file | profile 2 nhóm: thêm nhầm `[Groups/0]` | R2-108: tra `[GroupOrder] 0=<tên>` → `[Groups/N] Name=`; test đơn vị nhiều nhóm |

Giới hạn còn lại (ghi nhận, không chặn): A6 phần user gốc KHÔNG đăng nhập (hive chưa nạp)
vẫn có thể giữ mục bàn phím chết — không nạp hive người khác trong bộ gỡ vì rủi ro profile tạm.
