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
    /// Dùng `python` hay `python3` (Windows không có python3 luôn sẵn).
    python_alt: bool,
    /// True = thiếu chương trình local chỉ SKIP (CI ubuntu vẫn chặn thật).
    optional: bool,
}

const STEPS: &[Step] = &[
    Step {
        name: "fmt",
        guards: "định dạng code — rustfmt job",
        argv: &["cargo", "fmt", "--all", "--check"],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "clippy",
        guards: "lint -D warnings — chặn unused-mut… (2 lần đỏ 2026-10-02)",
        argv: &[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "check-linux",
        guards: "cross-compile Linux — chặn cfg(windows) lọt ra ngoài (sự cố E11)",
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
        python_alt: false,
        optional: false,
    },
    Step {
        name: "clippy-linux",
        guards: "clippy trên target Linux — chặn dead_code/cfg chỉ lộ ở non-Windows (CI clippy chạy trên ubuntu; 3050cd6)",
        argv: &[
            "cargo", "clippy", "--workspace", "--exclude", "textvn-win-hook",
            "--all-targets", "--target", "x86_64-unknown-linux-gnu", "--", "-D", "warnings",
        ],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "test",
        guards: "unit + integration toàn workspace",
        argv: &["cargo", "test", "--workspace"],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "verify-abi",
        guards: "header C khớp code Rust + sizeof/offset (P0-2 §6)",
        argv: &["cargo", "run", "-q", "-p", "textvn-cli", "--", "verify"],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "sizes",
        guards: "IME ABI 20/532 — đổi struct phải bump ABI",
        argv: &["cargo", "run", "-q", "-p", "textvn-cli", "--", "sizes"],
        python_alt: false,
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
        python_alt: false,
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
        python_alt: false,
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
        python_alt: false,
        optional: false,
    },
    Step {
        name: "replay-linux",
        guards: "corpus trên adapter Linux (IBus/Fcitx5) — phủ EN/VI-detect nền tảng Linux (0.2.13)",
        argv: &[
            "cargo", "run", "-q", "-p", "textvn-cli", "--", "replay",
            "corpus/shared", "--adapter", "linux",
        ],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "check-tables",
        guards: "bảng keymap sinh ra khớp generator",
        argv: &["cargo", "run", "-q", "-p", "xtask", "--", "check-tables"],
        python_alt: false,
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
        python_alt: false,
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
        python_alt: false,
        optional: false,
    },
    Step {
        name: "version-sync",
        guards: "14 chỗ ghi version khớp Cargo.toml",
        argv: &[
            "cargo",
            "run",
            "-q",
            "-p",
            "xtask",
            "--",
            "check-version-sync",
        ],
        python_alt: false,
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
        python_alt: false,
        optional: false,
    },
    Step {
        name: "hygiene-apis",
        guards: "API giống malware bị cấm (Farch-3 / AV policy)",
        argv: &[".github/scripts/check_no_injection_apis.py"],
        python_alt: true,
        optional: false,
    },
    Step {
        name: "store-validate",
        guards: "3 bước manual package validation của Microsoft trên dist/TextVN-setup-*.exe (silent/ARP/bundleware + uninstall) — Checklist S; máy dev đang cài TextVN thì SKIP (CI chạy thật)",
        argv: &[
            "powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
            "tools/win/validate-store-package.ps1", "-AllowSkip",
        ],
        python_alt: false,
        optional: false,
    },
    Step {
        name: "iss-tabs",
        guards: ".iss không chứa TAB — escape backslash-t bị ghi thành TAB phá pattern/đường dẫn (B12/B13; ISCC không bắt)",
        argv: &[".github/scripts/check_iss_tabs.py"],
        python_alt: true,
        optional: false,
    },
    Step {
        name: "ascii-ps1",
        guards: ".ps1 ASCII-only (G7/A5) — PowerShell 5.1 đọc ANSI, ký tự ngoài ASCII phá cú pháp (sự cố test-portable.ps1 2026-10-03)",
        argv: &[".github/scripts/check_ps1_ascii.py"],
        python_alt: true,
        optional: false,
    },
    Step {
        name: "hygiene-docs",
        guards: "link markdown nội bộ không hỏng",
        argv: &[".github/scripts/check_doc_links.py"],
        python_alt: true,
        optional: false,
    },
    Step {
        name: "homebrew",
        guards: "cask Ruby hợp lệ + sha256 đúng định dạng (B7b)",
        argv: &["ruby", "-c", "packaging/homebrew/textvn.rb"],
        python_alt: false,
        // Windows local thường không có ruby — SKIP nếu thiếu (CI ubuntu chặn).
        optional: true,
    },
];

fn python_runner() -> &'static str {
    static PY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
    PY.get_or_init(|| {
        for candidate in ["python", "python3"] {
            if Command::new(candidate)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return candidate;
            }
        }
        "python"
    })
}

pub fn run() -> Result<(), String> {
    println!("preflight — mirror các gate CI, chạy trước khi commit/tag");
    println!("(không pipe, fail-fast; working tree = những gì CI sẽ thấy)\n");

    let mut failed: Vec<String> = Vec::new();
    for (i, step) in STEPS.iter().enumerate() {
        print!(
            "[{:>2}/{}] {:<15} — {} … ",
            i + 1,
            STEPS.len(),
            step.name,
            step.guards
        );
        let status = if step.python_alt {
            Command::new(python_runner()).arg(step.argv[0]).status()
        } else {
            let mut cmd = Command::new(step.argv[0]);
            cmd.args(&step.argv[1..]);
            cmd.status()
        };
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
        println!(
            "\nOK — {} bước PASS. An toàn để commit/tag (CI sẽ thấy đúng cây này).",
            STEPS.len()
        );
        Ok(())
    } else {
        Err(format!(
            "\nFAIL tại: {}. Sửa xong chạy lại `cargo xtask preflight` trước khi commit.",
            failed.join(", ")
        ))
    }
}
