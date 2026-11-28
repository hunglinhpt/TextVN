// SPDX-License-Identifier: GPL-3.0-or-later
//! RSI-2 — gate chống rò rỉ tập đánh giá (`cargo xtask eval-leak`, preflight + repo-hygiene).
//!
//! `corpus/eval/*.txt` là tập HELD-OUT của bench Dream RSI KSR-oracle (`tools/bench`). Nếu dữ liệu dự
//! đoán (`data/*phrases*.tsv`, do `cargo xtask gen-*-phrases` sinh) chứa một đoạn dài của câu đánh giá,
//! bộ dự đoán "thuộc" câu đó và KSR bị thổi phồng — đúng lỗi số 22% cũ (spike SPK-RSI). Luật: không câu
//! đánh giá nào chung một chuỗi ≥ [`LEAK_N`] token liên tiếp (âm tiết Việt / từ Anh, chữ thường) với
//! cặp `ngữ cảnh + dự đoán` của bất kỳ dòng dữ liệu nào. ≤ 5 token là cụm từ thường gặp của ngôn ngữ
//! ("the end of the month", "bộ phận chăm sóc"); ≥ 6 là dữ liệu đã chứa đúng câu đó.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Số token liên tiếp tối thiểu coi là rò rỉ.
pub const LEAK_N: usize = 6;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR luôn nằm trong repo")
        .to_path_buf()
}

fn content_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// Mọi chuỗi `n` token của dữ liệu dự đoán → tên tệp nguồn (để báo lỗi).
pub fn data_ngrams(tsv_files: &[(String, String)], n: usize) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (name, text) in tsv_files {
        for line in content_lines(text) {
            let mut cols = line.split('\t');
            let joined = format!(
                "{} {}",
                cols.next().unwrap_or(""),
                cols.next().unwrap_or("")
            )
            .to_lowercase();
            let w: Vec<&str> = joined.split_whitespace().collect();
            for g in w.windows(n) {
                out.entry(g.join(" ")).or_insert_with(|| name.clone());
            }
        }
    }
    out
}

/// Câu đánh giá rò rỉ: `(câu, chuỗi trùng, tệp dữ liệu)`.
pub fn leaks<'a>(
    eval_text: &'a str,
    grams: &HashMap<String, String>,
    n: usize,
) -> Vec<(&'a str, String, String)> {
    let mut out = Vec::new();
    for s in content_lines(eval_text) {
        let lower = s.to_lowercase();
        let w: Vec<&str> = lower.split_whitespace().collect();
        if let Some((g, src)) = w
            .windows(n)
            .map(|g| g.join(" "))
            .find_map(|g| grams.get(&g).map(|src| (g, src.clone())))
        {
            out.push((s, g, src));
        }
    }
    out
}

pub fn cli() -> Result<(), String> {
    let root = repo_root();
    let read_dir =
        |dir: &str, pred: &dyn Fn(&str) -> bool| -> Result<Vec<(String, String)>, String> {
            let mut v = Vec::new();
            let rd = fs::read_dir(root.join(dir)).map_err(|e| format!("{dir}: {e}"))?;
            for ent in rd.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                if pred(&name) {
                    let text =
                        fs::read_to_string(ent.path()).map_err(|e| format!("{dir}/{name}: {e}"))?;
                    v.push((format!("{dir}/{name}"), text));
                }
            }
            v.sort();
            Ok(v)
        };
    let data = read_dir("data", &|n| n.contains("phrases") && n.ends_with(".tsv"))?;
    let evals = read_dir("corpus/eval", &|n| n.ends_with(".txt"))?;
    if evals.is_empty() {
        return Err("corpus/eval/*.txt không có — tập đánh giá RSI-2 bị xoá?".into());
    }
    let grams = data_ngrams(&data, LEAK_N);
    let mut bad = 0;
    let mut total = 0;
    for (name, text) in &evals {
        total += content_lines(text).count();
        for (s, g, src) in leaks(text, &grams, LEAK_N) {
            bad += 1;
            eprintln!("  {name}: \"{s}\" — trùng \"{g}\" ({src})");
        }
    }
    println!(
        "eval-leak: {total} câu đánh giá ({} tệp) · {} tệp dữ liệu · rò rỉ ≥ {LEAK_N} token: {bad}",
        evals.len(),
        data.len()
    );
    if bad == 0 {
        Ok(())
    } else {
        Err(format!(
            "eval-leak: {bad} câu đánh giá trùng dữ liệu dự đoán — viết lại câu trong corpus/eval/ hoặc loại câu đó khỏi nguồn sinh data"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsi_2_bat_ro_ri_dai_bo_qua_cum_ngan() {
        let data = vec![(
            "data/x_phrases.tsv".to_string(),
            "the client\tasked for a revised contract\t9\nend of\tthe month\t5\n".to_string(),
        )];
        let g = data_ngrams(&data, LEAK_N);
        // 6 từ liên tiếp trùng ⇒ rò rỉ.
        let hit = leaks("the client asked for a revised quote\n", &g, LEAK_N);
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].1, "the client asked for a revised");
        // Cụm thường gặp ≤ 5 từ không tính.
        assert!(leaks("we meet at the end of the month\n", &g, LEAK_N).is_empty());
        // Chú thích/dòng trống bỏ qua; không phân biệt hoa thường.
        assert!(leaks("# the client asked for a revised contract\n\n", &g, LEAK_N).is_empty());
        assert_eq!(
            leaks("The Client Asked For A Revised deal\n", &g, LEAK_N).len(),
            1
        );
    }
}
