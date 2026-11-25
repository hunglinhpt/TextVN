# Graph Report - TextVN  (2026-11-25)

## Corpus Check
- 528 files · ~1,178,051 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 321 file(s) not represented in the graph (top: .keys 248, (none) 15, .ass 12)

## Summary
- 11646 nodes · 23839 edges · 599 communities (497 shown, 102 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 1477 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `98bc36e9`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- duplicate_word.rs
- compose.c
- cli/src/replay.rs
- config/src/lib.rs
- TsfShared
- abnormal_1000.rs
- windows-hook/src/main.rs
- toml.rs
- xtask/src/main.rs
- ffi/src/lib.rs
- strategy/src/lib.rs
- vowel_table_generated.rs
- bench/src/main.rs
- cli/src/register.rs
- doc.rs
- keymap.rs
- audit_r2_140.rs
- validate.rs
- .new
- telex.rs
- P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết
- windows-tsf/src/lib.rs
- restore_en.rs
- P0-2 — Hợp đồng FFI v1 (nguồn sự thật duy nhất)
- P1-1 — TSF ADAPTER (Windows) — Solution chi tiết
- P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết
- converter.rs
- engine.c
- P2-1 — IMK ADAPTER (macOS) — Solution chi tiết
- syllables.rs
- selection_check.rs
- P0-3 — Config schema · Preset (appdb) · Strategy model · IPC
- P2-2 — CGEVENTTAP FALLBACK (macOS, opt-in) — Solution chi tiết
- properties
- store.rs
- windows-hook/src/lib.rs
- Chi tiết từng mục
- .process
- lexicon.rs
- P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator
- properties
- properties
- properties
- win_corpus_cases.rs
- `ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)
- P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết
- P2-4 — UI, Config, IPC, Packaging & Release (macOS) — Solution chi tiết
- appdb.v1.schema.json
- $defs
- TextVN — Workflow & Ground Rules thống nhất cho mọi agent
- AGENT HANDBOOK — Quy ước làm việc cho mọi agent/contributor
- P1-5 — TEST PLAN (Windows) — Solution chi tiết
- M1 — TSF core (tuần 3–6) — dep: WIN-002, WIN-003
- P2-5 — TEST PLAN (macOS) — Solution chi tiết
- A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004
- charset.rs
- doctor.rs
- StrategyResolver
- P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết
- string
- T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004
- settings_dialog.rs
- fix.rs
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- Id
- textvn-ffi
- type
- compose.rs
- P1-0 — WINDOWS MASTER PLAN (Phần 1)
- tray/src/lib.rs
- M0 — Spike & nền (tuần 1–2)
- P2-0 — macOS MASTER PLAN (Phần 2)
- fst.rs
- P3-0 — LINUX MASTER PLAN (Phần 3)
- TextVNInputController
- predict.rs
- 1. Chi tiết Giải pháp & Mã nguồn Từng Task (Turnkey Implementation Recipes)
- M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)
- A4 — Test & hardening (tuần 15–18)
- T5 — Tray / Settings / Packaging (tuần 11–14)
- panel_idle_1000.rs
- merge_style_drain
- Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)
- png.rs
- textvn-common.sh
- art_icons.rs
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- super
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- Engine
- config.rs
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`
- SvcManager
- IpcClient
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- properties
- ByteCursor<'a>
- AssistTests
- when
- P1 — REVIEW LOG (Phần 1: Windows)
- atomic
- entry
- id
- article_an.rs
- language.rs
- MarkedState
- ffi/src/settings.rs
- P1-6 — TASKS Windows (WBS) — Nhận việc từng task
- P2-6 — TASKS macOS (WBS) — Nhận việc từng task
- office_productivity.rs
- String
- style_profile.rs
- P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN
- windows
- adr/README.md
- tsf-min
- idle_popup_routing.rs
- test_delete_surrounding.cpp
- Attribution dữ liệu Tatoeba (CC BY 2.0 FR)
- vni.rs
- TextVNAppTests
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- ipc/src/lib.rs
- win_tools.rs
- bool
- store_win.rs
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- tray/src/assist.rs
- guids.rs
- uia_spike.ps1
- cli/src/assist.rs
- register_wow64_mirror
- EmojiPickerState
- autostart.rs
- Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- dict.rs
- ipc_client.c
- P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế
- os_macos.rs
- assist_tools.rs
- mmap.rs
- developer-guide.md
- package_bootstrap.rs
- win32-uia.lib.ps1
- Dict
- engine.cpp
- Issue
- ipc_client.rs
- bitset
- verify_targets.ps1
- verify.rs
- Performance Audit — TextVN Windows Platform
- HWND
- shutil
- P4-12 — Recall trên **văn bản thật** (bộ nhớ có nhãn) + FPR
- spelling.rs
- .suggest
- .new
- build-release.ps1
- AppDelegate
- itertools
- P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN
- rules_win.rs
- P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết
- verify-evidence.md
- DialogContext
- dictionary_budget.rs
- hygiene.rs
- IpcServer
- invariants.rs
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- ime_result_v1
- windows-tsf/src/assist.rs
- .register
- EmojiPicker
- cli/src/main.rs
- phrase_repeat.rs
- Đóng góp cho TextVN
- ImeEngine
- dict/src/lib.rs
- Result
- macro_text.rs
- dual_mode_matrix.rs
- trigger
- OutputStrategy
- phrases.rs
- field-detect/src/lib.rs
- e2e_ibus.c
- IpcServerAssistTests
- ipc_server.rs
- e2e-assist-p6.ps1
- e2e_fcitx5.c
- emoji_popup.rs
- key_event.rs
- chords.rs
- edit_session.rs
- undo.rs
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- username_identity_1000.rs
- LoadError
- .new
- settings_window.c
- number_vn.rs
- foreground.rs
- settings_window_gtk3.c
- Foundation
- ThreadState
- Path
- EmojiPickerModel
- P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- properties
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- linux_tools.rs
- EventTapController
- en_office.rs
- Báo cáo dựng & kiểm thử — TextVN
- remove_added_languages
- dream_rsi_100.rs
- Dictionary
- ime_conflict_1000.rs
- tsf-min/src/lib.rs
- parse_ctrl_shift_marker
- smoke-imk.sh
- templates.rs
- Kế hoạch ký số (code signing) — TextVN
- os_windows.rs
- SettingsView
- test-typing.ps1
- env-mac — Môi trường build macOS (MAC-001)
- MathContextFsm
- Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)
- windows-tsf/src/replay.rs
- tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)
- PackageDescription
- text_prediction.rs
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- e57_math_sync.rs
- e2e-linux.sh
- mem-check.sh
- soak.sh
- test_compose.c
- e2e-assist-apps.ps1
- default
- build-macos.sh
- stage-linux.sh
- build-rust.sh
- svc.rs
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
- CandidateOverlay
- ConfigWatcher
- path
- OverlayView
- command
- check_version_sync.rs
- NSRange
- Rgba
- P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công
- properties
- tone_inject_one
- SettingsDoc
- emoji_picker.rs
- worker_main
- 3. Nhật ký finding
- ghost.rs
- Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)
- verify-store-package.sh
- cuas.rs
- predict_next_phrase_candidates
- Result
- .unregister
- sentence_grammar.rs
- IpcMessage
- Option
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- emoji_catalog.rs
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- FieldDetect
- settings_model.c
- src/style_merge.rs
- en_office_phrases_10k.rs
- check_mac_targets.rs
- macro.rs
- Diagnostics
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- os
- syllable_spelling.rs
- viqr.rs
- AXSnapshot
- preset_matrix.rs
- tip.rs
- diacritic_typo.rs
- argparse
- AssistCheck
- macSpellContainsCallback
- plan_sentence_auto_fix
- 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft
- panel_idle.rs
- dual_mode_rapid.rs
- trace.rs
- default
- rsi
- package_family_full_names
- [0.2.37] — 2026-10-24
- readme_numbers.rs
- TextVN — Bộ gõ tiếng Việt thế hệ mới cho Windows, macOS và Linux
- P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A
- Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store
- en_spelling.rs
- e63_expert_personas_audit.rs
- ◻ Đề xuất tính năng (ưu tiên theo giá trị user)
- A. WINDOWS
- Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI
- push_key
- prepare-ppa.sh
- 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"
- T6 — Test & hardening (tuần 15–18)
- collect_windows_env_summary
- FieldContext
- plan_key
- 3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow)
- Added
- .contains
- inflate_raw_bounded
- English
- 2. Nhật ký finding
- e57_word_sync_fca.rs
- pipe_name
- MathOutputMode
- P4-11 — ĐO TÀI NGUYÊN ASSIST (F61) — TIP THẬT
- Strategy
- DictError
- P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)
- Phân phối Linux — kênh, store và auto-update
- mac_corpus_cases.rs
- Word
- deflate_fixed
- KeyTranslator
- sign-artifacts.sh
- P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19)
- P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4
- Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại
- ime_assist_issue
- Ctx
- ime_context
- P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)
- error_success
- unmark_as_unmark_tone
- e50_unikey_restore.rs
- ViState
- Rà soát backlog theo tài liệu kế hoạch — 2026-11-08
- eval.rs
- Option
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- TapPermission
- icons.rs
- display
- classifier.rs
- P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả
- show_macro_editor
- TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV
- mask.rs
- simple_telex.rs
- textvn_config
- CachedFormulaStore
- publish-aur.sh
- os_spell.rs
- quick_telex.rs
- StageInfo
- style_learning_100.rs
- p11_math_toggle.rs
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- per_app
- Security Policy
- abnormal.rs
- press
- trigger_keys
- core/src/lib.rs
- Option
- properties
- Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- P8/V55 — DFMEA + DRBFM: 5 fix phản hồi test thật của chủ repo (2026-11-05)
- .new
- export_diagnostics_zip
- rca_v57_adversarial.rs
- 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU
- learning_curve.rs
- gen_vocab.rs
- say
- caps.rs
- pathlib
- ChordFsm
- emoji_picker.c
- measure_assist_rss.ps1
- 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux
- ctrl_shift_marker_path
- style_learning_v59.rs
- test-linux-distro-matrix.sh
- StylePending
- core
- default
- os_linux.rs
- press
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- math
- English
- x86probe
- outputcharset
- test-linux-el8-package.sh
- [0.2.32] — 2026-10-12
- Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên
- entries
- conflict_notice.rs
- linux-fcitx5/src/engine.h
- [0.2.14] — 2026-10-03
- MathEngine
- do_register
- worker_main
- engine_boundary_fca.rs
- ModifierToggle
- save
- linux-settings/src/main.c
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- display
- [0.2.28] — 2026-10-10
- Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening
- HookEngine
- IpcCodecTests
- vn_freq_data
- Vec
- overlay.rs
- 2. Windows — Spell Checking API (`spellcheck.h`, Win8+)
- .CreateInstance
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- Hướng dẫn phát triển TextVN
- non_text_focus_gate.rs
- claim_debounced
- idle_ms
- items
- [0.2.16] — 2026-10-03
- max_sentence_chars
- HookState
- [0.2.13] — 2026-10-03
- diacritic_style.rs
- [0.2.33] — 2026-10-18
- P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15
- IMPLEMENTATION STATUS — adapter macOS (Farch-4)
- [0.2.81] — 2026-11-18
- TextVN — Chỉ mục tài liệu & Giao thức Review
- [0.2.38] — 2026-10-24
- Vec
- resolve.rs
- e50_audit_edge.rs
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- mode
- Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent
- quick_telex
- 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)
- Broker
- corpus.rs
- Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành
- [0.2.65] — 2026-11-06
- [0.2.8] — 2026-10-02
- build-linux-el8.sh
- [0.2.85] — 2026-11-22
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- [0.2.42] — 2026-10-24
- hud_pause_ms
- windowsandmessaging
- leader_key
- collect_report
- palette_max_results
- [0.2.46] — 2026-10-26
- P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)
- test-linux-smoke.sh
- [0.2.54] — 2026-10-30
- KeyKind
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- Khắc phục sự cố TextVN (Troubleshooting)
- English
- diag_summary.rs
- Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục
- DreamRsi
- build-linux-compat.sh
- textvn-settings-launcher.sh
- [0.2.0] — 2026-09-30
- Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất
- 2. Kết quả kiểm thử
- P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết
- [0.2.78] — 2026-11-14
- [0.2.64] — 2026-11-04
- needs_reopen_notice
- [0.2.72] — 2026-11-08
- [0.2.74] — 2026-11-08
- [0.2.9] — 2026-10-02
- bz2
- dbus_mainloop_glib
- pre_translate_message
- TRAY_EXIT_PENDING
- gi_repository
- importlib_util
- json
- pil
- re
- struct
- textvn_core
- trusted_install_roots
- B. macOS
- ime_assist_plan
- subprocess
- sys
- tempfile
- unicodedata
- urllib_request
- xml_etree_elementtree
- zipfile
- .process
- ActionKind
- [0.2.69] — 2026-11-08
- AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN
- cli/build.rs
- e62_math_all_editors.rs
- en_restore.rs
- 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)
- style_stats_text
- [0.2.89] — 2026-11-25
- [0.2.10] — 2026-10-03
- getdpiforwindow
- hidpi
- [0.2.75] — 2026-11-08
- [0.2.79] — 2026-11-14
- can_apply_fix
- [0.2.40] — 2026-10-24
- [0.2.34] — 2026-10-18
- MACRO_LAYOUT
- [0.2.47] — 2026-10-26
- [0.2.49] — 2026-10-26
- [0.2.4] — 2026-10-01
- [0.2.35] — 2026-10-23
- WORDLIST_LAYOUT
- [0.2.50] — 2026-10-28
- [0.2.67] — 2026-11-07
- [0.2.83] — 2026-11-21
- textvn_settings.h
- [0.2.68] — 2026-11-07
- [0.2.70] — 2026-11-08
- CTRL_TAP_MAX
- DIAG_EXPORT_IN_PROGRESS

## God Nodes (most connected - your core abstractions)
1. `Engine` - 128 edges
2. `TsfShared` - 110 edges
3. `TextVNInputController` - 107 edges
4. `Báo cáo dựng & kiểm thử — TextVN` - 95 edges
5. `Changelog` - 92 edges
6. `SvcManager` - 76 edges
7. `AppDelegate` - 66 edges
8. `Issue` - 64 edges
9. `Ctx` - 61 edges
10. `TextVNState` - 54 edges

## Surprising Connections (you probably didn't know these)
- `Changed` --references--> `lc_state_read_assist_enabled()`  [INFERRED]
  CHANGELOG.md → adapters/linux-common/src/compose.c
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-7-TASKS.md → adapters/linux-common/src/field_detect.c
- `C. Trên Linux: IPC Client bỏ qua Broadcast toàn cục `app_id: "*"`` --references--> `handle_ipc_message()`  [INFERRED]
  docs/release/AUDIT_AND_PRODUCTION_ROADMAP_v0.2.19.md → adapters/linux-common/src/ipc_client.c
- `A9. [TRUNG] `apply_plan` (edit_session.rs:394-436): delete xong, insert fail → mất chữ đã xoá một nhịp gõ` --references--> `apply_plan()`  [INFERRED]
  docs/release/AUDIT-3PLATFORM-2026-10-07.md → adapters/linux-common/tests/test_compose.c

## Import Cycles
- 1-file cycle: `dict/src/mmap.rs -> dict/src/mmap.rs`
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`
- 2-file cycle: `dict/src/lib.rs -> dict/src/os_windows.rs -> dict/src/lib.rs`

## Communities (599 total, 102 thin omitted)

### Community 0 - "duplicate_word.rs"
Cohesion: 0.11
Nodes (16): bat_lap_tu_thuong(), cau_mix_whitelist_theo_tu(), check(), EN_INTENTIONAL_REPEATS, INTENTIONAL_REPEATS, is_aabb(), is_compound_then_modal(), PRONOUN_CAP_FIRE (+8 more)

### Community 1 - "compose.c"
Cohesion: 0.05
Nodes (64): lc_config_state, allow_macro_when_vi_off, assist_auto_fix, assist_mode, auto_capitalize, loaded, math_enabled, mtime_ns (+56 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.08
Nodes (46): action_name(), adapter_profile(), AdapterProfile, build_json(), caps_bits(), Case, case_without_any_expectation_is_rejected(), Cmd (+38 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.04
Nodes (53): assist_defaults_match_p4_0_section_7(), assist_full_block_parses(), ASSIST_TRIGGER_KEYS, AssistDisplay, AssistLanguage, AssistSettings, Config, ConfigError (+45 more)

### Community 4 - "TsfShared"
Cohesion: 0.13
Nodes (42): FixPlan, ActiveAssist, apply_assist_suggestion(), apply_assist_undo(), apply_ctrl_tap_in_context(), apply_sentence_auto_fix(), apply_sentence_fix_all(), ApplyAssistFixSession (+34 more)

### Community 5 - "abnormal_1000.rs"
Cohesion: 0.06
Nodes (31): a1000_builder_deterministic_va_id_stable(), a1000_tat_ca_case_qua_4_rao_chan(), assert_utf8(), budget_for(), build_cases(), Case, clip(), DECOMP (+23 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (47): CAPS_HELD, caps_lock_bit(), CAPS_TOGGLED, CTRL_DOWN, empty_ime_result(), FocusProbe, get_active_modifiers(), get_process_name_for_window() (+39 more)

### Community 7 - "toml.rs"
Cohesion: 0.16
Nodes (15): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+7 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.11
Nodes (37): banner(), emit(), format_rust(), gen_tables(), index_of(), key_char(), KEYMAP_MAC_OUT_RUST, KEYMAP_MAC_OUT_SWIFT (+29 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.03
Nodes (138): remove_auto_capitalize_conflicts(), assist_check_finds_vietnamese_issue(), assist_check_ignores_clean_or_english_text(), assist_check_instance_reads_sentence_buffer(), assist_check_v1_fail_open_on_empty_null_and_bad_utf16(), assist_check_v1_matches_utf8_path(), assist_check_v1_utf16_offset_is_char_not_utf16(), assist_plan_sentence_fix_happy_path() (+130 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.09
Nodes (20): IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO (+12 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.08
Nodes (27): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+19 more)

### Community 12 - "bench/src/main.rs"
Cohesion: 0.11
Nodes (32): black_box, bench_dream_rsi(), bench_field_switch_resolve(), bench_ime_key(), bench_resolve(), best_of(), best_of_giu_vong_nhanh_nhat(), check_dream_rsi() (+24 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.06
Nodes (34): authorization, call_layout_or_tip(), call_layout_or_tip_spec(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, CLSID_INNER (+26 more)

### Community 14 - "doc.rs"
Cohesion: 0.13
Nodes (25): atomic_write(), create_private_dir(), DocKind, insert_nested(), invalid_values_are_rejected_without_changing_the_doc(), macros_text_round_trip_through_doc(), MAX_FILE_BYTES, missing_keys_read_schema_defaults() (+17 more)

### Community 15 - "keymap.rs"
Cohesion: 0.05
Nodes (28): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, END, ESCAPE, F1 (+20 more)

### Community 16 - "audit_r2_140.rs"
Cohesion: 0.10
Nodes (32): abnormal_ngoac_gach_ngang_tron_chu_khong_mat_ky_tu(), abnormal_tu_cuc_dai_va_ngoac_lien_tiep(), field(), learning(), mode_en(), opts(), r2_140_auto_restore_tat_van_khong_pha_chi_muc(), r2_140_quick_telex_nhan_ngoac() (+24 more)

### Community 17 - "validate.rs"
Cohesion: 0.12
Nodes (22): plausible_ascii_word(), is_vowel(), unmark(), coda_after_nucleus_ok(), CODAS, decompose(), has_vowel(), is_valid_word() (+14 more)

### Community 18 - ".new"
Cohesion: 0.13
Nodes (35): ArcDict, build_candidates_cli(), cli(), comma_join(), comma_sorted_join(), en_adjacent_words(), EngineDict, fetch_cache_bytes() (+27 more)

### Community 19 - "telex.rs"
Cohesion: 0.07
Nodes (23): codas, char_idx(), circum_pair(), f(), fold(), fold_with(), fold_with_caps(), fold_with_caps_impl() (+15 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.04
Nodes (53): after_request_edit_session(), classify_tsf_field(), config_file_path(), d16_auto_capitalize_theo_config(), DllCanUnloadNow(), DllGetClassObject(), EditSessionResult, engine_session_sentence_tail_and_glyph_len() (+45 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.06
Nodes (56): am_tiet_viet_that_khong_bi_restore(), canh_bao_cai_muc_mo_ho(), chars(), complete_word(), complete_word_candidates(), data_en_common_file_fires(), data_stop_en_file_is_self_consistent(), data_vn_common_file_is_valid() (+48 more)

### Community 23 - "P0-2 — Hợp đồng FFI v1 (nguồn sự thật duy nhất)"
Cohesion: 0.18
Nodes (10): 0. Bất biến (invariants), 1. Header (bản 1:1 với `ffi/include/textvn_ffi.h`), 2. Ngữ nghĩa `action` — adapter PHẢI làm đúng bảng này, 3. Vòng đời & thread model, 4. Mapping sang adapter preedit-vs-replace (quan trọng), 5. Mã lỗi & hành vi khi lỗi, 6. Versioning & kiểm chứng bằng CI, 7. Adapter walkthrough (Windows TSF — mẫu để adapter khác làm theo) (+2 more)

### Community 24 - "P1-1 — TSF ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping sang task — chi tiết ở `P1-6-TASKS.md`), 11. Chế độ test & chẩn đoán, 12. Failure modes & xử lý, 1. Tổng quan & quyết định, 2. Cấu trúc file (bám đúng `P0-1 §1`), 3. Vòng đời COM (lifecycle), 4. State machine của ThreadState, 5. Key flow (+9 more)

### Community 25 - "P3-1 — IBUS ADAPTER (Linux) — Solution chi tiết"
Cohesion: 0.11
Nodes (18): 10. Playbook triển khai (mapping `P3-7-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module & repo, 3. Vòng đời IBus, 4. State machine (giống `P1-1 §4`, preedit thay composition), 5.1 Translate `keyval` → `ime_key_v1` (SPIKE `LNX-004` — RL1) (+10 more)

### Community 26 - "converter.rs"
Cohesion: 0.08
Nodes (41): extract_text(), LatexParser, LatexParser<'a>, MathNode, MAX_DEPTH, MAX_TOKENS, nested_braces(), parse_ascii() (+33 more)

### Community 27 - "engine.c"
Cohesion: 0.09
Nodes (65): lc_assist_auto_capitalize_enabled(), lc_assist_is_active(), lc_comp_apply(), lc_modifier_toggle_reset(), lc_plan_commit_text(), lc_log(), apply_assist_candidate(), apply_assist_undo() (+57 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+9 more)

### Community 29 - "syllables.rs"
Cohesion: 0.07
Nodes (23): CONFIDENCE_AMBIGUOUS, CONFIDENCE_UNAMBIGUOUS, confusable(), DATA, decompose(), dict(), dong_bat_thuong_khong_panic(), EXTRA (+15 more)

### Community 30 - "selection_check.rs"
Cohesion: 0.10
Nodes (27): ApplySelectionFixSession, ApplySelectionFixSession_Impl, DEBOUNCE_MS, get_selection_anchor(), is_selection_key_up(), MAX_SELECTION_CHARS, MAX_TRANSCODE_CHARS, MIN_SELECTION_CHARS (+19 more)

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
Cohesion: 0.13
Nodes (17): callback_budget_matrix(), CallbackDecision, caps_toggle_step(), classify_origin(), e48_classify_origin_and_keyboard_state_truth_table(), INJECT_TAG, keyboard_state_for(), KeyOrigin (+9 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - ".process"
Cohesion: 0.08
Nodes (23): ApplyAssistFixSession_Impl, ApplyAssistUndoSession_Impl, ApplySentenceFixSession_Impl, AssistRecallSession_Impl, caret_at_composition_end(), clear_assist_undo_xoa_state(), CompSink_Impl, CtrlTapSession_Impl (+15 more)

### Community 38 - "lexicon.rs"
Cohesion: 0.08
Nodes (47): Discipline, alias_index(), alias_never_collides_with_other_trigger(), ALL_ENTRIES, all_slices(), binary_search_consistency(), CHEMISTRY, CS_AI (+39 more)

### Community 39 - "P0-4 — Test strategy · Định dạng corpus `.keys` · Replay simulator"
Cohesion: 0.15
Nodes (12): 1. 7 lớp test (chi tiết trong PLAN §4.3, bổ sung chỗ chạy), 2.1 Tổng quan, 2.2 Bảng lệnh (chính thức), 2.3 Ví dụ đầy đủ (phải chạy được ngay), 2.3b Lệnh bổ sung (bắt buộc có trong parser), 2.4 Tên phím chuẩn (case-sensitive), 2. Định dạng corpus `.keys`, 3. Simulator semantics (`cli/src/replay.rs`) — mô hình tính toán (+4 more)

### Community 40 - "properties"
Cohesion: 0.13
Nodes (15): type, $ref, $ref, matchClause, $ref, $ref, additionalProperties, minProperties (+7 more)

### Community 41 - "properties"
Cohesion: 0.06
Nodes (34): default, type, $comment, default, type, $comment, default, type (+26 more)

### Community 42 - "properties"
Cohesion: 0.12
Nodes (16): minLength, type, minLength, type, properties, action, combo, count (+8 more)

### Community 43 - "win_corpus_cases.rs"
Cohesion: 0.29
Nodes (11): all_cases(), CASES_PART1, CASES_PART2, CASES_PART3, CASES_PART4, CASES_PART5, CASES_PART6, CASES_PART7 (+3 more)

### Community 44 - "`ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter)"
Cohesion: 0.22
Nodes (8): 1. Bất biến (P0-2 §0) — đọc trước khi viết adapter, 2. Header (nguyên văn — sinh từ `ffi/include/textvn_ffi.h`), 3. Chú giải: vòng đời chuẩn của adapter, 4. Lỗi thường gặp (rút từ bug B1–B13), 5. Khi nào phải bump `IME_ABI_VERSION`, `ffi.v1` — TextVN engine C-ABI v1 (bản chú giải cho adapter), Ghi chú P4-13 — từ điển dấu thanh dựng sẵn (2026-11-08), `ime_style_profile_persist` (R2-152)

### Community 45 - "P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Mapping task, 1. Tray — `TextVN.exe` (chạy 1 instance, notify icon), 2. IPC server (P0-3 §4) — chạy trong tray, 3. Settings — hộp thoại Win32 (`tray/src/settings_dialog.rs`), 4. Cài đặt — `TextVN-setup-<ver>-windows-x64.exe` (Inno Setup 6.5+, x86_64), 5. Ký số & nguồn gốc (RW3 — antivirus), 6. Watchdog & process orchestration (tray), 7. Updater — crate `textvn-updater` (lib, chạy trong `TextVN.exe`) (+3 more)

### Community 46 - "P2-4 — UI, Config, IPC, Packaging & Release (macOS) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Task (chi tiết `P2-6-TASKS.md`), 1. Status menu — `TextVN.app` (LSUIElement, 1 instance), 2. IPC server (mirror `P1-4 §2`) — unix socket, 3. Settings — SwiftUI (cửa sổ 1, sidebar 6 tab — parity với `P2-0`/PLAN §2.3 M6), 4. Cài đặt & gỡ — `.pkg` (per-user, không sudo — PLAN §3.7), 5. Ký số & Gatekeeper (RM3), 6. Health & restart, 7. Updater (Ed25519 "Sparkle-style" — `PLAN §3.7`) (+3 more)

### Community 47 - "appdb.v1.schema.json"
Cohesion: 0.12
Nodes (15): additionalProperties, const, $comment, $id, $ref, properties, appdb_version, min_engine_version (+7 more)

### Community 48 - "$defs"
Cohesion: 0.17
Nodes (12): maxLength, minLength, type, $defs, appIdentifier, fieldRole, matcher, semver (+4 more)

### Community 49 - "TextVN — Workflow & Ground Rules thống nhất cho mọi agent"
Cohesion: 0.10
Nodes (19): 10. Dọn dẹp — checklist trước khi đóng phiên / tick Done, 11. Cập nhật docs & nghiệp vụ — checklist khi "xong", 12.1 Workflow hiện có, 12.2 11 check của `repo-hygiene`, 12.3 Tham chiếu Marketplace (yêu cầu user 2026-09-27), 12.4 Workflow phát hành (`release-candidate`), 12. GitHub Actions — giữ repo sạch sau mỗi lần code, 1. Ground rules (G-IDs) — áp dụng mọi lúc (+11 more)

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

### Community 55 - "charset.rs"
Cohesion: 0.06
Nodes (63): ansi_byte(), BASE_CHARS, Charset, decode_tcvn3(), decode_vni_windows(), detect(), enc(), encode() (+55 more)

### Community 56 - "doctor.rs"
Cohesion: 0.09
Nodes (23): BUILD_GIT_SHA, BUILD_TARGET, classify_distribution(), collect_linux_env_summary(), command_on_path(), current_distribution(), current_euid(), detect_fcitx5_profile_conflicts() (+15 more)

### Community 57 - "StrategyResolver"
Cohesion: 0.14
Nodes (15): Key, StrategyResolver, .cacheCount, .hasAppdb, Bool, Data, Int, Int32 (+7 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "string"
Cohesion: 0.06
Nodes (59): jb_cstr(), jb_json_string(), jb_raw(), jb_u32(), lc_assist_filter_auto_capitalize(), lc_assist_kind_name(), lc_assist_report_json(), lc_assist_expected_document_tail() (+51 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.02
Nodes (111): normalize_word_list, ABOUT_DETAILS, ABOUT_TAGLINE, about_text(), about_text_matches_every_platform(), ASSIST_MODE_HINTS, ASSIST_MODE_LABELS, BM_GETCHECK (+103 more)

### Community 62 - "fix.rs"
Cohesion: 0.11
Nodes (16): apply_auto_fixes(), chi_giu_loi_can_sua(), Fix, plan_sentence_fix(), Option, Vec, select_non_overlapping_fixes(), SentenceFixPlan (+8 more)

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

### Community 68 - "Id"
Cohesion: 0.17
Nodes (32): available_languages(), class_of(), close_spell_document(), contains_word(), handle_job(), language_match(), msg_send_available_languages(), msg_send_check_spelling() (+24 more)

### Community 69 - "textvn-ffi"
Cohesion: 0.32
Nodes (17): textvn-appdb, textvn-assist, textvn-bench, textvn-cli, textvn-config, textvn-core, textvn-dict, textvn-ffi (+9 more)

### Community 70 - "type"
Cohesion: 0.11
Nodes (24): items, default, items, type, $comment, default, items, type (+16 more)

### Community 71 - "compose.rs"
Cohesion: 0.05
Nodes (24): BACK, backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CONTROL, enter_commits_before_app_receives_key_b2(), ESCAPE, escape_restores_raw_keys() (+16 more)

### Community 72 - "P1-0 — WINDOWS MASTER PLAN (Phần 1)"
Cohesion: 0.22
Nodes (8): 1. Mục tiêu phần Windows (exit condition của Phần 1), 2. Kiến trúc 3 process (đã chốt với P0-1/P0-3), 3. Workstream & file solution tương ứng, 4. Lộ trình (Windows slice của PLAN §2.6), 5. Dependency graph (thứ tự kéo việc), 6. Rủi ro & mitigation (riêng Windows), 7. Definition of Done cho PHẦN 1, P1-0 — WINDOWS MASTER PLAN (Phần 1)

### Community 73 - "tray/src/lib.rs"
Cohesion: 0.06
Nodes (23): ipcserver, settings, show_settings_dialog, copy_to_wide_buf(), CTRL_SHIFT_MARKER, CTRL_SHIFT_MARKER_DECLINED, CTRL_SHIFT_MARKER_FREED, CTRL_SHIFT_MARKER_RECORD (+15 more)

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
Cohesion: 0.05
Nodes (34): AssistCandidate, SentenceFixPlan, Int, TextTarget, Any, AssistIssue, AssistSettings, AssistUndoState (+26 more)

### Community 79 - "predict.rs"
Cohesion: 0.08
Nodes (47): CompoundRow, adapt_capitalization(), adapt_continuation_case(), collect_phrase_matches(), du_lieu_phrases_invariant(), en_common_lines(), en_compounds(), en_compounds_buckets() (+39 more)

### Community 80 - "1. Chi tiết Giải pháp & Mã nguồn Từng Task (Turnkey Implementation Recipes)"
Cohesion: 0.05
Nodes (36): 0. Tổng quan Kiến trúc & Thứ tự Thi công (Dependency Graph), 1.1 Bài toán & Hành vi mục tiêu, 1.2 Mã nguồn cần thêm, 1. Chi tiết Giải pháp & Mã nguồn Từng Task (Turnkey Implementation Recipes), 1b. Trạng thái triển khai (2026-10-25), 2.1 Bài toán & Hành vi mục tiêu, 2.2 Quy tắc chính xác & Phát hiện từ Review 2, 2.3 Mã nguồn cần thêm trong `core/src/lib.rs` (+28 more)

### Community 81 - "M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC)"
Cohesion: 0.25
Nodes (8): M5 — Test automation & hardening (tuần 15–18, kéo dài đến RC), WIN-060 · `appcomptest` harness (L) — `P1-5 §4`, WIN-061 · Targets JSON cho 12 app CI (M), WIN-062 · Perf bench + baseline (M) — `P1-5 §5`, WIN-063 · Soak 24h script (M), WIN-064 · CI jobs (M) — `P1-5 §8`, WIN-065 · Fuzz + ASan/TSan job Windows (M), WIN-066 · Security + license checklist Phần 1 (M) — Handbook §8

### Community 82 - "A4 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): A4 — Test & hardening (tuần 15–18), MAC-060 · AX harness `tools/mac/ax-driver` (L) — `P2-5 §4`, MAC-061 · Targets JSON 12 app (cùng format Windows) (M), MAC-062 · Perf bench + `perf/baseline-mac.json` (M) — `P2-5 §5`, MAC-063 · Soak 24h script (M), MAC-064 · CI jobs macos (M) — `P2-5 §8`, MAC-065 · Fuzz + ASan job mac (S), MAC-066 · Security + license checklist Phần 2 (M) — Handbook §8 + audit tap (P2-2 §7)

### Community 83 - "T5 — Tray / Settings / Packaging (tuần 11–14)"
Cohesion: 0.25
Nodes (8): LNX-050 · Tray SNI + menu 9 mục + single instance (M) — `P3-5 §1`, LNX-051 · IPC server + watcher + health (L) — `P3-5 §2/§4`, LNX-052 · Settings GTK4 6 tab + parity checklist (L) — `P3-5 §3`, LNX-053 · systemd user unit + config init + hot-reload (M), LNX-054 · `.deb` cài/gỡ sạch (L) — `P3-5 §5`, LNX-055 · `doctor` full (M) — `P3-5 §6`, LNX-056 · Update-notify + `.rpm` + AUR (M) — `P3-5 §7/§8`, T5 — Tray / Settings / Packaging (tuần 11–14)

### Community 84 - "panel_idle_1000.rs"
Cohesion: 0.09
Nodes (47): build_p1_a(), build_p1_b(), build_p1_c(), build_p2_ids(), clip(), feed(), IDLE_MS, is_sentence_trigger() (+39 more)

### Community 85 - "merge_style_drain"
Cohesion: 0.11
Nodes (16): merge_style_drain(), parse_style_drain(), parse_style_rsi_delta(), Option, RsiTotals, StyleCorrection, StyleProfileSettings, StyleWord (+8 more)

### Community 86 - "Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference)"
Cohesion: 0.20
Nodes (9): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Oracle UniKey — dùng engine UniKey làm chuẩn đối chiếu (golden reference) (+1 more)

### Community 87 - "png.rs"
Cohesion: 0.12
Nodes (26): ico_eight_frames_header(), adler32(), bbox(), box_blur_1d(), CL_ORDER, crc32(), crc32_update(), decode_png_rgba() (+18 more)

### Community 88 - "textvn-common.sh"
Cohesion: 0.07
Nodes (76): put(), install.sh script, usage(), textvn-common.sh script, tv_check_ime_conflicts(), tv_check_package(), tv_desktop_quote(), tv_detect_pkg_mgr() (+68 more)

### Community 89 - "art_icons.rs"
Cohesion: 0.09
Nodes (32): encode_rgba, APP_E, APP_V, art_compose_layout_and_png_size_under_hygiene_limit(), ART_SHADOW, ART_WHITE, badge_png_valid_structure(), badge_shadow_aa_and_glyph_margin() (+24 more)

### Community 90 - "config.v1.schema.json"
Cohesion: 0.25
Nodes (7): additionalProperties, $comment, $id, required, $schema, title, type

### Community 91 - "Verified Ops — sổ thao tác / tool đã kiểm chứng"
Cohesion: 0.29
Nodes (6): A. Repo · Git · Graphify, B. Build · Test · Script, C. Windows platform (chi tiết → spike specs), D. Môi trường máy dev (snapshot 2026-09-27) — khỏi kiểm tra lại, E. Fixture / app test sẵn có (không dựng lại), Verified Ops — sổ thao tác / tool đã kiểm chứng

### Community 92 - "super"
Cohesion: 0.06
Nodes (36): bat_dau_cau_thuong(), check(), is_sentence_start(), Vec, check(), OFFICE_BIGRAMS, OFFICE_PHRASES, OFFICE_TRIGRAMS (+28 more)

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
Cohesion: 0.11
Nodes (10): Engine, Outcome, Arc, Cell, KeyEvent, Option, Tail, valid_style_entry() (+2 more)

### Community 97 - "config.rs"
Cohesion: 0.08
Nodes (29): ConfigError, combo_signature(), ConfigError, custom_output_strips_backslash(), default_config_backward_compatible(), default_hotkeys(), e70_protected_hotkeys_case_insensitive(), HotkeySpec (+21 more)

### Community 98 - "T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005"
Cohesion: 0.29
Nodes (7): LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`, LNX-031 · Cache + AT-SPI event invalidate + budget (M), LNX-032 · Preset 20 app + corpus ≥ 40 case (M) — `P3-4 §3`, LNX-033 · Appdb loader + `ime_appdb_verify` (S) — P0-2 §1, không viết lại, LNX-034 · Override chain `P3-4 §6` (path Linux) (M), LNX-035 · `doctor` env matrix + framework auto (M) — `P3-4 §7`, `P3-5 §6`, T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005

### Community 99 - "P3-REVIEW-LOG — Phần 3 (Linux)"
Cohesion: 0.22
Nodes (8): Code Review Round 1 — Đúng & Đủ (Mã nguồn C/C++), Code Review Round 2 — Nhất quán & Sẵn sàng (Mã nguồn & Kiểm thử), Kiểm chứng sau fix (Review 2 cuối), Kết quả tổng, P3-REVIEW-LOG — Phần 3 (Linux), Review 1 — Đúng & Đủ, Review 2 — Nhất quán & Sẵn sàng, Rủi ro còn mở của Phần 3 (theo dõi, không phải finding)

### Community 100 - "Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`"
Cohesion: 0.29
Nodes (6): 1. Bảng scope — API nào cần elevation?, 2. Fallback đã chứng minh cho `--scope user`, 3. Kết quả verify Win+Space / registry, 4. Finding (bắt buộc áp cho impl), 5. Exit, Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`

### Community 101 - "SvcManager"
Cohesion: 0.11
Nodes (17): DocError, apply_option_checkbox(), effective_enabled(), persist_keeps_unknown_keys_and_reset_keeps_macros(), AtomicBool, AtomicU64, DiacriticStyle, FnOnce (+9 more)

### Community 102 - "IpcClient"
Cohesion: 0.13
Nodes (15): Frame, frame, needMore, violation, IpcClient, IpcClientDelegate, Data, DispatchSourceRead (+7 more)

### Community 103 - "env.md — Môi trường build Windows (WIN-001)"
Cohesion: 0.33
Nodes (5): 1. Build host (máy dev), 2. Tools DoD (Handbook §4 mục 4), 3. Bằng chứng acceptance (WIN-001), 4. Gap & việc cần khi lên milestone liên quan, env.md — Môi trường build Windows (WIN-001)

### Community 104 - "A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009"
Cohesion: 0.33
Nodes (6): A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009, MAC-040 · Tap thread skeleton + owner check (M) — `P2-2 §2/§3`, MAC-041 · Callback + self-disable (M) — `P2-2 §3`, MAC-042 · Inject §5 (3 modes + marker + restore) (L), MAC-043 · Permission UX + revocation poll (M) — `P2-2 §4`, MAC-044 · Rule owner `imk|tap` + IPC sync (S, dep: MAC-034)

### Community 105 - "properties"
Cohesion: 0.06
Nodes (32): $comment, default, type, $comment, default, maximum, minimum, type (+24 more)

### Community 106 - "ByteCursor<'a>"
Cohesion: 0.33
Nodes (3): ByteCursor<'a>, Option, Self

### Community 107 - "AssistTests"
Cohesion: 0.08
Nodes (22): AssistIssue, AssistConfigParser, AssistDisplay, AssistSettings, .isAutoFixSafe, .isDisplayEnabled, .isHighlight, .isMode2AutoFix (+14 more)

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "atomic"
Cohesion: 0.16
Nodes (13): ClassFactory, HR_CLASS_E_NOAGGREGATION, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, ObjGuard, AtomicI32, Default (+5 more)

### Community 111 - "entry"
Cohesion: 0.50
Nodes (4): entry, additionalProperties, required, type

### Community 112 - "id"
Cohesion: 0.50
Nodes (4): maxLength, minLength, type, id

### Community 113 - "article_an.rs"
Cohesion: 0.10
Nodes (16): A_DESPITE_VOWEL_LETTER, an_despite_consonant_letter(), bat_a_an_sai(), BOTH_OK, check(), en_issues(), expected_article(), NON_NOUN_FOLLOWERS (+8 more)

### Community 114 - "language.rs"
Cohesion: 0.11
Nodes (23): classify(), classify_tokens(), EN_FREQ_DATA, en_freq_words(), EN_FUNCTION_WORDS, EN_MIXED_INDICATORS, has_en_indicator(), has_en_indicator_tokens() (+15 more)

### Community 115 - "MarkedState"
Cohesion: 0.10
Nodes (21): ApplyReplace, Int, Void, MarkedState, .graphemeCount, .isEmpty, .scalarCount, .utf16Count (+13 more)

### Community 116 - "ffi/src/settings.rs"
Cohesion: 0.15
Nodes (35): assist_nested_paths_round_trip(), c(), c_api_round_trip(), cstr(), doc_err(), english_words_text_round_trip(), HEADER, ime_settings (+27 more)

### Community 119 - "office_productivity.rs"
Cohesion: 0.13
Nodes (15): apply_outcome(), hd_cty_macros(), key_for_char(), office_auto_capitalize_across_sentences_and_newlines(), office_backspace_mid_word_and_continue_typing(), office_bilingual_sentences_auto_restore_cleanly(), office_custom_english_words_and_escape_restore(), office_legacy_charsets_keep_sentence_tail_and_glyph_len_synced() (+7 more)

### Community 120 - "String"
Cohesion: 0.05
Nodes (68): AssistDisplay, AssistSettings, .isOn, CodingKeys, accepts, chars_saved, corrections, count (+60 more)

### Community 121 - "style_profile.rs"
Cohesion: 0.14
Nodes (21): c(), correction_for_linear(), CorrectionLookup, MAX_CORRECTIONS, MAX_ENTRY_CHARS, MAX_TOTAL_BYTES, MAX_WORDS, p7_bytes_cap_16kb() (+13 more)

### Community 122 - "P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN"
Cohesion: 0.06
Nodes (31): 0. Tóm tắt điều hành, 10. Rủi ro & giảm thiểu (cập nhật sau vòng 3–5), 11.1 Vòng 2 — "Nhất quán & Sẵn sàng" (đã xong ở v1), 11.2 Vòng 3–5 — review chuyên sâu (2026-10-20, 3 lượt: SA/QA → Audit → PO/BA/Design), 11.3 Vòng 6 — CROSS-REVIEW (agent B, 2026-10-20) + phản hồi agent A, 11.4 Vòng 7–9 — review **sau khi implement P4-1** (2026-10-20), 11.5 Kết luận các lượt review, 11.6 Vòng 8–10 — review **sau khi làm 4 spike** (2026-10-20) → `P4-3-SPIKES.md` (+23 more)

### Community 127 - "idle_popup_routing.rs"
Cohesion: 0.13
Nodes (17): check_sentence_on_demand(), AUTOCORRECT, che_do_tat_thi_im_lang_o_moi_diem_dung(), dung_1s_o_che_do_goi_y_bao_loi_dung_cho(), dung_1s_o_che_do_sua_loi_an_toan_bao_loi_dung_cho(), feed(), HIGHLIGHT, lap_lai_cung_mot_diem_dung_cho_cung_ket_qua() (+9 more)

### Community 128 - "test_delete_surrounding.cpp"
Cohesion: 0.09
Nodes (21): DeleteCall, offset, size, FakeInputContext, deletes, InputContext, string, string (+13 more)

### Community 129 - "Attribution dữ liệu Tatoeba (CC BY 2.0 FR)"
Cohesion: 0.10
Nodes (17): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN", 1. Bản attribution ngắn (để dán vào README / CHANGELOG / gói phát hành), 2. Nguồn dữ liệu, 3. Vì sao attribution là bắt buộc (+9 more)

### Community 130 - "vni.rs"
Cohesion: 0.14
Nodes (19): late_stroke(), push_key(), fold(), fold_impl(), fold_steps(), free_marking_off_requires_adjacent(), n(), o() (+11 more)

### Community 131 - "TextVNAppTests"
Cohesion: 0.06
Nodes (22): AppInfo, .aboutMessage, .aboutTitle, Data, TextVNAppTests, Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging), Code Review Round 2 — Nhất quán & Sẵn sàng (Kiểm thử, Autostart S5 & Residue S9), Code Review Round 3 — Đúng & Đủ (Rà soát chi tiết từng dòng Carbon, IMK XPC, Protocol Invariants) (+14 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.15
Nodes (19): f(), finish(), fold(), fold_caps(), fold_effective_caps(), FoldEffective, generated_key_tables_are_self_consistent(), is_word_char() (+11 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (17): 10. Sự cố 2026-10-11 — Kaspersky cách ly bản cài từ Microsoft Store (MSIX 1.2.29.0), 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5 (+9 more)

### Community 134 - "ipc/src/lib.rs"
Cohesion: 0.12
Nodes (20): assist_messages_round_trip(), assist_report_decodes_minimal_payload(), assist_report_round_trip_has_no_sentence_text(), AssistIssue, candidate_pipe_names(), CodecError, decode_exact_frame(), encode_frame() (+12 more)

### Community 135 - "win_tools.rs"
Cohesion: 0.07
Nodes (63): deflate_fixed, asset_size(), build_synthetic_pe(), build_zip(), check_pe_imports_cli(), contains_vietnamese(), decode_entities(), EXPECTED_DISPLAY_NAME (+55 more)

### Community 138 - "store_win.rs"
Cohesion: 0.10
Nodes (42): appx, commandext, OsStr, shellexecutew, LaunchMode, after_tray_start(), cmd_exe_path(), CREATE_BREAKAWAY_FROM_JOB (+34 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.20
Nodes (9): §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20), 1. Bảng ms/query — 10 phần tử (n=20/query), 2. Property cold/warm, 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`), 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script), 5. Kết luận cache strategy (acceptance `P1-3 §2`), 6. Limitation & follow-up, 7. Reproduce (+1 more)

### Community 140 - "tray/src/assist.rs"
Cohesion: 0.12
Nodes (22): clean_sentence_is_silent(), decide(), enabled(), issue(), kill_switch_off_wins_over_everything(), LAST_ASSIST_TOAST_MS, long_message_is_truncated_by_chars(), notify_report() (+14 more)

### Community 142 - "guids.rs"
Cohesion: 0.31
Nodes (10): CLSID_TEXTVN_TIP, DISPATTR_TEXTVN, GUID_PRESERVED_ALT1, GUID_PRESERVED_ALT2, GUID_PRESERVED_ALT3, GUID_PRESERVED_TOGGLE, LANGID_EN, LANGID_VI (+2 more)

### Community 143 - "uia_spike.ps1"
Cohesion: 0.60
Nodes (3): Log(), Measure-It(), Test-Target()

### Community 144 - "cli/src/assist.rs"
Cohesion: 0.14
Nodes (22): ArcDict, builtin_default_flags_tone_error_and_no_dict_does_not(), cmd_assist(), cmd_score(), cmd_show(), data(), dic_flags_unknown_vietnamese_word_and_keeps_known_ones(), issue_json() (+14 more)

### Community 145 - "register_wow64_mirror"
Cohesion: 0.24
Nodes (11): icon_path(), register_wow64_mirror(), registration_owned_by(), registration_owned_by_dir(), Scope, unregister_wow64_mirror(), unregister_wow64_mirror_if_owned(), wow_clsid_key() (+3 more)

### Community 147 - "EmojiPickerState"
Cohesion: 0.12
Nodes (14): EmojiItem, EmojiPickerState, hit_test_tabs_and_grid_cells(), keyboard_navigation_scrolling_search_and_commit(), parse_emoji_tsv(), parse_tsv_handles_valid_and_malformed_lines(), PickerHit, PickerKeyAction (+6 more)

### Community 149 - "autostart.rs"
Cohesion: 0.06
Nodes (63): pcwstr, REG_SAM_FLAGS, registry, APP_RUN_VALUE_NAME, autostart_command(), autostart_disabled_by_user(), AUTOSTART_DISABLED_MARKER, autostart_disabled_marker_path() (+55 more)

### Community 150 - "Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)"
Cohesion: 0.33
Nodes (5): 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`), 4. Reproduce, Spike WIN-005 — WH_KEYBOARD_LL + SendInput + UIA trên GHA (`spikes/hook-probe`)

### Community 151 - "Changelog"
Cohesion: 0.02
Nodes (82): [0.2.11] — 2026-10-03, [0.2.17] — 2026-10-03, [0.2.18] — 2026-10-03, [0.2.19] — 2026-10-04, [0.2.1] — 2026-09-30, [0.2.21] — 2026-10-04, [0.2.22] — 2026-10-04, [0.2.23] — 2026-10-05 (+74 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.06
Nodes (67): CloseHandle(), PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-09, sau v0.2.27 + vòng rà soát 2), id_exit, ordering, shell, tray_icon_uid, add_tray_icon(), APP_INSTANCE (+59 more)

### Community 154 - "menu.rs"
Cohesion: 0.09
Nodes (29): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_EXPORT_DIAG, ID_HEALTH_STATUS (+21 more)

### Community 155 - "SettingsController"
Cohesion: 0.11
Nodes (18): Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer, MacroTrigger (+10 more)

### Community 156 - "dict.rs"
Cohesion: 0.07
Nodes (15): BridgeDict, BridgeDict<F>, Budgeted, Budgeted<'a, D>, budgeted_boc_no_dictionary_fail_open(), Budgeted<'_, D>, Dictionary, NoDictionary (+7 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.06
Nodes (71): lc_ipc_client, client_disconnect(), get_time_ms(), handle_ipc_message(), ipc_debug_enabled(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override() (+63 more)

### Community 158 - "P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế"
Cohesion: 0.18
Nodes (10): 0. Kết luận 1 dòng, 1. API có sẵn — đã verify (2026-10-21), 2. So sánh "OS-first" vs "tự mang từ điển", 3. Kiến trúc provider (đề xuất), 4. Quyết định mới (bổ sung D32–D37), 5. Finding mới (Fassist-67..72) — rút ra từ nghiên cứu này, 6. Tác động lộ trình, 7. Rủi ro mới (+2 more)

### Community 159 - "os_macos.rs"
Cohesion: 0.08
Nodes (18): MAX_SUGGEST_RESULTS, ObjcObject, ObjcSelector, PROBE_TIMEOUT, Duration, SEL_AVAILABLE_LANGUAGES, SEL_CHECK_SPELLING, SEL_CLOSE_DOC (+10 more)

### Community 160 - "assist_tools.rs"
Cohesion: 0.07
Nodes (24): civil_date_from_epoch(), civil_from_days(), EN_EXCLUDE_IDS, EN_INTENTIONAL_REPEATS, EN_MAX_LEN, EN_MAX_SENTENCES, EN_MIN_LEN, EN_SEED (+16 more)

### Community 162 - "mmap.rs"
Cohesion: 0.07
Nodes (35): CreateFileMappingW(), CreateFileW(), FILE_ATTRIBUTE_NORMAL, file_len(), FILE_MAP_READ, FILE_SHARE_READ, GENERIC_READ, INVALID_HANDLE_VALUE (+27 more)

### Community 163 - "developer-guide.md"
Cohesion: 0.20
Nodes (3): 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

### Community 164 - "package_bootstrap.rs"
Cohesion: 0.26
Nodes (19): cleanup_old_backups(), copy_dir_recursive(), copy_overwrite(), dispatch(), installed_dir_for(), Option, Path, PathBuf (+11 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "Dict"
Cohesion: 0.21
Nodes (13): Dict, EnchantDict, handle_job(), raw_dict_check(), raw_dict_exists(), raw_dict_suggest(), raw_free_dict(), raw_free_string_list() (+5 more)

### Community 167 - "engine.cpp"
Cohesion: 0.03
Nodes (130): lc_text_ends_with(), applyAssistCandidate(), AssistCandidateWord, deleteGlyphs_, originalTail_, replacement_, st_, InputContext (+122 more)

### Community 168 - "Issue"
Cohesion: 0.09
Nodes (36): span_len(), abbrev_khong_nuot_tu_2_chu_cai_va_dau_cham_ngoac_kep(), check(), check_sentence_with(), check_sentence_with_kiem_tra_ca_dau_cau_va_spelling(), check_sentence_with_style(), check_with(), check_with_bao_tu_la_theo_tu_dien() (+28 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.07
Nodes (42): assist_state_update_and_snapshot(), CLIENT, GLOBAL_KEY, global_switch_from_tray_disables_every_app(), image_is_textvn_exe(), IpcClient, IpcState, LAST_HOTKEY_TOGGLE_MS (+34 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "verify.rs"
Cohesion: 0.06
Nodes (59): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+51 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "HWND"
Cohesion: 0.13
Nodes (42): apply_dpi_change(), close_macro_editor(), close_word_list_editor(), copy_text_to_clipboard(), dialog_wnd_proc(), get_chk(), handle_diag_export_done(), handle_tsf_setup_done() (+34 more)

### Community 177 - "P4-12 — Recall trên **văn bản thật** (bộ nhớ có nhãn) + FPR"
Cohesion: 0.06
Nodes (34): 10. Cập nhật 2026-10-23 — nâng recall `diacritic_typo` (đóng **D19**), 11. Cập nhật 2026-10-23 (vòng 23 — audit "như Grammarly"): FPR → 0 %, 12.1 Số đo hiện tại (cùng 2 corpus, CLI debug rebuild), 12.2 Cảnh báo: recall 83,7 % KHÔNG còn là số độc lập, 12.3 Thí nghiệm bổ sung: spelling theo âm tiết bắt được gì mà bảng TYPOS không bắt?, 12.4 Công cụ cho P4-12c, 12. Cập nhật 2026-11-08 — đo `spelling` bằng từ điển (P4-12a) và cảnh báo quá khớp, 13.1 Số đo (cùng máy, build debug) (+26 more)

### Community 178 - "spelling.rs"
Cohesion: 0.22
Nodes (17): check(), check_text(), FakeDictionary, match_case(), MAX_SUGGESTIONS, MIN_WORD_CHARS, ngan_sach_suggest_het_thi_tu_la_van_duoc_bao_khong_kem_goi_y(), no_dictionary_khong_bao_gi() (+9 more)

### Community 179 - ".suggest"
Cohesion: 0.07
Nodes (39): Int, [0.2.55] — 2026-10-30, [0.2.73] — 2026-11-08, Added, Added, Added (công cụ đo, không đổi hành vi sản phẩm), Changed, Fixed (+31 more)

### Community 180 - ".new"
Cohesion: 0.16
Nodes (12): contains_khong_reset_cua_so_con_han(), EnchantSpellChecker, feature_tat_thi_fail_open_toan_phan(), AtomicUsize, Instant, Job, Result, Self (+4 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.32
Nodes (3): Sign-TextVNFile(), Write-Fail(), Write-Ok()

### Community 182 - "AppDelegate"
Cohesion: 0.04
Nodes (45): AppDelegate, .frontAppEnabled, .isVietnameseMode, .vietnameseModeEnabled, Bool, Date, Int, Int32 (+37 more)

### Community 184 - "P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN"
Cohesion: 0.10
Nodes (19): 0. Quyết định (TL;DR), 10. Liên kết, 1. Harper là gì — facts (đã xác minh, không suy đoán), 2. Kiến trúc `harper-core` (master, 2026-10-20) — bản đồ module, 3. Khoảng cách tiếng Việt — bảng "mượn được gì", 4.1 PO (Product Owner), 4.2 BA (Business Analyst), 4.3 SA (Solution Architect) (+11 more)

### Community 185 - "rules_win.rs"
Cohesion: 0.15
Nodes (13): address_or_search(), classify(), contains(), ControlType, known(), lower(), ProbeResult, role() (+5 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "DialogContext"
Cohesion: 0.67
Nodes (4): DialogContext, Arc, IpcServer, show_settings_dialog()

### Community 189 - "dictionary_budget.rs"
Cohesion: 0.18
Nodes (16): bo_voc_dyn_dictionary_truc_tiep_giong_ctx_dict(), bridge_dict_chuyen_tiep_dung_contains_va_suggest(), bridge_dict_voi_no_dictionary_co_tich_cac_duoc(), budget_khong_cho_phep_vuot_max_suggest(), contains_di_qua_trong_suot_va_khong_ton_ngan_sach(), CountingDictionary, dict_cham_thoi_gian_do_duoc_tang_theo_so_lan_goi(), elapsed_do_duoc_thoi_gian_dung_cho_suggest() (+8 more)

### Community 190 - "hygiene.rs"
Cohesion: 0.10
Nodes (34): read, ascii_ps1(), BANNED_APIS, collect_md(), contains_set_hook_call(), contains_word(), doc_link_targets(), doc_links_bat_ca_dang_target() (+26 more)

### Community 191 - "IpcServer"
Cohesion: 0.05
Nodes (44): AssistMessageOutcome, assistReport, toggleAssist, violation, IpcServer, IpcServerDelegate, Any, Bool (+36 more)

### Community 192 - "invariants.rs"
Cohesion: 0.18
Nodes (17): current_tone(), apply(), backspace_removes_exactly_one_visible_char(), engine_never_overflows_result_or_eats_existing_text(), held_key_after_toned_word_keeps_previous_text(), K, key_event(), Lcg (+9 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - "ime_result_v1"
Cohesion: 0.08
Nodes (32): zeroed_result(), cstr, fill_result(), ime_key(), ime_key_v1, ime_result_v1, null_args_and_abi_errors_fail_open(), oversized_outcome_is_rejected_not_truncated() (+24 more)

### Community 195 - "windows-tsf/src/assist.rs"
Cohesion: 0.04
Nodes (58): a2_double_space_va_space_after_punct_sua_dung(), a2_lap_tu_xoa_dung_lan_lap(), ASSIST_KIND_LOWERCASE_SENTENCE, assist_trigger(), AssistTrigger, CHECK_BUDGET_MS, check_via_engine(), chon_loi_vong_tron_va_theo_so() (+50 more)

### Community 196 - ".register"
Cohesion: 0.10
Nodes (20): MacSpellProvider, Bool, MacSpellProviderTests, [0.2.12] — 2026-10-03, [0.2.5] — 2026-10-02, Added, Fixed (portable), Fixed (Windows) (+12 more)

### Community 197 - "EmojiPicker"
Cohesion: 0.14
Nodes (18): clamp_to_monitor(), create_popup_window(), EmojiPicker, lazy_d2d1_create_factory(), lazy_dwm_round_corners(), lazy_dwrite_create_factory(), Default, Drop (+10 more)

### Community 198 - "cli/src/main.rs"
Cohesion: 0.11
Nodes (21): abi_sizes(), cmd_check_dlopen(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister() (+13 more)

### Community 199 - "phrase_repeat.rs"
Cohesion: 0.10
Nodes (10): looks_like_code_or_list(), bat_lap_cum_2_tu_suggestion_only(), check(), check_window(), EN_SKIP_WORDS, IMPERATIVE_AFTER, PHRASE_WHITELIST, Vec (+2 more)

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (19): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+11 more)

### Community 201 - "ImeEngine"
Cohesion: 0.06
Nodes (26): Action, commit, pass, replace, restore, FFI, ImeEngine, ImeError (+18 more)

### Community 202 - "dict/src/lib.rs"
Cohesion: 0.12
Nodes (32): AsRef, builder, checksum_phat_hien_hong_data(), data_assist(), dic_dau_cu_chua_hoa_khong_chua_hoa_moi(), dic_dau_moi_chua_hoa_moi_khong_chua_hoa_cu(), dic_for_each_final_tra_tat_ca_tu(), dic_vi_dau_cu_khong_chua_tu_khong_co_trong_file() (+24 more)

### Community 203 - "Result"
Cohesion: 0.09
Nodes (22): Cell, Default, GUID, IEnumTfDisplayAttributeInfo, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, ObjGuard, Option (+14 more)

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "dual_mode_matrix.rs"
Cohesion: 0.17
Nodes (30): apply(), boundary_expect(), BS, CASES, count_case(), en_common_corpus(), en_freq_corpus(), ESC (+22 more)

### Community 206 - "trigger"
Cohesion: 0.13
Nodes (15): $comment, default, type, enabled, suggest, trigger, additionalProperties, $comment (+7 more)

### Community 207 - "OutputStrategy"
Cohesion: 0.29
Nodes (7): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, Int64

### Community 208 - "phrases.rs"
Cohesion: 0.16
Nodes (18): drop_issues_in_protected(), EN_DATA, en_set(), is_fixed_phrase(), is_fixed_phrase_for(), is_lexical(), load(), MAX_PHRASE_WORDS (+10 more)

### Community 209 - "field-detect/src/lib.rs"
Cohesion: 0.11
Nodes (19): probe_focused_element(), CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), FieldContext, normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated(), ProbeCache (+11 more)

### Community 210 - "e2e_ibus.c"
Cohesion: 0.17
Nodes (30): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+22 more)

### Community 211 - "IpcServerAssistTests"
Cohesion: 0.11
Nodes (13): IpcServerAssistTests, Any, Int, Int32, IpcServer, URL, ack, pong (+5 more)

### Community 212 - "ipc_server.rs"
Cohesion: 0.10
Nodes (34): filesystem, pipes, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, broadcast_reaches_subscriber(), ClientSink, connected_client_pid(), crash_counter_increments_atomically() (+26 more)

### Community 213 - "e2e-assist-p6.ps1"
Cohesion: 0.31
Nodes (6): Get-ViEnabled(), Norm(), NP-Case(), Report(), Set-ViEnabled(), W-Case()

### Community 214 - "e2e_fcitx5.c"
Cohesion: 0.11
Nodes (45): app_handles(), build_config_dir(), check(), clear(), ctrl_current_im(), ctrl_set_current_im(), ctrl_shift_tap(), ctrl_tap() (+37 more)

### Community 215 - "emoji_popup.rs"
Cohesion: 0.06
Nodes (53): common, direct2d, directwrite, dwm, toggle_tray_emoji_popup, CATEGORY_TABS, CELL_H, CELL_W (+45 more)

### Community 216 - "key_event.rs"
Cohesion: 0.11
Nodes (46): is_modifier_vk(), active_modifiers(), clears_assist_undo(), compartment_i32(), context_blocks_typing(), defer_to_key_down(), eaten_label(), end_composition() (+38 more)

### Community 217 - "chords.rs"
Cohesion: 0.08
Nodes (36): all_symbol_chords_three_modes(), altgr_chord(), altgr_layer_full(), armed(), double_leader_literal(), dream_rsi_delta(), dream_rsi_delta_counts(), emit() (+28 more)

### Community 218 - "edit_session.rs"
Cohesion: 0.07
Nodes (38): apply_display_attribute(), apply_plan(), ApplySentenceFixSession, backspace_shift(), caret_anchor(), DEFERRED_FIX_MS, EmojiCommitSession_Impl, estimate_font_px() (+30 more)

### Community 219 - "undo.rs"
Cohesion: 0.28
Nodes (13): chars(), is_nucleus(), mark_cluster(), mark_horn(), mark_horn_pair_and_single(), mark_vowel(), mark_vowel_applies_and_undoes(), Marked (+5 more)

### Community 223 - "username_identity_1000.rs"
Cohesion: 0.22
Nodes (18): a1000_builder_deterministic_va_id_stable(), a1000_username_giu_nguyen_byte_identical(), apply_outcome(), build_cases(), cap(), Case, dd(), Expect (+10 more)

### Community 224 - "LoadError"
Cohesion: 0.20
Nodes (10): LoadError, corrupt, missing, ApplyError, cannotDelete, rejected, KeyTranslatorError, noChar (+2 more)

### Community 225 - ".new"
Cohesion: 0.13
Nodes (17): contains_giua_cua_so_khong_reset_bo_dem(), en_us_neu_may_co_thi_provider_hoat_dong(), MacSpellChecker, phuong_thuc_tat_ca_fail_open_khong_panic(), AtomicUsize, Instant, Job, Mutex (+9 more)

### Community 226 - "settings_window.c"
Cohesion: 0.10
Nodes (50): add_escape_closes(), button(), gboolean, GCallback, GFile, GFileMonitor, GFileMonitorEvent, gpointer (+42 more)

### Community 227 - "number_vn.rs"
Cohesion: 0.15
Nodes (11): civil_from_days(), days_since_epoch_now(), DIGITS, find_date_shorthand(), find_number_shorthand(), ngay_thang_nam_din_hinh(), read_triple(), read_vnd_words() (+3 more)

### Community 229 - "foreground.rs"
Cohesion: 0.14
Nodes (17): accessibility, foregroundtracker, HWINEVENTHOOK, threading, app_id_from_image_path(), app_of_window(), ForegroundTracker, is_shell_window_class() (+9 more)

### Community 230 - "settings_window_gtk3.c"
Cohesion: 0.10
Nodes (50): add_escape_closes(), button(), gboolean, GCallback, GFile, GFileMonitor, GFileMonitorEvent, gpointer (+42 more)

### Community 231 - "Foundation"
Cohesion: 0.10
Nodes (19): assistKindName(), UInt32, InjectedMarker, TranscodeSelectionTests, LayoutCache, .current, Data, Int64 (+11 more)

### Community 232 - "ThreadState"
Cohesion: 0.09
Nodes (12): mode1_display_enabled(), mode2_auto_fix_enabled(), AssistSettings, Option, Self, TextOwnership, TextRange, ThreadState (+4 more)

### Community 233 - "Path"
Cohesion: 0.16
Nodes (18): append_log_line(), is_reparse_link(), log_rotates_when_larger_than_limit(), log_writer_checks_reparse_before_creating_dirs(), machine_scope_dll(), machine_scope_dll_allowed(), normalize_win_path(), path_is_under() (+10 more)

### Community 234 - "EmojiPickerModel"
Cohesion: 0.07
Nodes (30): EmojiCanvasView, .isFlipped, EmojiCategoryTab, EmojiPickerAction, close, commit, redraw, reload (+22 more)

### Community 235 - "P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`"
Cohesion: 0.12
Nodes (16): 1. Công cụ đo: `textvn-cli assist score --corpus <dir>`, 2.1 Vòng 2 (2026-10-21) — đo trên **văn bản thật** (đóng F80), 2.2 Vòng 3 (2026-10-21) — mở rộng luật ngữ pháp + `--strict` / `--per-kind`, 2.3 Vong 4 (2026-10-21) - do tren **1.500 cau that** (Tatoeba) -> 4 loi engine that, 2.4 Tu dien FST (hunspell-vi) — ket qua tra cuy (P4-4 AS-040/AS-041), 2. Kết quả đo (2026-10-21), 3. Văn bản sạch dùng để đo FPR — `corpus/assist/vi_clean_synthetic.ass`, 4.1 Finding vòng 2 (SA/QA) — phát hiện nhờ đo trên văn bản thật (+8 more)

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.25
Nodes (8): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor, Sự cố đã gặp khi phát hành — tra trước khi xử lý

### Community 237 - "properties"
Cohesion: 0.22
Nodes (9): properties, $comment, default, enum, $comment, default, enum, auto_fix (+1 more)

### Community 238 - "Hướng dẫn sử dụng TextVN"
Cohesion: 0.12
Nodes (17): 1. Cài đặt, 2. Bật/tắt tiếng Việt, 3. Kiểu gõ, 4. Bảng điều khiển, 5. Gõ tắt, 6. Cấu hình, 7. Xử lý sự cố, Chẩn đoán trợ lý trên Linux (nếu gợi ý không hiện) (+9 more)

### Community 239 - "keymap_mac.rs"
Cohesion: 0.20
Nodes (4): MAC_KEY_COUNT, mac_to_canonical(), Option, modifier_dung_vk_is_modifier_nhan_duoc()

### Community 240 - "linux_tools.rs"
Cohesion: 0.18
Nodes (21): parse_xml, check_elf_abi(), check_ibus_component(), child_text(), component_duong_dan_dac_biet(), descendant_first(), e67_check_elf_abi_accepts_glibc_228_and_blocks_newer_or_execstack(), ElfAbiInfo (+13 more)

### Community 241 - "EventTapController"
Cohesion: 0.04
Nodes (47): CGFlagMapper, EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, Bool, Int, NSObjectProtocol (+39 more)

### Community 242 - "en_office.rs"
Cohesion: 0.16
Nodes (19): add(), add_filters_non_word_keys(), base_w(), body_matches_committed_tsv(), build_rows(), is_lowercased_word(), MAX_PRED, max_pred_filter() (+11 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.03
Nodes (79): 1. Gói phát hành, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Báo cáo dựng & kiểm thử — TextVN, Bản 0.2.16 — portable tự chữa trên Win11 24H2+ (B7) + hết ghost khi gỡ (0.2.15), Bản 0.2.1 — phát hành pre-release đã xác minh (+71 more)

### Community 244 - "remove_added_languages"
Cohesion: 0.24
Nodes (14): added_language_removable(), is_input_method_value(), languages_added_marker_path(), languages_missing(), languages_without(), parse_languages_marker(), read_languages_added(), record_languages_added() (+6 more)

### Community 245 - "dream_rsi_100.rs"
Cohesion: 0.26
Nodes (22): apply(), ctrl_space(), d100_sweep_tab_accept_tich_luy(), d_drain_json_words_va_rsi_song_chung(), d_en_accept_savings(), d_engine_moi_zero_va_no_div(), d_g8_chi_counter_khong_noi_dung(), d_keys_typed_loai_phim_khong_phai_go_chu() (+14 more)

### Community 246 - "Dictionary"
Cohesion: 0.23
Nodes (5): Dictionary, EnchantSpellChecker, MacSpellChecker, Vec, WindowsSpellChecker

### Community 247 - "ime_conflict_1000.rs"
Cohesion: 0.17
Nodes (18): a200_is_plain_letter_matrix(), b300_engine_pass_mapping(), build_plain_matrix(), build_sequences(), c200_inject_shape(), d300_end_to_end_sequences(), e48_hook_origin_guard_prevents_textvn_duplication(), hook_engine_available() (+10 more)

### Community 248 - "tsf-min/src/lib.rs"
Cohesion: 0.09
Nodes (29): ITfKeyEventSink, ClassFactory, CLSID_TEXTVN_TIP, CompSink, DllCanUnloadNow(), DllGetClassObject(), EditSession, HR_CLASSNOTAVAILABLE (+21 more)

### Community 249 - "parse_ctrl_shift_marker"
Cohesion: 0.30
Nodes (12): ctrl_shift_marker_record_round_trip(), ctrl_shift_marker_says_freed(), format_ctrl_shift_marker(), give_ctrl_shift_back_to_windows(), merge_ctrl_shift_record(), parse_ctrl_shift_marker(), read_ctrl_shift_record(), record_ctrl_shift_freed() (+4 more)

### Community 250 - "smoke-imk.sh"
Cohesion: 0.48
Nodes (5): clear_doc(), read_doc(), say(), smoke-imk.sh script, type_keys()

### Community 251 - "templates.rs"
Cohesion: 0.14
Nodes (21): bmat_pmat_env_names(), cell(), count_utf32(), cursor_never_exceeds_length(), det_must_be_square(), generate(), hpt_and_cases(), in_bounds() (+13 more)

### Community 252 - "Kế hoạch ký số (code signing) — TextVN"
Cohesion: 0.20
Nodes (10): 1. So sánh các đường chính thống (cập nhật 2026-10-02), 2. Lộ trình đề xuất — SignPath Foundation, 3. Smart App Control / SmartScreen — kỳ vọng thực tế, 4. Việc đã wire trong repo, Bước 1 — Nộp đơn (làm một lần, maintainer tự làm), Bước 2 — Tạo project + artifact configuration trên SignPath.io, Bước 3 — Bật ký trong GitHub Actions (đã nối sẵn — chỉ cần đặt secret), Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai) (+2 more)

### Community 253 - "os_windows.rs"
Cohesion: 0.08
Nodes (44): CORRECTIVE_ACTION, cached_support(), collect_issues(), collect_suggestions(), create_checker(), create_factory(), en_us_neu_may_co_thi_provider_hoat_dong(), handle_job() (+36 more)

### Community 254 - "SettingsView"
Cohesion: 0.10
Nodes (17): ConflictScan, EnglishWordsSheet, .body, MacroEditorSheet, .body, .trimmedExpand, .trimmedTrigger, RadioButton (+9 more)

### Community 255 - "test-typing.ps1"
Cohesion: 0.36
Nodes (4): Check(), Clear-Verified(), Codes(), Norm()

### Community 256 - "env-mac — Môi trường build macOS (MAC-001)"
Cohesion: 0.40
Nodes (4): Acceptance MAC-001, env-mac — Môi trường build macOS (MAC-001), Lệnh kiểm nhanh, Yêu cầu

### Community 257 - "MathContextFsm"
Cohesion: 0.13
Nodes (11): enter_exit_math(), escaped_block_paren_and_text_spans(), feed_dollar_toggles(), is_math_env_name(), MathContextFsm, MAX_STACK, Default, Option (+3 more)

### Community 258 - "Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)"
Cohesion: 0.15
Nodes (13): 1. Bố cục, 2. Tuỳ chọn ↔ khoá cấu hình, 3. Hành vi chung, 4. Cửa sổ Gõ tắt, 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+), 6. Nhận diện EN/VI (không có UI riêng — hành vi engine), 7. Icon — một thiết kế cho 3 nền tảng, 8.1 Nội dung panel (+5 more)

### Community 259 - "windows-tsf/src/replay.rs"
Cohesion: 0.13
Nodes (28): CLASS_NAME, DeferredReplay, deliver(), HOPS, MODIFIER_POLL_MAX, MODIFIER_POLL_MS, modifiers_down(), module_instance() (+20 more)

### Community 260 - "tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)"
Cohesion: 0.40
Nodes (4): 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật (cập nhật 2026-10-09), tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

### Community 262 - "text_prediction.rs"
Cohesion: 0.17
Nodes (27): apply_outcome(), engine_with_predict(), key_ctrl_space(), key_for_char(), predict_engine(), press_key(), r2_120_khong_du_doan_qua_dong_moi(), r2_121_ctrl_space_khi_tat_tieng_viet_khong_ghi_cach_ao() (+19 more)

### Community 263 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.15
Nodes (13): 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release, Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF (+5 more)

### Community 264 - "e57_math_sync.rs"
Cohesion: 0.31
Nodes (12): backspace(), buf_str(), e57a_arrow_clears_math_word(), e57a_backspace_syncs_math_word(), e57a_template_still_expands_without_backspace(), e57b_inline_eval_no_delete_overflow(), e57c_telex_suspended_inside_eval_word(), left() (+4 more)

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - "test_compose.c"
Cohesion: 0.05
Nodes (81): lc_comp, len, text, lc_key, ch, kind, lc_modifier_toggle, armed (+73 more)

### Community 270 - "default"
Cohesion: 0.18
Nodes (11): additionalProperties, $comment, default, type, auto_fix, language, max_sentence_chars, mode (+3 more)

### Community 274 - "svc.rs"
Cohesion: 0.09
Nodes (18): atomic_write_file(), corrupt_config_is_backed_up_not_lost(), default_config_dir(), failed_config_save_keeps_memory_and_version_unchanged(), failed_state_save_keeps_memory_and_version_unchanged(), migration_kill_switch_bat_lai_dung_mot_lan(), Arc, BTreeMap (+10 more)

### Community 290 - "CandidateOverlay"
Cohesion: 0.11
Nodes (21): ActionSink, CandidateOverlay, create_font(), left_button_down(), measure(), metrics_for_hwnd(), overlay_wnd_proc(), paint_overlay() (+13 more)

### Community 291 - "ConfigWatcher"
Cohesion: 0.23
Nodes (7): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem

### Community 292 - "path"
Cohesion: 0.14
Nodes (17): all_cases, btreeset, fs, path, add_splits_from(), FnMut, Result, run() (+9 more)

### Community 293 - "OverlayView"
Cohesion: 0.12
Nodes (11): acceptance_dod_scenarios_1_to_7(), AssistUndoState, cau_dai_chi_ve_duoi_va_marker_lech_dung(), fix_candidate(), OverlayView, per_app_tat_assist_rieng_theo_config(), Instant, Self (+3 more)

### Community 294 - "command"
Cohesion: 0.09
Nodes (21): find_rc_exe(), main(), Option, PathBuf, find_rc_exe(), main(), Option, PathBuf (+13 more)

### Community 295 - "check_version_sync.rs"
Cohesion: 0.22
Nodes (12): check_site(), extract(), PLIST_KEY, repo_hien_tai_dong_bo(), Option, Result, run(), Site (+4 more)

### Community 296 - "NSRange"
Cohesion: 0.06
Nodes (15): IMKTextTarget, Bool, Int, TextCommandTarget, NSRange, CapitalizeMockTarget, Bool, Int (+7 more)

### Community 297 - "Rgba"
Cohesion: 0.25
Nodes (10): apply_rounded(), rounded_mask(), Canvas, downsample2_chinh_xac_alpha_thap(), paste_with_mask_doc_kenh_alpha_va_khong_panic_lech_chieu_cao(), png_roundtrip_markers_and_pixels(), Rgba, rounded_rect_coverage() (+2 more)

### Community 298 - "P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công"
Cohesion: 0.13
Nodes (14): 0. Chuỗi phụ thuộc (đã xong / sắp làm), 1.1 API công khai (đề xuất, khớp D23/D27/D29), 1.2 Quy tắc bắt buộc, 1.3 Chỗ sẽ sửa (đã đọc trong repo), 1.4 Kiểm chứng, 1. AS-021 — FFI `ime_assist_*` (C-ABI), 2.1 Luồng, 2.2 Quy tắc hành vi (D11 + D12) (+6 more)

### Community 299 - "properties"
Cohesion: 0.15
Nodes (13): properties, $comment, default, type, hide_on_auto_fix, show_on_comma, show_on_sentence_end, $comment (+5 more)

### Community 300 - "tone_inject_one"
Cohesion: 0.21
Nodes (12): &T, is_combining_mark(), Rng, rng_deterministic_and_sample_distinct(), BTreeMap, Option, strip_tone(), tone_inject_changes_one_tone() (+4 more)

### Community 301 - "SettingsDoc"
Cohesion: 0.16
Nodes (10): default_at(), defaults(), lookup_nested(), MacroEntry, Option, Vec, SettingsDoc, Display (+2 more)

### Community 302 - "emoji_picker.rs"
Cohesion: 0.08
Nodes (31): CATEGORY_TABS, CELL_H, CELL_W, CLASS_NAME, D2dResources, FOOTER_HEIGHT, GRID_COLS, GRID_TOP (+23 more)

### Community 303 - "worker_main"
Cohesion: 0.18
Nodes (13): Job, Pool, raw_pool_pop(), raw_pool_push(), c_void, Drop, Receiver, SyncSender (+5 more)

### Community 304 - "3. Nhật ký finding"
Cohesion: 0.13
Nodes (14): 0. Kết luận, 1. Phương pháp, 2. Bằng chứng CI, 3.1 Microsoft Store / MSIX, 3.2 Bảo mật Windows, 3.3 QA Windows (tray, CLI, bộ cài), 3.5 Tiếng Việt / engine, 3.6 CI, ký số, phát hành (+6 more)

### Community 305 - "ghost.rs"
Cohesion: 0.09
Nodes (24): CLASS_NAME, COLOR_GHOST, COLOR_KEY, ghost_font(), ghost_wnd_proc(), GhostText, MAX_GHOST_CHARS, measure() (+16 more)

### Community 306 - "Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)"
Cohesion: 0.08
Nodes (24): Nộp TextVN lên Microsoft Store — gói MSIX, 0. Checklist S — BẮT BUỘC 100% trước khi nộp (không bỏ bước nào), 1. Điền form Partner Center — từng ô một, 1a-bis. Trạng thái kỳ vọng của "Package validation", 1a. "Provide any switches required for silent installation…", 1a-ter. Lịch sử 7 vòng + kế hoạch nộp có thứ tự, 1b. "Installer runs in silent mode but does not require switches", 1c. "Select the languages that your application supports" (+16 more)

### Community 307 - "verify-store-package.sh"
Cohesion: 0.60
Nodes (4): die(), GNUPGHOME, verify-store-package.sh script, valid_sig()

### Community 308 - "cuas.rs"
Cohesion: 0.12
Nodes (43): class_name(), edit_selection(), edit_text_before_caret(), EM_GETSEL, EM_GETSELTEXT, EM_REPLACESEL, EM_SETSEL, give_up() (+35 more)

### Community 309 - "predict_next_phrase_candidates"
Cohesion: 0.12
Nodes (20): complete_vn_word, bang_van_phong_phu_5_nhom_va_bilingual(), predict_next_phrase_candidates(), predict_prefix_candidates(), test_predict_next_phrase_english(), test_predict_next_phrase_vietnamese(), test_predict_prefix_compounds_vietnamese(), test_predict_prefix_english() (+12 more)

### Community 310 - "Result"
Cohesion: 0.13
Nodes (21): ClassFactory_Impl, CompSink_Impl, EditSession_Impl, KeySink_Impl, log(), BOOL, IClassFactory_Impl, ITfComposition (+13 more)

### Community 311 - ".unregister"
Cohesion: 0.09
Nodes (21): [0.2.15] — 2026-10-03, [0.2.26] — 2026-10-05, [0.2.3] — 2026-10-01, Added, Added (Windows) — hỗ trợ app 32-bit (Zalo/Office x86) — vòng 14, Changed (CI — audit GitHub Actions, 5 major), Fixed (Windows), Fixed (Windows) (+13 more)

### Community 312 - "sentence_grammar.rs"
Cohesion: 0.15
Nodes (13): check_comma_after_conjunction(), check_en(), check_en_agreement(), check_vi(), EnCorrelativeClause, merge_contractions(), preserve_case(), push_delete_issue() (+5 more)

### Community 313 - "IpcMessage"
Cohesion: 0.13
Nodes (18): AssistReportIssue, .json, IpcMessage, assistReport, configReload, crashReport, getSnapshot, hello (+10 more)

### Community 315 - "Option"
Cohesion: 0.14
Nodes (23): broadcast_international(), ctrl_shift_taken_by_windows(), delete_reg_value(), ensure_modern_language_list(), enum_subkeys(), grant_appcontainer_read(), hkey_users_relative_paths_reach_the_loaded_hive(), reg_read_multi_sz() (+15 more)

### Community 316 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.13
Nodes (15): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release (+7 more)

### Community 317 - "emoji_catalog.rs"
Cohesion: 0.10
Nodes (26): bilingual_diacritic_insensitive_search(), CAT_ACTIVITIES, CAT_FOOD, CAT_GESTURES, CAT_NATURE, CAT_RECENT, CAT_SMILEYS, CAT_SYMBOLS (+18 more)

### Community 318 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.15
Nodes (13): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG (+5 more)

### Community 319 - "FieldDetect"
Cohesion: 0.16
Nodes (10): Entry, FieldDetect, Date, pid_t, Set, TimeInterval, AXUIElement, [0.2.31] — 2026-10-11 (+2 more)

### Community 320 - "settings_model.c"
Cohesion: 0.11
Nodes (48): lc_state_write_enabled(), tv_paths, edit_bool(), edit_reset(), edit_str(), edit_u32(), get_bool(), get_index() (+40 more)

### Community 321 - "src/style_merge.rs"
Cohesion: 0.16
Nodes (17): MergeOutcome, merge_at(), merge_from_json(), PENDING_RSI, Arc, Duration, Instant, Mutex (+9 more)

### Community 322 - "en_office_phrases_10k.rs"
Cohesion: 0.30
Nodes (14): apply(), che_do_en_mo_rong_cau_van_phong(), che_do_vi_code_switch_ban_goi_y_en(), letter(), office_rows(), press(), Action, KeyEvent (+6 more)

### Community 323 - "check_mac_targets.rs"
Cohesion: 0.13
Nodes (26): APP_DB, appdb(), CI_APPS, collect_preset_ids(), FIELD_ROLES, good(), has_preset_id(), Json (+18 more)

### Community 324 - "macro.rs"
Cohesion: 0.15
Nodes (15): Emoji, emoji_glyph_is_multi_char(), apply_macro_case(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches() (+7 more)

### Community 325 - "Diagnostics"
Cohesion: 0.10
Nodes (15): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, UInt64, Audit tiếp diễn — TextVN, 2026-09-30 (+7 more)

### Community 326 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.14
Nodes (12): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG, 6. PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT) (+4 more)

### Community 328 - "syllable_spelling.rs"
Cohesion: 0.26
Nodes (11): CLEAN, clean_corpus_fpr_at_most_half_percent(), clean_sentences(), EXTRA, injected_tone_errors_are_caught(), key_and_tone(), ROWS, Vec (+3 more)

### Community 329 - "viqr.rs"
Cohesion: 0.13
Nodes (12): apply_key, fold(), fold_impl(), fold_steps(), marker_mark_index(), marker_to_tone(), n(), DiacriticStyle (+4 more)

### Community 330 - "AXSnapshot"
Cohesion: 0.27
Nodes (3): AXSnapshot, FieldRules, FieldRulesTests

### Community 331 - "preset_matrix.rs"
Cohesion: 0.26
Nodes (13): b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace(), caps_missing_downgrades(), db(), disabled_app_passthrough(), finder_rename_editbox(), MAC_CAPS (+5 more)

### Community 332 - "tip.rs"
Cohesion: 0.08
Nodes (33): ASSIST_KEYS, current_exe_name(), Default, F, GUID, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeInfo (+25 more)

### Community 333 - "diacritic_typo.rs"
Cohesion: 0.09
Nodes (23): base_group(), bat_ca_cau_corpus_that_moi_nhat(), bat_sai_dau(), check(), giu_hoa_thuong(), has_diacritic(), khong_beu_tu_hop_le_tung_o_bang_typos(), MAX_UNKNOWN_TOKENS (+15 more)

### Community 335 - "AssistCheck"
Cohesion: 0.23
Nodes (15): after_trigger(), AssistCheck, check_sentence(), check_text(), check_text_fragment(), check_text_min_words(), d16_auto_capitalize_bat_thi_khong_con_loi_l6(), d16_auto_capitalize_tat_thi_van_con_loi_l6() (+7 more)

### Community 336 - "macSpellContainsCallback"
Cohesion: 0.44
Nodes (8): macSpellContainsCallback(), macSpellSuggestCallback(), CChar, Int32, UInt32, UnsafeMutableRawPointer, UnsafePointer, UnsafeMutablePointer

### Community 337 - "plan_sentence_auto_fix"
Cohesion: 0.21
Nodes (15): all_issue_views_giu_du_hon_3_loi_cho_mode2_va_fix_all(), c_buf_to_string(), issue(), issues_from_ffi(), IssueView, plan_sentence_auto_fix(), plan_sentence_auto_fix_bo_qua_loi_chi_goi_y(), plan_sentence_auto_fix_chi_sua_phan_duoi() (+7 more)

### Community 338 - "3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft"
Cohesion: 0.12
Nodes (17): 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU), 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?, 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft, 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối, 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store, 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation, 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?, 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft (+9 more)

### Community 339 - "panel_idle.rs"
Cohesion: 0.24
Nodes (8): hwnd_alive(), PANEL_IDLE_MS, register_timer_hwnd(), ITfContext, Rc, schedule(), TIMER_PANEL_IDLE, refcell

### Community 340 - "dual_mode_rapid.rs"
Cohesion: 0.10
Nodes (44): Context, EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset, apply() (+36 more)

### Community 341 - "trace.rs"
Cohesion: 0.18
Nodes (10): enabled(), event(), File, Mutex, Option, sink(), Arguments, sync (+2 more)

### Community 342 - "default"
Cohesion: 0.13
Nodes (15): corrections, decided, enabled, max_total_bytes, rsi, words, style_profile, accepts (+7 more)

### Community 343 - "rsi"
Cohesion: 0.11
Nodes (18): minimum, type, minimum, type, accepts, chars_saved, keys_typed, minimum (+10 more)

### Community 344 - "package_family_full_names"
Cohesion: 0.24
Nodes (10): MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, PWSTR, message_box(), open_apps_settings(), package_family_full_names(), report_error(), Vec (+2 more)

### Community 345 - "[0.2.37] — 2026-10-24"
Cohesion: 0.67
Nodes (3): [0.2.37] — 2026-10-24, Added, Fixed

### Community 346 - "readme_numbers.rs"
Cohesion: 0.42
Nodes (8): comma(), count(), dot(), group(), readme_corpora_numbers_match_data_files(), readme_has_no_stale_dictionary_claims(), root(), Path

### Community 347 - "TextVN — Bộ gõ tiếng Việt thế hệ mới cho Windows, macOS và Linux"
Cohesion: 0.10
Nodes (21): 0. Universal Scientific, Engineering & Math Engine (mới ở 0.2.83 — crate `textvn-math`), 1. Công thái học Dream RSI & Kiểm chuẩn Hiệu năng (P10 / G19), 1. Kiến trúc In-Process thuần khiết & An toàn bộ nhớ tuyệt đối, 2. Hệ thống Dự đoán Ngữ cảnh Song ngữ & Công thái học Dream RSI (P10), 2. Mô hình Dự đoán Ngữ cảnh P6 & Hồ sơ Phong cách P7 (Corpora 53.955 mục), 3. Kiểu Gõ & Biến Đổi Âm Vị Học Chuẩn Mực, 3. Trợ lý Ngữ pháp & Ngữ cảnh On-Device (Grammarly-Grade Offline), 4. Di sản Chuẩn UniKey 20 Năm & Tiện ích Văn phòng Hiện đại (+13 more)

### Community 348 - "P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A"
Cohesion: 0.22
Nodes (8): 0. Kết luận của agent A sau khi verify lại từng fact, 1. Disposition 6 finding của agent B, 2. Finding mới của agent A (Fassist-38..45) — phát hiện khi review lại đề xuất của B, 3. Quyết định mới D21–D29 (bổ sung D1–D20), 4. Lộ trình cập nhật (chèn vào `P4-0` §8), 5. Trạng thái finding sau cross-review, 6. Liên kết, P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A

### Community 349 - "Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store"
Cohesion: 0.17
Nodes (12): 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào, 2. Vòng 15 — những gì đã triển khai trong `release.yml`, 3. Khoá ký GPG release (TextVN Release Signing), 4. Authenticode cho Microsoft Store (chưa xong — đang chờ), 4b. macOS — Developer ID + notarization, 5. Quy tắc khóa-tay (đừng lặp), 6.1. Sự cố & Nguyên nhân gốc rễ (Sự cố R6 — 2026-10-11), 6.2. Quy tắc bắt buộc cho mọi release từ v0.2.32 (+4 more)

### Community 350 - "en_spelling.rs"
Cohesion: 0.17
Nodes (22): all_caps_va_chu_hoa_giua_khong_bao(), am_tiet_viet_khong_dau_khong_bao(), base_dict(), cau_tieng_viet_khong_chay_luat(), check(), check_text(), damerau_distance(), EN_SENTENCE (+14 more)

### Community 351 - "e63_expert_personas_audit.rs"
Cohesion: 0.36
Nodes (10): opts_math(), opts_plain(), type_stream(), type_stream_with_strategy(), view1_math_researcher_latex_paper_and_delimiters(), view2_student_math_physics_chemistry_normal_mode(), view3_bilingual_vi_default_wins_vs_unikey_undo_en_wins(), view3_scientific_dev_terms_continuations_and_unikey_habits() (+2 more)

### Community 352 - "◻ Đề xuất tính năng (ưu tiên theo giá trị user)"
Cohesion: 0.20
Nodes (9): A. Hoàn thiện User Style Profile (P7) — vòng tiếp theo, B. Prediction chất lượng cao hơn, C. Ổn định hệ thống, D. Trải nghiệm văn phòng (persona tiếp theo), E. Chất lượng nội bộ, Thứ tự đề xuất (nếu duyệt), ✅ Đã thực hiện (0.2.65 + vòng 56), ◻ Đề xuất tính năng (ưu tiên theo giá trị user) (+1 more)

### Community 353 - "A. WINDOWS"
Cohesion: 0.08
Nodes (23): A10. [TRUNG] COM callback `ActivateEx`/`Deactivate`/`OnSetFocus` không bọc `catch_unwind` (tip.rs:90-124,174-183) — panic thoát qua ABI = abort host (Notepad/Chrome). Nhánh `try_borrow_mut` fail tại ActivateEx: Advise đã thực hiện nhưng cookie không lưu → Deactivate không Unadvise (leak tới hết thread). **Fix:** catch_unwind như `DoEditSession`; borrow-fail trả `Err(E_FAIL)`., A11. [TRUNG] `svc.rs:135-235` — `RwLock.write().unwrap()`: poison → panic dây chuyền trong `wnd_proc`. **Fix:** `unwrap_or_else(|e| e.into_inner())`., A12. [TRUNG] Uninstaller per-user non-elevated: `schedule-delete` (PFO) luôn ACCESS_DENIED → `.old-*` sót vĩnh viễn (register.rs:793-834). **Fix:** RunOnce như uninstall.ps1 portable đã làm., A13. [THẤP] Đa user với bản máy: user B còn per-user override trỏ DLL đã xoá; CLI cũng bị xoá → không tự lành (main.rs:601-603 return sớm). Fix dài hạn: marker machine-level yêu cầu tray user B tự unregister khi thấy path chết., A14. [THẤP] `reg_value_names` buffer 512 u16 — value dài hơn 511 ký tự, A15. [THẤP] `show_activation_refused_notice_once` (main.rs:337-351) ghi marker, A16. [THẤP] `compose`/`replay` — (đã soát, chỉ ghi nhận) replay post đúng, A1. [CAO] ILOT spec cho bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100) 🔎 (+15 more)

### Community 354 - "Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI"
Cohesion: 0.25
Nodes (7): 1. Microsoft nói chính xác điều gì, 2. Điều này thay đổi nhận định gì, 3. Ba con đường (đối chiếu chi phí / rủi ro / thời gian), 4. Kế hoạch chốt (theo thứ tự ưu tiên — cập nhật 2026-10-09), 5. Hiện trạng kỹ thuật đã sẵn sàng (2026-10-09), 6. Việc cần làm — checklist, Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

### Community 355 - "push_key"
Cohesion: 0.11
Nodes (29): base_keys(), fix_uo(), vni_viqr_marks_after_whole_cluster(), Horn, post_fixes(), promote_uo_pair(), push_key(), apply_key() (+21 more)

### Community 357 - "3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM""
Cohesion: 0.29
Nodes (7): 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store, 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM", 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM", 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant), 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên, Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`, Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC

### Community 358 - "T6 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): LNX-060 · AT-SPI driver `tools/linux/atspi-driver` (L) — `P3-6 §4`, LNX-061 · Targets JSON 12 app (cùng format 3 OS) (M), LNX-062 · Perf bench + `perf/baseline-linux.json` (M) — `P3-6 §5`, LNX-063 · Soak 24h script × 2 (X11 + Wayland VM) (M), LNX-064 · CI jobs linux + dist matrix (M) — `P3-6 §8`, LNX-065 · Fuzz + ASan/valgrind job (S), LNX-066 · Security + license checklist Phần 3 (M) — Handbook §8 + audit x11 (P3-3 §7), T6 — Test & hardening (tuần 15–18)

### Community 359 - "collect_windows_env_summary"
Cohesion: 0.18
Nodes (14): collect_macos_env_summary(), collect_windows_env_summary(), detect_windows_conflict_name(), DoctorReport, e69_markdown_report_renders_diagnostics_safely(), format_windows_version(), MacOsEnvSummary, parse_plist_string_value() (+6 more)

### Community 360 - "FieldContext"
Cohesion: 0.14
Nodes (13): FieldContext, FieldRole, Bool, UInt32, 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng), 2. Trách nhiệm từng crate & ranh giới, 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa), 4. CI (GitHub Actions) (+5 more)

### Community 361 - "plan_key"
Cohesion: 0.19
Nodes (14): CompositionPlan, ctrl_space_plan_never_inserts_space_on_pass(), delete_beyond_composition_is_reported_not_guessed(), e49_tsf_focus_terminal_state_machine_528(), EngineStep, is_context_disabled_or_empty(), plan_invariant_unknown_action_is_fail_open(), plan_key() (+6 more)

### Community 362 - "3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow)"
Cohesion: 0.20
Nodes (9): 1. Tổng quan kiến trúc, 2.1 Không đóng gói tĩnh (No Static Linking), 2.2 Nguồn gốc Upstream, 2. Tuân thủ Giấy phép LGPL (Linux — enchant-2 & libspelling), 3.1 Xử lý cục bộ & Không gửi qua mạng, 3.2 Phạm vi tiến trình (Process Boundaries), 3.3 Quyền kiểm soát & Kill-Switch, 3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow) (+1 more)

### Community 363 - "Added"
Cohesion: 0.33
Nodes (6): [0.1.0] — 2026-09-27, Added, Architecture, Core, Developer Tools, Windows Platform

### Community 364 - ".contains"
Cohesion: 0.22
Nodes (11): ArcDict, en_clean_sentence(), en_is_mechanically_clean(), en_lone_i(), en_missing_space_after(), en_space_before_punct(), gen_en_tatoeba_cli(), Arc (+3 more)

### Community 365 - "inflate_raw_bounded"
Cohesion: 0.24
Nodes (7): BitReader, BitReader<'a>, fixed_tables(), HuffTable, inflate_raw_bounded(), Result, Self

### Community 366 - "English"
Cohesion: 0.09
Nodes (21): 10. Performance budgets (measured, enforced by benchmarks), 1. Bật / tắt (Enable / Disable), 1. Enable / Disable, 2. Layer A — Leader chords (`` ` ``), 2. Layer A — Leader chords (phím leader `` ` ``), 3. Layer B — AltGr (opt-in), 3. Layer B — AltGr (tùy chọn), 4. Alias tiếng Việt (gõ `\tên` + Space) (+13 more)

### Community 367 - "2. Nhật ký finding"
Cohesion: 0.29
Nodes (7): NativeBoundaryTests, Bool, CGEventSource, Int, UInt16, TapInjector, 2. Nhật ký finding

### Community 368 - "e57_word_sync_fca.rs"
Cohesion: 0.31
Nodes (10): backspace_pops_word_buf(), backspace_resets_armed_chord(), caret_moved_clears_word(), engine_sticky(), eval_span_keeps_boundary_chars(), failed_eval_does_not_stick(), inline_eval_delete_excludes_closing_eq(), word_is_command_includes_inline_eval() (+2 more)

### Community 370 - "MathOutputMode"
Cohesion: 0.15
Nodes (14): AppCategory, AppFormatKind, AppProfile, best_mode_for_app(), dream_rsi_formula(), dream_rsi_score(), is_protected_shortcut(), resolve_app() (+6 more)

### Community 371 - "P4-11 — ĐO TÀI NGUYÊN ASSIST (F61) — TIP THẬT"
Cohesion: 0.20
Nodes (9): 1. Phương pháp, 2. Môi trường đo, 3.1 RSS ảnh DLL (phần **quy được** cho DLL), 3.2 Toàn tiến trình (giới hạn trên — **không** quy hết cho DLL), 3. Kết quả (2026-10-22), 4. So với budget (`P4-0-MASTER-PLAN.md` D25 · P4-0 §"Perf"), 5. Việc tiếp theo, 6. Dọn dẹp sau đo (đã làm) (+1 more)

### Community 373 - "Strategy"
Cohesion: 0.21
Nodes (9): downgrade(), resolve(), ResolveInput, Option, Strategy, default_for_field(), FIELD_ROLES, parse_field_role() (+1 more)

### Community 374 - "DictError"
Cohesion: 0.12
Nodes (11): DictError, Display, Error, Formatter, From, Result, Self, ComApartment (+3 more)

### Community 376 - "P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)"
Cohesion: 0.33
Nodes (5): Chi tiết P4-1 (đã xong), P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist), Recipe P4-2a — refactor buffer trong `core` (đọc trước khi code), Task mở (chi tiết acceptance & phân công Agent), Trạng thái tổng quan

### Community 377 - "Phân phối Linux — kênh, store và auto-update"
Cohesion: 0.10
Nodes (19): 0. Nguyên tắc (khác gì PPA), 1. Lộ trình ba bước, 2. Việc upstream đã chuẩn bị (bằng chứng cho maintainer), Bước A — RFP (nhờ cộng đồng đóng gói) hoặc ITP (tự đóng gói rồi xin sponsor), Bước B — mentors.debian.net upload (kèm ITP), Bước C — Liên hệ Debian Input Method Team (nhanh nhất), Nộp TextVN vào Debian / Ubuntu chính thức (RFP → ITP → Sponsor), 1. Ràng buộc quyết định kênh: TextVN là IME hệ thống (+11 more)

### Community 378 - "mac_corpus_cases.rs"
Cohesion: 0.49
Nodes (9): all_cases(), bs(), case(), CorpusCase, du_nhom_case_theo_p2_5(), imk(), Vec, tap() (+1 more)

### Community 380 - "deflate_fixed"
Cohesion: 0.36
Nodes (7): BitWriter, deflate_fixed(), encode_dist(), encode_len(), hash3(), inflate_raw_bounded_chan_zip_bomb_va_dynamic_hdist32_eob(), write_fixed_litlen()

### Community 381 - "KeyTranslator"
Cohesion: 0.12
Nodes (12): KeyMapMacGenerated, UInt32, KeyTranslator, Data, KeyEvent, NSEvent, UInt16, UInt32 (+4 more)

### Community 384 - "P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19)"
Cohesion: 0.20
Nodes (9): 1. Dream RSI là gì, 2. Các accept-point + công thức `chars_saved`, 3. Luật `keys_typed` (G19b), 4. Drain + persist, 5. Gate + bench (G19c), 6. Audit view (G19d), Bẫy đã ghi nhận (đừng lặp), P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19) (+1 more)

### Community 385 - "P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4"
Cohesion: 0.20
Nodes (9): 0. Tóm tắt kết luận, 1. S1 — TSF candidate list sau commit, 2. S2 — Đọc vị trí caret (precondition D12), 3. S3 — Nguồn từ điển tiếng Việt, 4. S4a — Chi phí per-process (đo thật, Windows x64), 5. S4b — `harper-core` cho tiếng Anh (v2, không chặn v1), 6. Quyết định mới từ spike, 7. Việc còn lại sau 4 spike (+1 more)

### Community 386 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.33
Nodes (5): C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 387 - "ime_assist_issue"
Cohesion: 0.22
Nodes (9): ime_assist_issue, auto_fixable, confidence, end, kind, message, start, suggest (+1 more)

### Community 388 - "Ctx"
Cohesion: 0.07
Nodes (38): check_confusions(), check_pronoun(), check_redundant_mix(), COMPARATIVES, COPULAS, in_quotes_span(), push_replace(), REDUNDANT_MIX_PAIRS (+30 more)

### Community 389 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 390 - "P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)"
Cohesion: 0.15
Nodes (12): 10. Vòng 46 (tiếp) — Pass E2 đối kháng + quét O(n²) toàn luật (2026-10-31), 11. Vòng 49-50 — MobaXterm 3-pass audit + panel che vùng gõ (2026-11-03), 1. Bản đồ đường tiếng Anh (đã trace từng nhánh), 2. Root cause đã fix (vòng 42), 3. Đề xuất (cần quyết định PO — không tự ý đổi), 4. Kiểm chứng, 5. Pass EN-1 (agent audit độc lập) — finding + fix (vòng 42), 6. Pass EN-2 (đối kháng 2 commit vòng 42) — finding + fix (vòng 42, 2026-10-31) (+4 more)

### Community 395 - "e50_unikey_restore.rs"
Cohesion: 0.17
Nodes (21): assert_matrix(), e50_after_restore_skip_further_restore(), e50_daat_space_no_duplicate(), e50_dat_space_no_duplicate(), e50_unikey_task_space_no_restore(), e60_ambiguous_vn_preserved_and_space_complete_guard(), e60_contrast_double_letters_and_triple_undo(), e60_freq3000_unikey_undo_sweep() (+13 more)

### Community 396 - "ViState"
Cohesion: 0.06
Nodes (24): IpcClient, TimeInterval, Bool, ViState, ViStateStore, AssistIpcTests, EnableGatingTests, 0. Thiết kế tóm tắt (Design Intent) (+16 more)

### Community 397 - "Rà soát backlog theo tài liệu kế hoạch — 2026-11-08"
Cohesion: 0.29
Nodes (6): 1. Đã xử lý trong lượt rà soát này, 2. Cần chủ repo / bên ngoài (agent không tự làm được), 3. Cần người / thiết bị thật, 4. Rủi ro/hoãn đã ghi (không đổi, chờ điều kiện), 5. Tính năng mới đáng làm tiếp (agent làm được) — theo thứ tự đề xuất, Rà soát backlog theo tài liệu kế hoạch — 2026-11-08

### Community 398 - "eval.rs"
Cohesion: 0.14
Nodes (12): apply_func(), apply_op(), apply_unary(), eval(), eval_delimited(), MAX_DEPTH, MAX_TOKENS, precedence() (+4 more)

### Community 399 - "Option"
Cohesion: 0.24
Nodes (11): Entry, FieldRoles, id_matches(), MatchClause, Matcher, Preset, Option, Vec (+3 more)

### Community 400 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.04
Nodes (46): 0. Executive Summary, 1.1 Bảng tổng hợp, 1.2 Kế thừa & from-scratch — quyết định rõ ràng, 1.3 Ma trận bug kinh niên → kế hoạch xử lý (Đây là "bài toán thật" của dự án), 1. Phân tích 7 nguồn tham chiếu (Audit of sources), 2.1 Tầm nhìn & USP, 2.2 Personas & Jobs-to-be-done, 2.3 Backlog ưu tiên (MoSCoW) (+38 more)

### Community 401 - "TapPermission"
Cohesion: 0.22
Nodes (7): NSLock, Bool, TimeInterval, Timer, Void, TapPermission, ApplicationServices

### Community 402 - "icons.rs"
Cohesion: 0.29
Nodes (6): HICON, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V, load_app_icon(), HINSTANCE

### Community 403 - "display"
Cohesion: 0.25
Nodes (8): idle_ms, show_on_comma, show_on_sentence_end, additionalProperties, $comment, default, type, display

### Community 404 - "classifier.rs"
Cohesion: 0.15
Nodes (10): accepts_latex_frac(), accepts_mathml(), accepts_unicode_math(), classify(), count_occurrences(), is_currency_span(), is_hard_reject(), is_unicode_math() (+2 more)

### Community 405 - "P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả"
Cohesion: 0.25
Nodes (7): 0. Kết quả, 1. Thiết kế (vòng 1 — SA), 2. Vòng 2 — Audit (license / privacy / AV), 3. Vòng 3 — PO/QA (chất lượng & rủi ro), 4. Bằng chứng (để agent sau không phải đo lại), 5. Còn lại cho P4-2, P4-5 — Review P4-2a: refactor buffer `core` (đệ quy 2 buffer) + kết quả

### Community 406 - "show_macro_editor"
Cohesion: 0.17
Nodes (24): HGDIOBJ, add_button_tooltips(), center_on_work_area(), create_and_show_window(), create_control(), create_control_ex(), create_dialog_controls(), delete_ui_fonts() (+16 more)

### Community 407 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 408 - "mask.rs"
Cohesion: 0.36
Nodes (8): che_email(), che_url(), expand_non_space(), expand_word(), is_word_char(), masked_ranges(), masked_ranges_chars(), Vec

### Community 409 - "simple_telex.rs"
Cohesion: 0.38
Nodes (4): fold(), n(), DiacriticStyle, Vec

### Community 410 - "textvn_config"
Cohesion: 0.19
Nodes (14): decay_count_lon_giam_nua(), ho_so_nho_khong_prune(), merge_delta_va_prune(), prune_mot_lan_khi_ho_so_truong_thanh(), StyleWord, Vec, words(), bam_cong_tac_ghi_decided_va_ton_trong_lua_chon() (+6 more)

### Community 411 - "CachedFormulaStore"
Cohesion: 0.19
Nodes (11): CachedFormula, CachedFormulaStore, clear_works(), ClipboardWatcher, latest_wins(), MAX_PAYLOAD, Default, Mutex (+3 more)

### Community 413 - "os_spell.rs"
Cohesion: 0.08
Nodes (39): BREAKER_COOLDOWN_MS, breaker_trip_va_phuc_hoi(), CALL_TIMEOUT_MS, ComApartment, create_checkers(), EN_LANG_TAGS, first_error_start(), handle_job() (+31 more)

### Community 414 - "quick_telex.rs"
Cohesion: 0.25
Nodes (7): apply, apply(), apply_with_undo(), expansion(), q(), Option, Vec

### Community 415 - "StageInfo"
Cohesion: 0.29
Nodes (6): stage_json_roundtrip_and_validation(), StageInfo, launch_package_app(), Result, spawn_detached(), write_stage()

### Community 416 - "style_learning_100.rs"
Cohesion: 0.28
Nodes (20): apply(), drain_count(), learning_engine(), letter(), press(), Action, KeyEvent, Vec (+12 more)

### Community 417 - "p11_math_toggle.rs"
Cohesion: 0.35
Nodes (11): math_default_off_normal_typing_unchanged(), math_enabled_passthrough_counts_keys_typed_once(), math_span_suspends_telex_and_counts_once(), math_sticky_boundary_expands_alias(), math_sticky_command_suspends_telex(), math_sticky_inline_eval(), math_toggle_off_resets_armed_chord(), opts_with_math() (+3 more)

### Community 418 - "T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006"
Cohesion: 0.29
Nodes (7): LNX-020 · Addon skeleton + lifecycle (M) — `P3-2 §3`, LNX-021 · keyEvent → PASS (M) — `P3-2 §4`, LNX-022 · Preedit + commit + word boundary (L) — `P3-2 §5`, LNX-023 · SelectionReplace + BackspaceType/surrounding (M) — `P3-2 §5`, LNX-024 · focus/commit-before-hide (M) — `P3-2 §3`, LNX-025 · Framework switch + chống đôi (M) — `P3-2 §7`, T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006

### Community 419 - "per_app"
Cohesion: 0.33
Nodes (6): type, additionalProperties, $comment, default, type, per_app

### Community 420 - "Security Policy"
Cohesion: 0.50
Nodes (4): Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 421 - "abnormal.rs"
Cohesion: 0.02
Nodes (20): check_sentence(), bat_cum_sai_va_giu_hoa_thuong(), crlf_va_tab_khong_panic(), EmptyDict, ky_tu_nul_va_ky_tu_dieu_khien(), ky_tu_zero_width_va_combining(), Dictionary, Vec (+12 more)

### Community 422 - "press"
Cohesion: 0.22
Nodes (15): abnormal_fuzz_tong_hop_bat_bien(), abnormal_peek_khong_doi_ket_qua_phim_that(), apply(), char_key(), ctrl_space(), press(), r2_141_co_het_han_sau_mot_phim(), r2_144_ctrl_space_khi_du_doan_tat_van_hoan_tat_theo_yeu_cau() (+7 more)

### Community 423 - "trigger_keys"
Cohesion: 0.33
Nodes (6): enum, trigger_keys, $comment, default, items, type

### Community 424 - "core/src/lib.rs"
Cohesion: 0.06
Nodes (85): Action, ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, apply(), auto_capitalize_after_dot_and_enter(), auto_capitalize_needs_space_after_dot() (+77 more)

### Community 425 - "Option"
Cohesion: 0.14
Nodes (17): advice_for_failure(), append_register_log_tail(), DIAG_EXPORT_RESULT, DIALOG_CTX, read_log_tail(), register_and_activate_tsf(), register_log_path(), register_log_tail_reads_last_lines() (+9 more)

### Community 426 - "properties"
Cohesion: 0.14
Nodes (14): default, maxItems, type, $comment, default, type, default, maximum (+6 more)

### Community 428 - "Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit"
Cohesion: 0.33
Nodes (6): Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit, Bổ sung sau commit `ec1ac2b` — triệt để hóa gợi ý lỗi đăng ký TSF (ảnh lỗi của người dùng), Kiểm chứng chạy trên host Windows (trước commit), Perf — job `perf regression` đỏ trên CI là **nhiễu runner**, không phải hồi quy, Phát hành — v0.2.2 đã publish (release candidate, chưa ký số), Trạng thái

### Community 429 - "`docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)"
Cohesion: 0.25
Nodes (7): 0. Cách chạy 1 app (không bỏ qua bước này), 1. Windows (20 app — `P1-5 §3`), 2. macOS (20 app — `P2-5 §3`), 3. Linux (20 app — `P3-6 §3`), 4. Ghi chú bắt buộc khi điền bảng, 5. Tóm tắt release candidate, `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)

### Community 430 - "P8/V55 — DFMEA + DRBFM: 5 fix phản hồi test thật của chủ repo (2026-11-05)"
Cohesion: 0.25
Nodes (7): F1 — "uong" tự thành "ương" (phá chuẩn UniKey 20 năm), F2 — xung đột OS-IME trên TSF-bypass apps (MobaXterm), F3 — username `truong.giap` bị bind, F4 — dead-key Windows (`Ctrl+'`, `Ctrl+` + letter…), F5 — panel popup hiện liên tục che vùng gõ, P8/V55 — DFMEA + DRBFM: 5 fix phản hồi test thật của chủ repo (2026-11-05), Tổng hợp gate tests mới (vòng 55)

### Community 431 - ".new"
Cohesion: 0.23
Nodes (11): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), decide_full_truth_table(), empty_result(), engine_never_transforms_when_policy_says_pass(), hook_db(), injected_chord_secure_and_key_up_always_pass(), ready() (+3 more)

### Community 432 - "export_diagnostics_zip"
Cohesion: 0.20
Nodes (14): build_pkzip(), collect_linux_install_diag_tail(), collect_log_tail(), collect_windows_register_log_tail(), crc32(), export_diagnostics_zip(), pkzip_builder_creates_valid_structure(), redact_paths_replaces_username() (+6 more)

### Community 433 - "rca_v57_adversarial.rs"
Cohesion: 0.35
Nodes (14): adv1_peek_khong_hoc(), adv2_tab_accept_count_1(), adv3_drain_shape_json_chuan(), adv4_cycle_hoc_dung_candidate(), adv5_drain_2_lan_chi_pending_moi(), apply(), engine(), letter() (+6 more)

### Community 434 - "3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 435 - "learning_curve.rs"
Cohesion: 0.42
Nodes (13): delta(), engine(), khong_hoc_rac_ban_phim_nhung_hoc_tu_that_va_viet_tat(), peek_khong_tieu_thu_delta(), press(), KeyEvent, Vec, rsi_delta_cac_luot_drain_cong_lai_bang_snapshot() (+5 more)

### Community 436 - "gen_vocab.rs"
Cohesion: 0.11
Nodes (19): collections, complete_vn_word(), Option, v(), VN_SYLLABLES_DATA, vn_syllables_sorted(), diacriticstyle, is_valid_word (+11 more)

### Community 437 - "say"
Cohesion: 0.23
Nodes (14): activate_for_session(), activate_tip(), active_profile_is_textvn(), com_init(), delete_tree(), do_unregister(), is_service_profile_sid(), say() (+6 more)

### Community 438 - "caps.rs"
Cohesion: 0.31
Nodes (4): is_closer(), is_sentence_end(), sentence_start(), starts()

### Community 440 - "ChordFsm"
Cohesion: 0.25
Nodes (13): ChordFsm, ChordState, frac_cursor_latex(), fsm(), hud_due_after_pause(), latex_mode_no_chain(), Default, Self (+5 more)

### Community 441 - "emoji_picker.c"
Cohesion: 0.36
Nodes (13): append_utf8_char(), ime_instance, lc_emoji_picker, lc_emoji_close(), lc_emoji_format_aux(), lc_emoji_handle_key(), lc_emoji_init(), lc_emoji_open() (+5 more)

### Community 443 - "3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux"
Cohesion: 0.33
Nodes (6): 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn

### Community 444 - "ctrl_shift_marker_path"
Cohesion: 0.40
Nodes (6): compatibility_hook_path(), ctrl_shift_marker_path(), legacy_freed_layout_hotkey(), restore_ctrl_shift_if_we_freed(), Option, PathBuf

### Community 445 - "style_learning_v59.rs"
Cohesion: 0.35
Nodes (13): learning_opts(), letter(), recency_khong_boost_danh_sach_count_desc_sau_load(), recency_khong_xoa_lich_su_count_cao(), recency_touch_thang_hon_count_lau_quan(), KeyEvent, rsi_next_phrase_office_en_dung_cong_thuc(), space() (+5 more)

### Community 446 - "test-linux-distro-matrix.sh"
Cohesion: 0.56
Nodes (8): cleanup(), distro_fcitx5_e2e(), distro_ibus_install(), distro_ibus_portable(), distro_system_install(), fail(), new_test_home(), test-linux-distro-matrix.sh script

### Community 447 - "StylePending"
Cohesion: 0.33
Nodes (5): BTreeMap, Formatter, Result, StylePending, Debug

### Community 448 - "core"
Cohesion: 0.18
Nodes (22): core, ITfInputProcessorProfiles, activate(), clsid_key(), com_init(), dll_path(), guid_str(), ILOT_DEFPROFILE (+14 more)

### Community 449 - "default"
Cohesion: 0.13
Nodes (15): altgr_enabled, chord_timeout_ms, clipboard_smart_paste, custom_keybindings, hotkeys, hud_pause_ms, leader_key, output_mode (+7 more)

### Community 450 - "os_linux.rs"
Cohesion: 0.16
Nodes (13): BACKEND_ORDERING, MAX_SUGGEST_RESULTS, PROBE_TIMEOUT, Duration, Mutex, Vec, SUGGEST_BUDGET, SUGGEST_CALLS_PER_CHECK (+5 more)

### Community 451 - "press"
Cohesion: 0.29
Nodes (10): backspace_in_composition_matrix(), ctrl_shift_tap_toggles_once_and_only_without_other_keys(), Doc, focus_switch_mid_word_commits_and_never_deletes_old_text(), focus_switch_rapid_three_window_matrix(), mode_en_english_completion_works_within_composition(), navigation_key_commits_as_is_and_starts_new_word(), press() (+2 more)

### Community 452 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.29
Nodes (7): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust, Phát hiện W-06: Điểm mù kiến trúc 32-bit WOW64 — Zalo PC và toàn bộ ứng dụng x86 không thể nạp TSF DLL 64-bit

### Community 454 - "English"
Cohesion: 0.10
Nodes (21): 0. Universal Scientific, Engineering & Math Engine (new in 0.2.83 — `textvn-math` crate), 1. Dream RSI Ergonomic Metrics & Performance Benchmarks (P10 / G19), 1. Pure In-Process Architecture & Uncompromising Memory Safety, 2. Contextual Bilingual Prediction & Dream RSI Ergonomics (P10), 2. Contextual Prediction Model P6 & User Style Profile P7 (53,955-Entry Corpora), 3. On-Device Grammar & Context Assistant (Grammarly-Grade Offline), 3. Typing Engines & Phonotactic Transformation Standard, 4. 20-Year UniKey Heritage Standard & Modern Office Utilities (+13 more)

### Community 457 - "test-linux-el8-package.sh"
Cohesion: 0.56
Nodes (8): cleanup(), el8_install(), el8_portable(), el8_system_install(), el8_wayland_live_portable(), fail(), new_el8_home(), test-linux-el8-package.sh script

### Community 458 - "[0.2.32] — 2026-10-12"
Cohesion: 0.40
Nodes (5): [0.2.32] — 2026-10-12, Changed, Fixed, Known issues, Security / quy trình

### Community 459 - "Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên"
Cohesion: 0.40
Nodes (5): 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`), 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`), 3. Bảng xung đột có thể xảy ra và cách giải, 4. Nguyên tắc điều chỉnh thiết kế (ghi để không lặp tranh luận), Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên

### Community 460 - "entries"
Cohesion: 0.50
Nodes (4): items, type, $ref, entries

### Community 461 - "conflict_notice.rs"
Cohesion: 0.23
Nodes (12): balloon_text(), ConflictSummary, e70_balloon_text_fits_shell_limit(), e70_notify_once_per_conflict_set(), e70_summarize_keeps_only_conflicts_and_skips_ctrl_shift(), HANDLED_ELSEWHERE, JSON, MARKER_FILE (+4 more)

### Community 462 - "linux-fcitx5/src/engine.h"
Cohesion: 0.02
Nodes (101): lc_selection_info, byte_len, char_len, del_offset, start_byte, TextVNEngineFactory, create, ime_assist_plan (+93 more)

### Community 463 - "[0.2.14] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.14] — 2026-10-03, Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau"), Fixed (Windows), Tests

### Community 464 - "MathEngine"
Cohesion: 0.14
Nodes (8): AppGuard, is_math_boundary(), is_non_text_latex_cmd(), is_word_boundary(), MathEngine, MAX_WORD_BUF, Option, Self

### Community 465 - "do_register"
Cohesion: 0.17
Nodes (21): ctf_category_keys(), ctf_layout_matches_tsf_registry_schema(), ctf_profile_key(), ctf_tip_key(), do_register(), do_status(), layout_install_accepted(), machine_profile_metadata_ok() (+13 more)

### Community 466 - "worker_main"
Cohesion: 0.27
Nodes (10): en_us_neu_may_co_thi_provider_hoat_dong(), Job, Option, Receiver, SyncSender, spawn_worker(), store_support(), support_cached() (+2 more)

### Community 467 - "engine_boundary_fca.rs"
Cohesion: 0.35
Nodes (10): boundary_expands_dynamic_template_mat2x2(), boundary_expands_vn_alias_can(), engine_sticky(), eval_div_zero_stays_literal(), inline_eval_compact_syntax(), inline_eval_spaced_syntax(), reset_all_clears_word_buf(), Option (+2 more)

### Community 468 - "ModifierToggle"
Cohesion: 0.29
Nodes (6): is_ctrl_vk(), is_shift_vk(), modifier_toggle_rapid_sequence_matrix(), ModifierToggle, Instant, standalone_ctrl_tap_locks_valid_word_and_restores_invalid_syllable()

### Community 469 - "save"
Cohesion: 0.36
Nodes (9): gen_app_icons_cli(), gen_store_art_cli(), gen_tray_icons_cli(), out_dir(), repo_root(), Path, PathBuf, Result (+1 more)

### Community 470 - "linux-settings/src/main.c"
Cohesion: 0.33
Nodes (4): gpointer, GtkApplication, on_app_activate(), gtk

### Community 471 - "Targets JSON — `tools/appcomptest/targets/` (WIN-061)"
Cohesion: 0.25
Nodes (7): 1. File & schema, 2. Cách chạy, 3. Acceptance (WIN-061), 4. Findings, 5. Limitation / follow-up, Locator keys (AND trong 1 locator, OR = thứ tự mảng), Targets JSON — `tools/appcomptest/targets/` (WIN-061)

### Community 472 - "display"
Cohesion: 0.40
Nodes (5): display, hide_on_auto_fix, idle_ms, show_on_comma, show_on_sentence_end

### Community 473 - "[0.2.28] — 2026-10-10"
Cohesion: 0.40
Nodes (5): [0.2.28] — 2026-10-10, Added, Fixed — kiểm trên Windows 11 thật và rà soát đối kháng (R2-88…R2-98, chi tiết: `docs/release/code-review-2026-10-09-round2.md` §3.7), Fixed — rà soát mã nguồn chuyên sâu 2026-10-07 (chi tiết: `docs/release/code-review-2026-10-07.md`), Fixed — rà soát vòng 2: Microsoft Store, ký số, QA (chi tiết: `docs/release/code-review-2026-10-09-round2.md`)

### Community 474 - "Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening"
Cohesion: 0.50
Nodes (4): Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening, Nguyên nhân + fix + bằng chứng, Phát hành — v0.2.10 & v0.2.11 đã publish, Sự cố phát hành mới (R4) — publish fail im lặng

### Community 475 - "HookEngine"
Cohesion: 0.29
Nodes (5): HookEngine, Drop, ime_instance, Result, Self

### Community 476 - "IpcCodecTests"
Cohesion: 0.19
Nodes (7): IpcCodecTests, Data, [0.2.84] — 2026-11-22, Fixed, Kiểm thử / Testing, Sửa lỗi / Fixed, Testing

### Community 478 - "Vec"
Cohesion: 0.22
Nodes (4): PredictCycle, retype_chunk(), EmojiEntry, Vec

### Community 479 - "overlay.rs"
Cohesion: 0.06
Nodes (42): COLOR_BG, COLOR_BORDER, COLOR_ERROR, COLOR_MUTED, COLOR_SELECTED_BG, COLOR_TEXT, dispatch_mouse_action(), HEADER_H (+34 more)

### Community 480 - "2. Windows — Spell Checking API (`spellcheck.h`, Win8+)"
Cohesion: 0.12
Nodes (15): 2.1 Kiểm tra **thật** crate `windows` 0.61.3 (đã grep registry cargo), 2.2 Chuỗi gọi API (đúng thứ tự, đúng kiểu), 2.3 COM activation & apartment (rủi ro thật), 2.4 Đơn vị offset — **đo thực nghiệm**, tài liệu MS **không** nói, 2.5 Số đo thực nghiệm (để đặt ngân sách, không phải benchmark chính thức), 2.6 Rủi ro riêng của Windows, 2.7 Code sketch (Rust — **sketch trong doc**, không phải code sản phẩm), 2. Windows — Spell Checking API (`spellcheck.h`, Win8+) (+7 more)

### Community 481 - ".CreateInstance"
Cohesion: 0.22
Nodes (8): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Ref, Result

### Community 482 - "T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007"
Cohesion: 0.33
Nodes (6): LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`, LNX-041 · Callback + self-disable (M) — `P3-3 §3`, LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`, LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`, LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S), T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### Community 483 - "Hướng dẫn phát triển TextVN"
Cohesion: 0.20
Nodes (10): 1. Kiến trúc, 2. Dựng, 3. Kiểm thử, 4. Quy ước bắt buộc (CI chặn), 5. Phát hành, 6. Tài liệu liên quan, Hướng dẫn phát triển TextVN, Linux (8 dòng distro doanh nghiệp & phổ thông: EL8 · EL9 · SLES 15 · Ubuntu 22.04/24.04 · Debian 12 · Fedora 41 · Arch) (+2 more)

### Community 485 - "claim_debounced"
Cohesion: 0.40
Nodes (5): claim_debounced(), LAST_GLOBAL_TOGGLE_MS, AtomicU64, system_millis(), try_claim_global_toggle()

### Community 486 - "idle_ms"
Cohesion: 0.40
Nodes (5): $comment, default, minimum, type, idle_ms

### Community 487 - "items"
Cohesion: 0.25
Nodes (8): $comment, default, items, type, items, maxItems, minItems, custom_keybindings

### Community 488 - "[0.2.16] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.16] — 2026-10-03, Fixed (Windows) — portable "giải nén chạy ngay" hoạt động trên Win11 24H2+, Verified live trên máy thật (máy chủ repo, build 26300 — trường hợp từng fail)

### Community 489 - "max_sentence_chars"
Cohesion: 0.40
Nodes (5): $comment, default, minimum, type, max_sentence_chars

### Community 490 - "HookState"
Cohesion: 0.26
Nodes (7): HookMode, HookState, .CALLBACK_BUDGET, .MAX_SLOW_STREAK, .ORPHAN_TIMEOUT, Duration, slow_callback_self_disables_and_heartbeat_expires()

### Community 491 - "[0.2.13] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.13] — 2026-10-03, Added, Documented, Fixed

### Community 492 - "diacritic_style.rs"
Cohesion: 0.24
Nodes (9): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), vowel_span(), is_vowel (+1 more)

### Community 493 - "[0.2.33] — 2026-10-18"
Cohesion: 0.50
Nodes (4): [0.2.33] — 2026-10-18, Added, Changed, Fixed

### Community 494 - "P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15"
Cohesion: 0.25
Nodes (7): 0. Kết quả đối chiếu với handoff, 1. Vòng 16 — SA/QA, 2. Vòng 17 — Audit (license/privacy/AV), 3. Vòng 18 — PO/UX, 4.1 Bổ sung (2026-10-21, sau khi CI báo đỏ), 4. Kết luận, P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15

### Community 495 - "IMPLEMENTATION STATUS — adapter macOS (Farch-4)"
Cohesion: 0.33
Nodes (5): Còn mở (2026-10-09), IMPLEMENTATION STATUS — adapter macOS (Farch-4), Tiếp theo (tuần tự), Trạng thái (2026-10-09), Đã triển khai (bảng thành phần)

### Community 496 - "[0.2.81] — 2026-11-18"
Cohesion: 0.50
Nodes (4): [0.2.81] — 2026-11-18, Sửa lỗi / Fixed, Thay đổi / Changed, Thêm mới / Added

### Community 497 - "TextVN — Chỉ mục tài liệu & Giao thức Review"
Cohesion: 0.29
Nodes (7): 0. Tài liệu cho người dùng & phát hành, 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 498 - "[0.2.38] — 2026-10-24"
Cohesion: 0.50
Nodes (4): [0.2.38] — 2026-10-24, Added, Changed, Fixed

### Community 499 - "Vec"
Cohesion: 0.27
Nodes (11): collect_linux_diagnostics(), collect_macos_diagnostics(), collect_windows_diagnostics(), detect_apple_vi_input_sources(), detect_linux_im_env_mismatch(), detect_other_vi_tips(), DiagnosticItem, e70_detect_other_vi_tips_from_user_profile_values() (+3 more)

### Community 500 - "resolve.rs"
Cohesion: 0.33
Nodes (10): default_for_field, downgrade_without_cap(), inp(), secure_field_role_passthrough(), secure_role_beats_hint_and_presets_without_secure_flag(), step1_secure_beats_everything(), step2_disabled_passthrough(), step3_user_preset_beats_system() (+2 more)

### Community 501 - "e50_audit_edge.rs"
Cohesion: 0.36
Nodes (9): e50_multiple_en_words_in_sequence(), e50_no_restore_when_not_needed(), e50_restore_then_another_en_word(), e50_restore_then_immediate_typing(), e50_unikey_dd_after_restore(), e50_vietnamese_after_restore(), Vec, type_and_get() (+1 more)

### Community 502 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.33
Nodes (6): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust

### Community 503 - "mode"
Cohesion: 0.50
Nodes (4): $comment, default, enum, mode

### Community 504 - "Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent"
Cohesion: 0.50
Nodes (4): Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent, Giải pháp + bằng chứng kiểm thử, Phát hành — v0.2.8 đã publish (release candidate, chưa ký số), Sự cố trong phiên — tra trước khi lặp lại

### Community 505 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 506 - "3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)"
Cohesion: 0.33
Nodes (6): 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu, 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions), 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions), 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết, Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE), Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)

### Community 507 - "Broker"
Cohesion: 0.25
Nodes (7): Broker, EnchantBroker, raw_broker_free(), raw_broker_init(), raw_set_ordering(), resolve_tag(), Drop

### Community 508 - "corpus.rs"
Cohesion: 0.33
Nodes (7): corpus_co_du_lop_hieu_biet(), corpus_files(), corpus_vi_basic_passes(), parse_corpus(), Path, PathBuf, Vec

### Community 509 - "Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành"
Cohesion: 0.50
Nodes (4): Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành, Gate trước tag 0.2.4, Tiếp tục audit ZCode — engine (2026-10-01), Tiếp tục audit ZCode — runtime chưa có bằng chứng production

### Community 510 - "[0.2.65] — 2026-11-06"
Cohesion: 0.67
Nodes (3): [0.2.65] — 2026-11-06, Added, Fixed

### Community 511 - "[0.2.8] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.8] — 2026-10-02, Added, Changed, Fixed

### Community 514 - "[0.2.85] — 2026-11-22"
Cohesion: 0.40
Nodes (5): [0.2.85] — 2026-11-22, Fixed, Kiểm thử / Testing, Sửa lỗi / Fixed, Testing

### Community 516 - "[0.2.42] — 2026-10-24"
Cohesion: 0.67
Nodes (3): [0.2.42] — 2026-10-24, Added, Fixed

### Community 517 - "hud_pause_ms"
Cohesion: 0.33
Nodes (6): $comment, default, maximum, minimum, type, hud_pause_ms

### Community 518 - "windowsandmessaging"
Cohesion: 0.18
Nodes (12): gdi, libraryloader, setfocus, EDIT, main(), HWND, LPARAM, LRESULT (+4 more)

### Community 519 - "leader_key"
Cohesion: 0.33
Nodes (6): $comment, default, maxLength, minLength, type, leader_key

### Community 520 - "collect_report"
Cohesion: 0.20
Nodes (10): abi_sizes(), check_instance_mutex(), check_pipe_listening(), collect_report(), config_path(), count_panic_records(), default_export_path(), e70_export_contains_panic_tail_and_build_identity() (+2 more)

### Community 521 - "palette_max_results"
Cohesion: 0.33
Nodes (6): $comment, default, maximum, minimum, type, palette_max_results

### Community 522 - "[0.2.46] — 2026-10-26"
Cohesion: 0.67
Nodes (3): [0.2.46] — 2026-10-26, Added, Fixed

### Community 523 - "P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)"
Cohesion: 0.22
Nodes (8): 1. Nguồn `FieldContext` trên Windows, 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`), 3. App preset mặc định — `data/appdb.default.json` (phần Windows), 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4), 5. Tích hợp với adapter — API nội bộ chung, 6. Test & bằng chứng, 7. Mapping task, P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

### Community 525 - "[0.2.54] — 2026-10-30"
Cohesion: 0.67
Nodes (3): [0.2.54] — 2026-10-30, Added, Fixed

### Community 526 - "KeyKind"
Cohesion: 0.33
Nodes (3): e62_math_consumed_pass_altgr_and_enter_replace(), KeyKind, idle_arms()

### Community 527 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.22
Nodes (8): 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux), 7. Env matrix & detect framework (dùng cho `textvn doctor` — `PLAN §5.3`), 8. Test, P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết

### Community 528 - "Khắc phục sự cố TextVN (Troubleshooting)"
Cohesion: 0.22
Nodes (9): 1. Bắt đầu: xuất gói chẩn đoán (30 giây), 2. Xung đột bộ gõ (gõ lặp chữ, mất dấu, phím V/E không phản hồi), 3. Bảng mã lỗi đầy đủ, 4. Linux: phụ thuộc theo distro, 5. Windows: doanh nghiệp, LTSC, portable, quyền quản trị, 6. Báo lỗi, Gói ZIP chứa gì, Khắc phục sự cố TextVN (Troubleshooting) (+1 more)

### Community 529 - "English"
Cohesion: 0.22
Nodes (9): 1. Start here: export a diagnostic bundle (30 seconds), 2. IME conflicts (doubled characters, lost diacritics, V/E toggle not responding), 3. Complete error-code table, 4. Linux: dependencies per distribution, 5. Windows: enterprise, LTSC, portable, administrator rights, 6. Reporting a bug, English, Log locations on disk (+1 more)

### Community 530 - "diag_summary.rs"
Cohesion: 0.36
Nodes (8): build_issue_summary(), current_user_tokens(), e70_redact_handles_unicode_usernames(), e70_summary_keeps_only_zip_name_and_redacts_user(), e70_summary_without_findings(), redact_token(), redact_user_tokens(), Vec

### Community 531 - "Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục"
Cohesion: 0.67
Nodes (3): Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục, Kiểm chứng E2E trên máy thật (2026-10-02, sau khi publish), Phát hành — v0.2.7 đã publish (release candidate, chưa ký số)

### Community 536 - "[0.2.0] — 2026-09-30"
Cohesion: 0.25
Nodes (8): [0.2.0] — 2026-09-30, Added, Added (macOS — Farch-4), Added (trước đó), Changed, Fixed, Fixed (review vòng 3 — trước tag v0.2.0), Known limitations

### Community 537 - "Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất"
Cohesion: 0.50
Nodes (4): 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển, 0.2.14 — DPI gốc + UI thống nhất + common errors, Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất, Phát hành + xác minh máy chủ repo

### Community 538 - "2. Kết quả kiểm thử"
Cohesion: 0.50
Nodes (4): 2. Kết quả kiểm thử, Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng), Kịch bản 1 — Cài đặt, Kịch bản 2 — Giải nén ra dùng luôn

### Community 539 - "P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết"
Cohesion: 0.25
Nodes (7): 2. Bảng AX → `IME_FIELD_*` (module `FieldDetect.swift`, mirror `P1-3 §2`), 3. App preset mặc định — `data/appdb.default.json` (phần macOS), 4. Override & state (khớm `P0-3 §3.1` — cùng bảng với `P1-3 §4`, khác path macOS), 5. API nội bộ (dùng chung IMK + tap), 6. Test, 7. Task, P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết

### Community 540 - "[0.2.78] — 2026-11-14"
Cohesion: 0.67
Nodes (3): [0.2.78] — 2026-11-14, Added, Fixed

### Community 541 - "[0.2.64] — 2026-11-04"
Cohesion: 0.50
Nodes (4): [0.2.64] — 2026-11-04, Added, Changed, Fixed

### Community 543 - "[0.2.72] — 2026-11-08"
Cohesion: 0.50
Nodes (4): [0.2.72] — 2026-11-08, Changed, Fixed, Tests

### Community 544 - "[0.2.74] — 2026-11-08"
Cohesion: 0.50
Nodes (4): [0.2.74] — 2026-11-08, Added, Changed, Fixed

### Community 545 - "[0.2.9] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.9] — 2026-10-02, Added, Documented, Tests

### Community 556 - "textvn_core"
Cohesion: 0.13
Nodes (15): type_stream(), convert(), cp1252(), cp1258(), engine(), mojibake(), r2_139_bang_ma_cu_ve_bang_ma_dang_dung(), r2_139_go_lai_chu_go_nham_che_do_e() (+7 more)

### Community 558 - "B. macOS"
Cohesion: 0.05
Nodes (37): MacroRules, Problem, emptyExpansion, emptyTrigger, expansionTooLong, .message, triggerHasSpace, triggerTooLong (+29 more)

### Community 559 - "ime_assist_plan"
Cohesion: 0.25
Nodes (8): ime_assist_plan, abi_version, delete_chars, delete_from, delete_glyphs, fixed_count, original_tail, replacement

### Community 567 - ".process"
Cohesion: 0.48
Nodes (5): EngineOutcome, is_hook_owned(), is_plain_letter(), KeyEvent, Option

### Community 569 - "[0.2.69] — 2026-11-08"
Cohesion: 0.40
Nodes (5): [0.2.69] — 2026-11-08, Added, Changed, Fixed, Performance

### Community 570 - "AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN"
Cohesion: 0.33
Nodes (5): 0. Đọc trước khi làm bất cứ việc gì (trình tự), 1. Luật không được phá (vi phạm = task chưa xong), 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự, 3. Kiến trúc & ràng buộc kỹ thuật cốt lõi, AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN

### Community 571 - "cli/build.rs"
Cohesion: 0.47
Nodes (5): emit_build_identity(), find_rc_exe(), main(), Option, PathBuf

### Community 572 - "e62_math_all_editors.rs"
Cohesion: 0.67
Nodes (5): e62_en_mode_winword_autobyapp_esc_and_altgr(), e62_immediate_chords_aliases_and_eval_without_sticky(), e62_math_spans_and_text_exit_preserve_vietnamese(), opts_math(), type_stream()

### Community 573 - "en_restore.rs"
Cohesion: 0.53
Nodes (4): ambiguous_pairs_keep_vietnamese(), en_words_auto_restore_on_space(), top_3000_english_words_type_correctly(), type_word()

### Community 574 - "4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)"
Cohesion: 0.33
Nodes (6): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), Phát hiện M-01: Không đồng bộ giao diện Menu Bar khi nhận IPC, Phát hiện M-02: Quyền Hardened Runtime (Entitlements) thừa thãi, Phát hiện M-03: Thử nghiệm kiểm thử CI thiếu kiến trúc x86_64, Phát hiện M-04: Thử nghiệm thực tế trên phần cứng macOS vật lý (Real Hardware & TCC Permissions Gap), Phát hiện M-05: Cấu hình `non_preedit` chưa liên kết vào IMK (Gõ chữ bị gạch chân)

### Community 576 - "[0.2.89] — 2026-11-25"
Cohesion: 0.40
Nodes (5): [0.2.89] — 2026-11-25, Added, Changed & Fixed, Kiểm thử / Testing, Testing, Tính năng mới & Sửa lỗi / Added, Changed & Fixed

### Community 577 - "[0.2.10] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.10] — 2026-10-03, Documented, Fixed, Tests

### Community 580 - "[0.2.75] — 2026-11-08"
Cohesion: 0.50
Nodes (4): [0.2.75] — 2026-11-08, Changed, Fixed, Tests

### Community 581 - "[0.2.79] — 2026-11-14"
Cohesion: 0.50
Nodes (4): [0.2.79] — 2026-11-14, Added, Changed, Fixed

### Community 583 - "[0.2.40] — 2026-10-24"
Cohesion: 0.67
Nodes (3): [0.2.40] — 2026-10-24, Added, Fixed

### Community 584 - "[0.2.34] — 2026-10-18"
Cohesion: 0.50
Nodes (4): [0.2.34] — 2026-10-18, Added, Docs, Fixed

### Community 586 - "[0.2.47] — 2026-10-26"
Cohesion: 0.67
Nodes (3): [0.2.47] — 2026-10-26, Added, Fixed

### Community 587 - "[0.2.49] — 2026-10-26"
Cohesion: 0.67
Nodes (3): [0.2.49] — 2026-10-26, Added, Fixed

### Community 588 - "[0.2.4] — 2026-10-01"
Cohesion: 0.67
Nodes (3): [0.2.4] — 2026-10-01, Fixed, Verification

### Community 589 - "[0.2.35] — 2026-10-23"
Cohesion: 0.67
Nodes (3): [0.2.35] — 2026-10-23, Added, Fixed

### Community 591 - "[0.2.50] — 2026-10-28"
Cohesion: 0.67
Nodes (3): [0.2.50] — 2026-10-28, Added, Fixed

### Community 592 - "[0.2.67] — 2026-11-07"
Cohesion: 0.67
Nodes (3): [0.2.67] — 2026-11-07, Added, Fixed

### Community 593 - "[0.2.83] — 2026-11-21"
Cohesion: 0.67
Nodes (3): [0.2.83] — 2026-11-21, Sửa lỗi / Fixed, Thêm mới / Added

### Community 595 - "[0.2.68] — 2026-11-07"
Cohesion: 0.67
Nodes (3): [0.2.68] — 2026-11-07, Added, Fixed

### Community 596 - "[0.2.70] — 2026-11-08"
Cohesion: 0.67
Nodes (3): [0.2.70] — 2026-11-08, Fixed, Tests

### Community 615 - "DIAG_EXPORT_IN_PROGRESS"
Cohesion: 0.67
Nodes (3): DIAG_EXPORT_IN_PROGRESS, AtomicBool, TSF_SETUP_IN_PROGRESS

## Knowledge Gaps
- **2718 isolated node(s):** `kind`, `ch`, `text`, `len`, `delete_before` (+2713 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 4392 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **102 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `1. Danh sách lỗi (E-IDs)` connect `engine.c` to `compose.c`, `config.rs`, `.register`, `engine.cpp`, `NSRange`, `ImeEngine`, `syllable_spelling.rs`, `AssistTests`, `dual_mode_matrix.rs`, `B. macOS`, `cli/src/assist.rs`, `.suggest`, `e2e_fcitx5.c`, `SettingsView`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `compose.c`, `TextVNAppTests`, `ViState`, `ConfigWatcher`, `NSRange`, `B. macOS`, `.suggest`, `AppDelegate`, `IpcMessage`, `StrategyResolver`, `FieldDetect`, `IpcServer`, `.register`, `ImeEngine`, `AXSnapshot`, `TextVNInputController`, `IpcServerAssistTests`, `IpcCodecTests`, `IpcClient`, `Foundation`, `FieldContext`, `EmojiPickerModel`, `AssistTests`, `2. Nhật ký finding`, `EventTapController`, `MarkedState`, `KeyTranslator`, `SettingsView`?**
  _High betweenness centrality (0.106) - this node is a cross-community bridge._
- **Why does `VÒNG REVIEW AS-050 / AS-060 — 3 lượt (SA/QA → Audit → PO/UX) — 2026-10-21 (`R-1…R-14`)` connect `string` to `compose.c`, `engine.cpp`, `ImeEngine`, `AssistTests`, `TextVNInputController`, `plan_sentence_auto_fix`, `IpcServer`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Are the 22 inferred relationships involving `TextVNInputController` (e.g. with `.emojiRecordUse()` and `.emojiSearch()`) actually correct?**
  _`TextVNInputController` has 22 INFERRED edges - model-reasoned connections that need verification._
- **What connects `kind`, `ch`, `text` to the rest of the system?**
  _2718 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `duplicate_word.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11076923076923077 - nodes in this community are weakly interconnected._
- **Should `compose.c` be split into smaller, more focused modules?**
  _Cohesion score 0.050203527815468114 - nodes in this community are weakly interconnected._