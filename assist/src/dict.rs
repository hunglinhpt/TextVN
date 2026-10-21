// SPDX-License-Identifier: GPL-3.0-or-later
//! Trait tu dien cho luat **spelling** (AS-040/AS-041).
//!
//! Crate `assist` giu **0 dep, portable** (P4-0 D32): no day khong biet OS
//! nao, khong tu mo file. Noi dung tu dien (FST mmap) do **crate khac** nap
//! (vi du `textvn-dict`) va truyen vao qua trait nay.
//!
//! - Trait nay chi la **giao dien** - khong I/O, khong network.
//! - [`NoDictionary`] la trien khai "khong co tu dien" (fallback khi OS co
//!   spellchecker rieng, hoac khi chua nap du lieu): moi tu deu coi la dung,
//!   khong bao loi spelling nao.

/// Mot tu dien: tra loi "tu nay co dung khong" + "goi y tu nao".
pub trait Dictionary {
    /// `true` neu `word` (da chuan hoa: lowercase) co trong tu dien.
    fn contains(&self, word: &str) -> bool;

    /// Goi y tu thay the cho `word`, toi da `limit` ket qua, tot nhat truoc.
    ///
    /// Mang rong = khong goi y duoc (khong phai loi).
    fn suggest(&self, word: &str, limit: usize) -> Vec<String>;
}

/// Tu dien "rong" - dung khi khong nap duoc du lieu (hoac OS da co spellchecker).
///
/// Moi tu deu coi la **dung** (khong bao loi spelling) va khong co goi y nao.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoDictionary;

impl Dictionary for NoDictionary {
    fn contains(&self, _word: &str) -> bool {
        true
    }

    fn suggest(&self, _word: &str, _limit: usize) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tu_dien_rong_khong_bao_loi() {
        let dict = NoDictionary;
        assert!(dict.contains("xyzzy"));
        assert!(dict.suggest("xyzzy", 3).is_empty());
    }
}
