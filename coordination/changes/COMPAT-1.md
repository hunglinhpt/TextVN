<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# COMPAT-1 — Ma trận Windows `windows-2022` + `windows-2025` + `windows-11-arm`

## Tiếng Việt

### Tóm tắt thay đổi
- Cập nhật [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml):
  - Chuyển job `windows-package` sang ma trận `os: [windows-2022, windows-2025]` (`fail-fast: false`, cache key tách theo `win-pkg-${{ matrix.os }}-v1`, artifact tách theo `textvn-windows-x64-${{ matrix.os }}`).
  - Kết hợp cùng job `windows-arm64` (`runs-on: windows-11-arm`), CI Windows chạy đủ cả 3 môi trường: `windows-2022`, `windows-2025` và `windows-11-arm`.
- Cập nhật [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Thêm job `windows-2025-e2e` (`needs: windows`, `runs-on: windows-2025`, `timeout-minutes: 45`) chạy đủ 4 bài kiểm thử E2E trên đúng gói phát hành: `test-portable.ps1`, `test-installer.ps1`, `simulate-store-validation.ps1` và `test-msix-sideload.ps1`.
  - Nối `windows-2025-e2e` vào `publish.needs`.

### Kiểm chứng
- Gate hygiene cục bộ (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`, `ascii-ps1`) PASS.
- Kết quả thực tế trên `windows-2025` và `windows-2022` được CI `ci-windows` kiểm chứng ngay khi push.

---

## English

### Summary of Changes
- Updated [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml):
  - Expanded `windows-package` to matrix `os: [windows-2022, windows-2025]` (`fail-fast: false`, isolated `win-pkg-${{ matrix.os }}-v1` caches, and `textvn-windows-x64-${{ matrix.os }}` artifacts).
  - Together with `windows-arm64` (`windows-11-arm`), Windows CI now exercises `windows-2022`, `windows-2025`, and `windows-11-arm`.
- Updated [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Added `windows-2025-e2e` (`needs: windows`, `runs-on: windows-2025`, `timeout-minutes: 45`) running `test-portable.ps1`, `test-installer.ps1`, `simulate-store-validation.ps1`, and `test-msix-sideload.ps1` against the release artifacts.
  - Added `windows-2025-e2e` to `publish.needs`.

### Verification
- Local hygiene gates (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`, `ascii-ps1`) PASSED.
- Real `windows-2025` + `windows-2022` E2E execution is verified by `ci-windows` on push.

## VI — Sửa sau review (2026-11-28)

- **Review CHƯA PASS — 3 điểm**:
  1. `windows-2025-e2e` trong release.yml CHƯA TỪNG chạy ⇒ dispatch
     `gh workflow run release.yml --ref main` trên commit đủ mới và ghi run id (G23).
  2. Parity thiếu: job windows (2022) kiểm biến thể `-machine.exe` (ARP HKLM) nhưng
     windows-2025-e2e không ⇒ đường HKLM là chỗ dễ khác nhau giữa Server 2022/2025.
  3. Phụ thuộc: cột windows-11-arm đang đỏ (ARM-W4/E89) ⇒ COMPAT-1 chỉ done sau khi
     ARM-W4 xanh.
- **Fix (2)**: thêm bước `Machine installer E2E (ARP ở HKLM, windows-2025)` vào job
  windows-2025-e2e — chạy `validate-store-package.ps1 -MachineOnly` trên gói
  `*-machine.exe` từ artifact `release-windows`; `continue-on-error: true` đúng giai
  đoạn lấy bằng chứng như job windows (đổi gate cứng sau 2 run xanh liên tiếp).
- **Fix (1)**: dispatch release.yml trên commit chứa cả 2 fix này; run id ghi dưới đây.
- **(3)**: chờ run ci-windows của commit 2fa0c14e (fix E89 focus) — xanh thì đủ cột.
- Đổi tên artifact `textvn-windows-x64-<os>`: reviewer xác nhận không còn ai tiêu thụ
  tên cũ (đã grep toàn repo).

## English — Post-review fixes (2026-11-28)

- **Review not passed — 3 points**: (1) the release.yml windows-2025-e2e job never ran ⇒
  dispatch release.yml on a commit containing this fix and record the run id (G23);
  (2) parity gap — the windows (2022) job exercises the `-machine.exe` (ARP HKLM)
  variant but windows-2025-e2e didn't ⇒ HKLM is where Server 2022/2025 can diverge;
  (3) dependency — the windows-11-arm column is red (ARM-W4/E89) ⇒ COMPAT-1 is done
  only after ARM-W4 goes green.
- **Fix (2)**: new `Machine installer E2E (ARP ở HKLM, windows-2025)` step in the
  windows-2025-e2e job running `validate-store-package.ps1 -MachineOnly` on the
  `*-machine.exe` from the release-windows artifact; continue-on-error matches the
  evidence-gathering phase used by the windows job (hard gate after 2 consecutive green
  runs).
- **Fix (1)**: release.yml dispatched on the commit carrying both fixes; run id recorded
  below. **(3)**: waiting for the ci-windows run of 2fa0c14e (E89 focus fix).
- Artifact rename `textvn-windows-x64-<os>`: reviewer confirmed no remaining consumer of
  the old name (repo-wide grep).
