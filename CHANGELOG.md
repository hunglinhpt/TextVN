# Changelog

Tất cả thay đổi đáng chú ý của dự án TextVN sẽ được ghi lại ở đây.

Định dạng dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
và dự án này tuân thủ [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- **Tray Icon**: Icon TextVN 16/32/48px nhúng qua winresource, manifest DPI PerMonitorV2
- **CLI**: `textvn-cli register` / `textvn-cli unregister` — đăng ký/hủy TSF TIP per-user (WIN-003, WIN-010)
- **CLI**: `textvn-cli register status` — kiểm tra trạng thái đăng ký TSF
- **Windows manifest**: asInvoker (không cần admin), Windows 10/11 compatibility

### Fixed
- **Windows TSF không gõ được tiếng Việt** (chi tiết `docs/specs/tsf-typing-overhaul.md`, Ftsf-1…12):
  security gate không bao giờ mở; đăng ký per-user dừng trước `InstallLayoutOrTip`; composition
  mở mới mỗi phím, không commit; app treo khi Deactivate (join thread IPC); tắt tiếng Việt từ khay
  không có hiệu lực; hotkey Ctrl+Shift+Space đảo 2 lần; Delete/F-key/mũi tên bị coi là chữ;
  không gõ được ở Start search/Settings/app Store.
- TSF dùng mô hình composition (cả từ trong một composition, commit tại ranh giới) — chạy cả
  app TSF-aware lẫn IMM32/CUAS; `replay --adapter tsf` kiểm toàn bộ corpus qua mô hình này.
- Giảm heuristic AV: bỏ `TerminateProcess` khỏi `--stop`, bỏ `textvn_ffi.dll` khỏi gói Windows,
  `input.dll` chỉ nạp từ System32, pipe từ chối client từ xa.
- `cli/src/register.rs` bị empty do overwrite — viết lại hoàn toàn với full TSF registration flow
- Unsafe COM blocks bọc trong closure để dùng `?` operator đúng cách

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
