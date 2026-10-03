#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh art cho phần listing Microsoft Store của TextVN.

Yêu cầu Partner Center (ảnh listing, KHÁC tile trong gói MSIX):
- Box art 1:1: 1080x1080 hoặc 2160x2160 (BẮT BUỘC) — script sinh 2160 rồi hạ 1080.
- Poster art 2:3: 720x1080 hoặc 1440x2160 (khuyến nghị) — sinh 1440x2160 rồi hạ 720x1080.
- .png, < 50 MB (file ở đây chỉ vài trăm KB).

Thiết kế: nền đỏ cờ Việt Nam (gradient) + sao vàng 5 cánh + chữ V trắng đậm —
đồng bộ tinh thần với icon mode tiếng Việt [V] của bộ gõ.

Nguồn sự thật của script này là file `scripts/generate_store_art.py`; chạy:

    python scripts/generate_store_art.py

Kết quả ở `store/art/`. CI không cần Pillow (ảnh được commit sẵn).
"""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "store" / "art"

# Bảng màu cờ Việt Nam + dịu hoá cho nền gradient.
FLAG_RED = (218, 37, 29)
RED_DARK = (150, 18, 14)
STAR_YELLOW = (255, 221, 0)
V_WHITE = (255, 255, 255)

# Font đậm nhất có sẵn trên Windows; fallback theo thứ tự.
FONT_CANDIDATES = [
    r"C:\Windows\Fonts\seguibl.ttf",  # Segoe UI Black
    r"C:\Windows\Fonts\arialbd.ttf",  # Arial Bold
]


def load_font(size: int) -> ImageFont.FreeTypeFont:
    for path in FONT_CANDIDATES:
        if Path(path).exists():
            return ImageFont.truetype(path, size)
    raise SystemExit("Không tìm thấy font đậm (seguibl/arialbd) trong C:\\Windows\\Fonts")


def diag_gradient(size: tuple[int, int], c0: tuple[int, int, int], c1: tuple[int, int, int]) -> Image.Image:
    """Nền gradient chéo 45° — cùng hướng với badge icon của app."""
    w, h = size
    grad = Image.new("RGB", (w + h, w + h))
    px = grad.load()
    total = (w + h) * 2
    for y in range(w + h):
        for x in range(w + h):
            t = (x + y) / total
            px[x, y] = tuple(round(a + (b - a) * t) for a, b in zip(c0, c1))
    return grad.crop((0, 0, w, h)).convert("RGBA")


def star_points(cx: float, cy: float, outer: float) -> list[tuple[float, float]]:
    """10 đỉnh sao 5 cánh (đỉnh hướng lên), tỉ lệ vàng chuẩn 0.38197."""
    inner = outer * 0.38197
    pts = []
    for i in range(10):
        r = outer if i % 2 == 0 else inner
        ang = -math.pi / 2 + i * math.pi / 5
        pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
    return pts


def draw_star(draw: ImageDraw.ImageDraw, cx: float, cy: float, outer: float) -> None:
    draw.polygon(star_points(cx, cy, outer), fill=STAR_YELLOW)


def compose(w: int, h: int, star_r: float, star_cy: float, v_size: int, v_cy: float) -> Image.Image:
    img = diag_gradient((w, h), FLAG_RED, RED_DARK)
    draw = ImageDraw.Draw(img)
    cx = w / 2
    # Chữ V ở dưới, sao vàng ở trên — bố cục dọc cân giữa.
    font = load_font(v_size)
    bbox = draw.textbbox((cx, v_cy), "V", font=font, anchor="mm")
    # Đổ bóng nhẹ cho V nổi trên nền đỏ.
    off = max(4, v_size // 60)
    draw.text((cx + off, v_cy + off), "V", font=font, anchor="mm", fill=(90, 10, 8, 255))
    draw.text((cx, v_cy), "V", font=font, anchor="mm", fill=V_WHITE)
    draw_star(draw, cx, star_cy, star_r)
    return img


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    # 1:1 — 2160 (nguồn) + 1080.
    box = compose(2160, 2160, star_r=380, star_cy=760, v_size=980, v_cy=1530)
    box.resize((1080, 1080), Image.LANCZOS).save(OUT / "box-art-1080.png", "PNG")
    box.save(OUT / "box-art-2160.png", "PNG")

    # 2:3 — 1440x2160 (nguồn) + 720x1080.
    poster = compose(1440, 2160, star_r=300, star_cy=620, v_size=760, v_cy=1400)
    poster.resize((720, 1080), Image.LANCZOS).save(OUT / "poster-art-720x1080.png", "PNG")
    poster.save(OUT / "poster-art-1440x2160.png", "PNG")

    for f in sorted(OUT.glob("*.png")):
        print(f"{f.relative_to(ROOT)}  {f.stat().st_size // 1024} KB")


if __name__ == "__main__":
    main()
