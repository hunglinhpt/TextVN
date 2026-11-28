<!-- TASK: SF-7 · commit: <SHA_CODE> · agent: claude-opus-7e41b2d9 -->
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
- **Phần 2 (sau khi SF-3 nhả `ffi/`)**: C ABI `ime_emoji_picker_*` + Linux (`emoji_picker.c`) và macOS
  (`EmojiPickerModel`) gọi bản dùng chung; khi đó xoá 2 bản sao, và nhãn/biểu tượng nhóm (đang lặp ở
  `CATEGORY_TABS` tray, `CATEGORY_NAMES` Linux, `tabs` macOS) chuyển vào `data/ui/strings.toml` (SF-9).

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
- **Part 2 (after SF-3 releases `ffi/`)**: C ABI `ime_emoji_picker_*` + Linux (`emoji_picker.c`) and macOS
  (`EmojiPickerModel`) call the shared one; then delete the 2 copies, and move the category labels/icons (repeated in
  the tray's `CATEGORY_TABS`, Linux `CATEGORY_NAMES`, macOS `tabs`) into `data/ui/strings.toml` (SF-9).

## Kiểm thử / Testing
- `cargo test -p textvn-session` = 52 passed, gồm 12 test `sf7_*` (đóng không xử lý phím; mở nhóm Gần đây; Esc
  xoá ô tìm; Tab/Shift+Tab xoay vòng 8 nhóm + xoá ô tìm; tìm `rocket` ⇒ Enter chèn 🚀 + MRU lên đầu; Enter không
  kết quả ⇒ đóng; chọn nhanh chỉ khi ô rỗng; `0` là chữ tìm; mũi tên lưới và danh sách; Xuống kẹp ở mục cuối +
  cuộn theo chọn + chọn nhanh theo phần đang hiện; cuộn chuột kẹp biên; ô tìm 32 ký tự + bỏ ký tự điều khiển;
  bố cục suy biến không chia cho 0).
- Đột biến (D3), mỗi đột biến làm đỏ đúng 1 test: Xuống không kẹp; chọn nhanh bỏ qua phần đang hiện; đóng không xoá
  ô tìm (G8); Tab không xoá ô tìm; không ghi MRU.
- `cargo test -p textvn-tray emoji` xanh (test cũ `tray_emoji_state_pick_and_surrogate_utf16` giữ nguyên ý);
  `cargo clippy -p textvn-tray -p textvn-session --all-targets -D warnings` sạch trên Windows và
  `--target x86_64-unknown-linux-gnu`; rustfmt sạch.
