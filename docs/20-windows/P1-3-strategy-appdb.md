# P1-3 — STRATEGY ENGINE & APP PRESET trên Windows (WS4)

> Nối `P0-3 §3` (resolve) + `P1-1 §6` / `P1-2 §5` (thực thi).
> Sản phẩm: (a) pipeline detect `field_role` bằng **UIA**, (b) **appdb mặc định 20 app**, (c) cache/owner rules.

## 1. Nguồn `FieldContext` trên Windows

```
                    ┌── TSF (in-proc):  app_id = exe của HOST PROCESS (GetModuleFileNameA, chuẩn hoá)
FieldContext ◄──────┤                    field_role ← cache field-detect (module chung, xem §2)
(app_id,            └── Hook:            app_id = foreground process exe
 field_role,                            field_role ← UIA worker (P1-2 §4)
 secure, caps,                             (cả 2 đều dùng CÙNG crate field-detect → 1 nguồn sự thật)
 engine_owner)
```

- **Caps:** TSF = `PREEDIT|SELECTION|FIELD_DETECT|INJECT_VK`; Hook = `SELECTION|FIELD_DETECT|INJECT_VK`
  (hook không có `PREEDIT` → preset `Preedit` tự downgrade theo `P0-3 §3.1`).
- **`engine_owner`** (P1-2 §6): `tsf` mặc định; `hook` cho game/legacy/console.
- **Normalize `app_id`:** `basename(exe).to_lowercase()` (vd: `chrome.exe`); không có path, không có full path
  (ổn định qua mọi phiên bản app).

## 2. Bảng UIA → `IME_FIELD_*` (rules_engine: `field-detect/src/rules_win.rs`)

| # | Điều kiện (ưu tiên giảm dần) | `field_role` | `secure` |
|---|---|---|---|
| R1 | `CurrentIsPassword == TRUE` | `secure` | 1 |
| R2 | `ClassName ∈ {Windows.UI.Core.CoreWindow}` **và** element name match `/(address\|search)/i` | `address_bar` | 0 |
| R3 | `ControlType == Edit` **và** (`AutomationId ∈ {Address, SearchBox, ...}` hoặc name match `/(address and search bar\|address bar\|search)/i`) | `address_bar` / `search` | 0 |
| R4 | `ControlType == ComboBox` (hoặc Edit có `IsValuePatternAvailable` + `IsExpandCollapsePatternAvailable`) | `combo` | 0 |
| R5 | Excel grid: window/ancestor class `Excel7`/`XLGRID` (đọc từ hwnd gốc nếu element không trỏ được) | `candidate` | 0 |
| R6 | `ControlType ∈ {Edit}` + `IsKeyboardFocusable` + **không** multiline (`IsControlElement` & không `TextArea`) | `editbox` | 0 |
| R7 | `ControlType == Document` **và** Chromium/Electron (`Chrome_RenderWidgetHostHWND` ancestor) | `web` | 0 |
| R8 | `ControlType == TextArea` hoặc `ControlType == Edit` + `IsMultiline` | `textarea` | 0 |
| R9 | Terminal class (`CASCADIA_HOSTING_WINDOW_CLASS`, `PuTTY`, `xterm`, `WezTerm`) | `terminal` | 0 |
| R10 | Không xác định được | `unknown` | 0 |

> **Ghi chú vocab R6/R8 (F6-1):** `TextArea` ở đây = **variant enum nội bộ** `field-detect::rules_win::ControlType`,
> `multiline` = field snapshot do UIA adapter map (P1-2 §4) — KHÔNG phải static .NET/PowerShell UIA
> (`[ControlType]::TextArea` = silent NULL, xem `docs/specs/win-test-common-errors.md` **A15**).
> Locator JSON của harness dùng `ControlType=Edit` + check multiline.

**Heuristic phụ (khi UIA không trả gì):** theo `ClassName` của hwnd foreground
(`Chrome_WidgetWin_1` → `web`, `OpusApp` → `textarea`…) — bảng lưu trong appdb preset theo app
(không hard-code trong rules engine).

**Budget:** query ≤ 2ms (async, cache 2s/hwnd — `P1-2 §4`); vượt → dùng role đã cache/preset.

## 3. App preset mặc định — `data/appdb.default.json` (phần Windows)

> Cột `S` = strategy (P0-3), `I` = inject_mode (P1-2 §5.4), `O` = engine_owner, `EN` = enabled mặc định.
> Field không ghi = theo field-role default của `P0-3 §3.1`.

| # | id | match | field khi | S | I | O | EN | Ghi chú (bug) |
|---|---|---|---|---|---|---|---|---|
| 1 | `win.chrome.url` | `chrome.exe, msedge.exe, brave.exe, opera.exe` | address_bar, search, combo | SelectionReplace | vk_then_unicode | tsf | ✓ | **B1** |
| 2 | `win.firefox.url` | `firefox.exe` | address_bar, search | SelectionReplace | vk_then_unicode | tsf | ✓ | **B1** |
| 3 | `win.chrome.body` | như #1 | body, web, editbox | Preedit | — | tsf | ✓ | |
| 4 | `win.excel.cell` | `excel.exe` | candidate, editbox | SelectionReplace | vk_then_unicode | tsf | ✓ | **B1** autocomplete ô |
| 5 | `win.word.body` | `winword.exe` | body, textarea | Preedit | — | tsf | ✓ | gợi ý ở cuối câu |
| 6 | `win.notepad` | `notepad.exe` | mọi role | Preedit | — | tsf | ✓ | app smoke chính |
| 7 | `win.wt` | `windowsterminal.exe, wezterm-gui.exe, alacritty.exe, kitty.exe` | terminal, body | ForwardAsCommit | unicode | tsf | ✓ | **B8** |
| 8 | `win.conhost` | `cmd.exe, powershell.exe, pwsh.exe` | terminal | ForwardAsCommit | unicode | **hook** | ✓ | console cũ không TSF |
| 9 | `win.vscode` | `code.exe` | body, web | Preedit | — | tsf | **✗** | P2: dev thích EN (ignore list) |
| 10 | `win.jetbrains` | `idea64.exe, pycharm64.exe, clion64.exe…` | candidate | SelectionReplace | vk_then_unicode | tsf | ✓ | **B3** code completion |
| 11 | `win.slack` | `slack.exe` | body | Preedit | — | tsf | ✓ | **B2** Enter lặp từ |
| 12 | `win.discord` | `discord.exe` | body | Preedit | — | tsf | ✓ | **B2**, Electron **B7** |
| 13 | `win.zalo` | `zalo.exe` | body | Preedit | — | tsf | ✓ | **B2** |
| 14 | `win.explorer` | `explorer.exe` | address_bar, search, combo | SelectionReplace | vk_then_unicode | tsf | ✓ | **B1** |
| 15 | `win.mail` | `olk.exe, outlook.exe` | body | Preedit | — | tsf | ✓ | |
| 16 | `win.libreoffice` | `soffice.exe, swriter.exe, scalc.exe` | body | Preedit | — | tsf | ✓ | **B4** trễ |
| 17 | `win.figma` | `figma.exe, canva.exe` | web | Preedit | — | tsf | ✓ | canvas text |
| 18 | `win.rdp` | `mstsc.exe` | unknown | BackspaceType | unicode | tsf | ✓ | cần test matrix |
| 19 | `win.game.chat` | danh sách `data/games_blocklist.txt` | — | Passthrough | — | **hook** | ✗ | **B9**, inject chỉ khi chat box (P1-2 §8) |
| 20 | `win.terminal.legacy` | `putty.exe, mintty.exe` | terminal | ForwardAsCommit | unicode | hook | ✓ | không TSF |

**Nguyên tắc thêm preset:**
- **Thứ tự file = thứ tự bảng:** entry có `field khi` hẹp hơn (vd `win.chrome.url`) PHẢI đứng trước
  entry rộng hơn cùng exe (`win.chrome.body`) — theo quy tắc "entry đầu khớp thắng" của `P0-3 §2.1`.
- Chỉ thêm khi **có corpus hoặc appcomptest case** chứng minh (không thêm theo cảm tính).
- `notes` ghi bug `Bn` liên quan → traceable ngược `PLAN §1.3`.
- Preset mới không được đổi strategy của app đã pass test mà không qua PR review 2 người.

## 4. Override & state người dùng (nhắc lại P0-3 §3.1 + §4)

**Thứ tự resolve — PHẢI giống hệt `P0-3 §3.1` (bảng này chỉ là ánh xạ nguồn → bước):**

| Nguồn (Windows) | Bước P0-3 §3.1 | Ví dụ |
|---|---|---|
| UIA `IsPassword` → `ctx.secure=1` | 1 → Passthrough (không override được — S3) | KeePassXC |
| `config.ignore_apps[]` (disable_vi/force_vi/passthrough) + `state.json` toggle → tính `ctx.enabled` | 2 → `!enabled` Passthrough (macro theo `allow_macro_when_vi_off`) | dev: `code.exe → disable_vi` |
| User `appdb.json` (`%APPDATA%\TextVN\appdb.json`) | 3 → strategy user thắng preset hệ thống | `code.exe → enabled=true` |
| Preset hệ thống (§3 `appdb.default.json`) | 4 | |
| Field-role default + `ctx.caps` (P0-3 §3.1 bước 5) | 5 | |
| Fallback BackspaceType | 6 | |

- `enabled_default` trong preset **chỉ là giá trị mặc định** khi chưa có `state.json`/`config.ignore_apps`
  → không đấu với bước 2 (khác với "user override" ở bước 3).
- `config.app_overrides` (method/diacritic_style…) là **lớp phủ method/style** theo `P0-3 §1`,
  không can thiệp chọn strategy.
- Toggle EN/VN trong tray → ghi `state.json {app_id: enabled}` → broadcast `StateUpdate` (P0-3 §4).
- **Không bao giờ** override được `secure` (S3).

## 5. Tích hợp với adapter — API nội bộ chung

```rust
// crate textvn-field-detect (field-detect/ — P0-1 §1), 1 codebase dùng cho cả TSF & hook
pub struct FieldContext { pub app_id: String, pub field_role: u32, pub secure: bool,
                          pub caps: u32, pub engine_owner: EngineOwner }
impl FieldContext {
    /// Tra preset + rules → ime_context_v1 (cache theo key: hwnd|exe)
    pub fn resolve(&self, cfg: &Config, appdb: &AppDb) -> ime_context_v1;
}
```
- Cache key: `app_id` (TSF) / `hwnd` (hook); TTL 2s; invalidate khi focus/foreground đổi.
- Mỗi lần resolve: O(1) hashmap + ≤3 regex precompiled (budget P0-3 §3.3).

## 6. Test & bằng chứng

| Hạng mục | Test | File |
|---|---|---|
| Resolve đúng bảng §3 | Unit `strategy::resolve` (20 preset × role × caps) | `strategy/tests/preset_matrix.rs` |
| UIA rules | Unit với element mock (UIA interface giả lập) | `field-detect/tests/rules_win.rs` |
| Hành vi thật | Appcomptest 12 app CI / 20 app nightly | `P1-5 §3` |
| Regression bug | Corpus `win/bug_B{1,2,3,6,7,8}_*` | `corpus/win/` |
| Không xử lý đôi | corpus `owner_no_double` | `corpus/win/` |

## 7. Mapping task

| Task | Nội dung | Acceptance |
|---|---|---|
| WIN-004 | Spike UIA: query role/IsPassword + đo latency 10 phần tử phổ biến | `docs/specs/uia-spike.md` (kèm số ms) |
| WIN-030 | crate `field-detect` + rules R1–R10 | Unit rules pass |
| WIN-031 | Cache + invalidation + budget 2ms | `cargo run --release -p textvn-bench -- field-switch --iters 20000` mô phỏng 200 hwnd/app; p99 cache+resolve < 2ms, không gọi UIA đồng bộ |
| WIN-032 | `engine_owner` + override chain §4 | Unit test thứ tự ưu tiên |
| WIN-033 | appdb loader + verify chữ ký Ed25519 (P0-3 §2.2) | Test file sai chữ ký bị từ chối |
| WIN-034 | Preset §3 (20 mục) + corpus tương ứng ≥ 40 case | `replay corpus/win` pass |
| WIN-035 | Chế độ "thêm preset từ user" trong Settings (P1-4 §3) | UI tạo preset + lưu file hợp lệ |
