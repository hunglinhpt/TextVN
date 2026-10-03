# Changelog

Tất cả thay đổi đáng chú ý của dự án TextVN sẽ được ghi lại ở đây.

Định dạng dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
và dự án này tuân thủ [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.17] — 2026-10-03

### Fixed (Windows) — Package validation của Microsoft Store (B13)
- **Bộ cài mặc định PER-USER** (`PrivilegesRequired=lowest`): validator của Store
  chạy installer trong sandbox không auto-elevate; bản cài đòi admin (0.2.16)
  làm 3 check đỏ — "Silent install check: could not identify…", "Entry in add or
  remove programs" và "Bundleware check" (không đọc được entry) → nút Submit
  mờ. Per-user không cần UAC nên silent install qua validator sạch; muốn cài
  phạm vi máy dùng `/ALLUSERS` (Inno tự xin elevation) — KHÔNG thêm vào ô
  switches của Store.
- **Silent install luôn exit 0** khi đã chép đủ file: trước đây đăng ký TSF
  lỗi trong sandbox → exit 10 → Store coi là "cài thất bại". Nay app TỰ đăng ký
  ở lần chạy đầu (0.2.16: tray tự đề nghị UAC một lần nếu Windows từ chối
  per-user) — exit 10 chỉ còn cho cài tương tác.
- **Prompt đăng ký máy của tray chạy cả khi `--autostart`**: tránh trường hợp
  máy từ chối per-user + người dùng không bao giờ mở app → gõ hỏng IM LẶNG
  (vẫn guard: một lần duy nhất + bỏ qua khi elevated).
- Theo [manual package validation của Microsoft](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/manual-package-validation):
  đã chạy đủ 3 bước tại máy thật (silent install không tương tác → entry
  Add/Remove đúng Name/Publisher/Version → đúng 1 entry).

## [0.2.16] — 2026-10-03

### Fixed (Windows) — portable "giải nén chạy ngay" hoạt động trên Win11 24H2+
- **Tray tự đề nghị đăng ký phạm vi máy khi Windows từ chối per-user** (B7):
  phát hiện `activate` thất bại (chưa có HKLM) → hộp thoại một lần
  "Đăng ký phạm vi máy để gõ được ngay? (UAC một lần)" → chạy
  `textvn-cli register --scope machine` ELEVATED → chờ HKLM → register lại
  per-user → activate. Không hỏi khi `--autostart` (không làm phiền lúc đăng
  nhập) và bỏ qua nếu máy đã có đăng ký máy.
- **Tự bật lại chế độ TIẾNG VIỆT sau khi đăng ký máy**: bẫy phát hiện khi
  chẩn đoán live — TIP đúng + activation OK nhưng `global_enabled=false`
  (EN mode, ví dụ do người dùng bấm Ctrl+Shift thử trước đó) cũng cho ra chữ
  raw, khiến "đăng ký thành công" mà vẫn tưởng hỏng.

### Verified live trên máy thật (máy chủ repo, build 26300 — trường hợp từng fail)
- Cài portable 0.2.15 → tray mới hiện prompt → OK + UAC Yes → HKLM COM trỏ
  DLL trong thư mục portable; `ActivateProfile(VI, session) → OK`;
  HKL Notepad = `0x042A`; gõ thật `dduocj text ` → **`được text ` PASS**.
- Đây là lần đầu tiên đường portable đứng một mình gõ được trên máy này.

## [0.2.15] — 2026-10-03

### Fixed (Windows)
- **Gỡ TextVN không còn để lại ghost trong Win+Space** (B11): `unregister` chỉ
  xoá COM + CTF keys, không đụng "danh sách ngôn ngữ hiện đại" — sau khi gỡ,
  TextVN vẫn hiện trong Win+Space nhưng chọn vào là chết. Giờ
  `remove_modern_language_list()` xoá đúng entry (giữ ngôn ngữ) + broadcast
  `WM_SETTINGCHANGE("International")` chuẩn (SendMessageTimeout + lParam
  "International" thay cho PostMessage lParam=0). **Đã verify 2 chiều trên
  máy thật**: unregister → TextVN biến mất khỏi cả `vi` lẫn `en-US`;
  register → trở lại đủ.
- **Hết DLL mồ côi sau khi gỡ** (B12 — "tại sao không tắt TSF rồi xoá DLL?":
  DLL do TSF của Windows nạp vào mọi app đang mở, không phải service TextVN,
  không thể tắt; Windows cấm xoá file đang map):
  - Installer: `usPostUninstall` → `DeleteFile` trước, trượt thì ghi
    `PendingFileRenameOperations` (chuẩn Windows cho mọi installer) cho DLL +
    `*.old-*` + thư mục — không dùng `MoveFileExW` qua external import vì
    Pascal Script không có kiểu `Pointer` để truyền NULL (ISCC từ chối).
  - Sửa luôn bug `[UninstallDelete]` chứa **TAB thay vì `\`** trong pattern
    (`{app}<TAB>extvn-tsf.dll.old-*`) — chưa bao giờ khớp; thêm pattern
    `TextVN.exe.old-*`.
  - Portable: `uninstall.ps1` khi xoá trượt sẽ ghi
    `HKCU\...\RunOnce\TextVNCleanup = cmd.exe /c rmdir /s /q "<dir>"` —
    tự dọn ở lần đăng nhập kế tiếp (không cần admin). Hướng dẫn portable đã
    đổi thành "tự dọn, không cần làm gì thêm".
  - **Verify máy thật**: giữ DLL bằng `LoadLibraryEx` (mô phỏng app đang mở)
    → `uninstall.ps1` phát hiện khoá + ghi RunOnce đúng; nhả khoá + chạy đúng
    lệnh RunOnce → toàn bộ file bị xoá.

### Verified — toàn bộ luồng portable "chạy ngay" (yêu cầu chủ repo)

| Hạng mục | Kết quả |
|---|---|
| Giải nén ZIP (13 file: exe/cli/dll/scripts/hướng dẫn/report) | ✓ đủ |
| `TextVN.exe` first-run tự đăng ký | ✓ COM→folder giải nén, CTF Enable=1, TIP registry OK |
| TextVN xuất hiện trong Win+Space (cả en-US lẫn vi) | ✓ (0.2.12 fix — lỗi "mất khỏi danh sách" không tái diễn) |
| Tray + IPC + doctor | ✓ đang chạy, pipe lắng nghe |
| `uninstall.ps1` | ✓ gỡ COM/CTF sạch; ghost modern-list — đã fix trong bản này |
| `activate` (kích hoạt tự động) | ✗ E_FAIL — **B7 tái diễn đúng như tài liệu**: build 26300 từ chối per-user |
| Gõ thật trong Notepad (chọn qua Win+Space) | ✗ không lần nào ra TextVN (chỉ MS-Telex) — portable-only KHÔNG đủ trên build này; **bộ cài phạm vi máy là đường duy nhất đã chứng minh gõ được** |
| Lỗi cũ khác (ghost COM folder xoá / mất Win+Space / autostart ghost) | ✓ không tái diễn |

Hướng dẫn portable (`HUONG_DAN_SU_DUNG.txt`) đã nêu rõ: máy Win11 24H2+ nên
dùng bộ cài.

## [0.2.14] — 2026-10-03

### Fixed (Windows)
- **DPI theo màn hình — hết cắt chữ**: dialog dùng `GetDeviceCaps(LOGPIXELSX)`
  (system-DPI cũ 96) trong khi manifest PerMonitorV2 render theo DPI monitor
  thật (192 ở scaling 200%) → chữ to gấp đôi, control tràn, nhãn bị "nuốt"
  ("…gõ sa"). Giờ: `GetDpiForSystem` + sau khi tạo cửa sổ đọc
  `GetDpiForWindow` — khác thì resize đúng cỡ rồi mới layout; áp cho cả 3
  cửa sổ (Bảng điều khiển, Gõ tắt, Từ điển EN). Đã xác minh bằng ảnh chụp
  DPI-aware: 1226px đúng cho màn 200%, đủ nhãn (B10).
- **Nhãn dài không bao giờ bị cắt**: checkbox "Khôi phục từ tiếng Anh khi gõ
  sai" trở lại full bề rộng cột; nút "Từ điển EN..." chuyển xuống HÀNG NÚT
  dưới như Linux (B9 + quy tắc mới trong ui-spec §1).

### Changed — UI thống nhất 3 nền tảng (yêu cầu chủ repo: "buộc phải giống nhau")
- macOS: nhãn khớp từng chữ với Windows/Linux ("Khôi phục từ tiếng Anh khi
  gõ sai", "Gõ tắt cả khi tắt tiếng Việt", "Gõ tắt...", "Từ điển EN..."),
  cùng thứ tự tuỳ chọn; thêm toggle "Bật gõ tiếng Việt" (đọc/ghi trạng thái
  toàn cục trong AppDelegate — cùng nguồn với menu bar); nút "Từ điển EN..."
  cạnh "Gõ tắt...".
- Icon: macOS menu bar đổi tint V/E sang `systemPink`/`systemBlue` khớp hệ màu
  badge V(#C2185B)/E(#0288D1) của Windows/Linux (ui-spec §7 mới).
- Test `labels_match_linux_settings_panel` mở rộng: đối chiếu nhãn
  Windows ↔ Linux ↔ macOS (include SettingsView.swift) — CI chặn nếu lệch.

### Tests
- 20 bước preflight (thêm `ascii-ps1`); 359+ test toàn workspace.

## [0.2.13] — 2026-10-03

Bản hoàn thiện đa nền tảng cho tính năng tự xác định EN/VI (theo yêu cầu
"đã hoàn thiện cho cả Linux và macOS chưa").

### Added
- **"Từ điển EN..." trên macOS và Linux** (parity với Windows từ 0.2.9):
  - macOS: Settings → "Từ điển EN..." — sheet quản lý danh sách như bảng gõ tắt.
  - Linux: cửa sổ Cài đặt → nút "Từ điển EN..." — cửa sổ soạn thảo như "Gõ tắt...".
  - FFI mới `ime_settings_english_words_text` / `ime_settings_set_english_words_text`
    (dùng chung quy tắc chuẩn hoá trim/lowercase/ASCII/dedupe của cả 3 nền tảng),
    có test round-trip + kiểm tra header tự động.
- **CI phủ Linux adapter**: `replay corpus/shared --adapter linux` được thêm vào
  job replay (trước đây chỉ headless/tsf/mac) + vào `cargo xtask preflight`
  (19 bước) — các case EN-detect giờ regression-test cả nền tảng Linux.

### Fixed
- **macOS xoá `english_words` mỗi lần lưu config**: `ConfigModel.swift` thiếu
  trường này nên `store.persist()` ghi lại config sẽ vứt bỏ danh sách từ của
  người dùng (bắt khi đồng bộ 0.2.13). Đã thêm vào model + decode.

### Documented
- `docs/release/parity-checklist.md`: hàng "Từ điển EN" ✓ cả 3 nền tảng + vị
  trí UI từng nơi; bỏ `english_words` khỏi mục "chưa có trên UI".
- `installer/windows/tests/test-portable.ps1`: ghi chú B7 trung thực — CI chạy
  elevated nên kịch bản portable KHÔNG chứng minh được typing với đăng ký
  thuần per-user; Windows 11 24H2+ nên dùng bộ cài phạm vi máy.
- `docs/user-guide.md`: vị trí "Từ điển EN..." trên từng nền tảng.

## [0.2.12] — 2026-10-03

### Fixed (Windows)
- **Bộ cài nâng cấp không còn lỗi "DeleteFile failed; code 5"** khi
  `textvn-tsf.dll` đang được TSF nạp trong tiến trình (explorer/Notepad…):
  installer đổi tên DLL cũ thành `textvn-tsf.dll.old-<thời-gian>` trước khi
  ghi file mới (`PrepareToInstall` → `RenameLockedTsfDll`; Windows cho phép
  rename file đang map dù không cho xoá) + `[UninstallDelete]` dọn file `.old-*`.
- **TextVN không còn biến mất khỏi danh sách Win+Space**: `register` ghi profile
  vào **store ngôn ngữ hiện đại** (`HKCU\Control Panel\International\User
  Profile\<tag>`) + broadcast `WM_SETTINGCHANGE("International")` — nguyên
  nhân gốc vụ "không chọn được TextVN để gõ" sau các chu kỳ unregister/register
  (bắt thật máy chủ repo 2026-10-03).
- **Chẩn đoán activate trung thực**: `textvn-cli activate` kiểm tra profile
  ĐANG CHỌN (`GetActiveLanguageProfile`) thay vì chỉ "được phép dùng"; thử ma
  trận VI/EN × cờ; thông báo lỗi nêu đúng việc cần làm (chọn bằng Win+Space
  hoặc cài phạm vi máy).
- Tài liệu: `docs/user-guide.md` + `HUONG_DAN_SU_DUNG.txt` ghi rõ **giới hạn
  per-user trên Windows mới** (một số build 10/11 từ chối ActivateProfile cho
  đăng ký chỉ-HKCU → bản portable đơn thuần có thể gõ được sau khi chọn bằng
  Win+Space, còn muốn cắm-là-chạy nên dùng bộ cài phạm vi máy). CI luôn test
  luồng /ALLUSERS; kịch bản per-user chỉ kiểm tra file/trạng thái đăng ký.

### Known issue (ghi nhận từ máy chủ repo, 0.2.12 chưa xử lý được)
- `ActivateProfile` trả `E_FAIL` cho TIP chỉ đăng ký per-user trên Windows
  build 26300 (24H2+/Insider): không thể chuyển bộ gõ bằng API — phải chọn
  thủ công lần đầu (Win+Space) hoặc dùng bộ cài phạm vi máy. Theo dõi ở
  `docs/specs/win-test-common-errors.md` (B7).

## [0.2.11] — 2026-10-03

### Fixed
- `textvn-cli activate` (đường toggle của tray) **check-then-activate**:
  profile đang bật mà blind-call `ActivateProfile` trả E_FAIL (0x80004005)
  trên một số build Windows (bắt thật trên máy chủ repo) → giờ kiểm tra
  enabled trước, đã active = exit 0; sau lỗi còn verify lại một lần nữa.
- **Quy trình**: publish job thêm thông báo lỗi RÕ khi `RELEASE_REPORT.json`
  lệch tag/version — sự cố tag v0.2.11 đầu tiên bị push mà không bump version
  khiến publish fail im lặng (jq -e không in gì); R4 ghi vào
  `docs/release/release-process.md`.

## [0.2.10] — 2026-10-03

Bản vá theo phản hồi trực tiếp của chủ repo trên 0.2.9.

### Fixed
- **"Đã chuyển sang tiếng Anh mà vẫn gõ tiếng Việt"**: khi bấm Ctrl+Shift
  TRONG KHI bộ gõ active trong app là bàn phím khác (Microsoft Việt / US
  trong Win+Space), trước đây chỉ đổi mode + icon của TextVN — thứ người
  dùng gõ vẫn do bàn phím cũ quyết định. Giờ mỗi lần toggle bằng hotkey,
  tray chạy `textvn-cli activate` kích hoạt **profile TextVN cho phiên**:
  mode đổi → bộ gõ đổi → chữ gõ ra theo icon ngay lập tức (đúng nghĩa
  chuyển mode của UniKey).
- **State tray ↔ TIP không còn diverge**: toggle từ TIP gửi giá trị tính từ
  state cục bộ — nếu lệch với tray (bấm đôi trong 250ms, tray restart…)
  thì giá trị sai có thể "kẹt" mãi. Tray giờ trả **snapshot state chuẩn**
  cho lệnh toggle (kể cả khi bị debounce bỏ qua) và TIP nhận lại; thêm
  debounce 250ms phía TIP khớp cửa sổ phía tray.
- **Nâng cấp app mới không còn bị app cũ ảnh hưởng** (báo cáo lớn nhất):
  đổi phiên bản (ghi nhãn `last_version` trong `state.json`) kích hoạt
  migration lúc khởi động — reset mọi tuỳ chọn về mặc định, xoá per-app
  overrides, bật lại tiếng Việt, **unregister + register lại TSF** rửa sạch
  key của phiên bản cũ. GIỮ nội dung người dùng: **từ điển EN tự thêm**
  (`english_words`), gõ tắt (`macros`), emoji. Lần chạy đầu (chưa có nhãn)
  không reset gì.

### Documented
- `docs/release/store-submission.md` §6 viết lại: Partner Center giờ chỉ
  nhận gói **`.exe` / `.msi`** (không còn MSIX cho loại submission này);
  chứng thư đến từ luồng chứng nhận của Store, cert riêng (SignPath /
  mua) chỉ là tuỳ chọn cho phân phối trực tiếp.

### Tests
- Tray: `version_change_migrates_and_preserves_user_content`,
  `same_version_does_not_migrate` (3×); `toggle_global_responds_with_
  authoritative_snapshot`. TSF: `hotkey_toggle_claim_debounces` (3×).

## [0.2.9] — 2026-10-02

Bản chốt: người dùng tự mở rộng từ điển, quy tắc ưu tiên ngôn ngữ công khai,
kiểm soát release chặt hơn. Tiếp 0.2.8.

### Added
- **"Từ điển EN..." trong Bảng điều khiển**: thêm/xoá từ tiếng Anh cá nhân
  (`config.english_words`) bằng UI — từ trong danh sách thắng cả lớp bảo vệ
  âm tiết Việt thông dụng (quyết định tường minh). Editor cửa sổ riêng, DPI
  đa màn hình như bảng chính; lưu ngay → TSF reload hot.
- `cargo xtask preflight` (+ alias `cargo xtask`): **17 gate mirror CI** chạy
  một lệnh, fail-fast, exit code thật — Checklist A0 mới trong
  `docs/release/release-process.md`; bảng "bước → validate gì → sự cố thật"
  và 3 sự cố mới (R1 pipe che exit code, R2 baseline chết theo runner,
  R3 alias xtask).

### Documented
- **[language-detection.md](docs/specs/language-detection.md)**: ma trận xung
  đột ngôn ngữ đầy đủ — thứ tự ưu tiên mỗi phím (secure → chord → macro →
  mode → gợi ý → restore → Escape), nguyên tắc "mode đang bật thắng cặp mơ
  hồ", cách đổi ý từng trường hợp.
- README: mục "Điểm vượt trội so với các bộ gõ khác" (12 dòng so sánh có
  cơ sở với UniKey/EVKey/OpenKey).
- `docs/release/store-submission.md`: quy trình publish Store từng bước
  (reserv name → submission → certification) + 2 phương án tích hợp chứng
  thư số (SignPath Foundation hoặc mua riêng) và checklist sau khi ký.

### Tests
- Engine: ưu tiên user-dictionary thắng vn_common; `auto_restore_english=false`
  giữ fold kiểu UniKey. Tray: `normalize_word_list` (trim/lowercase/comment,
  khử trùng lặp, rỗng). UI smoke ×3: mở editor, mở lại vẫn 1 cửa sổ, Lưu ghi
  đúng `english_words` vào config và broadcast reload.

## [0.2.8] — 2026-10-02

Bản tính năng: xác định ngôn ngữ khi gõ, đa màn hình DPI, chuẩn bị nộp
Microsoft Store. Thông tin phát triển đổi thành **LinhBH.CoM**.

### Added
- **Tự xác định tiếng Anh / tiếng Việt** (báo cáo: "gõ tiếng Việt xong rồi gõ
  tiếng Anh bên cạnh thì sai"): từ điển EN thông dụng dựng sẵn
  (`data/en_common.txt`) chốt ở ranh giới từ — Space trả lại đúng từ EN thay
  vì fold kiểu `text`→`tẽt`, `is`→`í`, `saw`→`să`, `water`→`watẻ`… Cặp mơ hồ
  hai chiều (`cow` = cách gõ Telex của `cơ`, `sex`→`sẽ`, `queen`→`quên`…) ưu
  tiên tiếng Việt qua lưới `data/vn_common.txt` — đúng hành vi UniKey. Tắt:
  `auto_restore_english=false`.
- **Tab gợi ý hoàn tất từ EN**: từ đang gõ đã biến đổi và là tiền tố của một
  từ EN thông dụng → Tab hoàn tất thành từ đầy đủ (macro vẫn ưu tiên; từ
  chưa biến đổi nào Tab đi qua như cũ).
- **Ctrl+Z / Escape khi gõ sai**: Escape là restore thủ công sẵn có từ trước
  (giờ có case corpus riêng); Ctrl+Z vẫn là undo của ứng dụng (TextVN phát
  text thường nên undo app hoạt động tự nhiên).
- Hỗ trợ **cài im lặng cho Microsoft Store**: `PrivilegesRequiredOverridesAllowed`
  trong bộ cài Inno Setup — `/CURRENTUSER` giờ là cờ thật (không cần UAC);
  hướng dẫn nộp Store: `docs/release/store-submission.md`; CI đã chạy cả hai
  kịch bản silent (per-user + machine) từ trước.
- `PRIVACY_POLICY.txt` ở gốc repo (song ngữ EN/VI) — link cho phần khai báo
  Store, chủ repo tự cập nhật khi có thay đổi.

### Fixed
- **Dialog DPI đa màn hình**: kéo Bảng điều khiển (và cửa sổ Gõ tắt) sang màn
  hình có DPI khác giờ tự resize + re-layout + tạo lại font (`WM_DPICHANGED`
  — trước đây kích thước giữ nguyên như màn cũ → tràn/sai lệch).

### Changed
- Thông tin phát triển / publisher / CompanyName / copyright hiển thị:
  `hunglinhpt` → **`LinhBH.CoM`** (URL repo `github.com/hunglinhpt/TextVN`
  và ID `io.github.hunglinhpt.textvn` giữ nguyên).

## [0.2.7] — 2026-10-02

Bản vá theo báo cáo **Ctrl+Shift trên bản 0.2.5/0.2.6 không đổi được mode và
icon khay**.

### Fixed (Windows)
- **Ctrl+Shift hoạt động toàn cục như UniKey**: trước đây toggle chỉ xử lý
  trong TIP — khi TextVN không phải bộ gó active (user đứng ở bàn phím
  US/Microsoft Việt trong danh sách Win+Space) thì không có gì nhận tổ hợp.
  Tray giờ cài `WH_KEYBOARD_LL` **chỉ quan sát** (không ăn phím, không inject —
  mọi event đi tiếp qua `CallNextHookEx`) để dò Ctrl+Shift tap ở mọi app, đồng
  bộ icon `[V]`/`[E]` tức thì.
- **Chống toggle đôi (E11)**: một lần bấm tới tray qua hai đường (LL hook +
  TIP in-process) — `try_claim_global_toggle()` khoá cửa sổ 250ms chéo nguồn,
  nguồn sau chỉ nhận Ack + broadcast state hiện tại; cả hai nguồn tính cùng
  giá trị từ cùng state nền nên không lệch.

## [0.2.6] — 2026-10-02

Bản vá khẩn sau báo cáo **"không gõ được tiếng Việt (kể cả admin) + danh sách
bàn phím quá nhiều layout"** trên bản 0.2.5.

### Fixed (Windows)
- **Ghost registration — nguyên nhân thật của "không gõ được"**: COM
  `InprocServer32` trỏ vào thư mục portable **đã bị xoá** → TSF không nạp được
  TIP, admin cũng không cứu được (HKCU override HKLM). Chạy lại `TextVN.exe`
  từ thư mục portable **còn tồn tại** sẽ tự sửa COM sang đường dẫn mới
  (`tsf_registration_is_current` → register lại); `register status`/`doctor`
  báo `FILE MISSING` đúng chỗ.
- **Danh sách bàn phím — giữ nguyên VI + EN (thiết kế đúng)**: hai lần thử
  thay đổi trong 0.2.6-dev đều vỡ: (a) gỡ layout trước khi thêm lại làm
  Windows deactivate TIP cho phiên → gõ raw; (b) bỏ layout EN mặc định làm
  app có ngôn ngữ nhập en-US không còn nhận TextVN → gõ raw. Cả hai đã revert
  và chốt comment tại chỗ; Win+Space hiển thị 4 layout (US · VI Microsoft ·
  VI TextVN · EN TextVN) là **thiết kế**: profile EN cần cho ngữ cảnh nhập
  tiếng Anh, có thể tự ẩn bớt trong Settings → Typing nếu muốn.

## [0.2.5] — 2026-10-02

Bản vá cho **bản portable** — ba fix của 0.2.4 vận hành đủ trên bản cài nhưng
còn lỗ trên portable (đã repro thật trên máy không admin). Kèm pipeline ký số
SignPath (opt-in).

### Fixed (portable)
- **Gạch chân không tắt được trên portable**: `InstallLayoutOrTip` viết lại cây
  CTF per-user và **xoá mất toàn bộ key `Category`** ghi trước nó (repro:
  sau đăng ký chỉ còn `LanguageProfile`) — category display-attribute provider
  biến mất nên app vẫn vẽ gạch chân. Nay `RegisterCategory` qua API được thử
  **độc lập** với `RegisterProfile` (TSF thường chấp nhận non-admin), và khi
  API thất bại, category HKCU được ghi **SAU `InstallLayoutOrTip`** — verified
  live: cả 4 category (kèm `{046B8C80-…}`) sống sót sau ILOT trên tài khoản
  thường.
- Ctrl+Shift + đăng ký: dùng chung code đã sửa ở 0.2.4 (không phân biệt
  installer/portable) — test live lại trên bản portable: `register` exit 0,
  `register status` OK, không admin.

### Added
- **Pipeline ký số SignPath** (opt-in qua secrets `SIGNPATH_*`): có secret →
  binary + installer được ký tự động khi build release; không secret → giữ
  nguyên release-candidate chưa ký. Kế hoạch + so sánh các đường free
  (SignPath Foundation / Azure Artifact Signing / SSL.com / Certum):
  [docs/release/code-signing-plan.md](docs/release/code-signing-plan.md).

## [0.2.4] — 2026-10-01

Bản **release candidate chưa ký số**. Trọng tâm: bộ gõ TSF phải đăng ký và gõ
được trong bản cài Windows, đồng thời không để lại override COM hỏng khi UAC
dùng tài khoản quản trị khác. CI kiểm thử cài/gõ thật/gỡ trên Windows, IBus và
Fcitx5 trên Linux, Swift hai kiến trúc và đóng gói trên macOS.

### Fixed
- **Đăng ký TSF không cần quyền Administrator**: Sửa hậu kiểm `registration_ok()` trong `textvn-cli`
  để chấp nhận profile HKCU `Enable = 1` (ghi bởi `register_ctf_per_user` và `InstallLayoutOrTip`),
  thay vì chỉ dựa vào `IsEnabledLanguageProfile` (vốn đọc HKLM). Đăng ký đầy đủ category
  `GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER` (GUID thật `{046B8C80-…}` — bản trước ghi sai `{2464BEB0-…}`
  làm fallback per-user đăng ký category rác, TSF không bao giờ nhận diện provider). Giờ đây người dùng
  chạy bản portable hoặc tài khoản tiêu chuẩn không bị báo lỗi `FAIL: hậu kiểm đăng ký TSF không đạt`
  và không bị ép chạy Administrator.
- **Tắt hoàn toàn gạch chân chữ đang gõ (Clean composition không gạch chân)** trên cả 3 hệ điều hành:
  - *Windows TSF*: Triển khai `ITfDisplayAttributeProvider`, `ITfDisplayAttributeInfo`, trả về `TF_LS_NONE`
    (không vẽ gạch chân) và gán `DISPATTR_TEXTVN` lên dải composition trong `edit_session.rs`.
  - *Linux IBus*: Loại bỏ thuộc tính `IBUS_ATTR_TYPE_UNDERLINE` trong `engine.c`.
  - *Linux Fcitx5*: Đổi cờ định dạng preedit sang `fcitx::TextFormatFlag::NoFlag` trong `engine.cpp`.
  - *macOS IMK*: Truyền `NSAttributedString` với `.underlineStyle = 0` (`NSUnderlineStyle([])`)
    thay vì chuỗi trần sang `client.setMarkedText()` trong `IMKTextTarget.swift`.
- **Sửa lỗi không đổi icon và không chuyển mode gõ khi bấm Ctrl + Shift** trên cả 3 hệ điều hành:
  - *Windows*: Tự động giải phóng `Ctrl+Shift` khỏi `HKCU\Keyboard Layout\Toggle` của hệ thống khi `TextVN.exe`
    khởi động. Tổ hợp được nhận diện **in-process trong TIP** (`ModifierToggle` + `KeyTraceSink`) — một bản
    0.2.4-dev từng cài thêm `WH_KEYBOARD_LL` trong tray khiến **một lần bấm bị toggle đôi** (TSF + hook cùng
    bắn → bấm không đổi mode/icon) và vi phạm chính sách AV A2/A3 (hook LL chỉ thuộc gói compatibility
    opt-in); hook đã bị xoá, tray nhận kết quả qua IPC để đổi icon `[V]`/`[E]` tức thì.
  - *Installer*: cài **per-user** (mặc định, không UAC) từng luôn báo "Windows từ chối đăng ký" vì chạy
    `register --scope machine` không elevation — nay nhánh per-user đăng ký per-user đúng phạm vi
    (CI `test-installer.ps1` kiểm cả 2 kịch bản `/ALLUSERS` và per-user).
  - *macOS*: Thêm bộ theo dõi trạng thái `flagsChanged` trong `TextVNInputController.swift` để nhận diện
    tổ hợp `Ctrl + Shift` tap mà không kèm phím khác; gửi IPC `ToggleViEn` sang menu bar app `TextVN.app`;
    cập nhật nút thanh menu bar hiển thị rõ chỉ báo chế độ `[V]` / `[E]`.
  - *Linux*: Triển khai `subModeIconImpl` trong Fcitx5 engine để icon khay hệ thống và submode panel
    tự động chuyển đổi mượt mà giữa `textvn_v` và `textvn_e`; đảm bảo IBus cập nhật property icon khi toggle.
- Installer Windows đăng ký COM/profile TSF ở phạm vi máy, sau đó thêm layout
  và kích hoạt trong phiên của người dùng gốc. Dọn override HKCU cũ thay vì
  tạo override mới trỏ DLL có thể bị xoá khi gỡ.
- VNI không nuốt phím `0` khi không có dấu để xoá; gõ tắt theo phím gốc hoạt
  động sau khi Telex đã biến đổi chữ hiển thị.
- IPC tray giới hạn hàng đợi/worker cho subscriber chậm; đọc frame thiếu dữ
  liệu có timeout. Sửa các nhánh `cfg` khiến Linux/macOS không biên dịch.
- EventTap macOS thoát được khi dừng sát thời điểm khởi động run loop; dùng
  API `CFRunLoopMode.defaultMode` tương thích SDK Swift hiện tại.

### Verification
- CI `ci-shared` và `ci-macos` xanh trên commit sửa code; job perf trên runner
  dùng chung chỉ là cảnh báo, không chặn release.
- Chưa có Authenticode, Developer ID/notarization, hoặc GUI smoke trên máy Mac
  thật; không gắn nhãn production.

## [0.2.3] — 2026-10-01

Bản phát hành sau 2 vòng audit chuyên sâu toàn repo (code Rust, GitHub Actions,
icon/packaging — từng dòng). **Chủ đề: icon đủ trên cả 3 nền tảng** — macOS
trước đây không có icon bundle nào (pipeline sinh .icns tồn tại sẵn nhưng thiếu
file PNG nguồn nên âm thầm tắt từ trước đến nay). Vẫn **release candidate,
chưa ký số**.

### Added
- **Icon cho macOS (đủ như Windows)**: `packaging/macos/icons/TextVN-1024.png`
  (+512) — render từ thiết kế badge `resources/icons/textvn_v.svg` bằng script
  mới `scripts/generate_app_icons.py`; `build-macos.sh` sinh `TextVN.icns` vào
  cả `TextVN-IM.app` và `TextVN.app` (Input Sources, Finder/Spotlight hết icon
  trắng mặc định); `Info-IM.plist` thêm `tsInputMethodIconFileKey` để input
  menu trên menu bar có icon.
- **Icon raster Linux cho panel cũ**: `resources/icons/textvn_v.png` /
  `textvn_e.png` 128px cài kèm SVG vào hicolor (`install.sh`, stage, manifest
  uninstall đều cập nhật); AppStream metainfo thêm `<icon type="stock">` —
  Software Center hiển thị icon thay vì ô trống.
- **Icon portable Linux**: bản chạy ngay (`textvn-portable.sh`) stage hicolor
  icons vào `$XDG_RUNTIME_DIR` + truyền `XDG_DATA_DIRS` cho daemon — engine
  panel (IBus/Fcitx5) và cửa sổ cài đặt resolve được `textvn_v` thay vì icon
  mặc định (GNOME Shell tự vẽ panel theo session — giới hạn đã ghi trong script).
- `scripts/generate_app_icons.py`: một nguồn sinh mọi PNG icon từ SVG brand
  (chạy bằng tay, PNG commit sẵn — CI không cần Pillow).

### Fixed (Windows)
- **Cửa sổ Bảng điều khiển + Gõ tắt có icon title bar/Alt-Tab** (trước đây
  dùng icon Windows mặc định): `tray.rc` nhúng icon trung tính ID 3, class
  đăng ký `hIcon`; loader `load_app_icon` tách thành module dùng chung
  `tray/src/icons.rs` (G15 — một bản cho tray và cửa sổ).
- **Installer có icon riêng** (`SetupIconFile`) thay vì icon mặc định Inno Setup.

### Fixed (đăng ký TSF — audit vòng sâu, 3 major)
- **Lỗi thiếu DLL giờ vào `register.log`**: trước đây `resolve_dll_path` fail
  chỉ in stderr (console ẩn nuốt mất) → hộp thoại tray đọc đuôi log của run
  CŨ và ghép advice sai nguyên nhân. Nay ghi header run mới + lỗi vào log
  trước khi thoát.
- **Windows Single Language không còn bị chặn đăng ký**: profile EN (0x0409)
  chuyển thành best-effort (WARN) — trước đây ILOT/hậu kiểm fail cứng trên EN
  khiến toàn bộ đăng ký báo FAIL dù VI hoạt động đầy đủ. Hậu kiểm bắt buộc VI.
- **Đăng ký `--scope machine` dọn override per-user**: key HKCU CLSID cũ (trỏ
  DLL portable đã xoá) che HKLM mới ở mức COM — xoá trước khi ghi machine;
  hậu kiểm `server_ok` dò cả 2 hive thay vì HKCU-first tuyệt đối.
- `unregister` báo thất bại (exit 1) khi còn key không xoá được; scope machine
  xoá thêm cây CTF TIP HKLM; `Enable=1` ghi lại SAU `InstallLayoutOrTip` (spike
  doc: cập nhật input list có thể reset enabled flag); `FreeLibrary` input.dll
  sau mỗi lần gọi; `doctor` tìm `data/` cạnh exe thay vì chỉ cwd; cửa sổ sửa
  Gõ tắt không còn đóng giả thành công khi không lấy được trạng thái ứng dụng
  (bảng vừa sửa bị vứt im lặng); FFI dùng hằng `IME_FIELD_SECURE` thay số ma
  thuật `10` + ghi contract pointer `ime_last_error`.

### Changed (CI — audit GitHub Actions, 5 major)
- `ci-shared.yml`: thêm `permissions: contents: read` (mọi job token chỉ đọc);
  `cancel-in-progress` chỉ trên PR (push graphify/docs sau commit code từng
  giết run CI của commit code); bỏ qua push chỉ chứa `**.md`/`docs/`/
  `graphify-out/`; **timeout-minutes cho 12/13 job** (mặc định 360′ là lãng phí
  khi cargo hang); **Swatinem/rust-cache** cho các job nặng (test/replay/clippy/
  fuzz/linux-adapters/windows-package); `persist-credentials: false` ở checkout.
- `release.yml`: **publish draft-first** — tạo draft, upload hết asset
  (`--clobber`, rerun được) rồi mới public; jq assert thêm
  `feature_profile == "tsf-only"`; job `publish` có timeout; job macOS chạy
  `swift test` TRƯỚC khi đóng gói.
- `ci-macos.yml`: bỏ `needs: engine-static` vô ích ở job swift (tự build
  staticlib per-arch) + timeout 60′; `repo-hygiene` có timeout + token chỉ đọc;
  `targets-verify` chỉ tải installer từ hostname allowlist.

### Housekeeping
- Quy trình phát hành chuẩn hoá: **`docs/release/release-process.md`** (G16
  trong `00-WORKFLOW.md`) — Checklist A (local) → Checklist B (repo/docs) bắt
  buộc cho mọi agent/contributor, dựa trên v0.2.0–v0.2.2 đã phát hành thành
  công; bảng sự cố khi phát hành (typing smoke flake B5, perf continue-on-error,
  publish draft rerun…).

## [0.2.2] — 2026-09-30

Bản vá nghiêm túc cho bản 0.2.1: **ưu tiên cao nhất** là F3-13 — khi TextVN tự
khởi động theo login item trên macOS, cửa sổ **Cài đặt không còn tự bật lên**
nữa. Gói vẫn chưa ký số, chưa có smoke GUI trên máy Mac thật nên **không phải
production**.

### Fixed (macOS — ưu tiên bản này)
- **F3-13 · login launch không mở Cài đặt**: `SMAppService` **không** truyền
  `--autostart` vào argv, nên cờ mở dialog trước đây hiểu nhầm là khởi động thủ
  công. Thêm `AutostartManager.isAutostartConfigured()` coi cả trạng thái
  `.requiresApproval` là “đã đăng ký”, cộng `config.autostart`; `NSLog` ghi rõ
  `settings/loginLaunch/showDialogOnStartup` để chẩn đoán. Trade-off đã ghi trong
  review log: mở tay sau khi bật autostart cũng không tự mở Settings.
- **F3-6 · fallback LaunchAgent thật sự hoạt động**: trước đây `setAutostart(true)`
  coi là xong khi `SMAppService.register()` trả về mà status vẫn `.requiresApproval`,
  khiến plist fallback không bao giờ được ghi → bật autostart rồi restart là mất.
  Nay đo lại `status == .enabled`, chỉ ghi plist khi SM **không** nhận.
- **F3-8 · menu bar theo `ui-spec`**: icon là SF Symbol **template** (tự đổi màu
  sáng/tối, badge `error` màu cam khi IMK crash), submenu **Dấu** đầy đủ, mục
  **Bật tiếng Việt cho {app}** đọc app foreground gần nhất, mỗi mục có đánh dấu
  trạng thái riêng, và **Sức khoẻ** in PID tiến trình IMK + tuổi heartbeat.
- **Nit F3-15/16/17/18/19/20/21**: vòng quan sát workspace được huỷ khi terminate
  (hết retain cycle), hotkey `Ctrl+Shift+Space` so **mask** nên vẫn toggle khi
  Caps Lock bật, `IMKApp` huỷ observer khi thoát, Secure Input tính lại **trước**
  khi nuốt `keyDown`, `Diagnostics` đọc Int đúng kiểu và có fallback,
  `KeyTranslator` dùng mask modifier, `setAutostart` không gọi hai lần, bỏ nhánh
  `SettingsController` chết.
- `applyReplace` escape XML đường dẫn LaunchAgent; `IpcClient` ghi log chẩn đoán
  khi socket lỗi; `ipcServer(_:didChangeEnabled:)` phát `ConfigReload` để IMK
  nạp lại `config.enabled` (trước đó bật/tắt ở menu không tới IMK).

### Fixed (Windows)
- **Thông báo lỗi attachment sai**: danh sách dò DLL trong `textvn register` lặp
  `textvn-tsf.dll` hai lần và câu lỗi ghi “(hoặc `textvn-tsf.dll`)” — tức **tên
  đúng** `textvn_win_tsf.dll` mà gói phát hành mang lại không bao giờ hiện ra
  đúng ở phần giải thích. Bỏ trùng lặp, sửa câu lỗi, và hộp thoại “Cài & bật TSF”
  giờ hiện **lý do thật** lấy từ đuôi `register.log` kèm gợi ý đúng nguyên nhân
  (thiếu quyền Administrator / `ACCESS_DENIED` khi ghi HKCU / thiếu DLL cạnh
  `textvn-cli.exe`) thay vì luôn đổ lỗi cho ACL.
- **Gợi ý lỗi đăng ký khớp log thật (triệt để hóa đợt trên)**: log CLI in lỗi
  registry chỉ dạng hex `FAIL 0x00000005` không kèm tên ký hiệu, nên nhánh gợi ý
  `ACCESS_DENIED` trong hộp thoại là **dead code** — lỗi ACL thật (sandbox, AV,
  policy chặn ghi `HKCU\Software\Classes\CLSID`) vẫn luôn nhận gợi ý generic.
  Nay mỗi dòng log registry mang đủ ngữ cảnh
  `Registry create HKCU\…\CLSID → FAIL 0x00000005 (ERROR_ACCESS_DENIED)`, dòng
  FAIL nêu đúng khoá bị chặn, và hộp thoại khớp **cả hai** định dạng log (bản
  mới có tên ký hiệu, bản 0.2.0/0.2.1 đã cài chỉ có hex). Test mới
  `lstatus_name_covers_acl_and_common_registry_errors`; `advice_matches_real_cause`
  được mở rộng với đúng chuỗi log thật thay vì chuỗi mô phỏng.

### Fixed (macOS — tiếp)
- `EventTapController.start()` là idempotent (gọi hai lần không rò tap/thread) và
  `stop()` chặn race với thread chưa gắn runloop; app foreground đọc từ **cache**
  cập nhật qua notification thay vì gọi `NSWorkspace.frontmostApplication` — vốn là
  XPC từ khoá 2ms trong callback event tap.

### Changed
- Job `perf regression` trên CI vẫn `continue-on-error` (runner dùng chung, số đo
  nhiễu). Đo trên máy local: `ime_key` p50 687 → **500 ns (−27,2%)**,
  `parse_config` 781 → 768 ns (−1,7%) — **không có hồi quy**; xem
  [báo cáo build](docs/release/build-release-report.md).

### Housekeeping
- Dọn file temp của phiên làm việc, `.gitignore` thêm `/build/` và `.vscode/`
  (artifact CMake và cấu hình IDE máy-locale trước đó lọt vào untracked).

## [0.2.1] — 2026-09-30

Release candidate bảo trì: macOS không tự bật Settings ở phiên login qua
`SMAppService`; đường dẫn LaunchAgent được XML-escape; mặc định khởi động cùng
macOS là opt-in và UI “Đặt dấu tự do” ghi đúng khoá. Windows `-BuildInstaller`
giờ dừng nếu không tìm thấy Inno Setup và không ghi đè ZIP đã tồn tại.
Gói chưa ký số và chưa có smoke GUI trên macOS nên **không phải production**.

## [0.2.0] — 2026-09-30

Mục tiêu: bản release candidate dùng được hằng ngày trên Windows, Linux và macOS (Farch-4). Kết quả kiểm thử:
[docs/release/build-release-report.md](docs/release/build-release-report.md).

### Added
- **Linux chạy được thật**: adapter IBus và Fcitx5 dựng, nạp và gõ được tiếng Việt (kiểm
  bằng ibus-daemon 1.5.29 và fcitx5 5.1.7 thật — `scripts/e2e-linux.sh`, job CI `linux-adapters`).
- **Gói Linux** `TextVN-<ver>-linux-<arch>.tar.gz`: `./install.sh` (per-user mặc định, `--system`)
  tự thêm TextVN vào danh sách bộ gõ GNOME/IBus/Fcitx5; `./textvn-portable.sh` chạy ngay từ
  thư mục giải nén (kể cả chỉ đọc); `uninstall.sh` gỡ đúng file đã cài (`--purge` xoá cấu hình).
- **Bảng điều khiển thống nhất** Windows (Win32), Linux (GTK4), macOS (SwiftUI): cùng tuỳ chọn, nhãn, bố cục
  ([ui-spec](docs/release/ui-spec.md)); trình sửa **Gõ tắt** (`gõ tắt = nội dung`, báo lỗi từng dòng).
- Engine: bảng mã xuất **Unicode tổ hợp, TCVN3 (ABC), VNI Windows** (bảng lấy từ UniKey);
  **Quick Telex**; Telex `z` xoá dấu; gõ được khi bật **Caps Lock** (`VIEETJ` → VIỆT) mà vẫn
  giữ nguyên chữ viết tắt gõ bằng Shift (`USA`, `JSON`).
- Trạng thái V/E lưu ở `state.json` trên cả hai nền tảng; phím **Ctrl+Shift** (nhấn-nhả) kiểu
  UniKey trong TSF, IBus, Fcitx5.
- Kiểm thử gói Windows trên máy Windows thật (job CI `windows-package`): `build-release.ps1`,
  kịch bản **cài đặt** (Inno Setup im lặng) và **giải nén dùng ngay** (zip), mỗi kịch bản gõ
  thật qua TSF vào Notepad/WordPad rồi gỡ sạch.
- Kiểm thử từ vựng thật `core/tests/common_words.rs` (~400 từ × Telex/VNI/kiểu cũ/`uow`/giữa
  từ/Caps Lock); corpus 113 kịch bản × 5 adapter.
- `TEXTVN_TSF_TRACE=<file>`: nhật ký chẩn đoán key sink TSF (tắt mặc định, không ghi nội dung gõ).
- Tài liệu: [hướng dẫn sử dụng](docs/user-guide.md), [hướng dẫn phát triển](docs/developer-guide.md),
  [đối chiếu bộ gõ tham chiếu và bug đã biết](docs/specs/reference-parity.md).

### Added (macOS — Farch-4)

- **macOS IMK adapter** (`adapters/macos-imk/`, Swift): `TextVN-IM.app` — IMKServer +
  `TextVNInputController` theo P2-1 (marked text lifecycle, commit-before-hide B13,
  marked ≤8 grapheme B11, SelectionReplace không-backspace B1, fail-open S4)
- **macOS CGEventTap fallback** (`adapters/macos-tap/`, Swift): tap opt-in per-app
  `engine_owner: "tap"`, marker loop-guard `TXVN`, self-disable khi chậm 2ms ×50,
  injector BackspaceType/SelectionReplace (P2-2)
- **Bảng keycode macOS → VK canonical** duy nhất `data/tables/keymap_mac.toml`
  → generate cả Rust (`core/src/keymap_mac_generated.rs`) lẫn Swift
  (`KeyMapMacGenerated.swift`) qua `cargo xtask gen-tables` — 92 phím
- **AppDB preset macOS** 20 mục (`mac.safari.url`…`mac.game.nokbd`, engine_owner
  `imk`/`tap`) trong `data/appdb.default.json`
- **Corpus macOS 114 case** `corpus/mac/` (P2-5 §2: B1/B2/B3/B8/B11/B13/secure/
  owner/imk_preedit 40/mac_bs_type 30/tap 20) — gen bằng `cargo xtask gen-mac-corpus`,
  replay `--adapter mac` pass 114/114 headless mọi OS
- **CI `ci-macos.yml`**: build staticlib 2 arch + lipo, swift build/test cả 2 package,
  replay corpus mac, ABI gates; `ci-shared` replay thêm `--adapter mac`
- Tài liệu: `docs/30-macos/IMPLEMENTATION-STATUS.md` (trạng thái trung thực),
  `docs/30-macos/env-mac.md` (MAC-001)

### Fixed
- Corpus win drift: 14 case trong `xtask/src/win_corpus_cases.rs` lệch với
  `corpus/win/*.keys` đã sửa tay (tổng `tongj`→`toongr`, `rooid`→`roofi`,
  thiếu `auto_capitalize=false`, combo `Esc`→`Escape`…) — `check-win-corpus`
  nay pass và đã vào CI replay
- `cargo xtask` TOML parser hỗ trợ hex `0x…` cho bảng keycode
- **macOS subsystem audit review round 3&4** (F2-029…F2-040):
  - Bọc con trỏ Carbon `UCKeyTranslate` (`UnsafePointer<UCKeyboardLayout>`) qua `layout.withUnsafeBytes`.
  - Dispatch trực tiếp `client.doCommand(by:)` trên `IMKTextInput` thay vì ép kiểu `NSResponder` (tránh fail trên out-of-process XPC session).
  - Khai báo mở rộng `NSRange.notFound` và `String.utf16Count`.
  - Sửa `CGEvent.tapEnable` gọi đúng `CFMachPort` thay vì proxy.
  - Thêm helper `ImeEngine.string(fromUTF32:len:)` trong `CoreBridge`.
  - Đảm bảo an toàn bộ nhớ ARM64 (Apple Silicon) bằng `loadUnaligned` khi giải mã frame IPC.
  - Hủy `retryTimer` khi `IpcClient.tearDown()`, hỗ trợ toggle toàn cục `appID.isEmpty`.
- **macOS CI fix MAC-031** (commit `f10eb1a` — 2026-09-29):
  - `adapters/macos-imk/Package.swift`: thay `URL(fileURLWithPath: #filePath)` bằng
    `String.components(separatedBy:)` thuần — `URL`/`Foundation` không khả dụng trong
    `PackageDescription` scope trên Xcode 26.6 / Swift 6.
  - `adapters/macos-imk/build-rust.sh`: thêm `PKG_DIR`/`ROOT_DIR` tuyệt đối; thay mảng
    `PROFILE=()`/`EMPTY_SAFE` bằng scalar `PROFILE_FLAG=""` — bash 3.2 macOS ném
    `unbound variable` khi expand mảng rỗng dưới `set -u`.
  - `.github/workflows/ci-macos.yml`: matrix `arch: [arm64, x86_64]` → `include` với
    `rust_target: aarch64-apple-darwin`/`x86_64-apple-darwin`; `cargo build` chạy từ
    workspace root; copy FFI header bước riêng.

### Fixed (review vòng 3 — trước tag v0.2.0)

Chi tiết: [P2-REVIEW-LOG — Round 9](docs/30-macos/P2-REVIEW-LOG.md).

- **Installer Windows kẹt 0.1.0** (blocker): `#define MyAppVersion` cứng đè `/DMyAppVersion`
  của script release → nay `#ifndef`; hết version-skew ở manifest Win32, component IBus,
  Hello IPC Linux, cask Homebrew, AppStream. Gate mới `cargo xtask check-version-sync`.
- **macOS**: bật/tắt tiếng Việt từ menu bar nay tới được IMK (quy ước `app_id = "*"` toàn
  cục chung mọi nền tảng); cửa sổ Cài đặt "Mở rộng" không còn bị cắt; TextVN-IM không còn
  có thể chết vì SIGPIPE; sửa `config.json` ngoài app có hiệu lực ngay (hot-reload
  MAC-053); config hỏng được giữ lại `config.json.corrupt-<ts>` thay vì bị ghi đè mất
  macro; IPC server đóng kết nối khi vi phạm giao thức và kiểm uid peer; tắt tự khởi động
  gỡ cả LaunchAgent fallback; gỡ cài đặt sạch cả scope hệ thống, Login Item và receipt pkg.
- **Windows tray**: mục menu "Bật tiếng Việt cho {app}" hoạt động (nhớ app đang gõ trước khi
  click khay); click đầu sau khi đóng menu không bị nuốt; ghi `state.json` lỗi không còn
  phát trạng thái sai; chống 2 instance khi `GetLastError` bị đè; vòng message thoát đúng
  khi `GetMessageW` lỗi.
- **Linux**: `60-textvn.conf` không còn sót sau khi gỡ khi `XDG_CONFIG_HOME` khác mặc định;
  gỡ bản `--system` bỏ TextVN khỏi danh sách bộ gõ; cảnh báo khi Fcitx5 sẽ không nạp addon.
- **CI xanh cả 3 nền tảng lần đầu** (MAC-032): code macOS (IMK, tap, app) nay build + test
  trên Xcode 26.6 cả arm64/x86_64 và đóng gói `.pkg`; sửa kèm lỗi thật lộ ra khi chạy:
  gõ từ dài hơn 8 ký tự ở chế độ preedit không còn mất ký tự thứ 9; tap CGEvent không rò
  bộ nhớ mỗi phím; installer Windows compile được và báo lỗi (exit 10) khi đăng ký TSF
  thất bại; `textvn-portable.sh` không rollback nhầm trên phiên chưa có ô nhập đang focus.

### Added (trước đó)
- **Tray Icon**: Icon TextVN 16/32/48px nhúng qua winresource, manifest DPI PerMonitorV2
- **CLI**: `textvn-cli register` / `textvn-cli unregister` — đăng ký/hủy TSF TIP per-user (WIN-003, WIN-010)
- **CLI**: `textvn-cli register status` — kiểm tra trạng thái đăng ký TSF
- **Windows manifest**: asInvoker (không cần admin), Windows 10/11 compatibility

### Fixed
- **Windows TSF không gõ được tiếng Việt** (`docs/specs/tsf-typing-overhaul.md`, Ftsf-1…14):
  security gate không bao giờ mở; đăng ký per-user dừng trước `InstallLayoutOrTip`; composition
  mở mới mỗi phím; app treo khi Deactivate; hotkey đảo 2 lần; Delete/F-key/mũi tên bị coi là
  chữ; `doctor` kiểm sai CLSID.
- TSF: dấu cách/dấu câu được chốt **cùng** từ; Enter/Tab/phím điều hướng ở ứng dụng Win32
  cổ điển (Notepad, WinForms… qua CUAS) được xử lý ở pha `OnKeyDown` — trước đây các phím này
  ra **trước** chữ (" được", "\nchào").
- TSF: Ctrl+Shift chuyển V/E được cả ở ứng dụng TSF-aware (Word, WordPad): phím chuyển bố cục
  của Windows nuốt Shift trước key sink, nay nhận qua `ITfKeyTraceEventSink`. Báo trạng thái
  tuyệt đối cho tray thay vì "đảo".
- Gỡ bản portable (menu khay hoặc `uninstall.ps1`) bỏ luôn mục tự khởi động trỏ vào thư mục đó.
- Ctrl+Shift "lúc được lúc không" khi ngôn ngữ của TextVN có hơn một bàn phím: phím tắt đổi bố
  cục mặc định của Windows (cũng là Ctrl+Shift) chuyển đi mất TextVN mỗi lần bấm thứ hai. Tuỳ
  chọn **Dành Ctrl + Shift cho TextVN** (bảng điều khiển, bộ cài chọn sẵn, `TextVN.exe
  --free-ctrl-shift`), `doctor` báo khi Windows còn giữ phím này.
- Đặt dấu sai chính tả: `của`→cuả, `nghĩa`→nghiã, `thuỷ`/`thủy` theo kiểu dấu, `được`→đựơc
  (kiểu cũ); dấu tự dời khi gõ thêm chữ (`hòa`+`n` → hoàn).
- `d` + nguyên âm tự thành `đ` (không gõ được dân, dạy, dưới…): `đ` giờ chỉ từ `dd` như
  UniKey/OpenKey/Bamboo. Sửa `gi`+nguyên âm (giữa, giờ), `qu`+ơ, `thuở`, `uow` → ươ.
- Gõ tắt không còn bung sau Home/End/F-key hay tổ hợp Ctrl/Alt (có thể xoá nhầm chỗ).
- Linux: engine reset mỗi lần caret đổi (không biến đổi được chữ nào), mất chữ khi Enter/focus,
  addon Fcitx5 không bao giờ nạp, IBus sai bus name, cấu hình từ bảng điều khiển không được
  đọc, bảng điều khiển ghi đè mất gõ tắt; per-user IBus không hiện sau đăng nhập lại (cache
  registry của ibus-daemon).
- Tray Windows: mất icon khi Explorer khởi động lại; lưu cấu hình làm mất khoá người dùng và
  ghi đè file hỏng; menu **Gỡ cài đặt** cho bản cài lẫn bản portable.
- Installer: thiếu bản dịch tiếng Việt của Inno Setup nên không biên dịch được; cài im lặng
  không còn bật UAC.
- Giảm bề mặt bị phần mềm diệt virus nghi ngờ: không `SendInput` trong gói mặc định,
  `input.dll` chỉ nạp từ System32, không `TerminateProcess`, pipe từ chối client từ xa.

### Changed
- Tra bảng nguyên âm trên đường phím nóng: nhánh ASCII + tìm kiếm nhị phân thay vì duyệt tuyến tính.
- Gói Windows mặc định **TSF-only**; hook tương thích chỉ có trong gói Compatibility (opt-in).

### Known limitations
- macOS: adapter IMK, CGEventTap, Menu Bar App, Settings SwiftUI và kịch bản đóng gói/cài đặt đã hoàn thiện mã nguồn và kiểm thử logic/replay (114/114 case pass); cần máy macOS vật lý để kiểm chứng giao diện đồ họa và cấp chứng chỉ Apple Developer ID.
- Bản dựng CI chưa ký số Authenticode (Windows SmartScreen sẽ cảnh báo lần đầu).

---

## [0.1.0] — 2026-09-27

### Added

#### Core
- Engine gõ tiếng Việt (Telex, VNI, VIQR, Simple Telex)
- Kiểu bỏ dấu: Chuẩn mới (`hoà`, `thuỷ`) và Cổ điển (`hòa`, `thủy`)
- Config schema v1 với JSON parser + validator
- Corpus replay system (golden test suite)

#### Windows Platform
- **TSF TIP** (`textvn-tsf.dll`): Text Input Processor đăng ký với Windows Input Framework
  - `ITfTextInputProcessorEx::Activate/Deactivate`
  - `ITfKeyEventSink`: xử lý phím gõ
  - `ITfEditSession::DoEditSession`: commit text vào ứng dụng
  - Named Pipe IPC client (kết nối tới tray)
- **System Tray** (`textvn-tray.exe`): tray app với 9-item context menu
  - Bật/Tắt tiếng Việt toàn cục
  - Chọn chế độ gõ (Telex/VNI/VIQR/Simple Telex) qua submenu
  - Chọn kiểu bỏ dấu (Chuẩn mới/Cổ điển)
  - Per-app toggle cho ứng dụng đang foreground
  - Game/Compat mode (tắt hook)
  - Health submenu (clients, version)
  - Gỡ cài đặt / Thoát
  - Single-instance mutex (`Local\TextVNTray`)
  - Named Pipe IPC server (`\\.\pipe\textvn-ipc-v1`)
  - Auto-start via HKCU Run key
- **CLI** (`textvn.exe`):
  - `replay` — chạy golden corpus test
  - `verify` — kiểm tra header C vs Rust ABI
  - `sizes` — verify struct layout (20 bytes key, 532 bytes result)
  - `config init/default/validate`
  - `doctor [--json] [--export]` — chẩn đoán môi trường (WIN-058)
  - `register [--scope user] [--dll path]`
  - `unregister`
- **Installer**: Inno Setup 6 script — cài đặt per-user, không cần admin (WIN-054, WIN-055)
- **Low-level Hook** (`textvn-hook.exe`): WH_KEYBOARD_LL hook với 2ms timebox, injection guard

#### Developer Tools
- `textvn doctor --export diag.zip` — xuất báo cáo chẩn đoán (zero-dependency ZIP)
- REUSE compliance (1006+ files)
- Graphify knowledge graph (2389 nodes)
- `docs/specs/tsf-registration-spike.md` — research findings per-user TSF registration

### Architecture
- Named Pipe IPC protocol (textvn-ipc crate)
- `IpcClient` dùng `Mutex<Option<JoinHandle>>` để `stop()` có thể gọi từ `&self`
- `IpcServer::stop()` dùng dummy connect để unblock `ConnectNamedPipe`
- Per-user registration: HKCU + `InstallLayoutOrTip(input.dll)`

---

## Versioning Policy

- **MAJOR** (x.0.0): breaking change ABI, config schema, IPC protocol
- **MINOR** (0.x.0): tính năng mới backward-compatible
- **PATCH** (0.0.x): bug fix, perf, docs, security

Bump version trong `Cargo.toml` (workspace `[workspace.package]`) và tạo Git tag:
```powershell
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0
```

[Unreleased]: https://github.com/hunglinhpt/TextVN/compare/v0.2.17...HEAD
[0.2.17]: https://github.com/hunglinhpt/TextVN/compare/v0.2.16...v0.2.17
[0.2.16]: https://github.com/hunglinhpt/TextVN/compare/v0.2.15...v0.2.16
[0.2.15]: https://github.com/hunglinhpt/TextVN/compare/v0.2.14...v0.2.15
[0.2.14]: https://github.com/hunglinhpt/TextVN/compare/v0.2.13...v0.2.14
[0.2.13]: https://github.com/hunglinhpt/TextVN/compare/v0.2.12...v0.2.13
[0.2.12]: https://github.com/hunglinhpt/TextVN/compare/v0.2.11...v0.2.12
[0.2.11]: https://github.com/hunglinhpt/TextVN/compare/v0.2.10...v0.2.11
[0.2.10]: https://github.com/hunglinhpt/TextVN/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/hunglinhpt/TextVN/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/hunglinhpt/TextVN/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/hunglinhpt/TextVN/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/hunglinhpt/TextVN/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/hunglinhpt/TextVN/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/hunglinhpt/TextVN/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/hunglinhpt/TextVN/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/hunglinhpt/TextVN/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/hunglinhpt/TextVN/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/hunglinhpt/TextVN/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hunglinhpt/TextVN/releases/tag/v0.1.0
