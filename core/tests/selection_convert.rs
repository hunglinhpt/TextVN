// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-139 — chuyển đổi vùng bôi đen (`Engine::convert_selection`, Ctrl+Shift+F9):
//! chữ lạ do đọc nhầm UTF-8, bảng mã cũ → bảng mã đang dùng, chữ gõ nhầm chế độ E.
//!
//! Chủ repo: "bôi đen văn bản ở bảng mã khác chuyển về Telex hoặc mode đang chạy
//! ở TextVN" + "mở file lên nhưng chữ bị biến thành ký tự lạ".

use textvn_core::{repair_mojibake, Engine, EngineOptions, Method, OutputCharset};

fn engine(method: Method, out: OutputCharset) -> Engine {
    Engine::new(EngineOptions {
        method,
        enabled: true,
        output_charset: out,
        ..Default::default()
    })
}

fn convert(e: &Engine, s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    e.convert_selection(&chars).map(|v| v.into_iter().collect())
}

/// Byte → ký tự theo cp1252 (byte không định nghĩa → C1 như Windows).
fn cp1252(b: u8) -> char {
    let special = [
        (0x80, '€'),
        (0x82, '‚'),
        (0x83, 'ƒ'),
        (0x84, '„'),
        (0x85, '…'),
        (0x86, '†'),
        (0x87, '‡'),
        (0x88, 'ˆ'),
        (0x89, '‰'),
        (0x8A, 'Š'),
        (0x8B, '‹'),
        (0x8C, 'Œ'),
        (0x8E, 'Ž'),
        (0x91, '‘'),
        (0x92, '’'),
        (0x93, '“'),
        (0x94, '”'),
        (0x95, '•'),
        (0x96, '–'),
        (0x97, '—'),
        (0x98, '˜'),
        (0x99, '™'),
        (0x9A, 'š'),
        (0x9B, '›'),
        (0x9C, 'œ'),
        (0x9E, 'ž'),
        (0x9F, 'Ÿ'),
    ];
    special
        .iter()
        .find(|(k, _)| *k == b)
        .map(|(_, c)| *c)
        .unwrap_or_else(|| char::from_u32(u32::from(b)).unwrap())
}

/// Byte → ký tự theo cp1258 (Windows tiếng Việt).
fn cp1258(b: u8) -> char {
    let mark = |cp: u32| char::from_u32(cp).unwrap();
    match b {
        0x8A | 0x8E | 0x9A | 0x9E => char::from_u32(u32::from(b)).unwrap(),
        0xC3 => 'Ă',
        0xCC => mark(0x300),
        0xD0 => 'Đ',
        0xD2 => mark(0x309),
        0xD5 => 'Ơ',
        0xDD => 'Ư',
        0xDE => mark(0x303),
        0xE3 => 'ă',
        0xEC => mark(0x301),
        0xF0 => 'đ',
        0xF2 => mark(0x323),
        0xF5 => 'ơ',
        0xFD => 'ư',
        0xFE => '₫',
        _ => cp1252(b),
    }
}

/// Văn bản UTF-8 mà app đọc nhầm theo bảng mã ANSI.
fn mojibake(s: &str, table: fn(u8) -> char) -> String {
    s.as_bytes().iter().map(|&b| table(b)).collect()
}

const SAMPLE: &str = "Tiếng Việt: Cộng hòa xã hội chủ nghĩa — đi học “đúng giờ” ₫";

#[test]
fn r2_139_khoi_phuc_chu_la_cp1252_cp1258_va_ma_hoa_doi() {
    let e = engine(Method::Telex, OutputCharset::UnicodePrecomposed);
    let bad = mojibake(SAMPLE, cp1252);
    assert_ne!(bad, SAMPLE);
    assert_eq!(convert(&e, &bad).as_deref(), Some(SAMPLE), "cp1252");
    let bad58 = mojibake(SAMPLE, cp1258);
    assert_eq!(
        convert(&e, &bad58).as_deref(),
        Some(SAMPLE),
        "cp1258 (Windows tiếng Việt)"
    );
    let twice = mojibake(&bad, cp1252);
    assert_eq!(convert(&e, &twice).as_deref(), Some(SAMPLE), "mã hoá đôi");
    // Văn bản Việt đúng (có Đ ư ơ ă — cũng nằm trong bảng cp1258) KHÔNG bị "sửa".
    let ok: Vec<char> = "Đường đi ở đâu, ăn ơn ưng ý".chars().collect();
    assert_eq!(repair_mojibake(&ok), None);
    assert_eq!(convert(&e, "Đường đi ở đâu"), None);
}

#[test]
fn r2_139_bang_ma_cu_ve_bang_ma_dang_dung() {
    let tcvn = textvn_core::transform::charset::encode(
        &"Tiếng Việt chuẩn".chars().collect::<Vec<_>>(),
        OutputCharset::Tcvn3,
    )
    .into_iter()
    .collect::<String>();
    let vni = textvn_core::transform::charset::encode(
        &"Tiếng Việt chuẩn".chars().collect::<Vec<_>>(),
        OutputCharset::VniWindows,
    )
    .into_iter()
    .collect::<String>();
    // Đang dùng Unicode: TCVN3/VNI → Unicode.
    let uni = engine(Method::Telex, OutputCharset::UnicodePrecomposed);
    assert_eq!(
        convert(&uni, &tcvn).as_deref(),
        Some("Tiếng Việt chuẩn"),
        "TCVN3 → Unicode"
    );
    assert_eq!(
        convert(&uni, &vni).as_deref(),
        Some("Tiếng Việt chuẩn"),
        "VNI-Windows → Unicode"
    );
    // Đang dùng TCVN3: văn bản Unicode và VNI → TCVN3.
    let t3 = engine(Method::Telex, OutputCharset::Tcvn3);
    assert_eq!(
        convert(&t3, "Tiếng Việt chuẩn").as_deref(),
        Some(tcvn.as_str())
    );
    assert_eq!(convert(&t3, &vni).as_deref(), Some(tcvn.as_str()));
    // Đã đúng bảng mã đang dùng ⇒ không đổi.
    assert_eq!(convert(&uni, "Tiếng Việt chuẩn"), None);
}

#[test]
fn r2_139_go_lai_chu_go_nham_che_do_e() {
    let telex = engine(Method::Telex, OutputCharset::UnicodePrecomposed);
    assert_eq!(
        convert(&telex, "tooi ddi hocj").as_deref(),
        Some("tôi đi học")
    );
    assert_eq!(convert(&telex, "Vieetj Nam.").as_deref(), Some("Việt Nam."));
    // Câu lẫn: từ đã có dấu giữ nguyên, chỉ từ ASCII gõ nhầm được gõ lại; xuống
    // dòng/khoảng trắng giữ nguyên.
    assert_eq!(
        convert(&telex, "tôi muốn ddi hocj\nngayf mai").as_deref(),
        Some("tôi muốn đi học\nngày mai")
    );
    // Không bung gõ tắt khi gõ lại (hd không thành "hợp đồng").
    let mut opts = telex.options().clone();
    opts.macro_trigger = textvn_core::MacroTrigger::Space;
    opts.macros = vec![textvn_core::MacroDef {
        trigger: "hd".into(),
        expand: "hợp đồng".into(),
        when: textvn_core::MacroWhen::default(),
    }];
    let with_macro = Engine::new(opts);
    assert_eq!(
        convert(&with_macro, "hd soos 1").as_deref(),
        Some("hd số 1")
    );

    let vni = engine(Method::Vni, OutputCharset::UnicodePrecomposed);
    assert_eq!(
        convert(&vni, "to6i d9i ho5c").as_deref(),
        Some("tôi đi học")
    );

    // Bảng mã đang dùng là TCVN3: chữ gõ lại xuất ra TCVN3.
    let t3 = engine(Method::Telex, OutputCharset::Tcvn3);
    let want = textvn_core::transform::charset::encode(
        &"tôi đi học".chars().collect::<Vec<_>>(),
        OutputCharset::Tcvn3,
    )
    .into_iter()
    .collect::<String>();
    assert_eq!(
        convert(&t3, "tooi ddi hocj").as_deref(),
        Some(want.as_str())
    );

    // Không có gì để đổi ⇒ None (caller không ghi document).
    assert_eq!(convert(&telex, "123 456"), None);
}
