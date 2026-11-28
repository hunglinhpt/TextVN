# Graph Report - TextVN  (2026-11-28)

## Corpus Check
- 651 files · ~1,380,257 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 584 file(s) not represented in the graph (top: .jsonl 254, .keys 250, (none) 17)

## Summary
- 13855 nodes · 28359 edges · 711 communities (606 shown, 105 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 1846 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ac54124b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Ctx
- board/src/lib.rs
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
- dream_rsi_oracle_run
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
- ipc/src/sandbox.rs
- Chi tiết từng mục
- Result
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
- Issue
- appdb/src/lib.rs
- M4 — Tray / Settings / Packaging (tuần 11–14) — dep: WIN-016 (IPC contract đã chốt từ P0)
- A0 — Spike & môi trường (tuần 1–2)
- A3 — Menu bar / Settings / Packaging (tuần 11–14)
- T0 — Spike & môi trường (tuần 1–2)
- Id
- textvn-ffi
- type
- windows-tsf/src/compose.rs
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
- 2. Cách lấy oracle
- png.rs
- textvn-common.sh
- art_icons.rs
- config.v1.schema.json
- Verified Ops — sổ thao tác / tool đã kiểm chứng
- crate
- M2 — Field detect + AppDB (tuần 7–10) — dep: WIN-004, WIN-006
- M3 — Hook (tuần 7–11, song song M2) — dep: WIN-005
- A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006
- Engine
- config.rs
- T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005
- P3-REVIEW-LOG — Phần 3 (Linux)
- spikes/README.md
- SvcManager
- IpcClient
- env.md — Môi trường build Windows (WIN-001)
- A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009
- properties
- ByteCursor<'a>
- ImeEngine
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
- Equatable
- style_profile.rs
- P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN
- session_abi.rs
- ARM-W4 — Bộ cài Inno Setup + portable ZIP Windows ARM64 (ADR-015)
- arm64x-probe
- idle_popup_routing.rs
- test_delete_surrounding.cpp
- Attribution dữ liệu Tatoeba (CC BY 2.0 FR)
- vni.rs
- EmojiPicker
- method/mod.rs
- Antivirus false positive (RW3) — Kaspersky flag build production
- Message
- win_tools.rs
- bool
- store_win.rs
- Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)
- tray/src/assist.rs
- guids.rs
- uia_spike.ps1
- cli/src/assist.rs
- say
- EmojiPickerState
- autostart.rs
- ime_instance
- Changelog
- tray/src/main.rs
- menu.rs
- SettingsController
- time
- ipc_client.c
- P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế
- os_macos.rs
- assist_tools.rs
- mmap.rs
- README.md
- package_bootstrap.rs
- win32-uia.lib.ps1
- Dict
- engine.cpp
- syllable_spelling.rs
- ipc_client.rs
- bitset
- verify_targets.ps1
- verify.rs
- Performance Audit — TextVN Windows Platform
- gen_ui.rs
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
- sandbox_e2e.rs
- dictionary_budget.rs
- hygiene.rs
- IpcServer
- core/tests/invariants.rs
- P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết
- ime_key
- windows-tsf/src/assist.rs
- .register
- imk_abi.rs
- cli/src/main.rs
- phrase_repeat.rs
- Đóng góp cho TextVN
- tsf-min/src/lib.rs
- dict/src/lib.rs
- Result
- macro_text.rs
- dream_rsi_100.rs
- trigger
- P0-5 — Nền tảng thống nhất + chip ARM (Apple M · Windows on ARM · Linux aarch64)
- phrases.rs
- field-detect/src/lib.rs
- e2e_ibus.c
- e50_unikey_restore.rs
- ipc_server.rs
- e2e-assist-p6.ps1
- e2e_fcitx5.c
- emoji_popup.rs
- key_event.rs
- chords.rs
- client.rs
- push_key
- uninstall_linux.sh
- build-linux.sh
- fromrawhandle
- username_identity_1000.rs
- r2_171_caps_after_enter_reset.rs
- .new
- settings_window.c
- number_vn.rs
- foreground.rs
- settings_window_gtk3.c
- Foundation
- Problem
- Path
- EmojiPickerModel
- P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`
- Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor
- properties
- Hướng dẫn sử dụng TextVN
- keymap_mac.rs
- linux_tools.rs
- test_compose.c
- en_office.rs
- Báo cáo dựng & kiểm thử — TextVN
- Option
- regression_index.rs
- Dictionary
- ime_conflict_1000.rs
- .OnTestKeyDown
- parse_ctrl_shift_marker
- smoke-imk.sh
- templates.rs
- Kế hoạch ký số (code signing) — TextVN
- os_windows.rs
- quality_trend.rs
- test-typing.ps1
- env-mac — Môi trường build macOS (MAC-001)
- MathContextFsm
- Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)
- C. LINUX + SHARED
- tis-select.swift
- PackageDescription
- text_prediction.rs
- 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)
- e57_math_sync.rs
- e2e-desktop.sh
- mem-check.sh
- soak.sh
- compose.c
- e2e-assist-apps.ps1
- default
- build-macos.sh
- stage-linux.sh
- build-rust.sh
- toggle.rs
- postinstall
- uninstall-check.sh
- install_macos.sh
- notarize-macos.sh
- trace_diff.rs
- uninstall_macos.sh
- functional
- iostream
- keysink
- settings_window
- compose_abi.rs
- ConfigWatcher
- fs
- trace_compose.c
- command
- check_version_sync.rs
- String
- Canvas
- P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công
- properties
- tone_inject_one
- ComposePolicy
- emoji_picker.rs
- tip.rs
- 3. Nhật ký finding
- ghost.rs
- Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)
- verify-store-package.sh
- cuas.rs
- en_office_phrases_10k.rs
- windows-tsf/src/replay.rs
- Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)
- sentence_grammar.rs
- quality_score.rs
- wide
- 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)
- emoji_catalog.rs
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- AXSnapshot
- settings_model.c
- src/style_merge.rs
- ConfigStore
- dual_mode_matrix.rs
- macro.rs
- config/src/sandbox.rs
- BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)
- os
- lc_key
- viqr.rs
- AssistCandidateWord
- fix.rs
- Ref
- diacritic_typo.rs
- argparse
- linux-settings/src/main.c
- macSpellContainsCallback
- SessionShadow
- 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft
- r2_173_prediction_chain.rs
- EngineOptions
- trace.rs
- default
- rsi
- Option
- P0-6 — Solution spec: `textvn-session` (một nghiệp vụ cho 3 OS) + build ARM
- readme_numbers.rs
- TextVN — Bộ gõ tiếng Việt thế hệ mới cho Windows, macOS và Linux
- sandbox_mirror.rs
- Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store
- en_spelling.rs
- e63_expert_personas_audit.rs
- ◻ Đề xuất tính năng (ưu tiên theo giá trị user)
- A. WINDOWS
- Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI
- diacritic_style.rs
- prepare-ppa.sh
- 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM"
- T6 — Test & hardening (tuần 15–18)
- ime_context
- undo.rs
- EmojiPicker
- 3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow)
- Added
- MacroEntry
- inflate_raw_bounded
- English
- Result
- e57_word_sync_fca.rs
- pipe_name
- MathOutputMode
- P4-11 — ĐO TÀI NGUYÊN ASSIST (F61) — TIP THẬT
- Strategy
- DictError
- P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)
- Phân phối Linux — kênh, store và auto-update
- textvn-portable.sh script
- Word
- lc_config_state
- P3-8 — Đối chiếu lỗi của bộ gõ khác với TextVN Linux (2026-11-26/27)
- sign-artifacts.sh
- P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19)
- P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4
- Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại
- Diagnostics
- rules_mac.rs
- verify_full
- P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)
- error_success
- unmark_as_unmark_tone
- ime_key
- ViState
- Rà soát backlog theo tài liệu kế hoạch — 2026-11-08
- eval.rs
- set_reg_value
- PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)
- worker_main
- icons.rs
- display
- classifier.rs
- test_assist_result.c
- HWND
- TextVNAppTests
- super
- predict_impl
- session/tests/invariants.rs
- CachedFormulaStore
- publish-aur.sh
- os_spell.rs
- complete_word_candidates
- Option
- src/emoji.rs
- p11_math_toggle.rs
- T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006
- per_app
- bench/src/main.rs
- abnormal.rs
- ime_assist_issue
- trigger_keys
- core/src/lib.rs
- DialogContext
- properties
- Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit
- `docs/compat.md` — Ma trận tương thích app (L7, P0-4 §1)
- P8/V55 — DFMEA + DRBFM: 5 fix phản hồi test thật của chủ repo (2026-11-05)
- ime_context
- export_diagnostics_zip
- Measure
- 2. Nhật ký finding
- learning_curve.rs
- gen_vocab.rs
- CodingKeys
- caps.rs
- pathlib
- ChordFsm
- emoji_picker.c
- measure_assist_rss.ps1
- CloseHandle
- KeyTranslator
- style_learning_v59.rs
- test-linux-distro-matrix.sh
- StylePending
- tsf-min/src/register.rs
- default
- os_linux.rs
- plan_key_with
- TapPermission
- math
- English
- x86probe
- outputcharset
- test-linux-el8-package.sh
- lc_edit_batch_active
- OwnerRule
- entries
- conflict_notice.rs
- lc_compose.h
- [0.2.14] — 2026-10-03
- MathEngine
- do_register
- worker_main
- engine_boundary_fca.rs
- ModifierToggle
- save
- eval_leak.rs
- Targets JSON — `tools/appcomptest/targets/` (WIN-061)
- display
- [0.2.28] — 2026-10-10
- Bản 0.2.10 + 0.2.11 — toggle = activate bộ gõ, migration nâng cấp, publish hardening
- HookEngine
- Option
- vn_freq_data
- ime_result
- overlay.rs
- P0-7 — TASKS: `textvn-session` + ARM (WBS cho nhiều agent)
- .CreateInstance
- T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007
- paint_d2d
- non_text_focus_gate.rs
- claim_debounced
- idle_ms
- items
- suggest_vn.rs
- max_sentence_chars
- windows-hook/src/lib.rs
- P0-8 — Dream RSI 2.0 · kiến trúc vòng lặp · vòng nghiệp vụ (hiệu năng + chất lượng mọi tính năng)
- P3-6 — TEST PLAN (Linux) — Solution chi tiết
- core
- P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15
- rules_linux.rs
- [0.2.96] — 2026-11-28
- TextVN — Chỉ mục tài liệu & Giao thức Review
- [0.2.38] — 2026-10-24
- Vec
- ime_result
- ImeError
- tests/reset.rs
- mode
- Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent
- quick_telex
- .contains
- src/toggle_corpus.rs
- .commit
- Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành
- Vec
- [0.2.93] — 2026-11-27
- build-linux-el8.sh
- audit_r2_142.rs
- .run
- collect_windows_env_summary
- hud_pause_ms
- libraryloader
- leader_key
- collect_report
- palette_max_results
- ime_assist_issue
- R2-26 — TextVN trong tiến trình sandbox Windows (AppContainer / LPAC)
- test-linux-smoke.sh
- [0.2.3] — 2026-10-01
- KeyKind
- P0 — REVIEW LOG (Phần 0: Nền tảng chung)
- Khắc phục sự cố TextVN (Troubleshooting)
- English
- diag_summary.rs
- Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục
- AutostartManager
- build-linux-compat.sh
- textvn-settings-launcher.sh
- worker_main
- Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất
- 2. Kết quả kiểm thử
- ui_strings_generated.rs
- [0.2.73] — 2026-11-08
- [0.2.64] — 2026-11-04
- [0.2.85] — 2026-11-22
- [0.2.94] — 2026-11-27
- [0.2.95] — 2026-11-27
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
- KeyOutcome
- 4. Linux — `libspelling` (GNOME) **và** `enchant-2`
- test-linux-package.sh
- P4-9 — REVIEW (3 vòng) kết quả Agent Dict: từ điển tiếng Việt (AS-040/AS-041)
- subprocess
- sys
- tempfile
- unicodedata
- urllib_request
- xml_etree_elementtree
- zipfile
- coordination/ — giao nhận việc & thông báo giữa các agent (G21)
- [0.2.81] — 2026-11-18
- run_tray_app
- [0.2.29] — 2026-10-10
- linux-fcitx5/src/engine.h
- .testDefaultConfig
- textvn_core
- ime_imk_input
- [0.2.8] — 2026-10-02
- WndProc
- [0.2.66] — 2026-11-07
- getdpiforwindow
- hidpi
- BIZ-3 — `cargo xtask quality-trend`: bảng xu hướng KPI theo bản cho README/CHANGELOG
- [0.2.91] — 2026-11-26
- EditBatch
- RSI-1 — Bench KSR-oracle / Bench KSR-oracle
- tokenize.rs
- Vec
- probe/src/main.rs
- .run
- B. macOS
- .push
- BIZ-1 — `cargo xtask quality-score`: bảng điểm chất lượng chung
- RSI-7 — Cổng ngôn ngữ cho chữ mờ / Ghost language gate
- PERF-1 — Baseline hiệu năng theo target triple (`perf/baseline-<triple>.json`)
- [0.2.46] — 2026-10-26
- ime_imk_input
- toggle_and_batch.rs
- has_word_icase
- EN
- resolve_baseline_path
- chars
- CTextVNFFI/include/textvn_ffi.h
- ARM-L4 — E2E 6 distro Linux trên container ARM64 (`release.yml`)
- GLOBAL_TOGGLE_TEST_LOCK
- [0.2.10] — 2026-10-03
- error_pipe_busy
- P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết
- [0.2.79] — 2026-11-14
- Session
- sim_run_oracle
- ime_session_event
- ime_compose_plan
- [0.2.77] — 2026-11-12
- COMPAT-1 — Ma trận Windows `windows-2022` + `windows-2025` + `windows-11-arm`
- apply
- ffi/include/textvn_ffi.h
- ime_session_event
- tv_restart_ibus
- mac_corpus_cases.rs
- preflight.rs
- 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành
- ime_compose_plan
- PickerLayout
- 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU
- ime_assist_plan
- Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit
- ARM-L3 — Phát hành gói `TextVN-<v>-linux-aarch64.tar.gz` trong `release.yml`
- ARM-L5 — Gói `.deb arm64` và `PKGBUILD`/`.SRCINFO` `aarch64`
- ARM-W3 — Đăng ký TSF trên Windows ARM64 (IsWow64Process2, forwarder & ACL 4 DLL)
- COMPAT-2 — Mở rộng ma trận distro E2E (claude-opus-c4c5a567)
- English
- ime_assist_plan
- Measurement
- ARM-W6 — `.msixbundle` x64 + arm64 (claude-opus-7e41b2d9) — phần 1 + 2
- ctrl_shift_marker_path
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- lc_sentence_fix_plan
- AppDelegate.swift
- lc_find_settings_binary
- ime_session_cmd
- 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)
- ARM-W5b.md
- BIZ-2 — Miss report Dream RSI (claude-opus-c4c5a567)
- R2-175 — Fcitx5 5.0.x giữ Ctrl+Shift: `textvn doctor` cảnh báo (claude-opus-c4c5a567)
- REG-1 — Sổ hồi quy + gate (claude-opus-c4c5a567)
- REG-2 — Bù bằng chứng cho mã lỗi cũ (claude-opus-c4c5a567)
- ime_imk_op
- 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?
- 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)
- ime_session_cmd
- ime_session_config
- ime_session_out
- ime_imk_plan
- ARM-L1 + ARM-L2 — CI Linux aarch64 native + gate `char` không dấu (claude-opus-c4c5a567)
- ARM-M1.md
- ARM-W1 — Build + test native Windows ARM64 (claude-opus-c4c5a567)
- ARM-W2 — Forwarder ARM64X cho TSF (`textvn-tsf.dll`) (antigravity-gemini-37c7c6bc)
- BOARD-1.md
- LOOP-2 — Chỉ mục tiền tố top-K cho `predict_prefix_candidates` (antigravity-gemini-37c7c6bc)
- RSI-2 — Corpus đánh giá Dream RSI tách tập + gate chống rò rỉ (claude-opus-c4c5a567)
- SF-1b.md
- Audit sau 0.2.68 — DFMEA + DRBFM (R2-140…R2-153)
- 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)
- ime_session_config
- ime_suggest
- ime_suggest
- test-macos-universal-gate.sh
- [0.2.92] — 2026-11-27
- [0.2.13] — 2026-10-03
- Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên
- ARM-M2 — E2E IMK trên runner macOS arm64 — ĐẠT MỘT PHẦN, giới hạn runner (claude-opus-c4c5a567)
- COMPAT-3 — E2E gõ tiếng Việt THẬT trong phiên desktop Linux (claude-opus-c4c5a567)
- FIX-V53 — test ngân sách thời gian v53 chống nhiễu runner (R2-177)
- SF-9 (Phần 1 + Phần 2 / Part 1 + Part 2) — Một nguồn chữ UI & mô hình cài đặt dùng chung 3 nền tảng
- 1. Bảng mã sự cố
- [0.2.35] — 2026-10-23
- TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV
- ime_imk_op
- [0.2.61] — 2026-11-03
- scan_run_key_for_conflicts
- ARM-W7.md
- DreamRsi
- ime_imk_plan
- [0.2.63] — 2026-11-04
- Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được
- Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath
- [0.2.74] — 2026-11-08
- Bản 0.2.16 — portable tự chữa trên Win11 24H2+ (B7) + hết ghost khi gỡ (0.2.15)
- Bản 0.2.31 — hoàn tất roadmap audit 3 nền tảng (A10, B7/B8/B9/B11/B12/B13, C1/C3/C7/C8/C9) + gate clippy-linux
- Bản 0.2.32 — rà soát toàn hệ thống sau 0.2.31 (R2-101…R2-115) + CI từng nền tảng
- Bản 0.2.4 — đăng ký không cần admin + clean composition + Ctrl+Shift single-path
- Bản 0.2.6 — chốt đăng ký TSF VI+EN, hết ghost registration
- Bản 0.2.9 — từ điển EN cá nhân + quy tắc ưu tiên + preflight 17 gate
- regression-index.md
- run-distro-container.sh
- [0.2.75] — 2026-11-08
- Trạng thái ký số macOS — Developer ID, notarization, stapling
- ime_assist_result
- refresh_style_stats
- [0.2.16] — 2026-10-03
- [0.2.47] — 2026-10-26
- [0.2.54] — 2026-10-30
- [0.2.67] — 2026-11-07
- [0.2.70] — 2026-11-08
- with_prefix
- P3-7 — TASKS LINUX (WBS) — Nhận việc từng task
- needs_reopen_notice
- trusted_install_roots

## God Nodes (most connected - your core abstractions)
1. `Engine` - 135 edges
2. `TsfShared` - 116 edges
3. `TextVNInputController` - 111 edges
4. `Báo cáo dựng & kiểm thử — TextVN` - 102 edges
5. `Changelog` - 99 edges
6. `SvcManager` - 76 edges
7. `TextVNState` - 68 edges
8. `AppDelegate` - 67 edges
9. `Issue` - 64 edges
10. `Ctx` - 61 edges

## Surprising Connections (you probably didn't know these)
- `4. C ABI (`ffi/`, header `ffi/include/textvn_ffi.h`)` --references--> `lc_plan_key()`  [INFERRED]
  docs/10-shared/P0-6-SESSION-FRAMEWORK-SPEC.md → adapters/linux-common/src/compose.c
- `Changed` --references--> `lc_state_read_assist_enabled()`  [INFERRED]
  CHANGELOG.md → adapters/linux-common/src/compose.c
- `1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh` --references--> `lc_env_check()`  [INFERRED]
  docs/specs/reference-parity.md → adapters/linux-common/src/env.c
- `9. Task (chi tiết `P3-7-TASKS.md`)` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-4-strategy-appdb.md → adapters/linux-common/src/field_detect.c
- `5. Perf (ngưỡng — `PLAN §5.5`)` --references--> `lc_field_detect()`  [INFERRED]
  docs/40-linux/P3-6-test-plan.md → adapters/linux-common/src/field_detect.c

## Import Cycles
- 1-file cycle: `dict/src/mmap.rs -> dict/src/mmap.rs`
- 2-file cycle: `core/src/transform/vowel_table.rs -> core/src/transform/vowel_table_generated.rs -> core/src/transform/vowel_table.rs`
- 2-file cycle: `dict/src/lib.rs -> dict/src/os_windows.rs -> dict/src/lib.rs`

## Communities (711 total, 105 thin omitted)

### Community 0 - "Ctx"
Cohesion: 0.05
Nodes (43): bat_lap_tu_thuong(), cau_mix_whitelist_theo_tu(), check(), EN_INTENTIONAL_REPEATS, INTENTIONAL_REPEATS, is_aabb(), is_compound_then_modal(), PRONOUN_CAP_FIRE (+35 more)

### Community 1 - "board/src/lib.rs"
Cohesion: 0.08
Nodes (61): agent_of(), allowed_flags(), append(), append_round_trip_on_temp_board(), Args, BOARD_PATH, cargo_subcommand_args(), check_dor() (+53 more)

### Community 2 - "cli/src/replay.rs"
Cohesion: 0.07
Nodes (62): action_name(), adapter_profile(), AdapterProfile, build_json(), canonical_case_rel(), caps_bits(), Case, case_without_any_expectation_is_rejected() (+54 more)

### Community 3 - "config/src/lib.rs"
Cohesion: 0.04
Nodes (56): assist_defaults_match_p4_0_section_7(), assist_full_block_parses(), ASSIST_TRIGGER_KEYS, AssistDisplay, AssistLanguage, AssistSettings, Config, ConfigError (+48 more)

### Community 4 - "TsfShared"
Cohesion: 0.05
Nodes (93): ActiveAssist, apply_assist_suggestion(), apply_assist_undo(), apply_ctrl_tap_in_context(), apply_display_attribute(), apply_plan(), apply_sentence_auto_fix(), apply_sentence_fix_all() (+85 more)

### Community 5 - "abnormal_1000.rs"
Cohesion: 0.07
Nodes (30): a1000_builder_deterministic_va_id_stable(), a1000_tat_ca_case_qua_4_rao_chan(), assert_utf8(), budget_for(), build_cases(), Case, clip(), DECOMP (+22 more)

### Community 6 - "windows-hook/src/main.rs"
Cohesion: 0.08
Nodes (44): CAPS_HELD, caps_lock_bit(), CAPS_TOGGLED, CTRL_DOWN, empty_ime_result(), FocusProbe, get_active_modifiers(), get_process_name_for_window() (+36 more)

### Community 7 - "toml.rs"
Cohesion: 0.15
Nodes (16): fmt, comment_trong_chuoi_khong_bi_cat(), Doc, hex_int_cho_bang_keycode(), parse(), parse_bang_doc_day_du(), parse_value(), BTreeMap (+8 more)

### Community 8 - "xtask/src/main.rs"
Cohesion: 0.11
Nodes (38): insert_nested(), banner(), emit(), format_rust(), gen_tables(), index_of(), key_char(), KEYMAP_MAC_OUT_RUST (+30 more)

### Community 9 - "ffi/src/lib.rs"
Cohesion: 0.03
Nodes (104): remove_auto_capitalize_conflicts(), assist_check_finds_vietnamese_issue(), assist_check_ignores_clean_or_english_text(), assist_check_instance_reads_sentence_buffer(), assist_check_v1_fail_open_on_empty_null_and_bad_utf16(), assist_check_v1_matches_utf8_path(), assist_check_v1_utf16_offset_is_char_not_utf16(), assist_plan_sentence_fix_happy_path() (+96 more)

### Community 10 - "strategy/src/lib.rs"
Cohesion: 0.07
Nodes (22): ActionKind, Option, IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY (+14 more)

### Community 11 - "vowel_table_generated.rs"
Cohesion: 0.08
Nodes (27): BREVE, CIRCUMFLEX, HORN, is_marker(), REMOVE_MARKS_KEY, Option, STROKE_DOUBLE, STROKE_KEY (+19 more)

### Community 12 - "dream_rsi_oracle_run"
Cohesion: 0.13
Nodes (12): check_dream_rsi(), dream_rsi_check_hoi_quy(), dream_rsi_gate_duot_bien_ghost_sai(), dream_rsi_json_co_dinh_dang(), dream_rsi_oracle_run(), dream_rsi_report(), dream_rsi_sim_deterministic(), dream_rsi_sim_khong_lech_dich_va_giu_bat_bien() (+4 more)

### Community 13 - "cli/src/register.rs"
Cohesion: 0.05
Nodes (37): call_layout_or_tip(), call_layout_or_tip_spec(), CAT_DISPLAY_ATTRIBUTE_PROVIDER, CAT_IMMERSIVE, CAT_SYSTRAY, CAT_TIP_KEYBOARD, CLSID_INNER, clsid_key() (+29 more)

### Community 14 - "doc.rs"
Cohesion: 0.10
Nodes (32): atomic_write(), create_private_dir(), default_at(), defaults(), DocKind, invalid_values_are_rejected_without_changing_the_doc(), lookup_nested(), macros_text_round_trip_through_doc() (+24 more)

### Community 15 - "keymap.rs"
Cohesion: 0.04
Nodes (36): BACK, CAPSLOCK, CONTROL, DELETE, DOWN, END, ESCAPE, F1 (+28 more)

### Community 16 - "audit_r2_140.rs"
Cohesion: 0.08
Nodes (45): abnormal_fuzz_tong_hop_bat_bien(), abnormal_ngoac_gach_ngang_tron_chu_khong_mat_ky_tu(), abnormal_peek_khong_doi_ket_qua_phim_that(), abnormal_tu_cuc_dai_va_ngoac_lien_tiep(), char_key(), ctrl_space(), field(), learning() (+37 more)

### Community 17 - "validate.rs"
Cohesion: 0.12
Nodes (23): unmark(), coda_after_nucleus_ok(), CODAS, decompose(), has_vowel(), is_valid_word(), lower_str(), NUCLEI (+15 more)

### Community 18 - ".new"
Cohesion: 0.22
Nodes (19): build_candidates_cli(), comma_join(), comma_sorted_join(), fetch_cache_bytes(), kappa_cli(), kappa_report(), load_valid_syllables(), pool_from_local_ass() (+11 more)

### Community 19 - "telex.rs"
Cohesion: 0.07
Nodes (23): codas, char_idx(), circum_pair(), f(), fold(), fold_with(), fold_with_caps(), fold_with_caps_impl() (+15 more)

### Community 20 - "P1-2 — HOOK ADAPTER (Windows) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Mapping task (chi tiết `P1-6-TASKS.md`), 11. Failure modes, 1. Quyết định & phạm vi, 2. Kiến trúc tiến trình, 3. Hook callback — luật bất biến, 4. Focus & field detect (không trong callback), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (bug B1 — address bar/Excel) (+9 more)

### Community 21 - "windows-tsf/src/lib.rs"
Cohesion: 0.03
Nodes (64): mode1_display_enabled(), mode2_auto_fix_enabled(), after_request_edit_session(), classify_tsf_field(), config_file_path(), d16_auto_capitalize_theo_config(), DllCanUnloadNow(), DllGetClassObject() (+56 more)

### Community 22 - "restore_en.rs"
Cohesion: 0.07
Nodes (22): en_common_contains(), EN_COMMON_DATA, EN_EXTRA_LEXICON_WORDS, EN_FREQ_DATA, EN_WORDS_10K_DATA, in_base_lexicon_lower(), in_undo_exact_lexicon(), is_cvc_single_sfr() (+14 more)

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
Nodes (74): lc_assist_is_active(), lc_comp_apply(), lc_log(), lc_utf32_to_utf8(), apply_assist_candidate(), apply_assist_undo(), assist_idle_arm(), assist_idle_cancel() (+66 more)

### Community 28 - "P2-1 — IMK ADAPTER (macOS) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Playbook triển khai (mapping `P2-6-TASKS.md`), 11. Chẩn đoán, 12. Failure modes, 1. Quyết định & capability, 2. Cấu trúc bundle & repo, 3. Vòng đời IMK, 4. State machine (giống `P1-1 §4`, marked text thay composition), 5. Key flow (+9 more)

### Community 29 - "syllables.rs"
Cohesion: 0.07
Nodes (22): CONFIDENCE_AMBIGUOUS, CONFIDENCE_UNAMBIGUOUS, confusable(), DATA, decompose(), dict(), dong_bat_thuong_khong_panic(), EXTRA (+14 more)

### Community 30 - "selection_check.rs"
Cohesion: 0.14
Nodes (22): ApplySelectionFixSession, DEBOUNCE_MS, get_selection_anchor(), is_selection_key_up(), MAX_SELECTION_CHARS, MAX_TRANSCODE_CHARS, MIN_SELECTION_CHARS, on_key_up() (+14 more)

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
Cohesion: 0.07
Nodes (29): APP_ID, ARG_AUTOSTART, ARG_GUARD, ARG_INSTALL, ARG_RELAY, build_command_line(), decide_store_run_value(), install_args() (+21 more)

### Community 35 - "ipc/src/sandbox.rs"
Cohesion: 0.10
Nodes (32): current_user_sid_string(), full_pipe_sddl_unchanged_and_has_no_appcontainer_access(), image_is_textvn_exe(), open_tray_pipe(), open_tray_pipe_from(), path_dacl_sddl(), pipe_sddl(), PipeOpen (+24 more)

### Community 36 - "Chi tiết từng mục"
Cohesion: 0.13
Nodes (14): #10 — SendInput đến app elevated (UIPI), #1 — `CoCreateInstance` trong DLL, #2 — `ITfKeystrokeMgr::AdviseKeyEventSink`, #3 — `GetStart` / `GetSelection` (+ finding), #4 — `StartComposition` → `SetText` → `EndComposition`, #5 — Bảng `OnTestKeyDown` vs `OnKeyDown`, #6 — Đăng ký scope=user không admin, #7 — Win+Space + `ActivateProfile` (+6 more)

### Community 37 - "Result"
Cohesion: 0.11
Nodes (15): ApplyAssistFixSession_Impl, ApplyAssistUndoSession_Impl, ApplySentenceFixSession_Impl, AssistRecallSession_Impl, CompSink_Impl, CtrlTapSession_Impl, EmojiCommitSession_Impl, EndCompositionSession_Impl (+7 more)

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
Cohesion: 0.08
Nodes (25): BUILD_GIT_SHA, BUILD_TARGET, classify_distribution(), collect_linux_env_summary(), command_on_path(), current_distribution(), current_euid(), detect_fcitx5_profile_conflicts() (+17 more)

### Community 57 - "StrategyResolver"
Cohesion: 0.06
Nodes (34): Key, StrategyResolver, .cacheCount, .hasAppdb, Bool, Data, Int, Int32 (+26 more)

### Community 58 - "P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`), 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`), 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`), 4. IPC/state/health (mirror `P2-4 §6`), 5. Cài đặt & gỡ — `.deb` (primary), 6. `textvn doctor` (bản Linux — `PLAN §5.3`), 7. Update — **không self-update** (quyết định riêng của Linux), 8. Phân phối (+2 more)

### Community 59 - "string"
Cohesion: 0.07
Nodes (45): lc_detect_active_framework(), lc_env_check(), get_time_ms(), lc_classify_field(), lc_field_detect(), lc_is_terminal_app(), str_contains_icase(), ensure_parent_dirs() (+37 more)

### Community 60 - "T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004"
Cohesion: 0.18
Nodes (11): LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`, LNX-011 · process_key_event → PASS (M) — `P3-1 §5`, LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`, LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`, LNX-014 · SelectionReplace (L) — `P3-1 §6.2`, LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`, LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`, LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005) (+3 more)

### Community 61 - "settings_dialog.rs"
Cohesion: 0.02
Nodes (113): assist_mode_hints, MacroCtl, normalize_word_list, about_text(), about_text_matches_every_platform(), BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX (+105 more)

### Community 62 - "Issue"
Cohesion: 0.08
Nodes (37): span_len(), abbrev_khong_nuot_tu_2_chu_cai_va_dau_cham_ngoac_kep(), check(), check_sentence_with(), check_sentence_with_kiem_tra_ca_dau_cau_va_spelling(), check_sentence_with_style(), check_with(), check_with_bao_tu_la_theo_tu_dien() (+29 more)

### Community 63 - "appdb/src/lib.rs"
Cohesion: 0.10
Nodes (34): AppDb, AppDbError, DB, documented_metadata_and_match_clause_parse(), engine_version_is_supported(), EngineOwner, Entry, FieldRoles (+26 more)

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
Cohesion: 0.30
Nodes (19): textvn-appdb, textvn-assist, textvn-bench, textvn-board, textvn-cli, textvn-config, textvn-core, textvn-dict (+11 more)

### Community 70 - "type"
Cohesion: 0.11
Nodes (24): items, default, items, type, $comment, default, items, type (+16 more)

### Community 71 - "windows-tsf/src/compose.rs"
Cohesion: 0.04
Nodes (48): BACK, backspace_in_composition_matrix(), backspace_inside_word_refolds_and_empty_word_ends_composition(), CAPITAL, composition_holds_whole_word_until_boundary(), CONTROL, ctrl_space_plan_never_inserts_space_on_pass(), CTRL_TAP_MAX (+40 more)

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
Nodes (34): KeyEvent, AssistCandidate, SentenceFixPlan, TextTarget, Any, AssistIssue, AssistSettings, AssistUndoState (+26 more)

### Community 79 - "predict.rs"
Cohesion: 0.05
Nodes (70): adapt_capitalization(), adapt_continuation_case(), char_prefix(), collect_phrase_matches(), du_lieu_phrases_invariant(), en_common_lines(), en_compounds(), en_compounds_buckets() (+62 more)

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
Cohesion: 0.10
Nodes (19): merge_style_drain(), parse_style_drain(), Option, RsiTotals, bam_cong_tac_ghi_decided_va_ton_trong_lua_chon(), doc(), file_cu_enabled_false_chua_decided_van_bat(), mac_dinh_giu_du_lieu_hoc_va_tra_cong_tac_ve_bat() (+11 more)

### Community 86 - "2. Cách lấy oracle"
Cohesion: 0.18
Nodes (10): 1. Kết quả kiểm tra môi trường Windows (WIN-007), 2. Cách lấy oracle, 3. Quy tắc chung, 4. Coverage yêu cầu & Tiến độ, Cách A (khuyến nghị cho CI/Linux): x-unikey CLI, Cách B: UniKey Windows trên VM/Desktop (Áp dụng cho WIN-007), Cách C (dự phòng): Bảng Unicode chuẩn + quy tắc chính tả tiếng Việt, Cách D (đã kiểm chứng 2026-11-27, máy dev Windows): build `ukengine` x-unikey 1.0.4 bằng MSVC (+2 more)

### Community 87 - "png.rs"
Cohesion: 0.11
Nodes (32): ico_eight_frames_header(), adler32(), bbox(), BitReader, BitWriter, CL_ORDER, crc32_update(), decode_png_rgba() (+24 more)

### Community 88 - "textvn-common.sh"
Cohesion: 0.14
Nodes (27): put(), install.sh script, usage(), tv_check_ime_conflicts(), tv_check_package(), tv_desktop_quote(), tv_detect_pkg_mgr(), tv_diag_log() (+19 more)

### Community 89 - "art_icons.rs"
Cohesion: 0.12
Nodes (33): encode_rgba, APP_E, APP_V, apply_rounded(), art_compose_layout_and_png_size_under_hygiene_limit(), ART_SHADOW, ART_WHITE, badge_png_valid_structure() (+25 more)

### Community 90 - "config.v1.schema.json"
Cohesion: 0.25
Nodes (7): additionalProperties, $comment, $id, required, $schema, title, type

### Community 91 - "Verified Ops — sổ thao tác / tool đã kiểm chứng"
Cohesion: 0.29
Nodes (6): A. Repo · Git · Graphify, B. Build · Test · Script, C. Windows platform (chi tiết → spike specs), D. Môi trường máy dev (snapshot 2026-09-27) — khỏi kiểm tra lại, E. Fixture / app test sẵn có (không dựng lại), Verified Ops — sổ thao tác / tool đã kiểm chứng

### Community 92 - "crate"
Cohesion: 0.06
Nodes (35): bat_dau_cau_thuong(), check(), is_sentence_start(), looks_like_code_or_list(), Vec, bat_cum_sai_va_giu_hoa_thuong(), check(), OFFICE_BIGRAMS (+27 more)

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
Cohesion: 0.09
Nodes (14): appended_part(), Engine, Outcome, plausible_ascii_word(), PredictCycle, retype_chunk(), Cell, EmojiEntry (+6 more)

### Community 97 - "config.rs"
Cohesion: 0.08
Nodes (30): ConfigError, combo_signature(), ConfigError, custom_output_strips_backslash(), default_config_backward_compatible(), default_hotkeys(), e70_protected_hotkeys_case_insensitive(), HotkeySpec (+22 more)

### Community 98 - "T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005"
Cohesion: 0.29
Nodes (7): LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`, LNX-031 · Cache + AT-SPI event invalidate + budget (M), LNX-032 · Preset 20 app + corpus ≥ 40 case (M) — `P3-4 §3`, LNX-033 · Appdb loader + `ime_appdb_verify` (S) — P0-2 §1, không viết lại, LNX-034 · Override chain `P3-4 §6` (path Linux) (M), LNX-035 · `doctor` env matrix + framework auto (M) — `P3-4 §7`, `P3-5 §6`, T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005

### Community 99 - "P3-REVIEW-LOG — Phần 3 (Linux)"
Cohesion: 0.22
Nodes (8): Code Review Round 1 — Đúng & Đủ (Mã nguồn C/C++), Code Review Round 2 — Nhất quán & Sẵn sàng (Mã nguồn & Kiểm thử), Kiểm chứng sau fix (Review 2 cuối), Kết quả tổng, P3-REVIEW-LOG — Phần 3 (Linux), Review 1 — Đúng & Đủ, Review 2 — Nhất quán & Sẵn sàng, Rủi ro còn mở của Phần 3 (theo dõi, không phải finding)

### Community 100 - "spikes/README.md"
Cohesion: 0.06
Nodes (28): 1. Câu hỏi và tiêu chí đóng, 2. Thiết kế, 3. Kết quả CI, Spike SPK-ARM64X — một TIP cho tiến trình x64 giả lập và ARM64 native (`spikes/arm64x-forwarder`), 1. Bảng kết quả — Local vs GHA (n=1 mỗi dòng, 3 lần dispatch GHA), 2026-11-27 — mở lại thành SPK-RUNNER (ma trận runner hiện tại), 2. Findings `S5-{n}` (không xóa), 3. Quyết định nightly (acceptance `P1-5 §1/§5`) (+20 more)

### Community 101 - "SvcManager"
Cohesion: 0.06
Nodes (35): DocError, apply_option_checkbox(), atomic_write_file(), corrupt_config_is_backed_up_not_lost(), default_config_dir(), effective_enabled(), failed_config_save_keeps_memory_and_version_unchanged(), failed_state_save_keeps_memory_and_version_unchanged() (+27 more)

### Community 102 - "IpcClient"
Cohesion: 0.04
Nodes (51): IpcServerAssistTests, Any, Int, Int32, IpcServer, AssistReportIssue, .json, Frame (+43 more)

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

### Community 107 - "ImeEngine"
Cohesion: 0.04
Nodes (34): AssistIssue, FFI, ImeEngine, AnyObject, Bool, CChar, ime_result_v1, Int32 (+26 more)

### Community 108 - "when"
Cohesion: 0.33
Nodes (6): oneOf, field_role, when, additionalProperties, properties, type

### Community 109 - "P1 — REVIEW LOG (Phần 1: Windows)"
Cohesion: 0.40
Nodes (4): Cross-check checklist (Review 2) — kết quả, P1 — REVIEW LOG (Phần 1: Windows), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27)

### Community 110 - "atomic"
Cohesion: 0.17
Nodes (12): ClassFactory, HR_CLASS_E_NOAGGREGATION, HR_E_POINTER, HR_S_OK, OBJECT_COUNT, ObjGuard, AtomicI32, Default (+4 more)

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
Cohesion: 0.08
Nodes (28): ImkPlanner, Op, clearMarkedState, deleteBackward, insertAtCaret, insertAtMarkedRange, insertAtSelection, resetEngine (+20 more)

### Community 116 - "ffi/src/settings.rs"
Cohesion: 0.15
Nodes (35): assist_nested_paths_round_trip(), c(), c_api_round_trip(), cstr(), doc_err(), english_words_text_round_trip(), HEADER, ime_settings (+27 more)

### Community 119 - "office_productivity.rs"
Cohesion: 0.13
Nodes (15): apply_outcome(), hd_cty_macros(), key_for_char(), office_auto_capitalize_across_sentences_and_newlines(), office_backspace_mid_word_and_continue_typing(), office_bilingual_sentences_auto_restore_cleanly(), office_custom_english_words_and_escape_restore(), office_legacy_charsets_keep_sentence_tail_and_glyph_len_synced() (+7 more)

### Community 120 - "Equatable"
Cohesion: 0.15
Nodes (25): AssistDisplay, AssistSettings, .isOn, EmojiEntry, MathHotkey, MathSettings, RsiTotals, .isZero (+17 more)

### Community 121 - "style_profile.rs"
Cohesion: 0.14
Nodes (21): c(), correction_for_linear(), CorrectionLookup, MAX_CORRECTIONS, MAX_ENTRY_CHARS, MAX_TOTAL_BYTES, MAX_WORDS, p7_bytes_cap_16kb() (+13 more)

### Community 122 - "P4-0 — MASTER PLAN: Trợ lý ngữ pháp tiếng Việt (Assist) cho TextVN"
Cohesion: 0.09
Nodes (22): 0. Tóm tắt điều hành, 10. Rủi ro & giảm thiểu (cập nhật sau vòng 3–5), 11.1 Vòng 2 — "Nhất quán & Sẵn sàng" (đã xong ở v1), 11.2 Vòng 3–5 — review chuyên sâu (2026-10-20, 3 lượt: SA/QA → Audit → PO/BA/Design), 11.3 Vòng 6 — CROSS-REVIEW (agent B, 2026-10-20) + phản hồi agent A, 11.4 Vòng 7–9 — review **sau khi implement P4-1** (2026-10-20), 11.5 Kết luận các lượt review, 11.6 Vòng 8–10 — review **sau khi làm 4 spike** (2026-10-20) → `P4-3-SPIKES.md` (+14 more)

### Community 123 - "session_abi.rs"
Cohesion: 0.04
Nodes (74): assistitem, ime_session_event(), ime_session_free(), ime_session_new(), ime_session_reload_config(), ime_session, borrow_str(), caps_from_bits() (+66 more)

### Community 124 - "ARM-W4 — Bộ cài Inno Setup + portable ZIP Windows ARM64 (ADR-015)"
Cohesion: 0.06
Nodes (29): ARM-W4 — Bộ cài Inno Setup + portable ZIP Windows ARM64 (ADR-015), English, English — E89 CONCLUSION (2026-11-28, run cb28db20 — decisive dump), English — E89 PROBE BUGFIX (2026-11-28, run 504960ff), English — E89 round 2 (2026-11-28, CI run 46111640219), English — E89 round 3 (2026-11-28, CI run 46113896660), English — Post-review fix (E89, 2026-11-28), Kiểm chứng (+21 more)

### Community 127 - "idle_popup_routing.rs"
Cohesion: 0.13
Nodes (17): check_sentence_on_demand(), AUTOCORRECT, che_do_tat_thi_im_lang_o_moi_diem_dung(), dung_1s_o_che_do_goi_y_bao_loi_dung_cho(), dung_1s_o_che_do_sua_loi_an_toan_bao_loi_dung_cho(), feed(), HIGHLIGHT, lap_lai_cung_mot_diem_dung_cho_cung_ket_qua() (+9 more)

### Community 128 - "test_delete_surrounding.cpp"
Cohesion: 0.07
Nodes (28): InputContext, InputContext, string, surroundingTextBeforeCaretMatches(), textBeforeCaretMatches(), DeleteCall, offset, size (+20 more)

### Community 129 - "Attribution dữ liệu Tatoeba (CC BY 2.0 FR)"
Cohesion: 0.10
Nodes (17): 1. Mục tiêu và phạm vi kiểm tra, 2. Kết quả kiểm tra chi tiết theo kênh, 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm, 4. Quyết định & Kết luận, WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN", 1. Bản attribution ngắn (để dán vào README / CHANGELOG / gói phát hành), 2. Nguồn dữ liệu, 3. Vì sao attribution là bắt buộc (+9 more)

### Community 130 - "vni.rs"
Cohesion: 0.14
Nodes (19): late_stroke(), push_key(), fold(), fold_impl(), fold_steps(), free_marking_off_requires_adjacent(), n(), o() (+11 more)

### Community 131 - "EmojiPicker"
Cohesion: 0.16
Nodes (15): clamp_to_monitor(), create_popup_window(), EmojiPicker, lazy_dwm_round_corners(), Default, Drop, F, HWND (+7 more)

### Community 132 - "method/mod.rs"
Cohesion: 0.15
Nodes (19): f(), finish(), fold(), fold_caps(), fold_effective_caps(), FoldEffective, generated_key_tables_are_self_consistent(), is_word_char() (+11 more)

### Community 133 - "Antivirus false positive (RW3) — Kaspersky flag build production"
Cohesion: 0.12
Nodes (17): 10. Sự cố 2026-10-11 — Kaspersky cách ly bản cài từ Microsoft Store (MSIX 1.2.29.0), 1. Sự việc, 2. Bang trigger — đã xác minh trên code/installer, 3. SHA256 artifacts hiện tại (dùng cho AV-1 submit), 4.1. Các phương án đã cân nhắc và lý do KHÔNG làm, 4. Ke hoach xu ly (task), 5. Runbook submit Kaspersky (AV-1), 6. Ke hoach ky so (AV-2) — theo P1-4 §5 (+9 more)

### Community 134 - "Message"
Cohesion: 0.08
Nodes (31): read_next_message(), Result, W, send_message(), encode_outgoing(), frame_buffer_handles_short_reads_coalesced_frames_and_oversize(), FrameBuffer, Result (+23 more)

### Community 135 - "win_tools.rs"
Cohesion: 0.08
Nodes (30): deflate_fixed, arm_w2_def_files_va_args_hop_le(), arm_w2_pe_is_arm64x_phan_biet_arm64x_voi_arm64_thuong_va_x64(), build_arm64x_cli(), build_synthetic_arm64x_pe(), BuildArm64xArgs, contains_vietnamese(), EXPECTED_DISPLAY_NAME (+22 more)

### Community 138 - "store_win.rs"
Cohesion: 0.11
Nodes (40): appx, commandext, OsStr, shellexecutew, LaunchMode, after_tray_start(), cmd_exe_path(), CREATE_BREAKAWAY_FROM_JOB (+32 more)

### Community 139 - "Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)"
Cohesion: 0.22
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

### Community 145 - "say"
Cohesion: 0.20
Nodes (17): delete_tree(), do_unregister(), icon_path(), is_service_profile_sid(), register_wow64_mirror(), registration_owned_by(), registration_owned_by_dir(), say() (+9 more)

### Community 147 - "EmojiPickerState"
Cohesion: 0.12
Nodes (13): D2dResources, EmojiPickerState, Inner, lazy_d2d1_create_factory(), lazy_dwrite_create_factory(), PickerHit, PickerKeyAction, ID2D1DCRenderTarget (+5 more)

### Community 149 - "autostart.rs"
Cohesion: 0.06
Nodes (63): pcwstr, REG_SAM_FLAGS, registry, APP_RUN_VALUE_NAME, autostart_command(), autostart_disabled_by_user(), AUTOSTART_DISABLED_MARKER, autostart_disabled_marker_path() (+55 more)

### Community 150 - "ime_instance"
Cohesion: 0.08
Nodes (36): EmojiItem, copy_str_to_c_buf(), copy_str_to_c_buf_khong_cat_giua_codepoint_utf8(), explicit_fix_text(), guarded(), ime_appdb_verify(), ime_assist_glyph_len(), ime_assist_plan_sentence_fix() (+28 more)

### Community 151 - "Changelog"
Cohesion: 0.02
Nodes (118): [0.2.11] — 2026-10-03, [0.2.17] — 2026-10-03, [0.2.18] — 2026-10-03, [0.2.19] — 2026-10-04, [0.2.1] — 2026-09-30, [0.2.21] — 2026-10-04, [0.2.22] — 2026-10-04, [0.2.23] — 2026-10-05 (+110 more)

### Community 153 - "tray/src/main.rs"
Cohesion: 0.07
Nodes (37): id_exit, shell, tray_icon_uid, APP_INSTANCE, check_status(), ensure_tsf_tip_registered(), flag_value(), free_ctrl_shift_cli() (+29 more)

### Community 154 - "menu.rs"
Cohesion: 0.09
Nodes (29): HMENU, add_radio_menu_item(), ID_CURRENT_APP_TOGGLE, ID_DIACRITIC_NEW, ID_DIACRITIC_OLD, ID_EXIT, ID_EXPORT_DIAG, ID_HEALTH_STATUS (+21 more)

### Community 155 - "SettingsController"
Cohesion: 0.11
Nodes (18): Arc, AtomicBool, BTreeMap, DiacriticStyle, Duration, Instant, IpcServer, MacroTrigger (+10 more)

### Community 156 - "time"
Cohesion: 0.07
Nodes (16): BridgeDict, BridgeDict<F>, Budgeted, Budgeted<'a, D>, budgeted_boc_no_dictionary_fail_open(), Budgeted<'_, D>, Dictionary, NoDictionary (+8 more)

### Community 157 - "ipc_client.c"
Cohesion: 0.15
Nodes (39): lc_ipc_client, client_disconnect(), get_time_ms(), handle_ipc_message(), ipc_debug_enabled(), lc_ipc_client_check_config_reload(), lc_ipc_client_free(), lc_ipc_client_get_app_override() (+31 more)

### Community 158 - "P4-4 — Tận dụng năng lực check ngôn ngữ **có sẵn của OS/app** (OS-first) — nghiên cứu & thiết kế"
Cohesion: 0.18
Nodes (10): 0. Kết luận 1 dòng, 1. API có sẵn — đã verify (2026-10-21), 2. So sánh "OS-first" vs "tự mang từ điển", 3. Kiến trúc provider (đề xuất), 4. Quyết định mới (bổ sung D32–D37), 5. Finding mới (Fassist-67..72) — rút ra từ nghiên cứu này, 6. Tác động lộ trình, 7. Rủi ro mới (+2 more)

### Community 159 - "os_macos.rs"
Cohesion: 0.08
Nodes (19): MAX_SUGGEST_RESULTS, ObjcObject, ObjcSelector, PROBE_TIMEOUT, Duration, SEL_AVAILABLE_LANGUAGES, SEL_CHECK_SPELLING, SEL_CLOSE_DOC (+11 more)

### Community 160 - "assist_tools.rs"
Cohesion: 0.07
Nodes (22): civil_date_from_epoch(), civil_from_days(), EN_EXCLUDE_IDS, EN_INTENTIONAL_REPEATS, EN_MAX_LEN, EN_MAX_SENTENCES, EN_MIN_LEN, EN_SEED (+14 more)

### Community 162 - "mmap.rs"
Cohesion: 0.08
Nodes (34): CreateFileMappingW(), CreateFileW(), FILE_ATTRIBUTE_NORMAL, file_len(), FILE_MAP_READ, FILE_SHARE_READ, GENERIC_READ, INVALID_HANDLE_VALUE (+26 more)

### Community 163 - "README.md"
Cohesion: 0.14
Nodes (7): 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh, 2. Tính năng tham chiếu → trạng thái, Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28), Báo cáo lỗ hổng bảo mật, Các phiên bản được hỗ trợ, Security Design, Security Policy

### Community 164 - "package_bootstrap.rs"
Cohesion: 0.26
Nodes (19): cleanup_old_backups(), copy_dir_recursive(), copy_overwrite(), dispatch(), installed_dir_for(), Option, Path, PathBuf (+11 more)

### Community 165 - "win32-uia.lib.ps1"
Cohesion: 0.36
Nodes (4): Find-UiAElementByLocator(), Get-UiAControlType(), Initialize-TextVNUiA(), New-UiACond()

### Community 166 - "Dict"
Cohesion: 0.16
Nodes (17): Broker, Dict, EnchantBroker, EnchantDict, raw_broker_free(), raw_dict_check(), raw_dict_exists(), raw_dict_suggest() (+9 more)

### Community 167 - "engine.cpp"
Cohesion: 0.04
Nodes (129): lc_assist_debug_enabled(), lc_assist_live_key(), lc_assist_should_auto_fix(), lc_assist_should_display(), lc_config_sync(), lc_fixed_sentence(), lc_is_transcode_hotkey(), lc_plan_sentence_fix_ex() (+121 more)

### Community 168 - "syllable_spelling.rs"
Cohesion: 0.26
Nodes (11): CLEAN, clean_corpus_fpr_at_most_half_percent(), clean_sentences(), EXTRA, injected_tone_errors_are_caught(), key_and_tone(), ROWS, Vec (+3 more)

### Community 170 - "ipc_client.rs"
Cohesion: 0.08
Nodes (36): assist_state_update_and_snapshot(), CLIENT, connect_tray(), GLOBAL_KEY, global_switch_from_tray_disables_every_app(), IpcClient, IpcState, LAST_HOTKEY_TOGGLE_MS (+28 more)

### Community 172 - "verify_targets.ps1"
Cohesion: 0.38
Nodes (10): Expand-Path(), Find-AppWindow(), Get-AppVersion(), Get-FixtureUri(), Get-ProcNameOfHwnd(), Get-ResolvedArgs(), New-ProfileDir(), Resolve-CommandPath() (+2 more)

### Community 173 - "verify.rs"
Cohesion: 0.06
Nodes (59): abi_constants(), abi_exports(), abi_structs(), default_header(), define_boc_comment_va_bo_guard(), export_dao_thu_tu_bi_phat_hien(), export_rust_dung_thu_tu_header_that(), export_thieu_ham_bi_phat_hien() (+51 more)

### Community 174 - "Performance Audit — TextVN Windows Platform"
Cohesion: 0.11
Nodes (17): 1. Non-Functional Requirements (NFR), 2. Phân tích hot path, 2a. WH_KEYBOARD_LL Hook Callback, 2b. TSF Key Event Sink, 2c. IPC Named Pipe, 3. CPU Usage Analysis, 4. Memory Footprint, 5. Startup Time Budget (+9 more)

### Community 175 - "gen_ui.rs"
Cohesion: 0.09
Nodes (49): camel(), check_snake_id(), check_text(), hand_copies(), HAND_WRITTEN, header(), is_generated(), LICENSE (+41 more)

### Community 177 - "P4-12 — Recall trên **văn bản thật** (bộ nhớ có nhãn) + FPR"
Cohesion: 0.06
Nodes (34): 10. Cập nhật 2026-10-23 — nâng recall `diacritic_typo` (đóng **D19**), 11. Cập nhật 2026-10-23 (vòng 23 — audit "như Grammarly"): FPR → 0 %, 12.1 Số đo hiện tại (cùng 2 corpus, CLI debug rebuild), 12.2 Cảnh báo: recall 83,7 % KHÔNG còn là số độc lập, 12.3 Thí nghiệm bổ sung: spelling theo âm tiết bắt được gì mà bảng TYPOS không bắt?, 12.4 Công cụ cho P4-12c, 12. Cập nhật 2026-11-08 — đo `spelling` bằng từ điển (P4-12a) và cảnh báo quá khớp, 13.1 Số đo (cùng máy, build debug) (+26 more)

### Community 178 - "spelling.rs"
Cohesion: 0.22
Nodes (17): check(), check_text(), FakeDictionary, match_case(), MAX_SUGGESTIONS, MIN_WORD_CHARS, ngan_sach_suggest_het_thi_tu_la_van_duoc_bao_khong_kem_goi_y(), no_dictionary_khong_bao_gi() (+9 more)

### Community 179 - ".suggest"
Cohesion: 0.07
Nodes (38): [0.2.55] — 2026-10-30, Added, Fixed, 0. Kết luận 1 trang, 10. Phát hiện mới (đề xuất mã — **chờ chủ `P4-TASKS` cấp số**, không tự gán `Fassist-*`), 11. TL;DR cho người code, 1.1 Trait (nguồn: `assist/src/dict.rs`), 1.2 Luật tiêu thụ trait (nguồn: `assist/src/rules/spelling.rs`) (+30 more)

### Community 180 - ".new"
Cohesion: 0.16
Nodes (12): contains_khong_reset_cua_so_con_han(), EnchantSpellChecker, feature_tat_thi_fail_open_toan_phan(), AtomicUsize, Instant, Job, Result, Self (+4 more)

### Community 181 - "build-release.ps1"
Cohesion: 0.31
Nodes (6): Assert-PeMachine(), Copy-DllAllowLocked(), Sign-TextVNFile(), Write-Fail(), Write-Ok(), Write-Warn()

### Community 182 - "AppDelegate"
Cohesion: 0.05
Nodes (34): AppDelegate, .frontAppEnabled, .isVietnameseMode, .vietnameseModeEnabled, Bool, Date, Int, Int32 (+26 more)

### Community 184 - "P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN"
Cohesion: 0.10
Nodes (19): 0. Quyết định (TL;DR), 10. Liên kết, 1. Harper là gì — facts (đã xác minh, không suy đoán), 2. Kiến trúc `harper-core` (master, 2026-10-20) — bản đồ module, 3. Khoảng cách tiếng Việt — bảng "mượn được gì", 4.1 PO (Product Owner), 4.2 BA (Business Analyst), 4.3 SA (Solution Architect) (+11 more)

### Community 185 - "rules_win.rs"
Cohesion: 0.10
Nodes (26): b1_url_search_selection_replace(), body_preedit(), candidate_selection_replace(), caps_missing_downgrades(), db(), disabled_app_passthrough(), finder_rename_editbox(), MAC_CAPS (+18 more)

### Community 186 - "P3-3 — X11 FALLBACK (Linux, opt-in) — Solution chi tiết"
Cohesion: 0.11
Nodes (17): 10. Task (chi tiết `P3-7-TASKS.md`), 1. Quyết định & phạm vi, 2. Kiến trúc, 3. Bắt phím & callback (mirror `P1-2 §3`, `P2-2 §3`), 4. Feature detect (mirror `PLAN §5.3` — weak detection), 5.1 `BackspaceType`, 5.2 `SelectionReplace` (B1 — app không qua framework), 5.3 `ForwardAsCommit` — chèn không xóa (terminal X11 hiếm khi cần). (+9 more)

### Community 188 - "sandbox_e2e.rs"
Cohesion: 0.18
Nodes (15): duration, io, PSID, textvn_ipc, CHILD_ENV, child_in_appcontainer(), env(), own_app_id() (+7 more)

### Community 189 - "dictionary_budget.rs"
Cohesion: 0.18
Nodes (16): bo_voc_dyn_dictionary_truc_tiep_giong_ctx_dict(), bridge_dict_chuyen_tiep_dung_contains_va_suggest(), bridge_dict_voi_no_dictionary_co_tich_cac_duoc(), budget_khong_cho_phep_vuot_max_suggest(), contains_di_qua_trong_suot_va_khong_ton_ngan_sach(), CountingDictionary, dict_cham_thoi_gian_do_duoc_tang_theo_so_lan_goi(), elapsed_do_duoc_thoi_gian_dung_cho_suggest() (+8 more)

### Community 190 - "hygiene.rs"
Cohesion: 0.09
Nodes (36): read, ascii_ps1(), BANNED_APIS, collect_md(), contains_set_hook_call(), contains_word(), doc_link_targets(), doc_links_bat_ca_dang_target() (+28 more)

### Community 191 - "IpcServer"
Cohesion: 0.11
Nodes (18): AssistMessageOutcome, assistReport, toggleAssist, violation, IpcServer, IpcServerDelegate, Any, Bool (+10 more)

### Community 192 - "core/tests/invariants.rs"
Cohesion: 0.18
Nodes (17): apply_checked(), backspace_removes_exactly_one_visible_char(), e84_du_lieu_am_tiet_vn_khong_chua_tu_en_ngoai_tu_dien(), engine_never_overflows_result_or_eats_existing_text(), held_key_after_toned_word_keeps_previous_text(), key_event(), Lcg, letters() (+9 more)

### Community 193 - "P3-2 — FCITX5 ADDON (Linux) — Solution chi tiết"
Cohesion: 0.17
Nodes (11): 10. Failure modes, 1. Quyết định & capability, 2. Cấu trúc module, 3. Vòng đời, 4. Key flow, 5. `apply_replace` (đối chiếu `P3-1 §6` — cùng semantics, API khác), 6. Link FFI từ C++ (RL10 — spike `LNX-003`), 7. Tương thích 2 adapter song song (điều phối IBus + Fcitx5) (+3 more)

### Community 194 - "ime_key"
Cohesion: 0.09
Nodes (31): cstr, ime_key(), ime_key_v1, ime_suggest(), ime_suggest_v1, abi_invariants_giu_duoi_input_ngau_nhien(), apply_doc(), assert_sane() (+23 more)

### Community 195 - "windows-tsf/src/assist.rs"
Cohesion: 0.03
Nodes (101): a2_double_space_va_space_after_punct_sua_dung(), a2_lap_tu_xoa_dung_lan_lap(), acceptance_dod_scenarios_1_to_7(), after_trigger(), all_issue_views_giu_du_hon_3_loi_cho_mode2_va_fix_all(), ASSIST_KIND_LOWERCASE_SENTENCE, assist_trigger(), AssistCheck (+93 more)

### Community 196 - ".register"
Cohesion: 0.11
Nodes (18): MacSpellProvider, Bool, Int, MacSpellProviderTests, [0.2.12] — 2026-10-03, [0.2.15] — 2026-10-03, [0.2.26] — 2026-10-05, [0.2.5] — 2026-10-02 (+10 more)

### Community 197 - "imk_abi.rs"
Cohesion: 0.06
Nodes (56): borrow_utf8(), IME_IMK_ABI_VERSION, IME_IMK_ARENA, IME_IMK_AT_CARET, IME_IMK_AT_MARKED_RANGE, IME_IMK_AT_SELECTION, IME_IMK_COMMIT, ime_imk_input_v1 (+48 more)

### Community 198 - "cli/src/main.rs"
Cohesion: 0.10
Nodes (21): abi_sizes(), cmd_check_dlopen(), cmd_config(), cmd_doctor(), cmd_register(), cmd_replay(), cmd_sizes(), cmd_unregister() (+13 more)

### Community 199 - "phrase_repeat.rs"
Cohesion: 0.11
Nodes (9): bat_lap_cum_2_tu_suggestion_only(), check(), check_window(), EN_SKIP_WORDS, IMPERATIVE_AFTER, PHRASE_WHITELIST, Vec, VI_SKIP_WORDS (+1 more)

### Community 200 - "Đóng góp cho TextVN"
Cohesion: 0.11
Nodes (19): 1. Fork & Clone, 2. Tạo branch, 3. Phát triển, 4. Commit message, 5. Pull Request, 6. Phát hành version mới, Cách báo cáo lỗi (Bug Report), Cấu trúc dự án & ownership (+11 more)

### Community 201 - "tsf-min/src/lib.rs"
Cohesion: 0.07
Nodes (39): ITfKeyEventSink, ClassFactory, CLSID_TEXTVN_TIP, CompSink, CompSink_Impl, DllCanUnloadNow(), DllGetClassObject(), EditSession_Impl (+31 more)

### Community 202 - "dict/src/lib.rs"
Cohesion: 0.14
Nodes (25): builder, checksum_phat_hien_hong_data(), data_assist(), dic_dau_cu_chua_hoa_khong_chua_hoa_moi(), dic_dau_moi_chua_hoa_moi_khong_chua_hoa_cu(), dic_for_each_final_tra_tat_ca_tu(), dic_vi_dau_cu_khong_chua_tu_khong_co_trong_file(), dic_vi_dau_cu_phat_hien_tat_ca_tu() (+17 more)

### Community 203 - "Result"
Cohesion: 0.13
Nodes (13): GUID, IEnumTfDisplayAttributeInfo, ITfDisplayAttributeInfo, ITfDisplayAttributeProvider_Impl, Option, Result, TextVNDisplayAttributeInfo_Impl, TextVNDisplayAttributeProvider_Impl (+5 more)

### Community 204 - "macro_text.rs"
Cohesion: 0.13
Nodes (18): blank_comment_and_crlf_lines_are_ignored(), entry(), error_codes_are_distinct_and_messages_non_empty(), escape(), EXPAND_MAX, format(), MacroLineError, MacroLineErrorKind (+10 more)

### Community 205 - "dream_rsi_100.rs"
Cohesion: 0.08
Nodes (57): apply, apply(), apply_with_undo(), expansion(), q(), Option, Vec, ctrl_space() (+49 more)

### Community 206 - "trigger"
Cohesion: 0.13
Nodes (15): $comment, default, type, enabled, suggest, trigger, additionalProperties, $comment (+7 more)

### Community 207 - "P0-5 — Nền tảng thống nhất + chip ARM (Apple M · Windows on ARM · Linux aarch64)"
Cohesion: 0.08
Nodes (23): 10. Rủi ro, 1. Tóm tắt khuyến nghị, 2. Nghiên cứu bộ gõ — mới & nổi tiếng (2026-11-27), 4. "Native từ đầu" — đánh giá theo từng OS, 5.1 Kiến trúc, 5.2 Chất lượng "vượt trội" — gate bắt buộc trước khi thay một khối, 5.3 Thứ tự chuyển (mỗi bước một bản phát hành), 5. Nền tảng thống nhất: `textvn-session` (SF) (+15 more)

### Community 208 - "phrases.rs"
Cohesion: 0.15
Nodes (19): drop_issues_in_protected(), EN_DATA, en_set(), is_fixed_phrase(), is_fixed_phrase_for(), is_lexical(), load(), MAX_PHRASE_WORDS (+11 more)

### Community 209 - "field-detect/src/lib.rs"
Cohesion: 0.14
Nodes (17): K, CachedProbe, current_nonsecure_probe_can_use_capability_and_preset(), linux_and_macos_probes_respect_generation_and_security_gate(), normalize_app_id(), pending_or_secure_context_never_transforms(), probe_cache_expires_and_can_be_invalidated(), ProbeCache (+9 more)

### Community 210 - "e2e_ibus.c"
Cohesion: 0.16
Nodes (32): app_handles(), gboolean, gchar, gpointer, guint, IBusText, check(), clear() (+24 more)

### Community 211 - "e50_unikey_restore.rs"
Cohesion: 0.17
Nodes (21): assert_matrix(), e50_after_restore_skip_further_restore(), e50_daat_space_no_duplicate(), e50_dat_space_no_duplicate(), e50_unikey_task_space_no_restore(), e60_ambiguous_vn_preserved_and_space_complete_guard(), e60_contrast_double_letters_and_triple_undo(), e60_freq3000_unikey_undo_sweep() (+13 more)

### Community 212 - "ipc_server.rs"
Cohesion: 0.07
Nodes (43): filesystem, pipes, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, broadcast_reaches_subscriber(), client_app_id(), ClientSink, connected_client_pid() (+35 more)

### Community 213 - "e2e-assist-p6.ps1"
Cohesion: 0.31
Nodes (6): Get-ViEnabled(), Norm(), NP-Case(), Report(), Set-ViEnabled(), W-Case()

### Community 214 - "e2e_fcitx5.c"
Cohesion: 0.10
Nodes (46): app_handles(), build_config_dir(), check(), clear(), ctrl_current_im(), ctrl_set_current_im(), ctrl_shift_tap(), ctrl_tap() (+38 more)

### Community 215 - "emoji_popup.rs"
Cohesion: 0.05
Nodes (56): common, direct2d, directwrite, dwm, dxgi_format_b8g8r8a8_unorm, emoji, keyboardandmouse, toggle_tray_emoji_popup (+48 more)

### Community 216 - "key_event.rs"
Cohesion: 0.10
Nodes (49): is_modifier_vk(), active_modifiers(), clears_assist_undo(), compartment_i32(), context_blocks_typing(), defer_to_key_down(), deferred_key_disposition(), DeferredKey (+41 more)

### Community 217 - "chords.rs"
Cohesion: 0.08
Nodes (36): all_symbol_chords_three_modes(), altgr_chord(), altgr_layer_full(), armed(), double_leader_literal(), dream_rsi_delta(), dream_rsi_delta_counts(), emit() (+28 more)

### Community 218 - "client.rs"
Cohesion: 0.09
Nodes (21): ASSIST_APP_ID, backoff_doubles_to_cap_and_resets(), ClientEffect, DEFAULT_BACKOFF_INITIAL_MS, DEFAULT_BACKOFF_MAX_MS, GLOBAL_APP_ID, IpcClientState, join_unix() (+13 more)

### Community 219 - "push_key"
Cohesion: 0.12
Nodes (30): base_keys(), fix_uo(), vni_viqr_marks_after_whole_cluster(), Horn, post_fixes(), promote_uo_pair(), push_key(), apply_key() (+22 more)

### Community 223 - "username_identity_1000.rs"
Cohesion: 0.13
Nodes (25): corpus_co_du_lop_hieu_biet(), corpus_files(), corpus_vi_basic_passes(), parse_corpus(), Path, PathBuf, Vec, a1000_builder_deterministic_va_id_stable() (+17 more)

### Community 224 - "r2_171_caps_after_enter_reset.rs"
Cohesion: 0.29
Nodes (14): auto_capitalize_off_is_untouched(), caps_after_enter_survives_toolkit_reset(), caps_after_enter_without_reset_unchanged(), double_reset_after_enter_still_capitalizes(), enter(), enter_then_click_elsewhere_does_not_capitalize(), opts(), other_tails_with_control_chars_still_rejected() (+6 more)

### Community 225 - ".new"
Cohesion: 0.13
Nodes (17): contains_giua_cua_so_khong_reset_bo_dem(), en_us_neu_may_co_thi_provider_hoat_dong(), MacSpellChecker, phuong_thuc_tat_ca_fail_open_khong_panic(), AtomicUsize, Instant, Job, Mutex (+9 more)

### Community 226 - "settings_window.c"
Cohesion: 0.10
Nodes (51): add_escape_closes(), button(), gboolean, GCallback, GFile, GFileMonitor, GFileMonitorEvent, gpointer (+43 more)

### Community 227 - "number_vn.rs"
Cohesion: 0.15
Nodes (11): civil_from_days(), days_since_epoch_now(), DIGITS, find_date_shorthand(), find_number_shorthand(), ngay_thang_nam_din_hinh(), read_triple(), read_vnd_words() (+3 more)

### Community 229 - "foreground.rs"
Cohesion: 0.14
Nodes (17): accessibility, foregroundtracker, HWINEVENTHOOK, pwstr, app_id_from_image_path(), app_of_window(), ForegroundTracker, is_shell_window_class() (+9 more)

### Community 230 - "settings_window_gtk3.c"
Cohesion: 0.10
Nodes (55): add_escape_closes(), button(), gboolean, GCallback, GFile, GFileMonitor, GFileMonitorEvent, gpointer (+47 more)

### Community 231 - "Foundation"
Cohesion: 0.08
Nodes (22): assistKindName(), UInt32, KeyMapMacTests, MarkedStateTests, MarkedStateUnitTests, NativeBoundaryTests, TranscodeSelectionTests, LayoutCache (+14 more)

### Community 232 - "Problem"
Cohesion: 0.09
Nodes (18): MacroRules, Problem, emptyExpansion, emptyTrigger, expansionTooLong, .message, triggerHasSpace, triggerTooLong (+10 more)

### Community 233 - "Path"
Cohesion: 0.14
Nodes (24): append_log_line(), arm_w3_companion_dlls_for_acl_du_4_dll_tren_arm64(), arm_w3_select_inproc_dll_uu_tien_forwarder_tren_arm64(), companion_dlls_for_acl(), fresh_test_dir(), host_is_arm64(), is_arm64_host(), is_arm64_native_machine() (+16 more)

### Community 234 - "EmojiPickerModel"
Cohesion: 0.08
Nodes (27): EmojiCanvasView, .isFlipped, EmojiCategoryTab, EmojiPickerAction, close, commit, redraw, reload (+19 more)

### Community 235 - "P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`"
Cohesion: 0.12
Nodes (16): 1. Công cụ đo: `textvn-cli assist score --corpus <dir>`, 2.1 Vòng 2 (2026-10-21) — đo trên **văn bản thật** (đóng F80), 2.2 Vòng 3 (2026-10-21) — mở rộng luật ngữ pháp + `--strict` / `--per-kind`, 2.3 Vong 4 (2026-10-21) - do tren **1.500 cau that** (Tatoeba) -> 4 loi engine that, 2.4 Tu dien FST (hunspell-vi) — ket qua tra cuy (P4-4 AS-040/AS-041), 2. Kết quả đo (2026-10-21), 3. Văn bản sạch dùng để đo FPR — `corpus/assist/vi_clean_synthetic.ass`, 4.1 Finding vòng 2 (SA/QA) — phát hiện nhờ đo trên văn bản thật (+8 more)

### Community 236 - "Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor"
Cohesion: 0.22
Nodes (9): 0. Nguyên tắc (đã xác minh qua 3 lần phát hành), 0b. Một lần là xong (G23 — chủ repo 2026-11-27: "sửa 1 lần, giải quyết lỗi cho tất cả, không re-work"), A0. `cargo xtask preflight` — BẮT BUỘC exit 0 (thay cho chạy tay từng lệnh), Bằng chứng quy trình chạy thành công, Bộ docs của một bản phát hành — chú thích từng file, Checklist A — máy local (TRƯỚC khi commit/tag), Checklist B — repo/CI (SAU khi local xanh), Quy trình phát hành TextVN — bắt buộc cho mọi agent và contributor (+1 more)

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

### Community 241 - "test_compose.c"
Cohesion: 0.05
Nodes (74): lc_comp, len, text, lc_plan, cursor_left, delete_before, eaten, end (+66 more)

### Community 242 - "en_office.rs"
Cohesion: 0.13
Nodes (23): add(), add_filters_non_word_keys(), base_w(), body_matches_committed_tsv(), build_rows(), is_lowercased_word(), MAX_PRED, max_pred_filter() (+15 more)

### Community 243 - "Báo cáo dựng & kiểm thử — TextVN"
Cohesion: 0.03
Nodes (76): 1. Gói phát hành, 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này), 4. Phạm vi chưa kiểm tự động, 5. Hiệu năng, 6. Tái lập, Báo cáo dựng & kiểm thử — TextVN, Bản 0.2.1 — phát hành pre-release đã xác minh, Bản 0.2.20 — CI xanh lại + validation vòng 3 (harness đúng như Microsoft) (+68 more)

### Community 244 - "Option"
Cohesion: 0.23
Nodes (16): added_language_removable(), is_input_method_value(), languages_added_marker_path(), languages_missing(), languages_without(), parse_languages_marker(), read_languages_added(), record_languages_added() (+8 more)

### Community 245 - "regression_index.rs"
Cohesion: 0.13
Nodes (34): check(), check_ref(), cli(), collect_codes(), collect_tests(), discover(), e88_bootstrap_chi_dem_file_git_tracked(), INDEX (+26 more)

### Community 246 - "Dictionary"
Cohesion: 0.16
Nodes (12): AsRef, Dictionary, EnchantSpellChecker, GLOBAL, load(), MacSpellChecker, Arc, OnceLock (+4 more)

### Community 247 - "ime_conflict_1000.rs"
Cohesion: 0.14
Nodes (22): caps_toggle_step(), e48_classify_origin_and_keyboard_state_truth_table(), keyboard_state_for(), vk_to_unicode(), b300_engine_pass_mapping(), build_plain_matrix(), build_sequences(), c200_inject_shape() (+14 more)

### Community 248 - ".OnTestKeyDown"
Cohesion: 0.25
Nodes (9): ClassFactory_Impl, EditSession, KeySink_Impl, BOOL, IClassFactory_Impl, ITfContext, ITfKeyEventSink_Impl, LPARAM (+1 more)

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
Cohesion: 0.09
Nodes (33): CORRECTIVE_ACTION, collect_issues(), collect_suggestions(), en_us_neu_may_co_thi_provider_hoat_dong(), handle_job(), is_actionable(), Job, MAX_SUGGEST_RESULTS (+25 more)

### Community 254 - "quality_trend.rs"
Cohesion: 0.15
Nodes (36): arrow(), bang_lines(), biz_3_bang_xu_huong_dung_gia_tri_va_huong(), biz_3_baseline_bang_0_hang_nhieu_la_gach(), biz_3_derived_display_khop_cong_thuc_cli(), biz_3_eval_hash_khac_bo_mui_ten_rsi(), biz_3_kpi_thieu_o_ban_cu(), biz_3_kpi_tut_bao_loi_va_exit() (+28 more)

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

### Community 259 - "C. LINUX + SHARED"
Cohesion: 0.18
Nodes (11): C1. [TRUNG — latent] `lc_ipc_recv_frame` mất byte khi EAGAIN giữa frame, C2. [TRUNG] Parser IPC dispatch bằng `strstr` toàn JSON → nhầm nhánh khi, C3. [TRUNG — kiến trúc] IPC Linux là dead path: không tồn tại server, C4. [TRUNG — latent] Override per-app "dính" sang app kế tiếp (ibus, C5. [THẤP] compose.c:92-94 — `cur->len == LC_COMP_MAX` → ghi, C6. [THẤP] utf.c:49-106 — `lc_utf8_to_utf32` không chặn overlong, C7. [THẤP] GTK settings (settings_window.c:483,214-250) — `TvWin` free, C8. [THẤP] `tv_activate_fcitx5` (textvn-common.sh:355-363) thêm TextVN (+3 more)

### Community 260 - "tis-select.swift"
Cohesion: 0.16
Nodes (15): Carbon, CFString, Never, 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061), 2. `ax-driver` (MAC-060) — thiết kế, chưa code, 3. Việc còn lại trên máy thật (cập nhật 2026-10-09), tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6), allSources() (+7 more)

### Community 262 - "text_prediction.rs"
Cohesion: 0.16
Nodes (29): apply_outcome(), engine_with_predict(), key_ctrl_space(), key_for_char(), predict_engine(), press_key(), r2_120_khong_du_doan_qua_dong_moi(), r2_121_ctrl_space_khi_tat_tieng_viet_khong_ghi_cach_ao() (+21 more)

### Community 263 - "4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements)"
Cohesion: 0.14
Nodes (14): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release, Phát hiện CI-02: Đóng gói Inno Setup Silent bỏ qua đăng ký TSF (+6 more)

### Community 264 - "e57_math_sync.rs"
Cohesion: 0.31
Nodes (12): backspace(), buf_str(), e57a_arrow_clears_math_word(), e57a_backspace_syncs_math_word(), e57a_template_still_expands_without_backspace(), e57b_inline_eval_no_delete_overflow(), e57c_telex_suspended_inside_eval_word(), left() (+4 more)

### Community 265 - "e2e-desktop.sh"
Cohesion: 0.13
Nodes (19): fail(), HOME, oracle_cases(), run_cases(), say(), e2e-desktop.sh script, shadow_check(), sway_session() (+11 more)

### Community 266 - "mem-check.sh"
Cohesion: 0.83
Nodes (3): cpu_pct(), rss_mb(), mem-check.sh script

### Community 267 - "soak.sh"
Cohesion: 0.83
Nodes (3): pid_of(), rss_kb(), soak.sh script

### Community 268 - "compose.c"
Cohesion: 0.07
Nodes (62): ime_session, lc_modifier_toggle, armed, ctrl_armed, ctrl_down_us, lc_session_shadow, mismatches, now_override_ms (+54 more)

### Community 270 - "default"
Cohesion: 0.18
Nodes (11): additionalProperties, $comment, default, type, auto_fix, language, max_sentence_chars, mode (+3 more)

### Community 271 - "build-macos.sh"
Cohesion: 0.39
Nodes (7): build_swift(), build-macos.sh script, textvn_uni_archs_of(), textvn_uni_have_both_archs(), textvn_uni_is_macho(), textvn_uni_scan_dir(), package-macos-pkg.sh script

### Community 274 - "toggle.rs"
Cohesion: 0.07
Nodes (28): CTRL_TAP_MAX_MS, is_ctrl_vk(), is_modifier_vk(), is_shift_vk(), is_toggle_chord(), MOD_ALT, MOD_CAPS, MOD_CTRL (+20 more)

### Community 279 - "trace_diff.rs"
Cohesion: 0.06
Nodes (66): APP_DB, appdb(), CI_APPS, collect_preset_ids(), FIELD_ROLES, good(), has_preset_id(), Json (+58 more)

### Community 290 - "compose_abi.rs"
Cohesion: 0.08
Nodes (30): compose, call(), IME_CKEY_ALTGR, IME_CKEY_BACKSPACE, IME_CKEY_CHAR, IME_CKEY_CTRL_SPACE, IME_CKEY_ENTER, IME_CKEY_ESCAPE (+22 more)

### Community 291 - "ConfigWatcher"
Cohesion: 0.18
Nodes (8): ConfigWatcher, TimeInterval, URL, Void, DispatchSource, DispatchSourceFileSystemObject, DispatchWorkItem, Notification

### Community 292 - "fs"
Cohesion: 0.18
Nodes (13): all_cases, btreeset, fs, path, Result, run(), add_curated(), add_splits_from() (+5 more)

### Community 293 - "trace_compose.c"
Cohesion: 0.11
Nodes (56): ime_result_v1, collect_json_files(), emit_trace_cmd(), ends_with(), free_app(), free_case(), hex_val(), is_ctrl_space() (+48 more)

### Community 294 - "command"
Cohesion: 0.09
Nodes (22): find_rc_exe(), main(), Option, PathBuf, find_rc_exe(), main(), Option, PathBuf (+14 more)

### Community 295 - "check_version_sync.rs"
Cohesion: 0.22
Nodes (12): check_site(), extract(), PLIST_KEY, repo_hien_tai_dong_bo(), Option, Result, run(), Site (+4 more)

### Community 296 - "String"
Cohesion: 0.04
Nodes (31): Int, UiSettingsModel, UiSettingSpec, EmojiItem, UInt8, IMKTextTarget, Bool, Int (+23 more)

### Community 297 - "Canvas"
Cohesion: 0.18
Nodes (8): box_blur_1d(), Canvas, downsample2_chinh_xac_alpha_thap(), paste_with_mask_doc_kenh_alpha_va_khong_panic_lech_chieu_cao(), png_roundtrip_markers_and_pixels(), rounded_rect_coverage(), rounded_rect_va_rasterizer_khong_panic_va_giu_hang_dau(), Option

### Community 298 - "P4-6 — Handoff AS-021 (FFI) + AS-022 (trigger): bản đồ & recipe để thi công"
Cohesion: 0.13
Nodes (14): 0. Chuỗi phụ thuộc (đã xong / sắp làm), 1.1 API công khai (đề xuất, khớp D23/D27/D29), 1.2 Quy tắc bắt buộc, 1.3 Chỗ sẽ sửa (đã đọc trong repo), 1.4 Kiểm chứng, 1. AS-021 — FFI `ime_assist_*` (C-ABI), 2.1 Luồng, 2.2 Quy tắc hành vi (D11 + D12) (+6 more)

### Community 299 - "properties"
Cohesion: 0.15
Nodes (13): properties, $comment, default, type, hide_on_auto_fix, show_on_comma, show_on_sentence_end, $comment (+5 more)

### Community 300 - "tone_inject_one"
Cohesion: 0.23
Nodes (11): &T, is_combining_mark(), Rng, rng_deterministic_and_sample_distinct(), Option, strip_tone(), tone_inject_changes_one_tone(), tone_inject_one() (+3 more)

### Community 301 - "ComposePolicy"
Cohesion: 0.10
Nodes (13): BoundaryPolicy, ComposePolicy, .LINUX, .TSF, Composer, CompositionPlan, EngineStep, RawEngineResult (+5 more)

### Community 302 - "emoji_picker.rs"
Cohesion: 0.10
Nodes (25): CATEGORY_TABS, CELL_H, CELL_W, CLASS_NAME, EmojiItem, FOOTER_HEIGHT, GRID_COLS, GRID_TOP (+17 more)

### Community 303 - "tip.rs"
Cohesion: 0.12
Nodes (19): ASSIST_KEYS, current_exe_name(), Default, GUID, ITfDisplayAttributeInfo, ITfKeystrokeMgr, ObjGuard, Option (+11 more)

### Community 304 - "3. Nhật ký finding"
Cohesion: 0.12
Nodes (15): 0. Kết luận, 1. Phương pháp, 2. Bằng chứng CI, 3.1 Microsoft Store / MSIX, 3.2 Bảo mật Windows, 3.3 QA Windows (tray, CLI, bộ cài), 3.5 Tiếng Việt / engine, 3.6 CI, ký số, phát hành (+7 more)

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
Cohesion: 0.09
Nodes (55): class_name(), edit_selection(), edit_text_before_caret(), EM_GETSEL, EM_GETSELTEXT, EM_REPLACESEL, EM_SETSEL, give_up() (+47 more)

### Community 309 - "en_office_phrases_10k.rs"
Cohesion: 0.13
Nodes (26): bang_van_phong_phu_5_nhom_va_bilingual(), predict_next_phrase_candidates(), predict_prefix_candidates(), test_predict_next_phrase_english(), test_predict_next_phrase_vietnamese(), test_predict_prefix_compounds_vietnamese(), test_predict_prefix_english(), che_do_en_mo_rong_cau_van_phong() (+18 more)

### Community 310 - "windows-tsf/src/replay.rs"
Cohesion: 0.14
Nodes (27): CLASS_NAME, DeferredReplay, deliver(), HOPS, MODIFIER_POLL_MAX, MODIFIER_POLL_MS, modifiers_down(), module_instance() (+19 more)

### Community 311 - "Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon)"
Cohesion: 0.29
Nodes (7): Bản 0.2.3 — icon đủ 3 nền tảng + audit sâu 2 vòng (code/CI/icon), Icon — trả lời câu hỏi "Linux/macOS đã có icon như Windows chưa?", Kiểm chứng chạy trên host Windows (2026-10-01, sau toàn bộ thay đổi), Phát hành — v0.2.3 đã publish (release candidate, chưa ký số), Quy trình phát hành mới, Sửa theo audit vòng 1 — GitHub Actions (5 major + minors), Sửa theo audit vòng 1 — đăng ký TSF (3 major + 7 minor)

### Community 312 - "sentence_grammar.rs"
Cohesion: 0.15
Nodes (13): check_comma_after_conjunction(), check_en(), check_en_agreement(), check_vi(), EnCorrelativeClause, merge_contractions(), preserve_case(), push_delete_issue() (+5 more)

### Community 313 - "quality_score.rs"
Cohesion: 0.17
Nodes (26): btreemap, layout_dialog_controls(), biz_1_ghi_va_doc_lai_tep_kpi(), biz_1_kpi_tut_thi_do_theo_huong_tot(), cli(), collect(), derived_display(), direction() (+18 more)

### Community 315 - "wide"
Cohesion: 0.19
Nodes (16): ctrl_shift_taken_by_windows(), delete_reg_value(), enum_subkeys(), grant_appcontainer_read(), hkey_users_relative_paths_reach_the_loaded_hive(), reg_read_named(), register_with_tsf_api(), remove_ctf_assembly_defaults() (+8 more)

### Community 316 - "4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT)"
Cohesion: 0.13
Nodes (15): 4.2. Module macOS (IMK, CGEventTap, AppKit Menu Bar, Packaging & Entitlements), 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), 4.4. Core Engine, FFI ABI, Restore English & Data, 4.5. CI/CD, Supply Chain, VirusTotal & Microsoft Store Validation, 4. CHI TIẾT KIỂM TOÁN TỪNG DÒNG MÃ (MODULE-BY-MODULE AUDIT), Phát hiện C-01: Hiệu năng quét từ điển `restore_en.rs`, Phát hiện C-02: Dung lượng tập từ vựng còn quá nhỏ, Phát hiện CI-01: Script ký số chưa được gắn vào quy trình Release (+7 more)

### Community 317 - "emoji_catalog.rs"
Cohesion: 0.09
Nodes (26): bilingual_diacritic_insensitive_search(), CAT_ACTIVITIES, CAT_FOOD, CAT_GESTURES, CAT_NATURE, CAT_RECENT, CAT_SMILEYS, CAT_SYMBOLS (+18 more)

### Community 318 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.15
Nodes (13): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5.4. MA TRẬN ĐỐI CHIẾU TOÀN DIỆN 3 NỀN TẢNG (WINDOWS · MACOS · LINUX), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG (+5 more)

### Community 319 - "AXSnapshot"
Cohesion: 0.04
Nodes (49): ConflictScan, AXSnapshot, Entry, FieldContext, FieldDetect, FieldRole, FieldRules, Bool (+41 more)

### Community 320 - "settings_model.c"
Cohesion: 0.11
Nodes (48): tv_paths, edit_bool(), edit_reset(), edit_str(), edit_u32(), get_bool(), get_index(), get_u32() (+40 more)

### Community 321 - "src/style_merge.rs"
Cohesion: 0.16
Nodes (17): MergeOutcome, merge_at(), merge_from_json(), PENDING_RSI, Arc, Duration, Instant, Mutex (+9 more)

### Community 322 - "ConfigStore"
Cohesion: 0.08
Nodes (28): ConfigStore, .config, Void, ActiveAlert, about, autostartFailed, clearLearned, .id (+20 more)

### Community 323 - "dual_mode_matrix.rs"
Cohesion: 0.18
Nodes (28): boundary_expect(), BS, CASES, count_case(), en_common_corpus(), en_freq_corpus(), ESC, gate_total_case_count_at_least_1000() (+20 more)

### Community 324 - "macro.rs"
Cohesion: 0.15
Nodes (15): Emoji, emoji_glyph_is_multi_char(), apply_macro_case(), def(), emoji_tail_matches(), eq_ignore_case(), find(), macro_tail_matches() (+7 more)

### Community 325 - "config/src/sandbox.rs"
Cohesion: 0.17
Nodes (14): parse_style_rsi_delta(), full_config(), Option, rsi_only_drops_words_and_caps_counters(), sandbox_config_json(), sandbox_copy_keeps_typing_options_and_learning_switch(), sandbox_copy_never_contains_user_content(), SANDBOX_KEEP_KEYS (+6 more)

### Community 326 - "BÁO CÁO KIỂM TOÁN CHUYÊN SÂU & LỘ TRÌNH ĐẠT CHUẨN PRODUCTION VÀ KÝ SỐ TOÀN DIỆN (v0.2.19)"
Cohesion: 0.14
Nodes (12): 1.1. Hiện trạng Git & Trạng thái nhánh, 1.2. Nhận định từ Chuyên gia Kiểm toán, 1. TỔNG QUAN TÌNH TRẠNG REPOSITORY & CÁC COMMIT MỚI NHẤT, 5.1. Ký số Windows (Authenticode qua SignPath Foundation / Azure Trusted Signing), 5.2. Ký số & Notarization macOS (Apple Developer ID Application & Installer), 5.3. Ký số & Phân phối Linux (GPG, PPA, COPR, AUR, Flatpak), 5. LỘ TRÌNH KÝ SỐ CHÍNH THỨC & SẴN SÀNG CHO MỌI NGƯỜI DÙNG, 6. PHƯƠNG ÁN XỬ LÝ TRIỆT ĐỂ (STEP-BY-STEP REMEDIATION BLUEPRINT) (+4 more)

### Community 328 - "lc_key"
Cohesion: 0.10
Nodes (25): lc_key, ch, kind, lc_key_classify(), lc_key_classify_with_mods(), lc_key_engine_ch(), lc_key_to_ime(), test_classify() (+17 more)

### Community 329 - "viqr.rs"
Cohesion: 0.13
Nodes (12): apply_key, fold(), fold_impl(), fold_steps(), marker_mark_index(), marker_to_tone(), n(), DiacriticStyle (+4 more)

### Community 330 - "AssistCandidateWord"
Cohesion: 0.11
Nodes (23): lc_selection_slice(), utf8_char_len(), applyAssistCandidate(), AssistCandidateWord, deleteGlyphs_, originalTail_, replacement_, st_ (+15 more)

### Community 331 - "fix.rs"
Cohesion: 0.11
Nodes (16): apply_auto_fixes(), chi_giu_loi_can_sua(), Fix, plan_sentence_fix(), Option, Vec, select_non_overlapping_fixes(), SentenceFixPlan (+8 more)

### Community 332 - "Ref"
Cohesion: 0.18
Nodes (13): F, IEnumTfDisplayAttributeInfo, ITfContext, ITfDisplayAttributeProvider_Impl, ITfTextInputProcessor_Impl, ITfThreadMgr, Result, run_guarded() (+5 more)

### Community 333 - "diacritic_typo.rs"
Cohesion: 0.16
Nodes (15): base_group(), bat_ca_cau_corpus_that_moi_nhat(), bat_sai_dau(), check(), giu_hoa_thuong(), has_diacritic(), khong_beu_tu_hop_le_tung_o_bang_typos(), MAX_UNKNOWN_TOKENS (+7 more)

### Community 335 - "linux-settings/src/main.c"
Cohesion: 0.24
Nodes (7): gpointer, GtkApplication, on_action_diagnose(), on_app_activate(), GSimpleAction, gtk, GVariant

### Community 336 - "macSpellContainsCallback"
Cohesion: 0.44
Nodes (8): macSpellContainsCallback(), macSpellSuggestCallback(), CChar, Int32, UInt32, UnsafeMutableRawPointer, UnsafePointer, UnsafeMutablePointer

### Community 337 - "SessionShadow"
Cohesion: 0.07
Nodes (29): KeyMapMacGenerated, SessionShadow, .isActive, .mods, Bool, OpaquePointer, UInt32, UInt64 (+21 more)

### Community 338 - "3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft"
Cohesion: 0.18
Nodes (11): 3.4.1. Giải phẫu bản chất chuỗi lỗi (Dependency Chain) trong bộ thẩm định tự động của Microsoft, 3.4.2. Bốn điểm nghẽn cốt lõi (Root Causes) khiến TextVN v0.2.20 tiếp tục bị từ chối, 3.4.3. Thiết kế bài test trên CI mô phỏng đúng 100% kỳ vọng của Microsoft Store, 3.4.4. Kế hoạch hành động dứt điểm để vượt qua Store Validation, 3.4. Điều tra chuyên sâu: Giải mã 3 lỗi từ chối của Microsoft Store (Silent Install, Add/Remove Programs, Bundleware) & Thiết kế bài test CI chuẩn xác theo kỳ vọng của Microsoft, Gốc rễ 1: Sự ngộ nhận tai hại về HKCU (Per-User) vs HKLM (Per-Machine) trong môi trường tự động của Store, Gốc rễ 2: Lệch chuỗi định danh tuyệt đối (Exact String Mismatch) giữa Partner Center và Registry, Gốc rễ 3: Điểm mù của CI Test Harness hiện tại (`validate-store-package.ps1` & `simulate-store-validation.ps1`) (+3 more)

### Community 339 - "r2_173_prediction_chain.rs"
Cohesion: 0.43
Nodes (16): chain_works_for_formal_letter_phrases(), ctrl_space(), ctrl_space_always_cycles_after_accept(), dream_rsi_counts_each_chained_accept_once(), engine(), ghost_is_exactly_what_tab_inserts_across_chains(), greeting_and_office_contexts_chain(), key() (+8 more)

### Community 340 - "EngineOptions"
Cohesion: 0.11
Nodes (41): Context, EngineOptions, Default, DiacriticStyle, MacroTrigger, Method, OutputCharset, boundary_expect() (+33 more)

### Community 341 - "trace.rs"
Cohesion: 0.18
Nodes (10): enabled(), event(), File, Mutex, Option, sink(), Arguments, sync (+2 more)

### Community 342 - "default"
Cohesion: 0.13
Nodes (15): corrections, decided, enabled, max_total_bytes, rsi, words, style_profile, accepts (+7 more)

### Community 343 - "rsi"
Cohesion: 0.11
Nodes (18): minimum, type, minimum, type, accepts, chars_saved, keys_typed, minimum (+10 more)

### Community 344 - "Option"
Cohesion: 0.17
Nodes (16): MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, current_package_family_name(), current_package_full_name(), message_box(), open_apps_settings(), package_family_full_names(), read_package_string() (+8 more)

### Community 345 - "P0-6 — Solution spec: `textvn-session` (một nghiệp vụ cho 3 OS) + build ARM"
Cohesion: 0.12
Nodes (17): 0. Thuật ngữ, 10. An toàn & riêng tư, 11. Việc chưa chốt (ghi ADR khi chốt), 1. Mục tiêu đo được, 2. Kiến trúc, 3. Mô hình dữ liệu (Rust), 4. C ABI (`ffi/`, header `ffi/include/textvn_ffi.h`), 5b. UI dùng chung nguyên lý (G22) (+9 more)

### Community 346 - "readme_numbers.rs"
Cohesion: 0.42
Nodes (8): comma(), count(), dot(), group(), readme_corpora_numbers_match_data_files(), readme_has_no_stale_dictionary_claims(), root(), Path

### Community 347 - "TextVN — Bộ gõ tiếng Việt thế hệ mới cho Windows, macOS và Linux"
Cohesion: 0.10
Nodes (21): 0. Universal Scientific, Engineering & Math Engine (mới ở 0.2.83 — crate `textvn-math`), 1. Công thái học Dream RSI & Kiểm chuẩn Hiệu năng (P10 / G19), 1. Kiến trúc In-Process thuần khiết & An toàn bộ nhớ tuyệt đối, 2. Hệ thống Dự đoán Ngữ cảnh Song ngữ & Công thái học Dream RSI (P10), 2. Mô hình Dự đoán Ngữ cảnh P6 & Hồ sơ Phong cách P7 (Corpora 53.952 mục), 3. Kiểu Gõ & Biến Đổi Âm Vị Học Chuẩn Mực, 3. Trợ lý Ngữ pháp & Ngữ cảnh On-Device (Grammarly-Grade Offline), 4. Di sản Chuẩn UniKey 20 Năm & Tiện ích Văn phòng Hiện đại (+13 more)

### Community 348 - "sandbox_mirror.rs"
Cohesion: 0.29
Nodes (15): authorization, apply_sandbox_dacl(), ensure_dir(), io_err(), mirror_dir(), mirror_dir_dacl_is_protected_and_grants_sandbox_read_only(), mirror_writes_clean_config_and_state(), Error (+7 more)

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

### Community 355 - "diacritic_style.rs"
Cohesion: 0.27
Nodes (9): DiacriticStyle, entry(), has_diacritic(), pick_tone_target(), Option, target(), vowel_span(), is_vowel (+1 more)

### Community 357 - "3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM""
Cohesion: 0.29
Nodes (7): 1. Giải mã toàn diện lịch sử 10 vòng thử nghiệm thất bại trên Microsoft Store, 2. Bằng chứng thép từ Vòng 9: Sự thất bại của bản Per-user 0.2.23 chứng minh dứt điểm giả thuyết "Validator chỉ đọc HKLM", 3.4.6. Điều tra chuyên sâu với Siêu dữ liệu Định danh đã xác thực: Product Name = "TextVN" & Publisher Display Name = "LinhBH.CoM", 3. Đánh giá kỹ thuật về Bản cài đặt Máy (Machine-Install Variant), 4. Kế hoạch hành động dứt điểm theo thứ tự ưu tiên, Bước 1: Nộp bản Machine `TextVN-setup-0.2.23-windows-x64-machine.exe`, Bước 2: Chuyển sang MSIX Full-Trust nếu Bản Máy bị chặn UAC

### Community 358 - "T6 — Test & hardening (tuần 15–18)"
Cohesion: 0.25
Nodes (8): LNX-060 · AT-SPI driver `tools/linux/atspi-driver` (L) — `P3-6 §4`, LNX-061 · Targets JSON 12 app (cùng format 3 OS) (M), LNX-062 · Perf bench + `perf/baseline-linux.json` (M) — `P3-6 §5`, LNX-063 · Soak 24h script × 2 (X11 + Wayland VM) (M), LNX-064 · CI jobs linux + dist matrix (M) — `P3-6 §8`, LNX-065 · Fuzz + ASan/valgrind job (S), LNX-066 · Security + license checklist Phần 3 (M) — Handbook §8 + audit x11 (P3-3 §7), T6 — Test & hardening (tuần 15–18)

### Community 359 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 360 - "undo.rs"
Cohesion: 0.28
Nodes (13): chars(), is_nucleus(), mark_cluster(), mark_horn(), mark_horn_pair_and_single(), mark_vowel(), mark_vowel_applies_and_undoes(), Marked (+5 more)

### Community 361 - "EmojiPicker"
Cohesion: 0.16
Nodes (8): EmojiPicker, PickerOutcome, Default, EmojiEntry, Option, Vec, sf7_mui_ten_luoi_va_danh_sach(), sf7_xuong_ke_o_muc_cuoi_va_cuon_theo_chon()

### Community 362 - "3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow)"
Cohesion: 0.22
Nodes (8): 1. Tổng quan kiến trúc, 2.1 Không đóng gói tĩnh (No Static Linking), 2.2 Nguồn gốc Upstream, 2. Tuân thủ Giấy phép LGPL (Linux — enchant-2 & libspelling), 3.2 Phạm vi tiến trình (Process Boundaries), 3.3 Quyền kiểm soát & Kill-Switch, 3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow), Tuân thủ Giấy phép & Riêng tư — Các Nhà cung cấp Chính tả Hệ điều hành (OS Spellcheck Providers)

### Community 363 - "Added"
Cohesion: 0.33
Nodes (6): [0.1.0] — 2026-09-27, Added, Architecture, Core, Developer Tools, Windows Platform

### Community 364 - "MacroEntry"
Cohesion: 0.16
Nodes (8): LoadError, corrupt, missing, MacroEntry, .id, Result, URL, .body

### Community 365 - "inflate_raw_bounded"
Cohesion: 0.23
Nodes (8): BitReader<'a>, fixed_tables(), HuffTable, inflate_raw(), inflate_raw_bounded(), Result, Self, zlib_decompress()

### Community 366 - "English"
Cohesion: 0.09
Nodes (21): 10. Performance budgets (measured, enforced by benchmarks), 1. Bật / tắt (Enable / Disable), 1. Enable / Disable, 2. Layer A — Leader chords (`` ` ``), 2. Layer A — Leader chords (phím leader `` ` ``), 3. Layer B — AltGr (opt-in), 3. Layer B — AltGr (tùy chọn), 4. Alias tiếng Việt (gõ `\tên` + Space) (+13 more)

### Community 367 - "Result"
Cohesion: 0.20
Nodes (21): check_pe_imports_cli(), find_eocd(), is_crt_runtime_dll(), is_forbidden_eager_graphics_dll(), machine_code(), manifest_mojibake(), map_version(), parse_zip64_extra() (+13 more)

### Community 368 - "e57_word_sync_fca.rs"
Cohesion: 0.31
Nodes (10): backspace_pops_word_buf(), backspace_resets_armed_chord(), caret_moved_clears_word(), engine_sticky(), eval_span_keeps_boundary_chars(), failed_eval_does_not_stick(), inline_eval_delete_excludes_closing_eq(), word_is_command_includes_inline_eval() (+2 more)

### Community 370 - "MathOutputMode"
Cohesion: 0.14
Nodes (14): AppCategory, AppFormatKind, AppProfile, best_mode_for_app(), dream_rsi_formula(), dream_rsi_score(), is_protected_shortcut(), resolve_app() (+6 more)

### Community 371 - "P4-11 — ĐO TÀI NGUYÊN ASSIST (F61) — TIP THẬT"
Cohesion: 0.20
Nodes (9): 1. Phương pháp, 2. Môi trường đo, 3.1 RSS ảnh DLL (phần **quy được** cho DLL), 3.2 Toàn tiến trình (giới hạn trên — **không** quy hết cho DLL), 3. Kết quả (2026-10-22), 4. So với budget (`P4-0-MASTER-PLAN.md` D25 · P4-0 §"Perf"), 5. Việc tiếp theo, 6. Dọn dẹp sau đo (đã làm) (+1 more)

### Community 373 - "Strategy"
Cohesion: 0.14
Nodes (19): default_for_field, downgrade(), downgrade_without_cap(), inp(), resolve(), ResolveInput, Option, secure_field_role_passthrough() (+11 more)

### Community 374 - "DictError"
Cohesion: 0.22
Nodes (7): DictError, Display, Error, Formatter, From, Result, Self

### Community 376 - "P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist)"
Cohesion: 0.33
Nodes (5): Chi tiết P4-1 (đã xong), P4-TASKS — Bảng task Phần 4 "Trợ lý ngữ pháp" (Assist), Recipe P4-2a — refactor buffer trong `core` (đọc trước khi code), Task mở (chi tiết acceptance & phân công Agent), Trạng thái tổng quan

### Community 377 - "Phân phối Linux — kênh, store và auto-update"
Cohesion: 0.10
Nodes (19): 0. Nguyên tắc (khác gì PPA), 1. Lộ trình ba bước, 2. Việc upstream đã chuẩn bị (bằng chứng cho maintainer), Bước A — RFP (nhờ cộng đồng đóng gói) hoặc ITP (tự đóng gói rồi xin sponsor), Bước B — mentors.debian.net upload (kèm ITP), Bước C — Liên hệ Debian Input Method Team (nhanh nhất), Nộp TextVN vào Debian / Ubuntu chính thức (RFP → ITP → Sponsor), 1. Ràng buộc quyết định kênh: TextVN là IME hệ thống (+11 more)

### Community 378 - "textvn-portable.sh script"
Cohesion: 0.21
Nodes (24): tv_err(), tv_fcitx5_is_kwin_wayland(), tv_ibus_needs_live_register(), tv_preferred_framework(), tv_print_report_hint(), tv_process_environment(), tv_restart_fcitx5(), tv_running() (+16 more)

### Community 380 - "lc_config_state"
Cohesion: 0.11
Nodes (24): lc_config_state, allow_macro_when_vi_off, assist_auto_fix, assist_mode, auto_capitalize, loaded, math_enabled, mtime_ns (+16 more)

### Community 381 - "P3-8 — Đối chiếu lỗi của bộ gõ khác với TextVN Linux (2026-11-26/27)"
Cohesion: 0.13
Nodes (14): 1. Khác biệt kiến trúc quyết định phần lớn kết luận, 2. Lỗi tương tự CÓ ở TextVN — đã sửa trong 0.2.92, 4. Khác biệt CHỦ ĐÍCH (chính sách, không sửa), 5. Giới hạn đã biết của nền tảng (ghi vào `docs/TROUBLESHOOTING.md` §4), 6. Cách tái hiện oracle (G4), 7.1 Lỗi tương tự CÓ ở TextVN — đã sửa trong 0.2.92, 7.2 KHÔNG xảy ra ở TextVN (đã kiểm bằng replay / mã), 7.3 Khác biệt CHỦ ĐÍCH (chính sách, không sửa) (+6 more)

### Community 384 - "P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19)"
Cohesion: 0.20
Nodes (9): 1. Dream RSI là gì, 2. Các accept-point + công thức `chars_saved`, 3. Luật `keys_typed` (G19b), 4. Drain + persist, 5. Gate + bench (G19c — KSR-oracle từ 0.2.96), 6. Audit view (G19d), Bẫy đã ghi nhận (đừng lặp), P10 — Dream RSI: đo tiết kiệm phím của user nhờ prediction (vòng 58, G19) (+1 more)

### Community 385 - "P4-3 — SPIKES chặn (S1–S4): kết luận trước khi làm P4-3/P4-4"
Cohesion: 0.20
Nodes (9): 0. Tóm tắt kết luận, 1. S1 — TSF candidate list sau commit, 2. S2 — Đọc vị trí caret (precondition D12), 3. S3 — Nguồn từ điển tiếng Việt, 4. S4a — Chi phí per-process (đo thật, Windows x64), 5. S4b — `harper-core` cho tiếng Anh (v2, không chặn v1), 6. Quyết định mới từ spike, 7. Việc còn lại sau 4 spike (+1 more)

### Community 386 - "Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại"
Cohesion: 0.29
Nodes (6): A. PowerShell & C# interop, C. UIA & ứng dụng test, Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại, D. Build & DLL, E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec), F. Quy trình

### Community 387 - "Diagnostics"
Cohesion: 0.14
Nodes (9): Diagnostics, .crashCount, Date, DispatchSourceTimer, Int, TimeInterval, UInt64, Code Review Round 3 — Đúng & Đủ (Rà soát chi tiết từng dòng Carbon, IMK XPC, Protocol Invariants) (+1 more)

### Community 388 - "rules_mac.rs"
Cohesion: 0.16
Nodes (14): probe_focused_element(), FieldContext, Option, SecurityState, AxSnapshot, classify(), first_hint(), is_terminal_hint() (+6 more)

### Community 389 - "verify_full"
Cohesion: 0.16
Nodes (22): asset_size(), decode_entities(), fake_msix_archive(), find(), find_tag_end(), is_valid_xml_char(), mini_manifest(), MsixArchive (+14 more)

### Community 390 - "P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)"
Cohesion: 0.15
Nodes (12): 10. Vòng 46 (tiếp) — Pass E2 đối kháng + quét O(n²) toàn luật (2026-10-31), 11. Vòng 49-50 — MobaXterm 3-pass audit + panel che vùng gõ (2026-11-03), 1. Bản đồ đường tiếng Anh (đã trace từng nhánh), 2. Root cause đã fix (vòng 42), 3. Đề xuất (cần quyết định PO — không tự ý đổi), 4. Kiểm chứng, 5. Pass EN-1 (agent audit độc lập) — finding + fix (vòng 42), 6. Pass EN-2 (đối kháng 2 commit vòng 42) — finding + fix (vòng 42, 2026-10-31) (+4 more)

### Community 395 - "ime_key"
Cohesion: 0.22
Nodes (9): ime_key, abi_version, ch, is_injected, is_repeat, key_down, mods, _reserved (+1 more)

### Community 396 - "ViState"
Cohesion: 0.08
Nodes (17): IpcClient, Bool, ViState, ViStateStore, AssistIpcTests, EnableGatingTests, Bản 0.2.69 — audit toàn bộ tính năng mới, đồng bộ 3 OS (R2-140…R2-153, 2026-11-08), Bất biến mỗi target assert (không chỉ "không panic") (+9 more)

### Community 397 - "Rà soát backlog theo tài liệu kế hoạch — 2026-11-08"
Cohesion: 0.29
Nodes (6): 1. Đã xử lý trong lượt rà soát này, 2. Cần chủ repo / bên ngoài (agent không tự làm được), 3. Cần người / thiết bị thật, 4. Rủi ro/hoãn đã ghi (không đổi, chờ điều kiện), 5. Tính năng mới đáng làm tiếp (agent làm được) — theo thứ tự đề xuất, Rà soát backlog theo tài liệu kế hoạch — 2026-11-08

### Community 398 - "eval.rs"
Cohesion: 0.14
Nodes (12): apply_func(), apply_op(), apply_unary(), eval(), eval_delimited(), MAX_DEPTH, MAX_TOKENS, precedence() (+4 more)

### Community 399 - "set_reg_value"
Cohesion: 0.25
Nodes (8): broadcast_international(), ensure_modern_language_list(), RegValue, remove_modern_language_list(), runonce_cleanup_command(), runonce_cleanup_command_respects_260_chars(), schedule_delete(), set_reg_value()

### Community 400 - "PLAN — Bộ gõ tiếng Việt mã nguồn mở thế hệ mới (Win · macOS · Linux)"
Cohesion: 0.04
Nodes (46): 0. Executive Summary, 1.1 Bảng tổng hợp, 1.2 Kế thừa & from-scratch — quyết định rõ ràng, 1.3 Ma trận bug kinh niên → kế hoạch xử lý (Đây là "bài toán thật" của dự án), 1. Phân tích 7 nguồn tham chiếu (Audit of sources), 2.1 Tầm nhìn & USP, 2.2 Personas & Jobs-to-be-done, 2.3 Backlog ưu tiên (MoSCoW) (+38 more)

### Community 401 - "worker_main"
Cohesion: 0.18
Nodes (13): Job, Pool, raw_pool_pop(), raw_pool_push(), c_void, Drop, Receiver, SyncSender (+5 more)

### Community 402 - "icons.rs"
Cohesion: 0.29
Nodes (6): HICON, IDI_ICON_E, IDI_ICON_T, IDI_ICON_V, load_app_icon(), HINSTANCE

### Community 403 - "display"
Cohesion: 0.25
Nodes (8): idle_ms, show_on_comma, show_on_sentence_end, additionalProperties, $comment, default, type, display

### Community 404 - "classifier.rs"
Cohesion: 0.15
Nodes (10): accepts_latex_frac(), accepts_mathml(), accepts_unicode_math(), classify(), count_occurrences(), is_currency_span(), is_hard_reject(), is_unicode_math() (+2 more)

### Community 405 - "test_assist_result.c"
Cohesion: 0.22
Nodes (20): jb_cstr(), jb_json_string(), jb_raw(), jb_u32(), lc_assist_filter_auto_capitalize(), lc_assist_kind_name(), lc_assist_report_json(), ime_instance (+12 more)

### Community 406 - "HWND"
Cohesion: 0.11
Nodes (58): HGDIOBJ, add_button_tooltips(), apply_dpi_change(), center_on_work_area(), close_macro_editor(), close_word_list_editor(), copy_text_to_clipboard(), create_and_show_window() (+50 more)

### Community 407 - "TextVNAppTests"
Cohesion: 0.05
Nodes (20): Data, TextVNAppTests, Còn mở (2026-10-09), IMPLEMENTATION STATUS — adapter macOS (Farch-4), Tiếp theo (tuần tự), Trạng thái (2026-10-09), Đã triển khai (bảng thành phần), Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging) (+12 more)

### Community 408 - "super"
Cohesion: 0.13
Nodes (15): che_email(), che_url(), expand_non_space(), expand_word(), is_word_char(), masked_ranges(), masked_ranges_chars(), Vec (+7 more)

### Community 409 - "predict_impl"
Cohesion: 0.18
Nodes (16): build_heavy_index(), compare_prefix_candidates(), is_ext_of(), loop2b_prefix_index_footprint_budget(), mode_index(), ModeIndex, PoolMode, predict_impl() (+8 more)

### Community 410 - "session/tests/invariants.rs"
Cohesion: 0.09
Nodes (21): IntoIter, IntoIterator, AssistItem, Commands, Vec, SessionCommand, APP_IDS, BEFORE_TEXTS (+13 more)

### Community 411 - "CachedFormulaStore"
Cohesion: 0.19
Nodes (11): CachedFormula, CachedFormulaStore, clear_works(), ClipboardWatcher, latest_wins(), MAX_PAYLOAD, Default, Mutex (+3 more)

### Community 413 - "os_spell.rs"
Cohesion: 0.08
Nodes (39): BREAKER_COOLDOWN_MS, breaker_trip_va_phuc_hoi(), CALL_TIMEOUT_MS, ComApartment, create_checkers(), EN_LANG_TAGS, first_error_start(), handle_job() (+31 more)

### Community 414 - "complete_word_candidates"
Cohesion: 0.18
Nodes (21): complete_word(), complete_word_candidates(), en_common_lines(), en_common_set(), en_common_sorted(), en_freq_lines(), en_freq_set(), en_freq_sorted() (+13 more)

### Community 415 - "Option"
Cohesion: 0.22
Nodes (12): guard_decision(), GuardAction, is_valid_pfn(), parse_install_args(), parse_relay_args(), parse_version_quad(), prunable_version_dir(), Option (+4 more)

### Community 416 - "src/emoji.rs"
Cohesion: 0.17
Nodes (15): CATEGORY_COUNT, opened(), PickerKey, QUERY_MAX_CHARS, QUICK_PICK_MAX, sf7_bo_cuc_suy_bien_khong_chia_cho_0(), sf7_cuon_chuot_kep_bien(), sf7_dong_thi_khong_xu_ly_phim() (+7 more)

### Community 417 - "p11_math_toggle.rs"
Cohesion: 0.35
Nodes (11): math_default_off_normal_typing_unchanged(), math_enabled_passthrough_counts_keys_typed_once(), math_span_suspends_telex_and_counts_once(), math_sticky_boundary_expands_alias(), math_sticky_command_suspends_telex(), math_sticky_inline_eval(), math_toggle_off_resets_armed_chord(), opts_with_math() (+3 more)

### Community 418 - "T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006"
Cohesion: 0.29
Nodes (7): LNX-020 · Addon skeleton + lifecycle (M) — `P3-2 §3`, LNX-021 · keyEvent → PASS (M) — `P3-2 §4`, LNX-022 · Preedit + commit + word boundary (L) — `P3-2 §5`, LNX-023 · SelectionReplace + BackspaceType/surrounding (M) — `P3-2 §5`, LNX-024 · focus/commit-before-hide (M) — `P3-2 §3`, LNX-025 · Framework switch + chống đôi (M) — `P3-2 §7`, T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006

### Community 419 - "per_app"
Cohesion: 0.33
Nodes (6): type, additionalProperties, $comment, default, type, per_app

### Community 420 - "bench/src/main.rs"
Cohesion: 0.09
Nodes (19): black_box, DEV_EN, DEV_VI, HELDOUT_EN, HELDOUT_VI, host_target_triple(), len_bucket(), MISS_LEN_BUCKETS (+11 more)

### Community 421 - "abnormal.rs"
Cohesion: 0.02
Nodes (19): check_sentence(), crlf_va_tab_khong_panic(), EmptyDict, ky_tu_nul_va_ky_tu_dieu_khien(), ky_tu_zero_width_va_combining(), Dictionary, Vec, script_la_va_huong_doc_khac() (+11 more)

### Community 422 - "ime_assist_issue"
Cohesion: 0.22
Nodes (9): ime_assist_issue, auto_fixable, confidence, end, kind, message, start, suggest (+1 more)

### Community 423 - "trigger_keys"
Cohesion: 0.33
Nodes (6): enum, trigger_keys, $comment, default, items, type

### Community 424 - "core/src/lib.rs"
Cohesion: 0.06
Nodes (86): Action, ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, auto_capitalize_after_dot_and_enter(), auto_capitalize_needs_space_after_dot(), auto_capitalize_only_first_letter_of_word() (+78 more)

### Community 425 - "DialogContext"
Cohesion: 0.10
Nodes (25): advice_for_failure(), append_register_log_tail(), DIAG_EXPORT_RESULT, DiagExportSuccess, DIALOG_CTX, DialogContext, normalize_word_list(), read_log_tail() (+17 more)

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

### Community 431 - "ime_context"
Cohesion: 0.22
Nodes (9): ime_context, abi_version, app_id, caps, element_name, enabled, field_role, hint (+1 more)

### Community 432 - "export_diagnostics_zip"
Cohesion: 0.24
Nodes (11): build_pkzip(), collect_linux_install_diag_tail(), collect_log_tail(), collect_windows_register_log_tail(), crc32(), export_diagnostics_zip(), pkzip_builder_creates_valid_structure(), redact_paths_replaces_username() (+3 more)

### Community 433 - "Measure"
Cohesion: 0.16
Nodes (18): ArcDict, cli(), EngineDict, gen_vi_tatoeba_cli(), load_dic_paths(), Measure, measure_report(), render_json_measures() (+10 more)

### Community 434 - "2. Nhật ký finding"
Cohesion: 0.05
Nodes (46): CGFlagMapper, EventTapController, .currentFrontAppID, .isActive, .isSelfDisabled, Bool, Int, NSObjectProtocol (+38 more)

### Community 435 - "learning_curve.rs"
Cohesion: 0.42
Nodes (13): delta(), engine(), khong_hoc_rac_ban_phim_nhung_hoc_tu_that_va_viet_tat(), peek_khong_tieu_thu_delta(), press(), KeyEvent, Vec, rsi_delta_cac_luot_drain_cong_lai_bang_snapshot() (+5 more)

### Community 436 - "gen_vocab.rs"
Cohesion: 0.18
Nodes (14): collections, diacriticstyle, is_known_en_word, normalize_tone, check_en_words10k(), EN_COMPOUND_GROUPS, gen_en_compounds(), gen_vn_compounds() (+6 more)

### Community 437 - "CodingKeys"
Cohesion: 0.10
Nodes (20): CodingKeys, accepts, chars_saved, corrections, count, decided, enabled, from (+12 more)

### Community 438 - "caps.rs"
Cohesion: 0.31
Nodes (4): is_closer(), is_sentence_end(), sentence_start(), starts()

### Community 440 - "ChordFsm"
Cohesion: 0.27
Nodes (13): ChordFsm, ChordState, frac_cursor_latex(), fsm(), hud_due_after_pause(), latex_mode_no_chain(), Default, Self (+5 more)

### Community 441 - "emoji_picker.c"
Cohesion: 0.36
Nodes (13): append_utf8_char(), ime_instance, lc_emoji_picker, lc_emoji_close(), lc_emoji_format_aux(), lc_emoji_handle_key(), lc_emoji_init(), lc_emoji_open() (+5 more)

### Community 443 - "CloseHandle"
Cohesion: 0.16
Nodes (18): CloseHandle(), PHỤ LỤC — Trạng thái xử lý từng phát hiện (cập nhật 2026-10-09, sau v0.2.27 + vòng rà soát 2), current_process_is_app_container(), apply_ctrl_shift_default_once(), elevated_register_succeeded(), ensure_hook_running(), image_is_under(), is_process_elevated() (+10 more)

### Community 444 - "KeyTranslator"
Cohesion: 0.11
Nodes (15): AppInfo, .aboutMessage, .aboutTitle, UInt32, KeyTranslator, Data, KeyEvent, NSEvent (+7 more)

### Community 445 - "style_learning_v59.rs"
Cohesion: 0.19
Nodes (20): decay_count_lon_giam_nua(), ho_so_nho_khong_prune(), merge_delta_va_prune(), prune_mot_lan_khi_ho_so_truong_thanh(), StyleWord, Vec, words(), learning_opts() (+12 more)

### Community 446 - "test-linux-distro-matrix.sh"
Cohesion: 0.56
Nodes (8): cleanup(), distro_fcitx5_e2e(), distro_ibus_install(), distro_ibus_portable(), distro_system_install(), fail(), new_test_home(), test-linux-distro-matrix.sh script

### Community 447 - "StylePending"
Cohesion: 0.33
Nodes (5): BTreeMap, Formatter, Result, StylePending, Debug

### Community 448 - "tsf-min/src/register.rs"
Cohesion: 0.19
Nodes (21): ITfInputProcessorProfiles, activate(), clsid_key(), com_init(), dll_path(), guid_str(), ILOT_DEFPROFILE, install() (+13 more)

### Community 449 - "default"
Cohesion: 0.13
Nodes (15): altgr_enabled, chord_timeout_ms, clipboard_smart_paste, custom_keybindings, hotkeys, hud_pause_ms, leader_key, output_mode (+7 more)

### Community 450 - "os_linux.rs"
Cohesion: 0.17
Nodes (12): BACKEND_ORDERING, MAX_SUGGEST_RESULTS, PROBE_TIMEOUT, Duration, Mutex, Vec, SUGGEST_BUDGET, SUGGEST_CALLS_PER_CHECK (+4 more)

### Community 451 - "plan_key_with"
Cohesion: 0.31
Nodes (16): plan_key_with(), backspace_shrinks_and_ends_when_empty(), chars(), ctrl_space_pass_never_keeps_composition_r2_121(), delete_beyond_composition_becomes_delete_before(), enter_tab_return_real_key_without_newline_b2(), linux_overflow_commits_and_resets_c5(), pass_char_enters_composition() (+8 more)

### Community 452 - "TapPermission"
Cohesion: 0.22
Nodes (7): NSLock, Bool, TimeInterval, Timer, Void, TapPermission, ApplicationServices

### Community 454 - "English"
Cohesion: 0.10
Nodes (21): 0. Universal Scientific, Engineering & Math Engine (new in 0.2.83 — `textvn-math` crate), 1. Dream RSI Ergonomic Metrics & Performance Benchmarks (P10 / G19), 1. Pure In-Process Architecture & Uncompromising Memory Safety, 2. Contextual Bilingual Prediction & Dream RSI Ergonomics (P10), 2. Contextual Prediction Model P6 & User Style Profile P7 (53,952-Entry Corpora), 3. On-Device Grammar & Context Assistant (Grammarly-Grade Offline), 3. Typing Engines & Phonotactic Transformation Standard, 4. 20-Year UniKey Heritage Standard & Modern Office Utilities (+13 more)

### Community 457 - "test-linux-el8-package.sh"
Cohesion: 0.56
Nodes (8): cleanup(), el8_install(), el8_portable(), el8_system_install(), el8_wayland_live_portable(), fail(), new_el8_home(), test-linux-el8-package.sh script

### Community 458 - "lc_edit_batch_active"
Cohesion: 0.29
Nodes (15): lc_edit_batch, is_cont(), lc_edit_batch_active(), lc_edit_batch_commit(), lc_edit_batch_delete(), lc_edit_batch_end(), lc_edit_batch_take(), lc_utf8_char_count() (+7 more)

### Community 459 - "OwnerRule"
Cohesion: 0.24
Nodes (6): EngineOwner, imk, tap, OwnerRule, Bool, OwnerRuleTests

### Community 460 - "entries"
Cohesion: 0.50
Nodes (4): items, type, $ref, entries

### Community 461 - "conflict_notice.rs"
Cohesion: 0.23
Nodes (12): balloon_text(), ConflictSummary, e70_balloon_text_fits_shell_limit(), e70_notify_once_per_conflict_set(), e70_summarize_keeps_only_conflicts_and_skips_ctrl_shift(), HANDLED_ELSEWHERE, JSON, MARKER_FILE (+4 more)

### Community 462 - "lc_compose.h"
Cohesion: 0.12
Nodes (17): lc_selection_info, byte_len, char_len, del_offset, start_byte, lc_assist_expected_document_tail(), lc_text_ends_with(), utf8_last_char_len() (+9 more)

### Community 463 - "[0.2.14] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.14] — 2026-10-03, Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau"), Fixed (Windows), Tests

### Community 464 - "MathEngine"
Cohesion: 0.14
Nodes (8): AppGuard, is_math_boundary(), is_non_text_latex_cmd(), is_word_boundary(), MathEngine, MAX_WORD_BUF, Option, Self

### Community 465 - "do_register"
Cohesion: 0.15
Nodes (25): activate_for_session(), activate_tip(), active_profile_is_textvn(), com_init(), ctf_category_keys(), ctf_layout_matches_tsf_registry_schema(), ctf_profile_key(), ctf_tip_key() (+17 more)

### Community 466 - "worker_main"
Cohesion: 0.19
Nodes (13): en_us_neu_may_co_thi_provider_hoat_dong(), handle_job(), Job, raw_broker_init(), Option, Receiver, SyncSender, spawn_worker() (+5 more)

### Community 467 - "engine_boundary_fca.rs"
Cohesion: 0.35
Nodes (10): boundary_expands_dynamic_template_mat2x2(), boundary_expands_vn_alias_can(), engine_sticky(), eval_div_zero_stays_literal(), inline_eval_compact_syntax(), inline_eval_spaced_syntax(), reset_all_clears_word_buf(), Option (+2 more)

### Community 468 - "ModifierToggle"
Cohesion: 0.21
Nodes (11): claim_hotkey_toggle_at(), ctrl_shift_tap_toggles_once_and_only_without_other_keys(), is_ctrl_vk(), is_shift_vk(), modifier_toggle_rapid_sequence_matrix(), ModifierToggle, AtomicU64, Instant (+3 more)

### Community 469 - "save"
Cohesion: 0.36
Nodes (9): gen_app_icons_cli(), gen_store_art_cli(), gen_tray_icons_cli(), out_dir(), repo_root(), Path, PathBuf, Result (+1 more)

### Community 470 - "eval_leak.rs"
Cohesion: 0.22
Nodes (19): biz2_eval_leak_quet_ca_corpus_dev(), biz_1_scan_quet_cay_dem_dung_ro_ri_va_loi_khi_mat_tap_danh_gia(), cli(), content_lines(), data_ngrams(), LEAK_N, leak_sources(), leaks() (+11 more)

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

### Community 476 - "Option"
Cohesion: 0.33
Nodes (10): fcitx5_ctrl_shift_taken(), fcitx5_key_is_ctrl_shift(), linux_fcitx5_ctrl_shift_taken(), parse_fcitx5_version(), Option, Path, run_doctor(), run_doctor_ext() (+2 more)

### Community 478 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, cursor_left, delete_count, flags, insert, insert_len (+2 more)

### Community 479 - "overlay.rs"
Cohesion: 0.05
Nodes (67): ActionSink, Anchor, CandidateOverlay, COLOR_BG, COLOR_BORDER, COLOR_ERROR, COLOR_MUTED, COLOR_SELECTED_BG (+59 more)

### Community 480 - "P0-7 — TASKS: `textvn-session` + ARM (WBS cho nhiều agent)"
Cohesion: 0.15
Nodes (13): 1. Vai trò, 2. Definition of Ready (DoR) — chỉ `claim` khi ĐỦ, 3. Definition of Done (DoD) — task chỉ `release --status done` khi ĐỦ, 4.1 Đúng (correctness), 4.2 Nhất quán 3 OS (parity), 4.3 Bảo mật & riêng tư, 4.4 Hiệu năng & Dream RSI, 4.5 ARM (+5 more)

### Community 481 - ".CreateInstance"
Cohesion: 0.25
Nodes (7): ClassFactory_Impl, BOOL, c_void, GUID, IClassFactory_Impl, IUnknown, Result

### Community 482 - "T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007"
Cohesion: 0.33
Nodes (6): LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`, LNX-041 · Callback + self-disable (M) — `P3-3 §3`, LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`, LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`, LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S), T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### Community 483 - "paint_d2d"
Cohesion: 0.33
Nodes (9): paint_d2d(), paint_gdi_fallback(), paint_popup(), rgba(), D2D1_COLOR_F, HDC, RECT, Result (+1 more)

### Community 485 - "claim_debounced"
Cohesion: 0.40
Nodes (5): claim_debounced(), LAST_GLOBAL_TOGGLE_MS, AtomicU64, system_millis(), try_claim_global_toggle()

### Community 486 - "idle_ms"
Cohesion: 0.40
Nodes (5): $comment, default, minimum, type, idle_ms

### Community 487 - "items"
Cohesion: 0.25
Nodes (8): $comment, default, items, type, items, maxItems, minItems, custom_keybindings

### Community 488 - "suggest_vn.rs"
Cohesion: 0.14
Nodes (12): complete_vn_word, complete_vn_word(), Option, v(), VN_SYLLABLES_DATA, vn_syllables_sorted(), peek_completion_ho_so_day_ngan_sach_moi_phim(), prediction_perf_ngan_sach_moi_phim() (+4 more)

### Community 489 - "max_sentence_chars"
Cohesion: 0.40
Nodes (5): $comment, default, minimum, type, max_sentence_chars

### Community 490 - "windows-hook/src/lib.rs"
Cohesion: 0.08
Nodes (37): a_fast_callback_resets_slow_streak(), auto_only_processes_hook_owned_nonsecure_focus(), callback_budget_matrix(), CallbackDecision, classify_origin(), decide_full_truth_table(), empty_result(), engine_never_transforms_when_policy_says_pass() (+29 more)

### Community 491 - "P0-8 — Dream RSI 2.0 · kiến trúc vòng lặp · vòng nghiệp vụ (hiệu năng + chất lượng mọi tính năng)"
Cohesion: 0.18
Nodes (10): 1. Tóm tắt, 2. Hiện trạng đo được (2026-11-27, 0.2.94), 3. Nghiên cứu — cái đã được chứng minh, 4. Dream RSI 2.0 — KPI (bảng điểm), 5. Vòng nghiệp vụ ("giấc mơ" tiến hoá có kiểm soát), 6. Lộ trình mô hình (chất lượng), 7. Kiến trúc vòng lặp (hiệu năng) — trong `textvn-session` (G22), 8. Vòng chất lượng cho MỌI tính năng (một bảng điểm) (+2 more)

### Community 492 - "P3-6 — TEST PLAN (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Ma trận test, 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`), 3. App matrix Linux (20 app — đủ 3 framework), 4.1 Smoke (PR), 4. AT-SPI driver — `tools/linux/atspi-driver`, 5. Perf (ngưỡng — `PLAN §5.5`), 6. Release gate (RC Linux), 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`) (+2 more)

### Community 493 - "core"
Cohesion: 0.21
Nodes (11): Cell, Default, ObjGuard, Self, Vec, TextVNDisplayAttributeInfo, TextVNDisplayAttributeProvider, TextVNEnumDisplayAttributeInfo (+3 more)

### Community 494 - "P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15"
Cohesion: 0.25
Nodes (7): 0. Kết quả đối chiếu với handoff, 1. Vòng 16 — SA/QA, 2. Vòng 17 — Audit (license/privacy/AV), 3. Vòng 18 — PO/UX, 4.1 Bổ sung (2026-10-21, sau khi CI báo đỏ), 4. Kết luận, P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15

### Community 495 - "rules_linux.rs"
Cohesion: 0.25
Nodes (16): ATSPI_FLAG_PASSWORD, AtspiSnapshot, chat_apps_and_elements_select_backspace_type(), classify(), has_word_icase(), is_terminal_app(), LinuxProbeResult, none_snapshot_is_fail_safe_unknown() (+8 more)

### Community 496 - "[0.2.96] — 2026-11-28"
Cohesion: 0.40
Nodes (5): [0.2.96] — 2026-11-28, Added, Changed, Thay đổi / Changed, Thêm / Added

### Community 497 - "TextVN — Chỉ mục tài liệu & Giao thức Review"
Cohesion: 0.29
Nodes (7): 0. Tài liệu cho người dùng & phát hành, 1. Trạng thái các phần, 2. Thứ tự đọc cho agent mới (bắt buộc), 3. Giao thức Review (bắt buộc cho mọi phần), 4. Quy ước đặt tên (không đổi sau khi đã viết docs), 5. Changelog của chỉ mục, TextVN — Chỉ mục tài liệu & Giao thức Review

### Community 498 - "[0.2.38] — 2026-10-24"
Cohesion: 0.50
Nodes (4): [0.2.38] — 2026-10-24, Added, Changed, Fixed

### Community 499 - "Vec"
Cohesion: 0.24
Nodes (11): collect_linux_diagnostics(), collect_macos_diagnostics(), collect_windows_diagnostics(), detect_apple_vi_input_sources(), detect_linux_im_env_mismatch(), detect_other_vi_tips(), DiagnosticItem, e70_detect_other_vi_tips_from_user_profile_values() (+3 more)

### Community 500 - "ime_result"
Cohesion: 0.20
Nodes (10): ime_result, abi_version, action, cursor_left, delete_count, flags, insert, insert_len (+2 more)

### Community 501 - "ImeError"
Cohesion: 0.14
Nodes (13): ImeError, invalidArg, Data, ApplyError, cannotDelete, rejected, KeyTranslatorError, noChar (+5 more)

### Community 502 - "tests/reset.rs"
Cohesion: 0.22
Nodes (13): reset_after_enter_ms, down(), e78_app_and_plain_resets_clear(), ENTER, KEY_A, r2_171_any_key_in_between_clears_even_modifier(), r2_171_enter_mark_survives_toolkit_reset_like_legacy(), r2_171_focus_change_clears() (+5 more)

### Community 503 - "mode"
Cohesion: 0.50
Nodes (4): $comment, default, enum, mode

### Community 504 - "Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent"
Cohesion: 0.50
Nodes (4): Bản 0.2.8 — tự xác định EN/VI + Tab gợi ý + DPI đa màn hình + Store silent, Giải pháp + bằng chứng kiểm thử, Phát hành — v0.2.8 đã publish (release candidate, chưa ký số), Sự cố trong phiên — tra trước khi lặp lại

### Community 505 - "quick_telex"
Cohesion: 0.50
Nodes (4): quick_telex, $comment, default, type

### Community 506 - ".contains"
Cohesion: 0.20
Nodes (12): ArcDict, en_adjacent_words(), en_clean_sentence(), en_is_mechanically_clean(), en_lone_i(), en_missing_space_after(), en_space_before_punct(), gen_en_tatoeba_cli() (+4 more)

### Community 507 - "src/toggle_corpus.rs"
Cohesion: 0.12
Nodes (18): CorpusEvent, CorpusKey, CorpusStep, DEFAULT_GAP_MS, events(), parse(), parse_step(), Option (+10 more)

### Community 508 - ".commit"
Cohesion: 0.11
Nodes (19): 1. Summary of Changes, 1. Tổng quan thay đổi, 2. Kiểm chứng & Bất biến, 2. Verification & Invariants, 3. Post-Comparison Bug Fix (during SF-1f), 3. Sửa bổ sung sau đối chiếu chéo (SF-1f), 4. Giới hạn phạm vi, 4. Scope Limits (+11 more)

### Community 509 - "Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành"
Cohesion: 0.50
Nodes (4): Bản 0.2.4 — hoàn tất code gate, chờ workflow phát hành, Gate trước tag 0.2.4, Tiếp tục audit ZCode — engine (2026-10-01), Tiếp tục audit ZCode — runtime chưa có bằng chứng production

### Community 510 - "Vec"
Cohesion: 0.22
Nodes (13): crc32(), arm_w1_pe_machine_doc_dung_kien_truc(), arm_w6_msixbundle_x64_arm64(), build_synthetic_pe(), build_zip(), fake_msix_zip_bytes(), fake_pe(), fake_png() (+5 more)

### Community 511 - "[0.2.93] — 2026-11-27"
Cohesion: 0.22
Nodes (9): [0.2.93] — 2026-11-27, Added, Docs, Fixed, Kiểm thử / Testing, Sửa lỗi / Fixed, Testing, Tài liệu / Docs (+1 more)

### Community 514 - "audit_r2_142.rs"
Cohesion: 0.19
Nodes (8): check(), Vec, cap_lien_tu_tuyen_tinh_tren_cau_dai(), cau_dung_khong_bi_sua(), fixes(), loi_that_van_bat_duoc(), Vec, check_sentence

### Community 515 - ".run"
Cohesion: 0.33
Nodes (5): ApplySelectionFixSession_Impl, ITfEditSession_Impl, Result, SelectionCheckSession_Impl, TranscodeSession_Impl

### Community 516 - "collect_windows_env_summary"
Cohesion: 0.19
Nodes (11): collect_macos_env_summary(), collect_windows_env_summary(), DoctorReport, e69_markdown_report_renders_diagnostics_safely(), format_windows_version(), MacOsEnvSummary, parse_plist_string_value(), print_linux_env_summary() (+3 more)

### Community 517 - "hud_pause_ms"
Cohesion: 0.33
Nodes (6): $comment, default, maximum, minimum, type, hud_pause_ms

### Community 518 - "libraryloader"
Cohesion: 0.20
Nodes (11): gdi, libraryloader, setfocus, EDIT, main(), HWND, LPARAM, LRESULT (+3 more)

### Community 519 - "leader_key"
Cohesion: 0.33
Nodes (6): $comment, default, maxLength, minLength, type, leader_key

### Community 520 - "collect_report"
Cohesion: 0.15
Nodes (15): abi_sizes(), check_instance_mutex(), check_pipe_listening(), collect_report(), config_path(), count_panic_records(), default_export_path(), e70_export_contains_panic_tail_and_build_identity() (+7 more)

### Community 521 - "palette_max_results"
Cohesion: 0.33
Nodes (6): $comment, default, maximum, minimum, type, palette_max_results

### Community 522 - "ime_assist_issue"
Cohesion: 0.22
Nodes (9): ime_assist_issue, auto_fixable, confidence, end, kind, message, start, suggest (+1 more)

### Community 523 - "R2-26 — TextVN trong tiến trình sandbox Windows (AppContainer / LPAC)"
Cohesion: 0.22
Nodes (9): 1. Vấn đề, 2. Số đo thật (spike 2026-11-27, Windows 11 26300, tiến trình con AppContainer + LPAC), 3.1 Bản sao cấu hình (`%APPDATA%\TextVN\sandbox\`), 3.2 Pipe sandbox `\\.\pipe\textvn-ipc-v1-s<phiên>-sandbox`, 3. Thiết kế, 4. Mô hình đe doạ (mọi ứng dụng Store của người dùng nối được pipe sandbox và đọc bản sao), 5. Giới hạn còn lại, 6. Kiểm thử (+1 more)

### Community 525 - "[0.2.3] — 2026-10-01"
Cohesion: 0.33
Nodes (6): [0.2.3] — 2026-10-01, Added, Changed (CI — audit GitHub Actions, 5 major), Fixed (Windows), Fixed (đăng ký TSF — audit vòng sâu, 3 major), Housekeeping

### Community 526 - "KeyKind"
Cohesion: 0.13
Nodes (12): ghost_waits_for_key_up(), hwnd_alive(), idle_arms(), PANEL_IDLE_MS, register_timer_hwnd(), ITfContext, Rc, schedule() (+4 more)

### Community 527 - "P0 — REVIEW LOG (Phần 0: Nền tảng chung)"
Cohesion: 0.33
Nodes (5): Cross-check checklist (Review 2) — kết quả, P0 — REVIEW LOG (Phần 0: Nền tảng chung), REVIEW 1 — *Đúng & Đủ* (2026-09-27), REVIEW 2 — *Nhất quán & Sẵn sàng* (2026-09-27), Review tổng thể đợt 3 (full sweep 34 file docs + tiếp nhận scaffold) — 2026-09-27

### Community 528 - "Khắc phục sự cố TextVN (Troubleshooting)"
Cohesion: 0.20
Nodes (10): 1. Bắt đầu: xuất gói chẩn đoán (30 giây), 2. Xung đột bộ gõ (gõ lặp chữ, mất dấu, phím V/E không phản hồi), 3. Bảng mã lỗi đầy đủ, 4. Linux: phụ thuộc theo distro, 5. Windows: doanh nghiệp, LTSC, portable, quyền quản trị, 6. Báo lỗi, Giới hạn đã biết khi gõ trên Linux (0.2.92), Gói ZIP chứa gì (+2 more)

### Community 529 - "English"
Cohesion: 0.20
Nodes (10): 1. Start here: export a diagnostic bundle (30 seconds), 2. IME conflicts (doubled characters, lost diacritics, V/E toggle not responding), 3. Complete error-code table, 4. Linux: dependencies per distribution, 5. Windows: enterprise, LTSC, portable, administrator rights, 6. Reporting a bug, English, Known typing limitations on Linux (0.2.92) (+2 more)

### Community 530 - "diag_summary.rs"
Cohesion: 0.36
Nodes (8): build_issue_summary(), current_user_tokens(), e70_redact_handles_unicode_usernames(), e70_summary_keeps_only_zip_name_and_redacts_user(), e70_summary_without_findings(), redact_token(), redact_user_tokens(), Vec

### Community 531 - "Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục"
Cohesion: 0.67
Nodes (3): Bản 0.2.7 — Ctrl+Shift đổi mode + icon toàn cục, Kiểm chứng E2E trên máy thật (2026-10-02, sau khi publish), Phát hành — v0.2.7 đã publish (release candidate, chưa ký số)

### Community 532 - "AutostartManager"
Cohesion: 0.16
Nodes (9): AutostartManager, Bool, URL, Fixed (macOS — ưu tiên bản này), (B) Windows / xtask / Linux — `R3-*`, Kiểm chứng vòng 10 (host Windows, trước commit), Round 9 — Fix review vòng 3 trước tag v0.2.0 (2026-09-30), Vòng 10 — đóng nốt các mục hoãn (phát hành `v0.2.2`) (+1 more)

### Community 536 - "worker_main"
Cohesion: 0.18
Nodes (15): cached_support(), ComApartment, create_checker(), create_factory(), is_supported_by(), open_factory_and_probe(), Drop, Option (+7 more)

### Community 537 - "Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất"
Cohesion: 0.50
Nodes (4): 0.2.13 — EN/VI đa nền tảng + fix macOS nuốt từ điển, 0.2.14 — DPI gốc + UI thống nhất + common errors, Bản 0.2.13 + 0.2.14 — hoàn thiện 3 nền tảng, DPI theo màn hình, UI thống nhất, Phát hành + xác minh máy chủ repo

### Community 538 - "2. Kết quả kiểm thử"
Cohesion: 0.50
Nodes (4): 2. Kết quả kiểm thử, Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng), Kịch bản 1 — Cài đặt, Kịch bản 2 — Giải nén ra dùng luôn

### Community 539 - "ui_strings_generated.rs"
Cohesion: 0.12
Nodes (17): ABOUT_DETAILS, ABOUT_TAGLINE, ASSIST_AUTO_FIX_VALUES, ASSIST_IDLE_MS_DEFAULT, ASSIST_MODE_HINTS, ASSIST_MODE_LABELS, ASSIST_MODE_VALUES, CHARSET_LABELS (+9 more)

### Community 540 - "[0.2.73] — 2026-11-08"
Cohesion: 0.40
Nodes (5): [0.2.73] — 2026-11-08, Added, Added (công cụ đo, không đổi hành vi sản phẩm), Changed, Fixed

### Community 541 - "[0.2.64] — 2026-11-04"
Cohesion: 0.50
Nodes (4): [0.2.64] — 2026-11-04, Added, Changed, Fixed

### Community 542 - "[0.2.85] — 2026-11-22"
Cohesion: 0.40
Nodes (5): [0.2.85] — 2026-11-22, Fixed, Kiểm thử / Testing, Sửa lỗi / Fixed, Testing

### Community 543 - "[0.2.94] — 2026-11-27"
Cohesion: 0.29
Nodes (7): [0.2.94] — 2026-11-27, Changed, Kiểm thử / Testing, Process, Quy trình / Process, Testing, Thay đổi / Changed

### Community 544 - "[0.2.95] — 2026-11-27"
Cohesion: 0.29
Nodes (7): [0.2.95] — 2026-11-27, Changed, Fixed, Process, Quy trình / Process, Sửa lỗi / Fixed, Thay đổi / Changed

### Community 545 - "[0.2.9] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.9] — 2026-10-02, Added, Documented, Tests

### Community 556 - "KeyOutcome"
Cohesion: 0.05
Nodes (41): Action, commit, pass, replace, restore, KeyOutcome, .isConsumed, .isError (+33 more)

### Community 557 - "4. Linux — `libspelling` (GNOME) **và** `enchant-2`"
Cohesion: 0.29
Nodes (7): 4.1 Sự thật quan trọng: `check_ranges` **chưa có** trong bản phát hành, 4.2 libspelling — ràng buộc & license, 4.3 `enchant-2` — phương án **nhẹ hơn** (khuyến nghị cho IBus/Fcitx5), 4.4 Từ điển `vi` trên Linux — **user cài**, ta không ship, 4.5 Code sketch (C — sketch trong doc), 4.6 Rủi ro riêng của Linux, 4. Linux — `libspelling` (GNOME) **và** `enchant-2`

### Community 558 - "test-linux-package.sh"
Cohesion: 0.26
Nodes (16): tv_runtime_dep_packages(), fail(), fcitx_profile(), new_home(), run_xtask(), scenario_install(), scenario_portable(), test-linux-package.sh script (+8 more)

### Community 559 - "P4-9 — REVIEW (3 vòng) kết quả Agent Dict: từ điển tiếng Việt (AS-040/AS-041)"
Cohesion: 0.29
Nodes (6): 1. Vòng 1 — SA / QA, 2. Vòng 2 — Audit (license) — **phát hiện nghiêm trọng**, 3. Vòng 3 — PO / QA (giá trị & phạm vi), 4. Kết luận & trạng thái, 5. Fassist-104 (major) — CI đỏ: **không được tự khai báo symbol `open`**, P4-9 — REVIEW (3 vòng) kết quả Agent Dict: từ điển tiếng Việt (AS-040/AS-041)

### Community 567 - "coordination/ — giao nhận việc & thông báo giữa các agent (G21)"
Cohesion: 0.33
Nodes (6): 1. Một nguồn sự thật: `coordination/board.jsonl`, 2. Định danh agent, 3. Quy tắc giao nhận việc (R1–R14), 4. Mảnh ghi chú phát hành `coordination/changes/<TASK>.md`, 5. Ví dụ một vòng, coordination/ — giao nhận việc & thông báo giữa các agent (G21)

### Community 568 - "[0.2.81] — 2026-11-18"
Cohesion: 0.50
Nodes (4): [0.2.81] — 2026-11-18, Sửa lỗi / Fixed, Thay đổi / Changed, Thêm mới / Added

### Community 569 - "run_tray_app"
Cohesion: 0.26
Nodes (13): add_tray_icon(), copy_to_wide_buf(), installed_since_boot(), notify_ime_conflicts_at_startup(), HWND, LPARAM, LRESULT, WPARAM (+5 more)

### Community 570 - "[0.2.29] — 2026-10-10"
Cohesion: 0.67
Nodes (3): [0.2.29] — 2026-10-10, Changed, Fixed — gỡ cài đặt sạch (R2-99, R2-100)

### Community 571 - "linux-fcitx5/src/engine.h"
Cohesion: 0.20
Nodes (10): Instance, TextVNEngineFactory, create, AddonFactory, AddonInstance, AddonManager, cstdint, InputContextProperty (+2 more)

### Community 572 - ".testDefaultConfig"
Cohesion: 0.13
Nodes (12): 0. Đọc trước khi làm bất cứ việc gì (trình tự), 1. Luật không được phá (vi phạm = task chưa xong), 2. Chiến lược phát hành — các mốc bắt buộc theo thứ tự, 3. Kiến trúc & ràng buộc kỹ thuật cốt lõi, AGENTS.md — luật bắt buộc cho MỌI agent làm việc trong repo TextVN, 0. Thiết kế tóm tắt (Design Intent), 1. DFMEA (Design Failure Mode and Effects Analysis), 2. DRBFM (Design Review Based on Failure Mode) (+4 more)

### Community 573 - "textvn_core"
Cohesion: 0.08
Nodes (25): e50_multiple_en_words_in_sequence(), e50_no_restore_when_not_needed(), e50_restore_then_another_en_word(), e50_restore_then_immediate_typing(), e50_unikey_dd_after_restore(), e50_vietnamese_after_restore(), Vec, type_and_get() (+17 more)

### Community 574 - "ime_imk_input"
Cohesion: 0.17
Nodes (12): ime_imk_input, abi_version, action, delete_count, insert, insert_len, marked, marked_len (+4 more)

### Community 575 - "[0.2.8] — 2026-10-02"
Cohesion: 0.50
Nodes (4): [0.2.8] — 2026-10-02, Added, Changed, Fixed

### Community 576 - "WndProc"
Cohesion: 0.09
Nodes (21): [0.2.89] — 2026-11-25, Added, Changed & Fixed, Kiểm thử / Testing, Testing, Tính năng mới & Sửa lỗi / Added, Changed & Fixed, Bản 0.2.89 — E69 Tự động cài phụ thuộc Linux (`ibus`/`fcitx5`/`gtk`), phát hiện xung đột bộ gõ cũ & xuất gói log chẩn đoán 1 chạm 3 OS (2026-11-25), objbase, HINSTANCE (+13 more)

### Community 577 - "[0.2.66] — 2026-11-07"
Cohesion: 0.50
Nodes (4): [0.2.66] — 2026-11-07, Added, Changed, Fixed

### Community 580 - "BIZ-3 — `cargo xtask quality-trend`: bảng xu hướng KPI theo bản cho README/CHANGELOG"
Cohesion: 0.14
Nodes (13): BIZ-3 — `cargo xtask quality-trend`: bảng xu hướng KPI theo bản cho README/CHANGELOG, Conservative decisions when the spec was open (recorded as required), EN, Kiểm thử / Testing, Làm gì, Một nguồn sự thật (G22 — không sao chép logic), Quyết định conservative khi spec không chốt (ghi rõ theo yêu cầu), Single source of truth (G22 — no copied logic) (+5 more)

### Community 581 - "[0.2.91] — 2026-11-26"
Cohesion: 0.40
Nodes (5): [0.2.91] — 2026-11-26, Changed, Cải thiện giao diện / Changed, Kiểm thử / Testing, Testing

### Community 582 - "EditBatch"
Cohesion: 0.15
Nodes (4): EDIT_BATCH_MAX_BYTES, EditBatch, Option, Self

### Community 583 - "RSI-1 — Bench KSR-oracle / Bench KSR-oracle"
Cohesion: 0.50
Nodes (3): EN, RSI-1 — Bench KSR-oracle / Bench KSR-oracle, VI

### Community 584 - "tokenize.rs"
Cohesion: 0.30
Nodes (11): chu_so_la_token_khac(), classify(), offset_la_ky_tu_unicode(), Item, Iterator, Vec, tach_dung_ba_token(), tokenize() (+3 more)

### Community 585 - "Vec"
Cohesion: 0.20
Nodes (13): biz2_dot_bien_ghost_sai_moi_tu_bo_lo(), biz2_miss_report_tat_dinh_va_heldout_chi_co_so(), biz2_tap_dev_tai_lap_duoc(), build_bench_corpus(), build_heldout(), miss_report_run(), MissAgg, MissDetail (+5 more)

### Community 586 - "probe/src/main.rs"
Cohesion: 0.24
Nodes (11): c_void, com, hmodule, IMAGE_FILE_MACHINE, itftextinputprocessor, machine_name(), main(), module_loaded() (+3 more)

### Community 587 - ".run"
Cohesion: 0.06
Nodes (37): ApplyReplace, InjectedMarker, Int64, ApplyReplaceParityTests, App, EmittedCase, Event, MacTrace (+29 more)

### Community 588 - "B. macOS"
Cohesion: 0.09
Nodes (20): OutputStrategy, backspaceType, forwardAsCommit, passthrough, preedit, selectionReplace, B10. [THẤP] `IMKTextTarget.swift:40-42` — `NSUnderlineStyle([]).rawValue` = 0, B11. [THẤP] Dead code/state chết: `heldMods` (chỉ ghi), `noteFlagsChanged()` (+12 more)

### Community 589 - ".push"
Cohesion: 0.15
Nodes (12): IpcCodecTests, Data, [0.2.84] — 2026-11-22, Fixed, Kiểm thử / Testing, Sửa lỗi / Fixed, Testing, EN (+4 more)

### Community 590 - "BIZ-1 — `cargo xtask quality-score`: bảng điểm chất lượng chung"
Cohesion: 0.17
Nodes (11): BIZ-1 — `cargo xtask quality-score`: bảng điểm chất lượng chung, Bằng chứng, EN, English — Post-review fixes (2026-11-28), Evidence, Làm gì, Sửa kèm, Touched files (+3 more)

### Community 591 - "RSI-7 — Cổng ngôn ngữ cho chữ mờ / Ghost language gate"
Cohesion: 0.50
Nodes (3): EN, RSI-7 — Cổng ngôn ngữ cho chữ mờ / Ghost language gate, VI

### Community 592 - "PERF-1 — Baseline hiệu năng theo target triple (`perf/baseline-<triple>.json`)"
Cohesion: 0.17
Nodes (11): English, English — Follow-up: real numbers from CI (2026-11-28), English — Post-review fix (R2-176, 2026-11-28), Kiểm chứng, PERF-1 — Baseline hiệu năng theo target triple (`perf/baseline-<triple>.json`), Summary of Changes, Tiếng Việt, Tóm tắt thay đổi (+3 more)

### Community 593 - "[0.2.46] — 2026-10-26"
Cohesion: 0.67
Nodes (3): [0.2.46] — 2026-10-26, Added, Fixed

### Community 594 - "ime_imk_input"
Cohesion: 0.17
Nodes (12): ime_imk_input, abi_version, action, delete_count, insert, insert_len, marked, marked_len (+4 more)

### Community 595 - "toggle_and_batch.rs"
Cohesion: 0.21
Nodes (7): kd(), ku(), sf2_standalone_ctrl_tap_gioi_han_400ms_r2_143(), sf2_toggle_ctrl_shift_space_va_capslock_adr011_va_debounce_250ms(), sf2_toggle_huy_khi_co_phim_khac_alt_super_hoac_focus_reset(), textvn_session, utf8_char_count

### Community 596 - "has_word_icase"
Cohesion: 0.20
Nodes (10): has_word_icase(), English, Kiểm chứng, Phần còn lại (Phần 2 — mở khi `SF-3` nhả `ffi/`), Remaining Scope (Part 2 — once `SF-3` releases `ffi/`), SF-6 (Phần 1): Luật nhận diện ô nhập Linux/macOS & Client IPC dùng chung, Summary of Changes, Tiếng Việt (+2 more)

### Community 597 - "EN"
Cohesion: 0.17
Nodes (11): ARM-W5 — E2E ma trận tiến trình (gõ TSF thật đa kiến trúc), Bằng chứng & giới hạn còn lại (cho `ARM-W5b`), EN, Evidence & remaining limitation (handed off to `ARM-W5b`), Goal (P0-7 acceptance), Kết hợp với E89, Làm gì, Mục tiêu (Acceptance P0-7) (+3 more)

### Community 598 - "resolve_baseline_path"
Cohesion: 0.25
Nodes (8): json_str(), num_str(), resolve_baseline_path(), Option, Path, PathBuf, Result, validate_baseline()

### Community 599 - "chars"
Cohesion: 0.29
Nodes (11): am_tiet_viet_that_khong_bi_restore(), canh_bao_cai_muc_mo_ho(), chars(), data_en_common_file_fires(), data_stop_en_file_is_self_consistent(), data_vn_common_file_is_valid(), default_english_words(), parse_word_list() (+3 more)

### Community 600 - "CTextVNFFI/include/textvn_ffi.h"
Cohesion: 0.20
Nodes (9): ime_assist_result, abi_version, count, issues, ime_instance, ime_os_spell, abi_version, ctx (+1 more)

### Community 601 - "ARM-L4 — E2E 6 distro Linux trên container ARM64 (`release.yml`)"
Cohesion: 0.18
Nodes (10): ARM-L4 — E2E 6 distro Linux trên container ARM64 (`release.yml`), Bằng chứng cuối (G23 dry-run), English, English — Post-review fix (G22, 2026-11-28), Kiểm chứng, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi (+2 more)

### Community 604 - "[0.2.10] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.10] — 2026-10-03, Documented, Fixed, Tests

### Community 606 - "P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết"
Cohesion: 0.18
Nodes (10): 1. Nguồn `FieldContext` trên Linux, 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`), 3. App preset mặc định — `data/appdb.default.json` (phần Linux), 4. Permission & fallback (RL4), 5. Module dùng chung `libtextvn-linux-common.a` (C), 6. Override & state (khớm `P0-3 §3.1` — path Linux), 7. Env matrix & detect framework (dùng cho `textvn doctor` — `PLAN §5.3`), 8. Test (+2 more)

### Community 607 - "[0.2.79] — 2026-11-14"
Cohesion: 0.50
Nodes (4): [0.2.79] — 2026-11-14, Added, Changed, Fixed

### Community 609 - "Session"
Cohesion: 0.12
Nodes (12): AssistAction, Caps, ResetKind, Self, Session, SessionEvent, TimerId, RESET_AFTER_ENTER_MS (+4 more)

### Community 610 - "sim_run_oracle"
Cohesion: 0.21
Nodes (16): build_sentence(), enter_key(), MissReason, KeyEvent, Self, Sentence, sim_baseline_keys(), sim_press() (+8 more)

### Community 611 - "ime_session_event"
Cohesion: 0.20
Nodes (10): ime_session_event, abi_version, arg, ch, kind, mods, now_ms, text (+2 more)

### Community 612 - "ime_compose_plan"
Cohesion: 0.20
Nodes (10): ime_compose_plan, abi_version, cursor_left, delete_before, eaten, end, has_text, reset_engine (+2 more)

### Community 613 - "[0.2.77] — 2026-11-12"
Cohesion: 0.67
Nodes (3): [0.2.77] — 2026-11-12, Added, Fixed

### Community 614 - "COMPAT-1 — Ma trận Windows `windows-2022` + `windows-2025` + `windows-11-arm`"
Cohesion: 0.20
Nodes (9): COMPAT-1 — Ma trận Windows `windows-2022` + `windows-2025` + `windows-11-arm`, English, English — Post-review fixes (2026-11-28), Kiểm chứng, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi, Verification (+1 more)

### Community 615 - "apply"
Cohesion: 0.47
Nodes (9): apply(), apply_from(), key(), Action, KeyEvent, Vec, sim1_pass_backspace_ctrl_space_va_ky_tu_dieu_khien(), sim1_replace_commit() (+1 more)

### Community 616 - "ffi/include/textvn_ffi.h"
Cohesion: 0.18
Nodes (10): ime_instance, ime_os_spell, abi_version, ctx, ime_session, ime_session_out, abi_version, arena (+2 more)

### Community 617 - "ime_session_event"
Cohesion: 0.20
Nodes (10): ime_session_event, abi_version, arg, ch, kind, mods, now_ms, text (+2 more)

### Community 618 - "tv_restart_ibus"
Cohesion: 0.29
Nodes (9): textvn-common.sh script, tv_ibus_clear_cache(), tv_ibus_refresh_cache(), tv_ok(), tv_restart_ibus(), tv_say(), tv_stop_wait_bus_engine(), tv_textvn_install_path() (+1 more)

### Community 619 - "mac_corpus_cases.rs"
Cohesion: 0.49
Nodes (9): all_cases(), bs(), case(), CorpusCase, du_nhom_case_theo_p2_5(), imk(), Vec, tap() (+1 more)

### Community 620 - "preflight.rs"
Cohesion: 0.27
Nodes (8): baseline_path_for(), Option, Path, Result, run(), Step, STEPS, target_triple_for()

### Community 621 - "3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 622 - "ime_compose_plan"
Cohesion: 0.20
Nodes (10): ime_compose_plan, abi_version, cursor_left, delete_before, eaten, end, has_text, reset_engine (+2 more)

### Community 623 - "PickerLayout"
Cohesion: 0.43
Nodes (4): PickerLayout, .GRID, .LIST, Self

### Community 624 - "3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU"
Cohesion: 0.22
Nodes (9): 3.1. Lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift trên cả 3 hệ điều hành, 3.2. Lỗi gõ chữ bị gạch chân (Underline / Preedit Artifacts) trên Windows, macOS và Linux, 3.3. Vấn đề phải chạy bằng quyền Administrator mới đăng ký được TSF trên Windows 11, 3. ĐIỀU TRA GỐC RỄ (ROOT-CAUSE ANALYSIS) CÁC LỖI TRỌNG YẾU ĐƯỢC NÊU, A. Trên Windows: Hiện tượng "Toggle Đôi" (Double-Toggle Race Condition), A. Trên Windows: Thiếu sót đăng ký Display Attribute Provider với Category Manager, B. Trên macOS: Cấu hình `non_preedit` bị bỏ quên trong IMK, C. Trên Linux: Chỉ sử dụng Client Preedit tiêu chuẩn (+1 more)

### Community 625 - "ime_assist_plan"
Cohesion: 0.25
Nodes (8): ime_assist_plan, abi_version, delete_chars, delete_from, delete_glyphs, fixed_count, original_tail, replacement

### Community 626 - "Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit"
Cohesion: 0.29
Nodes (7): Audit tiếp diễn — TextVN, 2026-09-30, Giới hạn và rủi ro còn mở, Lệnh tái lập, Perf — job `perf regression` vẫn đỏ trên CI, **không phải** hồi quy, Ranh giới xác minh, Vòng 10 — delta từ `v0.2.0` đến `v0.2.1`, Vòng 11 — `v0.2.2`: đóng F3-13 (mở nhầm Cài đặt lúc login), F3-8 và nit

### Community 627 - "ARM-L3 — Phát hành gói `TextVN-<v>-linux-aarch64.tar.gz` trong `release.yml`"
Cohesion: 0.25
Nodes (7): ARM-L3 — Phát hành gói `TextVN-<v>-linux-aarch64.tar.gz` trong `release.yml`, English, Kiểm chứng, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi, Verification

### Community 628 - "ARM-L5 — Gói `.deb arm64` và `PKGBUILD`/`.SRCINFO` `aarch64`"
Cohesion: 0.25
Nodes (7): ARM-L5 — Gói `.deb arm64` và `PKGBUILD`/`.SRCINFO` `aarch64`, English, Kiểm chứng, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi, Verification

### Community 629 - "ARM-W3 — Đăng ký TSF trên Windows ARM64 (IsWow64Process2, forwarder & ACL 4 DLL)"
Cohesion: 0.25
Nodes (7): ARM-W3 — Đăng ký TSF trên Windows ARM64 (IsWow64Process2, forwarder & ACL 4 DLL), English, Kiểm chứng, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi, Verification

### Community 630 - "COMPAT-2 — Mở rộng ma trận distro E2E (claude-opus-c4c5a567)"
Cohesion: 0.25
Nodes (7): COMPAT-2 — Mở rộng ma trận distro E2E (claude-opus-c4c5a567), EN, Part 1 — `9a0c101f` (pushed), Part 2 — `release.yml` + `scripts/run-distro-container.sh`, Phần 1 — `9a0c101f` (đã đẩy), Phần 2 — `release.yml` + `scripts/run-distro-container.sh`, VI

### Community 631 - "English"
Cohesion: 0.25
Nodes (7): English, Kiểm chứng, PERF-2 — Gate tất định đa kiến trúc (`x86_64` vs `aarch64`) trong `ci-shared`, Summary of Changes, Tiếng Việt, Tóm tắt thay đổi, Verification

### Community 632 - "ime_assist_plan"
Cohesion: 0.25
Nodes (8): ime_assist_plan, abi_version, delete_chars, delete_from, delete_glyphs, fixed_count, original_tail, replacement

### Community 633 - "Measurement"
Cohesion: 0.18
Nodes (16): bench_field_switch_resolve(), bench_resolve(), best_of(), best_of_giu_vong_nhanh_nhat(), compare(), compare_bao_thieu_baseline(), compare_bat_hoi_quy_qua_nguong(), compare_bat_vuot_ngan_sach() (+8 more)

### Community 634 - "ARM-W6 — `.msixbundle` x64 + arm64 (claude-opus-7e41b2d9) — phần 1 + 2"
Cohesion: 0.29
Nodes (6): ARM-W6 — `.msixbundle` x64 + arm64 (claude-opus-7e41b2d9) — phần 1 + 2, Bằng chứng / Evidence, EN, Kiểm thử / Testing, Tài liệu / Docs, VI

### Community 635 - "ctrl_shift_marker_path"
Cohesion: 0.40
Nodes (6): compatibility_hook_path(), ctrl_shift_marker_path(), legacy_freed_layout_hotkey(), restore_ctrl_shift_if_we_freed(), Option, PathBuf

### Community 636 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.29
Nodes (7): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust, Phát hiện W-06: Điểm mù kiến trúc 32-bit WOW64 — Zalo PC và toàn bộ ứng dụng x86 không thể nạp TSF DLL 64-bit

### Community 637 - "lc_sentence_fix_plan"
Cohesion: 0.40
Nodes (5): lc_sentence_fix_plan, delete_glyphs, fixed_count, original_tail, replacement

### Community 638 - "AppDelegate.swift"
Cohesion: 0.40
Nodes (3): Cocoa, SwiftUI, TextVNAppLib

### Community 639 - "lc_find_settings_binary"
Cohesion: 0.40
Nodes (6): lc_find_settings_binary(), [0.2.88] — 2026-11-25, Added, Changed & Fixed, Kiểm thử / Testing, Testing, Tính năng mới & Sửa lỗi / Added, Changed & Fixed

### Community 640 - "ime_session_cmd"
Cohesion: 0.33
Nodes (6): ime_session_cmd, arg0, arg1, kind, text_len, text_off

### Community 641 - "4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX)"
Cohesion: 0.33
Nodes (6): 4.1. Module Windows (TSF, Hook, Tray, CLI Register, Installer & MSIX), Phát hiện W-01: Hook bàn phím toàn cục `WH_KEYBOARD_LL` trong Tray, Phát hiện W-02: Lỗ hổng nâng quyền Portable UAC, Phát hiện W-03: Sửa đổi Registry Windows Hotkey thiếu hoàn trả, Phát hiện W-04: Thiếu kiểm soát truy cập (DACL) trên Named Pipe IPC, Phát hiện W-05: Đóng gói MSIX và bài toán Full-Trust

### Community 642 - "ARM-W5b.md"
Cohesion: 0.33
Nodes (5): Bằng chứng / Evidence, EN, Giới hạn / Limits, Tài liệu / Docs, VI

### Community 643 - "BIZ-2 — Miss report Dream RSI (claude-opus-c4c5a567)"
Cohesion: 0.33
Nodes (5): BIZ-2 — Miss report Dream RSI (claude-opus-c4c5a567), EN, Kiểm thử + đột biến, Tests + mutations, VI

### Community 644 - "R2-175 — Fcitx5 5.0.x giữ Ctrl+Shift: `textvn doctor` cảnh báo (claude-opus-c4c5a567)"
Cohesion: 0.33
Nodes (5): EN, Kiểm thử / Testing, R2-175 — Fcitx5 5.0.x giữ Ctrl+Shift: `textvn doctor` cảnh báo (claude-opus-c4c5a567), Tài liệu / Docs, VI

### Community 645 - "REG-1 — Sổ hồi quy + gate (claude-opus-c4c5a567)"
Cohesion: 0.33
Nodes (5): EN, Kiểm thử / Testing, REG-1 — Sổ hồi quy + gate (claude-opus-c4c5a567), Tài liệu / Docs, VI

### Community 646 - "REG-2 — Bù bằng chứng cho mã lỗi cũ (claude-opus-c4c5a567)"
Cohesion: 0.33
Nodes (5): EN, Kiểm thử / Testing, REG-2 — Bù bằng chứng cho mã lỗi cũ (claude-opus-c4c5a567), Tài liệu / Docs, VI

### Community 647 - "ime_imk_op"
Cohesion: 0.40
Nodes (5): ime_imk_op, arg, kind, text_len, text_off

### Community 648 - "3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?"
Cohesion: 0.33
Nodes (6): 1. Bằng chứng mã nguồn: Gói nộp lên Partner Center THỰC CHẤT VẪN LÀ GÓI PER-USER (Chỉ ghi vào HKCU), 2. Cơ chế thực tế của Microsoft Store Validator: Tại sao HKCU luôn bị mù trong Sandbox?, 3.4.5. Điều tra chuyên sâu lần nộp v0.2.21: Vì sao Store vẫn tiếp tục từ chối ở 3 bài kiểm tra dù đã sửa Publisher và nâng phiên bản?, 3. Lỗ hổng định danh ký tự: Nguy cơ lệch Codepoint nghiêm trọng giữa "Linh βùi" và Partner Center, 4. Nghịch lý bế tắc (Catch-22) của gói Raw Win32 EXE trên Microsoft Store, 5. Khuyến nghị dứt điểm: Dừng nộp file `.exe` và chuyển ngay sang nộp gói MSIX Full-Trust

### Community 649 - "3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions)"
Cohesion: 0.33
Nodes (6): 1. Đánh giá thành quả: Đập tan hoàn toàn 3 lỗi kiểm duyệt tự động cố hữu, 2. Giải mã thông báo mới từ Store: Chính sách 10.2.9 (Security - Package Submissions), 3.4.7. Bước ngoặt lịch sử trên v0.2.24: Vượt qua hoàn toàn 3 bài kiểm tra Silent/ARP/Bundleware & Giải phẫu Thông báo Chính sách 10.2.9 (Security - Package Submissions), 3. Hai phương án xử lý do chính Microsoft đề xuất và lộ trình hành động chi tiết, Phương án 1: Ký số cho file EXE bằng chứng thư số tin cậy (Giữ luồng Win32 EXE), Phương án 2 (Khuyến nghị hàng đầu): Repackage sang MSIX Full-Trust (Microsoft Ký số Miễn phí)

### Community 650 - "ime_session_cmd"
Cohesion: 0.33
Nodes (6): ime_session_cmd, arg0, arg1, kind, text_len, text_off

### Community 651 - "ime_session_config"
Cohesion: 0.40
Nodes (5): ime_session_config, abi_version, caps, config_json, config_len

### Community 652 - "ime_session_out"
Cohesion: 0.40
Nodes (5): ime_session_out, abi_version, arena, cmds, count

### Community 653 - "ime_imk_plan"
Cohesion: 0.40
Nodes (5): ime_imk_plan, abi_version, arena, count, ops

### Community 654 - "ARM-L1 + ARM-L2 — CI Linux aarch64 native + gate `char` không dấu (claude-opus-c4c5a567)"
Cohesion: 0.40
Nodes (4): ARM-L1 + ARM-L2 — CI Linux aarch64 native + gate `char` không dấu (claude-opus-c4c5a567), EN, Kiểm thử / Testing, VI

### Community 655 - "ARM-M1.md"
Cohesion: 0.40
Nodes (4): EN, Kiểm thử / Testing, Tài liệu / Docs, VI

### Community 656 - "ARM-W1 — Build + test native Windows ARM64 (claude-opus-c4c5a567)"
Cohesion: 0.40
Nodes (4): ARM-W1 — Build + test native Windows ARM64 (claude-opus-c4c5a567), EN, Kiểm thử / Testing, VI

### Community 657 - "ARM-W2 — Forwarder ARM64X cho TSF (`textvn-tsf.dll`) (antigravity-gemini-37c7c6bc)"
Cohesion: 0.40
Nodes (4): ARM-W2 — Forwarder ARM64X cho TSF (`textvn-tsf.dll`) (antigravity-gemini-37c7c6bc), EN, Kiểm thử / Testing, VI

### Community 658 - "BOARD-1.md"
Cohesion: 0.40
Nodes (4): EN, Kiểm thử / Testing, Tài liệu / Docs, VI

### Community 659 - "LOOP-2 — Chỉ mục tiền tố top-K cho `predict_prefix_candidates` (antigravity-gemini-37c7c6bc)"
Cohesion: 0.40
Nodes (4): EN, Kiểm thử / Testing, LOOP-2 — Chỉ mục tiền tố top-K cho `predict_prefix_candidates` (antigravity-gemini-37c7c6bc), VI

### Community 660 - "RSI-2 — Corpus đánh giá Dream RSI tách tập + gate chống rò rỉ (claude-opus-c4c5a567)"
Cohesion: 0.40
Nodes (4): EN, Kiểm thử / Testing, RSI-2 — Corpus đánh giá Dream RSI tách tập + gate chống rò rỉ (claude-opus-c4c5a567), VI

### Community 661 - "SF-1b.md"
Cohesion: 0.40
Nodes (4): EN, Kiểm thử / Testing, Tài liệu / Docs, VI

### Community 662 - "Audit sau 0.2.68 — DFMEA + DRBFM (R2-140…R2-153)"
Cohesion: 0.40
Nodes (4): 2. DFMEA (S/O/D 1–10, RPN sau khi sửa), 3. DRBFM — điểm thay đổi có chủ đích, 4. Rủi ro còn lại (đã biết, chưa xử lý), Audit sau 0.2.68 — DFMEA + DRBFM (R2-140…R2-153)

### Community 663 - "4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging)"
Cohesion: 0.40
Nodes (5): 4.3. Module Linux (Common IPC, IBus, Fcitx5, GTK4 Settings & Packaging), Phát hiện L-01: Bộ phân tích JSON IPC thủ công bằng `strstr`, Phát hiện L-02: Đường dẫn Socket không an toàn trong `/tmp`, Phát hiện L-03: Thiếu định dạng đóng gói chuẩn của các bản phân phối Linux, Phát hiện L-04: Xung đột Wayland vs X11 và Cơ chế Surrounding Text trên Chat Apps

### Community 664 - "ime_session_config"
Cohesion: 0.40
Nodes (5): ime_session_config, abi_version, caps, config_json, config_len

### Community 665 - "ime_suggest"
Cohesion: 0.40
Nodes (5): ime_suggest, abi_version, count, items, lens

### Community 666 - "ime_suggest"
Cohesion: 0.40
Nodes (5): ime_suggest, abi_version, count, items, lens

### Community 667 - "test-macos-universal-gate.sh"
Cohesion: 0.60
Nodes (4): ng(), ok(), test-macos-universal-gate.sh script, TEXTVN_UNI_LIPO

### Community 668 - "[0.2.92] — 2026-11-27"
Cohesion: 0.40
Nodes (5): [0.2.92] — 2026-11-27, Docs, Kiểm thử / Testing, Testing, Tài liệu / Docs

### Community 669 - "[0.2.13] — 2026-10-03"
Cohesion: 0.50
Nodes (4): [0.2.13] — 2026-10-03, Added, Documented, Fixed

### Community 670 - "Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên"
Cohesion: 0.40
Nodes (5): 1. Thứ tự xử lý mỗi phím (engine `core/src/lib.rs`), 2. Thứ tự con tại ranh giới từ (`restore_en::should_restore`), 3. Bảng xung đột có thể xảy ra và cách giải, 4. Nguyên tắc điều chỉnh thiết kế (ghi để không lặp tranh luận), Xác định ngôn ngữ khi gõ — ma trận xung đột và thứ tự ưu tiên

### Community 671 - "ARM-M2 — E2E IMK trên runner macOS arm64 — ĐẠT MỘT PHẦN, giới hạn runner (claude-opus-c4c5a567)"
Cohesion: 0.50
Nodes (3): ARM-M2 — E2E IMK trên runner macOS arm64 — ĐẠT MỘT PHẦN, giới hạn runner (claude-opus-c4c5a567), EN, VI

### Community 672 - "COMPAT-3 — E2E gõ tiếng Việt THẬT trong phiên desktop Linux (claude-opus-c4c5a567)"
Cohesion: 0.50
Nodes (3): COMPAT-3 — E2E gõ tiếng Việt THẬT trong phiên desktop Linux (claude-opus-c4c5a567), EN, VI

### Community 673 - "FIX-V53 — test ngân sách thời gian v53 chống nhiễu runner (R2-177)"
Cohesion: 0.50
Nodes (3): EN, FIX-V53 — test ngân sách thời gian v53 chống nhiễu runner (R2-177), VI

### Community 674 - "SF-9 (Phần 1 + Phần 2 / Part 1 + Part 2) — Một nguồn chữ UI & mô hình cài đặt dùng chung 3 nền tảng"
Cohesion: 0.40
Nodes (4): Audit chéo / Cross-audit (claude-opus-7e41b2d9, 2026-11-28), EN, SF-9 (Phần 1 + Phần 2 / Part 1 + Part 2) — Một nguồn chữ UI & mô hình cài đặt dùng chung 3 nền tảng, VI

### Community 675 - "1. Bảng mã sự cố"
Cohesion: 0.22
Nodes (7): lc_assist_idle_arms(), test_assist_idle_arms(), Tests, 1. Nghiệp vụ (hành vi chốt), 3. Kiểm chứng, P6 — Dự đoán văn bản song ngữ (Tab / Ctrl + Space): nghiệp vụ & audit, 1. Bảng mã sự cố

### Community 676 - "[0.2.35] — 2026-10-23"
Cohesion: 0.67
Nodes (3): [0.2.35] — 2026-10-23, Added, Fixed

### Community 677 - "TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV"
Cohesion: 0.40
Nodes (4): 1. Nguyên nhân gốc (Findings — G6, giữ ID), 2. Mô hình composition (quyết định kiến trúc), 3. Giảm heuristic AV (bổ sung `antivirus-false-positive.md` §9), TSF typing overhaul — sửa lỗi "không gõ được tiếng Việt" + giảm heuristic AV

### Community 678 - "ime_imk_op"
Cohesion: 0.40
Nodes (5): ime_imk_op, arg, kind, text_len, text_off

### Community 679 - "[0.2.61] — 2026-11-03"
Cohesion: 0.67
Nodes (3): [0.2.61] — 2026-11-03, Added, Fixed

### Community 680 - "scan_run_key_for_conflicts"
Cohesion: 0.67
Nodes (3): detect_windows_conflict_name(), HKEY, scan_run_key_for_conflicts()

### Community 683 - "ime_imk_plan"
Cohesion: 0.40
Nodes (5): ime_imk_plan, abi_version, arena, count, ops

### Community 684 - "[0.2.63] — 2026-11-04"
Cohesion: 0.50
Nodes (4): [0.2.63] — 2026-11-04, Added, Changed, Fixed

### Community 685 - "Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được"
Cohesion: 0.67
Nodes (3): Ba nguyên nhân gốc, ba fixes, xác minh máy thật, Bản 0.2.12 — hết lỗi nâng cấp DLL khoá + Win+Space store + xác minh máy thật gõ lại được, Phát hành — v0.2.12 đã publish

### Community 686 - "Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath"
Cohesion: 0.67
Nodes (3): Bản 0.2.5 — vá bản portable (gạch chân) + pipeline ký SignPath, Ký số — SignPath opt-in, Phát hành — v0.2.5 đã publish (release candidate, chưa ký số)

### Community 687 - "[0.2.74] — 2026-11-08"
Cohesion: 0.50
Nodes (4): [0.2.74] — 2026-11-08, Added, Changed, Fixed

### Community 698 - "[0.2.75] — 2026-11-08"
Cohesion: 0.50
Nodes (4): [0.2.75] — 2026-11-08, Changed, Fixed, Tests

### Community 699 - "Trạng thái ký số macOS — Developer ID, notarization, stapling"
Cohesion: 0.50
Nodes (4): 1. Tóm tắt, 2. Ảnh hưởng tới người dùng, 4. Việc còn lại để bật (chủ repo), Trạng thái ký số macOS — Developer ID, notarization, stapling

### Community 700 - "ime_assist_result"
Cohesion: 0.50
Nodes (4): ime_assist_result, abi_version, count, issues

### Community 701 - "refresh_style_stats"
Cohesion: 0.50
Nodes (4): refresh_style_stats(), RsiTotals, StyleProfileSettings, style_stats_text()

### Community 702 - "[0.2.16] — 2026-10-03"
Cohesion: 0.67
Nodes (3): [0.2.16] — 2026-10-03, Fixed (Windows) — portable "giải nén chạy ngay" hoạt động trên Win11 24H2+, Verified live trên máy thật (máy chủ repo, build 26300 — trường hợp từng fail)

### Community 703 - "[0.2.47] — 2026-10-26"
Cohesion: 0.67
Nodes (3): [0.2.47] — 2026-10-26, Added, Fixed

### Community 704 - "[0.2.54] — 2026-10-30"
Cohesion: 0.67
Nodes (3): [0.2.54] — 2026-10-30, Added, Fixed

### Community 705 - "[0.2.67] — 2026-11-07"
Cohesion: 0.67
Nodes (3): [0.2.67] — 2026-11-07, Added, Fixed

### Community 706 - "[0.2.70] — 2026-11-08"
Cohesion: 0.67
Nodes (3): [0.2.70] — 2026-11-08, Fixed, Tests

### Community 707 - "with_prefix"
Cohesion: 0.67
Nodes (3): Item, Iterator, with_prefix()

## Knowledge Gaps
- **3343 isolated node(s):** `kind`, `ch`, `text`, `len`, `delete_before` (+3338 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 5256 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **105 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `1. Danh sách lỗi (E-IDs)` connect `engine.c` to `test_delete_surrounding.cpp`, `core/tests/invariants.rs`, `config.rs`, `dual_mode_matrix.rs`, `.register`, `AXSnapshot`, `engine.cpp`, `Problem`, `String`, `syllable_spelling.rs`, `ImeEngine`, `cli/src/assist.rs`, `test_compose.c`, `e2e_ibus.c`, `.suggest`, `trace_diff.rs`, `.commit`, `lc_find_settings_binary`?**
  _High betweenness centrality (0.112) - this node is a cross-community bridge._
- **Why does `String` connect `String` to `tis-select.swift`, `ViState`, `AutostartManager`, `ConfigWatcher`, `KeyOutcome`, `2. Nhật ký finding`, `.suggest`, `CodingKeys`, `AppDelegate`, `StrategyResolver`, `KeyTranslator`, `IpcServer`, `AXSnapshot`, `ConfigStore`, `.register`, `.run`, `OwnerRule`, `.push`, `TextVNInputController`, `SessionShadow`, `IpcClient`, `Foundation`, `Problem`, `EmojiPickerModel`, `ImeEngine`, `MacroEntry`, `MarkedState`, `Equatable`, `.commit`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Why does `Changelog` connect `Changelog` to `[0.2.93] — 2026-11-27`, `[0.2.3] — 2026-10-01`, `[0.2.73] — 2026-11-08`, `[0.2.13] — 2026-10-03`, `[0.2.64] — 2026-11-04`, `[0.2.85] — 2026-11-22`, `[0.2.92] — 2026-11-27`, `[0.2.94] — 2026-11-27`, `[0.2.95] — 2026-11-27`, `README.md`, `[0.2.35] — 2026-10-23`, `[0.2.9] — 2026-10-02`, `[0.2.61] — 2026-11-03`, `engine.cpp`, `[0.2.63] — 2026-11-04`, `[0.2.74] — 2026-11-08`, `2. Nhật ký finding`, `.suggest`, `[0.2.81] — 2026-11-18`, `[0.2.29] — 2026-10-10`, `[0.2.75] — 2026-11-08`, `[0.2.16] — 2026-10-03`, `[0.2.47] — 2026-10-26`, `[0.2.54] — 2026-10-30`, `AXSnapshot`, `[0.2.66] — 2026-11-07`, `[0.2.67] — 2026-11-07`, `.register`, `[0.2.70] — 2026-11-08`, `WndProc`, `[0.2.8] — 2026-10-02`, `[0.2.91] — 2026-11-26`, `.push`, `[0.2.14] — 2026-10-03`, `[0.2.46] — 2026-10-26`, `e2e_fcitx5.c`, `[0.2.28] — 2026-10-10`, `[0.2.10] — 2026-10-03`, `[0.2.79] — 2026-11-14`, `[0.2.77] — 2026-11-12`, `IpcClient`, `Added`, `[0.2.96] — 2026-11-28`, `[0.2.38] — 2026-10-24`, `lc_find_settings_binary`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._
- **Are the 23 inferred relationships involving `TextVNInputController` (e.g. with `.emojiRecordUse()` and `.emojiSearch()`) actually correct?**
  _`TextVNInputController` has 23 INFERRED edges - model-reasoned connections that need verification._
- **What connects `kind`, `ch`, `text` to the rest of the system?**
  _3343 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Ctx` be split into smaller, more focused modules?**
  _Cohesion score 0.05203442879499218 - nodes in this community are weakly interconnected._
- **Should `board/src/lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08018648018648018 - nodes in this community are weakly interconnected._