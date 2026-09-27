# SPDX-License-Identifier: GPL-3.0-or-later
"""
Script tạo file icon .ico cho TextVN (tray badge theo cờ quốc gia):
- textvn_v.ico: Nền cờ Việt Nam (đỏ #DA251D + ngôi sao vàng #FFCD00),
  đè chữ 'V' Segoe UI Bold trắng viền mảnh -> 'V' nổi bật ở mọi size.
- textvn_e.ico: Nền cờ Mỹ (7 sọc đỏ/trắng + ô xanh #3C3B6E),
  đè chữ 'E' Segoe UI Bold trắng viền đen -> 'E' nổi trên nền sọc.
Chứa các size: 256, 128, 64, 48, 32, 24, 20, 16.
Preview liên hệ (contact sheet) ghi vào %TEMP%\\textvn_icon_preview.png.
"""

import math
import os
import shutil

from PIL import Image, ImageDraw, ImageFont

SIZES = [256, 128, 64, 48, 32, 24, 20, 16]

FONT_BOLD = r"C:\Windows\Fonts\segoeuib.ttf"
if not os.path.exists(FONT_BOLD):
    FONT_BOLD = r"C:\Windows\Fonts\arialbd.ttf"

# Bang mau
VN_RED = (218, 37, 29, 255)   # #DA251D - do co Viet Nam
VN_GOLD = (255, 205, 0, 255)  # #FFCD00 - vang sao vang
US_RED = (178, 34, 52, 255)   # #B22234
US_WHITE = (255, 255, 255, 255)
US_BLUE = (60, 59, 110, 255)  # #3C3B6E
SHADOW = (0, 0, 0, 110)


def _rounded_mask(size: int) -> Image.Image:
    """Mask bo gioc 20%% cua icon."""
    mask = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(mask)
    m = max(1, int(size * 0.03))
    radius = max(2, int(size * 0.20))
    d.rounded_rectangle([m, m, size - 1 - m, size - 1 - m], radius=radius, fill=255)
    return mask


def _draw_star(draw: ImageDraw.ImageDraw, cx: float, cy: float, r_out: float, color: tuple):
    """Sao 5 canh (dinh len tren)."""
    pts = []
    for i in range(10):
        ang = math.radians(-90 + i * 36)
        r = r_out if i % 2 == 0 else r_out * 0.44
        pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
    draw.polygon(pts, fill=color)


def _centered_text(draw: ImageDraw.ImageDraw, size: int, letter: str,
                   font: ImageFont.FreeTypeFont, cy: float,
                   fill: tuple, stroke: tuple, stroke_w: int):
    bbox = draw.textbbox((0, 0), letter, font=font, stroke_width=stroke_w)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    x = (size - tw) / 2 - bbox[0]
    y = cy - th / 2 - bbox[1]
    # Bai tot (shadow) nho
    if size >= 20:
        draw.text((x + max(1, size // 64), y + max(1, size // 64)), letter,
                  font=font, fill=SHADOW, stroke_width=stroke_w, stroke_fill=SHADOW)
    draw.text((x, y), letter, font=font, fill=fill,
              stroke_width=stroke_w, stroke_fill=stroke)


def _letter_font(size: int) -> ImageFont.FreeTypeFont:
    font_size = max(9, int(size * (0.60 if size <= 24 else 0.56)))
    try:
        return ImageFont.truetype(FONT_BOLD, font_size)
    except Exception:
        return ImageFont.load_default()


def create_vn_icon(size: int) -> Image.Image:
    """Co Viet Nam: nen do + sao vang + chu V trang noi."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    mask = _rounded_mask(size)

    base = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(base)
    d.rectangle([0, 0, size, size], fill=VN_RED)
    # Sao vang hoi len tren de chu V o duoi doc ro (dac biet o 16px)
    _draw_star(d, size * 0.5, size * 0.37, size * 0.31, VN_GOLD)

    img.paste(base, (0, 0), mask)

    d2 = ImageDraw.Draw(img)
    font = _letter_font(size)
    stroke_w = max(1, size // 40)
    _centered_text(d2, size, "V", font, size * 0.60, US_WHITE,
                   (30, 20, 10, 190), stroke_w)
    return img


def create_us_icon(size: int) -> Image.Image:
    """Co My: 7 song do/trang + o xanh co sao + chu E trang noi."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    mask = _rounded_mask(size)

    base = Image.new("RGBA", (size, size), US_WHITE)
    d = ImageDraw.Draw(base)

    # 13 song co chuan (7 do, 6 trang); size nho dung 7 day de khong lem
    n_bands = 13 if size >= 32 else 7
    stripe_h = size / float(n_bands)
    for i in range(n_bands):
        if i % 2 == 0:
            y0 = i * stripe_h
            y1 = (i + 1) * stripe_h
            d.rectangle([0, y0, size, y1], fill=US_RED)

    # O xanh (canton): rong 40%%, cao 7/13 (band thu 4)
    cw = size * 0.40
    ch = stripe_h * min(7, n_bands // 2 + 1)
    d.rectangle([0, 0, cw, ch], fill=US_BLUE)

    # Sao trang trong canton (grid don gian; bo qua o size qua nho)
    if size >= 32:
        cols, rows = 5, 4
        r = max(1.0, size * 0.022)
        for row in range(rows):
            for col in range(cols):
                x = cw * (col + 1) / (cols + 1)
                y = ch * (row + 1) / (rows + 1)
                _draw_star(d, x, y, r * 2.2, US_WHITE)
    elif size >= 20:
        # 4 cham trang don gian
        r = max(1.0, size * 0.035)
        for gx, gy in [(0.25, 0.25), (0.6, 0.25), (0.25, 0.6), (0.6, 0.6)]:
            d.ellipse([cw * gx - r, ch * gy - r, cw * gx + r, ch * gy + r],
                      fill=US_WHITE)

    img.paste(base, (0, 0), mask)

    d2 = ImageDraw.Draw(img)
    font = _letter_font(size)
    stroke_w = max(1, size // 16)
    _centered_text(d2, size, "E", font, size * 0.55, US_WHITE,
                   (15, 15, 25, 225), stroke_w)
    return img


def generate_ico(create_fn, output_path: str):
    images = [create_fn(s) for s in SIZES]
    master = images[0]
    master.save(
        output_path,
        format="ICO",
        sizes=[(s, s) for s in SIZES],
        append_images=images[1:],
    )
    print(f"Generated: {output_path} ({os.path.getsize(output_path)} bytes)")


def write_preview_sheet(v_ico: str, e_ico: str):
    """Contact sheet kiem tra tren nen sang/toi + zoom pixel."""
    tmp = os.path.join(os.environ.get("TEMP", "."), "textvn_icon_preview.png")
    v = Image.open(v_ico)
    e = Image.open(e_ico)

    w, h = 980, 560
    sheet = Image.new("RGBA", (w, h), (240, 240, 240, 255))
    d = ImageDraw.Draw(sheet)
    try:
        lbl = ImageFont.truetype(FONT_BOLD, 18)
        lbl_s = ImageFont.truetype(FONT_BOLD, 13)
    except Exception:
        lbl = lbl_s = ImageFont.load_default()

    d.text((30, 12), "TextVN tray icons - 256px", font=lbl, fill=(40, 40, 40))
    sheet.paste(v.resize((220, 220), Image.LANCZOS), (40, 45), v.resize((220, 220), Image.LANCZOS))
    sheet.paste(e.resize((220, 220), Image.LANCZOS), (300, 45), e.resize((220, 220), Image.LANCZOS))

    # Nen toi (taskbar dark)
    d.rectangle([560, 45, 940, 265], fill=(32, 32, 32))
    d.text((575, 50), "dark bg", font=lbl_s, fill=(200, 200, 200))
    sheet.paste(v.resize((160, 160), Image.LANCZOS), (580, 80), v.resize((160, 160), Image.LANCZOS))
    sheet.paste(e.resize((160, 160), Image.LANCZOS), (760, 80), e.resize((160, 160), Image.LANCZOS))

    # Native sizes tren nen sang + toi
    d.text((30, 285), "native 48/32/24/20/16 (sang)", font=lbl_s, fill=(40, 40, 40))
    x = 40
    for s in [48, 32, 24, 20, 16]:
        iv, ie = v.resize((s, s), Image.LANCZOS), e.resize((s, s), Image.LANCZOS)
        sheet.paste(iv, (x, 310), iv)
        sheet.paste(ie, (x, 310 + 55), ie)
        x += s + 24

    d.text((420, 285), "native 48/32/24/20/16 (toi)", font=lbl_s, fill=(40, 40, 40))
    d.rectangle([415, 305, 760, 400], fill=(32, 32, 32))
    x = 425
    for s in [48, 32, 24, 20, 16]:
        iv, ie = v.resize((s, s), Image.LANCZOS), e.resize((s, s), Image.LANCZOS)
        sheet.paste(iv, (x, 312), iv)
        sheet.paste(ie, (x, 312 + 55), ie)
        x += s + 24

    # Zoom 16px x6 (nearest) de thay pixel
    d.text((30, 420), "16px zoom x6 (nearest)", font=lbl_s, fill=(40, 40, 40))
    z = 6
    iv16 = v.resize((16, 16), Image.NEAREST).resize((16 * z, 16 * z), Image.NEAREST)
    ie16 = e.resize((16, 16), Image.NEAREST).resize((16 * z, 16 * z), Image.NEAREST)
    sheet.paste(iv16, (40, 445), iv16)
    sheet.paste(ie16, (160, 445), ie16)
    iv32 = v.resize((32, 32), Image.NEAREST).resize((32 * 3, 32 * 3), Image.NEAREST)
    ie32 = e.resize((32, 32), Image.NEAREST).resize((32 * 3, 32 * 3), Image.NEAREST)
    sheet.paste(iv32, (300, 445), iv32)
    sheet.paste(ie32, (410, 445), ie32)

    sheet.save(tmp)
    print(f"Preview: {tmp}")


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    out_dir = os.path.join(here, "..", "tray", "resources")
    os.makedirs(out_dir, exist_ok=True)

    v_ico = os.path.join(out_dir, "textvn_v.ico")
    e_ico = os.path.join(out_dir, "textvn_e.ico")

    generate_ico(create_vn_icon, v_ico)
    generate_ico(create_us_icon, e_ico)
    write_preview_sheet(v_ico, e_ico)

    # Cap nhat default textvn.ico cho installer va tray
    installer_dir = os.path.join(here, "..", "installer", "windows", "resources")
    os.makedirs(installer_dir, exist_ok=True)
    shutil.copyfile(v_ico, os.path.join(out_dir, "textvn.ico"))
    shutil.copyfile(v_ico, os.path.join(installer_dir, "textvn.ico"))
    print("Updated default textvn.ico in tray and installer resources.")


if __name__ == "__main__":
    main()
