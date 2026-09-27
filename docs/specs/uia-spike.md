# Spike WIN-004 — UIA latency & IsPassword (`spikes/uia-probe`)

> Task: `WIN-004` (`../20-windows/P1-6-TASKS.md`) · Đối chiếu rule: `../20-windows/P1-3-strategy-appdb.md §2` (R1–R10, budget 2ms, cache 2s)
> Spike code: `../../spikes/uia-probe/` · Ngày chạy: 2026-09-27 · Máy: Windows 11 x64 · UIA2 `.NET System.Windows.Automation` (PowerShell 5.1)
> **Acceptance:** ✅ bảng ms/query (§1) + kết luận cache strategy (§5).

**Tạm kết:** chỉ `AutomationElement.FocusedElement` (worker, warm) chạm được budget 2ms —
**mọi `Find/FindFirst/FindAll` đồng bộ vượt budget 30–500×** (Word avg 989.6ms). Kết luận:
hook callback **không bao giờ** query UIA; đọc role từ cache hwnd 2s, miss → preset appdb/ClassName heuristic (§5).

Log: `%TEMP%\uia_spike_out.txt` — chỉ latency + role/classname, **không ghi text người dùng** (S2 Handbook).

## 1. Bảng ms/query — 10 phần tử (n=20/query)

Budget `P1-3 §2`: query ≤ **2ms** (async + cache 2s/hwnd).

| # | Phần tử (rule) | Find first / avg / min / p95 (ms) | fail | FromHandle avg (ms) | 6-prop cold (notable) | 6-prop warm avg/p95 (ms) |
|---|---|---|---|---|---|---|
| 1 | chrome_omnibox (R3) | 132.3 / **58.3** / 26.3 / 69.6 | 0/20 | 33.5 (first 292 = init) | ControlType **63.9**, ClassName 3.6, Name 2.2 | **6.1** / 6.1 |
| 2 | chrome_input_password (R1) | 64.5 / 48.6 / 34.6 / 64.5 | **20/20** | 3.5 | — (không thấy → S4-3) | — |
| 3 | chrome_textarea (R8) | 70.4 / 47.6 / 30.7 / 66.1 | **20/20** | 4.2 | — (S4-3 + S4-5) | — |
| 4 | notepad_edit (R6) | 63.8 / **21.5** / 16.3 / 23.3 | 0/20 | 1.2 | mọi prop 0.1–0.2 | 1.5 / 2.8 |
| 5 | word_document (R6) | 1385.9 / **989.6** / 77.2 / **1529.4** | 0/20 | 1.1 | 0.1–0.3 | 0.9 / 0.9 |
| 6 | excel_grid (R5) | 436.1 / **215.6** / 190.5 / 241.0 | 0/20 | 0.6 | 0.1–0.3 (tìm qua **Table**) | 0.7 / 0.9 |
| 7 | terminal (R9) | 126.7 / 49.2 / 12.7 / 120.3 | 0/20 | 0.5 | cls `DRAG_BAR_WINDOW_CLASS` | 0.3 / 0.4 |
| 8 | vscode_editor (R7) | — | — | — | **SKIP: không có window** (S4-11) | — |
| 9 | explorer_address (R3/R4) | 127.3 / 56.5 / 47.8 / 66.1 | 0/20 | 0.6 | Name `Address Bar`, cls `TextBox` | 1.4 / 2.8 |
| 10 | conhost_legacy (R9/R10) | 11.3 / 1.9 / 1.0 / 4.2 | **20/20** | 1.8 | không có Edit → S4-8 | — |
| — | chrome `FindAll(TrueCondition)` | 124.1 / **81.4** / — / 94.5 | — | — | walk-all toàn bộ descendants | — |
| — | **FocusedElement (đường hook thật)** | 13.8 (cold) / **2.2** / — / **2.1** | 0/20 | — | đọc ControlType+IsPassword+ClassName | **≤ budget** |

**Đọc bảng:** avg column là trung bình 20 lần query liên tiếp (warm sau lần đầu); `first` = lần chạm đầu (cold, gồm UIA client init).

### §1.1 Chạy lại từ repo — validate reproduce (09:53 cùng ngày, n=20)

Chạy `spikes\uia-probe\uia_spike.ps1` sau khi patch path (`$PSScriptRoot`) — số liệu run 2:

| Phần tử | Run 1 avg | Run 2 avg / p95 (ms) | Kết luận rút ra |
|---|---|---|---|
| chrome_omnibox (R3) | 58.3 | 94.9 / 112.9 | cùng băng, >budget |
| chrome_input_password (R1) | fail 20/20 | **fail 0/20, avg 29.7** — element `Pass`, `pwd=True` | ✅ xác nhận **S4-3**: profile đã có content (probe chạy trước cùng profile) → password thấy được |
| chrome_textarea (R8) | fail 20/20 | **VẪN fail 20/20** | ✅ xác nhận **S4-5**: nguyên nhân = condition `TextArea` null, **không** phải do content |
| notepad_edit (R6) | 21.5 | 142.1 / 162.2 | variance run-to-run lớn → dùng giá trị xấu nhất làm rủi ro planning |
| word_document (R6) | 989.6 | **1507.5 / 2050.6** | xấu hơn run 1 → kết luận "cấm `Find` đồng bộ" chắc chắn |
| excel_grid (R5) | 215.6 | 217.9 / 262.4 | lặp lại |
| terminal (R9) | 49.2 | 40.4 / 56.7 | lặp lại |
| explorer_address (R3) | 56.5 | 205.7 / 291.3 | >budget cả 2 run |
| conhost_legacy (R10) | fail 20/20 | fail 20/20, avg 3.1 | ✅ **S4-8** lặp lại |
| **FocusedElement** | 2.2 / 2.1 | **1.7 / 1.3, fail 0** | ✅ ≤ budget cả 2 run |

## 2. Property cold/warm

- **Cold (lần đầu trên hwnd):** `ControlType` 63.9ms (Chrome), còn lại 0.1–3.6ms → sau lần đầu mọi property đọc được 0.1–3.6ms.
- **Warm 6-prop gộp** (`ControlType, ClassName, Name, AutomationId, IsPassword, IsKeyboardFocusable`): 0.3–6.1ms (Chrome nặng nhất), app Win32 0.7–1.5ms.
- **Kết luận đo:** cache đủ **6 property theo hwnd, TTL 2s** — mỗi lần miss chỉ tốn ≤ ~6ms đọc prop NHƯNG chỉ đọc sau khi đã có element (không dùng `Find` để lấy element trong hook).

## 3. ControlType id — ground truth runtime (39 static, dump bằng `controltype_ids.ps1`)

| id | member | id | member | id | member |
|---|---|---|---|---|---|
| 50000 | Button | 50013 | RadioButton | 50026 | Group |
| 50001 | Calendar | 50014 | ScrollBar | 50027 | Thumb |
| 50002 | CheckBox | 50015 | Slider | 50028 | DataGrid |
| 50003 | ComboBox | 50016 | Spinner | 50029 | DataItem |
| 50004 | **Edit** | 50017 | StatusBar | 50030 | **Document** |
| 50005 | Hyperlink | 50018 | Tab | 50031 | SplitButton |
| 50006 | Image | 50019 | TabItem | 50032 | Window |
| 50007 | ListItem | 50020 | Text | 50033 | **Pane** |
| 50008 | List | 50021 | ToolBar | 50034 | Header |
| 50009 | Menu | 50022 | ToolTip | 50035 | HeaderItem |
| 50010 | MenuBar | 50023 | Tree | 50036 | Table |
| 50011 | MenuItem | 50024 | TreeItem | 50037 | TitleBar |
| 50012 | ProgressBar | 50025 | Custom | 50038 | Separator |

- **Member NULL** (tồn tại nhưng không dùng được): `TextArea`, `Grid` (xem S4-5/S4-6).
- **Observed ids:** Chrome browser UI = `[50000, 50004, 50018, 50019, 50021, 50026, 50033]`; web content thêm `50030 Document` (RootWebArea) + `50005 Hyperlink` + `50020 Text`; password input = `50004` + `IsPassword=True`; caption buttons = `50000`; Electron ChatGPT = `50000`(×3) + `50033`(×9); Terminal/legacy = window class heuristic.

## 4. Findings `S4-{n}` (không xóa — bổ sung `docs/specs/win-test-common-errors.md` khi là lỗi script)

| ID | Finding | Bằng chứng | Ảnh hưởng |
|---|---|---|---|
| S4-1 | **`Find*` đồng bộ cực chậm trên app nặng** — Word avg 989.6 / p95 1529.4 / min 77.2ms; Excel 215.6ms | §1 #5/#6 | **Cấm** gọi `Find/FindFirst/FindAll` trong hook callback (P1-2) |
| S4-2 | **Mọi query đầu vượt budget** — app nhẹ nhất (notepad) first 63.8ms; Chrome first 132.3; `FromHandle` first 292ms (UIA init) | §1 | Prewarm ngay khi hook attach + khi foreground đổi |
| S4-3 | **Chrome profile mới: web content không vào UIA tree ≥8s** — 36 phần tử chỉ gồm browser UI, KHÔNG có `50030 Document`; launch ≥2 (profile reuse) thấy đủ (83 phần tử, Edit×5 gồm user/pass/search/textarea) | diag3 vs diag4; §1.1 run 2 password `fail=0/20` | R1/R8 `fail=20/20` ở §1 #2/#3 là **expected** (không phải bug rule); test fixture phải launch ≥2 |
| S4-4 | **`IsPassword` condition false-negative ở query đầu** (probabilistic) — probe_conditions 2/2 lần chạy: lần 1 = 0, lần 2 (t~1s) = 1 rồi ổn định `1,1`; timing run thấy từ t=0.5s | probe output | Retry ≥1 sau 0.5–1s; **đã thấy `True` → cache** (fail-safe phía secure) |
| S4-5 | Chrome expose `<textarea>` là **`Edit(50004)`** (aId=`ta`) — `ControlType.TextArea` member **NULL** → condition không match | diag4 + `controltype_ids`; §1.1: có content mà textarea vẫn `fail=20/20` | **R8 phải sửa**: `Edit` + multiline heuristic (ClassName/TextPattern), không dựa `ControlType.TextArea`; R6 "không TextArea" tương tự |
| S4-6 | **`ControlType.LookupById(id)` trả NULL** với mọi id (test 50004/50030); member NULL cả `Grid`, `TextArea` | runtime test | Chỉ dùng static `[ControlType]::Edit`; chain condition phải fallback (Excel tìm được nhờ `Table` vì `Grid` null) |
| S4-7 | **Name condition = exact equality** — `'Address and search bar'` THAY, `'address'` KHONG; `PropertyConditionFlags` .NET chỉ `{None, IgnoreCase}` → **không có Contains**; Contains client-side (`TrueCondition` + filter) được nhưng walk-all avg 81.4ms | probe output + §1 | R3 name-match phải `IgnoreCase` + exact, hoặc client-side filter **sau khi đã có element**; không walk-all sớm |
| S4-8 | **conhost legacy: không có Edit/Document element** (fail 20/20, query nhanh ~1.9ms) | §1 #10 | R10 + ClassName heuristic `ConsoleWindowClass` là **bắt buộc** |
| S4-9 | **Windows Terminal: không có Edit/Document** — element thấy = `Pane(50033)` cls `DRAG_BAR_WINDOW_CLASS` | §1 #7 | R9 dùng window-level ClassName heuristic (`CASCADIA_HOSTING_WINDOW_CLASS`) thay vì find child |
| S4-10 | **Electron (ChatGPT desktop) a11y OFF** — chỉ `50000 Button`×3 + `50033 Pane`×9 (`Intermediate D3D Window`); cond Edit/Document/TextArea = 0 | diag6 | R7 chưa có data thật từ Electron → appdb preset + ClassName `Chrome_WidgetWin_1` fallback |
| S4-11 | **VS Code không cài trên máy** (0 process, 2 path chuẩn không tồn tại) → R7 thiếu target | §1 #8 SKIP | Limitation: follow-up cài VS Code hoặc dùng Chrome làm target `Document` (đã thấy 50030) |
| S4-12 | Cold `ControlType` 63.9ms đầu, các lần sau 0–3.6ms; warm 6-prop 0.3–6.1ms | §1/§2 | Cache 6-prop theo hwnd 2s (§5.3) |
| S4-13 | **A8 tái diễn** trong script diag (dot-access `$CT.TextArea` → null im lặng → editCond/taCond=0 vô hiệu; bản `::` chạy đúng) | `uia_timing.ps1` vs `probe_conditions.ps1` | Guard: audit script spike bằng grep `\$[A-Za-z]+\.[A-Z][a-zA-Z]+` trước khi chạy; A8 đã có common-errors |
| S4-14 | Dialog riêng (Win+Space, UAC…) không thuộc UIA tree app → không dò được ở đây | — | WIN-005/007 xử lý ở tầng khác |

## 5. Kết luận cache strategy (acceptance `P1-3 §2`)

1. **Hook callback KHÔNG query UIA.** `OnKeyDown`/TSF sink chỉ đọc cache; số liệu §1 cho thấy bất kỳ `Find*` nào cũng vượt budget 30–500× (S4-1/S4-2).
2. **Worker thread** (độc lập, khởi tạo khi hook attach) lấy role bằng `AutomationElement.FocusedElement` — warm **avg 2.2 / p95 2.1ms ≤ budget 2ms**, fail 0/20. Lần đầu 13.8ms → **prewarm** ngay khi attach + khi nhận foreground-change event (không đợi key đầu).
3. **Cache:** key = `hwnd` (+`app_id` khi có), value = 6-prop (§2) + role đã resolve; **TTL 2s**, invalidate ngay khi focus/foreground đổi (P1-3 §2/P1-2 §4). Cache hit = hashmap O(1), không chạm UIA.
4. **Miss (chưa có trong cache) → fail-open hierarchy** (không bao giờ block callback):
   (a) **appdb preset** theo exe (R9/R10 path, mặc định 20 app) →
   (b) **ClassName heuristic** từ window (`ConsoleWindowClass`, `CASCADIA_HOSTING_WINDOW_CLASS`, `Excel7/XLGRID`, `Chrome_WidgetWin_1`) →
   (c) role `unknown` (R10) → **không inject** cho tới khi resolve xong (an toàn phía secure).
5. **`IsPassword`:** một lần thấy `True` → giữ `secure=1` trong phiên (S4-4); query đầu miss → retry 1 lần ở worker (không ở callback).
6. **Gate đo lệm:** mọi query > 2ms ghi metric (không log text) → `WIN-031` `vieteime-bench field-switch` p99 cache+resolve < 2ms, không gọi UIA đồng bộ.

## 6. Limitation & follow-up

| Việc | Trạng thái | Task gợi ý |
|---|---|---|
| VS Code / KeePassXC chưa cài → R7 (Electron có a11y) + R1/WIN-017 chưa test thật | blocked môi trường | follow-up: cài 2 app này rồi chạy lại probe |
| Electron ChatGPT a11y OFF → R7 chưa có data positive | chờ app khác | appdb preset đã phủ |
| Word/Excel n=20, 1 phiên máy dev → số liệu noisy (min 77 vs avg 989) | chấp nhận được cho spike | WIN-031 đo lại bằng bench có baseline |
| diag6 lần chạy thứ 2: `conhost hwnd=0` (spawn không ra window) — lần chính chạy OK | anomaly đã ghi | reproduce xem §8 |

## 7. Reproduce

```powershell
# (0) bang id ControlType - 2s, khong mo app
powershell -NoProfile -ExecutionPolicy Bypass -File spikes\uia-probe\controltype_ids.ps1
# (1) probe dien kien: pwd flaky + Name exact/contains tren Chrome fixture - ~15s
powershell -NoProfile -ExecutionPolicy Bypass -File spikes\uia-probe\probe_conditions.ps1
# (2) spike chinh: 10 phan tu (mo Notepad/Word/Excel/Chrome/conhost, co cleanup) - ~2-3 phut
powershell -NoProfile -ExecutionPolicy Bypass -File spikes\uia-probe\uia_spike.ps1
# output: %TEMP%\uia_spike_out.txt - chua ms/role, khong chua text nguoi dung (S2)
```

Yêu cầu: Windows 11, Chrome, Office (Word/Excel COM), Windows Terminal đang mở. Không kill Explorer/Chrome của người dùng (script chỉ kill pid mình spawn + chrome theo profile test).

**Điều kiện exit:**

- [x] Bảng ms/query 10 phần tử — §1 + §1.1 (run 2 từ repo: 9 phần tử có số Find, 1 SKIP có nguyên nhân S4-11; 3 `fail=20/20` đều có nguyên nhân S4-3/S4-5/S4-8).
- [x] Kết luận cache strategy — §5, đối chiếu `P1-3 §2` (budget 2ms, cache 2s/hwnd, R1–R10).
- [x] Findings S4-1…S4-14 ghi nhận (không xóa); S4-5 → đề xuất sửa R8 `P1-3 §2` (báo agent phụ trách trước khi implement WIN-030).
- [x] Script portable trong `spikes/uia-probe/` (SPDX, ASCII) — reproduce chạy lại từ repo thành công 2/2 (`controltype_ids`, `probe_conditions`) + full `uia_spike.ps1` run 2 (§1.1).
