#!/usr/bin/env python3
"""
Tạo vietime.ico — icon VietIME tray với chữ "V" nổi bật trên nền xanh.
ICO format: chứa 3 kích thước: 16x16, 32x32, 48x48 (BMP + PNG header).
Không dùng Pillow — tự viết raw ICO/BMP bytes.
"""
import struct
import os

def make_bmp_icon(size, bg_color, text_color):
    """Tạo BMP bitmap cho ICO (32bpp BGRA, bottom-up)."""
    w, h = size, size
    # Pixel grid — chữ "V" đơn giản
    pixels = []
    
    for row in range(h):
        # BMP là bottom-up: row=0 là dòng DƯỚI cùng
        y = h - 1 - row
        row_pixels = []
        for x in range(w):
            # Nền
            color = bg_color
            
            # Vẽ chữ "V" đơn giản: 2 đường chéo gặp ở giữa dưới
            # Normalize coords to 0..1
            nx = x / (w - 1) if w > 1 else 0
            ny = y / (h - 1) if h > 1 else 0
            
            # Margin: 15% từ mỗi cạnh
            margin = 0.15
            # Stroke width: 15% của chiều rộng
            stroke = 0.15
            
            # Left arm: từ (margin, 1-margin) xuống (0.5, margin)
            # Right arm: từ (1-margin, 1-margin) xuống (0.5, margin)
            
            # Left arm line: x = margin + (0.5-margin)*(1 - (y-margin)/(1-2*margin)) when y in [margin, 1-margin]
            if margin <= ny <= 1 - margin:
                t = (ny - margin) / (1 - 2 * margin)
                # left: nx from margin (at t=0/bottom) to 0.5 (at t=1/top)
                left_cx = margin + (0.5 - margin) * (1 - t)
                right_cx = (1 - margin) - (0.5 - margin) * (1 - t)
                
                if abs(nx - left_cx) < stroke / 2 or abs(nx - right_cx) < stroke / 2:
                    color = text_color
            
            r, g, b, a = color
            row_pixels.extend([b, g, r, a])  # BGRA
        pixels.append(bytes(row_pixels))
    
    # BMP INFOHEADER (40 bytes) — Note: height is doubled for XOR+AND mask in ICO
    bmp_data = bytearray()
    bmp_data += struct.pack('<I', 40)      # biSize
    bmp_data += struct.pack('<i', w)       # biWidth
    bmp_data += struct.pack('<i', h * 2)   # biHeight (×2 for ICO: XOR+AND)
    bmp_data += struct.pack('<H', 1)       # biPlanes
    bmp_data += struct.pack('<H', 32)      # biBitCount
    bmp_data += struct.pack('<I', 0)       # biCompression (BI_RGB)
    bmp_data += struct.pack('<I', w * h * 4)  # biSizeImage
    bmp_data += struct.pack('<i', 0)       # biXPelsPerMeter
    bmp_data += struct.pack('<i', 0)       # biYPelsPerMeter
    bmp_data += struct.pack('<I', 0)       # biClrUsed
    bmp_data += struct.pack('<I', 0)       # biClrImportant
    
    # XOR mask (pixel data, bottom-up)
    for row in pixels:
        bmp_data += row
    
    # AND mask (1bpp, bottom-up) — all 0 = fully opaque
    and_stride = ((w + 31) // 32) * 4
    for _ in range(h):
        bmp_data += b'\x00' * and_stride
    
    return bytes(bmp_data)


def make_ico(sizes):
    """
    Tạo ICO file với nhiều kích thước.
    sizes: list of (size, bg_rgba, text_rgba)
    """
    # ICO header: 6 bytes
    # Directory entries: 16 bytes × n
    n = len(sizes)
    header_size = 6 + n * 16
    
    # Tính offset và data
    entries = []
    data_offset = header_size
    
    images = []
    for size, bg, text in sizes:
        data = make_bmp_icon(size, bg, text)
        images.append(data)
    
    for i, (size, bg, text) in enumerate(sizes):
        data = images[i]
        s = size if size < 256 else 0  # 0 means 256 in ICO
        entries.append((s, s, 0, 0, 1, 32, len(data), data_offset))
        data_offset += len(data)
    
    # Build ICO
    ico = bytearray()
    ico += struct.pack('<HHH', 0, 1, n)  # reserved, type=1 (ICO), count
    
    for s_w, s_h, color_count, reserved, planes, bit_count, size_bytes, offset in entries:
        ico += struct.pack('<BBBBHHII', 
                          s_w, s_h, color_count, reserved,
                          planes, bit_count, size_bytes, offset)
    
    for data in images:
        ico += data
    
    return bytes(ico)


if __name__ == '__main__':
    # Màu VietIME: nền xanh lam đậm (#1565C0), chữ trắng
    BG = (0x15, 0x65, 0xC0, 0xFF)    # #1565C0 RGBA
    FG = (0xFF, 0xFF, 0xFF, 0xFF)    # white

    sizes = [
        (16,  BG, FG),
        (32,  BG, FG),
        (48,  BG, FG),
    ]
    
    ico_data = make_ico(sizes)
    
    out_dir = os.path.join(os.path.dirname(__file__), 'resources')
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, 'vietime.ico')
    
    with open(out_path, 'wb') as f:
        f.write(ico_data)
    
    print(f"Created: {out_path} ({len(ico_data)} bytes)")
    print(f"  Sizes: {[s[0] for s in sizes]}px, 32bpp BGRA")
