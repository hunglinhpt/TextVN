//! E50 / E60: Deep audit & fix bug "đang định gõ âm tiết thành đất, nó ra đata, space cái thì ra ddat"
//! "Task space ra Tassk" · E60: "muốn viết Sir nhưng ra sỉ và khi thêm r thì lại thành Sirr chứ ko phải Sir"
//!
//! Requirement:
//! - Nếu đã restore theo rule mới → skip rule Unikey
//! - Nếu đã tiếp tục gõ thêm ký tự theo style Unikey → giữ style Unikey tiếng Anh / tiếng Anh thắng, không restore sai (`Sirr` → `Sir`, `Tassk` → `Task`, `Linhss` → `Linhs`)

use strategy::{
    IME_FIELD_BODY, IME_STRATEGY_BACKSPACE_TYPE, IME_STRATEGY_FORWARD_AS_COMMIT,
    IME_STRATEGY_PREEDIT,
};
use textvn_core::keymap;
use textvn_core::method::{fold, Method};
use textvn_core::transform::DiacriticStyle;
use textvn_core::{Action, Context, Engine, EngineOptions, KeyEvent};

/// Helper: gõ một chuỗi qua engine (dùng đúng vk::SPACE/RETURN/TAB giống adapter thật)
fn type_keys(e: &mut Engine, keys: &str, buf: &mut Vec<char>) {
    for c in keys.chars() {
        let k = match c {
            ' ' => KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..Default::default()
            },
            '\n' => KeyEvent::key_down(keymap::vk::RETURN),
            '\t' => KeyEvent::key_down(keymap::vk::TAB),
            _ => KeyEvent::char_down(c),
        };
        let outcome = e.key(&k);
        match outcome.action {
            Action::Pass => buf.push(c),
            Action::Replace {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Restore {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
}

/// Helper: gõ và nhận kết quả với cấu hình mặc định
fn type_and_get(keys: &str) -> String {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, keys, &mut buf);
    buf.into_iter().collect()
}

fn type_with_opts(opts: EngineOptions, strat: i64, keys: &str) -> String {
    let mut e = Engine::new(opts);
    e.set_context(Context {
        enabled: true,
        secure: false,
        field_role: IME_FIELD_BODY,
        caps: 0,
        hint: strat,
        app_id: String::new(),
    });
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, keys, &mut buf);
    buf.into_iter().collect()
}

fn assert_matrix(cases: &[(&str, &str)], label: &str) {
    let strategies = [
        IME_STRATEGY_PREEDIT,
        IME_STRATEGY_BACKSPACE_TYPE,
        IME_STRATEGY_FORWARD_AS_COMMIT,
    ];
    for &(typed, want) in cases {
        assert_eq!(type_and_get(typed), want, "{label} default `{typed}`");
        for method in [Method::Telex, Method::SimpleTelex] {
            for style in [DiacriticStyle::Old, DiacriticStyle::New] {
                for auto_cap in [false, true] {
                    for strat in strategies {
                        let opts = EngineOptions {
                            method,
                            diacritic_style: style,
                            auto_capitalize: auto_cap,
                            ..Default::default()
                        };
                        let got = type_with_opts(opts, strat, typed);
                        assert_eq!(
                            got, want,
                            "{label} `{typed}` ({method:?}, {style:?}, cap={auto_cap}, strat={strat}) -> `{got}`"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn e50_unikey_task_space_no_restore() {
    // User báo: "Task space ra Tassk"
    // Gõ "Task " phải ra "Task " (không restore "Tassk")
    let result = type_and_get("Task ");
    assert_eq!(
        result, "Task ",
        "Task + space phải ra 'Task ', nhưng ra '{}'",
        result
    );
}

#[test]
fn e50_dat_space_no_duplicate() {
    // User báo: "nó ra đata, space cái thì ra ddat"
    // Gõ "dat " phải ra "dat " (không restore)
    let result = type_and_get("dat ");
    assert_eq!(
        result, "dat ",
        "dat + space phải ra 'dat ', nhưng ra '{}'",
        result
    );
}

#[test]
fn e50_daat_space_no_duplicate() {
    // Gõ "daat " phải ra "daat " (không có biến đổi Telex, không restore)
    // Theo Telex: 'daat' không có phím dấu nào, giữ nguyên
    let result = type_and_get("daat ");
    assert_eq!(
        result, "daat ",
        "daat + space phải ra 'daat ', nhưng ra '{}'",
        result
    );
}

/// Test E50 core: sau auto-restore, gõ tiếp sẽ skip restore
#[test]
fn e50_after_restore_skip_further_restore() {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();

    // Gõ "text " → auto-restore (text → tẽt, restore → text)
    type_keys(&mut e, "text ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "text ",
        "text + space phải restore ra 'text '"
    );

    // Tiếp tục gõ "Task " → KHÔNG restore (skip rule Unikey)
    buf.clear();
    type_keys(&mut e, "Task ", &mut buf);
    assert_eq!(
        buf.iter().collect::<String>(),
        "Task ",
        "Sau restore, Task + space phải ra 'Task ' (skip rule Unikey)"
    );
}

#[test]
fn e60_sir_and_r_undo_matrix() {
    let cases: &[(&str, &str)] = &[
        ("Sirr ", "Sir "),
        ("sirr ", "sir "),
        ("Sirr,", "Sir,"),
        ("Sirr.", "Sir."),
        ("Sirr:", "Sir:"),
        ("Sirr!", "Sir!"),
        ("Sirr?", "Sir?"),
        ("Sirr\n", "Sir\n"),
        ("SirrVN ", "SirVN "),
        ("carr ", "car "),
        ("barr ", "bar "),
        ("forr ", "for "),
        ("varr ", "var "),
        ("herr ", "her "),
        ("perr ", "per "),
        ("farr ", "far "),
        ("tarr ", "tar "),
        ("parr ", "par "),
        ("jarr ", "jar "),
        ("earr ", "ear "),
        ("airr ", "air "),
        ("ourr ", "our "),
        ("yourr ", "your "),
        ("orr ", "or "),
        ("morre ", "more "),
        ("corre ", "core "),
        ("porrt ", "port "),
        ("sorrt ", "sort "),
        ("serrver ", "server "),
        ("warr ", "war "),
        ("affterr ", "after "),
        ("firrst ", "first "),
        ("firrsst ", "first "),
        // Multi-undo + phụ âm đôi EN thật đứng sau (`addrress` -> `address`, `proffessional` -> `professional`).
        ("addrress ", "address "),
        ("exxpress ", "express "),
        ("proffessional ", "professional "),
        ("wirreless ", "wireless "),
        ("perrmission ", "permission "),
        ("refferred ", "referred "),
        ("tarriff ", "tariff "),
        ("acrross ", "across "),
        ("progrress ", "progress "),
        ("congrress ", "congress "),
    ];
    assert_matrix(cases, "E60 r-undo");

    // Gõ Caps Lock ON (`MOD_CAPS`): chữ hoa cũng mang dấu Telex nên lặp lại hoàn tác chuẩn UniKey.
    for (typed, want) in [("SIRR ", "SIR "), ("TASSK ", "TASK "), ("CARR ", "CAR ")] {
        let mut e = Engine::new(EngineOptions::default());
        let mut buf: Vec<char> = Vec::new();
        for c in typed.chars() {
            let mut k = KeyEvent::char_down(c);
            if c.is_ascii_alphabetic() {
                k.mods = keymap::MOD_CAPS;
            }
            let out = e.key(&k);
            match out.action {
                Action::Pass => buf.push(c),
                Action::Replace {
                    delete_count,
                    insert,
                }
                | Action::Restore {
                    delete_count,
                    insert,
                } => {
                    buf.truncate(buf.len() - delete_count as usize);
                    buf.extend(insert);
                }
                Action::Commit { insert } => buf.extend(insert),
            }
        }
        assert_eq!(buf.iter().collect::<String>(), want, "MOD_CAPS `{typed}`");
    }

    // ESC và Ctrl-tap trên "Sirr" phải giữ đúng "Sir", không bung lại "Sirr".
    for (trigger, want) in [("esc", "Sir"), ("ctrl_tap", "Sir")] {
        let mut e = Engine::new(EngineOptions::default());
        let mut buf: Vec<char> = Vec::new();
        type_keys(&mut e, "Sirr", &mut buf);
        assert_eq!(buf.iter().collect::<String>(), "Sir");
        let k = if trigger == "esc" {
            KeyEvent {
                vk: keymap::vk::ESCAPE,
                key_down: true,
                ..Default::default()
            }
        } else {
            let _ = e.key(&KeyEvent {
                vk: keymap::vk::CONTROL,
                key_down: true,
                ..Default::default()
            });
            KeyEvent {
                vk: keymap::vk::CONTROL,
                key_down: false,
                ..Default::default()
            }
        };
        let out = e.key(&k);
        match out.action {
            Action::Pass => {}
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
        assert_eq!(buf.iter().collect::<String>(), want, "Sirr + {trigger}");
    }
}

#[test]
fn e60_s_and_f_undo_matrix() {
    let cases: &[(&str, &str)] = &[
        ("Tassk ", "Task "),
        ("tassk ", "task "),
        ("Tassk,", "Task,"),
        ("TasskMgr ", "TaskMgr "),
        ("gass ", "gas "),
        ("hass ", "has "),
        ("hiss ", "his "),
        ("thiss ", "this "),
        ("yess ", "yes "),
        ("iss ", "is "),
        ("uss ", "us "),
        ("oss ", "os "),
        ("wass ", "was "),
        ("tesst ", "test "),
        ("lisst ", "list "),
        ("lasst ", "last "),
        ("fasst ", "fast "),
        ("russt ", "rust "),
        ("jusst ", "just "),
        ("musst ", "must "),
        ("besst ", "best "),
        ("resst ", "rest "),
        ("cosst ", "cost "),
        ("hosst ", "host "),
        ("losst ", "lost "),
        ("mosst ", "most "),
        ("posst ", "post "),
        ("casst ", "cast "),
        ("passt ", "past "),
        ("usser ", "user "),
        ("iff ", "if "),
        ("lefft ", "left "),
        ("sofft ", "soft "),
        ("shifft ", "shift "),
        ("inffo ", "info "),
        ("affter ", "after "),
        ("swifft ", "swift "),
        ("drafft ", "draft "),
        ("conffig ", "config "),
        ("deff ", "def "),
        ("reff ", "ref "),
    ];
    assert_matrix(cases, "E60 s/f-undo");
}

#[test]
fn e60_post_consonant_unikey_undo() {
    let cases: &[(&str, &str)] = &[
        ("Linhss ", "Linhs "),
        ("Namss ", "Nams "),
        ("Tuanrr ", "Tuanr "),
        ("Binhff ", "Binhf "),
        ("docss ", "docs "),
        ("bugss ", "bugs "),
        ("logss ", "logs "),
        ("Tassks ", "Tasks "),
        ("tasks ", "tasks "),
        ("userss ", "users "),
    ];
    assert_matrix(cases, "E60 post-consonant undo");
}

#[test]
fn e60_non_adjacent_telex_markers_preserved() {
    let cases: &[(&str, &str)] = &[
        ("textbox ", "textbox "),
        ("flexbox ", "flexbox "),
        ("maxbox ", "maxbox "),
        ("doses ", "doses "),
        ("forwarded ", "forwarded "),
        ("server ", "server "),
        ("deadend ", "deadend "),
        ("border ", "border "),
        ("order ", "order "),
        ("cases ", "cases "),
        ("uses ", "uses "),
        ("faxbax ", "faxbax "),
        ("moxbax ", "moxbax "),
        ("buxdax ", "buxdax "),
    ];
    assert_matrix(cases, "E60 non-adjacent marker");
}

#[test]
fn e60_contrast_double_letters_and_triple_undo() {
    let contrast: &[(&str, &str)] = &[
        ("pass ", "pass "),
        ("class ", "class "),
        ("process ", "process "),
        ("buffer ", "buffer "),
        ("buffered ", "buffered "),
        ("crossed ", "crossed "),
        ("referred ", "referred "),
        ("current ", "current "),
        ("array ", "array "),
        ("boss ", "boss "),
        ("off ", "off "),
        ("ass ", "ass "),
        ("arr[0] ", "arr[0] "),
        ("err ", "err "),
        ("bufferedSize ", "bufferedSize "),
        ("className ", "className "),
        ("passWord ", "passWord "),
        ("www ", "www "),
        ("ieee ", "ieee "),
        ("ericsson ", "ericsson "),
        ("transsexual ", "transsexual "),
        ("css ", "css "),
        ("rss ", "rss "),
        ("error ", "error "),
        ("sorry ", "sorry "),
        ("office ", "office "),
        ("coffee ", "coffee "),
        ("staff ", "staff "),
        ("stuff ", "stuff "),
        ("diff ", "diff "),
        ("less ", "less "),
        ("loss ", "loss "),
        ("mass ", "mass "),
        ("miss ", "miss "),
        ("mess ", "mess "),
        ("cross ", "cross "),
        ("press ", "press "),
        ("dress ", "dress "),
        ("stress ", "stress "),
        ("access ", "access "),
        ("success ", "success "),
        ("address ", "address "),
        ("express ", "express "),
        ("progress ", "progress "),
    ];
    assert_matrix(contrast, "E60 contrast");

    let triple_undo: &[(&str, &str)] = &[
        ("passs ", "pass "),
        ("classs ", "class "),
        ("offf ", "off "),
        ("asss ", "ass "),
        ("bufffer ", "buffer "),
        ("errror ", "error "),
        ("arrray ", "array "),
    ];
    assert_matrix(triple_undo, "E60 triple-undo");
}

#[test]
fn e60_ambiguous_vn_preserved_and_space_complete_guard() {
    // Không gõ đúp undo thì cặp mơ hồ vẫn giữ tiếng Việt như policy cũ.
    for (typed, want) in [
        ("sir ", "sỉ "),
        ("Sir ", "Sỉ "),
        ("car ", "cả "),
        ("bar ", "bả "),
        ("var ", "vả "),
        ("gas ", "gá "),
        ("has ", "há "),
        ("his ", "hí "),
        ("this ", "thí "),
        ("list ", "lít "),
        ("last ", "lát "),
        ("if ", "ì "),
        ("or ", "ỏ "),
    ] {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            ..Default::default()
        });
        let mut buf: Vec<char> = Vec::new();
        type_keys(&mut e, typed, &mut buf);
        assert_eq!(buf.iter().collect::<String>(), want, "VN kept: `{typed}`");
    }

    // Khi bật cả suggest_vn_complete lẫn suggest_space_complete, từ EN vừa undo từ en_words10k
    // vẫn phải giữ nguyên trên Space ("Sirr " -> "Sir ", không kéo dài).
    for (typed, want) in [
        ("Sirr ", "Sir "),
        ("deff ", "def "),
        ("shifft ", "shift "),
        ("parr ", "par "),
        ("earr ", "ear "),
    ] {
        let opts = EngineOptions {
            suggest_vn_complete: true,
            suggest_space_complete: true,
            ..Default::default()
        };
        assert_eq!(
            type_with_opts(opts, IME_STRATEGY_PREEDIT, typed),
            want,
            "space_complete guard `{typed}`"
        );
    }
}

#[test]
fn e60_freq3000_unikey_undo_sweep() {
    // Quét toàn bộ từ ASCII trong data/en_freq3000.txt bị Telex gán dấu thanh (trừ Q2 double
    // `off`/`ass` đã viết đúng sẵn nếu không bấm 3 lần): khi giả lập người dùng gõ lặp đúng
    // phím dấu đó theo thói quen UniKey rồi Space, tiếng Anh phải thắng 100%.
    const SOURCE: &str = include_str!("../../data/en_freq3000.txt");
    let mut checked = 0usize;
    let mut failures = Vec::new();
    for line in SOURCE.lines() {
        let w = line.trim();
        if w.is_empty()
            || w.starts_with('#')
            || w.len() < 2
            || !w.chars().all(|c| c.is_ascii_alphabetic())
        {
            continue;
        }
        // Tìm đúng vị trí phím đầu tiên tạo dấu thanh hoặc hạ sừng/mũ nhẹ bị kẹp,
        // mô phỏng kiểu người dùng thấy hiện dấu (`sỉ`, `cả`, `hí`, `bết`...) rồi gõ lại phím dấu.
        let chars: Vec<char> = w.chars().collect();
        let folded: String = fold(&chars, Method::Telex, DiacriticStyle::New, true)
            .into_iter()
            .collect();
        if folded == w {
            continue;
        }
        // Chèn thêm 1 phím lặp tại phím dấu thanh đầu tiên (`s`/`f`/`r`/`x`/`j`) nếu nó gây đổi thanh ngay tại bước đó
        // và không tạo ra 3 ký tự giống nhau hay dính Q2 (`of`/`as`).
        for idx in 1..chars.len() {
            let k = chars[idx];
            if !matches!(k, 's' | 'f' | 'r' | 'x' | 'j') {
                continue;
            }
            let prev_f = fold(&chars[..idx], Method::Telex, DiacriticStyle::New, true);
            let with_k = fold(&chars[..=idx], Method::Telex, DiacriticStyle::New, true);
            if with_k != [prev_f.as_slice(), &[k]].concat() {
                let mut undo_seq = String::with_capacity(w.len() + 2);
                undo_seq.extend(&chars[..=idx]);
                undo_seq.push(k);
                undo_seq.extend(&chars[idx + 1..]);
                if matches!(
                    undo_seq.as_str(),
                    "off" | "ass" | "loss" | "less" | "arr" | "ess" | "marry"
                ) {
                    continue;
                }
                undo_seq.push(' ');
                let got = type_and_get(&undo_seq);
                if got != format!("{w} ") {
                    failures.push(format!("{w}: `{undo_seq}` -> {got:?}"));
                }
                checked += 1;
                break;
            }
        }
    }
    assert!(checked >= 500, "sweep chỉ phủ {checked} từ");
    assert!(
        failures.is_empty(),
        "E60 sweep fail {}/{} từ:\n{}",
        failures.len(),
        checked,
        failures.join("\n")
    );
}

fn type_with_method(method: Method, auto_capitalize: bool, keys: &str) -> String {
    let mut opts = EngineOptions {
        method,
        auto_capitalize,
        ..Default::default()
    };
    opts.diacritic_style = DiacriticStyle::New;
    type_with_opts(opts, IME_STRATEGY_PREEDIT, keys)
}

#[test]
fn e61_tone_strip_z_and_vni_0_matrix() {
    // BUG-1 (E61): `z` (Telex/SimpleTelex) và `0` (VNI) gỡ dấu thanh cuối từ phải giữ
    // phần hiển thị thay vì bung ngược phím dấu + `z`/`0` ra document, đồng thời từ EN
    // thực thụ (`resize`, `serialize`, `quartz`, `hertz`) vẫn khôi phục nguyên văn.
    for method in [Method::Telex, Method::SimpleTelex] {
        for (typed, want) in [
            ("vieetjz ", "viêt "),
            ("dduowcjz ", "đươc "),
            ("hocjz ", "hoc "),
            ("depjz ", "dep "),
            ("tetsz ", "tet "),
            ("vietsz ", "viet "),
            ("aszz ", "az "),
            ("sirzz ", "siz "),
            ("sirez ", "sie "),
            ("taskz ", "tak "),
            ("tooiszz ", "tôiz "),
            ("quartz ", "quartz "),
            ("hertz ", "hertz "),
            ("megahertz ", "megahertz "),
            ("resize ", "resize "),
            ("serialize ", "serialize "),
            ("normalize ", "normalize "),
        ] {
            assert_eq!(
                type_with_method(method, false, typed),
                want,
                "E61 {method:?} `{typed}`"
            );
        }
    }

    for (typed, want) in [
        ("d9uo7c50 ", "đươc "),
        ("viet50 ", "viet "),
        ("hoc50 ", "hoc "),
        ("a100 ", "a0 "),
    ] {
        assert_eq!(
            type_with_method(Method::Vni, false, typed),
            want,
            "E61 VNI `{typed}`"
        );
    }

    // ESC ngay sau khi gỡ dấu bằng `z` giữ nguyên `viêt` trên màn hình.
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();
    type_keys(&mut e, "vieetjz", &mut buf);
    assert_eq!(buf.iter().collect::<String>(), "viêt");
    let esc = KeyEvent {
        vk: keymap::vk::ESCAPE,
        key_down: true,
        ..Default::default()
    };
    let out = e.key(&esc);
    if let Action::Restore {
        delete_count,
        insert,
    }
    | Action::Replace {
        delete_count,
        insert,
    } = out.action
    {
        buf.truncate(buf.len() - delete_count as usize);
        buf.extend(insert);
    }
    assert_eq!(buf.iter().collect::<String>(), "viêt", "vieetjz + Esc");
}

#[test]
fn e61_double_letters_scandinavian_and_tech_compounds() {
    // BUG-2 & BUG-4: từ tiếng Anh phụ âm đôi thực thiếu trong 10k và họ `-sson` Bắc Âu
    // không bị nuốt thành từ khác hoặc rớt phụ âm đôi.
    let protected_en: &[(&str, &str)] = &[
        ("dessert ", "dessert "),
        ("desserts ", "desserts "),
        ("marry ", "marry "),
        ("married ", "married "),
        ("scarred ", "scarred "),
        ("scarring ", "scarring "),
        ("busses ", "busses "),
        ("bussing ", "bussing "),
        ("buff ", "buff "),
        ("buffing ", "buffing "),
        ("lasses ", "lasses "),
        ("diss ", "diss "),
        ("doss ", "doss "),
        ("dossed ", "dossed "),
        ("garry ", "garry "),
        ("furr ", "furr "),
        ("marrry ", "marry "),
        ("lass ", "las "),
        ("buss ", "bus "),
        ("lasser ", "laser "),
        ("johnsson ", "johnson "),
        ("furry ", "furry "),
        ("newsstand ", "newsstand "),
        ("transship ", "transship "),
        ("transsonic ", "transsonic "),
        ("andersson ", "andersson "),
        ("johansson ", "johansson "),
        ("larsson ", "larsson "),
        ("svensson ", "svensson "),
        ("carlsson ", "carlsson "),
        ("nilsson ", "nilsson "),
        ("persson ", "persson "),
        ("olsson ", "olsson "),
        ("wolff ", "wolff "),
        ("orff ", "orff "),
        ("dorff ", "dorff "),
    ];
    assert_matrix(protected_en, "E61 BUG-2/BUG-4 protected EN");

    // BUG-3: thuật ngữ tech, số nhiều/chia từ, ghép 2 từ và PascalCase sau undo UniKey.
    let tech_and_compounds: &[(&str, &str)] = &[
        ("varrss ", "vars "),
        ("sirrss ", "sirs "),
        ("deffss ", "defs "),
        ("reffss ", "refs "),
        ("parrser ", "parser "),
        ("parrsers ", "parsers "),
        ("conffigs ", "configs "),
        ("deffaults ", "defaults "),
        ("spoooff ", "spoof "),
        ("forrmatter ", "formatter "),
        ("serrverless ", "serverless "),
        ("lefftmost ", "leftmost "),
        ("serrde ", "serde "),
        ("macoss ", "macos "),
        ("russtc ", "rustc "),
        ("posstgres ", "postgres "),
        ("dereff ", "deref "),
        ("unreff ", "unref "),
        ("esslint ", "eslint "),
        ("wassm ", "wasm "),
        ("axioss ", "axios "),
        ("centoss ", "centos "),
        ("varrname ", "varname "),
        ("sirrname ", "sirname "),
        ("tasskmgr ", "taskmgr "),
        ("deffval ", "defval "),
        ("reffcount ", "refcount "),
        ("conffigfile ", "configfile "),
        ("SerrdeJson ", "SerdeJson "),
        ("CurrIdx ", "CurIdx "),
        ("DereffPtr ", "DerefPtr "),
        ("MacossApp ", "MacosApp "),
    ];
    assert_matrix(tech_and_compounds, "E61 BUG-3 tech & compounds");
}

#[test]
fn e61_sentence_start_auto_cap_camel_and_switch_undo() {
    // BUG-5: auto_capitalize ở đầu câu vẫn tách camelCase sau khi phần đầu đã undo UniKey,
    // trong khi `Hello. bAn ` (OP-006 R1-4 chưa undo) vẫn giữ nguyên.
    for (typed, want) in [
        ("Hello. varrName ", "Hello. VarName "),
        ("Hello. sirrName ", "Hello. SirName "),
        ("Hello. tasskMgr ", "Hello. TaskMgr "),
        ("Hello. deffVal ", "Hello. DefVal "),
        ("Hello. reffCount ", "Hello. RefCount "),
        ("Hello. possX ", "Hello. PosX "),
        ("Hello. bAn ", "Hello. BAn "),
    ] {
        assert_eq!(type_and_get(typed), want, "E61 BUG-5 `{typed}`");
    }

    // BUG-6: đổi dấu thanh rồi undo (`asxx`/`a122`/`a'~~`) và đổi mũ<->sừng/trăng rồi undo
    // (`aaww`/`fowoo`/`a688`/`a^((`).
    for (method, typed, want) in [
        (Method::Telex, "asxx ", "ax "),
        (Method::Telex, "aaww ", "aw "),
        (Method::Telex, "fowoo ", "foo "),
        (Method::Vni, "a122 ", "a2 "),
        (Method::Vni, "a688 ", "a8 "),
        (Method::Viqr, "a'~~ ", "a~ "),
        (Method::Viqr, "a^(( ", "a( "),
    ] {
        assert_eq!(
            type_with_method(method, false, typed),
            want,
            "E61 BUG-6 {method:?} `{typed}`"
        );
    }

    // BUG-9 & BUG-10: non-adjacent stroke undo từ `dd`/`d9`, giữ `deadend`, và `dduowok`.
    assert_eq!(
        type_with_method(Method::Telex, false, "ddad "),
        "dad ",
        "E61 BUG-9 ddad"
    );
    assert_eq!(
        type_with_method(Method::Telex, false, "ddongd "),
        "dongd ",
        "E61 BUG-9 ddongd"
    );
    assert_eq!(
        type_with_method(Method::Telex, false, "deadend "),
        "deadend ",
        "E61 BUG-9 deadend contrast"
    );
    assert_eq!(
        type_with_method(Method::Vni, false, "d9a9 "),
        "da9 ",
        "E61 BUG-9 VNI d9a9"
    );
    assert_eq!(
        type_with_method(Method::Telex, false, "dduowok "),
        "dduowok ",
        "E61 BUG-10 dduowok"
    );
}

#[test]
fn e61_space_complete_guard_for_undone_tokens() {
    // BUG-7 + E61 Audit BUG-1: suggest_vn_complete + suggest_space_complete với vk::SPACE thật
    // không được nuốt token vừa undo UniKey hay gỡ dấu `z` ở Space.
    for (typed, want) in [
        ("majj ", "maj "),
        ("projj ", "proj "),
        ("subjj ", "subj "),
        ("adjj ", "adj "),
        ("errr ", "err "),
        ("Tassk ", "Task "),
        ("tesst ", "test "),
        ("aszz ", "az "),
        ("hocjz ", "hoc "),
        ("depjz ", "dep "),
        ("vietsz ", "viet "),
        ("sirzz ", "siz "),
        ("sirez ", "sie "),
        ("taskz ", "tak "),
        ("vieetjz ", "viêt "),
        ("dduowcjz ", "đươc "),
    ] {
        let opts = EngineOptions {
            suggest_vn_complete: true,
            suggest_space_complete: true,
            ..Default::default()
        };
        assert_eq!(
            type_with_opts(opts, IME_STRATEGY_PREEDIT, typed),
            want,
            "E61 Space-complete guard `{typed}`"
        );
    }

    // E61 Audit BUG-2: stderr, serverless và các từ EN phụ âm đôi thật / chia thì giữ nguyên vẹn.
    let audit_en: &[(&str, &str)] = &[
        ("stderr ", "stderr "),
        ("serverless ", "serverless "),
        ("perror ", "perror "),
        ("barred ", "barred "),
        ("barring ", "barring "),
        ("preferring ", "preferring "),
        ("transferring ", "transferring "),
        ("gassed ", "gassed "),
        ("gasses ", "gasses "),
        ("gassing ", "gassing "),
        ("masses ", "masses "),
        ("massed ", "massed "),
        ("bossed ", "bossed "),
        ("bosses ", "bosses "),
        ("tossed ", "tossed "),
        ("tosses ", "tosses "),
        ("messed ", "messed "),
        ("messes ", "messes "),
        ("passcode ", "passcode "),
        ("passkey ", "passkey "),
        ("passthrough ", "passthrough "),
        ("passable ", "passable "),
        ("compass ", "compass "),
        ("compress ", "compress "),
        ("repress ", "repress "),
        ("regress ", "regress "),
        ("redress ", "redress "),
        ("waitress ", "waitress "),
        ("fortress ", "fortress "),
        ("hostess ", "hostess "),
        ("goddess ", "goddess "),
        ("forrester ", "forrester "),
        ("caress ", "caress "),
        ("recess ", "recess "),
        ("sparrow ", "sparrow "),
        ("ramirez ", "ramirez "),
        ("horowitz ", "horowitz "),
        ("suarez ", "suarez "),
        ("alvarez ", "alvarez "),
        ("schwarz ", "schwarz "),
        ("kurtz ", "kurtz "),
        ("ress ", "res "),
        ("sirr ", "sir "),
        ("reff ", "ref "),
    ];
    assert_matrix(audit_en, "E61 Audit BUG-2 EN/tech integrity");

    // E61 Audit BUG-3 & BUG-4: ươ->uô qua 'o' giữ nguyên 'w' (`chuowot`, `muowoc`, `thuowo`),
    // fallback `userssa`/`asffa`/`carffa` giữ nguyên, và `aszzBar` giữ `azBar`.
    assert_eq!(
        type_with_method(Method::Telex, false, "chuowot "),
        "chuowot "
    );
    assert_eq!(type_with_method(Method::Telex, false, "muowoc "), "muowoc ");
    assert_eq!(
        type_with_method(Method::Telex, false, "userssa "),
        "userssa "
    );
    assert_eq!(type_with_method(Method::Telex, false, "asffa "), "asffa ");
    assert_eq!(type_with_method(Method::Telex, false, "carffa "), "carffa ");
    assert_eq!(type_with_method(Method::Telex, false, "aszzBar "), "azBar ");

    // E61 Pass-2 Audit: auto_capitalize + tone_strip + camelCase (`Hello. aszzBar`),
    // từ EN thực (hisses/referrer/cutlass/windlass/focussed/biassed/martinez/ortiz),
    // unstressed -er undo (bufferring/filterred/caterred) và uo_confirm_w khi `o` đổi `ươ`->`uô`.
    for (typed, want) in [
        ("Hello. aszzBar ", "Hello. AzBar "),
        ("Hello. tetszBar ", "Hello. TetBar "),
        ("Hello. vieetjzBar ", "Hello. ViêtBar "),
        ("hisses ", "hisses "),
        ("referrer ", "referrer "),
        ("preferrer ", "preferrer "),
        ("transferrer ", "transferrer "),
        ("occurrer ", "occurrer "),
        ("cutlass ", "cutlass "),
        ("windlass ", "windlass "),
        ("focussed ", "focussed "),
        ("focussing ", "focussing "),
        ("biassed ", "biassed "),
        ("biasses ", "biasses "),
        ("martinez ", "martinez "),
        ("fernandez ", "fernandez "),
        ("hernandez ", "hernandez "),
        ("rodriguez ", "rodriguez "),
        ("cortez ", "cortez "),
        ("ortiz ", "ortiz "),
        ("juarez ", "juarez "),
        ("bufferring ", "buffering "),
        ("bufferred ", "buffered "),
        ("filterred ", "filtered "),
        ("differred ", "differed "),
        ("differring ", "differing "),
        ("renderred ", "rendered "),
        ("renderring ", "rendering "),
        ("dduowoot ", "dduoot "),
        ("dduowoww ", "dduow "),
    ] {
        assert_eq!(type_and_get(typed), want, "E61 Pass-2 `{typed}`");
    }

    let mut e = Engine::new(EngineOptions::default());
    let mut buf = Vec::new();
    type_keys(&mut e, "thuowo", &mut buf);
    let out = e.key(&KeyEvent::key_down(keymap::vk::ESCAPE));
    if let Action::Restore {
        delete_count,
        insert,
    }
    | Action::Replace {
        delete_count,
        insert,
    } = out.action
    {
        buf.truncate(buf.len() - delete_count as usize);
        buf.extend(insert);
    }
    assert_eq!(buf.iter().collect::<String>(), "thuowo");
}
