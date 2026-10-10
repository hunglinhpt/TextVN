# P1-4 — UI, Config, IPC, Packaging & Release (Windows) — Solution chi tiết

> WS5 · crate `textvn-tray` → `TextVN.exe` (Win32 thuần — `tray/src/settings_dialog.rs`; ADR-004 "egui" không
> được áp dụng), CLI `textvn-cli.exe`, TIP `textvn-tsf.dll` + `textvn-tsf-x86.dll` (app 32-bit). Gói:
> `TextVN-setup-<ver>-windows-x64.exe` (per-user), `TextVN-setup-<ver>-windows-x64-machine.exe` (HKLM, Program
> Files — bản nộp Store EXE), `TextVN-portable-<ver>-windows-x64-<build>.zip`, `TextVN-<ver>-windows-x64.msix`
> (Store). **Updater chưa triển khai** (§7 là thiết kế). Schema IPC/config: `../10-shared/P0-3-config-preset-strategy.md`.
>
> **Trạng thái (2026-10-09):** các mục dưới là thiết kế gốc; chỗ nào lệch đã ghi chú tại chỗ. Ký
> Authenticode (SignPath Foundation) là hạng mục Windows duy nhất còn thiếu — §5.

## 1. Tray — `TextVN.exe` (chạy 1 instance, notify icon)

**Single instance:** mutex `Local\TextVNTray` + đã có instance → gửi `Ping` qua pipe rồi thoát.
**Autostart:** `HKCU\...\Run\TextVN` = `"<path>\TextVN.exe" --autostart` (bộ cài cho mọi người dùng ghi ở HKLM;
bản Store dùng `--autostart` hoặc `--msix-guard`)
(Không dùng Task Scheduler; không cần admin — S5.)

**Menu chuột phải (không mở cửa sổ):**

| Mục | Hành động |
|---|---|
| Bật/Tắt gõ tiếng Việt | svc → ghi `state.json` → broadcast `StateUpdate` (P0-3 §4) |
| Chế độ gõ (Telex/VNI/VIQR/…) | svc → ghi `config.json` → broadcast `ConfigReload` |
| Dấu (đậm/nghiêng/…) | submenu toggle `typo_*` |
| Cửa sổ đang gõ (tên app) + Enable cho app này | → state.json per-app |
| Game/Compat mode (bật hook) | Hook legacy chỉ được bật chủ động theo phiên; release mặc định là TSF-only |
| Cài đặt… | Mở hộp thoại Cài đặt Win32 (§3) |
| Sức khỏe / Trạng thái | Submenu: engine, hook (PID/uptime), pipe, version |
| Gỡ cài đặt | Bản cài: chạy `unins000.exe` cạnh `TextVN.exe`; bản Store: dọn phần ghi ngoài gói rồi mở Cài đặt › Ứng dụng; bản portable: gỡ đăng ký TSF rồi thoát (`tray/src/menu.rs`) |
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

## 3. Settings — hộp thoại Win32 (`tray/src/settings_dialog.rs`)

> Thực tế là **một hộp thoại** theo bố cục UniKey (`../release/ui-spec.md`), không phải 6 tab egui; bảng
> tab dưới đây là thiết kế gốc. Đối chiếu control thật: `../release/parity-checklist.md`.

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

## 4. Cài đặt — `TextVN-setup-<ver>-windows-x64.exe` (Inno Setup 6.5+, x86_64)

**Phạm vi mặc định: per-user (không admin)** — theo PLAN §3.7 + §8 (acceptance "Cài per-user").

Script thật (`installer/windows/TextVN-setup.iss`):

```text
[Files]  TextVN.exe, textvn-cli.exe, textvn-tsf.dll, textvn-tsf-x86.dll, resources\, data\, README/LICENSE/CHANGELOG
         (+ textvn-hook.exe chỉ khi build -IncludeCompatibilityHook)
[Tasks]  desktopicon (bỏ chọn) · autostart (checkedonce) · freectrlshift (checkedonce)
[Run]    textvn-cli.exe config init (skipifsilent) · TextVN.exe --free-ctrl-shift (task freectrlshift, chạy cả khi im lặng)
[Code]   ssPostInstall: textvn-cli.exe register (máy elevated trước, user sau); lỗi: có giao diện → exit 10,
         /VERYSILENT → chỉ chép file, exit 0 (app tự đăng ký ở lần chạy đầu)
[Registry] HKA\...\Run\TextVN = "TextVN.exe --autostart" (task autostart)
[Icons]  Start menu: TextVN · "Kiem tra he thong (TextVN Doctor)" = textvn-cli.exe doctor --pause · gỡ cài đặt
[UninstallRun] TextVN.exe --stop → textvn-cli.exe unregister → textvn-cli.exe unregister --scope machine
usUninstall: đổi tên DLL khoá thành .old-* rồi hẹn xoá (textvn-cli schedule-delete); usPostUninstall: xoá
         HKCU Run 'TextVN' nếu trỏ vào thư mục đang gỡ, trả Ctrl + Shift
Giữ %APPDATA%\TextVN (S9). Nâng cấp giữ nguyên tuỳ chọn người dùng.
```

Mặc định `PrivilegesRequired=lowest` (per-user, ARP HKCU, `%LOCALAPPDATA%\Programs\TextVN`). Bản máy:
`ISCC /DMachineInstall=1 /DOutputSuffix="-machine"` → `PrivilegesRequired=admin`, Program Files, ARP HKLM.

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
| Certificate | **SignPath Foundation (gói OSS)** — đã tích hợp: `release.yml` (job windows) truyền secret `SIGNPATH_*` → `build-release.ps1` gọi `tools/win/sign-signpath.ps1` (REST API, một yêu cầu ZIP deep-sign cho mọi PE) + ký cả hai bộ cài. **2026-10-09: chờ SignPath duyệt** — tới lúc đó `RELEASE_REPORT.json` ghi `authenticode: not-signed`. Không có workflow `signpath.yml` riêng. |
| Ký cái gì | `textvn-tsf.dll`, `textvn-tsf-x86.dll`, `TextVN.exe`, `textvn-cli.exe`, `TextVN-setup-*.exe` + `-machine.exe` (+ `textvn-hook.exe` ở gói Compatibility); `unins000.exe` chưa ký (giới hạn đã biết) |
| Toàn vẹn/provenance | Từ v0.2.27: GPG `.asc` (FPR `3921595ABC961199F15303B6C45B84D0C7F4A822`) + Sigstore cosign keyless cho mọi asset + `SHA256SUMS.txt` clearsign (`tools/release/sign-artifacts.sh`, bắt buộc trong CI). Không thay được Authenticode với SmartScreen. |
| Manifest | Ghi rõ nguồn build (commit SHA trong `version.json`), không self-modify |
| Submit | `release.yml` quét VirusTotal best-effort mỗi release; sau release đầu có Authenticode: submit Microsoft Security Intelligence → ghi `docs/release/code-signing-plan.md` |
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

## 7. Updater — crate `textvn-updater` (lib, chạy trong `TextVN.exe`)

> **Chưa triển khai** (WIN-056) — không có crate `textvn-updater`; người dùng cập nhật bằng cách cài đè
> bộ cài mới (nâng cấp giữ tuỳ chọn); bản Store do Store cập nhật.

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
| GitHub Releases | `TextVN-setup-<ver>-windows-x64.exe` + `-machine.exe` + `TextVN-portable-<ver>-windows-x64-<build>.zip` + `TextVN-<ver>-windows-x64.msix` + `SHA256SUMS.txt` (clearsign) + `.asc`/`.cosign.*` |
| Microsoft Store | **MSIX** (Store tự ký khi publish — đáp ứng 10.2.9 không cần Authenticode). Partner Center: Windows publisher ID `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`, publisher display name `LinhBH.CoM`; `Package/Identity/Name` lấy từ Partner Center qua repo variable `MSIX_IDENTITY_NAME`. Xem `../release/msix-submission.md`. Đường EXE (`-machine.exe`) chờ Authenticode. |
| winget | chưa làm (task `WIN-057`, sau khi có 2 release ổn định; chưa có `packaging/winget/`) |
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
