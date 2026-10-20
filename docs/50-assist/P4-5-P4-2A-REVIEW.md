# P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả

- **Ngày:** 2026-10-21 · **Trạng thái:** Đã merge (P4-2a ✅) · **Phần:** 4 — P4
- **Đối tượng:** `core/src/lib.rs` — thêm buffer câu `Engine.sentence` (Unicode) + `Engine::glyph_len_of` + `Engine::sentence_tail`, gom **21 call site** buffer vào **1 helper** `record(Tail)`.
- **Cross-ref:** `P4-TASKS.md` (AS-020) · `P4-0-MASTER-PLAN.md` §3.1 (D24/D26/D27/D31) · `docs/adr/README.md` (ADR-013) · `P4-3-SPIKES.md` (S2)

---

## 0. Kết quả

| Mục | Kết quả |
|---|---|
| Hành vi gõ | **không đổi** — bằng chứng **behavior hash** byte-identical (4 charset × 4 method × 6 seed × 600 phím × 3 context, có macro/emoji/suggest/quick-telex/backspace/escape) |
| Test `core` | **167 pass / 0 fail** (157 cũ + 10 mới) |
| clippy (`-D warnings`) / fmt | **0 warning** / sạch |
| Dependency / I/O | **không thêm** dep · không I/O · không log nội dung (S2) |
| API công khai thêm | `glyph_len_of(&[char]) -> usize` (D27) · `sentence_tail(max) -> String` |

## 1. Thiết kế (vòng 1 — SA)

- `enum Tail { Replace { delete, glyph, unicode }, Clear, Pop }` — **1 helper** `record()` là nơi **duy nhất** sửa `recent` / `sentence` / `sentence_widths` ⇒ không còn "chỗ sửa buffer trực tiếp" (đúng tinh thần ADR-013).
- **2 buffer tách bạch**: `recent` (glyph — thứ app thật nhận) vs `sentence` (char Unicode — thứ trợ lý ngữ pháp đọc). Vì 1 chữ có thể = 2 glyph (VNI-Windows / Unicode tổ hợp) nên thêm `sentence_widths: Vec<u8>` (bề rộng glyph của từng ký tự) ⇒ xoá đúng số ký tự Unicode khi app xoá N glyph.
- `Tail::Replace` nhận **cả** `glyph` (đúng chuỗi app nhận) lẫn `unicode` ⇒ giữ `recent` **byte-identical** với cũ ở 3 call site mà app nhận chuỗi **không encode** (restore-EN, VN-complete, ESC) — nếu helper tự `emit()` thì `recent` sẽ lệch document dưới TCVN3/VNI ⇒ **đổi hành vi gõ**.
- Site macro (F65): `recent` vẫn cắt `MAX_TEXT` **sau encode** (đúng như cũ); `sentence` cắt theo **số glyph đã vào document** ⇒ khớp document.

## 2. Vòng 2 — Audit (license / privacy / AV)

- Không dep mới · không I/O · không log · không `unsafe` ⇒ không vi phạm S2/P2/`antivirus-false-positive.md` §8.
- `sentence` giữ **text người dùng trong RAM** (nằm trong TIP in-process) ⇒ rủi ro đã được phân tích ở D26/D31; **giới hạn**: ≤ 256 ký tự ≈ 1 KB/process (D31) — chấp nhận được, đo lại trong TIP ở P4-2 (F61).
- Không có mã của bên thứ ba ⇒ không phát sinh nghĩa vụ attribution.

## 3. Vòng 3 — PO/QA (chất lượng & rủi ro)

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-73** | major | **Thiếu bất biến 3 buffer**: `sentence` và `sentence_widths` có thể lệch độ dài → hỏng **im lặng** (không panic) | ✅ Đã sửa: `debug_assert_eq!(self.sentence.len(), self.sentence_widths.len())` trong `record()` + test `sentence_widths_stay_in_sync` |
| **Fassist-74** | minor | `Tail::Pop` xoá **1 char Unicode** dù app chỉ xoá **1 glyph** → nếu chữ cuối 2-glyph thì `sentence` mất nhiều hơn thực tế (≤1 ký tự) | Chấp nhận (hướng **conservative**: mất thừa 1 ký tự còn hơn giữ sai) — ghi rõ trong doc-comment + test `sentence_backspace_pops_one_unicode_char` |
| **Fassist-75** | minor | ADR-013 ghi "buffer câu chỉ bật khi `assist.mode != off`" nhưng code **luôn bật** (D31) → doc lệch code | ✅ Cập nhật ADR-013: luôn bật, cap 256; gate theo `assist.mode` để làm ở P4-2 khi config có mục đó |
| **Fassist-76** | minor | `sentence_tail()` trả `String` (copy) → adapter C gọi mỗi trigger sẽ cấp phát | Ghi nhận: P4-2 nên thêm biến thể trả `&[char]` (view không copy) |
| **Fassist-77** | minor | `Tail::Replace` nhận 3 tham số (`delete`/`glyph`/`unicode`) → dễ **lệch cặp** khi thêm call site mới (site 523 cố ý lệch vì cắt `MAX_TEXT`) | Không thể assert `glyph.len() == unicode.len()` (site 523) → ghi rõ trong doc-comment + review checklist: **mọi call site mới phải kèm test** |

## 4. Bằng chứng (để agent sau không phải đo lại)

| Kiểm tra | Kết quả |
|---|---|
| `cargo test -p textvn-core -j 2` | 167 pass / 0 fail (144 unit + 2 + 9 + 3 + 3 + 6 integration) |
| `cargo clippy -p textvn-core --all-targets -j 2 -- -D warnings` | 0 warning |
| `cargo fmt -p textvn-core -- --check` | sạch |
| Behavior hash (script tạm, đã xoá) | `1025fb56feb4ca7a` **trước** = **sau** (byte-identical) |

## 5. Còn lại cho P4-2

- Dùng `Engine::sentence_tail(max)` làm nguồn câu cho `assist` (không đọc nội dung app ⇒ đúng D8) và `Engine::glyph_len_of()` để biết số ký tự cần xoá khi áp gợi ý (D27).
- FFI `ime_assist_*` (D23) + trigger `space/,/.` (D11) + toast + kill-switch (D20).
- Đo lại **per-process RSS trong TIP** (F61) + đo `assist` **async/timeout 50 ms** (D36) khi có provider OS.
