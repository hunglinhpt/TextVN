# Phân phối Linux — kênh, store và auto-update

> Trạng thái 2026-10-18. Bổ sung từ yêu cầu "public lên repo/store của các
> distro Linux" với góc nhìn audit + QA: mỗi kênh được đánh giá **theo loại
> ứng dụng** (IME hệ thống) chứ không theo độ nổi tiếng của store.

## 1. Ràng buộc quyết định kênh: TextVN là IME hệ thống

Trên Linux, TextVN **không phải một app tự chứa** — nó là:

- một **IBus engine** (tiến trình riêng, được `ibus-daemon` của HOST spawn qua
  component XML ở `/usr/share/ibus/component/`), và
- một **Fcitx5 addon** (thư viện `.so` được `fcitx5` của HOST nạp từ
  `FCITX_ADDON_DIRS`/libdir hệ thống), và
- một app settings GTK4 (phần duy nhất "bình thường").

Framework IME (ibus-daemon/fcitx5) chạy **trên host**, ngoài sandbox. Kênh
phân phối phải đặt file vào đúng vị trí hệ thống mà host daemon quét.

## 2. Đánh giá từng kênh (audit)

| Kênh | Phán quyết | Lý do (bằng chứng) |
|---|---|---|
| **AUR** (Arch/Manjaro/CachyOS/EndeavourOS…) | ✅ **ĐÃ CHUẨN BỊ** (`packaging/arch/PKGBUILD` + `.SRCINFO`) | Build từ nguồn của tag (chuẩn Arch), cài đúng layout `/usr/{bin,lib,share}` mà install.sh/daemon quét; người dùng Arch cài bằng helper AUR và được cập nhật cùng hệ thống. QA: `ci-aur.yml` chạy **makepkg thật** trong container `archlinux:base-devel` — build, đối chiếu `.SRCINFO`, kiểm đủ file then chốt, khớp version Cargo.toml |
| **Tarball GitHub Release** (mọi distro) | ✅ Hiện hành | `TextVN-<ver>-linux-<arch>.tar.gz` + `install.sh` (per-user/`--system`) + `textvn-portable.sh`; CI Linux cài thử thật với ibus-daemon/fcitx5 thật trước khi phát hành |
| **Debian/Ubuntu official repo** | ⏳ Cần maintainer distro | Không thể tự nộp: cần gói `.deb` tuân policy (multiarch paths, `dh` packaging) + sponsor trong Debian/Ubuntu. Đây là việc của maintainer distro khi có nhu cầu — tài liệu cho họ: layout cài đặt đã chuẩn (`/usr/share/ibus/component`, `fcitx5` addon libdir multiarch — `textvn-common.sh` liệt kê đủ `/usr/lib/<triplet>/fcitx5`, `/usr/lib64/fcitx5`, `/usr/lib/fcitx5`) |
| **PPA (Launchpad)** | ⏳ Tuỳ chọn, cần tài khoản Ubuntu SSO của owner | Kênh `.deb` cho người Ubuntu muốn auto-update qua apt. Đánh đổi: phải duy trì build recipe. Chưa làm — AUR + tarball phủ đủ nhu cầu hiện tại |
| **Flathub (Flatpak)** | ❌ **KHÔNG PHÙ HỢP** | Flatpak sandbox: một IBus engine trong Flatpak không thể được `ibus-daemon` của host spawn (component ở trong sandbox, host không thấy); Fcitx5 addon `.so` trong Flatpak không thể được `fcitx5` host nạp. Flatpak IME chỉ hoạt động cho app Flatpak khác qua portal — **mọi app host (Chrome, VS Code, GTK thường) sẽ KHÔNG gõ được tiếng Việt**. Các bộ gõ trên Flathub (ví dụ ibus-bogo cũ) hoạt động ở dạng "app ngoài", không phải engine hệ thống |
| **Snap Store** | ❌ KHÔNG PHÙ HỢP (cùng lý do) | Snap cũng sandbox; `classic` thoát sandbox về nguyên tắc được nhưng Snap Store không có kênh "system input method" — cài classic snap = giải nén binary vào `/snap`, không đặt được vào libdir addon mà fcitx5 quét nếu không can thiệp hệ thống, mất nghĩa "store" |
| **AppImage** | ❌ Không có giá trị thêm | Tarball hiện tại ĐÃ là "chạy ngay" (`textvn-portable.sh`) với cơ chế runtime/env sạch hơn (XDG_RUNTIME_DIR + khôi phục state, xem `textvn-portable.sh`) — AppImage chỉ thêm format không giải được bài toán tích hợp daemon |

**Kết luận**: đầu tư vào **AUR + tarball**; Flathub/Snap/AppImage bị loại có
căn cứ (loại ứng dụng), không phải vì bỏ lỡ.

## 3. AUR — nội dung đã chuẩn bị

| File | Vai trò |
|---|---|
| `packaging/arch/PKGBUILD` | Build từ source archive của tag: `cargo build --frozen` (ffi + cli) + cmake 3 adapter; cài layout `/usr` hệ thống; `check()` chạy test `textvn-core` |
| `packaging/arch/.SRCINFO` | Metadata AUR — CI bắt phải khớp `makepkg --printsrcinfo` |
| `.github/workflows/ci-aur.yml` | QA gate: container `archlinux:base-devel`, makepkg thật (user thường, không root), diff `.SRCINFO`, kiểm đủ 10 file then chốt, khớp version Cargo.toml, upload `.pkg` artifact |
| `tools/release/publish-aur.sh` | Đẩy PKGBUILD + .SRCINFO lên `ssh://aur@aur.archlinux.org/textvn.git` (cục bộ: ssh-agent sẵn; CI: secret `AUR_SSH_KEY`) |

### Bước của chủ repo để publish lần đầu (một lần, ~10 phút)

1. Tạo account `aur.archlinux.org` → Profile → **SSH Keys** → add public key.
2. Kích hoạt CI: `gh workflow run ci-aur` (hoặc push đụng `packaging/arch/**`)
   — phải xanh trước khi publish.
3. `./tools/release/publish-aur.sh` — push tự tạo gói `textvn` trên AUR.
4. Kiểm: trang `aur.archlinux.org/packages/textvn` + `paru -Si textvn`.

### Bump PKGBUILD khi phát hành bản mới (B7e)

1. `pkgver` + `sha256sums` (hash **mới** của source archive v<ver> — GitHub
   sinh archive per-tag) trong `PKGBUILD`, regenerate `.SRCINFO`.
2. CI `ci-aur` xanh (makepkg thật đã chạy được).
3. `./tools/release/publish-aur.sh` (từ chối chạy nếu `.SRCINFO`/PKGBUILD lệch
   version Cargo.toml).

## 4. Ma trận auto-update sau release (mục tiêu "release xong store tự có bản mới")

| Nền tảng / kênh | Tự động? | Cơ chế |
|---|---|---|
| Windows — Microsoft Store | ✅ | `store-publish` workflow nộp MSIX (repo variable `STORE_AUTO_PUBLISH` = `draft`/`submit`), Store tự cập nhật user |
| Windows — setup exe / portable | ⚙️ | Người dùng tải lại; bản máy/portable không tự update (thiết kế riêng tư — không mạng) |
| macOS — Homebrew tap | ✅ | `sync-cask.yml` trong tap tự bump cask (verify GPG) mỗi 6 giờ; user `brew upgrade` |
| macOS — `.pkg` | ⚙️ | Tải lại từ release |
| Linux — **AUR** | ✅* | Sau khi owner publish lần đầu: bump PKGBUILD bằng `tools/release/publish-aur.sh` (từ chối lệch version); user update qua `paru`/`yay -Syu` cùng hệ thống. Hoàn toàn tự động khi chủ repo đặt secret `AUR_SSH_KEY` + chạy script ở bước B7e (có thể đưa vào release workflow sau) |
| Linux — tarball | ⚙️ | Cài lại `install.sh` (nâng cấp giữ nguyên cấu hình + manifest dọn residue bản cũ) |

*"Tự động" của AUR = một lệnh `publish-aur.sh` sau release (CI đã build-verify
trước đó); đẩy thẳng tay qua SSH vào AUR không khuyến nghị — mất gate QA.

## 5. QA đã chạy cho gói AUR (bằng chứng)

- `bash -n` PKGBUILD + publish script — PASS.
- `ci-aur.yml`: makepkg thật trong `archlinux:base-devel` (môi trường người
  dùng AUR thật) — build, check() test engine, `.SRCINFO` diff, 10 file bắt
  buộc, version khớp Cargo.toml, artifact `.pkg` upload.
- Chạy thật: link run được ghi vào `build-release-report.md` khi có.
