# P0-1 — Repo layout, crate responsibilities, Build/Test/CI workflow

> Phần 0 · Tài liệu nền tảng chung. Đọc cùng `P0-2-ffi` và `P0-3-config-preset-strategy`.

## 1. Workspace layout (nguồn sự thật — mọi agent phải tôn trọng)

```
vietime/                                   # monorepo, Rust workspace + 1 số adapter không phải Rust
├── Cargo.toml                             # [workspace] members = core, adapters/*(rust), tray, cli...
├── rust-toolchain.toml                    # stable, pinned
├── deny.toml                              # cargo-deny: licenses, bans, sources
├── REUSE.toml                             # SPDX coverage
├── core/
│   ├── Cargo.toml                         # crate: vietime-core
│   └── src/
│       ├── lib.rs                         # pub API nội bộ (Rust-level); KHÔNG public ABI
│       ├── keymap.rs                      # normalize: VK/scan/char + modifier → KeyEvent
│       ├── buffer.rs                      # vòng quặp từ ngữ, word boundary, context stack
│       ├── validate.rs                    # 5 quy tắc âm tiết (kế thừa gonhanh §validation)
│       ├── transform/
│       │   ├── mod.rs
│       │   ├── vowel_table.rs             # bảng 72 entry (data gen từ tables/)
│       │   ├── tone.rs                    # sắc/huyền/hỏi/ngã/nặng + mark reposition
│       │   ├── diacritic_style.rs         # new(hoà)/old(hòa)
│       │   ├── stroke.rs                  # đ/Đ
│       │   └── undo.rs                    # ass→as, ww→w
│       ├── method/{telex,vni,viqr,simple_telex}.rs   # bảng phím → action (data-driven)
│       ├── post/{restore_en,caps,macro,emoji}.rs
│       └── suggest.rs                     # (Should) offline suggestion — tách feature flag
├── ffi/                                   # crate: vietime-ffi → staticlib "vietime_ffi"
│   ├── Cargo.toml                         # crate-type = ["staticlib","cdylib"] (cdylib chỉ để fuzz)
│   ├── src/lib.rs                         # C-ABI duy nhất, xem P0-2
│   └── include/vietime_ffi.h              # header C, generate bằng cbindgen + review tay
├── strategy/                              # crate: vietime-strategy (không depend OS)
│   └── src/{resolve.rs, action.rs, rules_field.rs}
├── config/                                # crate: vietime-config: load/validate/migrate config.v1.json
├── appdb/                                 # crate: vietime-appdb: parse + verify chữ ký preset
├── ipc/                                   # crate: vietime-ipc: message enum + JSON codec + pipe auth
├── field-detect/                          # crate: vietime-field-detect: FieldContext → preset/role
│   │                                      #   (dùng CHUNG cho windows-tsf + windows-hook — 1 nguồn sự thật)
├── adapters/
│   ├── windows-tsf/                       # crate: vietime-win-tsf → cdylib "vietime-tsf.dll"
│   │   ├── build.rs                       # link ole32/ole32uuid/advapi32
│   │   └── src/{lib.rs, class.rs, tip.rs, thread.rs, edit_session.rs, key_event.rs,
│   │            composition.rs, display_attr.rs, register.rs, ipc_client.rs}
│   ├── windows-hook/                      # crate: vietime-win-hook → bin "vietime-hook.exe"
│   │   └── src/{main.rs, hook.rs, inject.rs, focus.rs, guard.rs}
│   │       # PROCESS RIÊNG (crash-isolated): tray spawn + watchdog restart <500ms.
│   │       # Tuyệt đối KHÔNG nhúng hook vào tray (UI lag = gõ lag).
│   ├── macos-imk/                         # Phần 2 (Swift package, không nằm Rust workspace)
│   ├── macos-tap/                         # Phần 2 (Swift, CGEventTap opt-in — xem P2-2)
│   ├── linux-ibus/                        # Phần 3 (C) — engine IBus
│   ├── linux-fcitx5/                      # Phần 3 (C++) — addon fcitx5
│   ├── linux-common/                      # Phần 3 (C) — AT-SPI field detect + ipc/log helpers
│   └── linux-x11/                         # Phần 3 (Rust: crate vietime-x11) — fallback opt-in (P3-3)
├── tray/                                  # crate: vietime-tray → bin "vietime-tray.exe" (egui)
│   └── src/{main.rs, tray_icon.rs, ui/{settings.rs, apps.rs, macros.rs, about.rs}, svc.rs}
├── updater/                               # crate: vietime-updater (lib) — verify Ed25519 + spawn setup
├── cli/                                   # crate: vietime-cli → bin "vietime.exe"
│   └── src/{main.rs, doctor.rs, replay.rs, verify.rs, register.rs, config.rs, ipc.rs, tray.rs}
├── data/                                  # KHÔNG build-time embed toàn bộ (trừ bảng transform)
│   ├── tables/{telex,vni,viqr,simple_telex}.toml   # nguồn → gen bằng xtask
│   ├── appdb.default.json                 # preset mặc định (có chữ ký trong release)
│   ├── preset.pub                         # pubkey verify preset update
│   └── spelling/, emoji.tsv, stop_en.txt, games_blocklist.txt   # blocklist game (P1-2 §8 / P2-2 §7)
├── docs/                                  # tài liệu dự án: 00-INDEX, handbook, PLAN, adr/, specs/,
│                                          #   security/, release/, compliance/ + 10-shared/20-windows/
│                                          #   30-macos/40-linux (P0/P1/P2/P3)
├── perf/                                  # baseline-{win,mac,linux}.json — ngưỡng P1-5 §5 / P2-5 §5
├── schemas/
│   ├── ffi.v1.md                          # copy có chú giải của include/vietime_ffi.h
│   ├── config.v1.schema.json
│   ├── appdb.v1.schema.json
│   └── ipc.v1.md
├── corpus/{shared,win,mac,linux}/         # golden keys → expect
├── tests/conformance/                     # chạy replay trên nhiều "adapter mode"
├── fuzz/{ffi_key,config_parse,appdb_parse}/
├── packaging/windows/{vietime.iss, sign.ps1}
├── packaging/homebrew/vietime.rb          # cask — Phần 2 (P2-4 §8)
├── packaging/linux/{vietime.spec, ci-install.sh, uninstall-check.sh, aur/}  # Phần 3 (P3-5)
├── tools/mac/                             # ax-driver (Swift), soak.sh, mem-check.sh, uninstall-check.sh — P2-5
├── tools/linux/                           # atspi-driver, smoke-ibus.sh, soak.sh, mem-check.sh — P3-6
├── tools/appcomptest/                     # crate: vietime-appcomptest — UIA driver (Rust) — xem P1-5
├── tools/bench/                           # crate: vietime-bench — perf microbench (P1-5 §5)
├── tools/win/                             # script PowerShell: smoke-tsf.ps1, soak.ps1, mem-check.ps1
└── .github/workflows/{ci-shared,ci-windows,ci-macos,ci-linux,release}.yml
```

**Quy tắc ownership:** mỗi thư mục trên = 1 owner trong `CODEOWNERS`. Agent nhận task chỉ được
đụng file trong phạm vi task; sửa file ngoài phạm vi → ghi rõ lý do trong PR.

## 2. Trách nhiệm từng crate & ranh giới

| Crate | Responsibility | Được phép phụ thuộc (dependency) | TUYỆT ĐỐI KHÔNG có |
|---|---|---|---|
| `vietime-core` | Biến đổi KeyEvent → OutputAction | std (feature `std`), không khác | I/O, network, thread, log text |
| `vietime-ffi` | Bọc core thành C-ABI ổn định | `vietime-core` | alloc qua FFI, global mutable state ngoài instance |
| `vietime-strategy` | Chọn strategy từ context + preset | `vietime-appdb` (read-only types) | gọi OS API (adapter cung cấp `FieldContext`) |
| `vietime-config` | Parse/validate/migrate config | serde, jsonschema | network, tự tải preset |
| `vietime-appdb` | Parse preset, verify Ed25519 | ed25519-dalek | tự tải file (updater làm việc đó) |
| `vietime-ipc` | Codec message + policy auth pipe | serde, tokio(bản tray) / windows(rảnh TSF) | TCP, listener ngoài pipe đặt tên |
| `vietime-win-tsf` | COM TIP, composition, key sink | ffi, strategy, config, ipc, windows | tải DLL ngoài hệ thống, network |
| `vietime-win-hook` | WH_KEYBOARD_LL + inject | ffi, strategy, config, windows | block > 2ms trong callback |
| `vietime-tray` | Tray + settings(egui) + IPC server + spawn hook + updater | mọi crate config/appdb | chạy trong process app khác |
| `vietime-field-detect` | `FieldContext` → app_id/role/strategy lookup (chuẩn hoá exe, UIA rules Win) | appdb, config (read-only) | OS API (adapter cung cấp element/cache) |
| `vietime-cli` | `doctor`(`--export/--stats`) / `replay` / `verify` / `register` / `uninstall` / `purge` / `config init+validate` / `ipc probe` / `tray --stop` / `sizes` | ffi, config, appdb | — |

*(dependency = "phụ thuộc"; giữ nguyên thuật ngữ `dependency` trong code).*

## 3. Lệnh chuẩn (mọi agent dùng đúng, không bịa)

```bash
# --- build ---
cargo build --workspace --target x86_64-pc-windows-msvc      # Win
cargo build --release -p vietime-win-tsf                     # ra vietime-tsf.dll (static CRT)
cargo xtask gen-tables                                       # gen bảng transform từ data/tables/*.toml
cargo xtask cbindgen                                         # regenerate ffi/include/vietime_ffi.h (rồi review tay)

# --- test ---
cargo test --workspace                                        # unit + integration
cargo run -p vietime-cli -- replay corpus/shared --adapter headless   # golden corpus
cargo run -p vietime-cli -- sizes   # in + verify size struct FFI (20/532) — exit 1 nếu lệch (P0-2 §6)
cargo run -p vietime-cli -- replay corpus/mac --adapter mac           # corpus macOS (P2-5)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo deny check
reuse lint                                                    # SPDX

# --- fuzz (nightly / PR chạm parser) ---
cargo fuzz run ffi_key -- -max_total_time=60
cargo fuzz run config_parse -- -max_total_time=60

# --- Windows-only ---
cargo run -p vietime-cli -- register --install-dir "C:\Program Files\VietIME"   # đăng ký TIP (per-user)
cargo run -p vietime-cli -- doctor                                              # chẩn đoán env
cargo test -p vietime-win-tsf -- --ignored                                       # test cần desktop session

# --- macOS-only (Phần 2) ---
./adapters/macos-imk/build-rust.sh                          # staticlib universal (aarch64 + x86_64)
cd adapters/macos-imk && swift build && swift test          # IMK bundle (corpus mac: xem mục test trên)
```

> **Lưu ý môi trường dev:** target `x86_64-pc-windows-msvc` (không dùng gnu) vì TSF/COM link thẳng SDK.
> `cargo xtask` là duy nhất nơi gen code — **không edit tay** file đã generate (danh sách generate:
> `ffi/include/vietime_ffi.h`, `core/src/transform/vowel_table.rs`).

## 4. CI (GitHub Actions)

| Workflow | Trigger | Nội dung | Exit gate |
|---|---|---|---|
| `ci-shared.yml` | push/PR | fmt, clippy, deny, reuse, test core+ffi+strategy, fuzz smoke 60s, replay corpus/shared trên Ubuntu+Windows+macOS | 100% pass |
| `ci-windows.yml` | PR chạm `adapters/windows*`, `tray/`, `corpus/win/` | build msvc, unit, `replay corpus/win`, **UIA smoke** (Notepad + Chrome address bar) trên `windows-latest` | 100% pass |
| `ci-macos.yml` / `ci-linux.yml` | như trên | build + test phần tương ứng | 100% pass |
| `nightly.yml` | 03:00 | fuzz 10 phút/target, soak 1h, full 40-app matrix (tự động subset 12 app) | báo cáo artifact |
| `release.yml` | tag `v*` | build 3 OS × arch → sign → SBOM → GitHub Release → winget/brew/deb submit | chữ ký verify bằng `vietime verify` |

**Matrix tối thiểu cho `release.yml` (Windows):** `x86_64-msvc`, `i686-msvc`, `aarch64-msvc`.

## 5. Versioning & branching

- SemVer: `MAJOR.MINOR.PATCH`. **FFI có version riêng** (`IME_ABI_V1`) — bump ABI = MAJOR riêng của ABI (xem P0-2 §6).
- Preset/appdb có `appdb_version` + `min_engine_version` (preset mới không được phá engine cũ).
- `main` luôn releasable; feature branch sống < 5 ngày; hotfix → `hotfix/x.y.z` → backport.
- Release channel: `stable` (mặc định), `beta` (opt-in trong tray). Updater đọc channel từ config.

## 6. Tham chiếu ngược

- Danh sách bug kinh niên B1–B13 cần fix: `../../PLAN.md §1.3`.
- Task Windows cụ thể: `../20-windows/P1-6-TASKS.md`.
- Quy trình review: `../00-INDEX.md §3`.
