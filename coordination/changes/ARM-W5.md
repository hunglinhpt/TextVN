# ARM-W5 — E2E ma trận tiến trình (gõ TSF thật đa kiến trúc)

> Agent: antigravity-gemini-37c7c6bc · 2026-11-28 · P0-7 Luồng W · Dep: ARM-W3 (done).

## VI

### Mục tiêu (Acceptance P0-7)
Gõ TSF thật xanh ở: **Notepad trên ARM64**, **một app x64 giả lập**, **một app x86**,
**sandbox_e2e ARM64**.

### Làm gì
1. **`tools/win/e2e-editor.c`** (mới) — bộ sửa Win32 tối giản (1 cửa sổ + control
   `EDIT` chuẩn phủ client) để biên dịch **cùng một nguồn cho mọi kiến trúc**:
   `cl /Fe:e2e-editor-x64.exe …` (chạy được dưới giả lập x64 trên máy ARM64) và
   `…-x86.exe` (best-effort — runner có thể thiếu toolset x86, thiếu thì bỏ qua cột
   đó có cảnh báo). Class `Edit` chuẩn để `EditOf` của test-typing tìm thấy ngay.
   Đã biên dịch thử x64 trên máy dev — BUILD_OK.
2. **`installer/windows/tests/test-typing.ps1`**: tham số `-ExtraApps` + env
   `TEXTVN_E2E_EXTRA_EDITORS` (danh sách exe phân cách `;`) — mỗi app bổ sung đi qua
   **toàn bộ ma trận case** hiện có (các case gõ, Ctrl+Shift EN/VN, Caps Lock) trong
   vòng `foreach ($a in $apps)`; test-portable không cần đổi (env tự kế thừa).
3. **`.github/workflows/ci-windows.yml`** (job windows-11-arm): bước
   `ARM-W5 build e2e editors` — vswhere → `vcvarsall x64` build chắc chắn; x86
   best-effort (`|| echo WARN`); ghép `TEXTVN_E2E_EXTRA_EDITORS` từ các exe TỒN TẠI
   vào `GITHUB_ENV`. Ý nghĩa: app x64/x86 nạp **DLL đúng kiến trúc của nó qua
   forwarder ARM64X** — chính là bất biến ARM-W2 ("đột biến: đăng ký DLL ARM64 thay
   forwarder ⇒ e2e app x64 đỏ") được gate tự động.
4. **sandbox_e2e ARM64** — bằng chứng trực tiếp từ log job windows-11-arm
   (run 46111640219): `test sandbox_e2e::sandbox_appcontainer_end_to_end ... ok`
   + `child_in_appcontainer ... ok` (chạy native trong `cargo test --workspace`).

### Kết hợp với E89
Thang focus 4 mức (E89v2: SW_RESTORE + chặn rect ngoài màn hình + verify cursor)
áp cho **mọi app** trong ma trận, gồm editors x64/x86.

### Bằng chứng / đột biến
- `Parser::ParseFile` + `ascii-ps1` (G7) xanh cho cả 2 tệp .ps1/.c sửa thêm.
- YAML `ci-windows.yml` parse OK.
- Build thật: `e2e-editor-x64.exe` BUILD_OK trên máy dev (cl 14.51 Hostx64/x64).
- Đột biến (kế hoạch, chạy trên runner): đăng ký textvn-tsf-arm64.dll thay forwarder
  ⇒ editor x64 đỏ (không nạp được TIP x64). Hành vi này là gate tự nhiên của bước mới;
  xác nhận khi run CI đầu tiên xanh (sẽ ghi run id vào đây).

## EN

### Goal (P0-7 acceptance)
Real TSF typing must pass on: **Notepad on ARM64**, **an emulated x64 app**, **an x86
app**, **sandbox_e2e ARM64**.

### What
1. **`tools/win/e2e-editor.c`** (new) — minimal Win32 editor (one window + a standard
   `EDIT` control filling the client) compiled **from one source for every
   architecture**: x64 (runs under emulation on ARM64) and x86 (best-effort — the
   runner may lack the x86 toolset; that column is skipped with a warning). The
   standard `Edit` class is found immediately by test-typing's `EditOf`. Built
   locally for x64 — BUILD_OK.
2. **`installer/windows/tests/test-typing.ps1`**: `-ExtraApps` parameter + env
   `TEXTVN_E2E_EXTRA_EDITORS` (`;`-separated exes) — every extra app goes through the
   **full existing case matrix** (typing cases, Ctrl+Shift EN/VN, Caps Lock) inside
   the `foreach ($a in $apps)` loop; test-portable needs no change (env inherits).
3. **`.github/workflows/ci-windows.yml`** (windows-11-arm job): new
   `ARM-W5 build e2e editors` step — vswhere → `vcvarsall x64` mandatory build; x86
   best-effort (`|| echo WARN`); `TEXTVN_E2E_EXTRA_EDITORS` assembled from existing
   exes into `GITHUB_ENV`. Meaning: x64/x86 apps load the **DLL matching their own
   architecture through the ARM64X forwarder** — ARM-W2's invariant ("mutate:
   register the ARM64 DLL instead of the forwarder ⇒ x64-app e2e goes red") becomes
   an automatic gate.
4. **sandbox_e2e ARM64** — direct evidence from the windows-11-arm job log
   (run 46111640219): `test sandbox_e2e::sandbox_appcontainer_end_to_end ... ok`
   + `child_in_appcontainer ... ok` (native inside `cargo test --workspace`).

### With E89
The 4-rung focus ladder (E89v2: SW_RESTORE + off-screen rect guard + cursor verify)
applies to **every app** in the matrix, including the x64/x86 editors.

### Evidence / mutations
- `Parser::ParseFile` + `ascii-ps1` (G7) green for both edited files.
- `ci-windows.yml` YAML parse OK.
- Real build: `e2e-editor-x64.exe` BUILD_OK on the dev machine (cl 14.51
  Hostx64/x64).
- Mutation (planned, runs on the runner): registering textvn-tsf-arm64.dll instead of
  the forwarder makes the x64 editor red (cannot load the x64 TIP). The new step is
  the natural gate; the first green CI run will be recorded here.
