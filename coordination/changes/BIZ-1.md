# BIZ-1 — `cargo xtask quality-score`: bảng điểm chất lượng chung

> Agent: antigravity-gemini-37c7c6bc · 2026-11-28 · P0-8 §5/§8, P0-7 "Luồng R".
> Lưu ý: khung ban đầu của `xtask/src/quality_score.rs` là bản nháp mồ côi tìm thấy
> trong cây (không ai claim/reserve — đã notify `*` trước khi hấp thụ, công nhận
> trong ghi chú này).

## VI

### Làm gì
- Lệnh mới **`cargo xtask quality-score [--write]`** (`xtask/src/quality_score.rs`):
  một bảng KPI chung cho MỌI tính năng, gom từ công cụ ĐÃ CÓ (G22 — không đo lại):
  - Dream RSI KSR-oracle: `textvn-bench dream-rsi` (RSI-1, corpus held-out `corpus/eval/` RSI-2)
    → `rsi_saved_keys`, `rsi_vi_saved`, `rsi_en_saved`, `rsi_ghosts_shown/good`, `rsi_noise`,
    `rsi_accepts`, `rsi_invariant_breaks`, `rsi_diverged`, `rsi_max_chain`.
  - Trợ lý ngữ pháp: `xtask assist-score --json --mode check` (ghim mode cho tất định)
    → `assist_errors/detected/clean/fp_hard`.
  - Chống tái diễn: số mã lỗi + nợ `todo:REG-2` từ `regression_index` (REG-1)
    → `regression_codes`, `regression_todo`.
  - Rò rỉ tập đánh giá: `eval_leak` (RSI-2) → `eval_leak`.
- So với bản TRƯỚC trong `perf/quality/<version>.json`: tiết kiệm phím/detect/không được GIẢM;
  nhiễu/FP/nợ không được TĂNG; bất biến tuyệt đối (`rsi_diverged`, `rsi_invariant_breaks`,
  `eval_leak`, `regression_todo`) phải = 0. KPI Dream RSI chỉ so khi `eval_hash` (FNV-1a của
  `corpus/eval/*.txt` + `tools/bench/src/main.rs`) trùng — đổi corpus = baseline mới.
- `--write`: ghi `perf/quality/<version>.json` + nối `perf/quality/history.jsonl`
  (idempotent theo version) — bước B4b của release manager trước khi tag (G16/G23).

### Sửa kèm
- `xtask/src/eval_leak.rs`: tách `pub fn scan(root) -> LeakScan` — một nguồn cho CLI
  `eval-leak` lẫn KPI `eval_leak` (G22); CLI in chi tiết từ cùng kết quả quét. Test mới
  `biz_1_scan_quet_cay_dem_dung_ro_ri_va_loi_khi_mat_tap_danh_gia` (cây mẫu trong temp dir:
  đếm đúng + fail-closed khi mất tập đánh giá).
- `xtask/src/main.rs`: dispatch `quality-score` + dòng help.
- `xtask/src/preflight.rs`: bước **`quality`** ngay sau `perf` (28 bước) — binaries bench/xtask
  đã build nên bước này nhanh; chưa có bản trước ⇒ chỉ gate bất biến tuyệt đối (đúng P0-8 §8
  "preflight (nhanh)"); sau B4b đầu tiên có baseline ⇒ gate KPI tụt đầy đủ.

### Bằng chứng
- `cargo test -p xtask` 107/107 (3 test mới `biz_1_*`).
- `cargo run -q -p xtask -- quality-score` trên máy dev: `quality-score 0.2.96: KSR 7.3%
  (VI 7.9% · EN 5.7%) · precision 15.1% · nhiễu 2705 · trợ lý recall 83.7% FP 0 · mã lỗi 231
  (nợ 0) · rò rỉ eval 0` — khớp baseline RSI-2 (687/9469 = 7,3%).
- Đột biến D3: gỡ `rsi_noise` khỏi nhóm "thấp hơn là tốt" ⇒ `biz_1_kpi_tut_thi_do_theo_huong_tot`
  ĐỎ; revert ⇒ xanh (nhiễu tăng phải chặn).
- Việc còn: BIZ-2 (miss report), BIZ-3 (README/CHANGELOG xu hướng KPI); dòng "B4b chạy
  `cargo xtask quality-score --write`" sẽ do release manager thêm vào
  `docs/release/release-process.md` ở bản phát hành kế (tệp đang theo quy trình rm-lock).

## EN

### What
- New command **`cargo xtask quality-score [--write]`** (`xtask/src/quality_score.rs`):
  one shared KPI scorecard for ALL features, gathered from EXISTING tools (G22 — no re-measuring):
  - Dream RSI KSR-oracle: `textvn-bench dream-rsi` (RSI-1, held-out corpus `corpus/eval/` RSI-2)
    → `rsi_saved_keys`, `rsi_vi_saved`, `rsi_en_saved`, `rsi_ghosts_shown/good`, `rsi_noise`,
    `rsi_accepts`, `rsi_invariant_breaks`, `rsi_diverged`, `rsi_max_chain`.
  - Grammar assistant: `xtask assist-score --json --mode check` (mode pinned for determinism)
    → `assist_errors/detected/clean/fp_hard`.
  - Regression defence: error-code count + `todo:REG-2` debt from `regression_index` (REG-1)
    → `regression_codes`, `regression_todo`.
  - Eval-set leakage: `eval_leak` (RSI-2) → `eval_leak`.
- Compared against the PREVIOUS version in `perf/quality/<version>.json`: keystroke savings /
  detections must not DROP; noise/FP/debt must not RISE; absolute invariants (`rsi_diverged`,
  `rsi_invariant_breaks`, `eval_leak`, `regression_todo`) must be 0. Dream RSI KPIs only compare
  when `eval_hash` (FNV-1a over `corpus/eval/*.txt` + `tools/bench/src/main.rs`) matches — a corpus
  change means a new baseline.
- `--write`: writes `perf/quality/<version>.json` + appends `perf/quality/history.jsonl`
  (idempotent per version) — release manager's B4b step before tagging (G16/G23).

### Touched files
- `xtask/src/eval_leak.rs`: extracted `pub fn scan(root) -> LeakScan` — single source for both the
  `eval-leak` CLI and the `eval_leak` KPI (G22); CLI prints details from the same scan. New test
  `biz_1_scan_quet_cay_dem_dung_ro_ri_va_loi_khi_mat_tap_danh_gia` (sample tree in temp dir:
  correct count + fail-closed when the eval set disappears).
- `xtask/src/main.rs`: `quality-score` dispatch + help line.
- `xtask/src/preflight.rs`: new **`quality`** step right after `perf` (28 steps) — bench/xtask
  binaries are already built so the step is fast; no previous version yet ⇒ only absolute-invariant
  gate (P0-8 §8 "fast preflight"); after the first B4b a baseline exists ⇒ full regression gate.

### Evidence
- `cargo test -p xtask` 107/107 (3 new `biz_1_*` tests).
- `cargo run -q -p xtask -- quality-score` on the dev machine: `quality-score 0.2.96: KSR 7.3%
  (VI 7.9% · EN 5.7%) · precision 15.1% · noise 2705 · assistant recall 83.7% FP 0 · error codes 231
  (debt 0) · eval leaks 0` — matches the RSI-2 baseline (687/9469 = 7.3%).
- Mutation D3: removing `rsi_noise` from the "lower is better" set makes
  `biz_1_kpi_tut_thi_do_theo_huong_tot` RED; reverted ⇒ green (rising noise must be blocked).
- Remaining: BIZ-2 (miss report), BIZ-3 (README/CHANGELOG KPI trend); the "B4b runs
  `cargo xtask quality-score --write`" line will be added to `docs/release/release-process.md`
  by the release manager in the next release (file is under the rm-lock process).

## VI — Sửa sau review (2026-11-28)

- **Review CHƯA PASS (2 lỗi, "gate không bao giờ được vô hiệu")**:
  1. `perf/quality/` rỗng + release-process.md không có bước `--write` ⇒ gate
     so-bản-trước vô hiệu vĩnh viễn. **Fix**: (a) thêm bước **B4c** vào
     `docs/release/release-process.md` — `cargo xtask quality-score --write` + commit
     `perf/quality/<N.N.N>.json` + `history.jsonl` cùng bộ docs B4; (b) commit ngay
     `perf/quality/0.2.96.json` (số thật đo từ main: KSR 7,3%, recall 83,7%, nợ 0,
     rò rỉ 0) ⇒ gate sống từ bây giờ.
  2. `previous_file()` loại chính version hiện tại ⇒ sau khi phát hành 0.2.97, commit
     dev kế tiếp (vẫn 0.2.97) so với 0.2.96 — tụt so với bản vừa phát hành mà vẫn
     "đạt". **Fix**: `previous_file(dir, version, include_equal)` — đo thường (không
     `--write`) so với bản GẦN NHẤT ≤ version (gồm chính nó); khi `--write` mới loại
     chính nó (đang ghi đè). Test mới `biz_1_chon_ban_so_sanh_theo_write` phủ cả 2
     nhánh + nhánh chưa có bản cùng version.
- **PERF-1 review gợi ý (áp dụng)**: `validate_baseline` từ chối `run` bắt đầu
  `local-dev` khi `GITHUB_ACTIONS=true` — số máy dev không được quay lại gate CI.
- Bằng chứng: `cargo test -p xtask quality_score` 3/3, bench 11/11, clippy -D warnings
  sạch; `quality-score --write` ghi 0.2.96.json + history.jsonl thật.

## English — Post-review fixes (2026-11-28)

- **Review not passed (2 defects, "a gate must never be disabled")**:
  1. Empty `perf/quality/` + no `--write` step in release-process.md ⇒ the
     previous-version gate would stay dead forever. **Fix**: (a) new step **B4c** in
     `docs/release/release-process.md` — `cargo xtask quality-score --write` + commit
     `perf/quality/<N.N.N>.json` + `history.jsonl` with the B4 docs; (b) committed
     `perf/quality/0.2.96.json` right now (real numbers measured from main: KSR 7.3%,
     recall 83.7%, debt 0, leaks 0) so the gate is alive from today.
  2. `previous_file()` excluded the CURRENT version ⇒ after 0.2.97 ships, the next dev
     commit (still 0.2.97) compared against 0.2.96 — a regression vs the just-released
     version would pass. **Fix**: `previous_file(dir, version, include_equal)` — normal
     measurement compares against the CLOSEST version ≤ current (including itself);
     only `--write` excludes itself (it is overwriting it). New test
     `biz_1_chon_ban_so_sanh_theo_write` covers both branches + the missing-current
     case.
- **PERF-1 review suggestion applied**: `validate_baseline` rejects a `run` starting
  with `local-dev` when `GITHUB_ACTIONS=true` — dev-machine numbers can never re-enter
  the CI gate.
- Evidence: xtask quality_score tests 3/3, bench 11/11, clippy clean at -D warnings;
  `quality-score --write` wrote real 0.2.96.json + history.jsonl.
