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
| P4 | `adapters/windows-tsf/` | 3.9k | COM refcount, edit session, re-entrancy, fail-open | ✅ |
| P5 | `adapters/windows-hook/` + `tray/` | 7.1k | Hook chỉ quan sát, pipe server (DACL, giới hạn frame), race | ✅ |
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
| CR-09 | P1 (bảo mật) | `adapters/windows-tsf/src/ipc_client.rs`, `adapters/windows-hook/src/main.rs`, `tray/src/{main,ipc_server}.rs`, `cli/src/doctor.rs` | 9 chỗ mở **client** named pipe bằng `OpenOptions` mặc định (không `SECURITY_SQOS_PRESENT`) → server cấp quyền **impersonate** client. TIP nằm trong mọi process (kể cả app quyền admin) và thử nối lại mỗi 2–10 s khi tray chưa chạy: process nào chiếm tên `\\.\pipe\textvn-ipc-v1` trước sẽ nhận kết nối và gọi được `ImpersonateNamedPipeClient` (tài liệu std Rust cảnh báo đúng ca này). | Helper `textvn_ipc::pipe_client_options()` (read+write + `SECURITY_IDENTIFICATION`) dùng ở mọi client; server chỉ cần `GetNamedPipeClientProcessId` nên không mất chức năng. Cross-check `--target x86_64-pc-windows-msvc` xanh | ✅ Fixed |
| CR-10 | P2 | tray/TIP/hook — tên pipe | `\\.\pipe\` là namespace **toàn máy**: Fast User Switching/RDS → user thứ hai không tạo được instance (DACL user đầu), TIP của user thứ hai rơi về chế độ offline, Ctrl+Shift/tray không tới được TIP. Server cũng không dùng `FILE_FLAG_FIRST_PIPE_INSTANCE` để phát hiện chiếm tên. | Đề xuất: pipe theo phiên `textvn-ipc-v1-<SessionId>` (đổi hợp đồng đặt tên 00-INDEX §4 + 4 thành phần) + `FIRST_PIPE_INSTANCE` ở instance đầu. Cần máy Windows đa phiên để kiểm — để ngỏ | 📝 Open |
| CR-11 | P2 | `{ffi,cli,tray}/build.rs`, `adapters/windows-{tsf,hook}/build.rs` | Sinh `FILEVERSION 0.2.27,0` / `PRODUCTVERSION 0.2.27,0` — RC cần 4 số phân cách **dấu phẩy**; khối VS_FIXEDFILEINFO (số Explorer, WACK/Store, so sánh phiên bản khi thay file) bị đọc sai. Chuỗi `FileVersion` vẫn đúng nên `build-release.ps1` không bắt được. | `version.replace('.', ",")` → `0,2,27,0` ở cả 5 build script | ✅ Fixed |
| CR-12 | P3 (perf) | `adapters/windows-tsf/src/edit_session.rs` `apply_display_attribute` | `CoCreateInstance(CLSID_TF_CategoryMgr)` + `RegisterGUID` ở **mỗi phím** đang soạn. | Cache atom theo thread (`thread_local!`, như SampleIME) | ✅ Fixed |
| CR-13 | P1 | `adapters/windows-hook/src/{lib,main}.rs` | Hook (BackspaceType) không reset engine khi **đổi cửa sổ foreground**, khi gặp **chord** (callback trả sớm, engine không thấy Ctrl+←/Ctrl+Z…) và khi `SendInput` **thất bại** (engine đã tính như thể REPLACE tới app) → phím dấu kế tiếp gửi Backspace xoá chữ ở cửa sổ/vị trí khác. | `HookEngine::reset()`; gọi khi foreground đổi, chord với phím không phải modifier, inject lỗi. Test `reset_drops_word_so_next_key_cannot_replace` | ✅ Fixed |
| CR-14 | P2 | hook ↔ TSF | `textvn-hook.exe` chạy `HookMode::Always`, còn `ThreadState::owns_engine` của TSF không được gọi ở đâu ngoài test: bật "chế độ tương thích" trong khi TIP TextVN đang là bộ gõ active ở app thì **hai engine cùng xử lý** (hook PASS chữ cho TSF soạn, rồi gửi Backspace vào composition). Click chuột đổi vị trí con trỏ cũng không được hook nhìn thấy. | Đề xuất: TIP báo `TipActive{pid}` qua IPC, hook bỏ qua pid đó (hoặc TSF tôn trọng `engine_owner`), + `WH_MOUSE_LL` chỉ quan sát để reset từ khi click. Cần Windows thật để kiểm — để ngỏ | 📝 Open |
| CR-15 | P2 (pin) | `tray/src/ipc_server.rs` `handle_client_connection` | Mỗi kết nối (mỗi process có TIP + hook) poll pipe **cố định 10 ms** → vài chục app = hàng nghìn lần đánh thức CPU/giây khi không gõ. | Backoff 10 → 100 ms khi rảnh, về 10 ms ngay khi có frame (client chỉ gửi lúc kết nối/toggle) | ✅ Fixed |
| CR-16 | P2 (bảo mật) | `tray/src/main.rs` `offer_machine_registration_if_needed` (SEC-02) | Gate "chỉ xin UAC khi chạy từ thư mục cài đặt chuẩn" dùng `contains("\\program files\\")` → `D:\x\Program Files\y\TextVN.exe` (thư mục người dùng ghi được) lọt qua, CLI cạnh nó được chạy elevated. | `path_is_under_any` so **tiền tố theo ranh giới thư mục** với `%ProgramFiles%`, `%ProgramFiles(x86)%`, `%ProgramW6432%`, `%LOCALAPPDATA%\Programs`. Test `trusted_install_gate_is_prefix_not_substring` | ✅ Fixed |
| CR-17 | P1 | `tray/src/main.rs` khởi động | `hotkey::free_ctrl_shift()` chạy **vô điều kiện mỗi lần tray khởi động** → người dùng bỏ chọn "Dành Ctrl + Shift cho TextVN" (Bảng điều khiển) hoặc bỏ task `freectrlshift` của bộ cài thì lần đăng nhập sau phím tắt Windows của họ lại bị gỡ (trái user-guide "bỏ chọn thì trả lại cho Windows"). | Bản cài Inno: để bộ cài quyết định; portable/Store: áp mặc định **một lần** (marker `%APPDATA%\TextVN\ctrl_shift_default_applied`). Gỡ bản portable từ menu trả lại phím tắt cho Windows (như BUG-07 của bộ cài) | ✅ Fixed |
| CR-18 | P1 | `tray/src/package_bootstrap.rs` `stage_package_payload` | `build-msix.ps1` đóng gói `textvn-tsf-x86.dll` (Zalo/Office 32-bit, vòng 14) nhưng bootstrap **không copy** nó ra thư mục staged → bản Microsoft Store: `register` bỏ mirror WOW64, app x86 không có tiếng Việt. Bản staged cũng thiếu `LICENSE` (GPL). | Stage thêm `textvn-tsf-x86.dll`, `LICENSE`, `CHANGELOG.md`; test Windows mở rộng | ✅ Fixed |
| CR-19 | P3 | `tray/src/settings_dialog.rs` ↔ `config/src/doc.rs` | Logic chuẩn hoá từ điển EN nhân bản ở 2 nơi kèm chú thích "phải giống hệt". | Một nguồn: `textvn_config::doc::normalize_english_words` | ✅ Fixed |
| CR-20 | P3 | `tray/src/svc.rs` migration; `tray/src/settings.rs` | (a) Đổi phiên bản = **reset mọi tuỳ chọn gõ** (kiểu gõ VNI, bảng mã…) về mặc định ở *mỗi* lần cập nhật — người dùng VNI phải chọn lại sau mỗi bản. (b) `SettingsController` (6 tab, debounce) không được UI thật dùng — mã chết ~300 dòng. | Quyết định sản phẩm/kiến trúc — đề xuất giới hạn migration theo phiên bản không tương thích đã biết; gỡ hoặc nối `SettingsController`. Để ngỏ | 📝 Open |

## 3. Nhật ký tiến độ

- 2026-10-07 — P0 baseline xanh; lập plan.
- 2026-10-07 — P1 `core/` + P2 `ffi/` rà xong: CR-01…CR-05 sửa + test; gate fmt/clippy (Linux+Windows)/test/replay 4 adapter/verify xanh.
  Đã rà và **không** thấy lỗi: `method/{telex,vni,viqr,simple_telex}`, `transform/{tone,undo,stroke,vowel_table,diacritic_style,charset}`,
  `validate`, `post/{macro,caps,emoji,quick_telex}`, `ffi/src/settings.rs`, header C (bản mac giống hệt bản gốc).
- 2026-10-07 — P3 rà xong: CR-06 (S3), CR-07 sửa; CR-08 ghi nhận. `ipc` codec (giới hạn 64 KiB, từ chối trailing/UTF-8 sai),
  `config/doc.rs` (ghi nguyên tử 0600, sao lưu `.bak`), `field-detect` (generation gate, Unknown ⇒ Passthrough) không có lỗi.
- 2026-10-07 — P4 `windows-tsf` rà xong: CR-09 (pipe impersonation), CR-11, CR-12 sửa; CR-10 ghi nhận. Edit session (gate S3 đọc lại mỗi phím,
  self-heal caret, fail-open khi `RequestEditSession` từ chối), `replay.rs` (CUAS), `compose.rs` (kế hoạch thuần, đã có test) không thấy lỗi.
  Ghi chú chưa kiểm được trên Windows thật: `pending_eaten_vk` giả định mọi app gọi `OnKeyDown` sau `OnTestKeyDown` = TRUE (finding E6 cũ).
- 2026-10-07 — P5 hook + tray rà xong: CR-13, 15–19 sửa; CR-14, CR-20 ghi nhận. Pipe server (DACL user+SY+BA, `PIPE_REJECT_REMOTE_CLIENTS`,
  PID từ kernel thay vì tin message, frame ≤ 64 KiB, timeout payload 5 s, writer riêng có hàng đợi giới hạn), `svc.rs` (persist-trước-publish),
  `--stop` (không TerminateProcess), LL hook của tray chỉ quan sát — không thấy lỗi.
