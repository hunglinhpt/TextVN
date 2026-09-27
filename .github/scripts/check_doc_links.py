#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check relative markdown links trong docs/ + *.md goc.

Chay local:  python .github/scripts/check_doc_links.py
Dung trong:  .github/workflows/repo-hygiene.yml (check #6)
Quy uoc:     00-WORKFLOW.md §12 — link hong = CI do, fix truoc khi nhan task moi.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
LINK_RE = re.compile(r"\]\(([^)\s]+)\)")
SKIP_PREFIX = ("http://", "https://", "#", "mailto:", "tel:")


def main() -> int:
    files = [ROOT / "PLAN.md", ROOT / "README.md"] + sorted(ROOT.glob("docs/**/*.md"))
    checked = 0
    broken = []
    scanned = 0
    for md in files:
        if not md.is_file():
            continue
        scanned += 1
        text = md.read_text(encoding="utf-8", errors="replace")
        for match in LINK_RE.finditer(text):
            target = match.group(1)
            if target.startswith(SKIP_PREFIX):
                continue
            path = target.split("#", 1)[0]
            if not path:
                continue
            checked += 1
            if not (md.parent / path).exists():
                broken.append(f"{md.relative_to(ROOT)}: {target}")
    print(f"check_doc_links: {checked} relative link trong {scanned} file md")
    if broken:
        print("BROKEN relative links:")
        for line in broken:
            print(f"  {line}")
        return 1
    print("check_doc_links: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
