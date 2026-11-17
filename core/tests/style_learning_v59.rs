// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 59 — nâng cấp **User Profile** học cá nhân (WS1):
//!
//! - **F-P10-1 recency touch**: gõ lại từ cũ phải làm mới vị trí trong
//!   `style_words` (mới nhất cuối) → prediction cộng recency boost cho từ
//!   vừa gõ lại — từ count thấp nhưng MỚI thắng từ count cao nhưng LÂU QUÁN.
//! - **F-P10-2 trigram**: ngữ cảnh 2 từ + từ thứ 3 được học (`the quick brown`)
//!   cạnh bigram cũ — next-phrase khớp khoá dài hơn sẽ cụ thể hơn.
//! - **F-P10-3 Dream consolidation** sống ở crate `config`
//!   (`config/tests/dream_consolidation.rs`).
//!
//! G8: chỉ assert hành vi/đếm + drain JSON tối thiểu — không log nội dung.

use textvn_core::keymap::vk;
use textvn_core::{Engine, EngineOptions, KeyEvent};

fn letter(c: char) -> KeyEvent {
    KeyEvent::char_down(c)
}

fn space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        ch: ' ' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn tab() -> KeyEvent {
    KeyEvent {
        vk: vk::TAB,
        ch: '\t' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn type_str(e: &mut Engine, s: &str) {
    for c in s.chars() {
        let k = if c == ' ' { space() } else { letter(c) };
        e.key(&k);
    }
}

fn learning_opts() -> EngineOptions {
    EngineOptions {
        style_learning_enabled: true,
        // Prediction phải bật để Tab đi qua đường hoàn tất có ứng viên cá nhân
        // (Tier 0 style_words trong `complete_word_candidates`).
        suggest_vn_complete: true,
        ..Default::default()
    }
}

/// F-P10-1: từ gõ lại GẦN ĐÂY thắng ứng viên cá nhân count cao hơn nhưng
/// LÂU KHÔNG DÙNG (recency boost +12 cho 12 mục cuối danh sách hồ sơ).
#[test]
fn recency_touch_thang_hon_count_lau_quan() {
    let opts = learning_opts();
    let mut e = Engine::new(opts);
    // "projectx" dùng 10 lần (đứng đầu hồ sơ — cũ).
    for _ in 0..10 {
        e.style_note_word("projectx");
    }
    // 30 từ giữa để "projectx" rời khỏi vùng recency (24 mục cuối).
    let fillers: Vec<String> = (0..30)
        .map(|i| format!("zz{}{}", (b'a' + i / 26) as char, (b'a' + i % 26) as char))
        .collect();
    for f in &fillers {
        e.style_note_word(f);
    }
    // "projecty" dùng 8 lần — MỚI NHẤT (đuôi hồ sơ).
    for _ in 0..8 {
        e.style_note_word("projecty");
    }

    // Tab hoàn tất "proj" → ứng viên cá nhân thắng nguồn tĩnh, và recency
    // chọn "projecty" (100+8+12) thay "projectx" (100+10+0).
    type_str(&mut e, "proj");
    let out = e.key(&tab());
    let inserted: String = match out.action {
        textvn_core::Action::Replace { insert, .. } | textvn_core::Action::Commit { insert } => {
            insert.into_iter().collect()
        }
        _ => String::new(),
    };
    assert!(
        inserted.contains("projecty"),
        "recency phải chọn 'projecty', chèn: {inserted}"
    );
    assert!(
        !inserted.contains("projectx"),
        "'projectx' (count cao, lâu quán) không được thắng recency: {inserted}"
    );
}

/// F-P10-1 (phần gỡ): từ lâu không dùng nhưng count cao vẫn phải vào ứng
/// viên — recency KHÔNG xoá lịch sử, chỉ xếp thứ tự.
/// Chạy ở **Mode EN**: ở Mode VI Telex, gõ "report" fold thành "repỏt" (r
/// sau nguyên âm = dấu) → Tab đúng thiết kế phải khôi phục từ EN gốc trước
/// (tier restore `i32::MIN` — guard `sort`→`sỏt`), không thể khớp cá nhân.
#[test]
fn recency_khong_xoa_lich_su_count_cao() {
    let opts = EngineOptions {
        enabled: false,
        style_learning_enabled: true,
        suggest_vn_complete: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    for _ in 0..30 {
        e.style_note_word("reportdeck");
    }
    // 30 filler để "reportdeck" rời khỏi cửa sổ recency (24 mục cuối) —
    // đúng kịch bản "lâu không dùng, chỉ còn count giữ nó sống".
    for i in 0..30u32 {
        let f = format!(
            "zz{}{}",
            char::from_u32(97 + i / 26).unwrap(),
            char::from_u32(97 + i % 26).unwrap()
        );
        e.style_note_word(&f);
    }
    type_str(&mut e, "report");
    let out = e.key(&tab());
    let inserted: String = match out.action {
        textvn_core::Action::Replace { insert, .. } | textvn_core::Action::Commit { insert } => {
            insert.into_iter().collect()
        }
        _ => String::new(),
    };
    assert!(
        inserted.contains("reportdeck"),
        "count cao vẫn phải hoàn tất: {inserted}"
    );
}

/// F-P10-2: gõ 3 từ liền trong Mode EN → trigram `he ran fast` được học cạnh
/// bigram (`he ran`, `ran fast`) — drain JSON mang đủ 6 mục (3 đơn + 2 đôi +
/// 1 ba).
#[test]
fn trigram_hoc_the_quick_brown() {
    let opts = EngineOptions {
        // Mode EN: tiếng Việt tắt — chữ đi thẳng, ranh giới từ vẫn học.
        enabled: false,
        suggest_vn_complete: true,
        style_learning_enabled: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    type_str(&mut e, "he ran fast");
    e.key(&space());
    let drain = e.style_drain_json();
    for expected in ["he", "ran", "fast", "he ran", "ran fast", "he ran fast"] {
        assert!(
            drain.contains(&format!("\"word\":\"{expected}\"")),
            "drain thiếu '{expected}': {drain}"
        );
    }
}

/// P7 regression guard (R2-146/E-series): cụm dính dấu câu KHÔNG được học —
/// trigram mới không được mở lỗ hổng học qua dấu.
#[test]
fn trigram_khong_hoc_qua_dau_cau() {
    let opts = EngineOptions {
        enabled: false,
        style_learning_enabled: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    type_str(&mut e, "ran fast");
    // Dấu chấm giữa — ngắt mạch: "ran fast" vẫn học (không có dấu giữa),
    // nhưng cụm có dấu câu bên trong thì không thể sinh.
    e.key(&KeyEvent::char_down('.'));
    type_str(&mut e, "now");
    e.key(&space());
    let drain = e.style_drain_json();
    // bigram "ran fast" có (chữ liền nhau); KHÔNG được có cụm chứa dấu chấm.
    assert!(drain.contains("\"word\":\"ran fast\""), "{drain}");
    assert!(
        !drain.contains("fast. now"),
        "trigram không được học qua dấu câu: {drain}"
    );
}

/// Dream RSI (G19 view vòng 59): đường accept next-phrase văn phòng EN tăng
/// đúng công thức ở Mode EN — bối cảnh nhiều từ Tab = nguyên glyph cụm chèn.
#[test]
fn rsi_next_phrase_office_en_dung_cong_thuc() {
    let opts = EngineOptions {
        enabled: false,
        suggest_vn_complete: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    type_str(&mut e, "please find the executive summary below");
    let rsi0 = e.dream_rsi_snapshot();
    let out = e.key(&tab());
    let inserted: String = match out.action {
        textvn_core::Action::Replace { insert, .. } | textvn_core::Action::Commit { insert } => {
            insert.into_iter().collect()
        }
        _ => String::new(),
    };
    let rsi1 = e.dream_rsi_snapshot();
    assert_eq!(rsi1.accepts - rsi0.accepts, 1);
    assert_eq!(
        rsi1.chars_saved - rsi0.chars_saved,
        inserted.chars().count() as u64
    );
    // keys_typed: Tab đếm 1 theo baseline dream_rsi_100 (Tab ch='	' ≠ 0).
    assert_eq!(rsi1.keys_typed - rsi0.keys_typed, 1, "Tab đếm đúng 1 phím");
    // Trước Tab, các phím chữ đã được đếm: 34 chữ + 5 dấu cách = 39.
    assert_eq!(rsi0.keys_typed, 39, "34 chữ + 5 space = 39 phím thật");
}
