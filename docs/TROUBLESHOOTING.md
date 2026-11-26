# Khắc phục sự cố TextVN (Troubleshooting)

> Bản tiếng Anh ở mục [# English](#english) cuối file — cùng cấu trúc, cùng số liệu.
> Hướng dẫn cài đặt đầy đủ: [user-guide.md](user-guide.md). Báo lỗi:
> <https://github.com/hunglinhpt/TextVN/issues>.

## 1. Bắt đầu: xuất gói chẩn đoán (30 giây)

Trước khi báo lỗi, hãy xuất **gói chẩn đoán** — tự ẩn tên người dùng và **không bao giờ
chứa nội dung bạn gõ, clipboard hay từ vựng cá nhân** (G8/S2).

| Cách | Windows | Linux | macOS |
|---|---|---|---|
| Giao diện | Bảng điều khiển → **Hướng dẫn** → **[Chẩn đoán & xuất log]** (nút đầu tiên), hoặc chuột phải biểu tượng khay → **Chẩn đoán & xuất log...** | Bảng cài đặt → **Hướng dẫn** → **[Chẩn đoán & xuất log]** (nút nổi bật hàng đầu), hoặc menu IBus/Fcitx5 → **Chẩn đoán & xuất log…** | App menu bar → **Chẩn đoán & xuất log...**, hoặc Cài đặt → **Hướng dẫn** → **[Chẩn đoán & xuất log]** |
| Dòng lệnh | `textvn-cli --export-log` (cạnh `TextVN.exe`) | `textvn --export-log` | — (giao diện tạo tệp `.md`) |
| Kết quả | `textvn-diag-<thời điểm>.zip` trên Desktop | `textvn-diag-<thời điểm>.zip` trên Desktop / `$HOME` | `textvn-diag-<thời điểm>.md` trên Desktop |

Sau khi xuất, hộp thoại hiển thị ngay các phát hiện chính kèm nút **[Mở thư mục tệp ZIP]** (Windows/Linux — macOS tự mở Finder chọn sẵn tệp), **[Copy tóm tắt lỗi]** (Windows/macOS — chép tóm tắt đã ẩn tên người dùng) và **[Mở trang báo lỗi]**. Dán tóm tắt vào ô mô tả issue rồi kéo thả tệp vừa mở vào.

Các lệnh dòng lệnh khác (Windows dùng `textvn-cli`, Linux dùng `textvn`):

| Lệnh | Tác dụng | Exit code |
|---|---|---|
| `--check-env [--json]` | Kiểm tra môi trường, phụ thuộc, xung đột bộ gõ (= `doctor`) | 0 sạch · 1 có lỗi |
| `--diagnose [--json]` | Báo cáo Markdown sẵn dán GitHub Issues (= `doctor --report`) | 0 · 1 |
| `--export-log [<đường-dẫn.zip>]` | Gói ZIP chẩn đoán (= `doctor --export`) | 0 · 2 lỗi ghi tệp |
| `doctor --pause` | Như `--check-env`, giữ cửa sổ console (shortcut Start menu) | 0 · 1 |

### Gói ZIP chứa gì

| Tệp | Nội dung |
|---|---|
| `report_summary.md` | Tóm tắt Markdown: phiên bản, OS, các mã lỗi/cảnh báo kèm gợi ý |
| `version.json` | `app_version`, `git_sha` (commit build), `target` (target triple), `distribution` (`installer-user` / `installer-machine` / `portable` / `store` / `system-package` / `user-install` / `app-bundle`), `abi_version` |
| `system_info.json` | Trạng thái pipe IPC, tray, hook, TIP TSF + mảng `diagnostics[]` |
| `windows_env.json` / `linux_env.json` / `macos_env.json` | Windows: phiên bản/Build/UBR/Edition, chi tiết TIP (HKCU/HKLM), xung đột · Linux: os-release, kernel, glibc, trình quản lý gói, phiên X11/Wayland, desktop, SELinux/AppArmor, biến IM, IBus/Fcitx5/GTK, xung đột |
| `config.redacted.json` | Cấu hình — gõ tắt, emoji, từ tiếng Anh, Style Profile chỉ còn **số mục** |
| `panic_tail.log` | Bản ghi panic gần nhất (`ERR_PANIC`): vị trí mã nguồn + tên hàm backtrace; **không** ghi thông điệp động |
| `register_tail.log` (Windows) · `install_diag_tail.log` (Linux) · `tsf_tail.log` | Đuôi log đăng ký TSF / cài đặt / runtime, đã ẩn tên người dùng |

### Vị trí log trên máy

| OS | Thư mục | Tệp |
|---|---|---|
| Windows | `%LOCALAPPDATA%\TextVN\logs\` | `register.log`, `tsf-min.log`, `store.log`, `panic.log` |
| Linux | `$XDG_STATE_HOME/TextVN/log/` hoặc `~/.local/state/TextVN/log/` | `install-diag.log`, `textvn.log`, `panic.log` |
| macOS | `~/Library/Logs/TextVN/` | `im-<pid>.log`, `textvn.log` |

`panic.log` (từ 0.2.90) do tray `TextVN.exe`, `textvn-cli` và `textvn-hook.exe` ghi khi gặp
panic: thời điểm, thành phần, phiên bản, target, tên luồng, `file:dòng:cột`, tên hàm trong
backtrace. Thông điệp panic dạng chuỗi động (có thể chứa dữ liệu) **không bao giờ được
ghi** — chỉ ghi độ dài; chỉ giữ hằng chuỗi biên dịch sẵn trong mã nguồn. Tối đa 64 KiB
(tự xoay vòng).

## 2. Xung đột bộ gõ (gõ lặp chữ, mất dấu, phím V/E không phản hồi)

Hai bộ gõ tiếng Việt cùng xử lý một phím → chữ bị biến đổi hai lần (`aa` → `â` → `aâ`),
mất dấu, hoặc phím tắt chuyển chế độ bị bộ gõ kia giành. TextVN tự phát hiện:

- **Windows**: tray chạy kiểm tra xung đột ở nền **mỗi lần khởi động** và hiện thông báo
  khay **một lần cho mỗi tập xung đột mới** (marker `%APPDATA%\TextVN\conflict_notice`;
  hết xung đột thì marker tự xoá; tắt kiểm tra: biến môi trường
  `TEXTVN_SKIP_CONFLICT_CHECK=1`).
- **Linux**: `install.sh` / `textvn-portable.sh` kiểm khi cài/chạy và ghi `install-diag.log`.
- **Mọi OS**: `--check-env`, `--diagnose`, gói chẩn đoán.

| Mã | OS | Phát hiện | Cách khắc phục |
|---|---|---|---|
| `W_CONFLICT_VI_IME` | Windows | `UniKey`, `EVKey`, `OpenKey`, `GoTiengViet` đang chạy (mutex) hoặc `UniKey`/`EVKey`/`OpenKey`/`GoTiengViet`/`VietKey` tự khởi động (`HKCU`/`HKLM\...\Run`) — không duyệt danh sách tiến trình (an toàn với phần mềm diệt virus) | Chuột phải biểu tượng bộ gõ cũ ở khay → Thoát; tắt "Khởi động cùng Windows" trong bộ gõ đó hoặc *Task Manager → Startup apps → Disable* |
| `W_CONFLICT_VI_TIP` | Windows | Bộ gõ TSF tiếng Việt khác đang bật trong danh sách ngôn ngữ: **Microsoft Vietnamese Telex**, **Microsoft Vietnamese Number Key-Based (VNI)** (CLSID `{C2CB2CF0-AF47-413E-9780-8BC3A3C16068}`) hoặc TIP lạ | *Cài đặt → Thời gian & ngôn ngữ → Ngôn ngữ & vùng → Tiếng Việt → Tùy chọn ngôn ngữ → Bàn phím* → xoá bàn phím đó (giữ **TextVN**). Nếu không, Win+Space / Alt+Shift có thể chuyển sang bộ gõ Microsoft |
| `W_CTRL_SHIFT_TAKEN` | Windows | Windows giữ Ctrl+Shift để đổi bố cục bàn phím | Bảng điều khiển → **Dành Ctrl + Shift cho TextVN**, hoặc `TextVN.exe --free-ctrl-shift` |
| `W_CONFLICT_VI_IME` | macOS | `OpenKey.app`, `EVKey.app`, `GoTiengViet.app` đã cài | Thoát bộ gõ cũ khỏi thanh menu; tắt quyền Accessibility / Input Monitoring của nó trong *System Settings → Privacy & Security* |
| `W_CONFLICT_APPLE_VI_IM` | macOS | Bộ gõ có sẵn **Apple Vietnamese Telex / Simple Telex / VNI / VIQR** đang bật | *System Settings → Keyboard → Text Input → Edit…* → chọn bộ gõ Vietnamese của Apple → nút **−** |
| `CONFLICT_DUAL_IM_DAEMON` | Linux | `ibus-daemon` và `fcitx5` cùng chạy trong phiên | Chỉ giữ một framework: GNOME/RHEL/Ubuntu → IBus; KDE → Fcitx5. Vd. KDE: `ibus exit` rồi bỏ autostart IBus |
| `W_CONFLICT_VI_IME` | Linux | Engine tiếng Việt khác đang chạy: `ibus-engine-unikey` / `bamboo` / `bogo` / `teni` (IBus), hoặc `fcitx` v4 / `scim` / `uim` / `xvnkb` / `xvnhook` | IBus: *Cài đặt → Bàn phím → Nguồn nhập* bỏ engine cũ, hoặc `sudo apt-get remove ibus-unikey ibus-bamboo` / `sudo dnf remove ibus-bamboo`; thoát `xvnkb` |
| `W_CONFLICT_FCITX5_VI_IME` | Linux | Profile Fcitx5 còn `unikey` / `bamboo` / `m17n_vi_*` | `fcitx5-configtool` → gỡ bộ gõ cũ, chỉ giữ **TextVN** |
| `W_IM_ENV_MISMATCH` | Linux | Biến `GTK_IM_MODULE` / `QT_IM_MODULE` / `XMODIFIERS` trỏ tới framework khác framework đang chạy (vd. chạy IBus nhưng `GTK_IM_MODULE=fcitx`) | Sửa `~/.config/environment.d/*.conf`, `~/.xprofile` hoặc `/etc/environment` cho khớp (`ibus` ↔ `@im=ibus`, `fcitx` ↔ `@im=fcitx`), đăng xuất/đăng nhập |

Thoát bộ gõ cũ xong: chạy lại `--check-env` để xác nhận không còn cảnh báo.

## 3. Bảng mã lỗi đầy đủ

| Mã | Mức | Nguồn | Ý nghĩa | Xử lý |
|---|---|---|---|---|
| `E_ABI_MISMATCH` | lỗi | doctor (3 OS) | Kích thước struct FFI giữa CLI và adapter lệch | Cài lại trọn một bản phát hành (không trộn tệp hai phiên bản) |
| `E_CONFIG_INVALID` | lỗi | doctor | `config.json` sai schema `config.v1` — engine chạy mặc định | Sao lưu rồi xoá `config.json`; TextVN tự tạo lại |
| `E_CONFIG_UNREADABLE` | lỗi | doctor | Không đọc được `config.json` | Kiểm tra quyền thư mục cấu hình |
| `E_TSF_UNREGISTERED` | lỗi | doctor (Windows) | TIP TSF chưa đăng ký / DLL không còn tồn tại | Hướng dẫn → **[Cài & bật TSF]**, hoặc bộ cài `TextVN-setup-<ver>-windows-x64-machine.exe` |
| `E_DEP_MISSING` | lỗi | doctor + installer (Linux) | Thiếu `ibus`/`libibus`/`fcitx5`/GTK hoặc ABI không nạp được | `./install.sh --install-deps`, hoặc lệnh ở §4 |
| `E_DEP_INSTALL_FAIL` | lỗi | installer (Linux) | Trình quản lý gói cài thất bại | Chạy tay lệnh in ra, xem `install-diag.log` |
| `E_BUILD_DEP_MISSING` / `E_BUILD_DEP_INSTALL_FAIL` | lỗi | `scripts/install_linux.sh` | Thiếu công cụ/header để build từ mã nguồn | Cài `cmake`, `pkg-config`, `ibus-devel`/`libibus-1.0-dev`, `gtk4-devel` hoặc `gtk3-devel` |
| `E_GLIBC_INCOMPAT` | lỗi | installer (Linux) | `bin/textvn` không khởi động được trên máy (loader/glibc không tương thích — vd. glibc < 2.28, cũ hơn RHEL 8) | Dựng từ mã nguồn: `./scripts/install_linux.sh`; bản dựng sẵn hỗ trợ glibc ≥ 2.28 |
| `E_GTK_MISSING` / `W_GTK_MISSING` | lỗi/cảnh báo | installer / doctor | Bảng cài đặt GUI không mở được (thiếu GTK 4 lẫn GTK 3) — bộ gõ vẫn chạy | Cài `gtk3` (EL8/SLES) hoặc `gtk4` |
| `E_PKG_INVALID` | lỗi | installer (Linux) | Tarball thiếu tệp / tải lỗi | Tải lại, kiểm `SHA256SUMS.txt` |
| `E_KWIN_WAYLAND` | lỗi | `textvn-portable.sh` (Linux) | KDE Plasma Wayland quản lý Fcitx5 qua `WAYLAND_SOCKET` — bản portable không restart Fcitx5 được mà không làm đứt KWin | Dùng `./install.sh` (per-user), hoặc `./textvn-portable.sh --ibus` |
| `W_DEP_NO_IM_FRAMEWORK` | cảnh báo | doctor (Linux) | Không thấy `ibus-daemon` lẫn `fcitx5` | Cài IBus (GNOME/RHEL) hoặc Fcitx5 (KDE) |
| `W_RECENT_PANIC` | cảnh báo | doctor (3 OS) | `panic.log` có bản ghi `ERR_PANIC` | Xuất gói chẩn đoán và báo lỗi |
| Mã xung đột | cảnh báo | §2 | Xem bảng §2 | — |

## 4. Linux: phụ thuộc theo distro

`install.sh` tự nhận diện `/etc/os-release` + trình quản lý gói và đề nghị cài phần thiếu:
`./install.sh` (hỏi `[Y/n]`), `./install.sh --install-deps` (tự cài), `./install.sh
--no-install-deps` (chỉ kiểm tra). Không bao giờ tự cài Fcitx5 đè lên máy GNOME đang dùng IBus.
Cài tay:

| Distro | Lệnh |
|---|---|
| RHEL 8/9 · Rocky · AlmaLinux · Fedora | `sudo dnf install -y ibus ibus-libs gtk3` |
| SLES 15 · openSUSE Leap | `sudo zypper install -y ibus libibus-1_0-5 libgtk-3-0` |
| Ubuntu · Debian · Linux Mint · Pop!_OS | `sudo apt-get install -y ibus libibus-1.0-5 libgtk-4-1` (Ubuntu 20.04 / Debian 11: `libgtk-3-0`) |
| Arch · Manjaro | `sudo pacman -S ibus gtk4` (hoặc `fcitx5`) |

**RHEL 8.10 / Rocky 8 / AlmaLinux 8**: Fcitx5 không có trong BaseOS/AppStream — dùng **IBus**
(mặc định của GNOME). TextVN chạy IBus ngay khi cài, không cần Fcitx5.

**Vì sao plugin Fcitx5 và bảng cài đặt GTK 4 cần glibc 2.35?** Tarball có hai baseline:
`bin/textvn`, `textvn-ibus-engine`, `textvn-settings-gtk3` dựng trên AlmaLinux 8
(**glibc ≤ 2.28** — cổng `cargo xtask check-elf-abi` chặn ký hiệu mới hơn), chạy trên mọi
distro trong bảng. `libtextvn-fcitx5.so` và `textvn-settings-gtk4` chỉ có ý nghĩa trên máy có
Fcitx5 ≥ 5.0.14 / GTK 4 — các thư viện đó chỉ có trên distro glibc ≥ 2.35, nên dựng trên
Ubuntu 22.04. Máy cũ hơn tự dùng IBus + GTK 3 (`bin/textvn-settings` tự chọn).

Kiểm thử thật trong CI mỗi lần push: E2E đầy đủ trên RHEL 8.10 / AlmaLinux 8, RHEL 9 /
AlmaLinux 9, SLES 15 / openSUSE Leap 15.6, Ubuntu 22.04, Debian 12, Ubuntu 24.04, Fedora 41,
Arch Linux; smoke (ldd, CLI chẩn đoán, `--check-abi`, cài/gỡ per-user) trên Ubuntu 20.04 và
Debian 11.

## 5. Windows: doanh nghiệp, LTSC, portable, quyền quản trị

| Tình huống | Hành vi / cách xử lý |
|---|---|
| Không có quyền quản trị | Bộ cài `TextVN-setup-<ver>-windows-x64.exe` và bản portable chỉ ghi `HKCU` — không cần UAC |
| Triển khai cho cả máy (IT) | `TextVN-setup-<ver>-windows-x64-machine.exe` (Program Files + `HKLM`), cài im lặng: xem [user-guide.md](user-guide.md) §Cài đặt im lặng |
| Windows 11 Enterprise 23H2/24H2+ từ chối kích hoạt bản per-user / portable | Hướng dẫn → **[Cài & bật TSF]** báo `RegisteredNeedsManualActivate`; cài bản `-machine.exe` |
| Windows 10 LTSC 2019/2021, Enterprise không có Microsoft Store / WebView2 | Không cần: TextVN không dùng WebView2, không cần Store; CRT liên kết tĩnh (`+crt-static`) nên không cần VC++ Redistributable; API chỉ có trên Win11 được nạp động (fallback trên Win10 1507+) |
| Portable trong `D:\Tools\TextVN`, USB, thư mục có dấu tiếng Việt | Chạy được, không cần quyền quản trị. Cấu hình lưu ở `%APPDATA%\TextVN`, log ở `%LOCALAPPDATA%\TextVN\logs` (xem lưu ý dưới) |
| Ứng dụng chạy quyền Administrator | TSF nạp TextVN vào cả ứng dụng nâng quyền của cùng tài khoản. Riêng cửa sổ Cài đặt: nếu một bản TextVN đang chạy quyền quản trị, UIPI chặn bản thường mở nó — TextVN báo rõ; thoát bản nâng quyền |
| SmartScreen / phần mềm diệt virus cảnh báo | Chưa có chữ ký Authenticode (chờ SignPath); xem [antivirus-false-positive.md](specs/antivirus-false-positive.md) |

**Vì sao bản portable vẫn lưu cấu hình ở `%APPDATA%`?** Bộ gõ TSF là DLL nạp vào **tiến
trình của mọi ứng dụng**, kể cả ứng dụng sandbox AppContainer (Edge, ứng dụng Store, ô tìm
kiếm). Các tiến trình này không đọc được thư mục tuỳ ý như `D:\Tools\TextVN` hay USB — nếu
cấu hình nằm cạnh `TextVN.exe`, gõ tiếng Việt trong các ứng dụng đó sẽ hỏng. `%APPDATA%` thuộc
người dùng hiện tại, luôn ghi được, không cần quyền quản trị và không đụng `HKLM`. Gỡ bản
portable: chuột phải biểu tượng khay → **Gỡ cài đặt** (gỡ đăng ký TSF `HKCU`) rồi xoá thư mục;
`%APPDATA%\TextVN` được giữ lại để dùng tiếp — xoá tay nếu muốn.

## 6. Báo lỗi

1. Xuất gói chẩn đoán (§1). 2. Windows: **[Copy tóm tắt lỗi]** → **[Mở trang báo lỗi]**.
3. Mở issue tại <https://github.com/hunglinhpt/TextVN/issues>, dán tóm tắt, mô tả ứng dụng
đang gõ, chuỗi phím, kết quả thực tế và mong đợi, rồi đính kèm tệp ZIP / `.md`.

---

# English

> Same structure and numbers as the Vietnamese section above. Full installation guide:
> [user-guide.md](user-guide.md). Bug reports: <https://github.com/hunglinhpt/TextVN/issues>.

## 1. Start here: export a diagnostic bundle (30 seconds)

Before reporting a bug, export a **diagnostic bundle** — it redacts your user name and
**never contains anything you typed, your clipboard or your personal vocabulary** (G8/S2).

| How | Windows | Linux | macOS |
|---|---|---|---|
| UI | Control panel → **Help (*Hướng dẫn*)** → **[Chẩn đoán & xuất log]** (top Command Link), or right-click the tray icon → **Chẩn đoán & xuất log...** | Settings window → **Help** → **[Chẩn đoán & xuất log]** (prominent top button), or IBus/Fcitx5 indicator menu → **Chẩn đoán & xuất log…** | Menu bar app → **Chẩn đoán & xuất log...**, or Settings → **Help** → **[Chẩn đoán & xuất log]** |
| Command line | `textvn-cli --export-log` (next to `TextVN.exe`) | `textvn --export-log` | — (the UI writes an `.md` file) |
| Output | `textvn-diag-<timestamp>.zip` on the Desktop | `textvn-diag-<timestamp>.zip` on the Desktop / `$HOME` | `textvn-diag-<timestamp>.md` on the Desktop |

After exporting, the dialog shows the key findings immediately along with **[Mở thư mục tệp ZIP]** (*open ZIP folder* on Windows/Linux — macOS reveals the file directly in Finder), **[Copy tóm tắt lỗi]** (*copy error summary* on Windows/macOS, user name redacted), and **[Mở trang báo lỗi]** (*open the bug-report page*). Paste the summary into the issue description, then drag the revealed file in.

Other command-line entry points (Windows uses `textvn-cli`, Linux uses `textvn`):

| Command | Effect | Exit code |
|---|---|---|
| `--check-env [--json]` | Check environment, dependencies and IME conflicts (= `doctor`) | 0 clean · 1 problems found |
| `--diagnose [--json]` | Markdown report ready to paste into GitHub Issues (= `doctor --report`) | 0 · 1 |
| `--export-log [<path.zip>]` | Diagnostic ZIP bundle (= `doctor --export`) | 0 · 2 write error |
| `doctor --pause` | Same as `--check-env`, keeps the console window open (Start-menu shortcut) | 0 · 1 |

### What the ZIP contains

| File | Content |
|---|---|
| `report_summary.md` | Markdown summary: version, OS, error/warning codes with hints |
| `version.json` | `app_version`, `git_sha` (build commit), `target` (target triple), `distribution` (`installer-user` / `installer-machine` / `portable` / `store` / `system-package` / `user-install` / `app-bundle`), `abi_version` |
| `system_info.json` | IPC pipe, tray, hook and TSF TIP status + the `diagnostics[]` array |
| `windows_env.json` / `linux_env.json` / `macos_env.json` | Windows: version/Build/UBR/Edition, TIP detail (HKCU/HKLM), conflicts · Linux: os-release, kernel, glibc, package manager, X11/Wayland session, desktop, SELinux/AppArmor, IM variables, IBus/Fcitx5/GTK, conflicts |
| `config.redacted.json` | Configuration — macros, emoji, English words and Style Profile reduced to **item counts** |
| `panic_tail.log` | Most recent panic records (`ERR_PANIC`): source location + backtrace function names; dynamic messages are **not** recorded |
| `register_tail.log` (Windows) · `install_diag_tail.log` (Linux) · `tsf_tail.log` | Tails of the TSF registration / install / runtime logs, user name redacted |

### Log locations on disk

| OS | Directory | Files |
|---|---|---|
| Windows | `%LOCALAPPDATA%\TextVN\logs\` | `register.log`, `tsf-min.log`, `store.log`, `panic.log` |
| Linux | `$XDG_STATE_HOME/TextVN/log/` or `~/.local/state/TextVN/log/` | `install-diag.log`, `textvn.log`, `panic.log` |
| macOS | `~/Library/Logs/TextVN/` | `im-<pid>.log`, `textvn.log` |

`panic.log` (since 0.2.90) is written by the `TextVN.exe` tray, `textvn-cli` and
`textvn-hook.exe` when they panic: timestamp, component, version, target, thread name,
`file:line:column`, backtrace function names. Dynamic-string panic messages (which could
carry data) are **never recorded** — only their length; only compile-time constant strings
from the source code are kept. Capped at 64 KiB (self-rotating).

## 2. IME conflicts (doubled characters, lost diacritics, V/E toggle not responding)

When two Vietnamese input methods process the same key, text is transformed twice
(`aa` → `â` → `aâ`), diacritics get lost, or the other IME grabs the mode-switch hotkey.
TextVN detects this automatically:

- **Windows**: the tray runs a background conflict check **on every start** and shows a
  tray notification **once per new conflict set** (marker `%APPDATA%\TextVN\conflict_notice`;
  the marker is removed once conflicts are gone; disable the check with the environment
  variable `TEXTVN_SKIP_CONFLICT_CHECK=1`).
- **Linux**: `install.sh` / `textvn-portable.sh` check at install/run time and write `install-diag.log`.
- **All OSes**: `--check-env`, `--diagnose`, the diagnostic bundle.

| Code | OS | Detects | Fix |
|---|---|---|---|
| `W_CONFLICT_VI_IME` | Windows | `UniKey`, `EVKey`, `OpenKey`, `GoTiengViet` running (mutex) or `UniKey`/`EVKey`/`OpenKey`/`GoTiengViet`/`VietKey` set to auto-start (`HKCU`/`HKLM\...\Run`) — no process-list enumeration (antivirus-safe) | Right-click the old IME's tray icon → Exit; disable its "start with Windows" option or *Task Manager → Startup apps → Disable* |
| `W_CONFLICT_VI_TIP` | Windows | Another Vietnamese TSF input method enabled in the language list: **Microsoft Vietnamese Telex**, **Microsoft Vietnamese Number Key-Based (VNI)** (CLSID `{C2CB2CF0-AF47-413E-9780-8BC3A3C16068}`) or an unknown TIP | *Settings → Time & language → Language & region → Vietnamese → Language options → Keyboards* → remove that keyboard (keep **TextVN**). Otherwise Win+Space / Alt+Shift may switch to the Microsoft IME |
| `W_CTRL_SHIFT_TAKEN` | Windows | Windows reserves Ctrl+Shift for switching keyboard layouts | Control panel → **Dành Ctrl + Shift cho TextVN** (*reserve Ctrl+Shift for TextVN*), or `TextVN.exe --free-ctrl-shift` |
| `W_CONFLICT_VI_IME` | macOS | `OpenKey.app`, `EVKey.app`, `GoTiengViet.app` installed | Quit the old IME from the menu bar; revoke its Accessibility / Input Monitoring permission in *System Settings → Privacy & Security* |
| `W_CONFLICT_APPLE_VI_IM` | macOS | Built-in **Apple Vietnamese Telex / Simple Telex / VNI / VIQR** enabled | *System Settings → Keyboard → Text Input → Edit…* → select Apple's Vietnamese input source → **−** button |
| `CONFLICT_DUAL_IM_DAEMON` | Linux | `ibus-daemon` and `fcitx5` both running in the session | Keep one framework: GNOME/RHEL/Ubuntu → IBus; KDE → Fcitx5. E.g. on KDE: `ibus exit` and remove the IBus autostart |
| `W_CONFLICT_VI_IME` | Linux | Another Vietnamese engine running: `ibus-engine-unikey` / `bamboo` / `bogo` / `teni` (IBus), or `fcitx` v4 / `scim` / `uim` / `xvnkb` / `xvnhook` | IBus: *Settings → Keyboard → Input Sources* remove the old engine, or `sudo apt-get remove ibus-unikey ibus-bamboo` / `sudo dnf remove ibus-bamboo`; quit `xvnkb` |
| `W_CONFLICT_FCITX5_VI_IME` | Linux | Fcitx5 profile still contains `unikey` / `bamboo` / `m17n_vi_*` | `fcitx5-configtool` → remove the old IME, keep only **TextVN** |
| `W_IM_ENV_MISMATCH` | Linux | `GTK_IM_MODULE` / `QT_IM_MODULE` / `XMODIFIERS` point to a framework other than the running one (e.g. IBus running but `GTK_IM_MODULE=fcitx`) | Fix `~/.config/environment.d/*.conf`, `~/.xprofile` or `/etc/environment` to match (`ibus` ↔ `@im=ibus`, `fcitx` ↔ `@im=fcitx`), then log out and back in |

After quitting the old IME, run `--check-env` again to confirm the warning is gone.

## 3. Complete error-code table

| Code | Level | Source | Meaning | Action |
|---|---|---|---|---|
| `E_ABI_MISMATCH` | error | doctor (3 OSes) | FFI struct sizes differ between the CLI and the adapter | Reinstall one complete release (do not mix files from two versions) |
| `E_CONFIG_INVALID` | error | doctor | `config.json` violates the `config.v1` schema — engine runs defaults | Back up and delete `config.json`; TextVN recreates it |
| `E_CONFIG_UNREADABLE` | error | doctor | `config.json` cannot be read | Check permissions of the configuration directory |
| `E_TSF_UNREGISTERED` | error | doctor (Windows) | TSF TIP not registered / DLL no longer exists | Help → **[Cài & bật TSF]** (*install & enable TSF*), or the `TextVN-setup-<ver>-windows-x64-machine.exe` installer |
| `E_DEP_MISSING` | error | doctor + installer (Linux) | `ibus`/`libibus`/`fcitx5`/GTK missing or its ABI fails to load | `./install.sh --install-deps`, or the commands in §4 |
| `E_DEP_INSTALL_FAIL` | error | installer (Linux) | The package manager failed to install | Run the printed command manually, check `install-diag.log` |
| `E_BUILD_DEP_MISSING` / `E_BUILD_DEP_INSTALL_FAIL` | error | `scripts/install_linux.sh` | Missing tools/headers for building from source | Install `cmake`, `pkg-config`, `ibus-devel`/`libibus-1.0-dev`, `gtk4-devel` or `gtk3-devel` |
| `E_GLIBC_INCOMPAT` | error | installer (Linux) | `bin/textvn` fails to start on this machine (loader/glibc incompatibility — e.g. glibc < 2.28, older than RHEL 8) | Build from source: `./scripts/install_linux.sh`; prebuilt binaries support glibc ≥ 2.28 |
| `E_GTK_MISSING` / `W_GTK_MISSING` | error/warning | installer / doctor | The GUI settings window cannot open (neither GTK 4 nor GTK 3) — the IME still works | Install `gtk3` (EL8/SLES) or `gtk4` |
| `E_PKG_INVALID` | error | installer (Linux) | Tarball missing files / corrupt download | Download again, verify `SHA256SUMS.txt` |
| `E_KWIN_WAYLAND` | error | `textvn-portable.sh` (Linux) | KDE Plasma Wayland manages Fcitx5 through `WAYLAND_SOCKET` — the portable build cannot restart Fcitx5 without breaking KWin | Use `./install.sh` (per-user), or `./textvn-portable.sh --ibus` |
| `W_DEP_NO_IM_FRAMEWORK` | warning | doctor (Linux) | Neither `ibus-daemon` nor `fcitx5` found | Install IBus (GNOME/RHEL) or Fcitx5 (KDE) |
| `W_RECENT_PANIC` | warning | doctor (3 OSes) | `panic.log` contains `ERR_PANIC` records | Export a diagnostic bundle and report the bug |
| Conflict codes | warning | §2 | See the §2 table | — |

## 4. Linux: dependencies per distribution

`install.sh` detects `/etc/os-release` + the package manager and offers to install what is
missing: `./install.sh` (asks `[Y/n]`), `./install.sh --install-deps` (installs unattended),
`./install.sh --no-install-deps` (check only). It never force-installs Fcitx5 onto a GNOME
machine using IBus. Manual installation:

| Distribution | Command |
|---|---|
| RHEL 8/9 · Rocky · AlmaLinux · Fedora | `sudo dnf install -y ibus ibus-libs gtk3` |
| SLES 15 · openSUSE Leap | `sudo zypper install -y ibus libibus-1_0-5 libgtk-3-0` |
| Ubuntu · Debian · Linux Mint · Pop!_OS | `sudo apt-get install -y ibus libibus-1.0-5 libgtk-4-1` (Ubuntu 20.04 / Debian 11: `libgtk-3-0`) |
| Arch · Manjaro | `sudo pacman -S ibus gtk4` (or `fcitx5`) |

**RHEL 8.10 / Rocky 8 / AlmaLinux 8**: Fcitx5 is not in BaseOS/AppStream — use **IBus**
(GNOME's default). TextVN works on IBus right after installation, Fcitx5 is not required.

**Why do the Fcitx5 plugin and the GTK 4 settings window need glibc 2.35?** The tarball has
two baselines: `bin/textvn`, `textvn-ibus-engine` and `textvn-settings-gtk3` are built on
AlmaLinux 8 (**glibc ≤ 2.28** — the `cargo xtask check-elf-abi` gate rejects newer symbols)
and run on every distribution in the table. `libtextvn-fcitx5.so` and `textvn-settings-gtk4`
only matter on machines with Fcitx5 ≥ 5.0.14 / GTK 4 — those libraries only exist on glibc ≥
2.35 distributions, so they are built on Ubuntu 22.04. Older machines automatically use
IBus + GTK 3 (`bin/textvn-settings` selects it).

Real CI tests on every push: full E2E on RHEL 8.10 / AlmaLinux 8, RHEL 9 / AlmaLinux 9,
SLES 15 / openSUSE Leap 15.6, Ubuntu 22.04, Debian 12, Ubuntu 24.04, Fedora 41 and Arch
Linux; smoke tests (ldd, diagnostic CLI, `--check-abi`, per-user install/uninstall) on
Ubuntu 20.04 and Debian 11.

## 5. Windows: enterprise, LTSC, portable, administrator rights

| Situation | Behaviour / fix |
|---|---|
| No administrator rights | The `TextVN-setup-<ver>-windows-x64.exe` installer and the portable build only write `HKCU` — no UAC needed |
| Machine-wide deployment (IT) | `TextVN-setup-<ver>-windows-x64-machine.exe` (Program Files + `HKLM`), silent install: see [user-guide.md](user-guide.md) §*Cài đặt im lặng* (silent install) |
| Windows 11 Enterprise 23H2/24H2+ refuses to activate the per-user / portable build | Help → **[Cài & bật TSF]** reports `RegisteredNeedsManualActivate`; install the `-machine.exe` build |
| Windows 10 LTSC 2019/2021, Enterprise without Microsoft Store / WebView2 | Not needed: TextVN uses no WebView2 and no Store; the CRT is statically linked (`+crt-static`) so no VC++ Redistributable is required; Windows 11-only APIs are loaded dynamically (fallback on Windows 10 1507+) |
| Portable in `D:\Tools\TextVN`, on a USB stick, or in a folder with Vietnamese characters | Works without administrator rights. Configuration is stored in `%APPDATA%\TextVN`, logs in `%LOCALAPPDATA%\TextVN\logs` (see the note below) |
| Applications running as Administrator | TSF loads TextVN into elevated applications of the same account too. Only the Settings window: if an elevated TextVN instance is running, UIPI blocks a normal instance from opening it — TextVN reports this clearly; exit the elevated instance |
| SmartScreen / antivirus warning | No Authenticode signature yet (awaiting SignPath); see [antivirus-false-positive.md](specs/antivirus-false-positive.md) |

**Why does the portable build still store configuration in `%APPDATA%`?** A TSF input method
is a DLL loaded into **every application's process**, including AppContainer-sandboxed ones
(Edge, Store apps, the search box). Those processes cannot read arbitrary folders such as
`D:\Tools\TextVN` or a USB stick — if the configuration lived next to `TextVN.exe`, Vietnamese
typing would break in those applications. `%APPDATA%` belongs to the current user, is always
writable, needs no administrator rights and never touches `HKLM`. Removing the portable build:
right-click the tray icon → **Gỡ cài đặt** (*uninstall* — removes the `HKCU` TSF registration),
then delete the folder; `%APPDATA%\TextVN` is kept for reuse — delete it manually if wanted.

## 6. Reporting a bug

1. Export a diagnostic bundle (§1). 2. Windows: **[Copy tóm tắt lỗi]** → **[Mở trang báo lỗi]**.
3. Open an issue at <https://github.com/hunglinhpt/TextVN/issues>, paste the summary, describe
the application you were typing in, the key sequence, the actual and the expected result, then
attach the ZIP / `.md` file.
