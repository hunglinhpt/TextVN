# P1-5 — TEST PLAN (Windows) — Solution chi tiết

> WS6 · kế thừa `../10-shared/P0-4-test-and-corpus.md`. File này = test **hệ thống/adapter Windows**.
> Mọi test phải chạy được 1 lệnh trên CI hoặc ghi rõ "manual + ai chạy, ở đâu".

## 1. Ma trận test (test pyramid)

| Lớp | Chạy ở đâu | Lệnh | Gate |
|---|---|---|---|
| Unit (core/strategy/field/rules) | mỗi PR, mọi OS | `cargo test --workspace` | PR |
| Corpus replay (headless, `--adapter win` = bản mô phỏng strategy Windows) | mỗi PR | `cargo run -p textvn-cli -- replay corpus/ --adapter win` | PR |
| Fuzz (K8-2) | PR 15 phút + nightly 60 phút | `cargo +nightly fuzz run key_event -- -max_total_time=900` | PR (15') / nightly |
| UIA harness (appcomptest) 12 app | nightly GHA `windows-latest` (RW5 **đã verify**: UIA chạy được trên runner ephemeral — run 36297278626 xanh cả `windows-2022`/`windows-2025`) hoặc self-hosted | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\verify_targets.ps1` (CI: `gh workflow run targets-verify.yml`) — crate `textvn-appcomptest` §4 = planned | nightly |
| Perf microbench | mỗi PR (3 bench engine) + WIN-031 khi đổi field detect | `cargo run --release -p textvn-bench -- check perf/baseline-win.json`; `cargo run --release -p textvn-bench -- field-switch --iters 20000` | PR (ngưỡng §5 / p99 cache+resolve < 2ms) |
| Soak 24h | Chủ nhật weekly | `tools\win\soak.ps1 -Hours 24` | weekly → báo Slack/Issue |
| Manual checklist | trước mỗi release RC | `P1-5 §6` | release |

## 2. Corpus Windows — `corpus/win/` (mục tiêu ≥ 300 case)

| File | Nội dung | Bug |
|---|---|---|
| `bug_B1_urlbar.keys` | address bar: `ddiêc` → `điệ` khi selection (Chrome, Edge, Firefox, Explorer) | B1 |
| `bug_B1_excel_autocomplete.keys` | ô Excel có autocomplete list | B1 |
| `bug_B2_chat_enter.keys` | `xin chào<enter>` ×3 → không lặp từ, không mất câu | B2 |
| `bug_B2_space_midword.keys` | gõ giữa câu, Space ngay sau khi commit | B2 |
| `bug_B3_completion.keys` | popup candidate đang mở: `dd` → `đ` không đóng/bẩn popup | B3 |
| `bug_B3_jetbrains.keys` | IDE completion (mô phỏng role=candidate) | B3 |
| `bug_B6_combo_pass.keys` | Ctrl+C/V/X/Z/A/F, Alt+Tab, Win+L, Ctrl+Shift+Esc → không nuốt | B6 |
| `bug_B7_electron.keys` | Discord/Slack: preedit + Enter + sửa giữa preedit (Electron) | B7 |
| `bug_B8_terminal.keys` | ForwardAsCommit trong WT/conhost: `越南` UTF-8 + `được` | B8 |
| `bug_B9_game_chat.keys` | game chat (owner=hook) + ngoài chat → passthrough | B9 |
| `secure_field_passthrough.keys` | role=secure → mọi phím PASS, không inject | S3 |
| `owner_no_double.keys` | 2 adapter (tsf+hook mock) theo rule P1-2 §6 → chỉ 1 xử lý | — |
| `hook_*` (≥ 40) | BackspaceType/SelectionReplace/inject unicode/vk_then_unicode (bản hook-sim) | — |
| `tsf_preedit_*` (≥ 40) | composition lifecycle: preedit → commit/space/enter/focus-loss | — |
| `restore_en_*` | auto-restore EN khi app EN (B5) | B5 |

Adapter `win` trong simulator = stack strategy y hệt `P0-3 §3.1` + capability TSF/Hook (`P1-3 §1`)
→ corpus chạy được trên mọi OS (không cần Windows thật cho mặt text-correctness).

## 3. App matrix (bằng chứng `PLAN §5.4` — Windows subset 20 app)

| # | App | Phường test | CI? | Bug liên quan |
|---|---|---|---|---|
| 1 | Notepad (Win11) | body, undo, IME off | ✓ | smoke |
| 2 | WordPad/Word 2019 | body + suggestions | ✓ | B2 |
| 3 | VS Code | editor, multi-cursor | ✓ | B7, P2 |
| 4 | Chrome | address bar, search box, web form, contenteditable | ✓ | B1 |
| 5 | Edge | address bar (Chromium khác UA) | ✓ | B1 |
| 6 | Firefox | address bar (Gecko, không Chromium) | ✓ | B1 |
| 7 | Excel | cell autocomplete, formula bar | ✓ | B1 |
| 8 | Windows Terminal | terminal role | ✓ | B8 |
| 9 | Explorer | address + search | ✓ | B1 |
| 10 | Slack (Electron) | body + Enter | ✓ | B2, B7 |
| 11 | Discord (Electron) | body + emoji popup | ✓ | B7 |
| 12 | JetBrains IDE (IntelliJ) | editor + completion popup | ✓ | B3 |
| 13 | Outlook/Thunderbird | compose body | nightly | B2 |
| 14 | Zalo Desktop | chat input | nightly | B2 |
| 15 | LibreOffice Writer | body (cross-app, không MS) | nightly | B4 |
| 16 | Figma/Canva | web canvas text | nightly | — |
| 17 | KeePassXC | secure field → PASSTHROUGH | nightly | S3 |
| 18 | RDP (mstsc) | unknown role | nightly | — |
| 19 | PowerShell 7 + conhost | owner=hook, ForwardAsCommit | nightly | B8 |
| 20 | 1 game có chat (preset blocklist) | owner=hook, ngoài chat passthrough | manual | B9 |

**Ngưỡng pass (mục tiêu Phần 1):** 12 app CI ≥ 95% case; đủ 20 app ≥ 95% (release gate §6).

## 4. UIA harness — `tools/appcomptest` (crate `textvn-appcomptest`)

> **Trạng thái (WIN-061):** harness **đang chạy** bằng PowerShell — `tools\win\verify_targets.ps1`
> (spawn app theo `launch.profile`, poll locator per-field ≤10s, assert text, cleanup) với targets
> `tools\appcomptest/targets/<app>.json` (12 app, fallback locator 2–3 selector); check tĩnh:
> `tools\win\check_targets.ps1`. CI = `targets-verify.yml` (dispatch). Lệnh `appcomptest.exe`
> bên dưới = **planned** (crate Rust `textvn-appcomptest` chưa build) — giữ làm đặc tả.

```text
appcomptest.exe --suite ci|full --only <app_id> --report out/report.json
1. Launch app (CreateProcessW, chờ window ready — UIA IsWindowEnabled)
2. Query UIA: tìm element theo rules (đúng field: address bar = {AutomationId/Name/ControlType} —
   rules file: tools/appcomptest/targets/<app>.json — CÙNG format với preset → reuse)
3. Focus element: Element::SetFocus()  (không click chuột — ổn định hơn)
4. Gửi input:  SendInput (key down/up thật, layout thật) — KHÔNG dùng UIA ValuePattern.Value
   (phải đi qua đúng đường phím → mới test được adapter; các case cần paste/selection dùng
    SendInput Ctrl+V với clipboard đã set)
5. Assert:  UIA getText (TextPattern, fallback ValuePattern) == expectation (từ corpus .keys)
6. Ghi report: {app, case, pass, ms_typed_to_visible, notes}  → out/report.json + markdown summary
7. Cleanup: đóng app (WM_CLOSE, kill sau 5s), dọn clipboard
```

- **Budget mỗi case < 15s** (timeout → fail, không treo suite).
- **Latency đo:** timestamp trước SendInput → poll text thay đổi 1ms → `t_glyph_ms`. Ghi p50/p99 vào report.
- Targets file có **fallback locator** (2-3 selector) vì UIA tree đổi theo version app.

### 4.1 Smoke script (PR không cần app matrix)

`tools/win/smoke-tsf.ps1` — Notepad + 5 case (được, chắc, tiếng anh, toggle, undo) — chạy được ở
GHA `windows-latest` nếu RW5 không chặn; nếu chặn → chạy local, đánh dấu trong report (không gate PR).

## 5. Perf (ngưỡng — liên kết `PLAN §5.5`)

| Metric | Nguồn đo | Ngưỡng |
|---|---|---|
| `ime_key` p50/p99 (chuỗi gõ thật: `duocj`/`toi`/`nguyen`/`vn`) | `cargo run -p textvn-bench --release` | p99 < 0.5 ms |
| Strategy resolve p99 | cùng lệnh, mục `ime_strategy_resolve` | < 2 ms (budget P0-3) |
| Hook callback p50/p99 (thật, qua hook-stats) | `textvn doctor --stats` | p99 < 2 ms, 0 self-disable |
| TSF glyph latency (appcomptest) | report `t_glyph_ms` | p95 < 50 ms (gồm cả roundtrip app) |
| RSS steady-state (3 process) | `tools/win/mem-check.ps1` | tray < 60MB, hook < 30MB, mỗi tsf instance < 15MB |
| CPU idle (không gõ) | ETW 60s | 0% (không poll nóng) |

Regression >10% so với baseline `perf/baseline-win.json` → CI fail (chỉ 3 bench chính trên PR).

> **Ghi chú hiện thực (2026-09-27, W3):**
> - `textvn-bench` **không dùng criterion** (0 dependency, chạy được 3 OS trong CI). Lệnh:
>   `cargo run -p textvn-bench --release -- [run|write <file>|check <file>]`.
> - Hồi quy **so `p50`**, không so `p99`: ở thang nano-giây p99 bị nhiễu timer/scheduler
>   (đo lại liên tiếp ±80%), ngưỡng 10% trên p99 là ngưỡng trên tiếng ồn. `p99` vẫn được
>   kiểm theo **ngân sách tuyệt đối** ở bảng trên (đó mới là con số có ý nghĩa cho người dùng).
>   Benchmark dưới 100 ns/lời gọi thì bỏ gate % (dưới nhiễu timer) — công cụ in rõ lý do.
> - Ba chống nhiễu đo (đều thêm sau khi *đo thật* thấy sai): **đo theo lô** (mỗi lần bấm giờ
>   = N lời gọi, chia đều — `Instant` trên Windows ngày ~100 ns), **best-of-3** (giữ vòng có
>   p50 nhỏ nhất — tiến trình khác làm một vòng chậm đi chứ không phải code chậm đi), và
>   **ngưỡng nhiễu** 100 ns. Sau đó 3 lần đo liên tiếp lệch nhau ±3.7% (trước đó là ±13%).
> - Job CI `perf` hiện `continue-on-error` vì runner GHA dùng chung; **gate cứng chạy trên
>   runner self-hosted** (risk RW5, `P1-5 §1`). `perf/baseline-mac.json` /
>   `baseline-linux.json` **chưa có** — phải sinh trên đúng OS đó, không dùng số của máy khác.

## 6. Release gate (cổng ra RC)

- [ ] `corpus/` (shared+win) 100% pass, `--adapter win` trên cả 3 OS.
- [ ] App matrix 12 CI ≥ 95%, 20 app ≥ 95% (report đính kèm release notes).
- [ ] Fuzz 8h không crash + 0 leak (LSAN); ASan/TSan job xanh.
- [ ] Soak 24h: 0 crash, 0 hook self-disable, RSS tăng < 10MB.
- [ ] Perf §5 đạt, không regression >10%.
- [ ] Security checklist (Handbook §8): S1–S9 ghi chứng cứ; `doctor --export` không lộ text (grep test).
- [ ] License audit: `cargo deny` (GPL-3.0-or-later-compatible), NOTICE đầy đủ.
- [ ] Cài/gỡ trên VM sạch × 2 mode (per-user, system): 0 residual (reg/file/service).
- [ ] Ký số: mọi binary chạy được có chữ ký hợp lệ (`Get-AuthenticodeSignature`).
- [ ] Docs: README, compat 20 app, changelog tiếng Việt.

## 7. Manual checklist (trước RC — ai đó chạy tay, ghi kết quả vào `docs/release/rc-checklist-win.md`)

1. Cài per-user trên VM Win10 22H2 + Win11 24H2 (2 máy/VM).
2. Chọn TextVN bằng Win+Space → gõ Telex/VNI trong Notepad, Chrome, Word.
3. Toggle `Ctrl+Shift+Space` hoạt động ở mọi app đã test.
4. Đổi config trong Settings → hiệu lực < 1s (không cần restart).
5. Game mode: hook tự bật/tắt theo foreground (P1-3 §3 row 19).
6. Cập nhật beta → stable qua Updater (kèm kill app giữa chừng → rollback chạy).
7. Gỡ cài đặt → chọn "Giữ config" → cài lại → config còn nguyên.
8. `textvn doctor --export` gửi được (mình đọc trước, không có text content).
9. App elevated (Notepad chạy admin): theo P1-2 §7 (uiAccess ON → gõ được; OFF → hiện warning).
10. Không có antivirus false positive sau khi submit (kiểm tra 7 ngày sau release).

## 8. CI jobs (nhắc lại `P0-1 §4` + bổ sung)

| Job | Trigger | Nội dung |
|---|---|---|
| `ci-shared.yml` | PR/push | fmt, clippy -D warnings, test 3 OS, build 3 target, corpus replay (3 OS), cargo-deny, docs |
| `ci-windows.yml` | PR/push | build+test windows, `textvn sizes` (P0-2 §6), smoke-tsf (nếu RW5 ok), perf bench 3 mục |
| `ci-nightly-win.yml` | schedule 02:00 UTC | appcomptest `--suite ci` (12 app), soak 2h, fuzz 60', report → artifact + Issue |
| `ci-release.yml` | tag `v*` | build, sign (SignPath), tạo GitHub Release + SHA256SUMS, winget manifest PR |
