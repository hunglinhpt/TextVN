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
        _reserved: 0,
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

fn render(all: &[Measurement], with_platform: bool, dream: &DreamRsiStat) -> String {
    let mut s = String::from("{\n  \"schema\": \"textvn-bench.v1\",\n");
    if with_platform {
        let _ = writeln!(s, "  \"platform\": \"{}\",", platform());
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
fn compare(baseline_json: &str, fresh: &[Measurement]) -> Result<String, Vec<String>> {
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
    // P10 Dream RSI (G19c): savings-rate/accepts giảm so baseline = fail.
    check_dream_rsi(baseline_json, &bench_dream_rsi(), &mut lines, &mut fails);
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

// --- P10 Dream RSI (G19c): savings-rate trên corpus văn phòng — khối DETERMINISTIC ---

/// Corpus mô phỏng gõ văn phòng: VN Telex (transform thật) + EN từ điển phổ thông.
/// KHÔNG phụ thuộc máy (không timing) — cùng số trên 3 OS ⇒ so baseline được tuyệt đối.
const RSI_WORDS: [&str; 16] = [
    // VN gõ Telex (transform thật → engine học + ghost)
    "duocj", "tooi", "lamf", "vaof", "khoong", "ddeer", "vangf", "moocj",
    // EN (completion qua en_words10k / freq3000)
    "release", "report", "team", "project", "client", "contract", "review", "meeting",
];

/// Một phép đo Dream RSI trên corpus.
struct DreamRsiStat {
    /// Số phím MÃ user phải gõ KHÔNG có prediction (tổng ký tự + space/từ).
    baseline_keys: u64,
    /// Số phím thật khi có prediction (mô phỏng user: gõ nửa từ → Tab nếu có ghost,
    /// không thì gõ hết; space kết từ).
    actual_keys: u64,
    /// baseline − actual: số phím tiết kiệm được.
    saved_keys: u64,
    savings_rate_pct: u64,
    /// Counters từ engine (đối chiếu chéo — saved_keys đo theo phím gõ thô, engine_chars_saved đo theo ký tự/glyph).
    engine_accepts: u64,
    engine_chars_saved: u64,
}

impl DreamRsiStat {
    fn to_json(&self) -> String {
        format!(
            "{{\"baseline_keys\":{},\"actual_keys\":{},\"saved_keys\":{},\"savings_rate_pct\":{},\"accepts\":{},\"engine_chars_saved\":{}}}",
            self.baseline_keys,
            self.actual_keys,
            self.saved_keys,
            self.savings_rate_pct,
            self.engine_accepts,
            self.engine_chars_saved
        )
    }
}

/// Mô phỏng user gõ corpus qua engine thật: gõ nửa từ → có ghost thì Tab nhận,
/// không thì gõ nốt; space kết từ. Đếm phím thật so với baseline không-prediction.
fn bench_dream_rsi() -> DreamRsiStat {
    use textvn_core::keymap::{self, KeyEvent};
    use textvn_core::{Engine, EngineOptions, Method};

    let mut e = Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        auto_restore_english: true,
        ..Default::default()
    });
    let mut actual = 0u64;
    for _ in 0..3 {
        for w in RSI_WORDS {
            let chars: Vec<char> = w.chars().collect();
            let half = (chars.len() / 2).max(1);
            for &c in &chars[..half] {
                let _ = e.key(&KeyEvent::char_down(c));
                actual += 1;
            }
            if e.peek_completion().is_some() {
                let _ = e.key(&KeyEvent {
                    vk: keymap::vk::TAB,
                    ch: '\t' as u32,
                    key_down: true,
                    ..Default::default()
                });
                actual += 1;
            } else {
                for &c in &chars[half..] {
                    let _ = e.key(&KeyEvent::char_down(c));
                    actual += 1;
                }
            }
            let _ = e.key(&KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..Default::default()
            });
            actual += 1;
        }
        let _ = e.key(&KeyEvent {
            vk: keymap::vk::RETURN,
            ch: '\n' as u32,
            key_down: true,
            ..Default::default()
        });
        actual += 1;
    }
    let baseline_keys: u64 = 3 * RSI_WORDS
        .iter()
        .map(|w| w.chars().count() as u64 + 1)
        .sum::<u64>()
        + 3; // 3 lần Enter kết dòng
    let s = e.dream_rsi_snapshot();
    let saved = baseline_keys.saturating_sub(actual);
    DreamRsiStat {
        baseline_keys,
        actual_keys: actual,
        saved_keys: saved,
        savings_rate_pct: saved * 100 / baseline_keys.max(1),
        engine_accepts: s.accepts,
        engine_chars_saved: s.chars_saved,
    }
}

/// G19c — Dream RSI hồi quy: savings-rate/accepts giảm so baseline = tính năng
/// làm gợi ý kém hơn → fail. Khối `dream_rsi` là deterministic (không timing)
/// nên so tuyệt đối được; baseline cũ thiếu khối → bỏ qua (chỉ ghi chú).
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
    let old_rate = parse("\"savings_rate_pct\":");
    let old_accepts = parse("\"accepts\":");
    lines.push(format!(
        "  {:<22} saved {}/{} phím · rate {}% · accepts {} (G19c)",
        "dream-rsi",
        fresh.saved_keys,
        fresh.baseline_keys,
        fresh.savings_rate_pct,
        fresh.engine_accepts
    ));
    if let Some(old) = old_rate {
        if fresh.savings_rate_pct < old {
            fails.push(format!(
                "dream-rsi: savings-rate {old}% → {}% — G19c: tính năng làm gợi ý kém hơn, không được merge",
                fresh.savings_rate_pct
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
        // Standalone: in số Dream RSI corpus (G19c) không cần đo timing.
        let dream = bench_dream_rsi();
        println!("{}", dream.to_json());
        println!(
            "saved {}/{} phím · savings-rate {}% · accepts {} · engine_chars_saved {}",
            dream.saved_keys,
            dream.baseline_keys,
            dream.savings_rate_pct,
            dream.engine_accepts,
            dream.engine_chars_saved
        );
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
    let dream = bench_dream_rsi();

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
            let Some(p) = path else {
                eprintln!("usage: textvn-bench write <file.json>");
                return ExitCode::FAILURE;
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
            let Some(p) = path else {
                eprintln!("usage: textvn-bench check <file.json>");
                return ExitCode::FAILURE;
            };
            let Ok(old) = std::fs::read_to_string(&p) else {
                eprintln!("không đọc được baseline `{p}` — chạy `write` trước");
                return ExitCode::FAILURE;
            };
            match compare(&old, &all) {
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
            eprintln!("lệnh lạ `{other}` — dùng: run | write <file> | check <file> | dream-rsi [--iters N]");
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
        assert!(compare(base, &[m("ime_key", 1050)]).is_ok());
        // +30% → fail
        let err = compare(base, &[m("ime_key", 1300)]).unwrap_err();
        assert!(err.iter().any(|e| e.contains("1300")), "{err:?}");
        // nhanh hơn 30% → vẫn pass (hồi quy là một chiều)
        assert!(compare(base, &[m("ime_key", 700)]).is_ok());
    }

    #[test]
    fn compare_bat_vuot_ngan_sach() {
        // baseline rất nhanh (không lệch %) nhưng p99 vẫn vượt ngân sách 0.5 ms
        let base = r#"{"name":"ime_key","p50_ns":10,"p99_ns":12,"mean_ns":11}"#;
        let err = compare(base, &[m("ime_key", 600_000)]).unwrap_err();
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
        let err = compare("{}", &[m("ime_key", 10)]).unwrap_err();
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
            engine_accepts: accepts,
            engine_chars_saved: rate * 318 / 100,
        }
    }

    /// G19c: savings-rate giảm → fail; tăng/giữ nguyên → pass; baseline thiếu
    /// khối → bỏ qua (chỉ ghi chú re-record).
    #[test]
    fn dream_rsi_check_hoi_quy() {
        let base = r#"{"dream_rsi":{"baseline_keys":318,"actual_keys":249,"saved_keys":69,"savings_rate_pct":21,"accepts":33,"engine_chars_saved":57}}"#;

        // Giữ nguyên rate + accepts → pass.
        let mut lines = Vec::new();
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(21, 33), &mut lines, &mut fails);
        assert!(fails.is_empty(), "{fails:?}");

        // Rate giảm 21 → 18 → fail G19c.
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(18, 33), &mut lines, &mut fails);
        assert!(
            fails
                .iter()
                .any(|f| f.contains("savings-rate") && f.contains("không được merge")),
            "{fails:?}"
        );

        // Accepts giảm 33 → 20 → fail.
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(21, 20), &mut lines, &mut fails);
        assert!(fails.iter().any(|f| f.contains("accepts")), "{fails:?}");

        // Rate tăng (tiến hóa) → pass.
        let mut fails = Vec::new();
        check_dream_rsi(base, &dream(25, 40), &mut lines, &mut fails);
        assert!(fails.is_empty(), "{fails:?}");

        // Baseline cũ không có khối dream_rsi → bỏ qua, không fail.
        let mut fails = Vec::new();
        let mut lines2 = Vec::new();
        check_dream_rsi("{}", &dream(1, 1), &mut lines2, &mut fails);
        assert!(fails.is_empty());
        assert!(lines2.iter().any(|l| l.contains("re-record")));
    }

    /// Corpus bench_dream_rsi phải DETERMINISTIC (không timing): 2 lần chạy
    /// liên tiếp ra cùng số — điều kiện để gate G19c so tuyệt đối.
    #[test]
    fn dream_rsi_sim_deterministic() {
        let a = bench_dream_rsi();
        let b = bench_dream_rsi();
        assert_eq!(a.baseline_keys, b.baseline_keys);
        assert_eq!(a.actual_keys, b.actual_keys);
        assert_eq!(a.savings_rate_pct, b.savings_rate_pct);
        assert_eq!(a.engine_accepts, b.engine_accepts);
        assert!(a.engine_accepts > 0, "corpus phải sinh được accepts");
        assert!(a.saved_keys > 0, "corpus phải tiết kiệm được phím");
    }

    /// JSON của dream_rsi parse được như khối baseline thật (bench string-find).
    #[test]
    fn dream_rsi_json_co_dinh_dang() {
        let s = dream(21, 33).to_json();
        assert!(s.starts_with("{\"baseline_keys\":"));
        assert!(s.contains("\"savings_rate_pct\":21"));
        assert!(s.contains("\"accepts\":33"));
        assert!(s.ends_with('}'));
    }
}
