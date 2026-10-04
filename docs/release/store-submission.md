# Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)

> Trạng thái: hướng dẫn cho chủ repo, áp từ 0.2.8; **khớp từng ô của form
> Partner Center** (cập nhật 2026-10-03 theo hướng dẫn thực tế của form).
> Bộ cài Inno Setup đã hỗ trợ đủ cờ im lặng chuẩn và CI kiểm chứng cả hai kịch
> bản silent mỗi lần release (`installer/windows/tests/test-installer.ps1`:
> per-user `/CURRENTUSER` + machine `/ALLUSERS`, đều chạy `/VERYSILENT`).

## 0. Checklist S — BẮT BUỘC 100% trước khi nộp (không bỏ bước nào)

| # | Bước | Bắt buộc | Được kiểm tự động bởi |
|---|---|---|---|
| S1 | Bản phát hành có đủ 8 asset (7 + `*.msix`), CI 4/4 job xanh | ✔ | `release.yml` publish validate |
| S2 | **Store package validation** (silent / ARP / bundleware / uninstall) PASS trên runner sạch | ✔ | `tools/win/validate-store-package.ps1` — chạy trong `ci-shared` + `release.yml` + `cargo xtask preflight` |
| S3 | Setup exe được đưa lên branch `approved` tại `vX.Y.Z/` (raw URL 200) | ✔ | quy trình B7c + `curl -sI` kiểm 200 |
| S4 | Ảnh listing đủ: box art 1:1 ≥1080px + poster 2:3 + ít nhất 1 screenshot ≥1366×768 | ✔ | `store/art/` + `store/screenshots/` (generator: `scripts/generate_store_art.py`) |
| S5 | Ô switches = `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` (KHÔNG `/ALLUSERS`, KHÔNG `/CURRENTUSER`) | ✔ | mục §1a |
| S6 | KHÔNG tích "Installer runs in silent mode but does not require switches" | ✔ | mục §1b |
| S7 | Languages: Vietnamese + English; App type: Desktop | ✔ | mục §1c/§1d |
| S8 | Return codes: successful = `0`, cancelled = `2`, còn lại bỏ trống | ✔ | mục §1e |
| S9 | Privacy policy URL = `PRIVACY_POLICY.txt` (raw/blob); Support URL = issues | ✔ | repo |
| S10 | Publisher display name trong Partner Center = `LinhBH.CoM` (khớp ARP) | ✔ | Bước 2 của validation đối chiếu tự động |
| S11 | Sau khi Submit: theo dõi Package validation — mọi mục đỏ phải được đưa thành mã lỗi (B*/E*) TRƯỚC khi nộp lại | ✔ | quy tắc B4⑤ |
| S12 | MSIX dự phòng sẵn sàng cùng phiên bản (khi exe bị từ chối) | ✔ | asset `*.msix` + §2c |

Vì sao bắt buộc cứng: lần nộp 0.2.16 đã đỏ 3 mục validation vì bộ cài đòi
admin trong sandbox (B13) — từ 0.2.17 mọi bước ở S2 chạy tự động trong CI mỗi
commit, một bước không đạt = không được tag/phát hành.

## 1. Điền form Partner Center — từng ô một

### 1a. "Provide any switches required for silent installation…"

Dán **đúng chuỗi này**:

```
/VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

- **Bộ cài PHẢI là per-user (`PrivilegesRequired=lowest`, manifest `asInvoker`)** —
  ĐÍNH CHÍNH 0.2.20: kết luận cũ "phải là machine install (admin/HKLM)" là **SAI**.
  Validator chạy installer bằng `CreateProcess` **non-elevated**: exe
  `requireAdministrator` fail NGAY `ERROR_ELEVATION_REQUIRED` (740) trước khi cài
  gì cả → cả 3 mục đỏ (đo thật: exit=2 — B13e). Với per-user, entry Add/Remove
  nằm ở HKCU và **Programs and Features hiển thị gộp cả hai hive** → validator
  vẫn thấy; gateway tự động là bước 0 của `validate-store-package.ps1` (manifest
  `asInvoker` + ARP exact từng chữ). **silent luôn exit 0** giữ từ 0.2.17.
- **Mỗi lần nộp phải là phiên bản mới + cập nhật ô Package URL** — nộp lại cùng
  gói/URL thì Partner Center báo *"We did not find any changes in the Package or
  the Silent install parameters"* và **không chạy lại validation** (kết quả 3 mục
  đỏ cũ còn nguyên — B13g).
- **Không** thêm `/CURRENTUSER`/`/ALLUSERS` vào ô của Store — mặc định đã đúng.

| Tham số | Tác dụng |
|---|---|
| `/VERYSILENT` | Không hiện bất kỳ cửa sổ wizard nào (Inno Setup chuẩn). |
| `/SUPPRESSMSGBOXES` | Chặn mọi hộp thoại phụ (cảnh báo, ghi đè…). |
| `/NORESTART` | Không bao giờ khởi động lại máy sau cài. |

### 1a-bis. Trạng thái kỳ vọng của "Package validation"

**Chuỗi định danh CHỐT (chủ tài khoản xác nhận 2026-10-04, Partner Center):**

| Trường | Giá trị ĐÚNG | Ghi vào installer |
|---|---|---|
| Tên sản phẩm đã reserve (Product name) | `TextVN` | ARP `DisplayName` = `TextVN` (`AppVerName`) — **khớp ✓** |
| Publisher display name | `Linh βùi` (beta **U+03B2**, u-grave U+00F9) | ARP `Publisher` = `Linh βùi` (`AppPublisher`) — **từ 0.2.21** |

- ✅ **Malware check — clean** (đúng như đã thấy).
- ✅ **Silent install check — pass** khi: manifest exe là **asInvoker** (gate bước 0
  của `validate-store-package.ps1`; `requireAdministrator` = validator
  `CreateProcess` non-elevated fail 740 NGAY trước khi cài → cả 3 mục đỏ — B13e),
  silent exit 0, không dialog ngôn ngữ (`ShowLanguageDialog=no`).
- ✅ **Entry in add or remove programs / Bundleware check — pass** khi: đúng **1**
  entry mới, ARP `DisplayName == "TextVN"` (AppVerName), `Publisher == "Linh βùi"`,
  có `DisplayVersion` — harness so **exact từng chữ** (B13f/STO-03) và in
  codepoint khi lệch. Entry per-user nằm ở HKCU: Programs and Features hiển thị
  gộp cả hai hive.
- ✅ **Code sign check — valid** (Store ký lại khi publish).
- ⚠️ **BẪY "We did not find any changes in the Package or the Silent install
  parameters"** (B13g): nộp lại cùng gói/cùng URL → Partner Center **không chạy
  lại** validation, 3 mục đỏ cũ còn nguyên. Mỗi lần nộp phải là **phiên bản mới**
  (byte mới → hash mới) và **cập nhật ô Package URL** trỏ đúng file mới trong
  branch `approved`.

### 1a-ter. Nếu Store VẪN báo 3 mục đỏ sau 0.2.20 — hai giả thuyết còn lại (vòng 4)

Phân tích vòng 4 (STO-02/STO-03 trong báo cáo audit) chỉ ra 2 nghi vấn độc lập;
harness đã có công cụ kiểm riêng cho từng nghi vấn:

1. **Validator chỉ đọc hive MÁY (HKLM/WOW6432Node) hoặc chạy dưới account khác**
   → entry per-user (HKCU) vô hình. Hai bước kiểm chứng:
   ```powershell
   # (a) đối chiếu với gói ĐANG nộp (per-user): bằng chứng phải nằm ở HKLM
   powershell -File tools\win\validate-store-package.ps1 -MachineOnly `
       -Setup dist\TextVN-setup-<ver>-windows-x64.exe      # → sẽ FAIL: đúng như dự đoán
   # (b) dựng BIẾN THỂ MÁY rồi kiểm lại (chạy từ shell ELEVATED):
   ISCC.exe /DMyAppVersion=<ver> /DMachineInstall=1 "/DOutputSuffix=-machine" installer\windows\TextVN-setup.iss
   powershell -File tools\win\validate-store-package.ps1 -MachineOnly `
       -Setup dist\TextVN-setup-<ver>-windows-x64-machine.exe   # → phải PASS
   ```
   Nếu (b) PASS → nộp bản `-machine` (kèm `/ALLUSERS` trong ô switches) **với điều
   kiện VM của Store được phép elevate**; nếu VM không elevate được thì bản máy
   cũng fail y như 0.2.18 (đo thật: shell non-elevated → exit=2) → chỉ còn MSIX.
   ⚠️ Lưu ý đã đo: `PrivilegesRequired=admin` + `PrivilegesRequiredOverridesAllowed`
   ⇒ manifest vẫn là **asInvoker** (Inno tự relaunch elevated) — đừng dùng manifest
   để phân biệt hai biến thể.
   CI `ci-shared` có bước "Machine-install variant" chạy đúng (b) trên runner
   elevated để lấy bằng chứng mỗi commit.
   **Bằng chứng 2026-10-04 (run 37198638991, runner sạch elevated):**
   `PASS 0)` manifest asInvoker → `PASS 1)` silent install **exit 0 trong 1s** →
   `PASS 2)` **ARP entry 'TextVN' | 'LinhBH.CoM' | '0.2.20' ở HKLM** (`-MachineOnly`)
   → `PASS 3)` đúng 1 entry → `PASS 4)` gỡ cài sạch. Tức biến thể máy là lựa chọn
   **thật** khi môi trường cài được phép elevate; ngược lại phải dùng MSIX.
2. **Lệch chuỗi định danh so với Partner Center (STO-03)** — ARP DisplayName
   phải khớp **từng chữ** với tên sản phẩm đã reserve (ví dụ nếu Partner Center
   là `TextVN - Bộ gõ tiếng Việt` thì `TextVN` là MISMATCH), và Publisher phải
   khớp **Publisher display name** (danh tính tài khoản, ví dụ `Bùi Hùng Linh` —
   KHÔNG phải `LinhBH.CoM`). Kiểm bằng biến môi trường:
   ```powershell
   $env:STORE_APP_NAME='<tên thật trên Partner Center>'
   $env:STORE_PUBLISHER_NAME='<publisher display name thật>'
   powershell -File tools\win\validate-store-package.ps1
   ```
   Harness sẽ FAIL nếu ARP hiện tại lệch → build lại installer với đúng chuỗi
   (đã tham số hoá sẵn):
   ```powershell
   ISCC.exe /DMyAppVersion=<ver> "/DMyAppName=<tên thật>" "/DMyAppPublisher=<publisher thật>" installer\windows\TextVN-setup.iss
   ```
   ⚠️ Hai giá trị này CHỈ chủ tài khoản đọc được (Partner Center → Product
   identity / Publisher display name) — không suy đoán.
3. **MSIX (đường Microsoft khuyến nghị cho đúng ca fail này)** — bỏ qua cả 3
   check. Cần đúng 2 giá trị Product identity; script cảnh báo rõ khi còn
   placeholder và nhận giá trị qua biến môi trường để CI/thợ build lại một lệnh:
   ```powershell
   $env:MSIX_IDENTITY_NAME='<Package/Identity/Name>'; $env:MSIX_PUBLISHER='CN=<Publisher>'
   powershell -File tools\win\build-msix.ps1     # upload .msix TRỰC TIẾP, không qua URL
   ```

### 1b. "Installer runs in silent mode but does not require switches"

**KHÔNG tích** — bộ cài của TextVN CẦN các switch ở 1a (không tự silent mặc định).

### 1c. "Select the languages that your application supports"

Tích **Vietnamese** và **English** (installer + UI có cả hai).

### 1d. "Select your app type"

Chọn **Desktop** (hoặc **Win32/PC** tuỳ dropdown) — KHÔNG phải UWP/MSIX trừ khi
nộp gói `.msix` (§2c).

### 1e. EXE Return Codes

- **Documentation URL**: `https://jrsoftware.org/ishelp/index.php?topic=setupexitcodes`
- **Installation successful**: `0`
- **Installation cancelled by user**: `2`
- **Các kịch bản còn lại (already exists / in progress / disk full / reboot /
  network / rejected…)**: **bỏ trống** — Inno gộp lỗi nghiêm trọng vào mã 3/4;
  Store cho phép bỏ trống các kịch bản không hỗ trợ.
- **Lưu ý riêng của TextVN**: khi đăng ký TSF thất bại, bộ cài trả **exit code
  10** (không khai báo trong form) → Store coi là *cài thất bại* thay vì báo
  thành công mà app không gõ được — **đúng chủ đích, không cần khai báo thêm**.

## 2. Hành vi của bộ cài trong chế độ silent (đã kiểm chứng)

- Không có trang wizard chặn: `DisableDirPage=yes`, trang Welcome/Ready/Finish
  bị bỏ qua tự nhiên trong `/VERYSILENT`.
- Hai `[Run]` entry `runhidden runasoriginaluser` (`textvn-cli config init` và
  `TextVN.exe --free-ctrl-shift`) là lệnh CLI **tự thoát**, không mở UI.
- Entry tự mở TextVN sau cài có cờ `skipifsilent` → silent không khởi chạy app.
- Lỗi đăng ký TSF KHÔNG hiện hộp thoại: `WizardSilent` guard; báo lỗi qua
  **exit code 10** (`GetCustomSetupExitCode`), thành công = 0.
- Gỡ cài đặt cũng im lặng được: `unins000.exe /VERYSILENT /SUPPRESSMSGBOXES
  /NORESTART`.
- Cấu hình người dùng (`%APPDATA%\TextVN`) được giữ lại sau gỡ.

## 2a. Ảnh listing (Box art / Poster art)

Ảnh cho phần **Store listing** (khác tile trong gói MSIX):
`store/art/` (sinh bằng `python scripts/generate_store_art.py` — nền đỏ cờ VN +
sao vàng + chữ V trắng, đồng bộ icon mode tiếng Việt [V]).

| Ô trong Partner Center | File | Kích thước | Ghi chú |
|---|---|---|---|
| 1:1 Box art (BẮT BUỘC) | `box-art-2160.png` (hoặc `box-art-1080.png`) | 2160×2160 (1080×1080) | PNG, 152 KB (< 50 MB ✓) |
| 2:3 Poster art (khuyến nghị) | `poster-art-1440x2160.png` (hoặc `poster-art-720x1080.png`) | 1440×2160 (720×1080) | PNG, 119 KB |

Bản sao tải nhanh (raw URL, không redirect):

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/store-art/box-art-2160.png
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/store-art/poster-art-1440x2160.png
```

## 2b. Nguồn tải cho Partner Center — dùng branch `approved` (raw URL, KHÔNG redirect)

`https://github.com/.../releases/download/...` trả **redirect** sang URL ký tạm
thời; một số luồng upload của Store xử lý kém loại URL này. Vì vậy các bản
**đã được chủ repo duyệt** được copy (byte-identical) sang branch **`approved`**:

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/<ver>/TextVN-setup-<ver-số>-windows-x64.exe
```

Ví dụ **bản hiện hành v0.2.21** (đã kiểm: HTTP 200 trực tiếp, 0 redirect,
byte-identical với asset của GitHub Release — SHA-256
`fa486e6d9d2fda9345009d31b1b07ab0fbbd6740534b7236c8c17d860572d279`; đã được CI
validate lại từ chính URL này: silent exit 0 + `ARP entry Name='TextVN'
Publisher='Linh βùi' Version='0.2.21'`):

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.21/TextVN-setup-0.2.21-windows-x64.exe
```

Cập nhật khi có bản mới được duyệt: tải setup exe + portable zip +
`SHA256SUMS.txt` của tag tương ứng vào thư mục `vX.Y.Z/` trên branch
`approved` rồi push (xem README của chính branch đó). Xác minh sau khi push:
`curl -sI <raw-url>` phải trả `200` và `Content-Length` đúng cỡ file.

## 2c. Dự phòng MSIX (khi gói .exe bị Store từ chối)

Gói MSIX được build kèm **mọi** release (bắt buộc từ 0.2.17; publish job chặn nếu thiếu)
và nằm trong branch `approved` tại `vX.Y.Z/TextVN-<ver>-windows-x64.msix`.

**Bản chất gói này:** bộ gõ TSF không chạy được trong sandbox MSIX, nên gói
dùng **`runFullTrust`** — đóng vai trò kênh phân phối: cài xong, người dùng mở
TextVN một lần, ứng dụng tự đăng ký TSF như bản portable (0.2.16: tự đề nghị
đăng ký phạm vi máy qua UAC nếu Windows từ chối per-user). Chi tiết đóng gói:
`installer/windows/msix/` + `tools/win/build-msix.ps1`.

**Khi nộp:**

1. **Nộp UNSIGNED** — Store ký lại khi publish. Không cần cert.
2. **Product type phải hỗ trợ MSIX**: luồng "EXE/MSI" hiện tại (mục §1) chỉ
   nhận exe/msi. Muốn nộp MSIX phải dùng sản phẩm/loại gói hỗ trợ MSIX trong
   Partner Center (nếu không thấy lựa chọn, tạo product mới dạng MSIX/PWA).
3. **Publisher/Identity phải khớp Partner Center**: sau khi reserve tên, mở
   *Product identity* trong Partner Center lấy `Package/Identity/Name` và
   `Package/Identity/Publisher`, rồi build lại đúng:
   ```
   powershell -NoProfile -ExecutionPolicy Bypass -File tools\winuild-msix.ps1 `
     -Publisher "CN=<Publisher từ Partner Center>" -IdentityName "<Name từ Partner Center>"
   ```
   (Manifest sai Publisher là lỗi upload phổ biến nhất.)
4. Sau khi cài từ Store, lần chạy đầu tiên người dùng mở TextVN để hoàn tất
   đăng ký bộ gõ (hướng dẫn này nên ghi trong phần Description của listing).

**Cảnh báo certification:** IME cần ghi registry/COM (TSF TIP) — chính vì thế
mới phải `runFullTrust`. Nếu reviewer hỏi, giải trình: đây là bộ gõ hệ thống,
quyền full-trust là bắt buộc về mặt kỹ thuật; dữ liệu xử lý 100% cục bộ (dẫn
`PRIVACY_POLICY.txt`). Đường exe/msi (§1) vẫn là đường chính.

## 3. Checklist trước khi submit

1. [ ] Tải `TextVN-setup-<ver>-windows-x64.exe` từ GitHub Release (có
       `SHA256SUMS.txt` — ghi lại hash để điền vào Partner Center nếu được hỏi).
2. [ ] Test local trên máy sạch: chạy với đúng bộ tham số ở §1, kiểm exit
       code 0 và gõ được tiếng Việt trong Notepad sau khi mở TextVN từ Start
       Menu (silent KHÔNG tự mở app).
3. [ ] Điền Privacy policy URL (xem `PRIVACY_POLICY.txt` ở gốc repo — dùng
       link GitHub blob/raw của file này; cập nhật nội dung ngay trong repo
       khi có thay đổi).
4. [ ] Điền Description/Notes theo `README.md` mục "Tính năng".
5. [ ] Giấy phép: GPL-3.0-or-later (khai báo đúng ở mục Legal).

## 4. Nếu Store từ chối

- **"Installer requires elevation"**: thêm `/CURRENTUSER` vào tham số.
- **"Installer shows UI"**: kiểm tra lại đã dùng `/VERYSILENT` (không phải
  `/SILENT` — `/SILENT` vẫn hiện thanh tiến trình).
- **"App does not launch after install"**: đúng hành vi — silent không tự mở
  app; người dùng mở từ Start Menu. Không thêm auto-launch cho luồng Store.

## 5. Quy trình publish lên Microsoft Store — từng bước

1. **Tài khoản**: đăng ký [Partner Center](https://partner.microsoft.com/dashboard)
   (tài khoản cá nhân ~$19 hoặc công ty ~$99, thuế/ID xác minh một lần).
2. **Reserve app name**: Apps and Games → Overview → New product → Name.
   Đặt đúng tên hiển thị: `TextVN - Bộ gõ tiếng Việt` (hoặc giữ tên đã reserve).
3. **Chuẩn bị gói**:
   - Tải `TextVN-setup-<ver>-windows-x64.exe` từ GitHub Release (khuyên dùng
     bản có cờ CI xanh; `RELEASE_REPORT.json` bên trong ZIP portable cho biết
     commit + checksum).
   - Ghi lại SHA-256 từ `SHA256SUMS.txt`.
4. **Tạo submission**: Start submission →
   - **Product name / description**: chép từ `README.md` mục mô tả + ghi rõ
     điểm khác biệt (xem README "Điểm vượt trội so với các bộ gõ khác").
   - **Installer parameters** (quan trọng nhất):
     `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER`
   - **Privacy policy URL**: link GitHub blob của `PRIVACY_POLICY.txt`
     (`https://github.com/hunglinhpt/TextVN/blob/main/PRIVACY_POLICY.txt`).
   - **Support URL**: `https://github.com/hunglinhpt/TextVN/issues`.
   - **Website**: `https://linhbh.com` (nếu đã có site; không thì dùng repo).
   - **Age rating**: điền survey — kết quả thường là E (Everyone) vì không có
     nội dung nhạy cảm; IME không thu thập dữ liệu nên khai báo không có quyền.
   - **Screenshots**: chụp Bảng điều khiển (màn 1), gõ Telex trong Notepad
     (màn 2), menu khay hệ thống (màn 3) — 1366x768 trở lên, PNG/JPG.
5. **Certification**: đợi review (thường 1–3 ngày cho bản .exe silent). Nếu
   bị từ chối, tra §4 bên dưới trước khi nộp lại.
6. **Sau khi pass**: chọn Publish; listing hiện lên Store trong vài giờ.
7. **Bản cập nhật**: mỗi lần phát hành 0.2.x, lặp lại bước 3–6 với installer
   mới; tham số giữ nguyên. Label submission với tên phiên bản để dễ tra.

## 6. Loại gói và chứng thư số

> Cập nhật 2026-10-02 (điều chỉnh theo thực tế Partner Center + quyết định
> chủ repo): **Partner Center giờ chỉ nhận gói `.exe` hoặc `.msi`** (không còn
> luồng MSIX cho loại submission này). Bộ cài Inno Setup `.exe` của TextVN
> khớp loại này; luồng silent ở §1 là bắt buộc.

**Về chứng thư số:**

- **Luồng Store**: sau khi submission được **chứng nhận (certified)**, luồng
  phân phối của Store đảm bảo phần app được ký theo cơ chế của Microsoft —
  chủ repo sẽ có cert/ký từ luồng này, KHÔNG cần mua chứng thư riêng để phân
  phối qua Store. Điền mục chi phí/certificate theo hướng dẫn của Partner
  Center tại thời điểm submit.
- **Phân phối trực tiếp (GitHub Releases)**: bản tải trực tiếp vẫn là
  "unsigned" → SmartScreen hiện cảnh báo "Unknown publisher" — chấp nhận cho
  bản OSS hiện tại. Khi chủ repo đã có cert từ luồng Store (hoặc muốn bật
  sớm), pipeline đã có sẵn 2 nhánh opt-in không cần sửa code:
  1. **SignPath Foundation** (miễn phí cho OSS): thêm 4 secret
     `SIGNPATH_API_TOKEN` / `SIGNPATH_ORGANIZATION_ID` / `SIGNPATH_PROJECT_KEY` /
     `SIGNPATH_POLICY` → workflow tự ký 3 binary + bộ cài trước khi đóng gói.
  2. **Chứng thư riêng** (nếu dùng): đặt secret
     `SIGNTOOL_CERTIFICATE_THUMBPRINT`; `build-release.ps1 -SignCertificateThumbprint`
     ký bằng signtool (đã hỗ trợ sẵn).

### Lưu ý kiểm tra sau khi có cert/ký (bắt buộc trước khi release)
- `Get-AuthenticodeSignature <exe>` → Status = Valid.
- Bộ cài đã ký không được đổi sau khi upload lên Partner Center (hash khớp
  `SHA256SUMS.txt`).
