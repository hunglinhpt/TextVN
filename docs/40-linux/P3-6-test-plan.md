# P3-6 — TEST PLAN (Linux) — Solution chi tiết

> Kế thừa `../10-shared/P0-4-test-and-corpus.md` (7 lớp) + format `P1-5` (Win) / `P2-5` (mac).
> Mọi test 1 lệnh hoặc ghi rõ "manual + ai chạy ở đâu".

## 1. Ma trận test

| Lớp | Chạy ở đâu | Lệnh | Gate |
|---|---|---|---|
| Unit (core/strategy/field/AT-SPI rules/C helpers) | mỗi PR | `cargo test --workspace` + `cmake --build && ctest` (linux-common, ibus, fcitx5 unit) | PR |
| Corpus replay (`--adapter linux`) | mỗi PR | `cargo run -p textvn-cli -- replay corpus/ --adapter linux` | PR |
| Fuzz | PR 15' + nightly 60' | `cargo +nightly fuzz run key_event -- -max_total_time=900` | PR/nightly |
| IBus smoke (xvfb) | mỗi PR (ubuntu-latest) | `xvfb-run ./tools/linux/smoke-ibus.sh` (gedit 5 case) | PR |
| AT-SPI driver 12 app | **nightly** (GHA xvfb nếu spike LNX-008 pass → `P3-6 §4`) | `tools/linux/atspi-driver --suite ci` | nightly |
| Perf bench | mỗi PR (3 bench chính) | `cargo bench -p textvn-bench` | PR (ngưỡng §5) |
| Soak 24h | weekly (VM X11 + VM Wayland) | `tools/linux/soak.sh -hours 24` | weekly → Issue |
| Matrix 3 distro cài đặt | trước RC | `packaging/linux/ci-install.sh` trên Ubuntu/Fedora/Arch | release |
| Manual | trước RC | `§7` | release |

## 2. Corpus Linux — `corpus/linux/` (mục tiêu ≥ 300 case, định dạng `P0-4`)

| File | Nội dung | Bug |
|---|---|---|
| `bug_B1_firefox_url.keys` | Firefox/Chrome address bar selection → `điệ` không backspace | B1 |
| `bug_B1_settings_search.keys` / `bug_B1_libreoffice_calc.keys` | ô tìm kiếm, ô Calc | B1 |
| `bug_B2_enter_commit.keys` | Enter trong chat → commit-before-hide, không lặp từ | B2 |
| `bug_B2_libreoffice_dbus.keys` | matrix bamboo: 6 key liên tiếp LibreOffice không nhảy chữ | B2 |
| `bug_B3_jetbrains_completion.keys` | role=candidate (R5) | B3 |
| `bug_B8_terminal.keys` | gnome-terminal/konsole ForwardAsCommit + UTF-8 | B8 |
| `bug_B10_wayland_guard.keys` | Wayland: textvn-x11 không khởi động; gõ qua framework OK | B10 |
| `bug_B11_short_preedit.keys` | preedit ngắn, commit ở word boundary | B11 |
| `bug_B13_focus_loss.keys` | click sang ô khác khi preedit → không mất chữ | B13 |
| `secure_field_passthrough.keys` | password (AT-SPI R1 / GTK input-purpose) → PASS | S3 |
| `owner_no_double.keys` | ibus vs fcitx5 vs x11 không đôi phím | — |
| `ibus_preedit_*` (≥ 40) | marked/preedit lifecycle | — |
| `fcitx_preedit_*` (≥ 40) | như trên, adapter fcitx5 | — |
| `linux_bs_*` (≥ 30) | BackspaceType qua `delete_surrounding` | — |
| `restore_en_*` | auto-restore EN (B5) — dùng chung shared | B5 |
| `x11_*` (≥ 20) | inject modes theo spike LNX-007 | — |

Adapter `linux` = capability IBus/Fcitx5/X11 (`P3-4 §1`) + preset Linux (`P3-4 §3`) — headless 3 OS.

## 3. App matrix Linux (20 app — đủ 3 framework)

| # | App | Đường test | Framework | CI? | Bug |
|---|---|---|---|---|---|
| 1 | gedit / GNOME Text Editor | body, undo | ibus | ✓ | smoke (M1) |
| 2 | Firefox | address bar + web form | ibus | ✓ | B1 |
| 3 | Chrome/Chromium | address bar + contenteditable | ibus | ✓ | B1 |
| 4 | GNOME Settings | search field | ibus | ✓ | B1 |
| 5 | Nautilus | rename + location bar | ibus | ✓ | B1 |
| 6 | GNOME Terminal | terminal role | ibus | ✓ | B8 |
| 7 | Text Editor (2 window gõ xen kẽ) | multi-context | ibus | ✓ | B13 |
| 8 | VS Code | editor, multi-cursor | ibus | ✓ | B7/B11 |
| 9 | Slack | body + Enter | ibus | ✓ | B2 |
| 10 | Discord (Electron) | body | ibus | ✓ | B7 |
| 11 | LibreOffice Writer | body + D-Bus 6 key | ibus | ✓ | B2 |
| 12 | LibreOffice Calc | cell | ibus | ✓ | B1 |
| 13 | Konsole | terminal (KDE) | fcitx5 | nightly | B8 |
| 14 | Kate | body | fcitx5 | nightly | |
| 15 | KDE System Settings | search | fcitx5 | nightly | B1 |
| 16 | Thunderbird | compose | ibus | nightly | B2 |
| 17 | Telegram Desktop | body + Enter | ibus | nightly | B2 |
| 18 | IntelliJ IDEA | editor + completion | ibus | nightly | B3 |
| 19 | Obsidian (Electron) | body | ibus | nightly | B7 |
| 20 | 1 game X11 (`games_blocklist`) | owner=x11 opt-in | x11 | manual | B10/RM8 |

**Ngưỡng:** 12 CI ≥ 95%; đủ 20 ≥ 95% (release gate §6).

## 4. AT-SPI driver — `tools/linux/atspi-driver`

```text
atspi-driver --suite ci|full --only <app_id> --report out/report.json
1. spawn app (gio launch <desktop-file>) → chờ window active
2. AT-SPI: focused object → tìm element theo tools/linux/targets/<app>.json
   (CÙNG format targets JSON với Windows appcomptest/mac ax-driver — 2 locator fallback)
3. Focus: ATSPI_ACTION_CLICK (fallback: xdotool windowactivate + click tọa độ)
4. Gửi input: XTEST key events (qua đúng đường phím — không set value trực tiếp)
5. Assert: ATSPI_TEXT get_text / state → so :expect (format P0-4)
6. Report JSON cùng format Windows/mac (id, status, ms, notes)
7. Cleanup: đóng app, dọn clipboard (xclip)
```
- Ngôn ngữ driver: **Python `pyatspi`** hoặc **Rust crate `atspi`** — spike `LNX-008` chốt
  (ưu tiên Python nếu GHA cài pyatspi nhanh hơn; không phụ thuộc vào GUI toolkit ngoài).
- Timeout 15s/case → fail. Đo `t_glyph_ms` (p50/p99) như mac.
- **RM5 (Linux):** nếu AT-SPI trên GHA xvfb không hoạt động → fallback:
  gate PR = corpus headless + ibus smoke (chạy được), AT-SPI driver = nightly local/self-hosted;
  ghi lý do vào report (giống `P2-5 §4`).

### 4.1 Smoke (PR)
`tools/linux/smoke-ibus.sh` — xvfb + gedit + 5 case (được, chắc, tiếng anh, toggle, undo).

## 5. Perf (ngưỡng — `PLAN §5.5`)

| Metric | Nguồn | Ngưỡng |
|---|---|---|
| `ime_key` p99 | `cargo bench -p textvn-bench -- key_latency` | < 0.5 ms |
| Strategy resolve p99 | bench `resolve` | < 2 ms |
| AT-SPI resolve p99 | `lc_field_detect` bench (C) | < 2 ms |
| Glyph latency (driver) | `t_glyph_ms` | p95 < 50 ms |
| RSS steady (tray + engine) | `tools/linux/mem-check.sh` | tray < 80MB, engine < 40MB |
| CPU idle | `pidstat` 60s | 0% poll nóng (chỉ AT-SPI cache TTL) |
| D-Bus/IPC overhead | counter bamboo "6× optimization" | không tăng poll D-Bus (0 poll trong hot path) |

Regression >10% vs `perf/baseline-linux.json` → fail.

## 6. Release gate (RC Linux)

- [ ] `corpus/` (shared+linux) 100% pass với `--adapter linux`.
- [ ] Matrix 12 CI ≥ 95%, 20 app ≥ 95% (report đính kèm release notes).
- [ ] Fuzz 8h không crash; ASan/valgrind job xanh (C/C++ addons + linux-common).
- [ ] Soak 24h × 2 (X11 VM + Wayland VM): 0 crash, RSS delta < 10MB, 0 engine restart ngoài ý muốn.
- [ ] Perf §5 đạt, không regression >10%.
- [ ] Cài/gỡ trên **Ubuntu 22.04/24.04 + Fedora + Arch**: 0 residue system
      (`packaging/linux/uninstall-check.sh`); input source vẫn/đã biến mất đúng.
- [ ] Wayland verify: GNOME Wayland + KDE Wayland gõ tốt; `textvn-x11` **không** khởi động (B10).
- [ ] Security checklist S1–S9 (Handbook §8) + audit `docs/security/x11-grant.md`.
- [ ] License audit: REUSE/`cargo deny` + IBus/Fcitx5/GTK deps compatible GPL-3.
- [ ] `docs/compat.md` có số liệu Linux; README Linux section (cài + chọn input source + env matrix).

## 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-linux.md`)

1. Cài `.deb` trên Ubuntu VM sạch (Wayland) → `ibus restart` → **Thêm TextVN** trong Settings → gõ Telex (gedit/Firefox/Settings search).
2. Cài trên KDE VM → fcitx5 path → Konsole/Kate gõ đúng; chọn framework trong tray.
3. Gõ Telex/VNI: LibreOffice Writer/Calc, VS Code, terminal, chat app (Enter không lặp — B2).
4. Toggle EN/VN hoạt động; `restore_en` chạy (B5).
5. Đổi Settings → hiệu lực <1s (không cần logout).
6. AT-SPI on/off → preset fallback đúng (RL4); Wayland: doctor giải thích giới hạn x11 (B10).
7. X11 opt-in: chọn game blocklist → gõ được (nếu spike cho phép) / bị giới hạn → warning đúng; tắt → phím tự nhiên.
8. Update-notify: bản mới → mở trang release; `apt/dnf upgrade` qua bản mới (§7).
9. Gỡ (`apt remove`) → 0 residue system; config giữ; `textvn purge` có xác nhận.
10. `textvn doctor --export` không chứa text content.

## 8. CI jobs

| Job | Trigger | Nội dung |
|---|---|---|
| `ci-shared.yml` | PR/push | (P0-1) 3 OS: fmt/clippy/test/corpus |
| `ci-linux.yml` | PR chạm `adapters/linux*`, `tray/`, `corpus/linux/` | build C/C++/Rust, unit (CTest+cargo), `replay corpus/linux`, `smoke-ibus` (xvfb), `textvn sizes` |
| `ci-dist-matrix.yml` | nightly | build trong container `ubuntu:22.04`, `fedora:latest`, `archlinux:latest` + `ci-install.sh` (cài/gỡ check) |
| `ci-nightly-linux.yml` | schedule | atspi-driver `--suite ci` (ho fallback ghi reason), fuzz 60', soak 2h, report → Issue |
| `ci-release.yml` | tag `v*` | build `.deb` + `.rpm` + AUR pkgver bump + SHA256SUMS |
