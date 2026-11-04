// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask gen-vocab` — sinh dữ liệu từ vựng prediction (thay script
//! Python `scripts/gen_vocab_v52.py` — vòng 53: ngôn ngữ policy chỉ Rust,
//! xtask mượn `textvn-core` (path dep, 0 crate ngoài) để dùng đúng một nguồn
//! sự thật: `is_valid_word` + `normalize_tone` + `strip_diacritics`.
//!
//! 1. `data/vn_syllables.txt` — 3000 âm tiết tần suất cao (giữ nguyên byte/thứ
//!    tự, nguồn hermitdave/FrequencyWords) + âm tiết còn thiếu từ
//!    `data/assist/vi-DauMoi.dic` (lọc qua `is_valid_word`, chuẩn hoá kiểu
//!    dấu CŨ bằng `normalize_tone`), sắp alphabet.
//! 2. `data/en_words10k.txt` — nguồn đã commit (google-10000-english, import
//!    một lần); task chỉ kiểm format, không tái sinh.
//! 3. `data/vn_compounds.txt` — giữ mục cũ + thêm bảng vòng 53 (13 domain),
//!    lowercase hoá mục HOA-đầu (không bao giờ khớp typed lowercase).
//!
//! Idempotent — sinh lại byte-identical khi không đổi bảng.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use textvn_core::transform::tone::normalize_tone;
use textvn_core::transform::DiacriticStyle;
use textvn_core::validate::is_valid_word;

pub fn run() -> Result<(), String> {
    let n_syll = gen_vn_syllables()?;
    let n_comp = gen_vn_compounds()?;
    let n_en = check_en_words10k()?;
    println!("gen-vocab: syllables {n_syll} · compounds {n_comp} · en_words10k {n_en}");
    if n_syll < 6500 || n_comp < 1900 || n_en < 9000 {
        return Err(format!(
            "SỐ LƯỢNG KHÔNG ĐẠT: syll={n_syll} comp={n_comp} en={n_en}"
        ));
    }
    Ok(())
}

/// Chuẩn hoá kiểu dấu về CŨ (`hoà`→`hòa`) — dùng đúng `normalize_tone` của
/// core (quy tắc qu/gi đã có trong `vowel_span`, không phải port lại).
fn to_old_style(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    normalize_tone(&mut chars, DiacriticStyle::Old);
    chars.into_iter().collect()
}

fn gen_vn_syllables() -> Result<usize, String> {
    let freq_raw =
        fs::read_to_string("data/vn_freq3000.txt").map_err(|e| format!("vn_freq3000.txt: {e}"))?;
    let mut header = String::new();
    let mut freq_words: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for line in freq_raw.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            header.push_str(line);
            header.push('\n');
        } else if !t.is_empty() {
            freq_words.push(t.to_string());
            seen.insert(t.to_string());
        }
    }

    let dic_raw =
        fs::read_to_string("data/assist/vi-DauMoi.dic").map_err(|e| format!("dic: {e}"))?;
    let mut dic_added: Vec<String> = Vec::new();
    let mut rejected = 0usize;
    for line in dic_raw.lines() {
        let w = line.trim();
        // HOA/số/từ ghép — không phải âm tiết gõ hoàn tất.
        if w.is_empty()
            || w != w.to_lowercase()
            || w.contains(' ')
            || !w.chars().all(|c| c.is_alphabetic())
        {
            continue;
        }
        // Gate CHÍNH: validator của core — loại loanword/typo ("email",
        // "basoi", "têt", nhóm "qùy"…) một cách có nguồn sự thật duy nhất.
        let old = to_old_style(w);
        // Exact-match dedup (giữ hành vi Python): "hòa"/"hoà" là HAI mục hợp
        // lệ riêng — canon-key khử-dấu từng giết ~3k bổ sung (lỗi đã bắt).
        if seen.contains(&old) {
            continue;
        }
        if !is_valid_word(&old.chars().collect::<Vec<_>>()) {
            rejected += 1;
            continue;
        }
        seen.insert(old.clone());
        dic_added.push(old);
    }
    // Âm tiết trần dic thiếu nhưng compound/phrase data dùng ("ổ cứng", "với ạ").
    for extra in ["ổ", "ở", "ạ", "ả", "ã", "á", "à"] {
        if !seen.contains(extra) {
            seen.insert(extra.to_string());
            dic_added.push(extra.to_string());
        }
    }
    dic_added.sort();

    let mut out = String::new();
    out.push_str("# Âm tiết tiếng Việt — dữ liệu prediction/Tab-completion (vòng 52)\n");
    out.push_str("# 3000 đầu = tần suất cao nhất (hermitdave/FrequencyWords, OPUS 2018,\n");
    out.push_str("#   thứ tự + byte giữ nguyên từ vn_freq3000.txt); phần sau = bổ sung từ\n");
    out.push_str("#   data/assist/vi-DauMoi.dic (đã lọc thường/1 âm tiết/validator, kiểu CŨ).\n");
    out.push_str("# Sinh bởi `cargo xtask gen-vocab` — KHÔNG sửa tay, chạy lại task.\n");
    out.push_str(&header);
    for w in &freq_words {
        out.push_str(w);
        out.push('\n');
    }
    for w in &dic_added {
        out.push_str(w);
        out.push('\n');
    }
    fs::write("data/vn_syllables.txt", out).map_err(|e| e.to_string())?;
    println!(
        "vn_syllables.txt: {} freq + {} dic (validator loại {rejected}) = {}",
        freq_words.len(),
        dic_added.len(),
        freq_words.len() + dic_added.len()
    );
    Ok(freq_words.len() + dic_added.len())
}

/// Bảng bổ sung vòng 53 — 13 domain mới (~1.9k mục). Weight 100 = dùng nhiều
/// nhất trong domain; toàn bộ từ ghép/cụm từ đơn vị THẬT.
const NEW_COMPOUND_GROUPS: &[(&str, &[(&str, u32)])] = &[
    (
        "bất động sản - nhà đất",
        &[
            ("bất động sản", 100),
            ("nhà đất", 94),
            ("căn hộ", 96),
            ("chung cư", 94),
            ("nhà phố", 88),
            ("biệt thự", 86),
            ("đất nền", 84),
            ("lô đất", 84),
            ("khu đất", 82),
            ("mặt bằng", 92),
            ("mặt tiền", 88),
            ("diện tích", 96),
            ("mét vuông", 90),
            ("diện tích sàn", 76),
            ("sàn thương mại", 80),
            ("tầng hầm", 76),
            ("thang máy", 86),
            ("sổ đỏ", 84),
            ("sổ hồng", 82),
            ("hợp đồng mua bán", 90),
            ("hợp đồng đặt cọc", 84),
            ("giấy sang tên", 78),
            ("công chứng chứng thực", 80),
            ("thế chấp ngân hàng", 84),
            ("vay mua nhà", 88),
            ("vay mua xe", 84),
            ("trả góp nhà", 80),
            ("trả trước", 82),
            ("đặt cọc", 88),
            ("cọc trước", 82),
            ("nhận nhà", 84),
            ("bàn giao nhà", 84),
            ("nội thất", 90),
            ("ngoại thất", 80),
            ("trang trí nhà", 78),
            ("sửa chữa nhà", 80),
            ("xây nhà", 84),
            ("xây dựng", 96),
            ("thiết kế nhà", 80),
            ("phòng khách", 86),
            ("phòng ngủ", 86),
            ("phòng bếp", 82),
            ("nhà bếp", 82),
            ("bàn thờ", 76),
            ("sân thượng", 78),
            ("ban công", 80),
            ("khu vực", 94),
            ("khu dân cư", 82),
            ("khu đô thị", 86),
            ("dự án nhà ở", 82),
            ("nhà ở xã hội", 80),
            ("cư dân", 84),
            ("ban quản lý", 84),
            ("phí quản lý", 80),
            ("tiền điện nước", 80),
            ("phí gửi xe", 78),
            ("gửi xe", 80),
            ("giữ xe", 76),
            ("cho thuê nhà", 86),
            ("thuê nhà trọ", 80),
            ("nhà trọ", 78),
            ("ký gửi", 74),
        ],
    ),
    (
        "ngân hàng - bảo hiểm",
        &[
            ("ngân hàng", 100),
            ("tài khoản", 100),
            ("sổ tiết kiệm", 86),
            ("tiết kiệm", 92),
            ("gửi tiền", 88),
            ("rút tiền", 88),
            ("rút vốn", 80),
            ("chuyển tiền", 92),
            ("nhận tiền", 82),
            ("tiền gửi", 84),
            ("tiền vay", 84),
            ("vay tiêu dùng", 82),
            ("vay mua nhà", 88),
            ("trả nợ gốc", 80),
            ("trả lãi", 80),
            ("kỳ hạn gửi", 78),
            ("biên lai", 84),
            ("sao kê", 84),
            ("sao kê tài khoản", 82),
            ("số dư tài khoản", 84),
            ("thẻ ATM", 82),
            ("thẻ tín dụng", 88),
            ("thẻ ghi nợ", 84),
            ("thẻ nội địa", 78),
            ("thẻ quốc tế", 80),
            ("rút tiền mặt", 84),
            ("mã PIN", 80),
            ("mã OTP", 82),
            ("xác thực OTP", 80),
            ("phí chuyển khoản", 84),
            ("phí thường niên", 78),
            ("hạn mức thẻ", 78),
            ("khoản vay", 90),
            ("hồ sơ vay", 82),
            ("giải ngân", 84),
            ("trả góp tháng", 84),
            ("nợ quá hạn", 82),
            ("tra soát", 76),
            ("bảo hiểm", 100),
            ("hợp đồng bảo hiểm", 90),
            ("đơn bảo hiểm", 84),
            ("phí bảo hiểm", 86),
            ("mức bồi thường", 80),
            ("bồi thường", 88),
            ("quy trình bồi thường", 80),
            ("nhân viên tư vấn", 84),
            ("người thụ hưởng", 82),
            ("bảo hiểm nhân thọ", 84),
            ("bảo hiểm sức khỏe", 84),
            ("bảo hiểm xe", 82),
            ("bảo hiểm nhà", 78),
            ("bảo hiểm du lịch", 78),
            ("trường hợp bảo hiểm", 76),
            ("điều khoản loại trừ", 74),
            ("thời hạn bảo hiểm", 80),
            ("gia hạn bảo hiểm", 80),
            ("hồ sơ bồi thường", 82),
            ("nếu xảy ra tổn thất", 74),
            ("tổn thất", 80),
        ],
    ),
    (
        "hải quan - xuất nhập khẩu - logistics",
        &[
            ("xuất khẩu", 92),
            ("nhập khẩu", 92),
            ("xuất nhập khẩu", 88),
            ("hải quan", 96),
            ("tờ khai hải quan", 88),
            ("tờ khai", 90),
            ("thông quan", 86),
            ("hàng hóa", 96),
            ("hàng nhập", 80),
            ("hàng xuất", 80),
            ("hàng cấm", 74),
            ("hàng hóa mẫu", 74),
            ("container", 84),
            ("container lớn", 74),
            ("khai báo hải quan", 80),
            ("thuế xuất khẩu", 82),
            ("thuế nhập khẩu", 84),
            ("mã HS", 80),
            ("mã hàng hóa", 76),
            ("chứng từ xuất nhập khẩu", 82),
            ("hợp đồng ngoại", 78),
            ("thư tín dụng", 80),
            ("thư tín dụng quốc tế", 74),
            ("vận đơn", 84),
            ("vận đơn đường biển", 78),
            ("hối phiếu", 74),
            ("kho bãi", 80),
            ("kho hàng", 88),
            ("xuất kho", 82),
            ("nhập kho", 82),
            ("kiểm đếm", 84),
            ("kiểm kê", 84),
            ("tồn kho", 88),
            ("nhập tồn", 74),
            ("xuất tồn", 72),
            ("thẻ kho", 74),
            ("phiếu kho", 78),
            ("kho vận", 78),
            ("vận tải", 88),
            ("vận chuyển", 94),
            ("đơn vị vận chuyển", 80),
            ("chuyển phát", 84),
            ("giao nhận", 84),
            ("bên giao", 78),
            ("bên nhận", 78),
            ("bên gửi hàng", 76),
            ("bên mua", 78),
            ("bên bán", 78),
            ("bên A", 72),
            ("bên B", 72),
            ("hai bên", 90),
            ("các bên", 92),
            ("bên thứ ba", 78),
            ("đại lý vận chuyển", 74),
            ("cước vận chuyển", 82),
            ("cước phí", 82),
            ("phí lưu kho", 78),
            ("phí handling", 72),
            ("thời gian giao hàng", 84),
            ("lịch giao hàng", 82),
            ("giao trễ", 74),
            ("bưu điện", 78),
            ("bưu cục", 80),
            ("bưu phẩm", 80),
            ("kiện hàng", 84),
            ("trọng lượng", 86),
            ("cân nặng", 82),
            ("quy cách đóng gói", 78),
        ],
    ),
    (
        "sản xuất - nhà máy - kỹ thuật",
        &[
            ("sản xuất", 100),
            ("dây chuyền", 90),
            ("dây chuyền sản xuất", 88),
            ("nhà máy", 94),
            ("xưởng sản xuất", 84),
            ("công nhân", 90),
            ("tổ sản xuất", 82),
            ("ca sản xuất", 80),
            ("lệnh sản xuất", 78),
            ("kế hoạch sản xuất", 86),
            ("năng lực sản xuất", 82),
            ("công suất", 90),
            ("công suất thiết kế", 78),
            ("nguyên vật liệu", 86),
            ("vật tư", 82),
            ("phụ tùng", 80),
            ("linh kiện", 86),
            ("thiết bị", 96),
            ("máy móc", 90),
            ("máy móc thiết bị", 88),
            ("công cụ", 80),
            ("dụng cụ", 80),
            ("bảo trì", 86),
            ("bảo dưỡng", 84),
            ("sửa chữa", 88),
            ("bảo hộ lao động", 82),
            ("an toàn lao động", 86),
            ("vệ sinh an toàn", 80),
            ("tai nạn lao động", 78),
            ("sự cố kỹ thuật", 80),
            ("sự cố máy móc", 76),
            ("dừng máy", 74),
            ("chạy thử", 76),
            ("vận hành thử", 76),
            ("nghiệm thu kỹ thuật", 80),
            ("tiêu chuẩn kỹ thuật", 84),
            ("quy trình kỹ thuật", 84),
            ("hồ sơ kỹ thuật", 82),
            ("bản vẽ kỹ thuật", 80),
            ("sơ đồ", 84),
            ("sơ đồ khối", 74),
            ("thông số kỹ thuật", 82),
            ("công nghệ sản xuất", 82),
            ("chuyển giao công nghệ", 80),
            ("tự động hóa", 88),
            ("robot", 78),
            ("robot công nghiệp", 74),
            ("kiểm soát chất lượng", 88),
            ("khoanh vùng lỗi", 72),
            ("hàng lỗi", 76),
            ("hàng hỏng", 74),
            ("hàng đạt", 76),
            ("tỷ lệ lỗi", 78),
            ("tỷ lệ đạt", 78),
            ("lô hàng", 86),
            ("số lô", 78),
            ("hạn sử dụng", 84),
            ("ngày sản xuất", 80),
            ("xuất xứ", 82),
            ("nhà cung cấp", 92),
        ],
    ),
    (
        "mua sắm - đấu thầu - hợp đồng",
        &[
            ("mua sắm", 92),
            ("mua sắm công", 82),
            ("đấu thầu", 94),
            ("gói thầu", 88),
            ("hồ sơ mời thầu", 84),
            ("hồ sơ dự thầu", 86),
            ("báo giá dự thầu", 80),
            ("thầu phụ", 72),
            ("chủ đầu tư", 86),
            ("nhà thầu", 88),
            ("nhà thầu phụ", 74),
            ("bảo đảm dự thầu", 76),
            ("bảo đảm thực hiện", 78),
            ("đặt trước", 78),
            ("hợp đồng", 100),
            ("ký hợp đồng", 94),
            ("soạn hợp đồng", 80),
            ("rà soát hợp đồng", 78),
            ("điều khoản", 92),
            ("điều khoản hợp đồng", 84),
            ("điều kiện thanh toán", 82),
            ("thanh toán đợt", 76),
            ("đợt thanh toán", 78),
            ("bên giao khoán", 72),
            ("nghiệm thu", 88),
            ("biên bản nghiệm thu", 86),
            ("quyết toán hợp đồng", 78),
            ("thanh lý hợp đồng", 82),
            ("đàm phán", 88),
            ("thương lượng", 78),
            ("đàm phán hợp đồng", 78),
            ("ký kết", 90),
            ("ký kết hợp đồng", 86),
            ("hợp tác kinh doanh", 84),
            ("thỏa thuận hợp tác", 84),
            ("biên bản ghi nhớ", 82),
            ("thư mời hợp tác", 76),
            ("đối tác chiến lược", 84),
            ("quan hệ đối tác", 82),
            ("phạm vi công việc", 84),
            ("thời gian thực hiện", 82),
            ("giá trị hợp đồng", 84),
            ("tổng giá trị", 80),
            ("đơn giá", 88),
            ("báo giá", 94),
            ("yêu cầu báo giá", 82),
            ("so sánh báo giá", 76),
            ("đàm phán giá", 74),
            ("giảm giá", 88),
            ("chiết khấu", 86),
            ("khuyến mãi", 90),
            ("tặng kèm", 74),
            ("quà tặng", 80),
            ("hóa đơn đỏ", 84),
            ("chứng từ thanh toán", 80),
        ],
    ),
    (
        "truyền thông - báo chí - nội dung",
        &[
            ("truyền thông", 96),
            ("báo chí", 84),
            ("thông cáo báo chí", 80),
            ("họp báo", 80),
            ("phát ngôn", 76),
            ("người phát ngôn", 78),
            ("phỏng vấn", 92),
            ("bài viết", 88),
            ("bài báo", 84),
            ("bài phỏng vấn", 76),
            ("bình luận", 86),
            ("tiêu đề", 82),
            ("tóm tắt", 90),
            ("dàn ý", 78),
            ("bản thảo", 84),
            ("soạn thảo", 88),
            ("biên tập", 82),
            ("kiểm duyệt", 80),
            ("xuất bản", 84),
            ("đăng tải", 82),
            ("phát hành", 86),
            ("phát sóng", 80),
            ("lên sóng", 78),
            ("kênh truyền hình", 78),
            ("chương trình", 96),
            ("kịch bản", 84),
            ("quảng cáo", 94),
            ("video quảng cáo", 78),
            ("spot quảng cáo", 70),
            ("chiến dịch", 88),
            ("chiến dịch truyền thông", 82),
            ("kế hoạch truyền thông", 82),
            ("thông điệp", 82),
            ("hình ảnh", 94),
            ("video", 88),
            ("hồ sơ ảnh", 72),
            ("băng rôn", 74),
            ("poster", 76),
            ("tờ rơi", 74),
            ("standee", 72),
            ("cảm nhận", 78),
            ("phản hồi", 100),
            ("góp ý", 88),
            ("đánh giá", 94),
            ("mạng xã hội", 90),
            ("bài đăng", 80),
            ("cộng đồng", 88),
            ("người theo dõi", 74),
            ("lượt xem", 76),
            ("lượt thích", 74),
            ("tương tác", 82),
            ("trend", 78),
            ("hashtag", 72),
            ("livestream", 76),
            ("trực tiếp", 88),
            ("phát trực tiếp", 78),
        ],
    ),
    (
        "nghiên cứu - khoa học - giáo dục đại học",
        &[
            ("nghiên cứu", 100),
            ("nghiên cứu khoa học", 86),
            ("đề tài", 86),
            ("đề tài nghiên cứu", 82),
            ("dự án nghiên cứu", 78),
            ("giảng viên", 88),
            ("nhà nghiên cứu", 78),
            ("trợ lý nghiên cứu", 76),
            ("học giả", 72),
            ("luận văn", 84),
            ("luận án", 82),
            ("tiểu luận", 80),
            ("bài báo khoa học", 80),
            ("tạp chí khoa học", 76),
            ("hội thảo khoa học", 80),
            ("báo cáo khoa học", 78),
            ("thí nghiệm", 82),
            ("phòng thí nghiệm", 84),
            ("thiết bị thí nghiệm", 74),
            ("dữ liệu nghiên cứu", 78),
            ("khảo sát", 92),
            ("phiếu khảo sát", 80),
            ("kết quả khảo sát", 80),
            ("mẫu khảo sát", 74),
            ("đối tượng khảo sát", 76),
            ("phương pháp", 94),
            ("phương pháp nghiên cứu", 80),
            ("cách tiếp cận", 82),
            ("giả thuyết", 76),
            ("kết luận", 92),
            ("khuyến nghị", 84),
            ("tài liệu tham khảo", 80),
            ("trích dẫn", 78),
            ("nguồn tham khảo", 76),
            ("tổng quan", 84),
            ("tổng quan nghiên cứu", 74),
            ("phạm vi nghiên cứu", 78),
            ("mục tiêu nghiên cứu", 78),
            ("kết quả nghiên cứu", 82),
            ("đóng góp khoa học", 72),
            ("đào tạo", 96),
            ("cao học", 82),
            ("tiến sĩ", 84),
            ("học vị", 74),
            ("học bổng", 82),
            ("nghiên cứu sinh", 76),
            ("sinh viên", 94),
            ("học viên", 84),
            ("thực tập", 86),
            ("thực tập sinh", 84),
            ("kỳ thi", 82),
            ("đề thi", 84),
            ("đáp án", 82),
            ("thang điểm", 76),
            ("xếp loại", 84),
            ("tốt nghiệp", 86),
            ("tuyển sinh", 84),
            ("học phí", 86),
            ("học kỳ", 84),
            ("chuyên ngành", 86),
            ("chương trình học", 80),
        ],
    ),
    (
        "xây dựng - bất động sản dự án",
        &[
            ("xây dựng", 100),
            ("công trình", 94),
            ("công trường", 82),
            ("thầu xây dựng", 80),
            ("thi công", 92),
            ("thi công xây dựng", 82),
            ("giám sát thi công", 80),
            ("kỹ sư", 88),
            ("kiến trúc sư", 82),
            ("bản vẽ", 84),
            ("bản vẽ thi công", 78),
            ("bản vẽ thiết kế", 80),
            ("thiết kế", 92),
            ("thiết kế cơ bản", 76),
            ("thiết kế kỹ thuật", 78),
            ("dự toán", 86),
            ("báo giá vật liệu", 76),
            ("vật liệu", 86),
            ("vật liệu xây dựng", 80),
            ("xi măng", 78),
            ("thép", 76),
            ("cốt thép", 72),
            ("bê tông", 80),
            ("bê tông cốt thép", 76),
            ("gạch đá", 72),
            ("móng nhà", 74),
            ("khung nhà", 72),
            ("hoàn thiện", 90),
            ("hoàn công", 74),
            ("nghiệm thu công trình", 82),
            ("bàn giao công trình", 82),
            ("khánh thành", 80),
            ("tiến độ thi công", 82),
            ("nhân công", 80),
            ("thợ xây", 74),
            ("thợ hồ", 70),
            ("an toàn công trường", 76),
            ("biển báo an toàn", 72),
            ("mũ bảo hộ", 74),
            ("khai thác", 78),
            ("vận hành công trình", 76),
            ("bảo trì công trình", 78),
            ("nâng cấp", 88),
            ("cải tạo", 82),
            ("sửa chữa lớn", 76),
            ("bảo hành công trình", 76),
        ],
    ),
    (
        "nông - lâm - ngư nghiệp",
        &[
            ("nông nghiệp", 90),
            ("nông dân", 84),
            ("nông sản", 84),
            ("nông trại", 80),
            ("trang trại", 80),
            ("vườn cây", 74),
            ("ruộng đồng", 74),
            ("cây trồng", 78),
            ("vật nuôi", 78),
            ("giống cây", 74),
            ("giống vật", 72),
            ("vụ mùa", 76),
            ("mùa vụ", 84),
            ("thu hoạch", 84),
            ("gieo trồng", 76),
            ("chăm sóc", 88),
            ("bón phân", 72),
            ("phun thuốc", 72),
            ("sâu bệnh", 74),
            ("dịch hại", 72),
            ("năng suất", 90),
            ("sản lượng", 84),
            ("hóa đơn nông sản", 70),
            ("hợp tác xã", 82),
            ("hộ kinh doanh", 82),
            ("kho lương", 70),
            ("ao cá", 72),
            ("nuôi trồng", 74),
            ("thủy sản", 80),
            ("hải sản", 82),
            ("chế biến", 84),
            ("chế biến thực phẩm", 78),
            ("bảo quản", 82),
            ("vận chuyển nông sản", 74),
            ("xuất khẩu nông sản", 78),
            ("thị trường nông sản", 74),
            ("đầu vào", 84),
            ("đầu ra", 84),
            ("nhà giống", 70),
            ("trại giống", 70),
        ],
    ),
    (
        "du lịch - khách sạn - nhà hàng",
        &[
            ("du lịch", 96),
            ("khách sạn", 94),
            ("nhà hàng", 92),
            ("đặt phòng", 88),
            ("đặt bàn", 82),
            ("đặt tour", 80),
            ("tour du lịch", 82),
            ("chuyến đi", 84),
            ("chuyến bay", 86),
            ("vé máy bay", 90),
            ("vé tàu", 80),
            ("xe khách", 78),
            ("nhận phòng", 84),
            ("trả phòng", 84),
            ("lưu trú", 78),
            ("phòng đơn", 76),
            ("phòng đôi", 76),
            ("phòng gia đình", 74),
            ("hạng vé", 74),
            ("hành lý", 82),
            ("ký gửi hành lý", 76),
            ("xách tay", 74),
            ("quá cảnh", 72),
            ("điểm đến", 82),
            ("điểm tham quan", 78),
            ("tham quan", 88),
            ("du thuyền", 74),
            ("khu nghỉ dưỡng", 80),
            ("nghỉ dưỡng", 84),
            ("kỳ nghỉ", 86),
            ("lịch trình", 82),
            ("hướng dẫn viên", 80),
            ("thẻ boarding", 70),
            ("sân bay", 86),
            ("ga tàu", 76),
            ("bến xe", 76),
            ("lễ tân", 80),
            ("buồng phòng", 74),
            ("dịch vụ phòng", 76),
            ("buffet", 74),
            ("gọi món", 78),
            ("thực đơn", 80),
            ("đặt món", 78),
            ("hóa đơn nhà hàng", 74),
            ("tip phục vụ", 70),
            ("phục vụ", 90),
            ("khách du lịch", 78),
            ("đoàn du lịch", 74),
            ("khách quốc tế", 76),
            ("bảo hiểm du lịch", 76),
        ],
    ),
    (
        "y tế - chăm sóc sức khỏe",
        &[
            ("sức khỏe", 100),
            ("khám chữa bệnh", 80),
            ("khám bệnh", 86),
            ("chữa bệnh", 80),
            ("bệnh viện", 92),
            ("phòng khám", 90),
            ("trung tâm y tế", 80),
            ("bác sĩ", 92),
            ("điều dưỡng", 78),
            ("y tá", 76),
            ("dược sĩ", 76),
            ("kê đơn", 78),
            ("đơn thuốc", 80),
            ("toa thuốc", 76),
            ("viện phí", 76),
            ("viện đường", 70),
            ("nhập viện", 80),
            ("xuất viện", 80),
            ("tái khám", 78),
            ("khám tổng quát", 80),
            ("khám chuyên khoa", 78),
            ("xét nghiệm", 84),
            ("kết quả xét nghiệm", 80),
            ("chẩn đoán", 84),
            ("điều trị", 86),
            ("phác đồ điều trị", 74),
            ("phòng bệnh", 78),
            ("tiêm phòng", 78),
            ("vắc xin", 78),
            ("sổ sức khỏe", 74),
            ("hồ sơ bệnh án", 78),
            ("bệnh án", 76),
            ("theo dõi sức khỏe", 76),
            ("sức khỏe định kỳ", 78),
            ("khám sức khỏe định kỳ", 78),
            ("bảo hiểm y tế", 86),
            ("bảo hiểm sức khỏe", 84),
            ("chi phí y tế", 80),
            ("viện phí bảo hiểm", 72),
            ("cấp cứu", 84),
            ("số điện thoại cấp cứu", 72),
            ("y tế công cộng", 74),
            ("dịch bệnh", 80),
            ("phòng chống dịch", 76),
            ("cách ly", 76),
            ("theo dõi y tế", 72),
            ("sức khỏe tinh thần", 74),
            ("thư giãn", 84),
            ("nghỉ ngơi", 86),
            ("tập thể dục", 82),
            ("dinh dưỡng", 70),
            ("chế độ ăn", 78),
            ("an toàn thực phẩm", 78),
        ],
    ),
    (
        "pháp lý - thủ tục hành chính",
        &[
            ("pháp lý", 90),
            ("pháp luật", 92),
            ("quy định pháp luật", 84),
            ("luật pháp", 84),
            ("văn bản pháp luật", 80),
            ("nghị định", 90),
            ("thông tư", 88),
            ("nghị quyết", 88),
            ("quyết định", 100),
            ("chỉ thị", 78),
            ("quyết định ban hành", 76),
            ("hồ sơ pháp lý", 82),
            ("giấy tờ pháp lý", 78),
            ("thủ tục hành chính", 84),
            ("thủ tục", 92),
            ("đăng ký", 96),
            ("đăng ký kinh doanh", 84),
            ("đăng ký thuế", 80),
            ("đăng ký biến động", 74),
            ("cấp phép", 84),
            ("giấy phép", 92),
            ("giấy phép xây dựng", 78),
            ("giấy phép kinh doanh", 82),
            ("chứng nhận", 90),
            ("chứng nhận đăng ký", 78),
            ("chứng nhận doanh nghiệp", 76),
            ("giấy chứng nhận", 86),
            ("mã số thuế", 84),
            ("mã số doanh nghiệp", 78),
            ("vốn điều lệ", 82),
            ("điều lệ công ty", 80),
            ("thành lập công ty", 80),
            ("thành lập doanh nghiệp", 78),
            ("người đại diện", 84),
            ("theo pháp luật", 84),
            ("công văn", 96),
            ("công văn số", 78),
            ("số hiệu văn bản", 74),
            ("văn bản đến", 74),
            ("văn bản đi", 74),
            ("trình ký", 82),
            ("phê duyệt", 96),
            ("đơn từ", 84),
            ("đơn xin", 82),
            ("tờ trình", 86),
            ("báo cáo", 100),
            ("thẩm quyền", 82),
            ("căn cứ", 92),
            ("hướng dẫn thực hiện", 82),
            ("tổ chức thực hiện", 82),
            ("kiểm tra thực hiện", 78),
            ("tổng kết thực hiện", 74),
        ],
    ),
    (
        "cảm xúc - quan hệ - giao tiếp đời sống",
        &[
            ("tình cảm", 86),
            ("cảm xúc", 84),
            ("cảm giác", 84),
            ("niềm vui", 84),
            ("niềm hy vọng", 72),
            ("niềm tin", 84),
            ("sự quan tâm", 80),
            ("sự chia sẻ", 78),
            ("sự đồng hành", 74),
            ("sự tin tưởng", 78),
            ("sự thấu hiểu", 76),
            ("sự cố gắng", 76),
            ("sự cố vấn", 70),
            ("tinh thần", 92),
            ("thái độ", 88),
            ("thái độ làm việc", 80),
            ("cách cư xử", 76),
            ("lời nói", 86),
            ("hành động", 88),
            ("ý chí", 78),
            ("nỗ lực", 84),
            ("cố gắng", 90),
            ("chăm chỉ", 88),
            ("trách nhiệm", 94),
            ("lòng tốt", 76),
            ("lòng biết ơn", 76),
            ("lòng trắc ẩn", 72),
            ("sự tử tế", 74),
            ("quan hệ", 90),
            ("mối quan hệ", 90),
            ("mối quan hệ tốt đẹp", 78),
            ("quan hệ công việc", 78),
            ("sự hợp tác", 80),
            ("sự tin cậy", 72),
            ("bạn bè", 88),
            ("người thân", 86),
            ("gia đình", 96),
            ("cô chú", 74),
            ("anh chị em", 76),
            ("bạn đồng hành", 72),
            ("người đồng nghiệp", 78),
            ("đồng nghiệp", 88),
            ("cùng nhau", 88),
            ("chung tay", 78),
            ("giúp đỡ nhau", 76),
            ("chia sẻ cùng", 72),
            ("đồng lòng", 76),
            ("hòa đồng", 80),
            ("vui vẻ", 90),
            ("hạnh phúc", 92),
            ("bình an", 82),
            ("an tâm", 82),
            ("yên tâm", 90),
            ("lo lắng", 86),
            ("bận rộn", 84),
            ("mệt mỏi", 84),
            ("căng thẳng", 84),
            ("thư giãn", 86),
            ("xả stress", 72),
            ("sự cân bằng", 76),
            ("cân bằng cuộc sống", 76),
        ],
    ),
    (
        "số liệu - biểu mẫu - quản trị nội bộ",
        &[
            ("biểu mẫu", 92),
            ("mẫu biểu", 80),
            ("bảng biểu", 84),
            ("biểu đồ", 88),
            ("biểu đồ cột", 74),
            ("biểu đồ tròn", 74),
            ("biểu đồ đường", 74),
            ("bảng số liệu", 86),
            ("bảng tổng hợp", 84),
            ("bảng theo dõi", 80),
            ("bảng đánh giá", 78),
            ("bảng phân công", 78),
            ("bảng so sánh", 80),
            ("cột biểu đồ", 70),
            ("trục số", 70),
            ("chỉ tiêu", 90),
            ("chỉ số", 90),
            ("chỉ số đo lường", 76),
            ("chỉ tiêu kế hoạch", 80),
            ("chỉ tiêu năm", 76),
            ("kỳ báo cáo", 82),
            ("kỳ đánh giá", 78),
            ("kỳ kiểm kê", 76),
            ("kỳ họp", 80),
            ("định kỳ", 88),
            ("đột xuất", 76),
            ("thường xuyên", 90),
            ("định mức", 82),
            ("tiêu chí", 88),
            ("tiêu chí đánh giá", 80),
            ("tiêu chí tuyển", 70),
            ("thang điểm", 76),
            ("thang bậc", 72),
            ("mức đóng", 80),
            ("mức chi", 76),
            ("mức thu", 76),
            ("hạn mức", 82),
            ("ngưỡng", 78),
            ("trần số lượng", 70),
            ("danh mục", 92),
            ("danh sách", 100),
            ("danh sách nhân sự", 80),
            ("danh sách khách hàng", 82),
            ("danh sách việc cần làm", 76),
            ("checklist", 78),
            ("bản ghi", 78),
            ("bản ghi chép", 74),
            ("sổ tay", 82),
            ("sổ theo dõi", 76),
            ("nhật ký", 84),
            ("nhật ký công việc", 76),
            ("biên bản", 96),
            ("biên bản làm việc", 82),
            ("biên bản xác nhận", 78),
            ("hồ sơ", 100),
            ("hồ sơ dự án", 80),
            ("hồ sơ hoàn công", 74),
            ("lưu trữ", 88),
            ("kho hồ sơ", 72),
            ("thời hạn lưu trữ", 74),
        ],
    ),
];

fn gen_vn_compounds() -> Result<usize, String> {
    let path = "data/vn_compounds.txt";
    let old_raw = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut old: BTreeMap<String, u32> = BTreeMap::new();
    let mut header = String::new();
    for line in old_raw.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            header.push_str(line);
            header.push('\n');
            continue;
        }
        if t.is_empty() {
            continue;
        }
        let mut parts = t.split('\t');
        let w = parts.next().unwrap_or("").trim();
        let weight: u32 = parts
            .next()
            .and_then(|x| x.trim().parse().ok())
            .unwrap_or(50);
        if w.is_empty() {
            continue;
        }
        // Mục HOA-đầu từ không bao giờ khớp typed lowercase → hạ về thường
        // (audit vòng 52 GHI-NHẬN 5: "RAM", "CV xin việc").
        let w_lower = w.to_lowercase();
        if w_lower != w && !old.contains_key(&w_lower) {
            old.insert(w_lower, weight);
        } else {
            old.entry(w_lower).or_insert(weight);
        }
    }
    let before = old.len();
    for (_group, entries) in NEW_COMPOUND_GROUPS {
        for &(w, weight) in *entries {
            // chỉ chữ cái thường + khoảng trắng (chặn rác — G3); mục HOA-đầu
            // đã được hạ lowercase ở bước đọc file cũ, bảng mới phải chuẩn.
            if w.chars().all(|c| c.is_lowercase() || c == ' ') {
                old.entry(w.to_string()).or_insert(weight);
            }
        }
    }
    let added = old.len() - before;

    let mut out = String::new();
    out.push_str("# Từ ghép tiếng Việt thông dụng — văn phòng/hành chính/kế toán/nhân sự/\n");
    out.push_str("# IT/kinh doanh/giáo dục/y tế/pháp lý/đời sống/thời gian (vòng 52) +\n");
    out.push_str("# 13 domain vòng 53 (bất động sản, ngân hàng-bảo hiểm, logistics, sản xuất,\n");
    out.push_str("# đấu thầu, truyền thông, nghiên cứu, xây dựng, nông nghiệp, du lịch,\n");
    out.push_str("# y tế, thủ tục hành chính, cảm xúc-quan hệ, biểu mẫu-quản trị).\n");
    out.push_str(
        "# Format: từ_ghép<TAB>weight. Sinh bởi `cargo xtask gen-vocab` — KHÔNG sửa tay.\n",
    );
    for (w, weight) in &old {
        out.push_str(w);
        out.push('\t');
        out.push_str(&weight.to_string());
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())?;
    println!("vn_compounds.txt: {} mục (+{added})", old.len());
    Ok(old.len())
}

fn check_en_words10k() -> Result<usize, String> {
    let raw =
        fs::read_to_string("data/en_words10k.txt").map_err(|e| format!("en_words10k: {e}"))?;
    let mut n = 0;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if !t.chars().all(|c| c.is_ascii_lowercase()) || t.len() < 2 {
            return Err(format!("en_words10k.txt: dòng không hợp lệ: {t:?}"));
        }
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vòng 53: mỗi mục bảng mới phải là chữ cái/khoảng trắng, weight 1-100.
    #[test]
    fn bang_moi_dinh_dang_hop_le() {
        for (group, entries) in NEW_COMPOUND_GROUPS {
            for (w, weight) in *entries {
                assert!(
                    w.chars().all(|c| c.is_alphabetic() || c == ' '),
                    "{group}: `{w}` có ký tự lạ"
                );
                assert!(
                    *weight >= 1 && *weight <= 100,
                    "{group}: `{w}` weight {weight}"
                );
                assert!(w.chars().count() >= 4, "{group}: `{w}` quá ngắn");
            }
        }
    }

    /// new_to_old idempotent và không phá qu/gi ("quỳ" giữ nguyên).
    #[test]
    fn to_old_style_an_toan() {
        for (w, want) in [
            ("quỳ", "quỳ"),
            ("quý", "quý"),
            ("hoà", "hòa"),
            ("thuỷ", "thủy"),
            ("khoẻ", "khỏe"),
            ("toàn", "toàn"),
        ] {
            assert_eq!(to_old_style(w), want, "{w}");
        }
    }
}
