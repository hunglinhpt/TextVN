// SPDX-License-Identifier: GPL-3.0-or-later
//! Hồi quy trên từ vựng thật: mỗi từ thông dụng → sinh chuỗi phím chuẩn (Telex, VNI, và
//! các biến thể gõ quen tay của người dùng UniKey) → gõ qua `Engine` + Space → phải ra
//! đúng từ đó. Bắt được các lỗi kiểu "không gõ được `dân`" mà test theo quy tắc lẻ bỏ sót.
//!
//! Danh sách phủ đủ phụ âm đầu (b c ch d đ g gh gi h k kh l m n ng ngh nh p ph qu r s t
//! th tr v x), mọi nguyên âm có dấu phụ (ă â ê ô ơ ư), các vần khó (uyê, uya, ươu, oă,
//! oai, uê, iêu, yêu, ưu…) và cả 5 thanh. Kiểu dấu: MỚI (mặc định — `hoá`, `khoẻ`, `thuỷ`).

use textvn_core::transform::vowel_table::locate;
use textvn_core::{Action, Engine, EngineOptions, KeyEvent, Method};

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

/// (âm cơ bản, phím dấu phụ) của một chữ cái — Telex.
fn telex_parts(entry: usize) -> (&'static str, &'static str) {
    // Thứ tự bảng âm: a ă â e ê i o ô ơ u ư y (vowels.toml).
    match entry {
        0 => ("a", ""),
        1 => ("a", "w"),
        2 => ("a", "a"),
        3 => ("e", ""),
        4 => ("e", "e"),
        5 => ("i", ""),
        6 => ("o", ""),
        7 => ("o", "o"),
        8 => ("o", "w"),
        9 => ("u", ""),
        10 => ("u", "w"),
        11 => ("y", ""),
        _ => unreachable!(),
    }
}

fn vni_mark(entry: usize) -> &'static str {
    match entry {
        1 => "8",
        2 | 4 | 7 => "6",
        8 | 10 => "7",
        _ => "",
    }
}

/// Chuỗi phím chuẩn: dấu phụ ngay sau nguyên âm, dấu thanh cuối từ.
fn keys_for(word: &str, method: Method) -> String {
    let mut out = String::new();
    let mut tone = 0usize;
    for c in word.chars() {
        if c == 'đ' {
            out.push_str(if method == Method::Vni { "d9" } else { "dd" });
            continue;
        }
        match locate(c) {
            Some((entry, t)) => {
                if t > 0 {
                    tone = t;
                }
                let (base, telex_mark) = telex_parts(entry);
                out.push_str(base);
                out.push_str(if method == Method::Vni {
                    vni_mark(entry)
                } else {
                    telex_mark
                });
            }
            None => out.push(c),
        }
    }
    if tone > 0 {
        let keys = if method == Method::Vni {
            ["1", "2", "3", "4", "5"]
        } else {
            ["s", "f", "r", "x", "j"]
        };
        out.push_str(keys[tone - 1]);
    }
    out
}

fn type_word(e: &mut Engine, keys: &str) -> String {
    type_word_mods(e, keys, 0)
}

fn type_word_mods(e: &mut Engine, keys: &str, mods: u32) -> String {
    let mut buf: Vec<char> = Vec::new();
    for c in keys.chars().chain(std::iter::once(' ')) {
        let k = KeyEvent {
            mods,
            ..KeyEvent::char_down(c)
        };
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

fn engine(method: Method) -> Engine {
    Engine::new(EngineOptions {
        method,
        auto_capitalize: false,
        ..Default::default()
    })
}

fn check_all(method: Method, variant: impl Fn(&str) -> Option<String>) -> Vec<String> {
    let mut failures = Vec::new();
    for word in WORDS.split_whitespace() {
        let Some(keys) = variant(word) else {
            continue;
        };
        let mut e = engine(method);
        let got = type_word(&mut e, &keys);
        let want = format!("{word} ");
        if got != want {
            failures.push(format!("{keys:>12} → {got:?} (cần {want:?})"));
        }
    }
    failures
}

fn assert_none(label: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{label}: {} từ gõ sai:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn telex_standard_keys() {
    assert_none(
        "Telex",
        check_all(Method::Telex, |w| Some(keys_for(w, Method::Telex))),
    );
}

#[test]
fn vni_standard_keys() {
    assert_none(
        "VNI",
        check_all(Method::Vni, |w| Some(keys_for(w, Method::Vni))),
    );
}

/// Thói quen UniKey: `uow` thay cho `uwow` (`nguowif` → `người`).
#[test]
fn telex_uow_habit() {
    assert_none(
        "Telex uow",
        check_all(Method::Telex, |w| {
            let k = keys_for(w, Method::Telex);
            k.contains("uwow").then(|| k.replace("uwow", "uow"))
        }),
    );
}

/// Dấu thanh gõ ngay sau nguyên âm (trước phụ âm cuối) — gõ tự do.
#[test]
fn telex_tone_right_after_vowel() {
    assert_none(
        "Telex tone-mid",
        check_all(Method::Telex, |w| {
            let k = keys_for(w, Method::Telex);
            let tone = k.chars().last().filter(|c| "sfrxj".contains(*c))?;
            let body = &k[..k.len() - 1];
            // Chèn sau ký tự nguyên âm/dấu phụ cuối cùng (trước phụ âm cuối).
            let pos = body
                .char_indices()
                .rev()
                .find(|(_, c)| "aeiouyw".contains(*c))
                .map(|(i, c)| i + c.len_utf8())?;
            (pos < body.len()).then(|| format!("{}{}{}", &body[..pos], tone, &body[pos..]))
        }),
    );
}

/// Kiểu dấu CŨ: chỉ vần mở `oa/oe/uy` khác (`hóa`, `khỏe`, `thủy`); mọi từ khác giữ nguyên.
#[test]
fn telex_old_style() {
    let old = |w: &str| -> String {
        match w {
            "hoá" => "hóa".into(),
            "khoẻ" => "khỏe".into(),
            "thuỷ" => "thủy".into(),
            "loà" => "lòa".into(),
            other => other.into(),
        }
    };
    let mut failures = Vec::new();
    for word in WORDS.split_whitespace() {
        let keys = keys_for(word, Method::Telex);
        let mut e = Engine::new(EngineOptions {
            method: Method::Telex,
            auto_capitalize: false,
            diacritic_style: textvn_core::DiacriticStyle::Old,
            ..Default::default()
        });
        let got = type_word(&mut e, &keys);
        let want = format!("{} ", old(word));
        if got != want {
            failures.push(format!("{keys:>12} → {got:?} (cần {want:?})"));
        }
    }
    assert_none("Telex kiểu cũ", failures);
}

/// Chữ hoa đầu từ (Shift): `Vieetj` → `Việt`, `DDaf` → `Đà`.
#[test]
fn telex_capitalized_with_shift() {
    let mut failures = Vec::new();
    for word in [
        "Việt", "Nam", "Hà", "Nội", "Đà", "Nẵng", "Huế", "Quảng", "Giang", "Ưng", "Ước",
    ] {
        let keys = keys_for(&word.to_lowercase(), Method::Telex);
        let mut cs = keys.chars();
        let first: String = cs.next().unwrap().to_uppercase().collect();
        let rest: String = cs.collect();
        // `Đ` = Shift+`d` hai lần.
        let keys = if word.starts_with('Đ') {
            format!("{first}{}{}", rest[..1].to_uppercase(), &rest[1..])
        } else {
            format!("{first}{rest}")
        };
        let mut e = engine(Method::Telex);
        let got = type_word(&mut e, &keys);
        if got != format!("{word} ") {
            failures.push(format!("{keys:>12} → {got:?} (cần {word:?})"));
        }
    }
    assert_none("Telex chữ hoa đầu", failures);
}

/// Caps Lock bật: gõ toàn chữ hoa, phím dấu hoa vẫn là phím dấu (`VIEETJ` → `VIỆT`).
/// Tắt Caps Lock, chữ hoa gõ bằng Shift là chữ thường lệ (`USA` giữ nguyên).
#[test]
fn telex_caps_lock_and_shift_acronyms() {
    const MOD_CAPS: u32 = 0x20;
    let mut failures = Vec::new();
    for word in WORDS.split_whitespace() {
        let keys = keys_for(word, Method::Telex).to_uppercase();
        let want = format!("{} ", word.to_uppercase());
        let mut e = engine(Method::Telex);
        let got = type_word_mods(&mut e, &keys, MOD_CAPS);
        if got != want {
            failures.push(format!("{keys:>12} → {got:?} (cần {want:?})"));
        }
    }
    assert_none("Telex Caps Lock", failures);

    const MOD_SHIFT: u32 = 0x1;
    for acronym in ["USA", "JSON", "CSS", "HTML", "DNS", "AWS", "OK", "IDE"] {
        let mut e = engine(Method::Telex);
        assert_eq!(
            type_word_mods(&mut e, acronym, MOD_SHIFT),
            format!("{acronym} "),
            "chữ viết tắt gõ bằng Shift phải giữ nguyên"
        );
    }
}
