#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Do recall cua luat `spelling` (dau thanh) bang LOI TONG HOP (P4-13).

Lay cau sach tu `corpus/assist/vi_real_clean.ass`, doi dau thanh cua MOT tu
(>= 3 ky tu, co nguyen am) sao cho ket qua KHONG la am tiet hop le trong
`data/vn_syllables.txt`, roi chay engine va dem so cau co issue de len span
tu bi sua. Day la **can tren** (loi nguoi that da dang hon; tu < 3 ky tu bi bo
qua) nhung du de so cau hinh voi nhau: `--no-dict` vs mac dinh (tu dien dung san).

Chay:
    python tools/assist/tone_inject.py                 # mac dinh = tu dien dung san
    python tools/assist/tone_inject.py --no-dict       # doi chung (luat cu)
    python tools/assist/tone_inject.py --seed 7 --max 2000
CHAY LAI `cargo build -p textvn-cli` TRUOC KHI DO.
"""
from __future__ import annotations

import argparse
import json
import os
import random
import subprocess
import sys
import tempfile
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLEAN = ROOT / "corpus" / "assist" / "vi_real_clean.ass"
SYL = ROOT / "data" / "vn_syllables.txt"

ROWS = [
    "aàáảãạ", "ăằắẳẵặ", "âầấẩẫậ", "eèéẻẽẹ", "êềếểễệ", "iìíỉĩị",
    "oòóỏõọ", "ôồốổỗộ", "ơờớởỡợ", "uùúủũụ", "ưừứửữự", "yỳýỷỹỵ",
]
BASE = {c: row[0] for row in ROWS for c in row}
TONE = {c: i for row in ROWS for i, c in enumerate(row)}


def strip_tone(word: str) -> tuple[str, int, int]:
    key, tone, marks = [], 0, 0
    for c in word:
        if c in BASE:
            key.append(BASE[c])
            if TONE[c]:
                tone, marks = TONE[c], marks + 1
        else:
            key.append(c)
    return "".join(key), tone, marks


def load_valid() -> set[tuple[str, int]]:
    valid = set()
    for line in SYL.read_text(encoding="utf-8").splitlines():
        line = line.strip().lower()
        if not line or line.startswith("#"):
            continue
        key, tone, marks = strip_tone(line)
        if marks <= 1:
            valid.add((key, tone))
    return valid


def with_tone(word: str, tone: int) -> str | None:
    """Dat dau `tone` len nguyen am cuoi cua word (da bo dau thanh cu)."""
    chars = list(word)
    idx = [i for i, c in enumerate(chars) if c in BASE]
    if not idx:
        return None
    for i in idx:
        chars[i] = ROWS[[r[0] for r in ROWS].index(BASE[chars[i]])][0]
    j = idx[-1]
    row = next(r for r in ROWS if r[0] == chars[j])
    chars[j] = row[tone]
    return "".join(chars)


def inject(sentence: str, valid: set, rng: random.Random):
    words = [(m.start(), m.group()) for m in __import__("re").finditer(r"[^\W\d_]+", sentence)]
    rng.shuffle(words)
    for start, w in words:
        if len(w) < 3 or w != w.lower() or unicodedata.normalize("NFC", w) != w:
            continue
        key, tone, marks = strip_tone(w)
        if not any(c in BASE for c in key) or (key, 0) not in valid and not any(
            (key, t) in valid for t in range(6)
        ):
            continue  # khoa la (khong co trong du lieu) → khong co loi "dau thanh" ro rang
        bad = [t for t in range(6) if t != tone and (key, t) not in valid]
        bad = [t for t in bad if with_tone(key, t)]
        if not bad:
            continue
        t = rng.choice(bad)
        wrong = with_tone(key, t)
        # giu nguyen cac nguyen am co diacritic co so (ă â ê ô ơ ư) – `key` da giu
        if wrong is None or wrong == w:
            continue
        return sentence[:start] + wrong + sentence[start + len(w):], start, start + len(wrong)
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default=None)
    ap.add_argument("--seed", type=int, default=20261108)
    ap.add_argument("--max", type=int, default=1000)
    ap.add_argument("--no-dict", action="store_true")
    args = ap.parse_args()

    cli = args.cli
    if not cli:
        for rel in ("target/debug/textvn-cli.exe", "target/debug/textvn-cli",
                    "target/release/textvn-cli.exe", "target/release/textvn-cli"):
            if (ROOT / rel).is_file():
                cli = str(ROOT / rel)
                break
    if not cli:
        raise SystemExit("ERROR: khong thay textvn-cli (cargo build -p textvn-cli)")

    valid = load_valid()
    rng = random.Random(args.seed)
    clean = [
        l.split("\t")[0].lstrip("~").strip()
        for l in CLEAN.read_text(encoding="utf-8").splitlines()
        if l.strip() and not l.lstrip().startswith("#")
    ]
    cases = []
    for s in clean:
        r = inject(s, valid, rng)
        if r:
            cases.append(r)
        if len(cases) >= args.max:
            break

    fd, tmp = tempfile.mkstemp(suffix=".txt")
    with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(c[0] for c in cases) + "\n")
    cmd = [cli, "assist", "check", "--file", tmp, "--json"]
    if args.no_dict:
        cmd.append("--no-dict")
    proc = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    os.unlink(tmp)
    issues = json.loads(proc.stdout or "[]")
    by_line: dict[int, list[dict]] = {}
    for it in issues:
        by_line.setdefault(int(it["line"]), []).append(it)

    hit = hit_exact = 0
    for i, (sent, a, b) in enumerate(cases, 1):
        found = by_line.get(i, [])
        if any(it.get("kind") in ("spelling", "diacritic_typo") for it in found):
            hit += 1
    total = len(cases)
    mode = "no-dict (luat cu)" if args.no_dict else "tu dien dung san"
    print(f"cau tong hop: {total}  | cau hinh: {mode}  | seed {args.seed}")
    print(f"phat hien (spelling|diacritic_typo): {hit}/{total} = {100 * hit / max(total, 1):.1f} %")
    return 0


if __name__ == "__main__":
    sys.exit(main())
