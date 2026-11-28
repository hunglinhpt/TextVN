# REG-2 — Bù bằng chứng cho mã lỗi cũ (claude-opus-c4c5a567)

## VI

- **Sổ hồi quy còn 0 nợ**: 230 mã lỗi (E*, `win/*`, R*, R2-*) đều có bằng chứng — 190 tham chiếu test,
  64 gate, 16 corpus/chương trình test, 56 bài học harness Windows, 36 mã chỉ-quy-trình; `todo-max = 0` ⇒ từ nay
  mã lỗi mới phải có bằng chứng thật ngay khi ghi (gate `regression-index` đỏ nếu thiếu).
- **Test mới** (rút luật ra hàm thuần, đột biến luật ⇒ đỏ), tất cả ở `adapters/windows-tsf`:
  R2-120h `style_drain_due` (drain dữ liệu học đúng mỗi 20 phím), R2-123 `deferred_key_disposition` (phím TIP đã
  ăn không replay Tab/Ctrl+Space thừa vào app CUAS), R2-127 `metrics_for_dpi` (cỡ chữ panel 20/15/18 px ở 96 DPI,
  scale theo DPI), E30 `assist_buffer_is_ours` (popup re-check sửa tiếp được sau lần sửa đầu).
- **Gate ghi rõ mã nó chặn**: preflight clippy (E8, E37, E52), check-linux (E9), clippy-linux (E14), ascii-ps1 (E32),
  repo-hygiene bước 2 (E54), `.gitattributes` (E31, E35); e2e thật `tools/win/e2e-assist-apps.ps1` ghi R2-103/132/135/137.
- Khám phá tự động: đầu tệp `tests/`, test bộ cài `installer/windows/tests/*.ps1`, chương trình test C/C++ Linux;
  `--merge` nâng dòng nợ khi đã có bằng chứng và CHỈ được hạ `todo-max`.

## EN

- **Regression index has zero debt**: all 230 error codes (E*, `win/*`, R*, R2-*) have evidence — 190 test refs,
  64 gates, 16 corpus/program tests, 56 Windows-harness lessons, 36 process-only codes; `todo-max = 0` ⇒ from now on a new
  error code must come with real evidence when it is recorded (the `regression-index` gate goes red otherwise).
- **New tests** (rule extracted into a pure function, mutating the rule ⇒ red), all in `adapters/windows-tsf`:
  R2-120h `style_drain_due` (learning data drained exactly every 20 keys), R2-123 `deferred_key_disposition` (keys the TIP
  consumed are not replayed as an extra Tab/Ctrl+Space into CUAS apps), R2-127 `metrics_for_dpi` (panel fonts 20/15/18 px at
  96 DPI, scaled by DPI), E30 `assist_buffer_is_ours` (the re-check popup can fix again after the first fix).
- **Gates name the codes they block**: preflight clippy (E8, E37, E52), check-linux (E9), clippy-linux (E14), ascii-ps1
  (E32), repo-hygiene step 2 (E54), `.gitattributes` (E31, E35); the real-app e2e `tools/win/e2e-assist-apps.ps1` names
  R2-103/132/135/137.
- Auto-discovery: headers of `tests/` files, installer tests `installer/windows/tests/*.ps1`, Linux C/C++ test programs;
  `--merge` upgrades debt rows once evidence exists and may only LOWER `todo-max`.

## Tài liệu / Docs

`docs/specs/regression-index.md`, `coordination/changes/REG-1.md`.

## Kiểm thử / Testing

`cargo test -p textvn-win-tsf` (4 test mới), `cargo xtask regression-index` 230 mã · todo 0/0, preflight 26/26.
