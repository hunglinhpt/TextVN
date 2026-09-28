# Antivirus false positive (RW3) — Kaspersky flag build production

- **Ngày:** 2026-09-27 | **Trạng thái:** đang xử lý | **Risk:** `RW3` (`docs/20-windows/P1-0-MASTER-PLAN.md`)
- Cross-ref: `P1-4 §5 Ký số & nguồn gốc`, task `WIN-055` (SignPath), `S5` (không admin), `G5` (push xanh = done)
- Ownership: ban đầu = agent kia (installer/, P1-4); **2026-09-27 user chọn "Tôi làm luôn"** → override G11 cho task AV này.

## 1. Sự việc

| Mốc | Thông tin |
|---|---|
| 2026-09-27 | Build production (portable zip / installer) bị **Kaspersky** flag trên **máy thứ 2** (user report) |
| Còn thiếu | **Tên detection + file bị flag chính xác** (AV-5) — chưa có, cần lấy từ máy Kaspersky |
| Máy dev này | `Get-MpThreatDetection` / `Get-MpThreat` = **rỗng**, Defender real-time đang **TẮT** → không phải Defender local; không có bằng chứng AV nào trên máy này |

## 2. Bang trigger — đã xác minh trên code/installer

| # | Hành vi trong package | Heuristics AV gọi là | Vị trí (đã xác minh) |
|---|---|---|---|
| T1 | `textvn-hook.exe`: `SetWindowsHookExW(WH_KEYBOARD_LL)` toàn cục + `SendInput` gõ giả lập | **Keylogger / Keyinject** (T1056.001) — nghi ngờ nặng nhất | `adapters/windows-hook`, `P1-2 §1`; **chỉ khi build `-IncludeCompatibilityHook`** (`TextVN-setup.iss` dòng 52–53 `#ifdef`); release mặc định = `tsf-only`, không build/đóng gói hook (`build-release.ps1` dòng 92 + 112–116) — cập nhật 2026-09-28 |
| T2 | `textvn-cli.exe register` chạy `runhidden` → ghi registry COM/TSF (HKCR CLSID + TextServicesFramework category) | Kiểu regsvr32-style COM persistence | `TextVN-setup.iss` `[Run]` dòng 73–76 (register = dòng 75) |
| T3 | `Software\Microsoft\Windows\CurrentVersion\Run` + `--autostart` | Autostart persistence (T1547.001) | `TextVN-setup.iss` `[Registry]` dòng 70–71 |
| T4 | `textvn-tsf.dll` được load vào mọi process (explorer, browser, Office…) | DLL injection-like | thiết kế TSF (bắt buộc) |
| T5 | **Toàn bộ artifact `NotSigned`** + publisher vô danh, không reputation | Static heuristics bắn mạnh khi + không cert | `Get-AuthenticodeSignature` = `NotSigned` trên mọi exe/dll (verify 2026-09-27) |
| T6 | Inno Setup nén LZMA ultra, `runhidden`, zip download (Mark-of-the-Web) | Packed/installer heuristic + SmartScreen "uncommon" | `textvn-setup.iss` `[Setup]` |

→ Đúng dự báo `RW3`: *"hook + SendInput = hành vi keyinject"*. **Không có hành vi tải code từ mạng / anti-debug / che giấu** trong package (đã review `build_installer.ps1`, `install.ps1`, `.iss`) — FP thuần vì hành vi IME + chưa ký.

## 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit)

```
7B59448840B68FE4ED4BCC6F9621DE4AC6137C30F058B4BDC8D259EEAE6894FC  dist\textvn-portable-0.1.0-windows-x64\textvn-tray.exe
FC5DE5FFC914382C37413279C50169038CA4E20EF15201D3E8C1A7BF6C1C1F6A  dist\textvn-portable-0.1.0-windows-x64\textvn-tsf.dll
C165CFC21C5C3C7812EFAF6ECF1B098726FA9CD29B01053B568B3FF36C271E4C  dist\textvn-portable-0.1.0-windows-x64\textvn.exe
69A80238A96D0D7263824540A2BD4938F9A41BACD2174C5C70BFA57F185A69C7  dist\textvn-portable-0.1.0-windows-x64\textvn_ffi.dll
5410E2745D8A9D0FACBBAC4D3296C9C44254DBB5A6E5A9E9D4B6D71F79A73DEA  dist\textvn-portable-0.1.0-windows-x64.zip
```

Regenerate sau mỗi build: `Get-FileHash -Algorithm SHA256 <file>` (installer: script in sẵn — `build_installer.ps1` bước `[AV-3]`).

## 4. Ke hoach xu ly (task)

| ID | Việc | Khi nào | Người |
|---|---|---|---|
| **AV-1** | **Submit false positive cho Kaspersky** — bước cụ thể ở §5; lặp lại với hash của `textvn-setup.exe` khi build | Ngay (không cần cert) | User (máy có Kaspersky) |
| **AV-2** | **Ký code — SignPath OSS** (fix thật, đúng `P1-4 §5` + `WIN-055`): ký `textvn.exe`, `textvn-tray.exe`, `textvn-hook.exe`, `textvn-tsf.dll`, `textvn-setup.exe` | Trung hạn | Cần user đăng ký account → giao agent |
| **AV-3** | Release gate: trước khi publish → in SHA256 (đã gắn vào `build_installer.ps1`) → submit scan (Kaspersky OpenTip / VT) → ghi `docs/release/signing-status.md` (`P1-4 §5`) | Mỗi release | Agent (CI/release) |
| **AV-4** | Các trigger **đã cân nhắc KHÔNG làm** (xem §4.1) | — | — |
| **AV-5** | Lấy **tên detection + file chính xác** từ máy Kaspersky → cập nhật §1/§2 rồi chốt lại trigger chính | Khi có thông tin | User |

### 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm

> **Cập nhật 2026-09-28:** phương án "Tách hook thành component tùy chọn" **đã làm** (xem cột Kết luận) — 2 dòng còn lại giữ nguyên.

| Phương án | Kết luận | Lý do |
|---|---|---|
| Bỏ `textvn-hook.exe` khỏi installer | **Không** | Đây là runtime feature (fallback cho game/app legacy/elevated — `P1-2 §1`); spawn chỉ qua menu opt-in (`tray/src/main.rs` dòng 459–462 `WM_START_COMPATIBILITY_HOOK`), watchdog không spawn (`tray/src/ipc_server.rs` 427–431 chỉ giữ chân thread), `doctor` báo `hook_running`. Bỏ hẳn = hỏng tính năng |
| Đổi mặc định task `autostart` → unchecked | **Hoãn** | Lợi ích AV ở tầng behavior rất nhỏ (static scan installer vẫn thấy entry `[Registry]` bất kể default); đổi UX cần chủ đích. Revisit sau AV-5 nếu Kaspersky flag chính installer |
| Tách hook thành component tùy chọn trong installer | **Đã làm** (2026-09-28) | `-IncludeCompatibilityHook` mặc định OFF: `build-release.ps1` dòng 92 profile `tsf-only` + dòng 112–116 loại `textvn-win-hook` khỏi build; installer chứa hook chỉ khi build riêng với cờ (`TextVN-setup.iss` dòng 52–53). Package mặc định hết file hook để static scan soi (T1); opt-in 2 lớp = gói build riêng + người dùng bấm menu "Chế độ tương thích" mỗi phiên |

## 5. Runbook submit Kaspersky (AV-1)

1. Máy có Kaspersky → mở https://opentip.kaspersky.com → paste **SHA256** (§3) của file bị flag → xem detection name (ghi lại vào §1, = AV-5).
2. Ở trang kết quả nếu Kaspersky báo malware → có link **"Report as false positive"** (cần đăng nhập tài khoản Kaspersky) → báo cáo với nội dung:
   - Tên file + SHA256 (chính xác từng chữ);
   - Link nguồn: `https://github.com/hunglinhpt/TextVN` (GPL-3.0, mã nguồn mở, dựng từ source, không có codec/packer);
   - Mô tả ngắn (EN): *"Open-source Vietnamese IME for Windows (Rust + TSF). Installer registers a Text Services Framework input method and optionally adds an autostart entry. Contains a low-level keyboard hook adapter (WH_KEYBOARD_LL) by design — it is an input method, not a keylogger. Source: <repo link>."*
3. Nếu file chưa từng bị phân tích → dùng **https://opentip.kaspersky.com/ → "Request analysis"** (upload file ≤ vài MB) rồi báo cáo FP theo bước 2.
4. Chạy lại file sau khi Kaspersky whitelist → ghi kết quả (ngày, tên detection cũ) vào bảng §1.

## 6. Ke hoach ky so (AV-2) — theo P1-4 §5

1. Đăng ký **SignPath OSS** (miễn phí cho repo OSS GPL): https://www.signpath.io/ → "Free code signing for open source" → nối repo `hunglinhpt/TextVN` (cần quyền owner GitHub).
2. Tạo signing policy (OV/EV-backed) + GitHub App của SignPath → secret `SIGNPATH_API_TOKEN` trong repo.
3. Pipeline ký (tên dự kiến `signpath.yml` theo `P1-4 §5`): submit signing-run theo commit SHA → tải artifact đã ký → **verify**: `Get-AuthenticodeSignature <file>` phải = `Valid`.
4. Sau release đầu tiên ký: submit danh sách hash cho **Microsoft Security Intelligence** (https://www.microsoft.com/wdsi/filesubmission) + VirusTotal → ghi `docs/release/signing-status.md`.
5. Portable: hướng dẫn người dùng `Unblock-File` nếu bị Mark-of-the-Web (`P1-4 §5`).

> Chưa tạo `signpath.yml` trong repo lúc này vì cần account/token thật của SignPath (bước 1–2 = user) — tránh CI chết nổi. Tạo khi có token.

## 7. Bang ghi chu

- `installer/windows/textvn-setup.iss` + `installer/windows/build_installer.ps1` = khu vực agent kia; task này được user override ("Tôi làm luôn") ngày 2026-09-27. Các thay đổi đi kèm: `build_installer.ps1` thêm in SHA256 installer (bước `[AV-3]`) + file doc này.
- `build-release.ps1` (root) đang `M` = WIP của agent kia → **không đụng**.
- S2: doc này không chứa log phím/người dùng — chỉ hash, tên file, hành vi đã công khai trong source.

## 8. Kiến trúc hành vi — nguyên tắc "không viết code giống malware" (user 2026-09-28)

Yêu cầu của user (2026-09-28): tối ưu kiến trúc để không kích hoạt heuristic AV (Kaspersky/Defender) — dùng API chính thống của hệ điều hành, tránh hook toàn cục thiếu bộ lọc. Áp dụng cho code Windows hiện tại và mọi adapter mới.

### 8.1. Nguyên tắc

| # | Nguyên tắc | Trạng thái |
|---|---|---|
| A1 | **Windows: TSF là đường gõ mặc định** — TIP đăng ký per-user tự động lần đầu mở tray (`tray/src/main.rs:300` `ensure_tsf_tip_registered`), installer `[Run] textvn-cli register` (`TextVN-setup.iss:75`); không cần admin (S5) | ✅ |
| A2 | **Release mặc định = `tsf-only`** — `build-release.ps1:92` chọn profile, dòng 112–116 loại `textvn-win-hook` khỏi build khi không có `-IncludeCompatibilityHook`; hook chỉ có trong gói build riêng có chủ đích | ✅ |
| A3 | **Global hook = tùy chọn 2 lớp, lọc đầy đủ** — chỉ `WH_KEYBOARD_LL` (không inject DLL; duy nhất 1 chỗ trong code phát hành: `adapters/windows-hook/src/main.rs:152`); spawn chỉ khi người dùng bấm menu "Chế độ tương thích" (`tray/src/menu.rs:265` → `WM_START_COMPATIBILITY_HOOK` → `tray/src/main.rs:459-462`), mỗi phiên — không auto-spawn, watchdog không spawn (`tray/src/ipc_server.rs:427-431`); filter chain: guard `LLKHF_INJECTED` → cổng `GLOBAL_ENABLED` → bỏ key-up / chord hệ thống → gate S3 field-detect (`field-detect/src/lib.rs:132-135`: Unknown/secure = đóng, fail-safe pass-through) → timebox callback → `SendInput` có loop-guard + fail-open | ✅ |
| A4 | **Cấm process injection** — không `CreateRemoteThread` / `WriteProcessMemory` / `VirtualAllocEx` / … (§8.3); regression guard = `repo-hygiene` **check #8** mỗi push (Farch-3) | ✅ |
| A5 | **Không log phím/text người dùng** (S2) — log chỉ PID/heartbeat/kết quả | ✅ |
| A6 | **Linux: framework IME chính thống** — fcitx5/ibus (`adapters/linux-*`) | ✅ |
| A7 | **macOS: InputMethodKit (IMK)** — repo chưa có adapter macOS | ⏳ Farch-4 (roadmap) |

### 8.2. Regression guard (Farch-3)

- `.github/scripts/check_no_injection_apis.py` — chạy ở `repo-hygiene` **check #8** mỗi push/PR (`docs/00-WORKFLOW.md` §12.2); local: `python .github/scripts/check_no_injection_apis.py`.
- Quét file code theo git (tracked + untracked chưa ignore; `*.rs *.c *.h *.cpp *.py *.ps1 *.sh *.iss`): tên API cấm (§8.3) + hook non-LL (`WH_KEYBOARD`/`WH_MOUSE` không phải `*_LL`) + `SetWindowsHookEx` không ghi rõ LL trên cùng dòng.
- Miễn trừ: file của chính guard này + `spikes/` (mã nháp probe không phát hành — WIN-005 `hook-spike`: khai báo P/Invoke và `SetWindowsHookExW(13, …)` = `WH_KEYBOARD_LL` dạng số). Copy nội dung từ `spikes/` sang `adapters/` vẫn bị chặn tại đích.
- Selftest 2026-09-28: guard PASS trên repo (129 file code) + 8 case dương/âm (LL được phép, `WH_KEYBOARD,`/`WH_CBT`/`CreateRemoteThread`/`PTRACE_ATTACH` bị chặn).

### 8.3. API bị cấm

| Nhóm | API |
|---|---|
| Thread/process injection | `CreateRemoteThread`, `RtlCreateUserThread`, `QueueUserAPC`, `SetThreadContext`, `Wow64SetContextThread`, `NtSetContextThread` |
| Đọc/ghi bộ nhớ process khác | `WriteProcessMemory`, `VirtualAllocEx`, `NtWriteVirtualMemory`, `process_vm_writev`, `PTRACE_ATTACH` |
| Hook non-LL (inject DLL vào process đích) | `WH_KEYBOARD`, `WH_MOUSE` (không phải `*_LL`) |

Được phép (có lý do IME chính đáng): `WH_KEYBOARD_LL` + `SendInput` (chỉ trong gói tương thích opt-in, có loop-guard), `GetKeyState` (đọc modifier), toàn bộ COM/TSF (`ITf*`).

### 8.4. Bằng chứng audit (2026-09-28)

| Mục | Kết quả |
|---|---|
| Grep 11 API injection / ghi vùng nhớ khác process trong repo | **0 match** |
| `SetWindowsHookEx` trong code phát hành | đúng **1 chỗ** = `WH_KEYBOARD_LL` (`adapters/windows-hook/src/main.rs:152`) |
| Auto-spawn hook? | **Không** — chỉ qua menu opt-in; `hook_watchdog_loop` chỉ sleep-loop (`tray/src/ipc_server.rs:427-431`) |
| Release mặc định | `tsf-only`, `textvn-win-hook` bị exclude (`build-release.ps1:92,112-116`) |
| Field-detect fail-safe | Unknown/secure → gate S3 đóng → pass-through (`field-detect/src/lib.rs:132-135`) |

### 8.5. Findings (G6 — giữ ID, không xóa)

| ID | Mức độ | Finding | Trạng thái |
|---|---|---|---|
| Farch-1 | major | Doc này stale so với kiến trúc opt-in `tsf-only`: §2 T1 "có trong installer", T2/T3 sai số dòng, §4.1 "watchdog spawn" + "Hoãn tách component" | còn mở |
| Farch-2 | major | Chưa có mục ghi nhận chính sách kiến trúc hành vi (nguyên tắc user 2026-09-28) → agent sau có thể vô tình phá | còn mở |
| Farch-3 | major | Không có regression guard API cấm (G-rule: không guard = chưa Done) | còn mở |
| Farch-4 | minor | macOS chưa có adapter IMK (đường chính thống macOS) — chưa bắt đầu, không phải bug | còn mở (roadmap) |
| Farch-5 | minor | Verify: không có auto-spawn hook — chuỗi opt-in đầy đủ (menu → `WM_START_COMPATIBILITY_HOOK` → spawn; watchdog no-op) | đóng / verified 2026-09-28 |
| Farch-6 | minor | Vòng 2: `docs/specs/verified-ops.md` A5 vẫn ghi "7 check / 7/7" sau khi thêm check #8 | còn mở |
