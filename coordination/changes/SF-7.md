<!-- TASK: SF-7 · commit: c10330cc · agent: claude-opus-7e41b2d9 -->
## VI
- **SF-7 phần 1 — MỘT máy trạng thái bộ chọn Emoji dùng chung (P0-6 §5, G22)**: `session/src/emoji.rs`
  (`textvn_session::emoji::EmojiPicker`) thay 3 bản lệch nhau (`tray/src/emoji_popup.rs`,
  `adapters/linux-common/src/emoji_picker.c`, `adapters/macos-imk/Sources/IMKLib/EmojiPickerPanel.swift`):
  mở/đóng, ô tìm, 8 nhóm (Tab/Shift+Tab xoay vòng), chọn bằng mũi tên, cuộn, chọn nhanh `1..9`, chèn + ghi
  MRU. Tìm/xếp hạng vẫn là MỘT nguồn `textvn_core::post::emoji_catalog::search_emojis`.
- **Bố cục là năng lực adapter** (`PickerLayout`): Linux = danh sách ứng viên native (`LIST`: 1 cột, 9 dòng,
  36 mục); Windows/macOS = lưới (`GRID`: 8×5, 120 mục). Trái/Phải = ±1, Lên/Xuống = ±`cols` ⇒ với 1 cột trùng
  đúng hành vi danh sách Linux cũ.
- **Hợp nhất hành vi** (trước đây mỗi OS một kiểu): ô tìm tối đa **32 ký tự** (macOS 32 ký tự, Linux 64
  byte); số `1..9` chọn nhanh trên phần **đang hiện** khi ô tìm rỗng (Linux có, macOS từng coi số là chữ tìm);
  `0` và số sau khi đã gõ chữ là chữ tìm; Enter khi không có kết quả ⇒ đóng; Backspace khi ô rỗng ⇒ không đổi.
- **Windows tray dùng trực tiếp**: `TrayEmojiState` chỉ còn bọc `EmojiPicker` + ô hover (chuyện vẽ Win32);
  `GRID_COLS`/`GRID_VISIBLE_ROWS` lấy từ `PickerLayout::GRID` (một nguồn). Hành vi tray giữ nguyên (120 kết
  quả, MRU trong RAM, cuộn kẹp biên).
- **G8**: không log; đóng bộ chọn là xoá ô tìm; MRU chỉ giữ chỉ số `CATALOG` trong RAM.
- **Phần 1b — MRU tách khỏi bộ chọn (`EmojiSource`)**: `Engine::emoji_record_use` vừa ghi MRU dùng chung
  với gõ tắt `:shortcode:` vừa đếm **Dream RSI** (`accepts` + `chars_saved`, G19). Bộ chọn tự giữ MRU riêng
  sẽ làm Linux/macOS (phần 2) mất cả hai ⇒ MRU nay là nguồn truyền vào mỗi lời gọi: `impl EmojiSource for
  textvn_core::Engine` (tiến trình có engine) và `for EmojiMru` (tray Windows, không có engine — giữ nguyên
  hành vi cũ). Test `sf7_nguon_engine_dem_dream_rsi_va_mru_chung`; đột biến bỏ `emoji_record_use` ⇒ ĐỎ.
- **Phần 1c — danh sách Linux phân trang**: `PickerLayout::paged` (LIST = `true`): cửa sổ nhảy theo trang 9 mục
  (`selected / 9 * 9`) như danh sách ứng viên native IBus/Fcitx5 đang hiển thị, nên chọn nhanh `1..9` khớp trang
  người dùng thấy; lưới (`GRID`) vẫn trượt từng hàng. Test `sf7_danh_sach_phan_trang_va_chon_nhanh_theo_trang`.
- **Ghi nhận (có từ trước, không do SF-7)**: chọn emoji ở tray Windows KHÔNG được đếm Dream RSI vì tray không
  có engine (MRU chỉ trong RAM của tray). Phần 2 nên gửi bộ đếm qua IPC `StyleUpdate` (đường `RsiOnly` đã có).
- **Phần 2a — C ABI + Linux dùng bản chung**: `ffi/src/emoji_picker_abi.rs` xuất 9 hàm `ime_emoji_picker_*`
  (new/free/open/close/switch_category/scroll/key/pick/view) + `ime_emoji_view_v1` (ảnh chụp để vẽ: nhóm, ô tìm,
  mục chọn, phần đang hiện ≤ 64 mục, glyph/shortcode 32 byte) + `ime_emoji_item_v1`; nguồn tìm/MRU là engine của
  `ime_instance` (MRU chung `:shortcode:` + Dream RSI). `pick` kiểm chỗ bộ đệm TRƯỚC khi ghi MRU (thiếu chỗ không đếm
  Dream RSI mà không chèn). `adapters/linux-common/src/emoji_picker.c` bỏ toàn bộ máy trạng thái riêng (ô tìm, Tab,
  mũi tên, chọn nhanh, tải kết quả) — chỉ còn ánh xạ phím Linux ⇒ `IME_EPK_KEY_*`, chép view vào `lc_emoji_picker`
  cho IBus/Fcitx5 vẽ (API `lc_emoji_*` giữ nguyên, engine.c/engine.cpp KHÔNG đổi) và dòng aux; bộ chọn dùng chung
  chỉ sống khi bảng mở (giải phóng ở mọi đường đóng). `textvn-cli verify`: 46 export, 20 hằng, 2 struct, sizeof/offset.
- **Phần 2b (sau)**: macOS `EmojiPickerModel`/`EmojiPickerPanel` gọi C ABI (cần vòng ci-macos — máy dev không có
  Swift); rồi chuyển nhãn/biểu tượng nhóm vào `data/ui/strings.toml` (SF-9).

## EN
- **SF-7 part 1 — ONE shared Emoji picker state machine (P0-6 §5, G22)**: `session/src/emoji.rs`
  (`textvn_session::emoji::EmojiPicker`) replaces 3 diverging copies (`tray/src/emoji_popup.rs`,
  `adapters/linux-common/src/emoji_picker.c`, `adapters/macos-imk/Sources/IMKLib/EmojiPickerPanel.swift`):
  open/close, search box, 8 categories (Tab/Shift+Tab wrap around), arrow selection, scrolling, `1..9` quick
  pick, insert + MRU record. Search/ranking stays ONE source: `textvn_core::post::emoji_catalog::search_emojis`.
- **Layout is an adapter capability** (`PickerLayout`): Linux = native candidate list (`LIST`: 1 column, 9 rows,
  36 items); Windows/macOS = grid (`GRID`: 8×5, 120 items). Left/Right = ±1, Up/Down = ±`cols` ⇒ with 1 column it
  matches the old Linux list behaviour exactly.
- **Unified behaviour** (each OS used to differ): search box at most **32 characters** (macOS 32 chars, Linux 64
  bytes); digits `1..9` quick-pick within the **visible** part while the search box is empty (Linux had it, macOS
  treated digits as search text); `0`, and digits after letters, are search text; Enter with no results ⇒ close;
  Backspace on an empty box ⇒ no change.
- **The Windows tray uses it directly**: `TrayEmojiState` now only wraps `EmojiPicker` + the hovered cell (a Win32
  drawing concern); `GRID_COLS`/`GRID_VISIBLE_ROWS` come from `PickerLayout::GRID` (one source). Tray behaviour is
  unchanged (120 results, in-RAM MRU, clamped scrolling).
- **G8**: no logging; closing the picker clears the search box; the MRU only keeps `CATALOG` indices in RAM.
- **Part 1b — MRU decoupled from the picker (`EmojiSource`)**: `Engine::emoji_record_use` both records the MRU
  shared with `:shortcode:` expansion and counts **Dream RSI** (`accepts` + `chars_saved`, G19). A picker-owned MRU
  would make Linux/macOS (part 2) lose both ⇒ the MRU is now a source passed into every call: `impl EmojiSource for
  textvn_core::Engine` (processes with an engine) and `for EmojiMru` (the Windows tray, no engine — behaviour
  unchanged). Test `sf7_nguon_engine_dem_dream_rsi_va_mru_chung`; mutation dropping `emoji_record_use` ⇒ RED.
- **Part 1c — paged Linux list**: `PickerLayout::paged` (LIST = `true`): the window jumps by 9-item pages
  (`selected / 9 * 9`) like the native IBus/Fcitx5 candidate list shows, so `1..9` quick-pick matches the page the user
  sees; the grid (`GRID`) still slides row by row. Test `sf7_danh_sach_phan_trang_va_chon_nhanh_theo_trang`.
- **Noted (pre-existing, not caused by SF-7)**: emoji picked in the Windows tray are NOT counted in Dream RSI because
  the tray has no engine (its MRU lives only in tray RAM). Part 2 should send the counters over IPC `StyleUpdate`
  (the `RsiOnly` path already exists).
- **Part 2a — C ABI + Linux uses the shared picker**: `ffi/src/emoji_picker_abi.rs` exports 9 `ime_emoji_picker_*`
  functions (new/free/open/close/switch_category/scroll/key/pick/view) + `ime_emoji_view_v1` (a drawing snapshot:
  category, search box, selection, visible part ≤ 64 items, 32-byte glyph/shortcode) + `ime_emoji_item_v1`; the
  search/MRU source is the `ime_instance` engine (MRU shared with `:shortcode:` + Dream RSI). `pick` checks the buffer
  BEFORE recording the MRU (too small never counts Dream RSI without inserting). `adapters/linux-common/src/emoji_picker.c`
  drops its whole private state machine (search box, Tab, arrows, quick pick, result loading) — it only maps Linux keys
  ⇒ `IME_EPK_KEY_*`, copies the view into `lc_emoji_picker` for IBus/Fcitx5 to draw (the `lc_emoji_*` API is unchanged,
  engine.c/engine.cpp NOT touched) and builds the aux line; the shared picker only lives while the panel is open (freed
  on every close path). `textvn-cli verify`: 46 exports, 20 constants, 2 structs, sizeof/offset.
- **Part 2b (later)**: macOS `EmojiPickerModel`/`EmojiPickerPanel` call the C ABI (needs ci-macos rounds — no Swift on
  the dev machine); then move the category labels/icons into `data/ui/strings.toml` (SF-9).

## Kiểm thử / Testing
- `cargo test -p textvn-session` = 72 passed, gồm 14 test `sf7_*` (đóng không xử lý phím; mở nhóm Gần đây; Esc
  xoá ô tìm; Tab/Shift+Tab xoay vòng 8 nhóm + xoá ô tìm; tìm `rocket` ⇒ Enter chèn 🚀 + MRU lên đầu; Enter không
  kết quả ⇒ đóng; chọn nhanh chỉ khi ô rỗng; `0` là chữ tìm; mũi tên lưới và danh sách; Xuống kẹp ở mục cuối +
  cuộn theo chọn + chọn nhanh theo phần đang hiện; cuộn chuột kẹp biên; ô tìm 32 ký tự + bỏ ký tự điều khiển;
  bố cục suy biến không chia cho 0).
- Đột biến (D3), mỗi đột biến làm đỏ đúng 1 test: Xuống không kẹp; chọn nhanh bỏ qua phần đang hiện; đóng không xoá
  ô tìm (G8); Tab không xoá ô tìm; không ghi MRU.
- `cargo test -p textvn-tray emoji` xanh (test cũ `tray_emoji_state_pick_and_surrogate_utf16` giữ nguyên ý);
  `cargo clippy -p textvn-tray -p textvn-session --all-targets -D warnings` sạch trên Windows và
  `--target x86_64-unknown-linux-gnu`; rustfmt sạch.
- **Phần 2a — kiểm thử**: `cargo test -p textvn-ffi` 69 passed gồm 4 test `sf7_abi_*` (tìm + Enter chèn glyph + đếm
  Dream RSI qua ABI; tham số sai / phím lạ / code point hỏng / ABI lệch; nhóm + cuộn + view lưới; catalog vừa bộ đệm
  cố định) + `const assert` lúc biên dịch; **test C Linux `adapters/linux-common/tests/test_emoji_picker.c` KHÔNG sửa
  chạy PASS trên bản chung** (MSVC `/W4 /WX /std:c11` + lib ffi trên máy dev); `textvn-cli verify` x86_64 + i686;
  clippy Windows + Linux.
