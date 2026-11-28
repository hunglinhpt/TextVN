<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-1c · Ghi vết TSF JSONL (`--trace`) và xuất JSON phím dùng chung (`--emit-keys`)

> **Trạng thái / Status**: Hoàn thành, chờ review / Ready for review  
> **Phạm vi / Scope**: `cli/src/main.rs`, `cli/src/replay.rs`, `corpus/traces/tsf/`, `coordination/changes/SF-1c.md`  
> **Phụ thuộc gỡ tắc / Unblocks**: `SF-1e` (macOS `TraceTests.swift`), `SF-1d` (Linux C `trace_compose.c`), `SF-1f` (`xtask trace-diff`)

---

## Tiếng Việt

### 1. Tổng quan thay đổi
Tiếp quản phần còn dở của `SF-1c` (`claude-opus-continue1` hết hạn mức) và đồng bộ 100% thiết kế liên nền tảng theo đề xuất `SF-1e` của `claude-opus-c4c5a567`:

1. **`textvn-cli replay --adapter tsf --trace <dir>`**:
   - Ghi lại từng bước thực thi của bộ mô phỏng TSF (`Sim::apply_tsf` và `Sim::tsf_end` dùng `plan_key` thật của `adapters/windows-tsf`) ra tệp `<dir>/<rel>.jsonl` (mỗi dòng 1 đối tượng JSON gọn):
     ```json
     {"case":"shared/telex_horn_w_01","step":36,"cmd":"commit","text":"tũn","n":0}
     ```
   - Hỗ trợ **đúng 5 lệnh `snake_case`** theo từ vựng P0-6 §3 / `SessionCommand`: `delete_before` (`n` = số ký tự UTF-32 trước composition), `commit` (`text` = chuỗi xác nhận), `set_preedit` (`text` = nội dung preedit, `""` kèm `n: 0` = ẩn preedit), `forward_key`, `eat_key`.
   - Chuẩn hóa `case` bằng dấu `/` bất kể Windows hay POSIX qua `canonical_case_rel` (tạo thư mục cha tương ứng bên trong `<dir>`).
   - Chặn từ đầu khi `--trace` truyền với adapter khác `tsf` (`exit 2`, không fail-open).

2. **`textvn-cli replay ... --emit-keys <dir>` (G22)**:
   - Xuất bộ test `.keys` đã phân tích xong ra `<dir>/<rel>.json` để `SF-1e` (Swift) và `SF-1d` (C) dùng chung 1 parser duy nhất thay vì nhân đôi bảng `key_by_name` / combo / `:config` / `:caps`.
   - Cấu trúc JSON:
     ```json
     {
       "case": "shared/...",
       "config_json": "{...}",
       "app": {"bundle":"...","field_role":0,"caps":0,"secure":false,"enabled":true},
       "keys": [{"vk":0,"ch":0,"mods":0,"down":1,"injected":false}],
       "resets": [4],
       "events": [{"step":0,"kind":"config|app|caps|reset|engine_new|mods|context",...}]
     }
     ```

3. **Vết chuẩn TSF trong `corpus/traces/tsf/`**:
   - Sinh đủ **136** tệp `.jsonl` tất định cho toàn bộ `corpus/shared` + `corpus/win`.

### 2. Kiểm chứng & Bất biến
- `cargo fmt --all --check` → PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` (Windows) và cross-target `x86_64-unknown-linux-gnu` (`--exclude textvn-win-hook`) → PASS.
- `cargo test -p textvn-cli` → **74/74 PASS**, gồm 4 test mới cho `SF-1c`:
  - `cli::tests::sf_1c_replay_flags_validate_trace_and_emit_keys`
  - `replay::tests::sf_1c_canonical_case_rel_chuan_hoa_moi_he_dieu_hanh`
  - `replay::tests::sf_1c_trace_tsf_tat_dinh_va_bat_bien_lenh` (kiểm tra tất định 2 lần, từ vựng 5 lệnh, mỗi bước ≤ 1 `delete_before`, ≤ 1 `commit`, đúng 1 `eat_key`/`forward_key`, và `commit` luôn đứng **TRƯỚC** `set_preedit("", 0)`)
  - `replay::tests::sf_1c_emit_keys_xuat_json_hop_le_va_tat_dinh`
- **Tất định đa lần chạy & kiểm tra chéo**:
  - Chạy `--trace` vào thư mục tạm đối chiếu byte-to-byte với `corpus/traces/tsf/` (136 tệp) → khớp 100%, SHA-256 hai lần chạy giống hệt nhau.
  - Chạy `--emit-keys` trên `corpus/shared` + `corpus/mac` (172 tệp JSON) → mọi tệp đều hợp lệ JSON.
- **Đột biến (D3)**:
  - Hoán đổi tạm thứ tự `set_preedit("", 0)` lên trước `commit` trong `Sim::tsf_end` (`cli/src/replay.rs`) → chạy `cargo test -p textvn-cli sf_1c_trace_tsf_tat_dinh_va_bat_bien_lenh` → **FAIL** với mã thoát 1 (`step 8: commit phải đứng TRƯỚC ẩn preedit`); hoàn nguyên → **74/74 PASS**.
- `cargo run -q -p xtask -- hygiene-docs` và `hygiene-secrets` → PASS.

### 3. Giới hạn phạm vi
- Vết trong `corpus/traces/tsf/` phản ánh bộ mô phỏng Rust TSF (`plan_key` + `Sim::apply_tsf`). Đối chiếu 3 nền tảng (TSF / Linux C `SF-1d` / macOS Swift `SF-1e`) thuộc phạm vi `SF-1f` (`xtask trace-diff`).

---

## English

### 1. Summary of Changes
Completed `SF-1c` (taking over stalled assignment after `claude-opus-continue1` hit quota with 0 WIP) while aligning 1:1 with the cross-platform trace design proposed for `SF-1e` by `claude-opus-c4c5a567`:

1. **`textvn-cli replay --adapter tsf --trace <dir>`**:
   - Records every execution step of the TSF simulator (`Sim::apply_tsf` and `Sim::tsf_end` driven by real `adapters/windows-tsf` `plan_key`) to `<dir>/<rel>.jsonl` using one compact JSON object per line:
     ```json
     {"case":"shared/telex_horn_w_01","step":36,"cmd":"commit","text":"tũn","n":0}
     ```
   - Enforces the exact **5 `snake_case` commands** aligned with P0-6 §3 / `SessionCommand`: `delete_before` (`n` = UTF-32 code points deleted before composition), `commit` (`text` = committed string), `set_preedit` (`text` = preedit content, `""` with `n: 0` = hide preedit), `forward_key`, `eat_key`.
   - Normalizes `case` identifiers with forward slashes across OSes via `canonical_case_rel` and creates nested subdirectories under `<dir>`.
   - Rejects `--trace` when paired with a non-`tsf` adapter (`exit 2`, fail-closed).

2. **`textvn-cli replay ... --emit-keys <dir>` (G22)**:
   - Exports parsed `.keys` test cases to `<dir>/<rel>.json` so `SF-1e` (Swift) and `SF-1d` (C) share a single `.keys` parser instead of duplicating `key_by_name`, modifier combos, `:config`, and `:caps` logic.
   - Emitted JSON schema:
     ```json
     {
       "case": "shared/...",
       "config_json": "{...}",
       "app": {"bundle":"...","field_role":0,"caps":0,"secure":false,"enabled":true},
       "keys": [{"vk":0,"ch":0,"mods":0,"down":1,"injected":false}],
       "resets": [4],
       "events": [{"step":0,"kind":"config|app|caps|reset|engine_new|mods|context",...}]
     }
     ```

3. **Golden TSF Traces in `corpus/traces/tsf/`**:
   - Generated **136** deterministic `.jsonl` golden trace files covering all cases in `corpus/shared` + `corpus/win`.

### 2. Verification & Invariants
- `cargo fmt --all --check` → PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` (Windows) and cross-target `x86_64-unknown-linux-gnu` (`--exclude textvn-win-hook`) → PASS.
- `cargo test -p textvn-cli` → **74/74 PASS**, including 4 new `SF-1c` tests:
  - `cli::tests::sf_1c_replay_flags_validate_trace_and_emit_keys`
  - `replay::tests::sf_1c_canonical_case_rel_chuan_hoa_moi_he_dieu_hanh`
  - `replay::tests::sf_1c_trace_tsf_tat_dinh_va_bat_bien_lenh` (checks determinism across 2 runs, 5-command vocabulary, per-step bounds `delete_before <= 1`, `commit <= 1`, `eat_key + forward_key == 1`, and `commit` strictly **BEFORE** `set_preedit("", 0)`)
  - `replay::tests::sf_1c_emit_keys_xuat_json_hop_le_va_tat_dinh`
- **Multi-run determinism & export checks**:
  - Re-ran `--trace` into a scratch directory and verified byte-for-byte / SHA-256 identity across all 136 `.jsonl` files in `corpus/traces/tsf/`.
  - Ran `--emit-keys` over `corpus/shared` + `corpus/mac` (172 JSON files) and verified every emitted file parses as valid JSON.
- **Mutation check (D3)**:
  - Temporarily swapped `set_preedit("", 0)` before `commit` in `Sim::tsf_end` (`cli/src/replay.rs`) → `cargo test -p textvn-cli sf_1c_trace_tsf_tat_dinh_va_bat_bien_lenh` **FAILED** with exit code 1 (`step 8: commit phải đứng TRƯỚC ẩn preedit`); reverted → **74/74 PASS**.
- `cargo run -q -p xtask -- hygiene-docs` and `hygiene-secrets` → PASS.

### 3. Scope Limits
- Traces in `corpus/traces/tsf/` capture the Rust TSF replay simulator (`plan_key` + `Sim::apply_tsf`). Cross-adapter trace diffing against Linux C (`SF-1d`) and macOS Swift (`SF-1e`) is the scope of `SF-1f` (`xtask trace-diff`).
