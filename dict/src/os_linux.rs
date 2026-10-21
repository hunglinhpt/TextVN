// SPDX-License-Identifier: GPL-3.0-or-later
//! Provider spell-check **Linux** (`enchant-2`, link động) — chỉ có trên Linux.
//!
//! Đây là **adapter ở biên** (P4-0 **D32**), dùng chung pattern với
//! [`crate::os_windows`]: worker thread + timeout 50 ms + fail-open + ngân sách
//! cho `suggest`. Crate `dict` không biết `assist`, `assist` không biết enchant.
//!
//! ## Vì sao dùng **enchant-2** thay vì `libspelling` (GNOME)
//!
//! Do đo thực nghiệm kiểm tra header theo từng tag (P4-10 §4.1):
//!
//! 1. `spelling_checker_check_ranges` (API trả offset **char** cho cả câu)
//!    **chỉ có trên `main` (1.0.alpha)**, không có ở 0.4.8/0.4.10 — mà bản
//!    distro đang phát hành là 0.4.x (Debian 13 = 0.4.8, Ubuntu 24.04 = 0.2.0,
//!    Fedora 43 = 0.4.9, Arch = 0.4.10). Trait `Dictionary` của `assist` chỉ
//!    cần `contains(word) -> bool` + `suggest(word, limit) -> Vec<String>` và
//!    `assist` đã tự tokenize ⇒ API theo **từng từ** của enchant là đủ;
//!    `check_ranges` để dành cho task v2 khi libspelling 1.0 vào distro.
//! 2. `libspelling` phụ thuộc **GTK4 + GtkSource5 + GLib** — kéo GTK4 vào tiến
//!    trình engine IBus/Fcitx5 = tăng RSS + bề mặt lỗi (P4-10 §4.2, rủi ro L2).
//!    `libspelling` thực chất chỉ là wrapper của `enchant-2`, nên đi thẳng
//!    enchant có cùng chức năng với phụ thuộc nhẹ hơn nhiều.
//! 3. API enchant-2 **ổn định từ 2.x** (khác libspelling còn 1.0.alpha —
//!    churn ⇒ rủi ro L3).
//!
//! ## Trait và encoding
//!
//! Enchant làm việc với **UTF-8**, tag dạng `xx_YY` (`vi`, `vi_VN`); caller đưa
//! BCP-47 (`vi`, `vi-VN`) — helper `tag_variants` (trong `backend`) đổi sang
//! `xx_YY` và thêm biến thể ngôn ngữ gốc (`vi_VN` → `vi`) để khớp cách distro
//! đặt tên từ điển hunspell. Ánh xạ API:
//!
//! | Trait | enchant |
//! |---|---|
//! | `contains(word)` | `enchant_dict_check(dict, word, len)` — **0** = đúng, **dương** = từ sai, **âm** = lỗi/tham số sai ⇒ fail-open (hợp đồng ghi rõ trong `enchant.h`) |
//! | `suggest(word, limit)` | `enchant_dict_suggest` + `enchant_dict_free_string_list` (bắt buộc free — W4) |
//!
//! Probe capability: `enchant_broker_dict_exists` (hỏi trực tiếp, rẻ) thay vì
//! `enchant_broker_list_dicts` (callback liệt kê tất cả — chỉ cần khi debug).
//!
//! ## License (AC-13, P4-10 §7.3)
//!
//! libenchant: **LGPL-2.1-or-later + exception** cho phép nạp backend không-LGPL
//! (hunspell là GPL). Ta **link động** (`-lenchant-2`, không static) ⇒ tương
//! thích GPL-3.0-or-later của repo. Thư viện C **không đi qua cargo-deny** ⇒
//! liệt kê trong `docs/compliance/` khi bật feature trong gói phát hành.
//!
//! ## Chi phí
//!
//! - In-process: enchant + backend `hunspell-vi` (nếu cài) nạp **trong cùng tiến
//!   trình** ⇒ text KHÔNG rời tiến trình TextVN (khác Windows out-of-proc) —
//!   bớt điều phải ghi trong privacy notice. Nhưng enchant gọi C qua FFI:
//!   hunspell có thể chậm với từ rất dài, backend ngoài (aspell…) có thể chậm
//!   (L5) ⇒ vẫn giữ worker thread + timeout 50 ms như Windows để chặn đường
//!   chậm một cách độc lập với nội dung text.
//! - ⚠️ Độ trễ thực nghiệm (nguội/warm, 1 000 từ) **chưa đo được** — không có
//!   máy Linux trong phiên này (P4-10 §9.7); phải đo trước khi bật feature
//!   trong gói Linux phát hành (AC-3/AC-4).
//!
//! ## Fallback khi không có dict `vi` (mặc định phổ biến)
//!
//! `hunspell-vi` chỉ có Fedora/RHEL/Alma/ALT; **không có** Debian/Ubuntu
//! (P4-10 §4.4). Ta **không ship** từ điển (ADR-014: license gói GPL-2.0-only).
//! Không tìm thấy tag ⇒ `new` trả [`DictError::NoDictionary`],
//! `is_supported` = `false` ⇒ caller dùng `assist::dict::NoDictionary` (mọi từ
//! đúng) ⇒ **không** báo lỗi spelling. Tuyệt đối **không** mượn checker ngôn
//! ngữ khác (Windows đo được: checker `en-US` báo sai 3/8 từ tiếng Việt ⇒
//! FPR không chấp nhận được).
//!
//! ## Feature `os-enchant` — mặc định **TẮT**
//!
//! CI không có libenchant ⇒ toàn bộ FFI nằm trong `backend` dưới
//! `#[cfg(feature = "os-enchant")]`. Feature tắt (mặc định): `new` luôn
//! `Err(NoDictionary)`, `contains` luôn `true`, `suggest` rỗng — hành vi y hệt
//! `NoDictionary` ⇒ **không hồi quy** (AC-1) và build/CI không cần cài
//! libenchant. Chỉ build gói Linux trên máy có `libenchant-2-dev` mới bật
//! `--features os-enchant`. Không thêm crate/dependency nào: các symbol
//! `enchant_*` **không trùng** symbol runtime của `std` (bài học Fassist-104 chỉ
//! chặn `open`/`close`) nên FFI thuần an toàn, giữ crate 0-dep.
//!
//! ## Privacy (S2)
//!
//! Không log `text`/`word` ở bất kỳ nhánh nào — chỉ log/đếm mã lỗi.

use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "os-enchant")]
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::DictError;

/// Ngan sach thoi gian cho `suggest` (P4-0 D36: `<= 50 ms`, het gio = tra rong).
pub const SUGGEST_BUDGET: Duration = Duration::from_millis(50);

/// Cap cung so lan `suggest` trong mot cua so ngan sach (P4-10 §5.3,
/// `MAX_SUGGEST_WORDS = 3`): do tre `enchant` chua do duoc (khong co may
/// Linux trong phien nay) nhung so Windows (3 x 6,9 ms ~ 21 ms) cho thay
/// van trong cua so 50 ms.
pub const SUGGEST_CALLS_PER_CHECK: usize = 3;

/// Ngan sach cho `suggest`: cua so thoi gian + dem so lan goi.
///
/// Tach rieng de test duoc tren may **khong co** libenchant — logic ngan sach
/// thuan Rust, khong lien quan FFI (giong SuggestBudget cua os_windows).
#[derive(Debug)]
struct SuggestBudget {
    used: AtomicUsize,
    deadline: Mutex<Instant>,
}

impl SuggestBudget {
    /// Chua mo cua so: moi `suggest` deu bi tu choi cho toi lan `contains` dau.
    ///
    /// Build feature-tat (mac dinh, CI khong co libenchant) khong co duong nao
    /// khoi tao provider (`os_provider_linux` = `Err(NoDictionary)`) nen
    /// `dead_code` bat o target lib — cho phep co muc tieu 1 fn nay (duo lai
    /// test ngan sach duoc chay doc lap khong can FFI; feature-bat dung nhu
    /// thuong), KHONG phai ban `#[allow]` khong co can cu.
    #[cfg_attr(not(feature = "os-enchant"), allow(dead_code))]
    fn new() -> Self {
        Self {
            used: AtomicUsize::new(SUGGEST_CALLS_PER_CHECK),
            deadline: Mutex::new(Instant::now()),
        }
    }

    /// Don vi goi `contains` tren Linux la **moi tu** (khac Windows: `check`
    /// moi cau), nen khong the "mo lai cua so" o moi loi goi: moi 1 tu OOV deu
    /// co 1 `contains` truoc => cap 3 lan se khong bao gio cham. Cua so chi mo
    /// LAI khi cot han: trong cua so 50 ms hien hanh, `contains` duoc goi tu do
    /// nhung `suggest` chi duoc toi da [`SUGGEST_CALLS_PER_CHECK`] lan.
    fn anchor(&self) {
        let mut deadline = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if Instant::now() >= *deadline {
            *deadline = Instant::now() + SUGGEST_BUDGET;
            self.used.store(0, Ordering::Relaxed);
        }
    }

    /// `true` neu con du ngan sach cho mot lan `suggest` nua.
    fn allow(&self) -> bool {
        let deadline = *self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if Instant::now() >= deadline {
            return false;
        }
        self.used.fetch_add(1, Ordering::Relaxed) < SUGGEST_CALLS_PER_CHECK
    }
}

/// Provider spell-check Linux dung chung 1 worker thread libenchant-2.
///
/// Tao bang [`EnchantSpellChecker::new`]. Provider **khong** tu bao loi: khi
/// may khong co tu dien cho `language_tag` (vi du `vi`/`vi_VN` — truong hop
/// mac dinh pho bien tren Debian/Ubuntu) thi `new` tra
/// [`DictError::NoDictionary`] va caller phai fallback
/// `assist::dict::NoDictionary` (moi tu dung => khong bao loi spelling).
///
/// Moi phuong thuc deu **fail-open**: loi enchant, timeout hoac worker chet =>
/// `contains` = `true` (tu dung), `suggest` = rong; khong panic, khong block
/// qua 50 ms. Khi feature `os-enchant` **TAT** (mac dinh, CI khong co
/// libenchant) thi `new` luon `NoDictionary` => cung la duong fail-open.
pub struct EnchantSpellChecker {
    /// Kenh gui job sang worker thread (chi co khi feature `os-enchant` bat).
    #[cfg(feature = "os-enchant")]
    tx: SyncSender<backend::Job>,
    /// Ngan sach `suggest` — luon co de lop ngan sach test duoc doc lap tren
    /// may khong co libenchant.
    budget: SuggestBudget,
}

#[cfg(feature = "os-enchant")]
impl EnchantSpellChecker {
    /// Tao checker cho `language_tag` (BCP-47, vi du `vi-VN` hay `vi`).
    ///
    /// Broker + resolve tag chay tren worker thread (co timeout 50 ms); ket qua
    /// capability **cache** theo tag cho toan process — moi tag hoi toi da
    /// 1 lan/phien (P4-10 AC-2).
    ///
    /// # Errors
    ///
    /// [`DictError::NoDictionary`] khi: khong co tu dien cho tag (va bien the
    /// ngon ngu goc) — truong hop **mac dinh** khi user chua cai `hunspell-vi`;
    /// thread khong spawn duoc; probe qua han 50 ms; broker khoi tao that bai;
    /// `request_dict` that bai. Caller dung `assist::dict::NoDictionary` =>
    /// **khong** bao loi spelling nao.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(target_os = "linux", feature = "os-enchant"))] {
    /// use textvn_dict::os_linux::EnchantSpellChecker;
    ///
    /// // May khong co tu dien vi (mac dinh pho bien) => Err(NoDictionary).
    /// if let Ok(checker) = EnchantSpellChecker::new("vi-VN") {
    ///     let _ = checker.contains("vietnam");
    /// }
    /// # }
    /// ```
    pub fn new(language_tag: &str) -> Result<Self, DictError> {
        if language_tag.trim().is_empty() {
            return Err(DictError::NoDictionary);
        }
        let Some((worker, ready)) = backend::spawn_worker(language_tag, true) else {
            return Err(DictError::NoDictionary);
        };
        if !ready {
            return Err(DictError::NoDictionary);
        }
        Ok(Self {
            tx: worker.tx,
            budget: SuggestBudget::new(),
        })
    }

    /// Cong capability: broker enchant co tu dien cho `language_tag` khong.
    ///
    /// Re sau lan hoi dau (cache). `false` = khong co tu dien => caller fallback
    /// `NoDictionary` (khong bao loi spelling). Moi loi libenchant cung tra
    /// `false` (fail-safe: tha im lang con hon bao sai).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(target_os = "linux", feature = "os-enchant"))] {
    /// use textvn_dict::os_linux::EnchantSpellChecker;
    ///
    /// if !EnchantSpellChecker::is_supported("vi-VN") {
    ///     // => dung assist::dict::NoDictionary, khong bao loi spelling.
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_supported(language_tag: &str) -> bool {
        if language_tag.trim().is_empty() {
            return false;
        }
        if let Some(supported) = backend::support_cached(language_tag) {
            return supported;
        }
        match backend::spawn_worker(language_tag, false) {
            Some((_worker, supported)) => supported,
            None => false,
        }
    }

    /// Kiem tra 1 tu (UTF-8, da chuan hoa lowercase) co trong tu dien khong.
    ///
    /// Goi `contains` **mang theo ngan sach**: don vi goi o day la **moi tu**
    /// (khac Windows — `check` moi cau) nen cua so ngan sach chi duoc mo lai
    /// khi cua so truoc het han (xem [`SUGGEST_BUDGET`]).
    ///
    /// # Fail-open
    ///
    /// Tu rong / loi enchant (`enchant_dict_check` tra **am**) / timeout 50 ms /
    /// worker ban => `true` (tu dung, KHONG bao loi spelling). Tra **0** (tu
    /// dung) => `true`; tra **duong** (tu sai) => `false` => assist bao loi va
    /// co the goi `suggest` (co ngan sach).
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.budget.anchor();
        if word.is_empty() {
            return true;
        }
        let (reply, result) = sync_channel(1);
        let job = backend::Job::Contains {
            word: word.to_owned(),
            reply,
        };
        // `try_send`: neu worker dang ban/treo thi bo ngay, KHONG block caller.
        if self.tx.try_send(job).is_err() {
            return true;
        }
        result.recv_timeout(backend::WORK_TIMEOUT).unwrap_or(true)
    }

    /// Goi y tu thay the cho `word` (UTF-8), toi da `limit` ket qua.
    ///
    /// **Co ngan sach** (bat buoc — do tre `enchant_dict_suggest` cua hunspell
    /// chua do duoc nhung so Windows 3-7 ms/tu cho thay phai chan cung 50 ms):
    ///
    /// - chi duoc goi toi da [`SUGGEST_CALLS_PER_CHECK`] lan trong cua so
    ///   [`SUGGEST_BUDGET`] hien hanh;
    /// - chua co cua so (chua goi `contains` lan nao) / het cua / vuot cap
    ///   => `Vec` rong.
    ///
    /// `limit` bi chan tren [`backend::MAX_SUGGEST_RESULTS`]; timeout / loi
    /// enchant => rong (fail-open, khong phai loi).
    #[must_use]
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let limit = limit.min(backend::MAX_SUGGEST_RESULTS);
        if limit == 0 || word.is_empty() || !self.budget.allow() {
            return Vec::new();
        }
        let (reply, result) = sync_channel(1);
        let job = backend::Job::Suggest {
            word: word.to_owned(),
            limit,
            reply,
        };
        if self.tx.try_send(job).is_err() {
            return Vec::new();
        }
        result
            .recv_timeout(backend::WORK_TIMEOUT)
            .unwrap_or_default()
    }
}

/// Impl khi feature `os-enchant` **TAT** (mac dinh: CI khong co libenchant):
/// khong FFI nao duoc compile/link, moi tu coi la dung, `suggest` rong — hanh
/// vi y het `assist::dict::NoDictionary` (fail-open, khong hoi quy = AC-1).
/// Lop ngan sach van con va duoc test doc lap (khong can libenchant).
#[cfg(not(feature = "os-enchant"))]
impl EnchantSpellChecker {
    /// Luon [`DictError::NoDictionary`]: feature tat => khong link libenchant
    /// => khong the co tu dien OS. Caller must fallback `NoDictionary`.
    pub fn new(_language_tag: &str) -> Result<Self, DictError> {
        Err(DictError::NoDictionary)
    }

    /// Luon `false`: feature tat => khong co capability nao.
    #[must_use]
    pub fn is_supported(_language_tag: &str) -> bool {
        false
    }

    /// Luon `true` (fail-open): moi tu dung => khong bao loi spelling.
    #[must_use]
    pub fn contains(&self, _word: &str) -> bool {
        self.budget.anchor();
        true
    }

    /// Luon rong (fail-open); van rat ngan sach de hanh vi dong nhat.
    #[must_use]
    pub fn suggest(&self, _word: &str, _limit: usize) -> Vec<String> {
        let _ = self.budget.allow();
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// backend: FFI thuan den libenchant-2 + worker thread — toan bo C nam o day
// CHI compile khi feature `os-enchant` bat (feature mac dinh TAT vi CI khong
// co libenchant; bat feature chi tren may build Linux co libenchant-2-dev).
// ---------------------------------------------------------------------------
#[cfg(feature = "os-enchant")]
mod backend {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int};
    use std::ptr;
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Timeout cho moi loi goi enchant (ms) — P4-0 D36: `<= 50 ms`, het gio =
    /// fail-open. Enchant in-process nhung C co the chim (tu dai hunspell,
    /// backend ngoai cham — L5) nen van can timeout phia caller.
    pub(super) const WORK_TIMEOUT: Duration = Duration::from_millis(50);

    /// Timeout cho buoc broker init + resolve tag (broker doc thu muc config
    /// + nap provider .so — co the cham o lan dau).
    const PROBE_TIMEOUT: Duration = Duration::from_millis(50);

    /// Cap cung so goi y tra ve cho **mot** lan `suggest` (tranh cap phat lon).
    pub(super) const MAX_SUGGEST_RESULTS: usize = 8;

    /// Thu tu backend de enchant nap: `hunspell,nuspell,aspell` (gach y theo
    /// L5 — chi backend trong tien trinh, bo qua backend la/ai khac).
    const BACKEND_ORDERING: &str = "hunspell,nuspell,aspell";

    /// Opaque `EnchantBroker` (dinh nghia that trong `enchant.h`).
    #[repr(C)]
    struct EnchantBroker {
        _private: [u8; 0],
    }

    /// Opaque `EnchantDict` (dinh nghia that trong `enchant.h`).
    #[repr(C)]
    struct EnchantDict {
        _private: [u8; 0],
    }

    /// `ssize_t` — trên mọi target Linux được hỗ trợ (LP64 arm64/amd64 và
    /// ILP32 arm32) `ssize_t` cùng kích thước `isize` (C: `long`/`int`).
    type SsizeT = isize;

    // Symbol an toan khai tay: moi symbol deu `enchant_*`, KHONG chung symbol
    // runtime cua std (bai hoc Fassist-104 chi chan `open`/`close`) => khong
    // can crate `libc`, giu crate 0-dep. `#[link] = link DONG (`-lenchant-2`,
    // LGPL) chi co hieu luc khi feature `os-enchant` bat (module nay cfg tat).
    #[link(name = "enchant-2")]
    extern "C" {
        #[link_name = "enchant_broker_init"]
        fn raw_broker_init() -> *mut EnchantBroker;

        #[link_name = "enchant_broker_free"]
        fn raw_broker_free(broker: *mut EnchantBroker);

        #[link_name = "enchant_broker_dict_exists"]
        fn raw_dict_exists(broker: *mut EnchantBroker, tag: *const c_char) -> c_int;

        #[link_name = "enchant_broker_request_dict"]
        fn raw_request_dict(broker: *mut EnchantBroker, tag: *const c_char) -> *mut EnchantDict;

        #[link_name = "enchant_broker_free_dict"]
        fn raw_free_dict(broker: *mut EnchantBroker, dict: *mut EnchantDict);

        #[link_name = "enchant_broker_set_ordering"]
        fn raw_set_ordering(
            broker: *mut EnchantBroker,
            tag: *const c_char,
            ordering: *const c_char,
        );

        #[link_name = "enchant_dict_check"]
        fn raw_dict_check(dict: *mut EnchantDict, word: *const c_char, len: SsizeT) -> c_int;

        #[link_name = "enchant_dict_suggest"]
        fn raw_dict_suggest(
            dict: *mut EnchantDict,
            word: *const c_char,
            len: SsizeT,
            out_n_suggs: *mut usize,
        ) -> *mut *mut c_char;

        #[link_name = "enchant_dict_free_string_list"]
        fn raw_free_string_list(dict: *mut EnchantDict, string_list: *mut *mut c_char);
    }

    /// Broker RAII: `enchant_broker_init` / `enchant_broker_free`.
    struct Broker {
        ptr: *mut EnchantBroker,
    }

    impl Broker {
        /// `None` = libenchant khong khoi tao duoc (thieu config dir...) =>
        /// caller fail-open (`NoDictionary`).
        fn init() -> Option<Self> {
            // SAFETY: ham thuan khong tham so; kiem tra NULL theo hop dong.
            let ptr = unsafe { raw_broker_init() };
            if ptr.is_null() {
                return None;
            }
            Some(Self { ptr })
        }

        /// Broker co dict cho `tag` (dang `xx_YY` cua enchant) khong?
        /// Hop dong `enchant_broker_dict_exists`: 1 = co, 0 = khong.
        fn dict_exists(&self, tag: &str) -> bool {
            // Tag co the xau/qua dai — CString that bai => coi nhu khong co.
            let Ok(c_tag) = CString::new(tag) else {
                return false;
            };
            // SAFETY: `c_tag` song suot loi goi; `self.ptr` con song.
            unsafe { raw_dict_exists(self.ptr, c_tag.as_ptr()) == 1 }
        }

        /// Dat thu tu backend cho tag — TRUOC `request_dict` (L5). Loi im
        /// lang (ham void): enchant bo qua ordering khong hop le.
        fn set_ordering(&self, tag: &str) {
            let (Ok(c_tag), Ok(c_order)) = (CString::new(tag), CString::new(BACKEND_ORDERING))
            else {
                return;
            };
            // SAFETY: 2 CString song suot loi goi; `self.ptr` con song.
            unsafe { raw_set_ordering(self.ptr, c_tag.as_ptr(), c_order.as_ptr()) };
        }

        /// Request dict cho tag da resolve; `None` => caller NoDictionary.
        fn request(&self, tag: &str) -> Option<Dict> {
            let Ok(c_tag) = CString::new(tag) else {
                return None;
            };
            // SAFETY: CString song suot loi goi; fail => NULL theo hop dong.
            let ptr = unsafe { raw_request_dict(self.ptr, c_tag.as_ptr()) };
            if ptr.is_null() {
                return None;
            }
            Some(Dict {
                broker: self.ptr,
                ptr,
            })
        }
    }

    impl Drop for Broker {
        fn drop(&mut self) {
            // SAFETY: can bang dung 1 `enchant_broker_init` thanh cong o tren.
            // Trong worker, `Dict` khong bao gio ton dai hon broker (khai bao
            // truoc, drop sau).
            unsafe { raw_broker_free(self.ptr) };
        }
    }

    /// Dict RAII: `request_dict` / `free_dict`. Giu con tro broker dang plain
    /// pointer (`EnchantBroker` la opaque C, khong dung reference duoc); thu
    /// tu khai bao trong worker bao dam broker song den khi dict drop.
    struct Dict {
        broker: *mut EnchantBroker,
        ptr: *mut EnchantDict,
    }

    impl Dict {
        /// `true` neu `word` dung chinh ta; **fail-open** = `true`.
        ///
        /// Hop dong `enchant_dict_check` (`enchant.h`): 0 = tu dung, so duong
        /// = tu sai, so am = tham so sai / loi. So am quy ve `true`: thien lech
        /// fail-open (de im lang con hon bao sai HET = tai hoa FPR dao
        /// nguoc).
        fn contains(&self, word: &str) -> bool {
            if word.is_empty() {
                return true;
            }
            // SAFETY: `word` song suot loi goi; `len` truyen bang BYTE (hop
            // dong header; `-1` chi danh cho chuoi NUL-terminated).
            let verdict =
                unsafe { raw_dict_check(self.ptr, word.as_ptr().cast(), word.len() as SsizeT) };
            verdict <= 0
        }

        /// Goi y cho 1 tu + giai phong day bang `enchant_dict_free_string_list`
        /// (W4: moi day ma enchant cap phai free dung 1 lan).
        fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
            if word.is_empty() || limit == 0 {
                return Vec::new();
            }
            // SAFETY: `word` song suot loi goi; `out_n_suggs = NULL` du phep
            // theo hop dong header ("location ... or null").
            let list = unsafe {
                raw_dict_suggest(
                    self.ptr,
                    word.as_ptr().cast(),
                    word.len() as SsizeT,
                    ptr::null_mut(),
                )
            };
            if list.is_null() {
                return Vec::new();
            }
            let mut out = Vec::new();
            let mut i = 0usize;
            while out.len() < limit {
                // SAFETY: doc phan tu thu `i` cua mang NULL-terminated ma
                // enchant cap — chi doc trong mang da tu khong vuot limit.
                let item: *mut c_char = unsafe { list.add(i).read() };
                if item.is_null() {
                    break;
                }
                // SAFETY: con tro NUL-terminated UTF-8 do enchant cap (hop
                // dong "strings are all UTF-8" trong header). Tu co UTF-8 hong
                // duoc bo qua (im lang con hon dung du lieu hong).
                let text = unsafe { CStr::from_ptr(item) };
                if let Ok(text) = text.to_str() {
                    out.push(text.to_owned());
                }
                i += 1;
            }
            // SAFETY: dung 1 lan cho dung con tro `list` ma enchant cap (W4).
            unsafe { raw_free_string_list(self.ptr, list) };
            out
        }
    }

    impl Drop for Dict {
        fn drop(&mut self) {
            // SAFETY: broker van song (trong worker broker duoc khai bao TRUOC
            // dict => drop SAU dict); `free_dict` dung 1 lan cho dict nay.
            unsafe { raw_free_dict(self.broker, self.ptr) };
        }
    }

    /// Lenh gui sang worker thread (chi du lieu "tho", khong gui con tro C).
    #[derive(Debug)]
    pub(super) enum Job {
        /// `contains` — reply: `true` = tu dung (hoac fail-open).
        Contains {
            word: String,
            reply: SyncSender<bool>,
        },
        /// `suggest` — reply: goi y (rong = fail-open).
        Suggest {
            word: String,
            limit: usize,
            reply: SyncSender<Vec<String>>,
        },
    }

    /// Handle cua worker thread (chi giu kenh gui; thread tu ket thuc khi drop).
    #[derive(Debug)]
    pub(super) struct Worker {
        pub(super) tx: SyncSender<Job>,
    }

    /// Cache "tag co duoc ho tro khong" - cong capability chi hoi 1 lan / phien
    /// (P4-10 AC-2). Khong cache du lieu nguoi dung, chi tag + bool.
    static SUPPORT_CACHE: Mutex<Vec<(String, bool)>> = Mutex::new(Vec::new());

    pub(super) fn support_cached(language_tag: &str) -> Option<bool> {
        let cache = SUPPORT_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .iter()
            .find(|(tag, _)| tag.eq_ignore_ascii_case(language_tag))
            .map(|(_, supported)| *supported)
    }

    fn store_support(language_tag: &str, supported: bool) {
        let mut cache = SUPPORT_CACHE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !cache
            .iter()
            .any(|(tag, _)| tag.eq_ignore_ascii_case(language_tag))
        {
            cache.push((language_tag.to_owned(), supported));
        }
    }

    /// Doi BCP-47 (`vi-VN`, `zh-Hant-TW`) sang dinh dang tag cua enchant
    /// (`xx_YY`: ngon ngu thuong, script Titlecase, vung UPPERCASE), tra kem
    /// bien the ngon ngu goc (`vi_VN` → `vi`) vi distro dat tu dien hunspell
    /// theo ca hai kieu (P4-10 §4.5 thu "vi" truoc, roi "vi_VN").
    fn tag_variants(language_tag: &str) -> Vec<String> {
        let tag = language_tag.trim();
        if tag.is_empty() {
            return Vec::new();
        }
        let parts: Vec<&str> = tag.split(['-', '_']).filter(|p| !p.is_empty()).collect();
        let Some((lang, rest)) = parts.split_first() else {
            return Vec::new();
        };
        let mut normalized = String::with_capacity(tag.len());
        normalized.push_str(&lang.to_lowercase());
        for (i, part) in rest.iter().enumerate() {
            normalized.push('_');
            match i {
                // vi tri 1 = subtag script (BCP-47 `Hant`, `Latn`) → Titlecase.
                1 => {
                    let mut chars = part.chars();
                    if let Some(first) = chars.next() {
                        normalized.extend(first.to_uppercase());
                        normalized.push_str(&chars.as_str().to_lowercase());
                    }
                }
                // vi tri >= 2 = vung / phan mo rong → UPPERCASE.
                _ => normalized.push_str(&part.to_uppercase()),
            }
        }
        let mut out = vec![normalized.clone()];
        if let Some((lang, _rest)) = normalized.split_once('_') {
            let base = lang.to_owned();
            if !out.contains(&base) {
                out.push(base);
            }
        }
        out
    }

    /// Chon bien the tag dau tien ma broker co tu dien.
    fn resolve_tag(broker: &Broker, language_tag: &str) -> Option<String> {
        tag_variants(language_tag)
            .into_iter()
            .find(|variant| broker.dict_exists(variant))
    }

    /// Tao worker thread va doi ket qua "san sang" (co timeout).
    ///
    /// `want_checker = false` chi hoi cong capability (KHONG tao `EnchantDict`
    /// — tiet kiem nap tu dien hunspell vao RAM chi vi probe).
    /// `None` = khong spawn duoc thread hoac probe qua han => caller fail-open.
    pub(super) fn spawn_worker(language_tag: &str, want_checker: bool) -> Option<(Worker, bool)> {
        let tag = language_tag.to_owned();
        let (job_tx, job_rx) = sync_channel::<Job>(1);
        let (ready_tx, ready_rx) = sync_channel::<bool>(1);
        std::thread::Builder::new()
            .name("textvn-os-spell".to_owned())
            .spawn(move || worker_main(&tag, want_checker, job_rx, ready_tx))
            .ok()?;
        let ready = ready_rx.recv_timeout(PROBE_TIMEOUT).ok()?;
        Some((Worker { tx: job_tx }, ready))
    }

    /// Than worker: **toan bo** vong doi libenchant nam trong ham nay (1 thread).
    ///
    /// Con tro `EnchantBroker`/`EnchantDict` KHONG BAO GIO qua channel — chi
    /// `String`/`Vec<String>` di chuyen (moi thu dung trong 1 worker thread,
    /// khong phu thuoc thread-safety cua backend enchant).
    fn worker_main(
        language_tag: &str,
        want_checker: bool,
        jobs: Receiver<Job>,
        ready: SyncSender<bool>,
    ) {
        // Da biet khong ho tro theo cache => khong ton broker init + doc thu
        // muc config (giong Windows: cache chot truoc).
        if support_cached(language_tag) == Some(false) {
            ready.send(false).ok();
            return;
        }
        let Some(broker) = Broker::init() else {
            ready.send(false).ok();
            return;
        };
        let Some(tag) = resolve_tag(&broker, language_tag) else {
            store_support(language_tag, false);
            ready.send(false).ok();
            return;
        };
        // Dat thu tu backend TRUOC request_dict (L5).
        broker.set_ordering(&tag);
        store_support(language_tag, true);
        if !want_checker {
            ready.send(true).ok();
            return; // chi probe => `broker` drop tai day, khong giu RAM dict.
        }
        let Some(dict) = broker.request(&tag) else {
            // "exists" nhung "request" that bai (dict hong, RAM...) => nho luon.
            store_support(language_tag, false);
            ready.send(false).ok();
            return;
        };
        if ready.send(true).is_err() {
            // Caller da bo (probe timeout) => khong giu thread; dict + broker
            // drop ngay tai day (drop dict => drop broker).
            return;
        }
        while let Ok(job) = jobs.recv() {
            handle_job(&dict, job);
        }
        // `dict` + `broker` duoc drop tai day, tren chinh thread nay.
    }

    fn handle_job(dict: &Dict, job: Job) {
        match job {
            Job::Contains { word, reply } => {
                reply.send(dict.contains(&word)).ok();
            }
            Job::Suggest { word, limit, reply } => {
                reply.send(dict.suggest(&word, limit)).ok();
            }
        }
    }

    #[cfg(test)]
    pub(super) fn tag_variants_for_test(language_tag: &str) -> Vec<String> {
        tag_variants(language_tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_rong_khong_co_tu_dien() {
        // Tag rong => fail nhanh, khong probe, khong spawn thread.
        assert!(!EnchantSpellChecker::is_supported(""));
        assert!(!EnchantSpellChecker::is_supported("   "));
        assert!(matches!(
            EnchantSpellChecker::new("  "),
            Err(DictError::NoDictionary)
        ));
        assert!(crate::os_provider_linux("   ").is_none());
    }

    #[test]
    fn tag_khong_hop_le_khong_co_tu_dien() {
        // "zz-ZZ" khong phai ngon ngu co that (test toan cac may, giong Windows).
        assert!(!EnchantSpellChecker::is_supported("zz-ZZ"));
        assert!(matches!(
            EnchantSpellChecker::new("zz-ZZ"),
            Err(DictError::NoDictionary)
        ));
    }

    #[test]
    fn suggest_chua_contains_thi_khong_goi() {
        let budget = SuggestBudget::new();
        assert!(!budget.allow(), "cua so ngan sach chua mo");
    }

    #[test]
    fn suggest_bi_cap_so_lan_goi() {
        let budget = SuggestBudget::new();
        budget.anchor();
        for i in 0..SUGGEST_CALLS_PER_CHECK {
            assert!(budget.allow(), "lan {i} phai duoc phep");
        }
        assert!(!budget.allow(), "vuot cap => fail-open (rong)");
    }

    #[test]
    fn suggest_het_ngan_sach_thoi_gian() {
        let budget = SuggestBudget::new();
        budget.anchor();
        std::thread::sleep(SUGGEST_BUDGET + Duration::from_millis(20));
        assert!(!budget.allow(), "qua 50 ms => fail-open (rong)");
    }

    #[test]
    fn contains_khong_reset_cua_so_con_han() {
        // Khac Windows: don vi goi la moi tu — `anchor` giua cua so CON HAN
        // khong duoc reset bo dem, chi cua so moi (da cot han) moi dat lai.
        let budget = SuggestBudget::new();
        budget.anchor();
        for _ in 0..SUGGEST_CALLS_PER_CHECK {
            assert!(budget.allow());
        }
        assert!(!budget.allow());
        budget.anchor();
        assert!(!budget.allow(), "anchor giua cua so khong reset");
    }

    /// Doi tag BCP-47 -> dinh dang `xx_YY` cua enchant (thuan Rust, khong FFI).
    #[cfg(all(target_os = "linux", feature = "os-enchant"))]
    #[test]
    fn tag_variants_bcp47_sang_dinh_dang_xx_yy() {
        assert_eq!(backend::tag_variants_for_test("vi"), vec!["vi"]);
        assert_eq!(backend::tag_variants_for_test("vi-VN"), vec!["vi_VN", "vi"]);
        assert_eq!(backend::tag_variants_for_test("vi_VN"), vec!["vi_VN", "vi"]);
        assert_eq!(
            backend::tag_variants_for_test(" VI-vn "),
            vec!["vi_VN", "vi"]
        );
        assert_eq!(backend::tag_variants_for_test("en-us"), vec!["en_US", "en"]);
        assert_eq!(
            backend::tag_variants_for_test("zh-Hant-TW"),
            vec!["zh_Hant_TW", "zh"]
        );
        assert!(backend::tag_variants_for_test("").is_empty());
        assert!(backend::tag_variants_for_test("  ---  ").is_empty());
    }

    /// Feature `os-enchant` TAT (mac dinh, CI khong co libenchant): provider
    /// tro ve trang thai NoDictionary tron ve (AC-1 — khong hoi quy).
    #[cfg(all(target_os = "linux", not(feature = "os-enchant")))]
    #[test]
    fn feature_tat_thi_fail_open_toan_phan() {
        assert!(!EnchantSpellChecker::is_supported("vi-VN"));
        assert!(!EnchantSpellChecker::is_supported("vi"));
        assert!(matches!(
            EnchantSpellChecker::new("vi-VN"),
            Err(DictError::NoDictionary)
        ));
        // Feature tat => struct khong co truong `tx`, tao truc tiep de thu
        // duong fail-open cua `contains`/`suggest` (same-module truy cap).
        let checker = EnchantSpellChecker {
            budget: SuggestBudget::new(),
        };
        assert!(checker.contains("helo"), "feature tat => moi tu dung");
        assert!(checker.suggest("helo", 3).is_empty());
    }

    /// Test tich hop **co dieu kien**: chi chay khi may co tu dien `en-US`
    /// (hunspell-us). May khong co (thuong) => bo qua; khong assert cung noi
    /// dung backend — chi hop dong cap `suggest` theo limit.
    #[cfg(all(target_os = "linux", feature = "os-enchant"))]
    #[test]
    fn en_us_neu_may_co_thi_provider_hoat_dong() {
        if !EnchantSpellChecker::is_supported("en-US") {
            return;
        }
        let Ok(checker) = EnchantSpellChecker::new("en-US") else {
            return; // "exists" nhung tao that bai => fail-open, khong fail test
        };
        // Tu co trong dict en => contains phai true (hop dong 0 = dung).
        assert!(checker.contains("hello"));
        let suggestions = checker.suggest("helo", 3);
        assert!(suggestions.len() <= 3, "phai cat theo limit");
    }

    /// Test tich hop tieng Viet (co dieu kien — tren may co `hunspell-vi`,
    /// thong thuong chi Fedora): khong assert cung noi dung, chi dam bao
    /// duong goi khong panic. May chua cai dict => bo qua.
    #[cfg(all(target_os = "linux", feature = "os-enchant"))]
    #[test]
    fn vi_neu_may_co_thi_duong_goi_khong_panic() {
        let vi =
            EnchantSpellChecker::is_supported("vi-VN") || EnchantSpellChecker::is_supported("vi");
        if !vi {
            return;
        }
        if let Ok(checker) = EnchantSpellChecker::new("vi") {
            let _ = checker.contains("hoà");
            let _ = checker.suggest("hoà", 3);
        }
    }
}
