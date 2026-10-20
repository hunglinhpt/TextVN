# P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)

> **Quy ước:** ID `AS-nnn` (Phần 4). Agent nhận việc theo ID, tick ở đây + bằng chứng.
> **DoD (7 mục, `docs/01-AGENT-HANDBOOK.md`):** test xanh · corpus pass · clippy+fmt sạch · `cargo deny` + REUSE · docs cập nhật · acceptance tick + **bằng chứng** · CI 3 OS xanh.
> **Cross-ref:** `P4-0-MASTER-PLAN.md` (plan D1–D29 · pha §8) · `P4-1-harper-evaluation.md` (đánh giá Harper) · `P4-2-CROSS-REVIEW-2026-10-20.md` (cross-review) · `docs/adr/README.md` (ADR-012, ADR-013)

## Trạng thái tổng quan

| Pha | Task | Nội dung | Trạng thái |
|---|---|---|---|
| P4-0 | AS-000 | Tài liệu: plan + đánh giá + cross-review + ADR-012/013 | ✅ xong |
| **P4-1** | AS-001…006 | Crate `assist/` (tokenize · mask · 6 module luật · fix) + L0 + `confidence` + corpus + test tích hợp + CLI `assist check` | ✅ **xong** (2026-10-20) |
| P4-1b | AS-007…010 | Spike **S1** TSF candidate list · **S2** caret 3 OS · **S3** từ điển VN | ✅ **xong** → `P4-3-SPIKES.md` |
| P4-1b+ | AS-011 | Spike **S4a** per-process RSS/binary/p95 (chặn P4-4) · **S4b** harper-core/EN (v2) | ✅ **xong** (S4a: RSS 8,86 MB · exe 993 KB · 0,52 ms/câu) |
| P4-1c | AS-012…014 | Patch docs cross-review: Fassist-32 (tiền đề) · 33/34 (wording) · 37 (verified-ops) | ✅ xong |
| **P4-2a** | **AS-020** | Refactor `core`: 13 call site `recent_replace` → 2 helper + `Engine.sentence` + `Engine::glyph_len_of` | ✅ **xong** (2026-10-21) |
| P4-2 | AS-021…025 | FFI `ime_assist_*` · trigger `space/,/.` · toast · kill-switch | 🔄 **đang làm** (Agent Core) |
| P4-3 | AS-030…036 | Candidate list Windows (`ITfCandidateListUI`) + A2 (sửa 1 từ) + precondition | ⬜ chờ nhận (Agent Windows/TSF) |
| P4-4 | AS-040…049 | Từ điển VN: **(a) glue provider OS** (Windows `ISpellCheckerFactory` · macOS `NSSpellChecker` · Linux `libspelling`) theo D32/D33 **trước** → **(b) FST fallback** từ `hunspell-vi` + attribution (D37) + gợi ý từ | ⬜ chờ nhận (Agent Dict/Data) |
| P4-5 | AS-050…056 | Mode 2 / A3 (tự sửa cả câu) + toast + undo | ⬜ chưa làm |
| P4-6 | AS-060…068 | Parity macOS/Linux + settings 3 OS + `ui-spec` | ⬜ chưa làm |

## Chi tiết P4-1 (đã xong)

| ID | Task | File | Acceptance | Bằng chứng |
|---|---|---|---|---|
| AS-001 | Tokenizer + mask | `assist/src/tokenize.rs`, `assist/src/mask.rs` | Offset **char Unicode**; mask URL/email | 34 unit test |
| AS-002 | 6 module luật objective (7 mã luật) | `assist/src/rules/*.rs` | L1/L2/L11 · L3 · L4 · L5 · L6 · L8 | unit test từng luật |
| AS-003 | L0 (bỏ qua câu không phải tiếng Việt) | `assist/src/rules/language.rs` | Tỷ lệ từ có dấu ≥ 50% | `cau_tieng_anh_khong_bao_loi` |
| AS-004 | `apply_auto_fixes` (char-space) + `confidence` | `assist/src/fix.rs`, `assist/src/lib.rs` | Không lệch offset khi sửa nhiều lỗi | `sua_lap_tu`, `sua_khoang_trang` |
| AS-005 | Corpus ≥ 60 câu + test tích hợp | `corpus/assist/vi_basic.ass`, `assist/tests/corpus.rs` | 69 câu · 6 loại lỗi đều có case | `corpus_vi_basic_passes` |
| AS-006 | CLI `textvn-cli assist check` | `cli/src/assist.rs` | `--json` · `--fix` · `--sentence` · `--file` (per-line) | smoke test tay (xem `docs/specs/verified-ops.md`) |

**Ghi chú ngôn ngữ:** thông điệp người dùng **có dấu tiếng Việt** (chuẩn UI của repo) — khác với comment code (ASCII) → finding Fassist-46.

## Task mở (chi tiết acceptance & phân công Agent)

| ID | Task | Acceptance | Phụ thuộc | Agent phân công | Trạng thái |
|---|---|---|---|---|---|
| AS-007 | **S1** — TSF: mở candidate list **sau khi commit** | Kết luận viết ra + bằng chứng (log/screenshot) | — | Agent Windows | ✅ xong (`P4-3-SPIKES.md`) |
| AS-008 | **S2** — đọc **vị trí caret** 3 OS (`ITfContext::GetSelection` / `AXRange` / AT-SPI `TextCaret`) | Kết luận + code mẫu | — | Agent Platform | ✅ xong (`P4-3-SPIKES.md`) |
| AS-009 | **S3** — khảo sát ≥ 3 nguồn từ điển VN (license + kích thước + chất lượng) | `docs/specs/assist-dictionary-survey.md` | — | Agent Dict | ✅ xong (`P4-3-SPIKES.md`) |
| AS-010 | **S4a** — đo per-process RSS/binary/p95 (Windows thật) | RSS ≤ +5 MB · binary ≤ +3 MB · p95 ≤ 2 ms | AS-002 | Agent Perf | ✅ xong (`P4-3-SPIKES.md`) |
| AS-011 | **S4b** — khảo sát `harper-core`/EN (v2, **không chặn v1**) | Kết luận + khuyến nghị | — | Agent Research | ✅ xong (`P4-3-SPIKES.md`) |
| AS-020 | Refactor `core` buffer | 13 call site → 2 helper; `Engine.sentence` (Unicode ≤256); `Engine::glyph_len_of`; **không** đổi hành vi gõ | ADR-013 | **Agent Core (ta)** | ✅ **xong** (2026-10-21: 8 test mới, 143/143 pass) |
| AS-021 | FFI `ime_assist_*` (C-ABI) + header + `textvn-cli verify` | Drift 0 (ADR-023); export struct/fn qua `ffi/` và `textvn_ffi.h` | AS-020 | **Agent Core (ta)** | ✅ **xong** (2026-10-21: 55 hằng, 6 struct, 16 hàm verify) |
| AS-022 | Trigger `space` / `,` / `.` (silent) + **idle 700 ms** / kết thúc câu (UI) | Điểm kiểm ≠ điểm hiển thị (D11); gọi `assist::check_sentence` qua sentence buffer | AS-021 | **Agent Core (ta)** | ✅ **xong** (2026-10-21: ime_assist_trigger_kind + check_instance) |
| AS-023 | Toast/tray notification + kill-switch 1 bước | Tắt được ngay, không cần mở app | AS-022 | **Agent Tray (ta)** | ✅ **xong** (2026-10-21: IPC toggle/notify + tray balloon + menu item) |
| AS-030 | `ITfCandidateListUI` (Windows) | Panel hiện cạnh con trỏ, không chiếm focus | AS-007 | Agent Windows/TSF | ⬜ chờ nhận |
| AS-031 | A2 — sửa 1 từ (Backspace+Type + precondition) | Sửa đúng vị trí ở Notepad/Word/Chrome/VSCode · `Ctrl+Z` hoàn tác | AS-030 + AS-008 | Agent Windows/TSF | ⬜ chờ nhận |
| AS-040 | Từ điển FST mmap + `OnceLock<Arc<…>>` | ≤ 3 MB · mmap < 10 ms · FPR ≤ 2% · license audit GPL hunspell-vi | AS-009 + AS-010 | Agent Dict/Data | ⬜ chờ nhận |
| AS-041 | Gợi ý từ (edit distance âm tiết) | Recall ≥ 80% trên corpus lỗi | AS-040 | Agent Dict/Data | ⬜ chờ nhận |
| AS-042 | Glue OS spellcheck provider (Windows `ISpellCheckerFactory` / macOS / Linux) | D32-D37 (OS-first) | AS-040 | Agent Platform/FFI | ⬜ chờ nhận |
| AS-050 | Mode 2 / A3 — tự sửa cả câu | Chỉ luật objective · precondition bắt buộc · undo | AS-031 + AS-040 | Agent Core/App | ⬜ chưa làm |
| AS-060 | macOS (IMK) + Linux (IBus/Fcitx5) parity | Cùng corpus pass 3 OS · `ui-spec` + `parity-checklist` cập nhật | AS-050 | Agent Mac/Linux | ⬜ chưa làm |

## Recipe P4-2a — refactor buffer trong `core` (đọc trước khi code)

`core/src/lib.rs` có **13 call site `recent_replace`** + 4 `recent.clear()` + 3 `recent.pop()`; `recent` là **glyph space** (`emit()` = `transform::charset::encode`), cap 64 (`RECENT_MAX`).

| Nhóm | Line (2026-10-20) | Lệnh hiện tại | Nguồn **Unicode** tương ứng (để nuôi `sentence`) |
|---|---|---|---|
| Reset | 202 `set_options` · 225 `reset` (focus) · 254 `DOWN` · 262 non-printable · 341 | `recent.clear()` | xoá `sentence` luôn |
| Phím in được | 257 · 460 · 570 `close_word` | `recent_replace(0, &[c])` | `c` (ký tự gõ) — đã Unicode |
| Commit từ | 450 | `let out = self.emit(&display); recent_replace(0, &out)` | **`display`** (trước khi encode) |
| Thay từ | 480 · 664 | `recent_replace(delete, &out)` | biến **trước** `emit` (cùng scope) |
| Macro/mở rộng | 523 | `let insert = self.emit(&expanded).take(MAX_TEXT); recent_replace(len, &insert)` | `expanded` — ⚠️ **cắt `MAX_TEXT` SAU encode** → phải giữ nguyên semantics |
| Restore | 550 | `insert = raw + c`; `recent_replace(delete, &insert)` | `raw` + `c` |
| Xoá từ | 618 | `recent_replace(delete, &insert)` | `insert` (Unicode) |
| Xoá từ (raw) | 683 | `recent_replace(delete, &raw)` | `raw` = `word.raw` (Unicode) |
| Xoá không chèn | 710 | `recent_replace(delete, &[])` | rỗng |
| Backspace | 248 · 725 · 765 | `recent.pop()` | ⚠️ pop **1 glyph**; `sentence` phải pop **1 char Unicode** → 2 buffer tách bạch (D24/D27) |

**Cách làm an toàn (1 commit, oracle = test `core`):**
1. Thêm `sentence: Vec<char>` (Unicode, cap 256 — D31) + `fn record(&mut self, delete: usize, insert_unicode: &[char])` **gộp cả 2 buffer** → không còn chỗ sửa `recent` trực tiếp.
2. `record()` phải **tái tạo đúng semantics của `recent`** (kể cả `MAX_TEXT` cắt sau encode ở site 523) — **không** đổi hành vi gõ.
3. Thay 13 call site `recent_replace` → `record` với **nguồn Unicode** ở bảng trên; 4 `clear()` → `clear()` cả hai; 3 `pop()` → pop cả hai (glyph 1 / Unicode 1).
4. `Engine::glyph_len_of(unicode_slice)` (D27) = `emit(unicode).len()` → adapter biết số ký tự cần xoá.
5. Gate: `cargo test -p textvn-core` **không đổi** (số test, kết quả) + `cargo test --workspace` xanh + clippy/fmt sạch.

> **F65 (major):** 2 call site có **hậu kỳ hậu-encode** (site 523 cắt `MAX_TEXT` sau encode) → đổi tham số sang Unicode **không được** thay đổi hành vi `recent` → phải giữ `MAX_TEXT` trong `record()`.
> **F66 (minor):** `recent.pop()` xoá 1 **glyph**; nếu glyph cuối là 1 chữ 2 byte (TCVN3/VNI-Windows) thì `sentence` phải xoá 1 **char Unicode** → xác nhận 2 buffer tách bạch (D24/D27), không dùng chung.



1. Đọc `P4-0` §3.1 (D1–D29) trước khi code — **không** quyết định lại D nào đã chốt.
2. Mỗi task: **1 commit** + list `git add` tường minh (G2) → push → CI xanh (G13) → `graphify update .` → commit `graphify-out/` → push (G5).
3. Bằng chứng bắt buộc trước khi tick: output CLI/screenshot + **log không chứa text người dùng** (S2).
4. Không đọc nội dung app khác; chỉ đọc **vị trí con trỏ** khi cần precondition (D8/D12).
5. Không thêm dependency mới ngoài allow-list `deny.toml`; `cargo deny check` phải xanh.
