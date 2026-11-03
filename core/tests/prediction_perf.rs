// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 52 — ngân sách hiệu năng prediction với corpora mở rộng (~31k mục):
//! peek/prefix/next-phrase chạy **mỗi phím** khi ghost bật → phải tuyến tính
//! nhỏ. Chạy `cargo test -p textvn-core --release --test prediction_perf
//! -- --nocapture` để in số đo; cap assert chỉ chặn hồi quy CẤU TRÚC (xem
//! ghi chú ở cap) — số đo chính xác đọc từ println.
//!
//! R2-120b (ci-aur đỏ 2026-11-04): cap 50µs máy-dev đỏ trên runner Arch
//! ("con" 62µs) — cap must nới cho runner variance, không cap sát máy-dev.

use textvn_core::post::predict::{predict_next_phrase_candidates, predict_prefix_candidates};
use textvn_core::post::suggest_vn::complete_vn_word;

fn time_n<F: FnMut()>(mut f: F) -> std::time::Duration {
    // warm-up cache tĩnh
    f();
    let t0 = std::time::Instant::now();
    for _ in 0..1000 {
        f();
    }
    t0.elapsed() / 1000
}

#[test]
fn prediction_perf_ngan_sach_moi_phim() {
    let debug = cfg!(debug_assertions);
    // Hệ số nới: debug chậm hơn release ~30-100× — budget assert chỉ để chặn
    // hồi quy CẤU TRÚC (O(n²) quay lại), số thật đọc ở run --release.
    let k = if debug { 100 } else { 1 };

    let d1 = time_n(|| {
        predict_prefix_candidates("phát", true, 5);
    });
    println!("prefix 'phát' (VI):       {:>10?}", d1);
    let d2 = time_n(|| {
        predict_prefix_candidates("con", true, 5);
    });
    println!("prefix 'con' (VI, ngắn): {:>10?}", d2);
    let d3 = time_n(|| {
        predict_prefix_candidates("the", false, 5);
    });
    println!("prefix 'the' (EN):       {:>10?}", d3);
    let d4 = time_n(|| {
        complete_vn_word("ngư");
    });
    println!("Tab VN 'ngư':            {:>10?}", d4);
    let d5 = time_n(|| {
        predict_next_phrase_candidates("chúng tôi xin trân trọng", 5);
    });
    println!("next-phrase đuôi dài:    {:>10?}", d5);
    let d6 = time_n(|| {
        predict_next_phrase_candidates("cộng hòa", 5);
    });
    println!("next-phrase 'cộng hòa':  {:>10?}", d6);
    let d7 = time_n(|| {
        predict_prefix_candidates("t", false, 5);
    });
    println!("prefix 't' (tệ nhất):    {:>10?}", d7);

    // Cap chặn HỒI QUY CẤU TRÚC (O(n²) quay lại = hàng chục ms), không phải
    // đo chính xác: runner CI (Arch/macOS) chậm hơn máy dev ~1.5× — ci-aur
    // từng đỏ 62µs với cap 50µs (bài học: cap sát máy-dev không dựng được trên
    // runner). Số thật đọc ở in-println phía trên.
    let cap = 500;
    assert!(d1.as_micros() < cap * k, "prefix phát: {d1:?}");
    assert!(d2.as_micros() < cap * k, "prefix con: {d2:?}");
    assert!(d3.as_micros() < cap * k, "prefix the: {d3:?}");
    assert!(d4.as_micros() < cap * k, "tab vn: {d4:?}");
    assert!(d5.as_micros() < cap * k, "phrase dài: {d5:?}");
    assert!(d6.as_micros() < cap * k, "phrase hòa: {d6:?}");
    assert!(d7.as_micros() < cap * k, "prefix 1-chữ: {d7:?}");
}
