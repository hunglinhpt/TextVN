#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""check-pe-imports.py — chặn binary Windows phụ thuộc VC++ runtime (R2-18).

TIP DLL được nạp vào MỌI tiến trình (kể cả app 32-bit), tray/CLI chạy từ bản
portable/MSIX trên máy có thể thiếu VC++ redist → phải link CRT tĩnh
(.cargo/config.toml `+crt-static`). Script đọc bảng import của từng PE (không cần
dumpbin/Visual Studio) và fail nếu thấy VCRUNTIME*/MSVCP*/ucrtbase*/api-ms-win-crt-*.

  python tools/win/check-pe-imports.py <file.exe|file.dll> [...]
"""

from __future__ import annotations

import re
import struct
import sys

FORBIDDEN = re.compile(r"^(vcruntime\d+.*|msvcp\d+.*|ucrtbase.*|api-ms-win-crt-.*)\.dll$", re.IGNORECASE)


def imports(path: str) -> list[str]:
    data = open(path, "rb").read()
    if data[:2] != b"MZ":
        raise ValueError("không phải PE (thiếu MZ)")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("không phải PE (thiếu chữ ký PE)")
    nsec = struct.unpack_from("<H", data, pe + 6)[0]
    opt_size = struct.unpack_from("<H", data, pe + 20)[0]
    opt = pe + 24
    magic = struct.unpack_from("<H", data, opt)[0]
    dd = opt + (96 if magic == 0x10B else 112)  # PE32 / PE32+
    imp_rva, _imp_size = struct.unpack_from("<II", data, dd + 8)  # data directory [1]
    sections = []
    sec = opt + opt_size
    for i in range(nsec):
        off = sec + 40 * i
        vsize, va, rawsize, rawptr = struct.unpack_from("<IIII", data, off + 8)
        sections.append((va, max(vsize, rawsize), rawptr))

    def rva_to_off(rva: int) -> int:
        for va, size, raw in sections:
            if va <= rva < va + size:
                return raw + (rva - va)
        raise ValueError(f"RVA {rva:#x} ngoài mọi section")

    names: list[str] = []
    if imp_rva == 0:
        return names
    off = rva_to_off(imp_rva)
    while True:
        orig, _ts, _fwd, name_rva, first = struct.unpack_from("<IIIII", data, off)
        if orig == 0 and name_rva == 0 and first == 0:
            break
        noff = rva_to_off(name_rva)
        end = data.index(b"\0", noff)
        names.append(data[noff:end].decode("ascii", "replace"))
        off += 20
    return names


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2
    bad = 0
    for path in argv:
        try:
            dlls = imports(path)
        except (OSError, ValueError, struct.error) as e:
            print(f"FAIL {path}: {e}")
            bad += 1
            continue
        hits = [d for d in dlls if FORBIDDEN.match(d)]
        if hits:
            print(f"FAIL {path}: phụ thuộc VC++ runtime {', '.join(hits)} — cần +crt-static")
            bad += 1
        else:
            print(f"OK   {path}: {len(dlls)} DLL import, không có VC++ runtime")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
