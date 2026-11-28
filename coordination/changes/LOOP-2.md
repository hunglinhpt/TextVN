<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# LOOP-2 — Chỉ mục tiền tố top-K cho `predict_prefix_candidates` (antigravity-gemini-37c7c6bc)

## VI

- **Mục tiêu (P0-8 §7 & §10)**: Cắt độ trễ đường nóng `predict_prefix_candidates` (vốn quét tuyến tính ~1.9k–13k mục, tốn `~93.99µs` cho tiền tố `"con"`) xuống `≤ 5µs`, giữ **100% đồng nhất kết quả & thứ tự** với thuật toán tham chiếu và **không suy giảm Dream RSI (G19)**.
- **Thiết kế (`core/src/post/predict.rs`)**:
  - `StaticPool` hợp nhất (`pool_en_only`, `pool_vn_only`, `pool_vn_plus_en`) dựng một lần qua `OnceLock`, giữ nguyên thứ tự khử trùng `first-writer-wins` và ngữ nghĩa phân tách `from_vn` / `is_compound_src` của từng nguồn.
  - `PrefixTable`: sắp theo `(key ASC, rank_cmp)` và dựng sẵn bảng `heavy_top_k` (`HEAVY_TOP_K = 24`) cho mọi tiền tố 2–5 ký tự có `> 24` mục (O(1) tra cứu top-K; tự fallback về `by_key` khi `limit` hoặc số mục trùng `personal`/bị loại vượt ngưỡng 24).
  - `ext_by_first_word`: tra cứu O(1) các từ ghép mở rộng đúng nghĩa (`exact_match = true, ext = true`) và tính chính xác `has_ext = personal_has_ext || static_has_ext`.
  - `vn_compounds_deacc_sorted`: bảng từ ghép bỏ dấu sắp sẵn để xử lý nhánh `exact_match = false` khi gõ không dấu ở mode VI.
  - Tối ưu không-cấp-phát bổ sung: nhánh `restyle()` trả sớm với chuỗi ASCII thuần; `predict_next_phrase_matches_with_personal()` dùng mảng stack `[&str; 4]` thay `Vec::collect()` và bỏ `pred.to_string()` khi `limit <= 32`.
- **Hiệu năng `--release` (`cargo test -p textvn-core --release --test prediction_perf -- --nocapture`)**:

| Tác vụ | Trước (`62445ea2`) | Sau (worktree sạch) | Tỉ lệ |
|---|---:|---:|---:|
| `prefix 'con' (VI, ngắn)` | `93.991µs` | **`548ns`** | **~171× nhanh hơn** |
| `prefix 'phát' (VI)` | `12.229µs` | **`1.536µs`** | **~8.0× nhanh hơn** |
| `prefix 'the' (EN)` | `5.878µs` | **`725ns`** | **~8.1× nhanh hơn** |
| `next-phrase đuôi dài` | `5.47µs` | **`2.817µs`** | **~1.9× nhanh hơn** |
| `next-phrase 'cộng hòa'` | `1.35µs` | **`818ns`** | **~1.7× nhanh hơn** |
| `peek_completion hồ sơ đầy` | `88.014µs` | **`60.973µs`** | **~1.4× nhanh hơn** |

- **Dream RSI (G19) & Bảo mật (G8)**: Không đổi trọng số, không đổi cổng RSI-7, giữ nguyên 100% đầu ra và bộ đếm `dream_rsi_100` (12/12 pass); không ghi log nội dung phím.

## EN

- **Goal (P0-8 §7 & §10)**: Cut hot-path `predict_prefix_candidates` latency (previously scanning ~1.9k–13k entries and taking `~93.99µs` on `"con"`) down to `≤ 5µs`, preserving **100% output & ranking parity** with the reference algorithm and **zero Dream RSI regression (G19)**.
- **Design (`core/src/post/predict.rs`)**:
  - Unified `StaticPool` caches (`pool_en_only`, `pool_vn_only`, `pool_vn_plus_en`) built once via `OnceLock`, preserving exact `first-writer-wins` deduplication and per-source `from_vn` / `is_compound_src` semantics.
  - `PrefixTable`: sorted by `(key ASC, rank_cmp)` with precomputed `heavy_top_k` (`HEAVY_TOP_K = 24`) for every 2–5 char prefix having `> 24` entries (O(1) lookup, automatically falling back to `by_key` whenever `limit` plus `personal`/excluded overlaps exceeds 24).
  - `ext_by_first_word`: O(1) lookup for true-extension compounds (`exact_match = true, ext = true`), computing exact `has_ext = personal_has_ext || static_has_ext`.
  - `vn_compounds_deacc_sorted`: sorted deaccented Vietnamese compound table for unaccented `exact_match = false` fallback in VI mode.
  - Extra zero-allocation wins: ASCII fast-path in `restyle()`, stack array `[&str; 4]` instead of per-entry `Vec` allocation in `predict_next_phrase_matches_with_personal()`, and zero `String` allocation for dedup when `limit <= 32`.
- **Release Benchmark (`cargo test -p textvn-core --release --test prediction_perf -- --nocapture`)**:

| Benchmark | Before (`62445ea2`) | After (clean worktree) | Speedup |
|---|---:|---:|---:|
| `prefix 'con' (VI, short)` | `93.991µs` | **`548ns`** | **~171× faster** |
| `prefix 'phát' (VI)` | `12.229µs` | **`1.536µs`** | **~8.0× faster** |
| `prefix 'the' (EN)` | `5.878µs` | **`725ns`** | **~8.1× faster** |
| `next-phrase long tail` | `5.47µs` | **`2.817µs`** | **~1.9× faster** |
| `next-phrase 'cộng hòa'` | `1.35µs` | **`818ns`** | **~1.7× faster** |
| `peek_completion full profile` | `88.014µs` | **`60.973µs`** | **~1.4× faster** |

- **Dream RSI (G19) & Security (G8)**: Weights, RSI-7 language gate, and candidate rankings are bit-for-bit identical; `dream_rsi_100` (12/12 pass) unchanged; zero keystroke content logging.

## Kiểm thử / Testing

- `rustfmt --edition 2021 --check core/src/post/predict.rs` -> PASS (0 diff).
- `cargo clippy -p textvn-core --all-targets -- -D warnings` -> PASS (0 warnings).
- Clean isolated worktree (`wt-loop2` @ `d5b0ac1e`): `cargo test -p textvn-core` -> ALL test binaries PASS (196 unit tests incl. `loop2_prefix_index_exact_parity` across 86 prefixes × 4 modes × 7 limits + personal/heavy overlays & `loop2_next_phrase_personal_stack_words`; `dream_rsi_100` 12/12; `r2_173_prediction_chain` 8/8; `text_prediction` 27/27; `prediction_perf` 2/2).
- Mutation kill: setting `has_ext = false` in `predict_impl` immediately failed `loop2_prefix_index_exact_parity` at `prefix="cô", is_vi=true, en_ctx=true, limit=1` (`["côi"]` != `["cô chú"]`), then reverted.
