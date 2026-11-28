<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# LOOP-2b — Lexicon dự đoán dùng chung: trả lại khởi động lạnh + bộ nhớ của LOOP-2 (claude-opus-c4c5a567)

## VI

- **Vấn đề (review LOOP-2, mã E90)**: LOOP-2 tăng tốc đường nóng `predict_prefix_candidates`
  (`con` 94 µs → 0,3 µs) nhưng dựng **3 pool nhân bản mục** (`EN`, `VN`, `VN+EN`, ~41k mục × 40 B)
  kèm bản sao `StaticEntry` trong bảng top-K cho tiền tố 2–5 ký tự. DLL TSF nạp vào **mọi** tiến
  trình gõ chữ, nên lần gợi ý đầu chậm hơn và bộ nhớ nhân theo số app. Chưa có gate nào đo việc này.
- **Thiết kế (`core/src/post/predict.rs`)**:
  - `Lexicon` **một bản**: mỗi từ Việt/EN lưu đúng một lần (`flags`: `VN`, `COMPOUND_SRC`,
    `SHADOWED_BY_VN`). Chế độ `PoolMode::{EnOnly, VnOnly, VnPlusEn}` chỉ là bộ lọc `admits()`,
    giữ đúng ngữ nghĩa first-writer-wins của 3 pool cũ: mục EN trùng từ Việt bị che ở `VN+EN`.
  - Top-K **lười theo chế độ** (`mode_index`, OnceLock riêng): chỉ lưu **chỉ số `u32`**, chỉ cho
    tiền tố 2–3 ký tự. Độ dài 4–5 chỉ có ~10 nhóm > 24 mục mà tốn ~0,7 ms dựng.
  - `ext_by_first_word` chung, lọc theo chế độ lúc tra (thứ tự `rank_weight_first` giữ nguyên).
  - Băm FNV-1a cho bảng tĩnh (không có bề mặt HashDoS), sort ổn định với khoá `u64` 8 byte đầu.
  - `take_table` gộp nhánh 1 biến thể và nhiều biến thể thành một hàm `collect_variant`,
    bỏ ~40 dòng lặp.
  - Bộ lọc E84 (`vn_syllables_sorted`): âm tiết có dấu bỏ qua `is_known_en_word` (tập EN toàn
    ASCII — test `loop2b_tap_en_toan_ascii` chặn tiền đề và đối chiếu kết quả với bộ lọc đầy đủ),
    bớt ~1 ms khởi động lạnh có từ trước LOOP-2.
- **Số đo** (release, Windows x86_64, trung vị 15 lần chạy xen kẽ, probe tiến trình mới gọi
  `predict_prefix_candidates`; bộ nhớ = private bytes tăng thêm):

| | lần gọi VI đầu | lần gọi EN đầu | bộ nhớ thêm |
|---|---:|---:|---:|
| trước LOOP-2 (`662f4450^`) | 8,8 ms | 0,02 ms | +1,25 MB |
| LOOP-2 (`662f4450`, main) | 15,3 ms | 3,4 ms | +4,64 MB |
| **LOOP-2b** | **10,2 ms** | **0,6 ms** | **+2,04 MB** |

  Thu hồi ~68% độ trễ VI, ~83% độ trễ EN và ~76% bộ nhớ thêm mà LOOP-2 gây ra. Phần cold còn
  lại chủ yếu là các nguồn dữ liệu có từ trước LOOP-2 (lọc E84 âm tiết, parse từ ghép: ~8 ms).
  Đường nóng (`prediction_perf`): `con` 280 ns, `phát` 0,9 µs, `the` 0,25 µs,
  `peek_completion` hồ sơ đầy 45 µs (LOOP-2: 51–61 µs).
- **Cấu trúc chỉ mục**: `prefix_index_heap_bytes()` = 965 206 B cho cả 3 chế độ (gate ≤ 1 150 000 B).
- **Dream RSI (G19)**: `textvn-bench dream-rsi` trùng khít baseline (KSR-oracle 687/9469, VI 524,
  EN 145, nhiễu 30,8/100, accepts 395, diverged 0, invariant_breaks 0). Replay 4 adapter xanh
  (58/136/172/58). G8: không log nội dung.

### Kiểm thử + đột biến
- Test mới `loop2b_lexicon_mode_parity_sweep`: so với bản tham chiếu brute-force trên **mọi**
  tiền tố 2 ký tự trong lexicon cộng 1/7 tiền tố 3 ký tự, × 3 chế độ × limit {1, 25}. Có assert
  bảo đảm phủ đủ cả 3 chế độ.
- Test mới `loop2b_prefix_index_footprint_budget`: gate bộ nhớ tất định, kèm assert `size_of::<StaticEntry>() == 40`.
- Đột biến đã thử (đều ĐỎ, đã khôi phục):
  - M1: `VnPlusEn` bỏ che mục EN trùng từ Việt ⇒ parity sweep đỏ (`ba`).
  - M2: `VnOnly` nhận cả EN ⇒ parity đỏ (`ac`, `en_ctx=false`).
  - M3: nhân bản lexicon (quay lại kiểu pool cũ) ⇒ footprint vượt ngân sách + parity đỏ.
- `cargo test -p textvn-core` 31/31 binary xanh; `cargo test --workspace --exclude xtask` 85/85
  (bỏ `xtask` vì WIP của agent khác đang làm hỏng bản dựng); clippy `-D warnings` + rustfmt sạch.

## EN

- **Problem (LOOP-2 review, code E90)**: LOOP-2 made the `predict_prefix_candidates` hot path fast
  (`con` 94 µs → 0.3 µs), but it built **three duplicated pools** (`EN`, `VN`, `VN+EN`, ~41k entries
  × 40 B) plus copied `StaticEntry` top-K tables for 2–5 char prefixes. The TSF DLL loads into
  **every** text process, so the first prediction got slower and memory scales with the number of
  apps. No gate measured either cost.
- **Design (`core/src/post/predict.rs`)**:
  - **One** `Lexicon`: each VN/EN word is stored exactly once (`flags`: `VN`, `COMPOUND_SRC`,
    `SHADOWED_BY_VN`). `PoolMode::{EnOnly, VnOnly, VnPlusEn}` is just an `admits()` filter that
    keeps the old pools' first-writer-wins semantics: EN entries duplicating a VN word are hidden
    in `VN+EN`.
  - **Lazy per-mode** top-K (`mode_index`, separate OnceLock): stores **`u32` indices** only, and
    only for 2–3 char prefixes. Lengths 4–5 had ~10 heavy groups but cost ~0.7 ms to build.
  - Shared `ext_by_first_word`, filtered by mode at lookup (`rank_weight_first` order preserved).
  - FNV-1a hashing for the static tables (no HashDoS surface) and a stable sort with an 8-byte
    `u64` key prefix.
  - `take_table` merges the single-variant and multi-variant branches into one `collect_variant`,
    removing ~40 duplicated lines.
  - E84 filter (`vn_syllables_sorted`): accented syllables skip `is_known_en_word` (the EN sets are
    ASCII-only — test `loop2b_tap_en_toan_ascii` guards that premise and checks the result against
    the full filter), saving ~1 ms of cold start that predates LOOP-2.
- **Measurements** (release, Windows x86_64, median of 15 interleaved runs, fresh process calling
  `predict_prefix_candidates`; memory = added private bytes):

| | first VI call | first EN call | added memory |
|---|---:|---:|---:|
| before LOOP-2 (`662f4450^`) | 8.8 ms | 0.02 ms | +1.25 MB |
| LOOP-2 (`662f4450`, main) | 15.3 ms | 3.4 ms | +4.64 MB |
| **LOOP-2b** | **10.2 ms** | **0.6 ms** | **+2.04 MB** |

  This recovers ~68% of the VI latency, ~83% of the EN latency and ~76% of the extra memory that
  LOOP-2 added. Most of the remaining cold cost is data sources that predate LOOP-2 (E84 syllable
  filter, compound parsing: ~8 ms). Hot path (`prediction_perf`): `con` 280 ns, `phát` 0.9 µs,
  `the` 0.25 µs, full-profile `peek_completion` 45 µs (LOOP-2: 51–61 µs).
- **Index structures**: `prefix_index_heap_bytes()` = 965,206 B for all 3 modes (gate ≤ 1,150,000 B).
- **Dream RSI (G19)**: `textvn-bench dream-rsi` is identical to the baseline (KSR-oracle 687/9469,
  VI 524, EN 145, noise 30.8/100, accepts 395, diverged 0, invariant_breaks 0). Replay for all 4
  adapters is green (58/136/172/58). G8: no content logging.

### Tests + mutations
- New `loop2b_lexicon_mode_parity_sweep`: compares against the brute-force reference on **every**
  2-char prefix in the lexicon plus 1/7 of 3-char prefixes, × 3 modes × limit {1, 25}. An assert
  checks that all three modes are covered.
- New `loop2b_prefix_index_footprint_budget`: deterministic memory gate, plus
  `size_of::<StaticEntry>() == 40`.
- Mutations tried (all RED, then reverted):
  - M1: `VnPlusEn` stops hiding EN entries that duplicate VN words ⇒ parity sweep red (`ba`).
  - M2: `VnOnly` admits EN ⇒ parity red (`ac`, `en_ctx=false`).
  - M3: duplicating the lexicon (back to the old pools) ⇒ footprint over budget + parity red.
- `cargo test -p textvn-core`: 31/31 binaries green. `cargo test --workspace --exclude xtask`:
  85/85 (`xtask` excluded because another agent's WIP currently breaks its build). clippy
  `-D warnings` and rustfmt are clean.
