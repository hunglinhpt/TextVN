// SPDX-License-Identifier: GPL-3.0-or-later
//! C API cho **bảng cài đặt** (`ffi/include/textvn_settings.h`) — tách khỏi ABI engine P0-2.
//!
//! Khác ABI engine có chủ đích: API này **có I/O file** và **cấp phát chuỗi trả về**
//! (giải phóng bằng `ime_settings_string_free`). Nó chỉ dành cho tiến trình UI
//! (`textvn-settings` trên Linux) và các adapter khi ghi `state.json`; không bao giờ
//! được gọi trong đường xử lý phím nóng.
//!
//! Logic thật nằm ở `textvn_config::SettingsDoc` — cùng code mà tray Windows dùng, nên
//! hai nền tảng đọc/ghi config giống hệt nhau (vá từng khoá, giữ khoá lạ, ghi nguyên tử).

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::OnceLock;

use textvn_config::{DocError, DocKind, SettingsDoc};

use crate::{IME_ERR_CONFIG, IME_ERR_INTERNAL, IME_ERR_INVALID_ARG, IME_OK};

pub const IME_SETTINGS_CONFIG: i32 = 0;
pub const IME_SETTINGS_STATE: i32 = 1;

/// Handle mờ phía C (tên theo header C).
#[allow(non_camel_case_types)]
pub struct ime_settings {
    doc: SettingsDoc,
}

fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

fn owned(s: String) -> *mut c_char {
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn doc_err(e: DocError) -> i32 {
    match e {
        DocError::Io => IME_ERR_INTERNAL,
        DocError::UnknownKey => IME_ERR_INVALID_ARG,
        DocError::BadValue => IME_ERR_CONFIG,
    }
}

/// Đọc `path` (UTF-8). Luôn trả handle khi tham số hợp lệ — file chưa có/hỏng → tài liệu
/// rỗng (xem `ime_settings_was_corrupt`). NULL khi tham số sai.
#[no_mangle]
pub extern "C" fn ime_settings_load(path: *const c_char, kind: i32) -> *mut ime_settings {
    let Some(path) = cstr(path) else {
        return std::ptr::null_mut();
    };
    let kind = match kind {
        IME_SETTINGS_CONFIG => DocKind::Config,
        IME_SETTINGS_STATE => DocKind::State,
        _ => return std::ptr::null_mut(),
    };
    catch_unwind(|| {
        Box::into_raw(Box::new(ime_settings {
            doc: SettingsDoc::load(&PathBuf::from(path), kind),
        }))
    })
    .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ime_settings_free(s: *mut ime_settings) {
    if !s.is_null() {
        drop(unsafe { Box::from_raw(s) });
    }
}

/// 1 nếu file gốc tồn tại nhưng hỏng (lần lưu đầu sẽ sao lưu nó thành `.bak`).
#[no_mangle]
pub extern "C" fn ime_settings_was_corrupt(s: *const ime_settings) -> i32 {
    match unsafe { s.as_ref() } {
        Some(s) => s.doc.was_corrupt() as i32,
        None => 0,
    }
}

/// 1/0 = giá trị (thiếu trong file → mặc định schema); -1 = khoá không có / sai kiểu.
#[no_mangle]
pub extern "C" fn ime_settings_get_bool(s: *const ime_settings, key: *const c_char) -> i32 {
    let (Some(s), Some(key)) = (unsafe { s.as_ref() }, cstr(key)) else {
        return -1;
    };
    catch_unwind(AssertUnwindSafe(|| match s.doc.get_bool(key) {
        Some(v) => v as i32,
        None => -1,
    }))
    .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn ime_settings_set_bool(
    s: *mut ime_settings,
    key: *const c_char,
    value: i32,
) -> i32 {
    let (Some(s), Some(key)) = (unsafe { s.as_mut() }, cstr(key)) else {
        return IME_ERR_INVALID_ARG;
    };
    catch_unwind(AssertUnwindSafe(|| match s.doc.set_bool(key, value != 0) {
        Ok(()) => IME_OK,
        Err(e) => doc_err(e),
    }))
    .unwrap_or(IME_ERR_INTERNAL)
}

/// Chuỗi mới cấp phát (giải phóng bằng `ime_settings_string_free`) hoặc NULL.
#[no_mangle]
pub extern "C" fn ime_settings_get_str(s: *const ime_settings, key: *const c_char) -> *mut c_char {
    let (Some(s), Some(key)) = (unsafe { s.as_ref() }, cstr(key)) else {
        return std::ptr::null_mut();
    };
    catch_unwind(AssertUnwindSafe(|| {
        s.doc
            .get_str(key)
            .map(owned)
            .unwrap_or(std::ptr::null_mut())
    }))
    .unwrap_or(std::ptr::null_mut())
}

/// Đặt khoá chuỗi (enum `method`, `output_charset`, …). Giá trị ngoài schema →
/// `IME_ERR_CONFIG`, tài liệu giữ nguyên.
#[no_mangle]
pub extern "C" fn ime_settings_set_str(
    s: *mut ime_settings,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    let (Some(s), Some(key), Some(value)) = (unsafe { s.as_mut() }, cstr(key), cstr(value)) else {
        return IME_ERR_INVALID_ARG;
    };
    catch_unwind(AssertUnwindSafe(|| match s.doc.set_str(key, value) {
        Ok(()) => IME_OK,
        Err(e) => doc_err(e),
    }))
    .unwrap_or(IME_ERR_INTERNAL)
}

/// Bảng gõ tắt dạng text `gõ tắt = nội dung` (mỗi dòng một mục).
#[no_mangle]
pub extern "C" fn ime_settings_macros_text(s: *const ime_settings) -> *mut c_char {
    let Some(s) = (unsafe { s.as_ref() }) else {
        return std::ptr::null_mut();
    };
    catch_unwind(AssertUnwindSafe(|| owned(s.doc.macros_text()))).unwrap_or(std::ptr::null_mut())
}

/// `IME_OK`, hoặc mã lỗi dòng > 0 (`ime_settings_macro_error_message`) kèm `*bad_line`
/// (đếm từ 1). Có lỗi → bảng gõ tắt cũ giữ nguyên.
#[no_mangle]
pub extern "C" fn ime_settings_set_macros_text(
    s: *mut ime_settings,
    text: *const c_char,
    bad_line: *mut u32,
) -> i32 {
    let (Some(s), Some(text)) = (unsafe { s.as_mut() }, cstr(text)) else {
        return IME_ERR_INVALID_ARG;
    };
    catch_unwind(AssertUnwindSafe(|| match s.doc.set_macros_text(text) {
        Ok(()) => IME_OK,
        Err(e) => {
            if !bad_line.is_null() {
                unsafe { *bad_line = u32::try_from(e.line).unwrap_or(u32::MAX) };
            }
            e.kind.code()
        }
    }))
    .unwrap_or(IME_ERR_INTERNAL)
}

/// Thông báo tiếng Việt (UTF-8, sống đến hết tiến trình) cho mã lỗi của
/// `ime_settings_set_macros_text`.
#[no_mangle]
pub extern "C" fn ime_settings_macro_error_message(code: i32) -> *const c_char {
    use textvn_config::macro_text::MacroLineErrorKind as K;
    static MESSAGES: OnceLock<Vec<(i32, CString)>> = OnceLock::new();
    let table = MESSAGES.get_or_init(|| {
        K::ALL
            .iter()
            .filter_map(|k| Some((k.code(), CString::new(k.message_vi()).ok()?)))
            .collect()
    });
    table
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, m)| m.as_ptr())
        .unwrap_or(c"lỗi không xác định".as_ptr())
}

/// Từ điển EN dạng text (mỗi dòng một từ) — 0.2.13, parity Linux với
/// "Từ điển EN..." trên Windows: từ trong danh sách được engine giữ nguyên
/// khi gõ (docs/specs/language-detection.md §2 cấp 6c).
#[no_mangle]
pub extern "C" fn ime_settings_english_words_text(s: *const ime_settings) -> *mut c_char {
    let Some(s) = (unsafe { s.as_ref() }) else {
        return std::ptr::null_mut();
    };
    catch_unwind(AssertUnwindSafe(|| owned(s.doc.english_words_text())))
        .unwrap_or(std::ptr::null_mut())
}

/// Chuẩn hoá + thay `english_words[]` (trim/lowercase/ASCII/dedupe — cùng quy
/// tắc ba nền tảng). `IME_OK` hoặc `IME_ERR_CONFIG`; lỗi → danh sách cũ giữ nguyên.
#[no_mangle]
pub extern "C" fn ime_settings_set_english_words_text(
    s: *mut ime_settings,
    text: *const c_char,
) -> i32 {
    let (Some(s), Some(text)) = (unsafe { s.as_mut() }, cstr(text)) else {
        return IME_ERR_INVALID_ARG;
    };
    catch_unwind(AssertUnwindSafe(|| {
        match s.doc.set_english_words_text(text) {
            Ok(()) => IME_OK,
            Err(_) => IME_ERR_CONFIG,
        }
    }))
    .unwrap_or(IME_ERR_INTERNAL)
}

/// Nút "Mặc định": mọi tuỳ chọn về mặc định, giữ gõ tắt/emoji/từ tiếng Anh.
#[no_mangle]
pub extern "C" fn ime_settings_reset_defaults(s: *mut ime_settings) {
    if let Some(s) = unsafe { s.as_mut() } {
        let _ = catch_unwind(AssertUnwindSafe(|| s.doc.reset_defaults()));
    }
}

/// Ghi nguyên tử vào `path` (tạo thư mục cha 0700, file 0600).
#[no_mangle]
pub extern "C" fn ime_settings_save(s: *mut ime_settings, path: *const c_char) -> i32 {
    let (Some(s), Some(path)) = (unsafe { s.as_mut() }, cstr(path)) else {
        return IME_ERR_INVALID_ARG;
    };
    catch_unwind(AssertUnwindSafe(|| {
        match s.doc.save(&PathBuf::from(path)) {
            Ok(()) => IME_OK,
            Err(e) => doc_err(e),
        }
    }))
    .unwrap_or(IME_ERR_INTERNAL)
}

#[no_mangle]
pub extern "C" fn ime_settings_string_free(p: *mut c_char) {
    if !p.is_null() {
        drop(unsafe { CString::from_raw(p) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = include_str!("../include/textvn_settings.h");
    const SOURCE: &str = include_str!("settings.rs");

    fn c(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    fn take(p: *mut c_char) -> String {
        assert!(!p.is_null());
        let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
        ime_settings_string_free(p);
        s
    }

    /// Header chép tay phải khai báo đúng và đủ các hàm export ở đây, cùng thứ tự.
    #[test]
    fn header_matches_exports() {
        let names = |src: &str, needle: &str| -> Vec<String> {
            src.match_indices(needle)
                .map(|(i, _)| {
                    let rest = &src[i + needle.len()..];
                    let end = rest
                        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                        .unwrap_or(rest.len());
                    format!("ime_settings_{}", &rest[..end])
                })
                .collect()
        };
        let rust = names(SOURCE, "pub extern \"C\" fn ime_settings_");
        let header: Vec<String> = names(HEADER, " ime_settings_")
            .into_iter()
            .chain(names(HEADER, "*ime_settings_"))
            .collect::<Vec<_>>();
        let mut header_sorted = header.clone();
        header_sorted.sort();
        header_sorted.dedup();
        let mut rust_sorted = rust.clone();
        rust_sorted.sort();
        assert_eq!(header_sorted, rust_sorted);
        assert!(HEADER.contains(&format!(
            "#define IME_SETTINGS_CONFIG {IME_SETTINGS_CONFIG}"
        )));
        assert!(HEADER.contains(&format!("#define IME_SETTINGS_STATE  {IME_SETTINGS_STATE}")));
    }

    #[test]
    fn c_api_round_trip() {
        let dir = std::env::temp_dir().join(format!("textvn-ffi-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.json");
        let cpath = c(path.to_str().unwrap());

        let s = ime_settings_load(cpath.as_ptr(), IME_SETTINGS_CONFIG);
        assert!(!s.is_null());
        assert_eq!(ime_settings_was_corrupt(s), 0);
        assert_eq!(ime_settings_get_bool(s, c("quick_telex").as_ptr()), 0);
        assert_eq!(ime_settings_get_bool(s, c("nope").as_ptr()), -1);
        assert_eq!(
            ime_settings_set_bool(s, c("quick_telex").as_ptr(), 1),
            IME_OK
        );
        assert_eq!(
            ime_settings_set_bool(s, c("nope").as_ptr(), 1),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_settings_set_str(s, c("method").as_ptr(), c("dvorak").as_ptr()),
            IME_ERR_CONFIG
        );
        assert_eq!(
            ime_settings_set_str(s, c("method").as_ptr(), c("vni").as_ptr()),
            IME_OK
        );
        assert_eq!(take(ime_settings_get_str(s, c("method").as_ptr())), "vni");

        let mut bad = 0u32;
        let rc = ime_settings_set_macros_text(s, c("vn = Việt Nam\nsai").as_ptr(), &mut bad);
        assert_eq!(bad, 2);
        let msg = unsafe { CStr::from_ptr(ime_settings_macro_error_message(rc)) };
        assert!(msg.to_str().unwrap().contains('='));
        assert_eq!(
            ime_settings_set_macros_text(s, c("vn = Việt Nam").as_ptr(), &mut bad),
            IME_OK
        );
        assert_eq!(take(ime_settings_macros_text(s)), "vn = Việt Nam\n");
        assert_eq!(ime_settings_save(s, cpath.as_ptr()), IME_OK);
        ime_settings_free(s);

        let cfg = textvn_config::parse_config(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(cfg.quick_telex);
        assert_eq!(cfg.method, textvn_config::Method::Vni);
        assert_eq!(cfg.macros[0].expand, "Việt Nam");

        let s = ime_settings_load(cpath.as_ptr(), IME_SETTINGS_CONFIG);
        ime_settings_reset_defaults(s);
        assert_eq!(ime_settings_get_bool(s, c("quick_telex").as_ptr()), 0);
        assert_eq!(take(ime_settings_macros_text(s)), "vn = Việt Nam\n");
        ime_settings_free(s);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn null_arguments_never_crash() {
        assert!(ime_settings_load(std::ptr::null(), 0).is_null());
        assert!(ime_settings_load(c("/x").as_ptr(), 7).is_null());
        ime_settings_free(std::ptr::null_mut());
        ime_settings_string_free(std::ptr::null_mut());
        ime_settings_reset_defaults(std::ptr::null_mut());
        assert_eq!(
            ime_settings_get_bool(std::ptr::null(), std::ptr::null()),
            -1
        );
        assert_eq!(
            ime_settings_set_bool(std::ptr::null_mut(), std::ptr::null(), 1),
            IME_ERR_INVALID_ARG
        );
        assert!(ime_settings_get_str(std::ptr::null(), std::ptr::null()).is_null());
        assert!(ime_settings_macros_text(std::ptr::null()).is_null());
    }

    /// Round-trip từ điển EN (0.2.13): set qua text → normalize → đọc lại →
    /// save → đọc file thấy `english_words` đúng; null-safe.
    #[test]
    fn english_words_text_round_trip() {
        let dir = std::env::temp_dir().join(format!("tvn_ew_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        let cpath = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let s = ime_settings_load(cpath.as_ptr(), IME_SETTINGS_CONFIG);
        assert_eq!(
            ime_settings_set_english_words_text(
                s,
                c("Text\n  file_download  \n# note\ntext\nCOWORK\n").as_ptr()
            ),
            IME_OK
        );
        // normalize: lowercase, bo ky tu la, dedupe giu thu tu
        assert_eq!(take(ime_settings_english_words_text(s)), "text\ncowork\n");
        assert_eq!(ime_settings_save(s, cpath.as_ptr()), IME_OK);
        let cfg = textvn_config::parse_config(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            cfg.english_words,
            vec!["text".to_string(), "cowork".to_string()]
        );
        ime_settings_free(s);
        assert!(ime_settings_english_words_text(std::ptr::null()).is_null());
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            ime_settings_save(std::ptr::null_mut(), std::ptr::null()),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(ime_settings_was_corrupt(std::ptr::null()), 0);
        let unknown = unsafe { CStr::from_ptr(ime_settings_macro_error_message(99)) };
        assert!(!unknown.to_bytes().is_empty());
    }
}
