// SPDX-License-Identifier: GPL-3.0-or-later
//! panel_idle_1000 — 1000 case P9 (vòng 55): panel trợ lý chỉ hiện khi người
//! dùng **DỪNG gõ ~1 giây** (`PANEL_IDLE_MS = 1000`, `panel_idle.rs`); mọi phím
//! gõ mới **re-schedule** (reset đếm); popup sống bị **ẨN NGAY** khi gõ tiếp;
//! timer fire mới **check + show** (`on_timer` → `check_sentence_on_demand`).
//!
//! Suite mức logic thuần — KHÔNG dựng COM thật cho đường timer (schedule cần
//! hwnd + ITfContext thật, xem phần "Giới hạn coverage" dưới). 6 phần:
//! - **P0** (10): hợp đồng nguồn qua `include_str!` — `PANEL_IDLE_MS == 1000`,
//!   timer id `!= 0x5EC1` (selection) / `!= 0x5EC2` (deferred fix), literal
//!   timer duy nhất trong mã nguồn, overlay WndProc so khớp symbol.
//! - **P1** (290): timing matrix — đồng hồ ảo (virtual clock) mô phỏng ngữ nghĩa
//!   `SetTimer`/`KillTimer` (mỗi schedule reset start; fire ăn pending): burst
//!   n phím, ngưỡng dừng 999/1000 ms, 2 burst cách ≥ 1000 ms.
//! - **P2** (300): hide-on-type matrix — popup sống + Char/Backspace → ẨN NGAY
//!   (edit_session.rs khối "P9 (vòng 55): gõ tiếp khi popup sống"); 300 tổ hợp
//!   (trigger câu × phím kế × độ trễ × số phím).
//! - **P3** (200): schedule-throttle — phím gõ reset fire-time từ phím mới
//!   nhất; phím không-gõ (Enter/Tab/CtrlSpace/Other/Escape) KHÔNG reset.
//! - **P4a** (150): check-on-timer path — 50 câu tiếng Việt (có lỗi / sạch)
//!   qua **TsfShared thật** (FFI engine) → `check_sentence_on_demand` (hàm mà
//!   `panel_idle::on_timer` gọi): Some khi có lỗi, None khi sạch; shape kết
//!   quả; deterministic chạy lại.
//! - **P4b** (50): hợp đồng no-panic `cancel()`/`on_timer()` khi PENDING rỗng
//!   (30) + G8: `clip ≤ 60` ký tự cho mọi message chứa câu người dùng (20).
//!
//! Giới hạn coverage (ghi nhận):
//! - `panel_idle::schedule`/`on_timer` thật cần hwnd + `ITfContext` COM ⇒
//!   đường timer được test bằng mô hình state-machine theo spec P9 + hợp đồng
//!   nguồn (P0). Model thể hiện hợp đồng của `schedule` theo doc-comment nguồn
//!   ("Gọi lại ở MỌI phím gõ để đếm lại") và spec task: phím gõ re-schedule,
//!   phím không-gõ giữ fire-time. Điểm gọi thực tế trong adapter hiện ở trigger
//!   câu (`edit_session.rs` `panel_idle::schedule`) — routing call-site không
//!   thuộc model này.
//! - Popup sống trong model = panel đã hiện qua P9 fire (trigger câu + dừng
//!   1 s + câu còn lỗi). Space/comma là `KeyKind::Char` ⇒ cũng rơi vào khối
//!   ẩn-P9 (603) của nguồn ⇒ net-effect sau phím Char nào cũng ẩn; model theo
//!   net-semantics đó.
//! - `register_timer_hwnd` với hwnd giả không được gọi (KillTimer/IsWindow trên
//!   hwnd rác ngoài phạm vi) ⇒ `cancel()`/`on_timer()` test trên trạng thái
//!   hwnd=0 + PENDING rỗng (đúng nhánh no-op không panic).
//! - Escape/Enter đóng popup ở lớp overlay riêng (ngoài khối P9) — model chỉ
//!   assert luật P9: Char/Backspace ẩn, phím khác không ẩn.
//! - P4a phụ thuộc kill-switch assist của máy (state.json → `IpcClient`).
//!   Nếu máy tắt assist, runner tự chuyển sang hợp đồng "gate tắt ⇒ luôn None".

#![cfg(windows)]

use std::rc::Rc;

use textvn_win_tsf::assist::check_sentence_on_demand;
use textvn_win_tsf::edit_session::TsfShared;
use textvn_win_tsf::panel_idle::{self, PANEL_IDLE_MS};
use textvn_win_tsf::ThreadState;

// ===========================================================================
// Định mức — meta-test đếm đúng 1000
// ===========================================================================

const N_P0: usize = 10; // hợp đồng nguồn
const N_P1: usize = 290; // timing matrix (A 180 + B 60 + C 50)
const N_P2: usize = 300; // hide-on-type (288 + 12 extras)
const N_P3: usize = 200; // schedule-throttle (90 + 60 + 50)
const N_P4A: usize = 150; // 50 câu × 3 (outcome/shape/determinism)
const N_P4B: usize = 50; // 30 hợp đồng no-panic + 20 G8 clip
const N_TOTAL: usize = 1000;

const IDLE_MS: u64 = 1_000; // phải khớp PANEL_IDLE_MS (P0 chốt)

// ===========================================================================
// G8 — clip thông điệp ≤ 60 ký tự (không log nguyên câu người dùng)
// ===========================================================================

/// Cắt theo **ký tự Unicode** (không phải byte) — câu tiếng Việt có dấu không
/// bị cắt giữa chữ; ký tự điều khiển được escape. Dùng cho MỌI assert message
/// chứa nội dung có thể mang text người dùng (G8).
fn clip(s: &str, max: usize) -> String {
    let mut out = String::new();
    for c in s.chars().take(max) {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7F => {
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

// ===========================================================================
// Mô hình — đồng hồ ảo theo ngữ nghĩa SetTimer/KillTimer của panel_idle
// ===========================================================================

/// Phím theo [`textvn_win_tsf::compose::KeyKind`] (đủ biến thể P9 quan tâm).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Key {
    /// Sinh ký tự in được (gồm space, dấu câu, số — kể cả có dấu tiếng Việt).
    Char(char),
    Backspace,
    Escape,
    Enter,
    Tab,
    CtrlSpace,
    /// Không sinh ký tự: mũi tên, Delete, Home/End, F-key…
    Other,
}

/// Phím gõ (mục đích: ẩn popup sống + re-schedule timer).
fn is_typing(key: Key) -> bool {
    matches!(key, Key::Char(_) | Key::Backspace)
}

/// Trigger câu P9 ('.' '!' '?') — điểm gọi `panel_idle::schedule` của adapter
/// (`edit_session.rs`: `after_trigger` → `AssistTrigger::Sentence`).
fn is_sentence_trigger(key: Key) -> bool {
    matches!(key, Key::Char('.' | '!' | '?'))
}

/// Mô hình panel P9 trên MỘT thread:
/// - `schedule(now)`: SetTimer re-arm — fire-time = now + 1000 (reset start);
/// - `tick(now)`: WM_TIMER tới mốc → `on_timer()`: KillTimer + take PENDING →
///   check câu (mô hình hóa bằng cờ `has_error` của case) → show/ẩn;
/// - phím gõ khi popup sống → ẨN NGAY (khối "P9 (vòng 55)" edit_session.rs);
/// - phím không-gõ: không reset, không ẩn.
#[derive(Default)]
struct PanelModel {
    /// None = không có timer pending; Some(t) = fire-time tuyệt đối (ms).
    fire_at: Option<u64>,
    /// Popup **sống** đang hiện (R2-129: panel theo câu, không phải Ctrl+Space).
    visible: bool,
    /// Số lần timer fire (mỗi fire ăn pending — chỉ 1 lần/mỗi schedule).
    fires: u32,
    /// Số lần popup sống bị ẨN vì gõ tiếp (đếm không trùng: popup đã ẩn thì
    /// phím sau không tăng thêm).
    hides: u32,
}

impl PanelModel {
    fn schedule(&mut self, now: u64) {
        self.fire_at = Some(now + IDLE_MS);
    }

    /// `on_timer` khi WM_TIMER tới: KillTimer + take PENDING; check câu xong
    /// show nếu còn lỗi (`has_error`), câu sạch → im lặng (không popup).
    fn tick(&mut self, now: u64, has_error: bool) {
        if let Some(t) = self.fire_at {
            if now >= t {
                self.fire_at = None;
                self.fires += 1;
                self.visible = has_error;
            }
        }
    }

    /// Một phím tại mốc `now` (timer đến hạn fire TRƯỚC phím cùng mốc — thứ tự
    /// hàng đợi message).
    fn press(&mut self, now: u64, key: Key, has_error: bool) {
        self.tick(now, has_error);
        // P9 (vòng 55): gõ tiếp khi popup sống → ẨN NGAY (edit_session.rs:601).
        if self.visible && is_typing(key) {
            self.visible = false;
            self.hides += 1;
        }
        // Phím gõ → re-arm idle 1 s (hợp đồng `schedule`: "Gọi lại ở MỌI phím
        // gõ để đếm lại" — panel_idle.rs). Phím không-gõ: không đụng timer.
        if is_typing(key) {
            self.schedule(now);
        }
    }
}

// ===========================================================================
// P0 — hợp đồng nguồn (10 case)
// ===========================================================================

fn parse_const_usize(src: &str, name: &str) -> Option<usize> {
    let marker = format!("const {name}: usize =");
    src.lines().find(|l| l.contains(&marker)).and_then(|l| {
        let rhs = l.split('=').nth(1)?;
        let hex = rhs.trim().trim_end_matches(';').trim();
        usize::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
    })
}

#[test]
fn p0_source_contract_10_case() {
    let src = include_str!("../src/panel_idle.rs");
    // (1) hằng số pub khớp 1000 ms.
    assert_eq!(PANEL_IDLE_MS, 1_000, "P0-1: PANEL_IDLE_MS phải là 1000");
    // (2) nguồn khai báo literal 1_000 (không ai đổi hằng mà quên spec).
    assert!(
        src.contains("pub const PANEL_IDLE_MS: u32 = 1_000;"),
        "P0-2: khai báo PANEL_IDLE_MS sai trong panel_idle.rs"
    );
    // (3)(4) timer id không đụng 2 timer đã kiểm chứng khác.
    let idle = parse_const_usize(src, "TIMER_PANEL_IDLE")
        .expect("P0-3: không đọc được TIMER_PANEL_IDLE từ nguồn");
    assert_ne!(idle, 0x5EC1, "P0-3: trùng TIMER_SELECTION_CHECK");
    assert_ne!(idle, 0x5EC2, "P0-4: trùng TIMER_DEFERRED_FIX");
    // (5) literal timer DUY NHẤT trong panel_idle.rs (SetTimer/KillTimer dùng
    // symbol — không hardcode chỗ khác).
    let lit = format!("0x{:04X}", idle);
    assert_eq!(
        src.matches(&lit).count(),
        1,
        "P0-5: literal {lit} phải xuất hiện đúng 1 lần"
    );
    // (6)(7) so với 2 timer thật của selection / deferred-fix (parse nguồn).
    let sel = parse_const_usize(
        include_str!("../src/selection_check.rs"),
        "TIMER_SELECTION_CHECK",
    )
    .expect("P0-6: không đọc được TIMER_SELECTION_CHECK");
    let deferred = parse_const_usize(include_str!("../src/edit_session.rs"), "TIMER_DEFERRED_FIX")
        .expect("P0-7: không đọc được TIMER_DEFERRED_FIX");
    assert_eq!(sel, 0x5EC1, "P0-6: TIMER_SELECTION_CHECK phải là 0x5EC1");
    assert_eq!(deferred, 0x5EC2, "P0-7: TIMER_DEFERRED_FIX phải là 0x5EC2");
    assert_ne!(idle, sel, "P0-6: timer idle đụng timer selection");
    assert_ne!(idle, deferred, "P0-7: timer idle đụng timer deferred-fix");
    // (8) overlay WndProc so khớp qua SYMBOL (không hardcode chỗ thứ hai).
    let overlay = include_str!("../src/overlay.rs");
    assert!(
        overlay.contains("panel_idle::TIMER_PANEL_IDLE"),
        "P0-8: overlay WndProc phải so khớp TIMER_PANEL_IDLE qua symbol"
    );
    // (9) WM_TIMER của idle → panel_idle::on_timer().
    assert!(
        overlay.contains("panel_idle::on_timer()"),
        "P0-9: WM_TIMER phải gọi panel_idle::on_timer()"
    );
    // (10) call-site re-arm tồn tại + marker P9 vòng 55 trong edit_session.
    let es = include_str!("../src/edit_session.rs");
    assert!(
        es.contains("panel_idle::schedule("),
        "P0-10: thiếu call-site panel_idle::schedule trong edit_session"
    );
    assert!(
        es.contains("P9 (vòng 55)"),
        "P0-10: thiếu marker 'P9 (vòng 55)' trong edit_session"
    );
}

// ===========================================================================
// P1 — timing matrix (290 case, đồng hồ ảo)
// ===========================================================================

/// A — burst: n phím cách nhau `gap ≤ 999 ms` ⇒ chỉ 1 fire, 1000 ms từ phím
/// CUỐI (mọi phím gõ re-schedule). 6n × 15 gap × 2 phím-cuối = 180.
fn build_p1_a() -> Vec<(String, usize, u64, Key)> {
    let gaps = [
        50u64, 100, 150, 200, 250, 300, 350, 400, 500, 600, 700, 800, 900, 950, 999,
    ];
    let mut cases = Vec::new();
    for n in 1..=6usize {
        for (gi, gap) in gaps.iter().enumerate() {
            for (li, last) in [Key::Char('a'), Key::Backspace].into_iter().enumerate() {
                cases.push((format!("A{n}.{gi}.{li}"), n, *gap, last));
            }
        }
    }
    cases
}

/// B — ngưỡng dừng: gõ 1 phím rồi dừng `idle` ms ⇒ fire ⟺ idle ≥ 1000.
/// 10 idle × 6 phím = 60.
fn build_p1_b() -> Vec<(String, u64, Key)> {
    let idles = [1u64, 250, 500, 750, 998, 999, 1000, 1001, 1500, 2500];
    let keys = [
        ("char_a", Key::Char('a')),
        ("char_space", Key::Char(' ')),
        ("char_dot", Key::Char('.')),
        ("char_comma", Key::Char(',')),
        ("backspace", Key::Backspace),
        ("char_digit", Key::Char('5')),
    ];
    let mut cases = Vec::new();
    for (ii, idle) in idles.iter().enumerate() {
        for (ki, (kn, key)) in keys.iter().enumerate() {
            cases.push((format!("B{ii}.{ki}{kn}"), *idle, *key));
        }
    }
    cases
}

/// C — 2 burst: gap giữa 2 phím gõ < 1000 ⇒ 1 fire; ≥ 1000 ⇒ fire đúng 2 lần
/// (fire #1 ở mốc 1000 trước khi phím #2 tới — tick trước press cùng mốc).
/// 5 gap × 10 cặp phím = 50.
fn build_p1_c() -> Vec<(String, u64, Key, Key)> {
    let gaps = [999u64, 1000, 1001, 1200, 1500];
    let pairs = [
        (Key::Char('a'), Key::Char('b')),
        (Key::Char('a'), Key::Backspace),
        (Key::Backspace, Key::Char('a')),
        (Key::Char('.'), Key::Char('b')),
        (Key::Char('b'), Key::Char('.')),
        (Key::Char('a'), Key::Char('a')),
        (Key::Backspace, Key::Backspace),
        (Key::Char('.'), Key::Char('.')),
        (Key::Char(' '), Key::Char('a')),
        (Key::Char('a'), Key::Char(' ')),
    ];
    let mut cases = Vec::new();
    for (gi, gap) in gaps.iter().enumerate() {
        for (pi, (a, b)) in pairs.iter().enumerate() {
            cases.push((format!("C{gi}.{pi}"), *gap, *a, *b));
        }
    }
    cases
}

fn run_p1() -> Result<(), String> {
    let mut fail = Vec::new();
    // A — burst n phím.
    for (id, n, gap, last) in build_p1_a() {
        let mut m = PanelModel::default();
        for i in 0..n {
            let key = if i + 1 == n {
                last
            } else {
                Key::Char((b'a' + (i % 3) as u8) as char)
            };
            m.press(i as u64 * gap, key, false);
        }
        let t_last = (n as u64 - 1) * gap;
        m.tick(t_last + 999, false);
        if m.fires != 0 {
            fail.push(format!(
                "P1/A{id}: fire sớm trước {t_last}+999 (fires={})",
                m.fires
            ));
        }
        m.tick(t_last + IDLE_MS, false);
        if m.fires != 1 {
            fail.push(format!(
                "P1/A{id}: phải đúng 1 fire ở {}+1000 (fires={})",
                t_last, m.fires
            ));
        }
        m.tick(t_last + 3000, false);
        if m.fires != 1 || m.fire_at.is_some() {
            fail.push(format!(
                "P1/A{id}: fire lặp hoặc pending sót (fires={}, pending={:?})",
                m.fires, m.fire_at
            ));
        }
        if m.visible {
            fail.push(format!("P1/A{id}: câu sạch không được hiện popup"));
        }
    }
    // B — ngưỡng dừng 999/1000.
    for (id, idle, key) in build_p1_b() {
        let mut m = PanelModel::default();
        m.press(0, key, false);
        if m.fire_at != Some(IDLE_MS) {
            fail.push(format!(
                "P1/B{id}: fire-time phải 1000 (got {:?})",
                m.fire_at
            ));
        }
        m.tick(idle, false);
        if idle >= IDLE_MS {
            if m.fires != 1 {
                fail.push(format!(
                    "P1/B{id}: dừng {idle} ms phải fire (fires={})",
                    m.fires
                ));
            }
            if m.fire_at.is_some() {
                fail.push(format!("P1/B{id}: fire xong còn pending"));
            }
        } else if m.fires != 0 {
            fail.push(format!(
                "P1/B{id}: dừng {idle} ms KHÔNG được fire (fires={})",
                m.fires
            ));
        }
        if m.visible {
            fail.push(format!("P1/B{id}: câu sạch không được hiện popup"));
        }
    }
    // C — 2 burst cách nhau.
    for (id, gap1, a, b) in build_p1_c() {
        let mut m = PanelModel::default();
        m.press(0, a, false);
        m.press(gap1, b, false);
        let want = if gap1 >= IDLE_MS { 2 } else { 1 };
        m.tick(gap1 + IDLE_MS, false);
        if m.fires != want {
            fail.push(format!(
                "P1/C{id}: gap {gap1} ⇒ {want} fire (got {})",
                m.fires
            ));
        }
        m.tick(gap1 + 4000, false);
        if m.fires != want || m.fire_at.is_some() {
            fail.push(format!(
                "P1/C{id}: fire/pending lẻ loi sau cùng (fires={}, pending={:?})",
                m.fires, m.fire_at
            ));
        }
    }
    if fail.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} case sai:\n{}",
            fail.len(),
            fail.iter()
                .map(|f| format!("  {}", clip(f, 60)))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[test]
fn p1_timing_matrix_290_case() {
    let a = build_p1_a();
    let b = build_p1_b();
    let c = build_p1_c();
    assert_eq!(a.len() + b.len() + c.len(), N_P1, "đúng 290 case P1");
    run_p1().expect("P1 timing matrix");
}

// ===========================================================================
// P2 — hide-on-type matrix (300 case)
// ===========================================================================

const P2_SOURCES: [&str; 3] = ["dot", "bang", "hoi"];
const P2_SOURCE_KEYS: [Key; 3] = [Key::Char('.'), Key::Char('!'), Key::Char('?')];
const P2_NEXT: [(&str, Key); 12] = [
    ("char_a", Key::Char('a')),
    ("char_space", Key::Char(' ')),
    ("char_comma", Key::Char(',')),
    ("char_dot", Key::Char('.')),
    ("char_digit", Key::Char('5')),
    ("char_vn", Key::Char('Ạ')),
    ("backspace", Key::Backspace),
    ("enter", Key::Enter),
    ("tab", Key::Tab),
    ("ctrl_space", Key::CtrlSpace),
    ("escape", Key::Escape),
    ("other", Key::Other),
];
const P2_DELAYS: [u64; 4] = [0, 10, 100, 999];

/// Setup chung: trigger câu ('.'/'!'/'?') + câu còn lỗi ⇒ schedule tại t=0 ⇒
/// dừng 1 s ⇒ timer fire ⇒ popup sống hiện (`visible=true`, fires=1).
fn setup_p2_popup(m: &mut PanelModel, source: Key) -> Result<(), String> {
    assert!(
        is_sentence_trigger(source),
        "setup P2: nguồn popup phải là trigger câu P9"
    );
    m.press(0, source, true);
    if m.visible {
        return Err("P9: trigger câu KHÔNG được hiện popup ngay (phải chờ idle 1s)".into());
    }
    m.tick(IDLE_MS, true);
    if m.fires != 1 || m.fire_at.is_some() {
        return Err(format!(
            "setup: timer phải fire đúng 1 lần (fires={}, pending={:?})",
            m.fires, m.fire_at
        ));
    }
    if !m.visible {
        return Err("setup: sau fire (câu còn lỗi) popup sống phải hiện".into());
    }
    Ok(())
}

fn run_p2() -> Result<(), String> {
    let mut fail = Vec::new();
    // 3 source × 12 phím-kế × 4 delay × 2 số-phím = 288.
    for (si, sn) in P2_SOURCES.into_iter().enumerate() {
        let source = P2_SOURCE_KEYS[si];
        for (ni, (nn, next)) in P2_NEXT.into_iter().enumerate() {
            for (di, delay) in P2_DELAYS.into_iter().enumerate() {
                for rep in [1usize, 2] {
                    let id = format!("M{sn}.{ni}{nn}.d{di}.r{rep}");
                    let mut m = PanelModel::default();
                    if let Err(e) = setup_p2_popup(&mut m, source) {
                        fail.push(format!("P2/{id}: {e}"));
                        continue;
                    }
                    let t0 = IDLE_MS + delay;
                    for i in 0..rep {
                        m.press(t0 + i as u64 * 50, next, true);
                    }
                    if is_typing(next) {
                        if m.visible || m.hides != 1 {
                            fail.push(format!(
                                "P2/{id}: popup sống + phím gõ ⇒ ẨN NGAY (visible={}, hides={})",
                                m.visible, m.hides
                            ));
                        }
                    } else if !m.visible || m.hides != 0 {
                        fail.push(format!(
                            "P2/{id}: phím không-gõ không được ẩn popup (visible={}, hides={})",
                            m.visible, m.hides
                        ));
                    }
                }
            }
        }
    }
    // Extras — gõ 2 phím liên tiếp sau popup sống: chỉ ẨN 1 lần (popup đã ẩn
    // thì phím sau không tăng `hides`). 3 source × 4 cặp = 12.
    for sn in P2_SOURCES.into_iter() {
        let source = match sn {
            "dot" => Key::Char('.'),
            "bang" => Key::Char('!'),
            _ => Key::Char('?'),
        };
        let seqs: [(&str, [Key; 2]); 4] = [
            ("char-bs", [Key::Char('x'), Key::Backspace]),
            ("bs-char", [Key::Backspace, Key::Char('x')]),
            ("char-char", [Key::Char('x'), Key::Char('y')]),
            ("bs-bs", [Key::Backspace, Key::Backspace]),
        ];
        for (qn, seq) in seqs {
            let id = format!("X{sn}.{qn}");
            let mut m = PanelModel::default();
            if let Err(e) = setup_p2_popup(&mut m, source) {
                fail.push(format!("P2/{id}: {e}"));
                continue;
            }
            for (i, key) in seq.into_iter().enumerate() {
                m.press(IDLE_MS + 50 + i as u64 * 50, key, true);
            }
            if m.visible || m.hides != 1 {
                fail.push(format!(
                    "P2/{id}: 2 phím gõ liên tiếp chỉ ẩn 1 lần (visible={}, hides={})",
                    m.visible, m.hides
                ));
            }
        }
    }
    if fail.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} case sai:\n{}",
            fail.len(),
            fail.iter()
                .map(|f| format!("  {}", clip(f, 60)))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[test]
fn p2_hide_on_type_matrix_300_case() {
    // 288 + 12 = 300 (meta đếm riêng dưới).
    run_p2().expect("P2 hide-on-type matrix");
}

fn build_p2_ids() -> Vec<String> {
    let mut ids = Vec::new();
    for sn in P2_SOURCES.into_iter() {
        for (ni, (nn, _)) in P2_NEXT.into_iter().enumerate() {
            for di in 0..P2_DELAYS.len() {
                for rep in [1usize, 2] {
                    ids.push(format!("M{sn}.{ni}{nn}.d{di}.r{rep}"));
                }
            }
        }
    }
    for sn in P2_SOURCES.into_iter() {
        for qn in ["char-bs", "bs-char", "char-char", "bs-bs"] {
            ids.push(format!("X{sn}.{qn}"));
        }
    }
    ids
}

// ===========================================================================
// P3 — schedule-throttle (200 case)
// ===========================================================================

const P3_KEYS: [(&str, Key); 9] = [
    ("char_a", Key::Char('a')),
    ("char_space", Key::Char(' ')),
    ("char_dot", Key::Char('.')),
    ("char_comma", Key::Char(',')),
    ("backspace", Key::Backspace),
    ("enter", Key::Enter),
    ("tab", Key::Tab),
    ("ctrl_space", Key::CtrlSpace),
    ("other", Key::Other),
];

fn run_p3() -> Result<(), String> {
    let mut fail = Vec::new();
    // S1 — 1 phím (hoặc 3 phím cùng mốc) sau khi pending: phím gõ reset
    // fire-time từ phím mới nhất; phím không-gõ giữ fire-time cũ.
    // 9 phím × 5 mốc × 2 số-phím = 90.
    for (ki, (kn, key)) in P3_KEYS.into_iter().enumerate() {
        for (ti, t) in [0u64, 250, 500, 750, 999].into_iter().enumerate() {
            for rep in [1usize, 3] {
                let id = format!("S1{ki}{kn}.{ti}.r{rep}");
                let mut m = PanelModel::default();
                m.press(0, Key::Char('k'), false); // schedule gốc: fire ở 1000
                for _ in 0..rep {
                    m.press(t, key, false);
                }
                let want = if is_typing(key) { t + IDLE_MS } else { IDLE_MS };
                if m.fire_at != Some(want) {
                    fail.push(format!(
                        "P3/{id}: fire-time phải {want} (got {:?})",
                        m.fire_at
                    ));
                }
                m.tick(want, false);
                if m.fires != 1 || m.fire_at.is_some() {
                    fail.push(format!(
                        "P3/{id}: đúng 1 fire rồi hết pending (fires={}, pending={:?})",
                        m.fires, m.fire_at
                    ));
                }
                m.tick(want + 3000, false);
                if m.fires != 1 {
                    fail.push(format!("P3/{id}: fire lặp ({})", m.fires));
                }
            }
        }
    }
    // S2 — phím không-gõ chen giữa KHÔNG kéo dài fire; phím gõ sau đó reset.
    // 2 phím-gõ × 6 mốc × 5 phím-không-gõ = 60.
    let typing2 = [("char_b", Key::Char('b')), ("backspace", Key::Backspace)];
    let fillers = [
        Key::Enter,
        Key::Tab,
        Key::CtrlSpace,
        Key::Other,
        Key::Escape,
    ];
    for (ti, t2) in [100u64, 300, 500, 700, 900, 999].into_iter().enumerate() {
        for (k2i, (k2n, key2)) in typing2.into_iter().enumerate() {
            for (fi, filler) in fillers.into_iter().enumerate() {
                let id = format!("S2{ti}.{k2i}{k2n}.{fi}");
                let mut m = PanelModel::default();
                m.press(0, Key::Char('k'), false);
                m.press(t2 / 2, filler, false);
                if m.fire_at != Some(IDLE_MS) {
                    fail.push(format!(
                        "P3/{id}: filler không-gõ phải giữ fire 1000 (got {:?})",
                        m.fire_at
                    ));
                }
                m.press(t2, key2, false);
                if m.fire_at != Some(t2 + IDLE_MS) {
                    fail.push(format!(
                        "P3/{id}: phím gõ tại {t2} phải reset fire → {} (got {:?})",
                        t2 + IDLE_MS,
                        m.fire_at
                    ));
                }
                m.tick(t2 + IDLE_MS, false);
                if m.fires != 1 {
                    fail.push(format!("P3/{id}: đúng 1 fire (got {})", m.fires));
                }
            }
        }
    }
    // S3 — "phím gõ cuối cùng thắng": cặp phím gõ liên tiếp cách `gap` ⇒ fire
    // = gap + 1000. 6 cặp × 7 gap = 42; +8: filler-then-typing.
    for (ai, a) in [Key::Char('a'), Key::Char('.'), Key::Backspace]
        .into_iter()
        .enumerate()
    {
        for (bi, b) in [Key::Char('b'), Key::Backspace].into_iter().enumerate() {
            for (gi, gap) in [10u64, 100, 250, 500, 750, 900, 999]
                .into_iter()
                .enumerate()
            {
                let id = format!("S3{ai}.{bi}.{gi}");
                let mut m = PanelModel::default();
                m.press(0, a, false);
                m.press(gap, b, false);
                if m.fire_at != Some(gap + IDLE_MS) {
                    fail.push(format!(
                        "P3/{id}: phím cuối tại {gap} thắng ⇒ fire {} (got {:?})",
                        gap + IDLE_MS,
                        m.fire_at
                    ));
                }
            }
        }
    }
    for (ni, nt) in [Key::Enter, Key::Tab, Key::CtrlSpace, Key::Other]
        .into_iter()
        .enumerate()
    {
        for (ti, t_nt) in [500u64, 900].into_iter().enumerate() {
            let id = format!("S3f{ni}.{ti}");
            let mut m = PanelModel::default();
            m.press(0, Key::Char('k'), false);
            m.press(t_nt, nt, false);
            if m.fire_at != Some(IDLE_MS) {
                fail.push(format!(
                    "P3/{id}: filler {t_nt} không được đổi fire (got {:?})",
                    m.fire_at
                ));
            }
            m.press(950, Key::Char('z'), false);
            if m.fire_at != Some(950 + IDLE_MS) {
                fail.push(format!(
                    "P3/{id}: phím gõ 950 phải reset fire 1950 (got {:?})",
                    m.fire_at
                ));
            }
        }
    }
    if fail.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} case sai:\n{}",
            fail.len(),
            fail.iter()
                .map(|f| format!("  {}", clip(f, 60)))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[test]
fn p3_schedule_throttle_200_case() {
    let s1 = P3_KEYS.len() * 5 * 2; // 90
    let s2 = 2 * 6 * 5; // 60
    let s3 = 3 * 2 * 7 + 4 * 2; // 42 + 8 = 50
    assert_eq!(s1 + s2 + s3, N_P3, "đúng 200 case P3");
    run_p3().expect("P3 schedule-throttle");
}

// ===========================================================================
// P4a — check-on-timer path: TsfShared thật → check_sentence_on_demand (150)
// ===========================================================================

/// Dựng `TsfShared` thật: FFI engine (EngineSession) + IpcClient global +
/// ThreadState mặc định (assist highlight, auto_capitalize=true, per_app rỗng).
fn make_shared() -> Rc<TsfShared> {
    let ipc = textvn_win_tsf::ipc_client::IpcClient::global("panel-idle-1000.test.exe");
    let thread = ThreadState::new("panel-idle-1000.test.exe", 0)
        .expect("ThreadState::new — FFI engine phải dựng được");
    Rc::new(TsfShared::new(0, ipc, thread))
}

/// Gõ nguyên câu qua engine (Telex-an toàn: các câu dưới không có cặp ký tự
/// biến đổi Telex) — đúng đường buffer câu D8/D31 của core.
fn feed(shared: &TsfShared, text: &str) {
    let mut thread = shared.thread.borrow_mut();
    for c in text.chars() {
        let _ = thread.engine.key_char(c);
    }
}

/// Kết quả 1 lần chạy: (câu engine giữ, số lỗi) hoặc None khi im lặng.
fn run_sentence_once(text: &str) -> Option<(String, usize)> {
    let shared = make_shared();
    feed(&shared, text);
    check_sentence_on_demand(&shared).map(|c| {
        let n = c.issue_count();
        (c.sentence, n)
    })
}

/// 50 câu: 30 câu có lỗi cơ học (double-space / dấu cách trước dấu câu / từ
/// lặp) phải Some; 20 câu sạch phải None. Chữ "có lỗi" của engine là hằng số
/// FFI — nếu engine đổi luật, case báo thẳng id câu để sửa bảng.
const P4A_ERROR: [&str; 30] = [
    "hom nay  troi qua dep.",
    "toi  di hoc.",
    "hom nay , troi met.",
    "hom nay . troi met.",
    "hom nay nay troi dep.",
    "toi lam  viec.",
    "con meo  di san.",
    "ban lam viec tot .",
    "ttoi  di hoc.",
    "hom  nay troi dep.",
    "troi  qua dep.",
    "hom nay troi  dep.",
    "hom nay troi dep ,",
    "toi  di hoc , ban  o nha.",
    "hom nay ,troi met.",
    "con  meo di san.",
    "ban  o dau.",
    "hom nay troi dep , ban lam viec tot .",
    "toi di hoc  . ban o nha.",
    "hom nay  , troi met.",
    "ban o  dau.",
    "troi qua  dep.",
    "hom nay  troi  dep  qua.",
    "toi  , ban , no  di hoc.",
    "meo  chon chuot.",
    "hom nay . ban lam viec.",
    "toi di hoc .",
    "ban  lam  gi.",
    "hom nay troi dep .",
    "con  cho  chon  chuot.",
];

const P4A_CLEAN: [&str; 20] = [
    "hom nay troi dep.",
    "toi di hoc.",
    "con meo di san.",
    "ban lam viec tot.",
    "troi qua dep.",
    "hom nay ban o dau.",
    "toi di cho choi.",
    "con cho chon chuot.",
    "ban mua gi hom nay.",
    "toi thich an com.",
    "hom nay met qua.",
    "ban di dau do.",
    "toi lam viec ca ngay.",
    "meo chon chuot trong nha.",
    "ban mua qua cho toi.",
    "troi mua ngay hom nay.",
    "toi di hoc som mai.",
    "ban an sang chua.",
    "toi nghe nhac moi.",
    "hom nay la chu nhat.",
];

fn p4a_sentences() -> Vec<(&'static str, bool)> {
    P4A_ERROR
        .into_iter()
        .map(|s| (s, true))
        .chain(P4A_CLEAN.into_iter().map(|s| (s, false)))
        .collect()
}

fn run_p4a() -> Result<(), String> {
    let gate_on = make_shared().is_assist_enabled();
    let mut fail = Vec::new();
    let mut n_some = 0usize;
    for (i, (text, want_error)) in p4a_sentences().into_iter().enumerate() {
        // (1) outcome: Some ⟺ câu có lỗi (gate tắt ⇒ luôn None — hợp đồng D20).
        let got = run_sentence_once(text);
        let want = want_error && gate_on;
        if got.is_some() != want {
            fail.push(format!(
                "P4a-E{:02}-outcome [{}]: muốn {} got {}",
                i + 1,
                clip(text, 60),
                if want { "Some" } else { "None" },
                if got.is_some() { "Some" } else { "None" }
            ));
        }
        // (2) shape: Some ⇒ 1..=8 lỗi (IME_MAX_ASSIST_ISSUES) + câu không rỗng.
        if let Some((sentence, count)) = &got {
            if !(1..=8).contains(count) {
                fail.push(format!(
                    "P4a-E{:02}-shape: count {count} ngoài 1..=8",
                    i + 1
                ));
            }
            if sentence.trim().is_empty() {
                fail.push(format!(
                    "P4a-E{:02}-shape: check trả câu rỗng [{}]",
                    i + 1,
                    clip(text, 60)
                ));
            }
        }
        // (3) determinism: chạy lại từ đầu (TsfShared mới) → cùng kết quả.
        let got2 = run_sentence_once(text);
        if got != got2 {
            fail.push(format!(
                "P4a-E{:02}-determinism: chạy lại khác kết quả ({got:?} vs {got2:?})",
                i + 1
            ));
        }
        if got.is_some() {
            n_some += 1;
        }
    }
    // Chống pass rỗng: gate BẬT mà KHÔNG câu nào Some ⇒ bảng lỗi chết hết
    // (engine đổi luật) — phải báo, không được nuốt thành "đều None".
    if gate_on && n_some == 0 {
        fail.push("P4a: gate bật nhưng 0/30 câu lỗi trả Some — engine check đổi luật?".to_string());
    }
    eprintln!("P4a: gate_on={gate_on}, câu trả Some={n_some}/50 (mong đợi 30 khi gate bật)");
    if fail.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} case sai:\n{}",
            fail.len(),
            fail.iter()
                .map(|f| format!("  {}", clip(f, 60)))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[test]
fn p4a_check_on_timer_150_case() {
    let sentences = p4a_sentences();
    assert_eq!(sentences.len(), 50, "50 câu P4a");
    run_p4a().expect("P4a check-on-timer (3 assert × 50 câu)");
}

// ===========================================================================
// P4b — hợp đồng no-panic cancel/on_timer (30) + G8 clip (20)
// ===========================================================================

fn run_p4b_contract() -> Result<(), String> {
    // PENDING luôn rỗng (không test nào gọi schedule — cần ITfContext thật);
    // TIMER_HWND = 0 trong thread test ⇒ KillTimer bị bỏ qua ⇒ cancel/on_timer
    // là no-op thuần: bắt buộc KHÔNG panic, không đổi trạng thái nhìn thấy.
    let fail: Vec<String> = Vec::new();
    // 10: cancel() gọi n lần liên tiếp.
    for n in 1..=10usize {
        for _ in 0..n {
            panel_idle::cancel();
        }
        panel_idle::on_timer(); // trạng thái vẫn hợp lệ sau loạt cancel
    }
    // 10: on_timer() gọi n lần liên tiếp (PENDING rỗng → về ngay, không panic).
    for n in 1..=10usize {
        for _ in 0..n {
            panel_idle::on_timer();
        }
        panel_idle::cancel();
    }
    // 5: xen kẽ cancel/on_timer (2n lần).
    for n in 1..=5usize {
        for _ in 0..(2 * n) {
            panel_idle::cancel();
            panel_idle::on_timer();
        }
    }
    // 5: sau khi dựng + drop TsfShared (Weak trong PENDING — nếu sau này
    // schedule được gọi ở thread này, weak hỏng cũng phải im lặng, không panic).
    for _ in 0..5 {
        let shared = make_shared();
        drop(shared);
        panel_idle::on_timer();
        panel_idle::cancel();
    }
    if !fail.is_empty() {
        return Err(fail.join("; "));
    }
    Ok(())
}

fn run_p4b_g8() -> Result<(), String> {
    let mut fail = Vec::new();
    let mut case = 0usize;
    let mut check = |cond: bool, msg: &str, fail: &mut Vec<String>| {
        case += 1;
        if !cond {
            fail.push(format!("G8-{case:02}: {msg}"));
        }
    };
    check(clip("", 60).is_empty(), "chuỗi rỗng ⇒ rỗng", &mut fail);
    check(clip("a", 60) == "a", "1 ký tự giữ nguyên", &mut fail);
    check(
        clip(&"x".repeat(60), 60) == "x".repeat(60),
        "đúng 60 ký tự ⇒ không thêm …",
        &mut fail,
    );
    check(
        clip(&"x".repeat(61), 60) == format!("{}…", "x".repeat(60)),
        "61 ký tự ⇒ 60 + …",
        &mut fail,
    );
    check(
        clip(&"y".repeat(100), 60).chars().count() == 61,
        "100 ký tự ⇒ đầu ra 61 ký tự",
        &mut fail,
    );
    {
        let vn = "Đây là câu tiếng Việt có dấu — cần cắt theo ký tự Unicode chứ không phải byte!";
        let cut = clip(vn, 60);
        check(
            cut.chars().count() == 61,
            "tiếng Việt cắt theo char",
            &mut fail,
        );
        check(
            clip(vn, 60) == vn.chars().take(60).collect::<String>() + "…",
            "tiếng Việt cắt đúng vị trí char 60",
            &mut fail,
        );
        let vn60: String = "Tiếng Việt có dấu nhỏ hơn giới hạn sáu mươi ký tự hợp lệ."
            .chars()
            .take(60)
            .collect();
        check(
            !clip(&vn60, 60).contains('…') && clip(&vn60, 60) == vn60,
            "đúng 60 char tiếng Việt ⇒ không …",
            &mut fail,
        );
    }
    check(clip("a\nb", 60) == "a\\nb", "\\n được escape", &mut fail);
    check(clip("a\rb", 60) == "a\\rb", "\\r được escape", &mut fail);
    check(clip("a\tb", 60) == "a\\tb", "\\t được escape", &mut fail);
    check(
        clip("\u{1}", 60) == "\\u{0001}",
        "control char 0x01 escape \\u{0001}",
        &mut fail,
    );
    check(
        clip("\u{7F}", 60) == "\\u{007F}",
        "control char 0x7F escape \\u{007F}",
        &mut fail,
    );
    check(
        clip("a\u{1}\nb", 60) == "a\\u{0001}\\nb",
        "mix control + text escape đủ",
        &mut fail,
    );
    {
        let emo = "😀x".repeat(40);
        let cut = clip(&emo, 60);
        check(
            cut.chars().count() == 61,
            "emoji cắt theo char (1 char/emoji)",
            &mut fail,
        );
        check(
            !cut.contains('\u{FFFD}'),
            "không sinh surrogate hỏng",
            &mut fail,
        );
    }
    check(
        !clip("a\nb\r\tc", 60).contains('\n'),
        "đầu ra không chứa \n thô",
        &mut fail,
    );
    {
        let s = "câu dài có ký tự điều khiển \t và tiếng Việt đa bottled ống …";
        let cut = clip(s, 60);
        check(
            cut.chars().count() <= 61,
            "clip ≤ 61 ký tự (60 + …)",
            &mut fail,
        );
        let again = clip(&cut, 60);
        check(
            again == cut,
            "clip idempotent trên đầu ra đã cắt",
            &mut fail,
        );
    }
    // Mọi câu trong bảng P4a khi xuất hiện trong assert phải nằm gọn trong 60
    // ký tự sau clip — G8: không log nguyên câu người dùng.
    let all_fit = p4a_sentences()
        .iter()
        .all(|(s, _)| clip(s, 60).chars().count() <= 61);
    check(all_fit, "50 câu bảng P4a clip ≤ 61 ký tự", &mut fail);
    if fail.is_empty() {
        Ok(())
    } else {
        Err(fail.join("; "))
    }
}

#[test]
fn p4b_contract_and_g8_50_case() {
    run_p4b_contract().expect("P4b hợp đồng no-panic (30 case)");
    run_p4b_g8().expect("P4b G8 clip-60 (20 case)");
}

// ===========================================================================
// Meta — đếm đúng 1000, id duy nhất, deterministic
// ===========================================================================

#[test]
fn meta_deterministic_1000() {
    let a = build_p1_a();
    let b = build_p1_b();
    let c = build_p1_c();
    let p2 = build_p2_ids();
    let s4a = p4a_sentences();
    assert_eq!(a.len() + b.len() + c.len(), N_P1, "P1 = 290");
    assert_eq!(p2.len(), N_P2, "P2 = 300");
    let s1 = P3_KEYS.len() * 5 * 2;
    let s2 = 2 * 6 * 5;
    let s3 = 3 * 2 * 7 + 4 * 2;
    assert_eq!(s1 + s2 + s3, N_P3, "P3 = 200");
    assert_eq!(s4a.len() * 3, N_P4A, "P4a = 150");
    // P4b: 30 (10 cancel + 10 on_timer + 5 xen kẽ + 5 sau-drop) + 20 G8.
    assert_eq!(10 + 10 + 5 + 5 + 20, N_P4B, "P4b = 50");
    // P0: 10 assert đánh số trong p0_source_contract_10_case.
    assert_eq!(
        N_P0 + N_P1 + N_P2 + N_P3 + N_P4A + N_P4B,
        N_TOTAL,
        "tổng = 1000"
    );
    // P2 id duy nhất + generator deterministic (build 2 lần giống hệt).
    let mut sorted = p2.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), p2.len(), "P2 id phải duy nhất");
    assert_eq!(p2, build_p2_ids(), "P2 generator deterministic");
    let s4a2 = p4a_sentences();
    assert_eq!(s4a, s4a2, "bảng câu P4a deterministic");
    // P1 generator deterministic.
    assert_eq!(a, build_p1_a(), "P1-A deterministic");
    assert_eq!(b, build_p1_b(), "P1-B deterministic");
    assert_eq!(c, build_p1_c(), "P1-C deterministic");
    // Ngưỡng quan trọng nhất của P9 — chốt lần cuối ở meta.
    assert_eq!(PANEL_IDLE_MS, IDLE_MS as u32, "PANEL_IDLE_MS == 1000");
}
