<!-- TASK: ARM-W7 · agent: antigravity-gemini-37c7c6bc -->
## VI
- **ARM-W7 — asset phát hành `windows-arm64` (ADR-015)**: job mới `windows-arm64`
  trong `release.yml` (runner `windows-11-arm` thật, assert `PROCESSOR_ARCHITECTURE`
  = ARM64) — cài native host `aarch64-pc-windows-msvc` + target `x86_64`/`i686` +
  Inno Setup 6 vào `GITHUB_PATH` giống hệt `ci-windows.yml`, chạy
  `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller`, upload artifact
  `release-windows-arm64` (portable + setup arm64). `publish.needs` thêm `windows-arm64`;
  assertions asset: portable == 2 (x64 == 1, arm64 == 1), setup không-machine == 2
  (x64 == 1, arm64 == 1), machine == 1; đồng thời vòng lặp kiểm `RELEASE_REPORT.json`
  và không chứa `textvn-hook.exe` chạy trên **cả hai** gói portable (x64 và arm64).
  `windows-2025-e2e` thu hẹp glob portable thành `*-windows-x64-*.zip`. Chữ ký
  GPG/cosign tự phủ asset mới vì `publish` ký mọi tệp trong `release-assets/`.
- **Bằng chứng CI xanh**:
  - `release-candidate` workflow_dispatch **46128960595** (`140929711097` `Windows arm64 (setup + portable)`): **PASS trong 8m37s**, đã tạo & tải artifact `release-windows-arm64` (từ commit `085ebc47` / `cadb689b`).
  - `ci-windows` **46128960881** (gồm fix RunOnce `test-portable.ps1` `0bd7d226`): **4/4 job PASS** (`windows-11-arm`, `windows-2022`, `windows-2025`, `clippy`).
- Việc release-manager gộp khi phát hành: cập nhật số asset ("34 asset") ở
  `docs/release/release-process.md` + README (B4/B7).
## EN
- **ARM-W7 — `windows-arm64` release assets (ADR-015)**: new `windows-arm64` job in
  `release.yml` (real `windows-11-arm` runner, asserts `PROCESSOR_ARCHITECTURE`
  = ARM64) — installs native `aarch64-pc-windows-msvc` Rust host + `x86_64`/`i686`
  targets + Inno Setup 6 into `GITHUB_PATH` matching `ci-windows.yml`, runs
  `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller`, and uploads
  `release-windows-arm64` (arm64 portable + setup). `publish.needs` gains `windows-arm64`;
  asset assertions: portable == 2 (x64 == 1, arm64 == 1), non-machine setups == 2
  (x64 == 1, arm64 == 1), machine == 1; `RELEASE_REPORT.json` and no-`textvn-hook.exe`
  checks loop over **both** portable zips (x64 and arm64). `windows-2025-e2e` narrows
  its portable glob to `*-windows-x64-*.zip`. GPG/cosign signing automatically covers
  the new assets since `publish` signs everything in `release-assets/`.
- **Green CI evidence**:
  - `release-candidate` workflow_dispatch **46128960595** (`140929711097` `Windows arm64 (setup + portable)`): **PASS in 8m37s**, built & uploaded artifact `release-windows-arm64` (from commit `085ebc47` / `cadb689b`).
  - `ci-windows` **46128960881** (including RunOnce `test-portable.ps1` fix `0bd7d226`): **4/4 jobs PASS** (`windows-11-arm`, `windows-2022`, `windows-2025`, `clippy`).
- Release-manager follow-up: update the asset count ("34 assets") in
  `docs/release/release-process.md` + README during the release (B4/B7).
