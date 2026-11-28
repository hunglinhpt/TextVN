<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-L3 — Phát hành gói `TextVN-<v>-linux-aarch64.tar.gz` trong `release.yml`

## Tiếng Việt

### Tóm tắt thay đổi
- Thêm job `linux-arm64` (`runs-on: ubuntu-24.04-arm`) vào [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Xác nhận kiến trúc runner (`uname -m` = `aarch64`), chạy `scripts/e2e-linux.sh` trên Ubuntu 24.04 ARM64.
  - Dựng baseline ABI EL8 (`almalinux:8` arm64: glibc 2.28 + IBus + GTK3) và compat (`ubuntu:22.04` arm64: glibc 2.35 + Fcitx5 + GTK4) tương tự job `linux` x86_64 (E67/E68).
  - Đóng gói `dist/TextVN-<v>-linux-aarch64.tar.gz` qua `scripts/build-linux.sh`, kiểm ELF `ARM aarch64` và chạy e2e gói qua `scripts/test-linux-package.sh`.
  - Upload artifact `release-linux-arm64` (`dist/TextVN-*-linux-aarch64.tar.gz`) và `linux-arm64-e2e-helpers` (`e2e_ibus`, `e2e_fcitx5` sẵn sàng cho `ARM-L4`).
- Cập nhật job `publish` trong [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Bổ sung `linux-arm64` vào `needs`.
  - Kiểm đủ cả `TextVN-*-linux-x86_64.tar.gz` (`-eq 1`) và `TextVN-*-linux-aarch64.tar.gz` (`-eq 1`) trước khi ký GPG/Sigstore và phát hành.
- **Lưu ý cho release manager (P0-7 §5 ARM-L3 / B4)**: gói `TextVN-<v>-linux-aarch64.tar.gz` cộng `.asc`, `.cosign.sig`, `.cosign.cert` làm tổng số asset phát hành tăng từ **34 lên 38** (cập nhật con số trong `docs/release/release-process.md` và `README.md` ở kỳ phát hành kế tiếp).

### Kiểm chứng
- `scripts/build-linux.sh`, `scripts/build-linux-el8.sh`, `scripts/build-linux-compat.sh`, `scripts/test-linux-package.sh` và `xtask check-elf-abi` đều trung lập kiến trúc và giữ nguyên không cần sửa.
- Gate hygiene cục bộ (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`) PASS.
- Kích hoạt `gh workflow run release.yml --ref main` (chế độ `workflow_dispatch` chỉ build/test artifact) để kiểm chứng job `linux-arm64` chạy thật trên `ubuntu-24.04-arm`.

---

## English

### Summary of Changes
- Added the `linux-arm64` job (`runs-on: ubuntu-24.04-arm`) to [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Verifies `uname -m` = `aarch64` and runs `scripts/e2e-linux.sh` natively on Ubuntu 24.04 ARM64.
  - Builds the EL8 baseline (`almalinux:8` arm64: glibc 2.28 + IBus + GTK3) and compat baseline (`ubuntu:22.04` arm64: glibc 2.35 + Fcitx5 + GTK4), mirroring the x86_64 `linux` job (E67/E68).
  - Packages `dist/TextVN-<v>-linux-aarch64.tar.gz` via `scripts/build-linux.sh`, asserts ELF `ARM aarch64`, and runs end-to-end tarball tests with `scripts/test-linux-package.sh`.
  - Uploads `release-linux-arm64` (`dist/TextVN-*-linux-aarch64.tar.gz`) and `linux-arm64-e2e-helpers` (`e2e_ibus`, `e2e_fcitx5` ready for `ARM-L4`).
- Updated the `publish` job in [`.github/workflows/release.yml`](../../.github/workflows/release.yml):
  - Added `linux-arm64` to `needs`.
  - Requires both `TextVN-*-linux-x86_64.tar.gz` (`-eq 1`) and `TextVN-*-linux-aarch64.tar.gz` (`-eq 1`) before signing and publishing.
- **Note for release manager (P0-7 §5 ARM-L3 / B4)**: adding `TextVN-<v>-linux-aarch64.tar.gz` plus `.asc`, `.cosign.sig`, and `.cosign.cert` raises total release assets from **34 to 38** (update `docs/release/release-process.md` and `README.md` on the next release).

### Verification
- Verified `scripts/build-linux.sh`, `scripts/build-linux-el8.sh`, `scripts/build-linux-compat.sh`, `scripts/test-linux-package.sh`, and `xtask check-elf-abi` are architecture-neutral with no script edits required.
- Local hygiene gates (`no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak`) PASSED.
- Dispatched `gh workflow run release.yml --ref main` (`workflow_dispatch`, artifact build/test only) to exercise `linux-arm64` end-to-end on `ubuntu-24.04-arm`.
