// SPDX-License-Identifier: GPL-3.0-or-later
//! BIZ-3 — bảng xu hướng KPI chất lượng theo bản (`cargo xtask quality-trend`).
//!
//! Đọc `perf/quality/*.json` (CHỈ ĐỌC — lệnh không ghi file nào), in ra markdown
//! song ngữ VI + EN 1:1 (G18): hai bảng `## VI` / `## EN` cùng số hàng, cùng ô giá
//! trị, chỉ khác nhãn — để release manager (giữ rm-lock, R3) dán vào README (mục
//! "Chất lượng theo bản") + CHANGELOG entry bản mới ở bước B4d phát hành.
//!
//! Một nguồn sự thật (G22 — cấm bản sao logic):
//! - giá trị ô: `quality_score::derived_display` — cùng công thức với chuỗi in
//!   `quality-score`;
//! - mũi tên: `quality_score::direction` của khoá underlying từng hàng;
//! - cảnh báo `⚠` lẫn gate Err: `quality_score::regressions` của bản MỚI NHẤT vs bản
//!   liền trước (cùng ngữ nghĩa quality-score — KHÔNG viết gate thứ hai).
//!
//! Fail-closed như BIZ-1: chưa có tệp `perf/quality/<ver>.json` ⇒ Err (gate không được
//! vờn trang trắng). Không I/O ngoài read_dir/read_to_string của `perf/quality`; lệnh
//! không có dữ liệu người dùng nên không dính G8.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::quality_score::{
    derived_display, direction, needs_same_eval, parse_file, regressions, repo_root, ver_key, Kpis,
};

/// Thông báo fail-closed khi chưa có baseline nào (B4c của BIZ-1 chưa từng chạy).
const CHUA_CO_BASELINE: &str =
    "chưa có perf/quality/<ver>.json — chạy `cargo xtask quality-score --write` (B4c) trước khi phát hành";

/// Một cột bảng xu hướng: version (tên tệp, không đuôi `.json`), eval_hash và KPI đã parse.
#[derive(Debug)]
struct Col {
    ver: String,
    eval: String,
    k: Kpis,
}

/// Đọc `perf/quality/*.json` (chỉ nhận tệp tên bắt đầu bằng số, toàn số + chấm), sắp
/// tăng dần `ver_key` — bản mới nhất nằm cuối (cột ngoài cùng bên phải). Không có tệp
/// hợp lệ ⇒ Err (fail-closed).
fn load_columns(root: &Path) -> Result<Vec<Col>, String> {
    let dir = root.join("perf/quality");
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("{CHUA_CO_BASELINE} (perf/quality: {e})"))?;
    let mut files: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let v = name.strip_suffix(".json")?.to_string();
            let la_version = v.starts_with(|c: char| c.is_ascii_digit())
                && v.chars().all(|c| c.is_ascii_digit() || c == '.');
            la_version.then(|| (v, e.path()))
        })
        .collect();
    files.sort_by_key(|(v, _)| ver_key(v));
    if files.is_empty() {
        return Err(CHUA_CO_BASELINE.to_string());
    }
    files
        .iter()
        .map(|(v, p)| {
            let text = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
            let (eval, k) = parse_file(&text);
            // Audit view-2 2026-11-28: JSON hỏng/ghi dở dang (crash giữa `fs::write`
            // của B4c) từng lọt qua `parse_file` khoan dung → cột toàn `—` mà vẫn Ok
            // (fail-open). Tệp không parse ra KPI nào ⇒ Err — cùng luật "gate không
            // được vờn trang trắng" với nhánh thư mục rỗng.
            if k.is_empty() {
                return Err(format!(
                    "{}: không parse ra KPI nào — tệp JSON hỏng/ghi dở dang? Chạy lại \
                     `cargo xtask quality-score --write` (B4c).",
                    p.display()
                ));
            }
            Ok(Col {
                ver: v.clone(),
                eval,
                k,
            })
        })
        .collect()
}

/// Một hàng KPI của bảng xu hướng (nhãn VI + EN riêng — G18 1:1, ô giá trị dùng chung).
struct Row {
    /// Nhãn hàng tiếng Việt — ghi rõ underlying key để truy vết về `quality-score`.
    vi: &'static str,
    /// Nhãn hàng tiếng Anh.
    en: &'static str,
    /// Khoá underlying cho `direction()` — mũi tên lấy từ đây (G22, một nguồn).
    dir_key: &'static str,
    /// Khoá thô phải ĐỦ trong bản thì hàng mới có giá trị (thiếu ⇒ ô `—`).
    keys: &'static [&'static str],
    /// Tên phần tử trong `derived_display` (cùng công thức `quality-score`); `None` = đọc khoá thẳng từ Kpis.
    derived: Option<&'static str>,
    /// `true` = hàng % (ngang khi chênh < 0,5% tương đối); `false` = đếm (ngang khi bằng giá trị).
    pct: bool,
    /// `true` = kèm số quy đổi trên 100 phím cơ sở (hàng nhiễu chữ mờ).
    per100: bool,
}

/// 11 hàng hiển thị của BIZ-3 (P0-8 §10) — giá trị/mũi tên/gate đều tái dùng
/// `quality_score` (G22). Hàng nhiễu lấy hướng từ `direction("rsi_noise")`.
const ROWS: [Row; 11] = [
    Row {
        vi: "KSR tổng — % phím tiết kiệm / phím cơ sở (`rsi_saved_keys` / `rsi_baseline_keys`)",
        en: "Overall KSR — % keystrokes saved / baseline (`rsi_saved_keys` / `rsi_baseline_keys`)",
        dir_key: "rsi_saved_keys",
        keys: &["rsi_saved_keys", "rsi_baseline_keys"],
        derived: Some("KSR"),
        pct: true,
        per100: false,
    },
    Row {
        vi: "KSR tiếng Việt — % (`rsi_vi_saved` / `rsi_vi_baseline`)",
        en: "Vietnamese KSR — % (`rsi_vi_saved` / `rsi_vi_baseline`)",
        dir_key: "rsi_vi_saved",
        keys: &["rsi_vi_saved", "rsi_vi_baseline"],
        derived: Some("KSR_VI"),
        pct: true,
        per100: false,
    },
    Row {
        vi: "KSR tiếng Anh — % (`rsi_en_saved` / `rsi_en_baseline`)",
        en: "English KSR — % (`rsi_en_saved` / `rsi_en_baseline`)",
        dir_key: "rsi_en_saved",
        keys: &["rsi_en_saved", "rsi_en_baseline"],
        derived: Some("KSR_EN"),
        pct: true,
        per100: false,
    },
    Row {
        vi: "Precision chữ mờ — % gợi ý đúng (`rsi_ghosts_good` / `rsi_ghosts_shown`)",
        en: "Ghost-text precision — % good (`rsi_ghosts_good` / `rsi_ghosts_shown`)",
        dir_key: "precision",
        keys: &["rsi_ghosts_good", "rsi_ghosts_shown"],
        derived: Some("PRECISION"),
        pct: true,
        per100: false,
    },
    Row {
        vi: "Nhiễu chữ mờ — đếm + số trên 100 phím cơ sở (`rsi_ghosts_shown` − `rsi_ghosts_good`)",
        en: "Ghost-text noise — count + per 100 baseline keys (`rsi_ghosts_shown` − `rsi_ghosts_good`)",
        dir_key: "rsi_noise",
        keys: &["rsi_noise", "rsi_baseline_keys"],
        derived: Some("NOISE"),
        pct: false,
        per100: true,
    },
    Row {
        vi: "Chấp nhận gợi ý — đếm (Dream RSI `rsi_accepts`)",
        en: "Suggestion accepts — count (Dream RSI `rsi_accepts`)",
        dir_key: "rsi_accepts",
        keys: &["rsi_accepts"],
        derived: None,
        pct: false,
        per100: false,
    },
    Row {
        vi: "Trợ lý ngữ pháp — recall % (`assist_detected` / `assist_errors`)",
        en: "Grammar assistant — recall % (`assist_detected` / `assist_errors`)",
        dir_key: "assist_detected",
        keys: &["assist_detected", "assist_errors"],
        derived: Some("RECALL"),
        pct: true,
        per100: false,
    },
    Row {
        vi: "Trợ lý ngữ pháp — FP trên câu sạch (`assist_fp_hard`)",
        en: "Grammar assistant — FP on clean sentences (`assist_fp_hard`)",
        dir_key: "assist_fp_hard",
        keys: &["assist_fp_hard"],
        derived: Some("FP"),
        pct: false,
        per100: false,
    },
    Row {
        vi: "Mã lỗi đã sửa — đếm (`regression_codes`)",
        en: "Fixed error codes — count (`regression_codes`)",
        dir_key: "regression_codes",
        keys: &["regression_codes"],
        derived: Some("CODES"),
        pct: false,
        per100: false,
    },
    Row {
        vi: "Nợ hồi quy — đếm (`regression_todo`)",
        en: "Regression debt — count (`regression_todo`)",
        dir_key: "regression_todo",
        keys: &["regression_todo"],
        derived: Some("DEBT"),
        pct: false,
        per100: false,
    },
    Row {
        vi: "Rò rỉ tập đánh giá — đếm (`eval_leak`)",
        en: "Eval-set leaks — count (`eval_leak`)",
        dir_key: "eval_leak",
        keys: &["eval_leak"],
        derived: Some("LEAK"),
        pct: false,
        per100: false,
    },
];

/// Giá trị của hàng trong một cột; `None` = thiếu khoá underlying (schema tiến hoá) ⇒ ô `—`.
fn row_value(col: &Col, disp: &BTreeMap<&'static str, f64>, row: &Row) -> Option<f64> {
    if row.keys.iter().any(|k| !col.k.contains_key(*k)) {
        return None;
    }
    if row.per100 {
        // Audit view-2 2026-11-28: tệp tay đúc `rsi_baseline_keys` = 0 từng cho ô
        // "inf/NaN /100" — coi như KPI không đo được (ô `—`) thay vì chia cho 0.
        if col.k["rsi_baseline_keys"] <= 0.0 {
            return None;
        }
    }
    Some(match row.derived {
        Some(name) => disp[name],
        None => col.k[row.dir_key],
    })
}

/// Hàng có phụ thuộc corpus đánh giá không — tái dùng `needs_same_eval` trên các khoá
/// underlying (G22): hàng có khoá đầu `rsi_` thì chỉ so khi `eval_hash` trùng.
fn row_needs_same_eval(row: &Row) -> bool {
    row.keys.iter().any(|k| needs_same_eval(k))
}

/// Mũi tên so với cột liền trước: `↑` tốt hơn, `↓` xấu hơn, `→` ngang — hướng tốt lấy
/// từ `direction()` (G22); ngang = hàng % chênh < 0,5% tương đối, hàng đếm bằng giá trị.
fn arrow(dir: i8, cur: f64, prev: f64, pct: bool) -> &'static str {
    let flat = cur == prev || (pct && prev != 0.0 && ((cur - prev) / prev).abs() < 0.005);
    if flat {
        "→"
    } else {
        let better = if dir >= 0 { cur > prev } else { cur < prev };
        if better {
            "↑"
        } else {
            "↓"
        }
    }
}

/// Ô giá trị của (cột, hàng): giá trị bản đó + quy đổi /100 (hàng nhiễu) + mũi tên so
/// cột liền trước + `⚠` nếu hàng nằm trong danh sách tụt của bản mới nhất.
fn render_cell(
    col: &Col,
    disp: &BTreeMap<&'static str, f64>,
    row: &Row,
    prev: Option<(&Col, &BTreeMap<&'static str, f64>)>,
    warn: bool,
) -> String {
    let Some(v) = row_value(col, disp, row) else {
        return "—".to_string();
    };
    let mut s = if row.pct {
        format!("{v:.1}%")
    } else {
        format!("{v}")
    };
    if row.per100 {
        // Presence của `rsi_baseline_keys` đã đảm bảo trong `row_value`.
        s.push_str(&format!(
            " ({:.1}/100)",
            100.0 * v / col.k["rsi_baseline_keys"]
        ));
    }
    if let Some((pk, pdisp)) = prev {
        match row_value(pk, pdisp, row) {
            None => {} // bản trước thiếu KPI (schema tiến hoá) → không so, không mũi tên
            Some(pv) => {
                if row_needs_same_eval(row) && pk.eval != col.eval {
                    // Corpus đánh giá đổi giữa 2 bản liền nhau → KPI Dream RSI là
                    // baseline mới, không so (cùng ngữ nghĩa `quality-score`).
                    s.push_str(" · n/a (corpus đổi)");
                } else {
                    let mui_ten = arrow(direction(row.dir_key), v, pv, row.pct);
                    s.push_str(&format!(" {mui_ten}"));
                }
            }
        }
    }
    if warn {
        s.push_str(" ⚠");
    }
    s
}

/// Một bảng markdown (VI hoặc EN): cột = các version (mới nhất bên phải), hàng = `ROWS`.
fn render_table(
    label: fn(&Row) -> &'static str,
    cols: &[Col],
    disps: &[BTreeMap<&'static str, f64>],
    warn_keys: &HashSet<String>,
) -> String {
    let mut s = String::from("| KPI");
    for c in cols {
        s.push_str(&format!(" | {}", c.ver));
    }
    s.push_str(" |\n");
    s.push_str(&format!("{}|\n", "|---".repeat(cols.len() + 1)));
    for row in &ROWS {
        s.push_str(&format!("| {} ", label(row)));
        for (i, col) in cols.iter().enumerate() {
            let prev = if i > 0 {
                Some((&cols[i - 1], &disps[i - 1]))
            } else {
                None
            };
            // `⚠` chỉ gắn ở cột mới nhất — hàng đang tụt gate của bản này.
            let warn = i + 1 == cols.len() && warn_keys.contains(row.dir_key);
            s.push_str(&format!(
                "| {} ",
                render_cell(col, &disps[i], row, prev, warn)
            ));
        }
        s.push_str("|\n");
    }
    s
}

/// Trích khoá KPI từ một dòng của `regressions()` (định dạng `"k: old → new"` hoặc
/// `"k = v (phải = 0)"`) — để ánh xạ `⚠` về đúng hàng, không tự tính lại gate.
fn regression_key(entry: &str) -> &str {
    entry.split([':', '=']).next().unwrap_or(entry).trim()
}

/// Toàn bộ đầu ra markdown: hai bảng `## VI` / `## EN` 1:1 (cùng số hàng, cùng ô giá
/// trị — chỉ nhãn và dòng ghi chú khác ngôn ngữ), bản mới nhất ở cột phải.
fn render(cols: &[Col]) -> String {
    let disps: Vec<BTreeMap<&'static str, f64>> = cols
        .iter()
        .map(|c| derived_display(&c.k).into_iter().collect())
        .collect();
    // `⚠` + gate: DUY NHẤT nguồn là `regressions()` của bản mới nhất vs bản liền trước
    // (G22 — cùng ngữ nghĩa quality-score, không viết luật so sánh thứ hai).
    let warn_keys: HashSet<String> = if cols.len() >= 2 {
        let (prev, cur) = (&cols[cols.len() - 2], &cols[cols.len() - 1]);
        regressions(&prev.k, &prev.eval, &cur.k, &cur.eval)
            .iter()
            .map(|e| regression_key(e).to_string())
            .collect()
    } else {
        HashSet::new()
    };
    let corpus_doi = cols.windows(2).any(|w| w[0].eval != w[1].eval);
    let mut s = String::from(
        "# Xu hướng KPI chất lượng theo bản phát hành / Quality KPI trend per release\n\n\
         Nguồn: `perf/quality/*.json` (lệnh chỉ đọc, không ghi file — BIZ-3).\n\n",
    );
    s.push_str("## VI\n\n");
    s.push_str(&render_table(|r| r.vi, cols, &disps, &warn_keys));
    s.push_str("\nGhi chú: `↑` tốt hơn · `↓` xấu hơn · `→` ngang so với cột liền trước (hàng % chênh < 0,5% tương đối; hàng đếm bằng giá trị) · `⚠` KPI tụt gate của bản mới nhất · `—` KPI chưa có ở bản đó (schema tiến hoá) · ô `n/a (corpus đổi)` = `eval_hash` khác giữa hai bản liền nhau. % KSR = phím tiết kiệm / phím cơ sở; precision = gợi ý chữ mờ đúng / tổng gợi ý hiện; recall = lỗi phát hiện / tổng lỗi corpus.\n");
    if corpus_doi {
        s.push_str("\nLưu ý corpus: `eval_hash` khác giữa các bản liền nhau — các ô `n/a (corpus đổi)` không so được (corpus đánh giá mới = baseline mới cho Dream RSI).\n");
    }
    s.push_str("\n## EN\n\n");
    s.push_str(&render_table(|r| r.en, cols, &disps, &warn_keys));
    s.push_str("\nNote: `↑` better · `↓` worse · `→` flat vs the previous column (percent rows: relative change < 0.5%; count rows: equal value) · `⚠` KPI failing the gate on the newest release · `—` KPI missing in that release (schema evolution) · a cell marked `n/a (corpus đổi)` means the adjacent releases have different `eval_hash` (cell markers are shared verbatim across both tables for strict 1:1). % KSR = keystrokes saved / baseline keystrokes; precision = good ghost suggestions / shown; recall = detected errors / total corpus errors.\n");
    if corpus_doi {
        s.push_str("\nCorpus note: `eval_hash` differs between adjacent releases — cells marked `n/a (corpus đổi)` are not comparable (a new eval corpus means a new Dream RSI baseline).\n");
    }
    s
}

/// Điểm vào kiểm thử được: đọc cột, in bảng (stdout), rồi gate bản mới nhất —
/// `regressions()` khác rỗng ⇒ trả `Err` kèm DANH SÁCH tụt (main in ra stderr, exit ≠ 0).
fn run(root: &Path) -> Result<(), String> {
    let cols = load_columns(root)?;
    println!("{}", render(&cols));
    let bad = match cols.as_slice() {
        // `load_columns` đảm bảo không rỗng.
        [] => unreachable!("load_columns luôn trả về ít nhất 1 cột hoặc Err"),
        // Chưa có bản liền trước ⇒ vẫn gate bất biến tuyệt đối (cùng ngữ nghĩa
        // quality-score khi perf/quality chỉ có 1 bản).
        [one] => regressions(&Kpis::new(), "", &one.k, &one.eval),
        _ => {
            let (prev, cur) = (&cols[cols.len() - 2], &cols[cols.len() - 1]);
            regressions(&prev.k, &prev.eval, &cur.k, &cur.eval)
        }
    };
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "quality-trend: {} KPI tụt ở bản mới nhất — không phát hành (G19/G23): {}",
            bad.len(),
            bad.join("; ")
        ))
    }
}

/// CLI `cargo xtask quality-trend` — không cờ nào (chỉ đọc `perf/quality` + in stdout).
pub fn cli(_args: &[String]) -> Result<(), String> {
    run(&repo_root())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bảng KPI mẫu từ cặp (tên, giá trị).
    fn kp(pairs: &[(&str, f64)]) -> Kpis {
        pairs.iter().map(|(n, v)| (n.to_string(), *v)).collect()
    }

    /// Ghi một tệp `perf/quality/<ver>.json` mẫu vào thư mục tạm (đúng định dạng `parse_file` đọc).
    fn ghi_file(dir: &Path, ver: &str, eval: &str, k: &Kpis) {
        let body: Vec<String> = k.iter().map(|(n, v)| format!("\"{n}\":{v}")).collect();
        fs::write(
            dir.join(format!("{ver}.json")),
            format!(
                "{{\"version\":\"{ver}\",\"eval_hash\":\"{eval}\",{}}}\n",
                body.join(",")
            ),
        )
        .unwrap();
    }

    /// Thư mục tạm riêng cho từng test (không đụng repo).
    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("textvn-biz3-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("perf/quality")).unwrap();
        d
    }

    /// Các dòng bảng `|...|` của một mục (`## VI` hoặc `## EN`) trong đầu ra.
    fn bang_lines(out: &str, from: &str, to: &str) -> Vec<String> {
        let seg = out.split(from).nth(1).unwrap();
        let seg = seg.split(to).next().unwrap();
        seg.lines()
            .filter(|l| l.starts_with('|'))
            .map(str::to_string)
            .collect()
    }

    /// Bỏ cột nhãn (ô đầu) để so 1:1 phần giá trị giữa bảng VI và bảng EN.
    fn bo_nhan(line: &str) -> String {
        line.split('|').skip(2).collect::<Vec<_>>().join("|")
    }

    /// Bản KPI đầy đủ dùng chung cho các test (số liệu thật của 0.2.96).
    fn kpi_day_du() -> Kpis {
        kp(&[
            ("rsi_saved_keys", 687.0),
            ("rsi_baseline_keys", 9469.0),
            ("rsi_vi_saved", 524.0),
            ("rsi_vi_baseline", 6598.0),
            ("rsi_en_saved", 145.0),
            ("rsi_en_baseline", 2556.0),
            ("rsi_ghosts_good", 480.0),
            ("rsi_ghosts_shown", 3185.0),
            ("rsi_noise", 2705.0),
            ("rsi_accepts", 395.0),
            ("assist_detected", 41.0),
            ("assist_errors", 49.0),
            ("assist_fp_hard", 0.0),
            ("regression_codes", 231.0),
            ("regression_todo", 0.0),
            ("eval_leak", 0.0),
        ])
    }

    /// 2 bản mẫu (KSR tăng, nhiễu tăng): đúng %, đúng mũi tên ↑/↓, đúng ⚠ ở nhiễu.
    #[test]
    fn biz_3_bang_xu_huong_dung_gia_tri_va_huong() {
        let root = tmp("huong");
        let dir = root.join("perf/quality");
        let v1 = kpi_day_du();
        let mut v2 = v1.clone();
        v2.insert("rsi_saved_keys".into(), 749.0); // KSR 7.3% → 7.9% — ↑
        v2.insert("rsi_noise".into(), 2800.0); // nhiễu tăng — ↓ và ⚠ (tụt gate)
        ghi_file(&dir, "0.2.96", "h1", &v1);
        ghi_file(&dir, "0.2.97", "h1", &v2);
        let cols = load_columns(&root).unwrap();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].ver, "0.2.96"); // tăng dần ver_key — bản mới nhất bên phải
        assert_eq!(cols[1].ver, "0.2.97");
        let out = render(&cols);
        // KSR tổng: 687/9469 = 7.3% → 749/9469 = 7.9% ↑ (2 bảng VI + EN).
        assert_eq!(out.matches("| 7.3% | 7.9% ↑ |").count(), 2);
        // Nhiễu: 2705 (28.6/100) → 2800 (29.6/100) ↓ kèm ⚠ (rsi_noise trong regressions).
        assert_eq!(
            out.matches("| 2705 (28.6/100) | 2800 (29.6/100) ↓ ⚠ |")
                .count(),
            2
        );
        // Accepts giữ nguyên → ngang.
        assert_eq!(out.matches("| 395 | 395 → |").count(), 2);
        // Cột cũ (0.2.96) không bao giờ có ⚠.
        assert!(!out.contains("(28.6/100) ⚠"));
        let _ = fs::remove_dir_all(&root);
    }

    /// G18 1:1: bảng VI và EN cùng số hàng + cùng ô giá trị (bỏ cột nhãn là so được).
    #[test]
    fn biz_3_vi_en_dong_nhat_1_1() {
        let root = tmp("1-1");
        let dir = root.join("perf/quality");
        let v = kp(&[
            ("rsi_saved_keys", 687.0),
            ("rsi_baseline_keys", 9469.0),
            ("rsi_noise", 2705.0),
            ("eval_leak", 0.0),
        ]);
        ghi_file(&dir, "0.2.96", "h", &v);
        ghi_file(&dir, "0.2.97", "h", &v);
        let out = render(&load_columns(&root).unwrap());
        let vi = bang_lines(&out, "## VI", "## EN");
        let en = bang_lines(&out, "## EN", "\u{0}");
        assert_eq!(vi.len(), en.len());
        assert_eq!(vi.len(), 1 + 1 + ROWS.len()); // header + separator + 11 hàng KPI
        for (a, b) in vi.iter().zip(en.iter()) {
            assert_eq!(
                bo_nhan(a),
                bo_nhan(b),
                "ô giá trị VI/EN phải giống hệt: {a}"
            );
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// eval_hash khác giữa 2 bản liền nhau: hàng rsi_* không mũi tên, ghi
    /// `n/a (corpus đổi)`; hàng trợ lý vẫn so; chân bảng có chú thích corpus (VI lẫn EN).
    #[test]
    fn biz_3_eval_hash_khac_bo_mui_ten_rsi() {
        let root = tmp("corpus");
        let dir = root.join("perf/quality");
        let mut v1 = kpi_day_du();
        v1.insert("assist_detected".into(), 40.0);
        let mut v2 = v1.clone();
        v2.insert("rsi_saved_keys".into(), 749.0); // rsi_* → n/a vì corpus đổi
        v2.insert("assist_detected".into(), 42.0); // recall 81.6% → 85.7% ↑ (vẫn so)
        ghi_file(&dir, "0.2.96", "h1", &v1);
        ghi_file(&dir, "0.2.97", "h2", &v2);
        let out = render(&load_columns(&root).unwrap());
        // Đúng 6 hàng rsi_* (KSR tổng/VI/EN, precision, nhiễu, accepts) × 2 bảng = 12 dòng.
        let na_lines: Vec<&str> = out
            .lines()
            .filter(|l| l.starts_with('|') && l.contains("n/a (corpus đổi)"))
            .collect();
        assert_eq!(na_lines.len(), 12);
        for line in &na_lines {
            assert!(
                !line.contains('↑') && !line.contains('↓'),
                "hàng corpus đổi không được có mũi tên: {line}"
            );
        }
        // Trợ lý recall không phụ thuộc corpus: 40/49 = 81.6% → 42/49 = 85.7% ↑ (2 bảng).
        assert_eq!(out.matches("| 81.6% | 85.7% ↑ |").count(), 2);
        // Gate bỏ qua KPI Dream RSI khi corpus đổi ⇒ không ⚠ nào trong bảng, run Ok.
        assert!(!out
            .lines()
            .filter(|l| l.starts_with('|'))
            .any(|l| l.contains('⚠')));
        assert!(run(&root).is_ok());
        // Chú thích corpus đổi ở cả 2 mục VI + EN.
        assert!(out.contains("corpus đánh giá mới = baseline mới cho Dream RSI"));
        assert!(out.contains("a new eval corpus means a new Dream RSI baseline"));
        let _ = fs::remove_dir_all(&root);
    }

    /// Nhiễu tăng qua ngưỡng ⇒ regressions() khác rỗng ⇒ run trả Err (exit ≠ 0);
    /// chưa có tệp nào cũng Err (fail-closed).
    #[test]
    fn biz_3_kpi_tut_bao_loi_va_exit() {
        // Chưa có tệp nào trong perf/quality ⇒ Err với thông báo chạy B4c trước.
        let empty_root = tmp("rong");
        let e = run(&empty_root).unwrap_err();
        assert!(e.contains("chưa có perf/quality/<ver>.json"), "{e}");
        assert!(e.contains("quality-score --write"), "{e}");
        let _ = fs::remove_dir_all(&empty_root);

        let root = tmp("gate");
        let dir = root.join("perf/quality");
        let v1 = kp(&[("rsi_noise", 100.0), ("rsi_baseline_keys", 1000.0)]);
        let mut v2 = v1.clone();
        v2.insert("rsi_noise".into(), 200.0); // nhiễu tăng — tụt gate (corpus giống nhau)
        ghi_file(&dir, "0.2.96", "h", &v1);
        ghi_file(&dir, "0.2.97", "h", &v2);
        let e = run(&root).unwrap_err();
        assert!(
            e.contains("rsi_noise: 100 → 200"),
            "thiếu danh sách tụt: {e}"
        );
        assert!(e.contains("không phát hành"), "{e}");
        // Cùng dữ liệu, nhiễu giữ nguyên ⇒ đạt (Err chỉ khi có tụt thật).
        let mut v2b = v1.clone();
        v2b.insert("assist_detected".into(), 40.0);
        v2b.insert("assist_errors".into(), 50.0);
        ghi_file(&dir, "0.2.97", "h", &v2b);
        assert!(run(&root).is_ok());
        let _ = fs::remove_dir_all(&root);
    }

    /// KPI thiếu ở bản cũ (schema tiến hoá): ô `—`, không mũi tên; bản mới có khoá mà
    /// bản cũ không có thì cũng không mũi tên (không có gì để so).
    #[test]
    fn biz_3_kpi_thieu_o_ban_cu() {
        let root = tmp("thieu");
        let dir = root.join("perf/quality");
        let v1 = kp(&[("rsi_saved_keys", 687.0), ("rsi_baseline_keys", 9469.0)]); // bản cũ chưa có KPI trợ lý
        let v2 = kp(&[
            ("rsi_saved_keys", 687.0),
            ("rsi_baseline_keys", 9469.0),
            ("assist_detected", 41.0),
            ("assist_errors", 49.0),
        ]);
        ghi_file(&dir, "0.2.96", "h", &v1);
        ghi_file(&dir, "0.2.97", "h", &v2);
        let out = render(&load_columns(&root).unwrap());
        // Hàng recall: ô cũ `—`, ô mới 83.7% (41/49) KHÔNG mũi tên (2 bảng).
        assert_eq!(out.matches("| — | 83.7% |").count(), 2);
        // Thiếu khoá bị bỏ qua khi gate (cùng ngữ nghĩa `regressions`) ⇒ Ok.
        assert!(run(&root).is_ok());
        let _ = fs::remove_dir_all(&root);
    }

    /// Render tất định: gọi lại cho ra chuỗi giống hệt (không thời gian/ngẫu nhiên),
    /// và bảng 1 cột không có mũi tên nào.
    #[test]
    fn biz_3_render_tat_dinh_lap_lai_giong_he() {
        let root = tmp("tat-dinh");
        let dir = root.join("perf/quality");
        let v = kp(&[
            ("rsi_saved_keys", 687.0),
            ("rsi_baseline_keys", 9469.0),
            ("rsi_noise", 2705.0),
            ("eval_leak", 0.0),
        ]);
        ghi_file(&dir, "0.2.96", "h", &v);
        let cols = load_columns(&root).unwrap();
        let out1 = render(&cols);
        let out2 = render(&cols);
        assert_eq!(out1, out2, "render phải tất định");
        for line in out1.lines().filter(|l| l.starts_with('|')) {
            assert!(
                !line.contains('→') && !line.contains('↑') && !line.contains('↓'),
                "bảng 1 cột không được có mũi tên: {line}"
            );
        }
        assert!(out1.contains("| KPI | 0.2.96 |"));
        let _ = fs::remove_dir_all(&root);
    }

    /// Đột biến (b): `derived_display` là DUY NHẤT nguồn công thức — số mẫu khớp đúng
    /// công thức chuỗi in của `cli()`. Ai sửa công thức một đường thôi ⇒ test này đỏ.
    #[test]
    fn biz_3_derived_display_khop_cong_thuc_cli() {
        let mut k = kpi_day_du();
        k.insert("regression_codes".into(), 234.0);
        let d: BTreeMap<&'static str, f64> = derived_display(&k).into_iter().collect();
        assert_eq!(d["KSR"], 100.0 * 687.0 / 9469.0);
        assert_eq!(d["KSR_VI"], 100.0 * 524.0 / 6598.0);
        assert_eq!(d["KSR_EN"], 100.0 * 145.0 / 2556.0);
        assert_eq!(d["PRECISION"], 100.0 * 480.0 / 3185.0);
        assert_eq!(d["NOISE"], 2705.0);
        assert_eq!(d["RECALL"], 100.0 * 41.0 / 49.0);
        assert_eq!(d["FP"], 0.0);
        assert_eq!(d["CODES"], 234.0);
        assert_eq!(d["DEBT"], 0.0);
        assert_eq!(d["LEAK"], 0.0);
        assert_eq!(
            derived_display(&k).len(),
            10,
            "đủ 10 giá trị cho chuỗi in cli()"
        );
    }

    /// Audit view-2 2026-11-28 (1): JSON hỏng/ghi dở dang (crash giữa `fs::write` của
    /// B4c) — `parse_file` khoan dung trả Kpis rỗng từng lọt qua ⇒ cột toàn `—` mà vẫn
    /// Ok (fail-open). `load_columns` phải Err ngay.
    #[test]
    fn biz_3_tep_json_hong_fail_closed() {
        let root = tmp("hong");
        let dir = root.join("perf/quality");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("0.2.96.json"), "{khong phai json vo dinh").unwrap();
        let err = load_columns(&root).unwrap_err();
        assert!(
            err.contains("không parse ra KPI nào"),
            "thông báo phải nêu lỗi parse + chỉ dẫn B4c: {err}"
        );
        assert!(
            err.contains("B4c"),
            "chỉ dẫn chạy lại quality-score --write: {err}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Audit view-2 2026-11-28 (2): tệp tay đúc `rsi_baseline_keys` = 0 — hàng nhiễu
    /// phải là ô `—` (KPI không đo được), tuyệt đối không in "inf/NaN /100".
    #[test]
    fn biz_3_baseline_bang_0_hang_nhieu_la_gach() {
        let root = tmp("bas0");
        let dir = root.join("perf/quality");
        let v = kp(&[
            ("rsi_baseline_keys", 0.0),
            ("rsi_saved_keys", 0.0),
            ("rsi_noise", 5.0),
            ("rsi_ghosts_shown", 10.0),
            ("rsi_ghosts_good", 5.0),
        ]);
        ghi_file(&dir, "0.2.96", "h", &v);
        let out = render(&load_columns(&root).unwrap());
        for line in out.lines().filter(|l| l.starts_with('|')) {
            assert!(
                !line.contains("inf") && !line.contains("NaN"),
                "không được chia cho 0: {line}"
            );
        }
        assert_eq!(
            out.matches("| — |").count(),
            18,
            "9 hàng không đo được (KSR VI/EN, nhiễu-baseline-0, accepts, recall, FP, mã, nợ, rò rỉ) × 2 bảng = 18 ô —; không sinh inf/NaN"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
