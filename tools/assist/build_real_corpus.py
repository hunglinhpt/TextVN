#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh **ung vien** cho tap corpus van ban THAT co nhan (P4-12) + kiem chung tap nhan.

Tach 2 thanh:
  1. SINH UNG VIEN (script nay, tu dong): pool cau Tatoeba (hoac file local)
     -> lay mau ngau nhien (seed co dinh) -> chay engine -> tach cau bi gan co / cau sach.
  2. **RA SOAT TAY** (con nguoi/agent): doc tung cau ung vien va **gan nhan loai loi
     that** -> moi tao duoc `corpus/assist/vi_real_errors.ass` / `vi_real_clean.ass`.
     Script nay **KHONG** gan nhan (nguon su that la nguoi, khong phai may).

Ung vien ghi vao `tools/assist/.out/` (git-ignored) - KHONG ghi de file da commit.

Chay:
    python tools/assist/build_real_corpus.py --pool corpus/assist/vi_tatoeba_clean.ass
    python tools/assist/build_real_corpus.py                # tai Tatoeba (can mang)
    python tools/assist/build_real_corpus.py --verify       # kiem chung tap nhan da commit
"""
from __future__ import annotations

import argparse
import bz2
import importlib.util
import io
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT_DIR = HERE / ".out"
SEED = 20261021
SAMPLE = 1000

sys.path.insert(0, str(HERE))
import score_real  # noqa: E402 - nap dong de dung lai tim CLI + doc corpus


def load_pool(use_local: Path | None) -> list[str]:
    """Pool cau da loc (unique, thu tu goc)."""
    if use_local is not None:
        if not use_local.is_file():
            raise SystemExit(f"ERROR: khong thay file pool: {use_local}")
        pool: list[str] = []
        seen: set[str] = set()
        for raw in use_local.read_text(encoding="utf-8").splitlines():
            if not raw.strip() or raw.lstrip().startswith("#"):
                continue
            sentence = raw.split("\t", 1)[0].strip()
            if sentence.startswith("~ "):
                sentence = sentence[2:].strip()
            if sentence and sentence not in seen:
                seen.add(sentence)
                pool.append(sentence)
        return pool
    spec = importlib.util.spec_from_file_location("gen_corpus_tatoeba", HERE / "gen_corpus_tatoeba.py")
    if spec is None or spec.loader is None:
        raise SystemExit("ERROR: khong nap duoc tools/assist/gen_corpus_tatoeba.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    raw = bz2.decompress(mod.fetch()).decode("utf-8", "replace")
    pool: list[str] = []
    seen = set()
    for line in io.StringIO(raw):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        text = mod.clean_sentence(parts[2])
        if text is None or text in seen:
            continue
        seen.add(text)
        pool.append(text)
    return pool


def main() -> int:
    ap = argparse.ArgumentParser(description="Sinh ung vien corpus van ban that co nhan")
    ap.add_argument("--pool", type=Path, default=None,
                    help="dung pool cau local (mac dinh: tai Tatoeba qua mang)")
    ap.add_argument("--seed", type=int, default=SEED)
    ap.add_argument("--sample", type=int, default=SAMPLE)
    ap.add_argument("--cli", default=None, help="duong dan textvn-cli")
    ap.add_argument("--scan-all", action="store_true",
                    help="quet TOAN BO pool de lay ung vien co loi (khong chi trong mau)")
    ap.add_argument("--verify", action="store_true",
                    help="kiem chung tap nhan da commit (dong cung phai khop engine)")
    args = ap.parse_args()

    cli = score_real.find_cli(args.cli)

    if args.verify:
        errors = score_real.read_labelled(score_real.ERRORS)
        clean = score_real.read_labelled(score_real.CLEAN)
        return score_real.verify(cli, errors, clean)

    try:
        pool = load_pool(args.pool)
    except Exception as exc:  # noqa: BLE001 - bao loi ro rang cho nguoi dung
        raise SystemExit(
            f"ERROR: khong tai duoc pool cau Tatoeba ({exc}).\n"
            "       Hay truyen --pool <file .ass> de dung pool local."
        ) from exc
    print(f"pool: {len(pool)} cau")

    sample = random.Random(args.seed).sample(pool, min(args.sample, len(pool)))
    print(f"mau: {len(sample)} cau (seed={args.seed})")

    header = [
        "# UNG VIEN (can RA SOAT TAY truoc khi gan nhan) - sinh boi build_real_corpus.py",
        f"# seed={args.seed} sample={len(sample)} pool={len(pool)}",
        "# Nhan cua engine chi la 'ung vien' - nguon su that la nguoi ra soat.",
    ]
    err_lines: list[str] = []
    clean_lines: list[str] = []
    if args.scan_all:
        # Phuong phap goc: cau co loi = TAT CA cau bi engine gan co trong toan bo pool;
        # cau sach = mau ngau nhien (tru cau bi gan co).
        scanned = score_real.run_engine(cli, pool, sentence_mode=False)
        for sentence, found in zip(pool, scanned):
            if found:
                err_lines.append(f"{sentence}\t{','.join(sorted(found))}")
        flagged = {s for s, f in zip(pool, scanned) if f}
        for sentence in sample:
            if sentence not in flagged:
                clean_lines.append(f"{sentence}\t")
    else:
        detected = score_real.run_engine(cli, sample, sentence_mode=False)
        for sentence, found in zip(sample, detected):
            if found:
                err_lines.append(f"{sentence}\t{','.join(sorted(found))}")
            else:
                clean_lines.append(f"{sentence}\t")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    err_path = OUT_DIR / "vi_real_errors.candidates.ass"
    clean_path = OUT_DIR / "vi_real_clean.candidates.ass"
    err_path.write_text("\n".join(header + err_lines) + "\n", encoding="utf-8")
    clean_path.write_text("\n".join(header + clean_lines) + "\n", encoding="utf-8")
    print(f"ung vien co loi  -> {err_path} ({len(err_lines)} cau)")
    print(f"ung vien sach    -> {clean_path} ({len(clean_lines)} cau)")
    print("Buoc tiep theo: RA SOAT TAY tung cau ung vien, gan nhan loai loi that,")
    print("roi moi tao duoc corpus/assist/vi_real_errors.ass + vi_real_clean.ass da commit.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
