// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask/src/mac_corpus_cases.rs` — khai báo các test case corpus macOS (MAC-006, P2-5 §2).
//!
//! Mọi chuỗi Telex/VNI/VIQR đều đã verify bằng replay thật (`replay --adapter mac`)
//! trước khi ghi vào đây — expect sai là lỗi dữ liệu, không phải lỗi engine.
//!
//! Profiles (P2-1 §1 + P2-3 §1):
//! - IMK mặc định: `preedit,selection,field_detect` (profile `--adapter mac`) — không ghi `:caps`.
//! - BackspaceType: `:caps field_detect` (mất preedit+selection → downgrade P0-3 §3.1).
//! - Tap opt-in: `:caps selection,field_detect,inject_vk` (không PREEDIT — P2-2 §5).

#[derive(Debug)]
pub struct CorpusCase {
    pub name: String,
    pub content: String,
}

fn case(name: &'static str, desc: &str, body: &str) -> CorpusCase {
    CorpusCase {
        name: format!("{name}.keys"),
        content: format!(
            "# corpus/mac/{name} — {desc} (P2-5 §2)\n{}\n",
            body.trim_matches('\n')
        ),
    }
}

/// IMK default caps — không ghi `:caps` (profile `--adapter mac` = Preedit+Selection+FieldDetect).
fn imk(name: &'static str, desc: &str, app: &str, field: &str, body: &str) -> CorpusCase {
    let full =
        format!(":config method=telex diacritic_style=new\n:app {app} field={field}\n{body}");
    case(name, desc, &full)
}
/// BackspaceType — chỉ field_detect: downgrade P0-3 §3.1 → xóa-bằng-backspace (MAC-015).
fn bs(name: &'static str, desc: &str, app: &str, field: &str, body: &str) -> CorpusCase {
    let full = format!(
        ":config method=telex diacritic_style=new\n:caps field_detect\n:app {app} field={field}\n{body}"
    );
    case(name, desc, &full)
}

/// Tap opt-in — caps theo P2-2 §5 (SELECTION|FIELD_DETECT|INJECT_VK, không PREEDIT).
fn tap(name: &'static str, desc: &str, app: &str, field: &str, body: &str) -> CorpusCase {
    let full = format!(
        ":config method=telex diacritic_style=new\n:caps selection,field_detect,inject_vk\n:app {app} field={field}\n{body}"
    );
    case(name, desc, &full)
}

// 114 case khai báo tuyến tính — push tuần tự dễ đọc hơn `vec![]` khổng lồ.
#[allow(clippy::vec_init_then_push)]
pub fn all_cases() -> Vec<CorpusCase> {
    let mut v: Vec<CorpusCase> = Vec::new();

    // ───────────────────────── bug_B1 — URL bar / Spotlight / Excel (P2-3 §3) ─────────────────────────
    v.push(imk(
        "bug_B1_safari_url_01",
        "B1 Safari address bar — SelectionReplace, không backspace ảo",
        "com.apple.safari",
        "address_bar",
        r#"
:type "vieetj"
:expect "việt"
:key Space
:type "nam"
:expect "việt nam""#,
    ));
    v.push(imk(
        "bug_B1_chrome_url_01",
        "B1 Chrome address bar (com.google.chrome)",
        "com.google.chrome",
        "address_bar",
        r#"
:type "tieengs"
:expect "tiếng"
:key Space
:type "vietj"
:expect "tiếng việt""#,
    ));
    v.push(imk(
        "bug_B1_spotlight_01",
        "B1 Spotlight search field",
        "com.apple.spotlight",
        "search",
        r#"
:type "hoaf"
:expect "hoà"
:key Space
:type "binhf"
:expect "hoà bình""#,
    ));
    v.push(imk(
        "bug_B1_excel_cell_01",
        "B1 Excel cell autocomplete (com.microsoft.excel)",
        "com.microsoft.excel",
        "editbox",
        r#"
:type "tieenf"
:expect "tiền"
:key Space
:type "luong"
:expect "tiền lương""#,
    ));

    // ───────────────────────── bug_B2 — Enter trong chat không nhân từ ─────────────────────────
    v.push(imk(
        "bug_B2_imessage_enter_01",
        "B2 Messages (com.apple.ichat): Enter ×3 không lặp từ",
        "com.apple.ichat",
        "body",
        r#"
:type "chaof"
:expect "chào"
:key Space
:type "banj"
:expect "chào bạn"
:key Enter
:expect "chào bạn\n"
:key Enter
:expect "chào bạn\n\n"
:key Enter
:expect "chào bạn\n\n\n""#,
    ));
    v.push(imk(
        "bug_B2_slack_enter_01",
        "B2 Slack (com.tinyspeck.slackmacgap) Enter commit",
        "com.tinyspeck.slackmacgap",
        "body",
        r#"
:config method=telex diacritic_style=new auto_capitalize=false
:type "xong"
:expect "xong"
:key Enter
:type "roofi"
:expect "xong\nrồi""#,
    ));
    v.push(imk(
        "bug_B2_space_midword_01",
        "B2 Space giữa từ commit ngay (marked ngắn — B11)",
        "com.apple.textedit",
        "body",
        r#"
:type "tieengs"
:expect "tiếng"
:expect_preedit "tiếng"
:key Space
:expect "tiếng "
:expect_preedit ""
:type "vietj"
:expect "tiếng việt""#,
    ));

    // ───────────────────────── bug_B3 — candidate popup (Xcode/JetBrains) ─────────────────────────
    v.push(imk(
        "bug_B3_xcode_completion_01",
        "B3 Xcode completion (role=candidate → SelectionReplace)",
        "com.apple.dt.xcode",
        "candidate",
        r#"
:type "ddang"
:expect "đang""#,
    ));
    v.push(imk(
        "bug_B3_jetbrains_completion_01",
        "B3 JetBrains candidate (vk_then_unicode cho tap; imk vẫn SelectionReplace)",
        "com.jetbrains.intellij",
        "candidate",
        r#"
:type "tieenf"
:expect "tiền""#,
    ));

    // ───────────────────────── bug_B8 — Terminal ForwardAsCommit ─────────────────────────
    v.push(imk(
        "bug_B8_terminal_01",
        "B8 Terminal.app ForwardAsCommit + UTF-8",
        "com.apple.terminal",
        "terminal",
        r#"
:type "dduocj"
:expect "được""#,
    ));
    v.push(imk(
        "bug_B8_iterm2_02",
        "B8 iTerm2 ForwardAsCommit, không xóa (P2-3 §3 #11)",
        "com.googlecode.iterm2",
        "terminal",
        r#"
:type "chaof"
:expect "chào"
:key Space
:type "banj"
:expect "chào bạn"
:key Enter
:expect "chào bạn\n""#,
    ));

    // ───────────────────────── bug_B11 — marked text ngắn, commit tức thì ─────────────────────────
    v.push(imk(
        "bug_B11_marked_commit_space_01",
        "B11 Space kết thúc từ → commit ngay, marked không sống qua Space",
        "com.apple.notes",
        "textarea",
        r#"
:type "chaof"
:expect "chào"
:expect_preedit "chào"
:key Space
:expect "chào "
:expect_preedit """#,
    ));
    v.push(imk(
        "bug_B11_marked_commit_enter_02",
        "B11 Enter commit marked (không giữ marked qua Enter)",
        "com.apple.textedit",
        "body",
        r#"
:type "dduocj"
:expect "được"
:expect_preedit "được"
:key Enter
:expect "được\n"
:expect_preedit """#,
    ));

    // ───────────────────────── bug_B13 — focus loss: commit-before-hide + reset sạch ─────────────────────────
    v.push(imk(
        "bug_B13_focus_loss_01",
        "B13 deactivateServer: adapter commit marked rồi ime_reset — từ sau reset phải sạch",
        "com.apple.textedit",
        "body",
        r#"
:type "chaof"
:expect "chào"
:expect_preedit "chào"
:reset
:type "rooif"
:expect "chàorồi"
:expect_preedit "rồi""#,
    ));
    v.push(imk(
        "bug_B13_focus_loss_02",
        "B13 focus đổi giữa 2 ô: không còn dấu dư từ raw cũ",
        "com.apple.notes",
        "textarea",
        r#"
:type "ddang"
:expect "đang"
:expect_preedit "đang"
:reset
:type "dduocj"
:expect "đangđược"
:expect_preedit "được"
"#,
    ));

    // ───────────────────────── secure field — S3: mật khẩu PASS tuyệt đối ─────────────────────────
    v.push(imk(
        "secure_field_passthrough_01",
        "S3 secure input (AXSecureTextField) → mọi phím PASS (từ có marker Telex — phân biệt được với không-secure)",
        "com.apple.safari",
        "editbox",
        r#"
:secure on
:type "duocj321"
:expect "duocj321"
:expect_action PASS"#,
    ));

    // ───────────────────────── owner rule (P2-2 §6) — không xử lý đôi IMK/tap ─────────────────────────
    // Lưu ý: headless replay chỉ mô phỏng được 1 engine — 2 case dưới pin hành vi
    // TỪNG phía (imk xử lý / tap xử lý thì IMK phải PASS); tích hợp thật do
    // corpus tap + test Swift `OwnerRuleTests` + ax-driver `owner_no_double` bảo vệ.
    v.push(imk(
        "owner_no_double_imk_01",
        "owner=imk: IMK xử lý bình thường (preset mac.terminal) — hành vi ≠ passthrough",
        "com.apple.terminal",
        "terminal",
        r#"
:type "dduocj"
:expect "được"
:expect_action REPLACE"#,
    ));
    v.push(imk(
        "owner_no_double_tap_02",
        "owner=tap: IMK ctx.enabled=0 → mọi phím PASS, tap xử lý (P2-2 §6)",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:enabled off
:type "wasd"
:expect "wasd"
:expect_action PASS"#,
    ));

    // ───────────────────────── restore_en (B5) trên mac ─────────────────────────
    v.push(imk(
        "restore_en_terminal_dir_01",
        "B5 lệnh EN trong iTerm2 được auto-restore (english_words)",
        "com.googlecode.iterm2",
        "terminal",
        r#"
:config method=telex diacritic_style=new english_words=dir,data
:type "dir"
:expect "dỉ"
:key Space
:expect "dir "
:type "data"
:expect "dir data"
:key Space
:expect "dir data ""#,
    ));
    v.push(imk(
        "restore_en_safari_docs_02",
        "B5 tên miền docs.rs không bị biến dạng",
        "com.apple.safari",
        "address_bar",
        r#"
:config method=telex diacritic_style=new auto_capitalize=false english_words=docs
:type "docs.rs"
:expect "docs.rs""#,
    ));
    v.push(imk(
        "restore_en_text_03",
        "B5 từ `text` → `tết` rồi restore khi hết từ",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=new english_words=text
:type "text"
:expect "tẽt"
:key Space
:expect "text ""#,
    ));

    // ───────────────────────── B6 — không nuốt chord/hotkey ─────────────────────────
    // DSL corpus chưa có modifier `Cmd` (P0-4 §2.4) — dùng chord Ctrl/Alt đại
    // diện để pin engine-level `is_chord`; chord Cmd riêng do Swift adapter
    // tự chặn ở `handle()` bước 1 (TextVNInputController, test trên máy thật).
    v.push(imk(
        "bug_B6_combo_pass_mac_01",
        "B6 chord hệ thống luôn PASS (engine không nuốt; Ctrl/Alt đại diện cho is_chord)",
        "com.apple.finder",
        "editbox",
        r#"
:combo Alt+Tab
:expect_action PASS
:combo Ctrl+Shift+Escape
:expect_action PASS"#,
    ));

    // ───────────────────────── macro / emoji ─────────────────────────
    v.push(imk(
        "macro_expand_cty_01",
        "macro mở rộng khi Space (macro_trigger=space)",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=new macro_trigger=space macro="cty=Công ty TNHH"
:type "cty"
:expect "cty"
:expect_action REPLACE
:key Space
:expect "Công ty TNHH""#,
    ));
    v.push(imk(
        "emoji_expand_smile_01",
        "emoji shortcut :smile + Space → 😀",
        "com.apple.notes",
        "textarea",
        r#"
:config method=telex diacritic_style=new macro_trigger=space emoji=":smile=😀"
:type ":smile"
:expect ":smile"
:expect_action REPLACE
:key Space
:expect "😀""#,
    ));

    // ───────────────────────── imk_preedit_* — vòng đời marked (40 case) ─────────────────────────
    v.push(imk(
        "imk_preedit_telex_duocj_01",
        "TextEdit golden duocj → được (setMarkedText)",
        "com.apple.textedit",
        "body",
        r#"
:type "dduocj"
:expect "được"
:expect_preedit "được"
:key Space
:expect "được "
:expect_preedit """#,
    ));
    v.push(imk(
        "imk_preedit_telex_multi_word_02",
        "hai từ nối tiếp, mỗi Space commit một từ",
        "com.apple.textedit",
        "body",
        r#"
:type "chaof"
:key Space
:type "banj"
:expect "chào bạn"
:expect_preedit "bạn""#,
    ));
    v.push(imk(
        "imk_preedit_telex_tieengs_03",
        "tiếng — iet + tone giữa marked",
        "com.apple.safari",
        "web",
        r#"
:type "tieengs"
:expect "tiếng""#,
    ));
    v.push(imk(
        "imk_preedit_telex_vieetj_04",
        "việt — ee→â + j nặng",
        "com.apple.textedit",
        "body",
        r#"
:type "vieetj"
:expect "việt""#,
    ));
    v.push(imk(
        "imk_preedit_telex_hoaf_new_05",
        "hoà kiểu mới (tone trên o thứ hai)",
        "com.apple.textedit",
        "body",
        r#"
:type "hoaf"
:expect "hoà""#,
    ));
    v.push(imk(
        "imk_preedit_telex_hoa_old_06",
        "hòa kiểu cũ (tone trên a)",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=old
:type "hoaf"
:expect "hòa""#,
    ));
    v.push(imk(
        "imk_preedit_telex_ddang_07",
        "đang — dd→đ",
        "com.apple.textedit",
        "body",
        r#"
:type "ddang"
:expect "đang""#,
    ));
    v.push(imk(
        "imk_preedit_telex_luong_08",
        "lương — cặp ươ ẩn",
        "com.apple.textedit",
        "body",
        r#"
:type "luong"
:expect "lương""#,
    ));
    v.push(imk(
        "imk_preedit_telex_tuong_09",
        "tương",
        "com.apple.textedit",
        "body",
        r#"
:type "tuong"
:expect "tương""#,
    ));
    v.push(imk(
        "imk_preedit_telex_tuoi_10",
        "tươi — uo+i → ươi",
        "com.apple.textedit",
        "body",
        r#"
:type "tuoi"
:expect "tươi""#,
    ));
    v.push(imk(
        "imk_preedit_telex_tuoir_11",
        "tưởi — tone hỏi trên ơ (style new)",
        "com.apple.textedit",
        "body",
        r#"
:type "tuoir"
:expect "tưởi""#,
    ));
    v.push(imk(
        "imk_preedit_telex_muoif_12",
        "mười",
        "com.apple.textedit",
        "body",
        r#"
:type "muoif"
:expect "mười""#,
    ));
    v.push(imk(
        "imk_preedit_telex_nuocs_13",
        "nước",
        "com.apple.textedit",
        "body",
        r#"
:type "nuocs"
:expect "nước""#,
    ));
    v.push(imk(
        "imk_preedit_telex_huwng_14",
        "hưng — uw→ư",
        "com.apple.textedit",
        "body",
        r#"
:type "huwng"
:expect "hưng""#,
    ));
    v.push(imk(
        "imk_preedit_telex_stroke_dd_15",
        "dd lên đ rồi từ có dấu",
        "com.apple.textedit",
        "body",
        r#"
:type "dduocj"
:expect "được""#,
    ));
    v.push(imk(
        "imk_preedit_telex_undo_ass_16",
        "bấm lại tone key → gỡ dấu + literal (ass→as)",
        "com.apple.textedit",
        "body",
        r#"
:type "ass"
:expect "as""#,
    ));
    v.push(imk(
        "imk_preedit_telex_undo_uww_17",
        "bấm lại w sau khi sừng → về gốc (uww→u)",
        "com.apple.textedit",
        "body",
        r#"
:type "uww"
:expect "u""#,
    ));
    v.push(imk(
        "imk_preedit_telex_undo_ww_18",
        "w đơn lẻ → chữ w (ww→w)",
        "com.apple.textedit",
        "body",
        r#"
:type "ww"
:expect "w""#,
    ));
    v.push(imk(
        "imk_preedit_telex_undo_ddd_19",
        "ddd → d (gỡ stroke)",
        "com.apple.textedit",
        "body",
        r#"
:type "ddd"
:expect "d""#,
    ));
    v.push(imk(
        "imk_preedit_telex_esc_restore_20",
        "Escape hủy biến đổi — RESTORE raw",
        "com.apple.textedit",
        "body",
        r#"
:type "ddang"
:expect "đang"
:key Escape
:expect "ddang"
:expect_preedit """#,
    ));
    v.push(imk(
        "imk_preedit_telex_backspace_fold_21",
        "Backspace sửa marked qua engine (không cho app sửa), xoá ký tự cuối như UniKey (R2-55)",
        "com.apple.textedit",
        "body",
        r#"
:type "chaof"
:expect "chào"
:key Backspace
:expect "chà""#,
    ));
    v.push(imk(
        "imk_preedit_telex_nav_cancel_22",
        "Left arrow → PASS, preedit nhả, chữ giữ nguyên",
        "com.apple.textedit",
        "body",
        r#"
:type "chaof"
:expect "chào"
:key Left
:expect "chào"
:expect_preedit """#,
    ));
    v.push(imk(
        "imk_preedit_telex_caps_dot_23",
        "tự viết hoa sau `. `",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=new auto_capitalize=true
:type "chao."
:key Space
:type "ban"
:expect "chao. Ban""#,
    ));
    v.push(imk(
        "imk_preedit_telex_caps_enter_24",
        "tự viết hoa sau Enter",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=new auto_capitalize=true
:type "xong"
:key Enter
:type "roofi"
:expect "xong\nRồi""#,
    ));
    v.push(imk(
        "imk_preedit_vni_duoc_25",
        "VNI được (du7o7c5)",
        "com.apple.textedit",
        "body",
        r#"
:config method=vni diacritic_style=new
:type "d9u7o7c5"
:expect "được""#,
    ));
    v.push(imk(
        "imk_preedit_vni_viet_26",
        "VNI việt (vie6t5)",
        "com.apple.textedit",
        "body",
        r#"
:config method=vni diacritic_style=new
:type "vie6t5"
:expect "việt""#,
    ));
    v.push(imk(
        "imk_preedit_vni_hoa_27",
        "VNI hoà (hoa2)",
        "com.apple.textedit",
        "body",
        r#"
:config method=vni diacritic_style=new
:type "hoa2"
:expect "hoà""#,
    ));
    v.push(imk(
        "imk_preedit_vni_tuoi_28",
        "VNI tưới (tu7o7i1)",
        "com.apple.textedit",
        "body",
        r#"
:config method=vni diacritic_style=new
:type "tu7o7i1"
:expect "tưới""#,
    ));
    v.push(imk(
        "imk_preedit_vni_nhieu_29",
        "VNI nhiều (nhie62u)",
        "com.apple.textedit",
        "body",
        r#"
:config method=vni diacritic_style=new
:type "nhie62u"
:expect "nhiều""#,
    ));
    v.push(imk(
        "imk_preedit_viqr_toi_30",
        "VIQR tôi (to^i)",
        "com.apple.textedit",
        "body",
        r#"
:config method=viqr diacritic_style=new
:type "to^i"
:expect "tôi""#,
    ));
    v.push(imk(
        "imk_preedit_viqr_viet_31",
        "VIQR việt (vie^t.)",
        "com.apple.textedit",
        "body",
        r#"
:config method=viqr diacritic_style=new
:type "vie^t."
:expect "việt""#,
    ));
    v.push(imk(
        "imk_preedit_viqr_nguoi_32",
        "VIQR người (ngu+o+`i)",
        "com.apple.textedit",
        "body",
        r#"
:config method=viqr diacritic_style=new
:type "ngu+o+`i"
:expect "người""#,
    ));
    v.push(imk(
        "imk_preedit_viqr_hoi_33",
        "VIQR hỏi (hoi?)",
        "com.apple.textedit",
        "body",
        r#"
:config method=viqr diacritic_style=new
:type "hoi?"
:expect "hỏi""#,
    ));
    v.push(imk(
        "imk_preedit_simple_telex_duongw_34",
        "Simple Telex: w là dấu sừng như UniKey (duongw → đương, R2-61)",
        "com.apple.textedit",
        "body",
        r#"
:config method=simple_telex diacritic_style=new
:type "dduongw"
:expect "đương""#,
    ));
    v.push(imk(
        "imk_preedit_simple_telex_duocj_35",
        "Simple Telex vẫn giữ tone/double (duocj → được)",
        "com.apple.textedit",
        "body",
        r#"
:config method=simple_telex diacritic_style=new
:type "dduocj"
:expect "được""#,
    ));
    v.push(imk(
        "imk_preedit_english_mixed_36",
        "từ EN chứa tone-key (master→mátẻ) được restore khi hết từ — không phá EN",
        "com.apple.textedit",
        "body",
        r#"
:config method=telex diacritic_style=new english_words=master
:type "master"
:expect "matẻ"
:key Space
:expect "master "
:type "dduocj"
:expect "master được""#,
    ));
    v.push(imk(
        "imk_preedit_app_textedit_37",
        "preset mac.textedit (body → Preedit)",
        "com.apple.textedit",
        "body",
        r#"
:type "hoanf"
:expect "hoàn""#,
    ));
    v.push(imk(
        "imk_preedit_app_safari_web_38",
        "preset mac.safari.body (web → Preedit)",
        "com.apple.safari",
        "web",
        r#"
:type "vieetj"
:expect "việt""#,
    ));
    v.push(imk(
        "imk_preedit_app_word_body_39",
        "preset mac.word.body",
        "com.microsoft.word",
        "body",
        r#"
:type "toong"
:expect "tông""#,
    ));
    v.push(imk(
        "imk_preedit_app_notes_40",
        "preset mac.notes (textarea)",
        "com.apple.notes",
        "textarea",
        r#"
:type "ghif"
:expect "ghì""#,
    ));

    // ───────────────────────── mac_bs_type_* — BackspaceType (30 case) ─────────────────────────
    v.push(bs(
        "mac_bs_type_telex_duocj_01",
        "BackspaceType duocj → được (n backspace + gõ lại)",
        "com.apple.finder",
        "editbox",
        r#"
:type "dduocj"
:expect "được""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_chaof_banj_02",
        "BackspaceType hai từ",
        "com.apple.finder",
        "editbox",
        r#"
:type "chaof"
:key Space
:type "banj"
:expect "chào bạn""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_tieengs_03",
        "BackspaceType tiếng",
        "com.apple.finder",
        "editbox",
        r#"
:type "tieengs"
:expect "tiếng""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_vieetj_04",
        "BackspaceType việt",
        "com.apple.finder",
        "editbox",
        r#"
:type "vieetj"
:expect "việt""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_hoaf_new_05",
        "BackspaceType hoà (new style)",
        "com.apple.finder",
        "editbox",
        r#"
:type "hoaf"
:expect "hoà""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_hoa_old_06",
        "BackspaceType hòa (old style)",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=old
:type "hoaf"
:expect "hòa""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_ddang_07",
        "BackspaceType đang",
        "com.apple.finder",
        "editbox",
        r#"
:type "ddang"
:expect "đang""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_luong_08",
        "BackspaceType lương",
        "com.apple.finder",
        "editbox",
        r#"
:type "luong"
:expect "lương""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_tuong_09",
        "BackspaceType tương",
        "com.apple.finder",
        "editbox",
        r#"
:type "tuong"
:expect "tương""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_tuoi_10",
        "BackspaceType tươi",
        "com.apple.finder",
        "editbox",
        r#"
:type "tuoi"
:expect "tươi""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_muoif_11",
        "BackspaceType mười",
        "com.apple.finder",
        "editbox",
        r#"
:type "muoif"
:expect "mười""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_nuocs_12",
        "BackspaceType nước",
        "com.apple.finder",
        "editbox",
        r#"
:type "nuocs"
:expect "nước""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_huwng_13",
        "BackspaceType hưng",
        "com.apple.finder",
        "editbox",
        r#"
:type "huwng"
:expect "hưng""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_stroke_14",
        "BackspaceType đ đơn (d+vowel)",
        "com.apple.finder",
        "editbox",
        r#"
:type "ddong"
:expect "đong""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_undo_ass_15",
        "BackspaceType undo tone (ass→as)",
        "com.apple.finder",
        "editbox",
        r#"
:type "ass"
:expect "as""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_undo_ww_16",
        "BackspaceType ww→w",
        "com.apple.finder",
        "editbox",
        r#"
:type "ww"
:expect "w""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_undo_uww_17",
        "BackspaceType uww→u",
        "com.apple.finder",
        "editbox",
        r#"
:type "uww"
:expect "u""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_esc_18",
        "BackspaceType Escape → RESTORE raw",
        "com.apple.finder",
        "editbox",
        r#"
:type "ddang"
:expect "đang"
:key Escape
:expect "ddang""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_backspace_19",
        "BackspaceType Backspace xoá ký tự cuối, giữ dấu (R2-55)",
        "com.apple.finder",
        "editbox",
        r#"
:type "chaof"
:expect "chào"
:key Backspace
:expect "chà""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_caps_dot_20",
        "BackspaceType auto-capitalize giữ đúng",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=new auto_capitalize=true
:type "chao."
:key Space
:type "ban"
:expect "chao. Ban""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_english_21",
        "BackspaceType EN chứa tone-key được restore (master)",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=new english_words=master
:type "master"
:expect "matẻ"
:key Space
:expect "master ""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_macro_22",
        "BackspaceType macro vẫn chạy",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=new macro_trigger=space macro="cty=Công ty TNHH"
:type "cty"
:key Space
:expect "Công ty TNHH""#,
    ));
    v.push(bs(
        "mac_bs_type_telex_emoji_23",
        "BackspaceType emoji vẫn chạy",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=new macro_trigger=space emoji=":smile=😀"
:type ":smile"
:key Space
:expect "😀""#,
    ));
    v.push(bs(
        "mac_bs_type_vni_duoc_24",
        "BackspaceType VNI được",
        "com.apple.finder",
        "editbox",
        r#"
:config method=vni diacritic_style=new
:type "d9u7o7c5"
:expect "được""#,
    ));
    v.push(bs(
        "mac_bs_type_vni_viet_25",
        "BackspaceType VNI việt",
        "com.apple.finder",
        "editbox",
        r#"
:config method=vni diacritic_style=new
:type "vie6t5"
:expect "việt""#,
    ));
    v.push(bs(
        "mac_bs_type_vni_hoa_26",
        "BackspaceType VNI hoà",
        "com.apple.finder",
        "editbox",
        r#"
:config method=vni diacritic_style=new
:type "hoa2"
:expect "hoà""#,
    ));
    v.push(bs(
        "mac_bs_type_viqr_toi_27",
        "BackspaceType VIQR tôi",
        "com.apple.finder",
        "editbox",
        r#"
:config method=viqr diacritic_style=new
:type "to^i"
:expect "tôi""#,
    ));
    v.push(bs(
        "mac_bs_type_viqr_viet_28",
        "BackspaceType VIQR việt",
        "com.apple.finder",
        "editbox",
        r#"
:config method=viqr diacritic_style=new
:type "vie^t."
:expect "việt""#,
    ));
    v.push(bs(
        "mac_bs_type_enter_multi_29",
        "BackspaceType Enter giữa 2 từ (B2 bản BackspaceType)",
        "com.apple.finder",
        "editbox",
        r#"
:config method=telex diacritic_style=new auto_capitalize=false
:type "xong"
:key Enter
:type "roofi"
:expect "xong\nrồi""#,
    ));
    v.push(bs(
        "mac_bs_type_simple_telex_30",
        "BackspaceType Simple Telex (w là dấu sừng, R2-61)",
        "com.apple.finder",
        "editbox",
        r#"
:config method=simple_telex diacritic_style=new
:type "dduongw"
:expect "đương""#,
    ));

    // ───────────────────────── tap_* — CGEventTap opt-in (20 case) ─────────────────────────
    v.push(tap(
        "tap_body_telex_duocj_01",
        "tap editbox → BackspaceType (không PREEDIT)",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "dduocj"
:expect "được""#,
    ));
    v.push(tap(
        "tap_body_telex_chaof_02",
        "tap hai từ có dấu",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "chaof"
:key Space
:type "banj"
:expect "chào bạn""#,
    ));
    v.push(tap(
        "tap_body_vni_duoc_03",
        "tap VNI được",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=vni diacritic_style=new
:type "d9u7o7c5"
:expect "được""#,
    ));
    v.push(tap(
        "tap_body_viqr_toi_04",
        "tap VIQR tôi",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=viqr diacritic_style=new
:type "to^i"
:expect "tôi""#,
    ));
    v.push(tap(
        "tap_body_simple_telex_05",
        "tap Simple Telex duongw → đương (R2-61)",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=simple_telex diacritic_style=new
:type "dduongw"
:expect "đương""#,
    ));
    v.push(tap(
        "tap_body_undo_ww_06",
        "tap undo ww→w",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "ww"
:expect "w""#,
    ));
    v.push(tap(
        "tap_body_esc_restore_07",
        "tap Escape RESTORE",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "ddang"
:expect "đang"
:key Escape
:expect "ddang""#,
    ));
    v.push(tap(
        "tap_body_backspace_08",
        "tap Backspace xoá ký tự cuối, giữ dấu (R2-55)",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "chaof"
:expect "chào"
:key Backspace
:expect "chà""#,
    ));
    v.push(tap(
        "tap_body_caps_dot_09",
        "tap auto-capitalize sau chấm",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=telex diacritic_style=new auto_capitalize=true
:type "chao."
:key Space
:type "ban"
:expect "chao. Ban""#,
    ));
    v.push(tap(
        "tap_body_english_10",
        "tap EN chứa tone-key được restore (master) — không phá EN",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=telex diacritic_style=new english_words=master
:type "master"
:expect "matẻ"
:key Space
:expect "master ""#,
    ));
    v.push(tap(
        "tap_address_safari_11",
        "tap SelectionReplace Safari URL (B1 app không-IMK)",
        "com.apple.safari",
        "address_bar",
        r#"
:type "vieetj"
:expect "việt"
:key Space
:type "nam"
:expect "việt nam""#,
    ));
    v.push(tap(
        "tap_address_chrome_12",
        "tap SelectionReplace Chrome URL",
        "com.google.chrome",
        "address_bar",
        r#"
:type "tieengs"
:expect "tiếng""#,
    ));
    v.push(tap(
        "tap_search_spotlight_13",
        "tap Spotlight search",
        "com.apple.spotlight",
        "search",
        r#"
:type "hoaf"
:expect "hoà""#,
    ));
    v.push(tap(
        "tap_candidate_xcode_14",
        "tap candidate Xcode",
        "com.apple.dt.xcode",
        "candidate",
        r#"
:type "ddang"
:expect "đang""#,
    ));
    v.push(tap(
        "tap_candidate_jetbrains_15",
        "tap candidate JetBrains (vk_then_unicode ở tap)",
        "com.jetbrains.intellij",
        "candidate",
        r#"
:type "tieenf"
:expect "tiền""#,
    ));
    v.push(tap(
        "tap_terminal_forward_16",
        "tap terminal ForwardAsCommit (chỉ chèn, không xóa)",
        "com.apple.terminal",
        "terminal",
        r#"
:type "dduocj"
:expect "được""#,
    ));
    v.push(tap(
        "tap_terminal_iterm2_17",
        "tap iTerm2 nhiều từ",
        "com.googlecode.iterm2",
        "terminal",
        r#"
:type "chaof"
:key Space
:type "banj"
:expect "chào bạn""#,
    ));
    v.push(tap(
        "tap_secure_18",
        "tap secure field → PASS (không đọc/không inject)",
        "com.apple.safari",
        "editbox",
        r#"
:secure on
:type "pass123"
:expect "pass123"
:expect_action PASS"#,
    ));
    v.push(tap(
        "tap_owner_imk_disabled_19",
        "owner=tap → IMK disabled, tap gõ bình thường (counter 0 inject từ IMK)",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:type "cuwsu"
:expect "cứu""#,
    ));
    v.push(tap(
        "tap_macro_20",
        "tap macro mở rộng",
        "com.valvesoftware.steam",
        "editbox",
        r#"
:config method=telex diacritic_style=new macro_trigger=space macro="cty=Công ty TNHH"
:type "cty"
:key Space
:expect "Công ty TNHH""#,
    ));

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_case_la_duy_nhat_va_dung_chuan() {
        let cases = all_cases();
        assert!(
            cases.len() >= 100,
            "corpus mac phải ≥100 case, thấy {}",
            cases.len()
        );
        let mut names: Vec<&str> = cases.iter().map(|c| c.name.as_str()).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "tên case trùng nhau");
        for c in &cases {
            assert!(c.name.ends_with(".keys"), "{c:?} phải kết thúc .keys");
            assert!(
                c.content.starts_with("# corpus/mac/"),
                "header sai: {}",
                c.name
            );
        }
    }

    #[test]
    fn du_nhom_case_theo_p2_5() {
        let cases = all_cases();
        let count = |p: &str| cases.iter().filter(|c| c.name.starts_with(p)).count();
        assert!(
            count("imk_preedit_") >= 40,
            "imk_preedit_* phải ≥40 (P2-5 §2)"
        );
        assert!(count("mac_bs_type_") >= 30, "mac_bs_type_* phải ≥30");
        assert!(count("tap_") >= 20, "tap_* phải ≥20");
        for req in [
            "bug_B1_safari_url_01",
            "bug_B1_chrome_url_01",
            "bug_B1_spotlight_01",
            "bug_B1_excel_cell_01",
            "bug_B2_imessage_enter_01",
            "bug_B3_xcode_completion_01",
            "bug_B11_marked_commit_space_01",
            "bug_B13_focus_loss_01",
            "bug_B8_terminal_01",
            "secure_field_passthrough_01",
            "owner_no_double_tap_02",
        ] {
            let want = format!("{req}.keys");
            assert!(
                cases.iter().any(|c| c.name == want),
                "thiếu case bắt buộc {want}"
            );
        }
    }
}
