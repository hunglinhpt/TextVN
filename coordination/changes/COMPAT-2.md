<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# COMPAT-2 — Mở rộng ma trận distro E2E (claude-opus-c4c5a567)

## VI

### Phần 1 — `9a0c101f` (đã đẩy)
- `ci-linux.yml` job `e2e-distro-matrix` còn **bản sao thứ 3 y nguyên** của khối `docker run`
  (deps từng distro, machine-id, user tester) mà ARM-L4 đã gom về
  `scripts/run-distro-container.sh` cho `release.yml` (G22). Job giờ gọi script dùng chung và
  kiểm `uname -m = x86_64` cả trong container.
- `scripts/test-linux-distro-matrix.sh`: `case` kỳ vọng GTK trước đây **lặng lẽ bỏ qua tag lạ**,
  nên distro mới thêm vào ma trận không bị kiểm GTK. Nay là danh sách đóng gồm cả các tag
  COMPAT-2; tag chưa khai báo ⇒ FAIL.

### Phần 2 — `release.yml` + `scripts/run-distro-container.sh`
- Thêm vào `release.yml` (cùng `scripts/run-distro-container.sh`):

| Tag | Ảnh | x86_64 | arm64 | Kỳ vọng settings |
|---|---|:-:|:-:|---|
| `ubuntu-24-04` | `ubuntu:24.04` | ✓ | ✓ | GTK4 |
| `ubuntu-26-04` | `ubuntu:26.04` | ✓ | ✓ | GTK4 |
| `mint-22` | `linuxmintd/mint22-amd64` | ✓ | — (Docker Hub chỉ có amd64) | GTK4 |
| `debian-13` | `debian:13` | ✓ | ✓ | GTK4 |
| `fedora-42` | `fedora:42` | ✓ | ✓ | GTK4 |
| `fedora-43` | `fedora:43` | ✓ | ✓ | GTK4 |
| `opensuse-tumbleweed` | `opensuse/tumbleweed` | ✓ | ✓ | GTK4 |
| `almalinux-10` | `almalinux:10` | ✓ | ✓ | GTK3/4 (chỉ IBus, fcitx5 ở EPEL) |

- Giữ nguyên 6 distro cũ (AlmaLinux 8/9, Leap 15.6, Ubuntu 22.04, Debian 12, Fedora 41) và 2 smoke
  LTS cũ. Đã kiểm tra trên Docker Hub ngày 2026-11-28: đủ ảnh và kiến trúc như bảng.
- **Cho release manager (G18)**: README VI/EN đổi số họ/bản distro được kiểm E2E
  (6 → 14 bản x86_64, 6 → 13 bản arm64); `release-process.md` B7 không đổi số asset.

## EN

### Part 1 — `9a0c101f` (pushed)
- The `ci-linux.yml` `e2e-distro-matrix` job still held a **third verbatim copy** of the
  `docker run` block (per-distro deps, machine-id, tester user) that ARM-L4 moved into
  `scripts/run-distro-container.sh` for `release.yml` (G22). It now calls the shared script and
  also asserts `uname -m = x86_64` inside the container.
- `scripts/test-linux-distro-matrix.sh`: the GTK expectation `case` used to **silently skip unknown
  tags**, so distros newly added to the matrix were never GTK-checked. It is now a closed list
  that includes the COMPAT-2 tags; an undeclared tag FAILS.

### Part 2 — `release.yml` + `scripts/run-distro-container.sh`
- Add the distros in the table above to `release.yml`, using the same
  `scripts/run-distro-container.sh`: x86_64 for all 8, arm64 for all but Mint 22, which only has
  an amd64 image on Docker Hub.
- The 6 existing distros (AlmaLinux 8/9, Leap 15.6, Ubuntu 22.04, Debian 12, Fedora 41) and the 2
  legacy LTS smoke jobs stay unchanged. Images and architectures were checked on Docker Hub on
  2026-11-28.
- **For the release manager (G18)**: update the README VI/EN count of E2E-tested distros
  (6 → 14 on x86_64, 6 → 13 on arm64). The release-process.md B7 asset count is unchanged.
