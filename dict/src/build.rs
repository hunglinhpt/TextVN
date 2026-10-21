// SPDX-License-Identifier: GPL-3.0-or-later
//! Builder FST: tu danh sach tu (UTF-8) -> file FST v1 byte-identical.
//!
//! Builder **deterministic**: cung input -> cung bytes (khong con timestamp
//! trong file). `xtask gen-dict` dua file `.fst` vao repo -> phai sinh lai
//! duoc byte-for-byte.

use std::collections::BTreeMap;

use crate::fst::{
    encode_node, encode_trans, fnv1a64, put_u32, put_u64, Node, Transition, HEADER_LEN, MAGIC,
    TRANS_LEN, VERSION,
};

/// Node trong trie (khi build).
#[derive(Debug, Default)]
struct BuildNode {
    /// Node con: ky tu -> id node con.
    children: BTreeMap<char, u32>,
    /// Neu node la final: chuoi tu day.
    word: Option<String>,
}

/// Builder trie cap ky tu.
#[derive(Debug, Default)]
pub struct Builder {
    /// Cac node (node 0 = root).
    nodes: Vec<BuildNode>,
}

impl Builder {
    /// Tao builder rong.
    pub fn new() -> Self {
        Self {
            nodes: vec![BuildNode::default()],
        }
    }

    /// So node hien tai.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Chen 1 tu vao trie. Tu rong bi bo qua.
    pub fn insert(&mut self, word: &str) {
        if word.is_empty() {
            return;
        }
        let mut cur = 0usize;
        for ch in word.chars() {
            let existing = self.nodes[cur].children.get(&ch).copied();
            let next = match existing {
                Some(id) => id,
                None => {
                    let id = self.nodes.len() as u32;
                    self.nodes.push(BuildNode::default());
                    self.nodes[cur].children.insert(ch, id);
                    id
                }
            };
            cur = next as usize;
        }
        self.nodes[cur].word = Some(word.to_string());
    }

    /// Chen nhieu tu.
    pub fn extend<I, S>(&mut self, words: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for w in words {
            self.insert(w.as_ref());
        }
    }

    /// Serialize trie thanh bytes file FST v1 (deterministic).
    pub fn to_bytes(&self) -> Vec<u8> {
        let node_count = self.nodes.len();
        let trans_count: usize = self.nodes.iter().map(|n| n.children.len()).sum();

        let offset_nodes = HEADER_LEN as u64;
        let offset_trans_table = offset_nodes + node_count as u64 * 8;
        let offset_trans_data = offset_trans_table + trans_count as u64 * 4;
        let offset_final_table = offset_trans_data + trans_count as u64 * TRANS_LEN as u64;
        let offset_final_data = offset_final_table + node_count as u64 * 4;

        // `final_data`: chuoi tu cua node final.
        let mut final_data: Vec<u8> = Vec::new();
        let mut final_offsets: Vec<u32> = vec![0; node_count];
        for (i, node) in self.nodes.iter().enumerate() {
            final_offsets[i] = final_data.len() as u32;
            if let Some(word) = &node.word {
                final_data.extend_from_slice(&(word.len() as u32).to_le_bytes());
                final_data.extend_from_slice(word.as_bytes());
            }
        }

        let total_len = offset_final_data + final_data.len() as u64;
        let mut out = vec![0u8; total_len as usize];

        put_u64(&mut out, 0, MAGIC);
        put_u64(&mut out, 8, VERSION);
        put_u64(&mut out, 0x10, node_count as u64);
        put_u64(&mut out, 0x18, trans_count as u64);
        put_u64(
            &mut out,
            0x20,
            self.nodes.iter().filter(|n| n.word.is_some()).count() as u64,
        );
        put_u64(&mut out, 0x28, offset_nodes);
        put_u64(&mut out, 0x30, offset_trans_table);
        put_u64(&mut out, 0x38, offset_trans_data);
        put_u64(&mut out, 0x40, offset_final_table);
        put_u64(&mut out, 0x48, offset_final_data);

        // Nodes + transition data.
        let mut trans_index = 0usize;
        for (i, node) in self.nodes.iter().enumerate() {
            let trans_off = trans_index as u32;
            for (&ch, &target) in &node.children {
                let bytes = encode_trans(&Transition { target, ch });
                let data_off = offset_trans_data as usize + trans_index * TRANS_LEN;
                out[data_off..data_off + TRANS_LEN].copy_from_slice(&bytes);
                trans_index += 1;
            }
            let encoded = encode_node(&Node {
                is_final: node.word.is_some(),
                trans_count: node.children.len() as u32,
                trans_offset: trans_off,
            });
            put_u64(&mut out, offset_nodes as usize + i * 8, encoded);
        }
        debug_assert_eq!(trans_index, trans_count);

        // `trans_table`: offset byte trong `trans_data`.
        trans_index = 0;
        for node in &self.nodes {
            for _ in &node.children {
                let off = (trans_index * TRANS_LEN) as u32;
                put_u32(&mut out, offset_trans_table as usize + trans_index * 4, off);
                trans_index += 1;
            }
        }

        // `final_table` + `final_data`.
        for (i, &off) in final_offsets.iter().enumerate() {
            put_u32(&mut out, offset_final_table as usize + i * 4, off);
        }
        let fd_off = offset_final_data as usize;
        out[fd_off..fd_off + final_data.len()].copy_from_slice(&final_data);

        let checksum = fnv1a64(&out[HEADER_LEN..]);
        put_u64(&mut out, 0x50, checksum);
        out
    }
}
