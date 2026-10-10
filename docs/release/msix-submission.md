# Nộp TextVN lên Microsoft Store — gói MSIX

> Cập nhật 2026-10-09. Đây là đường nộp Store **chính**: gói EXE vướng chính sách
> 10.2.9 (bắt buộc Authenticode — đang chờ SignPath Foundation duyệt, xem
> `store-policy-10-2-9.md`), còn MSIX được Store **tự ký** khi publish.

## 0. Trạng thái — gói nộp Store 0.2.28 đã build (identity Partner Center có từ 2026-10-10)

| Hạng mục | Trạng thái | Bằng chứng |
|---|---|---|
| Kênh Store chạy thật ngoài container MSIX | ✅ | CI `ci-shared` › *MSIX sideload*: cài gói ký tạm → mở app → từ shell ngoài gói thấy bản stage, TIP HKCU, tray chạy, Run; gỡ gói → guard dọn sạch (`installer/windows/tests/test-msix-sideload.ps1`) |
| Manifest đúng luật Partner Center | ✅ | `tools/win/verify-msix.py` chạy trong CI: Version phần đầu ≠ 0, Publisher, DisplayName, PublisherDisplayName, mô tả tiếng Việt không mojibake, ảnh đúng kích thước, payload đủ |
| Binary không phụ thuộc VC++ redist | ✅ | `+crt-static` + `tools/win/check-pe-imports.py` trong `build-release.ps1` |
| `Publisher` | ✅ `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545` | Partner Center › Account settings › Windows publisher ID |
| `PublisherDisplayName` | ✅ `LinhBH.CoM` | Partner Center › Publisher display name |
| `DisplayName` | ✅ `TextVN` | Tên đã reserve của sản phẩm MSIX (lựa chọn A, §2) — `Package/Identity/Name` do Partner Center sinh từ tên này |
| **`Package/Identity/Name`** | ✅ `23651Linhi.TextVN` | Partner Center › sản phẩm MSIX › Product identity; PFN `23651Linhi.TextVN_qgnrpq341n182`, Store ID `9NV2R7JNNFGK` |
| Gói nộp hiện hành 0.2.29 | ✅ asset `TextVN-0.2.29-windows-x64.msix` của GitHub Release v0.2.29, SHA-256 `435326b6d8d7fd00592c62ed621a4d469af573c35a111437a95df0a9c9e3d5fa` | Build trong chính lượt phát hành ([release-candidate #38044174107](https://github.com/hunglinhpt/TextVN/actions/runs/38044174107)) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.29/TextVN-0.2.29-windows-x64-store.msix` |
| Gói nộp 0.2.28 | ✅ `TextVN-0.2.28-windows-x64.msix`, SHA-256 `20eced0f3b870ec0f785abece181d399a8295a3780443813bd8631713bb72eed` | Build lại trên tag `v0.2.28` bằng ô `msix_identity_name` — [release-candidate #38038814173](https://github.com/hunglinhpt/TextVN/actions/runs/38038814173) (artifact `release-windows`): Version `1.2.28.0`, `verify-msix.py --require-store-identity` OK, sideload PASS với PFN trên. Bản lưu chính thức: branch `approved` › [`v0.2.28/TextVN-0.2.28-windows-x64-store.msix`](https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.28/TextVN-0.2.28-windows-x64-store.msix) (+ `.sha256`) |

Gói `.msix` kèm GitHub Release 0.2.28 mang identity **tạm** (`LinhBH.CoM.TextVN`) — chỉ để thử, không nộp; nộp gói ở hàng "Gói nộp 0.2.28" phía trên. Repo variable `MSIX_IDENTITY_NAME` = `23651Linhi.TextVN` **đã đặt** (2026-10-10): từ bản phát hành sau, `.msix` kèm release mang identity thật, có chữ ký GPG/Sigstore và nộp thẳng được (§3).

Gói `.msix` trên release 0.2.27 / branch `approved` **KHÔNG nộp được** (identity
placeholder, Version `0.2.27.0`, mô tả mojibake, ảnh 71×71 sai) — phải build lại
theo §3.

## 1. Vì sao gói MSIX của TextVN có cơ chế "stage-out"

Bộ gõ Windows (TSF TIP) là DLL được **mọi ứng dụng** nạp. Microsoft Docs:
module nạp vào tiến trình ngoài gói "không được phép" từ bên trong gói, và mọi
ghi `HKCU`/file mới dưới `AppData` của tiến trình có package identity bị **ảo hoá**
(chỉ gói thấy, mất khi gỡ). Vì vậy (code: `tray/src/package_bootstrap.rs`,
`tray/src/store.rs`, `tray/src/store_win.rs`):

1. `TextVN.exe` trong gói **không đăng ký gì** — chép payload ra
   `%USERPROFILE%\.textvn\msix-staging\<V>` (ngoài AppData = ghi thật), khởi chạy
   bước relay với `PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY` (breakaway) rồi thoát.
2. Relay chạy `--msix-install` **ngoài container**: cài vào
   `%LOCALAPPDATA%\Programs\TextVN-Store\<V>`, ghi `stage.json`, khởi chạy tray.
3. Tray (không có identity) đăng ký TSF **chỉ cho tài khoản hiện tại** (HKCU,
   không UAC), đặt mục khởi động `TextVN` (`--autostart` hoặc `--msix-guard`).
4. Mỗi lần đăng nhập, guard kiểm `GetPackagesByPackageFamily`: gói đã gỡ → gỡ đăng
   ký TIP, xoá Run, trả Ctrl + Shift, hẹn xoá thư mục; gói có bản mới hơn → mở app
   trong gói để stage lại.
5. Menu khay "Gỡ cài đặt" (bản Store) dọn ngay rồi mở Cài đặt › Ứng dụng.
6. Bản Store **hỏi** trước khi dành Ctrl + Shift (chính sách 10.2.8) và **không bao
   giờ** xin quyền Administrator (cần cài cho mọi người dùng → bộ cài
   `*-machine.exe`).

## 2. Partner Center — việc của chủ tài khoản

1. **Chọn tên sản phẩm MSIX.** Phản hồi chính thức của Microsoft (10.2.9, xem
   `store-policy-10-2-9.md` §1): muốn dùng **cùng tên** cho MSIX thì phải **xoá tên
   khỏi sản phẩm Win32 (EXE)** trước. Hai lựa chọn:
   - **A (khuyến nghị nếu chưa cần gói EXE trên Store):** xoá tên `TextVN` khỏi sản
     phẩm EXE, tạo sản phẩm mới loại **MSIX or PWA app**, reserve lại `TextVN`.
     DisplayName giữ mặc định `TextVN` — không cần đặt thêm biến.
   - **B (giữ sản phẩm EXE chờ SignPath):** tạo sản phẩm MSIX với **tên khác** (ví
     dụ `TextVN - Bộ gõ tiếng Việt`), rồi đặt repo variable `MSIX_DISPLAY_NAME` đúng
     từng ký tự tên đó (§3).
2. Mở sản phẩm MSIX › **Product management › Product identity**, chép
   **`Package/Identity/Name`** (dạng `12345LinhBHCoM.TextVN`). Kiểm hai giá trị còn
   lại đúng như bảng §0 (`Package/Identity/Publisher`, `Package/Properties/
   PublisherDisplayName`).

## 3. Build gói nộp Store

**Cách 1 — CI, build lại riêng gói cho bản đã phát hành (khuyến nghị):**

Release trên GitHub (từ 0.2.28) kèm gói `.msix` mang identity **tạm** — chỉ để thử, không
nộp. Khi đã có `Package/Identity/Name` (§2):

1. GitHub › **Actions › release-candidate › Run workflow**: *Use workflow from* = tag
   phiên bản (ví dụ `v0.2.28`); ô **msix_identity_name** = giá trị `Package/Identity/Name`.
   (Lựa chọn B: đặt trước repo variable `MSIX_DISPLAY_NAME` = tên đã reserve.)
2. Có identity → build `-RequireStoreIdentity` và verify `--require-store-identity`:
   còn placeholder là **fail**, không ra gói nộp nhầm. Chạy tay **chỉ build** — job
   `publish` chỉ chạy khi push tag, release đã công khai không bị đụng tới (kể cả khi
   chọn tag).
3. Tải artifact `release-windows` của lượt chạy đó → `TextVN-<ver>-windows-x64.msix`.

Repo variable `MSIX_IDENTITY_NAME` (Settings › Secrets and variables › Actions ›
**Variables**) = `23651Linhi.TextVN` đã đặt từ 2026-10-10 → mọi bản phát hành sau tự kèm
gói nộp được (tải ngay từ trang Release, hoặc bản chép `-store.msix` trên branch
`approved`); ô nhập để trống thì workflow dùng biến này.

**Cách 2 — máy Windows có Windows SDK:**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File build-release.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
  -IdentityName "<Package/Identity/Name>" -RequireStoreIdentity
#   (lựa chọn B: thêm  -DisplayName "<tên đã reserve>")
python tools\win\verify-msix.py dist\TextVN-<ver>-windows-x64.msix --expect-version <ver> --require-store-identity
```

**Version:** Partner Center từ chối Identity Version có phần đầu bằng 0 → gói map
`A.B.C` thành `(A+1).B.C.0` (0.2.27 → **1.2.27.0**; bản 1.0.0 sau này → 2.0.0.0, vẫn
tăng dần). Store chỉ cập nhật khi Version mới **lớn hơn** — đừng nộp lại số cũ.

**Tự kiểm trước khi nộp (tuỳ chọn, máy Windows của bạn, PowerShell Admin):**
`installer\windows\tests\test-msix-sideload.ps1 -Msix <file.msix>` — ký tạm, cài,
mở, kiểm, gỡ, kiểm dọn sạch; nên chạy trên Windows 11 24H2 thật một lần.

## 4. Nộp submission

1. **Packages:** upload file `.msix` (không ký — Store tự ký). Chờ validate.
2. **Properties:** Category *Utilities & tools*; Privacy policy URL =
   `https://github.com/hunglinhpt/TextVN/blob/main/PRIVACY_POLICY.txt` (mục 7/EN và
   7/VI mô tả phần chép ra ngoài gói và cách gỡ).
3. **Age ratings:** bảng câu hỏi IARC — không có nội dung nhạy cảm, không mạng.
4. **Store listing (vi + en-US):** mô tả; ảnh `store/art/` (box art 1:1, poster
   2:3) và `store/screenshots/`. Manifest khai `vi` + `en-US`; giao diện tiếng
   Việt — ghi rõ trong mô tả tiếng Anh "UI is Vietnamese".
5. **Submission options › Notes for certification** — dán:

   > TextVN is a Vietnamese input method (Text Services Framework TIP). Windows only
   > loads input methods registered for the user, which a packaged app cannot do from
   > inside its package (in-proc modules loaded by other processes are not permitted
   > from the package). Therefore `runFullTrust` is required and, on first launch, the
   > app copies its own program files (no user data) to %USERPROFILE%\.textvn and
   > %LOCALAPPDATA%\Programs\TextVN-Store, registers the input method for the current
   > user only (HKCU, no elevation, no UAC) and adds a startup entry. At every sign-in
   > that entry checks whether the package is still installed and, if it was removed,
   > unregisters the input method and deletes the copied files. The tray menu
   > "Gỡ cài đặt" (Uninstall) does the same immediately and opens Settings > Apps.
   > The app asks before changing the Windows Ctrl+Shift layout hotkey. All processing
   > is local; no network access, no telemetry.
   > Test: install, open TextVN, open Notepad, switch to TextVN with Win+Space, type
   > "tieengs vieejt" → "tiếng việt".

6. Submit. Certification thường 1–3 ngày.

## 5. Rủi ro còn lại & cách xử lý

| Rủi ro | Mức | Nếu bị từ chối |
|---|---|---|
| Tester coi việc chép binary ra ngoài gói là trái 10.2.2 / hướng dẫn đóng gói | Trung bình | Trả lời bằng ghi chú ở §4.5 (đây là cách duy nhất để một TSF IME hoạt động; hướng dẫn IME của Microsoft yêu cầu đăng ký qua `ITfInputProcessorProfileMgr`); nếu vẫn từ chối → đường EXE ký Authenticode (SignPath) |
| 10.2.7 (gỡ sạch): MSIX không có hook gỡ | Thấp | Guard ở lần đăng nhập kế + menu "Gỡ cài đặt"; tài liệu ở chính sách quyền riêng tư |
| Win11 24H2+ từ chối kích hoạt TIP chỉ-per-user (B7) | Chưa rõ trên máy thật | Bản Store không xin UAC — hiện hướng dẫn chuyển sang bộ cài `*-machine.exe` |
| Version hiển thị trong Cài đặt là 1.2.27.0 (khác 0.2.27 trong app) | Thấp | Do map version (§3) — ghi chú trong mô tả nếu cần |

## 6. Sau khi publish

- Cài từ Store trên máy sạch → mở TextVN → Win+Space chọn TextVN → gõ thử.
- Cập nhật: tag version mới → workflow build gói (Version mới lớn hơn) → nộp
  package mới vào submission mới; tray tự stage lại bản mới ở lần mở/đăng nhập kế.
- Gỡ: Cài đặt › Ứng dụng › TextVN › Gỡ; lần đăng nhập kế guard dọn phần còn lại
  (hoặc dùng "Gỡ cài đặt" ở menu khay trước).
