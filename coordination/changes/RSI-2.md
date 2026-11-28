<!-- TASK: RSI-2 · commit: <SHA_CODE> · agent: claude-opus-c4c5a567 -->
# RSI-2 — Corpus đánh giá Dream RSI tách tập + gate chống rò rỉ (claude-opus-c4c5a567)

## VI

- **Tập đánh giá tách riêng `corpus/eval/`**: `vi_office.txt` 106 câu + `en_office.txt`
  55 câu văn phòng (viết MỚI cho dự án, không lấy nguồn ngoài; header SPDX
  GPL-3.0-or-later + blanket `**` trong `REUSE.toml` gốc đã phủ). Tách khỏi dữ liệu
  dự đoán: không câu nào trùng ≥ 6 token liên tiếp với `data/*phrases*.tsv`
  (chống KSR bị thổi phồng — bài học 22% của spike SPK-RSI).
- **Gate `cargo xtask eval-leak`** (`xtask/src/eval_leak.rs`, unit test kèm):
  n-gram 6 token (âm tiết Việt/từ Anh, lowercase) giữa mọi `corpus/eval/*.txt` và
  mọi `data/*phrases*.tsv`; ≥ 1 trùng = đỏ kèm tên câu + tệp nguồn. Đã cắm vào
  3 điểm: **preflight** (bước mới, 26→27), **repo-hygiene** (bước 5c, 12→13 checks),
  và **ngay trong `gen-vn-phrases`/`gen-en-phrases`** — sinh xong tự kiểm chồng lấn
  (acceptance "gen-*-phrases kiểm chồng lấn = 0" đúng nghĩa đen).
- **Bench chuyển sang corpus mới** (`tools/bench/src/main.rs`): `include_str!` từ
  `corpus/eval/vi_office.txt`/`en_office.txt` (cũ `tools/eval/heldout_*.txt` đã xoá
  ở commit 35c1ad67); thêm cờ `TEXTVN_BENCH_DEBUG=1` in câu lệch đích (chỉ chữ
  corpus eval — không phải phím người dùng, G8).
- **Số đo mới (corpus 161 câu)**: KSR-oracle tổng **7,3%** (687/9469) · bench 16 từ
  ×3 giữ 5,7% (18/315) · VI **7,9%** (524/6598) · EN **5,7%** (145/2556) ·
  precision@ghost 15,1% · nhiễu 30,8/100 (VI 31,2 · EN 30,2) · accepts 395 ·
  chars_saved 944 · chuỗi nối sâu nhất 2 (6 accepts nối tiếp) · diverged 0 ·
  invariant_breaks 0. Khối `dream_rsi` trong `perf/baseline-win.json` đã re-record
  với số này; **các khối timing giữ nguyên** (baseline re-record từ CI).
- **Phát hiện soạn corpus (gate G19c bắt được)**: câu "the new intern starts…"
  làm `diverged = 1` — từ `intern` là tiền tố ĐÚNG của từ dài hơn trong dict
  (`internal`): engine hiện ghost `al` trên từ ĐÃ ĐỦ và "al" bị chèn khi gõ dấu
  cách ⇒ mô phỏng không tái lập được. Đổi thành "accountant" ⇒ `diverged = 0`.
  Quy tắc soạn corpus mới (ghi vào `docs/specs/dream-rsi.md` §Bẫy): từ trong câu
  eval không được là tiền tố đúng của từ dài hơn trong dict. **Finding engine để
  follow-up** (ghost hoàn tất trên từ đã đủ — cần sửa `core/`, đang bị SIM-1 giữ
  `core/src/lib.rs`; hành vi đo được giữ nguyên vẹn, không giấu).
- **Sửa nợ G22 — data sửa tay ở 0.2.93**: `b7784cae` đã thêm 10 cặp THẲNG vào
  `data/vn_phrases.tsv` (chào bạn/chúc bạn/phát sinh/xin chào) thay vì thêm vào
  generator ⇒ regen sẽ mất 10 cặp. Đã đưa đủ 10 cặp vào `add_curated` của
  `xtask/src/gen_vn_phrases.rs` (8792 dòng, regen khớp generator — 2 dòng chỉ đổi
  THỨ TỰ alphabet vì bản sửa tay chèn sai chỗ sort).
- **Docs**: `docs/specs/dream-rsi.md` (corpus mới + bảng số RSI-2 + bẫy soạn
  corpus), `docs/release/release-process.md` (A0 26→27 + dòng gate eval-leak),
  `docs/10-shared/P0-7-TASKS-ARM-SF.md` (D2 27/27), repo-hygiene 13 checks.
- Không sửa CHANGELOG/README (tệp nóng R3 — release manager gộp ở bản phát hành kế).
- **R12**: notify `*` kèm thông tin commit fix CI đỏ (G13).

## EN

- **Dedicated evaluation set `corpus/eval/`**: `vi_office.txt` 106 sentences +
  `en_office.txt` 55 office sentences (written FRESH for the project, no external
  sources; SPDX header GPL-3.0-or-later + covered by the root `REUSE.toml` blanket
  `**`). Split away from prediction data: no sentence shares a run of ≥ 6
  consecutive tokens with `data/*phrases*.tsv` (anti-inflated-KSR — the 22%
  lesson from spike SPK-RSI).
- **Gate `cargo xtask eval-leak`** (`xtask/src/eval_leak.rs`, with unit test):
  6-token n-grams (VN syllables / EN words, lowercased) across every
  `corpus/eval/*.txt` and every `data/*phrases*.tsv`; any overlap = red with the
  sentence + source file named. Wired into 3 places: **preflight** (new step,
  26→27), **repo-hygiene** (step 5c, 12→13 checks), and **inside
  `gen-vn-phrases`/`gen-en-phrases`** — generation self-checks overlap
  (the "gen-*-phrases overlap = 0" acceptance taken literally).
- **Bench switched to the new corpus** (`tools/bench/src/main.rs`): `include_str!`
  from `corpus/eval/vi_office.txt`/`en_office.txt` (old `tools/eval/heldout_*.txt`
  deleted in commit 35c1ad67); added `TEXTVN_BENCH_DEBUG=1` to print the diverged
  sentence (eval-corpus text only — never user keystrokes, G8).
- **New measurements (161-sentence corpus)**: KSR-oracle total **7.3%** (687/9469)
  · bench 16 words ×3 unchanged at 5.7% (18/315) · VI **7.9%** (524/6598) ·
  EN **5.7%** (145/2556) · precision@ghost 15.1% · noise 30.8/100 (VI 31.2 ·
  EN 30.2) · accepts 395 · chars_saved 944 · longest chain 2 (6 chained accepts) ·
  diverged 0 · invariant_breaks 0. The `dream_rsi` block in
  `perf/baseline-win.json` re-recorded with these numbers; **timing blocks
  untouched** (baseline must be re-recorded from CI).
- **Corpus-authoring discovery (caught by the G19c gate)**: the sentence "the new
  intern starts…" made `diverged = 1` — `intern` is a PROPER PREFIX of a longer
  dictionary word (`internal`): the engine shows ghost `al` on the ALREADY-COMPLETE
  word and "al" gets inserted when space is typed ⇒ the sim can't reproduce the
  text. Changed to "accountant" ⇒ `diverged = 0`. New corpus-authoring rule
  (recorded in `docs/specs/dream-rsi.md` §Traps): eval words must not be a proper
  prefix of a longer dictionary word. **Engine finding left as follow-up** (ghost
  completion on complete words — fix belongs in `core/`, currently reserved by
  SIM-1; measured behaviour left fully visible, not hidden).
- **G22 debt fixed — hand-edited data from 0.2.93**: `b7784cae` added 10 pairs
  DIRECTLY into `data/vn_phrases.tsv` (chào bạn/chúc bạn/phát sinh/xin chào)
  instead of the generator ⇒ regeneration would drop them. All 10 pairs moved into
  `add_curated` in `xtask/src/gen_vn_phrases.rs` (8792 rows, regen matches the
  generator — 2 rows differ only in alphabetical ORDER because the hand edit
  broke the sort position).
- **Docs**: `docs/specs/dream-rsi.md` (new corpus + RSI-2 numbers table + authoring
  trap), `docs/release/release-process.md` (A0 26→27 + eval-leak gate row),
  `docs/10-shared/P0-7-TASKS-ARM-SF.md` (D2 27/27), repo-hygiene 13 checks.
- No CHANGELOG/README edits (hot files per R3 — release manager merges at next release).
- **R12**: notified `*` including the red-CI fix commit (G13).

## Kiểm thử / Testing

- `cargo test -p xtask` = **99 passed** (gồm test `eval_leak`: trùng ≥ 6 token báo
  rò rỉ, ≤ 5 token cụm ngôn ngữ tự nhiên miễn trừ, comment/dòng trống bỏ qua,
  không phân biệt hoa thường). `cargo test -p textvn-bench` = **9 passed**
  (deterministic 2 lượt cùng số + đột biến gate `dream_rsi_gate_duot_bien_ghost_sai`).
- Đột biến D3 (đã thử, có exit code): thêm câu
  "chính sách đổi trả áp dụng trong vòng bảy ngày kể từ ngày nhận hàng" (trùng
  đúng 1 dòng `data/vn_phrases.tsv`) vào `corpus/eval/vi_office.txt` ⇒
  `eval-leak` **exit 1** với thông báo nêu tên câu + tệp nguồn; khôi phục nguyên
  trạng ⇒ exit 0. Đột biến corpus "intern" ⇒ `diverged=1` (gate bench check đỏ)
  đã mô tả ở trên — cả hai đều do gate CHẶN được, không phải phát hiện thủ công.
- `cargo xtask eval-leak` trên cây hiện tại: **161 câu · 3 tệp dữ liệu · 0 rò rỉ**;
  chạy lại sau `gen-vn-phrases`/`gen-en-phrases` (tự kiểm trong generator) vẫn 0.
- `cargo xtask preflight` — **27/27 PASS exit 0** tại **worktree sạch** riêng
  (`D:\AppAI\TextVN_wt_rsi2` = HEAD + đúng diff RSI-2): fmt · clippy ·
  check-linux · clippy-linux `-D warnings` · `cargo test --workspace` ·
  verify-abi · sizes 20/532 · replay win/tsf/mac/linux · check-tables ·
  en-office · win-corpus · mac-corpus · version-sync · perf (`ime_key` +2,2% ·
  `parse_config` +5,4% — trong biên độ) · hygiene-apis · store-validate SKIP
  (E18) · iss-tabs · ascii-ps1 · no-python · **eval-leak (bước 23 mới)** ·
  regression-index · hygiene-docs/secrets · homebrew SKIP local (CI chặn thật).
  **Lý do dùng worktree**: cây chung đang chứa WIP LOOP-2
  (`core/src/post/predict.rs` 937+/340− của antigravity-gemini-37c7c6bc) làm
  test thời gian `v53_abnormal_mo_rong_khong_panic_khong_treo` đỏ local
  (2,54–2,88s vs trần 2,5s) — đã notify R13 cho LOOP-2; test đó không đụng
  đường code RSI-2 sửa và CI sẽ chặn thật trên commit sạch.
