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

/// Helper: gõ một chuỗi qua engine
fn type_keys(e: &mut Engine, keys: &str, buf: &mut Vec<char>) {
    for c in keys.chars() {
        let k = KeyEvent::char_down(c);
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
                    "off" | "ass" | "loss" | "less" | "arr" | "ess"
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
