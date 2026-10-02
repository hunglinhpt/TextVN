# Graph Report - TextVN  (2026-10-02)

## Corpus Check
- 313 files · ~319,963 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 267 file(s) not represented in the graph (top: .keys 227, .toml 9, .rc 5)

## Summary
- 4742 nodes · 8778 edges · 316 communities (261 shown, 55 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 529 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f9d52117`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- core/src/lib.rs
- tsf-min/src/lib.rs
- cli/src/replay.rs
- config/src/lib.rs
- text
- verify.rs
- windows-hook/src/main.rs
- toml.rs
- xtask/src/main.rs
- ffi/src/lib.rs
- strategy/src/lib.rs
- vowel_table.rs
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
- TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux
- svc.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- AppDb
- windows-hook/src/lib.rs
- Chi tiết từng mục
- tip.rs
- super
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
- test_field_detect.c
- P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết
- rules_win.rs
- T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004
- settings_dialog.rs
- stdio
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- config_paths_fail_open_va_khong_echo_text
- textvn-ffi
- items
- compose.rs
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- lc_config_sync
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)
- P3-0 — LINUX MASTER PLAN (Phần 3)
- TextVNInputController
- undo.rs
- settings_window.c
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- T6 — Test & hardening (tuần 15–18)
- KeyTranslator
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
- .new
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- ObjGuard
- ipc_server.rs
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- ByteCursor<'a>
- Json
- when
- P1 — REVIEW LOG (Phần 1: Windows)
- TextVN IPC v1
- entry
- id
- auto_capitalize
- auto_restore_english
- MarkedState
- english_words
- P1-6 — TASKS Windows (WBS) — Nhận việc từng task
- P2-6 — TASKS macOS (WBS) — Nhận việc từng task
- ImeEngine
- config_version
- free_marking
- IpcClient
- windows
- adr/README.md
- tsf-min
- EngineSession
- xtask
- WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"
- TsfShared
- Strategy
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- encode_frame
- EventTapController
- bool
- P2-REVIEW-LOG — Phần 2 (macOS)
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- common_words.rs
- guids.rs
- uia_spike.ps1
- edit_session.rs
- P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow
- Project Common Errors — Sổ lỗi chung toàn dự án
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- compose.c
- ipc_client.c
- tray/src/lib.rs
- P0 — REVIEW LOG (Phần 0: Nền tảng chung)
- doc.rs
- keys_generated.rs
- tsf-min/src/register.rs
- win32-uia.lib.ps1
- trace.rs
- engine.cpp
- test_compose.c
- ipc_client.rs
- bitset
- verify_targets.ps1
- HWND
- Performance Audit — TextVN Windows Platform
- create_and_show_window
- generate_tray_icons.py
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- TrayApp
- struct
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- build-release.ps1
- AppDelegate
- check_no_injection_apis.py
- DialogContext
- Engine
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- register_and_activate_tsf
- Foundation
- .log
- String
- ime_result
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- windows-tsf/src/replay.rs
- OwnerRule
- cli/src/main.rs
- ime_context
- core
- Security Policy
- Đóng góp cho TextVN
- SvcManager
- 2. PRODUCT OWNER VIEW
- 3. SOLUTION ARCHITECT VIEW
- macro_text.rs
- SettingsView
- TextVNState
- linux-fcitx5/src/engine.h
- fuzz — L4 (P0-4 §1)
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- e2e_ibus.c
- generate_app_icons.py
- ime_key
- Result
- e2e_fcitx5.py
- AXSnapshot
- .OnTestKeyDown
- IpcMessage
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
- NSRange
- foreground.rs
- preset_matrix.rs
- AutostartManager
- FieldContext
- resolve.rs
- TapPermission
- diacritic_style.rs
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- Added
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- quick_telex.rs
- ActionKind
- Hướng dẫn phát triển TextVN
- Báo cáo dựng & kiểm thử — TextVN
- mac_corpus_cases.rs
- P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết
- ime_key
- .inject
- TextVNAppTests
- Option
- smoke-imk.sh
- AppDelegate.swift
- Kế hoạch ký số (code signing) — TextVN
- ApplyError
- check_version_sync.rs
- test-typing.ps1
- env-mac — Môi trường build macOS (MAC-001)
- icons.rs
- Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux)
- TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV
- tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)
- PackageDescription
- Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)
- .heartbeatSummary
- quick_telex
- e2e-linux.sh
- mem-check.sh
- soak.sh
- log.c
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
- OutputStrategy
- command
- ime_context
- 5. TECHNICAL EXPERT VIEW
- ime_result
- .value
- HookEngine
- classify
- key_event.rs
- Audit tiếp diễn — TextVN, 2026-09-30
- handleKey
- textvn_ffi
- .CreateInstance
- ime_suggest
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- [0.2.0] — 2026-09-30
- pre_translate_message
- Word
- class.rs
- IpcCodecTests
- cli/build.rs
- Frame

## God Nodes (most connected - your core abstractions)
1. `AppDelegate` - 49 edges
2. `SvcManager` - 45 edges
3. `MarkedState` - 37 edges
4. `TextVNInputController` - 37 edges
5. `TsfShared` - 37 edges
6. `TextVNEngine` - 32 edges
7. `SettingsController` - 32 edges
8. `IpcServer` - 31 edges
9. `Engine` - 29 edges
10. `TextVNAppTests` - 28 edges

## Surprising Connections (you probably didn't know these)
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `2. Cấu trúc file (bám đúng `P0-1 §1`)` --references--> `expect()`  [INFERRED]
  docs/20-windows/P1-1-tsf.md → adapters/linux-common/tests/test_compose.c
- `5. Changelog của chỉ mục` --references--> `text()`  [INFERRED]
  docs/00-INDEX.md → adapters/linux-fcitx5/tests/e2e_fcitx5.py
- `9. Spike checklist (tasks `LNX-002/003/004/005/008` — tuần 1–2, chặn WS1)` --references--> `commit_text()`  [INFERRED]
  docs/40-linux/P3-1-ibus.md → adapters/linux-ibus/src/engine.c

## Import Cycles
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`

## Communities (316 total, 55 thin omitted)

### Community 0 - "core/src/lib.rs"
Cohesion: 0.11
Nodes (31): ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, apply(), auto_capitalize_after_dot_and_enter(), auto_capitalize_only_first_letter_of_word(), backspace_after_sentence_end_cancels_auto_capitalize() (+23 more)

### Community 1 - "tsf-min/src/lib.rs"
Cohesion: 0.06
Nodes (50): ITfKeyEventSink, ClassFactory, ClassFactory_Impl, CLSID_TEXTVN_TIP, CompSink, CompSink_Impl, DllCanUnloadNow(), DllGetClassObject() (+42 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.08
Nodes (46): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, Cmd, collect_files() (+38 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.08
Nodes (28): Config, ConfigError, default_json(), defaults_match_p0_3_table(), DiacriticStyle, EmojiEntry, FULL_EXAMPLE, full_example_parses() (+20 more)

### Community 4 - "text"
Cohesion: 0.13
Nodes (15): check(), text(), 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate (+7 more)

### Community 5 - "verify.rs"
Cohesion: 0.11
Nodes (32): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+24 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (41): caps_lock_bit(), CTRL_DOWN, empty_ime_result(), FocusProbe, get_active_modifiers(), get_process_name_for_window(), GLOBAL_ENABLED, GlobalHookContext (+33 more)

### Community 7 - "toml.rs"
Cohesion: 0.15
Nodes (18): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+10 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.11
Nodes (37): banner(), emit(), format_rust(), gen_tables(), index_of(), key_char(), KEYMAP_MAC_OUT_RUST, KEYMAP_MAC_OUT_SWIFT (+29 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.08
Nodes (42): fill_result(), flow_new_key_reset_free(), IME_ABI_VERSION, ime_context_v1, IME_ERR_ABI, IME_ERR_CONFIG, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG (+34 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.09
Nodes (20): IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO (+12 more)

### Community 11 - "vowel_table.rs"
Cohesion: 0.08
Nodes (20): base_entry(), form(), A, A_BREVE, A_CIRC, E, E_CIRC, I (+12 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.12
Nodes (27): black_box, F, instant, textvn_field_detect, bench_field_switch_resolve(), bench_ime_key(), bench_parse_config(), bench_resolve() (+19 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.06
Nodes (70): authorization, activate_for_session(), call_layout_or_tip(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, clsid_key() (+62 more)

### Community 14 - "tone.rs"
Cohesion: 0.19
Nodes (22): fix_uo(), Horn, post_fixes(), push_key(), apply_key(), apply_key_literal_when_no_vowel(), apply_key_tone_and_undo(), apply_tone() (+14 more)

### Community 15 - "keymap.rs"
Cohesion: 0.06
Nodes (26): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, ESCAPE, F1, KeyEvent (+18 more)

### Community 16 - "vni.rs"
Cohesion: 0.09
Nodes (23): apply_key, fold(), marker_to_tone(), n(), push_key(), DiacriticStyle, Option, Vec (+15 more)

### Community 17 - "validate.rs"
Cohesion: 0.15
Nodes (17): CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI, ONSETS, qu_nucleus_starts_after_u() (+9 more)

### Community 18 - "macro.rs"
Cohesion: 0.15
Nodes (14): Emoji, emoji_glyph_is_multi_char(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches(), MacroDef (+6 more)

### Community 19 - "telex.rs"
Cohesion: 0.11
Nodes (14): circum_pair(), f(), fold(), fold_with(), fold_with_caps(), has_uo_pair(), n(), qu_and_open_uo() (+6 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.06
Nodes (42): after_request_edit_session(), classify_tsf_field(), config_file_path(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, external_edit_clears_ownership_before_next_replace(), focus_generation_rejects_old_uia_result_and_stays_fail_safe() (+34 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.13
Nodes (14): canh_bao_cai_muc_mo_ho(), chars(), danh_sach_khong_pha_tieng_viet_that(), data_stop_en_file_is_self_consistent(), default_english_words(), listed(), parse_word_list(), Vec (+6 more)

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
Cohesion: 0.17
Nodes (22): write_as, APP_DB, appdb(), CI_APPS, collect_preset_ids(), FIELD_ROLES, good(), has_preset_id() (+14 more)

### Community 27 - "engine.c"
Cohesion: 0.13
Nodes (36): lc_log(), gboolean, gchar, guint, IBusText, commit_text(), engine_disable(), engine_focus_in() (+28 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.12
Nodes (16): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+8 more)

### Community 29 - "TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux"
Cohesion: 0.29
Nodes (7): Cấu trúc, Dựng từ mã nguồn, Giấy phép, TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux, Trạng thái, Tải và cài, Đóng góp

### Community 30 - "svc.rs"
Cohesion: 0.12
Nodes (14): atomic_write_file(), corrupt_config_is_backed_up_not_lost(), default_config_dir(), failed_config_save_keeps_memory_and_version_unchanged(), failed_state_save_keeps_memory_and_version_unchanged(), Arc, BTreeMap, Option (+6 more)

### Community 31 - "P0-3 — Config schema · Preset (appdb) · Strategy model · IPC"
Cohesion: 0.12
Nodes (15): 1.1 Bảng trường (bắt buộc đánh số version để migrate), 1.2 Ví dụ đầy đủ, 1. Config người dùng — đường dẫn per-OS (1 nơi duy nhất cho mỗi OS), 2.1 Cấu trúc, 2.2 Ký số & cập nhật, 2. Preset theo ứng dụng — `appdb` (`schemas/appdb.v1.schema.json`), 3.1 Quy tắc phân quyền (đọc từ trên xuống, match là dừng), 3.2 Bảng lỗi đã biết → strategy kỳ vọng (dùng làm test oracle) (+7 more)

### Community 32 - "P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết"
Cohesion: 0.12
Nodes (15): 10. Task (chi tiết `P2-6-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Callback — luật bất biến (mirror `P1-2 §3`), 4. Chọn loại tap & permission (feature-detect 3 bước — `PLAN §5.1`), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 trên app không IMK), 5.3 `ForwardAsCommit` — chỉ chèn text, không xóa (terminal qua tap — hiếm). (+7 more)

### Community 33 - "properties"
Cohesion: 0.12
Nodes (16): type, enum, properties, enum, $ref, type, maxLength, type (+8 more)

### Community 34 - "AppDb"
Cohesion: 0.19
Nodes (6): Option, TextOwnership, TextRange, ThreadState, AppDb, Entry

### Community 35 - "windows-hook/src/lib.rs"
Cohesion: 0.14
Nodes (21): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), CallbackDecision, empty_result(), engine_never_transforms_when_policy_says_pass(), EngineOutcome, hook_db(), HookMode (+13 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "tip.rs"
Cohesion: 0.09
Nodes (28): current_exe_name(), Default, GUID, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ITfKeystrokeMgr (+20 more)

### Community 38 - "super"
Cohesion: 0.18
Nodes (5): fold(), n(), DiacriticStyle, Vec, super

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
Nodes (11): 10. Mapping task, 1. Tray — `textvn-tray.exe` (chạy 1 instance, notify icon), 2. IPC server (P0-3 §4) — chạy trong tray, 3. Settings — egui (cửa sổ 1, tab dọc), 4. Cài đặt — `textvn-setup.exe` (Inno Setup 6, x86_64), 5. Ký số & nguồn gốc (RW3 — antivirus), 6. Watchdog & process orchestration (tray), 7. Updater — crate `textvn-updater` (lib, chạy trong `textvn-tray.exe`) (+3 more)

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
Cohesion: 0.16
Nodes (21): abi_sizes(), build_pkzip(), check_instance_mutex(), check_pipe_listening(), collect_log_tail(), collect_report(), config_path(), crc32() (+13 more)

### Community 57 - "test_field_detect.c"
Cohesion: 0.31
Nodes (12): lc_classify_field(), lc_utf32_to_utf8(), lc_utf8_to_utf32(), main(), test_address_bar_field(), test_candidate_field(), test_chat_field(), test_combo_field() (+4 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "rules_win.rs"
Cohesion: 0.07
Nodes (32): CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), FieldContext, normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated(), ProbeCache, ProbeCache<K> (+24 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.03
Nodes (65): gdi, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_GROUPBOX, BS_PUSHBUTTON (+57 more)

### Community 62 - "stdio"
Cohesion: 0.13
Nodes (15): lc_detect_active_framework(), lc_env_check(), get_time_ms(), lc_field_detect(), str_contains_icase(), main(), test_env_report_generation(), ctype (+7 more)

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

### Community 68 - "config_paths_fail_open_va_khong_echo_text"
Cohesion: 0.15
Nodes (13): cstr, abi_invariants_giu_duoi_input_ngau_nhien(), assert_sane(), config_paths_fail_open_va_khong_echo_text(), new_result(), Rng, Self, STRATEGY_MAX_ID (+5 more)

### Community 69 - "textvn-ffi"
Cohesion: 0.41
Nodes (13): textvn-appdb, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-ffi, textvn-field-detect, textvn-fuzz (+5 more)

### Community 70 - "items"
Cohesion: 0.18
Nodes (13): default, items, type, items, additionalProperties, pattern, required, type (+5 more)

### Community 71 - "compose.rs"
Cohesion: 0.06
Nodes (40): BACK, backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CompositionPlan, CONTROL, ctrl_shift_tap_toggles_once_and_only_without_other_keys(), delete_beyond_composition_is_reported_not_guessed() (+32 more)

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "lc_config_sync"
Cohesion: 0.15
Nodes (18): ime_instance, lc_config_state, file_stamp(), lc_config_resolve_path(), lc_config_sync(), lc_state_read_enabled(), lc_state_resolve_path(), lc_state_sync() (+10 more)

### Community 74 - "M0 — Spike & nền (tuần 1–2)"
Cohesion: 0.22
Nodes (9): M0 — Spike & nền (tuần 1–2), WIN-001 · Chuẩn bị môi trường build (S, dep: —), WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**, WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002), WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001), WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001), WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song), WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —) (+1 more)

### Community 75 - "P2-0 — macOS MASTER PLAN (Phần 2)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần macOS (exit condition), 2. Kiến trúc process (nhất quán với `P1-0 §2`), 3. Workstream & file solution, 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2), 5. Dependency graph, 6. Rủi ro & mitigation (riêng macOS), 7. Definition of Done cho PHẦN 2, P2-0 — macOS MASTER PLAN (Phần 2)

### Community 76 - "P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên Windows, 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`), 3. App preset mặc định — `data/appdb.default.json` (phần Windows), 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4), 5. Tích hợp với adapter — API nội bộ chung, 6. Test & bằng chứng, 7. Mapping task, P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

### Community 77 - "P3-0 — LINUX MASTER PLAN (Phần 3)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Linux (exit condition), 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`), 3. Workstream & file solution, 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3), 5. Dependency graph, 6. Rủi ro & mitigation (riêng Linux), 7. Definition of Done cho PHẦN 3, P3-0 — LINUX MASTER PLAN (Phần 3)

### Community 78 - "TextVNInputController"
Cohesion: 0.08
Nodes (19): Any, Bool, Data, FieldContext, Int, IpcClient, NSEvent, pid_t (+11 more)

### Community 79 - "undo.rs"
Cohesion: 0.42
Nodes (7): is_vowel(), chars(), mark_horn(), mark_horn_pair_and_single(), mark_vowel(), mark_vowel_applies_and_undoes(), Vec

### Community 80 - "settings_window.c"
Cohesion: 0.06
Nodes (74): gpointer, GtkApplication, on_app_activate(), tv_paths, edit_bool(), edit_reset(), edit_str(), get_bool() (+66 more)

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
Cohesion: 0.09
Nodes (16): AppInfo, KeyMapMacGenerated, UInt32, KeyTranslator, Data, KeyEvent, NSEvent, UInt16 (+8 more)

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.25
Nodes (7): A. PowerShell & C# interop, B. Gửi phím & focus (dạng sai gây "phím bay vào app của user"), C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 88 - "textvn-common.sh"
Cohesion: 0.12
Nodes (41): put(), install.sh script, usage(), textvn-common.sh script, tv_check_package(), tv_err(), tv_fcitx5_system_addon_dir(), tv_ibus_clear_cache() (+33 more)

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

### Community 96 - ".new"
Cohesion: 0.24
Nodes (18): boundary_space_pass_with_default_caps(), danh_sach_khong_dung_cho_tu_khac(), emoji_expands_on_space_trigger(), macro_expands_on_tab_and_eats_trigger(), macro_never_runs_in_secure_field(), macro_not_expanded_after_caret_jump_or_chord(), macro_opts(), macro_trigger_matches_raw_even_after_telex_fold() (+10 more)

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

### Community 101 - "ObjGuard"
Cohesion: 0.33
Nodes (5): ClassFactory, ObjGuard, Default, Drop, Self

### Community 102 - "ipc_server.rs"
Cohesion: 0.10
Nodes (25): filesystem, HANDLE, io, pipes, read, broadcast_reaches_subscriber(), ClientSink, connected_client_pid() (+17 more)

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

### Community 107 - "Json"
Cohesion: 0.35
Nodes (5): Json, P<'a>, Option, Result, run()

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "TextVN IPC v1"
Cohesion: 0.40
Nodes (4): Implementation, Messages, TextVN IPC v1, Wire framing

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
Cohesion: 0.14
Nodes (15): ApplyReplace, Int, Void, MarkedState, .graphemeCount, .isEmpty, .scalarCount, .utf16Count (+7 more)

### Community 116 - "english_words"
Cohesion: 0.50
Nodes (4): $comment, default, type, english_words

### Community 119 - "ImeEngine"
Cohesion: 0.09
Nodes (23): Action, commit, pass, replace, restore, FFI, ImeEngine, ImeError (+15 more)

### Community 120 - "config_version"
Cohesion: 0.67
Nodes (3): $comment, const, config_version

### Community 121 - "free_marking"
Cohesion: 0.67
Nodes (3): default, type, free_marking

### Community 122 - "IpcClient"
Cohesion: 0.17
Nodes (9): IpcClient, IpcClientDelegate, DispatchSourceRead, DispatchSourceTimer, Int32, TimeInterval, Fixed, DispatchQueue (+1 more)

### Community 127 - "EngineSession"
Cohesion: 0.21
Nodes (7): EngineSession, macro_when_vi_off_runs_engine_in_passthrough(), Drop, ime_instance, Result, runtime_config_reload_updates_engine_method(), ime_reset()

### Community 129 - "WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN""
Cohesion: 0.33
Nodes (5): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

### Community 130 - "TsfShared"
Cohesion: 0.22
Nodes (9): caret_at_composition_end(), CompSink_Impl, finish(), ITfComposition, ITfCompositionSink_Impl, Option, Ref, RefCell (+1 more)

### Community 131 - "Strategy"
Cohesion: 0.19
Nodes (10): crate, downgrade(), resolve(), ResolveInput, Option, Strategy, default_for_field(), FIELD_ROLES (+2 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.23
Nodes (9): f(), fold(), fold_caps(), generated_key_tables_are_self_consistent(), is_word_char(), Method, DiacriticStyle, Vec (+1 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (16): 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5, 7. Bang ghi chu (+8 more)

### Community 134 - "encode_frame"
Cohesion: 0.13
Nodes (23): read_next_message(), R, Result, W, send_message(), CodecError, decode_exact_frame(), encode_frame() (+15 more)

### Community 135 - "EventTapController"
Cohesion: 0.05
Nodes (40): CGFlagMapper, EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, Bool, Int, NSObjectProtocol (+32 more)

### Community 138 - "P2-REVIEW-LOG — Phần 2 (macOS)"
Cohesion: 0.12
Nodes (17): (B) Windows / xtask / Linux — `R3-*`, Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging), Code Review Round 2 — Nhất quán & Sẵn sàng (Kiểm thử, Autostart S5 & Residue S9), Kiểm chứng sau fix (Review 2 cuối), Kiểm chứng vòng 10 (host Windows, trước commit), Kết quả kiểm tra sau fix, Kết quả tổng, P2-REVIEW-LOG — Phần 2 (macOS) (+9 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "common_words.rs"
Cohesion: 0.21
Nodes (21): assert_none(), check_all(), keys_for(), Method, Option, Vec, telex_capitalized_with_shift(), telex_caps_lock_and_shift_acronyms() (+13 more)

### Community 142 - "guids.rs"
Cohesion: 0.39
Nodes (7): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI, PROFILE_TEXTVN, GUID

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "edit_session.rs"
Cohesion: 0.13
Nodes (26): apply_display_attribute(), apply_plan(), CompSink, EndCompositionSession, focus_is_password_edit(), input_scopes(), KeyEditSession, read_field_signals() (+18 more)

### Community 145 - "P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow"
Cohesion: 0.25
Nodes (7): 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng), 2. Trách nhiệm từng crate & ranh giới, 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa), 4. CI (GitHub Actions), 5. Versioning & branching, 6. Tham chiếu ngược, P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow

### Community 149 - "autostart.rs"
Cohesion: 0.08
Nodes (44): error_success, registry, APP_RUN_VALUE_NAME, autostart_command(), command_points_into(), disable_autostart(), disable_autostart_for_dir(), disable_linux_autostart_in_dir() (+36 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Changelog"
Cohesion: 0.08
Nodes (24): [0.2.1] — 2026-09-30, [0.2.2] — 2026-09-30, [0.2.3] — 2026-10-01, [0.2.4] — 2026-10-01, [0.2.5] — 2026-10-02, [0.2.6] — 2026-10-02, [0.2.7] — 2026-10-02, Added (+16 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.09
Nodes (31): getmodulehandlew, id_exit, shell, check_status(), ensure_tsf_tip_registered(), free_ctrl_shift_cli(), HOTKEY_CTRL_DOWN, HOTKEY_OTHER_KEY_DOWN (+23 more)

### Community 154 - "menu.rs"
Cohesion: 0.10
Nodes (26): HMENU, textvn_config, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_HEALTH_STATUS (+18 more)

### Community 155 - "SettingsController"
Cohesion: 0.09
Nodes (18): Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer, MacroTrigger (+10 more)

### Community 156 - "compose.c"
Cohesion: 0.30
Nodes (14): boundary_char(), lc_comp, lc_key, lc_plan, is_executable(), lc_comp_apply(), lc_find_settings_binary(), lc_key_engine_ch() (+6 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.17
Nodes (23): lc_ipc_client, get_time_ms(), handle_ipc_message(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override(), lc_ipc_client_is_online(), lc_ipc_client_new() (+15 more)

### Community 158 - "tray/src/lib.rs"
Cohesion: 0.09
Nodes (16): ipcserver, ordering, settings, show_settings_dialog, compatibility_hook_path(), LAST_GLOBAL_TOGGLE_MS, AtomicIsize, AtomicU64 (+8 more)

### Community 159 - "P0 — REVIEW LOG (Phần 0: Nền tảng chung)"
Cohesion: 0.33
Nodes (5): Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

### Community 160 - "doc.rs"
Cohesion: 0.14
Nodes (22): atomic_write(), create_private_dir(), defaults(), DocKind, invalid_values_are_rejected_without_changing_the_doc(), macros_text_round_trip_through_doc(), MAX_FILE_BYTES, missing_keys_read_schema_defaults() (+14 more)

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
Cohesion: 0.08
Nodes (44): InputContext, Instance, KeyEvent, string, lc_ipc_client, launchSettings(), selfDir(), TextVNEngine (+36 more)

### Community 168 - "test_compose.c"
Cohesion: 0.29
Nodes (17): lc_key_classify(), doc, ime_instance, lc_plan, doc_text(), engine_with(), expect(), main() (+9 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.08
Nodes (34): CLIENT, GLOBAL_KEY, global_switch_from_tray_disables_every_app(), IpcClient, IpcState, local_toggle_flips_immediately_without_tray(), map(), parse_state_json() (+26 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "HWND"
Cohesion: 0.24
Nodes (23): HGDIOBJ, close_macro_editor(), dialog_wnd_proc(), get_chk(), macro_wnd_proc(), populate_controls_from_config(), read_window_text(), refresh_if_open() (+15 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "create_and_show_window"
Cohesion: 0.19
Nodes (15): center_on_work_area(), create_and_show_window(), create_control(), create_control_ex(), create_dialog_controls(), register_class(), HINSTANCE, Vec (+7 more)

### Community 176 - "generate_tray_icons.py"
Cohesion: 0.16
Nodes (21): FreeTypeFont, ImageDraw, math, os, _centered_text(), create_us_icon(), create_vn_icon(), _draw_star() (+13 more)

### Community 177 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.18
Nodes (9): FieldRole, UInt32, 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng) (+1 more)

### Community 178 - "TrayApp"
Cohesion: 0.19
Nodes (17): add_tray_icon(), APP_INSTANCE, copy_to_wide_buf(), ensure_hook_running(), Arc, HWND, IpcServer, LPARAM (+9 more)

### Community 180 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Nguồn `FieldContext` trên Linux, 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux), 7. Env matrix & detect framework (dùng cho `textvn doctor` — `PLAN §5.3`), 8. Test (+1 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.38
Nodes (3): Sign-TextVNFile(), Write-Fail(), Write-Ok()

### Community 182 - "AppDelegate"
Cohesion: 0.09
Nodes (15): AppDelegate, .frontAppEnabled, .isVietnameseMode, Bool, IpcServer, NSMenu, NSObjectProtocol, UInt32 (+7 more)

### Community 183 - "check_no_injection_apis.py"
Cohesion: 0.24
Nodes (6): Check relative markdown links trong docs/ + *.md goc. Chay local: python…, Check API inject / giong malware bi cam (Farch-3). Chay local: python…, pathlib, re, subprocess, sys

### Community 184 - "DialogContext"
Cohesion: 0.67
Nodes (4): DialogContext, Arc, IpcServer, show_settings_dialog()

### Community 185 - "Engine"
Cohesion: 0.30
Nodes (6): Action, Engine, Outcome, KeyEvent, Option, Vec

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "register_and_activate_tsf"
Cohesion: 0.20
Nodes (11): advice_for_failure(), DIALOG_CTX, read_log_tail(), register_and_activate_tsf(), register_log_path(), register_log_tail_reads_last_lines(), Option, Path (+3 more)

### Community 189 - "Foundation"
Cohesion: 0.09
Nodes (20): Bool, InjectedMarker, Int64, KeyMapMacTests, MarkedStateTests, MarkedStateUnitTests, LayoutCache, .current (+12 more)

### Community 190 - ".log"
Cohesion: 0.13
Nodes (11): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, Perf — job `perf regression` vẫn đỏ trên CI, **không phải** hồi quy, Ranh giới xác minh (+3 more)

### Community 191 - "String"
Cohesion: 0.13
Nodes (17): IpcServer, IpcServerDelegate, Any, Bool, Data, Date, DispatchSourceRead, Int (+9 more)

### Community 192 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - "windows-tsf/src/replay.rs"
Cohesion: 0.14
Nodes (23): CLASS_NAME, DeferredReplay, deliver(), HOPS, module_instance(), Pending, prepare(), HINSTANCE (+15 more)

### Community 195 - "OwnerRule"
Cohesion: 0.24
Nodes (6): EngineOwner, imk, tap, OwnerRule, Bool, OwnerRuleTests

### Community 196 - "cli/src/main.rs"
Cohesion: 0.18
Nodes (14): abi_sizes(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister(), cmd_verify() (+6 more)

### Community 197 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 198 - "core"
Cohesion: 0.21
Nodes (11): Cell, Default, ObjGuard, Self, Vec, TextVNDisplayAttributeInfo, TextVNDisplayAttributeProvider, TextVNEnumDisplayAttributeInfo (+3 more)

### Community 199 - "Security Policy"
Cohesion: 0.50
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (18): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+10 more)

### Community 201 - "SvcManager"
Cohesion: 0.18
Nodes (14): DocError, apply_option_checkbox(), persist_keeps_unknown_keys_and_reset_keeps_macros(), AtomicU64, DiacriticStyle, FnOnce, MacroEntry, MacroTrigger (+6 more)

### Community 202 - "2. PRODUCT OWNER VIEW"
Cohesion: 0.33
Nodes (6): 2.1 Tầm nhìn & USP, 2.3 Backlog ưu tiên (MoSCoW), 2.4 "EVKey Spec" — danh sách hành vi phải implement clean-room, 2.5 KPI & Definition of Success, 2.6 Release train, 2. PRODUCT OWNER VIEW

### Community 203 - "3. SOLUTION ARCHITECT VIEW"
Cohesion: 0.25
Nodes (8): 3.1 C4 — Context, 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*, 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**, 3.4 Output Strategy Engine (trái tim của app-compat), 3.5 Config & state, 3.6 UI/Tray (1 codebase, 3 native shell), 3.7 Packaging & Distribution, 3. SOLUTION ARCHITECT VIEW

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "SettingsView"
Cohesion: 0.13
Nodes (18): ActiveAlert, about, .alert, autostartFailed, .id, MacroEditorSheet, .body, RadioButton (+10 more)

### Community 206 - "TextVNState"
Cohesion: 0.17
Nodes (12): ime_instance, InputContext, lc_comp, lc_config_state, lc_modifier_toggle, TextVNState, comp, config (+4 more)

### Community 207 - "linux-fcitx5/src/engine.h"
Cohesion: 0.20
Nodes (10): Instance, TextVNEngineFactory, create, AddonFactory, AddonInstance, AddonManager, cstdint, InputContextProperty (+2 more)

### Community 208 - "fuzz — L4 (P0-4 §1)"
Cohesion: 0.33
Nodes (5): Bất biến mỗi target assert (không chỉ "không panic"), Chạy, fuzz — L4 (P0-4 §1), Kiểm chứng trên máy không có cargo-fuzz, Seed corpus

### Community 209 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.10
Nodes (20): 0. Executive Summary, 4.1 Ngôn ngữ & технологies (quyết định), 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh), 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu), 4.4 Quy trình, 4. ENGINEER VIEW, 6.1 Threat model (tóm tắt STRIDE cho IME), 6.2 License & compliance plan (+12 more)

### Community 210 - "e2e_ibus.c"
Cohesion: 0.22
Nodes (20): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+12 more)

### Community 211 - "generate_app_icons.py"
Cohesion: 0.33
Nodes (8): pil, gradient(), main(), pick_font(), Image, Sinh icon PNG ứng dụng TextVN từ thiết kế badge trong `resources/icons/*.svg`.…, Gradient chéo 45° khớp linearGradient 0%,0%→100%,100% (objectBoundingBox)., render()

### Community 212 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 213 - "Result"
Cohesion: 0.40
Nodes (4): EndCompositionSession_Impl, KeyEditSession_Impl, ITfEditSession_Impl, Result

### Community 214 - "e2e_fcitx5.py"
Cohesion: 0.15
Nodes (14): app_handles(), clear(), ctrl_shift_tap(), press_char(), pump(), Ghi như bảng cài đặt (tmp → rename); chờ để mtime chắc chắn khác lần trước., Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend…, send() (+6 more)

### Community 215 - "AXSnapshot"
Cohesion: 0.23
Nodes (4): AXSnapshot, FieldRules, Bool, FieldRulesTests

### Community 216 - ".OnTestKeyDown"
Cohesion: 0.26
Nodes (15): eaten_label(), guarded(), handle_key_up(), KeySink_Impl, KeyTraceSink_Impl, BOOL, GUID, ITfContext (+7 more)

### Community 217 - "IpcMessage"
Cohesion: 0.11
Nodes (18): IpcMessage, ack, configReload, crashReport, getSnapshot, hello, .json, .jsonData (+10 more)

### Community 218 - "TextVNConfig"
Cohesion: 0.11
Nodes (16): ConfigStore, .config, LoadError, corrupt, missing, MacroEntry, .id, Bool (+8 more)

### Community 225 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`), 8. CI jobs (+1 more)

### Community 226 - "show_dialog_on_startup"
Cohesion: 0.67
Nodes (3): show_dialog_on_startup, default, type

### Community 227 - "ffi/src/settings.rs"
Cohesion: 0.09
Nodes (43): enc(), encode(), Entry, OutputCharset, parse_pair(), push_pair(), Option, Vec (+35 more)

### Community 228 - "NSRange"
Cohesion: 0.14
Nodes (7): IMKTextTarget, Bool, Int, TextCommandTarget, NSRange, Code Review Round 3 — Đúng & Đủ (Rà soát chi tiết từng dòng Carbon, IMK XPC, Protocol Invariants), IMKTextInput

### Community 229 - "foreground.rs"
Cohesion: 0.13
Nodes (19): accessibility, foregroundtracker, HWINEVENTHOOK, pwstr, threading, app_id_from_image_path(), app_of_window(), ForegroundTracker (+11 more)

### Community 230 - "preset_matrix.rs"
Cohesion: 0.30
Nodes (11): b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace(), caps_missing_downgrades(), db(), disabled_app_passthrough(), finder_rename_editbox(), MAC_CAPS (+3 more)

### Community 231 - "AutostartManager"
Cohesion: 0.36
Nodes (4): AutostartManager, Bool, URL, Fixed (macOS — ưu tiên bản này)

### Community 232 - "FieldContext"
Cohesion: 0.29
Nodes (6): Entry, FieldContext, FieldDetect, Date, pid_t, TimeInterval

### Community 233 - "resolve.rs"
Cohesion: 0.36
Nodes (9): default_for_field, downgrade_without_cap(), inp(), secure_field_role_passthrough(), step1_secure_beats_everything(), step2_disabled_passthrough(), step3_user_preset_beats_system(), step5_field_defaults_with_caps() (+1 more)

### Community 234 - "TapPermission"
Cohesion: 0.20
Nodes (8): NSLock, Bool, T, TimeInterval, Void, TapPermission, ApplicationServices, Timer

### Community 235 - "diacritic_style.rs"
Cohesion: 0.21
Nodes (9): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), is_vowel, outputcharset (+1 more)

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.29
Nodes (7): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor, Sự cố đã gặp khi phát hành — tra trước khi xử lý

### Community 237 - "Added"
Cohesion: 0.33
Nodes (6): [0.1.0] — 2026-09-27, Added, Architecture, Core, Developer Tools, Windows Platform

### Community 238 - "Hướng dẫn sử dụng TextVN"
Cohesion: 0.17
Nodes (12): 1. Cài đặt, 2. Bật/tắt tiếng Việt, 3. Kiểu gõ, 4. Bảng điều khiển, 5. Gõ tắt, 6. Cấu hình, 7. Xử lý sự cố, Hướng dẫn sử dụng TextVN (+4 more)

### Community 239 - "keymap_mac.rs"
Cohesion: 0.20
Nodes (4): MAC_KEY_COUNT, mac_to_canonical(), Option, modifier_dung_vk_is_modifier_nhan_duoc()

### Community 240 - "quick_telex.rs"
Cohesion: 0.24
Nodes (6): apply, apply(), expansion(), q(), Option, Vec

### Community 242 - "Hướng dẫn phát triển TextVN"
Cohesion: 0.20
Nodes (10): 1. Kiến trúc, 2. Dựng, 3. Kiểm thử, 4. Quy ước bắt buộc (CI chặn), 5. Phát hành, 6. Tài liệu liên quan, Hướng dẫn phát triển TextVN, Linux (Ubuntu/Debian) (+2 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.05
Nodes (38): 1. Gói phát hành, 2. Kết quả kiểm thử, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Báo cáo dựng & kiểm thử — TextVN, Bản 0.2.1 — phát hành pre-release đã xác minh (+30 more)

### Community 244 - "mac_corpus_cases.rs"
Cohesion: 0.49
Nodes (9): all_cases(), bs(), case(), CorpusCase, du_nhom_case_theo_p2_5(), imk(), Vec, tap() (+1 more)

### Community 245 - "P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên macOS, 2. Bảng AX → `IME_FIELD_*` (module `FieldDetect.swift`, mirror `P1-3 §2`), 3. App preset mặc định — `data/appdb.default.json` (phần macOS), 4. Override & state (khớm `P0-3 §3.1` — cùng bảng với `P1-3 §4`, khác path macOS), 5. API nội bộ (dùng chung IMK + tap), 6. Test, 7. Task, P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết

### Community 246 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 247 - ".inject"
Cohesion: 0.47
Nodes (5): Bool, CGEventSource, Int, UInt16, TapInjector

### Community 249 - "Option"
Cohesion: 0.25
Nodes (10): Entry, FieldRoles, MatchClause, Matcher, Preset, Option, Vec, WireDb (+2 more)

### Community 250 - "smoke-imk.sh"
Cohesion: 0.48
Nodes (5): clear_doc(), read_doc(), say(), smoke-imk.sh script, type_keys()

### Community 251 - "AppDelegate.swift"
Cohesion: 0.33
Nodes (3): Cocoa, TextVNAppLib, XCTest

### Community 252 - "Kế hoạch ký số (code signing) — TextVN"
Cohesion: 0.20
Nodes (10): 1. So sánh các đường chính thống (cập nhật 2026-10-02), 2. Lộ trình đề xuất — SignPath Foundation, 3. Smart App Control / SmartScreen — kỳ vọng thực tế, 4. Việc đã wire trong repo, Bước 1 — Nộp đơn (làm một lần, maintainer tự làm), Bước 2 — Tạo artifact configuration trên SignPath.io, Bước 3 — Bật ký trong GitHub Actions (đã chờ sẵn trong `release.yml`), Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai) (+2 more)

### Community 253 - "ApplyError"
Cohesion: 0.29
Nodes (7): ApplyError, cannotDelete, rejected, KeyTranslatorError, noChar, noLayout, Error

### Community 254 - "check_version_sync.rs"
Cohesion: 0.22
Nodes (12): check_site(), extract(), PLIST_KEY, repo_hien_tai_dong_bo(), Option, Result, run(), Site (+4 more)

### Community 255 - "test-typing.ps1"
Cohesion: 0.36
Nodes (4): Check(), Clear-Verified(), Codes(), Norm()

### Community 256 - "env-mac — Môi trường build macOS (MAC-001)"
Cohesion: 0.40
Nodes (4): Acceptance MAC-001, env-mac — Môi trường build macOS (MAC-001), Lệnh kiểm nhanh, Yêu cầu

### Community 257 - "icons.rs"
Cohesion: 0.29
Nodes (6): HICON, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V, load_app_icon(), HINSTANCE

### Community 258 - "Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux)"
Cohesion: 0.40
Nodes (5): 1. Bố cục, 2. Tuỳ chọn ↔ khoá cấu hình, 3. Hành vi chung, 4. Cửa sổ Gõ tắt, Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux)

### Community 259 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 260 - "tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)"
Cohesion: 0.40
Nodes (4): 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật, tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

### Community 262 - "Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)"
Cohesion: 0.67
Nodes (3): 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

### Community 263 - ".heartbeatSummary"
Cohesion: 0.25
Nodes (4): Date, pid_t, URL, NSRunningApplication

### Community 264 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - "log.c"
Cohesion: 0.16
Nodes (13): ensure_parent_dirs(), lc_log_close(), lc_log_init(), bus_disconnected(), gpointer, main(), make_component(), IBusBus (+5 more)

### Community 269 - "method"
Cohesion: 0.67
Nodes (3): default, enum, method

### Community 270 - "output_charset"
Cohesion: 0.67
Nodes (3): default, enum, output_charset

### Community 274 - "EngineOptions"
Cohesion: 0.22
Nodes (7): Context, EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset

### Community 291 - "ConfigWatcher"
Cohesion: 0.19
Nodes (8): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem, Notification

### Community 292 - "path"
Cohesion: 0.18
Nodes (11): all_cases, fs, path, find_rc_exe(), main(), Option, PathBuf, Result (+3 more)

### Community 293 - "OutputStrategy"
Cohesion: 0.29
Nodes (7): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, Int64

### Community 294 - "command"
Cohesion: 0.15
Nodes (13): find_rc_exe(), main(), Option, PathBuf, find_rc_exe(), main(), Option, PathBuf (+5 more)

### Community 295 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 296 - "5. TECHNICAL EXPERT VIEW"
Cohesion: 0.33
Nodes (6): 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**), 5.2 Env compatibility (được yêu cầu đặc biệt), 5.3 Browser compatibility matrix (mục tiêu v1.0), 5.4 App compat matrix (40 app, giữ trong `docs/compat.md`, CI chạy subset), 5.5 Performance budget, 5. TECHNICAL EXPERT VIEW

### Community 297 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 299 - "HookEngine"
Cohesion: 0.33
Nodes (5): HookEngine, Drop, ime_instance, Result, Self

### Community 300 - "classify"
Cohesion: 0.40
Nodes (5): guint, lc_key, classify(), main(), keymap

### Community 301 - "key_event.rs"
Cohesion: 0.20
Nodes (18): is_modifier_vk(), active_modifiers(), defer_to_key_down(), end_composition(), guarded_option(), handle_key(), KeySink, KeyTraceSink (+10 more)

### Community 302 - "Audit tiếp diễn — TextVN, 2026-09-30"
Cohesion: 0.67
Nodes (3): Audit tiếp diễn — TextVN, 2026-09-30, Giới hạn và rủi ro còn mở, Lệnh tái lập

### Community 303 - "handleKey"
Cohesion: 0.12
Nodes (25): lc_modifier, lc_modifier_toggle, lc_modifier_toggle_down(), lc_modifier_toggle_reset(), lc_modifier_toggle_up(), test_modifier_toggle(), KeySym, lc_modifier (+17 more)

### Community 305 - "textvn_ffi"
Cohesion: 0.13
Nodes (12): ime_instance, ime_instance, ime_suggest, abi_version, count, items, lens, ibus (+4 more)

### Community 306 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 307 - "ime_suggest"
Cohesion: 0.40
Nodes (5): ime_suggest, abi_version, count, items, lens

### Community 309 - "[0.2.0] — 2026-09-30"
Cohesion: 0.25
Nodes (8): [0.2.0] — 2026-09-30, Added, Added (macOS — Farch-4), Added (trước đó), Changed, Fixed, Fixed (review vòng 3 — trước tag v0.2.0), Known limitations

### Community 312 - "class.rs"
Cohesion: 0.28
Nodes (8): HR_CLASS_E_NOAGGREGATION, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, AtomicI32, HRESULT, atomic, com

### Community 315 - "cli/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 319 - "Frame"
Cohesion: 0.40
Nodes (5): Frame, frame, needMore, violation, Data

## Knowledge Gaps
- **1199 isolated node(s):** `ic`, `inst`, `comp`, `config`, `toggle` (+1194 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1966 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **55 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TextVNInputController` connect `TextVNInputController` to `NSRange`, `P2-REVIEW-LOG — Phần 2 (macOS)`, `Hướng dẫn phát triển TextVN`, `MarkedState`, `KeyTranslator`, `[0.2.0] — 2026-09-30`, `ImeEngine`, `IpcClient`, `Foundation`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Why does `Hướng dẫn phát triển TextVN` connect `Hướng dẫn phát triển TextVN` to `developer-guide.md`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **Why does `1. Kiến trúc` connect `Hướng dẫn phát triển TextVN` to `TextVNInputController`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `AppDelegate` (e.g. with `(A) macOS app / IMK / packaging / CI — `F3-*`` and `Vòng 10 — đóng nốt các mục hoãn (phát hành `v0.2.2`)`) actually correct?**
  _`AppDelegate` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 19 inferred relationships involving `MarkedState` (e.g. with `TextVNInputController` and `.testBackspaceTypeDeletesRealPrefixBeyondMarked()`) actually correct?**
  _`MarkedState` has 19 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ic`, `inst`, `comp` to the rest of the system?**
  _1199 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `core/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11428571428571428 - nodes in this community are weakly interconnected._