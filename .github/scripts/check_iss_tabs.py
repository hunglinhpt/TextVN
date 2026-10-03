#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# check_iss_tabs.py — file .iss KHÔNG được chứa ký tự TAB.
#
# Vì sao: bug lặp 2 lần (2026-10-12..13) — escape `\t` bị ghi thành TAB thật
# trong chuỗi Pascal/đường dẫn, ví dụ `{app}<TAB>extvn-tsf.dll`. Hậu quả:
#   - [UninstallDelete] pattern không bao giờ khớp (.old-* không được dọn);
#   - RenameLockedTsfDll nhận sai đường dẫn → tính năng rename-DLL vô hiệu.
# ISCC compile VẪN XANH (TAB là ký tự hợp lệ trong string literal) nên không có
# gate nào bắt — script này là gate duy nhất.
#
# Dùng CHUNG bởi .github/workflows/repo-hygiene.yml và `cargo xtask preflight`.
import subprocess
import sys


def tracked_iss() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.iss"], capture_output=True, text=True, check=True
    ).stdout
    return [f for f in out.splitlines() if f.strip()]


def main() -> int:
    bad: list[str] = []
    for path in tracked_iss():
        with open(path, "r", encoding="utf-8", newline="") as fh:
            for lineno, line in enumerate(fh, 1):
                if "\t" in line:
                    bad.append(f"{path}:{lineno}")
    if bad:
        print("::error::.iss chua ky tu TAB (escape \\t bi ghi sai):", file=sys.stderr)
        for loc in bad:
            print("  " + loc, file=sys.stderr)
        return 1
    print(f"check_iss_tabs: OK ({len(tracked_iss())} file .iss khong co TAB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
