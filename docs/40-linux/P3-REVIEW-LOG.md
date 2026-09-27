# P3-REVIEW-LOG — Phần 3 (Linux)

> Ghi **cả 2 review** (không xóa finding cũ). Finding `F3-xxx`.
> Đúng & Đủ (Review 1) + Nhất quán & Sẵn sàng (Review 2) — Handbook §6.
> Quy tắc: fix hết `blocker` + `major` trước khi kết thúc review; `minor` fix cùng đợt hoặc ghi rõ lý do.

## Kết quả tổng

| Review | Scope | Tổng | blocker | major | minor | Trạng thái |
|---|---|---|---|---|---|---|
| **Review 1 — Đúng & Đủ** | P3-0…P3-7 vs PLAN/P0/ADR + cross-part | 12 | 0 | 3 | 9 | ✅ 12/12 đã fix |
| **Review 2 — Nhất quán & Sẵn sàng** | Tham chiếu chéo, nhãn nhóm/milestone, enum schema, placeholder | 3 | 0 | 2 | 1 | ✅ 3/3 đã fix |
| **Review 3 — Đối chuẩn 3 Repo** | BambooMintKey, ibus-bamboo, OpenKey + UniKey UI Parity | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |
| **Review 4 — Tinh hoa Fcitx5 & Lotus** | fcitx/fcitx5, fcitx5-lotus, Non-preedit, Bug Prevention Matrix | 3 | 0 | 3 | 0 | ✅ 3/3 đã fix |

**→ Phần 3 đạt 4/4 review (đối chuẩn và phòng ngừa toàn diện).** 0 `blocker`/`major` mở.

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

## Review 3 — Đối chuẩn 3 Repo Tham chiếu (BambooMintKey, ibus-bamboo, OpenKey) & Parity UniKey

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-016 | major | Thiếu cơ chế cài đặt rootless không cần `sudo` (`~/.local/`), chỉ có `/usr/` trong spec đóng gói → user thông thường hoặc máy hạn chế quyền không cài được | ✅ Fixed | Bổ sung bảng đường dẫn Rootless `~/.local/` song song `/usr/` (`P3-0 §2.1`); bổ sung `scripts/install_linux.sh` và `scripts/uninstall_linux.sh` (LNX-054). |
| F3-017 | major | Thiếu phân tích kiến trúc lý giải vì sao mô hình hook của OpenKey sụp đổ trên Wayland và vì sao Fcitx5 vượt trội hơn IBus trên Wayland (`text-input-v3`) | ✅ Fixed | Bổ sung phân tích kiến trúc đối chuẩn 3 repo vào `P3-0 §2.1`, `P3-2 §2`, `P3-3 §1`. |
| F3-018 | major | Settings GTK4 chưa có chuẩn layout UniKey 4.6 RC2 Compact/Expanded đồng bộ với Windows và chưa có quy chuẩn SVG icon badges theo màu chuẩn (Image 3) | ✅ Fixed | Quy chuẩn hóa layout Compact (~505x245px) và Expanded (~505x490px) trong `P3-5 §3`, `P3-7 LNX-050/LNX-052`, icon `textvn_v.svg` (crimson/purple) và `textvn_e.svg` (vibrant blue). |

## Review 4 — Tinh hoa Fcitx5 & Lotus (Gõ Không Gạch Chân & Phòng chống Bug Lịch sử)

| ID | Mức | Finding | Trạng thái | Cách fix |
|---|---|---|---|---|
| F3-019 | major | Gạch chân preedit gây giật con trỏ và layout reflow trong Discord/LibreOffice/Web (Underline bug) | ✅ Fixed | Kế thừa giải pháp từ `fcitx5-lotus` & `fcitx5`: Thiết lập **Chế độ gõ không gạch chân (Non-preedit Mode)** dùng `CapabilityFlag::SurroundingText` + `deleteSurroundingText` xóa lùi trực tiếp, và Clean Preedit không gán `TextFormatFlag::Underline` (`P3-2 §5`, `P3-5 §3`). |
| F3-020 | major | Cần bảng tổng hợp phòng chống triệt để các bug lịch sử (B1, B2, B6, B8/B11, B10, B13) để đội ngũ triển khai không lặp lại sai lầm của các bộ gõ đi trước | ✅ Fixed | Xây dựng bảng **Bug Prevention Matrix** chi tiết tại `P3-0 §2.2`: Commit-before-hide cho Enter trong chat (B2), SelectionReplace không gửi Backspace vào thanh địa chỉ (B1), Early modifier filter cho phím hệ thống (B6), Multi-window instance isolation (B13). |
| F3-021 | major | Thiếu giải pháp cho ứng dụng không hỗ trợ bất kỳ IM protocol nào (Wine, game fullscreen) | ✅ Fixed | Kế thừa kỹ thuật `uinput` từ `fcitx5-lotus`: Bổ sung `packaging/linux/udev/99-textvn-uinput.rules` và tùy chọn kernel uinput trong Settings GTK4 / scripts cài đặt. |

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

---

*Cập nhật trạng thái vào `docs/00-INDEX.md` (§1) khi đạt 2/2.*
