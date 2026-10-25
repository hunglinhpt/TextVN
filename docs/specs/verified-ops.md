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
| A5 | GitHub Action **`repo-hygiene`** — 8 check "sạch" repo (`00-WORKFLOW.md §12`) | ✓ run **đầu tiên = success** (2026-09-27, [run 36289296482](https://github.com/hunglinhpt/TextVN/actions/runs/36289296482), commit `a4d80e4`; lúc đó 7 check) | chạy local trước khi push: 8/8 pass (git-bash giả định + `python .github/scripts/check_doc_links.py` + `python .github/scripts/check_no_injection_apis.py`) — công thức "pre-check local → push → xem run" |

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
| B8 | `cargo run -p textvn-cli -- verify` | ✓ FFI ABI 4 tầng + 11 export functions khớp 100% (2026-09-27) | `textvn_ffi.h` đồng bộ tuyệt đối Rust ABI |
| B9 | `cargo clippy --workspace --all-targets -- -D warnings` → **phải thêm `-j 2`** trên máy dev này | ✓ chạy `-j 2` → 0 warning (2026-10-20); **không** `-j 2` → fail với `os error 1455: The paging file is too small` (mmap `libcore.rmeta` thất bại) | paging file máy dev quá nhỏ cho clippy full-parallel; dùng `-j 2` cho clippy **và** `cargo test --workspace -j 2` (Fassist-37) |
| B10 | `cargo test --workspace` → thêm `-j 2` khi song song nhiều crate | ✓ 0 fail với `-j 2` (2026-10-20) | đồng nguyên nhân B9; CI không bị ảnh hưởng (runner đủ RAM) |
| B9 | `cargo run -p textvn-cli -- replay corpus/shared corpus/win` | ✓ 100/100 test cases pass (2026-09-27); **134/134** sau khi corpus tăng (2026-10-23) | test suite cho B1-B9, secure context, hook, tsf preedit |
| B10 | `cargo run -p textvn-bench` | ✓ ime_key p50=5.7µs (<500µs), resolve p50=57ns (<2ms) (2026-09-27) | đạt budget performance vượt trội |
| B11 | `cargo build -p textvn-win-tsf` | ✓ Sinh `target/debug/textvn_win_tsf.dll` (~1MB) (2026-09-27) | TSF COM in-process DLL hoàn chỉnh (WIN-010) |
| B12 | `cargo test -p textvn-win-tsf -p textvn-win-hook -p textvn-tray` | ✓ 29/29 Windows platform tests pass 100% (2026-09-27) | Bộ 3 thành phần Windows (TSF 15, Hook 5, Tray 9) hoạt động đồng bộ |
| B13 | `cargo run -p textvn-cli -- doctor --export` | ✓ Xuất file ZIP PKZIP stored hợp lệ, pass grep test S2 không chứa text thô (2026-09-27) | WIN-058 diagnostics report xuất zip 4 file: version, config.redacted, system_info, tsf_tail |
| B14 | `cargo test -p textvn-tray` | ✓ 13/13 tests pass 100% (2026-09-27) | WIN-052 SettingsController debounce 300ms, 6 tabs model, hotkey validation, parity-checklist.md |
| B15 | `cargo run -p textvn-cli -j 2 -- assist check` (7 kịch bản Acceptance DoD AS-030/031/050) | ✓ 7/7 kịch bản pass 100% (2026-10-21) | Kiểm chứng câu sạch ("Hôm nay trời đẹp." -> 0 lỗi, 0 sửa); lặp từ ("tôi tôi đi học." -> Mode 1 hiện gợi ý, Mode 2 sửa "tôi đi học."); thừa khoảng trắng ("đi học ." -> sửa "đi học."); chữ thường đầu câu ("hôm nay tôi đi học." -> sửa "Hôm nay tôi đi học."); Undo 1 bước 5s; Hết hạn Undo >5s; Precondition D12 (chặn bôi đen/lệch caret) |
| B16 | `cargo test -p textvn-win-tsf -p textvn-assist -j 2` | ✓ 75/75 TSF + 51/51 Assist unit & acceptance tests pass (2026-10-21) | `acceptance_dod_scenarios_1_to_7` và `scenario_1..4` xác nhận Mode 1 / Mode 2 / Precondition D12 / Undo 5s hoạt động đúng chuẩn |
| B17 | **Đo RSS in-proc của TIP (F61)**: build release sang `CARGO_TARGET_DIR` tạm → `textvn-cli register --scope user --dll <dll mới>` → `notepad.exe -m <scratch>` (Notepad Store tạo **process mới** đúng) → `powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\measure_assist_rss.ps1 -ProcessId <pid> -DurationSec 40 -IntervalMs 200 -IdleSec 8 -OutFile <json>` → đóng **đúng** process đó → `register --dll <đường gốc>` khôi phục | ✓ 2026-10-22: DLL image RSS idle **560 KiB** → gõ+assist **744 KiB p95** (budget ≤ +5 MB / +20 MB); 145 mẫu/200 ms; DLL 0.2.34.0 SHA-256 `6D2183…366`; config `%APPDATA%` không đổi (hash trước/sau giữ nguyên) | Số đầy đủ + giới hạn (heap không tính vào `dll_ws`; process WS là **giới hạn trên**) ở `docs/50-assist/P4-11-MEASUREMENTS.md`. `typing_verified=false` do so khớp **một lần** quá sớm — bằng chứng thay thế: tiêu đề Notepad hiện câu Telex đã biên dịch ("tôi tôi …" ⇒ L1 lặp từ ⇒ Mode 1 phải hiện). Không dùng Notepad/phiên khác của người dùng (máy có sẵn 2 phiên) |
| B18 | **Chạy test C của `linux-common` trên Windows host** (không cần CMake/POSIX): `cargo build -p textvn-ffi` → `clang -std=c11 -Wall -Wextra -Werror -Iadapters/linux-common/include -Iffi/include <test>.c <src>.c target\debug\textvn_ffi.lib "<WinSDK>\um\x64\WS2_32.Lib" "<WinSDK>\um\x64\ntdll.lib" "<WinSDK>\um\x64\UserEnv.Lib"` → chạy exe (cần `target\debug` trong `PATH` để thấy `textvn_ffi.dll`) | ✓ 2026-10-22: `test_assist_result` **PASS** (D16 lọc L6 · Q-14 payload `AssistReport` escape/độ dài/fail-safe) | Chỉ áp cho file test **không** dùng POSIX (socket/`unlink`); test `test_compose`/`test_ipc_client` vẫn phải chạy trên CI Linux. Thiếu `UserEnv.Lib` ⇒ `LNK2019 __imp_GetUserProfileDirectoryW` (std `home_dir`) |
| B19 | `python tools/assist/gen_corpus_en_tatoeba.py` (sinh `corpus/assist/en_real_clean.ass` từ Tatoeba eng_sentences.tsv.bz2, ~25 MB tải qua mạng) | ✓ 2026-10-24: pool 234.292 câu → mẫu seed `20261023` 300 câu → bỏ 5 ID sau rà soát tay → **295 câu**; chạy lại lần 2 → file **byte-identical** (sha256 khớp) | Cache trong `tools/assist/.cache/` (git-ignored) ⇒ chạy lại offline; xuất LF cố định (`newline="\n"`); loại ID diễn ra **sau** chọn mẫu nên thêm `EXCLUDE_IDS` không resample. Lưu ý: corpus EN mới chỉ xanh khi engine EN v1 implement (xem header `en_basic.ass`) |


| B20 | Text Prediction Song ngữ Anh - Việt (P6): prefix completion + next-phrase prediction + tab cycling | ✓ 165 core + 6 integration tests pass 100%, preflight 23/23 PASS, p50=700ns (2026-10-25) | Tra cứu từ ghép VN, n-gram cụm từ tiếp theo VN/EN, không I/O, an toàn S9 Shift+Tab passthrough, G8 zero text logging |
| B21 | Trigger Ctrl+Space cho cả Text Prediction và Text Correct (Trợ lý sửa lỗi từ và câu) | ✓ 23/23 preflight PASS, 96/96 TSF tests, 9/9 text_prediction tests, p50=687ns (2026-10-25) | Trigger song song với Tab: hoàn tất tiền tố, dự đoán cụm tiếp theo, duyệt vòng ứng viên; khi không khớp hoặc có lỗi/bôi đen thì gọi on-demand assist check hiện overlay sửa lỗi |
| B22 | **Compile thử `installer/windows/TextVN-setup.iss` không cần build release**: tạo thư mục giả chứa 4 file 1 byte (`TextVN.exe`, `textvn-cli.exe`, `textvn_win_tsf.dll`, `textvn-tsf-x86.dll`) trong scratchpad → `& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" /Q "/DMyAppVersion=0.0.1" "/DTargetDir=<dir giả>" "/O<out>" installer\windows\TextVN-setup.iss` (thêm `"/DMachineInstall=1" "/DOutputSuffix=-machine"` cho biến thể máy) | ✓ 2026-10-25: exit 0 cả 2 biến thể (R2-124) | Chỉ kiểm cú pháp Pascal Script/section — **không chạy** setup.exe giả; hành vi cài thật kiểm trên `ci-windows` (`test-installer.ps1`) |
| B23 | **Chứng minh test hồi quy bắt đúng lỗi**: `git stash push -- <file nguồn đã sửa> <data>` (giữ file test) → `cargo test -p <crate> --test <file>` phải FAIL → `git stash pop` | ✓ 2026-10-25: `text_prediction` 9/16 FAIL trên mã cũ, 16/16 PASS trên mã mới (R2-120/R2-121) | Nhanh hơn checkout commit cũ; nhớ `git status` sau `pop` |
| B24 | **E2E trợ lý + dự đoán trên ứng dụng THẬT**: `$env:CARGO_TARGET_DIR=<tạm>; cargo build --release -p textvn-win-tsf -p textvn-cli` → `powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\e2e-assist-p6.ps1 -DevDir <tạm>\release` (`-Only <tên case>` chạy riêng 1 case Notepad; trace `%TEMP%\textvn-e2e-trace.log`) | ✓ 2026-10-26: **28/28 PASS** (Notepad Win11 19, Word 6, Excel 3) | Cần bản cài Program Files (để khôi phục đăng ký ở `finally`) + Office; chiếm bàn phím ~3 phút; tự bật chế độ V rồi trả lại; tạm tắt AutoCorrect của Word rồi trả lại. Xoá text giữa các case bằng `WM_SETTEXT` engine không thấy → script gõ phím End sau mỗi lần xoá (không thì buffer câu dính case trước). Chụp panel thật để soát giao diện: `System.Drawing.Graphics.CopyFromScreen` theo `GetWindowRect` của cửa sổ lớp `TextVNAssistOverlay` (SetProcessDPIAware trước) |

## C. Windows platform (chi tiết → spike specs)

| # | Thao tác | Kết quả | Chi tiết |
|---|---|---|---|
| C1 | Đăng ký TIP **không elevation**: `InstallLayoutOrTip` (input.dll), format `0x{lang:04X}:{CLSID}{ProfileGUID}` | ✓ (2026-09-27) | `tsf-registration-spike.md` — thứ tự **bắt buộc**: Install → EnableLanguageProfileByDefault → Activate |
| C2 | TIP register: CLSID ghi **HKCU** OK không admin; `Register`/`AddLanguageProfile`/`RegisterCategory` ghi **HKLM** → cần elevation | ✓ | ibid |
| C3 | Install + readback "TextVN": registry `0x0409` + `0x042A` `Enable=1` | ✓ (2026-09-27) | WIN-003 acceptance · `tsf-spike.md` #7 |
| C4 | Test SendInput tới app **elevated**: runas → user bấm UAC + **click tay** vào app (không focus được từ medium integrity) | ✓ chứng minh UIPI: `SendInput` trả 2/2 nhưng 0 phím đến nơi (2026-09-27) | mẫu `%TEMP%\opencode\test_uipi2.ps1` + `tfield2.exe` · `tsf-spike.md` #10 · plan uiAccess `P1-4` |
| C5 | Mở Office bằng COM cho app test: `New-Object -ComObject Word.Application` → `.Visible=$true` → `.Documents.Add()`; Excel tương tự `.Workbooks.Add()` | ✓ không license dialog (2026-09-27) | WIN-004 · cleanup: `.Close(0)`/`.Quit(0)` |
| C6 | Legacy console: `conhost.exe cmd.exe` → cửa sổ class `ConsoleWindowClass` | ✓ (2026-09-27) | test R9/R10 |
| C7 | UIA (`System.Windows.Automation`): **bắt buộc** load cả `UIAutomationClient` **và** `UIAutomationTypes` | ✓ (2026-09-27) | `ControlType` nằm ở Types — thiếu → `TypeNotFound` |
| C8 | Chrome test với profile riêng: `--user-data-dir=%TEMP%\<rieng>` + `file://` fixture HTML | ✓ (2026-09-27) | launch ≥2 web content xuất hiện trong UIA; launch 1 profile mới có thể KHÔNG thấy ≥8s → `uia-spike.md` S4-3 |
| C9 | Chrome SingletonLock: spawn 2 lần cùng profile → lần 2 forward sang instance cũ, pid mới **không có window** | ✓ đã gặp (2026-09-27) | kill sạch theo profile ở ĐẦU script (B3) rồi mới spawn |
| C10 | Spike UIA reproduce từ repo: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\uia-probe\{controltype_ids,probe_conditions,uia_spike}.ps1` | ✓ **3/3 chạy OK** (2026-09-27; run 2 đầy đủ = `uia-spike.md` §1.1) | output `%TEMP%\uia_probe_conditions_out.txt` / `%TEMP%\uia_spike_out.txt`; Office+Chrome tự mở, tự dọn sau khi chạy |
| C11 | Hook spike: local `spikes\hook-probe\hook_probe.ps1` + GHA `gh workflow run hook-spike.yml` (dispatch, artifact `hook-probe-out`) | ✓ local PASS + **3 run GHA xanh** (2026-09-27: 36291058821, 36291164891, 36291348415) | kết luận RW5 = `hook-spike.md` §3; script tự dọn notepad mới mở, giữ notepad sẵn có |
| C12 | `textvn-tray.exe` & IPC Named Pipe server (`\\.\pipe\textvn-ipc-v1`) | ✓ binary 1.15MB, 9 mục menu, autostart registry HKCU (2026-09-27) | WIN-050, WIN-051, WIN-053 hoàn thiện, Single-instance Mutex `Local\TextVNTray` |
| C13 | Inno Setup 6 script (`installer/windows/textvn-setup.iss`) & app manifest (`app.manifest`) | ✓ Verified 4 binary artifacts, per-user/all-users support, Rule S9 uninstall preserve (2026-09-27) | WIN-054, WIN-055 hoàn thiện |
| C14 | Harness targets JSON 12 app: local `powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\verify_targets.ps1`; CI `gh workflow run targets-verify.yml` | ✓ **12/12 OK, 0 MISS** local (2026-09-27); GHA run **36297278626** = 3/3 job xanh (`upgrade-cycle` + `cross-image` 2022/2025 + firefox) | `tools/appcomptest/targets/README.md` (F6-9…F6-13, schema `launch.profile`); check: `tools\win\check_targets.ps1` |

## D. Môi trường máy dev (snapshot 2026-09-27) — khỏi kiểm tra lại

| Thành phần | Trạng thái |
|---|---|
| Office | ✓ `C:\Program Files\Microsoft Office\root\Office16\{WINWORD,EXCEL}.EXE` |
| Chrome | ✓ `C:\Program Files\Google\Chrome\Application\chrome.exe` · Firefox ✓ `C:\Program Files\Mozilla Firefox\firefox.exe` |
| Windows Terminal | ✓ (class `CASCADIA_HOSTING_WINDOW_CLASS`) |
| VS Code | ✗ **không cài** (0 process, 2 path chuẩn không tồn tại) → test R7/Electron dùng app khác (ChatGPT/OpenCode chỉ expose caption + Pane — xem `uia-spike.md`) |
| KeePassXC | ✗ chưa cài → test password native dùng form Win32 `ES_PASSWORD` (`form_native_pw.ps1`) |
| gh CLI | ✓ (A1) |
| TIP "TextVN" | **ĐANG đăng ký & là default input** (spike WIN-002/003) — gõ mọi nơi ra "được"; cleanup cần xác nhận user: `tsf-min-register uninstall` + reset Assemblies Default/Profile |
| Log spike | `%LOCALAPPDATA%\TextVN\logs\tsf-min.log`, `tsf-register.log` (chỉ vk/HRESULT — S2) |

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
