// SPDX-License-Identifier: GPL-3.0-or-later
//! Provider spell-check **Windows** (Spell Checking API, Win8+) - chi co tren Windows.
//!
//! Day la **adapter o bien** (P4-0 **D32**): no chi lo ra API "kiem tra / goi y"
//! de crate khac (vi du `ffi`) cam vao `assist::dict::Dictionary`. Crate `dict`
//! khong biet `assist`, `assist` khong biet Windows.
//!
//! ## Vi sao **khong** dung cho tieng Viet o v1
//!
//! Windows **mac dinh khong co** language pack `vi-VN`: do thuc nghiem tren
//! Windows 11 sach cho `IsSupported("vi-VN") = FALSE`,
//! `CreateSpellChecker("vi-VN") = E_INVALIDARG` (P4-10 §2.5). Vi vay:
//!
//! - [`WindowsSpellChecker::new`] kiem tra cong `IsSupported` **mot lan / phien**
//!   (cache toan process) va tra [`DictError::NoDictionary`] khi khong co.
//! - `None`/`NoDictionary` nghia la caller dung `assist::dict::NoDictionary`:
//!   moi tu deu coi la dung => **khong bao loi spelling nao**. Day la duong
//!   **mac dinh** tren Windows, khong phai nhanh loi (P4-10 AC-1).
//! - Tuyet doi **khong** muon checker `en-US` thay the: do duoc 3/8 tu tieng
//!   Viet bi bao sai (P4-10 §2.5) => FPR khong the chap nhan.
//!
//! ## Chi phi (do thuc nghiem, P4-10 §2.5)
//!
//! | Phep do | Chi phi |
//! |---|---|
//! | `Check()` warm, ca cau 87 code unit | **0,057 ms** |
//! | `Suggest()` warm, tu da cache | **3,07 ms** |
//! | `Suggest()` warm, tu moi | **6,91 ms** |
//!
//! => `Check` du re de goi **1 lan / cau**, con `Suggest` dat gap **~50-100x**.
//! Vi vay:
//!
//! - [`WindowsSpellChecker::check`] la duong chinh (chi lay loi `REPLACE`/`DELETE`,
//!   **bo qua** `GET_SUGGESTIONS` - no doi hoi goi `Suggest` dat).
//! - [`WindowsSpellChecker::suggest`] bi **cap cung**
//!   [`SUGGEST_CALLS_PER_CHECK`] lan trong cua so [`SUGGEST_BUDGET`] (50 ms) do
//!   moi lan `check()` mo ra; het han hoac vuot cap => tra rong (fail-open).
//!
//! **Do thuc nghiem 2026-10-21 (Windows 11, checker `en-US`):** cac tu sai thong
//! thuong (`helo`, `teh`, `recieve`, `vietnam`, ...) deu tra
//! `CorrectiveAction = GET_SUGGESTIONS`; `DELETE` co voi tu lap (`the the`);
//! khong thay `REPLACE` trong mau. Vi vay `check` theo dung dac ta (chi
//! `REPLACE`/`DELETE`) **khong** bat duoc loi chinh ta thong thuong - no chi bat
//! truong hop OS da co cach sua duy nhat hoac tu lap. Do la ly do nua de provider
//! Windows chi la **tuy chon cong them** (phan co gia tri la `suggest` co ngan
//! sach); phai do lai khi co language pack `vi-VN`.
//!
//! ## COM / thread / timeout (bat buoc, P4-0 **D36**)
//!
//! Provider cua Windows co the chay **out-of-proc** (RPC) nen mot loi goi COM co
//! the treo khong gioi han. Do do:
//!
//! - Toan bo `CoInitializeEx` -> `CoCreateInstance` -> `Check`/`Suggest` ->
//!   `Release` -> `CoUninitialize` chay tren **mot worker thread duy nhat**
//!   (COM apartment). Khong bao gio chuyen `ISpellChecker` sang thread khac -
//!   chi `String`/`Vec<u16>` di qua channel.
//! - Moi loi goi co **timeout 50 ms** o phia caller (`recv_timeout`); hang doi
//!   job chi 1 slot va caller dung `try_send` nen **khong bao gio block** khi
//!   worker dang treo.
//! - Moi loi/qua han => **fail-open**: `check` = khong loi nao, `suggest` = rong.
//!
//! ## Don vi offset
//!
//! `StartIndex`/`Length` cua Windows la **UTF-16 code unit** (do thuc nghiem
//! bang ky tu astral), khong phai `char` Unicode cua `assist` (D27). Adapter
//! **co y** tra nguyen UTF-16 ([`SpellIssue`]) - caller phai doi sang `char`
//! truoc khi qua FFI.
//!
//! ## Privacy (S2)
//!
//! Khong log `text`/`word` o bat ky nhanh nao (ke ca khi COM loi) - chi duoc
//! log/do ma loi. Luu y: provider out-of-proc nghia la **text roi khoi tien
//! trinh TextVN** (P4-10 §2.6 W7, can neu trong privacy notice).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::core::{HSTRING, PWSTR};
use windows::Win32::Globalization::{
    ISpellChecker, ISpellCheckerFactory, ISpellingError, SpellCheckerFactory, CORRECTIVE_ACTION,
    CORRECTIVE_ACTION_DELETE, CORRECTIVE_ACTION_REPLACE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};

use crate::DictError;

/// Timeout cho moi loi goi COM (ms) - P4-0 D36: `<= 50 ms`, het gio = fail-open.
const WORK_TIMEOUT: Duration = Duration::from_millis(50);

/// Timeout cho buoc tao checker + cong `IsSupported` (nguoi ~9,5 ms do duoc).
const PROBE_TIMEOUT: Duration = Duration::from_millis(50);

/// Ngan sach thoi gian cho `suggest` sau moi lan `check()` (P4-0 D36).
///
/// Cua so mo o dau `check()`; qua moc nay `suggest` tra rong.
pub const SUGGEST_BUDGET: Duration = Duration::from_millis(50);

/// Cap cung so lan `suggest` trong mot cua so ngan sach (P4-10 §5.3).
///
/// 3 x 6,9 ms ~ 21 ms => van trong 50 ms.
pub const SUGGEST_CALLS_PER_CHECK: usize = 3;

/// Cap cung so goi y tra ve cho **mot** lan `suggest` (tranh cap phat lon).
const MAX_SUGGEST_RESULTS: usize = 8;

/// Mot loi spelling do Windows tra ve.
///
/// `start_utf16`/`len_utf16` la **UTF-16 code unit** (khong phai `char`) - caller
/// phai doi sang char truoc khi dua qua FFI (`start`/`end` cua
/// `ime_assist_issue_v1` la char, D27).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellIssue {
    /// Vi tri bat dau, tinh bang UTF-16 code unit.
    pub start_utf16: usize,
    /// Do dai, tinh bang UTF-16 code unit.
    pub len_utf16: usize,
    /// Tu thay the; **rong** = xoa tu (`CorrectiveAction` = `DELETE`).
    pub replacement: String,
}

/// Provider spell-check Windows dung chung 1 worker thread COM.
///
/// Tao bang [`WindowsSpellChecker::new`]. Provider **khong** tu bao loi: khi may
/// khong co tu dien cho `language_tag` (vi du `vi-VN` - mac dinh tren Windows)
/// thi `new` tra [`DictError::NoDictionary`] va caller phai fallback
/// `assist::dict::NoDictionary` (moi tu dung => khong bao loi spelling).
///
/// Moi phuong thuc deu **fail-open**: loi COM, timeout hoac worker chet => tra
/// ket qua rong, khong panic, khong block qua 50 ms.
#[derive(Debug)]
pub struct WindowsSpellChecker {
    tx: SyncSender<Job>,
    budget: SuggestBudget,
}

/// Lenh gui sang worker thread (chi du lieu "tho", khong gui interface COM).
#[derive(Debug)]
enum Job {
    Check {
        text: Vec<u16>,
        reply: SyncSender<Vec<SpellIssue>>,
    },
    Suggest {
        word: Vec<u16>,
        limit: usize,
        reply: SyncSender<Vec<String>>,
    },
}

impl WindowsSpellChecker {
    /// Tao checker cho `language_tag` (BCP-47, vi du `vi-VN`).
    ///
    /// Cong capability `IsSupported` duoc hoi **toi da mot lan / tag / process**
    /// (cache tinh); sau do tao `ISpellChecker` tren worker thread.
    ///
    /// # Errors
    ///
    /// - [`DictError::NoDictionary`] neu OS khong co tu dien cho tag (truong hop
    ///   **mac dinh** voi `vi-VN`), neu khong tao duoc COM factory, hoac neu
    ///   probe qua han 50 ms. Khi do caller dung `assist::dict::NoDictionary`
    ///   => **khong** bao loi spelling nao.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(windows)] {
    /// use textvn_dict::os_windows::WindowsSpellChecker;
    ///
    /// // Tren Windows mac dinh (khong co language pack vi-VN) => Err(NoDictionary).
    /// if let Ok(checker) = WindowsSpellChecker::new("vi-VN") {
    ///     let issues = checker.check(&"cau  van".encode_utf16().collect::<Vec<_>>());
    ///     let _ = issues;
    /// }
    /// # }
    /// ```
    pub fn new(language_tag: &str) -> Result<Self, DictError> {
        if language_tag.trim().is_empty() {
            return Err(DictError::NoDictionary);
        }
        let (worker, ready) = spawn_worker(language_tag, true).ok_or(DictError::NoDictionary)?;
        if !ready {
            return Err(DictError::NoDictionary);
        }
        Ok(Self {
            tx: worker.tx,
            budget: SuggestBudget::new(),
        })
    }

    /// Cong capability: OS co tu dien cho `language_tag` khong.
    ///
    /// Re sau lan hoi dau (cache). `false` = khong co tu dien => caller fallback
    /// `NoDictionary` (khong bao loi spelling). Moi loi COM cung tra `false`
    /// (fail-safe: tha im lang con hon bao sai).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(windows)] {
    /// use textvn_dict::os_windows::WindowsSpellChecker;
    ///
    /// // Windows mac dinh: `vi-VN` KHONG duoc ho tro.
    /// if !WindowsSpellChecker::is_supported("vi-VN") {
    ///     // => dung assist::dict::NoDictionary, khong bao loi spelling.
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_supported(language_tag: &str) -> bool {
        if language_tag.trim().is_empty() {
            return false;
        }
        if let Some(supported) = cached_support(language_tag) {
            return supported;
        }
        match spawn_worker(language_tag, false) {
            Some((_worker, supported)) => supported,
            None => false,
        }
    }

    /// Kiem tra ca doan text (UTF-16) va tra cac loi **sua duoc**.
    ///
    /// Chi tra loi co `CorrectiveAction` = `REPLACE`/`DELETE`; **bo qua**
    /// `GET_SUGGESTIONS` (phai goi `Suggest`, dat 3-7 ms/tu).
    ///
    /// Goi `check` **mo cua so ngan sach** cho `suggest` (xem [`SUGGEST_BUDGET`]).
    /// Timeout/loi COM => `Vec` rong (fail-open).
    ///
    /// Doc-comment ve offset: xem [`SpellIssue`] - `start`/`len` la UTF-16.
    #[must_use]
    pub fn check(&self, text_utf16: &[u16]) -> Vec<SpellIssue> {
        self.budget.open_window();
        if text_utf16.is_empty() {
            return Vec::new();
        }
        let (reply, result) = sync_channel(1);
        let job = Job::Check {
            text: text_utf16.to_vec(),
            reply,
        };
        // `try_send`: neu worker dang ban/treo thi bo qua ngay, KHONG block caller.
        if self.tx.try_send(job).is_err() {
            return Vec::new();
        }
        result.recv_timeout(WORK_TIMEOUT).unwrap_or_default()
    }

    /// Goi y tu thay the cho `word` (UTF-16), toi da `limit` ket qua.
    ///
    /// **Co ngan sach** (bat buoc vi `Suggest` ~3-7 ms/tu):
    ///
    /// - chi duoc goi toi da [`SUGGEST_CALLS_PER_CHECK`] lan trong cua so
    ///   [`SUGGEST_BUDGET`] mo ra boi lan `check()` gan nhat;
    /// - chua goi `check()` (cua so chua mo) / het han / vuot cap => `Vec` rong.
    ///
    /// `limit` bi chan tren `MAX_SUGGEST_RESULTS`; timeout/loi COM => rong
    /// (fail-open, khong phai loi).
    #[must_use]
    pub fn suggest(&self, word_utf16: &[u16], limit: usize) -> Vec<String> {
        let limit = limit.min(MAX_SUGGEST_RESULTS);
        if limit == 0 || word_utf16.is_empty() || !self.budget.allow() {
            return Vec::new();
        }
        let (reply, result) = sync_channel(1);
        let job = Job::Suggest {
            word: word_utf16.to_vec(),
            limit,
            reply,
        };
        if self.tx.try_send(job).is_err() {
            return Vec::new();
        }
        result.recv_timeout(WORK_TIMEOUT).unwrap_or_default()
    }
}

/// Ngan sach cho `suggest`: cua so thoi gian + dem so lan goi.
///
/// Tach rieng de test duoc tren may **khong co** spellchecker he thong.
#[derive(Debug)]
struct SuggestBudget {
    used: AtomicUsize,
    deadline: Mutex<Instant>,
}

impl SuggestBudget {
    /// Chua mo cua so: moi `suggest` deu bi tu choi cho toi lan `check()` dau.
    fn new() -> Self {
        Self {
            used: AtomicUsize::new(SUGGEST_CALLS_PER_CHECK),
            deadline: Mutex::new(Instant::now()),
        }
    }

    /// Mo cua so ngan sach moi (goi o dau moi `check()`).
    fn open_window(&self) {
        self.used.store(0, Ordering::Relaxed);
        *self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Instant::now() + SUGGEST_BUDGET;
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

/// Cache "tag co duoc ho tro khong" - cong capability chi hoi 1 lan / phien
/// (P4-10 AC-2). Khong cache du lieu nguoi dung, chi tag + bool.
static SUPPORT_CACHE: Mutex<Vec<(String, bool)>> = Mutex::new(Vec::new());

fn cached_support(language_tag: &str) -> Option<bool> {
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

/// Handle cua worker thread (chi giu dau gui; thread tu ket thuc khi bi drop).
#[derive(Debug)]
struct Worker {
    tx: SyncSender<Job>,
}

/// Tao worker thread va doi ket qua "san sang" (co timeout).
///
/// `want_checker = false` chi hoi cong capability (khong tao `ISpellChecker`).
/// `None` = khong spawn duoc thread hoac probe qua han => caller fail-open.
fn spawn_worker(language_tag: &str, want_checker: bool) -> Option<(Worker, bool)> {
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

/// Than worker: **toan bo** vong doi COM nam trong ham nay (1 thread).
fn worker_main(
    language_tag: &str,
    want_checker: bool,
    jobs: Receiver<Job>,
    ready: SyncSender<bool>,
) {
    let _apartment = ComApartment::initialize();

    // Da biet la khong ho tro => khong ton 9,5 ms tao COM factory.
    if cached_support(language_tag) == Some(false) {
        ready.send(false).ok();
        return;
    }

    let Some((factory, supported)) = open_factory_and_probe(language_tag) else {
        ready.send(false).ok();
        return;
    };
    if !supported || !want_checker {
        ready.send(supported).ok();
        return;
    }

    let Some(checker) = create_checker(&factory, language_tag) else {
        // Ho tro nhung tao that bai (RPC loi, pack hong...) => nho luon.
        store_support(language_tag, false);
        ready.send(false).ok();
        return;
    };
    if ready.send(true).is_err() {
        // Caller da bo (timeout) => khong giu thread.
        return;
    }

    while let Ok(job) = jobs.recv() {
        handle_job(&checker, job);
    }
    // `checker` + COM apartment duoc drop tai day, tren chinh thread nay.
}

/// Tao `ISpellCheckerFactory` va hoi `IsSupported` (co cache).
fn open_factory_and_probe(language_tag: &str) -> Option<(ISpellCheckerFactory, bool)> {
    let factory = create_factory()?;
    let supported = match cached_support(language_tag) {
        Some(supported) => supported,
        None => {
            let supported = is_supported_by(&factory, language_tag);
            store_support(language_tag, supported);
            supported
        }
    };
    Some((factory, supported))
}

fn create_factory() -> Option<ISpellCheckerFactory> {
    // SAFETY: `SpellCheckerFactory` la in-proc COM server (P4-10 §2.3); goi tren
    // worker thread da `CoInitializeEx`, ket qua chi dung trong thread nay.
    unsafe { CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER) }.ok()
}

fn is_supported_by(factory: &ISpellCheckerFactory, language_tag: &str) -> bool {
    // SAFETY: `factory` con song; tag la BCP-47, khong chua NUL.
    unsafe { factory.IsSupported(&HSTRING::from(language_tag)) }
        .map(|supported| supported.as_bool())
        .unwrap_or(false)
}

fn create_checker(factory: &ISpellCheckerFactory, language_tag: &str) -> Option<ISpellChecker> {
    // SAFETY: nhu tren; tra `E_INVALIDARG` neu tag khong co pack => `.ok()` = None.
    unsafe { factory.CreateSpellChecker(&HSTRING::from(language_tag)) }.ok()
}

fn handle_job(checker: &ISpellChecker, job: Job) {
    match job {
        Job::Check { text, reply } => {
            let issues = collect_issues(checker, &text);
            reply.send(issues).ok();
        }
        Job::Suggest { word, limit, reply } => {
            let suggestions = collect_suggestions(checker, &word, limit);
            reply.send(suggestions).ok();
        }
    }
}

/// `true` neu `CorrectiveAction` dang xu ly duoc ma khong can goi `Suggest`.
///
/// `GET_SUGGESTIONS` **duoc tinh la loi** (Fassist-111): do that tren Windows 11
/// cho thay moi loi chinh ta thuong deu tra `GET_SUGGESTIONS`; bo qua no se lam
/// `check()` khong bat duoc loi nao. Phan goi y van di qua `Suggest` (co ngan sach).
fn is_actionable(action: CORRECTIVE_ACTION) -> bool {
    action == CORRECTIVE_ACTION_REPLACE
        || action == CORRECTIVE_ACTION_DELETE
        || action == CORRECTIVE_ACTION_GET_SUGGESTIONS
}

fn collect_issues(checker: &ISpellChecker, text: &[u16]) -> Vec<SpellIssue> {
    if text.is_empty() {
        return Vec::new();
    }
    // SAFETY: `checker` chi dung tren worker thread; `HSTRING` song qua loi goi.
    let Ok(enumerator) = (unsafe { checker.Check(&HSTRING::from_wide(text)) }) else {
        return Vec::new();
    };

    let mut issues = Vec::new();
    loop {
        let mut item: Option<ISpellingError> = None;
        // SAFETY: `item` la out-param hop le; `Next` tra `S_FALSE` (khong loi) khi het.
        let hr = unsafe { enumerator.Next(&mut item) };
        if hr.is_err() {
            break;
        }
        let Some(error) = item else {
            break;
        };

        let start = unsafe { error.StartIndex() };
        let len = unsafe { error.Length() };
        let action = unsafe { error.CorrectiveAction() };
        let (Ok(start), Ok(len), Ok(action)) = (start, len, action) else {
            continue;
        };
        if !is_actionable(action) {
            continue;
        }

        // `GET_SUGGESTIONS` cũng được coi là lỗi (Fassist-111): đo thật trên
        // Windows 11 cho thấy **hầu hết** lỗi chính tả thường trả
        // `GET_SUGGESTIONS` ⇒ nếu bỏ qua thì `check()` không phát hiện được
        // lỗi nào. Offset/len có sẵn (miễn phí); phần **gợi ý** vẫn được
        // lấy qua `Suggest` với ngân sách 3–7 ms/từ (`SUGGEST_BUDGET`).
        let replacement = if action == CORRECTIVE_ACTION_DELETE {
            String::new()
        } else if action == CORRECTIVE_ACTION_GET_SUGGESTIONS {
            // Rỗng = "cần gợi ý", không phải "xoá".
            String::new()
        } else {
            replacement_of(&error)
        };
        issues.push(SpellIssue {
            start_utf16: start as usize,
            len_utf16: len as usize,
            replacement,
        });
    }
    issues
}

fn replacement_of(error: &ISpellingError) -> String {
    // SAFETY: `Replacement` tra LPWSTR do COM cap phat bang CoTaskMem.
    let Ok(ptr) = (unsafe { error.Replacement() }) else {
        return String::new();
    };
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: con tro NUL-terminated do he thong cap; copy ra `String` truoc khi free.
    let text = unsafe { ptr.to_string() }.unwrap_or_default();
    // SAFETY: dung 1 lan cho dung con tro `Replacement` tra ve (P4-10 W4).
    unsafe { CoTaskMemFree(Some(ptr.as_ptr().cast())) };
    text
}

fn collect_suggestions(checker: &ISpellChecker, word: &[u16], limit: usize) -> Vec<String> {
    if word.is_empty() || limit == 0 {
        return Vec::new();
    }
    // SAFETY: `checker` chi dung tren worker thread; `HSTRING` song qua loi goi.
    let Ok(enumerator) = (unsafe { checker.Suggest(&HSTRING::from_wide(word)) }) else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(limit);
    let mut buffer = [PWSTR::null(); MAX_SUGGEST_RESULTS];
    loop {
        // SAFETY: `buffer` la mang out hop le; `S_FALSE` (khong loi) = het.
        let hr = unsafe { enumerator.Next(&mut buffer, None) };
        if hr.is_err() {
            break;
        }
        let mut fetched = 0usize;
        for ptr in buffer.iter() {
            if ptr.is_null() {
                break;
            }
            fetched += 1;
            if out.len() < limit {
                // SAFETY: con tro NUL-terminated do he thong cap.
                if let Ok(text) = unsafe { ptr.to_string() } {
                    out.push(text);
                }
            }
            // SAFETY: moi PWSTR do `Next` tra ve phai free rieng (P4-10 W4).
            unsafe { CoTaskMemFree(Some(ptr.as_ptr().cast())) };
        }
        if fetched == 0 || out.len() >= limit {
            break;
        }
        buffer = [PWSTR::null(); MAX_SUGGEST_RESULTS];
    }
    out
}

/// `CoInitializeEx`/`CoUninitialize` theo RAII tren worker thread.
struct ComApartment {
    /// `true` neu chinh loi goi nay da khoi tao COM (S_OK hoac S_FALSE).
    owned: bool,
}

impl ComApartment {
    fn initialize() -> Self {
        // SAFETY: goi 1 lan tren worker thread moi; MTA la apartment phu hop cho
        // Spell Checking API (khong can marshalling vi moi thu o 1 thread).
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Self { owned: hr.is_ok() }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: can bang dung 1 `CoInitializeEx` thanh cong o tren, cung thread.
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_khong_hop_le_thi_khong_co_tu_dien() {
        // "zz-ZZ" khong phai language tag co that => IsSupported = FALSE tren moi may.
        assert!(!WindowsSpellChecker::is_supported("zz-ZZ"));
        let err = WindowsSpellChecker::new("zz-ZZ").expect_err("phai la NoDictionary");
        assert!(matches!(err, DictError::NoDictionary));
        // Fallback cua crate: `None` => caller dung assist::dict::NoDictionary
        // (`contains` = true) => KHONG bao loi spelling nao.
        assert!(crate::os_provider("zz-ZZ").is_none());
    }

    #[test]
    fn tag_rong_thi_khong_co_tu_dien() {
        assert!(!WindowsSpellChecker::is_supported(""));
        assert!(matches!(
            WindowsSpellChecker::new(""),
            Err(DictError::NoDictionary)
        ));
        assert!(crate::os_provider("  ").is_none());
    }

    #[test]
    fn vi_vn_khong_ho_tro_thi_fallback_an_toan() {
        // Dieu kien that cua may: mac dinh Windows KHONG co pack vi-VN (P4-10 §2.5).
        // Khong assert cung - may co pack thi provider phai tao duoc.
        if WindowsSpellChecker::is_supported("vi-VN") {
            assert!(WindowsSpellChecker::new("vi-VN").is_ok());
            return;
        }
        assert!(matches!(
            WindowsSpellChecker::new("vi-VN"),
            Err(DictError::NoDictionary)
        ));
        assert!(
            crate::os_provider("vi-VN").is_none(),
            "khong ho tro => None => NoDictionary => khong loi spelling"
        );
    }

    #[test]
    fn chi_sua_loi_replace_va_delete() {
        use windows::Win32::Globalization::{
            CORRECTIVE_ACTION_GET_SUGGESTIONS, CORRECTIVE_ACTION_NONE,
        };
        assert!(is_actionable(CORRECTIVE_ACTION_REPLACE));
        assert!(is_actionable(CORRECTIVE_ACTION_DELETE));
        // `GET_SUGGESTIONS` doi hoi `Suggest` (3-7 ms/tu) => bo qua.
        assert!(!is_actionable(CORRECTIVE_ACTION_GET_SUGGESTIONS));
        assert!(!is_actionable(CORRECTIVE_ACTION_NONE));
    }

    #[test]
    fn suggest_chua_check_thi_khong_goi() {
        let budget = SuggestBudget::new();
        assert!(!budget.allow(), "cua so ngan sach chua mo");
    }

    #[test]
    fn suggest_bi_cap_so_lan_goi() {
        let budget = SuggestBudget::new();
        budget.open_window();
        for i in 0..SUGGEST_CALLS_PER_CHECK {
            assert!(budget.allow(), "lan {i} phai duoc phep");
        }
        assert!(!budget.allow(), "vuot cap => fail-open (rong)");

        // `check()` moi mo lai cua so + reset dem.
        budget.open_window();
        assert!(budget.allow(), "cua so moi phai cho phep lai");
    }

    #[test]
    fn suggest_het_ngan_sach_thoi_gian() {
        let budget = SuggestBudget::new();
        budget.open_window();
        std::thread::sleep(SUGGEST_BUDGET + Duration::from_millis(20));
        assert!(!budget.allow(), "qua 50 ms => fail-open (rong)");
    }

    /// Test tich hop **co dieu kien**: chi chay khi may co checker `en-US`.
    ///
    /// May khong co pack => bo qua (test (a) khong phu thuoc he thong). Muc dich:
    /// kiem tra duong COM that (worker thread, `Check`, `Suggest`, offset UTF-16)
    /// va cap `suggest` o muc provider.
    #[test]
    fn en_us_neu_may_co_thi_provider_hoat_dong() {
        fn utt16(text: &str) -> Vec<u16> {
            text.encode_utf16().collect()
        }

        if !WindowsSpellChecker::is_supported("en-US") {
            return;
        }
        let Ok(checker) = WindowsSpellChecker::new("en-US") else {
            return; // pack co nhung COM khong tao duoc => fail-open, khong fail test
        };

        // Tu dung => khong bao loi.
        assert!(
            checker.check(&utt16("hello")).is_empty(),
            "tu dung khong duoc bao loi"
        );

        // "the the" => Windows bao DELETE (do thuc nghiem 2026-10-21). Khong
        // assert cung: ban Windows khac co the doi hanh vi => chi kiem tra khi
        // OS thuc su bao (va luon kiem tra offset nam trong bien).
        let duplicated = utt16("the the");
        let issues = checker.check(&duplicated);
        for issue in &issues {
            assert!(issue.len_utf16 > 0, "loi phai co do dai > 0");
            assert!(
                issue.start_utf16 + issue.len_utf16 <= duplicated.len(),
                "offset phai nam trong text (UTF-16 code unit)"
            );
        }
        if let Some(dup) = issues.iter().find(|issue| issue.start_utf16 == 4) {
            assert_eq!(dup.len_utf16, 3);
            assert!(dup.replacement.is_empty(), "DELETE => replacement rong");
        }

        // Offset la **UTF-16 code unit**: ky tu astral chiem 2 code unit nen
        // "the" thu 2 bat dau o 7 (char index chi la 6).
        let astral = utt16("\u{1D518} the the");
        let dup_astral = checker.check(&astral);
        if let Some(dup) = dup_astral.iter().find(|issue| {
            issue.len_utf16 == 3 && issue.replacement.is_empty() && issue.start_utf16 > 3
        }) {
            assert_eq!(
                dup.start_utf16, 7,
                "offset la UTF-16 code unit, khong phai char (char index = 6)"
            );
        }

        // `check` vua mo cua so 50 ms: toi da SUGGEST_CALLS_PER_CHECK lan suggest.
        let word = utt16("helo");
        let mut any_suggestion = false;
        for i in 0..SUGGEST_CALLS_PER_CHECK {
            let suggestions = checker.suggest(&word, 3);
            assert!(suggestions.len() <= 3, "lan {i}: phai cat theo limit");
            any_suggestion |= !suggestions.is_empty();
        }
        if any_suggestion {
            assert!(
                checker.suggest(&word, 3).is_empty(),
                "vuot cap => fail-open (rong)"
            );
        }
    }
}
