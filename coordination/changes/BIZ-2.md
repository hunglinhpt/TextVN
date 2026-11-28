<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# BIZ-2 — Miss report Dream RSI (claude-opus-c4c5a567)

## VI

- **Lệnh mới** `cargo run --release -p textvn-bench -- miss-report [--json <tệp>]`. Lệnh in bảng
  Markdown ra stdout, và ghi JSON máy đọc (`schema: textvn-miss.v1`) nếu có `--json`.
- **Đo gì**: với mỗi từ của câu đích, người dùng oracle (cùng bộ mô phỏng KSR-oracle RSI-1) có
  phải gõ tay hết không, và **vì sao**:
  - `no_ghost`: không có chữ mờ nào, tức là thiếu dữ liệu;
  - `wrong_ghost`: chỉ có chữ mờ sai, tức là xếp hạng hoặc cổng ngôn ngữ chọn nhầm;
  - `midword`: chữ mờ đúng nhưng dừng giữa từ.

  Kèm số lượt chữ mờ sai, và phân bố theo độ dài từ (1-2 / 3-4 / 5-7 / 8+).
- **Tách tập (RSI-2)**:
  - Tập held-out `corpus/eval/` **chỉ ra số tổng hợp**, không liệt kê từ nào. Lý do: nếu đem từ
    hay cụm của tập đánh giá làm dữ liệu, cụm < 6 token sẽ lọt gate eval-leak ⇒ KSR bị thổi phồng.
  - Bảng **top-50 từ bỏ lỡ** và **top-50 chữ mờ sai** chỉ lấy từ tập dev: bench cùng
    `corpus/dev/` mới (30 câu VI + 20 câu EN, viết mới cho dự án, GPL-3.0-or-later). Đây là tập
    **được phép** làm đầu vào dữ liệu bản sau.
  - Tập held-out chạy **đúng thứ tự** của `dream_rsi_oracle_run` (bench → VI → EN, một engine)
    nên số khớp phép đo gate. Tập dev chạy trên engine riêng.
- `sim_run_oracle` nhận thêm tham số `Option<&mut WordLog>` (không nhân bản bộ mô phỏng, G22).
  Các lời gọi gate truyền `None` ⇒ **Dream RSI trùng khít baseline** (đối chiếu JSON từng byte
  với `perf/baseline-win.json`).
- **Số hiện tại** (main, 0.2.96):

| Corpus | Từ | Được gợi ý | Bỏ lỡ | không chữ mờ | chữ mờ sai | dừng giữa từ |
|---|---:|---:|---:|---:|---:|---:|
| held-out VI | 1159 | 272 | 887 (76,5%) | 118 | 769 | 0 |
| held-out EN | 447 | 210 | 237 (53,0%) | 38 | 174 | 25 |
| dev VI | 359 | 86 | 273 (76,0%) | 25 | 248 | 0 |
| dev EN | 161 | 78 | 83 (51,6%) | 12 | 61 | 10 |

  Kết luận cho RSI-3…RSI-5: phần lớn từ Việt bị bỏ lỡ là do **chữ mờ sai** (769/887), không
  phải thiếu dữ liệu (118). Ưu tiên chấm điểm/xếp hạng (RSI-3) và ngưỡng tin cậy (RSI-4) trước
  việc thêm dữ liệu.

### Kiểm thử + đột biến
- Gate `cargo xtask eval-leak` (`leak_sources`, test `biz2_eval_leak_quet_ca_corpus_dev`, theo review
  claude-opus-7e41b2d9) quét cả `corpus/dev/`: tập dev **không chung chuỗi ≥ 6 token** với
  `corpus/eval/` (cùng ngưỡng `LEAK_N`, liệt kê mọi chỗ trùng). Lúc viết, test bắt được 5 chuỗi
  trùng thật, đã viết lại câu. Mọi câu dev phải tái lập được bằng Telex; test này đã loại 4 câu
  EN có `of/has/us/noon` (R2-56).
- `biz2_miss_report_tat_dinh_va_heldout_chi_co_so`:
  - kết quả tất định (2 lần chạy giống hệt);
  - khối `heldout` chỉ gồm khoá và số, không lộ từ nào ≥ 5 ký tự của eval;
  - các bất biến đếm: `covered + missed = words`, tổng theo lý do và theo độ dài khớp;
  - **mọi từ trong bảng chi tiết phải thuộc tập dev/bench**.
- `biz2_dot_bien_ghost_sai_moi_tu_bo_lo`: dùng đột biến “luôn hiện chữ mờ sai” ⇒ 0 từ được
  gợi ý, mọi từ đều bỏ lỡ.
- Đột biến đã thử: cho câu held-out VI chảy vào bảng chi tiết ⇒ ĐỎ (`từ bỏ lỡ ngoài tập dev: hai`).
- `cargo test -p textvn-bench` 14/14; clippy `-D warnings` + rustfmt sạch.
- **Cho release manager / BIZ-1**: nên thêm vào bước B4c lệnh
  `textvn-bench miss-report --json perf/quality/miss-<N.N.N>.json` để lưu xu hướng theo bản
  (`release-process.md` đang do BIZ-1 giữ chỗ).

## EN

- **New command** `cargo run --release -p textvn-bench -- miss-report [--json <file>]`. It prints a
  Markdown table to stdout, and writes machine-readable JSON (`schema: textvn-miss.v1`) when
  `--json` is given.
- **What it measures**: for every word of the target sentence, whether the oracle user (the same
  RSI-1 KSR-oracle simulator) had to type it in full, and **why**:
  - `no_ghost`: no ghost text at all, i.e. missing data;
  - `wrong_ghost`: only wrong ghosts, i.e. ranking or the language gate picked the wrong word;
  - `midword`: a correct ghost that stops mid-word.

  It also counts wrong-ghost events and breaks misses down by word length (1-2 / 3-4 / 5-7 / 8+).
- **Split discipline (RSI-2)**:
  - The held-out set `corpus/eval/` reports **aggregates only** and never lists words. Reason: if
    its words or phrases were used as data, phrases under 6 tokens would slip past the eval-leak
    gate ⇒ inflated KSR.
  - The **top-50 missed words** and **top-50 wrong ghosts** tables come only from the dev set:
    bench plus the new `corpus/dev/` (30 VI + 20 EN sentences, written new for the project,
    GPL-3.0-or-later). This is the set that **may** feed the next release's data.
  - Held-out runs in **exactly** the order of `dream_rsi_oracle_run` (bench → VI → EN, one
    engine), so its numbers match the gate measurement. Dev runs on a separate engine.
- `sim_run_oracle` gains an `Option<&mut WordLog>` parameter (no duplicated simulator, G22). Gate
  call sites pass `None` ⇒ **Dream RSI is identical to the baseline** (byte-for-byte JSON match
  against `perf/baseline-win.json`).
- **Current numbers**: see the table above. Takeaway for RSI-3…RSI-5: most Vietnamese misses come
  from **wrong ghosts** (769/887), not missing data (118). Prioritise scoring/ranking (RSI-3) and
  the confidence threshold (RSI-4) over adding more data.

### Tests + mutations
- The `cargo xtask eval-leak` gate (`leak_sources`, test `biz2_eval_leak_quet_ca_corpus_dev`, after
  review by claude-opus-7e41b2d9) now scans `corpus/dev/`: the dev set shares **no run of 6+ tokens** with
  `corpus/eval/` (same `LEAK_N` threshold; every overlap is listed). While writing the corpus it
  caught 5 real overlaps, which were rewritten. Every dev sentence must reproduce under Telex; this
  removed 4 EN sentences containing `of/has/us/noon` (R2-56).
- `biz2_miss_report_tat_dinh_va_heldout_chi_co_so`:
  - output is deterministic (two runs are identical);
  - the `heldout` block contains only keys and numbers, with no eval word of 5+ chars;
  - counting invariants hold: `covered + missed = words`, and the per-reason and per-length totals
    add up;
  - **every word in the detail tables must belong to the dev/bench set**.
- `biz2_dot_bien_ghost_sai_moi_tu_bo_lo`: under the “always show a wrong ghost” mutation, no word
  is covered and every word is missed.
- Mutation tried: feeding held-out VI sentences into the detail tables ⇒ RED
  (`từ bỏ lỡ ngoài tập dev: hai`).
- `cargo test -p textvn-bench` 14/14; clippy `-D warnings` and rustfmt are clean.
- **For the release manager / BIZ-1**: consider adding
  `textvn-bench miss-report --json perf/quality/miss-<N.N.N>.json` to step B4c to keep a per-release
  trend (`release-process.md` is currently reserved by BIZ-1).
