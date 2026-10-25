// SPDX-License-Identifier: GPL-3.0-or-later
//! Kiểm thử tích hợp End-to-End cho Trợ lý ngữ pháp (`textvn-assist`) trong môi trường văn phòng:
//! - Tự động lập kế hoạch sửa câu nhiều lỗi (`plan_sentence_fix`) cho email, báo cáo, biên bản họp.
//! - Đảm bảo 0% dương tính giả (FPR = 0) trên văn bản chứa URL, email, đường dẫn file, inline code,
//!   chữ viết tắt hành chính (`TP.`, `PGS.`, `TS.`), emoticon (`:)`, `:D`), danh sách chấm phẩy `;`.
//! - Bắt chính xác mạo từ `a`/`an` trong câu pha trộn Việt–Anh công sở và bảo vệ tuyệt đối từ `"an"` tiếng Việt.

use textvn_assist::{check, check_sentence, plan_sentence_fix, IssueKind};

#[test]
fn office_sentence_end_to_end_auto_fix_plan() {
    let cases = [
        (
            "em xin gửi gửi  báo cáo tuần này ,anh xem giúp em nhé.",
            "Em xin gửi báo cáo tuần này, anh xem giúp em nhé.",
        ),
        (
            "hôm nay phòng họp sớm.Tuy nhiên  cần chuẩn bị tài liệu trước .",
            "Hôm nay phòng họp sớm. Tuy nhiên cần chuẩn bị tài liệu trước.",
        ),
        (
            "vui lòng kiểm tra nghiã của từ này  trước khi in ấn .",
            "Vui lòng kiểm tra nghĩa của từ này trước khi in ấn.",
        ),
        (
            "báo cáo  tài chính quý này đã hoàn tất ,tuy nhiên cần rà soát lại .",
            "Báo cáo tài chính quý này đã hoàn tất, tuy nhiên cần rà soát lại.",
        ),
    ];

    for (raw, expected_fixed) in cases {
        let issues = check_sentence(raw);
        assert!(
            !issues.is_empty(),
            "phải phát hiện lỗi trong câu văn phòng: `{raw}`"
        );
        let plan = plan_sentence_fix(raw, &issues).expect("phải lập được kế hoạch sửa câu");
        let prefix: String = raw.chars().take(plan.delete_from).collect();
        let fixed = format!("{prefix}{}", plan.replacement);
        assert_eq!(fixed, expected_fixed, "kết quả sửa câu `{raw}`");
        let recheck = check_sentence(&fixed);
        assert!(
            recheck.is_empty(),
            "câu sau khi tự sửa phải sạch lỗi 100%, nhưng còn: {recheck:?}"
        );
    }
}

#[test]
fn office_clean_emails_reports_and_masked_spans_zero_false_positives() {
    let clean_sentences = [
        "Kính gửi Ban Giám đốc, em xin đính kèm báo cáo doanh thu quý III.",
        "Vui lòng gửi phản hồi qua địa chỉ hr-support@company.com.vn trước 17h00.",
        "Tài liệu hướng dẫn đã được cập nhật tại https://internal.company.vn/docs/v2?lang=vi#setup.",
        "File dự toán nằm ở D:\\DuAn\\TaiChinh\\2026\\du_toan_q4.xlsx nhé anh.",
        "Chạy lệnh `cargo xtask preflight` trước khi tạo pull request lên nhánh chính.",
        "Buổi làm việc tại TP. Hồ Chí Minh với PGS. TS. Trần Văn B đã thành công tốt đẹp.",
        "Tự do và an ninh thông tin là ưu tiên hàng đầu của ngân hàng.",
        "Công an thành phố phối hợp giữ gìn trật tự và bình an cho người dân.",
        "Phương án A, kế hoạch B và mục N/A trong phần Q&A đều đã được duyệt.",
        "Cảm ơn chị nhiều nhé :) chúc chị cuối tuần vui vẻ :D",
        "Nội dung cuộc họp gồm: (1) nhân sự; (2) ngân sách; (3) tiến độ triển khai.",
    ];

    for s in clean_sentences {
        let issues = check_sentence(s);
        assert!(
            issues.is_empty(),
            "câu văn phòng chuẩn KHÔNG được báo lỗi giả: `{s}` -> {issues:?}"
        );
    }
}

#[test]
fn office_bilingual_article_an_and_vietnamese_an_protection() {
    // Phát hiện đúng lỗi a/an trong câu pha trộn Việt–Anh công sở:
    let err_cases = [
        ("Em vừa tạo an unit test mới cho hệ thống.", "a"),
        ("Anh tạo giúp em a issue trên Jira nhé.", "an"),
        ("Chúng ta cần chờ khoảng a hour để hệ thống đồng bộ.", "an"),
        ("Đây là an universal solution cho bài toán này.", "a"),
        ("Họ yêu cầu ký a NDA trước khi nhận tài liệu.", "an"),
    ];
    for (s, want_sug) in err_cases {
        let issues = check(s);
        let art = issues
            .iter()
            .find(|i| i.kind == IssueKind::ArticleAn)
            .unwrap_or_else(|| panic!("phải bắt được lỗi ArticleAn trong `{s}`"));
        assert_eq!(
            art.suggestions.first().map(String::as_str),
            Some(want_sug),
            "gợi ý mạo từ cho `{s}`"
        );
    }
}

/// OP-004 + OP-005 (P5): cụm từ hành chính sai chính tả + khoảng trắng
/// trước '%' — sửa đúng qua `plan_sentence_fix` (đường Mode 2/FixAll).
#[test]
fn op004_and_op005_office_collocation_and_percent_spacing() {
    let cases = [
        (
            "Chúng tôi xin bổ xung tài liệu và kiểm xoát tiến độ đạt 100 %.",
            "Chúng tôi xin bổ sung tài liệu và kiểm soát tiến độ đạt 100%.",
        ),
        (
            "Đơn vị xử dụng ngân sách rất chính sác sau khi sát nhập.",
            "Đơn vị sử dụng ngân sách rất chính xác sau khi sáp nhập.",
        ),
        (
            "Anh ấy giành thời gian ôn tập để dành chiến thắng.",
            "Anh ấy dành thời gian ôn tập để giành chiến thắng.",
        ),
    ];
    for (raw, expected) in cases {
        let issues = check_sentence(raw);
        assert!(!issues.is_empty(), "phải phát hiện lỗi trong: `{raw}`");
        let plan = plan_sentence_fix(raw, &issues).unwrap_or_else(|| panic!("phải lập được kế hoạch sửa: `{raw}`"));
        let prefix: String = raw.chars().take(plan.delete_from).collect();
        let fixed = format!("{prefix}{}", plan.replacement);
        assert_eq!(fixed, expected, "kết quả sửa câu `{raw}`");
    }
}
