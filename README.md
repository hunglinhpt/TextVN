# TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux

Tiếng Việt · English (mục [English](#english) cuối trang)

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![CI Windows](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-windows.yml?branch=main&label=CI%20Windows)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-windows.yml?query=branch%3Amain)
[![CI Linux](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-linux.yml?branch=main&label=CI%20Linux)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-linux.yml?query=branch%3Amain)
[![CI macOS](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-macos.yml?branch=main&label=CI%20macOS)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-macos.yml?query=branch%3Amain)
[![CI engine (3 OS)](https://img.shields.io/github/actions/workflow/status/hunglinhpt/TextVN/ci-shared.yml?branch=main&label=CI%20engine%20%283%20OS%29)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-shared.yml?query=branch%3Amain)

[![Tải TextVN trên Microsoft Store](https://get.microsoft.com/images/en-us%20dark.svg)](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)

TextVN là bộ gõ tiếng Việt mã nguồn mở. Một engine Rust dùng chung cho **Windows**
(Text Services Framework), **macOS** (Input Method Kit) và **Linux** (IBus, Fcitx5),
một bảng điều khiển với cùng tuỳ chọn và cùng file cấu hình trên cả ba nền tảng.

- **Gõ như UniKey/OpenKey**: Telex, VNI, VIQR, Telex đơn giản; đặt dấu theo chính tả —
  **mặc định dấu cũ** (`hòa`, `thủy`, `khỏe`; đổi sang dấu mới `hoà`/`thuỷ` trong bảng
  điều khiển), `uow` → ươ, `z` xoá dấu, gõ dấu cuối từ hay giữa từ đều được, gõ được khi
  bật Caps Lock.
- **Không mất chữ, không lặp chữ, không gạch chân**: cả từ nằm trong vùng soạn (composition/preedit)
  không đường kẻ (TF_LS_NONE trên Windows TSF; macOS gõ kiểu non-preedit sau cài; Fcitx5/IBus
  không kềm gạch — chỉ Chromium/Electron/Qt trên Linux tự vẽ gạch phía app) và chốt ở ranh
  giới từ — không gửi Backspace giả, nên không có lỗi kinh điển của bộ gõ kiểu hook
  trên Chrome, Electron, Excel, thanh địa chỉ.
- **Tránh bị phần mềm diệt virus nhận nhầm**: gói mặc định không hook bàn phím toàn cục,
  không tiêm mã vào process khác, không giả lập phím bằng `SendInput`
  ([lý do](docs/specs/antivirus-false-positive.md)).
- **Tiện ích**: bảng mã TCVN3/VNI Windows/Unicode tổ hợp, gõ tắt, Quick Telex, khôi phục từ
  tiếng Anh (từ điển 3000 từ tần suất cao), Tab hoàn tất từ — tiếng Anh luôn bật, tiếng Việt
  bật qua `suggest.enabled` — tự viết hoa đầu câu, Ctrl+Shift để chuyển V/E tức thì và cập
  nhật biểu tượng trạng thái.
- **Trợ lý ngữ pháp (thử nghiệm, bật mặc định ở chế độ Chỉ gợi ý)**: kiểm tra cả câu
  tiếng Việt **lẫn tiếng Anh (kể cả câu mix văn phòng)** ngay
  trên máy (không gửi đi đâu, không log nội dung) — bắt lặp từ, khoảng trắng/space quanh
  dấu câu, viết hoa đầu câu, **sai vị trí dấu** ("nghiã"→"nghĩa") và gợi ý sửa; chế độ *Tự
  sửa lỗi* tự sửa các lỗi chắc chắn rồi hoàn tác được bằng Ctrl+Z trong 5 giây. Panel
  tại caret hiển thị câu vừa gõ **gạch chân đỏ đúng từ sai** + tổng số lỗi (kiểu
  Grammarly), không toast khay khi gõ thường. Chuẩn chất lượng "không bao giờ bêu chữ
  đúng": **0 %** câu đúng bị báo nhầm trên tập kiểm 981 câu tiếng Việt + 295 câu tiếng
  Anh (0.2.38, chi tiết [P4-12 §11](docs/50-assist/P4-12-RECALL-REAL.md)).

## Tải và cài

Bản mới nhất: **[GitHub Release v0.2.60](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.60)**
(release candidate do CI build và kiểm thử). **Mọi file phát hành đều có chữ ký GPG
(`.asc`) và Sigstore (`.cosign.sig` + `.cosign.cert`)**, `SHA256SUMS.txt` được clearsign
— cách kiểm: [signing.md](docs/release/signing.md). Chưa có: chữ ký Authenticode cho
Windows (đang chờ SignPath Foundation duyệt — SmartScreen có thể cảnh báo, phần mềm diệt
virus có thể cách ly nhầm: đã gặp với Kaspersky, xem
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) và Apple
Developer ID/notarization cho macOS (Gatekeeper cảnh báo lần mở đầu). Bản 0.2.60 là **bản tối ưu hiệu năng sau audit**: 5 đường O(n²) trong trợ lý (phát hiện thiếu-space văn bản dày dấu câu, quét lặp, ngoặc kép, guard spelling) về tuyến tính — dán 640k ký tự 45s → 0.9s; phục hồi phát hiện ngày vô lý cuối văn bản. Bản 0.2.59 thêm **cảnh báo ngày tháng vô lý** ("32/13/2025", "31/02/2026" — calendar-aware) và **số tiền phân cách hỗn loạn** ("1.250.00đ") — đề xuất từ persona kế toán/HR/bán hàng. Bản 0.2.58 là **vòng audit persona 7 profile** (kế toán/nhân sự/bán hàng/CSKH/giáo viên/quản lý/người dùng thường): chat không dấu hết bị auto-fix phá chữ ("an com", "i di"), trợ động từ EN lạc sau từ Việt được báo, phẩy-sau-"but" tiếng Anh được báo đối xứng "nhưng". Bản 0.2.57 là **vòng audit toàn bề mặt 3 lượt** (tĩnh cõ gõ + hành trình phím 3 OS + đối kháng): VIQR hết toggle vô tận khi lặp phím dấu (freeze như Telex/VNI), macOS hết mất từ khi toggle VI/đổi cài đặt giữa từ, `DD` hoa gạch thành `Đ`. Bản 0.2.56 là **bản audit chế độ tiếng Anh**: trợ lý hết bỏ qua luật EN trên câu mix ("Anh should of done it sớm hơn") và văn bản Việt không dấu ("noi it do met"), chính tả EN tự phục hồi sau probe chậm, nguyên văn trong ngoặc kép không bị sửa, SVA trên văn bản dài O(n) thay O(n²). Bản 0.2.55 là **vòng audit 3 lượt từng dòng** (luật trợ lý + engine/FFI + 3 OS): sửa 3 luật tự sửa phá chữ đúng ("Tuy giỏi nhưng không nên kiêu ngạo", "more minutes, then try again", "their going rate"), fix trợ lý tự tắt trên câu mix ngắn ("y kiến hay"), đóng rò bộ nhớ IBus, và tăng tốc dự đoán ~35× (715µs → 20µs mỗi phím); kèm +40 thành ngữ/tục ngữ, +20 từ láy, +25 cặp collocation văn phòng, dự đoán role +142 mẫu. Bản 0.2.54 thêm **phát hiện lặp cụm 2–4 từ** (chỉ gợi ý: "vui lòng vui lòng kiểm tra", "đã hoàn thành đã hoàn thành"), **từ tiếng Anh thừa trong câu Việt** ("đã been hủy", "trước before 17h"), **+19 cặp collocation văn phòng** (cảm kết→cam kết, xắp xếp→sắp xếp, học sinh thiếu dấu), **+68 thành ngữ/tục ngữ + 24 từ láy** vào vùng bảo vệ, **dự đoán theo role văn phòng** (kế toán/nhân sự/kinh doanh/CSKH/giáo viên, +132 mẫu), và hardening 3 nền tảng: Windows hết mất chữ trên error-path, macOS hồi sinh chord Ctrl+Shift+F9, Linux hết đóng băng bàn phím khi tray treo. Bản 0.2.53 sửa **hòa hợp chủ-vị với dạng rút gọn tiếng Anh** ("He don't" → "doesn't"), thêm 7 cặp collocation tiếng Việt (sẳn sàng→sẵn sàng, chần chờ→chần chừ, cố găng→cố gắng, trực trặc→trúc trắc…) và mở **Tab hoàn tất từ tiếng Anh chưa fold** khi dự đoán bật. Bản 0.2.52 mở rộng **trợ lý ngữ pháp và dự đoán tiếng Anh toàn diện**: 3356 từ vựng tần suất (coder, tech, business, OS APIs Win32/POSIX/Cocoa, Gen Z slang), 3750 mẫu câu & cụm từ ngữ cảnh (Tab/Ctrl+Space), luật sửa lỗi closed-list (`their own`, `except for`, `led to`, `lose`, `principal`, `take effect`), và kiểm tra chính tả tiếng Anh OS Spell dual-language với RAII ComApartment. Bản 0.2.51 hoàn tất **ngữ pháp tiếng Anh** (trước đây chỉ có a/an): sửa `should of`→`should have`, `your welcome`→`you're welcome`, `its been`→`it's been`, `their going`→`they're going`, `is to big`→`is too big`, `better then`→`better than`, đại từ `i`→`I` — danh sách đóng chỉ chạy trên câu tiếng Anh. Bản 0.2.50 hoàn tất vòng chức năng: **Ctrl+Shift+F9 chuyển mã vùng bôi đen trên cả 3 nền tảng**, **kiểm tra chính tả từ điển của Windows** (D39), **bật được dự đoán văn bản từ bảng điều khiển** (trước đây không có nút), và một vòng audit 3 lượt sửa 2 HIGH + 5 MEDIUM + 10 LOW. Bản 0.2.49 làm **Ctrl+Shift+F9 chuyển đổi văn bản bôi đen thông minh**: sửa chữ lạ khi mở file sai bảng mã (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), chuyển bảng mã cũ TCVN3/VNI sang bảng mã đang dùng, gõ lại chữ gõ nhầm khi quên bật tiếng Việt (`tooi ddi hocj` → `tôi đi học`); trợ lý sửa được ở ô nhập WinForms/Delphi; nhắc mở lại ứng dụng sau khi cập nhật. Bản 0.2.48 chốt **phím tắt trợ lý chỉ còn Ctrl+Space** (Windows/Linux; macOS Option+Space) — bỏ Alt+Space vì đụng menu cửa sổ và PowerToys Run/Copilot/ChatGPT. Bản 0.2.47 thêm **chữ mờ gợi ý sẵn** sau con trỏ (Windows; macOS/Linux xem trước trong bảng ứng viên) và **một phím tắt vừa sửa vừa dự đoán**: Ctrl+Space (macOS Option+Space) sửa hết lỗi của câu (kể cả chính tả) rồi mới dự đoán, không có việc thì phím về ứng dụng; nút *Sửa tất cả* và Alt+1…3 nay hoạt động ở mọi ứng dụng kể cả Teams/Electron (VS Code) và Notepad++ (E2E thật 44/44). Bản 0.2.46 thêm **dự đoán văn bản song ngữ** (bật `suggest.enabled`: Tab/Ctrl+Space hoàn tất từ và đoán cụm tiếp theo — *Kính gửi* → *quý khách hàng*) và **trợ lý ngữ pháp sống theo câu kiểu Grammarly** trên cả 3 nền tảng: câu sai hiện gợi ý ngay, sửa xong ẩn ngay, không chiếm phím gõ; panel Windows luôn hiện được (trước đây không neo được ở Notepad Win11), chữ to dễ đọc, bỏ toast "phát hiện lỗi"; bộ cài tự quét và dọn bản cũ (kiểm chứng E2E 28/28 trên Notepad/Word/Excel thật). Bản 0.2.45 thêm **gói năng suất văn phòng P5**: gõ tắt theo kiểu chữ (HD→HỢP ĐỒNG), bảo vệ công thức Excel (`=sum(...)`), gõ tắt số tiền (`=15450000bc`+Tab → *Mười lăm triệu đồng chẵn*), sửa cụm từ hành chính (bổ xung→bổ sung), khoảng trắng trước % (`100 %`→`100%`), bảo vệ identifier coder (useState/KPI/EBITDA) và **chuyển mã vùng bôi đen Ctrl+Shift+F9** (TCVN3/VNI/Unicode tổ hợp → Unicode). Bản 0.2.44 là vòng **audit sâu 3 lượt toàn bộ trợ lý ngữ pháp** (khóa loạt FP cuối: tiếng Việt thiếu dấu, tên file, ngoặc cong, viết tắt ĐH., DPI panel, Fcitx5 fresh-install). Bản 0.2.43 làm phím **Enter trên popup vùng bôi đen áp "Sửa tất cả"** (trước đây bị nuốt mà không sửa — finding cuối cùng của vòng audit selection-check). Bản 0.2.42 thêm check ngữ pháp khi bôi đen câu (Shift+mũi tên → popup + Sửa tất cả). Bản 0.2.41 sửa trợ lý chết khi nâng cấp (3 root cause). Bản 0.2.40 là vòng audit 3 nền tảng (sửa lỗi article_an phá chữ Việt, đồng bộ default). Bản 0.2.39 làm panel trợ lý **bấm được bằng chuột** (nút Sửa tất cả, click từ gạch chân). Bản 0.2.38 **bật trợ lý ngữ pháp mặc định** (Chỉ gợi ý) với panel tại caret kiểu Grammarly + toast khay chỉ khi tự sửa. Bản 0.2.37 thêm
**Trợ lý ngữ pháp tiếng Anh** (a/an theo âm, lặp từ EN) + **câu mix Việt–Anh văn phòng**
được kiểm tra + hoàn tất từ bằng **Space** (tùy chọn) — đo trên corpus thật: 0/981 (VI) và
0/295 (EN) câu đúng bị báo nhầm, recall 100% cả 2 tập lỗi. Bản 0.2.36 nâng
chất lượng **Trợ lý ngữ pháp** lên chuẩn "không bao giờ bêu chữ đúng": **FPR 0 %** trên
981 câu sạch (bỏ 5 cặp typo vi phạm chính sách khiến autocorrect sửa sai "quyến rũ",
"thuở xưa"…; recall câu 83,7 %), câu VIẾT HOA có dấu được kiểm tra. Bản 0.2.35 ra mắt
**Trợ lý ngữ pháp** (kiểm tra + gợi ý + tự sửa câu, tắt mặc định). Bản 0.2.34 mở
**kênh phân phối Linux**: AUR (`paru -S textvn`) + PPA Debian/Ubuntu; bản 0.2.33 đổi
**mặc định sau cài sang dấu cũ** trên cả 3 nền tảng
(người dùng cũ giữ nguyên lựa chọn), thêm ma trận hồi quy 16 combo bảng mã × kiểu gõ,
mở rộng từ điển tự khôi phục tiếng Anh lên 3000 từ tần suất (cân theo luật ưu tiên âm
tiết Việt không có cách gõ khác) và thêm Tab hoàn tất từ tiếng Việt (tùy chọn
`suggest.enabled`) — chi tiết:
[CHANGELOG.md](CHANGELOG.md). Commit và tag phát hành có chữ ký git.

| Nền tảng | Cài đặt | Giải nén dùng ngay |
|---|---|---|
| Windows 10/11 x64 | `TextVN-setup-<ver>-windows-x64.exe` (cho tài khoản của bạn, không cần admin) · `TextVN-setup-<ver>-windows-x64-machine.exe` (cho mọi người dùng, Program Files, UAC một lần) | `TextVN-portable-<ver>-windows-x64-*.zip` → chạy `TextVN.exe` |
| Windows (Microsoft Store) | ✅ **[TextVN trên Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (miễn phí, Store ID `9NV2R7JNNFGK`) — bấm *Tải*, mở TextVN một lần để đăng ký bộ gõ cho tài khoản; Store tự cập nhật, gói do Microsoft ký. Dòng lệnh: `winget install --id 9NV2R7JNNFGK -s msstore`. File `TextVN-<ver>-windows-x64.msix` trên GitHub Release là gói nộp Store (chưa ký) — không cài trực tiếp được | — |
| macOS 13+ (Apple Silicon / Intel) | **Homebrew (khuyên dùng): `brew install --cask hunglinhpt/tap/textvn`** — tự đặt bộ gõ vào `~/Library/Input Methods` và app menu bar vào `/Applications`; cập nhật theo `brew upgrade`. Hoặc `TextVN-mac-v<ver>.pkg` → mở để cài, hoặc `installer -pkg … -target CurrentUserHomeDirectory` (không sudo) | `TextVN-macos-universal-v<ver>.zip` (hoặc `.tar.gz`) chứa `TextVN-IM.app` + `TextVN.app` — xem [Hướng dẫn sử dụng, mục macOS](docs/user-guide.md) |
| Linux (IBus / Fcitx5) | **AUR** (Arch/Manjaro…): `paru -S textvn` — build từ mã nguồn tag, QA makepkg thật trong CI · **PPA Ubuntu** (chuẩn bị — owner kích hoạt `ppa:hunglinhpt/textvn`, xem [linux-distribution.md](docs/release/linux-distribution.md)): `sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn` · Hoặc `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → `./install.sh` (per-user, không cần root; `sudo ./install.sh --system` cho mọi người dùng) — phân tích kênh store Linux: [linux-distribution.md](docs/release/linux-distribution.md) | cùng tarball → `./textvn-portable.sh` |

Chi tiết cài, gỡ, sử dụng và xử lý sự cố: **[Hướng dẫn sử dụng](docs/user-guide.md)**.

Lưu ý Windows: `TextVN-setup-<ver>-windows-x64.exe` cài **cho tài khoản hiện tại**
(`%LOCALAPPDATA%\Programs\TextVN`, không UAC); `TextVN-setup-<ver>-windows-x64-machine.exe`
cài **cho mọi người dùng** (Program Files, UAC một lần). Mỗi tài khoản Windows tự được
đăng ký bộ gõ ở lần đầu TextVN chạy; nâng cấp **giữ nguyên** kiểu gõ, tuỳ chọn và thiết
lập theo từng ứng dụng. Trên **Windows 11 24H2+** một số máy từ chối kích hoạt bộ gõ chỉ
đăng ký cho tài khoản (lỗi B7 trong `docs/specs/win-test-common-errors.md`): vì an toàn,
bản cài cho tài khoản, bản portable và bản Store **không bao giờ xin UAC** — TextVN báo
một lần và hướng dẫn dùng bộ cài `-machine.exe` (chỉ bản nằm trong Program Files mới
được đăng ký phạm vi máy).

## Điểm vượt trội so với các bộ gõ khác

Những gì TextVN làm mà UniKey / EVKey / OpenKey hiện **không có** (hoặc làm
khác — so sánh theo tính năng công khai tại thời điểm 2026-10):

| Tính năng | TextVN | Bộ gõ truyền thống |
|---|---|---|
| **Tự xác định EN/VI khi gõ chung đoạn văn** | Từ điển EN 3000 từ tần suất cao nhất + lưới bảo vệ âm tiết Việt thông dụng: gõ xen kẽ 2 ngôn ngữ không cần đổi mode, từ nào là tiếng Anh được trả lại nguyên vẹn khi gõ dấu cách | Telex áp lên mọi từ — `text` thành `tẽt`, `is` thành `í`; muốn gõ tiếng Anh phải bấm tắt mode từng từ hoặc tự gỡ dấu |
| **Ưu tiên tiếng Việt theo mode** (không phá tiếng Việt) | Cặp mơ hồ `cow`/`cơ`, `sex`/`sẽ`, `queen`/`quên` được giải theo mode đang bật — đúng chuẩn UniKey cho tiếng Việt, đồng thời trả tự do cho tiếng Anh | Phải chọn: hoặc phá tiếng Anh hoặc tắt transform |
| **Tab gợi ý hoàn tất từ** | Gõ dở `tes` → Tab thành `test` (EN, 3000 từ tần suất + từ điển bổ sung cá nhân, luôn bật); gõ dở `ngư` → Tab thành `ngưng` (tiếng Việt, 3000 âm tiết tần suất, bật qua `suggest.enabled`) | Không có |
| **Từ điển EN cá nhân** | Thêm/xoá từ bằng UI (Từ điển EN…); quyết định của người dùng thắng mọi phỏng đoán engine | Sửa file config thủ công (nếu có) |
| **Trợ lý ngữ pháp câu tiếng Việt** | Kiểm tra + gợi ý + tự sửa lỗi câu vừa gõ (lặp từ, dấu câu, viết hoa, sai dấu) hoàn toàn offline; 0 % báo nhầm trên tập kiểm 981 câu đúng; hoàn tác 1 bước Ctrl+Z | Không có |
| **Escape khôi phục nguyên phím** | Gõ sai ở mọi thời điểm → Escape trả lại đúng chuỗi đã gõ | Nhiều bộ gõ chỉ có undo ở mức câu |
| **Ctrl+Shift đổi mode ở mọi ứng dụng** | Bộ dò tap chỉ quan sát (không ăn phím) + debounce chống lật đôi khi cả tray lẫn engine cùng nhận | Hook toàn cục có thể gây lật đôi / xung đột AV |
| **Không gạch chân preedit, không cài admin** | TSF per-user HKCU, portable giải nén chạy ngay, không UAC | UniKey/EVKey cần cài đặt + một số phiên bản yêu cầu admin |
| **Tự sửa ghost registration** | Đăng ký trỏ vào thư mục đã xoá → chạy lại `TextVN.exe` từ thư mục mới là tự sửa | Phải gỡ/cài lại thủ công |
| **Dialog DPI-aware đa màn hình** | Kéo giữa màn khác DPI tự co giãn + re-layout + font theo DPI | Thường render sai/khểnh khi đa màn hình |
| **Cài im lặng chuẩn Store** | `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` — CI kiểm cài/gỡ im lặng, ARP và giả lập validator của Store ở mỗi PR và mỗi release | Không kiểm chứng luồng Store |
| **Riêng tư mặc định** | Xử lý phím 100% cục bộ, không telemetry, không mạng; tự tắt ở ô mật khẩu; chính sách riêng tư công khai trong repo | Thường không công bố chính sách riêng tư |
| **Số lượng tiến trình** | 1 tiến trình tray + TIP in-process; global keyboard hook chỉ quan sát modifier (không ăn phím, không inject) | UniKey dùng hook toàn cục ăn phím |

### Tính năng đầy đủ (tích luỹ đến 0.2.60)

**Kiểu gõ & biến đổi**
- Telex, VNI, TCVN3 (VIQR đọc được), Quick Telex (cc→ch, gg→gi…), gõ tự do (free marking)
- 3 bảng mã xuất: Unicode, TCVN3, VNI (+ Unicode tổ hợp) — chuyển đổi qua lại giữ nguyên chữ
- Kiểu dấu cũ/mới chọn được (`hoà`/`hòa`, `thuỷ`/`thủy`) — cả dữ liệu dự đoán tự theo kiểu đã chọn
- Chữ hoa thông minh: `DD` → `Đ`, Caps Lock/Shift từng phím, tự viết hoa đầu câu (tắt được)
- Escape khôi phục nguyên chuỗi phím đã gõ ở mọi thời điểm
- Chống vòng lặp phím (E28): gõ lặp dấu không sinh chu kỳ vô tận — đủ 3 kiểu Telex/VNI/VIQR
- Gõ tắt (macro) theo kiểu chữ (HD→HỢP ĐỒNG), chạy được cả khi tắt tiếng Việt (tùy chọn)
- Emoji gõ tắt; số tiền `=1.550.000đ` → chữ Việt; bảo vệ công thức Excel và identifier code

**Gợi ý & dự đoán (P6 — Tab, Ctrl+Space)**
- Hoàn tất từ tiếng Việt: `ngư`+Tab → `ngưng` (3000 âm tiết + từ ghép văn phòng)
- Hoàn tất từ tiếng Anh: `develop`+Tab → `developed` (3000 từ tần suất + thuật ngữ kỹ thuật/kinh doanh + từ điển cá nhân)
- Dự đoán cụm tiếp theo theo ngữ cảnh (`kính gửi ` → `quý khách hàng`; theo role: kế toán, nhân sự, kinh doanh, CSKH, giáo viên — 140+ mẫu Việt / 30+ mẫu Anh)
- Chữ mờ gợi ý sẵn (ghost preview) sau con trỏ trên cả 3 hệ điều hành
- Ctrl+Space: sửa hết lỗi câu (kể cả chính tả) rồi dự đoán tiếp — một phím hai việc

**Trợ lý ngữ pháp (kiểu Grammarly, offline hoàn toàn)**
- 15 loại lỗi: lặp từ/cụm 2–4 từ, thiếu/thừa khoảng trắng, chữ thường đầu câu, sai dấu thanh, phủ định kép, phẩy cuối câu, phẩy sau liên từ, a/an, cặp từ dễ nhầm EN, đại từ "i", chính tả EN (OS spell), chính tả VI (từ điển), ngày tháng vô lý (32/13, 31/02 — calendar-aware), số tiền phân cách hỗn loạn (1.250.00đ)
- Panel popup tại caret (Windows: overlay; macOS: bảng ứng viên; Linux: lookup table + aux) — bấm được bằng chuột, Alt+1…3, Enter áp "Sửa tất cả"
- Mode 1 chỉ gợi ý / Mode 2 tự sửa (lỗi objective tự sửa, lỗi mơ hồ gợi ý) — hoàn tác Ctrl+Z 5 giây
- Kiểm tra theo vùng bôi đen (Shift+mũi tên → popup + Sửa tất cả) và khi bôi đen câu dài
- Per-app: tắt riêng từng ứng dụng; tự tắt ở ô mật khẩu; FPR 0 % trên 2739 câu đúng chuẩn mực

**Chuyển mã & tiện ích văn phòng (P5)**
- Ctrl+Shift+F9 chuyển mã vùng bôi đen: TCVN3/VNI/Unicode tổ hợp → Unicode, tự nhận diện bảng mã (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), gõ lại chữ gõ nhầm khi quên bật (`tooi ddi hocj` → `tôi đi học`) — cả 3 hệ điều hành
- 3000 từ tần suất + thuật ngữ EN (coder/tech/business/OS APIs) cho restore tiếng Anh thông minh
- Bảo vệ: từ điển EN cá nhân, identifier camelCase, chuỗi trong nháy kép nguyên văn

**Điều khiển & riêng tư**
- Ctrl+Space sửa/dự đoán · Ctrl+Shift gạt VN/EN (nhớ trạng thái) · Ctrl+Shift+F9 chuyển mã
- Per-app override + ignore list; bảng cài đặt riêng từng OS (tray Windows, GTK4 Linux, app menu macOS)
- Offline 100 %, không telemetry, không mạng; tự tắt ô mật khẩu; riêng tư công khai trong repo

Engine Rust dùng chung, 190+ test trợ lý + 230 kịch bản replay qua 4 adapter trên 3 OS mỗi commit.

Engine bằng Rust, test bằng corpus replay (hơn 230 kịch bản `*.keys`) qua 4 mô phỏng
adapter (headless, tsf, mac, linux) trên 3 hệ điều hành ở mỗi commit; quy tắc xung đột
ngôn ngữ công khai tại
[docs/specs/language-detection.md](docs/specs/language-detection.md).

## Trạng thái

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | ✅ Release candidate — gõ thật qua TSF được CI kiểm tra trên Windows (bộ cài, portable, MSIX cài thử). Còn thiếu duy nhất chữ ký Authenticode (chờ SignPath Foundation duyệt). **Đã phát hành trên [Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (gói MSIX, từ 2026-10-11). |
| macOS 13+ Apple Silicon / Intel | ✅ Release candidate — bộ gõ IMK + app menu bar + `.pkg` per-user (không sudo); CI build 2 kiến trúc (arm64 + x86_64), chạy `swift test` trên arm64. Chưa ký Developer ID/notarize (bản phát hành ký ad-hoc, `.pkg` chưa ký) → Gatekeeper cảnh báo lần mở đầu ([signing-status-mac.md](docs/release/signing-status-mac.md)). Module CGEventTap (`adapters/macos-tap`) có mã + test nhưng **không** có trong bản phát hành. |
| Linux IBus / Fcitx5 | ✅ Release candidate — tarball `install.sh` (per-user hoặc `--system`) + chạy ngay, Settings GTK4; CI kiểm tra với ibus-daemon và fcitx5 thật. Đã có AUR (`paru -S textvn`) và PPA Ubuntu/Debian (`ppa:hunglinhpt/textvn`, chưa có `.rpm`). |

Kết quả kiểm thử của phiên bản hiện tại: [build-release-report.md](docs/release/build-release-report.md) ·
thay đổi: [CHANGELOG.md](CHANGELOG.md).

### Tóm tắt phiên bản gần đây

| Phiên bản | Nội dung chính |
|---|---|
| 0.2.60 | Hiệu năng: 5 đường O(n²) → tuyến tính (dán 640k ký tự 45s → 0.9s); ngày vô lý cuối văn bản |
| 0.2.59 | Cảnh báo ngày tháng vô lý (32/13, 31/02 — calendar-aware) + số tiền phân cách hỗn loạn (1.250.00đ) |
| 0.2.58 | Chat không dấu hết bị phá chữ ("an com", "i di"); trợ động từ EN lạc sau từ VI; parity phẩy-but |
| 0.2.57 | VIQR freeze marker; macOS toggle VI/đổi cài đặt giữa từ hết mất chữ; `DD` hoa gạch |
| 0.2.56 | EN mode: luật EN chạy trên câu mix; WinSpell tự phục hồi; nguyên văn trong ngoặc kép không sửa; SVA O(n) |
| 0.2.55 | Lặp cụm 2–4 từ; từ EN thừa trong câu Việt; audit 3 pass A/B/C; dự đoán nhanh 35× |
| 0.2.54 | Lặp cụm suggestion-only; audit 3 OS từng dòng; abnormal tests vét cạn |
| 0.2.52–53 | SVA dạng rút gọn ("He don't"); EN Tab mở cho từ chưa fold; collocation +7 |
| 0.2.50–51 | EN grammar hoàn tất (should of, your welcome, then/than…); Ngữ pháp câu EN + SVA; bảng cài đặt bật dự đoán |
| 0.2.45–49 | P5 văn phòng: macro kiểu chữ, số tiền, bảo vệ Excel/identifier; Ctrl+Shift+F9 chuyển mã thông minh 3 OS; ghost preview |
| 0.2.44 | Audit sâu 3 lượt; 1061 âm tiết VN cho từ thiếu dấu; a/an theo gốc Âm-phiêu |

Chi tiết đầy đủ từng bản: [CHANGELOG.md](CHANGELOG.md).

## Dựng từ mã nguồn

```powershell
# Windows: Rust stable + VS 2022 Build Tools (C++)
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller   # cần Inno Setup 6.5+
```

```bash
# macOS 13+: Xcode CLT 15+, rustup với targets aarch64-apple-darwin + x86_64-apple-darwin
cd adapters/macos-imk
./build-rust.sh                   # build engine + swift build + assemble TextVN-IM.app
./build-rust.sh --lib-only        # chỉ build libtextvn_ffi.a (cho swift test)
./build-rust.sh --release         # build bản release
```

```bash
# macOS — đóng gói phát hành (từ thư mục gốc repo; script tự kiểm tool khi chạy)
scripts/build-macos.sh                            # universal nếu đủ 2 arch; nếu không, tên zip ghi arch thực tế
scripts/package-macos-pkg.sh --rebuild            # .pkg per-user (pkgbuild + productbuild, không sudo)
scripts/package-macos-pkg.sh --rebuild --notarize # + notarytool + staple; ký Developer ID chỉ khi có
                                                  #   secret Apple — docs/release/signing-status-mac.md
scripts/notarize-macos.sh dist/macos/TextVN-mac-v<ver>.pkg   # notarize riêng lẻ (tuỳ chọn)
```

```bash
# Linux (Ubuntu/Debian)
scripts/install_linux.sh        # dựng + cài per-user
scripts/build-linux.sh          # tạo tarball phát hành
```

Kiến trúc, kiểm thử, quy ước và quy trình phát hành: **[Hướng dẫn phát triển](docs/developer-guide.md)**.

## Cấu trúc

```
core/              engine (Telex/VNI/VIQR, đặt dấu, bảng mã, gõ tắt) — không phụ thuộc OS
config/  ffi/      cấu hình config.v1 · C ABI cho adapter
adapters/          macos-imk · macos-tap · macos-app
                   windows-tsf · windows-hook (gói Compatibility)
                   linux-ibus · linux-fcitx5 · linux-common · linux-settings (GTK4)
tray/  cli/        khay + bảng điều khiển Windows · textvn-cli (register, doctor, replay)
corpus/            kịch bản gõ *.keys (shared · win · mac) chạy qua mô phỏng của mọi adapter
installer/  packaging/  scripts/    đóng gói và kiểm thử gói
docs/              đặc tả (10-shared, 20-windows, 30-macos, 40-linux), specs/, release/
```

## Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) và [SECURITY.md](SECURITY.md). Báo lỗi gõ: ghi
chính xác chuỗi phím, kết quả nhận được, ứng dụng và hệ điều hành.

## Giấy phép

[GPL-3.0-or-later](LICENSE) © 2026 LinhBH.CoM


---

# English

TextVN is an open-source Vietnamese input method. One shared Rust engine powers
**Windows** (Text Services Framework), **macOS** (Input Method Kit) and
**Linux** (IBus, Fcitx5), with the same control panel, the same options and the
same config file on all three platforms.

- **Types like UniKey/OpenKey**: Telex, VNI, VIQR, Simple Telex; standard tone
  placement with **old-style diacritics by default** (`hòa`, `thủy`, `khỏe`;
  switch to new style `hoà`/`thuỷ` in the control panel), `uow` → ươ, `z`
  strips tones, tone keys work after the vowel or at the end of the word, and
  Caps Lock typing is supported.
- **No lost characters, no doubled characters, no underline**: the whole word
  stays inside the composition range with no underline (TF_LS_NONE on Windows
  TSF; macOS uses non-preedit typing after install; Fcitx5/IBus attach no
  underline — only Chromium/Electron/Qt apps on Linux draw their own) and is
  committed at word boundaries — no synthetic Backspace, so none of the
  classic hook-style IME bugs in Chrome, Electron, Excel or the address bar.
- **Avoids antivirus false positives**: the default package installs no global
  keyboard hook, injects no code into other processes and never fakes keys
  with `SendInput` ([why](docs/specs/antivirus-false-positive.md)).
- **Utilities**: TCVN3 / VNI-Windows / decomposed-Unicode output charsets,
  text shortcuts, Quick Telex, English auto-restore (3000 most frequent
  words), Tab completion — English always on, Vietnamese via `suggest.enabled`
  — automatic sentence capitalization, instant Ctrl+Shift VI/EN toggle with
  tray-icon state.
- **Grammar assistant (experimental, on by default in suggest-only mode)**:
  checks the sentence you just typed — Vietnamese **and English, including
  office-style mixed sentences** ("vui lòng check lại file report nhé") —
  fully on-device (nothing sent anywhere, nothing logged): duplicate words,
  spaces around punctuation, sentence capitalization, misplaced tone marks
  ("nghiã" → "nghĩa"), English a/an ("a apple" → "an apple") with a
  suggestion popup at the caret that underlines the wrong words and shows
  the total count, Grammarly-style; the *Auto-fix* mode fixes the certain
  errors and supports a one-step Ctrl+Z undo for 5 seconds. Quality bar
  "never flags correct text": **0 %** of correct sentences flagged on a
  981-sentence Vietnamese + 295-sentence English benchmark (0.2.38,
  details in [P4-12 §11](docs/50-assist/P4-12-RECALL-REAL.md)).

## Download & install

Latest: **[GitHub Release v0.2.60](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.60)**
(release candidate, built and tested by CI). **Every release file carries a GPG
signature (`.asc`) and a Sigstore signature (`.cosign.sig` + `.cosign.cert`)**;
`SHA256SUMS.txt` is clear-signed — how to verify: [signing.md](docs/release/signing.md).
Not yet available: Authenticode signing for Windows (waiting on SignPath
Foundation — SmartScreen may warn, antivirus may mis-quarantine: seen with
Kaspersky, see
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) and
Apple Developer ID / notarization for macOS (Gatekeeper warns on first open).
0.2.60 is the **performance-audit release**: five quadratic paths in the assistant are now linear (640k-char paste 45s → 0.9s); end-of-text invalid dates detected again. 0.2.59 adds **impossible-date warnings** ("32/13/2025", "31/02/2026" — calendar-aware) and **inconsistently-grouped money warnings** ("1.250.00đ") from the accounting/HR/sales personas. 0.2.58 is a **seven-persona audit release** (accounting/HR/sales/support/teachers/PMs/everyday users): unaccented Vietnamese chat is no longer mangled by EN rules ("an com", "i di"), stray EN auxiliaries after Vietnamese words are flagged ("Chưa been đối chiếu"), and comma-after-"but" mirrors the Vietnamese rule. 0.2.57 is a **whole-surface three-pass audit release**: VIQR freezes repeated markers like Telex/VNI (no endless toggle), macOS no longer loses the word being typed when toggling Vietnamese or reloading settings mid-word, uppercase DD strokes to Đ. 0.2.56 is the **English-mode audit release**: EN grammar rules now apply on mixed Vietnamese-dominant sentences ("Anh should of done it sớm hơn") and unaccented Vietnamese text ("noi it do met"), Windows spell-check recovers after a slow probe, quoted speech is no longer auto-corrected, SVA on long text is O(n). 0.2.55 is a **three-pass line-by-line audit release** (assistant rules + engine/FFI + all 3 OS adapters): fixes three auto-fix rules that corrupted correct text ("Tuy giỏi nhưng không nên kiêu ngạo", "more minutes, then try again", "their going rate"), the assistant no longer disables itself on short mixed sentences ("y kiến hay"), closes an IBusText leak, and speeds prediction ~35× (715µs → 20µs per keystroke); plus 40+ idioms, 20+ reduplicative words, 25 office collocation pairs and 142 new role-based prediction patterns. 0.2.54 adds **multi-window phrase-repeat detection** (suggestion-only: "vui lòng vui lòng kiểm tra", "đã hoàn thành đã hoàn thành"), **redundant English words in Vietnamese sentences** ("đã been hủy", "trước before 17h"), **19 office collocation pairs**, **68 idioms/proverbs + 24 reduplicative words** to the protected whitelist, **role-based phrase prediction** (accounting/HR/sales/support/teachers, +132 patterns), and cross-platform hardening: Windows no longer loses text on error paths, macOS restores the Ctrl+Shift+F9 chord, Linux no longer freezes typing when the tray hangs. 0.2.53 fixes **English subject-verb agreement for contractions** ("He don't" → "doesn't"), adds seven Vietnamese collocation pairs (sẳn sàng → sẵn sàng, chần chờ → chần chừ, trực trặc → trúc trắc as a whole-phrase fix) and opens **English Tab completion for unfolded words** when prediction is enabled.
0.2.52 expands the **English grammar assistant & text prediction engine**: 3356 frequent English words (coder, tech, business, Win32/POSIX/Cocoa APIs, Gen Z slang), 3750 contextual next-phrase prediction patterns (Tab / Ctrl+Space), closed-list confusion rules (`their own`, `except for`, `led to`, `lose`, `principal`, `take effect`), and dual-language Windows OS spell checking with RAII ComApartment protection.
0.2.51 completes **English grammar** (previously only a/an): fixes `should of`→`should have`, `your welcome`→`you're welcome`, `its been`→`it's been`, `their going`→`they're going`, `is to big`→`is too big`, `better then`→`better than`, lowercase pronoun `i`→`I` — closed lists, English-dominant sentences only.
0.2.50 completes the loop: **Ctrl+Shift+F9 on all three platforms** (macOS + Linux were missing), **Windows OS spell checking** (D39), a **prediction toggle reachable in the settings UI** (it was config-only before), and a three-pass audit fixing 2 HIGH + 5 MEDIUM + 10 LOW.
0.2.49 makes **Ctrl+Shift+F9 a smart selection converter**: repairs garbled text from files opened with the wrong encoding, converts legacy TCVN3/VNI text to the current encoding, and re-types text typed while Vietnamese was off (`tooi ddi hocj` → `tôi đi học`); the assistant now fixes WinForms/Delphi text boxes; the installer and tray remind you to reopen running apps after an update.
0.2.48 makes the assistant hotkey **Ctrl+Space only** (Windows/Linux; Option+Space on macOS) — Alt+Space is dropped because it belongs to the window menu and to PowerToys Run/Copilot/ChatGPT.
0.2.47 adds **ghost text** after the caret (Windows; macOS/Linux preview it in the candidate panel) and **one hotkey that fixes then predicts**: Ctrl+Space (Option+Space on macOS) fixes every issue of the sentence (spelling included) before predicting, and passes the key to the app when there is nothing to do; *Fix all* and Alt+1..3 now work in every app, Teams/Electron (VS Code) and Notepad++ included (real-app E2E 44/44).
0.2.46 adds **bilingual text prediction** (opt-in `suggest.enabled`: Tab/Ctrl+Space completes words and predicts the next phrase — *Kính gửi* → *quý khách hàng*) and a **live, Grammarly-style grammar assistant** on all three platforms: suggestions appear as soon as a sentence is wrong and disappear once it is fixed, without stealing typing keys; the Windows panel now always shows (it could not anchor in Windows 11 Notepad), is larger and easier to read, and the noisy "issues detected" toast is gone; the installer scans for and cleans up older installs (real-app E2E 28/28 on Notepad/Word/Excel). 
0.2.45 ships the **P5 office-productivity pack**: case-adaptive text macros (HD→HỢP ĐỒNG), spreadsheet formula guard (`=sum(...)`), money shorthand (`=15450000bc`+Tab → *Mười lăm triệu đồng chẵn*), office collocation fixes (bổ xung→bổ sung), percent spacing (`100 %`→`100%`), coder/business identifier protection (useState/KPI/EBITDA) and a **selection transcoder on Ctrl+Shift+F9** (TCVN3/VNI/decomposed Unicode → Unicode).
0.2.44 is a **three-pass deep audit of the whole grammar assistant** (locks in
zero false positives: diacritic-less Vietnamese, office file names, curly
quotes, ĐH. abbreviations, DPI-scaled panel, Fcitx5 fresh install).
0.2.43 makes **Enter on the selected-text popup apply "Fix all"** (it used to
swallow the key and apply nothing — the last open finding from the
selection-check audit); 0.2.42 updates the popup right after fixing so it
closes when the sentence is clean or refreshes with the remaining issues;
0.2.41 adds **selected-text grammar checking** (select 3–256 chars with the
keyboard → 500 ms debounce → popup with "Fix all") and clickable suggestion
rows/underlines; 0.2.38–0.2.40 enabled the **grammar assistant by default in
suggest-only mode** with a Grammarly-style caret popup and English `a`/`an`
checks in English and mixed VI-EN sentences.
0.2.36 hardens the **grammar assistant** to the "never flags correct text"
bar: **0 % false positives** on 981 clean sentences (removes 5 policy-violating
typo pairs that made autocorrect rewrite correct words like "quyến rũ",
"thuở xưa"; sentence recall 83.7 %), all-caps accented sentences are now
checked. 0.2.35 introduced the **grammar assistant**; 0.2.34 opened **Linux
distribution channels**: AUR (`paru -S textvn`) and the Debian/Ubuntu PPA;
0.2.33 switched the **fresh-install default to old-style diacritics** on all
three platforms (existing users keep their choice) and added the 16-combo
charset × method regression matrix, a 3000-word English auto-restore
dictionary and optional Vietnamese Tab completion — details:
[CHANGELOG.md](CHANGELOG.md). Release commits and tags are git-signed.

| Platform | Install | Run without installing |
|---|---|---|
| Windows 10/11 x64 | `TextVN-setup-<ver>-windows-x64.exe` (current user, no admin) · `TextVN-setup-<ver>-windows-x64-machine.exe` (all users, Program Files, one UAC prompt) | `TextVN-portable-<ver>-windows-x64-*.zip` → run `TextVN.exe` |
| Windows (Microsoft Store) | ✅ **[TextVN on Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (free, Store ID `9NV2R7JNNFGK`) — click *Get*, open TextVN once to register the input method for your account; the Store updates it automatically, package signed by Microsoft. CLI: `winget install --id 9NV2R7JNNFGK -s msstore`. The `TextVN-<ver>-windows-x64.msix` on GitHub Release is the Store submission package (unsigned) — not directly installable | — |
| macOS 13+ (Apple Silicon / Intel) | **Homebrew (recommended): `brew install --cask hunglinhpt/tap/textvn`** — puts the input method into `~/Library/Input Methods` and the menu-bar app into `/Applications`; updates via `brew upgrade`. Or `TextVN-mac-v<ver>.pkg` → open to install, or `installer -pkg … -target CurrentUserHomeDirectory` (no sudo) | `TextVN-macos-universal-v<ver>.zip` (or `.tar.gz`) with `TextVN-IM.app` + `TextVN.app` — see the [user guide, macOS section](docs/user-guide.md) |
| Linux (IBus / Fcitx5) | **AUR** (Arch/Manjaro…): `paru -S textvn` — builds from the tagged source, real makepkg QA in CI · **Ubuntu PPA** (prepared — owner activates `ppa:hunglinhpt/textvn`, see [linux-distribution.md](docs/release/linux-distribution.md)): `sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn` · Or `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → `./install.sh` (per-user, no root; `sudo ./install.sh --system` for all users) — Linux store-channel analysis: [linux-distribution.md](docs/release/linux-distribution.md) | same tarball → `./textvn-portable.sh` |

Install, uninstall, usage and troubleshooting: **[User guide](docs/user-guide.md)** (Vietnamese).

Windows note: `TextVN-setup-<ver>-windows-x64.exe` installs **for the current
user** (`%LOCALAPPDATA%\Programs\TextVN`, no UAC);
`TextVN-setup-<ver>-windows-x64-machine.exe` installs **for all users**
(Program Files, one UAC prompt). Each Windows account gets the input method
registered on TextVN's first run; upgrades **keep** your typing method,
options and per-app settings. On **Windows 11 24H2+** some machines reject
per-user-only input methods (bug B7 in `docs/specs/win-test-common-errors.md`):
the per-user installer, the portable build and the Store package therefore
**never ask for UAC** — TextVN tells you once and points to the
`-machine.exe` installer (only a Program Files install can register
machine-wide).

## Highlights over other Vietnamese IMEs

What TextVN does that UniKey / EVKey / OpenKey currently **don't** (or do
differently — per their public feature pages as of 2026-10):

- **Automatic EN/VI detection while typing mixed text** — a 3000-word English
  frequency dictionary plus a common-Vietnamese-syllable protection grid:
  type both languages without switching modes; English words come back intact
  at the space. Traditional IMEs transform everything — `text` becomes `tẽt`.
- **Mode-based Vietnamese priority** — ambiguous pairs (`cow`/`cơ`,
  `sex`/`sẽ`, `queen`/`quên`) resolve by the current mode, UniKey-style for
  Vietnamese while staying friendly to English.
- **Tab completion** — half-typed `tes` + Tab → `test` (English, always on);
  `ngư` + Tab → `ngưng` (Vietnamese, 3000 frequent syllables, enable via
  `suggest.enabled`).
- **Vietnamese sentence grammar assistant** — checks, suggests and (opt-in)
  auto-fixes the sentence you just typed (duplicate words, punctuation
  spacing, capitalization, misplaced tone marks), fully offline; 0 % of
  correct sentences flagged on a 981-sentence benchmark; one-step Ctrl+Z undo.
- **Escape restores your raw keystrokes** at any moment.
- **Ctrl+Shift mode switch in every app** — an observing-only tap detector
  (eats no keys) with double-toggle debouncing.
- **No preedit underline, no admin install** — per-user TSF in HKCU, portable
  builds that just run, no UAC.
- **Self-healing ghost registrations** — if the registration points to a
  deleted folder, run `TextVN.exe` from the new folder and it fixes itself.
- **DPI-aware dialogs** across mixed-DPI monitors.
- **Store-grade silent install** — `/VERYSILENT /SUPPRESSMSGBOXES
  /NORESTART`; CI exercises silent install/uninstall, ARP entries and a
  Store-validator simulation on every PR and release.
- **Private by default** — 100 % local key processing, no telemetry, no
  network; auto-disables in password fields; public privacy policy in-repo.

### Full feature set (accumulated through 0.2.60)

**Input methods & transforms**
- Telex, VNI, TCVN3 (VIQR readable), Quick Telex (cc→ch, gg→gi…), free marking
- 3 output charsets: Unicode, TCVN3, VNI (+ decomposed Unicode) — lossless round-trip conversion
- Old/new diacritic style switch (`hoà`/`hòa`, `thuỷ`/`thủy`) — prediction data follows the chosen style
- Smart caps: `DD` → `Đ`, per-key Caps Lock/Shift, auto-capitalize sentence starts (optional)
- Escape restores the exact typed string at any point
- Anti-key-loop (E28): repeated tone keys never cycle endlessly — across Telex/VNI/VIQR
- Text macros with case styles (HD→HỢP ĐỒNG); work even with Vietnamese off (optional)
- Emoji shortcuts; `=1.550.000đ` → Vietnamese words; Excel formula and identifier protection

**Suggestions & prediction (P6 — Tab, Ctrl+Space)**
- Vietnamese word completion: `ngư`+Tab → `ngưng` (3000 syllables + office compounds)
- English completion: `develop`+Tab → `developed` (3000 frequency words + tech/business terms + personal dictionary)
- Next-phrase prediction by context (`kính gửi ` → `quý khách hàng`; role-based: accounting, HR, sales, support, teachers — 140+ VI / 30+ EN patterns)
- Ghost preview after the caret on all three platforms
- Ctrl+Space: fix the whole sentence (including spelling) then keep predicting — one key, two jobs

**Grammar assistant (Grammarly-style, fully offline)**
- 15 issue kinds: word/2–4-word-phrase repeats, spacing, sentence-case, diacritic typos, double negatives, end-of-sentence and post-conjunction commas, a/an, EN confusions, lowercase "i", EN spelling (OS spell), VI spelling (dictionary), impossible dates (32/13, 31/02 — calendar-aware), inconsistent money grouping (1.250.00đ)
- Popup panel at the caret (Windows overlay; macOS candidate panel; Linux lookup table + aux) — mouse-clickable, Alt+1…3, Enter applies fix-all
- Mode 1 suggest-only / Mode 2 auto-correct (objective errors fixed, ambiguous ones suggested) — Ctrl+Z undo for 5 seconds
- Selection checking (arrow keys → popup + fix-all) and long-sentence checks
- Per-app disable; auto-off in password fields; 0 % false positives on a 2739-sentence curated clean set

**Transcoding & office extras (P5)**
- Ctrl+Shift+F9 converts the selection: TCVN3/VNI/decomposed → Unicode with automatic charset detection (`Tiáº¿ng Viá»‡t` → `Tiếng Việt`), re-types text typed while the IME was off (`tooi ddi hocj` → `tôi đi học`) — all three platforms
- 3000 frequency words + EN terminology (coder/tech/business/OS APIs) for smart English restore
- Protection: personal EN dictionary, camelCase identifiers, quoted strings kept verbatim

**Control & privacy**
- Ctrl+Space fix/predict · Ctrl+Shift VI/EN toggle (state remembered) · Ctrl+Shift+F9 transcode
- Per-app overrides + ignore lists; native settings per OS (Windows tray, Linux GTK4, macOS menu bar app)
- 100 % offline, no telemetry, no network; auto-off in password fields; public privacy policy in the repo

One Rust engine, 190+ assistant tests + 230 replay scenarios through 4 adapter simulators on 3 OSes per commit.

The Rust engine is tested by corpus replay (230+ `*.keys` scenarios) through
four adapter simulations (headless, tsf, mac, linux) on three operating
systems at every commit; the language-conflict rules are published at
[docs/specs/language-detection.md](docs/specs/language-detection.md).

## Status

| Platform | Status |
|---|---|
| Windows 10/11 x64 (TSF) | ✅ Release candidate — real TSF typing is CI-tested on Windows (installer, portable, MSIX trial install). Only Authenticode signing is missing (waiting on SignPath Foundation). **Published on the [Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (MSIX, since 2026-10-11). |
| macOS 13+ Apple Silicon / Intel | ✅ Release candidate — IMK input method + menu-bar app + per-user `.pkg` (no sudo); CI builds both architectures (arm64 + x86_64) and runs `swift test` on arm64. No Developer ID / notarization yet (releases are ad-hoc signed) → Gatekeeper warns on first open ([signing-status-mac.md](docs/release/signing-status-mac.md)). The CGEventTap module (`adapters/macos-tap`) has code + tests but is **not** shipped. |
| Linux IBus / Fcitx5 | ✅ Release candidate — `install.sh` tarball (per-user or `--system`) + run-in-place, GTK4 settings; CI tests with real ibus-daemon and fcitx5. AUR PKGBUILD + Debian `debian/` packaging are in-repo with real build QA (see [linux-distribution.md](docs/release/linux-distribution.md)). |

Test results of the current version:
[build-release-report.md](docs/release/build-release-report.md) · changes:
[CHANGELOG.md](CHANGELOG.md).

### Recent releases at a glance

| Version | Highlights |
|---|---|
| 0.2.60 | Performance: five O(n²) paths → linear (640k-char paste 45s → 0.9s); end-of-text date detection |
| 0.2.59 | Impossible-date warnings (32/13, 31/02 — calendar-aware) + inconsistent money grouping (1.250.00đ) |
| 0.2.58 | Unaccented Vietnamese chat no longer mangled ("an com", "i di"); stray EN auxiliaries flagged; comma-after-"but" parity |
| 0.2.57 | VIQR marker freeze; macOS toggle/settings mid-word no longer lose text; uppercase DD stroke |
| 0.2.56 | EN rules run on mixed sentences; WinSpell self-recovery; quoted speech verbatim; SVA O(n) |
| 0.2.55 | 2–4-word phrase repeats; stray EN words in Vietnamese sentences; 3-pass A/B/C audit; 35× prediction speedup |
| 0.2.54 | Phrase repeats suggestion-only; line-by-line 3-OS audit; exhaustive abnormal tests |
| 0.2.52–53 | Contracted SVA ("He don't"); EN Tab for unfolded words; +7 collocations |
| 0.2.50–51 | Full EN grammar (should of, your welcome, then/than…); sentence grammar + SVA; prediction toggle UI |
| 0.2.45–49 | P5 office: case-style macros, money words, Excel/identifier protection; smart Ctrl+Shift+F9 transcode 3 OS; ghost preview |
| 0.2.44 | Deep 3-round audit; 1061 VN syllables for unaccented words; vowel-origin a/an rules |

Full details per release: [CHANGELOG.md](CHANGELOG.md).

## Build from source

```powershell
# Windows: Rust stable + VS 2022 Build Tools (C++)
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller   # needs Inno Setup 6.5+
```

```bash
# macOS 13+: Xcode CLT 15+, rustup with aarch64-apple-darwin + x86_64-apple-darwin targets
cd adapters/macos-imk
./build-rust.sh                   # build engine + swift build + assemble TextVN-IM.app
./build-rust.sh --lib-only        # libtextvn_ffi.a only (for swift test)
./build-rust.sh --release         # release build
```

```bash
# Linux (Ubuntu/Debian)
scripts/install_linux.sh        # build + install per-user
scripts/build-linux.sh          # produce the release tarball
```

Architecture, testing, conventions and the release process: see the
**[developer guide](docs/developer-guide.md)** (Vietnamese) and
[docs/release/release-process.md](docs/release/release-process.md).

## License

[GPL-3.0-or-later](LICENSE) © 2026 LinhBH.CoM
