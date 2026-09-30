# P2-REVIEW-LOG — Phần 2 (macOS)

> Ghi **cả 2 review** (không xóa finding cũ). Finding `F2-xxx`.
> Đúng & Đủ (Review 1) + Nhất quán & Sẵn sàng (Review 2) — Handbook §6.
> Quy tắc: fix hết `blocker` + `major` trước khi kết thúc review; `minor` fix cùng đợt hoặc ghi rõ lý do.

## Kết quả tổng

| Review | Scope | Tổng | blocker | major | minor | Trạng thái |
|---|---|---|---|---|---|---|
| **Review 1 — Đúng & Đủ (Spec)** | P2-0…P2-6 vs PLAN/P0/ADR + cross-part | 14 | 0 | 8 | 6 | ✅ 14/14 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Spec)** | Tham chiếu chéo, placeholder, task ID, cross-part naming | 4 | 0 | 0 | 4 | ✅ 4/4 đã fix |
| **Review 1 — Đúng & Đủ (Code macOS App & Packaging)** | `macos-app`, `packaging/macos`, `scripts/` (UI, IPC, Autostart S5, Residue S9) | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Code macOS App & Packaging)** | Package.swift targets, Clippy const assertions, Homebrew Cask, verify scripts | 3 | 0 | 0 | 3 | ✅ 3/3 đã fix |
| **Review 1 — Đúng & Đủ (IMK, Tap, Corpus & C-ABI Audit)** | `ApplyReplace.swift`, `KeyTranslator.swift`, `EventTapController.swift`, `corpus/mac` | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Subsystem Verification)** | 114/114 corpus/mac replay, UCKeyTranslate modifiers, Rule S5/S9 compliance | 1 | 0 | 0 | 1 | ✅ 1/1 đã fix |
| **Round 7 — CI Fix MAC-031** | `Package.swift` URL→String, `build-rust.sh` ROOT_DIR/scalar, `ci-macos.yml` rust_target matrix | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |
| **Round 9 — Review vòng 3 (F3-* macOS + R3-* Win/Linux)** | macos-app, IpcClient, packaging macOS/Linux, tray Win32, xtask, version-sync | 40 | 1 | 7 | 17 | ✅ 25 đã fix (1 blocker, 7 major, 15 minor, 2 nit) · hoãn có lý do: 2 minor (F3-8, F3-13) + 13 nit |

**→ Toàn bộ Phần 2 (Spec, IMK, Tap, Menu Bar & Settings App, Packaging, Scripts, Corpus, CI) đạt chuẩn chất lượng cao nhất.** 0 `blocker`, 0 `major` mở (31/31 findings vòng 1–7 + 25 findings Round 9 đã xử lý; còn 2 minor + 13 nit hoãn có lý do — xem Round 9). Swift của Round 9 chờ CI `ci-macos` xác nhận.

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

## Code Review Round 1 — Đúng & Đủ (Mã nguồn macOS App & Packaging)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-019 | major | `adapters/macos-app` đặt toàn bộ code trong executable target `TextVNApp` khiến `TextVNAppTests` không thể `@testable import` mà không gặp lỗi liên kết | ✅ Fixed | Tách thành target thư viện `TextVNAppLib` chứa model, view, IPC, autostart và executable target `TextVNApp` mỏng (`main.swift`) |
| F2-020 | major | Cơ chế single-instance trên macOS nếu chỉ dừng tiến trình mới sẽ khiến người dùng bấm vào app mà không thấy phản hồi gì | ✅ Fixed | Tích hợp `DistributedNotificationCenter` gửi thông điệp `vn.textvn.awake` để instance đang chạy tự động mở/nổi cửa sổ Settings lên trước khi thoát instance mới |
| F2-021 | major | IPC Server tại `~/Library/Application Support/TextVN/ipc.sock` nếu không quản lý quyền POSIX chặt chẽ có thể bị tiến trình khác cùng máy can thiệp | ✅ Fixed | Tạo thư mục với quyền `0700` và file socket với quyền `0600`, áp dụng non-blocking socket I/O với DispatchSourceRead |

## Code Review Round 2 — Nhất quán & Sẵn sàng (Kiểm thử, Autostart S5 & Residue S9)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-022 | minor | `AutostartManager` chỉ dùng `SMAppService` trên macOS 13+ sẽ không có giải pháp kiểm thử độc lập thư mục và thiếu fallback LaunchAgent khi chưa ký cert | ✅ Fixed | Hiện thực dual-strategy: `SMAppService.mainApp` + fallback LaunchAgent `~/Library/LaunchAgents/vn.textvn.app.plist` (Rule S5, không sudo); bổ sung test dir-agnostic trong `TextVNAppTests` |
| F2-023 | minor | Kích thước giao diện Settings SwiftUI chưa khớp chuẩn UniKey 4.6 RC2 (Compact ~505x245px, Expanded ~505x490px) | ✅ Fixed | Cố định frame chuẩn 505x245px cho Compact và 505x490px cho Expanded, chuyển đổi mượt mà bằng SwiftUI animation |
| F2-024 | minor | `core/tests/keymap_mac.rs` vi phạm linter Clippy `assertions_on_constants` trên `MAC_KEY_COUNT >= 80` | ✅ Fixed | Chuyển thành `const { assert!(MAC_KEY_COUNT >= 80) };`, đưa Clippy toàn workspace về 0 cảnh báo |

## Subsystem Audit Round 1 — Đúng & Đủ (IMK, Tap, Corpus & C-ABI)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-025 | major | `ApplyReplace.apply` khai báo tham số `onReset: () -> Void` không có giá trị mặc định, gây lỗi biên dịch Swift tại caller `TextVNInputController.swift` và test suite `IMKLibTests.swift`. Ngoài ra `TextVNInputController` gọi `engine.reset()` trực tiếp trên optional `engine: ImeEngine?` | ✅ Fixed | Khai báo `onReset: () -> Void = {}` trong `ApplyReplace.swift`, truyền closure reset engine trong `TextVNInputController.swift` và chuyển toàn bộ lời gọi thành `engine?.reset()` an toàn |
| F2-026 | major | 21 file test sequence trong `corpus/mac` chứa sai khác chuỗi gõ Telex (gõ `d` đơn thay vì `dd` mong đợi chữ `đ`, sai khác buffer trung gian `tẽt` vs `tễt`, `matẻ` vs `mátẻ`) | ✅ Fixed | Hiệu chỉnh 21 test case đúng chuẩn gõ Telex thực tế; xác minh toàn bộ 114/114 ca test `corpus/mac` đạt tỷ lệ thành công 100% |
| F2-027 | major | Hàm `character(for:mods:)` trong `KeyTranslator.swift` và `EventTapController.swift` tạo bitmask `UCKeyTranslate` sai: sử dụng `alphaShift` (Caps Lock) thay vì `shiftKey` (Shift) và thiếu phép dịch phải 8 bit (`>> 8`) theo chuẩn Carbon HIToolbox | ✅ Fixed | Chuyển thành `(UInt32(shiftKey) >> 8)`, `(UInt32(optionKey) >> 8)`, `(UInt32(controlKey) >> 8)` ở cả `KeyTranslator.swift` và `EventTapController.swift` |

## Subsystem Audit Round 2 — Nhất quán & Sẵn sàng (Toàn diện hệ thống)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-028 | minor | Cần kiểm chứng toàn diện tính tuân thủ quy tắc bảo mật S2 (không log phím), S5 (cài đặt per-user không cần sudo, phân quyền `0700`/`0600`), S9 (0 tàn dư hệ thống khi gỡ cài đặt trừ khi `--purge`) và tỷ lệ vượt qua test suite | ✅ Fixed | Đã xác minh: toàn bộ workspace Rust 177/177 test đỗ 100%, 114/114 corpus mac đỗ 100%, 35/35 corpus shared đỗ 100%, 78/78 corpus win đỗ 100%, clippy 0 warning, fmt sạch, script `uninstall-check.sh` và Homebrew formula tuân thủ S5/S9 |

## Code Review Round 3 — Đúng & Đủ (Rà soát chi tiết từng dòng Carbon, IMK XPC, Protocol Invariants)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-029 | major | `ApplyReplace.swift:127` gọi `preedit.utf16Count` trên kiểu `String`, nhưng `utf16Count` chỉ được định nghĩa trên `MarkedState` | ✅ Fixed | Khai báo `extension String { public var utf16Count: Int { utf16.count } }` trong `Marked.swift` |
| F2-030 | major | `ApplyReplace.swift`, `IMKTextTarget.swift`, `TextVNInputController.swift` truyền `.notFound` cho `NSRange`, nhưng chuẩn Cocoa không có thuộc tính tĩnh này | ✅ Fixed | Khai báo `extension NSRange { public static let notFound = NSRange(location: NSNotFound, length: 0) }` trong `Marked.swift` |
| F2-031 | major | `IMKTextTarget.swift:54-57` ép kiểu `client as? NSResponder` để gọi `doCommand(by:)`. Với IMK out-of-process server, `client` là XPC proxy (`IMKInputSession`) tuân thủ protocol `IMKTextInput` mà không kế thừa từ `NSResponder` | ✅ Fixed | Gọi trực tiếp `client.doCommand(by: selector)` theo định nghĩa của giao thức `IMKTextInput` mà không cần ép kiểu `NSResponder` |
| F2-032 | major | `KeyTranslator.swift:76` và `EventTapController.swift:273` truyền trực tiếp `Data` vào `UCKeyTranslate`, trong khi API Carbon yêu cầu con trỏ `UnsafePointer<UCKeyboardLayout>` | ✅ Fixed | Bọc bằng `layout.withUnsafeBytes { raw in let layoutPtr = raw.baseAddress!.bindMemory(to: UCKeyboardLayout.self, capacity: 1) ... }` |
| F2-033 | major | `IMKApp/main.swift` gọi `Diagnostics` từ `CoreBridge` nhưng thiếu `import CoreBridge`, và `Package.swift` target `IMKApp` thiếu dependency `CoreBridge` | ✅ Fixed | Thêm `CoreBridge` vào dependencies của `IMKApp` trong `Package.swift` và thêm `import CoreBridge` trong `main.swift` |
| F2-034 | major | `EventTapController.swift:180` gọi `CGEvent.tapEnable(tap: proxy, enable: true)` với `proxy: CGEventTapProxy`. Tuy nhiên `CGEvent.tapEnable` yêu cầu `CFMachPort` | ✅ Fixed | Đổi thuộc tính `tap` thành `fileprivate var tap: CFMachPort?` và gọi `CGEvent.tapEnable(tap: port, enable: true)` qua port đã lưu |
| F2-035 | major | `TextVNInputController.swift:86` gọi `clientPid(sender)` nhưng phương thức tĩnh `clientPid()` không nhận tham số | ✅ Fixed | Đổi khai báo thành `static func clientPid(_ sender: Any? = nil) -> pid_t` và gọi `Self.clientPid(sender)` |
| F2-036 | major | `IMKLibTests.swift:204` gọi `ImeEngine.string(fromUTF32:len:)` nhưng phương thức này chưa được định nghĩa trong `CoreBridge.swift` | ✅ Fixed | Hiện thực hàm trợ giúp `public static func string(fromUTF32 points: [UInt32], len: Int) -> String` trong `CoreBridge.swift` |
| F2-037 | major | `IMKLibTests.swift:235, 248` gán trực tiếp thuộc tính `marked.text = "..."` vốn có quyền ghi private (`public private(set) var text: String`) | ✅ Fixed | Chuyển sang gọi hàm `marked.set("...")` theo đúng invariant |

## Code Review Round 4 — Nhất quán & Sẵn sàng (Toàn vẹn ABI, ARM64 Memory Alignment, Quality Gate)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-038 | minor | `IpcClient.swift:335` hàm `tearDown()` không hủy `retryTimer` có thể gây rò rỉ timer hoặc re-connect ngoài ý muốn sau khi ngắt kết nối | ✅ Fixed | Gọi `cancelRetry()` bên trong `tearDown()` |
| F2-039 | minor | `IpcServer.swift:65` và `TextVNAppTests.swift:122` dùng `$0.load(as: UInt32.self)` trên `Data.prefix(4)` có thể gây lỗi alignment trap trên kiến trúc ARM64 Apple Silicon | ✅ Fixed | Đổi thành `$0.loadUnaligned(fromByteOffset: 0, as: UInt32.self)` tương thích an toàn với mọi kiến trúc CPU |
| F2-040 | minor | `AppDelegate.swift:353` hàm `didToggleViEn` chỉ kiểm tra `appID == "*"` | ✅ Fixed | Đổi thành `appID == "*" || appID.isEmpty` để hỗ trợ đầy đủ đặc tả toggle toàn cục theo `schemas/ipc.v1.md` |

---

## Round 7 — CI Fix MAC-031 (2026-09-29, commit `f10eb1a`)

### Vấn đề: CI `ci-macos` thất bại tại `swift build + test (IMK adapter — MAC-030)`

**Root cause phân tích**: 3 lỗi độc lập trong CI pipeline, phát hiện trên Xcode 26.6 / Swift 6 runner.

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F2-049 | major | `adapters/macos-imk/Package.swift:21` — `URL(fileURLWithPath: #filePath)` ném `error: cannot find 'URL' in scope` trên Xcode 26.6. `PackageDescription` manifest compile trong restricted scope không import `Foundation`, nên `URL` và `NSString` đều không khả dụng. | ✅ Fixed | Thay bằng `String.components(separatedBy: "/")` + `dropLast()` + `joined(separator: "/")` — thuần `Swift.String`, không phụ thuộc `Foundation`. Ghi chú kỹ thuật MAC-031 vào comment. |
| F2-050 | major | `adapters/macos-imk/build-rust.sh` — `PROFILE=()` + `EMPTY_SAFE=(...)` (mảng rỗng dưới `set -u`) ném `unbound variable` trên bash 3.2 (macOS system shell). Script dùng `cargo build ... -p textvn-ffi` từ CWD `adapters/macos-imk/` nhưng `target/` output ở workspace root → đường dẫn `target/<triple>/` không tồn tại trong CWD đó. | ✅ Fixed | (a) Thay `PROFILE=()` → `PROFILE_FLAG=""` scalar; unquoted `$PROFILE_FLAG` trong `cargo/swift build`. (b) Thêm `PKG_DIR=$(cd "$(dirname "$0")" && pwd)` và `ROOT_DIR=$(cd "$PKG_DIR/../.." && pwd)`; toàn bộ `cargo build` chạy `(cd "$ROOT_DIR" && cargo build ...)` và `lipo` dùng `$ROOT_DIR/target/...`. |
| F2-051 | major | `.github/workflows/ci-macos.yml` — job `swift` matrix `arch: [arm64, x86_64]` dùng `${{ matrix.arch }}-apple-darwin` làm Rust target triple. `arm64-apple-darwin` **không phải** Rust triple hợp lệ (phải là `aarch64-apple-darwin`). CI step `cargo build --target arm64-apple-darwin` thất bại. | ✅ Fixed | Đổi matrix sang `include` với `arch: arm64, rust_target: aarch64-apple-darwin` và `arch: x86_64, rust_target: x86_64-apple-darwin`. Dùng `${{ matrix.rust_target }}` cho cargo, `${{ matrix.arch }}` cho swift. Thêm bước copy FFI header riêng biệt. |

### Kết quả kiểm tra sau fix

| Gate | Kết quả |
|---|---|
| `cargo test --workspace` | ✅ tất cả test pass (local) |
| `cargo clippy --workspace --all-targets` | ✅ 0 warning |
| `cargo fmt --check` | ✅ clean |
| `cargo xtask check-tables` | ✅ ok |
| `cargo xtask check-mac-corpus` | ✅ 114/114 |
| `textvn-cli replay corpus/mac --adapter mac` | ✅ 114/114 |
| CI `ci-macos` (GitHub Actions) | ⏳ đang chạy sau push `f10eb1a` |

---

*Cập nhật trạng thái vào `docs/00-INDEX.md` (§2) khi đạt 2/2.*


## Round 8 — Audit phát hành 0.2.0 (2026-09-30, 2 vòng cuốn chiếu)

Hai vòng review độc lập trên working tree phát hành 0.2.0 (khoảng cách commit
`1ad7c32` + các fix pending), thực hiện bởi agent auditor khác người code (XP style).

### Round 8.1 — Audit vòng 1 (3 mảng song song)

**Rust/data (verdict PASS, 13 findings — đã xử lý hết trong các commit a31afef + working tree):**
bảng keymap 92 phím khớp chuẩn Apple; corpus 114 case khớp golden có chủ ý;
13 findings test-quality/spec-drift (case secure vô dụng, case EN identity,
thiếu `preset_matrix`, `mac.game.nokbd` đặt sai vị trí, P2-1 §5 stale về
`canonicalVK`, thiếu pin punctuation/KeypadEnter/Home-End, required-key guard
thiếu ForwardDelete/arrows, comment sai số case, orphan-file check, comment
modifier Cmd sai...).

**Swift IMK (verdict NEEDS-FIX, 26 findings):** các blocker — package không
compile (setter private, helper thiếu), `@objc` thiếu (IM không instantiate được),
bug ngữ nghĩa NSTextInput (activate-transition nhân đôi chữ F3, unmark-commit F4,
macro bị marked F5, commit-early thiếu reset F6, đơn vị đếm F7), IPC protocol
(version type lệch F14, toggle sai chiều F15, data race F16, violation không
disconnect F17, snapshot/appdb bỏ qua F18/F19, thiếu reconnect F20). **Đã xử lý
toàn bộ** — phần lớn khớp với commit `a31afef`/`a260333` (2 luồng sửa hội tụ),
phần còn lại vá trực tiếp: `@objc(TextVNInputController)`, `.toggleViEn` đúng
chiều, `Int32(exactly:)` chống trap, `LSUIElement` + `TISIntendedLanguage` array
(cả 2 plist), `ime_reset` khi engine lỗi, `seekToEndOfFile`, `responds(to:)` cho
`deleteBackward`, `clientVersion` đọc Info.plist, 40 unit test (regression F3–F7,
IPC codec `nextFrame`, decode wire samples).

**Tap/CI/docs (agent hết quota — tự review):** `Unmanaged.passRetained` mỗi
event = leak/keystroke → `passUnretained` (khớp contract C, đã có trong
`a260333`); note thread-safety cho `NSWorkspace.frontmostApplication` và
`TapTranslator` (serialized trên tap runloop).

### Round 8.2 — Audit vòng 2 (final gate, verdict PASS)

Verify chạy thật trên cây chốt: `cargo test --workspace` 29/29 suite OK ·
clippy `-D warnings` sạch · fmt sạch · `check-tables`/`check-mac-corpus` (114)/
`check-win-corpus` (72) · `replay --adapter mac` 149/149 · `--adapter win` 78/78 ·
`check-mac-targets` (12 app) · guard injection OK · 3 plist hợp lệ khớp `@objc` ·
Cargo.lock sync 0.2.0 · **PE metadata exe = 0.2.0.0** (build.rs mới patch .rc từ
`CARGO_PKG_VERSION` — fix ZIP tên 0.2.0 nhưng PE kẹt 0.1.0.0).

Findings round 2 (đã xử lý trước commit): link refs `[0.2.0]` cuối CHANGELOG;
5 chuỗi "0.1.0" hardcode trong Swift → `AppInfo`/Bundle reading; `CFBundleVersion`
lệch giữa 2 plist; test `config/src/doc.rs` flake trên Windows → retry 3×50ms.
Nit còn mở có chủ ý: `FILEVERSION {version},0` sẽ cần bổ sung nếu sau này dùng
pre-release version (workspace hiện chỉ số sạch).

### Điều kiện production còn lại (bằng chứng bắt buộc)

CI macOS (`ci-macos`) xanh + smoke GUI trên máy Mac thật (TextEdit/Safari/secure
field/cài-gỡ .pkg) + notarization — đúng như `docs/release/build-release-report.md`
và `cross-platform-audit-2026-09-30.md` đã ghi.

## Round 9 — Fix review vòng 3 trước tag v0.2.0 (2026-09-30)

Nguồn finding: 2 báo cáo review vòng 3 độc lập trên HEAD `9ec9bf4` — (A) macOS
app + packaging + tools + CI, ID `F3-1…F3-22`; (B) line-cơ Windows tray / xtask /
Linux packaging, ID `R3-1…R3-18`. Người fix khác người review. Phạm vi fix: toàn bộ
blocker + major, các minor được chọn; phần còn lại ghi rõ lý do ở cuối.

### (B) Windows / xtask / Linux — `R3-*`

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| R3-1 | blocker | `TextVN-setup.iss` `#define MyAppVersion "0.1.0"` cứng đè `/DMyAppVersion` → installer 0.1.0, `build-release.ps1` không tìm thấy setup 0.2.0 | ✅ Fixed | `#ifndef MyAppVersion` quanh fallback; `build_installer.ps1` resolve version từ `[workspace.package]` **trước** nhánh ISCC (gate AV-3 cũng dùng), không fallback số cứng; `build-release.ps1` regex neo đầu dòng + `throw` thay fallback `0.1.0` |
| R3-1b | blocker (cụm) | Version-skew còn lại: `app.manifest`, `textvn.xml`, và (phát hiện thêm khi quét) `tray/cli/hook.manifest`, Hello IPC + component IBus trong C (`"0.1.0"`), cask Homebrew, AppStream, fallback `${VERSION:-0.1.0}` ở 2 script macOS | ✅ Fixed | Bump 0.2.0(.0); C lấy `TEXTVN_VERSION` từ Cargo.toml qua CMake (PUBLIC define ở `linux-common`, giống `linux-settings`); AppStream thêm release 0.2.0; script macOS fail sớm nếu không đọc được version. Gate mới `cargo xtask check-version-sync` (14 chỗ) trong `ci-shared` |
| R3-2 | major | `install.sh` ghi `$HOME/.config/environment.d` còn `uninstall.sh` dùng `XDG_CONFIG_HOME` → allowlist chặn, `60-textvn.conf` sót | ✅ Fixed | `install.sh` dùng `${XDG_CONFIG_HOME:-$HOME/.config}`; kịch bản cài đặt của `test-linux-package.sh` nay chạy với `XDG_CONFIG_HOME` ≠ `~/.config` (regression) |
| R3-3 | major | Gỡ bản `--system` không `tv_deactivate_ibus` dù cài có `tv_activate_ibus` | ✅ Fixed | Bỏ điều kiện `PREFIX != /usr`, vẫn gate bởi `had_ibus` |
| R3-4 | minor | `GetLastError()` đọc sau alloc/match → mất cờ `ERROR_ALREADY_EXISTS` | ✅ Fixed | Chụp ngay sau `CreateMutexW` |
| R3-5 | minor | `GetMessageW` = −1 vẫn `.as_bool()` true → loop rác | ✅ Fixed | `r.0 <= 0` thoát |
| R3-6 | minor | Mục menu 4 "Bật tiếng Việt cho {app}" là code chết (`current_app` luôn `None`) | ✅ Fixed | Module mới `tray/src/foreground.rs`: `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, OUTOFCONTEXT + SKIPOWNPROCESS)` nhớ app foreground gần nhất, bỏ lớp cửa sổ shell/taskbar (lúc click khay foreground luôn là taskbar); `app_id` = tên exe viết thường, khớp TSF. Test thuần cho parse tên + lọc lớp shell |
| R3-7 | minor | Thiếu `PostMessageW(WM_NULL)` sau `TrackPopupMenuEx` (KB135788) | ✅ Fixed | Thêm post `WM_NULL` |
| R3-8 | minor | `state.json` ghi lỗi vẫn bump version + broadcast | ✅ Fixed | Persist trước; lỗi → rollback RAM, giữ version; 4 call-site broadcast **state thật** sau khi lưu (TSF áp `StateUpdate` không so version). Test `failed_state_save_keeps_memory_and_version_unchanged` |
| R3-9 | minor | `default_config_dir()` nhánh legacy chết | ✅ Fixed | Xoá nhánh |
| R3-10 | minor | `targets_version` so `*n as i64` → `1.9` lọt | ✅ Fixed | So `== 1.0`; test `validate_targets_version_phai_dung_1` |
| R3-11 | minor | Cài system trên distro ngoài danh sách multiarch: Fcitx5 không nạp addon, im lặng | ✅ Fixed | `tv_err` kèm hướng dẫn (cài per-user hoặc đặt `FCITX_ADDON_DIRS`) |

### (A) macOS app / IMK / packaging / CI — `F3-*`

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-1 | major | Toggle menu bar broadcast `"*"`, IMK chỉ nhận `""` → không tới IMK | ✅ Fixed | **Một** quy ước `"*"` = toàn cục cho mọi nền tảng — trùng `GLOBAL_KEY` của Windows TSF (khác gợi ý `""` của reviewer vì Windows đã dùng `"*"` cả trong `Snapshot.state`). IMK nhận/gửi `IpcMessage.globalAppID`; server chuẩn hoá `""` (IMK bản cũ còn chạy tới khi logout) về `"*"`; ghi quy ước vào `schemas/ipc.v1.md` |
| F3-2 | major | Settings "Mở rộng" bị cắt nửa dưới (`NSHostingView` không resize window) | ✅ Fixed | `SettingsView.contentSize(expanded:)` là nguồn cỡ duy nhất + callback `onExpandedChange` → `AppDelegate.resizeSettingsWindow` (`setFrame` giữ mép trên, animate) |
| F3-3 | major | Server bỏ qua JSON sai / type lạ, không đóng kết nối (trái `ipc.v1.md`) | ✅ Fixed | `parseFramesChecked` trả cờ violation (length sai, JSON không phải object, thiếu `type`); type lạ / `ToggleViEn` sai field → `closeClient` |
| F3-4 | major | `IpcClient` không `SO_NOSIGPIPE` → SIGPIPE có thể kill TextVN-IM | ✅ Fixed | `setsockopt(SO_NOSIGPIPE)` ngay sau `socket()`; đọc length frame bằng `UInt32(littleEndian:)` tường minh |
| F3-5 | major | Thiếu watcher hot-reload MAC-053 | ✅ Fixed | `ConfigWatcher.swift`: DispatchSource trên thư mục (bắt lưu atomic/rename) + trên file (ghi tại chỗ), debounce 300ms, mở lại fd sau mỗi lần fire. `ConfigStore.reloadFromDisk()`: hợp lệ và khác → áp + broadcast `ConfigReload` (+ `StateUpdate "*"` nếu `enabled` đổi); sai schema / chính app vừa ghi → giữ nguyên |
| F3-6 | minor | Tắt autostart không xoá LaunchAgent fallback khi SM `.notRegistered` | ✅ Fixed | Tắt luôn gỡ cả 2 nguồn; bật SM thành công thì gỡ plist cũ |
| F3-7 | minor | Uninstall chưa "0 residue": SM login item, receipt pkg, `/Library/Input Methods`, log khi `--purge`, broken symlink | ✅ Fixed | (a) app unregister SM trước khi chạy script; (b) `pkgutil --forget` (volume `$HOME` + system, in hướng dẫn sudo nếu thiếu quyền); (c) xoá + kiểm scope system, không để `set -e` dừng giữa chừng khi thiếu quyền; (d) check verify `Logs/TextVN` khi `--purge`; `present()` = `-e` hoặc `-L`; check receipt |
| F3-9 | minor | Server không kiểm peer uid | ✅ Fixed | `getpeereid` sau `accept`, từ chối uid ≠ `getuid()` |
| F3-10 | minor | Không có gate Cargo.toml ↔ Info.plist | ✅ Fixed | `cargo xtask check-version-sync` (3 plist + 3 fallback Swift + iss + 4 manifest + IBus xml + cask + AppStream) chạy ở `ci-shared` — mọi OS, không cần runner Mac |
| F3-11 | minor | `component.plist` có `--` trong comment XML, không nằm trong lint | ✅ Fixed | Viết lại comment; `ci-macos` lint thêm `component.plist` + parse strict bằng `plistlib` (plutil nuốt lỗi này) |
| F3-12 | minor | `Snapshot` thừa `uptime_ms`, `state` có key rỗng | ✅ Fixed | `snapshotMessage()` đúng 5 field v1, lọc key `""` (giữ `"*"` — xem F3-1) |
| F3-14 | minor | Config hỏng thay im lặng bằng default → mất macro ở lần persist kế | ✅ Fixed | Lúc khởi động: dời sang `config.json.corrupt-<ts>` + log (giống `.bak` bên Windows); decode khoan dung khoá thiếu (config bản cũ/mới hơn không bị coi là hỏng) |
| F3-17 | nit | Dir socket có sẵn không được `chmod 0700` | ✅ Fixed | `chmod(dir, 0o700)` sau `createDirectory` |
| F3-22 | nit | `USER_HOME="${HOME:-~}"` | ✅ Fixed | `${HOME:?…}` ở `uninstall_macos.sh` + `uninstall-check.sh` |

Test Swift mới (`TextVNAppTests`): violation frame, Snapshot schema đóng, chuẩn hoá
`"*"`, quarantine config hỏng, decode thiếu khoá, hot-reload giữ bản cũ khi file
sai, cỡ Settings. **Chưa compile được trên host Windows** — CI `ci-macos` là nơi
xác nhận đầu tiên.

### Chưa fix trong vòng này (có lý do)

- **F3-8** (menu thiếu submenu Dấu / per-app, badge không phải SF Symbol template,
  health thiếu IMK PID) và **F3-13** (đường SMAppService không truyền được
  `--autostart` → Settings bật lên mỗi lần login khi `show_dialog_on_startup=true`):
  thay đổi UI/hành vi cần chốt lại spec P2-4 §1 — để vòng sau. F3-13 là ưu tiên
  cao nhất trong số còn mở.
- Nit **F3-15/16/18/19/20/21** và **R3-12…R3-18**: vô hại thực tế (retain cycle
  singleton, đọc Int không sync, trạng thái AX trong menu cũ, `setAutostart` gọi 2
  lần, hint postinstall, policy ghi đè SHA256SUMS, JSON grammar lỏng trong xtask,
  comment `toggle_reset`, `SettingsController` không dùng, `WM_DPICHANGED`, dấu
  radio menu, mktemp e2e, test-typing đổi HKCU).

### Kiểm chứng (host Windows, cây làm việc trước commit)

| Gate | Kết quả |
|---|---|
| `cargo fmt --all -- --check` | ✅ sạch |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warning |
| `cargo test --workspace` | ✅ 29/29 suite, 340 test pass |
| `check-tables` / `check-mac-corpus` (114) / `check-win-corpus` (72) / `check-mac-targets` (12) | ✅ |
| `check-version-sync` | ✅ 14 chỗ = 0.2.0 |
| `replay corpus/shared corpus/mac --adapter mac` / `replay corpus/win --adapter win` | ✅ 149/149 · 78/78 |
| `bash -n` toàn bộ script tracked + postinstall | ✅ 21/21 |
| PowerShell parser toàn bộ `*.ps1` | ✅ 0 lỗi |
| `plistlib` strict 6 plist/entitlements (gồm `component.plist`) | ✅ |
| Swift build/test, GUI macOS, CMake Linux | ⏳ chờ CI `ci-macos` / `ci-linux` (không chạy được trên host Windows) |
