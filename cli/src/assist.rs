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
    // `assist` có subcommand `check` (args[2] = "check").
    let rest = match rest.first().map(String::as_str) {
        Some("check") => &rest[1..],
        Some(other) => {
            eprintln!("error: subcommand lạ `{other}` (chỉ hỗ trợ `check`)");
            eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
            return 2;
        }
        None => {
            eprintln!("error: thiếu subcommand (`check`)");
            eprintln!("dùng: textvn-cli assist check \"<câu>\" [--json] [--fix] [--sentence]");
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

/// Chuyển danh sách issue sang JSON.
fn to_json(issues: &[Issue]) -> String {
    let arr: Vec<serde_json::Value> = issues.iter().map(issue_json).collect();
    serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
}
