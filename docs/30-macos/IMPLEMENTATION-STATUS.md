# IMPLEMENTATION STATUS — adapter macOS (Farch-4)

> Cập nhật: 2026-09-29 · Tương ứng task MAC-001…066 (`P2-6-TASKS.md`).
> Nguyên tắc ghi nhận: **trung thực** — code xong ≠ verified; spike cần máy
> Mac thật thì ghi rõ, không tick trước.

## Đã code hoàn chỉnh (code-complete, chạy trên CI macOS runner)

| Thành phần | Vị trí | Task | Verify |
|---|---|---|---|
| Bảng keycode mac → VK canonical (nguồn TOML duy nhất) | `data/tables/keymap_mac.toml` → `core/src/keymap_mac_generated.rs` + `adapters/macos-imk/Sources/CoreBridge/KeyMapMacGenerated.swift` | MAC-013 (phần bảng) | `cargo test -p textvn-core --test keymap_mac` + `swift test` |
| 20 preset mac trong AppDB | `data/appdb.default.json` (`mac.*`, engine_owner `imk`/`tap`) | MAC-032 (phần preset) | `cargo test -p textvn-appdb` |
| Corpus mac 114 case | `corpus/mac/` (gen từ `xtask/src/mac_corpus_cases.rs`) | MAC-006 (≥60) · P2-5 §2 | `replay corpus/mac --adapter mac` = 114/114 (headless, mọi OS) |
| Generator corpus mac trong xtask | `cargo xtask gen-mac-corpus` / `check-mac-corpus` | — | `cargo test -p xtask` |
| CoreBridge FFI Swift | `adapters/macos-imk/Sources/CoreBridge/` | MAC-003 (phần code) | `swift test` trên runner |
| IMK adapter (controller, marked, apply, field detect, IPC, translator) | `adapters/macos-imk/Sources/IMKLib/` | MAC-010…019 (phần code) | `swift test` (logic thuần) + smoke trên máy thật ⏳ |
| Info.plist + entitlements + build-rust.sh (2 arch + lipo) | `adapters/macos-imk/` | MAC-001/003 (phần build) | CI `ci-macos.yml` |
| CGEventTap opt-in (tap thread, marker loop-guard, self-disable, injector 3 mode, permission poll) | `adapters/macos-tap/` | MAC-040…044 (phần code) | `swift test` (logic) + manual trên máy thật ⏳ |
| Menu Bar App (NSStatusItem, badge V/E, menu 9 mục, IPC server unix socket, autostart Rule S5) | `adapters/macos-app/` | MAC-050…053 (phần code) | `swift test` (`TextVNAppTests`) |
| Bảng điều khiển Settings SwiftUI (UniKey 4.6 RC2 parity: 505x245 / 505x490, macro editor) | `adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift` | MAC-052 (phần code) | `swift test` (`TextVNAppTests`) |
| Cấu hình đóng gói & Homebrew Cask (Info-IM, Info-App, Entitlements, uninstall-check, textvn.rb) | `packaging/macos/`, `packaging/homebrew/` | MAC-054/056/057 | `uninstall-check.sh` + CI |
| Kịch bản đóng gói & cài đặt (`build-macos.sh`, `install_macos.sh`, `uninstall_macos.sh`) | `scripts/` | MAC-054/056 | bash syntax check + CI |
| CI macOS | `.github/workflows/ci-macos.yml` | MAC-064 (job chính) | xanh trên GitHub ⏳ |
| Replay mac trong ci-shared | `.github/workflows/ci-shared.yml` (replay job + `--adapter mac`) | — | ⏳ |

## Chưa thể verify khi không có máy Mac (cần chạy thật)

| Hạng mục | Task | Lý do |
|---|---|---|
| Spike IMK template thật + `được` demo trong TextEdit | MAC-002 (S1/S2/S4) | cần macOS GUI + input source thật |
| Verify link static Swift ↔ Rust từ build-rust.sh (RM1) | MAC-003 (S3) | cần macOS runner có GUI test — CI chỉ build+test logic |
| Chốt cơ chế xóa (a)/(b)/(c) + `selectedRange()` Safari/Spotlight | MAC-004 (S5/S6) | manual |
| AX/TCC flow + secure detect | MAC-005 (S7/S8) | manual (TCC) |
| Đăng ký input source (kill TextInputMenuAgent / logout) | MAC-007 (S9) | manual |
| Notarization / Developer ID | MAC-008, MAC-055 | cần cert |
| CGEventTap thật với quyền grant/revoke | MAC-009, MAC-043 | manual |
| AX harness 12 app, soak 24h, perf baseline-mac | MAC-060…063 | máy thật + GUI |
| `.pkg` cài/gỡ sạch, updater, Homebrew cask | MAC-054/056/057 | release phase |

## Quyết định kỹ thuật đã chốt trong code (ghi lại cho review)

1. **ForwardAsCommit trên mac = BackspaceType mechanics** — `ApplyReplace.swift`
   (P0-2 §4 "gõ ngay không xóa" là trick range TSF; delete_count của engine
   (`owned`/`pending_delete`) là số ký tự engine chắc chắn sở hữu → xóa bằng
   key binding chuẩn là đúng ngữ nghĩa IMK; corpus `mac_bs_type_*`/`bug_B8_*` chốt hành vi).
2. **Caps IMK v1** = `PREEDIT|FIELD_DETECT|SELECTION`, KHÔNG tự nhận `INJECT_VK`
   (P2-1 §1) — BackspaceType vẫn chạy được vì cơ chế (a) `deleteBackward:`
   (key binding chuẩn, không cần quyền) luôn có sẵn; cơ chế (c) synthetic CGEvent
   chỉ tồn tại trong tap module opt-in.
3. **1 instance engine / process IMK**, main thread duy nhất (P0-2 §3); tap dùng
   instance B trên thread tap qua protocol `TapKeyHandler` — tách module, crash
   tap không kéo IMK (P2-1 §12).
4. **Toggle EN/VN = Ctrl+Shift+Space** (P2-6 MAC-018, giữ mặc định; ADR-011
   cho CapsLock dual-role vẫn mở).
5. **Marker loop-guard** `0x5458564E ("TXVN")` dùng chung IMK + tap — test
   `TapMarkerTests` chặn lệch giá trị.
6. **B11 limit marked ≤ 8 grapheme → commit-early**; **B13 commit-before-hide**
   trong `deactivateServer`/`didClose` — đúng P2-1 §7.

## Tiếp theo (tuần tự)

1. Chạy MAC-001/002 trên máy Mac thật → điền `env-mac.md`, `macos-spike.md`.
2. MAC-004 chốt cơ chế xóa → nếu cần mở `IME_CAP_INJECT_VK`, cập nhật caps + replay profile.
3. MAC-030/031 nâng FieldDetect lên AXObserver + cache chi tiết (hiện: gather async v1).
4. MAC-060…063 Chạy AX harness trên 12 app macOS, soak test 24h và đo perf baseline trên máy Mac thật.
