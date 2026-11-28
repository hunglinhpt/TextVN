// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask preflight` — **bắt buộc exit 0 trước khi commit/tag**.
//!
//! Mô phỏng đúng các gate của `ci-shared` + `repo-hygiene` theo thứ tự, fail
//! fast, in PASS/FAIL từng bước kèm lời giải thích "gate này chặn loại lỗi gì".
//!
//! Nguyên tắc (bài học 2026-10-02 — clippy đỏ 2 lần vì pipe che exit code):
//! mọi bước chạy bằng `Command::status()` — exit code **thật**, không bao giờ
//! gọi qua `| tail`/`| grep`. Chạy trước khi commit = CI sau đó thấy đúng
//! working tree này, nên preflight xanh ⇒ CI xanh (trừ nhiễu runner).

use std::process::Command;

struct Step {
    name: &'static str,
    /// Gate này chặn loại lỗi nào (dẫn sự cố thật đã gặp nếu có).
    guards: &'static str,
    argv: &'static [&'static str],
    /// True = thiếu chương trình local chỉ SKIP (CI ubuntu vẫn chặn thật).
    optional: bool,
}

const STEPS: &[Step] = &[
    Step {
        name: "fmt",
        guards: "định dạng code — rustfmt job",
        argv: &["cargo", "fmt", "--all", "--check"],
        optional: false,
    },
    Step {
        name: "clippy",
        guards: "lint -D warnings — chặn unused-mut… (2 lần đỏ 2026-10-02; E8 not_unsafe_ptr_arg_deref, E37 if_same_then_else, E52 CStr có byte 0)",
        argv: &[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        optional: false,
    },
    Step {
        name: "check-linux",
        guards: "cross-compile Linux — chặn cfg(windows) lọt ra ngoài (sự cố E11, E9 import cfg(windows) trên ubuntu CI)",
        argv: &[
            "cargo",
            "check",
            "--workspace",
            "--exclude",
            "textvn-win-hook",
            "--all-targets",
            "--target",
            "x86_64-unknown-linux-gnu",
        ],
        optional: false,
    },
    Step {
        name: "clippy-linux",
        guards: "clippy trên target Linux — chặn dead_code/cfg chỉ lộ ở non-Windows (CI clippy chạy trên ubuntu; 3050cd6; E14 unused import)",
        argv: &[
            "cargo", "clippy", "--workspace", "--exclude", "textvn-win-hook",
            "--all-targets", "--target", "x86_64-unknown-linux-gnu", "--", "-D", "warnings",
        ],
        optional: false,
    },
    Step {
        name: "test",
        guards: "unit + integration toàn workspace",
        argv: &["cargo", "test", "--workspace"],
        optional: false,
    },
    Step {
        name: "verify-abi",
        guards: "header C khớp code Rust + sizeof/offset (P0-2 §6)",
        argv: &["cargo", "run", "-q", "-p", "textvn-cli", "--", "verify"],
        optional: false,
    },
    Step {
        name: "sizes",
        guards: "IME ABI 20/532 — đổi struct phải bump ABI",
        argv: &["cargo", "run", "-q", "-p", "textvn-cli", "--", "sizes"],
        optional: false,
    },
    Step {
        name: "replay-win",
        guards: "corpus engine + Windows adapter (typing smoke CI)",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "textvn-cli",
            "--",
            "replay",
            "corpus/shared",
            "corpus/win",
            "--adapter",
            "win",
        ],
        optional: false,
    },
    Step {
        name: "replay-tsf",
        guards: "corpus với preedit/composition TSF thật",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "textvn-cli",
            "--",
            "replay",
            "corpus/shared",
            "corpus/win",
            "--adapter",
            "tsf",
        ],
        optional: false,
    },
    Step {
        name: "replay-mac",
        guards: "corpus macOS IMK",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "textvn-cli",
            "--",
            "replay",
            "corpus/shared",
            "corpus/mac",
            "--adapter",
            "mac",
        ],
        optional: false,
    },
    Step {
        name: "replay-linux",
        guards: "corpus trên adapter Linux (IBus/Fcitx5) — phủ EN/VI-detect nền tảng Linux (0.2.13)",
        argv: &[
            "cargo", "run", "-q", "-p", "textvn-cli", "--", "replay",
            "corpus/shared", "--adapter", "linux",
        ],
        optional: false,
    },
    Step {
        name: "check-tables",
        guards: "bảng keymap sinh ra khớp generator",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "check-tables"],
        optional: false,
    },
    Step {
        name: "check-en-office",
        guards: "data/en_office_phrases.tsv khớp generator Rust (E59)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "check-en-office"],
        optional: false,
    },
    Step {
        name: "check-win-corpus",
        guards: "corpus win sinh ra khớp danh sách case",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "check-win-corpus",
        ],
        optional: false,
    },
    Step {
        name: "check-mac-corpus",
        guards: "corpus mac sinh ra khớp",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "check-mac-corpus",
        ],
        optional: false,
    },
    Step {
        name: "version-sync",
        guards: "15 chỗ ghi version khớp Cargo.toml",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "check-version-sync",
        ],
        optional: false,
    },
    Step {
        name: "perf",
        guards: "hồi quy hiệu năng so baseline (baseline phải re-record từ CI)",
        argv: &[
            "cargo",
            "run",
            "-q",
            "--release",
            "-p",
            "textvn-bench",
            "--",
            "check",
            "perf/baseline-win.json",
        ],
        optional: false,
    },
    Step {
        name: "quality",
        guards: "bảng KPI chung BIZ-1 (Dream RSI oracle + trợ lý recall/FP + nợ hồi quy + rò rỉ eval): KPI tụt so bản trước trong perf/quality/ ⇒ đỏ; bất biến (rò rỉ/đột biến/vi phạm bất biến chữ mờ/nợ todo) phải = 0",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "quality-score"],
        optional: false,
    },
    Step {
        name: "hygiene-apis",
        guards: "API giống malware bị cấm (Farch-3 / AV policy)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "hygiene-apis"],
        optional: false,
    },
    Step {
        name: "store-validate",
        guards: "3 bước manual package validation của Microsoft trên dist/TextVN-setup-*.exe (silent/ARP/bundleware + uninstall) — Checklist S; máy dev: SKIP khi đang cài TextVN, khi dist/ khác version Cargo.toml, hoặc chưa đặt TEXTVN_STORE_VALIDATE=1 (harness cài/gỡ THẬT — E18); CI chạy thật",
        argv: &[
            "powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
            "tools/win/validate-store-package.ps1", "-AllowSkip",
        ],
        optional: false,
    },
    Step {
        name: "iss-tabs",
        guards: ".iss không chứa TAB — escape backslash-t bị ghi thành TAB phá pattern/đường dẫn (B12/B13; ISCC không bắt)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "iss-tabs"],
        optional: false,
    },
    Step {
        name: "ascii-ps1",
        guards: ".ps1 ASCII-only (G7/A5, E32) — PowerShell 5.1 đọc ANSI, ký tự ngoài ASCII phá cú pháp (sự cố test-portable.ps1 2026-10-03)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "ascii-ps1"],
        optional: false,
    },
    Step {
        name: "no-python",
        guards: "E59: repo không được chứa file .py (AGENTS.md language policy — mọi tool đã port sang cargo xtask)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "no-python"],
        optional: false,
    },
    Step {
        name: "eval-leak",
        guards: "RSI-2: tập đánh giá Dream RSI (corpus/eval) không trùng ≥6 token với data/*phrases* — chống KSR thổi phồng (bài học số 22%)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "eval-leak"],
        optional: false,
    },
    Step {
        name: "regression-index",
        guards: "REG-1: mã lỗi đã sửa (E/R/R2/win) ↔ test/gate còn tồn tại — xoá test chặn lỗi cũ hoặc thêm mã không bằng chứng = đỏ (P0-5 §9b)",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "regression-index"],
        optional: false,
    },
    Step {
        name: "hygiene-docs",
        guards: "link markdown nội bộ không hỏng",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "hygiene-docs"],
        optional: false,
    },
    Step {
        name: "hygiene-secrets",
        guards: "chặn rò rỉ private key, token, secret vào git repo",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "hygiene-secrets"],
        optional: false,
    },
    Step {
        name: "homebrew",
        guards: "cask Ruby hợp lệ + sha256 đúng định dạng (B7b)",
        argv: &["ruby", "-c", "packaging/homebrew/textvn.rb"],
        // Windows local thường không có ruby — SKIP nếu thiếu (CI ubuntu chặn).
        optional: true,
    },
];

/// PERF-1: ánh xạ `(arch, os)` sang Rust target triple chuẩn.
pub fn target_triple_for(arch: &str, os: &str) -> &'static str {
    match (arch, os) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86", "windows") => "i686-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => "unknown",
    }
}

/// Chọn baseline theo máy đang chạy (R2-176): file đúng triple nếu có; fallback
/// `perf/baseline-win.json` CHỈ khi host chính là x86_64-windows (đó là số thật của
/// triple đó). Host khác mà thiếu baseline arch mình ⇒ `None` — bước perf phải SKIP
/// có cảnh báo, tuyệt đối không so số chéo arch (gate vô nghĩa = fail-open).
pub fn baseline_path_for(root: &std::path::Path, arch: &str, os: &str) -> Option<String> {
    let triple = target_triple_for(arch, os);
    let rel = format!("perf/baseline-{triple}.json");
    if root
        .join("perf")
        .join(format!("baseline-{triple}.json"))
        .is_file()
    {
        return Some(rel);
    }
    (triple == "x86_64-pc-windows-msvc").then(|| "perf/baseline-win.json".to_string())
}

pub fn run() -> Result<(), String> {
    println!("preflight — mirror các gate CI, chạy trước khi commit/tag");
    println!("(không pipe, fail-fast; working tree = những gì CI sẽ thấy)\n");

    let perf_baseline = baseline_path_for(
        std::path::Path::new("."),
        std::env::consts::ARCH,
        std::env::consts::OS,
    );
    let mut failed: Vec<String> = Vec::new();
    let mut skipped: usize = 0;
    for (i, step) in STEPS.iter().enumerate() {
        print!(
            "[{:>2}/{}] {:<15} — {} … ",
            i + 1,
            STEPS.len(),
            step.name,
            step.guards
        );
        if step.name == "perf" {
            let Some(baseline) = perf_baseline.as_deref() else {
                println!(
                    "SKIP — chưa có baseline re-record từ CI cho arch này (R2-176); CI ghi số thật bằng `textvn-bench write --auto` + artifact"
                );
                skipped += 1;
                continue;
            };
            let mut argv: Vec<&str> = step.argv[1..].to_vec();
            if let Some(last) = argv.last_mut() {
                *last = baseline;
            }
            let mut cmd = Command::new(step.argv[0]);
            cmd.args(&argv);
            match cmd.status() {
                Ok(s) if s.success() => println!("PASS"),
                Ok(s) => {
                    println!("FAIL (exit {})", s.code().unwrap_or(-1));
                    failed.push(step.name.to_string());
                    break; // fail-fast: sửa hết lỗi này đã rồi mới tới lỗi kế
                }
                Err(e) => {
                    println!("KHÔNG CHẠY ĐƯỢC ({e})");
                    failed.push(format!("{}: {e}", step.name));
                    break;
                }
            }
            continue;
        }
        let mut cmd = Command::new(step.argv[0]);
        cmd.args(&step.argv[1..]);
        let status = cmd.status();
        match status {
            Ok(s) if s.success() => println!("PASS"),
            Ok(s) => {
                println!("FAIL (exit {})", s.code().unwrap_or(-1));
                failed.push(step.name.to_string());
                break; // fail-fast: sửa hết lỗi này đã rồi mới tới lỗi kế
            }
            Err(e) if step.optional => {
                println!("SKIP (thiếu chương trình local: {e}) — CI sẽ chặn thật")
            }
            Err(e) => {
                println!("KHÔNG CHẠY ĐƯỢC ({e})");
                failed.push(format!("{}: {e}", step.name));
                break;
            }
        }
    }

    if failed.is_empty() {
        if skipped > 0 {
            println!(
                "\nOK — {} bước PASS, {skipped} SKIP (thiếu baseline arch này — R2-176). An toàn để commit/tag (CI sẽ thấy đúng cây này).",
                STEPS.len() - skipped
            );
        } else {
            println!(
                "\nOK — {} bước PASS. An toàn để commit/tag (CI sẽ thấy đúng cây này).",
                STEPS.len()
            );
        }
        Ok(())
    } else {
        Err(format!(
            "\nFAIL tại: {}. Sửa xong chạy lại `cargo xtask preflight` trước khi commit.",
            failed.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf1_preflight_chon_dung_baseline_theo_may() {
        let dir =
            std::env::temp_dir().join(format!("textvn-perf1-preflight-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("perf")).unwrap();

        for (arch, os, triple) in [
            ("x86_64", "windows", "x86_64-pc-windows-msvc"),
            ("aarch64", "windows", "aarch64-pc-windows-msvc"),
            ("aarch64", "linux", "aarch64-unknown-linux-gnu"),
            ("x86_64", "linux", "x86_64-unknown-linux-gnu"),
            ("aarch64", "macos", "aarch64-apple-darwin"),
        ] {
            let name = format!("baseline-{triple}.json");
            std::fs::write(dir.join("perf").join(&name), "{}").unwrap();
            assert_eq!(
                baseline_path_for(&dir, arch, os),
                Some(format!("perf/{name}"))
            );
        }

        // R2-176: KHÔNG có file đúng triple ⇒ chỉ x86_64-windows được dùng baseline-win
        // (số thật của đúng arch đó); arch khác ⇒ None (bước perf SKIP có cảnh báo,
        // cấm so số chéo arch). File x86_64-windows cụ thể do vòng lặp tạo ra ở trên
        // phải xoá để chạm đúng nhánh fallback.
        std::fs::remove_file(
            dir.join("perf")
                .join("baseline-x86_64-pc-windows-msvc.json"),
        )
        .unwrap();
        assert_eq!(
            baseline_path_for(&dir, "x86_64", "windows"),
            Some("perf/baseline-win.json".to_string())
        );
        assert_eq!(baseline_path_for(&dir, "riscv64", "linux"), None);
        // Xoá các file triple cụ thể ⇒ arch tương ứng chuyển sang None (bước perf SKIP
        // có cảnh báo) — đúng trạng thái repo sau khi bỏ baseline số liệu giả (R2-176).
        for (arch, os, triple) in [
            ("aarch64", "windows", "aarch64-pc-windows-msvc"),
            ("aarch64", "linux", "aarch64-unknown-linux-gnu"),
            ("x86_64", "linux", "x86_64-unknown-linux-gnu"),
            ("aarch64", "macos", "aarch64-apple-darwin"),
        ] {
            std::fs::remove_file(dir.join("perf").join(format!("baseline-{triple}.json"))).unwrap();
            assert_eq!(baseline_path_for(&dir, arch, os), None);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
