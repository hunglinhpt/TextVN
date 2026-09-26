// SPDX-License-Identifier: GPL-3.0-or-later
//! `vietime` — CLI (P0-1 §2). Slice 1: subcommand `replay` (golden corpus, P0-4).
//! Subcommand khác (`doctor`, `verify`, `register`, `config`, `ipc probe`, `tray --stop`)
//! theo P0-1 §2 — làm dần ở các slice sau.

mod replay;

use std::process::exit;

const USAGE: &str = r#"vietime — VietIME CLI

Usage:
  vietime replay <dir-or-file...> [--adapter headless|win|mac|linux] [--json] [--filter <substring>]
  vietime --help

Exit codes (replay, P0-4 §4):
  0  tất cả pass
  1  có case fail
  2  lỗi dùng / lỗi parse file corpus
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("replay") => {
            let rest = &args[2..];
            let mut paths: Vec<String> = Vec::new();
            let mut adapter = "headless".to_string();
            let mut json = false;
            let mut filter: Option<String> = None;
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--adapter" => {
                        i += 1;
                        adapter = rest.get(i).cloned().unwrap_or_default();
                    }
                    "--json" => json = true,
                    "--filter" => {
                        i += 1;
                        filter = rest.get(i).cloned();
                        if filter.is_none() {
                            eprintln!("error: --filter cần giá trị");
                            exit(2);
                        }
                    }
                    other if other.starts_with("--") => {
                        eprintln!("error: flag lạ `{other}`");
                        exit(2);
                    }
                    other => paths.push(other.to_string()),
                }
                i += 1;
            }
            if paths.is_empty() {
                eprintln!("error: replay cần ít nhất 1 dir/file corpus");
                eprint!("{USAGE}");
                exit(2);
            }
            // Giá trị lạ → exit 2 (P0-4 §4)
            if !matches!(adapter.as_str(), "headless" | "win" | "mac" | "linux") {
                eprintln!("error: --adapter phải là headless|win|mac|linux (nhận `{adapter}`)");
                exit(2);
            }
            exit(replay::run(&paths, &adapter, json, filter.as_deref()));
        }
        Some("--help") | Some("-h") => {
            print!("{USAGE}");
        }
        Some(other) => {
            eprintln!("error: lệnh lạ `{other}`");
            eprint!("{USAGE}");
            exit(2);
        }
        None => {
            eprint!("{USAGE}");
            exit(2);
        }
    }
}
