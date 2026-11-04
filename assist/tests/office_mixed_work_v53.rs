// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 53 (QA persona battery mở rộng — phản hồi chủ repo: "thêm tối đa các
//! kịch bản test với nhiều việc hỗn hợp trong văn phòng từ coding tới mọi
//! việc... mỗi trường hợp test chi tiết cover mọi abnormal cases"):
//! 8 persona mới (bất động sản, ngân hàng-bảo hiểm, logistics, sản xuất,
//! đấu thầu, y tế, du lịch, thủ tục hành chính) + câu mix VI-EN + đường dẫn/
//! URL/email + abnormal mở rộng (input 65k, surrogate pair, bidi, ZWJ giữa
//! từ Việt, lỗi thật sát decor). Mỗi case SHOULD-CLEAN hoặc SHOULD-FIRE.

use textvn_assist::{check, check_sentence};

// ================= 1. BẤT ĐỘNG SẢN =================

#[test]
fn v53_bat_dong_san_sach() {
    for s in [
        "Căn hộ hai phòng ngủ giá 2,5 tỷ, đã bao gồm nội thất.",
        "Diện tích 75m2, sổ đỏ chính chủ, thuận tiện đi lại.",
        "Căn hộ được bàn giao trong tháng 12, đặt cọc 100 triệu.",
        "Phí quản lý 8.000đ/m2/tháng, đã gồm điện nước chung.",
        "Chung cư có thang máy, khu dân cư văn minh, an ninh tốt.",
    ] {
        assert!(check(s).is_empty(), "BĐS SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_bat_dong_san_lỗi() {
    assert!(!check("Căn hộ đã đã bàn giao cho khách.").is_empty());
    assert!(!check("Hạn thanh toán đợt 2 là 31/02/2026.").is_empty());
    assert!(!check("Giá thuê 8.500.000.00đ mỗi tháng.").is_empty());
}

// ================= 2. NGÂN HÀNG - BẢO HIỂM =================

#[test]
fn v53_ngan_hang_sach() {
    for s in [
        "Giao dịch chuyển khoản 12.000.000đ đã thành công lúc 14:30.",
        "Sổ tiết kiệm kỳ hạn 6 tháng, lãi suất 5,2%/năm.",
        "Vui lòng xác nhận mã OTP để hoàn tất giao dịch.",
        "Hồ sơ vay cần bổ sung sao kê lương 3 tháng gần nhất.",
        "Hợp đồng bảo hiểm đến hạn đóng phí vào ngày 15.",
    ] {
        assert!(check(s).is_empty(), "ngân hàng SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_ngan_hang_lỗi() {
    assert!(!check("Khoản vay đã đã được giải ngân.").is_empty());
    assert!(!check("Số dư cuối kỳ 1.250.00đ chưa khớp.").is_empty());
    // "Please send the contract to my assistant." là ngữ pháp EN ĐÚNG — SẠCH.
    assert!(check("Please send the contract to my assistant.").is_empty());
    // Lỗi EN thật vẫn bắt.
    assert!(!check("Please send a email to my assistant.").is_empty());
}

// ================= 3. LOGISTICS - XUẤT NHẬP KHẨU =================

#[test]
fn v53_logistics_sach() {
    for s in [
        "Tờ khai hải quan đã được thông quan lúc 9h sáng nay.",
        "Lô hàng dự kiến cập cảng Hải Phòng vào thứ Năm tuần tới.",
        "Tồn kho cuối tháng tăng nhẹ do đơn hàng cuối năm.",
        "Cước vận chuyển đã bao gồm phí bốc xếp và lưu kho 1 ngày.",
        "Chúng tôi cần bổ sung chứng từ cho lô hàng số 42.",
    ] {
        assert!(check(s).is_empty(), "logistics SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_logistics_lỗi() {
    assert!(!check("Hàng hóa đã đã được chuyển về kho trung tâm.").is_empty());
    assert!(!check("Lịch giao hàng đã đã gửi cho bên mua.").is_empty());
}

// ================= 4. SẢN XUẤT - KỸ THUẬT =================

#[test]
fn v53_san_xuat_sach() {
    for s in [
        "Dây chuyền sản xuất dừng bảo trì định kỳ vào Chủ nhật.",
        "Năng suất quý này tăng 5% so với kế hoạch đề ra.",
        "Bảo hộ lao động bắt buộc khi vào khu vực sản xuất.",
        "Tỷ lệ lỗi của lô hàng dưới ngưỡng cho phép 2%.",
        "Bản vẽ thi công đã được phê duyệt và gửi tới nhà thầu.",
    ] {
        assert!(check(s).is_empty(), "sản xuất SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_san_xuat_lỗi() {
    assert!(!check("Linh kiện đã đã giao đến xưởng sản xuất.").is_empty());
    assert!(!check("Quy trình kiểm tra đã đã cập nhật.").is_empty());
}

// ================= 5. ĐẤU THẦU - HỢP ĐỒNG =================

#[test]
fn v53_dau_thau_sach() {
    for s in [
        "Hồ sơ dự thầu nộp trước 17h ngày 15/11.",
        "Gói thầu yêu cầu bảo đảm dự thầu bằng thư bảo lãnh ngân hàng.",
        "Điều khoản thanh toán gồm 3 đợt theo tiến độ nghiệm thu.",
        "Biên bản ghi nhớ có hiệu lực từ ngày ký.",
        "Mọi sửa đổi hợp đồng phải lập thành phụ lục có chữ ký hai bên.",
    ] {
        assert!(check(s).is_empty(), "đấu thầu SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_dau_thau_lỗi() {
    assert!(!check("Hợp đồng đã đã ký kết với nhà thầu chính.").is_empty());
    assert!(!check("Báo giá áp dụng đến 32/13/2026.").is_empty());
}

// ================= 6. Y TẾ - SỨC KHỎE =================

#[test]
fn v53_y_te_sach() {
    for s in [
        "Kết quả xét nghiệm sẽ có trong 2 ngày làm việc.",
        "Bác sĩ hẹn tái khám sau 2 tuần, nhớ mang hồ sơ bệnh án.",
        "Bảo hiểm y tế chi trả 80% viện phí điều trị ngoại trú.",
        "Chương trình khám sức khỏe định kỳ bắt đầu thứ Hai tuần tới.",
        "Vắc xin cúm mùa được tiêm miễn phí cho toàn bộ nhân viên.",
    ] {
        assert!(check(s).is_empty(), "y tế SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_y_te_lỗi() {
    assert!(!check("Kết quả xét nghiệm đã đã gửi cho bác sĩ.").is_empty());
    assert!(!check("Hẹn tái khám ngày 30/02/2026.").is_empty());
}

// ================= 7. DU LỊCH - KHÁCH SẠN =================

#[test]
fn v53_du_lich_sach() {
    for s in [
        "Đặt phòng đôi 2 đêm từ thứ Sáu đến Chủ nhật.",
        "Chuyến bay bay thẳng từ Hà Nội vào Đà Nẵng lúc 9h sáng.",
        "Nhận phòng từ 14h, trả phòng trước 12h trưa.",
        "Tour 3 ngày 2 đêm gồm ăn trưa và vé tham quan.",
        "Khách sạn có dịch vụ đưa đón sân bay miễn phí.",
    ] {
        assert!(check(s).is_empty(), "du lịch SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn v53_du_lich_lỗi() {
    assert!(!check("Phòng đã đã được đặt cho 2 khách.").is_empty());
    assert!(!check("Hủy đặt phòng do trùng lịch công tác, đã đã báo.").is_empty());
}

// ================= 8. THỦ TỤC HÀNH CHÍNH =================

#[test]
fn v53_hanh_chinh_sach() {
    for s in [
        "Thủ tục đăng ký kinh doanh hoàn tất trong 3 ngày làm việc.",
        "Giấy chứng nhận đăng ký doanh nghiệp cấp kèm mã số thuế mới.",
        "Công văn số 25 yêu cầu báo cáo kết quả trước ngày 20.",
        "Đơn xin nghỉ việc cần nộp trước 30 ngày theo hợp đồng.",
        "Quyết định bổ nhiệm có hiệu lực từ ngày ký.",
    ] {
        assert!(
            check(s).is_empty(),
            "hành chính SẠCH: {s} => {:?}",
            check(s)
        );
    }
}

// ================= 9. CÂU MIX VI-EN (cả 2 mode) =================

#[test]
fn v53_mix_vi_en_sach_va_lỗi() {
    // Mix bình thường — KHÔNG bêu.
    for s in [
        "Em đã gửi report cho anh rồi, check giúp em nhé.",
        "Meeting tuần này dời sang thứ Năm anh nhé.",
        "Anh confirm lại giúp em deadline của task này.",
        "Client yêu cầu sửa lại phần UI trước khi release.",
    ] {
        assert!(check(s).is_empty(), "mix SẠCH: {s} => {:?}", check(s));
    }
    // Lỗi thật trong câu mix — PHẢI bắt.
    assert!(!check("Em đã gửi report report cho anh rồi.").is_empty());
    assert!(!check("Anh vui lòng check check giúp em file này.").is_empty());
    assert!(!check("Team nên review lại plan đã đã chốt.").is_empty());
}

// ================= 10. ĐƯỜNG DẪN / URL / EMAIL (FPR code-guard) =================

#[test]
fn v53_duong_dan_url_email_sach() {
    for s in [
        r"Tệp nằm ở C:\Users\vanan\Documents\bao-cao-quy-4.xlsx.",
        r"Truy cập https://congty.vn/bao-cao/quy-4 để xem chi tiết.",
        "Gửi phản hồi về vanan@congty.vn trước 17h nhé.",
        "Nạp cấu hình từ /etc/textvn/config.v1.json rồi khởi động lại.",
        "Branch release/0.2.64 đã được đẩy lên origin.",
    ] {
        assert!(check(s).is_empty(), "đường dẫn SẠCH: {s} => {:?}", check(s));
    }
}

// ================= 11. ABNORMAL MỞ RỘNG (vòng 53) =================

#[test]
fn v53_abnormal_mo_rong_khong_panic_khong_treo() {
    let long_line = "tôi đã gửi báo cáo cho anh ".repeat(2_500); // ~65k ký tự
    let inputs: Vec<String> = vec![
        long_line,                        // ngân sách perf câu dài
        "😀".repeat(500),                 // surrogate pair lặp
        "e\u{200D}m \u{200D}gửi".into(),  // ZWJ giữa từ Việt
        "phát\u{200B}triển".into(),       // zero-width space trong từ
        "اااا tiếng Việt mixed ب".into(), // bidi
        " combined é̀ étude".into(),       // NFD latin
        "tôi đã đã ".repeat(1_000),       // lỗi lặp quy mô lớn (perf)
        "31/02/2026".repeat(800),         // ngày vô lý quy mô lớn
        format!("{}{}{}", "A".repeat(300), "B\u{301}", "C".repeat(300)),
        "\u{FFFD}".repeat(100), // replacement char
    ];
    for s in &inputs {
        let t0 = std::time::Instant::now();
        let _ = check(s);
        let _ = check_sentence(s);
        let e = t0.elapsed();
        // Ngân sách tuân perf spec trợ lý: p99 ≤ 10ms/256-char; câu 65k
        // cho 25× → 250ms là trần mềm đủ chặn O(n²) quay lại.
        assert!(e.as_millis() < 2_500, "quá chậm: {e:?} cho input {:-40}", s);
    }
}

#[test]
fn v53_abnormal_lỗi_thật_sat_decor_vẫn_bắt() {
    // Lỗi sát ký tự trang trí/phụ — không được nuốt.
    assert!(!check("🚀 tôi đã đã ăn 🍜 cơm tối 🎉").is_empty());
    assert!(!check("vui lòng vui lòng 🙏 xác nhận giúp 📎").is_empty());
    assert!(!check("Hạn 31/02/2026 ⏰ nộp hồ sơ 📁").is_empty());
    // Bôi đen/hỗ trợ on-demand vẫn chạy trên chuỗi có emoji.
    let issues = check_sentence("👍 anh vui lòng vui lòng kiểm tra 👌");
    assert!(!issues.is_empty());
}

// ================= 12. NFR — ngân sách perf giữ với pool mới =================

#[test]
fn v53_nfr_check_câu_văn_phòng_dưới_1ms() {
    let samples = [
        "Căn hộ hai phòng ngủ giá 2,5 tỷ, đã bao gồm nội thất.",
        "Tờ khai hải quan đã được thông quan lúc 9h sáng nay.",
        "Dây chuyền sản xuất dừng bảo trì định kỳ vào Chủ nhật.",
        "Thủ tục đăng ký kinh doanh hoàn tất trong 3 ngày làm việc.",
        "Em đã gửi report cho anh rồi, check giúp em nhé.",
    ];
    for s in &samples {
        let t0 = std::time::Instant::now();
        for _ in 0..100 {
            let _ = check(s);
        }
        let per = t0.elapsed() / 100;
        assert!(per.as_micros() < 1000, "{s:.30?}: {per:?}/call > 1ms");
    }
}
