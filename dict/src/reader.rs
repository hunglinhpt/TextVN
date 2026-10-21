// SPDX-License-Identifier: GPL-3.0-or-later
//! Doc FST v1 tu mot buffer (mmap hoac in-memory) - **khong copy** du lieu.
//!
//! Reader chi giu `&[u8]` (buffer mmap) -> tra ket qua nhanh, khong alloc
//! (chi alloc khi goi y - [`crate::fst::Reader::final_word`]).

use crate::fst::{decode_node, decode_trans, get_u32, get_u64, read_header, Header, TRANS_LEN};

/// Reader FST tren buffer bat bien.
#[derive(Debug, Clone, Copy)]
pub struct Reader<'a> {
    buf: &'a [u8],
    header: Header,
}

impl<'a> Reader<'a> {
    /// Tao reader tu buffer. Tra `None` neu header sai/format khong ho tro.
    pub fn new(buf: &'a [u8]) -> Option<Self> {
        let header = read_header(buf)?;
        Some(Self { buf, header })
    }

    /// Header cua file.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// So tu trong tu dien.
    pub fn len(&self) -> usize {
        self.header.final_count as usize
    }

    /// `true` neu tu dien rong.
    pub fn is_empty(&self) -> bool {
        self.header.final_count == 0
    }

    /// Kiem tra checksum cua file (FNV-1a tren phan du lieu).
    pub fn verify_checksum(&self) -> bool {
        let Some(data) = self.buf.get(crate::fst::HEADER_LEN..) else {
            return false;
        };
        let Some(expected) = get_u64(self.buf, 0x50) else {
            return false;
        };
        crate::fst::fnv1a64(data) == expected
    }

    /// Node `id` -> (final, transition range).
    fn node(&self, id: u32) -> Option<(bool, u32, u32)> {
        let off = self.header.offset_nodes as usize + id as usize * 8;
        let raw = get_u64(self.buf, off)?;
        let node = decode_node(raw);
        Some((node.is_final, node.trans_offset, node.trans_count))
    }

    /// Tim transition theo ky tu tu node `id` (binary search).
    fn find(&self, id: u32, ch: char) -> Option<u32> {
        let (_, trans_off, trans_count) = self.node(id)?;
        let mut lo = 0u32;
        let mut hi = trans_count;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let idx = trans_off + mid;
            let t_off = self.header.offset_trans_table as usize + idx as usize * 4;
            let data_off = get_u32(self.buf, t_off)? as usize;
            let trans = decode_trans(self.buf, self.header.offset_trans_data as usize + data_off)?;
            match trans.ch.cmp(&ch) {
                std::cmp::Ordering::Equal => return Some(trans.target),
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
            }
        }
        None
    }

    /// `true` neu `word` co trong tu dien.
    pub fn contains(&self, word: &str) -> bool {
        let mut cur = 0u32;
        for ch in word.chars() {
            match self.find(cur, ch) {
                Some(next) => cur = next,
                None => return false,
            }
        }
        matches!(self.node(cur), Some((true, _, _)))
    }

    /// Chuoi tu cua node final (dung cho goi y).
    pub fn final_word(&self, id: u32) -> Option<&'a str> {
        let (is_final, _, _) = self.node(id)?;
        if !is_final {
            return None;
        }
        let t_off = self.header.offset_final_table as usize + id as usize * 4;
        let data_off = get_u32(self.buf, t_off)? as usize;
        let start = self.header.offset_final_data as usize + data_off;
        let len = get_u32(self.buf, start)? as usize;
        let bytes = self.buf.get(start + 4..start + 4 + len)?;
        std::str::from_utf8(bytes).ok()
    }

    /// Duyet tat ca node final, goi callback voi (id, tu).
    pub fn for_each_final(&self, mut f: impl FnMut(u32, &str)) {
        for id in 0..self.header.node_count as u32 {
            if let Some((true, _, _)) = self.node(id) {
                if let Some(word) = self.final_word(id) {
                    f(id, word);
                }
            }
        }
    }
}

/// Kich thuoc 1 transition (byte) - export de test.
pub const TRANS_SIZE: usize = TRANS_LEN;
