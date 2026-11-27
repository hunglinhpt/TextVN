# P0-8 — Dream RSI 2.0 · kiến trúc vòng lặp · vòng nghiệp vụ (hiệu năng + chất lượng mọi tính năng)

> Trạng thái: **nghiên cứu + kế hoạch** (2026-11-27). Triển khai sau/song song tái cấu trúc
> `textvn-session` (P0-5/P0-6/P0-7). Luật liên quan: G19 (Dream RSI), G22 (không duplicate — mọi vòng
> lặp nằm trong framework chung), G23 (một bản phát hành, sửa theo lớp). Task: P0-7 §5 "Luồng R".

## 1. Tóm tắt

1. **Số đo hiện tại chưa đáng tin**: bench G19c mô phỏng người dùng **nhấn Tab hễ có chữ mờ, không kiểm
   chữ mờ có đúng từ định gõ** ⇒ gợi ý SAI cũng được tính là "tiết kiệm"; tính năng hiện nhiều chữ mờ
   sai có thể làm số **tăng** và qua gate. Corpus chỉ 16 từ rời × 3 vòng. ⇒ Việc đầu tiên: **bench có
   oracle** + corpus thật, tách tập kiểm tra (RSI-1, RSI-2).
2. **Học cá nhân chỉ có tín hiệu dương**: từ/cặp + count, không thời gian/decay, không phạt khi người
   dùng từ chối, không ngữ cảnh n-gram cá nhân ⇒ Dream (tự tiến hoá) bị giới hạn. Nâng lên mô hình nội
   suy **base n-gram làm trơn + mô hình cá nhân có decay + cache recency + phạt từ chối**, kèm **ngưỡng
   tin cậy** để chỉ hiện chữ mờ khi lợi > hại (RSI-3…RSI-6).
3. **Đường nóng**: đo thật (release, máy dev): `prefix 'con'` **77,8 µs**, `peek_completion` hồ sơ đầy
   **80,8 µs**/phím, trong khi `ime_key` ~0,6 µs ⇒ dự đoán chiếm >99% chi phí mỗi phím. Chỉ mục tiền tố
   top-k + cache chữ mờ liên tục ⇒ mục tiêu ≤ 5 µs (LOOP-2, LOOP-3).
4. **Kiến trúc vòng lặp 2 tốc độ** trong `textvn-session`: đường nhanh mỗi phím (ngân sách cứng) + đường
   chậm lúc rảnh/ranh giới (assist, chấm lại ứng viên, "giấc mơ" hợp nhất hồ sơ) có cắt ngân sách và huỷ
   khi có phím mới (LOOP-1).
5. **Vòng nghiệp vụ**: một bảng điểm chất lượng chung cho MỌI tính năng (`xtask quality-score`): Dream
   RSI 2.0, trợ lý (recall/FPR), khôi phục tiếng Anh, Telex/VNI, tự viết hoa, độ trễ, crash — lưu xu
   hướng theo bản, gate "không tụt" trước mỗi phát hành (B4b/G23) (BIZ-1…BIZ-3).

## 2. Hiện trạng đo được (2026-11-27, 0.2.94)

| Hạng mục | Hiện trạng | Nguồn |
|---|---|---|
| Đếm Dream RSI | đúng & trung thực ở mức sự kiện: `keys_typed` loại phím ảo, mọi đường nhận có hook, Backspace gỡ (R2-147), chuỗi R2-173 | `docs/specs/dream-rsi.md`, `core/tests/dream_rsi_100.rs`, `r2_173_prediction_chain.rs` |
| Bench G19c | 16 từ (8 VN Telex + 8 EN) × 3 vòng; gõ nửa từ, **có chữ mờ là Tab** (không đối chiếu đích) ⇒ 72/318 phím, 22% | `tools/bench/src/main.rs` `RSI_WORDS`, `bench_dream_rsi` |
| Tách tập | không — bench dùng từ có trong bảng dự đoán (rò rỉ dữ liệu) | `data/*phrases*.tsv`, `en_freq3000.txt` |
| Hồ sơ cá nhân | ≤ 200 từ + ≤ 50 cặp sửa, count; recency suy từ thứ tự danh sách; gộp tuyến tính O(n) | `config/src/lib.rs` `StyleProfileSettings::merge_deltas`, `core/src/post/predict.rs` `recency_boost` |
| Tín hiệu âm | chỉ gỡ số đo khi Backspace/Esc ngay sau nhận; không hạ hạng ứng viên | `settle_predict(false)` |
| Độ trễ dự đoán | prefix VI 8,4 µs; **prefix 'con' 77,8 µs**; EN 5,9 µs; next-phrase 1–3 µs; **peek đầy 80,8 µs** | `core/tests/prediction_perf.rs --release` |
| KPI theo tính năng | trợ lý có `xtask assist-score` (recall/FPR) riêng; dự đoán chỉ một số tổng | `xtask/src/assist_tools.rs` |

## 3. Nghiên cứu — cái đã được chứng minh

- **Keystroke savings phải đi kèm mô hình người dùng mô phỏng rõ ràng** (Trnka & McCoy, ACL 2008): công
  thức tiết kiệm phím đơn thuần không đủ; cần định nghĩa người dùng nhận gợi ý khi nào. Nghiên cứu AAC cùng
  nhóm: ~50% keystroke savings nhưng **không nhanh hơn** do chi phí dò danh sách gợi ý lớn ⇒ phải đo cả
  **nhiễu** (gợi ý sai), không chỉ tiết kiệm.
- **Bàn phím di động hiện đại**: mô hình chính + mô hình cá nhân (lịch sử người dùng) bổ sung; dự đoán từ
  trạng thái n-gram bậc cao nhất khớp ngữ cảnh (Gboard). **Nội suy** mô hình nền với mô hình người dùng nhỏ
  tốt nhất khi dữ liệu người dùng ít; cache ưu tiên từ gõ gần đây (PPM) cải thiện xếp hạng. Cá nhân hoá
  trên thiết bị (DeepType) để giữ riêng tư.
- **Áp vào TextVN (offline, G8)**: không federated learning (không mạng — `deny.toml` cấm crate mạng); học
  100% trên máy; đánh giá trên corpus công khai; chỉ số người dùng là counter.

## 4. Dream RSI 2.0 — KPI (bảng điểm)

| KPI | Định nghĩa | Gate |
|---|---|---|
| **KSR-oracle** | tiết kiệm phím khi người dùng mô phỏng CHỈ nhận chữ mờ nếu nó là tiền tố đúng của phần còn lại của văn bản đích (từ/cụm), xét tại MỌI phím (không chỉ nửa từ); gồm chuỗi R2-173 | không giảm so bản trước (thay G19c cũ) |
| **Precision@ghost** | số chữ mờ ĐÚNG / số chữ mờ đã hiện | không giảm |
| **Noise rate** | chữ mờ SAI / 100 phím (chi phí nhận thức) | không tăng quá ngưỡng (đặt sau RSI-1 có số gốc) |
| **Chain depth** | số cụm nối tiếp đúng trung bình mỗi lần bắt đầu chuỗi | theo dõi |
| **Top-1 theo loại ngữ cảnh** | VN hoàn tất / EN hoàn tất / next-phrase / chuỗi | theo dõi + gate từng loại |
| **Độ trễ** | `session_event` + dự đoán p50/p99 mỗi target (x86_64, aarch64 Win/Linux/mac) | ngân sách P0-6 §9, LOOP-2/3 |
| **Học cá nhân** | KSR-oracle sau khi "người dùng" gõ N câu luyện (mô phỏng thói quen riêng) — đo đúng khả năng Dream | phải tăng theo N; không giảm so bản trước |

Mô phỏng người dùng (RSI-1): với mỗi câu đích, sau mỗi phím engine xử lý, nếu `peek_completion()` là tiền
tố của phần còn lại (sau chuẩn hoá chữ hoa/dấu cách) ⇒ Tab (1 phím) và tiến con trỏ đích; không thì gõ phím
kế tiếp. Chữ mờ hiện mà không khớp ⇒ +1 nhiễu. Ghi cả KSR khi người dùng "lười" (chỉ nhìn tại ranh giới
từ) để bắt được trần tiết kiệm thực tế.

## 5. Vòng nghiệp vụ ("giấc mơ" tiến hoá có kiểm soát)

```
 Đo (quality-score, corpus công khai, tách tập) → Phân tích (miss report: từ/cụm gõ tay hết mà không
 có gợi ý đúng; gợi ý sai hay gặp) → Cải tiến (dữ liệu bảng cụm, mô hình, ngưỡng) → Gate (không tụt
 mọi KPI, G19/G23) → Phát hành gộp (B4b) → Lưu xu hướng (perf/quality/<version>.json) → lặp
```

- **Tách tập**: corpus đánh giá (`corpus/eval/`) KHÔNG được dùng để sinh bảng dự đoán; `xtask gen-*-phrases`
  kiểm chồng lấn = 0 (rò rỉ ⇒ đỏ).
- **Nguồn**: Tatoeba (đã có, CC BY 2.0 FR — ghi công), mẫu văn phòng `en_office_phrases.tsv` (sinh), corpus
  câu tiếng Việt công khai có giấy phép tương thích (chọn ở RSI-2, ghi `REUSE.toml`).
- **Miss report**: tệp máy đọc + bảng top-50 cụm bỏ lỡ ⇒ đầu vào cho dữ liệu bản sau (không nội dung người dùng).
- **Xu hướng**: `perf/quality/history.jsonl` mỗi bản; README hiển thị KSR-oracle thật (G18), không dùng số 22% cũ.

## 6. Lộ trình mô hình (chất lượng)

| Bước | Nội dung | Điều kiện nhận |
|---|---|---|
| M1 | Bench oracle + corpus tách tập (RSI-1/2) | số gốc KSR-oracle/precision/noise công bố |
| M2 | **Chấm điểm thống nhất**: `score = λ·P_base(w|ngữ cảnh) + (1−λ)·P_user(w|ngữ cảnh) + recency` — base: n-gram làm trơn (âm tiết VN bigram/trigram, từ EN), user: count có **decay theo thời gian** (half-life ~30 ngày, lưu `last_used` theo ngày) + cache 12 mục gần nhất; λ điều chỉnh theo lượng dữ liệu người dùng | KSR-oracle ↑, precision không ↓ |
| M3 | **Ngưỡng tin cậy**: chỉ hiện chữ mờ khi `P(đúng)·ký tự tiết kiệm > (1−P)·chi phí nhiễu`; hiệu chuẩn trên tập kiểm tra | noise ↓ mà KSR không ↓ |
| M4 | **Phạt từ chối**: Backspace/Esc ngay sau nhận, hoặc chữ mờ bị bỏ qua ≥ k lần cùng ngữ cảnh ⇒ penalty có decay cho (ngữ cảnh, ứng viên) — lưu trong hồ sơ cục bộ (đã là nội dung người dùng, không rời máy) | precision ↑ |
| M5 | **N-gram cá nhân** (bigram cụm người dùng) + **"giấc mơ" hợp nhất** ở đường chậm/tray: tỉa, gộp, nâng cụm dùng nhiều thành mục cụm, cập nhật decay — ngoài đường nóng | KSR sau N câu luyện ↑ |
| M6 | (nghiên cứu, có gate) bộ chấm lại nơ-ron nhỏ CHỈ ở ranh giới từ, bất đồng bộ, n-gram dự phòng; chỉ nhận khi KSR ↑ đáng kể, p99 đạt ngân sách, không crate ngoài danh sách `deny.toml` | chỉ sau M1–M5 |

## 7. Kiến trúc vòng lặp (hiệu năng) — trong `textvn-session` (G22)

| ID | Thiết kế | Mục tiêu đo |
|---|---|---|
| LOOP-1 | **Hai tốc độ**: `event()` đường nhanh (biến đổi + dự đoán rẻ) ngân sách cứng; `poll_idle(budget_us)` đường chậm hợp tác (assist, chấm lại, hợp nhất hồ sơ) — adapter gọi khi rảnh/timer; mỗi việc mang **số thế hệ**, phím mới ⇒ huỷ/bỏ kết quả cũ; không thread trong core | đường nhanh p99 ≤ `ime_key` p99 + 10 µs |
| LOOP-2 | **Chỉ mục tiền tố top-k** sinh lúc build (`xtask`): mảng sắp xếp/FST + top-k mỗi nút tiền tố ⇒ truy vấn O(độ dài tiền tố + k) thay quét | `prefix 'con'` 77,8 → ≤ 5 µs |
| LOOP-3 | **Cache chữ mờ liên tục**: phím kế trùng ký tự đầu chữ mờ ⇒ chữ mờ mới = phần còn lại (không tính lại); sai/đổi caret ⇒ huỷ | `peek` trung bình 80,8 → ≤ 3 µs |
| LOOP-4 | `predict(&state)` thuần thay việc clone engine để chạy thử Tab | 0 clone ở đường nóng |
| LOOP-5 | 0 cấp phát đường nóng (arena cố định), bench đếm cấp phát | 0 alloc/KeyDown |
| LOOP-6 | Baseline theo target (PERF-1) + so trước/sau mỗi PR trên CI ARM/x86 | không hồi quy |

## 8. Vòng chất lượng cho MỌI tính năng (một bảng điểm)

| Tính năng | KPI | Nguồn/công cụ hiện có → bổ sung |
|---|---|---|
| Dự đoán (VN/EN/cụm/chuỗi) | KSR-oracle, precision, noise, độ trễ | RSI-1 (mới) |
| Trợ lý ngữ pháp | recall, FPR, kappa | `xtask assist-score` → gộp |
| Khôi phục tiếng Anh | đúng/sai trên `en_words10k` + từ Việt thật | test `r2_172_english_words_untouched` → thành KPI |
| Telex/VNI/VIQR | khớp oracle x-unikey (423/423 từ thông dụng) | B30 oracle → corpus KPI |
| Tự viết hoa, Math, emoji | tỷ lệ đúng trên corpus | test hiện có → KPI |
| Ổn định | panic.log = 0 trên e2e, `sandbox_e2e`, fuzz | doctor/E70 |
| Độ trễ | p50/p99 mỗi target | PERF-1 |
| Tương thích | e2e xanh trên toàn ma trận Windows/Linux distro/desktop/ARM (P0-5 §9b) | COMPAT-1…3 |
| Chống tái diễn | 100% mã lỗi đã sửa có test (`regression-index`) | REG-1/REG-2 |

`cargo xtask quality-score` **(mới, BIZ-1)** chạy tất cả, ghi `perf/quality/<version>.json`, so bản trước;
được gọi trong preflight (nhanh) và B4b (đủ).

## 9. Ràng buộc

- G8: không telemetry, không gửi gì ra ngoài; bảng điểm chạy trên corpus công khai; số đo người dùng chỉ là
  counter trong máy (Cài đặt hiển thị như hiện nay).
- G19: mọi đường nhận mới có hook; KSR-oracle thay số 22% cũ trong gate và README (công bố số mới trung
  thực, kể cả nếu thấp hơn).
- G22: mọi vòng lặp, mô hình, ngưỡng ở core/session; adapter không có logic dự đoán riêng.

## 10. Task (đưa vào P0-7 §5 "Luồng R")

| ID | Cỡ | Dep | Việc |
|---|---|---|---|
| RSI-1 | M | — | bench KSR-oracle + precision + noise (mọi phím, gồm chuỗi), thay `bench_dream_rsi`; baseline mới; G19c dùng KSR-oracle |
| RSI-2 | M | RSI-1 | `corpus/eval/` câu VN/EN công khai (giấy phép + REUSE), tách tập, kiểm rò rỉ trong `gen-*-phrases` |
| RSI-3 | L | RSI-2, SF-4 | chấm điểm nội suy base n-gram + cá nhân có decay + recency (M2) |
| RSI-4 | M | RSI-3 | ngưỡng tin cậy hiệu chuẩn (M3) |
| RSI-5 | M | RSI-3 | phạt từ chối có decay (M4) |
| RSI-6 | L | RSI-3, LOOP-1 | n-gram cá nhân + hợp nhất "giấc mơ" ở đường chậm (M5) |
| LOOP-1 | L | SF-3 | `poll_idle` + thế hệ/huỷ trong session |
| LOOP-2 | M | — | chỉ mục tiền tố top-k sinh lúc build |
| LOOP-3 | S | SF-4 | cache chữ mờ liên tục |
| LOOP-4/5 | M | SF-4 | dự đoán thuần không clone; 0 cấp phát |
| BIZ-1 | M | RSI-1 | `xtask quality-score` + `perf/quality/` + gọi trong preflight/B4b |
| BIZ-2 | S | BIZ-1 | miss report |
| BIZ-3 | S | BIZ-1 | README/CHANGELOG hiển thị xu hướng KPI theo bản (G18) |

Nguồn: Trnka & McCoy, "Evaluating Word Prediction: Framing Keystroke Savings", ACL-08 HLT Short Papers
(aclanthology.org/P08-2066); Hard et al., "Federated Learning for Mobile Keyboard Prediction" (arXiv
1811.03604 — mô hình Gboard: n-gram + mô hình cá nhân); nghiên cứu nội suy mô hình nền/người dùng cho bàn
phím điện thoại và PPM/cache (tổng hợp kết quả tìm kiếm 2026-11-27); DeepType (UbiComp 2019 — cá nhân hoá
trên thiết bị). Số đo nội bộ: `core/tests/prediction_perf.rs` (release, 2026-11-27).
