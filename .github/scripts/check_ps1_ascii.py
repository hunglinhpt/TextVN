#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# check_ps1_ascii.py — quy tắc G7/A5: MỌI file .ps1 trong repo phải ASCII-only.
#
# Vì sao: Windows PowerShell 5.1 đọc .ps1 không BOM theo ANSI codepage của máy
# → ký tự ngoài ASCII bị mojibake và có thể phá cú pháp script trên runner
# (sự cố 2026-10-03: comment tiếng Việt trong test-portable.ps1 làm
# repo-hygiene đỏ vì preflight không có bước này).
#
# Dùng CHUNG bởi .github/workflows/repo-hygiene.yml (bước 3) và
# `cargo xtask preflight` (bước ascii-ps1) — một nguồn sự thật duy nhất.
import subprocess
import sys

def tracked_ps1() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.ps1"], capture_output=True, text=True, check=True
    ).stdout
    return [f for f in out.splitlines() if f.strip()]

def main() -> int:
    bad: list[str] = []
    for path in tracked_ps1():
        with open(path, "rb") as fh:
            data = fh.read()
        if any(b > 0x7F for b in data):
            bad.append(path)
    if bad:
        print("::error::.ps1 phải ASCII-only (G7/A5):", file=sys.stderr)
        for path in bad:
            print("  " + path, file=sys.stderr)
        return 1
    print(f"check_ps1_ascii: OK ({len(tracked_ps1())} file .ps1 ASCII-only)")
    return 0

if __name__ == "__main__":
    sys.exit(main())
