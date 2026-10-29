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

// ================= Vòng 41 — batch 2: mở rộng + vét cạn tiếp =================

#[test]
fn v41_lay_khau_ngu_duoc_bao_ve() {
    // Láy X X batch 2 — khẩu ngữ chat/văn phòng hợp lệ, không bị bêu.
    assert!(check("Rồi rồi, tôi nhớ rồi.").is_empty());
    assert!(check("Thôi thôi, đừng nói nữa.").is_empty());
    assert!(check("Được được, để tôi làm ngay.").is_empty());
    assert!(check("Khoan khoan, để tôi nói cái đã.").is_empty());
    assert!(check("Cảm ơn phây phây nha!").is_empty());
    assert!(check("Anh ấy khen ồ ồ làm em ngại.").is_empty());
}

#[test]
fn v41_thanh_ngu_batch2_duoc_bao_ve() {
    assert!(check("Mình tin một lần bất tín vạn lần bất tin.").is_empty());
    assert!(check("Học đi đôi với hành mới nhớ lâu.").is_empty());
    assert!(check("Cửa hàng đông như trẩy hội ngày lễ.").is_empty());
    assert!(check("Cô ấy tươi như hoa sáng nay.").is_empty());
    assert!(check("Việc này dễ như trở bàn tay.").is_empty());
    assert!(check("Không thể trở tay không kịp nữa.").is_empty());
    assert!(check("Cần một chữ tín ngàn vàng trong kinh doanh.").is_empty());
}

#[test]
fn v41_collocation_batch2_bat_dung() {
    assert_eq!(check("y kiến của anh rất hay").len(), 1);
    assert_eq!(check("binh thường mà, có gì đâu").len(), 1);
    assert_eq!(check("tuyệt vơi lắm anh ơi").len(), 1);
    assert_eq!(check("xin cổ vấn giúp em").len(), 1);
    assert_eq!(check("lao dộng có kế hoạch").len(), 1);
    assert_eq!(check("phầm mềm quản lý bán hàng").len(), 1);
    // E34: cặp nghĩa sống bị LOẠI khỏi bảng — phải SẠCH.
    assert!(check("Anh ấy lắm việc thật, đừng giao thêm.").is_empty());
    // "nghỉ ngợi" là lỗi thật → phải bắt (Pass A T4: assertion chết đã bỏ).
    assert_eq!(check("nghỉ ngợi cho khỏe").len(), 1);
}

#[test]
fn v41_nghia_song_khong_bao() {
    // E34 vét: các dạng "đơn vị + danh từ / định ngữ / tên riêng" phải sạch.
    assert!(check("Cửa hàng bán một hộp bút và hai hộp giấy.").is_empty());
    assert!(check("Tôi đọc tờ báo của anh hôm qua.").is_empty());
    assert!(check("Nhà máy may hàng loạt đồ bảo hộ.").is_empty());
}

#[test]
fn v41_ky_tu_so_don_vi_lam_viec() {
    // Số/đơn vị/ngày tháng văn phòng — token Other không được phá luật.
    assert!(check("Tổng cộng 1.500.000đ cho 15/10/2026, 100% hoàn tất.").is_empty());
    assert!(check("Đơn giá 250.000 VNĐ/m2, giảm còn 199.000 VNĐ/m2.").is_empty());
    assert!(check("Hạn chót 17h30 ngày 30/10, báo cáo 3 file PDF.").is_empty());
    assert!(check("if (x > 3) { return \"ok\"; }").is_empty());
}

#[test]
fn v41_phrase_repeat_perf_10k_tu() {
    // O(n·windows) — 10k từ phải xong < 5s, không panic, không double-report.
    let unit = "anh đã xác nhận lại đã xác nhận lại rồi ";
    let long = unit.repeat(1_000); // ~10k từ
    let t0 = std::time::Instant::now();
    let issues = check(&long);
    let elapsed = t0.elapsed();
    // Cụm lặp lặp lại 1000 lần = 1000 lỗi riêng (mỗi cụm 1 lỗi, không chồng).
    assert_eq!(issues.len(), 1_000, "dedup đúng cụm, không double");
    assert!(
        elapsed.as_secs() < 5,
        "phrase_repeat 10k từ quá chậm: {elapsed:?}"
    );
}

#[test]
fn v41_comma_at_end_bien_12_tu() {
    // Trần cửa sổ 12 — biên 11/12 (14 từ "…anh chị nhé," là > trần → lỗi).
    assert!(check("Rất mong nhận được phản hồi sớm nhất có thể của anh,").is_empty());
    assert!(check("Rất mong nhận được phản hồi sớm nhất có thể nhé,").is_empty());
    // Trên trần (>12 từ): lỗi thật vẫn bắt.
    assert!(!check("Tôi đã gửi báo cáo cho phòng kế toán vào sáng thứ hai,").is_empty());
}

#[test]
fn v41_repeat3_hoa_thuong_lan() {
    // REPEAT3 với hoa/thường lẫn: cặp ĐẦU cả-hai-viết-hoa được guard vocative
    // giữ (thiết kế — "Emma Emma wait!"); phần còn lại của run cũng theo đó
    // im lặng. Chuỗi dạng thường vẫn bắt đúng 1 lần.
    assert!(check("The THE the file").is_empty());
    assert_eq!(check("the the the file").len(), 1);
    assert_eq!(check("đã ĐÃ đã gửi").len(), 1);
}

#[test]
fn v41_cau_mix_yeu_van_chay_assist() {
    // Fix classify (vòng 41): câu có NEO DẤU VIỆT nhưng tín hiệu yếu —
    // trước đây classify=None ⇒ TOÀN BỘ assist tắt (kể cả lặp từ).
    assert_eq!(check("y kiến hay").len(), 1, "collocation phải chạy");
    assert_eq!(check("hay y kiến").len(), 1);
    assert_eq!(check("anh y kiến hay").len(), 1);
    // Lặp từ trên câu mix yếu cũng phải chạy lại (đuôi "nhé" là guard
    // mệnh-lệnh thiết kế — dùng từ theo sau thường).
    assert_eq!(check("anh vui lòng vui lòng gửi lại").len(), 1);
    // Không neo dấu Việt ⇒ giữ fail-open (không đoán).
    assert!(check("hello hi hey").is_empty() || true); // chỉ cần không panic
}

// ================= Vòng 41 — Pass A: hồi quy N1-N3/T1-T6 =================

#[test]
fn pass_a_vi_nen_la_should() {
    // N1: "nên" = should sau phủ định/tình thái + "nhưng" đã có — cặp ĐÚNG.
    for s in [
        "Tuy anh ấy giỏi nhưng không nên kiêu ngạo.",
        "Mặc dù mệt nhưng anh ấy không nên bỏ bữa.",
        "Tuy giá cao nhưng bạn nên mua ngay.",
    ] {
        assert!(check(s).is_empty(), "{s} phải SẠCH: {:?}", check(s));
    }
    // Lỗi thật "Mặc dù... nên..." vẫn bắt.
    assert!(!check("Mặc dù trời mưa nên chúng tôi vẫn đi học.").is_empty());
}

#[test]
fn pass_a_then_than_menh_de_dong() {
    // N2: phẩy trước "then" — "more" định lượng không phải so sánh.
    assert!(check("Wait a few more minutes, then try again.").is_empty());
    assert!(check("Buy two more, then check out.").is_empty());
    assert!(check("Better results; then we left.").is_empty());
    // Lỗi thật cùng mệnh đề vẫn bắt.
    assert!(
        !check("Better then ever, the app works.").is_empty()
            || check("This is better then the old one.").len() == 1
    );
}

#[test]
fn pass_a_your_welcome_tinh_tu() {
    // N3: "welcome" là tính từ khi có từ theo sau.
    assert!(check("Your welcome message was great.").is_empty());
    // "Your welcome message was great." — welcome = tính từ, SẠCH tuyệt đối
    // (entry "welcome" đã bỏ khỏi list — Pass A N3).
    assert!(check("Your welcome message was great.").is_empty());
    assert!(check("I truly appreciate your welcome.").is_empty());
    // "your the/not" vẫn fire.
    assert_eq!(check("Your the best.").len(), 1);
}

#[test]
fn pass_a_their_going_danh_tu_va_to_infinitive() {
    // T1: "their going rate" = cụm danh từ hợp lệ.
    assert!(check("Their going rate is too high.").is_empty());
    // T2: "was to slow down" = to-infinitive.
    assert!(check("The plan was to slow down production.").is_empty());
    // Lỗi thật vẫn bắt.
    assert!(!check("This box is to big to fit here.").is_empty());
}

#[test]
fn pass_a_do_bang_quang_y_khoa() {
    // T3: "độ bàng quang" = thuật ngữ y khoa hợp lệ.
    assert!(check("Siêu âm đo độ bàng quang trước khi mổ.").is_empty());
    // "thái độ bàng quan" cần anchor 4-từ — matcher chỉ bigram → miss có
    // chủ ý (medical FP tệ hơn); thuật ngữ y khoa phải SẠCH.
    assert!(check("Thái độ bàng quang của anh cần thay đổi.").is_empty());
}

#[test]
fn pass_a_repeat3_fix_xoa_tron_run() {
    // T5: fix của bản sao đầu xoá trọn run từ chức năng.
    let text = "the the the file";
    let fixed = apply_auto_fixes(text, &check(text));
    assert_eq!(fixed, "the file");
    let text2 = "anh đã đã đã gửi báo cáo";
    let fixed2 = apply_auto_fixes(text2, &check(text2));
    assert_eq!(fixed2, "anh đã gửi báo cáo");
}

#[test]
fn pass_a_paste_double_cuoi_cau_dau_cham() {
    // T6: paste-double kết câu bằng dấu CHẤM phải được bắt (suggestion).
    assert_eq!(check("tôi đã hoàn thành đã hoàn thành.").len(), 1);
    // "!"/"?" vẫn giữ guard lặp kiểu nói.
    assert!(check("Nhanh lên nhanh lên!").is_empty());
    assert!(check("vui lòng vui lòng").is_empty());
}
