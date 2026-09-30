# Báo cáo dựng & kiểm thử — TextVN 0.1.0 (lịch sử)

> `v0.2.2` là bản mới nhất. Bằng chứng của nó nằm trong
> [audit 2026-09-30](cross-platform-audit-2026-09-30.md) “Vòng 11”.
>
> `v0.2.1` đã được phát hành dạng pre-release. Bằng chứng của bản đó nằm ở
> các phần lịch sử phía dưới. Không dùng số liệu 0.1.0 làm bằng chứng
> production cho 0.2.x.

## Bản 0.2.2 — vá F3-13 (mở nhầm Cài đặt lúc login) + F3-8, nit

**Mục tiêu bản này:** 0.2.1 đã cố vá F3-13 nhưng còn sót — với bản **chưa ký số**
đúng trạng thái release hiện tại, `SMAppService.mainApp.status` là
`.requiresApproval` dù login item vẫn chạy, nên TextVN hiểu là khởi động tay và
**bật cửa sổ Cài đặt mỗi lần đăng nhập**. Đây là lỗi được ưu tiên cao nhất và
đã sửa tận gốc. Đồng thời đóng nốt F3-8 (menu bar lệch spec) và nhóm nit.

### Kiểm chứng chạy trên host Windows (trước commit)

| Gate | Kết quả |
|---|---|
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warning |
| `cargo test --workspace` | ✅ **342 test pass** |
| `cargo run -q -p xtask -- check-version-sync` | ✅ 14 chỗ = 0.2.2 |
| `check-tables` · `check-mac-corpus` (114) · `check-win-corpus` (72) · `check-mac-targets` (12) | ✅ |
| `replay corpus/shared corpus/mac --adapter mac` | ✅ 149/149 |
| `replay corpus/win --adapter win` | ✅ 78/78 |
| `cargo run -q -p textvn-cli -- verify` | ✅ ABI v1 ↔ header: `ime_key_v1=20`, `ime_result_v1=532`, offsets + 11 export khớp |

### Perf — job `perf regression` đỏ trên CI là **nhiễu runner**, không phải hồi quy

Job cố ý để `continue-on-error` (runner Windows GHA dùng chung, 2 vCPU; gate cứng
chạy trên self-hosted theo risk RW5 của P1-5 §1). Đo lại trên máy local:

| Phép đo | Baseline p50 | Hiện tại p50 | Chênh lệch |
|---|---|---|---|
| `ime_key` | 687 ns | **500 ns** | **−27,2%** |
| `parse_config` | 781 ns | 768 ns | −1,7% |
| `ime_strategy_resolve` | — | 10 ns | dưới ngưỡng đo |

Không có hồi quy sản phẩm; cấu hình job **không** được nới lỏng. Chi tiết và
ranh giới xác minh: [audit Vòng 11](cross-platform-audit-2026-09-30.md).

### Trạng thái

- **Release candidate, không production.** Chưa có Authenticode (host không có
  code-signing certificate có private key), chưa có Developer ID, notarization.
- **Swift chưa compile ở vòng này**: host Windows không có toolchain macOS.
  Test Swift mới (F3-8, F3-13, `ViState`, toggle theo config, lifecycle observer)
  chỉ được xác nhận ở `ci-macos` sau khi push; GUI smoke trên máy Mac thật vẫn
  là **điều kiện production**.
- Số liệu CI và binary artifact của bản này được bổ sung sau khi workflow
  `release-candidate` chạy từ tag sạch.

## Bản 0.2.1 — phát hành pre-release đã xác minh

- Source/tag: `ca22eba2239d9736f51548169f0ce33417954f7a` / `v0.2.1`.
- CI cùng commit: [ci-shared #36718542260](https://github.com/hunglinhpt/TextVN/actions/runs/36718542260),
  [ci-macos #36718542232](https://github.com/hunglinhpt/TextVN/actions/runs/36718542232),
  [repo-hygiene #36718542268](https://github.com/hunglinhpt/TextVN/actions/runs/36718542268) —
  tổng thể đều `success`. Windows portable/installer gõ TSF thật, Linux
  IBus/Fcitx5 chạy package e2e, macOS Swift arm64+x86_64 build/test và đóng gói
  unsigned universal. Chưa có smoke GUI/macOS thật ngoài runner.
- [release-candidate #36719431897](https://github.com/hunglinhpt/TextVN/actions/runs/36719431897)
  build lại từ tag sạch; Windows/Linux/macOS/publish đều `success`.
  [GitHub Release v0.2.1](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.1)
  là **pre-release**, có sáu binary archive/installer + `SHA256SUMS.txt`.
- Checksum đã tải lại và so với digest trên GitHub; Windows portable ZIP
  `e5e5b4fb2062f1c18b5c444247d131276b560d95d7e801f336f92b0f8a9d93b3`,
  setup EXE `f6996d6fba97cf387119f25ddba6c216c9deda582987b9ea2301504b23e34dce`,
  macOS universal ZIP `728788ef1454d213505276b42efc0f897d08580d00f90dd030c428d9c4abbb4e`.
  Mã băm còn lại nằm trong asset `SHA256SUMS.txt` của release.
- Trạng thái: **release candidate, không production**. Chưa có Authenticode
  (host không có code-signing certificate với private key), Developer ID,
  notarization, macOS GUI smoke. Job `perf regression` tham khảo thất bại:
  runner Windows `parse_config` p50 781→1231 ns (+57,6%); đo local
  781→1050 ns (+34,4%). `ci-shared` cấu hình `continue-on-error` cho job này;
  cần đo đối chứng trên cùng phần cứng trước khi kết luận hồi quy sản phẩm.

Trạng thái: **release candidate** cho Windows 10/11 x64 và Linux x86_64 (IBus, Fcitx5).
macOS: mã nguồn đang ở beta; chưa có bằng chứng GUI/package production trên máy Mac thật.

> Audit tiếp diễn 2026-09-30: bảng kết quả CI bên dưới là **bằng chứng lịch sử cho
> commit `766029e`**, không chứng nhận checkout hiện tại hoặc bản ZIP mới. Các sửa
> Windows/Linux/macOS sau commit đó cần CI chạy lại và smoke GUI trên từng hệ điều
> hành. `RELEASE_REPORT.json` nằm trong mỗi ZIP Windows là nguồn trạng thái của
> **chính ZIP ấy**; nếu `status=release-candidate` thì không phát hành là production.
> Xem [audit đa nền tảng 2026-09-30](cross-platform-audit-2026-09-30.md)
> để biết các sửa sau commit lịch sử và phần native chưa xác thực.

Mọi số liệu dưới đây lấy từ CI (`.github/workflows/ci-shared.yml`, workflow `ci-shared`)
và từ máy dựng Linux. Cập nhật file này ở mỗi lần phát hành (xem
[developer-guide.md §5](../developer-guide.md#5-phát-hành)).

| | |
|---|---|
| Nhánh / commit kiểm | `claude/windows-input-method-upgrade-f8bwkv` @ `766029e` |
| Ghi chú | Các commit sau `766029e` chỉ sửa tài liệu và kịch bản test; CI của PR #1 chạy lại toàn bộ trên commit cuối |
| Lần chạy CI | [ci-shared #36458803476](https://github.com/hunglinhpt/TextVN/actions/runs/36458803476) — mọi job PASS (Windows package, Linux adapters…) trừ job hiệu năng tham khảo, xem §5 |
| Toolchain | Rust 1.98.1 stable · MSVC (windows-2022) · GCC/CMake (ubuntu-24.04) |
| Inno Setup | 6.5+ (bản dịch tiếng Việt đi kèm repo) |

## 1. Gói phát hành

| Gói | Tạo bởi | Nội dung |
|---|---|---|
| `TextVN-setup-0.1.0-windows-x64.exe` | `build-release.ps1 -BuildInstaller` | Cài per-user (không cần quyền quản trị), tự đăng ký TSF, khởi động cùng Windows, gỡ qua Settings → Apps |
| `TextVN-portable-0.1.0-windows-x64-<build>.zip` | `build-release.ps1` | `TextVN.exe`, `textvn-tsf.dll`, `textvn-cli.exe`, `install.ps1`/`uninstall.ps1`, `HUONG_DAN_SU_DUNG.txt`, `RELEASE_REPORT.json` |
| `TextVN-0.1.0-linux-x86_64.tar.gz` | `scripts/build-linux.sh` | adapter IBus + Fcitx5, `textvn-settings` (GTK4), `install.sh`, `uninstall.sh`, `textvn-portable.sh` |

Gói Windows mặc định **TSF-only** (không hook bàn phím toàn cục, không `SendInput`). Mã băm:
`SHA256SUMS-<build>.txt` (Windows) và `*.tar.gz.sha256` (Linux), đính kèm cùng artifact CI.

## 2. Kết quả kiểm thử

| Lớp | Phạm vi | Kết quả |
|---|---|---|
| Unit Rust (`cargo test --workspace`) | engine, config, ffi, TSF compose, tray, IPC… — Linux, macOS, Windows | 298 test trên Linux (Windows chạy thêm test riêng của tray/TSF) — **PASS** cả 3 OS |
| Từ vựng thật (`core/tests/common_words.rs`) | ~400 từ × Telex, VNI, dấu kiểu cũ, `uow`, dấu giữa từ, Caps Lock, chữ viết tắt | **PASS** |
| Corpus (`replay`) | 113 kịch bản × 5 mô phỏng adapter (headless, win, tsf, mac, linux) | **113/113 × 5 PASS** |
| ABI (`verify`, `sizes`) | header C ↔ Rust; 20/532 byte | **PASS** |
| fmt · clippy `-D warnings` (Linux + target Windows) · cargo-deny · REUSE · chặn API tiêm mã | | **PASS** |
| Fuzz smoke 60 s | `ffi_key`, `config_parse`, `appdb_parse` | **PASS** |
| Linux e2e (`scripts/e2e-linux.sh`) | ibus-daemon 1.5.29 và fcitx5 5.1.7 thật: gõ, focus-out, reset, Enter, Ctrl+Shift, `state.json`, gõ tắt bật/tắt, Caps Lock, ô mật khẩu | **PASS** |
| Bảng điều khiển Linux (`ctest`) | `settings_model`: vá từng khoá, gõ tắt, file hỏng | **PASS** |
| Hiệu năng (`textvn-bench`) | xem §5 | trong ngân sách |

### Kịch bản 1 — Cài đặt

| Nền tảng | Các bước được kiểm tự động | Kết quả |
|---|---|---|
| Windows (windows-2022, CI job *Windows package*) | Cài im lặng per-user bằng setup `.exe` → file đã cài, CLSID TSF, khởi động cùng Windows → **gõ thật qua TSF** (xem bảng gõ bên dưới) → gỡ im lặng → không còn file/đăng ký, chỉ giữ cấu hình người dùng | **PASS** |
| Linux IBus | `./install.sh` (per-user) → TextVN có trong danh sách bộ gõ → gõ qua ibus-daemon thật → `uninstall.sh` → không còn file nào ngoài cấu hình | **PASS** |
| Linux Fcitx5 | như trên với fcitx5 thật (profile Fcitx5 được thêm/bỏ TextVN) | **PASS** |

### Kịch bản 2 — Giải nén ra dùng luôn

| Nền tảng | Các bước được kiểm tự động | Kết quả |
|---|---|---|
| Windows | Giải nén zip → chạy `TextVN.exe` (tự đăng ký TSF từ thư mục giải nén) → **gõ thật qua TSF** → `uninstall.ps1` → hết đăng ký, cấu hình giữ nguyên | **PASS** |
| Linux IBus | Giải nén vào thư mục **chỉ đọc** → `./textvn-portable.sh` → gõ → `stop` → không ghi gì vào `~/.local` | **PASS** |
| Linux Fcitx5 | như trên | **PASS** |

### Gõ thật trên Windows (mỗi kịch bản, mỗi ứng dụng)

Kết quả dưới đây đúng cho **cả hai** kịch bản (giải nén dùng ngay và cài đặt) trong lần chạy CI ở trên.

`installer/windows/tests/test-typing.ps1` gửi phím bằng `SendInput` (VK + scan code như bàn
phím thật) vào **Notepad** (Win32 Edit — app IMM32 qua CUAS) và **WordPad** (RichEdit —
TSF-aware), rồi đọc lại nội dung:

| Case | Phím | Kỳ vọng | Notepad | WordPad |
|---|---|---|---|---|
| Telex cơ bản | `dduocj␣` | `được␣` | PASS | PASS |
| Chữ hoa đầu | `Vieetj Nam␣` | `Việt Nam␣` | PASS | PASS |
| `uow` | `nguowif␣` | `người␣` | PASS | PASS |
| Vị trí dấu | `cuar␣` | `của␣` | PASS | PASS |
| Tiếng Anh | `hello␣` | `hello␣` | PASS | PASS |
| Dấu câu | `Vieetj, Nam␣` | `Việt, Nam␣` | PASS | PASS |
| Enter + viết hoa đầu câu | `chaof⏎banj␣` | `chào⏎Bạn␣` | PASS | PASS |
| Tab | `tieengs⇥x␣` | `tiếng⇥x␣` | PASS | PASS |
| Home giữa từ | `chaof` Home `x␣` | `x␣chào` | PASS | PASS |
| Ctrl+Shift → E / → V | `as␣` | `as␣` / `á␣` | PASS | PASS |
| Caps Lock | `VIEETJ␣` | `VIỆT␣` | PASS | PASS |

## 3. Lỗi tìm ra nhờ kiểm thử thật (đã sửa trong bản này)

| Tìm bởi | Lỗi | Sửa |
|---|---|---|
| Gõ thật Windows (Notepad) | Dấu cách/dấu câu ra **trước** chữ (`␣được`): app IMM32 nhận kết quả composition sau phím không bị ăn | Ký tự ranh giới in được commit cùng từ (`compose.rs`) |
| Gõ thật Windows (Notepad) | Enter/Tab ra trước chữ, rồi (khi giữ phím ở pha test) mất hẳn Enter/Tab | App CUAS: commit từ ở `OnKeyDown` rồi trả phím gốc cho cửa sổ trong cùng process (`replay.rs`) |
| Gõ thật Windows (WordPad) | Ctrl+Shift không chuyển V/E ở app TSF-aware | `ITfKeyTraceEventSink` (`key_event.rs`) |
| Kịch bản cài đặt sau khi gỡ bản portable | Ctrl+Shift lúc được lúc không: phím tắt đổi bố cục của Windows chuyển đi mất TextVN | Tuỳ chọn "Dành Ctrl + Shift cho TextVN" (`tray/src/hotkey.rs`, bộ cài chọn sẵn) |
| Thử chỉ đăng ký trong vi-VN | App mới mở chạy bàn phím US, TextVN không hoạt động ngay sau khi cài | Giữ đăng ký cả en-US (Windows tiếng Anh) |
| `common_words.rs` | Đặt dấu sai (`cuả`, `nghiã`, `đựơc`…), `d` tự thành `đ`, `gi`/`qu`, `uow` | Viết lại theo quy tắc chính tả |
| e2e ibus/fcitx5 thật | Engine reset mỗi lần caret đổi, addon Fcitx5 không nạp, sai bus name IBus | `1e54df6` |
| Kịch bản cài Linux | ibus-daemon giữ cache registry, bản per-user không hiện sau đăng nhập lại | Xoá cache khi cài/gỡ |
| Dựng installer trên CI | Inno Setup không kèm bản dịch tiếng Việt → không biên dịch được | Kèm `Vietnamese.isl` |

## 4. Phạm vi chưa kiểm tự động

- Windows: Chrome/Edge/Electron, Microsoft Office, ô tìm kiếm Start, ứng dụng UWP chưa có
  kiểm thử gõ tự động (mô hình composition giống WordPad; kiểm tay trước khi phát hành).
- Linux: GNOME Shell/Wayland thật (CI dùng ibus-daemon/fcitx5 không có compositor);
  ứng dụng Qt/Electron.
- `production_blockers` trong `RELEASE_REPORT.json` của gói Windows: UI Automation gốc chưa tích
  hợp (cổng bảo mật dùng tín hiệu TSF trong process), DACL của named pipe chưa được kiểm
  định, bản CI **chưa ký Authenticode**.

## 5. Hiệu năng

`textvn-bench` (p50 mỗi từ gõ; ngân sách `ime_key` p99 < 0,5 ms):

| Đo | main | bản này | Ghi chú |
|---|---|---|---|
| `ime_key` (Linux, máy dựng) | 465 ns | 475 ns | +2 %: quy tắc đặt dấu/Caps Lock làm thêm việc, bù bằng tra bảng nguyên âm trực tiếp |
| `ime_key` (runner Windows, cùng ngày) | 762 ns | 775 ns | runner dùng chung; baseline cũ 687 ns — `main` cũng vượt ngưỡng 10 % |
| `parse_config` (runner Windows) | +56 % so với baseline | tương đương main | trôi của runner: cùng mã trên `main` cũng vượt ngưỡng hôm nay |

Job hiệu năng trên runner GitHub là tham khảo (`continue-on-error`); cổng cứng chạy trên
runner cố định (P1-5 §5).

## 6. Tái lập

```bash
cargo test --workspace && for a in headless win tsf mac linux; do
  cargo run -q -p textvn-cli -- replay corpus --adapter $a; done
scripts/e2e-linux.sh && scripts/build-linux.sh && scripts/test-linux-package.sh dist/TextVN-*.tar.gz
```

```powershell
powershell -File .\build-release.ps1 -BuildInstaller
powershell -File installer\windows\tests\test-portable.ps1 -Zip (Get-Item dist\TextVN-portable-*.zip).FullName
powershell -File installer\windows\tests\test-installer.ps1 -Setup (Get-Item dist\TextVN-setup-*.exe).FullName
```
