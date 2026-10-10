#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""verify-msix.py — kiểm gói .msix TRƯỚC khi nộp Partner Center (R2-09).

makeappx chỉ kiểm XSD: gói 0.2.27 hợp lệ XSD nhưng Partner Center sẽ từ chối
(Identity Version phần đầu = 0, identity placeholder) và mô tả tiếng Việt bị
mojibake. Script này mở chính file .msix và kiểm những luật Store + payload:

  python tools/win/verify-msix.py dist/TextVN-0.2.27-windows-x64.msix \
      --expect-version 0.2.27 [--require-store-identity]

Exit 0 = đạt; 1 = có lỗi (in từng dòng `FAIL ...`). Không phụ thuộc gói ngoài.
"""

from __future__ import annotations

import argparse
import os
import re
import struct
import sys
import xml.etree.ElementTree as ET
import zipfile

NS = {
    "f": "http://schemas.microsoft.com/appx/manifest/foundation/windows10",
    "uap": "http://schemas.microsoft.com/appx/manifest/uap/windows10",
    "rescap": "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities",
}

# Giá trị Partner Center của chủ tài khoản (docs/release/msix-submission.md).
PLACEHOLDER_NAME = "LinhBH.CoM.TextVN"
EXPECTED_PUBLISHER = "CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545"
EXPECTED_DISPLAY_NAME = "TextVN"
EXPECTED_PUBLISHER_DISPLAY = "LinhBH.CoM"

# Ảnh logo: thuộc tính manifest → kích thước bắt buộc (scale-100).
ASSET_SIZES = {
    "Square150x150Logo": (150, 150),
    "Square44x44Logo": (44, 44),
    "Wide310x150Logo": (310, 150),
    "Square71x71Logo": (71, 71),
}
STORE_LOGO_SIZE = (50, 50)
SPLASH_SIZE = (620, 300)

# File runtime mà tray/cli/TIP đọc từ đĩa (package_bootstrap stage-out chép nguyên).
REQUIRED_PAYLOAD = [
    "TextVN.exe",
    "textvn-cli.exe",
    "textvn-tsf.dll",
    "textvn-tsf-x86.dll",
    "resources/textvn.ico",
    "resources/textvn_v.ico",
    "resources/textvn_e.ico",
    "data/appdb.default.json",
]

# Dấu hiệu UTF-8 bị mã hoá 2 lần (PS 5.1 đọc template như ANSI — bug 0.2.27).
MOJIBAKE = re.compile("[ÃÂ][\u0080-¿]|á»|áº|Æ°")
VIETNAMESE = re.compile("[ăâđêôơưẠ-ỹ]", re.IGNORECASE)


def png_size(data: bytes) -> tuple[int, int] | None:
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    w, h = struct.unpack(">II", data[16:24])
    return w, h


def map_version(product: str) -> str:
    """0.2.27 → 1.2.27.0 — phải khớp tools/win/build-msix.ps1 (R2-01)."""
    m = re.fullmatch(r"(\d{1,5})\.(\d{1,5})\.(\d{1,5})", product)
    if not m:
        raise ValueError(f"version sản phẩm '{product}' không phải A.B.C")
    a, b, c = (int(x) for x in m.groups())
    return f"{a + 1}.{b}.{c}.0"


def check(
    path: str,
    expect_version: str | None,
    require_identity: bool,
    expect_display: str = EXPECTED_DISPLAY_NAME,
) -> list[str]:
    fails: list[str] = []
    try:
        z = zipfile.ZipFile(path)
    except (OSError, zipfile.BadZipFile) as e:
        return [f"không mở được gói: {e}"]
    names = set(z.namelist())
    lower = {n.lower(): n for n in names}

    def member(rel: str) -> str | None:
        return lower.get(rel.replace("\\", "/").lower())

    if "AppxManifest.xml" not in names:
        return ["thiếu AppxManifest.xml"]
    raw = z.read("AppxManifest.xml")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return ["AppxManifest.xml không phải UTF-8"]
    root = ET.fromstring(raw)

    ident = root.find("f:Identity", NS)
    if ident is None:
        return ["thiếu <Identity>"]
    name = ident.get("Name", "")
    publisher = ident.get("Publisher", "")
    version = ident.get("Version", "")
    if not re.fullmatch(r"[A-Za-z0-9.-]{3,50}", name):
        fails.append(f"Identity/Name '{name}' không hợp lệ (3-50 ký tự A-Za-z0-9.-)")
    if require_identity and name == PLACEHOLDER_NAME:
        fails.append(f"Identity/Name vẫn là placeholder '{PLACEHOLDER_NAME}' — đặt MSIX_IDENTITY_NAME")
    if publisher != EXPECTED_PUBLISHER:
        fails.append(f"Identity/Publisher '{publisher}' ≠ Partner Center '{EXPECTED_PUBLISHER}'")
    parts = version.split(".")
    if len(parts) != 4 or not all(p.isdigit() and int(p) <= 65535 for p in parts):
        fails.append(f"Identity/Version '{version}' phải là 4 số 0..65535")
    else:
        if parts[0] == "0":
            fails.append(f"Identity/Version '{version}': Partner Center từ chối phần đầu = 0")
        if parts[3] != "0":
            fails.append(f"Identity/Version '{version}': phần thứ 4 Store dành riêng, phải = 0")
    if expect_version:
        want = map_version(expect_version)
        if version != want:
            fails.append(f"Identity/Version '{version}' ≠ '{want}' (map từ {expect_version})")
    if ident.get("ProcessorArchitecture") != "x64":
        fails.append("Identity/ProcessorArchitecture phải là x64")

    props = root.find("f:Properties", NS)
    disp = props.findtext("f:DisplayName", default="", namespaces=NS) if props is not None else ""
    pub_disp = props.findtext("f:PublisherDisplayName", default="", namespaces=NS) if props is not None else ""
    desc = props.findtext("f:Description", default="", namespaces=NS) if props is not None else ""
    if disp != expect_display:
        fails.append(f"DisplayName '{disp}' ≠ tên đã reserve '{expect_display}'")
    if pub_disp != EXPECTED_PUBLISHER_DISPLAY:
        fails.append(f"PublisherDisplayName '{pub_disp}' ≠ '{EXPECTED_PUBLISHER_DISPLAY}'")
    if MOJIBAKE.search(text):
        fails.append("manifest có chuỗi mojibake (UTF-8 mã hoá 2 lần) — template đọc sai encoding")
    if not VIETNAMESE.search(desc):
        fails.append(f"Description không còn dấu tiếng Việt: '{desc}'")

    if root.find(".//rescap:Capability[@Name='runFullTrust']", NS) is None:
        fails.append("thiếu rescap:Capability runFullTrust")

    # Ảnh: mọi thuộc tính *Logo / SplashScreen tham chiếu phải tồn tại và đúng kích thước.
    logo = props.findtext("f:Logo", default="", namespaces=NS) if props is not None else ""
    refs: list[tuple[str, str, tuple[int, int] | None]] = [("Logo", logo, STORE_LOGO_SIZE)]
    for el in root.iter():
        for attr, val in el.attrib.items():
            if attr in ASSET_SIZES:
                refs.append((attr, val, ASSET_SIZES[attr]))
        if el.tag == f"{{{NS['uap']}}}SplashScreen":
            refs.append(("SplashScreen", el.get("Image", ""), SPLASH_SIZE))
    for attr, rel, size in refs:
        m = member(rel)
        if m is None:
            fails.append(f"{attr} trỏ '{rel}' không có trong gói")
            continue
        got = png_size(z.read(m))
        if got is None:
            fails.append(f"{attr} '{rel}' không phải PNG")
        elif size and got != size:
            fails.append(f"{attr} '{rel}' là {got[0]}x{got[1]}, cần {size[0]}x{size[1]}")

    for app in root.iterfind(".//f:Application", NS):
        exe = app.get("Executable", "")
        if member(exe) is None:
            fails.append(f"Application Executable '{exe}' không có trong gói")

    for rel in REQUIRED_PAYLOAD:
        if member(rel) is None:
            fails.append(f"payload thiếu {rel}")
    if not any(n.lower().startswith("data/tables/") and n.lower().endswith(".toml") for n in names):
        fails.append("payload thiếu data/tables/*.toml (engine đọc bảng từ đĩa)")
    return fails


def main(argv: list[str]) -> int:
    # Windows: stdout mặc định cp1252 → in tiếng Việt văng UnicodeEncodeError
    # (CI 2026-10-08: kiểm tra ĐẠT nhưng dòng OK làm bước đỏ).
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("msix")
    ap.add_argument("--expect-version", help="version sản phẩm A.B.C (Cargo.toml)")
    ap.add_argument("--require-store-identity", action="store_true",
                    help="fail khi Identity/Name còn placeholder (gói để nộp Store)")
    ap.add_argument("--display-name", default=os.environ.get("MSIX_DISPLAY_NAME", ""),
                    help="tên đã reserve cho sản phẩm MSIX (mặc định TextVN / biến MSIX_DISPLAY_NAME)")
    args = ap.parse_args(argv)
    # R2-95: cắt khoảng trắng và rỗng → mặc định, ĐÚNG như build-msix.ps1 — repo
    # variable dán kèm dấu cách từng làm verify báo lệch tên với gói vừa build.
    display_name = args.display_name.strip() or EXPECTED_DISPLAY_NAME
    fails = check(args.msix, args.expect_version, args.require_store_identity, display_name)
    for f in fails:
        print(f"FAIL {f}")
    if fails:
        return 1
    print(f"OK   {args.msix}: identity/version/tên hiển thị/encoding/ảnh/payload đạt luật Store")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
