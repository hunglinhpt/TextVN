// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-cli assist check` - kiểm tra câu tiếng Việt offline (P4-1).
//!
//! Exit: 0 câu sạch · 1 có lỗi · 2 lỗi dùng/thiếu tham số.
//!
//! Không log, không ghi file, không network (S2/P2) — chỉ đọc chuỗi trên dòng
//! lệnh (hoặc file) và in kết quả ra stdout.

use textvn_assist::{apply_auto_fixes, check, check_sentence, Issue};

/// Số lỗi tối đa in ra mặc định (tránh flood stdout khi file lớn — Fassist-56).
const MAX_PRINT_DEFAULT: usize = 50;

/// `textvn-cli assist check <text>|--file <path> [--json] [--fix] [--sentence]`
pub fn cmd_assist(rest: &[String]) -> i32 {
    // `assist` có subcommand `check` (args[2] = "check") và `score`.
    let rest = match rest.first().map(String::as_str) {
        Some("check") => &rest[1..],
        Some("score") => return cmd_score(&rest[1..]),
        Some("show") => return cmd_show(&rest[1..]),
        Some(other) => {
            eprintln!("error: subcommand lạ `{other}` (chỉ hỗ trợ `check`, `score`)");
            eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
            eprintln!("      textvn-cli assist score --corpus <dir>");
            return 2;
        }
        None => {
            eprintln!("error: thiếu subcommand (`check` | `score`)");
            eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
            eprintln!("      textvn-cli assist score --corpus <dir>");
            return 2;
        }
    };

    let mut text: Option<String> = None;
    let mut file: Option<String> = None;
    let mut json = false;
    let mut fix = false;
    let mut sentence = false;
    let mut max_print: usize = MAX_PRINT_DEFAULT;

    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json = true,
            "--fix" => fix = true,
            "--sentence" => sentence = true,
            "--max" => {
                i += 1;
                match rest.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(n) => max_print = n,
                    None => {
                        eprintln!("error: --max cần một số");
                        return 2;
                    }
                }
            }
            "--file" => {
                i += 1;
                match rest.get(i) {
                    Some(p) => file = Some(p.clone()),
                    None => {
                        eprintln!("error: --file cần đường dẫn");
                        return 2;
                    }
                }
            }
            s if s.starts_with("--") => {
                eprintln!("error: tham số lạ `{s}`");
                eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
                eprintln!("      textvn-cli assist check --file <path>");
                return 2;
            }
            s => {
                if text.is_some() {
                    eprintln!("error: chỉ nhận một câu (dùng --file để đọc từ file)");
                    return 2;
                }
                text = Some(s.to_string());
            }
        }
        i += 1;
    }

    match (text, file) {
        (Some(t), None) => run_text(&t, json, fix, sentence, max_print),
        (None, Some(path)) => match std::fs::read_to_string(&path) {
            Ok(content) => run_file(&path, &content, json, fix, sentence, max_print),
            Err(e) => {
                eprintln!("error: đọc file `{path}` thất bại: {e}");
                2
            }
        },
        (Some(_), Some(_)) => {
            eprintln!("error: không được vừa truyền câu và --file");
            2
        }
        (None, None) => {
            eprintln!("error: thiếu câu cần kiểm tra");
            eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
            eprintln!("      textvn-cli assist check --file <path>");
            2
        }
    }
}

/// Kiểm tra một chuỗi (một câu/đoạn).
fn run_text(source: &str, json: bool, fix: bool, sentence: bool, max_print: usize) -> i32 {
    let issues = if sentence {
        check_sentence(source)
    } else {
        check(source)
    };

    if json {
        println!("{}", to_json(&issues));
    } else {
        print_human(None, &issues, max_print);
        if fix {
            let fixed = apply_auto_fixes(source, &issues);
            if fixed != source {
                println!("=> {fixed}");
            }
        }
    }

    i32::from(!issues.is_empty())
}

/// Kiểm tra từng dòng của file (báo kèm số dòng) — L0 áp theo từng dòng để
/// file song ngữ không bị bỏ qua cả.
fn run_file(
    path: &str,
    content: &str,
    json: bool,
    fix: bool,
    sentence: bool,
    max_print: usize,
) -> i32 {
    let mut all: Vec<(usize, Vec<Issue>)> = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let issues = if sentence {
            check_sentence(line)
        } else {
            check(line)
        };
        if !issues.is_empty() {
            all.push((idx + 1, issues));
        }
    }

    if json {
        let arr: Vec<serde_json::Value> = all
            .iter()
            .flat_map(|(line, issues)| {
                issues.iter().map(move |i| {
                    let mut v = issue_json(i);
                    v["line"] = serde_json::json!(line);
                    v
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
        );
    } else {
        if all.is_empty() {
            println!("OK: không phát hiện lỗi trong `{path}`");
        }
        let mut budget = max_print;
        for (line, issues) in &all {
            if budget == 0 {
                break;
            }
            let take = budget.min(issues.len());
            print_human(Some(*line), &issues[..take], take);
            budget -= take;
            if fix {
                let source = content.lines().nth(line - 1).unwrap_or("");
                let fixed = apply_auto_fixes(source, issues);
                if fixed != source {
                    println!("  dòng {line} sau khi sửa: {fixed}");
                }
            }
        }
        if !all.is_empty() {
            let total: usize = all.iter().map(|(_, v)| v.len()).sum();
            println!("--- {total} lỗi ---");
            if total > max_print {
                println!("(chỉ in {max_print} lỗi đầu — dùng --max <n> để xem thêm)");
            }
        }
    }

    i32::from(!all.is_empty())
}

/// In kết quả dạng người đọc.
fn print_human(line: Option<usize>, issues: &[Issue], max_print: usize) {
    if issues.is_empty() {
        return;
    }
    let prefix = match line {
        Some(l) => format!("dòng {l}: "),
        None => String::new(),
    };
    for issue in issues.iter().take(max_print) {
        let suggestion = issue
            .suggestions
            .first()
            .map(|s| format!(" → {s}"))
            .unwrap_or_default();
        println!(
            "{prefix}[{}] {} ({}..{}){suggestion}",
            issue.kind.as_str(),
            issue.message,
            issue.span.start,
            issue.span.end
        );
    }
}

/// Chuyển 1 issue sang JSON.
fn issue_json(issue: &Issue) -> serde_json::Value {
    serde_json::json!({
        "kind": issue.kind.as_str(),
        "start": issue.span.start,
        "end": issue.span.end,
        "message": issue.message,
        "suggestions": issue.suggestions,
        "auto_fixable": issue.auto_fixable,
        "confidence": issue.confidence,
    })
}

/// `assist show` — render **mẫu overlay** (câu + vạch đánh dấu lỗi).
///
/// Đây là **spec hiển thị** cho lane UI (tray/overlay): mọi lỗi được gạch chân
/// đúng vị trí (offset **ký tự Unicode**) + 1 dòng thông điệp tiếng Việt.
fn cmd_show(rest: &[String]) -> i32 {
    let mut text: Option<String> = None;
    let mut json = false;
    for arg in rest {
        match arg.as_str() {
            "--json" => json = true,
            s if s.starts_with("--") => {
                eprintln!("error: tham số lạ `{s}`");
                return 2;
            }
            s => {
                if text.is_some() {
                    eprintln!("error: chỉ nhận một câu");
                    return 2;
                }
                text = Some(s.to_string());
            }
        }
    }
    let Some(text) = text else {
        eprintln!("error: thiếu câu cần hiển thị");
        eprintln!("dùng: textvn-cli assist show \"<câu>\" [--json]");
        return 2;
    };

    let issues = check(&text);
    if json {
        println!("{}", to_json(&issues));
        return 0;
    }

    let chars: Vec<char> = text.chars().collect();
    println!("{}", text);
    if issues.is_empty() {
        println!("(không có lỗi)");
        return 0;
    }
    for (n, issue) in issues.iter().enumerate() {
        let mut marker = vec![' '; chars.len()];
        for (i, slot) in marker.iter_mut().enumerate() {
            if i >= issue.span.start && i < issue.span.end {
                *slot = '^';
            }
        }
        let marker: String = marker.into_iter().collect();
        let suggestion = issue
            .suggestions
            .first()
            .map(|s| format!("  -> {s}"))
            .unwrap_or_default();
        println!("{marker}");
        println!("  {}. {}{suggestion}", n + 1, issue.message);
    }
    0
}

/// Chuyển danh sách issue sang JSON.
fn to_json(issues: &[Issue]) -> String {
    let arr: Vec<serde_json::Value> = issues.iter().map(issue_json).collect();
    serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
}

// ---------------------------------------------------------------------------
// `assist score` — đo recall / precision / FPR trên corpus (P4-0 §2, D19)
// ---------------------------------------------------------------------------

/// Chấm điểm corpus: recall (loại lỗi mong đợi có được phát hiện), precision
/// (loại lỗi báo ra đúng) và FPR (câu **sạch** bị báo nhầm).
///
/// Định dạng corpus: TSV `<câu>\t<kind1,kind2,...>`; cột kinds rỗng = câu sạch.
fn cmd_score(rest: &[String]) -> i32 {
    let mut corpus: Option<String> = None;
    let mut strict = false;
    let mut per_kind = false;
    let mut max_detail = 20usize;
    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--corpus" => {
                i += 1;
                match rest.get(i) {
                    Some(p) => corpus = Some(p.clone()),
                    None => {
                        eprintln!("error: --corpus cần đường dẫn");
                        return 2;
                    }
                }
            }
            "--strict" => strict = true,
            "--per-kind" => per_kind = true,
            "--max-detail" => {
                i += 1;
                max_detail = rest
                    .get(i)
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(20);
            }
            s => {
                eprintln!("error: tham số lạ `{s}`");
                eprintln!("dùng: textvn-cli assist score --corpus <dir> [--strict] [--per-kind]");
                return 2;
            }
        }
        i += 1;
    }
    let Some(dir) = corpus else {
        eprintln!("error: thiếu --corpus <dir>");
        eprintln!("dùng: textvn-cli assist score --corpus <dir>");
        return 2;
    };

    let path = std::path::Path::new(&dir);
    let mut files: Vec<std::path::PathBuf> = match std::fs::read_dir(path) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("ass"))
            .collect(),
        Err(e) => {
            eprintln!("error: đọc corpus `{dir}` thất bại: {e}");
            return 2;
        }
    };
    files.sort();

    let mut total_expected = 0usize;
    let mut total_matched = 0usize;
    let mut total_actual = 0usize;
    let mut total_false_positive = 0usize;
    let mut total_span_mismatch = 0usize;
    let mut clean_cases = 0usize;
    let mut clean_with_issue = 0usize;
    let mut soft_cases = 0usize;
    let mut soft_flagged = 0usize;
    let mut cases = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    // (số case mong đợi, số lỗi phát hiện, số case khớp) theo từng loại
    let mut per_kind_stats: std::collections::BTreeMap<String, (usize, usize, usize)> =
        std::collections::BTreeMap::new();

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            eprintln!("error: đọc {} thất bại", file.display());
            return 2;
        };
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim_end();
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let (sentence, expected) = line.split_once('\t').unwrap_or((line, ""));
            // Câu tiền tố "~ " = văn bản thật, có thể chứa lỗi THẬT (người viết
            // sai) -> vẫn tính vào mẫu số FPR nhưng báo riêng "ứng viên" (Fassist-97).
            let soft = sentence.starts_with('~');
            let sentence = sentence.trim_start_matches('~').trim_start();
            // Kỳ vọng: `kind` hoặc `kind@start-end` (span **char Unicode**).
            let expected: Vec<(String, Option<(u32, u32)>)> = expected
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| match s.split_once('@') {
                    Some((kind, span)) => {
                        let parts: Vec<&str> = span.split('-').collect();
                        let sp = match (parts.first(), parts.get(1)) {
                            (Some(a), Some(b)) => match (a.parse::<u32>(), b.parse::<u32>()) {
                                (Ok(x), Ok(y)) => Some((x, y)),
                                _ => None,
                            },
                            _ => None,
                        };
                        (kind.to_string(), sp)
                    }
                    None => (s.to_string(), None),
                })
                .collect();
            let actual: Vec<(String, u32, u32)> = check(sentence)
                .into_iter()
                .map(|i| {
                    (
                        i.kind.as_str().to_string(),
                        i.span.start as u32,
                        i.span.end as u32,
                    )
                })
                .collect();

            cases += 1;
            if expected.is_empty() {
                clean_cases += 1;
                if soft {
                    // Van ban that: cau co the chua loi THAT cua nguoi viet -> khong
                    // goi la "false positive", ma goi la "ung vien" (Fassist-97).
                    soft_cases += 1;
                    if !actual.is_empty() {
                        soft_flagged += 1;
                        if mismatches.len() < max_detail {
                            mismatches.push(format!(
                                "UNG VIEN (van ban that) [{}] `{}` -> {:?}",
                                file.display(),
                                sentence,
                                actual.iter().map(|(k, _, _)| k).collect::<Vec<_>>()
                            ));
                        }
                    }
                } else if !actual.is_empty() {
                    clean_with_issue += 1;
                    total_false_positive += actual.len();
                    if mismatches.len() < max_detail {
                        mismatches.push(format!(
                            "FALSE POSITIVE [{}] `{}` -> {:?}",
                            file.display(),
                            sentence,
                            actual.iter().map(|(k, _, _)| k).collect::<Vec<_>>()
                        ));
                    }
                }
                continue;
            }
            total_expected += expected.len();
            total_actual += actual.len();
            for (kind, span) in &expected {
                let same_kind: Vec<&(String, u32, u32)> =
                    actual.iter().filter(|(k, _, _)| k == kind).collect();
                let span_ok = match *span {
                    None => true,
                    Some((s, e)) => same_kind.iter().any(|(_, as_, ae)| *as_ == s && *ae == e),
                };
                if !same_kind.is_empty() {
                    total_matched += 1;
                    if strict && !span_ok {
                        total_span_mismatch += 1;
                    }
                } else if mismatches.len() < max_detail {
                    mismatches.push(format!(
                        "MISSED [{}:{}] `{}` -> thiếu {kind}",
                        file.display(),
                        idx + 1,
                        sentence
                    ));
                }
                let entry = per_kind_stats.entry(kind.clone()).or_insert((0, 0, 0));
                entry.0 += 1;
                entry.1 += same_kind.len();
                entry.2 += usize::from(!same_kind.is_empty());
            }
            for (kind, _, _) in &actual {
                if !expected.iter().any(|(k, _)| k == kind) {
                    total_false_positive += 1;
                    if mismatches.len() < max_detail {
                        mismatches.push(format!(
                            "EXTRA [{}:{}] `{}` -> thua {kind}",
                            file.display(),
                            idx + 1,
                            sentence
                        ));
                    }
                }
            }
        }
    }

    let pct = |num: usize, den: usize| -> f64 {
        if den == 0 {
            0.0
        } else {
            num as f64 * 100.0 / den as f64
        }
    };
    let recall = pct(total_matched, total_expected);
    let precision = pct(total_matched, total_actual);
    let fpr = pct(clean_with_issue, clean_cases);

    println!("--- assist score: {dir} ---");
    println!(
        "cases={cases} (cau sach={clean_cases}, co loi={})",
        cases - clean_cases
    );
    println!("recall    = {recall:.1}%  ({total_matched}/{total_expected} loai loi mong doi)");
    println!("precision = {precision:.1}%  ({total_matched}/{total_actual} loai loi bao ra)");
    println!("FPR       = {fpr:.1}%  ({clean_with_issue}/{clean_cases} cau sach bi bao nhầm)");
    println!("false positives = {total_false_positive}");
    if soft_cases > 0 {
        println!(
            "ung vien (van ban that) = {soft_flagged}/{soft_cases} cau bi gan co — can rao soat tay (co the la loi THAT cua nguoi viet)"
        );
    }
    if strict {
        println!(
            "span khong khop     = {total_span_mismatch} (loi bat dung loai nhung sai vi tri)"
        );
    }
    if per_kind {
        println!("--- theo loai loi ---");
        for (kind, (exp, act, hit)) in &per_kind_stats {
            let r = pct(*hit, *exp);
            println!("  {kind:28} case={exp:4}  phat hien={act:4}  khop={hit:4}  recall={r:.1}%");
        }
    }
    if !mismatches.is_empty() {
        println!("--- chi tiết (toi da {}) ---", mismatches.len());
        for m in &mismatches {
            println!("  {m}");
        }
    }
    0
}
