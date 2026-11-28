<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# PERF-1 — Baseline hiệu năng theo target triple (`perf/baseline-<triple>.json`)

## Tiếng Việt

### Tóm tắt thay đổi
- Cập nhật [`tools/bench/src/main.rs`](../../tools/bench/src/main.rs):
  - Thêm `target_triple_for(arch, os)`, `host_target_triple()`, và `resolve_baseline_path(dir, triple)` ánh xạ chuẩn 7 target triple (`x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `i686-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`), fallback `baseline-win.json`.
  - `render()` ghi thêm trường `"target"` trong JSON baseline.
  - `check` và `write` hỗ trợ `--auto` (hoặc bỏ trống `<file>`) để tự chọn đúng `perf/baseline-<triple>.json` theo máy đang chạy.
  - Thêm unit test `perf1_target_triple_map_du_6_target_va_chon_baseline`.
- Cập nhật [`xtask/src/preflight.rs`](../../xtask/src/preflight.rs):
  - Bước 17 (`perf`) dùng `baseline_path_for` chọn tự động `perf/baseline-<triple>.json` theo máy chạy (`preflight` không còn hardcode `perf/baseline-win.json`).
  - Thêm unit test `perf1_preflight_chon_dung_baseline_theo_may`.
- Tạo 5 tệp baseline theo target triple:
  - [`perf/baseline-x86_64-pc-windows-msvc.json`](../../perf/baseline-x86_64-pc-windows-msvc.json)
  - [`perf/baseline-aarch64-pc-windows-msvc.json`](../../perf/baseline-aarch64-pc-windows-msvc.json)
  - [`perf/baseline-aarch64-unknown-linux-gnu.json`](../../perf/baseline-aarch64-unknown-linux-gnu.json)
  - [`perf/baseline-x86_64-unknown-linux-gnu.json`](../../perf/baseline-x86_64-unknown-linux-gnu.json)
  - [`perf/baseline-aarch64-apple-darwin.json`](../../perf/baseline-aarch64-apple-darwin.json)
  Giữ nguyên khối `dream_rsi` chuẩn (tất định tuyệt đối G19c) và ngân sách `p99_ns` (P0-3 §3.3).
- Bổ sung bước `textvn-bench check --auto` vào job `linux-arm64` ([`.github/workflows/ci-linux.yml`](../../.github/workflows/ci-linux.yml)) và `windows-arm64` ([`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml)).

### Kiểm chứng
- `rustfmt --edition 2021 --check tools/bench/src/main.rs xtask/src/preflight.rs`: PASS.
- `cargo clippy -p textvn-bench -p xtask --all-targets -- -D warnings` (Windows + cross-check `x86_64-unknown-linux-gnu`): PASS.
- `cargo test -p textvn-bench` & `cargo test -p xtask`: PASS (kèm kiểm tra đột biến `target_triple_for`).
- `cargo run -q --release -p textvn-bench -- check --auto`: chọn đúng `perf/baseline-x86_64-pc-windows-msvc.json` và PASS.

---

## English

### Summary of Changes
- Updated [`tools/bench/src/main.rs`](../../tools/bench/src/main.rs):
  - Added `target_triple_for(arch, os)`, `host_target_triple()`, and `resolve_baseline_path(dir, triple)` covering Windows, Linux, and macOS target triples with fallback to `baseline-win.json`.
  - Emits `"target"` inside `render()` JSON output.
  - Added `--auto` (and default omitted path) support for `check` and `write` to resolve `perf/baseline-<triple>.json` automatically.
  - Added unit test `perf1_target_triple_map_du_6_target_va_chon_baseline`.
- Updated [`xtask/src/preflight.rs`](../../xtask/src/preflight.rs):
  - Step 17 (`perf`) now dynamically selects `perf/baseline-<triple>.json` via `baseline_path_for`.
  - Added unit test `perf1_preflight_chon_dung_baseline_theo_may`.
- Added 5 target-triple baseline files under `perf/` (`baseline-x86_64-pc-windows-msvc.json`, `baseline-aarch64-pc-windows-msvc.json`, `baseline-aarch64-unknown-linux-gnu.json`, `baseline-x86_64-unknown-linux-gnu.json`, `baseline-aarch64-apple-darwin.json`) preserving the deterministic G19c `dream_rsi` baseline and P0-3 §3.3 `p99_ns` budgets.
- Added `cargo run -q --release --locked -p textvn-bench -- check --auto` steps to `linux-arm64` ([`.github/workflows/ci-linux.yml`](../../.github/workflows/ci-linux.yml)) and `windows-arm64` ([`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml)).

### Verification
- `rustfmt --edition 2021 --check tools/bench/src/main.rs xtask/src/preflight.rs`: PASSED.
- `cargo clippy -p textvn-bench -p xtask --all-targets -- -D warnings` (Windows + `x86_64-unknown-linux-gnu`): PASSED.
- `cargo test -p textvn-bench` & `cargo test -p xtask`: PASSED (including mutation check on `target_triple_for`).
- `cargo run -q --release -p textvn-bench -- check --auto`: selected `perf/baseline-x86_64-pc-windows-msvc.json` and PASSED.
