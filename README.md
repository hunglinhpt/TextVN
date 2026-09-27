# VietIME — Bộ gõ Tiếng Việt cho Windows (Text Services Framework)

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-Windows%20x64-brightgreen)](docs/)

> **VietIME** là bộ gõ tiếng Việt mã nguồn mở, hiệu năng cao, tích hợp sâu vào Windows qua Text Services Framework (TSF). Hỗ trợ Telex, VNI, VIQR và Simple Telex với latency < 5ms.

---

## Tính năng

| Tính năng | Trạng thái |
|-----------|-----------|
| Telex / VNI / VIQR / Simple Telex | ✅ |
| Kiểu bỏ dấu: Chuẩn mới / Cổ điển | ✅ |
| Tray icon (minimize to system tray) | ✅ |
| Per-app enable/disable | ✅ |
| Named Pipe IPC (TSF ↔ Tray) | ✅ |
| Auto-start cùng Windows | ✅ |
| Không cần quyền Admin (per-user install) | ✅ |
| `vietime doctor` — chẩn đoán môi trường | ✅ |
| `vietime register/unregister` — cài TSF | ✅ |
| Inno Setup installer | ✅ |
| Emoji shortcut | 🔄 Đang phát triển |
| MacOS / Linux | 🔄 Đang phát triển |

---

## Yêu cầu hệ thống

- **Windows 10** (build 1903+) hoặc **Windows 11**
- Kiến trúc: `x86_64` (64-bit)
- Không yêu cầu quyền Administrator

---

## Cài đặt nhanh (cho người dùng)

### Cách 1 — Installer (khuyến nghị)

1. Tải file `vietime-setup-X.Y.Z.exe` từ [Releases](https://github.com/hunglinhpt/TextVN/releases)
2. Chạy installer — **không cần Admin**
3. VietIME tự động khởi động cùng Windows
4. Nhấn `Win+Space` để chọn VietIME

### Cách 2 — Portable (không cài đặt)

```powershell
# Giải nén vietime-portable-X.Y.Z.zip
# Chạy trong PowerShell:
.\vietime-tray.exe          # Khởi động tray
.\vietime.exe register      # Đăng ký TSF (chỉ cần 1 lần)
```

---

## Hướng dẫn sử dụng

### Chuyển đổi chế độ gõ

- **Chuột phải vào icon tray** → chọn "Chế độ gõ"
- Hoặc dùng phím tắt `Win+Space` để toggle VietIME / bàn phím hệ thống

### Tắt/Bật tiếng Việt

- Click vào icon tray để toggle toàn cục
- Hoặc chuột phải → "Bật gõ tiếng Việt"

### Tắt cho từng ứng dụng

- Mở ứng dụng cần tắt (ví dụ: một game)
- Chuột phải tray → "Bật tiếng Việt cho [tên app]" → bỏ check

### CLI

```powershell
vietime --help                    # Danh sách lệnh
vietime doctor                    # Chẩn đoán môi trường
vietime doctor --export diag.zip  # Xuất báo cáo chẩn đoán
vietime register                  # Đăng ký TSF TIP (nếu cần)
vietime unregister                # Hủy đăng ký
vietime register status           # Kiểm tra trạng thái
vietime config default            # Xem config mặc định
vietime config init               # Tạo config lần đầu
vietime config validate cfg.json  # Kiểm tra file config
```

---

## Build từ source

### Yêu cầu

- [Rust](https://rustup.rs/) stable (1.78+) với target `x86_64-pc-windows-msvc`
- Visual Studio 2022 (Build Tools) với C++ workload
- [LLVM/Clang](https://releases.llvm.org/) (cho bindgen)

### Cài đặt Rust target

```powershell
rustup target add x86_64-pc-windows-msvc
```

### Build debug (phát triển)

```powershell
cargo build --workspace
```

### Build release (production)

```powershell
cargo build --release --workspace --target x86_64-pc-windows-msvc
```

Binary output trong `target/x86_64-pc-windows-msvc/release/`:
- `vietime.exe` — CLI
- `vietime-tray.exe` — Tray app
- `vietime-tsf.dll` — TSF TIP (đăng ký với Windows)

### Build với icon nhúng

```powershell
cargo build --release -p vietime-tray --features embed-resources
```

### Build installer (Inno Setup)

```powershell
# Cài Inno Setup 6: https://jrsoftware.org/isdl.php
iscc installer\windows\vietime-setup.iss
```

### Chạy tests

```powershell
cargo test --workspace                        # Tất cả tests
cargo test -p vietime-cli                     # Chỉ CLI
cargo test -p vietime-tray                    # Chỉ Tray
cargo clippy --workspace                      # Lint
```

---

## Cấu trúc dự án

```
TextVN/
├── cli/                    # vietime CLI (register, doctor, replay, config)
├── tray/                   # vietime-tray (system tray, IPC server, menu)
├── adapters/
│   ├── windows-tsf/        # vietime-tsf.dll (TSF Text Input Processor)
│   └── windows-hook/       # vietime-hook.exe (WH_KEYBOARD_LL hook)
├── engine/                 # Core IME engine (platform-agnostic)
├── config/                 # Config schema + parser
├── ffi/                    # C ABI headers
├── ipc/                    # IPC protocol (Named Pipe messages)
├── strategy/               # Telex/VNI/VIQR transformation strategies
├── installer/windows/      # Inno Setup 6 script
├── docs/specs/             # Spec documents (TSF, hooks, etc.)
└── spikes/                 # Research spikes
```

---

## File cấu hình

Mặc định: `%APPDATA%\VietIME\config.json`

```json
{
  "config_version": 1,
  "method": "telex",
  "diacritic_style": "new",
  "global_enabled": true,
  "app_overrides": {}
}
```

---

## Gỡ cài đặt

### Qua installer:
- Windows Settings → Apps → VietIME → Uninstall
- Hoặc chạy lại `vietime-setup.exe` → Uninstall

### Thủ công:
```powershell
vietime-tray.exe --stop          # Dừng tray
vietime.exe unregister           # Hủy đăng ký TSF
# Xóa thư mục cài đặt
```

> **Lưu ý**: Cấu hình người dùng (`%APPDATA%\VietIME\`) **không bị xóa** khi gỡ cài đặt (theo [S9](SECURITY.md)).

---

## Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) để biết cách đóng góp.

## License

[GPL-3.0-or-later](LICENSE) © 2024 VietIME Contributors
