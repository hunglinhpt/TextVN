# TextVN — Bộ gõ Tiếng Việt cho Windows (Text Services Framework)

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-Windows%20x64-brightgreen)](docs/)

> **TextVN** là bộ gõ tiếng Việt mã nguồn mở cho Windows. Dự án cung cấp Telex, VNI, VIQR, Simple Telex, khay hệ thống và cấu hình theo ứng dụng. Hiện vẫn đang trong giai đoạn phát triển; xem phần “Trạng thái phát hành” trước khi dùng làm bộ gõ chính.

---

## Tính năng

| Tính năng | Trạng thái |
|-----------|-----------|
| Telex / VNI / VIQR / Simple Telex (engine + corpus) | ✅ |
| Kiểu bỏ dấu: Chuẩn mới / Cổ điển | ✅ |
| Tray icon (minimize to system tray) | ✅ |
| Per-app enable/disable | ✅ |
| Named Pipe IPC (TextVN components) | ✅ |
| Auto-start cùng Windows | ✅ |
| Không cần quyền Admin (per-user install) | ✅ |
| `textvn-cli doctor` — chẩn đoán môi trường | ✅ |
| `textvn-cli register/unregister` — đăng ký TSF per-user | 🧪 |
| Inno Setup installer | 🧪 |
| Emoji shortcut | 🔄 Đang phát triển |
| MacOS / Linux | 🔄 Đang phát triển |

## Trạng thái phát hành

TextVN chưa phải bản phát hành production. Engine và corpus chạy ổn định trong CI, nhưng UI Automation native, vòng đời composition TSF, DACL cho IPC và ký AppDB vẫn đang hoàn thiện. Không dùng bản build hiện tại để nhập mật khẩu hoặc dữ liệu quan trọng.

---

## Yêu cầu hệ thống

- **Windows 10** (build 1903+) hoặc **Windows 11**
- Kiến trúc: `x86_64` (64-bit)
- Không yêu cầu quyền Administrator

---

## Cài đặt nhanh (cho người dùng)

### Cách 1 — Installer (khuyến nghị)

1. Tải file `TextVN-setup-X.Y.Z-windows-x64.exe` từ [Releases](https://github.com/hunglinhpt/TextVN/releases), khi release có đánh dấu hỗ trợ.
2. Chạy installer — **không cần Admin**
3. TextVN tự động khởi động cùng Windows
4. Nhấn `Win+Space` để chọn TextVN sau khi đã đăng ký TSF.

### Cách 2 — Portable (không cài đặt)

```powershell
# Giải nén TextVN-portable-X.Y.Z.zip
# Chạy trong PowerShell:
.\TextVN.exe                # Khởi động tray
.\textvn-cli.exe register   # Đăng ký TSF (chỉ cần 1 lần)
```

---

## Hướng dẫn sử dụng

### Chuyển đổi chế độ gõ

- **Chuột phải vào icon tray** → chọn "Chế độ gõ"
- Hoặc dùng phím tắt `Win+Space` để toggle TextVN / bàn phím hệ thống

### Tắt/Bật tiếng Việt

- Click vào icon tray để toggle toàn cục
- Hoặc chuột phải → "Bật gõ tiếng Việt"

### Tắt cho từng ứng dụng

- Mở ứng dụng cần tắt (ví dụ: một game)
- Chuột phải tray → "Bật tiếng Việt cho [tên app]" → bỏ check

### CLI

```powershell
textvn-cli --help                    # Danh sách lệnh
textvn-cli doctor                    # Chẩn đoán môi trường
textvn-cli doctor --export diag.zip  # Xuất báo cáo chẩn đoán
textvn-cli register                  # Đăng ký TSF TIP (nếu cần)
textvn-cli unregister                # Hủy đăng ký
textvn-cli register status           # Kiểm tra trạng thái
textvn-cli config default            # Xem config mặc định
textvn-cli config init               # Tạo config lần đầu
textvn-cli config validate cfg.json  # Kiểm tra file config
```

---

## Build từ source

### Yêu cầu

- [Rust](https://rustup.rs/) stable (1.78+) với target `x86_64-pc-windows-msvc`
- Visual Studio 2022 (Build Tools) với C++ workload

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
# TSF-only mac dinh: khong build hay dong goi global Hook legacy
powershell -File .\build-release.ps1

# Chi dung khi can tuong thich game/app cu (global Hook legacy)
powershell -File .\build-release.ps1 -IncludeCompatibilityHook
```

Binary output trong `target/x86_64-pc-windows-msvc/release/`:
- `textvn-cli.exe` — CLI
- `TextVN.exe` — Tray app
- `textvn-tsf.dll` — TSF TIP (đăng ký với Windows)

Gói phát hành mặc định là **TSF-only**. `textvn-hook.exe` chỉ thuộc gói
Compatibility tùy chọn và chỉ chạy khi người dùng bật nó từ menu khay.

### Build installer (Inno Setup)

```powershell
# Cài Inno Setup 6: https://jrsoftware.org/isdl.php
iscc installer\windows\TextVN-setup.iss
```

### Chạy tests

```powershell
cargo test --workspace                        # Tất cả tests
cargo test -p textvn-cli                     # Chỉ CLI
cargo test -p textvn-tray                    # Chỉ Tray
cargo clippy --workspace                      # Lint
cargo run -p textvn-cli -- verify             # Kiểm tra ABI C/Rust
cargo run -p textvn-cli -- replay corpus/shared corpus/win --adapter win
```

---

## Cấu trúc dự án

```
TextVN/
├── cli/                    # TextVN CLI (register, doctor, replay, config)
├── tray/                   # TextVN tray (system tray, IPC server, menu)
├── adapters/
│   ├── windows-tsf/        # textvn-tsf.dll (TSF Text Input Processor)
│   └── windows-hook/       # Hook legacy (chi phat hanh trong goi Compatibility)
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

Mặc định: `%APPDATA%\TextVN\config.json`

```json
{
  "config_version": 1,
  "method": "telex",
  "diacritic_style": "new",
  "global_enabled": true,
  "app_overrides": {}
}
```

## Bảng điều khiển

Mở **Bảng điều khiển** từ menu chuột phải của icon TextVN. Tất cả tùy chọn đều ghi vào cấu hình hiện hành và phát thông báo reload tới các component đang kết nối.

- **Bảng mã:** Unicode dựng sẵn, Unicode tổ hợp, TCVN3 hoặc VNI Windows.
- **Kiểu gõ:** Telex, VNI, VIQR hoặc Simple Telex.
- **Tùy chọn gõ:** đặt dấu tự do, khôi phục từ tiếng Anh, dấu mới/cũ và bật/tắt tiếng Việt toàn cục.
- **Hệ thống:** khởi động cùng Windows, khôi phục mặc định, đóng về khay hoặc kết thúc ứng dụng.
- **Hướng dẫn / Thông tin:** mô tả nhanh, giấy phép GPL và tác giả `hunglinhpt`.

---

## Gỡ cài đặt

### Qua installer:
- Windows Settings → Apps → TextVN → Uninstall
- Hoặc chạy lại `TextVN-setup.exe` → Uninstall

### Thủ công:
```powershell
TextVN.exe --stop                # Dừng tray
textvn-cli.exe unregister        # Hủy đăng ký TSF
# Xóa thư mục cài đặt
```

> **Lưu ý**: Cấu hình người dùng (`%APPDATA%\TextVN\`) **không bị xóa** khi gỡ cài đặt (theo [S9](SECURITY.md)).

---

## Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) để biết cách đóng góp.

## License

[GPL-3.0-or-later](LICENSE) © 2026 hunglinhpt
