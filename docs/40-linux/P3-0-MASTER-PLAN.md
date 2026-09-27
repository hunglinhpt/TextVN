# P3-0 — LINUX MASTER PLAN (Phần 3)

> Điều kiện tiên quyết: Phần 2 đạt 2/2 (`../30-macos/P2-REVIEW-LOG.md`).
> Tài liệu này = kế hoạch chi tiết phần Linux. Solution chi tiết ở `P3-1 … P3-7`;
> nhận việc theo task `LNX-xxx` trong `P3-7-TASKS.md`.
> Quyết định nền tảng: **ADR-007** — dual adapter **IBus + Fcitx5**; **không grab phím trên Wayland**
> (bug B10 của bamboo-viet) — xem `adr/README.md` + `PLAN §5.2/§5.3`.

## 1. Mục tiêu phần Linux (exit condition)

**Phần 3 xong khi:**
1. Gõ Telex/VNI/VIQR chuẩn: `corpus/shared` + `corpus/linux` ≥ 300 case pass (`--adapter linux`).
2. **App matrix Linux 20 app ≥ 95%** (B1/B2/B6/B8/B10/B13 không còn open) — `P3-6 §3`.
3. Cả 2 adapter cùng sống: IBus **và** Fcitx5 cài được, gõ được, không xử lý đôi.
4. Wayland (GNOME/KDE) gõ tốt qua framework chuẩn; X11 fallback hoạt động cho app legacy (opt-in).
5. Đóng gói `.deb` (primary) + `.rpm` + AUR; `textvn doctor` chẩn đoán env (`GTK_IM_MODULE`…).
6. CI `ci-linux.yml` xanh trên matrix distro (Ubuntu LTS, Fedora, Arch).
7. Review 2/2 của Phần 3 hoàn tất.

**Không nằm trong Phần 3:** AI suggest, Hán-Nôm, snap/Flatpak cho IME (loại — không cung cấp
input method cho system được), global hook cài sẵn (`PLAN §3.7`: "không cài global hook").
**Bất biến:** FFI (`P0-2`), schema (`P0-3`), corpus (`P0-4`) — mọi thay đổi phải bump đúng `P0-2 §6`.

## 2. Kiến trúc process (nhất quán `P1-0 §2` / `P2-0 §2`)

```
┌─ Wayland / X11 text app (GTK, Qt, Chromium, Electron…) ─────────────────────┐
│  ┌── ibus-daemon ── spawn ──► textvn-ibus-engine (process riêng, C)        │
│  │     IBusEngine.process_key_event → ime_key() → preedit/commit/surrounding │
│  └── fcitx5 ── load addon ──► libtextvn-fcitx5.so (trong process fcitx5, C++)│
│        keyEvent → ime_key() → preedit/commit (instance per InputContext chung │
│        main loop — 1 instance, reset theo context)                           │
│     Cả 2 link: libtextvn-linux-common.a (AT-SPI field detect + helpers)     │
│              + libtextvn_ffi.a (engine) → ipc client unix socket            │
└──────────────────────────────────────────────────────────────────────────────┘
┌─ X11 legacy / game (opt-in, KHÔNG chạy mặc định, KHÔNG dùng trên Wayland) ──┐
│  textvn-x11 (process riêng, Rust, spawn + watchdog bởi tray)                │
│    XGrabKeyboard khi app mục tiêu focus → ime_key() (instance riêng/tuần)    │
│    → inject bằng XTEST (marker chống loop)                                   │
└──────────────────────────────────────────────────────────────────────────────┘
┌─ textvn-tray (GTK4 + StatusNotifier — SOURCE OF TRUTH) ─────────────────────┐
│  • tray menu + Settings GTK4 (ui-model JSON chung) • IPC server (unix socket) │
│  • watcher config/appdb/state • systemd user service (autostart)              │
│  • doctor (env vars, component xml, addon) • spawn/kill textvn-x11           │
│  • updater: KHÔNG self-update — báo bản mới + mở trang package manager       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Đường dẫn chuẩn Linux (bổ sung `P0-3 §1` — đã có sẵn từ review Phần 2):**

| Hạng mục | Đường dẫn |
|---|---|
| Config/state/appdb | `~/.config/TextVN/{config.json, state.json, appdb.json}` |
| IPC socket | `~/.config/TextVN/ipc.sock` (0600 — đúng user, `P0-3 §5`) |
| Log | `~/.local/state/TextVN/log/` — không bao giờ ghi nội dung phím (S2) |
| IBus component | `/usr/share/ibus/component/textvn.xml` + engine binary `/usr/lib/textvn/textvn-ibus-engine` |
| Fcitx5 addon | `/usr/share/fcitx5/addon/textvn.conf` + `/usr/lib/fcitx5/libtextvn-fcitx5.so` |
| Tray/autostart | `/usr/share/applications/textvn-settings.desktop` + `/etc/xdg/autostart/textvn-tray.desktop` (gói system) hoặc `~/.config/autostart/` (gói user) |

> Quy tắc: đường dẫn system **chỉ** nằm trong spec đóng gói (`packaging/linux/`),
> code nhận qua `--prefix`/env; đường dẫn user hardcode 1 chỗ (module paths trong crate config).

## 3. Workstream & file solution

| WS | Tên | File solution | Sản phẩm |
|---|---|---|---|
| WS1 | IBus adapter (primary) | **`P3-1-ibus.md`** | `textvn-ibus-engine` gõ Telex trong gedit (demo M1) |
| WS2 | Fcitx5 addon (dual) | **`P3-2-fcitx5.md`** | addon chạy trên fcitx5 ≥ 5.0.x |
| WS3 | X11 fallback (opt-in) | **`P3-3-x11.md`** | `textvn-x11` cho app legacy/game X11 |
| WS4 | AT-SPI field detect + AppDB | **`P3-4-strategy-appdb.md`** | rules AT-SPI + 20 preset Linux + doctor env |
| WS5 | Tray/Settings/IPC/Package | **`P3-5-ui-packaging-release.md`** | `textvn-tray` GTK4 + `.deb/.rpm/AUR` |
| WS6 | Test & automation | **`P3-6-test-plan.md`** | AT-SPI driver + matrix + CI 3 distro |
| WS7 | Task & điều phối | **`P3-7-TASKS.md`** | LNX-001…LNX-066 |

## 4. Lộ trình (slice Linux; tuần tính từ khi bắt đầu Phần 3)

| Milestone | Tuần | Nội dung | Exit criteria |
|---|---|---|---|
| **L0 – Spike** | 1–2 | `LNX-001..009`: IBus tối thiểu, link FFI (C/C++), keyval/shift, surrounding/delete, AT-SPI permission, fcitx5 API pin, XGrab+XTEST, GHA xvfb | Bảng spike **18 mục** (`P3-1 §9` = 10 + `P3-2 §8` = 8) trong `docs/specs/linux-*.md` |
| **L1 – Text correctness** | 3–6 | IBus composition + engine full + `corpus/linux` ≥ 150 case | replay xanh; gedit/Safari-equivalent (Firefox) gõ đúng |
| **L2 – Compat layer** | 7–10 | AT-SPI detect + preset 20 app + Fcitx5 addon + env matrix + **X11 fallback opt-in** (`LNX-040..044`) | 12 app matrix ≥ 90%; corpus `bug_B*` Linux pass; Wayland guard đúng (B10) |
| **L3 – UX & packaging** | 11–14 | Tray GTK4/Settings/IPC/.deb + doctor | Cài `.deb` trên Ubuntu VM sạch; parity checklist Linux điền đủ |
| **L4 – Hardening** | 15–18 | Fuzz, soak, perf, matrix 3 distro, `.rpm`+AUR | KPI `PLAN §5.5` (Linux) đạt; 20 app ≥ 95% |
| **L5 – RC Linux** | 19–20 | Security + license + docs | Cổng ra `P3-6 §6` |

## 5. Dependency graph

```
LNX-001 (env: Rust + ibus dev + fcitx5 dev + xvfb)
  ├─► LNX-002 (spike IBus minimal) ─► LNX-010..019 (IBus core)
  ├─► LNX-003 (spike link FFI từ C/C++) ─► mọi adapter
  ├─► LNX-004 (spike surrounding/delete_surrounding + selection) ─► LNX-014/015
  ├─► LNX-005 (spike AT-SPI + a11y permission) ─► LNX-030..031
  ├─► LNX-006 (spike fcitx5 API + version pin) ─► LNX-020..025
  ├─► LNX-007 (spike XGrabKeyboard + XTEST + Wayland detect) ─► LNX-040..044
  ├─► LNX-008 (spike GHA xvfb + AT-SPI trên runner) ─► RM5
  └─► LNX-009 (corpus linux đầu) ─► mọi task reproduce-trước-fix
LNX-032..035 (preset 20 app) sau LNX-030
LNX-050..056 (tray/package) song song từ tuần 6 (IPC schema chốt P0-3)
LNX-060..066 (harness/test) bắt đầu tuần 3, chặn release
```

## 6. Rủi ro & mitigation (riêng Linux)

| # | Rủi ro | P/I | Mitigation |
|---|---|---|---|
| RL1 | **keyval/shift semantics của IBus** chưa rõ (keyval đã apply modifier chưa?) → sai phím với layout có shift | Trung/Cao | Spike `LNX-004`: ghi lại hành vi thật với 3 layout (us, vi, viqr typewriter); fallback translate bằng `xkbcommon` (weak feature detection — `PLAN §5.3`) |
| RL2 | **Fcitx5 addon API đổi giữa các bản** (V2/V3/V4) | Trung/Trung | Spike `LNX-006` pin tối thiểu `fcitx5-dev >= 5.0.14` (ho số thật từ spike), `#if FCITX_VERSION` cho API khác; CI matrix Ubuntu LTS + Fedora + Arch |
| RL3 | **Wayland không grab được phím** (B10) → app không đi IM framework không gõ được | Cao/Trung | Theo `PLAN`: đi chuẩn IBus/Fcitx5; X11 module tách riêng (`P3-3`), document giới hạn trong README (app Wayland-native không qua framework = out of scope) |
| RL4 | **AT-SPI bị từ chối** (GNOME a11y disabled) → mất field detect | Trung/Trung | `org.a11y.Status` check + hướng dẫn bật trong Settings; từ chối → fallback preset theo WM_CLASS (vẫn đúng phần lớn) — `P3-4 §4` |
| RL5 | `delete_surrounding_text` hỗ trợ lệch (GTK ✓, Qt ⚠, Chromium ⚠) → BackspaceType chập chờn | Trung/Trung | capability probe khi `focus_in` (IBus báo `surrounding_text` mode); không hỗ trợ → downgrade strategy theo quy tắc `P0-3 §3.1` (caps) |
| RL6 | Đóng gói đa distro (glibc, system paths, systemd user unit) | Trung/Trung | `.deb` primary; spec paths tách `packaging/linux/`; test cài trên Ubuntu 22.04/24.04 + Fedora + Arch (CI matrix) |
| RL7 | Engine process chết làm mất gõ giữa chừng | Trung/Trung | IBus: engine process riêng → ibus-daemon respawn; fcitx5: addon crash = fcitx5 crash → `catch` mọi callback + soak 24h; tray hiện heartbeat |
| RL8 | **XTEST/opt-in bị coi là keylogger** | Trung/Cao | Không cài mặc định (`PLAN §3.7`), mô tả quyền rõ (`docs/security/x11-grant.md`), không chạy trên Wayland, blocklist game sẵn |
| RL9 | Đụng `GTK_IM_MODULE`/`QT_IM_MODULE`/`XMODIFIERS` sai → app không thấy IM | Trung/Trung | `textvn doctor` detect + đề nghị fix từng app (dùng `PLAN §5.3` env matrix); preset ghi chú env cần thiết |
| RL10 | Link **staticlib Rust** vào C/C++ addon (symbol conflicts với fcitx5, `-fno-exceptions`?) | Trung/Trung | Spike `LNX-003`: `libtextvn_ffi.a` + `linux-common` vào C và C++ smoke; fallback `cdylib` (`libtextvn_ffi.so`) — FFI giữ nguyên (`P0-2`) |

## 7. Definition of Done cho PHẦN 3

- [ ] Mọi task `LNX-*` trong `P3-7-TASKS.md` đạt DoD 7 mục (Handbook §4).
- [ ] `P3-6 §6` release gate pass.
- [ ] `docs/compat.md` cộng số liệu Linux (20 app, 3 distro).
- [ ] `P3-REVIEW-LOG.md` đạt 2/2 → `00-INDEX` cập nhật.
- [ ] 0 finding `blocker/major` mở; RL1–RL10 có owner + trạng thái.
- [ ] Bàn giao: FFI/schema không đổi (hoặc bump đúng `P0-2 §6`); roadmap3 OS hoàn tất.
