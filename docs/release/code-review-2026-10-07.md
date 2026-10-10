# Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)

> **Vòng 2 (2026-10-08/09):** `code-review-2026-10-09-round2.md` — 98 finding R2-*, kênh
> Store MSIX, ký số, QA Windows/Linux/macOS, engine. Trạng thái CR-20/CR-34 cập nhật ở dưới.

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
| P6 | `cli/` (register, doctor, replay, verify) | 5.1k | Registry/đường dẫn, quyền, xử lý lỗi | ✅ |
| P7 | `adapters/linux-*` (C/C++) | 4.5k | Buffer overflow, socket, GLib/fcitx lifetime | ✅ |
| P8 | `adapters/macos-*` (Swift) | 5.6k | Socket, IMK client lifetime, event tap | ✅ (rà tĩnh) |
| P9 | `scripts/` `packaging/` `installer/` `tools/` `.github/workflows/` | 6k | Shell quoting, injection trong workflow, quyền token | ✅ |
| P10 | `xtask/` `tools/bench` `fuzz/` | 3k | Tính đúng của gate | ✅ |
| P11 | Chạy lại toàn bộ gate + cập nhật báo cáo + push | — | — | ✅ |

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
| CR-20 | P3 | `tray/src/svc.rs` migration; `tray/src/settings.rs` | (a) Đổi phiên bản = **reset mọi tuỳ chọn gõ** (kiểu gõ VNI, bảng mã…) về mặc định ở *mỗi* lần cập nhật — người dùng VNI phải chọn lại sau mỗi bản. (b) `SettingsController` (6 tab, debounce) không được UI thật dùng — mã chết ~300 dòng. | (a) **Vòng 2 (`7f5082c`)**: đổi phiên bản không còn reset tuỳ chọn gõ/override, không unregister→register. (b) `SettingsController` vẫn chưa nối — để ngỏ | ✅ Fixed (a) / 📝 Open (b) |
| CR-21 | P1 (S2 riêng tư) | `cli/src/doctor.rs` `export_diagnostics_zip` | Gói chẩn đoán (`doctor --export`, CONTRIBUTING hướng dẫn **đính kèm vào issue công khai**) chứa nguyên `config.json`: nội dung gõ tắt (địa chỉ, số điện thoại, email…), emoji, từ điển cá nhân — chỉ tên user được che. | `redact_user_content`: `macros`/`emoji`/`english_words` → `"<REDACTED: n mục>"`; config hỏng chỉ ghi kích thước. Test `export_config_never_contains_user_authored_content` | ✅ Fixed |
| CR-22 | P2 (bảo mật) | `cli/src/register.rs` `say` | CLI chạy **elevated** (`register --scope machine` từ tray/bộ cài) append log vào `%LOCALAPPDATA%\TextVN\logs\register.log` — thư mục người dùng ghi được; junction/symlink ở đó biến lần ghi log thành ghi file tuỳ ý bằng quyền admin (CWE-59). | Bỏ ghi log file nếu `TextVN`, `logs` hoặc `register.log` là reparse point (symlink/junction, không đi theo link). Còn rủi ro TOCTOU nhỏ — ghi chú | ✅ Fixed |
| CR-23 | P3 (test) | `cli/src/replay.rs` `load_cases` | Case `.keys` không có dòng `:expect*` nào luôn PASS — corpus gõ lại mà quên khẳng định vẫn xanh. | Từ chối (parse error, exit 2). Toàn bộ 4 corpus hiện có vẫn xanh | ✅ Fixed |
| CR-24 | P1 (crash, tiềm ẩn) | `adapters/linux-common/src/ipc_client.c` `lc_ipc_send_frame` | Gửi bằng `write()` lên Unix socket **không** `MSG_NOSIGNAL`: server IPC thoát giữa chừng → `SIGPIPE` mặc định **giết process chủ** — tiến trình `textvn-ibus-engine` hoặc **cả daemon fcitx5** (addon nạp in-process) → mất bàn phím. Bản macOS đã chặn (`SO_NOSIGPIPE`), Linux thì chưa. Gói Linux hiện **chưa có** server IPC nên lỗi nằm im — nổ ngay khi có server. | `sendmsg(…, MSG_NOSIGNAL)` header+thân 2 iovec trong một lời gọi, chờ ngắn khi `EAGAIN`; gửi lỗi (handshake/toggle) → đóng kết nối, offline. Test `test_send_after_server_close_does_not_raise_sigpipe` | ✅ Fixed |
| CR-25 | P1 | `adapters/linux-common/src/ipc_client.c` `lc_ipc_recv_frame`/`poll` | Socket non-blocking: frame tới **từng phần** rồi `EAGAIN` → các byte đã đọc bị **vứt** → luồng lệch (thân bị hiểu thành header độ dài) → ngắt kết nối/đọc rác. | Đệm nhận theo client (`rbuf`, 64 KiB + 1), chỉ xử lý frame đủ, kết thúc chuỗi tại chỗ; độ dài > 64 KiB → đóng kết nối. Test `test_partial_frame_is_buffered` (đỏ trên code cũ) | ✅ Fixed |
| CR-26 | P2 (test) | `adapters/linux-common/tests/test_ipc_client.c` `test_global_snapshot_on_connect` | Test **không ghi 4 byte header** của frame Snapshot (rác trên stack) — UB: Release pass nhờ may, Debug/ASan đỏ. Đây chính là lỗi "fail không xác định được nguyên nhân kernel" ghi trong comment của test bên trên. Kèm: debug `FIONREAD` ghi `int` vào `long`. | Ghi header; debug in theo `recv`. 4/4 test xanh cả Release lẫn Debug+ASan/UBSan | ✅ Fixed |
| CR-27 | P3 | `adapters/linux-common/src/ipc_client.c` `lc_ipc_client_toggle_vi_en` | `app_id` (tên chương trình) chèn thẳng vào JSON — `"`/`\\` làm frame hỏng, server đóng kết nối. | Escape `"`, `\\`, bỏ ký tự điều khiển | ✅ Fixed |
| CR-28 | **P0** (mất chữ) | `adapters/macos-imk/Sources/IMKLib/TextVNInputController.swift` `handle` | Self-heal "app đã tự commit marked" (click chuột sang chỗ khác) chạy **sau** `engine.key()`: outcome đã tính với từ cũ (`delete_count = owned`) vẫn được áp sau khi reset → `preeditReplace` gọi `deleteBackward(owned)` **tại vị trí con trỏ mới** — xoá chữ không liên quan. Audit 2026-09-30 đã ghi "self-heal trước engine" nhưng thứ tự bị đảo lại sau đó (hồi quy). | Self-heal (và lấy `target`) chuyển lên **trước** `engine.key()`; thiếu target thì PASS mà không đụng engine | ✅ Fixed (CI macOS kiểm build) |
| CR-29 | P1 | `TextVNInputController.handle` (chord) | Cmd/Ctrl/Alt chord `return false` trước engine → từ đang gõ giữ nguyên: Cmd+← rồi gõ dấu xoá `owned` ký tự ở vị trí mới (non-preedit — mặc định macOS `non_preedit = true`), hoặc marked bị đè. | `finishWordBeforeChord`: chốt marked (như B13) + `engine.reset()` trước khi trả phím, như TSF | ✅ Fixed (CI macOS) |
| CR-30 | P1 (B2) | `TextVNInputController.handle` + `ApplyReplace.commit` | Enter/Tab ở ranh giới (từ đã biến đổi): engine trả `COMMIT{"\n"}`/`RESTORE{…"\t"}` → macOS **chèn "\n"/"\t" thành text và nuốt phím** → Messages/Slack xuống dòng thay vì gửi, Tab không chuyển ô. TSF/Linux đã tách ký tự ranh giới và trả phím thật. Corpus không thấy được (chỉ so text). | `withoutNativeBoundary`: bỏ `\n`/`\r`/`\t` cuối COMMIT/RESTORE khi phím là Enter/Tab, áp phần còn lại, `return false` để app nhận phím. `KeyOutcome` có `init` public. Test `NativeBoundaryTests` | ✅ Fixed (CI macOS) |
| CR-31 | P1 | `core/src/lib.rs` `Engine::key` | Ký tự điều khiển trong `ch` (layout macOS: `\x1C`–`\x1F` mũi tên, `\x01`/`\x04` Home/End, `\x7F` Delete xuôi, `\x03` Enter phím số) bị coi là **ranh giới** → Preedit trả `COMMIT{"\x1C"}` → adapter chèn ký tự điều khiển vào văn bản và nuốt phím. TSF/Linux tự lọc, macOS thì không. | Engine lọc: control (trừ `\n` `\r` `\t`) = phím điều hướng (bỏ từ, PASS). Test `control_chars_from_layout_are_navigation_not_boundary`; corpus 4 adapter xanh | ✅ Fixed |
| CR-32 | P1 (mất chữ) | `adapters/macos-tap/Sources/TextVNTap/TapInjector.swift` | (a) `inject`: ngân sách 64 event dùng chung cho Backspace **và** chèn → xoá đủ 64 ký tự (Esc/khôi phục từ dài, gõ tắt) thì không chèn gì mà vẫn trả `true`. (b) `selectAndReplace`: chỉ chọn ≤ 32 ký tự (phần dư nhân đôi) và chèn cả chuỗi trong 1 event (CGEvent ≤ 20 UniChar → cắt chữ). (c) Callback nuốt phím gốc kể cả khi inject thất bại. | Ngân sách chỉ cho Backspace; `selectAndReplace` chọn đủ (vượt → về Backspace) + chèn theo khối 20; inject lỗi → trả phím gốc. Ghi chú: `TapKeyHandler` chưa có hiện thực production (module đang "ngủ") | ✅ Fixed (CI macOS) |
| CR-33 | P2 (mất cấu hình) | `adapters/macos-app/Sources/TextVNAppLib/ConfigModel.swift` | Lưu config = mã hoá lại toàn bộ struct: **mất `emoji[]`** và trường `when` của gõ tắt (`vi_on` thành `always`) mỗi lần đổi một tuỳ chọn trên macOS (Windows/Linux giữ nguyên qua `SettingsDoc`). | Thêm `EmojiEntry`, `emoji`, `MacroEntry.when` (optional, tham số mặc định — tương thích nguồn). Test `testSavePreservesEmojiAndMacroWhen` | ✅ Fixed (CI macOS) |
| CR-34 | P3 | `schemas/ipc.v1.md`; Swift `IpcServer`/`IpcClient` | Schema không ghi kiểu trường: Rust/Linux dùng `appdb_version` số (`u32`), Swift gửi/nhận chuỗi `"1.0"` — hai cài đặt không nói chuyện được với nhau nếu trộn. | **Vòng 2:** schema ghi kiểu từng trường; `IpcServer.swift` gửi số `1`; client IMK nhận số (và chuỗi của app bản cũ). Test `testSnapshotMessageMatchesClosedSchema`, `testDecodeWireSamplesIpcV1` (CI `ci-macos`) | ✅ Fixed |
| CR-35 | P2 | `installer/windows/portable/uninstall.ps1` | (a) Trả Ctrl + Shift cho Windows mỗi khi `Layout Hotkey = 3` — kể cả khi chính user tự chọn "Not assigned" (không do TextVN đặt); lệch với tray sau CR-17 (chỉ trả khi có marker). (b) `Remove-Item -Path $dir`: thư mục tên có `[`/`]` bị hiểu là wildcard → không xoá gì, rơi sang RunOnce. | Chỉ restore khi có marker `%APPDATA%\TextVN\ctrl_shift_default_applied`, xoá marker sau gỡ; `-LiteralPath`. Bỏ biến `$hotkey` thừa | ✅ Fixed |
| CR-36 | P1 | `build-release.ps1` | (a) `textvn-tsf-x86.dll` **không nằm trong danh sách ký** (Authenticode/SignPath) — DLL được nạp vào mọi tiến trình 32-bit (Zalo, Office x86) là binary duy nhất chưa ký trong gói đã ký → SmartScreen/AV. (b) Không kiểm tra target `i686-pc-windows-msvc` → build x86 chỉ fail sau khi đã chạy hết clippy/test. (c) `Write-Fail "…0x{0:X4}" -f $machine` thiếu ngoặc → in nguyên chuỗi định dạng. | Thêm DLL x86 vào `$ReleaseSignFiles`; kiểm/cài cả hai target; bọc ngoặc `-f` | ✅ Fixed |
| CR-37 | P1 | `installer/windows/TextVN-setup.iss` `[Code]` | (a) Nâng cấp khi app 32-bit đang mở: `textvn-tsf-x86.dll` bị nạp nhưng `RenameLockedTsfDll` chỉ đổi tên bản x64 → hộp thoại file-in-use (cài im lặng thì thất bại). (b) Dọn backup cũ gọi `DeleteFile(AppDir + '' + Name)` — **thiếu `\`** nên không bao giờ xoá được, `.old-*` tích dần mỗi lần nâng cấp. (c) Gỡ cài đặt không hẹn xoá DLL x86 còn khoá → sót file + thư mục trong Program Files. | `RenameLockedFile(FileName)` dùng chung cho cả hai DLL, sửa đường dẫn; `[UninstallDelete]` thêm `textvn-tsf-x86.dll.old-*`; `schedule-delete` thêm DLL x86. `check_iss_tabs` xanh | ✅ Fixed (cần kiểm trên Windows thật) |
| CR-38 | P2 (hardening) | `.github/workflows/*.yml` | `actions/checkout` mặc định ghi token vào `.git/config` cho mọi bước sau — kể cả job `publish` (`contents: write`) và các bước chạy action bên thứ ba. Không thấy script injection (`${{ github.event.* }}` không đi vào `run:`), mọi workflow đã khai báo `permissions`. | `persist-credentials: false` cho cả 26 checkout (không job nào `git push`; `gh` dùng `GH_TOKEN`). **Khuyến nghị (Open):** ghim action bên thứ ba (`sigstore/cosign-installer`, `taiki-e/install-action`, `Swatinem/rust-cache`, `dtolnay/rust-toolchain`, `fsfe/reuse-action`) theo commit SHA — **vòng 2 (`c31d54c`)**: đã ghim 7 action bên thứ ba bằng SHA tra qua `git ls-remote` | ✅ Fixed |
| CR-39 | P2 | `docs/release/signing.md`, `tools/release/sign-artifacts.sh` | Hướng dẫn kiểm chữ ký phát hành **không chạy được**: `gpg --verify …exe.sig` (script tạo `.asc` — `--armor`), lệnh trộn 0.2.26/0.2.27; `cosign verify-blob --certificate-identity-regexp …release-candidate\.yml…` trong khi workflow ký là `release.yml` → người dùng làm theo luôn nhận "no matching signatures". File còn chứa 1 byte NUL thật (trong ví dụ `b'\x00'`) → git/grep coi là binary, diff không review được. | Sửa tên file/phiên bản/workflow, thay NUL bằng `\x00`; script in đúng tên `.asc` | ✅ Fixed |
| CR-40 | P2 (gate mù) | `fuzz/fuzz_targets/ffi_key.rs`, `ffi/tests/abi_invariants.rs`, `fuzz/README.md` | Fuzz/stress ABI **không thể** tìm ra CR-01: tối đa 64 phím/input (cần ≥ 65 phím chữ liền nhau) và không có bất biến "không xoá lẹm text có sẵn" — chỉ kiểm `delete_count ≤ 64`. README: lệnh "nightly.yml" không tồn tại và thiếu `--features ffi-fuzz` (chạy là lỗi required-features); công thức seed `key_duocj` sai định dạng input của harness. | Fuzz: tới 512 phím, chế độ "giữ phím" (lặp ≤ 32), bất biến 6 = mô hình độ dài document bảo thủ (chỉ đếm dư); seed corpus `fuzz/corpus/ffi_key/` (gồm `held_key_dep[_preedit]`). Stable: test `tu_dai_va_giu_phim_khong_xoa_lem_text_co_san` (60 × 512 phím, mọi strategy) — **đỏ trên engine trước CR-01**, xanh hiện tại; harness chạy 3000 input ngẫu nhiên không báo động giả. README sửa lệnh + mô tả định dạng | ✅ Fixed |
| CR-41 | P3 | `xtask/src/main.rs` `usage()` | `cargo xtask help` không liệt kê `preflight` — đúng lệnh mà quy trình bắt buộc chạy trước commit/tag. | Thêm dòng usage | ✅ Fixed |

## 4. Kết quả cuối (P11)

**41 finding** — P0: 2 · P1: 16 · P2: 13 · P3: 10. **Đã sửa 36** (kèm test tái hiện đỏ-trước-xanh-sau
ở mọi chỗ chạy được trong container), **để ngỏ 5** + 1 phần (CR-38: ghim SHA action).

| Gate (sau sửa) | Lệnh | Kết quả |
|---|---|---|
| fmt | `cargo fmt --all --check` | ✅ |
| clippy Linux | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| clippy Windows (cross-check) | `… --target x86_64-pc-windows-msvc -- -D warnings` | ✅ |
| test | `cargo test --workspace` | ✅ 368 test (Linux), 0 fail |
| replay | headless 39 · tsf 117 · mac 153 · linux 39 | ✅ |
| ABI | `textvn-cli verify` (11 export) · `sizes` (532 B) | ✅ |
| xtask | `check-tables` · `check-win-corpus` (72) · `check-mac-corpus` (114) · `check-mac-targets` (12) · `check-version-sync` (14) | ✅ |
| repo-hygiene | `check_doc_links` · `check_iss_tabs` · `check_no_injection_apis` · `check_ps1_ascii` | ✅ |
| linux-common (C) | ctest Release + Debug ASan/UBSan | ✅ 4/4 × 2 |
| fuzz `ffi_key` (harness mới) | 3000 input ngẫu nhiên + 3 seed, không instrument (container không có nightly) | ✅ |
| bench | `ime_key` p50 451 ns / p99 943 ns (ngân sách 0.5 ms) | ✅ |

**Chưa kiểm được trong container — giao cho CI/máy thật:** build + `swift test` macOS (CR-28–33, CI `ci-macos`);
PowerShell/Inno (CR-35–37, job Windows của CI/release — nhất là nâng cấp khi app 32-bit đang mở DLL x86);
IBus/Fcitx5 e2e (CI `linux-adapters`).

**Để ngỏ — cần quyết định sản phẩm hoặc máy Windows đa phiên/thật:**

| ID | Việc | Vì sao chưa làm |
|---|---|---|
| CR-10 | Pipe theo phiên (`textvn-ipc-v1-<SessionId>`) + `FIRST_PIPE_INSTANCE` | Đổi hợp đồng đặt tên 4 thành phần; cần Fast User Switching/RDS để kiểm |
| CR-14 | Hook bỏ qua tiến trình đang có TIP TextVN active; reset từ khi click | Cần Windows thật; thay đổi giao thức IPC |
| CR-20 (b) | Gỡ/nối `SettingsController` (phần migration đã sửa ở vòng 2) | Quyết định kiến trúc |
| CR-08 | Gõ tắt giữ khoảng trắng đầu/cuối, trigger có `=` | Đổi định dạng text ảnh hưởng 3 UI |

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
- 2026-10-07 — P6 `cli/` rà xong: CR-21–23 sửa. `register.rs` (ghi registry qua Win32 API, `input.dll` chỉ từ System32, ACL AppContainer,
  `schedule_delete` cho uninstaller), `verify.rs` (đối chiếu header ↔ Rust có test chống pass rỗng) không thấy lỗi khác.
- 2026-10-07 — P7 Linux rà xong: CR-24–27 sửa (linux-common build + test với ASan/UBSan trong container). IBus `engine.c`, Fcitx5 `engine.cpp`
  (mọi callback `try/catch`, fail-open), `compose.c` (giới hạn `LC_COMP_MAX`), `utf.c`, `log.c` (không ghi `/tmp` chung), `linux-settings`
  (ghi qua C API dùng chung) — không thấy lỗi khác. IBus/Fcitx5 thật không có trong container (CI `linux-adapters` phủ e2e).
- 2026-10-07 — P8 macOS rà tĩnh: CR-28–33 sửa, CR-34 ghi nhận. **Swift không có toolchain trong container** — thay đổi giữ tối thiểu,
  tương thích nguồn; build + `swift test` do CI `ci-macos` kiểm. IpcClient Swift (đệm frame, `SO_NOSIGPIPE`, decode chặt), IpcServer (0600,
  `getpeereid`), FieldDetect (CFTypeID trước downcast), KeyTranslator — không thấy lỗi khác.
- 2026-10-07 — P9 đóng gói/CI rà xong: CR-35–39 sửa. Shell (`bash -n` toàn bộ; `packaging/linux` install/uninstall có `set -eu`,
  quote đầy đủ, không `rm -rf` biến rỗng), ký phát hành (`tools/release/sign-artifacts.sh`: GPG `.asc` + clearsign `SHA256SUMS.txt` + cosign keyless; CR-39 chỉ
  là tài liệu), `release.yml` (draft-first, kiểm tag = version, payload đúng số lượng, không có hook), `build-msix.ps1`,
  `sign-signpath.ps1`, `virustotal-scan.ps1` (khoá API chỉ trong header, không in ra log) — không thấy lỗi khác. `check_ps1_ascii`,
  `check_iss_tabs` xanh. PowerShell/Inno không chạy được trong container — thay đổi nhỏ, kiểm bằng job Windows của CI/release.
- 2026-10-07 — P10 gate rà xong: CR-40, CR-41 sửa. `xtask` (`check-tables`/`check-*-corpus`/`check-mac-targets`/`check-version-sync`
  đều xanh; `format_rust` đóng stdin trước khi chờ — không deadlock; version-sync đủ 14 chỗ khớp thông báo của `release.yml`),
  `preflight` (exit code thật, không pipe), `config_parse`/`appdb_parse` fuzz, `tools/bench` (hồi quy theo p50, p99 theo ngân sách
  tuyệt đối) — không thấy lỗi khác. Bench Linux sau mọi thay đổi engine: `ime_key` p50 451 ns / p99 943 ns (ngân sách 0.5 ms).
- 2026-10-07 — P11: chạy lại toàn bộ gate (bảng §4) — xanh hết; cập nhật báo cáo, push nhánh `ccr-78d8ca29-1v3uym`.
