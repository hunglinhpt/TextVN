<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19)

> **Trạng thái:** Implemented 0.2.67 · **G19** (docs/00-WORKFLOW.md): mọi agent
> code bộ gõ buộc review & implement Dream RSI. User rule 2026-11-07: "gợi ý
> phải luôn tiến hóa… áp dụng dream rsi… áp rule cho mọi agent code hệ thống
> này buộc phải review implement áp dụng dream rsi cho bộ gõ này."

## 1. Dream RSI là gì

RSI (Repetitive Strain Injury) = chấn thương do gõ phím lặp lại. Sứ mệnh của
bộ gõ thông minh là **giảm số phím user phải gõ**. Dream RSI là bộ metrics
biến sứ mệnh đó thành **số đo kiểm chứng được**:

| Metric | Ý nghĩa | Nguồn đếm |
|---|---|---|
| `keys_typed` | Số phím gõ chữ **thật** của user | `Engine::key()` — key-down, không injected, không modifier, không chord, `printable().is_some() && !is_ctrl_space()` |
| `accepts` | Số lần user **nhận** prediction | Mỗi accept-point (bảng §2) |
| `chars_saved` | Tổng ký tự user **không phải gõ** | Công thức theo từng đường accept (bảng §2) |
| `savings_rate_pct()` | `chars_saved / (chars_saved + keys_typed) × 100` | Tính suy ra, saturating (không panic, không chia 0) |

**"Dream"** = mục tiêu tiến hóa: bộ gõ tự học (P7) ngày càng đoán đúng ⇒
savings-rate trên corpus **tăng dần qua các version**. Một tính năng mới
làm rate giảm = hồi quy chất lượng gợi ý = **không merge** (G19c).

**G8:** counters tổng hợp — không phím, không text, không từ trong metrics.

## 2. Các accept-point + công thức `chars_saved`

| # | Đường accept | Trigger | `chars_saved +=` | Vị trí |
|---|---|---|---|---|
| 1 | VN completion | Tab/Ctrl+Space sau Telex-transform (word active) | `completion.chars − typed.chars` (typed = `word.display`) | `try_vn_complete` |
| 2 | EN/style-word completion | Tab (word chưa active — ASCII thuần) | `completion.chars − typed.chars` | `try_english_complete` |
| 3 | Space-accept | Space với `suggest_space_complete` | như #2 (space user đã gõ không đếm là saved) | #2 với `append` |
| 4 | Cycle candidate | Tab/Ctrl+Space khi `last_predict` mở | `cand_mới.chars − glyphs_đã_chèn_của_cand_cũ` (saturating — cycle ngắn hơn không làm trôi lùi) | `try_cycle_predict` |
| 5 | Next-phrase | Ctrl+Space/Space sau ngữ cảnh nhiều từ | nguyên số glyph cụm engine chèn (`phrase_glyphs`) | `try_predict_next_phrase` |

Mọi hook đều `saturating_add` — không panic, không trôi lùi.

### Quy tắc khi THÊM đường accept mới (G19a)

1. Đếm `accepts += 1` ngay tại điểm Outcome được tạo (sau guard `MAX_TEXT`,
   trước khi mở cycle-state).
2. Đếm `chars_saved` theo đúng nghĩa "số ký tự user không phải gõ":
   - completion → candidate − phần user đã gõ;
   - cycle → chênh lệch glyph giữa candidate mới và cũ;
   - cụm mới chèn → nguyên glyph cụm (gồm space engine tự thêm).
3. KHÔNG đếm space user bấm làm `chars_saved` (nó nằm trong `keys_typed`).
4. Thêm case vào `core/tests/dream_rsi_100.rs` với số chính xác từng bước.

## 3. Luật `keys_typed` (G19b)

Đếm ĐÚNG khi: `key_down && !is_injected && !is_modifier && (!is_chord || is_ctrl_space) && printable().is_some() && !is_ctrl_space()`.
Không đếm: key-up, phím engine tự bơm (chống loop P0-3 §6), Shift/Ctrl/Alt/
Win/Caps đơn, chord hệ thống (Ctrl+V…), phím điều hướng (`printable == None`).
Lý do: rate phải phản ánh **gõ chữ thật** — đếm sai là metrics lying.

## 4. Drain + persist

`style_drain_json()` gắn kèm khối:

```json
"rsi":{"keys_typed":N,"accepts":N,"chars_saved":N,"savings_rate_pct":N}
```

- Adapter (TSF Windows) drain cùng words/corrections → IPC `Message::StyleUpdate`
  → tray. `DrainShape` (tray `style_merge.rs`) **bỏ qua khối `rsi` an toàn**
  (serde default — không `deny_unknown_fields`): merge words/corrections không
  chết vì metrics.
- Counters `DreamRsi` là **cumulative trong phiên** engine (không reset khi
  drain) — rate ổn định, phù hợp đối chiếu giữa các bản.

## 5. Gate + bench (G19c)

- **Gate:** `core/tests/dream_rsi_100.rs` — 12 test / 100 case sweep:
  - `d100_sweep_tab_accept_tich_luy` — 100 case accepts/keys/saved tích lũy
    không trôi + drain JSON khối `rsi` khớp snapshot;
  - 4 accept-path exact-formula tests (VN transform + cycle "ddk"→"đk",
    style-word Tab, Space-accept, EN "rele"→"release", next-phrase
    "cộng hòa "→"xã hội chủ nghĩa Việt Nam");
  - luật `keys_typed` (loại key-up/injected/modifier/chord/điều hướng);
  - saturating biên u64; engine mới = 0; G8 (Debug không lộ từ);
  - 100 phím tay không accept → rate 0 (metrics trung thực).
- **Bench:** `cargo run -p textvn-bench --release -- dream-rsi` — corpus văn
  phòng 16 từ (8 VN Telex + 8 EN) × 3 vòng, mô phỏng user gõ nửa từ → Tab khi
  có ghost: **deterministic** (debug = release, 3 OS cùng số). Kết quả 0.2.67:
  saved 69/318 phím · **savings-rate 21%** · accepts 33 · engine_chars_saved 57.
  Khối `"dream_rsi"` trong `perf/baseline-win.json` (re-record khi phát hành)
  — `check` fail nếu savings-rate **hoặc** accepts giảm so baseline (G19c);
  baseline cũ thiếu khối → bỏ qua kèm ghi chú re-record.

### Bẫy đã ghi nhận (đừng lặp)

- **Path VN cần Telex-transform:** tiền tố ASCII thuần ("kha") không làm
  word active → `try_vn_complete` bỏ qua → Tab đi `try_english_complete`
  (không mở cycle). Muốn test cycle thật: dùng transform ("ddk" → "đk").
- **`peek_completion()` trả GHOST DELTA** (`insert[delete..]`) — không phải
  candidate đầy đủ; delta == chars_saved cho Tab-accept lowercase.
- **`restyle(Old)` dịch dấu về âm tiết cuối** nếu cluster cuối không có dấu
  ("hòaianh" → "hoaiành") — từ test đa âm tiết phải có dấu ở âm cuối hoặc
  đuôi ASCII thuần.
- **`ddd` → "dd"** (bấm lại `d` gỡ đ + literal) — không dùng làm prefix test.
- **Candidate dictionary thắng consider-shortest** ("kqu" → "kqueue" tech
  term) — sweep phải assert counters động theo ghost, không hardcode từ.

## 6. Audit view (G19d)

Mỗi vòng audit 3× phải có 1 view Dream RSI:
1. Mọi accept-point đều có hook? (grep `accepts` vs các `return Some(Outcome)` prediction)
2. `chars_saved` đúng nghĩa (không đếm space user, không âm)?
3. `keys_typed` không đếm phím engine inject?
4. Metrics không lộ nội dung (G8) — Debug/JSON chỉ counter?
5. Savings-rate bench không giảm so baseline?
