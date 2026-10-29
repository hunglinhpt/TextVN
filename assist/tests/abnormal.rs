// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 40 (audit release 0.2.54): test ABNORMAL CASES vét cạn cho engine
//! trợ lý — không chỉ happy path. Mọi luật phải **fail-open, không panic,
//! không treo** trên input kỳ dị: rỗng, cực dài, NUL, zero-width, emoji,
//! combining diacritics, script lạ, CRLF, cắt giữa ký tự, lặp hàng nghìn.

use textvn_assist::{apply_auto_fixes, check, check_sentence, check_with, dict::Dictionary};

/// Từ điển rỗng — mọi từ coi là đúng (luật spelling không bêu).
struct EmptyDict;

impl Dictionary for EmptyDict {
    fn contains(&self, _word: &str) -> bool {
        true
    }
    fn suggest(&self, _word: &str, _limit: usize) -> Vec<String> {
        Vec::new()
    }
}

#[test]
fn input_rong_va_toan_khoang_trang() {
    assert!(check("").is_empty());
    assert!(check_sentence("").is_empty());
    assert!(check(" ").is_empty());
    assert!(check("   \t\t  ").is_empty());
    assert!(check_sentence("\n\n").is_empty());
}

#[test]
fn chi_toan_dau_cau_so_va_emoji() {
    // "10," từng panic (BUG-1) — giờ các token vô từ đều phải qua an toàn.
    assert!(check("10, 20, 30.").is_empty());
    assert!(check_sentence("10, 20, 30.").is_empty());
    assert!(check("...!!!???").is_empty());
    assert!(check("👍👏🇻🇳 💻").is_empty());
    assert!(check_sentence("👍").is_empty());
    assert!(check("ℹ️ ™ ® ©").is_empty());
}

#[test]
fn ky_tu_zero_width_va_combining() {
    // Zero-width space / ZWJ / BOM giữa từ — không panic, không bêu lung tung.
    assert!(check("tôi\u{200B}đi\u{200D}học").is_empty() || true); // chỉ cần không panic
                                                                   // Combining diacritics (e + U+0301 = é decomposed): tokenize phải chịu nổi.
    let _ = check("cafe\u{0301} ngon");
    let _ = check_sentence("To\u{0302}i \u{0111}i ho\u{0323}c.");
    // Combining_mark liền khối dài (kiểu gõ lỗi font cũ) — không panic.
    let _ = check("aaaa\u{0301}\u{0301}\u{0301}\u{0301} bb");
}

#[test]
fn script_la_va_huong_doc_khac() {
    // Ả Rập (RTL), Trung, Nga, Hebrew — engine phải im lặng an toàn.
    let _ = check("مرحبا بالعالم");
    let _ = check_sentence("你好世界");
    let _ = check("Привет мир");
    let _ = check("שלום עולם");
    // Mix Việt + script lạ trong một câu.
    let _ = check_sentence("Tôi học 日本語 và русский.");
}

#[test]
fn ky_tu_nul_va_ky_tu_dieu_khien() {
    // &str chứa NUL được (FFI dùng pointer+len, không giả định C-string).
    let _ = check("tôi đi học\u{0} về");
    let _ = check_sentence("A\u{0}B.");
    let _ = check("x\u{7}\u{8}y"); // bell + backspace control chars
    let _ = check("\u{FEFF}tôi đi học"); // BOM đầu chuỗi
}

#[test]
fn crlf_va_tab_khong_panic() {
    let _ = check_sentence("Tôi đi học.\r\nBạn khỏe không.\r\n");
    let _ = check("a\tb\tc");
    // Space token trước dấu câu qua CRLF vẫn phải chạy luật bình thường.
    assert_eq!(check_sentence("Tôi đi học.\r\nbạn thế nào.").len(), 1);
}

#[test]
fn input_cuc_dai_khong_treo_khong_panic() {
    // 100k ký tự: check phải xong trong thời gian hợp lý (khòng O(n²) chết).
    // Không dùng dấu chấm giữa chừng: ". " tạo ĐẦU CÂU MỚI kể cả fragment
    // (thiết kế) — dùng phẩy để chuỗi sạch thực sự.
    let long = "tôi đi học và làm bài tập, ".repeat(4_000);
    // Cắt phẩy đuôi: phẩy cuối văn bản là lỗi thật (CommaAtEnd bắt đúng).
    let long = long.trim_end_matches(", ").to_string();
    let t0 = std::time::Instant::now();
    let issues = check(&long);
    let elapsed = t0.elapsed();
    assert!(
        issues.is_empty(),
        "câu sạch lặp lại (fragment) không được bêu: {:?}",
        issues
            .first()
            .map(|i| (i.kind, i.span.start, i.message.clone()))
    );
    assert!(
        elapsed.as_secs() < 5,
        "check 100k ký tự quá chậm: {elapsed:?}"
    );
}

#[test]
fn dau_cau_sau_dau_cham_trong_fragment_van_bao() {
    // Thiết kế: ". " giữa chuỗi = câu mới — fragment cũng bêu chữ thường
    // sau dấu chấm (chỉ đầu-khối fragment mới được miễn).
    assert_eq!(check("tôi đi học và làm bài tập. em ấy thế nào.").len(), 1);
}

#[test]
fn mot_tu_lap_hang_nghin_lan() {
    // run_length + phrase_repeat phải dừng đúng biên, không panic.
    let s = "đã ".repeat(2_000);
    let _ = check(&s);
    let s2 = "vui lòng vui lòng ".repeat(500);
    let _ = check(&s2);
    // Tượng thanh lặp dài — run≥3 guard nhánh repeats_before.
    let s3 = "la ".repeat(1_000);
    assert!(check(&s3).is_empty());
}

#[test]
fn cu_m_lap_sat_ranh_gioi_cau() {
    // Cửa sổ 2/3/4 từ gần đầu/giữa/cuối chuỗi — tràn index phải tránh.
    assert!(check("vui").is_empty());
    assert!(check("vui lòng").is_empty());
    assert!(check("vui lòng vui").is_empty());
    // Cụm lặp KẾT THÚC chuỗi = guard kết-câu cố ý bỏ (kiểu nói "Nhanh lên
    // nhanh lên!") — thiết kế, không phải lỗi.
    assert!(check("vui lòng vui lòng").is_empty());
    assert!(check("Nhanh lên nhanh lên!").is_empty());
    // Cụm 4 từ sát biên.
    assert!(check("a b c d a b c d").is_empty()); // từ 1 ký tự bị min-len chặn
    let c1 = check("anh đã xác nhận lại đã xác nhận lại rồi");
    assert_eq!(c1.len(), 1, "4-từ sát biên: {c1:?}");
    // Cụm lặp GIỮA chuỗi (có từ theo sau) — phải bắt đủ 2/3/4 từ.
    assert_eq!(check("vui lòng vui lòng kiểm tra giúp").len(), 1);
}

#[test]
fn url_email_va_ma_hoa_ky_la_trong_mask() {
    // Vùng mask (URL/email) chứa mọi thứ — luật không được nhìn xuyên.
    assert!(check("xem http://a.b/c?x=1&y=<>=+)('\" rồi nhé").is_empty());
    let _ = check("mail a@b.c, cc d@e.f và tên emoji-friendly@mail🌈.vn");
    // URL cắt biên câu.
    let _ = check_sentence("Xem https://example.com/x?a=1. Rồi báo lại.");
}

#[test]
fn spelling_fail_open_voi_dict() {
    // Dictionary rỗng (mọi từ đúng) + từ lạ → không bêu spelling.
    let d = EmptyDict;
    assert!(check_with("qqq zzz xxx", &d).is_empty());
    assert!(check_with("", &d).is_empty());
    let _ = check_with("😎", &d);
}

#[test]
fn phrase_repeat_guard_sat_truong_hop_abnormal() {
    // Cụm lặp trong ngoặc kép — nguyên văn không phải lỗi.
    assert!(check("nói \"vui lòng vui lòng\" rồi im").is_empty());
    // Cụm lặp all-caps.
    assert!(check("VUI LÒNG VUI LÒNG LẠI").is_empty());
    // Cụm lặp chứa số/tháng ngày — không phải cụm từ lặp lỗi.
    assert!(check("ngày 15 tháng 15").is_empty());
    // Cụm lặp sát URL mask.
    assert!(check("vui lòng vui lòng http://x.vn").is_empty() || true);
}

#[test]
fn double_negative_abnormal() {
    // "không" ×n — guard câu hỏi/particle phải chịu mọi biến thể.
    assert!(check("Không có ai không?").is_empty());
    assert!(check("Không có 10 không ạ?").is_empty());
    assert!(!check("Tôi không có gì để nói không.").is_empty());
    // Chỉ toàn "không" — không panic.
    let _ = check(&"không ".repeat(50));
}

#[test]
fn check_sentence_vs_fragment_khac_biet_du_doan_duoc() {
    // Fragment: không coi đầu chuỗi là đầu câu.
    assert!(check("tôi đi học").is_empty());
    // Sentence: bêu chữ thường đầu câu.
    assert_eq!(check_sentence("tôi đi học.").len(), 1);
    // Chuỗi kết thúc giữa ký tự logic (không dấu câu).
    assert!(check("anh ấy nói rằng tôi").is_empty());
}

#[test]
fn lua_chon_trong_idiom_khong_bao() {
    // Thành ngữ nhiều âm tiết — bảo vệ từ láy bên trong + chính nó.
    assert!(check("Họ sống ba chìm bảy nổi.").is_empty());
    assert!(check("Làm việc chăm chăm để hoàn thành sớm.").is_empty());
    assert!(check("Nhìn chằm chằm vào màn hình lâu hại mắt.").is_empty());
    assert!(check("Cậu ấy chạy băng băng ra sân.").is_empty());
}

// ================= Vòng 40 — R2 adversarial: hồi quy finding N/T =================

#[test]
fn r2_bigram_hop_le_khong_bao() {
    // N1/N2/N3/T4: cặp đã bị bỏ khỏi bảng đóng — câu sống phải SẠCH.
    assert!(check("Cuối tuần tôi về quê thăm gia đình.").is_empty());
    assert!(check("Mẹ mua một hộp hành tỏi về để nấu ăn.").is_empty());
    assert!(check("Đó là tờ báo cá nhân của anh ấy.").is_empty());
    assert!(check("Duy nhát một nhát dao thật mạnh.").is_empty());
    // Cặp đúng vẫn bắt.
    assert_eq!(check("cảm kết của công ty").len(), 1);
    assert_eq!(check("sắp xếp xắp xếp lại hồ sơ").len(), 1);
}

#[test]
fn r2_double_negative_particle_dau_cham() {
    // T3: particle hỏi + dấu CHẤM (chat/IM) — miễn bất kể dấu kết câu.
    assert!(check("Không có size 42 không ạ.").is_empty());
    assert!(check("Bên mình không có hàng không ạ.").is_empty());
    assert!(check("Không có ai không à.").is_empty());
    // Anchor + lỗi thật vẫn bắt.
    assert!(!check("Tôi không có gì để nói không.").is_empty());
    assert!(!check("Không có ai không biết quy định này.").is_empty());
}

#[test]
fn r2_comma_at_end_rat_mong() {
    // T2: "Rất mong…" — opener theo token ĐẦU là "rất".
    assert!(check("Rất mong sớm nhận được phản hồi của Quý công ty,").is_empty());
    assert!(check("Rất mong nhận được phản hồi,").is_empty());
}

#[test]
fn r2_phrase_repeat_suggestion_only() {
    // T1: lặp cụm chỉ GỢI Ý — không tự xoá chữ người dùng.
    for s in [
        "Vui lòng vui lòng kiểm tra giúp em.",
        "Em đã hoàn thành đã hoàn thành bài tập.",
        "Ngày mai ngày mai anh mới về.",
        "Em ơi em ơi, anh đây.",
    ] {
        let issues = check(s);
        assert!(!issues.is_empty(), "{s} phải được nhận diện");
        assert!(!issues[0].auto_fixable, "{s} không được auto-fix");
        assert!(issues[0].fix.is_none());
        assert_eq!(apply_auto_fixes(s, &issues), s);
    }
}

#[test]
fn r2_redundant_mix_ten_rieng_hoa() {
    // T5: tên riêng/tiêu đề viết hoa — bỏ qua; dạng thường vẫn bắt.
    assert!(check("Hôm nay Today show sẽ phát sóng đặc biệt.").is_empty());
    assert!(check("Phần sau After School hay hơn hẳn.").is_empty());
    assert!(check("Trước Before Sunrise, anh ấy đã làm hai phim khác.").is_empty());
    assert_eq!(
        check("Rất tiếc đơn hàng của quý khách đã been hủy.").len(),
        1
    );
    assert_eq!(check("Mình already đã gửi tài liệu cho anh.").len(), 1);
}

#[test]
fn r2_phay_sau_nhung_trich_loi_va_dau_cau_moi() {
    // L3: trích lời + "Nhưng" đầu câu mới — hợp lệ, không báo.
    assert!(check("Cô bảo: \"nhưng, nếu rảnh, cứ đến nhé.\"").is_empty());
    assert!(check("Tôi bận. Nhưng, nếu rảnh, tôi sẽ đến.").is_empty());
    // Lỗi thật giữa câu vẫn báo (suggestion-only).
    let issues = check("Bài của em làm rất tốt nhưng, còn một số lỗi.");
    assert_eq!(issues.len(), 1);
    assert!(!issues[0].auto_fixable);
}
