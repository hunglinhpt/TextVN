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

/// Kết quả quét rò rỉ của cả cây repo — một nguồn cho CLI lẫn KPI `eval_leak` của
/// `quality-score` (BIZ-1, G22: không đo lại theo cách khác).
pub struct LeakScan {
    /// Số câu đánh giá đã quét (mọi tệp `corpus/eval/*.txt`).
    pub total_sentences: usize,
    /// Số tệp dữ liệu dự đoán đã đối chiếu.
    pub data_files: usize,
    /// Mỗi câu rò rỉ: `(tệp đánh giá, câu, chuỗi trùng, tệp dữ liệu)`.
    pub leaks: Vec<(String, String, String, String)>,
}

/// Nguồn KHÔNG được chung chuỗi ≥ [`LEAK_N`] token với tập đánh giá: dữ liệu dự đoán
/// `data/*phrases*.tsv` và tập dev `corpus/dev/*.txt` (BIZ-2 — tập dev được soi chi tiết bằng
/// miss report và làm đầu vào dữ liệu bản sau, nên dev ↔ eval trùng cũng là rò rỉ gián tiếp).
pub fn leak_sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for (dir, pred) in [
        (
            "data",
            (|n: &str| n.contains("phrases") && n.ends_with(".tsv")) as fn(&str) -> bool,
        ),
        ("corpus/dev", |n: &str| n.ends_with(".txt")),
    ] {
        let Ok(rd) = fs::read_dir(root.join(dir)) else {
            if dir == "data" {
                return Err(format!("{dir}: không đọc được"));
            }
            continue;
        };
        let mut v = Vec::new();
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            if pred(&name) {
                let text =
                    fs::read_to_string(ent.path()).map_err(|e| format!("{dir}/{name}: {e}"))?;
                v.push((format!("{dir}/{name}"), text));
            }
        }
        v.sort();
        out.extend(v);
    }
    Ok(out)
}

/// Quét toàn bộ repo: mọi câu trong `corpus/eval/*.txt` đối chiếu n-gram của
/// [`leak_sources`] (`data/*phrases*.tsv` + `corpus/dev/*.txt`). Lỗi nếu thư mục đánh giá
/// rỗng (tập held-out bị xoá).
pub fn scan(root: &Path) -> Result<LeakScan, String> {
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
    let data = leak_sources(root)?;
    let evals = read_dir("corpus/eval", &|n| n.ends_with(".txt"))?;
    if evals.is_empty() {
        return Err("corpus/eval/*.txt không có — tập đánh giá RSI-2 bị xoá?".into());
    }
    let grams = data_ngrams(&data, LEAK_N);
    let mut out = LeakScan {
        total_sentences: 0,
        data_files: data.len(),
        leaks: Vec::new(),
    };
    for (name, text) in &evals {
        out.total_sentences += content_lines(text).count();
        for (s, g, src) in leaks(text, &grams, LEAK_N) {
            out.leaks.push((name.clone(), s.to_string(), g, src));
        }
    }
    Ok(out)
}

pub fn cli() -> Result<(), String> {
    let scan = scan(&repo_root())?;
    for (name, s, g, src) in &scan.leaks {
        eprintln!("  {name}: \"{s}\" — trùng \"{g}\" ({src})");
    }
    println!(
        "eval-leak: {} câu đánh giá · {} tệp dữ liệu · rò rỉ ≥ {LEAK_N} token: {}",
        scan.total_sentences,
        scan.data_files,
        scan.leaks.len()
    );
    if scan.leaks.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "eval-leak: {} câu đánh giá trùng dữ liệu dự đoán — viết lại câu trong corpus/eval/ hoặc loại câu đó khỏi nguồn sinh data",
            scan.leaks.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BIZ-2 (review claude-opus-7e41b2d9): tập dev `corpus/dev/` phải nằm trong nguồn của gate
    /// eval-leak — dev trùng ≥ LEAK_N token với eval là rò rỉ gián tiếp; một câu dev chép từ eval
    /// phải bị báo.
    #[test]
    fn biz2_eval_leak_quet_ca_corpus_dev() {
        let src = leak_sources(&repo_root()).expect("đọc nguồn eval-leak");
        assert!(
            src.iter().any(|(n, _)| n.starts_with("corpus/dev/")),
            "eval-leak phải quét corpus/dev/: {:?}",
            src.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
        assert!(src.iter().any(|(n, _)| n.starts_with("data/")));
        // Đột biến: một câu dev chép nguyên từ eval ⇒ phải bị bắt.
        let eval_line = "em gửi anh bản báo cáo tuần này để anh xem trước buổi họp";
        let dev = vec![(
            "corpus/dev/x.txt".to_string(),
            format!("# c\n{eval_line}\n"),
        )];
        let grams = data_ngrams(&dev, LEAK_N);
        assert!(!leaks(eval_line, &grams, LEAK_N).is_empty());
    }

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

    /// BIZ-1: `scan()` — nguồn KPI `eval_leak` của quality-score — phải đếm đúng trên
    /// cây mẫu và fail-closed khi tập đánh giá bị xoá (E88 spirit: thiếu dữ liệu ⇒ đỏ).
    #[test]
    fn biz_1_scan_quet_cay_dem_dung_ro_ri_va_loi_khi_mat_tap_danh_gia() {
        let base = std::env::temp_dir().join(format!("textvn-eval-scan-{}", std::process::id()));
        let d = base.join("data");
        let e = base.join("corpus").join("eval");
        fs::create_dir_all(&d).unwrap();
        fs::create_dir_all(&e).unwrap();
        fs::write(
            d.join("x_phrases.tsv"),
            "the client\tasked for a revised contract\t9\n",
        )
        .unwrap();
        fs::write(e.join("a.txt"), "the client asked for a revised quote\n").unwrap();
        let found = scan(&base).expect("quét cây mẫu");
        assert_eq!(found.total_sentences, 1);
        assert_eq!(found.data_files, 1);
        assert_eq!(found.leaks.len(), 1);
        assert_eq!(found.leaks[0].0, "corpus/eval/a.txt");
        assert_eq!(found.leaks[0].2, "the client asked for a revised");
        assert_eq!(found.leaks[0].3, "data/x_phrases.tsv");

        // Tập đánh giá bị xoá ⇒ lỗi (không bao giờ báo "0 rò rỉ" khi không quét được gì).
        fs::remove_file(e.join("a.txt")).unwrap();
        assert!(scan(&base).is_err());
        fs::remove_dir_all(&base).unwrap();
    }
}
