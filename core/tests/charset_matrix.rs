// SPDX-License-Identifier: GPL-3.0-or-later
//! Ma trận 16 combo **bảng mã × kiểu gõ** (yêu cầu audit 0.2.33): 4 bảng mã
//! (`unicode dựng sẵn / unicode tổ hợp / TCVN3 / VNI Windows`) × 4 kiểu gõ
//! (`Telex / VNI / VIQR / Simple Telex`) — mỗi combo gõ nguyên danh sách từ
//! thông dụng phải ra đúng từ đó render theo bảng mã đang chọn.
//!
//! Chuỗi phím sinh **từ bảng phím chuẩn của từng kiểu gõ** (`method::keys_generated`),
//! không copy tay, nên test bám theo bảng phím thật của engine. Kỳ vọng là
//! `charset::encode(từ)` — hàm encode được kiểm riêng bằng anchor cứng dưới đây
//! (dãy byte TCVN3/VNI Windows so với bảng mã gốc, ví dụ TCVN3 `òa` = `ß`,
//! VNI Windows `ò` = `ø`); anchor chặn trường hợp encode sai thì test dutoàn.

use textvn_core::method::keys_generated as g;
use textvn_core::transform::charset::{encode, OutputCharset};
use textvn_core::transform::vowel_table::locate;
use textvn_core::{Action, DiacriticStyle, Engine, EngineOptions, KeyEvent, Method};

const WORDS: &str = "
một hai ba bốn năm sáu bảy tám chín mười trăm nghìn triệu
người được không những trong của này cho với các có là và đã sẽ đang rất cũng như khi
thì nhưng vì nên để từ đến ra vào lên xuống đi về làm biết nói thấy muốn phải cần nhiều
ít lớn nhỏ mới cũ tốt xấu đẹp vui buồn yêu thương nhớ quên học sinh giáo viên trường lớp
bài tập sách vở bút bàn ghế nhà cửa phòng bếp ăn uống ngủ thức dậy chơi đọc viết nghe
nhìn hỏi trả lời gọi điện thoại máy tính mạng chữ tiếng việt nam hà nội sài gòn huế đà
nẵng thơ hải quảng ninh nghệ an thanh hoá đồng nai bình dương long tây nguyên miền bắc
trung dân dạy dài dễ dàng dưới dược dịch dụng dự án dòng dừng duyên giữa gì giờ già giúp
giường khuya khuyên quyển quyết quốc quở quý thuở huơ nghiêng ngoằn ngoèo khoảng hoàng
hoạt động chuyển truyện chuyện nguyện tuyệt vời khuếch thuyền xuyên luật tuần xuân hạ
thu đông mưa nắng gió bão trời đất nước lửa cây cối hoa lá quả chim cá mèo chó gà vịt
trâu bò ngựa voi khỉ rồng rắn ếch ốc ổi ớt ưu điểm ước mơ ơn ừ ạ ấy ở ảnh hưởng tưởng
tượng phượng cường sướng khướu rượu bướu hươu cừu mắm muối tiêu đường chuối xoài bưởi
nhãn vải khoai lang sắn ngô lúa gạo cơm phở bún chả nem rán luộc nướng xào hấp kho canh
chua ngọt đắng cay mặn nhạt thơm ngon đói no khát mệt khoẻ ốm đau bệnh viện bác sĩ
thuốc tiêm khám chữa chúc mừng nhật tết đán giáng hạnh phúc khang thịnh vượng sức thành
công cảm xin lỗi chào tạm biệt hẹn gặp lại nghĩa nghĩ ngẫm nhẫn nhịn nhường chịu khó
khăn thuận lợi thuỷ loà gửi gương gần gũi ghét ghi kẻ kể kiến kìa phía phim phố phương
rõ ràng rộng rãi sông suối sớm muộn trưa tối đêm ngày tháng tuổi xe đạp xăng dầu vẫn
nghỉ ngơi ngoài oai oán uyển chuyện khuyết thiếu yếu yến xiếc hiếu kiêu ngạo
doanh hoạch huých huênh hoẵng tuân khuất khuya liệng yểm cướm mướp ướt buồm tuốt
chừng mực vườn kênh ếch xinh thích hành sạch tắm lâm thơm tôm xóm kem đêm tìm đẹp
xếp kịp họp hộp lớp búp đáp cặp tập mét hết ít một hớt bút mứt mát mặt thật ngoắt
khoét ngoẹo quạ quạt quyến quýnh huấn luyện thuyết nguội tuổi xuôi người ngượng
";

/// Bảng mã gốc dùng từ viết theo dấu MỚI; mặc định sản phẩm 0.2.33 là dấu CŨ —
/// chỉ 4 từ nhạy kiểu (`hoá khoẻ thuỷ loà`) cần đổi lại thành chữ kiểu cũ.
fn as_old_style(word: &str) -> &str {
    match word {
        "hoá" => "hóa",
        "khoẻ" => "khỏe",
        "thuỷ" => "thủy",
        "loà" => "lòa",
        other => other,
    }
}

/// (âm cơ bản, phím dấu phụ) của một chữ cái theo kiểu gõ.
/// Thứ tự bảng âm: a ă â e ê i o ô ơ u ư y (vowels.toml).
fn vowel_parts(entry: usize, method: Method) -> (&'static str, String) {
    // Dấu mũ Telex/Simple Telex = gõ lặp chính chữ cái; VNI `6`; VIQR `^`.
    let circ = |base: &'static str| match method {
        Method::Vni => "6".to_string(),
        Method::Viqr => "^".to_string(),
        _ => base.to_string(),
    };
    // Sừng ơ/ư: Telex/Simple Telex `w`; VNI `7`; VIQR `+`.
    let horn = || match method {
        Method::Vni => "7".to_string(),
        Method::Viqr => "+".to_string(),
        _ => "w".to_string(),
    };
    // Breve ă: Telex/Simple Telex `w`; VNI `8`; VIQR `(`.
    let breve = || match method {
        Method::Vni => "8".to_string(),
        Method::Viqr => "(".to_string(),
        _ => "w".to_string(),
    };
    match entry {
        0 => ("a", String::new()),
        1 => ("a", breve()),
        2 => ("a", circ("a")),
        3 => ("e", String::new()),
        4 => ("e", circ("e")),
        5 => ("i", String::new()),
        6 => ("o", String::new()),
        7 => ("o", circ("o")),
        8 => ("o", horn()),
        9 => ("u", String::new()),
        10 => ("u", horn()),
        11 => ("y", String::new()),
        _ => unreachable!("vowel.toml chỉ có 12 ô âm"),
    }
}

fn tone_keys(method: Method) -> &'static [char; 5] {
    match method {
        Method::Telex => &g::telex::TONE_KEYS,
        Method::SimpleTelex => &g::simple_telex::TONE_KEYS,
        Method::Vni => &g::vni::TONE_KEYS,
        Method::Viqr => &g::viqr::TONE_KEYS,
    }
}

/// Chuỗi phím chuẩn: dấu phụ ngay sau nguyên âm, dấu thanh cuối từ.
fn keys_for(word: &str, method: Method) -> String {
    let mut out = String::new();
    let mut tone = 0usize;
    for c in word.chars() {
        if c == 'đ' {
            // Telex/Simple Telex/VIQR `dd` (STROKE_DOUBLE); VNI `d9`.
            out.push_str(if method == Method::Vni { "d9" } else { "dd" });
            continue;
        }
        match locate(c) {
            Some((entry, t)) => {
                if t > 0 {
                    tone = t;
                }
                let (base, mark) = vowel_parts(entry, method);
                out.push_str(base);
                out.push_str(&mark);
            }
            None => out.push(c),
        }
    }
    if tone > 0 {
        out.push(tone_keys(method)[tone - 1]);
    }
    out
}

fn type_word(e: &mut Engine, keys: &str) -> String {
    let mut buf: Vec<char> = Vec::new();
    for c in keys.chars().chain(std::iter::once(' ')) {
        let k = KeyEvent::char_down(c);
        match e.key(&k).action {
            Action::Pass => buf.push(c),
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                let del = delete_count as usize;
                assert!(del <= buf.len(), "`{keys}`: delete_count vượt buffer");
                buf.truncate(buf.len() - del);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
    buf.into_iter().collect()
}

/// Engine đúng như chế độ mặc định sau cài (0.2.33): bật, Telex, dấu CŨ,
/// bảng mã theo tham số — ghim tường minh để test không trôi theo default.
fn engine(method: Method, charset: OutputCharset) -> Engine {
    Engine::new(EngineOptions {
        method,
        output_charset: charset,
        diacritic_style: DiacriticStyle::Old,
        auto_capitalize: false,
        ..Default::default()
    })
}

/// Anchor cứng: encode của vài từ theo từng bảng mã, so với bảng mã gốc
/// (TCVN3 / VNI Windows) và quy tắc tổ hợp Unicode. Chặn trường hợp cả engine
/// lẫn encode cùng lệch một hướng (kỳ vọng ma trận lấy từ `encode`).
#[test]
fn encode_anchors_match_legacy_tables() {
    let enc = |w: &str, cs: OutputCharset| -> String {
        let v: Vec<char> = w.chars().collect();
        encode(&v, cs).into_iter().collect()
    };
    // TCVN3 (ABC): dấu thanh là byte riêng (ò=0xDF, ế=0xD5, ư=0xAD, ơ=0xEA,
    // `đ`=0xAE) so với bảng TCVN3 gốc.
    assert_eq!(enc("hòa", OutputCharset::Tcvn3), "h\u{df}a");
    assert_eq!(enc("được", OutputCharset::Tcvn3), "®\u{ad}îc");
    assert_eq!(enc("tiếng", OutputCharset::Tcvn3), "ti\u{d5}ng");
    assert_eq!(enc("người", OutputCharset::Tcvn3), "ng\u{ad}\u{ea}i");
    assert_eq!(enc("mưa", OutputCharset::Tcvn3), "m\u{ad}a");
    assert_eq!(enc("thủy", OutputCharset::Tcvn3), "th\u{f1}y");
    // VNI Windows: dấu thanh là byte riêng sau nguyên âm (ò=0xF8, á=0xE1,
    // ủ=0xFA, `đ`=0xF1, ư=0xF6, ơ=0xF4) so với bảng VNI gốc.
    assert_eq!(enc("hòa", OutputCharset::VniWindows), "ho\u{f8}a");
    assert_eq!(
        enc("được", OutputCharset::VniWindows),
        "\u{f1}\u{f6}\u{f4}\u{ef}c"
    );
    assert_eq!(enc("tiếng", OutputCharset::VniWindows), "tie\u{e1}ng");
    assert_eq!(
        enc("người", OutputCharset::VniWindows),
        "ng\u{f6}\u{f4}\u{f8}i"
    );
    assert_eq!(enc("mưa", OutputCharset::VniWindows), "m\u{f6}a");
    assert_eq!(enc("thủy", OutputCharset::VniWindows), "thu\u{fb}y");
    // Unicode tổ hợp: dấu thanh là combining mark U+0300.. theo NFD; `đ` có
    // dạng dựng sẵn riêng (U+0111) nên giữ nguyên.
    assert_eq!(enc("hòa", OutputCharset::UnicodeDecomposed), "ho\u{300}a");
    assert_eq!(
        enc("được", OutputCharset::UnicodeDecomposed),
        "\u{111}\u{1b0}\u{1a1}\u{323}c"
    );
    assert_eq!(enc("mưa", OutputCharset::UnicodeDecomposed), "m\u{1b0}a");
    // Dựng sẵn: passthrough.
    assert_eq!(enc("được", OutputCharset::UnicodePrecomposed), "được");
}

/// Ma trận 4 bảng mã × 4 kiểu gõ = 16 combo gõ đúng toàn bộ danh sách từ.
#[test]
fn charset_times_method_matrix_types_correctly() {
    let charsets = [
        OutputCharset::UnicodePrecomposed,
        OutputCharset::UnicodeDecomposed,
        OutputCharset::Tcvn3,
        OutputCharset::VniWindows,
    ];
    let methods = [
        Method::Telex,
        Method::Vni,
        Method::Viqr,
        Method::SimpleTelex,
    ];
    let mut failures = Vec::new();
    for charset in charsets {
        for method in methods {
            for word in WORDS.split_whitespace() {
                let want_word = as_old_style(word);
                let keys = keys_for(want_word, method);
                let mut e = engine(method, charset);
                let got = type_word(&mut e, &keys);
                let v: Vec<char> = want_word.chars().collect();
                let want = format!("{} ", encode(&v, charset).into_iter().collect::<String>());
                if got != want {
                    failures.push(format!(
                        "{charset:?} × {method:?}: `{keys}` → {got:?} (cần {want:?})"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} combo/từ gõ sai:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
