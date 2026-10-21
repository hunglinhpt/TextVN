// SPDX-License-Identifier: GPL-3.0-or-later
//! Test ngân sách `suggest` của `Budgeted` + chuyển tiếp của `BridgeDict`
//! (P4-10 §5.3, AC-4 — dùng fake provider, không cần OS nào).
//!
//! Lý do tồn tại bộ bọc (P4-0 D32/D36/D40, `P4-10-OS-PROVIDER-SPEC.md`):
//! provider OS tốn **3,1–6,9 ms/`Suggest`** (đo thật trên Windows 11, finding
//! `P4-10-F3`) — gấp ~50–100 lần `Check` — nên nếu gọi `suggest` cho mọi từ
//! lạ thì một câu dài dễ vượt ngân sách 50 ms/câu. Fetch: giới hạn số lần
//! gọi + đo tổng thời gian, hết ngân sách → gợi ý rỗng (fail-open).

use std::cell::Cell;
use std::thread;
use std::time::Duration;

use textvn_assist::dict::{BridgeDict, Budgeted, Dictionary};

/// Từ điển giả đếm số lần bị gọi (fake provider).
struct CountingDictionary {
    words: Vec<&'static str>,
    contains_calls: Cell<usize>,
    suggest_calls: Cell<usize>,
}

impl CountingDictionary {
    fn new(words: &[&'static str]) -> Self {
        Self {
            words: words.to_vec(),
            contains_calls: Cell::new(0),
            suggest_calls: Cell::new(0),
        }
    }
}

impl Dictionary for CountingDictionary {
    fn contains(&self, word: &str) -> bool {
        self.contains_calls.set(self.contains_calls.get() + 1);
        self.words.contains(&word)
    }

    fn suggest(&self, _word: &str, limit: usize) -> Vec<String> {
        self.suggest_calls.set(self.suggest_calls.get() + 1);
        self.words
            .iter()
            .take(limit)
            .map(|w| (*w).to_string())
            .collect()
    }
}

/// Provider "chậm": mỗi lần `suggest` ngủ một lúc (giả lập 3–7 ms của OS,
/// rút ngắn còn ~2 ms để test nhanh nhưng vẫn đo được bằng `Instant`).
struct SlowDictionary {
    delay: Duration,
    suggest_calls: Cell<usize>,
}

impl Dictionary for SlowDictionary {
    fn contains(&self, _word: &str) -> bool {
        true
    }

    fn suggest(&self, _word: &str, _limit: usize) -> Vec<String> {
        thread::sleep(self.delay);
        self.suggest_calls.set(self.suggest_calls.get() + 1);
        Vec::new()
    }
}

#[test]
fn budget_khong_cho_phep_vuot_max_suggest() {
    let inner = CountingDictionary::new(&["xa", "xện"]);
    let dict = Budgeted::new(&inner, 3);
    assert_eq!(dict.max_suggest(), 3);

    // Gọi 10 lần nhưng chỉ 3 lần đầu đi tới từ điển thật.
    let mut results = Vec::new();
    for _ in 0..10 {
        results.push(dict.suggest("xún", 2));
    }
    assert_eq!(dict.suggest_calls(), 3, "số lần gọi phải bị giới hạn");
    assert_eq!(
        inner.suggest_calls.get(),
        3,
        "từ điển thật cũng chỉ được gọi 3 lần"
    );
    assert!(
        results[..3].iter().all(|r| !r.is_empty()),
        "các lần trong ngân sách có kết quả"
    );
    assert!(
        results[3..].iter().all(|r| r.is_empty()),
        "hết ngân sách → rỗng (fail-open)"
    );
}

#[test]
fn het_ngan_sach_fail_open_khong_panic_khong_chan() {
    let inner = CountingDictionary::new(&["xa"]);
    let dict = Budgeted::new(&inner, 1);
    assert!(!dict.suggest("xún", 1).is_empty());

    let start = std::time::Instant::now();
    for _ in 0..100 {
        assert!(dict.suggest("xún", 1).is_empty());
    }
    // Fail-open: trả rỗng ngay, không gọi vào từ điển thật nữa.
    assert_eq!(dict.suggest_calls(), 1);
    assert_eq!(inner.suggest_calls.get(), 1);
    assert!(
        start.elapsed() < Duration::from_millis(50),
        "100 lần fail-open phải gần như tức thì ({:?})",
        start.elapsed()
    );
}

#[test]
fn elapsed_do_duoc_thoi_gian_dung_cho_suggest() {
    let inner = CountingDictionary::new(&["xa", "xện", "zịa"]);
    let dict = Budgeted::new(&inner, 10);

    assert_eq!(
        dict.elapsed(),
        Duration::ZERO,
        "chưa gọi → chưa tốn thời gian"
    );
    dict.contains("xa"); // contains không tính vào elapsed và ngân sách
    assert_eq!(dict.elapsed(), Duration::ZERO);
    assert_eq!(dict.suggest_calls(), 0);

    for _ in 0..3 {
        dict.suggest("xún", 2);
    }
    assert!(
        dict.elapsed() > Duration::ZERO,
        "sau khi gọi suggest thì thời gian đo được phải > 0"
    );
    assert_eq!(dict.suggest_calls(), 3);
}

#[test]
fn dict_cham_thoi_gian_do_duoc_tang_theo_so_lan_goi() {
    let slow = SlowDictionary {
        delay: Duration::from_millis(2),
        suggest_calls: Cell::new(0),
    };
    let dict = Budgeted::new(&slow, 5);

    dict.suggest("xún", 3);
    let after_first = dict.elapsed();
    assert!(
        after_first >= Duration::from_millis(2),
        "lần gọi đầu phải tốn ≥ 2 ms (sleep tối thiểu): {:?}",
        after_first
    );

    // Thêm 2 lần gọi nữa → tổng thời gian đo được phải tăng thêm ≥ 2×2 ms
    // ⇒ chứng minh `Instant` đo từng lần và cộng dồn đúng.
    dict.suggest("zây", 3);
    dict.suggest("mền", 3);
    let after_third = dict.elapsed();
    assert!(
        after_third >= after_first + Duration::from_millis(4),
        "elapsed phải cộng dồn sau mỗi lần gọi: {} → {}",
        after_first.as_millis(),
        after_third.as_millis()
    );
    assert_eq!(dict.suggest_calls(), 3);
    assert_eq!(slow.suggest_calls.get(), 3);
}

#[test]
fn contains_di_qua_trong_suot_va_khong_ton_ngan_sach() {
    let inner = CountingDictionary::new(&["tôi"]);
    let dict = Budgeted::new(&inner, 0); // ngân sách suggest = 0

    // contains vẫn chạy đúng dù suggest đã bị cắt hoàn toàn.
    assert!(dict.contains("tôi"));
    assert!(!dict.contains("xún"));
    assert_eq!(inner.contains_calls.get(), 2);
    assert_eq!(dict.elapsed(), Duration::ZERO);
    assert!(dict.suggest("xún", 3).is_empty(), "ngân sách 0 → fail-open");
    assert_eq!(dict.suggest_calls(), 0);
    assert_eq!(inner.suggest_calls.get(), 0);
}

#[test]
fn bridge_dict_chuyen_tiep_dung_contains_va_suggest() {
    let inner = CountingDictionary::new(&["tôi", "đi", "học"]);
    let bridge = BridgeDict::new(&inner);

    // contains: đúng từ có, sai từ không.
    assert!(bridge.contains("tôi"), "phải chuyển tiếp kết quả true");
    assert!(!bridge.contains("xún"), "phải chuyển tiếp kết quả false");
    assert_eq!(inner.contains_calls.get(), 2);

    // suggest: chuyển tiếp nguyên mảng (giới hạn `limit` của từ điển thật).
    let direct = inner.suggest("tôi", 2);
    let via_bridge = bridge.suggest("tôi", 2);
    assert!(!direct.is_empty());
    assert_eq!(direct, via_bridge, "BridgeDict phải y hệt gọi thẳng");
    assert_eq!(inner.suggest_calls.get(), 2);

    // into_inner trả lại nguồn gốc.
    let back = BridgeDict::new(inner).into_inner();
    assert_eq!(back.contains_calls.get(), 2);
}

#[test]
fn bridge_dict_voi_no_dictionary_co_tich_cac_duoc() {
    // NoDictionary + BridgeDict + Budgeted phải cắm xếp lớp được (chuỗi
    // wrapper mà `ffi` sẽ dựng khi nối provider OS thật, P4-10 §5.2).
    let inner = BridgeDict::new(textvn_assist::dict::NoDictionary);
    let dict = Budgeted::new(&inner, 2);

    assert!(
        dict.contains("bất cứ từ nào"),
        "NoDictionary coi mọi từ là đúng"
    );
    assert!(
        dict.suggest("xún", 3).is_empty(),
        "NoDictionary không có gợi ý nào"
    );
    assert_eq!(dict.suggest_calls(), 1, "vẫn đếm cả khi từ điển trả rỗng");
}

#[test]
fn bo_voc_dyn_dictionary_truc_tiep_giong_ctx_dict() {
    // Cách luật spelling dùng: bọc `&dyn Dictionary` (D: ?Sized) như bọc
    // `Ctx::dict`, không cần clone hay chuyển kiểu.
    let inner = CountingDictionary::new(&["xa"]);
    let dyn_ref: &dyn Dictionary = &inner;
    let dict = Budgeted::new(dyn_ref, 1);

    assert!(dict.contains("xa"));
    assert!(!dict.suggest("zún", 2).is_empty());
    assert!(dict.suggest("zún", 2).is_empty(), "hết ngân sách (1)");
    assert_eq!(dict.suggest_calls(), 1);
}
