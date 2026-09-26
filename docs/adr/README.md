# ADR Index — Các quyết định kiến trúc của VietIME

> Quy tắc: mỗi ADR 1 file theo template `01-AGENT-HANDBOOK.md §7`.
> File này giữ **tóm tắt + trạng thái** để mọi agent tra nhanh.
> Mục `Chi tiết ở` trỏ tới tài liệu đã chốt (không trùng nội dung).

| ID | Tiêu đề | Status | Tóm tắt quyết định | Chi tiết ở |
|---|---|---|---|---|
| ADR-001 | Core viết bằng Rust + C-ABI | ✅ Accepted | Engine Rust (`vietime-core`), expose qua `vietime-ffi` (header `include/vietime_ffi.h`), instance-based, fail-open, không alloc chéo FFI | `../10-shared/P0-2-engine-ffi-contract.md` |
| ADR-002 | Output strategy engine 5 loại + preset data-driven | ✅ Accepted | `Preedit/BackspaceType/SelectionReplace/ForwardAsCommit/Passthrough`; chọn theo user preset → system preset → field-role default → fallback; capability-gated | `../10-shared/P0-3-config-preset-strategy.md §3` |
| ADR-003 | GPL-3.0-or-later + clean-room policy | ✅ Accepted | Toàn dự án GPL-3.0-or-later; ý tưởng từ EVKey/WinVNKey chỉ lấy qua spec clean-room (`docs/specs/`); REUSE/SPDX gate CI | `../01-AGENT-HANDBOOK.md §1, §7` |
| ADR-004 | UI Settings/Tray: egui (Windows) | ✅ Accepted | Pure Rust, không runtime ngoài, form sinh từ JSON schema; macOS/Linux có thể khác (SwiftUI/GTK4) nhưng giữ cùng `ui-model` JSON | `../20-windows/P1-4-ui-packaging-release.md` (Phần 1) |
| ADR-005 | Windows: TSF primary (Rust + `windows` crate) + hook fallback | ✅ Accepted (có spike) | TSF cho mọi app chuẩn; `vietime-hook.exe` (WH_KEYBOARD_LL) cho game/elevated; spike `WIN-002` tuần 1 quyết định giữ Rust hay quay C++/WRL | `../20-windows/P1-1-tsf.md`, `P1-2-hook.md` |
| ADR-006 | macOS: InputMethodKit primary, CGEventTap opt-in | ✅ Accepted | IMK primary (gõ không cần Accessibility permission); tap opt-in per-app với feature-detect 3 bước HID→Session→Annotated, không private API; SwiftUI settings cùng `ui-model` JSON | `../30-macos/P2-0-MASTER-PLAN.md`, `P2-1-imk.md`, `P2-2-eventtap.md` |
| ADR-007 | Linux: IBus + Fcitx5 dual adapter | ⬜ Proposed | Sẽ chốt trong Phần 3 | `../40-linux/` (Phần 3) |
| ADR-008 | Updater & signing: Ed25519 + OS signing | ⬜ Proposed (chi tiết ở Phần 1 cho Windows) | Verify chữ ký trước khi apply; rollback 1-click; preset update channel riêng | `../20-windows/P1-4-ui-packaging-release.md §5` |
| ADR-009 | Config schema v1, 1 định dạng cho mọi OS | ✅ Accepted | JSON + JSON Schema, `config_version` migrate, hot-reload qua IPC broadcast | `../10-shared/P0-3-config-preset-strategy.md §1` |
| ADR-010 | Test pyramid: golden corpus + app-compat automation | ✅ Accepted | Corpus `.keys` assertion inline làm oracle; UIA/AX/AT-SPI driver cho 40-app matrix | `../10-shared/P0-4-test-and-corpus.md` |
| ADR-011 | macOS toggle hotkey (Ctrl+Shift+Space vs CapsLock dual-role) | ⬜ Proposed | Chốt trong spike `MAC-018` trước khi implement; default giữ `Ctrl+Shift+Space` cho parity 3 OS | `../30-macos/P2-6-TASKS.md` (MAC-018) |

**Cập nhật ADR:** sửa table này + ghi `Status`/`Revisit trigger` trong file ADR chi tiết.
ADR `Proposed` phải được `Accepted` trước khi task liên quan bắt đầu.
