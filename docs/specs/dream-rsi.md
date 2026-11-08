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
| 4 | Cycle candidate (không cộng `accepts` — cả vòng là một lượt nhận) | Tab/Ctrl+Space khi `last_predict` mở | `cand_mới.chars − max(glyphs_cand_cũ, glyphs_dài_nhất_đã_chèn)` (saturating — duyệt dài → ngắn → dài không tính hai lần) | `try_cycle_predict` |
| 5 | Next-phrase | Ctrl+Space/Space sau ngữ cảnh nhiều từ | nguyên số glyph cụm engine chèn (`phrase_glyphs`) | `try_predict_next_phrase` |

Mọi hook đều `saturating_add` — không panic, không trôi lùi.

**Từ chối (R2-147):** Backspace/Esc ngay sau khi chèn gỡ phần vòng duyệt đó đã cộng
(`PredictCycle::rsi_accepts`/`rsi_saved`) — số đo chỉ gồm chữ người dùng giữ. Từ/cụm được học ở
phím kế tiếp (`settle_predict`), không phải lúc chèn; `reset()` vẫn học (đường fail-open khi áp
ứng viên lỗi được chấp nhận là rủi ro nhỏ: `reset()` coi như người dùng giữ ứng viên — đổi focus,
click sau khi nhận là đường phổ biến hơn nhiều so với áp lỗi). Duyệt vòng chỉ cộng `chars_saved`;
`accepts` đếm một lần cho cả vòng.

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
  → tray. `config::parse_style_drain` **bỏ qua khối `rsi` an toàn** (serde default — không
  `deny_unknown_fields`): merge words/corrections không chết vì metrics. macOS/Linux không có
  đường tray: `ime_style_profile_persist` ghi thẳng vào `config.json` lúc mất focus (R2-152).
- Counters `DreamRsi` là **cumulative trong phiên** engine (không reset khi
  drain) — rate ổn định, phù hợp đối chiếu giữa các bản.
- **Bền vững qua phiên (0.2.74):** drain còn mang khối `rsi_delta` = phần tăng **kể từ lần drain
  trước** (engine giữ `rsi_reported`, chốt khi `style_clear_pending`; peek không tiêu thụ). Host cộng
  dồn `rsi_delta` (KHÔNG cộng khối `rsi`) vào `config.style_profile.rsi` → Cài đặt hiển thị "Dream RSI:
  tiết kiệm X% số phím (N ký tự)" cùng định dạng 3 OS. Windows: tray gom drain chỉ-có-số-đo và ghi tối
  đa 1 lần/60 s, **không** phát `ConfigReload` (reload reset từ đang gõ); drain có từ/cặp thật ghi ngay
  kèm số đo đang gom. Giới hạn đã biết: Backspace hoàn tác một accept đã được báo (R2-147) làm
  số hiện tại thấp hơn số đã giao → delta kế tiếp bị thiếu 1 accept (sai lệch nhỏ, không âm).

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
  có ghost: **deterministic** (debug = release, 3 OS cùng số). Kết quả 0.2.69:
  saved 72/318 phím · **savings-rate 22%** · accepts 36 · engine_chars_saved 57.
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
