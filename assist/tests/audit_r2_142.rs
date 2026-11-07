// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-142 — hồi quy audit sau 0.2.68: câu ĐÚNG không được bị trợ lý viết lại (FPR=0 ưu
//! tiên hơn bắt thêm lỗi), và các lỗi thật của những luật đó vẫn phải bắt được.

use std::time::{Duration, Instant};
use textvn_assist::check_sentence;

fn fixes(text: &str) -> Vec<String> {
    check_sentence(text)
        .into_iter()
        .flat_map(|i| i.suggestions)
        .collect()
}

#[test]
fn cau_dung_khong_bi_sua() {
    for s in [
        "Either way, we will finish the report and send it.",
        "You can either call or email and we will reply.",
        "Neither of us knows whether he will come or not.",
        "Both teams can win or lose.",
        "Both of us agreed, and we may go today or tomorrow.",
        "He read the report yesterday.",
        "She set the table.",
        "He set up the server last week.",
        "Why doesn't it work?",
        "Didn't she come?",
        "Let it run overnight.",
        "This will make it work.",
        "When you finish it call me.",
        "Please let everyone know.",
        "Make someone feel welcome.",
        "I heard someone call.",
        "Hi everyone have a nice day.",
        "I will send you changes tomorrow.",
        "We sent you offers last week.",
        "He is married with two children.",
        "She is married with a son.",
        "Our focus in Vietnam is growth.",
        "Please focus in on the target.",
        "It was different than I expected.",
        "Place the object against the wall.",
        "Mặc dù khó khăn, chúng ta nên cố gắng.",
        "Dù thế nào, chúng ta nên bình tĩnh.",
        "Tuy bận, anh nên đi khám.",
        "Dù trời mưa, bạn nên mang ô.",
        "Do dịch bệnh, nhiều công ty đóng cửa, nhưng chúng tôi vẫn mở.",
        "Vì lý do sức khoẻ, anh ấy nghỉ việc, nhưng vẫn giữ liên lạc.",
        "Nhờ anh xem giúp, nhưng đừng sửa gì nhé.",
        "Nhờ sự giúp đỡ của anh, dự án hoàn thành, nhưng vẫn còn lỗi.",
    ] {
        let got = fixes(s);
        assert!(got.is_empty(), "sửa oan câu đúng {s:?}: {got:?}");
    }
}

#[test]
fn loi_that_van_bat_duoc() {
    for (s, want) in [
        ("Neither the manager or the staff agreed.", "nor"),
        ("Either tea and coffee is fine.", "or"),
        ("Both John or Mary came.", "and"),
        ("It work well.", "works"),
        ("She go to school.", "goes"),
        ("Everyone have a plan.", "has"),
        ("They is here.", "are"),
        ("You is late.", "are"),
        ("He is married with her.", "to"),
        ("I object against this plan.", "to"),
        ("We focus in quality.", "on"),
        ("This is different than the other one.", "from"),
        ("Mặc dù trời mưa nên tôi ở nhà.", "nhưng"),
        ("Mặc dù trời mưa, nên tôi ở nhà.", "nhưng"),
        ("Vì trời mưa nhưng tôi vẫn đi.", "nên"),
        ("Vì trời mưa to, nhưng tôi vẫn đi.", "nên"),
    ] {
        let got = fixes(s);
        assert!(
            got.iter().any(|f| f == want),
            "bỏ sót lỗi thật {s:?}: {got:?}, cần {want:?}"
        );
    }
}

#[test]
fn lap_tu_song_song() {
    assert!(!fixes("Play the song song again.").is_empty());
    assert!(fixes("Hai đường thẳng song song với nhau.").is_empty());
}

#[test]
fn trong_ngoac_kep_khong_sua_cap_lien_tu() {
    assert!(fixes("He wrote \"neither tea or coffee\" here.").is_empty());
}

/// R2-142 (perf): quét cặp liên từ từng là O(n³) — 2000 từ `either` mất 25 s ở debug.
#[test]
fn cap_lien_tu_tuyen_tinh_tren_cau_dai() {
    let s = "either ".repeat(2000);
    let t0 = Instant::now();
    let _ = fixes(&s);
    assert!(
        t0.elapsed() < Duration::from_secs(5),
        "chậm bất thường: {:?}",
        t0.elapsed()
    );
}
