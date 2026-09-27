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
| T1 | `textvn-hook.exe`: `SetWindowsHookExW(WH_KEYBOARD_LL)` toàn cục + `SendInput` gõ giả lập | **Keylogger / Keyinject** (T1056.001) — nghi ngờ nặng nhất | `adapters/windows-hook`, `P1-2 §1`; **có trong installer** (`textvn-setup.iss` dòng 51), không có trong portable zip |
| T2 | `textvn.exe register` chạy `runhidden` → ghi registry COM/TSF (HKCR CLSID + TextServicesFramework category) | Kiểu regsvr32-style COM persistence | `textvn-setup.iss` `[Run]` dòng 68–70 |
| T3 | `Software\Microsoft\Windows\CurrentVersion\Run` + `--autostart` | Autostart persistence (T1547.001) | `textvn-setup.iss` `[Registry]` dòng 64 |
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

| Phương án | Kết luận | Lý do |
|---|---|---|
| Bỏ `textvn-hook.exe` khỏi installer | **Không** | Đây là runtime feature (fallback cho game/app legacy/elevated — `P1-2 §1`), tray có watchdog spawn nó (`tray/src/ipc_server.rs:263`), `doctor` check nó chạy. Bỏ = hỏng tính năng |
| Đổi mặc định task `autostart` → unchecked | **Hoãn** | Lợi ích AV ở tầng behavior rất nhỏ (static scan installer vẫn thấy entry `[Registry]` bất kể default); đổi UX cần chủ đích. Revisit sau AV-5 nếu Kaspersky flag chính installer |
| Tách hook thành component tùy chọn trong installer | **Hoãn** | File vẫn nằm trong installer → static analysis không đổi; chỉ benefit khi người dùng không bật. Revisit cùng AV-4 |

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
