# Targets JSON — `tools/appcomptest/targets/` (WIN-061)

Quy tắc locate **field** cần focus khi test 12 app CI (P1-5 §3 matrix) — input cho harness
`appcomptest` (WIN-060, P1-5 §4). File cùng format với preset (`P0-3 §2.1`) để **reuse**
(`match` / `field_role` / link `preset`).

> **G15 — reuse tối đa:** ngữ nghĩa locator ở đây là **một spec duy nhất** cho mọi consumer:
> script PowerShell (`tools/win/verify_targets.ps1`) và crate Rust harness (WIN-060) + rules
> `field-detect` (WIN-030) **phải diễn giải y hệt** (§ Locator semantics). Không copy logic từng nơi.

## 1. File & schema

Một file/app: `<app_id>.json` — 12 file CI = `notepad, word, vscode, chrome, edge, firefox,
excel, terminal, explorer, slack, discord, jetbrains`.

```jsonc
{
  "targets_version": 1,
  "app_id": "chrome",                        // = ten file (check chot)
  "match": { "any": [ { "exe": "chrome.exe" } ] },   // app_id normalize (P1-3 §2)
  "launch": {
    "kind": "exe | com | shell",             // exe: Start-Process paths; com: COM (Word/Excel);
                                             // shell: lenh (wt.exe, explorer.exe)
    "paths": [ "...", "%LOCALAPPDATA%\\..." ],  // co wildcard (app-*\x.exe) + PATH fallback
    "args": [ "--new-window", "{fixture}" ],    // {fixture} = trang test contenteditable %TEMP%
    "com_prog_id": "Word.Application",       // chi kind=com
    "com_add": "doc | wb",
    "ready_class": "Chrome_WidgetWin_1",     // tim/san sang cua so
    "process": "olk.exe",                    // tuy chon: doi ten process (mac dinh = match.exe)
    "appx_package": "Microsoft.WindowsTerminal", // tuy chon: version qua Get-AppxPackage (f6-5)
    "close": "never",                        // tuy chon: explorer (KHONG dong)
    "focus_before_probe": true,              // tuy chon: focus truoc khi doc UIA (WT)
    "profile": {                             // tuy chon: profile TAM + user.js (f6-13)
      "prefs": { "browser.preonboarding.enabled": false }, // user_pref truoc khi chay
      "args": [ "-no-remote", "-profile", "{profile_dir}" ] // {profile_dir} = thu muc tam %TEMP%
    }
  },
  "fields": [
    {
      "field_role": "address_bar",           // whitelist P0-3 §2.1 (11 role)
      "preset": "win.chrome.url",            // link preset (reuse)
      "locators": [ { ... }, { ... } ],      // >= 2 selector, thu tu = uu tien
      "notes": "..."
    }
  ]
}
```

### Locator keys (AND trong 1 locator, OR = thứ tự mảng)

| key | nghĩa |
|---|---|
| `control_type` | ControlType UIA (**39 type của .NET — KHÔNG có `TextArea`/`Grid`**, xem F6-1) |
| `automation_id`, `name`, `class_name` | so đúng (PropertyCondition exact) |
| `name_regex`, `class_regex` | regex — filter client-side (S4: `PropertyCondition` .NET không có Contains) |

**Readiness (per-field):** mỗi field poll chính locator của nó ≤10s trước khi đánh giá (F6-10 —
readiness ANY-field cũ đánh giá `address_bar` khi mới có `Document` → MISS sớm trên runner).
**Focus:** `focus_before_probe` → `Focus-UiAWindow` (AttachThreadInput) — **WT: `TermControl`
chỉ xuất hiện SAU focus** (verify diag3).

## 2. Cách chạy

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\check_targets.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\verify_targets.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\verify_targets.ps1 -Only chrome,word
```

Shared lib (G15): `tools/win/lib/win32-uia.lib.ps1` (EnumWindows/UIA/focus/locator eval) +
`tools/win/lib/targets.lib.ps1` (load/schema) — dot-source, không copy.

## 3. Acceptance (WIN-061)

| # | Acceptance | Trạng thái | Evidence |
|---|---|---|---|
| 1 | Mỗi app ≥ 2 locator | ✅ **PASS** | `check_targets.ps1` → `PASS - 12 file, moi app >= 2 locator` |
| 2 | Chạy lại sau upgrade app version không vỡ (1 app) | 🟡 **upgrade-cycle GREEN** — run **36296260485** (2026-09-27): cài VS Code `1.137.0` (direct download từ manifest winget-pkgs) → verify OK → upgrade `.exe` chính thức → verify lại → version đổi + locator OK. **cross-image** run 36296260485: windows-2022 + 2025 — chrome/edge/explorer/notepad/terminal **OK cả 2 image** (version khác nhau: chrome 153/154, edge 152/154, explorer `SearchEditBox`/`TextBox`), chỉ **firefox MISS** = modal `TOU_ONBOARDING` fresh profile → **đã fix `launch.profile` prefs (F6-13)**, dispatch lần 3 để chốt. **Không đụng app trên máy local** (user rule 2026-09-27) |

**Verify local** (2026-09-27, Win11 26100, máy dev): **12/12 field rows OK, 0 FIELD_MISS**
(15 dòng = chrome/edge/excel có 2 field) — xem `verify-evidence.md` (auto-generated, S2: chỉ
ControlType/ClassName/chi-số, không ghi Name). App chưa cài (slack/discord/jetbrains) = ghi
`NOT_INSTALLED` — locator chưa verify local (limitation §5).

Hit thực tế (locator #0 trỏ đúng đối tượng):

| app | field | hit | resolves |
|---|---|---|---|
| notepad | body | `Document/RichEditD2DPT` | 1/3 |
| word | body | `Document/_WwG` | **3/3** |
| vscode | body | `Document/` | 2/3 |
| chrome | address_bar / web | `Edit/OmniboxViewViews` / `Document` | 2/3 · 1/3 |
| edge | address_bar / web | `Edit/OmniboxViewViews` / `Document` | **3/3** · 1/3 |
| firefox | address_bar | `ComboBox/urlbar-input textbox-input` | 2/3 |
| excel | candidate / editbox | `DataGrid/XLSpreadsheetGrid` / `Edit/XLFormulaBarEditor` | 1/3 · **2/2** |
| terminal | terminal | `Text/TermControl` (sau focus) | 1/3 |
| explorer | address_bar | `Edit/TextBox` (hoặc `UISmartProperty`) | 1/3 |

`resolves n/m` < full là **đúng thiết kế**: fallback locator nhắm **app version khác**
(GHA conhost = `Pane/Edit` [S5], legacy notepad = `Edit`) nên không resolve đồng thời trên
một version.

## 4. Findings

| ID | Finding | Xử lý |
|---|---|---|
| **F6-1** | `P1-3 §2 R8` dùng `ControlType == TextArea` — **không tồn tại** trên .NET UIA (39 statics, không TextArea/Grid; `GetProperty` sai vì statics là **FIELD**) → rule chạy runtime sẽ ra NULL | Locator JSON đã tránh (`Document`/`Edit`+`class`). **Cần sửa R8 khi `P1-3` hết WIP** (báo agent kia — G11) |
| **F6-2** | App **self-update ngay khi launch** (Discord 1.0.9257 → 9259 sau vài giây) → không dùng làm đối tượng upgrade-test | Chọn VS Code (control được, không tự update giữa chừng) trên **runner ephemeral**; local không cài/upgrade app nào |
| **F6-3** | Chromium **không expose `Document` với `about:blank`** (trang rỗng) → spawn `{fixture}` (contenteditable) thay about:blank | đã áp chrome/edge `launch.args` |
| **F6-4** | Win11 Notepad UIA tree render **async** — query ngay khi cửa sổ vừa mở = 0/3 | readiness poll ≤10s trong verify (S4: tree nóng sau render) |
| **F6-5** | `wt.exe` là **app-execution-alias** → `VersionInfo.ProductVersion` rỗng → cột version trống, mất dữ liệu so sánh version trên CI | key `launch.appx_package` optional → `Get-AppxPackage` (`terminal.json` = `1.24.11911.0`) |
| **F6-6** | Workflow GHA dễ fail ngầm: wrapper đọc `$LASTEXITCODE` **cuối** step → winget exit ≠ 0 (`0x8A15002B` khi "không có upgrade") làm fail step oan; thiếu `permissions`/`timeout-minutes` so với `hook-spike.yml`; bản cũ VS Code còn sót ở `Program Files` sẽ che `paths[0]` sau khi winget cài về `%LOCALAPPDATA%` | `exit 0` tường minh ở cuối step winget; `permissions: contents: read` + timeout; bước dọn `Program Files` nếu version ≠ baseline (chỉ trên runner ephemeral) |
| **F6-7** | `Write-Output 'text ' + $x` (thiếu ngoặc) → PowerShell tách thành nhiều argument → message sai/chữi | Luôn `Write-Output ('text ' + $x)` — paren bọc expression (A10-family) |
| **F6-8** | Step summary markdown: header 6 cột nhưng row evidence 7 cột (thiếu `ms`) → bảng render sai | Đổi header khớp đúng 7 cột của `verify-evidence.md` |
| **F6-9** | `Find-AppWindow` chọn **cửa sổ đầu tiên** khớp class+proc → bắt nhầm **popup owned** cùng class (Chrome fresh mở "Translate this page?" = `TranslateBubbleView`, 17 node, không omnibox/Document) thay vì cửa sổ chính | `VtWin.GetOwner` (GW_OWNER) + **2 pha**: bỏ qua owned, chỉ fallbackowned khi không có unowned. Chứng minh local diag7/diag8: bắt đúng cửa sổ → omnibox HIT |
| **F6-10** | Readiness **ANY-field** cũ cho phép evaluate `address_bar` khi mới chỉ `Document` (renderer) xong → field chưa render bị MISS sớm trên runner chậm | readiness **theo từng field**: poll chính locator của field đó ≤10s trước khi đánh giá; MISS → dump ≤25 node (ct/cls/aId, S2) vào log CI |
| **F6-11** | `winget` trên runner fresh fail: prompt thỏa thuận msstore không đọc được input (`0x8a150042`) + source hỏng (`0x8a15000f Data required by the source is missing`) → install/uninstall/upgrade đều đỏ | `upgrade-cycle` bỏ hẳn winget: cài bản cũ từ URL trong manifest winget-pkgs (GitHub raw), upgrade bằng `.exe` chính thức `?os=win32-x64-user` (in-place cùng thư mục) |
| **F6-12** | Win11 Notepad packaged **cold-start có thể >6s** lúc máy tải → spawn không tìm thấy window; lần chạy sau `$before` loại mất pid đã có window → false negative | `hook_probe` timeout 6s → 15s; verify dùng readiness poll ≤10s/field |
| **F6-13** | **Firefox profile MỚI mở modal `TOU_ONBOARDING`** (Terms of Use, bug 1964544) che toàn bộ toolbar → cây chỉ 51 node (36 Menu + modal), **không có `ComboBox`/`Edit`** → `address_bar` MISS 0/3 trên runner + local fresh. Pref `browser.aboutwelcome.enabled=false` **không** tắt được | key `launch.profile {prefs, args}` → verify tự tạo profile tạm + `user.js` với `browser.preonboarding.enabled=false` + `termsofuse.bypassNotification=true` (+`acceptedVersion=999`) → diag9: modal gone, cây 1012 node, urlbar HIT. Cleanup xóa profile tạm |

## 5. Limitation / follow-up

- slack, discord, jetbrains: **chưa cài local** → locator suy từ R7/R8 + chuẩn Electron/Swing,
  chưa verify thật. Verify trên app có sẵn khi nào cài (CI image không có Office/Slack…).
- Locale-dependent `name_regex` (S4: Name exact) — runner image = en-US; máy khác locale cần
  re-verify.
- `verify-evidence.md` regenerate mỗi lần chạy (máy/phiên khác nhau → khác nhau, không phải
  diff regression).
- WIN-060 (harness crate Rust) consume JSON này — build theo **cùng locator semantics** (G15);
  `field-detect` (WIN-030) tái dùng thay vì viết lại.
