// SPDX-License-Identifier: GPL-3.0-or-later
//! E63 — Multi-Persona Expert Audit & Regression Suite:
//! - View 1: Mathematical Researcher (LaTeX / Typst / Overleaf / VS Code / Markdown+LaTeX)
//! - View 2: High School & University Students (Toán / Lý / Hóa / Sinh / Tin học)
//! - View 3: UniKey Classic Power User & Bilingual VI/EN Mix (Vietnamese default wins vs intentional UniKey undo EN wins)
//! - View 4: Cross-Feature Interactions (Math + Emoji + Vietnamese + English)

use strategy::{
    IME_FIELD_BODY, IME_STRATEGY_BACKSPACE_TYPE, IME_STRATEGY_FORWARD_AS_COMMIT,
    IME_STRATEGY_PREEDIT,
};
use textvn_core::{
    keymap, Action, Context, DiacriticStyle, Engine, EngineOptions, KeyEvent, Method, FLAG_CONSUMED,
};
use textvn_math::MathOutputMode;

fn type_stream_with_strategy(e: &mut Engine, strategy_hint: i64, keys: &str) -> String {
    let ctx = Context {
        field_role: IME_FIELD_BODY,
        hint: strategy_hint,
        ..Default::default()
    };
    e.set_context(ctx);
    type_stream(e, keys)
}

fn type_stream(e: &mut Engine, keys: &str) -> String {
    let mut buf: Vec<char> = Vec::new();
    for c in keys.chars() {
        let k = match c {
            ' ' => KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..Default::default()
            },
            '\n' => KeyEvent::key_down(keymap::vk::RETURN),
            '\t' => KeyEvent::key_down(keymap::vk::TAB),
            '\x08' => KeyEvent::key_down(keymap::vk::BACK),
            _ => KeyEvent::char_down(c),
        };
        let out = e.key(&k);
        match out.action {
            Action::Pass => {
                if c == '\x08' {
                    buf.pop();
                } else if out.flags & FLAG_CONSUMED == 0 {
                    buf.push(c);
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
                buf.truncate(buf.len().saturating_sub(delete_count as usize));
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
    buf.into_iter().collect()
}

fn opts_math(mode: MathOutputMode) -> EngineOptions {
    let mut o = EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    };
    o.math.enabled = true;
    o.math.output_mode = mode;
    o
}

fn opts_plain() -> EngineOptions {
    EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    }
}

#[test]
fn view1_math_researcher_latex_paper_and_delimiters() {
    // 1. Bài báo LaTeX tiếng Việt đầy đủ: `\begin{document}`, `\section{...}`, `\textbf{...}`,
    // `\begin{theorem}`, `\begin{proof}`, `\\` KHÔNG được treo tiếng Việt!
    // Chỉ môi trường toán (`align*`, `equation*`, `subequations`) và `$...$` mới treo tiếng Việt,
    // còn `\intertext{...}` / `\text{...}` bên trong math bật lại tiếng Việt!
    let mut e = Engine::new(opts_math(MathOutputMode::Latex));
    let doc = type_stream(
        &mut e,
        "\\begin{document} \\section{Gioiws thieeuj} \\textbf{Ddinhj lys} ",
    );
    assert_eq!(
        doc,
        "\\begin{document} \\section{Giới thiệu} \\textbf{Định lý} "
    );

    let mut e2 = Engine::new(opts_math(MathOutputMode::Latex));
    let thm = type_stream(
        &mut e2,
        "\\begin{theorem} Giar suwr $f(x) = \\int_0^\\infty e^{-x^2} dx$ lien tucc. \\end{theorem} ",
    );
    assert!(
        thm.starts_with("\\begin{theorem} Giả sử $f(x) = \\int_0^\\infty e^{-x^2} dx$"),
        "theorem + inline math: {thm:?}"
    );

    let mut e3 = Engine::new(opts_math(MathOutputMode::Latex));
    let align = type_stream(
        &mut e3,
        "\\begin{align*} aa &= bb \\\\ \\intertext{vowis moij soos thuwcj} cc &= dd \\end{align*} chaof ",
    );
    assert_eq!(
        align,
        "\\begin{align*} aa &= bb \\\\ \\intertext{với mọi số thực} cc &= dd \\end{align*} chào "
    );

    // Môi trường tên dài (>16 ký tự): `\begin{subequations}...\end{subequations}`, `\begin{displaymath}`
    let mut e4 = Engine::new(opts_math(MathOutputMode::Latex));
    let subeq = type_stream(
        &mut e4,
        "\\begin{subequations} aa = bb \\end{subequations} dduwowcj ",
    );
    assert_eq!(
        subeq,
        "\\begin{subequations} aa = bb \\end{subequations} được "
    );

    // LaTeX raw args (`\frac{1}{2}`, `\sqrt[3]{x}`, `\alpha+1`, `\alpha^2`) không bị bung đè
    let mut e5 = Engine::new(opts_math(MathOutputMode::Latex));
    let raw = type_stream(
        &mut e5,
        "$\\frac{1}{2} + \\sqrt[3]{x} + \\alpha+1 + \\alpha^2$ tieengs Vieetj ",
    );
    assert_eq!(
        raw,
        "$\\frac{1}{2} + \\sqrt[3]{x} + \\alpha+1 + \\alpha^2$ tiếng Việt "
    );

    // Escaped dollar `\$50` không mở math span, `\\` xuống dòng LaTeX không treo tiếng Việt
    let mut e6 = Engine::new(opts_math(MathOutputMode::Latex));
    assert_eq!(
        type_stream(&mut e6, "Gias \\$50 ddoongf \\\\ dongf tieeps "),
        "Giá \\$50 đồng \\\\ dòng tiếp "
    );

    // Môi trường lồng nhau (`\begin{cases}...\end{cases}` bên trong `$...$`) + nhãn/trích dẫn
    let mut e7 = Engine::new(opts_math(MathOutputMode::Latex));
    assert_eq!(
        type_stream(
            &mut e7,
            "$f(x) = \\begin{cases} x & \\text{neeus } x > 0 \\\\ -x & \\text{neeus } x \\le 0 \\end{cases}$ \\label{eq:euler} \\cite{gauss1801} chaof "
        ),
        "$f(x) = \\begin{cases} x & \\text{nếu } x > 0 \\\\ -x & \\text{nếu } x \\le 0 \\end{cases}$ \\label{eq:euler} \\cite{gauss1801} chào "
    );

    // Chế độ Typst & AutoByApp (Code.exe → Latex, WINWORD.EXE → Normal)
    let mut e_typst = Engine::new(opts_math(MathOutputMode::Typst));
    assert_eq!(
        type_stream(&mut e_typst, "\\alpha + \\infty "),
        "alpha + infinity "
    );

    // Đường dẫn Windows khi bật Math & Science không được tự bung sai giữa chừng
    let mut e_path = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e_path, "C:\\Users\\hungl\\Desktop "),
        "C:\\Users\\hungl\\Desktop "
    );
}

#[test]
fn view2_student_math_physics_chemistry_normal_mode() {
    // 1. Superscript / Subscript chain hoạt động ngay ở Normal mode mặc định (Word/Zalo/Chrome...)
    let mut e = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(type_stream(&mut e, "x`^2 + y`^2 = z`^2 "), "x² + y² = z² ");
    assert_eq!(type_stream(&mut e, "x`^n + x`^-1 "), "xⁿ + x⁻¹ ");
    assert_eq!(type_stream(&mut e, "x`^(n+1) "), "x⁽ⁿ⁺¹⁾ ");
    assert_eq!(type_stream(&mut e, "H`_2O vaf H`_2SO`_4 "), "H₂O và H₂SO₄ ");
    assert_eq!(type_stream(&mut e, "SO`_4`^2- + Ca`^2+ "), "SO₄²⁻ + Ca²⁺ ");
    assert_eq!(
        type_stream(&mut e, "a`_i + a`_n + a`_n-1 "),
        "aᵢ + aₙ + aₙ₋₁ "
    );

    // 2. Alias Toán / Lý / Hóa học sinh Việt Nam + chữ thường hóa học + ion có `+`/`-`
    let mut e2 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(
            &mut e2,
            "\\voimoi x \\in \\R, \\can \\tichphan \\tong \\infty \\chia \\nhan "
        ),
        "∀ x ∈ ℝ, √x ∫ ∑ ∞ ÷ × "
    );
    let mut e3 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(
            &mut e3,
            "\\h2o + \\co2 \\canbanghoahoc \\h2so4 \\nh3 \\ch4 \\caco3 \\Fe2+ \\OH- \\SO4-2 "
        ),
        "H₂O + CO₂ ⇌ H₂SO₄ NH₃ CH₄ CaCO₃ Fe²⁺ OH⁻ SO₄²⁻ "
    );

    // 3. Toán tử & dấu ngoặc bao quanh `\alias`
    let mut e4 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e4, "(\\alpha+\\beta) = \\pi/2 + 2*\\pi "),
        "(α+β) = π/2 + 2*π "
    );

    // 4. Máy tính nhanh `=expr=` chuẩn xác (không nhiễu IEEE-754)
    for (expr, want) in [
        ("=0.1+0.2=", "0.3"),
        ("=sin(pi/6)=", "0.5"),
        ("=cos(pi/2)=", "0"),
        ("=sin(pi)=", "0"),
        ("=-0=", "0"),
        ("=sqrt(16)=", "4"),
        ("=2^10=", "1024"),
        ("=abs(-5)=", "5"),
        ("= 2 + 3 * 4 =", "14"),
    ] {
        let mut ex = Engine::new(opts_math(MathOutputMode::Normal));
        assert_eq!(type_stream(&mut ex, expr), want, "eval {expr}");
    }

    // 5. Dấu `=` lẻ trong bài làm tiếng Việt KHÔNG BAO GIỜ được giết Telex câu sau!
    let mut e6 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e6, "Ta cos x = 5, neen keets quar laf tieengs Vieetj "),
        "Ta có x = 5, nên kết quả là tiếng Việt "
    );
    let mut e7 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e7, "x = 5 neen dduwowcj "),
        "x = 5 nên được "
    );
    let mut e8 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e8, "a=5 vaf b=10 neen toorng =5+10= laf 15 "),
        "a=5 và b=10 nên tổng 15 là 15 "
    );

    // 6. Biểu thức lỗi/tràn/dấu `==` trong code không bao giờ panic hay treo Telex!
    for (seq, want) in [
        ("a == b neen chaof ", "a == b nên chào "),
        ("=1/0= chaof ", "=1/0= chào "),
        ("=2^10000= chaof ", "=2^10000= chào "),
        ("=(2+3= chaof ", "=(2+3= chào "),
        (
            "Cho a = b + c, suy ra dduwowcj ",
            "Cho a = b + c, suy ra được ",
        ),
        ("Cho a = e + 1, neen dduwowcj ", "Cho a = e + 1, nên được "),
        (
            "Ta cos x = sin(y) neen chaof ",
            "Ta có x = sin(y) nên chào ",
        ),
    ] {
        let mut ex = Engine::new(opts_math(MathOutputMode::Normal));
        assert_eq!(type_stream(&mut ex, seq), want, "eval guard `{seq}`");
    }

    // 7. Backspace trong SupChain thoát chain sạch
    let mut e_bs = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(type_stream(&mut e_bs, "x`^2\x083 "), "x3 ");
}

#[test]
fn view3_bilingual_vi_default_wins_vs_unikey_undo_en_wins() {
    // Cặp song ngữ: gõ bình thường -> tiếng Việt thắng; gõ lặp phím dấu kiểu UniKey -> tiếng Anh thắng
    // (KHÔNG bao giờ ra sirr/carr/varr/coss/taxx/rooom/showw!).
    let pairs: &[(&str, &str, &str, &str)] = &[
        // (gõ thường, kết quả VI thắng, gõ undo UniKey, kết quả EN thắng)
        ("sir ", "sỉ ", "sirr ", "sir "),
        ("Sir ", "Sỉ ", "Sirr ", "Sir "),
        ("car ", "cả ", "carr ", "car "),
        ("bar ", "bả ", "barr ", "bar "),
        ("var ", "vả ", "varr ", "var "),
        ("tar ", "tả ", "tarr ", "tar "),
        ("dir ", "dỉ ", "dirr ", "dir "),
        ("cur ", "củ ", "curr ", "cur "),
        ("par ", "pả ", "parr ", "par "),
        ("mar ", "mả ", "marr ", "mar "),
        ("xor ", "xỏ ", "xorr ", "xor "),
        ("ker ", "kẻ ", "kerr ", "ker "),
        ("or ", "ỏ ", "orr ", "or "),
        ("cos ", "có ", "coss ", "cos "),
        ("pos ", "pó ", "poss ", "pos "),
        ("bus ", "bú ", "buss ", "bus "),
        ("gas ", "gá ", "gass ", "gas "),
        ("has ", "há ", "hass ", "has "),
        ("his ", "hí ", "hiss ", "his "),
        ("os ", "ó ", "oss ", "os "),
        ("us ", "ú ", "uss ", "us "),
        ("this ", "thí ", "thiss ", "this "),
        ("list ", "lít ", "lisst ", "list "),
        ("last ", "lát ", "lasst ", "last "),
        ("if ", "ì ", "iff ", "if "),
        ("def ", "dè ", "deff ", "def "),
        ("ref ", "rè ", "reff ", "ref "),
        ("gif ", "gì ", "giff ", "gif "),
        ("tax ", "tã ", "taxx ", "tax "),
        ("box ", "bõ ", "boxx ", "box "),
        ("max ", "mã ", "maxx ", "max "),
        ("six ", "sĩ ", "sixx ", "six "),
        ("sex ", "sẽ ", "sexx ", "sex "),
        ("moon ", "môn ", "mooon ", "moon "),
        ("noon ", "nôn ", "nooon ", "noon "),
        ("teen ", "tên ", "teeen ", "teen "),
        ("been ", "bên ", "beeen ", "been "),
        ("queen ", "quên ", "queeen ", "queen "),
        ("too ", "tô ", "tooo ", "too "),
        ("bee ", "bê ", "beee ", "bee "),
        ("tree ", "trê ", "treee ", "tree "),
        ("cow ", "cơ ", "coww ", "cow "),
        ("now ", "nơ ", "noww ", "now "),
        ("how ", "hơ ", "howw ", "how "),
        ("low ", "lơ ", "loww ", "low "),
        ("row ", "rơ ", "roww ", "row "),
        ("bow ", "bơ ", "boww ", "bow "),
        ("dd ", "đ ", "ddd ", "dd "),
    ];

    for strategy in [
        IME_STRATEGY_PREEDIT,
        IME_STRATEGY_BACKSPACE_TYPE,
        IME_STRATEGY_FORWARD_AS_COMMIT,
    ] {
        for &(vi_in, vi_want, en_in, en_want) in pairs {
            let mut e_vi = Engine::new(opts_plain());
            let got_vi = type_stream_with_strategy(&mut e_vi, strategy, vi_in);
            assert_eq!(
                got_vi, vi_want,
                "VI default win failed (strategy={strategy}) `{vi_in}`"
            );

            let mut e_en = Engine::new(opts_plain());
            let got_en = type_stream_with_strategy(&mut e_en, strategy, en_in);
            assert_eq!(
                got_en, en_want,
                "EN UniKey undo failed (strategy={strategy}) `{en_in}`"
            );
        }
    }
}

#[test]
fn view3_scientific_dev_terms_continuations_and_unikey_habits() {
    let cases: &[(&str, &str)] = &[
        // Gõ tiếp sau undo UniKey (số nhiều, chia từ, dấu câu, camelCase)
        ("Sirr, ", "Sir, "),
        ("Sirr. ", "Sir. "),
        ("sirrs ", "sirs "),
        ("sirrss ", "sirs "),
        ("carrs ", "cars "),
        ("barrs ", "bars "),
        ("varrs ", "vars "),
        ("tassk ", "task "),
        ("casse ", "case "),
        ("basse ", "base "),
        ("tesst ", "test "),
        ("posst ", "post "),
        ("cosst ", "cost "),
        ("trusst ", "trust "),
        ("sysstem ", "system "),
        ("parrt ", "part "),
        ("parrse ", "parse "),
        ("corre ", "core "),
        ("sorrt ", "sort "),
        ("forrm ", "form "),
        ("marrk ", "mark "),
        ("roooms ", "rooms "),
        ("showwing ", "showing "),
        ("showw ", "show "),
        ("twwo ", "two "),
        ("wweb ", "web "),
        ("addd ", "add "),
        ("oddd ", "odd "),
        ("eoff ", "eof "),
        ("inff ", "inf "),
        ("ioss ", "ios "),
        ("ios ", "ios "),
        ("norr ", "nor "),
        ("nor ", "nor "),
        ("arccoss ", "arccos "),
        ("cossh ", "cosh "),
        ("mix ", "mix "),
        ("mixx ", "mix "),
        ("TexxtVN ", "TextVN "),
        ("VietNam ", "VietNam "),
        ("GreeenBox ", "GreenBox "),
        ("ScreeenSize ", "ScreenSize "),
        ("RoootDir ", "RootDir "),
        // Thuật ngữ & danh từ riêng Toán học / Khoa học / Lập trình
        ("room ", "room "),
        ("rooom ", "room "),
        ("seen ", "seen "),
        ("seeen ", "seen "),
        ("saw ", "saw "),
        ("saww ", "saw "),
        ("law ", "law "),
        ("laww ", "law "),
        ("raw ", "raw "),
        ("raww ", "raw "),
        ("Eulerr ", "Euler "),
        ("Fourier ", "Fourier "),
        ("Fourrier ", "Fourier "),
        ("FourrierTransform ", "FourierTransform "),
        ("Tensorr ", "Tensor "),
        ("TensorrShape ", "TensorShape "),
        ("Shaderr ", "Shader "),
        ("Solverr ", "Solver "),
        ("Scalarr ", "Scalar "),
        ("Molarr ", "Molar "),
        ("Keplerr ", "Kepler "),
        ("Dopplerr ", "Doppler "),
        ("Navierr ", "Navier "),
        ("Noetherr ", "Noether "),
        ("Cantorr ", "Cantor "),
        ("Pasteurr ", "Pasteur "),
        ("Schrodingerr ", "Schrodinger "),
        ("Bayess ", "Bayes "),
        ("BayessRule ", "BayesRule "),
        ("Galoiss ", "Galois "),
        ("Stokess ", "Stokes "),
        ("Linuss ", "Linus "),
        ("Sirri ", "Siri "),
        ("LaTeX ", "LaTeX "),
        ("BibTeX ", "BibTeX "),
        ("latexx ", "latex "),
        ("bibtexx ", "bibtex "),
        ("compilerr ", "compiler "),
        ("debuggerr ", "debugger "),
        ("lexerr ", "lexer "),
        // Đối chứng: tên nhà toán học có phụ âm đôi THẬT phải giữ nguyên vẹn
        ("Gauss ", "Gauss "),
        ("Poisson ", "Poisson "),
        ("Hausdorff ", "Hausdorff "),
        ("Kirchhoff ", "Kirchhoff "),
        ("Weierstrass ", "Weierstrass "),
    ];

    for &(typed, want) in cases {
        let mut e = Engine::new(opts_plain());
        let got = type_stream(&mut e, typed);
        assert_eq!(got, want, "E63 scientific/dev/continuation `{typed}`");
    }

    // Câu mix Việt-Anh thực tế theo thói quen UniKey cũ
    let mut e_mix = Engine::new(opts_plain());
    assert_eq!(
        type_stream(
            &mut e_mix,
            "Chaof Sirr, tooi ddang dungf Tensorr vaf Fourrier cho baif toasn coss(x) "
        ),
        "Chào Sir, tôi đang dùng Tensor và Fourier cho bài toán cos(x) "
    );

    // Thói quen gõ UniKey cổ điển: kiểu dấu Cũ (`hòa thủy`) & Mới (`hoà thuỷ`), đặt dấu muộn
    let mut opts_old = opts_plain();
    opts_old.diacritic_style = DiacriticStyle::Old;
    let mut e_old = Engine::new(opts_old);
    assert_eq!(
        type_stream(&mut e_old, "hoaf thuyr duongwf truongwf nguoiwf "),
        "hòa thủy dường trường người "
    );
    let mut opts_new = opts_plain();
    opts_new.diacritic_style = DiacriticStyle::New;
    let mut e_new = Engine::new(opts_new);
    assert_eq!(
        type_stream(&mut e_new, "hoaf thuyr duongw truongwf nguoiwf "),
        "hoà thuỷ dương trường người "
    );

    let mut opts_simple = opts_plain();
    opts_simple.method = Method::SimpleTelex;
    let mut e_simple = Engine::new(opts_simple);
    assert_eq!(
        type_stream(&mut e_simple, "Sirr carr varr coss buss Eulerr "),
        "Sir car var cos bus Euler "
    );

    // free_marking = false vẫn hoàn tác UniKey sạch
    let mut opts_strict = opts_plain();
    opts_strict.free_marking = false;
    let mut e_strict = Engine::new(opts_strict);
    assert_eq!(
        type_stream(&mut e_strict, "Sirr carr varr coss buss Eulerr "),
        "Sir car var cos bus Euler "
    );

    // suggest_vn_complete + suggest_space_complete KHÔNG được kéo dài từ vừa undo UniKey
    let mut opts_sug = opts_plain();
    opts_sug.suggest_vn_complete = true;
    opts_sug.suggest_space_complete = true;
    for (typed, want) in [
        ("Sirr ", "Sir "),
        ("Eulerr ", "Euler "),
        ("Fourrier ", "Fourier "),
        ("Tensorr ", "Tensor "),
        ("parrse ", "parse "),
        ("rooom ", "room "),
        ("showw ", "show "),
    ] {
        let mut e_sug = Engine::new(opts_sug.clone());
        assert_eq!(
            type_stream(&mut e_sug, typed),
            want,
            "space-complete `{typed}`"
        );
    }

    // Mọi dấu câu kết thúc sau `Sirr` / `Tassk` / `Eulerr` đều trả đúng từ EN
    for p in [
        ' ', '\n', '\t', ',', '.', '!', '?', ';', ':', ')', ']', '"', '\'', '/', '-',
    ] {
        let mut e_p = Engine::new(opts_plain());
        assert_eq!(
            type_stream(&mut e_p, &format!("Sirr{p}")),
            format!("Sir{p}"),
            "Sirr + punct {p:?}"
        );
        let mut e_e = Engine::new(opts_plain());
        assert_eq!(
            type_stream(&mut e_e, &format!("Eulerr{p}")),
            format!("Euler{p}"),
            "Eulerr + punct {p:?}"
        );
    }

    // VNI UniKey undo: CO22 -> CO2, a11 -> a1, e66 -> e6, u77 -> u7, a88 -> a8, d99 -> d9
    let mut opts_vni = opts_plain();
    opts_vni.method = Method::Vni;
    for (typed, want) in [
        ("CO22 ", "CO2 "),
        ("a11 ", "a1 "),
        ("e66 ", "e6 "),
        ("u77 ", "u7 "),
        ("a88 ", "a8 "),
        ("d99 ", "d9 "),
        ("toi6123450 ", "tôi "),
    ] {
        let mut e_v = Engine::new(opts_vni.clone());
        assert_eq!(type_stream(&mut e_v, typed), want, "VNI `{typed}`");
    }

    // Đổi dấu thanh liên tiếp giữa từ trong Telex (`asfrxjz` -> `a`, `tooisfrxjz` -> `tôi`)
    let mut e_sw = Engine::new(opts_plain());
    assert_eq!(type_stream(&mut e_sw, "asfrxjz tooisfrxjz "), "a tôi ");
}

#[test]
fn view4_cross_feature_math_emoji_and_bilingual_mix() {
    // Phối hợp Toán + Emoji `:shortcode:` (kể cả khi gõ kèm undo UniKey `:firre:`, `:starr:`)
    // + tiếng Việt + tiếng Anh trong cùng một phiên gõ!
    let mut e = Engine::new(opts_math(MathOutputMode::Normal));
    let out = type_stream(
        &mut e,
        "Chaof Sirr :smile: :fire: :firre: :star: :starr: x`^2 + \\alpha = =2+3= ",
    );
    assert_eq!(out, "Chào Sir 😄 🔥 🔥 ⭐ ⭐ x² + α = 5 ");

    // Trong math span `$...$`, dấu `:` KHÔNG mở emoji shortcode
    let mut e2 = Engine::new(opts_math(MathOutputMode::Latex));
    let math_colon = type_stream(&mut e2, "$f:fire:g$ :fire: ");
    assert_eq!(math_colon, "$f:fire:g$ 🔥 ");
}
