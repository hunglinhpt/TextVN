// SPDX-License-Identifier: GPL-3.0-or-later
//! E59: PNG encoder/decoder (RFC 1950/1951 thuần std) + canvas RGBA — thay Pillow
//! trong 3 script sinh icon/art (`generate_app_icons.py`, `generate_store_art.py`,
//! `generate_tray_icons.py`), đồng thời cung cấp `crc32` và `inflate_raw` cho
//! trình đọc ZIP/MSIX trong `win_tools.rs`.
//!
//! Nén PNG dùng bộ lọc Sub (filter 1, bpp=4) + LZ77 Fixed-Huffman DEFLATE
//! (`BTYPE=01`, tự rơi về `BTYPE=00` nếu dữ liệu không nén được), giữ file 1024px
//! và 2160px gọn xa dưới ngưỡng 1,5 MB của gate `repo-hygiene`.
//! Không unsafe, không crate ngoài.

// ---------------------------------------------------------------------------
//=== CRC-32 (PNG / ZIP) & Adler-32 (zlib) ====================================
// ---------------------------------------------------------------------------

pub(crate) fn crc32_update(mut crc: u32, data: &[u8]) -> u32 {
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    crc
}

pub(crate) fn crc32(data: &[u8]) -> u32 {
    !crc32_update(0xFFFF_FFFF, data)
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

/// Ghép 1 chunk PNG: len(4BE) + type + data + crc32(type+data) (streaming CRC, không copy).
fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    let len = u32::try_from(data.len()).expect("PNG chunk vượt quá u32::MAX");
    push_be32(out, len);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = !crc32_update(crc32_update(0xFFFF_FFFF, kind), data);
    push_be32(out, crc);
}

// ---------------------------------------------------------------------------
//=== RFC 1951 DEFLATE (Fixed-Huffman encoder + Full Stored/Fixed/Dynamic decoder)
// ---------------------------------------------------------------------------

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

struct BitWriter {
    out: Vec<u8>,
    buf: u64,
    nbits: u8,
}

impl BitWriter {
    fn new(cap: usize) -> Self {
        Self {
            out: Vec::with_capacity(cap),
            buf: 0,
            nbits: 0,
        }
    }
    #[inline]
    fn write_bits_lsb(&mut self, val: u32, count: u8) {
        self.buf |= (val as u64) << self.nbits;
        self.nbits += count;
        while self.nbits >= 8 {
            self.out.push((self.buf & 0xFF) as u8);
            self.buf >>= 8;
            self.nbits -= 8;
        }
    }
    #[inline]
    fn write_bits_msb(&mut self, val: u32, count: u8) {
        let rev = (val.reverse_bits()) >> (32 - count);
        self.write_bits_lsb(rev, count);
    }
    fn finish(mut self) -> Vec<u8> {
        if self.nbits > 0 {
            self.out.push((self.buf & 0xFF) as u8);
        }
        self.out
    }
}

#[inline]
fn write_fixed_litlen(bw: &mut BitWriter, sym: u16) {
    if sym <= 143 {
        bw.write_bits_msb(0x30 + sym as u32, 8);
    } else if sym <= 255 {
        bw.write_bits_msb(0x190 + (sym as u32 - 144), 9);
    } else if sym <= 279 {
        bw.write_bits_msb(sym as u32 - 256, 7);
    } else {
        bw.write_bits_msb(0xC0 + (sym as u32 - 280), 8);
    }
}

#[inline]
fn encode_len(len: usize) -> (u16, u32, u8) {
    if len == 258 {
        return (285, 0, 0);
    }
    let mut lo = 0usize;
    let mut hi = 28usize;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if LEN_BASE[mid] as usize <= len {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let extra = LEN_EXTRA[lo];
    (257 + lo as u16, (len - LEN_BASE[lo] as usize) as u32, extra)
}

#[inline]
fn encode_dist(dist: usize) -> (u32, u32, u8) {
    let mut lo = 0usize;
    let mut hi = DIST_BASE.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if DIST_BASE[mid] as usize <= dist {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let extra = DIST_EXTRA[lo];
    (lo as u32, (dist - DIST_BASE[lo] as usize) as u32, extra)
}

#[inline]
fn hash3(src: &[u8], pos: usize) -> usize {
    let v = (src[pos] as usize) | ((src[pos + 1] as usize) << 8) | ((src[pos + 2] as usize) << 16);
    ((v.wrapping_mul(2654435761)) >> 17) & 0x7FFF
}

/// Nén raw DEFLATE bằng LZ77 + Fixed Huffman (`BFINAL=1, BTYPE=01`, RFC 1951).
pub(crate) fn deflate_fixed(src: &[u8]) -> Vec<u8> {
    let mut bw = BitWriter::new((src.len() / 4).max(64));
    bw.write_bits_lsb(1, 1); // BFINAL = 1
    bw.write_bits_lsb(1, 2); // BTYPE = 01 (Fixed Huffman)

    let mut head = vec![u32::MAX; 1 << 15];
    let mut prev = vec![u32::MAX; 32_768];
    let n = src.len();
    let mut pos = 0usize;

    while pos < n {
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        if pos + 3 <= n {
            let h = hash3(src, pos);
            let mut cand = head[h];
            prev[pos & 0x7FFF] = cand;
            head[h] = pos as u32;

            let max_len = (n - pos).min(258);
            let mut steps = 0usize;
            while cand != u32::MAX && steps < 12 {
                let cpos = cand as usize;
                let dist = pos - cpos;
                if dist == 0 || dist > 32_768 {
                    break;
                }
                if src[cpos] == src[pos]
                    && src[cpos + 1] == src[pos + 1]
                    && src[cpos + 2] == src[pos + 2]
                    && (best_len < 3 || src[cpos + best_len - 1] == src[pos + best_len - 1])
                {
                    let mut m = 3;
                    while m < max_len && src[cpos + m] == src[pos + m] {
                        m += 1;
                    }
                    if m > best_len {
                        best_len = m;
                        best_dist = dist;
                        if best_len >= 64 {
                            break;
                        }
                    }
                }
                let next_cand = prev[cpos & 0x7FFF];
                if next_cand >= cand {
                    break;
                }
                cand = next_cand;
                steps += 1;
            }
        }
        if best_len >= 3 {
            let (lsym, lextra, lbits) = encode_len(best_len);
            write_fixed_litlen(&mut bw, lsym);
            if lbits > 0 {
                bw.write_bits_lsb(lextra, lbits);
            }
            let (dsym, dextra, dbits) = encode_dist(best_dist);
            bw.write_bits_msb(dsym, 5);
            if dbits > 0 {
                bw.write_bits_lsb(dextra, dbits);
            }
            // Cập nhật bảng băm cho các bước ngắn để giữ tỷ lệ nén tốt.
            let end = pos + best_len;
            if best_len < 32 {
                for p in (pos + 1)..end {
                    if p + 3 <= n {
                        let h = hash3(src, p);
                        prev[p & 0x7FFF] = head[h];
                        head[h] = p as u32;
                    }
                }
            }
            pos = end;
        } else {
            write_fixed_litlen(&mut bw, src[pos] as u16);
            pos += 1;
        }
    }
    write_fixed_litlen(&mut bw, 256); // EOB
    bw.finish()
}

/// zlib stream với DEFLATE "stored" blocks (BTYPE=00, LEN ≤ 65535/block).
fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    let mut z = Vec::with_capacity(raw.len() + raw.len() / 65_535 * 5 + 16);
    z.push(0x78); // CMF: deflate, 32K window
    z.push(0x01); // FLG: FLEVEL=0, check ok (0x7801 % 31 == 0)
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

/// zlib stream nén Fixed-Huffman DEFLATE (tự rơi về stored nếu dữ liệu không nén được).
fn zlib_compress(raw: &[u8]) -> Vec<u8> {
    let body = deflate_fixed(raw);
    let stored_est = raw.len() + (raw.len() / 65_535 + 1) * 5;
    if body.len() >= stored_est {
        return zlib_stored(raw);
    }
    let mut z = Vec::with_capacity(body.len() + 6);
    z.push(0x78); // CMF: deflate, 32K window
    z.push(0x9C); // FLG: default compression, check ok (0x789C % 31 == 0)
    z.extend_from_slice(&body);
    push_be32(&mut z, adler32(raw));
    z
}

struct BitReader<'a> {
    src: &'a [u8],
    pos: usize,
    buf: u64,
    nbits: u8,
}

impl<'a> BitReader<'a> {
    fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            buf: 0,
            nbits: 0,
        }
    }
    fn ensure(&mut self, count: u8) -> Result<(), String> {
        while self.nbits < count {
            let Some(&b) = self.src.get(self.pos) else {
                return Err("DEFLATE: hết dữ liệu sớm".into());
            };
            self.pos += 1;
            self.buf |= (b as u64) << self.nbits;
            self.nbits += 8;
        }
        Ok(())
    }
    fn read_bits(&mut self, count: u8) -> Result<u32, String> {
        if count == 0 {
            return Ok(0);
        }
        self.ensure(count)?;
        let mask = (1u64 << count) - 1;
        let val = (self.buf & mask) as u32;
        self.buf >>= count;
        self.nbits -= count;
        Ok(val)
    }
    fn align_byte(&mut self) {
        let drop = self.nbits % 8;
        self.buf >>= drop;
        self.nbits -= drop;
    }
    fn read_byte_aligned(&mut self) -> Result<u8, String> {
        if self.nbits >= 8 {
            let b = (self.buf & 0xFF) as u8;
            self.buf >>= 8;
            self.nbits -= 8;
            Ok(b)
        } else {
            let Some(&b) = self.src.get(self.pos) else {
                return Err("DEFLATE: hết byte trong stored block".into());
            };
            self.pos += 1;
            Ok(b)
        }
    }
    fn read_u16_aligned(&mut self) -> Result<u16, String> {
        let lo = self.read_byte_aligned()?;
        let hi = self.read_byte_aligned()?;
        Ok(u16::from_le_bytes([lo, hi]))
    }
}

struct HuffTable {
    count: [u16; 16],
    first_code: [u32; 16],
    first_sym: [u16; 16],
    symbols: Vec<u16>,
}

impl HuffTable {
    fn from_lengths(lengths: &[u8]) -> Result<Self, String> {
        let mut count = [0u16; 16];
        for &l in lengths {
            if l > 15 {
                return Err(format!("DEFLATE: độ dài mã Huffman > 15 ({l})"));
            }
            if l > 0 {
                count[l as usize] += 1;
            }
        }
        // Kiểm tra không oversubscribed (trừ bảng rỗng hoàn toàn cho dist không dùng).
        let total: u16 = count[1..].iter().sum();
        if total > 0 {
            let mut left: i32 = 1;
            for &c in &count[1..] {
                left = (left << 1) - c as i32;
                if left < 0 {
                    return Err("DEFLATE: bảng Huffman oversubscribed".into());
                }
            }
        }
        let mut first_code = [0u32; 16];
        let mut first_sym = [0u16; 16];
        let mut code = 0u32;
        let mut sym_off = 0u16;
        for bits in 1..=15 {
            code <<= 1;
            first_code[bits] = code;
            first_sym[bits] = sym_off;
            code += count[bits] as u32;
            sym_off += count[bits];
        }
        let mut offsets = first_sym;
        let mut symbols = vec![0u16; total as usize];
        for (sym, &l) in lengths.iter().enumerate() {
            if l > 0 {
                let idx = offsets[l as usize] as usize;
                symbols[idx] = sym as u16;
                offsets[l as usize] += 1;
            }
        }
        Ok(Self {
            count,
            first_code,
            first_sym,
            symbols,
        })
    }

    fn decode(&self, br: &mut BitReader<'_>) -> Result<u16, String> {
        let mut code = 0u32;
        for bits in 1..=15 {
            code = (code << 1) | br.read_bits(1)?;
            let cnt = self.count[bits] as u32;
            if cnt > 0 && code >= self.first_code[bits] && code - self.first_code[bits] < cnt {
                let idx = self.first_sym[bits] as usize + (code - self.first_code[bits]) as usize;
                return Ok(self.symbols[idx]);
            }
        }
        Err("DEFLATE: mã Huffman không hợp lệ".into())
    }
}

fn fixed_tables() -> (HuffTable, HuffTable) {
    let mut ll = [0u8; 288];
    for x in &mut ll[0..=143] {
        *x = 8;
    }
    for x in &mut ll[144..=255] {
        *x = 9;
    }
    for x in &mut ll[256..=279] {
        *x = 7;
    }
    for x in &mut ll[280..=287] {
        *x = 8;
    }
    let dist = [5u8; 32];
    (
        HuffTable::from_lengths(&ll).expect("fixed litlen hợp lệ"),
        HuffTable::from_lengths(&dist).expect("fixed dist hợp lệ"),
    )
}

/// Giải nén raw DEFLATE stream (RFC 1951 — hỗ trợ Stored, Fixed, Dynamic Huffman).
pub(crate) fn inflate_raw(src: &[u8]) -> Result<Vec<u8>, String> {
    const MAX_OUT: usize = 256 * 1024 * 1024;
    let mut br = BitReader::new(src);
    let mut out = Vec::new();
    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        match btype {
            0 => {
                br.align_byte();
                let len = br.read_u16_aligned()?;
                let nlen = br.read_u16_aligned()?;
                if len != !nlen {
                    return Err("DEFLATE: stored block LEN != !NLEN".into());
                }
                let n = len as usize;
                if out.len() + n > MAX_OUT {
                    return Err("DEFLATE: kích thước giải nén vượt giới hạn an toàn".into());
                }
                out.reserve(n);
                for _ in 0..n {
                    out.push(br.read_byte_aligned()?);
                }
            }
            1 | 2 => {
                let (ll_tbl, dist_tbl) = if btype == 1 {
                    fixed_tables()
                } else {
                    let hlit = br.read_bits(5)? as usize + 257;
                    let hdist = br.read_bits(5)? as usize + 1;
                    let hclen = br.read_bits(4)? as usize + 4;
                    if hlit > 286 || hdist > 30 {
                        return Err(format!("DEFLATE: HLIT={hlit}/HDIST={hdist} vượt chuẩn"));
                    }
                    let mut cl_lens = [0u8; 19];
                    for i in 0..hclen {
                        cl_lens[CL_ORDER[i]] = br.read_bits(3)? as u8;
                    }
                    let cl_tbl = HuffTable::from_lengths(&cl_lens)?;
                    let total = hlit + hdist;
                    let mut lens = Vec::with_capacity(total);
                    while lens.len() < total {
                        let sym = cl_tbl.decode(&mut br)?;
                        match sym {
                            0..=15 => lens.push(sym as u8),
                            16 => {
                                let prev =
                                    *lens.last().ok_or("DEFLATE: mã lặp 16 ở vị trí đầu bảng")?;
                                let rep = br.read_bits(2)? as usize + 3;
                                if lens.len() + rep > total {
                                    return Err("DEFLATE: mã lặp 16 vượt quá HLIT+HDIST".into());
                                }
                                lens.resize(lens.len() + rep, prev);
                            }
                            17 => {
                                let rep = br.read_bits(3)? as usize + 3;
                                if lens.len() + rep > total {
                                    return Err("DEFLATE: mã lặp 17 vượt quá HLIT+HDIST".into());
                                }
                                lens.resize(lens.len() + rep, 0);
                            }
                            18 => {
                                let rep = br.read_bits(7)? as usize + 11;
                                if lens.len() + rep > total {
                                    return Err("DEFLATE: mã lặp 18 vượt quá HLIT+HDIST".into());
                                }
                                lens.resize(lens.len() + rep, 0);
                            }
                            _ => return Err(format!("DEFLATE: mã code-length lạ {sym}")),
                        }
                    }
                    (
                        HuffTable::from_lengths(&lens[..hlit])?,
                        HuffTable::from_lengths(&lens[hlit..])?,
                    )
                };
                loop {
                    let sym = ll_tbl.decode(&mut br)?;
                    if sym < 256 {
                        if out.len() >= MAX_OUT {
                            return Err("DEFLATE: kích thước giải nén vượt giới hạn an toàn".into());
                        }
                        out.push(sym as u8);
                    } else if sym == 256 {
                        break;
                    } else {
                        let lidx = (sym as usize).checked_sub(257).unwrap_or(99);
                        if lidx >= LEN_BASE.len() {
                            return Err(format!("DEFLATE: length symbol lạ {sym}"));
                        }
                        let mlen =
                            LEN_BASE[lidx] as usize + br.read_bits(LEN_EXTRA[lidx])? as usize;
                        let dsym = dist_tbl.decode(&mut br)? as usize;
                        if dsym >= DIST_BASE.len() {
                            return Err(format!("DEFLATE: dist symbol lạ {dsym}"));
                        }
                        let dist =
                            DIST_BASE[dsym] as usize + br.read_bits(DIST_EXTRA[dsym])? as usize;
                        if dist == 0 || dist > out.len() {
                            return Err(format!(
                                "DEFLATE: khoảng cách {dist} vượt cửa sổ {}",
                                out.len()
                            ));
                        }
                        if out.len() + mlen > MAX_OUT {
                            return Err("DEFLATE: kích thước giải nén vượt giới hạn an toàn".into());
                        }
                        out.reserve(mlen);
                        for _ in 0..mlen {
                            let b = out[out.len() - dist];
                            out.push(b);
                        }
                    }
                }
            }
            _ => return Err("DEFLATE: BTYPE=3 không hợp lệ".into()),
        }
        if bfinal == 1 {
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) fn zlib_decompress(z: &[u8]) -> Result<Vec<u8>, String> {
    if z.len() < 6 {
        return Err("zlib: quá ngắn".into());
    }
    let cmf = z[0];
    let flg = z[1];
    if (cmf & 0x0F) != 8 || !(((cmf as u16) << 8) | flg as u16).is_multiple_of(31) {
        return Err("zlib: header CMF/FLG sai".into());
    }
    if (flg & 0x20) != 0 {
        return Err("zlib: không hỗ trợ preset dictionary".into());
    }
    let out = inflate_raw(&z[2..z.len() - 4])?;
    let want = u32::from_be_bytes([
        z[z.len() - 4],
        z[z.len() - 3],
        z[z.len() - 2],
        z[z.len() - 1],
    ]);
    let got = adler32(&out);
    if got != want {
        return Err(format!(
            "zlib: Adler-32 lệch (got 0x{got:08X} != want 0x{want:08X})"
        ));
    }
    Ok(out)
}

/// Encode canvas RGBA 8-bit/kênh (PNG Sub filter + LZ77 Fixed-Huffman zlib) → bytes PNG.
pub fn encode_rgba(canvas: &Canvas) -> Vec<u8> {
    let (w, h) = (canvas.w, canvas.h);
    assert!(w > 0 && h > 0, "PNG yêu cầu kích thước > 0 ({w}x{h})");
    // Filter 1 (Sub, bpp=4): hiệu quả cao với ảnh phẳng/gradient/icon bo góc.
    let stride = w * 4;
    let mut raw = Vec::with_capacity(h * (1 + stride));
    for y in 0..h {
        raw.push(1u8); // Filter type 1 = Sub
        let row = &canvas.px[y * stride..(y + 1) * stride];
        for i in 0..stride {
            let left = if i >= 4 { row[i - 4] } else { 0 };
            raw.push(row[i].wrapping_sub(left));
        }
    }
    let z = zlib_compress(&raw);
    let mut png = Vec::with_capacity(z.len() + 128);
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

    fn clamp_bounds(
        &self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    ) -> Option<(isize, isize, isize, isize)> {
        if self.w == 0 || self.h == 0 || x1 < x0 || y1 < y0 {
            return None;
        }
        let xa = (x0.floor() as isize).max(0);
        let ya = (y0.floor() as isize).max(0);
        let xb = (x1.floor() as isize).min(self.w as isize - 1);
        let yb = (y1.floor() as isize).min(self.h as isize - 1);
        if xa > xb || ya > yb {
            None
        } else {
            Some((xa, ya, xb, yb))
        }
    }

    /// Rect đặc (x0,y0)-(x1,y1) inclusive — bắt đầu từ `.floor()` để không cắt hàng/cột đầu.
    pub fn fill_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Rgba) {
        let Some((xa, ya, xb, yb)) = self.clamp_bounds(x0, y0, x1, y1) else {
            return;
        };
        for y in ya..=yb {
            for x in xa..=xb {
                self.blend_pixel(x, y, c);
            }
        }
    }

    /// Rounded-rect fill (toạ độ PIL [x0,y0,x1,y1], radius) — anti-alias bằng
    /// coverage 4×4 subsample (bắt đầu từ `.floor()` để giữ AA mép đầu).
    pub fn rounded_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, radius: f64, c: Rgba) {
        let Some((xa, ya, xb, yb)) = self.clamp_bounds(x0, y0, x1, y1) else {
            return;
        };
        for y in ya..=yb {
            for x in xa..=xb {
                let cov = rounded_rect_coverage(x as f64, y as f64, x0, y0, x1, y1, radius);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
            }
        }
    }

    /// Polygon fill (scanline even-odd) với 4×4 subsample AA; points hệ (x,y).
    pub fn polygon(&mut self, pts: &[(f64, f64)], c: Rgba) {
        if pts.len() < 3 {
            return;
        }
        let (min_x, max_x, min_y, max_y) = bbox(pts);
        let Some((xa, ya, xb, yb)) = self.clamp_bounds(min_x, min_y, max_x, max_y) else {
            return;
        };
        for y in ya..=yb {
            for x in xa..=xb {
                let cov = polygon_coverage(x as f64, y as f64, pts);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
            }
        }
    }

    /// Ellipse fill (bounding box [x0,y0,x1,y1]) — subsample AA.
    pub fn ellipse(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Rgba) {
        let Some((xa, ya, xb, yb)) = self.clamp_bounds(x0, y0, x1, y1) else {
            return;
        };
        for y in ya..=yb {
            for x in xa..=xb {
                let cov = ellipse_coverage(x as f64, y as f64, x0, y0, x1, y1);
                if cov > 0.0 {
                    let ca = (c.3 as f64 * cov).round() as u8;
                    self.blend_pixel(x, y, Rgba(c.0, c.1, c.2, ca));
                }
            }
        }
    }

    /// Nền gradient chéo từ (0,0)→(w-1,h-1) — thay `diag_gradient()` PIL.
    pub fn gradient_bg(&mut self, top: Rgba, bottom: Rgba) {
        let diag = ((self.w + self.h) as f64 - 1.0).max(1.0);
        for y in 0..self.h {
            for x in 0..self.w {
                let t = ((x + y) as f64 / diag).clamp(0.0, 1.0);
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

    /// Vẽ đè canvas khác qua mask alpha (kênh Alpha `* 4 + 3` của `mask`).
    pub fn paste_with_mask(&mut self, src: &Canvas, mask: &Canvas, dy: isize) {
        for y in 0..src.h.min(mask.h) {
            let ty = y as isize + dy;
            if ty < 0 || ty as usize >= self.h {
                continue;
            }
            for x in 0..src.w.min(mask.w).min(self.w) {
                let alpha = mask.px[(y * mask.w + x) * 4 + 3];
                if alpha == 0 {
                    continue;
                }
                let s = src.get_pixel(x, y);
                let out_a = ((alpha as u32 * s.3 as u32 + 127) / 255) as u8;
                if out_a == 0 {
                    continue;
                }
                self.blend_pixel(x as isize, ty, Rgba(s.0, s.1, s.2, out_a));
            }
        }
    }

    /// Downsample /2 chính xác (box 2×2 mean premultiplied không mất độ chính xác
    /// trên pixel alpha thấp) — thay LANCZOS resize cho các bước chia hết 2.
    pub fn downsample2(self) -> Canvas {
        let w = self.w / 2;
        let h = self.h / 2;
        let mut out = Canvas::new(w, h, Rgba(0, 0, 0, 0));
        for y in 0..h {
            for x in 0..w {
                let mut pre = [0u32; 4]; // r*a, g*a, b*a, a (max 4*255*255 = 260_100)
                for (dy, dx) in [(0usize, 0usize), (0, 1), (1, 0), (1, 1)] {
                    let p = self.get_pixel(x * 2 + dx, y * 2 + dy);
                    let a = p.3 as u32;
                    pre[0] += p.0 as u32 * a;
                    pre[1] += p.1 as u32 * a;
                    pre[2] += p.2 as u32 * a;
                    pre[3] += a;
                }
                let a = pre[3];
                let unpre =
                    |c: u32| -> u8 { (c + a / 2).checked_div(a).unwrap_or(0).min(255) as u8 };
                out.set_pixel(
                    x,
                    y,
                    Rgba(
                        unpre(pre[0]),
                        unpre(pre[1]),
                        unpre(pre[2]),
                        ((a + 2) / 4).min(255) as u8,
                    ),
                );
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
    if r == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    // Horizontal rồi vertical — tích lũy chạy (sliding window).
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = y * w;
        let mut acc: u32 = 0;
        for x in 0..r.min(w) {
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
        for y in 0..r.min(h) {
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
    if x1 <= x0 || y1 <= y0 {
        return 0.0;
    }
    let radius = radius.clamp(0.0, ((x1 - x0) * 0.5).min((y1 - y0) * 0.5));
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
    let n = pts.len();
    if n < 3 {
        return false;
    }
    // even-odd ray-cast chuẩn.
    let mut inside = false;
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
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn adler32_known_vector() {
        assert_eq!(adler32(b"123456789"), 0x091E_01DE);
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn zlib_stored_and_deflate_roundtrip() {
        for data in [
            Vec::new(),
            b"hello world! hello world! hello world!".to_vec(),
            vec![7u8; 200_000],
            (0..5000u32).map(|i| ((i * 73 + 19) & 0xFF) as u8).collect(),
        ] {
            let zs = zlib_stored(&data);
            assert_eq!(zlib_decompress(&zs).unwrap(), data);
            let zc = zlib_compress(&data);
            assert_eq!(zlib_decompress(&zc).unwrap(), data);
            let raw_def = deflate_fixed(&data);
            assert_eq!(inflate_raw(&raw_def).unwrap(), data);
        }
    }

    fn decode_png_rgba(png: &[u8]) -> (usize, usize, Vec<u8>) {
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        let mut pos = 8usize;
        let mut w = 0usize;
        let mut h = 0usize;
        let mut idat = Vec::new();
        while pos + 12 <= png.len() {
            let len =
                u32::from_be_bytes([png[pos], png[pos + 1], png[pos + 2], png[pos + 3]]) as usize;
            let kind = &png[pos + 4..pos + 8];
            let body = &png[pos + 8..pos + 8 + len];
            let got_crc = u32::from_be_bytes([
                png[pos + 8 + len],
                png[pos + 9 + len],
                png[pos + 10 + len],
                png[pos + 11 + len],
            ]);
            let want_crc = !crc32_update(crc32_update(0xFFFF_FFFF, kind), body);
            assert_eq!(got_crc, want_crc, "CRC sai cho chunk {:?}", kind);
            if kind == b"IHDR" {
                w = u32::from_be_bytes([body[0], body[1], body[2], body[3]]) as usize;
                h = u32::from_be_bytes([body[4], body[5], body[6], body[7]]) as usize;
            } else if kind == b"IDAT" {
                idat.extend_from_slice(body);
            }
            pos += 12 + len;
        }
        let raw = zlib_decompress(&idat).expect("zlib decode");
        let stride = w * 4;
        let mut px = vec![0u8; h * stride];
        for y in 0..h {
            let filter = raw[y * (1 + stride)];
            let src_row = &raw[y * (1 + stride) + 1..(y + 1) * (1 + stride)];
            let dst_row = &mut px[y * stride..(y + 1) * stride];
            match filter {
                0 => dst_row.copy_from_slice(src_row),
                1 => {
                    for i in 0..stride {
                        let left = if i >= 4 { dst_row[i - 4] } else { 0 };
                        dst_row[i] = src_row[i].wrapping_add(left);
                    }
                }
                other => panic!("filter không hỗ trợ trong test: {other}"),
            }
        }
        (w, h, px)
    }

    #[test]
    fn png_roundtrip_markers_and_pixels() {
        let mut c = Canvas::new(16, 12, Rgba(10, 20, 30, 255));
        c.gradient_bg(Rgba::rgb(218, 37, 29), Rgba::rgb(150, 18, 14));
        c.set_pixel(1, 1, Rgba(10, 20, 30, 255));
        let png = encode_rgba(&c);
        let (w, h, px) = decode_png_rgba(&png);
        assert_eq!((w, h), (16, 12));
        assert_eq!(px, c.px);
    }

    #[test]
    fn paste_with_mask_doc_kenh_alpha_va_khong_panic_lech_chieu_cao() {
        // Mask màu đen (R=0) nhưng Alpha=128 — trước đây đọc R=0 làm bóng đổ biến mất hoàn toàn.
        let src = Canvas::new(4, 4, Rgba(200, 100, 50, 255));
        let mask = Canvas::new(4, 2, Rgba(0, 0, 0, 128)); // mask.h < src.h
        let mut dst = Canvas::new(4, 4, Rgba(0, 0, 0, 0));
        dst.paste_with_mask(&src, &mask, 0);
        let p = dst.get_pixel(1, 1);
        assert!(p.3 >= 127 && p.3 <= 129, "phải đọc kênh alpha mask: {p:?}");
        assert_eq!(dst.get_pixel(1, 3).3, 0, "ngoài mask.h phải giữ trong suốt");
    }

    #[test]
    fn rounded_rect_va_rasterizer_khong_panic_va_giu_hang_dau() {
        // radius > min(w,h)/2 và rect rỗng không được panic trong f64::clamp.
        assert_eq!(
            rounded_rect_coverage(0.0, 0.0, 5.0, 5.0, 2.0, 2.0, 10.0),
            0.0
        );
        let cov = rounded_rect_coverage(2.0, 2.0, 1.0, 1.0, 4.0, 4.0, 99.0);
        assert!(cov > 0.0);
        // Toạ độ phân số [3.2, 4.8] phải vẽ cả pixel x=3 (trước đây .ceil() bỏ qua x=3).
        let mut c = Canvas::new(8, 8, Rgba(0, 0, 0, 0));
        c.ellipse(3.2, 3.2, 5.8, 5.8, Rgba(255, 255, 255, 255));
        assert!(c.get_pixel(3, 4).3 > 0, "mép trái phân số phải có AA");
        // point_in_poly rỗng và gradient 1x1 không panic.
        assert!(!point_in_poly(0.0, 0.0, &[]));
        let mut g1 = Canvas::new(1, 1, Rgba(0, 0, 0, 0));
        g1.gradient_bg(Rgba::rgb(10, 20, 30), Rgba::rgb(40, 50, 60));
        assert_eq!(g1.get_pixel(0, 0), Rgba(10, 20, 30, 255));
    }

    #[test]
    fn downsample2_chinh_xac_alpha_thap() {
        let mut c = Canvas::new(2, 2, Rgba(0, 0, 0, 0));
        c.set_pixel(0, 0, Rgba(200, 100, 50, 7));
        let d = c.downsample2();
        let p = d.get_pixel(0, 0);
        assert_eq!(p, Rgba(200, 100, 50, 2));
    }

    #[test]
    fn star_points_shape() {
        let pts = star_points(10.0, 10.0, 4.0, 0.38197);
        assert_eq!(pts.len(), 10);
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
        assert_eq!(&ico[0..2], &0u16.to_le_bytes());
        assert_eq!(&ico[2..4], &1u16.to_le_bytes());
        assert_eq!(ico[4], 2);
        assert_eq!(&ico[6..8], &[16, 16]);
        let len1 = u32::from_le_bytes([ico[14], ico[15], ico[16], ico[17]]);
        assert_eq!(len1 as usize, f1.len());
        let off1 = u32::from_le_bytes([ico[18], ico[19], ico[20], ico[21]]);
        assert_eq!(off1 as usize, 38);
        assert_eq!(&ico[38..38 + f1.len()], &f1[..]);
        assert_eq!(&ico[38 + f1.len()..], &f2[..]);
    }
}
