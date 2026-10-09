# Rà soát vòng 2 — audit · QA · chuyên gia — 2026-10-08/09

> Tiếp theo `code-review-2026-10-07.md` (vòng 1, CR-01..CR-41). Yêu cầu của chủ sở hữu:
> cập nhật tài liệu đúng hiện trạng (Linux đã ký số, macOS xong phần phát triển, EXE chỉ
> chờ SignPath), **hoàn thiện gói MSIX để nộp Microsoft Store**, rà thêm một lượt và sửa
> lỗi tiềm ẩn dưới góc nhìn audit, QA và chuyên gia. Finding giữ ID `R2-xx`, không xoá —
> chỉ đổi trạng thái (quy ước `00-INDEX.md` §3).

## 0. Kết luận

| Kênh | Trạng thái | Việc còn lại của chủ sở hữu |
|---|---|---|
| **Microsoft Store (MSIX)** | ✅ Sẵn sàng nộp. Kênh Store đã chạy thật trên runner Windows: cài gói, mở app, TIP đăng ký được ngoài container, gỡ gói thì guard dọn sạch (§2) | Lấy `Package/Identity/Name` ở Partner Center → đặt repo variable `MSIX_IDENTITY_NAME` (và `MSIX_DISPLAY_NAME` nếu dùng tên khác) → chạy workflow `release-candidate` → nộp file `.msix`. Từng bước: `msix-submission.md` |
| Windows EXE (Inno, portable) | ✅ Code ký số xong: SignPath REST API đúng tài liệu, một yêu cầu ký cho cả bộ binary, ký cả `-machine.exe` | Khi SignPath Foundation duyệt: đặt secret `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_POLICY`, `SIGNPATH_ARTIFACT_CONFIGURATION` (cấu hình ZIP, deep sign `*.exe`/`*.dll`) |
| Linux | ✅ Ký GPG + Sigstore cho mọi asset, CI **dừng** nếu thiếu/sai khoá (fail-closed) | — |
| macOS | ✅ Code; ký Developer ID + notarize + staple bật **khi có** secret `APPLE_*`, thiếu thì ký ad-hoc và ghi rõ trong release notes | Đặt secret Apple (xem `signing-status-mac.md`) |

**Tổng kết finding:** 87 finding R2 — **80 đã sửa** (R2-20/48/83 xong phần code, chờ chứng
chỉ Apple; R2-85 một phần), 1 chờ SignPath (R2-12), 3 giữ nguyên có lý do (R2-11, R2-47,
R2-67), 3 hoãn (R2-14, R2-26, R2-41 — §5). Vòng 1 xong thêm CR-20 (a), CR-34, CR-38.

**Không nộp** gói `.msix` đính kèm release v0.2.27 hay trên nhánh `approved`: identity
placeholder, Version `0.2.27.0` (Partner Center từ chối phần đầu = 0), mô tả tiếng Việt
mojibake, ảnh 71×71 sai — `tools/win/verify-msix.py` bắt đủ 8 lỗi trên gói đó.

## 1. Phương pháp

1. **Rà soát** song song theo 7 nhóm: kiến trúc MSIX, bảo mật Windows, QA Windows,
   Linux + macOS, tiếng Việt (2 nhóm: chính tả/engine, kiểu gõ), CI/phát hành → 87 finding.
2. **Kiểm chứng** từng finding: tái hiện (replay corpus, test, chạy script trên gói đã phát
   hành, đọc tài liệu Microsoft/Apple) trước khi sửa; finding trùng nhau gộp về một bản sửa.
3. **Sửa cuốn chiếu**: mỗi nhóm một commit (Conventional Commits, ID finding trong thân),
   test đỏ-trước-xanh-sau ở mọi chỗ chạy được; phần Windows/macOS kiểm bằng CI thật.

## 2. Bằng chứng CI

| Run | Commit | Nội dung | Kết quả |
|---|---|---|---|
| ci-shared [37864768411](https://github.com/hunglinhpt/TextVN/actions/runs/37864768411) | `6ccd5d4` | **MSIX sideload**: ký tạm, `Add-AppxPackage`, mở `shell:AppsFolder\<PFN>!TextVN`; từ shell không có identity thấy `stage.json` + `TextVN.exe` (không bị ảo hoá), TIP HKCU trỏ vào bản stage, tray chạy ngoài container, Run `TextVN`; `Remove-AppxPackage` → `--msix-guard` gỡ TIP + Run. Identity `1.2.27.0` | ✅ PASS toàn bộ |
| release-candidate [37865843028](https://github.com/hunglinhpt/TextVN/actions/runs/37865843028) | `6e19caa` | Build thử bản phát hành 3 nền tảng: verify MSIX, bộ cài machine, VirusTotal (gồm `.msix` và `-machine.exe`); `publish` bỏ qua vì không phải tag | ✅ |
| ci-macos [37887743367](https://github.com/hunglinhpt/TextVN/actions/runs/37887743367) · [37888463980](https://github.com/hunglinhpt/TextVN/actions/runs/37888463980) | `f2920f1` · `017bc82` | Swift build + test arm64/x86_64 (IMK, tap, app) gồm thay đổi macOS R2-43/44/49/50/51 và CR-34, plist, header C-ABI, gói candidate | ✅ |
| ci-shared [37917087193](https://github.com/hunglinhpt/TextVN/actions/runs/37917087193) | `d70af11` | **19/19 job**: fmt + Cargo.lock `--locked`, clippy Linux + Windows (gồm hook), test 3 OS, replay 4 adapter × 3 OS, fuzz (gồm Backspace mới), ASan/UBSan linux-common, cargo-deny, reuse, perf A/B, IBus/Fcitx5 e2e + gói Linux; gói Windows: portable + bộ cài gõ thật Notepad/WordPad, **R2-40** (gỡ trả Ctrl+Shift về đúng giá trị cũ — portable và bộ cài), Store validation + giả lập validator (cài im lặng 0,6 s), MSIX build + verify + **sideload PASS** (`1.2.27.0`) | ✅ |
| release-candidate [37917194716](https://github.com/hunglinhpt/TextVN/actions/runs/37917194716) | `d70af11` | Build phát hành 3 nền tảng `--locked`, VERSIONINFO bắt buộc, verify + sideload chính gói MSIX sắp phát hành, bộ cài machine, VirusTotal; `publish` bỏ qua vì không phải tag | ✅ |

**Sự cố CI trong vòng này (đã sửa):** từ `13ba81f` (R2-81) bước giả lập Store validator mở bản vừa
build mang MOTW bằng ShellExecute → hộp thoại cảnh báo bảo mật của Windows treo phiên không tương
tác tới khi job bị huỷ sau 60 phút (3 lượt ci-shared bị huỷ). `d70af11` chạy installer bằng
CreateProcess như harness cài im lặng, giới hạn 300 s, step `timeout-minutes: 15`.

## 3. Nhật ký finding

Mức độ như vòng 1: **P0** mất chữ/crash/lỗ hổng/bị Store từ chối · **P1** sai hành vi người
dùng thấy · **P2** rủi ro tiềm ẩn · **P3** chất lượng/nit. Trạng thái: ✅ đã sửa (commit) ·
⏳ chờ điều kiện ngoài code · ⏸ hoãn (lý do ở §5) · 📝 quyết định, không đổi code.

### 3.1 Microsoft Store / MSIX

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-01 | **P0** | Identity Version `0.2.27.0` — Partner Center từ chối phần đầu = 0 (makeappx vẫn nhận); bản pre-release sinh version sai schema | Map `A.B.C` → `(A+1).B.C.0` (0.2.27 → 1.2.27.0), kiểm định dạng | ✅ `b10be1c` |
| R2-02, R2-19, R2-22 | **P0** | Stage-out giả định bản chép ra chạy ngoài gói; thực tế ghi `%LOCALAPPDATA%`/HKCU của tiến trình có package identity bị **ảo hoá** → app khác không thấy TIP | Exe trong gói không đăng ký gì: chép ra `%USERPROFILE%\.textvn\msix-staging`, relay với `PROC_THREAD_ATTRIBUTE_DESKTOP_APP_POLICY` (breakaway), `--msix-install` chạy ngoài container cài vào `%LOCALAPPDATA%\Programs\TextVN-Store\<V>`; chốt cứng: còn identity thì từ chối mọi ghi TSF/Run/hotkey | ✅ `ca4b63b`, sideload CI |
| R2-04 | P1 | Gỡ gói Store nhưng TIP, binary đã chép, Run, phím tắt vẫn còn | Guard ở mỗi lần đăng nhập (`GetPackagesByPackageFamily`): gói đã gỡ → gỡ TIP, Run, trả Ctrl+Shift, hẹn xoá thư mục; menu "Gỡ cài đặt" bản Store dọn ngay rồi mở Cài đặt › Ứng dụng | ✅ `ca4b63b` |
| R2-05, R2-23 | P1 | Cập nhật Store không tới bản đã chép khi bật tự khởi động | Guard thấy gói mới hơn → mở app trong gói để stage lại | ✅ `ca4b63b` |
| R2-06 | P1 | Bản Store tự đổi phím tắt Ctrl+Shift của Windows không hỏi (chính sách 10.2.8) | Hỏi một lần khi người dùng mở app; lưu cả câu trả lời "Không" | ✅ `ca4b63b` |
| R2-07 | P1 | Win11 24H2+ cần UAC để kích hoạt TIP — hướng dẫn đóng gói Store coi là chặn | Bản Store không bao giờ xin UAC; chỉ đường tới bộ cài `*-machine.exe` | ✅ `ca4b63b` (rủi ro B7 còn ở `msix-submission.md` §5) |
| R2-08, R2-24 | P2/P1 | Thư mục stage trùng thư mục bộ cài Inno per-user → hai kênh đè/gỡ lẫn nhau | Thư mục riêng `TextVN-Store` | ✅ `ca4b63b` |
| R2-09 | P2 | Không bước nào kiểm chính file `.msix` (identity, version, encoding, ảnh, payload) | `tools/win/verify-msix.py` trong ci-shared và release | ✅ `b10be1c` |
| R2-10 | P2 | Hướng dẫn nộp MSIX sai | Viết lại `msix-submission.md` theo kiến trúc mới, ghi chú cho tester | ✅ `c605c9b` |
| R2-11 | P2 | Manifest khai `en-US` trong khi giao diện chỉ có tiếng Việt (10.7) | Giữ `en-US` để có listing tiếng Anh; mô tả tiếng Anh ghi rõ "UI is Vietnamese" (`msix-submission.md` §4) | 📝 |
| R2-12 | P2 | Binary chép ra ngoài gói không có chữ ký Authenticode (Store chỉ ký gói) | `build-release.ps1` ký binary **trước** khi `build-msix.ps1` đóng gói → bật SignPath là bản stage mang chữ ký | ⏳ SignPath |
| R2-13 | P3 | `build-msix.ps1` ghi manifest qua đường dẫn tương đối của .NET (bỏ qua `Set-Location`) | OutDir/TargetDir tuyệt đối | ✅ `b10be1c` |
| R2-14 | P3 | Chỉ có ảnh scale-100, không `resources.pri` → icon bị phóng to/viền màu ở Start | — | ⏸ |
| R2-37 | P2 | Bootstrap dừng ở lỗi chép đầu tiên rồi chạy tray từ `WindowsApps` | Lỗi chép → hộp thoại, không bao giờ chạy tray trong gói | ✅ `ca4b63b` |
| R2-73 | P1 | release.yml vẫn build MSIX identity placeholder, không gì chặn nộp nhầm | Repo variable `MSIX_IDENTITY_NAME` → `-RequireStoreIdentity` + `verify --require-store-identity`; manifest đọc UTF-8, Publisher/tên hiển thị theo Partner Center, ảnh 71×71 thật | ✅ `bcc452b`, `b10be1c` |

### 3.2 Bảo mật Windows

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-03, R2-15 | **P0** | Đăng ký TSF phạm vi máy (HKLM, mọi tài khoản nạp DLL) có thể trỏ vào thư mục người dùng ghi được | `register --scope machine` chỉ nhận DLL canonical dưới Program Files (known folder, không đọc biến môi trường), không dưới `WindowsApps`; tray bỏ `%LOCALAPPDATA%\Programs` khỏi gốc tin cậy | ✅ `6c7fecc` |
| R2-16 | **P0** | `uninstall.ps1` bản portable xoá đệ quy thư mục chứa nó — ZIP không có thư mục gốc nên giải nén vào Downloads rồi gỡ là mất cả Downloads | Chỉ xoá đúng file TextVN đã biết + bản `.old-*`, `rmdir` khi rỗng, file khoá hẹn xoá qua RunOnce; test đặt file người dùng và kiểm còn nguyên | ✅ `5bef5fa` |
| R2-17 | **P0** | Bộ cài `-machine.exe` nộp Store không bao giờ được ký, kể cả khi bật SignPath | Ký trong `build-release.ps1 -MachineInstaller` và release.yml | ✅ `31044df` |
| R2-18 | P1 | Binary x64 phụ thuộc `VCRUNTIME140.dll` (máy thiếu VC++ redist không nạp được TIP); MSIX không khai VCLibs | `+crt-static` cho cả 2 target MSVC + `tools/win/check-pe-imports.py` chặn trong build | ✅ `ae0a938` |
| R2-21 | P3 | Log CLI chạy quyền admin tạo thư mục trước khi kiểm reparse point (CR-22 chưa trọn) | Kiểm trước, tạo từng cấp, mở với `FILE_FLAG_OPEN_REPARSE_POINT`, xoay log 1 MiB | ✅ `6c7fecc` |
| R2-42 | P3 | Đề nghị UAC dò registry thay vì chờ tiến trình elevated | `ShellExecuteExW` + chờ + exit code | ✅ `6c7fecc` |

### 3.3 QA Windows (tray, CLI, bộ cài)

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-25 | P1 | Cài cho mọi người dùng: tài khoản khác không có TextVN trong danh sách bàn phím | Lần đầu tray chạy cho một tài khoản → `register` per-user | ✅ `7f5082c` |
| R2-26 | P1 | Trong app AppContainer (ô tìm kiếm Start, Calculator, app Store) TIP không đọc được state/config → luôn gõ Telex, bỏ qua V/E | — | ⏸ |
| R2-27 | P2 | Checkbox "Khởi động cùng Windows" chỉ thấy HKCU Run, bộ cài machine đặt HKLM | Đọc cả HKLM (view 64-bit); gỡ Inno xoá Run HKCU trỏ vào `{app}` | ✅ `13ba81f` |
| R2-28 | P2 | Menu "Bật tiếng Việt cho {app}" hiển thị bật trong khi TIP vẫn gõ tiếng Anh (toàn cục tắt) | Hiển thị đúng luật TIP (toàn cục AND override), mờ khi toàn cục tắt | ✅ `7f5082c` |
| R2-29 | P2 | Gỡ Inno: `ScheduleCleanupViaCli` chạy sau khi Inno đã xoá CLI | Chuyển sang `usUninstall`, đổi tên DLL khoá trước, chỉ hẹn xoá `.old-*` | ✅ `fe42d8c` |
| R2-30 | P2 | Gỡ một bản (portable/EXE/Store) dừng và gỡ đăng ký bản khác đang dùng | `unregister --if-owned-by <dir>`, `--stop --if-image-under <dir>` | ✅ `6c7fecc` |
| R2-31 | P2 | `config_version` đếm lại từ 1 mỗi lần tray khởi động → app chạy lâu bỏ lỡ thay đổi | Version khởi đầu duy nhất theo phiên tray | ✅ `7f5082c` |
| R2-32 | P2 | "Cài & bật TSF" chạy CLI đồng bộ trên thread UI giữ LL hook → treo hộp thoại, Windows có thể gỡ hook | Thread nền, khoá nút trong lúc chạy, kết quả về dialog bằng message. Menu "Gỡ cài đặt" vẫn đồng bộ vì tray thoát ngay sau đó | ✅ `395dd3b` |
| R2-33 | P2 | Mỗi lần đổi phiên bản: reset tuỳ chọn + unregister→register (mất kích hoạt, xếp lại bàn phím) — gồm CR-20 | Giữ tuỳ chọn; chỉ ghi lại đăng ký khi đã lệch | ✅ `7f5082c` |
| R2-34 | P2 | Bỏ "Dành Ctrl + Shift" vẫn ép kích hoạt TextVN sau mỗi Ctrl+Shift → không rời TextVN được | Chỉ kích hoạt profile khi Ctrl+Shift đã dành cho TextVN. Còn lại: V/E vẫn đảo khi Windows đổi bàn phím bằng cùng tổ hợp | ✅ `65beee8` |
| R2-35 | P2 | `register --scope machine` thoát trước khi ghi COM server 32-bit (WOW6432Node) | Ghi mirror WOW64 trước khi kết thúc | ✅ `6c7fecc` |
| R2-36 | P2 | Shortcut "Kiem tra he thong" mở console đóng ngay | `doctor --pause` | ✅ `7f5082c` |
| R2-38 | P3 | Cửa sổ "Từ điển EN" không qua `IsDialogMessage` (Tab chèn ký tự Tab) | Qua `IsDialogMessage` | ✅ `7f5082c` |
| R2-39 | P3 | Một HFONT dùng chung bị xoá khi đổi DPI → chữ hộp thoại khác hỏng trên màn hình lệch DPI | Cache một font cho mỗi DPI, xoá khi Bảng điều khiển bị huỷ | ✅ `1b29fa3` |
| R2-40 | P3 | Trả Ctrl+Shift khi gỡ không thống nhất giữa 3 đường; `Language Hotkey` không bao giờ được trả | Marker ghi giá trị Windows trước khi dành Ctrl+Shift (`freed` + `<tên>=<cũ>`); tray, `uninstall.ps1`, Inno chỉ trả mục vẫn là 3; test portable + installer kiểm về đúng `2` | ✅ `aa2796b` |
| R2-41 | P3 | Ctrl+Shift + chuột (click mở tab mới, kéo tạo shortcut) đảo V/E | — | ⏸ |
| R2-71 | P1 | Hồi quy CR-17: cài Inno im lặng (đường Store EXE) không dành Ctrl+Shift | Bỏ `skipifsilent` ở `--free-ctrl-shift` | ✅ `984b485` |
| R2-81 | P2 | Bước "Simulate Store validator" kiểm bản v0.2.27 đã phát hành, không phải bản vừa build | `-Setup` nhận file vừa build, gắn Zone.Identifier | ✅ `13ba81f` |
| R2-86 | P3 | Thiếu rc.exe/rc lỗi → binary mất VERSIONINFO + manifest mà build vẫn xanh | Bước "Verify PE metadata" fail khi CompanyName/FileVersion sai | ✅ `2f8b493` |

### 3.4 Linux, macOS

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-43 | P1 | macOS IMK: appdb rỗng truyền con trỏ khác NULL → FFI lỗi ở mọi phím → luôn BackspaceType | `StrategyResolver`: rỗng → NULL; preset `appdb.default.json` trong bundle; cache | ✅ `f2920f1` |
| R2-44 | P1 | macOS: đã tắt tiếng Việt thì Ctrl+Shift không bật lại được khi TextVN.app không chạy | `ViStateStore` chung tiến trình IMK; đồng bộ ở Snapshot kế | ✅ `f2920f1` |
| R2-45 | P1 | Linux: `FCITX_ADDON_DIRS` thiếu thư mục addon gốc của Fcitx5 | Luôn giữ thư mục gốc | ✅ `c91e5a2` |
| R2-46 | P1 | IBus `<exec>`/`<setup>` không trích dẫn → engine không chạy khi đường dẫn có khoảng trắng | Trích dẫn shell + escape XML | ✅ `c91e5a2` |
| R2-47 | P1 | Bundle id IM `vn.textvn.im` thiếu `.inputmethod.` theo mẫu IMK | Không đổi: chủ sở hữu báo phần macOS đã hoàn thành; đổi id = nguồn nhập mới với macOS (người dùng phải thêm lại, mất cấu hình). Nếu trên máy Mac sạch TextVN không hiện trong Input Sources thì đổi theo mẫu `.inputmethod.` kèm migration | 📝 |
| R2-20, R2-48, R2-83 | P2/P1 | macOS ký ad-hoc, `.pkg` không ký, không notarize | Developer ID + notarize + staple khi có secret `APPLE_*`; thiếu thì ký ad-hoc và ghi rõ | ✅ `f2920f1` · ⏳ secret Apple |
| R2-49 | P2 | macOS bỏ qua `non_preedit` lúc khởi động | Đọc khi tạo controller và mỗi lần activate; thiếu khoá = false ở cả IMK và app | ✅ `f2920f1` |
| R2-50 | P2 | Ô gõ tắt macOS không theo luật chung (cắt im lặng > 64 ký tự) | `MacroRules` cùng luật `macro_text.rs` | ✅ `f2920f1` |
| R2-51 | P2 | "Gỡ cài đặt TextVN…" thất bại im lặng với bản cài cho mọi người dùng | Hỏi quyền quản trị, chờ script, báo kết quả | ✅ `f2920f1` |
| R2-52 | P3 | AppStream metainfo nói sai trạng thái ký số / `non_preedit` | Sửa nội dung | ✅ `f2920f1` |
| R2-53 | P3 | GTK settings: đóng cửa sổ Gõ tắt xoá con trỏ cửa sổ Từ điển EN | Con trỏ riêng từng cửa sổ | ✅ `bb50a89` |
| R2-54 | P3 | `sudo install.sh --system` chạy gsettings/sửa profile với quyền root | Phạm vi hệ thống không đụng thiết lập người dùng | ✅ `c91e5a2` |

### 3.5 Tiếng Việt / engine

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-55 | P1 | Backspace hoàn tác phím cuối thay vì xoá ký tự cuối nhìn thấy (`ass`+⌫ → `á`, `tiếng`+⌫ → `tiêng`, xoá `được` cần 8 lần) | Xoá một ký tự đang hiển thị như UniKey, dời dấu thanh; `method::keys_for` dựng lại chuỗi phím để gõ tiếp vẫn đúng; test bất biến ngẫu nhiên 4 kiểu gõ. Corpus/test cũ mã hoá hành vi "fold-back" được cập nhật | ✅ `329e52e` |
| R2-56 | P1 | Auto-restore mặc định biến từ Việt thành tiếng Anh (`thí`, `hí`, `vơ`, `hót`) | Bỏ khỏi `en_common` các từ trùng âm tiết Việt thật; thêm âm tiết vào `vn_common` | ✅ `b5d859e` |
| R2-57 | P1 | Tab gợi ý tiếng Anh thay từ Việt (`có`+Tab → `cost`) | Như R2-56 | ✅ `b5d859e` |
| R2-58 | P1 | Auto-restore trả `đ` trong token không nguyên âm (`50.000đ`, `ĐT`, `đc`) | Giữ `đ` khi token không có nguyên âm | ✅ `76ef8cc` |
| R2-59 | P1 | Tự viết hoa sau mọi dấu `.` kể cả không có khoảng trắng (`google.Com`) | Đầu câu = `. ! ?` (+ dấu đóng) rồi khoảng trắng, hoặc Enter — đọc từ đuôi text, không giữ cờ riêng | ✅ `a21f178` |
| R2-60 | P1 | VNI/VIQR: dấu mũ/móc gõ cuối từ rơi sai nguyên âm | Chọn âm trên cả cụm nguyên âm cuối như UniKey (`toi6` → tôi, `nguoi72` → người, `luu7` → lưu); chỉ dời khi ra vần hợp lệ | ✅ `748f804` |
| R2-61 | P1 | Telex đơn giản không gõ được ă/ơ/ư | `aw/ow/uw` như UniKey Simple Telex; vẫn khác Telex: `ww` → `ww`, không có `w` đứng riêng → ư | ✅ `348afbf` |
| R2-62 | P1 | Thiếu luật thanh điệu với phụ âm tắc → `sort/port/part` thành âm tiết không tồn tại | Luật 6: âm cuối `c ch p t` chỉ mang sắc/nặng → auto-restore trả lại `sort`, `keep`, `chart`…; `text/next/art/cart` vẫn giữ làm gợi ý Tab | ✅ `3e936fe`, `283fd35` |
| R2-63 | P2 | Bấm phím dấu lần 3 bỏ dấu nhưng nuốt phím (`ooo` → `o`) | Lần ba gỡ dấu và gõ chữ đó: `xooong` → xoong, `ddd` → dd, `uww` → uw | ✅ `f58ab77` |
| R2-64 | P2 | Luật `iet` thêm dấu mũ không ai gõ (`Viet` → `Viêt`) | Bỏ luật; `Viet`, `quiet`, `KIET` giữ nguyên | ✅ `2550c95` |
| R2-65 | P2 | Telex `w` sau cụm nguyên âm rơi sai nguyên âm (`muaw` → `muă`) | Dùng chung logic cụm của R2-60: `muaw` → mưa, `voiws` → với | ✅ `aaf6c58` |
| R2-66 | P2 | Không có `w` đứng riêng → `ư` | `nhw` → như, `ddwngf` → đừng; `ww` → w; không sau `q`. Thêm luật vần mở không có âm cuối để bảo vệ tiếng Anh (`using/music/during`). Hệ quả biết trước: `w` + chữ số (`w3c`) → `ư3c` như UniKey — Escape trả lại | ✅ `8ca1e6d`, `902fa04` |
| R2-67 | P2 | VIẾT HOA bằng Shift chỉ biến đổi một nửa | Giữ nguyên: chữ hoa gõ bằng Shift (S F R X J W Z) là chữ thường để `USA`, `JSON` không bị biến đổi (user-guide §3, corpus `telex_caps_lock_01`, e2e IBus/Fcitx5); Caps Lock cho hành vi UniKey | 📝 |
| R2-68 | P2 | `d` gõ sau trong từ không tạo `đ` đầu từ | `duocjwd` → được, `dieend` → điên; chỉ khi từ đã có dấu Việt hoặc âm cuối hợp lệ (`did/dad/dead` giữ tiếng Anh) | ✅ `5c9c50c`, `754d0ee` |
| R2-69 | P2 | VNI `0` xoá mọi dấu thay vì chỉ thanh điệu | `0` chỉ gỡ dấu thanh như Telex `z` (`đường`+0 → đương); không có thanh thì `0` là chữ số (`0912`) | ✅ `35af989` |
| R2-70 | P3 | Không chuyển qua lại giữa dấu mũ/móc (`toow` → `tôw`) | `toow` → tơ, `awa` → â, `aaw` → ă, VNI `o67` → ơ | ✅ `b1a5917` |

### 3.6 CI, ký số, phát hành

| ID | Mức | Vấn đề | Xử lý | Trạng thái |
|---|---|---|---|---|
| R2-72 | P1 | Công cụ SignPath gọi route/trường không tồn tại, coi `WaitingForApproval` là lỗi | Viết lại theo REST API công bố: multipart `SigningRequests`, poll đến `Completed`, tải `SignedArtifact`, nhiều file = một ZIP deep sign = một lần duyệt, kiểm `Get-AuthenticodeSignature` | ✅ `31044df` |
| R2-74 | P2 | Ký GPG/cosign fail-open: thiếu secret vẫn phát hành bản không ký | Trong CI thiếu/sai khoá hoặc thiếu cosign → dừng; tự `gpg --verify` từng `.asc` | ✅ `f1568cd` |
| R2-75 | P2 | Fuzz `ffi_key` báo động giả với phím injected | Mô hình document bỏ qua phím injected; seed mới | ✅ `984b485` |
| R2-76 | P2 | Cask Homebrew giữ sha của v0.2.18 suốt 9 bản → `brew install` lỗi checksum | sha 0.2.27 (đối chiếu `SHA256SUMS` có chữ ký); publish cảnh báo khi lệch | ✅ `de3e1a7` |
| R2-77 | P2 | Tag phát hành có thể trỏ commit chưa qua CI | Release tự chạy sideload MSIX trên chính gói sắp phát hành; quy trình B3/B5 vẫn đòi CI xanh trước khi tag | ✅ `f31f574` |
| R2-78 | P2 | Chạy lại tag để lẫn asset cũ trong draft | Xoá asset cũ trước khi upload | ✅ `6e19caa` |
| R2-79 | P2 | Test sanitizer linux-common không job CI nào chạy | Job ASan + UBSan | ✅ `fe454a7` |
| R2-80 | P2 | `check-win-corpus` chỉ chạy trong ci-macos (bộ lọc path bỏ sót) | Thêm vào ci-shared | ✅ `fe454a7` |
| R2-82 | P2 | Release notes ghi "Unsigned release candidate" trái với thực tế đã ký GPG + Sigstore | Ghi đúng phần đã ký và phần còn thiếu | ✅ `de3e1a7` |
| R2-84 | P3 | Clippy target Windows cho `textvn-win-hook` không chạy trong CI | Job clippy cross-check Windows | ✅ `fe454a7` |
| R2-85 | P3 | Build phát hành không tái lập được: toolchain `stable` trôi, không `--locked`, Inno không ghim | Build release `--locked` (Windows/Linux/macOS), CI kiểm Cargo.lock khớp. Chưa ghim toolchain/Inno | ✅ một phần `f15a9c9` · ⏸ phần còn lại |
| R2-87 | P3 | VirusTotal bỏ qua 2 file nộp Store | Quét cả `.msix` và `-machine.exe`, sau khi build | ✅ `6e19caa` |

Vòng 1 cập nhật theo: **CR-20** (a) đã sửa cùng R2-33; **CR-34** (`appdb_version` số) đã sửa
ở `017bc82` (xem `code-review-2026-10-07.md`).

## 4. Quyết định thay chủ sở hữu

| Quyết định | Lý do | Đổi được ở |
|---|---|---|
| Version MSIX = `(A+1).B.C.0` | Partner Center từ chối phần đầu = 0; ánh xạ đơn điệu nên bản sau luôn lớn hơn. Cài đặt Windows hiển thị 1.2.27.0 khác số trong app | `tools/win/build-msix.ps1`, `verify-msix.py` |
| Giữ MSIX với kiến trúc relay + stage-out | Cách duy nhất để TIP (DLL nạp vào mọi app) hoạt động từ gói Store; đã chứng minh trên runner | — |
| Bản Store hỏi trước khi dành Ctrl+Shift | Chính sách 10.2.8 (không đổi thiết lập hệ thống khi chưa được đồng ý) | `apply_ctrl_shift_default_once` |
| Bản Store không bao giờ xin UAC | Hướng dẫn đóng gói Store; cần cài cho mọi người dùng → `*-machine.exe` | `offer_machine_registration_if_needed` |
| Đăng ký phạm vi máy chỉ từ Program Files | Chặn leo thang đặc quyền (P0) | `cli/src/register.rs` |
| Nâng cấp giữ tuỳ chọn người dùng | Người dùng VNI phải chọn lại sau mỗi bản là lỗi, không phải tính năng | `tray/src/svc.rs` |
| Override theo app chỉ **tắt** được khi toàn cục bật | Đúng luật TIP đang chạy; menu không hứa điều TIP không làm | `tray/src/menu.rs` |
| macOS: thiếu khoá `non_preedit` = false (gạch chân) | Khớp CHANGELOG 0.2.27 và Windows/Linux | `ConfigModel.swift` |
| Bỏ khỏi `en_common` các từ trùng âm tiết Việt (`this`, `his`, `host`…) | Auto-restore bật mặc định; gõ tiếng Việt đúng ưu tiên hơn khôi phục từ tiếng Anh | `data/en_common.txt` |
| Ctrl+Shift khi Windows còn giữ: không ép kích hoạt TextVN | Tôn trọng lựa chọn "trả lại cho Windows" | `tray/src/main.rs` |
| Backspace xoá ký tự cuối đang hiển thị (như UniKey/EVKey/OpenKey), không hoàn tác phím cuối | Hành vi cũ thêm dấu khi xoá (`ass`+⌫ → `á`) và cần nhiều lần ⌫ hơn số chữ; không đặc tả nào yêu cầu "hoàn tác phím". Corpus/test cũ mã hoá hành vi cũ được cập nhật | `core/src/lib.rs::on_backspace` |
| Tự viết hoa chỉ sau `. ! ?` + khoảng trắng hoặc Enter | `google.com`, `3.5 kg`, `file.txt` không phải đầu câu. `v.v. ` + chữ vẫn viết hoa (không phân biệt được viết tắt) | `core/src/post/caps.rs` |
| Telex: `w` đứng riêng → `ư` (như UniKey), chữ hoa gõ bằng Shift vẫn là chữ thường (R2-67 không đổi) | Người quen UniKey gõ `nhw`, `tw`; `USA`/`JSON` không bị biến đổi. Đổi lại `w` + chữ số (`w3c`) ra `ư3c` — Escape trả lại | `core/src/method/telex.rs` |

## 5. Hoãn — lý do và hướng làm

| ID | Việc | Vì sao chưa làm |
|---|---|---|
| R2-26 | Kênh state/config đọc được từ AppContainer (pipe có ACE `ALL APPLICATION PACKAGES` + nhãn low-integrity, hoặc file state trong thư mục có ACL `S-1-15-2-1`) | Mở bề mặt tấn công của pipe cho mọi app sandbox — cần thiết kế bảo mật riêng và máy Windows thật để kiểm |
| R2-41 | Bỏ qua chạm Ctrl+Shift khi có thao tác chuột | Tray thêm được `WH_MOUSE_LL`, nhưng TIP (trong tiến trình app) không thấy chuột và vẫn tự đảo V/E — cần tray báo "vừa có click" cho TIP qua bộ nhớ chung: thay đổi giao thức |
| R2-14 | Ảnh scale-200/targetsize + `resources.pri` (makepri) | Thẩm mỹ; PRI sai làm mất logo — cần kiểm trên Windows thật |
| R2-85 (phần còn lại) | Ghim toolchain Rust và Inno Setup | Chủ sở hữu chọn version; ghim lệch máy dev sẽ đỏ clippy |
| (mới) | Vần mở `ă`/`â` không có âm cuối (`aws` → `ắ` không được trả lại) | Hành vi có từ trước; bộ kiểm âm tiết còn dùng giữa từ (R2-60/66) — thêm luật cần đo lại toàn bộ corpus tiếng Anh |
| CR-08, CR-10, CR-14, CR-20 (b) | Như vòng 1 | Như vòng 1 (CR-34 và CR-38 đã xong ở vòng này — `017bc82`, `c31d54c`) |

## 6. Gate cục bộ (container Linux)

| Gate | Kết quả |
|---|---|
| `cargo fmt --all --check` · clippy Linux + Windows cross-check `-D warnings` | ✅ |
| `cargo test --workspace` | ✅ 416 test |
| replay headless · tsf · mac · linux | ✅ 55 · 133 · 169 · 55 |
| xtask `check-tables` · `check-win-corpus` · `check-mac-corpus` · `check-mac-targets` · `check-version-sync` | ✅ |
| repo-hygiene `check_doc_links` · `check_iss_tabs` · `check_ps1_ascii` · `check_no_injection_apis` | ✅ |
| `verify-msix.py` trên gói 0.2.27 đã phát hành | 8 lỗi (đúng kỳ vọng) |
| `check-pe-imports.py` trên binary 0.2.27 | bắt 3 file x64 phụ thuộc VCRUNTIME140 (đúng kỳ vọng) |

Swift, PowerShell, Inno và MSIX chỉ kiểm được trên CI (§2).
