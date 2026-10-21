# Graph Report - TextVN  (2026-10-21)

## Corpus Check
- 401 files · ~599,274 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 311 file(s) not represented in the graph (top: .keys 248, (none) 14, .toml 10)

## Summary
- 7174 nodes · 13669 edges · 433 communities (366 shown, 67 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 854 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `850945e0`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- .new
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
- .new
- svc.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- store.rs
- AppDb
- Chi tiết từng mục
- Tip_Impl
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
- tsf-min/src/register.rs
- doctor.rs
- StrategyResolver
- P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết
- field-detect/src/lib.rs
- T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004
- settings_dialog.rs
- windows-tsf/src/replay.rs
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- os_macos.rs
- textvn-ffi
- items
- compose.rs
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- tray/src/lib.rs
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- fst.rs
- P3-0 — LINUX MASTER PLAN (Phần 3)
- TextVNInputController
- Ctx
- settings_model.c
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- os_windows.rs
- TextVNConfig
- Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)
- KeyTranslator
- textvn-common.sh
- appdb.v1.schema.json
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- AutostartManager
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- Engine
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- CandidateOverlay
- settings_window.c
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- Issue
- ByteCursor<'a>
- os_linux.rs
- when
- P1 — REVIEW LOG (Phần 1: Windows)
- ObjGuard
- entry
- id
- auto_capitalize
- auto_restore_english
- MarkedState
- ipc_server.rs
- P1-6 — TASKS Windows (WBS) — Nhận việc từng task
- P2-6 — TASKS macOS (WBS) — Nhận việc từng task
- ImeEngine
- tray/src/assist.rs
- edit_session.rs
- P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN
- windows
- adr/README.md
- tsf-min
- EventTapController
- xtask
- WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"
- overlay.rs
- dict.rs
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- ipc/src/lib.rs
- rules_win.rs
- bool
- store_win.rs
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- common_words.rs
- guids.rs
- uia_spike.ps1
- TsfShared
- do_unregister
- .register
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- compose.c
- ipc_client.c
- P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế
- OverlayView
- SettingsDoc
- mmap.rs
- developer-guide.md
- package_bootstrap.rs
- win32-uia.lib.ps1
- ime_result_v1
- engine.cpp
- IpcClient
- ipc_client.rs
- bitset
- verify_targets.ps1
- with_ctx
- Performance Audit — TextVN Windows Platform
- HWND
- generate_tray_icons.py
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- spelling.rs
- .suggest
- ThreadState
- build-release.ps1
- AppDelegate
- sys
- P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN
- tip.rs
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- register_and_activate_tsf
- dictionary_budget.rs
- charset.rs
- IpcServer
- invariants.rs
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- String
- windows-tsf/src/assist.rs
- test_field_detect.c
- .CreateInstance
- cli/src/main.rs
- Security Policy
- Đóng góp cho TextVN
- SvcManager
- dict/src/lib.rs
- trace.rs
- macro_text.rs
- ConfigStore
- suggest
- preset_matrix.rs
- ViState
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- e2e_ibus.c
- generate_app_icons.py
- IpcServer
- ITfEditSession_Impl
- e2e_fcitx5.py
- AXSnapshot
- key_event.rs
- CloseHandle
- Json
- install_linux.sh
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- ime_field_body
- securitystate
- tsf-min/src/lib.rs
- test_settings_model.c
- ffi/src/settings.rs
- charset_matrix.rs
- foreground.rs
- P3-6 — TEST PLAN (Linux) — Solution chi tiết
- Foundation
- FieldDetect
- Path
- TapPermission
- P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- properties
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- quick_telex.rs
- Strategy
- Hướng dẫn phát triển TextVN
- Báo cáo dựng & kiểm thử — TextVN
- remove_added_languages
- Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`)
- Dictionary
- viqr.rs
- TextVNAppTests
- ffi_key.rs
- smoke-imk.sh
- ime_result
- Kế hoạch ký số (code signing) — TextVN
- generate_store_art.py
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- test-typing.ps1
- env-mac — Môi trường build macOS (MAC-001)
- A. WINDOWS
- Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)
- TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV
- tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)
- PackageDescription
- cli/src/assist.rs
- 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)
- quick_telex
- e2e-linux.sh
- mem-check.sh
- soak.sh
- 2. Windows — Spell Checking API (`spellcheck.h`, Win8+)
- [0.2.32] — 2026-10-12
- default
- build-macos.sh
- stage-linux.sh
- build-rust.sh
- mac_corpus_cases.rs
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
- textvn_ibus_engine.h
- command
- check_version_sync.rs
- IMKTextTarget
- linux-fcitx5/src/engine.h
- P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công
- idle_ms
- WORDLIST_LAYOUT
- run_tray_app
- map_file
- .start
- 3. Nhật ký finding
- capitalize_sentence.rs
- class.rs
- verify-store-package.sh
- Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)
- EngineOptions
- ime_assist_issue
- B. macOS
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- Added
- wide
- [0.2.3] — 2026-10-01
- .onEvent
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- textvn_ibus_modifier_kind
- 2. Kết quả kiểm thử
- macSpellContainsCallback
- Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)
- Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành
- ime_result
- Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent
- MACRO_LAYOUT
- load
- Option
- MockHandler
- Diagnostics
- simple_telex.rs
- .CreateInstance
- ime_assist_issue
- verify-msix.py
- diacritic_style
- CTextVNFFI/include/textvn_ffi.h
- do_register
- 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft
- handle_overlay_key
- linux-settings/src/main.c
- IMPLEMENTATION STATUS — adapter macOS (Farch-4)
- HookEngine
- .with
- package_family_full_names
- [0.2.0] — 2026-09-30
- 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành
- TextVN — Chỉ mục tài liệu & Giao thức Review
- 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU
- Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store
- keys_generated.rs
- core/src/lib.rs
- ime_key
- P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow
- Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI
- caps.rs
- prepare-ppa.sh
- 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"
- T6 — Test & hardening (tuần 15–18)
- .inject
- Problem
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- [0.2.10] — 2026-10-03
- 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)
- P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A
- gen_corpus_tatoeba.py
- [0.2.28] — 2026-10-10
- mask.rs
- [0.2.14] — 2026-10-03
- pipe_name
- ffi/include/textvn_ffi.h
- OutputStrategy
- P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15
- DictError
- ime_context
- Phân phối Linux — kênh, store và auto-update
- TapDecision
- Word
- Action
- 11. Audit
- sign-artifacts.sh
- corpus.rs
- P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- AppInfo
- diacritic_typo.rs
- 4. Đánh giá theo vai trò chuyên trách
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- error_success
- unmark_as_unmark_tone
- offer_machine_registration_if_needed
- on_dir_changed
- ime_key
- pre_translate_message
- suggest_vn.rs
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- 3. SOLUTION ARCHITECT VIEW
- [0.2.34] — 2026-10-18
- display
- 4. Hai chế độ — đặc tả chi tiết (sau vòng 3–5)
- P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả
- icons.rs
- 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?
- Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại
- ffi/build.rs
- StageInfo
- P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)
- publish-aur.sh
- ApplyError
- ime_suggest
- save_word_list
- tv_win_free
- c_buf_to_string
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- per_app
- TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux
- en_restore.rs
- MACRO_EX_NONE
- trigger_keys
- TextVN IPC v1
- 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)
- ime_instance_free
- Trạng thái ký số macOS — Developer ID, notarization, stapling
- defaults
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- CompSink_Impl
- EditSession_Impl

## God Nodes (most connected - your core abstractions)
1. `AppDelegate` - 65 edges
2. `SvcManager` - 61 edges
3. `TsfShared` - 51 edges
4. `TextVNInputController` - 49 edges
5. `Engine` - 41 edges
6. `Báo cáo dựng & kiểm thử — TextVN` - 40 edges
7. `Changelog` - 39 edges
8. `MarkedState` - 37 edges
9. `TextVNEngine` - 35 edges
10. `IpcServer` - 34 edges

## Surprising Connections (you probably didn't know these)
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  AUDIT_REPORT_PRODUCTION_READY.md → adapters/linux-common/src/ipc_client.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md → adapters/linux-common/src/ipc_client.c
- `2. Cấu trúc file (bám đúng `P0-1 §1`)` --references--> `expect()`  [INFERRED]
  docs/20-windows/P1-1-tsf.md → adapters/linux-common/tests/test_compose.c

## Import Cycles
- 1-file cycle: `dict/src/mmap.rs -> dict/src/mmap.rs`
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`
- 2-file cycle: `dict/src/lib.rs -> dict/src/os_windows.rs -> dict/src/lib.rs`

## Communities (433 total, 67 thin omitted)

### Community 0 - ".new"
Cohesion: 0.15
Nodes (34): auto_capitalize_after_dot_and_enter(), auto_capitalize_needs_space_after_dot(), auto_capitalize_only_first_letter_of_word(), backspace_after_sentence_end_cancels_auto_capitalize(), backspace_deletes_last_visible_char(), chord_never_eaten(), danh_sach_khong_dung_cho_tu_khac(), emoji_expands_on_space_trigger() (+26 more)

### Community 1 - "Result"
Cohesion: 0.24
Nodes (12): KeySink_Impl, log(), BOOL, ITfComposition, ITfContext, ITfKeyEventSink_Impl, ITfThreadMgr, LPARAM (+4 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.08
Nodes (47): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, case_without_any_expectation_is_rejected(), Cmd (+39 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.05
Nodes (43): assist_defaults_match_p4_0_section_7(), assist_full_block_parses(), ASSIST_TRIGGER_KEYS, AssistAutoFix, AssistDisplay, AssistLanguage, AssistMode, AssistSettings (+35 more)

### Community 4 - "text"
Cohesion: 0.10
Nodes (22): check(), text(), [0.2.8] — 2026-10-02, Added, Changed, Fixed, 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`), 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`) (+14 more)

### Community 5 - "verify.rs"
Cohesion: 0.11
Nodes (32): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+24 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (43): caps_lock_bit(), CTRL_DOWN, FocusProbe, get_active_modifiers(), get_process_name_for_window(), GLOBAL_ENABLED, GlobalHookContext, HOOK_MAIN_THREAD_ID (+35 more)

### Community 7 - "toml.rs"
Cohesion: 0.15
Nodes (18): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+10 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.11
Nodes (37): banner(), emit(), format_rust(), gen_tables(), index_of(), key_char(), KEYMAP_MAC_OUT_RUST, KEYMAP_MAC_OUT_SWIFT (+29 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.04
Nodes (65): assist_check_v1_fail_open_on_empty_null_and_bad_utf16(), assist_check_v1_matches_utf8_path(), assist_check_v1_utf16_offset_is_char_not_utf16(), assist_sentence_tail_v1_caps_in_chars_and_keeps_astral(), copy_str_to_c_buf(), fill_assist_result(), fresh_assist_result(), guarded() (+57 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.09
Nodes (20): IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO (+12 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.13
Nodes (15): A, A_BREVE, A_CIRC, E, E_CIRC, O, O_CIRC, O_HOOK (+7 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.12
Nodes (26): black_box, instant, textvn_field_detect, bench_field_switch_resolve(), bench_parse_config(), bench_resolve(), best_of(), best_of_giu_vong_nhanh_nhat() (+18 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.06
Nodes (37): call_layout_or_tip(), call_layout_or_tip_spec(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, CLSID_INNER, clsid_key() (+29 more)

### Community 14 - "tone.rs"
Cohesion: 0.12
Nodes (24): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), vowel_span(), apply_key() (+16 more)

### Community 15 - "keymap.rs"
Cohesion: 0.06
Nodes (26): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, ESCAPE, F1, KeyEvent (+18 more)

### Community 16 - "vni.rs"
Cohesion: 0.10
Nodes (29): late_stroke(), push_key(), fold(), free_marking_off_requires_adjacent(), n(), o(), push_key(), DiacriticStyle (+21 more)

### Community 17 - "validate.rs"
Cohesion: 0.14
Nodes (19): coda_after_nucleus_ok(), CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI, ONSETS (+11 more)

### Community 18 - "macro.rs"
Cohesion: 0.15
Nodes (14): Emoji, emoji_glyph_is_multi_char(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches(), MacroDef (+6 more)

### Community 19 - "telex.rs"
Cohesion: 0.09
Nodes (20): codas, circum_pair(), f(), fold(), fold_with(), fold_with_caps(), has_uo_pair(), late_d_strokes_the_onset() (+12 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.07
Nodes (39): after_request_edit_session(), classify_tsf_field(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, engine_session_sentence_tail_and_glyph_len(), external_edit_clears_ownership_before_next_replace(), focus_generation_rejects_old_uia_result_and_stays_fail_safe() (+31 more)

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
Cohesion: 0.20
Nodes (18): APP_DB, appdb(), CI_APPS, FIELD_ROLES, good(), LOCATOR_KEYS, mutated(), P (+10 more)

### Community 27 - "engine.c"
Cohesion: 0.13
Nodes (38): lc_modifier, lc_modifier_toggle, lc_modifier_toggle_down(), lc_modifier_toggle_reset(), lc_modifier_toggle_up(), lc_log(), test_modifier_toggle(), gboolean (+30 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+9 more)

### Community 29 - ".new"
Cohesion: 0.32
Nodes (10): broadcast_reaches_subscriber(), frame_encoding_and_read_message_roundtrip(), read_message(), Arc, Result, Self, server_starts_and_stops_cleanly(), serves_every_pipe_name() (+2 more)

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
Cohesion: 0.06
Nodes (39): APP_ID, ARG_AUTOSTART, ARG_GUARD, ARG_INSTALL, ARG_RELAY, build_command_line(), decide_store_run_value(), guard_decision() (+31 more)

### Community 35 - "AppDb"
Cohesion: 0.13
Nodes (24): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), CallbackDecision, empty_result(), engine_never_transforms_when_policy_says_pass(), EngineOutcome, hook_db(), HookMode (+16 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "Tip_Impl"
Cohesion: 0.15
Nodes (16): F, GUID, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ITfTextInputProcessor_Impl, ITfThreadMgr (+8 more)

### Community 38 - "super"
Cohesion: 0.07
Nodes (24): check(), Vec, MAX_GAP_WORDS, bat_khoang_trang_kep(), check(), Vec, INTENTIONAL_REPEATS, is_vietnamese() (+16 more)

### Community 39 - "P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator"
Cohesion: 0.15
Nodes (12): 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy), 2.1 Tổng quan, 2.2 Bảng lệnh (chính thức), 2.3 Ví dụ đầy đủ (phải chạy được ngay), 2.3b Lệnh bổ sung (bắt buộc có trong parser), 2.4 Tên phím chuẩn (case-sensitive), 2. Định dạng corpus `.keys`, 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán (+4 more)

### Community 40 - "matchClause"
Cohesion: 0.15
Nodes (13): type, $ref, $ref, matchClause, $ref, additionalProperties, minProperties, properties (+5 more)

### Community 41 - "properties"
Cohesion: 0.09
Nodes (22): default, type, $comment, const, default, type, default, enum (+14 more)

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

### Community 55 - "tsf-min/src/register.rs"
Cohesion: 0.06
Nodes (44): Cell, Default, GUID, IEnumTfDisplayAttributeInfo, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ObjGuard, Option (+36 more)

### Community 56 - "doctor.rs"
Cohesion: 0.14
Nodes (24): abi_sizes(), build_pkzip(), check_instance_mutex(), check_pipe_listening(), collect_log_tail(), collect_report(), config_path(), crc32() (+16 more)

### Community 57 - "StrategyResolver"
Cohesion: 0.20
Nodes (12): Key, StrategyResolver, .cacheCount, .hasAppdb, Bool, Data, Int, Int32 (+4 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "field-detect/src/lib.rs"
Cohesion: 0.11
Nodes (20): probe_focused_element(), K, CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), FieldContext, normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated() (+12 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.02
Nodes (87): hidpi, normalize_word_list, ABOUT_DETAILS, ABOUT_TAGLINE, about_text(), about_text_matches_every_platform(), BM_GETCHECK, BM_SETCHECK (+79 more)

### Community 62 - "windows-tsf/src/replay.rs"
Cohesion: 0.16
Nodes (21): CLASS_NAME, DeferredReplay, deliver(), HOPS, module_instance(), Pending, prepare(), HINSTANCE (+13 more)

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

### Community 68 - "os_macos.rs"
Cohesion: 0.05
Nodes (80): available_languages(), class_of(), close_spell_document(), contains_giua_cua_so_khong_reset_bo_dem(), contains_word(), en_us_neu_may_co_thi_provider_hoat_dong(), handle_job(), Job (+72 more)

### Community 69 - "textvn-ffi"
Cohesion: 0.35
Nodes (15): textvn-appdb, textvn-assist, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-dict, textvn-ffi (+7 more)

### Community 70 - "items"
Cohesion: 0.13
Nodes (17): default, items, type, $comment, default, items, type, additionalProperties (+9 more)

### Community 71 - "compose.rs"
Cohesion: 0.06
Nodes (40): BACK, backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CompositionPlan, CONTROL, ctrl_shift_tap_toggles_once_and_only_without_other_keys(), delete_beyond_composition_is_reported_not_guessed() (+32 more)

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "tray/src/lib.rs"
Cohesion: 0.05
Nodes (47): ipcserver, ordering, settings, show_settings_dialog, claim_debounced(), compatibility_hook_path(), copy_to_wide_buf(), CTRL_SHIFT_MARKER (+39 more)

### Community 74 - "M0 — Spike & nền (tuần 1–2)"
Cohesion: 0.22
Nodes (9): M0 — Spike & nền (tuần 1–2), WIN-001 · Chuẩn bị môi trường build (S, dep: —), WIN-002 · SPIKE: TSF-in-Rust (L, dep: WIN-001) — **ADR-005, chặn mọi task WS2**, WIN-003 · SPIKE: đăng ký TIP per-user (M, dep: WIN-002), WIN-004 · SPIKE: UIA latency & IsPassword (M, dep: WIN-001), WIN-005 · SPIKE: hook + GHA runner session (M, dep: WIN-001), WIN-006 · Corpus `corpus/win/` đầu tiên (M, dep: —, song song), WIN-007 · Xác nhận oracle UniKey (rủi ro carry-over từ Phần 0) (S, dep: —) (+1 more)

### Community 75 - "P2-0 — macOS MASTER PLAN (Phần 2)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần macOS (exit condition), 2. Kiến trúc process (nhất quán với `P1-0 §2`), 3. Workstream & file solution, 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2), 5. Dependency graph, 6. Rủi ro & mitigation (riêng macOS), 7. Definition of Done cho PHẦN 2, P2-0 — macOS MASTER PLAN (Phần 2)

### Community 76 - "fst.rs"
Cohesion: 0.07
Nodes (39): I, Builder, BuildNode, BTreeMap, Option, Self, Vec, decode_node() (+31 more)

### Community 77 - "P3-0 — LINUX MASTER PLAN (Phần 3)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Linux (exit condition), 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`), 3. Workstream & file solution, 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3), 5. Dependency graph, 6. Rủi ro & mitigation (riêng Linux), 7. Definition of Done cho PHẦN 3, P3-0 — LINUX MASTER PLAN (Phần 3)

### Community 78 - "TextVNInputController"
Cohesion: 0.09
Nodes (18): Any, Bool, Data, FieldContext, Int, IpcClient, NSEvent, pid_t (+10 more)

### Community 79 - "Ctx"
Cohesion: 0.11
Nodes (23): bat_lap_tu_thuong(), check(), Vec, Ctx, DEFAULT_SUGGEST_BUDGET, is_real_sentence_end(), is_sentence_end(), is_word() (+15 more)

### Community 80 - "settings_model.c"
Cohesion: 0.11
Nodes (25): tv_paths, edit_bool(), edit_reset(), edit_str(), get_bool(), get_index(), patch_file(), tv_config_reset_defaults() (+17 more)

### Community 81 - "M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)"
Cohesion: 0.25
Nodes (8): M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC), WIN-060 · `appcomptest` harness (L) — `P1-5 §4`, WIN-061 · Targets JSON cho 12 app CI (M), WIN-062 · Perf bench + baseline (M) — `P1-5 §5`, WIN-063 · Soak 24h script (M), WIN-064 · CI jobs (M) — `P1-5 §8`, WIN-065 · Fuzz + ASan/TSan job Windows (M), WIN-066 · Security + license checklist Phần 1 (M) — Handbook §8

### Community 82 - "A4 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): A4 — Test & hardening (tuần 15–18), MAC-060 · AX harness `tools/mac/ax-driver` (L) — `P2-5 §4`, MAC-061 · Targets JSON 12 app (cùng format Windows) (M), MAC-062 · Perf bench + `perf/baseline-mac.json` (M) — `P2-5 §5`, MAC-063 · Soak 24h script (M), MAC-064 · CI jobs macos (M) — `P2-5 §8`, MAC-065 · Fuzz + ASan job mac (S), MAC-066 · Security + license checklist Phần 2 (M) — Handbook §8 + audit tap (P2-2 §7)

### Community 83 - "T5 — Tray / Settings / Packaging (tuần 11–14)"
Cohesion: 0.25
Nodes (8): LNX-050 · Tray SNI + menu 9 mục + single instance (M) — `P3-5 §1`, LNX-051 · IPC server + watcher + health (L) — `P3-5 §2/§4`, LNX-052 · Settings GTK4 6 tab + parity checklist (L) — `P3-5 §3`, LNX-053 · systemd user unit + config init + hot-reload (M), LNX-054 · `.deb` cài/gỡ sạch (L) — `P3-5 §5`, LNX-055 · `doctor` full (M) — `P3-5 §6`, LNX-056 · Update-notify + `.rpm` + AUR (M) — `P3-5 §7/§8`, T5 — Tray / Settings / Packaging (tuần 11–14)

### Community 84 - "os_windows.rs"
Cohesion: 0.07
Nodes (50): CORRECTIVE_ACTION, cached_support(), collect_issues(), collect_suggestions(), ComApartment, create_checker(), create_factory(), en_us_neu_may_co_thi_provider_hoat_dong() (+42 more)

### Community 85 - "TextVNConfig"
Cohesion: 0.17
Nodes (17): AssistDisplay, AssistSettings, .isOn, EmojiEntry, LoadError, corrupt, missing, MacroEntry (+9 more)

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "KeyTranslator"
Cohesion: 0.26
Nodes (7): KeyTranslator, Data, KeyEvent, NSEvent, UInt16, UInt32, TISInputSource

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

### Community 92 - "AutostartManager"
Cohesion: 0.16
Nodes (9): AutostartManager, Bool, URL, [0.2.2] — 2026-09-30, Changed, Fixed (macOS — ưu tiên bản này), Fixed (Windows), Housekeeping (+1 more)

### Community 93 - "M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006"
Cohesion: 0.29
Nodes (7): M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006, WIN-030 · crate `field-detect` + rules R1–R10 (M) — `P1-3 §2`, WIN-031 · Cache + invalidation + budget 2ms (M), WIN-032 · `engine_owner` + override chain (M) — `P1-3 §4`, `P1-2 §6`, WIN-033 · appdb loader + Ed25519 verify (M) — `P0-3 §2.2`, WIN-034 · Preset 20 app + corpus ≥ 40 case (M) — `P1-3 §3`, WIN-035 · Settings: "thêm preset từ app đang chạy" (S, dep: WIN-052)

### Community 94 - "M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005"
Cohesion: 0.29
Nodes (7): M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005, WIN-040 · `textvn-hook.exe` skeleton + IPC + heartbeat (M) — `P1-2 §2`, WIN-041 · Callback theo `P1-2 §3` (L), WIN-042 · Inject engine 3 modes + modifier restore (L) — `P1-2 §5`, WIN-043 · Focus + UIA worker (M) — `P1-2 §4`, WIN-044 · Rule `engine_owner` phía hook (S, dep: WIN-032), WIN-045 · Game/legacy presets + blocklist (M) — `P1-2 §8`

### Community 95 - "A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006"
Cohesion: 0.29
Nodes (7): A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006, MAC-030 · FieldDetect rules R1–R10 + mock tests (M) — `P2-3 §2`, MAC-031 · Cache + AXObserver + budget (M), MAC-032 · Preset 20 app + corpus ≥ 40 case (M) — `P2-3 §3`, MAC-033 · Appdb loader + Ed25519 (S) — dùng `textvn-appdb`, không viết lại, MAC-034 · Override chain `P2-3 §4` (path mac) (M), MAC-035 · Settings "thêm app đang chạy" (S, dep: MAC-052)

### Community 96 - "Engine"
Cohesion: 0.27
Nodes (6): Engine, Outcome, KeyEvent, Option, Vec, Tail

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

### Community 101 - "CandidateOverlay"
Cohesion: 0.11
Nodes (20): Anchor, CandidateOverlay, create_font(), measure(), overlay_creation_and_visibility(), overlay_shows_view_and_hides(), overlay_wnd_proc(), paint_overlay() (+12 more)

### Community 102 - "settings_window.c"
Cohesion: 0.19
Nodes (29): add_escape_closes(), button(), GtkApplication, framed_grid(), labeled_dropdown(), on_about(), on_close(), on_default() (+21 more)

### Community 103 - "env.md — Môi trường build Windows (WIN-001)"
Cohesion: 0.33
Nodes (5): 1. Build host (máy dev), 2. Tools DoD (Handbook §4 mục 4), 3. Bằng chứng acceptance (WIN-001), 4. Gap & việc cần khi lên milestone liên quan, env.md — Môi trường build Windows (WIN-001)

### Community 104 - "A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009"
Cohesion: 0.33
Nodes (6): A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009, MAC-040 · Tap thread skeleton + owner check (M) — `P2-2 §2/§3`, MAC-041 · Callback + self-disable (M) — `P2-2 §3`, MAC-042 · Inject §5 (3 modes + marker + restore) (L), MAC-043 · Permission UX + revocation poll (M) — `P2-2 §4`, MAC-044 · Rule owner `imk|tap` + IPC sync (S, dep: MAC-034)

### Community 105 - "Issue"
Cohesion: 0.09
Nodes (29): apply_auto_fixes(), chi_giu_loi_can_sua(), Fix, span_len(), sua_khoang_trang(), sua_lap_tu(), check(), check_sentence() (+21 more)

### Community 106 - "ByteCursor<'a>"
Cohesion: 0.33
Nodes (3): ByteCursor<'a>, Option, Self

### Community 107 - "os_linux.rs"
Cohesion: 0.06
Nodes (55): BACKEND_ORDERING, Broker, contains_khong_reset_cua_so_con_han(), Dict, en_us_neu_may_co_thi_provider_hoat_dong(), EnchantBroker, EnchantDict, EnchantSpellChecker (+47 more)

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "ObjGuard"
Cohesion: 0.33
Nodes (5): ClassFactory, ObjGuard, Default, Drop, Self

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

### Community 116 - "ipc_server.rs"
Cohesion: 0.15
Nodes (15): authorization, duration, filesystem, pipes, PSECURITY_DESCRIPTOR, read, SECURITY_ATTRIBUTES, textvn_ipc (+7 more)

### Community 119 - "ImeEngine"
Cohesion: 0.09
Nodes (25): Action, commit, pass, replace, restore, AssistIssue, FFI, ImeEngine (+17 more)

### Community 120 - "tray/src/assist.rs"
Cohesion: 0.12
Nodes (22): clean_sentence_is_silent(), decide(), enabled(), issue(), kill_switch_off_wins_over_everything(), LAST_ASSIST_TOAST_MS, long_message_is_truncated_by_chars(), notify_report() (+14 more)

### Community 121 - "edit_session.rs"
Cohesion: 0.12
Nodes (33): FixPlan, ActiveAssist, apply_assist_suggestion(), apply_display_attribute(), apply_plan(), ApplyAssistFixSession, AssistRecallSession, backspace_shift() (+25 more)

### Community 122 - "P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN"
Cohesion: 0.11
Nodes (17): 0. Tóm tắt điều hành, 10. Rủi ro & giảm thiểu (cập nhật sau vòng 3–5), 12. Việc cần làm ngay, 13. Liên kết, 1. Bối cảnh & mapping với PLAN hiện có, 2. Mục tiêu & phiêu miêu, 3.1 Quyết định kiến trúc (đã chốt), 3.2 Ràng buộc bắt buộc (+9 more)

### Community 127 - "EventTapController"
Cohesion: 0.20
Nodes (9): EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, NSObjectProtocol, TapKeyHandler, CFMachPort, CFRunLoop (+1 more)

### Community 129 - "WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN""
Cohesion: 0.33
Nodes (5): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

### Community 130 - "overlay.rs"
Cohesion: 0.09
Nodes (25): COLOR_BG, COLOR_BORDER, COLOR_ERROR, COLOR_MUTED, COLOR_SELECTED_BG, COLOR_TEXT, HINT_H, left_button_down() (+17 more)

### Community 131 - "dict.rs"
Cohesion: 0.09
Nodes (16): BridgeDict, BridgeDict<F>, Budgeted, Budgeted<'a, D>, budgeted_boc_no_dictionary_fail_open(), Budgeted<'_, D>, Dictionary, NoDictionary (+8 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.11
Nodes (26): base_keys(), f(), finish(), fix_uo(), fold(), fold_caps(), generated_key_tables_are_self_consistent(), is_word_char() (+18 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (17): 10. Sự cố 2026-10-11 — Kaspersky cách ly bản cài từ Microsoft Store (MSIX 1.2.29.0), 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5 (+9 more)

### Community 134 - "ipc/src/lib.rs"
Cohesion: 0.11
Nodes (25): read_next_message(), read_next_message(), assist_messages_round_trip(), assist_report_decodes_minimal_payload(), assist_report_round_trip_has_no_sentence_text(), AssistIssue, candidate_pipe_names(), CodecError (+17 more)

### Community 135 - "rules_win.rs"
Cohesion: 0.16
Nodes (13): address_or_search(), classify(), contains(), ControlType, known(), lower(), ProbeResult, role() (+5 more)

### Community 138 - "store_win.rs"
Cohesion: 0.10
Nodes (42): appx, commandext, OsStr, shellexecutew, LaunchMode, after_tray_start(), cmd_exe_path(), CREATE_BREAKAWAY_FROM_JOB (+34 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "common_words.rs"
Cohesion: 0.23
Nodes (22): assert_none(), check_all(), engine(), keys_for(), Fn, Method, Option, Vec (+14 more)

### Community 142 - "guids.rs"
Cohesion: 0.39
Nodes (7): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI, PROFILE_TEXTVN, GUID

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "TsfShared"
Cohesion: 0.14
Nodes (17): caret_at_composition_end(), CompSink, CompSink_Impl, EndCompositionSession, finish(), KeyEditSession, Cell, IpcClient (+9 more)

### Community 145 - "do_unregister"
Cohesion: 0.22
Nodes (14): do_unregister(), icon_path(), reg_read_string(), register_wow64_mirror(), registration_owned_by(), registration_owned_by_dir(), Scope, unregister_tip() (+6 more)

### Community 147 - ".register"
Cohesion: 0.12
Nodes (16): MacSpellProvider, Bool, MacSpellProviderTests, [0.2.12] — 2026-10-03, [0.2.15] — 2026-10-03, [0.2.5] — 2026-10-02, Added, Fixed (portable) (+8 more)

### Community 149 - "autostart.rs"
Cohesion: 0.06
Nodes (63): pcwstr, REG_SAM_FLAGS, registry, APP_RUN_VALUE_NAME, autostart_command(), autostart_disabled_by_user(), AUTOSTART_DISABLED_MARKER, autostart_disabled_marker_path() (+55 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Changelog"
Cohesion: 0.04
Nodes (56): [0.2.11] — 2026-10-03, [0.2.13] — 2026-10-03, [0.2.16] — 2026-10-03, [0.2.17] — 2026-10-03, [0.2.18] — 2026-10-03, [0.2.19] — 2026-10-04, [0.2.1] — 2026-09-30, [0.2.21] — 2026-10-04 (+48 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.09
Nodes (27): id_exit, shell, APP_INSTANCE, check_status(), flag_value(), free_ctrl_shift_cli(), HOTKEY_CTRL_DOWN, HOTKEY_OTHER_KEY_DOWN (+19 more)

### Community 154 - "menu.rs"
Cohesion: 0.10
Nodes (26): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_HEALTH_STATUS, ID_HOOK_COMPAT_MODE (+18 more)

### Community 155 - "SettingsController"
Cohesion: 0.09
Nodes (19): textvn_config, Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer (+11 more)

### Community 156 - "compose.c"
Cohesion: 0.10
Nodes (48): boundary_char(), ime_instance, lc_comp, lc_config_state, lc_key, lc_plan, file_stamp(), is_executable() (+40 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.07
Nodes (64): lc_detect_active_framework(), lc_env_check(), lc_ipc_client, client_disconnect(), get_time_ms(), handle_ipc_message(), ipc_debug_enabled(), lc_ipc_client_check_config_reload() (+56 more)

### Community 158 - "P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế"
Cohesion: 0.18
Nodes (10): 0. Kết luận 1 dòng, 1. API có sẵn — đã verify (2026-10-21), 2. So sánh "OS-first" vs "tự mang từ điển", 3. Kiến trúc provider (đề xuất), 4. Quyết định mới (bổ sung D32–D37), 5. Finding mới (Fassist-67..72) — rút ra từ nghiên cứu này, 6. Tác động lộ trình, 7. Rủi ro mới (+2 more)

### Community 159 - "OverlayView"
Cohesion: 0.15
Nodes (15): a2_lap_tu_xoa_dung_lan_lap(), cau_dai_chi_ve_duoi_va_marker_lech_dung(), issue(), IssueView, ke_hoach_a2_dung_glyph_va_giu_duoi(), khong_tu_sua_loi_chi_goi_y(), OverlayView, plan_fix() (+7 more)

### Community 160 - "SettingsDoc"
Cohesion: 0.13
Nodes (22): atomic_write(), create_private_dir(), DocKind, invalid_values_are_rejected_without_changing_the_doc(), macros_text_round_trip_through_doc(), MAX_FILE_BYTES, missing_keys_read_schema_defaults(), normalize_english_words() (+14 more)

### Community 162 - "mmap.rs"
Cohesion: 0.13
Nodes (20): CreateFileMappingW(), CreateFileW(), FILE_ATTRIBUTE_NORMAL, FILE_MAP_READ, FILE_SHARE_READ, GENERIC_READ, INVALID_HANDLE_VALUE, MAP_PRIVATE (+12 more)

### Community 163 - "developer-guide.md"
Cohesion: 0.17
Nodes (6): Audit tiếp diễn — TextVN, 2026-09-30, Giới hạn và rủi ro còn mở, Lệnh tái lập, 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

### Community 164 - "package_bootstrap.rs"
Cohesion: 0.26
Nodes (19): cleanup_old_backups(), copy_dir_recursive(), copy_overwrite(), dispatch(), installed_dir_for(), Option, Path, PathBuf (+11 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "ime_result_v1"
Cohesion: 0.14
Nodes (27): empty_ime_result(), zeroed_result(), fill_result(), ime_instance, ime_instance_new(), ime_key(), ime_key_v1, ime_last_error() (+19 more)

### Community 167 - "engine.cpp"
Cohesion: 0.05
Nodes (75): InputContext, Instance, KeyEvent, KeySym, lc_modifier, string, fcitxModifierKind(), fcitxMods() (+67 more)

### Community 168 - "IpcClient"
Cohesion: 0.06
Nodes (36): Frame, frame, needMore, violation, IpcClient, IpcClientDelegate, IpcMessage, ack (+28 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.07
Nodes (35): assist_state_update_and_snapshot(), CLIENT, GLOBAL_KEY, global_switch_from_tray_disables_every_app(), image_is_textvn_exe(), IpcClient, IpcState, LAST_HOTKEY_TOGGLE_MS (+27 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "with_ctx"
Cohesion: 0.33
Nodes (9): DialogContext, populate_controls_from_config(), refresh_if_open(), Arc, FnOnce, IpcServer, save_config_change(), show_settings_dialog() (+1 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "HWND"
Cohesion: 0.16
Nodes (39): HGDIOBJ, apply_dpi_change(), center_on_work_area(), close_macro_editor(), close_word_list_editor(), create_and_show_window(), create_control(), create_control_ex() (+31 more)

### Community 176 - "generate_tray_icons.py"
Cohesion: 0.16
Nodes (21): math, os, _centered_text(), create_us_icon(), create_vn_icon(), _draw_star(), generate_ico(), _letter_font() (+13 more)

### Community 177 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.25
Nodes (7): 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng), Targets JSON — `tools/appcomptest/targets/` (WIN-061)

### Community 178 - "spelling.rs"
Cohesion: 0.23
Nodes (16): check(), check_text(), FakeDictionary, MAX_SUGGESTIONS, MIN_WORD_CHARS, ngan_sach_suggest_het_thi_tu_la_van_duoc_bao_khong_kem_goi_y(), no_dictionary_khong_bao_gi(), Dictionary (+8 more)

### Community 179 - ".suggest"
Cohesion: 0.07
Nodes (38): Int, Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27, 0. Kết luận 1 trang, 10. Phát hiện mới (đề xuất mã — **chờ chủ `P4-TASKS` cấp số**, không tự gán `Fassist-*`) (+30 more)

### Community 180 - "ThreadState"
Cohesion: 0.09
Nodes (16): config_file_path(), EngineSession, hotkey_toggle_switches_enabled_state_and_resets_buffer(), macro_when_vi_off_runs_engine_in_passthrough(), Drop, ime_instance, Option, PathBuf (+8 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.32
Nodes (3): Sign-TextVNFile(), Write-Fail(), Write-Ok()

### Community 182 - "AppDelegate"
Cohesion: 0.05
Nodes (34): AppDelegate, .frontAppEnabled, .isVietnameseMode, .vietnameseModeEnabled, Bool, Date, Int, Int32 (+26 more)

### Community 183 - "sys"
Cohesion: 0.14
Nodes (14): Check relative markdown links trong docs/ + *.md goc. Chay local: python…, main(), tracked_iss(), Check API inject / giong malware bi cam (Farch-3). Chay local: python…, is_binary_file(), list_tracked_files(), main(), Path (+6 more)

### Community 184 - "P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN"
Cohesion: 0.15
Nodes (12): 0. Quyết định (TL;DR), 10. Liên kết, 1. Harper là gì — facts (đã xác minh, không suy đoán), 2. Kiến trúc `harper-core` (master, 2026-10-20) — bản đồ module, 3. Khoảng cách tiếng Việt — bảng "mượn được gì", 5. So sánh phương án (điểm số 0–5, cao = tốt), 6. Điều kiện nếu sau này muốn dùng Harper cho phần **tiếng Anh**, 7. Rủi ro bảo trì (đã biết trước) (+4 more)

### Community 185 - "tip.rs"
Cohesion: 0.15
Nodes (16): current_exe_name(), Default, ITfKeystrokeMgr, ObjGuard, Option, Rc, RefCell, Tip (+8 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "register_and_activate_tsf"
Cohesion: 0.18
Nodes (13): advice_for_failure(), DIALOG_CTX, read_log_tail(), register_and_activate_tsf(), register_log_path(), register_log_tail_reads_last_lines(), Mutex, Option (+5 more)

### Community 189 - "dictionary_budget.rs"
Cohesion: 0.18
Nodes (16): bo_voc_dyn_dictionary_truc_tiep_giong_ctx_dict(), bridge_dict_chuyen_tiep_dung_contains_va_suggest(), bridge_dict_voi_no_dictionary_co_tich_cac_duoc(), budget_khong_cho_phep_vuot_max_suggest(), contains_di_qua_trong_suot_va_khong_ton_ngan_sach(), CountingDictionary, dict_cham_thoi_gian_do_duoc_tang_theo_so_lan_goi(), elapsed_do_duoc_thoi_gian_dung_cho_suggest() (+8 more)

### Community 190 - "charset.rs"
Cohesion: 0.18
Nodes (13): enc(), encode(), Entry, OutputCharset, parse_pair(), push_pair(), Option, Vec (+5 more)

### Community 191 - "IpcServer"
Cohesion: 0.14
Nodes (13): IpcServer, IpcServerDelegate, Any, Bool, Data, Date, DispatchSourceRead, Int (+5 more)

### Community 192 - "invariants.rs"
Cohesion: 0.21
Nodes (15): apply(), backspace_removes_exactly_one_visible_char(), engine_never_overflows_result_or_eats_existing_text(), held_key_after_toned_word_keeps_previous_text(), key_event(), Lcg, letters(), PREFIX (+7 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - "String"
Cohesion: 0.16
Nodes (11): String, .utf16Count, Int, DiagnosticsLog, Void, EngineOwner, imk, tap (+3 more)

### Community 195 - "windows-tsf/src/assist.rs"
Cohesion: 0.06
Nodes (37): after_trigger(), assist_trigger(), AssistCheck, AssistTrigger, can_apply_fix(), CHECK_BUDGET_MS, check_sentence(), chon_loi_vong_tron_va_theo_so() (+29 more)

### Community 196 - "test_field_detect.c"
Cohesion: 0.25
Nodes (14): lc_classify_field(), str_contains_icase(), lc_utf32_to_utf8(), lc_utf8_to_utf32(), main(), test_address_bar_field(), test_candidate_field(), test_chat_field() (+6 more)

### Community 197 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 198 - "cli/src/main.rs"
Cohesion: 0.16
Nodes (14): abi_sizes(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister(), cmd_verify() (+6 more)

### Community 199 - "Security Policy"
Cohesion: 0.50
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (18): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+10 more)

### Community 201 - "SvcManager"
Cohesion: 0.12
Nodes (17): DocError, apply_option_checkbox(), effective_enabled(), persist_keeps_unknown_keys_and_reset_keeps_macros(), AtomicBool, AtomicU64, DiacriticStyle, FnOnce (+9 more)

### Community 202 - "dict/src/lib.rs"
Cohesion: 0.16
Nodes (22): builder, checksum_phat_hien_hong_data(), data_assist(), dic_dau_cu_chua_hoa_khong_chua_hoa_moi(), dic_dau_moi_chua_hoa_moi_khong_chua_hoa_cu(), dic_for_each_final_tra_tat_ca_tu(), dic_vi_dau_cu_khong_chua_tu_khong_co_trong_file(), dic_vi_dau_cu_phat_hien_tat_ca_tu() (+14 more)

### Community 203 - "trace.rs"
Cohesion: 0.18
Nodes (10): enabled(), event(), File, Mutex, Option, sink(), Arguments, Sync (+2 more)

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "ConfigStore"
Cohesion: 0.07
Nodes (29): ConfigStore, .config, Void, ActiveAlert, about, .alert, autostartFailed, .id (+21 more)

### Community 206 - "suggest"
Cohesion: 0.18
Nodes (11): enabled, $comment, default, type, enabled, suggest, additionalProperties, $comment (+3 more)

### Community 207 - "preset_matrix.rs"
Cohesion: 0.30
Nodes (11): b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace(), caps_missing_downgrades(), db(), disabled_app_passthrough(), finder_rename_editbox(), MAC_CAPS (+3 more)

### Community 208 - "ViState"
Cohesion: 0.16
Nodes (9): Bool, ViState, ViStateStore, EnableGatingTests, Bất biến mỗi target assert (không chỉ "không panic"), Chạy, fuzz — L4 (P0-4 §1), Kiểm chứng trên máy không có cargo-fuzz (+1 more)

### Community 209 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.13
Nodes (14): get_time_ms(), lc_field_detect(), 1. Nguồn `FieldContext` trên Linux, 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux) (+6 more)

### Community 210 - "e2e_ibus.c"
Cohesion: 0.22
Nodes (20): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+12 more)

### Community 211 - "generate_app_icons.py"
Cohesion: 0.33
Nodes (8): pil, gradient(), main(), pick_font(), Image, Sinh icon PNG ứng dụng TextVN từ thiết kế badge trong `resources/icons/*.svg`.…, Gradient chéo 45° khớp linearGradient 0%,0%→100%,100% (objectBoundingBox)., render()

### Community 212 - "IpcServer"
Cohesion: 0.14
Nodes (13): ClientSink, IpcServer, PipeFrame, AtomicBool, AtomicU32, File, Instant, Mutex (+5 more)

### Community 213 - "ITfEditSession_Impl"
Cohesion: 0.29
Nodes (5): ApplyAssistFixSession_Impl, AssistRecallSession_Impl, EndCompositionSession_Impl, KeyEditSession_Impl, ITfEditSession_Impl

### Community 214 - "e2e_fcitx5.py"
Cohesion: 0.15
Nodes (14): app_handles(), clear(), ctrl_shift_tap(), press_char(), pump(), Ghi như bảng cài đặt (tmp → rename); chờ để mtime chắc chắn khác lần trước., Kiểm thử đầu-cuối với fcitx5 THẬT qua DBus frontend…, send() (+6 more)

### Community 215 - "AXSnapshot"
Cohesion: 0.19
Nodes (6): AXSnapshot, FieldRole, FieldRules, Bool, UInt32, FieldRulesTests

### Community 216 - "key_event.rs"
Cohesion: 0.18
Nodes (26): is_modifier_vk(), active_modifiers(), defer_to_key_down(), eaten_label(), end_composition(), guarded(), guarded_option(), handle_key() (+18 more)

### Community 217 - "CloseHandle"
Cohesion: 0.43
Nodes (7): CloseHandle(), ensure_hook_running(), image_is_under(), process_image_path(), Path, run_elevated_and_wait(), stop_running_instance()

### Community 218 - "Json"
Cohesion: 0.27
Nodes (8): collect_preset_ids(), has_preset_id(), Json, P<'a>, Option, Result, Vec, run()

### Community 225 - "tsf-min/src/lib.rs"
Cohesion: 0.16
Nodes (18): ClassFactory, CompSink, DllCanUnloadNow(), EditSession, HR_CLASSNOTAVAILABLE, HR_E_NOTIMPL, HR_E_POINTER, HR_S_FALSE (+10 more)

### Community 226 - "test_settings_model.c"
Cohesion: 0.26
Nodes (18): tv_config_set_bool(), tv_macros_load(), tv_paths_resolve(), tv_snapshot_load(), tv_paths, main(), read_file(), test_corrupt_file() (+10 more)

### Community 227 - "ffi/src/settings.rs"
Cohesion: 0.15
Nodes (32): c(), c_api_round_trip(), cstr(), doc_err(), english_words_text_round_trip(), HEADER, ime_settings, IME_SETTINGS_CONFIG (+24 more)

### Community 228 - "charset_matrix.rs"
Cohesion: 0.27
Nodes (12): as_old_style(), charset_times_method_matrix_types_correctly(), engine(), keys_for(), Method, OutputCharset, tone_keys(), type_word() (+4 more)

### Community 229 - "foreground.rs"
Cohesion: 0.14
Nodes (17): accessibility, foregroundtracker, HWINEVENTHOOK, threading, app_id_from_image_path(), app_of_window(), ForegroundTracker, is_shell_window_class() (+9 more)

### Community 230 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.20
Nodes (9): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`), 8. CI jobs (+1 more)

### Community 231 - "Foundation"
Cohesion: 0.06
Nodes (29): Bool, InjectedMarker, Int64, IpcCodecTests, KeyMapMacTests, MarkedStateTests, MarkedStateUnitTests, NativeBoundaryTests (+21 more)

### Community 232 - "FieldDetect"
Cohesion: 0.08
Nodes (24): Entry, FieldContext, FieldDetect, Date, pid_t, Set, TimeInterval, Fixed — hoàn tất roadmap audit 3 nền tảng (A6–A16, B6–B14, C1–C9) (+16 more)

### Community 233 - "Path"
Cohesion: 0.19
Nodes (15): append_log_line(), is_reparse_link(), log_rotates_when_larger_than_limit(), log_writer_checks_reparse_before_creating_dirs(), machine_scope_dll(), prepare_log_file(), program_files_roots(), resolve_dll_path() (+7 more)

### Community 234 - "TapPermission"
Cohesion: 0.22
Nodes (7): NSLock, Bool, TimeInterval, Void, TapPermission, ApplicationServices, Timer

### Community 235 - "P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`"
Cohesion: 0.12
Nodes (16): 1. Công cụ đo: `textvn-cli assist score --corpus <dir>`, 2.1 Vòng 2 (2026-10-21) — đo trên **văn bản thật** (đóng F80), 2.2 Vòng 3 (2026-10-21) — mở rộng luật ngữ pháp + `--strict` / `--per-kind`, 2.3 Vong 4 (2026-10-21) - do tren **1.500 cau that** (Tatoeba) -> 4 loi engine that, 2.4 Tu dien FST (hunspell-vi) — ket qua tra cuy (P4-4 AS-040/AS-041), 2. Kết quả đo (2026-10-21), 3. Văn bản sạch dùng để đo FPR — `corpus/assist/vi_clean_synthetic.ass`, 4.1 Finding vòng 2 (SA/QA) — phát hiện nhờ đo trên văn bản thật (+8 more)

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.25
Nodes (8): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor, Sự cố đã gặp khi phát hành — tra trước khi xử lý

### Community 237 - "properties"
Cohesion: 0.11
Nodes (18): properties, $comment, default, enum, $comment, default, enum, $comment (+10 more)

### Community 238 - "Hướng dẫn sử dụng TextVN"
Cohesion: 0.15
Nodes (13): 1. Cài đặt, 2. Bật/tắt tiếng Việt, 3. Kiểu gõ, 4. Bảng điều khiển, 5. Gõ tắt, 6. Cấu hình, 7. Xử lý sự cố, Cài đặt im lặng (cho quản trị / Microsoft Store) (+5 more)

### Community 239 - "keymap_mac.rs"
Cohesion: 0.20
Nodes (4): MAC_KEY_COUNT, mac_to_canonical(), Option, modifier_dung_vk_is_modifier_nhan_duoc()

### Community 240 - "quick_telex.rs"
Cohesion: 0.24
Nodes (6): apply, apply(), expansion(), q(), Option, Vec

### Community 241 - "Strategy"
Cohesion: 0.19
Nodes (16): default_for_field, downgrade(), downgrade_without_cap(), inp(), resolve(), ResolveInput, Option, secure_field_role_passthrough() (+8 more)

### Community 242 - "Hướng dẫn phát triển TextVN"
Cohesion: 0.20
Nodes (10): 1. Kiến trúc, 2. Dựng, 3. Kiểm thử, 4. Quy ước bắt buộc (CI chặn), 5. Phát hành, 6. Tài liệu liên quan, Hướng dẫn phát triển TextVN, Linux (Ubuntu/Debian) (+2 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.03
Nodes (58): 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển, 0.2.14 — DPI gốc + UI thống nhất + common errors, 1. Gói phát hành, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Ba nguyên nhân gốc, ba fixes, xác minh máy thật (+50 more)

### Community 244 - "remove_added_languages"
Cohesion: 0.18
Nodes (18): added_language_removable(), broadcast_international(), ctrl_shift_taken_by_windows(), is_input_method_value(), languages_added_marker_path(), languages_missing(), languages_without(), parse_languages_marker() (+10 more)

### Community 245 - "Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`)"
Cohesion: 0.33
Nodes (5): Báo cáo audit 3 nền tảng — TextVN v0.2.29 (HEAD `2fa6103`), D. ƯU TIÊN XỬ LÝ ĐỀ XUẤT (top 10, ghép 3 platform), E. Ghi chú quy trình, F. Rà lại các bản sửa sau 0.2.31 (2026-10-11) — sửa ở 0.2.32, Tổng quan

### Community 246 - "Dictionary"
Cohesion: 0.23
Nodes (5): Dictionary, EnchantSpellChecker, MacSpellChecker, Vec, WindowsSpellChecker

### Community 247 - "viqr.rs"
Cohesion: 0.15
Nodes (8): apply_key, fold(), marker_to_tone(), n(), DiacriticStyle, Option, Vec, viqr_as_keys

### Community 248 - "TextVNAppTests"
Cohesion: 0.06
Nodes (21): Data, TextVNAppTests, 0. Đọc trước khi làm bất cứ việc gì (trình tự), 1. Luật không được phá (vi phạm = task chưa xong), 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự, 3. Kiến trúc & ràng buộc kỹ thuật cốt lõi, AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN, Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging) (+13 more)

### Community 249 - "ffi_key.rs"
Cohesion: 0.12
Nodes (13): cstr, STRATEGY_MAX_ID, apply_doc(), assert_result_sane(), ByteCursor, MAX_KEYS, VK_BACK, VK_RETURN (+5 more)

### Community 250 - "smoke-imk.sh"
Cohesion: 0.48
Nodes (5): clear_doc(), read_doc(), say(), smoke-imk.sh script, type_keys()

### Community 251 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 252 - "Kế hoạch ký số (code signing) — TextVN"
Cohesion: 0.20
Nodes (10): 1. So sánh các đường chính thống (cập nhật 2026-10-02), 2. Lộ trình đề xuất — SignPath Foundation, 3. Smart App Control / SmartScreen — kỳ vọng thực tế, 4. Việc đã wire trong repo, Bước 1 — Nộp đơn (làm một lần, maintainer tự làm), Bước 2 — Tạo project + artifact configuration trên SignPath.io, Bước 3 — Bật ký trong GitHub Actions (đã nối sẵn — chỉ cần đặt secret), Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai) (+2 more)

### Community 253 - "generate_store_art.py"
Cohesion: 0.21
Nodes (13): A16. [THẤP] `compose`/`replay` — (đã soát, chỉ ghi nhận) replay post đúng, compose(), diag_gradient(), draw_star(), load_font(), main(), FreeTypeFont, Image (+5 more)

### Community 254 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.13
Nodes (15): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release (+7 more)

### Community 255 - "test-typing.ps1"
Cohesion: 0.36
Nodes (4): Check(), Clear-Verified(), Codes(), Norm()

### Community 256 - "env-mac — Môi trường build macOS (MAC-001)"
Cohesion: 0.40
Nodes (4): Acceptance MAC-001, env-mac — Môi trường build macOS (MAC-001), Lệnh kiểm nhanh, Yêu cầu

### Community 257 - "A. WINDOWS"
Cohesion: 0.12
Nodes (17): A10. [TRUNG] COM callback `ActivateEx`/`Deactivate`/`OnSetFocus` không bọc `catch_unwind` (tip.rs:90-124,174-183) — panic thoát qua ABI = abort host (Notepad/Chrome). Nhánh `try_borrow_mut` fail tại ActivateEx: Advise đã thực hiện nhưng cookie không lưu → Deactivate không Unadvise (leak tới hết thread). **Fix:** catch_unwind như `DoEditSession`; borrow-fail trả `Err(E_FAIL)`., A11. [TRUNG] `svc.rs:135-235` — `RwLock.write().unwrap()`: poison → panic dây chuyền trong `wnd_proc`. **Fix:** `unwrap_or_else(|e| e.into_inner())`., A12. [TRUNG] Uninstaller per-user non-elevated: `schedule-delete` (PFO) luôn ACCESS_DENIED → `.old-*` sót vĩnh viễn (register.rs:793-834). **Fix:** RunOnce như uninstall.ps1 portable đã làm., A13. [THẤP] Đa user với bản máy: user B còn per-user override trỏ DLL đã xoá; CLI cũng bị xoá → không tự lành (main.rs:601-603 return sớm). Fix dài hạn: marker machine-level yêu cầu tray user B tự unregister khi thấy path chết., A14. [THẤP] `reg_value_names` buffer 512 u16 — value dài hơn 511 ký tự, A15. [THẤP] `show_activation_refused_notice_once` (main.rs:337-351) ghi marker, A1. [CAO] ILOT spec cho bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100) 🔎, A2. [CAO] Mirror WOW64 CTF cho **HKCU** là đường registry chết 🔎 (+9 more)

### Community 258 - "Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)"
Cohesion: 0.25
Nodes (8): 1. Bố cục, 2. Tuỳ chọn ↔ khoá cấu hình, 3. Hành vi chung, 4. Cửa sổ Gõ tắt, 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+), 6. Nhận diện EN/VI (không có UI riêng — hành vi engine), 7. Icon — một thiết kế cho 3 nền tảng, Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)

### Community 259 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 260 - "tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)"
Cohesion: 0.40
Nodes (4): 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật (cập nhật 2026-10-09), tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

### Community 262 - "cli/src/assist.rs"
Cohesion: 0.20
Nodes (12): cmd_assist(), cmd_score(), cmd_show(), issue_json(), MAX_PRINT_DEFAULT, print_human(), Option, run_file() (+4 more)

### Community 263 - "4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)"
Cohesion: 0.14
Nodes (14): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release, Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF (+6 more)

### Community 264 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - "2. Windows — Spell Checking API (`spellcheck.h`, Win8+)"
Cohesion: 0.12
Nodes (15): 2.1 Kiểm tra **thật** crate `windows` 0.61.3 (đã grep registry cargo), 2.2 Chuỗi gọi API (đúng thứ tự, đúng kiểu), 2.3 COM activation & apartment (rủi ro thật), 2.4 Đơn vị offset — **đo thực nghiệm**, tài liệu MS **không** nói, 2.5 Số đo thực nghiệm (để đặt ngân sách, không phải benchmark chính thức), 2.6 Rủi ro riêng của Windows, 2.7 Code sketch (Rust — **sketch trong doc**, không phải code sản phẩm), 2. Windows — Spell Checking API (`spellcheck.h`, Win8+) (+7 more)

### Community 269 - "[0.2.32] — 2026-10-12"
Cohesion: 0.40
Nodes (5): [0.2.32] — 2026-10-12, Changed, Fixed, Known issues, Security / quy trình

### Community 270 - "default"
Cohesion: 0.13
Nodes (15): additionalProperties, $comment, default, type, auto_fix, display, language, max_sentence_chars (+7 more)

### Community 274 - "mac_corpus_cases.rs"
Cohesion: 0.49
Nodes (9): all_cases(), bs(), case(), CorpusCase, du_nhom_case_theo_p2_5(), imk(), Vec, tap() (+1 more)

### Community 291 - "ConfigWatcher"
Cohesion: 0.18
Nodes (8): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem, Notification

### Community 292 - "path"
Cohesion: 0.13
Nodes (15): find_rc_exe(), main(), Option, PathBuf, all_cases, find_rc_exe(), main(), Option (+7 more)

### Community 293 - "textvn_ibus_engine.h"
Cohesion: 0.20
Nodes (8): lc_log_close(), bus_disconnected(), gpointer, main(), make_component(), ibus, IBusBus, IBusComponent

### Community 294 - "command"
Cohesion: 0.15
Nodes (14): find_rc_exe(), main(), Option, PathBuf, command, find_rc_exe(), main(), Option (+6 more)

### Community 295 - "check_version_sync.rs"
Cohesion: 0.22
Nodes (12): check_site(), extract(), PLIST_KEY, repo_hien_tai_dong_bo(), Option, Result, run(), Site (+4 more)

### Community 296 - "IMKTextTarget"
Cohesion: 0.20
Nodes (5): IMKTextTarget, Bool, Int, TextCommandTarget, IMKTextInput

### Community 297 - "linux-fcitx5/src/engine.h"
Cohesion: 0.20
Nodes (10): Instance, TextVNEngineFactory, create, AddonFactory, AddonInstance, AddonManager, cstdint, InputContextProperty (+2 more)

### Community 298 - "P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công"
Cohesion: 0.13
Nodes (14): 0. Chuỗi phụ thuộc (đã xong / sắp làm), 1.1 API công khai (đề xuất, khớp D23/D27/D29), 1.2 Quy tắc bắt buộc, 1.3 Chỗ sẽ sửa (đã đọc trong repo), 1.4 Kiểm chứng, 1. AS-021 — FFI `ime_assist_*` (C-ABI), 2.1 Luồng, 2.2 Quy tắc hành vi (D11 + D12) (+6 more)

### Community 299 - "idle_ms"
Cohesion: 0.14
Nodes (14): properties, $comment, default, minimum, type, idle_ms, show_on_comma, show_on_sentence_end (+6 more)

### Community 301 - "run_tray_app"
Cohesion: 0.33
Nodes (11): add_tray_icon(), copy_to_wide_buf(), HWND, LPARAM, LRESULT, WPARAM, run_tray_app(), spawn_activate_profile() (+3 more)

### Community 302 - "map_file"
Cohesion: 0.15
Nodes (13): file_len(), map_file(), MapError, Mapping, Display, Drop, Error, Formatter (+5 more)

### Community 303 - ".start"
Cohesion: 0.18
Nodes (9): CGFlagMapper, Bool, UInt16, UInt32, TapTranslator, CGEventFlags, CGEventTapLocation, Fixed (macOS — tiếp) (+1 more)

### Community 304 - "3. Nhật ký finding"
Cohesion: 0.12
Nodes (15): 0. Kết luận, 1. Phương pháp, 2. Bằng chứng CI, 3.1 Microsoft Store / MSIX, 3.2 Bảo mật Windows, 3.3 QA Windows (tray, CLI, bộ cài), 3.5 Tiếng Việt / engine, 3.6 CI, ký số, phát hành (+7 more)

### Community 305 - "capitalize_sentence.rs"
Cohesion: 0.38
Nodes (4): bat_dau_cau_thuong(), check(), is_sentence_start(), Vec

### Community 306 - "class.rs"
Cohesion: 0.28
Nodes (8): HR_CLASS_E_NOAGGREGATION, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, AtomicI32, HRESULT, atomic, com

### Community 307 - "verify-store-package.sh"
Cohesion: 0.60
Nodes (4): die(), GNUPGHOME, verify-store-package.sh script, valid_sig()

### Community 308 - "Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)"
Cohesion: 0.29
Nodes (7): Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon), Icon — trả lời câu hỏi "Linux/macOS đã có icon như Windows chưa?", Kiểm chứng chạy trên host Windows (2026-10-01, sau toàn bộ thay đổi), Phát hành — v0.2.3 đã publish (release candidate, chưa ký số), Quy trình phát hành mới, Sửa theo audit vòng 1 — GitHub Actions (5 major + minors), Sửa theo audit vòng 1 — đăng ký TSF (3 major + 7 minor)

### Community 309 - "EngineOptions"
Cohesion: 0.14
Nodes (10): Context, disabled_all_pass(), EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset (+2 more)

### Community 310 - "ime_assist_issue"
Cohesion: 0.22
Nodes (9): ime_assist_issue, auto_fixable, confidence, end, kind, message, start, suggest (+1 more)

### Community 311 - "B. macOS"
Cohesion: 0.18
Nodes (11): B10. [THẤP] `IMKTextTarget.swift:40-42` — `NSUnderlineStyle([]).rawValue` = 0, B11. [THẤP] Dead code/state chết: `heldMods` (chỉ ghi), `noteFlagsChanged()`, B13. [THẤP] EnglishWordsSheet filter âm thầm ("Café"→"caf") + không cap 64;, B14. [THẤP] Test gap: không có test apply `replace` với `strategy: .backspaceType`, B4. [TRUNG] Crash-report pipeline đứt hoàn toàn, B6. [TRUNG] Double-click TextVN.app khi autostart bật = im lặng, B7. [TRUNG] gatherAX không dedupe in-flight + không AX messaging timeout, B8. [TRUNG] `IpcServer.appStates` tăng vô hạn qua ToggleViEn app_id tuỳ ý (+3 more)

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

### Community 317 - ".onEvent"
Cohesion: 0.20
Nodes (7): UnsafeMutableRawPointer, TapCallback, CGEvent, CGEventTapProxy, CGEventType, Double, Unmanaged

### Community 318 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.15
Nodes (13): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG (+5 more)

### Community 319 - "textvn_ibus_modifier_kind"
Cohesion: 0.29
Nodes (9): guint, lc_modifier, textvn_ibus_map_key(), textvn_ibus_modifier_kind(), textvn_ibus_mods(), guint, lc_key, classify() (+1 more)

### Community 320 - "2. Kết quả kiểm thử"
Cohesion: 0.50
Nodes (4): 2. Kết quả kiểm thử, Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng), Kịch bản 1 — Cài đặt, Kịch bản 2 — Giải nén ra dùng luôn

### Community 321 - "macSpellContainsCallback"
Cohesion: 0.44
Nodes (8): macSpellContainsCallback(), macSpellSuggestCallback(), CChar, Int32, UInt32, UnsafeMutableRawPointer, UnsafePointer, UnsafeMutablePointer

### Community 322 - "Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)"
Cohesion: 0.05
Nodes (36): 0. Trạng thái — ĐÃ PHÁT HÀNH trên Microsoft Store (2026-10-11), 1. Vì sao gói MSIX của TextVN có cơ chế "stage-out", 2. Partner Center — việc của chủ tài khoản, 3. Build gói nộp Store, 4. Nộp submission, 5. Rủi ro còn lại & cách xử lý, 6. Sau khi publish, 7.1 Workflow làm gì (+28 more)

### Community 323 - "Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành"
Cohesion: 0.50
Nodes (4): Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành, Gate trước tag 0.2.4, Tiếp tục audit ZCode — engine (2026-10-01), Tiếp tục audit ZCode — runtime chưa có bằng chứng production

### Community 324 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, delete_count, flags, insert, insert_len, preedit (+2 more)

### Community 325 - "Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent"
Cohesion: 0.50
Nodes (4): Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent, Giải pháp + bằng chứng kiểm thử, Phát hành — v0.2.8 đã publish (release candidate, chưa ký số), Sự cố trong phiên — tra trước khi lặp lại

### Community 327 - "load"
Cohesion: 0.29
Nodes (10): AsRef, GLOBAL, load(), os_provider(), Arc, OnceLock, Option, Path (+2 more)

### Community 328 - "Option"
Cohesion: 0.29
Nodes (10): apply_ctrl_shift_default_once(), ensure_tsf_tip_registered(), machine_registration_present(), read_registry_dword(), read_registry_string(), registry_key_exists(), HKEY, Option (+2 more)

### Community 329 - "MockHandler"
Cohesion: 0.31
Nodes (4): MockHandler, SelfDisableTests, Bool, TextVNTap

### Community 330 - "Diagnostics"
Cohesion: 0.07
Nodes (20): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, UInt64, KeyMapMacGenerated (+12 more)

### Community 331 - "simple_telex.rs"
Cohesion: 0.38
Nodes (4): fold(), n(), DiacriticStyle, Vec

### Community 332 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, CLSID_TEXTVN_TIP, DllGetClassObject(), PROFILE_GUID, c_void, GUID, IClassFactory_Impl, IUnknown

### Community 333 - "ime_assist_issue"
Cohesion: 0.22
Nodes (9): ime_assist_issue, auto_fixable, confidence, end, kind, message, start, suggest (+1 more)

### Community 334 - "verify-msix.py"
Cohesion: 0.15
Nodes (13): re, struct, imports(), main(), check-pe-imports.py — chặn binary Windows phụ thuộc VC++ runtime (R2-18). TIP…, check(), main(), map_version() (+5 more)

### Community 335 - "diacritic_style"
Cohesion: 0.50
Nodes (4): $comment, default, enum, diacritic_style

### Community 336 - "CTextVNFFI/include/textvn_ffi.h"
Cohesion: 0.09
Nodes (22): ime_assist_result, abi_version, count, issues, ime_context, abi_version, app_id, caps (+14 more)

### Community 337 - "do_register"
Cohesion: 0.15
Nodes (26): activate_for_session(), activate_tip(), active_profile_is_textvn(), com_init(), ctf_category_keys(), ctf_layout_matches_tsf_registry_schema(), ctf_profile_key(), ctf_tip_key() (+18 more)

### Community 338 - "3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft"
Cohesion: 0.18
Nodes (11): 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft, 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối, 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store, 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation, 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft, Gốc rễ 1: Sự ngộ nhận tai hại về HKCU (Per-User) vs HKLM (Per-Machine) trong môi trường tự động của Store, Gốc rễ 2: Lệch chuỗi định danh tuyệt đối (Exact String Mismatch) giữa Partner Center và Registry, Gốc rễ 3: Điểm mù của CI Test Harness hiện tại (`validate-store-package.ps1` & `simulate-store-validation.ps1`) (+3 more)

### Community 339 - "handle_overlay_key"
Cohesion: 0.33
Nodes (8): select_assist_issue(), selected_assist_issue(), handle_overlay_key(), KeySink, KeyTraceSink, ObjGuard, Rc, Self

### Community 340 - "linux-settings/src/main.c"
Cohesion: 0.33
Nodes (4): gpointer, GtkApplication, on_app_activate(), gtk

### Community 341 - "IMPLEMENTATION STATUS — adapter macOS (Farch-4)"
Cohesion: 0.22
Nodes (7): TapMarkerTests, Còn mở (2026-10-09), IMPLEMENTATION STATUS — adapter macOS (Farch-4), Quyết định kỹ thuật đã chốt trong code (ghi lại cho review), Tiếp theo (tuần tự), Trạng thái (2026-10-09), Đã triển khai (bảng thành phần)

### Community 342 - "HookEngine"
Cohesion: 0.33
Nodes (5): HookEngine, Drop, ime_instance, Result, Self

### Community 343 - ".with"
Cohesion: 0.22
Nodes (7): ITfKeyEventSink, FnOnce, ITfKeystrokeMgr, ITfTextInputProcessor_Impl, Option, Tip_Impl, TipInner

### Community 344 - "package_family_full_names"
Cohesion: 0.24
Nodes (10): MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, PWSTR, message_box(), open_apps_settings(), package_family_full_names(), report_error(), Vec (+2 more)

### Community 345 - "[0.2.0] — 2026-09-30"
Cohesion: 0.22
Nodes (9): [0.2.0] — 2026-09-30, Added, Added (macOS — Farch-4), Added (trước đó), Changed, Fixed, Fixed, Fixed (review vòng 3 — trước tag v0.2.0) (+1 more)

### Community 346 - "3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 347 - "TextVN — Chỉ mục tài liệu & Giao thức Review"
Cohesion: 0.29
Nodes (7): 0. Tài liệu cho người dùng & phát hành, 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 348 - "3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 349 - "Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store"
Cohesion: 0.17
Nodes (12): 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào, 2. Vòng 15 — những gì đã triển khai trong `release.yml`, 3. Khoá ký GPG release (TextVN Release Signing), 4. Authenticode cho Microsoft Store (chưa xong — đang chờ), 4b. macOS — Developer ID + notarization, 5. Quy tắc khóa-tay (đừng lặp), 6.1. Sự cố & Nguyên nhân gốc rễ (Sự cố R6 — 2026-10-11), 6.2. Quy tắc bắt buộc cho mọi release từ v0.2.32 (+4 more)

### Community 350 - "keys_generated.rs"
Cohesion: 0.19
Nodes (12): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+4 more)

### Community 351 - "core/src/lib.rs"
Cohesion: 0.08
Nodes (37): ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, apply(), backspaces(), boundary_space_commits_with_preedit_cap(), boundary_space_pass_with_default_caps() (+29 more)

### Community 352 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 353 - "P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow"
Cohesion: 0.25
Nodes (7): 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng), 2. Trách nhiệm từng crate & ranh giới, 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa), 4. CI (GitHub Actions), 5. Versioning & branching, 6. Tham chiếu ngược, P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow

### Community 354 - "Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI"
Cohesion: 0.25
Nodes (7): 1. Microsoft nói chính xác điều gì, 2. Điều này thay đổi nhận định gì, 3. Ba con đường (đối chiếu chi phí / rủi ro / thời gian), 4. Kế hoạch chốt (theo thứ tự ưu tiên — cập nhật 2026-10-09), 5. Hiện trạng kỹ thuật đã sẵn sàng (2026-10-09), 6. Việc cần làm — checklist, Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

### Community 355 - "caps.rs"
Cohesion: 0.31
Nodes (4): is_closer(), is_sentence_end(), sentence_start(), starts()

### Community 357 - "3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM""
Cohesion: 0.29
Nodes (7): 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store, 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM", 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM", 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant), 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên, Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`, Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC

### Community 358 - "T6 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): LNX-060 · AT-SPI driver `tools/linux/atspi-driver` (L) — `P3-6 §4`, LNX-061 · Targets JSON 12 app (cùng format 3 OS) (M), LNX-062 · Perf bench + `perf/baseline-linux.json` (M) — `P3-6 §5`, LNX-063 · Soak 24h script × 2 (X11 + Wayland VM) (M), LNX-064 · CI jobs linux + dist matrix (M) — `P3-6 §8`, LNX-065 · Fuzz + ASan/valgrind job (S), LNX-066 · Security + license checklist Phần 3 (M) — Handbook §8 + audit x11 (P3-3 §7), T6 — Test & hardening (tuần 15–18)

### Community 359 - ".inject"
Cohesion: 0.50
Nodes (5): Bool, CGEventSource, Int, UInt16, TapInjector

### Community 360 - "Problem"
Cohesion: 0.09
Nodes (17): MacroRules, Problem, emptyExpansion, emptyTrigger, expansionTooLong, .message, triggerHasSpace, triggerTooLong (+9 more)

### Community 361 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.33
Nodes (6): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust

### Community 362 - "[0.2.10] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.10] — 2026-10-03, Documented, Fixed, Tests

### Community 363 - "3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)"
Cohesion: 0.33
Nodes (6): 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu, 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions), 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions), 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết, Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE), Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)

### Community 364 - "P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A"
Cohesion: 0.22
Nodes (8): 0. Kết luận của agent A sau khi verify lại từng fact, 1. Disposition 6 finding của agent B, 2. Finding mới của agent A (Fassist-38..45) — phát hiện khi review lại đề xuất của B, 3. Quyết định mới D21–D29 (bổ sung D1–D20), 4. Lộ trình cập nhật (chèn vào `P4-0` §8), 5. Trạng thái finding sau cross-review, 6. Liên kết, P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A

### Community 365 - "gen_corpus_tatoeba.py"
Cohesion: 0.27
Nodes (9): argparse, bz2, io, clean_sentence(), fetch(), is_vietnamese_heavy(), main(), Sinh corpus CAU SACH tu Tatoeba (cau tieng Viet tu nhien) - P4-0 D19. Nguon:… (+1 more)

### Community 366 - "[0.2.28] — 2026-10-10"
Cohesion: 0.40
Nodes (5): [0.2.28] — 2026-10-10, Added, Fixed — kiểm trên Windows 11 thật và rà soát đối kháng (R2-88…R2-98, chi tiết: `docs/release/code-review-2026-10-09-round2.md` §3.7), Fixed — rà soát mã nguồn chuyên sâu 2026-10-07 (chi tiết: `docs/release/code-review-2026-10-07.md`), Fixed — rà soát vòng 2: Microsoft Store, ký số, QA (chi tiết: `docs/release/code-review-2026-10-09-round2.md`)

### Community 367 - "mask.rs"
Cohesion: 0.36
Nodes (7): che_email(), che_url(), expand_non_space(), expand_word(), is_word_char(), masked_ranges(), Vec

### Community 368 - "[0.2.14] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.14] — 2026-10-03, Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau"), Fixed (Windows), Tests

### Community 370 - "ffi/include/textvn_ffi.h"
Cohesion: 0.22
Nodes (8): ime_assist_result, abi_version, count, issues, ime_instance, ime_os_spell, abi_version, ctx

### Community 371 - "OutputStrategy"
Cohesion: 0.29
Nodes (7): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, Int64

### Community 373 - "P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15"
Cohesion: 0.25
Nodes (7): 0. Kết quả đối chiếu với handoff, 1. Vòng 16 — SA/QA, 2. Vòng 17 — Audit (license/privacy/AV), 3. Vòng 18 — PO/UX, 4.1 Bổ sung (2026-10-21, sau khi CI báo đỏ), 4. Kết luận, P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15

### Community 374 - "DictError"
Cohesion: 0.22
Nodes (7): DictError, Display, Error, Formatter, From, Result, Self

### Community 376 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 377 - "Phân phối Linux — kênh, store và auto-update"
Cohesion: 0.10
Nodes (19): 0. Nguyên tắc (khác gì PPA), 1. Lộ trình ba bước, 2. Việc upstream đã chuẩn bị (bằng chứng cho maintainer), Bước A — RFP (nhờ cộng đồng đóng gói) hoặc ITP (tự đóng gói rồi xin sponsor), Bước B — mentors.debian.net upload (kèm ITP), Bước C — Liên hệ Debian Input Method Team (nhanh nhất), Nộp TextVN vào Debian / Ubuntu chính thức (RFP → ITP → Sponsor), 1. Ràng buộc quyết định kênh: TextVN là IME hệ thống (+11 more)

### Community 378 - "TapDecision"
Cohesion: 0.25
Nodes (6): Int, TapDecision, pass, transform, CGEventSource, UInt32

### Community 380 - "Action"
Cohesion: 0.25
Nodes (3): Action, ActionKind, Option

### Community 381 - "11. Audit"
Cohesion: 0.25
Nodes (8): 11.1 Vòng 2 — "Nhất quán & Sẵn sàng" (đã xong ở v1), 11.2 Vòng 3–5 — review chuyên sâu (2026-10-20, 3 lượt: SA/QA → Audit → PO/BA/Design), 11.3 Vòng 6 — CROSS-REVIEW (agent B, 2026-10-20) + phản hồi agent A, 11.4 Vòng 7–9 — review **sau khi implement P4-1** (2026-10-20), 11.5 Kết luận các lượt review, 11.6 Vòng 8–10 — review **sau khi làm 4 spike** (2026-10-20) → `P4-3-SPIKES.md`, 11.7 Vòng 11 — review **bản đồ 13 call site** (chuẩn bị P4-2a, 2026-10-20), 11. Audit

### Community 384 - "corpus.rs"
Cohesion: 0.29
Nodes (8): corpus_co_du_lop_hieu_biet(), corpus_files(), corpus_vi_basic_passes(), parse_corpus(), Path, PathBuf, Vec, Expect

### Community 385 - "P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4"
Cohesion: 0.20
Nodes (9): 0. Tóm tắt kết luận, 1. S1 — TSF candidate list sau commit, 2. S2 — Đọc vị trí caret (precondition D12), 3. S3 — Nguồn từ điển tiếng Việt, 4. S4a — Chi phí per-process (đo thật, Windows x64), 5. S4b — `harper-core` cho tiếng Anh (v2, không chặn v1), 6. Quyết định mới từ spike, 7. Việc còn lại sau 4 spike (+1 more)

### Community 386 - "`docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)"
Cohesion: 0.25
Nodes (7): 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate, `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

### Community 387 - "AppInfo"
Cohesion: 0.29
Nodes (6): AppInfo, .aboutMessage, .aboutTitle, Round 8.2 — Audit vòng 2 (final gate, verdict PASS), Round 8 — Audit phát hành 0.2.0 (2026-09-30, 2 vòng cuốn chiếu), Điều kiện production còn lại (bằng chứng bắt buộc)

### Community 388 - "diacritic_typo.rs"
Cohesion: 0.38
Nodes (5): bat_sai_dau(), check(), giu_hoa_thuong(), Vec, TYPOS

### Community 389 - "4. Đánh giá theo vai trò chuyên trách"
Cohesion: 0.29
Nodes (7): 4.1 PO (Product Owner), 4.2 BA (Business Analyst), 4.3 SA (Solution Architect), 4.4 QA, 4.5 Audit (license / security / privacy), 4.6 Design (UX), 4. Đánh giá theo vai trò chuyên trách

### Community 390 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.29
Nodes (7): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust, Phát hiện W-06: Điểm mù kiến trúc 32-bit WOW64 — Zalo PC và toàn bộ ứng dụng x86 không thể nạp TSF DLL 64-bit

### Community 395 - "offer_machine_registration_if_needed"
Cohesion: 0.33
Nodes (7): PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-09, sau v0.2.27 + vòng rà soát 2), elevated_register_succeeded(), is_process_elevated(), offer_machine_registration_if_needed(), show_activation_refused_notice_once(), show_store_activation_notice_once(), show_untrusted_path_notice_once()

### Community 396 - "on_dir_changed"
Cohesion: 0.40
Nodes (6): gboolean, is_settings_file(), on_dir_changed(), GFile, GFileMonitor, GFileMonitorEvent

### Community 397 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 399 - "suggest_vn.rs"
Cohesion: 0.25
Nodes (5): complete_vn_word(), Option, v(), VN_FREQ_DATA, is_valid_word

### Community 400 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.07
Nodes (27): 0. Executive Summary, 4.1 Ngôn ngữ & технологies (quyết định), 4.2 Engine pipeline (kế thừa + mở rộng 7-stage của gonhanh), 4.3 Test strategy (không thể thiếu — đây là chỗ 7 dự án yếu), 4.4 Quy trình, 4. ENGINEER VIEW, 5.1 Chiến lược "sống sót qua OS update" (không sniff version, **capability detection**), 5.2 Env compatibility (được yêu cầu đặc biệt) (+19 more)

### Community 401 - "3. SOLUTION ARCHITECT VIEW"
Cohesion: 0.25
Nodes (8): 3.1 C4 — Context, 3.2 C4 — Containers (Monorepo) — *bản layout chính thức đã chốt ở `docs/10-shared/P0-1-repo-and-workflow.md`; dưới đây là bản rút gọn*, 3.3 Hợp đồng FFI v1 — **bản chính thức đã chốt: `docs/10-shared/P0-2-engine-ffi-contract.md`**, 3.4 Output Strategy Engine (trái tim của app-compat), 3.5 Config & state, 3.6 UI/Tray (1 codebase, 3 native shell), 3.7 Packaging & Distribution, 3. SOLUTION ARCHITECT VIEW

### Community 402 - "[0.2.34] — 2026-10-18"
Cohesion: 0.50
Nodes (4): [0.2.34] — 2026-10-18, Added, Docs, Fixed

### Community 403 - "display"
Cohesion: 0.25
Nodes (8): idle_ms, show_on_comma, show_on_sentence_end, additionalProperties, $comment, default, type, display

### Community 404 - "4. Hai chế độ — đặc tả chi tiết (sau vòng 3–5)"
Cohesion: 0.33
Nodes (6): 4.1 Điểm kiểm tra vs điểm hiển thị (D11), 4.2 Mode 1 — "Trợ lý ngữ pháp: gợi ý" (P4-1…P4-3), 4.3 Mode 2 — "tự sửa cả câu" (P4-5), 4.4 Ba mức áp dụng (thay cho "sửa bằng selection replace" — rủi ro map vị trí), 4.5 Precondition (D12) — chi tiết, 4. Hai chế độ — đặc tả chi tiết (sau vòng 3–5)

### Community 405 - "P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả"
Cohesion: 0.25
Nodes (7): 0. Kết quả, 1. Thiết kế (vòng 1 — SA), 2. Vòng 2 — Audit (license / privacy / AV), 3. Vòng 3 — PO/QA (chất lượng & rủi ro), 4. Bằng chứng (để agent sau không phải đo lại), 5. Còn lại cho P4-2, P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả

### Community 406 - "icons.rs"
Cohesion: 0.29
Nodes (6): HICON, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V, load_app_icon(), HINSTANCE

### Community 407 - "3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?"
Cohesion: 0.33
Nodes (6): 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU), 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?, 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?, 3. Lỗ hổng định danh ký tự: Nguy cơ lệch Codepoint nghiêm trọng giữa "Linh βùi" và Partner Center, 4. Nghịch lý bế tắc (Catch-22) của gói Raw Win32 EXE trên Microsoft Store, 5. Khuyến nghị dứt điểm: Dừng nộp file `.exe` và chuyển ngay sang nộp gói MSIX Full-Trust

### Community 408 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.33
Nodes (5): C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 409 - "ffi/build.rs"
Cohesion: 0.50
Nodes (4): find_rc_exe(), main(), Option, PathBuf

### Community 410 - "StageInfo"
Cohesion: 0.29
Nodes (6): stage_json_roundtrip_and_validation(), StageInfo, launch_package_app(), Result, spawn_detached(), write_stage()

### Community 411 - "P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)"
Cohesion: 0.33
Nodes (5): Chi tiết P4-1 (đã xong), P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist), Recipe P4-2a — refactor buffer trong `core` (đọc trước khi code), Task mở (chi tiết acceptance & phân công Agent), Trạng thái tổng quan

### Community 413 - "ApplyError"
Cohesion: 0.29
Nodes (7): ApplyError, cannotDelete, rejected, KeyTranslatorError, noChar, noLayout, Error

### Community 414 - "ime_suggest"
Cohesion: 0.40
Nodes (5): ime_suggest, abi_version, count, items, lens

### Community 415 - "save_word_list"
Cohesion: 0.40
Nodes (5): normalize_word_list(), read_window_text(), Vec, save_word_list(), UI_FONTS

### Community 416 - "tv_win_free"
Cohesion: 0.67
Nodes (3): gpointer, tv_win_free(), C7. [THẤP] GTK settings (settings_window.c:483,214-250) — `TvWin` free

### Community 418 - "T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007"
Cohesion: 0.33
Nodes (6): LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`, LNX-041 · Callback + self-disable (M) — `P3-3 §3`, LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`, LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`, LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S), T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### Community 419 - "per_app"
Cohesion: 0.33
Nodes (6): type, additionalProperties, $comment, default, type, per_app

### Community 420 - "TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux"
Cohesion: 0.25
Nodes (8): Cấu trúc, Dựng từ mã nguồn, Giấy phép, TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux, Trạng thái, Tải và cài, Điểm vượt trội so với các bộ gõ khác, Đóng góp

### Community 421 - "en_restore.rs"
Cohesion: 0.53
Nodes (5): ambiguous_pairs_keep_vietnamese(), en_words_auto_restore_on_space(), top_3000_english_words_type_correctly(), type_word(), textvn_core

### Community 423 - "trigger_keys"
Cohesion: 0.33
Nodes (6): enum, trigger_keys, $comment, default, items, type

### Community 424 - "TextVN IPC v1"
Cohesion: 0.33
Nodes (5): Implementation, Messages, TextVN IPC v1, Trợ lý ngữ pháp (P4-2 · AS-023/D20), Wire framing

### Community 426 - "4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)"
Cohesion: 0.40
Nodes (5): 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), Phát hiện L-01: Bộ phân tích JSON IPC thủ công bằng `strstr`, Phát hiện L-02: Đường dẫn Socket không an toàn trong `/tmp`, Phát hiện L-03: Thiếu định dạng đóng gói chuẩn của các bản phân phối Linux, Phát hiện L-04: Xung đột Wayland vs X11 và Cơ chế Surrounding Text trên Chat Apps

### Community 429 - "ime_instance_free"
Cohesion: 0.13
Nodes (22): assist_check_finds_vietnamese_issue(), assist_check_ignores_clean_or_english_text(), assist_check_instance_reads_sentence_buffer(), assist_rejects_bad_args(), assist_sentence_tail_and_glyph_len(), assist_set_spell_provider_and_check_instance(), assist_spell_provider_fail_open_on_error(), CSpellProvider (+14 more)

### Community 430 - "Trạng thái ký số macOS — Developer ID, notarization, stapling"
Cohesion: 0.40
Nodes (5): 1. Tóm tắt, 2. Ảnh hưởng tới người dùng, 3. Tooling đã nối (chỉ chạy khi có secret), 4. Việc còn lại để bật (chủ repo), Trạng thái ký số macOS — Developer ID, notarization, stapling

## Knowledge Gaps
- **1829 isolated node(s):** `ic`, `inst`, `comp`, `config`, `toggle` (+1824 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 2876 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **67 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `2. Nhật ký finding` connect `ipc_client.c` to `String`, `Foundation`, `IpcClient`, `.inject`, `Problem`, `TextVNInputController`, `MarkedState`, `.register`, `TextVNConfig`, `ImeEngine`, `TextVNAppTests`, `EventTapController`, `IpcServer`?**
  _High betweenness centrality (0.155) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `AppInfo`, `.register`, `ConfigWatcher`, `IMKTextTarget`, `IpcClient`, `.suggest`, `AppDelegate`, `StrategyResolver`, `.onEvent`, `IpcServer`, `MockHandler`, `ConfigStore`, `TextVNInputController`, `ViState`, `TextVNConfig`, `KeyTranslator`, `AXSnapshot`, `AutostartManager`, `Foundation`, `Problem`, `FieldDetect`, `.inject`, `MarkedState`, `ImeEngine`, `TextVNAppTests`, `TapDecision`, `EventTapController`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `text()` connect `text` to ``docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)`, `TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux`, `TextVN IPC v1`, `PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)`, `.suggest`, `e2e_fcitx5.py`, `TextVN — Chỉ mục tài liệu & Giao thức Review`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `AppDelegate` (e.g. with `B. Trên macOS: Quên cập nhật Icon Menu Bar & Luồng giao diện (UI Thread Violation)` and `(A) macOS app / IMK / packaging / CI — `F3-*``) actually correct?**
  _`AppDelegate` has 4 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ic`, `inst`, `comp` to the rest of the system?**
  _1829 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `cli/src/replay.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08028846153846154 - nodes in this community are weakly interconnected._
- **Should `config/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05432692307692308 - nodes in this community are weakly interconnected._