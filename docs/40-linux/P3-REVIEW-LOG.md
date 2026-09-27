# P3-REVIEW-LOG — Phần 3 (Linux)

> Ghi **cả 2 review** (không xóa finding cũ). Finding `F3-xxx`.
> Đúng & Đủ (Review 1) + Nhất quán & Sẵn sàng (Review 2) — Handbook §6.
> Quy tắc: fix hết `blocker` + `major` trước khi kết thúc review; `minor` fix cùng đợt hoặc ghi rõ lý do.

## Kết quả tổng

| Review | Scope | Tổng | blocker | major | minor | Trạng thái |
|---|---|---|---|---|---|---|
| **Review 1 — Đúng & Đủ (Spec)** | P3-0…P3-7 vs PLAN/P0/ADR + cross-part | 12 | 0 | 3 | 9 | ✅ 12/12 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Spec)** | Tham chiếu chéo, nhãn nhóm/milestone, enum schema, placeholder | 3 | 0 | 2 | 1 | ✅ 3/3 đã fix |
| **Review 1 — Đúng & Đủ (Code Fcitx5)** | `linux-common` & `linux-fcitx5` (FFI P0-2, AT-SPI, IPC, S2, B2, B6, B13) | 2 | 0 | 1 | 1 | ✅ 2/2 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Code Fcitx5)** | Build/test matrix, naming, lifecycle leak (LNX-020), workspace check | 3 | 0 | 1 | 2 | ✅ 3/3 đã fix |
| **Review 1 — Đúng & Đủ (Code IBus)** | `linux-ibus` (Non-preedit surrounding fallback, FFI P0-2, caps, S2, B2, B6, S8) | 2 | 0 | 1 | 1 | ✅ 2/2 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Code IBus)** | Unit test `test_ibus_engine`, GObject finalize lifecycle, workspace verification | 1 | 0 | 0 | 1 | ✅ 1/1 đã fix |
| **Review 1 — Đúng & Đủ (Settings & Packaging)** | `linux-settings` & `scripts/` (Charset schema, JSON spaces, CMake builds, Fcitx5 IM) | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng (Settings & Packaging)** | Clippy doc-lazy, uninstaller residue, uninstall-check.sh, uinput cleanup | 3 | 0 | 0 | 3 | ✅ 3/3 đã fix |

**→ Toàn bộ Phần 3 (Spec, Fcitx5, IBus, Settings & Packaging) đạt chuẩn chất lượng cao nhất.** 0 `blocker`, 0 `major` mở.

---

## Review 1 — Đúng & Đủ

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-001 | major | `adr/README`: ADR-007 (IBus + Fcitx5 dual) vẫn `⬜ Proposed` nhưng P3-0…P3-3 dùng làm nền tảng — vi phạm "ADR Accepted trước khi task liên quan bắt đầu" | ✅ Fixed | ADR-007 → ✅ Accepted, tóm tắt (IBus=C primary, Fcitx5=C++, không grab Wayland, X11 opt-in) + `Chi tiết ở` trỏ 4 file P3 |
| F3-002 | minor | CJK lọt vào `P3-2 §10` (`symbol冲突`) | ✅ Fixed | → "Xung đột symbol / `-fno-exceptions`"; grep CJK lại = 0 |
| F3-003 | minor | `P3-1 §10` bước 1 ghi "Env + 8 spike" — A0 có **7** spike (002–008), 001 = env, 009 = corpus | ✅ Fixed | → "Env + 7 spike + corpus linux đầu" |
| F3-004 | minor | `P3-0 §4` ghi "Bảng spike 12 mục" — thật = 10 (`P3-1 §9`) + 8 (`P3-2 §8`) = **18** | ✅ Fixed | → "18 mục (10 + 8)" |
| F3-005 | minor | Dependency graph `P3-0 §5` sai range: `LNX-010..017` / `020..024` / `040..043` trong khi task tới 019/025/044 | ✅ Fixed | Sửa cả 3 range |
| F3-006 | major | `engine_owner` enum `P0-3 §2.1` thiếu `"x11"` (P3-3/P3-4 dùng `owner: x11`) | ✅ Fixed | Thêm `"x11"` vào enum + trỏ luật chống đôi `P3-3 §6.1` |
| F3-007 | major | `P0-1 §1` layout thiếu mục P3 tham chiếu: `adapters/linux-common/`, `adapters/linux-x11/` (crate `textvn-x11`), `tools/linux/`, `packaging/linux/` | ✅ Fixed | Bổ sung đủ 4 dòng |
| F3-008 | minor | `textvn purge` (`P3-5 §5`) không có trong bảng CLI `P0-1 §2` | ✅ Fixed | Bổ sung `purge` |
| F3-009 | minor | `P0-3 §1/§5` còn hedge "(chốt ở Phần 3)/(đề xuất, chốt ở P3)" — Phần 3 đã chốt đường dẫn | ✅ Fixed | Bỏ hedge, trỏ `P3-0 §2` |
| F3-010 | minor | `00-INDEX §4` dòng Linux còn "(chi tiết chốt ở Phần 3)" | ✅ Fixed | Trỏ `40-linux/P3-0 §2` |
| F3-011 | minor | `P3-6 §3` dòng #7 tên app sloppy ("GNOME Calculator? / …") | ✅ Fixed | → "Text Editor (2 window gõ xen kẽ)" |
| F3-012 | minor | `00-INDEX §4` thiếu dòng Binary Linux (`textvn-tray`, `textvn-x11`, `textvn-ibus-engine`, `libtextvn-fcitx5.so`) | ✅ Fixed | Thêm dòng |

## Review 2 — Nhất quán & Sẵn sàng

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-013 | major | `inject_mode` enum (`P0-3 §2.1`) không có `keycode_ascii` (P3-3 §5.4/P3-4 §3 dùng) + ghi "(chỉ adapter hook dùng)" nhưng hook/tap/x11 đều dùng | ✅ Fixed | Enum += `keycode_ascii` (trỏ `P3-3 §5.4`); sửa mô tả thành "adapter inject: hook/tap/x11" |
| F3-014 | major | `P3-7` đánh dấu nhóm task `L0–L6` **đụng namespace milestone `L0–L5`** của `P3-0 §4` (khác nội dung) + module X11 không xuất hiện trong bảng milestone | ✅ Fixed | Đổi nhãn nhóm → `T0–T6` + bảng ánh xạ T↔L ở đầu `P3-7`; `P3-0 §4` thêm "X11 fallback opt-in (LNX-040..044)" vào L2 |
| F3-015 | minor | `00-INDEX §4` dòng Config/Socket Linux tham chiếu thừa `30-macos/P2-0` | ✅ Fixed | Chỉ trỏ `40-linux/P3-0 §2` |

## Kiểm chứng sau fix (Review 2 cuối)

- [x] `grep "[CJK]" docs/` → 0 (trừ `越南` có chủ đích `P1-5 §2` + trích dẫn finding trong log).
- [x] Mọi `LNX-xxx` được tham chiếu trong P3-0…P3-6 đều có trong `P3-7-TASKS.md`
      (001–009, 010–019, 020–025, 030–035, 040–044, 050–056, 060–066 — 50 task).
- [x] `grep "^## [LT]\d"` trong `P3-7` → chỉ còn `T0…T6`; bảng ánh xạ T↔L có ở đầu file.
- [x] Tham chiếu `§` nội bộ P3-0…P3-7 đối chiếu đầu mục — không trỏ sai.
- [x] `engine_owner` (P0-3) phủ đủ giá trị dùng ở 3 phần: `tsf|hook` (P1), `imk|tap` (P2), `ibus|fcitx5|x11` (P3).
- [x] `inject_mode` (P0-3) phủ đủ: `unicode|vk_then_unicode|selection|keycode_ascii`.
- [x] `--adapter linux` hợp lệ (`P0-4`); `corpus/linux/` được tạo chỗ trong `P0-1`.
- [x] ADR-004 (egui Windows) + PLAN §4.1 (GTK4 Linux) → `P3-5` dùng GTK4 cùng `ui-model` JSON — không mâu thuẫn.

## Rủi ro còn mở của Phần 3 (theo dõi, không phải finding)

| # | Rủi ro | Trạng thái | Task xử lý |
|---|---|---|---|
| RL1 | keyval/shift semantics IBus | ⬜ Chưa chạy spike (tuần 1) | LNX-004 |
| RL2 | Fcitx5 API version pin | ⬜ Chưa chạy spike | LNX-006 |
| RL3 | Wayland không grab (B10) — giới hạn có chủ đích | ⬜ Design + document (P3-3 §4) | LNX-044 + P3-6 §6 Wayland verify |
| RL4 | AT-SPI/a11y bị từ chối | ⬜ Chưa verify | LNX-005 |
| RL5 | `delete_surrounding` hỗ trợ lệch theo toolkit | ⬜ Chưa verify (bảng spike) | LNX-004 → caps |
| RL6 | Đóng gói đa distro | ⬜ Design có sẵn | LNX-054/056 + ci-dist-matrix |
| RL7 | Engine crash giữa chừng | ⬜ Design có sẵn (commit-before-hide, soak) | LNX-016/020 + LNX-063 |
| RL8 | X11 module bị coi là keylogger | ⬜ Opt-in + doc quyền (P3-3 §7) | LNX-044 + LNX-066 |
| RL9 | Env vars sai (`GTK_IM_MODULE`…) | ⬜ Design có sẵn | LNX-035 (doctor) |
| RL10 | Link staticlib vào C/C++ addon | ⬜ Chưa verify | LNX-003 |

## Code Review Round 1 — Đúng & Đủ (Mã nguồn C/C++)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-016 | major | `adapters/linux-fcitx5/CMakeLists.txt` đặt `OUTPUT_NAME "textvn"` với `PREFIX ""` sinh `textvn.so`, trong khi `conf/textvn.conf.in` ghi `Library=libtextvn.so`, và `00-INDEX`/`P3-2 §3`/`uninstall_linux.sh` quy ước `libtextvn-fcitx5.so` → fcitx5 không nạp được thư viện | ✅ Fixed | Sửa `CMakeLists.txt` thành `OUTPUT_NAME "textvn-fcitx5"` (sinh `libtextvn-fcitx5.so`); cập nhật `conf/textvn.conf.in` thành `Library=libtextvn-fcitx5.so` |
| F3-020 | minor | `engine.cpp` khởi tạo `caps` chỉ bật `IME_CAP_PREEDIT` khi `hasSurrounding` là true; thực tế Fcitx5 engine luôn hỗ trợ Preedit độc lập với SurroundingText | ✅ Fixed | `caps` bật sẵn `IME_CAP_PREEDIT \| IME_CAP_FIELD_DETECT \| IME_CAP_SELECTION`, surrounding probe dùng trực tiếp trong `apply_result` |

| F3-021 | major | `adapters/linux-ibus/src/apply.c` chạy nhánh Non-preedit không kiểm tra `self->has_surrounding`. Trong app không hỗ trợ surrounding, `delete_surrounding` bị bỏ qua trong khi `commit_text` vẫn gọi → lặp ký tự (gõ `as` ra `aá`) | ✅ Fixed | Sửa điều kiện thành `if (self->non_preedit && self->has_surrounding)`; khi không có surrounding tự động fallback về Preedit có gạch chân chuẩn IBus; bổ sung unit test `test_apply_fallback_preedit_when_no_surrounding` |
| F3-022 | minor | `adapters/linux-ibus/src/engine.c` line 104 gán `IME_CAP_PREEDIT` phụ thuộc vào `self->has_surrounding`. IBus luôn hỗ trợ hiển thị Preedit độc lập với surrounding; việc tắt preedit khiến core strategy downgrade nhầm sang `BackspaceType` | ✅ Fixed | Gán mặc định `ctx.caps = IME_CAP_PREEDIT \| IME_CAP_FIELD_DETECT \| IME_CAP_SELECTION;` |
| F3-024 | major | `adapters/linux-settings/src/settings_window.h` & `settings_window.c` gán bảng mã thứ 4 là `TEXTVN_CHARSET_VIQR` thay vì `TEXTVN_CHARSET_UNICODE_DECOMPOSED` (Unicode tổ hợp), vi phạm schema `config.v1.schema.json` và enum `OutputCharset` của `textvn-config` | ✅ Fixed | Đổi enum thành `TEXTVN_CHARSET_UNICODE_DECOMPOSED`, ánh xạ `"unicode_decomposed"` và nhãn "4. Unicode tổ hợp" |
| F3-025 | major | `adapters/linux-settings/src/settings_window.c` hàm `textvn_settings_load_from_json` dùng `strstr` tìm key-value không có khoảng trắng (`"method":"vni"`), trong khi `to_json` và serde sinh chuẩn có khoảng trắng (`"method": "vni"`) → load luôn fallback về Telex/Unicode | ✅ Fixed | Viết helper `json_get_str_val` bóc tách chuỗi độc lập khoảng trắng / thụt dòng; kiểm chứng 100% qua `test_settings_window.c` |
| F3-026 | major | `scripts/install_linux.sh` gọi `cargo build --release --bin textvn --bin textvn-tray` (không tồn tại trong Cargo target), bỏ sót biên dịch CMake adapters (`textvn-ibus-engine`, `libtextvn-fcitx5.so`, `textvn-settings`), và không cài đặt Fcitx5 inputmethod config (`inputmethod/textvn.conf`) | ✅ Fixed | Cập nhật `cargo build --release --workspace`, thêm nhánh build CMake cho IBus/Fcitx5/GTK4 khi có dependencies, cài đặt đúng `textvn-cli` → `textvn`, `TextVN` → `textvn-tray`, `textvn-settings`, `libtextvn-fcitx5.so`, `textvn-ibus-engine`, Fcitx5 addon + IM descriptors, và systemd user service |

## Code Review Round 2 — Nhất quán & Sẵn sàng (Mã nguồn & Kiểm thử)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-017 | major | `TextVNEngine::destroyContext` được viết nhưng không bao giờ đăng ký lắng nghe sự kiện hủy của `InputContext` (`InputContext::Destroyed`) → rò rỉ bộ nhớ và `ime_instance` khi mở/đóng app liên tục (vi phạm DoD `LNX-020`) | ✅ Fixed | Đăng ký callback `ic->connect<fcitx::InputContext::Destroyed>` ngay khi tạo context trong `getOrCreateContext()`; bổ sung mock signal và unit test `test_lifecycle_context_destroy()` kiểm chứng 0 leak |
| F3-018 | minor | `CMakeLists.txt` cài đặt file cấu hình `conf/textvn-addon.conf.in` vào thư mục `inputmethod/` dưới tên `textvn-addon.conf` thay vì `textvn.conf` theo định danh sub-IM | ✅ Fixed | Đổi tên file build thành `conf/textvn-im.conf` và chỉ định `RENAME textvn.conf` khi install vào `${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod` |
| F3-019 | minor | `adapters/windows-hook/Cargo.toml` thiếu feature `Win32_Security` khiến `CreateMutexW` không tìm thấy trong scope khi build toàn workspace | ✅ Fixed | Bổ sung `"Win32_Security"` vào `windows` dependencies của `textvn-win-hook` |
| F3-023 | minor | `adapters/linux-ibus/tests/ibus_mock.h` `g_object_unref` chỉ gọi `free(p)` không gọi `finalize` class method → test phải giải phóng thủ công `self->inst` thay vì kiểm chứng vòng đời GObject tự động dọn rác | ✅ Fixed | Bổ sung `s_mock_finalize` hook vào `g_object_unref` và `G_DEFINE_TYPE`; cập nhật toàn bộ test trong `test_ibus_engine.c` gọi `g_object_unref(e)` |
| F3-027 | minor | `tray/src/autostart.rs` vi phạm linter `clippy::doc-lazy-continuation` trên ghi chú Rule S5, gây chặn `-D warnings` khi build workspace | ✅ Fixed | Chèn dòng trống phân cách đoạn doc và chạy `cargo fmt` chuẩn hóa |
| F3-028 | minor | `scripts/uninstall_linux.sh` thiếu xóa `textvn-settings`, `inputmethod/textvn.conf`, `textvn.desktop`, `textvn-tray.service`; thiếu script kiểm chứng LNX-054 | ✅ Fixed | Bổ sung đủ danh mục file gỡ sạch trong `scripts/uninstall_linux.sh`; tạo script `packaging/linux/uninstall-check.sh` xác thực 0 residue (LNX-054) |
| F3-029 | minor | `packaging/linux/udev/99-textvn-uinput.rules` là file rỗng 0-byte tồn đọng sau khi loại bỏ uinput grabbing theo ADR-007 (zero key grabbing trên Wayland) | ✅ Fixed | Xóa file rỗng và thư mục `udev/` khỏi repo để đảm bảo vệ sinh mã nguồn |

---

*Cập nhật trạng thái vào `docs/00-INDEX.md` (§1) khi đạt 2/2.*
