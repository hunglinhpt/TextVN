# P2-4 — UI, Config, IPC, Packaging & Release (macOS) — Solution chi tiết

> WS4 · `TextVN.app` (NSStatusItem + SwiftUI — `PLAN §3.6`: "NSStatusItem SwiftUI menu",
> `PLAN §4.1`: SwiftUI Settings, cùng `ui-model` JSON với egui/GTK4 — ADR-004).
> Mirror `../20-windows/P1-4-ui-packaging-release.md`; schema: `P0-3`.

## 1. Status menu — `TextVN.app` (LSUIElement, 1 instance)

- **Single instance:** `NSRunningApplication` check + distributed notification `vn.textvn.awake`
  → instance cũ hiện Settings (mirror mutex Windows).
- **Login item:** `SMAppService.mainApp.register()` (macOS 13+; fallback `SMLoginItemSetEnabled`
  cho 11–12 — ghi version gate trong code). User toggle trong Settings → không tự bật khi cài (PLAN: opt-in).

**Menu (mirror `P1-4 §1` — 9 mục):**

| Mục | Hành động |
|---|---|
| Bật/Tắt gõ tiếng Việt | svc → `state.json` → broadcast `StateUpdate` |
| Chế độ gõ (Telex/VNI/VIQR/…) | svc → `config.json` → broadcast `ConfigReload` |
| Dấu (đậm/nghiêng/…) | submenu `typo_*` |
| App đang gõ + Enable app này | state.json per-app |
| Quyền Accessibility (nếu tap/AX cần) | mở System Settings đúng pane (`x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`) |
| Cài đặt… | mở cửa sổ SwiftUI (§3) |
| Sức khỏe | IMK PID + heartbeat, socket, AX/tap state, version |
| Gỡ cài đặt… | `textvn uninstall` (§4) — hỏi giữ config |
| Thoát | Đóng socket server; **không** kill IMK (system quản lý) |

- Badge: `vn-on` / `vn-off` / `error` (crash counter > 0) — template image SF Symbols (`keyboard`,
  `keyboard.badge`…), không dùng icon bitmap lệch theme.

## 2. IPC server (mirror `P1-4 §2`) — unix socket

```text
socket: ~/Library/Application Support/TextVN/ipc.sock  (bind 0600, dir 0700 — đúng user, tinh thần DACL P0-3 §5)
Codec:  giống hệt Windows (u32 length prefix + UTF-8 frame JSON, schema ipc.v1.md)
Message set = P0-3 §5 (Hello/GetSnapshot/Subscribe/ToggleViEn/CrashReport/Ping) — KHÔNG bịa thêm
Server: TextVN.app · Client: TextVN-IM.app (IMK + tap)
Watcher: config/appdb/state (debounce 300ms) → broadcast ConfigReload/StateUpdate
Offline: client đọc file khi activate — không block (P0-3 §4)
```
- Transport khác Windows (named pipe → unix socket) nhưng **schema/policy giữ nguyên** —
  bảng per-OS trong `P0-3 §5` (đã bổ sung review Phần 2 — finding F2-006).
- Auth: ngoài `SO_PEERCRED`-style check uid == current user, thêm check path client nằm trong
  `/Applications/TextVN.app` hoặc `~/Library/Input Methods/` (mirror `P0-3 §5`).

## 3. Settings — SwiftUI (cửa sổ 1, sidebar 6 tab — parity với `P2-0`/PLAN §2.3 M6)

| Tab | Nội dung (đối chiếu `P1-4 §3`) |
|---|---|
| General | `enabled_default`, method, typo options, `free_marking`, `spellcheck`, language (vi/en) |
| Applications | `app_overrides` + `ignore_apps` + "Thêm app đang chạy" + preset override |
| App-compat (riêng mac) | Bật AX permission, tap opt-in per-app, hiện trạng thái TCC |
| Hotkeys | Toggle EN/VN, sửa, check conflict |
| Update & About | Channel, kiểm tra cập nhật, changelog, export diagnostics, version |

- UI sinh từ **cùng `ui-model` JSON** (điều kiện ADR-004) → parity checklist điền vào
  `docs/release/parity-checklist.md` (giống `P1-4 §3`, không bịa số control).
- Mọi thay đổi → svc layer in-process → file → broadcast (một nguồn ghi — mirror `P1-4 §3`).

## 4. Cài đặt & gỡ — `.pkg` (per-user, không sudo — PLAN §3.7)

```text
textvn-mac.pkg (productbuild, component plist "relocatable=false"):
  1. /Applications/TextVN.app                      (settings/menu bar)
  2. ~/Library/Input Methods/TextVN-IM.app         (per-user payload — pkg dùng "Enable component" install scheme
                                                     hoặc script postinstall copy nếu pkg system-wide bắt buộc;
                                                     spike MAC-007 chốt — nếu pkg chỉ cài system-wide → postinstall script
                                                     cp -R vào ~/Library/Input Methods + chown user)
  3. postinstall: đăng ký input source (theo cách MAC-007 chốt) + in hướng dẫn:
     "System Settings → Keyboard → Input Sources → Add TextVN"
  4. không tự bật login item (user tự bật trong Settings)

uninstall (textvn uninstall):
  1. gõ lệnh unregister input source (nếu API) + xóa ~/Library/Input Methods/TextVN-IM.app
  2. xóa /Applications/TextVN.app, login item (nếu đang bật)
  3. hỏi "Giữ config ở ~/Library/Application Support/TextVN?" (mặc định GIỮ — S9)
  4. log xóa: ~/Library/Logs/TextVN (hỏi nốt) — 0 residue: check bằng script tools/mac/uninstall-check.sh
```

**Flow build thật hiện có trong repo — 3 script nối tiếp:**

```text
scripts/build-macos.sh            # cargo + swift (universal fat qua lipo) → dist/macos/stage
                                  # · ký per-component với entitlements thật (KHÔNG --deep — RM3)
                                  #   identity = $DEVELOPER_ID hoặc ad-hoc `-` khi dev
                                  # · .icns tự sinh từ PNG nếu chưa có icon; bundle thêm
                                  #   uninstall_macos.sh + uninstall-check.sh
scripts/package-macos-pkg.sh      # pkgbuild + productbuild → dist/macos/TextVN-mac-v<ver>.pkg
   --rebuild                      #   chạy build-macos.sh trước
   --notarize                     #   ký .pkg (Developer ID Installer) + notarize + staple
scripts/notarize-macos.sh         # notarytool submit --wait + stapler staple (gọi riêng cũng được)

Cài per-user (KHÔNG sudo):  installer -pkg dist/macos/TextVN-mac-v<ver>.pkg -target CurrentUserHomeDirectory
Cài toàn máy (admin, VM test): sudo installer -pkg … -target /
→ packaging/macos/distribution.xml chốt 2 scheme; pkg-scripts/postinstall đăng ký LaunchServices
```

## 5. Ký số & Gatekeeper (RM3)

| Hạng mục | Quyết định |
|---|---|
| Dev hằng ngày | `scripts/build-macos.sh` ký **từng bundle** ad-hoc (`$DEVELOPER_ID` trống → `-s -`; **không** `--deep` — Apple deprecated) + `xattr -dr com.apple.quarantine` khi dev từ source; **không** phân phối ad-hoc |
| Release | **Developer ID Application** (2 binary: `TextVN.app`, `TextVN-IM.app`) + **hardened runtime** + entitlements tối thiểu (`com.apple.security.automation.apple-events` nếu cần AX observer) |
| Notarization | `scripts/notarize-macos.sh` (hoặc `package-macos-pkg.sh --notarize`): `xcrun notarytool submit --wait` + `xcrun stapler staple` — **bắt buộc trước public** (gate `P2-5 §6`) |
| Kiểm chứng | `spctl -a -vv`, `codesign --verify --deep --strict`, install từ `.pkg` tải về trên VM sạch |
| Không có cert lúc dev | ghi vào `docs/release/signing-status-mac.md` (task MAC-055) — không release khi còn ad-hoc |

## 6. Health & restart

```text
TextVN-IM ghi heartbeat (log ring buffer + file ~/Library/Application Support/TextVN/im-heartbeat.json, 5s)
TextVN.app check 10s: cũ >15s → hiện "IME chưa hoạt động" + nút "Khởi động lại IME"
  (khởi động lại = xattr kill process IMK; system tự respawn khi user gõ — verify ở spike MAC-007)
Shutdown TextVN.app: đóng socket; IMK client nhận EOF → offline mode (không mất gõ)
```

## 7. Updater (Ed25519 "Sparkle-style" — `PLAN §3.7`)

| Bước | Chi tiết |
|---|---|
| 1. Check | GitHub API `releases/latest` (channel stable/beta — như `P1-4 §7` bước 1) |
| 2. Verify | SHA-256 + Ed25519 (key hardcode, đổi key = major bump — giống Windows) |
| 3. Download | `~/Library/Caches/TextVN/staging/<ver>/` |
| 4. Apply | Giải nén 2 bundle đã staple → `codesign` verify lại → thay bằng `NSFileManager.replaceItem` (IMK đang load → kill IMK trước, system respawn); **rollback:** marker `last-good`, nếu `doctor` không thấy version → khôi phục từ `staging/<old>` |
| 5. Launch | Hiện "Đã cập nhật vX.Y.Z" |

- Không dùng Sparkle dependency mặc định (giữ 0 dependency ngoài Apple framework) —
  nếu sau này muốn Sparkle, qua ADR mới (license check trước — MIT/BSD tương thích GPL-3).
- Offline/fail → im lặng giữ bản hiện tại (S5).

## 8. Phân phối

| Kênh | Nội dung |
|---|---|
| GitHub Releases | `.pkg` + `SHA256SUMS` + notes tiếng Việt (kèm hướng dẫn enable input source) |
| Homebrew cask | `packaging/homebrew/textvn.rb` (tên cask `textvn`) — submit sau 2 release ổn định (task MAC-057); audit `brew audit --strict` |
| Portable dev | zip đã ad-hoc sign cho tester nội bộ |

## 9. Chẩn đoán

- `textvn doctor --export` (bản mac của `textvn-cli` — cùng crate `P0-1`): version, input source state,
  socket state, AX/tap permission, IMK heartbeat, config (redact path) — **không text content** (S2, test grep).

## 10. Task (chi tiết `P2-6-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| MAC-050 | TextVN.app skeleton + status menu 9 mục | 9/9 hoạt động; single instance |
| MAC-051 | IPC server unix socket + watcher + health §6 | `textvn ipc probe` thấy 2 client; kill IMK → status hiện lỗi |
| MAC-052 | Settings SwiftUI 6 tab + parity checklist | `parity-checklist.md` đủ mục PLAN §2.3(M6)+§8 |
| MAC-053 | Login item (SMAppService) + config init + hot-reload | Đổi file → hiệu lực <1s; file sai schema → giữ bản cũ |
| MAC-054 | `.pkg` 2 scheme + uninstall sạch §4 | VM sạch: cài/gỡ = 0 residue (`uninstall-check.sh`) |
| MAC-055 | Developer ID + hardened runtime + notarization | `spctl -a` accept; `docs/release/signing-status-mac.md` |
| MAC-056 | Updater §7 + rollback | Test pre-release: update OK; sai hash → giữ bản cũ |
| MAC-057 | Homebrew cask + submit | `brew install --cask textvn` trên VM sạch pass |
| MAC-058 | `doctor --export` bản mac | Zip không chứa text content (grep test) |
