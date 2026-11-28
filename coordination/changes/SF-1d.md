<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-1d · Ghi vết vàng adapter Linux (`trace_compose.c`)

> **Trạng thái / Status**: Hoàn thành, chờ review / Ready for review  
> **Phạm vi / Scope**: `adapters/linux-common/tests/trace_compose.c`, `adapters/linux-common/CMakeLists.txt`, `.github/workflows/ci-linux.yml`, `corpus/traces/linux/`, `coordination/changes/SF-1d.md`  
> **Phụ thuộc / Depends on**: `SF-1c` (`--emit-keys`), gỡ tắc / unblocks `SF-1f` (`xtask trace-diff`)

---

## Tiếng Việt

### 1. Tổng quan thay đổi
Tiếp quản phần còn dở của `SF-1d` (`claude-opus-continue1` hết hạn mức với 0 WIP) để nối đủ bộ ba vết vàng `SF-1c` (TSF) / `SF-1d` (Linux C) / `SF-1e` (macOS Swift):

1. **`adapters/linux-common/tests/trace_compose.c` (mới)**:
   - Chạy **engine Rust thật** (`libtextvn_ffi`) cùng `lc_plan_key` + `lc_edit_batch` của `linux-common` trên luồng phím JSON do `textvn-cli replay … --adapter linux --emit-keys <dir>` xuất ra (`SF-1c` — **không** viết lại bộ đọc `.keys` bằng C theo G22).
   - Chuẩn hoá mọi thao tác preedit/commit/delete/key thành **cùng 5 lệnh `snake_case`** của `SF-1c`/`SF-1e`: `delete_before`, `commit`, `set_preedit`, `forward_key`, `eat_key` trên từng dòng JSONL `{"case","step","cmd","text","n"}`.
   - Khi đặt `TEXTVN_EMIT_KEYS=<dir>`, quét đệ quy mọi `*.json` (fail-closed nếu rỗng theo E94), chạy mỗi ca **2 lần** và kiểm tra trùng khớp byte-to-byte + bất biến P0-6 §3 (`delete_before <= 1`, `commit <= 1`, `eat_key + forward_key == 1`, `commit` đứng TRƯỚC `set_preedit("", 0)`), và ghi `TEXTVN_TRACE_OUT/<case>.jsonl` nếu được đặt.
   - Kèm 3 self-test chạy mặc định trong `ctest`: parser JSON fail-closed, tất định + `commit` trước ẩn preedit, và gom `lc_edit_batch`.

2. **CMake & CI (`.github/workflows/ci-linux.yml`)**:
   - Thêm `trace_compose` vào `adapters/linux-common/CMakeLists.txt`.
   - Trong job `linux-common tests (ASan + UBSan)` của `ci-linux.yml`: chạy `--emit-keys` trên `corpus/shared`, chạy `ctest --test-dir build-asan -R '^trace_compose$'`, kiểm tra ≥ 50 tệp vết + drift gate với `corpus/traces/linux/`, và upload artifact `sf1d-traces-linux`.

3. **Vết vàng Linux trong `corpus/traces/linux/shared/`**:
   - Sinh từ artifact `sf1d-traces-linux` của `ci-linux` run `46139542149` (job `140962194897`): đủ **58** tệp `.jsonl` (`3.951` dòng JSONL hợp lệ, `0` lệnh ngoài từ vựng) cho toàn bộ `corpus/shared/`.

### 2. Kiểm chứng
- `ci-linux` run `46139542149` (commit `af741194`):
  - Job `linux-common tests (ASan + UBSan)` (`140962194897`) → **PASS** (1m24s): cả `cmake Debug + -fsanitize=address,undefined` → `ctest`, `cmake -funsigned-char -Wsign-compare` → `ctest`, và `SF-1d vết Linux (ctest -R trace_compose, ASan+UBSan)` đều xanh.
  - Job `Linux aarch64 native (ubuntu-22.04-arm)` (`140962194808`) & `Linux aarch64 native (ubuntu-24.04-arm)` (`140962194882`) → **PASS** trên ARM64 thật.
  - `repo-hygiene` (`46139542170`), `ci-deb` (`46139542158`), `ci-aur` (`46139542151`) → **PASS**.
- Kiểm chứng cục bộ trên bộ `corpus/traces/linux/` vừa tải: **58** tệp `.jsonl`, `3951` dòng JSON hợp lệ, `0` lỗi định dạng/lệnh, tất định 2 lần chạy ngay trong `trace_compose.c`.
- `cargo fmt --all --check`, `xtask hygiene-docs`, `hygiene-secrets`, `hygiene-apis` → **PASS**.

### 3. Giới hạn phạm vi
- Giống `SF-1e`, `trace_compose.c` kiểm chứng lớp áp `lc_plan_key` + `lc_edit_batch` của `linux-common`; cổng phím phụ thuộc toolkit (`IBus`/`Fcitx5` event loop, hotkey/modifier GTK/Qt, live surrounding-text query) được kiểm bằng E2E daemon thật (`e2e_ibus` / `e2e_fcitx5`) và sẽ chuyển vào `textvn-session` ở `SF-2`.

---

## English

### 1. Summary of Changes
Took over stalled `SF-1d` (`claude-opus-continue1` hit quota with 0 WIP) to complete the three-adapter golden trace triad alongside `SF-1c` (TSF) and `SF-1e` (macOS Swift):

1. **`adapters/linux-common/tests/trace_compose.c` (new)**:
   - Runs the **real Rust engine** (`libtextvn_ffi`) with `linux-common`'s `lc_plan_key` and `lc_edit_batch` over the JSON key stream exported by `textvn-cli replay … --adapter linux --emit-keys <dir>` (`SF-1c` — **no** duplicated `.keys` parser in C under G22).
   - Normalizes preedit/commit/delete/key actions into the **exact 5 `snake_case` commands** shared with `SF-1c`/`SF-1e`: `delete_before`, `commit`, `set_preedit`, `forward_key`, `eat_key` as 1-line JSONL records `{"case","step","cmd","text","n"}`.
   - When `TEXTVN_EMIT_KEYS=<dir>` is set, recursively discovers all `*.json` inputs (fail-closed if empty per E94), runs each case **twice** asserting byte-for-byte determinism and P0-6 §3 per-step invariants (`delete_before <= 1`, `commit <= 1`, `eat_key + forward_key == 1`, `commit` strictly before `set_preedit("", 0)`), and writes `TEXTVN_TRACE_OUT/<case>.jsonl` when specified.
   - Includes 3 always-on self-tests executed under default `ctest`, ASan+UBSan, and `-funsigned-char -Wsign-compare`: fail-closed JSON parser checks, synthetic determinism + commit-before-hide invariant, and `lc_edit_batch` multi-edit coalescing.

2. **CMake & CI (`.github/workflows/ci-linux.yml`)**:
   - Added `trace_compose` to `adapters/linux-common/CMakeLists.txt`.
   - In `ci-linux.yml`'s `linux-common tests (ASan + UBSan)` job: runs `--emit-keys` over `corpus/shared`, executes `ctest --test-dir build-asan -R '^trace_compose$'`, asserts `>= 50` trace files plus drift gate against committed `corpus/traces/linux/`, and uploads `sf1d-traces-linux`.

3. **Linux Golden Traces in `corpus/traces/linux/shared/`**:
   - Downloaded from `ci-linux` run `46139542149` artifact `sf1d-traces-linux` (job `140962194897`): **58** `.jsonl` files (`3,951` valid JSONL lines, `0` out-of-vocabulary commands) covering all cases in `corpus/shared/`.

### 2. Verification
- `ci-linux` run `46139542149` (commit `af741194`):
  - `linux-common tests (ASan + UBSan)` (`140962194897`) → **PASS** (1m24s): `cmake Debug + -fsanitize=address,undefined` → `ctest`, `cmake -funsigned-char -Wsign-compare` → `ctest`, and `SF-1d vết Linux (ctest -R trace_compose, ASan+UBSan)` all passed.
  - `Linux aarch64 native (ubuntu-22.04-arm)` (`140962194808`) & `Linux aarch64 native (ubuntu-24.04-arm)` (`140962194882`) → **PASS** on native ARM64.
  - `repo-hygiene` (`46139542170`), `ci-deb` (`46139542158`), `ci-aur` (`46139542151`) → **PASS**.
- Local verification of `corpus/traces/linux/`: **58** `.jsonl` files, `3,951` valid JSONL lines, `0` command/parse errors, verified deterministic over 2 runs per case in `trace_compose.c`.
- `cargo fmt --all --check`, `xtask hygiene-docs`, `hygiene-secrets`, `hygiene-apis` → **PASS**.

### 3. Scope Limits
- Like `SF-1e`, `trace_compose.c` covers the shared C composition/batching layer (`lc_plan_key` + `lc_edit_batch`); toolkit event-loop plumbing (`IBus`/`Fcitx5` hotkeys, GTK/Qt queries) continues to be tested via live daemon E2E (`e2e_ibus` / `e2e_fcitx5`) and moves into `textvn-session` in `SF-2`.
