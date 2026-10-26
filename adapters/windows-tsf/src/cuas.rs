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
//! 2. Lúc giao: đọc chữ trước caret bằng `WM_IME_REQUEST/IMR_DOCUMENTFEED`
//!    (Scintilla, RichEdit… hỗ trợ); control họ EDIT không trả document feed
//!    (EDIT chuẩn, WinForms TextBox, Delphi TEdit/TMemo — R2-137, đo thật trên
//!    host 32-bit) ⇒ đọc bằng `EM_GETSEL` + `WM_GETTEXT`. Chỉ sửa khi đuôi đó
//!    **khớp đúng** đuôi câu engine đã phân tích (D12 — chống caret đã bị dời
//!    bằng chuột). Không đọc được ⇒ không sửa (fail-safe).
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
    GetClassNameW, IsWindow, IsWindowUnicode, PostMessageW, SendMessageW, WM_CHAR, WM_GETTEXT,
    WM_GETTEXTLENGTH, WM_KEYDOWN, WM_KEYUP,
};

/// `WM_IME_REQUEST` / `IMR_DOCUMENTFEED` (imm.h) — khai báo tại chỗ để không
/// kéo cả feature `Win32_UI_Input_Ime` chỉ vì một hằng và một struct.
const WM_IME_REQUEST: u32 = 0x0288;
const IMR_DOCUMENTFEED: usize = 7;
/// `EM_GETSEL`/`EM_SETSEL`/`EM_REPLACESEL` (winuser.h) — khai báo tại chỗ,
/// không kéo `Win32_UI_Controls`.
const EM_GETSEL: u32 = 0x00B0;
const EM_SETSEL: u32 = 0x00B1;
const EM_REPLACESEL: u32 = 0x00C2;
/// `IMR_RECONVERTSTRING` (imm.h) và thông điệp Scintilla (Scintilla.h).
const IMR_RECONVERTSTRING: usize = 4;
/// `EM_GETSELTEXT` (richedit.h: WM_USER + 62).
const EM_GETSELTEXT: u32 = 0x0400 + 62;
const SCI_GETCODEPAGE: u32 = 2137;
const SCI_GETSELTEXT: u32 = 2161;
const SCI_REPLACESEL: u32 = 2170;
/// Trần vùng chọn chuyển đổi (khớp `selection_check::MAX_TRANSCODE_CHARS` cỡ).
const MAX_SELECTION_UNITS: usize = 8 * 1024;
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

/// Việc CUAS chờ giao (hàng đợi `replay.rs`, chờ nhả phím bổ trợ).
pub enum Job {
    /// Sửa đuôi câu trước caret (trợ lý — R2-132/R2-137).
    Tail(TailFix),
    /// Thay vùng bôi đen bằng bản chuyển đổi (Ctrl+Shift+F9 — R2-139).
    Selection(SelectionFix),
}

/// Cách đọc/thay vùng chọn của control (R2-139).
#[derive(Clone, Copy, PartialEq, Eq)]
enum SelKind {
    /// Họ EDIT: `EM_GETSEL` + `WM_GETTEXT` / `EM_REPLACESEL`.
    Edit,
    /// Scintilla (Notepad++): `SCI_GETSELTEXT` / `SCI_REPLACESEL` (UTF-8).
    Scintilla,
    /// Control IME-aware khác: `IMR_RECONVERTSTRING` / gõ đè `WM_CHAR`.
    Reconvert,
}

/// Việc thay vùng chọn chờ giao.
pub struct SelectionFix {
    tid: u32,
    target: isize,
    kind: SelKind,
    /// Vùng chọn lúc bấm phím (UTF-16) — lúc giao phải còn y nguyên.
    original: Vec<u16>,
    replacement: Vec<u16>,
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
    let job = Job::Tail(TailFix {
        tid,
        target: target.0 as isize,
        original_tail: tail,
        replacement: replacement.encode_utf16().collect(),
    });
    let queued = crate::replay::schedule_tail_fix(job);
    crate::trace::event(tid, format_args!("assist cuas scheduled={queued}"));
    queued
}

/// R2-139: Ctrl+Shift+F9 ở app CUAS (Notepad++, ô EDIT/WinForms/Delphi) — text
/// store CUAS không chứa vùng chọn nên đường TSF (`TranscodeSession`) không đọc
/// được. Đọc vùng chọn thẳng từ control ngay lúc bấm phím (cùng thread UI, chỉ
/// đọc), chuyển đổi bằng `convert`, rồi hẹn THAY sau khi nhả phím bổ trợ (xác
/// minh lại vùng chọn lúc giao). `true` = phím đã được xử lý (kể cả khi không có
/// gì để đổi); `false` = không phải CUAS / không đọc được vùng chọn.
pub fn schedule_selection_convert(
    tid: u32,
    ctx: &ITfContext,
    convert: impl FnOnce(&str) -> Option<String>,
) -> bool {
    if !is_transitory(ctx) {
        return false;
    }
    // SAFETY: gọi trên thread UI của app (callback phím của TIP).
    let target = unsafe { GetFocus() };
    if target.is_invalid() || unsafe { !IsWindowUnicode(target).as_bool() } {
        return false;
    }
    let Some((kind, original)) = read_selection(target) else {
        crate::trace::event(tid, format_args!("transcode cuas skip: no selection"));
        return false;
    };
    if original.len() > MAX_SELECTION_UNITS {
        crate::trace::event(tid, format_args!("transcode cuas skip: selection too long"));
        return true;
    }
    let Ok(text) = String::from_utf16(&original) else {
        return true;
    };
    let Some(converted) = convert(&text).filter(|c| *c != text) else {
        crate::trace::event(tid, format_args!("transcode cuas: nothing to change"));
        return true;
    };
    let job = Job::Selection(SelectionFix {
        tid,
        target: target.0 as isize,
        kind,
        original,
        replacement: converted.encode_utf16().collect(),
    });
    let queued = crate::replay::schedule_tail_fix(job);
    crate::trace::event(tid, format_args!("transcode cuas scheduled={queued}"));
    queued
}

/// Đọc vùng chọn (UTF-16) của control đang focus; `None` = không có / không đọc được.
fn read_selection(target: HWND) -> Option<(SelKind, Vec<u16>)> {
    // RichEdit: vị trí EM_GETSEL tính đoạn bằng CR đơn (và gồm dấu kết đoạn cuối)
    // còn WM_GETTEXT trả CRLF ⇒ cắt từ WM_GETTEXT lệch — lấy thẳng EM_GETSELTEXT.
    if class_name(target).starts_with("richedit") {
        return richedit_selection(target).map(|s| (SelKind::Edit, s));
    }
    if is_edit_family(target) {
        let (start, end) = edit_selection(target)?;
        if start == end {
            return None;
        }
        let text = window_text(target)?;
        let end = end.min(text.len());
        return text
            .get(start..end)
            .filter(|s| !s.is_empty())
            .map(|s| (SelKind::Edit, s.to_vec()));
    }
    if class_name(target) == "scintilla" {
        return scintilla_selection(target).map(|s| (SelKind::Scintilla, s));
    }
    reconvert_selection(target).map(|s| (SelKind::Reconvert, s))
}

fn class_name(target: HWND) -> String {
    let mut buf = [0u16; 128];
    // SAFETY: đọc tên class vào buffer cục bộ.
    let n = unsafe { GetClassNameW(target, &mut buf) };
    let n = usize::try_from(n).unwrap_or(0).min(buf.len());
    String::from_utf16_lossy(&buf[..n]).to_lowercase()
}

/// RichEdit: chữ đang chọn qua `EM_GETSELTEXT` (UTF-16, kết thúc NUL).
fn richedit_selection(target: HWND) -> Option<Vec<u16>> {
    let (start, end) = edit_selection(target)?;
    if start == end || end - start > MAX_SELECTION_UNITS {
        return None;
    }
    let mut buf = vec![0u16; end - start + 2];
    // SAFETY: buffer đủ (end - start) ký tự + NUL; SendMessage cùng thread UI.
    let got = unsafe {
        SendMessageW(
            target,
            EM_GETSELTEXT,
            None,
            Some(LPARAM(buf.as_mut_ptr() as isize)),
        )
    }
    .0;
    let got = usize::try_from(got).ok()?.min(buf.len());
    (got > 0).then(|| buf[..got].to_vec())
}

/// `EM_GETSEL` (vị trí UTF-16 của cửa sổ Unicode).
fn edit_selection(target: HWND) -> Option<(usize, usize)> {
    let mut start = 0u32;
    let mut end = 0u32;
    // SAFETY: SendMessage cùng thread UI; con trỏ tới biến cục bộ sống hết lời gọi.
    unsafe {
        SendMessageW(
            target,
            EM_GETSEL,
            Some(WPARAM(&mut start as *mut u32 as usize)),
            Some(LPARAM(&mut end as *mut u32 as isize)),
        );
    }
    (start <= end).then_some((start as usize, end as usize))
}

/// Toàn bộ chữ của control (`WM_GETTEXT`, tối đa `MAX_FEED_BYTES`).
fn window_text(target: HWND) -> Option<Vec<u16>> {
    // SAFETY: SendMessage cùng thread UI.
    let len = unsafe { SendMessageW(target, WM_GETTEXTLENGTH, None, None) }.0;
    let len = usize::try_from(len).ok()?;
    if len > MAX_FEED_BYTES / 2 {
        return None;
    }
    let mut text = vec![0u16; len + 1];
    // SAFETY: buffer len+1 code unit, WM_GETTEXT ghi tối đa wParam ký tự kể cả NUL.
    let got = unsafe {
        SendMessageW(
            target,
            WM_GETTEXT,
            Some(WPARAM(text.len())),
            Some(LPARAM(text.as_mut_ptr() as isize)),
        )
    }
    .0;
    let got = usize::try_from(got).ok()?.min(len);
    text.truncate(got);
    Some(text)
}

/// Scintilla: `SCI_GETSELTEXT` (byte theo code page tài liệu — chỉ UTF-8).
fn scintilla_selection(target: HWND) -> Option<Vec<u16>> {
    // SAFETY: SendMessage cùng thread UI; buffer cục bộ đủ `need` + NUL.
    unsafe {
        if SendMessageW(target, SCI_GETCODEPAGE, None, None).0 != 65001 {
            return None;
        }
        let need = SendMessageW(target, SCI_GETSELTEXT, None, None).0;
        let need = usize::try_from(need).ok()?;
        if need == 0 || need > MAX_FEED_BYTES {
            return None;
        }
        let mut bytes = vec![0u8; need + 1];
        SendMessageW(
            target,
            SCI_GETSELTEXT,
            None,
            Some(LPARAM(bytes.as_mut_ptr() as isize)),
        );
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let text = std::str::from_utf8(&bytes[..end]).ok()?;
        (!text.is_empty()).then(|| text.encode_utf16().collect())
    }
}

/// Control IME-aware: vùng chọn qua `WM_IME_REQUEST/IMR_RECONVERTSTRING`.
fn reconvert_selection(target: HWND) -> Option<Vec<u16>> {
    // SAFETY: như `text_before_caret` — SendMessage cùng thread, buffer canh 4 byte.
    let need = unsafe {
        SendMessageW(
            target,
            WM_IME_REQUEST,
            Some(WPARAM(IMR_RECONVERTSTRING)),
            None,
        )
    }
    .0;
    let header = std::mem::size_of::<ReconvertString>();
    let need = usize::try_from(need).ok()?;
    if need <= header || need > MAX_FEED_BYTES {
        return None;
    }
    let mut buf = vec![0u32; need.div_ceil(4) + 1];
    let rc = buf.as_mut_ptr().cast::<ReconvertString>();
    // SAFETY: buf đủ chỗ cho header.
    unsafe {
        (*rc).dw_size = need as u32;
    }
    let got = unsafe {
        SendMessageW(
            target,
            WM_IME_REQUEST,
            Some(WPARAM(IMR_RECONVERTSTRING)),
            Some(LPARAM(rc as isize)),
        )
    }
    .0;
    if got == 0 {
        return None;
    }
    // SAFETY: header do app điền; offset được kiểm trong biên `need`.
    let info = unsafe { *rc };
    let str_off = info.dw_str_offset as usize;
    let str_len = info.dw_str_len as usize;
    let sel_off = info.dw_comp_str_offset as usize / 2;
    let sel_len = info.dw_comp_str_len as usize;
    if !str_off.is_multiple_of(2) || str_off + str_len * 2 > need || sel_off + sel_len > str_len {
        return None;
    }
    // SAFETY: vùng [str_off, str_off + 2*str_len) nằm trong buffer `need` byte.
    let text = unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr().cast::<u8>().add(str_off).cast::<u16>(),
            str_len,
        )
    };
    (sel_len > 0).then(|| text[sel_off..sel_off + sel_len].to_vec())
}

/// Thay vùng chọn (lúc giao, sau khi nhả phím bổ trợ) — xác minh lại trước.
fn run_selection(job: &SelectionFix) {
    let target = HWND(job.target as *mut _);
    // SAFETY: HWND capture từ GetFocus cùng thread; IsWindow chặn control đã huỷ.
    if unsafe { !IsWindow(Some(target)).as_bool() } {
        crate::trace::event(job.tid, format_args!("transcode cuas skip: gone"));
        return;
    }
    match read_selection(target) {
        Some((kind, now)) if kind == job.kind && now == job.original => {}
        _ => {
            crate::trace::event(
                job.tid,
                format_args!("transcode cuas skip: selection changed"),
            );
            return;
        }
    }
    // SAFETY: SendMessage/post cùng thread UI (ngoài callback TSF); buffer NUL sống
    // hết lời gọi.
    unsafe {
        match job.kind {
            SelKind::Edit => {
                let mut text = job.replacement.clone();
                text.push(0);
                SendMessageW(
                    target,
                    EM_REPLACESEL,
                    Some(WPARAM(1)),
                    Some(LPARAM(text.as_ptr() as isize)),
                );
            }
            SelKind::Scintilla => {
                let mut utf8 = String::from_utf16_lossy(&job.replacement).into_bytes();
                utf8.push(0);
                SendMessageW(
                    target,
                    SCI_REPLACESEL,
                    None,
                    Some(LPARAM(utf8.as_ptr() as isize)),
                );
            }
            SelKind::Reconvert => {
                for &unit in &job.replacement {
                    post(target, WM_CHAR, WPARAM(unit as usize), LPARAM(1));
                }
            }
        }
    }
    crate::trace::event(
        job.tid,
        format_args!("transcode cuas applied: {} units", job.original.len()),
    );
}

/// Giao việc CUAS (gọi từ cửa sổ message của replay.rs, ngoài edit session).
pub fn run(job: &Job) {
    match job {
        Job::Tail(tail) => run_tail(tail),
        Job::Selection(selection) => run_selection(selection),
    }
}

/// Giao việc sửa đuôi câu.
fn run_tail(job: &TailFix) {
    let target = HWND(job.target as *mut _);
    // SAFETY: HWND capture từ GetFocus cùng thread; IsWindow chặn control đã huỷ.
    if unsafe { !IsWindow(Some(target)).as_bool() } {
        crate::trace::event(job.tid, format_args!("assist cuas skip: gone"));
        return;
    }
    // Họ EDIT (R2-137): đọc + sửa bằng EM_* — chọn đúng đuôi rồi EM_REPLACESEL
    // (một thao tác, Ctrl+Z của control hoàn tác được). Backspace post kiểu
    // Scintilla KHÔNG dùng được ở đây: EDIT xoá ở WM_CHAR 0x08 mà TranslateMessage
    // xếp SAU các WM_CHAR đã post ⇒ chữ mới bị chèn trước rồi bị xoá (đo thật).
    if let Some(before) = edit_text_before_caret(target) {
        if !before.ends_with(&job.original_tail) {
            crate::trace::event(
                job.tid,
                format_args!("assist cuas skip: tail mismatch ({} units)", before.len()),
            );
            return;
        }
        let caret = before.len();
        let from = caret - job.original_tail.len();
        let mut text = job.replacement.clone();
        text.push(0);
        // SAFETY: SendMessage cùng thread UI (ngoài callback TSF); buffer NUL sống
        // hết lời gọi.
        unsafe {
            SendMessageW(
                target,
                EM_SETSEL,
                Some(WPARAM(from)),
                Some(LPARAM(caret as isize)),
            );
            SendMessageW(
                target,
                EM_REPLACESEL,
                Some(WPARAM(1)),
                Some(LPARAM(text.as_ptr() as isize)),
            );
        }
        crate::trace::event(
            job.tid,
            format_args!(
                "assist cuas applied (edit): -{} +{}",
                job.original_tail.len(),
                job.replacement.len()
            ),
        );
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

/// Bỏ việc (trace lý do — không chữ, S2).
pub fn give_up(job: &Job, why: &str) {
    let tid = match job {
        Job::Tail(tail) => tail.tid,
        Job::Selection(selection) => selection.tid,
    };
    crate::trace::event(tid, format_args!("cuas skip: {why}"));
}

fn post(target: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
    // SAFETY: post tới control của app trong cùng process; lỗi bỏ qua.
    let _ = unsafe { PostMessageW(Some(target), msg, wparam, lparam) };
}

/// Control thuộc họ EDIT (EDIT chuẩn, `WindowsForms10.EDIT…`, `RichEdit20W`,
/// Delphi `TEdit`/`TMemo`) — hiểu `EM_GETSEL`/`WM_GETTEXT` theo nghĩa EDIT.
fn is_edit_family(target: HWND) -> bool {
    let class = class_name(target);
    class.contains("edit") || class.contains("memo")
}

/// R2-137: chữ trước caret của control họ EDIT (không trả document feed) qua
/// `EM_GETSEL` (vị trí UTF-16 của cửa sổ Unicode) + `WM_GETTEXT`. Có vùng chọn
/// (start ≠ end) ⇒ None (precondition "không bôi đen").
fn edit_text_before_caret(target: HWND) -> Option<Vec<u16>> {
    if !is_edit_family(target) {
        return None;
    }
    let (start, end) = edit_selection(target)?;
    if start != end {
        return None;
    }
    let text = window_text(target)?;
    text.get(..end.min(text.len())).map(<[u16]>::to_vec)
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
