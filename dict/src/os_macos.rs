// SPDX-License-Identifier: GPL-3.0-or-later
//! Provider spell-check **macOS** (`NSSpellChecker`, AppKit) — FFI **thuần**
//! qua runtime Objective-C (`objc_msgSend`), **không thêm crate nào** (crate
//! `dict` giữ 0-dep — ngoại lệ duy nhất vẫn là crate `windows` của
//! [`crate::os_windows`]).
//!
//! Đây là **adapter ở biên** (P4-0 **D32**), bám đúng pattern của
//! [`crate::os_windows`] / [`crate::os_linux`]: **1 worker thread** + timeout
//! **50 ms** + `try_send` (caller không bao giờ block) + **fail-open** + ngân
//! sách cho `suggest` + cache capability theo tag (P4-10 AC-2).
//!
//! ## Đơn vị gọi — **word-level** (như Linux, khác Windows)
//!
//! | Trait | `NSSpellChecker` |
//! |---|---|
//! | `contains(word)` | `checkSpelling(of: word, startingAt: 0, …).length == 0` — trả **`NSRange`** của từ sai đầu tiên; `length == 0` = không có từ sai ⇒ từ đúng (P4-10 §3.1, bảng tổng hợp §6) |
//! | `suggest(word, limit)` | `guesses(forWordRange: in: language: inSpellDocumentWithTag:) -> [String]?` + cắt theo `limit` |
//!
//! Lưu ý API: `checkSpelling` trả `NSRange` (location/length), **không** phải
//! `NSTextCheckingResult` (đó là API của `NSTextChecker`, không phải
//! `NSSpellChecker`). `NSSpellChecker` là API Objective-C/Swift nên từ Rust gọi
//! qua runtime FFI: `objc_getClass` + `sel_registerName` + `objc_msgSend`
//! (runtime có sẵn trong mọi app macOS, link `-lobjc` — không shim Swift, không
//! build shim C). P4-10 §3.3 khuyến nghị variant "Swift sở hữu NSSpellChecker"
//! cho cầu `assist` (callback C-ABI); ở tầng `dict` này ta chọn `objc_msgSend`
//! vì **ít phụ thuộc nhất** (chỉ đạo của brief P4-10) — không bắt buộc thêm
//! file shim Swift vào build, crate giữ 0-dep. Khi tích hợp IMK thật
//! (adapters/macos-imk) nếu cần đúng kiến trúc Swift-owner thì có thể thay
//! tầng này bằng callback C-ABI mà không đổi trait `contains`/`suggest`.
//!
//! ## Vì sao **không** dùng `checkGrammar` ở v1 (Fassist-107 / F68 / P4-10-F6)
//!
//! `NSSpellChecker.checkGrammar(of:startingAt:language:wrap:…)` **có** tồn tại
//! (khác P4-4 ghi "chưa rõ"), nhưng bật nó sẽ làm lệch hành vi giữa 3 nền
//! tảng: Windows Spell Checking API **không** có API ngữ pháp, enchant-2
//! **không** có API ngữ pháp ⇒ chỉ macOS sẽ báo lỗi ngữ pháp ⇒ UX 3 OS khác
//! nhau. Fassist-107/F68 chốt: ngữ pháp **luôn** do engine của ta (core rules),
//! OS provider chỉ làm **checking spelling**. Tương tự `hasCorrection` **không
//! có** trong API công khai (P4-10-F5) — v1 không cần nó (path
//! contains/suggest không dùng); nếu sau này cần "một cách sửa duy nhất" hãy
//! dùng `correction(forWordRange:in:language:inSpellDocumentWithTag:)`.
//!
//! ## UI — candidate list do IMK server vẽ (D30)
//!
//! Trên macOS **app client không vẽ underline/gạch chân chữ sai** như Windows
//! (TSF `ITfContextView`): candidate list do **input method server (IMK) vẽ**
//! (`IMKInputController` là lớp cơ sở; `IMKCandidates` "presents candidates to
//! users" và là optional — P4-10 §3.4). Hệ quả: gợi ý spelling trên macOS chỉ
//! hiện qua **toast/panel** của IMK (D30). Đây là khác biệt **được chấp nhận**
//! ở v1 và phải ghi trong `ui-spec` — provider này chỉ trả dữ liệu
//! (`contains`/`suggest`), **không** tự vẽ bất kỳ UI nào.
//!
//! ## Chi phí — **chưa đo được**, phải đo trước khi bật trong gói
//!
//! Vẫn **không có máy macOS trong phiên này** (P4-10 §9 item 1/2): độ trễ
//! cold/warm của `checkSpelling`/`guesses` và việc block-main-thread khi spell
//! server chậm đều **chưa đo được** (AC-3/AC-4). Pattern worker + timeout 50 ms
//! của ta chặn đường chậm ở phía Rust độc lập với số đo đó, **nhưng** không
//! cung cấp số để ghi vào privacy/perf docs. Feature `os-macos` vì vậy **mặc
//! định TẮT** (giống `os-enchant` ở Linux): chỉ bật trong gói macOS phát hành,
//! sau khi đo được chi phí thật trên máy 14/15/26.
//!
//! ## Thread-safety — bắt buộc 1 worker thread (P4-10 M2)
//!
//! `NSSpellChecker` **không** an toàn gọi từ nhiều thread, và AppKit nói
//! chung có **thread-affinity** (M2). Pattern ở đây gọi **toàn bộ** Objective-C
//! runtime + `NSSpellChecker` trên **một worker thread duy nhất** của provider
//! (khác đề xuất M2 "main thread" — chọn worker như Windows/Linux để giữ
//! caller không bao giờ block và có deadline đồng nhất 50 ms; ghi nhận rủi ro
//! M2 vẫn còn, có thể chuyển sang dispatch-main trong adapter IMK v2 nếu đo
//! thấy spell server chậm). `NSSpellChecker.shared` / `NSArray` / `NSString`
//! **không bao giờ** qua channel — chỉ `String`/`Vec<String>` đi lại.
//!
//! `spellDocumentWithTag` dùng `uniqueSpellDocumentTag()` riêng cho TextVN
//! (M3) và `closeSpellDocumentWithTag` được gọi (có guard
//! `respondsToSelector:` — selector không có thì im lặng, fail-open) khi
//! worker kết thúc để đỡ giữ "ignored words" vĩnh viễn.
//!
//! ## Fallback khi không có từ điển `vi` (M1) — Fassist-107
//!
//! `availableLanguages` **không** chứa `vi` (máy chưa bật Spelling = Vietnamese
//! trong System Settings → Keyboard → Text) ⇒ `is_supported` = `false`,
//! `new` trả [`crate::DictError::NoDictionary`] ⇒ caller dùng
//! `assist::dict::NoDictionary` (mọi từ coi là đúng) ⇒ **không** báo lỗi
//! spelling nào (Fassist-107). Tuyệt đối **không** mượn checker ngôn ngữ khác
//! (đo Windows cho `en-US` → sai 3/8 từ tiếng Việt ⇒ FPR không chấp nhận —
//! P4-10 §2.5).
//!
//! ## Privacy (S2)
//!
//! Không log `text`/`word` ở bất kỳ nhánh nào — chỉ log/mã lỗi. Câu hỏi,
//! "text có rời tiến trình TextVN không" vẫn **chưa xác minh** (P4-10 bảng
//! §6: `NSSpellChecker` in-process nhưng có thể giao tiếp spell server của
//! OS) — phải xác minh trước khi bật trong gói, vì Windows đã bắt buộc ghi
//! notice cho out-of-proc.
//!
//! ## Module compile trên **mọi** target; FFI chỉ nằm trong `backend`
//!
//! Module được compile ở **mọi** OS (giống "shell" fail-open của
//! [`crate::os_linux`]) để test được trên máy **không** cần macOS (budget,
//! khớp ngôn ngữ, factory `None`) — đúng như yêu cầu brief. Toàn bộ FFI
//! (`objc_msgSend`…) nằm trong `backend` dưới
//! `#[cfg(all(target_os = "macos", feature = "os-macos"))]` — feature
//! **mặc định TẮT** ⇒ `cargo check`/test/CI trên bất kỳ OS nào **không** link
//! libobjc, không cần SDK AppKit; bật feature trên máy build macOS mới dùng.
//! Bài học Fassist-104: **không** tự khai báo symbol trùng symbol runtime của
//! `std` — mọi extern ở đây là `objc_*`/`sel_registerName`, `std` không xuất
//! `symbol` nào như thế ⇒ an toàn khai tay, không cần crate `objc`.

use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(all(target_os = "macos", feature = "os-macos"))]
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::DictError;

/// Ngân sách thời gian cho `suggest` (P4-0 D36: `<= 50 ms`, hết giờ = trả rỗng).
pub const SUGGEST_BUDGET: Duration = Duration::from_millis(50);

/// Cap số lần `suggest` trong một cửa sổ ngân sách (P4-10 §5.3, giống
/// Windows/Linux là 3).
///
/// Chi phí ước lượng macOS chưa đo được (không có máy); số Windows 3 × 6,9 ms
/// ≈ 21 ms cho thấy vẫn nằm trong cửa sổ 50 ms nên lấy số 3 cho đồng nhất.
pub const SUGGEST_CALLS_PER_CHECK: usize = 3;

/// Ngân sách cho `suggest`: cửa sổ thời gian + đếm số lần gọi.
///
/// Đơn vị gọi ở đây là **mỗi từ** (khác Windows: `check` mỗi câu) nên dùng
/// [`SuggestBudget::anchor`] — cửa sổ chỉ mở lại khi cửa sổ cũ hết hạn (giống
/// [`crate::os_linux`]). Tách riêng để test được trên máy **không cần macOS** —
/// logic ngân sách thuần Rust, không liên quan FFI.
///
/// `allow(dead_code)` có mục đích (không phải blanket-ban): khi feature
/// `os-macos` **tắt** (mặc định) struct chỉ được dùng trong test — như cách
/// `os_linux` cắm `cfg_attr` cho `SuggestBudget` của enchant.
#[derive(Debug)]
struct SuggestBudget {
    used: AtomicUsize,
    deadline: Mutex<Instant>,
}

#[cfg_attr(not(all(target_os = "macos", feature = "os-macos")), allow(dead_code))]
impl SuggestBudget {
    /// Chưa mở cửa sổ: mọi `suggest` bị từ chối cho tới lần `contains` đầu.
    fn new() -> Self {
        Self {
            used: AtomicUsize::new(SUGGEST_CALLS_PER_CHECK),
            deadline: Mutex::new(Instant::now()),
        }
    }

    /// Mở lại cửa sổ ngân sách khi đã hết hạn; giữa cửa sổ còn hạn thì **không**
    /// reset bộ đếm (đơn vị gọi mỗi từ — xem doc struct).
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

    /// `true` nếu còn đủ ngân sách cho một lần `suggest` nữa.
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

/// Provider spell-check macOS dùng chung 1 worker thread AppKit.
///
/// Tạo bằng [`MacSpellChecker::new`]. Provider **không** tự báo lỗi: khi máy
/// không có từ điển cho `language_tag` (điều kiện phổ biến — `vi` phải bật
/// Spelling trong System Settings) thì `new` trả
/// [`DictError::NoDictionary`] và caller phải fallback
/// `assist::dict::NoDictionary` (mọi từ đúng ⇒ không báo lỗi spelling).
///
/// Mọi phương thức đều **fail-open**: lỗi runtime Obj-C, timeout 50 ms hoặc
/// worker chết ⇒ `contains` = `true` (từ đúng), `suggest` = rỗng; không panic,
/// không block quá 50 ms. Khi feature `os-macos` **TẮT** (mặc định) thì `new`
/// luôn `NoDictionary` ⇒ cũng là đường fail-open trọn vẹn.
pub struct MacSpellChecker {
    /// Kênh gửi job sang worker thread (chỉ có khi FFI được compile).
    #[cfg(all(target_os = "macos", feature = "os-macos"))]
    tx: SyncSender<backend::Job>,
    /// Ngân sách `suggest` — luôn có để lớp ngân sách test được độc lập.
    budget: SuggestBudget,
}

#[cfg(all(target_os = "macos", feature = "os-macos"))]
impl MacSpellChecker {
    /// Tạo checker cho `language_tag` (BCP-47, ví dụ `vi-VN`).
    ///
    /// Probe (`availableLanguages`, `uniqueSpellDocumentTag`) chạy trên worker
    /// thread với timeout 50 ms; kết quả capability cache theo tag cho toàn
    /// process — mỗi tag hỏi tối đa 1 lần/phiên (P4-10 AC-2).
    ///
    /// # Errors
    ///
    /// [`DictError::NoDictionary`] khi: `availableLanguages` không có ngôn ngữ
    /// khớp `language_tag` (máy không bật Spelling = Vietnamese — M1), thread
    /// không spawn được, probe quá hạn 50 ms. Caller dùng
    /// `assist::dict::NoDictionary` ⇒ **không** báo lỗi spelling nào.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(target_os = "macos", feature = "os-macos"))] {
    /// use textvn_dict::os_macos::MacSpellChecker;
    ///
    /// // Máy không có từ điển vi (mặc định phổ biến) => Err(NoDictionary).
    /// if let Ok(checker) = MacSpellChecker::new("vi-VN") {
    ///     let _ = checker.contains("hoà");
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

    /// Cờ capability: từ điển hệ thống có ngôn ngữ khớp `language_tag` không.
    ///
    /// Dựa trên `NSSpellChecker.availableLanguages` (P4-10 §3.1) — **chưa xác
    /// minh trên máy thật** (không có máy macOS trong phiên). Nếu danh sách
    /// không có ngôn ngữ khớp tag thì trả `false` ⇒ caller fallback
    /// `NoDictionary` (không báo lỗi spelling). Cờ được cache theo tag và mọi
    /// lỗi runtime trả `false` (fail-safe: im lặng hơn là báo sai).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(target_os = "macos", feature = "os-macos"))] {
    /// use textvn_dict::os_macos::MacSpellChecker;
    ///
    /// if !MacSpellChecker::is_supported("vi-VN") {
    ///     // => dùng assist::dict::NoDictionary, không báo lỗi spelling.
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

    /// Kiểm tra 1 từ (UTF-8) có trong từ điển hệ thống không.
    ///
    /// Gọi `contains` **kèm ngân sách**: đơn vị gọi là **mỗi từ** (khác
    /// Windows — `check` mỗi câu) nên cửa sổ ngân sách chỉ mở lại khi cửa sổ
    /// trước hết hạn (xem [`SUGGEST_BUDGET`]).
    ///
    /// # Fail-open
    ///
    /// Từ rỗng / lỗi runtime Obj-C / timeout 50 ms / worker bận ⇒ `true` (từ
    /// đúng, KHÔNG báo lỗi spelling). `checkSpelling` trả `length == 0` ⇒
    /// `true`; trả range có `length` > 0 ⇒ `false` ⇒ assist báo lỗi và có thể
    /// gọi `suggest` (có ngân sách).
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
        // `try_send`: nếu worker đang bận/treo thì bỏ ngay, KHÔNG block caller.
        if self.tx.try_send(job).is_err() {
            return true;
        }
        result.recv_timeout(backend::WORK_TIMEOUT).unwrap_or(true)
    }

    /// Gợi ý từ thay thế cho `word` (UTF-8), tối đa `limit` kết quả.
    ///
    /// **Có ngân sách** (bắt buộc — chi phí `guesses` chưa đo được trên
    /// macOS, số Windows 3–7 ms/từ cho thấy phải chặn cứng 50 ms):
    ///
    /// - chỉ được gọi tối đa [`SUGGEST_CALLS_PER_CHECK`] lần trong cửa sổ
    ///   [`SUGGEST_BUDGET`] hiện hành;
    /// - chưa mở cửa sổ (chưa gọi `contains` lần nào) / hết cửa / vượt cap
    ///   ⇒ `Vec` rỗng.
    ///
    /// `limit` bị chặn trên [`backend::MAX_SUGGEST_RESULTS`]; timeout / lỗi
    /// runtime ⇒ rỗng (fail-open, không phải lỗi).
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

/// Impl khi feature `os-macos` **TẮT** (mặc định trên mọi OS — kể cả macOS
/// release builds chưa bật): không FFI nào được compile/link, mọi từ coi là
/// đúng, `suggest` rỗng — hành vi y hệt `assist::dict::NoDictionary`
/// (fail-open, không hồi quy = AC-1, Fassist-107). Lớp ngân sách vẫn còn và
/// được test độc lập (không cần libobjc).
#[cfg(not(all(target_os = "macos", feature = "os-macos")))]
impl MacSpellChecker {
    /// Luôn [`DictError::NoDictionary`]: feature tắt ⇒ không link libobjc
    /// ⇒ không thể có từ điển OS. Caller phải fallback `NoDictionary`.
    pub fn new(_language_tag: &str) -> Result<Self, DictError> {
        Err(DictError::NoDictionary)
    }

    /// Luôn `false`: feature tắt ⇒ không có capability nào.
    #[must_use]
    pub fn is_supported(_language_tag: &str) -> bool {
        false
    }

    /// Luôn `true` (fail-open): mọi từ đúng ⇒ không báo lỗi spelling.
    #[must_use]
    pub fn contains(&self, _word: &str) -> bool {
        self.budget.anchor();
        true
    }

    /// Luôn rỗng (fail-open); vẫn rút ngân sách để hành vi đồng nhất.
    #[must_use]
    pub fn suggest(&self, _word: &str, _limit: usize) -> Vec<String> {
        let _ = self.budget.allow();
        Vec::new()
    }
}

/// Quy tắc khớp ngôn ngữ giữa `availableLanguages` (danh sách tag của
/// `NSSpellChecker`, **thuần Rust — dùng cho test inject được**, brief P4-10: cho
/// phép test `is_supported` với danh sách giả) và BCP-47 tag của caller.
///
/// Trả về **tag được chọn** để truyền vào `checkSpelling`/`guesses` (phải là
/// một entry của `availableLanguages`, không dùng tag gốc của caller):
///
/// 1. khớp **đúng** (không phân biệt hoa thường ASCII) — tag `vi-VN` và
///    entry `vi-VN` ⇒ chọn `vi-VN`;
/// 2. nếu không, khớp **primary subtag** (BCP-47: mọi thứ trước `-`/`_` đầu
///    tiên, lowercase) — máy thường chỉ có entry `vi`, tag caller `vi-VN` ⇒
///    chọn `vi` (đúng sketch P4-10 §3.3: `$0.lowercased().hasPrefix("vi")`).
///
/// Không khớp ⇒ `None` ⇒ `NoDictionary` (Fassist-107: tuyệt đối **không** mượn
/// checker ngôn ngữ khác).
///
/// `allow(dead_code)` có mục đích (không phải blanket-ban): khi feature
/// `os-macos` **tắt** (mặc định) hàm chỉ được dùng trong test (inject danh
/// sách ngôn ngữ giả — yêu cầu brief); khi feature bật, backend `probe` dùng
/// nó thật.
#[cfg_attr(not(all(target_os = "macos", feature = "os-macos")), allow(dead_code))]
#[must_use]
fn language_match(languages: &[String], language_tag: &str) -> Option<String> {
    let tag = language_tag.trim();
    if tag.is_empty() {
        return None;
    }
    let primary = tag.split(['-', '_']).next().unwrap_or("").to_lowercase();
    if primary.is_empty() {
        return None;
    }
    for lang in languages {
        if lang.eq_ignore_ascii_case(tag) {
            return Some(lang.clone());
        }
    }
    for lang in languages {
        if lang.eq_ignore_ascii_case(&primary) {
            return Some(lang.clone());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// backend: FFI thuần vào runtime Objective-C — **toàn bộ** unsafe nằm ở đây và
// CHỈ compile khi `target_os = "macos"` **và** feature `os-macos` bật (mặc định
// TẮT ⇒ `cargo check`/test/CI trên mọi OS không link libobjc, không cần SDK
// AppKit, không phá build Windows/Linux — yêu cầu bắt buộc brief P4-10).
//
// ```
// [caller thread]                    [worker thread — DUY NHẤT]
//   contains() -> Job try_send  →  recv(); NSSpellChecker + NSString +
//   recv_timeout(50 ms)              availableLanguages/... ; Pool RAII
//   fail-open nếu timeout/lỗi    ←  send(result) — chỉ String/Vec<String>
// ```
// ---------------------------------------------------------------------------
#[cfg(all(target_os = "macos", feature = "os-macos"))]
mod backend {
    use std::ffi::{c_char, CStr};
    use std::os::raw::c_void;
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Timeout cho mỗi lời gọi AppKit (`ms`) — P4-0 D36: `<= 50 ms`, hết giờ =
    /// fail-open. Spell check có thể giao tiếp spell server của OS (chưa xác
    /// minh in/out process) nên deadline caller-side là cần thiết như Windows.
    pub(super) const WORK_TIMEOUT: Duration = Duration::from_millis(50);

    /// Timeout cho bước probe (`sharedSpellChecker` + `availableLanguages`).
    const PROBE_TIMEOUT: Duration = Duration::from_millis(50);

    /// Cap số gợi ý trả về cho **một** lần `suggest` (tránh cấp phát lớn).
    pub(super) const MAX_SUGGEST_RESULTS: usize = 8;

    /// `NSUInteger` trên aarch64/x86_64-apple-darwin (LP64) = `u64`.
    type NSUInteger = u64;
    /// `NSInteger` (LP64) = `i64`.
    type NSInteger = i64;
    /// `BOOL` trên macOS hiện hành (aarch64/x86_64) = `signed char` = `i8`.
    type ObjcBool = i8;

    /// `NSRange` của AppKit — **UTF-16 code unit** (P4-10 §3.2), không phải
    /// `char` Unicode của `assist` (D27). 2 × `NSUInteger` ⇒ trả về bằng
    /// thanh ghi trên cả arm64 (x0/x1) lẫn x86_64 (RAX/RDX) nên khai báo như
    /// kiểu trả về của `objc_msgSend` là đúng C ABI (cách `objc`/`objc2`
    /// crate làm).
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub(super) struct NsRange {
        pub(super) location: NSUInteger,
        pub(super) length: NSUInteger,
    }

    /// Đối tượng Objective-C (opaque) — mọi pointer method run-time.
    #[repr(C)]
    struct ObjcObject {
        _private: [u8; 0],
    }
    type Id = *mut ObjcObject;

    /// `SEL` (pointer opaque của runtime).
    #[repr(C)]
    struct ObjcSelector {
        _private: [u8; 0],
    }
    type Sel = *const ObjcSelector;

    // Tất cả extern trên đây là symbol **riêng** của libobjc (`objc_*`,
    // `sel_registerName`, `objc_msgSend`) — `std` không xuất symbol nào như
    // vậy nên khai tay an toàn (Fassist-104: chỉ cần tránh chạm tên std).
    #[link(name = "objc")]
    extern "C" {
        /// `id objc_getClass(const char *name)` — `None` nếu class không có.
        #[link_name = "objc_getClass"]
        fn raw_get_class(name: *const c_char) -> Id;

        /// `SEL sel_registerName(const char *str)`.
        #[link_name = "sel_registerName"]
        fn raw_sel_register_name(name: *const c_char) -> Sel;

        /// Pool RAII: `void *objc_autoreleasePoolPush(void)` — cần vì NSString
        /// tạo bằng `+stringWithCharacters:length:` là autoreleased (worker
        /// không có main run loop ⇒ phải tự push/pop, không dồn pool cho
        /// run loop nào).
        #[link_name = "objc_autoreleasePoolPush"]
        fn raw_pool_push() -> *mut c_void;

        #[link_name = "objc_autoreleasePoolPop"]
        fn raw_pool_pop(token: *mut c_void);
    }

    // `objc_msgSend` được khai báo **nhiều lần với chữ ký C cụ thể** (cách
    // `objc` crate làm): mỗi selector một signature — không dùng varargs vì
    // Rust không cho truyền struct (NSRange) qua varargs chắc chắn đúng ABI.
    #[link(name = "objc")]
    extern "C" {
        // `+ (instancetype)stringWithCharacters:(const unichar *)chars
        //        length:(NSUInteger)length`
        #[link_name = "objc_msgSend"]
        fn msg_send_string_with_chars(cls: Id, sel: Sel, chars: *const u16, len: NSUInteger) -> Id;

        // `+ (NSSpellChecker *)sharedSpellChecker`
        #[link_name = "objc_msgSend"]
        fn msg_send_shared_checker(cls: Id, sel: Sel) -> Id;

        // `+ (NSUInteger)uniqueSpellDocumentTag`
        #[link_name = "objc_msgSend"]
        fn msg_send_unique_tag(cls: Id, sel: Sel) -> NSUInteger;

        // `+ (void)closeSpellDocumentWithTag:(NSUInteger)tag` — gọi CÓ guard
        // `respondsToSelector:` bên dưới (selector không có ⇒ bỏ qua, im lặng).
        #[link_name = "objc_msgSend"]
        fn msg_send_close_spell_document(cls: Id, sel: Sel, tag: NSUInteger);

        // `- (BOOL)respondsToSelector:(SEL)aSelector` (metaclass-side guard).
        #[link_name = "objc_msgSend"]
        fn msg_send_responds_to(cls: Id, sel: Sel, selector: Sel) -> ObjcBool;

        // `- (NSArray<NSString *> *)availableLanguages`
        #[link_name = "objc_msgSend"]
        fn msg_send_available_languages(receiver: Id, sel: Sel) -> Id;

        // `- (NSUInteger)count` (NSArray)
        #[link_name = "objc_msgSend"]
        fn msg_send_count(receiver: Id, sel: Sel) -> NSUInteger;

        // `- (id)objectAtIndex:` (NSArray)
        #[link_name = "objc_msgSend"]
        fn msg_send_object_at(receiver: Id, sel: Sel, index: NSUInteger) -> Id;

        // `- (const char *)UTF8String` (NSString)
        #[link_name = "objc_msgSend"]
        fn msg_send_utf8_string(receiver: Id, sel: Sel) -> *const c_char;

        // `- (NSRange)checkSpelling:(NSString *)ofString startingAt:(NSInteger)
        //                  startingLocation language:(NSString *)language
        //                  wrap:(BOOL)wrapFlag
        //                  inSpellDocumentWithTag:(NSInteger)tag
        //                  wordCount:(NSInteger *)wordCount`
        //
        // (Nguyên bản Obj-C: `checkSpelling:startingAt:language:wrap:
        //  inSpellDocumentWithTag:wordCount:` — P4-10 §3.1 đã verify.)
        #[link_name = "objc_msgSend"]
        fn msg_send_check_spelling(
            receiver: Id,
            sel: Sel,
            text: Id,
            starting_at: NSUInteger,
            language: Id,
            wrap: ObjcBool,
            tag: NSInteger,
            word_count: Id, // `nil` — ta không đọc word count
        ) -> NsRange;

        // `- (NSArray<NSString *> *)guessesForWordRange:(NSRange)range
        //                  inString:(NSString *)string
        //                  language:(NSString *)language
        //                  inSpellDocumentWithTag:(NSInteger)tag`
        // (Nguyên bản Obj-C của Swift `guesses(forWordRange:...)`.)
        #[link_name = "objc_msgSend"]
        fn msg_send_guesses(
            receiver: Id,
            sel: Sel,
            range: NsRange,
            text: Id,
            language: Id,
            tag: NSInteger,
        ) -> Id;
    }

    /// Guard RAII autorelease pool: push lúc tạo, pop lúc drop (FIFO AppKit).
    struct Pool {
        token: *mut c_void,
    }

    impl Pool {
        /// Push một pool mới (nested pool hợp lệ; pop đúng chiều ngược push).
        fn push() -> Self {
            // SAFETY: hàm runtime thuần; token `NULL` cũng được API chấp nhận
            // (pop sẽ no-op qua guard `is_null` ở `drop`).
            Self {
                token: unsafe { raw_pool_push() },
            }
        }
    }

    impl Drop for Pool {
        fn drop(&mut self) {
            if !self.token.is_null() {
                // SAFETY: can đúng 1 lần `push` thành công ở trên, cùng thread.
                unsafe { raw_pool_pop(self.token) };
            }
        }
    }

    /// Lấy class runtime; `None` nếu không tồn tại (không thể xảy ra trên
    /// app macOS thật — guard để fail-open, không panic).
    fn class_of(name: &[u8]) -> Option<Id> {
        debug_assert!(name.ends_with(&[0]));
        // SAFETY: name là hằng số NUL-terminated hợp lệ.
        let cls = unsafe { raw_get_class(name.as_ptr().cast()) };
        if cls.is_null() {
            return None;
        }
        Some(cls)
    }

    /// SEL từ byte literal NUL-terminated.
    fn sel_of(name: &[u8]) -> Sel {
        debug_assert!(name.ends_with(&[0]));
        // SAFETY: name là hằng số NUL-terminated hợp lệ.
        unsafe { raw_sel_register_name(name.as_ptr().cast()) }
    }

    /// Tạo `NSString` autoreleased từ byte UTF-8 (qua UTF-16 nội bộ) — `None`
    /// nếu runtime lỗi (caller fail-open). Object chỉ sống trong pool hiện
    /// hành của worker.
    fn ns_string_from(text: &str) -> Option<Id> {
        if text.is_empty() {
            return None; // NSString rỗng vô nghĩa cho word-level check.
        }
        let Some(cls) = class_of(b"NSString\0") else {
            return None;
        };
        let sel = sel_of(b"stringWithCharacters:length:\0");
        let chars: Vec<u16> = text.encode_utf16().collect();
        // SAFETY: `cls` valid (đã kiểm non-null); `chars` sống suốt lời gọi;
        // cluster method +string… trả autoreleased instance (nằm trong pool
        // hiện hành của worker, pop sau khi xử lý xong job).
        let str_obj = unsafe {
            msg_send_string_with_chars(cls, sel, chars.as_ptr(), chars.len() as NSUInteger)
        };
        if str_obj.is_null() {
            return None;
        }
        Some(str_obj)
    }

    // Selectors dùng nhiều lần — tạo lại mỗi lần `sel_registerName` là rẻ
    // (runtime intern), không cần cache; giữ đơn giản cho fail-open.
    const SEL_CHECK_SPELLING: &[u8] =
        b"checkSpelling:startingAt:language:wrap:inSpellDocumentWithTag:wordCount:\0";
    const SEL_GUESSES: &[u8] = b"guessesForWordRange:inString:language:inSpellDocumentWithTag:\0";
    const SEL_SHARED: &[u8] = b"sharedSpellChecker\0";
    const SEL_AVAILABLE_LANGUAGES: &[u8] = b"availableLanguages\0";
    const SEL_UNIQUE_TAG: &[u8] = b"uniqueSpellDocumentTag\0";
    const SEL_COUNT: &[u8] = b"count\0";
    const SEL_OBJECT_AT: &[u8] = b"objectAtIndex:\0";
    const SEL_UTF8: &[u8] = b"UTF8String\0";
    const SEL_RESPONDS_TO: &[u8] = b"respondsToSelector:\0";
    const SEL_CLOSE_DOC: &[u8] = b"closeSpellDocumentWithTag:\0";

    /// Phiên làm việc giữ trên **worker thread** (không bao giờ qua channel):
    /// instance `NSSpellChecker`, tag resolved từ `availableLanguages`, và tag
    /// spell-document riêng cho TextVN (M3).
    struct Session {
        checker: Id,
        language: String,
        doc_tag: NSUInteger,
    }

    impl Drop for Session {
        fn drop(&mut self) {
            // SAFETY: `self.checker` hợp lệ tại thời điểm drop (worker chỉ kết
            // thúc sau khi hết mọi job); guarded bên trong đều fail-open.
            close_spell_document(self);
        }
    }

    /// Probe 1 lần: lấy `sharedSpellChecker`, đọc `availableLanguages`, rồi
    /// resolve tag (nguyên tắc Fassist-107 — chỉ dùng bản thân OS có entry)
    /// và cấp `uniqueSpellDocumentTag` (M3). `None` ⇒ không hỗ trợ.
    fn probe(language_tag: &str) -> Option<Session> {
        let cls = class_of(b"NSSpellChecker\0")?;
        let sel_shared = sel_of(SEL_SHARED);
        // SAFETY: `cls` non-null; `sharedSpellChecker` là class method công khai
        // AppKit (P4-10 §3.1). Trả instance chỉ dùng trên worker thread này.
        let checker = unsafe { msg_send_shared_checker(cls, sel_shared) };
        if checker.is_null() {
            return None;
        }
        let languages = available_languages(checker)?;
        let language = language_match(&languages, language_tag)?;
        let sel_tag = sel_of(SEL_UNIQUE_TAG);
        // SAFETY: `cls` non-null; class method verify từ bảng P4-10 §3.1. Tag
        // chỉ có ý nghĩa ở trong phiên này (worker tự giữ).
        let doc_tag = unsafe { msg_send_unique_tag(cls, sel_tag) };
        Some(Session {
            checker,
            language,
            doc_tag,
        })
    }

    /// Đọc `availableLanguages` và copy từng entry qua `UTF8String`. `None` =
    /// runtime lỗi / danh sách rỗng ⇒ caller coi như không hỗ trợ (fail-open).
    fn available_languages(checker: Id) -> Option<Vec<String>> {
        let sel = sel_of(SEL_AVAILABLE_LANGUAGES);
        // SAFETY: `checker` hợp lệ (non-null đã kiểm ở `probe`), method công
        // khai; trả NSArray autoreleased (nằm trong pool hiện hành).
        let array = unsafe { msg_send_available_languages(checker, sel) };
        if array.is_null() {
            return None;
        }
        let sel_count = sel_of(SEL_COUNT);
        // SAFETY: `array` là NSArray hợp lệ.
        let count = unsafe { msg_send_count(array, sel_count) };
        if count == 0 {
            return None;
        }
        let count = (count as usize).min(128);
        let sel_item = sel_of(SEL_OBJECT_AT);
        let sel_utf8 = sel_of(SEL_UTF8);
        let mut out = Vec::with_capacity(count);
        for idx in 0..count {
            // SAFETY: `idx < count` của mảng → hợp lệ theo contract NSArray.
            let item = unsafe { msg_send_object_at(array, sel_item, idx as NSUInteger) };
            if item.is_null() {
                break;
            }
            // SAFETY: NSString non-null; trả pointer NUL-terminated sống đến ít
            // nhất ngay sau lời gọi (copy ra Rust ngay).
            let utf8 = unsafe { msg_send_utf8_string(item, sel_utf8) };
            if utf8.is_null() {
                continue;
            }
            // SAFETY: pointer NUL-terminated từ runtime, đọc tạm (copy) trước
            // khi pool pop (constant chỉ tới lượt call này).
            let text = unsafe { CStr::from_ptr(utf8) };
            if let Ok(text) = text.to_str() {
                if !text.is_empty() {
                    out.push(text.to_owned());
                }
            }
        }
        if out.is_empty() {
            return None;
        }
        Some(out)
    }

    /// Kiểm tra 1 từ: `checkSpelling(...)` trả `NSRange` — **UTF-16** (P4-10
    /// §3.2); `length == 0` = từ đúng (true, nghĩa là "không có từ sai").
    /// **Fail-open**: `true` (từ đúng) với mọi lỗi (từ rỗng, runtime lỗi).
    fn contains_word(session: &Session, word: &str) -> bool {
        let (Some(text), Some(language)) =
            (ns_string_from(word), ns_string_from(&session.language))
        else {
            return true; // fail-open: từ rỗng / runtime lỗi ⇒ im lặng.
        };
        let sel = sel_of(SEL_CHECK_SPELLING);
        // SAFETY: `session.checker` hợp lệ, mọi object sống trong pool hiện
        // hành của job; `word_count` = nil (header cho phép nil — ta không đọc
        // word count).
        let range = unsafe {
            msg_send_check_spelling(
                session.checker,
                sel,
                text,
                0, // startingAt = 0
                language,
                0, // wrap = FALSE — không lặp lại từ đầu trong word-level.
                session.doc_tag as NSInteger,
                None, // wordCount: nil
            )
        };
        range.length == 0
    }

    /// Gợi ý cho 1 từ: `guessesForWordRange:...` → NSArray; copy tối đa
    /// `limit` (cap [`MAX_SUGGEST_RESULTS`]) sang `Vec<String>` rồi NSArray
    /// nhả cho pool. **Fail-open**: rỗng trước mọi lỗi (chỉ helper fail
    /// thuộc bộ chứa gợi ý, không bao giờ báo lỗi).
    fn suggestions_for(session: &Session, word: &str, limit: usize) -> Vec<String> {
        let (Some(text), Some(language)) =
            (ns_string_from(word), ns_string_from(&session.language))
        else {
            return Vec::new();
        };
        let utf16_len = word.encode_utf16().count() as NSUInteger;
        let range = NsRange {
            location: 0,
            length: utf16_len,
        };
        let sel = sel_of(SEL_GUESSES);
        // SAFETY: như `contains_word`; `range` là nhỏ struct theo giá trị theo
        // C ABI (đọc ghi chú trên `NsRange`).
        let array = unsafe {
            msg_send_guesses(
                session.checker,
                sel,
                range,
                text,
                language,
                session.doc_tag as NSInteger,
            )
        };
        if array.is_null() {
            return Vec::new();
        }
        let sel_count = sel_of(SEL_COUNT);
        // SAFETY: `array` là NSArray hợp lệ.
        let count = unsafe { msg_send_count(array, sel_count) } as usize;
        let sel_item = sel_of(SEL_OBJECT_AT);
        let sel_utf8 = sel_of(SEL_UTF8);
        let mut out = Vec::with_capacity(limit);
        for idx in 0..count.min(limit) {
            // SAFETY: `idx` trong bounds.
            let item = unsafe { msg_send_object_at(array, sel_item, idx as NSUInteger) };
            if item.is_null() {
                break;
            }
            // SAFETY: NSString non-null → pointer NUL-terminated (như trên).
            let utf8 = unsafe { msg_send_utf8_string(item, sel_utf8) };
            if utf8.is_null() {
                continue;
            }
            // SAFETY: copy ngay trước khi pool pop nhả object.
            let text = unsafe { CStr::from_ptr(utf8) };
            if let Ok(text) = text.to_str() {
                if !text.is_empty() {
                    out.push(text.to_owned());
                }
            }
        }
        out
    }

    /// `+closeSpellDocumentWithTag:` — **không** verify được chữ ký selector
    /// trong phiên (không có máy macOS), nên gọi **CÓ guard**
    /// `respondsToSelector:`: selector không tồn tại ⇒ im lặng (fail-open,
    /// tuyệt đối không cho runtime Obj-C ném exception khi teardown).
    fn close_spell_document(session: &Session) {
        let Some(cls) = class_of(b"NSSpellChecker\0") else {
            return;
        };
        let sel_close = sel_of(SEL_CLOSE_DOC);
        let sel_guard = sel_of(SEL_RESPONDS_TO);
        // SAFETY: guard là `+respondsToSelector:` (metaclass, NSObject) — BOOL;
        // chỉ khi selector tồn tại thì gọi `closeSpellDocumentWithTag:`.
        let responds = unsafe { msg_send_responds_to(cls, sel_guard, sel_close) };
        if responds == 0 {
            return;
        }
        unsafe { msg_send_close_spell_document(cls, sel_close, session.doc_tag) };
    }

    /// Lệnh gửi sang worker thread (chỉ dữ liệu "thô", con trỏ Obj-C không
    /// bao giờ qua channel).
    #[derive(Debug)]
    pub(super) enum Job {
        /// `contains` — reply: `true` = từ đúng (hoặc fail-open).
        Contains {
            word: String,
            reply: SyncSender<bool>,
        },
        /// `suggest` — reply: gợi ý (rỗng = fail-open).
        Suggest {
            word: String,
            limit: usize,
            reply: SyncSender<Vec<String>>,
        },
    }

    /// Handle của worker thread (chỉ giữ kênh gửi; thread tự kết thúc khi
    /// handle drop giống Windows/Linux).
    #[derive(Debug)]
    pub(super) struct Worker {
        pub(super) tx: SyncSender<Job>,
    }

    /// Cache "tag có được hỗ trợ không" — capability hỏi tối đa 1 lần / tag /
    /// process (P4-10 AC-2). Không cache dữ liệu người dùng, chỉ tag + bool.
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

    /// Tạo worker thread + đợi "sẵn sàng" (probe có timeout, như Windows).
    ///
    /// `want_checker = false` → chỉ hỏi capability, **không** giữ `Session`
    /// (không cấp spell-document tag vĩnh viễn cho probe-only).
    /// `None` = thread không spawn / probe quá hạn 50 ms ⇒ caller fail-open.
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

    /// Thân worker: **toàn bộ** vòng đời `NSSpellChecker` gói trong 1 thread,
    /// với autorelease pool hai lớp: pool **ngoài** giữ `Session` sống suốt
    /// phiên worker, pool **trong** mỗi job giải phóng NSString/NSArray sinh
    /// trong job (không dồn nhớ suốt phiên dài).
    fn worker_main(
        language_tag: &str,
        want_checker: bool,
        jobs: Receiver<Job>,
        ready: SyncSender<bool>,
    ) {
        // Đã biết không hỗ trợ theo cache => không tốn chi phí probe (như
        // Windows/Linux: cache chốt trước).
        if support_cached(language_tag) == Some(false) {
            ready.send(false).ok();
            return;
        }

        let outer_pool = Pool::push();
        let session = {
            let _inner_probe_pool = Pool::push();
            probe(language_tag)
        };
        let Some(session) = session else {
            store_support(language_tag, false);
            ready.send(false).ok();
            return; // `outer_pool` tự drop tại đây (pop trên đúng thread).
        };
        store_support(language_tag, true);

        if ready.send(true).is_err() {
            // Caller bỏ (probe timeout) => không giữ thread; `session` drop
            // (kèm closeSpellDocument guarded) + pool outer pop tại đây.
            return;
        }

        while let Ok(job) = jobs.recv() {
            // Pool **trong** mỗi job: giải phóng NSString/NSArray sinh trong
            // job (pool ngoài giữ `Session` sống).
            let _job_pool = Pool::push();
            handle_job(&session, job);
        }

        // Kết thúc worker: `session` drop (kèm `closeSpellDocument` guarded)
        // rồi pool ngoài pop — trên đúng thread sinh chúng (AppKit ràng buộc).
    }

    fn handle_job(session: &Session, job: Job) {
        match job {
            Job::Contains { word, reply } => {
                reply.send(contains_word(session, &word)).ok();
            }
            Job::Suggest { word, limit, reply } => {
                reply.send(suggestions_for(session, &word, limit)).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Logic ngân sách thuần Rust — chạy trên MỌI target (đúng yêu cầu
    // brief P4-10: chỉ test được thứ không cần macOS).
    // ---------------------------------------------------------------

    #[test]
    fn suggest_chua_contains_thi_khong_goi() {
        let budget = SuggestBudget::new();
        assert!(!budget.allow(), "cửa sổ ngân sách chưa mở");
    }

    #[test]
    fn suggest_bi_cap_so_lan_goi() {
        let budget = SuggestBudget::new();
        budget.anchor();
        for i in 0..SUGGEST_CALLS_PER_CHECK {
            assert!(budget.allow(), "lần {i} phải được phép");
        }
        assert!(!budget.allow(), "vượt cap ⇒ fail-open (rỗng)");
    }

    #[test]
    fn suggest_het_ngan_sach_thoi_gian() {
        let budget = SuggestBudget::new();
        budget.anchor();
        std::thread::sleep(SUGGEST_BUDGET + Duration::from_millis(20));
        assert!(!budget.allow(), "qua 50 ms ⇒ fail-open (rỗng)");
    }

    #[test]
    fn contains_giua_cua_so_khong_reset_bo_dem() {
        // Như Linux: đơn vị gọi mỗi từ ⇒ `anchor` giữa cửa sổ CÒN HẠN không
        // được reset bộ đếm; chỉ cửa sổ mới (đã hết hạn) mới đặt lại.
        let budget = SuggestBudget::new();
        budget.anchor();
        for _ in 0..SUGGEST_CALLS_PER_CHECK {
            assert!(budget.allow());
        }
        assert!(!budget.allow());
        budget.anchor();
        assert!(!budget.allow(), "anchor giữa cửa sổ không reset");
    }

    // ---------------------------------------------------------------
    // `language_match` — thuần Rust, inject được danh sách giả (yêu cầu
    // brief P4-10: test `is_supported` với danh sách ngôn ngữ giả).
    // ---------------------------------------------------------------

    #[test]
    fn khong_co_ngon_ngu_thi_khong_sua_loi() {
        // Máy chưa bật "Spelling = Vietnamese" (M1) ⇒ Fassist-107: phải là
        // None ⇒ NoDictionary ⇒ KHÔNG báo lỗi spelling nào.
        assert!(language_match(&["en".into(), "fr".into()], "vi-VN").is_none());
        assert!(language_match(&["en".into(), "vi_DauCu".into()], "vi").is_none());
    }

    #[test]
    fn khop_primary_subtag_vi_vn_sang_vi() {
        // Sketch P4-10 §3.3: entry `vi` khớp tag `vi-VN` ở MỨC primary.
        assert_eq!(
            language_match(&["en".into(), "vi".into()], "vi-VN"),
            Some("vi".to_owned())
        );
        // Đảo: entry "vi-VN"?? Không có trong availableLanguages thật nhưng
        // nếu có phải khớp đúng trước.
        assert_eq!(
            language_match(&["vi".into(), "vi-VN".into()], "vi-VN"),
            Some("vi-VN".to_owned())
        );
    }

    #[test]
    fn khop_khong_phan_biet_hoa_thuong() {
        assert_eq!(
            language_match(&["VI".into()], "vi"),
            Some("VI".to_owned()),
            "entry trong availableLanguages phải được dùng làm tag (không
             phải tag gốc của caller)"
        );
        assert_eq!(
            language_match(&["vi".into()], "VI-vn"),
            Some("vi".to_owned())
        );
    }

    #[test]
    fn tag_rong_va_khong_hop_le_khong_khop() {
        assert!(language_match(&["vi".into()], "").is_none());
        assert!(language_match(&["vi".into()], "   ").is_none());
        assert!(language_match(&[], "vi").is_none());
        // Khác primary subtag ⇒ không khớp (tránh mượn checker nhầm).
        assert!(language_match(&["en".into()], "vi").is_none());
    }

    // ---------------------------------------------------------------
    // Shell fail-open khi feature `os-macos` TẮT (mặc định trên mọi OS —
    // kể cả target thật khi chưa bật trong gói phát hành) — hành vi y hệt
    // `assist::dict::NoDictionary` (AC-1 / Fassist-107).
    // ---------------------------------------------------------------
    #[cfg(not(all(target_os = "macos", feature = "os-macos")))]
    mod shell_fail_open {
        use super::*;

        #[test]
        fn tag_rong_va_khong_hop_le_thi_khong_co_tu_dien() {
            // "zz-ZZ" không phải language tag có thật.
            assert!(!MacSpellChecker::is_supported("zz-ZZ"));
            assert!(!MacSpellChecker::is_supported("   "));
            assert!(matches!(
                MacSpellChecker::new("zz-ZZ"),
                Err(crate::DictError::NoDictionary)
            ));
            assert!(matches!(
                MacSpellChecker::new(""),
                Err(crate::DictError::NoDictionary)
            ));
        }

        #[test]
        fn phuong_thuc_tat_ca_fail_open_khong_panic() {
            let checker = MacSpellChecker {
                budget: SuggestBudget::new(),
            };
            assert!(checker.contains("helo"), "feature tắt ⇒ mọi từ đúng");
            assert!(checker.contains(""), "từ rỗng → true");
            assert!(checker.suggest("helo", 3).is_empty());
            assert!(checker.suggest("", 3).is_empty());
            assert!(checker.suggest("x", 0).is_empty());
        }

        #[test]
        fn factory_macos_tra_none_khi_khong_ho_tro() {
            // Ví dụ test bắt buộc brief P4-10: factory luôn trả `None` (feature tắt
            // ⇒ không thể có từ điển OS) ⇒ NoDictionary ⇒ không báo lỗi.
            assert!(crate::os_provider_macos("vi-VN").is_none());
            assert!(crate::os_provider_macos("vi").is_none());
            assert!(crate::os_provider_macos("   ").is_none());
        }
    }

    /// Test tích hợp **có điều kiện**, chỉ chạy trên macOS với feature
    /// `os-macos` bật (không máy macOS trong phiên ⇒ không chạy trên Windows).
    /// Kiểm tra đường đi AppKit thật: thread, UTF-16, ngân sách suggest.
    #[cfg(all(target_os = "macos", feature = "os-macos"))]
    mod integration {
        use super::*;

        #[test]
        fn en_us_neu_may_co_thi_provider_hoat_dong() {
            if !MacSpellChecker::is_supported("en-US") {
                return;
            }
            let Ok(checker) = MacSpellChecker::new("en-US") else {
                return; // có cờ nhưng khởi tạo lỗi ⇒ fail-open, không fail test
            };
            assert!(checker.contains("hello"), "từ đúng không được báo lỗi");
            let suggestions = checker.suggest("helo", 3);
            assert!(suggestions.len() <= 3, "phải cắt theo limit");
        }

        #[test]
        fn vi_neu_may_co_thi_duong_goi_khong_panic() {
            let vi = MacSpellChecker::is_supported("vi-VN") || MacSpellChecker::is_supported("vi");
            if !vi {
                return;
            }
            if let Ok(checker) = MacSpellChecker::new("vi") {
                let _ = checker.contains("hoà");
                let _ = checker.suggest("hoà", 3);
            }
        }
    }
}
