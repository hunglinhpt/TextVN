# Nộp TextVN lên Microsoft Store — gói MSIX

> Cập nhật 2026-10-12. **TextVN ĐÃ PHÁT HÀNH trên Microsoft Store** qua gói MSIX:
> https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB (Store ID `9NV2R7JNNFGK`).
> MSIX là đường Store **chính**: gói EXE vướng chính sách 10.2.9 (bắt buộc Authenticode —
> đang chờ SignPath Foundation duyệt, xem `store-policy-10-2-9.md`), còn MSIX được Store
> **tự ký** khi publish. Mỗi bản phát hành mới: nộp asset `.msix` của release đó làm bản
> cập nhật — tự động bằng workflow `store-publish` qua Store submission API (§7), hoặc nộp
> tay trên Partner Center (§3–§4).

| Hạng mục | Trạng thái | Bằng chứng |
|---|---|---|
| **Trang Store** | ✅ [TextVN trên Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB) — miễn phí, mục Năng suất, x64, nhà phát hành `LinhBH.CoM` | API catalog Store (`storeedgefd.dsx.mp.microsoft.com/v9.0/products/9NV2R7JNNFGK`, đọc 2026-10-12): `ReleaseDateUtc 2026-10-11T14:43:50Z`, `LastUpdateDateUtc 2026-10-12T06:26:34Z`, PFN `23651Linhi.TextVN_qgnrpq341n182`. winget: `winget install --id 9NV2R7JNNFGK -s msstore` (installer `msstore`, scope user) |
| Cài thật từ Store | ✅ / ⚠️ | 2026-10-11 21:54 máy chủ repo cài `23651Linhi.TextVN_1.2.29.0` từ `msixbundle` Store phát (AppXDeploymentServer: thành công, hash gói khớp asset release v0.2.29). Trên máy đó Kaspersky 21.26 cách ly các tệp đã stage ra AppData ~2 phút sau lần mở đầu, và Windows 11 build 26300 từ chối `ActivateProfile` per-user (B7) — `docs/specs/antivirus-false-positive.md` §10 |
| Kênh Store chạy thật ngoài container MSIX | ✅ | CI `ci-windows` › *MSIX sideload*: cài gói ký tạm → mở app → từ shell ngoài gói thấy bản stage, TIP HKCU, tray chạy, Run; gỡ gói → guard dọn sạch (`installer/windows/tests/test-msix-sideload.ps1`) |
| Manifest đúng luật Partner Center | ✅ | `tools/win/verify-msix.py` chạy trong CI: Version phần đầu ≠ 0, Publisher, DisplayName, PublisherDisplayName, mô tả tiếng Việt không mojibake, ảnh đúng kích thước, payload đủ |
| Binary không phụ thuộc VC++ redist | ✅ | `+crt-static` + `tools/win/check-pe-imports.py` trong `build-release.ps1` |
| Nộp bản cập nhật tự động (GitHub Actions) | ⏳ workflow sẵn sàng, chờ chủ tài khoản cài app Entra + 4 secret | `.github/workflows/store-publish.yml` — kiểm chữ ký/hash/identity rồi `msstore publish` lên `9NV2R7JNNFGK` (§7) |
| `Publisher` | ✅ `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545` | Partner Center › Account settings › Windows publisher ID |
| `PublisherDisplayName` | ✅ `LinhBH.CoM` | Partner Center › Publisher display name |
| `DisplayName` | ✅ `TextVN` | Tên đã reserve của sản phẩm MSIX (lựa chọn A, §2) — `Package/Identity/Name` do Partner Center sinh từ tên này |
| **`Package/Identity/Name`** | ✅ `23651Linhi.TextVN` | Partner Center › sản phẩm MSIX › Product identity; PFN `23651Linhi.TextVN_qgnrpq341n182`, Store ID `9NV2R7JNNFGK` |
| Gói nộp hiện hành 0.2.44 | ✅ asset `TextVN-0.2.44-windows-x64.msix` của GitHub Release v0.2.44, SHA-256 `c823914866afb75b84ed1b1315c3233f9d6215753d31c7cc6b512d088d21723d` | Build trong chính lượt phát hành (tag `v0.2.44` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.44/TextVN-0.2.44-windows-x64.msix` |
| Gói nộp 0.2.43 | ✅ asset `TextVN-0.2.43-windows-x64.msix` của GitHub Release v0.2.43, SHA-256 `1ad5806e9484f39924475293c89d9f77cc192f664c943fb0c10974f873699770` | Build trong chính lượt phát hành (tag `v0.2.43` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.43/TextVN-0.2.43-windows-x64.msix` |
| Gói nộp 0.2.42 | ✅ asset `TextVN-0.2.42-windows-x64.msix` của GitHub Release v0.2.42, SHA-256 `4216fdbed34244b1b0c96d08e85bd6e97f54ec8c68362892781ed859f8ae9107` | Build trong chính lượt phát hành (tag `v0.2.42` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.42/TextVN-0.2.42-windows-x64.msix` |
| Gói nộp 0.2.41 | ✅ asset `TextVN-0.2.41-windows-x64.msix` của GitHub Release v0.2.41, SHA-256 `3f2afbe1a1331bf2d654779881400a0140829cf9944cf42368c3b55612b67456` | Build trong chính lượt phát hành (tag `v0.2.41` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.41/TextVN-0.2.41-windows-x64.msix` |
| Gói nộp 0.2.40 | ✅ asset `TextVN-0.2.40-windows-x64.msix` của GitHub Release v0.2.40, SHA-256 `dec7041f26dd18de846a885a1203822ac6a6b1d86aff274394bbe33c0b419465` | Build trong chính lượt phát hành (tag `v0.2.40` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.40/TextVN-0.2.40-windows-x64.msix` |
| Gói nộp 0.2.39 | ✅ asset `TextVN-0.2.39-windows-x64.msix` của GitHub Release v0.2.39, SHA-256 `87283aadfab61c7de98d4e4c8572877867e22a45360c41dc9f47c8d775b39801` | Build trong chính lượt phát hành (tag `v0.2.39` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.39/TextVN-0.2.39-windows-x64.msix` |
| Gói nộp 0.2.38 | ✅ asset `TextVN-0.2.38-windows-x64.msix` của GitHub Release v0.2.38, SHA-256 `690eac642ae89733a41da44f192b3623d3dc9d9c200fd02ae6ed59c9f067f1f2` | Build trong chính lượt phát hành (tag `v0.2.38` git-signed, release-candidate 4/4 job xanh); có chữ ký GPG/Sigstore; bản chép `approved/v0.2.38/TextVN-0.2.38-windows-x64.msix` |
| Gói nộp 0.2.37 | ✅ asset `TextVN-0.2.37-windows-x64.msix` của GitHub Release v0.2.37, SHA-256 `7f333528e0ed80b86987fb36f1efd3de150e2f412e0b53a7e74f6b7363d6e204` | Build trong chính lượt phát hành (tag `v0.2.37` git-signed, release-candidate 4/4 job xanh) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.37/TextVN-0.2.37-windows-x64.msix` |
| Gói nộp 0.2.36 | ✅ asset `TextVN-0.2.36-windows-x64.msix` của GitHub Release v0.2.36, SHA-256 `fbda3c522f7ad5a56b41161c6d1308eff73cd0a0d5ebaac35b3a6135bd02ba16` | Build trong chính lượt phát hành (tag `v0.2.36` git-signed, release-candidate 4/4 job xanh) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.36/TextVN-0.2.36-windows-x64.msix` |
| Gói nộp 0.2.34 | ✅ asset `TextVN-0.2.34-windows-x64.msix` của GitHub Release v0.2.34, SHA-256 `7868c8ed59ca447c53e117dbbb5c9466a9294ed60101ba5e7bbce2ebc01f894c` | Build trong chính lượt phát hành (tag `v0.2.34` git-signed, release-candidate 4/4 job xanh) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.34/TextVN-0.2.34-windows-x64.msix`. Lưu ý: Kaspersky cách ly binary bản Store khi stage ra AppData — `docs/specs/antivirus-false-positive.md` §10 |
| Gói nộp 0.2.33 | ✅ asset `TextVN-0.2.33-windows-x64.msix` của GitHub Release v0.2.33, SHA-256 `ee41ee1f51ea84e6b6285d6f91a258909697641a55dbe9db1737d03678462162` | Build trong chính lượt phát hành (tag `v0.2.33` git-signed, release-candidate 4/4 job xanh) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.33/TextVN-0.2.33-windows-x64.msix`. Lưu ý: Kaspersky cách ly binary bản Store khi stage ra AppData — `docs/specs/antivirus-false-positive.md` §10 |
| Gói nộp 0.2.32 | ✅ asset `TextVN-0.2.32-windows-x64.msix` của GitHub Release v0.2.32, SHA-256 `f983185406f70d048ae5e3b20ed961e1d00a7769e6f99a50cbe121b922f26602` | Build trong chính lượt phát hành ([release-candidate #38190208484](https://github.com/hunglinhpt/TextVN/actions/runs/38190208484)) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.32/TextVN-0.2.32-windows-x64-store.msix`. Lưu ý: Kaspersky cách ly binary bản Store khi stage ra AppData — `docs/specs/antivirus-false-positive.md` §10 |
| Gói nộp 0.2.29 | ✅ asset `TextVN-0.2.29-windows-x64.msix` của GitHub Release v0.2.29, SHA-256 `435326b6d8d7fd00592c62ed621a4d469af573c35a111437a95df0a9c9e3d5fa` | Build trong chính lượt phát hành ([release-candidate #38044174107](https://github.com/hunglinhpt/TextVN/actions/runs/38044174107)) với identity từ repo variable; có chữ ký GPG/Sigstore; bản chép `approved/v0.2.29/TextVN-0.2.29-windows-x64-store.msix` |
| Gói nộp 0.2.28 | ✅ `TextVN-0.2.28-windows-x64.msix`, SHA-256 `20eced0f3b870ec0f785abece181d399a8295a3780443813bd8631713bb72eed` | Build lại trên tag `v0.2.28` bằng ô `msix_identity_name` — [release-candidate #38038814173](https://github.com/hunglinhpt/TextVN/actions/runs/38038814173) (artifact `release-windows`): Version `1.2.28.0`, `verify-msix.py --require-store-identity` OK, sideload PASS với PFN trên. Bản lưu chính thức: branch `approved` › [`v0.2.28/TextVN-0.2.28-windows-x64-store.msix`](https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.28/TextVN-0.2.28-windows-x64-store.msix) (+ `.sha256`) |

Gói `.msix` kèm GitHub Release 0.2.28 mang identity **tạm** (`LinhBH.CoM.TextVN`) — chỉ để thử, không nộp; nộp gói ở hàng "Gói nộp 0.2.28" phía trên. Repo variable `MSIX_IDENTITY_NAME` = `23651Linhi.TextVN` **đã đặt** (2026-10-10): từ bản phát hành sau, `.msix` kèm release mang identity thật, có chữ ký GPG/Sigstore và nộp thẳng được (§3).

Gói `.msix` trên release 0.2.27 / branch `approved` **KHÔNG nộp được** (identity
placeholder, Version `0.2.27.0`, mô tả mojibake, ảnh 71×71 sai) — phải build lại
theo §3.

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

| Rủi ro | Mức | Nếu bị từ chối |
|---|---|---|
| Tester coi việc chép binary ra ngoài gói là trái 10.2.2 / hướng dẫn đóng gói | Trung bình | Trả lời bằng ghi chú ở §4.5 (đây là cách duy nhất để một TSF IME hoạt động; hướng dẫn IME của Microsoft yêu cầu đăng ký qua `ITfInputProcessorProfileMgr`); nếu vẫn từ chối → đường EXE ký Authenticode (SignPath) |
| 10.2.7 (gỡ sạch): MSIX không có hook gỡ | Thấp | Guard ở lần đăng nhập kế + menu "Gỡ cài đặt"; tài liệu ở chính sách quyền riêng tư |
| Win11 24H2+ từ chối kích hoạt TIP chỉ-per-user (B7) | **Đã gặp** (2026-10-11, máy chủ repo Windows 11 build 26300, bản cài từ Store: `RegisterProfile`/`ActivateProfile` = `0x80004005`; runner CI Windows Server 2022 không gặp) | Bản Store không xin UAC — hướng dẫn chuyển sang bộ cài `*-machine.exe` (đăng ký HKLM) |
| Antivirus cách ly tệp đã stage ra AppData (chưa ký Authenticode) | **Đã gặp** với Kaspersky 21.26 (2026-10-11, ~2 phút sau lần mở đầu) | `docs/specs/antivirus-false-positive.md` §10: AV-5 lấy tên detection, AV-1 báo nhầm (hash 4 binary của gói), AV-2 SignPath, AV-6 Kaspersky Allowlist, AV-7 giảm hành vi giống dropper của kênh Store |
| Version hiển thị trong Cài đặt là 1.2.27.0 (khác 0.2.27 trong app) | Thấp | Do map version (§3) — ghi chú trong mô tả nếu cần |

- **Đã publish 2026-10-11** — trang Store: https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB
  (README, `docs/user-guide.md` và `HUONG_DAN_SU_DUNG.txt` trỏ link này; winget:
  `winget install --id 9NV2R7JNNFGK -s msstore`).
- Cài từ Store trên máy sạch → mở TextVN → Win+Space chọn TextVN → gõ thử.
- Cập nhật: tag version mới → workflow build gói (Version mới lớn hơn) → nộp
  package mới vào submission mới (tự động: §7); tray tự stage lại bản mới ở lần mở/đăng
  nhập kế.
- Gỡ: Cài đặt › Ứng dụng › TextVN › Gỡ; lần đăng nhập kế guard dọn phần còn lại
  (hoặc dùng "Gỡ cài đặt" ở menu khay trước).

Workflow `.github/workflows/store-publish.yml` nộp asset `.msix` của một GitHub Release
làm bản cập nhật cho sản phẩm `9NV2R7JNNFGK`, bằng **Microsoft Store Developer CLI**
(`msstore`) — CLI chính thức của Microsoft cho CI/CD, gọi Store submission API của
Partner Center. Nguồn: Microsoft Learn — [Publish app updates to Microsoft Store with
GitHub Actions](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/github-actions),
[Microsoft Store Developer CLI Commands (MSIX)](https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/commands);
hành vi chi tiết đối chiếu mã nguồn [microsoft/msstore-cli v0.4.3](https://github.com/microsoft/msstore-cli/tree/v0.4.3).

#| Job | Máy | Secret | Việc |
|---|---|---|---|
| `verify` | ubuntu-24.04 | không | Tag dạng `vA.B.C`, release đã công khai → tải `TextVN-<ver>-windows-x64.msix` + `.asc` + `.cosign.sig/.cert` + `SHA256SUMS.txt` → `tools/release/verify-store-package.sh`: chữ ký GPG của gói và của `SHA256SUMS.txt` do khoá phát hành `3921595A…` ký (public key lấy từ checkout, không lấy bản đính kèm release), SHA-256 khớp, Sigstore đúng `release.yml@refs/tags/<tag>` của repo này, `verify-msix.py --require-store-identity --expect-name <MSIX_IDENTITY_NAME>` (Version `(A+1).B.C.0`) |
| `submit` | windows-2022, environment `microsoft-store` | 4 secret | Kiểm lại SHA-256 gói nhận từ `verify` → tải CLI `msstore` bản ghim (version + SHA-256) → `msstore reconfigure` (kiểm luôn đăng nhập) → tắt telemetry CLI → **đọc trạng thái Store** (submission đang chờ/đã phát hành, version gói) và chặn sớm → `msstore publish <msix> --appId 9NV2R7JNNFGK` (chế độ `draft` thêm `--noCommit`; `check` bỏ qua) → `msstore submission status` + tóm tắt run |

Ba chế độ (ô `mode` khi chạy tay; mặc định `check`):

| Chế độ | Thay đổi gì trên Partner Center |
|---|---|
| `check` | Không gì cả — chỉ đăng nhập, đọc trạng thái + version đang có trên Store, báo `draft`/`submit` với tag đó có được phép không. Dùng để kiểm cài đặt |
| `draft` | Upload gói vào submission mới để nháp (`--noCommit`) — bạn bấm Submit trên Partner Center |
| `submit` | Upload + commit: nộp chứng nhận ngay |

Bước *Read Store state* chặn `draft`/`submit` **trước** khi gọi `msstore publish` (vì publish xoá
bản nháp đang chờ rồi mới thất bại) khi: submission đang xử lý/chứng nhận (`CommitStarted`,
`PreProcessing`, `Certification`, `PendingPublication`, `Publishing`, `Release`), hoặc bản đã
phát hành đã có version ≥ gói của tag.

Hành vi của `msstore publish` (mã nguồn v0.4.3) cần biết:

- Tạo submission mới **từ bản đang phát hành** và chỉ thay **package**: listing, ảnh, giá,
  thị trường, ghi chú chứng nhận giữ nguyên. Mục "What's new in this version" cũng giữ nội
  dung cũ → muốn viết mới thì chạy chế độ `draft`, sửa trên Partner Center rồi bấm Submit.
- **Xoá bản nháp đang chờ** (pending submission) trên Partner Center nếu có — đừng để dở bản
  nháp sửa tay khi chạy workflow. Submission đang chứng nhận dở thì không xoá được → bước
  publish báo lỗi; chờ xong (hoặc huỷ trên Partner Center) rồi chạy lại.
- `submit`: commit rồi chờ Partner Center xử lý commit xong (vài phút). Chứng nhận sau đó vẫn
  1–3 ngày — theo dõi trên Partner Center. `draft`: chỉ upload gói, không commit.
- Version gói phải **lớn hơn** bản đang trên Store (map §3 luôn tăng); nộp lại số cũ thì
  Partner Center từ chối.

#1. **Tenant:** Partner Center › Account settings › Tenants — tài khoản phải liên kết một
   Microsoft Entra tenant ([liên kết tenant có sẵn](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/associate-existing-azure-ad-tenant-with-partner-center-account)
   hoặc [tạo tenant mới](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/create-new-azure-ad-tenant)).
2. **App Entra:** <https://entra.microsoft.com> › Identity › Applications › App registrations ›
   New registration (ví dụ tên `TextVN Store Publisher`, single tenant, không cần redirect
   URI). Chép **Application (client) ID** và **Directory (tenant) ID**.
3. **Client secret:** app đó › Certificates & secrets › New client secret — chép cột
   **Value** ngay (chỉ hiện một lần) và ghi lại ngày hết hạn.
4. **Quyền Partner Center:** Account settings › User management › **Microsoft Entra
   applications** › Add Microsoft Entra application › chọn app ở bước 2 › vai trò
   **Manager**.
5. **Seller ID:** Partner Center › Account settings › Legal info (hoặc Developer settings /
   Identifiers) — chép **Seller ID** (dãy số).
6. **GitHub:** Settings › Environments › New environment `microsoft-store`:
   - Environment secrets: `PARTNER_CENTER_TENANT_ID`, `PARTNER_CENTER_SELLER_ID`,
     `PARTNER_CENTER_CLIENT_ID`, `PARTNER_CENTER_CLIENT_SECRET` (đặt ở cấp repo cũng chạy,
     nhưng environment giới hạn secret cho riêng job nộp).
   - Khuyến nghị: Deployment protection rules › **Required reviewers** = chính bạn → mỗi lượt
     nộp (kể cả tự động) dừng chờ bạn bấm Approve.
7. **Kiểm cài đặt:** Actions › **store-publish** › Run workflow (Use workflow from `main`) ›
   `tag` = bản phát hành mới nhất, `mode` = `check` → duyệt deployment → job `submit` phải đăng
   nhập được và in trạng thái + version đang có trên Store (không thay đổi gì).
8. **Nộp thật:** khi có tag mới hơn bản trên Store, chạy lại với `mode` = `draft` → kiểm
   Partner Center có bản nháp mang gói mới → bấm Submit (hoặc chạy với `submit`).

#Repo variable **`STORE_AUTO_PUBLISH`** (Settings › Secrets and variables › Actions ›
Variables):

| Giá trị | Sau khi `release-candidate` của một tag phát hành thành công |
|---|---|
| không đặt / giá trị khác | Không làm gì (mặc định) — chạy tay như 7.2 bước 8 |
| `draft` | Tự upload gói vào bản nháp; bạn sửa "What's new" nếu cần rồi bấm Submit |
| `submit` | Tự nộp chứng nhận (có Required reviewers thì vẫn chờ bạn duyệt) |

Trigger là `workflow_run` của `release-candidate` (lượt do push tag và kết thúc
`success`), **không** phải `release: published`: release do `GITHUB_TOKEN` tạo không kích hoạt
workflow khác (GitHub chỉ ngoại lệ `workflow_dispatch`/`repository_dispatch`).

#- `verify` không có secret; `submit` chỉ nộp đúng gói `verify` đã kiểm (so SHA-256 lần nữa).
  Chạy tay chỉ nhận từ `main` (secret chỉ đi cùng workflow đã review).
- Không dùng action bên thứ ba để cài CLI: action `microsoft/microsoft-store-apppublisher`
  mặc định tải bản `latest` và không kiểm hash, nên workflow tải thẳng
  `MSStoreCLI-win-x64.zip` từ release chính thức và ghim `MSSTORE_CLI_VERSION` +
  `MSSTORE_CLI_SHA256`. Nâng bản CLI: đổi cả hai (hash có trong
  `MSStoreCLI-win-x64.zip.sha256.txt` của release đó — tự tính lại để đối chiếu).
- Log của Actions là công khai: JSON submission chứa `FileUploadUrl` (SAS ghi được vào blob của
  submission) nên workflow **không bao giờ in nguyên JSON** (chỉ in trạng thái + gói), và
  **không dùng `--verbose`** cho `publish` — ở mức Information CLI in `FileUploadUrl` ra log.
  Ngoại lệ duy nhất: khi `submission get` lỗi (CLI chỉ in `Error!`), workflow chạy lại một lần
  với `--verbose` và **bỏ stdout** để log stderr hiện lý do (E22). Ví dụ
  *GetBaseMetadata* của Learn in cả submission ra log — đừng chạy nó trên repo này.
- Client secret Entra hết hạn (tối đa 24 tháng) → bước *Configure msstore* báo lỗi xác thực →
  tạo secret mới (7.2 bước 3) và cập nhật `PARTNER_CENTER_CLIENT_SECRET`.
- Phương án không cần secret: CLI v0.4.3 có `msstore reconfigure --clientAssertion` (đọc
  `MSSTORE_CLIENT_ASSERTION` hoặc `MSSTORE_CLIENT_ASSERTION_FILE`) — có thể dùng federated
  credential với OIDC của GitHub. Chưa nối vào workflow vì Microsoft Learn chưa mô tả.
- Tài liệu Learn lệch CLI ở cờ `-i` của `publish` — xem E19 (`docs/specs/project-common-errors.md`).
