# P3-5 — TRAY, CONFIG, IPC, PACKAGING & RELEASE (Linux) — Solution chi tiết

> WS5 · `textvn-tray` (crate dùng chung `P0-1`, nhánh Linux: **GTK4 Settings + StatusNotifier tray** —
> `PLAN §3.6/§4.1`: "GTK4 (Linux), cùng ui-model JSON-driven"). Mirror `P1-4` (Win) / `P2-4` (mac).
> Schema: `P0-3`. **Khác 2 OS: không self-update** (xem §7).

## 1. Tray — StatusNotifier/AppIndicator (`PLAN §3.6`)

```text
textvn-tray (1 instance — flock ~/.config/TextVN/tray.lock):
  D-Bus org.kde.StatusNotifierWatcher (đăng ký name, làm owner nếu host không có — fallback
  libayatana-appindicator khi SNI watcher không tồn tại, verify spike LNX-050)
  menu (mirror P1-4 §1 / P2-4 §1 — 9 mục):
    Bật/Tắt gõ tiếng Việt · Chế độ gõ · Dấu · App đang gõ + Enable app này ·
    Chọn framework (ibus/fcitx5/auto — P3-2 §7) · Cài đặt… · Sức khỏe · Gỡ cài đặt… · Thoát
  icon: pixel-art template (adwaita-compatible) — on/off/error badge
  "Thoát" = đóng socket + dừng textvn-x11 (nếu đang chạy); KHÔNG kill daemon của desktop
```

- Autostart: **systemd user unit** `textvn-tray.service` (`systemctl --global enable` ở postinst,
  `PLAN §3.7`) + fallback `~/.config/autostart/textvn-tray.desktop` khi không có systemd --user.

## 2. IPC server (mirror `P1-4 §2`, `P2-4 §2`)

```text
socket: ~/.config/TextVN/ipc.sock   (bind 0600, dir 0700 — P0-3 §5 bảng per-OS)
Codec/message set: giống hệt (u32 length + UTF-8 JSON, schema ipc.v1.md) — không bịa thêm
Server: textvn-tray · Client: ibus engine / fcitx5 addon / textvn-x11
Auth: SO_PEERCRED uid == current uid + check path client thuộc package (mirror P0-3 §5)
Watcher: config/appdb/state (debounce 300ms) → broadcast ConfigReload/StateUpdate
Offline: client đọc file — không block (P0-3 §4)
```

## 3. Settings — GTK4 (cửa sổ 1, sidebar 6 tab — parity `P1-4 §3`, `P2-4 §3`)

| Tab | Nội dung (đối chiếu) |
|---|---|
| General | `enabled_default`, method, typo options, `free_marking`, `spellcheck`, language |
| Applications | `app_overrides` + `ignore_apps` + "Thêm app đang chạy" + preset override |
| App-compat (riêng Linux) | Bật AT-SPI (a11y), framework `ibus/fcitx5/auto`, opt-in `textvn-x11` (X11), hiện trạng thái env matrix |
| Hotkeys | Toggle EN/VN, sửa, check conflict |
| Update & About | Kiểm tra bản mới (§7 — **mở trang PM thay vì tự update**), changelog, export diagnostics, version |

- Sinh từ **cùng `ui-model` JSON** (ADR-004) → `docs/release/parity-checklist.md` điền đủ
  (không bịa số control — như P1/P2).
- Mọi thay đổi → svc layer → file → broadcast (một nguồn ghi).

## 4. IPC/state/health (mirror `P2-4 §6`)

```text
mọi client ghi heartbeat: ~/.config/TextVN/heartbeat/<name>.json (5s, atomic rename)
tray check 10s: heartbeat cũ >15s → hiện "Engine chưa hoạt động" + nút "Khởi động lại"
  (ibus: spawn lại engine process / ibus restart nhẹ; fcitx5: hướng dẫn restart fcitx5;
   x11: kill + spawn lại — watchdog < 500ms)
Shutdown tray: đóng socket; client nhận EOF → offline mode (không mất gõ)
```

## 5. Cài đặt & gỡ — `.deb` (primary)

```text
textvn_1.x.y_amd64.deb
  /usr/share/ibus/component/textvn.xml          + /usr/lib/textvn/textvn-ibus-engine
  /usr/share/fcitx5/addon/textvn.conf           + /usr/lib/fcitx5/libtextvn-fcitx5.so
  /usr/bin/textvn (CLI), /usr/bin/textvn-x11, /usr/lib/textvn/textvn-tray
  /usr/share/applications/textvn-settings.desktop
  /usr/lib/systemd/user/textvn-tray.service
  /usr/share/doc/textvn/{README,compat,security}…
  /usr/share/metainfo/textvn.appdata.xml        (AppStream — required for good distros)

postinst: ibus restart (ignore fail nếu không chạy); systemctl --global enable textvn-tray
           ; update-desktop-database; update-mime-database (nếu có)
prerm/postrm: ibus restart; systemctl --global disable; xóa mọi file đã cài
gỡ sạch: 0 residue system (P3-6 §6 script check)
config user: KHÔNG đụng khi gỡ (giữ — S9); chỉ `purge` hỏi (textvn purge --yes xóa
             ~/.config/TextVN + ~/.local/state/TextVN sau khi xác nhận)
```

## 6. `textvn doctor` (bản Linux — `PLAN §5.3`)

```text
textvn doctor [--export]
  1. env matrix (P3-4 §7): GTK_IM_MODULE / QT_IM_MODULE / XMODIFIERS — đề nghị fix
  2. session: XDG_SESSION_TYPE=x11|wayland → giải thích giới hạn textvn-x11 (B10)
  3. framework: ibus-daemon / fcitx5 đang chạy? component xml + addon đã cài?
  4. a11y: org.a11y.Status.IsEnabled + socket AT-SPI
  5. socket ipc + heartbeat 3 client (ibus/fcitx5/x11)
  6. version, config (redact), caps/owner hiện tại
  --export: zip log + doctor output — KHÔNG text content (S2, test grep)
```

## 7. Update — **không self-update** (quyết định riêng của Linux)

| Việc | Cách |
|---|---|
| Kiểm tra bản mới | GitHub API `releases/latest` (như `P1-4 §7` bước 1) |
| Thông báo | Tray hiện "Đã có vX.Y.Z (gói của bạn: 1.x.y)" + nút **mở trang release / trang package** |
| Áp dụng | Người dùng chạy `sudo apt upgrade` / `sudo dnf upgrade` / `paru -Syu` (AUR) — **tray không tự thay file `/usr`** |
| Lý do | Không ghi đè file đã cài bởi PM (không phá dpkg/rpm db, không cần root, không cần chữ ký Ed25519 riêng — PM đã ký repo) |

> Win/mac vẫn dùng updater Ed25519 (`P1-4 §7`, `P2-4 §7`) — bất biến FFI/schema không đổi.

## 8. Phân phối

| Kênh | Nội dung | Trạng thái |
|---|---|---|
| GitHub Releases | `.deb` + `.rpm` + `SHA256SUMS` + notes tiếng Việt (hướng dẫn thêm input source) | M5 (L4) |
| AUR | `textvn-bin` (binary từ release) + `textvn` (source build) — `packaging/linux/aur/` | Sau 2 release ổn định |
| Distro repo | Debian/Ubuntu/Fedora/Arch: backlog (AppStream sẵn sàng) | Backlog |

## 9. Task (chi tiết `P3-7-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| LNX-050 | Tray SNI + menu 9 mục + lock instance | 9/9; 2 instance → 1 hiện cửa sổ đang mở |
| LNX-051 | IPC server + watcher + health §4 | `textvn ipc probe` thấy 3 client; kill engine → hiện lỗi + restart được |
| LNX-052 | Settings GTK4 6 tab + parity checklist | `parity-checklist.md` đủ PLAN §2.3 (M6) + §8 |
| LNX-053 | systemd user unit + config init + hot-reload | Đổi file → <1s; file sai schema → giữ bản cũ + warning |
| LNX-054 | `.deb` cài/gỡ sạch §5 | Ubuntu VM: cài/gỡ = 0 residue system; config giữ (S9) |
| LNX-055 | `doctor` full §6 | Test 4 tình huống env sai/wayland/a11y off/socket chết → đề nghị đúng |
| LNX-056 | Update-notify §7 + `.rpm` + AUR | Notify đúng version; `dnf`/AUR install pass trên VM |
