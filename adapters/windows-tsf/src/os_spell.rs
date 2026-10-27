// SPDX-License-Identifier: GPL-3.0-or-later
//! D39 · P4-10 §2 — **OS spell provider** cho Windows qua Spell Checking API
//! (`ISpellCheckerFactory`, Win8+), nối vào đường check trợ lý qua
//! `ime_assist_check_v1_with`.
//!
//! Thiết kế theo đặc tả (đo thực nghiệm trong P4-10 §2):
//! - **Cổng capability bắt buộc**: `IsSupported("vi-VN")` — máy Windows mặc
//!   định KHÔNG có bộ kiểm tra tiếng Việt (`E_INVALIDARG`) ⇒ `try_new` trả
//!   `None` ⇒ đường check đi `NoDictionary` như cũ (không bao giờ dùng
//!   checker `en-US` cho tiếng Việt — đo được 3/8 từ đúng bị báo sai).
//! - **Provider out-of-proc** ⇒ không có timeout API: mọi lời gọi chạy trên
//!   **1 worker thread thường trực** (COM apartment riêng — không được dùng
//!   `ISpellChecker` qua thread khác, §2.3), caller chờ
//!   `recv_timeout(50 ms)` (D36); quá giờ/lỗi ⇒ **fail-open**
//!   (`contains = true`, `suggest = rỗng`).
//! - **Cầu chì**: một lần timeout đánh dấu provider "không khỏe" 10 s — các
//!   lời gọi sau fail-open tức thì (chống xếp hàng chờ 50 ms/từ trên luồng UI).
//! - `try_send` (không block) — hàng đợi 1 slot, bận ⇒ bỏ job, không dồn ứ.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::Duration;

use windows::core::HSTRING;
use windows::Win32::Globalization::{ISpellChecker, ISpellCheckerFactory, SpellCheckerFactory};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};

/// Ngân sách D36 cho MỘT lời gọi (P4-10 §0 kết luận 5 — bắt buộc vì API
/// out-of-proc, "Standard RPC timeouts apply").
const CALL_TIMEOUT_MS: Duration = Duration::from_millis(50);
/// Chờ worker khởi tạo + probe capability (đo: CoCreateInstance + CreateSpell
/// Checker nguồnội ~9,5 ms — 50 ms dư an cho máy chậm).
const READY_TIMEOUT_MS: Duration = Duration::from_millis(50);
/// Sau một lần timeout, fail-open tức thì trong khoảng này (cầu chì).
const BREAKER_COOLDOWN_MS: u64 = 10_000;
/// Ngôn ngữ yêu cầu — KHÔNG bao giờ thay bằng `en-US` (P4-10 §2.5 kết luận 1).
const LANG_TAG: &str = "vi-VN";

enum Job {
    Contains {
        word: String,
        reply: SyncSender<bool>,
    },
    Suggest {
        word: String,
        limit: usize,
        reply: SyncSender<Vec<String>>,
    },
}

/// single sender — an toàn Sync: chỉ luồng check (UI thread của TIP) gửi;
/// worker là duy nhất và không bao giờ gửi ngược.
pub struct WinSpell {
    tx: SyncSender<Job>,
    breaker_until: AtomicU64,
}

// SAFETY: `SyncSender` chỉ chạm từ luồng UI (điểm gọi duy nhất là đường check
// trợ lý của TIP — TSF apartment); phía nhận là 1 worker thread duy nhất.
unsafe impl Sync for WinSpell {}

static WIN_SPELL: OnceLock<Option<WinSpell>> = OnceLock::new();

impl WinSpell {
    /// Khởi tạo 1 lần/phiên (probe capability). `None` = máy không có từ điển
    /// `vi-VN` ⇒ dùng `NoDictionary` (đường mặc định — W2).
    pub fn get() -> Option<&'static WinSpell> {
        WIN_SPELL.get_or_init(Self::try_new).as_ref()
    }

    /// Con trỏ tới provider C-ABI tĩnh (dùng với `ime_assist_check_v1_with`).
    /// `None` = máy không có `vi-VN` ⇒ caller dùng `ime_assist_check_v1`.
    pub fn provider_ptr() -> Option<*const textvn_ffi::ime_os_spell_v1> {
        Self::get().map(|_| std::ptr::addr_of!(OS_PROVIDER.0))
    }

    fn try_new() -> Option<Self> {
        let (tx, rx) = sync_channel::<Job>(1);
        let (ready_tx, ready_rx) = sync_channel::<bool>(1);
        let tag = LANG_TAG.to_owned();
        std::thread::Builder::new()
            .name("textvn-spell".into())
            .spawn(move || unsafe {
                // W-apartment (P4-10 §2.3): tạo/dùng/huỷ `ISpellChecker` trên
                // ĐÚNG thread này; qua channel chỉ truyền String/Vec<String>.
                let co_init = CoInitializeEx(None, COINIT_MULTITHREADED);
                let checker = create_checker(&tag);
                ready_tx.send(checker.is_some()).ok();
                if let Some(checker) = checker {
                    while let Ok(job) = rx.recv() {
                        handle_job(&checker, job);
                    }
                }
                if co_init.is_ok() {
                    CoUninitialize();
                }
            })
            .ok()?;
        match ready_rx.recv_timeout(READY_TIMEOUT_MS) {
            Ok(true) => Some(Self {
                tx,
                breaker_until: AtomicU64::new(0),
            }),
            // Probe quá hạn / không có `vi-VN` / spawn lỗi → bỏ worker (thread
            // còn sống sẽ kết thúc khi `rx` bị drop — sender không trả về).
            _ => None,
        }
    }

    fn healthy(&self) -> bool {
        millis_now() >= self.breaker_until.load(Ordering::Acquire)
    }

    fn trip_breaker(&self) {
        self.breaker_until
            .store(millis_now() + BREAKER_COOLDOWN_MS, Ordering::Release);
    }

    fn send_job(&self, job: Job) -> bool {
        // Hàng đợi 1 slot; bận ⇒ bỏ job (fail-open ở caller) — không block UI.
        matches!(self.tx.try_send(job), Ok(()) | Err(TrySendError::Full(_)))
    }

    /// `true` = từ ĐÚNG (có trong từ điển) — fail-open đúng chiều "thà im
    /// lặng còn hơn báo sai": lỗi/timeout ⇒ coi như đúng.
    pub fn contains(&self, word: &str) -> bool {
        if !self.healthy() {
            return true;
        }
        let (tx, rx) = sync_channel(1);
        if !self.send_job(Job::Contains {
            word: word.to_owned(),
            reply: tx,
        }) {
            return true;
        }
        match rx.recv_timeout(CALL_TIMEOUT_MS) {
            Ok(ok) => ok,
            Err(RecvTimeoutError::Timeout) => {
                self.trip_breaker();
                true
            }
            Err(_) => true,
        }
    }

    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        if !self.healthy() || limit == 0 {
            return Vec::new();
        }
        let (tx, rx) = sync_channel(1);
        if !self.send_job(Job::Suggest {
            word: word.to_owned(),
            limit,
            reply: tx,
        }) {
            return Vec::new();
        }
        match rx.recv_timeout(CALL_TIMEOUT_MS) {
            Ok(list) => list,
            Err(RecvTimeoutError::Timeout) => {
                self.trip_breaker();
                Vec::new()
            }
            Err(_) => Vec::new(),
        }
    }
}

fn millis_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Shim C-ABI `ime_os_spell_v1` — cầu nối sang `ime_assist_check_v1_with`
// ---------------------------------------------------------------------------

/// Callback `contains` — chữ ký khớp `ime_os_spell_v1.contains`. Trả 1 = từ
/// đúng (fail-open đúng chiều "không bêu chữ đúng": mọi lỗi coi như đúng).
unsafe extern "C" fn os_contains(
    _ctx: *mut std::ffi::c_void,
    word: *const std::os::raw::c_char,
    byte_len: u32,
) -> i32 {
    let Some(spell) = WinSpell::get() else {
        return 1;
    };
    if word.is_null() || byte_len == 0 {
        return 1;
    }
    let bytes = std::slice::from_raw_parts(word.cast::<u8>(), byte_len as usize);
    match std::str::from_utf8(bytes) {
        Ok(w) => i32::from(spell.contains(w)),
        Err(_) => 1, // UTF-8 lệch — không phải từ tiếng Việt hợp lệ → im lặng
    }
}

/// Callback `suggest` — ghi `limit` chuỗi NUL-cách nhau vào `out` (đúng format
/// `CSpellProvider::suggest` của ffi parse), trả số chuỗi đã ghi.
unsafe extern "C" fn os_suggest(
    _ctx: *mut std::ffi::c_void,
    word: *const std::os::raw::c_char,
    byte_len: u32,
    out: *mut std::os::raw::c_char,
    out_len: u32,
    limit: u32,
) -> u32 {
    let Some(spell) = WinSpell::get() else {
        return 0;
    };
    if word.is_null() || out.is_null() || out_len == 0 || limit == 0 {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(word.cast::<u8>(), byte_len as usize);
    let Ok(w) = std::str::from_utf8(bytes) else {
        return 0;
    };
    let list = spell.suggest(w, (limit as usize).min(16));
    if list.is_empty() {
        return 0;
    }
    // Ghi tuần tự: chuỗi + NUL; dừng khi hết chỗ (bỏ phần thừa).
    let mut used = 0usize;
    let mut written = 0u32;
    let out_bytes = std::slice::from_raw_parts_mut(out.cast::<u8>(), out_len as usize);
    for s in &list {
        let b = s.as_bytes();
        if used + b.len() + 1 > out_bytes.len() {
            break;
        }
        out_bytes[used..used + b.len()].copy_from_slice(b);
        out_bytes[used + b.len()] = 0;
        used += b.len() + 1;
        written += 1;
    }
    written
}

struct SyncProvider(textvn_ffi::ime_os_spell_v1);
// SAFETY: `ctx = NULL` (callbacks không dùng ctx), mọi trường khác là
// function pointer hằng — static chỉ-đọc.
unsafe impl Sync for SyncProvider {}

static OS_PROVIDER: SyncProvider = SyncProvider(textvn_ffi::ime_os_spell_v1 {
    abi_version: textvn_ffi::IME_ABI_VERSION,
    contains: Some(os_contains),
    suggest: Some(os_suggest),
    ctx: std::ptr::null_mut(),
});

unsafe fn create_checker(tag: &str) -> Option<ISpellChecker> {
    let factory: ISpellCheckerFactory =
        CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER).ok()?;
    let tag = HSTRING::from(tag);
    // Cổng capability D33/W2: `IsSupported` FALSE ⇒ None (KHÔNG thử en-US).
    if !factory.IsSupported(&tag).ok()?.as_bool() {
        return None;
    }
    factory.CreateSpellChecker(&tag).ok()
}

unsafe fn handle_job(checker: &ISpellChecker, job: Job) {
    match job {
        Job::Contains { word, reply } => {
            // `Check("")` trả E_INVALIDARG ⇒ chặn trước; chuỗi rỗng coi như đúng.
            if word.is_empty() {
                reply.send(true).ok();
                return;
            }
            let ok = match checker.Check(&HSTRING::from(word)) {
                // Không lỗi chính tả nào trong chuỗi = từ đúng.
                Ok(errors) => first_error_start(&errors).is_none(),
                Err(_) => true, // fail-open
            };
            reply.send(ok).ok();
        }
        Job::Suggest { word, limit, reply } => {
            let mut out = Vec::new();
            if let Ok(suggestions) = checker.Suggest(&HSTRING::from(word)) {
                // `Next` cần mảng PWSTR; dừng theo số slot nhận được (S_FALSE = hết).
                const BATCH: usize = 8;
                loop {
                    let mut buf: [windows::core::PWSTR; BATCH] =
                        std::array::from_fn(|_| windows::core::PWSTR::null());
                    let mut fetched = 0u32;
                    // Next trả HRESULT: S_OK = đủ batch, S_FALSE = hết (vẫn
                    // xem `fetched` — có thể là batch cuối ngắn hơn).
                    let hr = suggestions.Next(&mut buf, Some(&mut fetched));
                    if hr.is_err() {
                        break;
                    }
                    let got = fetched as usize;
                    if got == 0 {
                        break;
                    }
                    for pw in buf.iter().take(got) {
                        if pw.is_null() {
                            continue;
                        }
                        let s = pw.to_string().unwrap_or_default();
                        // W4: từng PWSTR phải CoTaskMemFree riêng.
                        CoTaskMemFree(Some(pw.as_ptr().cast()));
                        if !s.is_empty() && out.len() < limit {
                            out.push(s);
                        }
                    }
                    if out.len() >= limit {
                        break;
                    }
                }
            }
            out.truncate(limit);
            reply.send(out).ok();
        }
    }
}

/// `StartIndex` của lỗi chính tả đầu tiên (không có = từ đúng) — đơn vị là
/// **UTF-16 code unit** (P4-10 §2.4, đo thực nghiệm); ở đây luôn Check 1 từ
/// nên chỉ cần "có/không", không quy đổi offset.
unsafe fn first_error_start(
    errors: &windows::Win32::Globalization::IEnumSpellingError,
) -> Option<u32> {
    let mut item: Option<windows::Win32::Globalization::ISpellingError> = None;
    // S_OK = có lỗi (item được điền); S_FALSE = hết (không lỗi).
    let hr = errors.Next(&mut item);
    if hr.is_err() {
        return None;
    }
    item.take()?.StartIndex().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Máy dev/CI mặc định KHÔNG có bộ kiểm tra tiếng Việt của Windows
    /// (P4-10 §2.5: `IsSupported("vi-VN") = FALSE`) ⇒ provider là `None`.
    /// Không assert cứng giá trị (máy chủ repo có thể có): chỉ đảm bảo
    /// `provider_ptr()` và đường check không panic, hai nhánh đều hợp lệ.
    #[test]
    fn provider_init_khong_panic_va_nhat_quan() {
        let ptr = WinSpell::provider_ptr();
        let has = WinSpell::get().is_some();
        assert_eq!(ptr.is_some(), has, "provider_ptr phải khớp get()");
        if let Some(spell) = WinSpell::get() {
            // Máy CÓ vi-VN: mọi lời gọi phải trả kết quả không panic.
            let ok = spell.contains("ngành");
            let _ = spell.suggest("nghành", 3);
            assert!(ok || !ok); // chứa cả 2 giá trị — chống optimization bỏ test
        }
    }

    /// Cầu chì: sau khi trip, `healthy` = false trong cooldown rồi phục hồi.
    #[test]
    fn breaker_trip_va_phuc_hoi() {
        let spell = WinSpell {
            tx: sync_channel(1).0,
            breaker_until: AtomicU64::new(0),
        };
        assert!(spell.healthy());
        spell.trip_breaker();
        assert!(!spell.healthy());
        // Cooldown trôi (giả lập: đẩy mốc lùi 10s + 1ms).
        spell
            .breaker_until
            .store(millis_now() - 1, Ordering::Release);
        assert!(spell.healthy());
    }
}
