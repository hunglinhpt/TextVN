<!-- TASK: ARM-W7 · agent: antigravity-gemini-37c7c6bc -->
## VI
- **ARM-W7 — asset phát hành `windows-arm64` (ADR-015)**: job mới `windows-arm64`
  trong `release.yml` (runner `windows-11-arm` thật, assert `PROCESSOR_ARCHITECTURE`
  = ARM64) — `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller` như job
  ARM64 của ci-windows; upload artifact `release-windows-arm64`
  (portable + setup arm64). `publish.needs` thêm `windows-arm64`; assertions asset:
  portable == 2 (x64 + arm64, đếm riêng arm64 == 1), setup không-machine == 2 +
  `*-windows-arm64.exe` == 1, machine == 1. Chữ ký GPG/cosign tự phủ asset mới vì
  publish ký mọi tệp trong `release-assets/` (SHA256SUMS + `.asc` + `.cosign.*`).
- Việc release-manager gộp khi phát hành: cập nhật số asset ("34 asset") ở
  `docs/release/release-process.md` + README (B4/B7).
## EN
- **ARM-W7 — `windows-arm64` release assets (ADR-015)**: new `windows-arm64` job in
  `release.yml` (real `windows-11-arm` runner, asserts `PROCESSOR_ARCHITECTURE`
  = ARM64) running `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller` like
  ci-windows' ARM64 job; uploads the `release-windows-arm64` artifact
  (arm64 portable + setup). `publish.needs` gains `windows-arm64`; asset
  assertions: portable == 2 (x64 + arm64, arm64 counted separately == 1),
  non-machine setups == 2 + `*-windows-arm64.exe` == 1, machine == 1. GPG/cosign
  signing automatically covers the new assets since publish signs everything in
  `release-assets/` (SHA256SUMS + `.asc` + `.cosign.*`).
- Release-manager follow-up: update the asset count ("34 assets") in
  `docs/release/release-process.md` + README during the release (B4/B7).
