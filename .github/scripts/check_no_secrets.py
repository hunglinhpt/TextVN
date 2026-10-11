#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Kiem tra chong ro ri secret, token, private key trong repository.

Chay local:  python .github/scripts/check_no_secrets.py
Dung trong:  .github/workflows/repo-hygiene.yml va cargo xtask preflight
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

FORBIDDEN_EXTENSIONS = {
    ".key",
    ".pem",
    ".p12",
    ".pfx",
    ".pkcs12",
    ".keystore",
    ".sec",
    ".credentials",
    ".secret",
}

FORBIDDEN_NAME_PREFIXES = (
    ".env",
    "id_rsa",
    "id_ed25519",
    "textvn_signing_ed25519",
)

PATTERNS = [
    (
        "Private Key Header",
        re.compile(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY( BLOCK)?-----"),
    ),
    (
        "GitHub Token",
        re.compile(r"(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}"),
    ),
    (
        "AWS Access Key",
        re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    ),
]


def list_tracked_files():
    try:
        out = subprocess.check_output(
            ["git", "ls-files"], cwd=ROOT, text=True, encoding="utf-8"
        )
        return [f.strip() for f in out.splitlines() if f.strip()]
    except Exception as e:
        print(f"Error running git ls-files: {e}")
        return []


def is_binary_file(path: pathlib.Path) -> bool:
    try:
        with open(path, "rb") as f:
            chunk = f.read(1024)
            return b"\x00" in chunk
    except Exception:
        return True


def main() -> int:
    tracked = list_tracked_files()
    if not tracked:
        print("check_no_secrets: Khong lay duoc danh sach git tracked files")
        return 1

    errors = []
    scanned_count = 0

    for rel_path_str in tracked:
        # Bo qua graphify-out vi la cache graph da duoc ignore/kiem soat
        if rel_path_str.startswith("graphify-out/"):
            continue

        p = pathlib.Path(rel_path_str)
        filename = p.name.lower()
        ext = p.suffix.lower()

        # 1. Kiem tra ten file cam
        if ext in FORBIDDEN_EXTENSIONS:
            errors.append(f"File duoi cam bi track vao git: {rel_path_str}")
            continue

        if any(filename.startswith(pref) for pref in FORBIDDEN_NAME_PREFIXES):
            errors.append(f"File mang ten nhay cam bi track vao git: {rel_path_str}")
            continue

        full_path = ROOT / rel_path_str
        if not full_path.is_file():
            continue

        if is_binary_file(full_path):
            continue

        # 2. Quet noi dung file text
        try:
            content = full_path.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue

        scanned_count += 1
        for rule_name, regex in PATTERNS:
            for match in regex.finditer(content):
                # Bo qua chuoi regex mau nam trong chinh script kiem tra
                if rel_path_str.endswith("check_no_secrets.py") or rel_path_str.endswith("repo-hygiene.yml"):
                    continue
                # Tranh false positive tu doc vi du co placeholder
                matched_text = match.group(0)
                errors.append(
                    f"{rel_path_str}: Phat hien {rule_name} (doan khop: {matched_text[:15]}...)"
                )

    print(f"check_no_secrets: Da quet {scanned_count} files text trong repo")
    if errors:
        print("::error::Phat hien nguy co ro ri security/private key:")
        for err in errors:
            print(f"  [CANH BAO] {err}")
        return 1

    print("check_no_secrets: OK (khong co private key hoac token nao bi ro ri)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
