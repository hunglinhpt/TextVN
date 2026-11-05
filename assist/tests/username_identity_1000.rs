// SPDX-License-Identifier: GPL-3.0-or-later
//! username_identity_1000 — bộ 1000 kịch bản DETERMINISTIC (không RNG, sinh
//! thủ tục theo chỉ số i/j) kiểm chứng yêu cầu chủ repo vòng 55: **user văn
//! phòng gõ username/id dạng `truong.giap`, `tung.vu`, `linh84`, `ngoc.pt`,
//! `truong.giap@congty.vn`… thì engine TextVN (Telex mặc định) phải giữ nguyên
//! VẸN chuỗi khi gõ từng phím** — không dấu, không biến đổi âm tiết, không hoàn
//! tất từ (VN/EN space-complete), không ghost-apply, không tự viết hoa.
//!
//! Kiến trúc: dùng `textvn_core::Engine` qua phím thật (KeyEvent như
//! `core/tests/text_prediction.rs` — copy helper `press_key`/`apply_outcome`/
//! `key_for_char`). Cấu hình test cố tình "khó" cho username: bật
//! `suggest_vn_complete` + `suggest_space_complete` + `auto_restore_english`
//! và `style_words` có cả `"truong"` (từ cá nhân) — kể cả vậy username vẫn
//! phải nguyên vẹn.
//!
//! Phạm vi assert: chỉ hành vi commit THẬT (buffer cuối so byte-identical với
//! chuỗi phím đã gõ, gồm cả Space cuối / Enter vài block). KHÔNG assert
//! ghost/peek (chỉ preview, chưa commit).
//!
//! G8: KHÔNG bao giờ in text người dùng ngoài phím test — input toàn ASCII
//! username tự sinh; panic message rút gọn ≤ 60 ký tự (hàm `clip`).
//!
//! Case id STABLE `A0001..A1000` theo vị trí build — repro 1 case bằng id.
//! Nếu case FAIL ⇒ bug thật của engine: ghi NGUYÊN case (không sửa engine
//! trong file test này, không nới lỏng assert cho pass).

use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

// ============================ Cấu hình engine ============================

/// Engine Telex mặc định + bật đủ đường gợi ý/restore — đúng "kiểu user văn
/// phòng" khó nhất cho username. `style_words` có "truong" (P7 từ cá nhân)
/// để chứng minh từ cá nhân không biến username thành từ Việt có dấu.
fn username_engine() -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        auto_restore_english: true,
        style_words: vec![("truong".into(), 5)],
        ..Default::default()
    })
}

// =================== Helper phím thật (copy text_prediction.rs) ===================

fn key_for_char(c: char) -> KeyEvent {
    match c {
        ' ' => KeyEvent {
            vk: vk::SPACE,
            ch: ' ' as u32,
            key_down: true,
            ..Default::default()
        },
        '\t' => KeyEvent {
            vk: vk::TAB,
            ch: '\t' as u32,
            key_down: true,
            ..Default::default()
        },
        '\n' => KeyEvent {
            vk: vk::RETURN,
            ch: '\n' as u32,
            key_down: true,
            ..Default::default()
        },
        _ => KeyEvent::char_down(c),
    }
}

fn press_key(e: &mut Engine, doc: &mut Vec<char>, k: &KeyEvent) -> Action {
    let out = e.key(k);
    apply_outcome(doc, &out.action, k);
    out.action
}

fn apply_outcome(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => match k.vk {
            vk::BACK => {
                buf.pop();
            }
            vk::ESCAPE | vk::LEFT => {}
            _ => {
                // Ctrl+Space là hotkey, khi Pass không tự chèn ' ' vào văn bản
                if !k.is_ctrl_space() {
                    if let Some(c) = k.printable() {
                        if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                            buf.push(c);
                        }
                    }
                }
            }
        },
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            for _ in 0..*delete_count {
                buf.pop();
            }
            buf.extend(insert);
        }
        Action::Commit { insert } => {
            buf.extend(insert);
        }
    }
}

// ============================ Mô hình case ============================

#[derive(Clone, PartialEq, Eq, Debug)]
enum Expect {
    /// Buffer cuối byte-identical với toàn bộ chuỗi phím đã gõ (gồm Space/Enter cuối).
    Exact,
    /// Case câu thật: chỉ assert phần username giữ nguyên (đúng byte, đúng 1 lần,
    /// có ranh giới dấu cách hai bên) — phần tiếng Việt quanh nó không thuộc phạm vi.
    Contains(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Case {
    id: String,   // "A0001".."A1000" — STABLE theo vị trí
    desc: String, // không chứa nội dung phím
    keys: String, // toàn bộ phím gõ theo thứ tự (ASCII thuần)
    expect: Expect,
}

// ============================ Bảng 25 tên ============================

/// Họ/tên tiếng Việt phổ biến (âm không dấu) — nguyên liệu username.
const NAMES: [&str; 25] = [
    "truong", "giap", "tung", "vu", "linh", "hanh", "minh", "quang", "duc", "hoa", "thanh", "long",
    "thuy", "mai", "nam", "hung", "lan", "cuong", "nga", "tuan", "dung", "hien", "oanh", "bich",
    "thao",
];

/// Họ xen kẽ theo chỉ số — cặp (tên, họ) đa dạng, deterministic.
fn ho_of(i: usize, salt: usize) -> &'static str {
    NAMES[(i + salt + 1) % NAMES.len()]
}

/// Số đuôi 0..99 — deterministic theo chỉ số.
fn dd(i: usize, salt: usize) -> String {
    format!("{:02}", (i * 3 + salt) % 100)
}

fn cap(s: &str) -> String {
    let mut cs = s.chars();
    match cs.next() {
        Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
        None => String::new(),
    }
}

// ============================ build_cases ============================

fn push_exact(cases: &mut Vec<Case>, desc: &str, keys: String) {
    let n = cases.len() + 1;
    cases.push(Case {
        id: format!("A{n:04}"),
        desc: desc.to_string(),
        keys,
        expect: Expect::Exact,
    });
}

fn push_contains(cases: &mut Vec<Case>, desc: &str, keys: String, uid: String) {
    let n = cases.len() + 1;
    cases.push(Case {
        id: format!("A{n:04}"),
        desc: desc.to_string(),
        keys,
        expect: Expect::Contains(uid),
    });
}

fn build_cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::with_capacity(1000);

    // ---- M1: 12 dạng cơ bản × 25 tên = 300 (A0001..A0300) ----
    // Dạng: ten.ho | ho.ten | ten_ho | ten-ho | ten+2 số | ten.ho+123 |
    // t.ho | tenho | ho.ten+_vn | email-like | ten.ho.x | hoa-lẫn.
    for (i, &ten) in NAMES.iter().enumerate() {
        for f in 0..12usize {
            let ho = ho_of(i, f);
            let num = dd(i, 7 + f);
            let keys = match f {
                0 => format!("{ten}.{ho} "),
                1 => format!("{ho}.{ten} "),
                2 => format!("{ten}_{ho} "),
                3 => format!("{ten}-{ho} "),
                4 => format!("{ten}{num} "),
                5 => format!("{ten}.{ho}123 "),
                6 => format!("t.{ho} "),
                7 => format!("{ten}{ho} "),
                8 => format!("{ho}.{ten}_vn "),
                9 => format!("{ten}.{ho}@congty.vn "),
                10 => format!("{ten}.{ho}.x "),
                _ => {
                    let ten_c = cap(ten);
                    let ho_c = cap(ho);
                    format!("{ten_c}.{ho_c} ")
                }
            };
            push_exact(&mut cases, &format!("M1 f{f}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 300);

    // ---- M2: số đuôi 0..99 × 4 dạng × 25 tên = 100 (A0301..A0400) ----
    for (i, &ten) in NAMES.iter().enumerate() {
        for g in 0..4usize {
            let ho = ho_of(i, 20 + g);
            let num = dd(i, 31 + g * 3);
            let keys = match g {
                0 => format!("{ten}.{ho}{num} "),
                1 => format!("{ho}.{ten}{num} "),
                2 => format!("{ten}{ho}{num} "),
                _ => format!("{ten}{num} "),
            };
            push_exact(&mut cases, &format!("M2 g{g}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 400);

    // ---- M3: hoa/thường prefix × 4 dạng × 25 tên = 100 (A0401..A0500) ----
    for (i, &ten) in NAMES.iter().enumerate() {
        for h in 0..4usize {
            let ho = ho_of(i, 40 + h);
            let keys = match h {
                0 => format!("{}.{ho} ", cap(ten)),
                1 => format!("{ten}.{} ", cap(ho)),
                2 => format!("{}.{} ", cap(ten), cap(ho)),
                _ => format!("{}-{} ", cap(ten), cap(ho)),
            };
            push_exact(&mut cases, &format!("M3 h{h}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 500);

    // ---- M4: Enter cuối × 4 dạng × 25 tên = 100 (A0501..A0600) ----
    for (i, &ten) in NAMES.iter().enumerate() {
        for e in 0..4usize {
            let ho = ho_of(i, 60 + e);
            let num = dd(i, 91 + e);
            let keys = match e {
                0 => format!("{ten}.{ho}\n"),
                1 => format!("{ten}_{ho}\n"),
                2 => format!("{ten}{num}\n"),
                _ => format!("{ten}.{ho}@congty.vn\n"),
            };
            push_exact(&mut cases, &format!("M4 e{e}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 600);

    // ---- M5: 1-2 dấu chấm × 4 dạng × 25 tên = 100 (A0601..A0700) ----
    for (i, &ten) in NAMES.iter().enumerate() {
        for d in 0..4usize {
            let ho = ho_of(i, 80 + d);
            let num = dd(i, 13 + d);
            let keys = match d {
                0 => format!("{ten}.{ho}.x "),
                1 => format!("{ho}.{ten}.x "),
                2 => format!("{ten}.{ho}.x{num} "),
                _ => format!("t.{ho}.x "),
            };
            push_exact(&mut cases, &format!("M5 d{d}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 700);

    // ---- M6: username trong câu thật (gõ Tiếng Việt Telex xung quanh) ×
    //      2 mẫu câu × 25 tên = 50 (A0701..A0750) ----
    // Chỉ assert phần username giữ nguyên (đúng yêu cầu đề bài); các từ Việt
    // quanh câu do corpus test riêng của engine bảo đảm.
    for (i, &ten) in NAMES.iter().enumerate() {
        for s in 0..2usize {
            let ho = ho_of(i, 10 + s);
            if s == 0 {
                let uid = format!("{ten}.{ho}");
                let keys = format!("Tooi ddang nhapj bangf {uid} vaoof heej thoongs. ");
                push_contains(&mut cases, "M6 s0 cau dn", keys, uid);
            } else {
                let uid = format!("{ho}.{ten}");
                let keys = format!("Lieen heej {uid} qua email. ");
                push_contains(&mut cases, "M6 s1 cau lh", keys, uid);
            }
        }
    }
    debug_assert_eq!(cases.len(), 750);

    // ---- M7: 20 specials username/id văn phòng thật = 20 (A0751..A0770) ----
    const SPECIALS: [&str; 20] = [
        "admin",
        "root",
        "user01",
        "test.guest",
        "svc-account",
        "api.key",
        "dev10",
        "ngoc.pt",
        "van.dung.qc",
        "khanh.nn2",
        "admin01",
        "root123",
        "user.admin",
        "test.user",
        "guest.user",
        "service.account",
        "api.key2",
        "dev.10",
        "ngoc.pt2",
        "khanh.nn02",
    ];
    for (i, sp) in SPECIALS.iter().enumerate() {
        push_exact(&mut cases, &format!("M7 sp{i}"), format!("{sp} "));
    }
    debug_assert_eq!(cases.len(), 770);

    // ---- M8: lưới tổ hợp chỉ số (i, j) × 8 dạng × 25 tên = 200 (A0771..A0970) ----
    for (i, &ten) in NAMES.iter().enumerate() {
        for p in 0..8usize {
            let ho = ho_of(i, 100 + p * 3);
            let num = dd(i, 50 + p * 11);
            let keys = match p {
                0 => format!("{ten}.{ho}{num} "),
                1 => format!("{ten}-{ho}{num} "),
                2 => format!("{ten}_{ho}{num} "),
                3 => format!("{ten}{ho}{num} "),
                4 => format!("t.{ho}{num} "),
                5 => format!("{ten}.{ho}_vn "),
                6 => format!("{ten}.{ho}@congty.vn "),
                _ => format!("{}.{}{num} ", cap(ten), cap(ho)),
            };
            push_exact(&mut cases, &format!("M8 p{p}"), keys);
        }
    }
    debug_assert_eq!(cases.len(), 970);

    // ---- M9a: tên đơn thuần + Space × 25 = 25 (A0971..A0995) ----
    for ten in NAMES.iter() {
        push_exact(&mut cases, "M9a ten don", format!("{ten} "));
    }
    debug_assert_eq!(cases.len(), 995);

    // ---- M9b: 5 literal kinh điển (đúng chuỗi ví dụ trong yêu cầu) = 5
    //      (A0996..A1000) ----
    for (i, sp) in ["t.ho", "ten.ho", "t.ho42", "Ten.Hoa", "truong.giap"]
        .iter()
        .enumerate()
    {
        push_exact(&mut cases, &format!("M9b lit{i}"), format!("{sp} "));
    }
    debug_assert_eq!(cases.len(), 1000);

    cases
}

// ==================== G8: rút gọn + escape an toàn ====================

/// Rút gọn ≤ max ký tự + escape ký tự điều khiển. KHÔNG bao giờ in toàn bộ.
fn clip(s: &str, max: usize) -> String {
    let mut out = String::new();
    for c in s.chars().take(max) {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7F => {
                out.push_str(&format!("\\u{{{:04X}}}", c as u32))
            }
            c => out.push(c),
        }
    }
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

// ============================== Runner ==============================

/// Case FAIL ở đây = bug thật của engine (bind tiếng Việt/hoàn tất từ vào
/// username) — KHÔNG sửa test cho pass; ghi nguyên case id + input + buffer
/// thực tế để sửa engine. Chạy đủ 1000 case (không fail-fast) để lấy trọn
/// danh sách trong 1 lượt.
#[test]
fn a1000_username_giu_nguyen_byte_identical() {
    let cases = build_cases();
    assert_eq!(
        cases.len(),
        1000,
        "ma tran phai du 1000 case, co {}",
        cases.len()
    );

    let mut failures: Vec<String> = Vec::new();
    for c in &cases {
        // Engine MỚI cho từng case — cách ly trạng thái, không ghép chéo case.
        let mut e = username_engine();
        let mut doc: Vec<char> = Vec::new();
        for ch in c.keys.chars() {
            press_key(&mut e, &mut doc, &key_for_char(ch));
        }
        let got: String = doc.iter().collect();
        let ok = match &c.expect {
            Expect::Exact => got == c.keys,
            Expect::Contains(uid) => {
                got.matches(uid).count() == 1 && got.contains(&format!(" {uid} "))
            }
        };
        if !ok {
            failures.push(format!(
                "[{}] {} | go={} | nhan={}",
                c.id,
                c.desc,
                clip(&c.keys, 60),
                clip(&got, 60)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "username identity: {} case FAIL engine (danh sach day du):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Meta-test: builder pure, deterministic, id stable A0001..A1000, phím ASCII.
#[test]
fn a1000_builder_deterministic_va_id_stable() {
    let a = build_cases();
    let b = build_cases();
    assert_eq!(a.len(), 1000);
    assert_eq!(a[0].id, "A0001");
    assert_eq!(a[999].id, "A1000");
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            x.id,
            format!("A{:04}", i + 1),
            "id phai STABLE tai vi tri {i}"
        );
        assert_eq!(x.id, y.id);
        assert_eq!(x.desc, y.desc);
        assert_eq!(x.keys, y.keys, "keys phai deterministic tai {}", x.id);
        assert_eq!(x.expect, y.expect);
    }
    // Toàn bộ phím gõ là ASCII thuần (G8 + đề bài: username không Unicode ngoài
    // ASCII; câu tiếng Việt cũng gõ bằng phím Telex ASCII).
    for c in &a {
        assert!(
            c.keys.is_ascii(),
            "{}: keys phai ASCII thuần, có ký tự ngoài ASCII",
            c.id
        );
    }
}
