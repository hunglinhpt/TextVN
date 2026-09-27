# Graph Report - tvn-graph  (2026-09-27)

## Corpus Check
- 208 files · ~185,820 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 134 file(s) not represented in the graph (top: .keys 100, .toml 8, .rc 5)

## Summary
- 2967 nodes · 4977 edges · 225 communities (196 shown, 29 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 231 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d4b9dd0d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- core/src/lib.rs
- tsf-min/src/lib.rs
- replay.rs
- config/src/lib.rs
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
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
- ime_key
- P1-1 — TSF ADAPTER (Windows) — Solution chi tiết
- P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết
- viqr.rs
- HookState
- P2-1 — IMK ADAPTER (macOS) — Solution chi tiết
- TextVN — Bộ gõ Tiếng Việt cho Windows (Text Services Framework)
- cli/src/main.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- IpcClient
- field-detect/src/lib.rs
- Chi tiết từng mục
- Tip_Impl
- simple_telex.rs
- P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator
- matchClause
- properties
- properties
- win_corpus_cases.rs
- ime_instance
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
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- doctor.rs
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết
- P3-6 — TEST PLAN (Linux) — Solution chi tiết
- T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004
- settings_dialog.rs
- edit_session.rs
- .parse
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- ime_instance_free
- textvn-ffi
- items
- ime_suggest
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết
- P3-0 — LINUX MASTER PLAN (Phần 3)
- ime_context
- undo.rs
- tip.rs
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- T6 — Test & hardening (tuần 15–18)
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)
- Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại
- rules_win.rs
- appdb.v1.schema.json
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- TextVN — Chỉ mục tài liệu & Giao thức Review
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- P2-REVIEW-LOG — Phần 2 (macOS)
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- ObjGuard
- IpcServer
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- ByteCursor<'a>
- SvcManager
- when
- P1 — REVIEW LOG (Phần 1: Windows)
- TextVN IPC v1
- entry
- id
- auto_capitalize
- auto_restore_english
- diacritic_style
- english_words
- P1-6 — TASKS Windows (WBS) — Nhận việc từng task
- P2-6 — TASKS macOS (WBS) — Nhận việc từng task
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- config_version
- free_marking
- macro_trigger
- macros
- adr/README.md
- tsf-min
- core
- xtask
- WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"
- appdb/src/lib.rs
- Strategy
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- encode_frame
- .CreateInstance
- bool
- engine.h
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- Key
- guids.rs
- uia_spike.ps1
- key_event.rs
- ActionKind
- Project Common Errors — Sổ lỗi chung toàn dự án
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Added
- tray/src/main.rs
- menu.rs
- SettingsController
- super
- ipc_client.c
- tray/src/lib.rs
- P0 — REVIEW LOG (Phần 0: Nền tảng chung)
- ime_result
- AppDb
- Bảng kiểm tra tính đồng bộ giao diện cài đặt (Settings Parity Checklist)
- tsf-min/src/register.rs
- win32-uia.lib.ps1
- Quy trình phát triển
- TextVNEngine
- InputContext
- .new
- fcitx_mock.h
- verify_targets.ps1
- .process
- Performance Audit — TextVN Windows Platform
- HookEngine
- generate_tray_icons.py
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow
- struct
- Engine
- .OnCompositionTerminated
- check_doc_links.py
- push_key
- test_fcitx5_addon.cpp
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- ime_instance_new
- wnd_proc
- string
- InputPanel
- test_field_detect.c
- InputContextEvent
- field_detect.c
- config_parse.rs
- EngineOptions
- log.c
- ime_result_v1
- Security Policy
- Đóng góp cho TextVN
- vowel_table.rs
- 2. PRODUCT OWNER VIEW
- 3. SOLUTION ARCHITECT VIEW
- Build từ source
- CapabilityFlags
- Word
- boundary_space_commits_with_preedit_cap
- fuzz — L4 (P0-4 §1)
- 5. TECHNICAL EXPERT VIEW
- linux_common.h
- 4. ENGINEER VIEW
- 6. AUDIT EXPERT VIEW
- Hướng dẫn sử dụng
- Tiêu chuẩn code
- 7. TEAM, RISK & WORK BREAKDOWN
- 9. APPENDIX
- Cài đặt nhanh (cho người dùng)
- Gỡ cài đặt
- install_linux.sh
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- ime_field_body
- securitystate

## God Nodes (most connected - your core abstractions)
1. `InputContext` - 43 edges
2. `SvcManager` - 37 edges
3. `ime_key` - 35 edges
4. `IpcServer` - 35 edges
5. `SettingsController` - 33 edges
6. `Engine` - 26 edges
7. `TextVNEngine` - 23 edges
8. `AppDb` - 22 edges
9. `run_case()` - 22 edges
10. `type_buf()` - 22 edges

## Surprising Connections (you probably didn't know these)
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `5. Changelog của chỉ mục` --references--> `ime_key`  [INFERRED]
  docs/00-INDEX.md → ffi/include/textvn_ffi.h
- `3.3 Chi phí (budget mọi adapter phải tuân)` --references--> `ime_key`  [INFERRED]
  docs/10-shared/P0-3-config-preset-strategy.md → ffi/include/textvn_ffi.h
- `10. Playbook triển khai (mapping sang task — chi tiết ở `P1-6-TASKS.md`)` --references--> `ime_key`  [INFERRED]
  docs/20-windows/P1-1-tsf.md → ffi/include/textvn_ffi.h
- `12. Failure modes & xử lý` --references--> `ime_key`  [INFERRED]
  docs/20-windows/P1-1-tsf.md → ffi/include/textvn_ffi.h

## Import Cycles
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`

## Communities (225 total, 29 thin omitted)

### Community 0 - "core/src/lib.rs"
Cohesion: 0.12
Nodes (23): ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, apply(), backspace_folds_back(), boundary_space_pass_with_default_caps(), chord_never_eaten() (+15 more)

### Community 1 - "tsf-min/src/lib.rs"
Cohesion: 0.06
Nodes (50): ClassFactory, ClassFactory_Impl, CLSID_TEXTVN_TIP, CompSink, CompSink_Impl, DllCanUnloadNow(), DllGetClassObject(), EditSession (+42 more)

### Community 2 - "replay.rs"
Cohesion: 0.10
Nodes (45): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, Cmd, collect_files() (+37 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.08
Nodes (29): Config, ConfigError, default_json(), defaults_match_p0_3_table(), DiacriticStyle, EmojiEntry, FULL_EXAMPLE, full_example_parses() (+21 more)

### Community 4 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.25
Nodes (7): 0. Executive Summary, 1.1 Bảng tổng hợp, 1.2 Kế thừa & from-scratch — quyết định rõ ràng, 1.3 Ma trận bug kinh niên → kế hoạch xử lý (Đây là "bài toán thật" của dự án), 1. Phân tích 7 nguồn tham chiếu (Audit of sources), 8. ACCEPTANCE CRITERIA — "hoàn hảo" nghĩa là gì (v1.0), PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)

### Community 5 - "verify.rs"
Cohesion: 0.12
Nodes (33): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+25 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (40): accessibility, CTRL_DOWN, empty_ime_result(), get_active_modifiers(), get_process_name_for_window(), GLOBAL_ENABLED, GlobalHookContext, HOOK_MAIN_THREAD_ID (+32 more)

### Community 7 - "toml.rs"
Cohesion: 0.17
Nodes (18): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap, Display (+10 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.07
Nodes (55): find_rc_exe(), main(), Option, PathBuf, find_rc_exe(), main(), Option, PathBuf (+47 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.08
Nodes (24): c_char, ffi, IME_ABI_VERSION, ime_context_v1, IME_ERR_ABI, IME_ERR_CONFIG, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG (+16 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.09
Nodes (20): IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO (+12 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.08
Nodes (27): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+19 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.12
Nodes (28): black_box, F, ime_reset(), instant, bench_field_switch_resolve(), bench_ime_key(), bench_parse_config(), bench_resolve() (+20 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.10
Nodes (35): call_layout_or_tip(), clsid_registry_key(), CLSID_STR, CLSID_TIP, com_init(), do_register(), do_status(), do_unregister() (+27 more)

### Community 14 - "tone.rs"
Cohesion: 0.13
Nodes (18): chars(), DiacriticStyle, new_style_tone_on_last_of_run(), old_style_tone_on_first_of_run(), pick_tone_target(), Option, Vec, run_with_leading_consonant() (+10 more)

### Community 15 - "keymap.rs"
Cohesion: 0.06
Nodes (26): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, ESCAPE, F1, KeyEvent (+18 more)

### Community 16 - "vni.rs"
Cohesion: 0.18
Nodes (7): fold(), free_marking_off_requires_adjacent(), n(), o(), DiacriticStyle, String, vni_as_keys

### Community 17 - "validate.rs"
Cohesion: 0.14
Nodes (18): unmark(), CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI, ONSETS (+10 more)

### Community 18 - "macro.rs"
Cohesion: 0.13
Nodes (16): Emoji, emoji_glyph_is_multi_char(), String, def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches() (+8 more)

### Community 19 - "telex.rs"
Cohesion: 0.14
Nodes (10): circum_pair(), f(), fold(), fold_with(), n(), DiacriticStyle, Option, String (+2 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.12
Nodes (16): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel), 5.3 `ForwardAsCommit` (+8 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.07
Nodes (37): after_request_edit_session(), config_file_path(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, EngineSession, external_edit_clears_ownership_before_next_replace(), focus_generation_rejects_old_uia_result_and_stays_fail_safe() (+29 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.14
Nodes (15): canh_bao_cai_muc_mo_ho(), chars(), danh_sach_khong_pha_tieng_viet_that(), data_stop_en_file_is_self_consistent(), default_english_words(), listed(), parse_word_list(), String (+7 more)

### Community 23 - "ime_key"
Cohesion: 0.12
Nodes (18): 0. Bất biến (invariants), 1. Header (bản 1:1 với `ffi/include/textvn_ffi.h`), 2. Ngữ nghĩa `action` — adapter PHẢI làm đúng bảng này, 3. Vòng đời & thread model, 4. Mapping sang adapter preedit-vs-replace (quan trọng), 5. Mã lỗi & hành vi khi lỗi, 6. Versioning & kiểm chứng bằng CI, 7. Adapter walkthrough (Windows TSF — mẫu để adapter khác làm theo) (+10 more)

### Community 24 - "P1-1 — TSF ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping sang task — chi tiết ở `P1-6-TASKS.md`), 11. Chế độ test & chẩn đoán, 12. Failure modes & xử lý, 1. Tổng quan & quyết định, 2. Cấu trúc file (bám đúng `P0-1 §1`), 3. Vòng đời COM (lifecycle), 4. State machine của ThreadState, 5. Key flow (+9 more)

### Community 25 - "P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P3-7-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 2. Cấu trúc module & repo, 3. Vòng đời IBus, 4. State machine (giống `P1-1 §4`, preedit thay composition), 5.1 Translate `keyval` → `ime_key_v1` (SPIKE `LNX-004` — RL1), 5. Key flow (+9 more)

### Community 26 - "viqr.rs"
Cohesion: 0.15
Nodes (10): apply_key, fold(), marker_to_tone(), n(), push_key(), DiacriticStyle, Option, String (+2 more)

### Community 27 - "HookState"
Cohesion: 0.15
Nodes (18): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), empty_result(), engine_never_transforms_when_policy_says_pass(), hook_db(), HookMode, HookState, .CALLBACK_BUDGET (+10 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.12
Nodes (16): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow, 6.1 `Preedit` (mặc định — strategy `Preedit`) (+8 more)

### Community 29 - "TextVN — Bộ gõ Tiếng Việt cho Windows (Text Services Framework)"
Cohesion: 0.22
Nodes (9): Bảng điều khiển, Cấu trúc dự án, File cấu hình, License, TextVN — Bộ gõ Tiếng Việt cho Windows (Text Services Framework), Trạng thái phát hành, Tính năng, Yêu cầu hệ thống (+1 more)

### Community 30 - "cli/src/main.rs"
Cohesion: 0.20
Nodes (15): abi_sizes(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister(), cmd_verify() (+7 more)

### Community 31 - "P0-3 — Config schema · Preset (appdb) · Strategy model · IPC"
Cohesion: 0.12
Nodes (15): 1.1 Bảng trường (bắt buộc đánh số version để migrate), 1.2 Ví dụ đầy đủ, 1. Config người dùng — đường dẫn per-OS (1 nơi duy nhất cho mỗi OS), 2.1 Cấu trúc, 2.2 Ký số & cập nhật, 2. Preset theo ứng dụng — `appdb` (`schemas/appdb.v1.schema.json`), 3.1 Quy tắc phân quyền (đọc từ trên xuống, match là dừng), 3.2 Bảng lỗi đã biết → strategy kỳ vọng (dùng làm test oracle) (+7 more)

### Community 32 - "P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết"
Cohesion: 0.12
Nodes (15): 10. Task (chi tiết `P2-6-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Callback — luật bất biến (mirror `P1-2 §3`), 4. Chọn loại tap & permission (feature-detect 3 bước — `PLAN §5.1`), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 trên app không IMK), 5.3 `ForwardAsCommit` — chỉ chèn text, không xóa (terminal qua tap — hiếm). (+7 more)

### Community 33 - "properties"
Cohesion: 0.12
Nodes (16): type, enum, properties, enum, $ref, type, maxLength, type (+8 more)

### Community 34 - "IpcClient"
Cohesion: 0.13
Nodes (15): ipc_client_offline_tolerant_starts_and_stops_cleanly(), ipc_state_reload_detection(), IpcClient, IpcState, Arc, AtomicBool, AtomicU64, Default (+7 more)

### Community 35 - "field-detect/src/lib.rs"
Cohesion: 0.11
Nodes (22): FocusProbe, CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), FieldContext, normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated(), ProbeCache (+14 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "Tip_Impl"
Cohesion: 0.21
Nodes (11): FnOnce, ITfContext, ITfTextInputProcessor_Impl, ITfThreadMgr, R, Ref, Result, Tip_Impl (+3 more)

### Community 38 - "simple_telex.rs"
Cohesion: 0.32
Nodes (5): fold(), n(), DiacriticStyle, String, Vec

### Community 39 - "P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator"
Cohesion: 0.15
Nodes (12): 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy), 2.1 Tổng quan, 2.2 Bảng lệnh (chính thức), 2.3 Ví dụ đầy đủ (phải chạy được ngay), 2.3b Lệnh bổ sung (bắt buộc có trong parser), 2.4 Tên phím chuẩn (case-sensitive), 2. Định dạng corpus `.keys`, 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán (+4 more)

### Community 40 - "matchClause"
Cohesion: 0.15
Nodes (13): type, $ref, $ref, matchClause, $ref, additionalProperties, minProperties, properties (+5 more)

### Community 41 - "properties"
Cohesion: 0.15
Nodes (13): default, type, default, type, default, enum, default, enum (+5 more)

### Community 42 - "properties"
Cohesion: 0.15
Nodes (13): minLength, type, minLength, type, properties, expand, glyph, trigger (+5 more)

### Community 43 - "win_corpus_cases.rs"
Cohesion: 0.21
Nodes (14): Item, Iterator, Result, String, run(), all_cases(), CASES_PART1, CASES_PART2 (+6 more)

### Community 44 - "ime_instance"
Cohesion: 0.18
Nodes (11): 2. Kiến trúc tiến trình, 1. Quyết định & capability, 1. Quyết định & capability, 1. Quyết định & phạm vi, ime_instance, 1. Bất biến (P0-2 §0) — đọc trước khi viết adapter, 2. Header (nguyên văn — sinh từ `ffi/include/textvn_ffi.h`), 3. Chú giải: vòng đời chuẩn của adapter (+3 more)

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
Cohesion: 0.11
Nodes (18): 10. Dọn dẹp — checklist trước khi đóng phiên / tick Done, 11. Cập nhật docs & nghiệp vụ — checklist khi "xong", 12.1 Workflow hiện có, 12.2 7 check của `repo-hygiene`, 12.3 Tham chiếu Marketplace (yêu cầu user 2026-09-27), 12. GitHub Actions — giữ repo sạch sau mỗi lần code, 1. Ground rules (G-IDs) — áp dụng mọi lúc, 2. Bắt buộc đọc — trình tự mỗi phiên (+10 more)

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

### Community 55 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.15
Nodes (10): 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5), 8. Spike checklist (task `LNX-006` — tuần 1–2) (+2 more)

### Community 56 - "doctor.rs"
Cohesion: 0.17
Nodes (22): abi_sizes(), build_pkzip(), check_pipe_listening(), check_process_running(), check_tip_registered(), collect_log_tail(), collect_report(), config_path() (+14 more)

### Community 57 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Nguồn `FieldContext` trên Linux, 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux), 7. Env matrix & detect framework (dùng cho `textvn doctor` — `PLAN §5.3`), 8. Test (+1 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`), 8. CI jobs (+1 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.05
Nodes (64): gdi, HGDIOBJ, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_GROUPBOX (+56 more)

### Community 62 - "edit_session.rs"
Cohesion: 0.23
Nodes (10): CompSink, EditAction, ReplaceEditSession, ReplaceEditSession_Impl, Default, ITfContext, ITfEditSession_Impl, ObjGuard (+2 more)

### Community 63 - ".parse"
Cohesion: 0.17
Nodes (16): AppDbError, documented_metadata_and_match_clause_parse(), engine_version_is_supported(), first_matching_entry_and_role_win(), newer_engine_requirement_skips_only_affected_presets(), parse_engine_owner(), parse_role(), parse_strategy() (+8 more)

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

### Community 68 - "ime_instance_free"
Cohesion: 0.32
Nodes (7): ime_instance_free(), abi_invariants_giu_duoi_input_ngau_nhien(), assert_sane(), config_paths_fail_open_va_khong_echo_text(), new_result(), Rng, Self

### Community 69 - "textvn-ffi"
Cohesion: 0.40
Nodes (13): textvn-appdb, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-ffi, textvn-field-detect, textvn-fuzz (+5 more)

### Community 70 - "items"
Cohesion: 0.24
Nodes (10): default, items, type, items, additionalProperties, pattern, required, type (+2 more)

### Community 71 - "ime_suggest"
Cohesion: 0.29
Nodes (7): 8. Ngoài phạm vi v1 (không tự thêm), ime_suggest, abi_version, count, items, lens, 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên Windows, 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`), 3. App preset mặc định — `data/appdb.default.json` (phần Windows), 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4), 5. Tích hợp với adapter — API nội bộ chung, 6. Test & bằng chứng, 7. Mapping task, P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

### Community 74 - "M0 — Spike & nền (tuần 1–2)"
Cohesion: 0.22
Nodes (9): M0 — Spike & nền (tuần 1–2), WIN-001 · Chuẩn bị môi trường build (S, dep: —), WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**, WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002), WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001), WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001), WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song), WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —) (+1 more)

### Community 75 - "P2-0 — macOS MASTER PLAN (Phần 2)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần macOS (exit condition), 2. Kiến trúc process (nhất quán với `P1-0 §2`), 3. Workstream & file solution, 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2), 5. Dependency graph, 6. Rủi ro & mitigation (riêng macOS), 7. Definition of Done cho PHẦN 2, P2-0 — macOS MASTER PLAN (Phần 2)

### Community 76 - "P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên macOS, 2. Bảng AX → `IME_FIELD_*` (module `FieldDetect.swift`, mirror `P1-3 §2`), 3. App preset mặc định — `data/appdb.default.json` (phần macOS), 4. Override & state (khớm `P0-3 §3.1` — cùng bảng với `P1-3 §4`, khác path macOS), 5. API nội bộ (dùng chung IMK + tap), 6. Test, 7. Task, P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết

### Community 77 - "P3-0 — LINUX MASTER PLAN (Phần 3)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Linux (exit condition), 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`), 3. Workstream & file solution, 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3), 5. Dependency graph, 6. Rủi ro & mitigation (riêng Linux), 7. Definition of Done cho PHẦN 3, P3-0 — LINUX MASTER PLAN (Phần 3)

### Community 78 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 79 - "undo.rs"
Cohesion: 0.42
Nodes (7): is_vowel(), chars(), mark_horn(), mark_horn_pair_and_single(), mark_vowel(), mark_vowel_applies_and_undoes(), Vec

### Community 80 - "tip.rs"
Cohesion: 0.13
Nodes (17): Arc, Default, ITfKeyEventSink, ITfKeystrokeMgr, ObjGuard, Option, Rc, RefCell (+9 more)

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

### Community 85 - "`docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)"
Cohesion: 0.25
Nodes (7): 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate, `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.25
Nodes (7): A. PowerShell & C# interop, B. Gửi phím & focus (dạng sai gây "phím bay vào app của user"), C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 88 - "rules_win.rs"
Cohesion: 0.16
Nodes (13): address_or_search(), classify(), contains(), ControlType, known(), lower(), role(), role_of() (+5 more)

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
Nodes (6): 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 93 - "M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006"
Cohesion: 0.29
Nodes (7): M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006, WIN-030 · crate `field-detect` + rules R1–R10 (M) — `P1-3 §2`, WIN-031 · Cache + invalidation + budget 2ms (M), WIN-032 · `engine_owner` + override chain (M) — `P1-3 §4`, `P1-2 §6`, WIN-033 · appdb loader + Ed25519 verify (M) — `P0-3 §2.2`, WIN-034 · Preset 20 app + corpus ≥ 40 case (M) — `P1-3 §3`, WIN-035 · Settings: "thêm preset từ app đang chạy" (S, dep: WIN-052)

### Community 94 - "M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005"
Cohesion: 0.29
Nodes (7): M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005, WIN-040 · `textvn-hook.exe` skeleton + IPC + heartbeat (M) — `P1-2 §2`, WIN-041 · Callback theo `P1-2 §3` (L), WIN-042 · Inject engine 3 modes + modifier restore (L) — `P1-2 §5`, WIN-043 · Focus + UIA worker (M) — `P1-2 §4`, WIN-044 · Rule `engine_owner` phía hook (S, dep: WIN-032), WIN-045 · Game/legacy presets + blocklist (M) — `P1-2 §8`

### Community 95 - "A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006"
Cohesion: 0.29
Nodes (7): A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006, MAC-030 · FieldDetect rules R1–R10 + mock tests (M) — `P2-3 §2`, MAC-031 · Cache + AXObserver + budget (M), MAC-032 · Preset 20 app + corpus ≥ 40 case (M) — `P2-3 §3`, MAC-033 · Appdb loader + Ed25519 (S) — dùng `textvn-appdb`, không viết lại, MAC-034 · Override chain `P2-3 §4` (path mac) (M), MAC-035 · Settings "thêm app đang chạy" (S, dep: MAC-052)

### Community 96 - "P2-REVIEW-LOG — Phần 2 (macOS)"
Cohesion: 0.29
Nodes (6): Kiểm chứng sau fix (Review 2 cuối), Kết quả tổng, P2-REVIEW-LOG — Phần 2 (macOS), Review 1 — Đúng & Đủ, Review 2 — Nhất quán & Sẵn sàng, Rủi ro còn mở của Phần 2 (theo dõi, không phải finding)

### Community 97 - "T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006"
Cohesion: 0.29
Nodes (7): LNX-020 · Addon skeleton + lifecycle (M) — `P3-2 §3`, LNX-021 · keyEvent → PASS (M) — `P3-2 §4`, LNX-022 · Preedit + commit + word boundary (L) — `P3-2 §5`, LNX-023 · SelectionReplace + BackspaceType/surrounding (M) — `P3-2 §5`, LNX-024 · focus/commit-before-hide (M) — `P3-2 §3`, LNX-025 · Framework switch + chống đôi (M) — `P3-2 §7`, T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006

### Community 98 - "T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005"
Cohesion: 0.29
Nodes (7): LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`, LNX-031 · Cache + AT-SPI event invalidate + budget (M), LNX-032 · Preset 20 app + corpus ≥ 40 case (M) — `P3-4 §3`, LNX-033 · Appdb loader + `ime_appdb_verify` (S) — P0-2 §1, không viết lại, LNX-034 · Override chain `P3-4 §6` (path Linux) (M), LNX-035 · `doctor` env matrix + framework auto (M) — `P3-4 §7`, `P3-5 §6`, T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005

### Community 99 - "P3-REVIEW-LOG — Phần 3 (Linux)"
Cohesion: 0.29
Nodes (6): Kiểm chứng sau fix (Review 2 cuối), Kết quả tổng, P3-REVIEW-LOG — Phần 3 (Linux), Review 1 — Đúng & Đủ, Review 2 — Nhất quán & Sẵn sàng, Rủi ro còn mở của Phần 3 (theo dõi, không phải finding)

### Community 100 - "Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`"
Cohesion: 0.29
Nodes (6): 1. Bảng scope — API nào cần elevation?, 2. Fallback đã chứng minh cho `--scope user`, 3. Kết quả verify Win+Space / registry, 4. Finding (bắt buộc áp cho impl), 5. Exit, Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`

### Community 101 - "ObjGuard"
Cohesion: 0.33
Nodes (5): ClassFactory, ObjGuard, Default, Drop, Self

### Community 102 - "IpcServer"
Cohesion: 0.15
Nodes (13): File, broadcast_reaches_subscriber(), ClientSink, IpcServer, Arc, AtomicU32, Instant, Mutex (+5 more)

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

### Community 107 - "SvcManager"
Cohesion: 0.09
Nodes (20): OutputCharset, atomic_write_file(), default_config_dir(), Arc, AtomicU64, BTreeMap, DiacriticStyle, Method (+12 more)

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

### Community 115 - "diacritic_style"
Cohesion: 0.50
Nodes (4): $comment, default, enum, diacritic_style

### Community 116 - "english_words"
Cohesion: 0.50
Nodes (4): $comment, default, type, english_words

### Community 120 - "config_version"
Cohesion: 0.67
Nodes (3): $comment, const, config_version

### Community 121 - "free_marking"
Cohesion: 0.67
Nodes (3): default, type, free_marking

### Community 122 - "macro_trigger"
Cohesion: 0.67
Nodes (3): default, enum, macro_trigger

### Community 123 - "macros"
Cohesion: 0.67
Nodes (3): default, type, macros

### Community 127 - "core"
Cohesion: 0.24
Nodes (9): HR_E_NOTIMPL, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, AtomicI32, HRESULT, com, core (+1 more)

### Community 129 - "WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN""
Cohesion: 0.33
Nodes (5): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

### Community 130 - "appdb/src/lib.rs"
Cohesion: 0.19
Nodes (16): DB, EngineOwner, Entry, FieldRoles, input(), MatchClause, Matcher, Preset (+8 more)

### Community 131 - "Strategy"
Cohesion: 0.15
Nodes (19): crate, default_for_field, downgrade(), downgrade_without_cap(), inp(), resolve(), ResolveInput, Option (+11 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.19
Nodes (9): f(), fold(), generated_key_tables_are_self_consistent(), is_word_char(), Method, DiacriticStyle, String, Vec (+1 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.20
Nodes (9): 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5, 7. Bang ghi chu (+1 more)

### Community 134 - "encode_frame"
Cohesion: 0.12
Nodes (26): read_next_message(), R, Result, W, send_message(), read_next_message(), R, Result (+18 more)

### Community 135 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 138 - "engine.h"
Cohesion: 0.09
Nodes (19): TextVNAddon, engine_, TextVNAddon::TextVNAddon(), TextVNAddonFactory, ContextData, field_role, inst, non_preedit (+11 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "Key"
Cohesion: 0.09
Nodes (11): Key, mods_, release_, KeyEvent, filtered_, ic_, key_, KeyStates (+3 more)

### Community 142 - "guids.rs"
Cohesion: 0.39
Nodes (7): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI, PROFILE_TEXTVN, GUID

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "key_event.rs"
Cohesion: 0.15
Nodes (22): get_active_modifiers(), KeySink, KeySink_Impl, Arc, BOOL, GUID, ITfContext, ITfKeyEventSink_Impl (+14 more)

### Community 149 - "autostart.rs"
Cohesion: 0.15
Nodes (16): foundation, pathbuf, pcwstr, registry, APP_RUN_VALUE_NAME, disable_autostart(), enable_autostart(), is_autostart_enabled() (+8 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Added"
Cohesion: 0.17
Nodes (11): [0.1.0] — 2026-09-27, Added, Added, Architecture, Changelog, Core, Developer Tools, Fixed (+3 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.13
Nodes (16): getmodulehandlew, id_exit, OnceLock, shell, threading, APP_INSTANCE, IDI_ICON_E, IDI_ICON_V (+8 more)

### Community 154 - "menu.rs"
Cohesion: 0.09
Nodes (24): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_HEALTH_STATUS, ID_HOOK_COMPAT_MODE (+16 more)

### Community 155 - "SettingsController"
Cohesion: 0.09
Nodes (18): textvn_config, Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, MacroTrigger (+10 more)

### Community 156 - "super"
Cohesion: 0.10
Nodes (20): PIPE_NAME, atomic, duration, filesystem, HANDLE, io, openoptions, pipes (+12 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.18
Nodes (22): lc_ipc_client, get_time_ms(), handle_ipc_message(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override(), lc_ipc_client_is_online(), lc_ipc_client_new() (+14 more)

### Community 158 - "tray/src/lib.rs"
Cohesion: 0.12
Nodes (12): ipcserver, ordering, show_settings_dialog, compatibility_hook_path(), AtomicIsize, Option, PathBuf, TRAY_HWND (+4 more)

### Community 159 - "P0 — REVIEW LOG (Phần 0: Nền tảng chung)"
Cohesion: 0.33
Nodes (5): Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

### Community 160 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 163 - "Bảng kiểm tra tính đồng bộ giao diện cài đặt (Settings Parity Checklist)"
Cohesion: 0.25
Nodes (7): 1. Tab General (Cài đặt chung), 2. Tab Applications (Ứng dụng & Loại trừ), 3. Tab Hotkeys (Phím tắt chuyển đổi), 4. Tab Hook & Game (Chế độ tương thích sâu), 5. Tab Update (Cập nhật phần mềm), 6. Tab About & Support (Thông tin & Hỗ trợ), Bảng kiểm tra tính đồng bộ giao diện cài đặt (Settings Parity Checklist)

### Community 164 - "tsf-min/src/register.rs"
Cohesion: 0.19
Nodes (22): ITfInputProcessorProfiles, activate(), clsid_key(), com_init(), dll_path(), guid_str(), ILOT_DEFPROFILE, install() (+14 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "Quy trình phát triển"
Cohesion: 0.33
Nodes (6): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, Quy trình phát triển

### Community 167 - "TextVNEngine"
Cohesion: 0.19
Nodes (21): lc_log(), apply_commit_and_reset(), KeyEvent, lc_ipc_client, TextVNEngine, activate, contexts_, current_config_json_ (+13 more)

### Community 168 - "InputContext"
Cohesion: 0.13
Nodes (19): lc_utf32_to_utf8(), apply_result(), Destroyed, InputContext, caps_, committed_text_, deleted_offsets_, deleted_sizes_ (+11 more)

### Community 170 - ".new"
Cohesion: 0.25
Nodes (19): auto_capitalize_after_dot_and_enter(), auto_capitalize_only_first_letter_of_word(), backspace_after_sentence_end_cancels_auto_capitalize(), danh_sach_khong_dung_cho_tu_khac(), emoji_expands_on_space_trigger(), macro_expands_on_tab_and_eats_trigger(), macro_never_runs_in_secure_field(), macro_opts() (+11 more)

### Community 171 - "fcitx_mock.h"
Cohesion: 0.12
Nodes (6): connect(), bitset, Callback, cstdint, functional, vector

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - ".process"
Cohesion: 0.43
Nodes (5): CallbackDecision, EngineOutcome, is_hook_owned(), KeyEvent, Option

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "HookEngine"
Cohesion: 0.33
Nodes (5): HookEngine, Drop, ime_instance, Result, Self

### Community 176 - "generate_tray_icons.py"
Cohesion: 0.15
Nodes (22): FreeTypeFont, Image, ImageDraw, math, os, pil, _centered_text(), create_us_icon() (+14 more)

### Community 177 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.25
Nodes (7): 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng), Targets JSON — `tools/appcomptest/targets/` (WIN-061)

### Community 178 - "P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow"
Cohesion: 0.25
Nodes (7): 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng), 2. Trách nhiệm từng crate & ranh giới, 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa), 4. CI (GitHub Actions), 5. Versioning & branching, 6. Tham chiếu ngược, P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow

### Community 180 - "Engine"
Cohesion: 0.29
Nodes (6): Action, Engine, Outcome, KeyEvent, Option, Vec

### Community 182 - ".OnCompositionTerminated"
Cohesion: 0.33
Nodes (5): CompSink_Impl, ITfComposition, ITfCompositionSink_Impl, Ref, Result

### Community 183 - "check_doc_links.py"
Cohesion: 0.33
Nodes (4): Check relative markdown links trong docs/ + *.md goc. Chay local: python…, pathlib, re, sys

### Community 184 - "push_key"
Cohesion: 0.26
Nodes (17): Horn, post_fixes(), push_key(), push_key(), Vec, stroke(), unmark_all(), is_plain_d() (+9 more)

### Community 185 - "test_fcitx5_addon.cpp"
Cohesion: 0.15
Nodes (12): map_fcitx_key_to_ime(), main(), test_bug_b2_enter_handling(), test_bug_b6_shortcuts(), test_key_mapping(), test_lifecycle_context_destroy(), apply, cassert (+4 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.12
Nodes (16): 10. Task (chi tiết `P3-7-TASKS.md`), 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần)., 5.4 Chèn text (+8 more)

### Community 188 - "ime_instance_new"
Cohesion: 0.22
Nodes (15): flow_new_key_reset_free(), ime_instance, ime_instance_new(), ime_last_error(), ime_reload_config(), ime_set_context(), invalid_config_nonfatal_with_default_instance(), key_char() (+7 more)

### Community 189 - "wnd_proc"
Cohesion: 0.19
Nodes (15): HICON, check_status(), copy_to_wide_buf(), ensure_hook_running(), load_app_icon(), main(), HINSTANCE, HWND (+7 more)

### Community 190 - "string"
Cohesion: 0.21
Nodes (10): lc_detect_active_framework(), lc_env_check(), lc_utf8_to_utf32(), main(), test_env_report_generation(), string, assert, lc_env_info (+2 more)

### Community 191 - "InputPanel"
Cohesion: 0.21
Nodes (7): string, InputPanel, has_underline_, preedit_, Text, has_underline_, TextFormatFlag

### Community 192 - "test_field_detect.c"
Cohesion: 0.44
Nodes (10): lc_classify_field(), main(), test_address_bar_field(), test_candidate_field(), test_chat_field(), test_combo_field(), test_password_field(), test_search_field() (+2 more)

### Community 193 - "InputContextEvent"
Cohesion: 0.22
Nodes (4): InputContextEvent, ic_, InputMethodEngineV2, 10. Failure modes

### Community 194 - "field_detect.c"
Cohesion: 0.22
Nodes (8): get_time_ms(), lc_field_detect(), str_contains_icase(), ctype, 9. Task (chi tiết `P3-7-TASKS.md`), 5. Perf (ngưỡng — `PLAN §5.5`), lc_field_ctx, time

### Community 195 - "config_parse.rs"
Cohesion: 0.20
Nodes (7): cstr, STRATEGY_MAX_ID, assert_result_sane(), ByteCursor, fuzz_target, parse_config, ptr

### Community 196 - "EngineOptions"
Cohesion: 0.22
Nodes (6): disabled_all_pass(), EngineOptions, DiacriticStyle, MacroTrigger, Method, Self

### Community 197 - "log.c"
Cohesion: 0.29
Nodes (7): ensure_parent_dirs(), lc_log_close(), lc_log_init(), stat, stdarg, types, unistd

### Community 198 - "ime_result_v1"
Cohesion: 0.36
Nodes (7): zeroed_result(), fill_result(), ime_key(), ime_key_v1, ime_result_v1, null_args_and_abi_errors_fail_open(), zero_result()

### Community 199 - "Security Policy"
Cohesion: 0.33
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.29
Nodes (7): Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership, Hỏi & Hỗ trợ, License, Quy tắc ứng xử, Đóng góp cho TextVN, Đề xuất tính năng (Feature Request)

### Community 202 - "2. PRODUCT OWNER VIEW"
Cohesion: 0.29
Nodes (7): 2.1 Tầm nhìn & USP, 2.2 Personas & Jobs-to-be-done, 2.3 Backlog ưu tiên (MoSCoW), 2.4 "EVKey Spec" — danh sách hành vi phải implement clean-room, 2.5 KPI & Definition of Success, 2.6 Release train, 2. PRODUCT OWNER VIEW

### Community 203 - "3. SOLUTION ARCHITECT VIEW"
Cohesion: 0.29
Nodes (7): 3.1 C4 — Context, 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*, 3.4 Output Strategy Engine (trái tim của app-compat), 3.5 Config & state, 3.6 UI/Tray (1 codebase, 3 native shell), 3.7 Packaging & Distribution, 3. SOLUTION ARCHITECT VIEW

### Community 204 - "Build từ source"
Cohesion: 0.29
Nodes (7): Build debug (phát triển), Build installer (Inno Setup), Build release (production), Build từ source, Chạy tests, Cài đặt Rust target, Yêu cầu

### Community 205 - "CapabilityFlags"
Cohesion: 0.40
Nodes (3): CapabilityFlags, flags_, CapabilityFlag

### Community 207 - "boundary_space_commits_with_preedit_cap"
Cohesion: 0.33
Nodes (4): boundary_space_commits_with_preedit_cap(), Context, Default, secure_context_all_pass()

### Community 208 - "fuzz — L4 (P0-4 §1)"
Cohesion: 0.33
Nodes (5): Bất biến mỗi target assert (không chỉ "không panic"), Chạy, fuzz — L4 (P0-4 §1), Kiểm chứng trên máy không có cargo-fuzz, Seed corpus

### Community 209 - "5. TECHNICAL EXPERT VIEW"
Cohesion: 0.33
Nodes (6): 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**), 5.2 Env compatibility (được yêu cầu đặc biệt), 5.3 Browser compatibility matrix (mục tiêu v1.0), 5.4 App compat matrix (40 app, giữ trong `docs/compat.md`, CI chạy subset), 5.5 Performance budget, 5. TECHNICAL EXPERT VIEW

### Community 210 - "linux_common.h"
Cohesion: 0.60
Nodes (3): stdbool, stddef, stdint

### Community 211 - "4. ENGINEER VIEW"
Cohesion: 0.40
Nodes (5): 4.1 Ngôn ngữ & технологies (quyết định), 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh), 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu), 4.4 Quy trình, 4. ENGINEER VIEW

### Community 212 - "6. AUDIT EXPERT VIEW"
Cohesion: 0.40
Nodes (5): 6.1 Threat model (tóm tắt STRIDE cho IME), 6.2 License & compliance plan, 6.3 Security/quality gates (mọi PR phải qua), 6.4 External audit (trước v1.0), 6. AUDIT EXPERT VIEW

### Community 213 - "Hướng dẫn sử dụng"
Cohesion: 0.40
Nodes (5): Chuyển đổi chế độ gõ, CLI, Hướng dẫn sử dụng, Tắt/Bật tiếng Việt, Tắt cho từng ứng dụng

### Community 214 - "Tiêu chuẩn code"
Cohesion: 0.50
Nodes (4): Rust, Tests, Tiêu chuẩn code, Windows API

### Community 215 - "7. TEAM, RISK & WORK BREAKDOWN"
Cohesion: 0.50
Nodes (4): 7.1 Team (tối thiểu → lý tưởng), 7.2 Risk register, 7.3 WBS cấp 2 (sprint-ready), 7. TEAM, RISK & WORK BREAKDOWN

### Community 216 - "9. APPENDIX"
Cohesion: 0.50
Nodes (4): 9.1 Nguồn tham chiếu, 9.2 ADR cần viết trong tuần 1, 9.3 Definition of Done (mỗi feature), 9. APPENDIX

### Community 217 - "Cài đặt nhanh (cho người dùng)"
Cohesion: 0.67
Nodes (3): Cài đặt nhanh (cho người dùng), Cách 1 — Installer (khuyến nghị), Cách 2 — Portable (không cài đặt)

### Community 218 - "Gỡ cài đặt"
Cohesion: 0.67
Nodes (3): Gỡ cài đặt, Qua installer:, Thủ công:

## Knowledge Gaps
- **909 isolated node(s):** `engine_`, `inst`, `vi_enabled`, `non_preedit`, `field_role` (+904 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 1402 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ime_key` connect `ime_key` to `A0 — Spike & môi trường (tuần 1–2)`, `field_detect.c`, `TextVNEngine`, `ime_suggest`, `ime_instance`, `P2-1 — IMK ADAPTER (macOS) — Solution chi tiết`, `5. TECHNICAL EXPERT VIEW`, `linux_common.h`, `P1-5 — TEST PLAN (Windows) — Solution chi tiết`, `M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003`, `P2-5 — TEST PLAN (macOS) — Solution chi tiết`, `A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004`, `P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết`, `P1-1 — TSF ADAPTER (Windows) — Solution chi tiết`, `P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết`, `TextVN — Chỉ mục tài liệu & Giao thức Review`, `P0-3 — Config schema · Preset (appdb) · Strategy model · IPC`?**
  _High betweenness centrality (0.116) - this node is a cross-community bridge._
- **Why does `keyEvent` connect `TextVNEngine` to `InputContext`, `test_fcitx5_addon.cpp`, `ime_key`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Why does `lc_field_detect()` connect `field_detect.c` to `T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005`?**
  _High betweenness centrality (0.045) - this node is a cross-community bridge._
- **Are the 3 inferred relationships involving `InputContext` (e.g. with `test_apply_non_preedit_fallback_backspace()` and `test_apply_non_preedit_with_surrounding()`) actually correct?**
  _`InputContext` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 26 inferred relationships involving `ime_key` (e.g. with `keyEvent` and `5. Changelog của chỉ mục`) actually correct?**
  _`ime_key` has 26 INFERRED edges - model-reasoned connections that need verification._
- **What connects `engine_`, `inst`, `vi_enabled` to the rest of the system?**
  _909 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `core/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12318840579710146 - nodes in this community are weakly interconnected._