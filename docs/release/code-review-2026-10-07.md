# Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)

> Rà soát **từng dòng** dưới góc nhìn chuyên gia (correctness · an toàn bộ nhớ/FFI ·
> concurrency · bảo mật · fail-open S4 · riêng tư S2/S3), làm **cuốn chiếu**: rà xong
> module nào → sửa + test + commit module đó → sang module kế. Mọi finding giữ ID,
> **không xoá** — chỉ đổi trạng thái (quy ước `00-INDEX.md` §3).

## 0. Baseline (trước khi sửa)

| Gate | Lệnh | Kết quả |
|---|---|---|
| fmt | `cargo fmt --all --check` | ✅ |
| clippy Linux | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| clippy Windows (cross-check) | `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings` | ✅ |
| test | `cargo test --workspace` | ✅ (≈400 test) |
| replay | headless 39 · tsf 117 · mac 153 · linux 39 | ✅ |
| ABI | `textvn-cli verify` · `textvn-cli sizes` (532 B) | ✅ |
| xtask | `check-tables` · `check-version-sync` (14 site) · `check-mac-corpus` (114) · `check-mac-targets` | ✅ |

Giới hạn môi trường rà soát: container Linux — Rust (Linux + cross-check Windows) và C
(linux-common) biên dịch/chạy được; **Swift (macOS) không có toolchain** → chỉ rà tĩnh;
PowerShell/Inno chỉ rà tĩnh.

## 1. Kế hoạch (thứ tự theo rủi ro: engine chạy trong process của mọi app → FFI → adapter)

| Pha | Phạm vi | Dòng ~ | Trọng tâm | Trạng thái |
|---|---|---:|---|---|
| P1 | `core/` (engine: buffer, method, transform, post, validate, lib) | 4.6k | Đúng chính tả, panic/overflow (S4), invariant buffer, undo/restore | ✅ |
| P2 | `ffi/` (lib, settings, header C) | 1.4k | `unsafe`, null/len, `catch_unwind`, UTF-8 cắt giữa ký tự, ABI | ✅ |
| P3 | `config/` `appdb/` `strategy/` `field-detect/` `ipc/` | 2.9k | Parse input không tin cậy, giới hạn kích thước, ghi file nguyên tử | ✅ |
| P4 | `adapters/windows-tsf/` | 3.9k | COM refcount, edit session, re-entrancy, fail-open | ⏳ |
| P5 | `adapters/windows-hook/` + `tray/` | 7.1k | Hook chỉ quan sát, pipe server (DACL, giới hạn frame), race | ⏳ |
| P6 | `cli/` (register, doctor, replay, verify) | 5.1k | Registry/đường dẫn, quyền, xử lý lỗi | ⏳ |
| P7 | `adapters/linux-*` (C/C++) | 4.5k | Buffer overflow, socket, GLib/fcitx lifetime | ⏳ |
| P8 | `adapters/macos-*` (Swift) | 5.6k | Socket, IMK client lifetime, event tap | ⏳ |
| P9 | `scripts/` `packaging/` `installer/` `tools/` `.github/workflows/` | 6k | Shell quoting, injection trong workflow, quyền token | ⏳ |
| P10 | `xtask/` `tools/bench` `fuzz/` | 3k | Tính đúng của gate | ⏳ |
| P11 | Chạy lại toàn bộ gate + cập nhật báo cáo + push | — | — | ⏳ |

Mức độ: **P0** (mất chữ/crash/lỗ hổng) · **P1** (sai hành vi người dùng thấy) ·
**P2** (rủi ro tiềm ẩn/edge case) · **P3** (chất lượng code/nit).

## 2. Nhật ký finding

| ID | Mức | File:dòng | Mô tả | Xử lý | Trạng thái |
|---|---|---|---|---|---|
| CR-01 | **P0** | `core/src/lib.rs` `Engine::key` | Từ không giới hạn độ dài: gõ `dej` rồi giữ `p` (~65 lần, thói quen chat `đẹppppp…`) → `insert` > 64, FFI cắt bớt, `owned` lệch document → phím kế tiếp **xoá lẹm chữ phía trước** (tái hiện: đòi xoá 68 khi từ chỉ có 64). Restore/Esc với `raw` dài cũng bị cắt mất ký tự. | `MAX_WORD_KEYS = MAX_TEXT − 1`; vượt hoặc `emit()` > 64 (bảng mã 2 ký tự/chữ) → `close_word` (giữ nguyên từ trong document, phím đi như ký tự thường — cùng ngữ nghĩa ranh giới nên mọi adapter đã xử lý). Guard thêm Backspace (`asz`→`á` dài thêm ở bảng mã tổ hợp) và Tab gợi ý. Test: `core/tests/invariants.rs` (property test LCG 4 kiểu gõ × 4 bảng mã × preedit/không, có "giữ phím lặp") + `held_key_after_toned_word_keeps_previous_text` — đỏ trên code cũ, xanh sau sửa | ✅ Fixed |
| CR-02 | P1 | `ffi/src/lib.rs` `fill_result` | Lưới an toàn cuối **cắt** `insert`/`preedit` về 64 một cách im lặng — chính cơ chế biến CR-01 thành mất chữ. | Vượt `IME_MAX_TEXT` → từ chối: reset engine, PASS + `IME_FLAG_ERROR`, `last_error = "internal: result overflow"` (fail-open S4). Test `oversized_outcome_is_rejected_not_truncated`, `held_key_long_word_never_deletes_past_typed_text` | ✅ Fixed |
| CR-03 | P2 | `ffi/src/lib.rs` `ime_instance_new` / `ime_reload_config` / `ime_strategy_resolve` / `ime_appdb_verify` | Parse config/appdb (dữ liệu ngoài) nằm **ngoài** `catch_unwind` — trái P0-2 §5; từ Rust 1.81 panic vượt `extern "C"` là abort cả process chủ (TIP in-process = app người dùng). | Helper `guarded()` bọc mọi đường parse; panic → `IME_ERR_INTERNAL`, out vẫn fail-open | ✅ Fixed |
| CR-04 | P1 | `core/src/lib.rs` nhánh `is_chord` | Chord (Ctrl+←, Ctrl+Backspace, Ctrl+Z, Ctrl+V…) chỉ xoá `recent`, **giữ** `word`: adapter không tự reset (hook/Linux/mac BackspaceType) → phím kế tiếp REPLACE `owned` ký tự ở vị trí con trỏ mới (cùng lớp lỗi B7 đã sửa cho phím điều hướng). TSF tự reset nên không lộ trên Windows TSF. | Chord → `word.clear()` như phím điều hướng. Test `chord_drops_word_ownership`; corpus 4 adapter vẫn xanh | ✅ Fixed |
| CR-05 | P3 | `core/src/post/restore_en.rs` `complete_word` | Từ điển EN cá nhân so tiền tố **phân biệt hoa thường** (khác `listed()`): thêm `VnExpress` thì `vn`+Tab không gợi ý. | Lowercase ứng viên người dùng; thêm assert | ✅ Fixed |
| CR-06 | P1 (S3) | `strategy/src/resolve.rs` `resolve`; `core/src/lib.rs` | Gate bước 1 chỉ xét cờ `secure`: ô có `field_role = SECURE` mà adapter quên cờ thì hint/preset **thắng** (preset app không có `when` biến ô mật khẩu thành Preedit), và engine vẫn chạy macro (`allow_macro_when_vi_off`) + ghi `recent` trong ô mật khẩu. | `secure \|\| field_role == SECURE` ở resolver; engine dùng `Engine::secure()` cho mọi kiểm tra S3. Test `secure_role_beats_hint_and_presets_without_secure_flag`, `secure_role_without_flag_is_still_secure` | ✅ Fixed |
| CR-07 | P3 (perf) | `appdb/src/lib.rs` `Matcher::matches` | Mỗi lần so khớp entry dựng `MatchClause` tạm = clone 3 `String` — chạy trên đường phím (mac/Linux gọi `ime_strategy_resolve` mỗi key). | So khớp trên `&str` (`id_matches`), không alloc | ✅ Fixed |
| CR-08 | P3 | `config/src/macro_text.rs` `parse` | Nội dung gõ tắt bị `trim()` → khoảng trắng đầu/cuối có chủ ý (`"Trân trọng, "`) mất sau khi lưu; trigger chứa `=` (sửa tay trong JSON) đổi nghĩa khi round-trip qua ô soạn thảo. | Ghi nhận — đổi định dạng text ảnh hưởng 3 UI; để ngỏ, không sửa trong đợt này | 📝 Open |

## 3. Nhật ký tiến độ

- 2026-10-07 — P0 baseline xanh; lập plan.
- 2026-10-07 — P1 `core/` + P2 `ffi/` rà xong: CR-01…CR-05 sửa + test; gate fmt/clippy (Linux+Windows)/test/replay 4 adapter/verify xanh.
  Đã rà và **không** thấy lỗi: `method/{telex,vni,viqr,simple_telex}`, `transform/{tone,undo,stroke,vowel_table,diacritic_style,charset}`,
  `validate`, `post/{macro,caps,emoji,quick_telex}`, `ffi/src/settings.rs`, header C (bản mac giống hệt bản gốc).
- 2026-10-07 — P3 rà xong: CR-06 (S3), CR-07 sửa; CR-08 ghi nhận. `ipc` codec (giới hạn 64 KiB, từ chối trailing/UTF-8 sai),
  `config/doc.rs` (ghi nguyên tử 0600, sao lưu `.bak`), `field-detect` (generation gate, Unknown ⇒ Passthrough) không có lỗi.
