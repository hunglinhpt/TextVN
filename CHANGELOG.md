# Changelog

Tất cả thay đổi đáng chú ý của dự án TextVN sẽ được ghi lại ở đây.

Định dạng dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
và dự án này tuân thủ [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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

[Unreleased]: https://github.com/hunglinhpt/TextVN/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hunglinhpt/TextVN/releases/tag/v0.1.0
