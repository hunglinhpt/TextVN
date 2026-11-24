// SPDX-License-Identifier: GPL-3.0-or-later
//! Lexicon khoa học 6 lĩnh vực — ≥1.500 entry tĩnh compile-time.
//!
//! - Trigger KHÔNG có `\` đầu (engine khớp từ gõ bắt đầu bằng `\`).
//! - Zero-alloc tra cứu: bảng phẳng sort + binary search (`lookup_exact`,
//!   `lookup_prefix`), alias quét tuyến tính pointer (≪ 300 µs p99).
//! - Mỗi nhóm lĩnh vực có test riêng; SI/nguyên tố/Hy Lạp có test per-entry
//!   (macro sinh `#[test]` theo dòng dữ liệu).

use crate::config::Discipline;
use std::sync::OnceLock;

/// Một entry lexicon — toàn bộ `&'static`, zero heap.
#[derive(Debug, Clone, Copy)]
pub struct MathEntry {
    pub trigger: &'static str,
    pub aliases: &'static [&'static str],
    pub latex: &'static str,
    pub typst: &'static str,
    pub unicode: &'static str,
    pub discipline: Discipline,
    pub description_vi: &'static str,
    pub description_en: &'static str,
    pub base_weight: u16,
}

/// Macro entry gọn: `e!(trigger, [aliases], discipline, latex, typst, unicode, vi, en, weight)`
macro_rules! e {
    ($trig:expr, [$($al:expr),* $(,)?], $disc:ident, $lat:expr, $typ:expr, $uni:expr, $dvi:expr, $den:expr, $w:expr) => {
        MathEntry {
            trigger: $trig,
            aliases: &[$($al),*],
            latex: $lat,
            typst: $typ,
            unicode: $uni,
            discipline: Discipline::$disc,
            description_vi: $dvi,
            description_en: $den,
            base_weight: $w,
        }
    };
}

// ============================================================
// 1. KÝ HIỆU MATH THUẦN (~180)
// ============================================================
const MATH_SYMBOLS: &[MathEntry] = &[
    e!(
        "infinity",
        ["inf", "infty", "vocuc", "vc", "oo"],
        PureMath,
        "\\infty",
        "infinity",
        "∞",
        "Vô cùng",
        "Infinity",
        100
    ),
    e!(
        "sum",
        ["tong", "tongcong"],
        PureMath,
        "\\sum",
        "sum",
        "∑",
        "Tổng (ký hiệu)",
        "Summation",
        100
    ),
    e!(
        "prod",
        ["tich", "tichcong"],
        PureMath,
        "\\prod",
        "product",
        "∏",
        "Tích (ký hiệu)",
        "Product",
        95
    ),
    e!(
        "coprod",
        ["coprod"],
        PureMath,
        "\\coprod",
        "coprod",
        "∐",
        "Cộng trực tiếp",
        "Coproduct",
        75
    ),
    e!(
        "integral",
        ["tichphan", "int", "tp"],
        PureMath,
        "\\int",
        "integral",
        "∫",
        "Tích phân",
        "Integral",
        100
    ),
    e!(
        "iint",
        ["tichphan2", "iint"],
        PureMath,
        "\\iint",
        "integral.double",
        "∬",
        "Tích phân kép",
        "Double integral",
        95
    ),
    e!(
        "iiint",
        ["tichphan3", "iiint"],
        PureMath,
        "\\iiint",
        "integral.triple",
        "∭",
        "Tích phân bội 3",
        "Triple integral",
        90
    ),
    e!(
        "oint",
        ["tichphanvong", "oint"],
        PureMath,
        "\\oint",
        "integral.cont",
        "∮",
        "Tích phân đường kín",
        "Contour integral",
        90
    ),
    e!(
        "oiint",
        ["oiint"],
        PureMath,
        "\\oiint",
        "integral.cont.double",
        "∯",
        "Tích phân mặt kín",
        "Closed surface integral",
        80
    ),
    e!(
        "frac",
        ["phanso", "ps"],
        PureMath,
        "\\frac{a}{b}",
        "frac(a, b)",
        "a/b",
        "Phân số",
        "Fraction",
        100
    ),
    e!(
        "dfrac",
        ["dfrac"],
        PureMath,
        "\\dfrac{a}{b}",
        "frac(a, b)",
        "a/b",
        "Phân số lớn",
        "Display fraction",
        85
    ),
    e!(
        "sqrt",
        ["can", "canbac2"],
        PureMath,
        "\\sqrt{x}",
        "sqrt(x)",
        "√x",
        "Căn bậc hai",
        "Square root",
        100
    ),
    e!(
        "cbrt",
        ["canbac3", "cbrt"],
        PureMath,
        "\\sqrt[3]{x}",
        "root(x, 3)",
        "∛x",
        "Căn bậc ba",
        "Cube root",
        90
    ),
    e!(
        "root4",
        ["canbac4"],
        PureMath,
        "\\sqrt[4]{x}",
        "root(x, 4)",
        "∜x",
        "Căn bậc tư",
        "Fourth root",
        85
    ),
    e!(
        "partial",
        ["daohamrieng", "part"],
        PureMath,
        "\\partial",
        "partial",
        "∂",
        "Đạo hàm riêng",
        "Partial derivative",
        100
    ),
    e!(
        "nabla",
        ["grad", "gradien"],
        PureMath,
        "\\nabla",
        "nabla",
        "∇",
        "Nabla / gradient",
        "Nabla / gradient",
        100
    ),
    e!(
        "forall",
        ["moi", "voimoi", "voimoinguoi"],
        PureMath,
        "\\forall",
        "forall",
        "∀",
        "Với mọi",
        "For all",
        95
    ),
    e!(
        "exists",
        ["tontai"],
        PureMath,
        "\\exists",
        "exists",
        "∃",
        "Tồn tại",
        "There exists",
        95
    ),
    e!(
        "nexists",
        ["khongtontai"],
        PureMath,
        "\\nexists",
        "not exists",
        "∄",
        "Không tồn tại",
        "Does not exist",
        85
    ),
    e!(
        "isin",
        ["thuoc", "in"],
        PureMath,
        "\\in",
        "in",
        "∈",
        "Thuộc",
        "Element of",
        100
    ),
    e!(
        "notin",
        ["khongthuoc"],
        PureMath,
        "\\notin",
        "not in",
        "∉",
        "Không thuộc",
        "Not element of",
        95
    ),
    e!(
        "ni",
        ["chua"],
        PureMath,
        "\\ni",
        "contains",
        "∋",
        "Chứa (phần tử)",
        "Contains element",
        85
    ),
    e!(
        "subset",
        ["con"],
        PureMath,
        "\\subset",
        "subset",
        "⊂",
        "Tập con thực",
        "Proper subset",
        95
    ),
    e!(
        "subseteq",
        ["conbang"],
        PureMath,
        "\\subseteq",
        "subset.eq",
        "⊆",
        "Tập con",
        "Subset or equal",
        95
    ),
    e!(
        "supset",
        ["cha"],
        PureMath,
        "\\supset",
        "supset",
        "⊃",
        "Tập cha thực",
        "Proper superset",
        85
    ),
    e!(
        "supseteq",
        ["chabang"],
        PureMath,
        "\\supseteq",
        "supset.eq",
        "⊇",
        "Tập cha",
        "Superset or equal",
        85
    ),
    e!(
        "cup",
        ["hop"],
        PureMath,
        "\\cup",
        "union",
        "∪",
        "Hợp",
        "Union",
        100
    ),
    e!(
        "cap",
        ["giao"],
        PureMath,
        "\\cap",
        "inter",
        "∩",
        "Giao",
        "Intersection",
        100
    ),
    e!(
        "emptyset",
        ["rong", "taprong"],
        PureMath,
        "\\emptyset",
        "emptyset",
        "∅",
        "Tập rỗng",
        "Empty set",
        100
    ),
    e!(
        "varnothing",
        ["varnothing"],
        PureMath,
        "\\varnothing",
        "emptyset",
        "Ø",
        "Tập rỗng (biến thể)",
        "Empty set variant",
        80
    ),
    e!(
        "setminus",
        ["hieu"],
        PureMath,
        "\\setminus",
        "minus",
        "∖",
        "Hiệu tập",
        "Set difference",
        85
    ),
    e!(
        "pm",
        ["congtru", "dungsai"],
        PureMath,
        "\\pm",
        "plus.minus",
        "±",
        "Cộng trừ",
        "Plus-minus",
        100
    ),
    e!(
        "mp",
        ["trucong"],
        PureMath,
        "\\mp",
        "minus.plus",
        "∓",
        "Trừ cộng",
        "Minus-plus",
        85
    ),
    e!(
        "times",
        ["nhan", "nhanvo"],
        PureMath,
        "\\times",
        "times",
        "×",
        "Nhân",
        "Multiplication",
        100
    ),
    e!(
        "div",
        ["chia", "chiachinh"],
        PureMath,
        "\\div",
        "div",
        "÷",
        "Chia",
        "Division",
        100
    ),
    e!(
        "cdot",
        ["nhanvohuong"],
        PureMath,
        "\\cdot",
        "dot",
        "·",
        "Nhân vô hướng",
        "Dot product",
        95
    ),
    e!(
        "ast",
        ["sao"],
        PureMath,
        "\\ast",
        "ast",
        "∗",
        "Sao / tích chập",
        "Asterisk / convolution",
        80
    ),
    e!(
        "star",
        ["star"],
        PureMath,
        "\\star",
        "star",
        "⋆",
        "Ngôi sao nhỏ",
        "Star operator",
        75
    ),
    e!(
        "neq",
        ["khac", "khacnhau"],
        PureMath,
        "\\neq",
        "op(!=)",
        "≠",
        "Không bằng",
        "Not equal",
        100
    ),
    e!(
        "approx",
        ["xapxi"],
        PureMath,
        "\\approx",
        "approx",
        "≈",
        "Xấp xỉ",
        "Approximately equal",
        100
    ),
    e!(
        "leq",
        ["nhohon", "nhebang"],
        PureMath,
        "\\leq",
        "op(<=)",
        "≤",
        "Nhỏ hơn hoặc bằng",
        "Less or equal",
        100
    ),
    e!(
        "geq",
        ["lonhon", "lonbang"],
        PureMath,
        "\\geq",
        "op(>=)",
        "≥",
        "Lớn hơn hoặc bằng",
        "Greater or equal",
        100
    ),
    e!(
        "ll",
        ["honho"],
        PureMath,
        "\\ll",
        "op(<<)",
        "≪",
        "Nhỏ hơn nhiều",
        "Much less",
        85
    ),
    e!(
        "gg",
        ["honlon"],
        PureMath,
        "\\gg",
        "op(>>)",
        "≫",
        "Lớn hơn nhiều",
        "Much greater",
        85
    ),
    e!(
        "sim",
        ["dongdang"],
        PureMath,
        "\\sim",
        "tilde",
        "∼",
        "Đồng dạng",
        "Similar",
        90
    ),
    e!(
        "simeq",
        ["dongdangbang"],
        PureMath,
        "\\simeq",
        "tilde.eq",
        "≃",
        "Đồng dạng bằng",
        "Asymptotically equal",
        80
    ),
    e!(
        "cong",
        ["bangnhauhh"],
        PureMath,
        "\\cong",
        "tilde.op(==)",
        "≅",
        "Bằng nhau (hình học)",
        "Congruent",
        90
    ),
    e!(
        "equiv",
        ["tuongduong"],
        PureMath,
        "\\equiv",
        "op(==)",
        "≡",
        "Tương đương",
        "Identical / equivalent",
        90
    ),
    e!(
        "propto",
        ["tilethuan", "tyle"],
        PureMath,
        "\\propto",
        "prop",
        "∝",
        "Tỉ lệ thuận",
        "Proportional to",
        95
    ),
    e!(
        "asymp",
        ["tiemcan"],
        PureMath,
        "\\asymp",
        "asymp",
        "≍",
        "Tương đương tiệm cận",
        "Asymptotically equivalent",
        80
    ),
    e!(
        "prec",
        ["truoctrong"],
        PureMath,
        "\\prec",
        "prec",
        "≺",
        "Trước (thứ tự)",
        "Precedes",
        80
    ),
    e!(
        "succ",
        ["sautrong"],
        PureMath,
        "\\succ",
        "succ",
        "≻",
        "Sau (thứ tự)",
        "Succeeds",
        80
    ),
    e!(
        "degree",
        ["do", "deg"],
        PureMath,
        "^\\circ",
        "degree",
        "°",
        "Độ (góc/nhiệt)",
        "Degree",
        100
    ),
    e!(
        "prime",
        ["phay"],
        PureMath,
        "'",
        "prime",
        "′",
        "Phẩy (đạo hàm/phút)",
        "Prime",
        90
    ),
    e!(
        "dprime",
        ["phay2"],
        PureMath,
        "''",
        "prime.double",
        "″",
        "Hai phẩy (giây)",
        "Double prime",
        85
    ),
    e!(
        "vec",
        ["vector", "vecto"],
        PureMath,
        "\\vec{a}",
        "vec(a)",
        "a⃗",
        "Vector (mũi tên)",
        "Vector arrow",
        100
    ),
    e!(
        "hat",
        ["mudau"],
        PureMath,
        "\\hat{x}",
        "hat(x)",
        "x̂",
        "Mũ trên",
        "Hat accent",
        90
    ),
    e!(
        "bar",
        ["gachduoitren"],
        PureMath,
        "\\bar{x}",
        "overline(x)",
        "x̄",
        "Gạch trên",
        "Bar accent",
        90
    ),
    e!(
        "dotop",
        ["chamtren"],
        PureMath,
        "\\dot{x}",
        "dot(x)",
        "ẋ",
        "Đạo hàm theo thời gian",
        "Time derivative",
        90
    ),
    e!(
        "ddotop",
        ["2cham"],
        PureMath,
        "\\ddot{x}",
        "ddot(x)",
        "ẍ",
        "Đạo hàm cấp 2",
        "Second derivative",
        85
    ),
    e!(
        "tildeop",
        ["sactren"],
        PureMath,
        "\\tilde{x}",
        "tilde(x)",
        "x̃",
        "Dấu ngã trên",
        "Tilde accent",
        85
    ),
    e!(
        "overline",
        ["gach", "gachtren"],
        PureMath,
        "\\overline{x}",
        "overline(x)",
        "x̄",
        "Gạch trên dài",
        "Overline",
        90
    ),
    e!(
        "underline",
        ["gachchan"],
        PureMath,
        "\\underline{x}",
        "underline(x)",
        "x̲",
        "Gạch dưới",
        "Underline",
        85
    ),
    e!(
        "widehat",
        ["murong"],
        PureMath,
        "\\widehat{x}",
        "widehat(x)",
        "x̂",
        "Mũ rộng",
        "Wide hat",
        80
    ),
    e!(
        "implies",
        ["suyra", "ketqua"],
        PureMath,
        "\\implies",
        "op(=>)",
        "⇒",
        "Suy ra",
        "Implies",
        100
    ),
    e!(
        "impliedby",
        ["duocsuyra"],
        PureMath,
        "\\impliedby",
        "op(<=)",
        "⇐",
        "Được suy ra",
        "Implied by",
        85
    ),
    e!(
        "iff",
        ["tuongduonglogic"],
        PureMath,
        "\\iff",
        "op(<=>)",
        "⇔",
        "Tương đương (logic)",
        "If and only if",
        95
    ),
    e!(
        "land",
        ["va", "and"],
        PureMath,
        "\\land",
        "and",
        "∧",
        "Và (logic)",
        "Logical AND",
        95
    ),
    e!(
        "lor",
        ["hoac", "or"],
        PureMath,
        "\\lor",
        "or",
        "∨",
        "Hoặc (logic)",
        "Logical OR",
        95
    ),
    e!(
        "neg",
        ["phudinh", "not"],
        PureMath,
        "\\neg",
        "not",
        "¬",
        "Phủ định",
        "Negation",
        95
    ),
    e!(
        "oplus",
        ["congtructiep", "xor"],
        PureMath,
        "\\oplus",
        "plus.circle",
        "⊕",
        "Cộng trực tiếp / XOR",
        "Direct sum / XOR",
        90
    ),
    e!(
        "ominus",
        ["op"],
        PureMath,
        "\\ominus",
        "minus.circle",
        "⊖",
        "Trừ trong vòng tròn",
        "Circled minus",
        80
    ),
    e!(
        "otimes",
        ["tichtructiep"],
        PureMath,
        "\\otimes",
        "times.circle",
        "⊗",
        "Tích tensor",
        "Tensor product",
        90
    ),
    e!(
        "oslash",
        ["oslash"],
        PureMath,
        "\\oslash",
        "div.circle",
        "⊘",
        "Chia trong vòng tròn",
        "Circled division",
        75
    ),
    e!(
        "odot",
        ["odot"],
        PureMath,
        "\\odot",
        "dot.circle",
        "⊙",
        "Chấm trong vòng tròn",
        "Circled dot",
        85
    ),
    e!(
        "perp",
        ["vuonggoc"],
        PureMath,
        "\\perp",
        "perp",
        "⊥",
        "Vuông góc",
        "Perpendicular",
        100
    ),
    e!(
        "parallel",
        ["songsong", "songso"],
        PureMath,
        "\\parallel",
        "parallel",
        "∥",
        "Song song",
        "Parallel",
        100
    ),
    e!(
        "nparallel",
        ["khongsongso"],
        PureMath,
        "\\nparallel",
        "not parallel",
        "∦",
        "Không song song",
        "Not parallel",
        80
    ),
    e!(
        "angle",
        ["goc"],
        PureMath,
        "\\angle",
        "angle",
        "∠",
        "Góc",
        "Angle",
        100
    ),
    e!(
        "measuredangle",
        ["gocdo"],
        PureMath,
        "\\measuredangle",
        "angle.measured",
        "∡",
        "Góc đo",
        "Measured angle",
        80
    ),
    e!(
        "triangle",
        ["tamgiac"],
        PureMath,
        "\\triangle",
        "triangle",
        "△",
        "Tam giác",
        "Triangle",
        90
    ),
    e!(
        "square",
        ["hinhvuong"],
        PureMath,
        "\\square",
        "square",
        "□",
        "Hình vuông",
        "Square",
        85
    ),
    e!(
        "diamond",
        ["hinhthoi"],
        PureMath,
        "\\diamond",
        "diamond",
        "◇",
        "Hình thoi",
        "Diamond",
        80
    ),
    e!(
        "circ",
        ["tron"],
        PureMath,
        "\\circ",
        "circle.s",
        "∘",
        "Hợp hàm",
        "Function composition",
        90
    ),
    e!(
        "bullet",
        ["bullet"],
        PureMath,
        "\\bullet",
        "bullet",
        "•",
        "Dấu chấm tròn",
        "Bullet",
        80
    ),
    e!(
        "ldots",
        ["3chamduoi"],
        PureMath,
        "\\ldots",
        "dots",
        "…",
        "Ba chấm dưới",
        "Lower ellipsis",
        90
    ),
    e!(
        "cdots",
        ["3chamgiua"],
        PureMath,
        "\\cdots",
        "dots.c",
        "⋯",
        "Ba chấm giữa",
        "Centered ellipsis",
        90
    ),
    e!(
        "vdots",
        ["3chamdoc"],
        PureMath,
        "\\vdots",
        "dots.v",
        "⋮",
        "Ba chấm dọc",
        "Vertical ellipsis",
        85
    ),
    e!(
        "ddots",
        ["3chamcheo"],
        PureMath,
        "\\ddots",
        "dots.down",
        "⋱",
        "Ba chấm chéo xuống",
        "Diagonal ellipsis",
        85
    ),
    e!(
        "aleph",
        ["aleph"],
        PureMath,
        "\\aleph",
        "aleph",
        "ℵ",
        "Số aleph",
        "Aleph",
        80
    ),
    e!(
        "beth",
        ["beth"],
        PureMath,
        "\\beth",
        "beth",
        "ℶ",
        "Số beth",
        "Beth",
        75
    ),
    e!(
        "hbar",
        ["hbar", "hbach"],
        PureMath,
        "\\hbar",
        "hbar",
        "ℏ",
        "Hằng số Planck rút gọn",
        "Reduced Planck constant",
        95
    ),
    e!(
        "Re",
        ["phanthuc"],
        PureMath,
        "\\Re",
        "Re",
        "ℜ",
        "Phần thực",
        "Real part",
        85
    ),
    e!(
        "Im",
        ["phanao"],
        PureMath,
        "\\Im",
        "Im",
        "ℑ",
        "Phần ảo",
        "Imaginary part",
        85
    ),
    e!(
        "wp",
        ["wp"],
        PureMath,
        "\\wp",
        "wp",
        "℘",
        "Hàm Weierstrass",
        "Weierstrass p",
        75
    ),
    e!(
        "ell",
        ["ell"],
        PureMath,
        "\\ell",
        "ell",
        "ℓ",
        "Chữ l viết tay",
        "Script lowercase L",
        90
    ),
    e!(
        "norm",
        ["chuanchuan", "norm"],
        PureMath,
        "\\|x\\|",
        "norm(x)",
        "‖x‖",
        "Chuẩn vector",
        "Norm",
        95
    ),
    e!(
        "abs",
        ["trituyetdoi", "abs"],
        PureMath,
        "|x|",
        "abs(x)",
        "|x|",
        "Giá trị tuyệt đối",
        "Absolute value",
        95
    ),
    e!(
        "floor",
        ["hamsan"],
        PureMath,
        "\\lfloor x \\rfloor",
        "floor(x)",
        "⌊x⌋",
        "Hàm sàn",
        "Floor",
        90
    ),
    e!(
        "ceil",
        ["hamtran"],
        PureMath,
        "\\lceil x \\rceil",
        "ceil(x)",
        "⌈x⌉",
        "Hàm trần",
        "Ceiling",
        90
    ),
    e!(
        "lim",
        ["gioihan", "lim"],
        PureMath,
        "\\lim",
        "lim",
        "lim",
        "Giới hạn",
        "Limit",
        100
    ),
    e!(
        "limsup",
        ["limtren"],
        PureMath,
        "\\limsup",
        "lim sup",
        "lim sup",
        "Giới hạn trên",
        "Limit superior",
        80
    ),
    e!(
        "liminf",
        ["limduoi"],
        PureMath,
        "\\liminf",
        "lim inf",
        "lim inf",
        "Giới hạn dưới",
        "Limit inferior",
        80
    ),
    e!(
        "sup",
        ["cuctran"],
        PureMath,
        "\\sup",
        "sup",
        "sup",
        "Cận trên đúng",
        "Supremum",
        85
    ),
    e!(
        "infimum",
        ["cucduoi"],
        PureMath,
        "\\inf",
        "inf",
        "inf",
        "Cận dưới đúng",
        "Infimum",
        85
    ),
    e!(
        "max",
        ["cucdai"],
        PureMath,
        "\\max",
        "max",
        "max",
        "Cực đại",
        "Maximum",
        90
    ),
    e!(
        "min",
        ["cuctieu"],
        PureMath,
        "\\min",
        "min",
        "min",
        "Cực tiểu",
        "Minimum",
        90
    ),
    e!(
        "sin",
        ["sin"],
        PureMath,
        "\\sin",
        "sin",
        "sin",
        "Sin",
        "Sine",
        100
    ),
    e!(
        "cos",
        ["cos"],
        PureMath,
        "\\cos",
        "cos",
        "cos",
        "Cos",
        "Cosine",
        100
    ),
    e!(
        "tan",
        ["tan"],
        PureMath,
        "\\tan",
        "tan",
        "tan",
        "Tan",
        "Tangent",
        100
    ),
    e!(
        "cot",
        ["cot"],
        PureMath,
        "\\cot",
        "cot",
        "cot",
        "Cotangent",
        "Cotangent",
        90
    ),
    e!(
        "sec",
        ["sec"],
        PureMath,
        "\\sec",
        "sec",
        "sec",
        "Secant",
        "Secant",
        90
    ),
    e!(
        "csc",
        ["csc"],
        PureMath,
        "\\csc",
        "csc",
        "csc",
        "Cosecant",
        "Cosecant",
        90
    ),
    e!(
        "arcsin",
        ["arcsin"],
        PureMath,
        "\\arcsin",
        "arcsin",
        "arcsin",
        "Arc sin",
        "Arcsine",
        90
    ),
    e!(
        "arccos",
        ["arccos"],
        PureMath,
        "\\arccos",
        "arccos",
        "arccos",
        "Arc cos",
        "Arccosine",
        90
    ),
    e!(
        "arctan",
        ["arctan"],
        PureMath,
        "\\arctan",
        "arctan",
        "arctan",
        "Arc tan",
        "Arctangent",
        90
    ),
    e!(
        "sinh",
        ["sinh"],
        PureMath,
        "\\sinh",
        "sinh",
        "sinh",
        "Sin hyperbolic",
        "Hyperbolic sine",
        85
    ),
    e!(
        "cosh",
        ["cosh"],
        PureMath,
        "\\cosh",
        "cosh",
        "cosh",
        "Cos hyperbolic",
        "Hyperbolic cosine",
        85
    ),
    e!(
        "tanh",
        ["tanh"],
        PureMath,
        "\\tanh",
        "tanh",
        "tanh",
        "Tan hyperbolic",
        "Hyperbolic tangent",
        85
    ),
    e!(
        "log",
        ["loga", "log"],
        PureMath,
        "\\log",
        "log",
        "log",
        "Logarit",
        "Logarithm",
        100
    ),
    e!(
        "ln",
        ["ln"],
        PureMath,
        "\\ln",
        "ln",
        "ln",
        "Logarit tự nhiên",
        "Natural logarithm",
        100
    ),
    e!(
        "log10",
        ["log10"],
        PureMath,
        "\\log_{10}",
        "log(10, x)",
        "log₁₀",
        "Logarit cơ số 10",
        "Log base 10",
        90
    ),
    e!(
        "log2",
        ["log2"],
        PureMath,
        "\\log_{2}",
        "log(2, x)",
        "log₂",
        "Logarit cơ số 2",
        "Log base 2",
        90
    ),
    e!(
        "exp",
        ["exp", "hammu"],
        PureMath,
        "\\exp",
        "exp",
        "exp",
        "Hàm mũ",
        "Exponential",
        90
    ),
    e!(
        "det",
        ["dinhthuc"],
        PureMath,
        "\\det",
        "det",
        "det",
        "Định thức",
        "Determinant",
        95
    ),
    e!(
        "dim",
        ["sochieu"],
        PureMath,
        "\\dim",
        "dim",
        "dim",
        "Chiều (không gian)",
        "Dimension",
        85
    ),
    e!(
        "gcd",
        ["ucln"],
        PureMath,
        "\\gcd",
        "gcd",
        "gcd",
        "Ước chung lớn nhất",
        "Greatest common divisor",
        90
    ),
    e!(
        "lcm",
        ["bcnn"],
        PureMath,
        "\\mathrm{lcm}",
        "lcm",
        "lcm",
        "Bội chung nhỏ nhất",
        "Least common multiple",
        90
    ),
    e!(
        "mod",
        ["modul", "du"],
        PureMath,
        "\\mod",
        "mod",
        "mod",
        "Modulo",
        "Modulo",
        85
    ),
    e!(
        "binom",
        ["tohop", "chinhac"],
        PureMath,
        "\\binom{n}{k}",
        "binom(n, k)",
        "C(n,k)",
        "Tổ hợp",
        "Binomial coefficient",
        95
    ),
    e!(
        "matrix",
        ["matran"],
        PureMath,
        "\\begin{matrix}a & b\\\\c & d\\end{matrix}",
        "mat(a, b; c, d)",
        "[a b; c d]",
        "Ma trận",
        "Matrix",
        90
    ),
    e!(
        "trace",
        ["vet", "trace"],
        PureMath,
        "\\mathrm{tr}",
        "tr",
        "tr",
        "Vết ma trận",
        "Matrix trace",
        85
    ),
    e!(
        "rank",
        ["hang", "rank"],
        PureMath,
        "\\mathrm{rank}",
        "rank",
        "rank",
        "Hạng ma trận",
        "Matrix rank",
        80
    ),
    e!(
        "transpose",
        ["chuyenvi", "cv"],
        PureMath,
        "A^T",
        "A^T",
        "Aᵀ",
        "Chuyển vị",
        "Transpose",
        90
    ),
    e!(
        "hermitian",
        ["lienhop", "hermitian"],
        PureMath,
        "A^*",
        "A^*",
        "A*",
        "Liên hợp Hermitian",
        "Hermitian conjugate",
        80
    ),
    e!(
        "inverse",
        ["nghichdao", "thuannghich"],
        PureMath,
        "A^{-1}",
        "A^-1",
        "A⁻¹",
        "Nghịch đảo",
        "Inverse",
        90
    ),
    e!(
        "euler",
        ["soeuler", "e"],
        PureMath,
        "e",
        "e",
        "e",
        "Số Euler (cơ số mũ)",
        "Euler's number",
        100
    ),
    // E58: entry "pi" trùng lặp đã XOÁ — bảng GREEK (g_pi) đã sở hữu trigger "pi"
    // (latex \pi, unicode π). Giữ 1 nguồn sự thật duy nhất cho binary search.
    e!(
        "tauconst",
        ["tauconst"],
        PureMath,
        "\\tau",
        "tau",
        "τ",
        "2π (tau)",
        "Tau (2π)",
        80
    ),
    e!(
        "phi_golden",
        ["sovang", "phivang"],
        PureMath,
        "\\varphi",
        "phi",
        "φ",
        "Tỉ lệ vàng",
        "Golden ratio",
        90
    ),
    e!(
        "homeomorphic",
        ["dongdiep"],
        PureMath,
        "\\cong",
        "equiv",
        "≅",
        "Đồng điều/đồng胚",
        "Homeomorphic",
        70
    ),
    e!(
        "mapsto",
        ["anhxa"],
        PureMath,
        "\\mapsto",
        "mapsto",
        "↦",
        "Ánh xạ tới",
        "Maps to",
        90
    ),
    e!(
        "to",
        ["to", "den"],
        PureMath,
        "\\to",
        "arrow.r",
        "→",
        "Mũi tên tới",
        "Right arrow (maps)",
        95
    ),
    e!(
        "gets",
        ["gets"],
        PureMath,
        "\\gets",
        "arrow.l",
        "←",
        "Mũi tên từ",
        "Left arrow",
        85
    ),
    e!(
        "leftrightarrow",
        ["haichieu"],
        PureMath,
        "\\leftrightarrow",
        "arrow.l.r",
        "↔",
        "Hai chiều",
        "Left-right arrow",
        85
    ),
    e!(
        "hookrightarrow",
        ["hook"],
        PureMath,
        "\\hookrightarrow",
        "arrow.r.hook",
        "↪",
        "Nhúng",
        "Hooked arrow",
        75
    ),
    e!(
        "Rightarrow",
        ["suyla"],
        PureMath,
        "\\Rightarrow",
        "op(=>)",
        "⇒",
        "Suy ra (đậm)",
        "Rightwards double arrow",
        90
    ),
    e!(
        "Rightarrow_equiv",
        ["tuongduonghd"],
        PureMath,
        "\\Leftrightarrow",
        "op(<=>)",
        "⇔",
        "Tương đương hai chiều",
        "Left-right double arrow",
        85
    ),
    e!(
        "circledR",
        ["rvongtron"],
        PureMath,
        "\\circledR",
        "reg",
        "®",
        "Nhãn hiệu đăng ký",
        "Registered trademark",
        70
    ),
    e!(
        "mathbb_r",
        ["sothuc", "R"],
        PureMath,
        "\\mathbb{R}",
        "RR",
        "ℝ",
        "Tập số thực",
        "Real numbers",
        95
    ),
    e!(
        "mathbb_n",
        ["sotunhien", "N"],
        PureMath,
        "\\mathbb{N}",
        "NN",
        "ℕ",
        "Tập số tự nhiên",
        "Natural numbers",
        95
    ),
    e!(
        "mathbb_z",
        ["songuyen", "Z"],
        PureMath,
        "\\mathbb{Z}",
        "ZZ",
        "ℤ",
        "Tập số nguyên",
        "Integers",
        95
    ),
    e!(
        "mathbb_q",
        ["sohuuti", "Q"],
        PureMath,
        "\\mathbb{Q}",
        "QQ",
        "ℚ",
        "Tập số hữu tỉ",
        "Rational numbers",
        95
    ),
    e!(
        "mathbb_c",
        ["sophuc", "C"],
        PureMath,
        "\\mathbb{C}",
        "CC",
        "ℂ",
        "Tập số phức",
        "Complex numbers",
        95
    ),
    e!(
        "mathbb_h",
        ["quaterrnion", "H"],
        PureMath,
        "\\mathbb{H}",
        "HH",
        "ℍ",
        "Tập quaternion",
        "Quaternions",
        85
    ),
    e!(
        "mathbb_f",
        ["truonghuuhan", "F"],
        PureMath,
        "\\mathbb{F}",
        "FF",
        "𝔽",
        "Trường hữu hạn",
        "Finite field",
        85
    ),
    e!(
        "mathcal_l",
        ["hamlagrange", "Lcal"],
        PureMath,
        "\\mathcal{L}",
        "cal(L)",
        "ℒ",
        "Chữ L hoa in nghiêng",
        "Script capital L",
        90
    ),
    e!(
        "mathcal_o",
        ["dophuctap", "O"],
        PureMath,
        "\\mathcal{O}",
        "cal(O)",
        "𝒪",
        "Ký hiệu Big-O",
        "Big-O notation",
        95
    ),
    e!(
        "mathcal_e",
        ["Ecal"],
        PureMath,
        "\\mathcal{E}",
        "cal(E)",
        "ℰ",
        "Chữ E in nghiêng",
        "Script capital E",
        85
    ),
    e!(
        "mathcal_f",
        ["faraday", "Fcal"],
        PureMath,
        "\\mathcal{F}",
        "cal(F)",
        "ℱ",
        "Chữ F in nghiêng",
        "Script capital F",
        85
    ),
    e!(
        "mathbb_e",
        ["EE"],
        PureMath,
        "\\mathbb{E}",
        "EE",
        "𝔼",
        "Kỳ vọng",
        "Expectation operator",
        95
    ),
    e!(
        "var",
        ["phuongsai", "Var"],
        PureMath,
        "\\mathrm{Var}(X)",
        "Var(X)",
        "Var(X)",
        "Phương sai",
        "Variance",
        90
    ),
    e!(
        "covariance",
        ["hiepphuongsai", "Cov"],
        PureMath,
        "\\mathrm{Cov}(X,Y)",
        "Cov(X, Y)",
        "Cov(X,Y)",
        "Hiệp phương sai",
        "Covariance",
        90
    ),
    e!(
        "corrcoef",
        ["tuongquan", "rhoXY"],
        PureMath,
        "\\rho_{X,Y}",
        "rho_(X, Y)",
        "ρₓᵧ",
        "Hệ số tương quan",
        "Correlation coefficient",
        90
    ),
    e!(
        "binomdist",
        ["phoinhithuc"],
        PureMath,
        "\\mathrm{B}(n,p)",
        "B(n, p)",
        "B(n,p)",
        "Phân phối nhị thức",
        "Binomial distribution",
        80
    ),
    e!(
        "poissondist",
        ["phoipoisson"],
        PureMath,
        "\\mathrm{Pois}(\\lambda)",
        "Pois(lambda)",
        "Pois(λ)",
        "Phân phối Poisson",
        "Poisson distribution",
        80
    ),
    e!(
        "normaldist",
        ["phoichuan"],
        PureMath,
        "N(\\mu, \\sigma^2)",
        "cal(N)(mu, sigma^2)",
        "𝒩(μ,σ²)",
        "Phân phối chuẩn",
        "Normal distribution",
        95
    ),
    e!(
        "exponentdist",
        ["phoimu"],
        PureMath,
        "\\mathrm{Exp}(\\lambda)",
        "Exp(lambda)",
        "Exp(λ)",
        "Phân phối mũ",
        "Exponential distribution",
        75
    ),
    e!(
        "uniformdist",
        ["phoideu"],
        PureMath,
        "\\mathrm{U}(a,b)",
        "U(a, b)",
        "U(a,b)",
        "Phân phối đều",
        "Uniform distribution",
        75
    ),
    e!(
        "chisq",
        ["phoikhibinhphuong"],
        PureMath,
        "\\chi^2",
        "chi^2",
        "χ²",
        "Chi bình phương",
        "Chi-squared",
        85
    ),
    e!(
        "tdist",
        ["phoistudent"],
        PureMath,
        "t",
        "t",
        "t",
        "Phân phối Student",
        "Student's t",
        80
    ),
    e!(
        "fdist",
        ["phoiF"],
        PureMath,
        "F",
        "F",
        "F",
        "Phân phối F",
        "F distribution",
        75
    ),
    e!(
        "median",
        ["trungvi"],
        PureMath,
        "\\mathrm{med}",
        "med",
        "med",
        "Trung vị",
        "Median",
        85
    ),
    e!(
        "mode",
        ["somodal"],
        PureMath,
        "\\mathrm{mode}",
        "mode",
        "mode",
        "Mode / số mode",
        "Mode",
        75
    ),
    e!(
        "quartile",
        ["tuphanvi"],
        PureMath,
        "Q_1",
        "Q_1",
        "Q₁",
        "Tứ phân vị 1",
        "First quartile",
        80
    ),
    e!(
        "entropy_h",
        ["entropy"],
        PureMath,
        "H(X)",
        "H(X)",
        "H(X)",
        "Entropy của X",
        "Entropy of X",
        85
    ),
    e!(
        "kl_divergence",
        ["dolechKL"],
        PureMath,
        "D_{KL}(P\\|Q)",
        "D_KL(P || Q)",
        "Dₖₗ(P‖Q)",
        "Độ lệch KL",
        "KL divergence",
        85
    ),
    e!(
        "gradient_descent",
        ["giamgradien"],
        PureMath,
        "\\theta \\leftarrow \\theta - \\alpha\\nabla J",
        "theta <- theta - alpha nabla J",
        "θ ← θ − α∇J",
        "Cập nhật gradient descent",
        "Gradient descent update",
        90
    ),
    e!(
        "softmax_fn",
        ["hamsoftmax"],
        PureMath,
        "\\mathrm{softmax}(z_i)",
        "softmax(z_i)",
        "softmax(zᵢ)",
        "Hàm softmax",
        "Softmax function",
        85
    ),
    e!(
        "relu_fn",
        ["hamrelu"],
        PureMath,
        "\\mathrm{ReLU}(x)",
        "ReLU(x)",
        "ReLU(x)",
        "Hàm kích hoạt ReLU",
        "ReLU activation",
        85
    ),
    e!(
        "sigmoid_fn",
        ["hamsigmoid"],
        PureMath,
        "\\sigma(x)",
        "sigma(x)",
        "σ(x)",
        "Hàm sigmoid",
        "Sigmoid function",
        85
    ),
    e!(
        "attention_fn",
        ["cochechuy"],
        PureMath,
        "\\mathrm{Attention}(Q,K,V)",
        "Attention(Q, K, V)",
        "Attention(Q,K,V)",
        "Cơ chế chú ý",
        "Attention mechanism",
        85
    ),
    e!(
        "convolution",
        ["tichchap"],
        PureMath,
        "(f*g)(t)",
        "conv(f, g)",
        "(f∗g)(t)",
        "Tích chập",
        "Convolution",
        85
    ),
    e!(
        "fourier",
        ["biendoifourier"],
        PureMath,
        "\\hat{f}(\\omega)",
        "hat(f)(omega)",
        "f̂(ω)",
        "Biến đổi Fourier",
        "Fourier transform",
        90
    ),
    e!(
        "laplace_tr",
        ["biendoilaplace"],
        PureMath,
        "\\mathcal{L}\\{f\\}",
        "Laplace(f)",
        "ℒ{f}",
        "Biến đổi Laplace",
        "Laplace transform",
        90
    ),
    e!(
        "complex_i",
        ["soao", "i"],
        PureMath,
        "i",
        "i",
        "i",
        "Đơn vị ảo",
        "Imaginary unit",
        95
    ),
];

// ============================================================
// 2. CHỮ HY LẠP ĐẦY ĐỦ (48) — macro + test per-entry
// ============================================================
macro_rules! greek_table {
    (
        lower: [ $( $lid:ident : ($ltrig:literal, $llat:literal, $ltyp:literal, $luni:literal, $lvi:literal) ),* $(,)? ],
        upper: [ $( $uid:ident : ($utrig:literal, $ulat:literal, $utyp:literal, $uuni:literal, $uvi:literal) ),* $(,)? ]
    ) => {
        const GREEK: &[MathEntry] = &[
            $( e!($ltrig, [], PureMath, $llat, $ltyp, $luni, concat!("Chữ Hy Lạp ", $lvi), concat!("Greek letter ", $lvi), 100), )*
            $( e!($utrig, [], PureMath, $ulat, $utyp, $uuni, concat!("Chữ Hy Lạp hoa ", $uvi), concat!("Greek capital ", $uvi), 90), )*
        ];
        #[cfg(test)]
        mod greek_tests {
            use super::*;
            #[test]
            fn greek_count_48() {
                assert_eq!(GREEK.len(), 48, "24 thường + 24 hoa");
            }
            $( #[test] fn $lid() {
                let en = crate::lexicon::lookup_exact($ltrig).unwrap();
                assert_eq!(en.unicode, $luni);
                assert_eq!(en.latex, $llat);
                assert!(!en.typst.is_empty());
                assert_eq!(en.discipline, Discipline::PureMath);
            } )*
            $( #[test] fn $uid() {
                let en = crate::lexicon::lookup_exact($utrig).unwrap();
                assert_eq!(en.unicode, $uuni);
                assert_eq!(en.latex, $ulat);
                assert_eq!(en.discipline, Discipline::PureMath);
            } )*
        }
    };
}

greek_table! {
    lower: [
        g_alpha: ("alpha", "\\alpha", "alpha", "α", "alpha"),
        g_beta: ("beta", "\\beta", "beta", "β", "beta"),
        g_gamma: ("gamma", "\\gamma", "gamma", "γ", "gamma"),
        g_delta: ("delta", "\\delta", "delta", "δ", "delta"),
        g_epsilon: ("epsilon", "\\epsilon", "epsilon.alt", "ε", "epsilon"),
        g_varepsilon: ("varepsilon", "\\varepsilon", "epsilon", "ϵ", "epsilon biến thể"),
        g_zeta: ("zeta", "\\zeta", "zeta", "ζ", "zeta"),
        g_eta: ("eta", "\\eta", "eta", "η", "eta"),
        g_theta: ("theta", "\\theta", "theta", "θ", "theta"),
        g_vartheta: ("vartheta", "\\vartheta", "theta.alt", "ϑ", "theta biến thể"),
        g_iota: ("iota", "\\iota", "iota", "ι", "iota"),
        g_kappa: ("kappa", "\\kappa", "kappa", "κ", "kappa"),
        g_lambda: ("lambda", "\\lambda", "lambda", "λ", "lambda"),
        g_mu: ("mu", "\\mu", "mu", "µ", "mu"),
        g_nu: ("nu", "\\nu", "nu", "ν", "nu"),
        g_xi: ("xi", "\\xi", "xi", "ξ", "xi"),
        g_omicron: ("omicron", "o", "o", "ο", "omicron"),
        g_pi: ("pi", "\\pi", "pi", "π", "pi"),
        g_varpi: ("varpi", "\\varpi", "pi.alt", "ϖ", "pi biến thể"),
        g_rho: ("rho", "\\rho", "rho", "ρ", "rho"),
        g_varrho: ("varrho", "\\varrho", "rho.alt", "ϱ", "rho biến thể"),
        g_sigma: ("sigma", "\\sigma", "sigma", "σ", "sigma"),
        g_varsigma: ("varsigma", "\\varsigma", "sigma.s", "ς", "sigma cuối từ"),
        g_tau: ("tau", "\\tau", "tau", "τ", "tau"),
    ],
    upper: [
        g_alpha_u: ("Alpha", "A", "A", "Α", "alpha"),
        g_beta_u: ("Beta", "B", "B", "Β", "beta"),
        g_gamma_u: ("Gamma", "\\Gamma", "Gamma", "Γ", "gamma"),
        g_delta_u: ("Delta", "\\Delta", "Delta", "Δ", "delta"),
        g_epsilon_u: ("Epsilon", "E", "E", "Ε", "epsilon"),
        g_zeta_u: ("Zeta", "Z", "Z", "Ζ", "zeta"),
        g_eta_u: ("Eta", "H", "H", "Η", "eta"),
        g_theta_u: ("Theta", "\\Theta", "Theta", "Θ", "theta"),
        g_iota_u: ("Iota", "I", "I", "Ι", "iota"),
        g_kappa_u: ("Kappa", "K", "K", "Κ", "kappa"),
        g_lambda_u: ("Lambda", "\\Lambda", "Lambda", "Λ", "lambda"),
        g_mu_u: ("Mu", "M", "M", "Μ", "mu"),
        g_nu_u: ("Nu", "N", "N", "Ν", "nu"),
        g_xi_u: ("Xi", "\\Xi", "Xi", "Ξ", "xi"),
        g_omicron_u: ("Omicron", "O", "O", "Ο", "omicron"),
        g_pi_u: ("Pi", "\\Pi", "Pi", "Π", "pi"),
        g_rho_u: ("Rho", "P", "P", "Ρ", "rho"),
        g_sigma_u: ("Sigma", "\\Sigma", "Sigma", "Σ", "sigma"),
        g_tau_u: ("Tau", "T", "T", "Τ", "tau"),
        g_upsilon_u: ("Upsilon", "\\Upsilon", "Upsilon", "Υ", "upsilon"),
        g_phi_u: ("Phi", "\\Phi", "Phi", "Φ", "phi"),
        g_chi_u: ("Chi", "X", "X", "Χ", "chi"),
        g_psi_u: ("Psi", "\\Psi", "Psi", "Ψ", "psi"),
        g_omega_u: ("Omega", "\\Omega", "Omega", "Ω", "omega"),
    ]
}

// ============================================================
// 3. TIỀN TỐ SI × ĐƠN VỊ SI (20 × 28 = 560) — macro + test per-entry
// ============================================================
macro_rules! si_group {
    ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal, $units:tt) => {
        // Bảng sinh tự động: tên module/const giữ nguyên để khớp ký hiệu vật lý (si_M, u_meter...).
        #[allow(non_snake_case, non_upper_case_globals)]
        mod $pid {
            use super::{MathEntry, Discipline};
            si_group!(@consts $psym, $plat, $pen, $pvi, $units);
        }
    };
    (@consts $psym:literal, $plat:literal, $pen:literal, $pvi:literal, [ $( ($uid:ident, $usym:literal, $ulat:literal, $uen:literal, $uvi:literal) ),* $(,)? ]) => {
        $( pub const $uid: MathEntry = e!(
            concat!($psym, $usym), [], PhysicsMech,
            concat!("\\mathrm{", $plat, $ulat, "}"),
            concat!($psym, $usym),
            concat!($psym, $usym),
            concat!("Đơn vị ", $pvi, " ", $uvi),
            concat!($pen, " ", $uen),
            60
        ); )*
        pub const ENTRIES: &[MathEntry] = &[ $($uid),* ];
        si_group!(@tests $psym, [ $( ($uid, $usym) ),* ]);
    };
    (@tests $psym:literal, [ $( ($uid:ident, $usym:literal) ),* $(,)? ]) => {
        #[cfg(test)]
        mod tests {
            use crate::config::Discipline;
            // E58: assert trên const CỦA NHÓM (định dạng/sinh đúng) — KHÔNG dùng
            // lookup_exact toàn cục: 3 combo pico ("pm"±, "pH" hoá học, "ps" phanso)
            // nhường nghĩa cho entry quan trọng hơn qua SI_EXCLUDED ở all_slices,
            // nên lookup toàn cục không còn đảm bảo cho mọi combo.
            $( #[test]
            fn $uid() {
                let trig = concat!($psym, $usym);
                let en = &super::$uid;
                assert_eq!(en.trigger, trig);
                assert!(!en.unicode.is_empty(), "{trig} unicode rỗng");
                assert!(en.latex.starts_with("\\mathrm{"), "{trig} latex sai định dạng");
                assert!(en.latex.ends_with('}'));
                assert_eq!(en.discipline, Discipline::PhysicsMech);
                assert!(!en.description_vi.is_empty());
                assert!(!en.description_en.is_empty());
                assert!(en.base_weight > 0);
            } )*
        }
    };
}

macro_rules! si_table {
    (
        prefixes: [ $( ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal) ),* $(,)? ],
        units: $units:tt
    ) => {
        $( si_group!($pid, $psym, $plat, $pen, $pvi, $units); )*
        pub const SI_TABLES: &[&[MathEntry]] = &[ $( &$pid::ENTRIES ),* ];
        #[cfg(test)]
        mod si_tests {
            use super::SI_TABLES;
            #[test]
            fn si_total_560() {
                let n: usize = SI_TABLES.iter().map(|g| g.len()).sum();
                assert_eq!(n, 20 * 28, "20 tiền tố × 28 đơn vị");
            }
        }
    };
}

si_table! {
    prefixes: [
        (si_da, "da", "\\deca", "Deca", "đềca"),
        (si_h, "h", "\\hecto", "Hecto", "hêctô"),
        (si_k, "k", "\\kilo", "Kilo", "kilo"),
        (si_M, "M", "\\mega", "Mega", "mega"),
        (si_G, "G", "\\giga", "Giga", "giga"),
        (si_T, "T", "\\tera", "Tera", "tera"),
        (si_P, "P", "\\peta", "Peta", "peta"),
        (si_E, "E", "\\exa", "Exa", "exa"),
        (si_Z, "Z", "\\zetta", "Zetta", "zetta"),
        (si_Y, "Y", "\\yotta", "Yotta", "yotta"),
        (si_d, "d", "\\deci", "Deci", "đềxi"),
        (si_c, "c", "\\centi", "Centi", "xênti"),
        (si_m, "m", "\\milli", "Milli", "milli"),
        (si_mu, "µ", "\\mu ", "Micro", "micro"),
        (si_n, "n", "\\nano", "Nano", "nano"),
        // E58: combo pico "pm"/"pH"/"ps" trùng trigger quan trọng hơn (±/pH hoá học/
        // phanso §3.4) — hai bên giữ nguyên dữ liệu, bên thắng được xử ở all_slices
        // (SI_EXCLUDED) để mọi bảng tra cứu chỉ thấy NGHĨA quan trọng; format nhóm
        // vẫn được assert đủ qua test per-entry (dùng const, không dùng lookup toàn cục).
        (si_p, "p", "\\pico", "Pico", "pico"),
        (si_f, "f", "\\femto", "Femto", "femto"),
        (si_a, "a", "\\atto", "Atto", "atto"),
        (si_z, "z", "\\zepto", "Zepto", "zepto"),
        (si_y, "y", "\\yocto", "Yocto", "yocto"),
    ],
    units: [
        (u_meter, "m", "m", "meter", "mét"),
        (u_gram, "g", "g", "gram", "gam"),
        (u_second, "s", "s", "second", "giây"),
        (u_ampere, "A", "A", "Ampere", "ampe"),
        (u_kelvin, "K", "K", "Kelvin", "kelvin"),
        (u_mole, "mol", "mol", "Mole", "mol"),
        (u_candela, "cd", "cd", "Candela", "candela"),
        (u_hertz, "Hz", "Hz", "Hertz", "hertz"),
        (u_newton, "N", "N", "Newton", "niuton"),
        (u_pascal, "Pa", "Pa", "Pascal", "pascal"),
        (u_joule, "J", "J", "Joule", "jun"),
        (u_watt, "W", "W", "Watt", "oát"),
        (u_coulomb, "C", "C", "Coulomb", "culông"),
        (u_volt, "V", "V", "Volt", "vôn"),
        (u_farad, "F", "F", "Farad", "farad"),
        (u_ohm, "Ω", "\\Omega", "Ohm", "ôm"),
        (u_siemens, "S", "S", "Siemens", "siemens"),
        (u_weber, "Wb", "Wb", "Weber", "vebe"),
        (u_tesla, "T", "T", "Tesla", "tesla"),
        (u_henry, "H", "H", "Henry", "henry"),
        (u_lumen, "lm", "lm", "Lumen", "lumen"),
        (u_lux, "lx", "lx", "Lux", "lux"),
        (u_becquerel, "Bq", "Bq", "Becquerel", "beckerel"),
        (u_gray, "Gy", "Gy", "Gray", "gray"),
        (u_sievert, "Sv", "Sv", "Sievert", "sievert"),
        (u_katal, "kat", "kat", "Katal", "katal"),
        (u_radian, "rad", "rad", "Radian", "radian"),
        (u_steradian, "sr", "sr", "Steradian", "steradian"),
    ]
}

/// Alias ASCII cho hàng micro (µ khó gõ) — 28 entry riêng.
const SI_MICRO_ASCII: &[MathEntry] = &[
    e!(
        "um",
        [],
        PhysicsMech,
        "\\mathrm{\\mu m}",
        "µm",
        "µm",
        "Đơn vị micro mét",
        "Micrometer",
        80
    ),
    e!(
        "ug",
        [],
        PhysicsMech,
        "\\mathrm{\\mu g}",
        "µg",
        "µg",
        "Đơn vị micro gam",
        "Microgram",
        80
    ),
    e!(
        "us",
        [],
        PhysicsMech,
        "\\mathrm{\\mu s}",
        "µs",
        "µs",
        "Đơn vị micro giây",
        "Microsecond",
        80
    ),
    e!(
        "uA",
        [],
        PhysicsMech,
        "\\mathrm{\\mu A}",
        "µA",
        "µA",
        "Đơn vị micro ampe",
        "Microampere",
        80
    ),
    e!(
        "uK",
        [],
        PhysicsMech,
        "\\mathrm{\\mu K}",
        "µK",
        "µK",
        "Đơn vị micro kelvin",
        "Microkelvin",
        75
    ),
    e!(
        "umol",
        [],
        PhysicsMech,
        "\\mathrm{\\mu mol}",
        "µmol",
        "µmol",
        "Đơn vị micro mol",
        "Micromole",
        75
    ),
    e!(
        "uHz",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Hz}",
        "µHz",
        "µHz",
        "Đơn vị micro hertz",
        "Microhertz",
        75
    ),
    e!(
        "uN",
        [],
        PhysicsMech,
        "\\mathrm{\\mu N}",
        "µN",
        "µN",
        "Đơn vị micro niuton",
        "Micronewton",
        75
    ),
    e!(
        "uPa",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Pa}",
        "µPa",
        "µPa",
        "Đơn vị micro pascal",
        "Micropascal",
        75
    ),
    e!(
        "uJ",
        [],
        PhysicsMech,
        "\\mathrm{\\mu J}",
        "µJ",
        "µJ",
        "Đơn vị micro jun",
        "Microjoule",
        75
    ),
    e!(
        "uW",
        [],
        PhysicsMech,
        "\\mathrm{\\mu W}",
        "µW",
        "µW",
        "Đơn vị micro oát",
        "Microwatt",
        80
    ),
    e!(
        "uC",
        [],
        PhysicsMech,
        "\\mathrm{\\mu C}",
        "µC",
        "µC",
        "Đơn vị micro culông",
        "Microcoulomb",
        80
    ),
    e!(
        "uV",
        [],
        PhysicsMech,
        "\\mathrm{\\mu V}",
        "µV",
        "µV",
        "Đơn vị micro vôn",
        "Microvolt",
        80
    ),
    e!(
        "uF",
        ["uf"],
        ElectricalEmbedded,
        "\\mu F",
        "µF",
        "µF",
        "Đơn vị micro farad (tụ điện)",
        "Microfarad",
        90
    ),
    e!(
        "uohm",
        [],
        PhysicsMech,
        "\\mathrm{\\mu \\Omega}",
        "µΩ",
        "µΩ",
        "Đơn vị micro ôm",
        "Microohm",
        75
    ),
    e!(
        "uS",
        [],
        PhysicsMech,
        "\\mathrm{\\mu S}",
        "µS",
        "µS",
        "Đơn vị micro siemens",
        "Microsiemens",
        75
    ),
    e!(
        "uWb",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Wb}",
        "µWb",
        "µWb",
        "Đơn vị micro vebe",
        "Microweber",
        70
    ),
    e!(
        "uT",
        [],
        PhysicsMech,
        "\\mathrm{\\mu T}",
        "µT",
        "µT",
        "Đơn vị micro tesla",
        "Microtesla",
        75
    ),
    e!(
        "uH",
        ["uh"],
        ElectricalEmbedded,
        "\\mathrm{\\mu H}",
        "µH",
        "µH",
        "Đơn vị micro henry (cuộn cảm)",
        "Microhenry",
        85
    ),
    e!(
        "ulm",
        [],
        PhysicsMech,
        "\\mathrm{\\mu lm}",
        "µlm",
        "µlm",
        "Đơn vị micro lumen",
        "Microlumen",
        70
    ),
    e!(
        "ulux",
        [],
        PhysicsMech,
        "\\mathrm{\\mu lx}",
        "µlx",
        "µlx",
        "Đơn vị micro lux",
        "Microlux",
        70
    ),
    e!(
        "uBq",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Bq}",
        "µBq",
        "µBq",
        "Đơn vị micro beckerel",
        "Microbecquerel",
        70
    ),
    e!(
        "uGy",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Gy}",
        "µGy",
        "µGy",
        "Đơn vị micro gray",
        "Microgray",
        70
    ),
    e!(
        "uSv",
        [],
        PhysicsMech,
        "\\mathrm{\\mu Sv}",
        "µSv",
        "µSv",
        "Đơn vị micro sievert",
        "Microsievert",
        70
    ),
    e!(
        "ukat",
        [],
        PhysicsMech,
        "\\mathrm{\\mu kat}",
        "µkat",
        "µkat",
        "Đơn vị micro katal",
        "Microkatal",
        70
    ),
    e!(
        "urad",
        [],
        PhysicsMech,
        "\\mathrm{\\mu rad}",
        "µrad",
        "µrad",
        "Đơn vị micro radian",
        "Microradian",
        70
    ),
    e!(
        "usr",
        [],
        PhysicsMech,
        "\\mathrm{\\mu sr}",
        "µsr",
        "µsr",
        "Đơn vị micro steradian",
        "Microsteradian",
        70
    ),
];

// ============================================================
// 4. ĐƠN VỊ NGOÀI SI + THƯỜNG DÙNG (~40)
// ============================================================
const UNITS_MISC: &[MathEntry] = &[
    e!(
        "bar_unit",
        ["apluatbar"],
        PhysicsMech,
        "\\mathrm{bar}",
        "bar",
        "bar",
        "Áp suất bar",
        "Bar (pressure)",
        85
    ),
    e!(
        "atm",
        ["atmosphere"],
        PhysicsMech,
        "\\mathrm{atm}",
        "atm",
        "atm",
        "Áp suất khí quyển",
        "Atmosphere",
        85
    ),
    e!(
        "mmHg",
        ["mmhg", "thuyetngan"],
        PhysicsMech,
        "\\mathrm{mmHg}",
        "mm Hg",
        "mmHg",
        "Milimet thủy ngân",
        "Millimeters of mercury",
        85
    ),
    e!(
        "psi",
        ["psi"],
        PhysicsMech,
        "\\mathrm{psi}",
        "psi",
        "psi",
        "Pound trên inch vuông",
        "Pounds per square inch",
        80
    ),
    e!(
        "eV",
        ["ev", "electronvolt"],
        PhysicsMech,
        "\\mathrm{eV}",
        "eV",
        "eV",
        "Electronvolt",
        "Electronvolt",
        90
    ),
    e!(
        "MeV",
        ["mev"],
        PhysicsMech,
        "\\mathrm{MeV}",
        "MeV",
        "MeV",
        "Megaelectronvolt",
        "Megaelectronvolt",
        80
    ),
    e!(
        "GeV",
        ["gev"],
        PhysicsMech,
        "\\mathrm{GeV}",
        "GeV",
        "GeV",
        "Gigaelectronvolt",
        "Gigaelectronvolt",
        80
    ),
    e!(
        "TeV",
        ["tev"],
        PhysicsMech,
        "\\mathrm{TeV}",
        "TeV",
        "TeV",
        "Teraelectronvolt",
        "Teraelectronvolt",
        75
    ),
    e!(
        "cal",
        ["calo"],
        PhysicsMech,
        "\\mathrm{cal}",
        "cal",
        "cal",
        "Calo",
        "Calorie",
        80
    ),
    e!(
        "kcal",
        ["kilocalo"],
        PhysicsMech,
        "\\mathrm{kcal}",
        "kcal",
        "kcal",
        "Kilocalo",
        "Kilocalorie",
        80
    ),
    e!(
        "BTU",
        ["btu"],
        PhysicsMech,
        "\\mathrm{BTU}",
        "BTU",
        "BTU",
        "British Thermal Unit",
        "British Thermal Unit",
        75
    ),
    e!(
        "hp",
        ["maluc"],
        PhysicsMech,
        "\\mathrm{hp}",
        "hp",
        "hp",
        "Mã lực",
        "Horsepower",
        80
    ),
    e!(
        "rpm",
        ["vongphut"],
        PhysicsMech,
        "\\mathrm{rpm}",
        "rpm",
        "rpm",
        "Vòng trên phút",
        "Revolutions per minute",
        85
    ),
    e!(
        "kt",
        ["hainau"],
        PhysicsMech,
        "\\mathrm{kt}",
        "kt",
        "kt",
        "Hải lý / knot",
        "Knot",
        70
    ),
    e!(
        "lb",
        ["pound"],
        PhysicsMech,
        "\\mathrm{lb}",
        "lb",
        "lb",
        "Pound",
        "Pound",
        75
    ),
    e!(
        "kgf",
        ["kiloniutluc"],
        PhysicsMech,
        "\\mathrm{kgf}",
        "kgf",
        "kgf",
        "Kilogram lực",
        "Kilogram-force",
        75
    ),
    e!(
        "c_speed",
        ["vantocanhsang"],
        PhysicsMech,
        "c",
        "c",
        "c",
        "Tốc độ ánh sáng",
        "Speed of light",
        95
    ),
    e!(
        "celsius",
        ["doC", "degC"],
        PhysicsMech,
        "^\\circ\\mathrm{C}",
        "deg C",
        "°C",
        "Độ Celsius",
        "Degrees Celsius",
        90
    ),
    e!(
        "fahrenheit",
        ["doF", "degF"],
        PhysicsMech,
        "^\\circ\\mathrm{F}",
        "deg F",
        "°F",
        "Độ Fahrenheit",
        "Degrees Fahrenheit",
        85
    ),
    e!(
        "molL",
        ["nongdomol"],
        ChemistryBio,
        "\\mathrm{mol/L}",
        "mol / L",
        "mol/L",
        "Nồng độ mol trên lít",
        "Molarity",
        85
    ),
    e!(
        "molar_M",
        ["nongdoM"],
        ChemistryBio,
        "\\mathrm{M}",
        "M",
        "M",
        "Molar (nồng độ)",
        "Molar concentration",
        85
    ),
    e!(
        "millimolar",
        ["nongdomM"],
        ChemistryBio,
        "\\mathrm{mM}",
        "m M",
        "mM",
        "Millimolar",
        "Millimolar",
        80
    ),
    e!(
        "micromolar",
        ["nongdouM"],
        ChemistryBio,
        "\\mathrm{\\mu M}",
        "µ M",
        "µM",
        "Micromolar",
        "Micromolar",
        75
    ),
    e!(
        "ppm",
        ["phantrieu"],
        QuantFinance,
        "\\mathrm{ppm}",
        "ppm",
        "ppm",
        "Phần trên triệu",
        "Parts per million",
        85
    ),
    e!(
        "ppb",
        ["phannghinty"],
        QuantFinance,
        "\\mathrm{ppb}",
        "ppb",
        "ppb",
        "Phần trên tỷ",
        "Parts per billion",
        80
    ),
    e!(
        "percent",
        ["phantram", "%"],
        QuantFinance,
        "\\%",
        "percent",
        "%",
        "Phần trăm",
        "Percent",
        100
    ),
    e!(
        "permille",
        ["phannghin"],
        QuantFinance,
        "\\permil",
        "per mille",
        "‰",
        "Phần nghìn",
        "Per mille",
        90
    ),
    e!(
        "basis_point",
        ["diemcoban", "bps"],
        QuantFinance,
        "\\mathrm{bps}",
        "bps",
        "bps",
        "Điểm cơ bản",
        "Basis point",
        85
    ),
    e!(
        "px",
        ["diemanh"],
        ComputerScienceAi,
        "\\mathrm{px}",
        "px",
        "px",
        "Pixel",
        "Pixel",
        80
    ),
    e!(
        "dpi",
        ["dpi"],
        ComputerScienceAi,
        "\\mathrm{dpi}",
        "dpi",
        "dpi",
        "Điểm trên inch",
        "Dots per inch",
        80
    ),
    e!(
        "pt_unit",
        ["diemchu"],
        ComputerScienceAi,
        "\\mathrm{pt}",
        "pt",
        "pt",
        "Point (typography)",
        "Point (typography)",
        75
    ),
];

// ============================================================
// 5. NGUYÊN TỐ HOÁ HỌC (118) — macro + test per-entry
// ============================================================
macro_rules! element_table {
    ( $( $id:ident : ($sym:literal, $name:literal, $vi:literal) ),* $(,)? ) => {
        const ELEMENTS: &[MathEntry] = &[
            // E58: alias = tên VI ngăn trùng ký hiệu nguyên tố với trigger SI
            // ("Mg" megagram, "Pm" petameter, "Tm" terameter, "Re" Reynolds, "Es"
            // exa-second, "Ts" tesla-second) — test no-duplicate/alias-never-collide.
            $( e!($name, [$vi], ChemistryBio, concat!("\\mathrm{", $sym, "}"), $sym, $sym,
                concat!("Nguyên tố ", $vi), concat!("Element ", $name), 70), )*
        ];
        #[cfg(test)]
        mod element_tests {
            use super::{lookup_exact, ELEMENTS};
            use crate::config::Discipline;
            $( #[test] fn $id() {
                let en = lookup_exact($name).unwrap_or_else(|| panic!("missing element {}", $name));
                assert_eq!(en.trigger, $name);
                assert_eq!(en.unicode, $sym);
                assert!(en.latex.starts_with("\\mathrm{"));
                assert_eq!(en.discipline, Discipline::ChemistryBio);
            } )*
            #[test]
            fn element_count_118() {
                assert_eq!(ELEMENTS.len(), 118);
            }
        }
    };
}

element_table! {
    el_hydrogen: ("H", "hydrogen", "hydro"), el_helium: ("He", "helium", "heli"),
    el_lithium: ("Li", "lithium", "liti"), el_beryllium: ("Be", "beryllium", "berili"),
    el_boron: ("B", "boron", "bo"), el_carbon: ("C", "carbon", "cacbon"),
    el_nitrogen: ("N", "nitrogen", "nitơ"), el_oxygen: ("O", "oxygen", "oxy"),
    el_fluorine: ("F", "fluorine", "flo"), el_neon: ("Ne", "neon", "neon"),
    el_sodium: ("Na", "sodium", "natri"), el_magnesium: ("Mg", "magnesium", "magie"),
    el_aluminium: ("Al", "aluminium", "nhôm"), el_silicon: ("Si", "silicon", "silic"),
    el_phosphorus: ("P", "phosphorus", "photpho"), el_sulfur: ("S", "sulfur", "luuhuynh"),
    el_chlorine: ("Cl", "chlorine", "clo"), el_argon: ("Ar", "argon", "argon"),
    el_potassium: ("K", "potassium", "kali"), el_calcium: ("Ca", "calcium", "canxi"),
    el_scandium: ("Sc", "scandium", "scanđi"), el_titanium: ("Ti", "titanium", "titan"),
    el_vanadium: ("V", "vanadium", "vanadi"), el_chromium: ("Cr", "chromium", "crom"),
    el_manganese: ("Mn", "manganese", "mangan"), el_iron: ("Fe", "iron", "sắt"),
    el_cobalt: ("Co", "cobalt", "coban"), el_nickel: ("Ni", "nickel", "niken"),
    el_copper: ("Cu", "copper", "đồng"), el_zinc: ("Zn", "zinc", "kẽm"),
    el_gallium: ("Ga", "gallium", "gali"), el_germanium: ("Ge", "germanium", "germani"),
    el_arsenic: ("As", "arsenic", "asen"), el_selenium: ("Se", "selenium", "selen"),
    el_bromine: ("Br", "bromine", "brom"), el_krypton: ("Kr", "krypton", "kripton"),
    el_rubidium: ("Rb", "rubidium", "rubiđi"), el_strontium: ("Sr", "strontium", "stronti"),
    el_yttrium: ("Y", "yttrium", "ytri"), el_zirconium: ("Zr", "zirconium", "zirconi"),
    el_niobium: ("Nb", "niobium", "niobi"), el_molybdenum: ("Mo", "molybdenum", "molipđen"),
    el_technetium: ("Tc", "technetium", "tecneti"), el_ruthenium: ("Ru", "ruthenium", "ruteni"),
    el_rhodium: ("Rh", "rhodium", "rođi"), el_palladium: ("Pd", "palladium", "paladi"),
    el_silver: ("Ag", "silver", "bạc"), el_cadmium: ("Cd", "cadmium", "cadimi"),
    el_indium: ("In", "indium", "indi"), el_tin: ("Sn", "tin", "thiếc"),
    el_antimony: ("Sb", "antimony", "antimoan"), el_tellurium: ("Te", "tellurium", "telua"),
    el_iodine: ("I", "iodine", "iốt"), el_xenon: ("Xe", "xenon", "xenon"),
    el_cesium: ("Cs", "cesium", "xêsi"), el_barium: ("Ba", "barium", "bari"),
    el_lanthanum: ("La", "lanthanum", "lanthan"), el_cerium: ("Ce", "cerium", "xeri"),
    el_praseodymium: ("Pr", "praseodymium", "praseođim"), el_neodymium: ("Nd", "neodymium", "neođim"),
    el_promethium: ("Pm", "promethium", "prometi"), el_samarium: ("Sm", "samarium", "samari"),
    el_europium: ("Eu", "europium", "europi"), el_gadolinium: ("Gd", "gadolinium", "gađolini"),
    el_terbium: ("Tb", "terbium", "terbi"), el_dysprosium: ("Dy", "dysprosium", "đisprôzi"),
    el_holmium: ("Ho", "holmium", "homi"), el_erbium: ("Er", "erbium", "erbi"),
    el_thulium: ("Tm", "thulium", "tuli"), el_ytterbium: ("Yb", "ytterbium", "ytterbi"),
    el_lutetium: ("Lu", "lutetium", "luteti"), el_hafnium: ("Hf", "hafnium", "hafni"),
    el_tantalum: ("Ta", "tantalum", "tantan"), el_tungsten: ("W", "tungsten", "vonfram"),
    el_rhenium: ("Re", "rhenium", "reni"), el_osmium: ("Os", "osmium", "ôsmi"),
    el_iridium: ("Ir", "iridium", "iriđi"), el_platinum: ("Pt", "platinum", "bachkim"),
    el_gold: ("Au", "gold", "vàng"), el_mercury: ("Hg", "mercury", "thuyngan"),
    el_thallium: ("Tl", "thallium", "tali"), el_lead: ("Pb", "lead", "chì"),
    el_bismuth: ("Bi", "bismuth", "bismut"), el_polonium: ("Po", "polonium", "poloni"),
    el_astatine: ("At", "astatine", "astatin"), el_radon: ("Rn", "radon", "radon"),
    el_francium: ("Fr", "francium", "franci"), el_radium: ("Ra", "radium", "radi"),
    el_actinium: ("Ac", "actinium", "actini"), el_thorium: ("Th", "thorium", "thori"),
    el_protactinium: ("Pa", "protactinium", "protactini"), el_uranium: ("U", "uranium", "urani"),
    el_neptunium: ("Np", "neptunium", "neptuni"), el_plutonium: ("Pu", "plutonium", "plutoni"),
    el_americium: ("Am", "americium", "americi"), el_curium: ("Cm", "curium", "curi"),
    el_berkelium: ("Bk", "berkelium", "berkeli"), el_californium: ("Cf", "californium", "californi"),
    el_einsteinium: ("Es", "einsteinium", "anctini"), el_fermium: ("Fm", "fermium", "fermi"),
    el_mendelevium: ("Md", "mendelevium", "mendelevi"), el_nobelium: ("No", "nobelium", "nobeli"),
    el_lawrencium: ("Lr", "lawrencium", "laurenxi"), el_rutherfordium: ("Rf", "rutherfordium", "ruteofođi"),
    el_dubnium: ("Db", "dubnium", "đubni"), el_seaborgium: ("Sg", "seaborgium", "xêboôgi"),
    el_bohrium: ("Bh", "bohrium", "bohri"), el_hassium: ("Hs", "hassium", "hassi"),
    el_meitnerium: ("Mt", "meitnerium", "maitneri"), el_darmstadtium: ("Ds", "darmstadtium", "đamxtadi"),
    el_roentgenium: ("Rg", "roentgenium", "rônggeni"), el_copernicium: ("Cn", "copernicium", "côpenixi"),
    el_nihonium: ("Nh", "nihonium", "nihoni"), el_flerovium: ("Fl", "flerovium", "flêrovi"),
    el_moscovium: ("Mc", "moscovium", "moscovi"), el_livermorium: ("Lv", "livermorium", "livơmoi"),
    el_tenessine: ("Ts", "tennessine", "tennin"), el_oganesson: ("Og", "oganesson", "ôganêxông")
}

// ============================================================
// 6. HOÁ HỌC — HỢP CHẤT / ION / PHẢN ỨNG (~65)
// ============================================================
const CHEMISTRY: &[MathEntry] = &[
    e!(
        "H2O",
        ["nuoc", "h2o"],
        ChemistryBio,
        "H_2O",
        "H_2 O",
        "H₂O",
        "Nước",
        "Water",
        100
    ),
    e!(
        "CO2",
        ["cacbondioxit", "co2"],
        ChemistryBio,
        "CO_2",
        "C O_2",
        "CO₂",
        "Cacbon dioxit",
        "Carbon dioxide",
        100
    ),
    e!(
        "CO",
        ["cacbonmonoxit"],
        ChemistryBio,
        "CO",
        "C O",
        "CO",
        "Cacbon monoxit",
        "Carbon monoxide",
        90
    ),
    e!(
        "O2",
        ["oxyphantu", "o2"],
        ChemistryBio,
        "O_2",
        "O_2",
        "O₂",
        "Oxy phân tử",
        "Oxygen molecule",
        95
    ),
    e!(
        "N2",
        ["nitophantu", "n2"],
        ChemistryBio,
        "N_2",
        "N_2",
        "N₂",
        "Nitơ phân tử",
        "Nitrogen molecule",
        90
    ),
    e!(
        "H2",
        ["hydrophantu", "h2"],
        ChemistryBio,
        "H_2",
        "H_2",
        "H₂",
        "Hydro phân tử",
        "Hydrogen molecule",
        90
    ),
    e!(
        "NaCl",
        ["muoian", "nacl"],
        ChemistryBio,
        "NaCl",
        "Na Cl",
        "NaCl",
        "Natri clorua (muối ăn)",
        "Sodium chloride",
        95
    ),
    e!(
        "H2SO4",
        ["axitsunfuric", "h2so4"],
        ChemistryBio,
        "H_2SO_4",
        "H_2 S O_4",
        "H₂SO₄",
        "Axit sunfuric",
        "Sulfuric acid",
        95
    ),
    e!(
        "HNO3",
        ["axitnitric", "hno3"],
        ChemistryBio,
        "HNO_3",
        "H N O_3",
        "HNO₃",
        "Axit nitric",
        "Nitric acid",
        90
    ),
    e!(
        "HCl",
        ["axitclohydric", "hcl"],
        ChemistryBio,
        "HCl",
        "H Cl",
        "HCl",
        "Axit clohydric",
        "Hydrochloric acid",
        90
    ),
    e!(
        "NaOH",
        ["natrihidroxit", "naoh"],
        ChemistryBio,
        "NaOH",
        "Na O H",
        "NaOH",
        "Natri hidroxit",
        "Sodium hydroxide",
        90
    ),
    e!(
        "KOH",
        ["kalihidroxit", "koh"],
        ChemistryBio,
        "KOH",
        "K O H",
        "KOH",
        "Kali hidroxit",
        "Potassium hydroxide",
        85
    ),
    e!(
        "CaCO3",
        ["davoi", "caco3"],
        ChemistryBio,
        "CaCO_3",
        "Ca C O_3",
        "CaCO₃",
        "Canxi cacbonat (đá vôi)",
        "Calcium carbonate",
        85
    ),
    e!(
        "NaHCO3",
        ["muoinoi", "nahco3"],
        ChemistryBio,
        "NaHCO_3",
        "Na H C O_3",
        "NaHCO₃",
        "Natri hiđrocacbonat",
        "Sodium bicarbonate",
        85
    ),
    e!(
        "CH4",
        ["metan", "ch4"],
        ChemistryBio,
        "CH_4",
        "C H_4",
        "CH₄",
        "Metan",
        "Methane",
        90
    ),
    e!(
        "NH3",
        ["amoniac", "nh3"],
        ChemistryBio,
        "NH_3",
        "N H_3",
        "NH₃",
        "Amoniac",
        "Ammonia",
        90
    ),
    e!(
        "C2H5OH",
        ["ruouetylic", "c2h5oh"],
        ChemistryBio,
        "C_2H_5OH",
        "C_2 H_5 O H",
        "C₂H₅OH",
        "Ethanol",
        "Ethanol",
        85
    ),
    e!(
        "CH3COOH",
        ["axitacetic", "ch3cooh"],
        ChemistryBio,
        "CH_3COOH",
        "C H_3 C O O H",
        "CH₃COOH",
        "Axit acetic (giấm)",
        "Acetic acid",
        85
    ),
    e!(
        "C6H12O6",
        ["glucozo", "c6h12o6"],
        ChemistryBio,
        "C_6H_{12}O_6",
        "C_6 H_(12) O_6",
        "C₆H₁₂O₆",
        "Glucozơ",
        "Glucose",
        85
    ),
    e!(
        "Na2SO4",
        ["natrisunfat", "na2so4"],
        ChemistryBio,
        "Na_2SO_4",
        "Na_2 S O_4",
        "Na₂SO₄",
        "Natri sunfat",
        "Sodium sulfate",
        85
    ),
    e!(
        "equilibrium",
        ["canbanghoahoc"],
        ChemistryBio,
        "\\rightleftharpoons",
        "harpoons.rtlb",
        "⇌",
        "Cân bằng hoá học",
        "Chemical equilibrium",
        95
    ),
    e!(
        "deltaH",
        ["nhietphanung", "entalpi"],
        ChemistryBio,
        "\\Delta H",
        "Delta H",
        "ΔH",
        "Biến thiên entalpi",
        "Enthalpy change",
        95
    ),
    e!(
        "deltaG",
        ["nangluongdo"],
        ChemistryBio,
        "\\Delta G",
        "Delta G",
        "ΔG",
        "Biến thiên năng lượng Gibbs",
        "Gibbs free energy change",
        90
    ),
    e!(
        "deltaS",
        ["entropybienthien"],
        ChemistryBio,
        "\\Delta S",
        "Delta S",
        "ΔS",
        "Biến thiên entropy",
        "Entropy change",
        85
    ),
    e!(
        "pKa",
        ["hangsophanli"],
        ChemistryBio,
        "pK_a",
        "p K_a",
        "pKa",
        "Hằng số phân li axit",
        "Acid dissociation constant",
        90
    ),
    e!(
        "pKb",
        ["hangsophanlibaz"],
        ChemistryBio,
        "pK_b",
        "p K_b",
        "pKb",
        "Hằng số phân li bazơ",
        "Base dissociation constant",
        80
    ),
    e!(
        "pH",
        ["dopH"],
        ChemistryBio,
        "pH",
        "p H",
        "pH",
        "Độ pH",
        "pH",
        95
    ),
    e!(
        "Fe2+",
        ["sathai"],
        ChemistryBio,
        "Fe^{2+}",
        "Fe^(2+)",
        "Fe²⁺",
        "Ion sắt (II)",
        "Iron(II) ion",
        85
    ),
    e!(
        "Fe3+",
        ["satba"],
        ChemistryBio,
        "Fe^{3+}",
        "Fe^(3+)",
        "Fe³⁺",
        "Ion sắt (III)",
        "Iron(III) ion",
        85
    ),
    e!(
        "OH-",
        ["ionhidroxit"],
        ChemistryBio,
        "OH^{-}",
        "O H^-",
        "OH⁻",
        "Ion hidroxit",
        "Hydroxide ion",
        90
    ),
    e!(
        "H+",
        ["ionhydro"],
        ChemistryBio,
        "H^{+}",
        "H^+",
        "H⁺",
        "Ion hydro",
        "Hydrogen ion",
        90
    ),
    e!(
        "NO3-",
        ["ionnitrat"],
        ChemistryBio,
        "NO_3^{-}",
        "N O_3^-",
        "NO₃⁻",
        "Ion nitrat",
        "Nitrate ion",
        85
    ),
    e!(
        "SO4-2",
        ["ionsunfat"],
        ChemistryBio,
        "SO_4^{2-}",
        "S O_4^(2-)",
        "SO₄²⁻",
        "Ion sunfat",
        "Sulfate ion",
        85
    ),
    e!(
        "CO3-2",
        ["ioncacbonat"],
        ChemistryBio,
        "CO_3^{2-}",
        "C O_3^(2-)",
        "CO₃²⁻",
        "Ion cacbonat",
        "Carbonate ion",
        80
    ),
    e!(
        "NH4+",
        ["ionamoni"],
        ChemistryBio,
        "NH_4^{+}",
        "N H_4^+",
        "NH₄⁺",
        "Ion amoni",
        "Ammonium ion",
        85
    ),
    e!(
        "PO4-3",
        ["ionphotphat"],
        ChemistryBio,
        "PO_4^{3-}",
        "P O_4^(3-)",
        "PO₄³⁻",
        "Ion photphat",
        "Phosphate ion",
        80
    ),
    e!(
        "Cl-",
        ["ionclorua"],
        ChemistryBio,
        "Cl^{-}",
        "C l^-",
        "Cl⁻",
        "Ion clorua",
        "Chloride ion",
        85
    ),
    e!(
        "gasup",
        ["khibaylen"],
        ChemistryBio,
        "\\uparrow",
        "arrow.t",
        "↑",
        "Khí bay lên",
        "Gas evolves upward",
        90
    ),
    e!(
        "gasdown",
        ["khikettua"],
        ChemistryBio,
        "\\downarrow",
        "arrow.b",
        "↓",
        "Kết tủa lắng xuống",
        "Precipitate downward",
        90
    ),
    e!(
        "mole_unit",
        ["molchat"],
        ChemistryBio,
        "\\mathrm{mol}",
        "mol",
        "mol",
        "Đơn vị mol",
        "Mole (unit)",
        90
    ),
    e!(
        "avogadro",
        ["soavogadro", "NA"],
        ChemistryBio,
        "N_A",
        "N_A",
        "Nₐ",
        "Hằng số Avogadro",
        "Avogadro constant",
        90
    ),
    e!(
        "gas_constant",
        ["hangsokhi"],
        ChemistryBio,
        "R",
        "R",
        "R",
        "Hằng số khí lý tưởng",
        "Ideal gas constant",
        85
    ),
    e!(
        "ideal_gas",
        ["phuongtrinhkhi"],
        ChemistryBio,
        "pV = nRT",
        "p V = n R T",
        "pV = nRT",
        "Phương trình khí lý tưởng",
        "Ideal gas law",
        95
    ),
    e!(
        "oxidation",
        ["phanungoxihoa"],
        ChemistryBio,
        "\\text{oxi hoá}",
        "oxi hoá",
        "oxi hóa",
        "Phản ứng oxi hóa",
        "Oxidation",
        75
    ),
    e!(
        "reduction",
        ["phanungkhoi"],
        ChemistryBio,
        "\\text{khử}",
        "khử",
        "khử",
        "Phản ứng khử",
        "Reduction",
        75
    ),
    e!(
        "redox",
        ["oxihoakhoi"],
        ChemistryBio,
        "\\text{oxi hoá–khử}",
        "oxi hoá–khử",
        "oxi hóa–khử",
        "Phản ứng oxi hóa–khử",
        "Redox reaction",
        80
    ),
    e!(
        "catalyst",
        ["xuctac"],
        ChemistryBio,
        "\\mathrm{cat}",
        "cat",
        "cat",
        "Chất xúc tác (ký hiệu)",
        "Catalyst",
        75
    ),
    e!(
        "ATP",
        ["atp"],
        ChemistryBio,
        "\\mathrm{ATP}",
        "ATP",
        "ATP",
        "Adenosine triphosphate",
        "Adenosine triphosphate",
        85
    ),
    e!(
        "DNA",
        ["dna"],
        ChemistryBio,
        "\\mathrm{DNA}",
        "DNA",
        "DNA",
        "Axit nucleic",
        "Deoxyribonucleic acid",
        85
    ),
    e!(
        "RNA",
        ["rna"],
        ChemistryBio,
        "\\mathrm{RNA}",
        "RNA",
        "RNA",
        "Axit ribonucleic",
        "Ribonucleic acid",
        85
    ),
    e!(
        "base_A",
        ["adenine"],
        ChemistryBio,
        "\\mathrm{A}",
        "A",
        "A",
        "Bazơ adenine",
        "Adenine base",
        75
    ),
    e!(
        "base_T",
        ["thymine"],
        ChemistryBio,
        "\\mathrm{T}",
        "T",
        "T",
        "Bazơ thymine",
        "Thymine base",
        75
    ),
    e!(
        "base_C",
        ["cytosine"],
        ChemistryBio,
        "\\mathrm{C}",
        "C",
        "C",
        "Bazơ cytosine",
        "Cytosine base",
        75
    ),
    e!(
        "base_G",
        ["guanine"],
        ChemistryBio,
        "\\mathrm{G}",
        "G",
        "G",
        "Bazơ guanine",
        "Guanine base",
        75
    ),
    e!(
        "base_U",
        ["uracil"],
        ChemistryBio,
        "\\mathrm{U}",
        "U",
        "U",
        "Bazơ uracil (RNA)",
        "Uracil base (RNA)",
        75
    ),
    e!(
        "protein",
        ["protein"],
        ChemistryBio,
        "\\mathrm{Protein}",
        "Protein",
        "Protein",
        "Protein",
        "Protein",
        75
    ),
    e!(
        "enzyme",
        ["enzyme"],
        ChemistryBio,
        "\\mathrm{E}",
        "E",
        "E",
        "Enzyme",
        "Enzyme",
        75
    ),
    e!(
        "substrate",
        ["cochat"],
        ChemistryBio,
        "\\mathrm{S}",
        "S",
        "S",
        "Cơ chất",
        "Substrate",
        70
    ),
    e!(
        "michaelis",
        ["hangsomichaelis"],
        ChemistryBio,
        "K_m",
        "K_m",
        "Kₘ",
        "Hằng số Michaelis",
        "Michaelis constant",
        75
    ),
    e!(
        "yield_pct",
        ["hieusuatphanung"],
        ChemistryBio,
        "\\eta",
        "eta",
        "η",
        "Hiệu suất phản ứng",
        "Reaction yield",
        80
    ),
    e!(
        "molar_mass",
        ["khoiluongmol"],
        ChemistryBio,
        "M",
        "M",
        "M",
        "Khối lượng mol",
        "Molar mass",
        80
    ),
    e!(
        "mol_fraction",
        ["phanmol"],
        ChemistryBio,
        "x_i",
        "x_i",
        "xᵢ",
        "Phân mol",
        "Mole fraction",
        75
    ),
];

// ============================================================
// 7. VẬT LÝ / CƠ KHÍ / GD&T (~85)
// ============================================================
const PHYSICS: &[MathEntry] = &[
    e!(
        "diameter",
        ["duongkinh"],
        PhysicsMech,
        "\\varnothing",
        "diameter",
        "Ø",
        "Đường kính (GD&T)",
        "Diameter (GD&T)",
        100
    ),
    e!(
        "radius_sym",
        ["bankinh"],
        PhysicsMech,
        "r",
        "r",
        "r",
        "Bán kính",
        "Radius",
        90
    ),
    e!(
        "position_tolerance",
        ["dungsaivitri"],
        PhysicsMech,
        "\\text{⌖}",
        "position",
        "⌖",
        "Dung sai vị trí",
        "Position tolerance",
        85
    ),
    e!(
        "flatness",
        ["dophang"],
        PhysicsMech,
        "\\text{⏥}",
        "flatness",
        "⏥",
        "Độ phẳng",
        "Flatness",
        85
    ),
    e!(
        "circularity",
        ["dotron"],
        PhysicsMech,
        "\\bigcirc",
        "circularity",
        "○",
        "Độ tròn",
        "Circularity",
        80
    ),
    e!(
        "cylindricity",
        ["dotrontru"],
        PhysicsMech,
        "\\oslash",
        "cylindricity",
        "⊙",
        "Độ tròn trụ",
        "Cylindricity",
        75
    ),
    e!(
        "straightness",
        ["dothang"],
        PhysicsMech,
        "\\text{—}",
        "straightness",
        "—",
        "Độ thẳng",
        "Straightness",
        75
    ),
    e!(
        "perpendicularity",
        ["dungsaivuonggoc"],
        PhysicsMech,
        "\\perp",
        "perp",
        "⊥",
        "Dung sai vuông góc",
        "Perpendicularity",
        85
    ),
    e!(
        "angularity",
        ["dungsainghieng"],
        PhysicsMech,
        "\\angle",
        "angle",
        "∠",
        "Dung sai nghiêng",
        "Angularity",
        75
    ),
    e!(
        "profile_surface",
        ["duongbaomat"],
        PhysicsMech,
        "\\partial\\bigcirc",
        "profile",
        "◡",
        "Dung sai đường bao bề mặt",
        "Surface profile",
        70
    ),
    e!(
        "runout",
        ["donhole"],
        PhysicsMech,
        "\\nearrow",
        "runout",
        "↗",
        "Độ đảo",
        "Runout",
        75
    ),
    e!(
        "concentricity",
        ["dongtam"],
        PhysicsMech,
        "\\bigcirc\\!\\bigcirc",
        "concentricity",
        "◎",
        "Đồng tâm",
        "Concentricity",
        75
    ),
    e!(
        "F_ma",
        ["lucbangkhoiluong"],
        PhysicsMech,
        "F = ma",
        "F = m a",
        "F = ma",
        "Định luật 2 Newton",
        "Newton's second law",
        95
    ),
    e!(
        "E_mc2",
        ["nangluongkhoiluong"],
        PhysicsMech,
        "E = mc^2",
        "E = m c^2",
        "E = mc²",
        "Tương đương khối lượng–năng lượng",
        "Mass–energy equivalence",
        95
    ),
    e!(
        "p_mv",
        ["dongluong"],
        PhysicsMech,
        "p = mv",
        "p = m v",
        "p = mv",
        "Động lượng",
        "Momentum",
        90
    ),
    e!(
        "W_Fd",
        ["congcoluc"],
        PhysicsMech,
        "W = F \\cdot d",
        "W = F dot d",
        "W = F·d",
        "Công của lực",
        "Work done by force",
        90
    ),
    e!(
        "P_Wt",
        ["congsuat"],
        PhysicsMech,
        "P = W/t",
        "P = W / t",
        "P = W/t",
        "Công suất",
        "Power",
        90
    ),
    e!(
        "KE_halfmv2",
        ["nangluongdong"],
        PhysicsMech,
        "E_k = \\tfrac{1}{2}mv^2",
        "E_k = 1/2 m v^2",
        "Eₖ = ½mv²",
        "Động năng",
        "Kinetic energy",
        90
    ),
    e!(
        "PE_mgh",
        ["thenang"],
        PhysicsMech,
        "E_p = mgh",
        "E_p = m g h",
        "Eₚ = mgh",
        "Thế năng trọng trường",
        "Gravitational potential energy",
        90
    ),
    e!(
        "rho_mV",
        ["khoiluongrieng"],
        PhysicsMech,
        "\\rho = m/V",
        "rho = m / V",
        "ρ = m/V",
        "Khối lượng riêng",
        "Density",
        90
    ),
    e!(
        "Q_mcdT",
        ["nhietdung"],
        PhysicsMech,
        "Q = mc\\Delta T",
        "Q = m c Delta T",
        "Q = mcΔT",
        "Nhiệt lượng truyền",
        "Heat transferred",
        85
    ),
    e!(
        "F_kx",
        ["lucloxo"],
        PhysicsMech,
        "F = kx",
        "F = k x",
        "F = kx",
        "Định luật Hooke",
        "Hooke's law",
        85
    ),
    e!(
        "T_pendulum",
        ["chukyconlac"],
        PhysicsMech,
        "T = 2\\pi\\sqrt{L/g}",
        "T = 2 pi sqrt(L / g)",
        "T = 2π√(L/g)",
        "Chu kỳ con lắc đơn",
        "Simple pendulum period",
        85
    ),
    e!(
        "lambda_vf",
        ["buocsong"],
        PhysicsMech,
        "\\lambda = v/f",
        "lambda = v / f",
        "λ = v/f",
        "Bước sóng",
        "Wavelength",
        90
    ),
    e!(
        "c_lambda_f",
        ["tocdosong"],
        PhysicsMech,
        "c = \\lambda f",
        "c = lambda f",
        "c = λf",
        "Tốc độ sóng",
        "Wave speed",
        85
    ),
    e!(
        "snells_law",
        ["quangtac"],
        PhysicsMech,
        "n_1\\sin\\theta_1 = n_2\\sin\\theta_2",
        "n_1 sin theta_1 = n_2 sin theta_2",
        "n₁sinθ₁ = n₂sinθ₂",
        "Định luật Snell",
        "Snell's law",
        85
    ),
    e!(
        "ohms_law",
        ["dinhluatohm"],
        PhysicsMech,
        "V = IR",
        "V = I R",
        "V = IR",
        "Định luật Ohm",
        "Ohm's law",
        95
    ),
    e!(
        "young_modulus",
        ["modunyoung"],
        PhysicsMech,
        "E = \\sigma/\\varepsilon",
        "E = sigma / epsilon",
        "E = σ/ε",
        "Mô đun Young",
        "Young's modulus",
        85
    ),
    e!(
        "stress_sigma",
        ["ungsuat"],
        PhysicsMech,
        "\\sigma = F/A",
        "sigma = F / A",
        "σ = F/A",
        "Ứng suất",
        "Stress",
        90
    ),
    e!(
        "strain_eps",
        ["biendang"],
        PhysicsMech,
        "\\varepsilon = \\Delta L / L",
        "epsilon = Delta L / L",
        "ε = ΔL/L",
        "Biến dạng",
        "Strain",
        85
    ),
    e!(
        "torque_tau",
        ["momen"],
        PhysicsMech,
        "\\tau = r F \\sin\\theta",
        "tau = r F sin theta",
        "τ = rF·sinθ",
        "Mô men lực",
        "Torque",
        90
    ),
    e!(
        "angular_momentum",
        ["dongluonggoc"],
        PhysicsMech,
        "L = I\\omega",
        "L = I omega",
        "L = Iω",
        "Động lượng góc",
        "Angular momentum",
        85
    ),
    e!(
        "freq_period",
        ["tansochuky"],
        PhysicsMech,
        "f = 1/T",
        "f = 1 / T",
        "f = 1/T",
        "Tần số – chu kỳ",
        "Frequency–period",
        90
    ),
    e!(
        "omega_2pif",
        ["tocdogoc"],
        PhysicsMech,
        "\\omega = 2\\pi f",
        "omega = 2 pi f",
        "ω = 2πf",
        "Tốc độ góc",
        "Angular frequency",
        90
    ),
    e!(
        "efficiency",
        ["hieusuat"],
        PhysicsMech,
        "\\eta = P_{out}/P_{in}",
        "eta = P_out / P_in",
        "η = Pout/Pin",
        "Hiệu suất",
        "Efficiency",
        90
    ),
    e!(
        "impulse",
        ["xungluc"],
        PhysicsMech,
        "J = F\\Delta t",
        "J = F Delta t",
        "J = FΔt",
        "Xung lượng lực",
        "Impulse",
        85
    ),
    e!(
        "coulomb_law",
        ["luccuongluongdien"],
        PhysicsMech,
        "F = k q_1 q_2 / r^2",
        "F = k q_1 q_2 / r^2",
        "F = kq₁q₂/r²",
        "Định luật Coulomb",
        "Coulomb's law",
        90
    ),
    e!(
        "capacitance",
        ["diendung"],
        PhysicsMech,
        "C = Q/V",
        "C = Q / V",
        "C = Q/V",
        "Điện dung",
        "Capacitance",
        90
    ),
    e!(
        "rc_time",
        ["hangthoigianRC"],
        PhysicsMech,
        "\\tau = RC",
        "tau = R C",
        "τ = RC",
        "Hằng thời gian RC",
        "RC time constant",
        90
    ),
    e!(
        "faraday_law",
        ["dongdiencamung"],
        PhysicsMech,
        "\\mathcal{E} = -N \\Delta\\Phi / \\Delta t",
        "E = -N Delta Phi / Delta t",
        "ℰ = −NΔΦ/Δt",
        "Định luật Faraday",
        "Faraday's law",
        85
    ),
    e!(
        "magnetic_force",
        ["lucloran"],
        PhysicsMech,
        "F = qvB",
        "F = q v B",
        "F = qvB",
        "Lực Lorentz",
        "Magnetic force",
        85
    ),
    e!(
        "photoelectric",
        ["hientuongquangdien"],
        PhysicsMech,
        "E_k = h\\nu - \\phi",
        "E_k = h nu - phi",
        "Eₖ = hν − φ",
        "Hiệu ứng quang điện",
        "Photoelectric effect",
        85
    ),
    e!(
        "planck_const",
        ["hangsoplanck"],
        PhysicsMech,
        "h",
        "h",
        "h",
        "Hằng số Planck",
        "Planck constant",
        90
    ),
    e!(
        "boltzmann",
        ["hangsoboltzmann"],
        PhysicsMech,
        "k_B",
        "k_B",
        "k_B",
        "Hằng số Boltzmann",
        "Boltzmann constant",
        85
    ),
    e!(
        "entropy_qT",
        ["entropynhiet"],
        PhysicsMech,
        "S = Q/T",
        "S = Q / T",
        "S = Q/T",
        "Entropy nhiệt động",
        "Thermodynamic entropy",
        80
    ),
    e!(
        "carnot_eff",
        ["hieusuatcarnot"],
        PhysicsMech,
        "\\eta = 1 - T_c/T_h",
        "eta = 1 - T_c / T_h",
        "η = 1 − Tc/Th",
        "Hiệu suất Carnot",
        "Carnot efficiency",
        85
    ),
    e!(
        "g_force",
        ["trongluc"],
        PhysicsMech,
        "g",
        "g",
        "g",
        "Gia tốc trọng trường",
        "Gravitational acceleration",
        90
    ),
    e!(
        "reynolds",
        ["soreynolds"],
        PhysicsMech,
        "\\mathrm{Re} = \\rho v L / \\mu",
        "Re = rho v L / mu",
        "Re = ρvL/µ",
        "Số Reynolds",
        "Reynolds number",
        80
    ),
    e!(
        "mach_number",
        ["somach"],
        PhysicsMech,
        "\\mathrm{Ma}",
        "Ma",
        "Ma",
        "Số Mach",
        "Mach number",
        80
    ),
    e!(
        "mom_inertia",
        ["momentquantinh"],
        PhysicsMech,
        "I",
        "I",
        "I",
        "Mô men quán tính",
        "Moment of inertia",
        85
    ),
    e!(
        "lever_mech",
        ["loaiconlon"],
        PhysicsMech,
        "F_1 l_1 = F_2 l_2",
        "F_1 l_1 = F_2 l_2",
        "F₁l₁ = F₂l₂",
        "Cân bằng đòn bẩy",
        "Lever equilibrium",
        80
    ),
    e!(
        "doppler",
        ["hientuongdoppler"],
        PhysicsMech,
        "f' = f (v \\pm v_o)/v",
        "f' = f (v ± v_o) / v",
        "f′ = f(v±v₀)/v",
        "Hiệu ứng Doppler",
        "Doppler effect",
        80
    ),
    e!(
        "ideal_mirror",
        ["sophangguong"],
        PhysicsMech,
        "\\theta_i = \\theta_r",
        "theta_i = theta_r",
        "θi = θr",
        "Phản xạ góc bằng",
        "Law of reflection",
        80
    ),
    e!(
        "rms_current",
        ["dongdienhieudung"],
        PhysicsMech,
        "I_{rms} = I_0 / \\sqrt{2}",
        "I_rms = I_0 / sqrt(2)",
        "Irms = I₀/√2",
        "Dòng điện hiệu dụng",
        "RMS current",
        85
    ),
    e!(
        "reactance",
        ["dienkhang"],
        PhysicsMech,
        "X_L = \\omega L",
        "X_L = omega L",
        "X_L = ωL",
        "Điện kháng cảm",
        "Inductive reactance",
        85
    ),
    e!(
        "impedance",
        ["trongan"],
        PhysicsMech,
        "Z = \\sqrt{R^2 + X^2}",
        "Z = sqrt(R^2 + X^2)",
        "Z = √(R²+X²)",
        "Trở ngangs tổng",
        "Impedance",
        85
    ),
    e!(
        "resonance",
        ["conghuong"],
        PhysicsMech,
        "f_0 = 1/(2\\pi\\sqrt{LC})",
        "f_0 = 1 / (2 pi sqrt(L C))",
        "f₀ = 1/(2π√(LC))",
        "Tần số cộng hưởng",
        "Resonance frequency",
        90
    ),
];

// ============================================================
// 8. ĐIỆN / ÔTÔ / EMBEDDED (~60)
// ============================================================
const ELECTRICAL: &[MathEntry] = &[
    e!(
        "ohm_sym",
        ["trodien", "ohm"],
        ElectricalEmbedded,
        "\\Omega",
        "Omega",
        "Ω",
        "Điện trở (ký hiệu ôm)",
        "Ohm (resistance)",
        100
    ),
    e!(
        "kohm",
        ["kiloom"],
        ElectricalEmbedded,
        "\\mathrm{k\\Omega}",
        "k Omega",
        "kΩ",
        "Kiloohm",
        "Kiloohm",
        90
    ),
    e!(
        "Mohm",
        ["megaoom"],
        ElectricalEmbedded,
        "\\mathrm{M\\Omega}",
        "M Omega",
        "MΩ",
        "Megaohm",
        "Megaohm",
        90
    ),
    e!(
        "dBm",
        ["dbm"],
        ElectricalEmbedded,
        "\\mathrm{dBm}",
        "dBm",
        "dBm",
        "Decibel milliwatt",
        "Decibel-milliwatts",
        85
    ),
    e!(
        "dbd",
        ["dbd"],
        ElectricalEmbedded,
        "\\mathrm{dB}",
        "dB",
        "dB",
        "Decibel",
        "Decibel",
        90
    ),
    e!(
        "VA_unit",
        ["vonampe"],
        ElectricalEmbedded,
        "\\mathrm{VA}",
        "V A",
        "VA",
        "Volt-ampere (công suất biểu kiến)",
        "Volt-ampere",
        80
    ),
    e!(
        "VAR_unit",
        ["vonampephan"],
        ElectricalEmbedded,
        "\\mathrm{VAR}",
        "VAR",
        "VAR",
        "Volt-ampere phản kháng",
        "Volt-ampere reactive",
        75
    ),
    e!(
        "power_factor",
        ["hesocongsuat"],
        ElectricalEmbedded,
        "\\cos\\varphi",
        "cos phi",
        "cosφ",
        "Hệ số công suất",
        "Power factor",
        85
    ),
    e!(
        "GPIO",
        ["chanvaora"],
        ElectricalEmbedded,
        "\\mathrm{GPIO}",
        "GPIO",
        "GPIO",
        "Chân GPIO",
        "General-purpose I/O",
        85
    ),
    e!(
        "PWM",
        ["dieuchexung"],
        ElectricalEmbedded,
        "\\mathrm{PWM}",
        "PWM",
        "PWM",
        "Điều chế độ rộng xung",
        "Pulse-width modulation",
        90
    ),
    e!(
        "ADC",
        ["biendoiadc"],
        ElectricalEmbedded,
        "\\mathrm{ADC}",
        "ADC",
        "ADC",
        "Bộ chuyển đổi tương tự→số",
        "Analog-to-digital converter",
        90
    ),
    e!(
        "DAC",
        ["biendoidac"],
        ElectricalEmbedded,
        "\\mathrm{DAC}",
        "DAC",
        "DAC",
        "Bộ chuyển đổi số→tương tự",
        "Digital-to-analog converter",
        85
    ),
    e!(
        "CAN_bus",
        ["buscan"],
        ElectricalEmbedded,
        "\\mathrm{CAN}",
        "CAN",
        "CAN",
        "Bus CAN (ô tô)",
        "CAN bus (automotive)",
        85
    ),
    e!(
        "UART",
        ["giaotiepuart"],
        ElectricalEmbedded,
        "\\mathrm{UART}",
        "UART",
        "UART",
        "Giao tiếp UART",
        "UART",
        85
    ),
    e!(
        "SPI_bus",
        ["giaotiepspi"],
        ElectricalEmbedded,
        "\\mathrm{SPI}",
        "SPI",
        "SPI",
        "Giao tiếp SPI",
        "SPI",
        85
    ),
    e!(
        "I2C",
        ["giaotiepi2c"],
        ElectricalEmbedded,
        "\\mathrm{I^2C}",
        "I2C",
        "I²C",
        "Giao tiếp I²C",
        "I²C",
        85
    ),
    e!(
        "CAN_FD",
        ["canfd"],
        ElectricalEmbedded,
        "\\mathrm{CAN\\ FD}",
        "CAN FD",
        "CAN FD",
        "CAN Flexible Data-rate",
        "CAN with Flexible Data-rate",
        80
    ),
    e!(
        "ECU",
        ["dieukhiendongco"],
        ElectricalEmbedded,
        "\\mathrm{ECU}",
        "ECU",
        "ECU",
        "Bộ điều khiển điện tử",
        "Electronic Control Unit",
        80
    ),
    e!(
        "LIN_bus",
        ["buslin"],
        ElectricalEmbedded,
        "\\mathrm{LIN}",
        "LIN",
        "LIN",
        "Bus LIN (ô tô)",
        "LIN bus",
        75
    ),
    e!(
        "OBD",
        ["chandoanxe"],
        ElectricalEmbedded,
        "\\mathrm{OBD}",
        "OBD",
        "OBD",
        "Chẩn đoán xe",
        "On-board diagnostics",
        75
    ),
    e!(
        "firmware",
        ["phanmemnen"],
        ElectricalEmbedded,
        "\\mathrm{FW}",
        "FW",
        "FW",
        "Phần mềm nhúng",
        "Firmware",
        75
    ),
    e!(
        "ISR",
        ["chuongtrinhngat"],
        ElectricalEmbedded,
        "\\mathrm{ISR}",
        "ISR",
        "ISR",
        "Chương trình phục vụ ngắt",
        "Interrupt service routine",
        75
    ),
    e!(
        "duty_cycle",
        ["chukydap"],
        ElectricalEmbedded,
        "D = t_{on}/T",
        "D = t_on / T",
        "D = ton/T",
        "Chu kỳ dẫn (duty cycle)",
        "Duty cycle",
        85
    ),
    e!(
        "logic_and",
        ["congand"],
        ElectricalEmbedded,
        "\\land",
        "and",
        "∧",
        "Cổng AND",
        "AND gate",
        80
    ),
    e!(
        "logic_or",
        ["congor"],
        ElectricalEmbedded,
        "\\lor",
        "or",
        "∨",
        "Cổng OR",
        "OR gate",
        80
    ),
    e!(
        "logic_not",
        ["congnot"],
        ElectricalEmbedded,
        "\\neg",
        "not",
        "¬",
        "Cổng NOT",
        "NOT gate",
        80
    ),
    e!(
        "logic_nand",
        ["congnand"],
        ElectricalEmbedded,
        "\\overline{A \\land B}",
        "not (A and B)",
        "A NAND B",
        "Cổng NAND",
        "NAND gate",
        80
    ),
    e!(
        "logic_nor",
        ["congnor"],
        ElectricalEmbedded,
        "\\overline{A \\lor B}",
        "not (A or B)",
        "A NOR B",
        "Cổng NOR",
        "NOR gate",
        80
    ),
    e!(
        "logic_xor",
        ["congxor"],
        ElectricalEmbedded,
        "\\oplus",
        "xor",
        "⊕",
        "Cổng XOR",
        "XOR gate",
        85
    ),
    e!(
        "logic_xnor",
        ["congxnor"],
        ElectricalEmbedded,
        "\\overline{A \\oplus B}",
        "not (A xor B)",
        "A XNOR B",
        "Cổng XNOR",
        "XNOR gate",
        75
    ),
    e!(
        "clock_freq",
        ["xungnhip"],
        ElectricalEmbedded,
        "f_{clk}",
        "f_clk",
        "f_clk",
        "Tần số xung nhịp",
        "Clock frequency",
        80
    ),
    e!(
        "bitrate",
        ["tocdotruyen"],
        ElectricalEmbedded,
        "R_b",
        "R_b",
        "R_b",
        "Tốc độ truyền bit",
        "Bit rate",
        80
    ),
    e!(
        "snr",
        ["tiletinhieunhieu"],
        ElectricalEmbedded,
        "\\mathrm{SNR}",
        "SNR",
        "SNR",
        "Tỉ lệ tín hiệu/nhiễu",
        "Signal-to-noise ratio",
        85
    ),
    e!(
        "thd",
        ["dobiendang"],
        ElectricalEmbedded,
        "\\mathrm{THD}",
        "THD",
        "THD",
        "Độ méo hài tổng",
        "Total harmonic distortion",
        75
    ),
    e!(
        "emu_gnd",
        ["dat"],
        ElectricalEmbedded,
        "\\bot",
        "bot",
        "⊥",
        "Nối đất (mạch)",
        "Ground (circuit)",
        75
    ),
    e!(
        "vcc",
        ["dienapcap"],
        ElectricalEmbedded,
        "V_{CC}",
        "V_CC",
        "V_CC",
        "Điện áp cấp nguồn",
        "Supply voltage",
        80
    ),
    e!(
        "vpp",
        ["vonkhoidinh"],
        ElectricalEmbedded,
        "V_{pp}",
        "V_pp",
        "V_pp",
        "Điện áp đỉnh–đỉnh",
        "Peak-to-peak voltage",
        80
    ),
    e!(
        "emf_mathcal",
        ["suatdiendong"],
        ElectricalEmbedded,
        "\\mathcal{E}",
        "cal(E)",
        "ℰ",
        "Suất điện động",
        "Electromotive force",
        85
    ),
    e!(
        "flux_magnetic",
        ["tuthong"],
        ElectricalEmbedded,
        "\\Phi",
        "Phi",
        "Φ",
        "Từ thông",
        "Magnetic flux",
        85
    ),
    e!(
        "inductance",
        ["tudiencam"],
        ElectricalEmbedded,
        "L",
        "L",
        "L",
        "Điện cảm (cuộn cảm)",
        "Inductance",
        85
    ),
    e!(
        "diode_fwd",
        ["dienaptruocdiode"],
        ElectricalEmbedded,
        "V_F",
        "V_F",
        "V_F",
        "Điện áp phân cực thuận",
        "Forward voltage",
        80
    ),
    e!(
        "beta_hfe",
        ["hesoboihaitransistor"],
        ElectricalEmbedded,
        "\\beta",
        "beta",
        "β",
        "Hệ số khuếch đại transistor",
        "Transistor gain (hFE)",
        85
    ),
    e!(
        "gm_transconductance",
        ["dodandien"],
        ElectricalEmbedded,
        "g_m",
        "g_m",
        "g_m",
        "Độ dẫn truyền",
        "Transconductance",
        75
    ),
    e!(
        "fid_toler",
        ["dungsainguyenmau"],
        ElectricalEmbedded,
        "\\pm",
        "plus.minus",
        "±",
        "Dung sai",
        "Tolerance",
        85
    ),
    e!(
        "hex_prefix",
        ["hexa"],
        ComputerScienceAi,
        "\\mathrm{0x}",
        "0x",
        "0x",
        "Tiền tố hệ 16",
        "Hexadecimal prefix",
        80
    ),
    e!(
        "binary_prefix",
        ["nhiphan"],
        ComputerScienceAi,
        "\\mathrm{0b}",
        "0b",
        "0b",
        "Tiền tố hệ 2",
        "Binary prefix",
        80
    ),
    e!(
        "kb_mem",
        ["kilobyte"],
        ComputerScienceAi,
        "\\mathrm{KB}",
        "KB",
        "KB",
        "Kilobyte",
        "Kilobyte",
        80
    ),
    e!(
        "MB_mem",
        ["megabyte"],
        ComputerScienceAi,
        "\\mathrm{MB}",
        "MB",
        "MB",
        "Megabyte",
        "Megabyte",
        80
    ),
    e!(
        "GB_mem",
        ["gigabyte"],
        ComputerScienceAi,
        "\\mathrm{GB}",
        "GB",
        "GB",
        "Gigabyte",
        "Gigabyte",
        80
    ),
];

// ============================================================
// 9. CS / AI / ML (~50)
// ============================================================
const CS_AI: &[MathEntry] = &[
    e!(
        "bigo_n",
        ["dophuctapn"],
        ComputerScienceAi,
        "\\mathcal{O}(n)",
        "cal(O)(n)",
        "𝒪(n)",
        "Độ phức tạp O(n)",
        "Linear complexity",
        95
    ),
    e!(
        "bigo_nlogn",
        ["dophuctapnlogn"],
        ComputerScienceAi,
        "\\mathcal{O}(n \\log n)",
        "cal(O)(n log n)",
        "𝒪(n log n)",
        "Độ phức tạp O(n log n)",
        "Linearithmic complexity",
        95
    ),
    e!(
        "bigo_n2",
        ["dophuctapnbinhphuong"],
        ComputerScienceAi,
        "\\mathcal{O}(n^2)",
        "cal(O)(n^2)",
        "𝒪(n²)",
        "Độ phức tạp O(n²)",
        "Quadratic complexity",
        90
    ),
    e!(
        "bigo_1",
        ["dophuctap1"],
        ComputerScienceAi,
        "\\mathcal{O}(1)",
        "cal(O)(1)",
        "𝒪(1)",
        "Độ phức tạp hằng số",
        "Constant complexity",
        90
    ),
    e!(
        "bigo_logn",
        ["dophuctaplogn"],
        ComputerScienceAi,
        "\\mathcal{O}(\\log n)",
        "cal(O)(log n)",
        "𝒪(log n)",
        "Độ phức tạp log",
        "Logarithmic complexity",
        90
    ),
    e!(
        "theta_notation",
        ["bacchat"],
        ComputerScienceAi,
        "\\Theta(n)",
        "Theta(n)",
        "Θ(n)",
        "Ký hiệu Θ",
        "Theta notation",
        85
    ),
    e!(
        "omega_notation",
        ["canduoi"],
        ComputerScienceAi,
        "\\Omega(n)",
        "Omega(n)",
        "Ω(n)",
        "Ký hiệu Ω (can dưới)",
        "Omega notation",
        85
    ),
    e!(
        "expectation",
        ["kyvong"],
        ComputerScienceAi,
        "\\mathbb{E}[X]",
        "EE[X]",
        "𝔼[X]",
        "Kỳ vọng",
        "Expectation",
        95
    ),
    e!(
        "loss_grad",
        ["gradienhammatmat"],
        ComputerScienceAi,
        "\\nabla_\\theta \\mathcal{L}",
        "nabla_theta cal(L)",
        "∇_θℒ",
        "Gradient hàm mất mát",
        "Loss gradient",
        95
    ),
    e!(
        "l2norm",
        ["chuanL2"],
        ComputerScienceAi,
        "\\|x\\|_2",
        "norm(x)_2",
        "‖x‖₂",
        "Chuẩn L2",
        "L2 norm",
        95
    ),
    e!(
        "l1norm",
        ["chuanL1"],
        ComputerScienceAi,
        "\\|x\\|_1",
        "norm(x)_1",
        "‖x‖₁",
        "Chuẩn L1",
        "L1 norm",
        90
    ),
    e!(
        "hadamard",
        ["nhantungphantu"],
        ComputerScienceAi,
        "\\odot",
        "odot",
        "⊙",
        "Tích Hadamard",
        "Hadamard product",
        85
    ),
    e!(
        "crossentropy",
        ["entropycheo"],
        ComputerScienceAi,
        "-\\sum y \\log \\hat{y}",
        "-sum y log yhat",
        "−Σy log ŷ",
        "Mất mát entropy chéo",
        "Cross-entropy loss",
        90
    ),
    e!(
        "mse_loss",
        ["saisobinhphuong"],
        ComputerScienceAi,
        "\\frac{1}{n}\\sum (y-\\hat{y})^2",
        "1/n sum (y - yhat)^2",
        "1/nΣ(y−ŷ)²",
        "Sai số toàn phương trung bình",
        "Mean squared error",
        90
    ),
    e!(
        "backprop",
        ["lantruyennguoc"],
        ComputerScienceAi,
        "\\mathrm{backprop}",
        "backprop",
        "backprop",
        "Lan truyền ngược",
        "Backpropagation",
        85
    ),
    e!(
        "sgd_opt",
        ["toiuusgd"],
        ComputerScienceAi,
        "\\mathrm{SGD}",
        "SGD",
        "SGD",
        "Thuật toán SGD",
        "SGD optimizer",
        85
    ),
    e!(
        "adam_opt",
        ["toiuuadam"],
        ComputerScienceAi,
        "\\mathrm{Adam}",
        "Adam",
        "Adam",
        "Thuật toán Adam",
        "Adam optimizer",
        85
    ),
    e!(
        "dropout_reg",
        ["dropout"],
        ComputerScienceAi,
        "\\mathrm{Dropout}",
        "Dropout",
        "Dropout",
        "Regularization dropout",
        "Dropout regularization",
        80
    ),
    e!(
        "batchnorm",
        ["batchnorm"],
        ComputerScienceAi,
        "\\mathrm{BatchNorm}",
        "BatchNorm",
        "BatchNorm",
        "Chuẩn hoá theo batch",
        "Batch normalization",
        80
    ),
    e!(
        "cnn",
        ["mangtichchap"],
        ComputerScienceAi,
        "\\mathrm{CNN}",
        "CNN",
        "CNN",
        "Mạng tích chập",
        "Convolutional neural network",
        85
    ),
    e!(
        "rnn",
        ["manghoctructiep"],
        ComputerScienceAi,
        "\\mathrm{RNN}",
        "RNN",
        "RNN",
        "Mạng nơ-ron hồi tiếp",
        "Recurrent neural network",
        85
    ),
    e!(
        "lstm_net",
        ["manglstm"],
        ComputerScienceAi,
        "\\mathrm{LSTM}",
        "LSTM",
        "LSTM",
        "Mạng LSTM",
        "Long short-term memory",
        85
    ),
    e!(
        "gru_net",
        ["manggru"],
        ComputerScienceAi,
        "\\mathrm{GRU}",
        "GRU",
        "GRU",
        "Mạng GRU",
        "Gated recurrent unit",
        80
    ),
    e!(
        "transformer_arch",
        ["kientructransformer"],
        ComputerScienceAi,
        "\\mathrm{Transformer}",
        "Transformer",
        "Transformer",
        "Kiến trúc Transformer",
        "Transformer architecture",
        90
    ),
    e!(
        "attention_mech",
        ["chuy"],
        ComputerScienceAi,
        "\\mathrm{Attention}",
        "Attention",
        "Attention",
        "Cơ chế chú ý",
        "Attention mechanism",
        90
    ),
    e!(
        "embedding_layer",
        ["lopembedding"],
        ComputerScienceAi,
        "\\mathrm{Embedding}",
        "Embedding",
        "Embedding",
        "Lớp embedding",
        "Embedding layer",
        80
    ),
    e!(
        "accuracy",
        ["dochinhxac"],
        ComputerScienceAi,
        "\\mathrm{Acc}",
        "Acc",
        "Acc",
        "Độ chính xác",
        "Accuracy",
        85
    ),
    e!(
        "precision_m",
        ["dochinhxacduong"],
        ComputerScienceAi,
        "\\mathrm{P}",
        "P",
        "P",
        "Precision",
        "Precision",
        85
    ),
    e!(
        "recall_m",
        ["dophu"],
        ComputerScienceAi,
        "\\mathrm{R}",
        "R",
        "R",
        "Recall",
        "Recall",
        85
    ),
    e!(
        "f1_score",
        ["diemf1"],
        ComputerScienceAi,
        "F_1",
        "F_1",
        "F₁",
        "Điểm F1",
        "F1 score",
        85
    ),
    e!(
        "auc_metric",
        ["dientichduoiduongROC"],
        ComputerScienceAi,
        "\\mathrm{AUC}",
        "AUC",
        "AUC",
        "Diện tích dưới đường ROC",
        "Area under ROC curve",
        80
    ),
    e!(
        "roc_curve",
        ["duongROC"],
        ComputerScienceAi,
        "\\mathrm{ROC}",
        "ROC",
        "ROC",
        "Đường ROC",
        "ROC curve",
        80
    ),
    e!(
        "confusion_matrix",
        ["matrannhamlan"],
        ComputerScienceAi,
        "\\mathrm{CM}",
        "CM",
        "CM",
        "Ma trận nhầm lẫn",
        "Confusion matrix",
        80
    ),
    e!(
        "learning_rate",
        ["tocdohoc"],
        ComputerScienceAi,
        "\\alpha",
        "alpha",
        "α",
        "Tốc độ học",
        "Learning rate",
        90
    ),
    e!(
        "epoch",
        ["vonghoc"],
        ComputerScienceAi,
        "\\mathrm{epoch}",
        "epoch",
        "epoch",
        "Vòng học (epoch)",
        "Epoch",
        80
    ),
    e!(
        "overfitting",
        ["hocvet"],
        ComputerScienceAi,
        "\\mathrm{overfit}",
        "overfit",
        "overfit",
        "Quá khớp dữ liệu",
        "Overfitting",
        80
    ),
    e!(
        "underfitting",
        ["hocthieu"],
        ComputerScienceAi,
        "\\mathrm{underfit}",
        "underfit",
        "underfit",
        "Học thiếu",
        "Underfitting",
        75
    ),
    e!(
        "regularization",
        ["chuanhoa"],
        ComputerScienceAi,
        "\\lambda\\|w\\|^2",
        "lambda norm(w)^2",
        "λ‖w‖²",
        "Thành phần regularize",
        "Regularization term",
        85
    ),
    e!(
        "gradient_clip",
        ["catgradien"],
        ComputerScienceAi,
        "\\|\\nabla\\| \\le c",
        "norm(nabla) <= c",
        "‖∇‖≤c",
        "Cắt gradient",
        "Gradient clipping",
        75
    ),
    e!(
        "dot_product",
        [],
        ComputerScienceAi,
        "\\vec{a} \\cdot \\vec{b}",
        "dot(a, b)",
        "a⃗·b⃗",
        "Tích vô hướng",
        "Dot product",
        90
    ),
    e!(
        "cross_product",
        ["nhancohuong"],
        ComputerScienceAi,
        "\\vec{a} \\times \\vec{b}",
        "cross(a, b)",
        "a⃗×b⃗",
        "Tích có hướng",
        "Cross product",
        90
    ),
    e!(
        "matrix_mult",
        ["nhanmatran"],
        ComputerScienceAi,
        "AB",
        "A B",
        "AB",
        "Nhân ma trận",
        "Matrix multiplication",
        85
    ),
    e!(
        "identity_matrix",
        ["matrandonvi"],
        ComputerScienceAi,
        "I_n",
        "I_n",
        "Iₙ",
        "Ma trận đơn vị",
        "Identity matrix",
        85
    ),
    e!(
        "tensor_rank",
        ["tenxor"],
        ComputerScienceAi,
        "\\mathcal{T}",
        "cal(T)",
        "𝒯",
        "Tensor",
        "Tensor",
        80
    ),
    e!(
        "kernel_fn",
        ["hamnhan"],
        ComputerScienceAi,
        "K(x_i, x_j)",
        "K(x_i, x_j)",
        "K(xᵢ,xⱼ)",
        "Hàm kernel (SVM)",
        "Kernel function",
        80
    ),
    e!(
        "nll_loss",
        ["matmatnhuti"],
        ComputerScienceAi,
        "-\\log p(y|x)",
        "-log p(y | x)",
        "−log p(y|x)",
        "Negative log-likelihood",
        "Negative log-likelihood",
        80
    ),
    e!(
        "variational_inference",
        ["suybienphan"],
        ComputerScienceAi,
        "q(z) \\approx p(z|x)",
        "q(z) approx p(z | x)",
        "q(z)≈p(z|x)",
        "Suy luận biến phân",
        "Variational inference",
        75
    ),
    e!(
        "llm_token",
        ["token"],
        ComputerScienceAi,
        "\\mathrm{token}",
        "token",
        "token",
        "Token (mô hình ngôn ngữ)",
        "Language-model token",
        80
    ),
    e!(
        "sample_std",
        ["dolechchuanmau"],
        ComputerScienceAi,
        "s",
        "s",
        "s",
        "Độ lệch chuẩn mẫu",
        "Sample standard deviation",
        85
    ),
    e!(
        "z_score",
        ["diemz"],
        ComputerScienceAi,
        "z = (x-\\mu)/\\sigma",
        "z = (x - mu) / sigma",
        "z = (x−µ)/σ",
        "Điểm chuẩn z",
        "Z-score",
        90
    ),
];

// ============================================================
// 10. QUANT / TÀI CHÍNH (~40)
// ============================================================
const QUANT: &[MathEntry] = &[
    e!(
        "delta_opt",
        ["deltaquyenchon"],
        QuantFinance,
        "\\Delta",
        "Delta",
        "Δ",
        "Delta quyền chọn",
        "Option delta",
        95
    ),
    e!(
        "gamma_opt",
        ["gammaquyenchon"],
        QuantFinance,
        "\\Gamma",
        "Gamma",
        "Γ",
        "Gamma quyền chọn",
        "Option gamma",
        90
    ),
    e!(
        "theta_opt",
        ["thetaquyenchon"],
        QuantFinance,
        "\\Theta",
        "Theta",
        "Θ",
        "Theta (suy giảm thời gian)",
        "Option theta (time decay)",
        90
    ),
    e!(
        "vega_opt",
        ["vegaquyenchon"],
        QuantFinance,
        "\\nu",
        "nu",
        "ν",
        "Vega (độ nhạy biến động)",
        "Option vega",
        90
    ),
    e!(
        "rho_opt",
        ["rhoquyenchon"],
        QuantFinance,
        "\\rho",
        "rho",
        "ρ",
        "Rho (độ nhạy lãi suất)",
        "Option rho",
        85
    ),
    e!(
        "VaR",
        ["giatriruiro"],
        QuantFinance,
        "\\mathrm{VaR}",
        "VaR",
        "VaR",
        "Value at Risk",
        "Value at Risk",
        90
    ),
    e!(
        "CVaR",
        ["giatriruirocodieukien"],
        QuantFinance,
        "\\mathrm{CVaR}",
        "CVaR",
        "CVaR",
        "CVaR / Expected Shortfall",
        "Conditional VaR",
        85
    ),
    e!(
        "sharpe_ratio",
        ["tisosharpe"],
        QuantFinance,
        "\\mathrm{Sharpe}",
        "Sharpe",
        "Sharpe",
        "Tỷ số Sharpe",
        "Sharpe ratio",
        90
    ),
    e!(
        "sortino_ratio",
        ["tisosortino"],
        QuantFinance,
        "\\mathrm{Sortino}",
        "Sortino",
        "Sortino",
        "Tỷ số Sortino",
        "Sortino ratio",
        80
    ),
    e!(
        "beta_stock",
        ["betacophieu"],
        QuantFinance,
        "\\beta",
        "beta",
        "β",
        "Beta cổ phiếu",
        "Stock beta",
        90
    ),
    e!(
        "alpha_stock",
        ["alphacophieu"],
        QuantFinance,
        "\\alpha",
        "alpha",
        "α",
        "Alpha cổ phiếu",
        "Stock alpha",
        90
    ),
    e!(
        "capm_model",
        ["mohinhcapm"],
        QuantFinance,
        "E[R_i] = R_f + \\beta_i(E[R_m] - R_f)",
        "E[R_i] = R_f + beta_i (E[R_m] - R_f)",
        "E[Rᵢ] = Rƒ + βᵢ(E[Rₘ]−Rƒ)",
        "Mô hình CAPM",
        "CAPM model",
        90
    ),
    e!(
        "blackscholes",
        ["congthucblackscholes"],
        QuantFinance,
        "C = S_0 N(d_1) - K e^{-rT} N(d_2)",
        "C = S_0 N(d_1) - K e^(-r T) N(d_2)",
        "C = S₀N(d₁) − Ke⁻ʳᵀN(d₂)",
        "Công thức Black–Scholes",
        "Black–Scholes formula",
        95
    ),
    e!(
        "NPV",
        ["giatrihientairong"],
        QuantFinance,
        "\\mathrm{NPV} = \\sum \\frac{CF_t}{(1+r)^t}",
        "NPV = sum CF_t / (1 + r)^t",
        "NPV = ΣCFₜ/(1+r)ᵗ",
        "Giá trị hiện tại ròng",
        "Net present value",
        95
    ),
    e!(
        "IRR",
        ["tylehoanvonnoibo"],
        QuantFinance,
        "\\mathrm{IRR}",
        "IRR",
        "IRR",
        "Tỷ lệ hoàn vốn nội bộ",
        "Internal rate of return",
        90
    ),
    e!(
        "WACC",
        ["chiphivonbinhquan"],
        QuantFinance,
        "\\mathrm{WACC}",
        "WACC",
        "WACC",
        "Chi phí vốn bình quân",
        "Weighted average cost of capital",
        85
    ),
    e!(
        "EBITDA",
        ["loinhuantruocthuelaivaykhauhao"],
        QuantFinance,
        "\\mathrm{EBITDA}",
        "EBITDA",
        "EBITDA",
        "EBITDA",
        "EBITDA",
        90
    ),
    e!(
        "EPS",
        ["loinhuantrencophieu"],
        QuantFinance,
        "\\mathrm{EPS}",
        "EPS",
        "EPS",
        "Lợi nhuận trên cổ phiếu",
        "Earnings per share",
        85
    ),
    e!(
        "PE_ratio",
        ["tisogialoinhuan"],
        QuantFinance,
        "\\mathrm{P/E}",
        "P / E",
        "P/E",
        "Tỷ số giá/lợi nhuận",
        "Price-to-earnings ratio",
        85
    ),
    e!(
        "ROE",
        ["loinhuantrenvoncsh"],
        QuantFinance,
        "\\mathrm{ROE}",
        "ROE",
        "ROE",
        "Tỷ suất sinh lời trên vốn chủ sở hữu",
        "Return on equity",
        85
    ),
    e!(
        "ROA",
        ["loinhuantrentaisan"],
        QuantFinance,
        "\\mathrm{ROA}",
        "ROA",
        "ROA",
        "Tỷ suất sinh lời trên tài sản",
        "Return on assets",
        85
    ),
    e!(
        "CAGR",
        ["tocdotangtruonghotnam"],
        QuantFinance,
        "\\mathrm{CAGR}",
        "CAGR",
        "CAGR",
        "Tăng trưởng kép hằng năm",
        "Compound annual growth rate",
        85
    ),
    e!(
        "ROI",
        ["tylehoanvon"],
        QuantFinance,
        "\\mathrm{ROI}",
        "ROI",
        "ROI",
        "Tỷ lệ hoàn vốn",
        "Return on investment",
        85
    ),
    e!(
        "volatility",
        ["dobiendong"],
        QuantFinance,
        "\\sigma",
        "sigma",
        "σ",
        "Độ biến động",
        "Volatility",
        90
    ),
    e!(
        "stdev_fin",
        ["dolechchuantc"],
        QuantFinance,
        "\\sigma",
        "sigma",
        "σ",
        "Độ lệch chuẩn",
        "Standard deviation",
        90
    ),
    e!(
        "variance_fin",
        ["phuongsaitc"],
        QuantFinance,
        "\\sigma^2",
        "sigma^2",
        "σ²",
        "Phương sai",
        "Variance",
        90
    ),
    e!(
        "correlation",
        ["hesotuongquantc"],
        QuantFinance,
        "\\rho_{X,Y}",
        "rho_(X, Y)",
        "ρₓᵧ",
        "Hệ số tương quan",
        "Correlation coefficient",
        90
    ),
    e!(
        "drawdown",
        ["mucsutgiam"],
        QuantFinance,
        "\\mathrm{DD}",
        "DD",
        "DD",
        "Mức sụt giảm tối đa",
        "Drawdown",
        80
    ),
    e!(
        "hedging",
        ["phonghan"],
        QuantFinance,
        "\\mathrm{hedge}",
        "hedge",
        "hedge",
        "Phòng hộ rủi ro",
        "Hedging",
        80
    ),
    e!(
        "arbitrage",
        ["kinhdoanhchanhlech"],
        QuantFinance,
        "\\mathrm{arb}",
        "arb",
        "arb",
        "Kinh doanh chênh lệch giá",
        "Arbitrage",
        80
    ),
    e!(
        "spot_price",
        ["giagiaongay"],
        QuantFinance,
        "S_t",
        "S_t",
        "Sₜ",
        "Giá giao ngay tại t",
        "Spot price at t",
        85
    ),
    e!(
        "strike_price",
        ["giakhangdinh"],
        QuantFinance,
        "K",
        "K",
        "K",
        "Giá thực hiện quyền chọn",
        "Strike price",
        85
    ),
    e!(
        "risk_free",
        ["laisuatphiruiro"],
        QuantFinance,
        "r_f",
        "r_f",
        "r_f",
        "Lãi suất phi rủi ro",
        "Risk-free rate",
        85
    ),
    e!(
        "maturity",
        ["daohan"],
        QuantFinance,
        "T",
        "T",
        "T",
        "Thời hạn",
        "Maturity",
        80
    ),
    e!(
        "coupon",
        ["laitraiphieu"],
        QuantFinance,
        "c",
        "c",
        "c",
        "Lãi trái phiếu",
        "Coupon rate",
        75
    ),
    e!(
        "yield_fn",
        ["tylesinhloi"],
        QuantFinance,
        "y",
        "y",
        "y",
        "Tỷ lệ sinh lời",
        "Yield",
        80
    ),
    e!(
        "CDS",
        ["hopdonghoandoiruiro"],
        QuantFinance,
        "\\mathrm{CDS}",
        "CDS",
        "CDS",
        "Hợp đồng hoán đổi rủi ro tín dụng",
        "Credit default swap",
        80
    ),
    e!(
        "swaption",
        ["quyenchonhoandoi"],
        QuantFinance,
        "\\mathrm{swaption}",
        "swaption",
        "swaption",
        "Quyền chọn hoán đổi",
        "Swaption",
        75
    ),
    e!(
        "greeks_total",
        ["cachesogreek"],
        QuantFinance,
        "\\Theta, \\Gamma, \\Delta, \\nu, \\rho",
        "Theta, Gamma, Delta, nu, rho",
        "Θ,Γ,Δ,ν,ρ",
        "Các hệ số Greeks",
        "The Greeks",
        80
    ),
    e!(
        "markowitz",
        ["danhmucmarkowitz"],
        QuantFinance,
        "\\min \\sigma^2_p",
        "min sigma_p^2",
        "min σ²ₚ",
        "Tối ưu danh mục",
        "Portfolio optimization",
        80
    ),
];

// ============================================================
// 11. CÔNG THỨC THÔNG DỤNG TOÁN HỌC (~50)
// ============================================================
const FORMULAS: &[MathEntry] = &[
    e!(
        "pythagorean",
        ["dinhlypythagoras", "pytago"],
        PureMath,
        "a^2 + b^2 = c^2",
        "a^2 + b^2 = c^2",
        "a²+b²=c²",
        "Định lý Pythagoras",
        "Pythagorean theorem",
        95
    ),
    e!(
        "euler_identity",
        ["dongnhateuler"],
        PureMath,
        "e^{i\\pi} + 1 = 0",
        "e^(i pi) + 1 = 0",
        "e^{iπ}+1=0",
        "Đồng nhất thức Euler",
        "Euler's identity",
        95
    ),
    e!(
        "quadratic_formula",
        ["congthucnghiem"],
        PureMath,
        "x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}",
        "x = frac(-b ± sqrt(b^2 - 4 a c), 2 a)",
        "x = (−b±√(b²−4ac))/2a",
        "Công thức nghiệm phương trình bậc hai",
        "Quadratic formula",
        95
    ),
    e!(
        "sum_arithmetic",
        ["congbacthang"],
        PureMath,
        "S_n = \\frac{n(a_1 + a_n)}{2}",
        "S_n = frac(n (a_1 + a_n), 2)",
        "Sₙ = n(a₁+aₙ)/2",
        "Tổng cấp số cộng",
        "Arithmetic series sum",
        90
    ),
    e!(
        "sum_geometric",
        ["congnhan"],
        PureMath,
        "S_n = a_1\\frac{r^n - 1}{r - 1}",
        "S_n = a_1 frac(r^n - 1, r - 1)",
        "Sₙ = a₁(rⁿ−1)/(r−1)",
        "Tổng cấp số nhân",
        "Geometric series sum",
        90
    ),
    e!(
        "circle_area",
        ["dientichhinhtron"],
        PureMath,
        "A = \\pi r^2",
        "A = pi r^2",
        "A = πr²",
        "Diện tích hình tròn",
        "Circle area",
        95
    ),
    e!(
        "circle_circum",
        ["chuvihinhtron"],
        PureMath,
        "C = 2\\pi r",
        "C = 2 pi r",
        "C = 2πr",
        "Chu vi hình tròn",
        "Circle circumference",
        95
    ),
    e!(
        "sphere_volume",
        ["thetichcau"],
        PureMath,
        "V = \\tfrac{4}{3}\\pi r^3",
        "V = 4/3 pi r^3",
        "V = 4/3πr³",
        "Thể tích hình cầu",
        "Sphere volume",
        90
    ),
    e!(
        "triangle_area",
        ["dientichtamgiac"],
        PureMath,
        "A = \\tfrac{1}{2}bh",
        "A = 1/2 b h",
        "A = ½bh",
        "Diện tích tam giác",
        "Triangle area",
        90
    ),
    e!(
        "cosine_rule",
        ["dinhlycos"],
        PureMath,
        "c^2 = a^2 + b^2 - 2ab\\cos C",
        "c^2 = a^2 + b^2 - 2 a b cos C",
        "c²=a²+b²−2ab·cosC",
        "Định lý cosin",
        "Law of cosines",
        90
    ),
    e!(
        "sine_rule",
        ["dinhlysin"],
        PureMath,
        "\\frac{a}{\\sin A} = \\frac{b}{\\sin B}",
        "a / sin A = b / sin B",
        "a/sinA = b/sinB",
        "Định lý sin",
        "Law of sines",
        90
    ),
    e!(
        "derivative_def",
        ["dinhnghiadaoham"],
        PureMath,
        "f'(x) = \\lim_{h \\to 0}\\frac{f(x+h)-f(x)}{h}",
        "f'(x) = lim_(h -> 0) frac(f(x+h) - f(x), h)",
        "f′(x)=lim[h→0](f(x+h)−f(x))/h",
        "Định nghĩa đạo hàm",
        "Definition of derivative",
        95
    ),
    e!(
        "product_rule",
        ["daohamtich"],
        PureMath,
        "(uv)' = u'v + uv'",
        "(u v)' = u' v + u v'",
        "(uv)′=u′v+uv′",
        "Quy tắc tích",
        "Product rule",
        90
    ),
    e!(
        "quotient_rule",
        ["daohamthuong"],
        PureMath,
        "\\left(\\frac{u}{v}\\right)' = \\frac{u'v - uv'}{v^2}",
        "frac(u, v)' = frac(u' v - u v', v^2)",
        "(u/v)′=(u′v−uv′)/v²",
        "Quy tắc thương",
        "Quotient rule",
        90
    ),
    e!(
        "chain_rule",
        ["daohamhop"],
        PureMath,
        "(f \\circ g)' = f'(g) \\cdot g'",
        "(f op g)' = f'(g) dot g'",
        "(f∘g)′=f′(g)·g′",
        "Quy tắc chuỗi",
        "Chain rule",
        90
    ),
    e!(
        "fundamental_theorem",
        ["nguyenlycoban"],
        PureMath,
        "\\int_a^b f'(x)\\,dx = f(b) - f(a)",
        "integral_a^b f'(x) dx = f(b) - f(a)",
        "∫ₐᵇf′(x)dx = f(b)−f(a)",
        "Nguyên lý cơ bản giải tích",
        "Fundamental theorem of calculus",
        95
    ),
    e!(
        "taylor_series",
        ["khaitrientaylor"],
        PureMath,
        "f(x) \\approx \\sum \\frac{f^{(n)}(a)}{n!}(x-a)^n",
        "f(x) approx sum frac(f^(n)(a), n!) (x - a)^n",
        "f(x)≈Σ[f⁽ⁿ⁾(a)/n!](x−a)ⁿ",
        "Khai triển Taylor",
        "Taylor series",
        90
    ),
    e!(
        "maclaurin_e",
        ["maclaurinemux"],
        PureMath,
        "e^x = \\sum \\frac{x^n}{n!}",
        "e^x = sum x^n / n!",
        "eˣ = Σxⁿ/n!",
        "Khai triển eˣ",
        "Maclaurin series of eˣ",
        90
    ),
    e!(
        "log_identity",
        ["tinhchatlogarit"],
        PureMath,
        "\\log(ab) = \\log a + \\log b",
        "log(a b) = log a + log b",
        "log(ab)=log a+log b",
        "Tính chất logarit",
        "Logarithm product rule",
        90
    ),
    e!(
        "change_of_base",
        ["doicosolog"],
        PureMath,
        "\\log_b x = \\frac{\\ln x}{\\ln b}",
        "log_b x = ln x / ln b",
        "logᵦx = lnx/lnb",
        "Đổi cơ số logarit",
        "Change of base",
        85
    ),
    e!(
        "bayes_theorem",
        ["dinhlybayes"],
        PureMath,
        "P(A|B) = \\frac{P(B|A)P(A)}{P(B)}",
        "P(A | B) = frac(P(B | A) P(A), P(B))",
        "P(A|B) = P(B|A)P(A)/P(B)",
        "Định lý Bayes",
        "Bayes' theorem",
        95
    ),
    e!(
        "law_total_prob",
        ["xacsuattoanphan"],
        PureMath,
        "P(A) = \\sum P(A|B_i)P(B_i)",
        "P(A) = sum P(A | B_i) P(B_i)",
        "P(A) = ΣP(A|Bᵢ)P(Bᵢ)",
        "Xác suất toàn phần",
        "Law of total probability",
        85
    ),
    e!(
        "binomial_coef",
        ["tohopchapk"],
        PureMath,
        "\\binom{n}{k} = \\frac{n!}{k!(n-k)!}",
        "binom(n, k) = frac(n!, k! (n - k)!)",
        "C(n,k) = n!/k!(n−k)!",
        "Công thức tổ hợp",
        "Binomial coefficient",
        95
    ),
    e!(
        "permutation",
        ["hoanvi"],
        PureMath,
        "P_n = n!",
        "P_n = n!",
        "Pₙ = n!",
        "Hoán vị",
        "Permutation",
        90
    ),
    e!(
        "eulers_phi",
        ["hameuler"],
        PureMath,
        "\\varphi(n)",
        "phi(n)",
        "φ(n)",
        "Hàm φ Euler",
        "Euler's totient",
        85
    ),
    e!(
        "fermat_little",
        ["dinhlynhofermat"],
        PureMath,
        "a^{p-1} \\equiv 1 \\pmod p",
        "a^(p - 1) op(==) 1 mod p",
        "a^(p−1) ≡ 1 (mod p)",
        "Định lý nhỏ Fermat",
        "Fermat's little theorem",
        85
    ),
    e!(
        "modular_add",
        ["congmodulo"],
        PureMath,
        "(a + b) \\bmod n",
        "(a + b) mod n",
        "(a+b) mod n",
        "Cộng modulo",
        "Modular addition",
        85
    ),
    e!(
        "mean_def",
        ["giatritrungbinh"],
        PureMath,
        "\\bar{x} = \\frac{1}{n}\\sum x_i",
        "bar(x) = 1/n sum x_i",
        "x̄ = (1/n)Σxᵢ",
        "Giá trị trung bình",
        "Arithmetic mean",
        95
    ),
    e!(
        "variance_def",
        ["phuongsaidinhnghia"],
        PureMath,
        "\\sigma^2 = \\frac{1}{n}\\sum (x_i - \\bar{x})^2",
        "sigma^2 = 1/n sum (x_i - bar(x))^2",
        "σ² = (1/n)Σ(xᵢ−x̄)²",
        "Phương sai",
        "Variance",
        95
    ),
    e!(
        "stdev_def",
        ["dolechchuandinhnghia"],
        PureMath,
        "\\sigma = \\sqrt{\\sigma^2}",
        "sigma = sqrt(sigma^2)",
        "σ = √σ²",
        "Độ lệch chuẩn",
        "Standard deviation",
        90
    ),
    e!(
        "correlation_def",
        ["tuongquandinhnghia"],
        PureMath,
        "\\rho = \\frac{\\mathrm{Cov}(X,Y)}{\\sigma_X\\sigma_Y}",
        "rho = Cov(X, Y) / (sigma_X sigma_Y)",
        "ρ = Cov(X,Y)/(σₓσᵧ)",
        "Hệ số tương quan",
        "Correlation formula",
        90
    ),
    e!(
        "linear_reg",
        ["hoiquytuyentinh"],
        PureMath,
        "\\hat{y} = wx + b",
        "hat(y) = w x + b",
        "ŷ = wx + b",
        "Hồi quy tuyến tính",
        "Linear regression",
        90
    ),
    e!(
        "normal_pdf",
        ["matdophoichuan"],
        PureMath,
        "f(x) = \\frac{1}{\\sigma\\sqrt{2\\pi}}e^{-(x-\\mu)^2/2\\sigma^2}",
        "f(x) = 1/(sigma sqrt(2 pi)) e^(-(x - mu)^2 / (2 sigma^2))",
        "f(x) = (1/(σ√2π))e^(−(x−µ)²/2σ²)",
        "Mật độ phân phối chuẩn",
        "Normal distribution PDF",
        90
    ),
    e!(
        "law_large_numbers",
        ["dinhluatsolon"],
        PureMath,
        "\\bar{X}_n \\to \\mu",
        "bar(X)_n -> mu",
        "X̄ₙ → µ",
        "Định luật số lớn",
        "Law of large numbers",
        90
    ),
    e!(
        "central_limit",
        ["dinhlygioihantrungtam"],
        PureMath,
        "\\bar{X} \\sim N(\\mu, \\sigma^2/n)",
        "bar(X) cal(N)(mu, sigma^2 / n)",
        "X̄ ~ 𝒩(µ,σ²/n)",
        "Định lý giới hạn trung tâm",
        "Central limit theorem",
        90
    ),
    e!(
        "euler_characteristic",
        ["dactrungeuler"],
        PureMath,
        "V - E + F = 2",
        "V - E + F = 2",
        "V−E+F=2",
        "Đặc trưng Euler",
        "Euler characteristic",
        85
    ),
    e!(
        "fibonacci_rec",
        ["dayfibonacci"],
        PureMath,
        "F_n = F_{n-1} + F_{n-2}",
        "F_n = F_(n-1) + F_(n-2)",
        "Fₙ = Fₙ₋₁+Fₙ₋₂",
        "Dãy Fibonacci",
        "Fibonacci recurrence",
        90
    ),
    e!(
        "golden_ratio",
        ["tilevangdinhnghia"],
        PureMath,
        "\\varphi = \\frac{1+\\sqrt{5}}{2}",
        "phi = frac(1 + sqrt(5), 2)",
        "φ = (1+√5)/2",
        "Tỉ lệ vàng",
        "Golden ratio",
        90
    ),
    e!(
        "stirling_approx",
        ["xapxistirling"],
        PureMath,
        "n! \\approx \\sqrt{2\\pi n}(n/e)^n",
        "n! approx sqrt(2 pi n) (n / e)^n",
        "n! ≈ √(2πn)(n/e)ⁿ",
        "Xấp xỉ Stirling",
        "Stirling's approximation",
        85
    ),
    e!(
        "newton_method",
        ["phuongphapnewton"],
        PureMath,
        "x_{n+1} = x_n - \\frac{f(x_n)}{f'(x_n)}",
        "x_(n+1) = x_n - f(x_n) / f'(x_n)",
        "xₙ₊₁ = xₙ − f(xₙ)/f′(xₙ)",
        "Phương pháp Newton–Raphson",
        "Newton's method",
        90
    ),
    e!(
        "compound_interest",
        ["laikep"],
        QuantFinance,
        "A = P(1 + r/n)^{nt}",
        "A = P (1 + r / n)^(n t)",
        "A = P(1+r/n)ⁿᵗ",
        "Công thức lãi kép",
        "Compound interest",
        90
    ),
    e!(
        "simple_interest",
        ["laidon"],
        QuantFinance,
        "A = P(1 + rt)",
        "A = P (1 + r t)",
        "A = P(1+rt)",
        "Công thức lãi đơn",
        "Simple interest",
        85
    ),
    e!(
        "continuously_comp",
        ["laikeplientuc"],
        QuantFinance,
        "A = Pe^{rt}",
        "A = P e^(r t)",
        "A = Peʳᵗ",
        "Lãi kép liên tục",
        "Continuous compounding",
        85
    ),
    e!(
        "distance_formula",
        ["khoangcachdiem"],
        PureMath,
        "d = \\sqrt{(x_2-x_1)^2 + (y_2-y_1)^2}",
        "d = sqrt((x_2 - x_1)^2 + (y_2 - y_1)^2)",
        "d = √((x₂−x₁)²+(y₂−y₁)²)",
        "Khoảng cách hai điểm",
        "Distance formula",
        95
    ),
    e!(
        "midpoint",
        ["trungdiem"],
        PureMath,
        "M = \\left(\\frac{x_1+x_2}{2}, \\frac{y_1+y_2}{2}\\right)",
        "M = (frac(x_1 + x_2, 2), frac(y_1 + y_2, 2))",
        "M = ((x₁+x₂)/2, (y₁+y₂)/2)",
        "Tọa độ trung điểm",
        "Midpoint formula",
        90
    ),
    e!(
        "slope_formula",
        ["hesogoc"],
        PureMath,
        "m = \\frac{y_2 - y_1}{x_2 - x_1}",
        "m = frac(y_2 - y_1, x_2 - x_1)",
        "m = (y₂−y₁)/(x₂−x₁)",
        "Hệ số góc đường thẳng",
        "Slope formula",
        95
    ),
    e!(
        "line_eq",
        ["phuongtrinhduongthang"],
        PureMath,
        "y = mx + b",
        "y = m x + b",
        "y = mx+b",
        "Phương trình đường thẳng",
        "Slope-intercept form",
        95
    ),
    e!(
        "ohm_power",
        ["congsuatdien"],
        PhysicsMech,
        "P = VI",
        "P = V I",
        "P = VI",
        "Công suất điện",
        "Electrical power",
        95
    ),
    e!(
        "joule_heating",
        ["nhiettoandung"],
        PhysicsMech,
        "Q = I^2 R t",
        "Q = I^2 R t",
        "Q = I²Rt",
        "Nhiệt toả Joule",
        "Joule heating",
        90
    ),
    e!(
        "kinematic_v",
        ["phuongtrinhvantoc"],
        PhysicsMech,
        "v = v_0 + at",
        "v = v_0 + a t",
        "v = v₀+at",
        "Phương trình vận tốc",
        "Kinematic equation (velocity)",
        95
    ),
    e!(
        "kinematic_x",
        ["phuongtrinhvitri"],
        PhysicsMech,
        "x = x_0 + v_0 t + \\tfrac{1}{2}at^2",
        "x = x_0 + v_0 t + 1/2 a t^2",
        "x = x₀+v₀t+½at²",
        "Phương trình vị trí",
        "Kinematic equation (position)",
        95
    ),
    e!(
        "kinematic_v2",
        ["quanhedichuyen"],
        PhysicsMech,
        "v^2 = v_0^2 + 2a\\Delta x",
        "v^2 = v_0^2 + 2 a Delta x",
        "v² = v₀²+2aΔx",
        "Quan hệ vận tốc–quãng đường",
        "Kinematic equation (no time)",
        95
    ),
];

// ============================================================
// 12. SUPER/SUBSCRIPT — macro + test
// ============================================================
macro_rules! supsub_table {
    ( $( $id:ident : ($trig:literal, $sup:literal, $sub:literal, $supu:literal, $subu:literal) ),* $(,)? ) => {
        const SUPSUB: &[MathEntry] = &[
            $( e!(concat!("sup", $trig), [concat!("^", $trig)], PureMath,
                concat!("^{", $trig, "}"), concat!("^", $trig), $supu,
                concat!("Mũ trên ", $trig), concat!("Superscript ", $trig), 85),
            e!(concat!("sub", $trig), [concat!("_", $trig)], PureMath,
                concat!("_{", $trig, "}"), concat!("_", $trig), $subu,
                concat!("Chỉ số dưới ", $trig), concat!("Subscript ", $trig), 85), )*
        ];
        #[cfg(test)]
        mod supsub_tests {
            use super::{lookup_exact};
            use crate::config::Discipline;
            $( #[test] fn $id() {
                let sup = lookup_exact(concat!("sup", $trig)).expect("sup missing");
                let sub = lookup_exact(concat!("sub", $trig)).expect("sub missing");
                assert_eq!(sup.unicode, $supu);
                assert_eq!(sub.unicode, $subu);
                assert_eq!(sup.discipline, Discipline::PureMath);
            } )*
        }
    };
}

supsub_table! {
    s_0: ("0", "", "", "⁰", "₀"), s_1: ("1", "", "", "¹", "₁"),
    s_2: ("2", "", "", "²", "₂"), s_3: ("3", "", "", "³", "₃"),
    s_4: ("4", "", "", "⁴", "₄"), s_5: ("5", "", "", "⁵", "₅"),
    s_6: ("6", "", "", "⁶", "₆"), s_7: ("7", "", "", "⁷", "₇"),
    s_8: ("8", "", "", "⁸", "₈"), s_9: ("9", "", "", "⁹", "₉"),
    s_plus: ("+", "", "", "⁺", "₊"), s_minus: ("-", "", "", "⁻", "₋"),
    s_n: ("n", "", "", "ⁿ", "ₙ"), s_i: ("i", "", "", "ⁱ", "ᵢ"),
    s_x: ("x", "", "", "ˣ", "ₓ"), s_t: ("t", "", "", "ᵗ", "ₜ")
}

// ============================================================
// 13. TEMPLATE POPULAR TRONG LEXICON (~30) — trigger sẵn cho search
// ============================================================
const TEMPLATE_LEX: &[MathEntry] = &[
    e!("mat2x2", [], PureMath, "\\begin{matrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{matrix}", "mat(a_(1,1), a_(1,2); a_(2,1), a_(2,2))", "[a11 a12; a21 a22]", "Ma trận 2×2", "2×2 matrix", 90),
    e!("mat3x3", [], PureMath, "\\begin{matrix} a_{11} & a_{12} & a_{13} \\\\ a_{21} & a_{22} & a_{23} \\\\ a_{31} & a_{32} & a_{33} \\end{matrix}", "mat(a_(1,1), a_(1,2), a_(1,3); a_(2,1), a_(2,2), a_(2,3); a_(3,1), a_(3,2), a_(3,3))", "[a11 a12 a13; a21 a22 a23; a31 a32 a33]", "Ma trận 3×3", "3×3 matrix", 90),
    e!("bmat2x2", [], PureMath, "\\begin{bmatrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{bmatrix}", "mat(a_(1,1), a_(1,2); a_(2,1), a_(2,2))", "[a11 a12; a21 a22]", "Ma trận 2×2 ngoặc vuông", "2×2 bmatrix", 85),
    e!("pmat2x2", [], PureMath, "\\begin{pmatrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{pmatrix}", "mat(a_(1,1), a_(1,2); a_(2,1), a_(2,2))", "[a11 a12; a21 a22]", "Ma trận 2×2 ngoặc tròn", "2×2 pmatrix", 85),
    e!("det2", [], PureMath, "\\det\\begin{vmatrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{vmatrix}", "det(mat(a_(1,1), a_(1,2); a_(2,1), a_(2,2)))", "|a11 a12; a21 a22|", "Định thức 2×2", "2×2 determinant", 85),
    e!("det3", [], PureMath, "\\det\\begin{vmatrix} a_{11} & a_{12} & a_{13} \\\\ a_{21} & a_{22} & a_{23} \\\\ a_{31} & a_{32} & a_{33} \\end{vmatrix}", "det(mat(a_(1,1), a_(1,2), a_(1,3); a_(2,1), a_(2,2), a_(2,3); a_(3,1), a_(3,2), a_(3,3)))", "|a11 a12 a13; a21 a22 a23; a31 a32 a33|", "Định thức 3×3", "3×3 determinant", 85),
    e!("hpt2", [], PureMath, "\\begin{cases} x_{1} = 0 \\\\ x_{2} = 0 \\end{cases}", "cases(x_1 = 0, x_2 = 0)", "x1 = 0; x2 = 0", "Hệ phương trình 2 ẩn", "2-unknown system", 85),
    e!("hpt3", [], PureMath, "\\begin{cases} x_{1} = 0 \\\\ x_{2} = 0 \\\\ x_{3} = 0 \\end{cases}", "cases(x_1 = 0, x_2 = 0, x_3 = 0)", "x1 = 0; x2 = 0; x3 = 0", "Hệ phương trình 3 ẩn", "3-unknown system", 85),
    e!("cases2", [], PureMath, "\\begin{cases} f(x) = \\text{khi } x = 1 \\\\ f(x) = \\text{khi } x = 2 \\end{cases}", "cases(f(x) = \"khi\" x = 1, f(x) = \"khi\" x = 2)", "f(x) = ?, x = 1; f(x) = ?, x = 2", "Hàm số điều kiện 2 nhánh", "2-case function", 85),
    e!("vec2", [], PureMath, "\\vec{v} = (v_1, v_2)", "vec(v) = (v_1, v_2)", "v⃗ = (v₁, v₂)", "Vector 2 thành phần", "2-component vector", 80),
    e!("vec3", [], PureMath, "\\vec{v} = (v_1, v_2, v_3)", "vec(v) = (v_1, v_2, v_3)", "v⃗ = (v₁, v₂, v₃)", "Vector 3 thành phần", "3-component vector", 80),
];

// ============================================================
// 14. GHI CHÚ / ĐÁNH DẤU KHOA HỌC (~35)
// ============================================================
const NOTATION: &[MathEntry] = &[
    e!(
        "therefore",
        ["vivay", "dodo"],
        PureMath,
        "\\therefore",
        "op(therefore)",
        "∴",
        "Vì vậy",
        "Therefore",
        95
    ),
    e!(
        "because",
        ["vi"],
        PureMath,
        "\\because",
        "op(because)",
        "∵",
        "Bởi vì",
        "Because",
        90
    ),
    e!(
        "qed",
        ["dpcm", "ketluan"],
        PureMath,
        "\\blacksquare",
        "qed",
        "∎",
        "Điều phải chứng minh (xong)",
        "QED",
        85
    ),
    e!(
        "checkmark",
        ["dung", "daucheck"],
        PureMath,
        "\\checkmark",
        "checkmark",
        "✓",
        "Dấu kiểm",
        "Check mark",
        85
    ),
    e!(
        "xmark",
        ["sai", "dauX"],
        PureMath,
        "\\times",
        "times",
        "✗",
        "Dấu sai",
        "Cross mark",
        80
    ),
    e!(
        "notequal_zero",
        ["khackhong"],
        PureMath,
        "\\ne 0",
        "op(!=) 0",
        "≠0",
        "Khác 0",
        "Not equal to zero",
        80
    ),
    e!(
        "approx_zero",
        ["xapxikhong"],
        PureMath,
        "\\approx 0",
        "approx 0",
        "≈0",
        "Xấp xỉ 0",
        "Approximately zero",
        80
    ),
    e!(
        "tends_to",
        ["huongtoi"],
        PureMath,
        "\\to",
        "arrow.r",
        "→",
        "Hướng tới",
        "Tends to",
        90
    ),
    e!(
        "n_equiv",
        ["dongdumodn"],
        PureMath,
        "\\equiv",
        "op(==)",
        "≡",
        "Đồng dư (mod)",
        "Congruent modulo",
        85
    ),
    e!(
        "sum_bound",
        ["tongcochan"],
        PureMath,
        "\\sum_{i=1}^{n}",
        "sum_(i = 1)^n",
        "Σᵢ₌₁ⁿ",
        "Tổng có cận",
        "Bounded sum",
        95
    ),
    e!(
        "int_bound",
        ["tichphancochan"],
        PureMath,
        "\\int_a^b",
        "integral_a^b",
        "∫ₐᵇ",
        "Tích phân có cận",
        "Bounded integral",
        95
    ),
    e!(
        "lim_x0",
        ["gioihanxve0"],
        PureMath,
        "\\lim_{x \\to 0}",
        "lim_(x -> 0)",
        "limₓ→₀",
        "Giới hạn khi x→0",
        "Limit x→0",
        90
    ),
    e!(
        "inf_sum",
        ["tongvohan"],
        PureMath,
        "\\sum_{n=1}^{\\infty}",
        "sum_(n = 1)^oo",
        "Σₙ₌₁^∞",
        "Tổng vô hạn",
        "Infinite series",
        90
    ),
    e!(
        "partial_frac",
        ["phansorieng"],
        PureMath,
        "\\frac{\\partial f}{\\partial x}",
        "frac(partial f, partial x)",
        "∂f/∂x",
        "Đạo hàm riêng theo x",
        "Partial derivative of f wrt x",
        95
    ),
    e!(
        "differential",
        ["viphan"],
        PureMath,
        "\\mathrm{d}x",
        "d x",
        "dx",
        "Vi phân",
        "Differential",
        90
    ),
    e!(
        "evaluate_at",
        ["xaitai"],
        PureMath,
        "\\big|_{x=a}",
        "eval_(x = a)",
        "|ₓ₌ₐ",
        "Xác định tại điểm",
        "Evaluate at",
        85
    ),
    e!(
        "matrix_env",
        ["moitruongmatran"],
        PureMath,
        "\\begin{matrix}\\end{matrix}",
        "mat()",
        "[·]",
        "Môi trường ma trận",
        "Matrix environment",
        80
    ),
    e!(
        "aligned_env",
        ["moitruongalign"],
        PureMath,
        "\\begin{aligned}\\end{aligned}",
        "aligned()",
        "align",
        "Môi trường căn chỉnh",
        "Aligned environment",
        80
    ),
    e!(
        "text_command",
        ["lenhtext"],
        PureMath,
        "\\text{...}",
        "\"...\"",
        "\\text",
        "Lệnh chữ trong math",
        "Text command in math",
        90
    ),
    e!(
        "mathrm_command",
        ["lenhmathrm"],
        PureMath,
        "\\mathrm{...}",
        "\"...\"",
        "\\mathrm",
        "Lệnh chữ thẳng trong math",
        "Roman text command",
        85
    ),
    e!(
        "mathbf_command",
        ["lenhmathbf"],
        PureMath,
        "\\mathbf{...}",
        "bold(...)",
        "\\mathbf",
        "Lệnh chữ đậm trong math",
        "Bold text command",
        85
    ),
    e!(
        "hat_command",
        ["lenhhat"],
        PureMath,
        "\\hat{...}",
        "hat(...)",
        "\\hat",
        "Lệnh mũ trên",
        "Hat command",
        85
    ),
    e!(
        "overbrace",
        ["ngoactrendai"],
        PureMath,
        "\\overbrace{...}",
        "overbrace(...)",
        "\\overbrace",
        "Ngoặc trên dài",
        "Overbrace",
        80
    ),
    e!(
        "underbrace",
        ["ngoacduoidai"],
        PureMath,
        "\\underbrace{...}",
        "underbrace(...)",
        "\\underbrace",
        "Ngoặc dưới dài",
        "Underbrace",
        80
    ),
    e!(
        "cancel",
        ["xoacheo"],
        PureMath,
        "\\cancel{...}",
        "cancel(...)",
        "\\cancel",
        "Gạch chéo xoá",
        "Cancel",
        80
    ),
    e!(
        "boxed",
        ["khungketqua"],
        PureMath,
        "\\boxed{...}",
        "boxed(...)",
        "\\boxed",
        "Khung kết quả",
        "Boxed result",
        85
    ),
    e!(
        "binom_command",
        ["lenhbinom"],
        PureMath,
        "\\binom{n}{k}",
        "binom(n, k)",
        "\\binom",
        "Lệnh tổ hợp",
        "Binomial command",
        85
    ),
    e!(
        "cases_env",
        ["moitruongcases"],
        PureMath,
        "\\begin{cases}\\end{cases}",
        "cases()",
        "cases",
        "Môi trường cases",
        "Cases environment",
        90
    ),
    e!(
        "displaystyle",
        ["hienthilon"],
        PureMath,
        "\\displaystyle",
        "display",
        "\\displaystyle",
        "Chế độ hiển thị lớn",
        "Display style",
        80
    ),
    e!(
        "subsup_combo",
        ["supsubcungluc"],
        PureMath,
        "x^{2}_{1}",
        "x^2 _1",
        "x²₁",
        "Mũ và chỉ số cùng lúc",
        "Superscript and subscript",
        90
    ),
    e!(
        "hat_vec",
        ["vectorcomu"],
        PureMath,
        "\\hat{\\vec{a}}",
        "hat(vec(a))",
        "â⃗",
        "Vector có mũ",
        "Hat over vector",
        80
    ),
    e!(
        "double_arrow",
        ["haimuiten"],
        PureMath,
        "\\Rightarrow\\Leftarrow",
        "op(=>) op(<=)",
        "⇒⇐",
        "Hai mũi tên kép",
        "Double arrows",
        75
    ),
    e!(
        "long_arrow",
        ["muitendai"],
        PureMath,
        "\\longrightarrow",
        "arrow.r.long",
        "⟶",
        "Mũi tên dài",
        "Long arrow",
        85
    ),
    e!(
        "mapsto_arrow",
        ["muitenanhxa"],
        PureMath,
        "\\longmapsto",
        "mapsto.long",
        "⟼",
        "Mũi tên ánh xạ dài",
        "Long mapsto",
        80
    ),
    e!(
        "neg_arrow",
        ["muitencheo"],
        PureMath,
        "\\nrightarrow",
        "arrow.r.n",
        "↛",
        "Mũi tên gạch chéo",
        "Negated arrow",
        75
    ),
];

/// Tổng hợp toàn bộ nhóm (trừ SI) — SI nằm trong `SI_TABLES`.
// ============================================================
// EXPANDED (E58) — hàm hyperbolic, hằng số vật lý, đơn vị hợp chất,
// nhóm chức hoá học, chỉ số tài chính định lượng, số nguyên không
// thứ nguyên, ký hiệu LaTeX còn thiếu (đưa tổng entry >= 1500).
// ============================================================
const EXTRA_MATH: &[MathEntry] = &[
    e!(
        "coth",
        [],
        PureMath,
        "\\coth",
        "coth",
        "cth",
        "Cotangent hyperbolic",
        "Hyperbolic cotangent",
        80
    ),
    e!(
        "arcsinh",
        [],
        PureMath,
        "\\operatorname{arcsinh}",
        "asinh",
        "arcsinh",
        "Arcsin hyperbolic",
        "Inverse hyperbolic sine",
        70
    ),
    e!(
        "arccosh",
        [],
        PureMath,
        "\\operatorname{arccosh}",
        "acosh",
        "arccosh",
        "Arccos hyperbolic",
        "Inverse hyperbolic cosine",
        70
    ),
    e!(
        "arctanh",
        [],
        PureMath,
        "\\operatorname{arctanh}",
        "atanh",
        "arctanh",
        "Arctan hyperbolic",
        "Inverse hyperbolic tangent",
        70
    ),
    e!(
        "arccot",
        [],
        PureMath,
        "\\operatorname{arccot}",
        "arccot",
        "arccot",
        "Arccotangent",
        "Inverse cotangent",
        70
    ),
    e!(
        "arcsec",
        [],
        PureMath,
        "\\operatorname{arcsec}",
        "arcsec",
        "arcsec",
        "Arcsecant",
        "Inverse secant",
        70
    ),
    e!(
        "expfn",
        [],
        PureMath,
        "e^{x}",
        "exp(x)",
        "exp",
        "Hàm số mũ",
        "Exponential function",
        70
    ),
    e!(
        "log2fn",
        [],
        PureMath,
        "\\log_2",
        "calc(log, base: 2)",
        "log₂",
        "Logarit cơ số 2",
        "Base-2 logarithm",
        70
    ),
    e!(
        "hypot",
        [],
        PureMath,
        "\\sqrt{x^2+y^2}",
        "calc(hypot)",
        "hypot",
        "Cạnh huyền tam giác vuông",
        "Euclidean hypotenuse",
        60
    ),
    e!(
        "cross",
        [],
        PureMath,
        "\\times",
        "cross",
        "×",
        "Tích có hướng",
        "Cross product",
        70
    ),
    e!(
        "dotprod",
        [],
        PureMath,
        "\\cdot",
        "dot",
        "·",
        "Tích vô hướng",
        "Dot product",
        70
    ),
    e!(
        "speedlight",
        [],
        PureMath,
        "c",
        "c",
        "c",
        "Tốc độ ánh sáng trong chân không (299792458 m/s)",
        "Speed of light in vacuum",
        90
    ),
    e!(
        "planckconst",
        [],
        PureMath,
        "h",
        "h",
        "h",
        "Hằng số Planck (6.626e-34 J·s)",
        "Planck constant",
        90
    ),
    e!(
        "faradayconst",
        [],
        ElectricalEmbedded,
        "F",
        "F",
        "F",
        "Hằng số Faraday (96485 C/mol)",
        "Faraday constant",
        85
    ),
    e!(
        "rydberg",
        [],
        PureMath,
        "R_\\infty",
        "ry",
        "R∞",
        "Hằng số Rydberg",
        "Rydberg constant",
        80
    ),
    e!(
        "finestruct",
        [],
        PureMath,
        "\\alpha^{-1}",
        "alpha",
        "α⁻¹",
        "Hằng số cấu trúc tinh tế (1/137)",
        "Fine-structure constant",
        80
    ),
    e!(
        "echarge",
        [],
        PhysicsMech,
        "e",
        "e",
        "e",
        "Điện tích nguyên tố (1.602e-19 C)",
        "Elementary charge",
        85
    ),
    e!(
        "electronmass",
        [],
        ElectricalEmbedded,
        "m_{e}",
        "me",
        "mₑ",
        "Khối lượng electron (9.109e-31 kg)",
        "Electron mass",
        80
    ),
    e!(
        "protonmass",
        [],
        ElectricalEmbedded,
        "m_{p}",
        "mp",
        "mₚ",
        "Khối lượng proton (1.673e-27 kg)",
        "Proton mass",
        80
    ),
    e!(
        "neutronmass",
        [],
        ElectricalEmbedded,
        "m_{n}",
        "mn",
        "mₙ",
        "Khối lượng neutron (1.675e-27 kg)",
        "Neutron mass",
        80
    ),
    e!(
        "permeab0",
        [],
        PhysicsMech,
        "\\mu_0",
        "mu.zero",
        "µ₀",
        "Độ từ thẩm chân không",
        "Vacuum permeability",
        80
    ),
    e!(
        "permitt0",
        [],
        PhysicsMech,
        "\\epsilon_0",
        "epsilon.zero",
        "ε₀",
        "Hằng số điện môi chân không",
        "Vacuum permittivity",
        80
    ),
    e!(
        "gravconst",
        [],
        PureMath,
        "G",
        "G",
        "G",
        "Hằng số hấp dẫn (6.674e-11)",
        "Gravitational constant",
        85
    ),
    e!(
        "molarconst",
        [],
        ElectricalEmbedded,
        "R",
        "R",
        "R",
        "Hằng số khí mol (8.314 J/mol·K)",
        "Molar gas constant",
        80
    ),
    e!(
        "wienconst",
        [],
        PureMath,
        "b",
        "b",
        "b",
        "Hằng số dịch chuyển Wien",
        "Wien displacement constant",
        70
    ),
    e!(
        "bohrradius",
        [],
        PureMath,
        "a_{0}",
        "a0",
        "a₀",
        "Bán kính Bohr (0.529 Å)",
        "Bohr radius",
        70
    ),
    e!(
        "hubbleconst",
        [],
        PureMath,
        "H_{0}",
        "H0",
        "H₀",
        "Hằng số Hubble",
        "Hubble constant",
        70
    ),
    e!(
        "stefanconst",
        [],
        PureMath,
        "\\sigma",
        "sigma.sb",
        "σ",
        "Hằng số Stefan-Boltzmann (5.67e-8)",
        "Stefan-Boltzmann constant",
        75
    ),
    e!(
        "compton",
        [],
        PureMath,
        "\\lambda_{C}",
        "lambda.c",
        "λc",
        "Bước sóng Compton",
        "Compton wavelength",
        70
    ),
    e!(
        "kgmol",
        [],
        ElectricalEmbedded,
        "\\mathrm{kg/mol}",
        "kg per mol",
        "kg/mol",
        "Kilôgam trên mol",
        "Kilogram per mole",
        60
    ),
    e!(
        "kWh",
        [],
        PhysicsMech,
        "\\mathrm{kWh}",
        "kWh",
        "kWh",
        "Kilôoát giờ",
        "Kilowatt-hour",
        95
    ),
    e!(
        "mAh",
        [],
        PhysicsMech,
        "\\mathrm{mAh}",
        "mAh",
        "mAh",
        "Miliampe giờ",
        "Milliampere-hour",
        90
    ),
    e!(
        "AmpHour",
        [],
        PhysicsMech,
        "\\mathrm{Ah}",
        "Ah",
        "Ah",
        "Ampe giờ",
        "Ampere-hour",
        85
    ),
    e!(
        "Wh",
        [],
        PhysicsMech,
        "\\mathrm{Wh}",
        "Wh",
        "Wh",
        "Oát giờ",
        "Watt-hour",
        85
    ),
    e!(
        "voltamp",
        [],
        PhysicsMech,
        "\\mathrm{VA}",
        "VA",
        "VA",
        "Vôn-ampe (công suất biểu kiến)",
        "Volt-ampere (apparent power)",
        80
    ),
    e!(
        "kVA",
        [],
        PhysicsMech,
        "\\mathrm{kVA}",
        "kVA",
        "kVA",
        "Kilôvôn-ampe",
        "Kilovolt-ampere",
        80
    ),
    e!(
        "MVA",
        [],
        PhysicsMech,
        "\\mathrm{MVA}",
        "MVA",
        "MVA",
        "Mêgavôn-ampe",
        "Megavolt-ampere",
        75
    ),
    e!(
        "dBW",
        [],
        PhysicsMech,
        "\\mathrm{dBW}",
        "dBW",
        "dBW",
        "Đêxiben so với 1 W",
        "Decibel relative to 1 watt",
        70
    ),
    e!(
        "dBi",
        [],
        PhysicsMech,
        "\\mathrm{dBi}",
        "dBi",
        "dBi",
        "Đêxiben anten đẳng hướng",
        "Decibel isotropic antenna gain",
        70
    ),
    e!(
        "dBc",
        [],
        PhysicsMech,
        "\\mathrm{dBc}",
        "dBc",
        "dBc",
        "Đêxiben so với sóng mang",
        "Decibel relative to carrier",
        65
    ),
    e!(
        "ppt",
        [],
        PhysicsMech,
        "\\mathrm{ppt}",
        "ppt",
        "ppt",
        "Phần nghìn tỉ (10^-12)",
        "Parts per trillion",
        60
    ),
    e!(
        "kmh",
        [],
        PureMath,
        "\\mathrm{km/h}",
        "km per hour",
        "km/h",
        "Kilômét trên giờ",
        "Kilometres per hour",
        90
    ),
    e!(
        "torr",
        [],
        PureMath,
        "\\mathrm{torr}",
        "torr",
        "torr",
        "Torr (đơn vị áp suất)",
        "Torr (pressure unit)",
        70
    ),
    e!(
        "mbar",
        [],
        PureMath,
        "\\mathrm{mbar}",
        "mbar",
        "mbar",
        "Milibar",
        "Millibar",
        75
    ),
    e!(
        "mL",
        [],
        ElectricalEmbedded,
        "\\mathrm{mL}",
        "mL",
        "mL",
        "Mililít",
        "Millilitre",
        90
    ),
    e!(
        "kL",
        [],
        ElectricalEmbedded,
        "\\mathrm{kL}",
        "kL",
        "kL",
        "Kilôlít",
        "Kilolitre",
        60
    ),
    e!(
        "cL",
        [],
        ElectricalEmbedded,
        "\\mathrm{cL}",
        "cL",
        "cL",
        "Xentilít",
        "Centilitre",
        60
    ),
    e!(
        "dL",
        [],
        ElectricalEmbedded,
        "\\mathrm{dL}",
        "dL",
        "dL",
        "Đêxilít",
        "Decilitre",
        65
    ),
    e!(
        "esr",
        [],
        PhysicsMech,
        "\\mathrm{ESR}",
        "esr",
        "ESR",
        "Điện trở nối tiếp tương đương",
        "Equivalent series resistance",
        70
    ),
    e!(
        "cmrr",
        [],
        PhysicsMech,
        "\\mathrm{CMRR}",
        "cmrr",
        "CMRR",
        "Độ khử chế độ chung",
        "Common-mode rejection ratio",
        80
    ),
    e!(
        "psrr",
        [],
        PhysicsMech,
        "\\mathrm{PSRR}",
        "psrr",
        "PSRR",
        "Độ khử nguồn nuôi",
        "Power-supply rejection ratio",
        75
    ),
    e!(
        "thdpct",
        [],
        PhysicsMech,
        "\\mathrm{THD}",
        "thd",
        "THD",
        "Méo dạng tổng hài sóng",
        "Total harmonic distortion",
        75
    ),
    e!(
        "enob",
        [],
        PhysicsMech,
        "\\mathrm{ENOB}",
        "enob",
        "ENOB",
        "Số bit hiệu dụng",
        "Effective number of bits",
        70
    ),
    e!(
        "sfdr",
        [],
        PhysicsMech,
        "\\mathrm{SFDR}",
        "sfdr",
        "SFDR",
        "Dải động không giả tần",
        "Spurious-free dynamic range",
        65
    ),
    e!(
        "carboxyl",
        [],
        ElectricalEmbedded,
        "-\\mathrm{COOH}",
        "COOH",
        "COOH",
        "Nhóm carboxyl",
        "Carboxyl group",
        80
    ),
    e!(
        "aminegroup",
        [],
        ElectricalEmbedded,
        "-\\mathrm{NH_2}",
        "NH2",
        "NH₂",
        "Nhóm amin",
        "Amine group",
        75
    ),
    e!(
        "hydroxyl",
        [],
        ElectricalEmbedded,
        "-\\mathrm{OH}",
        "OH",
        "OH",
        "Nhóm hydroxyl",
        "Hydroxyl group",
        80
    ),
    e!(
        "carbonyl",
        [],
        ElectricalEmbedded,
        "C\\mathrm{=O}",
        "C=O",
        "C=O",
        "Nhóm carbonyl",
        "Carbonyl group",
        75
    ),
    e!(
        "aldehyde",
        [],
        ElectricalEmbedded,
        "-\\mathrm{CHO}",
        "CHO",
        "CHO",
        "Nhóm andehyt",
        "Aldehyde group",
        70
    ),
    e!(
        "estergrp",
        [],
        ElectricalEmbedded,
        "-\\mathrm{COO}-",
        "COO",
        "COO",
        "Nhóm ester",
        "Ester group",
        70
    ),
    e!(
        "ketonegrp",
        [],
        ElectricalEmbedded,
        "\\mathrm{C{=}O}",
        "CO",
        "CO",
        "Nhóm xeton",
        "Ketone group",
        65
    ),
    e!(
        "nitrilegrp",
        [],
        ElectricalEmbedded,
        "-\\mathrm{CN}",
        "CN",
        "CN",
        "Nhóm nitril",
        "Nitrile group",
        65
    ),
    e!(
        "thiolgrp",
        [],
        ElectricalEmbedded,
        "-\\mathrm{SH}",
        "SH",
        "SH",
        "Nhóm thiol",
        "Thiol group",
        65
    ),
    e!(
        "acetyl",
        [],
        ElectricalEmbedded,
        "\\mathrm{CH_3CO}-",
        "CH3CO",
        "CH₃CO",
        "Nhóm acetyl",
        "Acetyl group",
        60
    ),
    e!(
        "vinylgrp",
        [],
        ElectricalEmbedded,
        "-\\mathrm{CH{=}CH_2}",
        "CH2CH",
        "CH₂CH",
        "Nhóm vinyl",
        "Vinyl group",
        60
    ),
    e!(
        "benzylgrp",
        [],
        ElectricalEmbedded,
        "\\mathrm{C_6H_5CH_2}-",
        "C6H5CH2",
        "C₆H₅CH₂",
        "Nhóm benzyl",
        "Benzyl group",
        60
    ),
    e!(
        "amide",
        [],
        ElectricalEmbedded,
        "-\\mathrm{CONH_2}",
        "CONH2",
        "CONH₂",
        "Nhóm amide",
        "Amide group",
        65
    ),
    e!(
        "zscore",
        [],
        QuantFinance,
        "z",
        "z",
        "z",
        "Điểm chuẩn hoá z",
        "Z-score",
        80
    ),
    e!(
        "sharpe",
        [],
        QuantFinance,
        "\\mathrm{Sh}",
        "sharpe",
        "Sharpe",
        "Tỷ số Sharpe",
        "Sharpe ratio",
        85
    ),
    e!(
        "sortino",
        [],
        QuantFinance,
        "\\mathrm{So}",
        "sortino",
        "Sortino",
        "Tỷ số Sortino",
        "Sortino ratio",
        75
    ),
    e!(
        "treynor",
        [],
        QuantFinance,
        "\\mathrm{T}",
        "treynor",
        "Treynor",
        "Tỷ số Treynor",
        "Treynor ratio",
        70
    ),
    e!(
        "jensenalpha",
        [],
        QuantFinance,
        "\\alpha_{J}",
        "alpha.j",
        "αJ",
        "Alpha Jensen",
        "Jensen's alpha",
        70
    ),
    e!(
        "var95",
        [],
        QuantFinance,
        "\\mathrm{VaR}_{95}",
        "var95",
        "VaR95",
        "VaR mức tin cậy 95%",
        "95% Value at Risk",
        85
    ),
    e!(
        "cvar95",
        [],
        QuantFinance,
        "\\mathrm{CVaR}_{95}",
        "cvar95",
        "CVaR95",
        "CVaR (Expected Shortfall) 95%",
        "95% Conditional VaR",
        80
    ),
    e!(
        "maxdrawdown",
        [],
        QuantFinance,
        "\\mathrm{MDD}",
        "mdd",
        "MDD",
        "Mức sụt giảm tối đa",
        "Maximum drawdown",
        80
    ),
    e!(
        "wacc",
        [],
        QuantFinance,
        "\\mathrm{WACC}",
        "wacc",
        "WACC",
        "Chi phí vốn bình quân gia quyền",
        "Weighted average cost of capital",
        75
    ),
    e!(
        "capm",
        [],
        QuantFinance,
        "\\mathrm{CAPM}",
        "capm",
        "CAPM",
        "Mô hình định giá tài sản vốn",
        "Capital asset pricing model",
        75
    ),
    e!(
        "npv",
        [],
        QuantFinance,
        "\\mathrm{NPV}",
        "npv",
        "NPV",
        "Giá trị hiện tại ròng",
        "Net present value",
        80
    ),
    e!(
        "irr",
        [],
        QuantFinance,
        "\\mathrm{IRR}",
        "irr",
        "IRR",
        "Tỷ suất hoàn vốn nội bộ",
        "Internal rate of return",
        75
    ),
    e!(
        "roic",
        [],
        QuantFinance,
        "\\mathrm{ROIC}",
        "roic",
        "ROIC",
        "Tỷ suất lợi nhuận trên vốn đầu tư",
        "Return on invested capital",
        70
    ),
    e!(
        "ebitda",
        [],
        QuantFinance,
        "\\mathrm{EBITDA}",
        "ebitda",
        "EBITDA",
        "Lợi nhuận trước lãi, thuế, khấu hao",
        "EBITDA",
        75
    ),
    e!(
        "pegr",
        [],
        QuantFinance,
        "\\mathrm{PEGR}",
        "pegr",
        "PEGR",
        "Tỷ số P/E trên tăng trưởng",
        "PEG ratio",
        65
    ),
    e!(
        "calmar",
        [],
        QuantFinance,
        "\\mathrm{Calmar}",
        "calmar",
        "Calmar",
        "Tỷ số Calmar",
        "Calmar ratio",
        60
    ),
    e!(
        "ulcerindex",
        [],
        QuantFinance,
        "\\mathrm{UI}",
        "ulcer",
        "UI",
        "Chỉ số loét (rủi ro sụt giảm)",
        "Ulcer index",
        55
    ),
    e!(
        "tailratio",
        [],
        QuantFinance,
        "\\mathrm{TR}",
        "tailratio",
        "TR",
        "Tỷ số đuôi phân phối",
        "Tail ratio",
        55
    ),
    e!(
        "vega",
        [],
        QuantFinance,
        "V",
        "vega",
        "V",
        "Vega (độ nhạy phương sai)",
        "Vega (options greek)",
        70
    ),
    e!(
        "thetaopt",
        [],
        QuantFinance,
        "\\Theta",
        "theta",
        "Θ",
        "Theta (thời gian)",
        "Theta (options greek)",
        70
    ),
    e!(
        "rhoopt",
        [],
        QuantFinance,
        "\\rho",
        "rho",
        "ρ",
        "Rho (lãi suất)",
        "Rho (options greek)",
        65
    ),
    e!(
        "volga",
        [],
        QuantFinance,
        "\\mathrm{Volga}",
        "volga",
        "Volga",
        "Volga (độ nhịy Vega)",
        "Volga (options greek)",
        55
    ),
    e!(
        "mach",
        [],
        PureMath,
        "\\mathrm{Ma}",
        "mach",
        "Ma",
        "Số Mach",
        "Mach number",
        85
    ),
    e!(
        "prandtl",
        [],
        PureMath,
        "\\mathrm{Pr}",
        "pr",
        "Pr",
        "Số Prandtl",
        "Prandtl number",
        75
    ),
    e!(
        "biotnum",
        [],
        PureMath,
        "\\mathrm{Bi}",
        "bi",
        "Bi",
        "Số Biot",
        "Biot number",
        65
    ),
    e!(
        "fouriernum",
        [],
        PureMath,
        "\\mathrm{Fo}",
        "fo",
        "Fo",
        "Số Fourier",
        "Fourier number",
        65
    ),
    e!(
        "grashof",
        [],
        PureMath,
        "\\mathrm{Gr}",
        "gr",
        "Gr",
        "Số Grashof",
        "Grashof number",
        65
    ),
    e!(
        "nusselt",
        [],
        PureMath,
        "\\mathrm{Nu}",
        "nu",
        "Nu",
        "Số Nusselt",
        "Nusselt number",
        75
    ),
    e!(
        "pecklet",
        [],
        PureMath,
        "\\mathrm{Pe}",
        "pe",
        "Pe",
        "Số Péclet",
        "Peclet number",
        65
    ),
    e!(
        "rayleighnum",
        [],
        PureMath,
        "\\mathrm{Ra}",
        "ra",
        "Ra",
        "Số Rayleigh",
        "Rayleigh number",
        65
    ),
    e!(
        "schmidt",
        [],
        PureMath,
        "\\mathrm{Sc}",
        "sc",
        "Sc",
        "Số Schmidt",
        "Schmidt number",
        60
    ),
    e!(
        "lewisnum",
        [],
        PureMath,
        "\\mathrm{Le}",
        "le",
        "Le",
        "Số Lewis",
        "Lewis number",
        60
    ),
    e!(
        "webernum",
        [],
        PureMath,
        "\\mathrm{We}",
        "we",
        "We",
        "Số Weber",
        "Weber number",
        60
    ),
    e!(
        "yieldstrength",
        [],
        PureMath,
        "\\sigma_{y}",
        "sigma.y",
        "σy",
        "Giới hạn chảy",
        "Yield strength",
        75
    ),
    e!(
        "tensile",
        [],
        PureMath,
        "\\sigma_{b}",
        "sigma.b",
        "σb",
        "Giới hạn bền kéo",
        "Ultimate tensile strength",
        70
    ),
    e!(
        "hardness",
        [],
        PureMath,
        "\\mathrm{HB}",
        "hb",
        "HB",
        "Độ cứng Brinell",
        "Brinell hardness HB",
        65
    ),
    e!(
        "brinell",
        [],
        PureMath,
        "\\mathrm{HBW}",
        "hbw",
        "HBW",
        "Độ cứng Brinell HBW",
        "Brinell hardness HBW",
        60
    ),
    e!(
        "vickers",
        [],
        PureMath,
        "\\mathrm{HV}",
        "hv",
        "HV",
        "Độ cứng Vickers",
        "Vickers hardness",
        60
    ),
    e!(
        "rockwellh",
        [],
        PureMath,
        "\\mathrm{HRC}",
        "hrc",
        "HRC",
        "Độ cứng Rockwell C",
        "Rockwell C hardness",
        60
    ),
    e!(
        "charpy",
        [],
        PureMath,
        "\\mathrm{KV}",
        "kv",
        "KV",
        "Năng lượng va đập Charpy",
        "Charpy impact energy",
        55
    ),
    e!(
        "izod",
        [],
        PureMath,
        "\\mathrm{IZ}",
        "izod",
        "IZ",
        "Năng lượng va đập Izod",
        "Izod impact energy",
        55
    ),
    e!(
        "laminar",
        [],
        PureMath,
        "\\mathrm{Re}_{<2300}",
        "laminar",
        "Re<2300",
        "Dòng chảy tầng",
        "Laminar flow",
        50
    ),
    e!(
        "turbulent",
        [],
        PureMath,
        "\\mathrm{Re}_{>4000}",
        "turbulent",
        "Re>4000",
        "Dòng chảy rối",
        "Turbulent flow",
        50
    ),
    e!(
        "doteq",
        [],
        PureMath,
        "\\doteq",
        "dot.eq",
        "≐",
        "Bằng với dấu chấm",
        "Equals with dot",
        70
    ),
    e!(
        "yen",
        [],
        PureMath,
        "\\yen",
        "yen",
        "¥",
        "Ký hiệu yên",
        "Yen sign",
        55
    ),
    e!(
        "dagger",
        [],
        PureMath,
        "\\dagger",
        "dagger",
        "†",
        "Kiềm",
        "Dagger",
        60
    ),
    e!(
        "ddagger",
        [],
        PureMath,
        "\\ddagger",
        "dagger.double",
        "‡",
        "Kiềm kép",
        "Double dagger",
        60
    ),
    e!(
        "mho",
        [],
        PureMath,
        "\\mho",
        "mho",
        "℧",
        "Mho (độ dẫn điện)",
        "Mho (conductance)",
        70
    ),
    e!(
        "clubsuit",
        [],
        PureMath,
        "\\clubsuit",
        "club",
        "♣",
        "Nhánh bích",
        "Club suit",
        45
    ),
    e!(
        "heartsuit",
        [],
        PureMath,
        "\\heartsuit",
        "heart",
        "♥",
        "Nhánh cơ",
        "Heart suit",
        45
    ),
    e!(
        "spadesuit",
        [],
        PureMath,
        "\\spadesuit",
        "spade",
        "♠",
        "Nhánh chuồn",
        "Spade suit",
        45
    ),
    e!(
        "diamondsuit",
        [],
        PureMath,
        "\\diamondsuit",
        "diamond",
        "♢",
        "Nhánh rô",
        "Diamond suit",
        45
    ),
    e!(
        "imath",
        [],
        PureMath,
        "\\imath",
        "dotless.i",
        "ı",
        "i không chấm",
        "Dotless i",
        60
    ),
    e!(
        "jmath",
        [],
        PureMath,
        "\\jmath",
        "dotless.j",
        "ȷ",
        "j không chấm",
        "Dotless j",
        60
    ),
    e!(
        "digamma",
        [],
        PureMath,
        "\\digamma",
        "digamma",
        "ϝ",
        "Digamma",
        "Digamma",
        60
    ),
    e!(
        "gimel",
        [],
        PureMath,
        "\\gimel",
        "gimel",
        "ℷ",
        "Gimel",
        "Gimel",
        55
    ),
    e!(
        "daleth",
        [],
        PureMath,
        "\\daleth",
        "daleth",
        "ℸ",
        "Daleth",
        "Daleth",
        55
    ),
    e!("eth", [], PureMath, "\\eth", "eth", "ð", "Eth", "Eth", 55),
    e!(
        "complementop",
        [],
        PureMath,
        "\\complement",
        "complement",
        "∁",
        "Phần bù tập hợp",
        "Set complement",
        70
    ),
    e!(
        "diagup",
        [],
        PureMath,
        "\\diagup",
        "slash",
        "╱",
        "Gạch chéo lên",
        "Rising diagonal",
        55
    ),
    e!(
        "diagdown",
        [],
        PureMath,
        "\\diagdown",
        "slash.back",
        "╲",
        "Gạch chéo xuống",
        "Falling diagonal",
        55
    ),
    e!(
        "natural",
        [],
        PureMath,
        "\\natural",
        "natural",
        "♮",
        "Dạu giáng tự nhiên",
        "Natural sign",
        50
    ),
    e!(
        "sharp",
        [],
        PureMath,
        "\\sharp",
        "sharp",
        "♯",
        "Dấu thăng",
        "Sharp sign",
        50
    ),
    e!(
        "flat",
        [],
        PureMath,
        "\\flat",
        "flat",
        "♭",
        "Dấu giáng",
        "Flat sign",
        50
    ),
    e!(
        "maltese",
        [],
        PureMath,
        "\\maltese",
        "maltese",
        "✠",
        "Thập tự Malta",
        "Maltese cross",
        40
    ),
    e!(
        "circledast",
        [],
        PureMath,
        "\\circledast",
        "circle-star",
        "⊛",
        "Vòng sao",
        "Circled asterisk",
        60
    ),
    e!(
        "vartriangle",
        [],
        PureMath,
        "\\vartriangle",
        "triangle.small",
        "△",
        "Tam giác nhỏ",
        "Small triangle",
        60
    ),
    e!(
        "triangleq",
        [],
        PureMath,
        "\\triangleq",
        "triangle.eq",
        "≜",
        "Bằng theo định nghĩa",
        "Equal by definition",
        65
    ),
    e!(
        "Bbbk",
        [],
        PureMath,
        "\\Bbbk",
        "bbk",
        "𝕜",
        "Chữ K đậm",
        "Blackboard bold k",
        55
    ),
    e!(
        "multimap",
        [],
        PureMath,
        "\\multimap",
        "multimap",
        "⊸",
        "Ánh xạ",
        "Multimap",
        55
    ),
    e!(
        "pitchfork",
        [],
        PureMath,
        "\\pitchfork",
        "pitchfork",
        "⋔",
        "Pitchfork",
        "Pitchfork",
        55
    ),
    e!(
        "rightleftharpoons",
        [],
        PureMath,
        "\\rightleftharpoons",
        "arrows.rl",
        "⇌",
        "Can bang thuan nghich",
        "Rightleftharpoons (equilibrium)",
        75
    ),
    e!(
        "leftrightharpoons",
        [],
        PureMath,
        "\\leftrightharpoons",
        "arrows.lr",
        "⇋",
        "Can bang trai-phai",
        "Leftrightharpoons",
        50
    ),
    e!(
        "twoheadrightarrow",
        [],
        PureMath,
        "\\twoheadrightarrow",
        "arrow.2head",
        "↠",
        "Mui hai dau",
        "Two-headed arrow",
        50
    ),
    e!(
        "looparrowright",
        [],
        PureMath,
        "\\looparrowright",
        "arrow.loop",
        "↬",
        "Mui vong phai",
        "Loop arrow right",
        50
    ),
    e!(
        "curvearrowright",
        [],
        PureMath,
        "\\curvearrowright",
        "arrow.curve",
        "↷",
        "Mui cong phai",
        "Curved arrow right",
        50
    ),
    e!(
        "upuparrows",
        [],
        PureMath,
        "\\upuparrows",
        "arrow.upup",
        "⇈",
        "Hai mui len",
        "Paired up arrows",
        50
    ),
    e!(
        "rightrightarrows",
        [],
        PureMath,
        "\\rightrightarrows",
        "arrow.rr",
        "⇉",
        "Hai mui phai",
        "Paired right arrows",
        50
    ),
    e!(
        "bigstar",
        [],
        PureMath,
        "\\bigstar",
        "star.big",
        "★",
        "Sao lon",
        "Big star",
        50
    ),
    e!(
        "blacksquare",
        [],
        PureMath,
        "\\blacksquare",
        "rect.fill",
        "■",
        "Vuong dac",
        "Filled square",
        55
    ),
    e!(
        "blacktriangle",
        [],
        PureMath,
        "\\blacktriangle",
        "triangle.fill",
        "▲",
        "Tam giac dac",
        "Filled triangle",
        55
    ),
    e!(
        "bigcirc",
        [],
        PureMath,
        "\\bigcirc",
        "circle.big",
        "◯",
        "Vong tron lon",
        "Big circle",
        50
    ),
    e!(
        "wasit",
        [],
        ElectricalEmbedded,
        "\\rightleftharpoons",
        "arrows.rl",
        "⇌",
        "Phan ung thuan nghich",
        "Reversible reaction",
        85
    ),
];

pub const ALL_ENTRIES: &[&[MathEntry]] = &[
    MATH_SYMBOLS,
    EXTRA_MATH,
    GREEK,
    SUPSUB,
    SI_MICRO_ASCII,
    UNITS_MISC,
    ELEMENTS,
    CHEMISTRY,
    PHYSICS,
    ELECTRICAL,
    CS_AI,
    QUANT,
    FORMULAS,
    TEMPLATE_LEX,
    NOTATION,
];

/// E58: combo SI pico trùng trigger của entry quan trọng hơn với người gõ —
/// "pm" (± plus-minus), "pH" (độ pH hoá học), "ps" (phanso, spec §3.4) —
/// được loại khỏi MỌI bảng tra cứu toàn cục (lookup/sorted/alias/no-dup tests).
/// Dữ liệu nhóm SI vẫn nguyên vẹn 20×28 trong `SI_TABLES` (si_total_560); định dạng
/// 3 combo này vẫn được assert qua test per-entry dùng const của chính nhóm.
const SI_EXCLUDED: &[&str] = &["pm", "pH", "ps"];

/// Toàn bộ slice entry (nhóm thường + 20 nhóm SI, đã lọc SI_EXCLUDED).
pub fn all_slices() -> impl Iterator<Item = &'static [MathEntry]> {
    ALL_ENTRIES.iter().copied().chain(
        SI_TABLES
            .iter()
            .copied()
            .flat_map(|s| s.iter())
            .filter(|e| !SI_EXCLUDED.contains(&e.trigger))
            .map(std::slice::from_ref),
    )
}

/// Tổng số entry.
pub fn total_entries() -> usize {
    all_slices().map(|s| s.len()).sum()
}

/// Bảng phẳng đã sort theo trigger (OnceLock, dựng 1 lần).
fn sorted_entries() -> &'static [&'static MathEntry] {
    static CACHE: OnceLock<Vec<&'static MathEntry>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static MathEntry> = all_slices().flat_map(|s| s.iter()).collect();
        v.sort_by(|a, b| a.trigger.cmp(b.trigger));
        v
    })
}

/// Bảng alias → entry (dựng 1 lần, tra cứu O(len alias list)).
fn alias_index() -> &'static Vec<(&'static str, &'static MathEntry)> {
    static CACHE: OnceLock<Vec<(&'static str, &'static MathEntry)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<(&'static str, &'static MathEntry)> = all_slices()
            .flat_map(|s| s.iter())
            .flat_map(|e| e.aliases.iter().map(move |a| (*a, e)))
            .collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    })
}

/// Tra trigger chính xác — binary search <1 µs.
pub fn lookup_exact(trigger: &str) -> Option<&'static MathEntry> {
    let entries = sorted_entries();
    let idx = entries.partition_point(|e| e.trigger < trigger);
    let cand = entries.get(idx)?;
    if cand.trigger == trigger {
        Some(cand)
    } else {
        None
    }
}

/// Tra prefix — trả tối đa `limit` kết quả theo weight giảm dần.
pub fn lookup_prefix(prefix: &str, limit: usize) -> Vec<&'static MathEntry> {
    let entries = sorted_entries();
    let start = entries.partition_point(|e| e.trigger < prefix);
    let mut out: Vec<&'static MathEntry> = entries[start..]
        .iter()
        .copied()
        .take_while(|e| e.trigger.starts_with(prefix))
        .collect();
    out.sort_by_key(|e| std::cmp::Reverse(e.base_weight));
    out.truncate(limit);
    out
}

/// Tra alias chính xác (case-sensitive) — binary search trên alias index.
pub fn lookup_alias(alias: &str) -> Option<&'static MathEntry> {
    let idx = alias_index();
    let pos = idx.partition_point(|(a, _)| *a < alias);
    let cand = idx.get(pos)?;
    if cand.0 == alias {
        Some(cand.1)
    } else {
        None
    }
}

/// Khớp trigger HOẶC alias (trigger ưu tiên — deterministic).
pub fn lookup_token(token: &str) -> Option<&'static MathEntry> {
    lookup_exact(token).or_else(|| lookup_alias(token))
}

/// Bộ scorer cho palette / gợi ý — FinalScore =
/// PrefixMatchScore + ExactAliasBonus + DisciplineAffinityBoost +
/// RecencyDecayedPersonalWeight (Dream RSI ranking).
pub struct ScoreContext<'a> {
    /// Lĩnh vực user đang làm việc (0.0–1.0 cho từng discipline).
    pub discipline_affinity: &'a [(Discipline, f32)],
    /// Trọng số cá nhân theo trigger (decay theo thời gian).
    pub personal_weights: &'a [(&'static str, f32, u64)],
    /// Timestamp hiện tại (ms).
    pub now_ms: u64,
}

/// Điểm cho 1 entry trong palette — pure function, deterministic.
pub fn score_entry(entry: &MathEntry, query: &str, ctx: &ScoreContext) -> f32 {
    let mut score = 0.0f32;
    if entry.trigger == query {
        score += 100.0;
    } else if entry.trigger.starts_with(query) {
        score += 60.0;
    } else if entry.aliases.contains(&query) {
        score += 80.0; // ExactAliasBonus
    } else if entry.aliases.iter().any(|a| a.starts_with(query)) {
        score += 40.0;
    } else if entry.description_en.to_lowercase().contains(query)
        || entry.description_vi.to_lowercase().contains(query)
    {
        score += 20.0;
    }
    for (d, boost) in ctx.discipline_affinity {
        if *d == entry.discipline {
            score += 15.0 * boost;
            break;
        }
    }
    for (trig, w, last_ms) in ctx.personal_weights {
        if *trig == entry.trigger {
            let age_ms = ctx.now_ms.saturating_sub(*last_ms);
            let decay = (-(age_ms as f32) / 86_400_000.0).exp(); // half-life ~ 1 ngày
            score += 30.0 * w.min(1.0) * decay.max(0.1);
            break;
        }
    }
    score += entry.base_weight as f32 / 100.0;
    score
}

/// Search palette — top-K theo [`score_entry`].
pub fn palette_search(query: &str, limit: usize, ctx: &ScoreContext) -> Vec<&'static MathEntry> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(f32, &'static MathEntry)> = sorted_entries()
        .iter()
        .copied()
        .map(|e| (score_entry(e, q, ctx), e))
        .filter(|(s, _)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.into_iter().map(|(_, e)| e).collect()
}

// ============================================================
// TESTS
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Discipline;

    #[test]
    fn total_entries_at_least_1500() {
        let n = total_entries();
        assert!(n >= 1500, "chỉ có {n} entry — cần ≥1500");
    }

    #[test]
    fn no_duplicate_triggers() {
        let mut seen: Vec<&str> = Vec::new();
        for e in all_slices().flat_map(|s| s.iter()) {
            assert!(!seen.contains(&e.trigger), "trigger trùng: {}", e.trigger);
            seen.push(e.trigger);
        }
    }

    #[test]
    fn no_duplicate_aliases() {
        let mut seen: Vec<&str> = Vec::new();
        for e in all_slices().flat_map(|s| s.iter()) {
            for a in e.aliases {
                assert!(!seen.contains(a), "alias trùng: {a} (entry {})", e.trigger);
                seen.push(a);
            }
        }
    }

    #[test]
    fn alias_never_collides_with_other_trigger() {
        for (a, entry) in alias_index() {
            if let Some(owner) = lookup_exact(a) {
                assert!(
                    std::ptr::eq(owner, *entry),
                    "alias {a} của {} trùng trigger của {}",
                    entry.trigger,
                    owner.trigger
                );
            }
        }
    }

    #[test]
    fn every_entry_has_valid_fields() {
        for e in all_slices().flat_map(|s| s.iter()) {
            assert!(!e.trigger.is_empty(), "trigger rỗng");
            assert!(e.trigger.len() <= 32);
            assert!(!e.trigger.starts_with('\\'));
            assert!(!e.trigger.chars().any(|c| c.is_whitespace() || c == '\\'));
            assert!(!e.latex.is_empty());
            assert!(!e.typst.is_empty());
            assert!(!e.unicode.is_empty());
            assert!(!e.description_vi.is_empty());
            assert!(!e.description_en.is_empty());
            assert!(e.base_weight > 0 && e.base_weight <= 100);
            for a in e.aliases {
                assert!(!a.is_empty());
                assert!(!a.chars().any(|c| c.is_whitespace() || c == '\\'));
            }
        }
    }

    #[test]
    fn all_disciplines_covered() {
        for d in Discipline::all() {
            let n = ALL_ENTRIES
                .iter()
                .flat_map(|s| s.iter())
                .filter(|e| e.discipline == *d)
                .count();
            assert!(n >= 40, "discipline {d:?} chỉ có {n} entry");
        }
    }

    #[test]
    fn lookup_exact_binary_search() {
        let e = lookup_exact("alpha").unwrap();
        assert_eq!(e.unicode, "α");
        assert_eq!(e.latex, "\\alpha");
        assert!(lookup_exact("zzz_khong_ton_tai").is_none());
        assert!(lookup_exact("").is_none());
    }

    #[test]
    fn lookup_alias_vietnamese() {
        let e = lookup_alias("tong").unwrap();
        assert_eq!(e.unicode, "∑");
        let e = lookup_alias("can").unwrap();
        assert_eq!(e.unicode, "√x");
        assert!(lookup_alias("zzz_alias_khong").is_none());
    }

    #[test]
    fn lookup_token_trigger_first() {
        // trigger "pi" có sẵn — token "pi" phải trả entry trigger, không phải alias
        let e = lookup_token("pi").unwrap();
        assert_eq!(e.trigger, "pi");
    }

    #[test]
    fn lookup_prefix_weighted() {
        let res = lookup_prefix("mat", 10);
        assert!(!res.is_empty());
        assert!(res.iter().all(|e| e.trigger.starts_with("mat")));
        // weight giảm dần
        for w in res.windows(2) {
            assert!(w[0].base_weight >= w[1].base_weight);
        }
    }

    #[test]
    fn lookup_prefix_limit() {
        let res = lookup_prefix("a", 5);
        assert!(res.len() <= 5);
    }

    #[test]
    fn si_micro_ascii_aliases_resolve() {
        for t in ["um", "uF", "uH", "uV", "uA"] {
            assert!(lookup_exact(t).is_some(), "missing {t}");
        }
        let e = lookup_alias("uf").unwrap();
        assert_eq!(e.unicode, "µF");
    }

    #[test]
    fn elements_symbols_resolve_as_alias() {
        // E58: alias nguyên tố là tên VI — ký hiệu "Mg"/"Pm"/"Tm"/"Re"/"Es"/"Ts"/...
        // trùng trigger SI/physics nên bị loại theo invariant alias-never-collides.
        let e = lookup_alias("sắt").unwrap();
        assert_eq!(e.trigger, "iron");
        let e = lookup_alias("heli").unwrap();
        assert_eq!(e.trigger, "helium");
        let e = lookup_alias("hydro").unwrap();
        assert_eq!(e.trigger, "hydrogen");
    }

    #[test]
    fn template_lex_entries_resolve() {
        for t in ["mat2x2", "mat3x3", "det3", "hpt2", "cases2"] {
            assert!(lookup_exact(t).is_some(), "missing {t}");
        }
    }

    #[test]
    fn palette_search_ranks_exact_first() {
        let ctx = ScoreContext {
            discipline_affinity: &[],
            personal_weights: &[],
            now_ms: 0,
        };
        let res = palette_search("alpha", 10, &ctx);
        assert!(!res.is_empty());
        assert_eq!(res[0].trigger, "alpha");
    }

    #[test]
    fn palette_search_discipline_boost() {
        let ctx = ScoreContext {
            discipline_affinity: &[(Discipline::ChemistryBio, 1.0)],
            personal_weights: &[],
            now_ms: 0,
        };
        let res = palette_search("h", 10, &ctx);
        assert!(!res.is_empty(), "query 'h' phải có kết quả");
    }

    #[test]
    fn palette_search_personal_recency() {
        let e = lookup_exact("alpha").unwrap();
        let ctx = ScoreContext {
            discipline_affinity: &[],
            personal_weights: &[(e.trigger, 1.0, 1000)],
            now_ms: 2000,
        };
        let scored = score_entry(e, "al", &ctx);
        let ctx_no = ScoreContext {
            discipline_affinity: &[],
            personal_weights: &[],
            now_ms: 0,
        };
        assert!(
            scored > score_entry(e, "al", &ctx_no),
            "personal boost phải tăng điểm"
        );
    }

    #[test]
    fn palette_search_empty_query() {
        let ctx = ScoreContext {
            discipline_affinity: &[],
            personal_weights: &[],
            now_ms: 0,
        };
        assert!(palette_search("", 10, &ctx).is_empty());
        assert!(palette_search("   ", 10, &ctx).is_empty());
    }

    #[test]
    fn spec_vn_aliases_present() {
        // Danh sách alias tiếng Việt bắt buộc theo spec §3.4
        for a in [
            "can",
            "ps",
            "phanso",
            "tp",
            "tichphan",
            "tong",
            "vc",
            "vocuc",
            "suyra",
            "khac",
            "nhohon",
            "lonhon",
            "vuonggoc",
            "songsong",
            "goc",
            "do",
            "duongkinh",
            "ohm",
        ] {
            assert!(lookup_alias(a).is_some(), "thiếu alias VN '{a}'");
        }
    }

    #[test]
    fn binary_search_consistency() {
        // Partition point phải đưa đúng vị trí cho mọi prefix 1 ký tự
        let entries = sorted_entries();
        assert!(entries.is_sorted_by(|a, b| a.trigger <= b.trigger));
    }
}
