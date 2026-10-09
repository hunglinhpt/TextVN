// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask/src/win_corpus_cases.rs` — khai báo các test case corpus Windows (WIN-006).

pub struct CorpusCase {
    pub name: &'static str,
    pub content: &'static str,
}

pub const CASES_PART1: &[CorpusCase] = &[
    // --- bug_B1: Address bars & Excel (P1-5 §2) ---
    CorpusCase {
        name: "bug_B1_explorer_address_01.keys",
        content: "\
# corpus/win/bug_B1_explorer_address_01.keys — B1 Explorer path address bar (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,inject_vk,selection
:app explorer.exe field=address_bar
:type \"dduocj\"
:expect \"được\"
",
    },
    CorpusCase {
        name: "bug_B1_brave_address_01.keys",
        content: "\
# corpus/win/bug_B1_brave_address_01.keys — Brave browser address bar (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,inject_vk,selection
:app brave.exe field=address_bar
:type \"tieengs\"
:expect \"tiếng\"
:key Space
:type \"vietj\"
:expect \"tiếng việt\"
",
    },
    CorpusCase {
        name: "bug_B1_excel_formula_02.keys",
        content: "\
# corpus/win/bug_B1_excel_formula_02.keys — B1 Excel formula bar typing (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,inject_vk,selection
:app excel.exe field=editbox
:type \"toongr\"
:expect \"tổng\"
:key Space
:type \"soos\"
:expect \"tổng số\"
",
    },
    CorpusCase {
        name: "bug_B1_edge_search_01.keys",
        content: "\
# corpus/win/bug_B1_edge_search_01.keys — Edge search flyout field (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,inject_vk,selection
:app msedge.exe field=search
:type \"hoaf\"
:expect \"hoà\"
:key Space
:type \"binhf\"
:expect \"hoà bình\"
",
    },
    CorpusCase {
        name: "bug_B1_vivaldi_address_02.keys",
        content: "\
# corpus/win/bug_B1_vivaldi_address_02.keys — Vivaldi browser address bar (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,inject_vk,selection
:app vivaldi.exe field=address_bar
:type \"vieetj\"
:expect \"việt\"
:key Space
:type \"nam\"
:expect \"việt nam\"
",
    },
    // --- bug_B2: Chat enter & space midword (P1-5 §2) ---
    CorpusCase {
        name: "bug_B2_chat_enter_01.keys",
        content: "\
# corpus/win/bug_B2_chat_enter_01.keys — B2 Enter commits chat without duplicating (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app slack.exe field=body
:type \"chaof\"
:expect \"chào\"
:key Space
:type \"banj\"
:expect \"chào bạn\"
:key Enter
:expect \"chào bạn\\n\"
",
    },
    CorpusCase {
        name: "bug_B2_chat_enter_02.keys",
        content: "\
# corpus/win/bug_B2_chat_enter_02.keys — B2 Consecutive chat enters (P1-5 §2)
:config method=telex diacritic_style=new auto_capitalize=false
:caps preedit,selection
:app teams.exe field=body
:type \"xong\"
:expect \"xong\"
:key Enter
:expect \"xong\\n\"
:type \"roofi\"
:expect \"xong\\nrồi\"
",
    },
    CorpusCase {
        name: "bug_B2_space_midword_01.keys",
        content: "\
# corpus/win/bug_B2_space_midword_01.keys — B2 Space midword (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app wordpad.exe field=body
:type \"tieengs\"
:expect \"tiếng\"
:key Space
:type \"vieetj\"
:expect \"tiếng việt\"
",
    },
    CorpusCase {
        name: "bug_B2_space_midword_02.keys",
        content: "\
# corpus/win/bug_B2_space_midword_02.keys — B2 Space commit followed by vowel (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"hocj\"
:expect \"học\"
:key Space
:type \"an\"
:expect \"học an\"
",
    },
    CorpusCase {
        name: "bug_B2_zalo_enter_01.keys",
        content: "\
# corpus/win/bug_B2_zalo_enter_01.keys — Zalo PC chat enter (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app zalo.exe field=body
:type \"chaof\"
:expect \"chào\"
:key Enter
:expect \"chào\\n\"
",
    },
    CorpusCase {
        name: "bug_B2_telegram_enter_02.keys",
        content: "\
# corpus/win/bug_B2_telegram_enter_02.keys — Telegram desktop chat enter (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app telegram.exe field=body
:type \"dzoi\"
:expect \"dzoi\"
:key Enter
:expect \"dzoi\\n\"
",
    },
];

pub const CASES_PART2: &[CorpusCase] = &[
    CorpusCase {
        name: "bug_B2_teams_multiline_01.keys",
        content: "\
# corpus/win/bug_B2_teams_multiline_01.keys — Teams multiline formatting (P1-5 §2)
:config method=telex diacritic_style=new auto_capitalize=false
:caps preedit,selection
:app teams.exe field=body
:type \"mootj\"
:expect \"một\"
:key Enter
:type \"hais\"
:expect \"một\\nhái\"
",
    },
    // --- bug_B3: IDE completion popups (P1-5 §2) ---
    CorpusCase {
        name: "bug_B3_completion_01.keys",
        content: "\
# corpus/win/bug_B3_completion_01.keys — B3 Candidate popup active (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,selection
:app code.exe field=candidate
:type \"dd\"
:expect \"đ\"
",
    },
    CorpusCase {
        name: "bug_B3_jetbrains_01.keys",
        content: "\
# corpus/win/bug_B3_jetbrains_01.keys — B3 JetBrains IntelliJ autocomplete (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,selection
:app idea64.exe field=candidate
:type \"ddang\"
:expect \"đang\"
",
    },
    CorpusCase {
        name: "bug_B3_jetbrains_02.keys",
        content: "\
# corpus/win/bug_B3_jetbrains_02.keys — B3 IDE editor typing outside candidate (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection,field_detect
:app idea64.exe field=body
:type \"bieens\"
:expect \"biến\"
",
    },
    CorpusCase {
        name: "bug_B3_visualstudio_01.keys",
        content: "\
# corpus/win/bug_B3_visualstudio_01.keys — Visual Studio Intellisense popup (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,selection
:app devenv.exe field=candidate
:type \"ddem\"
:expect \"đem\"
",
    },
    CorpusCase {
        name: "bug_B3_rider_candidate_01.keys",
        content: "\
# corpus/win/bug_B3_rider_candidate_01.keys — JetBrains Rider completion popup (P1-5 §2)
:config method=telex diacritic_style=new
:caps field_detect,selection
:app rider64.exe field=candidate
:type \"dd\"
:expect \"đ\"
",
    },
    // --- bug_B6: Hotkeys & shortcuts pass-through (P1-5 §2) ---
    CorpusCase {
        name: "bug_B6_combo_pass_01.keys",
        content: "\
# corpus/win/bug_B6_combo_pass_01.keys — B6 Shortcut combinations pass through (P1-5 §2)
:config method=telex
:combo Ctrl+C
:expect_action PASS
:combo Ctrl+V
:expect_action PASS
:combo Ctrl+Z
:expect_action PASS
",
    },
    CorpusCase {
        name: "bug_B6_combo_pass_02.keys",
        content: "\
# corpus/win/bug_B6_combo_pass_02.keys — B6 Navigation and function shortcuts (P1-5 §2)
:config method=telex
:combo Alt+Tab
:expect_action PASS
:combo Ctrl+Shift+Escape
:expect_action PASS
",
    },
    CorpusCase {
        name: "bug_B6_combo_pass_03.keys",
        content: "\
# corpus/win/bug_B6_combo_pass_03.keys — B6 Windows key shortcuts (P1-5 §2)
:config method=telex
:combo Win+L
:expect_action PASS
:combo Win+Space
:expect_action PASS
",
    },
    CorpusCase {
        name: "bug_B6_system_hotkeys_01.keys",
        content: "\
# corpus/win/bug_B6_system_hotkeys_01.keys — Alt and Win modifier pass-through (P1-5 §2)
:config method=telex
:combo Ctrl+Shift+Space
:expect_action PASS
:combo Ctrl+Alt+Delete
:expect_action PASS
",
    },
    CorpusCase {
        name: "bug_B6_browser_tabs_02.keys",
        content: "\
# corpus/win/bug_B6_browser_tabs_02.keys — Browser tab shortcut passes through (P1-5 §2)
:config method=telex
:combo Ctrl+T
:expect_action PASS
:combo Ctrl+W
:expect_action PASS
",
    },
];

pub const CASES_PART3: &[CorpusCase] = &[
    // --- bug_B7: Electron apps (P1-5 §2) ---
    CorpusCase {
        name: "bug_B7_electron_discord_01.keys",
        content: "\
# corpus/win/bug_B7_electron_discord_01.keys — B7 Discord preedit & commit (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app discord.exe field=body
:type \"chaof\"
:expect \"chào\"
:key Space
:type \"banj\"
:expect \"chào bạn\"
",
    },
    CorpusCase {
        name: "bug_B7_electron_slack_01.keys",
        content: "\
# corpus/win/bug_B7_electron_slack_01.keys — B7 Slack Electron message typing (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app slack.exe field=body
:type \"hom\"
:expect \"hom\"
:key Space
:type \"nay\"
:expect \"hom nay\"
",
    },
    CorpusCase {
        name: "bug_B7_electron_vscode_01.keys",
        content: "\
# corpus/win/bug_B7_electron_vscode_01.keys — B7 VS Code editor preedit (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app code.exe field=body
:type \"chuooxi\"
:expect \"chuỗi\"
",
    },
    CorpusCase {
        name: "bug_B7_notion_01.keys",
        content: "\
# corpus/win/bug_B7_notion_01.keys — Notion Electron desktop app (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notion.exe field=body
:type \"ghi\"
:expect \"ghi\"
:key Space
:type \"chus\"
:expect \"ghi chú\"
",
    },
    CorpusCase {
        name: "bug_B7_slack_thread_01.keys",
        content: "\
# corpus/win/bug_B7_slack_thread_01.keys — Slack thread reply composer (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app slack.exe field=body
:type \"ddoongf\"
:expect \"đồng\"
:key Space
:type \"ys\"
:expect \"đồng ý\"
",
    },
    // --- bug_B8: Terminal & Conhost (P1-5 §2) ---
    CorpusCase {
        name: "bug_B8_terminal_01.keys",
        content: "\
# corpus/win/bug_B8_terminal_01.keys — B8 Windows Terminal ForwardAsCommit (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk,field_detect
:app windowsterminal.exe field=terminal
:type \"dduocj\"
:expect \"được\"
",
    },
    CorpusCase {
        name: "bug_B8_terminal_utf8_02.keys",
        content: "\
# corpus/win/bug_B8_terminal_utf8_02.keys — B8 Terminal UTF-8 text handling (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk,field_detect
:app windowsterminal.exe field=terminal
:type \"chaof\"
:expect \"chào\"
:key Space
:type \"banj\"
:expect \"chào bạn\"
",
    },
    CorpusCase {
        name: "bug_B8_conhost_03.keys",
        content: "\
# corpus/win/bug_B8_conhost_03.keys — B8 Legacy console host conhost.exe (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk,field_detect
:app conhost.exe field=terminal
:type \"thoat\"
:expect \"thoat\"
",
    },
    CorpusCase {
        name: "bug_B8_powershell_01.keys",
        content: "\
# corpus/win/bug_B8_powershell_01.keys — PowerShell 7 Windows Terminal (P1-5 §2)
:config method=telex diacritic_style=new english_words=dir,data
:caps inject_vk,field_detect
:app pwsh.exe field=terminal
:type \"dir\"
:expect \"dỉ\"
:key Space
:expect \"dir \"
:type \"data\"
:expect \"dir data\"
:key Space
:expect \"dir data \"
",
    },
    CorpusCase {
        name: "bug_B8_conhost_clear_01.keys",
        content: "\
# corpus/win/bug_B8_conhost_clear_01.keys — Conhost command line cls (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk,field_detect
:app conhost.exe field=terminal
:type \"cls\"
:expect \"cls\"
:key Enter
:expect \"cls\\n\"
",
    },
    // --- bug_B9: Games & chat (P1-5 §2) ---
    CorpusCase {
        name: "bug_B9_game_chat_01.keys",
        content: "\
# corpus/win/bug_B9_game_chat_01.keys — B9 Game chat active vs WASD (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app valorant.exe field=editbox
:type \"cuwsu\"
:expect \"cứu\"
",
    },
    CorpusCase {
        name: "bug_B9_game_chat_02.keys",
        content: "\
# corpus/win/bug_B9_game_chat_02.keys — B9 Game non-chat field passthrough (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app cs2.exe field=unknown
# Adapter đã nhận diện đây là gameplay ngoài ô chat → tắt context (B9).
:enabled off
:type \"wasd\"
:expect \"wasd\"
:expect_action PASS
",
    },
    CorpusCase {
        name: "bug_B9_league_chat_01.keys",
        content: "\
# corpus/win/bug_B9_league_chat_01.keys — League of Legends in-game chat (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app leagueoflegends.exe field=editbox
:type \"ddanhs\"
:expect \"đánh\"
",
    },
    CorpusCase {
        name: "bug_B9_dota_chat_01.keys",
        content: "\
# corpus/win/bug_B9_dota_chat_01.keys — Dota 2 team chat typing (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app dota2.exe field=editbox
:type \"ddi\"
:expect \"đi\"
",
    },
];
pub const CASES_PART4: &[CorpusCase] = &[
    // --- secure_field: Passwords (S3 / P1-5 §2) ---
    CorpusCase {
        name: "secure_field_keepassxc_01.keys",
        content: "\
# corpus/win/secure_field_keepassxc_01.keys — Secure field KeePassXC password (P1-5 §2)
:config method=telex
:app keepassxc.exe field=secure
:secure on
:type \"P@ssw0rd\"
:expect \"P@ssw0rd\"
:expect_action PASS
:secure off
",
    },
    CorpusCase {
        name: "secure_field_bitwarden_02.keys",
        content: "\
# corpus/win/secure_field_bitwarden_02.keys — Secure master password field (P1-5 §2)
:config method=telex
:app bitwarden.exe field=secure
:secure on
:type \"SecretKey\"
:expect \"SecretKey\"
:expect_action PASS
:secure off
",
    },
    CorpusCase {
        name: "secure_field_1password_01.keys",
        content: "\
# corpus/win/secure_field_1password_01.keys — 1Password Windows vault unlock (P1-5 §2)
:config method=telex
:app 1password.exe field=secure
:secure on
:type \"M@st3rP@ss\"
:expect \"M@st3rP@ss\"
:expect_action PASS
:secure off
",
    },
    CorpusCase {
        name: "secure_field_chrome_pwd_01.keys",
        content: "\
# corpus/win/secure_field_chrome_pwd_01.keys — Chrome HTML password input field (P1-5 §2)
:config method=telex
:app chrome.exe field=secure
:secure on
:type \"SecretPass\"
:expect \"SecretPass\"
:expect_action PASS
:secure off
",
    },
    // --- owner_no_double (P1-5 §2) ---
    CorpusCase {
        name: "owner_no_double_tsf_01.keys",
        content: "\
# corpus/win/owner_no_double_tsf_01.keys — TSF active context owns input (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection,field_detect
:app notepad.exe field=body
:type \"nguwowif\"
:expect \"người\"
",
    },
    CorpusCase {
        name: "owner_no_double_hook_02.keys",
        content: "\
# corpus/win/owner_no_double_hook_02.keys — Fallback hook owns input for legacy edit (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app cmd.exe field=terminal
:type \"thoat\"
:expect \"thoat\"
",
    },
    // --- tsf_preedit: Composition lifecycle (P1-5 §2) ---
    CorpusCase {
        name: "tsf_preedit_commit_space_01.keys",
        content: "\
# corpus/win/tsf_preedit_commit_space_01.keys — TSF commit on space (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"vieetj\"
:expect \"việt\"
:key Space
:expect \"việt \"
",
    },
    CorpusCase {
        name: "tsf_preedit_commit_enter_02.keys",
        content: "\
# corpus/win/tsf_preedit_commit_enter_02.keys — TSF commit on enter (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"nam\"
:expect \"nam\"
:key Enter
:expect \"nam\\n\"
",
    },
    CorpusCase {
        name: "tsf_preedit_backspace_03.keys",
        content: "\
# corpus/win/tsf_preedit_backspace_03.keys — TSF backspace inside preedit (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"dda\"
:expect \"đa\"
:key Backspace
:expect \"đ\"
",
    },
    CorpusCase {
        name: "tsf_preedit_multiletter_04.keys",
        content: "\
# corpus/win/tsf_preedit_multiletter_04.keys — TSF multi-syllable typing (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app word.exe field=body
:type \"tooi\"
:expect \"tôi\"
",
    },
];

pub const CASES_PART5: &[CorpusCase] = &[
    CorpusCase {
        name: "tsf_preedit_punctuation_05.keys",
        content: "\
# corpus/win/tsf_preedit_punctuation_05.keys — Punctuation terminates preedit (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"xong\"
:expect \"xong\"
:type \".\"
:expect \"xong.\"
",
    },
    CorpusCase {
        name: "tsf_preedit_undo_01.keys",
        content: "\
# corpus/win/tsf_preedit_undo_01.keys — TSF marker undo typing (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"ass\"
:expect \"as\"
",
    },
    CorpusCase {
        name: "tsf_preedit_escape_02.keys",
        content: "\
# corpus/win/tsf_preedit_escape_02.keys — TSF cancel via Escape (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"ddang\"
:expect \"đang\"
:key Escape
:expect \"ddang\"
",
    },
    CorpusCase {
        name: "tsf_preedit_double_vowel_01.keys",
        content: "\
# corpus/win/tsf_preedit_double_vowel_01.keys — Circumflex vowel creation in preedit (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"ee\"
:expect \"ê\"
",
    },
    CorpusCase {
        name: "tsf_preedit_horn_vowel_02.keys",
        content: "\
# corpus/win/tsf_preedit_horn_vowel_02.keys — Horn vowel creation via w (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"ow\"
:expect \"ơ\"
",
    },
    CorpusCase {
        name: "tsf_preedit_stroke_d_03.keys",
        content: "\
# corpus/win/tsf_preedit_stroke_d_03.keys — Stroke d via dd (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"dd\"
:expect \"đ\"
",
    },
    CorpusCase {
        name: "tsf_preedit_long_text_01.keys",
        content: "\
# corpus/win/tsf_preedit_long_text_01.keys — Continuous typing sentence (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app word.exe field=body
:type \"chucs\"
:expect \"chúc\"
:key Space
:type \"muwngf\"
:expect \"chúc mừng\"
:key Space
:type \"nawm\"
:expect \"chúc mừng năm\"
",
    },
    CorpusCase {
        name: "tsf_preedit_cursor_mid_01.keys",
        content: "\
# corpus/win/tsf_preedit_cursor_mid_01.keys — Left arrow navigates inside preedit buffer (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"tooi\"
:expect \"tôi\"
:key Left
:type \"x\"
:expect \"tôxi\"
",
    },
    CorpusCase {
        name: "tsf_preedit_accent_switch_02.keys",
        content: "\
# corpus/win/tsf_preedit_accent_switch_02.keys — Switch accent mark in place (P1-5 §2)
:config method=telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"toons\"
:expect \"tốn\"
:type \"f\"
:expect \"tồn\"
",
    },
];

pub const CASES_PART6: &[CorpusCase] = &[
    // --- hook_* (P1-5 §2) ---
    CorpusCase {
        name: "hook_backspace_type_01.keys",
        content: "\
# corpus/win/hook_backspace_type_01.keys — Hook BackspaceType replacement (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app legacy_app.exe field=editbox
:type \"tooi\"
:expect \"tôi\"
",
    },
    CorpusCase {
        name: "hook_selection_replace_02.keys",
        content: "\
# corpus/win/hook_selection_replace_02.keys — Hook SelectionReplace mode (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk,selection
:app legacy_app.exe field=editbox
:type \"dduocj\"
:expect \"được\"
",
    },
    CorpusCase {
        name: "hook_unicode_inject_03.keys",
        content: "\
# corpus/win/hook_unicode_inject_03.keys — Hook pure unicode inject (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app custom_win32.exe field=body
:type \"chaof\"
:expect \"chào\"
",
    },
    CorpusCase {
        name: "hook_vk_then_unicode_04.keys",
        content: "\
# corpus/win/hook_vk_then_unicode_04.keys — Hook vk backspace + unicode (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app custom_win32.exe field=body
:type \"nhaf\"
:expect \"nhà\"
",
    },
    CorpusCase {
        name: "hook_repeat_key_05.keys",
        content: "\
# corpus/win/hook_repeat_key_05.keys — Hook repeat double letter transformation (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app custom_win32.exe field=body
:type \"dd\"
:expect \"đ\"
",
    },
    CorpusCase {
        name: "hook_numeric_pad_01.keys",
        content: "\
# corpus/win/hook_numeric_pad_01.keys — Hook number keys unchanged (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app calc.exe field=editbox
:type \"12345\"
:expect \"12345\"
",
    },
    CorpusCase {
        name: "hook_special_symbols_01.keys",
        content: "\
# corpus/win/hook_special_symbols_01.keys — Special programming characters pass-through (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app code.exe field=body
:type \"[]{}()<>=;\"
:expect \"[]{}()<>=;\"
",
    },
    CorpusCase {
        name: "hook_vk_backspace_series_01.keys",
        content: "\
# corpus/win/hook_vk_backspace_series_01.keys — Sequential backspace deletes (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app legacy.exe field=editbox
:type \"abc\"
:expect \"abc\"
:key Backspace
:expect \"ab\"
:key Backspace
:expect \"a\"
",
    },
    CorpusCase {
        name: "hook_backspace_empty_06.keys",
        content: "\
# corpus/win/hook_backspace_empty_06.keys — Hook Backspace at start of buffer (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app legacy.exe field=editbox
:key Backspace
:expect \"\"
:expect_action PASS
",
    },
    CorpusCase {
        name: "hook_delete_key_07.keys",
        content: "\
# corpus/win/hook_delete_key_07.keys — Hook Delete removes character at cursor (P1-5 §2)
:config method=telex diacritic_style=new
:caps inject_vk
:app legacy.exe field=editbox
:type \"a\"
:expect \"a\"
:key Left
:key Delete
:expect \"\"
",
    },
];

pub const CASES_PART7: &[CorpusCase] = &[
    // --- restore_en & different methods (P1-5 §2) ---
    CorpusCase {
        name: "restore_en_english_app_01.keys",
        content: "\
# corpus/win/restore_en_english_app_01.keys — Auto restore in English context (P1-5 §2)
:config method=telex diacritic_style=new auto_restore_english=true
:caps preedit,selection
:app terminal.exe field=terminal
:type \"asdf\"
:expect \"àd\"
:expect_action RESTORE
:key Space
:expect \"asdf \"
",
    },
    CorpusCase {
        name: "restore_en_code_keyword_02.keys",
        content: "\
# corpus/win/restore_en_code_keyword_02.keys — Code keyword typed in editor (P1-5 §2)
:config method=telex diacritic_style=new auto_restore_english=true english_words=qwert
:caps preedit,selection
:app code.exe field=body
:type \"qwert\"
:expect \"qwẻt\"
:key Space
:expect \"qwert \"
",
    },
    CorpusCase {
        name: "restore_en_powershell_cmdlet_01.keys",
        content: "\
# corpus/win/restore_en_powershell_cmdlet_01.keys — PowerShell cmdlet in terminal (P1-5 §2)
:config method=telex diacritic_style=new english_words=process
:caps inject_vk,field_detect
:app pwsh.exe field=terminal
:type \"Get-Process\"
:expect \"Get-Proces\"
:key Space
:expect \"Get-Process \"
",
    },
    CorpusCase {
        name: "restore_en_subdomain_url_04.keys",
        content: "\
# corpus/win/restore_en_subdomain_url_04.keys — Domain name typing in address bar (P1-5 §2)
:config method=telex diacritic_style=new auto_capitalize=false english_words=docs
:caps field_detect,selection
:app chrome.exe field=address_bar
:type \"docs.rs\"
:expect \"docs.rs\"
",
    },
    CorpusCase {
        name: "vni_win_notepad_01.keys",
        content: "\
# corpus/win/vni_win_notepad_01.keys — VNI input method on Windows Notepad (P1-5 §2)
:config method=vni diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"d9u7o7ng2\"
:expect \"đường\"
",
    },
    CorpusCase {
        name: "viqr_win_notepad_01.keys",
        content: "\
# corpus/win/viqr_win_notepad_01.keys — VIQR input method on Windows Notepad (P1-5 §2)
:config method=viqr diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"ddo^`ng\"
:expect \"đồng\"
",
    },
    CorpusCase {
        name: "simple_telex_win_notepad_01.keys",
        content: "\
# corpus/win/simple_telex_win_notepad_01.keys — Simple Telex on Windows Notepad (P1-5 §2)
# `w` là dấu sừng như UniKey vneHookAll (R2-61 — bản cũ khoá `đươngw`, là lỗi).
:config method=simple_telex diacritic_style=new
:caps preedit,selection
:app notepad.exe field=body
:type \"dduongw\"
:expect \"đương\"
",
    },
];

pub fn all_cases() -> impl Iterator<Item = &'static CorpusCase> {
    CASES_PART1
        .iter()
        .chain(CASES_PART2.iter())
        .chain(CASES_PART3.iter())
        .chain(CASES_PART4.iter())
        .chain(CASES_PART5.iter())
        .chain(CASES_PART6.iter())
        .chain(CASES_PART7.iter())
}
