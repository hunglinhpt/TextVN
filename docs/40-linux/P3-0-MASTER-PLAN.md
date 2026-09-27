# P3-0 — LINUX MASTER PLAN (Phần 3)

> Điều kiện tiên quyết: Phần 2 đạt 2/2 (`../30-macos/P2-REVIEW-LOG.md`).
> Tài liệu này = kế hoạch chi tiết phần Linux. Solution chi tiết ở `P3-1 … P3-7`;
> nhận việc theo task `LNX-xxx` trong `P3-7-TASKS.md`.
> Quyết định nền tảng: **ADR-007** — dual adapter **IBus + Fcitx5**; **không grab phím trên Wayland**
> (bug B10 của bamboo-viet) — xem `adr/README.md` + `PLAN §5.2/§5.3`.

## 1. Mục tiêu phần Linux (exit condition)

**Phần 3 xong khi:**
1. Gõ Telex/VNI/VIQR chuẩn: `corpus/shared` + `corpus/linux` ≥ 300 case pass (`--adapter linux`).
2. **App matrix Linux 20 app ≥ 95%** (B1/B2/B6/B8/B10/B13 không còn open) — `P3-6 §3`.
3. Cả 2 adapter cùng sống: IBus **và** Fcitx5 cài được, gõ được, không xử lý đôi.
4. Wayland (GNOME/KDE) gõ tốt qua framework chuẩn; X11 fallback hoạt động cho app legacy (opt-in).
5. Đóng gói `.deb` (primary) + `.rpm` + AUR; `textvn doctor` chẩn đoán env (`GTK_IM_MODULE`…).
6. CI `ci-linux.yml` xanh trên matrix distro (Ubuntu LTS, Fedora, Arch).
7. Review 2/2 của Phần 3 hoàn tất.

**Không nằm trong Phần 3:** AI suggest, Hán-Nôm, snap/Flatpak cho IME (loại — không cung cấp
input method cho system được), global hook cài sẵn (`PLAN §3.7`: "không cài global hook").
**Bất biến:** FFI (`P0-2`), schema (`P0-3`), corpus (`P0-4`) — mọi thay đổi phải bump đúng `P0-2 §6`.

## 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`)

```
┌─ Wayland / X11 text app (GTK, Qt, Chromium, Electron…) ─────────────────────┐
│  ┌── ibus-daemon ── spawn ──► textvn-ibus-engine (process riêng, C)        │
│  │     IBusEngine.process_key_event → ime_key() → preedit/commit/surrounding │
│  └── fcitx5 ── load addon ──► libtextvn-fcitx5.so (trong process fcitx5, C++)│
│        keyEvent → ime_key() → preedit/commit (instance per InputContext chung │
│        main loop — 1 instance, reset theo context)                           │
│     Cả 2 link: libtextvn-linux-common.a (AT-SPI field detect + helpers)     │
│              + libtextvn_ffi.a (engine) → ipc client unix socket            │
└──────────────────────────────────────────────────────────────────────────────┘
┌─ X11 legacy / game (opt-in, KHÔNG chạy mặc định, KHÔNG dùng trên Wayland) ──┐
│  textvn-x11 (process riêng, Rust, spawn + watchdog bởi tray)                │
│    XGrabKeyboard khi app mục tiêu focus → ime_key() (instance riêng/tuần)    │
│    → inject bằng XTEST (marker chống loop)                                   │
└──────────────────────────────────────────────────────────────────────────────┘
┌─ textvn-tray (GTK4 + StatusNotifier — SOURCE OF TRUTH) ─────────────────────┐
│  • tray menu + Settings GTK4 (ui-model JSON chung) • IPC server (unix socket) │
│  • watcher config/appdb/state • systemd user service (autostart)              │
│  • doctor (env vars, component xml, addon) • spawn/kill textvn-x11           │
│  • updater: KHÔNG self-update — báo bản mới + mở trang package manager       │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Đối chuẩn & Kế thừa Tinh hoa từ các Repo Mã nguồn mở Hàng đầu

Nhằm xây dựng giải pháp gõ tiếng Việt trên Linux toàn diện và mượt mà nhất, kiến trúc TextVN được đối chuẩn và kế thừa tinh hoa từ 5 dự án mã nguồn mở chủ chốt:

| Dự án tham chiếu | Kiến trúc & Công nghệ | Điểm mạnh kế thừa | Giới hạn & Giải pháp nâng cấp của TextVN |
|---|---|---|---|
| **fcitx/fcitx5** (Official Upstream) | Framework bộ gõ hiện đại thế hệ mới của Linux; hỗ trợ Wayland (`text-input-v3`), C++ C-ABI, `InputMethodEngineV2`, `CapabilityFlags`. | • Kiến trúc module hóa cực cao, độ trễ tiệm cận 0ms.<br>• Hỗ trợ đầy đủ các cờ `CapabilityFlag::SurroundingText`, `CapabilityFlag::FormattedPreedit`, `CapabilityFlag::ClientSideInputPanel`.<br>• Giao thức Wayland chuẩn xác không cần snoop phím. | TextVN tuân thủ 100% chuẩn C++ Addon API của Fcitx5, tận dụng khả năng probe `CapabilityFlags` để điều tiết chế độ hiển thị linh hoạt theo từng cửa sổ. |
| **fcitx5-lotus** (`LotusInputMethod/fcitx5-lotus`) | Bộ gõ tiếng Việt cho Linux tối ưu hóa từ VMK, hướng đến trải nghiệm **gõ không gạch chân (Non-preedit) mượt mà**, tích hợp `uinput` virtual keyboard. | • **Gõ không gạch chân (Non-preedit):** Loại bỏ hoàn toàn đường gạch chân khó chịu khi gõ, chữ xuất hiện tự nhiên như gõ phím trực tiếp.<br>• **Hỗ trợ `uinput`:** Cho phép gửi phím cấp kernel ảo đối với các app/game không hỗ trợ IM framework.<br>• Tránh giật con trỏ và xung đột giao diện. | TextVN kế thừa triết lý **Gõ không gạch chân (Non-preedit Mode)**: Khi ứng dụng hỗ trợ Surrounding Text (`CapabilityFlag::SurroundingText`), TextVN thực hiện Direct Commit + `deleteSurroundingText`, mang lại trải nghiệm mượt mà không gạch chân; đồng thời cung cấp tùy chọn `uinput` cho Game/Wine. |
| **BambooMintKey** (`thatislg/BambooMintKey`) | Fcitx5 C-ABI Addon (`libbamboomintkey.so`), build CMake, cài đặt rootless `~/.local` hoặc system `/usr`. | • **Rootless per-user install:** Cho phép user/dev cài đặt ngay không cần `sudo`.<br>• **Script cài đặt 1 lệnh (`install_linux.sh`):** Tự phát hiện distro (Ubuntu, Fedora, Arch) và hot-reload `fcitx5 -r -d`.<br>• **Badge icon SVG vector:** Sắc nét ở mọi độ phân giải. | TextVN kế thừa trọn vẹn mô hình rootless + system dual path và script cài 1 chạm, đồng thời bổ sung adapter IBus để hỗ trợ mặc định cho Ubuntu GNOME. |
| **ibus-bamboo** (`bambooengine/ibus-bamboo`) | IBus Engine truyền thống viết bằng C/Go, hỗ trợ preedit và surrounding text mode. | • **Kỹ thuật Surrounding text:** Xử lý `ibus_engine_delete_surrounding_text` chuẩn xác khi ứng dụng hỗ trợ.<br>• **Phân định rõ ranh giới Wayland:** ibus-bamboo ghi nhận Fcitx5 vượt trội hơn IBus trên Wayland (`text-input-v3` không lệch con trỏ). | TextVN thiết kế Fcitx5 làm adapter khuyến nghị cho Wayland (KDE/Fedora/Arch/Ubuntu Fcitx5) và IBus làm native cho GNOME Ubuntu; dùng chung core Rust FFI. |
| **OpenKey** (`tuyenvm/OpenKey`) | Hook toàn cục (Windows: `SetWindowsHookEx`, macOS: `CGEventTap`, Linux: `XRecord`/`XTest`/`XGrabKeyboard`) + backspace simulation. | • **Bộ giải thuật gõ tiếng Việt nhanh gọn:** Tự phục hồi từ tiếng Anh, kiểm tra chính tả. | • **Thất bại trên Wayland:** Cơ chế hook toàn cục của OpenKey **hoàn toàn vô hiệu trên Wayland** do kiến trúc bảo mật cô lập input giữa các client.<br>• **Giải pháp TextVN:** Tuyệt đối **không dùng global hook trên Wayland** (fix triệt để bug B10); đi chuẩn giao thức Input Method (Fcitx5 + IBus); `textvn-x11` chỉ là tiến trình riêng biệt opt-in cho game/legacy X11. |

### 2.2 Bảng Phòng chống Triệt để các Bug Lịch sử (Bug Prevention Matrix)

| Mã Bug | Hiện tượng gặp phải trên các bộ gõ cũ | Nguyên nhân gốc rễ | Giải pháp Triệt để của TextVN |
|---|---|---|---|
| **Underline Bug** (Gạch chân giật cục) | Khi gõ từ có dấu, đường gạch chân dashed/solid liên tục co giãn, làm giật con trỏ và nhảy layout trong Discord, LibreOffice, Web browsers. | Bộ gõ luôn gán cờ `TextFormatFlag::Underline` vào chuỗi composition preedit, kích hoạt liên tục reflow layout của toolkit UI. | **Chế độ Gõ Không Gạch Chân (Non-preedit Mode — kế thừa Lotus):**<br>1. Nếu app hỗ trợ `SurroundingText`: TextVN commit thẳng ký tự và xóa lùi bằng `deleteSurroundingText(-len, len)` → Không gạch chân, không popup, mượt mà 100%.<br>2. Nếu app cần preedit: Bỏ cờ `Underline` (`setClientPreedit` trơn không gạch chân). |
| **Bug B1** (Autocomplete dropdown nhảy loạn) | Gõ URL trên thanh địa chỉ Firefox/Chrome bị mất chữ, nhảy về đầu dòng, hoặc gợi ý tìm kiếm nhấp nháy liên tục. | Bộ gõ phát sinh phím Backspace giả lập khiến thanh địa chỉ hủy cache gợi ý và reset vị trí con trỏ caret. | Áp dụng chiến lược `SelectionReplace` (thay thế trực tiếp vùng chọn) hoặc Direct Commit; tuyệt đối không gửi phím Backspace giả lập vào thanh địa chỉ. |
| **Bug B2** (Lặp từ cuối khi Enter) | Đang gõ dở trong Messenger, Telegram, Discord, nhấn Enter để gửi tin nhắn thì từ cuối cùng bị nhân đôi (ví dụ `xin chào chào`). | Khi phím Enter đến, bộ gõ vừa commit chuỗi preedit còn tồn đọng vừa cho phím Enter đi qua ứng dụng, dẫn đến ứng dụng nhận text 2 lần. | Thực thi nguyên tắc **Commit-Before-Hide**: Khi Enter nhấn xuống, nếu đang có buffer chưa commit thì commit ngay và nuốt/hủy sự kiện Enter nếu là xác nhận từ; trong chế độ Non-preedit, text đã nằm trong văn bản từ trước nên Enter gửi đi an toàn tuyệt đối. |
| **Bug B6** (Nuốt nhầm hotkey hệ thống) | Không thể bấm `Alt+Tab`, `Super+D`, `Ctrl+Shift+T` khi đang bật bộ gõ tiếng Việt. | Bộ gõ chặn và kiểm tra toàn bộ key-down mà không bỏ qua các modifier hệ thống. | Bộ lọc sớm ngay dòng đầu của `process_key_event`: Nếu có `Super`, `Alt` hoặc `Ctrl` đi kèm các phím điều hướng/hệ thống → Trả về `PASS` (false) ngay lập tức, không qua engine. |
| **Bug B8 & B11** (Hỏng phím trong Terminal) | Gõ tiếng Việt trong Terminal (`gnome-terminal`, `konsole`, `kitty`, `alacritty`) bị vỡ mã ANSI escape hoặc kẹt ký tự lạ. | Preedit inline xung đột với cơ chế raw pty của terminal emulator. | Preset tự động nhận diện vai trò `TERMINAL` qua AT-SPI hoặc tên ứng dụng, tự động chuyển sang chiến lược `ForwardAsCommit` hoặc Non-preedit không buffer. |
| **Bug B10** (Mất gõ / Treo trên Wayland) | Ứng dụng chạy trên Wayland (Ubuntu 22.04+, Fedora 38+) không nhận tiếng Việt hoặc bộ gõ tự crash. | Bộ gõ cố gắng gọi hàm X11 `XGrabKeyboard` hoặc `XRecord` trong phiên Wayland, bị compositor chặn vì lý do an ninh. | Tuyệt đối không gọi hook X11 trên Wayland; sử dụng Fcitx5 C-ABI Addon tích hợp qua giao thức Wayland native `text-input-v3` / `wayland-im`. |
| **Bug B13** (Xung đột đa cửa sổ / Multi-window bleed) | Gõ một nửa từ ở app A rồi chuyển sang app B (Alt+Tab), từ gõ dở bị chèn vào app B. | Bộ gõ dùng chung 1 engine instance / bộ đệm duy nhất cho mọi cửa sổ. | Thiết kế **1 instance per InputContext (`IC* → ime_instance*`)**: Mỗi cửa sổ có bộ đệm riêng biệt; sự kiện `focus_out` tự động kích hoạt commit chuỗi hiện tại và reset bộ đệm của context đó. |

**Đường dẫn chuẩn Linux (System vs Rootless Per-User):**

| Hạng mục | Đường dẫn System (`/usr`) | Đường dẫn Rootless User (`~/.local`) |
|---|---|---|
| Config/state/appdb | `~/.config/TextVN/{config.json, state.json, appdb.json}` | `~/.config/TextVN/{config.json, state.json, appdb.json}` |
| IPC socket | `~/.config/TextVN/ipc.sock` (0600 — đúng user, `P0-3 §5`) | `~/.config/TextVN/ipc.sock` (0600 — đúng user, `P0-3 §5`) |
| Log | `~/.local/state/TextVN/log/` | `~/.local/state/TextVN/log/` |
| IBus component | `/usr/share/ibus/component/textvn.xml` | `~/.local/share/ibus/component/textvn.xml` |
| IBus engine binary | `/usr/lib/textvn/textvn-ibus-engine` | `~/.local/lib/textvn/textvn-ibus-engine` |
| Fcitx5 addon config | `/usr/share/fcitx5/addon/textvn.conf` | `~/.local/share/fcitx5/addon/textvn.conf` |
| Fcitx5 addon `.so` | `/usr/lib/fcitx5/libtextvn-fcitx5.so` | `~/.local/lib/fcitx5/libtextvn-fcitx5.so` |
| Icon SVG (V/E badge) | `/usr/share/icons/hicolor/scalable/apps/` | `~/.local/share/icons/hicolor/scalable/apps/` |
| Tray/Settings desktop | `/usr/share/applications/textvn-settings.desktop` | `~/.local/share/applications/textvn-settings.desktop` |
| Autostart desktop | `/etc/xdg/autostart/textvn-tray.desktop` | `~/.config/autostart/textvn-tray.desktop` |
| Binaries (`textvn`, `textvn-tray`) | `/usr/bin/` | `~/.local/bin/` |

> Quy tắc: Bộ cài `scripts/install_linux.sh` hỗ trợ cả 2 chế độ:
> - Mặc định (hoặc truyền flag `--user`): cài rootless vào `~/.local/`, **không đòi hỏi quyền root / sudo**.
> - Truyền flag `--system`: cài vào `/usr/` thông qua `sudo` hoặc package manager (`.deb`, `.rpm`, AUR).


## 3. Workstream & file solution

| WS | Tên | File solution | Sản phẩm |
|---|---|---|---|
| WS1 | IBus adapter (primary) | **`P3-1-ibus.md`** | `textvn-ibus-engine` gõ Telex trong gedit (demo M1) |
| WS2 | Fcitx5 addon (dual) | **`P3-2-fcitx5.md`** | addon chạy trên fcitx5 ≥ 5.0.x |
| WS3 | X11 fallback (opt-in) | **`P3-3-x11.md`** | `textvn-x11` cho app legacy/game X11 |
| WS4 | AT-SPI field detect + AppDB | **`P3-4-strategy-appdb.md`** | rules AT-SPI + 20 preset Linux + doctor env |
| WS5 | Tray/Settings/IPC/Package | **`P3-5-ui-packaging-release.md`** | `textvn-tray` GTK4 + `.deb/.rpm/AUR` |
| WS6 | Test & automation | **`P3-6-test-plan.md`** | AT-SPI driver + matrix + CI 3 distro |
| WS7 | Task & điều phối | **`P3-7-TASKS.md`** | LNX-001…LNX-066 |

## 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3)

| Milestone | Tuần | Nội dung | Exit criteria |
|---|---|---|---|
| **L0 – Spike** | 1–2 | `LNX-001..009`: IBus tối thiểu, link FFI (C/C++), keyval/shift, surrounding/delete, AT-SPI permission, fcitx5 API pin, XGrab+XTEST, GHA xvfb | Bảng spike **18 mục** (`P3-1 §9` = 10 + `P3-2 §8` = 8) trong `docs/specs/linux-*.md` |
| **L1 – Text correctness** | 3–6 | IBus composition + engine full + `corpus/linux` ≥ 150 case | replay xanh; gedit/Safari-equivalent (Firefox) gõ đúng |
| **L2 – Compat layer** | 7–10 | AT-SPI detect + preset 20 app + Fcitx5 addon + env matrix + **X11 fallback opt-in** (`LNX-040..044`) | 12 app matrix ≥ 90%; corpus `bug_B*` Linux pass; Wayland guard đúng (B10) |
| **L3 – UX & packaging** | 11–14 | Tray GTK4/Settings/IPC/.deb + doctor | Cài `.deb` trên Ubuntu VM sạch; parity checklist Linux điền đủ |
| **L4 – Hardening** | 15–18 | Fuzz, soak, perf, matrix 3 distro, `.rpm`+AUR | KPI `PLAN §5.5` (Linux) đạt; 20 app ≥ 95% |
| **L5 – RC Linux** | 19–20 | Security + license + docs | Cổng ra `P3-6 §6` |

## 5. Dependency graph

```
LNX-001 (env: Rust + ibus dev + fcitx5 dev + xvfb)
  ├─► LNX-002 (spike IBus minimal) ─► LNX-010..019 (IBus core)
  ├─► LNX-003 (spike link FFI từ C/C++) ─► mọi adapter
  ├─► LNX-004 (spike surrounding/delete_surrounding + selection) ─► LNX-014/015
  ├─► LNX-005 (spike AT-SPI + a11y permission) ─► LNX-030..031
  ├─► LNX-006 (spike fcitx5 API + version pin) ─► LNX-020..025
  ├─► LNX-007 (spike XGrabKeyboard + XTEST + Wayland detect) ─► LNX-040..044
  ├─► LNX-008 (spike GHA xvfb + AT-SPI trên runner) ─► RM5
  └─► LNX-009 (corpus linux đầu) ─► mọi task reproduce-trước-fix
LNX-032..035 (preset 20 app) sau LNX-030
LNX-050..056 (tray/package) song song từ tuần 6 (IPC schema chốt P0-3)
LNX-060..066 (harness/test) bắt đầu tuần 3, chặn release
```

## 6. Rủi ro & mitigation (riêng Linux)

| # | Rủi ro | P/I | Mitigation |
|---|---|---|---|
| RL1 | **keyval/shift semantics của IBus** chưa rõ (keyval đã apply modifier chưa?) → sai phím với layout có shift | Trung/Cao | Spike `LNX-004`: ghi lại hành vi thật với 3 layout (us, vi, viqr typewriter); fallback translate bằng `xkbcommon` (weak feature detection — `PLAN §5.3`) |
| RL2 | **Fcitx5 addon API đổi giữa các bản** (V2/V3/V4) | Trung/Trung | Spike `LNX-006` pin tối thiểu `fcitx5-dev >= 5.0.14` (ho số thật từ spike), `#if FCITX_VERSION` cho API khác; CI matrix Ubuntu LTS + Fedora + Arch |
| RL3 | **Wayland không grab được phím** (B10) → app không đi IM framework không gõ được | Cao/Trung | Theo `PLAN`: đi chuẩn IBus/Fcitx5; X11 module tách riêng (`P3-3`), document giới hạn trong README (app Wayland-native không qua framework = out of scope) |
| RL4 | **AT-SPI bị từ chối** (GNOME a11y disabled) → mất field detect | Trung/Trung | `org.a11y.Status` check + hướng dẫn bật trong Settings; từ chối → fallback preset theo WM_CLASS (vẫn đúng phần lớn) — `P3-4 §4` |
| RL5 | `delete_surrounding_text` hỗ trợ lệch (GTK ✓, Qt ⚠, Chromium ⚠) → BackspaceType chập chờn | Trung/Trung | capability probe khi `focus_in` (IBus báo `surrounding_text` mode); không hỗ trợ → downgrade strategy theo quy tắc `P0-3 §3.1` (caps) |
| RL6 | Đóng gói đa distro (glibc, system paths, systemd user unit) | Trung/Trung | `.deb` primary; spec paths tách `packaging/linux/`; test cài trên Ubuntu 22.04/24.04 + Fedora + Arch (CI matrix) |
| RL7 | Engine process chết làm mất gõ giữa chừng | Trung/Trung | IBus: engine process riêng → ibus-daemon respawn; fcitx5: addon crash = fcitx5 crash → `catch` mọi callback + soak 24h; tray hiện heartbeat |
| RL8 | **XTEST/opt-in bị coi là keylogger** | Trung/Cao | Không cài mặc định (`PLAN §3.7`), mô tả quyền rõ (`docs/security/x11-grant.md`), không chạy trên Wayland, blocklist game sẵn |
| RL9 | Đụng `GTK_IM_MODULE`/`QT_IM_MODULE`/`XMODIFIERS` sai → app không thấy IM | Trung/Trung | `textvn doctor` detect + đề nghị fix từng app (dùng `PLAN §5.3` env matrix); preset ghi chú env cần thiết |
| RL10 | Link **staticlib Rust** vào C/C++ addon (symbol conflicts với fcitx5, `-fno-exceptions`?) | Trung/Trung | Spike `LNX-003`: `libtextvn_ffi.a` + `linux-common` vào C và C++ smoke; fallback `cdylib` (`libtextvn_ffi.so`) — FFI giữ nguyên (`P0-2`) |

## 7. Definition of Done cho PHẦN 3

- [ ] Mọi task `LNX-*` trong `P3-7-TASKS.md` đạt DoD 7 mục (Handbook §4).
- [ ] `P3-6 §6` release gate pass.
- [ ] `docs/compat.md` cộng số liệu Linux (20 app, 3 distro).
- [ ] `P3-REVIEW-LOG.md` đạt 2/2 → `00-INDEX` cập nhật.
- [ ] 0 finding `blocker/major` mở; RL1–RL10 có owner + trạng thái.
- [ ] Bàn giao: FFI/schema không đổi (hoặc bump đúng `P0-2 §6`); roadmap3 OS hoàn tất.
