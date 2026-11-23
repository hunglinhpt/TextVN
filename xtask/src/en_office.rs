// SPDX-License-Identifier: GPL-3.0-or-later
//! E59: port `scripts/gen_en_office_phrases.py` — sinh `data/en_office_phrases.tsv`
//! (P6, vòng 59): bảng ngữ cảnh câu tiếng Anh văn phòng + bilingual Việt-Anh.
//!
//! Dữ liệu nguồn: module generated [`crate::gen_en_office_data`] (dump 1 lần từ
//! bản Python lúc port — từ giờ sửa trực tiếp const đó). Thuật toán giữ nguyên
//! semantics python: chia câu ranh giới MỌI từ, weight = BASE[cat] + 2·min(i,6)
//! (cap 100), ghép trùng giữ weight cao nhất, sort lex theo (key, pred), LF.
//!
//! `run(true)` = ghi lại `data/en_office_phrases.tsv`; `run(false)` = so với
//! file đang có (gate `check-en-office` như `check-tables` — CI fail nếu lệch).

use std::collections::BTreeMap;

use crate::gen_en_office_data::{BILINGUAL, SENTENCES, STANDALONE};

/// Invariant `du_lieu_phrases_invariant` / MAX_TEXT=64 (bản python MAX_PRED).
const MAX_PRED: usize = 60;

fn base_w(cat: &str) -> u32 {
    match cat {
        "upward" => 82,
        "downward" => 82,
        "tech" => 84,
        "business" => 84,
        "everyday" => 76,
        "bilingual" => 90,
        other => unreachable!("category lạ `{other}` — xem gen_en_office_data.rs"),
    }
}

/// Chia ranh giới từ thứ i của câu: key = w[0..i], pred = w[i..].
fn sentence_rows(cat: &str, sentence: &str) -> Vec<(String, String, u32)> {
    let words: Vec<&str> = sentence.split_whitespace().collect();
    let mut out = Vec::new();
    for i in 1..words.len() {
        let key = words[..i].join(" ");
        let pred = words[i..].join(" ");
        if pred.chars().count() > MAX_PRED || key.chars().count() > MAX_PRED {
            continue;
        }
        let weight = (base_w(cat) + 2 * i.min(6) as u32).min(100);
        out.push((key, pred, weight));
    }
    out
}

/// Python `key.islower()` sau `.lower()`: hợp lệ khi có ÍT NHẤT 1 chữ thường
/// và 0 chữ hoa (từ-khóa/cụm toàn số hoặc ký hiệu bị loại).
fn is_lowercased_word(s: &str) -> bool {
    s.chars().any(|c| c.is_lowercase()) && !s.chars().any(|c| c.is_uppercase())
}

fn add(rows: &mut BTreeMap<(String, String), u32>, key: &str, pred: &str, weight: u32) {
    let key = key
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let pred = pred
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if key.is_empty() || pred.is_empty() || weight == 0 || weight > 100 {
        return;
    }
    if key.chars().count() > MAX_PRED || pred.chars().count() > MAX_PRED {
        return;
    }
    if !is_lowercased_word(&key) || !is_lowercased_word(&pred) {
        return;
    }
    let entry = rows.entry((key, pred)).or_insert(weight);
    if weight > *entry {
        *entry = weight;
    }
}

pub fn build_rows() -> BTreeMap<(String, String), u32> {
    let mut rows: BTreeMap<(String, String), u32> = BTreeMap::new();
    for (cat, sentence) in SENTENCES {
        for (k, p, w) in sentence_rows(cat, sentence) {
            add(&mut rows, &k, &p, w);
        }
    }
    for (k, p, w) in STANDALONE {
        add(&mut rows, k, p, *w);
    }
    for (k, p, w) in BILINGUAL {
        add(&mut rows, k, p, *w);
    }
    rows
}

fn render(rows: &BTreeMap<(String, String), u32>) -> String {
    let mut out = String::with_capacity(64 + rows.len() * 64);
    out.push_str("# en_office_phrases.tsv — ngữ cảnh cụm từ/câu tiếng Anh văn phòng (P6,\n");
    out.push_str("# vòng 59): upward reporting / downward management / tech / business /\n");
    out.push_str("# everyday glue / bilingual Viet-Eng code-switch. Sinh bởi\n");
    out.push_str("# `cargo xtask gen-en-office` (E59 — thay scripts/gen_en_office_phrases.py) — KHÔNG sửa tay.\n");
    out.push_str("# context_lower\tpredicted_phrase\tweight\n");
    for ((key, pred), weight) in rows {
        out.push_str(key);
        out.push('\t');
        out.push_str(pred);
        out.push('\t');
        out.push_str(&weight.to_string());
        out.push('\n');
    }
    out
}

pub fn run(write: bool) -> Result<(), String> {
    let rows = build_rows();
    if rows.len() < 10_000 {
        // Bảo vệ dữ liệu như bản python: bảng bị thu nhỏ bất thường = fail.
        return Err(format!(
            "FATAL: chỉ sinh được {} hàng < 10.000 — mở rộng gen_en_office_data.rs",
            rows.len()
        ));
    }
    let rel = "data/en_office_phrases.tsv";
    let path = crate::assist_tools::repo_root().join(rel);
    let content = render(&rows);
    // stdout UTF-8 portable; đổi header dòng 4 so bản python — body giữ nguyên.
    let mut cats: BTreeMap<&str, usize> = BTreeMap::new();
    for (key, _) in rows.keys() {
        let first = key.split_whitespace().next().unwrap_or("");
        *cats.entry(first).or_insert(0) += 1;
    }
    let mut top: Vec<(&str, usize)> = cats.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    top.truncate(8);
    let top_s: Vec<String> = top.iter().map(|(k, n)| format!("{k}:{n}")).collect();
    if write {
        std::fs::write(&path, &content).map_err(|e| format!("ghi `{rel}`: {e}"))?;
        println!(
            "  gen  {rel} ({} hàng, {} câu + {} standalone + {} bilingual)",
            rows.len(),
            SENTENCES.len(),
            STANDALONE.len(),
            BILINGUAL.len()
        );
        println!("  top opener keys: {}", top_s.join(", "));
        Ok(())
    } else {
        let cur = std::fs::read_to_string(&path)
            .map(|s| s.replace("\r\n", "\n"))
            .map_err(|e| {
                format!("không đọc được `{rel}` để so: {e} (chạy `cargo xtask gen-en-office`)")
            })?;
        if cur == content {
            println!("  ok   {rel} ({} hàng)", rows.len());
            Ok(())
        } else {
            Err(format!(
                "`{rel}` lệch với dữ liệu gen_en_office_data.rs — chạy `cargo xtask gen-en-office`"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Body phải KHỚP file TSV đã commit (bỏ 5 dòng header comment) — chống hồi
    /// quy thuật toán chia ranh giới + trùng key + weight của bản port E59.
    #[test]
    fn body_matches_committed_tsv() {
        let rel = crate::assist_tools::repo_root().join("data/en_office_phrases.tsv");
        let disk = std::fs::read_to_string(&rel).expect("thiếu data/en_office_phrases.tsv");
        let disk_body: Vec<&str> = disk
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter(|l| !l.trim().is_empty())
            .collect();
        let rows = build_rows();
        let mut expect: Vec<String> = Vec::with_capacity(rows.len());
        for ((k, p), w) in &rows {
            expect.push(format!("{k}\t{p}\t{w}"));
        }
        assert_eq!(disk_body.len(), expect.len(), "số hàng lệch");
        for (i, (d, e)) in disk_body.iter().zip(&expect).enumerate() {
            assert_eq!(d, e, "hàng {i} lệch:\n  disk: {d}\n  gen : {e}");
        }
    }

    #[test]
    fn rows_count_guard() {
        let rows = build_rows();
        assert!(rows.len() >= 10_000, "{} < 10.000", rows.len());
    }

    #[test]
    fn sentence_rows_weight_cap() {
        let long = "one two three four five six seven eight nine ten eleven twelve";
        let rs = sentence_rows("tech", long);
        let max_w = rs.iter().map(|(_, _, w)| *w).max().unwrap_or(0);
        assert_eq!(max_w, 96); // 84 + 2*6 = 96 < 100 cap không chạm
        let bilingual = sentence_rows("bilingual", "one two three four five six seven eight");
        let cap = bilingual.iter().map(|(_, _, w)| *w).max().unwrap_or(0);
        assert_eq!(cap, 100); // 90 + 2*6 = 100 -> cap
    }

    #[test]
    fn max_pred_filter() {
        let words: Vec<String> = (0..40).map(|i| format!("w{i}")).collect();
        let long_words: Vec<String> = (0..12)
            .map(|i| format!("waaaaaaaaaaaaaaaaaaaaaaaaaaa{i}"))
            .collect();
        let sent = format!("{} {}", words.join(" "), long_words.join(" "));
        let rs = sentence_rows("tech", &sent);
        for (k, p, _) in &rs {
            assert!(k.chars().count() <= MAX_PRED && p.chars().count() <= MAX_PRED);
        }
    }

    #[test]
    fn add_filters_non_word_keys() {
        let mut rows = BTreeMap::new();
        add(&mut rows, "2024", "hello world", 50); // key không chữ thường -> bỏ
        add(&mut rows, "hello world", "2024", 50); // pred không chữ -> bỏ
        add(&mut rows, "hello", "world", 50);
        add(&mut rows, "hello", "world", 70); // giữ weight cao nhất
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[&("hello".into(), "world".into())], 70);
    }
}
