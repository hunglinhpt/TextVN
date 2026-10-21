// SPDX-License-Identifier: GPL-3.0-or-later
//! Format tu dien **FST** (v1) - doc/sinh thuan Rust, 0 dependency.
//!
//! Tu dien Viet (AS-040) la mot **trie cap ky tu** (char-level trie): moi
//! node la mot `u64` little-endian:
//!
//! ```text
//! bit  63    : final (tu ket thuc tai node nay)
//! bit 32..62 : offset vao bang `trans_table` (mang `u32`)
//! bit  0..31 : so transition cua node
//! ```
//!
//! Moi transition (12 byte) trong `trans_data`:
//!
//! ```text
//! u32 : node dich
//! u8  : so byte cua ky tu (1..=4)
//! u8  : byte 0..3 cua ky tu (UTF-8)
//! u32 : reserved (0)
//! ```
//!
//! Transition cua **mot node luon sorted theo ky tu** (tang dan) -> tim bang
//! **binary search** tren buffer mmap.
//!
//! Node final tro toi bang `final_table` (**song song voi node**, `u32`/node ->
//! byte offset trong `final_data`) -> `final_data`: moi muc la `u32 len` +
//! `len` byte (chuoi UTF-8 cua tu) - can cho goi y.
//!
//! File layout:
//!
//! ```text
//! 0x00 u64  MAGIC   = "TVNFST01"
//! 0x08 u64  version = 1
//! 0x10 u64  node_count
//! 0x18 u64  trans_count
//! 0x20 u64  final_count
//! 0x28 u64  offset_nodes        (node_count * 8 byte)
//! 0x30 u64  offset_trans_table  (trans_count * 4 byte)
//! 0x38 u64  offset_trans_data   (trans_count * 12 byte)
//! 0x40 u64  offset_final_table  (final_count * 4 byte)
//! 0x48 u64  offset_final_data
//! 0x50 u64  checksum (FNV-1a 64 cua [0x58 .. EOF))
//! 0x58 ... du lieu
//! ```

/// Magic number cua file FST v1 (`"TVNFST01"`).
pub const MAGIC: u64 = 0x5456_4e46_5354_3031;
/// Phien ban format hien tai.
pub const VERSION: u64 = 1;
/// Do dai header (byte) - du lieu bat dau tu day.
pub const HEADER_LEN: usize = 0x58;

/// Kich thuoc mot transition (byte).
pub const TRANS_LEN: usize = 12;

/// Header cua file FST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// So node trong trie.
    pub node_count: u64,
    /// So transition.
    pub trans_count: u64,
    /// So tu (final).
    pub final_count: u64,
    /// Offset mang node (`u64`/node).
    pub offset_nodes: u64,
    /// Offset mang `trans_table` (`u32`/transition).
    pub offset_trans_table: u64,
    /// Offset du lieu transition.
    pub offset_trans_data: u64,
    /// Offset mang `final_table` (`u32`/final).
    pub offset_final_table: u64,
    /// Offset du lieu final.
    pub offset_final_data: u64,
}

/// Header doc duoc tu dau buffer.
pub fn read_header(buf: &[u8]) -> Option<Header> {
    if buf.len() < HEADER_LEN {
        return None;
    }
    if get_u64(buf, 0)? != MAGIC {
        return None;
    }
    if get_u64(buf, 8)? != VERSION {
        return None;
    }
    Some(Header {
        node_count: get_u64(buf, 0x10)?,
        trans_count: get_u64(buf, 0x18)?,
        final_count: get_u64(buf, 0x20)?,
        offset_nodes: get_u64(buf, 0x28)?,
        offset_trans_table: get_u64(buf, 0x30)?,
        offset_trans_data: get_u64(buf, 0x38)?,
        offset_final_table: get_u64(buf, 0x40)?,
        offset_final_data: get_u64(buf, 0x48)?,
    })
}

/// Doc `u64` little-endian tai `off`.
pub fn get_u64(buf: &[u8], off: usize) -> Option<u64> {
    let end = off.checked_add(8)?;
    Some(u64::from_le_bytes(buf.get(off..end)?.try_into().ok()?))
}

/// Doc `u32` little-endian tai `off`.
pub fn get_u32(buf: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    Some(u32::from_le_bytes(buf.get(off..end)?.try_into().ok()?))
}

/// Ghi `u64` little-endian vao buffer.
pub fn put_u64(buf: &mut [u8], off: usize, value: u64) {
    buf[off..off + 8].copy_from_slice(&value.to_le_bytes());
}

/// Ghi `u32` little-endian vao buffer.
pub fn put_u32(buf: &mut [u8], off: usize, value: u32) {
    buf[off..off + 4].copy_from_slice(&value.to_le_bytes());
}

/// FNV-1a 64 - checksum nhanh, khong can dependency.
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

/// Mot node trong trie (giai ma tu `u64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    /// Node nay la final (mot tu ket thuc tai day)?
    pub is_final: bool,
    /// So transition cua node.
    pub trans_count: u32,
    /// Offset vao `trans_table`.
    pub trans_offset: u32,
}

/// Giai ma 1 node tu `u64` raw.
pub fn decode_node(raw: u64) -> Node {
    Node {
        is_final: raw & (1 << 63) != 0,
        trans_count: (raw & 0xffff_ffff) as u32,
        trans_offset: ((raw >> 32) & 0x7fff_ffff) as u32,
    }
}

/// Dong goi 1 node thanh `u64` raw.
pub fn encode_node(node: &Node) -> u64 {
    let mut raw = node.trans_count as u64;
    raw |= ((node.trans_offset as u64) & 0x7fff_ffff) << 32;
    if node.is_final {
        raw |= 1 << 63;
    }
    raw
}

/// Mot transition (giai ma tu 12 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// Node dich.
    pub target: u32,
    /// Ky tu (UTF-8, <= 4 byte).
    pub ch: char,
}

/// Giai ma transition tai `off` trong `trans_data`.
pub fn decode_trans(buf: &[u8], off: usize) -> Option<Transition> {
    let target = get_u32(buf, off)?;
    let slice = buf.get(off + 4..off + 8)?;
    let len = slice[0] as usize;
    if len == 0 || len > 4 {
        return None;
    }
    let s = std::str::from_utf8(slice.get(1..1 + len)?).ok()?;
    let ch = s.chars().next()?;
    Some(Transition { target, ch })
}

/// Dong goi 1 transition thanh 12 byte.
pub fn encode_trans(trans: &Transition) -> [u8; TRANS_LEN] {
    let mut out = [0u8; TRANS_LEN];
    out[0..4].copy_from_slice(&trans.target.to_le_bytes());
    let mut bytes = [0u8; 4];
    let len = trans.ch.len_utf8();
    out[4] = len as u8;
    out[5..5 + len].copy_from_slice(trans.ch.encode_utf8(&mut bytes).as_bytes());
    out
}
