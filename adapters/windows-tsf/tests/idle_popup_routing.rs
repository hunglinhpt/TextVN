// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-154 — popup trợ lý khi người dùng DỪNG gõ ~1 s. Lỗi gốc: timer idle gọi
//! `show_assist_overlay_with_recheck` với `ec = 0` nên popup không bao giờ hiện, và chỉ phím
//! Space/`,`/`;`/`:` mới lên lịch timer (dừng giữa từ hay sau chữ cuối là mất hẳn). Suite này
//! khoá phần LOGIC không cần COM: bảng phím nào lên lịch, và câu/từ nào được báo lỗi khi dừng
//! ở các điểm khác nhau theo từng `assist.mode`. Phần timer + overlay thật được phủ bởi
//! `tools/win/e2e-assist-apps.ps1` (ca `live popup *`, `click fix-all mid-word`, `alt+1 mid-word`).

#![cfg(windows)]

use std::rc::Rc;

use textvn_win_tsf::assist::check_sentence_on_demand;
use textvn_win_tsf::compose::KeyKind;
use textvn_win_tsf::edit_session::TsfShared;
use textvn_win_tsf::panel_idle::idle_arms;
use textvn_win_tsf::ThreadState;

fn shared_with(config: &str) -> Rc<TsfShared> {
    let ipc = textvn_win_tsf::ipc_client::IpcClient::global("idle-popup-routing.test.exe");
    let mut thread = ThreadState::new("idle-popup-routing.test.exe", 0).expect("engine");
    thread
        .reload_config_bytes(config.as_bytes())
        .expect("config hợp lệ");
    Rc::new(TsfShared::new(0, ipc, thread))
}

fn feed(shared: &TsfShared, text: &str) {
    let mut thread = shared.thread.borrow_mut();
    for c in text.chars() {
        let _ = thread.engine.key_char(c);
    }
}

const HIGHLIGHT: &str = r#"{"config_version":1,"assist":{"mode":"highlight"}}"#;
const AUTOCORRECT: &str =
    r#"{"config_version":1,"assist":{"mode":"autocorrect","auto_fix":"safe"}}"#;
const OFF: &str = r#"{"config_version":1,"assist":{"mode":"off"}}"#;

// ---- bảng phím -------------------------------------------------------------------------

#[test]
fn moi_phim_go_chu_dau_cau_xoa_deu_lap_lich() {
    let mut count = 0;
    for c in ('a'..='z').chain('A'..='Z').chain('0'..='9').chain([
        ' ', '.', ',', ';', ':', '!', '?', '-', '"', '(', ')', 'đ', 'ệ',
    ]) {
        assert!(idle_arms(KeyKind::Char(c), true), "{c:?} phải lên lịch");
        count += 1;
    }
    assert!(idle_arms(KeyKind::Backspace, true));
    assert!(count > 60);
}

#[test]
fn phim_khong_doi_cau_khong_lap_lich() {
    for k in [
        KeyKind::Enter,
        KeyKind::Tab,
        KeyKind::Escape,
        KeyKind::CtrlSpace,
        KeyKind::Other,
    ] {
        assert!(!idle_arms(k, true), "{k:?} không được đếm lại");
    }
}

#[test]
fn tieng_viet_tat_thi_khong_lap_lich() {
    for k in [KeyKind::Char('a'), KeyKind::Char(' '), KeyKind::Backspace] {
        assert!(!idle_arms(k, false));
    }
}

// ---- dừng ở từng điểm: có lỗi ⇒ báo, sạch ⇒ im ------------------------------------------

/// (đã gõ, có lỗi?) — ASCII, không có cặp ký tự Telex biến đổi.
const PAUSE_POINTS: [(&str, bool); 16] = [
    ("hom nay nay troi dep", true), // dừng giữa/cuối câu, không dấu cách cuối
    ("hom nay nay troi dep ", true), // sau dấu cách
    ("hom nay nay,", true),         // ngay sau dấu phẩy
    ("hom nay nay, ", true),        // sau dấu phẩy + cách
    ("hom nay nay.", true),         // ngay sau dấu chấm
    ("hom nay nay. ", true),        // sau chấm + cách
    ("hom nay nay;", true),         // sau chấm phẩy
    ("hom nay nay!", true),         // sau chấm than
    ("hom nay  troi dep", true),    // hai dấu cách
    ("hom nay , troi dep", true),   // dấu cách trước dấu phẩy
    ("hom nay nay tro", true),      // dừng giữa từ (chữ cuối chưa xong)
    ("hom nay troi dep", false),
    ("hom nay troi dep ", false),
    ("hom nay troi dep.", false),
    ("hom nay troi dep, ", false),
    ("hom nay troi dep,", false), // dấu phẩy treo là câu đang gõ dở, không phải lỗi
];

fn pause_matrix(config: &str) {
    for (typed, has_error) in PAUSE_POINTS {
        let shared = shared_with(config);
        if !shared.is_assist_enabled() {
            // Máy tắt trợ lý (state.json) hoặc config không bật: hợp đồng = luôn im lặng.
            assert!(check_sentence_on_demand(&shared).is_none());
            continue;
        }
        feed(&shared, typed);
        let got = check_sentence_on_demand(&shared).map(|c| c.issue_count());
        assert_eq!(
            got.is_some_and(|n| n > 0),
            has_error,
            "{config} — dừng sau {typed:?}: {got:?}"
        );
    }
}

#[test]
fn dung_1s_o_che_do_goi_y_bao_loi_dung_cho() {
    pause_matrix(HIGHLIGHT);
}

#[test]
fn dung_1s_o_che_do_sua_loi_an_toan_bao_loi_dung_cho() {
    pause_matrix(AUTOCORRECT);
}

#[test]
fn che_do_tat_thi_im_lang_o_moi_diem_dung() {
    for (typed, _) in PAUSE_POINTS {
        let shared = shared_with(OFF);
        feed(&shared, typed);
        assert!(check_sentence_on_demand(&shared).is_none(), "{typed:?}");
        assert!(!shared.is_assist_display_enabled());
    }
}

#[test]
fn rong_hoac_chi_khoang_trang_thi_im_lang() {
    for typed in ["", " ", "   ", ",", "."] {
        let shared = shared_with(HIGHLIGHT);
        feed(&shared, typed);
        let got = check_sentence_on_demand(&shared);
        assert!(
            got.is_none() || got.is_some_and(|c| c.issue_count() == 0) || typed == ",",
            "{typed:?}"
        );
    }
}

#[test]
fn lap_lai_cung_mot_diem_dung_cho_cung_ket_qua() {
    let shared = shared_with(HIGHLIGHT);
    feed(&shared, "hom nay nay troi dep");
    let a = check_sentence_on_demand(&shared).map(|c| (c.sentence.clone(), c.issue_count()));
    let b = check_sentence_on_demand(&shared).map(|c| (c.sentence.clone(), c.issue_count()));
    assert_eq!(a, b, "check phải thuần — timer chạy lại không đổi kết quả");
}

// ---- hợp đồng nguồn: không edit cookie giả ---------------------------------------------

/// Timer/click không có edit cookie. Gọi hàm TSF cần `ec` với `0` biên dịch được nhưng mọi
/// `GetSelection`/`GetText` đều lỗi — popup im lặng biến mất (R2-154). Mã chạy ngoài luồng
/// phím phải xin session (`request_sync_or_async`).
#[test]
fn khong_co_edit_cookie_gia_trong_ma_nguon() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut scanned = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            assert!(
                !code.contains("ec: u32 = 0"),
                "{}:{} — edit cookie giả `0`: xin session thay vì gọi thẳng",
                path.display(),
                i + 1
            );
        }
        scanned += 1;
    }
    assert!(scanned > 10);
}

#[test]
fn timer_idle_xin_session_that() {
    let src = include_str!("../src/panel_idle.rs");
    assert!(
        src.contains("PanelIdleSession"),
        "on_timer phải dựng session"
    );
    assert!(src.contains("request_sync_or_async"));
}

#[test]
fn moi_phim_go_deu_di_qua_idle_arms() {
    let src = include_str!("../src/edit_session.rs");
    assert!(src.contains("panel_idle::idle_arms("));
    // Không còn điều kiện lên lịch tự chế rải rác (trước đây chỉ Space/`,`/`;`/`:`).
    assert!(!src.contains("trigger == AssistTrigger::Word\n                        ||"));
}
