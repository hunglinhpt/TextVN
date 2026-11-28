<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# PERF-2 — Gate tất định đa kiến trúc (`x86_64` vs `aarch64`) trong `ci-shared`

## Tiếng Việt

### Tóm tắt thay đổi
- Bổ sung 2 job cuối [`.github/workflows/ci-shared.yml`](../../.github/workflows/ci-shared.yml) để kiểm chứng đầu ra giữa `x86_64` (`ubuntu-24.04`) và `aarch64` (`ubuntu-24.04-arm`) giống hệt từng byte:
  - `determinism-measure`: chạy trên ma trận `[x86_64/ubuntu-24.04, aarch64/ubuntu-24.04-arm]`, kiểm tra `uname -m`, dựng `textvn-cli` và `textvn-bench --release --locked`, xuất 5 tệp chuẩn hoá vào `det-out/`:
    1. `replay-headless.json` (`replay corpus/shared --adapter headless --json`, cắt trường timing `,"ms":[0-9]+`)
    2. `replay-tsf.json` (`replay corpus/shared corpus/win --adapter tsf --json`, cắt `,"ms":[0-9]+`)
    3. `replay-mac.json` (`replay corpus/shared corpus/mac --adapter mac --json`, cắt `,"ms":[0-9]+`)
    4. `replay-linux.json` (`replay corpus/shared --adapter linux --json`, cắt `,"ms":[0-9]+`)
    5. `dream-rsi.txt` (`textvn-bench dream-rsi`)
    Tính `LC_ALL=C sha256sum` 5 tệp ra `SHA256SUMS.txt` và upload artifact `determinism-${{ matrix.arch }}`.
  - `determinism-compare` (`needs: determinism-measure`): tải cả hai artifact, kiểm đủ 5 dòng không rỗng và chạy `diff -u` trên `SHA256SUMS.txt`; nếu lệch 1 byte sẽ in diff từng tệp và thoát `1`.

### Kiểm chứng
- Cục bộ (Windows x86_64): chạy 2 lần liên tiếp `textvn-cli replay corpus/shared --adapter headless --json` (đã loại `"ms":...`) và `textvn-bench --release dream-rsi` → SHA-256 giữa 2 lần trùng khớp 100%; kiểm tra đột biến 1 byte trên bản sao `dream-rsi.txt` tạo hash khác biệt.
- Gate hygiene: `no-python`, `hygiene-docs`, `hygiene-apis`, `regression-index`, `eval-leak` PASS.
- Đối chiếu chéo `x86_64 == aarch64` thực tế chạy trên CI `ci-shared` (`determinism-measure` + `determinism-compare`).

---

## English

### Summary of Changes
- Added two jobs to [`.github/workflows/ci-shared.yml`](../../.github/workflows/ci-shared.yml) to enforce byte-identical determinism across `x86_64` (`ubuntu-24.04`) and `aarch64` (`ubuntu-24.04-arm`):
  - `determinism-measure`: runs on `[x86_64/ubuntu-24.04, aarch64/ubuntu-24.04-arm]`, asserts `uname -m`, builds `textvn-cli` and `textvn-bench --release --locked`, captures 5 normalized outputs into `det-out/`:
    1. `replay-headless.json` (`replay corpus/shared --adapter headless --json`, stripping `,"ms":[0-9]+`)
    2. `replay-tsf.json` (`replay corpus/shared corpus/win --adapter tsf --json`, stripping `,"ms":[0-9]+`)
    3. `replay-mac.json` (`replay corpus/shared corpus/mac --adapter mac --json`, stripping `,"ms":[0-9]+`)
    4. `replay-linux.json` (`replay corpus/shared --adapter linux --json`, stripping `,"ms":[0-9]+`)
    5. `dream-rsi.txt` (`textvn-bench dream-rsi`)
    Computes `LC_ALL=C sha256sum` into `SHA256SUMS.txt` and uploads `determinism-${{ matrix.arch }}`.
  - `determinism-compare` (`needs: determinism-measure`): downloads both artifacts, checks all 5 non-empty lines, and runs `diff -u` on `SHA256SUMS.txt`; any mismatch prints per-file diffs and exits `1`.

### Verification
- Local (Windows x86_64): two consecutive runs of `textvn-cli replay corpus/shared --adapter headless --json` (with `"ms":...` stripped) and `textvn-bench --release dream-rsi` yielded identical SHA-256 digests; a 1-byte mutation on a copy of `dream-rsi.txt` produced a distinct hash.
- Hygiene gates: `no-python`, `hygiene-docs`, `hygiene-apis`, `regression-index`, `eval-leak` PASS.
- Cross-architecture `x86_64 == aarch64` equivalence is executed in `ci-shared` (`determinism-measure` + `determinism-compare`).
