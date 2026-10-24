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

Bản mới nhất: **[GitHub Release v0.2.41](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.41)**
(release candidate do CI build và kiểm thử). **Mọi file phát hành đều có chữ ký GPG
(`.asc`) và Sigstore (`.cosign.sig` + `.cosign.cert`)**, `SHA256SUMS.txt` được clearsign
— cách kiểm: [signing.md](docs/release/signing.md). Chưa có: chữ ký Authenticode cho
Windows (đang chờ SignPath Foundation duyệt — SmartScreen có thể cảnh báo, phần mềm diệt
virus có thể cách ly nhầm: đã gặp với Kaspersky, xem
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) và Apple
Developer ID/notarization cho macOS (Gatekeeper cảnh báo lần mở đầu). Bản 0.2.41 sửa trợ lý chết khi nâng cấp (3 root cause). Bản 0.2.40 là vòng audit 3 nền tảng (sửa lỗi article_an phá chữ Việt, đồng bộ default). Bản 0.2.39 làm panel trợ lý **bấm được bằng chuột** (nút Sửa tất cả, click từ gạch chân). Bản 0.2.38 **bật trợ lý ngữ pháp mặc định** (Chỉ gợi ý) với panel tại caret kiểu Grammarly + toast khay chỉ khi tự sửa. Bản 0.2.37 thêm
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

Engine bằng Rust, test bằng corpus replay (hơn 230 kịch bản `*.keys`) qua 4 mô phỏng
adapter (headless, tsf, mac, linux) trên 3 hệ điều hành ở mỗi commit; quy tắc xung đột
ngôn ngữ công khai tại
[docs/specs/language-detection.md](docs/specs/language-detection.md).

## Trạng thái

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | ✅ Release candidate — gõ thật qua TSF được CI kiểm tra trên Windows (bộ cài, portable, MSIX cài thử). Còn thiếu duy nhất chữ ký Authenticode (chờ SignPath Foundation duyệt). **Đã phát hành trên [Microsoft Store](https://apps.microsoft.com/store/detail/9NV2R7JNNFGK?cid=DevShareMCLPCB)** (gói MSIX, từ 2026-10-11). |
| macOS 13+ Apple Silicon / Intel | ✅ Release candidate — bộ gõ IMK + app menu bar + `.pkg` per-user (không sudo); CI build 2 kiến trúc (arm64 + x86_64), chạy `swift test` trên arm64. Chưa ký Developer ID/notarize (bản phát hành ký ad-hoc, `.pkg` chưa ký) → Gatekeeper cảnh báo lần mở đầu ([signing-status-mac.md](docs/release/signing-status-mac.md)). Module CGEventTap (`adapters/macos-tap`) có mã + test nhưng **không** có trong bản phát hành. |
| Linux IBus / Fcitx5 | ✅ Release candidate — tarball `install.sh` (per-user hoặc `--system`) + chạy ngay, Settings GTK4; CI kiểm tra với ibus-daemon và fcitx5 thật. Chưa có `.deb`/`.rpm`/AUR. |

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

## Download & install

Latest: **[GitHub Release v0.2.41](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.41)**
(release candidate, built and tested by CI). **Every release file carries a GPG
signature (`.asc`) and a Sigstore signature (`.cosign.sig` + `.cosign.cert`)**;
`SHA256SUMS.txt` is clear-signed — how to verify: [signing.md](docs/release/signing.md).
Not yet available: Authenticode signing for Windows (waiting on SignPath
Foundation — SmartScreen may warn, antivirus may mis-quarantine: seen with
Kaspersky, see
[antivirus-false-positive.md](docs/specs/antivirus-false-positive.md)) and
Apple Developer ID / notarization for macOS (Gatekeeper warns on first open).
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
