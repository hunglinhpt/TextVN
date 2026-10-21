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

    /// Transition thu `idx` cua node (0..trans_count).
    fn transition(&self, trans_offset: u32, idx: u32) -> Option<crate::fst::Transition> {
        let t_idx = trans_offset + idx;
        let t_off = self.header.offset_trans_table as usize + t_idx as usize * 4;
        let data_off = get_u32(self.buf, t_off)? as usize;
        decode_trans(self.buf, self.header.offset_trans_data as usize + data_off)
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

    /// Goi y cac tu thay the cho `word` (AS-041), toi da `limit` ket qua.
    ///
    /// Thuat toan:
    /// 1. Tim kiem theo khung am tiet (skeleton match): duyet cac tu cung chu
    ///    cai co so, phat hien va sua triet de loi sai/thieu dau thanh.
    /// 2. Duyet cay Trie bang thuat toan Damerau-Levenshtein edit distance voi
    ///    co che cat nhanh (pruning) som cho cac loi go phim (chen, xoa, sua,
    ///    dao chu).
    /// 3. Xep hang ung vien: cung khung am tiet len dau -> cung dau thanh ->
    ///    edit distance nho -> do dai gan nhat -> on dinh alphabet.
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        if word.is_empty() || limit == 0 || self.is_empty() {
            return Vec::new();
        }
        let target_chars: Vec<char> = word.chars().collect();
        let target_len = target_chars.len();
        if target_len > 32 {
            return Vec::new();
        }
        let target_skeleton = base_skeleton(word);
        let target_skeleton_chars: Vec<char> = target_skeleton.chars().collect();
        let target_tone_stripped = strip_tone_word(word);
        let target_tone = word_tone(word);

        let mut candidates: Vec<Candidate> = Vec::new();

        // Phase 1: Tim tat ca bien the cung khung am tiet (skeleton search)
        self.search_skeleton(
            0,
            0,
            &target_skeleton_chars,
            word,
            &target_tone_stripped,
            target_tone,
            &mut candidates,
        );

        // Phase 2: Tim theo Damerau-Levenshtein edit distance <= 1
        let init_row: Vec<usize> = (0..=target_len).collect();
        self.search_dfs(
            0,
            0,
            &target_chars,
            word,
            &target_skeleton,
            &target_tone_stripped,
            target_tone,
            &init_row,
            &[],
            None,
            1,
            &mut candidates,
        );

        // Phase 3: Neu chua du limit va tu >= 3 ky tu, mo rong edit distance <= 2
        if candidates.len() < limit && target_len >= 3 {
            self.search_dfs(
                0,
                0,
                &target_chars,
                word,
                &target_skeleton,
                &target_tone_stripped,
                target_tone,
                &init_row,
                &[],
                None,
                2,
                &mut candidates,
            );
        }

        // Xep hang cac ung vien
        candidates.sort_by(|a, b| {
            b.same_vowel_base
                .cmp(&a.same_vowel_base)
                .then_with(|| b.same_skeleton.cmp(&a.same_skeleton))
                .then_with(|| b.same_tone.cmp(&a.same_tone))
                .then_with(|| a.dist.cmp(&b.dist))
                .then_with(|| {
                    let diff_a = (a.word.chars().count() as isize - target_len as isize).abs();
                    let diff_b = (b.word.chars().count() as isize - target_len as isize).abs();
                    diff_a.cmp(&diff_b)
                })
                .then_with(|| a.word.cmp(&b.word))
        });

        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::with_capacity(limit);
        for c in candidates {
            let formatted = match_case(&c.word, word);
            if seen.insert(formatted.clone()) {
                out.push(formatted);
                if out.len() >= limit {
                    break;
                }
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn search_skeleton(
        &self,
        node_id: u32,
        depth: usize,
        target_skeleton_chars: &[char],
        target_str: &str,
        target_tone_stripped: &str,
        target_tone: usize,
        candidates: &mut Vec<Candidate>,
    ) {
        let Some((is_final, trans_off, trans_count)) = self.node(node_id) else {
            return;
        };

        if depth == target_skeleton_chars.len() {
            if is_final {
                if let Some(word) = self.final_word(node_id) {
                    if word != target_str {
                        let tone = word_tone(word);
                        let same_tone = target_tone > 0 && tone == target_tone;
                        let same_vowel_base = strip_tone_word(word) == target_tone_stripped;
                        let dist = if same_vowel_base || same_tone {
                            1
                        } else {
                            word.chars()
                                .zip(target_str.chars())
                                .filter(|(a, b)| a != b)
                                .count()
                                .max(1)
                        };
                        candidates.push(Candidate {
                            word: word.to_string(),
                            dist,
                            same_vowel_base,
                            same_skeleton: true,
                            same_tone,
                        });
                    }
                }
            }
            return;
        }

        let expected_base = target_skeleton_chars[depth];
        for i in 0..trans_count {
            let Some(trans) = self.transition(trans_off, i) else {
                continue;
            };
            if base_char(trans.ch) == expected_base {
                self.search_skeleton(
                    trans.target,
                    depth + 1,
                    target_skeleton_chars,
                    target_str,
                    target_tone_stripped,
                    target_tone,
                    candidates,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn search_dfs(
        &self,
        node_id: u32,
        depth: usize,
        target_chars: &[char],
        target_str: &str,
        target_skeleton: &str,
        target_tone_stripped: &str,
        target_tone: usize,
        current_row: &[usize],
        prev_row: &[usize],
        last_char: Option<char>,
        max_dist: usize,
        candidates: &mut Vec<Candidate>,
    ) {
        if depth > 32 {
            return;
        }

        let Some((_, trans_off, trans_count)) = self.node(node_id) else {
            return;
        };

        let target_len = target_chars.len();
        let mut next_row = [0usize; 36];

        for i in 0..trans_count {
            let Some(trans) = self.transition(trans_off, i) else {
                continue;
            };

            next_row[0] = current_row[0] + 1;
            for j in 1..=target_len {
                let cost = if target_chars[j - 1] == trans.ch {
                    0
                } else {
                    1
                };
                let mut val = (current_row[j] + 1)
                    .min(next_row[j - 1] + 1)
                    .min(current_row[j - 1] + cost);
                if j >= 2 {
                    if let Some(prev_c) = last_char {
                        if prev_c == target_chars[j - 1]
                            && trans.ch == target_chars[j - 2]
                            && !prev_row.is_empty()
                        {
                            val = val.min(prev_row[j - 2] + 1);
                        }
                    }
                }
                next_row[j] = val;
            }

            let (is_child_final, _, _) = self.node(trans.target).unwrap_or((false, 0, 0));
            if is_child_final {
                let dist = next_row[target_len];
                if dist > 0 && dist <= max_dist {
                    if let Some(word) = self.final_word(trans.target) {
                        if word != target_str {
                            let same_skeleton = base_skeleton(word) == target_skeleton;
                            let same_vowel_base = strip_tone_word(word) == target_tone_stripped;
                            let tone = word_tone(word);
                            let same_tone = target_tone > 0 && tone == target_tone;
                            candidates.push(Candidate {
                                word: word.to_string(),
                                dist,
                                same_vowel_base,
                                same_skeleton,
                                same_tone,
                            });
                        }
                    }
                }
            }

            let min_cost = next_row[..=target_len]
                .iter()
                .copied()
                .min()
                .unwrap_or(usize::MAX);
            if min_cost <= max_dist {
                self.search_dfs(
                    trans.target,
                    depth + 1,
                    target_chars,
                    target_str,
                    target_skeleton,
                    target_tone_stripped,
                    target_tone,
                    &next_row[..=target_len],
                    current_row,
                    Some(trans.ch),
                    max_dist,
                    candidates,
                );
            }
        }
    }
}

/// Ung vien goi y trong tim kiem edit distance.
#[derive(Debug)]
struct Candidate {
    word: String,
    dist: usize,
    same_vowel_base: bool,
    same_skeleton: bool,
    same_tone: bool,
}

/// Bo dau thanh cua ky tu nhung giu nguyen chu cai co so (a, ă, â, e, ê, i, o, ô, ơ, u, ư, y).
pub fn strip_tone_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        _ => c.to_ascii_lowercase(),
    }
}

/// Bo dau thanh cua mot tu tieng Viet.
pub fn strip_tone_word(s: &str) -> String {
    s.chars().map(strip_tone_char).collect()
}

/// Tone cua mot ky tu tieng Viet (0 = khong dau, 1 = sac, 2 = huyen, 3 = hoi, 4 = nga, 5 = nang).
pub fn tone_of(c: char) -> usize {
    match c {
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => {
            1
        }
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => {
            2
        }
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => {
            3
        }
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => {
            4
        }
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => {
            5
        }
        _ => 0,
    }
}

/// Tone dau tien tim thay trong mot tu (0 neu khong co dau thanh).
pub fn word_tone(word: &str) -> usize {
    for c in word.chars() {
        let t = tone_of(c);
        if t > 0 {
            return t;
        }
    }
    0
}

/// Chuyen ky tu co dau thanh ky tu khong dau co so (skeleton) cua tieng Viet.
pub fn base_char(c: char) -> char {
    match c {
        'a' | 'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' | 'ă' | 'ằ' | 'ắ'
        | 'ẳ' | 'ẵ' | 'ặ' => 'a',
        'e' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => {
            'e'
        }
        'i' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'o' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ'
        | 'ở' | 'ỡ' | 'ợ' => 'o',
        'u' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => {
            'u'
        }
        'y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'd' | 'đ' => 'd',
        _ => c.to_ascii_lowercase(),
    }
}

/// Khung am tiet khong dau (skeleton) dung de so sanh va uu tien goi y.
pub fn base_skeleton(s: &str) -> String {
    s.chars().map(base_char).collect()
}

/// Dinh dang hoa/thuong cua tu goi y khop voi chuoi goc nguoi dung go.
pub fn match_case(candidate: &str, query: &str) -> String {
    let mut q_chars = query.chars();
    let Some(q_first) = q_chars.next() else {
        return candidate.to_string();
    };

    let q_all_upper = q_first.is_uppercase()
        && query
            .chars()
            .all(|c| !c.is_alphabetic() || c.is_uppercase());
    if q_all_upper {
        return candidate.to_uppercase();
    }

    if q_first.is_uppercase() {
        let mut c_chars = candidate.chars();
        if let Some(c_first) = c_chars.next() {
            let mut res = c_first.to_uppercase().collect::<String>();
            for c in c_chars {
                res.extend(c.to_lowercase());
            }
            return res;
        }
    }

    candidate.to_lowercase()
}

/// Kich thuoc 1 transition (byte) - export de test.
pub const TRANS_SIZE: usize = TRANS_LEN;
