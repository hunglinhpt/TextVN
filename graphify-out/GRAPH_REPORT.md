# Graph Report - TextVN  (2026-10-11)

## Corpus Check
- 341 files · ~468,690 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 295 file(s) not represented in the graph (top: .keys 248, .toml 10, (none) 6)

## Summary
- 5755 nodes · 10877 edges · 395 communities (325 shown, 70 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 704 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `deaff577`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- core/src/lib.rs
- Result
- cli/src/replay.rs
- config/src/lib.rs
- text
- verify.rs
- windows-hook/src/main.rs
- toml.rs
- xtask/src/main.rs
- ffi/src/lib.rs
- strategy/src/lib.rs
- vowel_table_generated.rs
- bench/src/main.rs
- cli/src/register.rs
- tone.rs
- keymap.rs
- vni.rs
- validate.rs
- macro.rs
- telex.rs
- P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết
- windows-tsf/src/lib.rs
- restore_en.rs
- P0-2 — Hợp đồng FFI v1 (nguồn sự thật duy nhất)
- P1-1 — TSF ADAPTER (Windows) — Solution chi tiết
- P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết
- check_mac_targets.rs
- engine.c
- P2-1 — IMK ADAPTER (macOS) — Solution chi tiết
- C. LINUX + SHARED
- svc.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- store.rs
- windows-hook/src/lib.rs
- Chi tiết từng mục
- tip.rs
- simple_telex.rs
- P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator
- matchClause
- properties
- properties
- win_corpus_cases.rs
- `ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)
- P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết
- P2-4 — UI, Config, IPC, Packaging & Release (macOS) — Solution chi tiết
- properties
- $defs
- TextVN — Workflow & Ground Rules thống nhất cho mọi agent
- AGENT HANDBOOK — Quy ước làm việc cho mọi agent/contributor
- P1-5 — TEST PLAN (Windows) — Solution chi tiết
- M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003
- P2-5 — TEST PLAN (macOS) — Solution chi tiết
- A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004
- Result
- doctor.rs
- StrategyResolver
- P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết
- field-detect/src/lib.rs
- T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004
- settings_dialog.rs
- IpcMessage
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- ime_result_v1
- textvn-ffi
- items
- compose.rs
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- tray/src/lib.rs
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- IMKLibTests.swift
- P3-0 — LINUX MASTER PLAN (Phần 3)
- TextVNInputController
- undo.rs
- settings_window.c
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- T6 — Test & hardening (tuần 15–18)
- B. macOS
- Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)
- Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại
- textvn-common.sh
- appdb.v1.schema.json
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- TextVN — Chỉ mục tài liệu & Giao thức Review
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- engine
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- class.rs
- IpcServer
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- ByteCursor<'a>
- Bảng kiểm tra đồng bộ giao diện cài đặt (Settings Parity Checklist)
- when
- P1 — REVIEW LOG (Phần 1: Windows)
- hotkey.rs
- entry
- id
- auto_capitalize
- auto_restore_english
- MarkedState
- english_words
- P1-6 — TASKS Windows (WBS) — Nhận việc từng task
- P2-6 — TASKS macOS (WBS) — Nhận việc từng task
- KeyOutcome
- config_version
- free_marking
- core
- windows
- adr/README.md
- tsf-min
- EngineSession
- xtask
- WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"
- Result
- Strategy
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- encode_frame
- .character
- bool
- store_win.rs
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- common_words.rs
- guids.rs
- uia_spike.ps1
- TsfShared
- do_register
- Project Common Errors — Sổ lỗi chung toàn dự án
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- compose.c
- ipc_client.c
- tsf-min/src/lib.rs
- P0 — REVIEW LOG (Phần 0: Nền tảng chung)
- SettingsDoc
- keys_generated.rs
- tsf-min/src/register.rs
- win32-uia.lib.ps1
- trace.rs
- engine.cpp
- IpcClient
- ipc_client.rs
- bitset
- verify_targets.ps1
- HWND
- Performance Audit — TextVN Windows Platform
- show_macro_editor
- generate_tray_icons.py
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- windows-tsf/src/replay.rs
- check-pe-imports.py
- invariants.rs
- build-release.ps1
- AppDelegate
- check_no_injection_apis.py
- EventTapController
- Engine
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- DialogContext
- Foundation
- .CreateInstance
- IpcServer
- ime_result
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- OwnerRule
- super
- ime_context
- rules_win.rs
- Security Policy
- Đóng góp cho TextVN
- SvcManager
- MockHandler
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- macro_text.rs
- ConfigStore
- ipc_server.rs
- AppDb
- ViState
- String
- e2e_ibus.c
- generate_app_icons.py
- ime_key
- EndCompositionSession_Impl
- e2e_fcitx5.py
- AXSnapshot
- key_event.rs
- stdio
- TextVNConfig
- install_linux.sh
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- ime_field_body
- securitystate
- P3-6 — TEST PLAN (Linux) — Solution chi tiết
- show_dialog_on_startup
- ffi/src/settings.rs
- IMKTextTarget
- foreground.rs
- resolve.rs
- AutostartManager
- FieldContext
- Path
- TapPermission
- diacritic_style.rs
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- TapKeyHandler
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- quick_telex.rs
- HookEngine
- Hướng dẫn phát triển TextVN
- Báo cáo dựng & kiểm thử — TextVN
- remove_added_languages
- A. WINDOWS
- ime_key
- .inject
- TextVNAppTests
- Option
- smoke-imk.sh
- textvn_ffi
- Kế hoạch ký số (code signing) — TextVN
- generate_store_art.py
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- test-typing.ps1
- env-mac — Môi trường build macOS (MAC-001)
- ffi_key.rs
- Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)
- TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV
- tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)
- PackageDescription
- TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- quick_telex
- e2e-linux.sh
- mem-check.sh
- soak.sh
- .start
- method
- output_charset
- build-macos.sh
- stage-linux.sh
- build-rust.sh
- EngineOptions
- postinstall
- uninstall-check.sh
- install_macos.sh
- notarize-macos.sh
- package-macos-pkg.sh
- uninstall_macos.sh
- functional
- iostream
- keysink
- settings_window
- signal
- ConfigWatcher
- path
- .onEvent
- command
- ModifierToggle
- [0.2.10] — 2026-10-03
- ime_result
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- [0.2.0] — 2026-09-30
- WORDLIST_LAYOUT
- .new
- ActionKind
- [0.2.13] — 2026-10-03
- 3. Nhật ký finding
- [0.2.14] — 2026-10-03
- [0.2.9] — 2026-10-02
- Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất
- Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)
- 2. PRODUCT OWNER VIEW
- pre_translate_message
- Word
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- Added
- Option
- [0.2.3] — 2026-10-01
- Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- 4. ENGINEER VIEW
- 2. Kết quả kiểm thử
- Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening
- Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)
- Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành
- ffi/build.rs
- Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent
- MACRO_LAYOUT
- Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được
- Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath
- Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục
- Diagnostics
- [0.2.4] — 2026-10-01
- [0.2.12] — 2026-10-03
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- verify-msix.py
- [0.2.15] — 2026-10-03
- [0.2.16] — 2026-10-03
- registration_ok
- 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft
- Option
- Option
- try_read_pipe_frame
- textvn_ibus_modifier_kind
- Nộp TextVN lên Microsoft Store — gói MSIX
- package_family_full_names
- run_store_guard
- 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành
- P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)
- 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU
- Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store
- OutputStrategy
- SecurityState
- P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow
- P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết
- Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI
- 1. Điền form Partner Center — từng ô một
- 3. SOLUTION ARCHITECT VIEW
- 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"
- StageInfo
- Bool
- GlobalHookContext
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- IMPLEMENTATION STATUS — adapter macOS (Farch-4)
- 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)
- 5. TECHNICAL EXPERT VIEW
- windows-hook/build.rs
- [0.2.28] — 2026-10-10
- 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối
- 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)
- Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)
- Trạng thái ký số macOS — Developer ID, notarization, stapling
- 6. AUDIT EXPERT VIEW
- tray/build.rs
- [0.2.8] — 2026-10-02
- .tapHandle
- [0.2.20] — 2026-10-04
- [0.2.26] — 2026-10-05
- [0.2.29] — 2026-10-10
- [0.2.5] — 2026-10-02
- Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)
- sign-artifacts.sh
- [0.2.22] — 2026-10-04
- [0.2.25] — 2026-10-05
- [0.2.7] — 2026-10-02
- Bản 0.2.16 — portable tự chữa trên Win11 24H2+ (B7) + hết ghost khi gỡ (0.2.15)
- Bản 0.2.4 — đăng ký không cần admin + clean composition + Ctrl+Shift single-path
- Bản 0.2.6 — chốt đăng ký TSF VI+EN, hết ghost registration
- Bản 0.2.9 — từ điển EN cá nhân + quy tắc ưu tiên + preflight 17 gate
- error_success
- unmark_as_unmark_tone

## God Nodes (most connected - your core abstractions)
1. `AppDelegate` - 65 edges
2. `SvcManager` - 54 edges
3. `TextVNInputController` - 50 edges
4. `MarkedState` - 37 edges
5. `TsfShared` - 37 edges
6. `TextVNEngine` - 35 edges
7. `Engine` - 35 edges
8. `Changelog` - 35 edges
9. `Báo cáo dựng & kiểm thử — TextVN` - 35 edges
10. `IpcServer` - 32 edges

## Surprising Connections (you probably didn't know these)
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  AUDIT_REPORT_PRODUCTION_READY.md → adapters/linux-common/src/ipc_client.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md → adapters/linux-common/src/ipc_client.c
- `Đã làm ĐÚNG: `lc_ipc_client_poll` + rbuf per-client stash (CR-25 fix +` --references--> `lc_ipc_client_poll()`  [INFERRED]
  docs/release/AUDIT-3PLATFORM-2026-10-07.md → adapters/linux-common/src/ipc_client.c

## Import Cycles
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`

## Communities (395 total, 70 thin omitted)

### Community 0 - "core/src/lib.rs"
Cohesion: 0.13
Nodes (42): ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, auto_capitalize_after_dot_and_enter(), auto_capitalize_needs_space_after_dot(), auto_capitalize_only_first_letter_of_word(), backspace_after_sentence_end_cancels_auto_capitalize() (+34 more)

### Community 1 - "Result"
Cohesion: 0.12
Nodes (23): ClassFactory_Impl, CompSink_Impl, EditSession_Impl, KeySink_Impl, log(), BOOL, c_void, FnOnce (+15 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.08
Nodes (48): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, case_without_any_expectation_is_rejected(), Cmd (+40 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.07
Nodes (29): Config, ConfigError, default_json(), defaults_match_p0_3_table(), DiacriticStyle, EmojiEntry, FULL_EXAMPLE, full_example_parses() (+21 more)

### Community 4 - "text"
Cohesion: 0.20
Nodes (12): check(), text(), 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`), 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`), 3. Bảng xung đột có thể xảy ra và cách giải, 4. Nguyên tắc điều chỉnh thiết kế (ghi để không lặp tranh luận), Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên, 1.1 Bảng tổng hợp (+4 more)

### Community 5 - "verify.rs"
Cohesion: 0.10
Nodes (33): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+25 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.09
Nodes (36): caps_lock_bit(), CTRL_DOWN, empty_ime_result(), get_active_modifiers(), get_process_name_for_window(), GLOBAL_ENABLED, HOOK_MAIN_THREAD_ID, IN_INJECTION (+28 more)

### Community 7 - "toml.rs"
Cohesion: 0.15
Nodes (18): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+10 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.13
Nodes (22): banner(), emit(), format_rust(), gen_tables(), KEYMAP_MAC_OUT_RUST, KEYMAP_MAC_OUT_SWIFT, KEYMAP_MAC_TOML, KEYS_OUT (+14 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.08
Nodes (37): ffi, guarded(), IME_ABI_VERSION, ime_appdb_verify(), ime_context_v1, IME_ERR_ABI, IME_ERR_CONFIG, IME_ERR_INTERNAL (+29 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.09
Nodes (20): IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO (+12 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.12
Nodes (16): A, A_BREVE, A_CIRC, E, E_CIRC, I, O, O_CIRC (+8 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.13
Nodes (26): black_box, F, textvn_field_detect, bench_field_switch_resolve(), bench_ime_key(), bench_parse_config(), bench_resolve(), best_of() (+18 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.06
Nodes (33): call_layout_or_tip(), call_layout_or_tip_spec(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, CLSID_INNER, clsid_registry_key() (+25 more)

### Community 14 - "tone.rs"
Cohesion: 0.12
Nodes (27): vni_viqr_marks_after_whole_cluster(), Horn, post_fixes(), push_key(), apply_key(), apply_key_literal_when_no_vowel(), apply_key_tone_and_undo(), apply_tone() (+19 more)

### Community 15 - "keymap.rs"
Cohesion: 0.06
Nodes (26): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, ESCAPE, F1, KeyEvent (+18 more)

### Community 16 - "vni.rs"
Cohesion: 0.09
Nodes (23): apply_key, late_stroke(), fold(), marker_to_tone(), n(), push_key(), DiacriticStyle, Option (+15 more)

### Community 17 - "validate.rs"
Cohesion: 0.13
Nodes (21): unmark(), coda_after_nucleus_ok(), CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI (+13 more)

### Community 18 - "macro.rs"
Cohesion: 0.15
Nodes (14): Emoji, emoji_glyph_is_multi_char(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches(), MacroDef (+6 more)

### Community 19 - "telex.rs"
Cohesion: 0.09
Nodes (16): codas, circum_pair(), f(), fold(), fold_with(), fold_with_caps(), has_uo_pair(), late_d_strokes_the_onset() (+8 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.06
Nodes (44): after_request_edit_session(), classify_tsf_field(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, external_edit_clears_ownership_before_next_replace(), focus_generation_rejects_old_uia_result_and_stays_fail_safe(), hotkey_toggle_switches_enabled_state_and_resets_buffer() (+36 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.09
Nodes (24): am_tiet_viet_that_khong_bi_restore(), canh_bao_cai_muc_mo_ho(), chars(), complete_word(), data_contains(), data_en_common_file_fires(), data_stop_en_file_is_self_consistent(), data_vn_common_file_is_valid() (+16 more)

### Community 23 - "P0-2 — Hợp đồng FFI v1 (nguồn sự thật duy nhất)"
Cohesion: 0.18
Nodes (10): 0. Bất biến (invariants), 1. Header (bản 1:1 với `ffi/include/textvn_ffi.h`), 2. Ngữ nghĩa `action` — adapter PHẢI làm đúng bảng này, 3. Vòng đời & thread model, 4. Mapping sang adapter preedit-vs-replace (quan trọng), 5. Mã lỗi & hành vi khi lỗi, 6. Versioning & kiểm chứng bằng CI, 7. Adapter walkthrough (Windows TSF — mẫu để adapter khác làm theo) (+2 more)

### Community 24 - "P1-1 — TSF ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping sang task — chi tiết ở `P1-6-TASKS.md`), 11. Chế độ test & chẩn đoán, 12. Failure modes & xử lý, 1. Tổng quan & quyết định, 2. Cấu trúc file (bám đúng `P0-1 §1`), 3. Vòng đời COM (lifecycle), 4. State machine của ThreadState, 5. Key flow (+9 more)

### Community 25 - "P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết"
Cohesion: 0.11
Nodes (18): 10. Playbook triển khai (mapping `P3-7-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module & repo, 3. Vòng đời IBus, 4. State machine (giống `P1-1 §4`, preedit thay composition), 5.1 Translate `keyval` → `ime_key_v1` (SPIKE `LNX-004` — RL1) (+10 more)

### Community 26 - "check_mac_targets.rs"
Cohesion: 0.13
Nodes (26): APP_DB, appdb(), CI_APPS, collect_preset_ids(), FIELD_ROLES, good(), has_preset_id(), Json (+18 more)

### Community 27 - "engine.c"
Cohesion: 0.13
Nodes (38): lc_modifier, lc_modifier_toggle, lc_modifier_toggle_down(), lc_modifier_toggle_reset(), lc_modifier_toggle_up(), lc_log(), test_modifier_toggle(), gboolean (+30 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+9 more)

### Community 29 - "C. LINUX + SHARED"
Cohesion: 0.12
Nodes (15): gpointer, tv_win_free(), Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`), C2. [TRUNG] Parser IPC dispatch bằng `strstr` toàn JSON → nhầm nhánh khi, C3. [TRUNG — kiến trúc] IPC Linux là dead path: không tồn tại server, C4. [TRUNG — latent] Override per-app "dính" sang app kế tiếp (ibus, C5. [THẤP] compose.c:92-94 — `cur->len == LC_COMP_MAX` → ghi, C7. [THẤP] GTK settings (settings_window.c:483,214-250) — `TvWin` free (+7 more)

### Community 30 - "svc.rs"
Cohesion: 0.11
Nodes (17): atomic_write_file(), corrupt_config_is_backed_up_not_lost(), default_config_dir(), failed_config_save_keeps_memory_and_version_unchanged(), failed_state_save_keeps_memory_and_version_unchanged(), Arc, BTreeMap, Path (+9 more)

### Community 31 - "P0-3 — Config schema · Preset (appdb) · Strategy model · IPC"
Cohesion: 0.12
Nodes (15): 1.1 Bảng trường (bắt buộc đánh số version để migrate), 1.2 Ví dụ đầy đủ, 1. Config người dùng — đường dẫn per-OS (1 nơi duy nhất cho mỗi OS), 2.1 Cấu trúc, 2.2 Ký số & cập nhật, 2. Preset theo ứng dụng — `appdb` (`schemas/appdb.v1.schema.json`), 3.1 Quy tắc phân quyền (đọc từ trên xuống, match là dừng), 3.2 Bảng lỗi đã biết → strategy kỳ vọng (dùng làm test oracle) (+7 more)

### Community 32 - "P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết"
Cohesion: 0.12
Nodes (15): 10. Task (chi tiết `P2-6-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Callback — luật bất biến (mirror `P1-2 §3`), 4. Chọn loại tap & permission (feature-detect 3 bước — `PLAN §5.1`), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 trên app không IMK), 5.3 `ForwardAsCommit` — chỉ chèn text, không xóa (terminal qua tap — hiếm). (+7 more)

### Community 33 - "properties"
Cohesion: 0.12
Nodes (16): type, enum, properties, enum, $ref, type, maxLength, type (+8 more)

### Community 34 - "store.rs"
Cohesion: 0.08
Nodes (25): APP_ID, ARG_AUTOSTART, ARG_GUARD, ARG_INSTALL, ARG_RELAY, build_command_line(), decide_store_run_value(), PFN (+17 more)

### Community 35 - "windows-hook/src/lib.rs"
Cohesion: 0.14
Nodes (22): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), CallbackDecision, empty_result(), engine_never_transforms_when_policy_says_pass(), EngineOutcome, hook_db(), HookMode (+14 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "tip.rs"
Cohesion: 0.09
Nodes (28): current_exe_name(), Default, GUID, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ITfKeystrokeMgr (+20 more)

### Community 38 - "simple_telex.rs"
Cohesion: 0.38
Nodes (4): fold(), n(), DiacriticStyle, Vec

### Community 39 - "P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator"
Cohesion: 0.15
Nodes (12): 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy), 2.1 Tổng quan, 2.2 Bảng lệnh (chính thức), 2.3 Ví dụ đầy đủ (phải chạy được ngay), 2.3b Lệnh bổ sung (bắt buộc có trong parser), 2.4 Tên phím chuẩn (case-sensitive), 2. Định dạng corpus `.keys`, 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán (+4 more)

### Community 40 - "matchClause"
Cohesion: 0.15
Nodes (13): type, $ref, $ref, matchClause, $ref, additionalProperties, minProperties, properties (+5 more)

### Community 41 - "properties"
Cohesion: 0.14
Nodes (14): default, type, $comment, default, enum, default, type, default (+6 more)

### Community 42 - "properties"
Cohesion: 0.15
Nodes (13): minLength, type, minLength, type, properties, expand, glyph, trigger (+5 more)

### Community 43 - "win_corpus_cases.rs"
Cohesion: 0.29
Nodes (11): Item, Iterator, all_cases(), CASES_PART1, CASES_PART2, CASES_PART3, CASES_PART4, CASES_PART5 (+3 more)

### Community 44 - "`ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)"
Cohesion: 0.29
Nodes (6): 1. Bất biến (P0-2 §0) — đọc trước khi viết adapter, 2. Header (nguyên văn — sinh từ `ffi/include/textvn_ffi.h`), 3. Chú giải: vòng đời chuẩn của adapter, 4. Lỗi thường gặp (rút từ bug B1–B13), 5. Khi nào phải bump `IME_ABI_VERSION`, `ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)

### Community 45 - "P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Mapping task, 1. Tray — `TextVN.exe` (chạy 1 instance, notify icon), 2. IPC server (P0-3 §4) — chạy trong tray, 3. Settings — hộp thoại Win32 (`tray/src/settings_dialog.rs`), 4. Cài đặt — `TextVN-setup-<ver>-windows-x64.exe` (Inno Setup 6.5+, x86_64), 5. Ký số & nguồn gốc (RW3 — antivirus), 6. Watchdog & process orchestration (tray), 7. Updater — crate `textvn-updater` (lib, chạy trong `TextVN.exe`) (+3 more)

### Community 46 - "P2-4 — UI, Config, IPC, Packaging & Release (macOS) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Task (chi tiết `P2-6-TASKS.md`), 1. Status menu — `TextVN.app` (LSUIElement, 1 instance), 2. IPC server (mirror `P1-4 §2`) — unix socket, 3. Settings — SwiftUI (cửa sổ 1, sidebar 6 tab — parity với `P2-0`/PLAN §2.3 M6), 4. Cài đặt & gỡ — `.pkg` (per-user, không sudo — PLAN §3.7), 5. Ký số & Gatekeeper (RM3), 6. Health & restart, 7. Updater (Ed25519 "Sparkle-style" — `PLAN §3.7`) (+3 more)

### Community 47 - "properties"
Cohesion: 0.17
Nodes (12): const, items, type, $ref, $ref, properties, appdb_version, entries (+4 more)

### Community 48 - "$defs"
Cohesion: 0.17
Nodes (12): maxLength, minLength, type, $defs, appIdentifier, fieldRole, matcher, semver (+4 more)

### Community 49 - "TextVN — Workflow & Ground Rules thống nhất cho mọi agent"
Cohesion: 0.10
Nodes (19): 10. Dọn dẹp — checklist trước khi đóng phiên / tick Done, 11. Cập nhật docs & nghiệp vụ — checklist khi "xong", 12.1 Workflow hiện có, 12.2 8 check của `repo-hygiene`, 12.3 Tham chiếu Marketplace (yêu cầu user 2026-09-27), 12.4 Workflow phát hành (`release-candidate`), 12. GitHub Actions — giữ repo sạch sau mỗi lần code, 1. Ground rules (G-IDs) — áp dụng mọi lúc (+11 more)

### Community 50 - "AGENT HANDBOOK — Quy ước làm việc cho mọi agent/contributor"
Cohesion: 0.18
Nodes (10): 1. Nguyên tắc bất di bất dịch (vi phạm = reject PR), 2. Cấu trúc thư mục & nơi đặt tài liệu, 3. Git & commit, 4. Definition of Done (DoD) cho một TASK, 5. Checklist Review (dùng cho cả 2 lượt — chi tiết hóa theo lượt), 6. Security rules khi dev & test, 7. Template ADR, 8. Thuật ngữ (giữ đúng, không dịch lung tung) (+2 more)

### Community 51 - "P1-5 — TEST PLAN (Windows) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Ma trận test (test pyramid), 2. Corpus Windows — `corpus/win/` (mục tiêu ≥ 300 case), 3. App matrix (bằng chứng `PLAN §5.4` — Windows subset 20 app), 4.1 Smoke script (PR không cần app matrix), 4. UIA harness — `tools/appcomptest` (crate `textvn-appcomptest`), 5. Perf (ngưỡng — liên kết `PLAN §5.5`), 6. Release gate (cổng ra RC), 7. Manual checklist (trước RC — ai đó chạy tay, ghi kết quả vào `docs/release/rc-checklist-win.md`) (+2 more)

### Community 52 - "M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003"
Cohesion: 0.18
Nodes (11): M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003, WIN-010 · TIP skeleton + COM lifecycle (L), WIN-011 · Key sink + engine PASS (M), WIN-012 · ReplaceEditSession `BackspaceType` (L) — `P1-1 §6.1`, WIN-013 · Composition + preedit + display attribute (L) — `P1-1 §7`, WIN-014 · Focus/commit-before-hide + `ime_reset` (M) — `P1-1 §3/§4`, WIN-015 · Hotkey preserve: toggle EN/VN `Ctrl+Shift+Space` + mặc định app EN (M), WIN-016 · ipc_client trong TSF (M) — `P1-1 §3` (bước 5–6) (+3 more)

### Community 53 - "P2-5 — TEST PLAN (macOS) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Ma trận test, 2. Corpus macOS — `corpus/mac/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix macOS (20 app — bằng chứng `PLAN §5.4` slice mac), 4.1 Smoke (PR, không cần matrix), 4. AX harness — `tools/mac/ax-driver` (Swift executable), 5. Perf (ngưỡng — `PLAN §5.5`), 6. Release gate (RC macOS), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-mac.md`; bản Windows: `rc-checklist-win.md`) (+2 more)

### Community 54 - "A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004"
Cohesion: 0.18
Nodes (11): A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004, MAC-010 · Bundle + IMKServer + controller rỗng (M), MAC-011 · handle → `ime_key` PASS (M), MAC-012 · Preedit qua marked + commit ngắn (L) — `P2-1 §6.1/§7`, MAC-013 · KeyTranslator (UCKeyTranslate) + đổi layout (M), MAC-014 · SelectionReplace `P2-1 §6.2` (L), MAC-015 · BackspaceType theo MAC-004 + RESTORE (L) — `P2-1 §6.3`, MAC-016 · Focus/commit-before-hide + reset (M) — `P2-1 §3/§4` (+3 more)

### Community 55 - "Result"
Cohesion: 0.13
Nodes (13): GUID, IEnumTfDisplayAttributeInfo, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, Option, Result, TextVNDisplayAttributeInfo_Impl, TextVNDisplayAttributeProvider_Impl (+5 more)

### Community 56 - "doctor.rs"
Cohesion: 0.14
Nodes (24): abi_sizes(), build_pkzip(), check_instance_mutex(), check_pipe_listening(), collect_log_tail(), collect_report(), config_path(), crc32() (+16 more)

### Community 57 - "StrategyResolver"
Cohesion: 0.10
Nodes (23): Key, StrategyResolver, .cacheCount, .hasAppdb, Bool, Data, Int, Int32 (+15 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "field-detect/src/lib.rs"
Cohesion: 0.14
Nodes (17): K, CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated(), ProbeCache, ProbeCache<K> (+9 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.02
Nodes (84): gdi, hidpi, normalize_word_list, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON (+76 more)

### Community 62 - "IpcMessage"
Cohesion: 0.11
Nodes (18): IpcMessage, ack, configReload, crashReport, getSnapshot, hello, .json, .jsonData (+10 more)

### Community 63 - "appdb/src/lib.rs"
Cohesion: 0.15
Nodes (21): AppDbError, DB, documented_metadata_and_match_clause_parse(), engine_version_is_supported(), EngineOwner, first_matching_entry_and_role_win(), input(), newer_engine_requirement_skips_only_affected_presets() (+13 more)

### Community 64 - "M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)"
Cohesion: 0.20
Nodes (10): M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0), WIN-050 · Tray skeleton + menu 9 mục (M) — `P1-4 §1`, WIN-051 · IPC server + state/watch + watchdog hook (L) — `P1-4 §2/§6`, WIN-052 · Settings egui 6 tab + parity checklist (L) — `P1-4 §3`, WIN-053 · Autostart + config init + hot-reload (M), WIN-054 · Installer 2 mode + uninstall sạch (L) — `P1-4 §4`, WIN-055 · uiAccess + ký số SignPath (M) — `P1-4 §5/§7`, `P1-2 §7`, WIN-056 · Updater + rollback (L) — `P1-4 §7` (+2 more)

### Community 65 - "A0 — Spike & môi trường (tuần 1–2)"
Cohesion: 0.20
Nodes (10): A0 — Spike & môi trường (tuần 1–2), MAC-001 · Môi trường build mac (S, dep: —), MAC-002 · SPIKE: IMK tối thiểu (L, dep: MAC-001) — **chặn mọi task WS1**, MAC-003 · SPIKE: Swift ↔ Rust staticlib (L, dep: MAC-001) — **RM1**, MAC-004 · SPIKE: cơ chế xóa/phím + selection (M, dep: MAC-002) — **RM2**, MAC-005 · SPIKE: AX + TCC (M, dep: MAC-002), MAC-006 · Corpus `corpus/mac/` đầu tiên (M, dep: —, song song), MAC-007 · SPIKE: đăng ký input source + GHA GUI/TCC (M, dep: MAC-002) (+2 more)

### Community 66 - "A3 — Menu bar / Settings / Packaging (tuần 11–14)"
Cohesion: 0.20
Nodes (10): A3 — Menu bar / Settings / Packaging (tuần 11–14), MAC-050 · TextVN.app skeleton + status menu 9 mục (M) — `P2-4 §1`, MAC-051 · IPC server unix socket + watcher + health (L) — `P2-4 §2/§6`, MAC-052 · Settings SwiftUI 6 tab + parity checklist (L) — `P2-4 §3`, MAC-053 · Login item + config init + hot-reload (M), MAC-054 · `.pkg` cài/gỡ sạch (L) — `P2-4 §4`, dep: MAC-007, MAC-055 · Developer ID + hardened runtime + notarization (M, dep: MAC-008) — **RM3**, MAC-056 · Updater Ed25519 + rollback (L) — `P2-4 §7` (+2 more)

### Community 67 - "T0 — Spike & môi trường (tuần 1–2)"
Cohesion: 0.20
Nodes (10): LNX-001 · Môi trường build Linux (S, dep: —), LNX-002 · SPIKE: IBus engine C tối thiểu (L, dep: LNX-001) — **chặn WS1**, LNX-003 · SPIKE: link `libtextvn_ffi.a` từ C và C++ (M, dep: LNX-001) — **RL10**, LNX-004 · SPIKE: keyval/shift + selection + surrounding (M, dep: LNX-002) — **RL1/RL5**, LNX-005 · SPIKE: AT-SPI + a11y permission (M, dep: LNX-002), LNX-006 · SPIKE: Fcitx5 addon API + version pin (L, dep: LNX-003) — **RL2**, LNX-007 · SPIKE: XGrabKeyboard + XTEST glyph + Wayland detect (L, dep: LNX-001) — **RL3/B10**, LNX-008 · SPIKE: GHA xvfb + AT-SPI trên runner (M, dep: LNX-002/005) (+2 more)

### Community 68 - "ime_result_v1"
Cohesion: 0.12
Nodes (27): fill_result(), flow_new_key_reset_free(), held_key_long_word_never_deletes_past_typed_text(), ime_instance_free(), ime_key(), ime_key_v1, ime_result_v1, key_char() (+19 more)

### Community 69 - "textvn-ffi"
Cohesion: 0.42
Nodes (13): textvn-appdb, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-ffi, textvn-field-detect, textvn-fuzz (+5 more)

### Community 70 - "items"
Cohesion: 0.18
Nodes (13): default, items, type, items, additionalProperties, pattern, required, type (+5 more)

### Community 71 - "compose.rs"
Cohesion: 0.06
Nodes (35): BACK, backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CompositionPlan, CONTROL, delete_beyond_composition_is_reported_not_guessed(), Doc (+27 more)

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "tray/src/lib.rs"
Cohesion: 0.06
Nodes (45): ipcserver, ordering, settings, show_settings_dialog, claim_debounced(), compatibility_hook_path(), CTRL_SHIFT_MARKER, CTRL_SHIFT_MARKER_DECLINED (+37 more)

### Community 74 - "M0 — Spike & nền (tuần 1–2)"
Cohesion: 0.22
Nodes (9): M0 — Spike & nền (tuần 1–2), WIN-001 · Chuẩn bị môi trường build (S, dep: —), WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**, WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002), WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001), WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001), WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song), WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —) (+1 more)

### Community 75 - "P2-0 — macOS MASTER PLAN (Phần 2)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần macOS (exit condition), 2. Kiến trúc process (nhất quán với `P1-0 §2`), 3. Workstream & file solution, 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2), 5. Dependency graph, 6. Rủi ro & mitigation (riêng macOS), 7. Definition of Done cho PHẦN 2, P2-0 — macOS MASTER PLAN (Phần 2)

### Community 76 - "IMKLibTests.swift"
Cohesion: 0.09
Nodes (12): KeyMapMacGenerated, UInt32, Bool, NSMenu, IpcCodecTests, KeyMapMacTests, MarkedStateTests, MarkedStateUnitTests (+4 more)

### Community 77 - "P3-0 — LINUX MASTER PLAN (Phần 3)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Linux (exit condition), 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`), 3. Workstream & file solution, 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3), 5. Dependency graph, 6. Rủi ro & mitigation (riêng Linux), 7. Definition of Done cho PHẦN 3, P3-0 — LINUX MASTER PLAN (Phần 3)

### Community 78 - "TextVNInputController"
Cohesion: 0.08
Nodes (19): Any, Bool, Data, FieldContext, Int, IpcClient, NSEvent, pid_t (+11 more)

### Community 79 - "undo.rs"
Cohesion: 0.24
Nodes (13): chars(), is_nucleus(), mark_cluster(), mark_horn(), mark_horn_pair_and_single(), mark_vowel(), mark_vowel_applies_and_undoes(), Marked (+5 more)

### Community 80 - "settings_window.c"
Cohesion: 0.06
Nodes (79): gpointer, GtkApplication, on_app_activate(), tv_paths, edit_bool(), edit_reset(), edit_str(), get_bool() (+71 more)

### Community 81 - "M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)"
Cohesion: 0.25
Nodes (8): M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC), WIN-060 · `appcomptest` harness (L) — `P1-5 §4`, WIN-061 · Targets JSON cho 12 app CI (M), WIN-062 · Perf bench + baseline (M) — `P1-5 §5`, WIN-063 · Soak 24h script (M), WIN-064 · CI jobs (M) — `P1-5 §8`, WIN-065 · Fuzz + ASan/TSan job Windows (M), WIN-066 · Security + license checklist Phần 1 (M) — Handbook §8

### Community 82 - "A4 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): A4 — Test & hardening (tuần 15–18), MAC-060 · AX harness `tools/mac/ax-driver` (L) — `P2-5 §4`, MAC-061 · Targets JSON 12 app (cùng format Windows) (M), MAC-062 · Perf bench + `perf/baseline-mac.json` (M) — `P2-5 §5`, MAC-063 · Soak 24h script (M), MAC-064 · CI jobs macos (M) — `P2-5 §8`, MAC-065 · Fuzz + ASan job mac (S), MAC-066 · Security + license checklist Phần 2 (M) — Handbook §8 + audit tap (P2-2 §7)

### Community 83 - "T5 — Tray / Settings / Packaging (tuần 11–14)"
Cohesion: 0.25
Nodes (8): LNX-050 · Tray SNI + menu 9 mục + single instance (M) — `P3-5 §1`, LNX-051 · IPC server + watcher + health (L) — `P3-5 §2/§4`, LNX-052 · Settings GTK4 6 tab + parity checklist (L) — `P3-5 §3`, LNX-053 · systemd user unit + config init + hot-reload (M), LNX-054 · `.deb` cài/gỡ sạch (L) — `P3-5 §5`, LNX-055 · `doctor` full (M) — `P3-5 §6`, LNX-056 · Update-notify + `.rpm` + AUR (M) — `P3-5 §7/§8`, T5 — Tray / Settings / Packaging (tuần 11–14)

### Community 84 - "T6 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): LNX-060 · AT-SPI driver `tools/linux/atspi-driver` (L) — `P3-6 §4`, LNX-061 · Targets JSON 12 app (cùng format 3 OS) (M), LNX-062 · Perf bench + `perf/baseline-linux.json` (M) — `P3-6 §5`, LNX-063 · Soak 24h script × 2 (X11 + Wayland VM) (M), LNX-064 · CI jobs linux + dist matrix (M) — `P3-6 §8`, LNX-065 · Fuzz + ASan/valgrind job (S), LNX-066 · Security + license checklist Phần 3 (M) — Handbook §8 + audit x11 (P3-3 §7), T6 — Test & hardening (tuần 15–18)

### Community 85 - "B. macOS"
Cohesion: 0.11
Nodes (18): KeyTranslator, Data, KeyEvent, NSEvent, UInt16, UInt32, B10. [THẤP] `IMKTextTarget.swift:40-42` — `NSUnderlineStyle([]).rawValue` = 0, B11. [THẤP] Dead code/state chết: `heldMods` (chỉ ghi), `noteFlagsChanged()` (+10 more)

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.25
Nodes (7): A. PowerShell & C# interop, B. Gửi phím & focus (dạng sai gây "phím bay vào app của user"), C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 88 - "textvn-common.sh"
Cohesion: 0.10
Nodes (47): put(), install.sh script, usage(), textvn-common.sh script, tv_check_package(), tv_err(), tv_fcitx5_env_addon_dirs(), tv_fcitx5_system_addon_dir() (+39 more)

### Community 89 - "appdb.v1.schema.json"
Cohesion: 0.25
Nodes (7): additionalProperties, $comment, $id, required, $schema, title, type

### Community 90 - "config.v1.schema.json"
Cohesion: 0.25
Nodes (7): additionalProperties, $comment, $id, required, $schema, title, type

### Community 91 - "Verified Ops — sổ thao tác / tool đã kiểm chứng"
Cohesion: 0.29
Nodes (6): A. Repo · Git · Graphify, B. Build · Test · Script, C. Windows platform (chi tiết → spike specs), D. Môi trường máy dev (snapshot 2026-09-27) — khỏi kiểm tra lại, E. Fixture / app test sẵn có (không dựng lại), Verified Ops — sổ thao tác / tool đã kiểm chứng

### Community 92 - "TextVN — Chỉ mục tài liệu & Giao thức Review"
Cohesion: 0.29
Nodes (7): 0. Tài liệu cho người dùng & phát hành, 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 93 - "M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006"
Cohesion: 0.29
Nodes (7): M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006, WIN-030 · crate `field-detect` + rules R1–R10 (M) — `P1-3 §2`, WIN-031 · Cache + invalidation + budget 2ms (M), WIN-032 · `engine_owner` + override chain (M) — `P1-3 §4`, `P1-2 §6`, WIN-033 · appdb loader + Ed25519 verify (M) — `P0-3 §2.2`, WIN-034 · Preset 20 app + corpus ≥ 40 case (M) — `P1-3 §3`, WIN-035 · Settings: "thêm preset từ app đang chạy" (S, dep: WIN-052)

### Community 94 - "M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005"
Cohesion: 0.29
Nodes (7): M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005, WIN-040 · `textvn-hook.exe` skeleton + IPC + heartbeat (M) — `P1-2 §2`, WIN-041 · Callback theo `P1-2 §3` (L), WIN-042 · Inject engine 3 modes + modifier restore (L) — `P1-2 §5`, WIN-043 · Focus + UIA worker (M) — `P1-2 §4`, WIN-044 · Rule `engine_owner` phía hook (S, dep: WIN-032), WIN-045 · Game/legacy presets + blocklist (M) — `P1-2 §8`

### Community 95 - "A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006"
Cohesion: 0.29
Nodes (7): A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006, MAC-030 · FieldDetect rules R1–R10 + mock tests (M) — `P2-3 §2`, MAC-031 · Cache + AXObserver + budget (M), MAC-032 · Preset 20 app + corpus ≥ 40 case (M) — `P2-3 §3`, MAC-033 · Appdb loader + Ed25519 (S) — dùng `textvn-appdb`, không viết lại, MAC-034 · Override chain `P2-3 §4` (path mac) (M), MAC-035 · Settings "thêm app đang chạy" (S, dep: MAC-052)

### Community 96 - "engine"
Cohesion: 0.28
Nodes (12): boundary_space_commits_with_preedit_cap(), chord_drops_word_ownership(), control_chars_from_layout_are_navigation_not_boundary(), disabled_all_pass(), english_passes_through_untouched(), escape_restores_raw(), golden_duocj_with_preedit(), keyup_does_not_change_state() (+4 more)

### Community 97 - "T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006"
Cohesion: 0.29
Nodes (7): LNX-020 · Addon skeleton + lifecycle (M) — `P3-2 §3`, LNX-021 · keyEvent → PASS (M) — `P3-2 §4`, LNX-022 · Preedit + commit + word boundary (L) — `P3-2 §5`, LNX-023 · SelectionReplace + BackspaceType/surrounding (M) — `P3-2 §5`, LNX-024 · focus/commit-before-hide (M) — `P3-2 §3`, LNX-025 · Framework switch + chống đôi (M) — `P3-2 §7`, T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006

### Community 98 - "T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005"
Cohesion: 0.29
Nodes (7): LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`, LNX-031 · Cache + AT-SPI event invalidate + budget (M), LNX-032 · Preset 20 app + corpus ≥ 40 case (M) — `P3-4 §3`, LNX-033 · Appdb loader + `ime_appdb_verify` (S) — P0-2 §1, không viết lại, LNX-034 · Override chain `P3-4 §6` (path Linux) (M), LNX-035 · `doctor` env matrix + framework auto (M) — `P3-4 §7`, `P3-5 §6`, T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005

### Community 99 - "P3-REVIEW-LOG — Phần 3 (Linux)"
Cohesion: 0.22
Nodes (8): Code Review Round 1 — Đúng & Đủ (Mã nguồn C/C++), Code Review Round 2 — Nhất quán & Sẵn sàng (Mã nguồn & Kiểm thử), Kiểm chứng sau fix (Review 2 cuối), Kết quả tổng, P3-REVIEW-LOG — Phần 3 (Linux), Review 1 — Đúng & Đủ, Review 2 — Nhất quán & Sẵn sàng, Rủi ro còn mở của Phần 3 (theo dõi, không phải finding)

### Community 100 - "Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`"
Cohesion: 0.29
Nodes (6): 1. Bảng scope — API nào cần elevation?, 2. Fallback đã chứng minh cho `--scope user`, 3. Kết quả verify Win+Space / registry, 4. Finding (bắt buộc áp cho impl), 5. Exit, Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`

### Community 101 - "class.rs"
Cohesion: 0.16
Nodes (13): ClassFactory, HR_CLASS_E_NOAGGREGATION, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, ObjGuard, AtomicI32, Default (+5 more)

### Community 102 - "IpcServer"
Cohesion: 0.18
Nodes (9): ClientSink, IpcServer, Arc, AtomicU32, Mutex, Self, Vec, write_frame_to_subscribers() (+1 more)

### Community 103 - "env.md — Môi trường build Windows (WIN-001)"
Cohesion: 0.33
Nodes (5): 1. Build host (máy dev), 2. Tools DoD (Handbook §4 mục 4), 3. Bằng chứng acceptance (WIN-001), 4. Gap & việc cần khi lên milestone liên quan, env.md — Môi trường build Windows (WIN-001)

### Community 104 - "A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009"
Cohesion: 0.33
Nodes (6): A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009, MAC-040 · Tap thread skeleton + owner check (M) — `P2-2 §2/§3`, MAC-041 · Callback + self-disable (M) — `P2-2 §3`, MAC-042 · Inject §5 (3 modes + marker + restore) (L), MAC-043 · Permission UX + revocation poll (M) — `P2-2 §4`, MAC-044 · Rule owner `imk|tap` + IPC sync (S, dep: MAC-034)

### Community 105 - "T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007"
Cohesion: 0.33
Nodes (6): LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`, LNX-041 · Callback + self-disable (M) — `P3-3 §3`, LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`, LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`, LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S), T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### Community 106 - "ByteCursor<'a>"
Cohesion: 0.33
Nodes (3): ByteCursor<'a>, Option, Self

### Community 107 - "Bảng kiểm tra đồng bộ giao diện cài đặt (Settings Parity Checklist)"
Cohesion: 0.50
Nodes (4): 2. Gõ tắt, 3. Hệ thống & nút, 4. Chưa có trên UI (chỉ sửa được trong file), Bảng kiểm tra đồng bộ giao diện cài đặt (Settings Parity Checklist)

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "hotkey.rs"
Cohesion: 0.14
Nodes (22): registry, CTRL_SHIFT, current(), free_ctrl_shift(), freed_configuration_leaves_other_hotkeys_alone(), hk(), language_hotkey_on_ctrl_shift_is_freed_too(), NOT_ASSIGNED (+14 more)

### Community 111 - "entry"
Cohesion: 0.50
Nodes (4): entry, additionalProperties, required, type

### Community 112 - "id"
Cohesion: 0.50
Nodes (4): maxLength, minLength, type, id

### Community 113 - "auto_capitalize"
Cohesion: 0.50
Nodes (4): $comment, default, type, auto_capitalize

### Community 114 - "auto_restore_english"
Cohesion: 0.50
Nodes (4): $comment, default, type, auto_restore_english

### Community 115 - "MarkedState"
Cohesion: 0.13
Nodes (17): ApplyReplace, Int, Void, MarkedState, .graphemeCount, .isEmpty, .scalarCount, .utf16Count (+9 more)

### Community 116 - "english_words"
Cohesion: 0.50
Nodes (4): $comment, default, type, english_words

### Community 119 - "KeyOutcome"
Cohesion: 0.09
Nodes (23): Action, commit, pass, replace, restore, FFI, ImeEngine, ImeError (+15 more)

### Community 120 - "config_version"
Cohesion: 0.67
Nodes (3): $comment, const, config_version

### Community 121 - "free_marking"
Cohesion: 0.67
Nodes (3): default, type, free_marking

### Community 122 - "core"
Cohesion: 0.21
Nodes (11): Cell, Default, ObjGuard, Self, Vec, TextVNDisplayAttributeInfo, TextVNDisplayAttributeProvider, TextVNEnumDisplayAttributeInfo (+3 more)

### Community 127 - "EngineSession"
Cohesion: 0.17
Nodes (9): config_file_path(), EngineSession, macro_when_vi_off_runs_engine_in_passthrough(), Drop, ime_instance, PathBuf, Result, runtime_config_reload_updates_engine_method() (+1 more)

### Community 129 - "WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN""
Cohesion: 0.33
Nodes (5): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

### Community 130 - "Result"
Cohesion: 0.27
Nodes (15): index_of(), key_char(), Map, Method, one_char(), parse_maps(), parse_method(), parse_vowels() (+7 more)

### Community 131 - "Strategy"
Cohesion: 0.19
Nodes (10): crate, downgrade(), resolve(), ResolveInput, Option, Strategy, default_for_field(), FIELD_ROLES (+2 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.19
Nodes (16): base_keys(), f(), finish(), fix_uo(), fold(), fold_caps(), generated_key_tables_are_self_consistent(), is_word_char() (+8 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (16): 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5, 7. Bang ghi chu (+8 more)

### Community 134 - "encode_frame"
Cohesion: 0.13
Nodes (23): read_next_message(), Result, W, send_message(), read_next_message(), Result, W, send_message() (+15 more)

### Community 135 - ".character"
Cohesion: 0.25
Nodes (7): CGFlagMapper, Bool, UInt16, UInt32, TapTranslator, CGEventFlags, Round 8.1 — Audit vòng 1 (3 mảng song song)

### Community 138 - "store_win.rs"
Cohesion: 0.15
Nodes (26): appx, commandext, OsStr, shellexecutew, after_tray_start(), cmd_exe_path(), CREATE_BREAKAWAY_FROM_JOB, CREATE_NO_WINDOW (+18 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "common_words.rs"
Cohesion: 0.22
Nodes (22): assert_none(), check_all(), keys_for(), Fn, Method, Option, Vec, telex_capitalized_with_shift() (+14 more)

### Community 142 - "guids.rs"
Cohesion: 0.39
Nodes (7): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI, PROFILE_TEXTVN, GUID

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "TsfShared"
Cohesion: 0.09
Nodes (38): KeyKind, apply_display_attribute(), apply_plan(), caret_at_composition_end(), CompSink, CompSink_Impl, EndCompositionSession, finish() (+30 more)

### Community 145 - "do_register"
Cohesion: 0.16
Nodes (26): activate_for_session(), activate_tip(), active_profile_is_textvn(), clsid_key(), com_init(), ctf_tip_key(), delete_tree(), do_register() (+18 more)

### Community 149 - "autostart.rs"
Cohesion: 0.11
Nodes (40): REG_SAM_FLAGS, APP_RUN_VALUE_NAME, autostart_command(), autostart_disabled_by_user(), AUTOSTART_DISABLED_MARKER, autostart_disabled_marker_path(), autostart_effective(), autostart_suppressed_for() (+32 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Changelog"
Cohesion: 0.09
Nodes (23): [0.2.11] — 2026-10-03, [0.2.17] — 2026-10-03, [0.2.18] — 2026-10-03, [0.2.19] — 2026-10-04, [0.2.1] — 2026-09-30, [0.2.21] — 2026-10-04, [0.2.23] — 2026-10-05, [0.2.24] — 2026-10-05 (+15 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.05
Nodes (68): PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-09, sau v0.2.27 + vòng rà soát 2), getmodulehandlew, HICON, id_exit, shell, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V (+60 more)

### Community 154 - "menu.rs"
Cohesion: 0.10
Nodes (25): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_HEALTH_STATUS, ID_HOOK_COMPAT_MODE (+17 more)

### Community 155 - "SettingsController"
Cohesion: 0.09
Nodes (19): textvn_config, Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer (+11 more)

### Community 156 - "compose.c"
Cohesion: 0.11
Nodes (44): boundary_char(), ime_instance, lc_comp, lc_config_state, lc_key, lc_plan, file_stamp(), is_executable() (+36 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.15
Nodes (38): lc_ipc_client, client_disconnect(), get_time_ms(), handle_ipc_message(), ipc_debug_enabled(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override() (+30 more)

### Community 158 - "tsf-min/src/lib.rs"
Cohesion: 0.11
Nodes (26): ITfKeyEventSink, ClassFactory, CLSID_TEXTVN_TIP, CompSink, DllCanUnloadNow(), DllGetClassObject(), EditSession, HR_CLASSNOTAVAILABLE (+18 more)

### Community 159 - "P0 — REVIEW LOG (Phần 0: Nền tảng chung)"
Cohesion: 0.33
Nodes (5): Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

### Community 160 - "SettingsDoc"
Cohesion: 0.11
Nodes (24): atomic_write(), create_private_dir(), defaults(), DocKind, invalid_values_are_rejected_without_changing_the_doc(), macros_text_round_trip_through_doc(), MAX_FILE_BYTES, missing_keys_read_schema_defaults() (+16 more)

### Community 162 - "keys_generated.rs"
Cohesion: 0.19
Nodes (12): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+4 more)

### Community 164 - "tsf-min/src/register.rs"
Cohesion: 0.19
Nodes (21): ITfInputProcessorProfiles, activate(), clsid_key(), com_init(), dll_path(), guid_str(), ILOT_DEFPROFILE, install() (+13 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "trace.rs"
Cohesion: 0.18
Nodes (10): enabled(), event(), File, Mutex, Option, sink(), Arguments, sync (+2 more)

### Community 167 - "engine.cpp"
Cohesion: 0.05
Nodes (77): InputContext, Instance, KeyEvent, KeySym, lc_modifier, string, fcitxModifierKind(), fcitxMods() (+69 more)

### Community 168 - "IpcClient"
Cohesion: 0.12
Nodes (16): Frame, frame, needMore, violation, IpcClient, IpcClientDelegate, Data, DispatchSourceRead (+8 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.08
Nodes (30): CLIENT, GLOBAL_KEY, global_switch_from_tray_disables_every_app(), IpcClient, IpcState, LAST_HOTKEY_TOGGLE_MS, local_toggle_flips_immediately_without_tray(), map() (+22 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "HWND"
Cohesion: 0.25
Nodes (24): apply_dpi_change(), close_macro_editor(), close_word_list_editor(), dialog_wnd_proc(), get_chk(), layout_dialog_controls(), macro_wnd_proc(), populate_controls_from_config() (+16 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "show_macro_editor"
Cohesion: 0.18
Nodes (24): HGDIOBJ, center_on_work_area(), create_and_show_window(), create_control(), create_control_ex(), create_dialog_controls(), delete_ui_fonts(), layout_macro_controls() (+16 more)

### Community 176 - "generate_tray_icons.py"
Cohesion: 0.18
Nodes (19): _centered_text(), create_us_icon(), create_vn_icon(), _draw_star(), generate_ico(), _letter_font(), main(), FreeTypeFont (+11 more)

### Community 177 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.25
Nodes (7): 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng), Targets JSON — `tools/appcomptest/targets/` (WIN-061)

### Community 178 - "windows-tsf/src/replay.rs"
Cohesion: 0.14
Nodes (23): CLASS_NAME, DeferredReplay, deliver(), HOPS, module_instance(), Pending, prepare(), HINSTANCE (+15 more)

### Community 179 - "check-pe-imports.py"
Cohesion: 0.40
Nodes (4): struct, imports(), main(), check-pe-imports.py — chặn binary Windows phụ thuộc VC++ runtime (R2-18). TIP…

### Community 180 - "invariants.rs"
Cohesion: 0.18
Nodes (17): apply(), backspace_removes_exactly_one_visible_char(), engine_never_overflows_result_or_eats_existing_text(), held_key_after_toned_word_keeps_previous_text(), key_event(), Lcg, letters(), PREFIX (+9 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.32
Nodes (3): Sign-TextVNFile(), Write-Fail(), Write-Ok()

### Community 182 - "AppDelegate"
Cohesion: 0.07
Nodes (24): AppDelegate, .frontAppEnabled, .isVietnameseMode, .vietnameseModeEnabled, Date, IpcServer, NSMenu, NSObjectProtocol (+16 more)

### Community 183 - "check_no_injection_apis.py"
Cohesion: 0.17
Nodes (10): Check relative markdown links trong docs/ + *.md goc. Chay local: python…, main(), tracked_iss(), Check API inject / giong malware bi cam (Farch-3). Chay local: python…, main(), tracked_ps1(), pathlib, re (+2 more)

### Community 184 - "EventTapController"
Cohesion: 0.22
Nodes (8): EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, NSObjectProtocol, CFMachPort, CFRunLoop, CFRunLoopSource

### Community 185 - "Engine"
Cohesion: 0.22
Nodes (12): Action, apply(), backspaces(), boundary_space_pass_with_default_caps(), Engine, Outcome, KeyEvent, Option (+4 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "DialogContext"
Cohesion: 0.12
Nodes (20): advice_for_failure(), DIALOG_CTX, DialogContext, normalize_word_list(), read_log_tail(), register_and_activate_tsf(), register_log_path(), register_log_tail_reads_last_lines() (+12 more)

### Community 189 - "Foundation"
Cohesion: 0.08
Nodes (25): ApplyError, cannotDelete, rejected, Bool, Int, TextCommandTarget, KeyTranslatorError, noChar (+17 more)

### Community 190 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 191 - "IpcServer"
Cohesion: 0.14
Nodes (13): IpcServer, IpcServerDelegate, Any, Bool, Data, Date, DispatchSourceRead, Int (+5 more)

### Community 192 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - "`docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)"
Cohesion: 0.25
Nodes (7): 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate, `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

### Community 195 - "OwnerRule"
Cohesion: 0.24
Nodes (6): EngineOwner, imk, tap, OwnerRule, Bool, OwnerRuleTests

### Community 196 - "super"
Cohesion: 0.05
Nodes (59): abi_sizes(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister(), cmd_verify() (+51 more)

### Community 197 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 198 - "rules_win.rs"
Cohesion: 0.17
Nodes (12): address_or_search(), classify(), contains(), ControlType, known(), lower(), ProbeResult, role() (+4 more)

### Community 199 - "Security Policy"
Cohesion: 0.50
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (18): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+10 more)

### Community 201 - "SvcManager"
Cohesion: 0.12
Nodes (17): DocError, apply_option_checkbox(), effective_enabled(), persist_keeps_unknown_keys_and_reset_keeps_macros(), AtomicBool, AtomicU64, DiacriticStyle, FnOnce (+9 more)

### Community 202 - "MockHandler"
Cohesion: 0.27
Nodes (5): MockHandler, SelfDisableTests, Bool, Double, TextVNTap

### Community 203 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.17
Nodes (11): 0. Executive Summary, 7.1 Team (tối thiểu → lý tưởng), 7.2 Risk register, 7.3 WBS cấp 2 (sprint-ready), 7. TEAM, RISK & WORK BREAKDOWN, 8. ACCEPTANCE CRITERIA — "hoàn hảo" nghĩa là gì (v1.0), 9.1 Nguồn tham chiếu, 9.2 ADR cần viết trong tuần 1 (+3 more)

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "ConfigStore"
Cohesion: 0.09
Nodes (26): ConfigStore, .config, Bool, Void, ActiveAlert, about, .alert, autostartFailed (+18 more)

### Community 206 - "ipc_server.rs"
Cohesion: 0.20
Nodes (15): authorization, filesystem, pipes, read, broadcast_reaches_subscriber(), crash_counter_increments_atomically(), frame_encoding_and_read_message_roundtrip(), PIPE_NAME (+7 more)

### Community 207 - "AppDb"
Cohesion: 0.24
Nodes (13): AppDb, b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace(), caps_missing_downgrades(), db(), disabled_app_passthrough(), finder_rename_editbox() (+5 more)

### Community 208 - "ViState"
Cohesion: 0.14
Nodes (12): Bool, ViState, ViStateStore, Bất biến mỗi target assert (không chỉ "không panic"), Chạy, fuzz — L4 (P0-4 §1), Kiểm chứng trên máy không có cargo-fuzz, Seed corpus (+4 more)

### Community 209 - "String"
Cohesion: 0.10
Nodes (16): Bool, Int, Int32, URL, UninstallOutcome, cancelled, failed, succeeded (+8 more)

### Community 210 - "e2e_ibus.c"
Cohesion: 0.22
Nodes (20): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+12 more)

### Community 211 - "generate_app_icons.py"
Cohesion: 0.39
Nodes (7): gradient(), main(), pick_font(), Image, Sinh icon PNG ứng dụng TextVN từ thiết kế badge trong `resources/icons/*.svg`.…, Gradient chéo 45° khớp linearGradient 0%,0%→100%,100% (objectBoundingBox)., render()

### Community 212 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 213 - "EndCompositionSession_Impl"
Cohesion: 0.50
Nodes (3): EndCompositionSession_Impl, KeyEditSession_Impl, ITfEditSession_Impl

### Community 214 - "e2e_fcitx5.py"
Cohesion: 0.15
Nodes (14): app_handles(), clear(), ctrl_shift_tap(), press_char(), pump(), Ghi như bảng cài đặt (tmp → rename); chờ để mtime chắc chắn khác lần trước., Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend…, send() (+6 more)

### Community 215 - "AXSnapshot"
Cohesion: 0.27
Nodes (3): AXSnapshot, FieldRules, FieldRulesTests

### Community 216 - "key_event.rs"
Cohesion: 0.18
Nodes (27): is_modifier_vk(), active_modifiers(), defer_to_key_down(), eaten_label(), end_composition(), guarded(), guarded_option(), handle_key() (+19 more)

### Community 217 - "stdio"
Cohesion: 0.06
Nodes (43): lc_detect_active_framework(), lc_env_check(), get_time_ms(), lc_classify_field(), lc_field_detect(), str_contains_icase(), ensure_parent_dirs(), lc_log_close() (+35 more)

### Community 218 - "TextVNConfig"
Cohesion: 0.08
Nodes (28): EmojiEntry, LoadError, corrupt, missing, MacroEntry, .id, MacroRules, Problem (+20 more)

### Community 225 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`), 8. CI jobs (+1 more)

### Community 226 - "show_dialog_on_startup"
Cohesion: 0.67
Nodes (3): show_dialog_on_startup, default, type

### Community 227 - "ffi/src/settings.rs"
Cohesion: 0.09
Nodes (44): enc(), encode(), Entry, OutputCharset, parse_pair(), push_pair(), Option, Vec (+36 more)

### Community 229 - "foreground.rs"
Cohesion: 0.14
Nodes (17): accessibility, foregroundtracker, HWINEVENTHOOK, threading, app_id_from_image_path(), app_of_window(), ForegroundTracker, is_shell_window_class() (+9 more)

### Community 230 - "resolve.rs"
Cohesion: 0.33
Nodes (10): default_for_field, downgrade_without_cap(), inp(), secure_field_role_passthrough(), secure_role_beats_hint_and_presets_without_secure_flag(), step1_secure_beats_everything(), step2_disabled_passthrough(), step3_user_preset_beats_system() (+2 more)

### Community 231 - "AutostartManager"
Cohesion: 0.11
Nodes (16): AutostartManager, Bool, URL, Fixed (macOS — ưu tiên bản này), (B) Windows / xtask / Linux — `R3-*`, Kiểm chứng vòng 10 (host Windows, trước commit), Round 9 — Fix review vòng 3 trước tag v0.2.0 (2026-09-30), Vòng 10 — đóng nốt các mục hoãn (phát hành `v0.2.2`) (+8 more)

### Community 232 - "FieldContext"
Cohesion: 0.26
Nodes (7): Entry, FieldContext, FieldDetect, Date, pid_t, TimeInterval, 7. Task

### Community 233 - "Path"
Cohesion: 0.15
Nodes (19): append_log_line(), grant_appcontainer_read(), is_reparse_link(), log_rotates_when_larger_than_limit(), log_writer_checks_reparse_before_creating_dirs(), machine_scope_dll(), machine_scope_dll_allowed(), normalize_win_path() (+11 more)

### Community 234 - "TapPermission"
Cohesion: 0.20
Nodes (8): NSLock, Bool, T, TimeInterval, Void, TapPermission, ApplicationServices, Timer

### Community 235 - "diacritic_style.rs"
Cohesion: 0.22
Nodes (10): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), vowel_span(), is_vowel (+2 more)

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.25
Nodes (8): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor, Sự cố đã gặp khi phát hành — tra trước khi xử lý

### Community 237 - "TapKeyHandler"
Cohesion: 0.33
Nodes (3): TapKeyHandler, TapMarkerTests, Quyết định kỹ thuật đã chốt trong code (ghi lại cho review)

### Community 238 - "Hướng dẫn sử dụng TextVN"
Cohesion: 0.15
Nodes (13): 1. Cài đặt, 2. Bật/tắt tiếng Việt, 3. Kiểu gõ, 4. Bảng điều khiển, 5. Gõ tắt, 6. Cấu hình, 7. Xử lý sự cố, Cài đặt im lặng (cho quản trị / Microsoft Store) (+5 more)

### Community 239 - "keymap_mac.rs"
Cohesion: 0.20
Nodes (4): MAC_KEY_COUNT, mac_to_canonical(), Option, modifier_dung_vk_is_modifier_nhan_duoc()

### Community 240 - "quick_telex.rs"
Cohesion: 0.24
Nodes (6): apply, apply(), expansion(), q(), Option, Vec

### Community 241 - "HookEngine"
Cohesion: 0.39
Nodes (5): HookEngine, Drop, ime_instance, Result, Self

### Community 242 - "Hướng dẫn phát triển TextVN"
Cohesion: 0.20
Nodes (10): 1. Kiến trúc, 2. Dựng, 3. Kiểm thử, 4. Quy ước bắt buộc (CI chặn), 5. Phát hành, 6. Tài liệu liên quan, Hướng dẫn phát triển TextVN, Linux (Ubuntu/Debian) (+2 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.10
Nodes (21): 1. Gói phát hành, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Báo cáo dựng & kiểm thử — TextVN, Bản 0.2.1 — phát hành pre-release đã xác minh, Bản 0.2.20 — CI xanh lại + validation vòng 3 (harness đúng như Microsoft) (+13 more)

### Community 244 - "remove_added_languages"
Cohesion: 0.21
Nodes (17): added_language_removable(), is_input_method_value(), languages_added_marker_path(), languages_missing(), languages_without(), parse_languages_marker(), read_languages_added(), record_languages_added() (+9 more)

### Community 245 - "A. WINDOWS"
Cohesion: 0.12
Nodes (17): A10. [TRUNG] COM callback `ActivateEx`/`Deactivate`/`OnSetFocus` không bọc `catch_unwind` (tip.rs:90-124,174-183) — panic thoát qua ABI = abort host (Notepad/Chrome). Nhánh `try_borrow_mut` fail tại ActivateEx: Advise đã thực hiện nhưng cookie không lưu → Deactivate không Unadvise (leak tới hết thread). **Fix:** catch_unwind như `DoEditSession`; borrow-fail trả `Err(E_FAIL)`., A11. [TRUNG] `svc.rs:135-235` — `RwLock.write().unwrap()`: poison → panic dây chuyền trong `wnd_proc`. **Fix:** `unwrap_or_else(|e| e.into_inner())`., A12. [TRUNG] Uninstaller per-user non-elevated: `schedule-delete` (PFO) luôn ACCESS_DENIED → `.old-*` sót vĩnh viễn (register.rs:793-834). **Fix:** RunOnce như uninstall.ps1 portable đã làm., A13. [THẤP] Đa user với bản máy: user B còn per-user override trỏ DLL đã xoá; CLI cũng bị xoá → không tự lành (main.rs:601-603 return sớm). Fix dài hạn: marker machine-level yêu cầu tray user B tự unregister khi thấy path chết., A14. [THẤP] `reg_value_names` buffer 512 u16 — value dài hơn 511 ký tự, A15. [THẤP] `show_activation_refused_notice_once` (main.rs:337-351) ghi marker, A1. [CAO] ILOT spec cho bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100) 🔎, A2. [CAO] Mirror WOW64 CTF cho **HKCU** là đường registry chết 🔎 (+9 more)

### Community 246 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 247 - ".inject"
Cohesion: 0.50
Nodes (5): Bool, CGEventSource, Int, UInt16, TapInjector

### Community 248 - "TextVNAppTests"
Cohesion: 0.05
Nodes (22): AppInfo, Data, TextVNAppTests, Cocoa, Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging), Code Review Round 2 — Nhất quán & Sẵn sàng (Kiểm thử, Autostart S5 & Residue S9), Kiểm chứng sau fix (Review 2 cuối), Kết quả kiểm tra sau fix (+14 more)

### Community 249 - "Option"
Cohesion: 0.24
Nodes (11): Entry, FieldRoles, id_matches(), MatchClause, Matcher, Preset, Option, Vec (+3 more)

### Community 250 - "smoke-imk.sh"
Cohesion: 0.48
Nodes (5): clear_doc(), read_doc(), say(), smoke-imk.sh script, type_keys()

### Community 251 - "textvn_ffi"
Cohesion: 0.05
Nodes (36): Instance, TextVNEngineFactory, create, ime_instance, ime_suggest, abi_version, count, items (+28 more)

### Community 252 - "Kế hoạch ký số (code signing) — TextVN"
Cohesion: 0.20
Nodes (10): 1. So sánh các đường chính thống (cập nhật 2026-10-02), 2. Lộ trình đề xuất — SignPath Foundation, 3. Smart App Control / SmartScreen — kỳ vọng thực tế, 4. Việc đã wire trong repo, Bước 1 — Nộp đơn (làm một lần, maintainer tự làm), Bước 2 — Tạo project + artifact configuration trên SignPath.io, Bước 3 — Bật ký trong GitHub Actions (đã nối sẵn — chỉ cần đặt secret), Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai) (+2 more)

### Community 253 - "generate_store_art.py"
Cohesion: 0.17
Nodes (15): A16. [THẤP] `compose`/`replay` — (đã soát, chỉ ghi nhận) replay post đúng, math, pil, compose(), diag_gradient(), draw_star(), load_font(), main() (+7 more)

### Community 254 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.13
Nodes (15): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release (+7 more)

### Community 255 - "test-typing.ps1"
Cohesion: 0.36
Nodes (4): Check(), Clear-Verified(), Codes(), Norm()

### Community 256 - "env-mac — Môi trường build macOS (MAC-001)"
Cohesion: 0.40
Nodes (4): Acceptance MAC-001, env-mac — Môi trường build macOS (MAC-001), Lệnh kiểm nhanh, Yêu cầu

### Community 257 - "ffi_key.rs"
Cohesion: 0.13
Nodes (12): cstr, STRATEGY_MAX_ID, assert_result_sane(), ByteCursor, MAX_KEYS, VK_BACK, VK_RETURN, VK_SPACE (+4 more)

### Community 258 - "Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)"
Cohesion: 0.25
Nodes (8): 1. Bố cục, 2. Tuỳ chọn ↔ khoá cấu hình, 3. Hành vi chung, 4. Cửa sổ Gõ tắt, 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+), 6. Nhận diện EN/VI (không có UI riêng — hành vi engine), 7. Icon — một thiết kế cho 3 nền tảng, Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)

### Community 259 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 260 - "tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)"
Cohesion: 0.40
Nodes (4): 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật (cập nhật 2026-10-09), tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

### Community 262 - "TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux"
Cohesion: 0.29
Nodes (7): Cấu trúc, Dựng từ mã nguồn, Giấy phép, TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux, Trạng thái, Tải và cài, Đóng góp

### Community 263 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.13
Nodes (15): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release, Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF (+7 more)

### Community 264 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - ".start"
Cohesion: 0.29
Nodes (6): CGEventTapLocation, [0.2.2] — 2026-09-30, Changed, Fixed (macOS — tiếp), Fixed (Windows), Housekeeping

### Community 269 - "method"
Cohesion: 0.67
Nodes (3): default, enum, method

### Community 270 - "output_charset"
Cohesion: 0.67
Nodes (3): default, enum, output_charset

### Community 274 - "EngineOptions"
Cohesion: 0.17
Nodes (8): Context, EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset, Self

### Community 291 - "ConfigWatcher"
Cohesion: 0.18
Nodes (8): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem, Notification

### Community 292 - "path"
Cohesion: 0.18
Nodes (11): find_rc_exe(), main(), Option, PathBuf, all_cases, fs, path, Result (+3 more)

### Community 293 - ".onEvent"
Cohesion: 0.25
Nodes (6): TapCallback, CGEvent, CGEventTapProxy, CGEventType, Unmanaged, UnsafeMutableRawPointer

### Community 294 - "command"
Cohesion: 0.21
Nodes (10): find_rc_exe(), main(), Option, PathBuf, command, python_runner(), Result, run() (+2 more)

### Community 295 - "ModifierToggle"
Cohesion: 0.43
Nodes (4): ctrl_shift_tap_toggles_once_and_only_without_other_keys(), is_ctrl_vk(), is_shift_vk(), ModifierToggle

### Community 296 - "[0.2.10] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.10] — 2026-10-03, Documented, Fixed, Tests

### Community 297 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 298 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Nguồn `FieldContext` trên Linux, 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux), 7. Env matrix & detect framework (dùng cho `textvn doctor` — `PLAN §5.3`), 8. Test (+1 more)

### Community 299 - "[0.2.0] — 2026-09-30"
Cohesion: 0.25
Nodes (8): [0.2.0] — 2026-09-30, Added, Added (macOS — Farch-4), Added (trước đó), Changed, Fixed, Fixed (review vòng 3 — trước tag v0.2.0), Known limitations

### Community 301 - ".new"
Cohesion: 0.52
Nodes (5): KeySink, KeyTraceSink, ObjGuard, Rc, Self

### Community 303 - "[0.2.13] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.13] — 2026-10-03, Added, Documented, Fixed

### Community 304 - "3. Nhật ký finding"
Cohesion: 0.13
Nodes (14): 0. Kết luận, 1. Phương pháp, 2. Bằng chứng CI, 3.1 Microsoft Store / MSIX, 3.2 Bảo mật Windows, 3.3 QA Windows (tray, CLI, bộ cài), 3.5 Tiếng Việt / engine, 3.6 CI, ký số, phát hành (+6 more)

### Community 305 - "[0.2.14] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.14] — 2026-10-03, Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau"), Fixed (Windows), Tests

### Community 306 - "[0.2.9] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.9] — 2026-10-02, Added, Documented, Tests

### Community 307 - "Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất"
Cohesion: 0.50
Nodes (4): 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển, 0.2.14 — DPI gốc + UI thống nhất + common errors, Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất, Phát hành + xác minh máy chủ repo

### Community 308 - "Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)"
Cohesion: 0.29
Nodes (7): Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon), Icon — trả lời câu hỏi "Linux/macOS đã có icon như Windows chưa?", Kiểm chứng chạy trên host Windows (2026-10-01, sau toàn bộ thay đổi), Phát hành — v0.2.3 đã publish (release candidate, chưa ký số), Quy trình phát hành mới, Sửa theo audit vòng 1 — GitHub Actions (5 major + minors), Sửa theo audit vòng 1 — đăng ký TSF (3 major + 7 minor)

### Community 309 - "2. PRODUCT OWNER VIEW"
Cohesion: 0.29
Nodes (7): 2.1 Tầm nhìn & USP, 2.2 Personas & Jobs-to-be-done, 2.3 Backlog ưu tiên (MoSCoW), 2.4 "EVKey Spec" — danh sách hành vi phải implement clean-room, 2.5 KPI & Definition of Success, 2.6 Release train, 2. PRODUCT OWNER VIEW

### Community 312 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.14
Nodes (12): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG, 6. PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT) (+4 more)

### Community 313 - "Added"
Cohesion: 0.33
Nodes (6): [0.1.0] — 2026-09-27, Added, Architecture, Core, Developer Tools, Windows Platform

### Community 315 - "Option"
Cohesion: 0.22
Nodes (14): broadcast_international(), ctrl_shift_taken_by_windows(), delete_reg_value(), ensure_modern_language_list(), enum_subkeys(), reg_read_named(), RegValue, remove_ctf_assembly_defaults() (+6 more)

### Community 316 - "[0.2.3] — 2026-10-01"
Cohesion: 0.33
Nodes (6): [0.2.3] — 2026-10-01, Added, Changed (CI — audit GitHub Actions, 5 major), Fixed (Windows), Fixed (đăng ký TSF — audit vòng sâu, 3 major), Housekeeping

### Community 317 - "Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit"
Cohesion: 0.33
Nodes (6): Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit, Bổ sung sau commit `ec1ac2b` — triệt để hóa gợi ý lỗi đăng ký TSF (ảnh lỗi của người dùng), Kiểm chứng chạy trên host Windows (trước commit), Perf — job `perf regression` đỏ trên CI là **nhiễu runner**, không phải hồi quy, Phát hành — v0.2.2 đã publish (release candidate, chưa ký số), Trạng thái

### Community 318 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.15
Nodes (13): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG (+5 more)

### Community 319 - "4. ENGINEER VIEW"
Cohesion: 0.40
Nodes (5): 4.1 Ngôn ngữ & технологies (quyết định), 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh), 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu), 4.4 Quy trình, 4. ENGINEER VIEW

### Community 320 - "2. Kết quả kiểm thử"
Cohesion: 0.50
Nodes (4): 2. Kết quả kiểm thử, Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng), Kịch bản 1 — Cài đặt, Kịch bản 2 — Giải nén ra dùng luôn

### Community 321 - "Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening"
Cohesion: 0.50
Nodes (4): Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening, Nguyên nhân + fix + bằng chứng, Phát hành — v0.2.10 & v0.2.11 đã publish, Sự cố phát hành mới (R4) — publish fail im lặng

### Community 322 - "Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)"
Cohesion: 0.13
Nodes (15): 0. Checklist S — BẮT BUỘC 100% trước khi nộp (không bỏ bước nào), 2. Hành vi của bộ cài trong chế độ silent (đã kiểm chứng), 2a. Ảnh listing (Box art / Poster art), 2b. Nguồn tải cho Partner Center — dùng branch `approved` (raw URL, KHÔNG redirect), 2c. MSIX — đường nộp Store CHÍNH, 3. Checklist trước khi submit, 4. Nếu Store từ chối, 5. Quy trình publish lên Microsoft Store — từng bước (+7 more)

### Community 323 - "Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành"
Cohesion: 0.50
Nodes (4): Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành, Gate trước tag 0.2.4, Tiếp tục audit ZCode — engine (2026-10-01), Tiếp tục audit ZCode — runtime chưa có bằng chứng production

### Community 324 - "ffi/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 325 - "Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent"
Cohesion: 0.50
Nodes (4): Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent, Giải pháp + bằng chứng kiểm thử, Phát hành — v0.2.8 đã publish (release candidate, chưa ký số), Sự cố trong phiên — tra trước khi lặp lại

### Community 327 - "Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được"
Cohesion: 0.67
Nodes (3): Ba nguyên nhân gốc, ba fixes, xác minh máy thật, Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được, Phát hành — v0.2.12 đã publish

### Community 328 - "Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath"
Cohesion: 0.67
Nodes (3): Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath, Ký số — SignPath opt-in, Phát hành — v0.2.5 đã publish (release candidate, chưa ký số)

### Community 329 - "Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục"
Cohesion: 0.67
Nodes (3): Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục, Kiểm chứng E2E trên máy thật (2026-10-02, sau khi publish), Phát hành — v0.2.7 đã publish (release candidate, chưa ký số)

### Community 330 - "Diagnostics"
Cohesion: 0.18
Nodes (7): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, FileHandle

### Community 331 - "[0.2.4] — 2026-10-01"
Cohesion: 0.67
Nodes (3): [0.2.4] — 2026-10-01, Fixed, Verification

### Community 332 - "[0.2.12] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.12] — 2026-10-03, Fixed (Windows), Known issue (ghi nhận từ máy chủ repo, 0.2.12 chưa xử lý được)

### Community 334 - "verify-msix.py"
Cohesion: 0.21
Nodes (10): argparse, os, check(), main(), map_version(), png_size(), verify-msix.py — kiểm gói .msix TRƯỚC khi nộp Partner Center (R2-09). makeappx…, 0.2.27 → 1.2.27.0 — phải khớp tools/win/build-msix.ps1 (R2-01). (+2 more)

### Community 335 - "[0.2.15] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.15] — 2026-10-03, Fixed (Windows), Verified — toàn bộ luồng portable "chạy ngay" (yêu cầu chủ repo)

### Community 336 - "[0.2.16] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.16] — 2026-10-03, Fixed (Windows) — portable "giải nén chạy ngay" hoạt động trên Win11 24H2+, Verified live trên máy thật (máy chủ repo, build 26300 — trường hợp từng fail)

### Community 337 - "registration_ok"
Cohesion: 0.21
Nodes (12): ctf_category_keys(), ctf_layout_matches_tsf_registry_schema(), ctf_profile_key(), machine_profile_metadata_ok(), profile_is_enabled(), profile_metadata_ok(), reg_key_exists(), reg_read_dword() (+4 more)

### Community 338 - "3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft"
Cohesion: 0.17
Nodes (12): 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU), 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?, 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft, 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store, 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation, 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?, 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft, 3. Lỗ hổng định danh ký tự: Nguy cơ lệch Codepoint nghiêm trọng giữa "Linh βùi" và Partner Center (+4 more)

### Community 339 - "Option"
Cohesion: 0.30
Nodes (10): guard_decision(), GuardAction, is_valid_pfn(), parse_install_args(), parse_relay_args(), parse_version_quad(), prunable_version_dir(), Option (+2 more)

### Community 340 - "Option"
Cohesion: 0.26
Nodes (12): current_package_family_name(), current_package_full_name(), detect_store_context(), dir_is_under(), read_package_string(), read_stage(), Fn, Option (+4 more)

### Community 341 - "try_read_pipe_frame"
Cohesion: 0.22
Nodes (11): HANDLE, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, connected_client_pid(), current_user_pipe_security(), peek_available(), PipeFrame, AtomicBool (+3 more)

### Community 342 - "textvn_ibus_modifier_kind"
Cohesion: 0.29
Nodes (9): guint, lc_modifier, textvn_ibus_map_key(), textvn_ibus_modifier_kind(), textvn_ibus_mods(), guint, lc_key, classify() (+1 more)

### Community 343 - "Nộp TextVN lên Microsoft Store — gói MSIX"
Cohesion: 0.20
Nodes (8): 0. Trạng thái — gói nộp Store 0.2.28 đã build (identity Partner Center có từ 2026-10-10), 1. Vì sao gói MSIX của TextVN có cơ chế "stage-out", 2. Partner Center — việc của chủ tài khoản, 3. Build gói nộp Store, 4. Nộp submission, 5. Rủi ro còn lại & cách xử lý, 6. Sau khi publish, Nộp TextVN lên Microsoft Store — gói MSIX

### Community 344 - "package_family_full_names"
Cohesion: 0.24
Nodes (10): MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, PWSTR, message_box(), open_apps_settings(), package_family_full_names(), report_error(), Vec (+2 more)

### Community 345 - "run_store_guard"
Cohesion: 0.24
Nodes (8): install_args(), LaunchMode, relay_and_install_args_roundtrip(), relay_args(), Vec, GuardOutcome, run_store_guard(), write_relaunch_hint()

### Community 346 - "3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 347 - "P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên Windows, 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`), 3. App preset mặc định — `data/appdb.default.json` (phần Windows), 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4), 5. Tích hợp với adapter — API nội bộ chung, 6. Test & bằng chứng, 7. Mapping task, P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

### Community 348 - "3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 349 - "Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store"
Cohesion: 0.22
Nodes (9): 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào, 2. Vòng 15 — những gì đã triển khai trong `release.yml`, 3. Khoá ký GPG release (TextVN Release Signing), 4. Authenticode cho Microsoft Store (chưa xong — đang chờ), 4b. macOS — Developer ID + notarization, 5. Quy tắc khóa-tay (đừng lặp), Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store, Verify một release (dùng GPG) (+1 more)

### Community 350 - "OutputStrategy"
Cohesion: 0.29
Nodes (7): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, Int64

### Community 351 - "SecurityState"
Cohesion: 0.36
Nodes (4): FocusProbe, FieldContext, Option, SecurityState

### Community 352 - "P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow"
Cohesion: 0.25
Nodes (7): 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng), 2. Trách nhiệm từng crate & ranh giới, 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa), 4. CI (GitHub Actions), 5. Versioning & branching, 6. Tham chiếu ngược, P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow

### Community 353 - "P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết"
Cohesion: 0.25
Nodes (7): 1. Nguồn `FieldContext` trên macOS, 2. Bảng AX → `IME_FIELD_*` (module `FieldDetect.swift`, mirror `P1-3 §2`), 3. App preset mặc định — `data/appdb.default.json` (phần macOS), 4. Override & state (khớm `P0-3 §3.1` — cùng bảng với `P1-3 §4`, khác path macOS), 5. API nội bộ (dùng chung IMK + tap), 6. Test, P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết

### Community 354 - "Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI"
Cohesion: 0.25
Nodes (7): 1. Microsoft nói chính xác điều gì, 2. Điều này thay đổi nhận định gì, 3. Ba con đường (đối chiếu chi phí / rủi ro / thời gian), 4. Kế hoạch chốt (theo thứ tự ưu tiên — cập nhật 2026-10-09), 5. Hiện trạng kỹ thuật đã sẵn sàng (2026-10-09), 6. Việc cần làm — checklist, Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

### Community 355 - "1. Điền form Partner Center — từng ô một"
Cohesion: 0.25
Nodes (8): 1. Điền form Partner Center — từng ô một, 1a-bis. Trạng thái kỳ vọng của "Package validation", 1a. "Provide any switches required for silent installation…", 1a-ter. Lịch sử 7 vòng + kế hoạch nộp có thứ tự, 1b. "Installer runs in silent mode but does not require switches", 1c. "Select the languages that your application supports", 1d. "Select your app type", 1e. EXE Return Codes

### Community 356 - "3. SOLUTION ARCHITECT VIEW"
Cohesion: 0.25
Nodes (8): 3.1 C4 — Context, 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*, 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**, 3.4 Output Strategy Engine (trái tim của app-compat), 3.5 Config & state, 3.6 UI/Tray (1 codebase, 3 native shell), 3.7 Packaging & Distribution, 3. SOLUTION ARCHITECT VIEW

### Community 357 - "3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM""
Cohesion: 0.29
Nodes (7): 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store, 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM", 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM", 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant), 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên, Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`, Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC

### Community 358 - "StageInfo"
Cohesion: 0.29
Nodes (6): stage_json_roundtrip_and_validation(), StageInfo, launch_package_app(), Result, spawn_detached(), write_stage()

### Community 359 - "Bool"
Cohesion: 0.40
Nodes (3): FieldRole, Bool, UInt32

### Community 360 - "GlobalHookContext"
Cohesion: 0.33
Nodes (6): GlobalHookContext, HWND, Instant, HHOOK, Receiver, Sender

### Community 361 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.33
Nodes (6): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust

### Community 362 - "IMPLEMENTATION STATUS — adapter macOS (Farch-4)"
Cohesion: 0.33
Nodes (5): Còn mở (2026-10-09), IMPLEMENTATION STATUS — adapter macOS (Farch-4), Tiếp theo (tuần tự), Trạng thái (2026-10-09), Đã triển khai (bảng thành phần)

### Community 363 - "3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)"
Cohesion: 0.33
Nodes (6): 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu, 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions), 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions), 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết, Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE), Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)

### Community 364 - "5. TECHNICAL EXPERT VIEW"
Cohesion: 0.33
Nodes (6): 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**), 5.2 Env compatibility (được yêu cầu đặc biệt), 5.3 Browser compatibility matrix (mục tiêu v1.0), 5.4 App compat matrix (40 app, giữ trong `docs/compat.md`, CI chạy subset), 5.5 Performance budget, 5. TECHNICAL EXPERT VIEW

### Community 365 - "windows-hook/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 366 - "[0.2.28] — 2026-10-10"
Cohesion: 0.40
Nodes (5): [0.2.28] — 2026-10-10, Added, Fixed — kiểm trên Windows 11 thật và rà soát đối kháng (R2-88…R2-98, chi tiết: `docs/release/code-review-2026-10-09-round2.md` §3.7), Fixed — rà soát mã nguồn chuyên sâu 2026-10-07 (chi tiết: `docs/release/code-review-2026-10-07.md`), Fixed — rà soát vòng 2: Microsoft Store, ký số, QA (chi tiết: `docs/release/code-review-2026-10-09-round2.md`)

### Community 367 - "3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối"
Cohesion: 0.40
Nodes (5): 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối, Gốc rễ 1: Sự ngộ nhận tai hại về HKCU (Per-User) vs HKLM (Per-Machine) trong môi trường tự động của Store, Gốc rễ 2: Lệch chuỗi định danh tuyệt đối (Exact String Mismatch) giữa Partner Center và Registry, Gốc rễ 3: Điểm mù của CI Test Harness hiện tại (`validate-store-package.ps1` & `simulate-store-validation.ps1`), Gốc rễ 4: Bẫy lưu đệm (Caching Trap B13g) của Partner Center

### Community 368 - "4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)"
Cohesion: 0.40
Nodes (5): 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), Phát hiện L-01: Bộ phân tích JSON IPC thủ công bằng `strstr`, Phát hiện L-02: Đường dẫn Socket không an toàn trong `/tmp`, Phát hiện L-03: Thiếu định dạng đóng gói chuẩn của các bản phân phối Linux, Phát hiện L-04: Xung đột Wayland vs X11 và Cơ chế Surrounding Text trên Chat Apps

### Community 369 - "Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)"
Cohesion: 0.40
Nodes (4): 0. Baseline (trước khi sửa), 1. Kế hoạch (thứ tự theo rủi ro: engine chạy trong process của mọi app → FFI → adapter), 4. Kết quả cuối (P11), Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)

### Community 370 - "Trạng thái ký số macOS — Developer ID, notarization, stapling"
Cohesion: 0.40
Nodes (5): 1. Tóm tắt, 2. Ảnh hưởng tới người dùng, 3. Tooling đã nối (chỉ chạy khi có secret), 4. Việc còn lại để bật (chủ repo), Trạng thái ký số macOS — Developer ID, notarization, stapling

### Community 371 - "6. AUDIT EXPERT VIEW"
Cohesion: 0.40
Nodes (5): 6.1 Threat model (tóm tắt STRIDE cho IME), 6.2 License & compliance plan, 6.3 Security/quality gates (mọi PR phải qua), 6.4 External audit (trước v1.0), 6. AUDIT EXPERT VIEW

### Community 373 - "tray/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 374 - "[0.2.8] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.8] — 2026-10-02, Added, Changed, Fixed

### Community 377 - "[0.2.20] — 2026-10-04"
Cohesion: 0.67
Nodes (3): [0.2.20] — 2026-10-04, Fixed (Windows) — CI đỏ .iss + vòng 3 validation (B13/B14), Security / hardening (audit 2026-10-04, phần còn lại)

### Community 378 - "[0.2.26] — 2026-10-05"
Cohesion: 0.67
Nodes (3): [0.2.26] — 2026-10-05, Added (Windows) — hỗ trợ app 32-bit (Zalo/Office x86) — vòng 14, Fixed (Windows) — gỡ cài sạch 100% key layout (vòng 13, bắt thật + dọn live trên máy chủ repo)

### Community 379 - "[0.2.29] — 2026-10-10"
Cohesion: 0.67
Nodes (3): [0.2.29] — 2026-10-10, Changed, Fixed — gỡ cài đặt sạch (R2-99, R2-100)

### Community 380 - "[0.2.5] — 2026-10-02"
Cohesion: 0.67
Nodes (3): [0.2.5] — 2026-10-02, Added, Fixed (portable)

### Community 381 - "Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)"
Cohesion: 0.67
Nodes (3): 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

## Knowledge Gaps
- **1505 isolated node(s):** `ic`, `inst`, `comp`, `config`, `toggle` (+1500 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 2366 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **70 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `2. Nhật ký finding` connect `ipc_client.c` to `OwnerRule`, `IpcClient`, `IMKLibTests.swift`, `TapKeyHandler`, `TextVNInputController`, `Rà soát mã nguồn chuyên sâu — 2026-10-07 (v0.2.27)`, `MarkedState`, `KeyOutcome`, `TextVNAppTests`, `TextVNConfig`, `.inject`, `IpcServer`?**
  _High betweenness centrality (0.153) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `ConfigWatcher`, `IpcClient`, `AppDelegate`, `EventTapController`, `StrategyResolver`, `Foundation`, `IpcMessage`, `IpcServer`, `OwnerRule`, `MockHandler`, `IMKLibTests.swift`, `ConfigStore`, `TextVNInputController`, `ViState`, `B. macOS`, `AXSnapshot`, `TextVNConfig`, `.inject`, `IMKTextTarget`, `AutostartManager`, `Bool`, `FieldContext`, `MarkedState`, `KeyOutcome`, `TextVNAppTests`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Why does `text()` connect `text` to ``docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)`, `PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)`, `2. PRODUCT OWNER VIEW`, `[0.2.8] — 2026-10-02`, `e2e_fcitx5.py`, `TextVN — Chỉ mục tài liệu & Giao thức Review`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `AppDelegate` (e.g. with `B. Trên macOS: Quên cập nhật Icon Menu Bar & Luồng giao diện (UI Thread Violation)` and `(A) macOS app / IMK / packaging / CI — `F3-*``) actually correct?**
  _`AppDelegate` has 4 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `TextVNInputController` (e.g. with `KeyTranslator` and `MarkedState`) actually correct?**
  _`TextVNInputController` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ic`, `inst`, `comp` to the rest of the system?**
  _1505 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `core/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12828282828282828 - nodes in this community are weakly interconnected._