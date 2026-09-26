# env.md — Môi trường build Windows (WIN-001)

> Task: `P1-6-TASKS.md` → **WIN-001**. Cập nhật khi đổi máy/upgrade tool.
> Ghi nhận: 2026-09-27 (agent Windows).

## 1. Build host (máy dev)

| Thành phần | Phiên bản | Trạng thái |
|---|---|---|
| OS | Windows 11 (SDK 10.0.26100.0) | ✓ ≥ 10.0.22621 theo yêu cầu |
| rustc | 1.98.1 (48a229cea 2026-09-01) | ✓ |
| rustup | 1.29.1 (d95a37b6a 2026-08-13) | ✓ |
| Target | `x86_64-pc-windows-msvc` (installed) | ✓ |
| Rust components | rustfmt, clippy | ✓ |
| VS Build Tools | 2026 (18) @ `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` | ✓ |
| MSVC | 14.51.36231 (`cl.exe` Hostx64) | ✓ |
| Windows SDK | 10.0.26100.0 | ✓ |
| Inno Setup 6 | **chưa cài** | ⬜ cần trước task đóng gói installer (M5) |
| Python | 3.12.10 | ✓ (reuse, script test) |

## 2. Tools DoD (Handbook §4 mục 4)

| Tool | Phiên bản | Trạng thái |
|---|---|---|
| `cargo-deny` | 0.20.2 | ✓ đã cài + `deny.toml` (repo root) → `cargo deny check` = **advisories/bans/licenses/sources ok** |
| `reuse` | 6.2.0 (pip) | ✓ đã cài + `.reuse/dep5` + `LICENSES/GPL-3.0-or-later.txt` → `reuse lint` = **compliant 84/84** |
| `cargo fmt` | rustfmt component | ✓ `cargo fmt --all --check` = 0 diff |

## 3. Bằng chứng acceptance (WIN-001)

```text
> cargo build --workspace --target x86_64-pc-windows-msvc
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.80s   → exit 0

> cargo test --workspace            → 58/58 pass (ffi size test 20/532 ✓)
> cargo clippy --workspace --all-targets   → 0 warning
> cargo fmt --all --check           → exit 0
> cargo deny check                  → advisories ok, bans ok, licenses ok, sources ok (exit 0)
> reuse lint                        → Congratulations! Compliant REUSE 3.3 (84/84) (exit 0)
```

## 4. Gap & việc cần khi lên milestone liên quan

1. **Inno Setup 6 chưa cài** → cài trước task đóng gói installer (M5, `ISCC.exe` tại `C:\Program Files (x86)\Inno Setup 6\`).
2. **CI 3 OS chưa có** (`ci-*.yml` — task `WIN-064`): DoD mục 7 hiện ghi "OS liên quan xanh, 2 OS còn lại chưa đổi".
3. `rust-toolchain.toml` để `channel = "stable"` (chưa pin số — TODO sẵn trong file, sau khi mọi dev machine thống nhất).
4. Máy sạch khác: theo `P0-1 §4` → `rustup target add x86_64-pc-windows-msvc` + VS Build Tools (workload "C++ build tools") + Python `pip install reuse` + `cargo install cargo-deny`.
