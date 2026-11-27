# Spike SPK-RSI — KSR-oracle thật của dự đoán so với số Dream RSI 22% (`spikes/rsi-oracle`)

> Task gỡ rủi ro: **RSI-1** (bench KSR-oracle), RSI-2 (tách tập), RSI-3 (ngưỡng tin cậy), PERF-2 (tất định)
> — `../10-shared/P0-7-TASKS-ARM-SF.md` Luồng R, `../10-shared/P0-8-DREAM-RSI-LOOPS.md` §2.
> Spike code: `../../spikes/rsi-oracle/` · CI: `../../.github/workflows/spikes.yml` (`rsi-oracle` ×4 + `arm64x-run`, `rsi-determinism`)
> Ngày chạy: 2026-11-27 · Máy: Windows 11 x64 · `textvn-core` 0.2.94 + bản sửa R2-174.

## 1. Câu hỏi và tiêu chí đóng

- **Hỏi:** gate G19c và README đang công bố "Dream RSI 22%" (`tools/bench::bench_dream_rsi`: gõ nửa từ,
  có chữ mờ là nhấn Tab, **không kiểm Tab chèn đúng chữ không**). Số thật khi người dùng chỉ nhận chữ mờ
  ĐÚNG là bao nhiêu? Chữ mờ đúng bao nhiêu phần trăm, gây nhiễu bao nhiêu?
- **Đóng khi:** có (a) tái lập đúng số bench, (b) KSR-oracle + precision@ghost + nhiễu/100 phím + độ sâu
  chuỗi R2-173 trên corpus tách riêng VI/EN, (c) số học cá nhân sau 1 lượt, (d) danh sách lỗi phát hiện.
  ⇒ **Đã đạt** (a)–(d). Phần tất định đa kiến trúc chờ run CI `spikes.yml`.

## 2. Phương pháp

- **Văn bản đích** = gõ CÙNG chuỗi phím với dự đoán TẮT (engine tự làm oracle chính tả) ⇒ mọi khác biệt
  chỉ đến từ dự đoán. Câu VI đổi sang Telex kiểu dấu cuối từ (`được` → `dduwowcj`); câu không tái lập
  đúng chữ bị loại và in ra.
- **Người dùng "bench"** = chép đúng chính sách `bench_dream_rsi`; đếm thêm số Tab nhận SAI.
- **Người dùng "oracle"** (Trnka & McCoy 2008): tại MỌI phím đọc `peek_completion()` (đúng thứ adapter vẽ);
  chỉ nhấn Tab khi `văn bản + chữ mờ` là tiền tố văn bản đích và dừng ở ranh giới từ; chữ mờ hiện mà sai
  = +1 nhiễu. "xét được" = chỉ tính vị trí mà văn bản hiện tại còn là tiền tố đích (loại trạng thái Telex
  trung gian `vơ` trước phím dấu). Kiểm thêm bất biến R2-173 "chữ mờ == Tab chèn".
- **Corpus**: 16 từ × 3 của bench; `eval/heldout_vi.txt` 40 câu văn phòng; `eval/heldout_en.txt` 20 câu.
  Chồng lấn trigram với `data/*phrases*`: **VI 35/40, EN 19/20** ⇒ chưa phải tập kiểm tra sạch (RSI-2).

## 3. Kết quả (sau sửa R2-174)

| Corpus | Chính sách | KSR | Tab nhận | Tab SAI | precision@ghost (xét được) | nhiễu /100 phím | chuỗi nối (sâu nhất) |
|---|---|---|---|---|---|---|---|
| bench 16 từ ×3 | bench | **22,9%** (72/315) | 36 | **33 (92%)** | — | — | — |
| bench 16 từ ×3 | oracle | **5,7%** (18/315) | 21 | 0 | 20,2% (25,0%) | 32,0 | 0 (1) |
| VI văn phòng 40 câu | bench | 18,4% | 273 | **273 (100%)** | — | — | — |
| VI văn phòng 40 câu | oracle | **10,1%** (267/2636) | 71 | 0 | **6,4% (9,2%)** | **45,2** | 2 (2) |
| VI — học, lượt 1 / lượt 2 | oracle | 12,3% / 12,3% | 111 / 124 | 0 | 9,4% / 10,2% | 47,3 / 48,3 | 4 (2) |
| EN văn phòng 15 câu | bench | 17,4% | 59 | 50 (85%) | — | — | — |
| EN văn phòng 15 câu | oracle | **4,4%** (32/725) | 60 | 0 | 25,3% (26,8%) | 34,5 | 0 (1) |
| EN — học, lượt 1 / lượt 2 | oracle | 5,7% / **52,8%** | 60 / 101 | 0 | 23,4% / 48,1% | 39,3 / 32,2 | 38 (6) |

Bất biến "chữ mờ == Tab chèn" (R2-173/E79): **0 vi phạm** trên mọi corpus.

## 4. Kết luận

1. **Số 22% bị thổi phồng**: 92% số Tab mà bench tính là "tiết kiệm" chèn SAI từ (`tooi` nửa từ `to` + Tab →
   `town`; `lamf` → `law`); KSR thật trên cùng corpus là **5,7%**. Bộ đếm `DreamRsi` trong engine cũng cộng
   các lần nhận sai này (19,0%). ⇒ RSI-1 thay G19c bằng KSR-oracle; README công bố số mới trung thực.
2. **Nhiễu cao ở chế độ VI**: chữ mờ hiện ở ~45/100 phím nhưng chỉ ~9% đúng — phần lớn là từ tiếng Anh
   chen vào khi đang gõ âm tiết Việt (`gu` → `guys`, `ho` → `horse`, `ba` → `bars`). ⇒ RSI-3 (ngưỡng tin cậy) +
   **cổng ngôn ngữ**: không gợi ý từ EN cho tiền tố là âm tiết/tiền tố Việt hợp lệ khi không có ngữ cảnh EN.
3. **Học cá nhân chỉ có ở EN**: EN lượt 2 tăng 5,7% → 52,8% (thuộc câu), VI lượt 2 không tăng (12,3% →
   12,3%) ⇒ RSI-5 (n-gram cá nhân cho cụm Việt) là khoảng trống lớn nhất của "Dream".
4. **Lỗi thật bắt được — R2-174** (đã sửa cùng spike): bật `suggest.trigger = tab_and_space`, Space viết lại
   từ ĐÃ ĐÚNG: `chúng ta␠` → `chúng taw␠`, `diễn ra␠` → `diễn raw␠`, `tham gia␠` → `tham giai␠`,
   `kiểm tra␠` → `kiểm trao␠`, `confirm␠` → `confirmed␠` (từ Việt gõ ASCII thuần bỏ qua kiểm tra hợp lệ;
   từ EN đã trọn không được kiểm trong từ điển). Test `r2_174_*` trong `core/tests/text_prediction.rs`.
5. **Giới hạn Telex đã biết** (không phải lỗi mới): ở chế độ VI, `this`/`of`/`if`/`last` thành `thí`/`ò`/`ì`/
   `lát` (âm tiết Việt hợp lệ) — 5/20 câu EN bị loại khỏi corpus vì lý do này.
6. **Nợ G22 phát hiện**: hàm áp `Action` vào văn bản (`apply`) đang bị chép ở **12 tệp test** + `core/src/lib.rs`
   tests (spike là bản thứ 13). ⇒ task **SIM-1**: một bộ mô phỏng văn bản dùng chung (feature `sim` của
   `textvn-core` hoặc crate test-kit) cho test, bench, spike.

## 5. Tái lập

```bash
CARGO_TARGET_DIR=target/spikes cargo run --release --manifest-path spikes/rsi-oracle/Cargo.toml
SPK_RSI_DEBUG=1 CARGO_TARGET_DIR=target/spikes cargo run --release --manifest-path spikes/rsi-oracle/Cargo.toml
```

G8: spike chỉ chạy trên corpus của chính nó; chế độ debug in chữ của corpus spike, không phải phím người dùng.
