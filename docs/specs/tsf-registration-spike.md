# Spike WIN-003 — Đăng ký TIP per-user, elevation scope, `InstallLayoutOrTip`

> Task: `WIN-003` (`P1-6-TASKS.md`, acceptance: *"registry paths thực tế + Win+Space thấy
> VietIME"*) · Checklist gốc: `../20-windows/P1-1-tsf.md §8` · Code: `../../spikes/tsf-min/src/register.rs`
> Ngày chạy: 2026-09-27 · Windows 11 x64 · Log bằng chứng: `%LOCALAPPDATA%\VietIME\logs\tsf-register.log`

CLSID của spike: `{6B7E1F80-4A2D-4E93-9C55-1F0A7D2E9C11}`
(`tsf_min::CLSID_VIETIME_TIP` — GUID **sẽ đổi** khi implement thật, mọi path dưới đây giữ nguyên).

> **Finding spike:** ID dạng `S3-{n}` — đánh số **dùng chung** cho 2 doc spike TSF
> (`tsf-spike.md` = S3-1/2, doc này = S3-2/3/4); khác với finding review log (`F0-*`/`F1-*`).

## 1. Bảng scope — API nào cần elevation?

| # | API / bước | Ghi vào | Không admin? | Bằng chứng (tsf-register.log) |
|---|---|---|---|---|
| 1 | COM server: `HKCU\Software\Classes\CLSID\{…}\InprocServer32` = `<path>\tsf_min.dll`, `ThreadingModel=Apartment` | **HKCU** | ✅ **OK** | `reg add … → OK` ×3, `reg query … → OK` |
| 2 | `ITfInputProcessorProfiles::Register(CLSID)` | **HKLM** (CTF) | ❌ cần elevation | chỉ chạy được qua `runas` |
| 3 | `AddLanguageProfile(CLSID, 0x042A/0x0409, PROFILE, L"VietIME", icon, idx 0)` | **HKLM** | ❌ cần elevation | elevated → `→ OK`, readback `"VietIME"` (utf16 = `0056 0069 0065 0074 0049 004d 0045`) |
| 4 | `EnableLanguageProfileByDefault(CLSID, lang, PROFILE, true)` | **HKLM** | ❌ → `E_FAIL 0x80004005` | ở user context **luôn fail, vô hại**; elevated → `→ OK` |
| 5 | `ITfCategoryMgr::RegisterCategory(CLSID, GUID_TFCAT_TIP_KEYBOARD, CLSID)` | **HKLM** | ❌ cần elevation | elevated → `→ OK` |
| 6 | **`InstallLayoutOrTip`** (`input.dll`, flag `ILOT_DEFPROFILE=0x2`) | **HKCU user input list** | ✅ **OK, KHÔNG cần elevation** | `InstallLayoutOrTip(0x0409:{CLSID}{ProfileGUID}, DEFPROFILE) → OK` |

**Kết luận (exit của `P1-1 §8`):** 4 API số 2–5 ghi HKLM → `vietime register --scope machine`
cần elevation; **scope user không đủ** nếu chỉ dùng 4 API đó.

## 2. Fallback đã chứng minh cho `--scope user`

```text
vietime register --scope user
 1. [KHÔNG admin] reg.exe ghi CLSID → HKCU\Software\Classes\CLSID\{…}\InprocServer32   (mục 1)
 2. [cần elevation MỘT LẦN] Register + AddLanguageProfile + EnableLanguageProfileByDefault
    + RegisterCategory  → HKLM                                                          (mục 2–5)
    (installer/vietime-setup.exe đã ký, UAC một lần cho 4 API này)
 3. [KHÔNG admin, mọi lần sau] InstallLayoutOrTip → danh sách input method của user
    spec = "0x{lang:04X}:{CLSID}{ProfileGUID}", flag ILOT_DEFPROFILE = 0x2               (mục 6)
 4. [KHÔNG admin] ActivateLanguageProfile(langid = locale ĐANG CHẠY) → kích hoạt ngay
```

- `InstallLayoutOrTip` **không có import lib** → `LoadLibraryW("input.dll")` +
  `GetProcAddress` (đúng MS sample); **không** có `FreeLibrary` trong bindings 0.61
  → tiến trình CLI thoát ngay, vô hại (ghi trong code).
- **Thứ tự BẮT BUỘC:** `InstallLayoutOrTip` → `EnableLanguageProfileByDefault` →
  `ActivateLanguageProfile` — cập nhật input list có thể **reset enabled flag** (đã thử
  đảo thứ tự: enable biến mất).
- `ActivateLanguageProfile` phải chạy dưới **locale đang active của session** (thử spike:
  0x0409 en-US) — locale khác ⇒ không load TIP (Keyman wiki). Không ép đổi ngôn ngữ session.
- Elevation output **bị ẩn console** → toàn bộ log đi qua `say()` ghi
  `%LOCALAPPDATA%\VietIME\logs\tsf-register.log` (append, có pid) — **bắt buộc** với
  `vietime register` thật, không thì không verify lại được.

```rust
// register.rs — install_layout_or_tip (mục 6, không cần elevation)
const ILOT_DEFPROFILE: u32 = 0x0000_0002;
let hmod = LoadLibraryW(w!("input.dll"))?;
let Some(fp) = GetProcAddress(hmod, s!("InstallLayoutOrTip")) else { return };
type Pfn = unsafe extern "system" fn(*const u16, u32) -> i32;
let spec = format!("0x{lang:04X}:{}{}", guid_str(&CLSID_VIETIME_TIP), guid_str(&PROFILE_GUID));
let wide: Vec<u16> = spec.encode_utf16().chain(std::iter::once(0)).collect();
let ret = unsafe { std::mem::transmute::<_, Pfn>(fp)(wide.as_ptr(), ILOT_DEFPROFILE) };
// ret != 0 = TRUE = TIP đã vào danh sách input method của user (không admin)
```

## 3. Kết quả verify Win+Space / registry

- Registry **cả 2 langid** `0x0409` (en-US) và `0x042A` (vi-VN) đều có profile
  `Enable=1` sau elevated install.
- Readback mô tả profile = `"VietIME"` — utf16 **sạch** từng byte
  (`0056 0069 0065 0074 0049 004d 0045`), không còn rác.
- TIP nằm trong language list của user (mục 6 OK) → Win+Space liệt kê; verify trực quan
  flyout là bước **optional còn lại** (không chặn exit của spike — registry/readback đủ).

## 4. Finding (bắt buộc áp cho impl)

> **S3-2 — `wcslen()` + slice không null-terminate → đọc rác / AV.**
> Các API TSF nhận chuỗi mô tả/icon (`AddLanguageProfile`, `ITfRange::SetText`) đọc bằng
> `wcslen()` chứ **không** theo length truyền vào. Bằng chứng trong log:
> readback **trước khi fix** = `"VietIME OWS\…"` + 20 byte rác
> (`utf16=… 0057 0053 005c b792 7f75 1873 …`) — chính là phần tiếp của allocation.
> **Fix (đã áp):** truyền slice đúng length nhưng allocation **phải đệm số 0 phía sau**;
> icon truyền `&[]` (ptr vô nghĩa 0x2) → **Access Violation** → icon = `&[0u16]` hoặc
> pointer hợp lệ + length 0. Áp dụng cho MỌI chỗ truyền `&[u16]` vào TSF
> (mọi text của composition — xem `tsf-spike.md` #4).

> **S3-3 — `EnableLanguageProfileByDefault` fail `0x80004005` ở user context.**
> Không phải bug của ta: API này ghi HKLM. Bỏ qua lỗi khi `--scope user`,
> **không** retry, **không** coi là fatal — số 2–5 đã chạy trong bước elevation.

> **S3-4 — readback trước khi `AddLanguageProfile` trả chuỗi rác.**
> Trước khi profile được add, `GetStringValue` đọc vùng chưa khởi tạo → không dùng
> readback để phán "profile đã có"; kiểm bằng `reg query` cột `Enable` + `RemoveLanguageProfile`
  trả OK/FAIL.

## 5. Exit

- [x] Registry paths thực tế đã ghi (bảng §1) — acceptance `WIN-003`.
- [x] Đường không-elevation chứng minh bằng chạy thật (`InstallLayoutOrTip → OK`).
- [x] Findings S3-2/3/4 ghi nhận, fix đã có trong spike.
- [ ] Verify trực quan Win+Space thấy "VietIME" — optional, còn lại.
- [ ] Cleanup cuối spike (hỏi user trước): reset `Assemblies` Default/Profile + `uninstall`.
