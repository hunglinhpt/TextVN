# P3-4 — AT-SPI FIELD DETECT & APP PRESET (Linux) — Solution chi tiết

> WS4 · nối `P0-3 §3` (resolve) + `P3-1 §6` / `P3-2 §5` / `P3-3 §5` (thực thi).
> Cùng format với `P1-3` (Win/UIA) và `P2-3` (mac/AX). `app_id` chuẩn Linux = **desktop-file-id**
> (fallback executable basename) — `P0-3 §2.1`.

## 1. Nguồn `FieldContext` trên Linux

```
                    ┌── IBus engine process: app_id từ focused AT-SPI app (LNX-005 chốt attr)
FieldContext ◄──────┼── Fcitx5 addon: cùng helper (dùng chung linux-common — §5)
 (app_id,           └── vietime-x11: WM_CLASS của _NET_ACTIVE_WINDOW (không qua AT-SPI)
  field_role,                 (1 codebase rules trong libvietime-linux-common.a)
  secure, caps,
  engine_owner)
```

- **Caps:** IBus/Fcitx5 = `PREEDIT|SELECTION|FIELD_DETECT`; X11 = `SELECTION|FIELD_DETECT|INJECT_VK`.
- **`engine_owner`:** `ibus` (mặc định) | `fcitx5` | `x11` — trường `P0-3 §2.1`;
  ánh xạ luật chống đôi: `P3-1` (framework active), `P3-2 §7`, `P3-3 §6.1`.
- **Normalize:** desktop-file-id lowercase (VD `org.gnome.TextEditor`); không có →
  executable basename lowercase; X11 → WM_CLASS second part lowercase.

## 2. Bảng AT-SPI → `IME_FIELD_*` (module trong `linux-common/src/field_detect.c`)

| # | Điều kiện (ưu tiên giảm dần) | `field_role` | `secure` |
|---|---|---|---|
| R1 | `ATSPI_ROLE_PASSWORD_TEXT` (hoặc state PASSWORD + role ENTRY) | `secure` | 1 |
| R2 | role `ENTRY` + name/description match `/(address\|url\|search\|tìm kiếm)/i` | `address_bar`/`search` | 0 |
| R3 | role `ENTRY` + state SEARCHABLE? (hoặc name match `/(search\|tìm)/i` khi không match R2) | `search` | 0 |
| R4 | role `COMBO_BOX` hoặc `ENTRY` trong combo ancestor | `combo` | 0 |
| R5 | role `TABLE`/`TREE_TABLE` + bundle ∈ {LibreOffice Calc, Gnumeric, Excel-web} khi cell editable | `candidate` | 0 |
| R6 | role `TERMINAL` (hoặc bundle ∈ {gnome-terminal, konsole, xfce4-terminal, kate-terminal}) | `terminal` | 0 |
| R7 | role `TEXT`/`SECTION` + editable | `textarea` | 0 |
| R8 | ancestor role `DOCUMENT_WEB` (hoặc bundle ∈ {firefox, chromium…} + role ENTRY/PARAGRAPH trong web area) | `web` | 0 |
| R9 | role `ENTRY` + state `EDITABLE` | `editbox` | 0 |
| R10 | Không query được (a11y bị tắt / app không expose) | `unknown` | 0 |

- **Không có a11y permission → role = `unknown`** + strategy từ preset (bước 4 `P0-3 §3.1`) —
  vẫn đúng phần lớn (RM4/RL4), mirror `P1-3 §2`/`P2-3 §2`.
- **Cache:** theo `(pid, AT-SPI object path)`, TTL 2s, invalidate khi app đổi
  (`ATSPI_EVENT_WINDOW_ACTIVATED`); budget: 0 query đồng bộ trong key callback (worker thread).

## 3. App preset mặc định — `data/appdb.default.json` (phần Linux)

> `S` = strategy · `I` = inject_mode (chỉ `x11`) · `O` = engine_owner · `EN` = enabled mặc định.
> Thứ tự file = thứ tự bảng (entry hẹp trước — `P0-3 §2.1`).

| # | id | match (desktop-file-id / WM_CLASS) | field khi | S | I | O | EN | Ghi chú |
|---|---|---|---|---|---|---|---|---|
| 1 | `lin.firefox.url` | `org.mozilla.firefox` | address_bar, search | SelectionReplace | — | ibus | ✓ | **B1** |
| 2 | `lin.chrome.url` | `google-chrome.desktop`, `chromium.desktop`, `brave-browser.desktop` | address_bar, search | SelectionReplace | — | ibus | ✓ | **B1** |
| 3 | `lin.gnome-settings` | `org.gnome.Settings`, `gnome-control-center` | search, entry | SelectionReplace | — | ibus | ✓ | ô tìm kiếm Settings |
| 4 | `lin.nautilus.rename` | `org.gnome.Nautilus`, `nemo.desktop`, `thunar.desktop` | editbox | SelectionReplace | — | ibus | ✓ | rename/Go to location (**B1**) |
| 5 | `lin.gnome-text-editor` | `org.gnome.TextEditor`, `gedit.desktop` | body | Preedit | — | ibus | ✓ | app smoke chính (demo M1) |
| 6 | `lin.firefox.body` | `org.mozilla.firefox` | web, body | Preedit | — | ibus | ✓ | |
| 7 | `lin.terminal` | `org.gnome.Terminal`, `org.gnome.Console` | terminal | ForwardAsCommit | — | ibus | ✓ | **B8**, marked ngắn (**B11**) |
| 8 | `lin.konsole` | `org.kde.konsole`, `org.kde.yakuake` | terminal | ForwardAsCommit | — | fcitx5 | ✓ | **B8** (KDE mặc định fcitx5) |
| 9 | `lin.vscode` | `com.visualstudio.code` | body | Preedit | — | ibus | **✗** | dev thích EN (ignore list) |
| 10 | `lin.jetbrains` | `jetbrains-idea.desktop`, `jetbrains-pycharm*` | candidate | SelectionReplace | — | ibus | ✓ | **B3** |
| 11 | `lin.libreoffice-writer` | `libreoffice-writer.desktop` | body | Preedit | — | ibus | ✓ | **B2** — verify D-Bus 6× (matrix bamboo) |
| 12 | `lin.libreoffice-calc` | `libreoffice-calc.desktop` | candidate | SelectionReplace | — | ibus | ✓ | **B1** (ô cell) |
| 13 | `lin.slack` | `slack.desktop` | body | Preedit | — | ibus | ✓ | **B2** |
| 14 | `lin.discord` | `discord.desktop` | body | Preedit | — | ibus | ✓ | **B2/B7** (Electron) |
| 15 | `lin.telegram` | `org.telegram.desktop` | body | Preedit | — | ibus | ✓ | **B2** |
| 16 | `lin.thunderbird` | `thunderbird.desktop` | body | Preedit | — | ibus | ✓ | |
| 17 | `lin.kate` | `org.kde.kate` | body | Preedit | — | fcitx5 | ✓ | editor KDE |
| 18 | `lin.obsidian` | `obsidian.desktop` | body | Preedit | — | ibus | ✓ | **B7** (Electron) |
| 19 | `lin.xfce-terminal` | `xfce4-terminal.desktop` | terminal | ForwardAsCommit | — | ibus | ✓ | **B8** |
| 20 | `lin.game.x11` | `data/games_blocklist.txt` (X11) | — | Passthrough | keycode_ascii* | **x11** | ✗ | opt-in module `P3-3`; *theo spike LNX-007 |

**Nguyên tắc thêm preset:** luôn kèm corpus/appcomptest case; `notes` trỏ bug `Bn` (như `P1-3 §3`).

## 4. Permission & fallback (RL4)

```text
lần đầu bật "App-compat thông minh (AT-SPI)" trong Settings:
  1. đọc org.a11y.Bus (XDG_RUNTIME_DIR/at-spi/bus/…) → nếu thiếu → hướng dẫn:
     GNOME: Settings → Accessibility → "Enable accessibility" (hoặc cài/run orca)
  2. org.a11y.Status.IsEnabled == false → prompt hướng dẫn; user bật
  3. vẫn không được → state.json ax_permission=denied → role=unknown (preset vẫn chạy)
```
- Không bao giờ block gõ vì thiếu permission (fail-open, `P0-3 §3.1` bước 5).

## 5. Module dùng chung `libvietime-linux-common.a` (C)

```c
/* adapters/linux-common/include/linux_common.h — link bởi ibus(C) + fcitx5(C++) + x11(Rust bindgen) */
typedef struct lc_field_ctx { char app_id[128]; uint32_t field_role; bool secure; } lc_field_ctx;
int  lc_field_detect(lc_field_ctx *out);              /* §2 rules — cache 2s, thread-safe */
int  lc_ipc_connect(const char *sock_path);           /* client ipc.v1 (P0-3 §5) */
void lc_log(const char *tag, const char *fmt, ...);   /* ghi vào ~/.local/state/.../log (S2: không text) */
int  lc_env_check(char *out, size_t len);             /* §7 env matrix cho doctor */
```
- **Resolve strategy KHÔNG viết lại:** gọi `ime_strategy_resolve()` (C-ABI — `P0-2 §1`);
  appdb verify bằng `ime_appdb_verify()`. Một implementation 3 OS (finding F2-011 của Phần 2).

## 6. Override & state (khớm `P0-3 §3.1` — path Linux)

| Nguồn | Bước P0-3 §3.1 |
|---|---|
| role `secure` (AT-SPI R1) → `secure=1` | 1 (không override được — S3) |
| `config.ignore_apps[]` + `state.json` toggle → `ctx.enabled` | 2 |
| User `~/.config/VietIME/appdb.json` | 3 |
| Preset hệ thống (§3) | 4 |
| Field-role default + `ctx.caps` (downgrade — `P0-3 §3.1`) | 5 |
| Fallback BackspaceType | 6 |

## 7. Env matrix & detect framework (dùng cho `vietime doctor` — `PLAN §5.3`)

| Biến | Giá trị gợi ý (IBus) | Giá trị (Fcitx5) | Ai cần |
|---|---|---|---|
| `GTK_IM_MODULE` | `ibus` (GNOME Wayland: mặc định) | `fcitx` | GTK3/4 app |
| `QT_IM_MODULE` | `ibus` | `fcitx` | Qt app (KDE…) |
| `XMODIFIERS` | `@im=ibus` | `@im=fcitx` | X11/XWayland app |
| `GLFW_IM_MODULE`/`SDL_IM_MODULE` | optional, document | optional | game SDL |

- **detect `linux_framework` "auto"** (P3-2 §7): ưu tiên `QT_IM_MODULE`/`GTK_IM_MODULE` trỏ tới
  fcitx5 → `fcitx5`, còn lại → `ibus` (GNOME Wayland luôn `ibus` nếu `XDG_CURRENT_DESKTOP=GNOME`).
- `vietime doctor` in bảng: biến nào sai → đề nghị fix (export trong shell profile/`~/.profile`,
  hoặc `.desktop` `env` line) — không tự sửa file của user (S9).

## 8. Test

| Hạng mục | Test | File |
|---|---|---|
| Resolve đúng §3 | Unit `strategy::resolve` (20 preset × role × caps — 3 OS) | `strategy/tests/preset_matrix.rs` |
| AT-SPI rules | Unit mock attributes (C: `linux-common/tests`) | ≥ 30 case |
| Hành vi thật | AT-SPI driver 12 app CI / 20 app nightly | `P3-6 §3` |
| Regression | corpus `linux/bug_B{1,2,3,8,10,13}_*` + `owner_no_double` | `corpus/linux/` |
| Env doctor | Integration: env sai → doctor đề nghị đúng | `P3-6 §1` |

## 9. Task (chi tiết `P3-7-TASKS.md`)

| Task | Nội dung | Acceptance |
|---|---|---|
| LNX-005 | Spike AT-SPI + a11y permission (S7/S8 `P3-1 §9` + attr app_id) | `docs/specs/linux-atspi-spike.md` |
| LNX-030 | `lc_field_detect` R1–R10 + mock tests | C unit ≥ 30 case pass |
| LNX-031 | Cache + event invalidate + budget 2s | resolve p99 < 2ms; 0 query đồng bộ trong callback |
| LNX-032 | Preset §3 (20 mục) + corpus ≥ 40 case | `replay corpus/linux` pass 100% |
| LNX-033 | Loader + `ime_appdb_verify` (P0-2 §1) | Sai chữ ký → từ chối + warning |
| LNX-034 | Override chain §6 (path Linux) | Unit thứ tự 6 bước |
| LNX-035 | `doctor` env matrix §7 + framework auto-detect | Test env sai → đề nghị đúng |
