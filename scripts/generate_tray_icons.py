# SPDX-License-Identifier: GPL-3.0-or-later
"""
Script tạo file icon .ico cho TextVN:
- textvn_v.ico: Chữ 'V' màu trắng trên nền Tím (#7B1FA2), bo góc nhẹ.
- textvn_e.ico: Chữ 'E' màu trắng trên nền Xanh (#1565C0), bo góc nhẹ.
Chứa các size: 256x256, 128x128, 64x64, 48x48, 32x32, 24x24, 20x20, 16x16.
"""

import os
import shutil
from PIL import Image, ImageDraw, ImageFont

SIZES = [256, 128, 64, 48, 32, 24, 20, 16]

FONT_PATH = "C:\\Windows\\Fonts\\segoeuib.ttf"
if not os.path.exists(FONT_PATH):
    FONT_PATH = "C:\\Windows\\Fonts\\arialbd.ttf"

def create_badge(size: int, letter: str, bg_color: tuple) -> Image.Image:
    # Tạo ảnh RGBA
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Bán kính bo góc tỉ lệ với kích thước
    radius = max(2, int(size * 0.22))
    margin = max(1, int(size * 0.04))
    
    # Vẽ nền bo góc (rounded rectangle)
    rect = [margin, margin, size - 1 - margin, size - 1 - margin]
    draw.rounded_rectangle(rect, radius=radius, fill=bg_color)
    
    # Tính font size vừa vặn với badge (tăng kích thước chữ một chút ở size nhỏ để dễ nhìn trên tray)
    scale = 0.72 if size <= 24 else 0.68
    font_size = max(9, int(size * scale))
    try:
        font = ImageFont.truetype(FONT_PATH, font_size)
    except Exception:
        font = ImageFont.load_default()
    
    # Canh giữa chữ hoàn hảo
    bbox = draw.textbbox((0, 0), letter, font=font)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    
    x = (size - text_w) / 2 - bbox[0]
    y = (size - text_h) / 2 - bbox[1]
    
    draw.text((x, y), letter, font=font, fill=(255, 255, 255, 255))
    return img

def generate_ico(letter: str, bg_color: tuple, output_path: str):
    # Tạo ảnh từ lớn đến nhỏ (256 -> 16)
    images = [create_badge(s, letter, bg_color) for s in SIZES]
    
    # Master image là 256x256, append các size nhỏ hơn
    master = images[0]
    append_list = images[1:]
    
    master.save(
        output_path,
        format="ICO",
        sizes=[(s, s) for s in SIZES],
        append_images=append_list,
    )
    print(f"Generated: {output_path} ({os.path.getsize(output_path)} bytes)")

def main():
    purple = (123, 31, 162, 255)  # #7B1FA2 Material Purple 700 (Tím)
    blue = (21, 101, 192, 255)    # #1565C0 Material Blue 800 (Xanh)

    out_dir = os.path.join(os.path.dirname(__file__), "..", "tray", "resources")
    os.makedirs(out_dir, exist_ok=True)

    v_ico = os.path.join(out_dir, "textvn_v.ico")
    e_ico = os.path.join(out_dir, "textvn_e.ico")

    generate_ico("V", purple, v_ico)
    generate_ico("E", blue, e_ico)

    # Cập nhật default textvn.ico cho installer và tray
    installer_dir = os.path.join(os.path.dirname(__file__), "..", "installer", "windows", "resources")
    os.makedirs(installer_dir, exist_ok=True)
    shutil.copyfile(v_ico, os.path.join(out_dir, "textvn.ico"))
    shutil.copyfile(v_ico, os.path.join(installer_dir, "textvn.ico"))
    print("Updated default textvn.ico in tray and installer resources.")

if __name__ == "__main__":
    main()
