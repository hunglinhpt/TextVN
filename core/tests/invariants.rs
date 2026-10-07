// SPDX-License-Identifier: GPL-3.0-or-later
//! Bất biến của `Engine::key` mà adapter dựa vào (P0-2 §2), kiểm trên chuỗi phím
//! giả ngẫu nhiên (LCG cố định seed — tái lập được, không cần crate ngoài):
//!
//! 1. `insert`/`preedit` ≤ `MAX_TEXT` — FFI chỉ chở được 64 ký tự; cắt bớt làm `owned`
//!    lệch document và phím kế tiếp xoá lẹm sang chữ của người dùng.
//! 2. `delete_count` không bao giờ vượt quá phần text gõ sau con trỏ ban đầu — text có
//!    sẵn trong document (`PREFIX`) không bao giờ bị engine xoá.
//!
//! Chuỗi phím có cả "giữ phím lặp" (một phím lặp 30–90 lần) để chạm giới hạn độ dài từ.

use textvn_core::keymap::vk;
use textvn_core::{
    Action, Context, Engine, EngineOptions, KeyEvent, Method, OutputCharset, MAX_TEXT,
};

const PREFIX: &str = "«giữ»";

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.next() as usize % xs.len()]
    }
}

#[derive(Clone, Copy)]
enum K {
    Ch(char),
    Vk(u32),
}

fn key_event(k: K) -> KeyEvent {
    match k {
        K::Ch(' ') => KeyEvent {
            vk: vk::SPACE,
            ch: ' ' as u32,
            key_down: true,
            ..Default::default()
        },
        K::Ch(c) => KeyEvent::char_down(c),
        K::Vk(v) => KeyEvent::key_down(v),
    }
}

/// Document = `PREFIX` + text gõ; con trỏ luôn ở cuối.
fn apply(doc: &mut Vec<char>, base: usize, o: &textvn_core::Outcome, k: &KeyEvent, ctx: &str) {
    assert!(
        o.preedit.len() <= MAX_TEXT,
        "{ctx}: preedit {} > MAX_TEXT",
        o.preedit.len()
    );
    match &o.action {
        Action::Pass => match k.vk {
            vk::BACK => {
                doc.pop();
            }
            vk::ESCAPE | vk::LEFT => {}
            _ => {
                if let Some(c) = k.printable() {
                    doc.push(c);
                }
            }
        },
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            assert!(
                insert.len() <= MAX_TEXT,
                "{ctx}: insert {} > MAX_TEXT",
                insert.len()
            );
            let d = usize::from(*delete_count);
            assert!(
                d <= doc.len() - base,
                "{ctx}: delete_count {d} lẹm vào text có sẵn (gõ được {})",
                doc.len() - base
            );
            doc.truncate(doc.len() - d);
            doc.extend_from_slice(insert);
        }
        Action::Commit { insert } => {
            assert!(
                insert.len() <= MAX_TEXT,
                "{ctx}: insert {} > MAX_TEXT",
                insert.len()
            );
            doc.extend_from_slice(insert);
        }
    }
}

fn run(method: Method, charset: OutputCharset, preedit: bool, seed: u64) {
    let letters: &[char] = match method {
        Method::Vni => &[
            'a', 'o', 'u', 'e', 'i', 'y', 'd', 'n', 'g', 'h', 't', 'p', '1', '2', '3', '4', '5',
            '6', '7', '8', '9', '0', 'A',
        ],
        Method::Viqr => &[
            'a', 'o', 'u', 'e', 'i', 'y', 'd', 'n', 'g', 'h', 't', 'p', '\'', '`', '?', '~', '.',
            '^', '(', '+', 'A',
        ],
        Method::Telex | Method::SimpleTelex => &[
            'a', 'o', 'u', 'e', 'i', 'y', 'd', 'n', 'g', 'h', 't', 'p', 's', 'f', 'r', 'x', 'j',
            'w', 'z', 'q', 'A', 'S',
        ],
    };
    let others = [
        K::Ch(' '),
        K::Ch(','),
        K::Vk(vk::BACK),
        K::Vk(vk::ESCAPE),
        K::Vk(vk::TAB),
        K::Vk(vk::RETURN),
        K::Vk(vk::LEFT),
    ];
    let mut e = Engine::new(EngineOptions {
        method,
        output_charset: charset,
        ..Default::default()
    });
    if preedit {
        e.set_context(Context {
            caps: strategy_caps_preedit(),
            ..Default::default()
        });
    }
    let mut rng = Lcg(seed);
    let mut doc: Vec<char> = PREFIX.chars().collect();
    let base = doc.len();
    for step in 0..3000 {
        let k = if rng.next().is_multiple_of(6) {
            rng.pick(&others)
        } else {
            K::Ch(rng.pick(letters))
        };
        // Giữ phím lặp: thỉnh thoảng một chữ lặp 30–90 lần.
        let repeat = if matches!(k, K::Ch(_)) && rng.next().is_multiple_of(40) {
            30 + rng.next() as usize % 60
        } else {
            1
        };
        for r in 0..repeat {
            let ev = key_event(k);
            // Backspace khi không còn chữ gõ: người dùng xoá text có sẵn — không thuộc
            // phạm vi bất biến (engine PASS, app xoá); bỏ qua để giữ PREFIX làm mốc.
            if ev.vk == vk::BACK && doc.len() == base {
                continue;
            }
            let o = e.key(&ev);
            let ctx =
                format!("{method:?}/{charset:?}/preedit={preedit} seed={seed} step={step}.{r}");
            apply(&mut doc, base, &o, &ev, &ctx);
        }
    }
    let head: String = doc[..base].iter().collect();
    assert_eq!(head, PREFIX);
}

fn strategy_caps_preedit() -> u32 {
    // Preedit cần adapter khai báo cả PREEDIT lẫn FIELD_DETECT (P0-3 §3.1).
    strategy::IME_CAP_PREEDIT | strategy::IME_CAP_FIELD_DETECT
}

#[test]
fn engine_never_overflows_result_or_eats_existing_text() {
    for method in [
        Method::Telex,
        Method::SimpleTelex,
        Method::Vni,
        Method::Viqr,
    ] {
        for charset in [
            OutputCharset::UnicodePrecomposed,
            OutputCharset::UnicodeDecomposed,
            OutputCharset::Tcvn3,
            OutputCharset::VniWindows,
        ] {
            for preedit in [false, true] {
                for seed in 1..=2u64 {
                    run(method, charset, preedit, seed);
                }
            }
        }
    }
}

/// Ca người dùng thật (chat): `dej` rồi giữ `p` → `đẹpppp…`. Trước đây từ vượt 64 ký
/// tự thì phím kế tiếp đòi xoá nhiều hơn chính từ đó → mất chữ phía trước.
#[test]
fn held_key_after_toned_word_keeps_previous_text() {
    for preedit in [false, true] {
        let mut e = Engine::new(EngineOptions::default());
        if preedit {
            e.set_context(Context {
                caps: strategy_caps_preedit(),
                ..Default::default()
            });
        }
        let mut doc: Vec<char> = "xin chao ".chars().collect();
        let base = doc.len();
        let typed = format!("dej{}", "p".repeat(100));
        for c in typed.chars() {
            let k = KeyEvent::char_down(c);
            let o = e.key(&k);
            apply(&mut doc, base, &o, &k, &format!("preedit={preedit}"));
        }
        let got: String = doc.iter().collect();
        assert_eq!(got, format!("xin chao dẹ{}", "p".repeat(100)));
    }
}
