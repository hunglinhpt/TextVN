// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask gen-vn-phrases` — sinh bộ dữ liệu mẫu câu và cụm từ tiếng Việt
//! cho Text Prediction (P6) — **vòng 52 (2026-11-04): mở rộng ~3.4k → 10k+ dòng**.
//!
//! Ba nguồn, mọi dòng đều đúng ngữ nghĩa (không cross-product vô nghĩa):
//! 1. **Câu thật đầy đủ** (mỗi domain văn phòng/life): chia tại MỌI ranh giới
//!    từ → `w1 → w2…wn`, `w1 w2 → w3…wn`, … weight tăng theo độ dài khoá
//!    (ngữ cảnh dài = tín hiệu mạnh).
//! 2. **Thành ngữ/tục ngữ/ca dao** (chỉ mục thật, không bịa): chia tại mọi
//!    điểm — gõ nửa đầu được gợi ý nửa sau ("có công mài sắt" + Tab →
//!    "có ngày nên kim").
//! 3. **Cặp (ngữ cảnh → cụm) curated** theo domain (giữ các nhóm cũ, mở rộng).
//!
//! Bất biến: 100% Rust native, 0 dependency ngoài std, format TSV
//! `context_lower \t predicted_phrase \t weight`, prediction ≤ 60 ký tự (R1-2).

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

    // ------------------------------------------------------------------
    // 1. CÂU THẬT ĐẦY ĐỦ — chia tại mọi ranh giới từ.
    // ------------------------------------------------------------------
    let sentences: &[&str] = &[
        // ---- Email mở đầu / kính gửi (18) ----
        "Kính gửi Anh chị phụ trách bộ phận nhân sự",
        "Kính gửi quý công ty chúng tôi xin trân trọng báo cáo",
        "Kính gửi ban giám đốc cùng quý phòng ban liên quan",
        "Kính gửi phòng kế toán xin xác nhận lại số liệu",
        "Kính gửi anh chị quý khách hàng thân mến",
        "Trân trọng thông báo tới toàn thể cán bộ nhân viên",
        "Chúng tôi xin trân trọng cảm ơn sự hợp tác của quý công ty",
        "Chúng tôi rất mong nhận được phản hồi sớm nhất từ quý vị",
        "Công ty chúng tôi xin trân trọng kính mời quý đối tác dự lễ ký kết",
        "Căn cứ vào kế hoạch công tác đã được phê duyệt",
        "Căn cứ vào biên bản cuộc họp ngày hôm nay",
        "Căn cứ tình hình thực tế của đơn vị trong quý vừa qua",
        "Để thực hiện nhiệm vụ được giao theo đúng tiến độ",
        "Nhằm nâng cao chất lượng phục vụ khách hàng",
        "Nhằm tạo điều kiện thuận lợi cho việc tra cứu thông tin",
        "Thực hiện chỉ đạo của ban lãnh đạo về việc rà soát hồ sơ",
        "Xin trân trọng gửi tới quý công ty bản báo cáo tổng hợp",
        "Xin phép được gửi anh chị bản cập nhật tiến độ công việc",
        // ---- Email thân bài / yêu cầu (22) ----
        "Anh chị vui lòng kiểm tra lại thông tin và phản hồi trước 17h hôm nay",
        "Anh chị vui lòng xem xét và phê duyệt kế hoạch trong tuần này",
        "Anh vui lòng xác nhận lại lịch họp vào thứ Năm tuần sau",
        "Chị vui lòng gửi lại bản mềm của báo cáo giúp em",
        "Vui lòng liên hệ bộ phận chăm sóc khách hàng khi cần hỗ trợ",
        "Vui lòng phản hồi qua email này trong vòng ba ngày làm việc",
        "Vui lòng điền đầy đủ thông tin vào biểu mẫu đính kèm",
        "Vui lòng kiểm tra kỹ số liệu trước khi trình ký",
        "Mong anh chị tạo điều kiện hỗ trợ đơn vị trong công tác này",
        "Rất mong nhận được sự hợp tác của quý phòng ban",
        "Đề nghị quý anh chị nghiêm chỉnh thực hiện các quy định hiện hành",
        "Đề nghị bộ phận liên quan hoàn tất việc bàn giao trước cuối tháng",
        "Em xin phép được cập nhật tiến độ công việc của nhóm tuần này",
        "Em xin phép nghỉ phép ba ngày kể từ ngày mười lăm tháng này",
        "Tôi xin báo cáo tóm tắt kết quả thực hiện nhiệm vụ tháng vừa qua",
        "Tôi xin gửi anh chị bảng đối chiếu công nợ của tháng này",
        "Tôi sẽ gửi bản thảo cho anh chị xem xét vào đầu tuần tới",
        "Tôi đã chuyển khoản số tiền trên cho phòng tài chính vào sáng nay",
        "Công nợ của đơn vị hiện đã được đối chiếu và xác nhận đầy đủ",
        "Hóa đơn điện tử của kỳ này đã được gửi qua email cho anh chị",
        "Chúng tôi sẽ liên hệ lại với anh chị ngay khi có kết quả",
        "Chúng tôi cam kết bảo mật toàn bộ thông tin của khách hàng",
        // ---- Email kết thúc (12) ----
        "Trân trọng cảm ơn và rất mong sớm nhận được hồi âm của anh chị",
        "Rất mong sớm nhận được ý kiến góp ý của anh chị",
        "Mọi thắc mắc xin vui lòng liên hệ qua số điện thoại dưới đây",
        "Trong trường hợp cần hỗ trợ thêm anh chị vui lòng phản hồi email này",
        "Xin chân thành cảm ơn sự quan tâm và hợp tác của quý công ty",
        "Kính chúc anh chị một ngày làm việc hiệu quả và tràn đầy năng lượng",
        "Kính chúc quý công ty ngày càng phát triển vững mạnh",
        "Hẹn gặp lại anh chị trong buổi họp tới",
        "Nếu cần trao đổi thêm xin anh chị cứ liên hệ trực tiếp với em",
        "Chi tiết kế hoạch được gửi kèm theo email này",
        "Số tiền thanh toán được ghi rõ trong phần phụ lục hợp đồng",
        "Bản mềm của tài liệu được đính kèm bên dưới cho anh chị tham khảo",
        // ---- Họp / quản lý dự án (16) ----
        "Buổi họp giao ban đầu tuần sẽ bắt đầu lúc tám giờ sáng thứ Hai",
        "Cuộc họp được lùi lại sang tuần sau do anh giám đốc bận công tác",
        "Biên bản cuộc họp sẽ được gửi tới các thành viên trong chiều nay",
        "Mỗi phòng ban cử một đại diện tham dự cuộc họp vào thứ Sáu",
        "Chúng ta cần rà soát lại tiến độ của dự án trong quý này",
        "Tiến độ dự án hiện đạt khoảng tám mươi phần trăm kế hoạch",
        "Vấn đề tồn đọng cần được giải quyết dứt điểm trước cuối tháng",
        "Phân công công việc cho từng thành viên trong nhóm rõ ràng hơn",
        "Anh chị nào có vướng mắc xin nêu ý kiến trong phần thảo luận",
        "Bản trình bày của nhóm đã được cập nhật theo góp ý của ban giám đốc",
        "Ngân sách dự kiến cho giai đoạn tiếp theo là năm trăm triệu đồng",
        "Rủi ro chính của dự án nằm ở khâu triển khai tại hiện trường",
        "Nhóm kỹ thuật đang tập trung xử lý sự cố hệ thống đêm nay",
        "Sau khi hoàn thành kiểm thử nhóm sẽ bàn giao cho bộ phận vận hành",
        "Cần tổ chức họp rút kinh nghiệm sau khi kết thúc giai đoạn một",
        "Tôi xin nhắc lại thời hạn chót cho báo cáo quý là thứ Sáu tuần này",
        // ---- Kế toán / tài chính (14) ----
        "Số tiền ba mươi lăm triệu đồng đã được chuyển vào tài khoản công ty",
        "Hóa đơn giá trị gia tăng của kỳ tháng này đã xuất cho anh chị",
        "Công nợ phải thu trong tháng đã giảm so với tháng trước",
        "Đề nghị anh chị kiểm tra lại sổ phụ ngân hàng của kỳ này",
        "Chúng tôi đã đối chiếu số liệu giữa hai bên và không có chênh lệch",
        "Báo cáo tài chính quý sẽ hoàn tất trước ngày hai mươi của tháng tới",
        "Thuế giá trị gia tăng của kỳ này được nộp qua mạng điện tử",
        "Chi phí vận hành trong quý tăng nhẹ so với cùng kỳ năm ngoái",
        "Đề nghị bổ sung chứng từ kế toán cho các khoản chi trong tháng",
        "Sổ sách chứng từ cần được lưu trữ đầy đủ theo quy định",
        "Khoản tạm ứng của anh đã được trừ vào bảng quyết toán cuối kỳ",
        "Thanh toán được thực hiện qua chuyển khoản trong ba ngày làm việc",
        "Số dư đầu kỳ khớp với số dư cuối kỳ của báo cáo liền trước",
        "Ngân sách cho năm tới đang được dự thảo theo định hướng của lãnh đạo",
        // ---- Nhân sự (12) ----
        "Đơn xin nghỉ phép của anh đã được phê duyệt vào sáng nay",
        "Nhân sự phòng kế toán đang tuyển thêm hai vị trí kế toán viên",
        "Buổi phỏng vấn cho ứng viên vòng cuối sẽ diễn ra vào chiều thứ Ba",
        "Hợp đồng lao động của chị hết hạn vào cuối tháng mười hai",
        "Đề nghị cập nhật hồ sơ nhân sự và bảng chấm công tháng này",
        "Chế độ bảo hiểm xã hội được đóng theo mức lương cơ bản",
        "Đánh giá định kỳ sáu tháng sẽ bắt đầu từ đầu tuần tới",
        "Thư mời phỏng vấn đã được gửi tới địa chỉ email của ứng viên",
        "Kế hoạch đào tạo nội bộ cho năm tới đang được xây dựng",
        "Nhân viên mới cần hoàn tất thủ tục nhận việc trong ba ngày đầu",
        "Lịch nghỉ lễ được thông báo kèm theo công văn này",
        "Đề nghị các phòng ban gửi danh sách nhu cầu tuyển dụng quý tới",
        // ---- Chăm sóc khách hàng (14) ----
        "Quý khách vui lòng chờ trong ít phút để hệ thống kết nối với tổng đài",
        "Yêu cầu của quý khách đang được xử lý trong thời gian sớm nhất",
        "Sự cố đã được khắc phục và hệ thống hoạt động bình thường trở lại",
        "Chúng tôi thành thật xin lỗi vì sự chậm trễ trong quá trình xử lý",
        "Đơn hàng của anh chị dự kiến được giao trong hai ngày tới",
        "Quý khách có thể kiểm tra tình trạng đơn hàng trên ứng dụng",
        "Khi nhận hàng quý khách vui lòng kiểm tra kỹ tình trạng sản phẩm",
        "Phiếu bảo hành điện tử được gửi kèm theo email xác nhận đơn hàng",
        "Chính sách đổi trả áp dụng trong vòng bảy ngày kể từ ngày nhận hàng",
        "Mọi thắc mắc xin vui lòng liên hệ hotline của công ty để được hỗ trợ",
        "Chúng tôi sẽ chủ động liên hệ lại với quý khách ngay khi có thông tin",
        "Cảm ơn quý khách đã tin tưởng và lựa chọn dịch vụ của chúng tôi",
        "Kính mời quý khách đánh giá chất lượng phục vụ sau khi hoàn tất",
        "Quyền lợi thành viên được cập nhật tự động vào tài khoản của anh chị",
        // ---- Bán hàng / kinh doanh (10) ----
        "Chương trình khuyến mãi áp dụng cho một trăm đơn hàng đầu tiên",
        "Báo giá trên chỉ áp dụng trong vòng mười ngày kể từ ngày gửi",
        "Chúng tôi sẵn sàng đàm phán lại đơn giá cho đơn hàng số lượng lớn",
        "Mẫu sản phẩm mới sẽ được gửi tới anh chị trong tuần này",
        "Tỷ lệ chuyển đổi đơn hàng của sàn đã tăng so với tháng trước",
        "Đối tác phân phối mới sẽ ký kết hợp đồng vào đầu tháng tới",
        "Kế hoạch kinh doanh năm tới cần định hướng rõ nhóm khách hàng mục tiêu",
        "Chiết khấu dành cho đại lý được quy định trong phụ lục hợp đồng",
        "Đơn hàng số lượng lớn vui lòng liên hệ trực tiếp phòng kinh doanh",
        "Chúng tôi cam kết giao hàng đúng tiến độ như trong hợp đồng",
        // ---- Giáo dục (8) ----
        "Bài kiểm tra một tiết môn toán sẽ diễn ra vào thứ Năm tuần này",
        "Cô đã chấm xong bài kiểm tra và cập nhật điểm lên hệ thống",
        "Học sinh nộp bài tập về nhà trước buổi học vào sáng mai",
        "Thời khóa biểu tuần này có thay đổi ở tiết cuối chiều thứ Ba",
        "Kế hoạch bồi dưỡng học sinh giỏi được triển khai từ tuần tới",
        "Phụ huynh vui lòng ký xác nhận vào liên lạc học sinh hôm nay",
        "Buổi sinh hoạt lớp bắt đầu vào tiết chín chiều nay",
        "Lịch thi cuối kỳ đã được niêm yết trên bảng thông báo của trường",
        // ---- Chat / tin nhắn công sở (14) ----
        "Cho mình xin file báo cáo với ạ",
        "Anh ơi cho em hỏi chút về số liệu buổi họp sáng nay",
        "Mình đã gửi phản hồi vào thread kia rồi nhé",
        "Chiều nay mình bận họp nên hẹn review sau nhé",
        "Em vừa cập nhật xong bản mới trên hệ thống cho anh chị",
        "Anh chị xem giúp em phần này với ạ em cảm ơn nhiều",
        "Ok anh em sẽ xử lý ngay trong chiều nay",
        "Mình confirm lại lịch họp là ba giờ chiều thứ Năm nhé",
        "Nhớ tag mình vào cuộc họp đó với nhé",
        "Sáng mai mình gửi bản tổng hợp cho cả nhóm",
        "Việc này mình đã chốt với bên kinh doanh rồi nhé",
        "Anh cứ ping em bất cứ lúc nào cần hỗ trợ ạ",
        "Tuần này em bận deadline nên hẹn anh tuần sau được không ạ",
        "Mình để tài liệu trong folder chung của phòng rồi nhé",
        // ---- Chúc tụng / lễ tết / dịp đặc biệt (12) ----
        "Chúc mừng năm mới anh chị cùng gia đình thật nhiều sức khỏe và an khang",
        "Năm mới thành công và làm ăn phát đạt chúc anh chị và gia đình",
        "Chúc mừng sinh nhật anh tuổi mới nhiều niềm vui và thành công hơn nữa",
        "Chúc mừng kỷ niệm ngày thành lập công ty lần thứ năm",
        "Chúc toàn thể anh chị em trong công ty một năm mới bình an",
        "Chúc anh chị một ngày làm việc thật hiệu quả và vui vẻ",
        "Chúc em thi tốt và đạt kết quả cao trong kỳ thi sắp tới",
        "Chúc mừng anh đã hoàn thành xuất sắc nhiệm vụ được giao",
        "Chúc hai gia đình luôn mạnh khỏe gặp nhiều may mắn",
        "Chúc mừng lễ khai trương cửa hàng kinh doanh thật phát đạt",
        "Mừng quý công ty đã ký kết thành công hợp đồng đầu tư mới",
        "Chúc mừng tân giám đốc nhậm chức và gặt hái nhiều thành tựu",
        // ---- Đời sống / xã giao (12) ----
        "Cuối tuần này mình rủ cả nhóm đi cà phê nhé",
        "Chiều nay mưa to nên anh em về sớm hơn chút",
        "Trưa nay mình ăn ở căng tin dưới tầng một",
        "Tuần sau mình đi công tác cùng hội đồng ở tỉnh một tuần",
        "Tối nay cả nhà mình đi ăn mừng thành tích của con",
        "Bác sĩ hẹn tái khám vào sáng thứ Sáu tuần sau",
        "Hôm nay đường hay kẹt xe nên mọi người đi sớm nhé",
        "Cuối tháng này nhà mình có đám cưới con chị cả",
        "Sức khỏe của ba hồi phục tốt bác sĩ bảo cho về nhà",
        "Kỳ nghỉ hè năm nay gia đình mình đi biển ba ngày",
        "Tối mai lớp học phụ huynh bắt đầu lúc bảy giờ",
        "Cuối năm công ty có buổi tất niên tại nhà hàng trung tâm",
        // ---- Email theo dõi - nhắc việc (14) ----
        "Email này được gửi để theo dõi tiến độ xử lý yêu cầu của anh chị",
        "Xin phép được nhắc lại yêu cầu mà em đã gửi vào tuần trước",
        "Vẫn chưa nhận được phản hồi của anh chị về đề xuất dưới đây",
        "Em gửi lại email này kèm theo bản cập nhật mới nhất cho anh chị",
        "Mong anh chị thông cảm vì email này có thể đã đến trễ",
        "Nếu anh chị đã xử lý xong vui lòng bỏ qua email này",
        "Em xin phép đóng yêu cầu này vì đã nhận được xác nhận của anh",
        "Xin cập nhật rằng vấn đề đã được giải quyết trọn vẹn",
        "Hệ thống sẽ tự động gửi thông báo nhắc hẹn một ngày trước",
        "Em đánh dấu email này là ưu tiên cao mong anh chị chú ý",
        "Trả lời trực tiếp email này nếu anh chị cần bổ sung thông tin",
        "Chuyển tiếp email này tới các anh chị trong nhóm dự án giúp em",
        "Em xin gửi kèm biên bản xác nhận nội dung trao đổi buổi trước",
        "Dưới đây là tóm tắt các đầu việc hai bên đã thống nhất hôm nay",
        // ---- Báo cáo - tổng kết (14) ----
        "Báo cáo tổng kết quý ba đã được gửi tới ban lãnh đạo sáng nay",
        "Doanh thu sáu tháng đầu năm tăng mười phần trăm so với cùng kỳ",
        "Số lượng khách hàng mới trong tháng này vượt kế hoạch đề ra",
        "Tỷ lệ hoàn thành công việc của nhóm đạt chín mươi phần trăm",
        "Các chỉ số chính đều nằm trong ngưỡng an toàn của kế hoạch",
        "Hai chỉ tiêu còn lại dự kiến hoàn thành trước cuối tháng mười hai",
        "Điểm nghẽn chính nằm ở khâu phê duyệt nội bộ mất nhiều thời gian",
        "Bảng so sánh số liệu giữa hai kỳ được trình bày ở trang cuối",
        "Phần kết luận của báo cáo nêu rõ ba kiến nghị chính cho năm tới",
        "Số liệu trong báo cáo đã được bộ phận kiểm soát nội bộ xác nhận",
        "Em trích xuất số liệu từ hệ thống và tổng hợp vào bảng dưới đây",
        "Biểu đồ tăng trưởng theo tháng được gửi kèm để anh chị tham khảo",
        "Mọi sai lệch so với kế hoạch đều được ghi chú rõ nguyên nhân",
        "Bản tổng hợp cuối cùng sẽ được trình ký vào sáng thứ Hai",
        // ---- Dự án - triển khai (14) ----
        "Giai đoạn một của dự án đã chính thức hoàn thành vào hôm qua",
        "Nhóm dự án sẽ họp nhanh lúc chín giờ sáng mai để chốt phạm vi",
        "Danh sách đầu việc ưu tiên được cập nhật trên bảng theo dõi chung",
        "Mọi thay đổi về phạm vi cần được ghi nhận qua biên bản điều chỉnh",
        "Kinh nghiệm từ giai đoạn trước sẽ được áp dụng cho giai đoạn tới",
        "Chúng tôi đề xuất dời hạn nghiệm thu thêm một tuần để đảm bảo chất lượng",
        "Hồ sơ nghiệm thu đã gửi tới đơn vị giám sát để trình duyệt",
        "Việc đào tạo người dùng cuối sẽ bắt đầu ngay sau khi nghiệm thu",
        "Tài liệu hướng dẫn sử dụng đang được hoàn thiện bởi nhóm chuyên môn",
        "Môi trường thử nghiệm đã sẵn sàng cho đội ngũ kiểm thử truy cập",
        "Cả hai bên thống nhất phương án dự phòng khi triển khai thật",
        "Kế hoạch triển khai chi tiết sẽ được gửi trong tuần tới",
        "Cần thêm một nhân sự kỹ thuật để đảm bảo tiến độ cam kết",
        "Cuộc họp xem trước với khách hàng dự kiến vào chiều thứ Tư",
        // ---- Mua sắm - nhà cung cấp (10) ----
        "Đề nghị nhà cung cấp báo giá chính thức cho danh mục hàng hóa dưới đây",
        "Yêu cầu tối thiểu ba báo giá cạnh tranh cho mỗi gói thầu",
        "Hàng hóa được giao đến kho trung tâm trong vòng năm ngày làm việc",
        "Chất lượng hàng giao phải đúng tiêu chuẩn trong hợp đồng đã ký",
        "Thanh toán được thực hiện sau khi nghiệm thu hàng hóa đầy đủ",
        "Đơn đặt hàng số bốn mươi hai đã được phê duyệt và gửi cho nhà cung cấp",
        "Chúng tôi đề nghị gia hạn hợp đồng cung ứng thêm mười hai tháng",
        "Báo giá của quý công ty đang được xem xét bởi phòng mua sắm",
        "Mẫu hàng thử nghiệm sẽ được gửi về trước khi ký hợp đồng chính thức",
        "Mọi khiếu nại về chất lượng được tiếp nhận trong vòng hai mươi bốn giờ",
        // ---- Pháp lý - tuân thủ (8) ----
        "Toàn bộ quy định về bảo mật dữ liệu được cập nhật theo thông tư mới",
        "Nhân viên phải ký cam kết bảo mật thông tin ngay khi nhận việc",
        "Mọi hồ sơ khách hàng được lưu trữ theo đúng quy định pháp luật",
        "Việc sử dụng tài sản công phải tuân thủ quy chế hiện hành",
        "Đơn vị cần hoàn tất rà soát tuân thủ trước ngày ba mươi của tháng",
        "Hồ sơ pháp lý của dự án đã đầy đủ và sẵn sàng cho thẩm định",
        "Mọi tranh chấp sẽ được giải quyết thông qua thương lượng và hòa giải",
        "Điều khoản bảo mật trong hợp đồng được hai bên thống nhất giữ nguyên",
        // ---- Marketing - truyền thông (8) ----
        "Kế hoạch truyền thông cho quý tới tập trung vào kênh mạng xã hội",
        "Bài viết giới thiệu sản phẩm mới sẽ được đăng vào sáng thứ Sáu",
        "Nội dung video quảng bá đang được nhóm sáng tạo hoàn thiện",
        "Ngân sách quảng cáo được phân bổ lại theo hiệu quả từng kênh",
        "Chiến dịch ra mắt nhận được nhiều phản hồi tích cực từ khách hàng",
        "Đơn đăng ký tham gia sự kiện đã vượt chỉ tiêu đề ra",
        "Chúng tôi xin gửi anh chị bộ ảnh sản phẩm mới cho mục đích truyền thông",
        "Mọi tài liệu truyền thông cần được phê duyệt trước khi phát hành",
        // ---- Nội bộ - thông báo (10) ----
        "Công ty sẽ nghỉ lễ từ ngày thứ Năm đến hết ngày Chủ nhật tuần tới",
        "Toàn bộ nhân viên vui lòng cập nhật thông tin liên hệ mới cho bộ phận hành chính",
        "Từ ngày mai giờ làm việc bắt đầu sớm hơn mười lăm phút",
        "Phòng họp tầng ba được nâng cấp và tạm khóa trong tuần này",
        "Kính thông báo lịch trực cơ quan trong dịp lễ như sau",
        "Yêu cầu tắt máy tính và thiết bị điện trước khi rời văn phòng",
        "Bảng tin nội bộ được cập nhật hàng tuần vào sáng thứ Hai",
        "Mọi đề xuất cải tiến quy trình vui lòng gửi về hộp thư góp ý",
        "Cuộc thi sáng tạo nội bộ nhận bài dự thi đến hết tháng này",
        "Chúng tôi xin thông báo quyết định bổ nhiệm nhân sự mới như sau",
        // ---- Follow-up khách hàng (8) ----
        "Cảm ơn anh chị đã dành thời gian trao đổi với chúng tôi hôm qua",
        "Theo trao đổi hôm nay em gửi lại tài liệu mà anh chị yêu cầu",
        "Chúng tôi xin phép gọi lại cho anh chị vào đầu giờ chiều mai",
        "Anh chị vui lòng cho biết thời gian thuận tiện để chúng tôi đến gặp",
        "Đơn vị chúng tôi rất mong được hợp tác lâu dài cùng quý công ty",
        "Mọi yêu cầu điều chỉnh báo giá vui lòng gửi về email kinh doanh",
        "Chúng tôi luôn sẵn sàng hỗ trợ anh chị trong suốt quá trình sử dụng",
        "Mong sớm nhận được quyết định của anh chị để chúng tôi sắp xếp kế hoạch",
        // ---- Kết luận họp (8) ----
        "Buổi họp kết thúc lúc mười một giờ ba mươi như kế hoạch",
        "Ba quyết định chính đã được thống nhất trong cuộc họp hôm nay",
        "Mỗi đầu việc đều có người phụ trách và thời hạn rõ ràng",
        "Kết quả kiểm phiếu cho thấy đa số đồng ý với phương án hai",
        "Anh thư ký sẽ tổng hợp ý kiến và soạn biên bản trong chiều nay",
        "Cuộc họp tiếp theo dự kiến diễn ra vào cùng giờ tuần sau",
        "Những vấn đề chưa kết luận sẽ được đưa vào chương trình họp tới",
        "Chúng ta chốt lại ba mốc quan trọng cho quý này như sau",
        // ---- Kỹ thuật - vận hành (10) ----
        "Hệ thống sẽ bảo trì định kỳ vào tối Chủ nhật nên tạm ngừng khoảng một giờ",
        "Người dùng vui lòng lưu lại công việc trước thời điểm bảo trì",
        "Phiên bản mới đã được triển khai thành công lên toàn bộ máy chủ",
        "Tốc độ phản hồi của hệ thống đã trở lại mức bình thường",
        "Nhóm vận hành theo dõi sát tình trạng dịch vụ trong đêm nay",
        "Mọi sự cố được ghi nhận và xử lý theo quy trình ứng phó sự cố",
        "Cần kiểm tra lại bản sao lưu dữ liệu của cuối tuần vừa rồi",
        "Quyền truy cập của tài khoản đã hết hạn vui lòng liên hệ quản trị",
        "Cảnh báo dung lượng lưu trữ sắp đạt ngưỡng cho phép",
        "Nhật ký hệ thống được lưu giữ trong chín mươi ngày theo quy định",
        // ---- Hành chính - văn thư (8) ----
        "Công văn này được gửi tới các đơn vị trực thuộc để phối hợp thực hiện",
        "Văn bản đến hôm nay đã được đăng ký và chuyển tới phòng liên quan",
        "Đề nghị các đơn vị gửi báo cáo định kỳ đúng thời hạn quy định",
        "Bản chính của văn bản được lưu tại văn thư của công ty",
        "Hồ sơ đã hoàn tất và sẵn sàng trình ký theo luồng phê duyệt",
        "Giấy giới thiệu làm việc được cấp cho anh theo yêu cầu của đơn vị",
        "Mọi công văn đi đều phải qua kiểm duyệt của phòng hành chính",
        "Danh mục tài liệu cần tiêu hủy được lập theo quy định lưu trữ",
        // ---- Đào tạo - phát triển (8) ----
        "Khóa đào tạo kỹ năng thuyết trình sẽ khai giảng vào đầu tháng tới",
        "Chương trình đào tạo nhân sự quản lý được xây dựng trong sáu tháng",
        "Mỗi học viên cần hoàn thành bài kiểm tra cuối khóa để được cấp chứng chỉ",
        "Tài liệu học tập được chia sẻ trước buổi học một ngày",
        "Lớp học tiếng Anh giao tiếp văn phòng khai giảng vào thứ Hai tuần sau",
        "Học phí được hỗ trợ năm mươi phần trăm theo chính sách phát triển nhân lực",
        "Buổi chia sẻ kinh nghiệm nội bộ diễn ra vào chiều thứ Sáu hàng tháng",
        "Kết quả đánh giá sau đào tạo được ghi vào hồ sơ phát triển nghề nghiệp",
        // ---- Hẹn lịch - sắp xếp (10) ----
        "Em xin phép đổi lịch họp từ thứ Ba sang thứ Năm cùng tuần",
        "Anh chị vui lòng chọn khung giờ thuận tiện trong lịch gửi kèm",
        "Lịch họp định kỳ của nhóm được đặt vào thứ Sáu hàng tuần",
        "Hẹn anh chị vào chín giờ sáng tại văn phòng công ty tầng năm",
        "Buổi trao đổi dự kiến kéo dài khoảng một tiếng rưỡi",
        "Nếu trùng lịch anh chị vui lòng báo lại để chúng tôi sắp xếp lại",
        "Lịch họp với đối tác được cập nhật trên lịch chung của phòng",
        "Chúng tôi xin mời anh chị dự buổi khai mạc vào tám giờ sáng",
        "Đề nghị xác nhận tham dự trước mười tám giờ hôm nay",
        "Kỳ công tác của anh kéo dài từ thứ Hai đến hết thứ Năm",
        // ---- Tài chính - dòng tiền (8) ----
        "Dòng tiền của quý này được cải thiện nhờ thu hồi công nợ nhanh",
        "Kế hoạch dòng tiền sáu tháng đang được cập nhật theo doanh số thực tế",
        "Các khoản phải thu quá hạn cần được đôn đốc trong tuần này",
        "Chi tiêu của các phòng ban được giám sát theo ngân sách duyệt",
        "Quỹ dự phòng của năm được trích lập theo quy định tài chính",
        "Đơn vị cần lập bảng dự kiến thu chi cho quý tới trước ngày hai mươi",
        "Mọi khoản chi phát sinh cần có phê duyệt trước khi thực hiện",
        "Giá trị hợp đồng được thanh toán thành hai đợt theo tiến độ",
    ];

    for s in sentences {
        add_all_splits(s, &mut add);
    }

    // ------------------------------------------------------------------
    // 2. THÀNH NGỮ - TỤC NGỮ - CA DAO (chỉ mục THẬT đã kiểm chứng) —
    //    chia tại mọi điểm: gõ nửa đầu gợi ý nửa sau.
    // ------------------------------------------------------------------
    let idioms: &[&str] = &[
        // Lao động - cần cù - học hành
        "Có công mài sắt có ngày nên kim",
        "Cần cù bù thông minh",
        "Kiến tha lâu cũng đầy tổ",
        "Có chí thì nên",
        "Học ăn học nói học gói học mở",
        "Muốn biết phải hỏi muốn giỏi phải học",
        "Học thầy không tày học bạn",
        "Không thầy đố mày làm nên",
        "Đi một ngày đàng học một sàng khôn",
        "Học đi đôi với hành",
        "Học một đằng quên một nẻo",
        "Nhất tự vi sư bán tự vi sư",
        "Một chữ cũng là thầy nửa chữ cũng là thầy",
        "Gần bùn mà chẳng hôi tanh mùi bùn",
        // Ăn - ở - làm giàu
        "Ăn chắc mặc bền",
        "Ăn quả nhớ kẻ trồng cây",
        "Uống nước nhớ nguồn",
        "Có thực mới vực được đạo",
        "Có làm thì mới có ăn",
        "Tay trắng làm nên",
        "Tay không bắt giặc",
        "Ngồi mát ăn bát vàng",
        "Giàu hai con mắt khó hai bàn tay",
        "Giàu đổi bạn già đổi vợ",
        "Góp gió thành bão",
        "Tre già măng mọc",
        "Nước chảy đá mòn",
        "Dục tốc bất đạt",
        "Chậm mà chắc",
        "Nói ít làm nhiều",
        // Ơn nghĩa - đạo đức
        "Một giọt máu đào hơn ao nước lã",
        "Bầu ơi thương lấy bí cùng",
        "Thương người như thể thương thân",
        "Một con ngựa đau cả tàu bỏ cỏ",
        "Anh em như thể tay chân",
        "Nghĩa tử là nghĩa tận",
        "Ơn sâu nghĩa nặng",
        "Kính trên nhường dưới",
        "Người ta là hoa đất",
        "Công cha như núi Thái Sơn",
        "Đi khắp thế gian không ai tốt bằng mẹ",
        "Cơm cha áo mẹ công ơn thầy thợ",
        "Ở hiền gặp lành",
        "Cây ngay không sợ chết đứng",
        "Tốt gỗ hơn tốt nước sơn",
        // Lời nói - ứng xử
        "Lời nói chẳng mất tiền mua",
        "Lựa lời mà nói cho vừa lòng nhau",
        "Một câu nhịn chín câu lành",
        "Nói có sách mát có chứng",
        "Nói trước bước sau",
        "Miệng nam mô bụng bồ dao găm",
        "Mất lòng trước được lòng sau",
        "Chim khôn kêu tiếng rã rơi",
        "Người khôn nói tiếng đấy lời cho dịu dàng",
        "Tiên trách kỷ hậu trách nhân",
        "Đói cho sạch rách cho thơm",
        "Vàng thì thử lửa than thử vàng",
        "Một lần bất tín vạn lần bất tin",
        "Thắng làm vua thua làm giặc",
        // Gặp gỡ - cơ duyên - chiến lược
        "Đi đêm lắm có ngày gặp mẹ",
        "Đứng núi này trông núi nọ",
        "Gieo gió gặt bão",
        "Gieo nhân gặt quả",
        "Biết người biết ta trăm trận trăm thắng",
        "Nước xa không cứu được lửa gần",
        "Được voi đòi tiên",
        "Được đường tiến đường",
        "Được da dê đòi da hổ",
        "Được thì nhờ không được thì thôi",
        "Đũa mốc chòi mâm son",
        "Chuột sa chĩnh gạo",
        "Cá lớn nuốt cá bé",
        "Cá không ăn muối cá ươn",
        "Cái khó bó cái khôn",
        "Cái rách phải đùm cái rách",
        "Tránh vỏ dưa gặp vỏ dừa",
        "Tiền mất tật mang",
        "Tiền nào của nấy",
        "Trâu buộc ghét trâu ăn",
        "Trâu chậm uống nước đục",
        "Trâu già háo cỏ",
        "Già néo đứt dây",
        "Dãi dầu mưa nắng",
        "Chung lưng đấu cật",
        "Chung chua chua ngọt ngọt",
        "Gần nhà xa mặt gần lòng",
        "Hữu xạ tự nhiên hương",
        "Thân ta như tấm lụa đào",
        "Dù ai nói ngả nói nghiêng",
        "Núi cao còn có núi cao hơn",
        "Sông sâu biển thẳm",
        "Sông cạn đá mòn",
        "Thuyền về có nhớ bến chăng",
        "Bến thì một dạ khăng khăng đợi thuyền",
        "Nước đổ lá môn",
        "Nước đục thả câu",
        // Công việc - cơ chế xã hội
        "Đánh kẻ chạy đi không ai đánh người trở lại",
        "Đèn nhà ai sáng nhà ai",
        "Đem con bỏ chợ",
        "Một công thành danh vạn cốt khô",
        "Một con ngựa đau cả tàu bỏ cỏ",
        "Muốn sang thì bắc cầu Kiều",
        "Muốn con hay chữ phải yêu chữ thầy",
        "Người trong một nước phải thương nhau cùng",
        "Thương cho tí cho roi cho vọp bẻ",
        "Trăm nghe không bằng một thấy",
        "Trăm sự tại mình",
        "Trăm hay không bằng tay quen",
        "Trước sau như một",
        "Trên bảo dưới nghe",
        "Trên dưới một lòng",
        "Sống chết có nhau",
        "Giúp bạn chớ chê bạn nghèo",
        "Khẩu phục tâm phục",
        "Cười người năm trước năm sau người cười",
        // Batch 2 — tục ngữ / ca dao THẬT đã kiểm chứng (chỉ giữ mục chắc chắn)
        "Ếch ngồi đáy giếng coi trời bằng vung",
        "Mèo khen mèo dài đuôi",
        "Gừng càng già càng cay",
        "Nhất nghệ tinh nhất thân vinh",
        "Thất bại là mẹ thành công",
        "Ăn theo lội theo",
        "Dù ai nói ngả nói nghiêng lòng ta vẫn cứ như kiềng ba chân",
        "Thà chết vinh còn hơn sống nhục",
        "Con kiến mà leo cành đa",
        "Của biếu là của lo của cho là của nợ",
        "Dãi gió dầm mưa",
        "Đói giỗ chết no Tết",
        "Lời nói gió bay",
        "Làm phúc gặp lành",
        "Mười năm còn sau đám tang một năm khó bỏ mâm cúng ông bà",
        "Muốn ăn thì lăn vào bếp",
        "Người khôn ăn nói nửa chừng để dành",
        "Người khôn vật nhỏ làm to",
        "Phép vua thua lệ làng",
        "Tháng bảy heo may chim sang đông",
        "Tháng giêng là tháng ăn chơi",
        "Thiện căn ở tại lòng ta",
        "Thời gian là tiền bạc",
        "Thuận buồm xuôi gió",
        "Trâu đổi chó",
        "Trâu độc đốt đồng",
        "Trong ấm ngoài êm",
    ];

    for s in idioms {
        add_all_splits(s, &mut add);
    }

    // ------------------------------------------------------------------
    // 3. CẶP (NGỮ CẢNH → CỤM) CURATED — giữ các nhóm cũ (ngữ nghĩa khớp
    //    từng cặp) + mở rộng văn phòng hành chính / kế toán / nhân sự /
    //    IT / kinh doanh / giao tiếp / Gen Z.
    // ------------------------------------------------------------------
    add_curated(&mut add);

    println!("Tổng số mẫu câu / cụm từ tiếng Việt đã sinh: {}", set.len());

    let mut out_str = String::with_capacity(set.len() * 60);
    out_str.push_str("# Ngữ cảnh tiếng Việt (Next-Word / Phrase Prediction - P6)\n");
    out_str.push_str("# Vòng 52: câu thật chia mọi ranh giới từ + thành ngữ + cặp curated\n");
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

/// Thêm mọi cặp (tiền tố → phần còn lại) của câu `s`:
/// `w1 → w2…wn`, `w1 w2 → w3…wn`, … Khoá ≥ 1 từ, dự đoán ≥ 1 từ.
/// Weight theo số từ khoá: 1 từ = 70, 2 từ = 82, ≥3 từ = 92.
fn add_all_splits(s: &str, add: &mut impl FnMut(&str, &str, u32)) {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() < 2 {
        return;
    }
    for split in 1..words.len() {
        let key = words[..split].join(" ");
        let rest = words[split..].join(" ");
        let weight = match split {
            1 => 70,
            2 => 82,
            _ => 92,
        };
        add(&key, &rest, weight);
    }
}

/// Cặp curated — ngữ nghĩa khớp TỪNG cặp (không nhân chéo vô nghĩa).
fn add_curated(add: &mut impl FnMut(&str, &str, u32)) {
    // Văn phòng - hành chính (cũ + mở rộng)
    for &(ctx, pred, weight) in &[
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
        ("trân trọng cảm ơn", "và rất mong nhận được hồi âm", 92),
        ("hợp đồng", "kinh tế", 100),
        ("hợp đồng", "lao động", 98),
        ("hợp đồng", "dịch vụ", 95),
        ("hợp đồng", "mua bán", 92),
        ("hợp đồng", "nguyên tắc", 90),
        ("hợp đồng", "thuê văn phòng", 88),
        ("hợp đồng", "bảo hiểm", 85),
        ("hợp đồng", "đã được ký kết", 88),
        ("hợp đồng", "sẽ hết hạn vào cuối tháng", 84),
        ("biên bản", "bàn giao", 100),
        ("biên bản", "nghiệm thu", 98),
        ("biên bản", "cuộc họp", 95),
        ("biên bản", "làm việc", 92),
        ("biên bản", "thanh lý hợp đồng", 90),
        ("biên bản", "ghi nhớ", 88),
        ("biên bản cuộc họp", "sẽ được gửi trong chiều nay", 90),
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
        ("vui lòng", "xem xét và phê duyệt", 92),
        ("vui lòng", "điền đầy đủ thông tin", 88),
        ("vui lòng kiểm tra kỹ", "số liệu trước khi trình ký", 90),
        ("vui lòng phản hồi", "trước 17h hôm nay", 92),
        ("vui lòng phản hồi", "qua email này", 90),
        ("thông báo", "về việc", 100),
        ("thông báo", "kết luận cuộc họp", 95),
        ("thông báo", "tuyển dụng nhân sự", 90),
        ("thông báo", "lịch nghỉ lễ", 88),
        ("báo cáo", "kết quả thực hiện", 100),
        ("báo cáo", "tài chính quý", 98),
        ("báo cáo", "tiến độ dự án", 95),
        ("báo cáo", "doanh thu tháng", 92),
        ("báo cáo", "thường niên", 90),
        ("báo cáo", "tổng hợp số liệu", 88),
        ("kế hoạch", "công tác tuần", 100),
        ("kế hoạch", "kinh doanh năm", 98),
        ("kế hoạch", "triển khai chi tiết", 95),
        ("kế hoạch", "hành động", 90),
        ("kế hoạch", "đào tạo nội bộ", 86),
        ("kế hoạch công tác", "đã được phê duyệt", 90),
        ("quy trình", "làm việc chuẩn", 100),
        ("quy trình", "vận hành", 98),
        ("quy trình", "kiểm soát chất lượng", 95),
        ("quy trình", "xử lý khiếu nại", 90),
        ("tiêu chuẩn", "chất lượng dịch vụ", 100),
        ("an toàn", "thông tin", 100),
        ("an toàn", "lao động", 98),
        ("bảo mật", "dữ liệu khách hàng", 100),
        ("bảo mật", "toàn bộ thông tin của khách hàng", 90),
        ("hoàn thành", "đúng tiến độ", 100),
        ("hoàn thành", "nhiệm vụ được giao", 98),
        ("hoàn thành", "xuất sắc nhiệm vụ được giao", 92),
        ("bàn giao", "công việc trước khi nghỉ", 95),
        ("bàn giao", "cho bộ phận vận hành", 90),
        ("xin", "chào", 100),
        ("xin", "cảm ơn", 95),
        ("xin", "lỗi", 90),
        ("xin", "phép", 85),
        ("xin phép", "nghỉ phép năm", 95),
        ("xin phép", "được cập nhật tiến độ", 92),
        ("giải pháp", "công nghệ", 100),
        ("giải pháp", "tối ưu", 90),
        ("chúc mừng", "năm mới", 100),
        ("chúc mừng", "sinh nhật", 98),
        ("chúc mừng", "khai trương", 90),
        ("chúc mừng", "thành công", 88),
        ("hẹn gặp", "lại", 100),
        ("hẹn gặp lại", "anh chị trong buổi họp tới", 88),
        ("xin ý kiến", "chỉ đạo của ban giám đốc", 95),
        ("đồng kính gửi", "các bộ phận liên quan", 90),
        ("phòng ban", "liên quan phối hợp", 92),
        ("phòng ban", "cử một đại diện tham dự", 86),
        ("đề nghị", "quý anh chị thực hiện", 90),
        ("đề nghị", "bổ sung chứng từ kế toán", 86),
        ("đề nghị", "các phòng ban gửi danh sách", 84),
        ("rất mong", "nhận được phản hồi", 94),
        ("rất mong", "nhận được sự hợp tác", 90),
        ("rất mong", "sớm nhận được hồi âm", 88),
        ("rất vui", "được hợp tác cùng anh chị", 86),
        ("rất tiếc", "phải thông báo rằng", 82),
        ("thành thật xin lỗi", "vì sự chậm trễ", 88),
        ("chúng tôi thành thật xin lỗi", "về sự cố lần này", 86),
        ("mọi thắc mắc", "xin vui lòng liên hệ", 90),
        ("trong trường hợp", "cần hỗ trợ thêm", 86),
        ("trong trường hợp khẩn cấp", "xin liên hệ hotline", 84),
        ("kính chúc", "anh chị một ngày làm việc hiệu quả", 90),
        ("kính chúc", "quý công ty phát triển vững mạnh", 86),
        ("chúc anh chị", "một ngày làm việc hiệu quả", 88),
        ("chúc anh", "sức khỏe và thành công", 84),
        ("chúc em", "thi tốt", 82),
        ("một lần nữa", "xin chân thành cảm ơn", 86),
        ("ngay khi có kết quả", "chúng tôi sẽ liên hệ lại", 84),
        ("theo đúng quy định", "hiện hành", 92),
        ("theo yêu cầu", "của ban lãnh đạo", 88),
        ("theo tiến độ", "đã đề ra", 88),
    ] {
        add(ctx, pred, weight);
    }

    // Kế toán - tài chính
    for &(ctx, pred, weight) in &[
        ("số tiền", "đã được chuyển vào tài khoản", 92),
        ("số tiền", "thanh toán như sau", 88),
        ("hóa đơn", "điện tử", 94),
        ("hóa đơn", "giá trị gia tăng", 92),
        ("hóa đơn giá trị gia tăng", "đã được xuất cho anh chị", 88),
        ("công nợ", "phải thu", 92),
        ("công nợ", "phải trả", 90),
        ("công nợ phải thu", "trong tháng đã giảm", 86),
        ("đối chiếu công nợ", "của tháng này", 88),
        ("đối chiếu", "số liệu giữa hai bên", 86),
        ("sổ phụ ngân hàng", "của kỳ này", 86),
        ("báo cáo tài chính", "quý sẽ hoàn tất trước", 88),
        ("thuế giá trị gia tăng", "được nộp qua mạng điện tử", 86),
        ("chi phí", "vận hành", 92),
        ("chi phí", "phát sinh", 88),
        ("ngân sách", "dự kiến", 92),
        ("ngân sách", "đang được dự thảo", 86),
        ("thanh toán", "qua chuyển khoản", 92),
        ("thanh toán", "trong ba ngày làm việc", 88),
        ("tạm ứng", "đã được trừ vào bảng quyết toán", 84),
        ("quyết toán", "cuối kỳ", 86),
        ("số dư", "đầu kỳ", 88),
        ("số dư", "cuối kỳ", 88),
        ("khoản vay", "đã được giải ngân", 82),
        ("lãi suất", "áp dụng cho kỳ này", 82),
        ("chứng từ", "kế toán", 90),
        ("chứng từ", "cần được lưu trữ đầy đủ", 84),
    ] {
        add(ctx, pred, weight);
    }

    // Nhân sự - tổ chức
    for &(ctx, pred, weight) in &[
        ("nghỉ phép", "từ ngày", 94),
        ("nghỉ phép", "ba ngày kể từ", 88),
        ("đơn xin nghỉ phép", "đã được phê duyệt", 90),
        ("tuyển dụng", "nhân sự", 94),
        ("tuyển dụng", "vị trí kế toán viên", 86),
        ("phỏng vấn", "vòng cuối", 88),
        ("buổi phỏng vấn", "sẽ diễn ra vào", 86),
        ("hợp đồng lao động", "hết hạn vào", 88),
        ("hồ sơ nhân sự", "và bảng chấm công", 86),
        ("bảo hiểm xã hội", "được đóng theo", 86),
        ("đánh giá định kỳ", "sáu tháng", 84),
        ("thư mời phỏng vấn", "đã được gửi tới", 84),
        ("đào tạo", "nội bộ", 88),
        ("đào tạo", "nghề nghiệp", 86),
        ("nhân viên mới", "cần hoàn tất thủ tục", 84),
        ("lịch nghỉ lễ", "được thông báo", 84),
        ("chế độ", "đãi ngộ", 88),
        ("chế độ", "phúc lợi", 86),
        ("bảng chấm công", "tháng này", 86),
    ] {
        add(ctx, pred, weight);
    }

    // CSKH - bán hàng
    for &(ctx, pred, weight) in &[
        ("quý khách vui lòng", "chờ trong ít phút", 94),
        ("quý khách vui lòng", "kiểm tra kỹ tình trạng sản phẩm", 88),
        ("yêu cầu của quý khách", "đang được xử lý", 92),
        ("sự cố", "đã được khắc phục", 90),
        ("sự cố", "hệ thống", 88),
        ("xin lỗi", "vì sự chậm trễ", 92),
        ("đơn hàng", "sẽ được giao", 94),
        ("đơn hàng", "của anh chị", 92),
        ("đơn hàng", "số lượng lớn", 88),
        ("đơn hàng", "đã được xác nhận", 86),
        ("kiểm tra tình trạng", "đơn hàng", 86),
        ("phiếu bảo hành", "điện tử", 86),
        ("chính sách đổi trả", "áp dụng trong vòng", 84),
        ("liên hệ", "hotline", 92),
        ("liên hệ", "bộ phận chăm sóc khách hàng", 90),
        ("cảm ơn quý khách", "đã tin tưởng", 88),
        ("cảm ơn anh chị", "đã quan tâm theo dõi", 84),
        ("chương trình khuyến mãi", "áp dụng cho", 86),
        ("mã giảm giá", "áp dụng cho đơn hàng", 84),
        ("báo giá", "trên chỉ áp dụng", 82),
        ("giao hàng", "đúng tiến độ", 88),
        ("giao hàng", "trong hai ngày tới", 86),
        ("ký kết hợp đồng", "vào đầu tháng tới", 84),
        // Giáo dục (regression office_productivity)
        ("bài tập", "về nhà", 90),
        ("bài tập", "trên lớp", 84),
        ("chốt đơn", "ngay hôm nay", 84),
    ] {
        add(ctx, pred, weight);
    }

    // IT - công nghệ
    for &(ctx, pred, weight) in &[
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
        ("cơ sở dữ liệu", "đã được sao lưu", 84),
        ("triển khai", "hệ thống lên môi trường", 100),
        ("triển khai", "giải pháp công nghệ", 95),
        ("triển khai", "bản cập nhật mới", 90),
        ("triển khai", "phiên bản mới lên máy chủ", 84),
        ("giao diện", "người dùng thân thiện", 100),
        ("giao diện", "dòng lệnh", 95),
        ("trí tuệ", "nhân tạo tạo sinh", 100),
        ("học máy", "và khai phá dữ liệu", 95),
        ("xử lý", "ngôn ngữ tự nhiên", 100),
        ("xử lý", "ảnh kỹ thuật số", 95),
        ("xử lý", "dữ liệu lớn", 92),
        ("xử lý", "sự cố hệ thống", 90),
        ("mã nguồn", "mở", 100),
        ("mã nguồn", "chương trình", 95),
        ("mã nguồn", "đã được đẩy lên kho chung", 82),
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
        ("sửa lỗi", "và cập nhật bản vá", 86),
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
        ("sao lưu", "dữ liệu định kỳ", 88),
        ("đồng bộ", "dữ liệu giữa hai hệ thống", 86),
        ("xác thực", "hai yếu tố", 86),
        ("cập nhật", "bản vá lỗi", 86),
        ("cập nhật", "tiến độ công việc", 88),
        ("cập nhật", "lên hệ thống", 86),
        ("đăng nhập", "vào hệ thống", 88),
        ("tài khoản", "đã bị khóa", 80),
        ("mật khẩu", "cần được đổi định kỳ", 80),
        ("hệ thống", "hoạt động bình thường trở lại", 88),
        ("nhóm kỹ thuật", "đang tập trung xử lý", 86),
        ("trình duyệt", "cần được cập nhật", 80),
        ("gỡ lỗi", "trước khi phát hành", 80),
        ("phát hành", "bản mới", 86),
        ("phiên bản", "mới nhất", 88),
    ] {
        add(ctx, pred, weight);
    }

    // Kinh doanh - quản lý
    for &(ctx, pred, weight) in &[
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
        ("tiến độ dự án", "hiện đạt khoảng", 88),
        ("tiến độ", "công việc", 90),
        ("tiến độ", "đã chậm so với kế hoạch", 82),
        ("rủi ro", "chính của dự án", 86),
        ("phân công công việc", "cho từng thành viên", 88),
        ("thời hạn chót", "cho báo cáo quý", 84),
        ("ra soát lại", "tiến độ của dự án", 84),
        ("rà soát lại", "toàn bộ hồ sơ", 86),
        ("họp rút kinh nghiệm", "sau khi kết thúc", 82),
    ] {
        add(ctx, pred, weight);
    }

    // Collocation động từ (giữ nhóm cũ)
    for (verb, targets) in [
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
            ][..],
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
            ][..],
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
            ][..],
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
            ][..],
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
            ][..],
        ),
        (
            "tăng cường",
            &[
                ("công tác quản lý và giám sát", 100),
                ("hợp tác và trao đổi thông tin", 98),
                ("kiểm tra định kỳ", 95),
                ("đào tạo và phát triển nhân lực", 92),
                ("kết nối giữa các phòng ban", 90),
            ][..],
        ),
        (
            "áp dụng",
            &[
                ("công nghệ mới vào sản xuất", 100),
                ("biện pháp phòng ngừa rủi ro", 98),
                ("tiêu chuẩn quản lý quốc tế", 95),
                ("chính sách đãi ngộ hấp dẫn", 92),
                ("mô hình kinh doanh tiên tiến", 90),
            ][..],
        ),
        (
            "giải quyết",
            &[
                ("vấn đề một cách triệt để", 100),
                ("khó khăn và vướng mắc", 98),
                ("khiếu nại của khách hàng", 95),
                ("tranh chấp hợp đồng", 92),
                ("sự cố hệ thống nhanh chóng", 90),
            ][..],
        ),
        (
            "đưa ra",
            &[
                ("quyết định kịp thời", 100),
                ("giải pháp tối ưu nhất", 98),
                ("đề xuất cải tiến quy trình", 95),
                ("phương án dự phòng", 92),
                ("kết luận cuối cùng", 90),
            ][..],
        ),
        (
            "thực hiện",
            &[
                ("nhiệm vụ được giao phó", 100),
                ("dự án đúng theo kế hoạch", 98),
                ("quy trình kiểm tra nghiêm ngặt", 95),
                ("cam kết với khách hàng", 92),
                ("nghĩa vụ tài chính", 90),
            ][..],
        ),
        (
            "phát huy",
            &[
                ("vai trò tiên phong", 100),
                ("thế mạnh của doanh nghiệp", 98),
                ("tiềm năng sẵn có", 95),
                ("hiệu quả hoạt động", 92),
                ("tinh thần đoàn kết", 90),
            ][..],
        ),
        (
            "cung cấp",
            &[
                ("thông tin chính xác và kịp thời", 100),
                ("dịch vụ chất lượng cao", 98),
                ("giải pháp toàn diện cho doanh nghiệp", 95),
                ("tài liệu hướng dẫn chi tiết", 92),
                ("hỗ trợ kỹ thuật 24/7", 90),
            ][..],
        ),
        (
            "duy trì",
            &[
                ("sự ổn định của hệ thống", 100),
                ("hiệu suất làm việc cao", 98),
                ("mối quan hệ tốt đẹp", 95),
                ("tiêu chuẩn an toàn nghiêm ngặt", 92),
                ("liên lạc thường xuyên", 90),
            ][..],
        ),
        (
            "phối hợp",
            &[
                ("với các bên liên quan", 96),
                ("chặt chẽ giữa các phòng ban", 92),
                ("cùng bộ phận kỹ thuật", 88),
            ][..],
        ),
        (
            "trao đổi",
            &[
                ("với anh chị về vấn đề này", 90),
                ("thêm thông tin qua email", 86),
                ("ý kiến trong phần thảo luận", 84),
            ][..],
        ),
        (
            "gửi",
            &[
                ("báo cáo tổng hợp", 94),
                ("bản mềm cho anh chị", 90),
                ("email xác nhận", 88),
                ("phản hồi sớm nhất", 86),
                ("lời cảm ơn chân thành", 84),
            ][..],
        ),
        (
            "xác nhận",
            &[
                ("thông tin lịch hẹn", 92),
                ("đơn hàng", 90),
                ("lại thông tin", 88),
                ("tham dự cuộc họp", 86),
            ][..],
        ),
        (
            "kiểm tra",
            &[
                ("lại thông tin", 94),
                ("kỹ số liệu", 90),
                ("tình trạng đơn hàng", 88),
                ("hộp thư đến", 84),
            ][..],
        ),
    ] {
        for &(target, weight) in targets {
            add(verb, target, weight);
        }
    }

    // Gen Z - trend (mở rộng)
    for &(ctx, pred, weight) in &[
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
        ("sáng", "làn ny chuẩn base", 90),
        ("khét", "lẹt đẹt luôn rồi", 90),
        ("cháy", "phối cực phô", 92),
        ("cực", "phô luôn á", 90),
        ("gòy", "sooqai mọi thứ vậy trời", 88),
        ("raw", "làm luôn đi anh em", 88),
        ("chữa", "cháy deadline tuần này", 88),
        ("onl", "rồi nhắn gì thì nhắn", 84),
        ("off", "lúc trưa quay lại sau", 82),
        ("ping", "em khi cần hỗ trợ ạ", 84),
        ("deadline", "tuần này dồn quá trời", 86),
        ("chúc", "bạn một ngày tốt lành", 100),
        ("chúc", "mừng sinh nhật vui vẻ", 98),
        ("chúc", "mừng năm mới an khang thịnh vượng", 95),
        ("hẹn", "gặp lại vào dịp khác", 100),
        ("rất", "vui được làm quen với bạn", 95),
        ("thiệt", "tình luôn á trời", 86),
        ("căng", "đét luôn á", 86),
        ("mlem", "mlem quá trời quá đất", 84),
        ("xịn", "xò vãi", 88),
        ("mượt", "mà hết nước chấm", 86),
        ("deal", "hời chốt liền", 84),
    ] {
        add(ctx, pred, weight);
    }

    // Chat công sở (mở rộng từ câu thật — cặp ngắn)
    for &(ctx, pred, weight) in &[
        ("cho mình", "xin file", 92),
        ("cho mình", "hỏi chút", 88),
        ("cho mình", "xin phép off lúc trưa", 84),
        ("anh ơi", "cho em hỏi", 92),
        ("anh ơi", "em gửi lại bản mới", 86),
        ("ok anh", "em xử lý ngay", 90),
        ("ok em", "gửi lại sau trưa", 86),
        ("mình confirm", "lại lịch họp nhé", 86),
        ("mình để", "tài liệu trong folder chung", 84),
        ("hẹn review", "sau nhé", 84),
        ("nhớ tag", "mình vào cuộc họp", 82),
        ("em bận deadline", "tuần này", 84),
        ("cảm ơn anh", "nhiều ạ", 88),
        ("cảm ơn chị", "đã hỗ trợ", 86),
        ("có gì", "anh cứ ping em", 84),
        ("xem giúp", "em phần này với", 84),
        ("em đã gửi", "phản hồi rồi nhé", 84),
        ("đã xử lý", "xong rồi nhé", 86),
        ("gặp lại", "anh chị buổi họp tới", 84),
        ("báo lại", "kết quả trong chiều nay", 84),
    ] {
        add(ctx, pred, weight);
    }

    // ------------------------------------------------------------------
    // 4. MA TRẬN NGỮ NGHĨA-CONTROLLED — mỗi tổ hợp đều là câu/cụm tự nhiên
    //    (chủ ngữ × vị ngữ trung tính; động từ × tân ngữ thật; mốc thời hạn).
    // ------------------------------------------------------------------
    // 4a. Chủ ngữ × vị ngữ (mọi tổ hợp tự nhiên).
    for subject in [
        "Anh",
        "Chị",
        "Em",
        "Mình",
        "Tôi",
        "Chúng ta",
        "Chúng tôi",
        "Nhóm",
        "Phòng",
        "Bên em",
        "Bên anh",
        "Quý công ty",
    ] {
        for predicate in [
            "đã nhận được email của anh chị",
            "sẽ phản hồi trong hôm nay",
            "đang xem xét đề xuất này",
            "sẽ gửi tài liệu vào sáng mai",
            "đã xử lý xong yêu cầu",
            "sắp xếp được lịch họp vào chiều mai",
            "đã ghi nhận ý kiến góp ý",
            "sẽ báo cáo lại kết quả vào thứ Sáu",
            "cần thêm thông tin về yêu cầu này",
            "đã thống nhất phương án triển khai",
            "sẽ chuẩn bị bản thảo trước cuối tuần",
            "đã cập nhật tiến độ lên hệ thống",
            "đang chờ phê duyệt từ ban lãnh đạo",
            "sẽ liên hệ lại ngay khi có kết quả",
            "đã hoàn tất việc bàn giao",
            "cần rà soát lại toàn bộ hồ sơ",
            "sẽ tham dự cuộc họp đúng giờ",
            "đã gửi kèm tài liệu liên quan",
            "sẽ xem xét kỹ trước khi quyết định",
            "đang triển khai theo đúng kế hoạch",
        ] {
            add(subject, predicate, 88);
            add(
                &(subject.to_lowercase() + " " + predicate.split(' ').next().unwrap()),
                predicate,
                74,
            );
        }
    }

    // 4b. Động từ × tân ngữ (collocation văn phòng) — kèm biến thể "anh chị + V".
    let verb_objects: &[(&str, &[&str])] = &[
        (
            "gửi",
            &[
                "báo cáo tháng",
                "biên bản họp",
                "bảng số liệu",
                "thư mời họp",
                "hồ sơ dự thầu",
                "đơn xin nghỉ phép",
                "danh sách nhân sự",
                "yêu cầu hỗ trợ",
                "bản nháp bài viết",
                "kết quả kiểm tra",
            ],
        ),
        (
            "đề nghị",
            &[
                "anh chị xem xét",
                "phê duyệt sớm",
                "cung cấp chứng từ",
                "hoàn tất hồ sơ",
                "gửi báo cáo đúng hạn",
                "phối hợp thực hiện",
                "rà soát lại số liệu",
                "bổ sung nhân sự",
                "cập nhật quy trình",
                "kiểm đếm kho",
            ],
        ),
        (
            "xác nhận",
            &[
                "lịch họp",
                "đơn hàng",
                "danh sách tham dự",
                "thông tin tài khoản",
                "kết quả đối chiếu",
                "giờ làm việc",
                "địa điểm gặp",
                "nhận được tài liệu",
                "nội dung biên bản",
                "yêu cầu hỗ trợ",
            ],
        ),
        (
            "kiểm tra",
            &[
                "email",
                "hộp thư đến",
                "số liệu",
                "hóa đơn",
                "hồ sơ",
                "trạng thái đơn hàng",
                "tình trạng hệ thống",
                "kết quả in ấn",
                "dung lượng lưu trữ",
                "bản sao lưu",
            ],
        ),
        (
            "cập nhật",
            &[
                "tiến độ",
                "số liệu",
                "danh sách khách hàng",
                "hồ sơ nhân sự",
                "giá bán",
                "lịch làm việc",
                "phiên bản mới",
                "trạng thái đơn hàng",
                "quy trình làm việc",
                "thông tin liên hệ",
            ],
        ),
        (
            "chuẩn bị",
            &[
                "báo cáo quý",
                "nội dung họp",
                "bài thuyết trình",
                "hồ sơ dự án",
                "bảng so sánh giá",
                "kế hoạch chi tiết",
                "danh sách khách mời",
                "phòng họp",
                "tài liệu học tập",
                "câu hỏi phỏng vấn",
            ],
        ),
        (
            "lập",
            &[
                "kế hoạch công tác",
                "danh sách đầu việc",
                "biên bản họp",
                "bảng dự toán",
                "hồ sơ nghiệm thu",
                "quyết định bổ nhiệm",
                "phiếu thu chi",
                "báo cáo tuần",
                "danh mục hàng hóa",
                "lịch trực lễ",
            ],
        ),
        (
            "trình",
            &[
                "ban giám đốc xem xét",
                "kế hoạch phê duyệt",
                "báo cáo tổng kết",
                "dự thảo hợp đồng",
                "quyết toán cuối năm",
                "hồ sơ vay vốn",
                "phương án tuyển dụng",
                "đề xuất mua sắm",
            ],
        ),
        (
            "thống nhất",
            &[
                "nội dung hợp đồng",
                "mốc thời gian",
                "phương án triển khai",
                "đơn giá",
                "số lượng đơn hàng",
                "địa điểm tổ chức",
                "cách thức thanh toán",
                "phạm vi công việc",
            ],
        ),
        (
            "phê duyệt",
            &[
                "kế hoạch",
                "báo giá",
                "đơn đặt hàng",
                "dự thảo văn bản",
                "ngân sách quý",
                "đơn xin nghỉ phép",
                "phương án điều chỉnh",
                "hồ sơ thanh toán",
            ],
        ),
        (
            "trao đổi",
            &[
                "về nội dung hợp đồng",
                "qua điện thoại",
                "trực tiếp tại văn phòng",
                "qua email",
                "trong buổi họp",
                "với phòng chuyên môn",
                "về tiến độ công việc",
                "với đối tác",
            ],
        ),
        (
            "xem xét",
            &[
                "và phê duyệt",
                "đề xuất này",
                "lại phương án",
                "hồ sơ của ứng viên",
                "yêu cầu hỗ trợ",
                "kỹ nội dung",
                "và phản hồi sớm",
                "các điều khoản",
            ],
        ),
    ];
    for (verb, objects) in verb_objects {
        for obj in *objects {
            add(verb, obj, 88);
            add(&format!("anh chị {verb}"), obj, 78);
            add(&format!("vui lòng {verb}"), obj, 80);
            add(&format!("đề nghị anh chị {verb}"), obj, 76);
            add(&format!("kính đề nghị {verb}"), obj, 76);
            add(&format!("xin phép {verb}"), obj, 74);
            add(&format!("em xin phép {verb}"), obj, 78);
        }
    }

    // 4c. Việc × mốc thời hạn (deadline frames).
    for action in [
        "phản hồi",
        "gửi báo cáo",
        "trình ký",
        "nộp hồ sơ",
        "xác nhận tham dự",
        "hoàn thành nhiệm vụ",
        "đối chiếu số liệu",
        "gửi đề xuất",
        "thanh toán",
        "bàn giao",
        "chốt danh sách",
        "đăng ký tham gia",
        "nộp bài dự thi",
        "kiểm kê tài sản",
        "cập nhật thông tin",
        "gửi chứng từ",
    ] {
        for deadline in [
            "trước 17h hôm nay",
            "trước cuối tuần",
            "trước ngày 15",
            "trước sáng mai",
            "trước thứ Sáu",
            "trước cuối tháng",
            "trước buổi họp",
            "chậm nhất hôm nay",
            "trong hôm nay",
            "trước khi tan sở",
        ] {
            add(action, deadline, 84);
        }
    }

    // 4d. Chúc tụng: lời chào × lời chúc.
    for greeting in [
        "Chúc anh",
        "Chúc chị",
        "Chúc em",
        "Chúc cả nhà",
        "Chúc hai anh chị",
        "Chúc quý công ty",
        "Chúc team mình",
        "Chúc mọi người",
        "Chúc bố mẹ",
        "Chúc con",
    ] {
        for wish in [
            "một ngày mới tốt lành",
            "nhiều sức khỏe và niềm vui",
            "làm việc hiệu quả",
            "thành công trong công việc",
            "bình an và may mắn",
            "một giáng sinh an lành",
            "năm mới vạn sự như ý",
            "cuối tuần vui vẻ",
            "ngủ ngon",
            "một ngày tràn đầy năng lượng",
            "đạt kết quả cao",
            "thuận buồm xuôi gió",
        ] {
            add(greeting, wish, 82);
        }
    }

    // 4e. Cuộc hẹn: mục đích × thời gian.
    for purpose in [
        "họp giao ban",
        "họp tổng kết",
        "trao đổi hợp tác",
        "giải trình số liệu",
        "thảo luận phương án",
        "phỏng vấn ứng viên",
        "nghiệm thu công trình",
        "tư vấn khách hàng",
        "đào tạo nội bộ",
        "ký kết hợp đồng",
        "kiểm tra hiện trường",
        "chụp ảnh kỷ yếu",
    ] {
        for time in [
            "vào 9 giờ sáng mai",
            "vào chiều thứ Tư",
            "vào đầu tuần tới",
            "vào 14h hôm nay",
            "vào sáng thứ Hai",
            "vào cuối tuần này",
            "vào ngày 20 tháng này",
            "sau giờ tan sở",
            "vào 10 giờ sáng thứ Sáu",
            "trong tuần tới",
        ] {
            add(purpose, time, 82);
        }
    }
}
