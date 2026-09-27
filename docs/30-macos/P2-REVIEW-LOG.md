# P2-REVIEW-LOG — Phần 2 (macOS)

> Ghi **cả 2 review** (không xóa finding cũ). Finding `F2-xxx`.
> Đúng & Đủ (Review 1) + Nhất quán & Sẵn sàng (Review 2) — Handbook §6.
> Quy tắc: fix hết `blocker` + `major` trước khi kết thúc review; `minor` fix cùng đợt hoặc ghi rõ lý do.

## Kết quả tổng

| Review | Scope | Tổng | blocker | major | minor | Trạng thái |
|---|---|---|---|---|---|---|
| **Review 1 — Đúng & Đủ** | P2-0…P2-6 vs PLAN/P0/ADR + cross-part | 14 | 0 | 8 | 6 | ✅ 14/14 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng** | Tham chiếu chéo, placeholder, task ID, cross-part naming | 4 | 0 | 0 | 4 | ✅ 4/4 đã fix |

**→ Phần 2 đạt 2/2 review.** 0 `blocker`/`major` mở.

---

## Review 1 — Đúng & Đủ

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-001 | major | `adr/README`: ADR-006 (IMK primary + tap opt-in) vẫn `⬜ Proposed` nhưng P2-0/P2-1/P2-2 đã dùng làm nền tảng — vi phạm quy tắc "ADR phải Accepted trước khi task liên quan bắt đầu" | ✅ Fixed | ADR-006 → ✅ Accepted, tóm tắt + `Chi tiết ở` trỏ P2-0/P2-1/P2-2 |
| F2-002 | major | `P2-6 MAC-018` trỏ `ADR-007` (= Linux dual adapter, sai chủ đề) cho quyết định hotkey mac | ✅ Fixed | Thêm `ADR-011` (macOS toggle hotkey, ⬜ Proposed, chốt ở spike MAC-018) + sửa ref |
| F2-003 | major | `ime sizes` / `ime_sizes()` được dùng ở `P1-5 §8`, `P2-1 §9`, `P2-5 §8`, `P2-6 MAC-003` nhưng **không tồn tại** trong FFI/CLI | ✅ Fixed | Định nghĩa subcommand `textvn sizes` (in + verify size struct 20/532 theo P0-2 §6) trong `P0-1 §2/§3`; sửa 4 chỗ tham chiếu |
| F2-004 | major | `P0-1 §1` layout thiếu các mục P2 tham chiếu: `adapters/macos-tap/`, `tools/mac/`, `packaging/homebrew/`, `docs/`, `perf/` | ✅ Fixed | Bổ sung đủ vào layout + ownership |
| F2-005 | major | `engine_owner` được P1-2/P1-3/P2-2/P2-3 dùng nhưng **không có** trong schema `appdb.v1` (P0-3 §2.1) | ✅ Fixed | Thêm field optional + enum per-OS (`tsf\|hook\|imk\|tap\|ibus\|fcitx5`), default theo OS |
| F2-006 | major | P0-3 §5 chỉ mô tả transport named pipe (Windows); P2-4 dùng unix socket | ✅ Fixed | Bảng transport per-OS trong P0-3 §5 (Windows pipe / mac+Linux unix socket), **cùng 1 schema ipc.v1**, vẫn cấm TCP/HTTP |
| F2-007 | major | P0-3 §1 hardcode `%APPDATA%` — không có đường dẫn mac/Linux (P2-0 §2 liệt kê path mac nhưng chưa vào nguồn sự thật) | ✅ Fixed | Bảng đường dẫn per-OS (config/state/appdb + log) trong P0-3 §1; P2-0 trỏ lại |
| F2-008 | minor | `P2-0 §4` ghi `MAC-001..008` và dependency graph thiếu node MAC-009 (spike tap) | ✅ Fixed | `MAC-001..009`, thêm `MAC-009 → MAC-040..044` |
| F2-009 | minor | `P2-0 §2` còn placeholder `finding F2-xxx` | ✅ Fixed | Trỏ thẳng F2-007 |
| F2-010 | minor | `P0-1 §3` thiếu lệnh build/test mac (`build-rust.sh`, `swift build/test`, replay `corpus/mac`) | ✅ Fixed | Thêm mục macOS-only + lệnh replay `--adapter mac` |
| F2-011 | major | Adapter **không phải Rust** (macOS Swift, Linux C/IBus) không có C-ABI để verify appdb (Ed25519) + resolve strategy → nguy cơ viết lại thuật toán P0-3 §3.1 = phân kỳ giữa các OS | ✅ Fixed | Thêm 2 hàm tĩnh vào `P0-2 §1`: `ime_appdb_verify()`, `ime_strategy_resolve()`; P2-3 §5 + MAC-030 bắt buộc dùng (không viết lại trong Swift) |
| F2-012 | minor | `data/games_blocklist.txt` (P1-2 §8, P2-2 §7) không có trong layout `data/` của P0-1 | ✅ Fixed | Bổ sung |
| F2-013 | minor | Comment `static CRT: see P1-4` trong P0-1 §3 trỏ mục không nói về CRT (ref mồ côi) | ✅ Fixed | Bỏ ref |
| F2-014 | minor | Ô bảng `config init\|validate` trong P0-1 §2 có `|` không escape → hỏng bảng markdown | ✅ Fixed | Đổi thành `config init+validate` |

## Review 2 — Nhất quán & Sẵn sàng

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-015 | minor | `P2-4 §2` còn placeholder `F2-xxx` (chưa trỏ ID thật) | ✅ Fixed | Trỏ F2-006 |
| F2-016 | minor | Dependency graph `P2-0` tham chiếu `MAC-040..045` — MAC-045 không tồn tại | ✅ Fixed | `MAC-040..044`, dep từ MAC-009 (khớp P2-6/P2-2) |
| F2-017 | minor | `textvn uninstall` (P2-4 §4) không có trong bảng CLI của `P0-1 §2` | ✅ Fixed | Bổ sung `uninstall` |
| F2-018 | minor | Bất nhất tên file RC checklist: P1-5 `rc-checklist.md` vs P2-5 `rc-checklist-mac.md` | ✅ Fixed | P1-5 → `rc-checklist-win.md`; P2-5 ghi chú cross-ref (cross-part, sửa cả Phần 1) |

## Kiểm chứng sau fix (Review 2 cuối)

- [x] `grep "[CJK]" docs/` → 0 (trừ `越南` **có chủ đích** trong test UTF-8 `P1-5 §2` + trích dẫn finding trong log).
- [x] Mọi `MAC-xxx` được tham chiếu trong P2-0…P2-5 đều có trong `P2-6-TASKS.md` (040..044, 050..058, 060..066 — không còn `MAC-045`).
- [x] `grep "ime sizes|ime_sizes"` → chỉ còn dạng đúng `textvn sizes`.
- [x] `grep "F2-xxx|F0-xxx|F1-xxx"` trong P0/P1/P2 → 0 (chỉ còn trong các REVIEW-LOG như trích dẫn lịch sử).
- [x] `IME_CAP_{PREEDIT,SELECTION,FIELD_DETECT,INJECT_VK}` trong P2 khớp `P0-2 §1`.
- [x] Tham chiếu `§` nội bộ P2-0…P2-6 đối chiếu đầu mục (P2-1 §1/§6/§9, P2-2 §4/§6, P2-3 §2/§3/§4/§5, P2-4 §1…§9, P2-5 §1…§8) — không còn trỏ sai.
- [x] Bảng markdown không còn ô chứa `|` chưa escape.

## Rủi ro còn mở của Phần 2 (theo dõi, không phải finding)

| # | Rủi ro | Trạng thái | Task xử lý |
|---|---|---|---|
| RM1 | Swift↔Rust staticlib link | ⬜ Chưa chạy spike (tuần 1) | MAC-003 |
| RM2 | Cơ chế xóa N ký tự trên IMK | ⬜ Chưa chạy spike | MAC-004 → chốt caps |
| RM3 | Chưa có Apple Developer ID cert | ⬜ Cần mua/làm phép | MAC-008 + MAC-055 |
| RM4 | AX/TCC bị từ chối | ⬜ Chưa verify | MAC-005 |
| RM5 | CI macOS không chạy AX harness | ⬜ Chưa verify | MAC-007 S10 |
| RM6 | Marked text lag ở app Electron/Office | ⬜ Design có sẵn (P2-1 §7) | MAC-012 + corpus B11 |
| RM7 | IMK process chết giữa chừng | ⬜ Design có sẵn (P2-4 §6) | MAC-051 |
| RM8 | CGEventTap bị coi là keylogger | ⬜ Design opt-in (P2-2 §7) | MAC-043 + MAC-066 |

---

*Cập nhật trạng thái vào `docs/00-INDEX.md` (§2) khi đạt 2/2.*
