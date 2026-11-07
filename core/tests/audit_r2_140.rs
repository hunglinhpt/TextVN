// SPDX-License-Identifier: GPL-3.0-or-later
//! Hồi quy audit sau 0.2.68 (R2-140…R2-149): mỗi ca ở đây là một lỗi đã tái hiện được
//! trên bản 0.2.68. Thiết kế DRBFM: `docs/release/AUDIT-0.2.69-DFMEA-DRBFM.md`.

use strategy::{IME_FIELD_ADDRESS_BAR, IME_FIELD_SEARCH, IME_FIELD_TERMINAL};
use textvn_core::{keymap, Action, Context, Engine, EngineOptions, KeyEvent, Method};

fn opts() -> EngineOptions {
    EngineOptions {
        method: Method::Telex,
        auto_capitalize: false,
        ..Default::default()
    }
}

fn apply(doc: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            if k.vk == keymap::vk::BACK {
                doc.pop();
            } else if let Some(c) = k.printable() {
                if !k.is_ctrl_space() {
                    doc.push(c);
                }
            }
        }
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            doc.truncate(doc.len().saturating_sub(usize::from(*delete_count)));
            doc.extend(insert);
        }
        Action::Commit { insert } => doc.extend(insert),
    }
}

fn press(e: &mut Engine, doc: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    apply(doc, &out.action, k);
}

fn char_key(c: char) -> KeyEvent {
    match c {
        '\t' => KeyEvent {
            vk: keymap::vk::TAB,
            ch: '\t' as u32,
            key_down: true,
            ..Default::default()
        },
        ' ' => KeyEvent {
            vk: keymap::vk::SPACE,
            ch: ' ' as u32,
            key_down: true,
            ..Default::default()
        },
        _ => KeyEvent::char_down(c),
    }
}

fn ctrl_space() -> KeyEvent {
    KeyEvent {
        vk: keymap::vk::SPACE,
        mods: keymap::MOD_CTRL,
        key_down: true,
        ..Default::default()
    }
}

fn type_in(e: &mut Engine, input: &str) -> String {
    let mut doc = Vec::new();
    for c in input.chars() {
        press(e, &mut doc, &char_key(c));
    }
    doc.into_iter().collect()
}

fn typed(input: &str) -> String {
    type_in(&mut Engine::new(opts()), input)
}

fn field(e: &mut Engine, role: u32) {
    e.set_context(Context {
        enabled: true,
        field_role: role,
        ..Default::default()
    });
}

fn mode_en(e: &mut Engine, role: u32) {
    e.set_context(Context {
        enabled: false,
        field_role: role,
        ..Default::default()
    });
}

/// Tên từ chỉ gồm chữ cái (`valid_style_entry` không nhận chữ số).
fn letters(mut i: u32) -> String {
    let mut s = String::new();
    loop {
        s.push(char::from(b'a' + (i % 26) as u8));
        i /= 26;
        if i == 0 {
            return s + "q";
        }
    }
}

fn learning() -> EngineOptions {
    EngineOptions {
        suggest_vn_complete: true,
        style_learning_enabled: true,
        ..opts()
    }
}

// ---- R2-140: phím ngoặc Telex --------------------------------------------------

#[test]
fn r2_140_code_va_markdown_giu_nguyen_ngoac() {
    for s in [
        "arr[0] ",
        "a[0]; ",
        "x[1] ",
        "x[i] ",
        "s[i] ",
        "m[n] ",
        "[1, 2, 3] ",
        "- [ ] task ",
        "- [x] done ",
        "dict[\"key\"] ",
        "vec![1] ",
        "foo[bar] ",
        "[[wiki]] ",
        "[ -f x ] ",
        "a[i][j] ",
        "s[i+1] ",
        "t[i-1] ",
        "[a, b] ",
        "[n, m] ",
        "{\"a\": [b, c]} ",
        "s[:5] ",
        "s[i:j] ",
        "[] ",
    ] {
        assert_eq!(typed(s), s, "ngoặc trong code/Markdown bị đổi: {s:?}");
    }
}

#[test]
fn r2_140_chu_viet_trong_ngoac_van_go_duoc() {
    for (input, want) in [
        ("[tooi] ", "[tôi] "),
        ("[ddaay] ", "[đây] "),
        ("[Vieetj Nam] ", "[Việt Nam] "),
        ("(ch]a) ", "(chưa) "),
        ("[h[n] ", "[hơn] "),
    ] {
        assert_eq!(typed(input), want, "{input:?}");
    }
}

#[test]
fn r2_140_gox_vowel_giua_tu_khong_doi() {
    for (input, want) in [
        ("ch]a ", "chưa "),
        ("m[f ", "mờ "),
        ("h[n ", "hơn "),
        ("th][ng ", "thương "),
        ("ddu[cj ", "được "),
        ("hu[ng ", "hương "),
        ("x[ ", "xơ "),
        ("h[n-kem ", "hơn-kem "),
        ("h[n/kem ", "hơn/kem "),
        ("th][ng-maij ", "thương-mại "),
        ("\"th]\" ", "\"thư\" "),
        ("h[, ", "hơ, "),
    ] {
        assert_eq!(typed(input), want, "{input:?}");
    }
}

#[test]
fn r2_140_dau_thanh_go_truoc_ngoac() {
    assert_eq!(typed("tus[ng "), "tướng ");
    assert_eq!(typed("tuf[ng "), "tường ");
}

#[test]
fn r2_140_chu_hoa_quanh_ngoac() {
    for (input, want) in [
        ("CH]A ", "CHƯA "),
        ("H[N ", "HƠN "),
        ("TH][NG ", "THƯƠNG "),
        ("H[n ", "Hơn "),
        ("H[ ", "Hơ "),
        ("Ch]a ", "Chưa "),
    ] {
        assert_eq!(typed(input), want, "{input:?}");
    }
}

#[test]
fn r2_140_go_lai_ngoac_gia_tri_lit_khong_bi_restore() {
    assert_eq!(typed("m[[ "), "m[ ");
    assert_eq!(typed("ch]] "), "ch] ");
}

#[test]
fn r2_140_quick_telex_nhan_ngoac() {
    let mut e = Engine::new(EngineOptions {
        quick_telex: true,
        ..opts()
    });
    assert_eq!(type_in(&mut e, "cc]a "), "chưa ");
}

#[test]
fn r2_140_terminal_va_thanh_dia_chi_khong_fold_ngoac() {
    for role in [IME_FIELD_TERMINAL, IME_FIELD_ADDRESS_BAR] {
        let mut e = Engine::new(opts());
        field(&mut e, role);
        assert_eq!(type_in(&mut e, "m[f "), "m[f ", "role {role}");
    }
}

#[test]
fn r2_140_auto_restore_tat_van_khong_pha_chi_muc() {
    let mut e = Engine::new(EngineOptions {
        auto_restore_english: false,
        ..opts()
    });
    assert_eq!(type_in(&mut e, "x[1] "), "x[1] ");
    assert_eq!(type_in(&mut e, "s[i] "), "s[i] ");
}

// ---- R2-141: gạch nối dài ------------------------------------------------------

#[test]
fn r2_141_code_cli_markdown_khong_bi_doi() {
    for s in [
        "git push --force ",
        "cargo build --release ",
        "i--;",
        "i-- ",
        "x--)",
        "<!-- x --> ",
        "|--|--|",
        "=A1--B1",
        "-- comment",
        "x = 1 -- comment ",
        "git checkout -- file ",
        "--- ",
        "a---b",
        "----",
    ] {
        assert_eq!(typed(s), s, "gạch ngang bị đổi: {s:?}");
    }
}

#[test]
fn r2_141_tra_lai_giu_enter_tab_la_phim_that() {
    for (k, want) in [(keymap::vk::RETURN, '\n'), (keymap::vk::TAB, '\t')] {
        let mut e = Engine::new(opts());
        let _ = type_in(&mut e, "hello--");
        let out = e.key(&KeyEvent {
            vk: k,
            ch: want as u32,
            key_down: true,
            ..Default::default()
        });
        // RESTORE: adapter tách ký tự ranh giới cuối thành phím thật (không nuốt Enter).
        assert!(
            matches!(out.action, Action::Restore { .. }),
            "{:?}",
            out.action
        );
    }
}

#[test]
fn r2_141_co_het_han_sau_mot_phim() {
    let mut e = Engine::new(opts());
    let mut doc: Vec<char> = Vec::new();
    for c in "a--b".chars() {
        press(&mut e, &mut doc, &char_key(c));
    }
    let back = KeyEvent {
        vk: keymap::vk::BACK,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut doc, &back);
    press(&mut e, &mut doc, &char_key(';'));
    assert_eq!(doc.iter().collect::<String>(), "a—;");
}

#[test]
fn r2_141_van_xuoi_van_doi() {
    assert_eq!(typed("a--b"), "a—b");
    assert_eq!(typed("Hà Nội -- TP.HCM"), "Hà Nội — TP.HCM");
    assert_eq!(typed("Vieetj--Nam "), "Việt—Nam ");
}

#[test]
fn r2_141_o_tim_kiem_va_lenh_khong_doi() {
    for role in [IME_FIELD_TERMINAL, IME_FIELD_ADDRESS_BAR, IME_FIELD_SEARCH] {
        let mut e = Engine::new(opts());
        field(&mut e, role);
        assert_eq!(type_in(&mut e, "a--b x -- y"), "a--b x -- y", "role {role}");
    }
}

// ---- R2-144: Mode EN ----------------------------------------------------------

#[test]
fn r2_144_terminal_giu_tab_cho_shell() {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        ..opts()
    });
    mode_en(&mut e, IME_FIELD_TERMINAL);
    assert_eq!(type_in(&mut e, "git che\t"), "git che\t");
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        ..opts()
    });
    field(&mut e, IME_FIELD_TERMINAL);
    assert_eq!(type_in(&mut e, "cd doc\t"), "cd doc\t");
}

#[test]
fn r2_144_terminal_khong_hoan_tat_tu_viet() {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        ..opts()
    });
    field(&mut e, IME_FIELD_TERMINAL);
    assert_eq!(type_in(&mut e, "ddi\t"), "đi\t");
    assert!(e.peek_completion().is_none());
}

#[test]
fn r2_144_mode_en_tab_can_du_doan_bat() {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: false,
        suggest_space_complete: true,
        ..opts()
    });
    mode_en(&mut e, 0);
    assert_eq!(type_in(&mut e, "devel\t"), "devel\t");
}

#[test]
fn r2_144_mode_en_space_khong_viet_lai_chu() {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        suggest_space_complete: true,
        ..opts()
    });
    mode_en(&mut e, 0);
    let s = "Tran Hung func impl cfg repo ";
    assert_eq!(type_in(&mut e, s), s);
}

#[test]
fn r2_144_ctrl_space_khi_du_doan_tat_van_hoan_tat_theo_yeu_cau() {
    for (input, want) in [("ebit", "EBITDA"), ("sort", "sort")] {
        let mut e = Engine::new(opts());
        let mut doc: Vec<char> = Vec::new();
        for c in input.chars() {
            press(&mut e, &mut doc, &char_key(c));
        }
        press(&mut e, &mut doc, &ctrl_space());
        assert_eq!(doc.iter().collect::<String>(), want, "{input:?}");
    }
}

// ---- R2-145: học trong terminal ------------------------------------------------

#[test]
fn r2_145_terminal_khong_hoc_chu_nao() {
    for enabled in [true, false] {
        let mut e = Engine::new(learning());
        e.set_context(Context {
            enabled,
            field_role: IME_FIELD_TERMINAL,
            ..Default::default()
        });
        let _ = type_in(&mut e, "hunter secret\n");
        let json = e.style_peek_json();
        assert!(
            !json.contains("hunter") && !json.contains("secret"),
            "chữ gõ ở terminal không được vào hồ sơ: {json}"
        );
    }
}

#[test]
fn r2_145_peek_khong_hoc_khong_dem() {
    let mut e = Engine::new(learning());
    let _ = type_in(&mut e, "rele");
    let before = e.dream_rsi_snapshot();
    assert!(e.peek_completion().is_some());
    let after = e.dream_rsi_snapshot();
    assert_eq!(before.accepts, after.accepts);
    assert_eq!(before.chars_saved, after.chars_saved);
    assert!(!e.style_peek_json().contains("release"));
}

// ---- R2-146: hồ sơ đầy ----------------------------------------------------------

#[test]
fn r2_146_tu_dung_nhieu_song_qua_loat_tu_moi() {
    let mut words: Vec<(String, u32)> = (0..499)
        .map(|i| (format!("old{}", letters(i)), 2))
        .collect();
    words.insert(0, ("hotword".to_string(), 200));
    let mut e = Engine::new(EngineOptions {
        style_words: words,
        ..learning()
    });
    for i in 0..10 {
        e.style_note_word(&format!("fresh{}", letters(i)));
    }
    let list = &e.options().style_words;
    assert_eq!(list.len(), 500, "không vượt trần");
    let hot = list.iter().find(|(w, _)| w == "hotword");
    assert_eq!(
        hot.map(|(_, c)| *c),
        Some(200),
        "từ dùng nhiều bị xoá lịch sử"
    );
    for i in 0..10 {
        let w = format!("fresh{}", letters(i));
        assert!(
            list.iter().any(|(x, _)| *x == w),
            "từ mới loại lẫn nhau: {w}"
        );
    }
}

#[test]
fn r2_146_tu_moi_lien_tuc_van_giam_trong_so_dinh_ky() {
    let mut words: Vec<(String, u32)> = (0..499)
        .map(|i| (format!("old{}", letters(i)), 2))
        .collect();
    words.insert(0, ("hotword".to_string(), 200));
    let mut e = Engine::new(EngineOptions {
        style_words: words,
        ..learning()
    });
    for i in 0..64 {
        e.style_note_word(&format!("fresh{}", letters(i)));
    }
    let hot = e.options().style_words.iter().find(|(w, _)| w == "hotword");
    assert_eq!(hot.map(|(_, c)| *c), Some(100), "đủ chu kỳ mới giảm nửa");
}

// ---- R2-147: học ứng viên -------------------------------------------------------

#[test]
fn r2_147_ung_vien_bi_bo_qua_khong_vao_ho_so() {
    let mut e = Engine::new(learning());
    let doc = type_in(&mut e, "rele\t\t ");
    assert!(!doc.is_empty());
    let last = doc.trim_end().to_string();
    let json = e.style_peek_json();
    assert!(json.contains(&format!("\"{last}\"")), "chốt: {json}");
    assert!(
        !json.contains("\"release\"") || last == "release",
        "ứng viên bị Tab bỏ qua vẫn được học: {json}"
    );
}

#[test]
fn r2_147_xoa_ngay_la_tu_choi() {
    for key in [keymap::vk::BACK, keymap::vk::ESCAPE] {
        let mut e = Engine::new(learning());
        let mut doc: Vec<char> = Vec::new();
        for c in "rele\t".chars() {
            press(&mut e, &mut doc, &char_key(c));
        }
        press(
            &mut e,
            &mut doc,
            &KeyEvent {
                vk: key,
                key_down: true,
                ..Default::default()
            },
        );
        assert!(
            !e.style_peek_json().contains("release"),
            "từ chối vẫn được học (vk {key})"
        );
    }
}

#[test]
fn r2_147_mot_ung_vien_chot_o_phim_ke_tiep() {
    let mut e = Engine::new(learning());
    e.style_note_word("zzqtest");
    let _ = e.style_drain_json();
    let _ = type_in(&mut e, "zzqt\t ");
    assert!(e.style_peek_json().contains("zzqtest"));
}

#[test]
fn r2_147_mot_ung_vien_xoa_ngay_cung_la_tu_choi() {
    let mut e = Engine::new(learning());
    e.style_note_word("zzqtest");
    let _ = e.style_drain_json();
    let before = e.dream_rsi_snapshot();
    let mut doc: Vec<char> = Vec::new();
    for c in "zzqt\t".chars() {
        press(&mut e, &mut doc, &char_key(c));
    }
    let back = KeyEvent {
        vk: keymap::vk::BACK,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut doc, &back);
    assert!(!e.style_peek_json().contains("zzqtest"));
    assert_eq!(before.accepts, e.dream_rsi_snapshot().accepts);
}

#[test]
fn r2_147_reset_khong_hoc_chu_chua_chac_vao_document() {
    let mut e = Engine::new(learning());
    let _ = type_in(&mut e, "rele\t");
    e.reset();
    assert!(!e.style_peek_json().contains("release"));
}

// ---- R2-148: cụm hai từ, drain --------------------------------------------------

#[test]
fn r2_145_mode_en_phim_dieu_khien_khong_hoc_tien_to() {
    let mut e = Engine::new(learning());
    mode_en(&mut e, 0);
    let _ = type_in(&mut e, "devel");
    // Layout macOS: Backspace có ch = 0x08, mũi tên 0x1C.
    for ch in [0x08u32, 0x1C] {
        let _ = e.key(&KeyEvent {
            vk: 0x25,
            ch,
            key_down: true,
            ..Default::default()
        });
    }
    assert!(!e.style_peek_json().contains("devel"));
}

#[test]
fn r2_147_mode_en_nhan_ung_vien_chi_dem_mot_lan() {
    let mut e = Engine::new(learning());
    mode_en(&mut e, 0);
    let _ = type_in(&mut e, "rele\t ");
    let json = e.style_peek_json();
    assert!(json.contains("\"word\":\"release\",\"count\":1"), "{json}");
}

#[test]
fn r2_147_tu_choi_go_khoi_dream_rsi() {
    let mut e = Engine::new(learning());
    let _ = type_in(&mut e, "rele");
    let before = e.dream_rsi_snapshot();
    let mut doc: Vec<char> = "rele".chars().collect();
    press(&mut e, &mut doc, &char_key('\t'));
    press(&mut e, &mut doc, &char_key('\t'));
    let back = KeyEvent {
        vk: keymap::vk::BACK,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut doc, &back);
    let after = e.dream_rsi_snapshot();
    assert_eq!(before.accepts, after.accepts);
    assert_eq!(before.chars_saved, after.chars_saved);
}

#[test]
fn r2_148_cum_khong_vat_qua_xuong_dong() {
    let mut e = Engine::new(learning());
    let _ = type_in(&mut e, "foo\n    bar ");
    assert!(!e.style_peek_json().contains("foo bar"));
}

#[test]
fn r2_148_cum_can_khoang_trang_that() {
    let mut e = Engine::new(learning());
    let _ = type_in(&mut e, "abc-ddef ");
    let json = e.style_peek_json();
    assert!(!json.contains("abc- "), "từ trước dính gạch: {json}");
}

#[test]
fn r2_148_drain_uu_tien_tu_go_nhieu_va_khong_mat_phan_con_lai() {
    let mut e = Engine::new(learning());
    for i in 0..300 {
        e.style_note_word(&format!("aword{}", letters(i)));
    }
    for _ in 0..3 {
        e.style_note_word("zebra");
    }
    let first = e.style_drain_json();
    assert!(first.contains("zebra"), "từ cuối vần ABC gõ nhiều bị cắt");
    let second = e.style_drain_json();
    assert!(
        second.contains("aword"),
        "phần vượt trần phải còn cho lượt drain sau: {second}"
    );
    let total = first.matches("\"word\":").count() + second.matches("\"word\":").count();
    assert_eq!(total, 301);
}

// ---- R2-149: Tab đầu trả từ gốc --------------------------------------------------

#[test]
fn r2_149_tab_dau_tra_tu_goc() {
    let vi = || {
        Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..opts()
        })
    };
    for w in ["user", "smart", "order", "forward", "sort"] {
        assert_eq!(type_in(&mut vi(), &format!("{w}\t")), w, "{w}");
    }
    // Fold ra âm tiết Việt hợp lệ là chữ Việt người dùng đang gõ: Tab không biến thành tiếng Anh.
    assert_eq!(type_in(&mut vi(), "hats\t"), "hát\t");
    assert_eq!(type_in(&mut vi(), "chaos\t"), "cháo\t");
    // Từ không còn gì để kéo dài: Tab tới app, từ về nguyên gốc.
    assert_eq!(type_in(&mut vi(), "abraham\t"), "abraham\t");
    // Từ không bị fold (`file`, `script`) hiển thị đúng rồi: Tab là dự đoán, được kéo dài.
    assert_eq!(type_in(&mut vi(), "file\t"), "file system");
    // Từ cá nhân dài hơn không che từ gốc.
    let mut e = Engine::new(EngineOptions {
        style_words: vec![("sorts".to_string(), 50)],
        ..learning()
    });
    assert_eq!(type_in(&mut e, "sort\t"), "sort");
}

#[test]
fn r2_149_ngu_canh_mot_tu_truoc() {
    use textvn_core::post::restore_en::has_en_context;
    assert!(has_en_context("hello "));
    assert!(has_en_context("machine "));
    assert!(has_en_context("say hello "));
    assert!(!has_en_context(""));
    assert!(!has_en_context("tôi "));
}

// ---- Bất biến chung (abnormal) ---------------------------------------------------

/// Không chuỗi phím nào làm document lệch với sổ của engine: mọi ký tự ngoặc/gạch
/// ngang trộn chữ, gõ lại từ đầu trên cùng engine vẫn không panic và không mất chữ.
#[test]
fn abnormal_ngoac_gach_ngang_tron_chu_khong_mat_ky_tu() {
    const ALPHABET: [char; 12] = ['[', ']', '-', ' ', 'a', 'h', 'n', 's', 'f', '0', '"', ';'];
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..4000 {
        let mut s = String::new();
        for _ in 0..12 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            s.push(ALPHABET[(seed % 12) as usize]);
        }
        let out = typed(&s);
        // Các chữ số và dấu không tham gia biến đổi đều phải còn nguyên, đúng thứ tự.
        let keep = |t: &str| {
            t.chars()
                .filter(|c| matches!(c, '0' | '"' | ';'))
                .collect::<String>()
        };
        assert_eq!(keep(&s), keep(&out), "input {s:?} → {out:?}");
    }
}

#[test]
fn ctrl_trai_phai_cung_la_modifier() {
    for vk in [0xA2u32, 0xA3] {
        let k = KeyEvent {
            vk,
            key_down: true,
            ..Default::default()
        };
        assert!(k.is_modifier() && k.is_ctrl_key());
    }
}

// ---- DFMEA: fuzz tổng hợp mọi đường mới ---------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }
}

/// Mọi tổ hợp phím trên mọi ngữ cảnh: engine không panic, không bao giờ xoá nhiều hơn chữ
/// đang có trong document, không học và không dự đoán ở terminal, drain luôn cân ngoặc.
#[test]
fn abnormal_fuzz_tong_hop_bat_bien() {
    const LETTERS: [char; 20] = [
        'a', 'e', 'o', 'u', 'i', 's', 'f', 'r', 'x', 'j', 'w', 'd', 'h', 'n', 't', 'c', 'A', 'H',
        'N', 'T',
    ];
    const OTHERS: [char; 10] = ['[', ']', '-', ' ', ' ', ' ', '0', '"', ';', '\n'];
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for round in 0..1500 {
        let suggest = round % 2 == 0;
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: suggest,
            suggest_space_complete: round % 4 == 0,
            style_learning_enabled: true,
            quick_telex: round % 5 == 0,
            auto_restore_english: round % 7 != 0,
            auto_capitalize: round % 3 == 0,
            ..Default::default()
        });
        let enabled = round % 3 != 1;
        let role = [
            0,
            IME_FIELD_TERMINAL,
            IME_FIELD_ADDRESS_BAR,
            IME_FIELD_SEARCH,
        ][round % 4];
        e.set_context(Context {
            enabled,
            field_role: role,
            ..Default::default()
        });
        let mut doc: Vec<char> = Vec::new();
        for step in 0..48 {
            let k = match rng.next(12) {
                0 => char_key('\t'),
                1 => KeyEvent {
                    vk: keymap::vk::BACK,
                    key_down: true,
                    ..Default::default()
                },
                2 => ctrl_space(),
                3 => {
                    let down = KeyEvent {
                        vk: keymap::vk::CONTROL,
                        key_down: true,
                        ..Default::default()
                    };
                    let _ = e.key(&down);
                    KeyEvent {
                        vk: keymap::vk::CONTROL,
                        key_down: false,
                        ..Default::default()
                    }
                }
                4..=6 => char_key(OTHERS[rng.next(10) as usize]),
                _ => char_key(LETTERS[rng.next(20) as usize]),
            };
            let out = e.key(&k);
            if let Action::Replace { delete_count, .. } | Action::Restore { delete_count, .. } =
                &out.action
            {
                assert!(
                    usize::from(*delete_count) <= doc.len(),
                    "round {round} step {step}: xoá {delete_count} trên {} ký tự",
                    doc.len()
                );
            }
            if k.key_down {
                apply(&mut doc, &out.action, &k);
            } else if !matches!(out.action, Action::Pass) {
                apply(&mut doc, &out.action, &k);
            }
            let _ = e.peek_completion();
        }
        let json = e.style_drain_json();
        assert_eq!(
            json.matches('{').count(),
            json.matches('}').count(),
            "drain lệch ngoặc: {json}"
        );
        if role == IME_FIELD_TERMINAL {
            assert!(
                !json.contains("\"word\""),
                "terminal không được học: {json}"
            );
        }
        if role == IME_FIELD_TERMINAL {
            assert_eq!(e.dream_rsi_snapshot().accepts, 0, "terminal không dự đoán");
        }
    }
}

/// Nhiều engine chia sẻ `Arc` sau `clone` (peek): học ở bản thật không rò sang bản sao cũ,
/// và `peek` trên hồ sơ đang học không đổi kết quả của phím thật.
#[test]
fn abnormal_peek_khong_doi_ket_qua_phim_that() {
    for input in ["rele", "phats", "cộng hòa", "sort", "ddi", "xin"] {
        let run = |peek: bool| {
            let mut e = Engine::new(learning());
            let mut doc: Vec<char> = Vec::new();
            for c in input.chars() {
                press(&mut e, &mut doc, &char_key(c));
                if peek {
                    let _ = e.peek_completion();
                }
            }
            press(&mut e, &mut doc, &char_key('\t'));
            press(&mut e, &mut doc, &char_key(' '));
            (doc, e.style_peek_json(), e.dream_rsi_snapshot().accepts)
        };
        assert_eq!(run(true), run(false), "{input}");
    }
}

#[test]
fn abnormal_tu_cuc_dai_va_ngoac_lien_tiep() {
    let s = "[".repeat(300) + &"]".repeat(300) + " ";
    assert_eq!(typed(&s), s);
    let long = "h[".repeat(200);
    let out = typed(&long);
    assert!(!out.is_empty());
    let dashes = "-".repeat(500);
    assert_eq!(typed(&dashes), dashes);
}
