<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-L4 — E2E 6 distro Linux trên container ARM64 (`release.yml`)

## Tiếng Việt

### Tóm tắt thay đổi
- Thêm job `linux-arm64-distro-matrix` (`needs: linux-arm64`, `runs-on: ubuntu-24.04-arm`, `timeout-minutes: 40`) vào [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Kiểm thử E2E thật trên cả 6 container ARM64 chính thức: `almalinux:8` (`el8`), `almalinux:9`, `opensuse/leap:15.6`, `ubuntu:22.04`, `debian:12`, `fedora:41`.
  - Xác nhận `test "$(uname -m)" = aarch64` cả trên runner chủ lẫn bên trong container Docker trước khi chạy `test-linux-el8-package.sh` / `test-linux-distro-matrix.sh` (cài per-user và `--system-only`).
  - Nối `linux-arm64-distro-matrix` vào `publish.needs` để chặn phát hành nếu bất kỳ distro ARM64 nào thất bại.

### Kiểm chứng
- `scripts/test-linux-el8-package.sh` và `scripts/test-linux-distro-matrix.sh` trung lập kiến trúc.
- Gate hygiene cục bộ (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`) PASS.
- Kích hoạt `gh workflow run release.yml --ref main` (`workflow_dispatch`) kiểm chứng 6/6 container ARM64 trên `ubuntu-24.04-arm`.

---

## English

### Summary of Changes
- Added `linux-arm64-distro-matrix` (`needs: linux-arm64`, `runs-on: ubuntu-24.04-arm`, `timeout-minutes: 40`) to [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Runs end-to-end packaging tests across all 6 official ARM64 container images: `almalinux:8` (`el8`), `almalinux:9`, `opensuse/leap:15.6`, `ubuntu:22.04`, `debian:12`, and `fedora:41`.
  - Asserts `test "$(uname -m)" = aarch64` on both host runner and inside each Docker container before executing `test-linux-el8-package.sh` / `test-linux-distro-matrix.sh` (per-user and `--system-only`).
  - Added `linux-arm64-distro-matrix` to `publish.needs`.

### Verification
- Verified `scripts/test-linux-el8-package.sh` and `scripts/test-linux-distro-matrix.sh` are architecture-neutral.
- Local hygiene gates (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`) PASSED.
- Dispatched `gh workflow run release.yml --ref main` (`workflow_dispatch`) to run all 6 ARM64 containers on `ubuntu-24.04-arm`.
