#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Do **recall** + **FPR** tren van ban THAT co nhan (P4-12, D19).

Input (commit trong repo, UTF-8 khong BOM, tach bang TAB that):
  corpus/assist/vi_real_errors.ass  -> "<cau>\\t<kind1,kind2>"  (cau co loi that, co nhan)
  corpus/assist/vi_real_clean.ass   -> "<cau>\\t"                (cau sach da kiem)

Tien to `~ ` = **dong mem** (nghia dung nhu test Rust `assist/tests/corpus.rs`):
  - `~ <cau>\\t<kind>`  trong file LOI   = loi that ma **engine bo sot** -> chi do recall.
  - `~ <cau>\\t`        trong file SACH  = cau sach nhung **engine bao nham** -> do FPR that.
  Dong cung (khong co `~`) = **ky vung cung** -> CI assert cung (test Rust lo cac dong nay).
Script nay bo `~` truoc khi kiem (giong test Rust) va bao cao **split hard/soft**.

Engine: `textvn-cli assist check --file <tmp> --json` (batch, offline, khong I/O mang).
CHAY LAI `cargo build -p textvn-cli` TRUOC KHI DO (binary cu = so do sai - Fassist-120).

Chay:
    python tools/assist/score_real.py
    python tools/assist/score_real.py --cli target/release/textvn-cli.exe
    python tools/assist/score_real.py --mode both      # so sanh check vs --sentence
    python tools/assist/score_real.py --verify          # kiem chung tap nhan (exit != 0 neu fail)
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ERRORS = ROOT / "corpus" / "assist" / "vi_real_errors.ass"
CLEAN = ROOT / "corpus" / "assist" / "vi_real_clean.ass"

CLI_CANDIDATES = (
    # DEV truoc: binary release co the **cu** (doi engine xong quen rebuild) =>
    # do sai ma khong ai biet (Fassist-120). Debug build luon moi hon.
    "target/debug/textvn-cli.exe",
    "target/debug/textvn-cli",
    "target/release/textvn-cli.exe",
    "target/release/textvn-cli",
)

# Thu muc nguon dung de canh bao binary cu.
SOURCE_DIRS = (
    "assist/src",
    "cli/src",
    "core/src",
    "ffi/src",
    "dict/src",
    "config/src",
)


def newest_source_mtime() -> float:
    """mtime moi nhat cua source engine/CLI (0 neu khong tim thay)."""
    newest = 0.0
    for rel in SOURCE_DIRS:
        base = ROOT / rel
        if not base.is_dir():
            continue
        for path in base.rglob("*.rs"):
            try:
                newest = max(newest, path.stat().st_mtime)
            except OSError:
                continue
    return newest



class Row:
    """1 dong corpus: cau (da bo `~`), nhan loai loi, co phai dong mem hay khong."""

    __slots__ = ("sentence", "kinds", "soft")

    def __init__(self, sentence: str, kinds: list[str], soft: bool) -> None:
        self.sentence = sentence
        self.kinds = kinds
        self.soft = soft


def find_cli(explicit: str | None) -> Path:
    """Tra ve duong dan CLI; neu explicit thi dung luon (khong ton tai -> loi)."""
    if explicit:
        p = Path(explicit)
        if not p.is_file():
            raise SystemExit(f"ERROR: CLI khong ton tai: {p}")
        return p
    for rel in CLI_CANDIDATES:
        p = ROOT / rel
        if p.is_file():
            warn_if_stale(p)
            return p
    for name in ("textvn-cli.exe", "textvn-cli"):
        for d in os.environ.get("PATH", "").split(os.pathsep):
            cand = Path(d) / name
            if cand.is_file():
                warn_if_stale(cand)
                return cand
    raise SystemExit(
        "ERROR: khong tim thay textvn-cli. Hay build: cargo build -p textvn-cli -j 2\n"
        "       (hoac truyen --cli <duong dan>)"
    )


def warn_if_stale(cli: Path) -> None:
    """Canh bao neu binary CU hon source (do sai - Fassist-120)."""
    try:
        bin_mtime = cli.stat().st_mtime
    except OSError:
        return
    src_mtime = newest_source_mtime()
    if src_mtime and bin_mtime < src_mtime:
        print(
            f"WARNING: binary CLI CU hon source ({cli}): "
            f"binary={_stamp(bin_mtime)} < source={_stamp(src_mtime)}\n"
            f"         -> ket qua do co the SAI. Hay chay: cargo build -p textvn-cli -j 2",
            file=sys.stderr,
        )


def _stamp(ts: float) -> str:
    import datetime

    return datetime.datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S")



def read_labelled(path: Path) -> list[Row]:
    """Doc file corpus: '<cau>\\t<kind1,kind2>' -> [Row] (bo `~ ` nhu test Rust)."""
    if not path.is_file():
        raise SystemExit(f"ERROR: thieu file corpus: {path}")
    out: list[Row] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        sentence, _, label = raw.partition("\t")
        soft = sentence.startswith("~")
        sentence = sentence.lstrip("~").strip()
        kinds = [k.strip() for k in label.split(",") if k.strip()]
        out.append(Row(sentence, kinds, soft))
    return out


def run_engine(cli: Path, sentences: list[str], sentence_mode: bool) -> list[set[str]]:
    """Chay engine 1 lan cho ca danh sach -> [set(kind)] theo thu tu cau."""
    if not sentences:
        return []
    fd, tmp_name = tempfile.mkstemp(prefix="textvn_assist_", suffix=".txt")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as fh:
            fh.write("\n".join(sentences) + "\n")
        cmd = [str(cli), "assist", "check", "--file", tmp_name, "--json"]
        if sentence_mode:
            cmd.append("--sentence")
        proc = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
        if not proc.stdout.strip():
            raise SystemExit(
                f"ERROR: CLI khong tra JSON (rc={proc.returncode}): {proc.stderr.strip()[:400]}"
            )
        try:
            issues = json.loads(proc.stdout)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"ERROR: JSON khong hop le: {exc}") from exc
    finally:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass

    by_line: dict[int, set[str]] = {}
    for it in issues:
        line = int(it.get("line", 0))
        by_line.setdefault(line, set()).add(str(it.get("kind", "")))
    return [by_line.get(i + 1, set()) for i in range(len(sentences))]


def wilson(successes: int, total: int) -> tuple[float, float]:
    """Khoang tin cay Wilson 95% (khong phu thuoc scipy)."""
    if total == 0:
        return (0.0, 0.0)
    z = 1.959964
    p = successes / total
    denom = 1 + z * z / total
    centre = (p + z * z / (2 * total)) / denom
    margin = z * ((p * (1 - p) / total + z * z / (4 * total * total)) ** 0.5) / denom
    return (max(0.0, centre - margin), min(1.0, centre + margin))


def measure(cli: Path, errors: list[Row], clean: list[Row], sentence_mode: bool) -> dict:
    """Do recall (tap nhan) + FPR (tap sach, tach hard/soft) -> dict."""
    det_err = run_engine(cli, [r.sentence for r in errors], sentence_mode)
    det_clean = run_engine(cli, [r.sentence for r in clean], sentence_mode)

    expected = Counter()
    detected = Counter()
    hit = 0
    for row, found in zip(errors, det_err):
        for k in row.kinds:
            expected[k] += 1
            if k in found:
                detected[k] += 1
        if any(k in found for k in row.kinds):
            hit += 1

    hard_clean_flagged = 0
    soft_clean_flagged = 0
    fp_kinds = Counter()
    for row, found in zip(clean, det_clean):
        if not found:
            continue
        if row.soft:
            soft_clean_flagged += 1
        else:
            hard_clean_flagged += 1
        for k in found:
            fp_kinds[k] += 1

    total_err = len(errors)
    total_clean = len(clean)
    recall = hit / total_err if total_err else 0.0
    fpr_all = (hard_clean_flagged + soft_clean_flagged) / total_clean if total_clean else 0.0
    r_lo, r_hi = wilson(hit, total_err)
    f_lo, f_hi = wilson(hard_clean_flagged + soft_clean_flagged, total_clean)
    return {
        "errors": total_err,
        "detected_errors": hit,
        "recall": recall,
        "recall_lo": r_lo,
        "recall_hi": r_hi,
        "clean": total_clean,
        "flagged_clean_hard": hard_clean_flagged,
        "flagged_clean_soft": soft_clean_flagged,
        "fpr": fpr_all,
        "fpr_lo": f_lo,
        "fpr_hi": f_hi,
        "per_kind_expected": dict(expected),
        "per_kind_detected": dict(detected),
        "fp_kinds": dict(fp_kinds),
    }


def report(mode: str, m: dict) -> None:
    print(f"=== mode: {mode} ===")
    print(f"Cau co loi (co nhan)   : {m['errors']}")
    print(f"  engine phat hien     : {m['detected_errors']}   recall = {m['recall'] * 100:.1f} %"
          f"   (Wilson 95%: {m['recall_lo'] * 100:.1f} - {m['recall_hi'] * 100:.1f} %)")
    print(f"Cau sach               : {m['clean']}")
    print(f"  bi gan co (FPR)      : {m['flagged_clean_hard'] + m['flagged_clean_soft']}"
          f"   FPR   = {m['fpr'] * 100:.1f} %"
          f"   (Wilson 95%: {m['fpr_lo'] * 100:.1f} - {m['fpr_hi'] * 100:.1f} %)")
    print(f"    trong do dong mem `~` (sach nhung engine bao nham): {m['flagged_clean_soft']}"
          f" | dong cung (hoi quy FP): {m['flagged_clean_hard']}")
    print("Recall theo loai loi:")
    for kind in sorted(m["per_kind_expected"], key=lambda k: (-m["per_kind_expected"][k], k)):
        exp = m["per_kind_expected"][kind]
        det = m["per_kind_detected"][kind]
        pct = (det / exp * 100) if exp else 0.0
        print(f"  {kind:<26} {det:>4}/{exp:<4} = {pct:>6.1f} %")
    if m["fp_kinds"]:
        print("Loai loi tren tap sach (nguon FPR):")
        for kind, n in sorted(m["fp_kinds"].items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"  {kind:<26} {n:>4}")


def verify(cli: Path, errors: list[Row], clean: list[Row]) -> int:
    """Kiem chung tap nhan: dong CUNG phai khop engine; dong MEM chi bao cao.

    Dong cung trong file LOI: engine phai bao DUNG cac loai loi da nhan.
    Dong cung trong file SACH: engine phai bao 0 issue (hoi quy FP = 0).
    """
    det_err = run_engine(cli, [r.sentence for r in errors], sentence_mode=False)
    det_clean = run_engine(cli, [r.sentence for r in clean], sentence_mode=False)
    failures: list[str] = []
    for row, found in zip(errors, det_err):
        if row.soft:
            continue
        got = sorted(found)
        want = sorted(row.kinds)
        if got != want:
            failures.append(f"LOI: cau `{row.sentence}` -> engine {got}, nhan {want}")
    for row, found in zip(clean, det_clean):
        if row.soft:
            continue
        if found:
            failures.append(f"LOI: cau sach `{row.sentence}` -> engine bao {sorted(found)}")
    if failures:
        print(f"verify: FAIL ({len(failures)} dong)")
        for f in failures[:20]:
            print(f"  {f}")
        return 1
    print("verify: OK")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description="Do recall/FPR tren corpus van ban that co nhan")
    ap.add_argument("--cli", help="duong dan textvn-cli (mac dinh: tu dong tim)")
    ap.add_argument("--mode", choices=["check", "sentence", "both"], default="check",
                    help="check = mac dinh; sentence = --sentence (coi la cau tron ven)")
    ap.add_argument("--verify", action="store_true",
                    help="kiem chung tap nhan (dong cung phai khop engine) -> exit != 0 neu fail")
    ap.add_argument("--json", action="store_true", help="in ket qua dang JSON")
    args = ap.parse_args()

    cli = find_cli(args.cli)
    errors = read_labelled(ERRORS)
    clean = read_labelled(CLEAN)
    if not errors:
        raise SystemExit("ERROR: tap co nhan rong")

    if args.verify:
        return verify(cli, errors, clean)

    modes = ["check", "sentence"] if args.mode == "both" else [args.mode]
    results = []
    for m in modes:
        res = measure(cli, errors, clean, sentence_mode=(m == "sentence"))
        res["mode"] = m
        report(m, res)
        results.append(res)
        print()
    if args.json:
        print(json.dumps(results, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
