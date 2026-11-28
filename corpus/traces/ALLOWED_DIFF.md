<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ALLOWED_DIFF — Khác biệt hợp lệ giữa 3 adapter (`SF-1f`, P0-6 §6.2)

> Mọi khác biệt theo cặp `(case, step)` giữa `corpus/traces/tsf/shared/`,
> `corpus/traces/linux/shared/`, và `corpus/traces/mac/shared/` **PHẢI** khớp một
> luật hẹp bên dưới (được `cargo xtask trace-diff` kiểm cấu trúc + nội dung chữ).
> Bất kỳ khác biệt nào ngoài bảng này — hoặc quy tắc không còn khớp bước nào — đều
> làm `trace-diff` / `preflight` / `ci-shared` thoát mã `1`.

## Bảng quy tắc được duyệt / Approved Diff Rules

| ID | Pair | Lý do kỹ thuật & Phạm vi hẹp / Technical Rationale & Narrow Scope | Approved by |
|---|---|---|---|
| `CUAS_PRINTABLE_BOUNDARY` | `tsf-linux` | Windows TSF (CUAS) nuốt phím ranh giới từ in được (`eat_key`) và commit `word + boundary`; Linux IBus/Fcitx5 commit `word` giữ nguyên chữ và `forward_key` phím ranh giới. Chỉ khớp khi phần `word` trước ký tự ranh giới giống hệt nhau. / TSF CUAS consumes printable word boundary (`eat_key`) committing `word + boundary`; Linux commits identical `word` and forwards boundary key (`forward_key`). | `antigravity-gemini-37c7c6bc` |
| `MAC_IMK_PREEDIT_LIFECYCLE` | `linux-mac` | Trên macOS IMK, phím chữ chưa biến đổi đi thẳng (`forward_key`); khi bắt đầu biến đổi phải `delete_before(d)` phần đã pass rồi `set_preedit(P)`; khi cập nhật marked phải thu hồi `set_preedit("")` trước `set_preedit(P)`; và ở ranh giới `:reset` hai bên commit giống nhau (`lc == mc`) trước khi phím chữ mới đi `forward_key` trên macOS — mọi chuỗi `P` và `commit` bắt buộc khớp Linux từng ký tự. / In macOS IMK, unmodified letters forward until the first edit deletes passed chars and activates preedit, preedit updates retract previous marked text first, and at `:reset` boundaries both flush identical text before the next raw char forwards on macOS; preedit/commit text MUST match Linux identically. | `antigravity-gemini-37c7c6bc` |
| `MAC_PRINTABLE_BOUNDARY_COMMIT` | `linux-mac` | Tại ranh giới từ in được, macOS (như TSF) commit `word + boundary` + `eat_key` khi vừa biến đổi (`word` khớp Linux từng byte), hoặc `forward_key` nếu cả từ đi pass-through chưa mở preedit. / At printable word boundaries, macOS commits identical `word + boundary` + `eat_key` like TSF, or `forward_key` for pass-through words never opened in preedit. | `antigravity-gemini-37c7c6bc` |
| `MAC_BACKSPACE_TYPE_CLEAR_ORDER` | `linux-mac` | Với `.restore`, gõ tắt/emoji, tự viết hoa, hoặc `--`, `ApplyReplace.backspaceType` thu hồi `set_preedit("")` trước khi xoá phần đã pass + `commit(T)`; chuỗi `T` bắt buộc khớp Linux byte-for-byte. / For restore, macro/emoji expansion, auto-cap, or em-dash, `ApplyReplace.backspaceType` retracts preedit before deleting passed chars and committing `T`, which MUST match Linux byte-for-byte. | `antigravity-gemini-37c7c6bc` |

## Ghi chú / Notes
- `corpus/traces/tsf/win/*.jsonl` phủ các ca đặc thù Windows (TSF/hook/sandbox), nên
  đối chiếu 3 nền tảng chạy trên giao điểm chuẩn **58 ca** `corpus/shared/`.
- `cargo xtask trace-diff` cũng sinh lại vết TSF vào thư mục tạm và đối chiếu
  byte-to-byte toàn bộ **136** tệp trong `corpus/traces/tsf/` (gate chống trôi vết TSF).

## SF-2 · corpus phím chuyển `corpus/toggle/toggle.cases` / SF-2 · toggle corpus

Khác biệt ĐÃ BIẾT giữa mã cũ từng OS và `textvn-session` (cột `<os>=` trong tệp corpus). Harness 3
OS khẳng định đúng các số này; shadow `TEXTVN_SESSION_SHADOW=1` đếm chúng là lệch. Quyết định giữ
hay đổi hành vi được đưa ra khi session thành mặc định (SF-2, bước "bật mặc định").
/ KNOWN differences between each OS's old code and `textvn-session` (the `<os>=` column of the corpus
file). The 3 OS harnesses assert exactly these numbers; the `TEXTVN_SESSION_SHADOW=1` shadow counts
them as mismatches. Whether to keep or change the behaviour is decided when the session becomes the
default (SF-2, "enable by default" step).

- `alt_tap_between` · `mac=0`: IMK giữ cờ `otherKeyPressed` tới khi nhả hết Ctrl/Shift, nên Option/Cmd
  nhấn-thả giữa chừng huỷ cả lần bấm; session (như TSF/Linux) chỉ huỷ phần đang chờ, Shift nhấn sau
  vẫn bật/tắt. / IMK keeps `otherKeyPressed` until Ctrl/Shift are all released, so an Option/Cmd tap
  in between cancels the whole press; the session (like TSF/Linux) only cancels what is pending, and a
  later Shift press still toggles.
- `chord_with_alt` · `linux=1`: IBus/Fcitx5 cũ nhận `Ctrl+Shift+Alt+Space` là chord ADR-011 (không xét
  Alt); session, TSF (preserved key `TF_MOD_CONTROL|TF_MOD_SHIFT`) và IMK (`isToggleChord` so đúng
  mask) thì không. / Old IBus/Fcitx5 treat `Ctrl+Shift+Alt+Space` as the ADR-011 chord (Alt
  ignored); the session, TSF (preserved key `TF_MOD_CONTROL|TF_MOD_SHIFT`) and IMK (`isToggleChord`
  exact mask) do not.
- `double_toggle_fast` · `linux=2`: Linux cũ không có debounce 250 ms; session, TSF
  (`claim_hotkey_toggle_at`) và IMK (`MacModifierTap.debounceSeconds`) gộp 2 lần bật/tắt < 250 ms
  thành 1. / Old Linux has no 250 ms debounce; the session, TSF (`claim_hotkey_toggle_at`) and IMK
  (`MacModifierTap.debounceSeconds`) merge two toggles < 250 ms apart into one.
