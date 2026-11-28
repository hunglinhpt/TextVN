<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-W4 — Bộ cài Inno Setup + portable ZIP Windows ARM64 (ADR-015)

## Tiếng Việt

### Tóm tắt thay đổi
- Chốt quyết định đóng gói trong [`docs/adr/ADR-015-windows-arm64-packaging.md`](../../docs/adr/ADR-015-windows-arm64-packaging.md) và cập nhật [`docs/adr/README.md`](../../docs/adr/README.md): dùng một tệp `.iss` duy nhất với `/DTargetArch=x64|arm64|multi`, mặc định phát hành gói riêng theo kiến trúc `-windows-x64` và `-windows-arm64` (giữ chế độ `multi` sẵn sàng).
- Cập nhật [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss):
  - Hỗ trợ `#ifndef TargetArch` (`x64`/`arm64`/`multi`), thiết lập `OutputBaseFilename`, `ArchitecturesAllowed`, `ArchitecturesInstallIn64BitMode`, `TargetDir` và `TargetDirArm64`.
  - Bổ sung payload 4 DLL trên ARM64 (`textvn-tsf.dll` ARM64X forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) cùng dọn `.old-*` trong `[UninstallDelete]` và `RenameLockedTsfDll()`.
- Cập nhật [`build-release.ps1`](../../build-release.ps1):
  - Thêm `-Arch x64|arm64` (mặc định `x64`, cấm kết hợp `arm64` + `-IncludeCompatibilityHook`), dựng `aarch64-pc-windows-msvc` + `x86_64-pc-windows-msvc` (`textvn-tsf-x64.dll`) + `i686-pc-windows-msvc` (`textvn-tsf-x86.dll`) + `cargo xtask build-arm64x --out textvn-tsf.dll`.
  - Kiểm `check-pe-imports` đúng `--machine` cho từng trong 6 PE, đóng gói `TextVN-portable-<v>-windows-arm64-*.zip`, dựng bộ cài `/DTargetArch=arm64`, giữ G24 dọn `target/debug/build`.
- Cập nhật [`installer/windows/portable/uninstall.ps1`](../../installer/windows/portable/uninstall.ps1), [`installer/windows/tests/test-portable.ps1`](../../installer/windows/tests/test-portable.ps1), [`installer/windows/tests/test-installer.ps1`](../../installer/windows/tests/test-installer.ps1) để gỡ sạch và kiểm đủ cả 4 DLL trên ARM64.
- Cập nhật job `windows-arm64` trong [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml) chạy `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller` cùng `test-portable.ps1` và `test-installer.ps1` trên `windows-11-arm`.

### Kiểm chứng
- PowerShell AST parser parse sạch cả 4 tệp `.ps1`; `cargo xtask ascii-ps1` PASS.
- `cargo xtask iss-tabs`, `cargo xtask check-version-sync`, `cargo xtask no-python`, `cargo xtask hygiene-docs`, `cargo xtask hygiene-apis`, `cargo xtask hygiene-secrets`, `cargo xtask regression-index`, `cargo xtask eval-leak` PASS.
- `ISCC.exe` (Inno Setup 6.7.3) biên dịch thử sạch cả 3 chế độ `/DTargetArch=x64`, `/DTargetArch=arm64` và `/DTargetArch=multi` (kèm `[Code]` `IsArm64Arch()`).
- Đóng gói & cài/gỡ e2e trên `windows-11-arm` chạy trong CI job `windows-arm64`.

---

## English

### Summary of Changes
- Recorded the packaging architecture decision in [`docs/adr/ADR-015-windows-arm64-packaging.md`](../../docs/adr/ADR-015-windows-arm64-packaging.md) and indexed it in [`docs/adr/README.md`](../../docs/adr/README.md): one shared `.iss` parameterized by `/DTargetArch=x64|arm64|multi`, shipping separate `-windows-x64` and `-windows-arm64` artifacts by default with `multi` mode supported.
- Updated [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss):
  - Added `/DTargetArch` (`x64`/`arm64`/`multi`) controlling `OutputBaseFilename`, `ArchitecturesAllowed`, `ArchitecturesInstallIn64BitMode`, `TargetDir`, and `TargetDirArm64`.
  - Added the 4-DLL ARM64 TSF payload (`textvn-tsf.dll` ARM64X forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) plus `.old-*` cleanup in `[UninstallDelete]` and `RenameLockedTsfDll()`.
- Updated [`build-release.ps1`](../../build-release.ps1):
  - Added `-Arch x64|arm64` (default `x64`, rejects legacy hook on ARM64), building `aarch64-pc-windows-msvc` + `x86_64-pc-windows-msvc` (`textvn-tsf-x64.dll`) + `i686-pc-windows-msvc` (`textvn-tsf-x86.dll`) + `cargo xtask build-arm64x --out textvn-tsf.dll`.
  - Runs `check-pe-imports` with per-binary `--machine` flags across all 6 PEs, packages `TextVN-portable-<v>-windows-arm64-*.zip`, builds `/DTargetArch=arm64` installers, and preserves G24 `target/debug/build` cleanup.
- Updated [`installer/windows/portable/uninstall.ps1`](../../installer/windows/portable/uninstall.ps1), [`installer/windows/tests/test-portable.ps1`](../../installer/windows/tests/test-portable.ps1), and [`installer/windows/tests/test-installer.ps1`](../../installer/windows/tests/test-installer.ps1) to assert and clean all 4 ARM64 TSF DLLs.
- Updated the `windows-arm64` job in [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml) to run `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller`, `test-portable.ps1`, and `test-installer.ps1` on `windows-11-arm`.

### Verification
- PowerShell AST parser checks passed on all 4 modified `.ps1` files; `cargo xtask ascii-ps1` PASSED.
- `cargo xtask iss-tabs`, `cargo xtask check-version-sync`, `cargo xtask no-python`, `cargo xtask hygiene-docs`, `cargo xtask hygiene-apis`, `cargo xtask hygiene-secrets`, `cargo xtask regression-index`, and `cargo xtask eval-leak` PASSED.
- `ISCC.exe` (Inno Setup 6.7.3) compiled all 3 modes (`/DTargetArch=x64`, `/DTargetArch=arm64`, `/DTargetArch=multi` including `[Code]` `IsArm64Arch()`) cleanly.
- Real Windows ARM64 packaging and install/uninstall E2E run in the `windows-arm64` CI job (`windows-11-arm`).

## VI — Sửa sau review (E89, 2026-11-28)

- **Review CHƯA PASS**: ci-windows job ARM64 đỏ tại bước test-portable —
  `test-typing.ps1:141 "could not focus notepad main window"`. Nguyên nhân gốc: session
  runner windows-11-arm không có foreground window (`GetForegroundWindow() = NULL`) làm
  `AttachThreadInput(0)` vô nghĩa và `SetForegroundWindow` bị từ chối theo thuật toán
  foreground-lock của Win32 — thang cũ chỉ có 1 mức.
- **Fix cả lớp lỗi (E89)** — `Ensure-Focus` dùng thang 4 mức `FocusLadder`:
  1. `AttachThreadInput` + `SetForegroundWindow` (mức cũ, nhanh);
  2. `SwitchToThisWindow` ×3 (cho session "lạnh");
  3. Alt-trick: SendInput VK_MENU rồi `SetForegroundWindow` (mở khoá foreground-lock);
  4. **Click chuột thật** vào tâm cửa sổ chính (`SendInput MOUSEEVENTF_ABSOLUTE`, quy
     đổi 0–65535 theo GetSystemMetrics) — click đặt foreground + focus edit cùng lúc,
     không bao giờ chết trên runner có desktop tương tác.
  Mỗi mức đều VERIFY `GetForegroundWindow() == main`; fail hết mới throw kèm dump chẩn
  đoán E89 (giá trị fg/main hex — không có nội dung người dùng, G8).
- Bằng chứng: `Parser::ParseFile` xanh; `cargo xtask ascii-ps1` 30 file OK (G7); comment
  trong C# heredoc ASCII thuần.
- Việc còn (theo review): ADR-015 nêu rõ asset `windows-arm64` (setup/portable) là kế
  hoạch phát hành riêng — ghim task release kế (ARM-W7, đăng ký trên board).

## English — Post-review fix (E89, 2026-11-28)

- **Review not passed**: ci-windows ARM64 job red at the portable E2E step —
  `test-typing.ps1:141 "could not focus notepad main window"`. Root cause: the
  windows-11-arm runner session has no foreground window (`GetForegroundWindow() =
  NULL`), making `AttachThreadInput(0)` a no-op and `SetForegroundWindow` rejected by
  the Win32 foreground-lock algorithm — the old ladder had a single rung.
- **Class fix (E89)** — `Ensure-Focus` now walks a 4-rung `FocusLadder`:
  (1) AttachThreadInput + SetForegroundWindow; (2) SwitchToThisWindow ×3; (3) Alt-trick
  (SendInput VK_MENU) then SetForegroundWindow; (4) a REAL mouse click at the main
  window centre (MOUSEEVENTF_ABSOLUTE normalised to 0–65535 via GetSystemMetrics) which
  sets foreground and edit focus in one gesture. Every rung verifies
  `GetForegroundWindow() == main`; final failure throws with an E89 diagnostic dump
  (fg/main hex — no user content, G8).
- Evidence: Parser::ParseFile green; ascii-ps1 gate 30 files OK (G7); C# heredoc
  comments pure ASCII.
- Remaining (per review): ADR-015 states windows-arm64 setup/portable release assets are
  a separate planned release task (ARM-W7, to be registered on the board).

## VI — E89 vòng 2 (2026-11-28, CI run 46111640219)

- **Bằng chứng dump của vòng 1 chỉ ra nguyên nhân thật**: `fg=0x901E8` — runner CÓ
  foreground window (không phải null như giả thuyết đầu) nhưng cả 4 mức thang đều
  không đổi được nó; click tuyệt-định cũng vô hiệu ⇒ cửa sổ Notepad đang ở trạng thái
  **minimized** — `GetWindowRect` trả (-32000,-32000), click chuẩn-hóa rơi vào góc màn
  hình, không bao giờ chạm cửa sổ.
- **Fix vòng 2** (`test-typing.ps1`): (1) `Focus()` dùng `SW_RESTORE(9)` thay
  `SW_SHOW(5)` — SW_SHOW không gỡ minimized; (2) `ClickFocus` gọi `IsIconic` →
  `SW_RESTORE` + chờ 400 ms trước khi lấy rect; (3) chặn gửi click khi rect ngoài màn
  hình (virtual screen GetSystemMetrics 76/77); (4) sau click xác minh cả
  foreground lẫn vị trí con trỏ ±4 px (click bị nuốt = kênh này không đáng tin);
  (5) dump thêm `iconic=` vào lỗi cuối.
- Bằng chứng: Parser::ParseFile OK; ascii-ps1 OK (G7). Chờ run CI kế tiếp xác nhận.

## English — E89 round 2 (2026-11-28, CI run 46111640219)

- **Round-1 dump exposed the real cause**: `fg=0x901E8` — the runner HAS a foreground
  window (not null as first assumed) and none of the 4 ladder rungs changed it; even
  the absolute click was ineffective ⇒ Notepad started **minimized** — GetWindowRect
  returns (-32000,-32000), the normalised click lands in the screen corner and never
  touches the window.
- **Round-2 fix** (`test-typing.ps1`): (1) Focus() uses SW_RESTORE(9) instead of
  SW_SHOW(5) (SW_SHOW does not clear minimized); (2) ClickFocus checks IsIconic →
  SW_RESTORE + 400 ms wait before reading the rect; (3) refuses to click when the rect
  is off-screen (virtual screen via GetSystemMetrics 76/77); (4) after the click it
  verifies BOTH foreground and cursor position ±4 px (an swallowed click means this
  channel is not trustworthy); (5) the final error dump now includes `iconic=`.
- Evidence: Parser::ParseFile OK; ascii-ps1 OK (G7). Next CI run confirms.

## VI — E89 vòng 3 (2026-11-28, CI run 46113896660)

- **Dữ liệu mới**: `iconic=False` — không phải minimized; fg tồn tại (0x10202) mà cả
  4 mức thang (kể click thật) đều không đổi. Nghi vấn còn lại: session runner không
  route input (desktop nhập khác desktop thread) hoặc DPI/multi-monitor làm
  normalize sai (đã tính theo man hình chính thay vì màn hình ảo).
- **Fix vòng 3**: (1) normalize click theo **virtual screen** (SM_X/Y/CX/CYVIRTUALSCREEN
  76–79, có origin); (2) probe `SetCursorPos`+`GetCursorPos` ±4px TRƯỚC khi click —
  session không nhận input thì dừng sớm, không click mù; (3) dump chẩn đoán quyết
  định: tên desktop thread vs desktop input (`OpenInputDesktop` + `UOI_NAME`) + vị
  trí con trỏ (`cursor=x,y`).
- Nếu desktop nhập ≠ desktop thread ⇒ kết luận: ảnh runner windows-11-arm không hỗ
  trợ GUI input ⇒ bước typing trên runner này chuyển best-effort có cảnh báo (E89
  để mở), giá trị chính của ARM-W4 giữ nguyên (build/verify/ACL/installer).

## English — E89 round 3 (2026-11-28, CI run 46113896660)

- **New data**: `iconic=False`; a foreground window exists yet none of the 4 rungs
  (including a real click) changed it. Remaining suspects: the runner session does not
  route input (input desktop differs from the thread desktop), or DPI/multi-monitor
  broke normalization (primary-monitor maths instead of the virtual screen).
- **Round-3 fix**: (1) click normalisation over the **virtual screen** (SM 76-79 with
  origin); (2) `SetCursorPos`+`GetCursorPos` ±4px probe BEFORE clicking — an
  input-dead session stops early instead of blind-clicking; (3) decisive diagnostics
  in the dump: thread desktop vs input desktop names (`OpenInputDesktop` + `UOI_NAME`)
  and `cursor=x,y`.
- If the input desktop differs ⇒ conclusion: the windows-11-arm runner image does not
  support GUI input ⇒ the typing step becomes best-effort with a warning (E89 stays
  open); ARM-W4's core validations (build/verify/ACL/installer) stand.

## VI — E89 KẾT LUẬN (2026-11-28, run cb28db20 — dump quyết định)

- **Dump**: `thread-desktop=Default input-desktop=KHONG MO DUOC` — ảnh runner
  windows-11-arm **không route input** vào phiên (OpenInputDesktop thất bại) ⇒
  SendInput/click/setforeground vô nghĩa ở cấp nền tảng. 4 vòng fix trước đã đúng
  kỹ thuật (minimized → virtual-screen → probe cursor) — nguyên nhân cuối là ảnh
  runner, không phải code.
- **Phương án chốt** (giữ main xanh, trung thực về giới hạn):
  1. `test-typing.ps1` probe input-desktop ĐẦU TIÊN: không mở được ⇒ `exit 0` với
     WARN "SKIP E2E go chu (E89)" — mọi gate khác của job ARM64 (build, verify,
     check-pe-imports, installer/portable cấu trúc, sandbox_e2e trong cargo test)
     vẫn cứng.
  2. Ma trận kiến trúc x64/x86 (editor cùng nguồn) chuyển sang **job windows-2022/
     2025** nơi input thật (x64 native + x86 dưới WOW64) — giá trị cốt lõi của
     ARM-W2 ("forwarder phục vụ đúng DLL theo kiến trúc tiến trình") vẫn được
     gate bằng typing thật.
- E89 để mở theo nghĩa ảnh runner: khi GitHub cập nhật ảnh windows-11-arm có input
  desktop, bỏ probe skip là ARM64 tự typing lại đầy đủ.

## English — E89 CONCLUSION (2026-11-28, run cb28db20 — decisive dump)

- **Dump**: `thread-desktop=Default input-desktop=KHONG MO DUOC` — the windows-11-arm
  runner image does not route input into the session (OpenInputDesktop fails) ⇒
  SendInput/click/setforeground are impossible at platform level. The previous 4
  fix rounds were sound engineering (minimized → virtual-screen → cursor probe);
  the final cause is the runner image, not our code.
- **Settled plan** (keep main green, honest about the limit):
  1. `test-typing.ps1` probes the input desktop FIRST: unopenable ⇒ `exit 0` with a
     WARN "SKIP E2E go chu (E89)" — every other ARM64 job gate (build, verify,
     check-pe-imports, installer/portable structure, sandbox_e2e in cargo test)
     stays hard.
  2. The x64/x86 architecture matrix (single-source editor) moves to the
     **windows-2022/2025 job** where input works (x64 native + x86 under WOW64) —
     ARM-W2's core value ("the forwarder serves the DLL matching the process
     architecture") remains gated by REAL typing.
- E89 stays open only in the runner-image sense: when GitHub ships a windows-11-arm
  image with an input desktop, removing the skip probe restores full ARM64 typing.
