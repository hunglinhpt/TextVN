#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Cohen's kappa giua hai nguoi gan nhan (P4-12c, doc P4-12 §8).

Moi file nhan la TSV UTF-8, mot dong mot cau, **cung thu tu va cung so dong**:

    <cau>\\t<kind1,kind2>      (cot 2 rong = nguoi gan cho la cau sach)

Bao cao:
  * kappa cho nhan nhi phan "cau co loi?" (co / khong);
  * kappa cho tung loai loi (co kind do hay khong);
  * danh sach dong bat dong de 2 nguoi thao luan truoc khi chot nhan.

Chay:
    python tools/assist/kappa.py annotator_a.tsv annotator_b.tsv
    python tools/assist/kappa.py --self-test
Ngung >= 0.8 moi coi la dang tin cay (Landis & Koch: 0.61-0.80 = kha, > 0.80 = rat tot).
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path


def read(path: Path) -> list[tuple[str, frozenset[str]]]:
    rows = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw.strip() or raw.startswith("#"):
            continue
        sentence, _, label = raw.partition("\t")
        kinds = frozenset(k.strip() for k in label.split(",") if k.strip())
        rows.append((sentence.strip(), kinds))
    return rows


def kappa(a: list[bool], b: list[bool]) -> float:
    """Cohen's kappa nhi phan. 1.0 neu hai nguoi hoan toan trung; 0 = ngang may rui."""
    n = len(a)
    if n == 0 or n != len(b):
        raise ValueError("hai danh sach nhan phai cung do dai va khac rong")
    observed = sum(x == y for x, y in zip(a, b)) / n
    pa, pb = sum(a) / n, sum(b) / n
    expected = pa * pb + (1 - pa) * (1 - pb)
    if expected == 1.0:  # ca hai luon cung mot nhan -> khong co phuong sai de do
        return 1.0 if observed == 1.0 else 0.0
    return (observed - expected) / (1 - expected)


def report(a_rows, b_rows) -> int:
    if len(a_rows) != len(b_rows):
        print(f"ERROR: so dong khac nhau ({len(a_rows)} vs {len(b_rows)})", file=sys.stderr)
        return 2
    for i, ((sa, _), (sb, _)) in enumerate(zip(a_rows, b_rows), 1):
        if sa != sb:
            print(f"ERROR: dong {i} khac cau giua hai file", file=sys.stderr)
            return 2
    has_a = [bool(k) for _, k in a_rows]
    has_b = [bool(k) for _, k in b_rows]
    print(f"So cau: {len(a_rows)}")
    print(f"kappa 'cau co loi?'      : {kappa(has_a, has_b):.3f}")
    kinds = sorted({k for _, ks in a_rows + b_rows for k in ks})
    for kind in kinds:
        ka = [kind in ks for _, ks in a_rows]
        kb = [kind in ks for _, ks in b_rows]
        print(f"  {kind:<28} kappa {kappa(ka, kb):.3f}   (A={sum(ka)} B={sum(kb)})")
    diff = [i for i, ((_, x), (_, y)) in enumerate(zip(a_rows, b_rows), 1) if x != y]
    print(f"Dong bat dong: {len(diff)}" + (f" -> {diff[:30]}" if diff else ""))
    return 0


def self_test() -> int:
    assert kappa([True, False, True, False], [True, False, True, False]) == 1.0
    assert abs(kappa([True, True, False, False], [True, False, True, False])) < 1e-9  # ngang may rui
    assert kappa([True, False], [False, True]) == -1.0
    assert kappa([False, False], [False, False]) == 1.0
    k = kappa([True] * 8 + [False] * 12, [True] * 6 + [False] * 14)
    assert 0.5 < k < 1.0, k
    try:
        kappa([], [])
    except ValueError:
        pass
    else:
        raise AssertionError("rong phai bao loi")
    print("kappa self-test: OK")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description="Cohen's kappa giua hai file nhan")
    ap.add_argument("a", nargs="?", type=Path)
    ap.add_argument("b", nargs="?", type=Path)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not args.a or not args.b:
        ap.error("can 2 file nhan (hoac --self-test)")
    return report(read(args.a), read(args.b))


if __name__ == "__main__":
    sys.exit(main())
