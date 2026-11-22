# TextVN — Hướng dẫn Math & Khoa học (Math & Science Guide)

> **P11 — Universal Scientific, Engineering & Math Engine**: gõ ký hiệu khoa học,
> công thức LaTeX/Typst, ma trận, tính nhanh `=expr=`, dán công thức thông minh vào
> Word/MathType/Overleaf/Zalo. Tất cả offline, không keylogger, không network.
>
> **EN** (phần `# English` cuối file) — nội dung 1:1 với tiếng Việt.

---

## 1. Bật / tắt (Enable / Disable)

| Bước | Thao tác | Ghi chú |
|---|---|---|
| 1 | **UI (khuyến nghị):** checkbox `Math & Science` trong mục "Tùy chọn gõ" của cửa sổ Cài đặt (3 OS), hoặc chuột phải icon khay/menu bar → mục `Math & Science` (ngay dưới "Trợ lý ngữ pháp" ở Windows) | Lưu ngay vào `config.math.enabled`, engine nạp lại tức thì — không cần khởi động lại. Mặc định **TẮT** — hành vi gõ cũ giữ nguyên 100% |
| 2 | (Tuỳ chọn) chỉnh tay `config.v1.json`: `"math": { "enabled": true }` rồi ConfigReload | Tương đương bước 1 |
| 3 | Sticky Math Mode: `"sticky_math_mode": true` | Chords hoạt động ở **mọi** app, không cần trong `$...$` |

Tổ hợp phím toàn cục (đều remap được qua `math.hotkeys[]`):

| Hotkey | Hành động |
|---|---|
| `Ctrl+Alt+M` | Xoay vòng chế độ kết xuất: Normal → AutoByApp → Latex → UnicodeMath → Typst |
| `Ctrl+Shift+M` | Bật/tắt Sticky Math Mode |
| `Ctrl+Alt+V` | **Dán công thức thông minh** — tự chọn định dạng theo app đích |
| `Ctrl+Alt+Space` | Mở Palette tìm kiếm ký hiệu/công thức |

---

## 2. Layer A — Leader chords (phím leader `` ` ``)

Nhấn leader `` ` `` rồi 1 phím mnemonic. **Case-sensitive**:

| Nhóm | Phím → Ký tự |
|---|---|
| **Hy Lạp thường** | `` ` `` + `a` α · `b` β · `g` γ · `d` δ · `e` ε · `z` ζ · `h` η · `t` θ · `k` κ · `l` λ · `m` µ · `p` π · `r` ρ · `s` σ · `f` φ · `c` χ · `y` ψ · `w` ω |
| **Hy Lạp hoa** | `B` Β · `G` Γ · `D` Δ · `T` Θ · `L` Λ · `P` Π · `F` Φ · `Y` Ψ · `W` Ω · `X` Ξ |
| **Ký hiệu** | `8` ∞ · `+` ± · `x` × · `.` · · `=` ≠ · `~` ≈ · `<` ≤ · `>` ≥ · `o` ° · `O` ∅ · `n` ∇ · `N` ∂ · `A` ∀ · `E` ∃ |
| **Cấu trúc** (LaTeX/Typst/Unicode theo mode) | `/` phân số (cursor vào `{}` đầu) · `q` căn · `^` mũ · `_` chỉ số dưới · `i` ∫ · `I` ∬ · `S` ∑ |

Quy tắc FSM — **không bao giờ treo trạng thái**:

- `` ` `` × 2 → phát ký tự `` ` `` literal.
- `Esc` / `Space` → huỷ sạch, về Idle.
- Phím không map → phát literal `` ` `` + phím đó (không nuốt input).
- Quá 1500 ms hoặc mất focus → reset lặng lẽ.
- Pause > 200 ms khi Armed → Mini-HUD gợi ý mnemonic (không lấy focus).

Chế độ kết xuất ảnh hưởng đầu ra — ví dụ `/`:

| Mode | Output | Cursor |
|---|---|---|
| Normal | `÷` | — |
| Latex | `\frac{}{}` | trái 3 bước |
| Typst | `frac(, )` | trái 3 bước |
| UnicodeMath | `⁄` | — |

`^` / `_` ở mode **UnicodeMath** mở *chain*: `` ` ``+`^` rồi `25` → `²⁵` (phím nào không phải digit thoát chain).

## 3. Layer B — AltGr (tùy chọn)

Bật `"altgr_enabled": true` → giữ `AltGr` (Ctrl+Alt bên phải) + 1 phím:
`a`→α · `p`→π · `m`→µ · `o`→Ω · `=`→≠ · `+`→± · `2`→² · `3`→³ · `d`→° · `x`→× · `/`→÷ · `q`→√ · `8`→∞.

## 4. Alias tiếng Việt (gõ `\tên` + Space)

| Alias | Ký tự | Alias | Ký tự |
|---|---|---|---|
| `\tong` | ∑ | `\tichphan` · `\tp` | ∫ |
| `\phanso` · `\ps` | phân số | `\can` | √ |
| `\vocuc` · `\inf` | ∞ | `\suyra` | ⇒ |
| `\tuongduong` | ⇔ | `\khac` | ≠ |
| `\nhohon` | ≤ | `\lonhon` | ≥ |
| `\vuonggoc` | ⊥ | `\songsong` | ∥ |
| `\goc` | ∠ | `\do` | ° |
| `\duongkinh` | Ø | `\ohm` | Ω |
| `\thuannghich` | A⁻¹ | `\vc` | vector |

Đầy đủ trong palette (`Ctrl+Alt+Space`): **1.500+** ký tự/công thức/đơn vị/nguyên tố
(6 lĩnh vực: Toán thuần · Vật lý/Cơ khí/GD&T · Điện/Ô tô/Embedded · Hóa/Sinh ·
CS/AI/ML · Quant/Tài chính) + **560** tổ hợp tiền tố SI × đơn vị + **118** nguyên tố
hoá học. Ranking cá nhân theo Dream RSI (recency + affinity lĩnh vực).

## 5. Template động

| Gõ | Kết quả |
|---|---|
| `\mat3x3` | `\begin{matrix} a_{11} & … \end{matrix}` — cursor vào ô đầu |
| `\bmat2x3` · `\pmat2x2` | Ma trận ngoặc vuông/tròn |
| `\det3` | `\det\begin{vmatrix} … \end{vmatrix}` |
| `\hpt3` | Hệ phương trình 3 ẩn (`\begin{cases} x_{1} = 0 …`) |
| `\cases2` | Hàm điều kiện 2 nhánh |

Biên **1 ≤ N, M ≤ 10** (tổng ô ≤ 100). Template lớn hơn 64 ký tự (giới hạn ABI IME)
đi đường **Ctrl+Alt+V Smart Paste**.

## 6. Tính nhanh `=expr=`

Gõ `=2^10=` rồi **Space/Enter** → `= 1024 `. Hỗ trợ `+ - * / ^ %`, `sqrt sin cos
tan ln log10 abs`, `pi`, `e`, ngoặc `()` `[]`. Chia 0 / overflow / cú pháp sai →
không có gì xảy ra (fail-open, không sửa chữ của bạn).

## 7. Bypass ngữ cảnh — tiếng Việt không bị phá

- Trong `$...$`, `$$...$$`, `\(...\)`, `\[...\]`, `\begin{...}` → Telex/VNI **tạm ngừng**.
- Trong `\text{`, `\textrm{`, `\mbox{`, `\textbf{` → tiếng Việt **bật lại** (gõ chữ có dấu bình thường).
- `\{`, `\}`, `\$` là ký tự literal — không đổi ngữ cảnh.
- Token code **miễn nhiễm**: `0xFF`, `0b1010`, `u32`, `f64`, `CAN_FD`, `ECU_SW`, `GPIO_PIN`.

## 8. Dán công thức thông minh (Ctrl+Alt+V)

Copy công thức ở đâu đó, focus vào app đích, nhấn `Ctrl+Alt+V`:

| App đích | Định dạng dán |
|---|---|
| WINWORD.EXE / MathType | **Presentation MathML** (`<math xmlns="http://www.w3.org/1998/Math/MathML">`) — phương trình chỉnh sửa được |
| Code.exe / Overleaf / TeXstudio / Obsidian | LaTeX sạch |
| main.typ / Typst | Typst |
| Zalo / Teams / PowerPoint / Slack / Chrome Docs | Unicode math (`∫₀^∞ e⁻ˣ² dx`, `H₂SO₄ + 2NaOH → Na₂SO₄ + 2H₂O`) |

Bộ phân loại chỉ convert khi **score ≥ 0.75** — **0% false positive** trên đường dẫn
(`C:\Users\...`), tiền tệ (`$50 - $100`), JSON, code block, URL, văn xuôi.
Payload > 64 KB bị loại trước khi parse; công thức giữ **chỉ trong RAM** (không đĩa).

## 9. Bảo vệ phím tắt (không bao giờ bị nuốt)

- Toàn cục: `Ctrl+A/C/V/X/Z/Y/S` — math không đụng.
- MathType: `Ctrl+F/R/H/L/G/I`, `Alt+\`.
- Word Equation: `Alt+=`.
- OS reserved: `Win+*`, `Alt+Tab`, `Ctrl+Alt+Del`.

---

# English

> **P11 — Universal Scientific, Engineering & Math Engine**: type scientific symbols,
> LaTeX/Typst formulas, matrices, quick-calc `=expr=`, and smart-paste formulas into
> Word/MathType/Overleaf/Zalo. Fully offline, no keylogging, no network.

## 1. Enable / Disable

| Step | Action | Note |
|---|---|---|
| 1 | **UI (recommended):** the `Math & Science` checkbox in the Settings window's "Tùy chọn gõ" section (all 3 OS), or right-click the tray/menu-bar icon → `Math & Science` item (directly below "Trợ lý ngữ pháp" on Windows) | Saves to `config.math.enabled` instantly and the engine reloads immediately — no restart needed. Default **OFF** — legacy typing behavior unchanged 100% |
| 2 | (Optional) hand-edit `config.v1.json`: `"math": { "enabled": true }` then ConfigReload | Equivalent to step 1 |
| 3 | Sticky Math Mode: `"sticky_math_mode": true` | Chords work in **any** app, no `$...$` needed |

Global hotkeys (all remappable via `math.hotkeys[]`):

| Hotkey | Action |
|---|---|
| `Ctrl+Alt+M` | Cycle output mode: Normal → AutoByApp → Latex → UnicodeMath → Typst |
| `Ctrl+Shift+M` | Toggle Sticky Math Mode |
| `Ctrl+Alt+V` | **Smart Formula Paste** — auto-picks format for the focused app |
| `Ctrl+Alt+Space` | Open symbol/formula search Palette |

## 2. Layer A — Leader chords (`` ` ``)

Press leader `` ` `` then one mnemonic key. **Case-sensitive**:

| Group | Key → Symbol |
|---|---|
| **Greek lower** | `` ` ``+`a` α · `b` β · `g` γ · `d` δ · `e` ε · `z` ζ · `h` η · `t` θ · `k` κ · `l` λ · `m` µ · `p` π · `r` ρ · `s` σ · `f` φ · `c` χ · `y` ψ · `w` ω |
| **Greek upper** | `B` Β · `G` Γ · `D` Δ · `T` Θ · `L` Λ · `P` Π · `F` Φ · `Y` Ψ · `W` Ω · `X` Ξ |
| **Symbols** | `8` ∞ · `+` ± · `x` × · `.` · · `=` ≠ · `~` ≈ · `<` ≤ · `>` ≥ · `o` ° · `O` ∅ · `n` ∇ · `N` ∂ · `A` ∀ · `E` ∃ |
| **Structures** (LaTeX/Typst/Unicode per mode) | `/` fraction (cursor into first `{}`) · `q` sqrt · `^` sup · `_` sub · `i` ∫ · `I` ∬ · `S` ∑ |

FSM rules — **zero stuck states**:

- Leader × 2 → literal leader char.
- `Esc` / `Space` → clean cancel, back to Idle.
- Unmapped key → literal leader + that key (input never swallowed).
- Timeout (1500 ms) or focus loss → silent reset.
- Pause > 200 ms while Armed → non-focus-stealing Mini-HUD.

Output mode affects results — example `/`:

| Mode | Output | Cursor |
|---|---|---|
| Normal | `÷` | — |
| Latex | `\frac{}{}` | left 3 steps |
| Typst | `frac(, )` | left 3 steps |
| UnicodeMath | `⁄` | — |

`^` / `_` in **UnicodeMath** mode open a *chain*: leader+`^` then `25` → `²⁵`
(any non-digit exits the chain).

## 3. Layer B — AltGr (opt-in)

Enable `"altgr_enabled": true` → hold `AltGr` (right Ctrl+Alt) + one key:
`a`→α · `p`→π · `m`→µ · `o`→Ω · `=`→≠ · `+`→± · `2`→² · `3`→³ · `d`→° · `x`→× · `/`→÷ · `q`→√ · `8`→∞.

## 4. Vietnamese aliases (type `\name` + Space)

| Alias | Symbol | Alias | Symbol |
|---|---|---|---|
| `\tong` | ∑ | `\tichphan` · `\tp` | ∫ |
| `\phanso` · `\ps` | fraction | `\can` | √ |
| `\vocuc` · `\inf` | ∞ | `\suyra` | ⇒ |
| `\tuongduong` | ⇔ | `\khac` | ≠ |
| `\nhohon` | ≤ | `\lonhon` | ≥ |
| `\vuonggoc` | ⊥ | `\songsong` | ∥ |
| `\goc` | ∠ | `\do` | ° |
| `\duongkinh` | Ø | `\ohm` | Ω |
| `\thuannghich` | A⁻¹ | `\vc` | vector |

Full list in the palette (`Ctrl+Alt+Space`): **1,500+** symbols/formulas/units/elements
across 6 disciplines (Pure Math · Physics/Mech/GD&T · Electrical/Automotive/Embedded ·
Chemistry/Bio · CS/AI/ML · Quant/Finance) + **560** SI prefix × unit combos + **118**
chemical elements. Personal ranking via Dream RSI (recency + discipline affinity).

## 5. Dynamic templates

| Type | Result |
|---|---|
| `\mat3x3` | `\begin{matrix} a_{11} & … \end{matrix}` — cursor in first cell |
| `\bmat2x3` · `\pmat2x2` | Bracketed/parenthesized matrix |
| `\det3` | `\det\begin{vmatrix} … \end{vmatrix}` |
| `\hpt3` | 3-unknown equation system (`\begin{cases} x_{1} = 0 …`) |
| `\cases2` | 2-case piecewise function |

Bounds **1 ≤ N, M ≤ 10** (≤ 100 cells). Templates longer than 64 chars (the IME ABI
limit) go through **Ctrl+Alt+V Smart Paste** instead.

## 6. Inline calculator `=expr=`

Type `=2^10=` then **Space/Enter** → `= 1024 `. Supports `+ - * / ^ %`,
`sqrt sin cos tan ln log10 abs`, `pi`, `e`, parentheses `()` `[]`. Division by zero /
overflow / malformed input → nothing happens (fail-open, your text is never touched).

## 7. Context auto-bypass — Vietnamese stays safe

- Inside `$...$`, `$$...$$`, `\(...\)`, `\[...\]`, `\begin{...}` → Telex/VNI **suspended**.
- Inside `\text{`, `\textrm{`, `\mbox{`, `\textbf{` → Vietnamese **re-enabled**.
- `\{`, `\}`, `\$` are literal chars — no context change.
- Code tokens are **immune**: `0xFF`, `0b1010`, `u32`, `f64`, `CAN_FD`, `ECU_SW`, `GPIO_PIN`.

## 8. Smart formula paste (Ctrl+Alt+V)

Copy a formula anywhere, focus the target app, press `Ctrl+Alt+V`:

| Target app | Paste format |
|---|---|
| WINWORD.EXE / MathType | **Presentation MathML** (`<math xmlns="http://www.w3.org/1998/Math/MathML">`) — editable equation |
| Code.exe / Overleaf / TeXstudio / Obsidian | Clean LaTeX |
| main.typ / Typst | Typst |
| Zalo / Teams / PowerPoint / Slack / Chrome Docs | Unicode math (`∫₀^∞ e⁻ˣ² dx`, `H₂SO₄ + 2NaOH → Na₂SO₄ + 2H₂O`) |

The classifier only converts at **score ≥ 0.75** — **0% false positives** on paths
(`C:\Users\...`), currency (`$50 - $100`), JSON, code blocks, URLs, prose. Payloads
over 64 KB are rejected before parsing; formulas stay **in RAM only** (no disk).

## 9. Protected shortcuts (never hijacked)

- Global: `Ctrl+A/C/V/X/Z/Y/S` — math never touches them.
- MathType: `Ctrl+F/R/H/L/G/I`, `Alt+\`.
- Word Equation: `Alt+=`.
- OS reserved: `Win+*`, `Alt+Tab`, `Ctrl+Alt+Del`.

## 10. Performance budgets (measured, enforced by benchmarks)

| Path | Budget | Mechanism |
|---|---|---|
| Idle typing fast-check | < 5 µs, 0 alloc | One bool read when math disabled |
| Chord FSM / bypass | < 50 µs p99 | Zero-alloc Copy-enum FSM |
| Lexicon lookup | < 300 µs p99 | Sorted static slice + binary search |
| Clipboard AST convert | < 2 ms p99 (≤ 2 KB) | Off-UI-thread latest-wins worker |
| Resident RAM | < 3 MB | Static embedded tables, bounded pools |
| Parser safety | MAX_DEPTH 64 · MAX_TOKENS 4096 · zero `unsafe` in pure logic | Bounded recursive descent |

> Note: chord timeout needs a wall clock, which the pure core does not own (P0-1).
> The FSM resets via focus loss / cancel / double-leader today; adapter-side timing
> hooks can activate the 1500 ms timeout later without API changes.
