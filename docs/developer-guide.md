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
   macOS:   adapters/macos-imk (TextVN-IM.app, IMK) ◄─IPC─► adapters/macos-app (TextVN.app, menu bar)
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
| `adapters/macos-imk` | **IMK adapter** (Swift): `TextVN-IM.app` (IMKServer + `TextVNInputController`), marked text lifecycle, commit-before-hide, SelectionReplace. |
| `adapters/macos-tap` | **CGEventTap opt-in** (Swift package: loop-guard marker, self-disable khi chậm) — có test trong CI nhưng **chưa nhúng vào bản phát hành** (`scripts/build-macos.sh` chỉ build `macos-imk` + `macos-app`), chưa có công tắc trong Cài đặt. |
| `adapters/macos-app` | **Menu Bar App** (Swift/AppKit): cài đặt, V/E indicator, per-app toggle. |
| `adapters/windows-tsf` | TIP: key sink + key trace (`key_event.rs`), edit session, composition (`compose.rs` là mô hình thuần, có test), trả phím cho app CUAS (`replay.rs`), nhật ký chẩn đoán (`trace.rs`). |
| `tray` | Khay hệ thống, IPC server, bảng điều khiển Win32 (`settings_dialog.rs`), `svc.rs` (config/state), phím tắt Ctrl+Shift của Windows (`hotkey.rs`). |
| `adapters/windows-hook` | Hook tương thích (chỉ gói Compatibility, opt-in). |
| `adapters/linux-*` | IBus/Fcitx5/Settings (C/C++, CMake), link tĩnh `libtextvn_ffi.a`. |
| `cli` | `textvn-cli` (Linux cài thành `textvn`): `replay`, `verify`, `sizes`, `config default\|init\|validate`, `doctor [--json] [--pause] [--export]`, `register`/`unregister [--scope user\|machine]`, `activate`, `schedule-delete`. |
| `corpus/` | Kịch bản gõ `.keys` (P0-4) chạy qua 5 mô phỏng adapter. |

Mô hình gõ chung cho TSF, IBus, Fcitx5 và macOS IMK: **cả từ nằm trong composition/preedit**, mọi
`delete_count` của engine rơi trong đó, chốt ở ranh giới từ (Space, dấu câu, Enter, phím
điều hướng, chord). Không bao giờ xoá lùi chữ app đã nhận → không lỗi lặp/mất chữ kiểu
"không gạch chân" ở Chromium/Electron (xem `docs/specs/reference-parity.md` R1).

Đặc biệt, giao diện hiển thị composition được tinh chỉnh hoàn toàn không gạch chân (clean composition):
- **Windows TSF**: Cung cấp `ITfDisplayAttributeProvider` với thuộc tính `lsStyle = TF_LS_NONE` (`DISPATTR_TEXTVN`). Hỗ trợ đăng ký theo user profile (`HKCU`) không bắt buộc quyền Administrator.
- **Linux IBus / Fcitx5**: Tắt cờ `IBUS_ATTR_TYPE_UNDERLINE` và dùng `fcitx::TextFormatFlag::NoFlag`.
- **macOS IMK**: Cấu hình `NSAttributedString` trong marked text với `.underlineStyle = []`.

Bảng điều khiển thống nhất: [release/ui-spec.md](release/ui-spec.md).

## 2. Dựng

### Windows

Cần Rust stable + target `x86_64-pc-windows-msvc` và `i686-pc-windows-msvc` (DLL x86 cho app
32-bit — `build-release.ps1` tự `rustup target add`), VS 2022 Build Tools (C++).

```powershell
cargo build --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1                    # kiểm tra + build + zip portable
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller    # + Inno Setup 6.5+ (bộ cài per-user)
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller   # + bản -machine.exe
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1               # gói MSIX (cần Windows SDK)
```

Gói mặc định **TSF-only**; `-IncludeCompatibilityHook` thêm hook legacy. Mọi binary Windows
link CRT tĩnh (`.cargo/config.toml` `+crt-static` — không cần VC++ redist); `build-release.ps1`
kiểm bảng import PE bằng `tools/win/check-pe-imports.py` và fail nếu còn phụ thuộc runtime.
Ký Authenticode: `-SigningCertificateThumbprint <SHA1>` (cert local) hoặc SignPath khi có
`SIGNPATH_API_TOKEN` (`docs/release/code-signing-plan.md`). Trên Linux có thể kiểm tra code
Windows: `cargo clippy --target x86_64-pc-windows-msvc --workspace --all-targets` (CI chạy đúng
lệnh này, gồm cả hook).

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
swift test --arch x86_64          # x86_64 — chỉ trên máy Intel/Rosetta; runner arm64 của CI chỉ
                                  # build-tests x86_64, không chạy được bundle test (MAC-032)
```

Đóng gói phát hành (`.pkg` per-user, ZIP/tar.gz universal) và ký Developer ID/notarize có
điều kiện: `scripts/build-macos.sh`, `scripts/package-macos-pkg.sh`, `scripts/notarize-macos.sh`
— biến môi trường và trạng thái: [release/signing-status-mac.md](release/signing-status-mac.md).

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
| Gói Windows | CI job *Windows package* (`installer/windows/tests/*.ps1`) | build-release, zip portable và Inno Setup (per-user + `/ALLUSERS` + bản `-machine.exe`): đăng ký TSF, gõ thật bằng SendInput vào Notepad (IMM32/CUAS) và WordPad (TSF-aware), gỡ sạch; Store validation (silent/ARP/bundleware + giả lập validator) |
| Gói MSIX | `tools/win/build-msix.ps1` → `tools/win/verify-msix.py` → `installer/windows/tests/test-msix-sideload.ps1` | luật manifest của Partner Center; cài gói ký tạm, mở app, kiểm từ ngoài gói (stage-out, TIP HKCU, Run), gỡ gói → guard dọn sạch |
| C sanitizer | job `linux-common tests (ASan + UBSan)` | lỗi bộ nhớ/UB trong IPC client, compose, utf của `linux-common` |
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

- `ci-shared` (mỗi PR + push `main` không chỉ đổi docs): fmt; clippy Linux; clippy cross-check
  target Windows (gồm hook); `cargo test` trên 3 OS; replay corpus × 4 adapter (headless,
  tsf, mac, linux) trên 3 OS; `textvn-cli verify` + `sizes`; xtask `check-tables`,
  `check-mac-targets`, `check-version-sync`, `check-win-corpus`, `check-mac-corpus`;
  `linux-common` với ASan + UBSan; e2e IBus/Fcitx5 + gói Linux; fuzz smoke 60 s/target;
  `cargo-deny`; REUSE; perf A/B so merge-base (`continue-on-error`); job *Windows package*
  (portable + bộ cài gõ thật, Store validation, MSIX build + `verify-msix` + sideload).
  `ci-macos`: lint script/plist/header, staticlib universal, `swift build` 2 arch + `swift test`
  arm64, corpus mac, ZIP/PKG candidate. `repo-hygiene`: 10 check sạch repo.
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

> **Quy trình đầy đủ bắt buộc (checklist local → repo/docs, bộ tài liệu không
> được thiếu): [release/release-process.md](release/release-process.md)** —
> quy tắc G16 trong `docs/00-WORKFLOW.md`. Mục dưới đây chỉ là tóm tắt kỹ thuật.

1. Cập nhật `version` trong `Cargo.toml` (workspace), mọi manifest/plist/fallback
   Swift, rồi chạy `cargo run -q -p xtask -- check-version-sync`.
2. `CHANGELOG.md`, [build-release-report.md](release/build-release-report.md) (điền số liệu
   kiểm thử thật của lần build).
3. Windows: CI (`release.yml`, job `windows`) chạy `build-release.ps1 -BuildInstaller` rồi
   dựng bản `-machine.exe` và gói MSIX (build → `verify-msix.py` → cài thử sideload). Khi có
   secret `SIGNPATH_*`, mọi PE và cả hai bộ cài được ký Authenticode qua
   `tools/win/sign-signpath.ps1` (SignPath Foundation — **đang chờ duyệt**);
   `-SigningCertificateThumbprint` chỉ dành cho máy có chứng chỉ riêng. `RELEASE_REPORT.json`
   trong zip ghi `checks.authenticode` và trạng thái `release-candidate`. Gói MSIX nộp Store
   cần repo variable `MSIX_IDENTITY_NAME` ([release/msix-submission.md](release/msix-submission.md)).
4. Linux: `scripts/build-linux.sh` trên Ubuntu 24.04 (glibc của máy dựng là mức tối thiểu).
5. macOS: `scripts/build-macos.sh` và `scripts/package-macos-pkg.sh` trên runner Mac; ký
   Developer ID + notarize + staple tự chạy khi có secret Apple (chưa cấu hình — bản phát hành
   ký ad-hoc, `.pkg` chưa ký: [release/signing-status-mac.md](release/signing-status-mac.md)).
6. Chỉ sau khi `ci-shared`, `ci-macos`, `repo-hygiene` xanh trên **cùng commit**,
   tạo và push tag `v<version>`. `.github/workflows/release.yml` tự build lại
   từ tag, test gói ba nền tảng, kiểm `RELEASE_REPORT.json`, rồi đính kèm vào GitHub Release
   dạng pre-release: ZIP portable, `TextVN-setup-*-windows-x64.exe` + `-machine.exe`,
   `TextVN-*-windows-x64.msix`, Linux tarball, macOS ZIP/tar.gz/PKG và `gpg-release-key.asc`.
   Mỗi asset có chữ ký GPG `.asc` + Sigstore `.cosign.sig`/`.cosign.cert`, `SHA256SUMS.txt`
   được clearsign (`tools/release/sign-artifacts.sh` — bắt buộc trong CI, tự verify;
   [release/signing.md](release/signing.md)). Nếu workflow lỗi, sửa nguyên nhân và phát hành
   phiên bản mới; không ghi đè binary của tag cũ. Chỉ bỏ nhãn pre-release khi có Authenticode
   (Windows) và Developer ID + notarization (macOS); không đổi nhãn thành production bằng cách
   chỉ sửa metadata GitHub.

## 6. Tài liệu liên quan

- Đặc tả: `docs/10-shared/` (ABI, config, corpus), `docs/20-windows/`, `docs/30-macos/`, `docs/40-linux/`
- macOS: [`docs/30-macos/P2-0-MASTER-PLAN.md`](30-macos/P2-0-MASTER-PLAN.md), [`docs/30-macos/IMPLEMENTATION-STATUS.md`](30-macos/IMPLEMENTATION-STATUS.md), [`docs/30-macos/P2-REVIEW-LOG.md`](30-macos/P2-REVIEW-LOG.md)
- Đối chiếu bộ gõ tham chiếu & bug đã biết: [specs/reference-parity.md](specs/reference-parity.md)
- TSF: [specs/tsf-typing-overhaul.md](specs/tsf-typing-overhaul.md)
- Antivirus: [specs/antivirus-false-positive.md](specs/antivirus-false-positive.md)

