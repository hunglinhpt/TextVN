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

fn render(all: &[Measurement], with_platform: bool) -> String {
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
    s.push_str("  ]\n}\n");
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

    let all = if cmd == "field-switch" {
        vec![best_of(|| bench_field_switch_resolve(iters))]
    } else {
        vec![
            best_of(|| bench_ime_key(iters)),
            best_of(|| bench_parse_config(iters.clamp(100, 5_000))),
            best_of(|| bench_resolve(iters)),
        ]
    };

    match cmd {
        "run" | "field-switch" => {
            let output = render(&all, true);
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
            if let Err(e) = std::fs::write(&p, render(&all, true)) {
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
            eprintln!("lệnh lạ `{other}` — dùng: run | write <file> | check <file> [--iters N]");
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
}
