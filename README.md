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

## Phiên bản gần đây

Chi tiết đầy đủ từng bản: [CHANGELOG.md](CHANGELOG.md).

| Phiên bản | Nội dung chính |
|---|---|
| [0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65) | Chuẩn UniKey vần uo ("luong"/"uong" plain, `luonwg`/`tuowi`/`tuowng`) · Hook swallow-and-unicode chống xung đột OS-IME (MobaXterm) · Dead-key Windows (`Ctrl+'`+`e`→`é`) · Panel debounce 1s · Chặn hoàn tất username/id (`truong.giap`) · 3 gate suites 3000 case + DFMEA/DRBFM |
| [0.2.64](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.64) | Corpora 35.681 mục · 13 domain từ ghép mới (bất động sản, ngân hàng-bảo hiểm, logistics…) · gen-vocab Rust thay Python · User Style Profile (học từ cá nhân, opt-in) · abnormal_1000 · DFMEA/DRBFM |
| [0.2.63](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.63) | Corpora ~31.000 mục (6.675 âm tiết + 1.214 từ ghép + 9.868 từ Anh + 10.208 câu/cụm/thành ngữ) · mở rộng đúng nghĩa · QA văn phòng 23 test · 3 fix FPR |
| [0.2.62](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.62) | Audit 3 pass MobaXterm + sweep dòng terminal (KiTTY/SecureCRT/Xshell) hook-owner · Panel không đè dòng caret |
| [0.2.61](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.61) | Khắc phục gõ MobaXterm/PuTTY (appdb exe_prefix) · e2e Linux dự đoán văn bản (ghost/Tab) trên IBus + Fcitx5 |
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

## Tải và cài

Bản mới nhất: **[GitHub Release v0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65)**
(release candidate do CI build và kiểm thử). **Mọi file phát hành đều có chữ ký GPG
(`.asc`) và Sigstore (`.cosign.sig` + `.cosign.cert`)**, `SHA256SUMS.txt` được clearsign
— cách kiểm: [signing.md](docs/release/signing.md).

| Nền tảng | Cài đặt | Giải nén dùng ngay |
|---|---|---|
| Windows 10/11 x64 | `TextVN-setup-<ver>-windows-x64.exe` (cho tài khoản của bạn, không cần admin) · `TextVN-setup-<ver>-windows-x64-machine.exe` (cho mọi người dùng, Program Files, UAC một lần) | `TextVN-portable-<ver>-windows-x64-*.zip` → chạy `TextVN.exe` |
| Windows (Microsoft Store) | ✅ **[TextVN trên Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (miễn phí, Store ID `9NV2R7JNNFGK`) — bấm *Tải*, mở TextVN một lần để đăng ký bộ gõ cho tài khoản; Store tự cập nhật, gói do Microsoft ký. Dòng lệnh: `winget install --id 9NV2R7JNNFGK -s msstore`. File `TextVN-<ver>-windows-x64.msix` trên GitHub Release là gói nộp Store (chưa ký) — không cài trực tiếp được | — |
| macOS 13+ (Apple Silicon / Intel) | **Homebrew (khuyên dùng): `brew install --cask hunglinhpt/tap/textvn`** — tự đặt bộ gõ vào `~/Library/Input Methods` và app menu bar vào `/Applications`; cập nhật theo `brew upgrade`. Hoặc `TextVN-mac-v<ver>.pkg` → mở để cài, hoặc `installer -pkg … -target CurrentUserHomeDirectory` (không sudo) | `TextVN-macos-universal-v<ver>.zip` (hoặc `.tar.gz`) chứa `TextVN-IM.app` + `TextVN.app` — xem [Hướng dẫn sử dụng, mục macOS](docs/user-guide.md) |
| Linux (IBus / Fcitx5) | **AUR** (Arch/Manjaro…): `paru -S textvn` — build từ mã nguồn tag, QA makepkg thật trong CI · **PPA Ubuntu** (chuẩn bị — owner kích hoạt `ppa:hunglinhpt/textvn`, xem [linux-distribution.md](docs/release/linux-distribution.md)): `sudo add-apt-repository ppa:hunglinhpt/textvn && sudo apt install textvn` · Hoặc `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → `./install.sh` (per-user, không cần root; `sudo ./install.sh --system` cho mọi người dùng) — phân tích kênh store Linux: [linux-distribution.md](docs/release/linux-distribution.md) | cùng tarball → `./textvn-portable.sh` |

Chi tiết cài, gỡ, sử dụng và xử lý sự cố: **[Hướng dẫn sử dụng](docs/user-guide.md)**.

**Lưu ý Windows**: bản `-machine.exe` cài **cho mọi người dùng** (Program Files, UAC một
lần); bản thường cài cho tài khoản hiện tại (`%LOCALAPPDATA%\Programs\TextVN`, không
UAC). Trên **Windows 11 24H2+** một số máy từ chối kích hoạt bộ gõ chỉ đăng ký cho tài
khoản (lỗi B7 trong `docs/specs/win-test-common-errors.md`): bản cài tài khoản, bản
portable và bản Store **không bao giờ xin UAC** — TextVN báo một lần và hướng dẫn dùng
bộ cài `-machine.exe`.

**Lưu ý chữ ký chưa có** (đang chờ duyệt — SmartScreen/diệt-virus có thể cảnh báo lần
đầu): Authenticode cho Windows (chờ SignPath — đã gặp Kaspersky cách ly nhầm, xem
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) và Apple
Developer ID/notarization cho macOS (Gatekeeper lần mở đầu). Xem [Trạng thái](#trạng-thật).

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

## Tính năng đầy đủ (tích luỹ đến 0.2.65)

**Dự đoán văn bản (P6) & Phong cách cá nhân (P7)** — corpora 35.681 mục:
6.672 âm tiết Việt + 1.941 từ ghép (13 domain mới: bất động sản, ngân hàng-bảo
hiểm, logistics, sản xuất, đấu thầu, y tế, du lịch…) + 9.868 từ Anh + 13.854 cặp
câu/cụm/thành ngữ (câu thật chia mọi ranh giới từ, thành ngữ thật, curated văn
phòng theo role); User Style Profile (học từ cá nhân & cặp sửa lỗi, opt-in,
100% offline, cap 200 từ / 50 cặp); Tab/Ctrl+Space hoàn tất từ & đoán cụm;
xếp hạng mở rộng đúng nghĩa; tra cứu micro-giây.

**Kiểu gõ & biến đổi (chuẩn UniKey 20 năm)**
- Vần `uo` chuẩn UniKey: `luong`/`uong`/`truong.giap` giữ nguyên khi không dấu;
  chỉ khi có biến đổi VN hoặc gõ `w`/dấu chủ động mới thành `ươ` (`luonwg`,
  `tuowi`, `tuowng`, `duongf`→`dường`)
- Chống xung đột 2 bộ gõ (swallow-and-unicode): trên app bypass TSF (MobaXterm,
  KiTTY…), hook tự nuốt phím và inject qua KEYEVENTF_UNICODE ngăn Windows VIE IME
  cướp phím và hiện ô chữ nhật nhỏ
- Khôi phục dead-key Windows (`Ctrl+'`+`e` → `é`) trên bàn phím quốc tế
- Chặn hoàn tất token ID/username: không bao giờ tự động hoàn tất qua ranh giới
  dấu chấm, gạch ngang, email (`truong.giap`, `t.ho`)
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
- Panel popup tại caret với debounce 1s: phím gõ re-schedule, popup sống ẩn ngay khi gõ tiếp, dừng ~1s mới check và hiển thị, không bao giờ đè lên dòng caret
- Panel hỗ trợ chuột bấm, Alt+1…3, Enter áp "Sửa tất cả"
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

## Recent releases at a glance

Full details per release: [CHANGELOG.md](CHANGELOG.md).

| Version | Highlights |
|---|---|
| [0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65) | UniKey-standard `uo` rhyme (`luong` plain, `luonwg`/`tuowng`) · Hook swallow-and-unicode fixes OS-IME conflict (MobaXterm) · Windows dead-key restored · Panel 1s debounce · ID/username punctuation gate · 3 gate suites 3000 cases + DFMEA/DRBFM |
| [0.2.64](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.64) | Corpora 35,681 entries · 13 new compound domains · Rust `xtask gen-vocab` replaces Python · User Style Profile (opt-in personal learning) · 1000-case abnormal suite · "bay bay" FP fixed |
| [0.2.63](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.63) | Corpora ~31,000 entries (6,675 syllables + 1,214 compounds + 9,868 EN words + 10,208 phrases) · Meaning-controlled ranking · Office QA battery (23 tests) · 3 final FPR fixes |
| [0.2.62](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.62) | 3-pass MobaXterm audit + terminal family sweep (KiTTY/SecureCRT/Xshell) hook-owned · Panel never overlaps typing line |
| [0.2.61](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.61) | MobaXterm/PuTTY fix (appdb `exe_prefix`) · Linux e2e text prediction (ghost/Tab) on IBus + Fcitx5 |
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

## Download & install

Latest release: **[GitHub Release v0.2.65](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.65)**
(release candidate, built and tested by CI). **Every release file carries a GPG
signature (`.asc`) and a Sigstore signature (`.cosign.sig` + `.cosign.cert`)**;
`SHA256SUMS.txt` is clear-signed — how to verify: [signing.md](docs/release/signing.md).
Not yet available: Authenticode signing for Windows (waiting on SignPath
Foundation — SmartScreen may warn, antivirus may mis-quarantine: seen with
Kaspersky, see
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) and
Apple Developer ID / notarization for macOS (Gatekeeper warns on first open).

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

## Full feature set (accumulated through 0.2.65)

**Text prediction (P6) & User Style Profile (P7)** — 35,681-entry corpora:
6,672 Vietnamese syllables + 1,941 compounds (13 new domains: real estate,
banking & insurance, logistics, manufacturing, tendering, healthcare, travel…)
+ 9,868 English words + 13,854 context→phrase pairs (real sentences, idioms,
role-based patterns); User Style Profile (personal vocabulary & correction
learning, opt-in, 100% offline, strict 200-word / 50-pair cap); Tab/Ctrl+Space
completion & next-phrase; meaningful-extension ranking; microsecond lookups.

**Input methods & transforms (20-year UniKey standard)**
- UniKey-standard `uo` rhyme: `luong`/`uong`/`truong.giap` stay unaccented by default;
  only explicit `w` or tone keys produce `ươ` (`luonwg`, `tuowi`, `tuowng`,
  `duongf`→`dường`)
- Anti OS-IME conflict (swallow-and-unicode): on TSF-bypassing apps (MobaXterm,
  KiTTY…), the hook swallows keystrokes and injects via KEYEVENTF_UNICODE to
  prevent Windows VIE IME from grabbing keys and popping its composition box
- Windows dead-key passthrough (`Ctrl+'`+`e` → `é`) on international layouts
- Identifier and punctuation gate: Space/Tab never completes across dot, dash,
  or email boundaries (`truong.giap`, `t.ho`)
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
- Popup panel at the caret with 1s debounce: keystrokes reschedule, active popup hides instantly when typing resumes, checks and displays only after ~1s idle; never overlaps the caret typing line
- Mouse-clickable panel, Alt+1…3, Enter applies fix-all
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
