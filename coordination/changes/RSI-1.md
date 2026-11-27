# RSI-1 — Bench KSR-oracle / Bench KSR-oracle

> Mảnh ghi chú cho agent giữ khoá phát hành (R3) — gộp vào CHANGELOG/README/report khi phát hành 0.2.96.

## VI

**Bench KSR-oracle (RSI-1, P0-7 Luồng R)** — hấp thụ spike SPK-RSI vào `tools/bench` rồi xoá
spike. Người dùng mô phỏng giờ là **oracle** (Trnka & McCoy 2008): tại mọi phím đọc chữ mờ,
chỉ nhận Tab khi `văn bản + chữ mờ` là tiền tố đúng của văn bản đích và hết ở ranh giới từ;
chữ mờ sai hiện ra = +1 nhiễu. Số công bố trung thực (thay 22% thổi phồng — 92% Tab chèn sai):
**KSR-oracle tổng 8,6%** (317/3676 phím) · bench 16 từ 5,7% · VI văn phòng **10,1%** · EN **4,4%**
· precision@ghost 13,4% · nhiễu 33,0/100 phím · chuỗi Tab nối sâu nhất 2. Corpus held-out VI/EN
dời sang `tools/bench/eval/`. Gate G19c chuyển sang KSR-oracle: KSR tổng/VI/EN giảm, NHIỄU TĂNG,
accepts giảm, vi phạm R2-173 hay câu lệch đích ⇒ đỏ; đột biến "luôn hiện chữ mờ sai" được đo
trong unit test `dream_rsi_gate_duot_bien_ghost_sai` (KSR giảm + noise tăng ⇒ đỏ). Baseline
`perf/baseline-win.json` re-record. Spike `spikes/rsi-oracle/` xoá; CI `spikes.yml` bỏ 3 job
rsi-oracle/rsi-determinism (tất định đa kiến trúc đã kiểm chứng byte-identical trên 5
arch/OS — run #46029470340; PERF-2 sẽ chuyển vào ci-shared). Tài liệu: `docs/specs/dream-rsi.md`
§5 viết lại, `docs/specs/rsi-oracle-spike.md` ghi số sau hấp thụ + ghi chú tất định, README
feature P10 (VI+EN) cập nhật cổng ngôn ngữ + gate mới.

## EN

**Bench KSR-oracle (RSI-1, P0-7 lane R)** — absorbed the SPK-RSI spike into `tools/bench` and
deleted the spike. The simulated user is now an **oracle** (Trnka & McCoy 2008): at every key it
reads the ghost and accepts Tab only when `text + ghost` is a correct prefix of the target text
and ends at a word boundary; a wrong ghost shown = +1 noise. Honest published numbers (replacing
the inflated 22% — 92% of Tab accepts were wrong): **overall KSR-oracle 8.6%** (317/3676 keys) ·
bench 16 words 5.7% · office VI **10.1%** · EN **4.4%** · precision@ghost 13.4% · noise 33.0/100
keys · deepest chained-Tab run 2. VI/EN held-out corpora moved to `tools/bench/eval/`. Gate G19c
switches to KSR-oracle: lower overall/VI/EN KSR, HIGHER noise, fewer accepts, an R2-173 invariant
break or a diverged sentence all fail; the "always show wrong ghosts" mutation is measured by the
`dream_rsi_gate_duot_bien_ghost_sai` unit test (KSR drop + noise rise ⇒ red). Baseline
`perf/baseline-win.json` re-recorded. Spike `spikes/rsi-oracle/` deleted; `spikes.yml` drops the 3
rsi-oracle/rsi-determinism jobs (cross-arch determinism proven byte-identical on 5 arch/OS — run
#46029470340; PERF-2 moves into ci-shared). Docs: `docs/specs/dream-rsi.md` §5 rewritten,
`docs/specs/rsi-oracle-spike.md` post-absorption numbers + determinism note, README P10 feature
(VI+EN) updated with the language gate and the new gate.
