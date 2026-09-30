# P2-5 — TEST PLAN (macOS) — Solution chi tiết

> Kế thừa `../10-shared/P0-4-test-and-corpus.md` (7 lớp) + format `../20-windows/P1-5-test-plan.md`.
> Mọi test 1 lệnh hoặc ghi rõ "manual + ai chạy ở đâu".

## 1. Ma trận test

| Lớp | Chạy ở đâu | Lệnh | Gate |
|---|---|---|---|
| Unit (core/strategy/field/AX rules) | mỗi PR, 3 OS | `cargo test --workspace` + `swift test --package-path adapters/macos-imk` | PR |
| Corpus replay (`--adapter mac`) | mỗi PR | `cargo run -p textvn-cli -- replay corpus/ --adapter mac` | PR |
| Fuzz | PR 15' + nightly 60' | `cargo +nightly fuzz run key_event -- -max_total_time=900` | PR/nightly |
| AX harness (12 app) | **macos-latest nếu RM5 pass**, khác → local nightly (ghi lý do) | `tools/mac/ax-driver --suite ci` | nightly |
| Perf bench | mỗi PR (3 bench chính) | `cargo bench -p textvn-bench` | PR (ngưỡng §5) |
| Soak 24h | weekly (local Mac thật) | `tools/mac/soak.sh -Hours 24` | weekly → Issue |
| Manual | trước RC | `§7` | release |

## 2. Corpus macOS — `corpus/mac/` (mục tiêu ≥ 300 case, định dạng `P0-4`)

| File | Nội dung | Bug |
|---|---|---|
| `bug_B1_safari_url.keys` | Safari address bar: selection → `điệ` không backspace | B1 |
| `bug_B1_chrome_url.keys` / `bug_B1_spotlight.keys` | như trên | B1 |
| `bug_B1_excel_cell.keys` | Excel cell autocomplete | B1 |
| `bug_B2_imessage_enter.keys` | Messages: `<enter>` ×3 không lặp từ | B2 |
| `bug_B3_xcode_completion.keys` | role=candidate (§P2-3 R5) | B3 |
| `bug_B11_marked_commit.keys` | marked ngắn: Space/động từ commit ngay, không lag gạch chân | B11 |
| `bug_B13_focus_loss.keys` | click sang ô khác khi đang marked → chữ không mất | B13 |
| `bug_B8_terminal.keys` | Terminal/iTerm2 ForwardAsCommit + UTF-8 | B8 |
| `secure_field_passthrough.keys` | secure input → PASS | S3 |
| `owner_no_double.keys` | IMK+tap rule `P2-2 §6` | — |
| `tap_*` (≥ 20) | inject modes của tap (bản mô phỏng) | — |
| `imk_preedit_*` (≥ 40) | marked lifecycle (tương ứng `tsf_preedit_*`) | — |
| `restore_en_*` | auto-restore EN (B5) — dùng chung shared | B5 |
| `mac_bs_type_*` (≥ 30) | BackspaceType theo cơ chế MAC-004 | — |

Adapter `mac` = capability IMK/tap (`P2-3 §1`) + preset mac (`P2-3 §3`) — chạy headless mọi OS.

## 3. App matrix macOS (20 app — bằng chứng `PLAN §5.4` slice mac)

| # | App | Đường test | CI? | Bug |
|---|---|---|---|---|
| 1 | TextEdit | body, undo | ✓ | smoke |
| 2 | Safari | address bar + web form | ✓ | B1 |
| 3 | Chrome | address bar + web + contenteditable | ✓ | B1 |
| 4 | Firefox | address bar (Gecko) | ✓ | B1 |
| 5 | Spotlight (⌘Space) | search | ✓ | B1 |
| 6 | Notes | textarea + marked | ✓ | B11 |
| 7 | Terminal | terminal role | ✓ | B8 |
| 8 | VS Code | editor, multi-cursor | ✓ | B7/B11 |
| 9 | Xcode | editor + completion | ✓ | B3 |
| 10 | Slack | body + Enter | ✓ | B2 |
| 11 | Discord (Electron) | body | ✓ | B7 |
| 12 | Finder | rename + Go to folder | ✓ | B1 |
| 13 | iTerm2 | terminal + paste | nightly | B8/B11 |
| 14 | Messages | chat input + Enter | nightly | B2 |
| 15 | Mail | compose | nightly | B2 |
| 16 | Microsoft Word | body + suggestions | nightly | B2 |
| 17 | Microsoft Excel | cell autocomplete | nightly | B1 |
| 18 | System Settings | search field | nightly | B1 |
| 19 | IntelliJ IDEA | editor + completion | nightly | B3 |
| 20 | 1 app game (tap opt-in, `games_blocklist` mac) | owner=tap, ngoài chat passthrough | manual | RM8 |

**Ngưỡng:** 12 CI ≥ 95%; đủ 20 ≥ 95% (release gate §6).

## 4. AX harness — `tools/mac/ax-driver` (Swift executable)

```text
ax-driver --suite ci|full --only <app_id> --report out/report.json
1. open -a <app>  → chờ NSWorkspace frontmost + AX window ready
2. AXFocusedUIElement / AXUIElementCreateApplication → tìm element theo
   tools/mac/targets/<app>.json (CÙNG format targets JSON với Windows appcomptest — 2 locator fallback)
3. Focus: AXUIElementSetAttributeValue(kAXFocusedUIElementAttribute) (fallback: CGEvent click tọa độ window)
4. Gửi input: CGEvent keyDown/keyup (đi qua đúng đường phím — KHÔNG set AX value)
   (paste case: clipboard qua NSPasteboard + ⌘V)
5. Assert: AXValue/AXSelectedText/AXAttributedString → so :expect (đúng format P0-4)
6. Report JSON = format giống Windows appcomptest (id, status, ms, notes) → hợp nhất dashboard
7. Cleanup: ⌘Q app, dọn clipboard
```
- Timeout 15s/case → fail, không treo suite.
- **Schema gate (mọi OS, làm trước khi có Mac):** `cargo run -q -p xtask -- check-mac-targets`
  — `targets/*.json` đúng schema `tools/mac/README.md §1`, `preset` phải là `id` **có thật**
  trong `data/appdb.default.json`, mỗi field ≥ 2 locator AX, đủ 12 app CI
  (chạy trong job `xtask check-tables` của `ci-shared.yml` + 9 unit test `cargo test -p xtask`).
- Đo latency: timestamp trước khi post key → poll AX text đổi (1ms) → `t_glyph_ms` (p50/p99).
- **RM5:** lần đầu chạy trên GHA `macos-latest` (task MAC-007 S10) — nếu TCC chặn →
  ghi vào report "AX harness = local nightly", gate PR vẫn bằng corpus headless.

### 4.1 Smoke (PR, không cần matrix)
`tools/mac/smoke-imk.sh` — TextEdit + 5 case (được, chắc, tiếng anh, toggle, undo) — local/CI nếu có GUI.

## 5. Perf (ngưỡng — `PLAN §5.5`)

| Metric | Nguồn | Ngưỡng |
|---|---|---|
| `ime_key` p99 | `cargo bench -p textvn-bench -- key_latency` | < 0.5 ms |
| Strategy resolve p99 | bench `resolve` | < 2 ms |
| Tap callback p99 (nếu bật) | doctor counter | < 2 ms, 0 self-disable |
| Glyph latency (ax-driver) | `t_glyph_ms` | p95 < 50 ms |
| RSS steady (2 process) | `tools/mac/mem-check.sh` | IMK < 60MB, TextVN.app < 80MB |
| CPU idle | `top -l 2` 60s | 0% poll nóng |

Regression >10% vs `perf/baseline-mac.json` → fail (3 bench chính trên PR).

## 6. Release gate (RC macOS)

- [ ] `corpus/` (shared+mac) 100% pass với `--adapter mac`.
- [ ] Matrix 12 CI ≥ 95%, 20 app ≥ 95% (report đính kèm release notes).
- [ ] Fuzz 8h không crash; ASan job (xcodebuild) xanh.
- [ ] Soak 24h: 0 crash, 0 IMK restart ngoài ý muốn, RSS delta < 10MB.
- [ ] Perf §5 đạt, không regression >10%.
- [ ] **Notarization pass** (`spctl -a -vv` accept) — hard gate (RM3).
- [ ] Cài/gỡ VM sạch: 0 residue (`tools/mac/uninstall-check.sh`); input source biến mất sau gỡ.
- [ ] Security checklist S1–S9 (Handbook §8) + audit tap module (P2-2 §7).
- [ ] License audit: REUSE/`cargo deny` + Sparkle-đường-dây (nếu chọn) compatible GPL-3.
- [ ] `docs/compat.md` có số liệu mac; README mac section (cài + enable input source).

## 7. Manual checklist (trước RC — ghi `docs/release/rc-checklist-mac.md`; bản Windows: `rc-checklist-win.md`)

1. Cài `.pkg` trên VM macOS 13/14/15 sạch → hiện hướng dẫn → **bật input source** trong Settings.
2. Gõ Telex/VNI: TextEdit, Safari address bar, Spotlight, Terminal.
3. Toggle EN/VN hoạt động mọi app; `restore_en` chạy đúng (B5).
4. Đổi Settings → hiệu lực <1s (không cần logout).
5. Quyền AX: cấp/từ chối → preset fallback đúng (RM4); tap opt-in: cấp quyền → app không-IMK gõ được.
6. Update beta → stable qua updater; kill giữa chừng → rollback.
7. Gỡ (giữ config) → cài lại → config còn; input source không còn trong menu.
8. `textvn doctor --export` không có text content.
9. Gatekeeper: cài từ `.pkg` tải về trên VM khác (không cần `xattr` manual) → notarization OK.

## 8. CI jobs

| Job | Trigger | Nội dung |
|---|---|---|
| `ci-shared.yml` | PR/push | (P0-1) 3 OS incl. macOS: fmt/clippy/test/corpus |
| `ci-macos.yml` | PR chạm `adapters/macos*`, `corpus/mac/` | `swift build/test` (arch arm64 + x86_64 check), `replay corpus/mac`, `smoke-imk` (nếu GUI OK), `textvn sizes` |
| `ci-nightly-mac.yml` | schedule | ax-driver `--suite ci` (hoặc fallback ghi reason), fuzz 60', soak 2h, report → Issue |
| `ci-release.yml` | tag `v*` | build universal (lipo), codesign + notarytool, `.pkg` + SHA256SUMS + cask file |
