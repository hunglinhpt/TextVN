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
//! - [`BridgeDict`] la newtype generic de crates khac (`ffi`, `dict`) gan
//!   nguon tu dien ben ngoai vao trait nay ma crate `assist` van **0 dep**.
//! - [`Budgeted`] la **bo ngan sach `suggest`**: gioi han so lan goi + do
//!   tong thoi gian (de khong vuot ngan sach 50 ms/cau, P4-0 D36/D40).

use std::cell::Cell;
use std::time::{Duration, Instant};

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

/// **Bo ngan sach `suggest`** boc mot tu dien khac (P4-10 §5.3, P4-0 D36/D40).
///
/// Vi sao can bo boc nay: provider OS ton **3,1–6,9 ms/`Suggest`** (do that
/// tren Windows 11, gap ~50–100 lan `Check` — so do cua agent B trong
/// `docs/50-assist/P4-10-OS-PROVIDER-SPEC.md`, finding `P4-10-F3`). Neu goi
/// `suggest` cho **moi** tu la trong mot cau thi de vuot ngan sach 50 ms/cau
/// (D36) va them delay vao luong ban phim. Do do bo boc nay:
///
/// - **Gioi han so lan** `suggest` (mac dinh
///   [`crate::rules::DEFAULT_SUGGEST_BUDGET`] = 2 lan/cau, D40): het ngan
///   sach → `suggest` tra ve rong (**fail-open**: thac im lang con hon bao
///   sai — khong panic, khong chan; toan bo loi van duoc bao nhung khong kem
///   goi y).
/// - **Do tong thoi gian** da dung cho `suggest` — lay qua `elapsed()` — de
///   kiem chung AC-4 (P4-10 §7.2: `suggest` khong pha ngan sach) va tinh
///   ngan sach hop ly bang so lieu that.
///
/// Ky thuat: `inner` la **borrow** (`&'a D`, cho phep `D: ?Sized`) de boc
/// truc tiep `&dyn Dictionary` trong [`crate::rules::Ctx`] ma khong phai
/// clone du lieu hay them blanket impl. `contains` di qua trong suot va
/// khong ton ngan sach (provider OS: `Check` chi ~0,057 ms — re). Chi dem +
/// do thoi gian, khong I/O, khong log chuoi nao (G8).
pub struct Budgeted<'a, D: Dictionary + ?Sized> {
    inner: &'a D,
    max_suggest: usize,
    calls: Cell<usize>,
    elapsed: Cell<Duration>,
}

impl<'a, D: Dictionary + ?Sized> Budgeted<'a, D> {
    /// Boc `inner` voi gioi han `max_suggest` lan goi `suggest`.
    pub fn new(inner: &'a D, max_suggest: usize) -> Self {
        Self {
            inner,
            max_suggest,
            calls: Cell::new(0),
            elapsed: Cell::new(Duration::ZERO),
        }
    }

    /// Gioi han so lan suggest da cau hinh.
    pub fn max_suggest(&self) -> usize {
        self.max_suggest
    }

    /// So lan `suggest` da di den tu dien that (khong tinh lan bi cat vi het
    /// ngan sach - nhung lan do tra ve rong va khong cham vao `inner`).
    pub fn suggest_calls(&self) -> usize {
        self.calls.get()
    }

    /// Tong thoi gian da dung cho cac lan `suggest` den tu dien that.
    ///
    /// Do bang `std::time::Instant` (Windows: do chinh xac ~100 ns), du de
    /// do 3–7 ms/`Suggest` cua provider OS (P4-10 §7.2, AC-4).
    pub fn elapsed(&self) -> Duration {
        self.elapsed.get()
    }
}

impl<D: Dictionary + ?Sized> Dictionary for Budgeted<'_, D> {
    fn contains(&self, word: &str) -> bool {
        self.inner.contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        // Het ngan sach -> fail-open: tra rong ngay, khong cham `inner`
        // (P4-10 §5.1.4: thac im lang con hon bao sai).
        if self.calls.get() >= self.max_suggest {
            return Vec::new();
        }
        let start = Instant::now();
        let out = self.inner.suggest(word, limit);
        self.elapsed.set(self.elapsed.get() + start.elapsed());
        self.calls.set(self.calls.get() + 1);
        out
    }
}

/// Newtype generic: gan nguon tu dien ben ngoai vao trait [`Dictionary`]
/// (P4-10 §5.2) ma crate `assist` van **0 dependency** (P4-0 D32).
///
/// Vi sao newtype generic chu khong impl thang cho kieu nguon ben ngoai:
///
/// - Crate nay khong the biet ten kieu nguon tu dien (`dictionaries::Fst`
///   cua crate `textvn-dict`, adapter C-ABI `ime_os_spell_v1` cua `ffi`…)
///   ma khong phu thuoc crate do - se tao phu thuoc vong. Newtype generic
///   chi "boi" bat ky kieu nao da trien khai [`Dictionary`]; phu thuoc chay
///   mot chieu (ben phu thuoc `assist`, khong nguoc lai).
/// - Bridge that su voi OS nam o crate `ffi` (C-ABI callback hoac Swift
///   bridge - P4-10 §5.2): adapter ben do impl [`Dictionary`] hoac bo vao
///   [`BridgeDict`] roi boc them [`Budgeted`] truoc khi truyen vao
///   [`crate::check_with`]. Cach day, toan bo ngan sach/fail-open nam trong
///   [`Budgeted`], [`BridgeDict`] chi dan chuyen (forward) - test duoc doc
///   lap bang fake provider ma khong can OS nao.
///
/// Huy ngan sach cua bridge C-ABI o `ffi` la co y: dieu bien (fn pointer,
/// `Vec<u8>` scratch, convert UTF-8 + UTF-16 chi o bien) thuoc ve lop FFI;
/// crate nay giu giao dien thuan logic, khong I/O (P4-0 §3).
#[derive(Debug, Clone, Copy, Default)]
pub struct BridgeDict<F> {
    inner: F,
}

impl<F> BridgeDict<F> {
    /// Boi mot nguon tu dien da trien khai [`Dictionary`].
    pub fn new(inner: F) -> Self {
        Self { inner }
    }

    /// Lay lai nguon tu dien goc.
    pub fn into_inner(self) -> F {
        self.inner
    }
}

impl<F: Dictionary> Dictionary for BridgeDict<F> {
    fn contains(&self, word: &str) -> bool {
        self.inner.contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.inner.suggest(word, limit)
    }
}

// Nguon tu dien bi borrow (`&T`, giong `Ctx::dict: &'a dyn Dictionary`) cung
// dung duoc voi BridgeDict/Budgeted - keo dai trait qua tham chieu ma khong
// can clone (0-dep, nguyen tac borrow thuan).
impl<T: Dictionary + ?Sized> Dictionary for &T {
    fn contains(&self, word: &str) -> bool {
        (**self).contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        (**self).suggest(word, limit)
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

    #[test]
    fn budgeted_boc_no_dictionary_fail_open() {
        let dict = Budgeted::new(&NoDictionary, 2);
        assert!(dict.contains("bat cu tu nao"));
        assert!(dict.suggest("xyzzy", 3).is_empty());
        assert_eq!(dict.suggest_calls(), 1);
        // Tu dien rong na~ gan nhu 0 thoi gian, nhung do that van > 0
        // (ca thoi gian do deu tich cong) - khong assert = ZERO.
        assert!(dict.elapsed() < std::time::Duration::from_millis(1));
    }
}
