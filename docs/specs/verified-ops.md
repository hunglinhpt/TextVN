# Verified Ops — sổ thao tác / tool đã kiểm chứng

> **Mục đích (G4 — `../00-WORKFLOW.md`):** đã kiểm chứng **một lần** → ghi đây → agent sau
> **không kiểm tra lại**. Tra sổ TRƯỚC khi cài tool / verify môi trường / dựng app test.
> Thao tác mới chạy đúng → **append dòng ngay** (kèm ngày). Format:
> `| # | Thao tác (lệnh/đường dẫn cụ thể) | Kết quả (ngày) | Chi tiết |`
> Không ghi secret / token / nội dung người dùng (S2). Không sửa entry cũ.

## A. Repo · Git · Graphify

| # | Thao tác | Kết quả | Chi tiết |
|---|---|---|---|
| A1 | `gh` CLI (`C:\Program Files\GitHub CLI\gh.exe`) — account `hunglinhpt` | ✓ đăng nhập keyring, `gh auth status` OK (2026-09-27) | sẵn sàng cho WIN-005 (chạy workflow GHA) |
| A2 | `git push origin main` (HTTPS) | ✓ (2026-09-27) — commit `5572c74`, `a8f89c2` | luôn add list tường minh (G2/G12) |
| A3 | `graphify update .` tại repo root | ✓ 1767 nodes · 2795 edges · 129 communities (2026-09-27) | chạy sau mỗi push; commit `graphify-out/` riêng |
| A4 | REUSE: SPDX header dòng 1 (`// SPDX-…` / `# SPDX-…` / `<!-- SPDX-… -->`) | ✓ `reuse lint` 888/888 (2026-09-27) | hoạt động cả chế độ `.reuse/dep5` lẫn `REUSE.toml` (WIP agent kia) |
| A5 | GitHub Action **`repo-hygiene`** — 7 check "sạch" repo (`00-WORKFLOW.md §12`) | ✓ run **đầu tiên = success** (2026-09-27, [run 36289296482](https://github.com/hunglinhpt/TextVN/actions/runs/36289296482), commit `a4d80e4`) | chạy local trước khi push: 7/7 pass (git-bash giả định + `python .github/scripts/check_doc_links.py`) — công thức "pre-check local → push → xem run" |

## B. Build · Test · Script

| # | Thao tác | Kết quả | Chi tiết |
|---|---|---|---|
| B1 | cargo = `$env:USERPROFILE\.cargo\bin\cargo.exe` | ✓ `fmt` · `clippy --all-targets` 0 warning · `test` · `cargo deny check` (2026-09-27, `spikes/tsf-min`) | DoD #4 |
| B2 | Chạy script PowerShell chuẩn | `powershell -NoProfile -ExecutionPolicy Bypass -File <x.ps1>` | ASCII-only · không `&&` · không inline `$var` · xem `win-test-common-errors.md` A1–A10 |
| B3 | Kill process tree chỉ cho app test mình mở | `taskkill /PID <id> /T /F`; với Chrome theo profile: `Get-CimInstance Win32_Process -Filter "Name='chrome.exe'"` → lọc `CommandLine -like '*<profile>%'` | **không** kill Chrome/Explorer của người dùng |
| B4 | Tìm cửa sổ (EnumWindows theo pid/class/title) | C# `Add-Type` mẫu `UW`/`UW2`… trong script spike (`%TEMP%\opencode\uia_*.ps1`) | tái sử dụng pattern này, không viết lại |
| B5 | Xem process giữ lock DLL | `tasklist /m tsf_min.dll` | rebuild cdylib bị lock → đóng app giữ lock trước |
| B6 | `reuse lint` qua `tools/win/check_reuse.ps1` | ✓ 970/970 files compliant (2026-09-27) | set UTF-8 tránh crash Windows charmap (E1) |
| B7 | `cargo run -p xtask -- check-tables` | ✓ Khớp 100% `data/tables/*.toml` → `core/` (2026-09-27) | verify tự động tables sinh code |
| B8 | `cargo run -p vietime-cli -- verify` | ✓ FFI ABI 4 tầng + 11 export functions khớp 100% (2026-09-27) | `vietime_ffi.h` đồng bộ tuyệt đối Rust ABI |
| B9 | `cargo run -p vietime-cli -- replay corpus/shared corpus/win` | ✓ 100/100 test cases pass (2026-09-27) | test suite cho B1-B9, secure context, hook, tsf preedit |
| B10 | `cargo run -p vietime-bench` | ✓ ime_key p50=5.7µs (<500µs), resolve p50=57ns (<2ms) (2026-09-27) | đạt budget performance vượt trội |
| B11 | `cargo build -p vietime-win-tsf` | ✓ Sinh `target/debug/vietime_win_tsf.dll` (~1MB) (2026-09-27) | TSF COM in-process DLL hoàn chỉnh (WIN-010) |
| B12 | `cargo test -p vietime-win-tsf` | ✓ 15/15 tests pass (2026-09-27) | Kiểm chứng WIN-010..019: COM lifecycle, IPC client, SelectionReplace, ForwardAsCommit, hotkey toggle EN/VN, secure field |
| B13 | `cargo clippy -p vietime-win-tsf --all-targets -- -D warnings` | ✓ 0 warnings, 0 errors (2026-09-27) | Đảm bảo an toàn bộ nhớ và chuẩn coding Rust nghiêm ngặt |


## C. Windows platform (chi tiết → spike specs)

| # | Thao tác | Kết quả | Chi tiết |
|---|---|---|---|
| C1 | Đăng ký TIP **không elevation**: `InstallLayoutOrTip` (input.dll), format `0x{lang:04X}:{CLSID}{ProfileGUID}` | ✓ (2026-09-27) | `tsf-registration-spike.md` — thứ tự **bắt buộc**: Install → EnableLanguageProfileByDefault → Activate |
| C2 | TIP register: CLSID ghi **HKCU** OK không admin; `Register`/`AddLanguageProfile`/`RegisterCategory` ghi **HKLM** → cần elevation | ✓ | ibid |
| C3 | Install + readback "VietIME": registry `0x0409` + `0x042A` `Enable=1` | ✓ (2026-09-27) | WIN-003 acceptance · `tsf-spike.md` #7 |
| C4 | Test SendInput tới app **elevated**: runas → user bấm UAC + **click tay** vào app (không focus được từ medium integrity) | ✓ chứng minh UIPI: `SendInput` trả 2/2 nhưng 0 phím đến nơi (2026-09-27) | mẫu `%TEMP%\opencode\test_uipi2.ps1` + `tfield2.exe` · `tsf-spike.md` #10 · plan uiAccess `P1-4` |
| C5 | Mở Office bằng COM cho app test: `New-Object -ComObject Word.Application` → `.Visible=$true` → `.Documents.Add()`; Excel tương tự `.Workbooks.Add()` | ✓ không license dialog (2026-09-27) | WIN-004 · cleanup: `.Close(0)`/`.Quit(0)` |
| C6 | Legacy console: `conhost.exe cmd.exe` → cửa sổ class `ConsoleWindowClass` | ✓ (2026-09-27) | test R9/R10 |
| C7 | UIA (`System.Windows.Automation`): **bắt buộc** load cả `UIAutomationClient` **và** `UIAutomationTypes` | ✓ (2026-09-27) | `ControlType` nằm ở Types — thiếu → `TypeNotFound` |
| C8 | Chrome test với profile riêng: `--user-data-dir=%TEMP%\<rieng>` + `file://` fixture HTML | ✓ (2026-09-27) | launch ≥2 web content xuất hiện trong UIA; launch 1 profile mới có thể KHÔNG thấy ≥8s → `uia-spike.md` S4-3 |
| C9 | Chrome SingletonLock: spawn 2 lần cùng profile → lần 2 forward sang instance cũ, pid mới **không có window** | ✓ đã gặp (2026-09-27) | kill sạch theo profile ở ĐẦU script (B3) rồi mới spawn |
| C10 | Spike UIA reproduce từ repo: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\uia-probe\{controltype_ids,probe_conditions,uia_spike}.ps1` | ✓ **3/3 chạy OK** (2026-09-27; run 2 đầy đủ = `uia-spike.md` §1.1) | output `%TEMP%\uia_probe_conditions_out.txt` / `%TEMP%\uia_spike_out.txt`; Office+Chrome tự mở, tự dọn sau khi chạy |

## D. Môi trường máy dev (snapshot 2026-09-27) — khỏi kiểm tra lại

| Thành phần | Trạng thái |
|---|---|
| Office | ✓ `C:\Program Files\Microsoft Office\root\Office16\{WINWORD,EXCEL}.EXE` |
| Chrome | ✓ `C:\Program Files\Google\Chrome\Application\chrome.exe` · Firefox ✓ `C:\Program Files\Mozilla Firefox\firefox.exe` |
| Windows Terminal | ✓ (class `CASCADIA_HOSTING_WINDOW_CLASS`) |
| VS Code | ✗ **không cài** (0 process, 2 path chuẩn không tồn tại) → test R7/Electron dùng app khác (ChatGPT/OpenCode chỉ expose caption + Pane — xem `uia-spike.md`) |
| KeePassXC | ✗ chưa cài → test password native dùng form Win32 `ES_PASSWORD` (`form_native_pw.ps1`) |
| gh CLI | ✓ (A1) |
| TIP "VietIME" | **ĐANG đăng ký & là default input** (spike WIN-002/003) — gõ mọi nơi ra "được"; cleanup cần xác nhận user: `tsf-min-register uninstall` + reset Assemblies Default/Profile |
| Log spike | `%LOCALAPPDATA%\VietIME\logs\tsf-min.log`, `tsf-register.log` (chỉ vk/HRESULT — S2) |

## E. Fixture / app test sẵn có (không dựng lại)

| Vật phẩm | Đường dẫn | Dùng cho |
|---|---|---|
| App ghi text ra file | `%TEMP%\opencode\tfield.exe`, `tfield2.exe` (+ `make_tfield2.ps1`) | test SendInput/UIPI không cần app thật |
| Fixture HTML (input text/password/search, textarea, contenteditable) | `%TEMP%\opencode\uia_fixture.html` | spike UIA / field-detect |
| Script UIA (spike + diagnostic) | `%TEMP%\opencode\uia_spike.ps1`, `uia_diag*.ps1`, `uia_timing.ps1` | đo latency, rule R1–R10 |
| Script hook (đang chờ chạy) | `%TEMP%\opencode\hook_spike_local.ps1` | WIN-005 local |
| Script test gửi phím có focus-verify | `%TEMP%\opencode\verify_sendinput*.ps1`, `run_uipi.ps1` | test B-pattern |

> Spike scripts trong `%TEMP%` là evidence tạm — khi đóng spike sẽ copy bản portable vào
> `spikes/uia-probe/` (kèm SPDX header). Khi đó cập nhật bảng E.

---

**Cách thêm entry:** đánh số tiếp (A5, B6, C10, …); thành công mới ghi; không sửa/sửa đổi entry cũ.
