#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh corpus CAU SACH tu Tatoeba (cau tieng Viet tu nhien) - P4-0 D19.

Nguon: Tatoeba `vie_sentences.tsv.bz2` (per_language export).
License: Tatoeba sentences = **CC BY 2.0 FR** -> can attribution (bat buoc) trong
header cua file corpus + `docs/compliance/`.

Vi sao: do FPR tren **van ban that**, cau ngan tu nhien (khong phai cau mau sinh
tu template) - day la thuoc do FPR day du nhat (Fassist-80).

Chay:
    python tools/assist/gen_corpus_tatoeba.py
    python tools/assist/gen_corpus_tatoeba.py --out corpus/assist/vi_tatoeba_clean.ass
"""
from __future__ import annotations

import argparse
import bz2
import io
import sys
import urllib.request
from pathlib import Path

URL = "https://downloads.tatoeba.org/exports/per_language/vie/vie_sentences.tsv.bz2"
CACHE = Path(__file__).resolve().parent / ".cache" / "vie_sentences.tsv.bz2"

# Ky tu co dau tieng Viet (ngoai ASCII) - dung de loc cau "tieng Viet thật".
VN_CHARS = set(
    "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứừửữựỳýỷỹỵđ"
    "ÀÁẢÃẠĂẰẮẲẴẶÂẦẤẨẪẬÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴĐ"
)
MIN_LEN = 25
MAX_LEN = 200
MAX_SENTENCES = 1500


def fetch() -> bytes:
    if CACHE.is_file():
        return CACHE.read_bytes()
    CACHE.parent.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(URL, headers={"User-Agent": "textvn-audit/1.0"})
    with urllib.request.urlopen(req, timeout=120) as resp:  # noqa: S310
        data = resp.read()
    CACHE.write_bytes(data)
    return data


def is_vietnamese_heavy(text: str) -> bool:
    letters = [c for c in text if c.isalpha()]
    if not letters:
        return False
    marked = sum(1 for c in letters if c in VN_CHARS)
    return marked > 0 and marked * 100 >= len(letters) * 35


def clean_sentence(text: str) -> str | None:
    text = text.strip()
    if not (MIN_LEN <= len(text) <= MAX_LEN):
        return None
    if "http" in text or "<" in text or ">" in text or "|" in text:
        return None
    if text.count('"') % 2 != 0:
        return None
    if not is_vietnamese_heavy(text):
        return None
    # Bo dau nhay bao (dau nhay la dau cau tieng Viet, khong phai dau trich dan)
    return text


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="corpus/assist/vi_tatoeba_clean.ass")
    args = ap.parse_args()

    raw = bz2.decompress(fetch()).decode("utf-8", "replace")
    seen: set[str] = set()
    sentences: list[str] = []
    for line in io.StringIO(raw):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        text = clean_sentence(parts[2])
        if text is None or text in seen:
            continue
        seen.add(text)
        sentences.append(text)
        if len(sentences) >= MAX_SENTENCES:
            break

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    header = [
        "# corpus/assist/vi_tatoeba_clean.ass - cau SACH tu Tatoeba (van ban that)",
        "# Nguon: Tatoeba per-language export vie_sentences.tsv.bz2",
        "# License du lieu: CC BY 2.0 FR (Tatoeba) - attribution bat buoc (xem docs/compliance/)",
        f"# Sinh lap: python tools/assist/gen_corpus_tatoeba.py ({len(sentences)} cau)",
        "# Dung de do FPR tren cau ngan tu nhien (Fassist-80).",
        "# Tien to '~ ' = cau SACH *co the chua loi that* (van ban nguoi that) ->",
        "#   test KHONG assert 'khong loi' cho cac cau nay; `assist score` van dem",
        "#   chung vao mau so cua FPR va bao rieng so cau bi gan co (ung vien).",
    ]
    body = [f"~ {s}\t" for s in sentences]
    out.write_text("\n".join(header + body) + "\n", encoding="utf-8")
    print(f"wrote {len(sentences)} sentences -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
