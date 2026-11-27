# P0-7 — TASKS: `textvn-session` + ARM (WBS cho nhiều agent)

> Đọc theo thứ tự: `P0-5-ARM-PLATFORMS-PLAN.md` (vì sao) → `P0-6-SESSION-FRAMEWORK-SPEC.md` (thiết kế)
> → tệp này (việc). Giao nhận việc **bắt buộc** qua `cargo xtask board` (G21, luật R1–R14 ở
> `coordination/README.md`). Trạng thái ĐỘNG (ai giữ, đang làm, xong) nằm trong
> `coordination/board.jsonl`; tệp này chỉ giữ định nghĩa task + ghi chú handoff.
>
> Ước lượng: S ≤ 0,5 ngày · M = 1–2 ngày · L = 3–5 ngày. `Dep` = task phải `done` trên board trước.

## 1. Vai trò

| Vai trò | Ai | Việc |
|---|---|---|
| **Agent task** | bất kỳ agent nhận task qua `board claim` | làm đúng phạm vi task, viết test, mảnh ghi chú `coordination/changes/<TASK>.md`, tự audit, gửi review |
| **Agent review** | agent KHÁC agent task (bắt buộc khác người viết) | chạy audit §4 độc lập, ghi kết quả `board review-done --task <TASK> --msg "PASS/FAIL: …"` |
| **Release manager** | agent giữ `board rm-lock` | gộp mảnh ghi chú, bump version, CHANGELOG/README/report, tag, B7–B9 (G16/G20); một người tại một thời điểm |
| **Chủ repo** | Bui Hung Linh | chốt quyết định mở (P0-6 §11), duyệt ADR |

## 2. Definition of Ready (DoR) — chỉ `claim` khi ĐỦ

- [ ] Mọi `Dep` của task có trạng thái `done` trên board (`cargo xtask board status`), commit `done` đã ở `main`.
- [ ] Đã đọc P0-5, P0-6 (mục liên quan) và thẻ task; hiểu mục **Acceptance**.
- [ ] Các tệp trong **Phạm vi** không bị agent khác giữ chỗ (`board guard <tệp…>` OK) và có thể `reserve`.
- [ ] Biết lệnh kiểm chứng của task chạy được trên máy mình (hoặc ghi rõ "chỉ chạy trên CI", ví dụ runner ARM).
- [ ] Không có quyết định mở chặn task (P0-6 §11) — nếu có: `board notify --to '*'` và chờ chủ repo.
- [ ] Môi trường: `rustc` theo `rust-toolchain.toml`, `cargo xtask preflight` xanh trên `main` hiện tại.
- [ ] Đã `board inbox` + `ack`; không còn thư chưa xử lý liên quan task.

## 3. Definition of Done (DoD) — task chỉ `release --status done` khi ĐỦ

Gồm **DoD 8 mục** của `../01-AGENT-HANDBOOK.md §4`, cộng:

- [ ] D1 Acceptance của thẻ task tick đủ, kèm bằng chứng (lệnh + kết quả rút gọn) trong mục *Ghi chú*.
- [ ] D2 `cargo xtask preflight` 25/25 (homebrew được SKIP local) trên commit của task.
- [ ] D3 Test hồi quy MỚI cho hành vi mới/đổi (không chỉ sửa test cũ); test đỏ khi đột biến luật chính
      (ghi đột biến đã thử trong *Ghi chú*).
- [ ] D4 Khối `textvn-session`: vết vàng khớp (SF-G1) và một vết cho 3 OS (SF-G2) hoặc khác biệt đã
      ghi trong `corpus/traces/ALLOWED_DIFF.md`.
- [ ] D5 ARM: job CI ARM liên quan xanh (Linux `ubuntu-24.04-arm`/`22.04-arm`, Windows `windows-11-arm`).
- [ ] D6 Hiệu năng: không vượt ngân sách P0-6 §9 trên mọi target có số đo.
- [ ] D7 Dream RSI (G19): savings-rate corpus ≥ baseline, đường nhận mới (nếu có) có hook `accepts`/`chars_saved`.
- [ ] D8 Bảo mật/riêng tư: không log chữ người dùng (G8), ô secure passthrough (S3), không API cấm
      (`cargo xtask hygiene-apis`), `cargo deny check` sạch, `reuse lint` sạch.
- [ ] D9 Tài liệu: spec/ADR liên quan cập nhật; mảnh `coordination/changes/<TASK>.md` (VI + EN 1:1, G18).
- [ ] D10 Mã lỗi mới đã cấp qua `board note "alloc E.."` và ghi vào `docs/specs/project-common-errors.md`.
- [ ] D11 Review bởi agent khác: `review-done … PASS` trên board.
- [ ] D12 CI `main` xanh sau khi push (G13/G17) — đỏ thì task chưa xong.
- [ ] D13 **G22 — không duplicate**: không thêm nghiệp vụ/chữ UI trùng ở `adapters/`, `tray/`, Swift/C; logic chung
      nằm trong `core/`/`textvn-session`/dữ liệu sinh mã; nếu task chạm vào bản sao cũ thì ghi task hợp nhất.

## 4. Audit checklist (agent review chạy ĐỘC LẬP; agent task tự chạy trước)

### 4.1 Đúng (correctness)
- [ ] Mọi luật/ID trong bảng P0-6 §5 của khối liên quan có test và test xanh (liệt kê tên test).
- [ ] Bất biến đầu ra P0-6 §3 (≤ 1 xoá + ≤ 1 commit, commit trước ẩn preedit, đúng một `ForwardKey`/`EatKey`)
      được kiểm bằng test thuộc tính/fuzz ≥ 60 s.
- [ ] Ca biên: chuỗi rỗng, Unicode ngoài BMP, tổ hợp phím chồng (rollover), mất focus giữa từ, reset toolkit,
      caret bị app dời, ô secure, terminal, Mode EN, Math đang mở.
- [ ] Không panic qua FFI (`catch_unwind`), tràn bộ đệm C ABI trả `IME_ERR_BUFFER` và không đổi trạng thái.

### 4.2 Nhất quán 3 OS (parity)
- [ ] Cùng corpus ⇒ cùng chuỗi lệnh trên TSF/IMK/IBus/Fcitx5 (`cargo xtask trace-diff`).
- [ ] Không còn nghiệp vụ trùng ở adapter cho khối đã chuyển (grep từ khoá P0-5 §3.2 trong `adapters/`).
- [ ] UI/settings 3 OS giống nhau (`docs/release/ui-spec.md`) nếu task chạm UI — lấy từ nguồn sinh mã (SF-9),
      không gõ lại chữ/luật ở từng OS (G22).

### 4.3 Bảo mật & riêng tư
- [ ] G8: grep log mới (`log::`, `eprintln!`, `g_debug`, `NSLog`, `print`) không in phím/chữ.
- [ ] IPC: client sandbox không gửi nội dung (E80/E81), pipe sandbox chỉ quyền dữ liệu, xác thực server giữ nguyên.
- [ ] Không crate mới ngoài danh sách `deny.toml`; crate mới = review giấy phép + lý do.
- [ ] Tệp ARM64X forwarder không có mã; `check-pe-imports` đúng machine type từng tệp.

### 4.4 Hiệu năng & Dream RSI
- [ ] `ime_key`/`ime_session_event` p50/p99 trên x86_64 + aarch64 so baseline theo target.
- [ ] 0 cấp phát ở đường nóng KeyDown (bench đếm cấp phát).
- [ ] `dream_rsi_snapshot` corpus: savings-rate, accepts không giảm.

### 4.5 ARM
- [ ] Hash đầu ra replay/Dream RSI giống hệt x86_64.
- [ ] Linux: build C test với `-fsigned-char` và `-funsigned-char` đều xanh.
- [ ] Windows: e2e gõ thật ở tiến trình ARM64, x64 giả lập, x86, AppContainer (ô tìm kiếm Start mô phỏng bằng `sandbox_e2e`).

### 4.6 Phát hành (release manager)
- [ ] Checklist A + B1–B9 (`docs/release/release-process.md`), asset ARM có `.asc` + `.cosign.*`, `SHA256SUMS` ký.
- [ ] `coordination/changes/*.md` đã gộp và xoá; board `rm-unlock` + notify `*`.

## 5. Thẻ task

> Định dạng: **ID · tên (cỡ, Dep)** — Mục tiêu · Phạm vi (tệp giữ chỗ) · Bước · Acceptance · Audit riêng ·
> Ghi chú (agent điền khi handoff/done).

### Luồng A — Linux aarch64

**ARM-L1 · CI Linux trên runner ARM (M, Dep: —)**
- Mục tiêu: `cargo test --workspace`, build adapter IBus/Fcitx5/settings, e2e IBus/Fcitx5, replay 4 adapter,
  `verify` chạy native trên `ubuntu-24.04-arm` và `ubuntu-22.04-arm`.
- Phạm vi: `.github/workflows/ci-linux.yml`.
- Bước: thêm ma trận `runs-on` cho job test/adapter/e2e; cache theo `runner.arch`; timeout riêng ARM.
- Acceptance: run CI trên PR/commit có 3 cột (x86_64, 24.04-arm, 22.04-arm) xanh; log `uname -m` = `aarch64`.
- Audit riêng: không đổi job x86_64 hiện có; không cross-compile.

**ARM-L2 · Gate `char` không dấu (S, Dep: —)**
- Mục tiêu: bắt lỗi `char` có/không dấu trên x86 trước khi lên ARM.
- Phạm vi: `adapters/linux-common/CMakeLists.txt` (hoặc tệp CMake test), `.github/workflows/ci-linux.yml`.
- Bước: target test thứ hai biên dịch với `-funsigned-char -Wsign-compare -Werror`; giữ target mặc định.
- Acceptance: `ctest` xanh với cả hai cờ; đột biến: đổi `(unsigned char)c < 0x20` trong
  `assist_result.c` thành `c < 0x20` ⇒ một trong hai lượt đỏ.

**ARM-L3 · Asset `linux-aarch64` trong release (M, Dep: ARM-L1)**
- Phạm vi: `.github/workflows/release.yml` (job `linux-arm64` mới), `scripts/build-linux.sh` (chỉ nếu cần), `scripts/test-linux-package.sh`.
- Acceptance: release-candidate có `TextVN-<v>-linux-aarch64.tar.gz` + `.asc` + `.cosign.*`, có trong
  `SHA256SUMS.txt`; `test-linux-package.sh` xanh trên runner ARM; `publish` đếm đủ số asset mới.
- Audit riêng: release manager cập nhật con số "34 asset" ở `release-process.md` + README.

**ARM-L4 · E2E 6 distro trên ARM (M, Dep: ARM-L3)**
- Phạm vi: `release.yml`/`ci-linux.yml` (ma trận container), ảnh arm64 của Ubuntu 22.04/24.04, Debian 12,
  Fedora, AlmaLinux 8/9, openSUSE Leap.
- Acceptance: 6/6 container arm64 xanh; ghi distro nào không có ảnh arm64 (nếu có) trong *Ghi chú*.

**ARM-L5 · deb arm64 + AUR aarch64 (S, Dep: ARM-L3)**
- Phạm vi: `.github/workflows/ci-deb.yml`, `packaging/arch/PKGBUILD`, `.SRCINFO`, `.github/workflows/ci-aur.yml`.
- Acceptance: `ci-deb` build `arm64` xanh; `PKGBUILD arch=('x86_64' 'aarch64')` và `.SRCINFO` đồng bộ (R2-120c);
  `makepkg` trong container `archlinuxarm` (nếu chạy được trên runner) hoặc ghi giới hạn.

### Luồng SF — `textvn-session`

**SF-1a · Khung crate `session/` (M, Dep: —)**
- Mục tiêu: crate `textvn-session` (workspace member, `publish = false`, không I/O) với `Session`,
  `SessionEvent`, `SessionCommand`, `Caps` theo P0-6 §3; chưa có logic (trả `ForwardKey`).
- Phạm vi: `session/` (mới), `Cargo.toml` (members), `Cargo.lock`.
- Acceptance: `cargo test -p textvn-session` (test bất biến đầu ra trên session rỗng); `cargo deny check` sạch.

**SF-1b · C ABI `ime_session_*` (M, Dep: SF-1a)**
- Phạm vi: `ffi/src/lib.rs`, `ffi/include/textvn_ffi.h`, `adapters/macos-imk/Sources/CTextVNFFI/include/textvn_ffi.h`, `cli/src/verify.rs`.
- Acceptance: `textvn-cli verify` kiểm sizeof/offset 4 struct mới; test tràn `IME_SESSION_MAX_CMDS`/`ARENA`;
  CI `ci-macos` gate header khớp. **R12**: notify `*` trước khi push (đổi hợp đồng).

**SF-1c · Ghi vết TSF (S, Dep: SF-1a)**
- Phạm vi: `cli/src/replay.rs` (cờ `--trace <dir>`), `corpus/traces/tsf/` (mới).
- Acceptance: `cargo run -p textvn-cli -- replay corpus/shared corpus/win --adapter tsf --trace corpus/traces/tsf` sinh JSONL tất định (chạy 2 lần giống hệt).

**SF-1d · Ghi vết Linux (M, Dep: SF-1a)**
- Phạm vi: `adapters/linux-common/tests/trace_compose.c` (mới), CMake test, `corpus/traces/linux/`.
- Acceptance: `ctest -R trace` sinh vết cho corpus shared; tất định.

**SF-1e · Ghi vết macOS (M, Dep: SF-1a)**
- Phạm vi: `adapters/macos-imk/Tests/…/TraceTests.swift` (mới), `corpus/traces/mac/`; CI `ci-macos.yml`.
- Acceptance: `swift test --filter TraceTests` sinh vết trên runner macOS (arm64).

**SF-1f · `xtask trace-diff` + `ALLOWED_DIFF.md` (S, Dep: SF-1c, SF-1d, SF-1e)**
- Phạm vi: `xtask/src/trace_diff.rs` (mới), `xtask/src/main.rs`, `corpus/traces/ALLOWED_DIFF.md`.
- Acceptance: báo khác biệt 3 adapter theo ca/bước; exit ≠ 0 khi có khác biệt chưa ghi phép; bước mới trong preflight.

**SF-2 · toggle + batch vào session (L, Dep: SF-1b, SF-1f)**
- Phạm vi: `session/src/{toggle,batch}.rs`; adapter: `adapters/windows-tsf/src/compose.rs` (`ModifierToggle`),
  `adapters/linux-common/src/{compose.c,edit_batch.c}`, `adapters/linux-ibus/src/engine.c`,
  `adapters/linux-fcitx5/src/engine.cpp`, `adapters/macos-imk/Sources/IMKLib/TextVNInputController.swift`.
- Bước: viết khối trong session + test luật P0-6 §5 (toggle, batch) → shadow mode trong 3 adapter
  (`TEXTVN_SESSION_SHADOW=1`) → CI 0 lệch → bật mặc định ở bản kế tiếp → xoá mã cũ ở bản sau.
- Acceptance: SF-G1/G2 cho corpus toggle; e2e IBus/Fcitx5/TSF/IMK xanh; shadow 0 lệch.

**SF-3 · compose (L, Dep: SF-2)** — `plan_key`/`lc_plan_key`/`ApplyReplace`/`Marked` → `session/src/compose.rs`; luật P0-6 §5 hàng *compose* + *reset* (R2-171/E78). Acceptance như SF-2 cho toàn corpus.

**SF-4 · ghost (M, Dep: SF-3)** — `update_ghost`/`update_preview`/`updatePredictionPreview` → `session/src/ghost.rs`; giữ R2-134/R2-173/E79. Acceptance: test `r2_173_*` chạy qua session; invariant "chữ mờ = Tab chèn".

**SF-5 · assist (L, Dep: SF-3)** — `assist.rs`/`panel_idle.rs`/`assist_*.c`/`AssistModel.swift` → `session/src/assist.rs`; giữ AS-022/030/050, R2-131/143/154/155, R-3, R-5. Acceptance: corpus `corpus/assist` + e2e trợ lý 3 OS.

**SF-6 · field + ipc (M, Dep: SF-2)** — Linux/mac dùng crate `field-detect` qua FFI; IPC client một bản (`ipc_client.c`, `IpcClient.swift` bị xoá); giữ A7b/R2-110/R2-26/E80/E81. Acceptance: `sandbox_e2e`, e2e IPC 3 OS.

**SF-7 · emoji logic (S, Dep: SF-2)** — chỉ tìm/xếp hạng/MRU; panel vẽ ở adapter.

**SF-9 · UI dùng chung nguyên lý (M, Dep: SF-1a)** — `data/ui/strings.toml`, `data/ui/settings_model.toml`,
`xtask` `gen-ui`/`check-ui`, thay `ABOUT_*` (`tray/src/settings_dialog.rs`), `tv_about_text`
(`settings_model.c`), `AppInfo.swift`, dòng gợi ý Mode trợ lý bằng bản sinh mã (P0-6 §5b). Acceptance:
`check-ui` trong preflight; test `about_text_matches_every_platform` thành test của generator; 0 chuỗi UI gõ tay
trùng nhau giữa 3 OS (grep).

**SF-8 · Gate "nghiệp vụ chỉ trong session" (S, Dep: SF-5, SF-6)** — `xtask hygiene-session` (mới) cảnh báo
khi `adapters/` thêm từ khoá nghiệp vụ (P0-5 §3.2); thêm vào preflight + quy tắc trong `00-WORKFLOW`.

### Luồng W — Windows ARM64

**ARM-W1 · Build DLL/exe `aarch64-pc-windows-msvc` (M, Dep: —)**
- Phạm vi: `build-release.ps1`, `.github/workflows/ci-windows.yml` (job `windows-arm64`, `runs-on: windows-11-arm`).
- Acceptance: `cargo test --workspace` native ARM64 xanh; `textvn-tsf-arm64.dll` + tray/CLI ARM64 build được;
  `check-pe-imports` báo machine ARM64; `.ps1` ASCII-only (G7).

**ARM-W2 · Forwarder ARM64X (M, Dep: ARM-W1)**
- Phạm vi: `installer/windows/arm64x/` (mới: `dummy.c`, `tsf_arm64.def`, `tsf_x64.def`), `xtask/src/win_tools.rs`
  (`build-arm64x`, kiểm machine ARM64X trong `check-pe-imports`).
- Acceptance: `cargo xtask build-arm64x` sinh `textvn-tsf.dll` machine ARM64X, chỉ export 4 hàm forwarder, không import CRT.
- Audit riêng: đột biến — đăng ký DLL ARM64 thay forwarder ⇒ e2e app x64 (W5) đỏ.

**ARM-W3 · `register` trên máy ARM64 (M, Dep: ARM-W2)**
- Phạm vi: `cli/src/register.rs`.
- Acceptance: phát hiện máy ARM64 (`IsWow64Process2`); ghi `InprocServer32` = forwarder; ACL AppContainer
  cho 4 DLL; gỡ xoá đủ; test đơn vị cho chọn đường dẫn + test e2e trên `windows-11-arm`.

**ARM-W4 · Bộ cài Inno + portable arm64 (M, Dep: ARM-W3)**
- Phạm vi: `installer/windows/TextVN-setup.iss`, `build-release.ps1`, `installer/windows/tests/*.ps1`.
- Acceptance: `ArchitecturesAllowed=x64compatible arm64`; cài/gỡ thật trên runner ARM (`test-installer.ps1`,
  `test-portable.ps1`); quyết định một bộ cài đa kiến trúc hay riêng (P0-6 §11) ghi ADR.

**ARM-W5 · E2E ma trận tiến trình (L, Dep: ARM-W3)**
- Phạm vi: `tools/win/*.ps1` e2e, `ci-windows.yml`.
- Acceptance: gõ TSF thật xanh ở Notepad (ARM64), một app x64 giả lập, một app x86, `sandbox_e2e` ARM64.

**ARM-W6 · `.msixbundle` x64 + arm64 (L, Dep: ARM-W4)**
- Phạm vi: `tools/win/build-msix.ps1`, `installer/windows/msix/`, `.github/workflows/{release,store-publish}.yml`,
  `tools/release/verify-store-package.sh`, `xtask verify-msix`.
- Acceptance: bundle cùng identity/version; verify từng gói; sideload thử trên runner ARM; Store check mode xanh.

### Luồng M — macOS Apple Silicon

**ARM-M1 · Gate universal cho mọi binary (S, Dep: —)** — `scripts/build-macos.sh`, `scripts/package-macos-pkg.sh`: mọi Mach-O trong `.app`/`.pkg` phải `arm64 x86_64` (lặp OpenKey #189 ⇒ đỏ).

**ARM-M2 · E2E IMK gõ thật trên runner arm64 (M, Dep: —)** — `ci-macos.yml`: kịch bản gõ vào TextEdit qua IMK (khi runner cho phép quyền accessibility; nếu không, ghi giới hạn + thay bằng test bridge).

**ARM-M3 · Notarization (S, Dep: secrets Apple)** — chờ chủ repo cấp secret.

### Luồng P — hiệu năng & chung

**PERF-1 · Baseline theo target (M, Dep: ARM-L1, ARM-W1)** — `perf/baseline-<triple>.json`, `xtask/src/preflight.rs` bước 17, `tools/bench`; Acceptance: preflight chọn đúng baseline theo máy; CI so baseline ARM.

**PERF-2 · Tất định đa kiến trúc (S, Dep: ARM-L1)** — `ci-shared.yml`: hash đầu ra replay + Dream RSI x86_64 vs aarch64 giống hệt.

**WL-1 · Backend Wayland input-method-v2 (L, Dep: SF-3, SF-4, quyết định chủ repo)** — `adapters/linux-wayland-im/` (Rust, mới); chỉ wlroots (Sway, labwc, Hyprland); e2e headless Sway trên CI; không thay IBus/Fcitx5 (P0-5 §7.3).

## 6. Thứ tự gợi ý cho nhiều agent song song

```
Tuần 1:  ARM-L1 ─┬─ ARM-L2           SF-1a ─┬─ SF-1b
                  └─ (PERF-2)                ├─ SF-1c ─┐
                                             ├─ SF-1d ─┼─ SF-1f
                                             └─ SF-1e ─┘
Tuần 2:  ARM-L3 → ARM-L4, ARM-L5     SF-2           ARM-W1 → ARM-W2
Tuần 3+: ARM-W3 → ARM-W4/W5 → ARM-W6 SF-3 → SF-4, SF-5, SF-6, SF-7 → SF-8      ARM-M1/M2
```

Tệp "nóng" dễ đụng nhau — giữ chỗ trước, làm ngắn, nhả sớm: `Cargo.toml`, `Cargo.lock`,
`ffi/include/textvn_ffi.h`, `.github/workflows/*.yml`, `build-release.ps1`, `cli/src/replay.rs`.

## 7. Ghi chú task (agent điền khi handoff / done)

| Task | Agent | Ngày | Ghi chú (nhánh/commit, bằng chứng acceptance, đột biến đã thử, việc còn lại) |
|---|---|---|---|
| — | — | — | — |
