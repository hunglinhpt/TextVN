# P3-7 — TASKS LINUX (WBS) — Nhận việc từng task

> Quy ước giống `P1-6-TASKS.md` / `P2-6-TASKS.md`: DoD 7 mục (Handbook §4);
> S ≤ 0.5 ngày · M = 1–2 ngày · L = 3–5 ngày; finding ghi `P3-REVIEW-LOG.md`.
> **Điều kiện bắt đầu:** đọc `P3-0` → `P0-2` (FFI) → `P3-1` (adapter chính).
>
> **Nhóm task dùng tiền tố `T` (T0–T6) — KHÔNG nhầm với milestone `L0–L5` của `P3-0 §4`:**
> T0=L0 · T1=L1 · T2+T3+T4 ⊂ L2 · T5=L3 · T6=L4/L5.

## T0 — Spike & môi trường (tuần 1–2)

### LNX-001 · Môi trường build Linux (S, dep: —)
Rust toolchain, `libibus-1.0-dev`, `fcitx5-dev`, `libatspi2.0-dev`, `libxcb/xtest-dev`,
`xkbcommon`, `xvfb`, `cmake`/`ninja`, `debhelper`/`rpm-build`.
- **Acceptance:** `cargo build` + `pkg-config --exists ibus-1.0 fcitx5 atspi2` + `xvfb-run true` pass;
  ghi version vào `docs/40-linux/env-linux.md`.

### LNX-002 · SPIKE: IBus engine C tối thiểu (L, dep: LNX-001) — **chặn WS1**
Mẫu engine C → sửa: key → preedit → commit trong gedit; component XML hiện trong Input Sources.
- **Acceptance:** `docs/specs/linux-spike.md` S1/S2 (`P3-1 §9`): demo `được` trong gedit;
  ghi số `IBusEngine` object khi 2 app cùng gõ (verify §1 instance model).

### LNX-003 · SPIKE: link `libvietime_ffi.a` từ C và C++ (M, dep: LNX-001) — **RL10**
CMake link vào binary C (ibus) + `.so` C++ (fcitx5), `vietime sizes` pass.
- **Acceptance:** smoke `ime_instance_new/ime_key` không crash ở cả 2; nếu symbol/exception conflict →
  chốt fallback `libvietime_ffi.so` (cdylib), ghi trong spike (không đổi FFI).

### LNX-004 · SPIKE: keyval/shift + selection + surrounding (M, dep: LNX-002) — **RL1/RL5**
Test thật: shift+letter keyval? commit thay selection? `delete_surrounding` GTK/Qt/Chromium?
- **Acceptance:** `docs/specs/linux-keyval-spike.md` chốt translate (§5.1) + bảng surrounding
  support → quyết định caps/strategy (`P3-4 §1`).

### LNX-005 · SPIKE: AT-SPI + a11y permission (M, dep: LNX-002)
Query từ engine process: bị từ chối? attr `app_id` (desktop-file-id) lấy ở đâu? password field?
- **Acceptance:** `docs/specs/linux-atspi-spike.md`: flow permission + attr + kết quả S7/S8.

### LNX-006 · SPIKE: Fcitx5 addon API + version pin (L, dep: LNX-003) — **RL2**
Đủ F1–F8 trong `P3-2 §8` (CMake, keyEvent, preedit, surrounding, 2 framework song song).
- **Acceptance:** `docs/specs/linux-fcitx5-spike.md` 8/8 + bảng version 4 distro + pin tối thiểu.

### LNX-007 · SPIKE: XGrabKeyboard + XTEST glyph + Wayland detect (L, dep: LNX-001) — **RL3/B10**
Chốt chiến lược glyph có dấu (§5.4), marker chống loop (§6.0), Wayland guard (§4).
- **Acceptance:** `docs/specs/linux-x11-spike.md` chốt (a)/(b)/(c) + strategy khả thi; fail →
  module chỉ release phần document được (không đoán — Handbook §2).

### LNX-008 · SPIKE: GHA xvfb + AT-SPI trên runner (M, dep: LNX-002/005)
Build + replay + demo app trên `ubuntu-latest`; pyatspi vs Rust atspi → chốt driver.
- **Acceptance:** ghi vào `docs/specs/linux-ci-spike.md`: kết quả S9/S10 + ngôn ngữ driver
  (→ `P3-6 §4`) + verdict RM5-P3.

### LNX-009 · Corpus `corpus/linux/` đầu tiên (M, dep: —, song song)
≥ 60 case theo `P3-6 §2` (B1/B2/B8/B10/B11/B13/secure/owner_no_double).
- **Acceptance:** `replay corpus/linux --adapter linux` chạy được (fail OK — reproduce trước fix).

## T1 — IBus core (tuần 3–6) — dep: LNX-002/003/004

### LNX-010 · Component XML + engine rỗng (M) — `P3-1 §8`
- **Acceptance:** cài vào Input Sources, gedit gõ tiếng Anh không đổi; load/unload ×100 không leak (valgrind).

### LNX-011 · process_key_event → PASS (M) — `P3-1 §5`
- **Acceptance:** corpus `combo_pass` (shared) pass trên gedit + Firefox.

### LNX-012 · Preedit + commit ngắn (L) — `P3-1 §6.1/§7`
- **Acceptance:** `duocj` → `được`; corpus `ibus_preedit_*` ≥ 40 + `bug_B11_*` pass.

### LNX-013 · keymap translate + đổi layout (M) — `P3-1 §5.1`
- **Acceptance:** unit 100 phím; gõ đúng với layout us + tieng viet (typewriter) theo spike LNX-004.

### LNX-014 · SelectionReplace (L) — `P3-1 §6.2`
- **Acceptance:** corpus `bug_B1_firefox_url`, `bug_B1_settings_search` pass ≥ 90%.

### LNX-015 · BackspaceType qua surrounding (L) — `P3-1 §6.3`
- **Acceptance:** corpus `linux_bs_*` ≥ 30 case; không surrounding → downgrade đúng (P0-3 §3.1).

### LNX-016 · focus/commit-before-hide + instance lifecycle (M) — `P3-1 §3/§7`
- **Acceptance:** corpus `bug_B2_enter_commit`, `bug_B2_libreoffice_dbus`, `bug_B13_*` pass.

### LNX-017 · Secure/password → `secure=1` (M, dep: LNX-005)
- **Acceptance:** corpus `secure_field_passthrough` pass; Firefox password không ra ký tự (manual).

### LNX-018 · Toggle EN/VN + hotkey + property menu (M)
Hotkey `Ctrl+Shift+Space` (parity 3 OS — ADR-011).
- **Acceptance:** toggle mọi app; corpus `restore_en_*` pass (B5); property menu hiện On/Off.

### LNX-019 · IpcClient unix socket (M) — `P3-5 §2`
- **Acceptance:** đổi method trong Settings → gõ đổi <1s; kill tray → gõ vẫn được (offline).

## T2 — Fcitx5 addon (tuần 5–8, song song) — dep: LNX-006

### LNX-020 · Addon skeleton + lifecycle (M) — `P3-2 §3`
- **Acceptance:** fcitx5 load/unload ×100 không leak; instance_map free đúng khi IC destroy.

### LNX-021 · keyEvent → PASS (M) — `P3-2 §4`
- **Acceptance:** corpus `combo_pass` trên fcitx5 (konsole/gedit).

### LNX-022 · Preedit + commit + word boundary (L) — `P3-2 §5`
- **Acceptance:** corpus `fcitx_preedit_*` ≥ 40 case pass.

### LNX-023 · SelectionReplace + BackspaceType/surrounding (M) — `P3-2 §5`
- **Acceptance:** corpus `bug_B1_*`, `linux_bs_*` pass trên fcitx5.

### LNX-024 · focus/commit-before-hide (M) — `P3-2 §3`
- **Acceptance:** corpus `bug_B2_enter_commit`, `bug_B13_*` pass.

### LNX-025 · Framework switch + chống đôi (M) — `P3-2 §7`
- **Acceptance:** corpus `owner_no_double` pass; đổi framework trong Settings có hiệu lực (chỉ ghi nhận, daemon cần restart → doctor nói rõ).

## T3 — Field detect + AppDB (tuần 7–10) — dep: LNX-005

### LNX-030 · `lc_field_detect` R1–R10 + mock tests (M) — `P3-4 §2`
- **Acceptance:** C unit ≥ 30 case pass; link được từ cả C, C++, Rust (bindgen).

### LNX-031 · Cache + AT-SPI event invalidate + budget (M)
- **Acceptance:** resolve p99 < 2ms (bench); 0 query đồng bộ trong key callback (counter).

### LNX-032 · Preset 20 app + corpus ≥ 40 case (M) — `P3-4 §3`
- **Acceptance:** bảng 20 dòng đủ; `replay corpus/linux` pass 100%; `notes` trỏ `Bn`.

### LNX-033 · Appdb loader + `ime_appdb_verify` (S) — P0-2 §1, không viết lại
- **Acceptance:** file sai chữ ký → từ chối + warning, không crash.

### LNX-034 · Override chain `P3-4 §6` (path Linux) (M)
- **Acceptance:** unit thứ tự 6 bước; `ignore_apps` mode 3 loại đúng (EVKey spec #1).

### LNX-035 · `doctor` env matrix + framework auto (M) — `P3-4 §7`, `P3-5 §6`
- **Acceptance:** 4 tình huống env sai → đề nghị đúng; framework auto-detect đúng GNOME/KDE.

## T4 — X11 fallback opt-in (tuần 8–11, song song) — dep: LNX-007

### LNX-040 · Process skeleton + focus watch + grab per-app (M) — `P3-3 §2/§4`
- **Acceptance:** chỉ grab khi target focus; Wayland → 0 process; corpus `owner_no_double`.

### LNX-041 · Callback + self-disable (M) — `P3-3 §3`
- **Acceptance:** 1000 key không loop; p99 < 2ms; >50 lần chậm liên tiếp → self-disable + tray báo.

### LNX-042 · Injection theo spike LNX-007 + marker (L) — `P3-3 §5/§6`
- **Acceptance:** corpus `x11_*` ≥ 20 case (hoặc document giới hạn theo kết quả spike);
  không kẹt modifier sau 500 lần inject.

### LNX-043 · Watchdog + ipc + health (M) — `P3-5 §4`
- **Acceptance:** kill -9 → restart < 500ms; doctor hiện trạng thái x11.

### LNX-044 · Wayland guard + `docs/security/x11-grant.md` (S)
- **Acceptance:** Wayland session → module off + giải thích (B10); doc mô tả đủ quyền (S9).

## T5 — Tray / Settings / Packaging (tuần 11–14)

### LNX-050 · Tray SNI + menu 9 mục + single instance (M) — `P3-5 §1`
- **Acceptance:** 9/9; instance thứ 2 → hiện cửa sổ đang mở; có systemd lẫn không systemd đều chạy được.

### LNX-051 · IPC server + watcher + health (L) — `P3-5 §2/§4`
- **Acceptance:** `vietime ipc probe` thấy 3 client; kill engine → hiện lỗi + restart được.

### LNX-052 · Settings GTK4 6 tab + parity checklist (L) — `P3-5 §3`
- **Acceptance:** `docs/release/parity-checklist.md` đủ PLAN §2.3 (M6) + §8; debounce 300ms.

### LNX-053 · systemd user unit + config init + hot-reload (M)
- **Acceptance:** `systemctl --global enable` từ postinst; sửa config tay → <1s; file sai → giữ bản cũ + warning.

### LNX-054 · `.deb` cài/gỡ sạch (L) — `P3-5 §5`
- **Acceptance:** Ubuntu VM: cài → thêm input source → gỡ = 0 residue system
  (`packaging/linux/uninstall-check.sh`); config giữ khi remove (S9).

### LNX-055 · `doctor` full (M) — `P3-5 §6`
- **Acceptance:** 6 hạng mục; `--export` không text content (grep test).

### LNX-056 · Update-notify + `.rpm` + AUR (M) — `P3-5 §7/§8`
- **Acceptance:** notify đúng version + mở trang; `dnf install` pass (Fedora VM);
  PKGBUILD `namcap`/`namcap`-equivalent + `makepkg` pass (Arch VM hoặc container).

## T6 — Test & hardening (tuần 15–18)

### LNX-060 · AT-SPI driver `tools/linux/atspi-driver` (L) — `P3-6 §4`
- **Acceptance:** `--suite ci` 12 app × 5 case = 60 case 1 lệnh; report cùng format Win/mac;
  0 case >15s; driver language theo spike LNX-008.

### LNX-061 · Targets JSON 12 app (cùng format 3 OS) (M)
- **Acceptance:** mỗi app ≥ 2 locator; upgrade 1 app → chạy lại OK.

### LNX-062 · Perf bench + `perf/baseline-linux.json` (M) — `P3-6 §5`
- **Acceptance:** CI so baseline, regression >10% → fail.

### LNX-063 · Soak 24h script × 2 (X11 + Wayland VM) (M)
- **Acceptance:** chạy 2h thử trước: 0 crash, RSS delta < 10MB.

### LNX-064 · CI jobs linux + dist matrix (M) — `P3-6 §8`
- **Acceptance:** `ci-linux.yml` + `ci-dist-matrix.yml` xanh 3 lần liên tiếp; nightly report Issue khi fail.

### LNX-065 · Fuzz + ASan/valgrind job (S)
- **Acceptance:** fuzz 15' PR / 60' nightly; ASan (C/C++) + valgrind job xanh.

### LNX-066 · Security + license checklist Phần 3 (M) — Handbook §8 + audit x11 (P3-3 §7)
- **Acceptance:** S1–S9 có chứng cứ; REUSE/`cargo deny` pass; `docs/security/x11-grant.md` đầy đủ.

## Thứ tự khuyến nghị

```
Tuần 1: LNX-001 → LNX-002 (khối chính) ∥ LNX-009 ∥ LNX-003
Tuần 2: LNX-004, LNX-005, LNX-006, LNX-007, LNX-008 (5 spike còn lại)
Tuần 3–6: LNX-010 → 011 → 012 → 013 → 014 → 015 → 016 → 017 → 018 → 019
Tuần 5–8: LNX-020 → 021 → 022 → 023 → 024 → 025 (song song L1/L3)
Tuần 7–11: LNX-030 → 035 ∥ LNX-040 → 044 (song song)
Tuần 11–14: LNX-050 → 051 → 052 → 053 → 054 → 055 → 056
Tuần 15–18: LNX-060 → 061 → 062 → 063 → 064 → 065 → 066 → release gate `P3-6 §6`
```

**Checkpoint mỗi milestone:** chạy full pyramid `P3-6 §1` → ghi `P3-REVIEW-LOG.md` (mục Trạng thái task) →
có blocker/major mở thì không kéo milestone kế tiếp.
