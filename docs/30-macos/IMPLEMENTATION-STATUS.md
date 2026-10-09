# IMPLEMENTATION STATUS — adapter macOS (Farch-4)

> Cập nhật: 2026-10-09 (audit gốc 2026-09-30) · Tương ứng task MAC-001…066 (`P2-6-TASKS.md`).
> Nguyên tắc ghi nhận: **trung thực** — code xong ≠ verified; spike cần máy
> Mac thật thì ghi rõ, không tick trước.

## Trạng thái (2026-10-09)

Phát triển macOS **hoàn tất** (chủ repo xác nhận 2026-10-08): bộ gõ IMK (`TextVN-IM.app`), app
menu bar + Cài đặt (`TextVN.app`), gói `.pkg` per-user. CI `ci-macos` xanh — lint
script/plist/header, staticlib universal, `swift build` ×2 arch + `swift test` arm64 cho
`macos-imk`/`macos-tap`/`macos-app`, `replay corpus/mac`, ZIP/PKG candidate (vd. run
[37887743367](https://github.com/hunglinhpt/TextVN/actions/runs/37887743367) trên `f2920f1`).
`release.yml` build + test + phát hành `.pkg`/ZIP/tar.gz universal (v0.2.27) kèm chữ ký GPG +
Sigstore. Vòng rà soát 2 (2026-10-08/09) sửa thêm: strategy theo appdb (R2-43), `non_preedit`
đọc ngay (R2-49), trạng thái V/E chung tiến trình IMK (R2-44), luật gõ tắt như Windows/Linux
(R2-50), gỡ bản cài cho mọi người dùng (R2-51), ký Developer ID/notarize có điều kiện (R2-48).

**Còn mở:** Developer ID + notarization (MAC-055 — chưa có cert/secret, bản phát hành ký
ad-hoc: `docs/release/signing-status-mac.md`); updater (MAC-056); `ax-driver` (MAC-060);
`perf/baseline-mac.json` (MAC-062); tap `macos-tap` chưa nhúng vào bản phát hành — chi tiết
bảng "Còn mở" bên dưới.

## Đã triển khai (bảng thành phần)

Ghi chú lịch sử — audit 2026-09-30 sửa đồng bộ EN/VN IMK ↔ app (ToggleViEn, Snapshot, StateUpdate),
thứ tự self-heal trước engine, truyền IPC frame đầy đủ, retry khi mất kết nối,
smoke TextEdit không đụng tài liệu đang mở, nhãn kiến trúc artifact và gate
notarization. Chưa thể khẳng định những sửa này hoạt động thực tế cho đến khi
`swift build/test` trên macOS runner và smoke GUI trên máy Mac hoàn tất.
Workflow `ci-macos.yml` nay build/test cả `macos-app` và tạo ZIP/PKG **candidate
chưa ký** để kiểm; artifact CI này không phải bản production/notarized.
Script build/package từ chối ghi đè ZIP/PKG cùng version; khi phát hành bản mới
cần tăng version trong `Cargo.toml` hoặc lưu bản cũ trước.

| Thành phần | Vị trí | Task | Verify |
|---|---|---|---|
| Bảng keycode mac → VK canonical (nguồn TOML duy nhất) | `data/tables/keymap_mac.toml` → `core/src/keymap_mac_generated.rs` + `adapters/macos-imk/Sources/CoreBridge/KeyMapMacGenerated.swift` | MAC-013 (phần bảng) | `cargo test -p textvn-core --test keymap_mac` + `swift test` |
| 20 preset mac trong AppDB | `data/appdb.default.json` (`mac.*`, engine_owner `imk`/`tap`) | MAC-032 (phần preset) | `cargo test -p textvn-appdb` |
| Corpus mac 114 case | `corpus/mac/` (gen từ `xtask/src/mac_corpus_cases.rs`) | MAC-006 (≥60) · P2-5 §2 | `replay corpus/mac --adapter mac` = 114/114 (headless, mọi OS) |
| Generator corpus mac trong xtask | `cargo xtask gen-mac-corpus` / `check-mac-corpus` | — | `cargo test -p xtask` |
| CoreBridge FFI Swift | `adapters/macos-imk/Sources/CoreBridge/` | MAC-003 (phần code) | `swift test` trên runner |
| IMK adapter (controller, marked, apply, field detect, IPC, translator) | `adapters/macos-imk/Sources/IMKLib/` | MAC-010…019 (phần code) | `swift test` (CI ✅) · phát hành từ v0.2.x; chủ repo báo phát triển xong 2026-10-08 |
| Info.plist + entitlements + build-rust.sh (2 arch + lipo) | `adapters/macos-imk/` | MAC-001/003 (phần build) | CI `ci-macos.yml` |
| CGEventTap opt-in (tap thread, marker loop-guard, self-disable, injector 3 mode, permission poll) | `adapters/macos-tap/` | MAC-040…044 (phần code) | `swift test` (CI ✅) · **chưa nhúng vào bundle phát hành**, chưa có UI opt-in |
| Menu Bar App (NSStatusItem, badge V/E, menu 9 mục, IPC server unix socket, autostart Rule S5) | `adapters/macos-app/` | MAC-050…053 (phần code) | `swift test` (`TextVNAppTests`) |
| Bảng điều khiển Settings SwiftUI (UniKey 4.6 RC2 parity: 505x245 / 505x490, macro editor) | `adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift` | MAC-052 (phần code) | `swift test` (`TextVNAppTests`) |
| Cấu hình đóng gói & Homebrew Cask (Info-IM, Info-App, Entitlements, uninstall-check, textvn.rb) | `packaging/macos/`, `packaging/homebrew/` | MAC-054/056/057 | `uninstall-check.sh` + CI |
| Kịch bản đóng gói & cài đặt (`build-macos.sh`, `install_macos.sh`, `uninstall_macos.sh`) | `scripts/` | MAC-054/056 | bash syntax check + CI |
| Đóng gói `.pkg` + notarize (`package-macos-pkg.sh`, `notarize-macos.sh`, `component.plist`, `distribution.xml`, `pkg-scripts/postinstall`) | `scripts/`, `packaging/macos/` | MAC-054/055 | `bash -n` + `plutil` lint (job `packaging-lint`) · build `.pkg` thật trong `package-candidate` + `release.yml` ✅ · notarize ⏳ (chưa có Developer ID) |
| Harness macOS + targets 12 app (`soak.sh`, `smoke-imk.sh`, `mem-check.sh`, `targets/*.json`) | `tools/mac/` | MAC-061…063 · P2-5 §4/§5 | `bash -n` + `cargo run -p xtask -- check-mac-targets` (mọi OS) · chạy máy thật ⏳ |
| Gate schema targets mac trong xtask (parser JSON 0 dependency + 9 unit test) | `xtask/src/check_mac_targets.rs` | MAC-061 | `cargo test -p xtask` · job `xtask check-tables` (`ci-shared.yml`) |
| CI macOS | `.github/workflows/ci-macos.yml` | MAC-064 (job chính) | ✅ xanh (vd. run 37569093894 trên `main`, 37887743367 trên nhánh làm việc) — nightly chưa có |
| Replay mac trong ci-shared | `.github/workflows/ci-shared.yml` (replay job + `--adapter mac`) | — | ✅ 3 OS |

## Còn mở (2026-10-09)

| Hạng mục | Task | Ghi chú |
|---|---|---|
| Developer ID + hardened runtime + notarization | MAC-008, MAC-055 | Tooling có sẵn (`APPLE_DEVELOPER_ID_APP`, `DEVELOPER_ID_INSTALLER`, `APPLE_NOTARY_*`, `scripts/notarize-macos.sh`, `package-macos-pkg.sh --notarize`) nhưng chưa có cert và `release.yml` chưa truyền secret — `docs/release/signing-status-mac.md` |
| AX harness `ax-driver` 12 app, soak 24h, `perf/baseline-mac.json` | MAC-060…063 | `ax-driver` chưa code; `targets/*.json` đã gate bằng `xtask check-mac-targets` |
| Updater | MAC-056 | chưa có code |
| Submit Homebrew cask | MAC-057 | `packaging/homebrew/textvn.rb` có trong repo (sha256 đúng v0.2.27), chưa submit lên homebrew-cask; cập nhật sha sau mỗi release (B7b) |
| Nhúng tap opt-in vào bản phát hành + UI | MAC-040…044 | package có test; chưa ship |

Các spike MAC-002…007/009 (IMK template, link staticlib, cơ chế xoá, AX/TCC, đăng ký input
source) đã đóng qua code + mục "Quyết định kỹ thuật" bên dưới; không tạo bù các file
`docs/specs/macos-*-spike.md`.

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

1. MAC-055 khi có Apple Developer ID → thêm secret + nạp cert vào keychain runner + gọi
   `package-macos-pkg.sh --notarize` trong `release.yml` (`docs/release/signing-status-mac.md` §4).
2. MAC-060…063: viết `ax-driver`, chạy AX harness trên 12 app, soak 24h, ghi
   `perf/baseline-mac.json` trên máy Mac thật.
3. MAC-030/031 nâng FieldDetect lên AXObserver + cache chi tiết (hiện: gather async v1).
