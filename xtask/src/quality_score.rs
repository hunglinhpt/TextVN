// SPDX-License-Identifier: GPL-3.0-or-later
//! BIZ-1 — bảng điểm chất lượng chung (P0-8 §5/§8, `cargo xtask quality-score`).
//!
//! Gom KPI từ công cụ ĐÃ CÓ (G22 — không đo lại theo cách khác):
//! - Dream RSI KSR-oracle / precision / nhiễu: `textvn-bench dream-rsi` (RSI-1, corpus `corpus/eval/` RSI-2);
//! - trợ lý ngữ pháp recall / FPR: `xtask assist-score --json`;
//! - chống tái diễn: số mã lỗi + nợ `todo:REG-2` (`regression_index`, REG-1);
//! - rò rỉ tập đánh giá (`eval_leak`, RSI-2).
//!
//! `quality-score` so với tệp của bản TRƯỚC trong `perf/quality/` (KPI tất định so tuyệt đối): tiết
//! kiệm phím, phát hiện lỗi không được GIẢM; nhiễu, FPR, nợ không được TĂNG; bất biến/lệch đích = 0.
//! KPI Dream RSI chỉ so khi `eval_hash` (corpus đánh giá + từ bench) trùng — đổi corpus = baseline mới.
//! `--write` ghi `perf/quality/<version>.json` + nối `perf/quality/history.jsonl` (bước B4b phát hành).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR luôn nằm trong repo")
        .to_path_buf()
}

/// Số nguyên/thực của khoá phẳng `"key":<số>` đầu tiên trong `text`.
fn num(text: &str, key: &str) -> Option<f64> {
    let pat = format!("\"{key}\":");
    let p = text.find(&pat)? + pat.len();
    let s: String = text[p..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'e' | 'E' | '+'))
        .collect();
    s.parse().ok()
}

/// FNV-1a 64 — dấu vân tay corpus đánh giá (không cần crate băm).
fn fnv64(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.iter() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn workspace_version(root: &Path) -> Result<String, String> {
    let t = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| format!("Cargo.toml: {e}"))?;
    t.lines()
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("version = \"")
                .and_then(|r| r.strip_suffix('"'))
                .map(str::to_string)
        })
        .ok_or_else(|| "Cargo.toml: thiếu version".into())
}

fn eval_hash(root: &Path) -> Result<String, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(root.join("corpus/eval"))
        .map_err(|e| format!("corpus/eval: {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    files.push(root.join("tools/bench/src/main.rs")); // RSI_WORDS + chính sách oracle
    let blobs: Vec<Vec<u8>> = files
        .iter()
        .map(|p| fs::read(p).map(|b| b.into_iter().filter(|c| *c != b'\r').collect()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("eval_hash: {e}"))?;
    let refs: Vec<&[u8]> = blobs.iter().map(Vec::as_slice).collect();
    Ok(format!("{:016x}", fnv64(&refs)))
}

fn run_capture(root: &Path, argv: &[&str]) -> Result<String, String> {
    let out = Command::new(argv[0])
        .args(&argv[1..])
        .current_dir(root)
        .output()
        .map_err(|e| format!("{}: {e}", argv.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "{} thất bại ({}): {}",
            argv.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .last()
                .unwrap_or("")
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Bảng KPI phẳng: tên → giá trị.
pub type Kpis = BTreeMap<String, f64>;

/// Hướng tốt của một KPI: `+1` cao hơn là tốt, `-1` thấp hơn là tốt, `0` chỉ ghi (không gate).
pub fn direction(key: &str) -> i8 {
    match key {
        "rsi_saved_keys" | "rsi_vi_saved" | "rsi_en_saved" | "rsi_accepts" | "assist_detected"
        | "regression_codes" => 1,
        "rsi_noise"
        | "rsi_invariant_breaks"
        | "rsi_diverged"
        | "assist_fp_hard"
        | "regression_todo"
        | "eval_leak" => -1,
        _ => 0,
    }
}

/// KPI chỉ so được khi corpus của nó không đổi.
fn needs_same_eval(key: &str) -> bool {
    key.starts_with("rsi_")
}

/// So với bản trước: danh sách KPI tụt (rỗng = đạt).
pub fn regressions(prev: &Kpis, prev_eval: &str, cur: &Kpis, cur_eval: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (k, v) in cur {
        let Some(old) = prev.get(k) else { continue };
        if needs_same_eval(k) && prev_eval != cur_eval {
            continue;
        }
        let worse = match direction(k) {
            1 => v < old,
            -1 => v > old,
            _ => false,
        };
        if worse {
            out.push(format!("{k}: {old} → {v}"));
        }
    }
    // Bất biến tuyệt đối (không phụ thuộc bản trước).
    for k in [
        "rsi_invariant_breaks",
        "rsi_diverged",
        "eval_leak",
        "regression_todo",
    ] {
        if cur.get(k).copied().unwrap_or(0.0) > 0.0 {
            out.push(format!("{k} = {} (phải = 0)", cur[k]));
        }
    }
    out
}

fn collect(root: &Path) -> Result<Kpis, String> {
    let mut k = Kpis::new();
    let rsi = run_capture(
        root,
        &[
            "cargo",
            "run",
            "-q",
            "--release",
            "-p",
            "textvn-bench",
            "--",
            "dream-rsi",
        ],
    )?;
    let json = rsi.lines().next().unwrap_or("");
    for (dst, src) in [
        ("rsi_baseline_keys", "baseline_keys"),
        ("rsi_saved_keys", "saved_keys"),
        ("rsi_vi_baseline", "vi_baseline"),
        ("rsi_vi_saved", "vi_saved"),
        ("rsi_en_baseline", "en_baseline"),
        ("rsi_en_saved", "en_saved"),
        ("rsi_ghosts_shown", "ghosts_shown"),
        ("rsi_ghosts_good", "ghosts_good"),
        ("rsi_accepts", "accepts"),
        ("rsi_invariant_breaks", "invariant_breaks"),
        ("rsi_diverged", "diverged"),
        ("rsi_max_chain", "max_chain"),
    ] {
        k.insert(
            dst.into(),
            num(json, src).ok_or_else(|| format!("dream-rsi thiếu `{src}`"))?,
        );
    }
    k.insert(
        "rsi_noise".into(),
        k["rsi_ghosts_shown"] - k["rsi_ghosts_good"],
    );
    let assist = run_capture(
        root,
        &[
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "assist-score",
            "--json",
            "--mode",
            "check",
        ],
    )?;
    let a = &assist[assist.find('[').unwrap_or(0)..];
    for (dst, src) in [
        ("assist_errors", "errors"),
        ("assist_detected", "detected_errors"),
        ("assist_clean", "clean"),
        ("assist_fp_hard", "flagged_clean_hard"),
    ] {
        k.insert(
            dst.into(),
            num(a, src).ok_or_else(|| format!("assist-score thiếu `{src}`"))?,
        );
    }
    let codes = crate::regression_index::collect_codes(root);
    let idx = fs::read_to_string(root.join("docs/specs/regression-index.md"))
        .map_err(|e| format!("regression-index: {e}"))?;
    let todo = crate::regression_index::parse_index(&idx)?
        .rows
        .values()
        .filter(|(_, r)| r.contains(&crate::regression_index::Ref::Todo))
        .count();
    k.insert("regression_codes".into(), codes.len() as f64);
    k.insert("regression_todo".into(), todo as f64);
    k.insert(
        "eval_leak".into(),
        crate::eval_leak::scan(root)?.leaks.len() as f64,
    );
    Ok(k)
}

fn render(version: &str, eval: &str, k: &Kpis) -> String {
    let body: Vec<String> = k.iter().map(|(n, v)| format!("\"{n}\":{v}")).collect();
    format!(
        "{{\"version\":\"{version}\",\"eval_hash\":\"{eval}\",{}}}",
        body.join(",")
    )
}

fn parse_file(text: &str) -> (String, Kpis) {
    let eval = text
        .find("\"eval_hash\":\"")
        .map(|p| text[p + 13..].chars().take_while(|c| *c != '"').collect())
        .unwrap_or_default();
    let mut k = Kpis::new();
    for part in text
        .trim()
        .trim_matches(|c| c == '{' || c == '}')
        .split(',')
    {
        if let Some((n, v)) = part.split_once(':') {
            let n = n.trim().trim_matches('"');
            if let Ok(v) = v.trim().parse::<f64>() {
                k.insert(n.to_string(), v);
            }
        }
    }
    (eval, k)
}

/// So khoá phiên bản `a.b.c` (số).
fn ver_key(v: &str) -> Vec<u64> {
    v.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

/// Tệp của bản gần nhất TRƯỚC `version` (không tính chính nó).
fn previous_file(dir: &Path, version: &str) -> Option<(String, PathBuf)> {
    let cur = ver_key(version);
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let v = name.strip_suffix(".json")?.to_string();
            (v.chars().all(|c| c.is_ascii_digit() || c == '.') && ver_key(&v) < cur)
                .then(|| (v, e.path()))
        })
        .max_by_key(|(v, _)| ver_key(v))
}

pub fn cli(args: &[String]) -> Result<(), String> {
    let root = repo_root();
    let write = args.iter().any(|a| a == "--write");
    let version = workspace_version(&root)?;
    let eval = eval_hash(&root)?;
    let kpis = collect(&root)?;
    let pct = |a: f64, b: f64| if b > 0.0 { 100.0 * a / b } else { 0.0 };
    println!(
        "quality-score {version}: KSR {:.1}% (VI {:.1}% · EN {:.1}%) · precision {:.1}% · nhiễu {} · trợ lý recall {:.1}% FP {} · mã lỗi {} (nợ {}) · rò rỉ eval {}",
        pct(kpis["rsi_saved_keys"], kpis["rsi_baseline_keys"]),
        pct(kpis["rsi_vi_saved"], kpis["rsi_vi_baseline"]),
        pct(kpis["rsi_en_saved"], kpis["rsi_en_baseline"]),
        pct(kpis["rsi_ghosts_good"], kpis["rsi_ghosts_shown"]),
        kpis["rsi_noise"],
        pct(kpis["assist_detected"], kpis["assist_errors"]),
        kpis["assist_fp_hard"],
        kpis["regression_codes"],
        kpis["regression_todo"],
        kpis["eval_leak"],
    );
    let dir = root.join("perf/quality");
    let mut bad = Vec::new();
    match previous_file(&dir, &version) {
        Some((pv, p)) => {
            let (pe, pk) =
                parse_file(&fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?);
            if pe != eval {
                println!("  so với {pv}: corpus đánh giá đổi (eval_hash {pe} → {eval}) — KPI Dream RSI là baseline mới, KPI khác vẫn so");
            } else {
                println!("  so với {pv}: cùng corpus đánh giá");
            }
            bad = regressions(&pk, &pe, &kpis, &eval);
        }
        None => {
            println!("  chưa có bản trước trong perf/quality/ — `--write` ở B4b để ghi baseline");
            bad.extend(regressions(&Kpis::new(), "", &kpis, &eval));
        }
    }
    if write {
        fs::create_dir_all(&dir).map_err(|e| format!("perf/quality: {e}"))?;
        let line = render(&version, &eval, &kpis);
        fs::write(dir.join(format!("{version}.json")), format!("{line}\n"))
            .map_err(|e| format!("perf/quality/{version}.json: {e}"))?;
        let hist = dir.join("history.jsonl");
        let mut h = fs::read_to_string(&hist).unwrap_or_default();
        if !h
            .lines()
            .any(|l| l.contains(&format!("\"version\":\"{version}\"")))
        {
            h.push_str(&line);
            h.push('\n');
            fs::write(&hist, h).map_err(|e| format!("history.jsonl: {e}"))?;
        }
        println!("  đã ghi perf/quality/{version}.json + history.jsonl");
    }
    if bad.is_empty() {
        Ok(())
    } else {
        for b in &bad {
            eprintln!("  TỤT: {b}");
        }
        Err(format!(
            "quality-score: {} KPI tụt — không phát hành (G19/G23)",
            bad.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(pairs: &[(&str, f64)]) -> Kpis {
        pairs.iter().map(|(n, v)| (n.to_string(), *v)).collect()
    }

    #[test]
    fn biz_1_kpi_tut_thi_do_theo_huong_tot() {
        let prev = k(&[
            ("rsi_saved_keys", 600.0),
            ("rsi_noise", 100.0),
            ("assist_detected", 41.0),
        ]);
        // Tiết kiệm giảm + nhiễu tăng + phát hiện giảm ⇒ 3 KPI tụt.
        let cur = k(&[
            ("rsi_saved_keys", 599.0),
            ("rsi_noise", 101.0),
            ("assist_detected", 40.0),
        ]);
        assert_eq!(regressions(&prev, "h", &cur, "h").len(), 3);
        // Cải thiện ⇒ đạt.
        let better = k(&[
            ("rsi_saved_keys", 700.0),
            ("rsi_noise", 90.0),
            ("assist_detected", 42.0),
        ]);
        assert!(regressions(&prev, "h", &better, "h").is_empty());
        // Corpus đánh giá đổi ⇒ KPI Dream RSI không so; KPI trợ lý vẫn so.
        let r = regressions(&prev, "h1", &cur, "h2");
        assert_eq!(r.len(), 1);
        assert!(r[0].starts_with("assist_detected"));
        // Bất biến tuyệt đối kể cả khi không có bản trước.
        assert_eq!(
            regressions(&Kpis::new(), "", &k(&[("rsi_diverged", 1.0)]), "").len(),
            1
        );
    }

    #[test]
    fn biz_1_ghi_va_doc_lai_tep_kpi() {
        let cur = k(&[("rsi_saved_keys", 685.0), ("assist_fp_hard", 0.0)]);
        let (eval, back) = parse_file(&render("0.2.97", "abc123", &cur));
        assert_eq!(eval, "abc123");
        assert_eq!(back, cur);
        assert!(ver_key("0.2.10") > ver_key("0.2.9"));
        assert_eq!(
            num("{\"a\":1,\"saved_keys\":685}", "saved_keys"),
            Some(685.0)
        );
        assert_ne!(
            fnv64(&[b"a", b"b"]),
            fnv64(&[b"ab"]),
            "ranh giới tệp tính vào băm"
        );
    }
}
