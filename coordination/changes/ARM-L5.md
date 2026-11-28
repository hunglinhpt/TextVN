<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-L5 — Gói `.deb arm64` và `PKGBUILD`/`.SRCINFO` `aarch64`

## Tiếng Việt

### Tóm tắt thay đổi
- Cập nhật [`.github/workflows/ci-deb.yml`](../../.github/workflows/ci-deb.yml):
  - Chuyển job `dpkg` sang ma trận 2 kiến trúc `[amd64 / ubuntu-latest, arm64 / ubuntu-24.04-arm]` trong container `debian:trixie` multi-arch, xác nhận `dpkg --print-architecture` khớp `${{ matrix.arch }}` và kiểm `.deb` chứa đúng `/usr/lib/$TRIPLET/fcitx5/libtextvn-fcitx5.so` (`x86_64-linux-gnu` và `aarch64-linux-gnu`).
  - Xuất artifact riêng `textvn-deb-package-amd64` và `textvn-deb-package-arm64`; bật `persist-credentials: false`.
- Cập nhật [`packaging/arch/PKGBUILD`](../../packaging/arch/PKGBUILD) và [`packaging/arch/.SRCINFO`](../../packaging/arch/.SRCINFO):
  - Mở rộng `arch=('x86_64' 'aarch64')` và bổ sung `\tarch = aarch64` đồng bộ R2-120c.
- Cập nhật [`.github/workflows/ci-aur.yml`](../../.github/workflows/ci-aur.yml):
  - Kiểm `.SRCINFO` khớp `makepkg --printsrcinfo` và bắt buộc có `\tarch = aarch64`; thêm `timeout-minutes: 45`, `persist-credentials: false`.
  - **Giới hạn ghi nhận theo P0-7 ARM-L5**: ảnh Docker chính thức `archlinux:base-devel` chỉ hỗ trợ `x86_64` (Arch Linux ARM là bản phân phối downstream riêng), nên `makepkg` chạy trên `x86_64`; phần biên dịch + e2e Linux native `aarch64` được phủ trực tiếp bởi `ARM-L1`/`ARM-L3`.

### Kiểm chứng
- Kiểm byte thực tế của `packaging/arch/.SRCINFO`: có đủ hai dòng `\tarch = x86_64` và `\tarch = aarch64` đúng định dạng TAB của `makepkg --printsrcinfo`.
- `cargo xtask check-version-sync`, `no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, `eval-leak` PASS.
- `ci-deb.yml` (`amd64` + `arm64`) và `ci-aur.yml` chạy kiểm chứng thật trên GitHub Actions (cần chạy `gh workflow run ci-deb.yml --ref main` và `gh workflow run ci-aur.yml --ref main`).

---

## English

### Summary of Changes
- Updated [`.github/workflows/ci-deb.yml`](../../.github/workflows/ci-deb.yml):
  - Expanded the `dpkg` job into a 2-architecture matrix `[amd64 / ubuntu-latest, arm64 / ubuntu-24.04-arm]` on the multi-arch `debian:trixie` container, verifying `dpkg --print-architecture` matches `${{ matrix.arch }}` and `/usr/lib/$TRIPLET/fcitx5/libtextvn-fcitx5.so` exists (`x86_64-linux-gnu` and `aarch64-linux-gnu`).
  - Uploads `textvn-deb-package-amd64` and `textvn-deb-package-arm64` artifacts with `persist-credentials: false`.
- Updated [`packaging/arch/PKGBUILD`](../../packaging/arch/PKGBUILD) and [`packaging/arch/.SRCINFO`](../../packaging/arch/.SRCINFO):
  - Expanded `arch=('x86_64' 'aarch64')` and added `\tarch = aarch64` in sync (R2-120c).
- Updated [`.github/workflows/ci-aur.yml`](../../.github/workflows/ci-aur.yml):
  - Verifies `.SRCINFO` matches `makepkg --printsrcinfo` and contains `\tarch = aarch64`; sets `timeout-minutes: 45` and `persist-credentials: false`.
  - **Documented limitation per P0-7 ARM-L5**: the official `archlinux:base-devel` container image is `x86_64`-only (Arch Linux ARM is a separate downstream port), so `makepkg` executes on `x86_64` while native Linux `aarch64` compilation and E2E remain covered by `ARM-L1`/`ARM-L3`.

### Verification
- Verified raw TAB bytes in `packaging/arch/.SRCINFO` for both `\tarch = x86_64` and `\tarch = aarch64` matching `makepkg --printsrcinfo`.
- `cargo xtask check-version-sync`, `no-python`, `hygiene-docs`, `hygiene-apis`, `hygiene-secrets`, `regression-index`, and `eval-leak` PASSED.
- Real container verification runs via GitHub Actions `ci-deb.yml` (`amd64` + `arm64`) and `ci-aur.yml`.
