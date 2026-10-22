# Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)

> Một bảng điều khiển, cùng tuỳ chọn, cùng nhãn, cùng thứ tự trên CẢ 3 nền tảng
> (0.2.14 — yêu cầu chủ repo "buộc phải giống nhau").
> Code: Windows `tray/src/settings_dialog.rs` (Win32 thuần) · Linux
> `adapters/linux-settings/src/settings_window.c` (GTK4) + `settings_model.c` (logic, không GTK)
> · macOS `adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift` (SwiftUI).
> Test `labels_match_linux_settings_panel` (tray) đối chiếu nhãn từng chữ giữa
> Windows ↔ Linux ↔ macOS trong CI.

## 1. Bố cục

```
┌ TextVN - Bảng điều khiển ─────────────────────────────────────────────┐
│ ┌ Điều khiển ───────────────────────────────────────────────────────┐ │
│ │ Bảng mã: [Unicode dựng sẵn ▾]          Kiểu gõ: [Telex ▾]          │ │
│ │ Phím chuyển:  Ctrl + Shift   (hoặc Ctrl + Shift + Space)          │ │
│ └───────────────────────────────────────────────────────────────────┘ │
│ ┌ Tùy chọn gõ ──────────────────────────────────────────────────────┐ │
│ │ [x] Bật gõ tiếng Việt                 (•) Dấu mới (hoà, thuỷ)      │ │
│ │ [x] Khôi phục từ tiếng Anh khi gõ sai ( ) Dấu cũ (hòa, thủy)       │ │
│ │ [x] Đặt dấu tự do                     [x] Tự viết hoa chữ đầu câu  │ │
│ │ [ ] Quick Telex (cc→ch, nn→ng…)       [ ] Gõ tắt cả khi tắt tiếng Việt │
│ └───────────────────────────────────────────────────────────────────┘ │
│ ┌ Hệ thống (chỉ Windows) ───────────────────────────────────────────┐ │
│ │ [ ] Khởi động cùng Windows            [x] Bật hội thoại này khi khởi động │
│ │ [x] Dành Ctrl + Shift cho TextVN (tắt phím đổi bàn phím của Windows)  │ │
│ └───────────────────────────────────────────────────────────────────┘ │
│ [Hướng dẫn] [Thông tin] [Gõ tắt...] [Từ điển EN...] [Cài & bật TSF]¹ [Mặc định] [Đóng] [Kết thúc]¹ │
└───────────────────────────────────────────────────────────────────────┘
```

¹ Chỉ Windows. Linux không có nhóm "Hệ thống": IBus/Fcitx5 tự chạy cùng phiên đăng nhập,
không có tiến trình khay riêng; bảng mở từ menu IBus/Fcitx5 ("Cài đặt TextVN…") hoặc
launcher ứng dụng. ² macOS dùng bố cục SwiftUI riêng nhưng **cùng nhãn + cùng thứ tự
tuỳ chọn**; các hàng riêng của macOS (`Khởi động cùng OS`, `Gõ không gạch chân
(Non-preedit)`, `Chạy ngầm trong menu bar`) nằm ở nhóm "Hệ thống" và không phá parity.

> Quy tắc cứng cho nhãn dài (sự cố "gõ sa" 2026-10-03): **nhãn checkbox/radio luôn
> chiếm trọn bề rộng cột** — không bao giờ thu hẹp để nhét thêm nút cùng hàng, vì ở
> màn hình DPI nhỏ chữ sẽ bị cắt. Nút phụ (Từ điển EN...) đặt ở HÀNG NÚT dưới.

## 2. Tuỳ chọn ↔ khoá cấu hình

| Nhãn | Khoá | Mặc định |
|---|---|---|
| Bảng mã: Unicode dựng sẵn / Unicode tổ hợp / TCVN3 (ABC) / VNI Windows | `config.output_charset` = `unicode_precomposed` / `unicode_decomposed` / `tcvn3` / `vni_windows` | Unicode dựng sẵn |
| Kiểu gõ: Telex / VNI / VIQR / Telex đơn giản | `config.method` = `telex` / `vni` / `viqr` / `simple_telex` | Telex |
| Bật gõ tiếng Việt | `state.json` `global_enabled` (Windows/Linux); macOS: trạng thái toàn cục trong AppDelegate (menu bar + SettingsView cùng một nguồn) | bật |
| Dấu mới / Dấu cũ | `config.diacritic_style` = `new` / `old` | Dấu mới |
| Khôi phục từ tiếng Anh khi gõ sai | `config.auto_restore_english` | bật |
| Đặt dấu tự do | `config.free_marking` | bật |
| Tự viết hoa chữ đầu câu | `config.auto_capitalize` | bật |
| Quick Telex | `config.quick_telex` | tắt |
| Gõ tắt cả khi tắt tiếng Việt | `config.allow_macro_when_vi_off` | tắt |
| Khởi động cùng Windows | `HKCU\…\Run\TextVN` | tắt |
| Bật hội thoại này khi khởi động | `config.show_dialog_on_startup` | bật |
| Dành Ctrl + Shift cho TextVN | `HKCU\Keyboard Layout\Toggle` (`Layout Hotkey`/`Language Hotkey` ≠ Ctrl + Shift) — cài đặt của Windows, không nằm trong config | bộ cài chọn sẵn |
| Gõ tắt... | `config.macros[]`, `config.macro_trigger` (`tab`/`space`) | trống, Tab |
| Từ điển EN... | `config.english_words[]` (chuẩn hoá trim/lowercase/a–z/dedupe — CÙNG quy tắc 3 nền tảng) | trống |

File: Windows `%APPDATA%\TextVN\`, macOS `~/Library/Application Support/TextVN/`, Linux `$XDG_CONFIG_HOME/TextVN/` (mặc định `~/.config/TextVN/`).

## 3. Hành vi chung

- **Lưu ngay** khi đổi (không có nút Áp dụng). IME đọc lại file ở phím kế tiếp (Linux: theo
  mtime; Windows: tray phát `ConfigReload`/`StateUpdate` qua IPC).
- **Vá từng khoá** trên file mới nhất (`textvn_config::SettingsDoc`, C API
  `ffi/include/textvn_settings.h`): giữ nguyên gõ tắt, emoji, `english_words` và khoá người
  dùng tự thêm; ghi nguyên tử (tmp → fsync → rename); file hỏng được giữ thành `.bak`.
- **Theo trạng thái ngoài**: bấm Ctrl+Shift khi bảng đang mở → ô "Bật gõ tiếng Việt" cập nhật
  (Windows: `WM_UPDATE_TRAY_STATE`; Linux: `GFileMonitor` trên thư mục cấu hình).
- **Mặc định**: mọi tuỳ chọn gõ về mặc định; **không** xoá bảng gõ tắt (như UniKey).
- **Bàn phím**: Tab/Shift+Tab di chuyển, Esc đóng cửa sổ.
- **Popup "Thông tin"** — CÙNG nội dung trên 3 nền tảng (0.2.32, test
  `about_text_matches_every_platform` của tray đối chiếu từng chữ):

  ```
  TextVN <phiên bản>
  Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)

  Phát triển bởi: LinhBH.CoM
  Giấy phép: GPL-3.0-or-later
  https://github.com/hunglinhpt/TextVN
  ```

  Windows `tray/src/settings_dialog.rs` (`ABOUT_TAGLINE`/`ABOUT_DETAILS`) · Linux
  `adapters/linux-settings/src/settings_model.c` (`tv_about_text`) · macOS
  `adapters/macos-app/Sources/TextVNAppLib/AppInfo.swift` (popup trong Settings và mục
  About của menu bar cùng đọc `AppInfo.aboutMessage`).

## 4. Cửa sổ Gõ tắt

- Ô văn bản, mỗi dòng `gõ tắt = nội dung`; dòng trống / bắt đầu bằng `#` bỏ qua;
  `\n` = xuống dòng, `\\` = dấu `\`. Định dạng dùng chung: `config/src/macro_text.rs`.
- Kiểm tra khi bấm **Lưu**: thiếu `=`, chữ tắt rỗng/có khoảng trắng/dài quá 32 ký tự,
  nội dung rỗng/dài quá 64 ký tự (giới hạn `ime_result_v1`), trùng chữ tắt (không phân biệt
  hoa thường). Lỗi → báo "Dòng N: …", bôi đen dòng sai, không lưu gì.
- "Bung gõ tắt bằng phím: (•) Tab ( ) Space" → `macro_trigger`.
- Gõ tắt khớp **đầu từ** (không bung giữa chữ `xcty`), và bị huỷ sau Home/End/phím điều
  hướng/Ctrl+V… để không bao giờ xoá nhầm chữ.
- "Gõ tắt cả khi tắt tiếng Việt": khi tắt VN, chữ đang gõ được giữ trong vùng soạn (gạch
  chân) tới hết từ để có thể bung gõ tắt — không xoá lùi chữ đã nằm trong ứng dụng.

## 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+)

- Mỗi dòng một từ tiếng Anh; chỉ chữ a–z; `#` = ghi chú; chuẩn hoá lowercase + khử
  trùng lặp khi lưu (cùng quy tắc ở `tray::normalize_word_list`,
  `config::SettingsDoc::set_english_words_text` — một hành vi cho cả 3 UI).
- Từ trong danh sách thắng mọi phỏng đoán engine (kể cả `cow`→`cơ`) — xem
  `docs/specs/language-detection.md` §2 cấp 6c.
- Windows: nút hàng dưới → hộp thoại; Linux: nút hàng dưới → cửa sổ GTK; macOS:
  nút cạnh "Gõ tắt..." → sheet. Lưu → broadcast reload ngay.

## 6. Nhận diện EN/VI (không có UI riêng — hành vi engine)

- Bật/tắt bằng checkbox "Khôi phục từ tiếng Anh khi gõ sai" (`auto_restore_english`).
- Tab gợi ý từ EN + Escape khôi phục: hành vi engine, không cần cấu hình.
- Quy tắc đầy đủ: `docs/specs/language-detection.md`.

## 7. Icon — một thiết kế cho 3 nền tảng

| Ngữ cảnh | Thiết kế | Nguồn |
|---|---|---|
| Icon ứng dụng | Badge bo góc chữ V (hồng #C2185B), gradient trắng→#FCE4EC | `resources/icons/textvn_v.svg` → PNG/ICO/ICNS |
| Trạng thái **V** (đang gõ tiếng Việt) | Badge chữ V, màu hồng–tím | Windows `textvn_v.ico`; Linux `textvn_v.svg/png`; macOS menu bar chữ "[V]" + tint `systemPink` |
| Trạng thái **E** (tiếng Anh) | Badge chữ E, màu xanh dương #0288D1 | Windows `textvn_e.ico`; Linux `textvn_e.svg/png`; macOS "[E]" + tint `systemBlue` |
| Lỗi | — | macOS tint `systemOrange` (riêng macOS) |

Sinh lại PNG/ICNS: `python scripts/generate_app_icons.py` (nguồn sự thật là 2 SVG).

## 8. Trợ lý ngữ pháp — panel cạnh con trỏ (Assist)

> Đặc tả **UI hiển thị** của Trợ lý ngữ pháp (Phần 4). Nguồn: `docs/50-assist/P4-0-MASTER-PLAN.md`
> §4.1–§4.5 + §6 (D4 · D11 · D12 · D13 · D30 · D38) · định dạng render chuẩn:
> `textvn-cli assist show` (D38) · code Windows: `adapters/windows-tsf/src/assist.rs` (`OverlayView`).
>
> **Trạng thái parity (cập nhật 2026-10-22, CI 7/7 xanh):** Windows ⏳ (overlay TSF/Mode 1 + Mode 2/A3, chờ verify GUI app thật) · macOS ⏳ (`AssistModel.swift` + IMK candidate + `Cmd+Z` undo, chờ verify máy thật) · Linux ⏳ (Fcitx5 và IBus đều đã nối + precondition D12 + undo, chờ verify máy thật) · **Settings cả 3 OS ✓** (khung "Trợ lý ngữ pháp" Win32/SwiftUI/GTK4 nhãn khớp nhau). Mặc định tắt hoàn toàn (`mode:"off"`, `auto_fix:"off"`).



### 8.1 Nội dung panel

| Thành phần | Nội dung | Ghi chú |
|---|---|---|
| Câu vừa gõ | Câu đang gõ (chỉ phần cuối, tối đa **96 ký tự Unicode**; có `…` nếu bị cắt) | Chỉ hiện ở **điểm hiển thị**: kết thúc câu `.` `!` `?` hoặc gọi lại bằng `Ctrl+Shift+G` (D11) |
| Lỗi gạch đỏ | Vạch `^` (render **màu đỏ**) đúng span lỗi trong câu | Tối đa **3 lỗi**; phần dư gộp `(+N lỗi khác)` |
| Gợi ý | **1–3 gợi ý** (mỗi lỗi 1 gợi ý) kèm **thông điệp tiếng Việt ngắn** (vd `Lặp từ: "của của"`) | Lỗi **chỉ gợi ý** (không auto-fix) vẫn hiện nhưng **không** tự sửa |
| Hàng phím | `↑/↓ chọn lỗi · Enter sửa · Alt+1…3 gợi ý · Esc đóng` | Chuỗi **tĩnh** — không chứa nội dung người dùng (S2) |

### 8.2 Phím

| Phím | Tác dụng |
|---|---|
| `↑` / `↓` | Chọn lỗi trước / kế (vòng tròn) |
| `Enter` | Áp dụng gợi ý của lỗi đang chọn (A2 — Backspace+Type + precondition D12) |
| `Alt+1`…`Alt+3` | Chọn **và** áp dụng gợi ý thứ `n` |
| `Esc` | Đóng panel, **không** sửa |
| `Ctrl+Shift+G` | Gọi lại (kiểm tra + hiện panel nếu còn lỗi) |
| Phím khác | Đóng panel rồi để phím đi tiếp (gõ tiếp = tự tắt) |

### 8.3 Hành vi chung (3 nền tảng)

- **Không chiếm focus**: panel hiện **cạnh con trỏ**, không kéo focus về app (D4/D30).
- **Tự tắt**: click ra ngoài · **8 s** không tương tác · app đổi focus · gõ tiếp · `Esc`.
- **Mode 2 — "tự sửa cả câu" (A3)**: chỉ luật **objective** được tự sửa; có **precondition D12** (caret ở cuối câu + câu do chính ta gõ) → thiếu precondition thì **không sửa, chỉ gợi ý**; hộp thoại "Đã sửa N lỗi" + `Ctrl+Z` (giữ **5 s**).
- **A11y/riêng tư**: nhãn tiếng Việt · bàn phím đầy đủ · dark mode · **không** đưa nội dung câu vào accessibility tree; **không** log/ghi file nội dung câu (chỉ số lỗi) — S2.
- **Điểm kiểm tra ≠ điểm hiển thị** (D11): `space` `,` `;` `:` chỉ kiểm tra **im lặng**, không hiện panel.

### 8.4 Checklist kiểm thử giao diện thủ công (Manual UI Verification Checklist)

| STT | Bước kiểm thử | Thao tác | Kết quả mong muốn | Windows (TSF) | macOS (IMK) | Linux (Fcitx5/IBus) |
|---|---|---|---|:---:|:---:|:---:|
| 1 | Cài đặt mặc định | Mở bảng Cài đặt | Nhóm "Trợ lý ngữ pháp": Chế độ = "Tắt", Tự động sửa = "Tắt" (làm mờ khi Chế độ = Tắt) | □ | □ | □ |
| 2 | Bật Mode 1 (Chỉ gợi ý) | Chọn Chế độ = "Chỉ gợi ý", lưu | Icon nhận cấu hình qua IPC; gõ câu có lỗi (`tôi tôi đi học.`) | Hiện popup/candidate list cạnh con trỏ với 1-3 lỗi; không tự sửa text | □ | □ | □ |
| 3 | Chọn gợi ý Mode 1 | Bấm phím chọn lỗi (1/Enter/click) | Thay thế lỗi bằng từ gợi ý qua Backspace+Type; caret ở vị trí hợp lệ | □ | □ | □ |
| 4 | Bật Mode 2 (Tự sửa lỗi) | Chọn Chế độ = "Tự sửa lỗi", Tự động sửa = "An toàn", lưu | Gõ câu có lỗi khách quan (`hôm nay tôi đi học .`) rồi bấm dấu chấm | Tự sửa thành `Hôm nay tôi đi học.`; hiện thông báo toast nhẹ | □ | □ | □ |
| 5 | Hoàn tác Undo Mode 2 | Bấm `Ctrl+Z` (Windows/Linux) hoặc `Cmd+Z` (macOS) trong 5 giây sau auto-fix | Khôi phục chính xác đuôi câu cũ trước khi sửa | □ | □ | □ |
| 6 | Hết hạn hoàn tác | Đợi quá 5 giây rồi bấm `Ctrl+Z` / `Cmd+Z` | Undo của ứng dụng hoạt động bình thường, không khôi phục text cũ | □ | □ | □ |
| 7 | Precondition D12 | Bôi đen hoặc di chuyển con trỏ trước khi gõ phím trigger | Không kích hoạt auto-fix; không làm xáo trộn text của ứng dụng | □ | □ | □ |
| 8 | Tắt hoàn toàn (Kill-switch) | Chọn Chế độ = "Tắt" trong Cài đặt hoặc menu tray | Mọi xử lý assist ngừng ngay lập tức, pass-through 100% | □ | □ | □ |
