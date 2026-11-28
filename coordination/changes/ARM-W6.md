# ARM-W6 — `.msixbundle` x64 + arm64 (claude-opus-7e41b2d9) — phần 1/2

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
- **Phần 2 (sau khi ARM-W7 nhả `release.yml`)**: job `release.yml` build gói arm64 trên `windows-11-arm`, job gộp bundle,
  `store-publish.yml` nộp `.msixbundle`, sideload thử trên runner ARM.

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
- **Part 2 (after ARM-W7 releases `release.yml`)**: `release.yml` builds the arm64 package on `windows-11-arm`, a bundle
  job merges them, `store-publish.yml` submits the `.msixbundle`, sideload test on the ARM runner.

## Tài liệu / Docs

`docs/release/msix-submission.md` (mục `.msixbundle`).

## Kiểm thử / Testing

`cargo test -p xtask` 110/110 — mới `arm_w6_msixbundle_x64_arm64`; `verify_msix_bat_loi_tung_luat` mở rộng (arm64 hợp lệ,
lệch Machine, thiếu DLL đích, kiến trúc lạ).
