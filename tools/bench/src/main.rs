// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-bench` — micro-bench engine (P1-5 §5).
//!
//! ```bash
//! cargo run -p textvn-bench --release -- write  perf/baseline-win.json   # ghi baseline
//! cargo run -p textvn-bench --release -- check  perf/baseline-win.json   # CI: >10% chậm hơn → exit 1
//! cargo run -p textvn-bench --release                                    # in p50/p99
//! ```
//!
//! Ngân sách (P0-3 §3.3 / PLAN §3.3): `ime_key` p99 < 0.5 ms, `resolve` p99 < 2 ms.
//! Số đo bằng `std::time::Instant`, lấy **p50/p99** (không trung bình — tail mới quan trọng).
//! Không dùng criterion để giữ 0 dependency (`cargo deny` + CI 3 OS).

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};
use textvn_ffi::*;
use textvn_field_detect::{FieldContext, ProbeCache, ProbeSnapshot, SecurityState};

/// Ngưỡng hồi quy (P1-5 §5): chậm hơn baseline trên 10% là fail.
const REGRESSION_PCT: f64 = 10.0;
/// Dưới mức này (ns/lời gọi) thì phép so % là đo nhiễu timer, không phải hồi quy thật.
const NOISE_FLOOR_NS: f64 = 100.0;

/// Một phép đo: tên, số mẫu, p50/p99 (micro-giây).
struct Measurement {
    name: &'static str,
    samples: usize,
    p50_ns: u128,
    p99_ns: u128,
    mean_ns: u128,
}

impl Measurement {
    /// p99 có đạt ngân sách không (0 = không đặt ngân sách).
    fn budget_ns(&self) -> u128 {
        match self.name {
            "ime_key" => 500_000,                // 0.5 ms (P0-3 §3.3)
            "ime_strategy_resolve" => 2_000_000, // 2 ms (P0-3 §3.3)
            "field_switch_resolve" => 2_000_000, // WIN-031: 200 app/hwnd cache
            _ => 0,
        }
    }
    fn to_json(&self) -> String {
        format!(
            "{{\"name\":\"{}\",\"samples\":{},\"p50_ns\":{},\"p99_ns\":{},\"mean_ns\":{},\"budget_ns\":{}}}",
            self.name,
            self.samples,
            self.p50_ns,
            self.p99_ns,
            self.mean_ns,
            self.budget_ns()
        )
    }
}

/// Đo `f` chạy `iters` lần, trả p50/p99/mean (ns).
///
/// **Đo theo lô** (mỗi lần bấm giờ = `batch` lời gọi rồi chia đều) vì `Instant` trên
/// Windows chỉ ngày ~100 ns — đo từng lời gọi cho số nhiễu toát, p50/p99 vô nghĩa.
fn measure<F: FnMut(usize)>(
    name: &'static str,
    iters: usize,
    batch: usize,
    mut f: F,
) -> Measurement {
    let batch = batch.max(1);
    // warm-up: dữ liệu nóng, không tính vào số đo
    for i in 0..(iters / 10).max(1) {
        for b in 0..batch {
            f(i * batch + b);
        }
    }
    let mut samples: Vec<u128> = Vec::with_capacity(iters);
    for i in 0..iters {
        let t0 = Instant::now();
        for b in 0..batch {
            f(i * batch + b);
        }
        samples.push(t0.elapsed().as_nanos() / batch as u128);
    }
    samples.sort_unstable();
    let p = |q: f64| -> u128 {
        let idx = ((samples.len() as f64 - 1.0) * q).round() as usize;
        samples[idx.min(samples.len() - 1)]
    };
    let mean = samples.iter().sum::<u128>() / samples.len() as u128;
    Measurement {
        name,
        samples: samples.len(),
        p50_ns: p(0.50),
        p99_ns: p(0.99),
        mean_ns: mean,
    }
}

/// Một chuỗi gõ đại diện: dài + ngắn + có dấu (không chỉ đo trường hợp dễ).
const TYPE_SAMPLES: [&str; 4] = ["duocj", "toi", "nguyen", "vn"];

/// `ime_key` trên chuỗi gõ thật (P0-3 §3.3: p99 < 0.5 ms).
fn bench_ime_key(iters: usize) -> Measurement {
    let mut inst: *mut ime_instance = std::ptr::null_mut();
    let rc = ime_instance_new(std::ptr::null(), 0, &mut inst);
    assert_eq!(rc, IME_OK, "instance phải tạo được (không lỗi config)");
    let ctx = ime_context_v1 {
        abi_version: IME_ABI_VERSION,
        enabled: 1,
        secure: 0,
        field_role: IME_FIELD_BODY,
        caps: IME_CAP_PREEDIT | IME_CAP_INJECT_VK,
        app_id: std::ptr::null(),
        element_name: std::ptr::null(),
        hint: -1,
    };
    ime_set_context(inst, &ctx);
    let mut out = zero_result();
    let m = measure("ime_key", iters, 8, |i| {
        let s = TYPE_SAMPLES[i % TYPE_SAMPLES.len()];
        for ch in s.chars() {
            let key = ime_key_v1 {
                abi_version: IME_ABI_VERSION,
                vk: ch as u32,
                ch: ch as u32,
                mods: 0,
                key_down: 1,
                is_repeat: 0,
                is_injected: 0,
                _reserved: 0,
            };
            ime_key(inst, &key, &mut out);
        }
        // giữa các vòng: reset để không dồn buffer (đo đúng chi phí 1 từ)
        ime_reset(inst);
    });
    ime_instance_free(inst);
    m
}

fn zero_result() -> ime_result_v1 {
    ime_result_v1 {
        abi_version: 0,
        action: 0,
        delete_count: 0,
        insert_len: 0,
        preedit_len: 0,
        cursor_left: 0,
        flags: 0,
        insert: [0; IME_MAX_TEXT],
        preedit: [0; IME_MAX_TEXT],
    }
}

/// `parse_config` — parser chạy khi user bấm "Áp dụng" trong Settings (WIN-053).
fn bench_parse_config(iters: usize) -> Measurement {
    const CFG: &str = r#"{"config_version":1,"method":"telex","diacritic_style":"new",
        "auto_restore_english":true,"auto_capitalize":true,"macro_trigger":"tab",
        "macros":[{"trigger":"cty","expand":"Công ty TNHH","when":"always"}]}"#;
    measure("parse_config", iters, 16, |_| {
        // Cùng đường FFI dùng: `ime_reload_config` parse rồi đổi options.
        let mut inst: *mut ime_instance = std::ptr::null_mut();
        ime_instance_new(std::ptr::null(), 0, &mut inst);
        ime_reload_config(inst, CFG.as_ptr(), CFG.len());
        ime_instance_free(inst);
    })
}

/// `ime_strategy_resolve` — ngân sách p99 < 2 ms (P0-3 §3.3).
fn bench_resolve(iters: usize) -> Measurement {
    let ctx = ime_context_v1 {
        abi_version: IME_ABI_VERSION,
        enabled: 1,
        secure: 0,
        field_role: IME_FIELD_ADDRESS_BAR,
        caps: IME_CAP_SELECTION | IME_CAP_FIELD_DETECT,
        app_id: std::ptr::null(),
        element_name: std::ptr::null(),
        hint: -1,
    };
    let mut strategy: i64 = 0;
    measure("ime_strategy_resolve", iters, 128, |_| {
        ime_strategy_resolve(&ctx, std::ptr::null(), 0, &mut strategy);
    })
}

/// WIN-031: mô phỏng 200 cửa sổ/app đã có snapshot UIA trong cache. Đường này
/// chỉ lấy cache + dựng FieldContext + resolve strategy; tuyệt đối không gọi
/// UIA đồng bộ, vì hook callback không được block trên COM/UIA.
fn bench_field_switch_resolve(iters: usize) -> Measurement {
    const APPS: u64 = 200;
    let mut cache = ProbeCache::new(std::time::Duration::from_secs(2));
    for hwnd in 0..APPS {
        cache.insert(
            hwnd,
            ProbeSnapshot {
                field_role: IME_FIELD_BODY,
                security: SecurityState::NonSecure,
            },
            std::time::Duration::ZERO,
        );
    }
    measure("field_switch_resolve", iters, 64, |i| {
        let hwnd = (i as u64) % APPS;
        let snapshot = cache
            .get(&hwnd, std::time::Duration::from_millis(1))
            .expect("seed cache must be alive during benchmark");
        let mut ctx = FieldContext::pending(
            format!("app-{hwnd}.exe"),
            IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT,
            1,
        );
        assert!(ctx.apply_probe(1, snapshot.field_role, snapshot.security));
        black_box(ctx.resolve_strategy(true, None));
    })
}

fn platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    }
}

/// Ánh xạ `(std::env::consts::ARCH, std::env::consts::OS)` sang Rust target triple chuẩn.
fn target_triple_for(arch: &str, os: &str) -> &'static str {
    match (arch, os) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86", "windows") => "i686-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => "unknown",
    }
}

fn host_target_triple() -> &'static str {
    target_triple_for(std::env::consts::ARCH, std::env::consts::OS)
}

/// Chọn `perf/baseline-<triple>.json` nếu tồn tại, fallback về `perf/baseline-win.json`.
fn resolve_baseline_path(dir: &std::path::Path, triple: &str) -> std::path::PathBuf {
    let specific = dir.join(format!("baseline-{triple}.json"));
    if specific.is_file() {
        specific
    } else {
        dir.join("baseline-win.json")
    }
}

fn render(all: &[Measurement], with_platform: bool, dream: &DreamRsiStat) -> String {
    let mut s = String::from("{\n  \"schema\": \"textvn-bench.v1\",\n");
    if with_platform {
        let _ = writeln!(s, "  \"platform\": \"{}\",", platform());
        let _ = writeln!(s, "  \"target\": \"{}\",", host_target_triple());
    }
    let _ = writeln!(
        s,
        "  \"regression_pct\": {REGRESSION_PCT},\n  \"measurements\": ["
    );
    for (i, m) in all.iter().enumerate() {
        let comma = if i + 1 == all.len() { "" } else { "," };
        let _ = writeln!(s, "    {}{}", m.to_json(), comma);
    }
    let _ = writeln!(s, "  ],\n  \"dream_rsi\": {}", dream.to_json());
    s.push_str("}\n");
    s
}

/// So baseline cũ với số đo mới. `Ok` = trong ngân sách; `Err` = danh sách hồi quy.
///
/// **Hồi quy so `p50`, không so `p99`**: ở thang nano-giây, p99 bị chi phối bởi nhiễu
/// timer/scheduler (đo lại liên tiếp ra ±80%), nên ngưỡng 10% trên p99 là ngưỡng trên
/// tiếng ồn. `p99` vẫn được kiểm — theo **ngân sách tuyệt đối** (0.5 ms / 2 ms, P0-3 §3.3),
/// đó mới là con số có ý nghĩa cho người dùng.
fn compare(
    baseline_json: &str,
    fresh: &[Measurement],
    dream: &DreamRsiStat,
) -> Result<String, Vec<String>> {
    let mut fails = Vec::new();
    let mut lines = vec![format!(
        "platform={} — hồi quy theo p50 (ngưỡng {REGRESSION_PCT}%), p99 so ngân sách tuyệt đối",
        platform()
    )];
    for m in fresh {
        let key = format!("\"name\":\"{}\"", m.name);
        let Some(pos) = baseline_json.find(&key) else {
            fails.push(format!("{}: không có trong baseline", m.name));
            continue;
        };
        let tail = &baseline_json[pos..];
        let Some(p50_at) = tail.find("\"p50_ns\":") else {
            fails.push(format!("{}: baseline thiếu p50", m.name));
            continue;
        };
        let num: String = tail[p50_at + 9..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let Ok(old_p50) = num.parse::<f64>() else {
            fails.push(format!("{}: baseline p50 không đọc được", m.name));
            continue;
        };
        let pct = if old_p50 > 0.0 {
            (m.p50_ns as f64 - old_p50) / old_p50 * 100.0
        } else {
            0.0
        };
        // Ngân sách tuyệt đối (P0-3 §3.3) kiểm trước — không đợi hồi quy mới biết.
        let budget = m.budget_ns();
        if budget > 0 && m.p99_ns > budget {
            fails.push(format!(
                "{}: p99 {} ns VƯỢT ngân sách {budget} ns",
                m.name, m.p99_ns
            ));
        }
        // Dưới ngưỡng nhiễu (timer ~100 ns) thì % là vô nghĩa → chỉ so ngân sách, ghi rõ.
        let noisy = old_p50 < NOISE_FLOOR_NS;
        if !noisy && pct > REGRESSION_PCT {
            fails.push(format!(
                "{}: p50 {old_p50:.0} → {} ns ({pct:+.1}%)",
                m.name, m.p50_ns
            ));
        }
        lines.push(format!(
            "  {:<22} p50 {:>8} ns · p99 {:>9} ns / ngân sách {budget} · {}",
            m.name,
            m.p50_ns,
            m.p99_ns,
            if noisy {
                format!("bỏ qua % (p50 < {NOISE_FLOOR_NS:.0} ns — dưới nhiễu timer)")
            } else {
                format!("baseline p50 {old_p50:>8.0} ns · {pct:+6.1}%")
            }
        ));
    }
    // P10 Dream RSI (G19c): KSR-oracle giảm / nhiễu tăng so baseline = fail.
    check_dream_rsi(baseline_json, dream, &mut lines, &mut fails);
    if fails.is_empty() {
        Ok(lines.join("\n"))
    } else {
        Err(fails)
    }
}

/// Số vòng đo và **giữ vòng nhanh nhất** (best-of-N).
///
/// Lý do: tiến trình khác trên máy làm một vòng chậm đi, không phải code chậm đi. Chuẩn của
/// micro-bench là lấy **nhỏ nhất** trong N vòng — nó loại nhiễu lên từ hệ điều hành mà vẫn
/// giữ được xu hướng hồi quy thật của code. (Đo liên tiếp 3 vòng: chênh nhau tới 13% nếu
/// chỉ lấy 1 vòng ngẫu nhiên.)
const ROUNDS: u32 = 3;

// --- P10 Dream RSI (G19c): KSR-oracle trên corpus văn phòng — khối DETERMINISTIC ---
//
// RSI-1 (P0-7 Luồng R): hấp thụ spike SPK-RSI (`spikes/rsi-oracle` — đã xoá sau
// hấp thụ). Số cũ 21%/22% đến từ chính sách bench "gõ nửa từ → Tab khi có BẤT
// KỲ chữ mờ": nhận cả chữ mờ SAI nên phóng đại tiết kiệm (spike đo: bench
// 22,9% nhưng KSR-oracle 5,7%; VI 10,1%). Từ 0.2.96 người dùng mô phỏng chỉ
// nhận chữ mờ là **tiền tố đúng của phần còn lại** và hết ở ranh giới từ
// (Trnka & McCoy 2008) ⇒ `saved_keys` = KSR-oracle trung thực; kèm
// precision@ghost, nhiễu/100 phím, độ sâu chuỗi Tab nối tiếp (R2-173).

/// Corpus mô phỏng gõ văn phòng: VN Telex (transform thật) + EN từ điển phổ thông.
/// KHÔNG phụ thuộc máy (không timing) — cùng số trên 3 OS ⇒ so baseline được tuyệt đối.
const RSI_WORDS: [&str; 16] = [
    // VN gõ Telex (transform thật → engine học + ghost)
    "duocj", "tooi", "lamf", "vaof", "khoong", "ddeer", "vangf", "moocj",
    // EN (completion qua en_words10k / freq3000)
    "release", "report", "team", "project", "client", "contract", "review", "meeting",
];

/// RSI-2 (tách tập): câu văn phòng viết MỚI cho spike SPK-RSI, KHÔNG trùng
/// `data/*phrases*` (trigram overlap = 0 — kiểm chứng ở spike; RSI-2 sẽ chuyển
/// sang `corpus/eval/` + REUSE.toml). `xtask gen-*` KHÔNG được nạp corpus này
/// vào dữ liệu dự đoán — tệp đó đã sai thì gate KSR tự trắng.
const HELDOUT_VI: &str = include_str!("../../../corpus/eval/vi_office.txt");
const HELDOUT_EN: &str = include_str!("../../../corpus/eval/en_office.txt");

// ---------------------------------------------------------------------------------------
// Bộ mô phỏng văn bản (bản sao thứ 13 của `apply` trong repo — SIM-1 sẽ hợp nhất, G22)
// ---------------------------------------------------------------------------------------

fn sim_apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            if let Some(c) = k.printable() {
                if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                    buf.push(c);
                }
            }
        }
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            let del = *delete_count as usize;
            assert!(del <= buf.len(), "delete_count vượt buffer");
            buf.truncate(buf.len() - del);
            buf.extend_from_slice(insert);
        }
        Action::Commit { insert } => buf.extend_from_slice(insert),
    }
}

fn sim_press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    sim_apply(buf, &out.action, k);
}

fn special_key(code: u32, ch: char) -> KeyEvent {
    KeyEvent {
        vk: code,
        ch: ch as u32,
        key_down: true,
        ..Default::default()
    }
}
fn tab_key() -> KeyEvent {
    special_key(vk::TAB, '\t')
}
fn space_key() -> KeyEvent {
    special_key(vk::SPACE, ' ')
}
fn enter_key() -> KeyEvent {
    special_key(vk::RETURN, '\n')
}

/// Telex: chữ Việt có dấu → chuỗi phím (kiểu gõ dấu cuối từ, như người dùng UniKey phổ biến).
const SIM_VOWELS: [(&str, &str); 12] = [
    ("aàáảãạ", "a"),
    ("ăằắẳẵặ", "aw"),
    ("âầấẩẫậ", "aa"),
    ("eèéẻẽẹ", "e"),
    ("êềếểễệ", "ee"),
    ("iìíỉĩị", "i"),
    ("oòóỏõọ", "o"),
    ("ôồốổỗộ", "oo"),
    ("ơờớởỡợ", "ow"),
    ("uùúủũụ", "u"),
    ("ưừứửữự", "uw"),
    ("yỳýỷỹỵ", "y"),
];
const SIM_TONE_KEYS: [char; 6] = ['\0', 'f', 's', 'r', 'x', 'j'];

fn telex_keys(word: &str) -> String {
    let mut out = String::new();
    let mut tone = '\0';
    for c in word.chars() {
        if c == 'đ' {
            out.push_str("dd");
            continue;
        }
        let mut hit = false;
        for (set, base) in SIM_VOWELS {
            if let Some(pos) = set.chars().position(|v| v == c) {
                out.push_str(base);
                if pos > 0 {
                    tone = SIM_TONE_KEYS[pos];
                }
                hit = true;
                break;
            }
        }
        if !hit {
            out.push(c);
        }
    }
    if tone != '\0' {
        out.push(tone);
    }
    out
}

// ---------------------------------------------------------------------------------------
// Corpus → câu (chuỗi phím từng từ) + văn bản đích (gõ với dự đoán TẮT).
// ---------------------------------------------------------------------------------------

struct Sentence {
    /// Phím từng từ (không gồm dấu cách/Enter).
    words: Vec<Vec<KeyEvent>>,
    /// Văn bản đích của cả câu (kết thúc bằng '\n').
    target: Vec<char>,
    /// Vị trí trong `target` ngay sau dấu phân cách của từ i (dấu cách hoặc '\n').
    sep_end: Vec<usize>,
}

fn sim_opts() -> EngineOptions {
    EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        auto_restore_english: true,
        // Corpus toàn chữ thường — tắt capitalize để không nhiễu đo (spike SPK-RSI).
        auto_capitalize: false,
        ..Default::default()
    }
}

fn sim_keys(s: &str) -> Vec<KeyEvent> {
    s.chars().map(KeyEvent::char_down).collect()
}

/// Dựng câu từ danh sách chuỗi phím; đích = gõ CÙNG phím với dự đoán TẮT trên
/// engine RIÊNG của câu (engine tự làm oracle chính tả) ⇒ mọi khác biệt chỉ
/// đến từ dự đoán.
fn build_sentence(word_keys: &[Vec<KeyEvent>]) -> Sentence {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: false,
        suggest_space_complete: false,
        ..sim_opts()
    });
    let mut d = Vec::new();
    let mut sep_end = Vec::new();
    let n = word_keys.len();
    for (i, w) in word_keys.iter().enumerate() {
        for k in w {
            sim_press(&mut e, &mut d, k);
        }
        sim_press(
            &mut e,
            &mut d,
            &if i + 1 == n { enter_key() } else { space_key() },
        );
        sep_end.push(d.len());
    }
    Sentence {
        words: word_keys.to_vec(),
        target: d,
        sep_end,
    }
}

/// Corpus held-out → câu. VI encode Telex, EN gõ nguyên. Bỏ (và đếm) câu mà
/// gõ Telex dự đoán-tắt không ra đúng chữ — corpus commit phải rejected = 0
/// (assert ở `dream_rsi_oracle_run`); lệch là lỗi dữ liệu, không phải phím người dùng.
fn build_heldout(text: &str, is_vi: bool) -> (Vec<Sentence>, usize, usize) {
    let mut out = Vec::new();
    let mut lines = 0usize;
    let mut rejected = 0usize;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        lines += 1;
        let words: Vec<&str> = t.split_whitespace().collect();
        let keys: Vec<Vec<KeyEvent>> = words
            .iter()
            .map(|w| {
                sim_keys(&if is_vi {
                    telex_keys(w)
                } else {
                    (*w).to_string()
                })
            })
            .collect();
        let s = build_sentence(&keys);
        let want: String = format!("{}\n", words.join(" "));
        let got: String = s.target.iter().collect();
        if got != want {
            rejected += 1;
            eprintln!(
                "held-out {} bỏ dòng (Telex không tái lập): muốn {want:?} được {got:?}",
                if is_vi { "VI" } else { "EN" }
            );
            continue;
        }
        out.push(s);
    }
    (out, lines, rejected)
}

fn build_bench_corpus() -> Vec<Sentence> {
    (0..3)
        .map(|_| build_sentence(&RSI_WORDS.iter().map(|w| sim_keys(w)).collect::<Vec<_>>()))
        .collect()
}

// ---------------------------------------------------------------------------------------
// Số đo + người dùng oracle
// ---------------------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct OracleStat {
    baseline_keys: u64,
    actual_keys: u64,
    accepts: u64,
    /// Số vị trí phím có chữ mờ (nhìn được, đúng thứ adapter vẽ).
    ghosts_shown: u64,
    /// Chữ mờ đúng (dẫn tới văn bản đích).
    ghosts_good: u64,
    /// Chữ mờ hiện khi văn bản hiện tại CÒN là tiền tố của đích (loại trạng thái
    /// Telex trung gian như `vơ` trước phím dấu — ở đó không ghost nối-thêm nào đúng).
    ghosts_judgeable: u64,
    /// Chữ mờ đúng nhưng dừng giữa từ (người dùng mô phỏng không nhận).
    good_midword: u64,
    /// "chữ mờ == Tab chèn" bị vi phạm (R2-173 invariant).
    invariant_breaks: u64,
    /// câu có văn bản cuối ≠ đích sau khi nhận gợi ý.
    diverged: u64,
    max_chain: u64,
    chained_accepts: u64,
}

impl OracleStat {
    fn ksr_pct(&self) -> f64 {
        if self.baseline_keys == 0 {
            return 0.0;
        }
        100.0 * (self.baseline_keys.saturating_sub(self.actual_keys)) as f64
            / self.baseline_keys as f64
    }
    fn add(&mut self, o: &OracleStat) {
        self.baseline_keys += o.baseline_keys;
        self.actual_keys += o.actual_keys;
        self.accepts += o.accepts;
        self.ghosts_shown += o.ghosts_shown;
        self.ghosts_good += o.ghosts_good;
        self.ghosts_judgeable += o.ghosts_judgeable;
        self.good_midword += o.good_midword;
        self.invariant_breaks += o.invariant_breaks;
        self.diverged += o.diverged;
        self.max_chain = self.max_chain.max(o.max_chain);
        self.chained_accepts += o.chained_accepts;
    }
}

fn sim_baseline_keys(s: &Sentence) -> u64 {
    s.words.iter().map(|w| w.len() as u64 + 1).sum()
}

/// Người dùng oracle (Trnka & McCoy 2008): nhìn chữ mờ tại MỌI phím, chỉ nhận
/// Tab khi `văn bản + chữ mờ` là tiền tố của văn bản đích và hết ở ranh giới từ;
/// chữ mờ hiện mà không khớp = +1 nhiễu. Gồm chuỗi R2-173 (Tab nối tiếp).
///
/// `wrong_ghost` = ĐỘT BIẾN harness (test gate RSI-1): giả lập engine
/// "luôn hiện chữ mờ sai" — mọi vị trí xét được đều +1 nhiễu, không Tab nào được nhận
/// ⇒ KSR rơi về 0 và noise tăng vọt, gate G19c phải ĐỎ.
fn sim_run_oracle(e: &mut Engine, s: &Sentence, wrong_ghost: bool) -> OracleStat {
    let mut st = OracleStat {
        baseline_keys: sim_baseline_keys(s),
        ..Default::default()
    };
    let mut d: Vec<char> = Vec::new();
    let n = s.words.len();
    let (mut i, mut j) = (0usize, 0usize);
    let mut chain = 0u64;
    while i < n {
        if let Some(g) = e.peek_completion() {
            st.ghosts_shown += 1;
            if s.target.starts_with(&d) {
                st.ghosts_judgeable += 1;
            }
            let mut cand = d.clone();
            cand.extend(g.chars());
            if !wrong_ghost && cand.len() > d.len() && s.target.starts_with(&cand) {
                st.ghosts_good += 1;
                let l = cand.len();
                // Hết đúng ở cuối từ k (trước dấu phân cách) hoặc ngay sau dấu phân cách.
                let resume = (i..n).find_map(|k| {
                    if s.sep_end[k] - 1 == l {
                        Some((k, s.words[k].len()))
                    } else if s.sep_end[k] == l && k + 1 < n {
                        Some((k + 1, 0))
                    } else {
                        None
                    }
                });
                if let Some((ni, nj)) = resume {
                    sim_press(e, &mut d, &tab_key());
                    st.actual_keys += 1;
                    st.accepts += 1;
                    if chain > 0 {
                        st.chained_accepts += 1;
                    }
                    chain += 1;
                    st.max_chain = st.max_chain.max(chain);
                    if d != cand {
                        st.invariant_breaks += 1;
                        st.diverged += 1;
                        return st; // văn bản đã lệch — dừng câu (phần còn lại không tiết kiệm)
                    }
                    i = ni;
                    j = nj;
                    continue;
                }
                st.good_midword += 1;
            }
        }
        chain = 0;
        let w = &s.words[i];
        if j < w.len() {
            sim_press(e, &mut d, &w[j]);
            j += 1;
        } else {
            sim_press(
                e,
                &mut d,
                &if i + 1 == n { enter_key() } else { space_key() },
            );
            i += 1;
            j = 0;
        }
        st.actual_keys += 1;
    }
    if d != s.target {
        st.diverged += 1;
        // Chẩn đoán (chỉ chữ của corpus eval — không phải phím người dùng, G8).
        if std::env::var_os("TEXTVN_BENCH_DEBUG").is_some() {
            let want: String = s.target.iter().collect();
            let got: String = d.iter().collect();
            eprintln!("dream-rsi lệch đích: muốn {want:?} được {got:?}");
        }
    }
    st
}

/// Một phép đo Dream RSI (oracle) trên toàn corpus.
struct DreamRsiStat {
    /// Tổng 3 corpus: phím MÃ không prediction / phím thật (oracle) / tiết kiệm.
    baseline_keys: u64,
    actual_keys: u64,
    saved_keys: u64,
    /// KSR-oracle tổng = saved_keys × 100 / baseline_keys (làm tròn nguyên cho JSON).
    savings_rate_pct: u64,
    /// KSR-oracle theo corpus: bench 16 từ · VI held-out · EN held-out (số nguyên phím).
    bench_baseline: u64,
    bench_saved: u64,
    vi_baseline: u64,
    vi_saved: u64,
    en_baseline: u64,
    en_saved: u64,
    /// Đo chất lượng chữ mờ (oracle): hiện / đúng / xét được / đúng-giữa-từ.
    ghosts_shown: u64,
    ghosts_good: u64,
    ghosts_judgeable: u64,
    good_midword: u64,
    /// Chất lượng chữ mờ theo corpus (RSI-7: nhiễu VI phải giảm rõ so spike 45.2/100).
    vi_ghosts_shown: u64,
    vi_ghosts_good: u64,
    vi_actual: u64,
    en_ghosts_shown: u64,
    en_ghosts_good: u64,
    en_actual: u64,
    /// Chuỗi Tab nối tiếp (R2-173).
    max_chain: u64,
    chained_accepts: u64,
    /// Corpus held-out: câu đo được / câu bỏ (Telex không tái lập được chữ —
    /// từ EN mâu thuẫn Việt có chủ ý như "this/of/if/last", R2-56). Thông tin,
    /// không gate: corpus co/giãn ⇒ số KSR đổi ⇒ re-record baseline có ý thức.
    vi_sentences: usize,
    vi_rejected: usize,
    en_sentences: usize,
    en_rejected: usize,
    /// Bất biến/đích: phải luôn = 0, > 0 là lỗi engine (gate đỏ).
    invariant_breaks: u64,
    diverged: u64,
    /// Counters từ engine (đối chiếu chéo — saved_keys đo phím gõ thô,
    /// engine_chars_saved đo ký tự/glyph theo hook DreamRsi).
    engine_accepts: u64,
    engine_chars_saved: u64,
}

impl DreamRsiStat {
    fn ksr_pct(&self) -> f64 {
        pct(self.saved_keys, self.baseline_keys)
    }
    fn precision_pct(&self) -> f64 {
        pct(self.ghosts_good, self.ghosts_shown)
    }
    fn noise_per_100(&self) -> f64 {
        if self.actual_keys == 0 {
            0.0
        } else {
            100.0 * self.noise_stat() as f64 / self.actual_keys as f64
        }
    }
    fn noise_stat(&self) -> u64 {
        self.ghosts_shown.saturating_sub(self.ghosts_good)
    }
    fn to_json(&self) -> String {
        format!(
            "{{\"baseline_keys\":{},\"actual_keys\":{},\"saved_keys\":{},\"savings_rate_pct\":{},\"bench_baseline\":{},\"bench_saved\":{},\"vi_baseline\":{},\"vi_saved\":{},\"en_baseline\":{},\"en_saved\":{},\"ghosts_shown\":{},\"ghosts_good\":{},\"ghosts_judgeable\":{},\"good_midword\":{},\"vi_ghosts_shown\":{},\"vi_ghosts_good\":{},\"vi_actual\":{},\"en_ghosts_shown\":{},\"en_ghosts_good\":{},\"en_actual\":{},\"max_chain\":{},\"chained_accepts\":{},\"invariant_breaks\":{},\"diverged\":{},\"vi_sentences\":{},\"vi_rejected\":{},\"en_sentences\":{},\"en_rejected\":{},\"accepts\":{},\"engine_chars_saved\":{}}}",
            self.baseline_keys,
            self.actual_keys,
            self.saved_keys,
            self.savings_rate_pct,
            self.bench_baseline,
            self.bench_saved,
            self.vi_baseline,
            self.vi_saved,
            self.en_baseline,
            self.en_saved,
            self.ghosts_shown,
            self.ghosts_good,
            self.ghosts_judgeable,
            self.good_midword,
            self.vi_ghosts_shown,
            self.vi_ghosts_good,
            self.vi_actual,
            self.en_ghosts_shown,
            self.en_ghosts_good,
            self.en_actual,
            self.max_chain,
            self.chained_accepts,
            self.invariant_breaks,
            self.diverged,
            self.vi_sentences,
            self.vi_rejected,
            self.en_sentences,
            self.en_rejected,
            self.engine_accepts,
            self.engine_chars_saved
        )
    }
}

fn pct(a: u64, b: u64) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

/// Chạy toàn bộ corpus (bench ×3 + VI held-out + EN held-out) qua MỘT engine
/// (state cộng dồn, thứ tự cố định ⇒ tất định). `wrong_ghost` = đột biến test gate.
fn dream_rsi_oracle_run(wrong_ghost: bool) -> DreamRsiStat {
    let bench_corpus = build_bench_corpus();
    let (vi, _vi_lines, rej_vi) = build_heldout(HELDOUT_VI, true);
    let (en, _en_lines, rej_en) = build_heldout(HELDOUT_EN, false);

    let mut e = Engine::new(sim_opts());
    let before = e.dream_rsi_snapshot();
    let mut bench = OracleStat::default();
    let mut vi_stat = OracleStat::default();
    let mut en_stat = OracleStat::default();
    for s in &bench_corpus {
        bench.add(&sim_run_oracle(&mut e, s, wrong_ghost));
    }
    for s in &vi {
        vi_stat.add(&sim_run_oracle(&mut e, s, wrong_ghost));
    }
    for s in &en {
        en_stat.add(&sim_run_oracle(&mut e, s, wrong_ghost));
    }
    let after = e.dream_rsi_snapshot();
    let mut tot = OracleStat::default();
    tot.add(&bench);
    tot.add(&vi_stat);
    tot.add(&en_stat);
    DreamRsiStat {
        baseline_keys: tot.baseline_keys,
        actual_keys: tot.actual_keys,
        saved_keys: tot.baseline_keys.saturating_sub(tot.actual_keys),
        savings_rate_pct: (tot.ksr_pct()).round() as u64,
        bench_baseline: bench.baseline_keys,
        bench_saved: bench.baseline_keys.saturating_sub(bench.actual_keys),
        vi_baseline: vi_stat.baseline_keys,
        vi_saved: vi_stat.baseline_keys.saturating_sub(vi_stat.actual_keys),
        en_baseline: en_stat.baseline_keys,
        en_saved: en_stat.baseline_keys.saturating_sub(en_stat.actual_keys),
        ghosts_shown: tot.ghosts_shown,
        ghosts_good: tot.ghosts_good,
        ghosts_judgeable: tot.ghosts_judgeable,
        good_midword: tot.good_midword,
        vi_ghosts_shown: vi_stat.ghosts_shown,
        vi_ghosts_good: vi_stat.ghosts_good,
        vi_actual: vi_stat.actual_keys,
        en_ghosts_shown: en_stat.ghosts_shown,
        en_ghosts_good: en_stat.ghosts_good,
        en_actual: en_stat.actual_keys,
        max_chain: tot.max_chain,
        chained_accepts: tot.chained_accepts,
        invariant_breaks: tot.invariant_breaks,
        diverged: tot.diverged,
        vi_sentences: vi.len(),
        vi_rejected: rej_vi,
        en_sentences: en.len(),
        en_rejected: rej_en,
        engine_accepts: after.accepts - before.accepts,
        engine_chars_saved: after.chars_saved - before.chars_saved,
    }
}

/// Báo cáo người đọc (stdout `dream-rsi`): KSR per corpus + chất lượng chữ mờ.
fn dream_rsi_report(fresh: &DreamRsiStat) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "KSR-oracle tổng {:.1}% (tiết kiệm {}/{} phím) · bench {:.1}% · VI {:.1}% · EN {:.1}%",
        fresh.ksr_pct(),
        fresh.saved_keys,
        fresh.baseline_keys,
        pct(fresh.bench_saved, fresh.bench_baseline),
        pct(fresh.vi_saved, fresh.vi_baseline),
        pct(fresh.en_saved, fresh.en_baseline),
    );
    let _ = writeln!(
        s,
        "precision@ghost {:.1}% (xét được {:.1}%) · nhiễu {:.1}/100 phím · đúng-giữa-từ {} · chuỗi nối sâu nhất {} ({} nhận nối tiếp) · accepts {} · engine_chars_saved {}",
        fresh.precision_pct(),
        pct(fresh.ghosts_good, fresh.ghosts_judgeable),
        fresh.noise_per_100(),
        fresh.good_midword,
        fresh.max_chain,
        fresh.chained_accepts,
        fresh.engine_accepts,
        fresh.engine_chars_saved,
    );
    if fresh.invariant_breaks > 0 {
        let _ = writeln!(
            s,
            "VI PHẠM bất biến chữ mờ≠Tab chèn: {}",
            fresh.invariant_breaks
        );
    }
    if fresh.diverged > 0 {
        let _ = writeln!(s, "câu lệch đích: {}", fresh.diverged);
    }
    let vi_noise = fresh.vi_ghosts_shown.saturating_sub(fresh.vi_ghosts_good);
    let en_noise = fresh.en_ghosts_shown.saturating_sub(fresh.en_ghosts_good);
    let _ = writeln!(
        s,
        "nhiễu theo corpus: VI {:.1}/100 phím ({vi_noise} sai/{} hiện) · EN {:.1}/100 ({en_noise}/{} hiện)",
        pct(vi_noise, fresh.vi_actual),
        fresh.vi_ghosts_shown,
        pct(en_noise, fresh.en_actual),
        fresh.en_ghosts_shown,
    );
    let _ = writeln!(
        s,
        "corpus held-out: VI {} câu (bỏ {}) · EN {} câu (bỏ {}) — từ EN mâu thuẫn Việt (\"this/of/if/last\", R2-56) không tái lập được là loại dự kiến",
        fresh.vi_sentences,
        fresh.vi_rejected,
        fresh.en_sentences,
        fresh.en_rejected
    );
    s
}

/// G19c — Dream RSI hồi quy theo **KSR-oracle** (RSI-1): tiết kiệm phím giả lập
/// người dùng chỉ nhận chữ mờ ĐÚNG giảm so baseline = fail; NHIỄU (chữ mờ sai
/// hiện ra) TĂNG so baseline = fail — đột biến "luôn hiện chữ mờ sai" ⇒ KSR
/// không tăng và noise tăng ⇒ gate đỏ. VI/EN riêng lẻ không được giảm (RSI-7).
/// Bất biến chữ mờ≠Tab (R2-173) và lệch đích phải = 0. Khối `dream_rsi` là
/// deterministic (không timing) nên so tuyệt đối được; baseline cũ thiếu khối
/// → bỏ qua (chỉ ghi chú).
fn check_dream_rsi(
    baseline_json: &str,
    fresh: &DreamRsiStat,
    lines: &mut Vec<String>,
    fails: &mut Vec<String>,
) {
    let Some(pos) = baseline_json.find("\"dream_rsi\"") else {
        lines.push(
            "  dream-rsi: baseline chưa có khối — re-record baseline để bật gate G19c".into(),
        );
        return;
    };
    let tail = &baseline_json[pos..];
    let parse = |key: &str| -> Option<u64> {
        let p = tail.find(key)?;
        let num: String = tail[p + key.len()..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        num.parse().ok()
    };
    let old_saved = parse("\"saved_keys\":");
    let old_vi = parse("\"vi_saved\":");
    let old_en = parse("\"en_saved\":");
    let old_accepts = parse("\"accepts\":");
    let old_noise = parse("\"ghosts_shown\":").zip(parse("\"ghosts_good\":"));
    let old_invariant = parse("\"invariant_breaks\":");
    lines.push(format!(
        "  {:<22} KSR-oracle {}/{} phím ({:.1}%) · VI {}/{} · EN {}/{} · nhiễu {:.1}/100 · chuỗi nối {} · accepts {} (G19c)",
        "dream-rsi",
        fresh.saved_keys,
        fresh.baseline_keys,
        fresh.ksr_pct(),
        fresh.vi_saved,
        fresh.vi_baseline,
        fresh.en_saved,
        fresh.en_baseline,
        fresh.noise_per_100(),
        fresh.max_chain,
        fresh.engine_accepts
    ));
    if let Some(old) = old_saved {
        if fresh.saved_keys < old {
            fails.push(format!(
                "dream-rsi: KSR-oracle {old} → {} phím tiết kiệm — G19c: tính năng làm gợi ý kém hơn, không được merge",
                fresh.saved_keys
            ));
        }
    }
    if let Some(old) = old_vi {
        if fresh.vi_saved < old {
            fails.push(format!(
                "dream-rsi: KSR-oracle VI {old} → {} phím — cổng/tính năng làm VI kém hơn (RSI-7), không được merge",
                fresh.vi_saved
            ));
        }
    }
    if let Some(old) = old_en {
        if fresh.en_saved < old {
            fails.push(format!(
                "dream-rsi: KSR-oracle EN {old} → {} phím — cổng/tính năng làm EN kém hơn (RSI-7), không được merge",
                fresh.en_saved
            ));
        }
    }
    if let Some((old_shown, old_good)) = old_noise {
        let old = old_shown.saturating_sub(old_good);
        if fresh.noise_stat() > old {
            fails.push(format!(
                "dream-rsi: nhiễu {old} → {} chữ mờ sai/100 vị trí — chữ mờ sai nhiều hơn baseline (đột biến \"luôn hiện chữ mờ sai\"), không được merge",
                fresh.noise_stat()
            ));
        }
    }
    if let Some(old) = old_accepts {
        if fresh.engine_accepts < old {
            fails.push(format!(
                "dream-rsi: accepts {old} → {} — ít accept hơn baseline = gợi ý tệ hơn",
                fresh.engine_accepts
            ));
        }
    }
    if let Some(old) = old_invariant {
        if fresh.invariant_breaks > old {
            fails.push(format!(
                "dream-rsi: bất biến chữ mờ≠Tab chèn vi phạm {} lần (R2-173) — lỗi engine, không được merge",
                fresh.invariant_breaks
            ));
        }
    }
    if fresh.diverged > 0 {
        fails.push(format!(
            "dream-rsi: {} câu lệch đích — mô phỏng không tái lập được văn bản (oracle chỉ nhận chữ đúng ⇒ không được lệch), không được merge",
            fresh.diverged
        ));
    }
}

/// Đo N vòng, giữ vòng có `p50` nhỏ nhất.
fn best_of<F: FnMut() -> Measurement>(mut run: F) -> Measurement {
    let mut best: Option<Measurement> = None;
    for _ in 0..ROUNDS {
        let m = run();
        best = Some(match best {
            Some(b) if b.p50_ns <= m.p50_ns => b,
            _ => m,
        });
    }
    best.expect("ROUNDS >= 1")
}

fn main() -> std::process::ExitCode {
    use std::process::ExitCode;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("run");
    let path = args.get(1).cloned();
    // `--iters N` (mặc định 20 000 — đủ chính xác, chạy ~1 giây)
    let iters: usize = args
        .iter()
        .position(|a| a == "--iters")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);

    if cmd == "dream-rsi" {
        // Standalone: in số Dream RSI KSR-oracle (G19c, RSI-1) không cần đo timing.
        let dream = dream_rsi_oracle_run(false);
        println!("{}", dream.to_json());
        print!("{}", dream_rsi_report(&dream));
        return ExitCode::SUCCESS;
    }

    let all = if cmd == "field-switch" {
        vec![best_of(|| bench_field_switch_resolve(iters))]
    } else {
        vec![
            best_of(|| bench_ime_key(iters)),
            best_of(|| bench_parse_config(iters.clamp(100, 5_000))),
            best_of(|| bench_resolve(iters)),
        ]
    };
    let dream = dream_rsi_oracle_run(false);

    match cmd {
        "run" | "field-switch" => {
            let output = render(&all, true, &dream);
            if all
                .iter()
                .any(|m| m.budget_ns() > 0 && m.p99_ns > m.budget_ns())
            {
                eprintln!("field/strategy resolver vượt ngân sách p99");
                return ExitCode::FAILURE;
            }
            println!("{}", output.trim_end());
        }
        "write" => {
            let p = match path.as_deref() {
                None | Some("--auto") => {
                    format!("perf/baseline-{}.json", host_target_triple())
                }
                Some(explicit) => explicit.to_string(),
            };
            if let Some(dir) = std::path::Path::new(&p).parent() {
                if !dir.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(dir);
                }
            }
            if let Err(e) = std::fs::write(&p, render(&all, true, &dream)) {
                eprintln!("không ghi được `{p}`: {e}");
                return ExitCode::FAILURE;
            }
            println!("đã ghi baseline: {p}");
        }
        "check" => {
            let resolved = match path.as_deref() {
                None | Some("--auto") => {
                    resolve_baseline_path(std::path::Path::new("perf"), host_target_triple())
                }
                Some(explicit) => std::path::PathBuf::from(explicit),
            };
            let p = resolved.to_string_lossy().into_owned();
            let Ok(old) = std::fs::read_to_string(&resolved) else {
                eprintln!("không đọc được baseline `{p}` — chạy `write` trước");
                return ExitCode::FAILURE;
            };
            println!("baseline: {p}");
            match compare(&old, &all, &dream) {
                Ok(text) => println!("{text}"),
                Err(fails) => {
                    eprintln!("REGRESSION >{REGRESSION_PCT}%:");
                    for f in &fails {
                        eprintln!("  {f}");
                    }
                    return ExitCode::FAILURE;
                }
            }
        }
        other => {
            eprintln!("lệnh lạ `{other}` — dùng: run | write [<file>|--auto] | check [<file>|--auto] | dream-rsi [--iters N]");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// p99 = 2 × p50 (quan hệ đúng thực tế) — `compare` so `p50` nên truyền p50.
    fn m(name: &'static str, p50: u128) -> Measurement {
        Measurement {
            name,
            samples: 10,
            p50_ns: p50,
            p99_ns: p50 * 2,
            mean_ns: p50,
        }
    }

    #[test]
    fn compare_bat_hoi_quy_qua_nguong() {
        let base = r#"{"name":"ime_key","p50_ns":1000,"p99_ns":2000,"mean_ns":1100}"#;
        // +5% → pass
        assert!(compare(base, &[m("ime_key", 1050)], &dream(21, 33)).is_ok());
        // +30% → fail
        let err = compare(base, &[m("ime_key", 1300)], &dream(21, 33)).unwrap_err();
        assert!(err.iter().any(|e| e.contains("1300")), "{err:?}");
        // nhanh hơn 30% → vẫn pass (hồi quy là một chiều)
        assert!(compare(base, &[m("ime_key", 700)], &dream(21, 33)).is_ok());
    }

    #[test]
    fn compare_bat_vuot_ngan_sach() {
        // baseline rất nhanh (không lệch %) nhưng p99 vẫn vượt ngân sách 0.5 ms
        let base = r#"{"name":"ime_key","p50_ns":10,"p99_ns":12,"mean_ns":11}"#;
        let err = compare(base, &[m("ime_key", 600_000)], &dream(21, 33)).unwrap_err();
        assert!(err.iter().any(|e| e.contains("VƯỢT ngân sách")), "{err:?}");
    }

    #[test]
    fn best_of_giu_vong_nhanh_nhat() {
        let mut call = 0;
        let p50s = [900u128, 500, 700];
        let m = best_of(|| {
            let p = p50s[call % p50s.len()];
            call += 1;
            Measurement {
                name: "x",
                samples: 1,
                p50_ns: p,
                p99_ns: p,
                mean_ns: p,
            }
        });
        assert_eq!(call, 3, "phải đo đúng ROUNDS vòng");
        assert_eq!(m.p50_ns, 500, "phải giữ vòng nhanh nhất");
    }

    #[test]
    fn compare_bao_thieu_baseline() {
        let err = compare("{}", &[m("ime_key", 10)], &dream(21, 33)).unwrap_err();
        assert!(
            err.iter().any(|e| e.contains("không có trong baseline")),
            "{err:?}"
        );
    }

    fn dream(rate: u64, accepts: u64) -> DreamRsiStat {
        DreamRsiStat {
            baseline_keys: 318,
            actual_keys: 318 - rate * 318 / 100,
            saved_keys: rate * 318 / 100,
            savings_rate_pct: rate,
            bench_baseline: 100,
            bench_saved: rate,
            vi_baseline: 100,
            vi_saved: rate,
            en_baseline: 100,
            en_saved: rate,
            ghosts_shown: 100,
            ghosts_good: rate * 4,
            ghosts_judgeable: 90,
            good_midword: 2,
            vi_ghosts_shown: 40,
            vi_ghosts_good: rate * 2,
            vi_actual: 79,
            en_ghosts_shown: 60,
            en_ghosts_good: rate * 2,
            en_actual: 79,
            max_chain: 3,
            chained_accepts: 4,
            invariant_breaks: 0,
            diverged: 0,
            vi_sentences: 40,
            vi_rejected: 0,
            en_sentences: 15,
            en_rejected: 5,
            engine_accepts: accepts,
            engine_chars_saved: rate * 318 / 100,
        }
    }

    /// G19c (KSR-oracle, RSI-1): tiết kiệm giảm → fail; nhiễu tăng → fail;
    /// VI/EN giảm → fail; tăng/giữ nguyên → pass; baseline thiếu khối → bỏ qua.
    #[test]
    fn dream_rsi_check_hoi_quy() {
        let base = r#"{"dream_rsi":{"baseline_keys":318,"actual_keys":252,"saved_keys":66,"savings_rate_pct":21,"bench_baseline":100,"bench_saved":21,"vi_baseline":100,"vi_saved":21,"en_baseline":100,"en_saved":21,"ghosts_shown":100,"ghosts_good":84,"ghosts_judgeable":90,"good_midword":2,"max_chain":3,"chained_accepts":4,"invariant_breaks":0,"diverged":0,"accepts":33,"engine_chars_saved":57}}"#;

        // Giữ nguyên mọi chỉ số → pass.
        let mut lines = Vec::new();
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(21, 33), &mut lines, &mut fails);
        assert!(fails.is_empty(), "{fails:?}");

        // Tiết kiệm 69 → 50 phím → fail G19c (KSR-oracle giảm).
        let mut d = dream(21, 33);
        d.saved_keys = 50;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(
            fails
                .iter()
                .any(|f| f.contains("KSR-oracle") && f.contains("không được merge")),
            "{fails:?}"
        );

        // VI giảm 21 → 10 → fail (RSI-7).
        let mut d = dream(21, 33);
        d.vi_saved = 10;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("VI")), "{fails:?}");

        // EN giảm 21 → 10 → fail (RSI-7).
        let mut d = dream(21, 33);
        d.en_saved = 10;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("EN")), "{fails:?}");

        // Nhiễu tăng: ghosts_shown 100→120 (good giữ) → fail ("luôn hiện chữ mờ sai").
        let mut d = dream(21, 33);
        d.ghosts_shown = 120;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(
            fails
                .iter()
                .any(|f| f.contains("nhiễu") && f.contains("chữ mờ sai")),
            "{fails:?}"
        );

        // Accepts giảm 33 → 20 → fail.
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(21, 20), &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("accepts")), "{fails:?}");

        // Bất biến chữ mờ≠Tab vi phạm → fail (R2-173).
        let mut d = dream(21, 33);
        d.invariant_breaks = 2;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("R2-173")), "{fails:?}");

        // Lệch đích → fail.
        let mut d = dream(21, 33);
        d.diverged = 1;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("lệch đích")), "{fails:?}");

        // Tiến hóa: tiết kiệm tăng + nhiễu giảm → pass.
        let mut d = dream(25, 40);
        d.ghosts_shown = 90;
        let mut fails = Vec::new();
        check_dream_rsi(base, &d, &mut lines, &mut fails);
        assert!(fails.is_empty(), "{fails:?}");

        // Baseline cũ không có khối dream_rsi → bỏ qua, không fail.
        let mut fails = Vec::new();
        let mut lines2 = Vec::new();
        check_dream_rsi("{}", &dream(1, 1), &mut lines2, &mut fails);
        assert!(fails.is_empty());
        assert!(lines2.iter().any(|l| l.contains("re-record")));
    }

    /// ĐỘT BIẾN RSI-1: "luôn hiện chữ mờ sai" — KSR không tăng (rơi ~0) và noise
    /// tăng vọt ⇒ gate G19c PHẢI ĐỎ ở cả 2 tiêu chí. Chạy mô phỏng thật trên
    /// corpus thật (không fake số) — harness phải bắt được đột biến này.
    #[test]
    fn dream_rsi_gate_duot_bien_ghost_sai() {
        // to_json() chỉ là KHỐI trong — bọc `"dream_rsi"` như khối baseline thật.
        let base_block = format!(
            "{{\"dream_rsi\":{}}}",
            dream_rsi_oracle_run(false).to_json()
        );
        let base = dream_rsi_oracle_run(false);
        let mut lines = Vec::new();
        let mut fails = Vec::new();
        check_dream_rsi(&base_block, &base, &mut lines, &mut fails);
        assert!(
            fails.is_empty(),
            "baseline tự so với chính nó phải pass: {fails:?}"
        );

        let mutd = dream_rsi_oracle_run(true);
        assert!(
            mutd.saved_keys < base.saved_keys,
            "đột biến ghost sai phải làm tiết kiệm giảm ({} < {})",
            mutd.saved_keys,
            base.saved_keys
        );
        assert!(
            mutd.noise_stat() > base.noise_stat(),
            "đột biến ghost sai phải làm nhiễu tăng ({} > {})",
            mutd.noise_stat(),
            base.noise_stat()
        );
        let mut fails = Vec::new();
        check_dream_rsi(&base_block, &mutd, &mut lines, &mut fails);
        assert!(
            fails
                .iter()
                .any(|f| f.contains("KSR-oracle") && f.contains("không được merge")),
            "gate phải bắt KSR giảm: {fails:?}"
        );
        assert!(
            fails
                .iter()
                .any(|f| f.contains("nhiễu") && f.contains("chữ mờ sai")),
            "gate phải bắt noise tăng: {fails:?}"
        );
    }

    /// Người dùng oracle chỉ nhận chữ mờ đúng ⇒ mô phỏng không được lệch đích và
    /// không được vi phạm bất biến "chữ mờ == Tab chèn" (R2-173) trên corpus thật.
    #[test]
    fn dream_rsi_sim_khong_lech_dich_va_giu_bat_bien() {
        let a = dream_rsi_oracle_run(false);
        assert_eq!(
            a.diverged, 0,
            "oracle chỉ nhận chữ đúng — không được lệch đích"
        );
        assert_eq!(
            a.invariant_breaks, 0,
            "Tab phải chèn đúng văn bản chữ mờ đang hiển thị (R2-173)"
        );
        assert!(a.engine_accepts > 0, "corpus phải sinh được accepts");
        assert!(a.saved_keys > 0, "corpus phải tiết kiệm được phím");
        assert!(
            a.ghosts_shown > 0,
            "corpus phải có chữ mờ hiện ra để đo precision"
        );
        assert!(
            a.ghosts_good > 0,
            "phải có chữ mờ đúng trên corpus văn phòng"
        );
        assert!(
            a.vi_baseline > 0 && a.en_baseline > 0,
            "cả VI lẫn EN phải được đo"
        );
    }

    /// Corpus bench_dream_rsi phải DETERMINISTIC (không timing): 2 lần chạy
    /// liên tiếp ra cùng số — điều kiện để gate G19c so tuyệt đối.
    #[test]
    fn dream_rsi_sim_deterministic() {
        let a = dream_rsi_oracle_run(false);
        let b = dream_rsi_oracle_run(false);
        assert_eq!(a.baseline_keys, b.baseline_keys);
        assert_eq!(a.actual_keys, b.actual_keys);
        assert_eq!(a.saved_keys, b.saved_keys);
        assert_eq!(a.vi_saved, b.vi_saved);
        assert_eq!(a.en_saved, b.en_saved);
        assert_eq!(a.ghosts_shown, b.ghosts_shown);
        assert_eq!(a.engine_accepts, b.engine_accepts);
        assert_eq!(a.max_chain, b.max_chain);
        assert_eq!(a.savings_rate_pct, b.savings_rate_pct);
    }

    /// JSON của dream_rsi parse được như khối baseline thật (bench string-find):
    /// mọi key gate tra cứu phải xuất hiện đúng dạng `"key":`.
    #[test]
    fn dream_rsi_json_co_dinh_dang() {
        let s = dream(21, 33).to_json();
        assert!(s.starts_with("{\"baseline_keys\":"));
        assert!(s.contains("\"savings_rate_pct\":21"));
        assert!(s.contains("\"accepts\":33"));
        assert!(s.contains("\"vi_saved\":"));
        assert!(s.contains("\"en_saved\":"));
        assert!(s.contains("\"ghosts_shown\":100"));
        assert!(s.contains("\"invariant_breaks\":0"));
        assert!(s.ends_with('}'));
    }

    #[test]
    fn perf1_target_triple_map_du_6_target_va_chon_baseline() {
        assert_eq!(
            target_triple_for("x86_64", "windows"),
            "x86_64-pc-windows-msvc"
        );
        assert_eq!(
            target_triple_for("aarch64", "windows"),
            "aarch64-pc-windows-msvc"
        );
        assert_eq!(
            target_triple_for("x86_64", "linux"),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            target_triple_for("aarch64", "linux"),
            "aarch64-unknown-linux-gnu"
        );
        assert_eq!(
            target_triple_for("aarch64", "macos"),
            "aarch64-apple-darwin"
        );
        assert_eq!(target_triple_for("x86_64", "macos"), "x86_64-apple-darwin");

        let dir = std::env::temp_dir().join(format!("textvn-perf1-bench-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let arm_file = dir.join("baseline-aarch64-unknown-linux-gnu.json");
        std::fs::write(&arm_file, "{}").unwrap();
        assert_eq!(
            resolve_baseline_path(&dir, "aarch64-unknown-linux-gnu"),
            arm_file
        );
        assert_eq!(
            resolve_baseline_path(&dir, "riscv64gc-unknown-linux-gnu"),
            dir.join("baseline-win.json")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
