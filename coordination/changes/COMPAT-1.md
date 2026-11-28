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
