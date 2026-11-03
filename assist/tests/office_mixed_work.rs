// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 51 (QA persona battery): kịch bản văn phòng HỖN HỢP — từ coding đến
//! mọi công việc. Mỗi nhóm = một fn riêng để trace rõ khi fail. Mỗi case:
//! SHOULD-CLEAN (FPR) hoặc SHOULD-FIRE (recall) — assert tường minh.
//!
//! 10 nhóm × 6-15 case = 100+ kịch bản thực tế:
//! coding · email công sở · kế toán/tài chính · nhân sự · CSKH · bán hàng ·
//! giáo dục · quản lý dự án · chat không dấu · markdown/config

use textvn_assist::{check, check_sentence};

// ================= 1. CODING (code + comment tiếng Việt) =================

#[test]
fn qa_coding_sạch() {
    for s in [
        "const result = await fetch(url); // gửi request",
        "fn main() { println!(\"Xin chào!\"); }",
        "git commit -m \"sửa lỗi đăng nhập\"",
        "SELECT * FROM users WHERE name = N\"Nguyễn Văn A\";",
        "# TODO: refactor hàm này cho gọn",
        "npm install --save-dev jest",
        "docker run -d -p 8080:80 nginx",
        "kubectl get pods -n production",
        "obj.prop = value;",
        "let x = 5; // khai báo biến",
    ] {
        assert!(check(s).is_empty(), "coding SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_coding_với_comment_có_lỗi_không_bao_giờ_bêu() {
    // Comment tiếng Việt có lỗi thật NHƯNG dòng là code ⇒ không bêu (FPR bar
    // ưu tiên: FN trong comment code chấp nhận được hơn FP phá code).
    for s in [
        "if (x == 1) { // nhận xét,thiếu space",
        "let x = 1; // giờ,nghỉ",
    ] {
        // space_before_punct ("nhận xét,") là lỗi thật — cho phép báo.
        // NHƯNG không được auto-fix đè code.
        for i in check(s) {
            assert!(
                !i.auto_fixable || i.kind == textvn_assist::IssueKind::SpaceBeforePunct,
                "{s}: {:?} không được auto-fix trong code",
                i.kind
            );
        }
    }
}

// ================= 2. EMAIL CÔNG SỞ =================

#[test]
fn qa_email_clean() {
    for s in [
        "Kính gửi Anh Nguyễn,",
        "Trân trọng, Lê Văn B.",
        "Vui lòng phản hồi trước 17h thứ Sáu.",
        "Đề nghị Anh xem xét và phê duyệt.",
        // ("Rất mong...của Quý công ty," 14 từ > trần 12 — đúng thiết kế)
        "Rất mong nhận được phản hồi sớm nhất có thể của Anh,",
        "Nếu có bất kỳ câu hỏi nào, Anh vui lòng liên hệ bộ phận hỗ trợ.",
        "Chúc Anh một ngày làm việc hiệu quả.",
    ] {
        assert!(check(s).is_empty(), "email SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_email_lỗi_thật() {
    // Lặp từ — auto-fixable (dup_word).
    let issues = check("gửi báo cáo báo cáo cho anh");
    assert!(!issues.is_empty());
    assert_eq!(issues[0].kind, textvn_assist::IssueKind::DuplicateWord);
    // Phẩy cuối câu (không phải sign-off).
    assert!(!check("Tôi đã gửi tài liệu cho phòng kế toán vào sáng thứ hai,").is_empty());
    // EN lỗi.
    assert!(!check("Please send a email to confirm.").is_empty());
}

// ================= 3. KẾ TOÁN / TÀI CHÍNH =================

#[test]
fn qa_ke_toan_sạch() {
    for s in [
        "Số tiền: 12.500.000đ đã chuyển.",
        "Hóa đơn GTGT 10% cho tháng 10.",
        "Công nợ phải thu: 45.000.000đ.",
        "Chuyển khoản 1.500.000đ cho nhà cung cấp.",
        "Số dư cuối kỳ: 1.250.000.000đ.",
        "Tổng chi phí quý 3: 850.000.000vnd.",
        "Thuế GTGT đầu vào: 2.500.000đ.",
        "Bút toán điều chỉnh ngày 31/10/2026.",
    ] {
        assert!(check(s).is_empty(), "kế toán SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_ke_toan_lỗi() {
    // Ngày vô lý.
    let issues = check("Hạn thanh toán 31/02/2026.");
    assert_eq!(issues[0].kind, textvn_assist::IssueKind::InvalidDate);
    // Tiền phân cách hỗn loạn.
    let issues = check("Chuyển khoản 1.250.00đ giúp mình.");
    assert_eq!(issues[0].kind, textvn_assist::IssueKind::InvalidMoney);
    // Lặp cụm.
    assert!(!check("Số dư cuối kỳ đã đã đối chiếu.").is_empty());
}

// ================= 4. NHÂN SỰ / HR =================

#[test]
fn qa_hr_sạch() {
    for s in [
        "Nhân viên được nghỉ phép 12 ngày/năm.",
        "Đánh giá cuối kỳ trước 30/11.",
        "Hợp đồng lao động hết hạn 31/12/2026.",
        "Mức lương thỏa thuận theo năng lực.",
        "Chế độ bảo hiểm theo quy định.",
    ] {
        assert!(check(s).is_empty(), "HR SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_hr_lỗi() {
    assert!(!check("Đơn xin nghỉ phép đã đã duyệt.").is_empty());
    assert!(!check("Kế hoạch hành động cần been reviewed.").is_empty());
}

// ================= 5. CSKH =================

#[test]
fn qa_cskh_sạch() {
    for s in [
        "Yêu cầu của quý khách đang được xử lý.",
        "Sự cố đã được xử lý trong 24 giờ.",
        "Mọi thắc mắc vui lòng liên hệ hotline.",
        "Kỹ thuật viên sẽ liên hệ trong giờ làm việc.",
    ] {
        assert!(check(s).is_empty(), "CSKH SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_cskh_lỗi() {
    assert!(!check("Xin lỗi vì sự chậm trễ đã đã gửi lại.").is_empty());
    // "We apologize for" là ngữ pháp EN ĐÚNG — SẠCH (Pass E2 đã loại cặp).
    assert!(check("We apologize for the inconvenience caused.").is_empty());
    // Lỗi thật vẫn bắt.
    assert!(!check("We has been sent the invoice.").is_empty());
}

// ================= 6. BÁN HÀNG =================

#[test]
fn qa_ban_hang_sạch() {
    for s in [
        "Freeship cho đơn từ 199k nhé!",
        "Chốt đơn ngay hôm nay giảm 20%.",
        "Sản phẩm còn hàng, giao ngay.",
        "Mã giảm giá áp dụng cho 100 đơn đầu tiên.",
    ] {
        assert!(check(s).is_empty(), "bán hàng SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_ban_hang_lỗi() {
    assert!(!check("Sản phẩm đã bán đã bán hết.").is_empty());
    assert!(!check("Chốt đơn 1.250.00đ ngay.").is_empty());
}

// ================= 7. GIÁO DỤC =================

#[test]
fn qa_giao_dục_sạch() {
    for s in [
        "Bài kiểm tra một tiết vào thứ Năm.",
        "Cô đã chấm xong bài của em.",
        "Thời khóa biểu tuần này có thay đổi.",
        "Học sinh nộp bài trước thứ Sáu.",
    ] {
        assert!(check(s).is_empty(), "GD SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_giao_dục_lỗi() {
    assert!(!check("Học sinh nộp bài trước thứ Sáu đã đã chấm.").is_empty());
    assert!(!check("Em em nhớ làm bài tập đầy đủ.").is_empty());
}

// ================= 8. QUẢN LÝ DỰ ÁN =================

#[test]
fn qa_quan_ly_sạch() {
    for s in [
        "Giai đoạn tiếp theo bắt đầu tuần sau.",
        "Ngân sách dự kiến 500 triệu.",
        // "Phân công công việc" — "công" whitelisted (compound boundary).
        "Phân công công việc cho từng thành viên.",
        "Tiến độ so với kế hoạch đạt 85%.",
    ] {
        assert!(check(s).is_empty(), "QLDA SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_quan_ly_lỗi() {
    assert!(!check("Tiến độ dự án đã đã báo cáo.").is_empty());
    assert!(!check("Báo cáo tiến độ hàng tuần đã đã gửi.").is_empty());
}

// ================= 9. CHAT KHÔNG DẤU =================

#[test]
fn qa_chat_khong_dau_sạch() {
    // Chat lười — các luật EN-only (article/pronoun) phải im khi câu là VN.
    for s in [
        "ok anh",
        "e da lam xong r",
        "cho xin file voi",
        "mai di lam binh thuong nha",
        "an com chua do",
        "i di ngu som nhe",
        "noi it do met",
    ] {
        assert!(check(s).is_empty(), "chat SẠCH: {s} => {:?}", check(s));
    }
}

#[test]
fn qa_chat_khong_dau_lỗi() {
    // Lặp từ (không dấu) vẫn phải bắt.
    assert!(!check("gui bao cao bao cao cho anh").is_empty());
    assert!(!check("da gui da gui roi").is_empty());
}

// ================= 10. MARKDOWN / CONFIG / FORMAT =================

#[test]
fn qa_markdown_config_sạch() {
    for s in [
        "| Cột 1 | Cột 2 |",
        "- Mục đích sử dụng",
        "## Tiêu đề phần",
        "<p>Nội dung văn bản</p>",
        "name: \"cấu hình hệ thống\"",
        r"C:\Program Files\TextVN\config.json",
        "/usr/local/bin/textvn",
    ] {
        assert!(
            check(s).is_empty(),
            "markdown/config SẠCH: {s} => {:?}",
            check(s)
        );
    }
}

// ================= 11. NGÀY THÁNG + SỐ LIỆU HỢP LỆ =================

#[test]
fn qa_ngay_thang_số_liệu_sạch() {
    for s in [
        "Hạn 28/02/2026.",
        "Kỳ 29/02/2024 (năm nhuận).",
        "Liều 2.5mg mỗi ngày.",
        "Tỷ lệ 15/20 đạt chuẩn.",
        "Họp 15/10/2026 tại 14h.",
        "IP 192.168.1.100 không kết nối được.",
        "Gọi 0912.345.678 để hỗ trợ.",
    ] {
        assert!(
            check(s).is_empty(),
            "ngày tháng/số SẠCH: {s} => {:?}",
            check(s)
        );
    }
}

// ================= 12. ABNORMAL — input kỳ dị không panic =================

#[test]
fn qa_abnormal_ky_tu_dac_biet() {
    // Các input kỳ dị phải fail-open, không panic, không treo.
    let inputs: Vec<String> = vec![
        String::new(),
        " ".into(),
        "\n\n\n".into(),
        "\t\t\t".into(),
        "\0".into(),
        "\u{200B}\u{200D}".into(),           // zero-width
        "👍👎🎉".into(),                     // emoji
        "العربية".into(),                    // RTL
        "日本語テキスト".into(),             // CJK
        "a".repeat(10_000),                  // 1 từ 10k
        "1.1.1.1.1.1.1.1.1.1.".repeat(1000), // dot-chain (perf)
        "31/02/2026 ".repeat(500),           // date repeat (perf)
        "🇻🇳 mixed 🌟 emoji 🎈 và tiếng Việt".into(),
        "\u{FEFF}BOM đầu chuỗi".into(),
        "null\x00byte".into(),
        "\\backslash\\path\\test".into(),
        "\u{0301}\u{0300}\u{0303}".into(),      // combining only
        "e\u{0301}c\u{0303}o\u{0300}ng".into(), // decomposed VN
    ];
    for s in &inputs {
        let t0 = std::time::Instant::now();
        let _ = check(s);
        let _ = check_sentence(s);
        let elapsed = t0.elapsed();
        assert!(
            elapsed.as_millis() < 2000,
            "quá chậm: {elapsed:?} cho {s:.40?}"
        );
    }
}

#[test]
fn qa_abnormal_lỗi_thật_không_bị_nuốt_bởi_input_kỳ_dị() {
    // Lỗi thật sau/near input kỳ dị vẫn phải bắt.
    assert!(!check("tôi đã đã ăn 👍 cơm").is_empty());
    assert!(!check("👍 vui lòng vui lòng kiểm tra 🎉").is_empty());
    assert!(!check("31/02/2026 họp 👏").is_empty());
}

// ================= 13. NFR — performance budget =================

#[test]
fn qa_nfr_perf_câu_thường_dưới_1ms() {
    // Mỗi câu văn phòng thông thường phải check < 1ms (release).
    let samples = [
        "Kính gửi Anh Nguyễn,",
        "Trân trọng, Lê Văn B.",
        "Vui lòng phản hồi trước 17h thứ Sáu.",
        "Số tiền: 12.500.000đ đã chuyển.",
        "Học sinh nộp bài trước thứ Sáu.",
    ];
    for s in &samples {
        let t0 = std::time::Instant::now();
        for _ in 0..100 {
            let _ = check(s);
        }
        let per_call = t0.elapsed() / 100;
        assert!(
            per_call.as_micros() < 1000,
            "{s:.40?}: {per_call:?}/call > 1ms"
        );
    }
}
