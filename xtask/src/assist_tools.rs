// SPDX-License-Identifier: GPL-3.0-or-later
//! E59: port `tools/assist/*.py` sang xtask — policy repo: không Python.
//!
//! Bản đồ chuyển đổi (giữ tên lệnh gần bản cũ để docs dễ tra cứu):
//! - `kappa.py`                → `cargo xtask assist-kappa <a.tsv> <b.tsv> | --self-test`
//! - `score_real.py`           → `cargo xtask assist-score [--mode check|sentence|both]
//!                               [--verify] [--json] [--dic F] [--no-dict] [--errors F] [--clean F]`
//! - `tone_inject.py`          → `cargo xtask assist-tone-inject [--seed N] [--max N] [--no-dict] [--corpus F]`
//! - `build_real_corpus.py`    → `cargo xtask assist-build-candidates [--pool F] [--seed N]
//!                               [--sample N] [--scan-all] [--verify]`
//! - `gen_corpus_tatoeba.py`   → `cargo xtask assist-gen-vi-tatoeba [--out F] [--count N]`
//! - `gen_corpus_en_tatoeba.py`→ `cargo xtask assist-gen-en-tatoeba [--out F] [--count N] [--seed N]`
//!
//! Khác biệt cố ý so với bản Python (ghi vào docs khi sweep E59):
//! - Engine gọi **in-process** qua crate `textvn-assist` — đúng source-of-truth engine,
//!   loại bỏ hẳn lớp lỗi "binary CLI cũ hơn source" (Fassist-120) của bản subprocess.
//! - PRNG = SplitMix64 (full 64-bit seed, song ánh, deterministic) thay Mersenne Twister
//!   của Python: vẫn tái lập 100% cho cùng seed; corpus đã commit trong repo.
//! - Cache tải Tatoeba + thư mục ứng viên: `target/assist-out/` (gitignored mặc định),
//!   thay `tools/assist/.cache|.out` (thư mục Python bị xoá). Hai lệnh tải Tatoeba
//!   dùng `curl` và `bzip2` trong PATH — mọi logic xử lý còn lại thuần std.
//!
//! Không unsafe, 0 dep ngoài `textvn-assist`/`textvn-dict` (path, cùng workspace).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use textvn_assist::dict::{Dictionary, NoDictionary};
use textvn_assist::syllables::SyllableTones;
use textvn_assist::{Issue, IssueKind};

pub fn cli(args: &[String]) -> Result<(), String> {
    let code = match args.first().map(String::as_str) {
        Some("assist-kappa") => kappa_cli(&args[1..]),
        Some("assist-score") => score_cli(&args[1..]),
        Some("assist-tone-inject") => tone_inject_cli(&args[1..]),
        Some("assist-build-candidates") => build_candidates_cli(&args[1..]),
        Some("assist-gen-vi-tatoeba") => gen_vi_tatoeba_cli(&args[1..]),
        Some("assist-gen-en-tatoeba") => gen_en_tatoeba_cli(&args[1..]),
        _ => {
            eprintln!(
                "dùng: cargo xtask assist-<kappa|score|tone-inject|build-candidates|gen-vi-tatoeba|gen-en-tatoeba> ..."
            );
            return Err("lệnh assist lạ".into());
        }
    };
    if code == 0 {
        Ok(())
    } else {
        Err(format!("exit {code}"))
    }
}

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

// ---------------------------------------------------------------------------
//=== PRNG SplitMix64 (deterministic, full 64-bit seed, std-thuần) ===========
// ---------------------------------------------------------------------------
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    /// Fisher-Yates ngược trong mảng indices [0..n) — lấy `k` mẫu KHÔNG lặp.
    fn sample_indices(&mut self, n: usize, k: usize) -> Vec<usize> {
        let k = k.min(n);
        let mut idx: Vec<usize> = (0..n).collect();
        for i in 0..k {
            let j = i + (self.next_u32() as usize) % (n - i);
            idx.swap(i, j);
        }
        idx.truncate(k);
        idx
    }
    fn shuffle<T>(&mut self, xs: &mut [T]) {
        let n = xs.len();
        for i in (1..n).rev() {
            let j = (self.next_u32() as usize) % (i + 1);
            xs.swap(i, j);
        }
    }
    fn choose<'a, T>(&mut self, xs: &'a [T]) -> Option<&'a T> {
        if xs.is_empty() {
            None
        } else {
            Some(&xs[(self.next_u32() as usize) % xs.len()])
        }
    }
}

// ---------------------------------------------------------------------------
//=== Cohen's kappa (P4-12c) =================================================
// ---------------------------------------------------------------------------
fn kappa(a: &[bool], b: &[bool]) -> Result<f64, String> {
    let n = a.len();
    if n == 0 || n != b.len() {
        return Err("hai danh sách nhãn phải cùng độ dài và khác rỗng".into());
    }
    let observed = a.iter().zip(b).filter(|(x, y)| x == y).count() as f64 / n as f64;
    let pa = a.iter().filter(|x| **x).count() as f64 / n as f64;
    let pb = b.iter().filter(|x| **x).count() as f64 / n as f64;
    let expected = pa * pb + (1.0 - pa) * (1.0 - pb);
    if expected == 1.0 {
        return Ok(if observed == 1.0 { 1.0 } else { 0.0 });
    }
    Ok((observed - expected) / (1.0 - expected))
}

/// 1 dòng TSV: `<cau>\t<kind1,kind2>` (cột 2 rỗng = câu sạch). Bỏ dòng trống/`#`.
fn read_tsv_labelled(path: &Path) -> Result<Vec<(String, BTreeSet<String>)>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("không đọc được `{}`: {e}", path.display()))?;
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let (sentence, label) = match raw.split_once('\t') {
            Some((s, l)) => (s, l),
            None => (raw, ""),
        };
        let kinds: BTreeSet<String> = label
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_string)
            .collect();
        out.push((sentence.trim().to_string(), kinds));
    }
    Ok(out)
}

fn kappa_cli(args: &[String]) -> i32 {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("dùng: cargo xtask assist-kappa <ann_a.tsv> <ann_b.tsv> | --self-test");
        return 0;
    }
    if args.len() == 1 && args[0] == "--self-test" {
        return kappa_self_test();
    }
    if args.len() != 2 {
        eprintln!("dùng: cargo xtask assist-kappa <ann_a.tsv> <ann_b.tsv> (hoặc --self-test)");
        return 2;
    }
    let (a_rows, b_rows) = match (
        read_tsv_labelled(Path::new(&args[0])),
        read_tsv_labelled(Path::new(&args[1])),
    ) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };
    kappa_report(&a_rows, &b_rows)
}

fn kappa_report(
    a_rows: &[(String, BTreeSet<String>)],
    b_rows: &[(String, BTreeSet<String>)],
) -> i32 {
    if a_rows.len() != b_rows.len() {
        eprintln!(
            "ERROR: số dòng khác nhau ({} vs {})",
            a_rows.len(),
            b_rows.len()
        );
        return 2;
    }
    for (i, ((sa, _), (sb, _))) in a_rows.iter().zip(b_rows).enumerate() {
        if sa != sb {
            eprintln!("ERROR: dòng {} khác câu giữa hai file", i + 1);
            return 2;
        }
    }
    let has_a: Vec<bool> = a_rows.iter().map(|(_, k)| !k.is_empty()).collect();
    let has_b: Vec<bool> = b_rows.iter().map(|(_, k)| !k.is_empty()).collect();
    println!("Số câu: {}", a_rows.len());
    match kappa(&has_a, &has_b) {
        Ok(k) => println!("kappa 'câu có lỗi?'      : {k:.3}"),
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    }
    let mut kinds: BTreeSet<&str> = BTreeSet::new();
    for (_, ks) in a_rows.iter().chain(b_rows) {
        for k in ks {
            kinds.insert(k.as_str());
        }
    }
    for kind in kinds {
        let ka: Vec<bool> = a_rows.iter().map(|(_, ks)| ks.contains(kind)).collect();
        let kb: Vec<bool> = b_rows.iter().map(|(_, ks)| ks.contains(kind)).collect();
        let k = kappa(&ka, &kb).unwrap_or(f64::NAN);
        let a_hits = ka.iter().filter(|x| **x).count();
        let b_hits = kb.iter().filter(|x| **x).count();
        println!("  {kind:<28} kappa {k:.3}   (A={a_hits} B={b_hits})");
    }
    let diff: Vec<usize> = a_rows
        .iter()
        .zip(b_rows)
        .enumerate()
        .filter(|(_, ((_, x), (_, y)))| x != y)
        .map(|(i, _)| i + 1)
        .collect();
    if diff.is_empty() {
        println!("Dòng bất đồng: 0");
    } else {
        let head: Vec<String> = diff.iter().take(30).map(usize::to_string).collect();
        println!("Dòng bất đồng: {} -> [{}]", diff.len(), head.join(", "));
    }
    0
}

fn kappa_self_test() -> i32 {
    let t4 = [true, false, true, false];
    assert_eq!(kappa(&t4, &t4).unwrap(), 1.0);
    let a = [true, true, false, false];
    let b = [true, false, true, false];
    assert!(kappa(&a, &b).unwrap().abs() < 1e-9); // ngang máy rủi
    let x = [true, false];
    let y = [false, true];
    assert_eq!(kappa(&x, &y).unwrap(), -1.0);
    let f2 = [false, false];
    assert_eq!(kappa(&f2, &f2).unwrap(), 1.0);
    let big_a: Vec<bool> = std::iter::repeat_n(true, 8)
        .chain(std::iter::repeat_n(false, 12))
        .collect();
    let big_b: Vec<bool> = std::iter::repeat_n(true, 6)
        .chain(std::iter::repeat_n(false, 14))
        .collect();
    let k = kappa(&big_a, &big_b).unwrap();
    assert!((0.5..1.0).contains(&k), "{k}");
    assert!(kappa(&[], &[]).is_err());
    println!("kappa self-test: OK");
    0
}

// ---------------------------------------------------------------------------
//=== Đo recall/FPR trên corpus có nhãn (P4-12, D19) ========================
// ---------------------------------------------------------------------------
struct Row {
    sentence: String,
    kinds: Vec<String>,
    soft: bool,
}

fn read_labelled_soft(path: &Path) -> Result<Vec<Row>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("ERROR: thiếu file corpus `{}`: {e}", path.display()))?;
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        let (mut sentence, label) = match raw.split_once('\t') {
            Some((s, l)) => (s.to_string(), l),
            None => (raw.to_string(), ""),
        };
        let trimmed = sentence.trim_start();
        let soft = trimmed.starts_with('~');
        sentence = trimmed.trim_start_matches('~').trim().to_string();
        let kinds: Vec<String> = label
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_string)
            .collect();
        out.push(Row {
            sentence,
            kinds,
            soft,
        });
    }
    Ok(out)
}

enum EngineDict {
    /// `--no-dict`: luật cũ — không tra từ điển (đo đối chứng).
    NoDict,
    /// Mặc định: từ điển dấu thanh dựng sẵn của sản phẩm (đúng cái FFI dùng).
    Builtin,
    /// `--dic ...`: từ điển ngoài (`.fst` hoặc `.dic` hunspell).
    Loaded(Box<ArcDict>),
}

struct ArcDict(std::sync::Arc<textvn_dict::Dictionary>);

impl Dictionary for ArcDict {
    fn contains(&self, word: &str) -> bool {
        self.0.contains(word)
    }
    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.0.suggest(word, limit)
    }
}

/// Nạp từ điển như `textvn-cli assist check --dic` (một nguồn sự thật luật `--dic`).
fn load_dic_paths(paths: &[String]) -> Result<Box<ArcDict>, String> {
    if paths.len() == 1 && paths[0].ends_with(".fst") {
        let d = textvn_dict::load(&paths[0])
            .map_err(|e| format!("nạp `{}` thất bại: {e}", paths[0]))?;
        return Ok(Box::new(ArcDict(d)));
    }
    let mut builder = textvn_dict::Builder::new();
    for p in paths {
        if p.ends_with(".fst") {
            return Err("`.fst` không trộn được với `.dic` — chỉ truyền một file `.fst`".into());
        }
        let text = std::fs::read_to_string(p).map_err(|e| format!("đọc `{p}`: {e}"))?;
        for word in text
            .lines()
            .skip(1)
            .map(str::trim)
            .filter(|w| !w.is_empty())
        {
            builder.insert(&word.to_lowercase());
        }
    }
    let bytes = builder.to_bytes();
    // E44: gắn số thứ tự AtomicUsize để tránh ghi đè file đang mmap giữa nhiều lần gọi trong cùng process.
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = std::env::temp_dir().join(format!("xtask-dic-{}-{seq}.fst", std::process::id()));
    std::fs::write(&tmp, &bytes).map_err(|e| format!("ghi FST tạm thất bại: {e}"))?;
    let d = textvn_dict::load(&tmp).map_err(|e| format!("nạp FST tạm thất bại: {e}"));
    let _ = std::fs::remove_file(&tmp); // Windows không xoá được khi còn map;
    Ok(Box::new(ArcDict(d?)))
}

/// Chạy engine trong tiến trình này — mỗi câu một dòng (line = i+1 như CLI `--file`).
fn run_engine(
    sentences: &[String],
    sentence_mode: bool,
    dic: &EngineDict,
) -> Vec<BTreeSet<String>> {
    let mut out = Vec::with_capacity(sentences.len());
    for s in sentences {
        let issues: Vec<Issue> = match (sentence_mode, dic) {
            (false, EngineDict::NoDict) => textvn_assist::check_with(s, &NoDictionary),
            (true, EngineDict::NoDict) => textvn_assist::check_sentence_with(s, &NoDictionary),
            (false, EngineDict::Loaded(d)) => textvn_assist::check_with(s, d.as_ref()),
            (true, EngineDict::Loaded(d)) => textvn_assist::check_sentence_with(s, d.as_ref()),
            (false, EngineDict::Builtin) => textvn_assist::check_with(s, SyllableTones::builtin()),
            (true, EngineDict::Builtin) => {
                textvn_assist::check_sentence_with(s, SyllableTones::builtin())
            }
        };
        out.push(
            issues
                .into_iter()
                .map(|i| kind_str(i.kind).to_string())
                .collect(),
        );
    }
    out
}

fn kind_str(k: IssueKind) -> &'static str {
    // Đồng bộ `IssueKind::as_str` (không pub-export chuẩn RFC — gọi trực tiếp).
    k.as_str()
}

/// Khoảng tin cậy Wilson 95% (không phụ thuộc crate thống kê).
fn wilson(succ: usize, total: usize) -> (f64, f64) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let succ = succ.min(total);
    let z = 1.959964;
    let p = succ as f64 / total as f64;
    let denom = 1.0 + z * z / total as f64;
    let centre = (p + z * z / (2.0 * total as f64)) / denom;
    let margin = z
        * ((p * (1.0 - p) / total as f64 + z * z / (4.0 * total as f64 * total as f64)).sqrt())
        / denom;
    (0f64.max(centre - margin), 1.0f64.min(centre + margin))
}

struct Measure {
    errors: usize,
    detected_errors: usize,
    recall: f64,
    recall_lo: f64,
    recall_hi: f64,
    clean: usize,
    flagged_clean_hard: usize,
    flagged_clean_soft: usize,
    fpr: f64,
    fpr_lo: f64,
    fpr_hi: f64,
    per_kind_expected: BTreeMap<String, usize>,
    per_kind_detected: BTreeMap<String, usize>,
    fp_kinds: BTreeMap<String, usize>,
}

fn measure(errors: &[Row], clean: &[Row], sentence_mode: bool, dic: &EngineDict) -> Measure {
    let det_err = run_engine(
        &errors
            .iter()
            .map(|r| r.sentence.clone())
            .collect::<Vec<_>>(),
        sentence_mode,
        dic,
    );
    let det_clean = run_engine(
        &clean.iter().map(|r| r.sentence.clone()).collect::<Vec<_>>(),
        sentence_mode,
        dic,
    );

    let mut per_kind_expected: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_kind_detected: BTreeMap<String, usize> = BTreeMap::new();
    let mut hit = 0usize;
    for (row, found) in errors.iter().zip(&det_err) {
        for k in &row.kinds {
            *per_kind_expected.entry(k.clone()).or_insert(0) += 1;
            if found.contains(k) {
                *per_kind_detected.entry(k.clone()).or_insert(0) += 1;
            }
        }
        if row.kinds.iter().any(|k| found.contains(k)) {
            hit += 1;
        }
    }

    let mut hard = 0usize;
    let mut soft = 0usize;
    let mut fp_kinds: BTreeMap<String, usize> = BTreeMap::new();
    for (row, found) in clean.iter().zip(&det_clean) {
        if found.is_empty() {
            continue;
        }
        if row.soft {
            soft += 1;
        } else {
            hard += 1;
        }
        for k in found {
            *fp_kinds.entry(k.clone()).or_insert(0) += 1;
        }
    }

    let total_err = errors.len();
    let total_clean = clean.len();
    let recall = if total_err > 0 {
        hit as f64 / total_err as f64
    } else {
        0.0
    };
    let fpr = if total_clean > 0 {
        (hard + soft) as f64 / total_clean as f64
    } else {
        0.0
    };
    let (r_lo, r_hi) = wilson(hit, total_err);
    let (f_lo, f_hi) = wilson(hard + soft, total_clean);
    Measure {
        errors: total_err,
        detected_errors: hit,
        recall,
        recall_lo: r_lo,
        recall_hi: r_hi,
        clean: total_clean,
        flagged_clean_hard: hard,
        flagged_clean_soft: soft,
        fpr,
        fpr_lo: f_lo,
        fpr_hi: f_hi,
        per_kind_expected,
        per_kind_detected,
        fp_kinds,
    }
}

fn measure_report(mode: &str, m: &Measure) {
    println!("=== mode: {mode} ===");
    println!("Câu có lỗi (có nhãn)   : {}", m.errors);
    println!(
        "  engine phát hiện     : {}   recall = {:.1} %   (Wilson 95%: {:.1} - {:.1} %)",
        m.detected_errors,
        m.recall * 100.0,
        m.recall_lo * 100.0,
        m.recall_hi * 100.0
    );
    println!("Câu sạch               : {}", m.clean);
    println!(
        "  bị gán có (FPR)      : {}   FPR   = {:.1} %   (Wilson 95%: {:.1} - {:.1} %)",
        m.flagged_clean_hard + m.flagged_clean_soft,
        m.fpr * 100.0,
        m.fpr_lo * 100.0,
        m.fpr_hi * 100.0
    );
    println!(
        "    trong đó dòng mềm `~` (sạch nhưng engine báo nhầm): {} | dòng cứng (hồi quy FP): {}",
        m.flagged_clean_soft, m.flagged_clean_hard
    );
    println!("Recall theo loại lỗi:");
    let mut kinds: Vec<&String> = m.per_kind_expected.keys().collect();
    kinds.sort_by(|a, b| {
        let ea = m.per_kind_expected.get(*a).copied().unwrap_or(0);
        let eb = m.per_kind_expected.get(*b).copied().unwrap_or(0);
        eb.cmp(&ea).then_with(|| a.as_str().cmp(b.as_str()))
    });
    for kind in kinds {
        let exp = m.per_kind_expected[kind];
        let det = m.per_kind_detected.get(kind).copied().unwrap_or(0);
        let pct = if exp > 0 {
            det as f64 / exp as f64 * 100.0
        } else {
            0.0
        };
        println!("  {kind:<26} {det:>4}/{exp:<4} = {pct:>6.1} %");
    }
    if !m.fp_kinds.is_empty() {
        println!("Loại lỗi trên tập sạch (nguồn FPR):");
        let mut fps: Vec<(&String, usize)> = m.fp_kinds.iter().map(|(k, n)| (k, *n)).collect();
        fps.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.as_str().cmp(b.0.as_str())));
        for (kind, n) in fps {
            println!("  {kind:<26} {n:>4}");
        }
    }
}

/// Kiểm chứng tập nhãn: dòng CỨNG phải khớp engine; dòng MỀM chỉ báo cáo.
fn verify_labeled(errors: &[Row], clean: &[Row], dic: &EngineDict) -> i32 {
    let det_err = run_engine(
        &errors
            .iter()
            .map(|r| r.sentence.clone())
            .collect::<Vec<_>>(),
        false,
        dic,
    );
    let det_clean = run_engine(
        &clean.iter().map(|r| r.sentence.clone()).collect::<Vec<_>>(),
        false,
        dic,
    );
    let mut failures: Vec<String> = Vec::new();
    for (row, found) in errors.iter().zip(&det_err) {
        if row.soft {
            continue;
        }
        let mut got: Vec<&str> = found.iter().map(String::as_str).collect();
        got.sort_unstable();
        let mut want: Vec<&str> = row.kinds.iter().map(String::as_str).collect();
        want.sort_unstable();
        if got != want {
            failures.push(format!(
                "LỖI: câu `{}` -> engine {got:?}, nhãn {want:?}",
                row.sentence
            ));
        }
    }
    for (row, found) in clean.iter().zip(&det_clean) {
        if row.soft {
            continue;
        }
        if !found.is_empty() {
            let mut got: Vec<&str> = found.iter().map(String::as_str).collect();
            got.sort_unstable();
            failures.push(format!(
                "LỖI: câu sạch `{}` -> engine báo {got:?}",
                row.sentence
            ));
        }
    }
    if !failures.is_empty() {
        println!("verify: FAIL ({} dòng)", failures.len());
        for f in failures.iter().take(20) {
            println!("  {f}");
        }
        return 1;
    }
    println!("verify: OK");
    0
}

// ---------------------------------------------------------------------------
//=== `assist-score` — CLI đo recall/FPR =====================================
// ---------------------------------------------------------------------------
fn score_cli(args: &[String]) -> i32 {
    let mut mode = "check".to_string();
    let mut verify = false;
    let mut json = false;
    let mut dic: Vec<String> = Vec::new();
    let mut no_dict = false;
    let mut errors_path = None;
    let mut clean_path = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("dùng: cargo xtask assist-score [--mode check|sentence|both] [--verify] [--json] [--dic F] [--no-dict] [--errors F] [--clean F]");
                return 0;
            }
            "--mode" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some(m @ ("check" | "sentence" | "both")) => mode = m.to_string(),
                    _ => {
                        eprintln!("error: --mode nhận check|sentence|both");
                        return 2;
                    }
                }
            }
            "--verify" => verify = true,
            "--json" => json = true,
            "--no-dict" => no_dict = true,
            "--dic" => {
                i += 1;
                match args.get(i) {
                    Some(p) => dic.push(p.clone()),
                    None => {
                        eprintln!("error: --dic cần đường dẫn .fst hoặc .dic");
                        return 2;
                    }
                }
            }
            "--errors" => {
                i += 1;
                match args.get(i) {
                    Some(p) => errors_path = Some(p.into()),
                    None => {
                        eprintln!("error: --errors cần đường dẫn");
                        return 2;
                    }
                }
            }
            "--clean" => {
                i += 1;
                match args.get(i) {
                    Some(p) => clean_path = Some(p.into()),
                    None => {
                        eprintln!("error: --clean cần đường dẫn");
                        return 2;
                    }
                }
            }
            other => {
                eprintln!("error: cờ lạ `{other}`");
                eprintln!("dùng: cargo xtask assist-score [--mode check|sentence|both] [--verify] [--json] [--dic F]... [--no-dict] [--errors F] [--clean F]");
                return 2;
            }
        }
        i += 1;
    }
    if no_dict && !dic.is_empty() {
        eprintln!("error: --no-dict không dùng chung với --dic");
        return 2;
    }
    let dict = if !dic.is_empty() {
        match load_dic_paths(&dic) {
            Ok(d) => EngineDict::Loaded(d),
            Err(e) => {
                eprintln!("error: {e}");
                return 2;
            }
        }
    } else if no_dict {
        EngineDict::NoDict
    } else {
        EngineDict::Builtin
    };
    let root = repo_root();
    let errors_file = errors_path.unwrap_or_else(|| root.join("corpus/assist/vi_real_errors.ass"));
    let clean_file = clean_path.unwrap_or_else(|| root.join("corpus/assist/vi_real_clean.ass"));
    let errors = match read_labelled_soft(&errors_file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let clean = match read_labelled_soft(&clean_file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    if errors.is_empty() {
        eprintln!("ERROR: tập có nhãn rỗng");
        return 2;
    }
    if verify {
        return verify_labeled(&errors, &clean, &dict);
    }
    let modes: Vec<&str> = if mode == "both" {
        vec!["check", "sentence"]
    } else {
        vec![mode.as_str()] // "check" | "sentence"
    };
    let mut results: Vec<Measure> = Vec::new();
    for &m in &modes {
        let res = measure(&errors, &clean, m == "sentence", &dict);
        measure_report(m, &res);
        println!();
        results.push(res);
    }
    if json {
        println!("{}", render_json_measures(&results, &modes));
    }
    0
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' | '>' => {
                for unit in c.encode_utf8(&mut [0u8; 4]).bytes() {
                    out.push_str(&format!("\\u{:04x}", unit));
                }
            }
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn render_json_measures(results: &[Measure], modes: &[&str]) -> String {
    // JSON pretty (indent 2) viết tay — trợ giúp copy số liệu vào docs; không phải
    // contract API, chỉ cần người đọc được.
    let mut out = String::from("[\n");
    for (idx, m) in results.iter().enumerate() {
        let mode = modes.get(idx).copied().unwrap_or("check");
        out.push_str("  {\n");
        out.push_str(&format!("    \"mode\": \"{}\",\n", json_escape(mode)));
        out.push_str(&format!("    \"errors\": {},\n", m.errors));
        out.push_str(&format!(
            "    \"detected_errors\": {},\n",
            m.detected_errors
        ));
        out.push_str(&format!("    \"recall\": {},\n", m.recall));
        out.push_str(&format!("    \"recall_lo\": {},\n", m.recall_lo));
        out.push_str(&format!("    \"recall_hi\": {},\n", m.recall_hi));
        out.push_str(&format!("    \"clean\": {},\n", m.clean));
        out.push_str(&format!(
            "    \"flagged_clean_hard\": {},\n",
            m.flagged_clean_hard
        ));
        out.push_str(&format!(
            "    \"flagged_clean_soft\": {},\n",
            m.flagged_clean_soft
        ));
        out.push_str(&format!("    \"fpr\": {},\n", m.fpr));
        out.push_str(&format!("    \"fpr_lo\": {},\n", m.fpr_lo));
        out.push_str(&format!("    \"fpr_hi\": {},\n", m.fpr_hi));
        out.push_str("    \"per_kind_expected\": {");
        let exps: Vec<String> = m
            .per_kind_expected
            .iter()
            .map(|(k, n)| format!("\n      \"{}\": {}", json_escape(k), n))
            .collect();
        if exps.is_empty() {
            out.push_str("},\n");
        } else {
            out.push_str(&exps.join(","));
            out.push_str("\n    },\n");
        }
        out.push_str("    \"per_kind_detected\": {");
        let dets: Vec<String> = m
            .per_kind_detected
            .iter()
            .map(|(k, n)| format!("\n      \"{}\": {}", json_escape(k), n))
            .collect();
        if dets.is_empty() {
            out.push_str("},\n");
        } else {
            out.push_str(&dets.join(","));
            out.push_str("\n    },\n");
        }
        out.push_str("    \"fp_kinds\": {");
        let fps: Vec<String> = m
            .fp_kinds
            .iter()
            .map(|(k, n)| format!("\n      \"{}\": {}", json_escape(k), n))
            .collect();
        if fps.is_empty() {
            out.push_str("}\n");
        } else {
            out.push_str(&fps.join(","));
            out.push_str("\n    }\n");
        }
        out.push_str(if idx + 1 == results.len() {
            "  }\n"
        } else {
            "  },\n"
        });
    }
    out.push(']');
    out
}

// ---------------------------------------------------------------------------
//=== `assist-tone-inject` — lỗi tổng hợp đo luật dấu thanh (P4-13) =========
// ---------------------------------------------------------------------------
const TONE_ROWS: [&str; 12] = [
    "aàáảãạ",
    "ăằắẳẵặ",
    "âầấẩẫậ",
    "eèéẻẽẹ",
    "êềếểễệ",
    "iìíỉĩị",
    "oòóỏõọ",
    "ôồốổỗộ",
    "ơờớởỡợ",
    "uùúủũụ",
    "ưừứửữự",
    "yỳýỷỹỵ",
];

fn tone_tables() -> &'static (BTreeMap<char, char>, BTreeMap<char, usize>) {
    static T: OnceLock<(BTreeMap<char, char>, BTreeMap<char, usize>)> = OnceLock::new();
    T.get_or_init(|| {
        let mut base = BTreeMap::new();
        let mut tone = BTreeMap::new();
        for row in TONE_ROWS {
            let b = row.chars().next().expect("hàng tone rỗng?");
            // Python: {c: row[0] for c in row} — tính CẢ row[0] (chữ gốc cũng là
            // khóa bảng để strip_tone/with_tone nhận diện nguyên âm không dấu).
            for (i, c) in row.chars().enumerate() {
                base.insert(c, b);
                tone.insert(c, i);
            }
        }
        (base, tone)
    })
}

/// Bỏ dấu thanh của 1 từ → (khóa không dấu, dấu hiện tại, số dấu trong từ).
fn strip_tone(word: &str) -> (String, usize, usize) {
    let (base, tone) = tone_tables();
    let mut key = String::with_capacity(word.len());
    let mut cur_tone = 0usize;
    let mut marks = 0usize;
    for c in word.chars() {
        if let Some(b) = base.get(&c) {
            key.push(*b);
            if let Some(t) = tone.get(&c).copied().filter(|t| *t > 0) {
                cur_tone = t;
                marks += 1;
            }
        } else {
            key.push(c);
        }
    }
    (key, cur_tone, marks)
}

/// Đặt dấu `tone` lên nguyên âm cuối của `word` (đã bỏ dấu thanh cũ).
fn with_tone(word: &str, tone: usize) -> Option<String> {
    let (base, _) = tone_tables();
    let row_for = |b: char| TONE_ROWS.iter().find(|r| r.starts_with(b)).copied();
    let mut chars: Vec<char> = word.chars().collect();
    let idx: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| base.contains_key(c))
        .map(|(i, _)| i)
        .collect();
    if idx.is_empty() {
        return None;
    }
    for &i in &idx {
        chars[i] = base[&chars[i]]; // về dạng không dấu
    }
    let j = *idx.last().expect("idx rỗng đã chặn");
    let row = row_for(chars[j])?;
    let new_char = row.chars().nth(tone)?;
    chars[j] = new_char;
    Some(chars.into_iter().collect())
}

fn is_combining_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}'
    )
}

fn load_valid_syllables(path: &Path) -> Result<BTreeSet<(String, usize)>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("không đọc được `{}`: {e}", path.display()))?;
    let mut out = BTreeSet::new();
    for raw in text.lines() {
        let line = raw.trim().to_lowercase();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, tone, marks) = strip_tone(&line);
        if marks <= 1 {
            out.insert((key, tone));
        }
    }
    Ok(out)
}

/// Đổi dấu 1 từ trong câu thành dấu KHÔNG hợp lệ → (câu lỗi, char_start, char_end).
/// Port `tone_inject.py inject()`: chọn từ ≥3 ký tự, toàn thường, NFC (bỏ qua từ
/// chứa dấu kết hợp NFD), có nguyên âm trong bảng, và khóa là âm tiết hợp lệ nào đó.
fn tone_inject_one(
    sentence: &str,
    valid: &BTreeSet<(String, usize)>,
    rng: &mut Rng,
) -> Option<(String, usize, usize)> {
    let (base_map, _) = tone_tables();
    // Quét span gồm cả chữ cái lẫn combining marks để không tách đứt từ NFD rồi làm hỏng nó.
    let mut words: Vec<(usize, usize, usize, bool)> = Vec::new(); // (byte_start, byte_end, char_start, has_comb)
    let mut cur: Option<(usize, usize, bool)> = None;
    for (char_idx, (bidx, ch)) in sentence.char_indices().enumerate() {
        let is_comb = is_combining_mark(ch);
        if ch.is_alphabetic() || is_comb {
            match &mut cur {
                Some((_, _, has_c)) => *has_c |= is_comb,
                None => cur = Some((bidx, char_idx, is_comb)),
            }
        } else if let Some((bs, cs, has_c)) = cur.take() {
            words.push((bs, bidx, cs, has_c));
        }
    }
    if let Some((bs, cs, has_c)) = cur.take() {
        words.push((bs, sentence.len(), cs, has_c));
    }

    rng.shuffle(&mut words);
    for (byte_start, byte_end, char_start, has_comb) in words {
        if has_comb {
            continue; // Bỏ qua từ NFD (tương đương filter `unicodedata.normalize("NFC", w) != w`).
        }
        let w = &sentence[byte_start..byte_end];
        if w.chars().count() < 3 || w != w.to_lowercase() {
            continue;
        }
        let (key, tone, _marks) = strip_tone(w);
        if !key.chars().any(|c| base_map.contains_key(&c)) {
            continue;
        }
        let any_valid = (0..6).any(|t| valid.contains(&(key.clone(), t)));
        if !any_valid {
            continue;
        }
        let mut bad: Vec<usize> = (0..6)
            .filter(|t| *t != tone && !valid.contains(&(key.clone(), *t)))
            .collect();
        bad.retain(|t| with_tone(&key, *t).is_some());
        let Some(t) = rng.choose(&bad).copied() else {
            continue;
        };
        let Some(wrong) = with_tone(&key, t) else {
            continue;
        };
        if wrong == w {
            continue;
        }
        let wrong_chars = wrong.chars().count();
        return Some((
            format!(
                "{}{}{}",
                &sentence[..byte_start],
                wrong,
                &sentence[byte_end..]
            ),
            char_start,
            char_start + wrong_chars,
        ));
    }
    None
}

fn tone_inject_cli(args: &[String]) -> i32 {
    let mut seed: u64 = 20261108;
    let mut max: usize = 1000;
    let mut no_dict = false;
    let mut corpus: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "dùng: cargo xtask assist-tone-inject [--seed N] [--max N] [--no-dict] [--corpus F]"
                );
                return 0;
            }
            "--seed" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => seed = v,
                    None => {
                        eprintln!("error: --seed cần số");
                        return 2;
                    }
                }
            }
            "--max" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => max = v,
                    None => {
                        eprintln!("error: --max cần số");
                        return 2;
                    }
                }
            }
            "--no-dict" => no_dict = true,
            "--corpus" => {
                i += 1;
                match args.get(i) {
                    Some(p) => corpus = Some(p.into()),
                    None => {
                        eprintln!("error: --corpus cần đường dẫn");
                        return 2;
                    }
                }
            }
            other => {
                eprintln!("error: cờ lạ `{other}`");
                eprintln!(
                    "dùng: cargo xtask assist-tone-inject [--seed N] [--max N] [--no-dict] [--corpus F]"
                );
                return 2;
            }
        }
        i += 1;
    }
    let root = repo_root();
    let corpus_file = corpus.unwrap_or_else(|| root.join("corpus/assist/vi_real_clean.ass"));
    let text = match std::fs::read_to_string(&corpus_file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("ERROR: đọc `{}`: {e}", corpus_file.display());
            return 2;
        }
    };
    let valid = match load_valid_syllables(&root.join("data/vn_syllables.txt")) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 2;
        }
    };
    let mut rng = Rng::new(seed);
    let mut cases: Vec<String> = Vec::new();
    for line in text.lines() {
        if cases.len() >= max {
            break;
        }
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        // Tách cột trên `line` chưa trim để dòng bắt đầu bằng `\tlabel` không biến label thành câu.
        let sent = line
            .split('\t')
            .next()
            .unwrap_or("")
            .trim()
            .trim_start_matches('~')
            .trim();
        if sent.is_empty() {
            continue;
        }
        if let Some((bad_sent, _, _)) = tone_inject_one(sent, &valid, &mut rng) {
            cases.push(bad_sent);
        }
    }
    let total = cases.len();
    let dict = if no_dict {
        EngineDict::NoDict
    } else {
        EngineDict::Builtin
    };
    let found = run_engine(&cases, false, &dict);
    let hit = found
        .iter()
        .filter(|kinds| {
            kinds
                .iter()
                .any(|k| k == "spelling" || k == "diacritic_typo")
        })
        .count();
    let mode = if no_dict {
        "no-dict (luật cũ)"
    } else {
        "từ điển dựng sẵn"
    };
    println!("câu tổng hợp: {total}  | cấu hình: {mode}  | seed {seed}");
    println!(
        "phát hiện (spelling|diacritic_typo): {hit}/{total} = {:.1} %",
        100.0 * hit as f64 / total.max(1) as f64
    );
    0
}

// ---------------------------------------------------------------------------
//=== `assist-build-candidates` — sinh ứng viên corpus có nhãn =============
// ---------------------------------------------------------------------------
fn fetch_cache_bytes(url: &str, cache: &Path, timeout_sec: u32) -> Result<Vec<u8>, String> {
    if !cache.is_file() {
        if let Some(parent) = cache.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir cache: {e}"))?;
        }
        // Tải vào file tạm `.part.<pid>` rồi rename atomic để không đầu độc cache khi lỗi/hủy giữa chừng.
        let part = cache.with_extension(format!("bz2.part.{}", std::process::id()));
        let st = std::process::Command::new("curl")
            .args([
                "-fsSL",
                "--max-time",
                &timeout_sec.to_string(),
                "-A",
                "textvn-audit/1.0",
                "-o",
            ])
            .arg(&part)
            .arg(url)
            .status()
            .map_err(|e| {
                let _ = std::fs::remove_file(&part);
                format!("curl (tải Tatoeba) không chạy được: {e} — cần curl trong PATH")
            })?;
        if !st.success() {
            let _ = std::fs::remove_file(&part);
            return Err(format!(
                "tải {url} thất bại (cần curl trong PATH và kết nối mạng)"
            ));
        }
        std::fs::rename(&part, cache).map_err(|e| {
            let _ = std::fs::remove_file(&part);
            format!("đổi tên cache Tatoeba thất bại: {e}")
        })?;
    }
    let out = std::process::Command::new("bzip2")
        .arg("-dc")
        .arg(cache)
        .output()
        .map_err(|e| {
            format!(
                "bzip2 -dc (giải nén Tatoeba) không chạy được: {e} — cần `bzip2` trong PATH (Git for Windows có sẵn trong usr\\bin)"
            )
        })?;
    if !out.status.success() {
        return Err(format!(
            "bzip2 -dc thất bại: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Đọc cache mtime → "yyyy-mm-dd" (UTC) — ngày snapshot để header tái lập.
fn snapshot_date(cache: &Path) -> String {
    let secs = std::fs::metadata(cache)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if secs == 0 {
        return "unknown".into();
    }
    let days = (secs / 86_400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Chuyển ngày từ epoch days (UTC) — Howard Hinnant civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

const VI_URL: &str = "https://downloads.tatoeba.org/exports/per_language/vie/vie_sentences.tsv.bz2";
const VN_MARKED: &str = "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđÀÁẢÃẠĂẰẮẲẴẶÂẦẤẨẪẬÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴĐ";
const VI_MIN_LEN: usize = 25;
const VI_MAX_LEN: usize = 200;
const VI_MAX_SENTENCES: usize = 1500;

fn vi_vietnamese_heavy(text: &str) -> bool {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return false;
    }
    let marked = letters.iter().filter(|c| VN_MARKED.contains(**c)).count();
    marked > 0 && marked * 100 >= letters.len() * 35
}

fn vi_clean_sentence(text: &str) -> Option<&str> {
    let text = text.trim();
    let len = text.chars().count();
    if !(VI_MIN_LEN..=VI_MAX_LEN).contains(&len) {
        return None;
    }
    if text.contains('\t')
        || text.contains('\n')
        || text.contains("http")
        || text.contains('<')
        || text.contains('>')
        || text.contains('|')
    {
        return None;
    }
    if !text.matches('"').count().is_multiple_of(2) {
        return None;
    }
    if !vi_vietnamese_heavy(text) {
        return None;
    }
    Some(text)
}

/// Pool câu VI: từ cache Tatoeba hoặc file `.ass` local (như --pool).
fn vi_pool_from_tatoeba(root: &Path) -> Result<Vec<String>, String> {
    let cache = root.join("target/assist-out/cache/vie_sentences.tsv.bz2");
    let raw = fetch_cache_bytes(VI_URL, &cache, 120)?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut pool: Vec<String> = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('\t');
        let _id = parts.next();
        if let (Some(_), Some(text)) = (parts.next(), parts.next()) {
            match vi_clean_sentence(text) {
                Some(clean) if !seen.contains(clean) => {
                    seen.insert(clean.to_string());
                    pool.push(clean.to_string());
                }
                _ => {}
            }
        }
    }
    Ok(pool)
}

fn pool_from_local_ass(path: &Path) -> Result<Vec<String>, String> {
    if !path.is_file() {
        return Err(format!("ERROR: không thấy file pool: {}", path.display()));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("đọc {}: {e}", path.display()))?;
    let mut seen = BTreeSet::new();
    let mut pool: Vec<String> = Vec::new();
    for raw in text.lines() {
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        // Tách cột trên `raw` chưa trim để dòng bắt đầu bằng `\tlabel` không bị ghép thành câu.
        let mut sent = raw.split('\t').next().unwrap_or("").trim().to_string();
        if let Some(stripped) = sent.strip_prefix("~ ") {
            sent = stripped.trim().to_string();
        }
        if !sent.is_empty() && !seen.contains(&sent) {
            seen.insert(sent.clone());
            pool.push(sent);
        }
    }
    Ok(pool)
}

fn build_candidates_cli(args: &[String]) -> i32 {
    let mut seed: u64 = 20261021;
    let mut sample_n: usize = 1000;
    let mut pool_path: Option<PathBuf> = None;
    let mut scan_all = false;
    let mut verify = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("dùng: cargo xtask assist-build-candidates [--pool F] [--seed N] [--sample N] [--scan-all] [--verify]");
                return 0;
            }
            "--pool" => {
                i += 1;
                match args.get(i) {
                    Some(p) => pool_path = Some(p.into()),
                    None => {
                        eprintln!("error: --pool cần đường dẫn file .ass");
                        return 2;
                    }
                }
            }
            "--seed" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => seed = v,
                    None => {
                        eprintln!("error: --seed cần số");
                        return 2;
                    }
                }
            }
            "--sample" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => sample_n = v,
                    None => {
                        eprintln!("error: --sample cần số");
                        return 2;
                    }
                }
            }
            "--scan-all" => scan_all = true,
            "--verify" => verify = true,
            other => {
                eprintln!("error: cờ lạ `{other}`");
                eprintln!("dùng: cargo xtask assist-build-candidates [--pool F] [--seed N] [--sample N] [--scan-all] [--verify]");
                return 2;
            }
        }
        i += 1;
    }
    let root = repo_root();
    if verify {
        let errors = match read_labelled_soft(&root.join("corpus/assist/vi_real_errors.ass")) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let clean = match read_labelled_soft(&root.join("corpus/assist/vi_real_clean.ass")) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        return verify_labeled(&errors, &clean, &EngineDict::Builtin);
    }
    let pool = match pool_path {
        Some(p) => pool_from_local_ass(&p),
        None => vi_pool_from_tatoeba(&root),
    };
    let pool = match pool {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: không tải được pool câu Tatoeba ({e}).");
            eprintln!("       Hãy truyền --pool <file .ass> để dùng pool local.");
            return 1;
        }
    };
    println!("pool: {} câu", pool.len());
    let mut rng = Rng::new(seed);
    let sample_idx = rng.sample_indices(pool.len(), sample_n.min(pool.len()));
    let sample: Vec<String> = sample_idx.iter().map(|&i| pool[i].clone()).collect();
    println!("mẫu: {} câu (seed={seed})", sample.len());

    let header = [
        "# ỨNG VIÊN (cần RA SOÁT TAY trước khi gán nhãn) - sinh bằng cargo xtask assist-build-candidates".to_string(),
        format!("# seed={seed} sample={} pool={}", sample.len(), pool.len()),
        "# Nhãn của engine chỉ là 'ứng viên' - nguồn sự thật là người ra soát.".to_string(),
    ];
    let mut err_lines: Vec<String> = Vec::new();
    let mut clean_lines: Vec<String> = Vec::new();
    if scan_all {
        let scanned = run_engine(&pool, false, &EngineDict::Builtin);
        let mut flagged: BTreeSet<usize> = BTreeSet::new();
        for (i, found) in scanned.iter().enumerate() {
            if !found.is_empty() {
                err_lines.push(format!("{}\t{}", pool[i], comma_join(found)));
                flagged.insert(i);
            }
        }
        for &idx in &sample_idx {
            if !flagged.contains(&idx) {
                clean_lines.push(format!("{}\t", pool[idx]));
            }
        }
    } else {
        let detected = run_engine(&sample, false, &EngineDict::Builtin);
        for (sentence, found) in sample.iter().zip(&detected) {
            if !found.is_empty() {
                err_lines.push(format!("{sentence}\t{}", comma_join(found)));
            } else {
                clean_lines.push(format!("{sentence}\t"));
            }
        }
    }
    let out_dir = root.join("target/assist-out");
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("ERROR: không tạo được {}: {e}", out_dir.display());
        return 1;
    }
    let err_path = out_dir.join("vi_real_errors.candidates.ass");
    let clean_path = out_dir.join("vi_real_clean.candidates.ass");
    if let Err(e) = std::fs::write(
        &err_path,
        format!(
            "{}\n",
            header
                .iter()
                .chain(err_lines.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        ),
    ) {
        eprintln!("ERROR: không ghi được {}: {e}", err_path.display());
        return 1;
    }
    if let Err(e) = std::fs::write(
        &clean_path,
        format!(
            "{}\n",
            header
                .iter()
                .chain(clean_lines.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        ),
    ) {
        eprintln!("ERROR: không ghi được {}: {e}", clean_path.display());
        return 1;
    }
    println!(
        "ứng viên có lỗi  -> {} ({} câu)",
        err_path.display(),
        err_lines.len()
    );
    println!(
        "ứng viên sạch    -> {} ({} câu)",
        clean_path.display(),
        clean_lines.len()
    );
    println!("Bước tiếp theo: RA SOÁT TAY từng câu ứng viên, gán nhãn loại lỗi thật,");
    println!("rồi mới tạo được corpus/assist/vi_real_errors.ass + vi_real_clean.ass đã commit.");
    0
}

fn comma_join(set: &BTreeSet<String>) -> String {
    let mut v: Vec<&str> = set.iter().map(String::as_str).collect();
    v.sort_unstable();
    v.join(",")
}

// ---------------------------------------------------------------------------
//=== `assist-gen-vi-tatoeba` — sinh corpus VI sạch từ Tatoeba (P4-0 D19) ===
// ---------------------------------------------------------------------------
fn gen_vi_tatoeba_cli(args: &[String]) -> i32 {
    let mut out: PathBuf = repo_root().join("corpus/assist/vi_tatoeba_clean.ass");
    let mut count: usize = VI_MAX_SENTENCES;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("dùng: cargo xtask assist-gen-vi-tatoeba [--out F] [--count N]");
                return 0;
            }
            "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out = p.into(),
                    None => {
                        eprintln!("error: --out cần đường dẫn");
                        return 2;
                    }
                }
            }
            "--count" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => count = v,
                    None => {
                        eprintln!("error: --count cần số");
                        return 2;
                    }
                }
            }
            other => {
                eprintln!("error: cờ lạ `{other}`");
                eprintln!("dùng: cargo xtask assist-gen-vi-tatoeba [--out F] [--count N]");
                return 2;
            }
        }
        i += 1;
    }
    let root = repo_root();
    let pool = match vi_pool_from_tatoeba(&root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    let sentences: Vec<String> = pool.into_iter().take(count).collect();
    let header = [
        "# corpus/assist/vi_tatoeba_clean.ass - câu SẠCH từ Tatoeba (văn bản thật)".to_string(),
        "# Nguồn: Tatoeba per_language export vie_sentences.tsv.bz2".to_string(),
        "# License dữ liệu: CC BY 2.0 FR (Tatoeba) - attribution bắt buộc (xem docs/compliance/)"
            .to_string(),
        format!(
            "# Sinh lặp: cargo xtask assist-gen-vi-tatoeba ({} câu)",
            sentences.len()
        ),
        "# Dùng để đo FPR trên câu ngắn tự nhiên (Fassist-80).".to_string(),
        "# Tiền tố '~ ' = câu SẠCH *có thể chứa lỗi thật* (văn bản người thật) ->".to_string(),
        "#   test KHÔNG assert 'không lỗi' cho các câu này; `assist-score` vẫn đếm".to_string(),
        "#   chúng vào mẫu số của FPR và báo riêng số câu bị gán có (ứng viên).".to_string(),
    ];
    let body: Vec<String> = sentences.iter().map(|s| format!("~ {s}\t")).collect();
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("ERROR: không tạo được {}: {e}", parent.display());
            return 1;
        }
    }
    let content = format!(
        "{}\n",
        header
            .iter()
            .chain(body.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    if let Err(e) = std::fs::write(&out, content) {
        eprintln!("ERROR: không ghi được {}: {e}", out.display());
        return 1;
    }
    println!("wrote {} sentences -> {}", sentences.len(), out.display());
    0
}

// ---------------------------------------------------------------------------
//=== `assist-gen-en-tatoeba` — sinh corpus EN sạch từ Tatoeba (EN v1) =====
// ---------------------------------------------------------------------------
const EN_URL: &str = "https://downloads.tatoeba.org/exports/per_language/eng/eng_sentences.tsv.bz2";
const EN_MIN_LEN: usize = 20;
const EN_MAX_LEN: usize = 120;
const EN_MAX_SENTENCES: usize = 300;
const EN_SEED: u64 = 20261023;
const EN_INTENTIONAL_REPEATS: [&str; 3] = ["that", "had", "very"];
const EN_EXCLUDE_IDS: [u64; 5] = [327_602, 688_587, 8_016_962, 11_177_454, 12_815_708];
const EN_STRONG_PUNCT: &str = ",.;:!?";

/// `\s+[,.;:!?]` — khoảng trắng ngay trước dấu câu mạnh.
fn en_space_before_punct(text: &str) -> bool {
    let mut prev_ws = false;
    for c in text.chars() {
        if EN_STRONG_PUNCT.contains(c) && prev_ws {
            return true;
        }
        prev_ws = c.is_whitespace();
    }
    false
}

/// `[,.;:!?][A-Za-z]` — thiếu khoảng trắng sau dấu câu.
fn en_missing_space_after(text: &str) -> bool {
    let mut prev_punct = false;
    for c in text.chars() {
        if prev_punct && c.is_ascii_alphabetic() {
            return true;
        }
        prev_punct = EN_STRONG_PUNCT.contains(c);
    }
    false
}

fn en_adjacent_words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `(?<![A-Za-z'])i(?![A-Za-z'])` — chữ "i" đơn lẻ viết thường.
fn en_lone_i(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c != 'i' {
            continue;
        }
        let before_ok = i > 0 && (chars[i - 1].is_ascii_alphabetic() || chars[i - 1] == '\'');
        let after_ok =
            i + 1 < chars.len() && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '\'');
        if !before_ok && !after_ok {
            return true;
        }
    }
    false
}

/// Viết tắt tạo tách câu "bấp" (chính sách v2):
/// (a) `\b(?:[A-Za-z]\.){2,}` (a.m., i.e., U.S.A.)
/// (b) `\b[A-Z]{2,}\.` (cố ý phân biệt chữ hoa, khắc phục bug `(?i)` cũ của regex Python)
/// (c) `\b(?:Mr|Mrs|Ms|Dr|St|Jr|Sr|vs|etc|approx|Inc|Ltd|Prof|No)\.` (bắt buộc có dấu `.` sau)
fn en_abbrev_trap(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let n = chars.len();
    for i in 0..n {
        if i > 0 && is_word(chars[i - 1]) {
            continue;
        }
        // (a) `\b(?:[A-Za-z]\.){2,}`
        if i + 3 < n
            && chars[i].is_ascii_alphabetic()
            && chars[i + 1] == '.'
            && chars[i + 2].is_ascii_alphabetic()
            && chars[i + 3] == '.'
        {
            return true;
        }
        // (b) `\b[A-Z]{2,}\.`
        if chars[i].is_ascii_uppercase() {
            let mut j = i;
            while j < n && chars[j].is_ascii_uppercase() {
                j += 1;
            }
            if j - i >= 2 && j < n && chars[j] == '.' {
                return true;
            }
        }
        // (c) `\b(?:Mr|...|No)\.` — BẮT BUỘC có dấu `.` liền sau.
        if chars[i].is_ascii_alphabetic() {
            let mut j = i;
            while j < n && chars[j].is_ascii_alphabetic() {
                j += 1;
            }
            if j < n && chars[j] == '.' {
                let word: String = chars[i..j].iter().map(|c| c.to_ascii_lowercase()).collect();
                if matches!(
                    word.as_str(),
                    "mr" | "mrs"
                        | "ms"
                        | "dr"
                        | "st"
                        | "jr"
                        | "sr"
                        | "vs"
                        | "etc"
                        | "approx"
                        | "inc"
                        | "ltd"
                        | "prof"
                        | "no"
                ) {
                    return true;
                }
            }
        }
    }
    false
}

fn en_is_mechanically_clean(text: &str) -> bool {
    if text.contains("  ") {
        return false; // double_space
    }
    if en_space_before_punct(text) {
        return false; // space_before_punct
    }
    if en_missing_space_after(text) {
        return false; // space_after_punct
    }
    let words = en_adjacent_words(text);
    for pair in words.windows(2) {
        let a = pair[0].to_lowercase();
        let b = pair[1].to_lowercase();
        if a == b && !EN_INTENTIONAL_REPEATS.contains(&a.as_str()) {
            return false; // duplicate_word
        }
    }
    if en_lone_i(text) {
        return false; // lỗi v2 nhưng không thuộc tập sạch
    }
    true
}

fn en_clean_sentence(text: &str) -> Option<&str> {
    let text = text.trim();
    let len = text.chars().count();
    if !(EN_MIN_LEN..=EN_MAX_LEN).contains(&len) {
        return None;
    }
    if text.contains('\t') || text.contains('\n') {
        return None;
    }
    if text.contains("http") || text.contains('<') || text.contains('>') || text.contains('|') {
        return None;
    }
    if text.contains('"') {
        return None; // trích dẫn = chính sách v2
    }
    let first = text.chars().next()?;
    if !first.is_ascii_alphabetic() || !first.is_ascii_uppercase() {
        return None;
    }
    if !text.ends_with(['.', '!', '?']) {
        return None;
    }
    let letters = len;
    let ascii = text.chars().filter(|c| c.is_ascii()).count();
    if ascii * 10 < letters * 9 {
        return None;
    }
    if en_abbrev_trap(text) {
        return None;
    }
    if !en_is_mechanically_clean(text) {
        return None;
    }
    if text.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
        return None; // ISUPPER
    }
    Some(text)
}

fn gen_en_tatoeba_cli(args: &[String]) -> i32 {
    let mut out: PathBuf = repo_root().join("corpus/assist/en_real_clean.ass");
    let mut count: usize = EN_MAX_SENTENCES;
    let mut seed: u64 = EN_SEED;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "dùng: cargo xtask assist-gen-en-tatoeba [--out F] [--count N] [--seed N]"
                );
                return 0;
            }
            "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out = p.into(),
                    None => {
                        eprintln!("error: --out cần đường dẫn");
                        return 2;
                    }
                }
            }
            "--count" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => count = v,
                    None => {
                        eprintln!("error: --count cần số");
                        return 2;
                    }
                }
            }
            "--seed" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(v) => seed = v,
                    None => {
                        eprintln!("error: --seed cần số");
                        return 2;
                    }
                }
            }
            other => {
                eprintln!("error: cờ lạ `{other}`");
                eprintln!(
                    "dùng: cargo xtask assist-gen-en-tatoeba [--out F] [--count N] [--seed N]"
                );
                return 2;
            }
        }
        i += 1;
    }
    let root = repo_root();
    let cache = root.join("target/assist-out/cache/eng_sentences.tsv.bz2");
    let raw = match fetch_cache_bytes(EN_URL, &cache, 300) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    let text = String::from_utf8_lossy(&raw).into_owned();
    let mut seen = BTreeSet::new();
    let mut pool: Vec<(u64, String)> = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('\t');
        let id = match parts.next().and_then(|s| s.parse::<u64>().ok()) {
            Some(v) => v,
            None => continue,
        };
        let _lang = parts.next();
        let Some(sent) = parts.next() else { continue };
        match en_clean_sentence(sent) {
            Some(clean) if !seen.contains(clean) => {
                seen.insert(clean.to_string());
                pool.push((id, clean.to_string()));
            }
            _ => {}
        }
    }
    // Lấy mẫu trên TOÀN pool (kể cả ID sẽ bị loại) để mẫu ỔN ĐỊNH khi EXCLUDE_IDS tăng.
    let mut rng = Rng::new(seed);
    let idx = rng.sample_indices(pool.len(), count.min(pool.len()));
    let sample: Vec<(u64, String)> = idx
        .into_iter()
        .map(|i| pool[i].clone())
        .filter(|(id, _)| !EN_EXCLUDE_IDS.contains(id))
        .collect();
    let excluded = count.min(pool.len()) - sample.len();
    let header = [
        "# corpus/assist/en_real_clean.ass - REAL clean English sentences (EN v1 FPR set)".to_string(),
        "# Nguồn: Tatoeba per_language export eng_sentences.tsv.bz2 (CC BY 2.0 FR)".to_string(),
        "# License dữ liệu: CC BY 2.0 FR (Tatoeba) - attribution BẮT BUỘC:".to_string(),
        "#   docs/compliance/tatoeba-attribution.md + REUSE.toml (LicenseRef-Tatoeba).".to_string(),
        format!(
            "# Sinh lặp: cargo xtask assist-gen-en-tatoeba (seed={seed}, {}/{} câu, snapshot {})",
            sample.len(),
            pool.len(),
            snapshot_date(&cache)
        ),
        "# Tính tái lập: cùng snapshot (cache target/assist-out/cache/) + seed +".to_string(),
        "# EXCLUDE_IDS trong module xtask assist_tools => file giống hết; snapshot mới (hàng tuần) =>".to_string(),
        "# pool khác - dùng snapshot đã ghi ở trên để tái lập file này.".to_string(),
        "# HUMAN-IN-LOOP: mẫu đã được ra soát ngày 2026-10-24 (agent) - sạch theo".to_string(),
        "# 5 luật EN v1; câu không đạt -> thêm ID vào EXCLUDE_IDS rồi chạy lại.".to_string(),
        "# Định dạng: <câu>\\t (cột kinds rỗng = sạch, DÒNG CỨNG -> CI assert 0 lỗi).".to_string(),
        "# '~ ' = đã ra soát là sạch nhưng engine ĐANG BÁO NHẦM (FPR thật) -".to_string(),
        "#   thêm khi engine EN có mặt và phát hiện FP trên file này.".to_string(),
        "# Chỉ dùng để đo FPR (P4-12 EN); không phải engine tự kiểm tra.".to_string(),
    ];
    let body: Vec<String> = sample.iter().map(|(_, s)| format!("{s}\t")).collect();
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("ERROR: không tạo được {}: {e}", parent.display());
            return 1;
        }
    }
    let mut content = String::new();
    for line in header.iter().chain(body.iter()) {
        content.push_str(line);
        content.push('\n');
    }
    if let Err(e) = std::fs::write(&out, content) {
        eprintln!("ERROR: không ghi được {}: {e}", out.display());
        return 1;
    }
    println!(
        "pool: {} câu | excluded-by-id trong mẫu: {}",
        pool.len(),
        excluded
    );
    println!("wrote {} sentences -> {}", sample.len(), out.display());
    0
}

// ---------------------------------------------------------------------------
//=== Unit tests =============================================================
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kappa_matches_python_selftest() {
        let t = [true, false, true, false];
        assert_eq!(kappa(&t, &t).unwrap(), 1.0);
        let a = [true, true, false, false];
        let b = [true, false, true, false];
        assert!(kappa(&a, &b).unwrap().abs() < 1e-9);
        let x = [true, false];
        let y = [false, true];
        assert_eq!(kappa(&x, &y).unwrap(), -1.0);
        let f = [false, false];
        assert_eq!(kappa(&f, &f).unwrap(), 1.0);
        let big_a: Vec<bool> = std::iter::repeat_n(true, 8)
            .chain(std::iter::repeat_n(false, 12))
            .collect();
        let big_b: Vec<bool> = std::iter::repeat_n(true, 6)
            .chain(std::iter::repeat_n(false, 14))
            .collect();
        let k = kappa(&big_a, &big_b).unwrap();
        assert!((0.5..1.0).contains(&k), "{k}");
    }

    #[test]
    fn kappa_errors_on_empty_or_mismatch() {
        assert!(kappa(&[], &[]).is_err());
        assert!(kappa(&[true], &[true, false]).is_err());
    }

    #[test]
    fn wilson_bounds() {
        assert_eq!(wilson(0, 0), (0.0, 0.0));
        let (lo, hi) = wilson(10, 10);
        assert!(lo > 0.7 && hi <= 1.0, "{lo} {hi}");
        let (lo, hi) = wilson(0, 100);
        assert!(lo == 0.0 && hi > 0.0 && hi < 0.05, "{lo} {hi}");
    }

    #[test]
    fn rng_deterministic_and_sample_distinct() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        let av: Vec<u32> = (0..8).map(|_| a.next_u32()).collect();
        let bv: Vec<u32> = (0..8).map(|_| b.next_u32()).collect();
        assert_eq!(av, bv);
        let mut r = Rng::new(7);
        let idx = r.sample_indices(100, 30);
        let mut uniq = std::collections::BTreeSet::new();
        assert_eq!(idx.len(), 30);
        for i in &idx {
            assert!(uniq.insert(*i), "index lặp: {i}");
        }
        let mut xs = vec![1, 2, 3, 4, 5];
        r.shuffle(&mut xs);
        xs.sort_unstable();
        assert_eq!(xs, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn strip_tone_and_with_tone() {
        // Bảng tone: base là index 0; dấu thanh: à=1 á=2 ả=3 ã=4 ạ=5 (hàng a).
        // "được" = đ+ư+ợ+c: 'ợ' = ơ + nặng (index 5) — key "đươc" giữ ư/ơ.
        assert_eq!(strip_tone("được"), ("đươc".to_string(), 5, 1));
        assert_eq!(strip_tone("tài"), ("tai".to_string(), 1, 1));
        assert_eq!(strip_tone("tai"), ("tai".to_string(), 0, 0));
        // with_tone đặt dấu lên NGUYÊN ÂM CUỐI (giữ nguyên logic python).
        assert_eq!(with_tone("toan", 2).unwrap(), "toán");
        assert_eq!(with_tone("toan", 5).unwrap(), "toạn");
        assert_eq!(with_tone("ta", 0).unwrap(), "ta");
        assert_eq!(with_tone("bcd", 3), None);
    }

    #[test]
    fn tone_tables_base_complete() {
        let (base, tone) = tone_tables();
        for row in TONE_ROWS {
            for c in row.chars() {
                assert!(
                    base.contains_key(&c) && tone.contains_key(&c),
                    "thiếu {c:?}"
                );
            }
        }
    }

    #[test]
    fn tone_inject_changes_one_tone() {
        let valid: BTreeSet<(String, usize)> =
            [("tai".to_string(), 3usize), ("tai".to_string(), 0usize)]
                .into_iter()
                .collect();
        let mut rng = Rng::new(2026);
        let out = tone_inject_one("thuật toán tài chính", &valid, &mut rng);
        let (bad, _, _) = out.expect("phải đổi được dấu 1 từ");
        assert_ne!(bad, "thuật toán tài chính");
        // Chỉ từ 'tài' đổi dấu (thuật/toán không nằm trong valid → bỏ qua).
        assert_eq!(bad.split_whitespace().count(), 4);
    }

    #[test]
    fn vi_heavy_and_clean() {
        // Câu thật từ corpus đã commit (sinh qua cùng filter) — đọc 1 câu đầu.
        let f = repo_root().join("corpus/assist/vi_tatoeba_clean.ass");
        let text = std::fs::read_to_string(&f).expect("thiếu corpus vi_tatoeba_clean.ass");
        let sent = text
            .lines()
            .find(|l| l.starts_with("~ "))
            .expect("corpus rỗng?")[2..]
            .split('\t')
            .next()
            .unwrap();
        assert!(
            vi_vietnamese_heavy(sent),
            "câu corpus phải pass heavy: {sent}"
        );
        assert!(
            vi_clean_sentence(sent).is_some(),
            "câu corpus phải pass filter: {sent}"
        );
        assert!(!vi_vietnamese_heavy(
            "Hello world this is plain ascii text."
        ));
        assert_eq!(vi_clean_sentence("Câu ngắn."), None);
        let long = "q".repeat(201);
        assert_eq!(vi_clean_sentence(&long), None);
        // Dấu nháy LẺ (count('"') % 2 != 0) bị loại như bản python.
        assert_eq!(
            vi_clean_sentence("\"Câu bị thiếu dấu đóng và dài trên hai mươi lăm ký tự"),
            None
        );
    }

    #[test]
    fn en_matchers() {
        assert!(en_lone_i("i think so"));
        assert!(!en_lone_i("I think so"));
        assert!(!en_lone_i("it is fine"));
        assert!(!en_lone_i("ai said"));
        assert!(en_space_before_punct("one , two"));
        assert!(!en_space_before_punct("one, two"));
        assert!(en_missing_space_after("one,two"));
        assert!(!en_missing_space_after("one, two"));
        assert!(en_abbrev_trap("they arrived at 6 a.m. and left"));
        assert!(en_abbrev_trap("the U.S. government"));
        assert!(en_abbrev_trap("Mr. Smith is here"));
        assert!(!en_abbrev_trap("The dog runs fast."));
    }

    #[test]
    fn en_mechanical_clean_dup_words() {
        assert!(!en_is_mechanically_clean("the the end"));
        assert!(en_is_mechanically_clean("the that later"));
        assert!(en_is_mechanically_clean("He said that that was true."));
        assert!(en_is_mechanically_clean("They had had enough."));
    }

    #[test]
    fn en_clean_sentence_gates() {
        let good = "The quick brown fox jumps over the lazy dog.";
        assert!(en_clean_sentence(good).is_some());
        assert!(en_clean_sentence("short").is_none());
        assert!(en_clean_sentence("lowercase start of this sentence here.").is_none());
        assert!(en_clean_sentence("The dog runs fast").is_none()); // thiếu dấu chấm
        assert!(en_clean_sentence("The dog runs fast now!").is_some());
        assert!(en_clean_sentence("She said \"hi\" to everyone there.").is_none()); // quote
        assert!(en_clean_sentence("The U.S. is a country with many states.").is_none());
    }

    #[test]
    fn civil_date_from_epoch() {
        // 2026-11-22 = 20 779 ngày từ 1970-01-01 (UTC).
        let (y, m, d) = civil_from_days(20_779);
        assert_eq!((y, m, d), (2026, 11, 22));
        let (y, m, d) = civil_from_days(0);
        assert_eq!((y, m, d), (1970, 1, 1));
        let (y, m, d) = civil_from_days(10_957);
        assert_eq!((y, m, d), (2000, 1, 1));
    }

    #[test]
    fn json_escape_basics() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        assert_eq!(json_escape("<pwd>"), "\\u003cpwd\\u003e");
    }

    #[test]
    fn comma_sorted_join() {
        let mut s: BTreeSet<String> = BTreeSet::new();
        s.insert("spelling".into());
        s.insert("diacritic_typo".into());
        assert_eq!(comma_join(&s), "diacritic_typo,spelling");
    }

    #[test]
    fn read_labelled_soft_parses() {
        let dir = std::env::temp_dir();
        let f = dir.join(format!("xtask-assist-t-{}.ass", std::process::id()));
        std::fs::write(
            &f,
            "# comment\ncau sai\tspelling\n~ cau mem\t\ncau sach\t\n",
        )
        .unwrap();
        let rows = read_labelled_soft(&f).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].kinds, vec!["spelling"]);
        assert!(!rows[0].soft);
        assert!(rows[1].soft);
        assert_eq!(rows[1].sentence, "cau mem");
        assert_eq!(rows[2].kinds, Vec::<String>::new());
        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn en_abbrev_trap_khong_loai_cau_chua_no_va_co_ranh_gioi_tu() {
        assert!(!en_abbrev_trap("I have no idea what happened."));
        assert!(!en_abbrev_trap("No, thank you very much!"));
        assert!(!en_abbrev_trap("She uses iOS. It works well."));
        assert!(!en_abbrev_trap("Version 1a.b. is not a word boundary."));
        assert!(en_abbrev_trap("Mr. Smith is here today."));
        assert!(en_abbrev_trap("He said 'Mr. Smith' arrived."));
        assert!(en_abbrev_trap("Item No. 5 is missing."));
        assert!(en_abbrev_trap("The U.S.A. is large."));
        assert!(en_abbrev_trap("Meet at 8 a.m. tomorrow."));
        assert!(en_abbrev_trap("Call the FBI. Now."));
    }

    #[test]
    fn rng_splitmix64_khong_trung_seed_chan_le_va_64bit() {
        assert_ne!(Rng::new(0).next_u32(), Rng::new(1).next_u32());
        assert_ne!(Rng::new(20261022).next_u32(), Rng::new(20261023).next_u32());
        assert_ne!(
            Rng::new(1).next_u32(),
            Rng::new(1 | (1u64 << 40)).next_u32()
        );
    }

    #[test]
    fn tone_inject_bo_qua_tu_nfd_va_with_tone_chan_out_of_range() {
        assert!(with_tone("ban", 6).is_none());
        let mut valid = BTreeSet::new();
        valid.insert(("thu".to_string(), 0));
        valid.insert(("thu".to_string(), 2));
        let mut rng = Rng::new(42);
        // Từ NFD "thu\u{0323}c" phải bị bỏ qua, không bị tách thành "thu" rồi gắn thêm dấu.
        assert!(tone_inject_one("thu\u{0323}c", &valid, &mut rng).is_none());
    }

    #[test]
    fn pool_from_local_ass_bo_dong_bat_dau_bang_tab() {
        let f = std::env::temp_dir().join(format!("xtask-pool-{}.ass", std::process::id()));
        std::fs::write(&f, "\tlabel_only\nCâu bình thường.\t\n~ Câu mềm.\t\n").unwrap();
        let pool = pool_from_local_ass(&f).unwrap();
        let _ = std::fs::remove_file(&f);
        assert_eq!(pool, vec!["Câu bình thường.", "Câu mềm."]);
    }
}
