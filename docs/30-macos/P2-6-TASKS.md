# P2-6 — TASKS macOS (WBS) — Nhận việc từng task

> Quy ước giống `../20-windows/P1-6-TASKS.md`: DoD 7 mục (Handbook §4);
> S ≤ 0.5 ngày · M = 1–2 ngày · L = 3–5 ngày; finding ghi `P2-REVIEW-LOG.md`.
> **Điều kiện bắt đầu:** đọc `P2-0` → `P0-2` (FFI) → `P2-1` (adapter chính).

## A0 — Spike & môi trường (tuần 1–2)

### MAC-001 · Môi trường build mac (S, dep: —)
macOS 13+ với Xcode 15+ CLT, `rustup target add aarch64-apple-darwin x86_64-apple-darwin`,
SwiftPM, `swift-format`.
- **Acceptance:** `cargo build --release --target aarch64-apple-darwin` + `swift build` (template rỗng) pass;
  ghi version vào `docs/30-macos/env-mac.md`.

### MAC-002 · SPIKE: IMK tối thiểu (L, dep: MAC-001) — **chặn mọi task WS1**
Xcode template *Input Method* → sửa thành TextVN-IM tối thiểu: handle keyDown → marked → insertText.
- **Acceptance:** `docs/specs/macos-spike.md` có S1/S2/S4 (P2-1 §9): demo `được` trong TextEdit;
  input source hiện trong menu; list Info.plist keys **đối chiếu template thật**.

### MAC-003 · SPIKE: Swift ↔ Rust staticlib (L, dep: MAC-001) — **RM1**
`build-rust.sh` (2 arch + lipo) + `module.modulemap` + gọi `ime_key` từ Swift.
- **Acceptance:** smoke test Swift gọi `ime_instance_new/ime_key` không crash; size struct kiểm bằng
  `cargo run -p textvn-cli -- sizes` (P0-2 §6: 532/20) pass; ghi cách link vào `docs/specs/macos-spike.md`;
  fail → mở ADR (xcframework hoặc C dylib) trong 3 ngày, không lan ra.

### MAC-004 · SPIKE: cơ chế xóa/phím + selection (M, dep: MAC-002) — **RM2**
Thử (a) selector `deleteBackward` (b) `doCommand` (c) synthetic CGEvent + marker loop → chốt thứ tự.
- **Acceptance:** `docs/specs/macos-delete-spike.md` chốt cách BackspaceType; xác nhận `selectedRange()` ở
  Safari address bar/Spotlight; caps adapter chốt (có/không `IME_CAP_INJECT_VK`).

### MAC-005 · SPIKE: AX + TCC (M, dep: MAC-002)
Query AX từ process IMK: bị chặn? prompt thế nào? secure field detect? — **RM4**.
- **Acceptance:** `docs/specs/macos-ax-spike.md`: flow permission chính xác + fallback không-quyền.

### MAC-006 · Corpus `corpus/mac/` đầu tiên (M, dep: —, song song)
≥ 60 case theo `P2-5 §2` (B1/B2/B3/B11/B13/B8/secure/owner_no_double).
- **Acceptance:** `textvn replay corpus/mac --adapter mac` chạy được (fail OK — reproduce trước fix).

### MAC-007 · SPIKE: đăng ký input source + GHA GUI/TCC (M, dep: MAC-002)
Cách menu cập nhật sau khi copy bundle (kill TextInputMenuAgent / TIS API / logout); thử ax-driver trên `macos-latest`.
- **Acceptance:** `docs/specs/macos-registration-spike.md` chốt cách (a)/(b)/(c); kết luận RM5 (nightly ở đâu).

### MAC-008 · SPIKE: ký/notarization khả dụng (S, dep: —)
Kiểm tra Developer ID cert (có sẵn hay phải mua), thử `codesign` ad-hoc + `notarytool` flow.
- **Acceptance:** `docs/release/signing-status-mac.md`: trạng thái cert + ước số tiền/lệ phí + timeline.

### MAC-009 · SPIKE: CGEventTap 3 loại + permission (M, dep: MAC-001) — xem `P2-2 §4`
- **Acceptance:** `docs/specs/macos-tap-spike.md`: loại tap nào tạo được trên GHA/VM, marker loop có chống được không.

## A1 — IMK core (tuần 3–6) — dep: MAC-002/003/004

### MAC-010 · Bundle + IMKServer + controller rỗng (M)
Theo `P2-1 §2/§3`: main.swift, controller lifecycle, Info.plist §8.
- **Acceptance:** cài vào Input menu, TextEdit gõ tiếng Anh không đổi hành vi; activate/deactivate ×100 không leak (Instruments Leaks).

### MAC-011 · handle → `ime_key` PASS (M)
`P2-1 §5` bước 0–6 (chỉ PASS), chord check B6.
- **Acceptance:** corpus `combo_pass` (shared) pass trên TextEdit/Safari.

### MAC-012 · Preedit qua marked + commit ngắn (L) — `P2-1 §6.1/§7`
- **Acceptance:** `duocj` → `được` trong TextEdit; corpus `imk_preedit_*` ≥ 40 case + `bug_B11_*` pass.

### MAC-013 · KeyTranslator (UCKeyTranslate) + đổi layout (M)
- **Acceptance:** gõ đúng với layout ABC + Tiếng Việt (VNI Windows); unit test translate 100 phím.

### MAC-014 · SelectionReplace `P2-1 §6.2` (L)
- **Acceptance:** corpus `bug_B1_safari_url`, `bug_B1_spotlight`, `bug_B1_chrome_url` pass ≥ 90%.

### MAC-015 · BackspaceType theo MAC-004 + RESTORE (L) — `P2-1 §6.3`
- **Acceptance:** corpus `mac_bs_type_*` ≥ 30 case; 0 case còn chữ ảo (marked treo).

### MAC-016 · Focus/commit-before-hide + reset (M) — `P2-1 §3/§4`
- **Acceptance:** corpus `bug_B13_focus_loss` pass; manual click sang ô khác không mất chữ.

### MAC-017 · Secure field → `secure=1` (M, dep: MAC-005)
- **Acceptance:** corpus `secure_field_passthrough` pass; gõ mật khẩu không ra ký tự (manual Safari password).

### MAC-018 · Toggle EN/VN + hotkey + restore EN (M)
Hotkey chốt: `Ctrl+Shift+Space` (hoặc CapsLock dual-role — quyết định ở spike, ghi **ADR-011** nếu đổi).
- **Acceptance:** toggle hoạt động mọi app; corpus `restore_en_*` pass (B5).

### MAC-019 · IpcClient unix socket (M) — `P2-4 §2`
- **Acceptance:** đổi method trong Settings → gõ đổi <1s; kill TextVN.app → gõ vẫn được (offline).

## A2 — Field detect + AppDB (tuần 7–10) — dep: MAC-005/006

### MAC-030 · FieldDetect rules R1–R10 + mock tests (M) — `P2-3 §2`
- **Acceptance:** `swift test` ≥ 30 case mock AX dict pass; resolve đi qua `ime_strategy_resolve`
  (P0-2 §1) — **không** viết lại thuật toán P0-3 §3.1 trong Swift.

### MAC-031 · Cache + AXObserver + budget (M)
- **Acceptance:** benchmark ghi `P2-5 §5`: switch 200 lần app p99 < 2s? (cache hit) — cụ thể: resolve p99 < 2ms;
  0 AX query đồng bộ trong `handle()` (assert qua counter).

### MAC-032 · Preset 20 app + corpus ≥ 40 case (M) — `P2-3 §3`
- **Acceptance:** bảng 20 dòng đủ; `replay corpus/mac` pass 100%; `notes` trỏ `Bn`.

### MAC-033 · Appdb loader + Ed25519 (S) — dùng `textvn-appdb`, không viết lại
- **Acceptance:** file sai chữ ký → từ chối + warning trong status menu, không crash.

### MAC-034 · Override chain `P2-3 §4` (path mac) (M)
- **Acceptance:** unit test thứ tự 6 bước; `ignore_apps` mode 3 loại đúng (EVKey spec #1).

### MAC-035 · Settings "thêm app đang chạy" (S, dep: MAC-052)
- **Acceptance:** lưu appdb user hợp lệ (`textvn config validate`).

## A2b — EventTap opt-in (tuần 8–11, song song) — dep: MAC-009

### MAC-040 · Tap thread skeleton + owner check (M) — `P2-2 §2/§3`
- **Acceptance:** doctor thấy tap thread; app owner=imk → 0 inject (counter).

### MAC-041 · Callback + self-disable (M) — `P2-2 §3`
- **Acceptance:** manual 1000 key không loop; p99 < 2ms; `.tapDisabledByTimeout` → enable lại (test giả lập).

### MAC-042 · Inject §5 (3 modes + marker + restore) (L)
- **Acceptance:** corpus `tap_*` ≥ 20 case; không kẹt modifier sau 500 lần inject (test key state).

### MAC-043 · Permission UX + revocation poll (M) — `P2-2 §4`
- **Acceptance:** manual grant/revoke 2 tình huống; revoke → tap dừng, IMK vẫn gõ.

### MAC-044 · Rule owner `imk|tap` + IPC sync (S, dep: MAC-034)
- **Acceptance:** corpus `owner_no_double` pass.

## A3 — Menu bar / Settings / Packaging (tuần 11–14)

### MAC-050 · TextVN.app skeleton + status menu 9 mục (M) — `P2-4 §1`
- **Acceptance:** 9/9 mục; single instance.

### MAC-051 · IPC server unix socket + watcher + health (L) — `P2-4 §2/§6`
- **Acceptance:** `textvn ipc probe` thấy 2 client; kill IMK → menu hiện "IME chưa hoạt động" + restart được.

### MAC-052 · Settings SwiftUI 6 tab + parity checklist (L) — `P2-4 §3`
- **Acceptance:** `docs/release/parity-checklist.md` đủ mục `PLAN §2.3 (M6)` + `§8`; debounce 300ms.

### MAC-053 · Login item + config init + hot-reload (M)
- **Acceptance:** SMAppService register/unregister; sửa config tay → <1s; file sai schema → giữ bản cũ + warning.

### MAC-054 · `.pkg` cài/gỡ sạch (L) — `P2-4 §4`, dep: MAC-007
- **Acceptance:** VM macOS 13/15: cài → bật input source theo hướng dẫn → gỡ → `uninstall-check.sh` = 0 residue;
  config được giữ khi chọn (S9).

### MAC-055 · Developer ID + hardened runtime + notarization (M, dep: MAC-008) — **RM3**
- **Acceptance:** `spctl -a -vv` accept trên `.pkg`/2 app; `codesign --verify --deep --strict` pass;
  `docs/release/signing-status-mac.md` cập nhật.

### MAC-056 · Updater Ed25519 + rollback (L) — `P2-4 §7`
- **Acceptance:** update pre-release OK; sai hash → giữ bản cũ + log; kill giữa apply → rollback bằng `staging/<old>`.

### MAC-057 · Homebrew cask + submit (S, dep: MAC-056, release đầu)
- **Acceptance:** `brew install --cask textvn` trên VM sạch; `brew audit --strict` pass.

### MAC-058 · `textvn doctor --export` bản mac (M) — `P2-4 §9`
- **Acceptance:** zip không chứa text content (grep test) + đủ 8 hạng mục chẩn đoán.

## A4 — Test & hardening (tuần 15–18)

### MAC-060 · AX harness `tools/mac/ax-driver` (L) — `P2-5 §4`
- **Acceptance:** `--suite ci` 12 app × 5 case = 60 case 1 lệnh; report JSON cùng format Windows; 0 case >15s.

### MAC-061 · Targets JSON 12 app (cùng format Windows) (M)
- **Acceptance:** mỗi app ≥ 2 locator; upgrade 1 app → chạy lại OK.

### MAC-062 · Perf bench + `perf/baseline-mac.json` (M) — `P2-5 §5`
- **Acceptance:** CI so baseline, regression >10% → fail.

### MAC-063 · Soak 24h script (M)
- **Acceptance:** `tools/mac/soak.sh` chạy 2h thử trước: 0 crash, RSS delta < 10MB.

### MAC-064 · CI jobs macos (M) — `P2-5 §8`
- **Acceptance:** `ci-macos.yml` + `ci-nightly-mac.yml` xanh 3 lần liên tiếp; nightly report tạo Issue khi fail.

### MAC-065 · Fuzz + ASan job mac (S)
- **Acceptance:** fuzz 15' PR / 60' nightly; xcodebuild ASan pass.

### MAC-066 · Security + license checklist Phần 2 (M) — Handbook §8 + audit tap (P2-2 §7)
- **Acceptance:** S1–S9 có chứng cứ; REUSE/`cargo deny` pass; `docs/security/tap-permission.md` đầy đủ.

## Thứ tự khuyến nghị

```
Tuần 1: MAC-001 → MAC-002 (khối chính) ∥ MAC-006 ∥ MAC-008
Tuần 2: MAC-003, MAC-004, MAC-005, MAC-007, MAC-009 (5 spike còn lại)
Tuần 3–6: MAC-010 → 011 → 012 → 013 → 014 → 015 → 016 → 017 → 018 → 019
Tuần 7–11: A2 (030→035) ∥ A2b (040→044) song song
Tuần 11–14: MAC-050 → 051 → 052 → 053 → 054 → 055 → 056 → (057) → 058
Tuần 15–18: MAC-060 → 061 → 062 → 063 → 064 → 065 → 066 → release gate `P2-5 §6`
```

**Checkpoint mỗi milestone:** chạy full pyramid `P2-5 §1` → ghi `P2-REVIEW-LOG.md` (mục Trạng thái task) →
có blocker/major mở thì không kéo milestone kế tiếp.
