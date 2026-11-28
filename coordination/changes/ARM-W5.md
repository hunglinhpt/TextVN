# ARM-W5 — E2E ma trận tiến trình (gõ TSF thật đa kiến trúc)

> Agent: antigravity-gemini-37c7c6bc · 2026-11-28 · P0-7 Luồng W · Dep: ARM-W3 (done).

## VI

> **Trạng thái: Đạt một phần — giới hạn runner (E89).**
> Gõ TSF thật trên `e2e-editor-x64.exe` + `e2e-editor-x86.exe` + Notepad chỉ chạy thật được trên `windows-2022` / `windows-2025` (máy chủ x64, dùng DLL x64/x86 trực tiếp, KHÔNG có forwarder ARM64X). Trên `windows-11-arm`, `sandbox_e2e` chạy native và xanh, còn `test-typing.ps1` tự SKIP toàn bộ bước gõ GUI do giới hạn runner ARM64 không route input (E89 — bằng chứng 5 vòng). Vì vậy **chưa có bằng chứng runtime trên `windows-11-arm` cho định tuyến forwarder ARM64X**, và đột biến ARM-W2 ("đăng ký DLL ARM64 thay forwarder ⇒ app x64 đỏ") được bàn giao sang **`ARM-W5b`** (probe COM `CoCreateInstance` / nạp module headless trong 3 tiến trình ARM64/x64/x86 qua `InprocServer32` thật + gate đột biến `0x800700c1`, không cần route input GUI).

### Mục tiêu (Acceptance P0-7)
Gõ TSF thật xanh ở: **Notepad trên ARM64**, **một app x64 giả lập**, **một app x86**,
**sandbox_e2e ARM64**.

### Làm gì
1. **`tools/win/e2e-editor.c`** (mới) — bộ sửa Win32 tối giản (1 cửa sổ + control
   `EDIT` chuẩn phủ client) để biên dịch **cùng một nguồn cho mọi kiến trúc**
   (`cl /Fe:e2e-editor-x64.exe …`, `…-x86.exe`). Class `Edit` chuẩn để `EditOf`
   của `test-typing` tìm thấy ngay.
2. **`installer/windows/tests/test-typing.ps1`**: tham số `-ExtraApps` + env
   `TEXTVN_E2E_EXTRA_EDITORS` (danh sách exe phân cách `;`) — mỗi app bổ sung đi qua
   **toàn bộ ma trận case** hiện có (các case gõ, Ctrl+Shift EN/VN, Caps Lock) trong
   vòng `foreach ($a in $apps)` khi host hỗ trợ route input desktop; trên
   `windows-11-arm` bước này SKIP có kiểm soát theo E89.
3. **`.github/workflows/ci-windows.yml`**:
   - Job `windows-package` (`windows-2022` + `windows-2025`): biên dịch `e2e-editor-x64.exe`
     + `e2e-editor-x86.exe` và chạy `test-typing.ps1` gõ TSF thật trên Notepad +
     editor x64 + editor x86 (xác minh hành vi Win32 EDIT x64/WOW64 x86).
   - Job `windows-11-arm`: biên dịch `e2e-editor-x64.exe` + `e2e-editor-x86.exe` (x86
     best-effort khi runner thiếu toolset x86) và truyền `TEXTVN_E2E_EXTRA_EDITORS`
     nhưng `test-typing.ps1` hiện SKIP theo E89.
4. **sandbox_e2e ARM64** — bằng chứng trực tiếp từ log job `windows-11-arm`
   (run `46111640219` / `da7462da`): `test sandbox_e2e::sandbox_appcontainer_end_to_end ... ok`
   + `child_in_appcontainer ... ok` (chạy native trong `cargo test --workspace`).

### Kết hợp với E89
Thang focus 4 mức (E89v2: SW_RESTORE + chặn rect ngoài màn hình + verify cursor)
áp cho **mọi app** trong ma trận, gồm editors x64/x86.

### Bằng chứng & giới hạn còn lại (cho `ARM-W5b`)
- **Đạt**:
  - `sandbox_e2e` native ARM64 xanh (`windows-11-arm`).
  - `test-typing.ps1` gõ TSF thật trên `[notepad]`, `[e2e-editor-x64]`, `[e2e-editor-x86]`
    xanh trên `windows-2022` và `windows-2025` (run `46127440861`, `da7462da`).
  - `Parser::ParseFile` + `ascii-ps1` (G7) + YAML parse OK.
- **Giới hạn / chuyển `ARM-W5b`**:
  - Trên `windows-11-arm`, không thể gõ GUI qua `SendInput` (E89), nên `ARM-W5` chỉ
    đạt một phần và chưa gate được đột biến forwarder ARM64X ("đăng ký
    `textvn-tsf-arm64.dll` thay forwarder ⇒ tiến trình x64 giả lập thất bại").
  - `ARM-W5b` sẽ gate bằng probe COM headless (nạp `InprocServer32` thật từ tiến
    trình ARM64, x64 giả lập và x86 + kiểm đột biến `0x800700c1`).

## EN

> **Status: Partial — runner limitation (E89).**
> Real TSF typing in `e2e-editor-x64.exe` + `e2e-editor-x86.exe` + Notepad runs only on `windows-2022` / `windows-2025` (x64 host, loading x64/x86 DLLs directly, WITHOUT the ARM64X forwarder). On `windows-11-arm`, `sandbox_e2e` runs natively and passes, while `test-typing.ps1` skips GUI input typing due to the ARM64 runner failing to route desktop input (E89 — 5 rounds of evidence). Consequently, **there is not yet runtime evidence on `windows-11-arm` for ARM64X forwarder routing**, and the ARM-W2 mutation ("register the ARM64 DLL instead of the forwarder ⇒ emulated x64 app fails") is handed off to **`ARM-W5b`** (headless COM `CoCreateInstance` / module-load probe across ARM64/x64/x86 processes via the real registered `InprocServer32` + `0x800700c1` mutation gate, requiring no GUI input routing).

### Goal (P0-7 acceptance)
Real TSF typing must pass on: **Notepad on ARM64**, **an emulated x64 app**, **an x86
app**, **sandbox_e2e ARM64**.

### What
1. **`tools/win/e2e-editor.c`** (new) — minimal Win32 editor (one window + a standard
   `EDIT` control filling the client) compiled **from one source for every
   architecture** (`e2e-editor-x64.exe`, `e2e-editor-x86.exe`). The standard `Edit`
   class is found immediately by `test-typing`'s `EditOf`.
2. **`installer/windows/tests/test-typing.ps1`**: `-ExtraApps` parameter + env
   `TEXTVN_E2E_EXTRA_EDITORS` (`;`-separated exes) — every extra app goes through the
   **full existing case matrix** (typing cases, Ctrl+Shift EN/VN, Caps Lock) inside
   the `foreach ($a in $apps)` loop when desktop input routing works; on
   `windows-11-arm` this skips cleanly per E89.
3. **`.github/workflows/ci-windows.yml`**:
   - `windows-package` jobs (`windows-2022` + `windows-2025`): build `e2e-editor-x64.exe`
     + `e2e-editor-x86.exe` and run real TSF typing against Notepad + x64 editor +
     x86 editor (verifying native x64 and WOW64 x86 Win32 EDIT handling).
   - `windows-11-arm` job: builds `e2e-editor-x64.exe` + `e2e-editor-x86.exe` (x86
     best-effort if the runner lacks the x86 toolset) and passes `TEXTVN_E2E_EXTRA_EDITORS`,
     with `test-typing.ps1` skipping under E89.
4. **sandbox_e2e ARM64** — direct evidence from the `windows-11-arm` job log
   (run `46111640219` / `da7462da`): `test sandbox_e2e::sandbox_appcontainer_end_to_end ... ok`
   + `child_in_appcontainer ... ok` (native inside `cargo test --workspace`).

### With E89
The 4-rung focus ladder (E89v2: SW_RESTORE + off-screen rect guard + cursor verify)
applies to **every app** in the matrix, including the x64/x86 editors.

### Evidence & remaining limitation (handed off to `ARM-W5b`)
- **Verified**:
  - Native ARM64 `sandbox_e2e` passes (`windows-11-arm`).
  - Real TSF typing on `[notepad]`, `[e2e-editor-x64]`, `[e2e-editor-x86]` passes on
    `windows-2022` and `windows-2025` (runs `46127440861`, `da7462da`).
  - `Parser::ParseFile` + `ascii-ps1` (G7) + YAML parse OK.
- **Limitation / `ARM-W5b` handoff**:
  - On `windows-11-arm`, GUI `SendInput` cannot be routed (E89), so `ARM-W5` is
    partial and does not yet gate the ARM64X forwarder mutation ("registering
    `textvn-tsf-arm64.dll` instead of the forwarder fails the emulated x64 process").
  - `ARM-W5b` will gate this with a headless COM probe (loading the real
    `InprocServer32` from ARM64, emulated x64, and x86 processes + `0x800700c1`
    mutation check).
