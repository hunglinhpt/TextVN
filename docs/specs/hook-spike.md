# Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

> Task: `WIN-005` (`../20-windows/P1-6-TASKS.md`) · Risk: **RW5** (`../20-windows/P1-0-MASTER-PLAN.md`, `../20-windows/P1-5-test-plan.md §1/§5`)
> Spike code: `../../spikes/hook-probe/` · Workflow: `../../.github/workflows/hook-spike.yml` (dispatch-only)
> Ngày chạy: 2026-09-27 · Local: Windows 11 x64 session 1 · GHA: `windows-latest` (Server 2022) session 2
> **Acceptance:** ✅ "GHA có session desktop? SendInput có bị chặn?" — §1 · "quyết định nightly ở đâu" — §3.

**Tạm kết (RW5):**

1. **GHA CÓ desktop session** — `WTSGetActiveConsoleSessionId=2`, `UserInteractive=True`, tạo cửa sổ thấy được, `SetForegroundWindow` cross-process OK → **hook + window/focus không bị RW5 chặn**.
2. **SendInput KHÔNG bị chặn** trên runner — 2/2 cả self lẫn cross-process, `WH_KEYBOARD_LL` nhận đủ (vk đúng), reinstall 10/10.
3. **Nhưng UIA readback BỊ DEGRADED trên runner** — tree notepad chỉ **2 phần tử** (`ctIds=[50033]` = Pane, `cls='Edit'`), **không có `ValuePattern`/`TextPattern`** → **không đọc được text** (cần cho appcomptest verify).
4. → **Quyết định: §3.**

Log: `%TEMP%\hook_probe_out.txt` (GHA: artifact `hook-probe-out`) — chỉ ghi vk/số lần/độ dài (**không ghi text người dùng**, S2 Handbook).

## 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA)

| # | Check | Local (Win11, sess 1) | GHA `windows-latest` (sess 2) | Kết luận |
|---|---|---|---|---|
| 1 | Session desktop | `WTS=1`, `UserInteractive=True`, desktop class **có** | `WTS=2` (**session console thật**), `UserInteractive=True`, desktop class **có** | ✅ GHA có desktop |
| 2 | `GetForegroundWindow` | nonzero ✓ | nonzero ✓ | ✅ |
| 3 | `SetWindowsHookExW(WH_KEYBOARD_LL)` | ✓ 5.4ms | ✓ 6–7.3ms | ✅ (lastErr=183 = stale, hook nonzero → benign) |
| 4 | Focus cửa sổ tự tạo | `fg = hwnd` ✓ | `fg = hwnd` ✓ | ✅ |
| 5 | **SendInput (self)** | **2/2, hook +2, vk 0x41×2** | **2/2, hook +2, vk 0x41×2** (`err=183` stale) | ✅ không bị chặn |
| 6 | UIA `FocusedElement` (in-proc) | 5.0ms | 4.1–5.0ms | ✅ (controlType=null do Form không có control — cùng 2 nơi) |
| 7 | Mở notepad, tìm window | window tìm được (**pid khác launcher** — packaged) | window tìm được, **cùng session 2** | ⚠️ S5-5 |
| 8 | UIA `FromHandle(hwnd)` | 113–258ms | **466–1246ms** | ⚠️ GHA chậm (S5-4) |
| 9 | Find element chain `Edit→Document→ClassName` | → **`Document(50030)`** (`cls=RichEditD2DPT`) | → **`ClassName=Edit(ctId=50033)`** (Pane!) | ⚠️ S5-4 |
| 10 | UIA tree notepad | đầy đủ (40 phần tử, Win11 XAML Notepad) | **count=2, `ctIds=[50033]`**: `cls='Edit' aId='15'`, `cls='msctls_statusbar32' aId='1025'` | ❌ degraded (S5-4) |
| 11 | **Đọc text qua UIA** (`ValuePattern`/`TextPattern`) | ✅ `len0=2 → len1=3, injectNhan=True` (delta) | ❌ **`len=-2` = không pattern nào** | ❌ **GHA không đọc được text** |
| 12 | **SendInput vào notepad (cross-process)** | 2/2, hook +2 | **2/2, hook +2** | ✅ inject chạy trên GHA |
| 13 | Reinstall hook ×10 (leak) | 10/10 | 10/10 | ✅ |

**Evidence:** local = `%TEMP%\hook_probe_out.txt` · GHA = [run 36291058821](https://github.com/hunglinhpt/TextVN/actions/runs/36291058821) (cơ bản), [run 36291164891](https://github.com/hunglinhpt/TextVN/actions/runs/36291164891) (tree dump), [run 36291348415](https://github.com/hunglinhpt/TextVN/actions/runs/36291348415) (chain + pattern) — artifact `hook-probe-out`.

## 2. Findings `S5-{n}` (không xóa)

| ID | Finding | Evidence | Ảnh hưởng |
|---|---|---|---|
| S5-1 | **GHA `windows-latest` có desktop session tương tác** (`session=2` = console active, không phải session 0), tạo cửa sổ/focus/SendInput đều chạy | §1 #1–5, #12 | RW5 **không chặn** hook/inject/window tests → subset hook test đặt trên GHA được |
| S5-2 | **`SendInput` không bị chặn trên runner** (cùng session medium→medium, `ret=2/2`); `WH_KEYBOARD_LL` nhận đủ key injected (hook +2, vk 0x41) | §1 #5, #12 | WIN-040 hook test + inject smoke chạy được trên GHA |
| S5-3 | **`WH_KEYBOARD_LL` cần message pump** trên thread cài hook (A12); `err=183` sau thành công = stale error, bỏ qua khi `ret` đúng | common-errors A12 | Đúng kiến trúc `P1-2 §2` (GetMessage loop) |
| S5-4 | **UIA trên runner DEGRADED với app Win32 legacy** — notepad tree = **2 phần tử**, editor expose `Pane(50033)` (không `Edit`/`Document`), **không `ValuePattern`/`TextPattern`**, `FromHandle` 0.5–1.2s | §1 #8–11, tree dump | ❌ **appcomptest readback không chạy đủ trên GHA** → §3 |
| S5-5 | **Win11 Notepad = packaged** — `Start-Process -PassThru` trả launcher **không có window** (window thuộc pid khác, launcher có thể exit sớm → property null) | probe fix (lần chạy hỏng trước) | Harness phải quét `Get-Process` tìm `MainWindowHandle`, không tin pid launcher |
| S5-6 | **Win11 Notepad editor = `Document(50030)`** (`cls=RichEditD2DPT`, có Value+TextPattern) — **không** `Edit`; Notepad legacy/GHA = Pane(50033) `cls='Edit'` | local diag `diag_notepad_uia` + §1 #9 | Field-detect R6/R8 phải chain nhiều điều kiện (Edit→Document→ClassName) — đồng nhất hướng `uia-spike.md` S4-5/S4-6 |
| S5-7 | **Notepad có session restore** — nội dung cũ (kí tự inject lần chạy trước) vẫn còn khi mở lại → **so sánh absolute sai** | local `len0=1`/`len0=2` các lần | Harness verify theo **delta** (`len1 = len0 + n`), không so chuỗi absolute |
| S5-8 | `AutomationElement.FocusedElement` in-proc trên Form rỗng trả null (cùng 2 nơi) | §1 #6 | Không ảnh hưởng — worker thật query cross-process focused element (WIN-004 đã đo) |

## 3. Quyết định nightly (acceptance `P1-5 §1/§5`)

| Câu hỏi RW5 | Trả lời | Nền tảng |
|---|---|---|
| GHA có session desktop? | **CÓ** (session 2, interactive, cửa sổ/focus/SendInput OK) | S5-1 |
| SendInput có bị chặn? | **KHÔNG** (2/2, hook nhận đủ) | S5-2 |
| UIA đọc/text verify trên GHA? | **KHÔNG ĐỦ** — tree degraded + mất pattern (notepad/Win32 legacy) | S5-4 |

**Quyết định:**

1. **Nightly `appcomptest` (UIA harness, verify text): chạy LOCAL (máy dev/self-hosted)** — đánh dấu rõ trong report "chạy local, không gate PR" theo `P1-5 §5`. **Không đặt readback test trên GHA `windows-latest`.**
2. **Subset GHA được phép:** các test **không cần đọc text** — hook install/leak (reinstall ×10), SendInput smoke, spawn/focus window, session diagnostics (workflow `hook-spike.yml` tái dùng được, dispatch tay).
3. **Cửa nâng cấp:** trước khi chốt cứng "GHA không UIA", follow-up 1 lần với target **XAML/WinUI hoặc app có UIA provider in-proc rõ ràng** (không phải notepad legacy) — nếu pattern đọc được → mở lại tính năng readback trên GHA (ghi `P1-5 §1` cập nhật RW5 = "partial").
4. Kết quả này dùng chung cho **WIN-007** (RW5 verify ghi ở `P1-0`).

## 4. Reproduce

```powershell
# Local (Windows, ~15s — mo/closed notepad rieng, khong dot cua nguoi dung)
powershell -NoProfile -ExecutionPolicy Bypass -File spikes\hook-probe\hook_probe.ps1
# output: %TEMP%\hook_probe_out.txt
```

```bash
# GHA (can push len main truoc vi workflow doc script tu branch)
gh workflow run hook-spike.yml
gh run watch --exit-status   # xem log hoac artifact hook-probe-out
```

Yêu cầu: Windows có UIA + WinForms (PowerShell 5.1). Script tự dọn notepad mới mở (giữ notepad có sẵn của người dùng), S2: không log text.

**Điều kiện exit (`WIN-005`):**

- [x] `hook-spike.md` trả lời: **GHA có session desktop?** → CÓ (§1 #1–2) · **SendInput bị chặn?** → KHÔNG (§1 #5, #12).
- [x] **Quyết định nightly ở đâu** → §3 (local primary, GHA subset, cửa nâng cấp).
- [x] Kết quả RW5 ghi nhận để `P1-0 §RW5` + `P1-5 §1` cập nhật (báo agent phụ trách file chung trước khi sửa — G11).
- [x] Findings S5-1…S5-8 (không xóa); lỗi script mới vào `win-test-common-errors.md` (A11–A13).
- [x] Local + 3 run GHA đều xanh (evidence link §1); script portable, SPDX, ASCII.
