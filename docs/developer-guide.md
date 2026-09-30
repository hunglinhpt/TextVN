# Hướng dẫn phát triển TextVN

Tài liệu cho người sửa code, đóng gói và phát hành. Người dùng cuối xem
[user-guide.md](user-guide.md). Quy trình đóng góp (branch, commit, review):
[CONTRIBUTING.md](../CONTRIBUTING.md).

---

## 1. Kiến trúc

```
                ┌──────────────── core (Rust, không phụ thuộc OS) ────────────────┐
 phím ──► adapter ──► ffi (C ABI P0-2: ime_key → PASS/REPLACE/COMMIT/RESTORE) ──► engine
                │        method/ (Telex, VNI, VIQR, Telex đơn giản — bảng từ data/tables)
                │        transform/ (vị trí dấu, bảng mã xuất), post/ (gõ tắt, Quick Telex,
                │        viết hoa, khôi phục tiếng Anh), validate (âm tiết hợp lệ)
                ▼
   Windows: adapters/windows-tsf (textvn-tsf.dll, TSF TIP) ◄─IPC─► tray (TextVN.exe)
   Linux:   adapters/linux-ibus (engine IBus) · adapters/linux-fcitx5 (addon .so)
            adapters/linux-common (mô hình preedit, state.json, config sync)
            adapters/linux-settings (textvn-settings, GTK4)
```

| Crate / thư mục | Vai trò |
|---|---|
| `core` | Engine: fold chuỗi phím → chữ, quyết định action. Không I/O. |
| `config` | Parse/validate `config.v1`; `SettingsDoc` (vá từng khoá, ghi nguyên tử); `macro_text` (định dạng bảng gõ tắt). |
| `ffi` | C ABI engine (`ffi/include/textvn_ffi.h`, bất biến P0-2) + C API cấu hình (`textvn_settings.h`). |
| `strategy`, `appdb`, `field-detect` | Chọn chiến lược theo ứng dụng/ô nhập, cổng bảo mật ô mật khẩu. |
| `adapters/macos-imk` | **IMK adapter** (Swift): `TextVN-IM.app` (IMKServer + `TextVNInputController`), marked text lifecycle, commit-before-hide, SelectionReplace, CGEventTap fallback. |
| `adapters/macos-tap` | **CGEventTap fallback** (Swift): opt-in per-app, loop-guard marker, self-disable khi chậm. |
| `adapters/macos-app` | **Menu Bar App** (Swift/AppKit): cài đặt, V/E indicator, per-app toggle. |
| `adapters/windows-tsf` | TIP: key sink + key trace (`key_event.rs`), edit session, composition (`compose.rs` là mô hình thuần, có test), trả phím cho app CUAS (`replay.rs`), nhật ký chẩn đoán (`trace.rs`). |
| `tray` | Khay hệ thống, IPC server, bảng điều khiển Win32 (`settings_dialog.rs`), `svc.rs` (config/state), phím tắt Ctrl+Shift của Windows (`hotkey.rs`). |
| `adapters/windows-hook` | Hook tương thích (chỉ gói Compatibility, opt-in). |
| `adapters/linux-*` | IBus/Fcitx5/Settings (C/C++, CMake), link tĩnh `libtextvn_ffi.a`. |
| `cli` | `textvn-cli`: `register`, `doctor`, `replay`, `verify`, `config`. |
| `corpus/` | Kịch bản gõ `.keys` (P0-4) chạy qua 5 mô phỏng adapter. |

Mô hình gõ chung cho TSF, IBus, Fcitx5: **cả từ nằm trong composition/preedit**, mọi
`delete_count` của engine rơi trong đó, chốt ở ranh giới từ (Space, dấu câu, Enter, phím
điều hướng, chord). Không bao giờ xoá lùi chữ app đã nhận → không lỗi lặp/mất chữ kiểu
"không gạch chân" ở Chromium/Electron (xem `docs/specs/reference-parity.md` R1).

Bảng điều khiển thống nhất: [release/ui-spec.md](release/ui-spec.md).

## 2. Dựng

### Windows

Cần Rust stable + target `x86_64-pc-windows-msvc`, VS 2022 Build Tools (C++).

```powershell
cargo build --workspace
powershell -File .\build-release.ps1                    # kiểm tra + build + zip portable
powershell -File .\build-release.ps1 -BuildInstaller    # + Inno Setup 6.5+ (iscc trong PATH)
```

Gói mặc định **TSF-only**; `-IncludeCompatibilityHook` thêm hook legacy.
`-SigningCertificateThumbprint <SHA1>` ký Authenticode mọi PE. Trên Linux có thể kiểm tra
code Windows: `cargo clippy --target x86_64-pc-windows-msvc --workspace --all-targets`.

### macOS (13+)

Cần Xcode CLT 15+ (`xcode-select --install`) và rustup với 2 Mac targets:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

**Dựng toàn bộ (engine Rust universal + swift build + assemble .app):**

```bash
cd adapters/macos-imk
./build-rust.sh                   # debug — TextVN-IM.app trong adapters/macos-imk/build/
./build-rust.sh --release         # release
./build-rust.sh --lib-only        # chỉ build libtextvn_ffi.a (cho swift test nhanh)
```

Script `build-rust.sh`:
1. `cargo build` × 2 arch (aarch64 + x86_64) từ workspace root
2. `lipo -create` → `lib/libtextvn_ffi.a` universal
3. `swift build` (SwiftPM đọc `-L lib -ltextvn_ffi` từ `Package.swift`)
4. Assemble `TextVN-IM.app` bundle

**Chạy swift test đơn lẻ:**

```bash
cd adapters/macos-imk
./build-rust.sh --lib-only        # bước 1&2 trước
swift test --arch arm64           # arm64
swift test --arch x86_64          # x86_64
```

**Vấn đề đã biết (MAC-031)**: `Package.swift` dùng `String.components(separatedBy:)`
thay `URL` vì `Foundation` không khả dụng trong `PackageDescription` scope trên Xcode 26.6+.

### Linux (Ubuntu/Debian)

```bash
sudo apt install cmake pkg-config g++ libibus-1.0-dev libfcitx5core-dev libfcitx5utils-dev \
    fcitx5-modules-dev libgtk-4-dev ibus fcitx5 python3-dbus python3-gi
scripts/e2e-linux.sh            # build adapter + ctest + e2e IBus/Fcitx5 thật
scripts/install_linux.sh        # build từ nguồn rồi cài per-user (install.sh của gói)
scripts/build-linux.sh          # dist/TextVN-<ver>-linux-<arch>.tar.gz (+ .sha256)
```

## 3. Kiểm thử

| Lớp | Lệnh | Bắt được |
|---|---|---|
| Unit | `cargo test --workspace` | engine, config, ffi, tray, TSF compose |
| Từ vựng thật | `cargo test -p textvn-core --test common_words` | ~400 từ × Telex/VNI/kiểu cũ/`uow`/dấu giữa từ/Caps Lock — lỗi đặt dấu, `d`/`đ`, gi/qu |
| Corpus | `cargo run -p textvn-cli -- replay corpus --adapter <headless\|win\|tsf\|mac\|linux>` | hành vi phím theo từng adapter (composition TSF, backspace model…) |
| ABI | `cargo run -p textvn-cli -- verify` | header C ↔ Rust |
| Linux e2e | `scripts/e2e-linux.sh` | ibus-daemon + fcitx5 thật: gõ, focus-out, reset, Ctrl+Shift, state.json, gõ tắt, Caps Lock, ô mật khẩu |
| Gói Linux | `scripts/test-linux-package.sh dist/TextVN-*.tar.gz` | cài → gõ → gỡ sạch; portable (thư mục chỉ đọc) → gõ → stop; × IBus + Fcitx5 |
| Gói Windows | CI job *Windows package* (`installer/windows/tests/*.ps1`) | build-release, zip portable và Inno Setup: đăng ký TSF, gõ thật bằng SendInput vào Notepad (IMM32/CUAS) và WordPad (TSF-aware), gỡ sạch |
| Bảng điều khiển Linux | `ctest` trong `target/linux-adapters/settings` | `settings_model` (vá khoá, gõ tắt, file hỏng) |

Thêm case corpus: `corpus/shared/<chủ đề>_NN.keys` (cú pháp: `docs/10-shared/P0-4-test-and-corpus.md`,
`:mods +CapsLock` cho Caps Lock, `:enabled off` cho chế độ E). Mỗi thay đổi hành vi gõ
phải có case corpus chạy qua **cả 5 adapter**.

Chẩn đoán key sink TSF: đặt `TEXTVN_TSF_TRACE=<file>` cho process của app (ví dụ chạy
`set TEXTVN_TSF_TRACE=%TEMP%\tsf.log && notepad`) — ghi pha `test`/`down`/`up`, kết quả
eaten, lần chuyển V/E; phím sinh ký tự chỉ ghi `chr` (không lộ nội dung gõ). Test gõ Windows
bật sẵn và in ra khi có case lỗi.

Chạy thử bảng điều khiển GTK không cần màn hình: `gtk4-broadwayd :5` rồi
`GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 textvn-settings`, mở `http://127.0.0.1:8085`.

## 4. Quy ước bắt buộc (CI chặn)

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`
  (cả target Windows).
- `.ps1` chỉ ASCII (chuỗi tiếng Việt trong script: `[regex]::Unescape('đ...')`).
- Không dùng API tiêm mã/hook toàn cục ngoài gói Compatibility
  (`.github/scripts/check_no_injection_apis.py`, lý do: `docs/specs/antivirus-false-positive.md`).
- REUSE: mọi file có SPDX (tự động qua `REUSE.toml`); file bên thứ ba khai báo riêng
  (ví dụ `installer/windows/languages/Vietnamese.isl` — `LicenseRef-InnoSetup`).
- Log không bao giờ chứa nội dung phím gõ (S2).
- Header C chép tay: sửa `ffi/src/lib.rs` phải sửa `textvn_ffi.h` (kiểm bằng `verify`);
  `settings.rs` ↔ `textvn_settings.h` kiểm bằng test `header_matches_exports`.
- Bảng phím là dữ liệu: sửa `data/tables/*.toml` rồi `cargo run -p xtask -- gen-tables`
  (CI: `check-tables`).

## 5. Phát hành

1. Cập nhật `version` trong `Cargo.toml` (workspace), mọi manifest/plist/fallback
   Swift, rồi chạy `cargo run -q -p xtask -- check-version-sync`.
2. `CHANGELOG.md`, [build-release-report.md](release/build-release-report.md) (điền số liệu
   kiểm thử thật của lần build).
3. Windows: `build-release.ps1 -BuildInstaller -SigningCertificateThumbprint …` trên máy có
   chứng chỉ; `RELEASE_REPORT.json` trong zip ghi trạng thái (`release-candidate` cho tới khi
   hết `production_blockers`).
4. Linux: `scripts/build-linux.sh` trên Ubuntu 24.04 (glibc của máy dựng là mức tối thiểu).
5. macOS: `scripts/build-macos.sh` và `scripts/package-macos-pkg.sh` trên runner Mac;
   chỉ ký/notarize khi có Developer ID thật và đã smoke GUI trên Mac.
6. Chỉ sau khi `ci-shared`, `ci-macos`, `repo-hygiene` xanh trên **cùng commit**,
   tạo và push tag `v<version>`. `.github/workflows/release.yml` tự build lại
   từ tag, test gói ba nền tảng, kiểm `RELEASE_REPORT.json`, tính SHA-256 rồi
   đính kèm ZIP Windows, setup `.exe`, Linux tarball, macOS ZIP/tar/PKG và
   `SHA256SUMS.txt` vào GitHub Release dạng pre-release. Nếu workflow lỗi,
   sửa nguyên nhân và phát hành phiên bản mới; không ghi đè binary của tag cũ.
   Chỉ bỏ nhãn pre-release sau khi đã có ký số, notarization và smoke GUI native;
   không đổi nhãn thành production bằng cách chỉ sửa metadata GitHub.

## 6. Tài liệu liên quan

- Đặc tả: `docs/10-shared/` (ABI, config, corpus), `docs/20-windows/`, `docs/30-macos/`, `docs/40-linux/`
- macOS: [`docs/30-macos/P2-0-MASTER-PLAN.md`](30-macos/P2-0-MASTER-PLAN.md), [`docs/30-macos/IMPLEMENTATION-STATUS.md`](30-macos/IMPLEMENTATION-STATUS.md), [`docs/30-macos/P2-REVIEW-LOG.md`](30-macos/P2-REVIEW-LOG.md)
- Đối chiếu bộ gõ tham chiếu & bug đã biết: [specs/reference-parity.md](specs/reference-parity.md)
- TSF: [specs/tsf-typing-overhaul.md](specs/tsf-typing-overhaul.md)
- Antivirus: [specs/antivirus-false-positive.md](specs/antivirus-false-positive.md)

