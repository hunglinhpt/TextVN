# P2-3 — AX FIELD DETECT & APP PRESET (macOS) — Solution chi tiết

> WS3 · nối `P0-3 §3` (resolve) + `P2-1 §6` / `P2-2 §5` (thực thi).
> Cùng format với `../20-windows/P1-3-strategy-appdb.md`. `app_id` chuẩn mac = **bundle id** (P0-3 §2.1).

## 1. Nguồn `FieldContext` trên macOS

```
                 ┌── IMK:  app_id = NSRunningApplication(bundleIdentifier) của client PID
FieldContext ◄───┤          field_role ← AX query (P2-3 §2, opt-in TCC) — cache theo (pid, element)
 (app_id,        └── Tap:  app_id = frontmost application (NSWorkspace)
  field_role,                field_role ← cùng module FieldDetect (P2-2 §3.5)
  secure, caps,                (1 codebase — mirror `P1-3 §1` "1 nguồn sự thật")
  engine_owner)
```

- **Caps:** IMK = `PREEDIT|FIELD_DETECT|SELECTION` (+`INJECT_VK` nếu spike MAC-004 pass — `P2-1 §1`);
  Tap = `SELECTION|FIELD_DETECT|INJECT_VK` (không `PREEDIT`).
- **`engine_owner`:** `imk` (mặc định) | `tap` — field của preset, ánh xạ `P2-1 §5`/`P2-2 §6`.
- **Normalize:** bundle id lowercase (`com.google.chrome`); nếu `bundleIdentifier == nil`
  (process hệ thống) → dùng executable basename lowercase.

## 2. Bảng AX → `IME_FIELD_*` (module `FieldDetect.swift`, mirror `P1-3 §2`)

| # | Điều kiện (ưu tiên giảm dần) | `field_role` | `secure` |
|---|---|---|---|
| R1 | `AXSubrole == AXSecureTextField` (hoặc app ở secure input mode — S8) | `secure` | 1 |
| R2 | `AXRole == AXTextField` + (`AXPlaceholderValue`/`AXDescription`/`AXTitle` match `/(address\\|url\\|search)/i`) | `address_bar`/`search` | 0 |
| R3 | `AXSubrole == AXSearchField` | `search` | 0 |
| R4 | `AXRole ∈ {AXComboBox, AXPopUpButton}` | `combo` | 0 |
| R5 | `AXRole == AXTextField` + bundle ∈ {Excel, Numbers} khi đang ở grid (cell focus) | `candidate` | 0 |
| R6 | `AXRole == AXTextArea` + bundle ∈ {Terminal, iTerm2, Warp} (hoặc `AXIdentifier` chứa term) | `terminal` | 0 |
| R7 | `AXRole == AXTextArea` | `textarea` | 0 |
| R8 | `AXRole == AXWebArea` (hoặc ancestor có AXWebArea) | `web` | 0 |
| R9 | `AXRole == AXTextField` | `editbox` | 0 |
| R10 | Không query được (không TCC/app không AX) | `unknown` | 0 |

**Heuristic phụ khi không có TCC:** theo bundle id preset (`P2-1`… không — preset trong §3)
+ `AXRole` của **chính app mình** không cần TCC? → KHÔNG: AX process khác luôn cần quyền;
không quyền → role = `unknown` + strategy từ preset (bước 4 `P0-3 §3.1`) — vẫn đúng phần lớn.

**Permission flow:** Settings → "App-compat thông minh (AX)" lần đầu → `AXIsProcessTrustedWithOptions(prompt:)`;
từ chối → ghi `ax_permission: denied` trong `state.json`, preset vẫn chạy (RM4).
**Budget:** query async, cache 2s/pid, invalidate khi `NSWorkspace.didActivateApplication` +
`AXObserver(kAXFocusedUIElementChanged)`; vượt budget → dùng cache (giống `P1-3 §2`).

## 3. App preset mặc định — `data/appdb.default.json` (phần macOS)

> `S` = strategy · `I` = inject_mode (chỉ tap) · `O` = engine_owner · `EN` = enabled mặc định.
> Thứ tự file = thứ tự bảng (entry hẹp trước — `P0-3 §2.1`, xem ghi chú `P1-3 §3`).

| # | id | match (bundle id) | field khi | S | I | O | EN | Ghi chú |
|---|---|---|---|---|---|---|---|---|
| 1 | `mac.safari.url` | `com.apple.safari` | address_bar, search | SelectionReplace | — | imk | ✓ | **B1** |
| 2 | `mac.chrome.url` | `com.google.chrome`, `com.microsoft.edgemac`, `com.brave.browser` | address_bar, search | SelectionReplace | vk_then_unicode | imk | ✓ | **B1** |
| 3 | `mac.firefox.url` | `org.mozilla.firefox`, `org.mozilla.firefox-developer-edition` | address_bar, search | SelectionReplace | — | imk | ✓ | **B1** |
| 4 | `mac.spotlight` | `com.apple.spotlight`, `com.apple.systemuiserver` (Spotlight menu) | search, combo | SelectionReplace | — | imk | ✓ | **B1** (Spotlight = thanh tìm kiếm hệ thống) |
| 5 | `mac.safari.body` | `com.apple.safari` | web, body | Preedit | — | imk | ✓ | |
| 6 | `mac.excel.cell` | `com.microsoft.excel` | candidate, editbox | SelectionReplace | vk_then_unicode | imk | ✓ | **B1** |
| 7 | `mac.word.body` | `com.microsoft.word` | body, textarea | Preedit | — | imk | ✓ | **B2** gợi ý cuối câu |
| 8 | `mac.notes` | `com.apple.notes` | textarea | Preedit | — | imk | ✓ | |
| 9 | `mac.textedit` | `com.apple.textedit` | body | Preedit | — | imk | ✓ | app smoke chính |
| 10 | `mac.terminal` | `com.apple.terminal` | terminal | ForwardAsCommit | — | imk | ✓ | **B8**, giữ marked ngắn (B11) |
| 11 | `mac.iterm2` | `com.googlecode.iterm2` | terminal | ForwardAsCommit | — | imk | ✓ | **B8/B11** |
| 12 | `mac.vscode` | `com.microsoft.vscode` | body, web | Preedit | — | imk | **✗** | dev thích EN (ignore list) |
| 13 | `mac.xcode` | `com.apple.dt.xcode` | body, candidate | Preedit (candidate→SelectionReplace theo §2 R5) | — | imk | ✓ | **B3** |
| 14 | `mac.jetbrains` | `com.jetbrains.intellij`, `com.jetbrains.pycharm`, `com.jetbrains.clion` | candidate | SelectionReplace | vk_then_unicode | imk | ✓ | **B3** |
| 15 | `mac.slack` | `com.tinyspeck.slackmacgap` | body | Preedit | — | imk | ✓ | **B2** |
| 16 | `mac.discord` | `com.hnc.Discord` | body | Preedit | — | imk | ✓ | **B2** (Electron **B7**) |
| 17 | `mac.messages` | `com.apple.ichat` (Messages) | body | Preedit | — | imk | ✓ | **B2** |
| 18 | `mac.finder.rename` | `com.apple.finder` | editbox | SelectionReplace (fallback §6.3) | — | imk | ✓ | đổi tên file/Go to folder |
| 19 | `mac.systemsettings` | `com.apple.systempreferences` | search, editbox | SelectionReplace | — | imk | ✓ | ô tìm kiếm Settings |
| 20 | `mac.game.nokbd` | danh sách `data/games_blocklist.txt` (mac) | — | Passthrough | — | **tap** | ✗ | app không dùng IMK — user opt-in (P2-2 §7) |

**Nguyên tắc thêm preset:** như `P1-3 §3` — luôn kèm corpus/appcomptest case; `notes` trỏ bug `Bn`.

## 4. Override & state (khớm `P0-3 §3.1` — cùng bảng với `P1-3 §4`, khác path macOS)

| Nguồn (macOS) | Bước P0-3 §3.1 |
|---|---|
| Secure input mode / AX secure field → `secure=1` | 1 (không override được — S3) |
| `config.ignore_apps[]` + `state.json` toggle → `ctx.enabled` | 2 |
| User `~/Library/Application Support/TextVN/appdb.json` | 3 |
| Preset hệ thống (§3) | 4 |
| Field-role default + `ctx.caps` (downgrade rule `P0-3 §3.1`) | 5 |
| Fallback BackspaceType | 6 |

- Toggle EN/VN trong status menu → ghi `state.json` → broadcast `StateUpdate` (P0-3 §4, path mac §P2-0).

## 5. API nội bộ (dùng chung IMK + tap)

```swift
// adapters/macos-imk/Sources/IMKApp/FieldDetect.swift (tap import qua package dependency)
struct FieldContext { var appID: String; var role: UInt32; var secure: Bool;
                      var caps: UInt32; var owner: EngineOwner /* .imk | .tap */ }
func resolve(cfg: Config, appdb: AppDb) -> ime_context_v1   // cache theo (pid, element), TTL 2s
```

**Quan trọng (chống phân kỳ thuật toán):** Swift **không viết lại** logic parse/verify preset hay
quy tắc resolve 6 bước. Dùng 2 C-ABI có sẵn trong `textvn_ffi.h` (P0-2 §1):
`ime_appdb_verify(json,sig,…)` (Ed25519) + `ime_strategy_resolve(ctx, appdb_json, …)` → strategy id.
Swift chỉ: detect AX role (§2) → dựng `ime_context_v1` → gọi resolve → áp `P2-1 §6`.
Cùng implementation với Windows/Linux (test chung `strategy::resolve` unit + corpus `--adapter mac`).

## 6. Test

| Hạng mục | Test | File |
|---|---|---|
| Resolve đúng §3 | Unit `strategy::resolve` (20 preset × role × caps) — chạy 3 OS | `strategy/tests/preset_matrix.rs` |
| AX rules | Unit mock AX attributes (dictionary giả lập) | `adapters/macos-imk/Tests/FieldDetectTests` |
| Hành vi thật | AX harness 12 app CI / 20 app nightly | `P2-5 §3` |
| Regression | corpus `mac/bug_B{1,2,3,11,13}_*` + `owner_no_double` | `corpus/mac/` |

## 7. Task

| Task | Nội dung | Acceptance |
|---|---|---|
| MAC-005 | Spike AX + TCC (S7/S8 `P2-1 §9`) | `docs/specs/macos-ax-spike.md` |
| MAC-030 | `FieldDetect` rules R1–R10 + mock tests | Unit ≥ 30 case pass |
| MAC-031 | Cache/observer/budget 2s | Benchmark ghi `P2-5 §5`; 0 query đồng bộ trong callback |
| MAC-032 | Preset §3 (20 mục) + corpus ≥ 40 case | `replay corpus/mac` pass |
| MAC-033 | Loader + Ed25519 verify (dùng `textvn-appdb` — không viết lại) | Sai chữ ký → từ chối + warning |
| MAC-034 | Override chain §4 (path mac) | Unit thứ tự 6 bước |
| MAC-035 | Settings: thêm preset từ app đang chạy (P2-4 §3) | Lưu appdb user hợp lệ |
