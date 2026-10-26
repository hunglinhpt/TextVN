// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-132 — sửa đuôi câu ở app IMM32 chạy qua CUAS (Notepad++, WinForms, Delphi…).
//!
//! Với CUAS, text store của TSF chỉ chứa composition: `ShiftStart` không lùi qua
//! chữ đã commit (đo thật trên Notepad++: `shift 0/-12`) nên mọi kiểu sửa lùi
//! của trợ lý (A2 chọn gợi ý, A3 "Sửa tất cả", phím tắt) không làm gì. Đường
//! dự phòng ở đây không dùng range TSF:
//!
//! 1. Hẹn việc qua hàng đợi `replay.rs` (đi vòng vài lượt message) để app đã
//!    nhận xong kết quả composition vừa commit.
//! 2. Lúc giao: đọc chữ trên dòng hiện tại tới caret bằng
//!    `WM_IME_REQUEST/IMR_DOCUMENTFEED` (Scintilla, RichEdit… hỗ trợ) và chỉ sửa
//!    khi đuôi đó **khớp đúng** đuôi câu engine đã phân tích (D12 — chống caret
//!    đã bị dời bằng chuột). App không trả document feed ⇒ không sửa (fail-safe).
//! 3. Post `VK_BACK` × số glyph rồi `WM_CHAR` từng code unit của bản sửa vào
//!    chính control đang focus — cùng process, cùng thread UI (như replay.rs:
//!    không `SendInput`, không hook).
//!
//! Chỉ cửa sổ Unicode (`IsWindowUnicode`): `WM_CHAR` của cửa sổ ANSI làm mất
//! chữ Việt. Không ghi nội dung chữ vào trace (S2) — chỉ lý do + số đếm.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetFocus, MapVirtualKeyW, MAPVK_VK_TO_VSC, VK_BACK,
};
use windows::Win32::UI::TextServices::{ITfContext, TF_SS_TRANSITORY};
use windows::Win32::UI::WindowsAndMessaging::{
    IsWindow, IsWindowUnicode, PostMessageW, SendMessageW, WM_CHAR, WM_KEYDOWN, WM_KEYUP,
};

/// `WM_IME_REQUEST` / `IMR_DOCUMENTFEED` (imm.h) — khai báo tại chỗ để không
/// kéo cả feature `Win32_UI_Input_Ime` chỉ vì một hằng và một struct.
const WM_IME_REQUEST: u32 = 0x0288;
const IMR_DOCUMENTFEED: usize = 7;
/// Trần kích thước document feed (một dòng): quá thì không sửa.
const MAX_FEED_BYTES: usize = 64 * 1024;
/// Trần số Backspace một lần sửa (câu trợ lý ≤ 256 ký tự).
const MAX_BACKSPACES: usize = 512;

/// `RECONVERTSTRING` (imm.h) — 8 DWORD, chuỗi UTF-16 nằm sau header.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ReconvertString {
    dw_size: u32,
    dw_version: u32,
    dw_str_len: u32,
    dw_str_offset: u32,
    dw_comp_str_len: u32,
    dw_comp_str_offset: u32,
    dw_target_str_len: u32,
    dw_target_str_offset: u32,
}

/// Việc sửa đuôi câu chờ giao (giữ trong hàng đợi `replay.rs` vài lượt message).
pub struct TailFix {
    tid: u32,
    target: isize,
    /// Đuôi câu gốc (UTF-16) phải nằm ngay trước caret lúc giao.
    original_tail: Vec<u16>,
    /// Bản sửa (UTF-16) gõ lại sau khi lùi.
    replacement: Vec<u16>,
}

/// Context là CUAS (app IMM32 qua lớp tương thích) — TSF đánh dấu transitory.
pub fn is_transitory(ctx: &ITfContext) -> bool {
    // SAFETY: ctx hợp lệ trong callback/edit session của TSF.
    unsafe { ctx.GetStatus() }
        .map(|st| st.dwStaticFlags & TF_SS_TRANSITORY != 0)
        .unwrap_or(false)
}

/// Hẹn sửa đuôi câu qua đường CUAS. `true` = đã nhận việc (kết quả thật quyết
/// định lúc giao, có xác minh document feed); `false` = không áp dụng được
/// (không phải CUAS, cửa sổ không Unicode, số glyph lệch code unit…).
pub fn schedule_tail_fix(
    tid: u32,
    ctx: &ITfContext,
    original_tail: &str,
    delete_glyphs: u32,
    replacement: &str,
) -> bool {
    if !is_transitory(ctx) {
        return false;
    }
    let tail: Vec<u16> = original_tail.encode_utf16().collect();
    // Một Backspace = một ký tự: chỉ sửa khi glyph của core trùng code unit
    // (bảng mã Unicode dựng sẵn, không surrogate) — lệch thì không đoán.
    if tail.is_empty() || tail.len() != delete_glyphs as usize || tail.len() > MAX_BACKSPACES {
        crate::trace::event(tid, format_args!("assist cuas skip: glyphs"));
        return false;
    }
    // SAFETY: gọi trên thread UI của app (edit session của TIP).
    let target = unsafe { GetFocus() };
    if target.is_invalid() || unsafe { !IsWindowUnicode(target).as_bool() } {
        crate::trace::event(tid, format_args!("assist cuas skip: target"));
        return false;
    }
    let job = TailFix {
        tid,
        target: target.0 as isize,
        original_tail: tail,
        replacement: replacement.encode_utf16().collect(),
    };
    let queued = crate::replay::schedule_tail_fix(job);
    crate::trace::event(tid, format_args!("assist cuas scheduled={queued}"));
    queued
}

/// Giao việc (gọi từ cửa sổ message của replay.rs, ngoài edit session).
pub fn run(job: &TailFix) {
    let target = HWND(job.target as *mut _);
    // SAFETY: HWND capture từ GetFocus cùng thread; IsWindow chặn control đã huỷ.
    if unsafe { !IsWindow(Some(target)).as_bool() } {
        crate::trace::event(job.tid, format_args!("assist cuas skip: gone"));
        return;
    }
    let Some(before) = text_before_caret(target) else {
        crate::trace::event(job.tid, format_args!("assist cuas skip: no feed"));
        return;
    };
    if !before.ends_with(&job.original_tail) {
        crate::trace::event(
            job.tid,
            format_args!("assist cuas skip: tail mismatch ({} units)", before.len()),
        );
        return;
    }
    // SAFETY: MapVirtualKeyW thuần tra bảng layout.
    let scan = unsafe { MapVirtualKeyW(u32::from(VK_BACK.0), MAPVK_VK_TO_VSC) } as isize;
    let down = LPARAM(1 | (scan << 16));
    let up = LPARAM(1 | (scan << 16) | (0b11 << 30));
    for _ in 0..job.original_tail.len() {
        post(target, WM_KEYDOWN, WPARAM(VK_BACK.0 as usize), down);
        post(target, WM_KEYUP, WPARAM(VK_BACK.0 as usize), up);
    }
    for &unit in &job.replacement {
        post(target, WM_CHAR, WPARAM(unit as usize), LPARAM(1));
    }
    crate::trace::event(
        job.tid,
        format_args!(
            "assist cuas applied: -{} +{}",
            job.original_tail.len(),
            job.replacement.len()
        ),
    );
}

/// Bỏ việc sửa (trace lý do — không chữ, S2).
pub fn give_up(job: &TailFix, why: &str) {
    crate::trace::event(job.tid, format_args!("assist cuas skip: {why}"));
}

fn post(target: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
    // SAFETY: post tới control của app trong cùng process; lỗi bỏ qua.
    let _ = unsafe { PostMessageW(Some(target), msg, wparam, lparam) };
}

/// Chữ trên dòng hiện tại tới caret (UTF-16) qua `IMR_DOCUMENTFEED`.
fn text_before_caret(target: HWND) -> Option<Vec<u16>> {
    // SAFETY: SendMessage cùng thread UI (gọi thẳng WndProc của control); lần
    // đầu lParam = 0 ⇒ app trả kích thước cần cấp.
    let need =
        unsafe { SendMessageW(target, WM_IME_REQUEST, Some(WPARAM(IMR_DOCUMENTFEED)), None) }.0;
    let header = std::mem::size_of::<ReconvertString>();
    let need = usize::try_from(need).ok()?;
    if need <= header || need > MAX_FEED_BYTES {
        return None;
    }
    // Vec<u32> để buffer canh 4 byte cho header DWORD.
    let mut buf = vec![0u32; need.div_ceil(4) + 1];
    let rc = buf.as_mut_ptr().cast::<ReconvertString>();
    // SAFETY: buf đủ chỗ cho header (need > header) và canh 4 byte.
    unsafe {
        (*rc).dw_size = need as u32;
        (*rc).dw_version = 0;
    }
    let got = unsafe {
        SendMessageW(
            target,
            WM_IME_REQUEST,
            Some(WPARAM(IMR_DOCUMENTFEED)),
            Some(LPARAM(rc as isize)),
        )
    }
    .0;
    if got == 0 {
        return None;
    }
    // SAFETY: app đã điền header; mọi offset được kiểm trong biên `need` dưới đây.
    let info = unsafe { *rc };
    let str_off = info.dw_str_offset as usize;
    let str_len = info.dw_str_len as usize;
    let caret = info.dw_comp_str_offset as usize / 2;
    if !str_off.is_multiple_of(2) || str_off + str_len * 2 > need || caret > str_len {
        return None;
    }
    // SAFETY: vùng [str_off, str_off + 2*str_len) nằm trong buffer `need` byte.
    let text = unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr().cast::<u8>().add(str_off).cast::<u16>(),
            str_len,
        )
    };
    Some(text[..caret].to_vec())
}
