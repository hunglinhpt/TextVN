# Sổ hồi quy — mã lỗi đã sửa ↔ bằng chứng chống tái diễn (REG-1)

> Gate: `cargo xtask regression-index` (preflight + `repo-hygiene`). Mỗi mã trong `project-common-errors.md` (E*),
> `win-test-common-errors.md` (`win/…`), `release-process.md` (R*) và mọi `R2-*` trong `CHANGELOG.md` phải có dòng ở đây với
> ≥ 1 bằng chứng: `test:<tệp>::<hàm>` (xoá/đổi tên test ⇒ đỏ), `file:<corpus>`, `gate:<tệp xtask/script/workflow còn nhắc mã>`,
> `lesson:` (chỉ `win/*` — bài học harness, không phải hành vi sản phẩm), `process:<mô tả>` (mã chỉ-quy-trình),
> hoặc `todo:REG-2` (nợ — số dòng todo không được vượt `todo-max`, REG-2 giảm dần). Nhiều bằng chứng cách nhau ` · `.
> Mã mới: `cargo xtask regression-index --bootstrap --merge` thêm dòng (tự tìm test nhắc mã), rồi sửa tay cho đúng.

<!-- regression-index: todo-max = 81 -->

| Mã | Nguồn | Bằng chứng |
|---|---|---|
| E1 | project-common-errors | `todo:REG-2` |
| E10 | project-common-errors | `todo:REG-2` |
| E11 | project-common-errors | `test:tray/src/lib.rs::global_toggle_claim_debounces_within_window` · `gate:xtask/src/preflight.rs` |
| E12 | project-common-errors | `todo:REG-2` |
| E13 | project-common-errors | `gate:xtask/src/hygiene.rs` |
| E14 | project-common-errors | `todo:REG-2` |
| E15 | project-common-errors | `todo:REG-2` |
| E16 | project-common-errors | `todo:REG-2` |
| E17 | project-common-errors | `todo:REG-2` |
| E18 | project-common-errors | `gate:tools/win/validate-store-package.ps1` · `gate:xtask/src/preflight.rs` |
| E19 | project-common-errors | `todo:REG-2` |
| E2 | project-common-errors | `todo:REG-2` |
| E20 | project-common-errors | `todo:REG-2` |
| E21 | project-common-errors | `gate:.github/workflows/store-publish.yml` |
| E22 | project-common-errors | `gate:.github/workflows/store-publish.yml` |
| E23 | project-common-errors | `todo:REG-2` |
| E24 | project-common-errors | `todo:REG-2` |
| E25 | project-common-errors | `todo:REG-2` |
| E26 | project-common-errors | `todo:REG-2` |
| E27 | project-common-errors | `gate:tools/win/e2e-assist-p6.ps1` |
| E28 | project-common-errors | `test:core/src/method/viqr.rs::freeze_marker_lap` |
| E29 | project-common-errors | `todo:REG-2` |
| E3 | project-common-errors | `todo:REG-2` |
| E30 | project-common-errors | `todo:REG-2` |
| E31 | project-common-errors | `todo:REG-2` |
| E32 | project-common-errors | `todo:REG-2` |
| E33 | project-common-errors | `todo:REG-2` |
| E34 | project-common-errors | `test:assist/tests/abnormal.rs::v41_collocation_batch2_bat_dung` · `test:assist/tests/abnormal.rs::v41_nghia_song_khong_bao` · `test:assist/tests/abnormal.rs::v41b3_collocation_bat_dung_va_e34` |
| E35 | project-common-errors | `todo:REG-2` |
| E36 | project-common-errors | `todo:REG-2` |
| E37 | project-common-errors | `todo:REG-2` |
| E38 | project-common-errors | `todo:REG-2` |
| E39 | project-common-errors | `todo:REG-2` |
| E4 | project-common-errors | `todo:REG-2` |
| E40 | project-common-errors | `todo:REG-2` |
| E41 | project-common-errors | `todo:REG-2` |
| E42 | project-common-errors | `todo:REG-2` |
| E43 | project-common-errors | `todo:REG-2` |
| E44 | project-common-errors | `gate:xtask/src/assist_tools.rs` |
| E45 | project-common-errors | `todo:REG-2` |
| E46 | project-common-errors | `test:adapters/windows-tsf/src/compose.rs::hook_owned_yield_covers_whole_appdb_and_is_terminal` |
| E47 | project-common-errors | `test:core/tests/dual_mode_matrix.rs::suite_b2_backspace_after_transform_wysiwyg` · `test:core/tests/dual_mode_matrix.rs::suite_b3_e47_regression_pinned` · `test:core/tests/dual_mode_rapid.rs::suite_g2_multi_backspace_wysiwyg_pinned` |
| E48 | project-common-errors | `test:adapters/windows-hook/src/lib.rs::e48_classify_origin_and_keyboard_state_truth_table` · `test:adapters/windows-hook/tests/ime_conflict_1000.rs::e48_hook_origin_guard_prevents_textvn_duplication` · `test:adapters/windows-hook/tests/ime_conflict_1000.rs::e48_dual_mode_rapid_origin_matrix_140` |
| E49 | project-common-errors | `test:adapters/windows-tsf/src/compose.rs::e49_window_class_and_context_yield_matrix` · `test:adapters/windows-tsf/src/compose.rs::e49_tsf_focus_terminal_state_machine_528` · `test:adapters/windows-tsf/tests/non_text_focus_gate.rs::hook_owned_and_taskbar_root_are_gated_before_the_edit_session` |
| E5 | project-common-errors | `todo:REG-2` |
| E50 | project-common-errors | `test:core/tests/dual_mode_matrix.rs::suite_d_undo_tone_and_double_key_golden` · `test:core/tests/dual_mode_rapid.rs::e50_undo_repetition_and_texxtvn_matrix` · `test:core/tests/dual_mode_rapid.rs::e50_identifier_boundary_never_re_restores` |
| E51 | project-common-errors | `todo:REG-2` |
| E52 | project-common-errors | `todo:REG-2` |
| E53 | project-common-errors | `todo:REG-2` |
| E54 | project-common-errors | `todo:REG-2` |
| E55 | project-common-errors | `todo:REG-2` |
| E56 | project-common-errors | `todo:REG-2` |
| E57 | project-common-errors | `todo:REG-2` |
| E58 | project-common-errors | `test:math/src/lexicon.rs::elements_symbols_resolve_as_alias` · `test:math/src/templates.rs::hpt_and_cases` |
| E59 | project-common-errors | `test:xtask/src/en_office.rs::body_matches_committed_tsv` · `gate:.github/workflows/ci-linux.yml` · `gate:.github/workflows/ci-macos.yml` |
| E6 | project-common-errors | `todo:REG-2` |
| E60 | project-common-errors | `test:core/tests/e50_unikey_restore.rs::e60_sir_and_r_undo_matrix` · `test:core/tests/e50_unikey_restore.rs::e60_s_and_f_undo_matrix` · `test:core/tests/e50_unikey_restore.rs::e60_post_consonant_unikey_undo` |
| E61 | project-common-errors | `test:core/src/post/restore_en.rs::e61_extra_and_tech_tables_strictly_sorted` · `test:core/tests/dual_mode_matrix.rs::suite_d_undo_tone_and_double_key_golden` · `test:core/tests/e50_unikey_restore.rs::e61_tone_strip_z_and_vni_0_matrix` |
| E62 | project-common-errors | `test:adapters/linux-common/tests/test_compose.c::test_e62_math_plan` · `test:adapters/windows-hook/tests/ime_conflict_1000.rs::e62_hook_math_consumed_and_cursor_left` · `test:adapters/windows-tsf/src/compose.rs::e62_math_consumed_pass_altgr_and_enter_replace` |
| E63 | project-common-errors | `test:core/tests/e63_expert_personas_audit.rs::view3_scientific_dev_terms_continuations_and_unikey_habits` |
| E64 | project-common-errors | `gate:scripts/test-linux-package.sh` |
| E65 | project-common-errors | `test:cli/src/register.rs::e65_layout_install_accepts_en_or_vi_only` · `test:tray/src/settings_dialog.rs::button_row_has_icons_hints_and_no_tsf_button` · `test:tray/src/settings_dialog.rs::register_log_tail_reads_last_lines` |
| E66 | project-common-errors | `test:cli/src/register.rs::e66_portable_writes_hkcu_ctf_fallback_even_with_stale_hklm` · `gate:xtask/src/win_tools.rs` |
| E67 | project-common-errors | `test:xtask/src/linux_tools.rs::e67_check_elf_abi_accepts_glibc_228_and_blocks_newer_or_execstack` · `gate:.github/workflows/ci-linux.yml` · `gate:scripts/build-linux-el8.sh` |
| E68 | project-common-errors | `gate:.github/workflows/ci-linux.yml` · `gate:scripts/build-linux-compat.sh` |
| E69 | project-common-errors | `test:cli/src/doctor.rs::redact_paths_replaces_username` · `test:cli/src/doctor.rs::e69_linux_im_env_mismatch_detection` · `test:cli/src/doctor.rs::e69_fcitx5_profile_conflicts_detection` · `gate:scripts/test-linux-package.sh` |
| E7 | project-common-errors | `todo:REG-2` |
| E70 | project-common-errors | `test:cli/src/doctor.rs::e70_detect_other_vi_tips_from_user_profile_values` · `test:cli/src/doctor.rs::e70_detect_apple_vi_input_sources` · `test:cli/src/doctor.rs::e70_classify_distribution_all_os` · `gate:.github/workflows/ci-linux.yml` · `gate:scripts/test-linux-smoke.sh` |
| E71 | project-common-errors | `gate:.github/workflows/ci-linux.yml` |
| E72 | project-common-errors | `test:tray/src/settings_dialog.rs::help_window_links_match_every_platform` · `test:tray/src/settings_dialog.rs::button_row_has_icons_hints_and_no_tsf_button` |
| E73 | project-common-errors | `todo:REG-2` |
| E74 | project-common-errors | `todo:REG-2` |
| E75 | project-common-errors | `todo:REG-2` |
| E76 | project-common-errors | `todo:REG-2` |
| E77 | project-common-errors | `todo:REG-2` |
| E78 | project-common-errors | `todo:REG-2` |
| E79 | project-common-errors | `todo:REG-2` |
| E8 | project-common-errors | `todo:REG-2` |
| E80 | project-common-errors | `todo:REG-2` |
| E81 | project-common-errors | `todo:REG-2` |
| E82 | project-common-errors | `test:tray/src/ipc_server.rs::toggle_global_responds_with_authoritative_snapshot` · `test:tray/src/sandbox_e2e.rs::sandbox_appcontainer_end_to_end` |
| E83 | project-common-errors | `gate:xtask/src/main.rs` |
| E84 | project-common-errors | `test:core/tests/invariants.rs::e84_du_lieu_am_tiet_vn_khong_chua_tu_en_ngoai_tu_dien` · `test:core/tests/text_prediction.rs::rsi_7_cong_ngon_ngu_chan_en_cho_tien_to_viet_hop_le` · `gate:xtask/src/gen_vocab.rs` |
| E85 | project-common-errors | `todo:REG-2` |
| E86 | project-common-errors | `gate:.github/workflows/ci-linux.yml` · `gate:.github/workflows/ci-shared.yml` |
| E9 | project-common-errors | `todo:REG-2` |
| R1 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R2 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R2-100 | CHANGELOG | `test:cli/src/register.rs::languages_added_by_textvn_are_tracked_and_removed_only_when_unused` |
| R2-101 | CHANGELOG | `todo:REG-2` |
| R2-102 | CHANGELOG | `todo:REG-2` |
| R2-103 | CHANGELOG | `todo:REG-2` |
| R2-104 | CHANGELOG | `test:cli/src/register.rs::user_hive_paths_are_relative_to_hkey_users` · `test:cli/src/register.rs::hkey_users_relative_paths_reach_the_loaded_hive` |
| R2-105 | CHANGELOG | `todo:REG-2` |
| R2-106 | CHANGELOG | `test:tray/src/settings_dialog.rs::labels_match_linux_settings_panel` |
| R2-107 | CHANGELOG | `todo:REG-2` |
| R2-108 | CHANGELOG | `gate:scripts/test-linux-package.sh` |
| R2-109 | CHANGELOG | `test:tray/src/ipc_server.rs::serves_every_pipe_name` |
| R2-110 | CHANGELOG | `test:ipc/src/sandbox.rs::server_image_must_be_textvn_exe_by_file_name` |
| R2-111 | CHANGELOG | `todo:REG-2` |
| R2-112 | CHANGELOG | `gate:xtask/src/hygiene.rs` |
| R2-113 | CHANGELOG | `todo:REG-2` |
| R2-114 | CHANGELOG | `test:tray/src/lib.rs::no_poison_panicking_lock_unwrap_in_tray` |
| R2-115 | CHANGELOG | `test:cli/src/register.rs::runonce_cleanup_command_respects_260_chars` · `file:corpus/assist/vi_docs_clean.ass` |
| R2-117 | CHANGELOG | `gate:.github/workflows/ci-aur.yml` |
| R2-118 | CHANGELOG | `gate:tools/release/publish-aur.sh` |
| R2-119 | CHANGELOG | `gate:tools/release/prepare-ppa.sh` |
| R2-120 | CHANGELOG | `test:core/src/post/predict.rs::cum_noi_tiep_giu_chu_hoa_chuan` · `test:core/src/post/predict.rs::khong_du_doan_qua_dong_moi` · `test:core/tests/text_prediction.rs::r2_120_ngu_canh_nhieu_tu_truoc_hoan_tat_tien_to` · `gate:tools/win/e2e-assist-p6.ps1` |
| R2-120a | CHANGELOG | `todo:REG-2` |
| R2-120g | CHANGELOG | `todo:REG-2` |
| R2-120h | CHANGELOG | `todo:REG-2` |
| R2-121 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_plan_edge_cases` · `test:adapters/windows-tsf/src/compose.rs::ctrl_space_plan_never_inserts_space_on_pass` · `test:core/tests/text_prediction.rs::r2_121_ctrl_space_khong_khop_nha_tu_giu_cau` · `gate:tools/win/e2e-assist-p6.ps1` |
| R2-122 | CHANGELOG | `test:adapters/windows-tsf/src/assist.rs::phim_overlay_theo_p4_0_4_2` · `gate:tools/win/e2e-assist-apps.ps1` · `gate:tools/win/e2e-assist-p6.ps1` |
| R2-123 | CHANGELOG | `todo:REG-2` |
| R2-124 | CHANGELOG | `todo:REG-2` |
| R2-125 | CHANGELOG | `test:adapters/windows-tsf/src/overlay.rs::v49_panel_khong_che_dong_caret_khi_caret_ganh_dinh` · `test:adapters/windows-tsf/src/overlay.rs::r2_125_caret_rect_be_rong_0_van_neo_duoc` |
| R2-126 | CHANGELOG | `test:tray/src/assist.rs::mode_off_never_notifies` · `test:tray/src/assist.rs::only_auto_fix_shows_a_toast` |
| R2-127 | CHANGELOG | `todo:REG-2` |
| R2-128 | CHANGELOG | `todo:REG-2` |
| R2-129 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_assist_mode_semantics` · `test:adapters/windows-tsf/src/assist.rs::phim_overlay_theo_p4_0_4_2` · `gate:tools/win/e2e-assist-p6.ps1` |
| R2-130 | CHANGELOG | `todo:REG-2` |
| R2-131 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_assist_hotkey_explicit_preview` · `test:adapters/macos-imk/Tests/IMKLibTests/AssistTests.swift::testPlanSentenceAutoFixSkipsNonAutoFixableSpelling` · `test:adapters/windows-tsf/src/assist.rs::r2_131_a2_sua_loi_chinh_ta_khi_nguoi_dung_chon` · `gate:tools/win/e2e-assist-apps.ps1` · `gate:tools/win/e2e-assist-p6.ps1` |
| R2-132 | CHANGELOG | `todo:REG-2` |
| R2-133 | CHANGELOG | `test:adapters/macos-imk/Tests/IMKLibTests/IMKLibTests.swift::testCtrlSpaceExclusionFromChord` |
| R2-134 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_assist_hotkey_explicit_preview` · `test:core/tests/text_prediction.rs::r2_134_peek_completion_xem_truoc_dung_phan_tab_noi_them` · `gate:tools/win/e2e-assist-apps.ps1` |
| R2-135 | CHANGELOG | `todo:REG-2` |
| R2-136 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_assist_hotkey_explicit_preview` · `gate:tools/win/e2e-assist-apps.ps1` |
| R2-137 | CHANGELOG | `todo:REG-2` |
| R2-138 | CHANGELOG | `test:tray/src/lib.rs::r2_138_nhac_mo_lai_ung_dung_sau_nang_cap` |
| R2-139 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_transcode_hotkey` · `test:adapters/linux-common/tests/test_compose.c::test_transcode_selection_slice` · `test:core/tests/selection_convert.rs::r2_139_khoi_phuc_chu_la_cp1252_cp1258_va_ma_hoa_doi` · `gate:tools/win/e2e-assist-apps.ps1` |
| R2-140 | CHANGELOG | `test:core/src/lib.rs::telex_bracket_keys_and_double_bracket_escape_in_engine` · `test:core/tests/audit_r2_140.rs::r2_140_code_va_markdown_giu_nguyen_ngoac` · `test:core/tests/audit_r2_140.rs::r2_140_chu_viet_trong_ngoac_van_go_duoc` |
| R2-141 | CHANGELOG | `test:core/tests/audit_r2_140.rs::r2_141_code_cli_markdown_khong_bi_doi` · `test:core/tests/audit_r2_140.rs::r2_141_tra_lai_giu_enter_tab_la_phim_that` · `test:core/tests/audit_r2_140.rs::r2_141_co_het_han_sau_mot_phim` |
| R2-142 | CHANGELOG | `test:assist/tests/audit_r2_142.rs::cap_lien_tu_tuyen_tinh_tren_cau_dai` |
| R2-146 | CHANGELOG | `test:core/tests/audit_r2_140.rs::r2_146_tu_dung_nhieu_song_qua_loat_tu_moi` · `test:core/tests/audit_r2_140.rs::r2_146_tu_moi_lien_tuc_van_giam_trong_so_dinh_ky` · `test:core/tests/style_learning_v59.rs::trigram_khong_hoc_qua_dau_cau` |
| R2-148 | CHANGELOG | `test:core/tests/audit_r2_140.rs::r2_148_cum_khong_vat_qua_xuong_dong` · `test:core/tests/audit_r2_140.rs::r2_148_cum_can_khoang_trang_that` · `test:core/tests/audit_r2_140.rs::r2_148_drain_uu_tien_tu_go_nhieu_va_khong_mat_phan_con_lai` |
| R2-150 | CHANGELOG | `todo:REG-2` |
| R2-151 | CHANGELOG | `todo:REG-2` |
| R2-152 | CHANGELOG | `test:ffi/src/lib.rs::r2_152_persist_ghi_file_giu_khoa_khac_va_xoa_pending` · `test:ffi/src/lib.rs::r2_152_persist_loi_giu_pending_va_tham_so_hong` |
| R2-153 | CHANGELOG | `todo:REG-2` |
| R2-154 | CHANGELOG | `test:adapters/windows-tsf/tests/idle_popup_routing.rs::khong_co_edit_cookie_gia_trong_ma_nguon` · `gate:tools/win/e2e-assist-apps.ps1` |
| R2-155 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_assist_idle_arms` · `test:adapters/macos-imk/Tests/IMKLibTests/IMKLibTests.swift::testAssistIdleArmsBangPhim` |
| R2-156 | CHANGELOG | `todo:REG-2` |
| R2-157 | CHANGELOG | `todo:REG-2` |
| R2-158 | CHANGELOG | `todo:REG-2` |
| R2-159 | CHANGELOG | `test:adapters/windows-tsf/src/compose.rs::non_text_window_classes_are_recognised` · `gate:tools/win/e2e-non-text-focus.ps1` |
| R2-161 | CHANGELOG | `test:adapters/windows-tsf/src/compose.rs::non_text_top_level_classes` · `test:adapters/windows-tsf/src/compose.rs::hook_owned_terminals_are_yielded` · `test:adapters/windows-tsf/src/compose.rs::hook_owned_yield_covers_whole_appdb_and_is_terminal` · `gate:tools/win/e2e-non-text-focus.ps1` |
| R2-162 | CHANGELOG | `todo:REG-2` |
| R2-163 | CHANGELOG | `gate:.github/workflows/ci-macos.yml` |
| R2-164 | CHANGELOG | `todo:REG-2` |
| R2-165 | CHANGELOG | `test:session/tests/invariants.rs::thuoc_tinh_moi_event_toi_da_1_xoa_1_commit` · `test:session/tests/invariants.rs::commands_tu_choi_2_commit` · `test:session/tests/invariants.rs::commands_tu_choi_2_xoa` |
| R2-166 | CHANGELOG | `test:cli/src/doctor.rs::r2_166_mutter_forward_key_bug_range` |
| R2-167 | CHANGELOG | `test:session/tests/invariants.rs::thuoc_tinh_commit_truoc_khi_an_preedit` · `test:session/tests/invariants.rs::commands_cho_phep_commit_truoc_an_preedit` · `test:session/tests/invariants.rs::commands_tu_choi_commit_sau_an_preedit` |
| R2-168 | CHANGELOG | `test:core/tests/r2_168_unikey_oracle_linux.rs::r2_168_170_matches_unikey_oracle_or_documented_policy` · `test:core/tests/r2_168_unikey_oracle_linux.rs::r2_168_digit_after_space_keeps_word` · `test:core/tests/r2_168_unikey_oracle_linux.rs::r2_168_169_respect_auto_restore_switch` · `file:corpus/shared/r2_168_unikey_oracle_01.keys` |
| R2-169 | CHANGELOG | `test:core/src/validate.rs::oo_nucleus_needs_ng_or_c` · `file:corpus/shared/r2_168_unikey_oracle_01.keys` |
| R2-170 | CHANGELOG | `file:corpus/shared/r2_168_unikey_oracle_01.keys` |
| R2-171 | CHANGELOG | `todo:REG-2` |
| R2-172 | CHANGELOG | `test:core/src/validate.rs::place_name_k_coda_and_kr_onset` · `test:core/tests/r2_171_caps_after_enter_reset.rs::r2_172_place_names_type_in_both_tone_orders` · `test:core/tests/r2_171_caps_after_enter_reset.rs::r2_172_english_words_untouched` · `file:corpus/shared/r2_172_dak_lak_01.keys` |
| R2-173 | CHANGELOG | `test:adapters/windows-tsf/src/edit_session.rs::r2_173_ghost_waits_for_key_up_only_for_passed_char_keys` · `test:core/tests/r2_173_prediction_chain.rs::tab_accept_then_ghost_shows_continuation_and_tab_inserts_it` · `test:core/tests/r2_173_prediction_chain.rs::greeting_and_office_contexts_chain` |
| R2-174 | CHANGELOG | `test:core/tests/dream_rsi_100.rs::d_space_accept_savings` · `test:core/tests/rca_v57_adversarial.rs::adv5_drain_2_lan_chi_pending_moi` · `test:core/tests/text_prediction.rs::r2_174_space_complete_khong_viet_lai_tu_da_dung` |
| R2-26 | CHANGELOG | `test:cli/src/doctor.rs::r2_26_sandbox_mirror_verdict` · `test:config/src/sandbox.rs::sandbox_keys_cover_schema_and_code` · `test:tray/src/ipc_server.rs::sandbox_policy_allows_only_minimal_commands` |
| R2-28 | CHANGELOG | `test:adapters/macos-imk/Tests/IMKLibTests/IMKLibTests.swift::testViStateSnapshotAndPerAppOverrides` · `test:tray/src/svc.rs::effective_enabled_matches_tip_rule` |
| R2-55 | CHANGELOG | `test:adapters/linux-common/tests/test_compose.c::test_typing` · `test:adapters/windows-tsf/src/compose.rs::backspace_inside_word_refolds_and_empty_word_ends_composition` · `test:adapters/windows-tsf/src/compose.rs::backspace_in_composition_matrix` · `file:corpus/mac/imk_preedit_telex_backspace_fold_21.keys` · `file:corpus/mac/mac_bs_type_telex_backspace_19.keys` · `gate:xtask/src/mac_corpus_cases.rs` |
| R2-56 | CHANGELOG | `test:core/src/lib.rs::vn_tab_complete` · `test:core/src/post/restore_en.rs::r2_4_vn_ascii_prefix_not_completed_to_en` · `test:core/src/post/restore_en.rs::am_tiet_viet_that_khong_bi_restore` · `file:corpus/shared/r2_restore_vn_syllables_01.keys` |
| R2-88 | CHANGELOG | `todo:REG-2` |
| R2-95 | CHANGELOG | `test:xtask/src/win_tools.rs::verify_msix_bat_loi_tung_luat` · `gate:tools/win/build-msix.ps1` · `gate:xtask/src/win_tools.rs` |
| R2-98 | CHANGELOG | `test:tray/src/autostart.rs::autostart_effective_honours_per_user_disable_of_machine_entry` |
| R2-99 | CHANGELOG | `file:corpus/assist/vi_docs_clean.ass` |
| R3 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R4 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R5 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R6 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R7 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| R8 | release-process | `process:sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md` |
| win/A1 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A10 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A11 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A12 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A13 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A14 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A15 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A16 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A4 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A5 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A6 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A7 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A8 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/A9 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B1 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B10 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B11 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B12 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B13 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B14 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B15 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B16 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B18 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B19 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B20 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B21 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B22 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B4 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B5 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B6 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B7 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B8 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/B9 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C4 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C5 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C6 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/C7 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/D1 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/D2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/D3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E1 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E4 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E5 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E6 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/E7 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/F1 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/F2 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
| win/F3 | win-test-common-errors | `lesson:docs/specs/win-test-common-errors.md` |
