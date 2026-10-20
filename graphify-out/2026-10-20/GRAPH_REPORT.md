# Graph Report - TextVN  (2026-10-18)

## Corpus Check
- 351 files · ~505,649 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 303 file(s) not represented in the graph (top: .keys 248, (none) 14, .toml 10)

## Summary
- 5902 nodes · 11097 edges · 399 communities (327 shown, 72 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 699 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `861d6ecd`
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
- diacritic_style.rs
- keymap.rs
- push_key
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
- IpcServer
- svc.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- store.rs
- windows-hook/src/lib.rs
- Chi tiết từng mục
- Tip_Impl
- tsf-min/src/register.rs
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
- ime_result_v1
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- settings_model.c
- textvn-ffi
- items
- compose.rs
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- tray/src/lib.rs
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- core
- P3-0 — LINUX MASTER PLAN (Phần 3)
- TextVNInputController
- P2-REVIEW-LOG — Phần 2 (macOS)
- settings_window.c
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- T6 — Test & hardening (tuần 15–18)
- KeyTranslator
- Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)
- XCTestCase
- textvn-common.sh
- appdb.v1.schema.json
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- test_settings_model.c
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- type_keys
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- class.rs
- ipc_server.rs
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- tsf-min/src/lib.rs
- ByteCursor<'a>
- rules_win.rs
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
- SvcManager
- free_marking
- tip.rs
- windows
- adr/README.md
- tsf-min
- EventTapController
- xtask
- WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"
- tone.rs
- Strategy
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- encode_frame
- compose.c
- bool
- store_win.rs
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- common_words.rs
- guids.rs
- uia_spike.ps1
- TsfShared
- do_unregister
- Project Common Errors — Sổ lỗi chung toàn dự án
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- test_compose.c
- ipc_client.c
- log.c
- P0 — REVIEW LOG (Phần 0: Nền tảng chung)
- SettingsDoc
- Result
- package_bootstrap.rs
- win32-uia.lib.ps1
- trace.rs
- engine.cpp
- IpcClient
- ipc_client.rs
- bitset
- verify_targets.ps1
- HWND
- Performance Audit — TextVN Windows Platform
- create_and_show_window
- generate_tray_icons.py
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- B. macOS
- check-pe-imports.py
- invariants.rs
- build-release.ps1
- AppDelegate
- check_no_secrets.py
- ime_context
- Engine
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- register_and_activate_tsf
- Foundation
- .CreateInstance
- IpcServer
- ThreadState
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- .new
- OwnerRule
- linux_common.h
- 1. Lộ trình ba bước
- cli/src/main.rs
- Security Policy
- Đóng góp cho TextVN
- DocError
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- macro_text.rs
- SettingsView
- suggest
- AppDb
- String
- 2. PRODUCT OWNER VIEW
- e2e_ibus.c
- generate_app_icons.py
- abi_invariants.rs
- Result
- e2e_fcitx5.py
- AXSnapshot
- key_event.rs
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- ConfigStore
- install_linux.sh
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- ime_field_body
- securitystate
- linux-fcitx5/src/engine.h
- show_dialog_on_startup
- ffi/src/settings.rs
- Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`)
- foreground.rs
- P3-6 — TEST PLAN (Linux) — Solution chi tiết
- AutostartManager
- FieldDetect
- Path
- TapPermission
- OutputStrategy
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- quick_telex.rs
- 3. SOLUTION ARCHITECT VIEW
- Hướng dẫn phát triển TextVN
- Báo cáo dựng & kiểm thử — TextVN
- remove_added_languages
- A. WINDOWS
- fuzz — L4 (P0-4 §1)
- .inject
- RoundTwoMacAppTests
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
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
- Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- quick_telex
- e2e-linux.sh
- mem-check.sh
- soak.sh
- Trạng thái ký số macOS — Developer ID, notarization, stapling
- [0.2.32] — 2026-10-12
- 4. ENGINEER VIEW
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
- 6. AUDIT EXPERT VIEW
- command
- check_version_sync.rs
- Bản 0.2.31 — hoàn tất roadmap audit 3 nền tảng (A10, B7/B8/B9/B11/B12/B13, C1/C3/C7/C8/C9) + gate clippy-linux
- ime_result
- tv_win_free
- [0.2.0] — 2026-09-30
- WORDLIST_LAYOUT
- .new
- TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux
- [0.2.13] — 2026-10-03
- 3. Nhật ký finding
- [0.2.14] — 2026-10-03
- [0.2.9] — 2026-10-02
- verify-store-package.sh
- Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)
- CLIENT
- [0.2.10] — 2026-10-03
- Word
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- Added
- wide
- [0.2.3] — 2026-10-01
- Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- [0.2.33] — 2026-10-18
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
- .log
- 5. TECHNICAL EXPERT VIEW
- Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất
- vowel_table.rs
- verify-msix.py
- diacritic_style
- [0.2.16] — 2026-10-03
- do_register
- 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft
- DialogContext
- [0.2.12] — 2026-10-03
- [0.2.15] — 2026-10-03
- [0.2.29] — 2026-10-10
- [0.2.20] — 2026-10-04
- Option
- macro_trigger
- 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU
- Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store
- keys_generated.rs
- [0.2.18] — 2026-10-03
- ime_key
- [0.2.25] — 2026-10-05
- Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI
- caps.rs
- prepare-ppa.sh
- 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"
- StageInfo
- ime_context
- TextVNAppTests
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- TapDecision
- 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)
- Bản 0.2.32 — rà soát toàn hệ thống sau 0.2.31 (R2-101…R2-115) + CI từng nền tảng
- [0.2.28] — 2026-10-10
- GlobalHookContext
- pipe_name
- C. LINUX + SHARED
- [0.2.4] — 2026-10-01
- linux-settings/src/main.c
- ApplyError
- Phân phối Linux — kênh, store và auto-update
- [0.2.26] — 2026-10-05
- Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)
- sign-artifacts.sh
- TextVN — Chỉ mục tài liệu & Giao thức Review
- cli/build.rs
- on_dir_changed
- Bản 0.2.16 — portable tự chữa trên Win11 24H2+ (B7) + hết ghost khi gỡ (0.2.15)
- Bản 0.2.4 — đăng ký không cần admin + clean composition + Ctrl+Shift single-path
- Bản 0.2.6 — chốt đăng ký TSF VI+EN, hết ghost registration
- Bản 0.2.9 — từ điển EN cá nhân + quy tắc ưu tiên + preflight 17 gate
- error_success
- unmark_as_unmark_tone
- .value
- super
- Problem
- [0.2.34] — 2026-10-18
- [0.2.22] — 2026-10-04
- P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)
- ModifierToggle
- [0.2.8] — 2026-10-02
- publish-aur.sh

## God Nodes (most connected - your core abstractions)
1. `AppDelegate` - 65 edges
2. `SvcManager` - 55 edges
3. `TextVNInputController` - 49 edges
4. `Báo cáo dựng & kiểm thử — TextVN` - 40 edges
5. `Changelog` - 39 edges
6. `MarkedState` - 37 edges
7. `TsfShared` - 37 edges
8. `Engine` - 37 edges
9. `TextVNEngine` - 35 edges
10. `IpcServer` - 34 edges

## Surprising Connections (you probably didn't know these)
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `5. Perf (ngưỡng — `PLAN §5.5`)` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-6-test-plan.md → adapters/linux-common/src/field_detect.c
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md → adapters/linux-common/src/ipc_client.c
- `Đã làm ĐÚNG: `lc_ipc_client_poll` + rbuf per-client stash (CR-25 fix +` --references--> `lc_ipc_client_poll()`  [INFERRED]
  docs/release/AUDIT-3PLATFORM-2026-10-07.md → adapters/linux-common/src/ipc_client.c

## Import Cycles
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`

## Communities (399 total, 72 thin omitted)

### Community 0 - "core/src/lib.rs"
Cohesion: 0.13
Nodes (43): ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, auto_capitalize_after_dot_and_enter(), auto_capitalize_needs_space_after_dot(), auto_capitalize_only_first_letter_of_word(), backspace_after_sentence_end_cancels_auto_capitalize() (+35 more)

### Community 1 - "Result"
Cohesion: 0.12
Nodes (22): ClassFactory_Impl, CompSink_Impl, EditSession_Impl, KeySink_Impl, log(), BOOL, c_void, IClassFactory_Impl (+14 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.08
Nodes (47): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, case_without_any_expectation_is_rejected(), Cmd (+39 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.07
Nodes (30): Config, ConfigError, default_json(), defaults_match_p0_3_table(), DiacriticStyle, EmojiEntry, FULL_EXAMPLE, full_example_parses() (+22 more)

### Community 4 - "text"
Cohesion: 0.20
Nodes (12): check(), text(), 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`), 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`), 3. Bảng xung đột có thể xảy ra và cách giải, 4. Nguyên tắc điều chỉnh thiết kế (ghi để không lặp tranh luận), Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên, 1.1 Bảng tổng hợp (+4 more)

### Community 5 - "verify.rs"
Cohesion: 0.11
Nodes (32): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+24 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (40): caps_lock_bit(), CTRL_DOWN, empty_ime_result(), get_active_modifiers(), get_process_name_for_window(), GLOBAL_ENABLED, HOOK_MAIN_THREAD_ID, IN_INJECTION (+32 more)

### Community 7 - "toml.rs"
Cohesion: 0.15
Nodes (18): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+10 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.13
Nodes (21): banner(), emit(), format_rust(), gen_tables(), KEYMAP_MAC_OUT_RUST, KEYMAP_MAC_OUT_SWIFT, KEYMAP_MAC_TOML, KEYS_OUT (+13 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.07
Nodes (48): ffi, fill_result(), flow_new_key_reset_free(), guarded(), held_key_long_word_never_deletes_past_typed_text(), IME_ABI_VERSION, ime_appdb_verify(), ime_context_v1 (+40 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.07
Nodes (22): ActionKind, Option, IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY (+14 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.12
Nodes (16): A, A_BREVE, A_CIRC, E, E_CIRC, I, O, O_CIRC (+8 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.12
Nodes (27): black_box, instant, bench_field_switch_resolve(), bench_ime_key(), bench_parse_config(), bench_resolve(), best_of(), best_of_giu_vong_nhanh_nhat() (+19 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.06
Nodes (37): call_layout_or_tip(), call_layout_or_tip_spec(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, CLSID_INNER, clsid_key() (+29 more)

### Community 14 - "diacritic_style.rs"
Cohesion: 0.22
Nodes (10): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), vowel_span(), is_vowel (+2 more)

### Community 15 - "keymap.rs"
Cohesion: 0.06
Nodes (26): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, ESCAPE, F1, KeyEvent (+18 more)

### Community 16 - "push_key"
Cohesion: 0.09
Nodes (25): apply_key, late_stroke(), push_key(), fold(), marker_to_tone(), n(), push_key(), DiacriticStyle (+17 more)

### Community 17 - "validate.rs"
Cohesion: 0.14
Nodes (19): coda_after_nucleus_ok(), CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI, ONSETS (+11 more)

### Community 18 - "macro.rs"
Cohesion: 0.13
Nodes (22): Emoji, emoji_glyph_is_multi_char(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches(), MacroDef (+14 more)

### Community 19 - "telex.rs"
Cohesion: 0.08
Nodes (17): codas, circum_pair(), f(), fold(), fold_with(), fold_with_caps(), has_uo_pair(), late_d_strokes_the_onset() (+9 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.08
Nodes (37): after_request_edit_session(), classify_tsf_field(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, external_edit_clears_ownership_before_next_replace(), focus_generation_rejects_old_uia_result_and_stays_fail_safe(), hotkey_toggle_switches_enabled_state_and_resets_buffer() (+29 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.10
Nodes (23): am_tiet_viet_that_khong_bi_restore(), canh_bao_cai_muc_mo_ho(), chars(), complete_word(), data_contains(), data_en_common_file_fires(), data_stop_en_file_is_self_consistent(), data_vn_common_file_is_valid() (+15 more)

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
Cohesion: 0.11
Nodes (42): ime_instance, lc_config_state, file_stamp(), lc_config_sync(), lc_state_read_enabled(), lc_state_sync(), test_state_file(), gboolean (+34 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+9 more)

### Community 29 - "IpcServer"
Cohesion: 0.16
Nodes (11): ClientSink, IpcServer, PipeFrame, AtomicBool, AtomicU32, File, Mutex, Vec (+3 more)

### Community 30 - "svc.rs"
Cohesion: 0.14
Nodes (16): textvn_config, atomic_write_file(), corrupt_config_is_backed_up_not_lost(), default_config_dir(), failed_config_save_keeps_memory_and_version_unchanged(), Arc, BTreeMap, Path (+8 more)

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
Cohesion: 0.06
Nodes (39): APP_ID, ARG_AUTOSTART, ARG_GUARD, ARG_INSTALL, ARG_RELAY, build_command_line(), decide_store_run_value(), guard_decision() (+31 more)

### Community 35 - "windows-hook/src/lib.rs"
Cohesion: 0.15
Nodes (20): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), CallbackDecision, empty_result(), engine_never_transforms_when_policy_says_pass(), hook_db(), HookMode, HookState (+12 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "Tip_Impl"
Cohesion: 0.15
Nodes (16): F, GUID, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ITfTextInputProcessor_Impl, ITfThreadMgr (+8 more)

### Community 38 - "tsf-min/src/register.rs"
Cohesion: 0.19
Nodes (21): ITfInputProcessorProfiles, activate(), clsid_key(), com_init(), dll_path(), guid_str(), ILOT_DEFPROFILE, install() (+13 more)

### Community 39 - "P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator"
Cohesion: 0.15
Nodes (12): 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy), 2.1 Tổng quan, 2.2 Bảng lệnh (chính thức), 2.3 Ví dụ đầy đủ (phải chạy được ngay), 2.3b Lệnh bổ sung (bắt buộc có trong parser), 2.4 Tên phím chuẩn (case-sensitive), 2. Định dạng corpus `.keys`, 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán (+4 more)

### Community 40 - "matchClause"
Cohesion: 0.15
Nodes (13): type, $ref, $ref, matchClause, $ref, additionalProperties, minProperties, properties (+5 more)

### Community 41 - "properties"
Cohesion: 0.15
Nodes (13): default, type, $comment, const, default, enum, default, enum (+5 more)

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
Cohesion: 0.14
Nodes (15): Key, StrategyResolver, .cacheCount, .hasAppdb, Bool, Data, Int, Int32 (+7 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "field-detect/src/lib.rs"
Cohesion: 0.11
Nodes (20): FocusProbe, K, CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), FieldContext, normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated() (+12 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.02
Nodes (90): gdi, hidpi, MSG, normalize_word_list, ABOUT_DETAILS, ABOUT_TAGLINE, about_text(), about_text_matches_every_platform() (+82 more)

### Community 62 - "ime_result_v1"
Cohesion: 0.21
Nodes (8): EngineSession, macro_when_vi_off_runs_engine_in_passthrough(), Drop, ime_instance, Result, runtime_config_reload_updates_engine_method(), ime_reset(), ime_result_v1

### Community 63 - "appdb/src/lib.rs"
Cohesion: 0.10
Nodes (32): AppDbError, DB, documented_metadata_and_match_clause_parse(), engine_version_is_supported(), EngineOwner, Entry, FieldRoles, first_matching_entry_and_role_win() (+24 more)

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

### Community 68 - "settings_model.c"
Cohesion: 0.14
Nodes (24): lc_config_resolve_path(), lc_state_resolve_path(), lc_state_write_enabled(), lc_textvn_file_path(), tv_paths, edit_bool(), edit_reset(), edit_str() (+16 more)

### Community 69 - "textvn-ffi"
Cohesion: 0.42
Nodes (13): textvn-appdb, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-ffi, textvn-field-detect, textvn-fuzz (+5 more)

### Community 70 - "items"
Cohesion: 0.18
Nodes (13): default, items, type, items, additionalProperties, pattern, required, type (+5 more)

### Community 71 - "compose.rs"
Cohesion: 0.06
Nodes (36): BACK, backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CompositionPlan, CONTROL, delete_beyond_composition_is_reported_not_guessed(), Doc (+28 more)

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "tray/src/lib.rs"
Cohesion: 0.05
Nodes (45): ipcserver, ordering, settings, show_settings_dialog, claim_debounced(), compatibility_hook_path(), CTRL_SHIFT_MARKER, CTRL_SHIFT_MARKER_DECLINED (+37 more)

### Community 74 - "M0 — Spike & nền (tuần 1–2)"
Cohesion: 0.22
Nodes (9): M0 — Spike & nền (tuần 1–2), WIN-001 · Chuẩn bị môi trường build (S, dep: —), WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**, WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002), WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001), WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001), WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song), WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —) (+1 more)

### Community 75 - "P2-0 — macOS MASTER PLAN (Phần 2)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần macOS (exit condition), 2. Kiến trúc process (nhất quán với `P1-0 §2`), 3. Workstream & file solution, 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2), 5. Dependency graph, 6. Rủi ro & mitigation (riêng macOS), 7. Definition of Done cho PHẦN 2, P2-0 — macOS MASTER PLAN (Phần 2)

### Community 76 - "core"
Cohesion: 0.21
Nodes (11): Cell, Default, ObjGuard, Self, Vec, TextVNDisplayAttributeInfo, TextVNDisplayAttributeProvider, TextVNEnumDisplayAttributeInfo (+3 more)

### Community 77 - "P3-0 — LINUX MASTER PLAN (Phần 3)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Linux (exit condition), 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`), 3. Workstream & file solution, 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3), 5. Dependency graph, 6. Rủi ro & mitigation (riêng Linux), 7. Definition of Done cho PHẦN 3, P3-0 — LINUX MASTER PLAN (Phần 3)

### Community 78 - "TextVNInputController"
Cohesion: 0.09
Nodes (18): Any, Bool, Data, FieldContext, Int, IpcClient, NSEvent, pid_t (+10 more)

### Community 79 - "P2-REVIEW-LOG — Phần 2 (macOS)"
Cohesion: 0.08
Nodes (23): AppInfo, .aboutMessage, .aboutTitle, (B) Windows / xtask / Linux — `R3-*`, Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging), Code Review Round 2 — Nhất quán & Sẵn sàng (Kiểm thử, Autostart S5 & Residue S9), Kiểm chứng sau fix (Review 2 cuối), Kiểm chứng vòng 10 (host Windows, trước commit) (+15 more)

### Community 80 - "settings_window.c"
Cohesion: 0.16
Nodes (32): add_escape_closes(), button(), GtkApplication, framed_grid(), labeled_dropdown(), on_about(), on_close(), on_default() (+24 more)

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

### Community 85 - "KeyTranslator"
Cohesion: 0.26
Nodes (7): KeyTranslator, Data, KeyEvent, NSEvent, UInt16, UInt32, TISInputSource

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "XCTestCase"
Cohesion: 0.10
Nodes (11): KeyMapMacGenerated, UInt32, Bool, NSMenu, IpcCodecTests, KeyMapMacTests, MarkedStateTests, MarkedStateUnitTests (+3 more)

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

### Community 92 - "test_settings_model.c"
Cohesion: 0.29
Nodes (16): tv_macros_load(), tv_snapshot_load(), tv_paths, main(), read_file(), test_corrupt_file(), test_defaults_when_missing(), test_labels_and_values() (+8 more)

### Community 93 - "M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006"
Cohesion: 0.29
Nodes (7): M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006, WIN-030 · crate `field-detect` + rules R1–R10 (M) — `P1-3 §2`, WIN-031 · Cache + invalidation + budget 2ms (M), WIN-032 · `engine_owner` + override chain (M) — `P1-3 §4`, `P1-2 §6`, WIN-033 · appdb loader + Ed25519 verify (M) — `P0-3 §2.2`, WIN-034 · Preset 20 app + corpus ≥ 40 case (M) — `P1-3 §3`, WIN-035 · Settings: "thêm preset từ app đang chạy" (S, dep: WIN-052)

### Community 94 - "M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005"
Cohesion: 0.29
Nodes (7): M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005, WIN-040 · `textvn-hook.exe` skeleton + IPC + heartbeat (M) — `P1-2 §2`, WIN-041 · Callback theo `P1-2 §3` (L), WIN-042 · Inject engine 3 modes + modifier restore (L) — `P1-2 §5`, WIN-043 · Focus + UIA worker (M) — `P1-2 §4`, WIN-044 · Rule `engine_owner` phía hook (S, dep: WIN-032), WIN-045 · Game/legacy presets + blocklist (M) — `P1-2 §8`

### Community 95 - "A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006"
Cohesion: 0.29
Nodes (7): A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006, MAC-030 · FieldDetect rules R1–R10 + mock tests (M) — `P2-3 §2`, MAC-031 · Cache + AXObserver + budget (M), MAC-032 · Preset 20 app + corpus ≥ 40 case (M) — `P2-3 §3`, MAC-033 · Appdb loader + Ed25519 (S) — dùng `textvn-appdb`, không viết lại, MAC-034 · Override chain `P2-3 §4` (path mac) (M), MAC-035 · Settings "thêm app đang chạy" (S, dep: MAC-052)

### Community 96 - "type_keys"
Cohesion: 0.22
Nodes (9): boundary_space_commits_with_preedit_cap(), chord_drops_word_ownership(), control_chars_from_layout_are_navigation_not_boundary(), english_passes_through_untouched(), golden_duocj_with_preedit(), keyup_does_not_change_state(), navigation_cancels_preedit_before_next_typing(), secure_context_all_pass() (+1 more)

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

### Community 102 - "ipc_server.rs"
Cohesion: 0.15
Nodes (15): authorization, filesystem, HANDLE, pipes, PSECURITY_DESCRIPTOR, read, SECURITY_ATTRIBUTES, connected_client_pid() (+7 more)

### Community 103 - "env.md — Môi trường build Windows (WIN-001)"
Cohesion: 0.33
Nodes (5): 1. Build host (máy dev), 2. Tools DoD (Handbook §4 mục 4), 3. Bằng chứng acceptance (WIN-001), 4. Gap & việc cần khi lên milestone liên quan, env.md — Môi trường build Windows (WIN-001)

### Community 104 - "A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009"
Cohesion: 0.33
Nodes (6): A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009, MAC-040 · Tap thread skeleton + owner check (M) — `P2-2 §2/§3`, MAC-041 · Callback + self-disable (M) — `P2-2 §3`, MAC-042 · Inject §5 (3 modes + marker + restore) (L), MAC-043 · Permission UX + revocation poll (M) — `P2-2 §4`, MAC-044 · Rule owner `imk|tap` + IPC sync (S, dep: MAC-034)

### Community 105 - "tsf-min/src/lib.rs"
Cohesion: 0.10
Nodes (28): ITfKeyEventSink, ClassFactory, CLSID_TEXTVN_TIP, CompSink, DllCanUnloadNow(), DllGetClassObject(), EditSession, HR_CLASSNOTAVAILABLE (+20 more)

### Community 106 - "ByteCursor<'a>"
Cohesion: 0.33
Nodes (3): ByteCursor<'a>, Option, Self

### Community 107 - "rules_win.rs"
Cohesion: 0.16
Nodes (13): address_or_search(), classify(), contains(), ControlType, known(), lower(), ProbeResult, role() (+5 more)

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "hotkey.rs"
Cohesion: 0.07
Nodes (45): CLASS_NAME, DeferredReplay, deliver(), HOPS, module_instance(), Pending, prepare(), HINSTANCE (+37 more)

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
Nodes (18): ApplyReplace, Int, Void, MarkedState, .graphemeCount, .isEmpty, .scalarCount, .utf16Count (+10 more)

### Community 116 - "english_words"
Cohesion: 0.50
Nodes (4): $comment, default, type, english_words

### Community 119 - "KeyOutcome"
Cohesion: 0.07
Nodes (30): Action, commit, pass, replace, restore, FFI, ImeEngine, ImeError (+22 more)

### Community 120 - "SvcManager"
Cohesion: 0.14
Nodes (9): apply_option_checkbox(), effective_enabled(), failed_state_save_keeps_memory_and_version_unchanged(), AtomicBool, AtomicU64, Option, RwLock, svc_manager_initializes_and_toggles_state() (+1 more)

### Community 121 - "free_marking"
Cohesion: 0.67
Nodes (3): default, type, free_marking

### Community 122 - "tip.rs"
Cohesion: 0.16
Nodes (15): current_exe_name(), Default, ITfKeystrokeMgr, ObjGuard, Option, Rc, RefCell, Tip (+7 more)

### Community 127 - "EventTapController"
Cohesion: 0.06
Nodes (33): CGFlagMapper, EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, Bool, NSObjectProtocol, UInt16 (+25 more)

### Community 129 - "WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN""
Cohesion: 0.33
Nodes (5): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

### Community 130 - "tone.rs"
Cohesion: 0.24
Nodes (15): apply_key(), apply_key_literal_when_no_vowel(), apply_key_tone_and_undo(), apply_tone(), chars(), current_tone(), is_vowel(), key_to_tone() (+7 more)

### Community 131 - "Strategy"
Cohesion: 0.14
Nodes (20): crate, default_for_field, downgrade(), downgrade_without_cap(), inp(), resolve(), ResolveInput, Option (+12 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.18
Nodes (18): base_keys(), f(), finish(), fix_uo(), fold(), fold_caps(), generated_key_tables_are_self_consistent(), is_word_char() (+10 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (17): 10. Sự cố 2026-10-11 — Kaspersky cách ly bản cài từ Microsoft Store (MSIX 1.2.29.0), 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5 (+9 more)

### Community 134 - "encode_frame"
Cohesion: 0.13
Nodes (21): read_next_message(), Result, W, send_message(), candidate_pipe_names(), CodecError, decode_exact_frame(), encode_frame() (+13 more)

### Community 135 - "compose.c"
Cohesion: 0.11
Nodes (35): boundary_char(), lc_comp, lc_key, lc_modifier, lc_modifier_toggle, lc_plan, is_executable(), lc_comp_apply() (+27 more)

### Community 138 - "store_win.rs"
Cohesion: 0.11
Nodes (37): appx, commandext, OsStr, shellexecutew, LaunchMode, after_tray_start(), cmd_exe_path(), CREATE_BREAKAWAY_FROM_JOB (+29 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "common_words.rs"
Cohesion: 0.07
Nodes (52): enc(), encode(), Entry, OutputCharset, parse_pair(), push_pair(), Option, Vec (+44 more)

### Community 142 - "guids.rs"
Cohesion: 0.39
Nodes (7): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI, PROFILE_TEXTVN, GUID

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "TsfShared"
Cohesion: 0.10
Nodes (35): apply_display_attribute(), apply_plan(), caret_at_composition_end(), CompSink, CompSink_Impl, EndCompositionSession, finish(), focus_is_password_edit() (+27 more)

### Community 145 - "do_unregister"
Cohesion: 0.22
Nodes (14): do_unregister(), icon_path(), reg_read_string(), register_wow64_mirror(), registration_owned_by(), registration_owned_by_dir(), Scope, unregister_tip() (+6 more)

### Community 149 - "autostart.rs"
Cohesion: 0.11
Nodes (40): REG_SAM_FLAGS, APP_RUN_VALUE_NAME, autostart_command(), autostart_disabled_by_user(), AUTOSTART_DISABLED_MARKER, autostart_disabled_marker_path(), autostart_effective(), autostart_suppressed_for() (+32 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Changelog"
Cohesion: 0.07
Nodes (28): [0.2.11] — 2026-10-03, [0.2.17] — 2026-10-03, [0.2.19] — 2026-10-04, [0.2.1] — 2026-09-30, [0.2.21] — 2026-10-04, [0.2.23] — 2026-10-05, [0.2.24] — 2026-10-05, [0.2.27] — 2026-10-07 (+20 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.05
Nodes (68): PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-09, sau v0.2.27 + vòng rà soát 2), getmodulehandlew, HICON, id_exit, shell, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V (+60 more)

### Community 154 - "menu.rs"
Cohesion: 0.10
Nodes (25): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_HEALTH_STATUS, ID_HOOK_COMPAT_MODE (+17 more)

### Community 155 - "SettingsController"
Cohesion: 0.09
Nodes (18): Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer, MacroTrigger (+10 more)

### Community 156 - "test_compose.c"
Cohesion: 0.23
Nodes (19): doc, ime_instance, lc_plan, doc_text(), engine_with(), expect(), main(), press() (+11 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.11
Nodes (44): lc_ipc_client, client_disconnect(), get_time_ms(), handle_ipc_message(), ipc_debug_enabled(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override() (+36 more)

### Community 158 - "log.c"
Cohesion: 0.14
Nodes (16): lc_find_settings_binary(), ensure_parent_dirs(), lc_log(), lc_log_close(), lc_log_init(), launchSettings(), launch_settings(), bus_disconnected() (+8 more)

### Community 159 - "P0 — REVIEW LOG (Phần 0: Nền tảng chung)"
Cohesion: 0.33
Nodes (5): Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

### Community 160 - "SettingsDoc"
Cohesion: 0.13
Nodes (23): atomic_write(), create_private_dir(), defaults(), DocKind, invalid_values_are_rejected_without_changing_the_doc(), macros_text_round_trip_through_doc(), MAX_FILE_BYTES, missing_keys_read_schema_defaults() (+15 more)

### Community 162 - "Result"
Cohesion: 0.25
Nodes (16): index_of(), key_char(), Map, Method, one_char(), parse_keymap_mac(), parse_maps(), parse_method() (+8 more)

### Community 164 - "package_bootstrap.rs"
Cohesion: 0.26
Nodes (19): cleanup_old_backups(), copy_dir_recursive(), copy_overwrite(), dispatch(), installed_dir_for(), Option, Path, PathBuf (+11 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "trace.rs"
Cohesion: 0.18
Nodes (10): enabled(), event(), File, Mutex, Option, sink(), Arguments, sync (+2 more)

### Community 167 - "engine.cpp"
Cohesion: 0.06
Nodes (62): InputContext, Instance, KeyEvent, string, ime_instance, InputContext, lc_comp, lc_config_state (+54 more)

### Community 168 - "IpcClient"
Cohesion: 0.06
Nodes (33): Frame, frame, needMore, violation, IpcClient, IpcClientDelegate, IpcMessage, ack (+25 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.08
Nodes (30): GLOBAL_KEY, global_switch_from_tray_disables_every_app(), image_is_textvn_exe(), IpcClient, IpcState, LAST_HOTKEY_TOGGLE_MS, local_toggle_flips_immediately_without_tray(), map() (+22 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "HWND"
Cohesion: 0.18
Nodes (35): HGDIOBJ, apply_dpi_change(), center_on_work_area(), close_macro_editor(), close_word_list_editor(), delete_ui_fonts(), dialog_wnd_proc(), get_chk() (+27 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "create_and_show_window"
Cohesion: 0.23
Nodes (13): create_and_show_window(), create_control(), create_control_ex(), create_dialog_controls(), MACRO_EX_NONE, register_class(), HINSTANCE, system_dpi() (+5 more)

### Community 176 - "generate_tray_icons.py"
Cohesion: 0.18
Nodes (19): _centered_text(), create_us_icon(), create_vn_icon(), _draw_star(), generate_ico(), _letter_font(), main(), FreeTypeFont (+11 more)

### Community 177 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.25
Nodes (7): 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng), Targets JSON — `tools/appcomptest/targets/` (WIN-061)

### Community 178 - "B. macOS"
Cohesion: 0.20
Nodes (10): B10. [THẤP] `IMKTextTarget.swift:40-42` — `NSUnderlineStyle([]).rawValue` = 0, B11. [THẤP] Dead code/state chết: `heldMods` (chỉ ghi), `noteFlagsChanged()`, B13. [THẤP] EnglishWordsSheet filter âm thầm ("Café"→"caf") + không cap 64;, B5. [TRUNG] Nút "Mặc định" xoá macros/english_words/emoji + tắt autostart âm thầm, B6. [TRUNG] Double-click TextVN.app khi autostart bật = im lặng, B7. [TRUNG] gatherAX không dedupe in-flight + không AX messaging timeout, B8. [TRUNG] `IpcServer.appStates` tăng vô hạn qua ToggleViEn app_id tuỳ ý, B9. [THẤP] `clientPid()` bỏ qua `sender`, dùng `NSWorkspace.frontmostApplication` (TextVNInputController.swift:493-495) — panel phụ/XPC focus → appID sai → preset sai. Cải thiện qua sender metadata. (+2 more)

### Community 179 - "check-pe-imports.py"
Cohesion: 0.40
Nodes (4): struct, imports(), main(), check-pe-imports.py — chặn binary Windows phụ thuộc VC++ runtime (R2-18). TIP…

### Community 180 - "invariants.rs"
Cohesion: 0.19
Nodes (16): apply(), backspace_removes_exactly_one_visible_char(), engine_never_overflows_result_or_eats_existing_text(), held_key_after_toned_word_keeps_previous_text(), key_event(), Lcg, letters(), PREFIX (+8 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.32
Nodes (3): Sign-TextVNFile(), Write-Fail(), Write-Ok()

### Community 182 - "AppDelegate"
Cohesion: 0.07
Nodes (24): AppDelegate, .frontAppEnabled, .isVietnameseMode, .vietnameseModeEnabled, Bool, IpcServer, NSMenu, NSObjectProtocol (+16 more)

### Community 183 - "check_no_secrets.py"
Cohesion: 0.14
Nodes (15): Check relative markdown links trong docs/ + *.md goc. Chay local: python…, main(), tracked_iss(), Check API inject / giong malware bi cam (Farch-3). Chay local: python…, is_binary_file(), list_tracked_files(), main(), Path (+7 more)

### Community 184 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 185 - "Engine"
Cohesion: 0.19
Nodes (13): Action, apply(), backspaces(), boundary_space_pass_with_default_caps(), Engine, escape_restores_raw(), Outcome, KeyEvent (+5 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "register_and_activate_tsf"
Cohesion: 0.14
Nodes (16): advice_for_failure(), DIALOG_CTX, normalize_word_list(), read_log_tail(), register_and_activate_tsf(), register_log_path(), register_log_tail_reads_last_lines(), Mutex (+8 more)

### Community 189 - "Foundation"
Cohesion: 0.14
Nodes (15): InjectedMarker, Int64, LayoutCache, .current, Data, Int64, TapMarker, AppKit (+7 more)

### Community 190 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 191 - "IpcServer"
Cohesion: 0.14
Nodes (14): IpcServer, IpcServerDelegate, Any, Bool, Data, Date, DispatchSourceRead, Int (+6 more)

### Community 192 - "ThreadState"
Cohesion: 0.14
Nodes (9): config_file_path(), FieldContext, Into, Option, PathBuf, Self, TextOwnership, TextRange (+1 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - ".new"
Cohesion: 0.32
Nodes (10): broadcast_reaches_subscriber(), frame_encoding_and_read_message_roundtrip(), read_message(), Arc, Result, Self, server_starts_and_stops_cleanly(), serves_every_pipe_name() (+2 more)

### Community 195 - "OwnerRule"
Cohesion: 0.24
Nodes (6): EngineOwner, imk, tap, OwnerRule, Bool, OwnerRuleTests

### Community 196 - "linux_common.h"
Cohesion: 0.07
Nodes (39): lc_detect_active_framework(), lc_env_check(), get_time_ms(), lc_classify_field(), lc_field_detect(), str_contains_icase(), main(), test_env_report_generation() (+31 more)

### Community 197 - "1. Lộ trình ba bước"
Cohesion: 0.22
Nodes (7): 0. Nguyên tắc (khác gì PPA), 1. Lộ trình ba bước, 2. Việc upstream đã chuẩn bị (bằng chứng cho maintainer), Bước A — RFP (nhờ cộng đồng đóng gói) hoặc ITP (tự đóng gói rồi xin sponsor), Bước B — mentors.debian.net upload (kèm ITP), Bước C — Liên hệ Debian Input Method Team (nhanh nhất), Nộp TextVN vào Debian / Ubuntu chính thức (RFP → ITP → Sponsor)

### Community 198 - "cli/src/main.rs"
Cohesion: 0.16
Nodes (14): abi_sizes(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister(), cmd_verify() (+6 more)

### Community 199 - "Security Policy"
Cohesion: 0.50
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (18): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+10 more)

### Community 201 - "DocError"
Cohesion: 0.19
Nodes (10): DocError, persist_keeps_unknown_keys_and_reset_keeps_macros(), DiacriticStyle, FnOnce, MacroEntry, MacroTrigger, Method, OutputCharset (+2 more)

### Community 202 - "`docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)"
Cohesion: 0.25
Nodes (7): 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate, `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

### Community 203 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.17
Nodes (11): 0. Executive Summary, 7.1 Team (tối thiểu → lý tưởng), 7.2 Risk register, 7.3 WBS cấp 2 (sprint-ready), 7. TEAM, RISK & WORK BREAKDOWN, 8. ACCEPTANCE CRITERIA — "hoàn hảo" nghĩa là gì (v1.0), 9.1 Nguồn tham chiếu, 9.2 ADR cần viết trong tuần 1 (+3 more)

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "SettingsView"
Cohesion: 0.10
Nodes (22): ActiveAlert, about, .alert, autostartFailed, .id, EnglishWordsSheet, .body, MacroEditorSheet (+14 more)

### Community 206 - "suggest"
Cohesion: 0.18
Nodes (11): enabled, $comment, default, type, enabled, suggest, additionalProperties, $comment (+3 more)

### Community 207 - "AppDb"
Cohesion: 0.17
Nodes (17): EngineOutcome, is_hook_owned(), KeyEvent, Option, AppDb, b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace() (+9 more)

### Community 208 - "String"
Cohesion: 0.08
Nodes (19): Date, Int32, URL, UninstallOutcome, cancelled, failed, succeeded, String (+11 more)

### Community 209 - "2. PRODUCT OWNER VIEW"
Cohesion: 0.29
Nodes (7): 2.1 Tầm nhìn & USP, 2.2 Personas & Jobs-to-be-done, 2.3 Backlog ưu tiên (MoSCoW), 2.4 "EVKey Spec" — danh sách hành vi phải implement clean-room, 2.5 KPI & Definition of Success, 2.6 Release train, 2. PRODUCT OWNER VIEW

### Community 210 - "e2e_ibus.c"
Cohesion: 0.22
Nodes (20): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+12 more)

### Community 211 - "generate_app_icons.py"
Cohesion: 0.39
Nodes (7): gradient(), main(), pick_font(), Image, Sinh icon PNG ứng dụng TextVN từ thiết kế badge trong `resources/icons/*.svg`.…, Gradient chéo 45° khớp linearGradient 0%,0%→100%,100% (objectBoundingBox)., render()

### Community 212 - "abi_invariants.rs"
Cohesion: 0.18
Nodes (15): cstr, abi_invariants_giu_duoi_input_ngau_nhien(), apply_doc(), assert_sane(), config_paths_fail_open_va_khong_echo_text(), new_result(), Rng, Self (+7 more)

### Community 213 - "Result"
Cohesion: 0.40
Nodes (4): EndCompositionSession_Impl, KeyEditSession_Impl, ITfEditSession_Impl, Result

### Community 214 - "e2e_fcitx5.py"
Cohesion: 0.15
Nodes (14): app_handles(), clear(), ctrl_shift_tap(), press_char(), pump(), Ghi như bảng cài đặt (tmp → rename); chờ để mtime chắc chắn khác lần trước., Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend…, send() (+6 more)

### Community 215 - "AXSnapshot"
Cohesion: 0.17
Nodes (6): AXSnapshot, FieldRole, FieldRules, Bool, UInt32, FieldRulesTests

### Community 216 - "key_event.rs"
Cohesion: 0.18
Nodes (27): is_modifier_vk(), active_modifiers(), defer_to_key_down(), eaten_label(), end_composition(), guarded(), guarded_option(), handle_key() (+19 more)

### Community 217 - "T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007"
Cohesion: 0.33
Nodes (6): LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`, LNX-041 · Callback + self-disable (M) — `P3-3 §3`, LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`, LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`, LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S), T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### Community 218 - "ConfigStore"
Cohesion: 0.11
Nodes (18): ConfigStore, .config, EmojiEntry, LoadError, corrupt, missing, MacroEntry, .id (+10 more)

### Community 225 - "linux-fcitx5/src/engine.h"
Cohesion: 0.20
Nodes (10): Instance, TextVNEngineFactory, create, AddonFactory, AddonInstance, AddonManager, cstdint, InputContextProperty (+2 more)

### Community 226 - "show_dialog_on_startup"
Cohesion: 0.67
Nodes (3): show_dialog_on_startup, default, type

### Community 227 - "ffi/src/settings.rs"
Cohesion: 0.15
Nodes (32): c(), c_api_round_trip(), cstr(), doc_err(), english_words_text_round_trip(), HEADER, ime_settings, IME_SETTINGS_CONFIG (+24 more)

### Community 228 - "Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`)"
Cohesion: 0.33
Nodes (5): Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`), D. ƯU TIÊN XỬ LÝ ĐỀ XUẤT (top 10, ghép 3 platform), E. Ghi chú quy trình, F. Rà lại các bản sửa sau 0.2.31 (2026-10-11) — sửa ở 0.2.32, Tổng quan

### Community 229 - "foreground.rs"
Cohesion: 0.14
Nodes (17): accessibility, foregroundtracker, HWINEVENTHOOK, threading, app_id_from_image_path(), app_of_window(), ForegroundTracker, is_shell_window_class() (+9 more)

### Community 230 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 5. Perf (ngưỡng — `PLAN §5.5`), 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`) (+2 more)

### Community 231 - "AutostartManager"
Cohesion: 0.17
Nodes (7): AutostartManager, Bool, URL, Fixed (macOS — ưu tiên bản này), NSAlert, ServiceManagement, uid_t

### Community 232 - "FieldDetect"
Cohesion: 0.09
Nodes (24): Entry, FieldContext, FieldDetect, Date, pid_t, Set, TimeInterval, AXUIElement (+16 more)

### Community 233 - "Path"
Cohesion: 0.19
Nodes (15): append_log_line(), is_reparse_link(), log_rotates_when_larger_than_limit(), log_writer_checks_reparse_before_creating_dirs(), machine_scope_dll(), prepare_log_file(), program_files_roots(), resolve_dll_path() (+7 more)

### Community 234 - "TapPermission"
Cohesion: 0.20
Nodes (8): NSLock, Bool, T, TimeInterval, Void, TapPermission, ApplicationServices, Timer

### Community 235 - "OutputStrategy"
Cohesion: 0.29
Nodes (7): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, Int64

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.25
Nodes (8): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor, Sự cố đã gặp khi phát hành — tra trước khi xử lý

### Community 237 - "4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)"
Cohesion: 0.33
Nodes (6): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), Phát hiện M-01: Không đồng bộ giao diện Menu Bar khi nhận IPC, Phát hiện M-02: Quyền Hardened Runtime (Entitlements) thừa thãi, Phát hiện M-03: Thử nghiệm kiểm thử CI thiếu kiến trúc x86_64, Phát hiện M-04: Thử nghiệm thực tế trên phần cứng macOS vật lý (Real Hardware & TCC Permissions Gap), Phát hiện M-05: Cấu hình `non_preedit` chưa liên kết vào IMK (Gõ chữ bị gạch chân)

### Community 238 - "Hướng dẫn sử dụng TextVN"
Cohesion: 0.15
Nodes (13): 1. Cài đặt, 2. Bật/tắt tiếng Việt, 3. Kiểu gõ, 4. Bảng điều khiển, 5. Gõ tắt, 6. Cấu hình, 7. Xử lý sự cố, Cài đặt im lặng (cho quản trị / Microsoft Store) (+5 more)

### Community 239 - "keymap_mac.rs"
Cohesion: 0.20
Nodes (4): MAC_KEY_COUNT, mac_to_canonical(), Option, modifier_dung_vk_is_modifier_nhan_duoc()

### Community 240 - "quick_telex.rs"
Cohesion: 0.24
Nodes (6): apply, apply(), expansion(), q(), Option, Vec

### Community 241 - "3. SOLUTION ARCHITECT VIEW"
Cohesion: 0.25
Nodes (8): 3.1 C4 — Context, 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*, 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**, 3.4 Output Strategy Engine (trái tim của app-compat), 3.5 Config & state, 3.6 UI/Tray (1 codebase, 3 native shell), 3.7 Packaging & Distribution, 3. SOLUTION ARCHITECT VIEW

### Community 242 - "Hướng dẫn phát triển TextVN"
Cohesion: 0.20
Nodes (10): 1. Kiến trúc, 2. Dựng, 3. Kiểm thử, 4. Quy ước bắt buộc (CI chặn), 5. Phát hành, 6. Tài liệu liên quan, Hướng dẫn phát triển TextVN, Linux (Ubuntu/Debian) (+2 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.09
Nodes (23): 1. Gói phát hành, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Báo cáo dựng & kiểm thử — TextVN, Bản 0.2.1 — phát hành pre-release đã xác minh, Bản 0.2.20 — CI xanh lại + validation vòng 3 (harness đúng như Microsoft) (+15 more)

### Community 244 - "remove_added_languages"
Cohesion: 0.18
Nodes (18): added_language_removable(), broadcast_international(), ctrl_shift_taken_by_windows(), is_input_method_value(), languages_added_marker_path(), languages_missing(), languages_without(), parse_languages_marker() (+10 more)

### Community 245 - "A. WINDOWS"
Cohesion: 0.12
Nodes (17): A10. [TRUNG] COM callback `ActivateEx`/`Deactivate`/`OnSetFocus` không bọc `catch_unwind` (tip.rs:90-124,174-183) — panic thoát qua ABI = abort host (Notepad/Chrome). Nhánh `try_borrow_mut` fail tại ActivateEx: Advise đã thực hiện nhưng cookie không lưu → Deactivate không Unadvise (leak tới hết thread). **Fix:** catch_unwind như `DoEditSession`; borrow-fail trả `Err(E_FAIL)`., A11. [TRUNG] `svc.rs:135-235` — `RwLock.write().unwrap()`: poison → panic dây chuyền trong `wnd_proc`. **Fix:** `unwrap_or_else(|e| e.into_inner())`., A12. [TRUNG] Uninstaller per-user non-elevated: `schedule-delete` (PFO) luôn ACCESS_DENIED → `.old-*` sót vĩnh viễn (register.rs:793-834). **Fix:** RunOnce như uninstall.ps1 portable đã làm., A13. [THẤP] Đa user với bản máy: user B còn per-user override trỏ DLL đã xoá; CLI cũng bị xoá → không tự lành (main.rs:601-603 return sớm). Fix dài hạn: marker machine-level yêu cầu tray user B tự unregister khi thấy path chết., A14. [THẤP] `reg_value_names` buffer 512 u16 — value dài hơn 511 ký tự, A15. [THẤP] `show_activation_refused_notice_once` (main.rs:337-351) ghi marker, A1. [CAO] ILOT spec cho bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100) 🔎, A2. [CAO] Mirror WOW64 CTF cho **HKCU** là đường registry chết 🔎 (+9 more)

### Community 246 - "fuzz — L4 (P0-4 §1)"
Cohesion: 0.33
Nodes (5): Bất biến mỗi target assert (không chỉ "không panic"), Chạy, fuzz — L4 (P0-4 §1), Kiểm chứng trên máy không có cargo-fuzz, Seed corpus

### Community 247 - ".inject"
Cohesion: 0.50
Nodes (5): Bool, CGEventSource, Int, UInt16, TapInjector

### Community 248 - "RoundTwoMacAppTests"
Cohesion: 0.18
Nodes (6): Int, RoundTwoMacAppTests, URL, Cocoa, TextVNAppLib, XCTest

### Community 249 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.29
Nodes (7): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust, Phát hiện W-06: Điểm mù kiến trúc 32-bit WOW64 — Zalo PC và toàn bộ ứng dụng x86 không thể nạp TSF DLL 64-bit

### Community 250 - "smoke-imk.sh"
Cohesion: 0.48
Nodes (5): clear_doc(), read_doc(), say(), smoke-imk.sh script, type_keys()

### Community 251 - "textvn_ffi"
Cohesion: 0.08
Nodes (21): ime_instance, ime_result, abi_version, action, delete_count, flags, insert, insert_len (+13 more)

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
Cohesion: 0.15
Nodes (11): ime_key_v1, STRATEGY_MAX_ID, apply_doc(), assert_result_sane(), ByteCursor, MAX_KEYS, VK_BACK, VK_RETURN (+3 more)

### Community 258 - "Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)"
Cohesion: 0.25
Nodes (8): 1. Bố cục, 2. Tuỳ chọn ↔ khoá cấu hình, 3. Hành vi chung, 4. Cửa sổ Gõ tắt, 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+), 6. Nhận diện EN/VI (không có UI riêng — hành vi engine), 7. Icon — một thiết kế cho 3 nền tảng, Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)

### Community 259 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 260 - "tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)"
Cohesion: 0.40
Nodes (4): 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật (cập nhật 2026-10-09), tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

### Community 262 - "Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit"
Cohesion: 0.29
Nodes (7): Audit tiếp diễn — TextVN, 2026-09-30, Giới hạn và rủi ro còn mở, Lệnh tái lập, Perf — job `perf regression` vẫn đỏ trên CI, **không phải** hồi quy, Ranh giới xác minh, Vòng 10 — delta từ `v0.2.0` đến `v0.2.1`, Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit

### Community 263 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.15
Nodes (13): 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release, Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF (+5 more)

### Community 264 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - "Trạng thái ký số macOS — Developer ID, notarization, stapling"
Cohesion: 0.40
Nodes (5): 1. Tóm tắt, 2. Ảnh hưởng tới người dùng, 3. Tooling đã nối (chỉ chạy khi có secret), 4. Việc còn lại để bật (chủ repo), Trạng thái ký số macOS — Developer ID, notarization, stapling

### Community 269 - "[0.2.32] — 2026-10-12"
Cohesion: 0.40
Nodes (5): [0.2.32] — 2026-10-12, Changed, Fixed, Known issues, Security / quy trình

### Community 270 - "4. ENGINEER VIEW"
Cohesion: 0.40
Nodes (5): 4.1 Ngôn ngữ & технологies (quyết định), 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh), 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu), 4.4 Quy trình, 4. ENGINEER VIEW

### Community 274 - "EngineOptions"
Cohesion: 0.14
Nodes (10): Context, disabled_all_pass(), EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset (+2 more)

### Community 291 - "ConfigWatcher"
Cohesion: 0.23
Nodes (7): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem

### Community 292 - "path"
Cohesion: 0.18
Nodes (11): all_cases, fs, path, find_rc_exe(), main(), Option, PathBuf, Result (+3 more)

### Community 293 - "6. AUDIT EXPERT VIEW"
Cohesion: 0.40
Nodes (5): 6.1 Threat model (tóm tắt STRIDE cho IME), 6.2 License & compliance plan, 6.3 Security/quality gates (mọi PR phải qua), 6.4 External audit (trước v1.0), 6. AUDIT EXPERT VIEW

### Community 294 - "command"
Cohesion: 0.15
Nodes (14): find_rc_exe(), main(), Option, PathBuf, find_rc_exe(), main(), Option, PathBuf (+6 more)

### Community 295 - "check_version_sync.rs"
Cohesion: 0.22
Nodes (12): check_site(), extract(), PLIST_KEY, repo_hien_tai_dong_bo(), Option, Result, run(), Site (+4 more)

### Community 297 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 298 - "tv_win_free"
Cohesion: 0.67
Nodes (3): gpointer, tv_win_free(), C7. [THẤP] GTK settings (settings_window.c:483,214-250) — `TvWin` free

### Community 299 - "[0.2.0] — 2026-09-30"
Cohesion: 0.22
Nodes (9): [0.2.0] — 2026-09-30, Added, Added (macOS — Farch-4), Added (trước đó), Changed, Fixed, Fixed, Fixed (review vòng 3 — trước tag v0.2.0) (+1 more)

### Community 301 - ".new"
Cohesion: 0.52
Nodes (5): KeySink, KeyTraceSink, ObjGuard, Rc, Self

### Community 302 - "TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux"
Cohesion: 0.29
Nodes (7): Cấu trúc, Dựng từ mã nguồn, Giấy phép, TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux, Trạng thái, Tải và cài, Đóng góp

### Community 303 - "[0.2.13] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.13] — 2026-10-03, Added, Documented, Fixed

### Community 304 - "3. Nhật ký finding"
Cohesion: 0.12
Nodes (15): 0. Kết luận, 1. Phương pháp, 2. Bằng chứng CI, 3.1 Microsoft Store / MSIX, 3.2 Bảo mật Windows, 3.3 QA Windows (tray, CLI, bộ cài), 3.5 Tiếng Việt / engine, 3.6 CI, ký số, phát hành (+7 more)

### Community 305 - "[0.2.14] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.14] — 2026-10-03, Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau"), Fixed (Windows), Tests

### Community 306 - "[0.2.9] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.9] — 2026-10-02, Added, Documented, Tests

### Community 307 - "verify-store-package.sh"
Cohesion: 0.60
Nodes (4): die(), GNUPGHOME, verify-store-package.sh script, valid_sig()

### Community 308 - "Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)"
Cohesion: 0.29
Nodes (7): Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon), Icon — trả lời câu hỏi "Linux/macOS đã có icon như Windows chưa?", Kiểm chứng chạy trên host Windows (2026-10-01, sau toàn bộ thay đổi), Phát hành — v0.2.3 đã publish (release candidate, chưa ký số), Quy trình phát hành mới, Sửa theo audit vòng 1 — GitHub Actions (5 major + minors), Sửa theo audit vòng 1 — đăng ký TSF (3 major + 7 minor)

### Community 309 - "CLIENT"
Cohesion: 0.67
Nodes (3): CLIENT, IpcClient, OnceLock

### Community 310 - "[0.2.10] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.10] — 2026-10-03, Documented, Fixed, Tests

### Community 312 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.14
Nodes (12): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG, 6. PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT) (+4 more)

### Community 313 - "Added"
Cohesion: 0.33
Nodes (6): [0.1.0] — 2026-09-27, Added, Architecture, Core, Developer Tools, Windows Platform

### Community 315 - "wide"
Cohesion: 0.19
Nodes (20): delete_reg_value(), delete_tree(), enum_subkeys(), hkey_users_relative_paths_reach_the_loaded_hive(), is_service_profile_sid(), profile_metadata_ok(), reg_key_exists(), reg_read_dword() (+12 more)

### Community 316 - "[0.2.3] — 2026-10-01"
Cohesion: 0.33
Nodes (6): [0.2.3] — 2026-10-01, Added, Changed (CI — audit GitHub Actions, 5 major), Fixed (Windows), Fixed (đăng ký TSF — audit vòng sâu, 3 major), Housekeeping

### Community 317 - "Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit"
Cohesion: 0.33
Nodes (6): Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit, Bổ sung sau commit `ec1ac2b` — triệt để hóa gợi ý lỗi đăng ký TSF (ảnh lỗi của người dùng), Kiểm chứng chạy trên host Windows (trước commit), Perf — job `perf regression` đỏ trên CI là **nhiễu runner**, không phải hồi quy, Phát hành — v0.2.2 đã publish (release candidate, chưa ký số), Trạng thái

### Community 318 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.15
Nodes (13): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG (+5 more)

### Community 319 - "[0.2.33] — 2026-10-18"
Cohesion: 0.50
Nodes (4): [0.2.33] — 2026-10-18, Added, Changed, Fixed

### Community 320 - "2. Kết quả kiểm thử"
Cohesion: 0.50
Nodes (4): 2. Kết quả kiểm thử, Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng), Kịch bản 1 — Cài đặt, Kịch bản 2 — Giải nén ra dùng luôn

### Community 321 - "Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening"
Cohesion: 0.50
Nodes (4): Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening, Nguyên nhân + fix + bằng chứng, Phát hành — v0.2.10 & v0.2.11 đã publish, Sự cố phát hành mới (R4) — publish fail im lặng

### Community 322 - "Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)"
Cohesion: 0.05
Nodes (36): 0. Trạng thái — ĐÃ PHÁT HÀNH trên Microsoft Store (2026-10-11), 1. Vì sao gói MSIX của TextVN có cơ chế "stage-out", 2. Partner Center — việc của chủ tài khoản, 3. Build gói nộp Store, 4. Nộp submission, 5. Rủi ro còn lại & cách xử lý, 6. Sau khi publish, 7.1 Workflow làm gì (+28 more)

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

### Community 330 - ".log"
Cohesion: 0.08
Nodes (15): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, UInt64, IMKTextTarget (+7 more)

### Community 331 - "5. TECHNICAL EXPERT VIEW"
Cohesion: 0.33
Nodes (6): 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**), 5.2 Env compatibility (được yêu cầu đặc biệt), 5.3 Browser compatibility matrix (mục tiêu v1.0), 5.4 App compat matrix (40 app, giữ trong `docs/compat.md`, CI chạy subset), 5.5 Performance budget, 5. TECHNICAL EXPERT VIEW

### Community 332 - "Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất"
Cohesion: 0.50
Nodes (4): 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển, 0.2.14 — DPI gốc + UI thống nhất + common errors, Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất, Phát hành + xác minh máy chủ repo

### Community 333 - "vowel_table.rs"
Cohesion: 0.12
Nodes (22): Horn, strip_tone(), chars(), is_nucleus(), mark_cluster(), mark_horn(), mark_horn_pair_and_single(), mark_vowel() (+14 more)

### Community 334 - "verify-msix.py"
Cohesion: 0.21
Nodes (10): argparse, os, check(), main(), map_version(), png_size(), verify-msix.py — kiểm gói .msix TRƯỚC khi nộp Partner Center (R2-09). makeappx…, 0.2.27 → 1.2.27.0 — phải khớp tools/win/build-msix.ps1 (R2-01). (+2 more)

### Community 335 - "diacritic_style"
Cohesion: 0.50
Nodes (4): $comment, default, enum, diacritic_style

### Community 336 - "[0.2.16] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.16] — 2026-10-03, Fixed (Windows) — portable "giải nén chạy ngay" hoạt động trên Win11 24H2+, Verified live trên máy thật (máy chủ repo, build 26300 — trường hợp từng fail)

### Community 337 - "do_register"
Cohesion: 0.15
Nodes (26): activate_for_session(), activate_tip(), active_profile_is_textvn(), com_init(), ctf_category_keys(), ctf_layout_matches_tsf_registry_schema(), ctf_profile_key(), ctf_tip_key() (+18 more)

### Community 338 - "3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft"
Cohesion: 0.12
Nodes (17): 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU), 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?, 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft, 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối, 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store, 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation, 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?, 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft (+9 more)

### Community 339 - "DialogContext"
Cohesion: 0.67
Nodes (4): DialogContext, Arc, IpcServer, show_settings_dialog()

### Community 340 - "[0.2.12] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.12] — 2026-10-03, Fixed (Windows), Known issue (ghi nhận từ máy chủ repo, 0.2.12 chưa xử lý được)

### Community 341 - "[0.2.15] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.15] — 2026-10-03, Fixed (Windows), Verified — toàn bộ luồng portable "chạy ngay" (yêu cầu chủ repo)

### Community 342 - "[0.2.29] — 2026-10-10"
Cohesion: 0.67
Nodes (3): [0.2.29] — 2026-10-10, Changed, Fixed — gỡ cài đặt sạch (R2-99, R2-100)

### Community 343 - "[0.2.20] — 2026-10-04"
Cohesion: 0.67
Nodes (3): [0.2.20] — 2026-10-04, Fixed (Windows) — CI đỏ .iss + vòng 3 validation (B13/B14), Security / hardening (audit 2026-10-04, phần còn lại)

### Community 344 - "Option"
Cohesion: 0.18
Nodes (15): MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, PWSTR, current_package_family_name(), current_package_full_name(), message_box(), open_apps_settings(), package_family_full_names() (+7 more)

### Community 345 - "macro_trigger"
Cohesion: 0.67
Nodes (3): default, enum, macro_trigger

### Community 346 - "3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux"
Cohesion: 0.33
Nodes (6): 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn

### Community 348 - "3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 349 - "Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store"
Cohesion: 0.17
Nodes (12): 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào, 2. Vòng 15 — những gì đã triển khai trong `release.yml`, 3. Khoá ký GPG release (TextVN Release Signing), 4. Authenticode cho Microsoft Store (chưa xong — đang chờ), 4b. macOS — Developer ID + notarization, 5. Quy tắc khóa-tay (đừng lặp), 6.1. Sự cố & Nguyên nhân gốc rễ (Sự cố R6 — 2026-10-11), 6.2. Quy tắc bắt buộc cho mọi release từ v0.2.32 (+4 more)

### Community 350 - "keys_generated.rs"
Cohesion: 0.19
Nodes (12): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+4 more)

### Community 352 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 354 - "Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI"
Cohesion: 0.25
Nodes (7): 1. Microsoft nói chính xác điều gì, 2. Điều này thay đổi nhận định gì, 3. Ba con đường (đối chiếu chi phí / rủi ro / thời gian), 4. Kế hoạch chốt (theo thứ tự ưu tiên — cập nhật 2026-10-09), 5. Hiện trạng kỹ thuật đã sẵn sàng (2026-10-09), 6. Việc cần làm — checklist, Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

### Community 355 - "caps.rs"
Cohesion: 0.31
Nodes (4): is_closer(), is_sentence_end(), sentence_start(), starts()

### Community 357 - "3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM""
Cohesion: 0.29
Nodes (7): 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store, 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM", 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM", 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant), 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên, Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`, Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC

### Community 358 - "StageInfo"
Cohesion: 0.29
Nodes (6): stage_json_roundtrip_and_validation(), StageInfo, launch_package_app(), Result, spawn_detached(), write_stage()

### Community 359 - "ime_context"
Cohesion: 0.08
Nodes (24): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+16 more)

### Community 360 - "TextVNAppTests"
Cohesion: 0.09
Nodes (8): Data, TextVNAppTests, 0. Đọc trước khi làm bất cứ việc gì (trình tự), 1. Luật không được phá (vi phạm = task chưa xong), 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự, 3. Kiến trúc & ràng buộc kỹ thuật cốt lõi, AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN, Bản 0.2.33 — mặc định dấu cũ sau cài + ma trận 16 combo + từ điển EN top-3000 + Tab hoàn tất từ tiếng Việt

### Community 361 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.33
Nodes (6): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust

### Community 362 - "TapDecision"
Cohesion: 0.12
Nodes (13): Int, TapDecision, pass, transform, CGEventSource, UInt32, TapMarkerTests, Còn mở (2026-10-09) (+5 more)

### Community 363 - "3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)"
Cohesion: 0.33
Nodes (6): 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu, 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions), 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions), 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết, Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE), Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)

### Community 366 - "[0.2.28] — 2026-10-10"
Cohesion: 0.40
Nodes (5): [0.2.28] — 2026-10-10, Added, Fixed — kiểm trên Windows 11 thật và rà soát đối kháng (R2-88…R2-98, chi tiết: `docs/release/code-review-2026-10-09-round2.md` §3.7), Fixed — rà soát mã nguồn chuyên sâu 2026-10-07 (chi tiết: `docs/release/code-review-2026-10-07.md`), Fixed — rà soát vòng 2: Microsoft Store, ký số, QA (chi tiết: `docs/release/code-review-2026-10-09-round2.md`)

### Community 367 - "GlobalHookContext"
Cohesion: 0.16
Nodes (11): HookEngine, Drop, ime_instance, Result, Self, GlobalHookContext, HWND, Instant (+3 more)

### Community 370 - "C. LINUX + SHARED"
Cohesion: 0.17
Nodes (11): lc_utf32_to_utf8(), lc_utf8_to_utf32(), C2. [TRUNG] Parser IPC dispatch bằng `strstr` toàn JSON → nhầm nhánh khi, C3. [TRUNG — kiến trúc] IPC Linux là dead path: không tồn tại server, C4. [TRUNG — latent] Override per-app "dính" sang app kế tiếp (ibus, C5. [THẤP] compose.c:92-94 — `cur->len == LC_COMP_MAX` → ghi, C6. [THẤP] utf.c:49-106 — `lc_utf8_to_utf32` không chặn overlong, C8. [THẤP] `tv_activate_fcitx5` (textvn-common.sh:355-363) thêm TextVN (+3 more)

### Community 371 - "[0.2.4] — 2026-10-01"
Cohesion: 0.67
Nodes (3): [0.2.4] — 2026-10-01, Fixed, Verification

### Community 373 - "linux-settings/src/main.c"
Cohesion: 0.33
Nodes (4): gpointer, GtkApplication, on_app_activate(), gtk

### Community 374 - "ApplyError"
Cohesion: 0.29
Nodes (7): ApplyError, cannotDelete, rejected, KeyTranslatorError, noChar, noLayout, Error

### Community 377 - "Phân phối Linux — kênh, store và auto-update"
Cohesion: 0.17
Nodes (12): 1. Ràng buộc quyết định kênh: TextVN là IME hệ thống, 2. Đánh giá từng kênh (audit), 3. AUR — nội dung đã chuẩn bị, 4. Ma trận auto-update sau release (mục tiêu "release xong store tự có bản mới"), 4a. PPA Launchpad — các bước của chủ repo (một lần) + mỗi bản, 4b. Debian/Ubuntu chính thức — lộ trình sponsor, 5. QA đã chạy (bằng chứng), a) Một lần (~15 phút, owner thực hiện trên trình duyệt + Ubuntu/WSL) (+4 more)

### Community 378 - "[0.2.26] — 2026-10-05"
Cohesion: 0.67
Nodes (3): [0.2.26] — 2026-10-05, Added (Windows) — hỗ trợ app 32-bit (Zalo/Office x86) — vòng 14, Fixed (Windows) — gỡ cài sạch 100% key layout (vòng 13, bắt thật + dọn live trên máy chủ repo)

### Community 381 - "Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)"
Cohesion: 0.67
Nodes (3): 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

### Community 384 - "TextVN — Chỉ mục tài liệu & Giao thức Review"
Cohesion: 0.29
Nodes (7): 0. Tài liệu cho người dùng & phát hành, 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 385 - "cli/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 386 - "on_dir_changed"
Cohesion: 0.40
Nodes (6): gboolean, is_settings_file(), on_dir_changed(), GFile, GFileMonitor, GFileMonitorEvent

### Community 397 - "super"
Cohesion: 0.12
Nodes (11): fold(), n(), DiacriticStyle, Vec, complete_vn_word(), Option, v(), VN_FREQ_DATA (+3 more)

### Community 399 - "Problem"
Cohesion: 0.12
Nodes (15): MacroRules, Problem, emptyExpansion, emptyTrigger, expansionTooLong, .message, triggerHasSpace, triggerTooLong (+7 more)

### Community 402 - "[0.2.34] — 2026-10-18"
Cohesion: 0.50
Nodes (4): [0.2.34] — 2026-10-18, Added, Docs, Fixed

### Community 404 - "P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên Windows, 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`), 3. App preset mặc định — `data/appdb.default.json` (phần Windows), 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4), 5. Tích hợp với adapter — API nội bộ chung, 6. Test & bằng chứng, 7. Mapping task, P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

### Community 406 - "ModifierToggle"
Cohesion: 0.43
Nodes (4): ctrl_shift_tap_toggles_once_and_only_without_other_keys(), is_ctrl_vk(), is_shift_vk(), ModifierToggle

### Community 409 - "[0.2.8] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.8] — 2026-10-02, Added, Changed, Fixed

## Knowledge Gaps
- **1557 isolated node(s):** `ic`, `inst`, `comp`, `config`, `toggle` (+1552 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 2434 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **72 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `2. Nhật ký finding` connect `ipc_client.c` to `OwnerRule`, `TextVNAppTests`, `IpcClient`, `TextVNInputController`, `Problem`, `MarkedState`, `KeyOutcome`, `ConfigStore`, `.inject`, `EventTapController`, `IpcServer`?**
  _High betweenness centrality (0.149) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `Problem`, `ConfigWatcher`, `IpcClient`, `AppDelegate`, `StrategyResolver`, `Foundation`, `IpcServer`, `OwnerRule`, `.log`, `SettingsView`, `TextVNInputController`, `P2-REVIEW-LOG — Phần 2 (macOS)`, `KeyTranslator`, `AXSnapshot`, `XCTestCase`, `ConfigStore`, `.inject`, `AutostartManager`, `TextVNAppTests`, `FieldDetect`, `TapDecision`, `MarkedState`, `KeyOutcome`, `RoundTwoMacAppTests`, `EventTapController`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **Why does `lc_ipc_client_toggle_vi_en()` connect `ipc_client.c` to `engine.c`, `engine.cpp`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `AppDelegate` (e.g. with `B. Trên macOS: Quên cập nhật Icon Menu Bar & Luồng giao diện (UI Thread Violation)` and `(A) macOS app / IMK / packaging / CI — `F3-*``) actually correct?**
  _`AppDelegate` has 4 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `TextVNInputController` (e.g. with `KeyTranslator` and `MarkedState`) actually correct?**
  _`TextVNInputController` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ic`, `inst`, `comp` to the rest of the system?**
  _1557 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `core/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12753623188405797 - nodes in this community are weakly interconnected._