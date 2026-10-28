# Changelog

Tất cả thay đổi đáng chú ý của dự án TextVN sẽ được ghi lại ở đây.

Định dạng dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
và dự án này tuân thủ [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.52] — 2026-10-28

### Added
- **Trợ lý ngữ pháp & Text Prediction tiếng Anh toàn diện** (vòng 37 / AS-060):
  - **Mở rộng vốn từ vựng tần suất**: 3.356 từ tiếng Anh (vượt mốc 3.000 từ)
    bao gồm thuật ngữ coder, tech, business, OS APIs (Win32 `HRESULT`, `GetLastError`,
    `VirtualAlloc`; POSIX `syscall`, `epoll_create`, `pthread_create`; Cocoa
    `NSString`, `CGRect`, `RunLoop`) và tiếng lóng Gen Z (`rizz`, `skibidi`, `sigma`,
    `slay`, `delulu`, `brainrot`, `pookie`). Đảm bảo 100% test hồi quy gõ khôi phục.
  - **Dự đoán cụm từ tiếp theo**: 3.750 mẫu câu & cụm từ ngữ cảnh (`en_phrases.tsv`)
    sinh bởi generator native Rust (`cargo xtask gen-en-phrases`), tuân thủ nghiêm
    ngặt bất biến R1-2 (độ dài $\le 64$ ký tự). Hỗ trợ phím tắt `Tab` (hoàn tất tiền tố
    và dự đoán cụm) và `Ctrl + Space` (tự sửa từ/câu rồi kích hoạt dự đoán).
  - **Cặp từ dễ nhầm lẫn mới** (closed list, FPR = 0): `their own` (thay vì `there own`),
    `except for` (thay vì `accept for`), `led to` (thay vì `has lead to`), `lose`
    (thay vì `to/will loose`), `principal` (thay vì `principle engineer/architect`),
    `take effect`, `the effect of`.
  - **Kiểm tra chính tả tiếng Anh OS Spell dual-language**: worker Spell Checking API
    trên Windows hỗ trợ song ngữ `vi-VN` và `en-US`/`en-GB`, dispatch thông minh theo
    ký tự ASCII/Unicode, bảo vệ RAII `ComApartment` chống xung đột vòng đời COM.

**English** — 0.2.52 delivers a comprehensive English grammar assistant and text prediction engine:
3356 frequent English words covering tech, coder, business, OS APIs (Win32, POSIX, Cocoa) and Gen Z slang;
3750 contextual next-phrase prediction patterns in `data/en_phrases.tsv` generated with pure native Rust,
strictly bounded by invariant R1-2 (length <= 64); closed-list confusion rules (their own, except for,
led to, lose vs loose, principal vs principle, take effect, the effect of); and dual-language Windows OS
spell checking (vi-VN + en-US) protected by RAII ComApartment against COM teardown faults.

## [0.2.51] — 2026-10-28

### Added
- **Ngữ pháp tiếng Anh hoàn tất** (vòng 36 — trước đây trợ lý tiếng Anh chỉ
  có luật a/an, mọi cặp lỗi phổ biến đều im lặng — word_confusion 0/10 +
  lowercase_pronoun 0/7 trên corpus đề xuất):
  - **Cặp từ dễ nhầm** (danh sách đóng, chỉ câu En-dominant — không bao giờ
    đụng câu Việt): `should/could/would/must of` → `have` (guard "of
    course"); `your welcome/the/not/right` → `you're`; `its been/a/an` →
    `it's`; `their the/not/going` → `they're` (guard "going-away" đúng);
    `is to big` → `is too big` (copula + tính từ đóng; "went to big cities"
    đúng); `better then` → `better than` (so sánh-hơn, quét ngược ≤4 từ,
    dừng tại dấu kết câu).
  - **Đại từ "i" thường giữa câu** → "I" (guard "i.e.", không đụng
    "iPhone", không trùng L6 đầu câu, không đụng dòng code "i = i + 1").
- Mới: kind FFI `word_confusion` (12) + `lowercase_pronoun` (13) — sync đủ
  6 điểm (2 header C + assist_result.c + cli verify + Swift CoreBridge).

**English** — 0.2.51 completes English grammar support (previously only the
a/an rule was active — every common confusion pair was silently ignored):
should/could/would/must of → have; your welcome → you're welcome; its been →
it's been; their going → they're going; is to big → is too big; better then →
better than; and the standalone lowercase pronoun "i" → "I". All closed lists
with tight guards, English-dominant sentences only, never touching Vietnamese
text: 760 tests, English FPR 0/14 guard probes, Vietnamese FPR 0/981, recall
83.7%, clippy 0, preflight 23/23.

## [0.2.50] — 2026-10-28

### Added
- **Ctrl + Shift + F9 trên cả 3 nền tảng** (trước đây chỉ Windows): macOS đọc
  vùng chọn qua IMK và thay bằng văn bản chuyển mã; Linux IBus/Fcitx5 đọc
  surrounding text (anchor/cursor), chuyển mã rồi xoá/commit đúng ranh giới
  ký tự — đều có gate S3 (ô mật khẩu không đọc/sửa), chống TOCTOU và trần
  64 000 ký tự.
- **Kiểm tra chính tả từ điển hệ điều hành trên Windows** (D39): Spell
  Checking API với **cổng bắt buộc `vi-VN`** — máy không có bộ kiểm tra
  tiếng Việt (mặc định) thì giữ hành vi cũ (fail-open "thà im lặng còn hơn
  báo sai"); worker COM riêng + timeout 50 ms + cầu chì.
- **Bật được dự đoán văn bản từ bảng điều khiển** trên cả 3 nền tảng
  (trước đây `suggest.enabled` chỉ sửa tay config — dự đoán thực chất
  KHÔNG THỂ bật).
- **Chẩn đoán trợ lý trên Linux**: `TEXTVN_ASSIST_DEBUG=1` log lý do gate
  mở/đóng (chỉ cờ + số đếm, không log nội dung câu — S2).

### Fixed
- **Vòng 35 — audit 3 lượt trợ lý ngữ pháp 3 nền tảng** (R1 2 agent song
  song, R2 đối kháng diff, R3 docs): 2 HIGH (mask phá tên file
  "config.toml"/"Báo cáo.xlsx" với confidence 100; macOS chord F9 chết vì
  Swift dispatch tĩnh qua existential), 5 MEDIUM (PANIC "10," khi câu không
  có token chữ; Windows trợ lý chết fresh-install — assist_mode default Off;
  layout checkbox dự đoán đè viền; S3 macOS cửa sổ hẹp; comma_at_end bêu
  ký kết email) và 10 LOW đã fix. Kiểm chứng CI: 4 vòng CI liên tiếp, mỗi
  vòng sửa xong mới sang vòng kế.

**English** — 0.2.50 completes the grammar-assistant loop across all three
platforms: Ctrl+Shift+F9 text transcode now works on macOS and Linux too
(selection read + charset conversion with S3 password-field gates, TOCTOU
re-checks and a 64 000-char cap); Windows gains OS spell checking (D39)
through the Windows Spell Checking API with a mandatory `vi-VN` capability
gate and fail-open semantics; the text-prediction toggle is finally
reachable in the settings UI on all three platforms; and Linux gains a
`TEXTVN_ASSIST_DEBUG=1` diagnostics mode. A three-pass audit (two parallel
read-only agents, then adversarial review of the diff, then docs) fixed
2 HIGH, 5 MEDIUM and 10 LOW findings — including a Swift existential-dispatch
bug that silently killed the macOS transcode chord, a panic on "10,"
(digit tokens are not Words), and a fresh-install bug that left the Windows
assistant dead until the tray wrote its first config. 755 tests, FPR 0/981,
recall 83.7%, clippy 0, preflight 23/23.

## [0.2.49] — 2026-10-26

> Release candidate — mọi file đã ký GPG + Sigstore; Windows chưa có Authenticode, macOS chưa
> notarization (`docs/release/signing.md`).

### Added
- **Ctrl + Shift + F9 chuyển đổi văn bản bôi đen thông minh** (R2-139, Windows): tự nhận ra
  và sửa **chữ lạ khi mở file sai bảng mã** (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`, kể cả trên
  Windows tiếng Việt và văn bản bị mã hoá hai lần), **bảng mã cũ** TCVN3/ABC (.VnTime) ·
  VNI-Windows · Unicode tổ hợp → **bảng mã TextVN đang dùng**, và **chữ gõ nhầm khi quên bật
  tiếng Việt** (`tooi ddi hocj` → `tôi đi học` theo kiểu gõ đang dùng; từ đã có dấu giữ
  nguyên). Chạy cả ở Notepad++, ô nhập WinForms/Delphi; Ctrl+Z của ứng dụng hoàn tác.
- **Nhắc mở lại ứng dụng sau khi cập nhật** (R2-138): bộ gõ được nạp vào ứng dụng lúc ứng
  dụng mở, nên Zalo, VS Code… đang chạy vẫn dùng bản cũ cho tới khi mở lại. Trang cuối bộ cài
  và khay hệ thống (một lần, khi máy chưa khởi động lại) giờ nhắc việc này.

### Fixed
- **Ô nhập kiểu WinForms/Delphi và ô EDIT của Windows** (R2-137): popup trợ lý hiện nhưng
  "Sửa tất cả", Alt + 1…3, Ctrl + Space không sửa được — nay sửa được (đọc lại chữ để xác
  minh, thay một lần, Ctrl+Z của ô nhập hoàn tác được).

Kiểm chứng E2E trên ứng dụng thật: Notepad, Edge, Notepad++, VS Code và ứng dụng 32-bit
(RichEdit, EDIT — đường bộ gõ x86 như Zalo) 75/75 + Notepad/Word/Excel 28/28.

**English** — 0.2.49: Ctrl+Shift+F9 now converts a selection smartly (R2-139, Windows): it
repairs **garbled text from a file opened with the wrong encoding** (UTF-8 read as
cp1252/cp1258, including double encoding), converts **legacy encodings** (TCVN3/ABC,
VNI-Windows, decomposed Unicode) **to the encoding TextVN currently outputs**, and
**re-types text typed while Vietnamese was off** (`tooi ddi hocj` → `tôi đi học`, with the
current input method; words that already carry marks are kept) — also in Notepad++ and
WinForms/Delphi text boxes, undoable with the app's Ctrl+Z. After an update, the installer's
last page and a one-time tray notice remind you to **reopen running applications** (they
keep the old input method until reopened — R2-138). Fixed: in standard Windows EDIT,
WinForms and Delphi text boxes the assistant's Fix all, Alt+1..3 and Ctrl+Space now apply
(R2-137). Real-app E2E: Notepad, Edge, Notepad++, VS Code and 32-bit RichEdit/EDIT hosts
75/75; Notepad/Word/Excel 28/28.

## [0.2.48] — 2026-10-26

> Release candidate — mọi file đã ký GPG + Sigstore; Windows chưa có Authenticode, macOS chưa
> notarization (`docs/release/signing.md`).

### Changed
- **Phím tắt trợ lý chỉ còn Ctrl + Space** trên Windows và Linux (R2-136, theo yêu cầu): bỏ
  **Alt + Space** của 0.2.47 vì tổ hợp này thuộc menu cửa sổ và các trình khởi chạy
  (PowerToys Run, Copilot, ChatGPT…) — TextVN không bao giờ giữ Alt + Space nữa, kể cả khi
  câu đang có lỗi. Ctrl + Space tới thẳng bộ gõ: câu có lỗi thì sửa hết, câu đúng thì dự
  đoán, không có việc thì phím tới ứng dụng nguyên vẹn. macOS giữ **Option + Space**;
  **Alt + 1…3** (áp gợi ý khi popup đang hiện) giữ nguyên.

Kiểm chứng E2E trên ứng dụng thật: Notepad, Edge, Notepad++, VS Code 44/44 (Alt + Space
khi câu có lỗi vẫn mở menu cửa sổ, chữ không đổi) + Notepad/Word/Excel 28/28.

**English** — 0.2.48: the assistant hotkey is Ctrl+Space only on Windows and Linux
(R2-136, owner's decision): 0.2.47's Alt+Space is dropped because it belongs to the
window menu and to global launchers (PowerToys Run, Copilot, ChatGPT) — TextVN never
takes Alt+Space any more, even with errors pending. Ctrl+Space reaches the input method
directly: it fixes the sentence when it has errors, predicts when it is clean, and passes
to the app untouched when there is nothing to do. macOS keeps Option+Space; Alt+1..3
(apply a suggestion while the panel is shown) is unchanged. Real-app E2E: Notepad, Edge,
Notepad++ and VS Code 44/44 (Alt+Space with errors pending still opens the window menu,
text unchanged); Notepad/Word/Excel 28/28.

## [0.2.47] — 2026-10-26

> Release candidate — mọi file đã ký GPG + Sigstore; Windows chưa có Authenticode, macOS chưa
> notarization (`docs/release/signing.md`).

### Added
- **Chữ mờ gợi ý sẵn** (R2-134, khi bật dự đoán `suggest.enabled`): Windows hiện
  phần chữ Tab sẽ chèn bằng **chữ xám ngay sau con trỏ** (kiểu Grammarly/Copilot)
  — chỉ là hình vẽ, không bao giờ lọt vào văn bản; macOS hiện ô `⇥ …` trong bảng
  ứng viên, Linux hiện dòng phụ `⇥ …` của bảng IBus/Fcitx5.
- **Phím tắt trợ lý sửa hết rồi dự đoán** (R2-131/R2-133): Windows/Linux
  **Ctrl + Space hoặc Alt + Space**, macOS **Option + Space** — câu có lỗi thì
  sửa **hết** một lần (kể cả lỗi chính tả có gợi ý, hoàn tác Ctrl+Z 5 giây), câu
  đã đúng thì dự đoán như Tab, bấm tiếp dự đoán theo câu đã sửa; không có gì để
  làm thì phím tới ứng dụng như thường (Alt + Space vẫn mở menu cửa sổ, Option +
  Space vẫn gõ dấu cách cứng). Windows: **Ctrl + Shift + G** mở panel chọn từng lỗi.

### Fixed
- **Nút "Sửa tất cả" và chọn gợi ý sửa được lỗi chính tả** (R2-131): trước đây lỗi
  chính tả (loại chỉ-gợi-ý) bị bỏ qua cả khi bạn chủ động bấm sửa — nút không
  hiện/không làm gì, Enter/Alt+1…3/click trên gợi ý bị nuốt. Chế độ Tự sửa lỗi
  vẫn chỉ tự sửa lỗi chắc chắn.
- **Alt + 1…3 áp gợi ý trên Windows** (R2-133): Windows không đưa tổ hợp Alt vào
  bộ gõ theo đường thường nên phím không bao giờ tới TextVN — nay đăng ký phím
  dành riêng; khi không có popup, Alt + 1…3 vẫn tới ứng dụng.
- **Notepad++ và ứng dụng Win32 cũ (IMM32)** (R2-132): popup hiện nhưng mọi kiểu
  sửa (nút, Enter, Alt + 1…3, phím tắt) không làm gì vì ứng dụng không cho bộ gõ
  sửa chữ đã gõ — nay TextVN đọc lại dòng hiện tại để xác minh rồi sửa bằng phím;
  chờ bạn nhả Ctrl/Alt để không thành Ctrl+Backspace (xoá cả từ).
- **VS Code và ứng dụng Electron** (R2-135): phím tắt bấm khi đang gõ dở từ
  từng làm hỏng chữ (editor vẽ lại vùng nhập bất đồng bộ) — nay chốt từ rồi sửa
  sau một nhịp ngắn; Alt + Space khi không có gì để sửa vẫn mở menu cửa sổ ở
  cửa sổ khung tự vẽ (VS Code). VS Code không vẽ chữ mờ (đã có gợi ý riêng),
  Tab dự đoán vẫn hoạt động.

Kiểm chứng E2E trên ứng dụng thật: Notepad Win11, Edge (engine của Teams/WebView2),
Notepad++ và VS Code (Electron) 44/44 (`tools/win/e2e-assist-apps.ps1`) +
Notepad/Word/Excel 28/28. Chi tiết: `docs/50-assist/P6-TEXT-PREDICTION.md` §2b.

**English** — 0.2.47: ghost text — with prediction enabled, Windows draws the
text Tab would insert in grey right after the caret (drawn only, never typed
into the document); macOS shows a `⇥ …` candidate, Linux a `⇥ …` auxiliary line
(R2-134). One assistant hotkey fixes then predicts: Ctrl+Space or Alt+Space on
Windows/Linux, Option+Space on macOS — fixes every user-fixable issue of the
sentence at once (spelling suggestions included, Ctrl+Z undo for 5 s), predicts
like Tab when the sentence is clean, and passes the key to the app when there is
nothing to do (Alt+Space still opens the window menu); Ctrl+Shift+G opens the
pick-and-fix panel on Windows (R2-131/R2-133). Fixed: "Fix all" and picking a
suggestion now apply spelling suggestions (R2-131); Alt+1..3 now applies a
suggestion on Windows via TSF preserved keys and is handed back to the app when
no panel is shown (R2-133); Notepad++ and other IMM32 apps can now be fixed —
the current line is verified through IMR_DOCUMENTFEED and the fix is typed as
keys after modifiers are released (R2-132); VS Code/Electron: the hotkey pressed
mid-word no longer garbles text (the fix runs a beat later, after the editor
re-renders its input), and Alt+Space with nothing to fix still opens the window
menu of frameless windows; no ghost text in VS Code (it has its own) while Tab
prediction works (R2-135). Real-app E2E: Notepad, Edge (Teams/WebView2 engine),
Notepad++ and VS Code (Electron) 44/44; Notepad/Word/Excel 28/28.

## [0.2.46] — 2026-10-26

> Release candidate — mọi file đã ký GPG + Sigstore; Windows chưa có Authenticode, macOS chưa
> notarization (`docs/release/signing.md`).

### Added
- **Dự đoán văn bản song ngữ** (P6, bật bằng `"suggest": { "enabled": true }`,
  mặc định tắt): **Tab** hoặc **Ctrl + Space** hoàn tất từ/từ ghép (`phát` →
  `phát triển`), dự đoán cụm tiếp theo theo ngữ cảnh (`Kính gửi` → `quý khách
  hàng`, `Thank you` → `very much`), Tab liên tiếp duyệt vòng ứng viên.
  Ctrl + Space không có dự đoán thì mở **trợ lý ngữ pháp theo yêu cầu** cho câu
  đang gõ hoặc vùng bôi đen (Windows, macOS). Nghiệp vụ:
  `docs/50-assist/P6-TEXT-PREDICTION.md`.

### Fixed
- **Dự đoán đúng ngữ cảnh** (R2-120): `phát triển`+Tab ra `phát triển phần mềm`
  (trước: "phát triển khai"), `hợp đồng`+Tab ra `hợp đồng kinh tế` (trước: "hợp
  đồng ý"); dự đoán ngay sau từ vừa gõ, không cần dấu cách; `Xin`+Tab ra `Xin
  chào` (không "Xin Chào"); tên riêng/tiêu ngữ đúng chính tả (`Việt Nam`,
  `Độc lập - Tự do - Hạnh phúc`); không nối cụm sang dòng mới; kiểu dấu mới
  (`hoà`) được hỗ trợ và ứng viên theo kiểu dấu của bạn.
- **Ctrl + Space không làm lệch chữ** (R2-121): khi không có dự đoán/lỗi, từ
  đang gõ được chốt và TextVN quên đuôi văn bản trước khi phím tới ứng dụng
  (Windows, macOS, Linux) — trước đây có thể xoá/sửa nhầm chữ ở phím kế tiếp.
- **Tab không bị panel trợ lý tự hiện nuốt mất** (R2-122): Tab chỉ chuyển lỗi
  trong panel bạn tự mở bằng Ctrl + Space; panel tự hiện sau dấu câu thì Tab
  đóng panel và tới ứng dụng (chuyển ô form/Excel, thụt lề) như cũ.
- **Không thừa Tab/Ctrl + Space ở ứng dụng cũ (IMM32/CUAS)** khi TextVN đã dùng
  phím đó để hoàn tất/gõ tắt/mở trợ lý (R2-123).
- **Bộ cài không để lại hai bản TextVN** (R2-124): cài cho mọi người dùng tự gỡ
  bản cài riêng cho tài khoản (cùng mục khởi động cũ) — trước đây hai bản chạy
  lẫn nhau, tray/bộ gõ cũ có thể chạy thay bản mới (mất trợ lý ngữ pháp, toast);
  cài cho tài khoản khi đã có bản cho mọi người dùng thì dừng và hướng dẫn.
- **Panel trợ lý ngữ pháp luôn hiện được** (R2-125): trước đây panel không neo
  được vị trí caret ở nhiều ứng dụng (Notepad Win11…) nên **không bao giờ hiện**,
  chỉ còn toast — sửa 3 lỗi tìm vị trí caret, thêm đường lui tới control đang
  focus. Kiểm chứng E2E trên Notepad/Word/Excel thật.
- **Bỏ toast "Phát hiện N lỗi"** (R2-126): lỗi được sửa ngay trong panel thay vì
  thông báo gây nhiễu; Ctrl + Space chỉ dùng phím khi panel thật sự hiện.
- **Panel dễ đọc hơn** (R2-127): chữ to hơn ~1,3 lần; sửa lỗi vẽ câu và danh sách
  lỗi bằng font hệ thống không có dấu tiếng Việt ("đi học" từng hiện "li hlc"),
  gạch đỏ đúng chỗ, không còn cắt chữ.
- **Gõ tắt/số tiền vượt phần đang gõ dở hoạt động trên Notepad Win11 và Word**
  (R2-128): `=15000000bc`+Space ra `Mười lăm triệu đồng chẵn` — trước đây ứng
  dụng từ chối sửa phần chữ trước vùng đang soạn nên bỏ qua im lặng.
- **Panel sống theo câu** (R2-129): câu sai hiện panel ngay ở dấu cách/dấu phẩy,
  sửa xong ẩn ngay; panel không chiếm phím gõ (Enter vẫn xuống dòng); Ctrl + Space
  để chọn và sửa bằng phím. **macOS và Linux cùng hành vi**; Linux chọn gợi ý bằng
  **Alt + 1…3** (trước đây phím số trần — gõ "giá 100" có thể áp nhầm gợi ý, và
  trên Fcitx5 phím số không chọn được gợi ý). Tray Windows chỉ còn toast khi đã
  tự sửa (không toast "phát hiện lỗi").
- **Bộ cài quét và dọn bản cũ** (R2-130): nâng cấp tại chỗ dọn sạch file của bản
  cũ (dữ liệu đổi tên, từ điển ngừng phát hành, DLL cũ) mà giữ cài đặt và bộ gõ;
  cài cho tài khoản khi đã có bản cho mọi người dùng thì hỏi gỡ bằng quyền quản trị.

**English** — Unreleased: bilingual text prediction (P6, opt-in via
`suggest.enabled`): Tab or Ctrl+Space completes words/compounds and predicts the
next phrase from context, repeated presses cycle candidates; Ctrl+Space with no
prediction opens the grammar assistant on demand (Windows, macOS). Fixed:
context-aware ordering (`phát triển`+Tab → "phần mềm", not "triển khai"),
prediction right after a word without a space, no Title-Casing of continuations,
correct proper nouns, no prediction across line breaks, new-style tone marks
(R2-120); Ctrl+Space no longer desyncs the composing word when it reaches the
app (R2-121); the auto-shown assistant panel no longer swallows Tab (R2-122); no
duplicated Tab/Ctrl+Space in IMM32/CUAS apps when TextVN consumed the key
(R2-123); the all-users installer now removes a leftover per-user install and
its stale autostart entry, and the per-user installer refuses to install next to
an all-users install (R2-124). After real-app E2E on Notepad/Word/Excel: the
grammar panel now always shows (caret anchoring fixed — it never appeared in
Windows 11 Notepad, R2-125); the noisy "N issues detected" toast is gone (R2-126);
the panel is larger and draws Vietnamese correctly (it used the stock system font
for the sentence and issue list, R2-127); macros/money shorthand that reach before
the composing text work in Notepad/Word (TSF `TF_E_RANGE_NOT_COVERED`, the
composition is now extended instead, R2-128); the panel is "live" — shown as soon
as the sentence has an error, hidden as soon as it is fixed, never stealing typing
keys, Ctrl+Space to navigate and apply (R2-129) — same behaviour on macOS and Linux,
where suggestions are now applied with Alt+1..3 instead of bare digits (bare digits
type digits again; on Fcitx5 they never selected anything); the tray only toasts
after an auto-fix; in-place upgrades remove the old
version's leftover files and the per-user installer can remove an all-users install
with UAC (R2-130). Real-app E2E: 28/28 (`tools/win/e2e-assist-p6.ps1`).

## [0.2.45] — 2026-10-25

### Added
- **Gói năng suất văn phòng P5** (kế hoạch `docs/50-assist/P5-OFFICE-PRODUCTIVITY-PLAN.md`,
  7 task, mỗi task qua 2 lượt review riêng) — tất cả **trên máy bạn**, không
  gửi dữ liệu đi đâu:
  - **Gõ tắt thích ứng kiểu chữ** (OP-001): `hd` → `hợp đồng`, `Hd` →
    `Hợp đồng`, `HD` → `HỢP ĐỒNG`; gõ tắt đã có chữ hoa chủ đích
    (`cty` = `Công ty TNHH`) giữ nguyên 100%.
  - **Bảo vệ công thức bảng tính** (OP-002): `=sum(a1:a10)`, `=if(...)`,
    `=vlookup(...)` ở đầu ô Excel/Sheets không còn bị bỏ dấu thành `=ũm(`;
    gõ chữ số vẫn bình thường (không phá kiểu gõ VNI); trong ngoặc kép của
    công thức vẫn gõ tiếng Việt.
  - **Gõ tắt số tiền** (OP-003): `=15450000d`+Tab → `15.450.000 đồng`,
    `=15450000vnd`+Tab → `15.450.000 VNĐ`, `=15000000bc`+Tab →
    `Mười lăm triệu đồng chẵn` (đọc chuẩn kế toán: mốt/tư/lăm, "không trăm
    lẻ", "chẵn" khi tròn nghìn, tối đa 12 chữ số).
  - **Cụm từ hành chính dễ nhầm** (OP-004): tự phát hiện & sửa "bổ xung"→
    "bổ sung", "xử dụng"→"sử dụng", "kiểm xoát"→"kiểm soát", "chính sác"→
    "chính xác", "sát nhập"→"sáp nhập", "thăm quan"→"tham quan",
    "tựu chung"→"tựu trung", "giành thời gian"→"dành thời gian"… (danh sách
    đóng, giữ hoa/thường, không bêu "xoay sở" hay "kiểm sát nhập").
  - **Khoảng trắng trước %** (OP-005): `đạt 100 %` → `đạt 100%`.
  - **Bảo vệ identifier coder & thuật ngữ kinh doanh** (OP-006):
    `useState`/`onPress`/`getElementById` không bị bỏ dấu giữa từ; thuật
    ngữ tech/business (KPI, EBITDA, margin, revenue, cargo, docker…) tự
    khôi phục; Tab hoàn tất giữ case chuẩn (`ebit`+Tab → `EBITDA`). Âm tiết
    tiếng Việt thường vẫn được bảo vệ (`cacs` vẫn ra `các`).
  - **Chuyển mã văn bản bôi đen** (OP-007): bôi đen → **Ctrl+Shift+F9** →
    tự nhận diện **TCVN3 (.VnTime)** / **VNI-Windows** / **Unicode tổ hợp**
    và chuyển về Unicode dựng sẵn (undo = Ctrl+Z của app). Nhận diện
    fail-safe: văn bản Unicode thật hoặc không đủ tín hiệu thì **không đụng**
    (không bao giờ "hạ cấp" văn bản đúng).

**English** — 0.2.45 ships the seven-task P5 office-productivity pack
(`docs/50-assist/P5-OFFICE-PRODUCTIVITY-PLAN.md`, each task double-reviewed),
all on-device: case-adaptive text macros (`hd`/`Hd`/`HD` → lowercase/capitalised/
UPPERCASE expansion, deliberate casing preserved), spreadsheet formula guard
(`=sum(...)`, `=if(...)`, `=vlookup(...)` typed at the start of a cell are no
longer accent-mangled; the VNI number-based input method keeps working), money
shorthand (`=15450000d`+Tab → `15.450.000 đồng`, `…vnd` → `VNĐ`, `…bc` →
`Mười lăm triệu đồng chẵn` with accounting-grade Vietnamese number reading),
office collocation fixes ("bổ xung"→"bổ sung", "xử dụng"→"sử dụng", "sát
nhập"→"sáp nhập" …, closed list with case preservation and guards for correct
phrases), percent spacing (`100 %` → `100%`), coder/business identifier
protection (`useState`, `onPress`, `getElementById` never accent-folded; KPI/
EBITDA/margin/revenue restored; Tab completion keeps canonical casing), and a
selection transcoder: select text and press **Ctrl+Shift+F9** to detect TCVN3
(.VnTime) / VNI-Windows / decomposed-Unicode and convert it to precomposed
Unicode (app undo applies). Detection is fail-safe — real Unicode text or
ambiguous input is never touched. Verified: 159 core unit tests, 95 Windows TSF
tests, 718 workspace tests, preflight 23/23, VI corpus 0/981 false positives.

## [0.2.44] — 2026-10-25

### Fixed
- **Audit sâu 3 lượt toàn bộ trợ lý ngữ pháp** (từng dòng code, từng nền
  tảng) — khoá loạt FP cuối cùng để đạt chuẩn "không bao giờ bêu chữ đúng":
  - **Luật lặp-từ-ranh-giới-câu (L9) bị tắt**: bêu văn phạm ĐÚNG ("Em gửi
    báo cáo cho anh. **Anh** xem giúp em nhé.", "Tôi không. **Không** sao
    cả.") — corpus lỗi thật không có row nào đo được recall ⇒ mất recall 0,
    đổi FP 0. Kind giữ nguyên cho ABI.
  - **Tiếng Việt gõ thiếu dấu không bị sửa thành tiếng Anh nữa**: "an toan
    la quan trong" từng có nguy cơ mở luật a/an — nhận diện bằng bộ **1.061
    âm tiết Việt không dấu** (sinh từ vn_freq3000, trừ giao điểm top-3000
    từ EN: "to"/"do"/"an" quá mơ hồ); "An buon qua", "email nay giup toi
    an tam" sạch.
  - **Mạo từ a/an**: mở bảng ngoại lệ theo GỐC TỪ (hour\*/honest\*/heir\*/
    honor\*/honour\* — "an hourglass/honorable/heirloom" hết bị sửa ngược
    thành "a"), thêm /juː/ còn thiếu ("a Unicode string", "a euphemism",
    "a ukulele"), số đọc gộp **11xx/18xx = eleven/eighteen hundred** ("an
    1100", "an 1850" hết bị bêu), bỏ typo chết "tosla".
  - **Tên file văn phòng**: "report.pdf", "query.sql", "baocao.docx"…
    (31 phần mở rộng mới) không bị chèn dấu cách giữa tên ("report. pdf").
  - **Ngoặc kép cong “ ”**: lặp từ trong trích lời ("cậu nói “không
    không” mà") và dấu chấm trong trích ("…“tôi đi. bạn ở lại”.") không
    còn bị bêu; **viết tắt Việt viết hoa** ("tôi học tại ĐH. y khoa") hết
    bị tự viết hoa chữ sau.
  - **Code**: khoảng trắng kép trước `#` (chuẩn PEP 8) và dòng code căn
    lề ("x = 5;  y = 6") không còn bị bêu.
  - **Đa nền tảng**: Fcitx5 fresh-install giờ BẬT trợ lý như IBus/Windows/
    macOS (struct zeroed = Tắt trước đây); ngữ nghĩa toàn-cục-AND-override
    đồng bộ cả 3 OS theo luật R2-28 (kill-switch không bị per-app "hồi
    sinh"); gate "câu < 3 từ" port sang macOS/Linux (1 câu = 1 hành vi);
    Linux reconnect cap 10 s khớp hợp đồng.
  - **Windows**: panel trợ lý scale theo **DPI màn hình** (màn 150 % hết
    bị nhỏ ~2/3); câu < 3 từ im trên mọi nền tảng; FixAll trên popup chỉ
    gợi ý không còn đóng panel mà không làm gì.
  - **Performance**: check chỉ chạy ở điểm kết thúc câu (bỏ check thầm ở
    space/,/: — kết quả từng bị vứt); diacritic variants giảm sinh biến
    thể 2-dấu-thanh vô nghĩa; ime_key p50 4,2 µs / p99 10,2 µs (ngân sách
    500 µs).

**English** — 0.2.44 ships a three-pass, line-by-line audit of the whole
grammar assistant and closes the last false-positive classes: the
cross-sentence word-repeat rule is disabled (it flagged perfectly correct
Vietnamese like "…cho anh. Anh xem…"); diacritic-less Vietnamese ("an toan
la quan trong") is now recognised via a 1,061-entry Vietnamese ASCII
syllable list so the a/an rule never rewrites it; the a/an exception lists
use word stems (an hourglass/honorable/heirloom), add missing /juː/ words
(a Unicode string, a euphemism) and grouped number readings (an 1100/an
1850 = eleven/eighteen hundred); 31 office file extensions are masked
("report.pdf" no longer gets a space inserted); curly quotes “ ” and
Vietnamese capital abbreviations (ĐH.) are handled; double space before an
inline `#` comment and aligned code lines are never flagged. Cross-platform:
Fcitx5 fresh installs now have the assistant ON like every other platform,
the global-AND-override semantics match Windows on macOS/Linux (R2-28),
the <3-words gate is ported to macOS/Linux, Linux reconnect caps at 10 s.
Windows: the assist panel scales with monitor DPI, and the sentence check
only runs at sentence end (measurements: ime_key p50 4.2 µs / p99 10.2 µs
against a 500 µs budget). VI corpus 0/981 false positives, EN 0/295,
83.7 % sentence recall unchanged; 43 adversarial probes clean; 701 tests,
clippy 0, preflight 23/23.

## [0.2.43] — 2026-10-25

### Fixed
- **Enter trên popup vùng bôi đen giờ áp "Sửa tất cả"** (audit vòng 30
  finding 6 — finding LOW cuối cùng của tính năng selection-check):
  trước đây phím Enter trên popup selection-check bị nuốt mà KHÔNG sửa
  gì, vì đường áp cũ `apply_assist_suggestion` cần selection RỖNG
  (precondition A2 của sửa đơn-từ) nên luôn no-op trên popup selection.
  Giờ Enter = cùng hành vi nút **"Sửa tất cả"**: thay đúng vùng chọn
  bằng câu đã sửa (`ApplySelectionFixSession` — precondition đọc-lại
  text vùng chọn **trong edit lock**, undo thuộc app Ctrl+Z), sau đó
  **re-check câu đã sửa** — sạch thì panel tắt, còn lỗi gợi ý thì popup
  hiện lại (parity với đường caret v0.2.42). Enter tiếp theo xuống dòng
  bình thường.

**English** — 0.2.43 fixes the last open finding (LOW) from the
selection-check audit: pressing Enter on a selection popup used to
swallow the key and apply nothing, because the old apply path requires
an EMPTY selection (its precondition targets single-word fixes). Enter
on a selection popup now behaves exactly like the "Sửa tất cả" (fix
all) button — the selected range is replaced with the corrected
sentence through the same guarded session (content re-verified inside
the edit lock before writing; Ctrl+Z belongs to the app) — and the
fixed sentence is re-checked afterwards, so the panel hides when clean
and keeps showing remaining suggestions otherwise (parity with the
caret path from 0.2.42). The next Enter inserts a newline as usual.
674 tests, clippy 0, preflight 23/23.

## [0.2.42] — 2026-10-24

### Added
- **Kiểm tra ngữ pháp vùng bôi đen** (vòng 30 — yêu cầu chủ repo): bôi đen
  một câu **bằng bàn phím** (Shift+←→↑↓/Home/End) khi trợ lý đang bật →
  sau 500 ms popup hiện cạnh vùng chọn với tổng số lỗi + nút **"Sửa tất
  cả"** — bấm thay ĐÚNG vùng chọn bằng câu đã sửa (undo = Ctrl+Z của app).
  Phân biệt bôi-đen-để-sửa vs đọc/copy: chuột không qua key sink; Ctrl
  (Ctrl+A/C/X) và Alt bị lọc; vùng >256 ký tự bỏ (không check nửa câu).
  Precondition đọc-lại text vùng chọn **trong edit lock** trước khi ghi —
  người dùng gõ xen giữa lúc popup hiện thì KHÔNG ghi gì.

### Fixed
- Audit 2 lượt (pass-1: 8 finding gồm 2 HIGH; pass-2: SHIP): nút chết cho
  vùng 257-500 (check_text cắt đuôi 256 — hạ MAX về 256); hook chỉ ở pha
  TEST mà Notepad/Chrome/VS Code bỏ pha đó (thêm vào cả OnKeyUp); thiếu
  reset_engine sau SetText; timer chết khi hwnd overlay bị destroy (IsWindow
  guard); F-key/Alt false-positive; timer ID hardcode 2 chỗ.
- **Verify upgrade-mix KHÔNG cần restart máy**: DLL 0.2.40 đang chạy trong
  app + tray 0.2.42 mới cài → DLL cũ reconnect pipe ≤10 s → Snapshot mới
  (assist:true sau migration) → trợ lý sống lại ngay trong app đang mở
  (wire IPC byte-identical giữa 2 bản — agent xác nhận bằng git diff).

**English** — 0.2.42 adds grammar check on selected text: keyboard-select
(Shift+arrows/Home/End) a sentence you are editing and after 500 ms the
popup appears anchored at the selection with the error total and the
"Sửa tất cả" button; clicking replaces exactly the selected range with
the corrected sentence (content re-verified inside the edit lock before
writing; Ctrl+Z belongs to the app). Read/copy selections never trigger it
(mouse selection does not pass the key sink; Ctrl/Alt filtered); selections
over 256 characters are skipped rather than half-checked. Two-pass audit:
8 findings fixed (2 HIGH — dead button for 257-500 chars; hook missed
Notepad/Chrome/VS Code which skip the TEST phase), then a final pass
verified Notepad/Word/Chrome scenarios and the no-reboot upgrade path
(old DLL in a running app picks up the new tray snapshot within 10 s and
the assistant comes back alive). 674 tests, clippy 0, preflight 23/23.

## [0.2.41] — 2026-10-24

### Fixed
- **Sửa trợ lý ngữ pháp CHẾT hoàn toàn khi nâng cấp** (user báo "popup
  dưới-phải + gạch chân + correct không hoạt động ở 0.2.40") — 3 root
  cause, audit agent xác nhận end-to-end qua bảng 11 cổng:
  1. `IpcState::default()` khởi `assist_enabled = false` — DLL nạp vào app
     khi tray chưa gửi Snapshot ⇒ tắt toàn bộ trợ lý. Default giờ = true
     (khớp tray + config).
  2. state.json từ bản ≤0.2.37 giữ `assist_enabled: false` theo DEFAULT CŨ
     mãi kể cả sau 0.2.38–0.2.40 (so version KHÔNG bắt được — audit F1
     CRITICAL). Migration persisted-flag `assist_default_on`: bật lại đúng
     MỘT lần; tắt tay sau đó giữ nguyên tắt (test kịch bản thật).
  3. Toast phát hiện lỗi bị xoá ở 0.2.38/28 — khôi phục làm **fallback
     popup dưới-phải 8 giây/lần** khi overlay tại caret không neo được
     (app không cho anchor); overlay hiện được thì toast im. Cooldown
     riêng slot (audit F2: nhầm dùng 60 s của auto-fix).
- Audit xác nhận thêm: Mode 2/A2/A3 mouse path sạch; Win11 balloon hiện
  đúng không cần AUMID; macOS/Linux không có pattern chết tương tự; VI
  0/981 + EN 0/295 FP và recall 100% giữ nguyên.

**English** — 0.2.41 fixes the grammar assistant being completely dead for
upgraders ("popup doesn't work, underline and correct don't work on
0.2.40") — three root causes, verified end-to-end by an independent audit
agent over an 11-gate table: (1) the input-method DLL defaulted its
kill-switch to off until the tray's first snapshot arrived; (2) a state
file from 0.2.37 or earlier kept the kill-switch off forever even after
upgrading through 0.2.38-0.2.40 — a one-time persisted-flag migration now
turns it back on exactly once, and turning it off afterwards sticks;
(3) the detection toast removed in 0.2.38 returns as an 8-second-cooldown
bottom-right fallback popup for apps where the caret panel cannot anchor.
Benchmarks unchanged: 0/981 VI + 0/295 EN false positives, 100% recall;
673 tests, clippy 0, preflight 23/23.

## [0.2.40] — 2026-10-24

### Fixed
- **Audit 3 nền tảng bởi 2 agent audit chuyên gia song song (24 finding)**:
  - **CRITICAL — article_an phá chữ Việt**: "an khang thịnh vượng" bị
    auto-fix thành "a khang"! Câu Vi-dominant giờ chỉ mở khoá a/an khi có
    từ-điểm tiếng Anh khác ("email"/"report"…); "a"/"an" tự thân KHÔNG là
    tín hiệu (probe chứng minh 12/12 case đúng).
  - **HIGH — UNESCO-class**: unesco/unicef đọc /juː/ → "a UNESCO" đúng,
    "an UNESCO" sai bị tha vì nằm nhầm danh sách đọc-từng-chữ.
  - **HIGH — macOS IMK default lệch**: mode "off" trong khi app UI là
    highlight → người nâng cấp không có block assist bị TẮT hoàn toàn dù
    UI nói đang bật; Linux fresh-install tương tự (memset = 0 = off).
  - Linux `lc_assist_kind_name` thiếu case 11 "article_an" (replace
    trượt vòng 24 — bẫy lặp lại); ipc.v1.md kind list đủ 11 loại.
  - Click panel: vùng KHÔNG tương tác trong panel giờ ĐÓNG panel; click
    ĐẦU TIÊN trên lỗi chọn-sẵn = Select (không sửa document do 1 click
    vô tình); toast khay đồng bộ chính sách 60s trên MỌI nền tảng.
- **Performance đo bằng engine thật (release)**: mọi đường IME 14–52 µs/
  check — gấp 200 lần dưới ngân sách 10 ms (probe riêng cho URL/TLD-heavy,
  8-từ-lạ, câu 256 ký tự VI/EN/mix).

### Added
- **Panel tự cập nhật sau fix** (yêu cầu chủ repo): A2/A3/Mode-2 re-check
  câu ĐÃ sửa — còn lỗi hiển thị phần còn lại với tổng mới, sạch thì tự tắt.
- **`assist.display.hide_on_auto_fix`** (tùy chọn, mặc định false): ẩn
  panel hoàn toàn khi chế độ Tự sửa lỗi đang chạy — người dùng muốn auto
  sửa lặng lẽ không bị làm phiền (config serde + schema + Swift parity).

**English** — 0.2.40 is a full three-platform audit round by two parallel
expert audit agents (24 findings). Critical: the article rule corrupted
real Vietnamese ("an khang" → "a khang") — in Vietnamese sentences it now
requires another English indicator word; UNESCO-class acronyms (read
/juː/) are fixed; the macOS input-method and Linux fresh-install defaults
now actually turn the assistant on (highlight), matching the settings UI;
the Linux IPC wire name for article_an is registered. The panel now
re-checks the corrected sentence after any fix (updates with remaining
issues or disappears when clean), a `hide_on_auto_fix` option hides the
popup while auto-correct runs, clicking a non-interactive panel area
dismisses it, and the first click on the pre-selected issue only selects.
Tray toasts are auto-fix-only with a 60-second cooldown everywhere.
Performance measured on the real engine: 14-52 µs per check (200× under
budget). Benchmarks unchanged: 0/981 VI + 0/295 EN false positives, 100%
recall; 671 tests, clippy 0, preflight 23/23.

## [0.2.39] — 2026-10-24

### Added
- **Panel trợ lý ngữ pháp bấm được bằng chuột** (vòng 27 — "như Grammarly"):
  - Nút **"Sửa tất cả"** trên panel — áp sửa cả câu (chỉ lỗi chắc chắn), có
    hoàn tác Ctrl+Z trong 5 giây;
  - **Click lên từ gạch chân** hoặc dòng lỗi → chọn gợi ý; **click lần nữa**
    → áp sửa từ đó (precondition caret + Undo giữ nguyên);
  - Click **trong** panel không còn làm panel tắt (chỉ click ngoài mới tắt).
- Auto-correct mode (bật `Tự sửa lỗi` + `An toàn`) giữ nguyên hành vi: tự
  sửa câu tại dấu kết thúc câu, toast 60 giây/lần kèm Ctrl+Z.

**English** — 0.2.39 makes the grammar-assistant popup fully mouse-driven:
a "Sửa tất cả" (fix all) button applies the whole-sentence correction with
the 5-second Ctrl+Z undo; clicking an underlined word or an issue row
selects that suggestion and clicking again applies it; clicks inside the
popup no longer dismiss it (only outside clicks do). Auto-correct mode
keeps fixing qualifying sentences automatically at the sentence end as
before. Corpora unchanged: 0/981 (VI) and 0/295 (EN) false positives,
100% recall.

## [0.2.38] — 2026-10-24

### Changed
- **Trợ lý ngữ pháp bật mặc định ở chế độ Chỉ gợi ý** (0.2.38): fresh install
  và người dùng chưa từng tắt thấy panel gợi ý ngay; *Tự sửa lỗi* vẫn tắt
  (không bao giờ tự sửa nếu người dùng không bật thêm); kill-switch
  (công tắc khay/menu) vẫn tắt được tức thì. Đồng bộ đủ 3 nền tảng:
  config serde + schema + Swift ConfigModel + Windows tray state +
  Linux `assist.mode` thiếu-khoá → highlight + kill-switch default theo
  config (Windows state, Linux `lc_state_read_assist_enabled`).

### Added
- **Panel tổng lỗi kiểu Grammarly tại caret**: câu vừa gõ với **gạch chân đỏ
  đúng từng từ/đoạn sai** + dòng tổng kết "N lỗi trong câu vừa gõ" — thay
  cho toast khay thường xuyên.
- **Câu mix vượt Grammarly** (vòng 26): whitelist lặp từ chọn theo
  **TỪNG TỪ** ("check the the file nhé" bắt, "I know that that is true nhé"
  bỏ qua — cùng một câu); guard tên riêng hô gọi + ngoặc kép áp mọi ngôn
  ngữ ("Lê Lê ơi", nói "không không"); a/an chạy cả trong câu Việt mix
  ("please send a email to tôi nhé" → "an email").

### Fixed
- **Toast khay không còn gây noise**: toast "Phát hiện N lỗi" bị loại hoàn
  toàn (popup tại caret đã hiển thị đủ); toast duy nhất còn lại là khi
  *Tự sửa lỗi* vừa thay text — cooldown 60 giây, kèm nhắc Ctrl+Z.

**English** — 0.2.38 turns the grammar assistant **on by default in
suggest-only mode** (auto-fix stays opt-in; the tray/menu switch still kills
it instantly), synced across all three platforms. The caret popup now shows
the typed sentence with a red underline under each wrong word plus a
Grammarly-style total ("N lỗi trong câu vừa gõ"), replacing the frequent
tray toast — that toast now fires only after auto-fix actually rewrote a
sentence (at most once per 60 seconds, always with the Ctrl+Z hint). Mixed
sentences get stronger than Grammarly: the duplicate-word whitelist is
chosen **per word** ("check the the file nhé" is caught while "I know that
that is true nhé" passes — same sentence), vocative/proper-name and quoted
speech guards apply to accented Vietnamese names too, and the a/an rule
runs inside Vietnamese sentences ("please send a email to tôi nhé" → "an
email"). Measured on labeled real-text corpora: Vietnamese 0/981 and
English 0/295 false positives, recall 100% on both labeled error sets;
668 tests pass, preflight 23/23.

## [0.2.37] — 2026-10-24

### Added
- **Trợ lý ngữ pháp tiếng Anh (v1)**: câu En-dominant được kiểm tra — rule
  mạo từ **a/an theo ÂM** (không theo chữ) với danh sách ngoại lệ đóng
  ("a university", "an hour", "an MBA" đọc-chữ, "a NATO" đọc-từ, "an 8-bit"
  đọc số; dạng mơ hồ historic/FAQ/SQL → không bao giờ soi; dạng lạ → im
  lặng — fail-open); duplicate_word học lặp hợp lệ tiếng Anh ("that that",
  "had had", "very very", "ha ha"…) nhưng vẫn bắt lỗi thật ("the the book",
  "They they said"); guard lặp tên riêng hô gọi + từ trong ngoặc kép.
  Kind mới `article_an` (FFI 11) wired đủ: Rust FFI + header C (2 bản) +
  C `lc_assist_kind_name` + Swift CoreBridge/AssistModel + ABI verify.
- **Câu mix Việt–Anh văn phòng** ("vui lòng check lại file report nhé",
  "Anh gửi report cho team ASAP") được kiểm tra — trước đây bị bỏ qua im
  lặng. Gate ngôn ngữ mới: Vi-dominant / En-dominant / im-lặng (fail-open);
  các luật trung lập (khoảng trắng, dấu câu, lặp từ, viết hoa) chạy ở cả
  hai ngôn ngữ; luật riêng ngôn ngữ tự chặn trên nội dung không thuộc.
- **Hoàn tất từ bằng Space** (tùy chọn `suggest.trigger = "tab_and_space"`,
  mặc định `tab` — không đổi hành vi cũ): Space cũng hoàn tất từ đang gõ
  dở và kèm chính dấu cách; **nghiêm ngặt hơn Tab** — không bao giờ viết
  lại từ đã đúng ("test"+Space = "test "). Completion nạp đủ nguồn
  **3000 từ tần suất** (`data/en_freq3000.txt` — trước đây chỉ ~100 mục).

### Fixed
- 4 bug SỐNG phía Việt phát hiện bởi audit expert (agent song song): chèn
  space giữa viết tắt/tên miền ("e.g."→"e. g.", "a.m."→"a. m.", "U.S."→
  "U. S.", "example.com"→"example. com" — đã SỐNG từ trước khi có EN);
  khoảng trắng trước dãy chấm rời (". . ."); viết hoa hoá code/danh sách
  đề mục ("i = i + 1"→"I = i + 1", "i. introduction"→"I. Introduction").
  Đo lại: **VI 0/981 FP (giữ nguyên), EN 0/295 FP (corpus Tatoeba thật,
  CC BY 2.0 FR attribution đầy đủ), recall 100 % cả hai tập lỗi có nhãn**;
  666 test pass, preflight 23/23.

**English** — 0.2.37 ships the English grammar assistant v1: sentences
dominated by English are now checked — a new sound-based a/an article rule
with closed exception lists (a university, an hour, an MBA, a NATO, an
8-bit; ambiguous forms like historic/FAQ are never flagged), and the
duplicate-word rule learns English-valid repeats (that that, had had, very
very) while still catching real errors. Office-style mixed Vietnamese/
English sentences ("vui lòng check lại file report nhé") are now covered —
previously silently skipped. Four live Vietnamese-side bugs fixed: spaces
auto-inserted inside abbreviations/domains (e.g., a.m., U.S., example.com),
spaced ellipsis, and capitalizing code/list markers. Word completion gains
an optional Space trigger (`suggest.trigger = tab_and_space`, off by
default — never rewrites an already-correct word) and now draws on the
full 3000-word frequency dictionary. Measured on labeled real-text
corpora: Vietnamese 0/981 and English 0/295 false positives, recall 100 %
on both labeled error sets.

## [0.2.36] — 2026-10-23

### Fixed
- **Trợ lý ngữ pháp: FPR 0 % trên tập sạch 981 câu** (audit vòng 23 "như
  Grammarly" — 3 lượt rà từng dòng toàn bộ tính năng): bỏ 5 cặp trong bảng
  typo mà **từ "sai" là từ hợp lệ** (vi phạm chính sách Fassist-96 của chính
  repo) — trước đây Mode 2 **tự sửa chữ đúng thành chữ sai**: "quyến rũ"→
  "quyển", "thuở xưa"→"thướ", "đả thông"→"đã", "áy náy"→"ấy", "ngôi điếm"→
  "điểm" (chứng minh 5/5 câu đúng bị bêu + auto-fix sai). Đổi Recall câu
  87,8 % → 83,7 % (2 ca mơ hồ chuyển về nhóm cần ngữ cảnh — đánh đổi một
  chiều, không bao giờ lấy FPR đổi recall).
- Bổ sung 4 từ lặp hợp lệ vào whitelist (hết báo nhầm "năm năm trước", "ở
  đâu đâu", "chộp chộp, bum bum") — 4/5 FP còn lại trên tập sạch sạch hết.
- Gate nhận diện tiếng Việt (L0) giờ nhận **chữ hoa có dấu** — câu VIẾT HOA
  TOÀN BỘ không còn bị bỏ qua kiểm tra.
- Hồi quy mới: `khong_beu_tu_hop_le_tung_o_bang_typos`,
  `cau_viet_hoa_toan_bo_van_la_tieng_viet`; corpus dòng mềm `~` hóa cứng.
  Số đo đầy đủ: `docs/50-assist/P4-12-RECALL-REAL.md` §11.

**English** — 0.2.36 hardens the grammar assistant to the "never flags
correct text" bar (three full line-by-line audit passes): removed 5 typo-table
pairs whose "wrong" word is actually a valid Vietnamese word (they violated
the project's own no-ambiguity policy and made auto-correct rewrite correct
words: "quyến rũ" → "quyển", "thuở xưa" → "thướ", "đả thông" → "đã", "áy náy"
→ "ấy", "ngôi điếm" → "điểm"; proven on 5/5 correct sentences). Whitelisted 4
more valid reduplications ("năm năm trước", "ở đâu đâu", "chộp chộp, bum bum")
and taught the Vietnamese-language gate to recognize accented ALL-CAPS
sentences. Measured honestly on the labeled real-text corpora (CLI rebuilt
before measuring): **false positives 0.5 % → 0.0 % (0 of 981 clean sentences,
Wilson 0.0–0.4)**; sentence recall 87.8 % → 83.7 % — a deliberate one-way
trade (never trade FPR for recall). New regression tests
(`khong_beu_tu_hop_le_tung_o_bang_typos`, `cau_viet_hoa_toan_bo_van_la_tieng_viet`);
full numbers in `docs/50-assist/P4-12-RECALL-REAL.md` §11.

## [0.2.35] — 2026-10-23

### Added
- **Kênh Debian/Ubuntu: PPA + lộ trình kho chính thức** — `debian/` hoàn chỉnh
  (control/rules multiarch `/usr/lib/<triplet>/fcitx5/`, changelog, copyright
  dep5, quilt; IBus engine đặt `/usr/libexec/` với component XML sinh đường
  dẫn cuối) + QA gate `ci-deb.yml`: **dpkg-buildpackage thật** trong container
  `debian:trixie` qua đúng đường Launchpad (cargo vendor → build `--offline`
  → kiểm 9 file bắt buộc + addon đúng triplet + XML không placeholder +
  lintian). `tools/release/prepare-ppa.sh` chuẩn bị + upload source package
  lên `ppa:hunglinhpt/textvn` (vendored — Launchpad build không Internet,
  R2-119). Lộ trình Debian chính thức (ITP/mentors/sponsor + email Debian
  Input Method Team) + audit deps: cây Linux chỉ cần serde/serde_json (đã có
  `librust-*-dev`). Checklist thêm B7f; `debian/changelog` = site 15 của
  check-version-sync.
- **Trợ lý ngữ pháp rà lại sâu sau D19 (audit 3 vòng, `W1…W5`/`R-7`)**:
  logic kế hoạch tự sửa gộp về **một nguồn duy nhất** (`textvn_ffi::plan_issues_from_raw`)
  dùng chung cho cả 3 nền tảng; FFI chỉ trả kế hoạch khi `delete_glyphs` == số
  char Unicode của đuôi xoá (bảng mã VNI/TCVN3 không thể xoá lệch trên
  Linux/macOS); `assist.per_app` nay có hiệu lực trên **Windows** như macOS.

### Fixed
- **Recall corpus thật 57,1% → 87,8%** (D19 · `a557d3a`): luật `diacritic_typo`
  nâng 12,5% → **75%** với FPR giữ **0,5%** (⇐/≥ chuẩn D19: recall ≥ 80%,
  FPR ≤ 2%) — thêm bảng cặp từ-sai-KHÔNG-phải-từ-hợp-lệ (chính sách
  Fassist-96) + đường tổng quát qua biến thể E1/E2/E3 giới hạn ngân sách
  ≤8 từ lạ/câu.
- **macOS (W1)**: xác minh `glyph == grapheme` trước khi xoá trong cả auto-fix
  lẫn undo — bảng mã xuất ≠ Unicode/NFD ⇒ từ chối sửa (fail-safe, ngang
  `verify_delete_range` của Windows) thay vì xoá thừa/thiếu ký tự.
- **Windows (W3)**: nhánh undo bất đồng bộ không xoá state sớm nữa — session
  tự xoá khi hoàn tác **thành công**; `Ctrl+Z` hiếm khi fail không còn nuốt
  phím đồng thời mất khả năng hoàn tác.
- CI: `cargo fmt` cho bảng TYPOS mới (`1e52a99`).

> **Bản release candidate** — ký GPG + Sigstore như các bản trước, kèm chữ ký
> git trên commit/tag (huy hiệu "Verified" — R6). **Chưa** Authenticode
> (Windows — chờ SignPath Foundation) và **chưa** Developer ID + notarization
> (macOS) — vẫn là candidate, không phải production.

**English** — 0.2.35 introduces the experimental **grammar assistant**
(check-as-you-type suggestions and opt-in whole-sentence auto-fix for
duplicate words, punctuation spacing, capitalization and misplaced tone
marks; off by default, fully on-device), raising real-corpus recall from
57.1 % to 87.8 % at 0.5 % FPR, unifying the fix-planning logic into one
shared source across Windows/macOS/Linux with glyph-count verification
before deleting, enabling `assist.per_app` on Windows, and adding the
Debian/PPA packaging channel (`ppa:hunglinhpt/textvn`).

## [0.2.34] — 2026-10-18

### Added
- **Kênh phân phối Linux cho distro store** — AUR chuẩn bị xong
  (`packaging/arch/PKGBUILD` + `.SRCINFO` build từ mã nguồn tag; QA gate
  `ci-aur.yml` chạy **makepkg thật** trong container `archlinux:base-devel`:
  build, test engine, `.SRCINFO` diff, đủ file, version khớp Cargo.toml);
  `tools/release/publish-aur.sh` đẩy lên AUR (từ chối lệch version; SSH key
  cục bộ hoặc secret `AUR_SSH_KEY`). Flathub/Snap/AppImage bị loại **có căn
  cứ** (IME hệ thống cần nạp bởi ibus-daemon/fcitx5 trên host — sandbox của
  các format đó không cho phép; phân tích đầy đủ:
  `docs/release/linux-distribution.md` kèm ma trận auto-update từng kênh).
  Checklist phát hành thêm bước B7e; README hàng Linux trỏ AUR + doc.

### Fixed
- **Fcitx5 addon nâng chuẩn C++20** (R2-117): header Fcitx5 6.x (distro
  rolling như Arch) dùng `std::ranges`/`std::span`/`starts_with` trong template
  body — C++17 không biên dịch được (bắt được bởi chính gate `ci-aur`); Ubuntu
  không ảnh hưởng (libstdc++ 11+ đủ). Kèm theo: `publish-aur.sh` từ chối push
  khi `sha256sums` lệch archive tag thật (R2-118).

- **Homebrew tap chính thức cho macOS** — `brew install --cask
  hunglinhpt/tap/textvn` ([hunglinhpt/homebrew-tap](https://github.com/hunglinhpt/homebrew-tap)):
  cask dùng stanza `input_method` để đặt `TextVN-IM.app` vào
  `~/Library/Input Methods` (per-user, không cần admin) và `app` cho
  `TextVN.app` vào `/Applications`; pin `sha256` của mac zip từ
  `SHA256SUMS.txt` đã verify GPG. Cask tự cập nhật theo release mới bằng
  workflow `sync-cask.yml` trong tap (mỗi 6 giờ hoặc chạy tay — verify GPG
  rồi mới ghi, không cần PAT); CI lint kiểm cú pháp Ruby + stanza bắt buộc.
  README/user-guide trỏ cài đặt macOS về tap (cách khuyên dùng, cạnh `.pkg`
  và bản nén); checklist phát hành thêm bước B7d (kiểm tap sau publish);
  `AGENTS.md` mới ở gốc repo buộc mọi agent đi theo workflow/checklist.

- **Tự động nộp bản cập nhật lên Microsoft Store** — workflow `store-publish`
  (`.github/workflows/store-publish.yml`): lấy asset `.msix` của một GitHub Release, kiểm chữ
  ký GPG + `SHA256SUMS.txt` + Sigstore (đúng `release.yml` trên đúng tag) + luật Partner
  Center/identity/Version (`tools/release/verify-store-package.sh`, `verify-msix.py
  --expect-name`), rồi nộp lên sản phẩm `9NV2R7JNNFGK` bằng Microsoft Store Developer CLI
  (`msstore`, ghim v0.4.3 + SHA-256). Chạy tay (`check` = chỉ đọc trạng thái Store, `draft`,
  `submit`) hoặc tự động sau mỗi lần phát hành khi đặt repo variable `STORE_AUTO_PUBLISH`;
  chặn sớm khi Store đang chứng nhận hoặc đã có version ≥ gói, không in JSON submission (có
  `FileUploadUrl`) ra log công khai. Cần chủ tài khoản cài app Entra + 4
  secret một lần — `docs/release/msix-submission.md` §7.

### Docs
- **TextVN đã có trên Microsoft Store** (gói MSIX, phát hành 2026-10-11):
  https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB — cài bằng
  `winget install --id 9NV2R7JNNFGK -s msstore`. README (badge + bảng tải về), hướng dẫn
  sử dụng, `HUONG_DAN_SU_DUNG.txt`, tài liệu nộp Store và ghi chú phát hành tự sinh của
  `release.yml` trỏ link này.

**English** — 0.2.34 opens Linux distro-store distribution: an AUR package
built from the tagged source with a CI gate that runs real `makepkg` inside
an Arch container, plus a written analysis of why Flatpak/Snap/AppImage
cannot host a system-level IME; fixes the Fcitx5 addon not compiling on
rolling distros (C++20, R2-117).

## [0.2.33] — 2026-10-18

### Changed
- **Mặc định sau cài = bật gõ tiếng Việt + dấu cũ, đủ 3 nền tảng** (0.2.33):
  `DiacriticStyle::Old` (`hòa`, `thủy`, `khỏe` — dấu trên `o`, `u`, `o` của vần mở
  `oa/oe/uy`) trở thành mặc định của `Config` (serde) và `EngineOptions` (core),
  default `ConfigModel.swift` (macOS), và fallback thiếu khoá `diacritic_style`
  của Linux settings. Người dùng cũ **giữ nguyên** lựa chọn của mình (chỉ thiếu
  khoá hoặc cài mới mới nhận mặc định). Spec P0-3 (bảng trường + ví dụ) đồng bộ.
  macOS SettingsView/menu và Windows tray hiển thị theo config, không đổi mã.
- Test cơ chế backspace/common-words ghim tường minh `DiacriticStyle::New` (danh
  sách từ viết theo dấu mới); kiểu cũ phủ riêng ở `telex_old_style`.

### Added
- **Ma trận 16 combo bảng mã × kiểu gõ** (`core/tests/charset_matrix.rs`): 4 bảng
  mã (`unicode_precomposed/unicode_decomposed/tcvn3/vni_windows`) × 4 kiểu gõ
  (`telex/vni/viqr/simple_telex`) gõ đúng toàn bộ ~200 từ thông dụng, chuỗi phím
  sinh từ bảng phím thật của từng kiểu gõ; anchor cứng TCVN3/VNI Windows
  (`đ`=0xAE/0xF1, `ò`=0xDF/0xF8…) chặn engine và encode lệch cùng một hướng.
- **Từ điển EN mở rộng lên top-3000 từ tần suất** (`data/en_common.txt` 31 → 104
  mục; nguồn google-10000-english MIT, lọc tự động bằng chính engine): gõ tiếng
  Anh tự trả lại đúng từ khi gõ dấu cách — `data/en_freq3000.txt` + test
  `top_3000_english_words_type_correctly` ghim toàn bộ top-3000. Cân bằng
  R2-56: 12 âm tiết Việt **không có cách gõ Telex nào khác** thắng tiếng Việt
  và vào `vn_common.txt` (`hair`→hải, `lee`→lê, `six`→sĩ, `own`→ơn, `las`→lá,
  `chair`→chải, `tax`→tã, `tree`→trê, `ups`→úp, `var`→vả, `wa`→ưa, `mar`→mả);
  các fold có cách gõ khác (`cá`="cas", `sét`="sest", `đền`="ddenf"…) nhường EN.
- Test hồi quy auto-restore EN (`core/tests/en_restore.rs`): từ EN phải tự
  correct tại dấu cách (user không phải xoá), cặp mơ hồ ghim giữ tiếng Việt.
- **Tab hoàn tất từ tiếng Việt** (`suggest.enabled` — 0.2.33, **tắt mặc định**):
  gõ dở một âm tiết + Tab → âm tiết tần suất cao nhất khớp tiền tố
  (`ngư` + Tab → `ngưng`, giữ hoa đầu từ), từ 3000 âm tiết tần suất cao
  (`data/vn_freq3000.txt` — nguồn hermitdave/FrequencyWords/OpenSubtitles qua
  OPUS, lọc bằng `is_valid_word`). Tab không khớp → đi qua app như cũ; bật qua
  `"suggest": { "enabled": true }` trong `config.v1.json` (schema P0-3 đồng bộ).

### Fixed
- Ghim hành vi gõ tiếp sau Backspace dưới mặc định dấu cũ: `hòa` + BS → `hò`
  (dấu còn trên `o`), gõ lại `a` → `hòa`; gõ tiếp `f` khi từ đã có huyền là phím
  undo (`ass` → `as`, chuẩn Telex/UniKey) — 2 hàng test mới khoá hành vi này.
- `test_settings_model.c`: 2 assertion cập nhật theo mặc định dấu cũ (thiếu khoá
  → `new_diacritic == 0`; reset mặc định → dấu cũ).

**English** — 0.2.33 changes the fresh-install default to Vietnamese typing
enabled with old-style tone placement (`hòa`, `thủy`, `khỏe`) on all three
platforms (existing users keep their choice), adds a 16-combo charset ×
input-method regression matrix with hard-coded TCVN3/VNI anchors, expands
the English auto-restore dictionary to the 3000 most frequent words (with
the rule that Vietnamese syllables having no alternative Telex spelling
always win), and adds optional Vietnamese Tab completion (`suggest.enabled`).

## [0.2.32] — 2026-10-12

> **Bản release candidate** — ký GPG + Sigstore như v0.2.31; từ bản này commit và tag phát
> hành còn mang chữ ký git (huy hiệu "Verified" — gate mới trong `release.yml`, R6). **Chưa**
> Authenticode (Windows — chờ SignPath Foundation) và **chưa** Developer ID + notarization
> (macOS) — vẫn là candidate, không phải production.

Rà soát toàn hệ thống sau 0.2.31: từng dòng thay đổi của vòng 16 + roadmap audit, kiểm bằng
chạy thật (chi tiết + bằng chứng: `docs/release/code-review-2026-10-09-round2.md` §3.8,
`docs/release/AUDIT-3PLATFORM-2026-10-07.md` mục F).

### Changed
- **Popup "Thông tin" giống nhau trên cả 3 nền tảng** (yêu cầu chủ repo): "Bộ gõ tiếng Việt
  thông minh, đa nền tảng (Windows, macOS, Linux)" + phiên bản + phát triển/giấy phép/URL —
  thay câu riêng "cho Windows/macOS/Linux" của từng bản; bản Linux trước đây không hiện số
  phiên bản (R2-113, test `about_text_matches_every_platform`).
- **macOS: bảng điều khiển về đúng đặc tả chung** — bảng mã và kiểu gõ cùng nhãn, cùng thứ tự
  với Windows/Linux ("Telex đơn giản" thay "4. Microsoft", bỏ số thứ tự), phím chuyển cố định
  "Ctrl + Shift (hoặc Ctrl + Shift + Space)". Bỏ hai lựa chọn chỉ có trên macOS (Alt + Z, Caps
  Lock): hai nền tảng kia không có, và trên macOS Caps Lock còn bật luôn chữ hoa của hệ thống.
  Cấu hình cũ đã chọn Alt + Z/Caps Lock tự dùng lại Ctrl + Shift (R2-106).

### Fixed

**Linux**
- **Đổi V/E rồi chuyển cửa sổ bị lật lại** (hồi quy từ 0.2.30): bấm Ctrl+Shift (hoặc menu)
  sang E, bấm sang cửa sổ khác là quay về V — IBus và Fcitx5 (R2-107; e2e mới "EN/VN giữ sau
  focus-out/in", đỏ trên bản cũ).
- Fcitx5 nhiều nhóm bộ gõ: TextVN được thêm vào nhóm ĐANG DÙNG (theo `GroupOrder`), không còn
  vào nhóm đầu file (R2-108).

**Windows**
- Tray không còn tự thoát khi Windows hỏi đăng xuất/khởi động lại mà lần đó bị huỷ (ví dụ
  Notepad hỏi lưu và bạn bấm Cancel) — chỉ thoát khi phiên kết thúc thật (R2-102).
- Nâng cấp: chỉ dừng tray chạy từ chính thư mục cài (bản portable/Store khác đang dùng được
  giữ nguyên), và dừng chắc chắn trước khi thay file (R2-105).
- Nâng cấp từ bản ≤ 0.2.30 mà chưa mở lại app: app đang mở (còn nạp bộ gõ cũ) vẫn nhận bật/tắt
  V/E từ khay và ngược lại (R2-109). Kèm theo các lỗi tên pipe của bản 0.2.31: `doctor` báo
  IPC "chưa kết nối" dù tray đang chạy, chế độ tương thích (hook) không liên lạc được với tray,
  `--stop` không gửi được lệnh thoát qua pipe.
- `TextVN.exe --status` khi tray không chạy báo `OFFLINE` thay vì panic (R2-101).
- Gỡ bản cài máy dọn TextVN khỏi danh sách bàn phím của mọi tài khoản đang đăng nhập, kể cả
  tài khoản gốc khi quản trị viên gỡ hộ — bản 0.2.30–0.2.31 có bước này nhưng chưa từng xoá
  được (đường dẫn registry sai) (R2-104).
- TSF: lỗi hiếm khi kết thúc composition không còn gây lặp chữ (R2-103).
- Kết nối tới tray chỉ nhận đúng `TextVN.exe` cùng phiên đăng nhập (R2-110); bộ cài build với
  tên sản phẩm khác (`/DMyAppName`) không còn bị từ chối đăng ký TSF (R2-111).
- Bản cài per-user gỡ không cần quyền quản trị vẫn hẹn xoá được file đang bị khoá (RunOnce;
  lệnh quá 260 ký tự được báo là không hẹn được thay vì báo thành công — R2-115).
- Tray chịu được lock bị "poison" ở mọi chỗ (menu khay, kết nối IPC) — gate tự động chặn
  `.lock()/.read()/.write().unwrap()` trong mã chạy thật (R2-114).

**macOS**
- Ứng dụng yêu cầu chốt chữ (commitComposition) nhận đúng chữ đang gõ; tuỳ chọn "Gõ không gạch
  chân" áp vào cả chiến lược của engine; app đang chạy nhận đúng tiến trình đang focus khi có
  nhiều tiến trình cùng bundle id.

### Known issues
- **Phần mềm diệt virus có thể cách ly nhầm** khi binary chưa có chữ ký Authenticode (chờ
  SignPath): 2026-10-11 Kaspersky cách ly `textvn-tsf.dll`/`TextVN.exe` của bản cài từ
  Microsoft Store khoảng 2 phút sau lần chạy đầu (gói cài vẫn thành công) — bằng chứng, hash
  để báo false positive và việc cần làm: `docs/specs/antivirus-false-positive.md` §10.

### Security / quy trình
- Preflight không còn cài/gỡ thật bộ cài trên máy dev (gate `store-validate`: bỏ qua khi
  artifact cũ hơn `Cargo.toml` hoặc chưa đặt `TEXTVN_STORE_VALIDATE=1`; CI vẫn chạy thật) (E18).
- Gate chữ ký git trong `release.yml`: commit và tag phát hành phải được GitHub xác nhận
  "Verified" (R6); tag phát hành ký bằng `git tag -s`.
- Quét rò rỉ private key/token (`check_no_secrets.py`, repo-hygiene + preflight), quét cả file
  mới chưa track (R2-112); `.gitignore` chặn các loại file khoá.

## [0.2.31] — 2026-10-11

> **Bản release candidate** — đã ký GPG (`.asc`, FPR
> `3921595ABC961199F15303B6C45B84D0C7F4A822`) + Sigstore keyless
> (`.cosign.sig`/`.cosign.cert`), `SHA256SUMS.txt` được clearsign — y như v0.2.29.
> **Chưa** Authenticode (Windows — chờ SignPath Foundation duyệt) và **chưa**
> Developer ID + notarization (macOS) — vẫn là candidate, không phải production.

### Fixed — hoàn tất roadmap audit 3 nền tảng (A6–A16, B6–B14, C1–C9)

Chi tiết từng finding + trạng thái: `docs/release/AUDIT-3PLATFORM-2026-10-07.md`.

**Windows**
- **A10**: `Activate`/`Deactivate`/`ActivateEx`/`OnSetFocus` bọc `catch_unwind` —
  panic thoát qua ABI COM không còn `abort` tiến trình host (Notepad/Chrome).
  Nhánh lấy `inner` thất bại nay trả `E_FAIL` thay vì `Ok(())` im lặng (trước đây
  `Advise` xong nhưng không lưu cookie → `Deactivate` không `Unadvise`, leak tới
  hết thread).

**macOS**
- **B7**: gather AX dedupe theo pid (`beginGather`/`endGather`) + `AXUIElementSetMessagingTimeout(0.3)`
  — app không phản hồi không còn treo gather, kết quả cũ không đè context mới.
- **B8**: `IpcServer` cap `app_id` ≤ 256 ký tự và tổng entry ≤ 256 — flood cùng-uid
  không còn phình RAM / làm Snapshot vượt 64KB (client thật không nhận được).
- **B9**: `clientPid` ưu tiên danh tính client từ `sender` (bundle id) thay vì
  `NSWorkspace.frontmostApplication` — panel phụ/XPC không còn làm preset sai.
- **B11**: xoá state chết `heldMods` + `noteFlagsChanged()`; `crashCount` lấy `max`
  thay vì `+= 1` (bỏ qua `count`).
- **B12**: `ipcClient(_:didReceive:)` ràng buộc `dispatchPrecondition(.onQueue(.main))`;
  `IpcClient.deinit` đồng bộ `tearDown()` đúng queue (không data race nguyên tắc).
- **B13**: `Diagnostics` log xoay ở 1 MiB (opt-in `TEXTVN_LOG=1`) — log debug không
  phình vô hạn.

**Linux**
- **C1**: xoá `lc_ipc_recv_frame` khỏi `linux_common.h` + `ipc_client.c` (API chết,
  mồi cho đúng lỗi CR-25 mất byte khi EAGAIN giữa frame).
- **C3**: thống nhất điểm poll IPC giữa IBus và Fcitx5 (focus/activate + word-start,
  không trên đường phím).
- **C7**: cửa sổ GTK settings ngắt handler `destroy` của cửa sổ con + huỷ cửa sổ con
  TRƯỚC khi free `TvWin` — hết UAF theo thứ tự destroy.
- **C8**: `tv_activate_fcitx5` đọc `DefaultIM` của nhóm đang dùng thay vì cứng
  `[Groups/0]`; deactivate đánh lại chỉ số cho MỌI nhóm (profile nhiều group nay
  thấy được bộ gõ).
- **C9**: `--config` không có `/` không còn ghi `state.json` lệch chỗ; `XDG_CONFIG_HOME`
  phải là đường dẫn tuyệt đối (`[0]=='/'`); `log.c` từ chối chuỗi bị cắt âm thầm +
  dùng `localtime_r` (an toàn thread) + `XDG_STATE_HOME` phải tuyệt đối; fcitx5
  destructor gọi `unregisterAction` (addon reload không còn con trỏ treo);
  `stage-linux.sh` chặn thư mục nguy hiểm (hệ thống, tổ tiên repo, HOME/ROOT/CWD)
  trước `rm -rf`; `field_detect.c` ghi rõ là stub chưa nối AT-SPI.

### Fixed — gate `clippy-linux` đỏ trên `main` (phát hiện khi chạy preflight)

`tray/src/main.rs`: `use std::io::{Read, Write}` và `PIPE_NAME` chỉ dùng trong hàm
`#[cfg(windows)]` nhưng import không gate → `cargo clippy --target x86_64-unknown-linux-gnu -D warnings`
đỏ (E11). Nay gate bằng `#[cfg(windows)]`.

## [0.2.30] — 2026-10-07

### Fixed (vòng 16 — audit 3 nền tảng: 19 fix, mỗi lỗi rà ≥2 lần theo view audit)

**Windows** — chi tiết từng lỗi: `docs/release/AUDIT-3PLATFORM-2026-10-07.md`
- **A1**: ILOT gỡ bàn phím Windows tự gắn thiếu tiền tố `0x` (R2-100 nửa vời) — chuẩn hoá spec.
- **A2/A8**: mirror WOW64 CTF cho HKCU là đường registry chết (HKCU\Software
  shared trừ Classes) — chỉ mirror CTF ở Machine; dọn mirror per-user cũ.
- **A3**: cài/nâng cấp khi tray đang chạy — installer dừng tray trước (`PrepareToInstall`),
  tray xử lý WM_QUERYENDSESSION/WM_ENDSESSION graceful cho RestartManager.
- **A4**: `--stop` dùng OpenMutexW(SYNCHRONIZE) — CreateMutexW(bInitialOwner=TRUE)
  khiến process dò trở thành chủ mutex → instance mới tưởng trùng và thoát.
- **A5**: ShellExecuteExW (UAC) trên thread không COM → CoInitializeEx đầu hàm.
- **A11**: RwLock poison trong svc không còn panic dây chuyền (into_inner).
- **A15**: marker notice ghi TRƯỚC hộp thoại (kill giữa chừng không hỏi lại vô hạn).

**macOS**
- **B1 (CAO)**: `preeditReplace`/`commitEarly` xoá chữ THẬT trước khi thu hồi
  marked — NSTextView xoá nguyên marked range thay vì ký tự thật → mất chữ.
  Đã retract-before-delete, nhất quán với backspaceType.
- **B2 (CAO)**: IPC client 3 đường offline vĩnh viễn (path dài / violation /
  không reconnect lúc focus) — giờ luôn retry + connect lại lúc focus.
- **B3**: `switch_key` (Alt+Z / Caps Lock) UI chết → IMK thực thi thật +
  Ctrl+Shift tap chỉ toggle khi đúng mode. 3 unit test mới.
- **B4**: crash-report pipeline đứt (không ai gửi) → engine lỗi gửi `.crashReport`.
- **B5**: "Mặc định" giữ macros/english_words/emoji/autostart (hết xoá âm thầm).
- **B6**: double-click app luôn báo awake (hết im lặng khi autostart bật).
- **B10/B13**: underline `.single` thật; xoá macro case-insensitive; cap từ EN 64.

**Linux**
- **C2**: dispatch IPC khoá bằng `"type":"X"` đầu frame (state key trùng tên
  message không còn hijack parsing).
- **C4**: override per-app hết hiệu lực → trở về base state (state.json/broadcast).
- **C5**: guard tràn buffer compose (text[256] OOB intra-object).
- **C6**: UTF-8 decoder chặn overlong/surrogate/>U+10FFFF.

## [0.2.29] — 2026-10-10

### Fixed — gỡ cài đặt sạch (R2-99, R2-100)
- Gỡ cài đặt (bộ cài, portable, Store) **thoát TextVN trước** mọi bước và chờ nó tắt
  hẳn — trước đây gỡ lúc TextVN đang chạy có thể sót `TextVN.exe` vì file còn bị khoá.
  Menu khay "Gỡ cài đặt" của bản cài mở trình gỡ (có hộp UAC với bản cho mọi người
  dùng) rồi tự thoát.
- Gỡ cài đặt **dọn cả danh sách ngôn ngữ/bàn phím**: ngoài bàn phím TextVN, ngôn ngữ
  mà chính TextVN đã thêm (ví dụ máy chỉ có tiếng Anh, cài TextVN thêm "Tiếng Việt")
  được bỏ khi người dùng không gắn bàn phím nào khác cho nó — không còn sót "Tiếng
  Việt" với bàn phím Windows tự gắn. Ngôn ngữ bạn có sẵn thì giữ nguyên. Ctrl + Shift
  được trả cho Windows như trước.

### Changed
- Gói `.msix` kèm bản phát hành mang identity thật của Microsoft Store
  (`23651Linhi.TextVN`) — nộp thẳng được, có chữ ký GPG/Sigstore như mọi file.

## [0.2.28] — 2026-10-10

### Added
- Workflow phát hành có ô nhập `msix_identity_name`: build lại riêng gói MSIX nộp Store
  (identity từ Partner Center) trên tag đã phát hành; chạy tay chỉ build, không đụng
  release đã công khai (`docs/release/msix-submission.md` §3). Gói `.msix` kèm release
  0.2.28 mang identity tạm — chỉ để thử, không nộp.

### Fixed — kiểm trên Windows 11 thật và rà soát đối kháng (R2-88…R2-98, chi tiết: `docs/release/code-review-2026-10-09-round2.md` §3.7)
- Gỡ bản portable/Store từng nâng cấp từ 0.2.27 nay trả lại Ctrl + Shift cho Windows
  (trước đây bản ghi Ctrl + Shift bị mất khi nâng cấp).
- Gỡ một bản TextVN (portable, bộ cài) không còn tắt hay gỡ đăng ký bộ gõ của bản
  TextVN khác đang dùng (bản cài ở thư mục khác, bản Store).
- Bỏ chọn "Khởi động cùng Windows" ở bản cài cho mọi người dùng nay có hiệu lực (trước
  đây trông như đã lưu nhưng lần đăng nhập sau TextVN vẫn tự chạy).
- Gỡ gói Store rồi cài lại trước lần đăng nhập kế: tự khởi động bật như lần cài đầu.
- File bị khoá sau khi gỡ được dọn ở lần đăng nhập kế cả khi đường dẫn dài.
- `deed`, `deeds`, `dosed` không còn thành `đê`, `đế`, `đoé` (gõ `đê` vẫn là `ddee`).
- Phát triển: `build-release.ps1` không còn đăng ký bộ gõ trỏ vào thư mục build hay đổi
  Ctrl + Shift của máy dev, và không dừng khi DLL x86 trong thư mục build đang bị app
  32-bit nạp; test gõ thử chạy được trên Notepad Windows 11; test IPC không còn nối vào
  TextVN đang chạy trên máy.

### Fixed — rà soát vòng 2: Microsoft Store, ký số, QA (chi tiết: `docs/release/code-review-2026-10-09-round2.md`)
- **Microsoft Store (MSIX) chạy thật**: bản cài từ Store chép chương trình ra ngoài gói
  (`%USERPROFILE%\.textvn`, `%LOCALAPPDATA%\Programs\TextVN-Store`) bằng tiến trình thoát
  khỏi container rồi mới đăng ký bộ gõ — trước đây mọi ghi bị Windows ảo hoá nên app khác
  không thấy TextVN. Gỡ gói Store thì lần đăng nhập kế dọn sạch đăng ký, mục khởi động,
  Ctrl + Shift và file đã chép; Store cập nhật thì tự chép lại. Bản Store **hỏi** trước khi
  dành Ctrl + Shift và không bao giờ xin quyền Administrator. CI cài thử gói thật mỗi lần.
- Gói MSIX nộp được: Version `(A+1).B.C.0` (Partner Center từ chối số đầu 0), mô tả tiếng
  Việt không còn lỗi mã hoá, ảnh 71×71 đúng, identity theo Partner Center
  (`MSIX_IDENTITY_NAME`), `tools/win/verify-msix.py` kiểm gói trước khi nộp.
- **Bảo mật Windows**: không còn đăng ký bộ gõ cho cả máy từ thư mục người dùng ghi được
  (chỉ từ Program Files); gỡ bản portable chỉ xoá đúng file TextVN, không bao giờ xoá cả thư
  mục; mọi binary link CRT tĩnh (không cần VC++ redist); bộ cài `-machine.exe` được ký khi
  bật SignPath.
- Windows: nâng cấp **giữ nguyên** kiểu gõ, tuỳ chọn và thiết lập theo app (trước đây bị
  reset mỗi bản) và không làm mất kích hoạt bộ gõ; bản cài cho mọi người dùng tự đăng ký cho
  từng tài khoản; menu "Bật tiếng Việt cho {app}" hiển thị đúng; "Từ điển EN" dùng Tab/Esc
  được; "Kiem tra he thong" không đóng ngay; "Cài & bật TSF" không còn treo Bảng điều khiển.
- Windows: bỏ "Dành Ctrl + Shift" thì TextVN không còn kéo bạn về TextVN mỗi khi Windows
  đổi bàn phím; gỡ cài đặt (bộ cài, portable, Store) trả Ctrl + Shift về **đúng giá trị
  trước đó** (cả phím đổi ngôn ngữ), không đụng lựa chọn bạn tự đặt.
- Gõ tiếng Việt: auto-restore và Tab gợi ý không còn biến từ Việt thành tiếng Anh (`thí`,
  `vơ`, `hót`, `có`+Tab); giữ `đ` trong `50.000đ`, `ĐT`, `đc`, `đ/c`.
- **Backspace xoá ký tự cuối đang hiển thị** như UniKey (`tiếng`+⌫ → `tiến`; trước đây
  hoàn tác phím cuối: `ass`+⌫ ra `á`, xoá `được` cần 8 lần ⌫).
- Tự viết hoa chỉ ở đầu câu (`. ! ?` + khoảng trắng, hoặc Enter): `google.com`, `file.txt`,
  `3.5 kg` không còn thành `google.Com`…
- VNI/VIQR gõ dấu sau cả cụm nguyên âm ra đúng (`toi6` → tôi, `nguoi72` → người); Telex `w`
  sau cụm nguyên âm đúng âm (`muaw` → mưa, `voiws` → với); Telex `w` đứng riêng → `ư`
  (`nhw` → như) như UniKey; đổi qua lại giữa dấu mũ và móc (`toow` → tơ, `awa` → â); bấm phím
  dấu lần ba gỡ dấu và gõ chữ đó (`xooong` → xoong); `d` gõ sau trong từ thành `đ` đầu từ
  (`duocjwd` → được); VNI `0` chỉ gỡ dấu thanh; Telex đơn giản gõ được ă/ơ/ư bằng `aw/ow/uw`.
- Không còn thêm dấu mũ tự động cho `iet` (`Viet`, `quiet` giữ nguyên); từ tiếng Anh như
  `sort`, `keep`, `chart`, `using`, `music` không còn bị biến thành âm tiết không tồn tại.
- Lưu ý: `w` + chữ số (`w3c`) nay ra `ư3c` như UniKey — bấm Escape để trả lại.
- macOS: chọn cách chèn chữ theo app hoạt động trở lại (trước luôn rơi về một cách);
  "Gõ không gạch chân" có hiệu lực ngay; Ctrl + Shift bật lại được khi TextVN.app không chạy;
  gõ tắt cùng luật với Windows/Linux; gỡ bản cài cho mọi người dùng hỏi quyền quản trị; ký
  Developer ID + notarize tự chạy khi có chứng chỉ Apple.
- Linux: Fcitx5 luôn thấy addon gốc của nó; IBus chạy được khi đường dẫn cài có khoảng
  trắng; `sudo ./install.sh --system` không đụng thiết lập của người dùng; cửa sổ Cài đặt
  GTK không còn mất cửa sổ Từ điển EN.
- Phát hành: ký GPG + Sigstore **bắt buộc** trong CI (thiếu khoá thì dừng); SignPath dùng
  đúng REST API, một lần duyệt cho cả bộ binary; cask Homebrew đúng sha; build `--locked`;
  thiếu VERSIONINFO thì build fail; release notes nói đúng trạng thái ký số.

### Fixed — rà soát mã nguồn chuyên sâu 2026-10-07 (chi tiết: `docs/release/code-review-2026-10-07.md`)
- **Mất chữ (P0)**: giữ phím lặp sau từ có dấu (`đẹppppp…`, > 64 phím) từng làm phím
  kế tiếp xoá lẹm chữ phía trước — engine giờ đóng từ khi chạm giới hạn, FFI từ chối
  thay vì cắt kết quả (CR-01/02). macOS: self-heal con trỏ chạy trước engine (CR-28).
- Phím tắt Ctrl/Alt/Cmd giữa từ không còn để phím kế tiếp sửa nhầm từ ở chỗ khác
  (engine, hook Windows, macOS — CR-04/13/29); macOS: Enter/Tab sau từ có dấu gửi đúng
  phím thật (Messages/Slack gửi tin, Tab chuyển ô — CR-30), phím điều hướng không chèn
  ký tự điều khiển (CR-31).
- Ô mật khẩu (role secure) luôn đi thẳng kể cả khi app quên cờ secure (CR-06).
- Windows: pipe client không cho server giả mạo impersonate (CR-09); tray không còn
  chiếm lại Ctrl + Shift mỗi lần khởi động (CR-17); gói chẩn đoán không chứa gõ tắt/từ
  điển của người dùng (CR-21); bộ cài thay/gỡ được `textvn-tsf-x86.dll` khi app 32-bit
  đang mở và dọn đúng file `.old-*` (CR-37); DLL x86 được ký cùng các binary khác (CR-36).
- Linux: client IPC không còn SIGPIPE làm sập ibus-daemon/fcitx5, không mất frame
  nhận dở dang (CR-24/25).
- macOS: lưu cài đặt không còn xoá danh sách emoji và trường `when` của gõ tắt (CR-33).
- Hướng dẫn kiểm chữ ký phát hành (`docs/release/signing.md`) dùng đúng tên `.asc` và
  workflow `release.yml` (CR-39).

## [0.2.27] — 2026-10-07

### Added (macOS) — toggle "Gõ không gạch chân" được tôn trọng (BUG-05, vòng 15)
- **BUG-05 fix**: toggle "Gõ không gạch chân (Non-preedit)" trong Bảng điều
  khiển ghi `non_preedit` vào config nhưng IMK **từng bỏ qua hoàn toàn** —
  giờ IMK đọc khoá này: bật → strategy Preedit (gạch chân) hạ xuống
  BackspaceType (không gạch chân, tương thích rộng hơn); tắt → preedit có
  gạch chân như trước. Config cũ thiếu khoá → giữ gạch chân (không đổi
  hành vi âm thầm). 3 unit test mới.
- **Ký số release tự động** (cả 3 platform): GPG detached signature
  (FPR `3921595A…`) + Sigstore keyless provenance cho **mọi asset** +
  clearsign SHA256SUMS.txt — chi tiết và lệnh verify: `docs/release/signing.md`.
- **MSIX hoàn thiện**: payload thêm `textvn-tsf-x86.dll` (app 32-bit dùng
  được sau stage-out) + DisplayName = `TextVN` khớp Partner Center.
- Audit đối chiếu: BUG-03 macOS (IMK nhận `"*"`), M-01 (menu sync didSet),
  M-03 (Swift matrix 2 arch) — **đã được xử lý sẵn từ các vòng trước**
  (finding audit dựa trên snapshot cũ); SEC-04 docs đồng bộ path mới.

## [0.2.26] — 2026-10-05

### Added (Windows) — hỗ trợ app 32-bit (Zalo/Office x86) — vòng 14
- **Forensics thật trên máy chủ repo**: `Zalo.exe` là **32-bit** (PE machine
  `0x014C`); TextVN chỉ có DLL 64-bit → tiến trình 32-bit không thể nạp
  (ERROR_BAD_EXE_FORMAT 193) → "không gõ được tiếng Việt trên Zalo" (không
  phải lỗi strategy/appdb). `HKCR\WOW6432Node\CLSID\{TIP}` xác nhận **trống**.
- **Build thêm TIP DLL x86** (`i686-pc-windows-msvc`, CRT static — không cần
  VC redist x86) + **mirror đăng ký sang view 32-bit** (WOW6432Node: COM CLSID
  → x86 DLL, CTF TIP/profile/category) cho cả 2 scope; unregister xoá mirror.
- Bộ cài/portable đóng gói `textvn-tsf-x86.dll`; gate WOW6432Node trong
  test-installer + ghost gate mở rộng.
- Zalo appdb giữ Preedit (thiết kế B2 gốc — DLL load được thì Electron xử lý
  preedit tốt như Discord/Slack; nguyên nhân thật là kiến trúc, đã sửa).

### Fixed (Windows) — gỡ cài sạch 100% key layout (vòng 13, bắt thật + dọn live trên máy chủ repo)
- Dump registry trước/sau khi gỡ 0.2.19 cho thấy **27 path còn chứa GUID TIP**:
  `CTF\Assemblies\<lang>\{34745C63…}[Default]` (bộ gõ mặc định của 2 ngôn ngữ
  — Windows tự ghi khi ActivateLanguageProfile), `InputMethodOverride`
  (default IME override), và cả cây HKLM machine registration từ đợt self-heal
  portable. Chọn vào Win+Space là TIP chết.
- `unregister` giờ xoá **hết**: quét mọi ngôn ngữ trong Assemblies, xoá value
  `Default` nếu chứa CLSID TextVN; xoá `InputMethodOverride` nếu trỏ TextVN;
  unregister user-scope cố dọn nốt HKLM best-effort.
- Bộ cài: bỏ `Check: IsAdminInstallMode` — gỡ per-user cũng thử dọn nốt HKLM
  (không elevation thì fail êm, không ảnh hưởng exit code).
- Tray: đăng ký máy trỏ file không tồn tại = ghost → coi như chưa có để repair.
- Gate tự động: test-installer quét mọi key/value chứa GUID TIP sau uninstall
  (HKCU + HKLM) — lệch một key là CI đỏ.
- Verify live trên máy chủ repo: 27 path → elevated unregister → **0 path**.

## [0.2.25] — 2026-10-05

### Added (Windows) — MSIX đúng chuẩn: bootstrap stage-out (vòng 12 audit)
- **Vấn đề thật của MSIX trước đây**: tray chạy từ `C:\Program Files  WindowsApps\<Package>_<version>_…` tự đăng ký TSF **trỏ vào DLL trong
  package** — (a) thư mục package chứa version và bị XOÁ sau mỗi lần Store
  update → đăng ký COM/CTF treo lơ lửng → mất gõ cho tới khi mở lại app;
  (b) ghi registry của tiến trình đóng gói có thể bị registry virtualization
  điều hướng vào hive riêng → đăng ký vô hình với Notepad/explorer.
- **Giải pháp**: exe trong package giờ chỉ làm một việc — stage toàn bộ payload
  (tray, CLI, TSF DLL, resources, data) ra `%LOCALAPPDATA%\Programs\TextVN`
  rồi spawn bản staged và thoát. Bản staged chạy **non-packaged** → dùng lại
  nguyên vẹn cơ chế portable (tự đăng ký TSF, UAC phạm vi máy B7, Run key, IPC).
  File bị khoá khi re-stage xử lý bằng rename-then-copy (B8).
- **Payload MSIX thiếu `resources/` + `data/`** từ trước — đã bổ sung (tray
  staged cần icon + appdb mặc định).
- 4 unit test mới cho bootstrap (nhận diện package, stage đủ file/dir, ghi đè
  khi stage lại, không relaunch ngoài WindowsApps).
- Đo thật trên máy: tiến trình thường **đọc được** file trong WindowsApps
  (READ OK) — loại trừ nghi ngờ ACL; nguyên nhân còn lại là update-fragility
  + virtualization → bootstrap xử lý cả hai.

## [0.2.24] — 2026-10-05

### Fixed (Windows) — hoàn thiện nốt entry ARP + metadata (rà dòng lần 3) + chốt bản máy cho Store
- **`UninstallDisplayIcon`**: entry ARP trước đây **thiếu value `DisplayIcon`** —
  checker/icon của Programs and Features nhìn vào sẽ thấy entry không hoàn
  chỉnh. Đã verify end-to-end bằng probe: `DisplayIcon` giờ xuất hiện đúng
  đường dẫn exe.
- **`VersionInfoTextVersion`**: chuỗi `FileVersion` của setup.exe trước đây
  **TRỐNG** (chỉ có số nội bộ) — metadata file giờ đầy đủ `0.2.24` ở cả 2 biến
  thể (verify bằng Get-Item VersionInfo).
- **`SetupLogging=yes`**: log cài đặt ghi vào %TEMP% — nếu validation vẫn fail,
  log trên VM của Microsoft là bằng chứng duy nhất ta có thể yêu cầu qua ticket.
- **Chốt theo quyết định chủ repo**: bản **machine (admin/all-users, HKLM) là
  bản Store chính thức** — MS doc cho phép UAC ("UAC prompts are allowed");
  portable giữ nguyên; GitHub users vẫn nhận bản per-user.
- 0.2.24 = gói mới cho lần nộp kế tiếp (B13g).

## [0.2.23] — 2026-10-05

### Fixed (Windows) — Publisher display name THẬT là `LinhBH.CoM` (chủ tài khoản tìm đúng trang trong account và tự sửa)
- Kết thúc chuỗi đoán định danh: **Publisher display name = `LinhBH.CoM`** —
  chủ tài khoản tìm vào account settings (trang khác với nơi hay xem) và
  **chỉnh sửa trực tiếp** thành giá trị này.
- Giải thích trọn lịch sử: 0.2.16–0.2.20 đã ghi **đúng** `LinhBH.CoM` — các
  vòng đó thua vì lý do khác (0.2.16 exit 10; 0.2.18 admin/740; 0.2.19 bẫy
  B13g "no changes" — nộp lại URL cũ nên không revalidate). 0.2.21 (`Linh βùi`)
  và 0.2.22 (`Linh Bui`) là hai lần **đọc/suy đoán sai** từ nguồn không phải
  trang publisher info.
- ARP `Publisher` = `LinhBH.CoM`; `DisplayName` = `TextVN` (khớp ✓).
- **B18 bổ sung**: Publisher display name nằm ở trang riêng trong account —
  phải vào đúng trang đó và dump codepoint trước khi tin bất kỳ chuỗi nào.
  Repo VARIABLES cập nhật theo: `STORE_PUBLISHER_NAME=LinhBH.CoM`
  (CI đối chiếu giá trị thật mỗi commit).
- 0.2.23 = gói MỚI cho lần nộp kế tiếp (B13g); kèm machine variant là release
  asset chính thức (từ 0.2.22).

## [0.2.22] — 2026-10-04

### Fixed (Windows) — ROOT CAUSE thật của 6 vòng đỏ: publisher phải là "Linh Bui" (B18)
- Chủ tài khoản xác nhận **lần 2** (sau khi audit vòng 6 chỉ ra khả năng nhầm
  font): Publisher display name trên Partner Center là **`Linh Bui`** — ASCII
  thuần. Lần trước đọc nhầm font: chữ **B** (U+0042) trông giống beta **β**
  (U+03B2). Kết luận dựa trên bằng chứng: chuỗi publisher trong ARP **chưa bao
  giờ khớp** ở bất kỳ lần nộp nào — 0.2.16-0.2.20 để `LinhBH.CoM`, 0.2.21 để
  `Linh βùi` → đúng nguyên nhân *"could not identify the app name and the
  publisher name"*.
- ARP `Publisher` = `Linh Bui`; tên sản phẩm `TextVN` giữ nguyên (đã khớp ✓).
- **B18**: định danh phải xác minh bằng **codepoint dump** từ nguồn sự thật
  (`($s.ToCharArray() | % { 'U+{0:X4}' -f [int]$_ }) -join ' '`), không tin mắt.
  Harness + simulate dùng mặc định `Linh Bui` và in codepoint khi lệch.
- Đã đặt repo VARIABLES `STORE_APP_NAME=TextVN`, `STORE_PUBLISHER_NAME=Linh
  Bui` → CI đối chiếu ARP với giá trị thật mỗi commit (hết tự-so-mình).
- Machine variant (HKLM) giờ là **release asset chính thức**: `build-release
  -MachineInstaller`; release.yml build + validate `-MachineOnly` + kèm vào
  release; publish job kiểm đúng 2 setup exe.
- Lần nộp kế tiếp: **per-user 0.2.22 trước** (ít rủi ro nhất, kiểm đúng giả
  thuyết chuỗi); nếu vẫn đỏ → machine 0.2.22 (kiểm giả thuyết HKLM); nếu vẫn
  đỏ → MSIX (`CN=1A703CAB-…` đã có, chờ `Package/Identity/Name`).

## [0.2.21] — 2026-10-04

### Fixed (Windows) — Publisher khớp ĐÚNG Partner Center (STO-03, chủ tài khoản xác nhận)
- **ARP `Publisher` = `Linh βùi`** (beta U+03B2, ù U+00F9) — chính là **Publisher
  display name** trên Partner Center; trước đây để `LinhBH.CoM` (nhãn hiệu) nên
  validator báo *"could not identify the app name and the publisher name that
  your app has added in the add or remove programs"*. Tên sản phẩm đã reserve
  là `TextVN` → `DisplayName` giữ nguyên `TextVN` (khớp ✓).
- `.iss` giữ **UTF-8 BOM** để ISCC đọc đúng ký tự đặc biệt ở mọi phiên bản Inno
  (BOM = dấu hiệu UTF-8 theo spec, không phụ thuộc auto-detect).
- **Verify end-to-end (probe riêng, AppId tạm)**: cài im lặng → đọc ARP →
  `Publisher` đúng codepoint `U+004C U+0069 U+006E U+0068 U+0020 U+03B2 U+00F9
  U+0069`, khớp tuyệt đối → gỡ cài sạch. Harness & simulate giờ dựng chuỗi kỳ
  vọng bằng `[char]` (file .ps1 ASCII-only) và **in codepoint khi lệch**.
- Kèm từ vòng 4 audit (đã có trong 0.2.20/nhánh): `/DMachineInstall=1` +
  `-MachineOnly` (bằng chứng CI run 37198638991: ARP ở HKLM PASS 5/5 trên runner
  elevated), cảnh báo MSIX identity placeholder, `B15` (VirusTotal không chặn
  release), `B16` (uninstaller Inno bất đồng bộ → harness phải POLL).

## [0.2.20] — 2026-10-04

### Fixed (Windows) — CI đỏ .iss + vòng 3 validation (B13/B14)
- **CI "Windows package" đỏ**: `textvn-setup.iss` dùng `RegDeleteKeyValue` —
  Pascal Script của Inno Setup KHÔNG có hàm này (chỉ `RegDeleteValue`). Sửa +
  ghi mã **B14**: mọi thay đổi `[Code]` phải biên dịch thật bằng ISCC trước khi
  push. Uninstaller giờ chỉ trả `Layout Hotkey` về mặc định khi giá trị = 3
  (đúng override của TextVN, giống `uninstall.ps1`; tôn trọng lựa chọn khác
  của người dùng).
- **Harness `validate-store-package.ps1` "đúng như Microsoft expected"**: thêm
  bước 0 đọc manifest exe — BẮT BUỘC `asInvoker` (exe `requireAdministrator`
  làm `CreateProcess` non-elevated của validator fail 740 TRƯỚC khi cài → cả
  3 mục đỏ; đo thật exit=2 — B13e); ARP so **từng chữ** DisplayName `TextVN` +
  Publisher `LinhBH.CoM` + có DisplayVersion (hết `-like` — B13f).
- **Bẫy "We did not find any changes in the Package or the Silent install
  parameters"** ghi thành B13g: nộp lại cùng gói/cùng URL → Partner Center
  không chạy lại validation. Mỗi lần nộp phải là phiên bản mới (0.2.20).
- `build-release.ps1 -BuildInstaller` tìm được ISCC cài per-user bằng winget
  (`%LOCALAPPDATA%\Programs\Inno Setup 6\`) — trước đó fail dù mọi bước khác
  xanh.

### Security / hardening (audit 2026-10-04, phần còn lại)
- **SEC-02 vòng 2**: tray chỉ tự nâng quyền (UAC) từ thư mục cài chuẩn —
  Program Files (gồm MSIX `WindowsApps`) và `%LOCALAPPDATA%\Programs` (bộ cài
  per-user / gói .exe Store). Thư mục tự chọn (Downloads, ổ D: portable) bị TỪ
  CHỐI (CWE-426/732: DLL planting cạnh binary sẽ chạy bằng admin) + thông báo
  MỘT lần cách cài đặt (thread riêng, không chặn `--stop`). Bước nâng quyền chỉ
  tới sau khi activation per-user thất bại thật (B7) — bản cài chuẩn tự chữa
  được như cũ.
- **SEC-05**: named pipe IPC tạo với DACL chỉ user hiện tại (+SYSTEM/
  Administrators) — hết cảnh user khác cùng máy (phiên RDP) đọc ConfigReload/
  StateUpdate và gửi Shutdown giả.
- **SEC-04**: IPC client Linux ưu tiên `$XDG_RUNTIME_DIR/TextVN/ipc.sock`,
  fallback `/tmp/textvn-<uid>/`; guard `lstat` (đối tượng phải là socket của
  chính user) trước `connect`.
- **PERF-01**: quét từ điển restore-EN lọc theo độ dài trước `to_lowercase()`
  (hết alloc thừa mỗi ranh giới từ).
- **BUG-03 / 3.2.B / 3.2.C**: ghi rõ lý do hoãn trong phụ lục roadmap — parser
  Linux `app_id:"*"` là code chết (poll chưa được gọi; sửa cùng Linux tray
  P3-0); `non_preedit` macOS và non-preedit Linux là thay đổi hành vi cần test
  thật với Chromium/Office (F3-021).

## [0.2.19] — 2026-10-04

### Fixed (Windows) — loại bỏ TỪNG blocker còn lại của silent install (B13)
- **ShowLanguageDialog=no**: `/VERYSILENT` vẫn có thể bị hộp thoại chọn ngôn
  ngữ chặn khi locale máy cài không khớp [Languages] (auto → hiện dialog khi
  không tìm được ngôn ngữ hệ thống) — cài không hoàn tất = cả 3 check Store đỏ
  cùng lúc. Giờ không còn dialog nào trong mọi trường hợp.
- **Silent = pure file copy**: bỏ gọi đăng ký TSF trong tiến trình installer
  khi silent (API TSF có thể treo trong phiên không-interactive của validator)
  — app tự đăng ký ở lần chạy đầu (tray tự đề nghị UAC một lần khi cần, 0.2.16).
- Hai entry `[Run]` `config init`/`--free-ctrl-shift` thêm `skipifsilent`.
- Per-user giữ nguyên (0.2.18 đã thử admin/HKLM — CreateProcess của validator
  fail 740 với exe requireAdministrator trước khi cài gì cả, exit=2 bắt thật).
- Harness `validate-store-package.ps1`: ghi chú ARP hai hive; đã verify 4/4
  PASS với installer mới (silent 2s exit 0 → ARP 'TextVN'|'LinhBH.CoM'|'0.2.18'
  → 1 entry → uninstall sạch).

## [0.2.18] — 2026-10-03

### Fixed (Windows) — Package validation vòng 2 (B13)
- Sau khi 0.2.17 chuyển per-user, 3 mục validation vẫn đỏ: **validator của Store
  CHỈ đọc entry Add/Remove trong `HKLM`** — bản cài HKCU bị bỏ qua
  ([Microsoft Q&A 1922205](https://learn.microsoft.com/en-us/answers/questions/1922205/)).
  Khôi phục `PrivilegesRequired=admin` (machine install → HKLM; sandbox validator
  chạy installer đã elevated nên không sinh UAC tại đó) — **vẫn giữ silent luôn
  exit 0** (fix thật của vòng 1; exit 10 của 0.2.16 là cớ fail còn lại).
- **ARP DisplayName = đúng tên sản phẩm**: `AppVerName={#MyAppName}` — entry giờ
  là "TextVN (0.2.18)"→ hiển thị khớp "TextVN" trên Partner Center từng chữ
  (trước là "TextVN version 0.2.17", dễ bị đối chiếu trượt).
- Harness `validate-store-package.ps1` **ép entry mới phải nằm ở HKLM** (đúng
  cơ chế validator; cờ `-PerUser` chỉ để test riêng nhánh per-user).
- Chuỗi ô switches của Store giữ nguyên `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`;
  KHÔNG thêm `/CURRENTUSER` (đó là đường tắt gây chính lỗi HKCU này).

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

[Unreleased]: https://github.com/hunglinhpt/TextVN/compare/v0.2.52...HEAD
[0.2.52]: https://github.com/hunglinhpt/TextVN/compare/v0.2.51...v0.2.52
[0.2.51]: https://github.com/hunglinhpt/TextVN/compare/v0.2.50...v0.2.51
[0.2.50]: https://github.com/hunglinhpt/TextVN/compare/v0.2.49...v0.2.50
[0.2.49]: https://github.com/hunglinhpt/TextVN/compare/v0.2.48...v0.2.49
[0.2.48]: https://github.com/hunglinhpt/TextVN/compare/v0.2.47...v0.2.48
[0.2.47]: https://github.com/hunglinhpt/TextVN/compare/v0.2.46...v0.2.47
[0.2.46]: https://github.com/hunglinhpt/TextVN/compare/v0.2.45...v0.2.46
[0.2.45]: https://github.com/hunglinhpt/TextVN/compare/v0.2.44...v0.2.45
[0.2.44]: https://github.com/hunglinhpt/TextVN/compare/v0.2.43...v0.2.44
[0.2.43]: https://github.com/hunglinhpt/TextVN/compare/v0.2.42...v0.2.43
[0.2.42]: https://github.com/hunglinhpt/TextVN/compare/v0.2.41...v0.2.42
[0.2.41]: https://github.com/hunglinhpt/TextVN/compare/v0.2.40...v0.2.41
[0.2.40]: https://github.com/hunglinhpt/TextVN/compare/v0.2.39...v0.2.40
[0.2.39]: https://github.com/hunglinhpt/TextVN/compare/v0.2.38...v0.2.39
[0.2.38]: https://github.com/hunglinhpt/TextVN/compare/v0.2.37...v0.2.38
[0.2.37]: https://github.com/hunglinhpt/TextVN/compare/v0.2.36...v0.2.37
[0.2.36]: https://github.com/hunglinhpt/TextVN/compare/v0.2.35...v0.2.36
[0.2.35]: https://github.com/hunglinhpt/TextVN/compare/v0.2.34...v0.2.35
[0.2.34]: https://github.com/hunglinhpt/TextVN/compare/v0.2.33...v0.2.34
[0.2.33]: https://github.com/hunglinhpt/TextVN/compare/v0.2.32...v0.2.33
[0.2.32]: https://github.com/hunglinhpt/TextVN/compare/v0.2.31...v0.2.32
[0.2.31]: https://github.com/hunglinhpt/TextVN/compare/v0.2.30...v0.2.31
[0.2.30]: https://github.com/hunglinhpt/TextVN/compare/v0.2.29...v0.2.30
[0.2.29]: https://github.com/hunglinhpt/TextVN/compare/v0.2.28...v0.2.29
[0.2.28]: https://github.com/hunglinhpt/TextVN/compare/v0.2.27...v0.2.28
[0.2.27]: https://github.com/hunglinhpt/TextVN/compare/v0.2.26...v0.2.27
[0.2.26]: https://github.com/hunglinhpt/TextVN/compare/v0.2.25...v0.2.26
[0.2.25]: https://github.com/hunglinhpt/TextVN/compare/v0.2.24...v0.2.25
[0.2.24]: https://github.com/hunglinhpt/TextVN/compare/v0.2.23...v0.2.24
[0.2.23]: https://github.com/hunglinhpt/TextVN/compare/v0.2.22...v0.2.23
[0.2.22]: https://github.com/hunglinhpt/TextVN/compare/v0.2.21...v0.2.22
[0.2.21]: https://github.com/hunglinhpt/TextVN/compare/v0.2.20...v0.2.21
[0.2.20]: https://github.com/hunglinhpt/TextVN/compare/v0.2.19...v0.2.20
[0.2.19]: https://github.com/hunglinhpt/TextVN/compare/v0.2.18...v0.2.19
[0.2.18]: https://github.com/hunglinhpt/TextVN/compare/v0.2.17...v0.2.18
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
