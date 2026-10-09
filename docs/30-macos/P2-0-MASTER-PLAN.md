# P2-0 — macOS MASTER PLAN (Phần 2)

> Điều kiện tiên quyết: Phần 1 đạt 2/2 (`../20-windows/P1-REVIEW-LOG.md`).
> Tài liệu này = kế hoạch chi tiết phần macOS. Solution chi tiết ở `P2-1 … P2-6`;
> nhận việc theo task `MAC-xxx` trong `P2-6-TASKS.md`.
> Quyết định nền tảng: **ADR-006** — IMK là primary (không cần Accessibility permission để gõ),
> **CGEventTap là opt-in** (chỉ khi user bật cho app cần) — xem `adr/README.md` + `PLAN §5.1/§5.2`.
>
> **Trạng thái (2026-10-09):** phát triển hoàn tất — bộ gõ IMK + app menu bar + `.pkg` per-user;
> `ci-macos` xanh; v0.2.27 phát hành `.pkg`/ZIP universal ký GPG + Sigstore
> (`IMPLEMENTATION-STATUS.md`). Điều kiện §1 còn thiếu: (2) app matrix 20 app, (4) nightly AX
> harness, (5) Developer ID + notarization (RM3 — chưa có cert; bản phát hành ký ad-hoc,
> `docs/release/signing-status-mac.md`). Updater và tap opt-in **chưa có trong bản phát hành**.

## 1. Mục tiêu phần macOS (exit condition)

**Phần 2 xong khi:**
1. Gõ Telex/VNI/VIQR chuẩn: `corpus/shared` + `corpus/mac` ≥ 300 case pass (`--adapter mac`).
2. **App matrix macOS 20 app ≥ 95%** (bug B1/B2/B3/B11/B13 không còn open) — `P2-5 §3`.
3. Cài per-user vào `~/Library/Input Methods/` (không sudo), bật được input source, gỡ sạch.
4. CI `ci-macos.yml` xanh; nightly AX-harness chạy được (ho fallback manual có ghi lý do — RM5).
5. Ký **Developer ID + notarization** (gate public release; xem RM3 nếu chưa có cert).
6. Review 2/2 của Phần 2 hoàn tất.

**Không nằm trong Phần 2:** gợi ý AI (backlog), Hán-Nôm, chính tả mặc định Should, mạng xã hội…
**Bất biến:** FFI (`P0-2`), schema config/appdb/ipc (`P0-3`), corpus format (`P0-4`) **không đổi**
hoặc chỉ bump version đúng `P0-2 §6` — để Phần 3 (Linux) cắm vào được.

## 2. Kiến trúc process (nhất quán với `P1-0 §2`)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ App người dùng (TextEdit, Safari, VS Code, Terminal…)                     │
│   └── TextVN-IM.app  (~/Library/Input Methods/, process IMK server)     │
│         • IMKInputController.handle() → ime_key() → marked/insertText     │
│         • AX field detect (opt-in Accessibility) → ime_set_context        │
│         • ipc client → unix socket  ~/Library/Application Support/        │
│                                 TextVN/ipc.sock  (schema ipc.v1.md)      │
│         • (opt-in) CGEventTap fallback — P2-2 (chưa có trong bản phát hành)│
└──────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────┐
│ TextVN.app  (NSStatusItem, LSUIElement — SOURCE OF TRUTH, như tray Win)  │
│   • menu trạng thái + Settings (SwiftUI, ui-model JSON chung)             │
│   • IPC server (unix socket) • watcher config/appdb/state                 │
│   • login item (SMAppService) • updater: CHƯA triển khai (MAC-056)        │
│   • diagnostics export • spawn/kill module IMK khi cần (restart IME)      │
└──────────────────────────────────────────────────────────────────────────┘
        Core engine (textvn-core + textvn-ffi) link STATIC vào từng binary
        qua staticlib — hot path không qua IPC (giống Windows, P1-0 §2).
```

**Đường dẫn chuẩn macOS (bổ sung cho `P0-3` — Windows-only paths):**

| Hạng mục | Đường dẫn |
|---|---|
| Config / state / appdb user | `~/Library/Application Support/TextVN/{config.json, state.json, appdb.json}` |
| IPC socket | `~/Library/Application Support/TextVN/ipc.sock` (0600, đúng user — tinh thần DACL của `P0-3 §5`) |
| Log | `~/Library/Logs/TextVN/` — không bao giờ ghi nội dung phím (S2) |
| Input method | `~/Library/Input Methods/TextVN-IM.app` |
| Settings app | `/Applications/TextVN.app` |
| Staging update | `~/Library/Caches/TextVN/staging/<ver>/` |

> Quy tắc: mọi path trên **chỉ** được hardcode trong 1 module paths (trong crate config);
> bảng per-OS chính thức: `P0-3 §1` (đã bổ sung trong review Phần 2 — finding F2-007).

## 3. Workstream & file solution

| WS | Tên | File solution | Sản phẩm |
|---|---|---|---|
| WS1 | IMK adapter | **`P2-1-imk.md`** | `TextVN-IM.app` gõ được Telex trong TextEdit/Safari |
| WS2 | EventTap fallback (opt-in) | **`P2-2-eventtap.md`** | `macos-tap` module, có permission UX |
| WS3 | AX field detect + AppDB mac | **`P2-3-strategy-appdb.md`** | rules AX + 20 preset mac |
| WS4 | Menu bar/Settings/IPC/Update | **`P2-4-ui-packaging-release.md`** | `TextVN.app` + `.pkg` + notarization + brew |
| WS5 | Test & automation | **`P2-5-test-plan.md`** | AX harness + matrix + CI macos |
| WS6 | Task & điều phối | **`P2-6-TASKS.md`** | MAC-001…MAC-066 |

## 4. Lộ trình (macOS slice; tuần tính từ khi bắt đầu Phần 2)

| Milestone | Tuần | Nội dung | Exit criteria |
|---|---|---|---|
| **A0 – Spike** | 1–2 | `MAC-001..009`: IMK tối thiểu, Swift↔Rust staticlib, cơ chế xóa/phím (P2-1 §9), AX permission, đăng ký input source, tap 3 loại, không có cert → ghi rõ | Bảng spike 10/10 mục `P2-1 §9` trong `docs/specs/macos-spike.md` |
| **A1 – Text correctness** | 3–6 | IMK composition + engine full + `corpus/mac` ≥ 150 case | `replay corpus/mac --adapter mac` xanh; TextEdit/Safari/Terminal gõ đúng |
| **A2 – Compat layer** | 7–10 | WS3 (AX detect + preset) + WS2 (tap opt-in) + fix B1/B11 | 12 app matrix ≥ 90%; corpus `bug_B1_*`, `bug_B11_*` pass |
| **A3 – UX & packaging** | 11–14 | WS4: status item/SwiftUI/IPC/.pkg/uninstall sạch | Cài/gỡ trên macOS VM sạch; parity checklist mac điền đủ |
| **A4 – Hardening** | 15–18 | Fuzz, soak 24h, perf, **notarization**, brew cask draft | KPI `PLAN §2.5` (mac) đạt; 20 app ≥ 95% |
| **A5 – RC macOS** | 19–20 | Security + license audit + docs | Cổng ra `P2-5 §6` |

## 5. Dependency graph

```
MAC-001 (env Rust+Xcode) ─┬─► MAC-002 (spike IMK) ─► MAC-010..019 (IMK core)
                          ├─► MAC-003 (spike Swift↔Rust staticlib) ─┘
                          ├─► MAC-004 (spike xóa/phím/selection) ─► MAC-014/015 (apply_replace)
                          └─► MAC-005 (spike AX + TCC) ─► MAC-030..031 (field detect)
MAC-006 (corpus mac) ──► mọi task WS1/WS3 (reproduce trước fix)
MAC-007 (spike đăng ký input source) ─► MAC-054 (.pkg + register)
MAC-008 (spike ký/notarize) ─► MAC-055 (signing) ─► release
MAC-009 (spike tap 3 loại + permission) ─► MAC-040..044 (tap opt-in)
MAC-032..035 (preset 20 app) sau MAC-030
MAC-040..044 (tap) sau MAC-009 (spike tap) — opt-in, không chặn A1
MAC-050..058 (settings/updater) song song từ tuần 6 (IPC schema đã chốt P0-3)
MAC-060..066 (harness/test) bắt đầu tuần 3, chặn release
```

## 6. Rủi ro & mitigation (riêng macOS)

| # | Rủi ro | P/I | Mitigation |
|---|---|---|---|
| RM1 | **Swift↔Rust staticlib link** phức tạp (xcframework, lipo, symbol conflicts) | Cao/Cao | Spike `MAC-003` tuần 1; fallback: engine build thành `xcframework` (lipo 2 arch) + `module.modulemap`; nếu vẫn chặn → bridge qua C dynamic lib (giữ FFI) |
| RM2 | IMK API mỏng cho "xóa N ký tự" (BackspaceType) | Trung/Cao | Spike `MAC-004`: liệt kê selector `responds(to:)` + thử `CGEvent` synthetic với loop-guard `CGEventSourceSetUserData`; ghi kết quả → quyết định caps adapter (`P2-1 §1`) |
| RM3 | **Chưa có Apple Developer ID cert** → không notarize được → cài bị Gatekeeper chặn | Cao/Trung | Dev: ad-hoc sign + `xattr -d com.apple.quarantine` (mất an toàn); Public: bắt buộc Developer ID + `notarytool` trước A5 — task `MAC-055`, ghi vào `docs/release/signing-status-mac.md` |
| RM4 | AX (Accessibility) bị TCC từ chối → mất field detect | Trung/Trung | `AXIsProcessTrustedWithOptions(prompt:)` 1 lần khi user bật "App-compat thông minh"; từ chối → fallback preset theo bundle-id (vẫn đúng phần lớn) — `P2-3 §4` |
| RM5 | **CI macOS không có GUI/TCC** → AX harness không chạy trên GHA | Cao/Trung | Spike `MAC-007` xác minh; fallback: corpus headless gate PR (chạy được), AX harness = nightly local/self-hosted Mac, ghi rõ trong `P2-5 §1` |
| RM6 | B11: marked text gạch chân lag/đóng sớm ở một số app (Electron, Office) | Trung/Trung | Commit strategy "ngắn marked" (commit ở word boundary, không giữ preedit dài), preset `backspace_mode` per-app — `P2-1 §7`, corpus `bug_B11_*` |
| RM7 | IMK process chết/restart làm mất gõ dở | Trung/Trung | `ime_reset` an toàn khi IMK lifecycle event; soak test 24h; status item hiện "IME not responding" khi heartbeat mất |
| RM8 | CGEventTap bị coi là keylogger (permission/PR, bản phân phối) | Trung/Cao | **Opt-in**, mô tả rõ quyền, không bật mặc định, không gửi mạng; PLAN §5.1/ADR-006 |

## 7. Definition of Done cho PHẦN 2

- [ ] Mọi task `MAC-*` trong `P2-6-TASKS.md` đạt DoD 7 mục (Handbook §4).
- [ ] `P2-5 §6` release gate pass.
- [ ] `docs/compat.md` cộng số liệu mac (20 app).
- [x] `P2-REVIEW-LOG.md` đạt 2/2 → `00-INDEX` cập nhật.
- [ ] 0 finding `blocker/major` mở; RM1–RM8 có owner + trạng thái.
- [ ] Bàn giao Phần 3 (Linux): FFI/schema không đổi (hoặc bump đúng `P0-2 §6`).
