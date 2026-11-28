// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-7 phần 2 — C ABI `ime_emoji_picker_*` cho bộ chọn Emoji dùng chung
//! (`textvn_session::emoji::EmojiPicker`, P0-6 §5). Linux (IBus/Fcitx5 — `emoji_picker.c`) và
//! macOS (IMK — `EmojiPickerModel`) chỉ ánh xạ phím ⇒ `IME_EPK_KEY_*`, vẽ theo
//! [`ime_emoji_view_v1`] và chèn glyph khi nhận `IME_EPK_COMMIT`.
//!
//! Nguồn tìm kiếm/MRU là **engine của `ime_instance`** (`EmojiSource for Engine`): chèn qua bộ
//! chọn ghi MRU dùng chung với `:shortcode:` và đếm Dream RSI (G19) — đúng như
//! `ime_emoji_record_use` trước đây. Không callback, không cấp phát chéo: adapter cấp
//! `ime_emoji_view_v1` / bộ đệm glyph. G8: không log ô tìm hay glyph.

use textvn_session::emoji::{EmojiPicker, PickerKey, PickerLayout, PickerOutcome};

use crate::{ime_instance, IME_ERR_ABI, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG, IME_OK};

pub const IME_EMOJI_PICKER_ABI_VERSION: u32 = 1;

/// Bố cục (năng lực adapter).
pub const IME_EPK_LAYOUT_GRID: u32 = 0;
pub const IME_EPK_LAYOUT_LIST: u32 = 1;

/// Phím đã chuẩn hoá.
pub const IME_EPK_KEY_ESCAPE: u32 = 1;
pub const IME_EPK_KEY_ENTER: u32 = 2;
pub const IME_EPK_KEY_TAB: u32 = 3;
pub const IME_EPK_KEY_SHIFT_TAB: u32 = 4;
pub const IME_EPK_KEY_BACKSPACE: u32 = 5;
pub const IME_EPK_KEY_LEFT: u32 = 6;
pub const IME_EPK_KEY_RIGHT: u32 = 7;
pub const IME_EPK_KEY_UP: u32 = 8;
pub const IME_EPK_KEY_DOWN: u32 = 9;
/// Ký tự in được: `ch` = code point Unicode.
pub const IME_EPK_KEY_CHAR: u32 = 10;

/// Kết quả (≥ 0); lỗi là mã `IME_ERR_*` âm.
pub const IME_EPK_NOT_OPEN: i32 = 0;
pub const IME_EPK_REDRAW: i32 = 1;
pub const IME_EPK_COMMIT: i32 = 2;
pub const IME_EPK_CLOSED: i32 = 3;

/// Số mục tối đa trong một view (lưới 8×5 = 40, danh sách 9).
pub const IME_EPK_VIEW_MAX: usize = 64;
/// Byte tối đa của glyph / shortcode kể cả NUL (catalog hiện ≤ 11 / ≤ 19 byte — test khoá).
pub const IME_EPK_TEXT_MAX: usize = 32;
/// Byte ô tìm kể cả NUL: `QUERY_MAX_CHARS` (32) × 4 byte UTF-8 + NUL, làm tròn bội 4.
pub const IME_EPK_QUERY_MAX: usize = 132;

// Kiểm lúc BIÊN DỊCH: ô tìm tối đa và phần hiện của mọi bố cục vừa bộ đệm cố định của view.
const _: () = assert!(textvn_session::emoji::QUERY_MAX_CHARS * 4 < IME_EPK_QUERY_MAX);
const _: () =
    assert!(PickerLayout::GRID.cols * PickerLayout::GRID.visible_rows <= IME_EPK_VIEW_MAX);
const _: () =
    assert!(PickerLayout::LIST.cols * PickerLayout::LIST.visible_rows <= IME_EPK_VIEW_MAX);

/// Opaque.
#[allow(non_camel_case_types)]
pub struct ime_emoji_picker {
    picker: EmojiPicker,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ime_emoji_item_v1 {
    /// UTF-8, kết thúc NUL.
    pub glyph: [u8; IME_EPK_TEXT_MAX],
    /// ASCII, kết thúc NUL.
    pub shortcode: [u8; IME_EPK_TEXT_MAX],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ime_emoji_view_v1 {
    /// Adapter đặt = `IME_EMOJI_PICKER_ABI_VERSION` trước khi gọi.
    pub abi_version: u32,
    pub open: u32,
    pub category: u32,
    /// Chỉ số mục đang chọn (trong toàn bộ kết quả).
    pub selected: u32,
    /// Chỉ số (trong toàn bộ kết quả) của `items[0]`.
    pub first_visible: u32,
    /// Tổng số kết quả đã nạp.
    pub total: u32,
    /// Số mục hợp lệ trong `items` (≤ `IME_EPK_VIEW_MAX`).
    pub count: u32,
    /// Số byte của `query` (không tính NUL).
    pub query_len: u32,
    pub query: [u8; IME_EPK_QUERY_MAX],
    pub items: [ime_emoji_item_v1; IME_EPK_VIEW_MAX],
}

impl ime_emoji_view_v1 {
    /// View rỗng đã đặt `abi_version` (tiện cho Rust/test).
    #[must_use]
    pub fn new() -> Self {
        Self {
            abi_version: IME_EMOJI_PICKER_ABI_VERSION,
            open: 0,
            category: 0,
            selected: 0,
            first_visible: 0,
            total: 0,
            count: 0,
            query_len: 0,
            query: [0; IME_EPK_QUERY_MAX],
            items: [ime_emoji_item_v1 {
                glyph: [0; IME_EPK_TEXT_MAX],
                shortcode: [0; IME_EPK_TEXT_MAX],
            }; IME_EPK_VIEW_MAX],
        }
    }
}

impl Default for ime_emoji_view_v1 {
    fn default() -> Self {
        Self::new()
    }
}

/// Chép `s` vào mảng cố định kèm NUL; dài quá thì cắt ở ranh giới ký tự (không chẻ UTF-8).
fn put_cstr(dst: &mut [u8], s: &str) -> usize {
    let mut n = s.len().min(dst.len().saturating_sub(1));
    while n > 0 && !s.is_char_boundary(n) {
        n -= 1;
    }
    dst[..n].copy_from_slice(&s.as_bytes()[..n]);
    dst[n] = 0;
    n
}

fn to_key(key: u32, ch: u32) -> Option<PickerKey> {
    Some(match key {
        IME_EPK_KEY_ESCAPE => PickerKey::Escape,
        IME_EPK_KEY_ENTER => PickerKey::Enter,
        IME_EPK_KEY_TAB => PickerKey::Tab,
        IME_EPK_KEY_SHIFT_TAB => PickerKey::ShiftTab,
        IME_EPK_KEY_BACKSPACE => PickerKey::Backspace,
        IME_EPK_KEY_LEFT => PickerKey::Left,
        IME_EPK_KEY_RIGHT => PickerKey::Right,
        IME_EPK_KEY_UP => PickerKey::Up,
        IME_EPK_KEY_DOWN => PickerKey::Down,
        IME_EPK_KEY_CHAR => PickerKey::Char(char::from_u32(ch)?),
        _ => return None,
    })
}

fn outcome_code(o: PickerOutcome) -> i32 {
    match o {
        PickerOutcome::NotOpen => IME_EPK_NOT_OPEN,
        PickerOutcome::Redraw => IME_EPK_REDRAW,
        PickerOutcome::Commit(_) => IME_EPK_COMMIT,
        PickerOutcome::Closed => IME_EPK_CLOSED,
    }
}

/// Ghi glyph vào bộ đệm của adapter (`cap` gồm NUL). Thiếu chỗ ⇒ `false` (không ghi gì).
fn write_glyph(glyph: &str, out: *mut u8, cap: usize, out_len: *mut usize) -> bool {
    if out.is_null() || out_len.is_null() || cap < glyph.len() + 1 {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(glyph.as_ptr(), out, glyph.len());
        *out.add(glyph.len()) = 0;
        *out_len = glyph.len();
    }
    true
}

fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
}

pub(crate) fn ime_emoji_picker_new_impl(layout: u32, out: *mut *mut ime_emoji_picker) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let layout = match layout {
        IME_EPK_LAYOUT_GRID => PickerLayout::GRID,
        IME_EPK_LAYOUT_LIST => PickerLayout::LIST,
        _ => return IME_ERR_INVALID_ARG,
    };
    match guarded(|| {
        Box::into_raw(Box::new(ime_emoji_picker {
            picker: EmojiPicker::new(layout),
        }))
    }) {
        Some(p) => {
            unsafe { *out = p };
            IME_OK
        }
        None => IME_ERR_INTERNAL,
    }
}

pub(crate) fn ime_emoji_picker_free_impl(p: *mut ime_emoji_picker) {
    if p.is_null() {
        return;
    }
    let _ = guarded(|| unsafe { drop(Box::from_raw(p)) });
}

pub(crate) fn ime_emoji_picker_open_impl(
    p: *mut ime_emoji_picker,
    inst: *const ime_instance,
) -> i32 {
    if p.is_null() || inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let (p, inst) = unsafe { (&mut *p, &*inst) };
    match guarded(|| p.picker.open(&inst.engine)) {
        Some(()) => IME_OK,
        None => IME_ERR_INTERNAL,
    }
}

pub(crate) fn ime_emoji_picker_close_impl(p: *mut ime_emoji_picker) -> i32 {
    if p.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let p = unsafe { &mut *p };
    match guarded(|| p.picker.close()) {
        Some(()) => IME_OK,
        None => IME_ERR_INTERNAL,
    }
}

pub(crate) fn ime_emoji_picker_switch_category_impl(
    p: *mut ime_emoji_picker,
    inst: *const ime_instance,
    category: u32,
) -> i32 {
    if p.is_null() || inst.is_null() || category > u32::from(u8::MAX) {
        return IME_ERR_INVALID_ARG;
    }
    let (p, inst) = unsafe { (&mut *p, &*inst) };
    match guarded(|| p.picker.switch_category(&inst.engine, category as u8)) {
        Some(()) => IME_OK,
        None => IME_ERR_INTERNAL,
    }
}

pub(crate) fn ime_emoji_picker_scroll_impl(p: *mut ime_emoji_picker, rows: i32) -> i32 {
    if p.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let p = unsafe { &mut *p };
    match guarded(|| p.picker.scroll_by(i64::from(rows))) {
        Some(()) => IME_OK,
        None => IME_ERR_INTERNAL,
    }
}

/// Một phím. `IME_EPK_COMMIT` ⇒ glyph (UTF-8 + NUL) đã ở `glyph_out`, MRU + Dream RSI đã ghi.
/// Bộ đệm glyph thiếu chỗ khi commit ⇒ `IME_ERR_INVALID_ARG` (bộ chọn đã đóng — adapter không chèn).
pub(crate) fn ime_emoji_picker_key_impl(
    p: *mut ime_emoji_picker,
    inst: *mut ime_instance,
    key: u32,
    ch: u32,
    glyph_out: *mut u8,
    glyph_cap: usize,
    glyph_len: *mut usize,
) -> i32 {
    if p.is_null() || inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let Some(key) = to_key(key, ch) else {
        return IME_ERR_INVALID_ARG;
    };
    let (p, inst) = unsafe { (&mut *p, &mut *inst) };
    match guarded(|| p.picker.handle_key(&mut inst.engine, key)) {
        Some(PickerOutcome::Commit(g)) => {
            if write_glyph(g, glyph_out, glyph_cap, glyph_len) {
                IME_EPK_COMMIT
            } else {
                IME_ERR_INVALID_ARG
            }
        }
        Some(o) => outcome_code(o),
        None => IME_ERR_INTERNAL,
    }
}

/// Chọn trực tiếp mục `idx` (click chuột). Hợp lệ ⇒ `IME_EPK_COMMIT` + glyph; sai chỉ số ⇒
/// `IME_EPK_REDRAW` (không đổi gì).
pub(crate) fn ime_emoji_picker_pick_impl(
    p: *mut ime_emoji_picker,
    inst: *mut ime_instance,
    idx: u32,
    glyph_out: *mut u8,
    glyph_cap: usize,
    glyph_len: *mut usize,
) -> i32 {
    if p.is_null() || inst.is_null() || glyph_out.is_null() || glyph_len.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let (p, inst) = unsafe { (&mut *p, &mut *inst) };
    // Kiểm chỗ TRƯỚC khi chọn — chọn rồi mới thiếu chỗ thì MRU/Dream RSI đã ghi mà không chèn.
    let Some(g) = p.picker.items().get(idx as usize).map(|e| e.glyph) else {
        return IME_EPK_REDRAW;
    };
    if glyph_cap < g.len() + 1 {
        return IME_ERR_INVALID_ARG;
    }
    match guarded(|| p.picker.pick(&mut inst.engine, idx as usize)) {
        Some(Some(g)) => {
            let _ = write_glyph(g, glyph_out, glyph_cap, glyph_len);
            IME_EPK_COMMIT
        }
        Some(None) => IME_EPK_REDRAW,
        None => IME_ERR_INTERNAL,
    }
}

/// Ảnh chụp để vẽ: nhóm, ô tìm, mục chọn, phần đang hiện (≤ `IME_EPK_VIEW_MAX` mục).
pub(crate) fn ime_emoji_picker_view_impl(
    p: *const ime_emoji_picker,
    out: *mut ime_emoji_view_v1,
) -> i32 {
    if p.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let (p, out) = unsafe { (&*p, &mut *out) };
    if out.abi_version != IME_EMOJI_PICKER_ABI_VERSION {
        return IME_ERR_ABI;
    }
    let pk = &p.picker;
    out.open = u32::from(pk.is_open());
    out.category = u32::from(pk.category());
    out.selected = pk.selected() as u32;
    out.first_visible = pk.first_visible() as u32;
    out.total = pk.items().len() as u32;
    out.query_len = put_cstr(&mut out.query, pk.query()) as u32;
    let vis = pk.visible_items();
    let n = vis.len().min(IME_EPK_VIEW_MAX);
    for (slot, e) in out.items.iter_mut().zip(vis.iter().take(n)) {
        put_cstr(&mut slot.glyph, e.glyph);
        put_cstr(&mut slot.shortcode, e.shortcode);
    }
    out.count = n as u32;
    IME_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ime_instance_free, ime_instance_new};

    fn inst() -> *mut ime_instance {
        let mut i = std::ptr::null_mut();
        assert_eq!(ime_instance_new(std::ptr::null(), 0, &mut i), IME_OK);
        i
    }

    fn picker(layout: u32) -> *mut ime_emoji_picker {
        let mut p = std::ptr::null_mut();
        assert_eq!(ime_emoji_picker_new_impl(layout, &mut p), IME_OK);
        p
    }

    fn view(p: *const ime_emoji_picker) -> ime_emoji_view_v1 {
        let mut v = ime_emoji_view_v1::new();
        assert_eq!(ime_emoji_picker_view_impl(p, &mut v), IME_OK);
        v
    }

    fn cstr(b: &[u8]) -> &str {
        let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
        std::str::from_utf8(&b[..n]).unwrap()
    }

    #[test]
    fn sf7_abi_catalog_vua_bo_dem_co_dinh() {
        for e in
            textvn_core::post::emoji_catalog::search_emojis("", 0xFF, &Default::default(), 10_000)
        {
            assert!(
                e.glyph.len() < IME_EPK_TEXT_MAX,
                "glyph {} quá dài",
                e.shortcode
            );
            assert!(
                e.shortcode.len() < IME_EPK_TEXT_MAX,
                "shortcode {} quá dài",
                e.shortcode
            );
        }
    }

    #[test]
    fn sf7_abi_tim_enter_chen_glyph_dem_dream_rsi() {
        let (i, p) = (inst(), picker(IME_EPK_LAYOUT_LIST));
        assert_eq!(ime_emoji_picker_open_impl(p, i), IME_OK);
        let before = unsafe { (*i).engine.dream_rsi_snapshot() };
        let mut buf = [0u8; IME_EPK_TEXT_MAX];
        let mut n = 0usize;
        for c in "rocket".chars() {
            let rc = ime_emoji_picker_key_impl(
                p,
                i,
                IME_EPK_KEY_CHAR,
                c as u32,
                buf.as_mut_ptr(),
                buf.len(),
                &mut n,
            );
            assert_eq!(rc, IME_EPK_REDRAW);
        }
        let v = view(p);
        assert_eq!((v.open, cstr(&v.query[..]), v.query_len), (1, "rocket", 6));
        assert_eq!(cstr(&v.items[0].glyph), "🚀");
        assert_eq!(cstr(&v.items[0].shortcode), "rocket");
        let rc = ime_emoji_picker_key_impl(
            p,
            i,
            IME_EPK_KEY_ENTER,
            0,
            buf.as_mut_ptr(),
            buf.len(),
            &mut n,
        );
        assert_eq!(rc, IME_EPK_COMMIT);
        assert_eq!(&buf[..n], "🚀".as_bytes());
        assert_eq!(buf[n], 0);
        let after = unsafe { (*i).engine.dream_rsi_snapshot() };
        assert_eq!(after.accepts, before.accepts + 1, "G19 qua ABI");
        assert_eq!(view(p).open, 0);
        ime_emoji_picker_free_impl(p);
        ime_instance_free(i);
    }

    #[test]
    fn sf7_abi_tham_so_sai_va_abi_lech() {
        let (i, p) = (inst(), picker(IME_EPK_LAYOUT_GRID));
        let mut out = std::ptr::null_mut();
        assert_eq!(ime_emoji_picker_new_impl(9, &mut out), IME_ERR_INVALID_ARG);
        assert_eq!(
            ime_emoji_picker_open_impl(p, std::ptr::null()),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(ime_emoji_picker_open_impl(p, i), IME_OK);
        let mut buf = [0u8; 4];
        let mut n = 0usize;
        // phím lạ / code point hỏng
        assert_eq!(
            ime_emoji_picker_key_impl(p, i, 99, 0, buf.as_mut_ptr(), buf.len(), &mut n),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_emoji_picker_key_impl(p, i, IME_EPK_KEY_CHAR, 0xD800, buf.as_mut_ptr(), 4, &mut n),
            IME_ERR_INVALID_ARG
        );
        let mut v = ime_emoji_view_v1::new();
        v.abi_version = 7;
        assert_eq!(ime_emoji_picker_view_impl(p, &mut v), IME_ERR_ABI);
        // pick: sai chỉ số ⇒ REDRAW; bộ đệm 4 byte vừa 🚀 (4) + NUL? không ⇒ lỗi TRƯỚC khi ghi MRU.
        assert_eq!(
            ime_emoji_picker_pick_impl(p, i, 10_000, buf.as_mut_ptr(), buf.len(), &mut n),
            IME_EPK_REDRAW
        );
        let before = unsafe { (*i).engine.dream_rsi_snapshot() };
        assert_eq!(
            ime_emoji_picker_pick_impl(p, i, 0, buf.as_mut_ptr(), 1, &mut n),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            unsafe { (*i).engine.dream_rsi_snapshot() }.accepts,
            before.accepts,
            "thiếu chỗ ⇒ không ghi MRU/Dream RSI"
        );
        assert_eq!(view(p).open, 1, "vẫn mở");
        ime_emoji_picker_free_impl(p);
        ime_emoji_picker_free_impl(std::ptr::null_mut());
        ime_instance_free(i);
    }

    #[test]
    fn sf7_abi_nhom_cuon_va_view_luoi() {
        let (i, p) = (inst(), picker(IME_EPK_LAYOUT_GRID));
        assert_eq!(ime_emoji_picker_open_impl(p, i), IME_OK);
        assert_eq!(ime_emoji_picker_switch_category_impl(p, i, 1), IME_OK);
        let v = view(p);
        assert_eq!((v.category, v.first_visible), (1, 0));
        assert_eq!(v.count, v.total.min(40));
        assert_eq!(ime_emoji_picker_scroll_impl(p, 2), IME_OK);
        assert_eq!(view(p).first_visible, 16, "cuộn 2 hàng × 8 cột");
        let mut buf = [0u8; IME_EPK_TEXT_MAX];
        let mut n = 0usize;
        assert_eq!(
            ime_emoji_picker_key_impl(
                p,
                i,
                IME_EPK_KEY_TAB,
                0,
                buf.as_mut_ptr(),
                buf.len(),
                &mut n
            ),
            IME_EPK_REDRAW
        );
        assert_eq!(view(p).category, 2);
        assert_eq!(
            ime_emoji_picker_key_impl(
                p,
                i,
                IME_EPK_KEY_ESCAPE,
                0,
                buf.as_mut_ptr(),
                buf.len(),
                &mut n
            ),
            IME_EPK_CLOSED
        );
        assert_eq!(
            ime_emoji_picker_key_impl(
                p,
                i,
                IME_EPK_KEY_ENTER,
                0,
                buf.as_mut_ptr(),
                buf.len(),
                &mut n
            ),
            IME_EPK_NOT_OPEN
        );
        let v = view(p);
        assert_eq!(
            (v.open, v.query_len, v.count),
            (0, 0, 0),
            "G8: đóng là xoá ô tìm"
        );
        ime_emoji_picker_free_impl(p);
        ime_instance_free(i);
    }
}
