// SPDX-License-Identifier: GPL-3.0-or-later
//! # textvn-dict
//!
//! Tu dien Viet **OS-native mmap** cho luat spelling (AS-040/AS-041).
//!
//! Crate nay **0 dependency**: tu viet FST v1 ([`fst`]), builder ([`build`]),
//! reader ([`reader`]) va mmap bang FFI thuan ([`mmap`]).
//!
//! ```no_run
//! use textvn_dict::Dictionary;
//!
//! // Nap tu dien (mmap) - chi mot lan cho ca process.
//! let dict = textvn_dict::load("data/assist/vi.fst")?;
//! assert!(dict.contains("vietnam"));
//! # Ok::<(), textvn_dict::DictError>(())
//! ```
//!
//! Khong mo file, khong parse `.dic` luc request: file `.fst` da duoc
//! sinh san (xem `cargo xtask gen-dict`).
//!
//! ## Provider spell-check cua OS (Windows · Linux · macOS)
//!
//! Ngoai FST cua ta, crate con co provider spell-check **tuy chon** theo OS
//! (P4-10), moi OS mot module rieng:
//!
//! - **Windows** — [`os_windows`] (Spell Checking API, crate `windows`),
//!   sentence-level `check` + `suggest` co ngan sach:
//!
//! ```no_run
//! # #[cfg(windows)] {
//! // `None` = OS khong co tu dien => caller dung `assist::dict::NoDictionary`
//! // (moi tu dung => khong bao loi spelling). Tren Windows mac dinh, `vi-VN`
//! // roi vao dung nhanh nay vi khong co language pack.
//! if let Some(checker) = textvn_dict::os_provider("vi-VN") {
//!     let _ = checker.check(&"cau van".encode_utf16().collect::<Vec<_>>());
//! }
//! # }
//! ```
//!
//! Chi phi Windows va ly do **khong** dung cho tieng Viet o v1: xem doc cua
//! [`os_windows`] (`Check` ~0,06 ms/cau nhung `Suggest` 3-7 ms/tu, va Windows
//! mac dinh khong co `vi-VN`).
//!
//! - **Linux** — [`os_linux`] (enchant-2, FFI thuan + `[link]` dong, word-level
//!   `contains`/`suggest` dung khop trait `Dictionary` cua `assist`). Feature
//!   `os-enchant` **mac dinh TAT** (CI khong co libenchant): khi tat,
//!   `os_provider_linux()` luon tra `None` => `NoDictionary` => khong bao loi
//!   spelling. Vi sao enchant thay vi libspelling / license LGPL / chi phi /
//!   fallback khi thieu dict `vi`: xem doc cua [`os_linux`].
//!
//! - **macOS** — [`os_macos`] (`NSSpellChecker`, AppKit, FFI thuần runtime
//!   Objective-C `objc_msgSend` — không crate nào). Word-level
//!   `contains`/`suggest`, **1 worker thread** AppKit + timeout 50 ms +
//!   fail-open + ngân sách `suggest`. Feature `os-macos` **mặc định TẮT**:
//!   khi tắt (mặc định: CI/gói không bật), `os_provider_macos()` luôn trả
//!   `None` ⇒ `NoDictionary` ⇒ không báo lỗi spelling; UI gợi ý chỉ qua
//!   toast/panel IMK (D30), chi phí chưa đo được — chi tiết xem doc
//!   [`os_macos`].

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod build;
pub mod fst;
pub mod mmap;
#[cfg(target_os = "linux")]
pub mod os_linux;
pub mod os_macos;
#[cfg(all(windows, feature = "os-spell"))]
pub mod os_windows;
pub mod reader;

use std::path::Path;
use std::sync::{Arc, OnceLock};

pub use build::Builder;
pub use mmap::{MapError, Mapping};
pub use os_macos::MacSpellChecker;
pub use reader::{base_char, base_skeleton, Reader};

#[cfg(target_os = "linux")]
pub use os_linux::EnchantSpellChecker;
#[cfg(all(windows, feature = "os-spell"))]
pub use os_windows::{SpellIssue, WindowsSpellChecker};

/// Provider spell-check cua OS cho `language_tag`, hoac `None` neu OS **khong**
/// co tu dien cho ngon ngu do.
///
/// `None` la duong **binh thuong**, khong phai loi: caller dung
/// `assist::dict::NoDictionary` (moi tu dung => khong bao loi spelling). Tren
/// Windows mac dinh, `vi-VN` luon roi vao nhanh nay (khong co language pack).
///
/// Loi COM / timeout cung tra `None` (fail-open).
#[cfg(all(windows, feature = "os-spell"))]
#[must_use]
pub fn os_provider(language_tag: &str) -> Option<WindowsSpellChecker> {
    WindowsSpellChecker::new(language_tag).ok()
}

/// Provider spell-check **Linux** (`enchant-2`) cho `language_tag`, hoac `None`
/// neu khong co tu dien (hoac feature `os-enchant` TAT — mac dinh: CI khong co
/// libenchant, build Linux phat hanh moi bat).
///
/// `None` la nhanh **binh thuong**, khong phai loi: caller dung
/// `assist::dict::NoDictionary` (moi tu dung => khong bao loi spelling). Noi
/// dung chi tiet — vi sao enchant thay vi libspelling, license LGPL (dynamic
/// link), chi phi, fallback khi thieu dict `vi` — xem doc cua [`os_linux`].
#[cfg(target_os = "linux")]
#[must_use]
pub fn os_provider_linux(language_tag: &str) -> Option<os_linux::EnchantSpellChecker> {
    os_linux::EnchantSpellChecker::new(language_tag).ok()
}

/// Provider spell-check **macOS** (`NSSpellChecker`, AppKit) cho
/// `language_tag`, hoac `None` neu khong co tu dien (hoac feature `os-macos`
/// TAT — mac dinh).
///
/// `None` la nhanh **binh thuong**, khong phai loi: caller dung
/// `assist::dict::NoDictionary` (moi tu dung => khong bao loi spelling) —
/// Fassist-107. Noi dung chi tiet — api surface `checkSpelling`/`guesses`,
/// chi phi chua do duoc, UI chi qua toast IMK (D30), fallback khi thieu dict
/// `vi` — xem doc cua [`os_macos`].
#[must_use]
pub fn os_provider_macos(language_tag: &str) -> Option<os_macos::MacSpellChecker> {
    os_macos::MacSpellChecker::new(language_tag).ok()
}

/// Loi khi nap tu dien.
#[derive(Debug)]
pub enum DictError {
    /// Khong map duoc file.
    Map(MapError),
    /// File sai format FST.
    Format(&'static str),
    /// OS khong co tu dien cho language tag (vi du Windows mac dinh khong co
    /// `vi-VN`), hoac khong tao duoc provider (COM loi/timeout).
    ///
    /// Caller phai fallback `assist::dict::NoDictionary`: moi tu duoc coi la
    /// dung => **khong** bao loi spelling nao.
    NoDictionary,
}

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictError::Map(e) => write!(f, "{e}"),
            DictError::Format(msg) => write!(f, "invalid dictionary: {msg}"),
            DictError::NoDictionary => write!(f, "no OS spell checker for this language tag"),
        }
    }
}

impl std::error::Error for DictError {}

impl From<MapError> for DictError {
    fn from(e: MapError) -> Self {
        DictError::Map(e)
    }
}

/// Tu dien da map, dung chung cho moi client (read-only, thread-safe).
///
/// Dung `Arc<Dictionary>` -> clone rẻ; dung chung 1 mapping.
#[derive(Debug)]
pub struct Dictionary {
    mapping: Arc<Mapping>,
}

impl Dictionary {
    /// Reader (view) cua tu dien - de u `self` nen an toàn.
    pub fn reader(&self) -> Reader<'_> {
        // Safety/borrow: `mapping` la `Arc` -> song song voi `self`.
        Reader::new(self.mapping.as_slice()).expect("mapping da validate khi nap")
    }

    /// Buffer da map.
    pub fn mapping(&self) -> &Mapping {
        &self.mapping
    }

    /// `true` neu `word` co trong tu dien (chuoi da chuan hoa: lowercase).
    pub fn contains(&self, word: &str) -> bool {
        self.reader().contains(word)
    }

    /// So tu trong tu dien.
    pub fn len(&self) -> usize {
        self.reader().len()
    }

    /// `true` neu tu dien rong.
    pub fn is_empty(&self) -> bool {
        self.reader().is_empty()
    }

    /// Goi y cac tu thay the cho `word` (AS-041), toi da `limit` ket qua.
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.reader().suggest(word, limit)
    }
}

impl textvn_assist::dict::Dictionary for Dictionary {
    fn contains(&self, word: &str) -> bool {
        self.reader().contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.reader().suggest(word, limit)
    }
}

#[cfg(all(windows, feature = "os-spell"))]
impl textvn_assist::dict::Dictionary for WindowsSpellChecker {
    fn contains(&self, word: &str) -> bool {
        let u16s: Vec<u16> = word.encode_utf16().collect();
        self.check(&u16s).is_empty()
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let u16s: Vec<u16> = word.encode_utf16().collect();
        self.suggest(&u16s, limit)
    }
}

#[cfg(target_os = "linux")]
impl textvn_assist::dict::Dictionary for EnchantSpellChecker {
    fn contains(&self, word: &str) -> bool {
        self.contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.suggest(word, limit)
    }
}

impl textvn_assist::dict::Dictionary for MacSpellChecker {
    fn contains(&self, word: &str) -> bool {
        self.contains(word)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        self.suggest(word, limit)
    }
}

/// Nap tu dien tu file FST (mmap) - validate header + checksum.
pub fn load(path: impl AsRef<Path>) -> Result<Arc<Dictionary>, DictError> {
    let mapping = Mapping::open(path.as_ref())?;
    let reader = Reader::new(mapping.as_slice()).ok_or(DictError::Format("header sai"))?;
    if !reader.verify_checksum() {
        return Err(DictError::Format("checksum sai"));
    }
    Ok(Arc::new(Dictionary {
        mapping: Arc::new(mapping),
    }))
}

/// Global tu dien - nap **mot lan** cho ca process (`OnceLock`).
static GLOBAL: OnceLock<Option<Arc<Dictionary>>> = OnceLock::new();

/// Nap tu dien global tu `path` (idempotent).
///
/// Lần dau tien goi se map file; cac lan sau tra ve cung `Arc` (khong map
/// lai). Neu file khong hop le -> `None` (khong panic) - caller fallback.
pub fn set_global(path: impl AsRef<Path>) -> Option<Arc<Dictionary>> {
    GLOBAL.get_or_init(|| load(path.as_ref()).ok()).clone()
}

/// Lay tu dien global (neu da nap).
pub fn global() -> Option<Arc<Dictionary>> {
    GLOBAL.get().and_then(|d| d.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut b = Builder::new();
        b.extend(["vietnam", "viet", "tieng", "viet", "hanoi", "ha noi"]);
        b.to_bytes()
    }

    #[test]
    fn builder_deterministic() {
        assert_eq!(sample(), sample());
    }

    #[test]
    fn roundtrip_contains() {
        let bytes = sample();
        let reader = Reader::new(&bytes).expect("header hop le");
        assert!(reader.verify_checksum());
        assert!(reader.contains("vietnam"));
        assert!(reader.contains("viet"));
        assert!(reader.contains("ha noi"));
        assert!(!reader.contains("vietnamx"));
        assert!(!reader.contains("xyz"));
    }

    #[test]
    fn final_words_liet() {
        let bytes = sample();
        let reader = Reader::new(&bytes).expect("header hop le");
        let mut words: Vec<String> = Vec::new();
        reader.for_each_final(|_, w| words.push(w.to_string()));
        words.sort();
        assert_eq!(words, vec!["ha noi", "hanoi", "tieng", "viet", "vietnam"]);
    }

    #[test]
    fn header_sai_bi_tu_choi() {
        let mut bytes = sample();
        bytes[0] ^= 0xff;
        assert!(Reader::new(&bytes).is_none());
    }

    #[test]
    fn checksum_phat_hien_hong_data() {
        let mut bytes = sample();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        let reader = Reader::new(&bytes).expect("header hop le");
        assert!(!reader.verify_checksum());
    }

    #[test]
    fn mmap_doc_file_that() {
        let bytes = sample();
        let dir = std::env::temp_dir().join("textvn-dict-test");
        std::fs::create_dir_all(&dir).expect("tao thu muc temp");
        let path = dir.join("vi-test.fst");
        std::fs::write(&path, &bytes).expect("ghi file");

        let dict = load(&path).expect("nap tu dien qua mmap");
        assert!(dict.contains("vietnam"));
        assert!(!dict.contains("xyz"));
        assert_eq!(dict.len(), 5);

        let _ = std::fs::remove_file(&path);
    }

    /// Doc `.dic`: header so luong (dong dau) + cac tu UTF-8 (moi dong 1 tu).
    /// Tra danh sach tu (bo qua dong header, bo dong trong).
    fn load_dic_words(path: &std::path::Path) -> Vec<String> {
        let text = std::fs::read_to_string(path).expect("doc file .dic");
        text.lines()
            .skip(1)
            .map(str::trim)
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    }

    /// Duong dan den file du lieu o workspace root (`data/assist/*.dic`).
    fn data_assist(name: &str) -> std::path::PathBuf {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .join("..")
            .join("data")
            .join("assist")
            .join(name)
    }

    #[test]
    fn dic_vi_dau_cu_phat_hien_tat_ca_tu() {
        // vi-DauCu.dic: kieu dau cu (hoa, thuy), 6642 tu.
        let path = data_assist("vi-DauCu.dic");
        let words = load_dic_words(&path);
        assert!(!words.is_empty(), "vi-DauCu.dic khong duoc rong");

        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");
        assert!(reader.verify_checksum(), "checksum phai dung");
        assert_eq!(reader.len(), words.len(), "so tu final phai khop header");

        // Moi tu trong file phai duoc chua.
        for w in &words {
            assert!(
                reader.contains(w),
                "tu `{w}` phai duoc chua trong FST vi-DauCu"
            );
        }
    }

    #[test]
    fn dic_vi_dau_cu_khong_chua_tu_khong_co_trong_file() {
        let path = data_assist("vi-DauCu.dic");
        let words = load_dic_words(&path);
        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");

        // Tu chua trong tu dien -> khong duoc chua.
        assert!(!reader.contains("xyzzy"));
        assert!(!reader.contains("asdfgh"));
        assert!(!reader.contains("")); // tu rong bo qua khi build
    }

    #[test]
    fn dic_vi_dau_moi_khac_vi_dau_cu() {
        // vi-DauMoi.dic: kieu dau moi (hoa -> hoà co dau o e), cung 6642 tu.
        let cu = data_assist("vi-DauCu.dic");
        let moi = data_assist("vi-DauMoi.dic");
        let words_cu: std::collections::HashSet<_> = load_dic_words(&cu).into_iter().collect();
        let words_moi: std::collections::HashSet<_> = load_dic_words(&moi).into_iter().collect();

        assert_eq!(words_cu.len(), 6642, "vi-DauCu: 6642 tu");
        assert_eq!(words_moi.len(), 6642, "vi-DauMoi: 6642 tu");

        // Hai tu dien co nhieu tu chung (goc cung 1ec5/hunspell-vi).
        let overlap = words_cu.intersection(&words_moi).count();
        assert!(overlap > 6000, "nhieu tu chung giua 2 kieu dau: {overlap}");

        // Co tu chi xuat hien o mot kieu dau (vd "hoa" chi o Moi, "hoà" chi o Cu).
        let only_cu = words_cu.difference(&words_moi).count();
        let only_moi = words_moi.difference(&words_cu).count();
        assert!(only_cu > 0, "co tu chi o DauCu: {only_cu}");
        assert!(only_moi > 0, "co tu chi o DauMoi: {only_moi}");
    }

    #[test]
    fn dic_dau_cu_chua_hoa_khong_chua_hoa_moi() {
        // Dau cu (vi-DauCu): "hòa" (dau o a, kieu cu) -> chua; "hoà" (dau o e, kieu moi) -> khong chua.
        let path = data_assist("vi-DauCu.dic");
        let words = load_dic_words(&path);
        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");

        assert!(reader.contains("hòa"), "DauCu phai chua `hòa` (dau o a)");
        assert!(
            !reader.contains("hoà"),
            "DauCu khong phai chua `hoà` (dau o e la kieu moi)"
        );
    }

    #[test]
    fn dic_dau_moi_chua_hoa_moi_khong_chua_hoa_cu() {
        // Dau moi (vi-DauMoi): "hoà" (dau o e, kieu moi) -> chua; "hòa" (dau o a, kieu cu) -> khong chua.
        let path = data_assist("vi-DauMoi.dic");
        let words = load_dic_words(&path);
        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");

        assert!(reader.contains("hoà"), "DauMoi phai chua `hoà` (dau o e)");
        assert!(
            !reader.contains("hòa"),
            "DauMoi khong phai chua `hòa` (dau o a la kieu cu)"
        );
    }

    #[test]
    fn dic_for_each_final_tra_tat_ca_tu() {
        let path = data_assist("vi-DauCu.dic");
        let words = load_dic_words(&path);
        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");

        let mut final_words: Vec<String> = Vec::new();
        reader.for_each_final(|_, w| final_words.push(w.to_string()));
        final_words.sort();
        let mut expected = words.clone();
        expected.sort();
        assert_eq!(
            final_words, expected,
            "for_each_final phai tra day du tat ca tu"
        );
    }

    #[test]
    fn suggest_sample_dictionary() {
        let bytes = sample();
        let reader = Reader::new(&bytes).expect("FST hop le");

        // Gợi ý cho từ gõ sai 1 ký tự
        let sg = reader.suggest("vietna", 3);
        assert!(!sg.is_empty());
        assert_eq!(sg[0], "vietnam");

        // Gợi ý cho từ đảo ký tự (Damerau transposition)
        let sg2 = reader.suggest("teing", 3);
        assert!(!sg2.is_empty());
        assert_eq!(sg2[0], "tieng");
    }

    #[test]
    fn suggest_dic_dau_moi_typos_chinh_xac() {
        let path = data_assist("vi-DauMoi.dic");
        let words = load_dic_words(&path);
        let mut b = Builder::new();
        b.extend(words.iter().map(|w| w.as_str()));
        let bytes = b.to_bytes();
        let reader = Reader::new(&bytes).expect("FST hop le");

        // AS-041: loi sai vi tri dau pho bien phai co goi y dung o vi tri hang dau
        let test_cases = [
            ("nghiã", "nghĩa"),
            ("cuả", "của"),
            ("tiêng", "tiếng"),
            ("nhứng", "nhưng"),
            ("nguyến", "nguyễn"),
        ];

        let mut correct_top1 = 0;
        for (typo, expected) in test_cases {
            let suggestions = reader.suggest(typo, 3);
            assert!(
                !suggestions.is_empty(),
                "typo `{typo}` phai co it nhat 1 goi y"
            );
            if suggestions[0] == expected {
                correct_top1 += 1;
            } else {
                assert!(
                    suggestions.contains(&expected.to_string()),
                    "typo `{typo}` phai chua `{expected}` trong top goi y: {suggestions:?}"
                );
            }
        }

        // Ty le recall Top-1 phai dat tren 80% (o day 5/5 = 100%)
        let recall = (correct_top1 as f64) / (test_cases.len() as f64);
        assert!(
            recall >= 0.80,
            "recall Top-1 phai >= 80%, thuc te: {recall}"
        );
    }
}
