# P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công

- **Ngày:** 2026-10-21 · **Trạng thái:** Handoff (chưa implement) · **Phần:** 4 — P4 · **Agent Core**
- **Vì sao là handoff:** `P4-0` §3.1 đã chốt hợp đồng (D22/D23/D24/D26/D27/D29/D31) nhưng **chưa có mã**. File này ghi lại **chính xác chỗ sẽ sửa + cách sửa + cách kiểm chứng** để không phải suy lại.
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §3.1 · `P4-5-P4-2A-REVIEW.md` (P4-2a xong) · `docs/10-shared/P0-2-engine-ffi-contract.md` (hợp đồng FFI) · `P4-TASKS.md` (AS-020…AS-022)

---

## 0. Chuỗi phụ thuộc (đã xong / sắp làm)

```
AS-020 P4-2a  ✅ xong  (Engine.sentence Unicode + glyph_len_of + record())
   └─> AS-021 FFI ime_assist_*   ← HÀNH ĐỘNG TIẾP THEO
         └─> AS-022 trigger space/,/. (silent) + idle 700ms / hết câu (UI)
               └─> AS-023 toast + kill-switch (Agent Tray) · AS-030/031 (Agent Windows/TSF)
```

## 1. AS-021 — FFI `ime_assist_*` (C-ABI)

### 1.1 API công khai (đề xuất, khớp D23/D27/D29)

```c
/* include/textvn_ffi.h — ABI mới, cần bump IME_ABI_VERSION */
typedef struct {
    uint32_t kind;          /* 0 = không lỗi; 1..N = assist_issue_kind_t */
    uint32_t start_char;    /* offset KY TỰ UNICODE trong câu (UTF-16?) */
    uint32_t length_char;   /* độ dài (ký tự Unicode) */
    uint32_t flags;         /* bit0: auto_fixable, bit1: has_suggestion */
    uint32_t suggestion_len;/* số ký tự UTF-16 của gợi ý đầu (0 = không có) */
} ime_assist_issue_v1;

/* Kiểm tra 1 câu. Trả về số issue ghi vào `out` (cap = out_cap).
 * text_utf16/len = câu ở dạng UTF-16 (chuẩn TSF/AX).
 * Hàm LUÔN trả 0 (fail-open) trừ khi truyền con trỏ sai.
 * Không bao giờ log nội dung câu (S2). */
int32_t ime_assist_check_v1(const uint16_t *text_utf16, int32_t len,
                            ime_assist_issue_v1 *out, int32_t out_cap);

/* Số ký tự GLYPH app phải xoá để áp gợi ý Unicode (D27). */
int32_t ime_assist_glyph_len_v1(const uint16_t *text_utf16, int32_t len);
```

### 1.2 Quy tắc bắt buộc

| # | Quy tắc | Nguồn |
|---|---|---|
| 1 | **UTF-16 ở biên**: nhận `uint16_t*`, quy đổi sang `char` (Rust) trước khi gọi `assist` (crate dùng **char offset**) | D29 |
| 2 | **Offset trả về = char Unicode** (không phải byte, không phải UTF-16) → adapter tự quy đổi khi highlight | D27/D29 |
| 3 | **Không `unsafe` vượt biên**: đọc `len` phần tử, kiểm tra null, `catch_unwind` quanh lời gọi `assist` | Handbook S4 |
| 4 | **Fail-open**: lỗi bất kỳ → trả 0 issue, **không** panic, **không** log câu | S2/P0-2 |
| 5 | **Bump `IME_ABI_VERSION`** + cập nhật `textvn_ffi.h` + `ffi/tests/abi_invariants.rs` + `textvn-cli verify` (drift 0) | DoD #3/#4, ADR-023 |
| 6 | **Không thêm artifact**: `textvn_ffi` đã có sẵn (D14) | D14 |
| 7 | Không đụng `assist` crate (0-dep, portable) — chỉ `ffi` gọi vào | D32 |

### 1.3 Chỗ sẽ sửa (đã đọc trong repo)

| File | Việc |
|---|---|
| `ffi/src/lib.rs` | thêm `ime_assist_check_v1` + `ime_assist_glyph_len_v1` (đặt cạnh các `ime_*` hiện có) |
| `include/textvn_ffi.h` | khai báo struct + 2 prototype; **thứ tự** phải khớp header Rust (kiểm tra bằng `textvn-cli verify`) |
| `ffi/tests/abi_invariants.rs` | thêm case: struct size/offset đúng, export count +1, version bump |
| `ffi/` bump version | `IME_ABI_VERSION` (đọc hằng số hiện tại trong `ffi/src/lib.rs`) |

### 1.4 Kiểm chứng

```
cargo test -p textvn-ffi -j 2        # ABI invariants
cargo run -p textvn-cli -- verify    # header C khớp code Rust (drift 0)
cargo clippy --workspace --all-targets -j 2 -- -D warnings
cargo fmt --all --check
```

## 2. AS-022 — Trigger (điểm kiểm tra ≠ điểm hiển thị)

### 2.1 Luồng

```
người dùng gõ space/,/.        (adapter TSF/IMK/IBus)
   └─> adapter bắt phím (compose.rs: KeyKind) — SAU khi phím đã xử lý xong
        └─> lấy câu: Engine::sentence_tail(max=500)   ← không đọc app (D8)
             └─> assist::check_sentence(câu)  (bỏ qua nếu L0 = không phải tiếng Việt)
                  └─> nếu có lỗi:
                       • Mode 1 (highlight): gửi lên tray/overlay (UI ở P4-3/AS-023)
                       • Mode 2 (auto-correct): áp lỗi objective **trong lúc commit** (P4-5)
```

### 2.2 Quy tắc hành vi (D11 + D12)

| # | Quy tắc |
|---|---|
| 1 | **Điểm kiểm tra** = mọi phím `space` `,` `.` `!` `?` `;` `:` → check **im lặng** (không hiện UI) |
| 2 | **Điểm hiển thị** = khi câu **kết thúc** bằng `.` `!` `?` **hoặc** user **ngừng gõ ≥ 700 ms** **hoặc** `Ctrl+Shift+G` → mới hiện candidate/overlay |
| 3 | **Không** check khi: đang compose · app không phải ô nhập · `assist.mode=off` · câu < 3 từ · câu bị mask |
| 4 | **Không chặn input path**: check chạy **ngoài** đường gõ (worker thread của adapter) hoặc sau khi phím đã xử lý; vượt budget → **bỏ qua im lặng** (fail-open) |
| 5 | **Không đọc nội dung app** — chỉ dùng `Engine::sentence_tail()` (D8) |
| 6 | Áp sửa (Mode 2) cần **precondition**: caret ở cuối câu (đọc **vị trí**, S2 — adapter đã có `selection_range()` + `GetText` ở `edit_session.rs`) → không xác minh được thì **không sửa** (D12) |
| 7 | Số ký tự cần xoá = `Engine::glyph_len_of(unicode_slice)` (D27) — **không** dùng `len()` của char |

### 2.3 Chỗ sẽ sửa

| File | Việc |
|---|---|
| `adapters/windows-tsf/src/compose.rs` | nhận diện phím trigger (`KeyKind`), sau khi commit xong → gọi callback sang tầng trên |
| `adapters/windows-tsf/src/lib.rs` (hoặc `tip.rs`) | nối callback → `ime_assist_check_v1` + `Engine::sentence_tail()`; timer idle 700 ms |
| `adapters/macos-imk/*.swift` · `adapters/linux-*` | cùng logic ở OS kia (P4-6) — dùng chung hợp đồng FFI (D23) |
| `ipc/src/lib.rs` | (nếu cần) message mới `AssistIssues { … }` để tray hiển thị — theo `schemas/ipc.v1.md` |

### 2.4 Kiểm chứng

- Unit: trigger chỉ bắn khi đúng phím; câu sạch → **không** gọi UI; câu lỗi → đúng 1 lần.
- Perf: không tăng p99 của `ime_key` (PLAN §5.5: `ime_key` < 0.5 ms, adapter+strategy < 3 ms).
- Manual: Notepad/Word/Chrome gõ "tôi tôi đi học." → overlay hiện **đúng 1 lần** (không spam mỗi space).
- Không log câu (S2): `doctor`/log không chứa nội dung.

## 3. Rủi ro cần canh

| # | Rủi ro | Chặn bằng |
|---|---|---|
| R18 | Header C lệch Rust → ABI sai giữa app và DLL | `textvn-cli verify` (drift 0) + `abi_invariants.rs` |
| R19 | Sửa text đúng vị trí (đã commit) → sai | **Chỉ sửa trước khi commit** hoặc **precondition caret** (D12); không sửa hậu-commit bằng ITfRange |
| R20 | Overlay spam khi gõ thường | D11 (điểm kiểm ≠ điểm hiển thị) |
| R21 | Rò rỉ text qua log/crash | Không log; không `String` tồn tại lâu; không đưa vào `crash` handler |

## 4. Definition of Done (cho AS-021/AS-022)

- [ ] `cargo test --workspace -j 2` xanh · clippy 0 warning · fmt sạch
- [ ] `textvn-cli verify` drift 0 (header ↔ Rust) · `cargo deny check` OK · REUSE OK
- [ ] Corpus `corpus/assist/` pass (không đổi hành vi luật)
- [ ] Manual: 1 câu lỗi → 1 lần hiện overlay; câu sạch → im lặng; log **không** có text
- [ ] `P4-TASKS.md` tick AS-021/AS-022 + bằng chứng (screenshot/CLI output)
- [ ] Commit list tường minh → push → CI 3 OS xanh → `graphify update .` → commit `graphify-out/` → push
