<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-1f — `xtask trace-diff` + `ALLOWED_DIFF.md` + gate chống trôi TSF (`antigravity-gemini-37c7c6bc`)

## VI

- **`xtask/src/trace_diff.rs` (mới)**: thêm lệnh `cargo xtask trace-diff [--skip-fresh]`
  (không thêm crate ngoài, tái sử dụng `check_mac_targets::parse_json` theo G22) đối chiếu
  bộ 3 vết vàng (`corpus/traces/tsf/shared/`, `corpus/traces/linux/shared/`,
  `corpus/traces/mac/shared/`) trên toàn bộ **58 ca** / **1.852 bước** `corpus/shared/`.
- **Bất biến kiểm tra bắt buộc (P0-6 §3 & §6.2)**:
  - Chỉ chấp nhận 5 lệnh chuẩn: `delete_before` (`n > 0`), `commit`, `set_preedit`, `forward_key`, `eat_key`.
  - Tối đa 1 `delete_before` và 1 `commit` mỗi bước; trên TSF & Linux bắt buộc `commit` đứng
    TRƯỚC `set_preedit("")` (`strict_commit_before_hide = true`).
  - Mọi khác biệt theo cặp `(case, step)` giữa TSF vs Linux và Linux vs macOS bắt buộc khớp
    đúng 1 quy tắc hẹp trong `corpus/traces/ALLOWED_DIFF.md` kèm đối chiếu nội dung chữ; quy tắc
    nào không khớp bước nào (`0 hit`) cũng khiến gate thoát mã `1`.
- **Gate chống trôi vết TSF (`check_tsf_freshness`)**: sinh lại toàn bộ **136 tệp** `corpus/shared`
  + `corpus/win` qua `textvn-cli replay --adapter tsf --trace <tmp>` và so sánh byte-to-byte với
  `corpus/traces/tsf/` (khớp **136/136**).
- **Tích hợp & chống tái diễn**: gắn `trace-diff` vào `xtask/src/main.rs`, `xtask/src/preflight.rs`
  (sau `replay-linux`), và `.github/workflows/ci-shared.yml` (job `replay`).
- **4 test đơn vị + đột biến D3** trong `xtask/src/trace_diff.rs`:
  - `sf1f_repo_traces_and_allowed_diff_pass`
  - `sf1f_dot_bien_xoa_rule_trong_allowed_diff_phai_do` (xoá quy tắc trong `ALLOWED_DIFF.md` ⇒ đỏ)
  - `sf1f_dot_bien_sai_chu_commit_khong_duoc_rule_nuot` (lệch chuỗi `commit` ⇒ đỏ)
  - `sf1f_fail_closed_khi_thieu_tep_hoac_lenh_la` (thiếu tệp `E94` / lệnh lạ ⇒ đỏ)
- **2 bug thật đã bắt được và sửa trước khi khoá gate**:
  1. `SF-1c` (`d6cefeb4`, `cli/src/replay.rs`): `Sim::apply_tsf` bỏ sót lệnh `"commit"` khi
     `plan.end == true` — đã sửa lấy chuỗi từ `plan.text` và sinh lại 136 vết TSF.
  2. `SF-1e` (`35e327ea`, `adapters/macos-imk/Tests/IMKLibTests/TraceTests.swift`): vòng lặp phát
     lại từng gửi `down == 0` của phím chữ qua `applyOutcome(.pass)` làm flush `marked` và
     `engine.reset()` sau mỗi phím (`ww` → `ưư`, `toowf ` → `tôưf`), thiếu `ctrlTap` và flush
     khi `:reset` — đã sửa và sinh lại 58 vết macOS trên arm64 CI (run `46142495555`).
- **Số liệu kết quả `cargo xtask trace-diff`**:
  - `58 ca shared · 1852 bước (TSF==Linux 1566/1852, Linux==Mac 76/1852)`
  - Khác biệt đã duyệt: `CUAS_PRINTABLE_BOUNDARY=286`, `MAC_IMK_PREEDIT_LIFECYCLE=1476`,
    `MAC_PRINTABLE_BOUNDARY_COMMIT=288`, `MAC_BACKSPACE_TYPE_CLEAR_ORDER=12` (**0** khác biệt ngoài bảng).
  - Gate kiểm tra cục bộ xanh 100%: `fmt`, `clippy` (Windows + Linux cross `-D warnings`),
    `cargo test -p xtask` (134 passed), `cargo test -p textvn-cli` (74 passed), `trace-diff`
    (TSF freshness 136/136), `check-ui`, `regression-index --todo-max 0`, `hygiene-docs/secrets/apis`.

## EN

- **`xtask/src/trace_diff.rs` (new)**: adds `cargo xtask trace-diff [--skip-fresh]` (zero external
  crates, reusing `check_mac_targets::parse_json` per G22) comparing the three golden trace sets
  (`corpus/traces/tsf/shared/`, `corpus/traces/linux/shared/`, `corpus/traces/mac/shared/`) across
  all **58 cases** / **1,852 steps** in `corpus/shared/`.
- **Enforced invariants (P0-6 §3 & §6.2)**:
  - Accepts only the 5 canonical commands: `delete_before` (`n > 0`), `commit`, `set_preedit`,
    `forward_key`, `eat_key`.
  - At most 1 `delete_before` and 1 `commit` per step; on TSF & Linux, `commit` MUST precede
    hiding preedit `set_preedit("")` (`strict_commit_before_hide = true`).
  - Every `(case, step)` delta for TSF vs Linux and Linux vs macOS must match an approved narrow
    structural + text-equality rule in `corpus/traces/ALLOWED_DIFF.md`; stale rules (`0 hits`) also
    fail with exit code `1`.
- **TSF drift gate (`check_tsf_freshness`)**: regenerates all **136 files** for `corpus/shared` +
  `corpus/win` via `textvn-cli replay --adapter tsf --trace <tmp>` and verifies byte-for-byte
  equality against `corpus/traces/tsf/` (**136/136** matching).
- **Wiring & regression prevention**: wired `trace-diff` into `xtask/src/main.rs`,
  `xtask/src/preflight.rs` (after `replay-linux`), and `.github/workflows/ci-shared.yml`
  (`replay` job).
- **4 unit + D3 mutation tests** in `xtask/src/trace_diff.rs`:
  - `sf1f_repo_traces_and_allowed_diff_pass`
  - `sf1f_dot_bien_xoa_rule_trong_allowed_diff_phai_do` (dropping an `ALLOWED_DIFF.md` rule fails)
  - `sf1f_dot_bien_sai_chu_commit_khong_duoc_rule_nuot` (mismatched `commit` text fails)
  - `sf1f_fail_closed_khi_thieu_tep_hoac_lenh_la` (missing files `E94` / unknown command fails)
- **2 real bugs caught and fixed before locking the gate**:
  1. `SF-1c` (`d6cefeb4`, `cli/src/replay.rs`): `Sim::apply_tsf` omitted `"commit"` when
     `plan.end == true` — fixed to take committed text from `plan.text` and regenerated all 136 TSF traces.
  2. `SF-1e` (`35e327ea`, `adapters/macos-imk/Tests/IMKLibTests/TraceTests.swift`): harness sent normal
     `down == 0` key-ups through `applyOutcome(.pass)`, prematurely flushing `marked` and calling
     `engine.reset()` after every key (`ww` → `ưư`, `toowf ` → `tôưf`), and skipped `ctrlTap` /
     marked flush on `:reset` — fixed and regenerated all 58 macOS traces on arm64 CI (run `46142495555`).
- **`cargo xtask trace-diff` summary**:
  - `58 shared cases · 1852 steps (TSF==Linux 1566/1852, Linux==Mac 76/1852)`
  - Approved deltas: `CUAS_PRINTABLE_BOUNDARY=286`, `MAC_IMK_PREEDIT_LIFECYCLE=1476`,
    `MAC_PRINTABLE_BOUNDARY_COMMIT=288`, `MAC_BACKSPACE_TYPE_CLEAR_ORDER=12` (**0** unapproved diffs).
  - Full local gate pass: `fmt`, `clippy` (Windows + Linux cross `-D warnings`),
    `cargo test -p xtask` (134 passed), `cargo test -p textvn-cli` (74 passed), `trace-diff`
    (TSF freshness 136/136), `check-ui`, `regression-index --todo-max 0`, `hygiene-docs/secrets/apis`.
