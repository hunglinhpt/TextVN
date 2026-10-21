# P4-10 — Đặc tả **OS spell provider** (Windows · macOS · Linux)

- **Ngày:** 2026-10-21 · **Trạng thái:** Đặc tả kỹ thuật (chưa code) · **Phần:** 4 — P4
- **Phạm vi:** cách lấy **gợi ý từ** từ spell-checker **có sẵn của OS** và cắm vào
  `assist::dict::Dictionary`; **không** gồm luật ngữ pháp (luật của ta) và không gồm UI overlay
  (`docs/release/ui-spec.md`, P4-3).
- **Quyết định liên quan:** `docs/50-assist/P4-0-MASTER-PLAN.md` §3.1 — **D32** (provider ở biên
  FFI/adapter), **D33** (OS-first), **D35** (L0 dùng OS), **D36** (async, timeout ≤ 50 ms,
  fail-open), **D39** (không phát hành `hunspell-vi`) · `docs/adr/README.md` **ADR-012** (không
  vendor Harper), **ADR-014** (license `hunspell-vi`) · nền nghiên cứu: `P4-4-OS-NATIVE-ASSIST.md`,
  `P4-9-REVIEW-DICT.md`.
- **Code hiện có (đọc thật, 2026-10-21):** `assist/src/dict.rs` (trait + `NoDictionary`),
  `assist/src/lib.rs` (`check_with(text, &dyn Dictionary)`), `assist/src/rules/spelling.rs`
  (luật AS-040 gọi `contains`/`suggest`), `assist/src/rules/language.rs` (L0 heuristic),
  `ffi/src/lib.rs` (`ime_assist_result_v1`, `IME_ASSIST_KIND_SPELLING = 10`).

> **Cách đọc tài liệu này.** Mọi khẳng định về API đều có nguồn ở §8. Chỗ **đo được trên máy**
> ghi rõ "đo thực nghiệm" + môi trường. Chỗ **không kiểm chứng được** ghi ⚠️ và liệt kê lại ở §9 —
> **không** suy đoán thay cho dữ kiện.

---

## 0. Kết luận 1 trang

| # | Kết luận | Hệ quả |
|---|---|---|
| 1 | **Crate `windows` 0.61.3 (đang dùng) ĐÃ CÓ** `ISpellCheckerFactory`, `ISpellChecker`, `IEnumSpellingError`, `ISpellingError`, `IEnumString` **và hằng `SpellCheckerFactory`** (CLSID `7AB36653-…`) trong `windows::Win32::Globalization` | **Không** phải gọi COM thủ công bằng vtable; chỉ cần bật thêm feature `Win32_Globalization` (repo **chưa** bật) |
| 2 | Trên Windows 11 (OS build 26300, PowerShell build 26100) sạch: `IsSupported("vi-VN") = FALSE`, `CreateSpellChecker("vi-VN")` trả `E_INVALIDARG (0x80070057)`; `en-US` = `TRUE` (đo thực nghiệm) | Đường **mặc định** trên Windows là **fallback `NoDictionary`** = chỉ luật objective. Provider Windows là *tùy chọn cộng thêm*, không phải đường chính |
| 3 | `ISpellChecker::Check` **rẻ** (~0,06 ms/90 code unit, warm) nhưng `Suggest` **đắt** (~3,1 ms/từ cache, ~6,9 ms/từ mới) — chênh **~50–100×** (đo thực nghiệm) | **Cấm** gọi `suggest()` cho mọi từ OOV trong luồng check; `contains` dùng `Check`, `suggest` phải **lazy** hoặc **có ngân sách** |
| 4 | `StartIndex`/`Length` của Windows là **UTF-16 code unit**, không phải char Unicode (đo thực nghiệm bằng ký tự astral; MS Learn **không** nói đơn vị) | Adapter **buộc** phải convert UTF-16 → char (`assist` là char offset, D27) |
| 5 | MS Learn nói rõ Spell Checking API **out-of-proc** ("Standard RPC error codes may be returned … because they are out-of-proc. Standard RPC timeouts apply") | **D36 là bắt buộc**, không phải phòng xa: phải worker thread + timeout 50 ms + fail-open |
| 6 | Linux: `spelling_checker_check_ranges()` (offset **char Unicode**) **CHỈ có trên `main` (1.0.alpha)**, **KHÔNG** có trong 0.4.8/0.4.10 — mà distro đang phát hành 0.4.x | Với bản phát hành, dùng `spelling_checker_check_word` + `list_corrections` (đúng khớp trait `Dictionary`); `check_ranges` để dành khi 1.0 ra. **Sửa lại giả định ở P4-4 §1** |
| 7 | `hunspell-vi` có trên Fedora/RHEL/Alma/ALT; **không thấy** trong Debian/Ubuntu (Repology, 2026-10-21) và license gói = **GPL-2.0-only** | Trên Debian/Ubuntu: **user tự cài** từ điển; ta **không ship** (ADR-014). Docs P4-4 ghi "gói sẵn của Debian/Ubuntu" là **sai** |
| 8 | macOS: **không** có `hasCorrection` trong API công khai `NSSpellChecker` (2026-10-21) | Dùng `correction(forWordRange:in:language:inSpellDocumentWithTag:) -> String?`; `NSRange` = **UTF-16** → phải convert |

**Chiến lược chốt:** `assist` **không đổi** (0-dep, portable, char offset). Mỗi OS có **một adapter**
implement `Dictionary`, đặt ở biên (Windows: crate `ffi`/`adapters/windows-*`; macOS: Swift IMK;
Linux: C trong `adapters/linux-*`). Không có `vi`/`vi-VN` → adapter **không tồn tại** →
`NoDictionary`. Mọi lời gọi provider đi qua **worker thread + `recv_timeout(50ms)` + fail-open**.

---

## 1. Điểm cắm thật trong code hiện tại

### 1.1 Trait (nguồn: `assist/src/dict.rs`)

```rust
pub trait Dictionary {
    /// `true` nếu `word` (đã chuẩn hoá: lowercase) có trong từ điển.
    fn contains(&self, word: &str) -> bool;
    /// Gợi ý thay thế, tối đa `limit`, tốt nhất trước. Rỗng = không gợi ý (không phải lỗi).
    fn suggest(&self, word: &str, limit: usize) -> Vec<String>;
}
```

- Hợp đồng **đồng bộ** (`&self`, không `async`) — mọi bất đồng bộ/đợi chờ **phải** nằm *bên trong*
  adapter, và **không** được vượt ngân sách.
- `NoDictionary`: `contains` = `true` (mọi từ đúng), `suggest` = rỗng.

### 1.2 Luật tiêu thụ trait (nguồn: `assist/src/rules/spelling.rs`)

Với **mỗi** token là `TokenKind::Word` (đã trừ masked URL/email), dài ≥ 3 char, không có chữ hoa
bên trong, **có** ít nhất một chữ có dấu tiếng Việt:

```rust
let lower = word.to_lowercase();
if ctx.dict.contains(&lower) { continue; }          // 1 lời gọi / từ
let suggestions = ctx.dict.suggest(&lower, 3);      // +1 lời gọi / từ OOV
```

⇒ **Hệ quả thiết kế (đo được, xem §2.5):** nếu `suggest` của OS ~3–7 ms/từ thì một câu 20 từ
lỗi hết sẽ ~60–140 ms ⇒ **vượt 50 ms**. Vì vậy đặc tả này **không** cho adapter gọi `Suggest`
trong `contains`; và `suggest` bị chặn bởi **ngân sách** (§5.3).

### 1.3 Biên FFI hiện có (nguồn: `ffi/src/lib.rs`)

- `ime_assist_issue_v1 { kind, start, end, auto_fixable, confidence, message, suggest_len, suggest }`
  — `suggest` **một** chuỗi, buffer cố định ⇒ gợi ý của OS phải **cắt** trước khi qua FFI.
- `IME_ASSIST_KIND_SPELLING = 10` đã có ⇒ không cần đổi ABI để bật provider.
- `start`/`end` là **char Unicode** (D27) ⇒ mọi offset UTF-16/byte của OS phải đổi **trước** FFI.

---

## 2. Windows — Spell Checking API (`spellcheck.h`, Win8+)

### 2.1 Kiểm tra **thật** crate `windows` 0.61.3 (đã grep registry cargo)

Nguồn: `%USERPROFILE%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\windows-0.61.3\src\Windows\Win32\Globalization\mod.rs`
(file 954 707 byte; grep `SpellCheckerFactory` → **16 hit**, tất cả trong file này).

| Symbol | Có trong `windows` 0.61.3? | Vị trí | Ghi chú |
|---|---|---|---|
| `ISpellCheckerFactory` | ✅ | `Windows::Win32::Globalization` | IID `8e018a9d-2415-4677-bf08-794ea61f94bb`; methods `IsSupported`, `CreateSpellChecker`, `SupportedLanguages` |
| `SpellCheckerFactory` (CLSID) | ✅ | cùng module | `pub const SpellCheckerFactory: GUID = GUID::from_u128(0x7ab36653_1796_484b_bdfa_e74f1db7c1dc)` — **không** cần tự viết GUID |
| `ISpellChecker` | ✅ | cùng module | IID `b6fd0b71-e2bc-4653-8d05-f197e412770b`; `Check`, `Suggest`, `Add`, `Ignore`, `AutoCorrect`, `OptionIds`, `ComprehensiveCheck` |
| `IEnumSpellingError` | ✅ | cùng module | IID `803e3bd4-2828-4410-8290-418d1d73c762`; `unsafe fn Next(&self, *mut Option<ISpellingError>) -> HRESULT` (trả `S_FALSE` khi hết) |
| `ISpellingError` | ✅ | cùng module | IID `b7c82d61-fbe8-4b47-9b27-6c0d2e0de0a3`; `StartIndex`, `Length`, `CorrectiveAction`, `Replacement` |
| `CORRECTIVE_ACTION_{NONE,GET_SUGGESTIONS,REPLACE,DELETE}` | ✅ | cùng module | `0,1,2,3` |
| `IEnumString` | ✅ | `Windows::Win32::System::Com` | IID `00000101-0000-0000-c000-000000000046` |

**Hai điều kiện feature (quan trọng, dễ sai):**

1. Cần bật `features = ["Win32_Globalization"]` — **repo chưa bật**. `Cargo.toml` của
   `adapters/windows-tsf` và `adapters/windows-hook` hiện chỉ có `Win32_Foundation`,
   `Win32_System_Com`, … ⇒ phải thêm khi làm provider.
2. `ISpellChecker::Suggest()` và `ISpellCheckerFactory::SupportedLanguages()` được sinh **có điều
   kiện** `#[cfg(feature = "Win32_System_Com")]` (vì trả `IEnumString` của `System::Com`).
   `Win32_System_Com` **đã** bật ở cả 2 adapter ⇒ coi như đủ, nhưng nếu crate nào chỉ bật
   `Win32_Globalization` thì `Suggest` **không tồn tại** (compile error khó hiểu).

⇒ **Kết luận:** *không* phải đặc tả "gọi COM thủ công bằng vtable". Đặc tả dưới đây dùng crate
`windows` (an toàn ABI, có sẵn `Interface::IID`). Phần "COM thủ công" chỉ giữ lại như **phương án
dự phòng** (§2.7) kèm rủi ro.

### 2.2 Chuỗi gọi API (đúng thứ tự, đúng kiểu)

```
CoInitializeEx(NULL, COINIT_MULTITHREADED)                 // 1 lần / worker thread
CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER)
    -> ISpellCheckerFactory
factory.IsSupported("vi-VN") -> BOOL                        // cổng capability, RẺ, làm 1 lần
factory.CreateSpellChecker("vi-VN") -> ISpellChecker        // E_INVALIDARG nếu không có
checker.Check("cả câu") -> IEnumSpellingError               // 1 lời gọi / câu (không phải / từ)
    loop: enum.Next(&mut Option<ISpellingError>) -> HRESULT // S_OK = còn, S_FALSE = hết
          err.StartIndex, err.Length, err.CorrectiveAction, err.Replacement (LPWSTR -> CoTaskMemFree)
checker.Suggest("từ") -> IEnumString                        // CHỈ khi UI hỏi; đắt
    loop: enum.Next(&mut [PWSTR; N], None)                  // giải phóng từng PWSTR bằng CoTaskMemFree
```

Nguồn: MS Learn `ISpellCheckerFactory` (`IsSupported`, `CreateSpellChecker`), `ISpellChecker::Check`
("Checks the spelling of the supplied text and returns a collection of spelling errors"),
`IEnumSpellingError::Next`, `ISpellingError::StartIndex`; quy tắc bộ nhớ ("All returned LPWSTR
strings … must be released with **CoTaskMemFree**") từ "About the Spell Checking API" §Interfaces.

### 2.3 COM activation & apartment (rủi ro thật)

- MS Learn: *"**ISpellCheckerFactory** is the starting point for clients … which **should be created
  as an in-proc COM Server**"* ⇒ `CLSCTX_INPROC_SERVER`.
- Nhưng *provider* (bộ kiểm tra ngôn ngữ) **có thể** chạy **out-of-proc** (MS Learn §Security:
  *"The Spell Checking API may load external code (spell check providers). It will run this code
  **out-of-proc** and under a restricted security context"*), và §Error handling: *"Standard RPC
  error codes may be returned by any of the API calls **because they are out-of-proc**. Standard RPC
  timeouts apply."*
- **Hệ quả 1 — timeout:** không có timeout tham số. Cách duy nhất là **gọi trên thread riêng** và
  bỏ qua kết quả nếu quá 50 ms (`mpsc::RecvTimeoutError::Timeout` ⇒ fail-open). Không có cách "huỷ"
  lời gọi COM đang chạy ⇒ thread có thể còn treo; phải **giới hạn số thread** (1 thread thường trực
  + hàng đợi 1 slot, bỏ yêu cầu mới nếu đang bận) để không rò thread.
- **Hệ quả 2 — apartment:** KHÔNG tạo `ISpellChecker` ở thread A rồi dùng ở thread B (marshalling
  giữa STA/MTA có thể trả `RPC_E_WRONG_THREAD`). **Tạo, dùng, huỷ trong cùng worker thread**; chỉ
  truyền `String`/`Vec<String>` qua channel.
- **Hệ quả 3 — `CoInitialize`:** TSF TIP chạy trong tiến trình app (COM đã init bởi TSF), nhưng
  `textvn-hook`/`tray`/CLI thì không ⇒ worker thread **phải** `CoInitializeEx(COINIT_MULTITHREADED)`
  và `CoUninitialize()` khi kết thúc. Repo đã có tiền lệ: `adapters/windows-hook/src/main.rs` gọi
  `CoInitializeEx`, `adapters/windows-tsf/src/edit_session.rs` gọi `CoCreateInstance` với
  `CLSCTX_INPROC_SERVER`.
- **Hệ quả 4 — lỗi đã biết ngoài repo:** `RPC_E_CANTCALLOUT_ININPUTSYNCCALL (0x8001010D)` khi gọi
  trong input-sync call; `0x8007045A` (DLL init failed) trên một số máy. Đây là **lý do** phải bọc
  `HRESULT` thành `Option` và fail-open, không `unwrap`. (Nguồn: issue dotnet/wpf #8772 — ⚠️ nguồn
  cộng đồng, không phải MS Learn.)

### 2.4 Đơn vị offset — **đo thực nghiệm**, tài liệu MS **không** nói

MS Learn `ISpellingError::StartIndex` chỉ ghi *"position in the checked text"*, **không** nêu đơn vị.

Đo trên Windows 11 (OS build 26300, PowerShell build 26100), checker `en-US`:

| Chuỗi kiểm tra | Độ dài `.NET` (UTF-16) | `StartIndex` báo | Suy ra |
|---|---|---|---|
| `"ô helo"` (`ô` = U+00F4) | 6 | **2** | char == UTF-16 (ký tự BMP) |
| `"\uD835\uDD18 helo"` (astral `𝔘` = 2 code unit) | 7 | **3** | **UTF-16 code unit** (char index của `h` = 2) |

⇒ **Kết luận:** Windows trả **UTF-16 code unit**. Adapter phải convert:
`char_index = text[..utf16_index].chars().count()` (hoặc map 1 lần bằng `Vec<usize>` cho cả câu).
Với tiếng Việt dựng sẵn (NFC, toàn BMP) thì hai đơn vị **trùng nhau** — nghĩa là bug này **ẩn** trên
corpus thường và chỉ lộ khi có emoji/ký tự astral ⇒ **bắt buộc có test riêng** (§7).

### 2.5 Số đo thực nghiệm (để đặt ngân sách, không phải benchmark chính thức)

Môi trường: Windows 11 (OS build 26300, PowerShell build 26100) · checker `en-US` (máy **không** có `vi-VN`) · probe C# qua COM
(PowerShell 5.1), **không** phải code sản phẩm, chạy trong `%TEMP%`, không commit.

| Phép đo | Kết quả |
|---|---|
| `CoCreateInstance` + `CreateSpellChecker("en-US")` (nguội) | **9,49 ms** |
| `Check()` lần đầu sau khi tạo (nguội) | **2,68 ms** |
| `Check()` warm, câu 87 code unit | **0,057 ms** (200 lần) |
| `Check()` warm, câu tiếng Việt có dấu 51 code unit | **0,055 ms** (100 lần) |
| `Suggest("truong")` warm (cùng từ, cache) | **3,07–3,11 ms** (3 lô × 50) |
| `Suggest("tu<i>")` warm (từ khác nhau) | **6,91 ms** |
| `IsSupported("vi-VN")` / `("vi")` | `S_OK`, **FALSE** |
| `IsSupported("en-US")` / `("en-GB")` | `S_OK`, **TRUE** |
| `CreateSpellChecker("vi-VN")` | **`0x80070057` = E_INVALIDARG** |
| `Check(câu tiếng Việt không dấu)` bằng checker `en-US` | **3 lỗi / 8 từ** (báo sai) |
| `Suggest("truong")` trả về | `["Truong", "truing"]` (không có gợi ý tiếng Việt) |

**Ba kết luận có hệ quả trực tiếp:**

1. **Không** dùng checker `en-US` cho text tiếng Việt (3/8 từ bị báo sai ⇒ FPR khủng khiếp). Cổng
   `IsSupported("vi-VN")` là **bắt buộc**, không phải tối ưu.
2. `Check` đủ rẻ để gọi **1 lần / câu** (0,06 ms) ⇒ dùng cho `contains` ở mức *câu* rồi tra theo
   offset, **tốt hơn** gọi `Check` cho từng từ.
3. `Suggest` 3–7 ms ⇒ **không bao giờ** để nó chạy tuyến tính theo số từ OOV trong đường check.

### 2.6 Rủi ro riêng của Windows

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| W1 | Provider out-of-proc → block không timeout được | **Cao** | worker thread thường trực + `recv_timeout(50ms)` + fail-open (D36) |
| W2 | `vi-VN` không có (mặc định) | **Cao** (thực tế 100 %) | capability probe 1 lần/phiên ⇒ `NoDictionary`; **không** thử `en-US` thay thế |
| W3 | Offset UTF-16 vs char | Trung bình | convert tường minh + test astral (§7) |
| W4 | Rò bộ nhớ `LPWSTR`/`IEnumString` | Trung bình | `CoTaskMemFree` từng `PWSTR`, `Release` từng interface; review theo checklist COM |
| W5 | **Lint "runtime symbol"** (bài học `dict/src/mmap.rs`, Fassist-104) | Trung bình | **Không** tự khai `extern "system" { fn CoCreateInstance(...) }` — dùng crate `windows` (nó tự `link!` đúng cách). Nếu buộc khai tay: chỉ khai symbol **không** trùng runtime Rust; tuyệt đối không khai `ole32`/`CoInitialize` bằng tay khi crate đã có |
| W6 | Feature `Win32_Globalization` làm tăng binary | Thấp | đo Δsize; nếu > 200 KB thì cân nhắc `windows-sys` + tự khai vtable (đánh đổi W5) |
| W7 | AV/privacy: provider có thể là **tiến trình bên thứ ba** | Thấp–TB | Chỉ dùng checker do OS đăng ký (không cấu hình provider riêng); ghi rõ trong `docs/` AV: text **rời khỏi** tiến trình TextVN khi provider out-of-proc — **phải nói trong privacy notice** (⚠️ chưa xác minh được provider mặc định của MS có gửi mạng hay không) |

### 2.7 Code sketch (Rust — **sketch trong doc**, không phải code sản phẩm)

**Đường chính (khuyến nghị):**

```rust
// Đặt ở ffi/src/os_spell/win.rs (hoặc adapters/windows-*/src/os_spell.rs) — KHÔNG đặt trong `assist`.
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::time::Duration;
use windows::core::{Interface, HSTRING, PWSTR};
use windows::Win32::Globalization::{
    ISpellChecker, ISpellCheckerFactory, IEnumSpellingError, SpellCheckerFactory,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};

pub struct WinSpell { tx: SyncSender<Job> }   // 1 worker thread thường trực / tiến trình

enum Job {
    Contains { word: String, reply: SyncSender<bool> },
    Suggest  { word: String, limit: usize, reply: SyncSender<Vec<String>> },
}

impl WinSpell {
    /// `None` nếu máy không có spell checker cho `vi-VN` (trường hợp MẶC ĐỊNH trên Windows).
    pub fn try_new(lang_tag: &str) -> Option<Self> {
        let (tx, rx) = sync_channel::<Job>(1);
        let (ready_tx, ready_rx) = sync_channel::<bool>(1);
        let tag = lang_tag.to_owned();
        std::thread::Builder::new()
            .name("textvn-spell".into())
            .spawn(move || unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                let checker = create_checker(&tag);          // xem dưới
                ready_tx.send(checker.is_some()).ok();
                let Some(checker) = checker else { CoUninitialize(); return };
                // Vòng đời: mọi lời gọi COM ở ĐÚNG thread này (W-apartment).
                while let Ok(job) = rx.recv() { handle(&checker, job); }
                CoUninitialize();
            })
            .ok()?;
        match ready_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(true) => Some(Self { tx }),
            _ => None,                                        // probe quá hạn/chậm → fallback
        }
    }

    fn contains(&self, word: &str) -> bool {                  // fail-open = true
        let (tx, rx) = sync_channel(1);
        if self.tx.send(Job::Contains { word: word.into(), reply: tx }).is_err() { return true; }
        rx.recv_timeout(Duration::from_millis(50)).unwrap_or(true)
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let (tx, rx) = sync_channel(1);
        if self.tx.send(Job::Suggest { word: word.into(), limit, reply: tx }).is_err() { return Vec::new(); }
        rx.recv_timeout(Duration::from_millis(50)).unwrap_or_default()   // hết giờ → rỗng, KHÔNG lỗi
    }
}

unsafe fn create_checker(tag: &str) -> Option<ISpellChecker> {
    let factory: ISpellCheckerFactory =
        CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_INPROC_SERVER).ok()?;
    let tag = HSTRING::from(tag);
    if !factory.IsSupported(&tag).ok()?.as_bool() { return None; }   // cổng D33
    factory.CreateSpellChecker(&tag).ok()
}

unsafe fn handle(checker: &ISpellChecker, job: Job) {
    match job {
        Job::Contains { word, reply } => {
            // Check() một từ: E_INVALIDARG nếu chuỗi rỗng ⇒ chặn trước.
            if word.is_empty() { reply.send(true).ok(); return; }
            let ok = match checker.Check(&HSTRING::from(word)) {
                Ok(e) => first_error(&e).is_none(),                  // không lỗi = đúng
                Err(_) => true,                                      // fail-open
            };
            reply.send(ok).ok();
        }
        Job::Suggest { word, limit, reply } => {
            let mut out = Vec::new();
            if let Ok(en) = checker.Suggest(&HSTRING::from(word)) {
                let mut buf = [PWSTR::null(); 8];
                // Next(&mut [PWSTR], Option<*mut u32>) — lặp tới S_FALSE
                while en.Next(&mut buf, None).is_ok() && out.len() < limit {
                    for p in buf.iter() {
                        if p.is_null() { continue; }
                        out.push(p.to_string().unwrap_or_default());
                        CoTaskMemFree(Some(p.as_ptr() as *const _));   // W4
                    }
                    buf = [PWSTR::null(); 8];
                }
            }
            out.truncate(limit);
            reply.send(out).ok();
        }
    }
}
```

```rust
// Adapter — implement trait THẬT của repo (assist::dict::Dictionary).
impl textvn_assist::dict::Dictionary for WinSpell {
    fn contains(&self, word: &str) -> bool { self.contains(word) }
    fn suggest(&self, word: &str, limit: usize) -> Vec<String> { self.suggest(word, limit) }
}
```

> **Ghi chú sketch:** các chỗ dễ sai khi code thật: (a) `IEnumString::Next` cần **mảng `PWSTR`** và
> trả `S_FALSE` khi hết — vòng lặp phải dừng theo `got`, không theo `celt`; (b) `Check("")` trả
> `E_INVALIDARG`; (c) mỗi `PWSTR` phải `CoTaskMemFree` **riêng**; (d) `ISpellChecker` **không** được
> gửi sang thread khác (§2.3).

**Phương án dự phòng — COM thủ công (chỉ khi buộc bỏ crate `windows`)**

Không khuyến nghị. Nếu phải làm: tự khai `#[repr(C)] struct ISpellCheckerFactory_Vtbl { base:
IUnknown_Vtbl, SupportedLanguages, IsSupported, CreateSpellChecker }` + `extern "system"` cho
`CoCreateInstance`/`CoInitializeEx` + tự tạo GUID. Rủi ro: sai thứ tự vtable = **UB im lặng**
(đúng loại lỗi đã làm crash probe C# ở §2.5 khi khai sai `IEnumString`), dễ dính lint "runtime
symbol" (W5/Fassist-104), và mất kiểm tra `cfg(feature)` của crate. **Chỉ dùng nếu** Δsize của
`Win32_Globalization` vượt ngân sách **và** có test ABI bằng máy thật 3 phiên bản Windows.

---

## 3. macOS — `NSSpellChecker` (AppKit)

### 3.1 API đã verify (Apple Developer Documentation, JSON 2026-10-21)

| Thành phần | Chữ ký | Ghi chú |
|---|---|---|
| `NSSpellChecker.shared` | `class var shared: NSSpellChecker` | "một instance / app" |
| `availableLanguages` | `var availableLanguages: [String]` | danh sách ngôn ngữ có từ điển |
| `checkSpelling(of:startingAt:language:wrap:inSpellDocumentWithTag:wordCount:)` | `-> NSRange` | trả **range của từ sai đầu tiên**; lặp bằng `startingAt = NSMaxRange(prev)`; `length == 0` = hết |
| `checkSpelling(of:startingAt:)` | `-> NSRange` | bản tiện dụng, không tag |
| `guesses(forWordRange:in:language:inSpellDocumentWithTag:)` | `-> [String]?` | gợi ý cho **một** word range |
| `correction(forWordRange:in:language:inSpellDocumentWithTag:)` | `-> String?` | "single proposed correction if a word is mis-spelled" |
| `checkGrammar(of:startingAt:language:wrap:inSpellDocumentWithTag:details:)` | `-> NSRange` | **có** API ngữ pháp (khác P4-4 §1 ghi "chưa rõ") — nhưng **v1 không dùng** (F68/D-finding: ngữ pháp luôn của ta) |
| `uniqueSpellDocumentTag()` / `closeSpellDocument(withTag:)` | `-> Int` / `Void` | cấp/giải phóng "spell document" (chứa ignored words) |
| `ignoreWord(_:inSpellDocumentWithTag:)`, `hasLearnedWord(_:)` | | tôn trọng từ user đã thêm |
| `language(forWordRange:in:orthography:)` | `-> String?` | nhận ngôn ngữ của 1 range |

⚠️ **`hasCorrection` KHÔNG có** trong danh sách API công khai của `NSSpellChecker` (đối chiếu
`topicSections` của trang lớp, 2026-10-21). Dùng `correction(forWordRange:…) != nil`.

### 3.2 Ràng buộc offset

`NSRange` của AppKit đếm theo **UTF-16 code unit** (`NSString.length`). `assist` dùng **char
Unicode**. Với tiếng Việt NFC thì trùng, nhưng **phải** convert bằng `String.utf16`/`NSString`
chứ không dùng `String.count`:

```swift
let ns = text as NSString
var loc = 0
while loc < ns.length {
    let r = checker.checkSpelling(of: text, startingAt: loc, language: "vi",
                                  wrap: false, inSpellDocumentWithTag: tag, wordCount: nil)
    if r.length == 0 { break }
    // r.location/r.length = UTF-16 → đổi sang char cho `assist`
    let startChar = ns.substring(to: r.location).count
    let lenChar   = ns.substring(with: r).count
    …
    loc = NSMaxRange(r)
}
```

### 3.3 Sketch Swift (adapter trong `adapters/macos-imk`, KHÔNG trong `assist`)

```swift
import AppKit

/// Cầu nối C-ABI: Rust gọi 2 callback này; Swift giữ NSSpellChecker.
/// (Đặt trong file Swift của IMK; export bằng @_cdecl để Rust lấy function pointer.)
final class MacSpellProvider {
    private let checker = NSSpellChecker.shared
    private let lang = "vi"
    private var tag: Int = 0

    func isAvailable() -> Bool {
        checker.availableLanguages.contains { $0.lowercased().hasPrefix("vi") }
    }

    func contains(_ word: String) -> Bool {
        let ns = word as NSString
        let r = checker.checkSpelling(of: word, startingAt: 0, language: lang,
                                      wrap: false, inSpellDocumentWithTag: tag, wordCount: nil)
        return r.length == 0                     // không tìm thấy từ sai ⇒ từ đúng
    }

    func suggest(_ word: String, limit: Int) -> [String] {
        let ns = word as NSString
        let all = NSRange(location: 0, length: ns.length)
        let g = checker.guesses(forWordRange: all, in: word, language: lang,
                                inSpellDocumentWithTag: tag) ?? []
        return Array(g.prefix(limit))
    }
}
```

> **Kiến trúc đúng theo D23/D32:** Swift **sở hữu** `NSSpellChecker` (AppKit), Rust **không** gọi
> AppKit. Hai chiều đều được, nhưng chọn **một**:
> **(a)** Swift gọi `ime_assist_check_v1(text, provider)` với `provider` = bảng con trỏ hàm C
> (`contains`/`suggest`/`ctx`) — Rust không cần biết macOS; hoặc
> **(b)** Rust gọi ra `ime_os_spell_*` do Swift export — Rust phải biết tên symbol ⇒ bẩn hơn.
> **Khuyến nghị (a)**: giữ `assist` + `ffi` hoàn toàn portable.

### 3.4 UI: IMK **input server** vẽ candidate list — không phải app

- Apple: `IMKInputController` là lớp cơ sở cho input controller; `IMKCandidates` *"presents
  candidates to users"* và *"is optional"*. Tức là **tiến trình input method** vẽ UI ứng viên của
  nó, còn app client chỉ nhận text qua `IMKTextInput`.
- ⇒ **Không** thể kỳ vọng vẽ underline/overlay trong app như Windows (TSF `ITfContextView`).
  Đúng như **D30**: macOS = **toast/panel** (cửa sổ phụ của IMK), không inline.
- Hệ quả UX: gợi ý spelling trên macOS chỉ hiện ở toast/panel ⇒ **không** dùng để "gạch chân" như
  Windows. Đây là khác biệt **được chấp nhận** ở v1, phải ghi trong `ui-spec`.

### 3.5 L0 dùng OS (D35) — `NLLanguageRecognizer`

- Apple: `NLLanguageRecognizer` (macOS **10.14+**), `dominantLanguage(for:)` / `processString(_:)` +
  `dominantLanguage`, `languageHypotheses(withMaximum:)`, `languageHints`, `languageConstraints`.
- Apple **cảnh báo nguyên văn**: *"Language identification is most reliable for longer passages of
  text. For very short inputs of only a few words, accuracy is lower…"* và *"Don't use an instance
  … from more than one thread simultaneously."*
- ⇒ **D35 chỉ dùng được cho câu dài**; cho **fragment 2–3 từ** (đúng cảnh IME) **vẫn phải** giữ
  heuristic `rules::language::is_vietnamese` làm chốt. Đề xuất: `NLLanguageRecognizer` **chỉ** để
  *tăng* độ tin cậy khi `is_vietnamese` lưỡng lự (tỷ lệ từ có dấu 0,4–0,6), không thay thế.

### 3.6 Rủi ro riêng của macOS

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| M1 | `availableLanguages` **không** có `vi` trên máy chưa bật từ điển Việt | **Cao** | probe 1 lần ⇒ `NoDictionary`; hướng dẫn user bật trong System Settings → Keyboard → Text → Spelling (ghi ở docs, **không** tự bật) |
| M2 | AppKit **thread-affinity**: gọi `NSSpellChecker` ngoài main thread | Trung bình | chỉ gọi trên main thread (IMK chạy trên main run loop) ⇒ không cần timeout thread như Windows; **nhưng** vẫn phải có "deadline mềm" để không sập UI nếu spell server chậm (⚠️ chưa đo được độ trễ macOS) |
| M3 | `NSSpellChecker` dùng chung `spellDocumentWithTag` với app khác | Thấp | dùng `uniqueSpellDocumentTag()` **riêng** cho TextVN; `closeSpellDocument` khi kết thúc phiên |
| M4 | User đã `learnWord` trong app khác | Thấp | `hasLearnedWord`/`ignoreWord` tôn trọng ⇒ **giảm** FPR (điểm cộng của OS-first) |
| M5 | Ngữ pháp OS khác 3 nền tảng | Trung bình | **không** dùng `checkGrammar` ở v1 (giữ F68: ngữ pháp luôn của ta) |

---

## 4. Linux — `libspelling` (GNOME) **và** `enchant-2`

### 4.1 Sự thật quan trọng: `check_ranges` **chưa có** trong bản phát hành

Đã kiểm tra header **theo từng tag** (raw.githubusercontent.com/GNOME/libspelling):

| Tag | `get_default` | `check_word` | `list_corrections` | **`check_ranges`** |
|---|---|---|---|---|
| `main` (1.0.alpha) | ✅ | ✅ | ✅ | ✅ `DexFuture *(checker, GBytes *text, const SpellingRange *ranges, gsize n_ranges)` |
| **0.4.10** | ✅ | ✅ | ✅ | ❌ **không có** |
| **0.4.8** | ✅ | ✅ | ✅ | ❌ **không có** |

Bản distro đang có (Repology, 2026-10-21): Debian 13 = **0.4.8**, Debian 14/unstable = **0.4.10**,
Ubuntu 24.04 = **0.2.0**, Ubuntu 25.10/26.04 = 0.4.7–0.4.9, Fedora 43 = 0.4.9, Arch/openSUSE
Tumbleweed = 0.4.10.

⇒ **Hệ quả (sửa giả định P4-4 §1):** với bản phát hành, **không** dùng được API trả offset char
cho cả câu. Nhưng **không cần**: `assist` đã tự tokenize (`assist/src/tokenize.rs`) và trait
`Dictionary` chỉ cần **theo từ** ⇒ `check_word` + `list_corrections` **khớp hoàn hảo**:

| Trait | libspelling 0.4.x | Ghi chú |
|---|---|---|
| `contains(word)` | `spelling_checker_check_word(checker, word, -1) -> gboolean` | **đồng bộ**, `word_len` tính bằng **byte**, `-1` = NUL-terminated UTF-8 |
| `suggest(word, limit)` | `spelling_checker_list_corrections(checker, word) -> char **` | mảng **NULL-terminated**, `g_strfreev`; **cắt** theo `limit` |

`spelling_checker_check_ranges` + `SpellingMistake { offset, length (Unicode chars),
byte_offset, byte_length }` là **đường nâng cấp tương lai** khi libspelling 1.0 vào distro — lúc đó
có thể check cả câu 1 lời gọi và offset **đã đúng char** (khớp D27). Ghi thành task riêng, **không**
chặn v1.

### 4.2 libspelling — ràng buộc & license

- License: **LGPL-2.1-or-later** (README + trang docs ghi `LGPL-2.1-or-later`). ⇒ **link động**,
  không static; phải khai trong `docs/compliance` + `LICENSES/` (**cargo-deny không thấy** vì đây
  là thư viện C, không phải crate).
- Phụ thuộc **GTK 4 + GtkSource 5 + GLib/GObject/Gio** (trang docs "Dependencies"). ⇒ **vấn đề
  thật** với engine IBus/Fcitx5 (thường chỉ GLib): kéo GTK4 vào tiến trình engine = tăng RSS,
  tăng bề mặt lỗi, có thể xung đột main loop.
- `spelling_init()` phải gọi trên **main thread** trước mọi API khác.
- `checker_get_default()` dùng provider + ngôn ngữ **mặc định của hệ thống**; muốn `vi` phải
  `spelling_checker_set_language(checker, "vi")` hoặc `spelling_checker_new(provider, "vi")`.
- Version API: docs hiện ghi **Library Version 1.0.alpha** ⇒ API còn **chưa ổn định** → phải
  `#if SPELLING_CHECK_VERSION(...)` hoặc `dlsym` để không vỡ khi lên 1.0.

### 4.3 `enchant-2` — phương án **nhẹ hơn** (khuyến nghị cho IBus/Fcitx5)

libspelling **chỉ là wrapper của enchant-2** (README: *"Currently, libspelling wraps `enchant-2`
only"*). Dùng enchant trực tiếp: bỏ được GTK4/GtkSource, chỉ cần **GLib**.

API thật (header `lib/enchant.h`, repo `rrthomas/enchant`):

```c
EnchantBroker *enchant_broker_init (void);
int            enchant_broker_dict_exists (EnchantBroker *broker, const char *const tag);
EnchantDict   *enchant_broker_request_dict (EnchantBroker *broker, const char *const tag);
void           enchant_broker_free_dict (EnchantBroker *broker, EnchantDict *dict);
int            enchant_dict_check (EnchantDict *dict, const char *const word, ssize_t len);
char         **enchant_dict_suggest (EnchantDict *dict, const char *const word, ssize_t len, size_t *out_n_suggs);
void           enchant_dict_free_string_list (EnchantDict *dict, char **string_list);
```

- License: **LGPL-2.1-or-later** + **exception** cho phép nạp backend không-LGPL (header nói rõ).
- Toàn bộ I/O là **UTF-8**; tag dạng `xx_YY` (`vi`, `vi_VN`).
- **Trong tiến trình** (hunspell/nuspell là lib) ⇒ độ trễ thấp, **không** cần timeout thread như
  Windows (nhưng vẫn phải có deadline mềm vì hunspell có thể chậm với từ rất dài).

| | libspelling 0.4.x | enchant-2 trực tiếp |
|---|---|---|
| API cho trait | `check_word` + `list_corrections` | `enchant_dict_check` + `enchant_dict_suggest` |
| Phụ thuộc | GTK4 + GtkSource5 + GLib | **chỉ GLib** |
| Offset char cho cả câu | ❌ (chỉ 1.0) | ❌ (ta tự tokenize — đủ) |
| API ổn định | ⚠️ 1.0.alpha | ✅ ổn định từ 2.x |
| License | LGPL-2.1+ | LGPL-2.1+ (có exception) |
| Khuyến nghị | dùng khi app đã có GTK4 (GNOME Text Editor) | **mặc định cho engine IME** |

### 4.4 Từ điển `vi` trên Linux — **user cài**, ta không ship

- Repology 2026-10-21: `hunspell-vi` có ở **Fedora 39–44/rawhide** (license gói `GPL-2.0-only`),
  AlmaLinux 8/9, ALT, Amazon Linux 1…; **không thấy** entry Debian/Ubuntu.
  ⇒ Khẳng định cũ "gói sẵn của Debian/Ubuntu" (P4-4 §1) **sai**; phải sửa.
- License `GPL-2.0-only` **khớp** kết luận ADR-014 (không "or later" ⇒ không tương thích GPL-3.0)
  ⇒ **ta không ship**, chỉ dùng từ điển **user đã cài trong hệ thống**. Đây chính là lý do OS-first
  có giá trị về license.

### 4.5 Code sketch (C — sketch trong doc)

```c
/* adapters/linux-*/src/os_spell.c — biên C, KHÔNG trong crate `assist` */
#include <glib.h>
#include <enchant.h>
#include <string.h>

static EnchantBroker *g_broker;
static EnchantDict   *g_dict;

/* gọi 1 lần lúc khởi tạo engine */
int textvn_os_spell_init(void) {
    g_broker = enchant_broker_init();
    if (!g_broker) return 0;
    /* thử "vi" trước, rồi "vi_VN" — enchant dùng dạng xx_YY */
    const char *tag = enchant_broker_dict_exists(g_broker, "vi")      ? "vi"
                    : enchant_broker_dict_exists(g_broker, "vi_VN")   ? "vi_VN" : NULL;
    if (!tag) return 0;                          /* ⇒ caller dùng NoDictionary */
    g_dict = enchant_broker_request_dict(g_broker, tag);
    return g_dict != NULL;
}

/* Dictionary::contains */
int textvn_os_spell_contains(const char *word, gssize len) {
    if (!g_dict) return 1;                        /* fail-open */
    return enchant_dict_check(g_dict, word, len) == 0 ? 1 : 0;   /* 0 = đúng */
}

/* Dictionary::suggest — trả số gợi ý đã ghi vào out (NUL-separated), 0 nếu không có */
guint textvn_os_spell_suggest(const char *word, gssize len, char *out, gsize out_len, guint limit) {
    if (!g_dict || !out || out_len == 0) return 0;
    char **s = enchant_dict_suggest(g_dict, word, len, NULL);
    if (!s) return 0;
    guint n = 0; gsize used = 0;
    for (guint i = 0; s[i] && n < limit; i++) {
        gsize l = strlen(s[i]);
        if (used + l + 1 >= out_len) break;
        memcpy(out + used, s[i], l + 1); used += l + 1; n++;
    }
    enchant_dict_free_string_list(g_dict, s);      /* bắt buộc */
    return n;
}
```

```c
/* Bản libspelling (khi muốn đi đúng D33) */
#include <libspelling.h>
spelling_init();                                            /* main thread, 1 lần */
SpellingChecker *c = spelling_checker_get_default();
spelling_checker_set_language(c, "vi");
gboolean ok = spelling_checker_check_word(c, word, -1);      /* -1: NUL-terminated, len = BYTE */
char **corr = spelling_checker_list_corrections(c, word);    /* NULL-terminated */
g_strfreev(corr);
```

> **Không** gọi `spelling_checker_check_ranges` trong code v1 — symbol **không tồn tại** ở 0.4.x ⇒
> link fail trên Debian 13/Ubuntu. Nếu muốn "future-proof": `dlsym(RTLD_DEFAULT,
> "spelling_checker_check_ranges")` + fallback `check_word`.

### 4.6 Rủi ro riêng của Linux

| # | Rủi ro | Mức | Giảm thiểu |
|---|---|---|---|
| L1 | libspelling 0.4.x **không có** `check_ranges` | **Cao** (đã xảy ra) | dùng `check_word`; `check_ranges` = task v2 (khi 1.0) |
| L2 | libspelling kéo **GTK4+GtkSource5** vào engine IBus/Fcitx5 | **Cao** | **enchant-2 trực tiếp** làm mặc định; libspelling = tùy chọn |
| L3 | API libspelling **1.0.alpha** (churn) | Trung bình | `dlsym` + `SPELLING_CHECK_VERSION`; không gọi API mới |
| L4 | User **không** cài `hunspell-vi` (đặc biệt Debian/Ubuntu) | **Cao** | `dict_exists` ⇒ `NoDictionary`; docs hướng dẫn cài `hunspell-vi`/`hunspell-vi-VN` (tuỳ distro) — **không** tự cài |
| L5 | enchant nạp **backend ngoài** (aspell/hspell…) có thể chậm | Thấp | deadline mềm; `enchant_broker_set_ordering` chỉ `hunspell,nuspell,aspell` |
| L6 | Thiếu `libenchant-2.so` trên máy tối giản | Trung bình | link **động** + `dlopen` (không bắt buộc link cứng) ⇒ thiếu lib = fallback, không crash |
| L7 | License LGPL không đi qua cargo-deny | Trung bình | khai `docs/compliance/assist-os-provider.md` + `LICENSES/`; dynamic link only |

---

## 5. Một trait — ba adapter

### 5.1 Bảng map

| OS | Nguồn | `contains(word)` | `suggest(word, limit)` | Khi thiếu `vi` |
|---|---|---|---|---|
| **Windows** | `ISpellChecker` (`vi-VN`) | `Check(word)` → không có `ISpellingError` = đúng (hoặc `Check(cả câu)` 1 lần rồi tra offset) | `Suggest(word)` → `IEnumString`, cắt `limit` | `IsSupported` = false ⇒ **`NoDictionary`** |
| **macOS** | `NSSpellChecker` | `checkSpelling(of: word, startingAt: 0, …).length == 0` | `guesses(forWordRange: full, in: word, …)` | `availableLanguages` không có `vi` ⇒ **`NoDictionary`** |
| **Linux** | `enchant-2` / `libspelling` | `enchant_dict_check(...) == 0` / `spelling_checker_check_word(...)` | `enchant_dict_suggest` / `spelling_checker_list_corrections` | `dict_exists` false ⇒ **`NoDictionary`** |
| *(dự phòng v2)* | FST của ta (`textvn-dict`) | tra FST | gợi ý theo edit-distance | — |

**Quy tắc chung (bắt buộc):**

1. **Cổng capability 1 lần / phiên** (`IsSupported` / `availableLanguages` / `dict_exists`), cache
   kết quả; **không** probe lại mỗi câu (Windows probe nguội 9,5 ms — đo được).
2. **Không có `vi` ⇒ `NoDictionary`**, tuyệt đối **không** mượn checker ngôn ngữ khác (đo được:
   checker `en-US` báo sai 3/8 từ tiếng Việt).
3. `contains` và `suggest` **độc lập**: `contains` phải rẻ; `suggest` phải có ngân sách.
4. Mọi lỗi/timeout ⇒ **fail-open** (`contains = true`, `suggest = []`) — thà im lặng còn hơn báo sai.
5. **Không log** `word`/`text` ở bất kỳ nhánh nào (S2), kể cả khi lỗi COM/HRESULT. Chỉ log **mã**
   (`hr`, `provider kind`, `reason`) và **đếm**.

### 5.2 Hai kiểu cắm (chọn theo OS)

```
(A) Trait Rust (Windows, Linux-in-process):
    assist  ←  adapter (Rust/C)  ←  OS API
    Ưu: đơn giản, cùng tiến trình, không cần ABI mới.

(B) C-ABI callback (macOS Swift, hoặc Linux C tách rời):
    assist::check_with(text, &BridgeDict)  →  BridgeDict gọi fn pointer C do Swift/C cung cấp
    typedef struct ime_os_spell_v1 {
        uint32_t abi_version;                      /* = 1 */
        int32_t (*contains)(void *ctx, const char *word, uint32_t byte_len);  /* 1/0/-1(không biết) */
        uint32_t (*suggest)(void *ctx, const char *word, uint32_t byte_len,
                            char *out, uint32_t out_len, uint32_t limit);
        void *ctx;
    } ime_os_spell_v1;
    Ưu: `assist` vẫn 0-dep; biên rõ; test được bằng fake provider.
```

- `BridgeDict` (Rust, ở `ffi` hoặc crate adapter) implement `Dictionary`, giữ **`Vec<u8>` scratch**
  để không cấp phát mỗi từ, convert UTF-8 ↔ UTF-16 chỉ ở biên.
- **Không** đưa `ime_os_spell_v1` vào crate `assist` (giữ 0-dep + portable, D32).

### 5.3 Ngân sách `suggest` (bắt buộc, vì trait là đồng bộ)

```rust
/// Bọc adapter OS: sau `budget` tính từ lần check đầu, `suggest` trả rỗng (fail-open).
pub struct Budgeted<D: Dictionary> { inner: D, deadline: std::time::Instant, budget: Duration }
impl<D: Dictionary> Dictionary for Budgeted<D> {
    fn contains(&self, w: &str) -> bool { self.inner.contains(w) }
    fn suggest(&self, w: &str, limit: usize) -> Vec<String> {
        if std::time::Instant::now() >= self.deadline { return Vec::new(); }
        self.inner.suggest(w, limit)
    }
}
```

- Ngân sách đề xuất: **tổng 50 ms / câu** (D36). `contains` được ưu tiên (rẻ); `suggest` bị cắt
  **trước**.
- Kèm **cap cứng** số từ được `suggest`: `MAX_SUGGEST_WORDS = 3` / câu (khớp `MAX_SUGGESTIONS = 3`
  đang có trong `rules/spelling.rs`). 3 × 6,9 ms ≈ 21 ms — vẫn trong 50 ms.

---

## 6. Bảng so sánh cuối

| Tiêu chí | **Windows** (Spell Checking API) | **macOS** (`NSSpellChecker`) | **Linux** (`libspelling`) | **Linux** (`enchant-2`) |
|---|---|---|---|---|
| API chính | `ISpellCheckerFactory` → `ISpellChecker::{Check,Suggest}` | `checkSpelling(of:startingAt:language:wrap:inSpellDocumentWithTag:wordCount:)`, `guesses(...)` | `spelling_checker_check_word` / `list_corrections` (**không** `check_ranges` ở 0.4.x) | `enchant_dict_check` / `enchant_dict_suggest` |
| Ngôn ngữ hỗ trợ | theo language pack đã cài; **`vi-VN` mặc định = KHÔNG** (đo được) | theo bộ từ điển hệ thống; `vi` **chưa xác minh** trên máy | bất kỳ dict enchant thấy; `vi` = **user cài** | như libspelling |
| Offset trả về | **UTF-16 code unit** (đo được; doc không nói) | **UTF-16** (`NSRange`) | 0.4.x: theo **từ**; 1.0: **char Unicode** | theo **từ** (ta tự tokenize) |
| License | API của OS (không có license riêng cho ta) | AppKit (OS) | **LGPL-2.1-or-later** | **LGPL-2.1-or-later** + exception |
| Phụ thuộc | crate `windows` + feature `Win32_Globalization` (+`Win32_System_Com`) | AppKit (có sẵn trong IMK) | GTK4 + GtkSource5 + GLib | **GLib** |
| Đồng bộ / async | **đồng bộ nhưng có thể out-of-proc** → cần thread+timeout | đồng bộ, main thread | `check_word` đồng bộ; `check_ranges` = `DexFuture` | đồng bộ, in-process |
| Độ trễ đo được | create 9,5 ms (nguội) · Check 0,057 ms/câu 87 unit · **Suggest 3,1–6,9 ms** | ⚠️ **chưa đo** | ⚠️ **chưa đo** | ⚠️ **chưa đo** |
| Rủi ro bảo mật | provider **out-of-proc, restricted context**; RPC timeout; có thể là code bên thứ ba | thấp (in-process, OS) | thấp (LGPL, in-process) | thấp |
| AV / privacy | text **rời tiến trình** (out-of-proc) — phải nói trong privacy notice; **không** phải tiến trình TextVN | text ở lại tiến trình app? (NSSpellChecker in-process + spell server của OS — ⚠️ chưa xác minh) | in-process | in-process |
| Rủi ro lớn nhất | `vi-VN` không có + block | `vi` không có + không vẽ được inline | **API `check_ranges` chưa tồn tại** + GTK4 | thiếu `vi` dict (Debian/Ubuntu) |
| Khuyến nghị v1 | **có**, sau cổng `IsSupported`, chỉ để *tăng* gợi ý | **có** (khi `availableLanguages` có `vi`) | tùy chọn (app có GTK4) | **mặc định cho engine** |

---

## 7. Khuyến nghị + thứ tự triển khai + tiêu chí nghiệm thu

### 7.1 Thứ tự (theo *giá trị / rủi ro / công sức*)

| Bước | Việc | Vì sao thứ tự này | Công sức | Rủi ro |
|---|---|---|---|---|
| **P0** | `Budgeted<D>` + `BridgeDict` (Rust, trong `ffi`) + test bằng **fake provider** | Không cần OS nào; khoá được ngân sách 50 ms & fail-open **trước** khi có provider thật | 1–2 ngày | Thấp |
| **P0** | Cổng capability + `NoDictionary` mặc định + **counter** (không log text) | Trên Windows (đa số user) kết quả **y hệt** hôm nay ⇒ không hồi quy FPR | 0,5 ngày | Thấp |
| **P1** | **Linux/enchant-2** (C, `dlopen`) | Không cần GTK4; API ổn định; `vi` khả dụng cao nhất trong 3 OS (Fedora) | 2–3 ngày | Trung bình |
| **P2** | **Windows** (`windows` + `Win32_Globalization`) | Đa số user **không** có `vi-VN` ⇒ giá trị thấp, nhưng code sẵn để khi user cài language pack; rủi ro block ⇒ cần P0 xong | 2–3 ngày | **Cao** (COM/thread) |
| **P3** | **macOS** (`NSSpellChecker` trong Swift IMK) | Cần đo trước; UI chỉ toast; `vi` chưa xác minh | 2–3 ngày | Trung bình |
| **P4** | libspelling (tùy chọn) khi **1.0** vào distro | chỉ để lấy `check_ranges` (char offset) — hiện **không** cần | 1–2 ngày | Thấp |
| **P5** | L0 bằng OS (D35) — `NLLanguageRecognizer` / `GetUserDefaultLocaleName` | Chỉ *bổ trợ* heuristic; làm sau cùng vì không chặn gì | 1–2 ngày | Thấp |

### 7.2 Tiêu chí nghiệm thu (đo được, không cảm tính)

| ID | Tiêu chí | Cách đo | Ngưỡng |
|---|---|---|---|
| AC-1 | **Không hồi quy** khi OS không có `vi` | chạy corpus P4-8 trên máy Windows sạch | issue set **giống hệt** bản `NoDictionary` (diff = 0) |
| AC-2 | Capability probe rẻ, 1 lần/phiên | đếm số lần gọi `IsSupported` trong 1 phiên 1 000 câu | **= 1** |
| AC-3 | Độ trễ end-to-end không vượt budget | p99 `ime_assist_check_v1` trên câu 256 char | **≤ 10 ms**; hard fail nếu > **50 ms** |
| AC-4 | `suggest` không phá budget | câu có 10 từ OOV | tổng thời gian check **≤ 50 ms**, số từ được suggest **≤ 3** |
| AC-5 | Fail-open thật | fake provider ngủ 500 ms | check trả về **trong ≤ 60 ms**, **0** issue spelling, **0** panic |
| AC-6 | Offset đúng khi có astral char | test `"𝔘 helo"` với provider Windows giả lập + unit test convert | `span.start == 2` (char), không phải 3 |
| AC-7 | Không log nội dung (S2) | grep CI: không `log`/`println`/`format!` chứa `text`/`word` trong adapter; test runtime bắt output | **0** dòng chứa text người dùng |
| AC-8 | Không thêm tiến trình/không network | Process Monitor/`netstat` khi bật provider; đếm process | **0** process mới do TextVN; **0** kết nối mạng (**trừ** provider OS out-of-proc — ghi rõ ngoại lệ) |
| AC-9 | Không tăng footprint | `size` binary + RSS trước/sau | Δbinary **≤ 200 KB**; ΔRSS **≤ 1 MB** |
| AC-10 | Linux: chạy được trên bản phát hành | Debian 13 (libspelling 0.4.8 **hoặc** enchant-2), Ubuntu 24.04 | **0** link error; thiếu từ điển `vi` → fallback, **0** crash |
| AC-11 | Linux: không kéo GTK4 | `ldd` trên binary engine | **không** có `libgtk-4.so` (nếu chọn enchant-2) |
| AC-12 | macOS: UI đúng D30 | thử trên 3 app (TextEdit/Safari/VSCode) | gợi ý chỉ ở toast/panel; **0** crash khi app không có cửa sổ |
| AC-13 | License | `cargo xtask preflight` + review `docs/compliance` | LGPL khai đủ; **không** static link; `Casks`/installer **không** chứa `.dic` (ADR-014) |

### 7.3 Việc **phải** làm kèm (không được quên)

1. **Sửa** `P4-4-OS-NATIVE-ASSIST.md` §1: (a) Linux `check_ranges` **không có** ở 0.4.x; (b)
   `hunspell-vi` **không** có sẵn trong Debian/Ubuntu.
2. **Sửa** `P4-0-MASTER-PLAN.md` nếu có chỗ nói "macOS API ngữ pháp chưa rõ" — `checkGrammar`
   **có** (nhưng v1 **không dùng**, giữ F68).
3. `deny.toml` **không** thấy thư viện C ⇒ thêm mục vào `docs/compliance/` + `REUSE.toml` cho
   libspelling/enchant (LGPL, dynamic link).
4. Ghi **privacy**: khi provider Windows out-of-proc, text rời tiến trình TextVN → nêu trong
   `antivirus-false-positive.md` / privacy notice, kèm nút tắt provider (`assist.scope = off`).

---

## 8. Nguồn

**Microsoft Learn** (truy cập 2026-10-21)

1. Spell Checking API (tổng quan) — <https://learn.microsoft.com/en-us/windows/win32/intl/spell-checker-api>
2. About the Spell Checking API (out-of-proc, restricted context, RPC timeout, `CoTaskMemFree`,
   vị trí user dictionary `%AppData%\Microsoft\Spelling\<tag>`) —
   <https://learn.microsoft.com/en-us/windows/win32/intl/about-the-spell-checker-api>
3. `ISpellCheckerFactory` ("should be created as an in-proc COM Server", Win8+) —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nn-spellcheck-ispellcheckerfactory>
4. `ISpellChecker` (danh sách method) —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nn-spellcheck-ispellchecker>
5. `ISpellChecker::Check` (`E_INVALIDARG` khi text rỗng) —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nf-spellcheck-ispellchecker-check>
6. `ISpellCheckerFactory::IsSupported` (BCP47) —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nf-spellcheck-ispellcheckerfactory-issupported>
7. `IEnumSpellingError` / `IEnumSpellingError::Next` —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nn-spellcheck-ienumspellingerror>
8. `ISpellingError::get_StartIndex` (**không** nêu đơn vị offset) —
   <https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nf-spellcheck-ispellingerror-get_startindex>

**Crate (đọc local)**

9. `windows-0.61.3/src/Windows/Win32/Globalization/mod.rs` — `ISpellCheckerFactory` (IID
   `8e018a9d-…`), `ISpellChecker` (IID `b6fd0b71-…`), `IEnumSpellingError` (IID `803e3bd4-…`),
   `ISpellingError` (IID `b7c82d61-…`), `SpellCheckerFactory` GUID `0x7ab36653_…`, `CORRECTIVE_ACTION_*`
10. `windows-0.61.3/Cargo.toml` — feature `Win32_Globalization` (dòng 439), `Win32_System_Com`
    (dòng 589); `ISpellChecker::Suggest` có `#[cfg(feature = "Win32_System_Com")]`
11. `windows-core-0.61.2/src/windows.rs` — `impl Param<PCWSTR> for &HSTRING` (dùng `&HSTRING` được)

**Apple Developer** (JSON, 2026-10-21)

12. `NSSpellChecker` (topic sections: `checkSpelling(of:startingAt:language:wrap:inSpellDocumentWithTag:wordCount:)`,
    `guesses(forWordRange:in:language:inSpellDocumentWithTag:)`,
    `correction(forWordRange:in:language:inSpellDocumentWithTag:)`,
    `availableLanguages`, `uniqueSpellDocumentTag()`) —
    <https://developer.apple.com/documentation/appkit/nsspellchecker>
13. `NLLanguageRecognizer` (macOS 10.14+, cảnh báo text ngắn, không dùng chéo thread) —
    <https://developer.apple.com/documentation/naturallanguage/nllanguagerecognizer>
14. `IMKInputController` / `IMKCandidates` ("presents candidates to users", optional) —
    <https://developer.apple.com/documentation/inputmethodkit/imkinputcontroller>,
    <https://developer.apple.com/documentation/inputmethodkit/imkcandidates>

**GNOME libspelling**

15. README (LGPLv2.1+, chỉ wrap `enchant-2`, user tự cài dict, mô tả `check_ranges` char offset) —
    <https://github.com/GNOME/libspelling/blob/main/README.md>
16. `SpellingChecker` (check_word / list_corrections / get_default / init) —
    <https://gnome.pages.gitlab.gnome.org/libspelling/libspelling-1/class.Checker.html>,
    <https://gnome.pages.gitlab.gnome.org/libspelling/libspelling-1/method.Checker.check_word.html>,
    <https://gnome.pages.gitlab.gnome.org/libspelling/libspelling-1/method.Checker.list_corrections.html>,
    <https://gnome.pages.gitlab.gnome.org/libspelling/libspelling-1/method.Checker.check_ranges.html>
17. `SpellingMistake { offset, length, byte_offset, byte_length }` (offset = **Unicode characters**)
    — <https://gnome.pages.gitlab.gnome.org/libspelling/libspelling-1/struct.Mistake.html>
18. Header theo tag: `lib/spelling-checker.h` @ `main` / `0.4.10` / `0.4.8` —
    <https://raw.githubusercontent.com/GNOME/libspelling/main/lib/spelling-checker.h> (và `…/0.4.10/…`,
    `…/0.4.8/…`) ⇒ `check_ranges` chỉ có ở `main`

**enchant**

19. Trang chủ (LGPL + exception, backend, UTF-8, tag `xx_YY`) — <https://rrthomas.github.io/enchant/>
20. Header `lib/enchant.h` (chữ ký `enchant_broker_init`, `enchant_broker_dict_exists`,
    `enchant_broker_request_dict`, `enchant_dict_check`, `enchant_dict_suggest`,
    `enchant_dict_free_string_list`) —
    <https://github.com/rrthomas/enchant/blob/main/lib/enchant.h>

**Phiên bản gói trong distro (Repology API, 2026-10-21)**

21. `libspelling` — <https://repology.org/api/v1/project/libspelling> (Debian 13 = 0.4.8; Ubuntu
    24.04 = 0.2.0; Fedora 43 = 0.4.9; Arch = 0.4.10)
22. `hunspell-vi` — <https://repology.org/api/v1/project/hunspell-vi> (Fedora 39–44 `GPL-2.0-only`;
    Alma/ALT/Amazon; **không** có entry Debian/Ubuntu)

**Đo thực nghiệm (máy này, 2026-10-21)** — probe C#/PowerShell trong `%TEMP%`, **không** commit,
**không** phải code sản phẩm: Windows 11 (OS build 26300, PowerShell build 26100), checker `en-US`. Số liệu ở §2.5.

---

## 9. ⚠️ Chưa verify được (đừng coi là đã chốt)

| # | Điều chưa verify | Vì sao | Cách verify sau |
|---|---|---|---|
| 1 | `NSSpellChecker.availableLanguages` **có `vi`** trên macOS hiện hành | không có máy macOS trong phiên này | chạy 20 dòng Swift in `availableLanguages` trên macOS 14/15/26 |
| 2 | Độ trễ `NSSpellChecker` (cold/warm) và việc có **block main thread** không | như trên | Instruments/`CFAbsoluteTime` quanh 100 lần `checkSpelling` |
| 3 | Windows provider mặc định của MS có **gửi mạng** không | MS Learn **không** khẳng định; provider có thể của bên thứ ba | Process Monitor (TCP) khi `Check` lần đầu trên máy sạch |
| 4 | `RPC_E_CANTCALLOUT_ININPUTSYNCCALL` có xảy ra khi gọi từ **TSF edit session** (input-sync) không | chỉ có báo cáo cộng đồng (dotnet/wpf #8772), chưa tái hiện | test trên Win11 + Word/Notepad, gọi từ `ITfEditSession::DoEditSession` |
| 5 | Windows: `Check()` trên **cả câu** vs **từng từ** — cái nào đúng hơn cho trait `contains` | mới đo tốc độ, chưa so chất lượng | so trên corpus P4-8: FPR/khớp `NoDictionary` |
| 6 | `hunspell-vi` có trong **Debian/Ubuntu** hay không (Repology **không** thấy, nhưng `packages.debian.org` chặn bot) | site bị PoW challenge | `apt-cache search hunspell | grep -i vi` trên Debian 13/Ubuntu 24.04 thật |
| 7 | Độ trễ `enchant-2` + `hunspell-vi` (Linux) | không có máy Linux trong phiên này | bench 1 000 từ trên Fedora 43 |
| 8 | libspelling **1.0** khi nào vào distro (để dùng `check_ranges`) | chưa có lịch phát hành | theo dõi tag + Repology trước mỗi release |
| 9 | Có **user dictionary** đọc được trên mọi bản Windows/macOS không (F71) | chỉ đọc được doc, chưa thử trên máy có user words | test với `%AppData%\Microsoft\Spelling\vi-VN\default.dic` tự tạo |
| 10 | Δsize/ΔRSS thật khi bật `Win32_Globalization` | chưa build (task code) | đo ở PR đầu tiên (AC-9) |

---

## 10. Phát hiện mới (đề xuất mã — **chờ chủ `P4-TASKS` cấp số**, không tự gán `Fassist-*`)

| ID tạm | Mức | Phát hiện | Xử lý đề xuất |
|---|---|---|---|
| **P4-10-F1** | **critical** | `spelling_checker_check_ranges` **không tồn tại** ở libspelling 0.4.x (bản distro đang phát hành) ⇒ đặc tả P4-4 §1 ("check_ranges → offset char") **không dùng được ở v1** | Dùng `check_word` + `list_corrections`; `check_ranges` = task v2; sửa P4-4 |
| **P4-10-F2** | major | `hunspell-vi` **không** có sẵn trong Debian/Ubuntu (chỉ Fedora/RHEL/Alma/ALT) + `GPL-2.0-only` | Sửa P4-4 §1; docs hướng dẫn user cài; giữ ADR-014 |
| **P4-10-F3** | major | `Suggest` của Windows **đắt ~50–100×** `Check` (3,1–6,9 ms vs 0,057 ms) mà `rules/spelling.rs` gọi `suggest` cho **mọi** từ OOV ⇒ nguy cơ vượt 50 ms | `Budgeted<D>` + cap `MAX_SUGGEST_WORDS = 3` (§5.3); đo ở AC-4 |
| **P4-10-F4** | major | Offset Windows là **UTF-16 code unit**, MS Learn **không** nói; bug **ẩn** với tiếng Việt NFC | convert tường minh + test astral (AC-6) |
| **P4-10-F5** | minor | `hasCorrection` (ghi trong brief) **không** có trong API công khai `NSSpellChecker` | dùng `correction(forWordRange:…)` |
| **P4-10-F6** | minor | `NSSpellChecker.checkGrammar` **có** (khác P4-4 "chưa rõ") nhưng dùng nó sẽ làm lệch hành vi 3 OS | giữ F68: **không** dùng; ghi rõ trong ADR/plan |
| **P4-10-F7** | minor | `libspelling` kéo **GTK4 + GtkSource5** vào engine IME; API còn **1.0.alpha** | enchant-2 trực tiếp làm mặc định (chỉ GLib) |
| **P4-10-F8** | minor | `cargo-deny` **không** kiểm soát license thư viện C (libspelling/enchant) | thêm `docs/compliance/assist-os-provider.md` + `LICENSES/` |

---

## 11. TL;DR cho người code

1. `assist` **không đổi**. Viết adapter implement `assist::dict::Dictionary` ở biên.
2. **P0 trước:** `Budgeted<D>` + `BridgeDict` + fake provider + test timeout/fail-open. Chưa cần OS.
3. **Linux trước** (enchant-2, `dlopen`, chỉ GLib). **Windows sau** (crate `windows` +
   `Win32_Globalization`, worker thread riêng, `recv_timeout(50ms)`), **macOS cuối** (Swift +
   `NSSpellChecker`, UI toast).
4. Cổng `vi`/`vi-VN` **1 lần/phiên**; không có ⇒ `NoDictionary` (Windows mặc định rơi vào đây).
5. `contains` dùng `Check` (0,06 ms); `suggest` **có ngân sách** (3–7 ms/từ!). Không log text.
6. Offset: Windows/macOS = **UTF-16** → convert sang **char** trước FFI. Linux 0.4.x = theo từ.
