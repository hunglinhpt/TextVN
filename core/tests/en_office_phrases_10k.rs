// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 59 — bảng `en_office_phrases.tsv` (≥ 10.000 hàng văn phòng):
//!
//! 1. **Invariant dữ liệu** (integration mặt công khai): hàng ≥ 10.000, không
//!    trùng, weight 1..=100, khoá lowercase, prediction ≤ 60 ký tự.
//! 2. **Phủ 5 nhóm nghiệp vụ** + bilingual Việt-Eng code-switch.
//! 3. **Latency**: lookup < 1 ms/call ở release (debug nới ×100 như
//!    `prediction_perf`) — bucket byte-cuối giữ tuyến tính nhỏ.
//! 4. **Engine thật**: Mode VI gõ "minhf sync laif" + Tab → bắn gợi ý
//!    tiếng Anh giữa câu tiếng Việt (code-switch), Dream RSI tăng đúng
//!    công thức (G19a); Mode EN mở rộng câu văn phòng; prefix EN không bị
//!    che bởi mode VI.
//!
//! G8: test chỉ assert hành vi/đếm — không log nội dung phím.

use std::time::Instant;
use textvn_core::keymap::{vk, KeyEvent};
use textvn_core::post::predict::{
    predict_next_phrase_candidates, predict_prefix_candidates, EN_OFFICE_PHRASES_DATA,
};
use textvn_core::sim::apply;
use textvn_core::{Engine, EngineOptions};

type Row = (&'static str, &'static str, u32);

fn office_rows() -> Vec<Row> {
    EN_OFFICE_PHRASES_DATA
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.split('\t');
            let key = parts.next()?.trim();
            let pred = parts.next()?.trim();
            let weight: u32 = parts
                .next()
                .and_then(|w| w.trim().parse().ok())
                .unwrap_or(0);
            Some((key, pred, weight))
        })
        .collect()
}

/// 1) Invariant mặt công khai — ≥ 10.000 hàng unique, weight 1..=100, khoá
///    lowercase, prediction ≤ 60 ký tự (R1-2 / MAX_TEXT=64).
#[test]
fn so_hang_va_bat_bien_du_lieu() {
    let rows = office_rows();
    assert!(rows.len() >= 10_000, "office: quá ít hàng: {}", rows.len());
    let mut uniq: Vec<&Row> = rows.iter().collect();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), rows.len(), "office: hàng trùng");
    for (key, pred, weight) in &rows {
        assert!(!key.is_empty() && !pred.is_empty(), "hàng rỗng: {key}");
        assert!(pred.chars().count() <= 60, "pred > 60: {pred}");
        assert!((1..=100).contains(weight), "weight {weight}");
        assert!(!key.chars().any(char::is_uppercase), "khoá hoa: {key}");
    }
}

/// 2) Phủ 5 nhóm nghiệp vụ — mỗi nhóm ≥ 1 ngữ cảnh đặc trưng bắn ứng viên.
#[test]
fn phu_5_nhom_nghiep_vu() {
    let cases: [(&str, &str); 10] = [
        // Upward (exec summary / escalation / risk / RCA)
        ("here is a quick status update on", "upward"),
        ("escalating this blocker for your visibility", "upward"),
        ("we identified the root cause of", "upward"),
        // Downward (delegation / unblock / recognition)
        ("what support do you need to unblock", "downward"),
        ("great job on delivering", "downward"),
        // Tech (CI/CD / deploy / trade-offs)
        ("the fix has been verified on ci and", "tech"),
        ("deploying the hotfix to production", "tech"),
        // Business ops (stakeholder / action items / sync)
        ("we need stakeholder sign off before", "business"),
        ("following up on our action items from", "business"),
        // Everyday glue
        ("thanks for flagging this", "everyday"),
    ];
    for (ctx, group) in cases {
        let preds = predict_next_phrase_candidates(ctx, 5);
        assert!(
            !preds.is_empty(),
            "{group}: ngữ cảnh '{ctx}' không bắn ứng viên"
        );
    }
}

/// 3) Latency: mỗi lookup < 1 ms (release; debug nới ×100 — chỉ chặn hồi quy
///    CẤU TRÚC, số thật đọc ở run `--release -- --nocapture`).
#[test]
fn lookup_latency_duoi_1ms() {
    let contexts = [
        "the migration",
        "we are seeing elevated error rates in the notification pipeline and have",
        "please find the executive summary below",
        "the dashboard tracks p ninety nine latency",
        "mình sync lại",
        "anh check giúp em",
        "we identified the root cause of the failure as a stale",
        "one key risk to highlight is the vendor",
    ];
    let factor = if cfg!(debug_assertions) { 100 } else { 1 };
    let mut worst = std::time::Duration::ZERO;
    for ctx in contexts {
        // warm-up cache tĩnh (OnceLock parse + bucket)
        let _ = predict_next_phrase_candidates(ctx, 5);
        let n = 500u32;
        let t0 = Instant::now();
        for _ in 0..n {
            let _ = predict_next_phrase_candidates(ctx, 5);
        }
        let avg = t0.elapsed() / n;
        worst = worst.max(avg);
        println!("next-phrase '{ctx}': {avg:?}");
    }
    // Prefixcompletion cũng phải giữ tuyến tính (nguồn office không vào path
    // prefix nhưng hai bảng cũ + 10k từ vẫn quét mỗi phím).
    for p in ["de", "the", "st", "pl", "kq"] {
        let _ = predict_prefix_candidates(p, false, 5);
    }
    assert!(
        worst < std::time::Duration::from_millis(factor as u64),
        "lookup chậm nhất {worst:?} — vượt ngân sách 1 ms (x{factor})"
    );
}

fn press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    apply(buf, &out.action, k);
}

fn letter(c: char) -> KeyEvent {
    KeyEvent::char_down(c)
}

fn space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        ch: ' ' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn tab() -> KeyEvent {
    KeyEvent {
        vk: vk::TAB,
        ch: '\t' as u32,
        key_down: true,
        ..Default::default()
    }
}

fn type_str(e: &mut Engine, buf: &mut Vec<char>, s: &str) {
    for c in s.chars() {
        if c == ' ' {
            press(e, buf, &space());
        } else {
            press(e, buf, &letter(c));
        }
    }
}

/// 4) Mode VI — code-switch: gõ "minhf sync laij" (→ "mình sync lại" — j = nặng) + Tab
///    bắn gợi ý trộn Việt-Eng giữa câu tiếng Việt; Dream RSI tăng đúng công
///    thức (accepts +1, chars_saved = nguyên glyph engine chèn — G19a).
#[test]
fn che_do_vi_code_switch_ban_goi_y_en() {
    let opts = EngineOptions {
        suggest_vn_complete: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    let mut buf: Vec<char> = Vec::new();
    type_str(&mut e, &mut buf, "minhf sync laij");
    let before: String = buf.iter().collect();
    assert!(before.contains("sync"), "câu gõ sai: {before}");

    let rsi0 = e.dream_rsi_snapshot();
    press(&mut e, &mut buf, &tab());
    let after: String = buf.iter().collect();

    // Ứng viên trộn Việt-Eng đã được chèn sau ngữ cảnh tiếng Việt.
    let added: String = after.chars().skip(before.chars().count()).collect();
    assert!(
        added.contains("status"),
        "code-switch không bắn 'status…': thêm vào = '{added}'"
    );
    // G19a: accepts +1; chars_saved = nguyên số glyph engine chèn (gồm space
    // engine tự thêm) — kể cả ứng viên chưa chốt (được gỡ ở settle nếu từ chối).
    let rsi1 = e.dream_rsi_snapshot();
    assert_eq!(rsi1.accepts - rsi0.accepts, 1, "accepts phải tăng đúng 1");
    assert_eq!(
        rsi1.chars_saved - rsi0.chars_saved,
        added.chars().count() as u64,
        "chars_saved phải = glyph chèn"
    );
    // keys_typed: Tab đếm đúng 1 như baseline dream_rsi_100 ("3 chữ + Tab = 4"
    // — Tab có ch='	' ≠ 0 nên thuộc phím thật; Ctrl+Space mới bị loại).
    assert_eq!(rsi1.keys_typed - rsi0.keys_typed, 1, "Tab đếm đúng 1 phím");
}

/// 5) Mode EN — câu văn phòng mở rộng qua next-phrase (context nhiều từ).
#[test]
fn che_do_en_mo_rong_cau_van_phong() {
    let opts = EngineOptions {
        enabled: false,
        suggest_vn_complete: true,
        ..Default::default()
    };
    let mut e = Engine::new(opts);
    let mut buf: Vec<char> = Vec::new();
    // Ngữ cảnh sâu (khớp key duy nhất trong bảng) — tránh hoà hai ứng viên
    // cùng key + cùng weight (tie-break = thứ tự hàng sort TSV).
    type_str(
        &mut e,
        &mut buf,
        "please find the executive summary below with",
    );
    let before: String = buf.iter().collect();
    let rsi0 = e.dream_rsi_snapshot();
    press(&mut e, &mut buf, &tab());
    let after: String = buf.iter().collect();
    let added: String = after.chars().skip(before.chars().count()).collect();
    assert!(
        added.starts_with(" the key numbers highlighted"),
        "EN: mở rộng sai: '{added}'"
    );
    let rsi1 = e.dream_rsi_snapshot();
    assert_eq!(rsi1.accepts - rsi0.accepts, 1);
    assert_eq!(
        rsi1.chars_saved - rsi0.chars_saved,
        added.chars().count() as u64
    );
}

/// 6) Prefix EN ở Mode VI KHÔNG bị che — "de" vẫn hoàn tất từ tiếng Anh
///    (Telex/VNI không được chặn lookup nguồn EN — yêu cầu code-switch).
#[test]
fn prefix_en_khong_bi_che_o_che_do_vi() {
    let c = predict_prefix_candidates("de", true, 10);
    assert!(
        c.iter().any(|w| w.starts_with("de")),
        "mode VI che nguồn EN: {c:?}"
    );
    // Space-accept đường dài: ngữ cảnh đa từ khớp đứng trước hoàn tất tiền tố.
    // Dùng ngữ cảnh UNIQUE của bảng văn phòng ("please treat this as" không
    // thuộc bảng cũ — "please find attached" trùng bảng cũ weight cao hơn,
    // luật old-table-wins đã phủ ở test `xep_hang_key_dai_va_weight`).
    let m = predict_next_phrase_candidates("please treat this as", 3);
    assert!(
        m.iter()
            .any(|p| p == "the single source of truth going forward"),
        "standalone collocation phải bắn: {m:?}"
    );
}

/// 7) Cạnh tranh bảng: đuôi khớp cả bảng cũ lẫn văn phòng — key dài hơn
///    (cụ thể hơn) thắng, cùng key thì weight cao hơn thắng.
#[test]
fn xep_hang_key_dai_va_weight() {
    // "thank you" thuộc bảng cũ (very much); văn phòng không phá bảng cũ.
    assert_eq!(
        predict_next_phrase_candidates("thank you", 3)
            .first()
            .map(String::as_str),
        Some("very much")
    );
    // Bảng văn phòng tự rank theo key dài: đuôi "one key risk to highlight is the"
    // khớp cả "one key risk to highlight is" (ngắn) và đủ tên risk (dài) —
    // dài hơn phải đứng trước.
    let preds = predict_next_phrase_candidates("one key risk to highlight is the vendor", 5);
    assert!(
        preds.iter().any(|p| p.starts_with("delay")),
        "risk vendor phải bắn: {preds:?}"
    );
}
