# P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)

> **Quy ước:** ID `AS-nnn` (Phần 4). Agent nhận việc theo ID, tick ở đây + bằng chứng.
> **DoD (7 mục, `docs/01-AGENT-HANDBOOK.md`):** test xanh · corpus pass · clippy+fmt sạch · `cargo deny` + REUSE · docs cập nhật · acceptance tick + **bằng chứng** · CI 3 OS xanh.
> **Cross-ref:** `P4-0-MASTER-PLAN.md` (plan D1–D29 · pha §8) · `P4-1-harper-evaluation.md` (đánh giá Harper) · `P4-2-CROSS-REVIEW-2026-10-20.md` (cross-review) · `docs/adr/README.md` (ADR-012, ADR-013)

## Trạng thái tổng quan

| Pha | Task | Nội dung | Trạng thái |
|---|---|---|---|
| P4-0 | AS-000 | Tài liệu: plan + đánh giá + cross-review + ADR-012/013 | ✅ xong |
| **P4-1** | AS-001…006 | Crate `assist/` (tokenize · mask · 6 module luật · fix) + L0 + `confidence` + corpus + test tích hợp + CLI `assist check` | ✅ **xong** (2026-10-20) |
| P4-1b | AS-007…010 | Spike **S1** TSF candidate list · **S2** caret 3 OS · **S3** từ điển VN | ⬜ chưa làm |
| P4-1b+ | AS-011 | Spike **S4a** per-process RSS/binary/p95 (chặn P4-4) · **S4b** harper-core/EN (v2) | ⬜ chưa làm |
| P4-1c | AS-012…014 | Patch docs cross-review: Fassist-32 (tiền đề) · 33/34 (wording) · 37 (verified-ops) | ✅ xong |
| P4-2a | AS-020 | Refactor `core`: 13 call site `recent_replace` → 2 helper + `Engine.sentence` + `Engine::glyph_len_of` | ⬜ chưa làm |
| P4-2 | AS-021…025 | FFI `ime_assist_*` · trigger `space/,/.` · toast · kill-switch | ⬜ chưa làm |
| P4-3 | AS-030…036 | Candidate list Windows (`ITfCandidateListUI`) + A2 (sửa 1 từ) + precondition | ⬜ chưa làm |
| P4-4 | AS-040…046 | Từ điển VN (FST mmap + `OnceLock`) + gợi ý từ (chỉ sau S4a) | ⬜ chưa làm |
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

## Task mở (chi tiết acceptance)

| ID | Task | Acceptance | Phụ thuộc |
|---|---|---|---|
| AS-007 | **S1** — TSF: mở candidate list **sau khi commit** | Kết luận viết ra + bằng chứng (log/screenshot) | — |
| AS-008 | **S2** — đọc **vị trí caret** 3 OS (`ITfContext::GetSelection` / `AXRange` / AT-SPI `TextCaret`) | Kết luận + code mẫu | — |
| AS-009 | **S3** — khảo sát ≥ 3 nguồn từ điển VN (license + kích thước + chất lượng) | `docs/specs/assist-dictionary-survey.md` | — |
| AS-010 | **S4a** — đo per-process RSS/binary/p95 (Windows thật) | RSS ≤ +5 MB · binary ≤ +3 MB · p95 ≤ 2 ms | AS-002 |
| AS-011 | **S4b** — khảo sát `harper-core`/EN (v2, **không chặn v1**) | Kết luận + khuyến nghị | — |
| AS-020 | Refactor `core` buffer | 13 call site → 2 helper; `Engine.sentence` (Unicode ≤256, chỉ bật khi `assist.mode != off`); `Engine::glyph_len_of`; **không** đổi hành vi gõ | ADR-013 |
| AS-021 | FFI `ime_assist_*` (C-ABI) + header + `textvn-cli verify` | Drift 0 (ADR-023) | AS-020 |
| AS-022 | Trigger `space` / `,` / `.` (silent) + **idle 700 ms** / kết thúc câu (UI) | Điểm kiểm ≠ điểm hiển thị (D11) | AS-021 |
| AS-023 | Toast/tray notification + kill-switch 1 bước | Tắt được ngay, không cần mở app | AS-022 |
| AS-030 | `ITfCandidateListUI` (Windows) | Panel hiện cạnh con trỏ, không chiếm focus | AS-007 |
| AS-031 | A2 — sửa 1 từ (Backspace+Type + precondition) | Sửa đúng vị trí ở Notepad/Word/Chrome/VSCode · `Ctrl+Z` hoàn tác | AS-030 + AS-008 |
| AS-040 | Từ điển FST mmap + `OnceLock<Arc<…>>` | ≤ 3 MB · mmap < 10 ms · FPR ≤ 2% · license audit | AS-009 + AS-010 |
| AS-041 | Gợi ý từ (edit distance âm tiết) | Recall ≥ 80% trên corpus lỗi | AS-040 |
| AS-050 | Mode 2 / A3 — tự sửa cả câu | Chỉ luật objective · precondition bắt buộc · undo | AS-031 + AS-040 |
| AS-060 | macOS (IMK) + Linux (IBus/Fcitx5) parity | Cùng corpus pass 3 OS · `ui-spec` + `parity-checklist` cập nhật | AS-050 |

## Quy tắc nhận việc

1. Đọc `P4-0` §3.1 (D1–D29) trước khi code — **không** quyết định lại D nào đã chốt.
2. Mỗi task: **1 commit** + list `git add` tường minh (G2) → push → CI xanh (G13) → `graphify update .` → commit `graphify-out/` → push (G5).
3. Bằng chứng bắt buộc trước khi tick: output CLI/screenshot + **log không chứa text người dùng** (S2).
4. Không đọc nội dung app khác; chỉ đọc **vị trí con trỏ** khi cần precondition (D8/D12).
5. Không thêm dependency mới ngoài allow-list `deny.toml`; `cargo deny check` phải xanh.
