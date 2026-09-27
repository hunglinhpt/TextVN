# P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết

> WS5 · crate `textvn-win-tray` → `textvn-tray.exe` (egui — ADR-004), installer `textvn-setup.exe`,
> updater crate `textvn-updater` (lib, chạy trong tray; apply qua `textvn-setup.exe`). Schema IPC/config: `../10-shared/P0-3-config-preset-strategy.md`.

## 1. Tray — `textvn-tray.exe` (chạy 1 instance, notify icon)

**Single instance:** mutex `Local\TextVNTray` + đã có instance → gửi `Ping` qua pipe rồi thoát.
**Autostart:** `HKCU\...\Run\TextVN` = `"<path>\textvn-tray.exe" --autostart`
(Không dùng Task Scheduler; không cần admin — S5.)

**Menu chuột phải (không mở cửa sổ):**

| Mục | Hành động |
|---|---|
| Bật/Tắt gõ tiếng Việt | svc → ghi `state.json` → broadcast `StateUpdate` (P0-3 §4) |
| Chế độ gõ (Telex/VNI/VIQR/…) | svc → ghi `config.json` → broadcast `ConfigReload` |
| Dấu (đậm/nghiêng/…) | submenu toggle `typo_*` |
| Cửa sổ đang gõ (tên app) + Enable cho app này | → state.json per-app |
| Game/Compat mode (bật hook) | Hook legacy chỉ được bật chủ động theo phiên; release mặc định là TSF-only |
| Cài đặt… | Mở egui Settings (§3) |
| Sức khỏe / Trạng thái | Submenu: engine, hook (PID/uptime), pipe, version |
| Gỡ cài đặt | Chạy `textvn-setup.exe /UNINSTALL` |
| Thoát | Shutdown: `ime` clients được thông báo, hook exited |

**Badge icon:** `vn-on` (màu), `vn-off` (xám), `error` (warning — khi crash counter > 0).

## 2. IPC server (P0-3 §4) — chạy trong tray

```text
tokio (feature "net") task, named pipe \\.\pipe\textvn-ipc-v1 (DWORD mode PIPE_ACCESS_DUPLEX)
- Accept loop → per-connection frame `u32 little-endian length + JSON UTF-8` theo `schemas/ipc.v1.md`
- `Hello{pid, abi, version}` → `Snapshot{config_version, state, appdb_version, channel}`
- `GetSnapshot{}` → `Snapshot{...}` · `Subscribe{pid}` → ack (push `StateUpdate`/`ConfigReload` khi đổi)
- `ToggleViEn{app_id, enabled}` → ack + broadcast `StateUpdate`
- `CrashReport{code, count}` (không text!) → ghi counter → hiện badge error
- `Ping{}` → `Pong{uptime_ms}` (sức khỏe hook đọc từ `hook-stats.json` §6, không nhét vào `Pong`)
- **Danh sách message đúng bằng `P0-3 §5` — thêm message mới = sửa P0-3 + schema `ipc.v1.md` trước (review 2).**
- Threading: IPC KHÔNG bao giờ gọi ime_* (chỉ fan-out file/state; mỗi process tự đọc — S7)
```

**Config hot-reload:** tray watch `config.json` + `appdb.json` (notify crate, debounce 300ms)
→ validate (JSON schema, Ed25519 cho appdb) → broadcast `ConfigReload`.
File sai schema → **giữ config cũ**, ghi warning + hiện trong Settings (không crash — P0-2 §5).

## 3. Settings — egui (cửa sổ 1, tab dọc)

> Parity checklist — mỗi control Settings phải khớp backlog `PLAN §2.3 (M6)` + tiêu chí `PLAN §8`;
> điền số liệu thật vào `docs/release/parity-checklist.md` (mỗi dòng = 1 control + 1 test).

| Tab | Nội dung |
|---|---|
| **General** | Bật VN theo mặc định (`enabled_default`), method (Telex/VNI/VIQR/…), typo options (4+2 nhóm), `free_marking`, `spellcheck` (mặc định Should), language: EN/VI |
| **Applications** | Danh sách `app_overrides` + `ignore_apps` (mode dropdown) + nút "Thêm từ process đang chạy" (lấy foreground exe) + presets override (gộp vào appdb user, §P1-3 §4) |
| **Hotkeys** | Toggle `Ctrl+Shift+Space` (reserve key), sửa được, kiểm tra conflict |
| **Hook & Game** | `hook.mode` off/auto/always, danh sách game dùng hook, cảnh báo uiAccess/admin (P1-2 §7) |
| **Update** | Channel (stable/beta), "Kiểm tra cập nhật", link changelog |
| **About/Help** | Version, link docs, "Export diagnostics", "Open logs folder" |

- State/config lưu qua **svc layer in-process** (Settings nằm chung process với IPC server) → svc ghi
  file rồi broadcast theo P0-3 §4; egui **không** ghi file trực tiếp (một nguồn ghi = svc).
- Debounce 300ms mỗi control → một lần ghi + một lần `ConfigReload` (P0-3 §4).

## 4. Cài đặt — `textvn-setup.exe` (Inno Setup 6, x86_64)

**Phạm vi mặc định: per-user (không admin)** — theo PLAN §3.7 + §8 (acceptance "Cài per-user").

```text
[Run]
1.  Copy release mặc định: textvn-tsf.dll, textvn-tray.exe, textvn.exe, updater, data\. `textvn-hook.exe` chỉ có trong gói Compatibility được yêu cầu rõ ràng.
2.  "<dir>\textvn.exe" register --scope user     ← P1-1 §8 (đăng ký TIP + COM per-user)
3.  "<dir>\textvn.exe" config init               ← tạo %APPDATA%\TextVN\{config.json, appdb.json} nếu chưa có
4.  Tạo Start Menu + Desktop (optional)
5.  Tạo Uninstall key HKCU\...\Uninstall\TextVN
6.  Launch tray (--autostart)

[UninstallRun]
1.  textvn tray --stop (đóng tray + hook)
2.  "<dir>\textvn.exe" unregister --scope user   ← P1-1 §8 đảo ngược
3.  Xoá Run key, Start Menu
4.  Giữ %APPDATA%\TextVN (hỏi "Xoá config?") — S9: không xoá data người dùng khi gỡ
```

**UI:** Welcome → [Options: auto-start ☑ | tạo desktop ☐ | kênh beta ☐] → Install → Finish.
Không có "tiện ích đi kèm", không đổi homepage, không adware (YAML template giữ trong `installer/`).

**Manifest (trong exe):**
```xml
<requestedExecutionLevel level="asInvoker" uiAccess="false|true"/>  <!-- uiAccess=true CHỈ khi:
     (a) file đã ký, (b) cài vào Program Files (installer mode --system với option "Run as admin") -->
```
`--system` mode: chạy as admin, cài `C:\Program Files\TextVN\`, `uiAccess=true`, register `--scope machine`.
Task `WIN-055` test đủ 2 mode trên VM sạch.

## 5. Ký số & nguồn gốc (RW3 — antivirus)

| Hạng mục | Quyết định |
|---|---|
| Certificate | **SignPath OSS** (miễn phí cho repo OSS — nộp qua pipeline `signpath.yml`) → code signing cert EV/OV |
| Ký cái gì | `textvn-tsf.dll`, `textvn-tray.exe`, `textvn-hook.exe`, `textvn.exe`, `textvn-setup.exe` (mọi artifact chạy được) |
| Manifest | Ghi rõ nguồn build (commit SHA trong `version.json`), không self-modify |
| Submit | Sau release đầu: submit Microsoft Security Intelligence + VirusTotal hash list → ghi `docs/release/signing-status.md` |
| Portable | Zip đã ký bên trong từng exe; hướng dẫn "Unblock file" nếu bị Mark-of-the-Web |

## 6. Watchdog & process orchestration (tray)

```text
tray chỉ spawn `textvn-hook.exe` sau khi người dùng chọn chế độ Compatibility → lưu PID + start time
health: file %LOCALAPPDATA%\TextVN\hook-stats.json (hook ghi mỗi 5s: ts, p50/p99 — xem P1-2 §9)
tray check 10s một lần:
  - PID chết hoặc `hook-stats.json` cũ >15s → restart (exponential backoff 0.5s→5s, max 5 lần/phút)
  - restart quá 5 lần/phút → dừng spawn, hiện badge lỗi trong doctor
shutdown (tray exit): WM_CLOSE → hook tự exit (P1-2 §11: hook không sống mồ côi)
```

## 7. Updater — crate `textvn-updater` (lib, chạy trong `textvn-tray.exe`)

| Bước | Chi tiết |
|---|---|
| 1. Check | GET `https://api.github.com/repos/hunglinhpt/TextVN/releases/latest` (channel: `stable` → non-prerelease; `beta` → any) |
| 2. Verify | SHA-256 khớp `SHA256SUMS` + Ed25519 (key hardcode trong updater, key đổi = major bump) |
| 3. Download | `%LOCALAPPDATA%\TextVN\staging\<ver>\` |
| 4. Apply | (a) tray hiện "Đang cập nhật…" → spawn `textvn-setup.exe /SILENT /UPDATE` → **self-exit** (kèm `--wait-pids tray,hook`); (b) setup chờ process/tệp hết bị khoá (`textvn-tsf.dll` còn load → hẹn `MoveFileEx` pending-reboot + báo người dùng); (c) thay file → ghi marker `last-good`; **rollback:** nếu khởi động lại mà `doctor` báo version không khớp marker → chạy lại bản cũ còn trong `staging\<old>` |
| 5. Launch | tray tự khởi động lại, hiện "Đã cập nhật vX.Y.Z" |

- **Không bao giờ** tự động update khi `spellcheck`/hotkey đang gõ dở (chỉ check, hỏi người dùng trước khi Apply).
- Offline/fail → im lặng, giữ bản hiện tại (S5: app không chết vì mạng).

## 8. Phân phối

| kênh | Nội dung |
|---|---|
| GitHub Releases | `textvn-setup-x.y.z.exe` + `.zip` (portable) + `SHA256SUMS` + notes tiếng Việt |
| winget | `wingetcreate submit` (task `WIN-057`, sau khi có 2 release ổn định; manifest trong `packaging/winget/`) |
| Website/Landing | Trỏ thẳng GitHub (không build site trong Phần 1) |

**Channels:** `stable` (mặc định), `beta` (pre-release flag). Không có delta update (package < 5MB → full replace).

## 9. Chẩn đoán & hỗ trợ

- `textvn doctor --export` → zip: version.json, config (đã redact đường dẫn user), hook-stats, tsf log tail 200 dòng,
  OS/build info, TIP registration state, pipe status — **không** có nội dung text (S2).
- Settings → "Export diagnostics" gọi lệnh trên.

## 10. Mapping task

| Task | Nội dung | Acceptance |
|---|---|---|
| WIN-050 | Tray skeleton + single-instance + menu §1 | Manual: 10 mục menu hoạt động |
| WIN-051 | IPC server + state/watch (§2, §6) | `textvn ipc probe` thấy 2 client (sim + hook) |
| WIN-052 | Settings egui 6 tab (§3) + parity checklist | `docs/release/parity-checklist.md` đủ mục (PLAN §2.3 M6 + §8) |
| WIN-053 | Autostart + config init + tray↔settings debounce | Unit + manual restart |
| WIN-054 | Inno installer 2 mode (per-user/system) + uninstall sạch | VM sạch: cài/gỡ 0 residual (kiểm tra registry/FILE) |
| WIN-055 | uiAccess manifest + ký (SignPath) | Elevated app gõ được; binary có chữ ký hợp lệ |
| WIN-056 | Updater (§7) + rollback | Test trên pre-release: update + fail-path |
| WIN-057 | winget manifest + submit | `winget install TextVN` chạy được |
| WIN-058 | `textvn doctor --export` (§9) | Zip không chứa text content (grep test) |
