# P3-8 — Đối chiếu lỗi của bộ gõ khác với TextVN Linux (2026-11-26/27)

> Phạm vi: toàn bộ issue công khai của **Ngó Sen** (`ngosen/ngosen`, 100 issue), **OpenKey**
> (`tuyenvm/OpenKey`, 288 issue), **ibus-bamboo** (`BambooEngine/ibus-bamboo`, 520 issue + PR
> #581/#611/#618), và **oracle x-unikey 1.0.4** (`ukengine` build thật — `docs/specs/oracle-unikey.md`
> §2 Cách D). Mốc mã: v0.2.91 (`52f28509`). Kết quả sửa: v0.2.92 (R2-165 … R2-172, E73 … E77).
> Vòng 2 (§7): Gõ Nhanh, fcitx5-lotus, xkey, goxkey, PHTV, fcitx5-unikey, ibus-unikey, VietType, vnkey,
> claude-code-vietnamese-fix — 1.250 issue.

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

## 7. Vòng 2 (2026-11-27) — Gõ Nhanh, fcitx5-lotus, xkey, goxkey, PHTV, fcitx5-unikey, ibus-unikey, VietType, vnkey, claude-code-vietnamese-fix

Phạm vi: toàn bộ issue mở + đóng của `khaphanspace/gonhanh.org` (258), `LotusInputMethod/fcitx5-lotus`
(241), `xmannv/xkey` (313), `huytd/goxkey` (98), `PhamHungTien/PHTV` (201), `fcitx/fcitx5-unikey` (40),
`vn-input/ibus-unikey` (25), `dinhngtu/VietType` (40 — TSF thuần như TextVN Windows), `marixdev/vnkey`
(22 — engine Rust), `manhit96/claude-code-vietnamese-fix` (12) — **1.250 issue**. Mọi ca "gõ X ra Y"
trong tiêu đề (≈ 160 ca) được chạy qua `textvn-cli replay` (cấu hình mặc định) và đối chiếu oracle
x-unikey 1.0.4 (§6).

### 7.1 Lỗi tương tự CÓ ở TextVN — đã sửa trong 0.2.92

| ID | Lỗi | Nguồn đối chiếu | Sửa | Chặn tái diễn |
|---|---|---|---|---|
| R2-171 | Ứng dụng GTK (gedit, GNOME Text Editor, xed, Mousepad…): gõ `ban` Enter `tooi` ra `tôi` thay vì `Tôi` — GtkTextView gọi `gtk_im_context_reset` ngay khi chèn dòng mới, IBus `reset` / Fcitx5 `InputContextReset` tới engine giữa Enter và chữ kế tiếp, `Engine::reset` xoá sạch đuôi văn bản nên mất mốc "đầu dòng" (3 OS dùng chung core) | fcitx5-lotus #597 (đúng nguyên nhân), xkey #303/#289, Gõ Nhanh #245/#302 | `Engine::reset` giữ mốc đầu dòng khi đuôi văn bản kết thúc bằng xuống dòng; reset giữa câu (click, đổi focus, sau `. `) vẫn xoá sạch, không đoán ngữ cảnh | `core/tests/r2_171_caps_after_enter_reset.rs` (6 ca: có/không reset, reset 2 lần, reset giữa câu, sau `. `, tắt auto-capitalize) |
| R2-172 | Không gõ được địa danh chính thức **Đắk Lắk**, **Đắk Mil**, **Đăk Glei**, **Krông Búk**, **Krông Pắc** — luật ghép vần không có âm cuối `k`, âm đầu `kr` ⇒ Space trả về `DDawks Lawks`, `Kroong` | xkey #135; x-unikey 1.0.4 chỉ gõ được `đắk` khi đặt dấu TRƯỚC `k` (`ddawsk`), `ddawks` ra `đăks` | âm đầu `kr` (`validate::ONSETS`); âm cuối `k` **chỉ** trong `đăk/lăk/buk` với thanh ngang/sắc (`validate::place_name_k_coda`) — dấu trước hay sau `k` đều được | `validate::tests::place_name_k_coda_and_kr_onset` (cả ca phải sai: `hăk dúk túk húk múk đặk krill`), `r2_172_english_words_untouched` (quét `en_words10k.txt`: `hawk`, `kruger`…), `corpus/shared/r2_172_dak_lak_01.keys` (4 adapter); đánh đổi có chủ đích: từ Anh hiếm `busk` thành `búk` |

### 7.2 KHÔNG xảy ra ở TextVN (đã kiểm bằng replay / mã)

| Lỗi của bộ gõ khác | Nguồn | TextVN |
|---|---|---|
| Lặp phụ âm/nguyên âm khi auto-restore tiếng Anh: `mos`→`moss`, `sims`→`simss`, `ses`→`sess`, `music`→`mussic`, `business`→`bussiness`, `console`→`conssole`, `momo`→`momoo`, `There`→`Therere`, `respect`→`respeect` | Gõ Nhanh #356 #142 #296 #193 #337 #131 #348 #230 #147 | `mó`, `sím`, `sé`, `music`, `business`, `console`, `momo`, `there`, `respect` (E50/E60 `UndoTracker` đòi liền kề) |
| Nhân đôi nguyên âm cách quãng biến từ Anh/viết tắt: `SATA`→`SATAA`, `TOTO`→`TÔT`, `MAMA`, `mama`→`mâm` | Gõ Nhanh #367, goxkey #123, x-unikey `mama`→`mâm` | giữ nguyên `SATA TETE TOTO MAMA mama coco` (§7.3) |
| Từ thường bị hỏng: `tieefn`→`tieên`, `bawngf/thawngf`, `xuaats`, `quowis`, `khoangr`→`khoan`, `nghieepj`, `Tuwowng`, `Giuwxa`→`Giuẵ`, `dduwowcj`, `muwa`, `Uwf`→`Ừ`, `hawngf`, `giusp`→`gíúp`, `cuwowis`, `huowng`, `chuaanr`, `Theer`, `theem`→`thèm`, `giowis` | Gõ Nhanh #272 #262 #259 #243 #200 #105 #99 #74 #14 #151 #122; xkey #196 #139 #150; goxkey #98 #14; PHTV #178 #145 #110 | ra đúng `tiền bằng thằng xuất quới khoảng nghiệp Tương Giữa được mưa Ừ hằng giúp cưới hương chuẩn Thể thêm giới` |
| Từ Anh bị gán dấu: `useEffect`, `daydream`, `fomo`, `datadog`, `datasuite`, `deadline`, `await`, `cursor`, `view`, `queue`, `year your our ear early their`, `wwork`, `int1234` | Gõ Nhanh #410 #402 #355 #319 #246 #51 #116 #115 #145; xkey #339 #141; PHTV #176 #132 | giữ nguyên tất cả |
| Caps Lock + phím `[` vẫn ra `ơ` thường | fcitx5-lotus #387 #383 | `TƠ` |
| Enter chèn ký tự `\n` vào ô đổi tên file thay vì kết thúc composition + gửi phím | VietType #43 | TSF kết thúc composition rồi để Enter đi tới app như phím thật (`KeyKind::native_boundary_char`) |
| Đổi focus giữa từ làm mất từ | VietType #42 (Run/Explorer rename), fcitx5-unikey #54 | TSF `OnSetFocus` commit composition (test vòng đời `compose.rs`); IBus preedit chế độ `IBUS_ENGINE_PREEDIT_COMMIT`; Fcitx5 FocusOut xử lý như fcitx5-unikey upstream (lỗi #54 nằm ở fcitx5 < 5.1.20 + mutter 50.1). Explorer rename/Run thật: nên kiểm tay mỗi bản |
| Click đổi chỗ con trỏ nhưng buffer engine không reset ⇒ in lại từ cũ | fcitx5-lotus #548 #9, fcitx5-unikey #53 | IBus/Fcitx5 `reset` ⇒ chốt từ + xoá trạng thái |
| So sánh lẫn offset UTF-8 ký tự với byte | fcitx5-lotus #436 | IBus đổi `cursor_pos` (ký tự) sang byte bằng vòng lặp riêng; Fcitx5 dùng API `SurroundingText` |
| Panic xuyên FFI khi mutex bị poison | vnkey #25 | FFI không giữ `Mutex`, mọi hàm bọc `catch_unwind` |
| Lớp lỗi gửi Backspace giả (Claude Code CLI, Ghostty, Firefox URL bar, Google Sheets, Excel nhảy chữ) | claude-code-vietnamese-fix, Gõ Nhanh #326/#261/#418, xkey #322, fcitx5-lotus uinput (≈ 120 issue) | TextVN không gửi Backspace giả trên cả 3 OS (TSF composition · IMK marked text · preedit) |

### 7.3 Khác biệt CHỦ ĐÍCH (chính sách, không sửa)

| Hành vi | Bộ gõ khác | TextVN | Lý do |
|---|---|---|---|
| Nhân đôi nguyên âm **cách quãng**: `neues`→`nếu`, `xauas`→`xấu`, `hieuer`→`hiểu`, `khongo`→`không` | x-unikey, Gõ Nhanh (#182/#183), VietType #28 | chỉ nhân đôi liền kề (`neeus`, `xaaus`, `hieeur`, `khoong`) | chính người dùng Gõ Nhanh báo lỗi ngược (#367 `SATA TETE TOTO MAMA`, #348 `momo`); x-unikey biến `mama` thành `mâm` |
| Chữ HOA gõ bằng **Shift** là chữ thường lệ (`NGUWOWIF`, `VIEETJ` giữ phím) | UniKey/OpenKey nhận dấu | Caps Lock cho hành vi UniKey (`VIỆT NAM ĐƯỜNG`) | R2-67: `USA`, `JSON`, `SWE` không bị biến đổi (user-guide §3, `telex_caps_lock_01`) |
| `w` đứng riêng + Space ⇒ `w` (auto-restore EN); trong từ vẫn `nhw`→`như`, `tw`→`tư` | x-unikey `ư` | giữ `w` | `w` một mình gần như luôn là chữ Latin |
| Backspace quay lại sửa từ đã chốt (`nhu ` ⌫ `w` ⇒ `như`) | fcitx5-lotus #366, OpenKey "phục hồi từ" | `nhuw` | E47: dựng lại chuỗi phím sau Backspace từng chèn phím lạ vào văn bản; giữ WYSIWYG |
| `[`/`]` ⇒ `ơ`/`ư` | fcitx5-lotus, xkey #70/#43 | như nhau (`t[`→`tơ`, Caps Lock `TƠ`) | — |
