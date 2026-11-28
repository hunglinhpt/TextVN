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

## VI — Sửa sau review (G22, 2026-11-28)

- **Review: ACCEPTANCE ĐẠT nhưng CHƯA PASS G22** — khối `docker run` (~60 dòng: deps
  dnf/zypper/apt, machine-id, useradd tester, nhánh el8) trong job
  `linux-arm64-distro-matrix` là bản sao y nguyên của job x86_64, chỉ khác 1 dòng assert
  arch ⇒ sửa phụ thuộc distro một bên sẽ trôi bên kia.
- **Fix cả lớp lỗi**: tách thành `scripts/run-distro-container.sh` — MỘT nguồn cho toàn
  bộ kiến thức deps-per-distro + user tester + mode el8/matrix (tham số hoá image/tag/
  mode/tarball/helpers/expect-arch). Hai job `linux-distro-matrix` (x86_64) và
  `linux-arm64-distro-matrix` (aarch64) giờ chỉ còn chọn runner, artifact và glob gói;
  tên job GIỮ NGUYÊN để `publish.needs` không đổi. Bước arm64 truyền `expect-arch=aarch64`
  (assert cả runner lẫn container — nội dung assert cũ giữ nguyên qua script).
- Bằng chứng: `python yaml.safe_load` OK; `bash -n` script OK; dry-run release.yml trên
  commit kế tiếp (run id ghi ở đây sau khi xanh).

## English — Post-review fix (G22, 2026-11-28)

- **Review: acceptance met but NOT passed on G22** — the arm64 job's `docker run` block
  (~60 lines: dnf/zypper/apt deps, machine-id, tester useradd, el8 branch) was a verbatim
  copy of the x86_64 job differing by one arch assert ⇒ a distro-dependency fix on one
  side silently drifts from the other.
- **Class fix**: extracted `scripts/run-distro-container.sh` as the single source for
  per-distro deps + tester user + el8/matrix modes (parameterised image/tag/mode/tarball/
  helpers/expect-arch). The two jobs keep their ids (publish.needs unchanged) and now only
  pick runner, artifacts, and tarball glob; the arm64 job passes `expect-arch=aarch64`
  (runner + container asserts preserved through the script).
- Evidence: yaml.safe_load OK; bash -n OK; a fresh release.yml dry-run on the next commit
  (run id to be recorded here once green).

## Bằng chứng cuối (G23 dry-run)

- **Release run `46111841387` (commit 2beb299d, dispatch 2026-11-28): SUCCESS 17/17**
  — gồm `Linux ARM64 E2E` 6/6 (RHEL 8.10/9, SLES 15.6, Ubuntu 22.04, Debian 12,
  Fedora 41) qua `scripts/run-distro-container.sh` (G22 một nguồn) và `Windows E2E
  (windows-2025)` (COMPAT-1). Reviewer đã ghi "Sau khi gộp chạy lại release dry-run
  (G23) rồi release done" ⇒ điều kiện đạt.
