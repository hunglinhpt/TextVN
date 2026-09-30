# Changelog

Tất cả thay đổi đáng chú ý của dự án TextVN sẽ được ghi lại ở đây.

Định dạng dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
và dự án này tuân thủ [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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

[Unreleased]: https://github.com/hunglinhpt/TextVN/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/hunglinhpt/TextVN/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/hunglinhpt/TextVN/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/hunglinhpt/TextVN/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hunglinhpt/TextVN/releases/tag/v0.1.0
