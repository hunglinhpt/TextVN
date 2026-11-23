// SPDX-License-Identifier: GPL-3.0-or-later
//! E59: port 3 script sinh icon/art (đã xoá theo policy không-Python):
//! - `scripts/generate_app_icons.py`  → `cargo xtask gen-app-icons`
//! - `scripts/generate_store_art.py`  → `cargo xtask gen-store-art`
//! - `scripts/generate_tray_icons.py` → `cargo xtask gen-tray-icons`
//!
//! Khác biệt cố ý so bản Pillow (ghi rõ để audit sau khỏi thắc mắc):
//! 1. **Chữ V/E bằng glyph vector dựng sẵn** (polygon const) thay vì load font
//!    hệ thống (Segoe UI Black chỉ có trên Windows) — phát sinh deterministic
//!    như nhau trên mọi máy; hình dáng chữ đậm tương đương, KHÔNG pixel-đúng
//!    bản cũ (raster font ≠ polygon). Assets đã commit KHÔNG tự regenerate —
//!    công cụ chỉ chạy khi cần sinh lại thủ công.
//! 2. PNG xuất qua PNG-encoder nén LZ77 Fixed-Huffman + lọc Sub thuần std
//!    (xem `png.rs`) — giữ mọi PNG (kể cả 1024px/2160px) dưới ngưỡng 1,5 MB.
//! 3. Downsample /2 = box-mean premultiplied (LANCZOS của PIL không port —
//!    sai lệch texture của bản cũ chấp nhận cho art).
//! 4. Preview-sheet (TEMP) của bản tray bỏ (dev nicety, không trong CI).
//!
//! Không unsafe, không crate ngoài, RGBA canvas xem [`crate::png`].

use crate::png::{encode_ico, encode_rgba, star_points, Canvas, Rgba};

fn repo_root() -> std::path::PathBuf {
    crate::assist_tools::repo_root()
}

fn save(out_root: &std::path::Path, rel: String, canvas: &Canvas) -> Result<(), String> {
    let path = out_root.join(&rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let bytes = encode_rgba(canvas);
    std::fs::write(&path, &bytes).map_err(|e| format!("ghi {}: {e}", path.display()))?;
    println!("  gen  {} ({} bytes)", path.display(), bytes.len());
    Ok(())
}

/// `--out-dir <path>`: ghi vào thư mục khác (mặc định = repo). Cho phép sinh thử
/// mà KHÔNG đè assets đã commit. Từ chối mọi cờ lạ để tránh typo đè nhầm repo.
fn out_dir(args: &[String]) -> Result<std::path::PathBuf, String> {
    let mut dir: Option<std::path::PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out-dir" {
            let val = args
                .get(i + 1)
                .ok_or_else(|| "--out-dir cần đường dẫn".to_string())?;
            dir = Some(std::path::PathBuf::from(val));
            i += 2;
        } else if let Some(val) = args[i].strip_prefix("--out-dir=") {
            if val.is_empty() {
                return Err("--out-dir cần đường dẫn".into());
            }
            dir = Some(std::path::PathBuf::from(val));
            i += 1;
        } else {
            return Err(format!(
                "tham số lạ `{}` (chỉ hỗ trợ `--out-dir <path>`)",
                args[i]
            ));
        }
    }
    Ok(dir.unwrap_or_else(repo_root))
}

// ---------------------------------------------------------------------------
//=== Glyph V/E vector (hộp chuẩn 100×100, origin trên-trái) ==================
// ---------------------------------------------------------------------------
/// Chữ V đậm: 2 nét chéo xuống gặp nhau — 7 đỉnh outline kín.
fn glyph_v() -> Vec<(f64, f64)> {
    vec![
        (2.0, 0.0),
        (31.0, 0.0),
        (50.0, 58.0),
        (69.0, 0.0),
        (98.0, 0.0),
        (64.0, 100.0),
        (36.0, 100.0),
    ]
}

/// Chữ E đậm: dọc trái + 3 thanh ngang — 12 đỉnh outline kín.
fn glyph_e() -> Vec<(f64, f64)> {
    vec![
        (2.0, 0.0),
        (92.0, 0.0),
        (92.0, 24.0),
        (27.0, 24.0),
        (27.0, 38.0),
        (84.0, 38.0),
        (84.0, 60.0),
        (27.0, 60.0),
        (27.0, 76.0),
        (92.0, 76.0),
        (92.0, 100.0),
        (2.0, 100.0),
    ]
}

/// Vẽ glyph (hộp 100) vào vùng (x, y, w, h) — căn giữa theo khung đặt; nếu
/// `stroke` Some((màu, độ dày)) thì vẽ outline phrase trước rồi glyph chính.
// Helper đồ hoạ: gộp toạ độ thành tuple chỉ làm code khó đọc hơn - giữ rõ ràng.
#[allow(clippy::too_many_arguments)]
fn draw_glyph(
    canvas: &mut Canvas,
    letter: char,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    fill: Rgba,
    stroke: Option<(Rgba, f64)>,
) {
    let base = if letter == 'V' { glyph_v() } else { glyph_e() };
    let to = |pts: &[(f64, f64)], dx: f64, dy: f64| -> Vec<(f64, f64)> {
        pts.iter()
            .map(|(gx, gy)| (x + (gx / 100.0) * w + dx, y + (gy / 100.0) * h + dy))
            .collect()
    };
    if let Some((sc, sw)) = stroke {
        // Nở hình thái theo 12 hướng tròn (giữ đúng viền trong của chữ không lồi như E/V).
        let steps = 12usize;
        for k in 0..steps {
            let ang = 2.0 * std::f64::consts::PI * (k as f64) / (steps as f64);
            canvas.polygon(&to(&base, sw * ang.cos(), sw * ang.sin()), sc);
        }
    }
    canvas.polygon(&to(&base, 0.0, 0.0), fill);
}

// ---------------------------------------------------------------------------
//=== gen-app-icons — badge TextVN (thay generate_app_icons.py) ===============
// ---------------------------------------------------------------------------
/// (stroke, gradient_bottom, text) — hex khớp `resources/icons/textvn_{v,e}.svg`.
const APP_V: (&str, &str, &str) = ("#C2185B", "#FCE4EC", "#880E4F");
const APP_E: (&str, &str, &str) = ("#0288D1", "#E1F5FE", "#01579B");

fn render_app_badge(
    letter: char,
    stroke: Rgba,
    grad_bottom: Rgba,
    text_color: Rgba,
    size: usize,
) -> Canvas {
    let s = size as f64 / 128.0;
    let canvas = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    // Bóng đổ: silhouette ngoài (rect 4..124 r28) — blur + dy=2 (SVG feDropShadow).
    let mut silhouette = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    silhouette.rounded_rect(
        4.0 * s,
        4.0 * s,
        124.0 * s,
        124.0 * s,
        28.0 * s,
        Rgba(0, 0, 0, 255),
    );
    let blurred = silhouette.blur_alpha(3.0 * s);
    let mut shadow = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    shadow.paste_with_mask(
        // 0x40 (~25% opacity) khớp `stroke + "40"` (generate_app_icons.py) và SVG flood-opacity="0.25".
        &Canvas::new(size, size, Rgba(stroke.0, stroke.1, stroke.2, 0x40)),
        &blurred,
        (2.0 * s).round() as isize,
    );
    let mut out = canvas;
    out.composite(&shadow, 0);
    // Viền ngoài
    out.rounded_rect(4.0 * s, 4.0 * s, 124.0 * s, 124.0 * s, 28.0 * s, stroke);
    // Gradient nền trong (mép trong của stroke: 12..116, r20)
    let mut inner = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    {
        let (w, h) = (inner.w, inner.h);
        let mut mask = Canvas::new(w, h, Rgba(0, 0, 0, 0));
        mask.rounded_rect(
            12.0 * s,
            12.0 * s,
            116.0 * s,
            116.0 * s,
            20.0 * s,
            Rgba(255, 255, 255, 255),
        );
        let mut grad = Canvas::new(w, h, Rgba(255, 255, 255, 255));
        grad.gradient_bg(Rgba(255, 255, 255, 255), grad_bottom);
        for y in 0..h {
            for x in 0..w {
                let a = mask.px[(y * w + x) * 4 + 3];
                if a > 0 {
                    let g = grad.get_pixel(x, y);
                    inner.set_pixel(x, y, Rgba(g.0, g.1, g.2, a));
                }
            }
        }
    }
    out.composite(&inner, 0);
    // Glyph cap-height ~60*s căn giữa trong khung trong [12..116]*s (không đụng viền ngoài 4..12*s).
    draw_glyph(
        &mut out,
        letter,
        32.0 * s,
        34.0 * s,
        64.0 * s,
        60.0 * s,
        text_color,
        None,
    );
    out
}

pub fn gen_app_icons_cli(args: &[String]) -> Result<(), String> {
    let root = out_dir(args)?;
    std::fs::create_dir_all(root.join("packaging/macos/icons"))
        .map_err(|e| format!("mkdir: {e}"))?;
    let _ = std::fs::create_dir_all(root.join("resources/icons"));
    for (name, letter, (stroke, grad, text)) in [("textvn_v", 'V', APP_V), ("textvn_e", 'E', APP_E)]
    {
        let img128 = render_app_badge(
            letter,
            Rgba::from_hex(stroke),
            Rgba::from_hex(grad),
            Rgba::from_hex(text),
            128,
        );
        save(&root, format!("resources/icons/{name}.png"), &img128)?;
    }
    let (stroke, grad, text) = APP_V;
    let big = render_app_badge(
        'V',
        Rgba::from_hex(stroke),
        Rgba::from_hex(grad),
        Rgba::from_hex(text),
        1024,
    );
    save(&root, "packaging/macos/icons/TextVN-1024.png".into(), &big)?;
    save(
        &root,
        "packaging/macos/icons/TextVN-512.png".into(),
        &big.downsample2(),
    )?;
    println!("gen-app-icons: OK");
    Ok(())
}

// ---------------------------------------------------------------------------
//=== gen-store-art — box/poster Microsoft Store (thay generate_store_art.py) =
// ---------------------------------------------------------------------------
const FLAG_RED: Rgba = Rgba::rgb(218, 37, 29);
const RED_DARK: Rgba = Rgba::rgb(150, 18, 14);
const STAR_YELLOW: Rgba = Rgba::rgb(255, 221, 0);
const ART_WHITE: Rgba = Rgba::rgb(255, 255, 255);
const ART_SHADOW: Rgba = Rgba(90, 10, 8, 255);

fn compose_art(w: usize, h: usize, star_r: f64, star_cy: f64, v_size: f64, v_cy: f64) -> Canvas {
    let mut canvas = Canvas::new(w, h, FLAG_RED);
    canvas.gradient_bg(FLAG_RED, RED_DARK);
    let cx = w as f64 / 2.0;
    // Chữ V + bóng đổ lệch (+off, +off) giống generate_store_art.py (tỉ lệ vàng 0.38197).
    let off = (4.0f64).max(v_size / 60.0);
    draw_glyph(
        &mut canvas,
        'V',
        cx - v_size / 2.0 + off,
        v_cy - v_size * 0.62 / 2.0 + off,
        v_size,
        v_size * 0.62,
        ART_SHADOW,
        None,
    );
    draw_glyph(
        &mut canvas,
        'V',
        cx - v_size / 2.0,
        v_cy - v_size * 0.62 / 2.0,
        v_size,
        v_size * 0.62,
        ART_WHITE,
        None,
    );
    canvas.polygon(&star_points(cx, star_cy, star_r, 0.38197), STAR_YELLOW);
    canvas
}

pub fn gen_store_art_cli(args: &[String]) -> Result<(), String> {
    let root = out_dir(args)?;
    let out = root.join("store/art");
    std::fs::create_dir_all(&out).map_err(|e| format!("mkdir: {e}"))?;
    // 1:1 — 2160 (nguồn) + 1080.
    let box_art = compose_art(2160, 2160, 380.0, 760.0, 980.0, 1530.0);
    save(&root, "store/art/box-art-2160.png".into(), &box_art)?;
    save(
        &root,
        "store/art/box-art-1080.png".into(),
        &box_art.downsample2(),
    )?;
    // 2:3 — 1440x2160 (nguồn) + 720x1080.
    let poster = compose_art(1440, 2160, 300.0, 620.0, 760.0, 1400.0);
    save(&root, "store/art/poster-art-1440x2160.png".into(), &poster)?;
    save(
        &root,
        "store/art/poster-art-720x1080.png".into(),
        &poster.downsample2(),
    )?;
    println!("gen-store-art: OK (store/art/*.png)");
    Ok(())
}

// ---------------------------------------------------------------------------
//=== gen-tray-icons — .ico cờ VN/US tray (thay generate_tray_icons.py) =======
// ---------------------------------------------------------------------------
const TRAY_SIZES: [usize; 8] = [256, 128, 64, 48, 32, 24, 20, 16];
const VN_RED: Rgba = Rgba::rgb(218, 37, 29);
const VN_GOLD: Rgba = Rgba::rgb(255, 205, 0);
const US_RED: Rgba = Rgba::rgb(178, 34, 52);
const US_WHITE: Rgba = Rgba::rgb(255, 255, 255);
const US_BLUE: Rgba = Rgba::rgb(60, 59, 110);
const TRAY_STROKE_V: Rgba = Rgba(30, 20, 10, 190);
const TRAY_STROKE_E: Rgba = Rgba(15, 15, 25, 225);

/// Mask bo góc 20%% như bản python (m=3%%, radius 20%%).
fn rounded_mask(size: usize) -> Canvas {
    let mut mask = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    let m = (size as f64 * 0.03).max(1.0);
    let radius = (size as f64 * 0.20).max(2.0);
    mask.rounded_rect(
        m,
        m,
        size as f64 - 1.0 - m,
        size as f64 - 1.0 - m,
        radius,
        Rgba(255, 255, 255, 255),
    );
    mask
}

fn apply_rounded(base: &Canvas, size: usize) -> Canvas {
    let mask = rounded_mask(size);
    let mut out = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    out.paste_with_mask(base, &mask, 0);
    out
}

fn tray_vn(size: usize) -> Canvas {
    let mut base = Canvas::new(size, size, Rgba(0, 0, 0, 0));
    base.fill_rect(0.0, 0.0, size as f64 - 1.0, size as f64 - 1.0, VN_RED);
    let s = size as f64;
    base.polygon(&star_points(s * 0.5, s * 0.37, s * 0.31, 0.38197), VN_GOLD);
    let mut out = apply_rounded(&base, size);
    // Chữ V trắng strokeWidth size/40.
    let sw = (size / 40).max(1) as f64;
    let font_h = s * if size <= 24 { 0.60 } else { 0.56 };
    draw_glyph(
        &mut out,
        'V',
        s * 0.2,
        s * 0.60 - font_h / 2.0,
        s * 0.6,
        font_h,
        US_WHITE,
        Some((TRAY_STROKE_V, sw)),
    );
    out
}

fn tray_us(size: usize) -> Canvas {
    let s = size as f64;
    let mut base = Canvas::new(size, size, US_WHITE);
    let n_bands = if size >= 32 { 13 } else { 7 };
    let stripe_h = s / n_bands as f64;
    for i in 0..n_bands {
        if i % 2 == 0 {
            base.fill_rect(
                0.0,
                i as f64 * stripe_h,
                s - 1.0,
                (i + 1) as f64 * stripe_h,
                US_RED,
            );
        }
    }
    let cw = s * 0.40;
    let ch = stripe_h * (7.min(n_bands / 2 + 1).min(n_bands)) as f64;
    base.fill_rect(0.0, 0.0, cw, ch.max(0.0), US_BLUE);
    if size >= 32 {
        let r = (s * 0.022).max(1.0);
        for row in 0..4 {
            for col in 0..5 {
                let x = cw * (col + 1) as f64 / 6.0;
                let y = ch * (row + 1) as f64 / 5.0;
                base.polygon(&star_points(x, y, r * 2.2, 0.38197), US_WHITE);
            }
        }
    } else if size >= 20 {
        let r = (s * 0.035).max(1.0);
        for (gx, gy) in [(0.25, 0.25), (0.6, 0.25), (0.25, 0.6), (0.6, 0.6)] {
            base.ellipse(cw * gx - r, ch * gy - r, cw * gx + r, ch * gy + r, US_WHITE);
        }
    }
    let mut out = apply_rounded(&base, size);
    let sw = (size / 16).max(1) as f64;
    let font_h = s * if size <= 24 { 0.60 } else { 0.56 };
    draw_glyph(
        &mut out,
        'E',
        s * 0.2,
        s * 0.55 - font_h / 2.0,
        s * 0.6,
        font_h,
        US_WHITE,
        Some((TRAY_STROKE_E, sw)),
    );
    out
}

pub fn gen_tray_icons_cli(args: &[String]) -> Result<(), String> {
    let root = out_dir(args)?;
    let tray_dir = root.join("tray/resources");
    std::fs::create_dir_all(&tray_dir).map_err(|e| format!("mkdir: {e}"))?;
    for (name, gen) in [
        ("textvn_v", tray_vn as fn(usize) -> Canvas),
        ("textvn_e", tray_us as fn(usize) -> Canvas),
    ] {
        let frames: Vec<(u32, u32, Vec<u8>)> = TRAY_SIZES
            .iter()
            .map(|&s| {
                let c = gen(s);
                (s as u32, s as u32, encode_rgba(&c))
            })
            .collect();
        let refs: Vec<(u32, u32, &[u8])> = frames
            .iter()
            .map(|(w, h, p)| (*w, *h, p.as_slice()))
            .collect();
        let ico = encode_ico(&refs);
        let path = tray_dir.join(format!("{name}.ico"));
        std::fs::write(&path, &ico).map_err(|e| format!("ghi {}: {e}", path.display()))?;
        println!("  gen  tray/resources/{name}.ico ({} bytes)", ico.len());
    }
    // Default textvn.ico cho tray + installer (giữ layout bản python).
    let v_ico = std::fs::read(tray_dir.join("textvn_v.ico"))
        .map_err(|e| format!("đọc textvn_v.ico: {e}"))?;
    std::fs::write(root.join("tray/resources/textvn.ico"), &v_ico)
        .map_err(|e| format!("ghi: {e}"))?;
    let inst = root.join("installer/windows/resources");
    std::fs::create_dir_all(&inst).map_err(|e| format!("mkdir: {e}"))?;
    std::fs::write(inst.join("textvn.ico"), &v_ico).map_err(|e| format!("ghi: {e}"))?;
    println!("gen-tray-icons: OK (tray/resources/*.ico + installer/windows/resources/textvn.ico)");
    Ok(())
}

// ---------------------------------------------------------------------------
//=== Unit tests ==============================================================
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::encode_rgba;

    #[test]
    fn badge_png_valid_structure() {
        let c = render_app_badge(
            'V',
            Rgba::from_hex(APP_V.0),
            Rgba::from_hex(APP_V.1),
            Rgba::from_hex(APP_V.2),
            128,
        );
        assert_eq!(c.w, 128);
        // viền trái trên có màu stroke (điểm 5,5 trong mask rounded tại 128px?
        // mép 4*s=4, điểm (5,5) nằm trong bo góc — kiểm pixel (64,6) mép trên
        // giữa chắc chắn nằm trong viền stroke.
        let p = c.get_pixel(64, 6);
        assert!(p.3 > 200, "viền trên phải đặc: {p:?}");
        // tâm trong gradient gần trắng.
        let p = c.get_pixel(64, 30);
        assert!(
            p.0 > 230 && p.3 > 200,
            "gradient nền trong gần trắng: {p:?}"
        );
    }

    #[test]
    fn glyph_out_of_bounds_clamped() {
        // Glyph vẽ ra ngoài canvas không panic; canvas 8px vẫn chạy.
        let mut c = Canvas::new(8, 8, Rgba(0, 0, 0, 0));
        draw_glyph(
            &mut c,
            'V',
            -5.0,
            -5.0,
            20.0,
            20.0,
            US_WHITE,
            Some((TRAY_STROKE_V, 2.0)),
        );
        let p = c.get_pixel(4, 4);
        assert!(p.3 > 0);
    }

    #[test]
    fn tray_vn_us_all_sizes_no_panic() {
        for &s in TRAY_SIZES.iter() {
            let v = tray_vn(s);
            let e = tray_us(s);
            // Nền phải có alpha đặc gần tâm (mask bo góc phủ toàn giữa).
            let mid = s / 2;
            assert!(v.get_pixel(mid, mid).3 > 200, "VN {s}px");
            assert!(e.get_pixel(mid, mid).3 > 200, "US {s}px");
            let png = encode_rgba(&v);
            assert_eq!(&png[0..4], &[0x89, b'P', b'N', b'G']);
        }
    }

    #[test]
    fn ico_eight_frames_header() {
        let frames: Vec<(u32, u32, Vec<u8>)> = TRAY_SIZES
            .iter()
            .map(|&s| (s as u32, s as u32, encode_rgba(&tray_vn(s))))
            .collect();
        let refs: Vec<(u32, u32, &[u8])> = frames
            .iter()
            .map(|(w, h, p)| (*w, *h, p.as_slice()))
            .collect();
        let ico = encode_ico(&refs);
        assert_eq!(&ico[4..6], &8u16.to_le_bytes()); // 8 frames (u16 count)
        assert_eq!(&ico[6..8], &[0, 0]); // entry 1: 256 → 0
        assert_eq!(ico[6 + 4 * 16], 32); // entry 5 (SIZES[4]): 32px
    }

    #[test]
    fn badge_shadow_aa_and_glyph_margin() {
        let c = render_app_badge(
            'V',
            Rgba::from_hex(APP_V.0),
            Rgba::from_hex(APP_V.1),
            Rgba::from_hex(APP_V.2),
            128,
        );
        // Dưới mép dưới (y=126) phải có bóng đổ mờ 0 < alpha <= 64 (trước đây đọc kênh Red = 0).
        let sh = c.get_pixel(64, 126);
        assert!(sh.3 > 0 && sh.3 <= 64, "bóng đổ dưới mép badge: {sh:?}");
        // Vùng đệm trên bên trong (y=16, x=40) phải là nền sáng, KHÔNG bị glyph đè lên viền.
        let top_pad = c.get_pixel(40, 16);
        assert!(
            top_pad.0 > 220 && top_pad.1 > 210,
            "glyph không được đè lề trên: {top_pad:?}"
        );
        // Góc bo của tray_vn phải có pixel AA trung gian (0 < alpha < 255).
        let tv = tray_vn(64);
        let has_aa = (0..20)
            .flat_map(|y| (0..20).map(move |x| (x, y)))
            .any(|(x, y)| {
                let a = tv.get_pixel(x, y).3;
                a > 0 && a < 255
            });
        assert!(has_aa, "góc bo tray icon phải có anti-aliasing");
    }

    #[test]
    fn out_dir_tu_choi_co_la() {
        assert!(out_dir(&["--outdir".into(), "/tmp/x".into()]).is_err());
        assert_eq!(
            out_dir(&["--out-dir=/tmp/x".into()]).unwrap(),
            std::path::PathBuf::from("/tmp/x")
        );
    }

    #[test]
    fn art_compose_layout_and_png_size_under_hygiene_limit() {
        let c = compose_art(1440, 2160, 300.0, 620.0, 760.0, 1400.0);
        // Góc (0,0) ≈ FLAG_RED; góc (w-1,h-1) ≈ RED_DARK (gradient chéo).
        let tl = c.get_pixel(0, 0);
        let br = c.get_pixel(1439, 2159);
        assert!(
            tl.0.abs_diff(218) < 4 && tl.1.abs_diff(37) < 4,
            "TL phải đỏ cờ: {tl:?}"
        );
        assert!(br.0 < 165 && br.1 < 40, "BR phải đỏ tối: {br:?}");
        // Sao vàng trên nền đỏ: điểm gần (cx, star_cy) phải VÀNG (255,221,0).
        let star = c.get_pixel(720, 620);
        assert!(
            star.0 > 200 && star.1 > 190 && star.2 < 40,
            "sao vàng: {star:?}"
        );
        // Bóng đổ chữ V phải lộ pixel tối ART_SHADOW ở vùng lệch (+off, +off) dưới đáy chữ V.
        let bottom_y = (1400.0 + 760.0 * 0.62 / 2.0) as usize;
        let cref = &c;
        let has_shadow = (bottom_y + 1..bottom_y + 14)
            .flat_map(|y| (700..750).map(move |x| cref.get_pixel(x, y)))
            .any(|p| p.0 < 110 && p.1 < 25 && p.2 < 25);
        assert!(has_shadow, "Store art phải có bóng đổ lệch phía dưới chữ V");

        // Cả TextVN-1024 và poster 1440×2160 nén PNG phải < 1,5 MB (gate repo-hygiene).
        let big_icon = render_app_badge(
            'V',
            Rgba::from_hex(APP_V.0),
            Rgba::from_hex(APP_V.1),
            Rgba::from_hex(APP_V.2),
            1024,
        );
        let icon_png = encode_rgba(&big_icon);
        assert!(
            icon_png.len() < 1_500_000,
            "TextVN-1024.png quá lớn ({} bytes >= 1.5 MB)",
            icon_png.len()
        );
        let poster_png = encode_rgba(&c);
        assert!(
            poster_png.len() < 1_500_000,
            "poster-art-1440x2160.png quá lớn ({} bytes >= 1.5 MB)",
            poster_png.len()
        );
    }
}
