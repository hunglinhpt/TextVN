# Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)

> **Trạng thái hiện hành (2026-10-09):** đường nộp Store **CHÍNH là gói MSIX** — làm
> theo [`msix-submission.md`](msix-submission.md) (tài liệu chuẩn; chỉ còn thiếu
> `Package/Identity/Name` từ Partner Center). Đường **EXE** trong tài liệu này (bản
> `TextVN-setup-<ver>-windows-x64-machine.exe`) đã PASS 3 check tự động từ 0.2.24 nhưng bị
> policy **10.2.9** chặn cho tới khi exe có chữ ký Authenticode qua SignPath Foundation
> (đang chờ duyệt — `store-policy-10-2-9.md`); khi có secret SignPath, CI ký cả hai bộ
> cài kể cả `-machine.exe`. Các mục dưới giữ làm tài liệu cho đường EXE; phần nhật ký có
> ngày là lịch sử, không sửa.

> Trạng thái: hướng dẫn cho chủ repo, áp từ 0.2.8; **khớp từng ô của form
> Partner Center** (cập nhật 2026-10-03 theo hướng dẫn thực tế của form).
> Bộ cài Inno Setup đã hỗ trợ đủ cờ im lặng chuẩn và CI kiểm chứng cả hai kịch
> bản silent mỗi lần release (`installer/windows/tests/test-installer.ps1`:
> per-user `/CURRENTUSER` + machine `/ALLUSERS`, đều chạy `/VERYSILENT`).

## 0. Checklist S — BẮT BUỘC 100% trước khi nộp (không bỏ bước nào)

| # | Bước | Bắt buộc | Được kiểm tự động bởi |
|---|---|---|---|
| S1 | Bản phát hành đủ 34 asset: 8 artifact (portable zip, setup exe, setup `-machine.exe`, `*.msix`, Linux tar.gz, mac pkg/zip/tar.gz) + `SHA256SUMS.txt` (clearsign) + `gpg-release-key.asc` + `.asc`/`.cosign.sig`/`.cosign.cert` cho từng artifact; CI 4/4 job xanh | ✔ | `release.yml` publish validate + `signing.md` |
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
| S12 | Gói MSIX (đường Store CHÍNH) build với `Package/Identity/Name` thật (repo variable `MSIX_IDENTITY_NAME` → `-RequireStoreIdentity`), `verify-msix.py` + sideload test xanh | ✔ | [`msix-submission.md`](msix-submission.md) §0/§3 |

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

**Chuỗi định danh CHỐT (2026-10-05 — chủ tài khoản tìm ĐÚNG trang trong account
và TỰ SỬA Publisher display name thành `LinhBH.CoM`):**

| Trường | Giá trị ĐÚNG | Ghi vào installer |
|---|---|---|
| Tên sản phẩm đã reserve (Product name) | `TextVN` | ARP `DisplayName` = `TextVN` (`AppVerName`) — **khớp ✓** |
| Publisher display name | **`LinhBH.CoM`** (nguồn sự thật, chủ tài khoản chỉnh trong account settings) | ARP `Publisher` = `LinhBH.CoM` (`AppPublisher`) — **từ 0.2.23; 0.2.16–0.2.20 đã từng đúng** |

**Lịch sử 7 vòng giải thích trọn**: 0.2.16–0.2.20 chuỗi đã đúng nhưng thua vì
lý do khác (0.2.16 exit 10; 0.2.18 admin/740; 0.2.19 bẫy B13g "no changes" —
nộp lại URL cũ nên không revalidate); 0.2.21 `Linh βùi` và 0.2.22 `Linh Bui`
là hai lần đọc/suy đoán sai từ nguồn không phải trang Publisher info. Bài học
B18 bổ sung: giá trị nằm ở một trang riêng trong account — vào đúng trang đó,
dump codepoint, rồi mới tin.

- ✅ **Malware check — clean** (đúng như đã thấy).
- ✅ **Silent install check — pass** khi: manifest exe là **asInvoker** (gate bước 0
  của `validate-store-package.ps1`; `requireAdministrator` = validator
  `CreateProcess` non-elevated fail 740 NGAY trước khi cài → cả 3 mục đỏ — B13e),
  silent exit 0, không dialog ngôn ngữ (`ShowLanguageDialog=no`).
- ✅ **Entry in add or remove programs / Bundleware check — pass** khi: đúng **1**
  entry mới, ARP `DisplayName == "TextVN"` (AppVerName), `Publisher == "LinhBH.CoM"`,
  có `DisplayVersion` — harness so **exact từng chữ** (B13f/STO-03) và in
  codepoint khi lệch. Entry per-user nằm ở HKCU: Programs and Features hiển thị
  gộp cả hai hive. Xem §1a-ter cho lịch sử 7 vòng và kế hoạch nộp có thứ tự.
- ❌ **Code sign check (policy 10.2.9)**: Store **không** ký lại gói EXE nộp qua URL — bản máy 0.2.24 bị chặn vì exe chưa ký. Chỉ đạt khi exe có chữ ký Authenticode SHA-256 (SignPath Foundation — đang chờ duyệt); xem §6a và `store-policy-10-2-9.md`. (Store chỉ tự ký gói MSIX.)
- ⚠️ **BẪY "We did not find any changes in the Package or the Silent install
  parameters"** (B13g): nộp lại cùng gói/cùng URL → Partner Center **không chạy
  lại** validation, 3 mục đỏ cũ còn nguyên. Mỗi lần nộp phải là **phiên bản mới**
  (byte mới → hash mới) và **cập nhật ô Package URL** trỏ đúng file mới trong
  branch `approved`.

### 1a-ter. Lịch sử 7 vòng + kế hoạch nộp có thứ tự

**Kết cục vòng 5–7 (2026-10-04/05):** 0.2.21 (`Linh βùi`) và 0.2.22 (`Linh Bui`)
đều là chuỗi đoán sai — chủ tài khoản sau đó tìm đúng trang trong account và
**tự sửa Publisher display name thành `LinhBH.CoM`** (giá trị mà 0.2.16–0.2.20
vẫn ghi đúng; các vòng đó thua vì exit 10 / admin=740 / bẫy B13g). Không còn
nghi vấn nào về chuỗi định danh; hai nghi vấn phụ (HKCU vô hình; sandbox
non-elevated) là **kế hoạch dự phòng có thứ tự** dưới đây.

**Kế hoạch nộp có thứ tự (mỗi bước chỉ nộp khi bước trước đã đỏ THẬT — đã được
revalidate, không phải trạng thái cũ):**

**Bước 1 — per-user 0.2.23 (publisher đúng `LinhBH.CoM`):** ❌ **ĐÃ ĐỎ (vòng 9,
2026-10-05) — xem B19.** Định danh đúng nguồn-sự-thật, silent chuẩn, ARP đúng
từng chữ (CI chứng minh trên chính file nộp) mà vẫn đỏ 3 mục ⇒ EXE per-user
đã cạn; nguyên nhân nằm ngoài tầm ảnh hưởng của gói (validator đọc HKLM /
chạy dưới account khác / sandbox non-elevated / SmartScreen chặn exe chưa ký).

**Bước 2 — machine 0.2.24 (ARP ở HKLM) — THẺ EXE CUỐI CÙNG, nộp ngay:**

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe
SHA-256: 739a5c7b06b2634d43a0ea86a4ccf1d66a6c6585ed02c6173effbbbe8cfa5acb
```

0.2.24 hoàn thiện nốt 3 khe hở phía gói mà rà dòng lần 3 tìm ra: entry ARP giờ
có **`DisplayIcon`** (trước đây thiếu — verify bằng probe), metadata setup exe
đầy đủ **FileVersion** (trước đây trống), và **SetupLogging** ghi log vào
%TEMP% của VM validator — bằng chứng duy nhất có thể yêu cầu nếu fail lần nữa.

Cùng publisher đúng, cài vào Program Files + HKLM. **QUYẾT ĐỊNH CHỦ REPO
(2026-10-05): bản máy là bản Store CHÍNH THỨC từ 0.2.23** — hướng dẫn của
Microsoft KHÔNG cấm admin: tài liệu manual-package-validation ghi nguyên văn
*"Note — UAC (User Account Control) prompts are allowed"*. Khác biệt then chốt
với 0.2.18 đã thất bại: 0.2.18 dùng manifest `requireAdministrator` →
`CreateProcess` non-elevated của validator fail 740 TRƯỚC khi cài; bản máy hiện
tại giữ manifest `asInvoker` (do `PrivilegesRequiredOverridesAllowed`) và để
Inno tự relaunch elevated — VM có UAC "never notify" hoặc đã elevated thì cài
thông suốt, đúng kịch bản "UAC allowed" của tài liệu. PASS = dùng bản máy cho
mọi lần nộp sau (và hợp B7: machine-wide TSF là luồng gõ tin cậy nhất Win11
24H2). Vẫn đỏ = sandbox không cho elevation bằng bất kỳ cách nào → đường EXE
hết phương án, chuyển bước 3. Người dùng tải ngoài Store vẫn nhận bản per-user
(không UAC); **bản portable giữ nguyên** (zip, không ARP).

**Bước 3 — MSIX (triệt để, Microsoft khuyến nghị cho đúng ca fail này; bỏ qua
cả 3 check):** Windows publisher ID của tài khoản đã có:
`CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545` (dùng cho `Identity@Publisher`).
**Cập nhật 2026-10-09:** MSIX nay là đường nộp **chính** — làm theo
[`msix-submission.md`](msix-submission.md) §2–§4. Còn thiếu `Package/Identity/Name`
(trang Product identity của sản phẩm MSIX) và phải **build lại**: file `.msix` của
v0.2.27 không nộp được (identity placeholder, Version `0.2.27.0`, mô tả mojibake). Build
khuyến nghị qua CI (repo variable `MSIX_IDENTITY_NAME`); build tay:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
  -IdentityName "<Package/Identity/Name từ Partner Center>" -RequireStoreIdentity
```

(Publisher mặc định đã là `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`, PublisherDisplayName
`LinhBH.CoM`, DisplayName `TextVN` — đổi bằng `-DisplayName`/`MSIX_DISPLAY_NAME` nếu sản
phẩm MSIX dùng tên khác.) Upload `.msix` TRỰC TIẾP (không dùng Package URL).

**Đối chiếu giá trị thật trên CI**: repo VARIABLES đã đặt — `STORE_APP_NAME=
TextVN`, `STORE_PUBLISHER_NAME=LinhBH.CoM` → mọi bước validate của CI so ARP với
đúng 2 chuỗi này mỗi commit. Đổi giá trị khi Partner Center đổi (chủ tài khoản
thực hiện `gh variable set` hoặc Settings → Secrets and variables → Variables).

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
- **Lưu ý riêng của TextVN**: ở chế độ im lặng (luồng Store) bộ cài **chỉ chép
  file** — không gọi API đăng ký TSF, luôn exit `0` khi chép đủ (từ 0.2.19, sự cố
  B13); app tự đăng ký ở lần mở đầu. Exit `10` chỉ xảy ra khi cài **có giao diện** mà
  đăng ký TSF thất bại — không cần khai báo trong form.

## 2. Hành vi của bộ cài trong chế độ silent (đã kiểm chứng)

- Không có trang wizard chặn: `DisableDirPage=yes`, trang Welcome/Ready/Finish
  bị bỏ qua tự nhiên trong `/VERYSILENT`.
- `[Run]` `textvn-cli config init` có `skipifsilent` → **không chạy** khi `/VERYSILENT`.
  `TextVN.exe --free-ctrl-shift` (task `freectrlshift`, `runhidden`) **vẫn chạy** khi im
  lặng — lệnh tự thoát, không mở UI, dành Ctrl + Shift cho TextVN (từ CR-17 tray để bộ
  cài quyết định).
- Entry tự mở TextVN sau cài có cờ `skipifsilent` → silent không khởi chạy app.
- Silent **không đăng ký TSF** (`WizardSilent` → `RegistrationOK := True`), không hiện
  hộp thoại, exit `0`; đăng ký diễn ra khi người dùng mở TextVN lần đầu.
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

Ví dụ **bản hiện hành v0.2.29** (bản máy — file nộp của đường EXE; ARP
`Publisher='LinhBH.CoM'`):

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.29/TextVN-setup-0.2.29-windows-x64-machine.exe
SHA-256: 0e227c5900b77ce018ca9e5e51ffef20222d652a7a571b702b70e0723f85c25d
```

(khớp `SHA256SUMS.txt` đã clearsign của release; chưa ký Authenticode nên vẫn bị policy
10.2.9 chặn — đường EXE chỉ còn chờ SignPath Foundation. Gói MSIX nộp Store của cùng
bản: `approved/v0.2.29/TextVN-0.2.29-windows-x64-store.msix` — xem `msix-submission.md`.)

Cập nhật khi có bản mới được duyệt: chép các file Windows của tag (setup exe,
`-machine.exe`, portable zip, `.msix`) kèm `.asc`/`.cosign.sig`/`.cosign.cert` của
từng file, `SHA256SUMS.txt` và `gpg-release-key.asc` vào thư mục `vX.Y.Z/` trên branch
`approved` rồi push (xem README của chính branch đó). Xác minh sau khi push:
`curl -sI <raw-url>` phải trả `200` và `Content-Length` đúng cỡ file.

## 2c. MSIX — đường nộp Store CHÍNH

Đã chuyển sang tài liệu riêng, là nguồn chuẩn duy nhất:
[`msix-submission.md`](msix-submission.md) — trạng thái, cơ chế stage-out ra ngoài gói
(`%USERPROFILE%\.textvn\msix-staging` → `%LOCALAPPDATA%\Programs\TextVN-Store\<V>`, chỉ
đăng ký HKCU, guard mỗi lần đăng nhập dọn sạch sau khi gỡ gói — CI cài thử thật bằng
`installer/windows/tests/test-msix-sideload.ps1`), lựa chọn tên sản phẩm, build qua repo
variable `MSIX_IDENTITY_NAME`, map Version `A.B.C` → `(A+1).B.C.0`, ghi chú certification.

Gói `.msix` được build kèm **mọi** release (bắt buộc từ 0.2.17; publish job chặn nếu
thiếu). File trên GitHub Release chưa ký, chỉ dùng để nộp Store. Đường exe (§1) chờ chữ
ký Authenticode (SignPath Foundation).

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

- **"Installer requires elevation"**: KHÔNG thêm `/CURRENTUSER` (với bản `-machine.exe`
  cờ này chuyển sang cài per-user, ARP về HKCU — lỗi B13/B19). Kiểm lại đã nộp đúng
  bản máy và manifest `asInvoker` (gate bước 0 của `validate-store-package.ps1`).
- **"Installer shows UI"**: kiểm tra lại đã dùng `/VERYSILENT` (không phải
  `/SILENT` — `/SILENT` vẫn hiện thanh tiến trình).
- **"App does not launch after install"**: đúng hành vi — silent không tự mở
  app; người dùng mở từ Start Menu. Không thêm auto-launch cho luồng Store.

## 5. Quy trình publish lên Microsoft Store — từng bước

1. **Tài khoản**: đăng ký [Partner Center](https://partner.microsoft.com/dashboard)
   (tài khoản cá nhân ~$19 hoặc công ty ~$99, thuế/ID xác minh một lần).
2. **Reserve app name**: Apps and Games → Overview → New product → Name. Tên đã
   reserve: `TextVN` (ARP `DisplayName` phải đúng chuỗi này). Dùng lại tên này cho
   sản phẩm MSIX thì phải xoá tên khỏi sản phẩm EXE trước — xem `msix-submission.md` §2.
3. **Chuẩn bị gói**:
   - Tải `TextVN-setup-<ver>-windows-x64.exe` từ GitHub Release (khuyên dùng
     bản có cờ CI xanh; `RELEASE_REPORT.json` bên trong ZIP portable cho biết
     commit + checksum).
   - Ghi lại SHA-256 từ `SHA256SUMS.txt`.
4. **Tạo submission**: Start submission →
   - **Product name / description**: chép từ `README.md` mục mô tả + ghi rõ
     điểm khác biệt (xem README "Điểm vượt trội so với các bộ gõ khác").
   - **Installer parameters** (quan trọng nhất):
     `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` (đúng §1a — KHÔNG `/CURRENTUSER`, KHÔNG `/ALLUSERS`)
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

## 6. Loại gói, Chứng thư số & Chính sách 10.2.9 (Cập nhật v0.2.24)

> **Cập nhật quan trọng 2026-10-06 (Sau kết quả thẩm định v0.2.24):**  
> Bản máy `TextVN-setup-0.2.24-windows-x64-machine.exe` đã **PASS 100% cả 3 bài test tự động** (Silent install, ARP entry, Bundleware).  
> Hệ thống chuyển sang vòng **Policy Review** và trả về thông báo **Chính sách 10.2.9 Security - Package Submissions**:  
> *"Package should be signed with SHA256 or higher algorithm"*.

### 6a. Bản chất Chính sách 10.2.9 đối với gói Win32 EXE

Đối với các ứng dụng nộp dưới dạng Win32 EXE/MSI qua URL tải về:
- Microsoft Store **KHÔNG** tự động ký số cho file EXE raw tải từ URL bên ngoài.
- File EXE nộp lên **BẮT BUỘC PHẢI ĐÃ ĐƯỢC KÝ SỐ** bằng chứng thư Authenticode hợp lệ thuộc *Microsoft Trusted Root Program*, sử dụng thuật toán băm SHA-256 trở lên.
- Nếu nộp file EXE chưa ký (`Unsigned`), Partner Center sẽ lập tức từ chối theo Policy 10.2.9.

### 6b. Hai hướng xử lý chính thức

```
                           +-----------------------------------------------+
                           |        THÔNG BÁO CHÍNH SÁCH STORE 10.2.9      |
                           +-----------------------------------------------+
                                    |                             |
                                    v                             v
             [ HƯỚNG 1: KÝ CHO EXE ]             [ HƯỚNG 2: CHUYỂN SANG MSIX ]
             - Đăng ký SignPath Foundation       - Microsoft Store KÝ SỐ MIỄN PHÍ
             - Hoặc dùng Azure Trusted Signing   - Host miễn phí trên CDN Store
             - Ký SHA-256 rồi nộp lại EXE URL    - Giải phóng tên Win32 cũ -> Tạo MSIX
```

#### Hướng 1: Ký số cho file Win32 EXE (Giữ luồng EXE hiện tại)
1. **SignPath Foundation** (Miễn phí cho Open Source):
   - Duyệt đơn đăng ký tại [signpath.org/open-source](https://signpath.org/open-source).
   - Thêm secrets vào GitHub repo: `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` (tên cũ `SIGNPATH_PROJECT_KEY` vẫn nhận), `SIGNPATH_POLICY`, `SIGNPATH_ARTIFACT_CONFIGURATION` (cấu hình ZIP, deep sign `*.exe`/`*.dll`).
   - *(cập nhật 2026-10-09)* Khi có secret, `build-release.ps1` gửi **một** yêu cầu ký (ZIP deep sign) cho 4 PE (`TextVN.exe`, `textvn-cli.exe`, `textvn-tsf.dll`, `textvn-tsf-x86.dll`) rồi ký từng bộ cài: `TextVN-setup-<ver>-windows-x64.exe` và `-machine.exe` (file nộp Store — `release.yml` ký ở bước "Build + validate machine installer"). `tools/win/sign-signpath.ps1` dùng REST API công bố của SignPath, chờ cả bước duyệt tay. `unins000.exe` vẫn chưa ký (giới hạn đã biết).
   - Nộp lại URL file EXE đã ký lên Partner Center.

#### Hướng 2 (Khuyến nghị hàng đầu): Chuyển sang định dạng MSIX Full-Trust
Đây là phương án được chính reviewer của Microsoft Store khuyến nghị:
> *"Microsoft Store offers many complimentary benefits for MSIX format such as code signing, hosting etc."*

- **Ưu điểm lớn nhất:** Gói `.msix` nộp lên ở trạng thái **UNSIGNED**, chính hệ thống Microsoft Store sẽ tự động ký số bằng chứng thư gốc của Microsoft khi phát hành. Không cần mua cert, không cần chờ SignPath duyệt.
- **Lưu ý then chốt về App Name:**
  > *"Note that you have to delete your app name from existing Win32 app in Partner Center in case you want to use the same for MSIX packaged app."*
  - **Cách A (Dùng đúng tên `TextVN`):**
    1. Vào sản phẩm `TextVN` Win32 hiện tại trong Partner Center $\rightarrow$ Product management $\rightarrow$ Product identity $\rightarrow$ Bấm **Delete product** để giải phóng tên `TextVN`.
    2. Bấm **New product** $\rightarrow$ Chọn loại **MSIX or PWA application** $\rightarrow$ Đặt tên `TextVN`.
    3. Vào trang Product identity lấy `Package/Identity/Name` và `Package/Identity/Publisher`.
    4. Chạy `tools/win/build-msix.ps1` để tạo file `.msix`.
    5. Tải file `.msix` trực tiếp lên submission mới $\rightarrow$ Store tự động ký và phát hành.
  - **Cách B (Không cần xóa sản phẩm cũ):**
    1. Tạo sản phẩm mới dạng **MSIX or PWA application** với tên hiển thị bổ sung như `TextVN - Bộ gõ tiếng Việt`.
    2. Đóng gói MSIX (repo variable `MSIX_DISPLAY_NAME` = đúng tên đó) và upload trực tiếp.
  - Chọn A hay B là quyết định của chủ tài khoản; các bước chi tiết và cách build hiện hành: [`msix-submission.md`](msix-submission.md) §2–§3.

### 6c. Kiểm tra chữ ký Authenticode cục bộ
- `Get-AuthenticodeSignature <path-to-exe>` → Kiểm tra `Status = Valid` và thuật toán băm SHA256.

