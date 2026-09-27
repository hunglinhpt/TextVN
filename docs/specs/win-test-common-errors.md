# Common errors khi test spike Windows (WIN-002/003) — đừng lặp lại

> Mục đích: ghi các lỗi **thật đã mắc** trong quá trình chạy spike TSF trên Windows 11,
> để agent/task sau không mất hàng giờ debug lại. Mọi mục đều có bằng chứng chạy thật.
> Quy ước ghi (G3, ID `A{n}` liền mạch, không xóa): `../00-WORKFLOW.md §3`.
> Liên quan: `tsf-spike.md`, `tsf-registration-spike.md`, `../20-windows/P1-5-test-plan.md`.

## A. PowerShell & C# interop

| # | Lỗi | Triệu chứng | Cách đúng |
|---|---|---|---|
| A1 | **Set field lồng nhau của value-type struct** | `$d.ki.wVk = 0x46` **im lặng không ghi** (PowerShell sửa trên 1 bản copy) → `SendInput` trả `2/2` nhưng inject **vk=0** → phím rỗng, không chữ, không log | Set trên biến riêng rồi gán cả struct: `$ki = New-Object T+KEYBDINPUT; $ki.wVk = 0x46; $d.ki = $ki`. Kiểm nhanh: `Marshal::StructureToPtr` + dump bytes |
| A2 | **Sai kích thước struct `INPUT`** | `SendInput` → `err=87 ERROR_INVALID_PARAMETER` | Trên **x64** `sizeof(INPUT) = 40` (type 4 + pad 4 + union 32). Struct C# phải `type` + `ki` + `IntPtr _unionTail`; luôn in `Marshal::SizeOf` ra console |
| A3 | **`$LASTEXITCODE` / `$_` / `$var` bị shell nuốt** | Lỗi parse `ExpectedValueExpression` khi chạy `powershell -Command "..."` inline | **Luôn** viết file `.ps1` rồi chạy `-File` |
| A4 | Dùng `&&` trong PowerShell 5.1 | Parse error | Dùng `;`, hoặc `if ($LASTEXITCODE -eq 0) { … }` |
| A5 | Script `.ps1` chứa Unicode/emoji | Chữ mojibake trong console/log | `.ps1` **ASCII-only** (quy ước Handlog); unicode test data đặt trong console output codepoint `U+XXXX` |
| A6 | Redirect `*>` ra file rồi đọc bằng read-tool | `Cannot read binary file` | File là **UTF-16** → đọc `Get-Content -Encoding Unicode` |
| A7 | C# trong `Add-Type` thừa biến | `CS0219` warning → fail (some hosts treat as error) | Xóa biến thừa; biến dùng 1 lần đặt tên `_` không được trong C# → dùng ngay |
| A8 | **Truy cập static member qua biến holding type** | `$UAE = [AutomationElement]` rồi `$UAE.NameProperty` → **`$null` im lặng** (PowerShell không resolve static qua instance lookup) → `PropertyCondition` ctor lỗi "Value cannot be null" | Luôn dùng `[Namespace.Type]::StaticMember`; biến type chỉ để truyền kiểu, không để lấy static. (Đã mất 2 lần chạy vì lỗi này.) |
| A9 | `[UAE]::Method()` với type accelerator tự đặt | `Unable to find type [UAE]` | Type accelerator phải đăng ký thật; biến `$UAE` không dùng được trong `[...]` → ghi đủ namespace hoặc alias có thật |
| A10 | **`[Type]::Member` truyền vào function không bọc ngoặc** | `PCC [System.X]::NameProperty 'v'` → PowerShell parse `[System.X]` thành type literal rời, `::NameProperty` thành arg riêng → ctor "Cannot find an overload ... argument count 2" | Luôn `PCC ([System.X]::NameProperty) 'v'` — bọc ngoặc `()` quanh `[Type]::Member` khi truyền vào function/scriptblock |
| A11 | **Type lồng nhau trong C# `Add-Type` dùng `.` thay vì `+`** | `[HK.LowLevelProc]{...}` → `Unable to find type [HK.LowLevelProc]` (TypeNotFound); script dừng giữa chừng | Type literal PowerShell cho nested type = **`[Outer+Inner]`** — ví dụ `[HK+LowLevelProc]`, `[HK+KEYBDINPUT]` (đã mất 1 lần chạy hook spike) |
| A12 | **`WH_KEYBOARD_LL` cần message pump trên thread cài hook** | (a) `[Thread]::Start({...})` → `does not contain a method named 'Start'` (Start là instance); (b) bỏ qua pump → hook bị hệ thống gỡ âm thầm sau ~300ms (`LowLevelHooksTimeout`), callback không bao giờ chạy | Cài hook trên thread có loop: PowerShell → `Add-Type System.Windows.Forms` + pump `[Application]::DoEvents()` định kỳ trong mọi `Start-Sleep` chờ (PumpSleep) |
| A13 | **Dùng `lock` (keyword C#) trong phần PowerShell** | `lock ([HK]::Recv) { ... }` → `CommandNotFoundException: The term 'lock' is not recognized` | PowerShell không có `lock` → `[System.Threading.Monitor]::Enter/Exit`, hoặc bỏ đi khi callback cùng thread (pump `DoEvents` = đồng bộ) |
| A14 | **`powershell -File script.ps1 -Only a,b` truyền mảng thành MỘT string** | `$Only = @('chrome,word')` → filter `-notcontains 'chrome,word'` → **0 file được chạy**, output rỗng tưởng "không có app nào" (WIN-061) | Đầu script tách lại: `$Only = @($Only \| ForEach-Object { $_ -split ',' } \| Where-Object { $_ -ne '' })` — xem `verify_targets.ps1` dòng 13 |
| A15 | **Reflection `GetProperty` trên statics của `System.Windows.Automation.ControlType`** | `typeof(ControlType).GetProperty('Document')` → `$null` im lặng vì statics là **public static FIELD** (dùng `GetField`); thêm: .NET UIA chỉ có **39 type, KHÔNG có `TextArea`/`Grid`** → locator `TextArea` không match bao giờ (spec P1-3 R8 sai = F6-1, WIN-061) | Dùng helper `Get-UiAControlType` (lib `win32-uia.lib.ps1`, GetField); chỉ dùng 39 type thật — `check_targets.ps1` bắt control_type không tồn tại ngay lúc static check |

## B. Gửi phím & focus (dạng sai gây "phím bay vào app của user")

| # | Lỗi | Triệu chứng | Cách đúng |
|---|---|---|---|
| B1 | **Không verify focus ngay trước khi send** | Phím lọt vào VS Code/Claude của người dùng (đã xảy ra vài lần, user phải tự undo) | **Bắt buộc** in `GetGUIThreadInfo(0).hwndFocus` **trước MỖI burst**, đúng class + đúng pid thì mới `SendInput`; pause 3–5s giữa các burst vì user đang dùng máy song song |
| B2 | Chọn sai target window | `FindTopByPid` bắt window ẩn/`MainWindowHandle` = launcher stub → focus fail âm thầm | Lọc `IsWindowVisible` + class không phải `Chrome_*`; xác nhận `GetForegroundWindow` pid khớp |
| B3 | Gửi rồi không đọc kết quả | `SendInput=2/2` nhưng text rỗng → tưởng TIP lỗi | Sau send: poll text (`WM_GETTEXT`) **và** log TIP; nếu text rỗng mà `SendInput` thành công → nghi struct/focus (A1/A2/B1), **không** vội kết luận TIP fail |
| B4 | Dùng `SendKeys` làm bằng chứng cho UIPI test | `SendKeys` đi đường khác, kết quả không nói lên `SendInput` bị chặn | Test #10 phải test đúng API cần kết luận; `keybd_event`/`SendKeys` chỉ dùng để đối chiếu |

## C. UIA & ứng dụng test

| # | Lỗi | Triệu chứng | Cách đúng |
|---|---|---|---|
| C1 | Tìm WinForms TextBox theo `ControlType=Edit` hoặc `AccessibleName` | Không tìm thấy (0 kết quả) | WinForms expose là **`ControlType.Pane` + class `WindowsForms10.EDIT.*`**, Name rỗng; tìm theo `ClassNameProperty` |
| C2 | Nhấn Button bằng `InvokePattern` | `Pattern not supported` | Legacy WinForms button cũng là `Pane` → dùng focus + `Space`, hoặc P/Invoke `BM_CLICK` |
| C3 | Đọc text WinForms qua `ValuePattern` | Không supported | Đọc cross-process bằng `SendMessage(WM_GETTEXT=0x000D)` (cùng desktop thì cho phép) |
| C4 | Tưởng `IsPassword` đáng tin | Native `ES_PASSWORD` (GWL_STYLE=0x20 xác nhận) vẫn trả `IsPassword=False` | **Không** dựa `IsPassword` cho field detect → xem `tsf-spike.md` #9, cần lớp detect riêng (P1-3) |
| C5 | Test CredUI/`CredentialUIBroker.exe` bằng UIA | Subtree = 1, không đọc được field | Secure host **không expose descendants** — đây là kết quả spike (#9), không phải bug test |
| C6 | Dùng Notepad Win11 làm target **elevated** | `Start-Process -Verb RunAs` → pid là launcher, window thuộc instance MSIX khác (medium) | Tự build app test riêng: `Add-Type -OutputAssembly tfield.exe` (WinForms có TextBox) — `make_tfield.ps1` |
| C7 | Quên kill process test cũ | Form cũ giữ focus/foreground, kết quả bị lẫn | Trước mỗi run: `Get-Process tfield,notepad,… \| Stop-Process -Force` + `Start-Sleep` |

## D. Build & DLL

| # | Lỗi | Triệu chứng | Cách đúng |
|---|---|---|---|
| D1 | **DLL TIP bị lock khi rebuild** | `failed to remove tsf_min.dll — Access is denied (os error 5)` | DLL bị **load vào chính các process mà TIP activate** (`tasklist /m tsf_min.dll` → explorer/claude/OpenCode/msedge). Đóng app đó (hoặc build `--bin tsf-min-register` — exe không lock) |
| D2 | Sửa `register.rs` mà không rebuild được cdylib | Tương tự D1 | `cargo build --bin tsf-min-register` đủ cho CLI; đổi code TIP thì phải đóng app giữ lock |
| D3 | Quên `cargo fmt/clippy/deny/reuse` trước commit | CI đỏ | Chạy đủ 5 gate (Handbook §4) trước khi `git add` |

## E. TSF API (đã mắc thật — xem chi tiết trong 2 file spec)

| # | Lỗi | Triệu chứng | Cách đúng |
|---|---|---|---|
| E1 | Truyền `&[u16]` **không null-terminate** vào API đọc `wcslen()` | Readback chuỗi + **rác** phía sau; icon `&[]` (ptr 0x2) → **AV** | Slice đúng length nhưng allocation **có số 0 đệm sau**; mọi chỗ truyền `&[u16]` vào TSF đều vậy |
| E2 | Dùng `GetStart(ec)` làm anchor cho composition | Text bị ghi vào **đầu document** thay vì caret | Anchor = `GetSelection(TF_DEFAULT_SELECTION)`; `GetStart` = document start |
| E3 | Gọi `EnableLanguageProfileByDefault` không elevation rồi tưởng fail | `E_FAIL 0x80004005` | API ghi HKLM — ở user context fail là **bình thường**, bỏ qua (S3-3) |
| E4 | Đảo thứ tự `InstallLayoutOrTip` / `EnableLanguageProfileByDefault` | Enabled flag biến mất | Thứ tự cố định: InstallLayoutOrTip → Enable → Activate |
| E5 | Debug tiến trình elevated bằng console output | Không thấy gì (console bị ẩn) | Ghi log file %LOCALAPPDATA%\VietIME\logs\*.log qua helper `say()` |
| E6 | Giả định `OnTestKeyDown` luôn tồn tại | App khác nhau trả khác nhau (bảng `tsf-spike.md` #5) | Logic ăn phím đúng ở **cả 2 pha**; `TRUE` ở test ⇒ `OnKeyDown` **phải** xử lý thật |

## F. Quy trình

| # | Lỗi | Cách đúng |
|---|---|---|
| F1 | Sửa code/docs rồi… quên commit/-push | **Xong việc là commit + push ngay** (git add list tường minh, không `git add -A` vì workspace có agent khác song song) |
| F2 | Graph repo cũ sau khi đổi nhiều file | `graphify update .` sau mỗi lần commit để graph khớp code |
| F3 | Test mất kết quả vì timeout nuốt output | Chạy nền (`background`) + ghi output ra file .txt, đọc file sau khi xong |
