# ARM-W6 — `.msixbundle` x64 + arm64 (claude-opus-7e41b2d9) — phần 1 + 2

## VI

- **`tools/win/build-msix.ps1 -Arch x64|arm64`**: gói arm64 lấy payload của `build-release.ps1 -Arch arm64`
  (exe ARM64 + forwarder ARM64X `textvn-tsf.dll` + `textvn-tsf-arm64.dll` + `textvn-tsf-x64.dll` + `textvn-tsf-x86.dll`),
  `ProcessorArchitecture` lấy từ `-Arch` (template `{{ARCH}}`), tên `TextVN-<ver>-windows-<arch>.msix`. Mặc định x64 —
  lệnh cũ không đổi kết quả.
- **`-Bundle`**: gộp hai gói đã build thành `TextVN-<ver>-windows.msixbundle` (`makeappx bundle /bv <identity version>`).
  makeappx tự chọn bản `arm64` trên máy ARM64.
- **`cargo xtask verify-msix`**: kiến trúc khai báo phải khớp PE Machine của `TextVN.exe`; gói arm64 bắt buộc đủ DLL
  đích; tệp `.msixbundle` ⇒ kiểm manifest bundle, đúng hai gói x64 + arm64, từng gói đạt toàn bộ luật, identity khớp.
- **Kiểm thử thật (máy dev, makeappx 10.0.26100)**: build gói x64 + một gói "arm64" GIẢ từ exe x64 rồi `-Bundle` ⇒
  `verify-msix` bắt đúng `TextVN.exe Machine 0x8664 không khớp ProcessorArchitecture arm64`, đọc được
  `AppxBundleManifest.xml` thật của makeappx.

- **Phần 2 (CI phát hành)**: `release.yml` job `windows-arm64` build + `verify-msix` gói `TextVN-<ver>-windows-arm64.msix`
  trên runner ARM64 thật (PowerShell, kiểm Machine 0xAA64 + đủ DLL đích); job mới `windows-msixbundle` (windows-2022,
  cần `windows` + `windows-arm64`) gộp `-Bundle` + `verify-msix` bundle → artifact `release-windows-msixbundle`;
  `publish` cần job này và bắt buộc đúng 2 `.msix` (x64, arm64) + 1 `.msixbundle`. `store-publish.yml` nộp
  `.msixbundle` khi release có, release cũ (trước ARM-W6) nộp `.msix` x64 kèm `::warning::`;
  `verify-store-package.sh` nhận `PKG_NAME` (chỉ 2 tên hợp lệ của đúng version).
- **Chưa làm (nói thật)**: sideload thử gói arm64 trên runner ARM — gói chưa ký (Store ký khi phát hành), `Add-AppxPackage`
  không nhận gói chưa ký nếu không bật Developer Mode + chứng thư tự ký; để lại cho lần nộp Store thật (chủ repo).
- **Release-manager gộp khi phát hành**: số asset B7 ("34 asset" ở `release-process.md`, `store-submission.md` S1,
  AGENTS.md) — 0.2.97 có 13 artifact (thêm linux-aarch64 tar.gz, portable + setup arm64, msix arm64, msixbundle)
  ⇒ 13 × 4 + `SHA256SUMS.txt` + `gpg-release-key.asc` = **54 asset**.

## EN

- **`tools/win/build-msix.ps1 -Arch x64|arm64`**: the arm64 package takes the `build-release.ps1 -Arch arm64` payload
  (ARM64 exe + ARM64X forwarder `textvn-tsf.dll` + `textvn-tsf-arm64.dll` + `textvn-tsf-x64.dll` + `textvn-tsf-x86.dll`),
  `ProcessorArchitecture` comes from `-Arch` (template `{{ARCH}}`), file name `TextVN-<ver>-windows-<arch>.msix`.
  Default x64 — existing commands produce the same result.
- **`-Bundle`**: merges the two built packages into `TextVN-<ver>-windows.msixbundle` (`makeappx bundle /bv <identity
  version>`). Windows picks the `arm64` package on ARM64 devices.
- **`cargo xtask verify-msix`**: the declared architecture must match the PE Machine of `TextVN.exe`; arm64 packages must
  carry every target DLL; a `.msixbundle` ⇒ bundle manifest checked, exactly two packages x64 + arm64, each passing every
  rule, identities matching.
- **Real test (dev machine, makeappx 10.0.26100)**: built an x64 package + a FAKE "arm64" package from x64 exes, then
  `-Bundle` ⇒ `verify-msix` correctly reports `TextVN.exe Machine 0x8664 không khớp ProcessorArchitecture arm64`, and
  reads makeappx's real `AppxBundleManifest.xml`.
- **Part 2 (release CI)**: `release.yml` job `windows-arm64` builds + `verify-msix` the package
  `TextVN-<ver>-windows-arm64.msix` on a real ARM64 runner (PowerShell, checks Machine 0xAA64 + every target DLL); new job
  `windows-msixbundle` (windows-2022, needs `windows` + `windows-arm64`) runs `-Bundle` + `verify-msix` on the bundle →
  artifact `release-windows-msixbundle`; `publish` needs it and requires exactly 2 `.msix` (x64, arm64) + 1 `.msixbundle`.
  `store-publish.yml` submits the `.msixbundle` when the release has one; older releases (before ARM-W6) submit the x64
  `.msix` with a `::warning::`; `verify-store-package.sh` takes `PKG_NAME` (only the 2 valid names for that version).
- **Not done (honestly)**: sideload test of the arm64 package on the ARM runner — the package is unsigned (the Store signs
  it on publish), and `Add-AppxPackage` refuses unsigned packages without Developer Mode + a self-signed cert; left for the
  real Store submission (repo owner).
- **For the release manager at cut time**: the B7 asset count ("34 assets" in `release-process.md`, `store-submission.md`
  S1, AGENTS.md) — 0.2.97 has 13 artifacts (adds linux-aarch64 tar.gz, arm64 portable + setup, arm64 msix, msixbundle)
  ⇒ 13 × 4 + `SHA256SUMS.txt` + `gpg-release-key.asc` = **54 assets**.

## Bằng chứng / Evidence
- Dry-run B4b `release-candidate` 46130615662 (9e772256): job `Windows arm64 (setup + portable)` và
  `Windows .msixbundle (x64 + arm64)` XANH — makeappx gộp đúng 2 gói `TextVN-0.2.96-windows-{x64,arm64}.msix`,
  `verify-msix` bundle đạt luật Store với identity `MSIX_IDENTITY_NAME`. Tải 3 artifact Windows về mô phỏng
  assert của `publish` (chỉ chạy khi push tag): `*.msix`=2, x64=1, arm64=1, `.msixbundle`=1, portable=2 ⇒ đạt;
  `verify-msix --require-store-identity --expect-name 23651Linhi.TextVN` cục bộ trên bundle + gói arm64 ⇒ OK;
  `verify-store-package.sh` với `PKG_NAME` lạ ⇒ FATAL (chặn đúng).
- Dry run B4b `release-candidate` 46130615662 (9e772256): jobs `Windows arm64 (setup + portable)` and
  `Windows .msixbundle (x64 + arm64)` GREEN — makeappx bundled exactly the 2 packages
  `TextVN-0.2.96-windows-{x64,arm64}.msix`, bundle `verify-msix` passes Store rules with the `MSIX_IDENTITY_NAME`
  identity. Downloaded the 3 Windows artifacts to simulate the `publish` assertions (they only run on tag push):
  `*.msix`=2, x64=1, arm64=1, `.msixbundle`=1, portable=2 ⇒ pass; local `verify-msix --require-store-identity
  --expect-name 23651Linhi.TextVN` on the bundle + arm64 package ⇒ OK; `verify-store-package.sh` with an unknown
  `PKG_NAME` ⇒ FATAL (correctly rejected).

## Tài liệu / Docs

`docs/release/msix-submission.md` (mục `.msixbundle` + CI phần 2, hàng `verify` của store-publish).

## Kiểm thử / Testing

`cargo test -p xtask` 110/110 — mới `arm_w6_msixbundle_x64_arm64`; `verify_msix_bat_loi_tung_luat` mở rộng (arm64 hợp lệ,
lệch Machine, thiếu DLL đích, kiến trúc lạ).
