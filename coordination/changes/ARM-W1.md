# ARM-W1 — Build + test native Windows ARM64 (claude-opus-c4c5a567)

## VI

- **CI Windows ARM64 thật** (`ci-windows.yml` job `windows-arm64`, runner `windows-11-arm` — Snapdragon X / Surface
  Pro X / Windows Dev Kit): rustup với host **native** `aarch64-pc-windows-msvc` (kiểm bắt buộc — toolchain x64 giả lập
  sẽ build ra x64 âm thầm), `cargo test --workspace` (không hook — quyết định chủ repo: TSF thuần trên ARM), `verify` +
  `sizes` + replay corpus TSF, build release TIP/tray/CLI ARM64.
- **Kết quả lần chạy đầu** (run [46097743527](https://github.com/hunglinhpt/TextVN/actions/runs/46097743527)):
  **2327 test pass / 0 fail** (82 binary, gồm e2e AppContainer thật `sandbox_appcontainer_end_to_end`);
  `textvn_win_tsf.dll`, `TextVN.exe`, `textvn-cli.exe` đều **Machine 0xAA64**, không VC++ runtime.
- `.cargo/config.toml`: `+crt-static` cho `aarch64-pc-windows-msvc` (cùng luật R2-18 với x64/x86).
- `cargo xtask check-pe-imports --machine x64|x86|arm64` — kiểm trường Machine của COFF header (đóng gói nhiều kiến
  trúc không lẫn DLL sai kiến trúc).
- Đóng gói (forwarder ARM64X, bộ cài, MSIX bundle) là ARM-W2…W6.

## EN

- **Real Windows ARM64 CI** (`ci-windows.yml` job `windows-arm64`, runner `windows-11-arm` — Snapdragon X / Surface
  Pro X / Windows Dev Kit): rustup with a **native** `aarch64-pc-windows-msvc` host (asserted — an emulated x64 toolchain
  would silently build x64), `cargo test --workspace` (no hook — owner decision: pure TSF on ARM), `verify` + `sizes` +
  TSF corpus replay, release build of the ARM64 TIP/tray/CLI.
- **First run** (run [46097743527](https://github.com/hunglinhpt/TextVN/actions/runs/46097743527)):
  **2327 tests pass / 0 fail** (82 binaries, including the real AppContainer e2e `sandbox_appcontainer_end_to_end`);
  `textvn_win_tsf.dll`, `TextVN.exe`, `textvn-cli.exe` are all **Machine 0xAA64** with no VC++ runtime.
- `.cargo/config.toml`: `+crt-static` for `aarch64-pc-windows-msvc` (same R2-18 rule as x64/x86).
- `cargo xtask check-pe-imports --machine x64|x86|arm64` — checks the COFF header Machine field (multi-arch packaging
  cannot mix in a DLL of the wrong architecture).
- Packaging (ARM64X forwarder, installer, MSIX bundle) is ARM-W2…W6.

## Kiểm thử / Testing

`cargo test -p xtask` (`arm_w1_pe_machine_doc_dung_kien_truc`); local `check-pe-imports --machine arm64` on an x64 binary ⇒
FAIL as expected; CI run above.
