# P3-8 — Đối chiếu lỗi của bộ gõ khác với TextVN Linux (2026-11-26/27)

> Phạm vi: toàn bộ issue công khai của **Ngó Sen** (`ngosen/ngosen`, 100 issue), **OpenKey**
> (`tuyenvm/OpenKey`, 288 issue), **ibus-bamboo** (`BambooEngine/ibus-bamboo`, 520 issue + PR
> #581/#611/#618), và **oracle x-unikey 1.0.4** (`ukengine` build thật — `docs/specs/oracle-unikey.md`
> §2 Cách D). Mốc mã: v0.2.91 (`52f28509`). Kết quả sửa: v0.2.92 (R2-165 … R2-170, E73 … E75).

## 1. Khác biệt kiến trúc quyết định phần lớn kết luận

| Bộ gõ | Cách đưa chữ vào app trên Linux | Hệ quả |
|---|---|---|
| Ngó Sen "Gõ Sen" | gửi thẳng chữ, sửa bằng **forward BackSpace** + commit | dính mọi lỗi thứ tự phím/commit của GTK, mutter, Chromium (#1012, #1013, #225, #227, #569, #206, #205, #72, #40, #516, #1008) |
| ibus-bamboo (mặc định) | **preedit** (gạch chân), các chế độ phụ dùng surrounding/forward | chung lớp lỗi preedit với TextVN |
| OpenKey | chỉ macOS/Windows (hook) — không có bản Linux | chỉ đối chiếu engine |
| **TextVN** | **preedit** cho từ đang gõ; `delete_surrounding` chỉ cho trợ lý/sửa câu/Ctrl+Z/chuyển mã; **không bao giờ** forward BackSpace (chỉ `Left` cho mẫu Math) | không dính lớp lỗi forward-BackSpace; dính lớp lỗi "nhiều commit/phím" và "thứ tự preedit/commit" (đã sửa, §2) |

## 2. Lỗi tương tự CÓ ở TextVN — đã sửa trong 0.2.92

| ID | Lỗi | Nguồn đối chiếu | Sửa | Chặn tái diễn |
|---|---|---|---|---|
| R2-165 | Một phím gửi nhiều commit + xoá (trợ lý tự sửa câu: commit từ + commit dấu + xoá + commit bản sửa; Ctrl+Space sửa câu). text-input-v3 chỉ giữ lệnh cuối mỗi `done`, mutter gộp lệnh (`defer_done`) ⇒ **mất chữ / xoá lẹm** ("Xin chào. Tôi tôi đi học." → "Xin chTôi đi học.") | Ngó Sen #1019, #207, #1016; ibus#2979 | `adapters/linux-common/{include/lc_edit_batch.h,src/edit_batch.c}` dùng chung IBus + Fcitx5: mọi phím gửi **tối đa 1 xoá + 1 commit** | `test_edit_batch` (mô phỏng mutter gộp), e2e IBus/Fcitx5 bất biến "≤1 commit, ≤1 xoá mỗi phím" cả phiên |
| R2-166 | Mẫu Math dời con trỏ bằng `Left` gửi lại **chỉ có nhấn, keycode 0** ⇒ GNOME coi phím đang giữ (app tự lặp, lần sau bị bỏ), Wayland không dựng được `wl_keyboard.key`; mutter 50.0–50.3 (Ubuntu 26.04) bỏ mọi phím gửi lại | Ngó Sen #1017, #1020; ibus-bamboo ForwardKeyEvent | gửi **nhấn + nhả** kèm keycode thật (evdev 105 / XKB 113), commit đã gom tới trước; `textvn-cli --diagnose` cảnh báo `W_MUTTER_FORWARD_KEY_BUG` | e2e "press == release, keycode ≠ 0"; `doctor::r2_166_mutter_forward_key_bug_range` |
| R2-167 | Chốt từ: **ẩn preedit TRƯỚC rồi mới commit** ⇒ Chrome/Electron huỷ composition rồi chèn riêng ⇒ Messenger/Instagram/GitHub **lặp từ cuối sau Enter**, hoặc đẩy từ lên đầu câu | ibus-bamboo #574, #589, #560, #447, #250 (sửa ở PR #581); fcitx5-unikey cùng thứ tự đúng | commit trước, đóng preedit sau (IBus `commit_text`, Fcitx5 `commitText`), giữ đúng khi đang gom | e2e "lúc commit tới, preedit cũ vẫn còn" (IBus + Fcitx5) |
| R2-168 | `Sose2026` → `Sóe2026` (chữ số dính liền sau từ không có thật) | Ngó Sen #613; x-unikey `autoNonVnRestore` cho `Sose2026` | chữ số dính liền + từ đã biến đổi + **không có trong `vn_syllables.txt`** ⇒ trả phím; từ thật giữ (`cos2` → `có2`, `lớp12`) | `core/tests/r2_168_unikey_oracle_linux.rs`, `corpus/shared/r2_168_unikey_oracle_01.keys` |
| R2-169 | `booots` → `boót` (hoàn tác `ooo` rồi dấu thanh) | Ngó Sen #959; x-unikey `boots`; OpenKey #41 (`Thoòng`, `voọc`) | **luật ghép vần**: vần `oo` chỉ khép bằng `ng`/`c` (`validate::oo_coda_ok`) — giữ `xoong boong goòng moóc soóc Thoòng voọc`; dữ liệu tần suất bỏ 3 từ Anh lẫn vào (`too moon boom`) | `validate::tests::oo_nucleus_needs_ng_or_c` + corpus trên (E75: bản nháp dùng từ điển đã làm hỏng `Thoòng`/`voọc`) |
| R2-170 | `dzũng`, `dzô`, `Dzũng` bị trả về `dzuxng` | Ngó Sen #958; x-unikey gõ được | âm đầu `dz` trong `validate::ONSETS` | oracle test + corpus |

## 3. KHÔNG xảy ra ở TextVN (đã kiểm bằng mã + test)

- Toàn bộ lớp lỗi forward BackSpace của Ngó Sen (§1) và ibus-bamboo chế độ ForwardKeyEvent/backspace giả.
- Enter trong preedit phải bấm 2 lần (Ngó Sen #439, ibus-bamboo #320): Enter vừa chốt từ vừa tới app (`lc_plan_pass_through`, e2e "Enter commit (B2)").
- Super + phím bị nuốt trong preedit (Ngó Sen #391, ibus-bamboo #580/#106): chord có Super chốt từ rồi đi thẳng.
- Phát hiện app qua `org.gnome.Shell.Eval` hỏng trên GNOME 50 (ibus-bamboo #616): TextVN không dùng Shell.Eval.
- Gỡ cài để lại `XMODIFIERS` (Ngó Sen #807): `uninstall.sh` xoá `environment.d/60-textvn.conf`.
- Engine (423/423 từ thông dụng `core/tests/common_words.rs` khớp; x-unikey chỉ lệch `huơ`): `dduoxwcj`, `looxi`, `DD`, `huychj`, `quowrn`, `hoongf`, `ddutws`, `SDDT`, `[Toongr` (x-unikey hỏng: `oTongr`), `(hinhf`, `Next` (x-unikey `Nẽt`), `official`/`necessary` (x-unikey `oficial`), `text#primary`/`test1.c` (ibus-bamboo #429), `JWWT` (ibus-bamboo #267), VNI `dui9 dieu926 nua7 chua75 tri3e6n d9uo75c hu71o7ng dau96 hieu63 tua62n` (OpenKey #58–#79).

## 4. Khác biệt CHỦ ĐÍCH (chính sách, không sửa)

| Ca | TextVN | x-unikey / người báo | Lý do |
|---|---|---|---|
| `teens␣` | `teens` | `tến` (Ngó Sen #989) | tự khôi phục từ tiếng Anh (`auto_restore_english`, tắt được) |
| `uow`, `huow`, `khuow` | `uơ`, `huơ`, `khuơ` | x-unikey `ươ hươ khươ` | vần mở viết `uơ` đúng chính tả (OpenKey #229/#133 phàn nàn đúng lỗi x-unikey) |
| `cuaa` | `cuâ` | OpenKey #312 muốn `cuaa` | giống UniKey/x-unikey |
| `zaj`, `Zawngf` | trả phím | x-unikey `zạ`, `Zằng` (OpenKey #327, Ngó Sen #701) | `z` đầu từ chưa bật: dễ phá từ Anh (`zoom` → `zôm`); ứng viên tuỳ chọn teencode tắt mặc định |
| `dduowjcd␣` | `duowjcd` | x-unikey `dượcd` | `d` thứ ba gỡ `đ` như UniKey rồi khôi phục chuỗi phím hiệu lực |
| `chaiiif` | hiển thị `chaìii`, Space trả `chaiiif` | x-unikey giữ `chaiiif` ngay | gõ kéo dài (Ngó Sen #852/#898) là tính năng riêng, chưa hỗ trợ |

## 5. Giới hạn đã biết của nền tảng (ghi vào `docs/TROUBLESHOOTING.md` §4)

| Hiện tượng | Nguồn | Trạng thái |
|---|---|---|
| Thanh địa chỉ Chrome (Wayland) không tự điền URL khi đang có preedit: gõ `face` + Enter ra tìm kiếm | ibus-bamboo #619 | giới hạn preedit của Chromium; cách tránh: Space/→ trước Enter, hoặc Ctrl+Shift sang EN |
| Terminal: `sudo` (echo tắt) vẫn hiện preedit ⇒ lộ mật khẩu trên màn hình khi đang bật tiếng Việt | ibus-bamboo #7/#441, Ngó Sen #106 | terminal không báo ô mật khẩu cho bộ gõ; khuyên Ctrl+Shift sang EN (terminal ở EN nhận phím thẳng) |
| vim chế độ thường trong terminal: `dd` thành `đ` | Ngó Sen #302/#587 | chung mọi bộ gõ; tắt tiếng Việt khi điều khiển vim |
| Click chuột / đổi ô khi còn preedit: từ có thể theo con trỏ sang chỗ mới (LibreOffice Qt không báo đổi ô) | ibus-bamboo #385/#364/#292/#607; Ngó Sen #351; LibreOffice bug 173064 | phụ thuộc toolkit (reset bất đồng bộ); cần test máy thật |
| Chuyển mã vùng bôi đen (Ctrl+Shift+F9) không chạy với GTK3/Firefox trên Wayland | Ngó Sen #1014 (GTK3 `imwayland.c` luôn gửi anchor = cursor) | lỗi upstream |
| Chrome/Electron qua GTK IM (X11) và trên Sway không báo surrounding ⇒ trợ lý sửa câu tự tắt êm | Ngó Sen #220, #1022 | fail-safe D12 |
| Sway bỏ commit khi serial trễ | Ngó Sen #1023 (wlroots#4120) | lỗi upstream |
| Steam/XIM không chuyển phím cho bộ gõ | Ngó Sen #678 | chạy Steam với `LC_CTYPE=zh_CN.UTF-8` |

## 6. Cách tái hiện oracle (G4)

`ukengine` + `vnconv` + `byteio` + `ukinterface/unikey.cpp` của x-unikey 1.0.4 build bằng MSVC trên máy
dev Windows (không cần Linux): `cl /EHsc /utf-8 /permissive /Zc:strictStrings- /D_CONST_RETURN=
/DUNIKEYHOOK …` + harness gọi `UnikeySetup` → `UnikeySetOutputCharset(CONV_CHARSET_XUTF8)` →
`UnikeyFilter` (áp `UnikeyBackspaces` theo ký tự, phím không đổi gì thì app tự chèn như `xim.c`).
Chi tiết: `docs/specs/verified-ops.md`. Mã x-unikey KHÔNG được chép vào repo (chỉ dùng sinh kỳ vọng).
