//! E59: PNG encoder + canvas RGB(A) thuần std — thay Pillow trong 3 script sinh
//! icon/art đã xoá (`scripts/generate_app_icons.py`, `generate_store_art.py`,
//! `generate_tray_icons.py`).
//!
//! Với đèflate: dùng zlib "stored" blocks (BTYPE=00 — hợp lệ RFC 1951/1950,
//! mọi decoder PNG đọc được). Ưu tiên công cụ đúng-chuẩn + không-dependency +
//! không rủi ro nén tự viết: file PNG phát sinh chỉ LỚN HƠN bản Pillow (chỉ
//! dùng khi dev tái sinh thủ công — assets đã commit KHÔNG regenerate thường
//! xuyên, xem docs `docs/release/ui-spec.md` + store-submission).
//! Không unsafe, không crate ngoài.

// ---------------------------------------------------------------------------
//=== CRC-32 (PNG chunks) & Adler-32 (zlib) ===================================
// ---------------------------------------------------------------------------
fn crc32(data: &[u8]) -> u32 {
    // Bảng tính on-the-fly (polynomial 0xEDB88320) — 8 bit/lần đủ nhanh cho file
    // vài MB chạy 1 lần trên máy dev.
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in data.chunks(5552) {
        for &byte in chunk {
            a += byte as u32;
            b += a;
        }
        a %= MOD;
        b %= MOD;
    }
    (b << 16) | (a & 0xFFFF)
}

fn push_be32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Ghép 1 chunk PNG: len(4BE) + type + data + crc32(type+data).
fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    push_be32(out, data.len() as u32);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_src = Vec::with_capacity(4 + data.len());
    crc_src.extend_from_slice(kind);
    crc_src.extend_from_slice(data);
    push_be32(out, crc32(&crc_src));
}

/// zlib stream với DEFLATE "stored" blocks (BTYPE=00, LEN ≤ 65535/block).
fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    let mut z = Vec::with_capacity(raw.len() + raw.len() / 65_535 * 5 + 16);
    z.push(0x78); // CMF: deflate, 32K window
    z.push(0x01); // FLG: FLEVEL=0 (fastest), không preset dict (check ok)
    let mut off = 0usize;
    loop {
        let n = (raw.len() - off).min(65_535);
        let last = off + n >= raw.len();
        z.push(if last { 1 } else { 0 });
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[off..off + n]);
        off += n;
        if last {
            break;
        }
    }
    push_be32(&mut z, adler32(raw));
    z
}

/// Encode canvas RGBA 8-bit/kênh (filter 0 mỗi scanline) → bytes PNG hoàn chỉnh.
pub fn encode_rgba(canvas: &Canvas) -> Vec<u8> {
    let (w, h) = (canvas.w, canvas.h);
    // Filter 0: mỗi dòng thêm 1 byte 0 → raw stream = h * (1 + w*4).
    let mut raw = Vec::with_capacity(h * (1 + w * 4));
    for y in 0..h {
        raw.push(0u8);
        let row = y * w * 4;
        raw.extend_from_slice(&canvas.px[row..row + w * 4]);
    }
    let mut png = Vec::with_capacity(raw.len() + 128);
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let mut ihdr = Vec::with_capacity(13);
    push_be32(&mut ihdr, w as u32);
    push_be32(&mut ihdr, h as u32);
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression: deflate
    ihdr.push(0); // filter method
    ihdr.push(0); // interlace: none
    png_chunk(&mut png, b"IHDR", &ihdr);
    let z = zlib_stored(&raw);
    png_chunk(&mut png, b"IDAT", &z);
    png_chunk(&mut png, b"IEND", &[]);
    png
}

// ---------------------------------------------------------------------------
//=== Canvas RGBA cơ bản =====================================================
// ---------------------------------------------------------------------------
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>, // RGBA, w*h*4
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Rgba(r, g, b, 255)
    }
    pub fn from_hex(s: &str) -> Self {
        // "#RRGGBB" hoặc "#RRGGBBAA"
        let hex = s.strip_prefix('#').unwrap_or(s);
        let v = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
        match hex.len() {
            6 => Rgba(v(0), v(2), v(4), 255),
            8 => Rgba(v(0), v(2), v(4), v(6)),
            _ => panic!("hex màu lạ: {s}"),
        }
    }
}

impl Canvas {
    pub fn new(w: usize, h: usize, bg: Rgba) -> Self {
        let mut px = Vec::with_capacity(w * h * 4);
        for _ in 0..w * h {
            px.extend_from_slice(&[bg.0, bg.1, bg.2, bg.3]);
        }
        Canvas { w, h, px }
    }

    #[inline]
    fn blend_pixel(&mut self, x: isize, y: isize, c: Rgba) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let i = (y as usize * self.w + x as usize) * 4;
        let sa = c.3 as u32;
        if sa == 255 {
            self.px[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, 255]);
            return;
        }
        let da = self.px[i + 3] as u32;
        let out_a = sa + da * (255 - sa) / 255;
        if out_a == 0 {
            self.px[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, 0]);
            return;
        }
        // source-over (công thức chuẩn premultiplied rút gọn cho alpha u8).
        for (ch, sv) in [c.0 as u32, c.1 as u32, c.2 as u32].iter().enumerate() {
            let dv = self.px[i + ch] as u32;
            let val = ((sv * sa + dv * da * (255 - sa) / 255) / out_a).min(255) as u8;
            self.px[i + ch] = val;
        }
        self.px[i + 3] = out_a as u8;
    }

    /// Vẽ phủ đặc (không blend — ghi đè 4 byte) khi chắc chắn pixel nằm trong.
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, c: Rgba) {
        let i = (y * self.w + x) * 4;
        self.px[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, c.3]);
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Rgba {
        let i = (y * self.w + x) * 4;
        Rgba(self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3])
    }

    /// Rect đặc (x0,y0)-(x1,y1) inclusive — toạ độ float trắng trợ.
    pub fn fill_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Rgba) {
        let (xa, ya) = (x0.ceil().max(0.0), y0.ceil().max(0.0));
        let (xb, yb) = (x1.floor(), y1.floor());
        let mut y = ya as isize;
        while (y as f64) <= yb {
            let mut x = xa as isize;
            while (x as f64) <= xb {
                self.blend_pixel(x, y, c);
                x += 1;
            }
            y += 1;
        }
    }

    /// Rounded-rect fill (toạ độ PIL [x0,y0,x1,y1], radius) — anti-alias bằng
    /// coverage 4× subsample (đủ mượt cho icon 16..1024px).
    pub fn rounded_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, radius: f64, c: Rgba) {
        let mut y = y0.max(0.0).ceil() as isize;
        while (y as f64) <= y1.floor() {
            let mut x = x0.max(0.0).ceil() as isize;
            while (x as f64) <= x1.floor() {
                let cov = rounded_rect_coverage(x as f64, y as f64, x0, y0, x1, y1, radius);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
                x += 1;
            }
            y += 1;
        }
    }

    /// Polygon fill (scanline even-odd) với 4×4 subsample AA; points hệ (x,y).
    pub fn polygon(&mut self, pts: &[(f64, f64)], c: Rgba) {
        let (min_x, max_x, min_y, max_y) = bbox(pts);
        let mut y = (min_y.max(0.0).ceil()) as isize;
        while (y as f64) <= max_y.floor() {
            let mut x = (min_x.max(0.0).ceil()) as isize;
            while (x as f64) <= max_x.floor() {
                let cov = polygon_coverage(x as f64, y as f64, pts);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
                x += 1;
            }
            y += 1;
        }
    }

    /// Ellipse fill (bounding box [x0,y0,x1,y1]) — subsample AA.
    pub fn ellipse(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Rgba) {
        let mut y = y0.max(0.0).ceil() as isize;
        while (y as f64) <= y1.floor() {
            let mut x = x0.max(0.0).ceil() as isize;
            while (x as f64) <= x1.floor() {
                let cov = ellipse_coverage(x as f64, y as f64, x0, y0, x1, y1);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
                x += 1;
            }
            y += 1;
        }
    }

    /// Nền gradient chéo từ đỏ góc (0,0)→(w,h) — thay `diag_gradient()` PIL.
    pub fn gradient_bg(&mut self, top: Rgba, bottom: Rgba) {
        let diag = (self.w + self.h) as f64 - 1.0;
        for y in 0..self.h {
            for x in 0..self.w {
                let t = (x + y) as f64 / diag;
                let mix =
                    |a: u8, b: u8| -> u8 { (a as f64 + (b as f64 - a as f64) * t).round() as u8 };
                self.set_pixel(
                    x,
                    y,
                    Rgba(
                        mix(top.0, bottom.0),
                        mix(top.1, bottom.1),
                        mix(top.2, bottom.2),
                        255,
                    ),
                );
            }
        }
    }

    /// Composite source-over canvas khác (alpha-aware, đầy đủ canvas).
    pub fn composite(&mut self, other: &Canvas, dy: isize) {
        for y in 0..other.h {
            let ty = y as isize + dy;
            if ty < 0 || ty as usize >= self.h {
                continue;
            }
            for x in 0..other.w {
                let s = other.get_pixel(x, y);
                if s.3 == 0 {
                    continue;
                }
                self.blend_pixel(x as isize, ty, s);
            }
        }
    }

    /// Vẽ đè canvas khác qua mask L (alpha 0..255 theo channel đầu của mask).
    pub fn paste_with_mask(&mut self, src: &Canvas, mask: &Canvas, dy: isize) {
        for y in 0..src.h {
            let ty = y as isize + dy;
            if ty < 0 || ty as usize >= self.h {
                continue;
            }
            for x in 0..src.w.min(mask.w).min(self.w) {
                let alpha = mask.px[(y * mask.w + x) * 4];
                let s = src.get_pixel(x, y);
                if alpha == 0 {
                    continue;
                }
                self.blend_pixel(
                    x as isize,
                    ty,
                    Rgba(s.0, s.1, s.2, (alpha as u32 * s.3 as u32 / 255) as u8),
                );
            }
        }
    }

    /// Downsample /2 chính xác (box 2×2 mean premultiplied) — thay LANCZOS
    /// resize cho các bước chia hết 2.
    pub fn downsample2(self) -> Canvas {
        let w = self.w / 2;
        let h = self.h / 2;
        let mut out = Canvas::new(w, h, Rgba(0, 0, 0, 0));
        for y in 0..h {
            for x in 0..w {
                // premultiplied mean (tránh méo màu trên alpha thấp):
                let mut pre = [0u32; 4]; // r,g,b premul + a
                for (dy, dx) in [(0usize, 0usize), (0, 1), (1, 0), (1, 1)] {
                    let p = self.get_pixel(x * 2 + dx, y * 2 + dy);
                    let a = [p.0, p.1, p.2, p.3][3] as u32;
                    for ch in 0..3 {
                        pre[ch] += [p.0, p.1, p.2, p.3][ch] as u32 * a / 255;
                    }
                    pre[3] += a;
                }
                let a = pre[3];
                let v = |c: u32| {
                    if a == 0 {
                        0
                    } else {
                        ((c / 4) * 255 / (a / 4).max(1)).min(255) as u8
                    }
                };
                out.set_pixel(x, y, Rgba(v(pre[0]), v(pre[1]), v(pre[2]), (a / 4) as u8));
            }
        }
        out
    }

    /// Gaussian blur gần đúng (3 pass box blur) trên ALPHA — cho bóng đổ icon.
    pub fn blur_alpha(&self, radius: f64) -> Canvas {
        let passes = 3;
        let r = (radius.max(0.0) * 3.0 / 4.0).round() as usize; // box ≈ gaussian
        let mut alpha: Vec<u8> = (0..self.w * self.h).map(|i| self.px[i * 4 + 3]).collect();
        for _ in 0..passes {
            alpha = box_blur_1d(&alpha, self.w, self.h, r);
        }
        let mut out = self.clone_canvas();
        let px = &mut out.px;
        for (i, a) in alpha.iter().enumerate() {
            px[i * 4 + 3] = *a;
        }
        out
    }

    pub fn clone_canvas(&self) -> Canvas {
        Canvas {
            w: self.w,
            h: self.h,
            px: self.px.clone(),
        }
    }
}

fn box_blur_1d(src: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    if r == 0 {
        return src.to_vec();
    }
    // Horizontal rồi vertical — tích lũy chạy (sliding window).
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = y * w;
        let mut acc: u32 = 0;
        for x in 0..(r.min(w)).min(w) {
            acc += src[row + x] as u32;
        }
        for x in 0..w {
            let add = if x + r < w {
                src[row + x + r] as u32
            } else {
                0
            };
            let sub = if x > r {
                src[row + x - r - 1] as u32
            } else {
                0
            };
            acc += add;
            acc = acc.saturating_sub(sub);
            let cnt = ((x + r).min(w - 1).saturating_sub(x.saturating_sub(r))) as u32 + 1;
            tmp[row + x] = ((acc + cnt / 2) / cnt.max(1)).min(255) as u8;
        }
    }
    let mut out = vec![0u8; w * h];
    for x in 0..w {
        let mut acc: u32 = 0;
        for y in 0..(r.min(h)).min(h) {
            acc += tmp[y * w + x] as u32;
        }
        for y in 0..h {
            let add = if y + r < h {
                tmp[(y + r) * w + x] as u32
            } else {
                0
            };
            let sub = if y > r {
                tmp[(y - r - 1) * w + x] as u32
            } else {
                0
            };
            acc += add;
            acc = acc.saturating_sub(sub);
            let cnt = ((y + r).min(h - 1).saturating_sub(y.saturating_sub(r))) as u32 + 1;
            out[y * w + x] = ((acc + cnt / 2) / cnt.max(1)).min(255) as u8;
        }
    }
    out
}

// ---------------------------------------------------------------------------
//=== Coverage helpers (4×4 subsample anti-alias) =============================
// ---------------------------------------------------------------------------
fn rounded_rect_coverage(px: f64, py: f64, x0: f64, y0: f64, x1: f64, y1: f64, radius: f64) -> f64 {
    let radius = radius.max(0.0);
    let mut hits = 0u32;
    for sy in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
        for sx in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
            let x = px + sx;
            let y = py + sy;
            let cx = x.clamp(x0 + radius, x1 - radius);
            let cy = y.clamp(y0 + radius, y1 - radius);
            let inside = if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                let dx = x - cx;
                let dy = y - cy;
                dx * dx + dy * dy <= radius * radius + 1e-9
            } else {
                false
            };
            hits += inside as u32;
        }
    }
    hits as f64 / 16.0
}

fn polygon_coverage(px: f64, py: f64, pts: &[(f64, f64)]) -> f64 {
    let mut hits = 0u32;
    for sy in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
        for sx in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
            hits += point_in_poly(px + sx, py + sy, pts) as u32;
        }
    }
    hits as f64 / 16.0
}

fn point_in_poly(x: f64, y: f64, pts: &[(f64, f64)]) -> bool {
    // even-odd ray-cast chuẩn.
    let mut inside = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn ellipse_coverage(px: f64, py: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let mut hits = 0u32;
    let rx = (x1 - x0) / 2.0;
    let ry = (y1 - y0) / 2.0;
    if rx <= 0.0 || ry <= 0.0 {
        return 0.0;
    }
    let cx = (x0 + x1) / 2.0;
    let cy = (y0 + y1) / 2.0;
    for sy in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
        for sx in (0..4).map(|i| (i as f64 + 0.5) / 4.0) {
            let dx = (px + sx - cx) / rx;
            let dy = (py + sy - cy) / ry;
            hits += (dx * dx + dy * dy <= 1.0) as u32;
        }
    }
    hits as f64 / 16.0
}

fn bbox(pts: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let min_x = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    (min_x, max_x, min_y, max_y)
}

/// Sao 5 cánh (đỉnh hướng lên) — thay `_draw_star`/`star_points` PIL.
pub fn star_points(cx: f64, cy: f64, outer: f64, inner_ratio: f64) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(10);
    for i in 0..10 {
        let r = if i % 2 == 0 {
            outer
        } else {
            outer * inner_ratio
        };
        let ang = -std::f64::consts::PI / 2.0 + i as f64 * std::f64::consts::PI / 5.0;
        pts.push((cx + r * ang.cos(), cy + r * ang.sin()));
    }
    pts
}

// ---------------------------------------------------------------------------
//=== ICO container (PNG payload — chuẩn Vista+) ==============================
// ---------------------------------------------------------------------------
/// Ghép nhiều frame PNG thành file .ico (mỗi frame là PNG hoàn chỉnh — cùng
/// cách `master.save(format="ICO")` của Pillow với sources PNG).
pub fn encode_ico(frames: &[(u32, u32, &[u8])]) -> Vec<u8> {
    // frames = (w, h, png_bytes) — thứ tự = thứ tự entry.
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    out.extend_from_slice(&(frames.len() as u16).to_le_bytes()); // count
    let mut offset = 6 + frames.len() * 16;
    for (w, h, png) in frames {
        out.push(if *w >= 256 { 0 } else { *w as u8 });
        out.push(if *h >= 256 { 0 } else { *h as u8 });
        out.push(0); // palette
        out.push(0); // reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // color planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits/pixel
        let len = png.len() as u32;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += len as usize;
    }
    for (_, _, png) in frames {
        out.extend_from_slice(png);
    }
    out
}

// ---------------------------------------------------------------------------
//=== Unit tests ==============================================================
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_vector() {
        // Vector chuẩn: CRC32("123456789") = 0xCBF43926.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn adler32_known_vector() {
        assert_eq!(adler32(b"123456789"), 0x091E_01DE);
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn zlib_stored_stream_structure() {
        let data = vec![7u8; 200_000]; // 4 block
        let z = zlib_stored(&data);
        assert_eq!(&z[0..2], &[0x78, 0x01]);
        // decode ngược: blocks stored — final block byte tại pos 2.
        let mut pos = 2usize;
        let mut out = Vec::new();
        let mut last = false;
        while !last {
            last = z[pos] & 1 == 1;
            let len = u16::from_le_bytes([z[pos + 1], z[pos + 2]]) as usize;
            let nlen = u16::from_le_bytes([z[pos + 3], z[pos + 4]]).to_le_bytes();
            assert_eq!((!(len as u16)).to_le_bytes(), nlen);
            out.extend_from_slice(&z[pos + 5..pos + 5 + len]);
            pos += 5 + len;
        }
        assert_eq!(out, data);
        assert_eq!(z[pos..pos + 4], adler32(&data).to_be_bytes());
    }

    #[test]
    fn png_roundtrip_markers_and_pixels() {
        let mut c = Canvas::new(3, 2, Rgba(0, 0, 0, 0));
        c.set_pixel(1, 1, Rgba(10, 20, 30, 255));
        let png = encode_rgba(&c);
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        // IHDR: w=3 h=2 bitdepth 8 colortype 6
        assert_eq!(&png[16..20], &3u32.to_be_bytes());
        assert_eq!(&png[20..24], &2u32.to_be_bytes());
        assert_eq!(&png[24..27], &[8, 6, 0]);
        // Decode IDAT stored → pixel đúng vị trí (filter 0 mỗi dòng).
        let (_, idat) = find_idat(&png).expect("IDAT");
        let mut pos = 2usize;
        let mut raw = Vec::new();
        let mut last = false;
        while !last {
            last = idat[pos] & 1 == 1;
            let len = u16::from_le_bytes([idat[pos + 1], idat[pos + 2]]) as usize;
            raw.extend_from_slice(&idat[pos + 5..pos + 5 + len]);
            pos += 5 + len;
        }
        // dòng 1 (y=1), x=1 → offset trong raw = (1*(1+3*4)) + 1 + 1*4
        let idx = 13 + 1 + 4;
        assert_eq!(&raw[idx..idx + 4], &[10, 20, 30, 255]);
    }

    fn find_idat(png: &[u8]) -> Option<(usize, Vec<u8>)> {
        let mut pos = 8usize;
        while pos + 8 <= png.len() {
            let len =
                u32::from_be_bytes([png[pos], png[pos + 1], png[pos + 2], png[pos + 3]]) as usize;
            let kind = &png[pos + 4..pos + 8];
            if kind == b"IDAT" {
                return Some((pos, png[pos + 8..pos + 8 + len].to_vec()));
            }
            pos += 12 + len;
        }
        None
    }

    #[test]
    fn star_points_shape() {
        let pts = star_points(10.0, 10.0, 4.0, 0.38197);
        assert_eq!(pts.len(), 10);
        // đỉnh đầu hướng lên: y < 10
        assert!(pts[0].1 < 10.0);
        assert!((pts[0].1 - (10.0 - 4.0)).abs() < 1e-9);
    }

    #[test]
    fn point_in_poly_squares() {
        let sq = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        assert!(point_in_poly(5.0, 5.0, &sq));
        assert!(!point_in_poly(15.0, 5.0, &sq));
    }

    #[test]
    fn ico_container_layout() {
        let mut f1 = vec![0x89u8, b'P', b'N', b'G', 1, 2, 3];
        f1.resize(70, 9);
        let mut f2 = f1.clone();
        f2[8..11].copy_from_slice(&[77, 88, 99]);
        let ico = encode_ico(&[(16, 16, &f1), (32, 32, &f2)]);
        // header 6 + 2*16 = 38; entry đầu: w=16,h=16,planes=1,bpp=32
        assert_eq!(&ico[0..2], &0u16.to_le_bytes()); // reserved
        assert_eq!(&ico[2..4], &1u16.to_le_bytes());
        assert_eq!(ico[4], 2);
        assert_eq!(&ico[6..8], &[16, 16]); // entry 1: w,h
        let len1 = u32::from_le_bytes([ico[14], ico[15], ico[16], ico[17]]);
        assert_eq!(len1 as usize, f1.len());
        let off1 = u32::from_le_bytes([ico[18], ico[19], ico[20], ico[21]]);
        assert_eq!(off1 as usize, 38); // 6 + 2*16
        assert_eq!(&ico[38..38 + f1.len()], &f1[..]);
        assert_eq!(&ico[38 + f1.len()..], &f2[..]);
    }
}
