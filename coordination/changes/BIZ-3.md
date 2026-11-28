# BIZ-3 — `cargo xtask quality-trend`: bảng xu hướng KPI theo bản cho README/CHANGELOG

<!-- TASK: BIZ-3 · commit: (điều phối điền SHA khi commit) · agent: claude-opus-continue1 -->

> Agent: claude-opus-continue1 · 2026-11-28 · P0-8 §10 "README/CHANGELOG hiển thị
> xu hướng KPI theo bản (G18)" · Dep: BIZ-1 (đã xong — `quality-score` + B4c).

## VI

### Làm gì
- Lệnh mới **`cargo xtask quality-trend`** (`xtask/src/quality_trend.rs`): đọc
  `perf/quality/*.json` (CHỈ ĐỌC — không ghi file nào), in ra markdown song ngữ
  VI + EN 1:1 (G18): hai bảng `## VI` / `## EN` cùng số hàng, cùng ô giá trị, chỉ
  khác nhãn — để release manager (giữ rm-lock, R3) dán vào README (mục "Chất lượng
  theo bản") + CHANGELOG entry bản mới ở bước B4d.
- Cột = từng version trong `perf/quality/*.json` (chỉ nhận tên toàn số + chấm),
  sắp tăng dần `ver_key`, bản mới nhất bên phải. 11 hàng KPI (nhãn ghi rõ
  underlying key): KSR tổng/VI/EN, precision chữ mờ, nhiễu chữ mờ (đếm + quy đổi
  trên 100 phím cơ sở), accepts Dream RSI, recall/FP trợ lý ngữ pháp, mã lỗi đã
  sửa, nợ hồi quy, rò rỉ eval.
- Mũi tên so với CỘT LIỀN TRƯỚC: `↑` tốt hơn / `↓` xấu hơn / `→` ngang (hàng %
  chênh < 0,5% tương đối; hàng đếm bằng giá trị); hàng nằm trong regressions của
  cột mới nhất thì thêm `⚠`. KPI thiếu ở bản cũ (schema tiến hoá) → ô `—`, không
  mũi tên. `eval_hash` khác giữa 2 bản liền nhau → các hàng rsi_* ghi
  `n/a (corpus đổi)` + chân bảng có 1 dòng chú thích (VI và EN đều có).
- Fail-closed như BIZ-1: chưa có tệp `perf/quality/<ver>.json` ⇒ Err với thông báo
  chạy `quality-score --write` (B4c) trước khi phát hành (gate không được vờn
  trang trắng).

### Một nguồn sự thật (G22 — không sao chép logic)
- Giá trị ô: tái dùng **`derived_display`** (hàm mới trong `quality_score.rs`) —
  DUY NHẤT nơi định nghĩa 10 công thức hiển thị (KSR %, VI %, EN %, precision %,
  nhiễu đếm, recall %, FP, mã lỗi, nợ, rò rỉ); `quality_score::cli()` được refactor
  dùng đúng hàm này nên **số bảng trend = số quality-score một cách cấu trúc**
  (output của `quality-score` không đổi từng byte — đã đối chiếu trước/sau refactor).
- Mũi tên: `direction()` của underlying key từng hàng (một nguồn với gate).
- `⚠` lẫn gate Err: `regressions()` của bản mới nhất vs bản liền trước (cùng ngữ
  nghĩa quality-score — KHÔNG viết gate thứ hai). 1 cột duy nhất ⇒ gate bất biến
  tuyệt đối (như nhánh chưa-có-bản-trước của quality-score).
- Row phụ thuộc corpus: tái dùng `needs_same_eval` (đổi `pub`) trên các khoá underlying.

### Sửa kèm (đúng 5 tệp reserve — KHÔNG đụng tệp khác)
- `xtask/src/quality_trend.rs` (mới): renderer + 7 test `biz_3_*`.
- `xtask/src/main.rs`: dispatch `quality-trend` + dòng help (khớp style lệnh khác).
- `xtask/src/quality_score.rs`: CHỈ thêm `pub fn derived_display` + refactor `cli()`
  dùng nó (không đổi hành vi in, không đổi gate); đổi `pub` cho `repo_root`,
  `parse_file`, `ver_key`, `needs_same_eval` để BIZ-3 tái dùng.
- `docs/release/release-process.md`: bước **B4d** mới ngay sau B4c (BIZ-1).
- `coordination/changes/BIZ-3.md`: file này.

### Quyết định conservative khi spec không chốt (ghi rõ theo yêu cầu)
- `direction()` thêm **`"precision" => 1`** (cao hơn là tốt): hàng precision % của
  bảng trend cần hướng mũi tên từ MỘT nguồn, nhưng không có khoá KPI thô nào mang
  hướng này. Khoá `precision` không bao giờ xuất hiện trong `perf/quality/*.json`
  do `quality-score` ghi (`collect`/`render` không sinh) ⇒ gate `quality-score`
  KHÔNG đổi. Đổi lại: precision trend có mũi tên trung thực thay vì "→" vô nghĩa.
- `workspace_version` không dùng: bảng xu hướng lấy cột từ tên tệp (đúng thiết kế
  bảng) và luật "không I/O ngoài read_dir/read_to_string của perf/quality" cấm đọc
  Cargo.toml — nên giữ trend độc lập hoàn toàn với workspace.

## EN

### What
- New command **`cargo xtask quality-trend`** (`xtask/src/quality_trend.rs`): reads
  `perf/quality/*.json` (READ-ONLY — writes no files), prints a bilingual VI + EN
  1:1 markdown (G18): two tables `## VI` / `## EN` with the same rows, identical
  value cells, only labels differ — for the release manager (rm-lock holder, R3)
  to paste into README ("Chất lượng theo bản" section) + the new release's
  CHANGELOG entry at step B4d.
- Columns = each version in `perf/quality/*.json` (only digit+dot filenames), sorted
  by `ver_key` ascending, newest on the right. 11 KPI rows (labels spell out the
  underlying keys): overall/VI/EN KSR, ghost-text precision, ghost-text noise
  (count + per-100-baseline-keys conversion), Dream RSI accepts, grammar-assistant
  recall/FP, fixed error codes, regression debt, eval leaks.
- Arrows compare against the ADJACENT PREVIOUS column: `↑` better / `↓` worse /
  `→` flat (percent rows: relative change < 0.5%; count rows: equal value); a row
  present in the newest column's regressions gets `⚠`. A KPI missing in an older
  release (schema evolution) → `—` cell, no arrow. Different `eval_hash` between
  adjacent releases → rsi_* rows read `n/a (corpus đổi)` + a one-line footer note
  (both VI and EN).
- Fail-closed like BIZ-1: no `perf/quality/<ver>.json` at all ⇒ Err telling the
  caller to run `quality-score --write` (B4c) before releasing (a gate must not
  play dead on a blank page).

### Single source of truth (G22 — no copied logic)
- Cell values: reuses **`derived_display`** (new function in `quality_score.rs`) —
  the ONLY place defining the 10 display formulas (KSR %, VI %, EN %, precision %,
  noise count, recall %, FP, error codes, debt, leaks); `quality_score::cli()` was
  refactored to use the same function so **trend numbers = quality-score numbers
  structurally** (quality-score output verified byte-identical before/after).
- Arrows: `direction()` of each row's underlying key (one source with the gate).
- `⚠` and the gate Err: `regressions()` of the newest release vs its predecessor
  (same semantics as quality-score — NO second gate). A single column ⇒ absolute
  invariants only (like quality-score's no-previous-version branch).
- Corpus-dependent rows: reuses `needs_same_eval` (made `pub`) over underlying keys.

### Touched files (exactly the 5 reserved ones — nothing else)
- `xtask/src/quality_trend.rs` (new): renderer + 7 `biz_3_*` tests.
- `xtask/src/main.rs`: `quality-trend` dispatch + help line (matches existing style).
- `xtask/src/quality_score.rs`: ONLY added `pub fn derived_display` + refactored
  `cli()` to use it (no print-behavior change, no gate change); made `repo_root`,
  `parse_file`, `ver_key`, `needs_same_eval` `pub` for BIZ-3 reuse.
- `docs/release/release-process.md`: new step **B4d** right after B4c (BIZ-1).
- `coordination/changes/BIZ-3.md`: this file.

### Conservative decisions when the spec was open (recorded as required)
- `direction()` gained **`"precision" => 1`** (higher is better): the trend's
  precision row needs its arrow direction from ONE source, and no raw KPI key
  carries that direction. The `precision` key never appears in `perf/quality/*.json`
  files written by quality-score (`collect`/`render` never produce it) ⇒ the
  quality-score gate is UNCHANGED. Payoff: precision trends get honest arrows
  instead of a meaningless always-`→`.
- `workspace_version` is intentionally unused: trend columns come from filenames
  (per the table design) and the "no I/O beyond read_dir/read_to_string of
  perf/quality" rule forbids reading Cargo.toml — trend stays fully independent
  from the workspace.

## Tài liệu / Docs
- VI: `docs/release/release-process.md` — thêm bước **B4d** ngay sau B4c: chạy
  `cargo xtask quality-trend`, dán 2 bảng VI+EN vào README (mục "Chất lượng theo
  bản") + CHANGELOG entry bản mới — rm-lock thực hiện (R3). Agent task BIZ-3
  KHÔNG tự sửa `README.md`/`CHANGELOG.md` (tệp rm-lock); rm-lock chạy lệnh và tự
  dán output ở B4d của bản phát hành kế.
- EN: `docs/release/release-process.md` — new step **B4d** right after B4c: run
  `cargo xtask quality-trend`, paste the VI+EN tables into README ("Chất lượng
  theo bản" section) + the new release's CHANGELOG entry — done by the rm-lock
  holder (R3). The BIZ-3 task agent does NOT touch `README.md`/`CHANGELOG.md`
  (rm-locked files); the rm-lock holder runs the command and pastes the output
  at B4d of the next release.

## Kiểm thử / Testing
- VI: 7 test mới `biz_3_*` trong `quality_trend.rs`; `cargo test -p xtask`
  **118/118 xanh** (gồm 3 test `biz_1_*` cũ). `quality-trend` trên
  `perf/quality/` thật chạy 2 lần — diff rỗng (tất định tuyệt đối);
  `quality-score` sau refactor **output không đổi từng byte** + exit 0 (khớp 10
  số: KSR 7.3%, VI 7.9%, EN 5.7%, precision 15.1%, nhiễu 2705, recall 83.7%,
  FP 0, mã lỗi 234, nợ 0, rò rỉ 0). `cargo clippy -p xtask --all-targets
  -- -D warnings` sạch; rustfmt sạch cho 3 tệp Rust của task (lưu ý: `cargo fmt
  --all --check` đang đỏ ở `core/src/lib.rs` + các test core — WIP của agent
  khác, G2 không đụng).
- EN: 7 new `biz_3_*` tests in `quality_trend.rs`; `cargo test -p xtask`
  **118/118 green** (including the 3 existing `biz_1_*` tests). `quality-trend`
  on real `perf/quality/` ran twice — empty diff (fully deterministic);
  post-refactor `quality-score` **output byte-identical** + exit 0 (matches all
  10 numbers: KSR 7.3%, VI 7.9%, EN 5.7%, precision 15.1%, noise 2705, recall
  83.7%, FP 0, codes 234, debt 0, leaks 0). `cargo clippy -p xtask
  --all-targets -- -D warnings` clean; rustfmt clean for the task's 3 Rust files
  (note: `cargo fmt --all --check` is currently red on `core/src/lib.rs` + core
  tests — another agent's WIP, untouched per G2).
- VI (đột biến): (a) đổi `direction("rsi_noise")` thành `+1` trong bản thử ⇒
  `biz_3_bang_xu_huong_dung_gia_tri_va_huong` ĐỎ (kèm `biz_3_kpi_tut_bao_loi_va_exit`
  đỏ — đúng vì direction là một nguồn của gate) ⇒ đã revert, KHÔNG commit đột
  biến — mũi tên nhiễu và gate cùng treo vào `direction()` (G22); (b) rà công thức
  qua `biz_3_derived_display_khop_cong_thuc_cli`: 10 giá trị `derived_display`
  khớp đúng công thức chuỗi in của `cli()` — ai sửa công thức một đường thôi thì
  test này đỏ.
- EN (mutations): (a) flipping `direction("rsi_noise")` to `+1` in a scratch build
  makes `biz_3_bang_xu_huong_dung_gia_tri_va_huong` RED (plus
  `biz_3_kpi_tut_bao_loi_va_exit` red — correct, since direction is the gate's
  single source) ⇒ reverted, mutation NOT committed — the noise arrow and the
  gate both hang off `direction()` (G22); (b) formula audit via
  `biz_3_derived_display_khop_cong_thuc_cli`: all 10 `derived_display` values
  match the exact formulas of `cli()`'s printed line — changing either path alone
  turns this test red.
- VI (cần soát khi review): ① khoá hiển thị `"precision"` mới trong `direction()`
  (đã chứng minh không đổi gate — xem trên); ② B4d trong release-process.md là
  bước dành cho rm-lock — cần nhắc ở chu kỳ phát hành kế; ③ ô đánh dấu
  `n/a (corpus đổi)` giữ nguyên tiếng Việt trong cả 2 bảng để đảm bảo 1:1 tuyệt
  đối về ô giá trị (chú thích giải thích bằng tiếng Anh nằm ở dòng Note cuối
  bảng EN).
- EN (review pointers): ① the new `"precision"` display key in `direction()`
  (proven gate-neutral — see above); ② B4d in release-process.md is an
  rm-lock-holder step — remind at the next release cycle; ③ the cell marker
  `n/a (corpus đổi)` stays verbatim Vietnamese in both tables to guarantee strict
  1:1 value cells (the English explanation lives in the EN table's trailing Note).
