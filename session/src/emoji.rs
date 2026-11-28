// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `emoji` dùng chung 3 OS trong `textvn-session` (P0-6 §5, task SF-7).
//!
//! MỘT máy trạng thái cho bộ chọn Emoji thay 3 bản lệch nhau (`tray/src/emoji_popup.rs`,
//! `adapters/linux-common/src/emoji_picker.c`, `adapters/macos-imk/.../EmojiPickerPanel.swift`):
//! mở/đóng, ô tìm, nhóm, chọn, phím, cuộn, chọn nhanh 1..9 và MRU. Tìm/xếp hạng vẫn là
//! `textvn_core::post::emoji_catalog::search_emojis` (một nguồn). Adapter chỉ:
//! 1. ánh xạ phím OS ⇒ [`PickerKey`], 2. vẽ theo [`EmojiPicker::visible_items`] + nhóm + ô tìm,
//! 3. chèn glyph khi nhận [`PickerOutcome::Commit`].
//!
//! Bố cục là NĂNG LỰC của adapter ([`PickerLayout`]): Linux dùng danh sách ứng viên native
//! (`cols = 1`, 9 dòng/trang, 36 mục), Windows/macOS dùng lưới 8×5. Mũi tên Trái/Phải = ±1,
//! Lên/Xuống = ±`cols` — với `cols = 1` trùng đúng hành vi danh sách Linux cũ.
//!
//! Thuần logic: không I/O, không đồng hồ, không log (G8 — không ghi ô tìm hay emoji được chọn).
//!
//! **MRU không nằm trong bộ chọn** mà ở [`EmojiSource`] do adapter truyền vào mỗi lời gọi: trong
//! tiến trình có engine (TSF/IBus/Fcitx5/IMK) nguồn là `textvn_core::Engine` — `emoji_record_use`
//! vừa ghi MRU dùng chung với gõ tắt `:shortcode:` vừa đếm Dream RSI (`accepts` + `chars_saved`,
//! G19); tray Windows (không có engine) dùng [`EmojiMru`] trong RAM.

use textvn_core::post::emoji_catalog::{search_emojis, EmojiEntry, EmojiMru};

/// Nguồn tìm kiếm + MRU cho bộ chọn (một nguồn xếp hạng `search_emojis`).
pub trait EmojiSource {
    /// Kết quả cho ô tìm `query` (rỗng = theo nhóm) ở nhóm `category`, tối đa `limit`.
    fn search(&self, query: &str, category: u8, limit: usize) -> Vec<&'static EmojiEntry>;
    /// Ghi nhận emoji vừa chèn (MRU; với engine: kèm hook Dream RSI G19).
    fn record_use(&mut self, glyph: &str);
}

/// MRU trong RAM (tray Windows — tiến trình không có engine).
impl EmojiSource for EmojiMru {
    fn search(&self, query: &str, category: u8, limit: usize) -> Vec<&'static EmojiEntry> {
        search_emojis(query, category, self, limit)
    }
    fn record_use(&mut self, glyph: &str) {
        self.record(glyph);
    }
}

/// Engine (TSF/IBus/Fcitx5/IMK): MRU chung với `:shortcode:` + đếm Dream RSI khi chèn (G19).
impl EmojiSource for textvn_core::Engine {
    fn search(&self, query: &str, category: u8, limit: usize) -> Vec<&'static EmojiEntry> {
        self.emoji_search(query, category, limit)
    }
    fn record_use(&mut self, glyph: &str) {
        let _ = self.emoji_record_use(glyph);
    }
}

/// Số nhóm (0 = Gần đây/MRU, 1..=7 nhóm cố định của `emoji_catalog`).
pub const CATEGORY_COUNT: u8 = 8;
/// Ô tìm tối đa (ký tự Unicode) — hợp nhất macOS (32 ký tự) và Linux (64 byte ≈ 21..64 ký tự).
pub const QUERY_MAX_CHARS: usize = 32;
/// Phím số chọn nhanh `1..=QUICK_PICK_MAX` trên phần đang hiện (khi ô tìm còn rỗng).
pub const QUICK_PICK_MAX: u8 = 9;

/// Bố cục bộ chọn do adapter khai (năng lực OS, không phải nghiệp vụ).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickerLayout {
    /// Số cột (1 = danh sách dọc).
    pub cols: usize,
    /// Số hàng hiện cùng lúc.
    pub visible_rows: usize,
    /// Số kết quả tối đa nạp từ `search_emojis`.
    pub max_items: usize,
}

impl PickerLayout {
    /// Lưới 8×5 của Windows tray / macOS panel.
    pub const GRID: Self = Self {
        cols: 8,
        visible_rows: 5,
        max_items: 120,
    };
    /// Danh sách ứng viên native IBus/Fcitx5 (9 mục/trang, 36 mục — `LC_EMOJI_MAX_ITEMS`).
    pub const LIST: Self = Self {
        cols: 1,
        visible_rows: 9,
        max_items: 36,
    };

    fn sane(self) -> Self {
        Self {
            cols: self.cols.max(1),
            visible_rows: self.visible_rows.max(1),
            max_items: self.max_items.max(1),
        }
    }
}

/// Phím đã chuẩn hoá bởi adapter (VK canonical / keyCode macOS / keysym ⇒ ý nghĩa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKey {
    /// Đóng không chèn.
    Escape,
    /// Chèn mục đang chọn (không có mục ⇒ đóng).
    Enter,
    /// Nhóm kế tiếp (xoay vòng), xoá ô tìm.
    Tab,
    /// Nhóm trước (xoay vòng), xoá ô tìm.
    ShiftTab,
    /// Xoá 1 ký tự cuối của ô tìm.
    Backspace,
    /// Chọn mục trước.
    Left,
    /// Chọn mục sau.
    Right,
    /// Lên 1 hàng (`cols` mục).
    Up,
    /// Xuống 1 hàng (`cols` mục, kẹp ở mục cuối).
    Down,
    /// Ký tự in được (`ch`) — chữ số `1..9` khi ô tìm rỗng = chọn nhanh.
    Char(char),
}

/// Kết quả một thao tác — adapter vẽ lại / chèn / ẩn panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerOutcome {
    /// Panel đang đóng — adapter xử lý phím như bình thường.
    NotOpen,
    /// Trạng thái hiển thị đổi (chọn, cuộn, ô tìm, nhóm) — vẽ lại.
    Redraw,
    /// Chèn glyph này rồi ẩn panel (MRU đã ghi).
    Commit(&'static str),
    /// Đóng panel không chèn gì.
    Closed,
}

/// Máy trạng thái bộ chọn Emoji dùng chung 3 OS.
#[derive(Debug, Clone)]
pub struct EmojiPicker {
    layout: PickerLayout,
    open: bool,
    category: u8,
    query: String,
    items: Vec<&'static EmojiEntry>,
    selected: usize,
    scroll_row: usize,
}

impl Default for EmojiPicker {
    fn default() -> Self {
        Self::new(PickerLayout::GRID)
    }
}

impl EmojiPicker {
    /// Bộ chọn đóng, MRU rỗng.
    #[must_use]
    pub fn new(layout: PickerLayout) -> Self {
        Self {
            layout: layout.sane(),
            open: false,
            category: 0,
            query: String::new(),
            items: Vec::new(),
            selected: 0,
            scroll_row: 0,
        }
    }

    /// Bố cục đang dùng.
    #[must_use]
    pub fn layout(&self) -> PickerLayout {
        self.layout
    }
    /// Panel đang mở?
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Nhóm hiện tại `0..CATEGORY_COUNT`.
    #[must_use]
    pub fn category(&self) -> u8 {
        self.category
    }
    /// Ô tìm hiện tại (chỉ để vẽ — G8: không log).
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }
    /// Toàn bộ kết quả đã nạp.
    #[must_use]
    pub fn items(&self) -> &[&'static EmojiEntry] {
        &self.items
    }
    /// Chỉ số mục đang chọn (trong [`Self::items`]).
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }
    /// Hàng đầu đang hiện.
    #[must_use]
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }
    /// Chỉ số mục đầu tiên đang hiện.
    #[must_use]
    pub fn first_visible(&self) -> usize {
        self.scroll_row * self.layout.cols
    }
    /// Phần kết quả đang hiện (`visible_rows × cols` mục kể từ [`Self::first_visible`]).
    #[must_use]
    pub fn visible_items(&self) -> &[&'static EmojiEntry] {
        let start = self.first_visible().min(self.items.len());
        let end = (start + self.layout.cols * self.layout.visible_rows).min(self.items.len());
        &self.items[start..end]
    }
    /// Mở ở nhóm 0 (Gần đây), ô tìm rỗng.
    pub fn open(&mut self, src: &dyn EmojiSource) {
        self.open = true;
        self.category = 0;
        self.query.clear();
        self.reload(src);
    }

    /// Đóng và xoá ô tìm (G8: không giữ chữ tìm sau khi đóng).
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.items.clear();
        self.selected = 0;
        self.scroll_row = 0;
    }

    /// Chuyển nhóm (click tab / Tab): xoá ô tìm, nạp lại, về đầu.
    pub fn switch_category(&mut self, src: &dyn EmojiSource, cat: u8) {
        self.category = cat % CATEGORY_COUNT;
        self.query.clear();
        self.reload(src);
    }

    /// Cuộn `delta` hàng (bánh xe chuột), kẹp trong phạm vi.
    pub fn scroll_by(&mut self, delta: i64) {
        let max_row = self.max_scroll_row() as i64;
        self.scroll_row = (self.scroll_row as i64 + delta).clamp(0, max_row) as usize;
    }

    /// Chọn mục `idx` (click chuột / chọn nhanh): ghi MRU, đóng, trả glyph. Sai chỉ số ⇒ `None`.
    pub fn pick(&mut self, src: &mut dyn EmojiSource, idx: usize) -> Option<&'static str> {
        if !self.open {
            return None;
        }
        let glyph = self.items.get(idx)?.glyph;
        src.record_use(glyph);
        self.close();
        Some(glyph)
    }

    /// Xử lý một phím khi panel mở.
    pub fn handle_key(&mut self, src: &mut dyn EmojiSource, key: PickerKey) -> PickerOutcome {
        if !self.open {
            return PickerOutcome::NotOpen;
        }
        match key {
            PickerKey::Escape => {
                self.close();
                PickerOutcome::Closed
            }
            PickerKey::Enter => self.commit_or_close(src, self.selected),
            PickerKey::Tab => {
                self.switch_category(src, (self.category + 1) % CATEGORY_COUNT);
                PickerOutcome::Redraw
            }
            PickerKey::ShiftTab => {
                self.switch_category(src, (self.category + CATEGORY_COUNT - 1) % CATEGORY_COUNT);
                PickerOutcome::Redraw
            }
            PickerKey::Backspace => {
                if self.query.pop().is_some() {
                    self.reload(src);
                }
                PickerOutcome::Redraw
            }
            PickerKey::Left => {
                self.move_to(self.selected.checked_sub(1));
                PickerOutcome::Redraw
            }
            PickerKey::Right => {
                self.move_to(Some(self.selected + 1).filter(|&i| i < self.items.len()));
                PickerOutcome::Redraw
            }
            PickerKey::Up => {
                self.move_to(self.selected.checked_sub(self.layout.cols));
                PickerOutcome::Redraw
            }
            PickerKey::Down => {
                if !self.items.is_empty() {
                    let next = (self.selected + self.layout.cols).min(self.items.len() - 1);
                    self.move_to(Some(next));
                }
                PickerOutcome::Redraw
            }
            PickerKey::Char(c) => self.on_char(src, c),
        }
    }

    fn on_char(&mut self, src: &mut dyn EmojiSource, c: char) -> PickerOutcome {
        if self.query.is_empty() {
            if let Some(d) = c
                .to_digit(10)
                .filter(|d| (1..=u32::from(QUICK_PICK_MAX)).contains(d))
            {
                let idx = self.first_visible() + d as usize - 1;
                return if idx < self.items.len() {
                    self.commit_or_close(src, idx)
                } else {
                    PickerOutcome::Redraw
                };
            }
        }
        if c.is_control() || self.query.chars().count() >= QUERY_MAX_CHARS {
            return PickerOutcome::Redraw;
        }
        self.query.push(c);
        self.reload(src);
        PickerOutcome::Redraw
    }

    fn commit_or_close(&mut self, src: &mut dyn EmojiSource, idx: usize) -> PickerOutcome {
        match self.pick(src, idx) {
            Some(g) => PickerOutcome::Commit(g),
            None => {
                self.close();
                PickerOutcome::Closed
            }
        }
    }

    fn move_to(&mut self, idx: Option<usize>) {
        if let Some(i) = idx {
            self.selected = i;
            self.ensure_visible();
        }
    }

    fn max_scroll_row(&self) -> usize {
        let rows = self.items.len().div_ceil(self.layout.cols);
        rows.saturating_sub(self.layout.visible_rows)
    }

    fn ensure_visible(&mut self) {
        let row = self.selected / self.layout.cols;
        if row < self.scroll_row {
            self.scroll_row = row;
        } else if row >= self.scroll_row + self.layout.visible_rows {
            self.scroll_row = row + 1 - self.layout.visible_rows;
        }
    }

    fn reload(&mut self, src: &dyn EmojiSource) {
        self.items = src.search(&self.query, self.category, self.layout.max_items);
        self.selected = 0;
        self.scroll_row = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened_custom(layout: PickerLayout, m: &EmojiMru) -> EmojiPicker {
        opened(layout, m)
    }

    fn opened(layout: PickerLayout, m: &EmojiMru) -> EmojiPicker {
        let mut p = EmojiPicker::new(layout);
        p.open(m);
        p
    }

    #[test]
    fn sf7_dong_thi_khong_xu_ly_phim() {
        let mut m = EmojiMru::default();
        let mut p = EmojiPicker::new(PickerLayout::GRID);
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Enter),
            PickerOutcome::NotOpen
        );
        assert_eq!(p.pick(&mut m, 0), None);
    }

    #[test]
    fn sf7_mo_nhom_gan_day_co_ket_qua_va_esc_xoa_o_tim() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::GRID, &m);
        assert!(p.is_open() && p.category() == 0 && !p.items().is_empty());
        p.handle_key(&mut m, PickerKey::Tab);
        p.handle_key(&mut m, PickerKey::Char('t'));
        assert_eq!(p.query(), "t");
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Escape),
            PickerOutcome::Closed
        );
        assert!(
            !p.is_open() && p.query().is_empty() && p.items().is_empty(),
            "G8: đóng là xoá ô tìm"
        );
    }

    #[test]
    fn sf7_tab_xoay_vong_8_nhom_va_xoa_o_tim() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::LIST, &m);
        p.handle_key(&mut m, PickerKey::Char('x'));
        p.handle_key(&mut m, PickerKey::ShiftTab);
        assert_eq!(p.category(), 7);
        assert!(p.query().is_empty());
        for want in [0, 1, 2, 3, 4, 5, 6, 7, 0] {
            p.handle_key(&mut m, PickerKey::Tab);
            assert_eq!(p.category(), want);
        }
    }

    #[test]
    fn sf7_tim_rocket_enter_chen_va_mru_len_dau() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::GRID, &m);
        for c in "rocket".chars() {
            p.handle_key(&mut m, PickerKey::Char(c));
        }
        assert_eq!(p.items().first().map(|e| e.glyph), Some("🚀"));
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Enter),
            PickerOutcome::Commit("🚀")
        );
        assert!(!p.is_open());
        p.open(&m);
        assert_eq!(p.items()[0].glyph, "🚀", "MRU đứng đầu nhóm Gần đây");
    }

    #[test]
    fn sf7_enter_khi_khong_co_ket_qua_thi_dong() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::GRID, &m);
        for c in "zzzzqqqqxxxx".chars() {
            p.handle_key(&mut m, PickerKey::Char(c));
        }
        assert!(p.items().is_empty());
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Enter),
            PickerOutcome::Closed
        );
    }

    #[test]
    fn sf7_so_1_9_chon_nhanh_chi_khi_o_tim_rong() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::LIST, &m);
        let third = p.items()[2].glyph;
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Char('3')),
            PickerOutcome::Commit(third)
        );
        // Ô tìm đã có chữ ⇒ số là chữ tìm, không chọn nhanh.
        let mut p = opened(PickerLayout::GRID, &m);
        p.handle_key(&mut m, PickerKey::Char('a'));
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Char('1')),
            PickerOutcome::Redraw
        );
        assert!(p.is_open());
        assert_eq!(p.query(), "a1");
    }

    #[test]
    fn sf7_so_0_va_so_ngoai_trang_khong_chon() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::LIST, &m);
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Char('0')),
            PickerOutcome::Redraw
        );
        assert_eq!(p.query(), "0", "0 không phải phím chọn nhanh ⇒ là chữ tìm");
    }

    #[test]
    fn sf7_mui_ten_luoi_va_danh_sach() {
        let mut m = EmojiMru::default();
        let mut g = opened(PickerLayout::GRID, &m);
        g.switch_category(&m, 1);
        assert!(g.items().len() > 20);
        g.handle_key(&mut m, PickerKey::Down);
        assert_eq!(g.selected(), 8);
        g.handle_key(&mut m, PickerKey::Right);
        assert_eq!(g.selected(), 9);
        g.handle_key(&mut m, PickerKey::Up);
        assert_eq!(g.selected(), 1);
        g.handle_key(&mut m, PickerKey::Up);
        assert_eq!(g.selected(), 1, "Lên ở hàng đầu = giữ nguyên");
        g.handle_key(&mut m, PickerKey::Left);
        g.handle_key(&mut m, PickerKey::Left);
        assert_eq!(g.selected(), 0, "Trái ở đầu = giữ nguyên");
        // Danh sách (cols = 1): Lên/Xuống = ±1 như Linux cũ.
        let mut l = opened(PickerLayout::LIST, &m);
        l.handle_key(&mut m, PickerKey::Down);
        l.handle_key(&mut m, PickerKey::Down);
        assert_eq!(l.selected(), 2);
        l.handle_key(&mut m, PickerKey::Up);
        assert_eq!(l.selected(), 1);
    }

    #[test]
    fn sf7_xuong_ke_o_muc_cuoi_va_cuon_theo_chon() {
        let mut m = EmojiMru::default();
        let mut g = opened(PickerLayout::GRID, &m);
        g.switch_category(&m, 1);
        let n = g.items().len();
        for _ in 0..1000 {
            g.handle_key(&mut m, PickerKey::Down);
        }
        assert_eq!(g.selected(), n - 1, "Xuống kẹp ở mục cuối");
        let row = g.selected() / 8;
        assert!(
            row >= g.scroll_row() && row < g.scroll_row() + 5,
            "mục chọn luôn đang hiện"
        );
        assert!(g.visible_items().len() <= 40);
        assert!(g
            .visible_items()
            .iter()
            .any(|e| e.glyph == g.items()[n - 1].glyph));
        // Chọn nhanh tính theo phần ĐANG HIỆN.
        let first = g.visible_items()[0].glyph;
        assert_eq!(
            g.handle_key(&mut m, PickerKey::Char('1')),
            PickerOutcome::Commit(first)
        );
    }

    #[test]
    fn sf7_cuon_chuot_kep_bien() {
        let m = EmojiMru::default();
        let mut g = opened(PickerLayout::GRID, &m);
        g.switch_category(&m, 1);
        g.scroll_by(-5);
        assert_eq!(g.scroll_row(), 0);
        g.scroll_by(10_000);
        let rows = g.items().len().div_ceil(8);
        assert_eq!(g.scroll_row(), rows.saturating_sub(5));
    }

    #[test]
    fn sf7_o_tim_toi_da_32_ky_tu_va_bo_ky_tu_dieu_khien() {
        let mut m = EmojiMru::default();
        let mut p = opened(PickerLayout::GRID, &m);
        p.handle_key(&mut m, PickerKey::Char('a'));
        for _ in 0..100 {
            p.handle_key(&mut m, PickerKey::Char('ơ'));
        }
        assert_eq!(p.query().chars().count(), QUERY_MAX_CHARS);
        p.handle_key(&mut m, PickerKey::Char('\u{7f}'));
        p.handle_key(&mut m, PickerKey::Char('\n'));
        assert_eq!(p.query().chars().count(), QUERY_MAX_CHARS);
        for _ in 0..40 {
            p.handle_key(&mut m, PickerKey::Backspace);
        }
        assert!(p.query().is_empty());
        assert_eq!(
            p.handle_key(&mut m, PickerKey::Backspace),
            PickerOutcome::Redraw
        );
    }

    #[test]
    fn sf7_bo_cuc_suy_bien_khong_chia_cho_0() {
        let mut m = EmojiMru::default();
        let mut p = opened_custom(
            PickerLayout {
                cols: 0,
                visible_rows: 0,
                max_items: 0,
            },
            &m,
        );
        p.handle_key(&mut m, PickerKey::Down);
        p.scroll_by(3);
        assert_eq!(p.layout().cols, 1);
        assert!(p.visible_items().len() <= 1);
    }

    /// G19: chèn emoji qua bộ chọn trong tiến trình có engine phải đếm Dream RSI (accepts +
    /// chars_saved) và đưa emoji lên MRU dùng chung với `:shortcode:`.
    #[test]
    fn sf7_nguon_engine_dem_dream_rsi_va_mru_chung() {
        let mut eng = textvn_core::Engine::new(textvn_core::EngineOptions::default());
        let before = eng.dream_rsi_snapshot();
        let mut p = EmojiPicker::new(PickerLayout::LIST);
        p.open(&eng);
        for c in "rocket".chars() {
            p.handle_key(&mut eng, PickerKey::Char(c));
        }
        assert_eq!(
            p.handle_key(&mut eng, PickerKey::Enter),
            PickerOutcome::Commit("🚀")
        );
        let after = eng.dream_rsi_snapshot();
        assert_eq!(
            after.accepts,
            before.accepts + 1,
            "G19: accept emoji phải được đếm"
        );
        assert_eq!(after.chars_saved, before.chars_saved + 1);
        p.open(&eng);
        assert_eq!(
            p.items()[0].glyph,
            "🚀",
            "MRU của engine (chung với :shortcode:)"
        );
    }
}
