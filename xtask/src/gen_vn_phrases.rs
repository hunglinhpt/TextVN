// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask gen-vn-phrases` — sinh bộ dữ liệu ~3000 mẫu câu và cụm từ tiếng Việt
//! chuyên sâu cho trợ lý Text Prediction và Ngữ pháp tiếng Việt (P6).
//!
//! Bao gồm 5 nhóm lĩnh vực chính:
//! 1. Văn phòng & Hành chính công sở (Office & Administration)
//! 2. Công nghệ thông tin & Lập trình (Tech & Software Engineering)
//! 3. Kinh doanh, Quản lý & Tài chính (Business & Finance)
//! 4. Cặp từ ngữ nghĩa & Động từ - Danh từ chuyên sâu (Deep Collocations)
//! 5. Giao tiếp đời sống & Ngôn ngữ thế hệ Gen Z (Everyday & Gen Z)
//!
//! Bất biến: 100% Rust native, 0 dependency ngoài std, format TSV:
//! `context_lower \t predicted_phrase \t weight`

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn run() -> Result<(), String> {
    let mut set: BTreeSet<(String, String, u32)> = BTreeSet::new();

    let mut add = |ctx: &str, pred: &str, weight: u32| {
        let c = ctx.trim().to_lowercase();
        let p = pred.trim().to_string();
        // Invariant R1-2: độ dài cụm chèn không vượt quá MAX_TEXT = 64
        if !c.is_empty() && !p.is_empty() && p.chars().count() <= 60 {
            set.insert((c, p, weight));
        }
    };

    // ----------------------------------------------------
    // NHÓM 1: VĂN PHÒNG & HÀNH CHÍNH CÔNG SỞ (OFFICE)
    // ----------------------------------------------------
    let office_data = [
        ("cộng hòa", "xã hội chủ nghĩa Việt Nam", 100),
        ("độc lập", "- Tự do - Hạnh phúc", 100),
        ("kính gửi", "quý khách hàng", 100),
        ("kính gửi", "ban giám đốc", 98),
        ("kính gửi", "quý công ty", 95),
        ("kính gửi", "quý đối tác", 92),
        ("kính gửi", "phòng nhân sự", 90),
        ("kính gửi", "phòng tài chính kế toán", 88),
        ("kính gửi", "toàn thể cán bộ nhân viên", 85),
        ("kính chuyển", "ban lãnh đạo xem xét", 95),
        ("kính chuyển", "phòng ban chuyên môn", 90),
        ("kính đề nghị", "quý cơ quan hỗ trợ", 95),
        ("trân trọng", "cảm ơn", 100),
        ("trân trọng", "kính chào", 98),
        ("trân trọng", "thông báo", 95),
        ("trân trọng", "gửi lời cảm ơn sâu sắc", 90),
        ("hợp đồng", "kinh tế", 100),
        ("hợp đồng", "lao động", 98),
        ("hợp đồng", "dịch vụ", 95),
        ("hợp đồng", "mua bán", 92),
        ("hợp đồng", "nguyên tắc", 90),
        ("hợp đồng", "thuê văn phòng", 88),
        ("hợp đồng", "bảo hiểm", 85),
        ("biên bản", "bàn giao", 100),
        ("biên bản", "nghiệm thu", 98),
        ("biên bản", "cuộc họp", 95),
        ("biên bản", "làm việc", 92),
        ("biên bản", "thanh lý hợp đồng", 90),
        ("biên bản", "ghi nhớ", 88),
        ("căn cứ", "vào quy định", 100),
        ("căn cứ", "nghị định", 95),
        ("căn cứ", "thông tư", 92),
        ("căn cứ", "luật doanh nghiệp", 90),
        ("căn cứ", "tình hình thực tế", 88),
        ("căn cứ", "kết quả cuộc họp", 85),
        ("vui lòng", "liên hệ", 100),
        ("vui lòng", "kiểm tra lại", 98),
        ("vui lòng", "xác nhận", 95),
        ("vui lòng", "phản hồi sớm", 92),
        ("vui lòng", "xem tài liệu đính kèm", 90),
        ("vui lòng", "cho biết ý kiến", 88),
        ("thông báo", "về việc", 100),
        ("thông báo", "kết luận cuộc họp", 95),
        ("thông báo", "tuyển dụng nhân sự", 90),
        ("thông báo", "lịch nghỉ lễ", 88),
        ("báo cáo", "kết quả thực hiện", 100),
        ("báo cáo", "tài chính quý", 98),
        ("báo cáo", "tiến độ dự án", 95),
        ("báo cáo", "doanh thu tháng", 92),
        ("báo cáo", "thường niên", 90),
        ("kế hoạch", "công tác tuần", 100),
        ("kế hoạch", "kinh doanh năm", 98),
        ("kế hoạch", "triển khai chi tiết", 95),
        ("kế hoạch", "hành động", 90),
        ("quy trình", "làm việc chuẩn", 100),
        ("quy trình", "vận hành", 98),
        ("quy trình", "kiểm soát chất lượng", 95),
        ("quy trình", "xử lý khiếu nại", 90),
        ("tiêu chuẩn", "chất lượng dịch vụ", 100),
        ("an toàn", "thông tin", 100),
        ("an toàn", "lao động", 98),
        ("bảo mật", "dữ liệu khách hàng", 100),
        ("hoàn thành", "đúng tiến độ", 100),
        ("hoàn thành", "nhiệm vụ được giao", 98),
        ("bàn giao", "công việc trước khi nghỉ", 95),
        ("xin", "chào", 100),
        ("xin", "cảm ơn", 95),
        ("xin", "lỗi", 90),
        ("xin", "phép", 85),
        ("giải pháp", "công nghệ", 100),
        ("giải pháp", "tối ưu", 90),
        ("chúc mừng", "năm mới", 100),
        ("hẹn gặp", "lại", 100),
        ("xin ý kiến", "chỉ đạo của ban giám đốc", 95),
        ("xin phép", "nghỉ phép năm", 95),
        ("đồng kính gửi", "các bộ phận liên quan", 90),
        ("phòng ban", "liên quan phối hợp", 92),
    ];

    for &(ctx, pred, weight) in &office_data {
        add(ctx, pred, weight);
    }

    // ----------------------------------------------------
    // NHÓM 2: CÔNG NGHỆ THÔNG TIN & LẬP TRÌNH (TECH)
    // ----------------------------------------------------
    let tech_data = [
        ("phát triển", "phần mềm", 100),
        ("phát triển", "hệ thống", 95),
        ("phát triển", "ứng dụng di động", 92),
        ("phát triển", "tính năng mới", 90),
        ("kiểm thử", "tự động hóa", 100),
        ("kiểm thử", "hiệu năng hệ thống", 95),
        ("kiểm thử", "bảo mật", 90),
        ("kiểm thử", "chấp nhận người dùng", 88),
        ("cơ sở", "dữ liệu quan hệ", 100),
        ("cơ sở", "dữ liệu phân tán", 95),
        ("triển khai", "hệ thống lên môi trường", 100),
        ("triển khai", "giải pháp công nghệ", 95),
        ("triển khai", "bản cập nhật mới", 90),
        ("giao diện", "người dùng thân thiện", 100),
        ("giao diện", "dòng lệnh", 95),
        ("trí tuệ", "nhân tạo tạo sinh", 100),
        ("học máy", "và khai phá dữ liệu", 95),
        ("xử lý", "ngôn ngữ tự nhiên", 100),
        ("xử lý", "ảnh kỹ thuật số", 95),
        ("xử lý", "dữ liệu lớn", 92),
        ("mã nguồn", "mở", 100),
        ("mã nguồn", "chương trình", 95),
        ("tối ưu hóa", "hiệu năng truy vấn", 100),
        ("tối ưu hóa", "tốc độ tải trang", 95),
        ("tối ưu hóa", "dung lượng bộ nhớ", 90),
        ("bảo mật", "hệ thống thông tin", 100),
        ("bảo mật", "mạng máy tính", 95),
        ("điện toán", "đám mây", 100),
        ("mạng", "máy tính cục bộ", 95),
        ("lỗi", "cú pháp", 100),
        ("lỗi", "thời gian chạy", 95),
        ("lỗi", "tràn bộ nhớ", 92),
        ("hệ điều hành", "máy tính", 100),
        ("kiến trúc", "vi dịch vụ", 100),
        ("kiến trúc", "hệ thống phân tán", 95),
        ("kéo", "request duyệt mã nguồn", 90),
        ("tích hợp", "hệ thống liên tục", 95),
        ("bản dựng", "tự động trên máy chủ", 90),
        ("kho chứa", "mã nguồn dự án", 92),
        ("đường dẫn", "tệp tin cấu hình", 90),
        ("tham số", "dòng lệnh", 92),
        ("bộ nhớ", "đệm truy xuất nhanh", 95),
        ("khóa chính", "của bảng dữ liệu", 95),
        ("khóa ngoại", "liên kết bảng", 92),
        ("chỉ mục", "tăng tốc tìm kiếm", 95),
    ];

    for &(ctx, pred, weight) in &tech_data {
        add(ctx, pred, weight);
    }

    // ----------------------------------------------------
    // NHÓM 3: KINH DOANH, QUẢN LÝ & TÀI CHÍNH (BUSINESS)
    // ----------------------------------------------------
    let business_data = [
        ("chiến lược", "kinh doanh dài hạn", 100),
        ("chiến lược", "tiếp thị số", 95),
        ("chiến lược", "phát triển thị trường", 92),
        ("chiến lược", "giữ chân khách hàng", 90),
        ("quản lý", "dự án chuyên nghiệp", 100),
        ("quản lý", "rủi ro tài chính", 98),
        ("quản lý", "chuỗi cung ứng", 95),
        ("quản lý", "mối quan hệ khách hàng", 92),
        ("quản lý", "chất lượng dịch vụ", 90),
        ("quản lý", "nhân sự hiệu quả", 88),
        ("doanh thu", "bán hàng và dịch vụ", 100),
        ("doanh thu", "thuần từ hoạt động", 95),
        ("lợi nhuận", "sau thuế thu nhập", 100),
        ("lợi nhuận", "gộp từ bán hàng", 95),
        ("dòng tiền", "thuần từ kinh doanh", 98),
        ("chi phí", "vận hành doanh nghiệp", 95),
        ("chi phí", "bán hàng và quản lý", 92),
        ("phân tích", "thị trường mục tiêu", 100),
        ("phân tích", "đối thủ cạnh tranh", 98),
        ("phân tích", "dữ liệu người dùng", 95),
        ("khách hàng", "tiềm năng", 100),
        ("khách hàng", "mục tiêu của dự án", 95),
        ("khách hàng", "thân thiết", 92),
        ("đối tác", "chiến lược toàn diện", 100),
        ("đối tác", "kinh doanh tin cậy", 95),
        ("chuỗi", "cung ứng toàn cầu", 100),
        ("tiếp thị", "nội dung hấp dẫn", 95),
        ("tỷ lệ", "chuyển đổi đơn hàng", 100),
        ("tỷ lệ", "tăng trưởng doanh thu", 98),
        ("tỷ lệ", "giữ chân khách hàng", 95),
        ("chỉ số", "đo lường hiệu quả", 100),
        ("nguồn vốn", "đầu tư phát triển", 100),
        ("nguồn vốn", "lưu động của công ty", 95),
        ("giá trị", "cốt lõi của doanh nghiệp", 100),
        ("thị phần", "trên thị trường nội địa", 95),
        ("năng lực", "cạnh tranh cốt lõi", 100),
    ];

    for &(ctx, pred, weight) in &business_data {
        add(ctx, pred, weight);
    }

    // ----------------------------------------------------
    // NHÓM 4: CẶP TỪ NGỮ NGHĨA & ĐỘNG TỪ - DANH TỪ (COLLOCATIONS)
    // ----------------------------------------------------
    let collocation_verbs: &[(&str, &[(&str, u32)])] = &[
        (
            "nâng cao",
            &[
                ("chất lượng dịch vụ", 100),
                ("năng lực cạnh tranh", 98),
                ("hiệu quả công việc", 95),
                ("trình độ chuyên môn", 92),
                ("nhận thức của cộng đồng", 90),
                ("uy tín thương hiệu", 88),
                ("tinh thần trách nhiệm", 85),
                ("vị thế trên thị trường", 82),
            ],
        ),
        (
            "đáp ứng",
            &[
                ("nhu cầu của khách hàng", 100),
                ("yêu cầu kỹ thuật khắt khe", 98),
                ("tiêu chuẩn chất lượng quốc tế", 95),
                ("kỳ vọng của ban lãnh đạo", 92),
                ("tiến độ đề ra", 90),
                ("quy định của pháp luật", 88),
            ],
        ),
        (
            "thúc đẩy",
            &[
                ("tăng trưởng kinh tế", 100),
                ("phát triển bền vững", 98),
                ("tiến độ hoàn thành dự án", 95),
                ("hợp tác song phương", 92),
                ("chuyển đổi số toàn diện", 90),
                ("đổi mới sáng tạo", 88),
            ],
        ),
        (
            "đảm bảo",
            &[
                ("chất lượng sản phẩm", 100),
                ("an toàn thông tin tuyệt đối", 98),
                ("tiến độ thi công công trình", 95),
                ("quyền lợi hợp pháp của người lao động", 92),
                ("tính minh bạch trong báo cáo", 90),
                ("tính liên tục của dịch vụ", 88),
                ("tính bảo mật của hệ thống", 85),
            ],
        ),
        (
            "xây dựng",
            &[
                ("chiến lược phát triển dài hạn", 100),
                ("kế hoạch hành động cụ thể", 98),
                ("hệ thống quản lý chất lượng", 95),
                ("thương hiệu uy tín", 92),
                ("văn hóa doanh nghiệp vững mạnh", 90),
                ("mối quan hệ đối tác tin cậy", 88),
                ("quy trình làm việc chuẩn hóa", 85),
            ],
        ),
        (
            "tăng cường",
            &[
                ("công tác quản lý và giám sát", 100),
                ("hợp tác và trao đổi thông tin", 98),
                ("kiểm tra định kỳ", 95),
                ("đào tạo và phát triển nhân lực", 92),
                ("kết nối giữa các phòng ban", 90),
            ],
        ),
        (
            "áp dụng",
            &[
                ("công nghệ mới vào sản xuất", 100),
                ("biện pháp phòng ngừa rủi ro", 98),
                ("tiêu chuẩn quản lý quốc tế", 95),
                ("chính sách đãi ngộ hấp dẫn", 92),
                ("mô hình kinh doanh tiên tiến", 90),
            ],
        ),
        (
            "giải quyết",
            &[
                ("vấn đề một cách triệt để", 100),
                ("khó khăn và vướng mắc", 98),
                ("khiếu nại của khách hàng", 95),
                ("tranh chấp hợp đồng", 92),
                ("sự cố hệ thống nhanh chóng", 90),
            ],
        ),
        (
            "đưa ra",
            &[
                ("quyết định kịp thời", 100),
                ("giải pháp tối ưu nhất", 98),
                ("đề xuất cải tiến quy trình", 95),
                ("phương án dự phòng", 92),
                ("kết luận cuối cùng", 90),
            ],
        ),
        (
            "thực hiện",
            &[
                ("nhiệm vụ được giao phó", 100),
                ("dự án đúng theo kế hoạch", 98),
                ("quy trình kiểm tra nghiêm ngặt", 95),
                ("cam kết với khách hàng", 92),
                ("nghĩa vụ tài chính", 90),
            ],
        ),
        (
            "phát huy",
            &[
                ("vai trò tiên phong", 100),
                ("thế mạnh của doanh nghiệp", 98),
                ("tiềm năng sẵn có", 95),
                ("hiệu quả hoạt động", 92),
                ("tinh thần đoàn kết", 90),
            ],
        ),
        (
            "cung cấp",
            &[
                ("thông tin chính xác và kịp thời", 100),
                ("dịch vụ chất lượng cao", 98),
                ("giải pháp toàn diện cho doanh nghiệp", 95),
                ("tài liệu hướng dẫn chi tiết", 92),
                ("hỗ trợ kỹ thuật 24/7", 90),
            ],
        ),
        (
            "duy trì",
            &[
                ("sự ổn định của hệ thống", 100),
                ("hiệu suất làm việc cao", 98),
                ("mối quan hệ tốt đẹp", 95),
                ("tiêu chuẩn an toàn nghiêm ngặt", 92),
                ("liên lạc thường xuyên", 90),
            ],
        ),
    ];

    for (verb, targets) in collocation_verbs {
        for &(target, weight) in *targets {
            add(verb, target, weight);
        }
    }

    // ----------------------------------------------------
    // NHÓM 5: GIAO TIẾP ĐỜI SỐNG & GEN Z (EVERYDAY & GEN Z)
    // ----------------------------------------------------
    let genz_data = [
        ("bắt", "trend cực nhanh", 100),
        ("đu", "idol mọi lúc mọi nơi", 95),
        ("flex", "nhẹ thành tích học tập", 100),
        ("flex", "nhẹ chiếc cúp mới", 95),
        ("chill", "chill cuối tuần cùng bạn bè", 100),
        ("cà", "khịa cho vui cửa vui nhà", 95),
        ("quay", "xe vào phút chót", 100),
        ("xu", "cà na ghê luôn", 100),
        ("u", "là trời bất ngờ chưa", 100),
        ("ét", "o ét cứu với", 100),
        ("trầm", "cảm với cái deadline này", 95),
        ("thao", "túng tâm lý đỉnh cao", 100),
        ("hạt", "nhài ghê cơ", 95),
        ("mãi", "mận mãi keo", 100),
        ("tới", "công chuyện luôn rồi", 100),
        ("đỉnh", "nóc kịch trần bay phấp phới", 100),
        ("thả", "thính dính luôn", 95),
        ("chốt", "đơn ầm ầm", 100),
        ("chốt", "deal thành công rực rỡ", 98),
        ("hết", "nước chấm luôn", 100),
        ("chê", "nha không chơi vậy đâu", 95),
        ("gét", "gô làm ngay và luôn", 100),
        ("cứ", "phải gọi là uy tín", 95),
        ("không", "thể nào chuẩn hơn được nữa", 95),
        ("chúc", "bạn một ngày tốt lành", 100),
        ("chúc", "mừng sinh nhật vui vẻ", 98),
        ("chúc", "mừng năm mới an khang thịnh vượng", 95),
        ("hẹn", "gặp lại vào dịp khác", 100),
        ("rất", "vui được làm quen với bạn", 95),
    ];

    for &(ctx, pred, weight) in &genz_data {
        add(ctx, pred, weight);
    }

    // ----------------------------------------------------
    // TẠO TỔ HỢP CÂU HOÀN CHỈNH ĐỂ DỰ ĐOÁN ĐOẠN DÀI (P6)
    // ----------------------------------------------------
    let prefixes = [
        "chúng tôi xin trân trọng",
        "công ty chúng tôi cam kết sẽ",
        "đội ngũ kỹ thuật đang tập trung",
        "ban giám đốc yêu cầu các phòng ban",
        "phòng nhân sự thông báo về việc",
        "phòng tài chính kế toán đã hoàn tất",
        "kính đề nghị ban lãnh đạo xem xét",
        "chúng tôi rất mong nhận được",
        "vui lòng phối hợp với các bên để",
        "nhằm mục đích nâng cao chất lượng",
        "để đảm bảo tiến độ công việc và",
        "với mục tiêu phát triển bền vững",
        "theo đúng quy định hiện hành của",
        "căn cứ vào biên bản cuộc họp",
        "trong thời gian tới chúng tôi sẽ",
        "kính gửi quý khách hàng thông tin",
        "bộ phận chăm sóc khách hàng sẽ",
        "nhóm phát triển phần mềm đang",
        "hệ thống của chúng tôi hiện đã",
        "các bên tham gia đã thống nhất",
        "rất mong quý công ty tạo điều kiện",
        "xin vui lòng kiểm tra lại và",
        "chúng tôi đã chủ động triển khai",
        "hãy liên hệ với chúng tôi khi",
        "ban quản trị dự án đã phê duyệt",
        "người dùng có thể dễ dàng",
        "sau khi hoàn thành việc kiểm thử",
        "để tối ưu hóa hiệu năng của",
        "khi phát hiện sự cố hệ thống cần",
        "đối tác chiến lược của chúng tôi",
        "doanh nghiệp cần tập trung vào việc",
        "trong quý tới chúng tôi dự kiến",
        "báo cáo tài chính quý này đã",
        "hợp đồng kinh tế giữa hai bên sẽ",
        "quy trình vận hành tiêu chuẩn yêu cầu",
        "chúc bạn và gia đình luôn",
        "chúc mừng sinh nhật và chúc bạn",
        "chúc mừng năm mới an khang",
        "chúng ta hãy cùng nhau",
        "hãy bắt đầu ngày mới với",
        "đừng quên kiểm tra kỹ trước khi",
        "tôi xin phép được báo cáo về",
        "em xin phép được cập nhật tiến độ",
        "anh chị vui lòng phản hồi trước",
        "nhờ anh chị hỗ trợ giải quyết",
    ];

    let continuations = [
        // Văn phòng & Hành chính (15)
        "hoàn thành xuất sắc nhiệm vụ được giao",
        "gửi báo cáo tổng kết tuần này đúng hạn",
        "bàn giao công việc đầy đủ trước khi nghỉ",
        "rà soát lại toàn bộ hồ sơ thanh toán",
        "phê duyệt kế hoạch công tác cho quý tới",
        "soạn thảo biên bản nghiệm thu hợp đồng",
        "tổ chức cuộc họp giao ban đầu tuần",
        "đính kèm bảng tổng hợp số liệu chi tiết",
        "gửi lời cảm ơn chân thành đến quý đối tác",
        "xác nhận thông tin lịch hẹn gặp trực tiếp",
        "thông báo lịch nghỉ phép năm chính thức",
        "cập nhật biểu mẫu đánh giá nhân viên mới",
        "hướng dẫn quy trình xin chữ ký điện tử",
        "lưu trữ chứng từ kế toán theo quy định",
        "xử lý dứt điểm các vướng mắc còn tồn đọng",
        // Công nghệ thông tin & Lập trình (15)
        "viết mã nguồn sạch và tối ưu hiệu năng",
        "chạy toàn bộ kiểm thử tự động trước khi đẩy mã",
        "triển khai phiên bản cập nhật lên máy chủ",
        "tối ưu hóa câu lệnh truy vấn cơ sở dữ liệu",
        "sửa lỗi rò rỉ bộ nhớ trong tiến trình ngầm",
        "cấu hình đường ống tích hợp và triển khai liên tục",
        "thiết kế giao diện người dùng trực quan hiện đại",
        "bảo mật hệ thống bằng xác thực hai yếu tố",
        "xây dựng kiến trúc vi dịch vụ có khả năng mở rộng",
        "đồng bộ dữ liệu giữa các máy chủ phân tán",
        "giám sát chỉ số tài nguyên và hiệu năng ứng dụng",
        "xử lý ngoại lệ và ghi log hệ thống chi tiết",
        "tái cấu trúc mô-đun để giảm độ phụ thuộc",
        "kiểm tra độ trễ mạng và thông lượng truyền tải",
        "đóng gói ứng dụng vào container độc lập",
        // Kinh doanh & Quản lý (15)
        "thúc đẩy doanh thu bán hàng trong quý tới",
        "mở rộng thị phần sang các thị trường tiềm năng",
        "cải thiện tỷ lệ chuyển đổi khách hàng tiềm năng",
        "giảm thiểu chi phí vận hành và rủi ro tài chính",
        "đàm phán điều khoản hợp đồng có lợi cho công ty",
        "xây dựng mối quan hệ đối tác chiến lược bền vững",
        "nghiên cứu nhu cầu thực tế của người tiêu dùng",
        "nâng cao trải nghiệm và sự hài lòng của khách hàng",
        "định vị thương hiệu vững chắc trên thị trường",
        "thu hút nguồn vốn đầu tư từ các quỹ uy tín",
        "hoạch định dòng tiền lưu động cho các tháng tới",
        "tối ưu hóa chuỗi cung ứng và phân phối sản phẩm",
        "triển khai chiến dịch tiếp thị đa kênh hiệu quả",
        "đánh giá chỉ số hiệu quả công việc định kỳ",
        "đảm bảo tính minh bạch trong quản trị doanh nghiệp",
        // Hệ thống & OS APIs (10)
        "giải phóng tài nguyên bộ nhớ hệ thống sau khi dùng",
        "xử lý thông điệp cửa sổ trong vòng lặp sự kiện",
        "ghi dữ liệu an toàn xuống đĩa cứng với cờ đồng bộ",
        "lắng nghe kết nối mạng trên cổng chỉ định",
        "đồng bộ luồng bằng biến điều kiện và khóa tương hỗ",
        "đọc tệp tin cấu hình từ đường dẫn hệ điều hành",
        "đăng ký dịch vụ khởi động cùng hệ thống",
        "bắt tín hiệu ngắt và thoát tiến trình an toàn",
        "phân bổ bộ nhớ ảo bằng lời gọi hệ thống",
        "khởi tạo tiến trình con và theo dõi mã trạng thái",
        // Đời sống & Gen Z (10)
        "bắt kịp xu hướng và tạo nội dung ấn tượng",
        "chốt đơn mỏi tay trong ngày hội siêu ưu đãi",
        "cháy hết mình với dự án tâm huyết này",
        "tạo ra năng lượng tích cực cho cả đội ngũ",
        "giữ vững phong độ và đạt kết quả đỉnh nóc",
        "xử lý gọn gàng hạn chót không trượt phát nào",
        "thư giãn cuối tuần để nạp lại năng lượng mới",
        "kết nối và chia sẻ niềm vui cùng bạn bè",
        "tận hưởng từng khoảnh khắc ý nghĩa trong cuộc sống",
        "không ngừng học hỏi và hoàn thiện bản thân mỗi ngày",
    ];

    let mut combo_count = 0;
    for &p in &prefixes {
        for &c in &continuations {
            let weight = 80 + (combo_count % 16);
            add(p, c, weight);
            combo_count += 1;
        }
    }

    println!("Tổng số mẫu câu / cụm từ tiếng Việt đã sinh: {}", set.len());

    let mut out_str = String::with_capacity(set.len() * 60);
    out_str.push_str("# Ngữ cảnh tiếng Việt (Next-Word / Phrase Prediction - P6)\n");
    out_str.push_str("# context_lower\tpredicted_phrase\tweight\n");
    for (ctx, pred, weight) in &set {
        out_str.push_str(ctx);
        out_str.push('\t');
        out_str.push_str(pred);
        out_str.push('\t');
        out_str.push_str(&weight.to_string());
        out_str.push('\n');
    }

    let target_path = Path::new("data/vn_phrases.tsv");
    fs::write(target_path, out_str)
        .map_err(|e| format!("Không thể ghi file {}: {e}", target_path.display()))?;

    println!(
        "Ghi thành công {} dòng vào {}",
        set.len(),
        target_path.display()
    );
    Ok(())
}
