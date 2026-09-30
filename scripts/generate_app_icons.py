#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh icon PNG ứng dụng TextVN từ thiết kế badge trong `resources/icons/*.svg`.

Nguồn sự thật là 2 SVG (textvn_v / textvn_e — badge bo góc, viền, gradient và
chữ cái). SVG dùng <text> với font hệ thống nên không rasterize trực tiếp được
bằng công cụ có sẵn; script này vẽ lại đúng hình học và màu của SVG bằng Pillow
(PIL) ở 1024px rồi xuất:

- ``packaging/macos/icons/TextVN-1024.png`` — nguồn .icns cho build-macos.sh
  (16..512 + @2x, sips/iconutil tự làm trên máy macOS).
- ``packaging/macos/icons/TextVN-512.png`` — fallback ≥512 theo yêu cầu script.
- ``resources/icons/textvn_v.png`` / ``textvn_e.png`` 128px — raster hicolor cho
  panel/tray cũ không render SVG (stage-linux.sh + install.sh cài kèm SVG).

Chạy một lần trên máy dev, PNG được commit; CI không cần Pillow:

    python scripts/generate_app_icons.py

Thiết kế khớp SVG (hệ toạ độ 128, scale S = size/128):
- badge: rect x=8..120, rx=24; stroke 8 (nửa trong, nửa ngoài) màu #C2185B/#0288D1
- gradient nền chéo 0%→100%: #FFFFFF→#FCE4EC (V) / #E1F5FE (E)
- chữ: baseline y=93, font-size 82, đậm nhất có sẵn, #880E4F (V) / #01579B (E)
- bóng đổ: dy=2, blur stdDev=3, màu viền, alpha 25%
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent

# (letter, stroke, gradient_bottom, text) — trùng hex của 2 file SVG.
VARIANTS = {
    "textvn_v": ("V", "#C2185B", "#FCE4EC", "#880E4F"),
    "textvn_e": ("E", "#0288D1", "#E1F5FE", "#01579B"),
}

# Ưu tiên font đậm nhất theo SVG font-weight 900; fonts trên Windows host, rơi
# xuống DejaVu (Linux) nếu thiếu.
FONT_CANDIDATES = [
    r"C:\Windows\Fonts\seguibl.ttf",  # Segoe UI Black
    r"C:\Windows\Fonts\segoeuib.ttf",  # Segoe UI Bold
    r"C:\Windows\Fonts\arialbd.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]


def pick_font() -> str:
    for candidate in FONT_CANDIDATES:
        if Path(candidate).is_file():
            return candidate
    raise SystemExit("Không tìm thấy font đậm nào trong: " + ", ".join(FONT_CANDIDATES))


def gradient(size: int, top: str, bottom: str) -> Image.Image:
    """Gradient chéo 45° khớp linearGradient 0%,0%→100%,100% (objectBoundingBox)."""
    import numpy as np

    t, b = (
        int(top[i : i + 2], 16) for i in (1, 3, 5)
    ), (int(bottom[i : i + 2], 16) for i in (1, 3, 5))
    t, b = tuple(t), tuple(b)
    ys, xs = np.mgrid[0:size, 0:size]
    # T trong [0,1] theo toạ độ chéo của toàn ảnh (bbox 8..120 ×8 ≈ 0..size).
    diag = (xs + ys) / (2 * size - 1)
    arr = np.empty((size, size, 3), dtype=np.uint8)
    for ch in range(3):
        arr[..., ch] = (t[ch] + (b[ch] - t[ch]) * diag).astype(np.uint8)
    return Image.fromarray(arr, "RGB")


def render(letter: str, stroke: str, grad_bottom: str, text_color: str, size: int) -> Image.Image:
    s = size / 128.0
    outer = [round(4 * s), round(4 * s), round(124 * s), round(124 * s)]  # mép ngoài stroke
    inner = [round(12 * s), round(12 * s), round(116 * s), round(116 * s)]  # mép trong stroke
    radius_outer, radius_inner = round(24 * s + 4 * s), round(24 * s - 4 * s)

    # Bóng đổ: silhouette của badge (rect ngoài), blur, tint màu viền 25%,
    # dịch xuống dy=2 (SVG feDropShadow).
    silhouette = Image.new("L", (size, size), 0)
    ImageDraw.Draw(silhouette).rounded_rectangle(outer, radius_outer, fill=255)
    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    tinted = Image.new("RGBA", (size, size), stroke + "40")  # 25% alpha
    shadow.paste(tinted, (0, round(2 * s)), silhouette.filter(ImageFilter.GaussianBlur(3 * s)))

    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    img.alpha_composite(shadow)

    badge = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(badge)
    draw.rounded_rectangle(outer, radius_outer, fill=stroke)  # lớp viền
    grad = gradient(size, "#FFFFFF", grad_bottom)
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle(inner, radius_inner, fill=255)
    badge.paste(grad, (0, 0), mask)

    font = ImageFont.truetype(pick_font(), round(82 * s))
    draw.text((64 * s, 93 * s), letter, font=font, fill=text_color, anchor="ms")
    img.alpha_composite(badge)
    return img


def main() -> None:
    out_macos = ROOT / "packaging" / "macos" / "icons"
    out_macos.mkdir(parents=True, exist_ok=True)
    for name, (letter, stroke, grad, text) in VARIANTS.items():
        img128 = render(letter, stroke, grad, text, 128)
        img128.save(ROOT / "resources" / "icons" / f"{name}.png", optimize=True)
    # Nguồn .icns macOS: 1024 (sips tự sinh mọi mức từ nguồn lớn nhất).
    big = render("V", *VARIANTS["textvn_v"][1:], 1024)
    big.save(out_macos / "TextVN-1024.png", optimize=True)
    big.resize((512, 512), Image.LANCZOS).save(out_macos / "TextVN-512.png", optimize=True)
    print("Đã sinh:", out_macos / "TextVN-1024.png", "và 2 PNG hicolor 128px")


if __name__ == "__main__":
    main()
