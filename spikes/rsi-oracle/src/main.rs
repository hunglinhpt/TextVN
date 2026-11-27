// SPDX-License-Identifier: GPL-3.0-or-later
//! SPK-RSI — spike cho RSI-1 (P0-7 Luồng R, P0-8 §2): đo **KSR-oracle** thật của bộ dự đoán
//! và so với con số `bench_dream_rsi` đang dùng trong gate G19c / README (22%).
//!
//! Hai "người dùng mô phỏng" chạy trên CÙNG chuỗi phím:
//! - **bench** — chép đúng chính sách `tools/bench::bench_dream_rsi`: gõ nửa từ, có chữ mờ là
//!   nhấn Tab (kể cả chữ mờ sai), không sửa. Đếm thêm số Tab nhận SAI mà bench không thấy.
//! - **oracle** (Trnka & McCoy 2008) — tại MỌI phím nhìn chữ mờ (`peek_completion`, đúng thứ
//!   adapter vẽ); chỉ nhấn Tab khi `văn bản + chữ mờ` là tiền tố của văn bản đích và kết thúc ở
//!   ranh giới từ; chữ mờ hiện mà không khớp = +1 nhiễu. Gồm chuỗi R2-173 (Tab nối tiếp).
//!
//! Văn bản đích = kết quả gõ CÙNG phím với dự đoán TẮT (engine tự làm oracle chính tả), nên
//! mọi khác biệt chỉ đến từ dự đoán. G8: spike chỉ chạy trên corpus của chính nó (`eval/`),
//! in counter; chỉ in chữ khi corpus tự mâu thuẫn (lỗi dữ liệu dev, không phải phím người dùng).
//!
//! Chạy: `cargo run --release --manifest-path spikes/rsi-oracle/Cargo.toml`
//! (`-- --verbose` in thêm từng câu lệch đích).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

/// Chép nguyên `tools/bench::RSI_WORDS` để tái lập đúng số 22% (kiểm chứng mô phỏng).
const RSI_WORDS: [&str; 16] = [
    "duocj", "tooi", "lamf", "vaof", "khoong", "ddeer", "vangf", "moocj", "release", "report",
    "team", "project", "client", "contract", "review", "meeting",
];

// ---------------------------------------------------------------------------------------
// Bộ mô phỏng văn bản (bản sao thứ 13 của `apply` trong repo — ghi nhận ở README spike:
// RSI-1 phải đưa nó vào MỘT chỗ dùng chung, G22).
// ---------------------------------------------------------------------------------------

fn apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            if let Some(c) = k.printable() {
                if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                    buf.push(c);
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
            let del = *delete_count as usize;
            assert!(del <= buf.len(), "delete_count vượt buffer");
            buf.truncate(buf.len() - del);
            buf.extend_from_slice(insert);
        }
        Action::Commit { insert } => buf.extend_from_slice(insert),
    }
}

fn press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    apply(buf, &out.action, k);
}

fn special(code: u32, ch: char) -> KeyEvent {
    KeyEvent {
        vk: code,
        ch: ch as u32,
        key_down: true,
        ..Default::default()
    }
}
fn tab() -> KeyEvent {
    special(vk::TAB, '\t')
}
fn space() -> KeyEvent {
    special(vk::SPACE, ' ')
}
fn enter() -> KeyEvent {
    special(vk::RETURN, '\n')
}

// ---------------------------------------------------------------------------------------
// Telex: chữ Việt có dấu → chuỗi phím (kiểu gõ dấu cuối từ, như người dùng UniKey phổ biến).
// ---------------------------------------------------------------------------------------

const VOWELS: [(&str, &str); 12] = [
    ("aàáảãạ", "a"),
    ("ăằắẳẵặ", "aw"),
    ("âầấẩẫậ", "aa"),
    ("eèéẻẽẹ", "e"),
    ("êềếểễệ", "ee"),
    ("iìíỉĩị", "i"),
    ("oòóỏõọ", "o"),
    ("ôồốổỗộ", "oo"),
    ("ơờớởỡợ", "ow"),
    ("uùúủũụ", "u"),
    ("ưừứửữự", "uw"),
    ("yỳýỷỹỵ", "y"),
];
const TONE_KEYS: [char; 6] = ['\0', 'f', 's', 'r', 'x', 'j'];

fn telex(word: &str) -> String {
    let mut out = String::new();
    let mut tone = '\0';
    for c in word.chars() {
        if c == 'đ' {
            out.push_str("dd");
            continue;
        }
        let mut hit = false;
        for (set, base) in VOWELS {
            if let Some(pos) = set.chars().position(|v| v == c) {
                out.push_str(base);
                if pos > 0 {
                    tone = TONE_KEYS[pos];
                }
                hit = true;
                break;
            }
        }
        if !hit {
            out.push(c);
        }
    }
    if tone != '\0' {
        out.push(tone);
    }
    out
}

// ---------------------------------------------------------------------------------------
// Corpus → câu (chuỗi phím từng từ) + văn bản đích (gõ với dự đoán TẮT).
// ---------------------------------------------------------------------------------------

struct Sentence {
    /// Phím từng từ (không gồm dấu cách/Enter).
    words: Vec<Vec<KeyEvent>>,
    /// Văn bản đích của cả câu (kết thúc bằng '\n').
    target: Vec<char>,
    /// Vị trí trong `target` ngay sau dấu phân cách của từ i (dấu cách hoặc '\n').
    sep_end: Vec<usize>,
}

fn opts(predict: bool, learn: bool) -> EngineOptions {
    EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: predict,
        suggest_space_complete: predict,
        auto_restore_english: true,
        auto_capitalize: false,
        style_learning_enabled: learn,
        ..Default::default()
    }
}

fn keys_of(s: &str) -> Vec<KeyEvent> {
    s.chars().map(KeyEvent::char_down).collect()
}

/// Dựng câu từ danh sách chuỗi phím; đích = gõ với dự đoán tắt trên engine RIÊNG của câu.
fn build(word_keys: Vec<Vec<KeyEvent>>) -> Sentence {
    let mut e = Engine::new(opts(false, false));
    let mut d = Vec::new();
    let mut sep_end = Vec::new();
    let n = word_keys.len();
    for (i, w) in word_keys.iter().enumerate() {
        for k in w {
            press(&mut e, &mut d, k);
        }
        press(&mut e, &mut d, &if i + 1 == n { enter() } else { space() });
        sep_end.push(d.len());
    }
    Sentence {
        words: word_keys,
        target: d,
        sep_end,
    }
}

fn read_corpus(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("đọc {}: {e}", path.display()))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Câu văn bản → `Sentence`; bỏ câu mà gõ Telex (dự đoán tắt) không ra đúng chữ (lỗi bộ đổi
/// Telex hoặc luật engine — in ra để sửa corpus).
fn build_text(lines: &[String], is_vi: bool, verbose: bool, rejected: &mut usize) -> Vec<Sentence> {
    let mut out = Vec::new();
    for (ln, line) in lines.iter().enumerate() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let keys: Vec<Vec<KeyEvent>> = words
            .iter()
            .map(|w| keys_of(&if is_vi { telex(w) } else { (*w).to_owned() }))
            .collect();
        let s = build(keys);
        let want: String = format!("{}\n", words.join(" "));
        let got: String = s.target.iter().collect();
        if got != want {
            *rejected += 1;
            if verbose {
                eprintln!(
                    "  corpus dòng {}: Telex không tái lập đúng chữ — bỏ",
                    ln + 1
                );
                eprintln!("    muốn {want:?}\n    được {got:?}");
            }
            continue;
        }
        out.push(s);
    }
    out
}

// ---------------------------------------------------------------------------------------
// Số đo
// ---------------------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct Stat {
    baseline_keys: u64,
    actual_keys: u64,
    accepts: u64,
    /// bench: Tab nhận chữ mờ SAI (từ sau Tab ≠ từ đích) — bench hiện không đo.
    wrong_accepts: u64,
    /// oracle: số vị trí phím có chữ mờ.
    ghosts_shown: u64,
    /// oracle: chữ mờ đúng (dẫn tới văn bản đích).
    ghosts_good: u64,
    /// oracle: chữ mờ hiện khi văn bản hiện tại CÒN là tiền tố của đích (loại trạng thái Telex
    /// trung gian như `vơ` trước phím dấu — ở đó không chữ mờ nối-thêm nào có thể đúng).
    ghosts_judgeable: u64,
    /// oracle: chữ mờ đúng nhưng dừng giữa từ (người dùng mô phỏng không nhận).
    good_midword: u64,
    /// "chữ mờ == Tab chèn" bị vi phạm (R2-173 invariant).
    invariant_breaks: u64,
    /// câu có văn bản cuối ≠ đích sau khi nhận gợi ý.
    diverged: u64,
    max_chain: u64,
    chained_accepts: u64,
}

impl Stat {
    fn ksr(&self) -> f64 {
        if self.baseline_keys == 0 {
            return 0.0;
        }
        100.0 * (self.baseline_keys as f64 - self.actual_keys as f64) / self.baseline_keys as f64
    }
    fn add(&mut self, o: &Stat) {
        self.baseline_keys += o.baseline_keys;
        self.actual_keys += o.actual_keys;
        self.accepts += o.accepts;
        self.wrong_accepts += o.wrong_accepts;
        self.ghosts_shown += o.ghosts_shown;
        self.ghosts_good += o.ghosts_good;
        self.ghosts_judgeable += o.ghosts_judgeable;
        self.good_midword += o.good_midword;
        self.invariant_breaks += o.invariant_breaks;
        self.diverged += o.diverged;
        self.max_chain = self.max_chain.max(o.max_chain);
        self.chained_accepts += o.chained_accepts;
    }
}

fn baseline_keys(s: &Sentence) -> u64 {
    s.words.iter().map(|w| w.len() as u64 + 1).sum()
}

/// Chính sách của `bench_dream_rsi` hiện tại (gõ nửa từ → Tab nếu có chữ mờ bất kỳ).
fn run_bench(e: &mut Engine, s: &Sentence) -> Stat {
    let mut st = Stat {
        baseline_keys: baseline_keys(s),
        ..Default::default()
    };
    let mut d: Vec<char> = Vec::new();
    let n = s.words.len();
    for (i, w) in s.words.iter().enumerate() {
        let start = d.len();
        let half = (w.len() / 2).max(1);
        for k in &w[..half] {
            press(e, &mut d, k);
            st.actual_keys += 1;
        }
        if e.peek_completion().is_some() {
            press(e, &mut d, &tab());
            st.actual_keys += 1;
            st.accepts += 1;
            let word_start = if i == 0 { 0 } else { s.sep_end[i - 1] };
            let want = &s.target[word_start..s.sep_end[i] - 1];
            if d.get(start..) != Some(want) {
                st.wrong_accepts += 1;
                if debug() && st.wrong_accepts <= 3 {
                    let got: String = d[start..].iter().collect();
                    let w: String = want.iter().collect();
                    eprintln!("    [bench sai] đích {w:?} · sau Tab {got:?}");
                }
            }
        } else {
            for k in &w[half..] {
                press(e, &mut d, k);
                st.actual_keys += 1;
            }
        }
        press(e, &mut d, &if i + 1 == n { enter() } else { space() });
        st.actual_keys += 1;
    }
    if d != s.target {
        st.diverged += 1;
    }
    st
}

/// Người dùng oracle: nhìn chữ mờ tại mọi phím, chỉ nhận khi đúng và hết ở ranh giới từ.
fn run_oracle(e: &mut Engine, s: &Sentence) -> Stat {
    let mut st = Stat {
        baseline_keys: baseline_keys(s),
        ..Default::default()
    };
    let mut d: Vec<char> = Vec::new();
    let n = s.words.len();
    let (mut i, mut j) = (0usize, 0usize);
    let mut chain = 0u64;
    while i < n {
        if let Some(g) = e.peek_completion() {
            st.ghosts_shown += 1;
            if s.target.starts_with(&d) {
                st.ghosts_judgeable += 1;
            }
            let mut cand = d.clone();
            cand.extend(g.chars());
            if cand.len() > d.len() && s.target.starts_with(&cand) {
                st.ghosts_good += 1;
                let l = cand.len();
                // Hết đúng ở cuối từ k (trước dấu phân cách) hoặc ngay sau dấu phân cách.
                let resume = (i..n).find_map(|k| {
                    if s.sep_end[k] - 1 == l {
                        Some((k, s.words[k].len()))
                    } else if s.sep_end[k] == l && k + 1 < n {
                        Some((k + 1, 0))
                    } else {
                        None
                    }
                });
                if let Some((ni, nj)) = resume {
                    press(e, &mut d, &tab());
                    st.actual_keys += 1;
                    st.accepts += 1;
                    if chain > 0 {
                        st.chained_accepts += 1;
                    }
                    chain += 1;
                    st.max_chain = st.max_chain.max(chain);
                    if d != cand {
                        st.invariant_breaks += 1;
                        st.diverged += 1;
                        return st; // văn bản đã lệch — dừng câu (đếm phần còn lại là không tiết kiệm)
                    }
                    i = ni;
                    j = nj;
                    continue;
                }
                st.good_midword += 1;
            }
        }
        chain = 0;
        let w = &s.words[i];
        if j < w.len() {
            press(e, &mut d, &w[j]);
            j += 1;
        } else {
            press(e, &mut d, &if i + 1 == n { enter() } else { space() });
            i += 1;
            j = 0;
        }
        st.actual_keys += 1;
    }
    if d != s.target {
        st.diverged += 1;
        if debug() {
            let a: String = s.target.iter().collect();
            let b: String = d.iter().collect();
            eprintln!(
                "    [oracle lệch] đích {a:?}
                  được {b:?}"
            );
        }
    }
    st
}

fn debug() -> bool {
    std::env::var_os("SPK_RSI_DEBUG").is_some()
}

fn run(
    corpus: &[Sentence],
    oracle: bool,
    learn: bool,
    passes: usize,
) -> Vec<(Stat, (u64, u64, u64))> {
    let mut e = Engine::new(opts(true, learn));
    (0..passes)
        .map(|_| {
            let before = e.dream_rsi_snapshot();
            let mut tot = Stat::default();
            for s in corpus {
                let st = if oracle {
                    run_oracle(&mut e, s)
                } else {
                    run_bench(&mut e, s)
                };
                tot.add(&st);
            }
            let after = e.dream_rsi_snapshot();
            let rsi = (
                after.accepts - before.accepts,
                after.chars_saved - before.chars_saved,
                after.keys_typed - before.keys_typed,
            );
            (tot, rsi)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Chồng lấn corpus đánh giá ↔ dữ liệu dự đoán (RSI-2: tập kiểm tra phải tách)
// ---------------------------------------------------------------------------------------

fn trigrams(text: &str) -> Vec<String> {
    let w: Vec<&str> = text.split_whitespace().collect();
    w.windows(3).map(|t| t.join(" ")).collect()
}

fn data_trigrams(root: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    for f in [
        "data/vn_phrases.tsv",
        "data/en_phrases.tsv",
        "data/en_office_phrases.tsv",
    ] {
        let Ok(txt) = std::fs::read_to_string(root.join(f)) else {
            continue;
        };
        for line in txt.lines().filter(|l| !l.starts_with('#')) {
            let mut cols = line.split('\t');
            let joined = format!(
                "{} {}",
                cols.next().unwrap_or(""),
                cols.next().unwrap_or("")
            )
            .to_lowercase();
            set.extend(trigrams(&joined));
        }
    }
    set
}

fn overlap(lines: &[String], data: &HashSet<String>) -> (usize, usize) {
    let hit = lines
        .iter()
        .filter(|l| trigrams(l).iter().any(|t| data.contains(t)))
        .count();
    (hit, lines.len())
}

// ---------------------------------------------------------------------------------------

fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

fn report(name: &str, st: &Stat, rsi: (u64, u64, u64), oracle: bool) {
    println!(
        "  {name:<34} KSR {:>5.1}%  ({}/{} phím)  nhận {:>3}",
        st.ksr(),
        st.baseline_keys - st.actual_keys.min(st.baseline_keys),
        st.baseline_keys,
        st.accepts
    );
    if oracle {
        let noise = st.ghosts_shown - st.ghosts_good;
        println!(
            "  {:<34} precision@ghost {:>5.1}% (xét được {:>5.1}%) · nhiễu {:>5.1}/100 phím · đúng-giữa-từ {} · chuỗi nối {} (sâu nhất {})",
            "",
            pct(st.ghosts_good, st.ghosts_shown),
            pct(st.ghosts_good, st.ghosts_judgeable),
            pct(noise, st.actual_keys),
            st.good_midword,
            st.chained_accepts,
            st.max_chain
        );
    } else {
        println!(
            "  {:<34} Tab nhận SAI {} / {} ({:.0}%) — bench hiện tính là tiết kiệm",
            "",
            st.wrong_accepts,
            st.accepts,
            pct(st.wrong_accepts, st.accepts)
        );
    }
    let engine_rate = pct(rsi.1, rsi.1 + rsi.2);
    println!(
        "  {:<34} engine DreamRsi: accepts {} · chars_saved {} · keys_typed {} · savings_rate {:.1}%{}{}",
        "",
        rsi.0,
        rsi.1,
        rsi.2,
        engine_rate,
        if st.invariant_breaks > 0 {
            format!(" · VI PHẠM chữ mờ≠Tab {}", st.invariant_breaks)
        } else {
            String::new()
        },
        if st.diverged > 0 {
            format!(" · câu lệch đích {}", st.diverged)
        } else {
            String::new()
        }
    );
}

fn main() {
    let verbose = std::env::args().any(|a| a == "--verbose");
    // `SPK_RSI_ROOT` = gốc repo khi chạy binary build ở máy khác (CI: build chéo x64 → chạy
    // trên windows-11-arm để so tất định đa kiến trúc — PERF-2).
    let root: PathBuf = std::env::var_os("SPK_RSI_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let here: PathBuf = root.join("spikes/rsi-oracle");

    // Corpus 1: đúng chuỗi phím của bench hiện tại (16 từ × 3 vòng, mỗi vòng kết bằng Enter).
    let bench_corpus: Vec<Sentence> = (0..3)
        .map(|_| build(RSI_WORDS.iter().map(|w| keys_of(w)).collect()))
        .collect();

    let vi_lines = read_corpus(&here.join("eval/heldout_vi.txt"));
    let en_lines = read_corpus(&here.join("eval/heldout_en.txt"));
    let mut rej_vi = 0;
    let mut rej_en = 0;
    let vi = build_text(&vi_lines, true, verbose, &mut rej_vi);
    let en = build_text(&en_lines, false, verbose, &mut rej_en);

    let data = data_trigrams(&root);
    let (ov_vi, n_vi) = overlap(&vi_lines, &data);
    let (ov_en, n_en) = overlap(&en_lines, &data);

    println!(
        "SPK-RSI — KSR-oracle vs bench_dream_rsi (rsi-oracle {})",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "corpus: bench 16 từ×3 · VI {} câu (bỏ {rej_vi}) · EN {} câu (bỏ {rej_en})",
        vi.len(),
        en.len()
    );
    println!(
        "chồng lấn trigram với data/*phrases*: VI {ov_vi}/{n_vi} câu · EN {ov_en}/{n_en} câu (RSI-2: tập kiểm tra phải = 0)"
    );
    println!();

    println!("[1] corpus của bench (tái lập số G19c)");
    let r = run(&bench_corpus, false, false, 1);
    report("bench policy (Tab mọi chữ mờ)", &r[0].0, r[0].1, false);
    let r = run(&bench_corpus, true, false, 1);
    report("oracle", &r[0].0, r[0].1, true);
    println!();

    for (name, corpus) in [
        ("VI văn phòng (held-out)", &vi),
        ("EN văn phòng (held-out)", &en),
    ] {
        println!("[2] {name}");
        let r = run(corpus, false, false, 1);
        report("bench policy", &r[0].0, r[0].1, false);
        let r = run(corpus, true, false, 1);
        report("oracle", &r[0].0, r[0].1, true);
        let r = run(corpus, true, true, 2);
        report("oracle + học (lượt 1)", &r[0].0, r[0].1, true);
        report("oracle + học (lượt 2, cùng câu)", &r[1].0, r[1].1, true);
        println!();
    }
}
