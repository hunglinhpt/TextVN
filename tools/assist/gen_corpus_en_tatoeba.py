#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh corpus EN cau SACH tu Tatoeba (English grammar assistant v1).

Nguon: Tatoeba `eng_sentences.tsv.bz2` (per_language export).
License: Tatoeba sentences = **CC BY 2.0 FR** -> attribution (bat buoc) trong
header file corpus + `docs/compliance/tatoeba-attribution.md` + `REUSE.toml`
(annotation LicenseRef-Tatoeba, gionh block tieng Viet).

Modeled tren `gen_corpus_tatoeba.py` (ban VI), khac:
  - do dai 20-120 ky tu (cau Anh Tatoeba ngan hon cau dich VI),
  - loc "sach co hoc" (mechanically clean) de tap FPR khong chua loi lap quan
    ma 5 luat EN v1 phat hien duoc (double space, space truoc/sau dau, lap tu),
  - loai cau bam "bap" viet tat (a.m./U.S./Mr. ...) vi chinh sach tach cau
    theo viet tat la v2 - giu tap sach don nghia cho v1,
  - lay mau NGAU NHIEN seed co dinh (khong lay N cau dau nhu ban VI) vi pool
    EN rat lon (~180k cau).

Tinh tai lap (reproducible): cung snapshot export (cache trong
`tools/assist/.cache/`) + cung seed + cung EXCLUDE_IDS => file giong het ky tu.
Export Tatoeba lam moi hang tuan -> snapshot khac cho pool khac; snapshot da
dung de sinh file commit duoc ghi trong header file corpus.

HUMAN-IN-LOOP: script chi LOC + LAY MAU; nguoi/agent phai RA SOAT 300 cau
(loi chinh ta/ngu phap ngoai 5 luat v1 khong anh huong FPR v1 nhung phai doc
het it nhat 1 lan). Cau ra soat thay khong dat -> them ID Tatoeba vao
EXCLUDE_IDS roi chay lai (dinh danh, khong xoa tay).

Chay:
    python tools/assist/gen_corpus_en_tatoeba.py
    python tools/assist/gen_corpus_en_tatoeba.py --out corpus/assist/en_real_clean.ass --count 300
"""
from __future__ import annotations

import argparse
import bz2
import io
import random
import re
import sys
import urllib.request
from pathlib import Path

URL = "https://downloads.tatoeba.org/exports/per_language/eng/eng_sentences.tsv.bz2"
CACHE = Path(__file__).resolve().parent / ".cache" / "eng_sentences.tsv.bz2"

MIN_LEN = 20
MAX_LEN = 120
MAX_SENTENCES = 300
SEED = 20261023

# Lap tu HOP LE tieng Anh (chinh sach en_basic.ass, phan EN cua Fassist-94):
#   "that that" (menh de quan he chong), "had had" (qua khu hoan thanh),
#   "very very" (tang nghia) -> KHONG loai khoi tap sach.
INTENTIONAL_REPEATS_EN = frozenset({"that", "had", "very"})

# ID cau Tatoeba bi loai sau RA SOAT TAY (nguon su that la nguoi ra soat).
# Chi loai SAU lay mau (sample-full-pool-roi-bo) de them exclude KHONG lam
# doi ca mau -> audit truoc do van con dung.
# Ra soat lan 1 (2026-10-24, 300 cau seed 20261023):
EXCLUDE_IDS: frozenset[int] = frozenset({
    327602,    # "died of heart attack" - thieu menh tu "a" (loi that, v2 article)
    688587,    # "what quality that you possess is..." - cau sai cu phap (that thua)
    8016962,   # chua tu ngu khong phu hop de commit vao repo
    11177454,  # "fallen ... or almost fell" - sai song song (loi that, ngoai v1)
    12815708,  # "london ... the national gallery" - danh rieng viet thuong (loi that)
})

# Dau cau "manh" ma 5 luat EN v1 quan tam.
_STRONG_PUNCT = ",.;:!?"
# Viet tat tao "bap" tach cau (chinh sach v2) -> loai khoi tap sach v1.
_ABBREV_TRAP = re.compile(
    r"(?i)\b(?:[a-z]\.){2,}"                      # a.m. / i.e. / U.S.A.
    r"|\b[A-Z]{2,}\.(?:\s+[a-z])?"                # U.S. government / USA.
    r"|\b(?:Mr|Mrs|Ms|Dr|St|Jr|Sr|vs|etc|approx|Inc|Ltd|Prof|No)\."  # Mr. / etc.
)
# "i" roi rac viet thuong (loi lowercase_pronoun v2) -> khong thuoc tap sach.
_LONE_I = re.compile(r"(?<![A-Za-z'])i(?![A-Za-z'])")
# Giong engine: bat moi khoang trang truoc dau cau, ke ca dau cham dau
# chuoi "..." (ellipsis "một ..." -> space_before_punct nhu engine).
_SPACE_BEFORE_PUNCT = re.compile(r"\s+[" + re.escape(_STRONG_PUNCT) + r"]")
_MISSING_SPACE_AFTER = re.compile(r"[" + re.escape(_STRONG_PUNCT) + r"][A-Za-z]")
_ADJACENT_WORD = re.compile(r"[A-Za-z]+")


def fetch() -> bytes:
    if CACHE.is_file():
        return CACHE.read_bytes()
    CACHE.parent.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(URL, headers={"User-Agent": "textvn-audit/1.0"})
    with urllib.request.urlopen(req, timeout=300) as resp:  # noqa: S310
        data = resp.read()
    CACHE.write_bytes(data)
    return data


def snapshot_date() -> str:
    """Ngay snapshot export (ngay tai/cache) - ghi vao header de tai lap."""
    import datetime

    try:
        ts = CACHE.stat().st_mtime
    except OSError:
        ts = 0.0
    if not ts:
        return "unknown"
    return datetime.datetime.fromtimestamp(ts).strftime("%Y-%m-%d")


def is_mechanically_clean(text: str) -> bool:
    """Cau SACH theo 5 luat EN v1 (khoang trang/dau cau/lap tu/hoa dau cau)."""
    if "  " in text:
        return False                                   # double_space
    if _SPACE_BEFORE_PUNCT.search(text):
        return False                                   # space_before_punct
    if _MISSING_SPACE_AFTER.search(text):
        return False                                   # space_after_punct
    words = _ADJACENT_WORD.findall(text)
    lowered = [w.lower() for w in words]
    for prev, cur in zip(lowered, lowered[1:]):
        if prev == cur and cur not in INTENTIONAL_REPEATS_EN:
            return False                               # duplicate_word
    if _LONE_I.search(text):
        return False                                   # loi v2 nhung khong thuoc tap sach
    return True


def clean_sentence(text: str) -> str | None:
    """Loc 1 cau Tatoeba; tra ve None neu khong dat chuuan tap sach."""
    text = text.strip()
    if not (MIN_LEN <= len(text) <= MAX_LEN):
        return None
    if "\t" in text or "\n" in text:
        return None
    if "http" in text or "<" in text or ">" in text or "|" in text:
        return None
    if text.count('"') != 0:
        return None                                    # trich dan = chinh sach v2
    if not text[0].isascii() or not text[0].isupper():
        return None                                    # fragment check + that end
    if text[-1] not in ".!?":
        return None
    # Ky tu phai gan toan ASCII (cho phep cafe/naive, loai cau lan tieng khac).
    ascii_ratio = sum(1 for c in text if c.isascii()) / len(text)
    if ascii_ratio < 0.9:
        return None
    if _ABBREV_TRAP.search(text):
        return None                                    # tach cau theo viet tat = v2
    if not is_mechanically_clean(text):
        return None
    if text.isupper():
        return None
    return text


def main() -> int:
    ap = argparse.ArgumentParser(description="Sinh corpus EN sach tu Tatoeba")
    ap.add_argument("--out", default="corpus/assist/en_real_clean.ass")
    ap.add_argument("--count", type=int, default=MAX_SENTENCES)
    ap.add_argument("--seed", type=int, default=SEED)
    args = ap.parse_args()

    raw = bz2.decompress(fetch()).decode("utf-8", "replace")
    seen: set[str] = set()
    pool: list[tuple[int, str]] = []  # (tatoeba_id, sentence) theo thu tu export
    for line in io.StringIO(raw):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        try:
            sid = int(parts[0])
        except ValueError:
            continue
        text = clean_sentence(parts[2])
        if text is None or text in seen:
            continue
        seen.add(text)
        pool.append((sid, text))

    # Lay mau tren TOAN pool (co ca ID se bi loai) de mau ON DINH khi
    # EXCLUDE_IDS tang (them exclude chi bo dung cau do, khong resample).
    sample = random.Random(args.seed).sample(pool, min(args.count, len(pool)))
    sample = [(sid, s) for sid, s in sample if sid not in EXCLUDE_IDS]
    excluded_in_sample = min(args.count, len(pool)) - len(sample)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    header = [
        "# corpus/assist/en_real_clean.ass - REAL clean English sentences (EN v1 FPR set)",
        "# Nguon: Tatoeba per_language export eng_sentences.tsv.bz2 (CC BY 2.0 FR)",
        "# License du lieu: CC BY 2.0 FR (Tatoeba) - attribution BAT BUOC:",
        "#   docs/compliance/tatoeba-attribution.md + REUSE.toml (LicenseRef-Tatoeba).",
        f"# Sinh lap: python tools/assist/gen_corpus_en_tatoeba.py"
        f" (seed={args.seed}, {len(sample)}/{len(pool)} cau, snapshot {snapshot_date()})",
        "# Tinh tai lap: cung snapshot (cache tools/assist/.cache/) + seed +",
        "# EXCLUDE_IDS trong script => file giong het; snapshot moi (hang tuan) =>",
        "# pool khac - dung snapshot da ghi o tren de tai lap file nay.",
        "# HUMAN-IN-LOOP: mau da duoc ra soat ngay 2026-10-24 (agent) - sach theo",
        "# 5 luat EN v1; cau khong dat -> them ID vao EXCLUDE_IDS roi chay lai.",
        "# Dinh dang: <cau>\\t (cot kinds rong = sach, DONG CUNG -> CI assert 0 loi).",
        "# '~ ' = da ra soat la sach nhung engine DANG BAO NHAM (FPR that) -",
        "#   them khi engine EN co mat va phat hien FP tren file nay.",
        "# Chi dung de do FPR (P4-12 EN); khong phai engine tu kiem tra.",
    ]
    body = [f"{s}\t" for _, s in sample]
    # LF cung dau tren moi he dieu hanh (dinh dang corpus gionh cac file viet tay).
    with out.open("w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(header + body) + "\n")
    print(f"pool: {len(pool)} cau | excluded-by-id trong mau: {excluded_in_sample}")
    print(f"wrote {len(sample)} sentences -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
